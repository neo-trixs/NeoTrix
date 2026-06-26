// G394 + G395 + G402: Self-calibrating judge + verification-gated agent loop + interlocking verification gates
use crate::core::nt_core_hcube::VsaVector;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub description: String,
    pub predicted_gain: f64,
    pub realized_gain: Option<f64>,
    pub proposed_by: String,
    pub vsa_signature: VsaVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeCalibration {
    pub total_predictions: u64,
    pub calibration_error_ema: f64,
    pub overconfidence_rate: f64,
    pub underconfidence_rate: f64,
    pub last_calibration: f64,
}

impl JudgeCalibration {
    pub fn new() -> Self {
        Self {
            total_predictions: 0,
            calibration_error_ema: 0.0,
            overconfidence_rate: 0.0,
            underconfidence_rate: 0.0,
            last_calibration: 1.0,
        }
    }

    pub fn record_outcome(&mut self, predicted: f64, realized: f64) {
        let error = (predicted - realized).abs();
        let alpha = 0.05;
        self.calibration_error_ema = alpha * error + (1.0 - alpha) * self.calibration_error_ema;
        if realized < predicted - 0.1 {
            self.overconfidence_rate = (self.overconfidence_rate * self.total_predictions as f64
                + 1.0)
                / (self.total_predictions as f64 + 1.0);
        }
        if realized > predicted + 0.1 {
            self.underconfidence_rate = (self.underconfidence_rate * self.total_predictions as f64
                + 1.0)
                / (self.total_predictions as f64 + 1.0);
        }
        self.total_predictions += 1;
        self.last_calibration = 1.0 / (1.0 + self.calibration_error_ema);
    }

    pub fn calibrated_score(&self, raw_score: f64) -> f64 {
        let penalty = self.overconfidence_rate * 0.3;
        raw_score * (1.0 - penalty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    Syntax,
    Semantic,
    Safety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate: GateType,
    pub passed: bool,
    pub reason: String,
    pub severity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub proposal_id: u64,
    pub gates_passed: Vec<GateResult>,
    pub all_passed: bool,
    pub judge_score: f64,
    pub calibrated_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationGate {
    pub judge: JudgeCalibration,
    pub active_proposals: VecDeque<Proposal>,
    pub completed_proposals: Vec<Proposal>,
    pub gate_history: VecDeque<VerificationResult>,
    pub history_capacity: usize,
}

impl VerificationGate {
    pub fn new() -> Self {
        Self {
            judge: JudgeCalibration::new(),
            active_proposals: VecDeque::new(),
            completed_proposals: Vec::new(),
            gate_history: VecDeque::with_capacity(1000),
            history_capacity: 1000,
        }
    }

    pub fn submit_proposal(&mut self, proposal: Proposal) {
        self.active_proposals.push_back(proposal);
    }

    pub fn evaluate_proposal(&mut self, proposal: &Proposal) -> VerificationResult {
        let syntax = self.check_syntax_gate(proposal);
        let semantic = self.check_semantic_gate(proposal);
        let safety = self.check_safety_gate(proposal);

        let gates_passed = vec![syntax, semantic, safety];
        let all_passed = gates_passed.iter().all(|g| g.passed);
        let raw_score = if all_passed {
            proposal.predicted_gain
        } else {
            0.0
        };
        let calibrated_score = self.judge.calibrated_score(raw_score);

        VerificationResult {
            proposal_id: proposal.id,
            gates_passed,
            all_passed,
            judge_score: raw_score,
            calibrated_score,
        }
    }

    fn check_syntax_gate(&self, proposal: &Proposal) -> GateResult {
        let has_vsa = !proposal.vsa_signature.as_bytes().is_empty()
            && proposal.vsa_signature.as_bytes().iter().any(|&b| b != 0);
        let has_non_empty_desc = !proposal.description.is_empty();
        let passed = has_vsa && has_non_empty_desc && proposal.predicted_gain.is_finite();
        GateResult {
            gate: GateType::Syntax,
            passed,
            reason: if passed {
                "VSA signature valid, description non-empty, gain finite".into()
            } else {
                "Missing VSA signature or empty description or invalid gain".into()
            },
            severity: if passed { 0 } else { 2 },
        }
    }

    fn check_semantic_gate(&self, proposal: &Proposal) -> GateResult {
        let gain_valid = (0.0..=1.0).contains(&proposal.predicted_gain);
        let has_source = !proposal.proposed_by.is_empty();
        GateResult {
            gate: GateType::Semantic,
            passed: gain_valid && has_source,
            reason: if gain_valid && has_source {
                format!(
                    "Gain {:.3} in valid range, source: {}",
                    proposal.predicted_gain, proposal.proposed_by
                )
            } else {
                format!(
                    "Gain {:.3} out of [0,1] or missing source",
                    proposal.predicted_gain
                )
            },
            severity: if gain_valid && has_source { 1 } else { 2 },
        }
    }

    fn check_safety_gate(&self, proposal: &Proposal) -> GateResult {
        let is_safe = proposal.predicted_gain >= -0.5;
        let is_reversible =
            !proposal.description.contains("delete") && !proposal.description.contains("destroy");
        GateResult {
            gate: GateType::Safety,
            passed: is_safe && is_reversible,
            reason: if is_safe && is_reversible {
                "Gain not catastrophic, action appears reversible".into()
            } else {
                "Potential safety concern: gain too negative or destructive action".into()
            },
            severity: if !is_safe { 3 } else { 0 },
        }
    }

    pub fn finalize_proposal(&mut self, proposal: Proposal, realized_gain: f64) {
        let mut completed = proposal;
        completed.realized_gain = Some(realized_gain);
        self.judge
            .record_outcome(completed.predicted_gain, realized_gain);
        self.completed_proposals.push(completed);
    }

    pub fn calibration_stats(&self) -> JudgeCalibration {
        self.judge.clone()
    }
}
