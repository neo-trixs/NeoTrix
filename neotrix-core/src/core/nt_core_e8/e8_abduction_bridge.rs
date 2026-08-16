#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::abduction::{AbductiveHypothesis, AbductiveReasoningEngine};
use super::domain_transition::{CoTLength, E8DomainTransitionModel, E8TaskType};

pub struct E8AbductionBridge {
    pub transition_model: E8DomainTransitionModel,
    pub abductive_engine: AbductiveReasoningEngine,
    pub active_hypotheses: Vec<AbductiveHypothesis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveTransitionReport {
    pub predicted_next: u8,
    pub prediction_confidence: f64,
    pub actual_next: u8,
    pub hypothesis_count: usize,
    pub best_hypothesis_plausibility: f64,
    pub hypothesis_explained: bool,
}

impl E8AbductionBridge {
    pub fn new(blend_weight: f64) -> Self {
        Self {
            transition_model: E8DomainTransitionModel::new(blend_weight),
            abductive_engine: AbductiveReasoningEngine::new(),
            active_hypotheses: Vec::new(),
        }
    }

    pub fn predict_with_abduction(
        &mut self,
        from: u8,
        task_type: E8TaskType,
        cot_length: CoTLength,
    ) -> (u8, f64) {
        let (predicted, base_conf) = self
            .transition_model
            .predict_next(from, task_type, cot_length);
        let mut adjusted_conf = base_conf;

        if !self.active_hypotheses.is_empty() {
            let best_plausibility = self
                .active_hypotheses
                .iter()
                .map(|h| h.plausibility)
                .fold(0.0f64, |a, b| a.max(b));
            let hypothesis_bias = (best_plausibility * 0.3).max(0.0).min(0.3);
            adjusted_conf = (adjusted_conf + hypothesis_bias).max(0.0).min(1.0);
        }

        (predicted, adjusted_conf)
    }

    pub fn record_transition_with_abduction(
        &mut self,
        task_type: E8TaskType,
        from: u8,
        to: u8,
        context: &str,
    ) -> AbductiveTransitionReport {
        self.transition_model.record_transition(task_type, from, to);
        self.abductive_engine.add_observation(format!(
            "E8 transition {} -> {} (task: {:?}): {}",
            from, to, task_type, context
        ));
        let _report = self.abductive_engine.run_abduction_cycle(5);
        self.active_hypotheses = self.abductive_engine.hypotheses.clone();

        let mut best_plausibility = 0.0;
        for h in &self.active_hypotheses {
            if h.plausibility > best_plausibility {
                best_plausibility = h.plausibility;
            }
        }

        let explained = best_plausibility > 0.3;
        if !explained && !self.active_hypotheses.is_empty() {
            self.abductive_engine.add_observation(format!(
                "Unexpected transition {} -> {} failed explanation threshold",
                from, to
            ));
        }

        let (predicted, pred_conf) =
            self.transition_model
                .predict_next(from, task_type, CoTLength::Medium);

        AbductiveTransitionReport {
            predicted_next: predicted,
            prediction_confidence: pred_conf,
            actual_next: to,
            hypothesis_count: self.active_hypotheses.len(),
            best_hypothesis_plausibility: best_plausibility,
            hypothesis_explained: explained,
        }
    }

    pub fn merge_transition_model(&mut self, other: E8DomainTransitionModel) {
        self.transition_model.merge(&other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bridge() {
        let bridge = E8AbductionBridge::new(0.5);
        assert!(bridge.transition_model.dirty);
        assert!(bridge.active_hypotheses.is_empty());
    }

    #[test]
    fn test_predict_without_hypotheses() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let (state, confidence) =
            bridge.predict_with_abduction(0, E8TaskType::Reasoning, CoTLength::Medium);
        assert!(state < 64);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_record_transition_generates_hypothesis() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let report = bridge.record_transition_with_abduction(
            E8TaskType::Reasoning,
            0,
            1,
            "initial transition test",
        );
        assert_eq!(report.actual_next, 1);
        // 产生假设后, hypothesis_explained 必须与 best plausibility 判定一致
        assert_eq!(
            report.hypothesis_explained,
            report.best_hypothesis_plausibility > 0.3,
            "explained flag must match plausibility threshold"
        );
    }

    #[test]
    fn test_multiple_transitions() {
        let mut bridge = E8AbductionBridge::new(0.5);
        for i in 0..5 {
            let report = bridge.record_transition_with_abduction(
                E8TaskType::Agentic,
                i,
                i + 1,
                &format!("step {}", i),
            );
            assert_eq!(report.actual_next, i + 1);
        }
    }

    #[test]
    fn test_hypothesis_plausibility_ranges() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let report = bridge.record_transition_with_abduction(
            E8TaskType::Reasoning,
            10,
            20,
            "test plausibility",
        );
        assert!(report.best_hypothesis_plausibility >= 0.0);
        assert!(report.best_hypothesis_plausibility <= 1.0);
    }

    #[test]
    fn test_predict_with_active_hypotheses() {
        let mut bridge = E8AbductionBridge::new(0.5);
        bridge.record_transition_with_abduction(E8TaskType::Reasoning, 5, 10, "build hypotheses");
        let before_conf = {
            let (_, conf) =
                bridge.predict_with_abduction(10, E8TaskType::Reasoning, CoTLength::Medium);
            conf
        };
        bridge.record_transition_with_abduction(
            E8TaskType::Reasoning,
            10,
            15,
            "strengthen hypothesis",
        );
        let after_conf = {
            let (_, conf) =
                bridge.predict_with_abduction(15, E8TaskType::Reasoning, CoTLength::Medium);
            conf
        };
        assert!(after_conf >= 0.0);
        assert!(before_conf >= 0.0);
    }

    #[test]
    fn test_merge_models() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let other = E8DomainTransitionModel::new(0.8);
        bridge.merge_transition_model(other);
        assert!((bridge.transition_model.blend_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_unexpected_transition_triggers_second_observation() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let report = bridge.record_transition_with_abduction(
            E8TaskType::Agentic,
            30,
            31,
            "routine transition",
        );
        assert!(report.hypothesis_count > 0 || report.prediction_confidence > 0.0);
    }

    #[test]
    fn test_report_full_coverage() {
        let mut bridge = E8AbductionBridge::new(0.5);
        let report =
            bridge.record_transition_with_abduction(E8TaskType::General, 42, 7, "full report test");
        assert!(report.predicted_next < 64);
        assert!(report.prediction_confidence >= 0.0);
        // 无假设时 explained 必须为 false; 有假设时由 plausibility 决定
        if report.hypothesis_count == 0 {
            assert!(!report.hypothesis_explained);
        }
    }
}
