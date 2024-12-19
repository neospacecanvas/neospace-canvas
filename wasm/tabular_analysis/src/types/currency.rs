use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;
//TODO: Currently only dollars are supported, support for other currencies is needed
#[derive(Debug, Clone, Copy)]
pub enum CurrencySymbol {
    USD, // Start with just USD
}

impl CurrencySymbol {
    fn symbol(&self) -> &str {
        match self {
            CurrencySymbol::USD => "$",
        }
    }

    fn code(&self) -> &str {
        match self {
            CurrencySymbol::USD => "USD",
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        let s = s.trim();
        match s {
            "$" | "USD" => Some(CurrencySymbol::USD),
            _ => None,
        }
    }

    fn format_value(&self, amount: f64) -> String {
        match self {
            CurrencySymbol::USD => format!("{}{:.2}", self.symbol(), amount),
        }
    }
}

static CURRENCY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // USD patterns only
        Regex::new(r"^\$\d+(?:,\d{3})*(?:\.\d{2})?$").unwrap(),
        Regex::new(r"^\d+(?:,\d{3})*(?:\.\d{2})?USD$").unwrap(),
        Regex::new(r"^USD\d+(?:,\d{3})*(?:\.\d{2})?$").unwrap(),
    ]
});

#[derive(Debug)]
pub struct CurrencyType;

impl TypeDetection for CurrencyType {
    fn detect_confidence(value: &str) -> f64 {
        let clean_value = value.replace(' ', "");
        if clean_value.is_empty() {
            return 0.0;
        }

        if Self::is_definite_match(&clean_value) {
            return 1.0;
        }

        // Look for USD indicators
        if clean_value.starts_with('$') || clean_value.contains("USD") {
            return 0.9;
        }

        // Check for number with 2 decimal places
        if clean_value.matches('.').count() == 1 {
            if let Some(decimals) = clean_value.split('.').nth(1) {
                if decimals.len() == 2 && decimals.chars().all(|c| c.is_ascii_digit()) {
                    return 0.5;
                }
            }
        }

        0.0
    }

    fn is_definite_match(value: &str) -> bool {
        let clean_value = value.replace(' ', "");
        CURRENCY_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(&clean_value))
    }

    fn normalize(value: &str) -> Option<String> {
        let clean_value = value.replace(' ', "");
        if clean_value.is_empty() {
            return None;
        }

        // Extract number and parse it
        let numeric_part: String = clean_value
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
            .collect();

        let amount = numeric_part.replace(',', "").parse::<f64>().ok()?;

        // Only handle USD for now
        Some(CurrencySymbol::USD.format_value(amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_normalization() {
        let test_cases: Vec<(&str, Option<String>)> = vec![
            ("$1234.56", Some("$1234.56".into())),
            ("$ 1234.56", Some("$1234.56".into())),
            ("$1,234.56", Some("$1234.56".into())),
            ("1234.56 USD", Some("$1234.56".into())),
            ("USD 1234.56", Some("$1234.56".into())),
            ("1234.567", Some("$1234.57".into())),
            ("ABC", None),
            ("", None),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                CurrencyType::normalize(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_currency_detection() {
        let test_cases = vec![
            ("$1234.56", 1.0),
            ("$ 1234.56", 1.0),
            ("1234.56 USD", 1.0),
            ("1234.56", 0.5),
            ("ABC", 0.0),
            ("", 0.0),
        ];

        for (input, expected) in test_cases {
            assert!(
                (CurrencyType::detect_confidence(input) - expected).abs() < f64::EPSILON,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_definite_matches() {
        let test_cases = vec![
            ("$1234.56", true),
            ("$ 1234.56", true),
            ("1234.56 USD", true),
            ("1234.567", false),
            ("$ABC", false),
            ("", false),
        ];

        for (input, should_match) in test_cases {
            assert_eq!(
                CurrencyType::is_definite_match(input),
                should_match,
                "Failed for input: {}",
                input
            );
        }
    }
}
