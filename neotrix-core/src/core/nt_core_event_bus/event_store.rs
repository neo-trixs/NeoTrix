//! Event Sourcing Core — 吸收自 arcagent-state
//! Append-only 事件日志作为唯一真相源，fold重建状态

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    pub event_type: String,
    pub payload: String,
    pub timestamp: u64,
    pub causation_id: Option<u64>,
}

pub struct EventStore {
    events: Vec<Event>,
    checkpoints: Vec<(u64, HashMap<String, String>)>, // (event_id, state)
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new(), checkpoints: Vec::new() }
    }

    pub fn append(&mut self, event_type: String, payload: String) -> u64 {
        let id = self.events.len() as u64;
        self.events.push(Event {
            id, event_type, payload,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            causation_id: self.events.last().map(|e| e.id),
        });
        id
    }

    pub fn checkpoint(&mut self) {
        let state = self.fold();
        let event_id = self.events.last().map(|e| e.id).unwrap_or(0);
        self.checkpoints.push((event_id, state));
    }

    pub fn fold(&self) -> HashMap<String, String> {
        let mut state = HashMap::new();
        for event in &self.events {
            state.insert(format!("event_{}", event.id), event.payload.clone());
        }
        state
    }

    pub fn fold_from(&self, from_id: u64) -> HashMap<String, String> {
        let mut state = HashMap::new();
        for event in self.events.iter().filter(|e| e.id >= from_id) {
            state.insert(format!("event_{}", event.id), event.payload.clone());
        }
        state
    }

    pub fn events(&self) -> &[Event] { &self.events }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
}

impl Default for EventStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_basic() {
        let mut store = EventStore::new();
        store.append("fact".into(), "Water is wet".into());
        store.append("decision".into(), "Use Rust".into());

        assert_eq!(store.len(), 2);
        let state = store.fold();
        assert_eq!(state["event_0"], "Water is wet");
        assert_eq!(state["event_1"], "Use Rust");
    }

    #[test]
    fn test_checkpoint_and_fold() {
        let mut store = EventStore::new();
        store.append("a".into(), "1".into());
        store.checkpoint();
        store.append("b".into(), "2".into());

        let state = store.fold_from(1);
        assert_eq!(state.len(), 1);
    }
}
