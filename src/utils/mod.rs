pub mod config;
pub mod error;
pub mod icons;
pub mod logging;
pub mod ui_helpers;

// Re-export important types
pub use config::{AuthConfig, Config};
pub use error::AppError;
pub use ui_helpers::ContainerExt;
