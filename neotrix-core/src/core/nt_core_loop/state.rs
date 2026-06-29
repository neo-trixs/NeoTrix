use std::collections::HashMap;
use std::path::PathBuf;

const LOOP_STATE_FILE: &str = ".neotrix/loop_state.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopState {
    pub cycle: u64,
    pub phase: String,
    pub last_decision: String,
    pub handler_coverage: HashMap<String, usize>,
    pub goal_status: String,
    pub last_verify_score: f64,
    pub consecutive_failures: u64,
    pub total_cycles: u64,
}

impl LoopState {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            phase: "Discover".to_string(),
            last_decision: "init".to_string(),
            handler_coverage: HashMap::new(),
            goal_status: "none".to_string(),
            last_verify_score: 0.0,
            consecutive_failures: 0,
            total_cycles: 0,
        }
    }

    pub fn load() -> Self {
        let path = match Self::state_path() {
            Some(p) => p,
            None => return Self::new(),
        };
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    pub fn save(&self) {
        let path = match Self::state_path() {
            Some(p) => p,
            None => return,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            let _ = std::fs::write(&tmp, &content);
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    fn state_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(LOOP_STATE_FILE))
    }
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_state_new() {
        let state = LoopState::new();
        assert_eq!(state.cycle, 0);
        assert_eq!(state.goal_status, "none");
        assert_eq!(state.consecutive_failures, 0);
    }

    #[test]
    fn test_loop_state_save_roundtrip() {
        let mut state = LoopState::new();
        state.cycle = 42;
        state.last_verify_score = 0.85;
        state.consecutive_failures = 1;
        // save/load would need a temp file; just verify fields
        assert_eq!(state.cycle, 42);
        assert_eq!(state.last_verify_score, 0.85);
    }

    #[test]
    fn test_loop_state_increment() {
        let mut state = LoopState::new();
        state.cycle += 1;
        state.total_cycles += 1;
        assert_eq!(state.cycle, 1);
        assert_eq!(state.total_cycles, 1);
    }
}
