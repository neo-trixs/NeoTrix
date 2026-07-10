#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::resonance::{ResonanceMatrix, MODULE_COUNT, RESONANCE_THRESHOLD};
use crate::core::nt_core_consensus::{
    ReflectionPipeline, ConsensusReport, ConsensusConfig, ReflectionHead,
};
use crate::core::nt_core_hex::ReasoningHexagram;

pub struct ConsensusResonanceBridge {
    pub resonance: ResonanceMatrix,
    pub consensus: ReflectionPipeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResonanceReport {
    pub consensus_converged: bool,
    pub consensus_iterations: u32,
    pub consensus_confidence: f64,
    pub resonance_strength_avg: f64,
    pub modules_in_resonance: usize,
}

impl ConsensusResonanceBridge {
    pub fn new(resonance: ResonanceMatrix) -> Self {
        let config = ConsensusConfig::default();
        let consensus = ReflectionPipeline::new(config);
        Self { resonance, consensus }
    }

    pub fn resonate_with_consensus(
        &mut self,
        states: &[ReasoningHexagram; MODULE_COUNT],
        observations: &[String],
    ) -> ConsensusResonanceReport {
        let consensus_report = self.consensus.run_consensus_cycle(observations);
        let confidence = consensus_report.confidence;
        let mut strengths = [[0u32; MODULE_COUNT]; MODULE_COUNT];
        let mut total_strength: u64 = 0;
        let mut in_resonance = 0;
        for i in 0..MODULE_COUNT {
            for j in 0..MODULE_COUNT {
                let base = states[i].resonance_strength(&states[j]);
                let modulation = if consensus_report.converged {
                    (confidence * 2.0).round() as u32
                } else {
                    0
                };
                let modulated = (base as u32).saturating_add(modulation).min(6);
                strengths[i][j] = modulated;
                total_strength += modulated as u64;
                if modulated >= RESONANCE_THRESHOLD as u32 && i != j {
                    in_resonance += 1;
                }
            }
        }
        self.resonance.strengths = strengths;
        let avg_strength = total_strength as f64 / (MODULE_COUNT * MODULE_COUNT) as f64;
        ConsensusResonanceReport {
            consensus_converged: consensus_report.converged,
            consensus_iterations: consensus_report.iterations,
            consensus_confidence: confidence,
            resonance_strength_avg: avg_strength,
            modules_in_resonance: in_resonance,
        }
    }

    pub fn add_consensus_head(&mut self, head: ReflectionHead) {
        self.consensus.add_head(head);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_states() -> [ReasoningHexagram; MODULE_COUNT] {
        let mut states = [ReasoningHexagram(0); MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            states[i] = ReasoningHexagram((i * 5 % 64) as u8);
        }
        states
    }

    #[test]
    fn test_new_bridge() {
        let base = ResonanceMatrix::from_states(&test_states());
        let bridge = ConsensusResonanceBridge::new(base);
        assert_eq!(bridge.resonance.strengths.len(), MODULE_COUNT);
    }

    #[test]
    fn test_resonate_with_consensus_produces_report() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        let observations = vec!["test observation".to_string()];
        let report = bridge.resonate_with_consensus(&test_states(), &observations);
        assert!(report.resonance_strength_avg >= 0.0);
        assert!(report.resonance_strength_avg <= 6.0);
        assert!(report.modules_in_resonance >= 0);
    }

    #[test]
    fn test_multiple_observations() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        let observations = vec!["obs1".to_string(), "obs2".to_string(), "obs3".to_string()];
        let report = bridge.resonate_with_consensus(&test_states(), &observations);
        assert!(report.consensus_iterations > 0);
    }

    #[test]
    fn test_add_head() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        let head = ReflectionHead::new(99, "test-perspective".into());
        bridge.add_consensus_head(head);
        assert_eq!(bridge.consensus.heads.len(), 1);
    }

    #[test]
    fn test_resonance_strength_ranges() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        let observations = vec!["test".to_string()];
        let report = bridge.resonate_with_consensus(&test_states(), &observations);
        assert!(report.resonance_strength_avg >= 0.0);
        assert!(report.resonance_strength_avg <= 6.0);
    }

    #[test]
    fn test_consecutive_resonances() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        for i in 0..3 {
            let obs = vec![format!("obs_{}", i)];
            let report = bridge.resonate_with_consensus(&test_states(), &obs);
            assert!(report.resonance_strength_avg >= 0.0);
            assert!(report.consensus_iterations > 0 || report.consensus_converged);
        }
    }

    #[test]
    fn test_empty_observations() {
        let base = ResonanceMatrix::from_states(&test_states());
        let mut bridge = ConsensusResonanceBridge::new(base);
        let report = bridge.resonate_with_consensus(&test_states(), &[]);
        assert!(report.resonance_strength_avg >= 0.0);
    }
}
