use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DataType {
    Integer,
    Decimal,
    Currency,
    Date,
    Email,
    Phone,
    Categorical,
    Text
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
    sample_values: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericStats {
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    std_dev: f64,
    quartiles: [f64; 3]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextStats {
    min_length: usize,
    max_length: usize,
    avg_length: f64,
    most_common: Vec<(String, usize)>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anomaly {
    row_index: usize,
    value: String,
    expected_type: DataType,
    found_type: DataType,
    suggestion: Option<String>
}

#[derive(Debug, Serialize)]
pub struct CSVFile {
    columns: Vec<ColumnMetadata>,
    row_count: usize,
    suggested_sql: String,
}