use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

// Constants for categorical detection
const MAX_CARDINALITY_RATIO: f64 = 0.05; // Maximum 5% unique values (keeping conservative)
const MIN_SAMPLE_SIZE: usize = 20; // Need at least 20 values for reliable detection
const MIN_CATEGORY_FREQUENCY: usize = 3; // Each category should appear at least 3 times
const MAX_CATEGORY_LENGTH: usize = 100; // Maximum reasonable length for a category value
const MIN_NON_EMPTY_RATIO: f64 = 0.5; // At least 50% of values should be non-empty

// Common categorical patterns
static CATEGORICAL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Common boolean patterns
        Regex::new(r"^(?i)(true|false|yes|no|y|n|t|f)$").unwrap(),
        // Common rating patterns
        Regex::new(r"^(?i)(high|medium|low|critical|major|minor)$").unwrap(),
        // Common status patterns
        Regex::new(r"^(?i)(active|inactive|pending|completed|cancelled|failed|success)$").unwrap(),
        // Common level patterns
        Regex::new(r"^(?i)(beginner|intermediate|advanced|expert)$").unwrap(),
    ]
});

// Common categorical column name patterns
static CATEGORICAL_NAME_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)type").unwrap(),
        Regex::new(r"(?i)category").unwrap(),
        Regex::new(r"(?i)status").unwrap(),
        Regex::new(r"(?i)level").unwrap(),
        Regex::new(r"(?i)grade").unwrap(),
        Regex::new(r"(?i)rating").unwrap(),
        Regex::new(r"(?i)priority").unwrap(),
    ]
});

#[derive(Debug)]
pub struct CategoricalType;

impl TypeDetection for CategoricalType {
    fn detect_confidence(value: &str) -> f64 {
        // For single value detection, we can only check against known patterns
        if value.trim().is_empty() {
            return 0.0;
        }

        if Self::is_definite_match(value) {
            return 1.0;
        }

        // Check against categorical patterns
        if CATEGORICAL_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(value))
        {
            return 0.9;
        }

        // For unknown values, return a low confidence
        // The true categorical nature will be determined by analyzing the full column
        0.3
    }

    fn is_definite_match(value: &str) -> bool {
        let clean_value = value.trim();
        if clean_value.is_empty() {
            return false;
        }

        // Check against known categorical patterns
        CATEGORICAL_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(clean_value))
    }

    fn normalize(value: &str) -> Option<String> {
        let clean_value = value.trim();
        if clean_value.is_empty() {
            return None;
        }

        // For categorical values, we want to preserve exact casing and format
        // Just trim whitespace and return
        Some(clean_value.to_string())
    }
}

impl CategoricalType {
    /// Analyzes a column of data to determine if it's likely categorical
    pub fn analyze_column(values: &[String], column_name: &str) -> (bool, f64) {
        if values.len() < MIN_SAMPLE_SIZE {
            return (false, 0.0);
        }

        let score = Self::calculate_categorical_score(values, column_name);
        (score > 0.7, score) // Consider it categorical if score > 0.7
    }

    fn calculate_categorical_score(values: &[String], column_name: &str) -> f64 {
        let mut score = 0.0;

        // Primary factors (70% of total score)
        score += 0.4 * Self::cardinality_ratio_score(values);
        score += 0.2 * Self::value_distribution_score(values);
        score += 0.1 * Self::value_frequency_score(values);

        // Secondary factors (30% of total score)
        score += 0.1 * Self::pattern_match_score(values);
        score += 0.1 * Self::length_consistency_score(values);
        score += 0.1 * Self::column_name_score(column_name);

        score
    }

    fn cardinality_ratio_score(values: &[String]) -> f64 {
        // Filter out empty values
        let non_empty_values: Vec<_> = values.iter().filter(|v| !v.trim().is_empty()).collect();

        if non_empty_values.is_empty() {
            return 0.0;
        }

        // Check if we have enough non-empty values
        let non_empty_ratio = non_empty_values.len() as f64 / values.len() as f64;
        if non_empty_ratio < MIN_NON_EMPTY_RATIO {
            return 0.0;
        }

        let total_values = non_empty_values.len() as f64;
        let unique_values: HashSet<_> = non_empty_values.iter().collect();
        let unique_count = unique_values.len() as f64;

        let ratio = unique_count / total_values;
        if ratio <= MAX_CARDINALITY_RATIO {
            1.0
        } else if ratio <= MAX_CARDINALITY_RATIO * 2.0 {
            0.5
        } else {
            0.0
        }
    }

    fn value_distribution_score(values: &[String]) -> f64 {
        // Filter out empty values
        let non_empty_values: Vec<_> = values.iter().filter(|v| !v.trim().is_empty()).collect();

        if non_empty_values.is_empty() {
            return 0.0;
        }

        let mut value_counts: HashMap<&String, usize> = HashMap::new();
        for value in non_empty_values.iter() {
            *value_counts.entry(value).or_insert(0) += 1;
        }

        // Calculate average length of values
        let avg_length: f64 = non_empty_values.iter().map(|s| s.len()).sum::<usize>() as f64
            / non_empty_values.len() as f64;

        // If average length is high (like in comments/sentences),
        // require a much stricter frequency distribution
        let length_penalty = if avg_length > 15.0 { 0.5 } else { 1.0 };

        // Calculate frequency distribution
        let frequent_values = value_counts
            .values()
            .filter(|&&count| count >= MIN_CATEGORY_FREQUENCY)
            .count();

        // Score based on how many values meet minimum frequency,
        // with penalty for long text values
        ((frequent_values as f64 / value_counts.len() as f64) * length_penalty).min(1.0)
    }

    fn value_frequency_score(values: &[String]) -> f64 {
        let mut value_counts = HashMap::new();
        for value in values {
            *value_counts.entry(value).or_insert(0) += 1;
        }

        // Check if each unique value appears enough times
        let sufficient_frequency = value_counts
            .values()
            .filter(|&&count| count >= MIN_CATEGORY_FREQUENCY)
            .count();

        (sufficient_frequency as f64 / value_counts.len() as f64).min(1.0)
    }

    fn pattern_match_score(values: &[String]) -> f64 {
        let pattern_matches = values
            .iter()
            .filter(|&value| {
                CATEGORICAL_PATTERNS
                    .iter()
                    .any(|pattern| pattern.is_match(value))
            })
            .count();

        (pattern_matches as f64 / values.len() as f64).min(1.0)
    }

    fn length_consistency_score(values: &[String]) -> f64 {
        let lengths: Vec<usize> = values.iter().map(|s| s.len()).collect();
        if lengths.is_empty() {
            return 0.0;
        }

        // Calculate mean length
        let mean_length: f64 = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

        // Calculate standard deviation
        let variance: f64 = lengths
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean_length;
                diff * diff
            })
            .sum::<f64>()
            / lengths.len() as f64;

        let std_dev = variance.sqrt();

        // Score based on coefficient of variation (CV = std_dev / mean)
        // Lower CV means more consistent lengths
        let cv = std_dev / mean_length;
        (1.0 - (cv / 2.0)).max(0.0).min(1.0)
    }

    fn column_name_score(column_name: &str) -> f64 {
        if CATEGORICAL_NAME_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(column_name))
        {
            1.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_categorical_detection() {
        let test_cases = vec![
            // Boolean-like values
            ("true", 1.0),
            ("false", 1.0),
            ("yes", 1.0),
            ("no", 1.0),
            // Status values
            ("active", 1.0),
            ("pending", 1.0),
            ("completed", 1.0),
            // Non-categorical values
            ("random text", 0.3),
            ("12345", 0.3),
            ("", 0.0),
        ];

        for (input, expected) in test_cases {
            let confidence = CategoricalType::detect_confidence(input);
            assert!(
                (confidence - expected).abs() < f64::EPSILON,
                "Failed for input: {}. Expected {}, got {}",
                input,
                expected,
                confidence
            );
        }
    }

    #[test]
    fn test_pokemon_types() {
        // Simulate Pokemon type data
        let types: Vec<String> = vec![
            "Water".to_string(),
            "Fire".to_string(),
            "Grass".to_string(),
            "Electric".to_string(),
            "Psychic".to_string(),
            "Normal".to_string(),
            "Flying".to_string(),
            "Ground".to_string(),
            "Rock".to_string(),
            "Bug".to_string(),
            "Poison".to_string(),
            "Fighting".to_string(),
            "Ghost".to_string(),
            "Ice".to_string(),
            "Dragon".to_string(),
            "Dark".to_string(),
            "Steel".to_string(),
            "Fairy".to_string(),
        ]
        .into_iter()
        .cycle() // Repeat the types
        .take(500) // Generate 500 type entries (simulating multiple Pokemon)
        .collect();

        let (is_categorical, confidence) = CategoricalType::analyze_column(&types, "type");
        assert!(
            is_categorical,
            "Pokemon types should be detected as categorical"
        );
        assert!(
            confidence > 0.8,
            "Should have high confidence for Pokemon types"
        );
    }

    #[test]
    fn test_us_states_dataset() {
        // Create a realistic distribution of US states data
        let states: Vec<String> = vec![
            ("California", 50),
            ("Texas", 40),
            ("Florida", 30),
            ("New York", 25),
            ("Illinois", 20),
            ("Pennsylvania", 20),
            ("Ohio", 15),
            ("Michigan", 15),
            ("Georgia", 15),
            ("North Carolina", 10),
            ("New Jersey", 10),
            ("Virginia", 10),
            ("Washington", 8),
            ("Arizona", 8),
            ("Massachusetts", 8),
            ("Tennessee", 5),
            ("Indiana", 5),
            ("Maryland", 5),
            ("Missouri", 3),
            ("Wisconsin", 3),
            ("Colorado", 3),
            ("Minnesota", 2),
            ("South Carolina", 2),
            ("Alabama", 2),
            ("Louisiana", 2),
            ("Kentucky", 2),
            ("Oregon", 2),
            // Add remaining states with lower frequencies
            ("Oklahoma", 1),
            ("Connecticut", 1),
            ("Utah", 1),
            ("Iowa", 1),
            ("Nevada", 1),
            ("Arkansas", 1),
            ("Mississippi", 1),
            ("Kansas", 1),
            ("New Mexico", 1),
            ("Nebraska", 1),
            ("Idaho", 1),
            ("Hawaii", 1),
            ("New Hampshire", 1),
            ("Maine", 1),
            ("Montana", 1),
            ("Rhode Island", 1),
            ("Delaware", 1),
            ("South Dakota", 1),
            ("North Dakota", 1),
            ("Alaska", 1),
            ("Vermont", 1),
            ("Wyoming", 1),
            ("West Virginia", 1),
        ]
        .into_iter()
        .flat_map(|(state, count)| vec![state.to_string(); count])
        .collect();

        let (is_categorical, confidence) = CategoricalType::analyze_column(&states, "state");
        // With stricter ratio, states should not be auto-detected as categorical
        assert!(
            !is_categorical,
            "US states should not be auto-detected as categorical due to high cardinality"
        );
        assert!(
            confidence < 0.7,
            "Should have lower confidence for US states due to high cardinality"
        );
    }

    #[test]
    fn test_customer_status_dataset() {
        // Simulate customer status data with realistic distribution
        let statuses: Vec<String> = vec![
            ("active", 1000),  // Most customers are active
            ("inactive", 200), // Some inactive
            ("pending", 50),   // Few pending
            ("suspended", 30), // Very few suspended
            ("closed", 20),    // Very few closed
        ]
        .into_iter()
        .flat_map(|(status, count)| vec![status.to_string(); count])
        .collect();

        let (is_categorical, confidence) = CategoricalType::analyze_column(&statuses, "status");
        assert!(
            is_categorical,
            "Customer status should be detected as categorical"
        );
        assert!(
            confidence > 0.9,
            "Should have very high confidence for status"
        );
    }

    #[test]
    fn test_non_categorical_data() {
        // Test cases that should NOT be detected as categorical

        // Test case 1: Names (unique values)
        let names: Vec<String> = (0..100).map(|i| format!("Person_{}", i)).collect();
        let (is_cat_names, conf_names) = CategoricalType::analyze_column(&names, "name");
        assert!(!is_cat_names, "Names should not be detected as categorical");
        assert!(conf_names < 0.5, "Should have low confidence for names");

        // Test case 2: Timestamps
        let timestamps: Vec<String> = (0..100)
            .map(|i| format!("2024-01-{:02} 10:{:02}:00", i % 31 + 1, i % 60))
            .collect();
        let (is_cat_time, conf_time) = CategoricalType::analyze_column(&timestamps, "timestamp");
        assert!(
            !is_cat_time,
            "Timestamps should not be detected as categorical"
        );
        assert!(conf_time < 0.5, "Should have low confidence for timestamps");

        // Test case 3: Free-form text
        let comments: Vec<String> = (0..100)
            .map(|i| {
                let base = match i % 4 {
                    0 => "Great",
                    1 => "Good",
                    2 => "Not bad",
                    _ => "Okay",
                };
                let product = match i % 3 {
                    0 => "product",
                    1 => "service",
                    _ => "experience",
                };
                let detail = match i % 5 {
                    0 => "would recommend!",
                    1 => "but could be better.",
                    2 => "and very helpful.",
                    3 => "will buy again.",
                    _ => "thanks!",
                };
                format!("{} {} {}", base, product, detail)
            })
            .collect();
        let (is_cat_comments, conf_comments) =
            CategoricalType::analyze_column(&comments, "comment");
        assert!(
            !is_cat_comments,
            "Comments should not be detected as categorical"
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test case 1: Empty dataset
        let empty: Vec<String> = vec![];
        let (is_cat_empty, conf_empty) = CategoricalType::analyze_column(&empty, "empty");
        assert!(!is_cat_empty, "Empty dataset should not be categorical");
        assert!(
            conf_empty == 0.0,
            "Empty dataset should have zero confidence"
        );

        // Test case 2: Dataset too small
        let small: Vec<String> = vec!["A".to_string(), "B".to_string()];
        let (is_cat_small, conf_small) = CategoricalType::analyze_column(&small, "small");
        assert!(!is_cat_small, "Too small dataset should not be categorical");
        assert!(
            conf_small == 0.0,
            "Too small dataset should have zero confidence"
        );

        // Test case 3: All null/empty values
        let nulls: Vec<String> = vec!["".to_string(); 100];
        let (is_cat_nulls, conf_nulls) = CategoricalType::analyze_column(&nulls, "nulls");
        assert!(!is_cat_nulls, "All nulls should not be categorical");
        assert!(conf_nulls < 0.5, "All nulls should have low confidence");

        // Test case 4: Single value repeated
        let single: Vec<String> = vec!["Same".to_string(); 100];
        let (is_cat_single, conf_single) = CategoricalType::analyze_column(&single, "single");
        assert!(is_cat_single, "Single repeated value should be categorical");
        assert!(
            conf_single > 0.7,
            "Single repeated value should have high confidence"
        );
    }

    #[test]
    fn test_column_name_influence() {
        let values = vec!["A".to_string(), "B".to_string(), "C".to_string()]
            .into_iter()
            .cycle()
            .take(100)
            .collect::<Vec<_>>();

        // Test with categorical column names
        let (is_cat_type, conf_type) = CategoricalType::analyze_column(&values, "user_type");
        let (is_cat_status, conf_status) = CategoricalType::analyze_column(&values, "status");
        let (is_cat_category, conf_category) = CategoricalType::analyze_column(&values, "category");

        assert!(
            conf_type > 0.7,
            "Column named 'user_type' should boost confidence"
        );
        assert!(
            conf_status > 0.7,
            "Column named 'status' should boost confidence"
        );
        assert!(
            conf_category > 0.7,
            "Column named 'category' should boost confidence"
        );

        // Test with non-categorical column names
        let (_, conf_name) = CategoricalType::analyze_column(&values, "name");
        let (_, conf_desc) = CategoricalType::analyze_column(&values, "description");

        assert!(
            conf_name < conf_type,
            "Non-categorical column name should have lower confidence"
        );
        assert!(
            conf_desc < conf_status,
            "Non-categorical column name should have lower confidence"
        );
    }
}
