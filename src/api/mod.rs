pub mod auth;
pub mod client;
pub mod models;

// Re-export commonly used types
pub use auth::AuthManager;
pub use client::AniListClient;
pub use models::{Avatar, Media, MediaCoverImage, MediaListEntry, MediaTitle, User};
