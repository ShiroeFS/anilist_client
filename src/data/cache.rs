use crate::api::models::{Media, MediaListEntry};
use chrono::{DateTime, Utc};
use log::{debug, trace};
use std::collections::HashMap;

pub struct MediaCache {
    media: HashMap<i32, (Media, DateTime<Utc>)>,
    max_age_seconds: i64,
}

impl MediaCache {
    pub fn new(max_age_seconds: i64) -> Self {
        debug!(
            "Creating new MediaCache with max age: {} seconds",
            max_age_seconds
        );
        Self {
            media: HashMap::new(),
            max_age_seconds,
        }
    }

    pub fn add(&mut self, media: Media) {
        debug!(
            "Caching media: {} (ID: {})",
            media
                .title
                .romaji
                .as_ref()
                .unwrap_or(&String::from("Unknown")),
            media.id
        );
        self.media.insert(media.id, (media, Utc::now()));
    }

    pub fn get(&self, id: i32) -> Option<&Media> {
        trace!("Checking cache for media ID: {}", id);
        self.media.get(&id).and_then(|(media, timestamp)| {
            let age = Utc::now().signed_duration_since(*timestamp).num_seconds();
            if age <= self.max_age_seconds {
                trace!("Cache hit for media ID: {} (age: {} seconds)", id, age);
                Some(media)
            } else {
                debug!(
                    "Cache expired for media ID: {} (age: {} seconds > max: {})",
                    id, age, self.max_age_seconds
                );
                None
            }
        })
    }

    pub fn invalidate(&mut self, id: i32) {
        debug!("Invalidating cache for media ID: {}", id);
        self.media.remove(&id);
    }

    pub fn clear(&mut self) {
        debug!("Clearing entire media cache ({} items)", self.media.len());
        self.media.clear();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.media.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.media.is_empty()
    }
}

pub struct ListCache {
    entries: HashMap<(i32, String), (Vec<MediaListEntry>, DateTime<Utc>)>,
    max_age_seconds: i64,
}

impl ListCache {
    pub fn new(max_age_seconds: i64) -> Self {
        debug!(
            "Creating new ListCache with max age: {} seconds",
            max_age_seconds
        );
        Self {
            entries: HashMap::new(),
            max_age_seconds,
        }
    }

    pub fn add(&mut self, user_id: i32, status: String, entries: Vec<MediaListEntry>) {
        debug!(
            "Caching list for user ID: {}, status: {}, entries: {}",
            user_id,
            status,
            entries.len()
        );
        self.entries
            .insert((user_id, status), (entries, Utc::now()));
    }

    pub fn get(&self, user_id: i32, status: &str) -> Option<&Vec<MediaListEntry>> {
        trace!(
            "Checking cache for user ID: {}, status: {}",
            user_id,
            status
        );
        self.entries
            .get(&(user_id, status.to_string()))
            .and_then(|(entries, timestamp)| {
                let age = Utc::now().signed_duration_since(*timestamp).num_seconds();
                if age <= self.max_age_seconds {
                    trace!(
                        "Cache hit for user ID: {}, status: {} (age: {} seconds)",
                        user_id,
                        status,
                        age
                    );
                    Some(entries)
                } else {
                    debug!(
                        "Cache expired for user ID: {}, status: {} (age: {} seconds > max: {})",
                        user_id, status, age, self.max_age_seconds
                    );
                    None
                }
            })
    }

    pub fn invalidate(&mut self, user_id: i32, status: &str) {
        debug!(
            "Invalidating cache for user ID: {}, status: {}",
            user_id, status
        );
        self.entries.remove(&(user_id, status.to_string()));
    }

    pub fn invalidate_all_for_user(&mut self, user_id: i32) {
        debug!("Invalidating all cache entries for user ID: {}", user_id);
        self.entries.retain(|(id, _), _| *id != user_id);
    }

    pub fn clear(&mut self) {
        debug!("Clearing entire list cache ({} items)", self.entries.len());
        self.entries.clear();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{Media, MediaCoverImage, MediaTitle};
    use chrono::Duration;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_media_cache() {
        let mut cache = MediaCache::new(1); // 1 second expiry

        // Create test media
        let media = Media {
            id: 123,
            title: MediaTitle {
                romaji: Some("Test Anime".to_string()),
                english: Some("Test Anime EN".to_string()),
                native: None,
            },
            description: Some("Test description".to_string()),
            episodes: Some(12),
            duration: Some(24),
            genres: Some(vec!["Action".to_string(), "Adventure".to_string()]),
            average_score: Some(8.5),
            cover_image: Some(MediaCoverImage {
                large: Some("https://example.com/large.jpg".to_string()),
                medium: Some("https://example.com/medium.jpg".to_string()),
            }),
            banner_image: None,
            status: Some("RELEASING".to_string()),
            format: Some("TV".to_string()),
        };

        // Add to cache
        cache.add(media.clone());

        // Should be in cache
        let cached = cache.get(123);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, 123);

        // Check other methods
        cache.invalidate(123);
        assert!(cache.get(123).is_none());

        // Add back and test expiration
        cache.add(media);
        assert!(cache.get(123).is_some());

        // Wait for cache to expire
        sleep(StdDuration::from_secs(2));
        assert!(cache.get(123).is_none());

        // Test clear
        let mut cache = MediaCache::new(60);
        cache.add(Media {
            id: 123,
            title: MediaTitle {
                romaji: Some("Test".to_string()),
                english: None,
                native: None,
            },
            description: None,
            episodes: None,
            duration: None,
            genres: None,
            average_score: None,
            cover_image: None,
            banner_image: None,
            status: None,
            format: None,
        });

        assert!(cache.get(123).is_some());
        cache.clear();
        assert!(cache.get(123).is_none());
    }

    #[test]
    fn test_list_cache() {
        let mut cache = ListCache::new(1); // 1 second expiry

        // Create test entry
        let entry = MediaListEntry {
            id: 456,
            media_id: 123,
            status: "CURRENT".to_string(),
            score: Some(8.0),
            progress: Some(5),
            updated_at: chrono::Utc::now().timestamp(),
            media: None,
        };

        // Add to cache
        cache.add(789, "CURRENT".to_string(), vec![entry.clone()]);

        // Should be in cache
        let cached = cache.get(789, "CURRENT");
        assert!(cached.is_some());
        let entries = cached.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 456);

        // Test invalidation for specific status
        cache.invalidate(789, "CURRENT");
        assert!(cache.get(789, "CURRENT").is_none());

        // Test invalidation for all user entries
        cache.add(789, "CURRENT".to_string(), vec![entry.clone()]);
        cache.add(789, "PLANNING".to_string(), vec![entry.clone()]);
        assert!(cache.get(789, "CURRENT").is_some());
        assert!(cache.get(789, "PLANNING").is_some());

        cache.invalidate_all_for_user(789);
        assert!(cache.get(789, "CURRENT").is_none());
        assert!(cache.get(789, "PLANNING").is_none());

        // Test clear
        cache.add(789, "COMPLETED".to_string(), vec![entry]);
        assert!(cache.get(789, "COMPLETED").is_some());
        cache.clear();
        assert!(cache.get(789, "COMPLETED").is_none());
    }
}
