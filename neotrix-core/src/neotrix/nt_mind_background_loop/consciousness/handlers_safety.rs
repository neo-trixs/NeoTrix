#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_experience::faithfulness_auditor::FaithfulnessAuditor;
use crate::core::nt_core_experience::fggm_safety::FggmSafetyUnifier;
use crate::core::nt_core_hcube::koopman_operator::KoopmanOperator;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_negentropy::dysib_layer::DySIBLayer;

// SAFETY handlers extracted from modules_core.rs
// 6 handlers

impl ConsciousnessIntegration {
    // ── PCC Safety Gate ──

    pub fn handle_pcc_safety_tick(&mut self) -> String {
        let n = self.pcc_safety.obligation_count();
        let v = self.pcc_safety.verified_count();
        log::debug!("MODULES: pcc_safety_tick obligations={} verified={}", n, v);
        format!("pcc_safety:{}_obl_{}_ver", n, v)
    }

    // ── FGGM Safety Unifier ──

    pub fn handle_fggm_safety_tick(&mut self) -> String {
        if self.fggm_safety.is_none() {
            let contracts = vec![
                "negentropy >= 0".to_string(),
                "edit_safety == verified".to_string(),
                "knowledge >= integrity".to_string(),
                "self_improvement is safe".to_string(),
                "no_regression in skills".to_string(),
            ];
            self.fggm_safety = Some(FggmSafetyUnifier::new(contracts, true, 10));
            log::info!("FGGM: initialized with default contracts");
        }
        let fggm = match self.fggm_safety.as_mut() {
            Some(f) => f,
            None => {
                log::error!("[handlers_safety] fggm_safety not initialized");
                return "fggm_safety:unavailable".into();
            }
        };
        let result = fggm.check_proposal("fggm_auto_safety_check");
        log::info!("FGGM: phase_results={:?}", result.failed_phases());
        format!(
            "fggm_safety:passed={}_failed={}",
            result.passed_phases().len(),
            result.failed_phases().len()
        )
    }

    // ── Ball Verifier safety gate ──

    pub fn handle_ball_verifier_tick(&mut self) -> String {
        let r = self.ball_verifier.radius;
        log::debug!("MODULES: ball_verifier_tick radius={:.4}", r);
        format!("ball_verifier:radius={:.4}", r)
    }

    // ── Faithfulness Checker ──

    /// Check evidence citations in the last response for faithfulness to source material.
    /// Runs after response generation; pushes verdict summary into response_buffer.
    /// Requires evidence_manager to be accessible via self (wired through ConsciousnessIntegration).

    pub fn handle_faithfulness_tick(&mut self) -> String {
        let last = self.last_response.as_deref().unwrap_or("");
        if last.is_empty() {
            return "faithfulness:no_response".to_string();
        }
        let summary = self.faithfulness_checker.summary();
        let msg = format!("faithfulness:{}", summary);
        self.response_buffer.push_back(msg.clone());
        msg
    }

    // ── Faithfulness Auditor (P1.22) ──

    pub fn handle_faithfulness_auditor_tick(&mut self) -> String {
        if self.faithfulness_auditor.is_none() {
            self.faithfulness_auditor = Some(FaithfulnessAuditor::new());
            return "faudit:init".into();
        }
        let fa = match self.faithfulness_auditor.as_ref() {
            Some(fa) => fa,
            None => {
                log::error!("[handlers_safety] faithfulness_auditor not initialized");
                return "faithfulness_auditor:unavailable".into();
            }
        };
        let avg = fa.rolling_average();
        if avg > 0.0 {
            format!("faudit:avg={:.3}", avg)
        } else {
            "faudit:ok".into()
        }
    }

    // ── Entity Resolver (P1.24) ──

    pub fn handle_dysib_tick(&mut self) -> String {
        if self.dysib_layer.is_none() {
            self.dysib_layer = Some(DySIBLayer::new());
            return "dysib:init".into();
        }
        let dysib = match self.dysib_layer.as_mut() {
            Some(dysib) => dysib,
            None => {
                log::error!("[handlers_safety] dysib_layer not initialized");
                return "dysib:unavailable".into();
            }
        };
        // Feed attractor state into both past/future to track phase space dynamics
        if self.attractor_state.len() == VSA_DIM {
            let state = self.attractor_state.clone();
            dysib.update(state.clone(), state);
        }
        format!("dysib:info={:.4}", dysib.predictive_info)
    }

    // ── Interaction Trace Predictor (P1.28) ──

    pub fn handle_koopman_tick(&mut self) -> String {
        if self.koopman_operator.is_none() {
            self.koopman_operator = Some(KoopmanOperator::new(4096));
            return "koop:init".into();
        }
        if self.cycle % 100 == 0 && self.attractor_state.len() == 4096 {
            let state = self.attractor_state.clone();
            if let Some(koop) = self.koopman_operator.as_mut() {
                if koop.observation_count() > 3 {
                    let predicted = koop.predict(&state);
                    let err = koop.prediction_error(&predicted, &state);
                    return format!("koop:err={:.4}", err);
                }
            }
        }
        "koop:idle".into()
    }
}
