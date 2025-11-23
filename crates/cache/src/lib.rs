//! Redis cache integration for caching and session management

pub mod client;
pub mod operations;
pub mod serialization;

// Re-export commonly used items
pub use client::*;
pub use operations::*;
pub use serialization::*;

// Re-export Redis types for convenience
pub use redis::{
    AsyncCommands, Client, Connection, ConnectionManager, RedisError, RedisResult,
};