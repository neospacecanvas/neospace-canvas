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
        // Get non-empty values
        let non_empty_values: Vec<&str> = values
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // If all values are empty, return default scores (will resolve to Text type)
        if non_empty_values.is_empty() {
            return TypeScores::default();
        }

        // For each type, check if ALL values match that type
        let scores = TypeScores {
            numeric: if non_empty_values
                .iter()
                .all(|&v| NumericType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| NumericType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
            currency: if non_empty_values
                .iter()
                .all(|&v| CurrencyType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| CurrencyType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
            date: if non_empty_values
                .iter()
                .all(|&v| DateType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| DateType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
            email: if non_empty_values
                .iter()
                .all(|&v| EmailType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| EmailType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
            phone: if non_empty_values
                .iter()
                .all(|&v| PhoneType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| PhoneType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
            categorical: if non_empty_values
                .iter()
                .all(|&v| CategoricalType::detect_confidence(v) == 1.0)
            {
                1.0
            } else {
                non_empty_values
                    .iter()
                    .map(|&v| CategoricalType::detect_confidence(v))
                    .sum::<f64>()
                    / non_empty_values.len() as f64
            },
        };

        scores
    }

    /// Returns the appropriate data type and its confidence score
    pub fn best_type(&self) -> (DataType, f64) {
        // First create the array and store it in a named variable
        let type_scores = [
            (DataType::Integer, self.numeric),
            (DataType::Currency, self.currency),
            (DataType::Date, self.date),
            (DataType::Email, self.email),
            (DataType::Phone, self.phone),
            (DataType::Categorical, self.categorical),
        ];

        // Now use into_iter() instead of iter() to take ownership of the values
        let perfect_match = type_scores
            .into_iter()
            .find(|(_, confidence)| (confidence - 1.0).abs() < f64::EPSILON);

        if let Some((dtype, confidence)) = perfect_match {
            (dtype, confidence) // No need for clone() or deref since we own the values
        } else {
            (DataType::Text, 0.0)
        }
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
