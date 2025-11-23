//! Redis client management and connection handling

use async_trait::async_trait;
use redis::{AsyncCommands, Client, ConnectionManager};
use shared::{AppError, AppResult, Cache, RedisConfig};
use std::time::Duration;
use tracing::{info, warn};

/// Redis cache manager
#[derive(Debug, Clone)]
pub struct RedisManager {
    connection_manager: ConnectionManager,
    default_ttl: u64,
}

impl RedisManager {
    /// Create a new Redis manager with connection pool
    pub async fn new(config: &RedisConfig) -> AppResult<Self> {
        info!("Initializing Redis connection manager");

        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::Redis(e))?;

        let connection_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Redis(e))?;

        // Test the connection
        let mut conn = connection_manager.clone();
        let _: String = conn.ping().await.map_err(|e| AppError::Redis(e))?;

        info!("Redis connection manager initialized successfully");

        Ok(Self {
            connection_manager,
            default_ttl: config.default_ttl,
        })
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> ConnectionManager {
        self.connection_manager.clone()
    }

    /// Check Redis health
    pub async fn health_check(&self) -> AppResult<RedisHealth> {
        let start = std::time::Instant::now();
        let mut conn = self.connection_manager.clone();

        match conn.ping().await {
            Ok(_) => {
                let response_time = start.elapsed();
                Ok(RedisHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time.as_millis() as u64,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Redis health check failed: {}", e);
                Ok(RedisHealth {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Get Redis info
    pub async fn get_info(&self) -> AppResult<RedisInfo> {
        let mut conn = self.connection_manager.clone();
        let info: String = conn.info().await.map_err(|e| AppError::Redis(e))?;
        
        // Parse basic info from Redis INFO command
        let mut memory_used = 0u64;
        let mut connected_clients = 0u32;
        let mut total_commands_processed = 0u64;
        
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    memory_used = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("connected_clients:") {
                if let Some(value) = line.split(':').nth(1) {
                    connected_clients = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("total_commands_processed:") {
                if let Some(value) = line.split(':').nth(1) {
                    total_commands_processed = value.parse().unwrap_or(0);
                }
            }
        }

        Ok(RedisInfo {
            memory_used,
            connected_clients,
            total_commands_processed,
            raw_info: info,
        })
    }

    /// Flush all data (use with caution!)
    pub async fn flush_all(&self) -> AppResult<()> {
        warn!("Flushing all Redis data");
        let mut conn = self.connection_manager.clone();
        conn.flushall().await.map_err(|e| AppError::Redis(e))?;
        info!("Redis data flushed successfully");
        Ok(())
    }

    /// Flush database (use with caution!)
    pub async fn flush_db(&self) -> AppResult<()> {
        warn!("Flushing current Redis database");
        let mut conn = self.connection_manager.clone();
        conn.flushdb().await.map_err(|e| AppError::Redis(e))?;
        info!("Redis database flushed successfully");
        Ok(())
    }

    /// Get default TTL
    pub fn default_ttl(&self) -> u64 {
        self.default_ttl
    }
}

#[async_trait]
impl Cache for RedisManager {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        let mut conn = self.connection_manager.clone();
        let value: Option<String> = conn.get(key).await.map_err(|e| AppError::Redis(e))?;

        match value {
            Some(json_str) => {
                let deserialized: T = serde_json::from_str(&json_str)
                    .map_err(|e| AppError::Serialization(e))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let mut conn = self.connection_manager.clone();
        let json_str = serde_json::to_string(value)
            .map_err(|e| AppError::Serialization(e))?;

        let ttl_seconds = ttl.unwrap_or(self.default_ttl);
        
        conn.set_ex(key, json_str, ttl_seconds)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.connection_manager.clone();
        let deleted: u32 = conn.del(key).await.map_err(|e| AppError::Redis(e))?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.connection_manager.clone();
        let exists: bool = conn.exists(key).await.map_err(|e| AppError::Redis(e))?;
        Ok(exists)
    }

    async fn expire(&self, key: &str, ttl: u64) -> AppResult<bool> {
        let mut conn = self.connection_manager.clone();
        let result: bool = conn.expire(key, ttl as usize).await.map_err(|e| AppError::Redis(e))?;
        Ok(result)
    }

    async fn get_many<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.connection_manager.clone();
        let values: Vec<Option<String>> = conn.get(keys).await.map_err(|e| AppError::Redis(e))?;

        let mut results = Vec::with_capacity(values.len());
        for value in values {
            match value {
                Some(json_str) => {
                    let deserialized: T = serde_json::from_str(&json_str)
                        .map_err(|e| AppError::Serialization(e))?;
                    results.push(Some(deserialized));
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    async fn set_many<T>(&self, items: &[(String, T)], ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.connection_manager.clone();
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);

        // Use pipeline for better performance
        let mut pipe = redis::pipe();
        for (key, value) in items {
            let json_str = serde_json::to_string(value)
                .map_err(|e| AppError::Serialization(e))?;
            pipe.set_ex(key, json_str, ttl_seconds);
        }

        pipe.query_async(&mut conn)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(())
    }
}

/// Redis health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Redis health information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedisHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

/// Redis information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedisInfo {
    pub memory_used: u64,
    pub connected_clients: u32,
    pub total_commands_processed: u64,
    pub raw_info: String,
}

/// Session manager using Redis
pub struct SessionManager {
    redis: RedisManager,
    session_prefix: String,
    default_session_ttl: u64,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(redis: RedisManager, session_prefix: String, default_session_ttl: u64) -> Self {
        Self {
            redis,
            session_prefix,
            default_session_ttl,
        }
    }

    /// Create a new session
    pub async fn create_session<T>(&self, session_id: &str, data: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let key = format!("{}:{}", self.session_prefix, session_id);
        let ttl = ttl.unwrap_or(self.default_session_ttl);
        self.redis.set(&key, data, Some(ttl)).await
    }

    /// Get session data
    pub async fn get_session<T>(&self, session_id: &str) -> AppResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        let key = format!("{}:{}", self.session_prefix, session_id);
        self.redis.get(&key).await
    }

    /// Update session data
    pub async fn update_session<T>(&self, session_id: &str, data: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let key = format!("{}:{}", self.session_prefix, session_id);
        let ttl = ttl.unwrap_or(self.default_session_ttl);
        self.redis.set(&key, data, Some(ttl)).await
    }

    /// Delete session
    pub async fn delete_session(&self, session_id: &str) -> AppResult<bool> {
        let key = format!("{}:{}", self.session_prefix, session_id);
        self.redis.delete(&key).await
    }

    /// Extend session TTL
    pub async fn extend_session(&self, session_id: &str, ttl: Option<u64>) -> AppResult<bool> {
        let key = format!("{}:{}", self.session_prefix, session_id);
        let ttl = ttl.unwrap_or(self.default_session_ttl);
        self.redis.expire(&key, ttl).await
    }

    /// Check if session exists
    pub async fn session_exists(&self, session_id: &str) -> AppResult<bool> {
        let key = format!("{}:{}", self.session_prefix, session_id);
        self.redis.exists(&key).await
    }
}

/// Rate limiter using Redis
pub struct RateLimiter {
    redis: RedisManager,
    prefix: String,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(redis: RedisManager, prefix: String) -> Self {
        Self { redis, prefix }
    }

    /// Check if request is allowed (sliding window)
    pub async fn is_allowed(&self, key: &str, limit: u32, window_seconds: u64) -> AppResult<bool> {
        let redis_key = format!("{}:{}", self.prefix, key);
        let mut conn = self.redis.get_connection();
        
        let now = chrono::Utc::now().timestamp() as u64;
        let window_start = now - window_seconds;

        // Remove old entries and count current requests
        let script = r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window_start = tonumber(ARGV[2])
            local limit = tonumber(ARGV[3])
            local window_seconds = tonumber(ARGV[4])
            
            -- Remove old entries
            redis.call('ZREMRANGEBYSCORE', key, 0, window_start)
            
            -- Count current requests
            local current = redis.call('ZCARD', key)
            
            if current < limit then
                -- Add current request
                redis.call('ZADD', key, now, now)
                redis.call('EXPIRE', key, window_seconds)
                return 1
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&redis_key)
            .arg(now)
            .arg(window_start)
            .arg(limit)
            .arg(window_seconds)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(result == 1)
    }

    /// Get remaining requests for key
    pub async fn remaining(&self, key: &str, limit: u32, window_seconds: u64) -> AppResult<u32> {
        let redis_key = format!("{}:{}", self.prefix, key);
        let mut conn = self.redis.get_connection();
        
        let now = chrono::Utc::now().timestamp() as u64;
        let window_start = now - window_seconds;

        // Remove old entries and count current requests
        let _: () = conn.zrembyscore(&redis_key, 0, window_start as isize).await.map_err(|e| AppError::Redis(e))?;
        let current: u32 = conn.zcard(&redis_key).await.map_err(|e| AppError::Redis(e))?;

        Ok(limit.saturating_sub(current))
    }

    /// Reset rate limit for key
    pub async fn reset(&self, key: &str) -> AppResult<()> {
        let redis_key = format!("{}:{}", self.prefix, key);
        self.redis.delete(&redis_key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::RedisConfig;

    #[tokio::test]
    async fn test_redis_manager_creation() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connect_timeout: 5,
            response_timeout: 5,
            connection_timeout: 5,
            default_ttl: 3600,
        };

        // This test would require a running Redis instance
        // In a real test environment, you would use testcontainers
        // let manager = RedisManager::new(&config).await;
        // assert!(manager.is_ok());
    }
}