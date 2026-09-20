use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Vector = Vec<f64>;

/// 推理状态维度（与 L4 认知层共享的 kernel 维度契约）。
pub const KERNEL_DIM: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningMethod {
    Deductive,
    Inductive,
    Abductive,
    Analogical,
    FirstPrinciples,
    Recursive,
    Compositional,
    Adversarial,
    AutoFetch,
    KnowledgeRetrieval,
    GradientLearning,
    ArchitectureSearch,
    GpuCompute,
    DistributedConsensus,
    ExperienceDistill,
    EmergentAnalysis,
    SystemIntegration,
    EnsembleVoting,
    SelfImprovement,
    SparseRouting,
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

/// 推理输出包装器（统一 state_delta/confidence + trace_id + method）。
#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub state_delta: Vector,
    pub confidence: f64,
    pub trace_id: Option<String>,
    pub method: ReasoningMethod,
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

impl ReasoningKernel {
    pub fn new(stage: usize) -> Self {
        Self { stage: stage.min(EVOLUTION.len() - 1), state: vec![0.0; KERNEL_DIM] }
    }

    /// 真实推理：方法选择 → 多步状态演化 → 收敛度量 → 置信度。
    /// mode_values: optional E8Policy mode_values for method selection guidance.
    pub fn reason(
        &self,
        query: &[f64],
        context: Option<HashMap<String, Vector>>,
        mode_values: Option<&[f64]>,
    ) -> ReasoningOutput {
        let method = self.select_method(query, context.is_some(), mode_values);
        let steps = self.plan_steps(method);
        let mut state = self.state.clone();
        let mut intermediates: Vec<Vector> = Vec::with_capacity(steps);

        for t in 0..steps {
            let alpha = (t as f64 + 1.0) / steps as f64;
            for i in 0..state.len() {
                let q = query.get(i).copied().unwrap_or(0.0);
                let ctx = context
                    .as_ref()
                    .and_then(|m| m.values().next())
                    .and_then(|v| v.get(i))
                    .copied()
                    .unwrap_or(0.0);
                state[i] = state[i] * (1.0 - alpha * 0.3) + q * alpha * 0.6 + ctx * alpha * 0.3
                    + self.method_bias(method, i) * 0.1;
            }
            let norm: f64 = state.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
            for x in state.iter_mut() {
                *x /= norm;
            }
            intermediates.push(state.clone());
        }

        let convergence = if intermediates.len() >= 2 {
            let a = &intermediates[intermediates.len() - 2];
            let b = intermediates.last().expect("intermediates.len() >= 2");
            let delta: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
            1.0 - (delta / state.len() as f64).min(1.0)
        } else {
            0.5
        };

        let energy = query.iter().map(|x| x * x).sum::<f64>().sqrt();
        let query_scale = (energy / (query.len() as f64).sqrt()).min(1.0);
        let confidence = (convergence * 0.6 + query_scale * 0.4).clamp(0.05, 0.98);

        let trace_id = format!("kernel_{}_{}", self.stage, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos());

        ReasoningOutput {
            state_delta: state,
            confidence,
            trace_id: Some(trace_id),
            method,
        }
    }

    fn select_method(&self, query: &[f64], has_context: bool, mode_values: Option<&[f64]>) -> ReasoningMethod {
        let energy: f64 = query.iter().map(|x| x * x).sum();
        let active = query.iter().filter(|x| x.abs() > 0.5).count();
        let sparse = active < query.len() / 8;

        if let Some(mv) = mode_values {
            if let Some((best_idx, _)) = mv.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) {
                let hex_val = best_idx as u8;
                return self.hexagram_to_method(hex_val, self.stage, sparse, has_context);
            }
        }

        match self.stage {
            0..=2 => {
                if sparse { ReasoningMethod::KnowledgeRetrieval }
                else if has_context { ReasoningMethod::Inductive }
                else { ReasoningMethod::Deductive }
            }
            3..=4 => {
                if has_context { ReasoningMethod::Analogical }
                else { ReasoningMethod::Recursive }
            }
            5..=6 => {
                if energy > 2.0 { ReasoningMethod::Compositional }
                else { ReasoningMethod::Adversarial }
            }
            7..=8 => {
                if sparse { ReasoningMethod::AutoFetch }
                else { ReasoningMethod::FirstPrinciples }
            }
            9..=13 => {
                if has_context { ReasoningMethod::KnowledgeRetrieval }
                else { ReasoningMethod::GradientLearning }
            }
            14..=16 => {
                if energy > 2.0 { ReasoningMethod::SystemIntegration }
                else { ReasoningMethod::ExperienceDistill }
            }
            _ => {
                if has_context { ReasoningMethod::EnsembleVoting }
                else { ReasoningMethod::SelfImprovement }
            }
        }
    }

    fn hexagram_to_method(&self, hex: u8, _stage: usize, _sparse: bool, _has_context: bool) -> ReasoningMethod {
        let abstraction = (hex >> 5) & 1;
        let method_axis = (hex >> 3) & 1;
        let depth = (hex >> 2) & 1;
        match (method_axis, abstraction, depth) {
            (0, 0, 0) => ReasoningMethod::Deductive,
            (0, 0, 1) => ReasoningMethod::Inductive,
            (0, 1, 0) => ReasoningMethod::FirstPrinciples,
            (0, 1, 1) => ReasoningMethod::Analogical,
            (1, 0, 0) => ReasoningMethod::Recursive,
            (1, 0, 1) => ReasoningMethod::Compositional,
            (1, 1, 0) => ReasoningMethod::GradientLearning,
            (1, 1, 1) => ReasoningMethod::EnsembleVoting,
            _ => ReasoningMethod::Deductive,
        }
    }

    fn plan_steps(&self, method: ReasoningMethod) -> usize {
        match method {
            ReasoningMethod::Deductive => 4,
            ReasoningMethod::Inductive => 5,
            ReasoningMethod::Abductive => 5,
            ReasoningMethod::Analogical => 6,
            ReasoningMethod::FirstPrinciples => 8,
            ReasoningMethod::Recursive => 7,
            ReasoningMethod::Compositional => 8,
            ReasoningMethod::Adversarial => 6,
            ReasoningMethod::AutoFetch => 3,
            ReasoningMethod::KnowledgeRetrieval => 3,
            ReasoningMethod::GradientLearning => 10,
            ReasoningMethod::ArchitectureSearch => 9,
            ReasoningMethod::GpuCompute => 12,
            ReasoningMethod::DistributedConsensus => 8,
            ReasoningMethod::ExperienceDistill => 6,
            ReasoningMethod::EmergentAnalysis => 7,
            ReasoningMethod::SystemIntegration => 9,
            ReasoningMethod::EnsembleVoting => 5,
            ReasoningMethod::SelfImprovement => 10,
            ReasoningMethod::SparseRouting => 4,
        }
    }

    fn method_bias(&self, method: ReasoningMethod, i: usize) -> f64 {
        let phase = (i as f64 / 128.0) * std::f64::consts::TAU;
        match method {
            ReasoningMethod::Deductive => phase.sin() * 0.1,
            ReasoningMethod::Inductive => phase.cos() * 0.1,
            ReasoningMethod::Abductive => (phase * 2.0).sin() * 0.08,
            ReasoningMethod::Analogical => (phase * 0.5).cos() * 0.12,
            ReasoningMethod::FirstPrinciples => 0.05,
            ReasoningMethod::Recursive => (phase * 3.0).sin() * 0.06,
            ReasoningMethod::Compositional => (phase * 1.5).cos() * 0.1,
            ReasoningMethod::Adversarial => -(phase * 2.0).cos() * 0.08,
            ReasoningMethod::AutoFetch => 0.03,
            ReasoningMethod::KnowledgeRetrieval => (phase * 0.25).cos() * 0.15,
            ReasoningMethod::GradientLearning => (phase * 4.0).sin() * 0.05,
            ReasoningMethod::ArchitectureSearch => (phase * 0.75).cos() * 0.09,
            ReasoningMethod::GpuCompute => 0.04,
            ReasoningMethod::DistributedConsensus => (phase * 0.5).sin() * 0.07,
            ReasoningMethod::ExperienceDistill => (phase * 1.0).cos() * 0.11,
            ReasoningMethod::EmergentAnalysis => (phase * 2.5).sin() * 0.06,
            ReasoningMethod::SystemIntegration => (phase * 0.33).cos() * 0.13,
            ReasoningMethod::EnsembleVoting => 0.02,
            ReasoningMethod::SelfImprovement => (phase * 5.0).sin() * 0.04,
            ReasoningMethod::SparseRouting => 0.01,
        }
    }

    fn stage_methods(&self) -> Vec<ReasoningMethod> {
        match self.stage {
            0..=3 => vec![ReasoningMethod::Deductive, ReasoningMethod::Inductive, ReasoningMethod::KnowledgeRetrieval],
            4..=6 => vec![ReasoningMethod::Analogical, ReasoningMethod::Recursive, ReasoningMethod::Compositional],
            7..=9 => vec![ReasoningMethod::FirstPrinciples, ReasoningMethod::AutoFetch, ReasoningMethod::Adversarial],
            10..=13 => vec![ReasoningMethod::GradientLearning, ReasoningMethod::ArchitectureSearch, ReasoningMethod::GpuCompute],
            14..=16 => vec![ReasoningMethod::ExperienceDistill, ReasoningMethod::EmergentAnalysis, ReasoningMethod::SystemIntegration],
            _ => vec![ReasoningMethod::EnsembleVoting, ReasoningMethod::SelfImprovement, ReasoningMethod::SparseRouting],
        }
    }

    pub fn stats(&self) -> KernelStats {
        let active = self.stage_methods();
        KernelStats {
            stage: self.stage,
            label: EVOLUTION[self.stage].label.to_string(),
            state_dim: self.state.len(),
            total: active.len(),
            active,
            energy: self.state.iter().map(|x| x.abs()).sum::<f64>() / self.state.len().max(1) as f64,
        }
    }

    pub fn evolve_stage(&mut self) {
        self.stage = (self.stage + 1).min(EVOLUTION.len() - 1);
    }

    pub fn self_consistency(&self, query: &[f64], n_samples: usize) -> SelfConsistencyResult {
        let n = n_samples.max(1);
        let mut method_votes: HashMap<ReasoningMethod, usize> = HashMap::new();
        let mut confidences = Vec::with_capacity(n);
        let mut state_accum = vec![0.0; query.len()];
        for _ in 0..n {
            let out = self.reason(query, None, None);
            *method_votes.entry(out.method).or_insert(0) += 1;
            confidences.push(out.confidence);
            for (a, x) in state_accum.iter_mut().zip(out.state_delta.iter()) {
                *a += x;
            }
        }
        let (majority_method, majority_count) = method_votes
            .iter()
            .max_by_key(|(_, c)| **c)
            .map(|(m, c)| (*m, *c))
            .unwrap_or((ReasoningMethod::Deductive, 1));
        let consistency = majority_count as f64 / n as f64;
        let avg_confidence = confidences.iter().sum::<f64>() / n as f64;
        let norm: f64 = state_accum.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        for x in state_accum.iter_mut() {
            *x /= norm;
        }
        SelfConsistencyResult {
            majority_method,
            consistency,
            avg_confidence,
            aggregated_state: state_accum,
            n_samples: n,
        }
    }
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
