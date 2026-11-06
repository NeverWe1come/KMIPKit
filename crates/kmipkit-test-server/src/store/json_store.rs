//! Thread-safe file-backed JSON storage engine.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::model::StoredObject;

/// Search filter for `Locate` queries over stored objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocateFilter {
    pub object_type: Option<String>,
    pub state: Option<String>,
    pub cryptographic_algorithm: Option<String>,
    pub cryptographic_usage_mask: Option<u32>,
    pub name: Option<String>,
}

/// JSON filesystem store holding objects under `<root_dir>/objects/<uuid>.json`.
pub struct JsonStore {
    objects_dir: PathBuf,
    cache: RwLock<HashMap<String, StoredObject>>,
}

impl JsonStore {
    /// Creates a temporary store in a unique system temporary directory.
    pub fn ephemeral() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "kmipkit-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Self::new(dir, true).expect("ephemeral store creation succeeds")
    }

    /// Initializes the store under `root_dir`.
    ///
    /// If `clean` is true, existing files under `root_dir` are deleted.
    ///
    /// # Errors
    ///
    /// Returns an IO error if directory creation or file loading fails.
    pub fn new(root_dir: impl AsRef<Path>, clean: bool) -> io::Result<Self> {
        let root = root_dir.as_ref();
        let objects_dir = root.join("objects");

        if clean && root.exists() {
            fs::remove_dir_all(root)?;
        }
        fs::create_dir_all(&objects_dir)?;

        let mut cache = HashMap::new();
        for entry in fs::read_dir(&objects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(val) = serde_json::from_str(&contents) {
                        if let Some(obj) = StoredObject::from_json_value(&val) {
                            cache.insert(obj.unique_identifier.clone(), obj);
                        }
                    }
                }
            }
        }

        Ok(Self {
            objects_dir,
            cache: RwLock::new(cache),
        })
    }

    /// Persists an object to disk and updates the in-memory cache.
    ///
    /// # Errors
    ///
    /// Returns an IO error if disk serialization or atomic write fails.
    pub fn save(&self, object: &StoredObject) -> io::Result<()> {
        let json_val = object.to_json_value();
        let json_str = serde_json::to_string_pretty(&json_val)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let file_path = self
            .objects_dir
            .join(format!("{}.json", object.unique_identifier));
        let tmp_path = self
            .objects_dir
            .join(format!("{}.tmp", object.unique_identifier));

        fs::write(&tmp_path, json_str)?;
        fs::rename(&tmp_path, &file_path)?;

        let mut cache = self.cache.write().unwrap();
        cache.insert(object.unique_identifier.clone(), object.clone());
        Ok(())
    }

    /// Fetches an object by unique identifier from cache.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<StoredObject> {
        let cache = self.cache.read().unwrap();
        cache.get(id).cloned()
    }

    /// Deletes an object from disk and in-memory cache.
    ///
    /// # Errors
    ///
    /// Returns an IO error if disk removal fails.
    pub fn delete(&self, id: &str) -> io::Result<bool> {
        let file_path = self.objects_dir.join(format!("{id}.json"));
        let existed_on_disk = if file_path.exists() {
            fs::remove_file(&file_path)?;
            true
        } else {
            false
        };

        let mut cache = self.cache.write().unwrap();
        let existed_in_cache = cache.remove(id).is_some();

        Ok(existed_on_disk || existed_in_cache)
    }

    /// Queries the store for object identifiers matching the given criteria.
    #[must_use]
    pub fn locate(&self, filter: &LocateFilter) -> Vec<String> {
        let cache = self.cache.read().unwrap();
        let mut results = Vec::new();

        for obj in cache.values() {
            if let Some(ref t) = filter.object_type {
                if !obj.object_type.eq_ignore_ascii_case(t) {
                    continue;
                }
            }
            if let Some(ref s) = filter.state {
                if !obj.state.eq_ignore_ascii_case(s) {
                    continue;
                }
            }
            if let Some(ref a) = filter.cryptographic_algorithm {
                if obj
                    .cryptographic_algorithm
                    .as_deref()
                    .is_none_or(|algo| !algo.eq_ignore_ascii_case(a))
                {
                    continue;
                }
            }
            if let Some(mask) = filter.cryptographic_usage_mask {
                if obj
                    .cryptographic_usage_mask
                    .is_none_or(|m| (m & mask) != mask)
                {
                    continue;
                }
            }
            if let Some(ref n) = filter.name {
                let has_matching_name = obj.attributes.get("Name").is_some_and(|list| {
                    list.iter()
                        .any(|attr| attr.value.as_str().is_some_and(|name_str| name_str == n))
                });
                if !has_matching_name {
                    continue;
                }
            }
            results.push(obj.unique_identifier.clone());
        }

        results.sort();
        results
    }
}
