//! Shared consciousness / meta-cognition data types — L0 substrate.
//!
//! These pure-data types were previously defined in L5 Cognition, creating
//! L6→L5 bidirectional dependencies. Moving them to L0 breaks the cycle:
//! L5 re-exports them for backward compatibility, L6 imports from L0 directly.

use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// CoreSnapshot — 意识核心跨会话快照
// ============================================================

/// 分支成熟度持久化投影 — maturity 六布尔 + 真实计数 (self_test/module)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchMaturity {
    pub c0: bool,
    pub c1: bool,
    pub c2: bool,
    pub c3: bool,
    pub c4: bool,
    pub c5: bool,
    pub self_test_count: usize,
    pub module_count: usize,
}

/// 果实记录 — EvolutionFruit 的可持久化投影 (保留进化证据链)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FruitRecord {
    pub name: String,
    pub source_branch: String,
    pub description: String,
    pub produced_at_cycle: u64,
    pub quality: f64,
    pub claim: String,
    pub run_id: Option<String>,
    pub generation: u64,
}

/// 意识核心快照 — 可序列化的跨会话状态 (标量集合 + 果实记录, 不序列化整树)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreSnapshot {
    pub cycle: u64,
    pub resonance_cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance_active: bool,
    pub mars_system1_activations: u64,
    pub mars_system2_iterations: u64,
    pub mars_bridge_hits: u64,
    pub governance_compliance: f64,
    pub governance_constitution_count: usize,
    pub governance_fractal_depth: u64,
    pub weighted_fog_sum: f64,
    pub branch_health: HashMap<String, f64>,
    #[serde(default)]
    pub branch_fog: HashMap<String, f64>,
    #[serde(default)]
    pub branch_maturity: HashMap<String, BranchMaturity>,
    pub fruits: Vec<FruitRecord>,
    #[serde(default = "default_attention_source")]
    pub attention_source: String,
    #[serde(default)]
    pub recent_event_count: u64,
    #[serde(default)]
    pub shadow_instance_count: u64,
    #[serde(default)]
    pub compliance_execution_count: u64,
    #[serde(default)]
    pub constitution_check_count: u64,
    #[serde(default = "default_phi_source_tag")]
    pub phi_source_tag: String,
    #[serde(default)]
    pub phi_trend: Vec<f64>,
    #[serde(default)]
    pub coherence_trend: Vec<f64>,
}

fn default_attention_source() -> String {
    "auto".to_string()
}

const PHI_SOURCE_TAG_TREE: &str = "tree_snapshot_iit";

fn default_phi_source_tag() -> String {
    PHI_SOURCE_TAG_TREE.to_string()
}

// ============================================================
// AgentTrajectory / TrajectoryStep — 推理轨迹
// ============================================================

pub use crate::l0_substrate::nt_core_hex::ReasoningHexagram;
pub use crate::l0_substrate::nt_core_traits::SpecialistType;

/// One step in a multi-agent reasoning trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step_idx: usize,
    pub specialist: SpecialistType,
    pub e8_mode: ReasoningHexagram,
    pub action: String,
    pub input: String,
    pub output: String,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub external_reward: Option<f64>,
}

/// A full multi-step reasoning episode (trajectory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrajectory {
    pub trajectory_id: u64,
    pub task: String,
    pub steps: Vec<TrajectoryStep>,
    pub outcome_reward: Option<f64>,
    pub completed: bool,
    pub total_duration_ms: Option<u64>,
}

impl AgentTrajectory {
    pub fn new(trajectory_id: u64, task: String) -> Self {
        Self {
            trajectory_id,
            task,
            steps: Vec::new(),
            outcome_reward: None,
            completed: false,
            total_duration_ms: None,
        }
    }

    pub fn push(&mut self, step: TrajectoryStep) {
        self.steps.push(step);
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

// ============================================================
// AwarenessReport — 能力差距感知
// ============================================================

/// Describes a detected capability gap
#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub dimension: String,
    pub current: f64,
    pub required: f64,
    pub gap: f64,
    pub severity: GapSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GapSeverity {
    Critical,
    Significant,
    Moderate,
    Negligible,
}

/// Awareness report for current state
#[derive(Debug, Clone)]
pub struct AwarenessReport {
    pub gaps: Vec<CapabilityGap>,
    pub total_gap: f64,
    pub critical_count: u32,
    pub significant_count: u32,
    pub recommended_focus: Vec<String>,
    pub overall_health: f64,
}

// ============================================================
// IIT Phi — 集成信息论
// ============================================================

/// 共振宽度 σ — 维度间耦合范围
pub const PHI_RESONANCE_SIGMA: f64 = 0.15;

/// 最小 Φ 阈值 — 低于此值认为无集成
pub const PHI_MIN_THRESHOLD: f64 = 0.01;

/// Φ 历史窗口 (趋势分析)
pub const PHI_HISTORY_WINDOW: usize = 20;

/// IIT Φ 分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiReport {
    pub phi: f64,
    pub phi_raw: f64,
    pub total_resonance: f64,
    pub state_energy: f64,
    pub effective_dims: usize,
    pub max_resonance_pair: (usize, usize),
    pub phi_trend: f64,
    pub is_conscious_like: bool,
}

/// IIT Φ 计算器 — 量化 E8 64 卦空间的集成信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IITPhiCalculator {
    pub sigma: f64,
    pub phi_history: Vec<f64>,
}

impl Default for IITPhiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl IITPhiCalculator {
    pub fn new() -> Self {
        Self {
            sigma: PHI_RESONANCE_SIGMA,
            phi_history: Vec::with_capacity(PHI_HISTORY_WINDOW),
        }
    }

    pub fn with_sigma(mut self, sigma: f64) -> Self {
        self.sigma = sigma;
        self
    }

    /// 计算共振矩阵 R[i][j] = exp(-(x_i - x_j)² / σ²)
    pub fn resonance_matrix(&self, state: &[f64]) -> Vec<Vec<f64>> {
        let n = state.len().min(64);
        let mut r = vec![vec![0.0; n]; n];
        let sigma = if self.sigma.abs() < 1e-12 { 1.0 } else { self.sigma };
        let sigma2 = sigma * sigma;
        for i in 0..n {
            for j in 0..n {
                let diff = state[i] - state[j];
                r[i][j] = (-diff * diff / sigma2).exp();
            }
        }
        r
    }

    /// 计算集成信息 Φ
    pub fn compute_phi(&self, state: &[f64]) -> PhiReport {
        let clean_state: Vec<f64> = state
            .iter()
            .map(|&v| if v.is_finite() { v } else { 0.0 })
            .collect();
        let n = clean_state.len().min(64);
        if n == 0 {
            return PhiReport {
                phi: 0.0,
                phi_raw: 0.0,
                total_resonance: 0.0,
                state_energy: 0.0,
                effective_dims: 0,
                max_resonance_pair: (0, 0),
                phi_trend: 0.0,
                is_conscious_like: false,
            };
        }
        let r = self.resonance_matrix(&clean_state);
        let mean: f64 = clean_state.iter().take(n).sum::<f64>() / n as f64;
        let centered: Vec<f64> = clean_state.iter().take(n).map(|&v| v - mean).collect();
        let state_energy: f64 = centered.iter().map(|v| v * v).sum();
        let mut total_resonance = 0.0;
        let mut max_res = 0.0;
        let mut max_pair = (0, 1);
        for i in 0..n {
            for j in (i + 1)..n {
                let res = r[i][j] * centered[i].abs() * centered[j].abs();
                total_resonance += res;
                if res > max_res {
                    max_res = res;
                    max_pair = (i, j);
                }
            }
        }
        let intensity = if state_energy > 1e-12 {
            (total_resonance / state_energy).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let mut rho_num = 0.0;
        for i in 1..n {
            rho_num += centered[i] * centered[i - 1];
        }
        let rho = if state_energy > 1e-12 {
            (rho_num / state_energy).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let phi_raw = intensity * rho;
        let phi = phi_raw.clamp(0.0, 1.0);
        let mut history = self.phi_history.clone();
        history.push(phi);
        if history.len() > PHI_HISTORY_WINDOW {
            history.remove(0);
        }
        let phi_trend = if history.len() >= 2 {
            history.last().unwrap() - history[history.len() - 2]
        } else {
            0.0
        };
        PhiReport {
            phi,
            phi_raw,
            total_resonance,
            state_energy,
            effective_dims: n,
            max_resonance_pair: max_pair,
            phi_trend,
            is_conscious_like: phi > 0.33,
        }
    }

    /// 记录 Φ 值到历史
    pub fn record(&mut self, phi: f64) {
        self.phi_history.push(phi);
        if self.phi_history.len() > PHI_HISTORY_WINDOW {
            self.phi_history.remove(0);
        }
    }

    /// 完整周期: 计算 + 记录 + 报告
    pub fn analyze_state(&mut self, state: &[f64]) -> PhiReport {
        let report = self.compute_phi(state);
        self.record(report.phi);
        report
    }
}

// ============================================================
// EffortTier — 努力分层
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EffortTier {
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

impl EffortTier {
    pub fn from_difficulty(difficulty: f64, task_length: usize) -> Self {
        let length_factor = (task_length as f64 / 500.0).min(1.0);
        // 2026-09-27 修复: 原式上限 = 0.7*1 + 0.3*1 = 0.7, 而 Max 档要求 >= 0.8
        // → EffortTier::Max 是死代码。按 difficulty 归一化, 使五档都可达。
        let combined = (difficulty * 0.7 + length_factor * 0.3) / 0.7;
        if combined < 0.2 {
            EffortTier::Low
        } else if combined < 0.4 {
            EffortTier::Medium
        } else if combined < 0.6 {
            EffortTier::High
        } else if combined < 0.8 {
            EffortTier::XHigh
        } else {
            EffortTier::Max
        }
    }

    pub fn rollout_depth(&self) -> usize {
        match self {
            EffortTier::Low => 2,
            EffortTier::Medium => 4,
            EffortTier::High => 8,
            EffortTier::XHigh => 16,
            EffortTier::Max => 32,
        }
    }
}

// ============================================================
// OscillatorNetwork — Kuramoto 振荡绑定
// ============================================================

pub const DEFAULT_NATURAL_FREQ: f64 = 1.0;
pub const DEFAULT_COUPLING_K: f64 = 0.5;
pub const SYNCHRONIZE_STEPS: usize = 20;
pub const DT: f64 = 0.05;

/// Kuramoto 振荡器 — 每个 specialist module 一个
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KuramotoOscillator {
    pub phase: f64,
    pub natural_freq: f64,
    pub amplitude: f64,
}

impl KuramotoOscillator {
    pub fn new(natural_freq: f64) -> Self {
        Self {
            phase: random::<f64>() * 2.0 * std::f64::consts::PI,
            natural_freq,
            amplitude: 0.5,
        }
    }
}

/// 耦合振荡器网络 — 意识绑定的物理基础
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillatorNetwork {
    pub oscillators: Vec<KuramotoOscillator>,
    pub coupling_k: f64,
    pub coupling_matrix: Option<Vec<Vec<f64>>>,
}

impl OscillatorNetwork {
    pub fn new(n: usize) -> Self {
        let oscillators: Vec<KuramotoOscillator> = (0..n)
            .map(|i| {
                let freq = 0.5 + (i as f64 / n as f64) * 1.5;
                KuramotoOscillator::new(freq)
            })
            .collect();
        Self {
            oscillators,
            coupling_k: DEFAULT_COUPLING_K,
            coupling_matrix: None,
        }
    }

    pub fn step(&mut self) {
        let n = self.oscillators.len() as f64;
        let phases: Vec<f64> = self.oscillators.iter().map(|o| o.phase).collect();
        for i in 0..self.oscillators.len() {
            let mut sync_sum = 0.0;
            for j in 0..self.oscillators.len() {
                if i == j {
                    continue;
                }
                let k_ij = self
                    .coupling_matrix
                    .as_ref()
                    .and_then(|m| m.get(i))
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(self.coupling_k);
                sync_sum += k_ij * (phases[j] - phases[i]).sin();
            }
            let dtheta = self.oscillators[i].natural_freq + sync_sum / n;
            self.oscillators[i].phase =
                (self.oscillators[i].phase + DT * dtheta) % (2.0 * std::f64::consts::PI);
        }
    }

    /// 运行多步同步
    pub fn synchronize(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn coherence(&self) -> f64 {
        let n = self.oscillators.len() as f64;
        let sum_sin: f64 = self.oscillators.iter().map(|o| o.phase.sin()).sum();
        let sum_cos: f64 = self.oscillators.iter().map(|o| o.phase.cos()).sum();
        ((sum_sin * sum_sin + sum_cos * sum_cos).sqrt()) / n
    }

    /// 相位相干度 R = |Σ e^{iθ}| / N ∈ [0, 1]
    pub fn phase_coherence(&self) -> f64 {
        let n = self.oscillators.len() as f64;
        let (sum_cos, sum_sin): (f64, f64) = self
            .oscillators
            .iter()
            .map(|o| (o.phase.cos(), o.phase.sin()))
            .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
        (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n.max(1.0)
    }

    /// 平均相位 (序参量方向)
    pub fn mean_phase(&self) -> f64 {
        let (sum_cos, sum_sin): (f64, f64) = self
            .oscillators
            .iter()
            .map(|o| (o.phase.cos(), o.phase.sin()))
            .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
        sum_sin.atan2(sum_cos)
    }

    /// 用 salience 更新振幅: 高 salience → 高振幅
    pub fn update_amplitudes(&mut self, saliences: &[f64]) {
        let n = self.oscillators.len().min(saliences.len());
        for i in 0..n {
            self.oscillators[i].amplitude = saliences[i].clamp(0.0, 1.0);
        }
    }

    /// 同步后广播权重: 高相干模块获得更高广播权重
    pub fn broadcast_weights(&self) -> Vec<f64> {
        let mean_ph = self.mean_phase();
        self.oscillators
            .iter()
            .map(|o| {
                let phase_diff = (o.phase - mean_ph).abs();
                let sync_factor = (-phase_diff * 4.0).exp();
                o.amplitude * sync_factor
            })
            .collect()
    }

    /// 检测是否达到意识绑定阈值 (R > 0.7)
    pub fn is_bound(&self) -> bool {
        self.phase_coherence() > 0.7
    }
}

// ============================================================
// Quantum Fusion — 量子态检测与最优融合
// ============================================================

/// 单个检测源的输出 — 叠加态的一个分量。
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumSignal {
    pub value: f64,
    pub confidence: f64,
    pub source: String,
}

impl QuantumSignal {
    pub fn new(value: f64, confidence: f64, source: impl Into<String>) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            source: source.into(),
        }
    }
}

/// 融合超参数。
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumFusionConfig {
    pub confidence_floor: f64,
    pub high_entanglement: f64,
    pub conflict_threshold: f64,
}

impl Default for QuantumFusionConfig {
    fn default() -> Self {
        Self {
            confidence_floor: 0.2,
            high_entanglement: 0.75,
            conflict_threshold: 0.6,
        }
    }
}

/// 最优融合坍缩后的单一高可靠信号。
#[derive(Debug, Clone, PartialEq)]
pub struct FusedSignal {
    pub value: f64,
    pub confidence: f64,
    pub entropy: f64,
    pub entanglement: f64,
    pub dominant_source: Option<String>,
    pub fused_count: usize,
}

/// 多源信号的叠加态，提供纠缠度量与最优融合坍缩。
#[derive(Debug, Clone, Default)]
pub struct QuantumSuperposition {
    signals: Vec<QuantumSignal>,
}

impl QuantumSuperposition {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }

    pub fn push(&mut self, sig: QuantumSignal) {
        self.signals.push(sig);
    }

    pub fn from_signals(signals: impl IntoIterator<Item = QuantumSignal>) -> Self {
        Self {
            signals: signals.into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.signals.len()
    }

    pub fn signals(&self) -> &[QuantumSignal] {
        &self.signals
    }

    pub fn entanglement_dispersion(&self) -> f64 {
        if self.signals.is_empty() {
            return 0.0;
        }
        let total_conf: f64 = self.signals.iter().map(|s| s.confidence).sum();
        if total_conf <= 0.0 {
            return 0.0;
        }
        let mean: f64 = self
            .signals
            .iter()
            .map(|s| s.value * s.confidence)
            .sum::<f64>()
            / total_conf;
        let variance: f64 = self
            .signals
            .iter()
            .map(|s| s.confidence * (s.value - mean).powi(2))
            .sum::<f64>()
            / total_conf;
        (variance.sqrt()).clamp(0.0, 1.0)
    }

    pub fn entanglement(&self) -> f64 {
        1.0 - self.entanglement_dispersion()
    }

    pub fn entropy(&self) -> f64 {
        let effective: Vec<f64> = self.signals.iter().map(|s| s.value).collect();
        if effective.is_empty() {
            return 0.0;
        }
        const BUCKETS: usize = 8;
        let mut counts = [0usize; BUCKETS];
        for v in &effective {
            let idx = ((v * BUCKETS as f64) as usize).min(BUCKETS - 1);
            counts[idx] += 1;
        }
        let n = effective.len() as f64;
        let mut entropy = 0.0f64;
        for c in counts.iter().filter(|&&c| c > 0) {
            let p = *c as f64 / n;
            entropy -= p * p.ln();
        }
        (entropy / (BUCKETS as f64).ln()).clamp(0.0, 1.0)
    }

    pub fn fuse(&self) -> FusedSignal {
        let config = QuantumFusionConfig::default();
        let filtered: Vec<&QuantumSignal> = self
            .signals
            .iter()
            .filter(|s| s.confidence >= config.confidence_floor)
            .collect();
        if filtered.is_empty() {
            return FusedSignal {
                value: 0.0,
                confidence: 0.0,
                entropy: 1.0,
                entanglement: 0.0,
                dominant_source: None,
                fused_count: 0,
            };
        }
        let total_conf: f64 = filtered.iter().map(|s| s.confidence).sum();
        let base_value: f64 = filtered
            .iter()
            .map(|s| s.value * s.confidence)
            .sum::<f64>()
            / total_conf;
        let entanglement = self.entanglement();
        let confidence = if entanglement >= config.high_entanglement {
            (total_conf / filtered.len() as f64 * 1.1).min(1.0)
        } else if entanglement <= config.conflict_threshold {
            (total_conf / filtered.len() as f64 * 0.7).max(0.0)
        } else {
            total_conf / filtered.len() as f64
        };
        let dominant = filtered
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|s| s.source.clone());
        FusedSignal {
            value: base_value,
            confidence,
            entropy: self.entropy(),
            entanglement,
            dominant_source: dominant,
            fused_count: filtered.len(),
        }
    }
}

// ============================================================
// Narrative Types — 叙事/节奏共享类型
// ============================================================

/// 节段类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SegmentType {
    Setup,
    Conflict,
    Climax,
    Transition,
}

/// 节段数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentData {
    pub r#type: SegmentType,
    pub base_length: f32,
    pub content_priority: f32,
    pub is_core_scuang: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_maturity_default() {
        let bm = BranchMaturity::default();
        assert!(!bm.c0);
        assert!(!bm.c5);
        assert_eq!(bm.self_test_count, 0);
    }

    #[test]
    fn fruit_record_default() {
        let fr = FruitRecord::default();
        assert!(fr.name.is_empty());
        assert_eq!(fr.quality, 0.0);
    }

    #[test]
    fn core_snapshot_default() {
        let cs = CoreSnapshot::default();
        assert_eq!(cs.cycle, 0);
        assert_eq!(cs.phi, 0.0);
        assert!(!cs.gwt_resonance_active);
    }

    #[test]
    fn agent_trajectory_new() {
        let mut traj = AgentTrajectory::new(1, "test task".into());
        assert!(traj.is_empty());
        assert_eq!(traj.len(), 0);
        traj.push(TrajectoryStep {
            step_idx: 0,
            specialist: SpecialistType::Planner,
            e8_mode: crate::l0_substrate::nt_core_hex::ReasoningHexagram(0),
            action: "plan".into(),
            input: "x".into(),
            output: "y".into(),
            duration_ms: Some(10),
            success: true,
            external_reward: None,
        });
        assert_eq!(traj.len(), 1);
    }

    #[test]
    fn effort_tier_from_difficulty() {
        assert_eq!(EffortTier::from_difficulty(0.0, 0), EffortTier::Low);
        assert_eq!(EffortTier::from_difficulty(0.3, 0), EffortTier::Medium);
        assert_eq!(EffortTier::from_difficulty(0.5, 0), EffortTier::High);
        assert_eq!(EffortTier::from_difficulty(0.7, 0), EffortTier::XHigh);
        assert_eq!(EffortTier::from_difficulty(1.0, 0), EffortTier::Max);
    }

    #[test]
    fn effort_tier_rollout_depth() {
        assert_eq!(EffortTier::Low.rollout_depth(), 2);
        assert_eq!(EffortTier::Medium.rollout_depth(), 4);
        assert_eq!(EffortTier::High.rollout_depth(), 8);
        assert_eq!(EffortTier::XHigh.rollout_depth(), 16);
        assert_eq!(EffortTier::Max.rollout_depth(), 32);
    }

    #[test]
    fn iit_phi_calculator_new() {
        let calc = IITPhiCalculator::default();
        assert_eq!(calc.sigma, PHI_RESONANCE_SIGMA);
        assert!(calc.phi_history.is_empty());
    }

    #[test]
    fn iit_phi_calculator_compute_phi_empty() {
        let calc = IITPhiCalculator::new();
        let report = calc.compute_phi(&[]);
        assert_eq!(report.phi, 0.0);
        assert_eq!(report.effective_dims, 0);
        assert!(!report.is_conscious_like);
    }

    #[test]
    fn iit_phi_calculator_compute_phi_uniform() {
        let calc = IITPhiCalculator::new();
        let state = vec![1.0; 32];
        let report = calc.compute_phi(&state);
        assert_eq!(report.effective_dims, 32);
        assert!(report.phi >= 0.0);
    }

    #[test]
    fn iit_phi_calculator_record() {
        let mut calc = IITPhiCalculator::new();
        calc.record(0.5);
        calc.record(0.6);
        assert_eq!(calc.phi_history.len(), 2);
    }

    #[test]
    fn iit_phi_calculator_history_window() {
        let mut calc = IITPhiCalculator::new();
        for _ in 0..30 {
            calc.record(0.5);
        }
        assert!(calc.phi_history.len() <= PHI_HISTORY_WINDOW);
    }

    #[test]
    fn oscillator_network_new() {
        let net = OscillatorNetwork::new(4);
        assert_eq!(net.oscillators.len(), 4);
        assert_eq!(net.coupling_k, DEFAULT_COUPLING_K);
    }

    #[test]
    fn oscillator_network_coherence_range() {
        let net = OscillatorNetwork::new(8);
        let c = net.coherence();
        assert!(c >= 0.0 && c <= 1.0);
    }

    #[test]
    fn oscillator_network_phase_coherence_range() {
        let net = OscillatorNetwork::new(8);
        let pc = net.phase_coherence();
        assert!(pc >= 0.0 && pc <= 1.0);
    }

    #[test]
    fn oscillator_network_step() {
        let mut net = OscillatorNetwork::new(4);
        let phases_before: Vec<f64> = net.oscillators.iter().map(|o| o.phase).collect();
        net.step();
        let phases_after: Vec<f64> = net.oscillators.iter().map(|o| o.phase).collect();
        assert_ne!(phases_before, phases_after);
    }

    #[test]
    fn oscillator_network_synchronize() {
        let mut net = OscillatorNetwork::new(8);
        net.synchronize(SYNCHRONIZE_STEPS);
        let pc = net.phase_coherence();
        assert!(pc >= 0.0);
    }

    #[test]
    fn oscillator_network_update_amplitudes() {
        let mut net = OscillatorNetwork::new(4);
        net.update_amplitudes(&[0.1, 0.5, 0.9, 1.0]);
        assert!((net.oscillators[0].amplitude - 0.1).abs() < 1e-6);
        assert!((net.oscillators[2].amplitude - 0.9).abs() < 1e-6);
    }

    #[test]
    fn oscillator_network_broadcast_weights() {
        let net = OscillatorNetwork::new(4);
        let weights = net.broadcast_weights();
        assert_eq!(weights.len(), 4);
        for w in &weights {
            assert!(*w >= 0.0);
        }
    }

    #[test]
    fn quantum_signal_new() {
        let sig = QuantumSignal::new(0.8, 0.9, "sensor1");
        assert_eq!(sig.source, "sensor1");
        assert!(sig.confidence <= 1.0);
    }

    #[test]
    fn quantum_signal_clamps() {
        let sig = QuantumSignal::new(1.5, -0.1, "x");
        assert_eq!(sig.value, 1.0);
        assert_eq!(sig.confidence, 0.0);
    }

    #[test]
    fn quantum_superposition_empty() {
        let sp = QuantumSuperposition::new();
        assert!(sp.is_empty());
        assert_eq!(sp.len(), 0);
    }

    #[test]
    fn quantum_superposition_push() {
        let mut sp = QuantumSuperposition::new();
        sp.push(QuantumSignal::new(0.5, 0.8, "a"));
        sp.push(QuantumSignal::new(0.5, 0.8, "b"));
        assert_eq!(sp.len(), 2);
    }

    #[test]
    fn quantum_superposition_entanglement() {
        let mut sp = QuantumSuperposition::new();
        sp.push(QuantumSignal::new(0.5, 0.9, "a"));
        sp.push(QuantumSignal::new(0.5, 0.9, "b"));
        let e = sp.entanglement();
        assert!((e - 1.0).abs() < 0.01, "expected ~1.0, got {}", e);
    }

    #[test]
    fn quantum_superposition_entropy_range() {
        let mut sp = QuantumSuperposition::new();
        sp.push(QuantumSignal::new(0.1, 0.5, "a"));
        sp.push(QuantumSignal::new(0.9, 0.5, "b"));
        let ent = sp.entropy();
        assert!(ent >= 0.0 && ent <= 1.0);
    }

    #[test]
    fn quantum_superposition_fuse_empty() {
        let sp = QuantumSuperposition::new();
        let fused = sp.fuse();
        assert_eq!(fused.fused_count, 0);
        assert_eq!(fused.value, 0.0);
    }

    #[test]
    fn quantum_superposition_fuse_with_signals() {
        let mut sp = QuantumSuperposition::new();
        sp.push(QuantumSignal::new(0.5, 0.8, "a"));
        sp.push(QuantumSignal::new(0.6, 0.9, "b"));
        let fused = sp.fuse();
        assert_eq!(fused.fused_count, 2);
        assert!(fused.value > 0.0);
        assert!(fused.dominant_source.is_some());
    }

    #[test]
    fn segment_type_serde() {
        let s = SegmentType::Climax;
        let json = serde_json::to_string(&s).unwrap();
        let back: SegmentType = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn gap_severity_eq() {
        assert_eq!(GapSeverity::Critical, GapSeverity::Critical);
        assert_ne!(GapSeverity::Critical, GapSeverity::Negligible);
    }
}
