use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// the csv file will be passed in as a string
/// it is critical to use regular expressions to detect the most likely datatype
/// we need to read a column, decide what datatype it fits into, and what types it doesn't
/// we also need to decide if there is likely some type of error
#[derive(Debug, Serialize, Deserialize)]    
pub enum ColumnDatatypes {
    INTEGER,
    DECIMAL {precision: u8, scale: u8},
    CURRENCY,
    PERCENTAGE,
    DATE,
    DATETIME,
    TIME,
    CATEGORICAL,
    TEXT { max_length: usize},
    EMAIL,
    PHONE
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericStats { 
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    std_dev: f64,
    quartiles: [f64; 3],
    distinct_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextStats {
    min_length: usize,
    max_length: usize,
    avg_length: f64,
    distict_count: usize,
    most_common: Vec<(String, usize)>,
}



/// this struct is for keeping track of stats
/// on a column's metadata as its read
struct ColumnStats {

}
/// the CSV data representation
#[wasm_bindgen]
pub struct TabularDataAnalyzer {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_stats: HashMap<String, ColumnStats>
}