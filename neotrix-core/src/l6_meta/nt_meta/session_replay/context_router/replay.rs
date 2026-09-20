#![deny(clippy::unwrap_used)]

use super::context::{ContextSnapshot, ContextState};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// R-P130 compliant session replay logger.
/// Records context snapshots per turn and supports JSON export.
pub struct SessionReplayLogger {
    snapshots: Arc<Mutex<Vec<ContextSnapshot>>>,
}

impl SessionReplayLogger {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Log a context snapshot for the current turn (R-P130).
    pub fn log_turn(&self, snapshot: ContextSnapshot) {
        let mut snaps = self.snapshots.lock().expect("replay lock poisoned");
        snaps.push(snapshot);
    }

    /// Build and log a snapshot from raw state.
    pub fn log_state(&self, state: &ContextState) {
        let snap = ContextSnapshot::capture(state);
        self.log_turn(snap);
    }

    /// Retrieve all recorded snapshots.
    pub fn get_replay(&self) -> Vec<ContextSnapshot> {
        let snaps = self.snapshots.lock().expect("replay lock poisoned");
        snaps.clone()
    }

    /// Export the full replay as JSON (R-P130).
    pub fn export_replay(&self) -> String {
        let snaps = self.get_replay();
        serde_json::to_string_pretty(&snaps).unwrap_or_else(|_| "[]".to_string())
    }

    /// Number of recorded turns.
    pub fn turn_count(&self) -> usize {
        let snaps = self.snapshots.lock().expect("replay lock poisoned");
        snaps.len()
    }

    /// Clear all recorded snapshots.
    pub fn clear(&self) {
        let mut snaps = self.snapshots.lock().expect("replay lock poisoned");
        snaps.clear();
    }
}

impl Default for SessionReplayLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_at(turn: u64, task: &str) -> ContextState {
        ContextState::new().with_task(task).with_turn(turn)
    }

    #[test]
    fn test_log_and_retrieve() {
        let logger = SessionReplayLogger::new();
        logger.log_state(&state_at(0, "init"));
        logger.log_state(&state_at(1, "search"));
        logger.log_state(&state_at(2, "summarize"));

        let replay = logger.get_replay();
        assert_eq!(replay.len(), 3);
        assert_eq!(replay[0].turn, 0);
        assert_eq!(replay[2].state.current_task.as_deref(), Some("summarize"));
    }

    #[test]
    fn test_export_replay_json() {
        let logger = SessionReplayLogger::new();
        logger.log_state(&state_at(0, "task_a"));
        logger.log_state(&state_at(1, "task_b"));

        let json = logger.export_replay();
        let parsed: Vec<ContextSnapshot> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].turn, 0);
        assert_eq!(parsed[1].turn, 1);
    }

    #[test]
    fn test_turn_count() {
        let logger = SessionReplayLogger::new();
        assert_eq!(logger.turn_count(), 0);
        logger.log_state(&state_at(0, "x"));
        assert_eq!(logger.turn_count(), 1);
    }

    #[test]
    fn test_clear() {
        let logger = SessionReplayLogger::new();
        logger.log_state(&state_at(0, "x"));
        logger.clear();
        assert_eq!(logger.turn_count(), 0);
        assert!(logger.get_replay().is_empty());
    }

    #[test]
    fn test_export_empty() {
        let logger = SessionReplayLogger::new();
        let json = logger.export_replay();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_log_turn_directly() {
        let logger = SessionReplayLogger::new();
        let ctx = ContextState::new().with_task("direct_log").with_turn(7);
        let snap = ContextSnapshot::capture(&ctx);
        logger.log_turn(snap);
        let replay = logger.get_replay();
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].turn, 7);
    }

    #[test]
    fn test_clear_then_log() {
        let logger = SessionReplayLogger::new();
        logger.log_state(&state_at(0, "before"));
        logger.clear();
        logger.log_state(&state_at(1, "after"));
        let replay = logger.get_replay();
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].state.current_task.as_deref(), Some("after"));
    }

    #[test]
    fn test_export_json_valid_array() {
        let logger = SessionReplayLogger::new();
        for i in 0..5 {
            logger.log_state(&state_at(i, &format!("task_{}", i)));
        }
        let json = logger.export_replay();
        // Must be valid JSON array
        let parsed: Vec<ContextSnapshot> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 5);
        for (i, snap) in parsed.iter().enumerate() {
            assert_eq!(snap.turn, i as u64);
        }
    }

    #[test]
    fn test_turn_count_multiple_operations() {
        let logger = SessionReplayLogger::new();
        assert_eq!(logger.turn_count(), 0);
        logger.log_state(&state_at(0, "a"));
        logger.log_state(&state_at(1, "b"));
        assert_eq!(logger.turn_count(), 2);
        logger.clear();
        assert_eq!(logger.turn_count(), 0);
        logger.log_state(&state_at(2, "c"));
        assert_eq!(logger.turn_count(), 1);
    }

    #[test]
    fn test_logger_default() {
        let logger = SessionReplayLogger::default();
        assert_eq!(logger.turn_count(), 0);
    }

    #[test]
    fn test_logger_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let logger = Arc::new(SessionReplayLogger::new());
        let mut handles = vec![];

        for i in 0..4 {
            let logger_clone = Arc::clone(&logger);
            handles.push(thread::spawn(move || {
                logger_clone.log_state(&state_at(i, &format!("task_{}", i)));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(logger.turn_count(), 4);
    }
}
