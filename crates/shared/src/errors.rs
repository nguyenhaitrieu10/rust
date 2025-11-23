//! Common error types used across all services

use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Kafka error: {0}")]
    Kafka(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Validation(_) | AppError::BadRequest(_) => 400,
            AppError::Authentication(_) => 401,
            AppError::Authorization(_) => 403,
            AppError::NotFound(_) => 404,
            AppError::Conflict(_) => 409,
            _ => 500,
        }
    }

    /// Check if this error should be logged as an error (vs warning/info)
    pub fn should_log_error(&self) -> bool {
        matches!(
            self,
            AppError::Database(_)
                | AppError::Redis(_)
                | AppError::Kafka(_)
                | AppError::Internal(_)
                | AppError::ExternalService(_)
                | AppError::Configuration(_)
                | AppError::Network(_)
                | AppError::Io(_)
                | AppError::Generic(_)
        )
    }
}

/// Validation error details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Multiple validation errors
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        AppError::Validation(format!("Validation failed: {:?}", errors.errors))
    }
}