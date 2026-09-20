#![forbid(unsafe_code)]

//! Sensory Buffer — raw observations with TTL-based expiry.
//!
//! Tier 1 of the five-tier cascade. Holds unprocessed observations
//! that haven't yet been attended to by the working memory gate.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A single sensory observation with a creation timestamp.
#[derive(Debug, Clone)]
pub struct Observation {
    pub content: String,
    pub created_at: Instant,
    pub source: String,
}

/// SensoryBuffer: raw observations, TTL 60s, max 100 entries.
#[derive(Debug)]
pub struct SensoryBuffer {
    observations: VecDeque<Observation>,
    ttl: Duration,
    max_capacity: usize,
}

impl Default for SensoryBuffer {
    fn default() -> Self {
        Self {
            observations: VecDeque::new(),
            ttl: Duration::from_secs(60),
            max_capacity: 100,
        }
    }
}

impl SensoryBuffer {
    pub fn new(ttl: Duration, max_capacity: usize) -> Self {
        Self {
            observations: VecDeque::new(),
            ttl,
            max_capacity,
        }
    }

    /// Ingest a new observation. Drops oldest if at capacity.
    pub fn observe(&mut self, content: String, source: String) {
        self.evict_expired();
        if self.observations.len() >= self.max_capacity {
            self.observations.pop_front();
        }
        self.observations.push_back(Observation {
            content,
            created_at: Instant::now(),
            source,
        });
    }

    /// Return all non-expired observations, draining the buffer.
    pub fn drain_attended(&mut self) -> Vec<Observation> {
        self.evict_expired();
        self.observations.drain(..).collect()
    }

    /// Peek at the most recent observation without removing it.
    pub fn peek_latest(&self) -> Option<&Observation> {
        self.observations.back()
    }

    /// Current count of live observations.
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        while let Some(front) = self.observations.front() {
            if now.duration_since(front.created_at) > self.ttl {
                self.observations.pop_front();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_and_len() {
        let mut buf = SensoryBuffer::default();
        buf.observe("hello".into(), "test".into());
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn capacity_eviction() {
        let mut buf = SensoryBuffer::new(Duration::from_secs(60), 2);
        buf.observe("a".into(), "s".into());
        buf.observe("b".into(), "s".into());
        buf.observe("c".into(), "s".into());
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.peek_latest().unwrap().content, "c");
    }

    #[test]
    fn drain_clears_buffer() {
        let mut buf = SensoryBuffer::default();
        buf.observe("x".into(), "s".into());
        let items = buf.drain_attended();
        assert_eq!(items.len(), 1);
        assert!(buf.is_empty());
    }
}
