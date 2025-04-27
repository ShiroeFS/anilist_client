use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Local data models that map to database tables

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnime {
    pub id: i32,
    pub title: String,
    pub english_title: Option<String>,
    pub native_title: Option<String>,
    pub description: Option<String>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub genres: String, // comma-separated
    pub average_score: Option<f64>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedListEntry {
    pub id: i32,
    pub user_id: i32,
    pub media_id: i32,
    pub status: String, // CURRENT, PLANNING, COMPLETED, etc.
    pub score: Option<i32>,
    pub progress: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuth {
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
