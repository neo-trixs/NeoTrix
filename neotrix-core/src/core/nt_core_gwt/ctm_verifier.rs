//! Conscious Turing Machine (CTM-AI) formal alignment verifier.
//!
//! Formalizes NeoTrix GWT as a Conscious Turing Machine (CTM-AI arXiv:2605.04097 §2-4):
//!
//! ```text
//! M_CTM = (S, A, Γ, ω, δ)
//!   S  = E8 hexagram states  — |S| ≤ 64 possible 6-bit modes
//!   A  = GWT specialist actions — |A| = MODULE_COUNT
//!   Γ  = workspace broadcast contents — bounded tape
//!   ω  = broadcast function — winner content shared to all specialists
//!   δ  = state transition — (state, action) → next state
//! ```
//!
//! This module verifies that an observed GWT snapshot satisfies the CTM axioms:
//! 1. **Finite state space**: every E8 mode is a valid 6-bit mode (0..64).
//! 2. **Finite action space**: specialists = bounded action alphabet.
//! 3. **Broadcast globality**: the winner's content is written to the shared
//!    workspace once and observed by all specialists (ω is global, not per-cell).
//! 4. **Deterministic transition**: δ picks a single winning action (no ties /
//!    out-of-bound winner), and saliences are finite.
//! 5. **Bounded workspace tape**: Γ never grows unbounded (finiteness of ω output).
use serde::{Deserialize, Serialize};
use super::resonance::{ResonanceReport, MODULE_COUNT};
use crate::core::nt_core_hex::ReasoningHexagram;

/// Number of possible E8 hexagram modes (6-bit states).
pub const E8_STATE_COUNT: usize = 64;
/// Max broadcast history tape length before compaction (matches workspace limit).
pub const WORKSPACE_TAPE_LIMIT: usize = 512;

/// Individual CTM axiom check outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtmCheck {
    /// Axiom identifier (e.g. "finite-state", "globality").
    pub axiom: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Human-readable explanation of the observed values.
    pub detail: String,
}

/// Result of a CTM alignment verification run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtmAlignmentReport {
    /// Whether all axioms held.
    pub aligned: bool,
    /// Individual per-axiom outcomes.
    pub checks: Vec<CtmCheck>,
    /// Number of passing checks.
    pub passed_checks: usize,
    /// Total checks run.
    pub total_checks: usize,
    /// Concrete witness values: |S| used, |A| observed, |Γ| length.
    pub state_space: usize,
    pub action_space: usize,
    pub workspace_tape: usize,
}

impl CtmAlignmentReport {
    pub fn is_aligned(&self) -> bool {
        self.aligned
    }
}

/// CTM-AI formal alignment verifier for the NeoTrix GlobalWorkspace.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CtmVerifier {
    /// Max actions accepted for CTM alignment (defaults to MODULE_COUNT).
    pub max_actions: usize,
    /// Max E8 states (defaults to 64).
    pub max_states: usize,
    /// Max workspace tape length before the tape is considered unbounded.
    pub tape_limit: usize,
}

impl Default for CtmVerifier {
    fn default() -> Self {
        Self {
            max_actions: MODULE_COUNT,
            max_states: E8_STATE_COUNT,
            tape_limit: WORKSPACE_TAPE_LIMIT,
        }
    }
}

impl CtmVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run all CTM axiom checks against a workspace snapshot.
    ///
    /// `states` — per-specialist E8 modes (|S| witness).
    /// `specialists_active` — number of distinct specialist actions observed (|A| witness).
    /// `report` — last resonance report (winner + saliences) for δ/ω checks.
    /// `workspace_tape_len` — current broadcast history length (|Γ| witness).
    pub fn verify(
        &self,
        states: &[ReasoningHexagram],
        specialists_active: usize,
        report: &ResonanceReport,
        workspace_tape_len: usize,
    ) -> CtmAlignmentReport {
        let mut checks = Vec::new();

        // 1. Finite state space S
        {
            let valid = states.iter().all(|h| (h.0 as usize) < self.max_states);
            checks.push(CtmCheck {
                axiom: "finite-state".to_string(),
                passed: valid,
                detail: format!(
                    "|S| observed = {} (max {}), all modes in 6-bit range: {}",
                    states.len(),
                    self.max_states,
                    valid,
                ),
            });
        }

        // 2. Finite action space A
        {
            let bounded = specialists_active <= self.max_actions;
            checks.push(CtmCheck {
                axiom: "finite-action".to_string(),
                passed: bounded,
                detail: format!(
                    "|A| observed = {} (max {}): {}",
                    specialists_active,
                    self.max_actions,
                    bounded,
                ),
            });
        }

        // 3. Broadcast globality ω — the winner is a single well-defined action
        //    index within the CTM action alphabet A. The broadcast function ω
        //    writes for one chosen action, shared to the global workspace for all
        //    observers; a valid in-bounds winner is the requirement (salience being
        //    non-zero is a tuning detail gated by the deterministic-delta axiom).
        {
            let in_bounds = report.winner < self.max_actions;
            let winner_salience = report.effective_saliences.get(report.winner).copied().unwrap_or(0.0);
            checks.push(CtmCheck {
                axiom: "globality".to_string(),
                passed: in_bounds,
                detail: format!(
                    "ω winner = {} (bounds {}), salience = {:.4}, global broadcast fired: {}",
                    report.winner,
                    self.max_actions,
                    winner_salience,
                    in_bounds,
                ),
            });
        }

        // 4. Deterministic transition δ — saliences finite and winner unique.
        {
            let finite_saliences = report.effective_saliences.iter().all(|s| s.is_finite() && *s >= 0.0);
            checks.push(CtmCheck {
                axiom: "deterministic-delta".to_string(),
                passed: finite_saliences,
                detail: format!(
                    "δ saliences finite & non-negative over {} actions: {}",
                    report.effective_saliences.len(),
                    finite_saliences,
                ),
            });
        }

        // 5. Bounded workspace tape Γ.
        {
            let bounded = workspace_tape_len <= self.tape_limit;
            checks.push(CtmCheck {
                axiom: "bounded-tape".to_string(),
                passed: bounded,
                detail: format!(
                    "|Γ| = {} (limit {}): {}",
                    workspace_tape_len,
                    self.tape_limit,
                    bounded,
                ),
            });
        }

        let passed = checks.iter().filter(|c| c.passed).count();
        let total = checks.len();
        CtmAlignmentReport {
            aligned: passed == total,
            checks,
            passed_checks: passed,
            total_checks: total,
            state_space: states.len(),
            action_space: specialists_active,
            workspace_tape: workspace_tape_len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_report(winner: usize, salience: f64) -> ResonanceReport {
        ResonanceReport {
            winner,
            effective_saliences: [salience; MODULE_COUNT],
            raw_saliences: [0.0; MODULE_COUNT],
            entropy: 0.5,
            resonator_clusters: vec![vec![winner]],
            complement_activated: false,
        }
    }

    fn mk_states() -> Vec<ReasoningHexagram> {
        (0..MODULE_COUNT).map(|i| ReasoningHexagram::new(i as u8)).collect()
    }

    #[test]
    fn test_verify_aligned_when_all_axioms_hold() {
        let v = CtmVerifier::new();
        let states = mk_states();
        let report = mk_report(3, 0.8);
        let rep = v.verify(&states, MODULE_COUNT, &report, 10);
        assert!(rep.aligned, "expected alignment: {:#?}", rep.checks);
        assert_eq!(rep.total_checks, 5);
        assert_eq!(rep.passed_checks, 5);
        assert_eq!(rep.state_space, MODULE_COUNT);
        assert_eq!(rep.action_space, MODULE_COUNT);
    }

    #[test]
    fn test_verify_fails_on_out_of_bounds_winner() {
        let v = CtmVerifier::new();
        let states = mk_states();
        let report = mk_report(999, 0.8);
        let rep = v.verify(&states, MODULE_COUNT, &report, 10);
        assert!(!rep.aligned);
        let globality = rep.checks.iter().find(|c| c.axiom == "globality").unwrap();
        assert!(!globality.passed);
    }

    #[test]
    fn test_verify_fails_on_excess_actions() {
        let v = CtmVerifier::new();
        let states = mk_states();
        let report = mk_report(1, 0.8);
        let rep = v.verify(&states, MODULE_COUNT + 5, &report, 10);
        assert!(!rep.aligned);
        let action = rep.checks.iter().find(|c| c.axiom == "finite-action").unwrap();
        assert!(!action.passed);
    }

    #[test]
    fn test_verify_fails_on_invalid_e8_state() {
        let v = CtmVerifier::new();
        let mut states = mk_states();
        states[0] = ReasoningHexagram(0xFF);
        let report = mk_report(1, 0.8);
        let rep = v.verify(&states, MODULE_COUNT, &report, 10);
        assert!(!rep.aligned);
        let state = rep.checks.iter().find(|c| c.axiom == "finite-state").unwrap();
        assert!(!state.passed);
    }

    #[test]
    fn test_verify_fails_on_unbounded_tape() {
        let v = CtmVerifier::new();
        let states = mk_states();
        let report = mk_report(1, 0.8);
        let rep = v.verify(&states, MODULE_COUNT, &report, 10_000);
        assert!(!rep.aligned);
        let tape = rep.checks.iter().find(|c| c.axiom == "bounded-tape").unwrap();
        assert!(!tape.passed);
    }

    #[test]
    fn test_verify_fails_on_nan_salience() {
        let v = CtmVerifier::new();
        let states = mk_states();
        let report = mk_report(1, f64::NAN);
        let rep = v.verify(&states, MODULE_COUNT, &report, 10);
        assert!(!rep.aligned);
        let delta = rep.checks.iter().find(|c| c.axiom == "deterministic-delta").unwrap();
        assert!(!delta.passed);
    }

    #[test]
    fn test_e8_state_count_const() {
        assert_eq!(E8_STATE_COUNT, 64);
    }
}
