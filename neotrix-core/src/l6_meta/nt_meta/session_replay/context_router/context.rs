#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Live context state for a routing decision.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextState {
    pub current_task: Option<String>,
    pub environment: HashMap<String, String>,
    pub user_preferences: HashMap<String, String>,
    pub active_agents: Vec<String>,
    pub turn_number: u64,
}

impl ContextState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_task(mut self, task: impl Into<String>) -> Self {
        self.current_task = Some(task.into());
        self
    }

    pub fn with_turn(mut self, turn: u64) -> Self {
        self.turn_number = turn;
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    pub fn with_preference(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.user_preferences.insert(key.into(), value.into());
        self
    }

    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.active_agents.push(agent.into());
        self
    }
}

/// Immutable snapshot of context at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub state: ContextState,
    pub timestamp: u128,
    pub turn: u64,
}

impl ContextSnapshot {
    pub fn capture(state: &ContextState) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            state: state.clone(),
            timestamp: now,
            turn: state.turn_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_state_defaults() {
        let ctx = ContextState::new();
        assert!(ctx.current_task.is_none());
        assert!(ctx.environment.is_empty());
        assert!(ctx.user_preferences.is_empty());
        assert!(ctx.active_agents.is_empty());
        assert_eq!(ctx.turn_number, 0);
    }

    #[test]
    fn test_context_state_builder() {
        let ctx = ContextState::new()
            .with_task("summarize")
            .with_turn(3)
            .with_env("os", "macos")
            .with_preference("style", "concise")
            .with_agent("researcher");
        assert_eq!(ctx.current_task.as_deref(), Some("summarize"));
        assert_eq!(ctx.turn_number, 3);
        assert_eq!(ctx.environment.get("os").map(String::as_str), Some("macos"));
        assert_eq!(
            ctx.user_preferences.get("style").map(String::as_str),
            Some("concise")
        );
        assert_eq!(ctx.active_agents, vec!["researcher"]);
    }

    #[test]
    fn test_snapshot_capture() {
        let ctx = ContextState::new().with_task("test").with_turn(1);
        let snap = ContextSnapshot::capture(&ctx);
        assert_eq!(snap.turn, 1);
        assert_eq!(snap.state.current_task.as_deref(), Some("test"));
        assert!(snap.timestamp > 0);
    }

    #[test]
    fn test_multiple_active_agents() {
        let ctx = ContextState::new()
            .with_agent("coder")
            .with_agent("reviewer")
            .with_agent("tester");
        assert_eq!(ctx.active_agents.len(), 3);
        assert_eq!(ctx.active_agents[0], "coder");
        assert_eq!(ctx.active_agents[2], "tester");
    }

    #[test]
    fn test_context_state_clone() {
        let ctx = ContextState::new()
            .with_task("clone_test")
            .with_turn(5)
            .with_env("k", "v");
        let cloned = ctx.clone();
        assert_eq!(cloned.current_task, ctx.current_task);
        assert_eq!(cloned.turn_number, ctx.turn_number);
        assert_eq!(cloned.environment, ctx.environment);
    }

    #[test]
    fn test_context_state_debug_format() {
        let ctx = ContextState::new().with_task("debug");
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("ContextState"));
    }

    #[test]
    fn test_snapshot_serialize_deserialize() {
        let ctx = ContextState::new().with_task("ser").with_turn(2);
        let snap = ContextSnapshot::capture(&ctx);
        let json = serde_json::to_string(&snap).unwrap();
        let back: ContextSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.turn, 2);
        assert_eq!(back.state.current_task.as_deref(), Some("ser"));
    }

    #[test]
    fn test_context_state_builder_chaining_order() {
        let ctx = ContextState::new()
            .with_agent("a1")
            .with_turn(10)
            .with_task("t1")
            .with_env("e1", "v1")
            .with_preference("p1", "pv1");
        assert_eq!(ctx.active_agents, vec!["a1"]);
        assert_eq!(ctx.turn_number, 10);
        assert_eq!(ctx.current_task.as_deref(), Some("t1"));
        assert_eq!(ctx.environment.get("e1").map(String::as_str), Some("v1"));
        assert_eq!(
            ctx.user_preferences.get("p1").map(String::as_str),
            Some("pv1")
        );
    }
}
