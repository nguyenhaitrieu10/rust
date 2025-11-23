//! Common traits used across all services

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use crate::{AppResult, CorrelationId, PaginationParams, PaginatedResponse};

/// Repository trait for data access layer
#[async_trait]
pub trait Repository<T, ID> {
    /// Find entity by ID
    async fn find_by_id(&self, id: &ID) -> AppResult<Option<T>>;
    
    /// Find all entities with pagination
    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>>;
    
    /// Create new entity
    async fn create(&self, entity: &T) -> AppResult<T>;
    
    /// Update existing entity
    async fn update(&self, id: &ID, entity: &T) -> AppResult<T>;
    
    /// Delete entity by ID
    async fn delete(&self, id: &ID) -> AppResult<bool>;
    
    /// Check if entity exists
    async fn exists(&self, id: &ID) -> AppResult<bool>;
    
    /// Count total entities
    async fn count(&self) -> AppResult<u64>;
}

/// Service trait for business logic layer
#[async_trait]
pub trait Service<T, ID, CreateDto, UpdateDto> {
    /// Get entity by ID
    async fn get(&self, id: &ID) -> AppResult<T>;
    
    /// List entities with pagination
    async fn list(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>>;
    
    /// Create new entity
    async fn create(&self, dto: &CreateDto) -> AppResult<T>;
    
    /// Update existing entity
    async fn update(&self, id: &ID, dto: &UpdateDto) -> AppResult<T>;
    
    /// Delete entity
    async fn delete(&self, id: &ID) -> AppResult<bool>;
}

/// Event publisher trait
#[async_trait]
pub trait EventPublisher {
    /// Publish event to message broker
    async fn publish<T>(&self, topic: &str, event: &T, correlation_id: CorrelationId) -> AppResult<()>
    where
        T: Serialize + Send + Sync;
    
    /// Publish event with key for partitioning
    async fn publish_with_key<T>(&self, topic: &str, key: &str, event: &T, correlation_id: CorrelationId) -> AppResult<()>
    where
        T: Serialize + Send + Sync;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<T> {
    /// Handle incoming event
    async fn handle(&self, event: &T, correlation_id: CorrelationId) -> AppResult<()>;
    
    /// Get event type this handler processes
    fn event_type(&self) -> &'static str;
}

/// Cache trait for caching operations
#[async_trait]
pub trait Cache {
    /// Get value from cache
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;
    
    /// Set value in cache with TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;
    
    /// Delete value from cache
    async fn delete(&self, key: &str) -> AppResult<bool>;
    
    /// Check if key exists in cache
    async fn exists(&self, key: &str) -> AppResult<bool>;
    
    /// Set expiration for existing key
    async fn expire(&self, key: &str, ttl: u64) -> AppResult<bool>;
    
    /// Get multiple values from cache
    async fn get_many<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;
    
    /// Set multiple values in cache
    async fn set_many<T>(&self, items: &[(String, T)], ttl: Option<u64>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;
}

/// Job processor trait for background jobs
#[async_trait]
pub trait JobProcessor<T> {
    /// Process job payload
    async fn process(&self, payload: &T, correlation_id: CorrelationId) -> AppResult<()>;
    
    /// Get job type this processor handles
    fn job_type(&self) -> &'static str;
    
    /// Get maximum retry attempts
    fn max_retries(&self) -> u32 {
        3
    }
    
    /// Get retry delay in seconds
    fn retry_delay(&self) -> u64 {
        60
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck {
    /// Check if service is healthy
    async fn check(&self) -> AppResult<bool>;
    
    /// Get service name
    fn name(&self) -> &'static str;
    
    /// Get health check timeout in seconds
    fn timeout(&self) -> u64 {
        5
    }
}

/// Metrics collector trait
pub trait MetricsCollector {
    /// Increment counter metric
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);
    
    /// Record histogram value
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    
    /// Set gauge value
    fn set_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    
    /// Record timing
    fn record_timing(&self, name: &str, duration: std::time::Duration, labels: &[(&str, &str)]);
}

/// Validator trait for input validation
pub trait Validator<T> {
    type Error;
    
    /// Validate input
    fn validate(&self, input: &T) -> Result<(), Self::Error>;
}

/// Serializer trait for data serialization
pub trait Serializer {
    /// Serialize data to bytes
    fn serialize<T>(&self, data: &T) -> AppResult<Vec<u8>>
    where
        T: Serialize;
    
    /// Deserialize data from bytes
    fn deserialize<T>(&self, data: &[u8]) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>;
}

/// Connection pool trait
#[async_trait]
pub trait ConnectionPool<T> {
    /// Get connection from pool
    async fn get(&self) -> AppResult<T>;
    
    /// Get pool status
    async fn status(&self) -> PoolStatus;
}

/// Pool status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub pending_requests: u32,
}

/// Middleware trait for request processing
#[async_trait]
pub trait Middleware<Req, Res> {
    /// Process request before handler
    async fn before(&self, request: &mut Req) -> AppResult<()>;
    
    /// Process response after handler
    async fn after(&self, request: &Req, response: &mut Res) -> AppResult<()>;
}

/// Authentication trait
#[async_trait]
pub trait Authenticator {
    type User;
    type Token;
    
    /// Authenticate user with credentials
    async fn authenticate(&self, credentials: &str) -> AppResult<Self::User>;
    
    /// Generate token for user
    async fn generate_token(&self, user: &Self::User) -> AppResult<Self::Token>;
    
    /// Validate token
    async fn validate_token(&self, token: &str) -> AppResult<Self::User>;
    
    /// Refresh token
    async fn refresh_token(&self, token: &str) -> AppResult<Self::Token>;
}

/// Authorization trait
#[async_trait]
pub trait Authorizer {
    type User;
    type Resource;
    type Permission;
    
    /// Check if user has permission for resource
    async fn authorize(&self, user: &Self::User, resource: &Self::Resource, permission: &Self::Permission) -> AppResult<bool>;
    
    /// Get user permissions for resource
    async fn get_permissions(&self, user: &Self::User, resource: &Self::Resource) -> AppResult<Vec<Self::Permission>>;
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter {
    /// Check if request is allowed
    async fn is_allowed(&self, key: &str) -> AppResult<bool>;
    
    /// Get remaining requests for key
    async fn remaining(&self, key: &str) -> AppResult<u32>;
    
    /// Reset rate limit for key
    async fn reset(&self, key: &str) -> AppResult<()>;
}

/// Circuit breaker trait
#[async_trait]
pub trait CircuitBreaker {
    /// Execute function with circuit breaker protection
    async fn execute<F, T>(&self, f: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>> + Send,
        T: Send;
    
    /// Get circuit breaker state
    fn state(&self) -> CircuitBreakerState;
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Audit logger trait
#[async_trait]
pub trait AuditLogger {
    /// Log audit event
    async fn log(&self, event: &AuditEvent) -> AppResult<()>;
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub correlation_id: CorrelationId,
    pub metadata: serde_json::Value,
}