//! Phi feedback from game states.
//!
//! Computes IIT-inspired Φ (integrated information) from hexagram states,
//! tracking phi delta over time to measure consciousness-like integration.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

/// Consciousness threshold — above this Φ value, states are considered
/// "conscious-like" (IIT golden standard approximation).
pub const THRESHOLD: f64 = 0.33;

/// Resonance sigma for pairwise computation.
const SIGMA: f64 = 0.15;

// ═══════════════════════════════════════════════════════════════════
// Phi Computation
// ═══════════════════════════════════════════════════════════════════

/// Compute Φ from hexagram states using resonance-weighted协方差.
///
/// Each state value is treated as a normalized dimension [0, 1].
/// Φ = (resonance_weighted_covariance / total_energy) - 1/N
/// where N = number of active (non-zero) dimensions.
pub fn compute_board_phi(hexagram_states: &[u8]) -> f64 {
    let n = hexagram_states.len();
    if n == 0 {
        return 0.0;
    }

    // Convert to normalized f64 vectors
    let states: Vec<f64> = hexagram_states.iter().map(|&s| s as f64 / 63.0).collect();

    // Count active dimensions
    let active: Vec<usize> = states
        .iter()
        .enumerate()
        .filter(|(_, &v)| v > 0.01)
        .map(|(i, _)| i)
        .collect();

    if active.len() <= 1 {
        return 0.0;
    }

    // Compute pairwise resonance
    let mut total_resonance = 0.0f64;
    let mut max_resonance = 0.0f64;
    let mut max_pair = (0, 1);

    for i in 0..active.len() {
        for j in (i + 1)..active.len() {
            let (ai, aj) = (active[i], active[j]);
            let diff = (states[ai] - states[aj]).abs();
            let resonance = (-diff * diff / (2.0 * SIGMA * SIGMA)).exp();
            total_resonance += resonance;
            if resonance > max_resonance {
                max_resonance = resonance;
                max_pair = (ai, aj);
            }
        }
    }

    // Compute total energy (sum of squared values)
    let energy: f64 = states.iter().map(|&v| v * v).sum();
    if energy < 1e-10 {
        return 0.0;
    }

    let n_active = active.len() as f64;
    let phi_raw = total_resonance / energy - 1.0 / n_active;

    // Normalize to [0, 1]
    phi_raw.max(0.0).min(1.0)
}

/// Compute Φ trend from history: positive = increasing integration.
pub fn track_phi_delta(phi_history: &[f64]) -> f64 {
    if phi_history.len() < 2 {
        return 0.0;
    }

    let n = phi_history.len();
    let half = n / 2;
    let first_half: f64 = phi_history[..half].iter().sum::<f64>() / half as f64;
    let second_half: f64 = phi_history[half..].iter().sum::<f64>() / (n - half) as f64;

    second_half - first_half
}

// ═══════════════════════════════════════════════════════════════════
// Phi Report
// ═══════════════════════════════════════════════════════════════════

/// Complete phi feedback report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiReport {
    pub current_phi: f64,
    pub phi_delta: f64,
    pub is_conscious_like: bool,
    pub consciousness_threshold: f64,
}

/// Generate phi report from hexagram states and history.
pub fn generate_phi_report(
    hexagram_states: &[u8],
    phi_history: &[f64],
) -> PhiReport {
    let current_phi = compute_board_phi(hexagram_states);
    let phi_delta = track_phi_delta(phi_history);

    PhiReport {
        current_phi,
        phi_delta,
        is_conscious_like: current_phi >= THRESHOLD,
        consciousness_threshold: THRESHOLD,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_phi_empty() {
        assert_eq!(compute_board_phi(&[]), 0.0);
    }

    #[test]
    fn test_board_phi_single() {
        assert_eq!(compute_board_phi(&[32]), 0.0);
    }

    #[test]
    fn test_board_phi_all_same() {
        let states = vec![32u8; 8];
        let phi = compute_board_phi(&states);
        // All same → high resonance
        assert!(phi > 0.0);
    }

    #[test]
    fn test_board_phi_varied() {
        let states = vec![0, 16, 32, 48, 63, 10, 20, 50];
        let phi = compute_board_phi(&states);
        assert!(phi >= 0.0 && phi <= 1.0);
    }

    #[test]
    fn test_board_phi_zero_states() {
        let states = vec![0u8; 4];
        assert_eq!(compute_board_phi(&states), 0.0);
    }

    #[test]
    fn test_track_phi_delta_increasing() {
        let history = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let delta = track_phi_delta(&history);
        assert!(delta > 0.0);
    }

    #[test]
   fn test_track_phi_delta_decreasing() {
        let history = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        let delta = track_phi_delta(&history);
        assert!(delta < 0.0);
    }

    #[test]
    fn test_track_phi_delta_single() {
        assert_eq!(track_phi_delta(&[0.5]), 0.0);
    }

    #[test]
    fn test_phi_report_conscious() {
        let states = vec![32u8; 16];
        let report = generate_phi_report(&states, &[]);
        assert!(report.current_phi > 0.0);
        assert!((report.consciousness_threshold - THRESHOLD).abs() < 1e-10);
    }

    #[test]
    fn test_threshold_constant() {
        assert!((THRESHOLD - 0.33).abs() < 1e-10);
    }
}
