use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResourceLock {
    pub resource: String,
    pub holder: String,
}

pub struct ConcurrencyDetector {
    locks: HashMap<String, Vec<ResourceLock>>,
}

impl ConcurrencyDetector {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
        }
    }
    pub fn acquire(&mut self, resource: &str, holder: &str) -> bool {
        let locks = self.locks.entry(resource.to_string()).or_default();
        if locks.iter().any(|l| l.holder == holder) {
            return true;
        }
        locks.push(ResourceLock {
            resource: resource.to_string(),
            holder: holder.to_string(),
        });
        true
    }
    pub fn release(&mut self, resource: &str, holder: &str) -> bool {
        if let Some(locks) = self.locks.get_mut(resource) {
            let len = locks.len();
            locks.retain(|l| l.holder != holder);
            locks.len() < len
        } else {
            false
        }
    }
    pub fn detect_conflicts(&self) -> Vec<(String, String, String)> {
        self.locks
            .iter()
            .filter(|(_, locks)| locks.len() > 1)
            .map(|(res, locks)| {
                (
                    res.clone(),
                    locks[0].holder.clone(),
                    locks[1].holder.clone(),
                )
            })
            .collect()
    }
    pub fn lock_count(&self) -> usize {
        self.locks.values().map(|v| v.len()).sum()
    }
}
impl Default for ConcurrencyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_acquire_release() {
        let mut d = ConcurrencyDetector::new();
        assert!(d.acquire("file1", "agent1"));
        assert_eq!(d.lock_count(), 1);
        assert!(d.release("file1", "agent1"));
        assert_eq!(d.lock_count(), 0);
    }
    #[test]
    fn test_detect_conflict() {
        let mut d = ConcurrencyDetector::new();
        d.acquire("res", "a");
        d.acquire("res", "b");
        assert!(!d.detect_conflicts().is_empty());
    }
    #[test]
    fn test_no_conflict() {
        let mut d = ConcurrencyDetector::new();
        d.acquire("res", "a");
        assert!(d.detect_conflicts().is_empty());
    }
}
