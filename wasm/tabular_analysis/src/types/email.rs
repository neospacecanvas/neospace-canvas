use super::TypeDetection;
use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Updated pattern to prevent consecutive dots and require proper domain structure
        Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9._%+-]*[a-zA-Z0-9]@([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap(),
        // Stricter pattern with additional checks
        Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9._%+-]{0,63}@(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?\.){1,8}[a-zA-Z]{2,63}$").unwrap(),
    ]
});

#[derive(Debug)]
pub struct EmailType;

impl TypeDetection for EmailType {
    fn detect_confidence(value: &str) -> f64 {
        let clean_value = value.trim().to_lowercase();
        if clean_value.is_empty() {
            return 0.0;
        }

        if Self::is_definite_match(&clean_value) {
            return 1.0;
        }

        // Check for basic email indicators with improved validation
        if clean_value.contains('@') {
            let parts: Vec<&str> = clean_value.split('@').collect();
            if parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.') {
                // Additional checks for domain part
                let domain_parts: Vec<&str> = parts[1].split('.').collect();
                if domain_parts.iter().all(|&p| !p.is_empty()) && domain_parts.len() >= 2 {
                    return 0.7;
                }
            }
            return 0.3;
        }

        0.0
    }

    fn is_definite_match(value: &str) -> bool {
        let clean_value = value.trim().to_lowercase();
        if clean_value.is_empty() {
            return false;
        }

        // Additional pre-checks before regex
        if clean_value.contains("..") || clean_value.starts_with('.') || clean_value.ends_with('.')
        {
            return false;
        }

        EMAIL_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(&clean_value))
    }

    fn normalize(value: &str) -> Option<String> {
        let clean_value = value.trim().to_lowercase();
        if clean_value.is_empty() {
            return None;
        }

        if Self::is_definite_match(&clean_value) {
            Some(clean_value)
        } else {
            None
        }
    }
}
