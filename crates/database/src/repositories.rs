//! Repository implementations for data access

use async_trait::async_trait;
use shared::{AppResult, PaginationParams, PaginatedResponse, Repository, UserId, TenantId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::*;

/// User repository implementation
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, username, password_hash, first_name, last_name,
                   is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            FROM users 
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, username, password_hash, first_name, last_name,
                   is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            FROM users 
            WHERE username = $1 AND deleted_at IS NULL
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find users by tenant
    pub async fn find_by_tenant(&self, tenant_id: &TenantId, params: &PaginationParams) -> AppResult<PaginatedResponse<User>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, username, password_hash, first_name, last_name,
                   is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            FROM users 
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE tenant_id = $1 AND deleted_at IS NULL",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: users,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, user_id: &UserId) -> AppResult<()> {
        sqlx::query!(
            "UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Repository<User, UserId> for UserRepository {
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, username, password_hash, first_name, last_name,
                   is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            FROM users 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<User>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, username, password_hash, first_name, last_name,
                   is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            FROM users 
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await?
            .count
            .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: users,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let created_user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, tenant_id, email, username, password_hash, first_name, last_name,
                              is_active, is_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, tenant_id, email, username, password_hash, first_name, last_name,
                      is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            "#,
            user.id,
            user.tenant_id,
            user.email,
            user.username,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.is_active,
            user.is_verified,
            user.created_at,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_user)
    }

    async fn update(&self, id: &UserId, user: &User) -> AppResult<User> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET email = $2, username = $3, password_hash = $4, first_name = $5, last_name = $6,
                is_active = $7, is_verified = $8, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, tenant_id, email, username, password_hash, first_name, last_name,
                      is_active, is_verified, last_login_at, created_at, updated_at, deleted_at
            "#,
            id,
            user.email,
            user.username,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.is_active,
            user.is_verified
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    async fn delete(&self, id: &UserId) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE users SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &UserId) -> AppResult<bool> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND deleted_at IS NULL) as exists",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists.unwrap_or(false))
    }

    async fn count(&self) -> AppResult<u64> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0) as u64)
    }
}

/// Order repository implementation
pub struct OrderRepository {
    pool: PgPool,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find orders by user
    pub async fn find_by_user(&self, user_id: &UserId, params: &PaginationParams) -> AppResult<PaginatedResponse<Order>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                   total_amount, currency, items, shipping_address, billing_address, notes,
                   created_at, updated_at, deleted_at
            FROM orders 
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM orders WHERE user_id = $1 AND deleted_at IS NULL",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: orders,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }

    /// Find orders by status
    pub async fn find_by_status(&self, status: &OrderStatus, params: &PaginationParams) -> AppResult<PaginatedResponse<Order>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                   total_amount, currency, items, shipping_address, billing_address, notes,
                   created_at, updated_at, deleted_at
            FROM orders 
            WHERE status = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            status as &OrderStatus,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM orders WHERE status = $1 AND deleted_at IS NULL",
            status as &OrderStatus
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: orders,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }
}

#[async_trait]
impl Repository<Order, Uuid> for OrderRepository {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Order>> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                   total_amount, currency, items, shipping_address, billing_address, notes,
                   created_at, updated_at, deleted_at
            FROM orders 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(order)
    }

    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<Order>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                   total_amount, currency, items, shipping_address, billing_address, notes,
                   created_at, updated_at, deleted_at
            FROM orders 
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!("SELECT COUNT(*) as count FROM orders WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await?
            .count
            .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: orders,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }

    async fn create(&self, order: &Order) -> AppResult<Order> {
        let created_order = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO orders (id, tenant_id, user_id, order_number, status, total_amount, currency,
                               items, shipping_address, billing_address, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                      total_amount, currency, items, shipping_address, billing_address, notes,
                      created_at, updated_at, deleted_at
            "#,
            order.id,
            order.tenant_id,
            order.user_id,
            order.order_number,
            order.status as OrderStatus,
            order.total_amount,
            order.currency,
            order.items,
            order.shipping_address,
            order.billing_address,
            order.notes,
            order.created_at,
            order.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_order)
    }

    async fn update(&self, id: &Uuid, order: &Order) -> AppResult<Order> {
        let updated_order = sqlx::query_as!(
            Order,
            r#"
            UPDATE orders 
            SET status = $2, total_amount = $3, currency = $4, items = $5,
                shipping_address = $6, billing_address = $7, notes = $8, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, tenant_id, user_id, order_number, status as "status: OrderStatus",
                      total_amount, currency, items, shipping_address, billing_address, notes,
                      created_at, updated_at, deleted_at
            "#,
            id,
            order.status as OrderStatus,
            order.total_amount,
            order.currency,
            order.items,
            order.shipping_address,
            order.billing_address,
            order.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_order)
    }

    async fn delete(&self, id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE orders SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM orders WHERE id = $1 AND deleted_at IS NULL) as exists",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists.unwrap_or(false))
    }

    async fn count(&self) -> AppResult<u64> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM orders WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0) as u64)
    }
}

/// Job repository implementation
pub struct JobRepository {
    pool: PgPool,
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find pending jobs
    pub async fn find_pending(&self, limit: i64) -> AppResult<Vec<Job>> {
        let jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT id, tenant_id, job_type, status as "status: JobStatus", payload, result, error,
                   retry_count, max_retries, scheduled_at, started_at, completed_at, created_at, updated_at
            FROM jobs 
            WHERE status = 'pending' AND scheduled_at <= NOW()
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    /// Update job status
    pub async fn update_status(&self, id: &Uuid, status: JobStatus) -> AppResult<()> {
        sqlx::query!(
            "UPDATE jobs SET status = $2, updated_at = NOW() WHERE id = $1",
            id,
            status as JobStatus
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark job as started
    pub async fn mark_started(&self, id: &Uuid) -> AppResult<()> {
        sqlx::query!(
            "UPDATE jobs SET status = 'running', started_at = NOW(), updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark job as completed
    pub async fn mark_completed(&self, id: &Uuid, result: Option<serde_json::Value>) -> AppResult<()> {
        sqlx::query!(
            "UPDATE jobs SET status = 'completed', result = $2, completed_at = NOW(), updated_at = NOW() WHERE id = $1",
            id,
            result
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark job as failed
    pub async fn mark_failed(&self, id: &Uuid, error: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = 'failed', error = $2, retry_count = retry_count + 1, 
                completed_at = NOW(), updated_at = NOW() 
            WHERE id = $1
            "#,
            id,
            error
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Repository<Job, Uuid> for JobRepository {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Job>> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT id, tenant_id, job_type, status as "status: JobStatus", payload, result, error,
                   retry_count, max_retries, scheduled_at, started_at, completed_at, created_at, updated_at
            FROM jobs 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(job)
    }

    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<Job>> {
        let limit = params.limit.unwrap_or(20) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT id, tenant_id, job_type, status as "status: JobStatus", payload, result, error,
                   retry_count, max_retries, scheduled_at, started_at, completed_at, created_at, updated_at
            FROM jobs 
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!("SELECT COUNT(*) as count FROM jobs")
            .fetch_one(&self.pool)
            .await?
            .count
            .unwrap_or(0) as u64;

        Ok(PaginatedResponse {
            data: jobs,
            pagination: shared::PaginationInfo {
                total: Some(total),
                limit: limit as u32,
                offset: offset as u32,
                has_next: (offset + limit) < total as i64,
                has_prev: offset > 0,
                next_cursor: None,
                prev_cursor: None,
            },
        })
    }

    async fn create(&self, job: &Job) -> AppResult<Job> {
        let created_job = sqlx::query_as!(
            Job,
            r#"
            INSERT INTO jobs (id, tenant_id, job_type, status, payload, retry_count, max_retries,
                             scheduled_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tenant_id, job_type, status as "status: JobStatus", payload, result, error,
                      retry_count, max_retries, scheduled_at, started_at, completed_at, created_at, updated_at
            "#,
            job.id,
            job.tenant_id,
            job.job_type,
            job.status as JobStatus,
            job.payload,
            job.retry_count,
            job.max_retries,
            job.scheduled_at,
            job.created_at,
            job.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_job)
    }

    async fn update(&self, id: &Uuid, job: &Job) -> AppResult<Job> {
        let updated_job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs 
            SET status = $2, payload = $3, result = $4, error = $5, retry_count = $6,
                max_retries = $7, scheduled_at = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, tenant_id, job_type, status as "status: JobStatus", payload, result, error,
                      retry_count, max_retries, scheduled_at, started_at, completed_at, created_at, updated_at
            "#,
            id,
            job.status as JobStatus,
            job.payload,
            job.result,
            job.error,
            job.retry_count,
            job.max_retries,
            job.scheduled_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_job)
    }

    async fn delete(&self, id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM jobs WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM jobs WHERE id = $1) as exists",
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists.unwrap_or(false))
    }

    async fn count(&self) -> AppResult<u64> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM jobs")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0) as u64)
    }
}