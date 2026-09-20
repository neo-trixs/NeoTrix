#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-model cost breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelBreakdown {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_calls: u64,
    pub compute_ms: u64,
}

/// Per-agent cost breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentBreakdown {
    pub agent_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_calls: u64,
    pub compute_ms: u64,
}

/// Cost summary for a single session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionCostSummary {
    pub session_id: String,
    pub total_tokens: u64,
    pub total_api_calls: u64,
    pub total_compute_ms: u64,
    pub per_model: Vec<ModelBreakdown>,
    pub per_agent: Vec<AgentBreakdown>,
}

impl SessionCostSummary {
    /// Estimated cost in USD based on model pricing.
    /// Returns tokens * rate (simplified: $0.03/1K input, $0.06/1K output).
    pub fn estimated_cost_usd(&self) -> f64 {
        let mut cost = 0.0;
        for model in &self.per_model {
            cost += model.input_tokens as f64 * 0.00003;
            cost += model.output_tokens as f64 * 0.00006;
        }
        cost
    }
}

/// Dashboard aggregating costs across multiple sessions.
pub struct CostDashboard {
    sessions: Vec<SessionCostSummary>,
}

impl CostDashboard {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Add a session cost summary.
    pub fn add_session(&mut self, summary: SessionCostSummary) {
        self.sessions.push(summary);
    }

    /// Total cost across all sessions.
    pub fn total_cost(&self) -> f64 {
        self.sessions.iter().map(|s| s.estimated_cost_usd()).sum()
    }

    /// Total tokens across all sessions.
    pub fn total_tokens(&self) -> u64 {
        self.sessions.iter().map(|s| s.total_tokens).sum()
    }

    /// Total API calls across all sessions.
    pub fn total_api_calls(&self) -> u64 {
        self.sessions.iter().map(|s| s.total_api_calls).sum()
    }

    /// Per-model breakdown aggregated across all sessions.
    pub fn per_model_breakdown(&self) -> Vec<ModelBreakdown> {
        let mut map: HashMap<String, ModelBreakdown> = HashMap::new();
        for session in &self.sessions {
            for model in &session.per_model {
                let entry = map.entry(model.model.clone()).or_default();
                entry.input_tokens += model.input_tokens;
                entry.output_tokens += model.output_tokens;
                entry.api_calls += model.api_calls;
                entry.compute_ms += model.compute_ms;
            }
        }
        let mut result: Vec<ModelBreakdown> = map.into_values().collect();
        result.sort_by(|a, b| b.api_calls.cmp(&a.api_calls));
        result
    }

    /// Per-agent breakdown aggregated across all sessions.
    pub fn per_agent_breakdown(&self) -> Vec<AgentBreakdown> {
        let mut map: HashMap<String, AgentBreakdown> = HashMap::new();
        for session in &self.sessions {
            for agent in &session.per_agent {
                let entry = map.entry(agent.agent_id.clone()).or_default();
                entry.input_tokens += agent.input_tokens;
                entry.output_tokens += agent.output_tokens;
                entry.api_calls += agent.api_calls;
                entry.compute_ms += agent.compute_ms;
            }
        }
        let mut result: Vec<AgentBreakdown> = map.into_values().collect();
        result.sort_by(|a, b| b.api_calls.cmp(&a.api_calls));
        result
    }

    /// Compare two sessions side-by-side.
    pub fn compare_sessions(&self, session_a: &str, session_b: &str) -> Option<SessionComparison> {
        let a = self.sessions.iter().find(|s| s.session_id == session_a)?;
        let b = self.sessions.iter().find(|s| s.session_id == session_b)?;
        Some(SessionComparison {
            session_a: a.clone(),
            session_b: b.clone(),
            token_diff: b.total_tokens as i64 - a.total_tokens as i64,
            api_call_diff: b.total_api_calls as i64 - a.total_api_calls as i64,
            compute_diff_ms: b.total_compute_ms as i64 - a.total_compute_ms as i64,
        })
    }

    /// Export all session data as CSV.
    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "session_id,total_tokens,total_api_calls,total_compute_ms,estimated_cost_usd\n",
        );
        for s in &self.sessions {
            csv.push_str(&format!(
                "{},{},{},{},{:.6}\n",
                s.session_id,
                s.total_tokens,
                s.total_api_calls,
                s.total_compute_ms,
                s.estimated_cost_usd(),
            ));
        }

        csv.push_str("\n# Model Breakdown\n");
        csv.push_str("model,input_tokens,output_tokens,api_calls,compute_ms\n");
        for model in self.per_model_breakdown() {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                model.model,
                model.input_tokens,
                model.output_tokens,
                model.api_calls,
                model.compute_ms,
            ));
        }

        csv
    }

    /// Number of tracked sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get a session summary by ID.
    pub fn get_session(&self, session_id: &str) -> Option<&SessionCostSummary> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }
}

impl Default for CostDashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Side-by-side session comparison.
#[derive(Debug, Clone)]
pub struct SessionComparison {
    pub session_a: SessionCostSummary,
    pub session_b: SessionCostSummary,
    pub token_diff: i64,
    pub api_call_diff: i64,
    pub compute_diff_ms: i64,
}

impl SessionComparison {
    pub fn summary(&self) -> String {
        format!(
            "A={} vs B={}: tokens {:+}, calls {:+}, compute {:+}ms",
            self.session_a.session_id,
            self.session_b.session_id,
            self.token_diff,
            self.api_call_diff,
            self.compute_diff_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(id: &str, tokens: u64, calls: u64, model: &str) -> SessionCostSummary {
        SessionCostSummary {
            session_id: id.into(),
            total_tokens: tokens,
            total_api_calls: calls,
            total_compute_ms: calls * 100,
            per_model: vec![ModelBreakdown {
                model: model.into(),
                input_tokens: tokens / 2,
                output_tokens: tokens / 2,
                api_calls: calls,
                compute_ms: calls * 100,
            }],
            per_agent: vec![],
        }
    }

    #[test]
    fn test_total_cost() {
        let mut dash = CostDashboard::new();
        dash.add_session(make_session("s1", 1000, 5, "gpt-4"));
        dash.add_session(make_session("s2", 2000, 10, "gpt-4"));
        assert_eq!(dash.total_tokens(), 3000);
        assert_eq!(dash.total_api_calls(), 15);
        assert!(dash.total_cost() > 0.0);
    }

    #[test]
    fn test_per_model_breakdown() {
        let mut dash = CostDashboard::new();
        dash.add_session(make_session("s1", 1000, 5, "gpt-4"));
        dash.add_session(make_session("s2", 2000, 10, "claude-3"));
        let models = dash.per_model_breakdown();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_compare_sessions() {
        let mut dash = CostDashboard::new();
        dash.add_session(make_session("s1", 1000, 5, "gpt-4"));
        dash.add_session(make_session("s2", 2000, 10, "gpt-4"));
        let cmp = dash.compare_sessions("s1", "s2").unwrap();
        assert_eq!(cmp.token_diff, 1000);
        assert_eq!(cmp.api_call_diff, 5);
    }

    #[test]
    fn test_compare_sessions_missing() {
        let dash = CostDashboard::new();
        assert!(dash.compare_sessions("s1", "s2").is_none());
    }

    #[test]
    fn test_export_csv() {
        let mut dash = CostDashboard::new();
        dash.add_session(make_session("s1", 1000, 5, "gpt-4"));
        let csv = dash.export_csv();
        assert!(csv.contains("session_id"));
        assert!(csv.contains("s1"));
    }

    #[test]
    fn test_estimated_cost() {
        let s = make_session("s1", 1000, 5, "gpt-4");
        let cost = s.estimated_cost_usd();
        assert!(cost > 0.0);
        assert!(cost < 1.0);
    }
}
