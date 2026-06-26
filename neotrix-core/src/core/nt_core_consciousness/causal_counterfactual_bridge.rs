use super::causal_reasoning::{CausalEffect, DoCalculusEngine};
use super::counterfactual::{
    CoInResult, CounterfactualConfig, CounterfactualReasoner, FactualScenario, Intervention,
    InterventionType, StructuralCausalModel,
};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CausalAnalysisResult {
    pub causal_effect: CausalEffect,
    pub counterfactual_results: Vec<CoInResult>,
    pub confounded: bool,
    pub elapsed_ms: f64,
}

#[derive(Debug, Clone)]
pub struct CausalReasoningPipeline {
    pub model: StructuralCausalModel,
    pub reasoner: CounterfactualReasoner,
    pub engine: DoCalculusEngine,
}

impl CausalReasoningPipeline {
    pub fn new(model: StructuralCausalModel) -> Self {
        let config = CounterfactualConfig::default();
        let mut reasoner = CounterfactualReasoner::new(config);
        reasoner.scm = model.clone();
        Self {
            model: model.clone(),
            reasoner,
            engine: DoCalculusEngine::new(model),
        }
    }

    pub fn estimate_causal_effect(&self, cause: &str, effect: &str) -> CausalEffect {
        self.engine.estimate_causal_effect(cause, effect)
    }

    pub fn generate_counterfactual(&mut self, intervention: Intervention) -> CoInResult {
        let factual =
            self.build_factual_scenario(&format!("intervention on {}", intervention.target_node));
        self.reasoner.record_factual(factual);
        self.reasoner.ask_counterfactual(
            &format!("What if we changed {}?", intervention.target_node),
            intervention,
        )
    }

    pub fn is_confounded(&self, cause: &str, effect: &str) -> bool {
        self.engine.is_confounded(cause, effect)
    }

    pub fn compute_ate(&self, treatment: &str, outcome: &str, value: f64) -> f64 {
        self.engine.compute_ate(treatment, outcome, value)
    }

    pub fn full_causal_analysis(&mut self, cause: &str, effect: &str) -> CausalAnalysisResult {
        let start = Instant::now();
        let causal_effect = self.estimate_causal_effect(cause, effect);
        let confounded = self.is_confounded(cause, effect);

        let mut counterfactual_results = Vec::new();

        let remove_intervention = Intervention {
            target_node: cause.to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: format!("Remove cause {} to observe effect on {}", cause, effect),
        };
        let factual = self.build_factual_scenario("removal scenario");
        self.reasoner.record_factual(factual);
        counterfactual_results.push(self.reasoner.ask_counterfactual(
            &format!("What if {} were removed?", cause),
            remove_intervention,
        ));

        if let Some(node) = self.model.nodes.get(cause) {
            let double_intervention = Intervention {
                target_node: cause.to_string(),
                new_value: node.observed_value * 2.0,
                intervention_type: InterventionType::Do,
                rationale: format!("Double cause {} to amplify effect on {}", cause, effect),
            };
            let factual = self.build_factual_scenario("amplification scenario");
            self.reasoner.record_factual(factual);
            counterfactual_results.push(self.reasoner.ask_counterfactual(
                &format!("What if {} were doubled?", cause),
                double_intervention,
            ));
        }

        let context_intervention = Intervention {
            target_node: effect.to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Context,
            rationale: format!("Change context for {}", effect),
        };
        let factual = self.build_factual_scenario("context change scenario");
        self.reasoner.record_factual(factual);
        counterfactual_results.push(self.reasoner.ask_counterfactual(
            &format!("What if context for {} changed?", effect),
            context_intervention,
        ));

        CausalAnalysisResult {
            causal_effect,
            counterfactual_results,
            confounded,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    fn build_factual_scenario(&self, description: &str) -> FactualScenario {
        let mut node_values = HashMap::new();
        let mut node_vectors = HashMap::new();
        for (name, node) in &self.model.nodes {
            node_values.insert(name.clone(), node.observed_value);
            node_vectors.insert(name.clone(), node.vsa_vector.clone());
        }
        FactualScenario {
            node_values,
            node_vectors,
            timestamp: Instant::now(),
            description: description.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::counterfactual::{
        CausalEdge, CausalMechanism, CausalNode,
    };
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
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
    fn test_pipeline_new() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model.clone());
        assert_eq!(pipeline.model.nodes.len(), 3);
        assert_eq!(pipeline.model.edges.len(), 3);
        assert_eq!(pipeline.model.name, "test");
    }

    #[test]
    fn test_pipeline_estimate_effect() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model);
        let effect = pipeline.estimate_causal_effect("X", "Y");
        assert_eq!(effect.cause_var, "X");
        assert_eq!(effect.effect_var, "Y");
        assert!(effect.average_causal_effect >= -1.0);
        assert!(effect.average_causal_effect <= 1.0);
        assert!(!effect.adjustment_set.is_empty());
    }

    #[test]
    fn test_pipeline_generate_counterfactual() {
        let model = make_test_model();
        let mut pipeline = CausalReasoningPipeline::new(model);
        let intervention = Intervention {
            target_node: "X".to_string(),
            new_value: 0.0,
            intervention_type: InterventionType::Do,
            rationale: "test removal".to_string(),
        };
        let result = pipeline.generate_counterfactual(intervention);
        assert_eq!(result.query, "What if we changed X?");
        assert!(!result.counterfactuals.is_empty());
    }

    #[test]
    fn test_pipeline_is_confounded() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model);
        assert!(pipeline.is_confounded("X", "Y"));
        assert!(!pipeline.is_confounded("Z", "X"));
    }

    #[test]
    fn test_pipeline_compute_ate() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model);
        let ate = pipeline.compute_ate("X", "Y", 1.0);
        assert!(ate >= 0.0);
        assert!(ate <= 1.0);
    }

    #[test]
    fn test_pipeline_full_analysis_contains_both() {
        let model = make_test_model();
        let mut pipeline = CausalReasoningPipeline::new(model);
        let analysis = pipeline.full_causal_analysis("X", "Y");
        assert_eq!(analysis.causal_effect.cause_var, "X");
        assert_eq!(analysis.causal_effect.effect_var, "Y");
        assert!(!analysis.counterfactual_results.is_empty());
    }

    #[test]
    fn test_pipeline_three_counterfactuals() {
        let model = make_test_model();
        let mut pipeline = CausalReasoningPipeline::new(model);
        let analysis = pipeline.full_causal_analysis("X", "Y");
        assert_eq!(analysis.counterfactual_results.len(), 3);
        for result in &analysis.counterfactual_results {
            assert!(!result.counterfactuals.is_empty());
        }
    }

    #[test]
    fn test_pipeline_factual_build_has_all_nodes() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model);
        let factual = pipeline.build_factual_scenario("test factual");
        assert_eq!(factual.node_values.len(), 3);
        assert!(factual.node_values.contains_key("X"));
        assert!(factual.node_values.contains_key("Y"));
        assert!(factual.node_values.contains_key("Z"));
        assert_eq!(factual.node_vectors.len(), 3);
    }

    #[test]
    fn test_pipeline_effect_z_on_y() {
        let model = make_test_model();
        let pipeline = CausalReasoningPipeline::new(model);
        let effect = pipeline.estimate_causal_effect("Z", "Y");
        assert_eq!(effect.cause_var, "Z");
        assert_eq!(effect.effect_var, "Y");
    }

    #[test]
    fn test_pipeline_confounded_flag() {
        let model = make_test_model();
        let mut pipeline = CausalReasoningPipeline::new(model);
        let analysis = pipeline.full_causal_analysis("X", "Y");
        assert!(analysis.confounded);
        let analysis2 = pipeline.full_causal_analysis("Z", "X");
        assert!(!analysis2.confounded);
    }
}
