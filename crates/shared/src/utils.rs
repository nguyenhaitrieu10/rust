//! Utility functions used across all services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{AppError, AppResult, CorrelationId};

/// Generate a new correlation ID
pub fn generate_correlation_id() -> CorrelationId {
    Uuid::new_v4()
}

/// Generate a new UUID
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Get current UTC timestamp
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Convert SystemTime to DateTime<Utc>
pub fn system_time_to_datetime(time: SystemTime) -> AppResult<DateTime<Utc>> {
    let duration = time.duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Invalid system time: {}", e)))?;
    
    DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
        .ok_or_else(|| AppError::Internal("Failed to convert system time to datetime".to_string()))
}

/// Convert DateTime<Utc> to SystemTime
pub fn datetime_to_system_time(datetime: DateTime<Utc>) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(datetime.timestamp() as u64)
}

/// Format duration as human readable string
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let millis = duration.subsec_millis();

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{}s", seconds, millis / 100)
    } else {
        format!("{}ms", millis)
    }
}

/// Parse duration from string (e.g., "1h30m", "45s", "2d")
pub fn parse_duration(s: &str) -> AppResult<Duration> {
    let s = s.trim().to_lowercase();
    
    if s.is_empty() {
        return Err(AppError::BadRequest("Empty duration string".to_string()));
    }

    let mut total_seconds = 0u64;
    let mut current_number = String::new();
    
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else {
            if current_number.is_empty() {
                return Err(AppError::BadRequest(format!("Invalid duration format: {}", s)));
            }
            
            let number: u64 = current_number.parse()
                .map_err(|_| AppError::BadRequest(format!("Invalid number in duration: {}", current_number)))?;
            
            let multiplier = match ch {
                's' => 1,
                'm' => 60,
                'h' => 3600,
                'd' => 86400,
                _ => return Err(AppError::BadRequest(format!("Invalid duration unit: {}", ch))),
            };
            
            total_seconds += number * multiplier;
            current_number.clear();
        }
    }
    
    if !current_number.is_empty() {
        // Assume seconds if no unit specified
        let number: u64 = current_number.parse()
            .map_err(|_| AppError::BadRequest(format!("Invalid number in duration: {}", current_number)))?;
        total_seconds += number;
    }
    
    Ok(Duration::from_secs(total_seconds))
}

/// Sanitize string for logging (remove sensitive data)
pub fn sanitize_for_logging(input: &str) -> String {
    // Remove common sensitive patterns
    let patterns = [
        (r"password[\"':\s]*[\"']([^\"']+)[\"']", "password: \"***\""),
        (r"token[\"':\s]*[\"']([^\"']+)[\"']", "token: \"***\""),
        (r"secret[\"':\s]*[\"']([^\"']+)[\"']", "secret: \"***\""),
        (r"key[\"':\s]*[\"']([^\"']+)[\"']", "key: \"***\""),
        (r"authorization[\"':\s]*[\"']([^\"']+)[\"']", "authorization: \"***\""),
    ];
    
    let mut result = input.to_string();
    for (pattern, replacement) in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    
    result
}

/// Truncate string to maximum length
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Convert bytes to human readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Retry with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug,
{
    let mut delay = initial_delay;
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);
                
                if attempt < max_attempts {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * backoff_multiplier) as u64),
                        max_delay,
                    );
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

/// Hash password using Argon2
pub fn hash_password(password: &str) -> AppResult<String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    use argon2::{Argon2, PasswordVerifier};
    use argon2::password_hash::PasswordHash;
    
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate random string
pub fn generate_random_string(length: usize) -> String {
    use rand::{distributions::Alphanumeric, Rng};
    
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    use regex::Regex;
    
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
    
    email_regex.is_match(email)
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Extract domain from email
pub fn extract_domain_from_email(email: &str) -> Option<String> {
    email.split('@').nth(1).map(|s| s.to_lowercase())
}

/// Mask sensitive data for logging
pub fn mask_sensitive_data(data: &str, visible_chars: usize) -> String {
    if data.len() <= visible_chars * 2 {
        "*".repeat(data.len())
    } else {
        let start = &data[..visible_chars];
        let end = &data[data.len() - visible_chars..];
        format!("{}***{}", start, end)
    }
}

/// Convert snake_case to camelCase
pub fn snake_to_camel_case(snake_str: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for ch in snake_str.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Convert camelCase to snake_case
pub fn camel_to_snake_case(camel_str: &str) -> String {
    let mut result = String::new();
    
    for (i, ch) in camel_str.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    
    result
}

/// Pagination helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationHelper {
    pub offset: u32,
    pub limit: u32,
    pub total: Option<u64>,
}

impl PaginationHelper {
    pub fn new(offset: u32, limit: u32, total: Option<u64>) -> Self {
        Self { offset, limit, total }
    }
    
    pub fn has_next(&self) -> bool {
        if let Some(total) = self.total {
            (self.offset + self.limit) < total as u32
        } else {
            false
        }
    }
    
    pub fn has_prev(&self) -> bool {
        self.offset > 0
    }
    
    pub fn next_offset(&self) -> Option<u32> {
        if self.has_next() {
            Some(self.offset + self.limit)
        } else {
            None
        }
    }
    
    pub fn prev_offset(&self) -> Option<u32> {
        if self.has_prev() {
            Some(self.offset.saturating_sub(self.limit))
        } else {
            None
        }
    }
    
    pub fn total_pages(&self) -> Option<u32> {
        self.total.map(|total| {
            ((total as f64) / (self.limit as f64)).ceil() as u32
        })
    }
    
    pub fn current_page(&self) -> u32 {
        (self.offset / self.limit) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0ms");
        assert_eq!(format_duration(Duration::from_secs(1)), "1.0s");
        assert_eq!(format_duration(Duration::from_secs(61)), "1m 1s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }
    
    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("1h30m").unwrap(), Duration::from_secs(5400));
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
    
    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }
    
    #[test]
    fn test_snake_to_camel_case() {
        assert_eq!(snake_to_camel_case("hello_world"), "helloWorld");
        assert_eq!(snake_to_camel_case("user_id"), "userId");
        assert_eq!(snake_to_camel_case("simple"), "simple");
    }
    
    #[test]
    fn test_camel_to_snake_case() {
        assert_eq!(camel_to_snake_case("helloWorld"), "hello_world");
        assert_eq!(camel_to_snake_case("userId"), "user_id");
        assert_eq!(camel_to_snake_case("simple"), "simple");
    }
    
    #[test]
    fn test_mask_sensitive_data() {
        assert_eq!(mask_sensitive_data("password123", 2), "pa***23");
        assert_eq!(mask_sensitive_data("short", 2), "*****");
        assert_eq!(mask_sensitive_data("a", 2), "*");
    }
}