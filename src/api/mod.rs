pub mod auth;
pub mod client;
pub mod models;

// Re-export commonly used types
pub use auth::AuthManager;
pub use client::{
    AniListClient, AnimeDetails, AnimeSearch, UpdateMediaList, UserAnimeList, UserProfile, Viewer,
    anime_details, anime_search, update_media_list, user_anime_list, user_profile, viewer,
};
pub use models::{Avatar, Media, MediaCoverImage, MediaListEntry, MediaTitle, User};
