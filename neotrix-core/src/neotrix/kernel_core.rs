use serde::{Deserialize, Serialize};
use crate::neotrix::signal::Vector;
use std::collections::HashMap;
use crate::neotrix::nt_world_browse::circuits_types::{ReasoningMethod, ReasoningOutput, ReasoningTrace};

#[derive(Debug, Clone, Copy)]
pub struct StageInfo {
    pub label: &'static str,
    pub description: &'static str,
}

pub const EVOLUTION: &[StageInfo] = &[
    StageInfo { label: "Stage 0", description: "Initial" },
    StageInfo { label: "Stage 1", description: "Pattern Recognition" },
    StageInfo { label: "Stage 2", description: "Abstraction" },
    StageInfo { label: "Stage 3", description: "Analogy Engine" },
    StageInfo { label: "Stage 4", description: "Recursive Reasoner" },
    StageInfo { label: "Stage 5", description: "Compositional" },
    StageInfo { label: "Stage 6", description: "Adversarial" },
    StageInfo { label: "Stage 7", description: "First Principles" },
    StageInfo { label: "Stage 8", description: "Auto-Fetch" },
    StageInfo { label: "Stage 9", description: "Knowledge Retrieval" },
    StageInfo { label: "Stage 10", description: "Gradient Learning" },
    StageInfo { label: "Stage 11", description: "Architecture Search" },
    StageInfo { label: "Stage 12", description: "GPU Compute" },
    StageInfo { label: "Stage 13", description: "Distributed Consensus" },
    StageInfo { label: "Stage 14", description: "Experience Distill" },
    StageInfo { label: "Stage 15", description: "Emergent Analysis" },
    StageInfo { label: "Stage 16", description: "System Integration" },
    StageInfo { label: "Stage 17", description: "Ensemble Voting" },
    StageInfo { label: "Stage 18", description: "Self-Improvement" },
];

pub const KERNEL_DIM: usize = 128;

#[derive(Debug, Clone)]
pub struct KernelStats {
    pub stage: usize,
    pub label: String,
    pub state_dim: usize,
    pub total: usize,
    pub active: Vec<ReasoningMethod>,
    pub energy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningKernel {
    pub stage: usize,
    pub state: Vector,
}

impl ReasoningKernel {
    pub fn new(stage: usize) -> Self {
        let dim = KERNEL_DIM;
        Self {
            stage: stage.min(EVOLUTION.len() - 1),
            state: vec![0.0; dim],
        }
    }

    pub fn reason(&self, _query: &[f64], _context: Option<HashMap<String, Vector>>) -> ReasoningOutput {
        ReasoningOutput {
            state_delta: self.state.clone(),
            confidence: 0.5,
            trace: ReasoningTrace {
                method: ReasoningMethod::Deductive,
                steps: 1,
                intermediate_states: vec![],
                convergence: 0.5,
            },
        }
    }

    pub fn evolve_stage(&mut self) {
        self.stage = (self.stage + 1).min(EVOLUTION.len() - 1);
    }

    pub fn stats(&self) -> KernelStats {
        KernelStats {
            stage: self.stage,
            label: EVOLUTION[self.stage].label.to_string(),
            state_dim: self.state.len(),
            total: 8,
            active: vec![ReasoningMethod::Deductive],
            energy: self.state.iter().map(|x| x.abs()).sum::<f64>() / self.state.len().max(1) as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_kernel_new() {
        let k = ReasoningKernel::new(5);
        assert_eq!(k.stage, 5);
        assert_eq!(k.state.len(), KERNEL_DIM);
    }

    #[test]
    fn test_reasoning_kernel_stage_clamped() {
        let k = ReasoningKernel::new(100);
        assert_eq!(k.stage, EVOLUTION.len() - 1);
    }

    #[test]
    fn test_reasoning_kernel_evolve_stage() {
        let mut k = ReasoningKernel::new(0);
        k.evolve_stage();
        assert_eq!(k.stage, 1);
    }

    #[test]
    fn test_reasoning_kernel_evolve_stage_max() {
        let mut k = ReasoningKernel::new(EVOLUTION.len() - 1);
        k.evolve_stage();
        assert_eq!(k.stage, EVOLUTION.len() - 1);
    }

    #[test]
    fn test_reasoning_kernel_reason() {
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; KERNEL_DIM];
        let output = k.reason(&query, None);
        assert!((output.confidence - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_reasoning_kernel_stats() {
        let k = ReasoningKernel::new(2);
        let stats = k.stats();
        assert_eq!(stats.stage, 2);
        assert_eq!(stats.state_dim, KERNEL_DIM);
    }

    #[test]
    fn test_stage_info_const() {
        assert_eq!(EVOLUTION.len(), 19);
        assert_eq!(EVOLUTION[0].label, "Stage 0");
        assert_eq!(EVOLUTION[0].description, "Initial");
    }

    #[test]
    fn test_kernel_dim_constant() {
        assert_eq!(KERNEL_DIM, 128);
    }
}
