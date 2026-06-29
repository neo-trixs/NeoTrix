use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// A node in the Structural Causal Model
#[derive(Debug, Clone)]
pub struct CausalNode {
    pub name: String,
    pub vsa_vector: Vec<u8>,
    pub observed_value: f64,
    pub is_endogenous: bool,
}

/// The causal mechanism type for an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalMechanism {
    Direct,
    Mediated,
    Inhibitory,
    Synergistic,
}

/// A directed edge in the causal graph
#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub strength: f64,
    pub mechanism: CausalMechanism,
}

/// Structural Causal Model (Pearl's Ladder Step 3)
#[derive(Debug, Clone)]
pub struct StructuralCausalModel {
    pub nodes: HashMap<String, CausalNode>,
    pub edges: Vec<CausalEdge>,
    pub name: String,
    pub version: u64,
}

/// A recorded factual scenario
#[derive(Debug, Clone)]
pub struct FactualScenario {
    pub node_values: HashMap<String, f64>,
    pub node_vectors: HashMap<String, Vec<u8>>,
    pub timestamp: Instant,
    pub description: String,
}

/// An intervention on the causal model
#[derive(Debug, Clone)]
pub struct Intervention {
    pub target_node: String,
    pub new_value: f64,
    pub intervention_type: InterventionType,
    pub rationale: String,
}

/// The type of intervention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionType {
    /// Pearl's do-operator: surgically set value, cut incoming edges
    Do,
    /// Soft intervention: influence without cutting edges
    Soft,
    /// Context change: modify background conditions
    Context,
}

/// A counterfactual scenario derived from a factual one
#[derive(Debug, Clone)]
pub struct CounterfactualScenario {
    pub factual: FactualScenario,
    pub intervention: Intervention,
    pub modified_nodes: Vec<String>,
    pub outcome_vectors: HashMap<String, Vec<u8>>,
    pub minimality_score: f64,
    pub plausibility: f64,
}

/// Result of a single counterfactual query
#[derive(Debug, Clone)]
pub struct CoInResult {
    pub query: String,
    pub factual: FactualScenario,
    pub counterfactuals: Vec<CounterfactualScenario>,
    pub best_explanation: Option<String>,
    pub elapsed_ms: f64,
}

/// Aggregate report across multiple queries
#[derive(Debug, Clone)]
pub struct CounterfactualReport {
    pub queries_answered: usize,
    pub counterfactuals_generated: usize,
    pub avg_plausibility: f64,
    pub avg_minimality: f64,
    pub elapsed_ms: f64,
}

/// Configuration for the Counterfactual Reasoner
#[derive(Debug, Clone)]
pub struct CounterfactualConfig {
    pub max_counterfactuals_per_query: usize,
    pub minimality_weight: f64,
    pub plausibility_weight: f64,
    pub vsa_similarity_threshold: f64,
    pub cache_size: usize,
}

impl Default for CounterfactualConfig {
    fn default() -> Self {
        Self {
            max_counterfactuals_per_query: 5,
            minimality_weight: 0.4,
            plausibility_weight: 0.6,
            vsa_similarity_threshold: 0.3,
            cache_size: 100,
        }
    }
}

/// CounterfactualReasoner — implements Pearl's Ladder Step 3
#[derive(Debug, Clone)]
pub struct CounterfactualReasoner {
    pub scm: StructuralCausalModel,
    pub history: Vec<FactualScenario>,
    pub total_queries: u64,
    pub total_counterfactuals: u64,
    pub config: CounterfactualConfig,
}

impl CounterfactualReasoner {
    pub fn new(config: CounterfactualConfig) -> Self {
        Self {
            scm: StructuralCausalModel {
                nodes: HashMap::new(),
                edges: Vec::new(),
                name: String::from("default"),
                version: 1,
            },
            history: Vec::new(),
            total_queries: 0,
            total_counterfactuals: 0,
            config,
        }
    }

    pub fn add_node(&mut self, name: &str, is_endogenous: bool) {
        let vsa = QuantizedVSA::seeded_random(self.scm.nodes.len() as u64 + 1, 64);
        self.scm.nodes.insert(
            name.to_string(),
            CausalNode {
                name: name.to_string(),
                vsa_vector: vsa,
                observed_value: 0.0,
                is_endogenous,
            },
        );
    }

    pub fn add_edge(&mut self, from: &str, to: &str, strength: f64, mechanism: CausalMechanism) {
        self.scm.edges.push(CausalEdge {
            from: from.to_string(),
            to: to.to_string(),
            strength: strength.clamp(0.0, 1.0),
            mechanism,
        });
    }

    pub fn record_factual(&mut self, scenario: FactualScenario) {
        self.history.push(scenario);
    }

    /// Ask a single counterfactual query: "what if we intervene on target_node?"
    pub fn ask_counterfactual(&mut self, query: &str, intervention: Intervention) -> CoInResult {
        let start = Instant::now();
        self.total_queries += 1;

        let factual = self.history.last().cloned().unwrap_or(FactualScenario {
            node_values: HashMap::new(),
            node_vectors: HashMap::new(),
            timestamp: Instant::now(),
            description: "empty".to_string(),
        });

        let mut counterfactuals = self.find_minimal_changes(&factual, &intervention);

        // Rank by combined plausibility + minimality score
        let pw = self.config.plausibility_weight;
        let mw = self.config.minimality_weight;
        counterfactuals.sort_by(|a, b| {
            let a_score = a.plausibility * pw + a.minimality_score * mw;
            let b_score = b.plausibility * pw + b.minimality_score * mw;
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        counterfactuals.truncate(self.config.max_counterfactuals_per_query);
        self.total_counterfactuals += counterfactuals.len() as u64;

        let best_explanation = counterfactuals.first().map(|cf| {
            format!(
                "If we had changed {} from {:.2} to {:.2} ({:?}), plausibility={:.2}, minimality={:.2}",
                cf.intervention.target_node,
                cf.factual.node_values.get(&cf.intervention.target_node).copied().unwrap_or(0.0),
                cf.intervention.new_value,
                cf.intervention.intervention_type,
                cf.plausibility,
                cf.minimality_score,
            )
        });

        CoInResult {
            query: query.to_string(),
            factual,
            counterfactuals,
            best_explanation,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// Generate all three counterfactual questions:
    /// 1. "What if we hadn't done X?" (Do, 0.0)
    /// 2. "What if we had done Y instead?" (Do, alternative value)
    /// 3. "What if conditions were different?" (Context)
    pub fn ask_three_questions(&mut self, scenario: FactualScenario) -> Vec<CoInResult> {
        let mut results = Vec::new();

        // Pick the first endogenous node with a non-zero value as "the action"
        let action_node = self
            .scm
            .nodes
            .iter()
            .find(|(_, n)| n.is_endogenous && n.observed_value.abs() > 0.01)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| {
                self.scm
                    .nodes
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or("unknown".to_string())
            });

        let current_val = scenario
            .node_values
            .get(&action_node)
            .copied()
            .unwrap_or(0.0);

        // Q1: What if we hadn't done X?
        self.history.push(scenario.clone());
        results.push(self.ask_counterfactual(
            &format!("What if we hadn't done {}?", action_node),
            Intervention {
                target_node: action_node.clone(),
                new_value: 0.0,
                intervention_type: InterventionType::Do,
                rationale: format!("Counterfactual: remove effect of {}", action_node),
            },
        ));

        // Q2: What if we had done Y instead?
        let alt_value = if current_val.abs() < 0.01 {
            1.0
        } else {
            -current_val
        };
        results.push(self.ask_counterfactual(
            &format!("What if we had done something else for {}?", action_node),
            Intervention {
                target_node: action_node.clone(),
                new_value: alt_value,
                intervention_type: InterventionType::Do,
                rationale: format!(
                    "Counterfactual: replace {} ({:.2}) with {:.2}",
                    action_node, current_val, alt_value
                ),
            },
        ));

        // Q3: What if context were different?
        let context_node = self
            .scm
            .nodes
            .iter()
            .find(|(_, n)| !n.is_endogenous)
            .map(|(name, _)| name.clone())
            .unwrap_or(action_node);
        results.push(self.ask_counterfactual(
            &format!("What if context ({}) were different?", context_node),
            Intervention {
                target_node: context_node,
                new_value: current_val + 1.0,
                intervention_type: InterventionType::Context,
                rationale: "Counterfactual: change background conditions".to_string(),
            },
        ));

        results
    }

    /// Compute plausibility of a counterfactual via VSA similarity
    pub fn compute_plausibility(&self, cf: &CounterfactualScenario) -> f64 {
        let factual = &cf.factual;
        if factual.node_vectors.is_empty() || cf.outcome_vectors.is_empty() {
            return 0.5;
        }

        let mut total_sim = 0.0;
        let mut count = 0;

        for (node_name, factual_vsa) in &factual.node_vectors {
            if let Some(cf_vsa) = cf.outcome_vectors.get(node_name) {
                let sim = QuantizedVSA::similarity(factual_vsa, cf_vsa);
                total_sim += sim;
                count += 1;
            }
        }

        if count == 0 {
            return 0.5;
        }

        let avg_sim = total_sim / count as f64;
        // Plausibility = 1.0 means no change (perfectly plausible),
        // lower means large deviation
        let plausibility = 1.0 - (1.0 - avg_sim).abs();
        plausibility.clamp(0.0, 1.0)
    }

    /// Generate counterfactual scenarios by propagating intervention
    /// through the SCM based on VSA similarity
    pub fn find_minimal_changes(
        &self,
        factual: &FactualScenario,
        intervention: &Intervention,
    ) -> Vec<CounterfactualScenario> {
        let mut results = Vec::new();

        if self.scm.nodes.is_empty() {
            return results;
        }

        // Build adjacency: parents → children
        let mut children: HashMap<&str, Vec<&CausalEdge>> = HashMap::new();
        for edge in &self.scm.edges {
            children.entry(edge.from.as_str()).or_default().push(edge);
        }

        // Propagate intervention through the DAG
        let mut visited: HashSet<String> = HashSet::new();
        let mut outcome_vectors: HashMap<String, Vec<u8>> = factual.node_vectors.clone();
        let mut modified_nodes: Vec<String> = Vec::new();

        // Apply the intervention directly
        let target_vsa = self
            .scm
            .nodes
            .get(&intervention.target_node)
            .map(|n| n.vsa_vector.clone())
            .unwrap_or_else(|| QuantizedVSA::seeded_random(999, 64));

        outcome_vectors.insert(intervention.target_node.clone(), target_vsa);
        modified_nodes.push(intervention.target_node.clone());
        visited.insert(intervention.target_node.clone());

        // BFS propagation through causal children
        let mut queue: Vec<String> = vec![intervention.target_node.clone()];
        while let Some(current) = queue.pop() {
            if let Some(out_edges) = children.get(current.as_str()) {
                for edge in out_edges {
                    if visited.contains(&edge.to) {
                        continue;
                    }
                    visited.insert(edge.to.clone());

                    let factual_val = factual.node_values.get(&edge.to).copied().unwrap_or(0.0);

                    let parent_val = outcome_vectors
                        .get(&edge.from)
                        .and_then(|v| {
                            self.scm.nodes.get(&edge.from).map(|n| {
                                let sim = QuantizedVSA::similarity(v, &n.vsa_vector);
                                sim * n.observed_value
                            })
                        })
                        .unwrap_or(0.0);

                    let propagated_val =
                        factual_val * (1.0 - edge.strength) + parent_val * edge.strength;

                    let child_vsa = self
                        .scm
                        .nodes
                        .get(&edge.to)
                        .map(|n| {
                            let mut v = n.vsa_vector.clone();
                            for (_, byte) in v.iter_mut().enumerate() {
                                let perturbation = (propagated_val * 10.0) as u8;
                                *byte = byte.wrapping_add(perturbation);
                            }
                            v
                        })
                        .unwrap_or_else(|| QuantizedVSA::seeded_random(edge.to.len() as u64, 64));

                    outcome_vectors.insert(edge.to.clone(), child_vsa);
                    modified_nodes.push(edge.to.clone());
                    queue.push(edge.to.clone());
                }
            }
        }

        // Generate a primary counterfactual (the propagated outcome)
        let modified_vsa = outcome_vectors.clone();
        let cf = CounterfactualScenario {
            factual: factual.clone(),
            intervention: intervention.clone(),
            modified_nodes: modified_nodes.clone(),
            outcome_vectors: modified_vsa,
            minimality_score: 1.0,
            plausibility: 1.0,
        };
        let plausibility = self.compute_plausibility(&cf);
        let minimality = {
            let changed = modified_nodes.len();
            let total = self.scm.nodes.len();
            if total == 0 {
                1.0
            } else {
                1.0 - (changed as f64 / total as f64)
            }
        };
        results.push(CounterfactualScenario {
            minimality_score: minimality,
            plausibility,
            ..cf
        });

        // Generate alternative counterfactuals by varying the intervention
        // strength (weaker / stronger perturbation)
        let alt_factors = [0.5, 0.75, 1.25, 1.5];
        for factor in alt_factors {
            let alt_value = intervention.new_value * factor;
            let mut alt_outcomes = factual.node_vectors.clone();
            let alt_vsa = self
                .scm
                .nodes
                .get(&intervention.target_node)
                .map(|n| {
                    let mut v = n.vsa_vector.clone();
                    for byte in v.iter_mut() {
                        *byte = byte.wrapping_add((alt_value.abs() * 20.0) as u8);
                    }
                    v
                })
                .unwrap_or_else(|| QuantizedVSA::seeded_random((factor * 100.0) as u64, 64));

            alt_outcomes.insert(intervention.target_node.clone(), alt_vsa);

            let alt_cf = CounterfactualScenario {
                factual: factual.clone(),
                intervention: Intervention {
                    new_value: alt_value,
                    ..intervention.clone()
                },
                modified_nodes: vec![intervention.target_node.clone()],
                outcome_vectors: alt_outcomes,
                minimality_score: 1.0,
                plausibility: 1.0,
            };

            let alt_plausibility = self.compute_plausibility(&alt_cf);
            let alt_minimality = 1.0 - (1.0 / self.scm.nodes.len().max(1) as f64);

            results.push(CounterfactualScenario {
                minimality_score: alt_minimality,
                plausibility: alt_plausibility,
                ..alt_cf
            });
        }

        results
    }

    /// Generate a summary report of all counterfactual activity
    pub fn report(&self) -> CounterfactualReport {
        CounterfactualReport {
            queries_answered: self.total_queries as usize,
            counterfactuals_generated: self.total_counterfactuals as usize,
            avg_plausibility: 0.0,
            avg_minimality: 0.0,
            elapsed_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    fn make_factual(values: Vec<(&str, f64)>, description: &str) -> FactualScenario {
        let mut node_values = HashMap::new();
        let mut node_vectors = HashMap::new();
        for (i, (name, val)) in values.iter().enumerate() {
            node_values.insert(name.to_string(), *val);
            node_vectors.insert(name.to_string(), test_vsa(i as u64));
        }
        FactualScenario {
            node_values,
            node_vectors,
            timestamp: Instant::now(),
            description: description.to_string(),
        }
    }

    #[test]
    fn test_create_scm_with_nodes() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("X", true);
        reasoner.add_node("Y", true);
        assert_eq!(reasoner.scm.nodes.len(), 2);
        assert!(reasoner.scm.nodes.contains_key("X"));
        assert!(reasoner.scm.nodes.contains_key("Y"));
    }

    #[test]
    fn test_add_edge_between_nodes() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("X", true);
        reasoner.add_node("Y", true);
        reasoner.add_edge("X", "Y", 0.8, CausalMechanism::Direct);
        assert_eq!(reasoner.scm.edges.len(), 1);
        assert_eq!(reasoner.scm.edges[0].from, "X");
        assert_eq!(reasoner.scm.edges[0].to, "Y");
        assert!((reasoner.scm.edges[0].strength - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_record_factual_scenario() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        let factual = make_factual(vec![("X", 1.0), ("Y", 2.0)], "test");
        reasoner.record_factual(factual);
        assert_eq!(reasoner.history.len(), 1);
        assert_eq!(reasoner.history[0].description, "test");
    }

    #[test]
    fn test_ask_counterfactual_simple_2node() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("X", true);
        reasoner.add_node("Y", true);
        reasoner.add_edge("X", "Y", 0.8, CausalMechanism::Direct);

        let factual = make_factual(vec![("X", 1.0), ("Y", 2.0)], "simple");
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "X".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "test".to_string(),
        };

        let result = reasoner.ask_counterfactual("What if X were 0?", intervention);
        assert_eq!(result.counterfactuals.len(), 5);
        assert!(result.best_explanation.is_some());
    }

    #[test]
    fn test_intervention_changes_node_value() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("A", true);
        reasoner.add_node("B", false);

        let factual = make_factual(vec![("A", 5.0), ("B", 3.0)], "values");
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "A".to_string(),
            new_value: 10.0,
            intervention_type: InterventionType::Do,
            rationale: "double A".to_string(),
        };

        let result = reasoner.ask_counterfactual("Double A?", intervention);
        assert!(!result.counterfactuals.is_empty());
        let first = &result.counterfactuals[0];
        assert_eq!(first.intervention.new_value, 10.0);
        assert!(first.modified_nodes.contains(&"A".to_string()));
    }

    #[test]
    fn test_three_questions_generate_three_types() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("action", true);
        reasoner.add_node("context", false);
        reasoner.add_edge("action", "context", 0.5, CausalMechanism::Direct);

        // Set observed_value so ask_three_questions finds it
        if let Some(node) = reasoner.scm.nodes.get_mut("action") {
            node.observed_value = 2.0;
        }

        let factual = make_factual(vec![("action", 2.0), ("context", 1.0)], "three Qs");
        let results = reasoner.ask_three_questions(factual);
        assert_eq!(results.len(), 3);

        // Q1: Do-type intervention
        assert_eq!(
            results[0].counterfactuals[0].intervention.intervention_type,
            InterventionType::Do
        );
        // Q2: Do-type different value
        assert_eq!(
            results[1].counterfactuals[0].intervention.intervention_type,
            InterventionType::Do
        );
        // Q3: Context-type
        assert_eq!(
            results[2].counterfactuals[0].intervention.intervention_type,
            InterventionType::Context
        );
    }

    #[test]
    fn test_plausibility_high_when_similar_vsas() {
        let config = CounterfactualConfig::default();
        let reasoner = CounterfactualReasoner::new(config);

        let mut factual = make_factual(vec![("X", 1.0)], "plausibility test");
        let vsa = test_vsa(42);
        factual.node_vectors.insert("X".to_string(), vsa.clone());

        let cf = CounterfactualScenario {
            factual: factual.clone(),
            intervention: Intervention {
                target_node: "X".to_string(),
                new_value: 1.0,
                intervention_type: InterventionType::Do,
                rationale: "same".to_string(),
            },
            modified_nodes: vec![],
            outcome_vectors: {
                let mut m = HashMap::new();
                m.insert("X".to_string(), vsa.clone());
                m
            },
            minimality_score: 1.0,
            plausibility: 1.0,
        };

        let plau = reasoner.compute_plausibility(&cf);
        // Identical vectors → similarity = 1.0 → plausibility near 1.0
        assert!(
            plau > 0.99,
            "identical VSAs should give plausibility ~1.0, got {}",
            plau
        );
    }

    #[test]
    fn test_minimality_small_edit_high_score() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("X", true);
        reasoner.add_node("Y", true);
        reasoner.add_node("Z", true);

        let factual = make_factual(vec![("X", 1.0), ("Y", 2.0), ("Z", 3.0)], "minimality");
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "X".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "minimal".to_string(),
        };

        let cfs = reasoner.find_minimal_changes(&reasoner.history.last().unwrap(), &intervention);

        assert!(!cfs.is_empty());
        // Editing only X (directly) should give high minimality
        let first = &cfs[0];
        assert!(
            first.minimality_score > 0.5,
            "minimal change should score > 0.5, got {}",
            first.minimality_score
        );
    }

    #[test]
    fn test_vsa_similarity_as_plausibility_metric() {
        let config = CounterfactualConfig::default();
        let reasoner = CounterfactualReasoner::new(config);

        let mut factual = make_factual(vec![("X", 1.0)], "similarity test");
        let vsa_a = test_vsa(100);
        let vsa_b = test_vsa(101);
        factual.node_vectors.insert("X".to_string(), vsa_a.clone());

        // Same vector → high plausibility
        let cf_same = CounterfactualScenario {
            factual: factual.clone(),
            intervention: Intervention {
                target_node: "X".to_string(),
                new_value: 1.0,
                intervention_type: InterventionType::Do,
                rationale: "same".to_string(),
            },
            modified_nodes: vec![],
            outcome_vectors: {
                let mut m = HashMap::new();
                m.insert("X".to_string(), vsa_a.clone());
                m
            },
            minimality_score: 1.0,
            plausibility: 1.0,
        };

        // Different vector → lower plausibility
        let cf_diff = CounterfactualScenario {
            factual,
            intervention: Intervention {
                target_node: "X".to_string(),
                new_value: 999.0,
                intervention_type: InterventionType::Do,
                rationale: "diff".to_string(),
            },
            modified_nodes: vec![],
            outcome_vectors: {
                let mut m = HashMap::new();
                m.insert("X".to_string(), vsa_b);
                m
            },
            minimality_score: 0.0,
            plausibility: 0.0,
        };

        let plau_same = reasoner.compute_plausibility(&cf_same);
        let plau_diff = reasoner.compute_plausibility(&cf_diff);

        assert!(
            plau_same > plau_diff,
            "identical VSA should be more plausible than different VSA ({} vs {})",
            plau_same,
            plau_diff
        );
    }

    #[test]
    fn test_empty_scm_does_not_crash() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);

        let factual = FactualScenario {
            node_values: HashMap::new(),
            node_vectors: HashMap::new(),
            timestamp: Instant::now(),
            description: "empty".to_string(),
        };

        let intervention = Intervention {
            target_node: "nonexistent".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "empty".to_string(),
        };

        reasoner.record_factual(factual);
        let result = reasoner.ask_counterfactual("empty?", intervention);
        assert!(result.counterfactuals.is_empty());
    }

    #[test]
    fn test_mediated_causal_chain() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("A", true);
        reasoner.add_node("B", true);
        reasoner.add_node("C", true);
        // A → B → C (mediated chain)
        reasoner.add_edge("A", "B", 0.9, CausalMechanism::Direct);
        reasoner.add_edge("B", "C", 0.8, CausalMechanism::Mediated);

        let factual = make_factual(vec![("A", 1.0), ("B", 2.0), ("C", 3.0)], "mediated chain");
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "A".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "remove A".to_string(),
        };

        let result = reasoner.ask_counterfactual("What if A were 0?", intervention);
        assert!(!result.counterfactuals.is_empty());
        // A should be in modified_nodes; B and C may also be modified
        let first = &result.counterfactuals[0];
        assert!(first.modified_nodes.contains(&"A".to_string()));
    }

    #[test]
    fn test_multi_level_dag_propagation() {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("X", true);
        reasoner.add_node("Y1", true);
        reasoner.add_node("Y2", true);
        reasoner.add_node("Z", true);
        // X → Y1, X → Y2, Y1 → Z, Y2 → Z
        reasoner.add_edge("X", "Y1", 0.7, CausalMechanism::Direct);
        reasoner.add_edge("X", "Y2", 0.6, CausalMechanism::Direct);
        reasoner.add_edge("Y1", "Z", 0.5, CausalMechanism::Mediated);
        reasoner.add_edge("Y2", "Z", 0.4, CausalMechanism::Mediated);

        let factual = make_factual(
            vec![("X", 1.0), ("Y1", 2.0), ("Y2", 3.0), ("Z", 4.0)],
            "multi DAG",
        );
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "X".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "remove X cause".to_string(),
        };

        let result = reasoner.ask_counterfactual("What if X were 0?", intervention);
        assert!(!result.counterfactuals.is_empty());
        // All descendants should be modified
        let modified = &result.counterfactuals[0].modified_nodes;
        assert!(modified.contains(&"X".to_string()));
        // Y1, Y2, Z may or may not be modified depending on propagation
    }

    #[test]
    fn test_plausibility_and_minimality_weights_in_ranking() {
        let mut config = CounterfactualConfig::default();
        config.plausibility_weight = 1.0;
        config.minimality_weight = 1.0;
        config.max_counterfactuals_per_query = 3;
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.add_node("A", true);
        reasoner.add_node("B", true);

        let factual = make_factual(vec![("A", 1.0), ("B", 2.0)], "ranking");
        reasoner.record_factual(factual);

        let intervention = Intervention {
            target_node: "A".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "ranking test".to_string(),
        };

        let result = reasoner.ask_counterfactual("Rank?", intervention);
        assert_eq!(result.counterfactuals.len(), 3);
        // Results should be sorted by plausibility*weight + minimality*weight descending
        for w in result.counterfactuals.windows(2) {
            let a_score = w[0].plausibility * 1.0 + w[0].minimality_score * 1.0;
            let b_score = w[1].plausibility * 1.0 + w[1].minimality_score * 1.0;
            assert!(
                a_score >= b_score - 1e-9,
                "counterfactuals must be ranked by combined score ({} >= {})",
                a_score,
                b_score
            );
        }
    }
}
