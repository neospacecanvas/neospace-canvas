use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeFormat {
    /// HH:MM:SS (e.g., 13:45:30)
    Military24H,
    /// HH:MM:SS.mmm (e.g., 13:45:30.123)
    Military24HWithMs,
    /// HH:MM:SS AM/PM (e.g., 01:45:30 PM)
    Standard12H,
    /// HH:MM AM/PM (e.g., 01:45 PM)
    Standard12HNoSeconds,
    /// HH:MM:SS±HH:MM (e.g., 13:45:30+01:00)
    Military24HWithTz,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateTimeFormat {
    /// ISO8601 (e.g., 2024-03-19T13:45:30Z)
    Iso8601,
    /// ISO8601 with milliseconds (e.g., 2024-03-19T13:45:30.123Z)
    Iso8601WithMs,
    /// RFC2822 (e.g., Tue, 19 Mar 2024 13:45:30 +0000)
    Rfc2822,
    /// Common format (e.g., 2024-03-19 13:45:30)
    CommonFormat,
    /// US format (e.g., 03/19/2024 01:45:30 PM)
    UsFormat,
    /// European format (e.g., 19-03-2024 13:45:30)
    EuropeanFormat,
}

#[derive(Debug, Clone)]
pub struct DateTime {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: Option<u32>,
    timezone_offset_minutes: Option<i32>, // Offset in minutes from UTC
    format: DateTimeFormat,
}

impl DateTime {
    pub fn new(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        millisecond: Option<u32>,
        timezone_offset_minutes: Option<i32>,
        format: DateTimeFormat,
    ) -> Option<Self> {
        if !Self::is_valid_datetime(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond.unwrap_or(0),
            timezone_offset_minutes.unwrap_or(0),
        ) {
            return None;
        }

        Some(DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            timezone_offset_minutes,
            format,
        })
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let clean_value = value.trim();
        if clean_value.is_empty() {
            return None;
        }

        // Try ISO8601 first as it's most unambiguous
        if let Some(dt) = Self::parse_iso8601(clean_value) {
            return Some(dt);
        }

        // Try RFC2822
        if let Some(dt) = Self::parse_rfc2822(clean_value) {
            return Some(dt);
        }

        // Try common formats
        if let Some(dt) = Self::parse_common_format(clean_value) {
            return Some(dt);
        }

        // Try US format
        if let Some(dt) = Self::parse_us_format(clean_value) {
            return Some(dt);
        }

        // Try European format
        if let Some(dt) = Self::parse_european_format(clean_value) {
            return Some(dt);
        }

        None
    }

    fn parse_iso8601(value: &str) -> Option<Self> {
        static ISO8601_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?(?:Z|([+-]\d{2}:?\d{2}))?$").unwrap()
        });

        fn parse_rfc2822(value: &str) -> Option<Self> {
            static RFC2822_PATTERN: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^(?:(?:Mon|Tue|Wed|Thu|Fri|Sat|Sun), )?(\d{1,2}) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) (\d{4}) (\d{2}):(\d{2}):(\d{2}) ([+-]\d{4}|[A-Z]{3})$").unwrap()
            });

            let captures = RFC2822_PATTERN.captures(value)?;

            let month_names = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];

            let day = captures.get(1)?.as_str().parse().ok()?;
            let month = month_names
                .iter()
                .position(|&m| m == captures.get(2)?.as_str())? as u32
                + 1;
            let year = captures.get(3)?.as_str().parse().ok()?;
            let hour = captures.get(4)?.as_str().parse().ok()?;
            let minute = captures.get(5)?.as_str().parse().ok()?;
            let second = captures.get(6)?.as_str().parse().ok()?;

            let timezone_offset = match captures.get(7)?.as_str() {
                // Handle numeric timezone
                tz if tz.len() == 5 => {
                    let sign = if tz.starts_with('-') { -1 } else { 1 };
                    let hours = tz[1..3].parse::<i32>().ok()?;
                    let minutes = tz[3..5].parse::<i32>().ok()?;
                    Some(sign * (hours * 60 + minutes))
                }
                // Common timezone abbreviations (simplified)
                "UTC" => Some(0),
                "GMT" => Some(0),
                "EST" => Some(-5 * 60),
                "EDT" => Some(-4 * 60),
                "CST" => Some(-6 * 60),
                "CDT" => Some(-5 * 60),
                "MST" => Some(-7 * 60),
                "MDT" => Some(-6 * 60),
                "PST" => Some(-8 * 60),
                "PDT" => Some(-7 * 60),
                _ => None,
            };

            DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                None,
                timezone_offset,
                DateTimeFormat::Rfc2822,
            )
        }

        fn parse_common_format(value: &str) -> Option<Self> {
            static COMMON_PATTERN: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^(\d{4})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?$")
                    .unwrap()
            });

            let captures = COMMON_PATTERN.captures(value)?;

            let year = captures.get(1)?.as_str().parse().ok()?;
            let month = captures.get(2)?.as_str().parse().ok()?;
            let day = captures.get(3)?.as_str().parse().ok()?;
            let hour = captures.get(4)?.as_str().parse().ok()?;
            let minute = captures.get(5)?.as_str().parse().ok()?;
            let second = captures.get(6)?.as_str().parse().ok()?;
            let millisecond = captures.get(7).map(|ms| ms.as_str().parse().ok()).flatten();

            DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
                None,
                DateTimeFormat::CommonFormat,
            )
        }

        fn parse_us_format(value: &str) -> Option<Self> {
            static US_PATTERN: Lazy<Regex> = Lazy::new(|| {
                Regex::new(
                    r"^(\d{1,2})/(\d{1,2})/(\d{4})\s+(\d{1,2}):(\d{1,2}):(\d{1,2})(?:\s*(AM|PM))?$",
                )
                .unwrap()
            });

            let captures = US_PATTERN.captures(value)?;

            let month = captures.get(1)?.as_str().parse().ok()?;
            let day = captures.get(2)?.as_str().parse().ok()?;
            let year = captures.get(3)?.as_str().parse().ok()?;
            let mut hour = captures.get(4)?.as_str().parse().ok()?;
            let minute = captures.get(5)?.as_str().parse().ok()?;
            let second = captures.get(6)?.as_str().parse().ok()?;

            // Handle AM/PM if present
            if let Some(ampm) = captures.get(7) {
                match ampm.as_str() {
                    "PM" if hour < 12 => hour += 12,
                    "AM" if hour == 12 => hour = 0,
                    _ => {}
                }
            }

            DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                None,
                None,
                DateTimeFormat::UsFormat,
            )
        }

        fn parse_european_format(value: &str) -> Option<Self> {
            static EUROPEAN_PATTERN: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^(\d{1,2})-(\d{1,2})-(\d{4})\s+(\d{2}):(\d{2}):(\d{2})$").unwrap()
            });

            let captures = EUROPEAN_PATTERN.captures(value)?;

            let day = captures.get(1)?.as_str().parse().ok()?;
            let month = captures.get(2)?.as_str().parse().ok()?;
            let year = captures.get(3)?.as_str().parse().ok()?;
            let hour = captures.get(4)?.as_str().parse().ok()?;
            let minute = captures.get(5)?.as_str().parse().ok()?;
            let second = captures.get(6)?.as_str().parse().ok()?;

            DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                None,
                None,
                DateTimeFormat::EuropeanFormat,
            )
        }

        let captures = ISO8601_PATTERN.captures(value)?;

        let year = captures.get(1)?.as_str().parse().ok()?;
        let month = captures.get(2)?.as_str().parse().ok()?;
        let day = captures.get(3)?.as_str().parse().ok()?;
        let hour = captures.get(4)?.as_str().parse().ok()?;
        let minute = captures.get(5)?.as_str().parse().ok()?;
        let second = captures.get(6)?.as_str().parse().ok()?;

        let millisecond = captures.get(7).map(|ms| ms.as_str().parse().ok()).flatten();

        let timezone_offset = captures
            .get(8)
            .map(|tz| {
                let tz_str = tz.as_str();
                let sign = if tz_str.starts_with('-') { -1 } else { 1 };
                let parts: Vec<&str> = tz_str[1..].split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(hours), Ok(minutes)) =
                        (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                    {
                        Some(sign * (hours * 60 + minutes))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flatten();

        DateTime::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            timezone_offset,
            if millisecond.is_some() {
                DateTimeFormat::Iso8601WithMs
            } else {
                DateTimeFormat::Iso8601
            },
        )
    }

    pub fn to_format(&self, target_format: DateTimeFormat) -> String {
        match target_format {
            DateTimeFormat::Iso8601 => {
                let tz = self
                    .timezone_offset_minutes
                    .map(|offset| {
                        let sign = if offset >= 0 { '+' } else { '-' };
                        let hours = offset.abs() / 60;
                        let minutes = offset.abs() % 60;
                        format!("{}{:02}:{:02}", sign, hours, minutes)
                    })
                    .unwrap_or_else(|| "Z".to_string());

                format!(
                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
                    self.year, self.month, self.day, self.hour, self.minute, self.second, tz
                )
            }
            DateTimeFormat::Iso8601WithMs => {
                let ms = self
                    .millisecond
                    .map(|ms| format!(".{:03}", ms))
                    .unwrap_or_else(|| "".to_string());
                let tz = self
                    .timezone_offset_minutes
                    .map(|offset| {
                        let sign = if offset >= 0 { '+' } else { '-' };
                        let hours = offset.abs() / 60;
                        let minutes = offset.abs() % 60;
                        format!("{}{:02}:{:02}", sign, hours, minutes)
                    })
                    .unwrap_or_else(|| "Z".to_string());

                format!(
                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{}",
                    self.year, self.month, self.day, self.hour, self.minute, self.second, ms, tz
                )
            }
            DateTimeFormat::CommonFormat => {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    self.year, self.month, self.day, self.hour, self.minute, self.second
                )
            }
            DateTimeFormat::UsFormat => {
                let hour = if self.hour == 0 {
                    12
                } else if self.hour > 12 {
                    self.hour - 12
                } else {
                    self.hour
                };
                let ampm = if self.hour >= 12 { "PM" } else { "AM" };
                format!(
                    "{:02}/{:02}/{:04} {:02}:{:02}:{:02} {}",
                    self.month, self.day, self.year, hour, self.minute, self.second, ampm
                )
            }
            DateTimeFormat::EuropeanFormat => {
                format!(
                    "{:02}-{:02}-{:04} {:02}:{:02}:{:02}",
                    self.day, self.month, self.year, self.hour, self.minute, self.second
                )
            }
            DateTimeFormat::Rfc2822 => {
                let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let months = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];

                let day_of_week = self.day_of_week();
                let tz = self
                    .timezone_offset_minutes
                    .map(|offset| {
                        let sign = if offset >= 0 { '+' } else { '-' };
                        let hours = offset.abs() / 60;
                        let minutes = offset.abs() % 60;
                        format!("{}{:02}{:02}", sign, hours, minutes)
                    })
                    .unwrap_or_else(|| "+0000".to_string());

                format!(
                    "{}, {:02} {} {:04} {:02}:{:02}:{:02} {}",
                    days[day_of_week as usize],
                    self.day,
                    months[(self.month - 1) as usize],
                    self.year,
                    self.hour,
                    self.minute,
                    self.second,
                    tz
                )
            }
        }
    }

    fn is_valid_datetime(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        millisecond: u32,
        timezone_offset_minutes: i32,
    ) -> bool {
        // Validate date components
        if !Self::is_valid_date(year, month, day) {
            return false;
        }

        // Validate time components
        if hour >= 24 || minute >= 60 || second >= 60 || millisecond >= 1000 {
            return false;
        }

        // Validate timezone offset
        if timezone_offset_minutes.abs() > 24 * 60 {
            return false;
        }

        true
    }

    fn is_valid_date(year: u32, month: u32, day: u32) -> bool {
        if year < 1000 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 {
            return false;
        }

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

    fn day_of_week(&self) -> u32 {
        // Implementation of Zeller's congruence
        let (year, month) = if self.month <= 2 {
            (self.year - 1, self.month + 12)
        } else {
            (self.year, self.month)
        };

        let k = year % 100;
        let j = year / 100;

        let h = (self.day as u32
            + ((13 * (month + 1)) / 5) as u32
            + k
            + (k / 4) as u32
            + (j / 4) as u32
            + 5 * j as u32)
            % 7;

        (h + 6) % 7 // Adjust to make Sunday = 0, Monday = 1, etc.
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_format(self.format))
    }
}
