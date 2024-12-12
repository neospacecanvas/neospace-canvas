use std::io::Cursor;

use csv::Reader;
// Core logic in pure Rust - no WASM dependencies
use wasm_bindgen::prelude::*;

pub fn process_csv_internal(csv_data: String) -> Result<String, String> {
    let cursor = Cursor::new(csv_data);
    let mut reader = Reader::from_reader(cursor);
    let mut output = String::new();
    let mut row_num = 0;

    for result in reader.records() {
        match result {
            Ok(record) => {
                row_num += 1;
                output.push_str(&format!("\nRow {}: {:?}", row_num, record));
                for (column_index, field) in record.iter().enumerate() {
                    output.push_str(&format!("  Column {}: {}\n", column_index + 1, field));
                }
                output.push_str("------------------\n");
            }
            Err(e) => return Err(format!("error reading row: {}", e)),
        }
    }
    Ok(output)
}

// WASM wrapper
#[wasm_bindgen]
pub fn read_csv(csv_data: String) -> Result<String, JsValue> {
    process_csv_internal(csv_data).map_err(|e| JsValue::from_str(&e))
}

/// seperate pure rust tests from the webassembly tests
#[cfg(test)]
mod tests {

    use super::*;

    const TEST_CSV: &str = "name,age\nJohn,30\nJane,25";

    // Pure Rust tests in tests/rust_tests.rs
    #[test]
    fn test_csv_processing() {
        let result = process_csv_internal(TEST_CSV.to_string()).unwrap();
        println!("output: {}", result);
        assert!(result.contains("John"));
    }
}

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    const TEST_CSV: &str = "name,age\nJohn,30\nJane,25";

    #[wasm_bindgen_test]
    fn test_wasm_csv() {
        let result = read_csv(TEST_CSV.to_string()).unwrap();
        assert!(result.contains("John"));
    }
}
