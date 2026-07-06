//! # Shared State & Dead-End Registry
//!
//! Shared mutable state for agent coordination and a dead-end registry
//! to prevent repeated failed solution attempts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shared state store for inter-agent communication
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

/// Record of a dead-end (failed solution path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadEndRecord {
    pub task_description: String,
    pub attempted_solutions: Vec<String>,
    pub failure_reason: String,
    pub timestamp: i64,
    pub agent_id: String,
}

/// Registry of dead-ends to avoid repeating failures
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
