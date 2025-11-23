//! Cache operations and utilities

use async_trait::async_trait;
use redis::AsyncCommands;
use shared::{AppError, AppResult, CacheKey};
use std::collections::HashMap;

use crate::RedisManager;

/// Cache operations trait for specific data types
#[async_trait]
pub trait CacheOperations<T> {
    /// Get item from cache
    async fn get_cached(&self, key: &str) -> AppResult<Option<T>>;
    
    /// Set item in cache
    async fn set_cached(&self, key: &str, value: &T, ttl: Option<u64>) -> AppResult<()>;
    
    /// Delete item from cache
    async fn delete_cached(&self, key: &str) -> AppResult<bool>;
    
    /// Get multiple items from cache
    async fn get_many_cached(&self, keys: &[String]) -> AppResult<HashMap<String, T>>;
}

/// User cache operations
pub struct UserCacheOps {
    redis: RedisManager,
}

impl UserCacheOps {
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
    }

    /// Cache user by ID
    pub async fn cache_user_by_id<T>(&self, user_id: &str, user: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let key = CacheKey::new("user").add("id").add(user_id).build();
        self.redis.set(&key, user, ttl).await
    }

    /// Get cached user by ID
    pub async fn get_user_by_id<T>(&self, user_id: &str) -> AppResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        let key = CacheKey::new("user").add("id").add(user_id).build();
        self.redis.get(&key).await
    }

    /// Cache user by email
    pub async fn cache_user_by_email<T>(&self, email: &str, user: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let key = CacheKey::new("user").add("email").add(email).build();
        self.redis.set(&key, user, ttl).await
    }

    /// Get cached user by email
    pub async fn get_user_by_email<T>(&self, email: &str) -> AppResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        let key = CacheKey::new("user").add("email").add(email).build();
        self.redis.get(&key).await
    }

    /// Invalidate user cache
    pub async fn invalidate_user(&self, user_id: &str, email: Option<&str>) -> AppResult<()> {
        let id_key = CacheKey::new("user").add("id").add(user_id).build();
        self.redis.delete(&id_key).await?;

        if let Some(email) = email {
            let email_key = CacheKey::new("user").add("email").add(email).build();
            self.redis.delete(&email_key).await?;
        }

        Ok(())
    }
}

/// Configuration cache operations
pub struct ConfigCacheOps {
    redis: RedisManager,
}

impl ConfigCacheOps {
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
    }

    /// Cache configuration value
    pub async fn cache_config<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let cache_key = CacheKey::new("config").add(key).build();
        self.redis.set(&cache_key, value, ttl).await
    }

    /// Get cached configuration value
    pub async fn get_config<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync,
    {
        let cache_key = CacheKey::new("config").add(key).build();
        self.redis.get(&cache_key).await
    }

    /// Cache multiple configuration values
    pub async fn cache_configs<T>(&self, configs: &HashMap<String, T>, ttl: Option<u64>) -> AppResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let items: Vec<(String, &T)> = configs
            .iter()
            .map(|(key, value)| {
                let cache_key = CacheKey::new("config").add(key).build();
                (cache_key, value)
            })
            .collect();

        let items_owned: Vec<(String, T)> = items
            .into_iter()
            .map(|(key, value)| (key, value.clone()))
            .collect();

        self.redis.set_many(&items_owned, ttl).await
    }

    /// Invalidate configuration cache
    pub async fn invalidate_config(&self, key: &str) -> AppResult<bool> {
        let cache_key = CacheKey::new("config").add(key).build();
        self.redis.delete(&cache_key).await
    }
}

/// Metrics cache operations
pub struct MetricsCacheOps {
    redis: RedisManager,
}

impl MetricsCacheOps {
    pub fn new(redis: RedisManager) -> Self {
        Self { redis }
    }

    /// Increment counter metric
    pub async fn increment_counter(&self, metric_name: &str, labels: &[(&str, &str)]) -> AppResult<i64> {
        let key = self.build_metric_key(metric_name, labels);
        let mut conn = self.redis.get_connection();
        let result: i64 = conn.incr(&key, 1).await.map_err(|e| AppError::Redis(e))?;
        
        // Set expiration for metrics (24 hours)
        let _: bool = conn.expire(&key, 86400).await.map_err(|e| AppError::Redis(e))?;
        
        Ok(result)
    }

    /// Set gauge metric
    pub async fn set_gauge(&self, metric_name: &str, value: f64, labels: &[(&str, &str)]) -> AppResult<()> {
        let key = self.build_metric_key(metric_name, labels);
        let mut conn = self.redis.get_connection();
        
        conn.set_ex(&key, value, 86400).await.map_err(|e| AppError::Redis(e))?;
        
        Ok(())
    }

    /// Record histogram value
    pub async fn record_histogram(&self, metric_name: &str, value: f64, labels: &[(&str, &str)]) -> AppResult<()> {
        let key = self.build_metric_key(metric_name, labels);
        let mut conn = self.redis.get_connection();
        
        // Use sorted set to store histogram values
        let timestamp = chrono::Utc::now().timestamp_millis() as f64;
        conn.zadd(&key, value, timestamp).await.map_err(|e| AppError::Redis(e))?;
        
        // Keep only last hour of data
        let one_hour_ago = timestamp - (3600.0 * 1000.0);
        conn.zrembyscore(&key, 0.0, one_hour_ago).await.map_err(|e| AppError::Redis(e))?;
        
        // Set expiration
        let _: bool = conn.expire(&key, 3600).await.map_err(|e| AppError::Redis(e))?;
        
        Ok(())
    }

    /// Get metric value
    pub async fn get_metric(&self, metric_name: &str, labels: &[(&str, &str)]) -> AppResult<Option<String>> {
        let key = self.build_metric_key(metric_name, labels);
        self.redis.get(&key).await
    }

    /// Get histogram statistics
    pub async fn get_histogram_stats(&self, metric_name: &str, labels: &[(&str, &str)]) -> AppResult<HistogramStats> {
        let key = self.build_metric_key(metric_name, labels);
        let mut conn = self.redis.get_connection();
        
        let values: Vec<f64> = conn.zrange(&key, 0, -1).await.map_err(|e| AppError::Redis(e))?;
        
        if values.is_empty() {
            return Ok(HistogramStats::default());
        }

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        
        let mut sorted_values = values;
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_values[0];
        let max = sorted_values[count - 1];
        let p50 = sorted_values[count / 2];
        let p95 = sorted_values[(count as f64 * 0.95) as usize];
        let p99 = sorted_values[(count as f64 * 0.99) as usize];

        Ok(HistogramStats {
            count,
            sum,
            avg,
            min,
            max,
            p50,
            p95,
            p99,
        })
    }

    fn build_metric_key(&self, metric_name: &str, labels: &[(&str, &str)]) -> String {
        let mut key = CacheKey::new("metrics").add(metric_name);
        
        for (label_key, label_value) in labels {
            key = key.add(&format!("{}:{}", label_key, label_value));
        }
        
        key.build()
    }
}

/// Histogram statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            avg: 0.0,
            min: 0.0,
            max: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

/// Distributed lock using Redis
pub struct DistributedLock {
    redis: RedisManager,
    key: String,
    value: String,
    ttl: u64,
}

impl DistributedLock {
    /// Create a new distributed lock
    pub fn new(redis: RedisManager, key: String, ttl: u64) -> Self {
        let value = uuid::Uuid::new_v4().to_string();
        Self {
            redis,
            key,
            value,
            ttl,
        }
    }

    /// Acquire the lock
    pub async fn acquire(&self) -> AppResult<bool> {
        let mut conn = self.redis.get_connection();
        
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("PEXPIRE", KEYS[1], ARGV[2])
            else
                return redis.call("SET", KEYS[1], ARGV[1], "PX", ARGV[2], "NX")
            end
        "#;

        let result: Option<String> = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.value)
            .arg(self.ttl * 1000) // Convert to milliseconds
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(result.is_some())
    }

    /// Release the lock
    pub async fn release(&self) -> AppResult<bool> {
        let mut conn = self.redis.get_connection();
        
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.value)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(result == 1)
    }

    /// Extend the lock TTL
    pub async fn extend(&self, additional_ttl: u64) -> AppResult<bool> {
        let mut conn = self.redis.get_connection();
        
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("PEXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.value)
            .arg(additional_ttl * 1000) // Convert to milliseconds
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::Redis(e))?;

        Ok(result == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_stats_default() {
        let stats = HistogramStats::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.sum, 0.0);
        assert_eq!(stats.avg, 0.0);
    }
}