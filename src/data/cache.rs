use crate::api::models::{Media, MediaListEntry};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct MediaCache {
    media: HashMap<i32, (Media, DateTime<Utc>)>,
    max_age_seconds: i64,
}

impl MediaCache {
    pub fn new(max_age_seconds: i64) -> Self {
        Self {
            media: HashMap::new(),
            max_age_seconds: max_age_seconds,
        }
    }

    pub fn add(&mut self, media: Media) {
        self.media.insert(media.id, (media, Utc::now()));
    }

    pub fn get(&self, id: i32) -> Option<&Media> {
        self.media.get(&id).and_then(|(media, timestamp)| {
            let age = Utc::now().signed_duration_since(*timestamp).num_seconds();
            if age <= self.max_age_seconds {
                Some(media)
            } else {
                None
            }
        })
    }

    pub fn invalidate(&mut self, id: i32) {
        self.media.remove(&id);
    }

    pub fn clear(&mut self) {
        self.media.clear();
    }
}

pub struct ListCache {
    entries: HashMap<(i32, String), (Vec<MediaListEntry>, DateTime<Utc>)>,
    max_age_seconds: i64,
}

impl ListCache {
    pub fn new(max_age_seconds: i64) -> Self {
        Self {
            entries: HashMap::new(),
            max_age_seconds,
        }
    }

    pub fn add(&mut self, user_id: i32, status: String, entries: Vec<MediaListEntry>) {
        self.entries
            .insert((user_id, status), (entries, Utc::now()));
    }

    pub fn get(&self, user_id: i32, status: &str) -> Option<&Vec<MediaListEntry>> {
        self.entries
            .get(&(user_id, status.to_string()))
            .and_then(|(entries, timestamp)| {
                let age = Utc::now().signed_duration_since(*timestamp).num_seconds();
                if age <= self.max_age_seconds {
                    Some(entries)
                } else {
                    None
                }
            })
    }

    pub fn invalidate(&mut self, user_id: i32, status: &str) {
        self.entries.remove(&(user_id, status.to_string()));
    }

    pub fn invalidate_all_for_user(&mut self, user_id: i32) {
        self.entries.retain(|(id, _), _| *id != user_id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
