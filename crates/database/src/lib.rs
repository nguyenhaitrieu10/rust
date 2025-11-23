//! Database layer with SQLx integration and migration support

pub mod connection;
pub mod migrations;
pub mod models;
pub mod repositories;

// Re-export commonly used items
pub use connection::*;
pub use migrations::*;
pub use models::*;
pub use repositories::*;

// Re-export SQLx types for convenience
pub use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Row, Transaction, Postgres,
};