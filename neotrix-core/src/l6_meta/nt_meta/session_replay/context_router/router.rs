#![deny(clippy::unwrap_used)]

// 2026-09-30：`ContextSnapshot` 在本文件未使用 ⇒ 移除。
use super::context::ContextState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A routing decision produced by the context router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub target_agent: String,
    pub reasoning: String,
    pub confidence: f64,
}

/// Capability descriptor for an agent that the router evaluates.
#[derive(Debug, Clone)]
pub struct AgentCapability {
    pub name: String,
    pub keywords: Vec<String>,
}

impl AgentCapability {
    pub fn new(name: impl Into<String>, keywords: Vec<String>) -> Self {
        Self {
            name: name.into(),
            keywords,
        }
    }
}

/// Context-aware router that selects an agent based on task, context state, and preference history.
pub struct ContextRouter {
    agents: Vec<AgentCapability>,
    preference_history: HashMap<String, u64>,
}

impl ContextRouter {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            preference_history: HashMap::new(),
        }
    }

    /// Register an agent with its capability keywords.
    pub fn register_agent(&mut self, agent: AgentCapability) {
        self.agents.push(agent);
    }

    /// Route a task given the current context.
    /// Rules: match agent capabilities → check active_agents → check user_preferences → fallback.
    pub fn route(&mut self, task: &str, context: &ContextState) -> RouteDecision {
        let task_lower = task.to_lowercase();

        // Phase 1: keyword scoring against task
        let mut scores: Vec<(String, f64, String)> = self
            .agents
            .iter()
            .map(|cap| {
                let hits: usize = cap
                    .keywords
                    .iter()
                    .filter(|kw| task_lower.contains(kw.as_str()))
                    .count();
                let score = if cap.keywords.is_empty() {
                    0.0
                } else {
                    hits as f64 / cap.keywords.len() as f64
                };
                let reasoning = if hits > 0 {
                    format!(
                        "matched {}/{} keywords for '{}'",
                        hits,
                        cap.keywords.len(),
                        cap.name
                    )
                } else {
                    format!("no keyword match for '{}'", cap.name)
                };
                (cap.name.clone(), score, reasoning)
            })
            .collect();

        // Phase 2: boost if agent is already active
        for entry in &mut scores {
            if context.active_agents.iter().any(|a| a == &entry.0) {
                entry.1 += 0.15;
                entry.2.push_str(" + already active");
            }
        }

        // Phase 3: boost if agent matches user preference history
        for entry in &mut scores {
            if let Some(count) = self.preference_history.get(&entry.0) {
                let pref_boost = (*count as f64 * 0.05).min(0.2);
                entry.1 += pref_boost;
                entry
                    .2
                    .push_str(&format!(" + preference history ({count})"));
            }
        }

        // Phase 4: environment hints (e.g., debug mode → debugger agent)
        if let Some(mode) = context.environment.get("mode") {
            if mode == "debug" {
                for entry in &mut scores {
                    if entry.0.contains("debug") {
                        entry.1 += 0.1;
                        entry.2.push_str(" + debug environment");
                    }
                }
            }
        }

        // Select best
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (target, score, reasoning) = scores.into_iter().next().unwrap_or_else(|| {
            (
                "default".to_string(),
                0.0,
                "no agents registered, using default".to_string(),
            )
        });

        let confidence = score.clamp(0.0, 1.0);

        // Record preference
        *self.preference_history.entry(target.clone()).or_insert(0) += 1;

        RouteDecision {
            target_agent: target,
            reasoning,
            confidence,
        }
    }
}

impl Default for ContextRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_router() -> ContextRouter {
        let mut router = ContextRouter::new();
        router.register_agent(AgentCapability::new(
            "researcher",
            vec!["search".into(), "find".into(), "research".into()],
        ));
        router.register_agent(AgentCapability::new(
            "coder",
            vec!["code".into(), "implement".into(), "fix".into()],
        ));
        router.register_agent(AgentCapability::new(
            "debugger",
            vec!["debug".into(), "error".into(), "trace".into()],
        ));
        router
    }

    #[test]
    fn test_route_keyword_match() {
        let mut router = make_router();
        let ctx = ContextState::new();
        let decision = router.route("search the web", &ctx);
        assert_eq!(decision.target_agent, "researcher");
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_route_active_agent_boost() {
        let mut router = make_router();
        let ctx = ContextState::new().with_agent("coder");
        let decision = router.route("code a function", &ctx);
        assert_eq!(decision.target_agent, "coder");
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_route_preference_history() {
        let mut router = make_router();
        let ctx = ContextState::new();

        // First call: both have similar keyword match
        router.route("fix the bug", &ctx);

        // Second call with same agent should get preference boost
        let decision = router.route("fix the issue", &ctx);
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_route_debug_environment() {
        let mut router = make_router();
        let ctx = ContextState::new().with_env("mode", "debug");
        let decision = router.route("trace the error", &ctx);
        assert_eq!(decision.target_agent, "debugger");
    }

    #[test]
    fn test_route_no_agents() {
        let mut router = ContextRouter::new();
        let ctx = ContextState::new();
        let decision = router.route("anything", &ctx);
        assert_eq!(decision.target_agent, "default");
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_route_no_matching_keywords() {
        let mut router = make_router();
        let ctx = ContextState::new();
        let decision = router.route("deploy to production", &ctx);
        // No agent has "deploy" keyword — should fallback to first agent with 0 score
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_route_multiple_similar_agents() {
        let mut router = ContextRouter::new();
        router.register_agent(AgentCapability::new(
            "search_a",
            vec!["search".into(), "find".into()],
        ));
        router.register_agent(AgentCapability::new(
            "search_b",
            vec!["search".into(), "lookup".into()],
        ));
        let ctx = ContextState::new();
        let decision = router.route("search for docs", &ctx);
        // Both match "search" — both get 0.5 score, first one wins
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_route_preference_accumulates() {
        let mut router = make_router();
        let ctx = ContextState::new();

        // Route multiple times to build preference history
        for _ in 0..5 {
            router.route("search the web", &ctx);
        }

        let decision = router.route("search again", &ctx);
        assert!(decision.confidence > 0.0);
        assert!(decision.reasoning.contains("preference history"));
    }

    #[test]
    fn test_route_debug_mode_boost() {
        let mut router = make_router();
        let ctx = ContextState::new().with_env("mode", "debug");

        // "debug the error" matches both debugger and coder (has "fix" not in task)
        let decision = router.route("debug the error", &ctx);
        assert_eq!(decision.target_agent, "debugger");
    }

    #[test]
    fn test_agent_capability_new() {
        let cap = AgentCapability::new("test_agent", vec!["a".into(), "b".into()]);
        assert_eq!(cap.name, "test_agent");
        assert_eq!(cap.keywords.len(), 2);
    }

    #[test]
    fn test_route_decision_serialization() {
        let decision = RouteDecision {
            target_agent: "coder".into(),
            reasoning: "matched 2/3 keywords".into(),
            confidence: 0.75,
        };
        let json = serde_json::to_string(&decision).unwrap();
        let back: RouteDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.target_agent, "coder");
        assert!((back.confidence - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_context_router_default() {
        let router = ContextRouter::default();
        assert!(router.agents.is_empty());
    }
}
