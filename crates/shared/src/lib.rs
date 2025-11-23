//! Shared library containing common types, utilities, and traits
//! used across all microservices in the application.

pub mod config;
pub mod constants;
pub mod errors;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use config::*;
pub use constants::*;
pub use errors::*;
pub use traits::*;
pub use types::*;
pub use utils::*;