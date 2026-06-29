use std::collections::VecDeque;

const DEFAULT_WINDOW: usize = 20;
const DEFAULT_BETA: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditOutcome {
    Success,
    Failure,
    NoEffect,
}

#[derive(Debug, Clone)]
pub struct AgentEditHistory {
    pub agent_name: String,
    pub outcomes: VecDeque<EditOutcome>,
    pub max_len: usize,
}

impl AgentEditHistory {
    pub fn new(agent_name: &str, max_len: usize) -> Self {
        Self {
            agent_name: agent_name.to_string(),
            outcomes: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn record(&mut self, outcome: EditOutcome) {
        self.outcomes.push_back(outcome);
        while self.outcomes.len() > self.max_len {
            self.outcomes.pop_front();
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.outcomes.is_empty() {
            return 0.5;
        }
        let successes = self
            .outcomes
            .iter()
            .filter(|o| matches!(o, EditOutcome::Success))
            .count();
        successes as f64 / self.outcomes.len() as f64
    }

    pub fn total(&self) -> usize {
        self.outcomes.len()
    }
}

pub struct ReliabilityGate {
    agents: Vec<AgentEditHistory>,
    beta: f64,
    window: usize,
}

impl ReliabilityGate {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            beta: DEFAULT_BETA,
            window: DEFAULT_WINDOW,
        }
    }

    pub fn with_beta(mut self, beta: f64) -> Self {
        self.beta = beta;
        self
    }

    pub fn with_window(mut self, window: usize) -> Self {
        self.window = window;
        self
    }

    pub fn register_agent(&mut self, name: &str) {
        if !self.agents.iter().any(|a| a.agent_name == name) {
            self.agents.push(AgentEditHistory::new(name, self.window));
        }
    }

    pub fn record_outcome(&mut self, agent: &str, outcome: EditOutcome) {
        if let Some(history) = self.agents.iter_mut().find(|a| a.agent_name == agent) {
            history.record(outcome);
        }
    }

    pub fn success_rate(&self, agent: &str) -> f64 {
        self.agents
            .iter()
            .find(|a| a.agent_name == agent)
            .map(|h| h.success_rate())
            .unwrap_or(0.5)
    }

    pub fn gate_value(&self, agent: &str) -> f64 {
        let rate = self.success_rate(agent);
        sigmoid(self.beta * (rate - 0.5))
    }

    pub fn gate_delta(&self, agent: &str, raw_delta: f64) -> f64 {
        self.gate_value(agent) * raw_delta
    }

    pub fn is_reliable(&self, agent: &str, min_rate: f64) -> bool {
        self.success_rate(agent) >= min_rate
    }

    pub fn report(&self) -> ReliabilityReport {
        let entries: Vec<AgentReliabilityEntry> = self
            .agents
            .iter()
            .map(|a| AgentReliabilityEntry {
                agent: a.agent_name.clone(),
                success_rate: a.success_rate(),
                total_edits: a.total(),
                gate: sigmoid(self.beta * (a.success_rate() - 0.5)),
            })
            .collect();
        ReliabilityReport { agents: entries }
    }
}

#[derive(Debug, Clone)]
pub struct AgentReliabilityEntry {
    pub agent: String,
    pub success_rate: f64,
    pub total_edits: usize,
    pub gate: f64,
}

#[derive(Debug, Clone)]
pub struct ReliabilityReport {
    pub agents: Vec<AgentReliabilityEntry>,
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gate_defaults() {
        let g = ReliabilityGate::new();
        assert_eq!(g.window, 20);
        assert!((g.beta - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_register_and_track() {
        let mut g = ReliabilityGate::new();
        g.register_agent("meta_v1");
        assert!((g.success_rate("meta_v1") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_success_rate_after_records() {
        let mut g = ReliabilityGate::new();
        g.register_agent("meta_v1");
        for _ in 0..8 {
            g.record_outcome("meta_v1", EditOutcome::Success);
        }
        for _ in 0..2 {
            g.record_outcome("meta_v1", EditOutcome::Failure);
        }
        assert!((g.success_rate("meta_v1") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_gate_value_high_reliability() {
        let mut g = ReliabilityGate::new().with_beta(5.0);
        g.register_agent("reliable");
        for _ in 0..18 {
            g.record_outcome("reliable", EditOutcome::Success);
        }
        for _ in 0..2 {
            g.record_outcome("reliable", EditOutcome::Failure);
        }
        let gv = g.gate_value("reliable");
        assert!(
            gv > 0.8,
            "gate should be high for reliable agent, got {}",
            gv
        );
    }

    #[test]
    fn test_gate_value_low_reliability() {
        let mut g = ReliabilityGate::new().with_beta(5.0);
        g.register_agent("unreliable");
        for _ in 0..4 {
            g.record_outcome("unreliable", EditOutcome::Success);
        }
        for _ in 0..16 {
            g.record_outcome("unreliable", EditOutcome::Failure);
        }
        let gv = g.gate_value("unreliable");
        assert!(
            gv < 0.2,
            "gate should be low for unreliable agent, got {}",
            gv
        );
    }

    #[test]
    fn test_gate_delta_attenuates() {
        let mut g = ReliabilityGate::new().with_beta(5.0);
        g.register_agent("noisy");
        for _ in 0..5 {
            g.record_outcome("noisy", EditOutcome::Success);
        }
        for _ in 0..15 {
            g.record_outcome("noisy", EditOutcome::Failure);
        }
        let attenuated = g.gate_delta("noisy", 1.0);
        assert!(
            attenuated < 0.5,
            "unreliable agent delta should be attenuated, got {}",
            attenuated
        );
    }

    #[test]
    fn test_is_reliable() {
        let mut g = ReliabilityGate::new();
        g.register_agent("good");
        for _ in 0..20 {
            g.record_outcome("good", EditOutcome::Success);
        }
        assert!(g.is_reliable("good", 0.7));
    }

    #[test]
    fn test_report_generation() {
        let mut g = ReliabilityGate::new();
        g.register_agent("a1");
        g.register_agent("a2");
        g.record_outcome("a1", EditOutcome::Success);
        g.record_outcome("a2", EditOutcome::Failure);
        let r = g.report();
        assert_eq!(r.agents.len(), 2);
    }

    #[test]
    fn test_window_eviction() {
        let mut g = ReliabilityGate::new().with_window(10);
        g.register_agent("evict_test");
        for _ in 0..20 {
            g.record_outcome("evict_test", EditOutcome::Success);
        }
        let h = g
            .agents
            .iter()
            .find(|a| a.agent_name == "evict_test")
            .unwrap();
        assert_eq!(h.outcomes.len(), 10);
    }

    #[test]
    fn test_no_effect_counts_as_neutral() {
        let mut g = ReliabilityGate::new();
        g.register_agent("neutral");
        g.record_outcome("neutral", EditOutcome::NoEffect);
        g.record_outcome("neutral", EditOutcome::Success);
        g.record_outcome("neutral", EditOutcome::NoEffect);
        g.record_outcome("neutral", EditOutcome::NoEffect);
        let rate = g.success_rate("neutral");
        assert!((rate - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_unknown_agent_returns_default() {
        let g = ReliabilityGate::new();
        assert!((g.success_rate("unknown") - 0.5).abs() < 1e-6);
        assert!((g.gate_value("unknown") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sigmoid_symmetry() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-6);
        assert!((sigmoid(5.0 * 0.5) - sigmoid(-5.0 * 0.5)).abs() < 1e-6);
    }
}
