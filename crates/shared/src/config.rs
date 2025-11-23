//! Configuration management utilities

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub migrate_on_start: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/app".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            migrate_on_start: true,
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
    pub response_timeout: u64,
    pub connection_timeout: u64,
    pub default_ttl: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connect_timeout: 5,
            response_timeout: 5,
            connection_timeout: 5,
            default_ttl: 3600,
        }
    }
}

/// Kafka configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub group_id: String,
    pub client_id: String,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub session_timeout_ms: u32,
    pub heartbeat_interval_ms: u32,
    pub max_poll_interval_ms: u32,
    pub security_protocol: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: vec!["localhost:9092".to_string()],
            group_id: "default-group".to_string(),
            client_id: "rust-microservice".to_string(),
            auto_offset_reset: "earliest".to_string(),
            enable_auto_commit: true,
            session_timeout_ms: 30000,
            heartbeat_interval_ms: 3000,
            max_poll_interval_ms: 300000,
            security_protocol: None,
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
        }
    }
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub max_connections: usize,
    pub max_connection_rate: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            keep_alive: 75,
            client_timeout: 5000,
            client_shutdown: 5000,
            max_connections: 25000,
            max_connection_rate: 256,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
    pub jaeger_endpoint: Option<String>,
    pub service_name: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_path: None,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_files: Some(10),
            jaeger_endpoint: None,
            service_name: "rust-microservice".to_string(),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub namespace: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
            namespace: "app".to_string(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub bcrypt_cost: u32,
    pub cors_origins: Vec<String>,
    pub cors_methods: Vec<String>,
    pub cors_headers: Vec<String>,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key".to_string(),
            jwt_expiration: 3600,
            bcrypt_cost: 12,
            cors_origins: vec!["*".to_string()],
            cors_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            cors_headers: vec!["*".to_string()],
            rate_limit_requests: 100,
            rate_limit_window: 60,
        }
    }
}

/// Base application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: String,
    pub service_name: String,
    pub version: String,
    pub debug: bool,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub security: SecurityConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            service_name: "rust-microservice".to_string(),
            version: "0.1.0".to_string(),
            debug: true,
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            kafka: KafkaConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from files and environment variables
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Yaml::file("config/default.yml"))
            .merge(Yaml::file(format!("config/{}.yml", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()))))
            .merge(Env::prefixed("APP_"))
            .extract()
    }

    /// Load configuration with custom config path
    pub fn load_from_path(config_path: &str) -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Yaml::file(format!("{}/default.yml", config_path)))
            .merge(Yaml::file(format!("{}/{}.yml", config_path, std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()))))
            .merge(Env::prefixed("APP_"))
            .extract()
    }

    /// Get database URL as parsed URL
    pub fn database_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.database.url)
    }

    /// Get Redis URL as parsed URL
    pub fn redis_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.redis.url)
    }

    /// Get server bind address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get metrics bind address
    pub fn metrics_address(&self) -> String {
        format!("{}:{}", self.metrics.host, self.metrics.port)
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Get JWT expiration as Duration
    pub fn jwt_expiration_duration(&self) -> Duration {
        Duration::from_secs(self.security.jwt_expiration)
    }

    /// Get database connect timeout as Duration
    pub fn database_connect_timeout(&self) -> Duration {
        Duration::from_secs(self.database.connect_timeout)
    }

    /// Get Redis connect timeout as Duration
    pub fn redis_connect_timeout(&self) -> Duration {
        Duration::from_secs(self.redis.connect_timeout)
    }
}

/// Configuration validation trait
pub trait ValidateConfig {
    fn validate(&self) -> Result<(), String>;
}

impl ValidateConfig for AppConfig {
    fn validate(&self) -> Result<(), String> {
        // Validate database URL
        self.database_url()
            .map_err(|e| format!("Invalid database URL: {}", e))?;

        // Validate Redis URL
        self.redis_url()
            .map_err(|e| format!("Invalid Redis URL: {}", e))?;

        // Validate Kafka brokers
        if self.kafka.brokers.is_empty() {
            return Err("Kafka brokers cannot be empty".to_string());
        }

        // Validate JWT secret
        if self.security.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters".to_string());
        }

        // Validate server port
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        Ok(())
    }
}