use wasm_bindgen::prelude::*;

mod currency;
mod date;
//TODO: add back datetime when it becomes important
//mod datetime;
mod categorical;
mod email;
mod numeric;
mod phone;
pub mod type_scoring;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the detected data type of a column
#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
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

impl DataType {
    /// Returns true if the type typically contains numeric data
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            DataType::Integer | DataType::Decimal | DataType::Currency
        )
    }

    /// Returns true if the type typically contains temporal data
    pub fn is_temporal(&self) -> bool {
        matches!(self, DataType::Date)
    }

    /// Returns true if the type typically contains categorical data
    pub fn is_categorical(&self) -> bool {
        matches!(self, DataType::Categorical)
    }

    /// Returns true if the type typically benefits from indexing in SQL
    pub fn is_indexable(&self) -> bool {
        matches!(
            self,
            DataType::Integer
                | DataType::Date
                | DataType::Email
                | DataType::Categorical
                | DataType::Phone
        )
    }

    /// Returns a suggested SQL type based on the data type
    pub fn default_sql_type(&self) -> &'static str {
        match self {
            DataType::Integer => "INT",
            DataType::Decimal => "DECIMAL(10,2)",
            DataType::Currency => "DECIMAL(19,4)",
            DataType::Date => "DATE",
            DataType::Email => "VARCHAR(255)",
            DataType::Phone => "VARCHAR(20)",
            DataType::Categorical => "VARCHAR(50)",
            DataType::Text => "TEXT",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataType::Integer => "Integer",
                DataType::Decimal => "Decimal",
                DataType::Currency => "Currency",
                DataType::Date => "Date",
                DataType::Email => "Email",
                DataType::Phone => "Phone",
                DataType::Categorical => "Categorical",
                DataType::Text => "Text",
            }
        )
    }
}

/// Trait for type-specific detection and validation
pub trait TypeDetection {
    /// Returns a confidence score (0.0 to 1.0) that a value matches this type
    fn detect_confidence(value: &str) -> f64;

    /// Returns true if a value is definitely this type
    fn is_definite_match(value: &str) -> bool;

    /// Returns a normalized version of the value for this type, if possible
    fn normalize(value: &str) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_properties() {
        // Test numeric types
        assert!(DataType::Integer.is_numeric());
        assert!(DataType::Decimal.is_numeric());
        assert!(DataType::Currency.is_numeric());
        assert!(!DataType::Text.is_numeric());

        // Test temporal types
        assert!(DataType::Date.is_temporal());
        assert!(!DataType::Text.is_temporal());

        // Test categorical types
        assert!(DataType::Categorical.is_categorical());
        assert!(!DataType::Text.is_categorical());

        // Test indexable types
        assert!(DataType::Integer.is_indexable());
        assert!(DataType::Email.is_indexable());
        assert!(!DataType::Text.is_indexable());
    }

    #[test]
    fn test_default_sql_types() {
        assert_eq!(DataType::Integer.default_sql_type(), "INT");
        assert_eq!(DataType::Decimal.default_sql_type(), "DECIMAL(10,2)");
        assert_eq!(DataType::Currency.default_sql_type(), "DECIMAL(19,4)");
        assert_eq!(DataType::Date.default_sql_type(), "DATE");
        assert_eq!(DataType::Email.default_sql_type(), "VARCHAR(255)");
        assert_eq!(DataType::Phone.default_sql_type(), "VARCHAR(20)");
        assert_eq!(DataType::Categorical.default_sql_type(), "VARCHAR(50)");
        assert_eq!(DataType::Text.default_sql_type(), "TEXT");
    }

    #[test]
    fn test_display_implementation() {
        assert_eq!(format!("{}", DataType::Integer), "Integer");
        assert_eq!(format!("{}", DataType::Decimal), "Decimal");
        assert_eq!(format!("{}", DataType::Currency), "Currency");
        assert_eq!(format!("{}", DataType::Date), "Date");
        assert_eq!(format!("{}", DataType::Email), "Email");
        assert_eq!(format!("{}", DataType::Phone), "Phone");
        assert_eq!(format!("{}", DataType::Categorical), "Categorical");
        assert_eq!(format!("{}", DataType::Text), "Text");
    }
}
