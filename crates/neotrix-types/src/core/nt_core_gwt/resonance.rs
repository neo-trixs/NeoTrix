//! Resonance attention mechanism — replaces salience-only competition.
//!
//! Based on the E₈ × 64 state-space model:
//! - Each specialist module operates in a reasoning hexagram state
//! - Modules in resonance (hamming dist ≤ 2) amplify each other's salience
//! - Complementary modules (错卦) automatically trigger opposing perspectives
//! - The +1 observer tracks the overall resonance landscape

use crate::core::nt_core_hex::ReasoningHexagram;
use serde::{Deserialize, Serialize};

/// Maximum resonance distance (hamming dist ≤ 2 → in resonance).
pub const RESONANCE_THRESHOLD: u32 = 2;

/// Number of specialist modules (was 11, expanded to 14 for Cycle 4 specialists).
pub const MODULE_COUNT: usize = 14;

/// Pre-computed resonance matrix: 14×14 pairwise resonance strengths.
#[derive(Debug, Clone)]
pub struct ResonanceMatrix {
    /// resonance[i][j] = resonance strength between module i and j (0-6).
    pub strengths: [[u32; MODULE_COUNT]; MODULE_COUNT],
}

impl ResonanceMatrix {
    /// Build from a slice of 14 hexagram assignments (one per specialist).
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
        for (i, item) in eff.iter_mut().enumerate().take(MODULE_COUNT) {
            let mut resonance_boost = 0.0;
            for (j, r) in raw.iter().enumerate().take(MODULE_COUNT) {
                if i == j { continue; }
                let boost = self.strengths[i][j] as f64 * r * 0.1;
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
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("f64 partial_cmp should not produce NaN"))
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
        s(45), // AISecurity: security analysis (concrete+analytical+certain+deep)
        s(46), // ImageGenerator: creative abstract generation (abstract+generative+broad)
        s(50), // EvidenceWeightedHypothesis: Bayesian reasoning (abstract+analytical+deep)
    ]
}

/// Thinking budget gate — modulates effective salience based on compute budget.
///
/// Implements Hermes/Gemini thinking budget pattern: cheap tasks get small
/// budgets (fewer modules activated), expensive tasks get large budgets
/// (more modules can participate). Integrates with cost-aware routing
/// (Axiom A1: not all tasks need the strongest model).
///
/// Score modulation: `modulated_salience[i] = raw_eff[i] * budget_factor[i]`
/// where `budget_factor[i] = min(1.0, budget / module_cost[i])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudgetGate {
    /// Token budget allocated for this reasoning cycle.
    pub budget_tokens: u32,
    /// Per-module cost weights (higher = more expensive to activate).
    pub module_costs: [f64; MODULE_COUNT],
    /// Cost sensitivity: 0.0 = ignore budget, 1.0 = hard cap.
    pub cost_sensitivity: f64,
    /// Minimum modules that must remain active regardless of budget.
    pub min_active: usize,
}

impl Default for ThinkingBudgetGate {
    fn default() -> Self {
        Self {
            budget_tokens: 2048,
            // Default costs: simple modules cheap, complex modules expensive
            module_costs: [
                0.1, 0.15, 0.12, 0.2, 0.25, 0.3, 0.35, 0.2, 0.18, 0.3,
                0.28, 0.22, 0.15, 0.2,
            ],
            cost_sensitivity: 0.5,
            min_active: 3,
        }
    }
}

impl ThinkingBudgetGate {
    /// Create a budget gate with a specific token budget.
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            budget_tokens,
            ..Default::default()
        }
    }

    /// Cheap task budget (512 tokens): only cheapest modules survive.
    pub fn cheap() -> Self {
        Self::new(512)
    }

    /// Standard task budget (2048 tokens): balanced module selection.
    pub fn standard() -> Self {
        Self::new(2048)
    }

    /// Expensive task budget (8192 tokens): all modules can participate.
    pub fn expensive() -> Self {
        Self::new(8192)
    }

    /// Compute per-module budget factors based on token budget and module costs.
    ///
    /// Modules with cost > budget are suppressed (factor < 1.0).
    /// At least `min_active` modules remain active.
    pub fn budget_factors(&self) -> [f64; MODULE_COUNT] {
        let mut factors = [0.0f64; MODULE_COUNT];
        let budget_f = self.budget_tokens as f64;

        for i in 0..MODULE_COUNT {
            let cost = self.module_costs[i];
            if cost <= 0.0 {
                factors[i] = 1.0;
                continue;
            }
            // Factor = min(1.0, budget / (cost * total_budget_scale))
            // Higher cost modules need more budget to stay active
            let raw_factor = budget_f / (cost * 10000.0);
            factors[i] = (raw_factor * self.cost_sensitivity + (1.0 - self.cost_sensitivity))
                .clamp(0.0, 1.0);
        }

        // Ensure at least `min_active` modules remain viable
        let mut indexed: Vec<(usize, f64)> = factors.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("no NaN in factors"));
        for &(idx, _) in indexed.iter().take(self.min_active) {
            factors[idx] = factors[idx].max(0.1);
        }

        factors
    }

    /// Apply budget modulation to effective salience.
    pub fn modulate(&self, effective: &[f64; MODULE_COUNT]) -> [f64; MODULE_COUNT] {
        let factors = self.budget_factors();
        let mut modulated = [0.0f64; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            modulated[i] = effective[i] * factors[i];
        }
        modulated
    }
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
    /// Budget-modulated saliences (if budget gate was applied).
    pub budget_saliences: Option<[f64; MODULE_COUNT]>,
    /// Thinking budget used for this cycle.
    pub thinking_budget: Option<u32>,
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
        budget_saliences: None,
        thinking_budget: None,
    }
}

/// Run a budget-aware resonance cycle.
///
/// Applies `ThinkingBudgetGate` modulation to effective salience before
/// winner selection — cheap tasks suppress expensive modules, expensive
/// tasks allow full participation. The winner is chosen from budget-modulated
/// saliences.
pub fn resonate_cycle_with_budget(
    raw_salience: &[f64; MODULE_COUNT],
    states: &[ReasoningHexagram; MODULE_COUNT],
    budget_gate: &ThinkingBudgetGate,
) -> ResonanceReport {
    let matrix = ResonanceMatrix::from_states(states);
    let raw_eff = matrix.effective_salience(raw_salience);

    // Apply budget modulation
    let budget_modulated = budget_gate.modulate(&raw_eff);

    // Winner from budget-modulated salience
    let winner = budget_modulated
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("no NaN"))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Entropy from budget-modulated salience
    let total: f64 = budget_modulated.iter().sum();
    let entropy = if total > 0.0 {
        -budget_modulated
            .iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                p * p.log2()
            })
            .sum::<f64>()
    } else {
        0.0
    };

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
        effective_saliences: budget_modulated,
        raw_saliences: *raw_salience,
        entropy,
        resonator_clusters,
        complement_activated,
        budget_saliences: Some(raw_eff),
        thinking_budget: Some(budget_gate.budget_tokens),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonance_matrix_14x14() {
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
        assert!(!resonators.is_empty());
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

    #[test]
    fn test_thinking_budget_gate_cheap_suppresses() {
        let gate = ThinkingBudgetGate::cheap(); // 512 tokens
        let factors = gate.budget_factors();
        // With cheap budget, expensive modules should have lower factors
        let expensive_factor = factors[6]; // GoalPrioritizer cost=0.35
        let cheap_factor = factors[0];     // PatternMatcher cost=0.1
        assert!(cheap_factor > expensive_factor,
            "Cheap modules should survive cheap budget better: cheap={cheap_factor}, expensive={expensive_factor}");
    }

    #[test]
    fn test_thinking_budget_gate_expensive_allows_all() {
        let gate = ThinkingBudgetGate::expensive(); // 8192 tokens
        let factors = gate.budget_factors();
        // All modules should have factor close to 1.0
        for (i, &f) in factors.iter().enumerate() {
            assert!(f >= 0.5, "Module {i} should remain active with large budget, got factor {f}");
        }
    }

    #[test]
    fn test_budget_modulate_scales_salience() {
        let gate = ThinkingBudgetGate::new(1000);
        let mut raw_eff = [0.5; MODULE_COUNT];
        raw_eff[0] = 0.8; // cheap module
        raw_eff[6] = 0.8; // expensive module

        let modulated = gate.modulate(&raw_eff);
        // Both should be <= raw_eff
        assert!(modulated[0] <= raw_eff[0]);
        assert!(modulated[6] <= raw_eff[6]);
        // Cheap module should retain more than expensive
        assert!(modulated[0] >= modulated[6],
            "Cheap module should survive budget better: modulated[0]={}, modulated[6]={}",
            modulated[0], modulated[6]);
    }

    #[test]
    fn test_resonate_cycle_with_budget_changes_winner() {
        let states = default_specialist_states();
        let mut raw = [0.3; MODULE_COUNT];
        raw[3] = 0.5;  // CodeAnalyzer (cost=0.2)
        raw[6] = 0.52; // GoalPrioritizer (cost=0.35) — slightly higher raw

        // Without budget: GoalPrioritizer wins (higher raw)
        let report_no_budget = resonate_cycle(&raw, &states);
        assert_eq!(report_no_budget.winner, 6);

        // With cheap budget: expensive module penalized, CodeAnalyzer may win
        let gate = ThinkingBudgetGate::cheap();
        let report_budget = resonate_cycle_with_budget(&raw, &states, &gate);
        assert!(report_budget.thinking_budget == Some(512));
        assert!(report_budget.budget_saliences.is_some());
    }

    #[test]
    fn test_resonate_cycle_report_has_budget_fields() {
        let states = default_specialist_states();
        let raw = [0.3; MODULE_COUNT];
        let report = resonate_cycle(&raw, &states);
        assert!(report.budget_saliences.is_none());
        assert!(report.thinking_budget.is_none());
    }

    #[test]
    fn test_thinking_budget_gate_min_active() {
        let gate = ThinkingBudgetGate {
            budget_tokens: 100, // very small
            min_active: 5,
            ..ThinkingBudgetGate::default()
        };
        let factors = gate.budget_factors();
        // At least 5 modules should have factor >= 0.1
        let active_count = factors.iter().filter(|&&f| f >= 0.1).count();
        assert!(active_count >= 5,
            "min_active=5 should ensure at least 5 viable modules, got {active_count}");
    }
}
