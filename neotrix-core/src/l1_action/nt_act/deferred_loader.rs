//! # DeferredLoader — Lazy Resource Loading
//!
//! Lazy loading of resources and capabilities with prefetch and eviction support.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeta {
    pub resource_id: String,
    pub size_bytes: usize,
    pub priority: u8,
    pub loaded: bool,
    pub last_accessed: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DeferredError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Resource already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("Loading failed: {0}")]
    LoadFailed(String),
    #[error("Resource evicted: {0}")]
    Evicted(String),
}

pub struct DeferredLoader {
    resources: Arc<Mutex<HashMap<String, ResourceMeta>>>,
    loaded_keys: Arc<Mutex<HashSet<String>>>,
    capacity: usize,
    current_size: Arc<Mutex<usize>>,
}

impl DeferredLoader {
    pub fn new(capacity: usize) -> Self {
        Self {
            resources: Arc::new(Mutex::new(HashMap::new())),
            loaded_keys: Arc::new(Mutex::new(HashSet::new())),
            capacity,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    pub fn load<F, T>(&self, key: &str, loader: F) -> Result<T, DeferredError>
    where
        F: FnOnce() -> Result<T, DeferredError>,
        T: 'static,
    {
        {
            let loaded = self.loaded_keys.lock().unwrap();
            if loaded.contains(key) {
                return self.get_cached::<T>(key);
            }
        }
        let data = loader()?;
        let size_bytes = std::mem::size_of::<T>();
        let meta = ResourceMeta {
            resource_id: key.to_string(),
            size_bytes,
            priority: 1,
            loaded: true,
            last_accessed: get_timestamp(),
        };
        {
            let mut resources = self.resources.lock().unwrap();
            let mut loaded = self.loaded_keys.lock().unwrap();
            let mut current_size = self.current_size.lock().unwrap();
            if *current_size + size_bytes > self.capacity {
                self.evict_low_priority(&mut resources, &mut loaded, &mut current_size);
            }
            let size_bytes = meta.size_bytes;
            resources.insert(key.to_string(), meta);
            loaded.insert(key.to_string());
            *current_size += size_bytes;
        }
        self.cache_data(key, data)
    }

    pub fn prefetch<F, T>(&self, keys: &[String], loader: F) -> usize
    where
        F: Fn(&str) -> Result<T, DeferredError>,
        T: 'static,
    {
        let mut count = 0;
        for key in keys {
            if self.is_loaded(key) { continue; }
            if let Ok(data) = loader(key) {
                let size_bytes = std::mem::size_of::<T>();
                let meta = ResourceMeta {
                    resource_id: key.clone(),
                    size_bytes,
                    priority: 2,
                    loaded: true,
                    last_accessed: get_timestamp(),
                };
                let mut resources = self.resources.lock().unwrap();
                let mut loaded = self.loaded_keys.lock().unwrap();
                let mut current_size = self.current_size.lock().unwrap();
                resources.insert(key.clone(), meta.clone());
                loaded.insert(key.clone());
                *current_size += size_bytes;
                self.cache_data(key, data).ok();
                count += 1;
            }
        }
        count
    }

    pub fn evict(&self, key: &str) -> bool {
        let mut resources = self.resources.lock().unwrap();
        let mut loaded = self.loaded_keys.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        if let Some(meta) = resources.remove(key) {
            loaded.remove(key);
            *current_size = current_size.saturating_sub(meta.size_bytes);
            self.remove_cached(key);
            true
        } else {
            false
        }
    }

    pub fn is_loaded(&self, key: &str) -> bool {
        let loaded = self.loaded_keys.lock().unwrap();
        loaded.contains(key)
    }

    fn get_cached<T>(&self, key: &str) -> Result<T, DeferredError> {
        let resources = self.resources.lock().unwrap();
        if let Some(meta) = resources.get(key) {
            if meta.loaded {
                return self.retrieve_cached::<T>(key);
            }
        }
        Err(DeferredError::NotFound(key.to_string()))
    }

    fn cache_data<T>(&self, key: &str, data: T) -> Result<T, DeferredError> {
        let mut resources = self.resources.lock().unwrap();
        if let Some(meta) = resources.get_mut(key) {
            meta.loaded = true;
            meta.last_accessed = get_timestamp();
        }
        Ok(data)
    }

    fn remove_cached(&self, key: &str) {
        let _ = self.resources.lock().unwrap().remove(key);
    }

    fn retrieve_cached<T>(&self, _key: &str) -> Result<T, DeferredError> {
        Err(DeferredError::NotFound("cached data".to_string()))
    }

    fn evict_low_priority(&self, resources: &mut HashMap<String, ResourceMeta>, loaded: &mut HashSet<String>, current_size: &mut usize) {
        let low_priority_keys: Vec<String> = resources.iter().filter(|(_, meta)| meta.priority <= 1).map(|(k, _)| k.clone()).collect();
        for key in low_priority_keys {
            if let Some(meta) = resources.remove(&key) {
                loaded.remove(&key);
                *current_size = current_size.saturating_sub(meta.size_bytes);
            }
            if *current_size <= self.capacity / 2 { break; }
        }
    }

    pub fn current_usage(&self) -> usize { *self.current_size.lock().unwrap() }
    pub fn loaded_count(&self) -> usize { self.loaded_keys.lock().unwrap().len() }
}

impl Default for DeferredLoader {
    fn default() -> Self { Self::new(1024 * 1024 * 100) }
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_load_and_is_loaded() { let loader = DeferredLoader::new(1024); let data = loader.load("test", || Ok(vec![1u8;10])); assert!(data.is_ok()); assert!(loader.is_loaded("test")); }
    #[test] fn test_evict() { let loader = DeferredLoader::new(1024); loader.load("test", || Ok(vec![1u8;10])).ok(); assert!(loader.is_loaded("test")); assert!(loader.evict("test")); assert!(!loader.is_loaded("test")); }
    #[test] fn test_prefetch() { let loader = DeferredLoader::new(10240); let keys = vec!["a".to_string(), "b".to_string()]; let count = loader.prefetch(&keys, |k| Ok(vec![k.len() as u8])); assert_eq!(count, 2); }
}
