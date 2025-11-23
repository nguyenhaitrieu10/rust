//! API Service library

pub mod config;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod state;

// Re-export commonly used items
pub use config::*;
pub use handlers::*;
pub use middleware::*;
pub use routes::*;
pub use services::*;
pub use state::*;