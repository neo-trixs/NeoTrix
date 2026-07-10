#![forbid(unsafe_code)]

pub mod annotation;
pub mod fuzzy;
pub mod inference;

use serde::{Deserialize, Serialize};

pub use self::annotation::{FidelityAnnotation, FidelityLevel};
pub use self::fuzzy::FuzzyOperator;
pub use self::inference::{InferenceEngine, NesyFact, NesyRule, NesyValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NesyInference {
    pub step: u32,
    pub rule_applied: String,
    pub derived_fact: NesyFact,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NesyStats {
    pub facts: usize,
    pub rules: usize,
    pub inferences: u64,
    pub avg_confidence: f64,
}

pub struct NeuroSymbolicEngine {
    pub inference: InferenceEngine,
    #[allow(dead_code)]
    max_inferences: u64,
    inference_count: u64,
}

impl NeuroSymbolicEngine {
    pub fn new(max_facts: usize) -> Self {
        Self {
            inference: InferenceEngine::new(max_facts),
            max_inferences: 10000,
            inference_count: 0,
        }
    }

    pub fn add_fact(&mut self, fact: NesyFact) {
        self.inference.add_fact(fact);
    }

    pub fn add_rule(&mut self, rule: NesyRule) {
        self.inference.add_rule(rule);
    }

    pub fn infer(&self, query: &NesyFact, max_depth: usize) -> Vec<(NesyFact, f64, Vec<NesyInference>)> {
        let results = self.inference.resolve(query, max_depth);
        results
            .into_iter()
            .enumerate()
            .map(|(i, (fact, conf))| {
                let trace = vec![NesyInference {
                    step: i as u32,
                    rule_applied: format!("resolve:{}", fact.predicate),
                    derived_fact: fact.clone(),
                    confidence: conf,
                }];
                (fact, conf, trace)
            })
            .collect()
    }

    pub fn forward_chain(&self, max_steps: usize) -> Vec<NesyFact> {
        self.inference.saturate(max_steps)
    }

    pub fn backward_chain(&self, query: &NesyFact, max_depth: usize) -> Vec<(NesyFact, f64)> {
        self.inference.resolve(query, max_depth)
    }

    pub fn fuzzy_match(&self, a: &NesyValue, b: &NesyValue) -> f64 {
        match (a, b) {
            (NesyValue::Number(na), NesyValue::Number(nb)) => {
                let diff = (na - nb).abs();
                (1.0 - diff).max(0.0).min(1.0)
            }
            (NesyValue::Symbol(sa), NesyValue::Symbol(sb)) => {
                if sa == sb { 1.0 } else { FuzzyOperator::not(0.5) }
            }
            (NesyValue::Bool(ba), NesyValue::Bool(bb)) => {
                if ba == bb { 1.0 } else { 0.0 }
            }
            (NesyValue::Grounded(ga), NesyValue::Grounded(gb)) => {
                if ga == gb { 1.0 } else { 0.0 }
            }
            (NesyValue::List(la), NesyValue::List(lb)) => {
                if la.len() != lb.len() {
                    return 0.0;
                }
                let sim: f64 = la.iter().zip(lb.iter()).map(|(x, y)| self.fuzzy_match(x, y)).sum();
                sim / la.len() as f64
            }
            _ => 0.0,
        }
    }

    pub fn engine_stats(&self) -> NesyStats {
        let total_conf: f64 = self.inference.facts.iter().map(|f| f.confidence).sum();
        let avg = if self.inference.facts.is_empty() {
            0.0
        } else {
            total_conf / self.inference.facts.len() as f64
        };
        NesyStats {
            facts: self.inference.facts.len(),
            rules: self.inference.rules.len(),
            inferences: self.inference_count,
            avg_confidence: avg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(predicate: &str, args: Vec<NesyValue>, confidence: f64) -> NesyFact {
        NesyFact { predicate: predicate.to_string(), args, confidence }
    }

    fn sym(s: &str) -> NesyValue { NesyValue::Symbol(s.to_string()) }
    fn num(n: f64) -> NesyValue { NesyValue::Number(n) }
    fn var(v: &str) -> NesyValue { NesyValue::Variable(v.to_string()) }

    #[test]
    fn test_full_pipeline() {
        let mut engine = NeuroSymbolicEngine::new(100);
        engine.add_fact(fact("human", vec![sym("socrates")], 1.0));
        engine.add_rule(NesyRule {
            head: fact("mortal", vec![var("x")], 1.0),
            body: vec![fact("human", vec![var("x")], 1.0)],
            weight: 1.0,
        });
        let results = engine.backward_chain(&fact("mortal", vec![var("x")], 1.0), 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].0.predicate, "mortal");
    }

    #[test]
    fn test_infer_tracking() {
        let mut engine = NeuroSymbolicEngine::new(100);
        engine.add_fact(fact("a", vec![num(1.0)], 1.0));
        engine.add_rule(NesyRule {
            head: fact("b", vec![var("x")], 1.0),
            body: vec![fact("a", vec![var("x")], 1.0)],
            weight: 0.8,
        });
        let results = engine.infer(&fact("b", vec![num(1.0)], 1.0), 3);
        assert!(!results.is_empty());
        let (_fact, _conf, traces) = &results[0];
        assert!(!traces.is_empty());
        assert_eq!(traces[0].rule_applied, "resolve:b");
    }

    #[test]
    fn test_forward_chain_integration() {
        let mut engine = NeuroSymbolicEngine::new(100);
        engine.add_fact(fact("p", vec![num(1.0)], 1.0));
        engine.add_rule(NesyRule {
            head: fact("q", vec![var("x")], 1.0),
            body: vec![fact("p", vec![var("x")], 1.0)],
            weight: 1.0,
        });
        let derived = engine.forward_chain(10);
        assert!(derived.iter().any(|f| f.predicate == "q"));
    }

    #[test]
    fn test_fuzzy_match_numbers() {
        let engine = NeuroSymbolicEngine::new(10);
        let sim = engine.fuzzy_match(&num(0.5), &num(0.5));
        assert!((sim - 1.0).abs() < 1e-9);
        let sim2 = engine.fuzzy_match(&num(0.5), &num(1.0));
        assert!((sim2 - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_fuzzy_match_symbols() {
        let engine = NeuroSymbolicEngine::new(10);
        let sim = engine.fuzzy_match(&sym("hello"), &sym("hello"));
        assert!((sim - 1.0).abs() < 1e-9);
        let sim2 = engine.fuzzy_match(&sym("hello"), &sym("world"));
        assert!((sim2 - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_engine_stats() {
        let mut engine = NeuroSymbolicEngine::new(100);
        engine.add_fact(fact("a", vec![], 1.0));
        engine.add_fact(fact("b", vec![], 0.5));
        engine.add_rule(NesyRule {
            head: fact("c", vec![], 1.0),
            body: vec![],
            weight: 1.0,
        });
        let stats = engine.engine_stats();
        assert_eq!(stats.facts, 2);
        assert_eq!(stats.rules, 1);
        assert!((stats.avg_confidence - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_empty_stats() {
        let engine = NeuroSymbolicEngine::new(10);
        let stats = engine.engine_stats();
        assert_eq!(stats.facts, 0);
        assert_eq!(stats.rules, 0);
        assert_eq!(stats.inferences, 0);
    }

    #[test]
    fn test_fuzzy_match_mismatched_types() {
        let engine = NeuroSymbolicEngine::new(10);
        let sim = engine.fuzzy_match(&num(1.0), &sym("x"));
        assert!((sim - 0.0).abs() < 1e-9);
    }
}
