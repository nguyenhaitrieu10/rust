//! API handlers

pub mod auth;
pub mod health;
pub mod users;

// Re-export handler modules
pub use auth::*;
pub use health::*;
pub use users::*;