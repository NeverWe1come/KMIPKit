//! In-memory ephemeral metadata and tag cache with TTL.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::providers::CloudResourceMeta;

pub struct MetadataCache {
    ttl: Duration,
    entries: RwLock<BTreeMap<String, (CloudResourceMeta, Instant)>>,
}

impl MetadataCache {
    /// Creates a new MetadataCache with the specified TTL.
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: RwLock::new(BTreeMap::new()),
        }
    }

    /// Retrieves cached metadata if present and not expired.
    #[must_use]
    pub fn get(&self, unique_id: &str) -> Option<CloudResourceMeta> {
        let guard = self.entries.read().ok()?;
        if let Some((meta, inserted_at)) = guard.get(unique_id) {
            if inserted_at.elapsed() < self.ttl {
                return Some(meta.clone());
            }
        }
        None
    }

    /// Inserts or updates cached metadata.
    pub fn insert(&self, unique_id: String, meta: CloudResourceMeta) {
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(unique_id, (meta, Instant::now()));
        }
    }

    /// Invalidates cached metadata for a specific unique_id upon mutation.
    pub fn invalidate(&self, unique_id: &str) {
        if let Ok(mut guard) = self.entries.write() {
            guard.remove(unique_id);
        }
    }

    /// Clears the entire cache.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::CloudTagMap;

    #[test]
    fn test_cache_expiration_and_invalidation() {
        let cache = MetadataCache::new(Duration::from_millis(50));
        let meta = CloudResourceMeta {
            unique_id: "key-1".to_string(),
            tags: CloudTagMap::default(),
            native_state_enabled: true,
            created_at_epoch_secs: Some(123456789),
        };

        cache.insert("key-1".to_string(), meta.clone());
        assert_eq!(cache.get("key-1"), Some(meta.clone()));

        // Invalidation removes immediately
        cache.invalidate("key-1");
        assert_eq!(cache.get("key-1"), None);

        // Expiration
        cache.insert("key-1".to_string(), meta);
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cache.get("key-1"), None);
    }
}
