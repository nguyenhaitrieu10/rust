//! API service specific configuration

use shared::AppConfig;
use serde::{Deserialize, Serialize};

/// API service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base application configuration
    #[serde(flatten)]
    pub app: AppConfig,
    
    /// API specific settings
    pub api: ApiSettings,
}

/// API specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSettings {
    /// API version
    pub version: String,
    
    /// API title
    pub title: String,
    
    /// API description
    pub description: String,
    
    /// Maximum request size in bytes
    pub max_request_size: usize,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// Enable API documentation
    pub enable_docs: bool,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// Enable request logging
    pub enable_request_logging: bool,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Pagination settings
    pub pagination: PaginationSettings,
    
    /// Authentication settings
    pub auth: AuthSettings,
}

/// Pagination settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationSettings {
    /// Default page size
    pub default_page_size: u32,
    
    /// Maximum page size
    pub max_page_size: u32,
    
    /// Maximum offset
    pub max_offset: u32,
}

/// Authentication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    /// JWT secret key
    pub jwt_secret: String,
    
    /// JWT expiration time in seconds
    pub jwt_expiration: u64,
    
    /// JWT issuer
    pub jwt_issuer: String,
    
    /// JWT audience
    pub jwt_audience: String,
    
    /// Enable refresh tokens
    pub enable_refresh_tokens: bool,
    
    /// Refresh token expiration in seconds
    pub refresh_token_expiration: u64,
    
    /// Password minimum length
    pub password_min_length: usize,
    
    /// Password maximum length
    pub password_max_length: usize,
    
    /// Enable password complexity requirements
    pub password_complexity: bool,
    
    /// Account lockout settings
    pub lockout: LockoutSettings,
}

/// Account lockout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutSettings {
    /// Enable account lockout
    pub enabled: bool,
    
    /// Maximum failed attempts before lockout
    pub max_attempts: u32,
    
    /// Lockout duration in seconds
    pub lockout_duration: u64,
    
    /// Reset failed attempts after this duration (seconds)
    pub reset_duration: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            api: ApiSettings::default(),
        }
    }
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            version: "v1".to_string(),
            title: "Rust Microservices API".to_string(),
            description: "A professional, scalable Rust microservices API".to_string(),
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout: 30,
            enable_docs: true,
            enable_cors: true,
            enable_request_logging: true,
            enable_compression: true,
            pagination: PaginationSettings::default(),
            auth: AuthSettings::default(),
        }
    }
}

impl Default for PaginationSettings {
    fn default() -> Self {
        Self {
            default_page_size: 20,
            max_page_size: 1000,
            max_offset: 100000,
        }
    }
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiration: 3600, // 1 hour
            jwt_issuer: "rust-microservices".to_string(),
            jwt_audience: "api-users".to_string(),
            enable_refresh_tokens: true,
            refresh_token_expiration: 604800, // 7 days
            password_min_length: 8,
            password_max_length: 128,
            password_complexity: true,
            lockout: LockoutSettings::default(),
        }
    }
}

impl Default for LockoutSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            lockout_duration: 900, // 15 minutes
            reset_duration: 3600,  // 1 hour
        }
    }
}

impl ApiConfig {
    /// Load API configuration
    pub fn load() -> Result<Self, figment::Error> {
        use figment::{providers::{Env, Format, Yaml}, Figment};
        
        Figment::new()
            .merge(Yaml::file("config/api.yml"))
            .merge(Yaml::file(format!("config/api-{}.yml", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()))))
            .merge(Env::prefixed("API_"))
            .extract()
    }
    
    /// Get JWT expiration as Duration
    pub fn jwt_expiration_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.api.auth.jwt_expiration)
    }
    
    /// Get refresh token expiration as Duration
    pub fn refresh_token_expiration_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.api.auth.refresh_token_expiration)
    }
    
    /// Get request timeout as Duration
    pub fn request_timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.api.request_timeout)
    }
    
    /// Get lockout duration as Duration
    pub fn lockout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.api.auth.lockout.lockout_duration)
    }
    
    /// Validate API configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate base app config
        self.app.validate()?;
        
        // Validate JWT secret
        if self.api.auth.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters".to_string());
        }
        
        // Validate pagination settings
        if self.api.pagination.default_page_size > self.api.pagination.max_page_size {
            return Err("Default page size cannot be greater than max page size".to_string());
        }
        
        // Validate password settings
        if self.api.auth.password_min_length > self.api.auth.password_max_length {
            return Err("Password min length cannot be greater than max length".to_string());
        }
        
        // Validate request size
        if self.api.max_request_size == 0 {
            return Err("Max request size cannot be zero".to_string());
        }
        
        Ok(())
    }
}

/// API response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetadata {
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: uuid::Uuid,
    pub service: String,
}

impl ApiMetadata {
    pub fn new(version: String, service: String) -> Self {
        Self {
            version,
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4(),
            service,
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
    pub details: Option<serde_json::Value>,
    pub metadata: ApiMetadata,
}

impl ApiErrorResponse {
    pub fn new(
        error: String,
        message: String,
        code: u16,
        details: Option<serde_json::Value>,
        metadata: ApiMetadata,
    ) -> Self {
        Self {
            error,
            message,
            code,
            details,
            metadata,
        }
    }
}

/// API success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponse<T> {
    pub data: T,
    pub metadata: ApiMetadata,
}

impl<T> ApiSuccessResponse<T> {
    pub fn new(data: T, metadata: ApiMetadata) -> Self {
        Self { data, metadata }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.api.version, "v1");
        assert_eq!(config.api.pagination.default_page_size, 20);
        assert_eq!(config.api.auth.password_min_length, 8);
    }

    #[test]
    fn test_api_config_validation() {
        let mut config = ApiConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid JWT secret should fail
        config.api.auth.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
        
        // Reset and test pagination
        config = ApiConfig::default();
        config.api.pagination.default_page_size = 2000;
        config.api.pagination.max_page_size = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_api_metadata() {
        let metadata = ApiMetadata::new("v1".to_string(), "api-service".to_string());
        assert_eq!(metadata.version, "v1");
        assert_eq!(metadata.service, "api-service");
    }
}