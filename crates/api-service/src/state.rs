//! Application state management

use cache::RedisManager;
use database::DatabaseManager;
use shared::{AppConfig, AppResult};
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Debug, Clone)]
pub struct AppState {
    config: AppConfig,
    database: DatabaseManager,
    cache: RedisManager,
}

impl AppState {
    /// Create new application state
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        // Initialize database connection
        let database = DatabaseManager::new(&config.database).await?;

        // Initialize Redis cache
        let cache = RedisManager::new(&config.redis).await?;

        Ok(Self {
            config,
            database,
            cache,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get database manager
    pub fn database(&self) -> &DatabaseManager {
        &self.database
    }

    /// Get cache manager
    pub fn cache(&self) -> &RedisManager {
        &self.cache
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.config.is_production()
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.config.is_development()
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.config.service_name
    }

    /// Get service version
    pub fn version(&self) -> &str {
        &self.config.version
    }
}

/// Shared application state type
pub type SharedState = Arc<AppState>;

/// Create shared application state
pub fn create_shared_state(state: AppState) -> SharedState {
    Arc::new(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = AppConfig::default();
        
        // This test would require running database and Redis instances
        // In a real test environment, you would use testcontainers
        // let state = AppState::new(config).await;
        // assert!(state.is_ok());
    }
}