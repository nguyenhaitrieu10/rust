//! Serialization utilities for cache operations

use shared::{AppError, AppResult, Serializer};
use serde::{Deserialize, Serialize};

/// JSON serializer for cache operations
#[derive(Debug, Clone)]
pub struct JsonSerializer;

impl Serializer for JsonSerializer {
    fn serialize<T>(&self, data: &T) -> AppResult<Vec<u8>>
    where
        T: Serialize,
    {
        serde_json::to_vec(data).map_err(|e| AppError::Serialization(e))
    }

    fn deserialize<T>(&self, data: &[u8]) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(data).map_err(|e| AppError::Serialization(e))
    }
}

/// MessagePack serializer for more efficient cache storage
#[derive(Debug, Clone)]
pub struct MessagePackSerializer;

impl MessagePackSerializer {
    /// Serialize data to MessagePack format
    pub fn serialize_msgpack<T>(&self, data: &T) -> AppResult<Vec<u8>>
    where
        T: Serialize,
    {
        rmp_serde::to_vec(data).map_err(|e| AppError::Internal(format!("MessagePack serialization error: {}", e)))
    }

    /// Deserialize data from MessagePack format
    pub fn deserialize_msgpack<T>(&self, data: &[u8]) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        rmp_serde::from_slice(data).map_err(|e| AppError::Internal(format!("MessagePack deserialization error: {}", e)))
    }
}

/// Compression utilities for cache data
pub struct CompressionUtils;

impl CompressionUtils {
    /// Compress data using gzip
    pub fn compress_gzip(data: &[u8]) -> AppResult<Vec<u8>> {
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| AppError::Io(e))?;
        encoder.finish().map_err(|e| AppError::Io(e))
    }

    /// Decompress gzip data
    pub fn decompress_gzip(data: &[u8]) -> AppResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| AppError::Io(e))?;
        Ok(decompressed)
    }

    /// Compress data using LZ4
    pub fn compress_lz4(data: &[u8]) -> AppResult<Vec<u8>> {
        lz4_flex::compress_prepend_size(data)
            .map_err(|e| AppError::Internal(format!("LZ4 compression error: {}", e)))
            .map(|compressed| compressed)
    }

    /// Decompress LZ4 data
    pub fn decompress_lz4(data: &[u8]) -> AppResult<Vec<u8>> {
        lz4_flex::decompress_size_prepended(data)
            .map_err(|e| AppError::Internal(format!("LZ4 decompression error: {}", e)))
    }
}

/// Cache value wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheValue<T> {
    pub data: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub version: u32,
    pub compressed: bool,
    pub serialization_format: SerializationFormat,
}

impl<T> CacheValue<T> {
    /// Create a new cache value
    pub fn new(data: T, ttl: Option<u64>) -> Self {
        let now = chrono::Utc::now();
        let expires_at = ttl.map(|seconds| now + chrono::Duration::seconds(seconds as i64));

        Self {
            data,
            created_at: now,
            expires_at,
            version: 1,
            compressed: false,
            serialization_format: SerializationFormat::Json,
        }
    }

    /// Check if the cache value has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let remaining = expires_at - chrono::Utc::now();
            remaining.num_seconds().max(0)
        })
    }

    /// Mark as compressed
    pub fn with_compression(mut self) -> Self {
        self.compressed = true;
        self
    }

    /// Set serialization format
    pub fn with_format(mut self, format: SerializationFormat) -> Self {
        self.serialization_format = format;
        self
    }
}

/// Serialization format enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    MessagePack,
    Bincode,
}

/// Advanced cache serializer with compression and format options
pub struct AdvancedCacheSerializer {
    compression_threshold: usize,
    default_format: SerializationFormat,
}

impl AdvancedCacheSerializer {
    /// Create a new advanced cache serializer
    pub fn new(compression_threshold: usize, default_format: SerializationFormat) -> Self {
        Self {
            compression_threshold,
            default_format,
        }
    }

    /// Serialize cache value with optional compression
    pub fn serialize_cache_value<T>(&self, value: &CacheValue<T>) -> AppResult<Vec<u8>>
    where
        T: Serialize,
    {
        // First serialize the data based on format
        let serialized = match value.serialization_format {
            SerializationFormat::Json => serde_json::to_vec(value)
                .map_err(|e| AppError::Serialization(e))?,
            SerializationFormat::MessagePack => rmp_serde::to_vec(value)
                .map_err(|e| AppError::Internal(format!("MessagePack error: {}", e)))?,
            SerializationFormat::Bincode => bincode::serialize(value)
                .map_err(|e| AppError::Internal(format!("Bincode error: {}", e)))?,
        };

        // Apply compression if data is large enough
        if serialized.len() > self.compression_threshold {
            CompressionUtils::compress_lz4(&serialized)
        } else {
            Ok(serialized)
        }
    }

    /// Deserialize cache value with automatic decompression
    pub fn deserialize_cache_value<T>(&self, data: &[u8], format: SerializationFormat) -> AppResult<CacheValue<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Try to decompress first (LZ4 will fail gracefully if not compressed)
        let decompressed = CompressionUtils::decompress_lz4(data)
            .unwrap_or_else(|_| data.to_vec());

        // Deserialize based on format
        match format {
            SerializationFormat::Json => serde_json::from_slice(&decompressed)
                .map_err(|e| AppError::Serialization(e)),
            SerializationFormat::MessagePack => rmp_serde::from_slice(&decompressed)
                .map_err(|e| AppError::Internal(format!("MessagePack error: {}", e))),
            SerializationFormat::Bincode => bincode::deserialize(&decompressed)
                .map_err(|e| AppError::Internal(format!("Bincode error: {}", e))),
        }
    }

    /// Auto-detect best serialization format for data
    pub fn detect_best_format<T>(&self, data: &T) -> SerializationFormat
    where
        T: Serialize,
    {
        // Simple heuristic: try different formats and pick the smallest
        let json_size = serde_json::to_vec(data).map(|v| v.len()).unwrap_or(usize::MAX);
        let msgpack_size = rmp_serde::to_vec(data).map(|v| v.len()).unwrap_or(usize::MAX);
        let bincode_size = bincode::serialize(data).map(|v| v.len()).unwrap_or(usize::MAX);

        if bincode_size <= json_size && bincode_size <= msgpack_size {
            SerializationFormat::Bincode
        } else if msgpack_size <= json_size {
            SerializationFormat::MessagePack
        } else {
            SerializationFormat::Json
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub errors: u64,
    pub total_size_bytes: u64,
    pub avg_serialization_time_ms: f64,
    pub avg_deserialization_time_ms: f64,
}

impl CacheStats {
    /// Create new cache stats
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            sets: 0,
            deletes: 0,
            errors: 0,
            total_size_bytes: 0,
            avg_serialization_time_ms: 0.0,
            avg_deserialization_time_ms: 0.0,
        }
    }

    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate miss ratio
    pub fn miss_ratio(&self) -> f64 {
        1.0 - self.hit_ratio()
    }

    /// Record cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record cache set
    pub fn record_set(&mut self, size_bytes: u64) {
        self.sets += 1;
        self.total_size_bytes += size_bytes;
    }

    /// Record cache delete
    pub fn record_delete(&mut self, size_bytes: u64) {
        self.deletes += 1;
        self.total_size_bytes = self.total_size_bytes.saturating_sub(size_bytes);
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_value_creation() {
        let data = "test data".to_string();
        let cache_value = CacheValue::new(data.clone(), Some(3600));
        
        assert_eq!(cache_value.data, data);
        assert!(!cache_value.is_expired());
        assert!(cache_value.remaining_ttl().unwrap() > 0);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.record_set(100);
        
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.sets, 1);
        assert_eq!(stats.total_size_bytes, 100);
        assert_eq!(stats.hit_ratio(), 0.5);
    }

    #[test]
    fn test_json_serializer() {
        let serializer = JsonSerializer;
        let data = vec![1, 2, 3, 4, 5];
        
        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: Vec<i32> = serializer.deserialize(&serialized).unwrap();
        
        assert_eq!(data, deserialized);
    }
}