// Import our new type scoring module
use crate::types::{type_scoring::TypeScores, DataType};
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::io::Cursor;
use wasm_bindgen::prelude::*; // Add explicit import for to_value

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: DataType,
    pub confidence: f64,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct CSV {
    columns: Vec<Column>,
    row_count: usize,
}

#[derive(Debug)]
struct Column {
    header: String,
    values: Vec<String>,
    metadata: Option<ColumnMetadata>,
}

#[wasm_bindgen]
impl CSV {
    #[wasm_bindgen(constructor)]
    pub fn from_string(raw_data: String) -> Result<CSV, JsError> {
        let cursor = Cursor::new(raw_data);
        let mut reader = Reader::from_reader(cursor);

        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| JsError::new(&format!("Failed to read headers: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut columns: Vec<Column> = headers
            .into_iter()
            .map(|header| Column {
                header,
                values: Vec::new(),
                metadata: None,
            })
            .collect();

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

        let row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        Ok(CSV { columns, row_count })
    }

    #[wasm_bindgen]
    pub fn infer_column_types(&mut self) -> Result<(), JsError> {
        for i in 0..self.column_count() {
            if let Some((header, values)) = self.get_column(i) {
                // Use our new TypeScores to analyze the column
                let scores = TypeScores::from_column(values);
                let (data_type, confidence) = scores.best_type();

                let metadata = ColumnMetadata {
                    name: header.to_string(),
                    data_type,
                    confidence,
                };

                let js_metadata = to_value(&metadata)
                    .map_err(|e| JsError::new(&format!("Failed to serialize metadata: {}", e)))?;
                self.set_column_metadata(i, js_metadata)?;
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    #[wasm_bindgen]
    pub fn headers(&self) -> Result<JsValue, JsError> {
        let headers = self
            .columns
            .iter()
            .map(|col| col.header.clone())
            .collect::<Vec<String>>();
        to_value(&headers).map_err(|e| JsError::new(&format!("Failed to serialize headers: {}", e)))
    }

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

    #[wasm_bindgen]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn get_column(&self, index: usize) -> Option<(&str, &[String])> {
        self.columns
            .get(index)
            .map(|col| (col.header.as_str(), col.values.as_slice()))
    }

    pub(crate) fn get_columns(&self) -> Vec<(&str, &[String])> {
        self.columns
            .iter()
            .map(|col| (col.header.as_str(), col.values.as_slice()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[test]
    fn test_csv_from_string() {
        let data = "header1,header2\nvalue1,value2\nvalue4,value5";
        let csv = CSV::from_string(data.to_string()).unwrap();
        assert_eq!(csv.column_count(), 2);
        let (header, values) = csv.get_column(0).unwrap();
        assert_eq!(header, "header1");
        assert_eq!(values, &["value1", "value4"]);
    }

    #[wasm_bindgen_test]
    fn test_type_inference() {
        // Test numeric column
        let data = "numbers\n123\n456\n789";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Integer);
        assert!(metadata.confidence > 0.9);

        // Test mixed types
        let data = "mixed\n123\nabc\n456";
        let mut csv = CSV::from_string(data.to_string()).unwrap();
        csv.infer_column_types().unwrap();

        let metadata: ColumnMetadata = from_value(csv.get_column_metadata(0).unwrap()).unwrap();
        assert_eq!(metadata.data_type, DataType::Text);
    }
}
