use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiasType {
    ConfirmationBias,
    Anchoring,
    Availability,
    Overconfidence,
    Hindsight,
    Framing,
    SunkCost,
    GroupThink,
}

#[derive(Debug, Clone)]
pub struct BiasDetection {
    pub bias: BiasType,
    pub strength: f64,
    pub evidence: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MetacognitiveAssessment {
    pub cognitive_load: f64,
    pub uncertainty: f64,
    pub bias_indicators: Vec<BiasDetection>,
    pub reasoning_quality: f64,
    pub requires_intervention: bool,
    pub intervention_reason: String,
    pub confidence_calibration: f64,
}

#[derive(Debug, Clone)]
pub struct MetaIntervention {
    pub action: String,
    pub target_system: String,
    pub priority: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct MetacognitiveController {
    pub assessments: VecDeque<MetacognitiveAssessment>,
    pub interventions: VecDeque<MetaIntervention>,
    pub bias_history: Vec<BiasDetection>,
    pub max_history: usize,
    pub intervention_threshold: f64,
    pub bias_sensitivity: f64,
    pub calibration_window: VecDeque<(f64, f64)>,
    pub cycle_count: u64,
}

impl MetacognitiveController {
    pub fn new() -> Self {
        Self {
            assessments: VecDeque::new(),
            interventions: VecDeque::new(),
            bias_history: Vec::new(),
            max_history: 100,
            intervention_threshold: 0.7,
            bias_sensitivity: 0.5,
            calibration_window: VecDeque::new(),
            cycle_count: 0,
        }
    }

    pub fn monitor(
        &mut self,
        cognitive_load: f64,
        uncertainty: f64,
        reasoning_trace: &[&str],
    ) -> MetacognitiveAssessment {
        self.cycle_count += 1;

        let biases = self.detect_bias(reasoning_trace);
        let bias_penalty: f64 =
            biases.iter().map(|b| b.strength).sum::<f64>() / (biases.len() as f64).max(1.0);
        let quality =
            (1.0 - cognitive_load * 0.3 - uncertainty * 0.3 - bias_penalty * 0.4).clamp(0.0, 1.0);

        let total_bias_strength: f64 = biases.iter().map(|b| b.strength).sum();
        let requires_intervention = cognitive_load > self.intervention_threshold
            || uncertainty > self.intervention_threshold
            || total_bias_strength > self.intervention_threshold * 2.0;

        let mut reasons = Vec::new();
        if cognitive_load > self.intervention_threshold {
            reasons.push("high cognitive load");
        }
        if uncertainty > self.intervention_threshold {
            reasons.push("high uncertainty");
        }
        if total_bias_strength > self.intervention_threshold * 2.0 {
            reasons.push("elevated bias indicators");
        }
        let intervention_reason = if reasons.is_empty() {
            String::new()
        } else {
            reasons.join(", ")
        };

        let assessment = MetacognitiveAssessment {
            cognitive_load,
            uncertainty,
            bias_indicators: biases.clone(),
            reasoning_quality: quality,
            requires_intervention,
            intervention_reason: intervention_reason.clone(),
            confidence_calibration: self.calibration_error(),
        };

        self.assessments.push_back(assessment.clone());
        if self.assessments.len() > self.max_history {
            self.assessments.pop_front();
        }

        for bias in &biases {
            self.bias_history.push(bias.clone());
        }
        if self.bias_history.len() > self.max_history * 2 {
            self.bias_history
                .drain(0..self.bias_history.len() - self.max_history * 2);
        }

        if requires_intervention {
            if let Some(intervention) = self.decide_intervention(&assessment) {
                self.interventions.push_back(intervention);
                if self.interventions.len() > self.max_history {
                    self.interventions.pop_front();
                }
            }
        }

        assessment
    }

    pub fn decide_intervention(
        &mut self,
        assessment: &MetacognitiveAssessment,
    ) -> Option<MetaIntervention> {
        if !assessment.requires_intervention {
            return None;
        }

        let total_bias_strength: f64 = assessment.bias_indicators.iter().map(|b| b.strength).sum();

        if assessment.cognitive_load > assessment.uncertainty
            && assessment.cognitive_load > total_bias_strength
        {
            Some(MetaIntervention {
                action: "delegate_to_system1".to_string(),
                target_system: "system1".to_string(),
                priority: assessment.cognitive_load,
                reason: "High cognitive load — offloading to intuitive processing".to_string(),
            })
        } else if assessment.uncertainty > total_bias_strength {
            Some(MetaIntervention {
                action: "engage_deliberate_reasoning".to_string(),
                target_system: "system3".to_string(),
                priority: assessment.uncertainty,
                reason: "High uncertainty — engaging meta-cognitive analysis".to_string(),
            })
        } else {
            Some(MetaIntervention {
                action: "activate_analytical_reasoning".to_string(),
                target_system: "system2".to_string(),
                priority: total_bias_strength,
                reason: format!(
                    "Bias detected ({:.2} total) — activating System 2 oversight",
                    total_bias_strength
                ),
            })
        }
    }

    pub fn detect_bias(&self, reasoning_trace: &[&str]) -> Vec<BiasDetection> {
        let mut biases = Vec::new();
        let sensitivity = self.bias_sensitivity;

        let confirmation_strength =
            self.scan_for_keywords(reasoning_trace, &["because I think"], sensitivity);
        if confirmation_strength > 0.0 {
            biases.push(BiasDetection {
                bias: BiasType::ConfirmationBias,
                strength: confirmation_strength,
                evidence: vec!["reasoning contains 'because I think' pattern".to_string()],
                timestamp: self.cycle_count,
            });
        }

        let anchoring_strength =
            self.scan_for_keywords(reasoning_trace, &["first", "initial"], sensitivity);
        if anchoring_strength > 0.0 {
            biases.push(BiasDetection {
                bias: BiasType::Anchoring,
                strength: anchoring_strength,
                evidence: vec!["anchoring keywords detected (first/initial)".to_string()],
                timestamp: self.cycle_count,
            });
        }

        let availability_strength =
            self.scan_for_keywords(reasoning_trace, &["recent", "remember"], sensitivity);
        if availability_strength > 0.0 {
            biases.push(BiasDetection {
                bias: BiasType::Availability,
                strength: availability_strength,
                evidence: vec!["availability keywords detected (recent/remember)".to_string()],
                timestamp: self.cycle_count,
            });
        }

        let overconfidence_strength =
            self.scan_for_keywords(reasoning_trace, &["definitely", "certainly"], sensitivity);
        if overconfidence_strength > 0.0 {
            biases.push(BiasDetection {
                bias: BiasType::Overconfidence,
                strength: overconfidence_strength,
                evidence: vec![
                    "overconfidence keywords detected (definitely/certainly)".to_string()
                ],
                timestamp: self.cycle_count,
            });
        }

        biases
    }

    fn scan_for_keywords(&self, trace: &[&str], keywords: &[&str], sensitivity: f64) -> f64 {
        let total: usize = trace
            .iter()
            .filter(|line| keywords.iter().any(|kw| line.to_lowercase().contains(kw)))
            .count();

        if total == 0 {
            return 0.0;
        }

        let raw = (total as f64) / (trace.len() as f64).max(1.0);
        (raw * sensitivity * 2.0).clamp(0.0, 1.0)
    }

    pub fn record_calibration(&mut self, predicted: f64, actual: f64) {
        self.calibration_window.push_back((predicted, actual));
        if self.calibration_window.len() > self.max_history {
            self.calibration_window.pop_front();
        }
    }

    pub fn calibration_error(&self) -> f64 {
        if self.calibration_window.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .calibration_window
            .iter()
            .map(|(p, a)| (p - a).abs())
            .sum();
        sum / self.calibration_window.len() as f64
    }

    pub fn calibration_summary(&self) -> (f64, f64) {
        let ece = self.calibration_error();
        let meta_d = (3.0 * (1.0 - ece)).clamp(0.0, 3.0);
        (meta_d, ece)
    }

    pub fn assessment_count(&self) -> usize {
        self.assessments.len()
    }

    pub fn intervention_count(&self) -> usize {
        self.interventions.len()
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.intervention_threshold = threshold.clamp(0.0, 1.0);
    }

    pub fn set_bias_sensitivity(&mut self, sensitivity: f64) {
        self.bias_sensitivity = sensitivity.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.assessments.clear();
        self.interventions.clear();
        self.bias_history.clear();
        self.calibration_window.clear();
        self.cycle_count = 0;
    }
}

impl Default for MetacognitiveController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_controller_defaults() {
        let mc = MetacognitiveController::new();
        assert_eq!(mc.max_history, 100);
        assert_eq!(mc.intervention_threshold, 0.7);
        assert_eq!(mc.bias_sensitivity, 0.5);
        assert_eq!(mc.cycle_count, 0);
        assert!(mc.assessments.is_empty());
        assert!(mc.interventions.is_empty());
    }

    #[test]
    fn test_monitor_basic_assessment() {
        let mut mc = MetacognitiveController::new();
        let trace = vec!["step one", "step two", "step three"];
        let assessment = mc.monitor(0.3, 0.2, &trace);
        assert_eq!(assessment.cognitive_load, 0.3);
        assert_eq!(assessment.uncertainty, 0.2);
        assert!(!assessment.requires_intervention);
        assert!(assessment.reasoning_quality > 0.5);
        assert_eq!(mc.cycle_count, 1);
    }

    #[test]
    fn test_monitor_high_load_triggers_intervention() {
        let mut mc = MetacognitiveController::new();
        let trace = vec!["step one"];
        let assessment = mc.monitor(0.9, 0.1, &trace);
        assert!(assessment.requires_intervention);
        assert!(assessment.intervention_reason.contains("cognitive load"));
        assert_eq!(mc.intervention_count(), 1);
    }

    #[test]
    fn test_monitor_high_uncertainty_triggers_intervention() {
        let mut mc = MetacognitiveController::new();
        let trace = vec!["step one"];
        let assessment = mc.monitor(0.1, 0.95, &trace);
        assert!(assessment.requires_intervention);
        assert!(assessment.intervention_reason.contains("uncertainty"));
        assert_eq!(mc.intervention_count(), 1);
    }

    #[test]
    fn test_detect_confirmation_bias() {
        let mc = MetacognitiveController::new();
        let trace = vec!["because I think this is correct"];
        let biases = mc.detect_bias(&trace);
        assert!(biases.iter().any(|b| b.bias == BiasType::ConfirmationBias));
    }

    #[test]
    fn test_detect_anchoring() {
        let mc = MetacognitiveController::new();
        let trace = vec!["the first value we saw", "initial estimate"];
        let biases = mc.detect_bias(&trace);
        assert!(biases.iter().any(|b| b.bias == BiasType::Anchoring));
    }

    #[test]
    fn test_detect_availability() {
        let mc = MetacognitiveController::new();
        let trace = vec!["recent events show", "I remember that"];
        let biases = mc.detect_bias(&trace);
        assert!(biases.iter().any(|b| b.bias == BiasType::Availability));
    }

    #[test]
    fn test_detect_overconfidence() {
        let mc = MetacognitiveController::new();
        let trace = vec!["definitely correct", "certainly the best"];
        let biases = mc.detect_bias(&trace);
        assert!(biases.iter().any(|b| b.bias == BiasType::Overconfidence));
    }

    #[test]
    fn test_decide_intervention_returns_none_when_not_needed() {
        let mut mc = MetacognitiveController::new();
        let assessment = MetacognitiveAssessment {
            cognitive_load: 0.2,
            uncertainty: 0.2,
            bias_indicators: vec![],
            reasoning_quality: 0.9,
            requires_intervention: false,
            intervention_reason: String::new(),
            confidence_calibration: 0.0,
        };
        assert!(mc.decide_intervention(&assessment).is_none());
    }

    #[test]
    fn test_decide_intervention_high_load_returns_s1_intervention() {
        let mut mc = MetacognitiveController::new();
        let assessment = MetacognitiveAssessment {
            cognitive_load: 0.9,
            uncertainty: 0.3,
            bias_indicators: vec![],
            reasoning_quality: 0.4,
            requires_intervention: true,
            intervention_reason: "high cognitive load".to_string(),
            confidence_calibration: 0.0,
        };
        let intervention = mc.decide_intervention(&assessment).unwrap();
        assert_eq!(intervention.target_system, "system1");
        assert!(intervention.action.contains("system1"));
    }

    #[test]
    fn test_decide_intervention_high_uncertainty_returns_s3_intervention() {
        let mut mc = MetacognitiveController::new();
        let assessment = MetacognitiveAssessment {
            cognitive_load: 0.3,
            uncertainty: 0.9,
            bias_indicators: vec![],
            reasoning_quality: 0.4,
            requires_intervention: true,
            intervention_reason: "high uncertainty".to_string(),
            confidence_calibration: 0.0,
        };
        let intervention = mc.decide_intervention(&assessment).unwrap();
        assert_eq!(intervention.target_system, "system3");
        assert!(intervention.action.contains("system3"));
    }

    #[test]
    fn test_record_calibration() {
        let mut mc = MetacognitiveController::new();
        mc.record_calibration(0.8, 0.75);
        mc.record_calibration(0.9, 0.85);
        assert_eq!(mc.calibration_window.len(), 2);
    }

    #[test]
    fn test_calibration_error() {
        let mut mc = MetacognitiveController::new();
        mc.record_calibration(0.8, 0.8);
        mc.record_calibration(0.9, 0.9);
        assert!((mc.calibration_error() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_calibration_error_mismatch() {
        let mut mc = MetacognitiveController::new();
        mc.record_calibration(0.9, 0.5);
        let error = mc.calibration_error();
        assert!((error - 0.4).abs() < 1e-10, "expected ~0.4, got {}", error);
    }

    #[test]
    fn test_set_threshold() {
        let mut mc = MetacognitiveController::new();
        mc.set_threshold(0.5);
        assert_eq!(mc.intervention_threshold, 0.5);
        mc.set_threshold(1.5);
        assert_eq!(mc.intervention_threshold, 1.0);
        mc.set_threshold(-0.1);
        assert_eq!(mc.intervention_threshold, 0.0);
    }

    #[test]
    fn test_set_bias_sensitivity() {
        let mut mc = MetacognitiveController::new();
        mc.set_bias_sensitivity(0.8);
        assert_eq!(mc.bias_sensitivity, 0.8);
        mc.set_bias_sensitivity(1.5);
        assert_eq!(mc.bias_sensitivity, 1.0);
        mc.set_bias_sensitivity(-0.1);
        assert_eq!(mc.bias_sensitivity, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut mc = MetacognitiveController::new();
        let trace = vec!["step one"];
        mc.monitor(0.9, 0.1, &trace);
        assert_eq!(mc.cycle_count, 1);
        mc.reset();
        assert_eq!(mc.cycle_count, 0);
        assert!(mc.assessments.is_empty());
        assert!(mc.interventions.is_empty());
        assert!(mc.bias_history.is_empty());
        assert!(mc.calibration_window.is_empty());
    }

    #[test]
    fn test_intervention_count() {
        let mut mc = MetacognitiveController::new();
        assert_eq!(mc.intervention_count(), 0);
        mc.monitor(0.9, 0.1, &["test"]);
        assert_eq!(mc.intervention_count(), 1);
        mc.monitor(0.95, 0.1, &["test"]);
        assert_eq!(mc.intervention_count(), 2);
    }
}
