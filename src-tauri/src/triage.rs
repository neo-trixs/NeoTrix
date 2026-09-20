use serde::{Deserialize, Serialize};

/// Triage decision — the Cumora "cerebellum" pattern.
///
/// A small, fast model acts as a gate before the expensive reasoning model.
/// It classifies incoming messages into actionable categories and decides
/// whether to wake the full agent or suppress the message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriageAction {
    /// Message needs full agent attention — wake it.
    Wake,
    /// Message is noise — suppress, don't wake the agent.
    Suppress,
    /// Message is informational — update state but don't wake.
    UpdateOnly,
    /// Message is urgent — bypass queue, wake immediately.
    Urgent,
}

impl TriageAction {
    pub fn should_wake(&self) -> bool {
        matches!(self, Self::Wake | Self::Urgent)
    }

    pub fn should_update(&self) -> bool {
        !matches!(self, Self::Suppress)
    }
}

/// Triage classification of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    pub action: TriageAction,
    pub reason: String,
    pub suggested_tier: Option<String>,
    pub is_complex: bool,
    pub requires_tools: bool,
}

/// The triage gate — a lightweight classifier that sits before the main agent.
///
/// In Cumora, this is `triage-core.ts` — a small model (Haiku, GPT-4o-mini)
/// that classifies every incoming message. Only "wake-worthy" messages
/// reach the expensive model.
///
/// For NeoTrix, we implement this as a rule-based + optional LLM classifier.
pub struct TriageGate {
    /// Keywords that always trigger a wake.
    urgent_keywords: Vec<String>,
    /// Keywords that suppress (e.g., "typing indicator", "status update").
    suppress_keywords: Vec<String>,
    /// Max messages per minute before throttling.
    rate_limit: u32,
}

impl TriageGate {
    pub fn new() -> Self {
        Self {
            urgent_keywords: vec![
                "urgent".into(),
                "critical".into(),
                "error".into(),
                "failed".into(),
                "alert".into(),
                "immediately".into(),
                "asap".into(),
            ],
            suppress_keywords: vec![
                "typing".into(),
                "status update".into(),
                "heartbeat".into(),
                "ping".into(),
                "seen".into(),
                "delivered".into(),
            ],
            rate_limit: 30,
        }
    }

    /// Classify a message and decide the triage action.
    pub fn classify(&self, message: &str, is_from_agent: bool) -> TriageResult {
        let lower = message.to_lowercase();

        // Agent-to-agent noise suppression
        if is_from_agent {
            return TriageResult {
                action: TriageAction::Suppress,
                reason: "agent-to-agent message".into(),
                suggested_tier: None,
                is_complex: false,
                requires_tools: false,
            };
        }

        // Urgent keyword match
        for keyword in &self.urgent_keywords {
            if lower.contains(keyword.as_str()) {
                return TriageResult {
                    action: TriageAction::Urgent,
                    reason: format!("urgent keyword: {keyword}"),
                    suggested_tier: Some("think".into()),
                    is_complex: true,
                    requires_tools: false,
                };
            }
        }

        // Suppress keyword match
        for keyword in &self.suppress_keywords {
            if lower.contains(keyword.as_str()) {
                return TriageResult {
                    action: TriageAction::Suppress,
                    reason: format!("suppress keyword: {keyword}"),
                    suggested_tier: None,
                    is_complex: false,
                    requires_tools: false,
                };
            }
        }

        // Complexity heuristics
        let is_complex = message.len() > 500
            || message.contains("```")
            || message.contains("分析")
            || message.contains("analyze")
            || message.contains("compare")
            || message.contains("design")
            || message.contains("architect");

        // Tool-use heuristics
        let requires_tools = lower.contains("search")
            || lower.contains("fetch")
            || lower.contains("run")
            || lower.contains("execute")
            || lower.contains("install")
            || lower.contains("deploy");

        let action = if is_complex {
            TriageAction::Wake
        } else if requires_tools {
            TriageAction::Wake
        } else {
            TriageAction::UpdateOnly
        };

        let suggested_tier = if is_complex {
            Some("think".into())
        } else if requires_tools {
            Some("standard".into())
        } else {
            Some("fast".into())
        };

        TriageResult {
            action,
            reason: "rule-based classification".into(),
            suggested_tier,
            is_complex,
            requires_tools,
        }
    }
}

impl Default for TriageGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urgent_keywords_trigger_wake() {
        let gate = TriageGate::new();
        let result = gate.classify("This is an urgent error", false);
        assert_eq!(result.action, TriageAction::Urgent);
        assert!(result.is_complex);
    }

    #[test]
    fn suppress_keywords_block() {
        let gate = TriageGate::new();
        let result = gate.classify("typing indicator update", false);
        assert_eq!(result.action, TriageAction::Suppress);
    }

    #[test]
    fn agent_to_agent_suppressed() {
        let gate = TriageGate::new();
        let result = gate.classify("Hello from another agent", true);
        assert_eq!(result.action, TriageAction::Suppress);
    }

    #[test]
    fn complex_message_wakes() {
        let gate = TriageGate::new();
        let result = gate.classify("Please analyze the architecture of this system and compare with alternatives", false);
        assert_eq!(result.action, TriageAction::Wake);
        assert!(result.is_complex);
    }

    #[test]
    fn tool_request_wakes() {
        let gate = TriageGate::new();
        let result = gate.classify("search for the latest version", false);
        assert_eq!(result.action, TriageAction::Wake);
        assert!(result.requires_tools);
    }

    #[test]
    fn simple_message_update_only() {
        let gate = TriageGate::new();
        let result = gate.classify("ok", false);
        assert_eq!(result.action, TriageAction::UpdateOnly);
    }
}
