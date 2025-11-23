//! Common types used across all services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// User ID type
pub type UserId = Uuid;

/// Tenant ID type for multi-tenancy
pub type TenantId = Uuid;

/// Correlation ID for request tracing
pub type CorrelationId = Uuid;

/// Common pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<u32>,
    
    #[validate(range(min = 0))]
    pub offset: Option<u32>,
    
    pub cursor: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            cursor: None,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: Option<u64>,
    pub limit: u32,
    pub offset: u32,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub correlation_id: CorrelationId,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, correlation_id: CorrelationId) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            correlation_id,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String, correlation_id: CorrelationId) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            correlation_id,
            timestamp: Utc::now(),
        }
    }
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service: String,
    pub version: String,
    pub status: ServiceStatus,
    pub timestamp: DateTime<Utc>,
    pub dependencies: Vec<DependencyHealth>,
}

/// Service status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Dependency health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub status: ServiceStatus,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

/// Event metadata for Kafka messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub correlation_id: CorrelationId,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<UserId>,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

impl EventMetadata {
    pub fn new(
        event_type: impl Into<String>,
        source_service: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            source_service: source_service.into(),
            correlation_id,
            tenant_id: None,
            user_id: None,
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        }
    }

    pub fn with_tenant(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_user(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Generic event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    pub metadata: EventMetadata,
    pub payload: T,
}

impl<T> Event<T> {
    pub fn new(
        event_type: impl Into<String>,
        source_service: impl Into<String>,
        correlation_id: CorrelationId,
        payload: T,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(event_type, source_service, correlation_id),
            payload,
        }
    }
}

/// Job status for background processing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Background job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub job_id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub correlation_id: CorrelationId,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<UserId>,
}

/// Generic job wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job<T> {
    pub metadata: JobMetadata,
    pub payload: T,
}

/// Cache key builder helper
#[derive(Debug, Clone)]
pub struct CacheKey {
    parts: Vec<String>,
}

impl CacheKey {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            parts: vec![prefix.into()],
        }
    }

    pub fn add(mut self, part: impl Into<String>) -> Self {
        self.parts.push(part.into());
        self
    }

    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Database entity trait
pub trait Entity {
    type Id;
    
    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> &DateTime<Utc>;
    fn updated_at(&self) -> &DateTime<Utc>;
}

/// Soft delete trait
pub trait SoftDelete {
    fn deleted_at(&self) -> &Option<DateTime<Utc>>;
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// Multi-tenant entity trait
pub trait MultiTenant {
    fn tenant_id(&self) -> &TenantId;
}