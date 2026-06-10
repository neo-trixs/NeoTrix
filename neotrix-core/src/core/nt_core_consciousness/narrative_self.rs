use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_NARRATIVE_EVENTS: usize = 500;
const NARRATIVE_PATH: &str = ".neotrix/narrative.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub session_id: String,
    pub timestamp: u64,
    pub summary: String,
    pub reward: f64,
    pub duration_ms: u64,
    pub key_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeSelf {
    pub session_id: String,
    pub session_start: u64,
    pub session_end: Option<u64>,
    pub session_count: u64,
    pub personal_narrative: VecDeque<NarrativeEvent>,
    pub current_session_summary: String,
    pub current_session_reward: f64,
    pub current_session_start: u64,
    pub current_session_insights: Vec<String>,
    pub total_reward: f64,
}

impl Default for NarrativeSelf {
    fn default() -> Self {
        Self::new()
    }
}

impl NarrativeSelf {
    pub fn new() -> Self {
        let session_id = format!("ses_{:x}", rand::random::<u64>());
        Self {
            session_id,
            session_start: now_secs(),
            session_end: None,
            session_count: 0,
            personal_narrative: VecDeque::with_capacity(MAX_NARRATIVE_EVENTS),
            current_session_summary: String::new(),
            current_session_reward: 0.0,
            current_session_start: now_secs(),
            current_session_insights: Vec::with_capacity(16),
            total_reward: 0.0,
        }
    }

    pub fn record_iteration(&mut self, summary: &str, reward: f64, insight: Option<String>) {
        self.current_session_summary = summary.to_string();
        self.current_session_reward += reward;
        self.total_reward += reward;
        if let Some(i) = insight {
            if !self.current_session_insights.contains(&i) {
                self.current_session_insights.push(i);
            }
        }
    }

    pub fn end_session(&mut self) -> NarrativeEvent {
        let elapsed_secs = now_secs().saturating_sub(self.current_session_start);
        let event = NarrativeEvent {
            session_id: self.session_id.clone(),
            timestamp: now_secs(),
            summary: self.current_session_summary.clone(),
            reward: self.current_session_reward,
            duration_ms: elapsed_secs * 1000,
            key_insights: self.current_session_insights.clone(),
        };
        self.personal_narrative.push_back(event.clone());
        if self.personal_narrative.len() > MAX_NARRATIVE_EVENTS {
            self.personal_narrative.pop_front();
        }
        self.session_end = Some(now_secs());
        self.session_count += 1;
        event
    }

    pub fn start_new_session(&mut self) {
        let _ = self.end_session();
        self.session_id = format!("ses_{:x}", rand::random::<u64>());
        self.session_start = now_secs();
        self.session_end = None;
        self.current_session_summary = String::new();
        self.current_session_reward = 0.0;
        self.current_session_start = now_secs();
        self.current_session_insights.clear();
    }

    pub fn narrative_summary(&self, max_events: usize) -> String {
        let count = self.personal_narrative.len().min(max_events);
        let recent: Vec<&NarrativeEvent> = self.personal_narrative.iter().rev().take(count).collect();
        let mut out = format!("Sessions: {} | Total reward: {:.3}\n", self.session_count, self.total_reward);
        for (i, e) in recent.iter().enumerate().rev() {
            out.push_str(&format!(
                "  [{}. {}] reward={:.3} duration={}s insights={}\n",
                i + 1,
                e.summary.chars().take(60).collect::<String>(),
                e.reward,
                e.duration_ms / 1000,
                e.key_insights.len(),
            ));
        }
        out
    }

    pub fn save(&self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(home).join(NARRATIVE_PATH);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load() -> Option<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(home).join(NARRATIVE_PATH);
        std::fs::read_to_string(&path).ok().and_then(|s| {
            serde_json::from_str::<NarrativeSelf>(&s).ok()
        })
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_narrative_new_session() {
        let n = NarrativeSelf::new();
        assert!(n.session_id.starts_with("ses_"));
        assert!(n.session_end.is_none());
    }

    #[test]
    fn test_record_iteration_accumulates() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("task A", 0.5, Some("insight 1".into()));
        n.record_iteration("task B", 0.3, Some("insight 2".into()));
        assert!((n.current_session_reward - 0.8).abs() < 0.001);
        assert!((n.total_reward - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_end_session_creates_event() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("test task", 0.7, None);
        let event = n.end_session();
        assert_eq!(event.session_id, n.session_id);
    }

    #[test]
    fn test_start_new_session_rotates() {
        let mut n = NarrativeSelf::new();
        let first_id = n.session_id.clone();
        n.record_iteration("session A", 1.0, None);
        n.start_new_session();
        assert_ne!(n.session_id, first_id);
        assert_eq!(n.session_count, 1);
        assert_eq!(n.personal_narrative.len(), 1);
    }

    #[test]
    fn test_narrative_summary_format() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("important discovery", 0.9, Some("found pattern".into()));
        n.end_session();
        let summary = n.narrative_summary(10);
        assert!(summary.contains("Sessions:"));
        assert!(summary.contains("reward="));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut n = NarrativeSelf::new();
        n.record_iteration("save test", 0.5, None);
        n.end_session();

        n.save();
        let loaded = NarrativeSelf::load();
        assert!(loaded.is_some());
        if let Some(l) = loaded {
            assert_eq!(l.personal_narrative.len(), n.personal_narrative.len());
            assert_eq!(l.session_count, n.session_count);
        }
    }

    #[test]
    fn test_multiple_sessions_tracked() {
        let mut n = NarrativeSelf::new();
        for i in 0..3 {
            n.record_iteration(&format!("session {}", i), i as f64 * 0.5, None);
            n.start_new_session();
        }
        assert_eq!(n.session_count, 3);
        assert_eq!(n.personal_narrative.len(), 3);
    }

    #[test]
    fn test_max_narrative_events() {
        let mut n = NarrativeSelf::new();
        for i in 0..MAX_NARRATIVE_EVENTS + 10 {
            n.record_iteration(&format!("event {}", i), 0.1, None);
            n.start_new_session();
        }
        assert!(n.personal_narrative.len() <= MAX_NARRATIVE_EVENTS);
    }
}
