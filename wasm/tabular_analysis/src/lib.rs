use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// the csv file will be passed in as a string
/// it is critical to use regular expressions to detect the most likely datatype
/// we need to read a column, decide what datatype it fits into, and what types it doesn't
/// we also need to decide if there is likely some type of error

pub enum ColumnDatatypes {
    INTEGER,
    DECIMAL,
    CURRENCY,
    PERCENTAGE,
    DATE,
    DATETIME,
    TIME,
    CATEGORICAL,
    TEXT,
    EMAIL,
    PHONE
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