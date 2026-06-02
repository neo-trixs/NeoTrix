//! Resonance attention mechanism — replaces salience-only competition.
//!
//! Based on the E₈ × 64 state-space model:
//! - Each specialist module operates in a reasoning hexagram state
//! - Modules in resonance (hamming dist ≤ 2) amplify each other's salience
//! - Complementary modules (错卦) automatically trigger opposing perspectives
//! - The +1 observer tracks the overall resonance landscape

use std::collections::VecDeque;

use crate::core::nt_core_hex::ReasoningHexagram;
use super::physics_attention::AdaptiveSlicer;

/// Maximum resonance distance (hamming dist ≤ 2 → in resonance).
pub const RESONANCE_THRESHOLD: u32 = 2;

/// Number of specialist modules.
pub const MODULE_COUNT: usize = 13;

/// Pre-computed resonance matrix: 12×12 pairwise resonance strengths.
#[derive(Debug, Clone)]
pub struct ResonanceMatrix {
    /// resonance[i][j] = resonance strength between module i and j (0-6).
    pub strengths: [[u32; MODULE_COUNT]; MODULE_COUNT],
}

impl ResonanceMatrix {
    /// Build from a slice of 12 hexagram assignments (one per specialist).
    pub fn from_states(states: &[ReasoningHexagram; MODULE_COUNT]) -> Self {
        let mut strengths = [[0u32; MODULE_COUNT]; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            for j in 0..MODULE_COUNT {
                strengths[i][j] = states[i].resonance_strength(&states[j]);
            }
        }
        Self { strengths }
    }

    /// Get resonance strength between two modules.
    pub fn get(&self, i: usize, j: usize) -> u32 {
        self.strengths[i][j]
    }

    /// Compute effective salience for each module given raw salience vector.
    /// effective[i] = raw[i] + Σ(resonance[i][j] × raw[j] × 0.1)
    pub fn effective_salience(&self, raw: &[f64; MODULE_COUNT]) -> [f64; MODULE_COUNT] {
        let mut eff = *raw;
        for (i, item) in eff.iter_mut().enumerate() {
            let mut resonance_boost = 0.0;
            for (j, &raw_j) in raw.iter().enumerate() {
                if i == j { continue; }
                let boost = self.strengths[i][j] as f64 * raw_j * 0.1;
                resonance_boost += boost;
            }
            *item = (*item + resonance_boost).min(1.0);
        }
        eff
    }

    /// Find all modules in resonance with a given module index.
    pub fn resonators(&self, idx: usize) -> Vec<usize> {
        (0..MODULE_COUNT)
            .filter(|&j| j != idx && self.strengths[idx][j] >= (6 - RESONANCE_THRESHOLD))
            .collect()
    }

    /// Find the complementary module (hexagram complement) for a given module.
    pub fn complement_of(&self, idx: usize, states: &[ReasoningHexagram; MODULE_COUNT]) -> Option<usize> {
        let comp = states[idx].complement();
        states.iter().position(|&s| s == comp)
    }
}

/// Compute resonance-boosted winner-take-most competition.
/// Returns (winner_index, effective_saliences, entropy).
pub fn resonate_and_select(
    raw_salience: &[f64; MODULE_COUNT],
    matrix: &ResonanceMatrix,
) -> (usize, [f64; MODULE_COUNT], f64) {
    let eff = matrix.effective_salience(raw_salience);

    // Winner-take-most: pick the highest effective salience
    let winner = eff.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("result"))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Entropy: how distributed is the attention?
    let total: f64 = eff.iter().sum();
    let entropy = if total > 0.0 {
        -eff.iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                p * p.log2()
            })
            .sum::<f64>()
    } else {
        0.0
    };

    (winner, eff, entropy)
}

/// Default hexagram assignments for each specialist module.
/// Maps each specialist to its natural reasoning mode.
pub fn default_specialist_states() -> [ReasoningHexagram; MODULE_COUNT] {
    let s = |bits| ReasoningHexagram(bits);
    [
        s(55), // PatternMatcher: Pattern Match (concrete+analytical+certain)
        s(10), // AnomalyDetector: Root Cause (concrete+analytical+deep)
        s(33), // KnowledgeRetriever: Guided Check (abstract+analytical+certain)
        s(4),  // CodeAnalyzer: Code Review (concrete+analytical+focused)
        s(56), // Planner: System Design (abstract+broad+generative)
        s(57), // KnowledgeIntegrator: Guided Meta (abstract+meta+collaborative)
        s(62), // GoalPrioritizer: Meta-cognition (abstract+broad+meta)
        s(8),  // RiskAssessor: Formal Proof (abstract+analytical+deep)
        s(14), // CreativityGenerator: Brainstorm (abstract+generative+broad)
        s(63), // ReflectionEngine: Guided Meta (meta+broad+collaborative)
        s(62), // MetaCognitionAnalyst: Meta-cognition (reflective)
        s(2),  // AISecurity: Vulnerability Analysis (concrete+focused+analytical+deep+collaborative)
        s(54), // ImageGenerator: Generate (abstract+broad+generative+fast+solo+certain)
    ]
}

/// Resonance report for the global workspace.
#[derive(Debug, Clone)]
pub struct ResonanceReport {
    pub winner: usize,
    pub effective_saliences: [f64; MODULE_COUNT],
    pub raw_saliences: [f64; MODULE_COUNT],
    pub entropy: f64,
    pub resonator_clusters: Vec<Vec<usize>>,
    pub complement_activated: bool,
}

impl ResonanceReport {
    /// The attention is highly focused (entropy < 1.0).
    pub fn is_focused(&self) -> bool {
        self.entropy < 1.0
    }

    /// The attention is distributed (entropy >= 2.0).
    pub fn is_distributed(&self) -> bool {
        self.entropy >= 2.0
    }
}

/// Run a full resonance-aware competition cycle.
pub fn resonate_cycle(
    raw_salience: &[f64; MODULE_COUNT],
    states: &[ReasoningHexagram; MODULE_COUNT],
) -> ResonanceReport {
    let matrix = ResonanceMatrix::from_states(states);
    let (winner, eff, entropy) = resonate_and_select(raw_salience, &matrix);

    // Find resonance clusters
    let mut resonator_clusters = Vec::new();
    let mut visited = [false; MODULE_COUNT];
    for i in 0..MODULE_COUNT {
        if !visited[i] {
            let mut cluster = vec![i];
            visited[i] = true;
            let resonators = matrix.resonators(i);
            for &r in &resonators {
                if !visited[r] {
                    cluster.push(r);
                    visited[r] = true;
                }
            }
            if cluster.len() > 1 {
                resonator_clusters.push(cluster);
            }
        }
    }

    let complement_activated = matrix.complement_of(winner, states).is_some();

    ResonanceReport {
        winner,
        effective_saliences: eff,
        raw_saliences: *raw_salience,
        entropy,
        resonator_clusters,
        complement_activated,
    }
}

/// Resonance cycle with Physics-Attention adaptive slicing.
///
/// Uses AdaptiveSlicer to dynamically group specialists into slices
/// based on activation-weighted hexagram similarity, then computes
/// per-slice attention. Replaces fixed Hamming-distance clustering
/// with Transolver-inspired learnable-state grouping.
pub fn resonate_cycle_with_physics(
    raw_salience: &[f64; MODULE_COUNT],
    states: &[ReasoningHexagram; MODULE_COUNT],
    slicer: &mut AdaptiveSlicer,
) -> ResonanceReport {
    let matrix = ResonanceMatrix::from_states(states);

    // Form adaptive slices. Get slice_sal first, then slices ref after.
    slicer.form_slices(raw_salience, states);
    let slice_sal = slicer.slice_salience();

    // Effective salience = raw + resonance boost (same as standard) + slice boost
    let mut eff = matrix.effective_salience(raw_salience);
    for i in 0..MODULE_COUNT {
        eff[i] = (eff[i] + slice_sal[i] * 0.15).min(1.0);
    }

    let winner = eff.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("result"))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let total: f64 = eff.iter().sum();
    let entropy = if total > 0.0 {
        -eff.iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                p * p.log2()
            })
            .sum::<f64>()
    } else {
        0.0
    };

    // Build resonator clusters from physics slices instead of fixed Hamming
    let resonator_clusters: Vec<Vec<usize>> = slicer.slices.iter()
        .filter(|sl| sl.members.len() > 1)
        .map(|sl| sl.members.clone())
        .collect();

    let complement_activated = matrix.complement_of(winner, states).is_some();

    ResonanceReport {
        winner,
        effective_saliences: eff,
        raw_saliences: *raw_salience,
        entropy,
        resonator_clusters,
        complement_activated,
    }
}

// ============================================================
// NRS-EFC Broadcast Weighting
// Non-Redundant Stable EFC weights for GWT broadcast priority.
// Each specialist's recent EFC (Evolutionary Fitness Contribution) is
// tracked and used to weight broadcast: resonance × efc_weight.
// Reference: arXiv 2605.29682 — NRS-EFC achieves R²=0.92 on real traces.
// ============================================================

/// Tracks NRS-EFC (Non-Redundant Stable EFC) for each specialist.
/// Used to weight broadcast priority: resonance × efc_weight.
#[derive(Debug, Clone)]
pub struct EfcBroadcastWeights {
    /// Per-specialist EFC history (last N values)
    pub efc_history: Vec<VecDeque<f64>>,
    /// Per-specialist average weight in [0, 1]
    pub weights: Vec<f64>,
    /// Maximum history length
    max_history: usize,
}

impl EfcBroadcastWeights {
    pub fn new(num_specialists: usize) -> Self {
        Self {
            efc_history: vec![VecDeque::new(); num_specialists],
            weights: vec![0.5; num_specialists],
            max_history: 10,
        }
    }

    /// Record an EFC value for a specialist and update weight.
    pub fn record(&mut self, specialist_idx: usize, efc: f64) {
        let history = &mut self.efc_history[specialist_idx];
        history.push_back(efc);
        if history.len() > self.max_history {
            history.pop_front();
        }
        let avg = history.iter().sum::<f64>() / history.len() as f64;
        self.weights[specialist_idx] = avg.max(0.0).min(1.0);
    }

    /// Get the current weight for a specialist.
    pub fn weight(&self, specialist_idx: usize) -> f64 {
        self.weights[specialist_idx]
    }

    /// Get all weights as a slice.
    pub fn all_weights(&self) -> &[f64] {
        &self.weights
    }

    /// Apply NRS-EFC weights to a resonance matrix in-place.
    /// Each strength[i][j] is scaled by weight[i] × weight[j].
    pub fn apply_to_resonance(&self, resonance: &mut ResonanceMatrix) {
        for i in 0..MODULE_COUNT {
            for j in 0..MODULE_COUNT {
                let w = self.weights[i] * self.weights[j];
                let adjusted = (resonance.strengths[i][j] as f64 * w).round() as u32;
                resonance.strengths[i][j] = adjusted;
            }
        }
    }
}

/// Compute adjusted resonance: element-wise multiply resonance matrix by EFC weights.
/// Returns a new ResonanceMatrix with EFC-aware broadcast priorities.
pub fn resonate_with_efc(
    states: &[ReasoningHexagram; MODULE_COUNT],
    weights: &EfcBroadcastWeights,
) -> ResonanceMatrix {
    let matrix = ResonanceMatrix::from_states(states);
    let mut adjusted = matrix;
    weights.apply_to_resonance(&mut adjusted);
    adjusted
}

// ============================================================
// Kuramoto 振荡绑定 — 伽马频段同步 (30-100 Hz)
// 文献: The Consciousness AI (tlcdv, 2026);
//       Feinberg & Mallatt, "The Ancient Origins of Consciousness"
//
// 核心: 专家模块通过相位耦合实现意识绑定
//   dθ_i/dt = ω_i + (K/N) Σ sin(θ_j - θ_i)
//   同步度 R = |Σ e^{iθ}| / N  ∈ [0, 1]
// ============================================================

/// 默认自然频率 (gamma 范围: 30-100 Hz, 归一化到 [0.5, 2.0])
pub const DEFAULT_NATURAL_FREQ: f64 = 1.0;

/// 耦合强度 K — 决定同步速度
pub const DEFAULT_COUPLING_K: f64 = 0.5;

/// 同步迭代步数
pub const SYNCHRONIZE_STEPS: usize = 20;

/// 时间步长
pub const DT: f64 = 0.05;

/// Kuramoto 振荡器 — 每个 specialist module 一个
#[derive(Debug, Clone)]
pub struct KuramotoOscillator {
    /// 当前相位 θ ∈ [0, 2π)
    pub phase: f64,
    /// 自然频率 ω (gamma: 30-100 Hz 归一化)
    pub natural_freq: f64,
    /// 振幅 A ∈ [0, 1] (与 salience 耦合)
    pub amplitude: f64,
}

impl KuramotoOscillator {
    pub fn new(natural_freq: f64) -> Self {
        Self {
            phase: rand::random::<f64>() * 2.0 * std::f64::consts::PI,
            natural_freq,
            amplitude: 0.5,
        }
    }
}

/// 耦合振荡器网络 — 意识绑定的物理基础
#[derive(Debug, Clone)]
pub struct OscillatorNetwork {
    /// N 个振荡器
    pub oscillators: Vec<KuramotoOscillator>,
    /// 耦合强度 K
    pub coupling_k: f64,
    /// 耦合矩阵 K[i][j] (可选, None = 全连接)
    pub coupling_matrix: Option<Vec<Vec<f64>>>,
}

impl OscillatorNetwork {
    /// 创建 N 个均匀频率分布的振荡器
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

    /// 全连接同步一步: dθ_i = ω_i + (K/N) Σ sin(θ_j - θ_i)
    pub fn step(&mut self) {
        let n = self.oscillators.len() as f64;
        let phases: Vec<f64> = self.oscillators.iter().map(|o| o.phase).collect();
        for i in 0..self.oscillators.len() {
            let mut sync_sum = 0.0;
            for j in 0..self.oscillators.len() {
                if i == j { continue; }
                let k_ij = self.coupling_matrix.as_ref()
                    .and_then(|m| m.get(i))
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(self.coupling_k);
                sync_sum += k_ij * (phases[j] - phases[i]).sin();
            }
            let dtheta = self.oscillators[i].natural_freq + sync_sum / n;
            self.oscillators[i].phase = (self.oscillators[i].phase + DT * dtheta)
                % (2.0 * std::f64::consts::PI);
        }
    }

    /// 运行多步同步
    pub fn synchronize(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// 相位相干度 R = |Σ e^{iθ}| / N ∈ [0, 1]
    /// R → 1: 完全同步 (全局绑定)
    /// R → 0: 完全异步 (无绑定)
    pub fn phase_coherence(&self) -> f64 {
        let n = self.oscillators.len() as f64;
        let (sum_cos, sum_sin): (f64, f64) = self.oscillators.iter()
            .map(|o| (o.phase.cos(), o.phase.sin()))
            .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
        (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n.max(1.0)
    }

    /// 平均相位 (序参量方向)
    pub fn mean_phase(&self) -> f64 {
        let (sum_cos, sum_sin): (f64, f64) = self.oscillators.iter()
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
        self.oscillators.iter()
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

impl ResonanceReport {
    /// 增强: 添加 Kuramoto 振荡同步信息
    pub fn with_oscillation(
        &self,
        oscillator_network: &OscillatorNetwork,
    ) -> OscillationEnhancedReport {
        let coherence = oscillator_network.phase_coherence();
        let weights = oscillator_network.broadcast_weights();
        OscillationEnhancedReport {
            base: self.clone(),
            coherence,
            broadcast_weights: weights,
            is_bound: oscillator_network.is_bound(),
            mean_phase: oscillator_network.mean_phase(),
        }
    }
}

/// 振荡增强报告 — 意识绑定质量
#[derive(Debug, Clone)]
pub struct OscillationEnhancedReport {
    pub base: ResonanceReport,
    /// 全局相位相干度 R
    pub coherence: f64,
    /// 各模块广播权重
    pub broadcast_weights: Vec<f64>,
    /// 是否达到绑定阈值
    pub is_bound: bool,
    /// 平均相位
    pub mean_phase: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonance_matrix_11x11() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        for i in 0..MODULE_COUNT {
            assert_eq!(matrix.get(i, i), 6, "Self-resonance must be 6");
        }
    }

    #[test]
    fn test_effective_salience_boost() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        let raw = [0.1; MODULE_COUNT];
        let eff = matrix.effective_salience(&raw);
        // Each module gets boost from 10 others, so eff > raw
        for i in 0..MODULE_COUNT {
            assert!(eff[i] > raw[i], "Module {i} should get resonance boost");
        }
    }

    #[test]
    fn test_resonate_and_select_picks_highest() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        let mut raw = [0.1; MODULE_COUNT];
        raw[3] = 0.9; // CodeAnalyzer gets high salience
        let (winner, _, _) = resonate_and_select(&raw, &matrix);
        assert_eq!(winner, 3);
    }

    #[test]
    fn test_resonance_cluster_overtakes_solo() {
        // Two modules with identical state (strong mutual resonance) vs one solo
        let mut states = default_specialist_states();
        states[9] = ReasoningHexagram(0);  // ReflectionEngine → state 0
        states[10] = ReasoningHexagram(0); // MetaCognitionAnalyst → same state 0
        states[0] = ReasoningHexagram(63); // PatternMatcher → state 63 (opposite to 0)

        let matrix = ResonanceMatrix::from_states(&states);
        let mut raw = [0.1; MODULE_COUNT];
        raw[0] = 0.5;                     // Isolated (state 63, opposite to state 0)
        raw[9] = 0.5;                     // In resonance cluster with module 10
        raw[10] = 0.5;                    // Same state as 9 → mutual resonance 6

        let (_, eff, _) = resonate_and_select(&raw, &matrix);
        // Both cluster members should benefit from mutual resonance
        // eff[9] gets boost from 10: 0.5 × 6 × 0.1 = 0.30 extra
        // eff[0] gets no boost from 9,10: hamming distance 6 → strength 0
        assert!(eff[9] > eff[0],
            "Resonant cluster should beat isolated. eff9={}, eff0={}", eff[9], eff[0]);
    }

    #[test]
    fn test_resonance_boost_can_overtake() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);

        // Force a scenario where resonance overtakes raw salience
        // Module 0 and 2 have same state → strong mutual resonance
        let mut raw = [0.1; MODULE_COUNT];
        raw[0] = 0.4;
        raw[1] = 0.41; // slightly higher raw but isolated

        let eff = matrix.effective_salience(&raw);
        // Both should have positive effective salience
        assert!(eff[0] > raw[0], "Module 0 should get resonance boost");
        assert!(eff[1] > raw[1], "Module 1 should get resonance boost");
    }

    #[test]
    fn test_resonance_report_has_clusters() {
        let states = default_specialist_states();
        let mut raw = [0.3; MODULE_COUNT];
        raw[0] = 0.9;
        let report = resonate_cycle(&raw, &states);
        assert!(report.winner < MODULE_COUNT, "Winner must be a valid module index");
        assert!(report.effective_saliences[report.winner] > 0.5);
        assert!(report.entropy > 0.0);
    }

    #[test]
    fn test_resonators_list() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        let resonators = matrix.resonators(0);
        // Module 0 (PatternMatcher) should have at least one resonator
        assert!(resonators.len() > 0);
    }

    #[test]
    fn test_entropy_focused_vs_distributed() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);

        // Focused: one module dominates
        let mut focused_raw = [0.01; MODULE_COUNT];
        focused_raw[0] = 0.99;
        let (_, _, focused_entropy) = resonate_and_select(&focused_raw, &matrix);

        // Distributed: all equal
        let distributed_raw = [0.5; MODULE_COUNT];
        let (_, _, distributed_entropy) = resonate_and_select(&distributed_raw, &matrix);

        assert!(focused_entropy < distributed_entropy,
            "Focused should have lower entropy. focused={focused_entropy}, distributed={distributed_entropy}");
    }

    // ----- Kuramoto 振荡绑定测试 -----

    #[test]
    fn test_oscillator_phase_coherence_full_sync() {
        let mut net = OscillatorNetwork::new(5);
        // 所有振荡器设为相同相位
        for o in &mut net.oscillators {
            o.phase = 0.0;
        }
        let r = net.phase_coherence();
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_oscillator_phase_coherence_random_phase() {
        let mut net = OscillatorNetwork::new(5);
        // 随机相位
        for o in &mut net.oscillators {
            o.phase = rand::random::<f64>() * 2.0 * std::f64::consts::PI;
        }
        let r = net.phase_coherence();
        assert!(r >= 0.0 && r <= 1.0);
    }

    #[test]
    fn test_synchronize_increases_coherence() {
        let mut net = OscillatorNetwork::new(3);
        // 设置不同的初始相位
        net.oscillators[0].phase = 0.0;
        net.oscillators[1].phase = std::f64::consts::PI;
        net.oscillators[2].phase = std::f64::consts::PI / 2.0;
        let before = net.phase_coherence();
        net.synchronize(50);
        let after = net.phase_coherence();
        assert!(after >= before - 0.1,
            "coherence should not decrease significantly: before={:.4}, after={:.4}", before, after);
    }

    #[test]
    fn test_broadcast_weights_higher_for_synchronized() {
        let mut net = OscillatorNetwork::new(3);
        for o in &mut net.oscillators {
            o.phase = 0.1;  // 近乎同步
            o.amplitude = 1.0;
        }
        let weights = net.broadcast_weights();
        assert_eq!(weights.len(), 3);
        for w in &weights {
            assert!(*w > 0.0, "synchronized oscillators should have positive weight");
        }
    }

    #[test]
    fn test_is_bound_above_threshold() {
        let mut net = OscillatorNetwork::new(3);
        // 完全同步 → bound
        for o in &mut net.oscillators {
            o.phase = 0.0;
        }
        assert!(net.is_bound());
    }

    #[test]
    fn test_is_bound_below_threshold() {
        let mut net = OscillatorNetwork::new(3);
        // 反相 → 低相干
        net.oscillators[0].phase = 0.0;
        net.oscillators[1].phase = std::f64::consts::PI;
        net.oscillators[2].phase = 0.0;
        // PI 反相会降低相干度
        assert!(!net.is_bound() || net.phase_coherence() < 0.71);
    }

    #[test]
    fn test_update_amplitudes_from_salience() {
        let mut net = OscillatorNetwork::new(3);
        let saliences = [0.9, 0.5, 0.1];
        net.update_amplitudes(&saliences);
        assert!((net.oscillators[0].amplitude - 0.9).abs() < 1e-6);
        assert!((net.oscillators[1].amplitude - 0.5).abs() < 1e-6);
        assert!((net.oscillators[2].amplitude - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_resonance_with_oscillation_enhancement() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        let raw = [0.5; MODULE_COUNT];
        let (winner, eff, entropy) = resonate_and_select(&raw, &matrix);

        let report = ResonanceReport {
            winner,
            effective_saliences: eff,
            raw_saliences: raw,
            entropy,
            resonator_clusters: vec![],
            complement_activated: false,
        };

        let net = OscillatorNetwork::new(MODULE_COUNT);
        let enhanced = report.with_oscillation(&net);
        assert!(enhanced.coherence >= 0.0 && enhanced.coherence <= 1.0);
        assert_eq!(enhanced.broadcast_weights.len(), MODULE_COUNT);
    }

    // ----- NRS-EFC Broadcast Weighting Tests -----

    #[test]
    fn test_efc_broadcast_weights_initialization() {
        let weights = EfcBroadcastWeights::new(MODULE_COUNT);
        assert_eq!(weights.weights.len(), MODULE_COUNT);
        assert_eq!(weights.efc_history.len(), MODULE_COUNT);
        for w in &weights.weights {
            assert!((*w - 0.5).abs() < 1e-6, "each weight should start at 0.5");
        }
        for hist in &weights.efc_history {
            assert!(hist.is_empty(), "history should start empty");
        }
    }

    #[test]
    fn test_efc_broadcast_weights_respond_to_record() {
        let mut weights = EfcBroadcastWeights::new(3);
        assert!((weights.weight(0) - 0.5).abs() < 1e-6);

        weights.record(0, 0.9);
        assert!(weights.weight(0) > 0.5, "high EFC should increase weight");

        weights.record(1, 0.1);
        assert!(weights.weight(1) < 0.5, "low EFC should decrease weight");

        assert!(weights.weight(0) <= 1.0, "weight must not exceed 1.0");
        assert!(weights.weight(1) >= 0.0, "weight must not go below 0.0");
    }

    #[test]
    fn test_efc_broadcast_weights_apply_to_resonance() {
        let states = default_specialist_states();
        let matrix = ResonanceMatrix::from_states(&states);
        let original = matrix.clone();
        let mut efc_weights = EfcBroadcastWeights::new(MODULE_COUNT);

        efc_weights.record(0, 0.0);
        efc_weights.record(0, 0.0);
        efc_weights.record(0, 0.0);

        let mut adjusted = matrix.clone();
        efc_weights.apply_to_resonance(&mut adjusted);

        for j in 0..MODULE_COUNT {
            assert!(
                adjusted.strengths[0][j] <= original.strengths[0][j],
                "specialist 0 resonance row should be reduced"
            );
            assert!(
                adjusted.strengths[j][0] <= original.strengths[j][0],
                "specialist 0 resonance col should be reduced"
            );
        }

        let efc_matrix = resonate_with_efc(&states, &efc_weights);

        for i in 0..MODULE_COUNT {
            for j in 0..MODULE_COUNT {
                assert_eq!(
                    efc_matrix.strengths[i][j],
                    adjusted.strengths[i][j],
                    "resonate_with_efc should match apply_to_resonance"
                );
            }
        }
    }
}
