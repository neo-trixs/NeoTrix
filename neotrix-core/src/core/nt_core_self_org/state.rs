use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedState {
    store: HashMap<String, serde_json::Value>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.store.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.store.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.store.keys()
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadEndRecord {
    pub task_description: String,
    pub attempted_solutions: Vec<String>,
    pub failure_reason: String,
    pub timestamp: i64,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadEndRegistry {
    dead_ends: Vec<DeadEndRecord>,
}

impl DeadEndRegistry {
    pub fn new() -> Self {
        Self {
            dead_ends: Vec::new(),
        }
    }

    pub fn record(&mut self, record: DeadEndRecord) {
        self.dead_ends.push(record);
    }

    pub fn has_attempted(&self, task_description: &str) -> bool {
        self.dead_ends
            .iter()
            .any(|r| r.task_description == task_description)
    }

    pub fn all(&self) -> &[DeadEndRecord] {
        &self.dead_ends
    }
}

impl Default for DeadEndRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_state() {
        let mut s = SharedState::new();
        s.set("key".into(), serde_json::json!("value"));
        assert!(s.contains("key"));
        assert_eq!(s.get("key"), Some(&serde_json::json!("value")));
        let removed = s.remove("key");
        assert!(removed.is_some());
        assert!(!s.contains("key"));
    }

    #[test]
    fn test_dead_end_registry() {
        let mut reg = DeadEndRegistry::new();
        let record = DeadEndRecord {
            task_description: "solve x".into(),
            attempted_solutions: vec!["try a".into(), "try b".into()],
            failure_reason: "not enough data".into(),
            timestamp: 1000,
            agent_id: "agent1".into(),
        };
        reg.record(record);
        assert!(reg.has_attempted("solve x"));
        assert!(!reg.has_attempted("solve y"));
        assert_eq!(reg.all().len(), 1);
    }
}
