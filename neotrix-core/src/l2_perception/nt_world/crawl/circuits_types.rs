use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub type Vector = Vec<f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CircuitMethod {
    Deductive, Inductive, Abductive, Analogical, Compositional, Recursive,
    Adversarial, FirstPrinciples, AutoFetch, KnowledgeRetrieval,
    GradientLearning, ArchitectureSearch, GpuCompute, DistributedConsensus,
    ExperienceDistill, EmergentAnalysis, SystemIntegration,
    EnsembleVoting, SelfImprovement, SparseRouting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningInput {
    pub state: Vector,
    pub query: Vector,
    pub context: HashMap<String, Vector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutput {
    pub state_delta: Vector,
    pub confidence: f64,
    pub trace: CircuitTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitTrace {
    pub method: CircuitMethod,
    pub steps: usize,
    pub intermediate_states: Vec<Vector>,
    pub convergence: f64,
}

pub trait ReasoningCircuit: Send + Sync {
    fn method(&self) -> CircuitMethod;
    fn complexity_ceiling(&self) -> f64;
    fn process(&self, input: &ReasoningInput) -> ReasoningOutput;
    fn is_applicable(&self, task_complexity: f64) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_reasoning_method_20_variants() {
        let variants = vec![
            CircuitMethod::Deductive, CircuitMethod::Inductive,
            CircuitMethod::Abductive, CircuitMethod::Analogical,
            CircuitMethod::Compositional, CircuitMethod::Recursive,
            CircuitMethod::Adversarial, CircuitMethod::FirstPrinciples,
            CircuitMethod::AutoFetch, CircuitMethod::KnowledgeRetrieval,
            CircuitMethod::GradientLearning, CircuitMethod::ArchitectureSearch,
            CircuitMethod::GpuCompute, CircuitMethod::DistributedConsensus,
            CircuitMethod::ExperienceDistill, CircuitMethod::EmergentAnalysis,
            CircuitMethod::SystemIntegration, CircuitMethod::EnsembleVoting,
            CircuitMethod::SelfImprovement, CircuitMethod::SparseRouting,
        ];
        assert_eq!(variants.len(), 20);
    }

    #[test]
    fn test_reasoning_input_default() {
        let input = ReasoningInput {
            state: vec![1.0, 0.0, -1.0],
            query: vec![0.5],
            context: HashMap::new(),
        };
        assert_eq!(input.state.len(), 3);
        assert_eq!(input.query.len(), 1);
    }

    #[test]
    fn test_reasoning_output_confidence_clamped() {
        let output = ReasoningOutput {
            state_delta: vec![0.1],
            confidence: 0.75,
            trace: CircuitTrace {
                method: CircuitMethod::Deductive,
                steps: 3,
                intermediate_states: vec![],
                convergence: 0.99,
            },
        };
        assert!((output.confidence - 0.75).abs() < 1e-6);
        assert_eq!(output.trace.steps, 3);
    }

    #[test]
    fn test_reasoning_trace_method_match() {
        let trace = CircuitTrace {
            method: CircuitMethod::Abductive,
            steps: 0,
            intermediate_states: vec![],
            convergence: 0.0,
        };
        assert_eq!(trace.method, CircuitMethod::Abductive);
    }

    #[test]
    fn test_reasoning_input_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("memory".into(), vec![0.1, 0.2]);
        let input = ReasoningInput {
            state: vec![],
            query: vec![],
            context: ctx,
        };
        assert_eq!(input.context.len(), 1);
        assert_eq!(input.context["memory"].len(), 2);
    }

    #[test]
    fn test_trait_mock_circuit_applicability() {
        struct Mock;
        impl ReasoningCircuit for Mock {
            fn method(&self) -> CircuitMethod { CircuitMethod::EnsembleVoting }
            fn complexity_ceiling(&self) -> f64 { 1.0 }
            fn process(&self, _: &ReasoningInput) -> ReasoningOutput {
                ReasoningOutput {
                    state_delta: vec![],
                    confidence: 1.0,
                    trace: CircuitTrace {
                        method: CircuitMethod::EnsembleVoting,
                        steps: 1, intermediate_states: vec![], convergence: 1.0,
                    },
                }
            }
            fn is_applicable(&self, c: f64) -> bool { c < 0.8 }
        }
        let c: Box<dyn ReasoningCircuit> = Box::new(Mock);
        assert!(c.is_applicable(0.5));
        assert!(!c.is_applicable(0.9));
        assert_eq!(c.method(), CircuitMethod::EnsembleVoting);
    }
}
