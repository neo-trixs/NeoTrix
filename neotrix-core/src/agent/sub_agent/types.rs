use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for the sub-agent pool.
#[derive(Debug, Clone)]
pub struct SubAgentConfig {
    /// Maximum number of concurrent sub-agents (default 10, max 20).
    pub max_concurrency: usize,
    /// Maximum context window tokens per sub-agent.
    pub max_context_window: usize,
    /// Idle timeout in seconds before a sub-agent is considered timed out (default 300).
    pub idle_timeout_secs: u64,
}

impl Default for SubAgentConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            max_context_window: 4096,
            idle_timeout_secs: 300,
        }
    }
}

impl SubAgentConfig {
    /// Validate and clamp configuration values.
    pub fn validate(&mut self) {
        self.max_concurrency = self.max_concurrency.clamp(1, 20);
        self.idle_timeout_secs = self.idle_timeout_secs.max(1);
        self.max_context_window = self.max_context_window.max(256);
    }
}

/// Status of a sub-agent.
#[derive(Debug, Clone, PartialEq)]
pub enum SubAgentStatus {
    Running,
    Completed,
    Failed(String),
    TimedOut,
}

/// Structured result produced by a completed sub-agent.
#[derive(Debug, Clone)]
pub struct SubAgentResult {
    /// Summary of the sub-agent's output.
    pub summary: String,
    /// Evidence items collected during execution.
    pub evidence: Vec<String>,
    /// Execution metrics (e.g. tokens per second, confidence scores).
    pub execution_metrics: HashMap<String, f64>,
    /// Total tokens consumed.
    pub total_tokens: u64,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// A handle to a running sub-agent.
#[derive(Debug, Clone)]
pub struct SubAgentHandle {
    /// Unique identifier for this sub-agent.
    pub id: Uuid,
    /// Current status.
    pub status: SubAgentStatus,
    /// Unix timestamp (seconds) when the sub-agent started.
    pub started_at: u64,
    /// Shared slot that will be filled with the result when the sub-agent completes.
    pub result_slot: Arc<RwLock<Option<SubAgentResult>>>,
}

/// Events emitted by the sub-agent pool.
#[derive(Debug, Clone)]
pub enum SubAgentEvent {
    /// A sub-agent completed successfully.
    Done {
        id: Uuid,
        summary: String,
        duration_ms: u64,
        total_tokens: u64,
    },
    /// A sub-agent failed.
    Failed {
        id: Uuid,
        error: String,
    },
    /// A sub-agent timed out.
    TimedOut {
        id: Uuid,
        elapsed_secs: u64,
    },
}

impl SubAgentEvent {
    /// Format this event as a structured `<subagent:done>` style string.
    pub fn to_tag_string(&self) -> String {
        match self {
            SubAgentEvent::Done { id, summary, duration_ms, total_tokens } => {
                format!(
                    "<subagent:done id=\"{}\" duration_ms=\"{}\" tokens=\"{}\">{}",
                    id, duration_ms, total_tokens,
                    summary,
                )
            }
            SubAgentEvent::Failed { id, error } => {
                format!("<subagent:failed id=\"{}\">{}</subagent:failed>", id, error)
            }
            SubAgentEvent::TimedOut { id, elapsed_secs } => {
                format!(
                    "<subagent:timeout id=\"{}\" elapsed_secs=\"{}\"/>",
                    id, elapsed_secs,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_agent_config_default() {
        let cfg = SubAgentConfig::default();
        assert_eq!(cfg.max_concurrency, 10);
        assert_eq!(cfg.max_context_window, 4096);
    }

    #[test]
    fn test_sub_agent_config_validate_clamps() {
        let mut cfg = SubAgentConfig { max_concurrency: 50, max_context_window: 100, idle_timeout_secs: 0 };
        cfg.validate();
        assert_eq!(cfg.max_concurrency, 20);
        assert_eq!(cfg.max_context_window, 256);
        assert_eq!(cfg.idle_timeout_secs, 1);
    }

    #[test]
    fn test_sub_agent_result_creation() {
        let r = SubAgentResult {
            summary: "done".into(),
            evidence: vec!["e1".into()],
            execution_metrics: HashMap::new(),
            total_tokens: 500,
            duration_ms: 1000,
        };
        assert_eq!(r.summary, "done");
        assert_eq!(r.total_tokens, 500);
    }

    #[test]
    fn test_sub_agent_handle_new() {
        let handle = SubAgentHandle {
            id: Uuid::new_v4(),
            status: SubAgentStatus::Running,
            started_at: 1000,
            result_slot: Arc::new(RwLock::new(None)),
        };
        assert_eq!(handle.status, SubAgentStatus::Running);
    }

    #[test]
    fn test_sub_agent_status_partial_eq() {
        assert_eq!(SubAgentStatus::Running, SubAgentStatus::Running);
        assert_ne!(SubAgentStatus::Running, SubAgentStatus::Completed);
    }

    #[test]
    fn test_sub_agent_event_done_to_tag_string() {
        let event = SubAgentEvent::Done {
            id: Uuid::nil(),
            summary: "task completed".into(),
            duration_ms: 500,
            total_tokens: 100,
        };
        let tag = event.to_tag_string();
        assert!(tag.contains("task completed"));
        assert!(tag.contains("500"));
    }

    #[test]
    fn test_sub_agent_event_failed_to_tag_string() {
        let event = SubAgentEvent::Failed {
            id: Uuid::nil(),
            error: "timeout".into(),
        };
        let tag = event.to_tag_string();
        assert!(tag.contains("timeout"));
    }

    #[test]
    fn test_sub_agent_event_timed_out_to_tag_string() {
        let event = SubAgentEvent::TimedOut {
            id: Uuid::nil(),
            elapsed_secs: 300,
        };
        let tag = event.to_tag_string();
        assert!(tag.contains("300"));
    }
}
