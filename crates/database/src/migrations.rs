//! Database migration utilities

use shared::{AppError, AppResult};
use sqlx::{migrate::MigrateDatabase, PgPool, Postgres};
use tracing::{info, warn};

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> AppResult<()> {
        info!("Starting database migrations");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Check if database exists
    pub async fn database_exists(database_url: &str) -> AppResult<bool> {
        Ok(Postgres::database_exists(database_url)
            .await
            .map_err(|e| AppError::Database(e))?)
    }

    /// Create database if it doesn't exist
    pub async fn create_database_if_not_exists(database_url: &str) -> AppResult<()> {
        if !Self::database_exists(database_url).await? {
            info!("Database does not exist, creating it");
            Postgres::create_database(database_url)
                .await
                .map_err(|e| AppError::Database(e))?;
            info!("Database created successfully");
        } else {
            info!("Database already exists");
        }
        Ok(())
    }

    /// Drop database (use with caution!)
    pub async fn drop_database(database_url: &str) -> AppResult<()> {
        warn!("Dropping database: {}", database_url);
        Postgres::drop_database(database_url)
            .await
            .map_err(|e| AppError::Database(e))?;
        info!("Database dropped successfully");
        Ok(())
    }

    /// Get migration info
    pub async fn get_migration_info(&self) -> AppResult<Vec<MigrationInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT version, description, installed_on, success
            FROM _sqlx_migrations
            ORDER BY version
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let migrations = rows
            .into_iter()
            .map(|row| MigrationInfo {
                version: row.version,
                description: row.description,
                installed_on: row.installed_on,
                success: row.success,
            })
            .collect();

        Ok(migrations)
    }

    /// Check if migrations are up to date
    pub async fn is_up_to_date(&self) -> AppResult<bool> {
        // This is a simplified check - in a real implementation,
        // you might want to compare against embedded migrations
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations') as exists"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(result.exists.unwrap_or(false))
    }

    /// Validate database schema
    pub async fn validate_schema(&self) -> AppResult<SchemaValidation> {
        let mut validation = SchemaValidation {
            is_valid: true,
            missing_tables: Vec::new(),
            missing_columns: Vec::new(),
            errors: Vec::new(),
        };

        // Check for required tables
        let required_tables = vec![
            "users", "orders", "payments", "jobs", "events", 
            "sessions", "audit_logs", "tenants"
        ];

        for table in required_tables {
            let exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = $1) as exists",
                table
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

            if !exists.exists.unwrap_or(false) {
                validation.is_valid = false;
                validation.missing_tables.push(table.to_string());
            }
        }

        // Check for required columns in users table
        if validation.missing_tables.is_empty() || !validation.missing_tables.contains(&"users".to_string()) {
            let required_columns = vec![
                "id", "tenant_id", "email", "username", "password_hash",
                "created_at", "updated_at", "deleted_at"
            ];

            for column in required_columns {
                let exists = sqlx::query!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM information_schema.columns 
                        WHERE table_name = 'users' AND column_name = $1
                    ) as exists
                    "#,
                    column
                )
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e))?;

                if !exists.exists.unwrap_or(false) {
                    validation.is_valid = false;
                    validation.missing_columns.push(format!("users.{}", column));
                }
            }
        }

        Ok(validation)
    }

    /// Reset database (drop all tables and re-run migrations)
    pub async fn reset(&self) -> AppResult<()> {
        warn!("Resetting database - this will drop all data!");

        // Drop all tables
        let tables = sqlx::query!(
            r#"
            SELECT tablename FROM pg_tables 
            WHERE schemaname = 'public' 
            AND tablename != '_sqlx_migrations'
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        for table in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table.tablename))
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e))?;
        }

        // Drop migration table
        sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        // Re-run migrations
        self.migrate().await?;

        info!("Database reset completed successfully");
        Ok(())
    }

    /// Seed database with initial data
    pub async fn seed(&self) -> AppResult<()> {
        info!("Seeding database with initial data");

        // Create default tenant
        sqlx::query!(
            r#"
            INSERT INTO tenants (id, name, slug, settings, is_active, created_at, updated_at)
            VALUES (gen_random_uuid(), 'Default Tenant', 'default', '{}', true, NOW(), NOW())
            ON CONFLICT (slug) DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        // Add more seed data as needed
        info!("Database seeding completed successfully");
        Ok(())
    }
}

/// Migration information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationInfo {
    pub version: i64,
    pub description: String,
    pub installed_on: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

/// Schema validation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaValidation {
    pub is_valid: bool,
    pub missing_tables: Vec<String>,
    pub missing_columns: Vec<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_manager() {
        // This test would require a running PostgreSQL instance
        // In a real test environment, you would use testcontainers
        
        // let pool = PgPool::connect("postgresql://test:test@localhost:5432/test").await.unwrap();
        // let manager = MigrationManager::new(pool);
        // let result = manager.migrate().await;
        // assert!(result.is_ok());
    }
}