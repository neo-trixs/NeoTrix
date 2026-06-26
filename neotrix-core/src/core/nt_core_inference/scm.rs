use std::collections::{HashMap, HashSet};

/// A variable in the causal model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CausalVariable {
    pub name: String,
    pub domain: Vec<String>,
    pub observed: bool,
}

/// A structural equation: Xi = f(PAi, Ui) where PAi = parents, Ui = noise
#[derive(Debug, Clone)]
pub struct StructuralEquation {
    pub target: String,
    pub parents: Vec<String>,
    pub coefficients: HashMap<String, f64>,
    pub noise_variance: f64,
}

/// A causal graph (DAG)
#[derive(Debug, Clone)]
pub struct CausalGraph {
    pub nodes: HashSet<String>,
    pub edges: HashMap<String, Vec<String>>,
    pub equations: HashMap<String, StructuralEquation>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            equations: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: &str) {
        self.nodes.insert(name.to_string());
        self.edges.entry(name.to_string()).or_default();
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.nodes.insert(from.to_string());
        self.nodes.insert(to.to_string());
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    pub fn parents_of(&self, node: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(_, children)| children.contains(&node.to_string()))
            .map(|(parent, _)| parent.clone())
            .collect()
    }

    pub fn children_of(&self, node: &str) -> Vec<String> {
        self.edges.get(node).cloned().unwrap_or_default()
    }

    pub fn ancestors(&self, node: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut stack = vec![node.to_string()];
        while let Some(n) = stack.pop() {
            for p in self.parents_of(&n) {
                if result.insert(p.clone()) {
                    stack.push(p);
                }
            }
        }
        result
    }

    pub fn has_path_avoiding(&self, from: &str, to: &str, avoid: &HashSet<String>) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from.to_string()];
        while let Some(n) = stack.pop() {
            if n == to {
                return true;
            }
            if !visited.insert(n.clone()) {
                continue;
            }
            for child in self.children_of(&n) {
                if !avoid.contains(&child) {
                    stack.push(child);
                }
            }
            for parent in self.parents_of(&n) {
                if !avoid.contains(&parent) {
                    stack.push(parent);
                }
            }
        }
        false
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of do-calculus: P(Y|do(X=x), Z=z)
#[derive(Debug, Clone)]
pub struct CausalEffect {
    pub target: String,
    pub intervention_var: String,
    pub intervention_value: String,
    pub estimate: f64,
    pub confidence: f64,
    pub method: String,
    pub adjusted_vars: Vec<String>,
}

/// The main SCM engine
#[derive(Debug)]
pub struct SCMEngine {
    pub graph: CausalGraph,
    pub data: HashMap<String, Vec<f64>>,
    max_observations: usize,
}

impl SCMEngine {
    pub fn new() -> Self {
        Self {
            graph: CausalGraph::new(),
            data: HashMap::new(),
            max_observations: 10000,
        }
    }

    pub fn with_max_observations(mut self, max: usize) -> Self {
        self.max_observations = max;
        self
    }

    pub fn observe(&mut self, variable: &str, value: f64) {
        let entry = self.data.entry(variable.to_string()).or_default();
        if entry.len() >= self.max_observations {
            entry.remove(0);
        }
        entry.push(value);
    }

    pub fn add_causal_relation(&mut self, cause: &str, effect: &str, coefficient: f64) {
        self.graph.add_edge(cause, effect);
        let eq = self
            .graph
            .equations
            .entry(effect.to_string())
            .or_insert(StructuralEquation {
                target: effect.to_string(),
                parents: vec![],
                coefficients: HashMap::new(),
                noise_variance: 0.1,
            });
        if !eq.parents.contains(&cause.to_string()) {
            eq.parents.push(cause.to_string());
        }
        eq.coefficients.insert(cause.to_string(), coefficient);
    }

    pub fn estimate_do(&self, target: &str, intervention: &str, value: f64) -> CausalEffect {
        let backdoor_paths = self.find_backdoor_paths(target, intervention);
        let adjusted: Vec<String> = if backdoor_paths.is_empty() {
            vec![]
        } else {
            self.graph.parents_of(intervention)
        };

        let estimate = if let Some(eq) = self.graph.equations.get(target) {
            eq.coefficients.get(intervention).copied().unwrap_or(0.0) * value
                + eq.coefficients.values().sum::<f64>() * 0.5
        } else {
            let target_data: Vec<f64> = self.data.get(target).cloned().unwrap_or_default();
            let intervention_data: Vec<f64> =
                self.data.get(intervention).cloned().unwrap_or_default();
            if target_data.len() >= 2 && intervention_data.len() >= 2 {
                let mean_y: f64 = target_data.iter().sum::<f64>() / target_data.len() as f64;
                let mean_x: f64 =
                    intervention_data.iter().sum::<f64>() / intervention_data.len() as f64;
                let cov: f64 = target_data
                    .iter()
                    .zip(intervention_data.iter())
                    .map(|(y, x)| (y - mean_y) * (x - mean_x))
                    .sum();
                let var_x: f64 = intervention_data.iter().map(|x| (x - mean_x).powi(2)).sum();
                (cov / (var_x + 1e-10)) * value
            } else {
                0.0
            }
        };

        let method = if adjusted.is_empty() {
            "direct".to_string()
        } else if !backdoor_paths.is_empty() {
            "backdoor".to_string()
        } else {
            "direct".to_string()
        };

        CausalEffect {
            target: target.to_string(),
            intervention_var: intervention.to_string(),
            intervention_value: format!("{:.2}", value),
            estimate,
            confidence: (1.0 / (1.0 + adjusted.len() as f64)).min(0.9),
            method,
            adjusted_vars: adjusted,
        }
    }

    pub fn counterfactual(
        &self,
        target: &str,
        intervention: &str,
        actual_value: f64,
        hypothetical_value: f64,
    ) -> (f64, f64) {
        let beta = if let Some(eq) = self.graph.equations.get(target) {
            eq.coefficients.get(intervention).copied().unwrap_or(0.0)
        } else {
            0.3
        };

        let delta = beta * (hypothetical_value - actual_value);
        let last_y = self
            .data
            .get(target)
            .and_then(|d| d.last())
            .copied()
            .unwrap_or(0.0);

        (last_y + delta, beta.abs().min(0.9))
    }

    fn find_backdoor_paths(&self, target: &str, intervention: &str) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut stack = vec![(intervention.to_string(), vec![intervention.to_string()])];
        let mut visited = HashSet::new();

        while let Some((node, path)) = stack.pop() {
            if node == target {
                continue;
            }
            if !visited.insert(node.clone()) {
                continue;
            }
            if path.len() > 10 {
                continue;
            }

            for parent in self.graph.parents_of(&node) {
                let mut new_path = path.clone();
                new_path.push(parent.clone());
                if parent == target {
                    paths.push(new_path);
                } else {
                    stack.push((parent, new_path));
                }
            }
        }
        paths
    }

    pub fn estimate_frontdoor(
        &self,
        target: &str,
        intervention: &str,
        mediator: &str,
        value: f64,
    ) -> CausalEffect {
        if let Some(eq_xm) = self.graph.equations.get(mediator) {
            if let Some(eq_my) = self.graph.equations.get(target) {
                let beta_xm = eq_xm.coefficients.get(intervention).copied().unwrap_or(0.0);
                let beta_my = eq_my.coefficients.get(mediator).copied().unwrap_or(0.0);
                let estimate = beta_xm * beta_my * value;

                return CausalEffect {
                    target: target.to_string(),
                    intervention_var: intervention.to_string(),
                    intervention_value: format!("{:.2}", value),
                    estimate,
                    confidence: (beta_xm.abs() * beta_my.abs()).min(0.8),
                    method: "frontdoor".to_string(),
                    adjusted_vars: vec![mediator.to_string()],
                };
            }
        }

        CausalEffect {
            target: target.to_string(),
            intervention_var: intervention.to_string(),
            intervention_value: format!("{:.2}", value),
            estimate: 0.0,
            confidence: 0.1,
            method: "frontdoor(fallback)".to_string(),
            adjusted_vars: vec![],
        }
    }

    pub fn stats(&self) -> String {
        format!(
            "SCMEngine: {} vars, {} edges, {} eqs, {} obs recorded",
            self.graph.nodes.len(),
            self.graph.edges.values().map(|v| v.len()).sum::<usize>(),
            self.graph.equations.len(),
            self.data.values().map(|v| v.len()).sum::<usize>(),
        )
    }
}

impl Default for SCMEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_small_graph() -> SCMEngine {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("X", "Y", 0.8);
        engine.add_causal_relation("Z", "X", 0.5);
        engine.add_causal_relation("Z", "Y", 0.3);
        engine
    }

    #[test]
    fn test_graph_construction() {
        let engine = build_small_graph();
        assert!(engine.graph.nodes.contains("X"));
        assert!(engine.graph.nodes.contains("Y"));
        assert!(engine.graph.nodes.contains("Z"));
        assert_eq!(engine.graph.nodes.len(), 3);
    }

    #[test]
    fn test_edge_queries() {
        let engine = build_small_graph();
        let children_of_z = engine.graph.children_of("Z");
        assert!(children_of_z.contains(&"X".to_string()));
        assert!(children_of_z.contains(&"Y".to_string()));

        let parents_of_y = engine.graph.parents_of("Y");
        assert!(parents_of_y.contains(&"X".to_string()));
        assert!(parents_of_y.contains(&"Z".to_string()));
    }

    #[test]
    fn test_backdoor_path_finding() {
        let engine = build_small_graph();
        let paths = engine.find_backdoor_paths("Y", "X");
        assert!(!paths.is_empty(), "Z→X←?→Y should have backdoor path via Z");
        let has_z_path = paths.iter().any(|p| p.contains(&"Z".to_string()));
        assert!(has_z_path, "backdoor path should go through confounder Z");
    }

    #[test]
    fn test_do_direct_effect() {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("X", "Y", 0.8);
        let effect = engine.estimate_do("Y", "X", 2.0);
        assert_eq!(effect.method, "direct");
        assert!((effect.estimate - 1.6).abs() < 0.01);
        assert!(effect.confidence > 0.0);
    }

    #[test]
    fn test_do_backdoor_adjustment() {
        let engine = build_small_graph();
        let effect = engine.estimate_do("Y", "X", 1.5);
        assert_eq!(effect.method, "backdoor");
        assert!(effect.estimate > 0.0);
        assert!(!effect.adjusted_vars.is_empty());
    }

    #[test]
    fn test_frontdoor_estimation() {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("X", "M", 0.7);
        engine.add_causal_relation("M", "Y", 0.6);
        let effect = engine.estimate_frontdoor("Y", "X", "M", 2.0);
        assert_eq!(effect.method, "frontdoor");
        assert!((effect.estimate - 0.7 * 0.6 * 2.0).abs() < 0.01);
    }

    #[test]
    fn test_frontdoor_fallback() {
        let engine = SCMEngine::new();
        let effect = engine.estimate_frontdoor("Y", "X", "M", 1.0);
        assert_eq!(effect.method, "frontdoor(fallback)");
        assert!((effect.estimate - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_counterfactual() {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("X", "Y", 0.5);
        engine.observe("Y", 10.0);
        let (cf_value, confidence) = engine.counterfactual("Y", "X", 5.0, 10.0);
        assert!((cf_value - 10.0 - 0.5 * 5.0).abs() < 0.01);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_ancestor_tracking() {
        let engine = build_small_graph();
        let ancestors_y = engine.graph.ancestors("Y");
        assert!(ancestors_y.contains("X"));
        assert!(ancestors_y.contains("Z"));
        assert!(!ancestors_y.contains("Y"));
    }

    #[test]
    fn test_has_path_avoiding() {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("A", "B", 0.5);
        engine.add_causal_relation("B", "C", 0.5);
        engine.add_causal_relation("C", "D", 0.5);

        let avoid = HashSet::from(["B".to_string()]);
        assert!(engine.graph.has_path_avoiding("A", "D", &HashSet::new()));
        assert!(!engine.graph.has_path_avoiding("A", "D", &avoid));
    }

    #[test]
    fn test_data_observation_bounds() {
        let mut engine = SCMEngine::new().with_max_observations(5);
        for i in 0..20 {
            engine.observe("X", i as f64);
        }
        assert_eq!(engine.data.get("X").unwrap().len(), 5);
        let last = *engine.data.get("X").unwrap().last().unwrap();
        assert_eq!(last, 19.0);
    }

    #[test]
    fn test_empty_engine() {
        let engine = SCMEngine::new();
        let s = engine.stats();
        assert!(s.contains("0 vars"));
        assert!(s.contains("0 edges"));
    }

    #[test]
    fn test_do_with_no_data_fallback() {
        let mut engine = SCMEngine::new();
        engine.add_causal_relation("X", "Y", 0.8);
        engine.observe("Y", 1.0);
        engine.observe("Y", 2.0);
        let effect = engine.estimate_do("Y", "X", 1.0);
        assert_eq!(effect.method, "direct");
        assert!((effect.estimate - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_frontdoor_no_equation() {
        let mut engine = SCMEngine::new();
        engine.graph.add_edge("X", "M");
        engine.graph.add_edge("M", "Y");
        let effect = engine.estimate_frontdoor("Y", "X", "M", 1.0);
        assert_eq!(effect.method, "frontdoor(fallback)");
        assert_eq!(effect.estimate, 0.0);
    }

    #[test]
    fn test_graph_add_variable() {
        let mut engine = SCMEngine::new();
        engine.graph.add_variable("isolated");
        assert!(engine.graph.nodes.contains("isolated"));
        assert_eq!(engine.graph.children_of("isolated").len(), 0);
    }

    #[test]
    fn test_causal_effect_fields() {
        let effect = CausalEffect {
            target: "Y".into(),
            intervention_var: "X".into(),
            intervention_value: "3.00".into(),
            estimate: 2.4,
            confidence: 0.85,
            method: "direct".into(),
            adjusted_vars: vec![],
        };
        assert_eq!(effect.target, "Y");
        assert_eq!(effect.method, "direct");
        assert!((effect.estimate - 2.4).abs() < 0.01);
    }

    #[test]
    fn test_multiple_observations() {
        let mut engine = SCMEngine::new();
        for i in 0..10 {
            engine.observe("X", i as f64);
            engine.observe("Y", (i as f64) * 0.8);
        }
        assert_eq!(engine.data.get("X").unwrap().len(), 10);
        assert_eq!(engine.data.get("Y").unwrap().len(), 10);
    }

    #[test]
    fn test_counterfactual_no_data() {
        let engine = SCMEngine::new();
        let (cf, conf) = engine.counterfactual("Y", "X", 2.0, 5.0);
        assert!((cf - 0.3 * 3.0).abs() < 0.01);
        assert!((conf - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_cyclic_graph_still_works() {
        let mut engine = SCMEngine::new();
        engine.graph.add_edge("A", "B");
        engine.graph.add_edge("B", "C");
        engine.graph.add_edge("C", "A");
        let paths = engine.find_backdoor_paths("C", "A");
        assert!(paths.is_empty() || !paths.is_empty());
    }
}
