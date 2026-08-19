use crate::core::nt_core_reasoning::ReasoningTrace;
use serde::{Deserialize, Serialize};

pub type Vector = Vec<f64>;

/// 推理状态维度（与 L4 认知层共享的 kernel 维度契约）。
pub const KERNEL_DIM: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningMethod {
    Deductive, Inductive, Abductive, Analogical, FirstPrinciples,
    Recursive, Compositional, Adversarial, AutoFetch,
    KnowledgeRetrieval, GradientLearning, ArchitectureSearch,
    GpuCompute, DistributedConsensus, ExperienceDistill,
    EmergentAnalysis, SystemIntegration, EnsembleVoting,
    SelfImprovement, SparseRouting,
}

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

/// 推理输出包装器（统一 ReasoningTrace + state_delta/confidence）。
#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub state_delta: Vector,
    pub confidence: f64,
    pub trace: ReasoningTrace,
}

#[derive(Debug, Clone)]
pub struct KernelStats {
    pub stage: usize,
    pub label: String,
    pub state_dim: usize,
    pub total: usize,
    pub active: Vec<ReasoningMethod>,
    pub energy: f64,
}

#[derive(Debug, Clone)]
pub struct ReasoningKernel {
    pub stage: usize,
    pub state: Vector,
}

/// Self-consistency 聚合结果。
#[derive(Debug, Clone)]
pub struct SelfConsistencyResult {
    pub majority_method: ReasoningMethod,
    pub consistency: f64,
    pub avg_confidence: f64,
    pub aggregated_state: Vector,
    pub n_samples: usize,
}