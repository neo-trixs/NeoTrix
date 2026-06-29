use std::collections::HashMap;
use std::time::{Duration, Instant};

const HOT_IDLE_THRESHOLD: Duration = Duration::from_secs(60);
const WARM_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PoolTier {
    Hot,
    Warm,
    Cold,
}

pub trait Resource: Clone + Send + 'static {}

#[derive(Debug, Clone)]
pub struct PoolEntry<T: Resource> {
    pub name: String,
    pub resource: T,
    pub tier: PoolTier,
    pub last_access: Instant,
    pub access_count: u64,
}

pub struct ResourcePool<T: Resource> {
    hot: Vec<PoolEntry<T>>,
    warm: HashMap<String, PoolEntry<T>>,
    cold: HashMap<String, PoolEntry<T>>,
    max_hot: usize,
}

impl<T: Resource> ResourcePool<T> {
    pub fn new(max_hot: usize, max_warm: usize) -> Self {
        let _ = max_warm;
        Self {
            hot: Vec::with_capacity(max_hot),
            warm: HashMap::new(),
            cold: HashMap::new(),
            max_hot,
        }
    }

    pub fn register(&mut self, name: &str, resource: T) {
        let entry = PoolEntry {
            name: name.to_string(),
            resource,
            tier: PoolTier::Cold,
            last_access: Instant::now(),
            access_count: 0,
        };
        self.cold.insert(name.to_string(), entry);
    }

    pub fn access(&mut self, name: &str) -> Option<&T> {
        if let Some(idx) = self.hot.iter().position(|e| e.name == name) {
            self.hot[idx].last_access = Instant::now();
            self.hot[idx].access_count += 1;
            return Some(&self.hot[idx].resource);
        }
        if let Some(entry) = self.warm.get_mut(name) {
            entry.last_access = Instant::now();
            entry.access_count += 1;
            let resource = entry.resource.clone();
            let entry_hot = PoolEntry {
                name: name.to_string(),
                resource,
                tier: PoolTier::Hot,
                last_access: Instant::now(),
                access_count: entry.access_count,
            };
            self.promote_to_hot(entry_hot);
            return self
                .hot
                .iter()
                .find(|e| e.name == name)
                .map(|e| &e.resource);
        }
        if let Some(entry) = self.cold.remove(name) {
            let access_count = entry.access_count + 1;
            let resource = entry.resource.clone();
            let entry_warm = PoolEntry {
                name: name.to_string(),
                resource,
                tier: PoolTier::Warm,
                last_access: Instant::now(),
                access_count,
            };
            self.warm.insert(name.to_string(), entry_warm);
            return self.warm.get(name).map(|e| &e.resource);
        }
        None
    }

    pub fn access_mut(&mut self, name: &str) -> Option<&mut T> {
        if let Some(idx) = self.hot.iter().position(|e| e.name == name) {
            self.hot[idx].last_access = Instant::now();
            self.hot[idx].access_count += 1;
            return Some(&mut self.hot[idx].resource);
        }
        if let Some(entry) = self.warm.get_mut(name) {
            entry.last_access = Instant::now();
            entry.access_count += 1;
            let resource_clone = entry.resource.clone();
            let access_count = entry.access_count;
            let entry_hot = PoolEntry {
                name: name.to_string(),
                resource: resource_clone,
                tier: PoolTier::Hot,
                last_access: Instant::now(),
                access_count,
            };
            self.promote_to_hot(entry_hot);
            return self
                .hot
                .iter_mut()
                .find(|e| e.name == name)
                .map(|e| &mut e.resource);
        }
        if let Some(entry) = self.cold.remove(name) {
            let resource = entry.resource.clone();
            let access_count = entry.access_count + 1;
            let entry_warm = PoolEntry {
                name: name.to_string(),
                resource,
                tier: PoolTier::Warm,
                last_access: Instant::now(),
                access_count,
            };
            self.warm.insert(name.to_string(), entry_warm);
            return self.warm.get_mut(name).map(|e| &mut e.resource);
        }
        None
    }

    fn promote_to_hot(&mut self, entry: PoolEntry<T>) {
        self.warm.remove(&entry.name);
        if self.hot.len() >= self.max_hot {
            if let Some(evicted) = self.hot.pop() {
                let demoted = PoolEntry {
                    tier: PoolTier::Warm,
                    ..evicted
                };
                self.warm.insert(demoted.name.clone(), demoted);
            }
        }
        self.hot.push(entry);
    }

    pub fn evict_stale(&mut self) {
        let now = Instant::now();
        self.hot
            .retain(|e| now.duration_since(e.last_access) < HOT_IDLE_THRESHOLD);
        let stale_warm: Vec<String> = self
            .warm
            .iter()
            .filter(|(_, e)| now.duration_since(e.last_access) >= WARM_IDLE_TIMEOUT)
            .map(|(k, _)| k.clone())
            .collect();
        for key in stale_warm {
            if let Some(entry) = self.warm.remove(&key) {
                let demoted = PoolEntry {
                    tier: PoolTier::Cold,
                    ..entry
                };
                self.cold.insert(key, demoted);
            }
        }
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.hot.iter().any(|e| e.name == name)
            || self.warm.contains_key(name)
            || self.cold.contains_key(name)
    }

    pub fn tier_of(&self, name: &str) -> Option<PoolTier> {
        if self.hot.iter().any(|e| e.name == name) {
            return Some(PoolTier::Hot);
        }
        if self.warm.contains_key(name) {
            return Some(PoolTier::Warm);
        }
        if self.cold.contains_key(name) {
            return Some(PoolTier::Cold);
        }
        None
    }

    pub fn hot_count(&self) -> usize {
        self.hot.len()
    }

    pub fn warm_count(&self) -> usize {
        self.warm.len()
    }

    pub fn cold_count(&self) -> usize {
        self.cold.len()
    }

    pub fn total_count(&self) -> usize {
        self.hot.len() + self.warm.len() + self.cold.len()
    }

    pub fn hot_names(&self) -> Vec<String> {
        self.hot.iter().map(|e| e.name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Resource for String {}

    fn pool() -> ResourcePool<String> {
        let mut p = ResourcePool::new(2, 4);
        p.register("alpha", "alpha-resource".to_string());
        p.register("beta", "beta-resource".to_string());
        p.register("gamma", "gamma-resource".to_string());
        p
    }

    #[test]
    fn test_new_pool_starts_cold() {
        let p = pool();
        assert_eq!(p.cold_count(), 3);
        assert_eq!(p.hot_count(), 0);
        assert_eq!(p.warm_count(), 0);
    }

    #[test]
    fn test_first_access_promotes_to_warm() {
        let mut p = pool();
        let r = p.access("alpha");
        assert!(r.is_some());
        assert_eq!(r.unwrap(), "alpha-resource");
        assert_eq!(p.tier_of("alpha"), Some(PoolTier::Warm));
    }

    #[test]
    fn test_second_access_promotes_to_hot() {
        let mut p = pool();
        p.access("alpha");
        p.access("alpha");
        assert_eq!(p.tier_of("alpha"), Some(PoolTier::Hot));
    }

    #[test]
    fn test_hot_eviction_when_full() {
        let mut p = pool();
        p.access("alpha");
        p.access("beta");
        p.access("gamma");
        p.access("alpha");
        p.access("beta");
        p.access("gamma");
        // All three promoted, but max_hot=2 → one must be in warm
        assert_eq!(p.hot_count(), 2);
        assert_eq!(p.warm_count(), 1);
    }

    #[test]
    fn test_is_registered() {
        let p = pool();
        assert!(p.is_registered("alpha"));
        assert!(!p.is_registered("nonexistent"));
    }

    #[test]
    fn test_access_nonexistent_returns_none() {
        let mut p = ResourcePool::<String>::new(2, 4);
        assert!(p.access("void").is_none());
    }

    #[test]
    fn test_total_count() {
        let p = pool();
        assert_eq!(p.total_count(), 3);
    }

    #[test]
    fn test_access_mut_modifies_resource() {
        let mut p = pool();
        if let Some(r) = p.access_mut("alpha") {
            r.push_str("_modified");
        }
        let r = p.access("alpha").unwrap();
        assert_eq!(r, "alpha-resource_modified");
    }

    #[test]
    fn test_hot_names() {
        let mut p = pool();
        p.access("alpha");
        p.access("alpha");
        let names = p.hot_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "alpha");
    }

    #[test]
    fn test_pool_entry_tracking() {
        let mut p = pool();
        p.access("alpha");
        p.access("alpha");
        p.access("alpha");
        if let PoolTier::Hot = p.tier_of("alpha").unwrap() {
            let entry = p.hot.iter().find(|e| e.name == "alpha").unwrap();
            assert!(entry.access_count >= 3);
        }
    }
}
