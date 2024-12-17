// crate level declarations

use std::io::Cursor;

use csv::Reader;
// Core logic in pure Rust - no WASM dependencies
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

mod csv;

pub fn process_csv_internal(csv_data: String) -> Result<String, String> {
    // Log input in both environments
    #[cfg(not(target_arch = "wasm32"))]
    println!("Input CSV data: {}", csv_data);
    #[cfg(target_arch = "wasm32")]
    console::log_1(&format!("Input CSV data: {}", csv_data).into());

    let cursor = Cursor::new(csv_data);
    let mut reader = Reader::from_reader(cursor);
    let mut output = String::new();
    let mut row_num = 0;

    for result in reader.records() {
        match result {
            Ok(record) => {
                row_num += 1;
                output.push_str(&format!("\nRow {}: {:?}", row_num, record));
                // Log each row processing
                #[cfg(target_arch = "wasm32")]
                console::log_1(&format!("Processing row {}: {:?}", row_num, record).into());
            }
            Err(e) => {
                let error_msg = format!("error reading row: {}", e);
                #[cfg(target_arch = "wasm32")]
                console::log_1(&format!("Error: {}", error_msg).into());
                return Err(error_msg);
            }
        }
    }

    // Log output in both environments
    #[cfg(not(target_arch = "wasm32"))]
    println!("Output: {}", output);
    #[cfg(target_arch = "wasm32")]
    console::log_1(&format!("Output: {}", output).into());

    Ok(output)
}

#[wasm_bindgen]
pub fn read_csv(csv_data: String) -> Result<String, JsValue> {
    #[cfg(target_arch = "wasm32")]
    console::log_1(&"Starting CSV processing".into());

    let result = process_csv_internal(csv_data).map_err(|e| JsValue::from_str(&e));

    #[cfg(target_arch = "wasm32")]
    console::log_1(&format!("CSV processing result: {:?}", result).into());

    result
}

/// ==================
/// Tests:
/// =================

/// seperate pure rust tests from the webassembly tests
#[cfg(test)]
mod tests {

    use types::CSV;

    use super::*;

    const TEST_CSV: &str = "name,age\nJohn,30\nJane,25";

    // Pure Rust tests in tests/rust_tests.rs
    #[test]
    fn test_csv_processing() {
        let result = process_csv_internal(TEST_CSV.to_string()).unwrap();
        println!("output: {}", result);
        assert!(result.contains("John"));
    }

    #[test]
    fn test_csv_from_string() {
        // unwrap gives the actual CSV not Option
        let csv = CSV::from_string(TEST_CSV.to_string()).unwrap();
        assert_eq!(csv.row_count(), 2);
        assert_eq!(csv.column_count(), 2);
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
