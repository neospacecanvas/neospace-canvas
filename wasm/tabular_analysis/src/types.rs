use csv::Reader;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::{ErrorEvent, MessageEvent, Worker}; // ErrorEvent gets used in the wasm only

// Message types for worker communication
#[derive(Serialize, Deserialize)]
struct WorkerMessage {
    column_index: usize,
    column_data: Vec<String>,
    header: String,
}

#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    column_index: usize,
    metadata: ColumnMetadata,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
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
#[wasm_bindgen]
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

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmColumn {
    header: String,
    data: Arc<Vec<Vec<String>>>,
    column_index: usize,
}

#[wasm_bindgen]
impl WasmColumn {
    pub fn header(&self) -> String {
        self.header.clone()
    }

    pub fn get_values(&self) -> Vec<String> {
        self.data
            .iter()
            .map(|row| row[self.column_index].clone())
            .collect()
    }

    pub fn index(&self) -> usize {
        self.column_index
    }
}

//TODO: be sure to include warnings in the CSV impl for when the
// user has provided one that is too small for the statistical processing
// to be of real use.
#[wasm_bindgen]
impl CSV {
    #[wasm_bindgen(constructor)]
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

    // WASM-specific async implementation
    #[cfg(target_arch = "wasm32")]
    pub async fn analyze_with_workers(&self) -> Result<JsValue, JsValue> {
        use futures::channel::mpsc::{channel, Sender};
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::JsCast;
        use web_sys::{MessageEvent, Worker};

        let available_threads = web_sys::window()
            .unwrap()
            .navigator()
            .hardware_concurrency() as usize;

        let num_workers = match self.thread_count {
            Some(count) => count,
            None => (available_threads.saturating_sub(1)).max(1),
        };

        let (sender, mut receiver) = channel::<WorkerResponse>(self.column_count);
        let mut workers: Vec<Worker> = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let worker = Worker::new("./worker.js")
                .map_err(|e| JsValue::from_str(&format!("Failed to create worker: {:?}", e)))?;
            workers.push(worker);
        }

        let completed_count = Rc::new(RefCell::new(0));
        let total_columns = self.column_count;

        let chunks = self.distribute_work(num_workers);

        for (worker_idx, chunk) in chunks.iter().enumerate() {
            let worker = &workers[worker_idx];
            let sender = sender.clone();
            let completed_count = Rc::clone(&completed_count);

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let response: WorkerResponse =
                    serde_wasm_bindgen::from_value(event.data()).unwrap();

                let mut count = completed_count.borrow_mut();
                *count += 1;

                let sender = sender.clone();
                spawn_local(async move {
                    sender
                        .clone()
                        .send(response)
                        .await
                        .expect("Failed to send worker response");
                });
            }) as Box<dyn FnMut(MessageEvent)>);

            worker.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();

            let error_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                web_sys::console::error_1(&format!("Worker error: {:?}", e).into());
            }) as Box<dyn FnMut(ErrorEvent)>);

            worker.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
            error_callback.forget();

            for &col_idx in chunk {
                let message = WorkerMessage {
                    column_index: col_idx,
                    column_data: self.get_column(col_idx).unwrap().get_values(),
                    header: self.headers[col_idx].clone(),
                };

                worker
                    .post_message(&serde_wasm_bindgen::to_value(&message)?)
                    .map_err(|e| {
                        JsValue::from_str(&format!("Failed to post message to worker: {:?}", e))
                    })?;
            }
        }

        let mut columns = vec![None; self.column_count];
        let mut received = 0;

        while received < total_columns {
            if let Some(response) = receiver.next().await {
                columns[response.column_index] = Some(response.metadata);
                received += 1;
            }
        }

        for worker in workers {
            worker.terminate();
        }

        let analysis = CSVFile {
            columns: columns.into_iter().filter_map(|x| x).collect(),
            row_count: self.row_count,
            suggested_sql: self
                .generate_sql_schema(&columns.into_iter().filter_map(|x| x).collect::<Vec<_>>()),
        };

        serde_wasm_bindgen::to_value(&analysis)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize analysis: {}", e)))
    }
    // Helper method to distribute column indices among workers
    fn distribute_work(&self, num_workers: usize) -> Vec<Vec<usize>> {
        let mut chunks = vec![Vec::new(); num_workers];

        // Distribute column indices using round-robin
        for col_idx in 0..self.column_count {
            chunks[col_idx % num_workers].push(col_idx);
        }

        chunks
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

    pub fn headers(&self) -> Vec<String> {
        (*self.headers).clone()
    }

    pub fn get_column(&self, index: usize) -> Option<WasmColumn> {
        if index >= self.column_count() {
            return None;
        }
        Some(WasmColumn {
            header: self.headers[index].clone(),
            data: Arc::clone(&self.data),
            column_index: index,
        })
    }

    fn analyze_single_column(&self, column: Column) -> ColumnMetadata {
        // Get values for this column
        let values: Vec<&str> = self
            .data
            .iter()
            .map(|row| row[column.column_index].as_str())
            .collect();

        // Initial type inference with confidence
        let (inferred_type, confidence) = self.infer_type(&values);

        // Count distinct values and nulls
        let mut value_set = std::collections::HashSet::new();
        let mut null_count = 0;

        for &value in &values {
            if value.trim().is_empty() {
                null_count += 1;
            } else {
                value_set.insert(value);
            }
        }

        // Get sample values (up to 5 distinct values)
        let sample_values: Vec<String> = value_set.iter().take(5).map(|&s| s.to_string()).collect();

        // Collect statistics based on the inferred type
        let (numeric_stats, text_stats) = match inferred_type {
            DataType::Integer | DataType::Decimal | DataType::Currency => {
                (self.calculate_numeric_stats(&values), None)
            }
            DataType::Text | DataType::Email | DataType::Phone | DataType::Categorical => {
                (None, self.calculate_text_stats(&values))
            }
            DataType::Date => (None, None), // Date stats could be added later
        };

        // Find anomalies
        let anomalies = self.detect_anomalies(&values, &inferred_type);

        // Determine SQL type
        let sql_type =
            self.determine_sql_type(&inferred_type, &numeric_stats, &text_stats, null_count > 0);

        // Detect format pattern if applicable
        let format_pattern = match inferred_type {
            DataType::Date => Some(self.detect_date_format(&values)),
            DataType::Phone => Some(self.detect_phone_format(&values)),
            DataType::Currency => Some(self.detect_currency_format(&values)),
            _ => None,
        };

        ColumnMetadata {
            name: column.header.to_string(),
            data_type: inferred_type,
            confidence,
            row_count: values.len(),
            null_count,
            distinct_count: value_set.len(),
            numeric_stats,
            text_stats,
            format_pattern,
            anomalies,
            sql_type,
            sample_values,
        }
    }

    fn detect_date_format(&self, values: &[&str]) -> String {
        let mut format_counts: HashMap<&str, usize> = HashMap::new();

        for &value in values {
            if value.trim().is_empty() {
                continue;
            }

            let format = if value.contains('-') {
                if value.matches('-').count() == 2 {
                    if value.starts_with(char::is_numeric) {
                        if value.len() == 10 && value[0..4].chars().all(char::is_numeric) {
                            "YYYY-MM-DD"
                        } else {
                            "DD-MM-YYYY"
                        }
                    } else {
                        "MON-DD-YYYY"
                    }
                } else {
                    "UNKNOWN"
                }
            } else if value.contains('/') {
                if value.matches('/').count() == 2 {
                    if value.starts_with(char::is_numeric) {
                        if value.len() == 10 && value[0..4].chars().all(char::is_numeric) {
                            "YYYY/MM/DD"
                        } else {
                            "MM/DD/YYYY"
                        }
                    } else {
                        "MON/DD/YYYY"
                    }
                } else {
                    "UNKNOWN"
                }
            } else {
                "UNKNOWN"
            };

            *format_counts.entry(format).or_insert(0) += 1;
        }

        // Return the most common format
        format_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(format, _)| format.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn detect_phone_format(&self, values: &[&str]) -> String {
        let mut format_counts: HashMap<&str, usize> = HashMap::new();

        for &value in values {
            if value.trim().is_empty() {
                continue;
            }

            let format = if value.contains('(') && value.contains(')') {
                "(###) ###-####"
            } else if value.starts_with('+') {
                "+# ### ### ####"
            } else if value.contains('-') {
                "###-###-####"
            } else {
                "##########"
            };

            *format_counts.entry(format).or_insert(0) += 1;
        }

        format_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(format, _)| format.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn detect_currency_format(&self, values: &[&str]) -> String {
        let mut format_counts: HashMap<&str, usize> = HashMap::new();

        for &value in values {
            if value.trim().is_empty() {
                continue;
            }

            let format = if value.starts_with('$') {
                "$#,###.##"
            } else if value.starts_with('€') {
                "€#,###.##"
            } else if value.starts_with('£') {
                "£#,###.##"
            } else if value.ends_with("USD") {
                "#,###.## USD"
            } else if value.ends_with("EUR") {
                "#,###.## EUR"
            } else if value.ends_with("GBP") {
                "#,###.## GBP"
            } else {
                "#,###.##"
            };

            *format_counts.entry(format).or_insert(0) += 1;
        }

        format_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(format, _)| format.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn calculate_numeric_stats(&self, values: &[&str]) -> Option<NumericStats> {
        // Convert valid numbers to f64, filtering out non-numeric values
        let numbers: Vec<f64> = values
            .iter()
            .filter_map(|&v| {
                let cleaned = v.trim().replace(',', ""); // Remove commas from numbers
                if cleaned.is_empty() {
                    return None;
                }
                // Try to parse as number, ignoring currency symbols
                cleaned
                    .trim_start_matches(['$', '€', '£'])
                    .trim()
                    .parse::<f64>()
                    .ok()
            })
            .collect();

        if numbers.is_empty() {
            return None;
        }

        // Create sorted copy for percentile calculations
        let mut sorted = numbers.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = numbers.len();

        // Calculate mean
        let mean = numbers.iter().sum::<f64>() / len as f64;

        // Calculate median and quartiles
        let median = sorted[len / 2];
        let quartiles = [
            sorted[len / 4],     // Q1
            median,              // Q2
            sorted[3 * len / 4], // Q3
        ];

        // Calculate standard deviation
        let variance = numbers.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (len - 1) as f64;
        let std_dev = variance.sqrt();

        Some(NumericStats {
            min: *sorted.first().unwrap(),
            max: *sorted.last().unwrap(),
            mean,
            median,
            std_dev,
            quartiles,
        })
    }

    fn calculate_text_stats(&self, values: &[&str]) -> Option<TextStats> {
        let non_empty_values: Vec<&str> = values
            .iter()
            .map(|&v| v.trim())
            .filter(|&v| !v.is_empty())
            .collect();

        if non_empty_values.is_empty() {
            return None;
        }

        // Calculate length statistics
        let lengths: Vec<usize> = non_empty_values.iter().map(|s| s.len()).collect();

        let min_length = *lengths.iter().min().unwrap();
        let max_length = *lengths.iter().max().unwrap();
        // Calculate average length
        let avg_length = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

        // Find most common values and their frequencies
        let mut value_counts: HashMap<&str, usize> = HashMap::new();
        for &value in &non_empty_values {
            *value_counts.entry(value).or_insert(0) += 1;
        }

        // Sort by frequency and take top 5
        let mut most_common: Vec<(String, usize)> = value_counts
            .iter()
            .map(|(&k, &v)| (k.to_string(), v))
            .collect();

        most_common.sort_by(|a, b| {
            b.1.cmp(&a.1) // Sort by count descending
                .then(a.0.cmp(&b.0)) // Then by value ascending for stability
        });
        most_common.truncate(5);

        Some(TextStats {
            min_length,
            max_length,
            avg_length,
            most_common,
        })
    }

    // Helper function to safely calculate percentile
    fn percentile(sorted_values: &[f64], p: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let len = sorted_values.len();
        let rank = (p * (len - 1) as f64).round() as usize;
        sorted_values[rank.min(len - 1)]
    }

    // Helper function to check if value might be numeric
    fn might_be_numeric(value: &str) -> bool {
        let cleaned = value.trim().replace(',', "");
        if cleaned.is_empty() {
            return false;
        }

        cleaned
            .trim_start_matches(['$', '€', '£'])
            .trim()
            .parse::<f64>()
            .is_ok()
    }
    fn detect_anomalies(&self, values: &[&str], expected_type: &DataType) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // For empty columns, no anomalies to detect
        if values.is_empty() {
            return anomalies;
        }

        for (index, &value) in values.iter().enumerate() {
            // Skip empty/null values as they're handled separately
            if value.trim().is_empty() {
                continue;
            }

            // Detect actual type of this value
            let (found_type, confidence) = self.detect_single_value_type(value);

            // If types don't match and we're confident about the found type,
            // this is an anomaly
            if found_type != *expected_type && confidence > 0.7 {
                let suggestion = self.suggest_correction(value, expected_type);
                anomalies.push(Anomaly {
                    row_index: index,
                    value: value.to_string(),
                    expected_type: expected_type.clone(),
                    found_type,
                    suggestion,
                });
            }
        }

        anomalies
    }

    fn detect_single_value_type(&self, value: &str) -> (DataType, f64) {
        let value = value.trim();

        // Check against each type pattern
        for (data_type, patterns) in TYPE_PATTERNS.iter() {
            for pattern in patterns {
                if pattern.is_match(value) {
                    return (data_type.clone(), 1.0);
                }
            }
        }

        // Additional type checks for less strict matches
        if self.looks_like_number(value) {
            return (DataType::Decimal, 0.8);
        }

        if self.looks_like_date(value) {
            return (DataType::Date, 0.8);
        }

        // Default to Text with low confidence
        (DataType::Text, 0.5)
    }

    fn suggest_correction(&self, value: &str, expected_type: &DataType) -> Option<String> {
        match expected_type {
            DataType::Integer => {
                // Try to extract just the numbers
                let nums: String = value
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '-')
                    .collect();
                nums.parse::<i64>().ok().map(|n| n.to_string())
            }

            DataType::Decimal | DataType::Currency => {
                // Remove currency symbols and normalize decimals
                let cleaned = value.trim_start_matches(['$', '€', '£']).replace(',', "");
                cleaned.parse::<f64>().ok().map(|n| {
                    if *expected_type == DataType::Currency {
                        format!("${:.2}", n)
                    } else {
                        format!("{:.2}", n)
                    }
                })
            }

            DataType::Date => self.normalize_date(value),

            DataType::Phone => self.normalize_phone(value),

            DataType::Email => self.normalize_email(value),

            _ => None,
        }
    }

    fn looks_like_number(&self, value: &str) -> bool {
        let cleaned = value.replace(',', "");
        cleaned.parse::<f64>().is_ok()
    }

    fn looks_like_date(&self, value: &str) -> bool {
        // Simple check for date-like patterns
        let numbers: Vec<&str> = value.split(|c| c == '/' || c == '-' || c == '.').collect();

        if numbers.len() != 3 {
            return false;
        }

        numbers.iter().all(|&n| n.parse::<u32>().is_ok())
    }

    fn normalize_date(&self, value: &str) -> Option<String> {
        // Split on common date separators
        let parts: Vec<&str> = value.split(|c| c == '/' || c == '-' || c == '.').collect();

        if parts.len() != 3 {
            return None;
        }

        // Try to determine the format based on the values
        let (year, month, day) = self.guess_date_parts(&parts)?;

        // Return ISO formatted date
        Some(format!("{:04}-{:02}-{:02}", year, month, day))
    }

    fn guess_date_parts(&self, parts: &[&str]) -> Option<(u32, u32, u32)> {
        let numbers: Vec<u32> = parts.iter().filter_map(|&s| s.parse().ok()).collect();

        if numbers.len() != 3 {
            return None;
        }

        // If one number is clearly a year (>31)
        let year_pos = numbers.iter().position(|&n| n > 31)?;

        let year = numbers[year_pos];
        let mut other_nums = numbers.clone();
        other_nums.remove(year_pos);

        // Assume first number is month in ambiguous cases
        let (month, day) = if other_nums[0] <= 12 {
            (other_nums[0], other_nums[1])
        } else {
            (other_nums[1], other_nums[0])
        };

        // Validate ranges
        if month == 0 || month > 12 || day == 0 || day > 31 {
            return None;
        }

        // Convert 2-digit years
        let year = if year < 100 {
            if year < 50 {
                year + 2000
            } else {
                year + 1900
            }
        } else {
            year
        };

        Some((year, month, day))
    }

    fn normalize_phone(&self, value: &str) -> Option<String> {
        // Extract just the digits
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() >= 10 {
            let d = digits.as_str();
            Some(format!(
                "({}) {}-{}",
                &d[d.len() - 10..d.len() - 7],
                &d[d.len() - 7..d.len() - 4],
                &d[d.len() - 4..]
            ))
        } else {
            None
        }
    }

    fn normalize_email(&self, value: &str) -> Option<String> {
        let lower = value.to_lowercase();
        // Check basic email pattern
        if lower.contains('@') && lower.contains('.') {
            Some(lower)
        } else {
            None
        }
    }

    fn determine_sql_type(
        &self,
        data_type: &DataType,
        numeric_stats: &Option<NumericStats>,
        text_stats: &Option<TextStats>,
        nullable: bool,
    ) -> String {
        let base_type = match data_type {
            DataType::Integer => {
                // Use numeric stats to determine appropriate integer size
                if let Some(stats) = numeric_stats {
                    if stats.min >= i32::MIN as f64 && stats.max <= i32::MAX as f64 {
                        if stats.min >= 0.0 {
                            if stats.max <= u16::MAX as f64 {
                                "SMALLINT UNSIGNED".to_string()
                            } else {
                                "INT UNSIGNED".to_string()
                            }
                        } else {
                            "INT".to_string()
                        }
                    } else {
                        "BIGINT".to_string()
                    }
                } else {
                    "INT".to_string()
                }
            }
            DataType::Decimal => {
                if let Some(stats) = numeric_stats {
                    // Analyze decimal places needed
                    let max_decimal_places = self.calculate_max_decimal_places(stats);
                    let max_total_digits = self.calculate_total_digits(stats);
                    format!(
                        "DECIMAL({},{})",
                        max_total_digits.min(65),   // MySQL max precision
                        max_decimal_places.min(30)  // MySQL max scale
                    )
                } else {
                    "DECIMAL(10,2)".to_string() // Default if no stats available
                }
            }
            DataType::Currency => {
                "DECIMAL(19,4)".to_string() // Standard for currency
            }
            DataType::Date => "DATE".to_string(),
            DataType::Email => {
                if let Some(stats) = text_stats {
                    format!("VARCHAR({})", stats.max_length.min(255))
                } else {
                    "VARCHAR(255)".to_string() // Standard email length limit
                }
            }
            DataType::Phone => "VARCHAR(20)".to_string(), // Standard international phone length
            DataType::Categorical => {
                if let Some(stats) = text_stats {
                    if stats.max_length <= 1 {
                        "CHAR(1)".to_string()
                    } else if stats.most_common.len() <= 10 && stats.max_length <= 50 {
                        // For small sets of values, consider ENUM
                        self.generate_enum_type(&stats.most_common)
                    } else {
                        format!("VARCHAR({})", stats.max_length.min(255))
                    }
                } else {
                    "VARCHAR(50)".to_string() // Default for categorical
                }
            }
            DataType::Text => {
                if let Some(stats) = text_stats {
                    if stats.max_length <= 255 {
                        format!("VARCHAR({})", stats.max_length)
                    } else if stats.max_length <= 65535 {
                        "TEXT".to_string()
                    } else if stats.max_length <= 16777215 {
                        "MEDIUMTEXT".to_string()
                    } else {
                        "LONGTEXT".to_string()
                    }
                } else {
                    "TEXT".to_string() // Default if no stats
                }
            }
        };

        // Add NULL constraint if needed
        if nullable {
            format!("{} NULL", base_type)
        } else {
            format!("{} NOT NULL", base_type)
        }
    }

    fn calculate_max_decimal_places(&self, stats: &NumericStats) -> usize {
        let mut max_places = 0;

        // Helper function to count decimal places
        let mut count_decimals = |n: f64| {
            let s = format!("{:.10}", n); // Use high precision for checking
            if let Some(pos) = s.find('.') {
                let decimals = s[pos + 1..].trim_end_matches('0').len();
                max_places = max_places.max(decimals);
            }
        };

        // Check all significant values
        count_decimals(stats.min);
        count_decimals(stats.max);
        count_decimals(stats.mean);

        max_places
    }

    fn calculate_total_digits(&self, stats: &NumericStats) -> usize {
        let max_abs = stats.max.abs().max(stats.min.abs());
        let whole_digits = (max_abs.log10().floor() as i32 + 1).max(1) as usize;
        let decimal_places = self.calculate_max_decimal_places(stats);

        whole_digits + decimal_places
    }

    fn generate_enum_type(&self, values: &[(String, usize)]) -> String {
        let enum_values = values
            .iter()
            .map(|(val, _)| format!("'{}'", val.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");

        format!("ENUM({})", enum_values)
    }

    // Helper function to format nullability
    fn format_with_null(&self, base_type: &str, nullable: bool) -> String {
        if nullable {
            format!("{} NULL", base_type)
        } else {
            format!("{} NOT NULL", base_type)
        }
    }

    fn get_column_data(&self, col_idx: usize) -> Vec<String> {
        self.data.iter().map(|row| row[col_idx].clone()).collect()
    }

    fn generate_sql_schema(&self, columns: &[ColumnMetadata]) -> String {
        let mut sql = String::new();

        // Add main table creation
        sql.push_str("-- Main table creation\n");
        sql.push_str(&format!("CREATE TABLE analyzed_data (\n"));

        // Add columns
        for (i, col) in columns.iter().enumerate() {
            // Main column definition
            sql.push_str(&format!(
                "    {} {}",
                self.escape_identifier(&col.name),
                col.sql_type
            ));

            // Add null constraint
            if col.null_count == 0 {
                sql.push_str(" NOT NULL");
            }

            // Add comments for high anomaly counts or low confidence
            let mut comments = Vec::new();

            if col.confidence < 0.9 {
                comments.push(format!("type confidence: {:.1}%", col.confidence * 100.0));
            }

            if !col.anomalies.is_empty() {
                comments.push(format!("{} anomalies", col.anomalies.len()));
            }

            if !comments.is_empty() {
                sql.push_str(&format!(" -- {}", comments.join(", ")));
            }

            // Add comma if not last column
            if i < columns.len() - 1 {
                sql.push_str(",");
            }
            sql.push_str("\n");
        }

        sql.push_str(");\n\n");

        // Add indexes if appropriate
        sql.push_str("-- Recommended indexes\n");
        for col in columns {
            if self.should_create_index(col) {
                sql.push_str(&format!(
                    "CREATE INDEX idx_{} ON analyzed_data ({});\n",
                    self.normalize_identifier(&col.name),
                    self.escape_identifier(&col.name)
                ));
            }
        }

        // Add comments about data quality
        sql.push_str("\n-- Data Quality Notes:\n");
        for col in columns {
            if !col.anomalies.is_empty() || col.confidence < 0.9 {
                sql.push_str(&format!("-- Column '{}':\n", col.name));

                if col.confidence < 0.9 {
                    sql.push_str(&format!(
                        "--   Low type confidence ({:.1}%)\n",
                        col.confidence * 100.0
                    ));
                }

                if !col.anomalies.is_empty() {
                    sql.push_str(&format!("--   {} anomalies found\n", col.anomalies.len()));
                    // List first few anomalies as examples
                    for anomaly in col.anomalies.iter().take(3) {
                        sql.push_str(&format!(
                            "--     Row {}: '{}' (expected {}, found {})\n",
                            anomaly.row_index + 1,
                            anomaly.value,
                            format!("{:?}", anomaly.expected_type).to_lowercase(),
                            format!("{:?}", anomaly.found_type).to_lowercase()
                        ));
                    }
                }
            }
        }

        sql
    }

    fn should_create_index(&self, col: &ColumnMetadata) -> bool {
        // Create index if:
        // 1. Column has high cardinality (many unique values)
        // 2. Not too many nulls
        // 3. Appropriate type for indexing
        let unique_ratio = col.distinct_count as f64 / col.row_count as f64;
        let null_ratio = col.null_count as f64 / col.row_count as f64;

        match col.data_type {
            // Good candidates for indexing
            DataType::Integer | DataType::Date | DataType::Email => {
                unique_ratio > 0.1 && null_ratio < 0.5
            }
            // Categorical data with enough distinct values
            DataType::Categorical => {
                col.distinct_count > 1 && col.distinct_count <= 1000 && null_ratio < 0.3
            }
            // Don't index text fields unless they're likely foreign keys
            DataType::Text => {
                unique_ratio > 0.5
                    && col.distinct_count > 1
                    && col.distinct_count <= 10000
                    && null_ratio < 0.1
            }
            // Other types generally don't need indexes
            _ => false,
        }
    }

    fn escape_identifier(&self, name: &str) -> String {
        // Escape SQL identifiers to handle special characters and reserved words
        format!("`{}`", name.replace("`", "``"))
    }

    fn normalize_identifier(&self, name: &str) -> String {
        // Create a valid identifier for index names
        name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }

    fn infer_type(&self, values: &[&str]) -> (DataType, f64) {
        let non_empty_values: Vec<&str> = values
            .iter()
            .filter(|&&v| !v.trim().is_empty())
            .copied()
            .collect();

        if non_empty_values.is_empty() {
            return (DataType::Text, 1.0);
        }

        // Track matches for each type
        let mut matches = HashMap::new();
        let total_values = non_empty_values.len();

        // First pass: Check all specific patterns for each value
        for &value in &non_empty_values {
            let value = value.trim();

            // Order of checking from most specific to most general:

            // 1. Currency (most specific number format with symbols)
            if let Some(currency_patterns) = TYPE_PATTERNS.get(&DataType::Currency) {
                if currency_patterns
                    .iter()
                    .any(|pattern| pattern.is_match(value))
                {
                    *matches.entry(DataType::Currency).or_insert(0) += 1;
                    continue; // Move to next value if currency is found
                }
            }

            // 2. Phone (specific format with symbols)
            if let Some(phone_patterns) = TYPE_PATTERNS.get(&DataType::Phone) {
                if phone_patterns.iter().any(|pattern| pattern.is_match(value)) {
                    *matches.entry(DataType::Phone).or_insert(0) += 1;
                    continue;
                }
            }

            // 3. Email (specific format with @ and domain)
            if let Some(email_patterns) = TYPE_PATTERNS.get(&DataType::Email) {
                if email_patterns.iter().any(|pattern| pattern.is_match(value)) {
                    *matches.entry(DataType::Email).or_insert(0) += 1;
                    continue;
                }
            }

            // 4. Date (specific format with separators)
            if let Some(date_patterns) = TYPE_PATTERNS.get(&DataType::Date) {
                if date_patterns.iter().any(|pattern| pattern.is_match(value)) {
                    *matches.entry(DataType::Date).or_insert(0) += 1;
                    continue;
                }
            }

            // 5. Decimal (numbers with decimal point)
            if let Some(decimal_patterns) = TYPE_PATTERNS.get(&DataType::Decimal) {
                if decimal_patterns
                    .iter()
                    .any(|pattern| pattern.is_match(value))
                {
                    *matches.entry(DataType::Decimal).or_insert(0) += 1;
                    continue;
                }
            }

            // 6. Integer (whole numbers)
            if let Some(integer_patterns) = TYPE_PATTERNS.get(&DataType::Integer) {
                if integer_patterns
                    .iter()
                    .any(|pattern| pattern.is_match(value))
                {
                    *matches.entry(DataType::Integer).or_insert(0) += 1;
                    continue;
                }
            }

            // 7. Check for categorical (limited set of repeating values)
            if self.could_be_categorical(value) {
                *matches.entry(DataType::Categorical).or_insert(0) += 1;
                continue;
            }

            // 8. If nothing else matches, it's text (most general)
            *matches.entry(DataType::Text).or_insert(0) += 1;
        }

        // Special case for categorical data
        if self.is_likely_categorical(&non_empty_values) {
            let confidence = self.calculate_categorical_confidence(&non_empty_values);
            if confidence > 0.8 {
                return (DataType::Categorical, confidence);
            }
        }

        // Find the most specific type that matches all or most values
        let type_ordering = [
            DataType::Currency,
            DataType::Phone,
            DataType::Email,
            DataType::Date,
            DataType::Decimal,
            DataType::Integer,
            DataType::Categorical,
            DataType::Text,
        ];

        for data_type in type_ordering.iter() {
            if let Some(&count) = matches.get(data_type) {
                let confidence = count as f64 / total_values as f64;
                // If we have a high confidence match, use this type
                if confidence > 0.8 {
                    return (data_type.clone(), confidence);
                }
            }
        }

        // If no type has high confidence, find the most specific type with the most matches
        let (best_type, count) = matches
            .iter()
            .max_by_key(|(data_type, &count)| {
                // Create a tuple of (count, priority) where priority is the reverse index in type_ordering
                let priority = type_ordering.len()
                    - type_ordering
                        .iter()
                        .position(|t| t == *data_type)
                        .unwrap_or(0);
                (count, priority)
            })
            .unwrap_or((&DataType::Text, &0));

        (best_type.clone(), *count as f64 / total_values as f64)
    }
    fn could_be_categorical(&self, value: &str) -> bool {
        // Quick check for common categorical values
        let value_lower = value.to_lowercase();
        matches!(
            value_lower.as_str(),
            "yes"
                | "no"
                | "true"
                | "false"
                | "high"
                | "medium"
                | "low"
                | "1"
                | "2"
                | "3"
                | "4"
                | "5"
                | "active"
                | "inactive"
                | "pending"
                | "completed"
                | "cancelled"
        )
    }

    fn is_likely_categorical(&self, values: &[&str]) -> bool {
        let total_values = values.len();
        if total_values < 2 {
            return false;
        }

        // Count unique values
        let unique_values: std::collections::HashSet<_> = values.iter().copied().collect();
        let unique_ratio = unique_values.len() as f64 / total_values as f64;

        // Criteria for categorical:
        // 1. Low cardinality relative to total values
        // 2. Reasonable number of total values for statistics
        unique_ratio < 0.05 && total_values >= 20
    }

    fn calculate_categorical_confidence(&self, values: &[&str]) -> f64 {
        let total_values = values.len() as f64;
        let unique_values: std::collections::HashSet<_> = values.iter().copied().collect();

        // Factors affecting confidence:
        // 1. Ratio of unique values to total values
        let unique_ratio = unique_values.len() as f64 / total_values;
        let cardinality_score = (1.0 - unique_ratio).max(0.0);

        // 2. Consistency of values
        let categorical_matches = values
            .iter()
            .filter(|&&v| self.could_be_categorical(v))
            .count() as f64;
        let consistency_score = categorical_matches / total_values;

        // Weighted average of scores
        0.7 * cardinality_score + 0.3 * consistency_score
    }
}

static TYPE_PATTERNS: Lazy<HashMap<DataType, Vec<Regex>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Integer patterns
    m.insert(
        DataType::Integer,
        vec![
            Regex::new(r"^-?\d+$").unwrap(),
            Regex::new(r"^-?\d{1,3}(,\d{3})*$").unwrap(),
        ],
    );

    // Decimal/Float patterns
    m.insert(
        DataType::Decimal,
        vec![
            Regex::new(r"^-?\d*\.\d+$").unwrap(),
            Regex::new(r"^-?\d{1,3}(,\d{3})*\.\d+$").unwrap(),
            Regex::new(r"^-?\d+\.\d*$").unwrap(),
        ],
    );

    // Currency patterns
    m.insert(
        DataType::Currency,
        vec![
            // Dollar with commas
            Regex::new(r"^\$\s*\d{1,3}(,\d{3})*(\.\d{2})?$").unwrap(),
            // Euro with commas
            Regex::new(r"^€\s*\d{1,3}(,\d{3})*(\.\d{2})?$").unwrap(),
            // Pound with commas
            Regex::new(r"^£\s*\d{1,3}(,\d{3})*(\.\d{2})?$").unwrap(),
            // Simple currency (no commas)
            Regex::new(r"^[$€£]\d+(\.\d{2})?$").unwrap(),
            // Currency code at end - with commas
            Regex::new(r"^\d{1,3}(,\d{3})*(\.\d{2})?\s*(USD|EUR|GBP)$").unwrap(),
            // Currency code at end - without commas
            Regex::new(r"^\d+(\.\d{2})?\s*(USD|EUR|GBP)$").unwrap(),
        ],
    ); // Date patterns (common formats)
    m.insert(
        DataType::Date,
        vec![
            Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),     // ISO
            Regex::new(r"^\d{1,2}/\d{1,2}/\d{4}$").unwrap(), // US/UK
            Regex::new(r"^\d{1,2}-\d{1,2}-\d{4}$").unwrap(), // Common
            Regex::new(r"^\d{4}/\d{2}/\d{2}$").unwrap(),     // Asian
        ],
    );

    // Email patterns
    m.insert(
        DataType::Email,
        vec![Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()],
    );

    // Phone patterns
    m.insert(
        DataType::Phone,
        vec![
            Regex::new(r"^\+?\d{1,3}[-. ]?\d{3}[-. ]?\d{3}[-. ]?\d{4}$").unwrap(),
            Regex::new(r"^\(\d{3}\)\s*\d{3}[-. ]?\d{4}$").unwrap(),
            Regex::new(r"^\d{3}[-. ]?\d{3}[-. ]?\d{4}$").unwrap(),
        ],
    );

    m
});

// Worker-side code (to be compiled separately as worker.js)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn worker_main() {
    use web_sys::WorkerGlobalScope;

    let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();

    let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        let message: WorkerMessage = serde_wasm_bindgen::from_value(event.data()).unwrap();

        // Create temporary CSV for analysis
        let csv = CSV::new_from_column(message.column_data, message.header);
        let metadata = csv.analyze_single_column_worker();

        let response = WorkerResponse {
            column_index: message.column_index,
            metadata,
        };

        global
            .post_message(&serde_wasm_bindgen::to_value(&response).unwrap())
            .unwrap();
    }) as Box<dyn FnMut(MessageEvent)>);

    global.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = r#"id,number,decimal,currency,date,email,phone,category,text
1,1234,123.45,$1234.56,2024-01-01,test@example.com,(123) 456-7890,active,Some text
2,-5678,456.78,€2345.67,2024/02/15,another@test.com,+1-234-567-8901,inactive,More text
3,9012,-789.01,£3456.78,15/03/2024,third@email.com,123.456.7890,pending,Final text"#;

    #[test]
    fn test_csv_parsing() {
        let csv = CSV::from_string(SAMPLE_CSV.to_string()).unwrap();

        // Test basic structure
        assert_eq!(csv.row_count(), 3);
        assert_eq!(csv.column_count(), 9);

        // Test headers
        let headers = csv.headers();
        let expected_headers = vec![
            "id", "number", "decimal", "currency", "date", "email", "phone", "category", "text",
        ];
        assert_eq!(headers, expected_headers);

        // Test each column type's data
        let columns = [
            // Integer column
            (0, vec!["1", "2", "3"]),
            // Number with negatives
            (1, vec!["1234", "-5678", "9012"]),
            // Decimal numbers
            (2, vec!["123.45", "456.78", "-789.01"]),
            // Currency with different symbols
            (3, vec!["$1234.56", "€2345.67", "£3456.78"]),
            // Mixed date formats
            (4, vec!["2024-01-01", "2024/02/15", "15/03/2024"]),
            // Email addresses
            (
                5,
                vec!["test@example.com", "another@test.com", "third@email.com"],
            ),
            // Phone numbers in different formats
            (6, vec!["(123) 456-7890", "+1-234-567-8901", "123.456.7890"]),
            // Categorical data
            (7, vec!["active", "inactive", "pending"]),
            // Free text
            (8, vec!["Some text", "More text", "Final text"]),
        ];

        // Verify each column's data
        for (idx, expected_values) in columns.iter() {
            let column = csv.get_column(*idx).unwrap();
            assert_eq!(
                column.get_values(),
                *expected_values,
                "Column {} data mismatch",
                idx
            );
        }
    }

    #[test]
    fn test_invalid_column_access() {
        let csv = CSV::from_string(SAMPLE_CSV.to_string()).unwrap();
        assert!(
            csv.get_column(9).is_none(),
            "Should return None for invalid column index"
        );
    }

    #[test]
    fn test_empty_csv() {
        let empty_csv = "column1,column2\n";
        let csv = CSV::from_string(empty_csv.to_string()).unwrap();
        assert_eq!(csv.row_count(), 0);
        assert_eq!(csv.column_count(), 2);
        assert_eq!(csv.headers(), vec!["column1", "column2"]);
    }

    #[test]
    fn test_thread_count_setting() {
        let csv = CSV::from_string(SAMPLE_CSV.to_string())
            .unwrap()
            .with_thread_count(4);

        // Verify the CSV still works after setting thread count
        assert_eq!(csv.row_count(), 3);
        assert_eq!(csv.column_count(), 9);
        assert!(csv.get_column(0).is_some());
    }
}

#[cfg(test)]
mod advanced_tests {
    use super::*;

    #[test]
    fn test_type_inference() {
        // Each value needs to be in a proper CSV format with consistent columns
        let number_test = r#"numbers,extra
1,test
-1000,test
1234.56,test
$1234.56,test
€1234.56,test
1234.56,test"#;

        let csv = CSV::from_string(number_test.to_string()).unwrap();
        let values: Vec<&str> = csv.data.iter().map(|row| row[0].as_str()).collect();

        // Test each value individually
        for value in ["1", "-1000", "1234.56", "$1234.56", "€1234.56"] {
            let (detected_type, confidence) = csv.infer_type(&[value]);
            assert!(confidence > 0.5, "Low confidence for value: {}", value);
            match value {
                v if v.starts_with('$') || v.starts_with('€') => {
                    assert_eq!(detected_type, DataType::Currency)
                }
                v if v.contains('.') => assert_eq!(detected_type, DataType::Decimal),
                _ => assert_eq!(detected_type, DataType::Integer),
            }
        }
    }

    #[test]
    fn test_date_format_detection() {
        let dates_csv = r#"dates,extra
2024-01-01,test
01/02/2024,test
2024/01/01,test
15-03-2024,test"#;

        let csv = CSV::from_string(dates_csv.to_string()).unwrap();
        let col = Column {
            header: &csv.headers()[0],
            data: Arc::clone(&csv.data),
            column_index: 0,
        };

        for &date in &["2024-01-01", "01/02/2024", "15-03-2024", "2024/01/01"] {
            let format = csv.detect_date_format(&[date]);
            assert!(
                !format.contains("UNKNOWN"),
                "Failed to detect format for {}",
                date
            );
        }
    }

    #[test]
    fn test_anomaly_detection() {
        let anomaly_csv = r#"mixed,extra
1,test
2,test
three,test
4,test
5.0,test
VI,test
7,test"#;

        let csv = CSV::from_string(anomaly_csv.to_string()).unwrap();
        let values: Vec<&str> = csv.data.iter().map(|row| row[0].as_str()).collect();

        let anomalies = csv.detect_anomalies(&values, &DataType::Integer);

        // Check that non-integer values are detected as anomalies
        for value in ["three", "5.0", "VI"] {
            let found = anomalies.iter().any(|a| a.value == value);
            assert!(found, "Should detect '{}' as an anomaly", value);
        }
    }

    #[test]
    fn test_categorical_detection() {
        let categorical_csv = r#"status,count
active,100
inactive,50
active,75
pending,25
inactive,30
active,80
pending,45
active,90
inactive,60
pending,35
active,70
inactive,40"#; // Added more rows to meet the categorical criteria

        let csv = CSV::from_string(categorical_csv.to_string()).unwrap();
        let values: Vec<&str> = csv.data.iter().map(|row| row[0].as_str()).collect();

        // The original criteria for categorical data requires:
        // 1. Low cardinality relative to total values (unique_ratio < 0.05)
        // 2. At least 20 values for statistics
        assert!(
            csv.is_likely_categorical(&values),
            "Should detect repeating status values as categorical"
        );
    }

    #[test]
    fn test_sql_type_inference() {
        let mixed_csv = r#"id,price,category,description
1,10.50,active,short
2,1500.75,inactive,medium
3,25000.25,pending,longertext"#;

        let csv = CSV::from_string(mixed_csv.to_string()).unwrap();

        // Test ID column
        let id_col = Column {
            header: &csv.headers()[0],
            data: Arc::clone(&csv.data),
            column_index: 0,
        };
        let id_metadata = csv.analyze_single_column(id_col);
        assert!(
            id_metadata.sql_type.to_uppercase().contains("INT"),
            "ID should be inferred as INT type, got {}",
            id_metadata.sql_type
        );

        // Test price column
        let price_col = Column {
            header: &csv.headers()[1],
            data: Arc::clone(&csv.data),
            column_index: 1,
        };
        let price_metadata = csv.analyze_single_column(price_col);
        assert!(
            price_metadata.sql_type.to_uppercase().contains("DECIMAL"),
            "Price should be DECIMAL type, got {}",
            price_metadata.sql_type
        );

        // Test category column
        let category_col = Column {
            header: &csv.headers()[2],
            data: Arc::clone(&csv.data),
            column_index: 2,
        };
        let category_metadata = csv.analyze_single_column(category_col);
        assert!(
            category_metadata
                .sql_type
                .to_uppercase()
                .contains("VARCHAR")
                || category_metadata.sql_type.to_uppercase().contains("ENUM"),
            "Category should be VARCHAR or ENUM type, got {}",
            category_metadata.sql_type
        );
    }

    #[test]
    fn test_numeric_statistics() {
        let numbers_csv = r#"values,extra
10,test
20,test
30,test
40,test
50,test
-10,test
0,test"#;

        let csv = CSV::from_string(numbers_csv.to_string()).unwrap();
        let values: Vec<&str> = csv.data.iter().map(|row| row[0].as_str()).collect();

        if let Some(stats) = csv.calculate_numeric_stats(&values) {
            assert_eq!(stats.min, -10.0);
            assert_eq!(stats.max, 50.0);
            assert_eq!(stats.mean, 20.0);
            assert_eq!(stats.median, 20.0);

            // Test quartiles
            assert_eq!(stats.quartiles[0], 0.0); // Q1
            assert_eq!(stats.quartiles[1], 20.0); // Q2 (median)
            assert_eq!(stats.quartiles[2], 40.0); // Q3
        } else {
            panic!("Failed to calculate numeric stats");
        }
    }
}

#[cfg(test)]
mod currency_tests {
    use super::*;

    #[test]
    fn test_currency_patterns() {
        // Print out the actual patterns being used
        let patterns = &TYPE_PATTERNS
            .get(&DataType::Currency)
            .expect("Currency patterns should exist");
        for (i, pattern) in patterns.iter().enumerate() {
            println!("Currency pattern {}: {}", i, pattern.as_str());
        }

        // Test each currency pattern individually
        let test_cases = [
            "$1234.56",
            "€1234.56",
            "£1234.56",
            "$1,234.56",
            "€1,234.56",
            "£1,234.56",
            "1234.56 USD",
            "1234.56 EUR",
            "1234.56 GBP",
        ];

        for &value in test_cases.iter() {
            let (data_type, confidence) = CSV::dummy().infer_type(&[value]);
            assert_eq!(
                data_type,
                DataType::Currency,
                "Value '{}' should be detected as currency",
                value
            );
            assert!(
                confidence > 0.7,
                "Should have high confidence for currency value '{}'",
                value
            );
        }
    }

    impl CSV {
        fn dummy() -> Self {
            CSV {
                data: Arc::new(vec![]),
                headers: Arc::new(vec![]),
                row_count: 0,
                column_count: 0,
                thread_count: None,
            }
        }
    }
}
