// csv.rs

// Import core functionality for CSV parsing and type detection
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

// Import our type detection system
use crate::types::{type_scoring::TypeScores, DataType, TypeDetection};

// ColumnMetadata represents the analyzed properties of a CSV column
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: DataType,
    pub confidence: f64,
}

// CSV struct represents a parsed CSV file with type information
#[wasm_bindgen]
#[derive(Debug)]
pub struct CSV {
    columns: Vec<Column>,
    row_count: usize,
}

// Column represents a single column of data in the CSV
#[derive(Debug)]
struct Column {
    header: String,
    values: Vec<String>,
    metadata: Option<ColumnMetadata>,
}

// Implement core CSV functionality
#[wasm_bindgen]
impl CSV {
    // Constructor that creates a CSV from a string
    #[wasm_bindgen(constructor)]
    pub fn from_string(raw_data: String) -> Result<CSV, JsError> {
        // Create a cursor for reading the string data
        let cursor = Cursor::new(raw_data);
        let mut reader = Reader::from_reader(cursor);

        // Read headers from the CSV
        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| JsError::new(&format!("Failed to read headers: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect();

        // Initialize columns with headers
        let mut columns: Vec<Column> = headers
            .into_iter()
            .map(|header| Column {
                header,
                values: Vec::new(),
                metadata: None,
            })
            .collect();

        // Read all records and populate column values
        for result in reader.records() {
            match result {
                Ok(record) => {
                    for (i, field) in record.iter().enumerate() {
                        if i < columns.len() {
                            columns[i].values.push(field.to_string());
                        }
                    }
                }
                Err(e) => return Err(JsError::new(&format!("Error reading row: {}", e))),
            }
        }

        // Calculate row count from the first column (all columns should have same length)
        let row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        Ok(CSV { columns, row_count })
    }

    // Get the number of rows in the CSV
    #[wasm_bindgen]
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    // Get the number of columns in the CSV
    #[wasm_bindgen]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    // Get the headers of the CSV
    #[wasm_bindgen]
    pub fn headers(&self) -> Result<JsValue, JsError> {
        let headers = self
            .columns
            .iter()
            .map(|col| col.header.clone())
            .collect::<Vec<String>>();

        to_value(&headers).map_err(|e| JsError::new(&format!("Failed to serialize headers: {}", e)))
    }

    // Internal helper to get a column's data
    pub(crate) fn get_column(&self, index: usize) -> Option<(&str, &[String])> {
        self.columns
            .get(index)
            .map(|col| (col.header.as_str(), col.values.as_slice()))
    }

    // Internal helper to get all columns
    pub(crate) fn get_columns(&self) -> Vec<(&str, &[String])> {
        self.columns
            .iter()
            .map(|col| (col.header.as_str(), col.values.as_slice()))
            .collect()
    }

    #[wasm_bindgen]
    pub fn infer_column_types(&mut self) -> Result<(), JsError> {
        for i in 0..self.column_count() {
            if let Some((header, values)) = self.get_column(i) {
                // First pass: use TypeScores to get initial type analysis
                let scores = TypeScores::from_column(values);
                let (initial_type, confidence) = scores.best_type();

                // Second pass: enhance type detection with additional analysis
                let final_type = if initial_type == DataType::Text {
                    self.analyze_potential_categorical_data(values)
                        .unwrap_or(DataType::Text)
                } else {
                    initial_type
                };

                // Create and store the column metadata
                let metadata = ColumnMetadata {
                    name: header.to_string(),
                    data_type: final_type,
                    confidence,
                };

                let js_metadata = to_value(&metadata)
                    .map_err(|e| JsError::new(&format!("Failed to serialize metadata: {}", e)))?;
                self.set_column_metadata(i, js_metadata)?;
            }
        }
        Ok(())
    }

    /// Sets metadata for a specific column
    #[wasm_bindgen]
    pub fn set_column_metadata(
        &mut self,
        index: usize,
        js_metadata: JsValue,
    ) -> Result<(), JsError> {
        let metadata: ColumnMetadata = from_value(js_metadata)
            .map_err(|e| JsError::new(&format!("Failed to deserialize metadata: {}", e)))?;

        if let Some(column) = self.columns.get_mut(index) {
            column.metadata = Some(metadata);
            Ok(())
        } else {
            Err(JsError::new("Column index out of bounds"))
        }
    }

    /// Retrieves metadata for a specific column
    #[wasm_bindgen]
    pub fn get_column_metadata(&self, index: usize) -> Result<JsValue, JsError> {
        let metadata = self
            .columns
            .get(index)
            .and_then(|col| col.metadata.as_ref())
            .ok_or_else(|| JsError::new("No metadata found for column"))?;

        to_value(&metadata)
            .map_err(|e| JsError::new(&format!("Failed to serialize metadata: {}", e)))
    }

    /// Advanced analysis for potential categorical data
    fn analyze_potential_categorical_data(&self, values: &[String]) -> Option<DataType> {
        // Skip analysis if we don't have enough data
        if values.len() < 20 {
            return None;
        }

        // Calculate unique value statistics
        use std::collections::HashMap;
        let mut value_counts: HashMap<&str, usize> = HashMap::new();
        let mut non_empty_count = 0;

        for value in values {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                *value_counts.entry(trimmed).or_insert(0) += 1;
                non_empty_count += 1;
            }
        }

        // Calculate metrics for categorical detection
        let unique_count = value_counts.len();
        let unique_ratio = unique_count as f64 / non_empty_count as f64;

        // Check average value length to avoid treating long text as categorical
        let avg_length: f64 =
            value_counts.keys().map(|s| s.len()).sum::<usize>() as f64 / unique_count as f64;

        // Check frequency distribution
        let min_frequency = 3;
        let frequent_values = value_counts
            .values()
            .filter(|&&count| count >= min_frequency)
            .count();
        let frequency_ratio = frequent_values as f64 / unique_count as f64;

        // Decision criteria for categorical data:
        // 1. Low ratio of unique values (< 5%)
        // 2. Values aren't too long (< 50 chars on average)
        // 3. Most values appear multiple times
        if unique_ratio < 0.05 && avg_length < 50.0 && frequency_ratio > 0.7 {
            Some(DataType::Categorical)
        } else {
            None
        }
    }

    /// Retrieves a summary of the CSV structure and types
    #[wasm_bindgen]
    pub fn get_structure_summary(&self) -> Result<JsValue, JsError> {
        let summary = self
            .columns
            .iter()
            .map(|col| {
                let metadata = col.metadata.as_ref().map(|m| (m.data_type, m.confidence));
                (
                    col.header.clone(),
                    col.values.len(),
                    metadata.map(|(t, c)| (t.to_string(), c)),
                )
            })
            .collect::<Vec<_>>();

        to_value(&summary).map_err(|e| JsError::new(&format!("Failed to serialize summary: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // Basic CSV functionality tests
    #[test]
    fn test_csv_parsing() {
        // Test basic CSV parsing with standard data
        let data = "header1,header2\nvalue1,value2\nvalue4,value5";
        let csv = CSV::from_string(data.to_string()).unwrap();
        assert_eq!(csv.column_count(), 2);
        assert_eq!(csv.row_count(), 2);

        let (header, values) = csv.get_column(0).unwrap();
        assert_eq!(header, "header1");
        assert_eq!(values, &["value1", "value4"]);

        // Test CSV with empty lines and whitespace
        let data = "header1,header2\nvalue1,value2\n\nvalue4,value5\n";
        let csv = CSV::from_string(data.to_string()).unwrap();
        assert_eq!(csv.row_count(), 3); // Empty line is still a row
    }

    // Numeric type detection tests
    #[wasm_bindgen_test]
    fn test_numeric_detection() {
        // Test integer detection
        let data = "numbers\n123\n456\n789\n1,234\n-5,678";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Integer);
        assert!(metadata.confidence > 0.9);

        // Test decimal detection
        let data = "decimals\n123.45\n456.78\n789.01\n1,234.56\n-5,678.90";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Decimal);
        assert!(metadata.confidence > 0.9);
    }

    // Currency detection tests
    #[wasm_bindgen_test]
    fn test_currency_detection() {
        let data = "amounts\n$1,234.56\n$2,345.67\n$3,456.78\nUSD 4,567.89\n$-1,234.56";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Currency);
        assert!(metadata.confidence > 0.9);

        // Test with some missing currency symbols
        let data = "amounts\n$1,234.56\n2,345.67\n$3,456.78\n4,567.89";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        // Should still detect as currency if pattern is consistent enough
        assert_eq!(metadata.data_type, DataType::Currency);
    }

    // Date format detection tests
    #[wasm_bindgen_test]
    fn test_date_detection() {
        // Test ISO format dates
        let data = "dates\n2024-01-01\n2024-02-15\n2024-03-30";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Date);
        assert!(metadata.confidence > 0.9);

        // Test mixed date formats
        let data = "dates\n2024-01-01\n01/15/2024\n2024/01/30\n2024-02-01";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Date);
        // Confidence might be lower with mixed formats but should still be reasonable
        assert!(metadata.confidence > 0.7);
    }

    // Email format detection tests
    #[wasm_bindgen_test]
    fn test_email_detection() {
        let data =
            "emails\nuser@example.com\nname.surname@domain.co.uk\ntest123@subdomain.site.com";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Email);
        assert!(metadata.confidence > 0.9);

        // Test with some invalid emails mixed in
        let data = "emails\nuser@example.com\ninvalid.email\ntest@domain.com";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        // Should fall back to Text if too many invalid emails
        assert!(matches!(
            metadata.data_type,
            DataType::Email | DataType::Text
        ));
    }

    // Phone number detection tests
    #[wasm_bindgen_test]
    fn test_phone_detection() {
        let data = "phones\n(123) 456-7890\n123-456-7890\n1234567890\n+1-123-456-7890";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Phone);
        assert!(metadata.confidence > 0.8);

        // Test international formats
        let data = "phones\n+44 20 7123 4567\n+1 (123) 456-7890\n+61 2 8123 4567";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Phone);
    }

    // Categorical data detection tests
    #[wasm_bindgen_test]
    fn test_categorical_detection() {
        // Test obvious categorical data
        let data = "status\nactive\npending\nactive\npending\nactive\ncompleted";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Categorical);

        // Test with larger number of categories but still categorical
        let mut data = String::from("priority\n");
        for _ in 0..100 {
            data.push_str("High\nMedium\nLow\nCritical\n");
        }

        let mut csv = CSV::from_string(data).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Categorical);
    }

    // Multiple column type detection tests
    #[wasm_bindgen_test]
    fn test_multiple_columns() {
        let data = "id,name,email,status,amount\n\
                   1,John Smith,john@test.com,active,$1,234.56\n\
                   2,Jane Doe,jane@test.com,pending,$2,345.67\n\
                   3,Bob Wilson,bob@test.com,completed,$3,456.78";

        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        // Check each column's type
        let id_meta: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(id_meta.data_type, DataType::Integer);

        let name_meta: ColumnMetadata = from_value(csv.get_column_metadata(1).unwrap()).unwrap();
        assert_eq!(name_meta.data_type, DataType::Text);

        let email_meta: ColumnMetadata = from_value(csv.get_column_metadata(2).unwrap()).unwrap();
        assert_eq!(email_meta.data_type, DataType::Email);

        let status_meta: ColumnMetadata = from_value(csv.get_column_metadata(3).unwrap()).unwrap();
        assert_eq!(status_meta.data_type, DataType::Categorical);

        let amount_meta: ColumnMetadata = from_value(csv.get_column_metadata(4).unwrap()).unwrap();
        assert_eq!(amount_meta.data_type, DataType::Currency);
    }

    // Data quality and edge case tests
    #[wasm_bindgen_test]
    fn test_data_quality_handling() {
        // Test handling of missing values
        let data = "values\n123\n\n456\n\t\n789\n  \n";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(
            metadata.data_type,
            DataType::Integer,
            "Should handle empty/whitespace values"
        );

        // Test handling of quoted values
        let data = "text,\"header,with,comma\"\n\
                   value1,\"value,with,commas\"\n\
                   value2,\"another,quoted,value\"";
        let csv = CSV::from_string(data.to_string());
        assert!(csv.is_ok(), "Should handle quoted values with commas");
    }

    // Unicode and special character handling tests
    #[wasm_bindgen_test]
    fn test_special_characters() {
        // Test Unicode in text fields
        let data = "description\n🌟 Special offer!\n⭐ Featured item\n❤️ Popular choice";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Text);

        // Test special characters in categorical data
        let data = "status\n★ Gold\n★ Gold\n☆ Silver\n★ Gold\n☆ Silver";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Categorical);
    }

    // Error handling tests
    #[wasm_bindgen_test]
    fn test_error_handling() {
        // Test invalid column index
        let data = "header\nvalue";
        let csv = CSV::from_string(data.to_string()).unwrap();
        assert!(csv.get_column_metadata(999).is_err());

        // Test completely empty CSV
        let data = "";
        assert!(CSV::from_string(data.to_string()).is_err());

        // Test headers only
        let data = "header1,header2";
        let csv = CSV::from_string(data.to_string()).unwrap();
        assert_eq!(csv.row_count(), 0);
    }
}
