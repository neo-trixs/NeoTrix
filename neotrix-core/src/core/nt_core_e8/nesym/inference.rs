#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NesyValue {
    Symbol(String),
    Number(f64),
    Bool(bool),
    List(Vec<NesyValue>),
    Variable(String),
    Grounded(usize),
}

impl NesyValue {
    pub fn to_string(&self) -> String {
        match self {
            NesyValue::Symbol(s) => s.clone(),
            NesyValue::Number(n) => format!("{:.4}", n),
            NesyValue::Bool(b) => b.to_string(),
            NesyValue::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                format!("[{}]", inner.join(", "))
            }
            NesyValue::Variable(v) => format!("?{}", v),
            NesyValue::Grounded(id) => format!("#{}", id),
        }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, NesyValue::Variable(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NesyFact {
    pub predicate: String,
    pub args: Vec<NesyValue>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NesyRule {
    pub head: NesyFact,
    pub body: Vec<NesyFact>,
    pub weight: f64,
}

pub struct InferenceEngine {
    pub facts: Vec<NesyFact>,
    pub rules: Vec<NesyRule>,
    pub max_facts: usize,
}

impl InferenceEngine {
    pub fn new(max_facts: usize) -> Self {
        Self {
            facts: Vec::with_capacity(max_facts),
            rules: Vec::new(),
            max_facts,
        }
    }

    pub fn add_fact(&mut self, fact: NesyFact) {
        if self.facts.len() < self.max_facts {
            self.facts.push(fact);
        }
    }

    pub fn add_rule(&mut self, rule: NesyRule) {
        self.rules.push(rule);
    }

    pub fn unify(&self, a: &NesyFact, b: &NesyFact) -> Option<HashMap<String, NesyValue>> {
        if a.predicate != b.predicate || a.args.len() != b.args.len() {
            return None;
        }
        let mut bindings = HashMap::new();
        for (arg_a, arg_b) in a.args.iter().zip(b.args.iter()) {
            match (arg_a, arg_b) {
                (NesyValue::Variable(var), _) => {
                    bindings.insert(var.clone(), arg_b.clone());
                }
                (_, NesyValue::Variable(_)) => {
                    // If 'b' has a variable, we don't bind in this direction
                    // per spec: "Variable in 'a' binds to corresponding arg in 'b'"
                }
                (a_val, b_val) => {
                    if !values_equal(a_val, b_val) {
                        return None;
                    }
                }
            }
        }
        Some(bindings)
    }

    pub fn resolve(&self, query: &NesyFact, depth: usize) -> Vec<(NesyFact, f64)> {
        if depth == 0 {
            return Vec::new();
        }
        let mut results = Vec::new();
        for fact in &self.facts {
            if fact.predicate == query.predicate && fact.args.len() == query.args.len() {
                if self.unify(query, fact).is_some() {
                    results.push((fact.clone(), fact.confidence));
                }
            }
        }
        for rule in &self.rules {
            if self.unify(query, &rule.head).is_some() {
                let mut combined = 1.0;
                let mut valid = true;
                for body_fact in &rule.body {
                    let sub_results = self.resolve(body_fact, depth - 1);
                    if sub_results.is_empty() {
                        valid = false;
                        break;
                    }
                    let best_conf = sub_results.iter().map(|(_, c)| c).cloned().fold(0.0, f64::max);
                    combined *= best_conf;
                }
                if valid {
                    results.push((rule.head.clone(), rule.weight * combined));
                }
            }
        }
        results
    }

    pub fn saturate(&self, max_steps: usize) -> Vec<NesyFact> {
        let mut derived = self.facts.clone();
        let mut new_facts: Vec<NesyFact> = Vec::new();
        for _step in 0..max_steps {
            let mut added = false;
            for rule in &self.rules {
                for fact in &derived.clone() {
                    if rule.body.len() == 1 {
                        if let Some(bindings) = self.unify(&rule.body[0], fact) {
                            let new_args: Vec<NesyValue> = rule.head.args.iter()
                                .map(|arg| match arg {
                                    NesyValue::Variable(var) => bindings.get(var)
                                        .cloned()
                                        .unwrap_or_else(|| NesyValue::Variable(var.clone())),
                                    other => other.clone(),
                                })
                                .collect();
                            let candidate = NesyFact {
                                predicate: rule.head.predicate.clone(),
                                args: new_args,
                                confidence: rule.weight * fact.confidence,
                            };
                            if !derived.iter().any(|f| {
                                f.predicate == candidate.predicate && f.args == candidate.args
                            }) && !new_facts.iter().any(|f| {
                                f.predicate == candidate.predicate && f.args == candidate.args
                            }) {
                                new_facts.push(candidate);
                                added = true;
                            }
                        }
                    }
                }
            }
            if !added {
                break;
            }
            derived.extend(new_facts.drain(..));
        }
        derived
    }
}

fn values_equal(a: &NesyValue, b: &NesyValue) -> bool {
    match (a, b) {
        (NesyValue::Symbol(sa), NesyValue::Symbol(sb)) => sa == sb,
        (NesyValue::Number(na), NesyValue::Number(nb)) => (na - nb).abs() < 1e-9,
        (NesyValue::Bool(ba), NesyValue::Bool(bb)) => ba == bb,
        (NesyValue::List(la), NesyValue::List(lb)) => {
            la.len() == lb.len() && la.iter().zip(lb.iter()).all(|(x, y)| values_equal(x, y))
        }
        (NesyValue::Grounded(ga), NesyValue::Grounded(gb)) => ga == gb,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact( predicate: &str, args: Vec<NesyValue>, confidence: f64) -> NesyFact {
        NesyFact { predicate: predicate.to_string(), args, confidence }
    }

    fn sym(s: &str) -> NesyValue { NesyValue::Symbol(s.to_string()) }
    fn num(n: f64) -> NesyValue { NesyValue::Number(n) }
    fn var(v: &str) -> NesyValue { NesyValue::Variable(v.to_string()) }

    #[test]
    fn test_fact_addition() {
        let mut engine = InferenceEngine::new(100);
        let f = fact("mortal", vec![sym("socrates")], 1.0);
        engine.add_fact(f);
        assert_eq!(engine.facts.len(), 1);
        assert_eq!(engine.facts[0].predicate, "mortal");
    }

    #[test]
    fn test_rule_addition() {
        let mut engine = InferenceEngine::new(100);
        let rule = NesyRule {
            head: fact("mortal", vec![var("x")], 1.0),
            body: vec![fact("human", vec![var("x")], 1.0)],
            weight: 1.0,
        };
        engine.add_rule(rule);
        assert_eq!(engine.rules.len(), 1);
    }

    #[test]
    fn test_unification_matching() {
        let engine = InferenceEngine::new(10);
        let a = fact("father", vec![sym("john"), sym("mary")], 1.0);
        let b = fact("father", vec![sym("john"), sym("mary")], 1.0);
        assert!(engine.unify(&a, &b).is_some());
    }

    #[test]
    fn test_unification_variable_binding() {
        let engine = InferenceEngine::new(10);
        let a = fact("father", vec![var("x"), sym("mary")], 1.0);
        let b = fact("father", vec![sym("john"), sym("mary")], 1.0);
        let bindings = engine.unify(&a, &b);
        assert!(bindings.is_some());
        let bmap = bindings.unwrap();
        assert_eq!(bmap.len(), 1);
        match &bmap["x"] {
            NesyValue::Symbol(s) => assert_eq!(s, "john"),
            _ => panic!("expected Symbol"),
        }
    }

    #[test]
    fn test_unification_fail() {
        let engine = InferenceEngine::new(10);
        let a = fact("father", vec![sym("john"), sym("mary")], 1.0);
        let b = fact("father", vec![sym("john"), sym("peter")], 1.0);
        assert!(engine.unify(&a, &b).is_none());
    }

    #[test]
    fn test_backward_chaining() {
        let mut engine = InferenceEngine::new(100);
        engine.add_fact(fact("human", vec![sym("socrates")], 1.0));
        engine.add_rule(NesyRule {
            head: fact("mortal", vec![var("x")], 1.0),
            body: vec![fact("human", vec![var("x")], 1.0)],
            weight: 1.0,
        });
        let results = engine.resolve(&fact("mortal", vec![var("x")], 1.0), 5);
        assert!(!results.is_empty());
        let found = results.iter().any(|(f, _)| {
            f.predicate == "mortal" && f.args.len() == 1
        });
        assert!(found);
    }

    #[test]
    fn test_backward_chaining_confidence() {
        let mut engine = InferenceEngine::new(100);
        engine.add_fact(fact("human", vec![sym("socrates")], 0.8));
        engine.add_rule(NesyRule {
            head: fact("mortal", vec![var("x")], 1.0),
            body: vec![fact("human", vec![var("x")], 1.0)],
            weight: 0.9,
        });
        let results = engine.resolve(&fact("mortal", vec![sym("socrates")], 1.0), 5);
        assert!(!results.is_empty());
        let (_, conf) = &results[0];
        assert!((*conf - 0.72).abs() < 1e-9);
    }

    #[test]
    fn test_forward_chaining() {
        let mut engine = InferenceEngine::new(100);
        engine.add_fact(fact("animal", vec![sym("socrates")], 1.0));
        engine.add_rule(NesyRule {
            head: fact("living", vec![var("x")], 1.0),
            body: vec![fact("animal", vec![var("x")], 1.0)],
            weight: 1.0,
        });
        let results = engine.saturate(10);
        let living = results.iter().any(|f| f.predicate == "living");
        assert!(living);
    }

    #[test]
    fn test_forward_chaining_multi_step() {
        let mut engine = InferenceEngine::new(100);
        engine.add_fact(fact("a", vec![num(1.0)], 1.0));
        engine.add_rule(NesyRule {
            head: fact("b", vec![var("x")], 1.0),
            body: vec![fact("a", vec![var("x")], 1.0)],
            weight: 0.9,
        });
        engine.add_rule(NesyRule {
            head: fact("c", vec![var("x")], 1.0),
            body: vec![fact("b", vec![var("x")], 1.0)],
            weight: 0.8,
        });
        let results = engine.saturate(10);
        assert!(results.iter().any(|f| f.predicate == "c"));
    }

    #[test]
    fn test_nesy_value_to_string() {
        assert_eq!(sym("hello").to_string(), "hello");
        assert_eq!(num(3.14).to_string(), "3.1400");
        assert_eq!(NesyValue::Bool(true).to_string(), "true");
        assert_eq!(var("x").to_string(), "?x");
        assert_eq!(NesyValue::Grounded(42).to_string(), "#42");
    }

    #[test]
    fn test_nesy_value_is_variable() {
        assert!(var("x").is_variable());
        assert!(!sym("x").is_variable());
        assert!(!num(1.0).is_variable());
    }

    #[test]
    fn test_max_facts_respected() {
        let mut engine = InferenceEngine::new(2);
        engine.add_fact(fact("a", vec![], 1.0));
        engine.add_fact(fact("b", vec![], 1.0));
        engine.add_fact(fact("c", vec![], 1.0));
        assert_eq!(engine.facts.len(), 2);
    }

    #[test]
    fn test_resolve_depth_zero() {
        let engine = InferenceEngine::new(10);
        let results = engine.resolve(&fact("anything", vec![], 1.0), 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_unify_predicate_mismatch() {
        let engine = InferenceEngine::new(10);
        let a = fact("father", vec![sym("x")], 1.0);
        let b = fact("mother", vec![sym("x")], 1.0);
        assert!(engine.unify(&a, &b).is_none());
    }
}
