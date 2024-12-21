use crate::types::DataType;
use csv::Reader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug)]
pub struct CSV {
    columns: Vec<Column>,
    row_count: usize,
}

#[derive(Debug)]
pub struct Column {
    header: String,
    values: Vec<String>,
    metadata: Option<ColumnMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnMetadata {
    name: String,
    data_type: DataType,
    confidence: f64,
}

impl CSV {
    pub fn from_string(raw_data: String) -> Result<Self, String> {
        let cursor = Cursor::new(raw_data);
        let mut reader = Reader::from_reader(cursor);

        // use csv reader to extract headers
        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| format!("failed to read headers: {}", e))?
            .iter()
            .map(|header| header.to_string())
            .collect();

        let column_count = headers.len();

        let mut columns: Vec<Column> = headers
            .into_iter()
            .map(|header| Column {
                header,
                values: Vec::new(),
                metadata: None,
            })
            .collect();

        // reading data in column mahor format!
        for result in reader.records() {
            match result {
                Ok(record) => {
                    // iterate through rows and add columns to respective columns
                    for (i, field) in record.iter().enumerate() {
                        if i < column_count {
                            columns[i].values.push(field.to_string());
                        }
                    }
                }
                Err(e) => return Err(format!("error reading row: {}", e)),
            }
        }

        let row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        Ok(CSV { columns, row_count })
    }

    // getters
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn headers(&self) -> Vec<String> {
        self.columns.iter().map(|col| col.header.clone()).collect()
    }

    pub fn get_column_values(&self, index: usize) -> Option<&[String]> {
        self.columns.get(index).map(|col| col.values.as_slice())
    }

    // Get all column values for parallel processing
    pub fn get_columns(&self) -> Vec<(&str, &[String])> {
        self.columns
            .iter()
            .map(|col| (col.header.as_str(), col.values.as_slice()))
            .collect()
    }

    // Set analyzed metadata for a column
    pub fn set_column_metadata(&mut self, index: usize, metadata: ColumnMetadata) {
        if let Some(column) = self.columns.get_mut(index) {
            column.metadata = Some(metadata);
        }
    }

    // Get current metadata for a column
    pub fn get_column_metadata(&self, index: usize) -> Option<&ColumnMetadata> {
        self.columns.get(index)?.metadata.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_from_string() {
        let data = "header1,header2,header3\nvalue1,value2,value3\nvalue4,value5,value6";
        let csv = CSV::from_string(data.to_string()).unwrap();

        assert_eq!(csv.row_count(), 2);
        assert_eq!(csv.column_count(), 3);
        assert_eq!(csv.headers(), vec!["header1", "header2", "header3"]);

        // Test column values
        assert_eq!(csv.get_column_values(0).unwrap(), &["value1", "value4"]);
        assert_eq!(csv.get_column_values(1).unwrap(), &["value2", "value5"]);
        assert_eq!(csv.get_column_values(2).unwrap(), &["value3", "value6"]);
    }
}
