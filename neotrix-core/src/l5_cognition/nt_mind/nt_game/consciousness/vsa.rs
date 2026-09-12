//! VSA trajectory encoding for memory.
//!
//! Encodes game trajectories as VSA-like vectors for similarity search
//! and cross-session memory recall.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Types (self-contained)
// ═══════════════════════════════════════════════════════════════════

/// Simplified trajectory for encoding.
#[derive(Debug, Clone)]
pub struct Trajectory {
    pub total_reward: f64,
    pub length: usize,
    pub hexagrams: Vec<u8>,
    pub resonance_count: usize,
}

impl Trajectory {
    pub fn new(total_reward: f64, length: usize, hexagrams: Vec<u8>, resonance_count: usize) -> Self {
        Self {
            total_reward,
            length,
            hexagrams,
            resonance_count,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// VSA Encoder
// ═══════════════════════════════════════════════════════════════════

/// VSA trajectory encoder — converts trajectories to normalized vectors.
pub struct GameVsaEncoder;

impl GameVsaEncoder {
    /// Encode a trajectory as a VSA-like feature vector.
    ///
    /// Features:
    /// - total_reward (normalized by expected range [-1, 1])
    /// - length (normalized by 100)
    /// - avg_hexgram (mean hexagram value / 63)
    /// - resonance_count (normalized by trajectory length)
    /// - reward_variance (std deviation of step rewards, if available)
    /// - hexgram_diversity (number of distinct hexagrams / 64)
    ///
    /// Returns a unit-normalized vector.
    pub fn encode_trajectory(trajectory: &Trajectory) -> Vec<f64> {
        let len = trajectory.length.max(1) as f64;

        // Feature 1: normalized total reward
        let reward_norm = (trajectory.total_reward / 10.0).clamp(-1.0, 1.0);

        // Feature 2: normalized length
        let length_norm = (trajectory.length as f64 / 100.0).min(1.0);

        // Feature 3: average hexagram
        let avg_hex = if trajectory.hexagrams.is_empty() {
            0.0
        } else {
            let sum: u64 = trajectory.hexagrams.iter().map(|&h| h as u64).sum();
            sum as f64 / trajectory.hexagrams.len() as f64 / 63.0
        };

        // Feature 4: resonance density
        let resonance_density = (trajectory.resonance_count as f64 / len).min(1.0);

        // Feature 5: hexgram diversity
        let distinct: std::collections::HashSet<u8> =
            trajectory.hexagrams.iter().copied().collect();
        let diversity = distinct.len() as f64 / 64.0;

        let features = vec![reward_norm, length_norm, avg_hex, resonance_density, diversity];

        // Unit-normalize
        let norm: f64 = features.iter().map(|&f| f * f).sum::<f64>().sqrt();
        if norm < 1e-10 {
            return vec![0.0; features.len()];
        }
        features.iter().map(|&f| f / norm).collect()
    }

    /// Compute cosine similarity between two encoded vectors.
    pub fn similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|&x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if norm_a < 1e-10 || norm_b < 1e-10 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }
}

// ═══════════════════════════════════════════════════════════════════
// VSA Report
// ═══════════════════════════════════════════════════════════════════

/// Strategy signature — human-readable description of the encoded strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignature {
    pub aggressive: bool,
    pub efficient: bool,
    pub diverse: bool,
    pub resonant: bool,
}

impl StrategySignature {
    pub fn from_features(reward_norm: f64, length_norm: f64, diversity: f64, resonance: f64) -> Self {
        Self {
            aggressive: reward_norm > 0.3,
            efficient: length_norm < 0.3,
            diverse: diversity > 0.5,
            resonant: resonance > 0.3,
        }
    }
}

/// Complete VSA report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaReport {
    pub encoded_vector: Vec<f64>,
    pub strategy_signature: StrategySignature,
    /// Transfer potential: how likely this strategy transfers to new games.
    /// Higher resonance + diversity = higher transfer.
    pub transfer_potential: f64,
}

/// Generate VSA report from trajectory.
pub fn generate_vsa_report(trajectory: &Trajectory) -> VsaReport {
    let encoded = GameVsaEncoder::encode_trajectory(trajectory);
    let signature = if encoded.len() >= 5 {
        StrategySignature::from_features(encoded[0], encoded[1], encoded[4], encoded[3])
    } else {
        StrategySignature {
            aggressive: false,
            efficient: false,
            diverse: false,
            resonant: false,
        }
    };

    let transfer_potential = if encoded.len() >= 5 {
        (encoded[3] + encoded[4]) / 2.0
    } else {
        0.0
    };

    VsaReport {
        encoded_vector: encoded,
        strategy_signature: signature,
        transfer_potential,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_trajectory_unit_vector() {
        let traj = Trajectory::new(1.0, 10, vec![32, 16, 48], 5);
        let encoded = GameVsaEncoder::encode_trajectory(&traj);
        assert_eq!(encoded.len(), 5);
        let norm: f64 = encoded.iter().map(|&f| f * f).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_encode_empty_hexagrams() {
        let traj = Trajectory::new(0.0, 0, vec![], 0);
        let encoded = GameVsaEncoder::encode_trajectory(&traj);
        assert_eq!(encoded.len(), 5);
        // All zero features → zero vector
        let norm: f64 = encoded.iter().map(|&f| f * f).sum::<f64>().sqrt();
        assert!(norm < 1e-10);
    }

    #[test]
    fn test_similarity_identical() {
        let a = vec![0.5, 0.5, 0.0, 0.0, 0.707];
        let b = vec![0.5, 0.5, 0.0, 0.0, 0.707];
        let sim = GameVsaEncoder::similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = GameVsaEncoder::similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_different_lengths() {
        let a = vec![1.0];
        let b = vec![1.0, 0.0];
        assert_eq!(GameVsaEncoder::similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_vsa_report() {
        let traj = Trajectory::new(5.0, 20, vec![10, 20, 30, 40, 50], 8);
        let report = generate_vsa_report(&traj);
        assert_eq!(report.encoded_vector.len(), 5);
        assert!(report.strategy_signature.aggressive);
        assert!(report.strategy_signature.diverse);
        assert!(report.transfer_potential > 0.0);
    }

    #[test]
    fn test_strategy_signature_low_resonance() {
        let sig = StrategySignature::from_features(0.0, 0.5, 0.1, 0.1);
        assert!(!sig.aggressive);
        assert!(!sig.diverse);
        assert!(!sig.resonant);
    }
}
