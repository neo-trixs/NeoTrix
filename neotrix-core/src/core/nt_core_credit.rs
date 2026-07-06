use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role of an event in the credit assignment graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditRole {
    /// Direct action that produced an outcome
    Actor,
    /// Context that influenced the outcome
    Context,
    /// Intermediate computation that enabled the action
    Enabler,
    /// Outcome/reward signal
    Outcome,
}

/// A single credit event — timestamped action/decision with attribution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditEvent {
    pub id: String,
    pub parent_id: Option<String>,
    pub role: CreditRole,
    pub label: String,
    pub e8_state: u8,
    pub timestamp: i64,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

/// Directed edge with attribution weight between two credit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditEdge {
    pub from: String,
    pub to: String,
    pub attribution: f64,
    pub discount: f64,
}

impl CreditEdge {
    pub fn discounted_attribution(&self) -> f64 {
        self.attribution * self.discount
    }
}

/// Temporal credit assignment graph — connects actions to outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditGraph {
    pub events: HashMap<String, CreditEvent>,
    pub edges: Vec<CreditEdge>,
}

impl Default for CreditGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CreditGraph {
    pub fn new() -> Self {
        Self { events: HashMap::new(), edges: Vec::new() }
    }

    pub fn add_event(&mut self, event: CreditEvent) {
        self.events.insert(event.id.clone(), event);
    }

    pub fn add_edge(&mut self, edge: CreditEdge) {
        self.edges.push(edge);
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.edges.clear();
    }

    pub fn size(&self) -> (usize, usize) {
        (self.events.len(), self.edges.len())
    }

    pub fn merge(&mut self, other: CreditGraph) {
        for (id, event) in other.events {
            self.events.entry(id).or_insert(event);
        }
        self.edges.extend(other.edges);
    }

    /// Serialize the credit graph to JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("CreditGraph serialize: {}", e))
    }

    /// Deserialize a credit graph from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("CreditGraph deserialize: {}", e))
    }

    /// Backpropagate credit from outcome events through the graph.
    /// Uses temporal discounting: closer events get more credit.
    pub fn backpropagate(&self, gamma: f64) -> HashMap<String, f64> {
        let mut credits: HashMap<String, f64> = HashMap::new();
        let outcomes: Vec<&CreditEvent> = self.events.values()
            .filter(|e| e.role == CreditRole::Outcome)
            .collect();

        for outcome in outcomes {
            let mut visited = vec![outcome.id.as_str()];
            let mut stack = vec![(outcome.id.as_str(), 1.0f64)];
            while let Some((node_id, chain_discount)) = stack.pop() {
                let predecessors: Vec<&CreditEdge> = self.edges.iter()
                    .filter(|e| e.to == *node_id)
                    .collect();
                for edge in &predecessors {
                    if visited.contains(&edge.from.as_str()) { continue; }
                    visited.push(edge.from.as_str());
                    let contrib = edge.discounted_attribution() * chain_discount * gamma;
                    *credits.entry(edge.from.clone()).or_insert(0.0) += contrib;
                    stack.push((edge.from.as_str(), chain_discount * gamma));
                }
            }
        }
        credits
    }
}

/// E8-specific credit policy — maps state transitions to credit weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8CreditPolicy {
    pub step_discount: f64,
    pub state_visit_bonus: f64,
    pub novelty_bonus: f64,
}

impl Default for E8CreditPolicy {
    fn default() -> Self {
        Self {
            step_discount: 0.95,
            state_visit_bonus: 0.1,
            novelty_bonus: 0.2,
        }
    }
}

impl E8CreditPolicy {
    pub fn compute_attribution(&self, steps_ago: usize, visit_count: u64) -> f64 {
        let time_discount = self.step_discount.powi(steps_ago as i32);
        let novelty = if visit_count <= 1 { self.novelty_bonus } else { 0.0 };
        (time_discount + novelty).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(id: &str, role: CreditRole, state: u8, ts: i64, weight: f64) -> CreditEvent {
        CreditEvent {
            id: id.into(), parent_id: None, role,
            label: format!("{}_{}", id, ts),
            e8_state: state, timestamp: ts, weight,
            metadata: HashMap::new(),
        }
    }

    fn make_edge(from: &str, to: &str, attr: f64, disc: f64) -> CreditEdge {
        CreditEdge { from: from.into(), to: to.into(), attribution: attr, discount: disc }
    }

    #[test]
    fn test_credit_graph_empty() {
        let g = CreditGraph::new();
        assert!(g.events.is_empty());
        assert!(g.edges.is_empty());
        assert_eq!(g.size(), (0, 0));
    }

    #[test]
    fn test_clear() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a", CreditRole::Actor, 0, 0, 1.0));
        g.add_edge(make_edge("a", "b", 1.0, 1.0));
        g.clear();
        assert_eq!(g.size(), (0, 0));
    }

    #[test]
    fn test_merge() {
        let mut g1 = CreditGraph::new();
        g1.add_event(make_event("a", CreditRole::Actor, 0, 0, 1.0));
        let mut g2 = CreditGraph::new();
        g2.add_event(make_event("b", CreditRole::Actor, 1, 0, 1.0));
        g1.merge(g2);
        assert!(g1.events.contains_key("a"));
        assert!(g1.events.contains_key("b"));
    }

    #[test]
    fn test_backpropagate_simple_chain() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("action_1", CreditRole::Actor, 0, 100, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 1, 200, 10.0));
        g.add_edge(make_edge("action_1", "reward", 1.0, 0.9));
        let credits = g.backpropagate(0.95);
        assert!(credits.contains_key("action_1"));
        assert!((credits["action_1"] - 0.855).abs() < 0.01);
    }

    #[test]
    fn test_backpropagate_no_outcomes() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_edge(make_edge("a1", "a2", 0.5, 0.9));
        let credits = g.backpropagate(0.95);
        assert!(credits.is_empty());
    }

    #[test]
    fn test_backpropagate_branching() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("a2", CreditRole::Context, 1, 10, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 2, 20, 5.0));
        g.add_edge(make_edge("a1", "reward", 0.6, 1.0));
        g.add_edge(make_edge("a2", "reward", 0.4, 1.0));
        let credits = g.backpropagate(0.95);
        assert!((credits["a1"] - 0.57).abs() < 0.01);
        assert!((credits["a2"] - 0.38).abs() < 0.01);
    }

    #[test]
    fn test_backpropagate_chain_multi_step() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("s1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("s2", CreditRole::Enabler, 1, 10, 1.0));
        g.add_event(make_event("s3", CreditRole::Enabler, 2, 20, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 3, 30, 10.0));
        g.add_edge(make_edge("s1", "s2", 1.0, 0.9));
        g.add_edge(make_edge("s2", "s3", 1.0, 0.9));
        g.add_edge(make_edge("s3", "reward", 1.0, 0.9));
        let credits = g.backpropagate(0.95);
        assert!(credits.contains_key("s1"));
        assert!(credits["s1"] > 0.0);
        assert!(credits["s3"] > credits["s1"]); // closer events get more credit
    }

    #[test]
    fn test_backpropagate_cycle_no_infinite_loop() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("b", CreditRole::Actor, 1, 10, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 2, 20, 5.0));
        g.add_edge(make_edge("a", "b", 1.0, 0.9));
        g.add_edge(make_edge("b", "a", 0.5, 0.9));
        g.add_edge(make_edge("b", "reward", 1.0, 0.9));
        let credits = g.backpropagate(0.95);
        assert!(credits.contains_key("a"));
        assert!(credits.contains_key("b"));
        // visited set should prevent infinite loop
    }

    #[test]
    fn test_backpropagate_gamma_zero() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 1, 10, 5.0));
        g.add_edge(make_edge("a1", "reward", 1.0, 0.9));
        let credits = g.backpropagate(0.0);
        assert!(credits.contains_key("a1"));
        assert!((credits["a1"]).abs() < 1e-10);
    }

    #[test]
    fn test_backpropagate_gamma_one() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("reward", CreditRole::Outcome, 1, 10, 5.0));
        g.add_edge(make_edge("a1", "reward", 1.0, 1.0));
        let credits = g.backpropagate(1.0);
        assert!((credits["a1"] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_backpropagate_disconnected_graph() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("outcome", CreditRole::Outcome, 1, 10, 5.0));
        let credits = g.backpropagate(0.95);
        assert!(credits.is_empty());
    }

    #[test]
    fn test_backpropagate_multiple_outcomes() {
        let mut g = CreditGraph::new();
        g.add_event(make_event("a1", CreditRole::Actor, 0, 0, 1.0));
        g.add_event(make_event("r1", CreditRole::Outcome, 1, 10, 3.0));
        g.add_event(make_event("r2", CreditRole::Outcome, 2, 20, 7.0));
        g.add_edge(make_edge("a1", "r1", 0.5, 1.0));
        g.add_edge(make_edge("a1", "r2", 1.0, 1.0));
        let credits = g.backpropagate(0.95);
        assert!((credits["a1"] - (0.5 * 0.95 + 1.0 * 0.95)).abs() < 0.01);
    }

    #[test]
    fn test_e8_credit_policy_discount() {
        let p = E8CreditPolicy::default();
        assert!((p.step_discount - 0.95).abs() < 0.01);
        let near = p.compute_attribution(1, 0);
        let far = p.compute_attribution(10, 0);
        assert!(near > far);
    }

    #[test]
    fn test_e8_novelty_bonus_first_visit() {
        let p = E8CreditPolicy::default();
        let first = p.compute_attribution(5, 0);
        let repeat = p.compute_attribution(5, 5);
        assert!(first > repeat);
    }

    #[test]
    fn test_e8_credit_policy_capped() {
        let p = E8CreditPolicy::default();
        let val = p.compute_attribution(0, 0);
        assert!(val <= 1.0);
    }

    #[test]
    fn test_credit_edge_discounted() {
        let e = CreditEdge {
            from: "a".into(), to: "b".into(),
            attribution: 0.8, discount: 0.9,
        };
        assert!((e.discounted_attribution() - 0.72).abs() < 0.01);
    }
}
