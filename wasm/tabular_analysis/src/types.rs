use csv::Reader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DataType {
    Integer,
    Decimal,
    Currency,
    Date,
    Email,
    Phone,
    Categorical,
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnMetadata {
    name: String,
    data_type: DataType,
    confidence: f64,
    row_count: usize,
    null_count: usize,
    distinct_count: usize,
    numeric_stats: Option<NumericStats>,
    text_stats: Option<TextStats>,
    format_pattern: Option<String>,
    anomalies: Vec<Anomaly>,
    sql_type: String,
    sample_values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericStats {
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    std_dev: f64,
    quartiles: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextStats {
    min_length: usize,
    max_length: usize,
    avg_length: f64,
    most_common: Vec<(String, usize)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anomaly {
    row_index: usize,
    value: String,
    expected_type: DataType,
    found_type: DataType,
    suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CSVFile {
    columns: Vec<ColumnMetadata>,
    row_count: usize,
    suggested_sql: String,
}

//TODO: make a structure for reading in the CSV and iterating by row or col
#[derive(Debug)]
pub struct CSV {
    data: Arc<Vec<Vec<String>>>,
    headers: Arc<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
    thread_count: Option<usize>,
}

#[derive(Debug)]
pub struct Column<'a> {
    header: &'a str,
    data: Arc<Vec<Vec<String>>>,
    column_index: usize,
}

//TODO: be sure to include warnings in the CSV impl for when the
// user has provided one that is too small for the statistical processing
// to be of real use.
impl CSV {
    pub fn from_string(raw_data: String) -> Result<Self, String> {
        let cursor = Cursor::new(raw_data);
        let mut reader = Reader::from_reader(cursor);

        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| format!("Failed to read headers: {}", e))?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let column_count = headers.len();

        let mut data: Vec<Vec<String>> = Vec::new();
        // for each line in reader
        for line in reader.records() {
            // will either give error or Ok
            match line {
                Ok(line) => {
                    // if ok we want to add a row
                    let row: Vec<String> = line.iter().map(|field| field.to_string()).collect();
                    data.push(row);
                }
                Err(e) => return Err(format!("Error reading row: {}", e)),
            }
        }
        let row_count = data.len();
        Ok(CSV {
            data: Arc::new(data),
            headers: Arc::new(headers),
            row_count,
            column_count,
            thread_count: None,
        })
    }

    pub fn with_thread_count(mut self, threads: usize) -> Self {
        self.thread_count = Some(threads);
        self
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column_count(&self) -> usize {
        self.column_count
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    pub fn get_column(&self, index: usize) -> Option<Column> {
        if index >= self.column_count() {
            return None;
        }
        Some(Column {
            header: &self.headers[index],
            data: Arc::clone(&self.data),
            column_index: index,
        })
    }
}
