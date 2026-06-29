use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MutationStrategy {
    Refactor,
    Optimize,
    Generalize,
    Specialize,
    Compose,
}

impl MutationStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            MutationStrategy::Refactor => "refactor",
            MutationStrategy::Optimize => "optimize",
            MutationStrategy::Generalize => "generalize",
            MutationStrategy::Specialize => "specialize",
            MutationStrategy::Compose => "compose",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMutation {
    pub id: u64,
    pub strategy: MutationStrategy,
    pub source: String,
    pub target: String,
    pub confidence: f64,
    pub evaluator_score: f64,
    pub cycle_applied: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorFeedback {
    pub compile_success: bool,
    pub test_pass_rate: f64,
    pub complexity_delta: f64,
    pub performance_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMutationEngine {
    pub mutations: Vec<CodeMutation>,
    pub cycle: u64,
    pub max_history: usize,
    pub min_confidence_threshold: f64,
    pub strategy_success_rates: HashMap<MutationStrategy, f64>,
    pub strategy_counts: HashMap<MutationStrategy, u64>,
}

impl CodeMutationEngine {
    pub fn new() -> Self {
        let mut strategy_success_rates = HashMap::new();
        for s in &[
            MutationStrategy::Refactor,
            MutationStrategy::Optimize,
            MutationStrategy::Generalize,
            MutationStrategy::Specialize,
            MutationStrategy::Compose,
        ] {
            strategy_success_rates.insert(*s, 0.5);
        }
        let mut strategy_counts = HashMap::new();
        for s in &[
            MutationStrategy::Refactor,
            MutationStrategy::Optimize,
            MutationStrategy::Generalize,
            MutationStrategy::Specialize,
            MutationStrategy::Compose,
        ] {
            strategy_counts.insert(*s, 0);
        }
        Self {
            mutations: Vec::new(),
            cycle: 0,
            max_history: 100,
            min_confidence_threshold: 0.3,
            strategy_success_rates,
            strategy_counts,
        }
    }

    pub fn propose_mutation(&self, source: &str, strategy: MutationStrategy) -> Option<String> {
        if self
            .strategy_success_rates
            .get(&strategy)
            .copied()
            .unwrap_or(0.5)
            < self.min_confidence_threshold
        {
            return None;
        }
        let target = match strategy {
            MutationStrategy::Refactor => {
                if source.contains("if let") {
                    Some(source.replace("if let", "match"))
                } else if source.contains("match") {
                    Some(source.replace("match", "if let"))
                } else if source.contains("unwrap(") {
                    Some(source.replace("unwrap(", "unwrap_or_default("))
                } else {
                    None
                }
            }
            MutationStrategy::Optimize => {
                if source.contains("clone()") {
                    Some(source.replace("clone()", ""))
                } else if source.contains("collect::<Vec<_>>()") {
                    Some(source.replace("collect::<Vec<_>>()", "collect::<VecDeque<_>>()"))
                } else {
                    None
                }
            }
            MutationStrategy::Generalize => {
                if source.contains("String") {
                    Some(source.replace("String", "impl Into<String>"))
                } else if source.contains("Vec<") {
                    Some(source.replace("Vec<", "impl IntoIterator<Item="))
                } else {
                    None
                }
            }
            MutationStrategy::Specialize => {
                if source.contains("impl Into<String>") {
                    Some(source.replace("impl Into<String>", "String"))
                } else if source.contains("impl IntoIterator") {
                    Some(source.replace("impl IntoIterator<Item=", "Vec<"))
                } else {
                    None
                }
            }
            MutationStrategy::Compose => {
                if source.contains("map(") && source.contains("filter(") {
                    Some(format!(
                        "{}.filter_map(|x| x)",
                        source.split('.').next().unwrap_or("")
                    ))
                } else {
                    None
                }
            }
        };
        target
    }

    pub fn apply_mutation(&mut self, mutation: CodeMutation) -> String {
        self.cycle += 1;
        let mut m = mutation;
        m.id = self.mutations.len() as u64 + 1;
        m.cycle_applied = self.cycle;
        *self.strategy_counts.entry(m.strategy).or_insert(0) += 1;
        self.mutations.push(m.clone());
        if self.mutations.len() > self.max_history {
            self.mutations.remove(0);
        }
        m.target
    }

    pub fn record_evaluation(&mut self, strategy: MutationStrategy, feedback: &EvaluatorFeedback) {
        let rate = self.strategy_success_rates.entry(strategy).or_insert(0.5);
        let score = if feedback.compile_success {
            feedback.test_pass_rate * 0.6
                + (1.0 - feedback.complexity_delta * 0.2)
                + (1.0 + feedback.performance_delta * 0.2)
        } else {
            0.0
        };
        *rate = *rate * 0.7 + score * 0.3;
    }

    pub fn best_strategy(&self) -> MutationStrategy {
        self.strategy_success_rates
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, _)| *s)
            .unwrap_or(MutationStrategy::Refactor)
    }

    pub fn mutation_count(&self) -> usize {
        self.mutations.len()
    }

    pub fn success_rate_by_strategy(&self, strategy: MutationStrategy) -> f64 {
        self.strategy_success_rates
            .get(&strategy)
            .copied()
            .unwrap_or(0.5)
    }

    pub fn diagnostic(&self) -> String {
        let best = self.best_strategy();
        let rates: Vec<String> = self
            .strategy_success_rates
            .iter()
            .map(|(s, r)| format!("{}:{:.2}", s.name(), r))
            .collect();
        format!(
            "code_mut:total={}|best={}|rates=[{}]",
            self.mutations.len(),
            best.name(),
            rates.join(","),
        )
    }
}
