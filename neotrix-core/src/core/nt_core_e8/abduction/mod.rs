#![forbid(unsafe_code)]

pub mod causal_graph;
pub mod directed_search;
pub mod state_machine;

pub use causal_graph::{CausalEdge, CausalGraph, CausalNode};
pub use directed_search::{DirectedSearch, SearchStrategy};
pub use state_machine::{AbductiveState, AbductiveStateMachine};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveHypothesis {
    pub id: usize,
    pub causes: Vec<usize>,
    pub effect: usize,
    pub plausibility: f64,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductionCycleReport {
    pub hypotheses_generated: usize,
    pub best_hypothesis: Option<AbductiveHypothesis>,
    pub convergence: f64,
    pub cycles_used: u32,
    pub state_transitions: Vec<AbductiveState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveReasoningEngine {
    pub graph: CausalGraph,
    pub state_machine: AbductiveStateMachine,
    pub search: DirectedSearch,
    pub hypotheses: Vec<AbductiveHypothesis>,
    pub cycle_count: u32,
    max_hypotheses: usize,
    convergence_threshold: f64,
    next_hypothesis_id: usize,
}

impl AbductiveReasoningEngine {
    pub fn new() -> Self {
        Self {
            graph: CausalGraph::new(),
            state_machine: AbductiveStateMachine::new(),
            search: DirectedSearch::new(SearchStrategy::BFS, 10),
            hypotheses: Vec::new(),
            cycle_count: 0,
            max_hypotheses: 50,
            convergence_threshold: 0.8,
            next_hypothesis_id: 0,
        }
    }

    pub fn add_observation(&mut self, desc: String) {
        self.graph.add_node(desc, 1.0);
        if let Some(n) = self.graph.nodes.last_mut() {
            n.observed = true;
        }
        self.state_machine
            .transition(AbductiveState::HypothesisGeneration);
    }

    pub fn generate_hypotheses(&mut self, max_hypotheses: usize) -> Vec<AbductiveHypothesis> {
        let mut hypotheses = Vec::new();
        let observed_ids: Vec<usize> = self
            .graph
            .nodes
            .iter()
            .filter(|n| n.observed)
            .map(|n| n.id)
            .collect();

        for &obs_id in &observed_ids {
            let paths = self
                .graph
                .find_abductive_explanations(obs_id, self.search.max_depth);
            for path in &paths {
                if hypotheses.len() >= max_hypotheses {
                    break;
                }
                if path.len() < 2 {
                    continue;
                }
                let causes: Vec<usize> = path[..path.len() - 1].to_vec();
                let confidence: f64 = causes
                    .iter()
                    .filter_map(|cid| {
                        self.graph
                            .nodes
                            .iter()
                            .find(|n| n.id == *cid)
                            .map(|n| n.confidence)
                    })
                    .product();
                let plausibility = self.compute_plausibility(&causes, obs_id);

                let h = AbductiveHypothesis {
                    id: self.next_hypothesis_id,
                    causes: causes.clone(),
                    effect: obs_id,
                    plausibility,
                    evidence: Vec::new(),
                    confidence,
                };
                self.next_hypothesis_id += 1;
                hypotheses.push(h);
            }
            if hypotheses.len() >= max_hypotheses {
                break;
            }
        }

        self.hypotheses = hypotheses.clone();
        hypotheses
    }

    fn compute_plausibility(&self, causes: &[usize], effect: usize) -> f64 {
        if causes.is_empty() {
            return 0.0;
        }
        let mut total_strength = 0.0;
        let mut count = 0;
        for edge in &self.graph.edges {
            if edge.to == effect && causes.contains(&edge.from) {
                total_strength += edge.strength;
                count += 1;
            }
        }
        if count == 0 {
            return 0.0;
        }
        (total_strength / count as f64).max(0.0).min(1.0)
    }

    pub fn evaluate_hypothesis(&self, h: &AbductiveHypothesis) -> f64 {
        let pl = h.plausibility;
        let ev = (h.evidence.len() as f64).min(10.0) / 10.0;
        let cf = h.confidence;
        (pl * 0.4 + ev * 0.3 + cf * 0.3).max(0.0).min(1.0)
    }

    pub fn run_abduction_cycle(&mut self, max_hypotheses: usize) -> AbductionCycleReport {
        let initial_state = *self.state_machine.current();
        let mut state_transitions = vec![initial_state];

        if *self.state_machine.current() == AbductiveState::Observation {
            self.state_machine
                .transition(AbductiveState::HypothesisGeneration);
            state_transitions.push(AbductiveState::HypothesisGeneration);
        }

        let hypotheses = self.generate_hypotheses(max_hypotheses);

        self.state_machine.transition(AbductiveState::Evaluation);
        state_transitions.push(AbductiveState::Evaluation);

        let best_hypothesis = hypotheses
            .iter()
            .max_by(|a, b| {
                self.evaluate_hypothesis(a)
                    .partial_cmp(&self.evaluate_hypothesis(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        let convergence = best_hypothesis
            .as_ref()
            .map(|h| self.evaluate_hypothesis(h))
            .unwrap_or(0.0);

        if convergence >= self.convergence_threshold {
            self.state_machine.transition(AbductiveState::Revision);
            state_transitions.push(AbductiveState::Revision);
            self.state_machine.transition(AbductiveState::Acceptance);
            state_transitions.push(AbductiveState::Acceptance);
        } else {
            self.state_machine.transition(AbductiveState::Revision);
            state_transitions.push(AbductiveState::Revision);
            let regenerated = self.generate_hypotheses(max_hypotheses);
            if !regenerated.is_empty() {
                self.state_machine.transition(AbductiveState::Evaluation);
                state_transitions.push(AbductiveState::Evaluation);
                let re_best = regenerated
                    .iter()
                    .max_by(|a, b| {
                        self.evaluate_hypothesis(a)
                            .partial_cmp(&self.evaluate_hypothesis(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .cloned();
                let new_conv = re_best
                    .as_ref()
                    .map(|h| self.evaluate_hypothesis(h))
                    .unwrap_or(0.0);
                if new_conv >= self.convergence_threshold {
                    self.state_machine.transition(AbductiveState::Acceptance);
                    state_transitions.push(AbductiveState::Acceptance);
                }
            }
        }

        self.cycle_count += 1;

        AbductionCycleReport {
            hypotheses_generated: hypotheses.len(),
            best_hypothesis,
            convergence,
            cycles_used: self.cycle_count,
            state_transitions,
        }
    }

    pub fn causal_graph(&self) -> &CausalGraph {
        &self.graph
    }

    pub fn state(&self) -> AbductiveState {
        *self.state_machine.current()
    }
}

impl Default for AbductiveReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AbductiveReasoningEngine::new();
        assert_eq!(engine.state(), AbductiveState::Observation);
        assert_eq!(engine.cycle_count, 0);
    }

    #[test]
    fn test_add_observation() {
        let mut engine = AbductiveReasoningEngine::new();
        engine.add_observation("test event".into());
        assert_eq!(engine.graph.node_count(), 1);
        assert!(engine.graph.nodes[0].observed);
    }

    #[test]
    fn test_generate_hypotheses_with_observation() {
        let mut engine = AbductiveReasoningEngine::new();
        let cause = engine.graph.add_node("cause".into(), 0.8);
        let effect = engine.graph.add_node("effect".into(), 1.0);
        engine.graph.nodes.iter_mut().for_each(|n| {
            if n.id == effect {
                n.observed = true;
            }
        });
        engine.graph.add_edge(cause, effect, "triggers".into(), 0.9);
        engine
            .state_machine
            .transition(AbductiveState::HypothesisGeneration);
        let hypotheses = engine.generate_hypotheses(10);
        assert!(!hypotheses.is_empty());
        assert_eq!(hypotheses[0].effect, effect);
    }

    #[test]
    fn test_evaluate_hypothesis() {
        let engine = AbductiveReasoningEngine::new();
        let h = AbductiveHypothesis {
            id: 0,
            causes: vec![1, 2],
            effect: 3,
            plausibility: 0.7,
            evidence: vec!["e1".into(), "e2".into()],
            confidence: 0.8,
        };
        let score = engine.evaluate_hypothesis(&h);
        assert!((score - 0.7 * 0.4 - 0.2 * 0.3 - 0.8 * 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_run_abduction_cycle() {
        let mut engine = AbductiveReasoningEngine::new();
        let cause = engine.graph.add_node("rain".into(), 0.9);
        let effect = engine.graph.add_node("wet ground".into(), 1.0);
        engine.graph.nodes.iter_mut().for_each(|n| {
            if n.id == effect {
                n.observed = true;
            }
        });
        engine.graph.add_edge(cause, effect, "causes".into(), 0.95);
        engine
            .state_machine
            .transition(AbductiveState::HypothesisGeneration);

        let report = engine.run_abduction_cycle(10);
        assert!(report.hypotheses_generated > 0);
    }

    #[test]
    fn test_multi_cycle_convergence() {
        let mut engine = AbductiveReasoningEngine::new();
        engine.convergence_threshold = 0.3;
        let r1 = engine.graph.add_node("root".into(), 0.9);
        let r2 = engine.graph.add_node("root2".into(), 0.8);
        let obs = engine.graph.add_node("observed".into(), 1.0);
        engine.graph.nodes.iter_mut().for_each(|n| {
            if n.id == obs {
                n.observed = true;
            }
        });
        engine.graph.add_edge(r1, obs, "causes".into(), 0.7);
        engine.graph.add_edge(r2, obs, "causes".into(), 0.6);
        engine
            .state_machine
            .transition(AbductiveState::HypothesisGeneration);
        let report = engine.run_abduction_cycle(10);
        assert!(report.hypotheses_generated > 0);
        assert!(!report.state_transitions.is_empty());
    }

    #[test]
    fn test_causal_graph_access() {
        let engine = AbductiveReasoningEngine::new();
        let g = engine.causal_graph();
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn test_hypothesis_id_increment() {
        let mut engine = AbductiveReasoningEngine::new();
        let cause = engine.graph.add_node("c".into(), 0.9);
        let obs = engine.graph.add_node("o".into(), 1.0);
        engine.graph.nodes.iter_mut().for_each(|n| {
            if n.id == obs {
                n.observed = true;
            }
        });
        engine.graph.add_edge(cause, obs, "c".into(), 0.8);
        engine
            .state_machine
            .transition(AbductiveState::HypothesisGeneration);
        let h1 = engine.generate_hypotheses(5);
        let h2 = engine.generate_hypotheses(5);
        assert_eq!(h2[0].id, h1.len() as usize);
    }

    #[test]
    fn test_engine_default() {
        let engine = AbductiveReasoningEngine::default();
        assert_eq!(engine.max_hypotheses, 50);
        assert!((engine.convergence_threshold - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_cycle_report_fields() {
        let mut engine = AbductiveReasoningEngine::new();
        let c = engine.graph.add_node("c".into(), 0.8);
        let o = engine.graph.add_node("o".into(), 1.0);
        engine.graph.nodes.iter_mut().for_each(|n| {
            if n.id == o {
                n.observed = true;
            }
        });
        engine.graph.add_edge(c, o, "".into(), 0.7);
        engine
            .state_machine
            .transition(AbductiveState::HypothesisGeneration);
        let report = engine.run_abduction_cycle(5);
        assert!(report.cycles_used >= 1);
        assert!(report.hypotheses_generated <= 5);
    }
}
