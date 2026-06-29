use super::counterfactual::StructuralCausalModel;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CausalEffect {
    pub cause_var: String,
    pub effect_var: String,
    pub average_causal_effect: f64,
    pub confidence: f64,
    pub adjustment_set: Vec<String>,
    pub method: AdjustmentMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentMethod {
    BackDoor,
    FrontDoor,
    DoCalculus,
    InstrumentalVariable,
}

impl AdjustmentMethod {
    pub fn name(&self) -> &'static str {
        match self {
            AdjustmentMethod::BackDoor => "backdoor",
            AdjustmentMethod::FrontDoor => "frontdoor",
            AdjustmentMethod::DoCalculus => "do-calculus",
            AdjustmentMethod::InstrumentalVariable => "iv",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoCalculusEngine {
    pub model: StructuralCausalModel,
}

impl DoCalculusEngine {
    pub fn new(model: StructuralCausalModel) -> Self {
        Self { model }
    }

    pub fn find_backdoor_adjustment(&self, cause: &str, effect: &str) -> Vec<String> {
        let ancestors_effect = self.ancestors(effect);
        let descendants_cause = self.descendants(cause);
        let mut adjustment = Vec::new();
        for node in ancestors_effect.iter() {
            if node != cause && !descendants_cause.contains(node) {
                let parents = self.parents(node);
                for p in &parents {
                    if !adjustment.contains(p) {
                        adjustment.push(p.clone());
                    }
                }
            }
        }
        adjustment
    }

    pub fn find_frontdoor_adjustment(&self, cause: &str, effect: &str) -> Vec<String> {
        let mediators = self.find_mediators(cause, effect);
        let mut adjustment = Vec::new();
        for m in &mediators {
            let m_parents = self.parents(m);
            let cause_children = self.children(cause);
            if m_parents.iter().any(|p| cause_children.contains(p)) {
                if !adjustment.contains(m) {
                    adjustment.push(m.clone());
                }
            }
        }
        adjustment
    }

    pub fn estimate_causal_effect(&self, cause: &str, effect: &str) -> CausalEffect {
        let backdoor = self.find_backdoor_adjustment(cause, effect);
        let method = if !backdoor.is_empty() {
            AdjustmentMethod::BackDoor
        } else {
            let frontdoor = self.find_frontdoor_adjustment(cause, effect);
            if !frontdoor.is_empty() {
                AdjustmentMethod::FrontDoor
            } else {
                AdjustmentMethod::DoCalculus
            }
        };

        let adjustment_set = match method {
            AdjustmentMethod::BackDoor => backdoor,
            AdjustmentMethod::FrontDoor => self.find_frontdoor_adjustment(cause, effect),
            _ => vec![],
        };

        let ace = self.compute_ace(cause, effect, &adjustment_set);
        let conf = self.compute_confidence(cause, effect, &adjustment_set);

        CausalEffect {
            cause_var: cause.to_string(),
            effect_var: effect.to_string(),
            average_causal_effect: ace,
            confidence: conf,
            adjustment_set,
            method,
        }
    }

    pub fn compute_ate(&self, treatment: &str, outcome: &str, value: f64) -> f64 {
        let effect = self.estimate_causal_effect(treatment, outcome);
        effect.average_causal_effect * value
    }

    pub fn is_confounded(&self, cause: &str, effect: &str) -> bool {
        let backdoor = self.find_backdoor_adjustment(cause, effect);
        !backdoor.is_empty()
    }

    pub fn correlation_vs_causation(&self, x: &str, y: &str) -> (f64, f64, String) {
        let corr = self.compute_correlation(x, y);
        let ace = self.estimate_causal_effect(x, y).average_causal_effect;
        let verdict = if (corr - ace).abs() > 0.2 {
            format!(
                "⚠️ correlation ({:.2}) != causation ({:.2}), confounded by {}",
                corr,
                ace,
                self.find_confounder(x, y)
            )
        } else {
            format!("✅ correlation ({:.2}) ≈ causation ({:.2})", corr, ace)
        };
        (corr, ace, verdict)
    }

    fn ancestors(&self, node: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![node.to_string()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for edge in &self.model.edges {
                if edge.to == current {
                    if !visited.contains(&edge.from) {
                        stack.push(edge.from.clone());
                    }
                    if !result.contains(&edge.from) {
                        result.push(edge.from.clone());
                    }
                }
            }
        }
        result
    }

    fn descendants(&self, node: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![node.to_string()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for edge in &self.model.edges {
                if edge.from == current {
                    if !visited.contains(&edge.to) {
                        stack.push(edge.to.clone());
                    }
                    if !result.contains(&edge.to) {
                        result.push(edge.to.clone());
                    }
                }
            }
        }
        result
    }

    fn parents(&self, node: &str) -> Vec<String> {
        self.model
            .edges
            .iter()
            .filter(|e| e.to == node)
            .map(|e| e.from.clone())
            .collect()
    }

    fn children(&self, node: &str) -> Vec<String> {
        self.model
            .edges
            .iter()
            .filter(|e| e.from == node)
            .map(|e| e.to.clone())
            .collect()
    }

    fn find_mediators(&self, cause: &str, effect: &str) -> Vec<String> {
        let cause_descendants = self.descendants(cause);
        let effect_ancestors = self.ancestors(effect);
        cause_descendants
            .into_iter()
            .filter(|n| n != effect && effect_ancestors.contains(n))
            .collect()
    }

    fn find_confounder(&self, x: &str, y: &str) -> String {
        let x_ancestors: HashSet<String> = self.ancestors(x).into_iter().collect();
        let y_ancestors: HashSet<String> = self.ancestors(y).into_iter().collect();
        let common: Vec<&String> = x_ancestors.intersection(&y_ancestors).collect();
        if common.is_empty() {
            "none".to_string()
        } else {
            common
                .iter()
                .map(|s| (*s).clone())
                .collect::<Vec<String>>()
                .join(", ")
        }
    }

    fn compute_ace(&self, cause: &str, effect: &str, adjustment: &[String]) -> f64 {
        let mut ace = 0.0;
        if let (Some(cause_node), Some(effect_node)) =
            (self.model.nodes.get(cause), self.model.nodes.get(effect))
        {
            let base = QuantizedVSA::similarity(&cause_node.vsa_vector, &effect_node.vsa_vector);
            ace = base * 2.0 - 1.0;
            for adj in adjustment {
                if let Some(adj_node) = self.model.nodes.get(adj) {
                    let adj_sim =
                        QuantizedVSA::similarity(&cause_node.vsa_vector, &adj_node.vsa_vector);
                    ace -= adj_sim * 0.1;
                }
            }
        }
        ace.clamp(-1.0, 1.0)
    }

    fn compute_confidence(&self, cause: &str, effect: &str, _adjustment: &[String]) -> f64 {
        let mut confidence = 0.7;
        if let Some(_cause_node) = self.model.nodes.get(cause) {
            let evidence_count = self
                .model
                .edges
                .iter()
                .filter(|e| e.from == cause || e.to == cause)
                .count() as f64;
            confidence += evidence_count * 0.05;
        }
        if let Some(_effect_node) = self.model.nodes.get(effect) {
            let evidence_count = self
                .model
                .edges
                .iter()
                .filter(|e| e.from == effect || e.to == effect)
                .count() as f64;
            confidence += evidence_count * 0.05;
        }
        confidence.min(1.0)
    }

    fn compute_correlation(&self, x: &str, y: &str) -> f64 {
        if let (Some(xn), Some(yn)) = (self.model.nodes.get(x), self.model.nodes.get(y)) {
            QuantizedVSA::similarity(&xn.vsa_vector, &yn.vsa_vector)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::counterfactual::*;
    use std::collections::HashMap;

    fn make_test_model() -> StructuralCausalModel {
        let mut model = StructuralCausalModel {
            nodes: HashMap::new(),
            edges: vec![],
            name: "test".into(),
            version: 1,
        };
        model.nodes.insert(
            "X".into(),
            CausalNode {
                name: "X".into(),
                vsa_vector: QuantizedVSA::seeded_random(1, 256),
                observed_value: 1.0,
                is_endogenous: true,
            },
        );
        model.nodes.insert(
            "Y".into(),
            CausalNode {
                name: "Y".into(),
                vsa_vector: QuantizedVSA::seeded_random(2, 256),
                observed_value: 2.0,
                is_endogenous: true,
            },
        );
        model.nodes.insert(
            "Z".into(),
            CausalNode {
                name: "Z".into(),
                vsa_vector: QuantizedVSA::seeded_random(3, 256),
                observed_value: 0.5,
                is_endogenous: false,
            },
        );
        model.edges.push(CausalEdge {
            from: "Z".into(),
            to: "X".into(),
            strength: 0.8,
            mechanism: CausalMechanism::Direct,
        });
        model.edges.push(CausalEdge {
            from: "Z".into(),
            to: "Y".into(),
            strength: 0.6,
            mechanism: CausalMechanism::Direct,
        });
        model.edges.push(CausalEdge {
            from: "X".into(),
            to: "Y".into(),
            strength: 0.4,
            mechanism: CausalMechanism::Direct,
        });
        model
    }

    #[test]
    fn test_backdoor_adjustment() {
        let engine = DoCalculusEngine::new(make_test_model());
        let adj = engine.find_backdoor_adjustment("X", "Y");
        assert!(adj.contains(&"Z".to_string()));
    }

    #[test]
    fn test_causal_effect_estimation() {
        let engine = DoCalculusEngine::new(make_test_model());
        let effect = engine.estimate_causal_effect("X", "Y");
        assert_eq!(effect.cause_var, "X");
        assert_eq!(effect.effect_var, "Y");
        assert!(effect.average_causal_effect >= -1.0);
        assert!(effect.average_causal_effect <= 1.0);
    }

    #[test]
    fn test_is_confounded() {
        let engine = DoCalculusEngine::new(make_test_model());
        assert!(engine.is_confounded("X", "Y"));
    }

    #[test]
    fn test_correlation_vs_causation() {
        let engine = DoCalculusEngine::new(make_test_model());
        let (corr, ace, verdict) = engine.correlation_vs_causation("X", "Y");
        assert!((0.0..=1.0).contains(&corr));
        assert!((0.0..=1.0).contains(&ace));
        assert!(!verdict.is_empty());
    }

    #[test]
    fn test_find_mediators() {
        let engine = DoCalculusEngine::new(make_test_model());
        let mediators = engine.find_mediators("X", "Y");
        assert!(mediators.is_empty());
    }

    #[test]
    fn test_ancestors() {
        let engine = DoCalculusEngine::new(make_test_model());
        let ancestors = engine.ancestors("Y");
        assert!(ancestors.contains(&"X".to_string()));
        assert!(ancestors.contains(&"Z".to_string()));
    }

    #[test]
    fn test_descendants() {
        let engine = DoCalculusEngine::new(make_test_model());
        let descendants = engine.descendants("Z");
        assert!(descendants.contains(&"X".to_string()));
        assert!(descendants.contains(&"Y".to_string()));
    }

    #[test]
    fn test_parents_children() {
        let engine = DoCalculusEngine::new(make_test_model());
        let x_parents = engine.parents("X");
        assert!(x_parents.contains(&"Z".to_string()));
        let z_children = engine.children("Z");
        assert!(z_children.contains(&"X".to_string()));
        assert!(z_children.contains(&"Y".to_string()));
    }

    #[test]
    fn test_adjustment_method_names() {
        assert_eq!(AdjustmentMethod::BackDoor.name(), "backdoor");
        assert_eq!(AdjustmentMethod::FrontDoor.name(), "frontdoor");
        assert_eq!(AdjustmentMethod::DoCalculus.name(), "do-calculus");
    }

    #[test]
    fn test_compute_ate() {
        let engine = DoCalculusEngine::new(make_test_model());
        let ate = engine.compute_ate("X", "Y", 1.0);
        assert!((0.0..=1.0).contains(&ate));
    }
}
