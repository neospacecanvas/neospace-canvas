use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;

static NUMERIC_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Integer patterns
        Regex::new(r"^-?\d+$").unwrap(), // Basic integers
        Regex::new(r"^-?\d{1,3}(,\d{3})*$").unwrap(), // Integers with commas
        // Decimal patterns
        Regex::new(r"^-?\d*\.\d+$").unwrap(), // Decimals
        Regex::new(r"^-?\d{1,3}(,\d{3})*\.\d+$").unwrap(), // Decimals with commas
        Regex::new(r"^-?\d+\.\d*$").unwrap(), // Decimals with optional trailing zeros
    ]
});

#[derive(Debug)]
pub struct NumericType;

impl TypeDetection for NumericType {
    fn detect_confidence(value: &str) -> f64 {
        // For numeric types, we can be more binary in our detection
        // If it matches our patterns, we're 100% confident it's a number
        if Self::is_definite_match(value) {
            1.0
        } else {
            0.0
        }
    }

    fn is_definite_match(value: &str) -> bool {
        let clean_value = value.trim().replace(" ", "");
        if clean_value.is_empty() {
            return false;
        }

        NUMERIC_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(&clean_value))
    }

    fn normalize(value: &str) -> Option<String> {
        let clean_value = value.trim().replace(" ", "");
        if clean_value.is_empty() {
            return None;
        }

        // First check if it matches our patterns - if not, return None
        if !Self::is_definite_match(&clean_value) {
            return None;
        }

        // Remove commas and try to parse as a number
        let numeric_value = clean_value.replace(",", "");

        // Try parsing as different numeric types
        if let Ok(int_val) = numeric_value.parse::<i64>() {
            return Some(int_val.to_string());
        }

        if let Ok(float_val) = numeric_value.parse::<f64>() {
            // Format with limited decimal places to avoid floating point artifacts
            return Some(
                format!("{:.10}", float_val)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string(),
            );
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_patterns() {
        let test_cases = vec![
            // Basic integers
            ("123", true),
            ("-123", true),
            ("0", true),
            // Integers with commas
            ("1,234", true),
            ("1,234,567", true),
            ("-1,234,567", true),
            // Invalid integer formats
            ("1,23", false),   // Incorrect comma placement
            ("1,2345", false), // Incorrect grouping
            ("1,,234", false), // Double comma
            ("1,234,", false), // Trailing comma
            ("", false),       // Empty string
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                NumericType::is_definite_match(input),
                expected,
                "Integer pattern match failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_decimal_patterns() {
        let test_cases = vec![
            // Basic decimals
            ("123.45", true),
            ("-123.45", true),
            ("0.45", true),
            (".45", true),
            // Decimals with commas
            ("1,234.56", true),
            ("1,234,567.89", true),
            ("-1,234,567.89", true),
            // Edge cases
            ("123.", true), // Trailing decimal point
            // Invalid decimal formats
            ("123.45.67", false), // Multiple decimal points
            ("123,45", false),    // Comma instead of decimal
            ("1,23.45", false),   // Incorrect comma placement
            ("..", false),        // Multiple decimal points only
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                NumericType::is_definite_match(input),
                expected,
                "Decimal pattern match failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_normalization() {
        let test_cases = vec![
            // Integer normalization
            ("123", Some("123".to_string())),
            ("-123", Some("-123".to_string())),
            ("1,234", Some("1234".to_string())),
            ("1,234,567", Some("1234567".to_string())),
            // Decimal normalization
            ("123.45", Some("123.45".to_string())),
            ("123.450", Some("123.45".to_string())), // Remove trailing zeros
            ("123.0", Some("123".to_string())),      // Remove unnecessary decimal
            ("-123.45", Some("-123.45".to_string())),
            ("1,234.56", Some("1234.56".to_string())),
            // Edge cases
            ("0", Some("0".to_string())),
            ("0.0", Some("0".to_string())),
            (".5", Some("0.5".to_string())),
            // Invalid inputs
            ("abc", None),
            ("123.45.67", None),
            ("", None),
            ("   ", None),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                NumericType::normalize(input),
                expected,
                "Normalization failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test whitespace handling
        assert_eq!(NumericType::normalize("  123  "), Some("123".to_string()));
        assert_eq!(
            NumericType::normalize(" -123.45 "),
            Some("-123.45".to_string())
        );

        // Test very large numbers
        assert_eq!(
            NumericType::normalize("1234567890123456789"),
            Some("1234567890123456789".to_string())
        );

        // Test very small decimals
        assert_eq!(
            NumericType::normalize("0.0000000001"),
            Some("0.0000000001".to_string())
        );

        // Test scientific notation (should not be supported)
        assert_eq!(NumericType::normalize("1.23e5"), None);

        // Test mixed invalid characters
        assert_eq!(NumericType::normalize("12.34.5x"), None);
        assert_eq!(NumericType::normalize("12.34/5"), None);

        // Test multiple leading/trailing zeros
        assert_eq!(NumericType::normalize("00123"), Some("123".to_string()));
        assert_eq!(
            NumericType::normalize("123.4500"),
            Some("123.45".to_string())
        );
    }
}
