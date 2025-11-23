//! Database connection management

use shared::{AppError, AppResult, DatabaseConfig};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::{info, warn};

/// Database connection pool manager
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with connection pool
    pub async fn new(config: &DatabaseConfig) -> AppResult<Self> {
        info!("Initializing database connection pool");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout)))
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime)))
            .connect(&config.url)
            .await
            .map_err(|e| AppError::Database(e))?;

        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Database connection pool initialized successfully");

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get pool status information
    pub async fn pool_status(&self) -> PoolStatus {
        PoolStatus {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            used: self.pool.size() - self.pool.num_idle(),
            max_size: self.pool.options().get_max_connections(),
        }
    }

    /// Check database health
    pub async fn health_check(&self) -> AppResult<DatabaseHealth> {
        let start = std::time::Instant::now();
        
        match sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => {
                let response_time = start.elapsed();
                Ok(DatabaseHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time.as_millis() as u64,
                    pool_status: self.pool_status().await,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Database health check failed: {}", e);
                Ok(DatabaseHealth {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    pool_status: self.pool_status().await,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Close the connection pool
    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }

    /// Run database migrations
    pub async fn migrate(&self) -> AppResult<()> {
        info!("Running database migrations");
        
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Begin a new transaction
    pub async fn begin_transaction(&self) -> AppResult<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Execute a query with parameters
    pub async fn execute_query<'q>(
        &self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<sqlx::postgres::PgQueryResult> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Fetch one row from a query
    pub async fn fetch_one_query<'q>(
        &self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<sqlx::postgres::PgRow> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Fetch all rows from a query
    pub async fn fetch_all_query<'q>(
        &self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<Vec<sqlx::postgres::PgRow>> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Fetch optional row from a query
    pub async fn fetch_optional_query<'q>(
        &self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<Option<sqlx::postgres::PgRow>> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))
    }
}

/// Pool status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolStatus {
    pub size: u32,
    pub idle: u32,
    pub used: u32,
    pub max_size: u32,
}

/// Database health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Database health information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub pool_status: PoolStatus,
    pub error: Option<String>,
}

/// Database transaction wrapper
pub struct DatabaseTransaction<'a> {
    transaction: sqlx::Transaction<'a, sqlx::Postgres>,
}

impl<'a> DatabaseTransaction<'a> {
    /// Create a new transaction wrapper
    pub fn new(transaction: sqlx::Transaction<'a, sqlx::Postgres>) -> Self {
        Self { transaction }
    }

    /// Commit the transaction
    pub async fn commit(self) -> AppResult<()> {
        self.transaction
            .commit()
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> AppResult<()> {
        self.transaction
            .rollback()
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Execute a query within the transaction
    pub async fn execute_query<'q>(
        &mut self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<sqlx::postgres::PgQueryResult> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .execute(&mut *self.transaction)
            .await
            .map_err(|e| AppError::Database(e))
    }

    /// Fetch one row within the transaction
    pub async fn fetch_one_query<'q>(
        &mut self,
        query: &'q str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)],
    ) -> AppResult<sqlx::postgres::PgRow> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder
            .fetch_one(&mut *self.transaction)
            .await
            .map_err(|e| AppError::Database(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::DatabaseConfig;

    #[tokio::test]
    async fn test_database_manager_creation() {
        let config = DatabaseConfig {
            url: "postgresql://test:test@localhost:5432/test".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            migrate_on_start: false,
        };

        // This test would require a running PostgreSQL instance
        // In a real test environment, you would use testcontainers
        // let manager = DatabaseManager::new(&config).await;
        // assert!(manager.is_ok());
    }
}