pub mod config;
pub mod error;

// Re-export important types
pub use config::{AuthConfig, Config};
pub use error::AppError;
