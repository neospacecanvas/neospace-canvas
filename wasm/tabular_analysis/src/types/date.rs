use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateFormat {
    /// YYYY-MM-DD (e.g., 2024-03-19)
    Iso8601,
    /// MM/DD/YYYY (e.g., 03/19/2024)
    UsSlash,
    /// DD-MM-YYYY (e.g., 19-03-2024)
    EuropeanDash,
    /// DD/MM/YYYY (e.g., 19/03/2024)
    EuropeanSlash,
    /// YYYY/MM/DD (e.g., 2024/03/19)
    JapaneseSlash,
    /// MM-DD-YYYY (e.g., 03-19-2024)
    UsDash,
}

#[derive(Debug, Clone)]
pub struct Date {
    year: u32,
    month: u32,
    day: u32,
    format: DateFormat,
}

impl Date {
    pub fn new(year: u32, month: u32, day: u32, format: DateFormat) -> Option<Self> {
        if DateType::is_valid_date(year, month, day) {
            Some(Date {
                year,
                month,
                day,
                format,
            })
        } else {
            None
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let clean_value = value.trim();
        if clean_value.is_empty() {
            return None;
        }

        // Try each format
        for format in [
            DateFormat::Iso8601,
            DateFormat::JapaneseSlash,
            DateFormat::UsSlash,
            DateFormat::EuropeanDash,
            DateFormat::EuropeanSlash,
            DateFormat::UsDash,
        ] {
            if format.matches(clean_value) {
                if let Some((mut year, month, day)) = format.extract_components(clean_value) {
                    // Handle two-digit years
                    if year < 100 {
                        year += if year < 50 { 2000 } else { 1900 };
                    }

                    return Date::new(year, month, day, format);
                }
            }
        }
        None
    }

    pub fn to_format(&self, target_format: DateFormat) -> String {
        match target_format {
            DateFormat::Iso8601 => format!("{:04}-{:02}-{:02}", self.year, self.month, self.day),
            DateFormat::UsSlash => format!("{:02}/{:02}/{:04}", self.month, self.day, self.year),
            DateFormat::EuropeanDash => {
                format!("{:02}-{:02}-{:04}", self.day, self.month, self.year)
            }
            DateFormat::EuropeanSlash => {
                format!("{:02}/{:02}/{:04}", self.day, self.month, self.year)
            }
            DateFormat::JapaneseSlash => {
                format!("{:04}/{:02}/{:02}", self.year, self.month, self.day)
            }
            DateFormat::UsDash => format!("{:02}-{:02}-{:04}", self.month, self.day, self.year),
        }
    }

    pub fn format(&self) -> DateFormat {
        self.format
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> u32 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_format(self.format))
    }
}

impl DateFormat {
    fn pattern(&self) -> &'static str {
        match self {
            DateFormat::Iso8601 => r"^\d{4}-\d{2}-\d{2}$",
            DateFormat::UsSlash => r"^\d{1,2}/\d{1,2}/\d{4}$",
            DateFormat::EuropeanDash => r"^\d{1,2}-\d{1,2}-\d{4}$",
            DateFormat::EuropeanSlash => r"^\d{1,2}/\d{1,2}/\d{4}$",
            DateFormat::JapaneseSlash => r"^\d{4}/\d{2}/\d{2}$",
            DateFormat::UsDash => r"^\d{1,2}-\d{1,2}-\d{4}$",
        }
    }

    fn extract_components(&self, value: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = value.split(|c| c == '/' || c == '-').collect();
        if parts.len() != 3 {
            return None;
        }

        let numbers: Vec<u32> = parts.iter().filter_map(|&s| s.parse().ok()).collect();
        if numbers.len() != 3 {
            return None;
        }

        match self {
            DateFormat::Iso8601 | DateFormat::JapaneseSlash => {
                let (year, month, day) = (numbers[0], numbers[1], numbers[2]);
                Some((year, month, day))
            }
            DateFormat::UsSlash | DateFormat::UsDash => {
                let (month, day, year) = (numbers[0], numbers[1], numbers[2]);
                Some((year, month, day))
            }
            DateFormat::EuropeanDash | DateFormat::EuropeanSlash => {
                let (day, month, year) = (numbers[0], numbers[1], numbers[2]);
                Some((year, month, day))
            }
        }
    }

    fn matches(&self, value: &str) -> bool {
        static PATTERNS: Lazy<Vec<(DateFormat, Regex)>> = Lazy::new(|| {
            vec![
                DateFormat::Iso8601,
                DateFormat::UsSlash,
                DateFormat::EuropeanDash,
                DateFormat::EuropeanSlash,
                DateFormat::JapaneseSlash,
                DateFormat::UsDash,
            ]
            .into_iter()
            .map(|format| (format, Regex::new(format.pattern()).unwrap()))
            .collect()
        });

        PATTERNS
            .iter()
            .find(|(format, _)| format == self)
            .map(|(_, regex)| regex.is_match(value))
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct DateType;

impl TypeDetection for DateType {
    fn detect_confidence(value: &str) -> f64 {
        Date::from_str(value).map_or(0.0, |_| 1.0)
    }

    fn is_definite_match(value: &str) -> bool {
        Date::from_str(value).is_some()
    }

    fn normalize(value: &str) -> Option<String> {
        Date::from_str(value).map(|date| date.to_format(DateFormat::Iso8601))
    }
}

impl DateType {
    fn is_valid_date(year: u32, month: u32, day: u32) -> bool {
        if year < 1000 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 {
            return false;
        }

        // Check days in month, including leap years
        let days_in_month = match month {
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    29
                } else {
                    28
                }
            }
            _ => 31,
        };

        day <= days_in_month
    }
}
