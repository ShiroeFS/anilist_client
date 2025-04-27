use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub avatar: Option<Avatar>,
    pub banner_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCoverImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: i32,
    pub title: MediaTitle,
    pub description: Option<String>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub genres: Option<Vec<String>>,
    pub average_score: Option<f64>,
    pub cover_image: Option<MediaCoverImage>,
    pub banner_image: Option<String>,
    pub status: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListEntry {
    pub id: i32,
    pub media_id: i32,
    pub status: String,
    pub score: Option<f64>,
    pub progress: Option<i32>,
    pub updated_at: i64, // Unix timestamp
    pub media: Option<Media>,
}
