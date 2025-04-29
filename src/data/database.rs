use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Models that will be saved in the database
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

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> SqlResult<Self> {
        let db_path = Self::get_database_path()?;
        let conn = Connection::open(db_path)?;

        // Create tables if they don't exist
        Self::init_db(&conn)?;

        Ok(Self { conn })
    }

    fn get_database_path() -> SqlResult<PathBuf> {
        let proj_dirs = ProjectDirs::from("me", "camniel", "AniListClient").ok_or_else(|| {
            rusqlite::Error::InvalidPath("Could not determine project directory".into())
        })?;

        let data_dir = proj_dirs.data_dir();
        println!("{:?}", data_dir);
        std::fs::create_dir_all(data_dir).map_err(|e| {
            rusqlite::Error::InvalidPath(format!("Could not create data directory: {}", e).into())
        })?;

        Ok(data_dir.join("anilist.db"))
    }

    fn init_db(conn: &Connection) -> SqlResult<()> {
        // Create anime cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_anime(
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                english_title TEXT,
                native_title TEXT,
                description TEXT,
                episodes INTEGER,
                duration INTEGER,
                genres TEXT,
                average_score REAL,
                cover_image TEXT,
                banner_image TEXT,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_list_entries (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                media_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                score REAL,
                progress INTEGER,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(media_id) REFERENCES cached_anime(id),
                UNIQUE(user_id, media_id)
            )",
            [],
        )?;

        // Create user auth table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_auth (
                user_id INTEGER PRIMARY KEY,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                expires_at TEXT,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn cache_anime(&self, anime: &CachedAnime) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cached_anime (
                id, title, english_title, native_title, description,
                episodes, duration, genres, average_score,
                cover_image, banner_image, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                anime.id,
                anime.title,
                anime.english_title,
                anime.native_title,
                anime.description,
                anime.episodes,
                anime.duration,
                anime.genres,
                anime.average_score,
                anime.cover_image,
                anime.banner_image,
                anime.updated_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    // Get a cached anime
    pub fn get_cached_anime(&self, id: i32) -> SqlResult<Option<CachedAnime>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, english_title, native_title, description,
                episodes, duration, genres, average_score,
                cover_image, banner_image, updated_at
                FROM cached_anime
                WHERE id = ?",
        )?;

        let anime_iter = stmt.query_map([id], |row| {
            let updated_at_str: String = row.get(11)?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        11,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);
            Ok(CachedAnime {
                id: row.get(0)?,
                title: row.get(1)?,
                english_title: row.get(2)?,
                native_title: row.get(3)?,
                description: row.get(4)?,
                episodes: row.get(5)?,
                duration: row.get(6)?,
                genres: row.get(7)?,
                average_score: row.get(8)?,
                cover_image: row.get(9)?,
                banner_image: row.get(10)?,
                updated_at,
            })
        })?;

        let anime = anime_iter.filter_map(Result::ok).next();
        Ok(anime)
    }

    // Save or update a list entry
    pub fn save_list_entry(&self, entry: &CachedListEntry) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cached_list_entries (
                id, user_id, media_id, status, score, progress, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                entry.id,
                entry.user_id,
                entry.media_id,
                entry.status,
                entry.score,
                entry.progress,
                entry.updated_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    // Get a user's anime list with cached anime details
    pub fn get_user_anime_list(
        &self,
        user_id: i32,
        status: Option<&str>,
    ) -> SqlResult<Vec<(CachedListEntry, CachedAnime)>> {
        let query = match status {
            Some(_) => {
                "SELECT
                e.id, e.user_id, e.media_id, e.status, e.score, e.progress, e.updated_at,
                a.id, a.title, a.english_title, a.native_title, a.description,
                a.episodes, a.duration, a.genres, a.average_score,
                a.cover_image, a.banner_image, a.updated_at
                FROM cached_list_entries e
                JOIN cached_anime a ON e.media_id = a.id
                WHERE e.user_id = ? AND e.status = ?
                ORDER BY e.updated_at DESC"
            }
            None => {
                "SELECT
                e.id, e.user_id, e.media_id, e.status, e.score, e.progress, e.updated_at,
                a.id, a.title, a.english_title, a.native_title, a.description,
                a.episodes, a.duration, a.genres, a.average_score,
                a.cover_image, a.banner_image, a.updated_at
                FROM cached_list_entries e
                JOIN cached_anime a ON e.media_id = a.id
                WHERE e.user_id = ?
                ORDER BY e.updated_at DESC"
            }
        };

        let mut stmt = self.conn.prepare(query)?;

        let status_string = status.map(|s| s.to_string());
        let params: Vec<&dyn rusqlite::ToSql> = match &status_string {
            Some(status_str) => vec![&user_id, status_str],
            None => vec![&user_id],
        };

        let list_iter = stmt.query_map(params.as_slice(), |row| {
            let entry_updated_at_str: String = row.get(6)?;
            let entry_updated_at = DateTime::parse_from_rfc3339(&entry_updated_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            let anime_updated_at_str: String = row.get(18)?;
            let anime_updated_at = DateTime::parse_from_rfc3339(&anime_updated_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        18,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);
            let entry = CachedListEntry {
                id: row.get(0)?,
                user_id: row.get(1)?,
                media_id: row.get(2)?,
                status: row.get(3)?,
                score: row.get(4)?,
                progress: row.get(5)?,
                updated_at: entry_updated_at,
            };

            let anime = CachedAnime {
                id: row.get(7)?,
                title: row.get(8)?,
                english_title: row.get(9)?,
                native_title: row.get(10)?,
                description: row.get(11)?,
                episodes: row.get(12)?,
                duration: row.get(13)?,
                genres: row.get(14)?,
                average_score: row.get(15)?,
                cover_image: row.get(16)?,
                banner_image: row.get(17)?,
                updated_at: anime_updated_at,
            };

            Ok((entry, anime))
        })?;

        let results = list_iter.filter_map(Result::ok).collect();
        Ok(results)
    }

    // Save user authentication details
    pub fn save_auth(
        &self,
        user_id: i32,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_auth (
                user_id, access_token, refresh_token, expires_at, updated_at
                ) VALUES (?, ?, ?, ?, ?)",
            params![
                user_id,
                access_token,
                refresh_token,
                expires_at.map(|dt| dt.to_rfc3339()),
                Utc::now().to_rfc3339()
            ],
        )?;

        Ok(())
    }

    // Get stored authentication details
    pub fn get_auth(
        &self,
    ) -> SqlResult<Option<(i32, String, Option<String>, Option<DateTime<Utc>>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, access_token, refresh_token, expires_at
                FROM user_auth
                ORDER BY updated_at DESC
                LIMIT 1",
        )?;

        let auth_iter = stmt.query_map([], |row| {
            let expires_at_str: Option<String> = row.get(3)?;
            let expires_at = expires_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            });

            Ok((
                row.get(0)?, // user_id
                row.get(1)?, // access_token
                row.get(2)?, // refresh token
                expires_at,  // expires_at parsed as DateTime<Utc>
            ))
        })?;

        let auth = auth_iter.filter_map(Result::ok).next();
        Ok(auth)
    }

    // Clear all cached data
    pub fn clear_cache(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM cached_list_entries", [])?;
        self.conn.execute("DELETE FROM cached_anime", [])?;
        Ok(())
    }

    pub fn clear_auth(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM user_auth", [])?;
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        // Create a new database connection
        Self::new().expect("Failed to clone database connection")
    }
}
