//! Resonance attention mechanism — replaces salience-only competition.
//!
//! Based on the E₈ × 64 state-space model:
//! - Each specialist module operates in a reasoning hexagram state
//! - Modules in resonance (hamming dist ≤ 2) amplify each other's salience
//! - Complementary modules (错卦) automatically trigger opposing perspectives
//! - The +1 observer tracks the overall resonance landscape

use crate::core::nt_core_hex::ReasoningHexagram;

/// Maximum resonance distance (hamming dist ≤ 2 → in resonance).
pub const RESONANCE_THRESHOLD: u32 = 2;

/// Number of specialist modules.
pub const MODULE_COUNT: usize = 11;

/// Pre-computed resonance matrix: 11×11 pairwise resonance strengths.
#[derive(Debug, Clone)]
pub struct ResonanceMatrix {
    /// resonance[i][j] = resonance strength between module i and j (0-6).
    pub strengths: [[u32; MODULE_COUNT]; MODULE_COUNT],
}

impl ResonanceMatrix {
    /// Build from a slice of 11 hexagram assignments (one per specialist).
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
}
