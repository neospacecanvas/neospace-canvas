use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // International format with optional country code
        Regex::new(r"^\+?\d{1,3}[-. ]?\d{3}[-. ]?\d{3}[-. ]?\d{4}$").unwrap(),
        // US/Canada format with parentheses
        Regex::new(r"^\(\d{3}\)\s*\d{3}[-. ]?\d{4}$").unwrap(),
        // Basic format with separators
        Regex::new(r"^\d{3}[-. ]?\d{3}[-. ]?\d{4}$").unwrap(),
    ]
});

#[derive(Debug)]
pub struct PhoneType;

impl TypeDetection for PhoneType {
    fn detect_confidence(value: &str) -> f64 {
        let clean_value = value.replace(' ', "");
        if clean_value.is_empty() {
            return 0.0;
        }

        if Self::is_definite_match(&clean_value) {
            return 1.0;
        }

        // Check if it's mostly digits with some valid separators
        let digit_count = clean_value.chars().filter(|c| c.is_ascii_digit()).count();
        let separator_chars = ['+', '-', '.', '(', ')', ' '];
        let other_chars: usize = clean_value
            .chars()
            .filter(|c| !c.is_ascii_digit() && !separator_chars.contains(c))
            .count();

        // If we have the right number of digits and no invalid characters
        if digit_count >= 10 && digit_count <= 15 && other_chars == 0 {
            return 0.7;
        }

        // If it has the right number of digits but some invalid characters
        if digit_count >= 10 && digit_count <= 15 {
            return 0.3;
        }

        0.0
    }

    fn is_definite_match(value: &str) -> bool {
        let clean_value = value.replace(' ', "");
        PHONE_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(&clean_value))
    }

    fn normalize(value: &str) -> Option<String> {
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 10 || digits.len() > 15 {
            return None;
        }

        // Format as (XXX) XXX-XXXX for 10-digit numbers
        if digits.len() == 10 {
            Some(format!(
                "({}) {}-{}",
                &digits[..3],
                &digits[3..6],
                &digits[6..]
            ))
        } else {
            // For international numbers, format as +X-XXX-XXX-XXXX
            Some(format!(
                "+{}-{}-{}-{}",
                &digits[..digits.len() - 10],
                &digits[digits.len() - 10..digits.len() - 7],
                &digits[digits.len() - 7..digits.len() - 4],
                &digits[digits.len() - 4..]
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definite_matches() {
        let test_cases = vec![
            // US/Canada format
            ("(123) 456-7890", true),
            ("(123)456-7890", true),
            ("(123)4567890", true),
            // Basic format
            ("123-456-7890", true),
            ("123.456.7890", true),
            ("123 456 7890", true),
            // International format
            ("+1-123-456-7890", true),
            ("+44 123 456 7890", true),
            // Invalid formats
            ("123-456", false),
            ("abcd-efg-hijk", false),
            ("12345", false),
            ("", false),
            // Edge cases
            ("(123) 456-78901", false), // Too many digits
            ("(123) 456-789", false),   // Too few digits
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                PhoneType::is_definite_match(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_confidence_levels() {
        let test_cases = vec![
            // High confidence (1.0) - Perfect matches
            ("(123) 456-7890", 1.0),
            ("+1-123-456-7890", 1.0),
            // Medium confidence (0.7) - Mostly correct format
            ("123456#7890", 0.3), // Changed to 0.3 because it has invalid characters
            ("123.456.7890...", 0.7), // Still 0.7 because only valid separators
            // Low confidence (0.3) - Right digits but wrong format
            ("12345678901234@", 0.3),
            // No confidence (0.0) - Clear non-matches
            ("abc-def-ghij", 0.0),
            ("12345", 0.0),
            ("", 0.0),
        ];

        for (input, expected) in test_cases {
            let confidence = PhoneType::detect_confidence(input);
            assert!(
                (confidence - expected).abs() < f64::EPSILON,
                "Failed for input: {}. Expected confidence {}, got {}",
                input,
                expected,
                confidence
            );
        }
    }

    #[test]
    fn test_normalization() {
        let test_cases = vec![
            // US/Canada format normalization
            ("(123) 456-7890", Some("(123) 456-7890".to_string())),
            ("123.456.7890", Some("(123) 456-7890".to_string())),
            ("123 456 7890", Some("(123) 456-7890".to_string())),
            ("1234567890", Some("(123) 456-7890".to_string())),
            // International format normalization
            ("+1-123-456-7890", Some("+1-123-456-7890".to_string())),
            ("11234567890", Some("+1-123-456-7890".to_string())),
            ("+44 123 456 7890", Some("+44-123-456-7890".to_string())),
            // Invalid inputs
            ("123-456", None),
            ("abcd-efg-hijk", None),
            ("12345", None),
            ("", None),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                PhoneType::normalize(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test very long numbers
        assert_eq!(PhoneType::normalize("12345678901234567890"), None);

        // Test various separators
        assert_eq!(
            PhoneType::normalize("123---456---7890"),
            Some("(123) 456-7890".to_string())
        );

        // Test mixed separators
        assert_eq!(
            PhoneType::normalize("123.-_456 !@#$%^&*()7890"),
            Some("(123) 456-7890".to_string())
        );

        // Test numbers with spaces
        assert_eq!(
            PhoneType::normalize("   123 456   7890   "),
            Some("(123) 456-7890".to_string())
        );
    }
}
