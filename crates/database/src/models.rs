//! Database models and entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Entity, MultiTenant, SoftDelete, TenantId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for User {
    type Id = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl MultiTenant for User {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

impl SoftDelete for User {
    fn deleted_at(&self) -> &Option<DateTime<Utc>> {
        &self.deleted_at
    }
}

/// Order entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub order_number: String,
    pub status: OrderStatus,
    pub total_amount: i64, // Amount in cents
    pub currency: String,
    pub items: serde_json::Value,
    pub shipping_address: Option<serde_json::Value>,
    pub billing_address: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for Order {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl MultiTenant for Order {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

impl SoftDelete for Order {
    fn deleted_at(&self) -> &Option<DateTime<Utc>> {
        &self.deleted_at
    }
}

/// Order status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

/// Payment entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub order_id: Uuid,
    pub user_id: UserId,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub amount: i64, // Amount in cents
    pub currency: String,
    pub external_id: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub failure_reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Payment {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl MultiTenant for Payment {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

/// Payment method enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "lowercase")]
pub enum PaymentMethod {
    CreditCard,
    DebitCard,
    PayPal,
    BankTransfer,
    Cryptocurrency,
    Cash,
}

/// Payment status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Refunded,
}

/// Background job entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub tenant_id: Option<TenantId>,
    pub job_type: String,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Job {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Job status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Event entity for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub tenant_id: Option<TenantId>,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub version: i64,
    pub payload: serde_json::Value,
    pub metadata: serde_json::Value,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub user_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
}

impl Entity for Event {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.created_at // Events are immutable
    }
}

/// Session entity for user sessions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub expires_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Session {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl MultiTenant for Session {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

/// Audit log entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<UserId>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub correlation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Entity for AuditLog {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.created_at // Audit logs are immutable
    }
}

/// Tenant entity for multi-tenancy
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for Tenant {
    type Id = TenantId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl SoftDelete for Tenant {
    fn deleted_at(&self) -> &Option<DateTime<Utc>> {
        &self.deleted_at
    }
}

/// Create user DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub tenant_id: TenantId,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Update user DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
}

/// Create order DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderDto {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub items: serde_json::Value,
    pub total_amount: i64,
    pub currency: String,
    pub shipping_address: Option<serde_json::Value>,
    pub billing_address: Option<serde_json::Value>,
    pub notes: Option<String>,
}

/// Update order DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderDto {
    pub status: Option<OrderStatus>,
    pub items: Option<serde_json::Value>,
    pub total_amount: Option<i64>,
    pub shipping_address: Option<serde_json::Value>,
    pub billing_address: Option<serde_json::Value>,
    pub notes: Option<String>,
}