use crate::types::{
    categorical::CategoricalType, currency::CurrencyType, date::DateType, email::EmailType,
    numeric::NumericType, phone::PhoneType, DataType, TypeDetection,
};

/// Holds confidence scores for how well data matches each possible type
#[derive(Debug, Default)]
pub struct TypeScores {
    pub numeric: f64,
    pub currency: f64,
    pub date: f64,
    pub email: f64,
    pub phone: f64,
    pub categorical: f64,
}

impl TypeScores {
    /// Creates TypeScores from analyzing a column of values
    pub fn from_column(values: &[String]) -> Self {
        // Start with zeroed scores
        let mut scores = TypeScores::default();
        let mut valid_values = 0;

        // Process each non-empty value
        for value in values {
            let value = value.trim();
            if !value.is_empty() {
                valid_values += 1;
                // Add confidence scores from each type detector
                scores.numeric += NumericType::detect_confidence(value);
                scores.currency += CurrencyType::detect_confidence(value);
                scores.date += DateType::detect_confidence(value);
                scores.email += EmailType::detect_confidence(value);
                scores.phone += PhoneType::detect_confidence(value);
                scores.categorical += CategoricalType::detect_confidence(value);
            }
        }

        // Average the scores by dividing by number of valid values
        if valid_values > 0 {
            scores.numeric /= valid_values as f64;
            scores.currency /= valid_values as f64;
            scores.date /= valid_values as f64;
            scores.email /= valid_values as f64;
            scores.phone /= valid_values as f64;
            scores.categorical /= valid_values as f64;
        }

        scores
    }

    /// Returns the most likely data type and its confidence score
    pub fn best_type(&self) -> (DataType, f64) {
        let scores = [
            (DataType::Integer, self.numeric),
            (DataType::Currency, self.currency),
            (DataType::Date, self.date),
            (DataType::Email, self.email),
            (DataType::Phone, self.phone),
            (DataType::Categorical, self.categorical),
        ];

        // Find type with highest confidence
        scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(t, c)| (t.clone(), *c))
            .unwrap_or((DataType::Text, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_detection() {
        let values = vec!["123".to_string(), "456".to_string(), "789".to_string()];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Integer);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_currency_detection() {
        let values = vec![
            "$100.00".to_string(),
            "$250.50".to_string(),
            "$1,234.56".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Currency);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_date_detection() {
        let values = vec![
            "2024-01-01".to_string(),
            "2024-02-15".to_string(),
            "2024-03-30".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Date);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_email_detection() {
        let values = vec![
            "user@example.com".to_string(),
            "another@test.com".to_string(),
            "email@domain.org".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Email);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_phone_detection() {
        let values = vec![
            "(123) 456-7890".to_string(),
            "234-567-8901".to_string(),
            "345.678.9012".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Phone);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_categorical_detection() {
        let values = vec![
            "High".to_string(),
            "Medium".to_string(),
            "Low".to_string(),
            "High".to_string(),
            "Medium".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Categorical);
        assert!(confidence > 0.7);
    }

    #[test]
    fn test_mixed_types() {
        let values = vec![
            "123".to_string(),
            "abc".to_string(),
            "def".to_string(),
            "456".to_string(),
        ];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Text);
        assert!(confidence < 0.5);
    }

    #[test]
    fn test_empty_values() {
        let values = vec!["".to_string(), "  ".to_string(), "\n".to_string()];
        let scores = TypeScores::from_column(&values);
        let (data_type, confidence) = scores.best_type();
        assert_eq!(data_type, DataType::Text);
        assert_eq!(confidence, 0.0);
    }
}
