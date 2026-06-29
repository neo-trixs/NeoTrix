use crate::core::nt_core_consciousness::epistemic_honesty::EpistemicHonesty;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct HumilityConfig {
    pub defer_threshold: f64,
    pub unknown_unknown_window: usize,
    pub compositional_warning_threshold: f64,
    pub uncertainty_expression_levels: usize,
    pub domain_expertise_min_samples: usize,
}

impl Default for HumilityConfig {
    fn default() -> Self {
        Self {
            defer_threshold: 0.3,
            unknown_unknown_window: 50,
            compositional_warning_threshold: 0.5,
            uncertainty_expression_levels: 5,
            domain_expertise_min_samples: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UncertaintyLevel {
    Certain(f64),
    Confident(f64),
    ModeratelyConfident(f64),
    Uncertain(f64),
    VeryUncertain(f64),
    DontKnow(f64),
}

impl UncertaintyLevel {
    pub fn from_confidence(conf: f64) -> Self {
        if conf > 0.95 {
            Self::Certain(conf)
        } else if conf > 0.80 {
            Self::Confident(conf)
        } else if conf > 0.60 {
            Self::ModeratelyConfident(conf)
        } else if conf > 0.40 {
            Self::Uncertain(conf)
        } else if conf > 0.20 {
            Self::VeryUncertain(conf)
        } else {
            Self::DontKnow(conf)
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Certain(_) => "certain",
            Self::Confident(_) => "confident",
            Self::ModeratelyConfident(_) => "moderately confident",
            Self::Uncertain(_) => "uncertain",
            Self::VeryUncertain(_) => "very uncertain",
            Self::DontKnow(_) => "don't know",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpistemicBoundary {
    pub domain: String,
    pub samples: usize,
    pub accuracy: f64,
    pub avg_confidence: f64,
    pub calibration_error: f64,
    pub is_outside_expertise: bool,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct HumilityAssessment {
    pub raw_confidence: f64,
    pub calibrated_confidence: f64,
    pub uncertainty_level: UncertaintyLevel,
    pub recommendation: HumilityRecommendation,
    pub domain_boundary: Option<EpistemicBoundary>,
    pub unknown_unknown_risk: f64,
    pub composition_risk: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumilityRecommendation {
    AnswerConfidently,
    AnswerWithCaution(&'static str),
    Defer(&'static str),
    RequestClarification,
    EscalateToHuman,
}

#[derive(Debug, Clone)]
pub struct EpistemicHumility {
    pub config: HumilityConfig,
    pub honesty: EpistemicHonesty,
    pub domain_boundaries: HashMap<String, EpistemicBoundary>,
    pub unknown_unknown_tracker: UnknownUnknownTracker,
    pub composition_tracker: CompositionTracker,
    pub deferral_history: Vec<(String, f64, bool)>,
}

#[derive(Debug, Clone)]
pub struct UnknownUnknownTracker {
    pub surprise_events: VecDeque<SurpriseEvent>,
    pub detection_rate: f64,
    pub feature_weights: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct SurpriseEvent {
    pub domain: String,
    pub predicted_confidence: f64,
    pub actual_outcome: bool,
    pub surprise_magnitude: f64,
    pub detected_feature: String,
}

#[derive(Debug, Clone)]
pub struct CompositionTracker {
    pub task_complexity: usize,
    pub composition_history: VecDeque<(usize, bool)>,
    pub predicted_degradation: f64,
}

pub struct HumilityStats {
    pub total_assessments: usize,
    pub total_deferrals: usize,
    pub total_surprises: usize,
    pub deferral_accuracy: f64,
    pub unknown_unknown_rate: f64,
    pub composition_degradation: f64,
    pub domain_coverage: usize,
    pub overall_calibration: f64,
}

impl EpistemicHumility {
    pub fn new(config: HumilityConfig, honesty: EpistemicHonesty) -> Self {
        Self {
            config,
            honesty,
            domain_boundaries: HashMap::new(),
            unknown_unknown_tracker: UnknownUnknownTracker {
                surprise_events: VecDeque::with_capacity(100),
                detection_rate: 0.0,
                feature_weights: Vec::new(),
            },
            composition_tracker: CompositionTracker {
                task_complexity: 1,
                composition_history: VecDeque::with_capacity(50),
                predicted_degradation: 0.0,
            },
            deferral_history: Vec::new(),
        }
    }

    pub fn assess(
        &mut self,
        domain: &str,
        confidence: f64,
        task_complexity: usize,
    ) -> HumilityAssessment {
        let calibrated = self.honesty.honest_confidence(confidence);
        let boundary = self.domain_boundaries.get(domain).cloned();
        let unknown_unknown_risk = self.estimate_unknown_unknown_risk(domain, confidence);
        let composition_risk = self.estimate_composition_risk(task_complexity);

        let adjusted =
            calibrated * (1.0 - composition_risk * 0.5) * (1.0 - unknown_unknown_risk * 0.3);
        let level = UncertaintyLevel::from_confidence(adjusted);

        let should_defer = adjusted < self.config.defer_threshold;
        let outside_expertise = boundary.as_ref().map_or(true, |b| b.is_outside_expertise);
        let high_composition_risk = composition_risk > self.config.compositional_warning_threshold;

        let recommendation = if should_defer || outside_expertise {
            if outside_expertise && adjusted < 0.1 {
                HumilityRecommendation::EscalateToHuman
            } else if should_defer && confidence > 0.0 && confidence < 0.05 {
                HumilityRecommendation::RequestClarification
            } else if high_composition_risk {
                HumilityRecommendation::Defer("I can reason about parts, but the composition exceeds my reliability threshold")
            } else if outside_expertise {
                HumilityRecommendation::Defer("This domain is outside my calibrated expertise")
            } else {
                HumilityRecommendation::Defer("My confidence is below the reliable threshold")
            }
        } else if adjusted < 0.6 {
            HumilityRecommendation::AnswerWithCaution(
                "My confidence is moderate — verify independently",
            )
        } else {
            HumilityRecommendation::AnswerConfidently
        };

        HumilityAssessment {
            raw_confidence: confidence,
            calibrated_confidence: adjusted,
            uncertainty_level: level,
            recommendation,
            domain_boundary: boundary,
            unknown_unknown_risk,
            composition_risk,
        }
    }

    pub fn record_outcome(
        &mut self,
        domain: &str,
        confidence: f64,
        was_correct: bool,
        task_complexity: usize,
    ) {
        self.honesty.calibrate(confidence, was_correct);
        if confidence > 0.7 && !was_correct {
            self.record_surprise(domain, confidence, was_correct);
        }
        let adjusted = self.honesty.honest_confidence(confidence);
        if adjusted < self.config.defer_threshold {
            self.deferral_history
                .push((domain.to_string(), confidence, was_correct));
        }
        self.composition_tracker
            .composition_history
            .push_back((task_complexity, was_correct));
        if self.composition_tracker.composition_history.len() > 50 {
            self.composition_tracker.composition_history.pop_front();
        }
        self.update_composition_degradation();
        self.calibrate_domain_boundary(domain);
    }

    pub fn record_surprise(&mut self, domain: &str, confidence: f64, actual_outcome: bool) {
        let magnitude = if actual_outcome {
            (1.0 - confidence).max(0.0)
        } else {
            confidence
        };
        let event = SurpriseEvent {
            domain: domain.to_string(),
            predicted_confidence: confidence,
            actual_outcome,
            surprise_magnitude: magnitude,
            detected_feature: format!("high_confidence_failure:{}", domain),
        };
        self.unknown_unknown_tracker
            .surprise_events
            .push_back(event);
        if self.unknown_unknown_tracker.surprise_events.len() > self.config.unknown_unknown_window {
            self.unknown_unknown_tracker.surprise_events.pop_front();
        }
        let n = self.unknown_unknown_tracker.surprise_events.len();
        let recent_surprises = self
            .unknown_unknown_tracker
            .surprise_events
            .iter()
            .filter(|e| e.surprise_magnitude > 0.3)
            .count();
        self.unknown_unknown_tracker.detection_rate = if n > 0 {
            recent_surprises as f64 / n as f64
        } else {
            0.0
        };
    }

    pub fn calibrate_domain_boundary(&mut self, domain: &str) -> Option<EpistemicBoundary> {
        let boundary = self
            .domain_boundaries
            .entry(domain.to_string())
            .or_insert_with(|| EpistemicBoundary {
                domain: domain.to_string(),
                samples: 0,
                accuracy: 0.0,
                avg_confidence: 0.0,
                calibration_error: 0.0,
                is_outside_expertise: true,
                recommendation: "defer".to_string(),
            });
        boundary.samples += 1;
        if boundary.samples >= self.config.domain_expertise_min_samples {
            boundary.is_outside_expertise = false;
            let report = self.honesty.report();
            boundary.calibration_error = report.calibration_error;
            boundary.recommendation = if boundary.calibration_error < 0.1 && boundary.accuracy > 0.8
            {
                "answer confidently".to_string()
            } else if boundary.calibration_error < 0.2 && boundary.accuracy > 0.6 {
                "answer with caution".to_string()
            } else {
                "defer".to_string()
            };
        }
        Some(boundary.clone())
    }

    pub fn weakest_domains(&self, top_n: usize) -> Vec<EpistemicBoundary> {
        let mut domains: Vec<EpistemicBoundary> =
            self.domain_boundaries.values().cloned().collect();
        domains.sort_by(|a, b| {
            a.accuracy
                .partial_cmp(&b.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        domains.truncate(top_n);
        domains
    }

    pub fn strongest_domains(&self, top_n: usize) -> Vec<EpistemicBoundary> {
        let mut domains: Vec<EpistemicBoundary> =
            self.domain_boundaries.values().cloned().collect();
        domains.sort_by(|a, b| {
            b.accuracy
                .partial_cmp(&a.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        domains.truncate(top_n);
        domains
    }

    pub fn should_defer(&self, domain: &str) -> bool {
        self.domain_boundaries.get(domain).map_or(true, |b| {
            b.is_outside_expertise || b.accuracy < self.config.defer_threshold
        })
    }

    pub fn humility_expression(&self, assessment: &HumilityAssessment) -> String {
        match assessment.recommendation {
            HumilityRecommendation::AnswerConfidently => {
                format!(
                    "I'm {} about this answer.",
                    assessment.uncertainty_level.label()
                )
            }
            HumilityRecommendation::AnswerWithCaution(caveat) => {
                format!("I'm {} — {}.", assessment.uncertainty_level.label(), caveat)
            }
            HumilityRecommendation::Defer(reason) => {
                format!(
                    "I don't know. {} ({:.0}% confidence).",
                    reason,
                    assessment.calibrated_confidence * 100.0
                )
            }
            HumilityRecommendation::RequestClarification => {
                "I don't have enough information — could you clarify?".to_string()
            }
            HumilityRecommendation::EscalateToHuman => {
                "This falls outside my expertise — you should consult a human specialist."
                    .to_string()
            }
        }
    }

    pub fn stats(&self) -> HumilityStats {
        let total = self.deferral_history.len();
        let correct_deferrals = self
            .deferral_history
            .iter()
            .filter(|(_, _, correct)| *correct)
            .count();
        let deferral_accuracy = if total > 0 {
            correct_deferrals as f64 / total as f64
        } else {
            0.0
        };
        let total_surprises = self.unknown_unknown_tracker.surprise_events.len();
        let domain_coverage = self
            .domain_boundaries
            .values()
            .filter(|b| !b.is_outside_expertise)
            .count();
        let report = self.honesty.report();
        HumilityStats {
            total_assessments: report.total_predictions,
            total_deferrals: total,
            total_surprises,
            deferral_accuracy,
            unknown_unknown_rate: self.unknown_unknown_tracker.detection_rate,
            composition_degradation: self.composition_tracker.predicted_degradation,
            domain_coverage,
            overall_calibration: 1.0 - report.calibration_error.min(1.0),
        }
    }

    fn estimate_unknown_unknown_risk(&self, _domain: &str, confidence: f64) -> f64 {
        let base = self.unknown_unknown_tracker.detection_rate;
        let confidence_penalty = if confidence > 0.9 { 0.15 } else { 0.0 };
        (base + confidence_penalty).min(1.0)
    }

    fn estimate_composition_risk(&self, task_complexity: usize) -> f64 {
        let degradation = self.composition_tracker.predicted_degradation;
        let complexity_factor = (task_complexity as f64 - 1.0).max(0.0) * 0.1;
        (degradation * complexity_factor + complexity_factor * 0.3).min(1.0)
    }

    fn update_composition_degradation(&mut self) {
        let n = self.composition_tracker.composition_history.len();
        if n < 5 {
            return;
        }
        let mut complexity_success: HashMap<usize, (usize, usize)> = HashMap::new();
        for (complexity, success) in &self.composition_tracker.composition_history {
            let entry = complexity_success.entry(*complexity).or_insert((0, 0));
            entry.0 += 1;
            if *success {
                entry.1 += 1;
            }
        }
        if complexity_success.len() < 2 {
            return;
        }
        let mut max_complexity = 0;
        let mut min_complexity = usize::MAX;
        for c in complexity_success.keys() {
            if *c > max_complexity {
                max_complexity = *c;
            }
            if *c < min_complexity {
                min_complexity = *c;
            }
        }
        if max_complexity == min_complexity {
            return;
        }
        let simple_data = complexity_success.get(&min_complexity);
        let complex_data = complexity_success.get(&max_complexity);
        if let (Some((s_count, s_correct)), Some((c_count, c_correct))) =
            (simple_data, complex_data)
        {
            let simple_acc = *s_correct as f64 / *s_count as f64;
            let complex_acc = *c_correct as f64 / *c_count as f64;
            let diff = simple_acc - complex_acc;
            self.composition_tracker.predicted_degradation = diff.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::epistemic_honesty::HonestyConfig;

    #[test]
    fn test_default_config() {
        let config = HumilityConfig::default();
        assert_eq!(config.defer_threshold, 0.3);
        assert_eq!(config.unknown_unknown_window, 50);
        assert_eq!(config.compositional_warning_threshold, 0.5);
        assert_eq!(config.uncertainty_expression_levels, 5);
        assert_eq!(config.domain_expertise_min_samples, 10);
    }

    #[test]
    fn test_uncertainty_level_certain() {
        let level = UncertaintyLevel::from_confidence(0.96);
        assert_eq!(level.label(), "certain");
    }

    #[test]
    fn test_uncertainty_level_confident() {
        let level = UncertaintyLevel::from_confidence(0.85);
        assert_eq!(level.label(), "confident");
    }

    #[test]
    fn test_uncertainty_level_moderately_confident() {
        let level = UncertaintyLevel::from_confidence(0.70);
        assert_eq!(level.label(), "moderately confident");
    }

    #[test]
    fn test_uncertainty_level_uncertain() {
        let level = UncertaintyLevel::from_confidence(0.50);
        assert_eq!(level.label(), "uncertain");
    }

    #[test]
    fn test_uncertainty_level_very_uncertain() {
        let level = UncertaintyLevel::from_confidence(0.30);
        assert_eq!(level.label(), "very uncertain");
    }

    #[test]
    fn test_uncertainty_level_dont_know() {
        let level = UncertaintyLevel::from_confidence(0.10);
        assert_eq!(level.label(), "don't know");
    }

    #[test]
    fn test_assess_high_confidence() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        let assessment = humility.assess("math", 0.95, 1);
        assert_eq!(
            assessment.recommendation,
            HumilityRecommendation::AnswerConfidently
        );
    }

    #[test]
    fn test_assess_low_confidence_defers() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        let assessment = humility.assess("unknown_domain", 0.15, 1);
        assert!(matches!(
            assessment.recommendation,
            HumilityRecommendation::Defer(_)
        ));
    }

    #[test]
    fn test_assess_unknown_domain_defers() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        let assessment = humility.assess("novel_domain", 0.5, 1);
        assert!(
            matches!(
                assessment.recommendation,
                HumilityRecommendation::AnswerWithCaution(_)
            ) || matches!(assessment.recommendation, HumilityRecommendation::Defer(_))
        );
    }

    #[test]
    fn test_record_outcome_updates_boundary() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        for _ in 0..15 {
            humility.record_outcome("physics", 0.8, true, 1);
        }
        let boundary = humility.domain_boundaries.get("physics");
        assert!(boundary.is_some());
        let b = boundary.unwrap();
        assert_eq!(b.samples, 15);
        assert!(!b.is_outside_expertise);
    }

    #[test]
    fn test_record_surprise_tracks_events() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.record_surprise("biology", 0.9, false);
        assert_eq!(humility.unknown_unknown_tracker.surprise_events.len(), 1);
        assert!(humility.unknown_unknown_tracker.detection_rate > 0.0);
    }

    #[test]
    fn test_weakest_domains_ordering() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.domain_boundaries.insert(
            "a".to_string(),
            EpistemicBoundary {
                domain: "a".to_string(),
                samples: 10,
                accuracy: 0.9,
                avg_confidence: 0.85,
                calibration_error: 0.05,
                is_outside_expertise: false,
                recommendation: "answer confidently".to_string(),
            },
        );
        humility.domain_boundaries.insert(
            "b".to_string(),
            EpistemicBoundary {
                domain: "b".to_string(),
                samples: 10,
                accuracy: 0.5,
                avg_confidence: 0.6,
                calibration_error: 0.2,
                is_outside_expertise: false,
                recommendation: "defer".to_string(),
            },
        );
        let weakest = humility.weakest_domains(1);
        assert_eq!(weakest.len(), 1);
        assert_eq!(weakest[0].domain, "b");
    }

    #[test]
    fn test_strongest_domains_ordering() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.domain_boundaries.insert(
            "a".to_string(),
            EpistemicBoundary {
                domain: "a".to_string(),
                samples: 10,
                accuracy: 0.9,
                avg_confidence: 0.85,
                calibration_error: 0.05,
                is_outside_expertise: false,
                recommendation: "answer confidently".to_string(),
            },
        );
        humility.domain_boundaries.insert(
            "b".to_string(),
            EpistemicBoundary {
                domain: "b".to_string(),
                samples: 10,
                accuracy: 0.5,
                avg_confidence: 0.6,
                calibration_error: 0.2,
                is_outside_expertise: false,
                recommendation: "defer".to_string(),
            },
        );
        let strongest = humility.strongest_domains(1);
        assert_eq!(strongest.len(), 1);
        assert_eq!(strongest[0].domain, "a");
    }

    #[test]
    fn test_should_defer_unknown_domain() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        assert!(humility.should_defer("unknown"));
    }

    #[test]
    fn test_should_not_defer_known_good_domain() {
        let config = HumilityConfig {
            domain_expertise_min_samples: 1,
            ..Default::default()
        };
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.domain_boundaries.insert(
            "math".to_string(),
            EpistemicBoundary {
                domain: "math".to_string(),
                samples: 20,
                accuracy: 0.95,
                avg_confidence: 0.9,
                calibration_error: 0.03,
                is_outside_expertise: false,
                recommendation: "answer confidently".to_string(),
            },
        );
        assert!(!humility.should_defer("math"));
    }

    #[test]
    fn test_humility_expression_self_assured() {
        let assessment = HumilityAssessment {
            raw_confidence: 0.98,
            calibrated_confidence: 0.97,
            uncertainty_level: UncertaintyLevel::Certain(0.97),
            recommendation: HumilityRecommendation::AnswerConfidently,
            domain_boundary: None,
            unknown_unknown_risk: 0.02,
            composition_risk: 0.0,
        };
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        let expr = humility.humility_expression(&assessment);
        assert!(expr.contains("certain"));
    }

    #[test]
    fn test_humility_expression_defer() {
        let assessment = HumilityAssessment {
            raw_confidence: 0.2,
            calibrated_confidence: 0.15,
            uncertainty_level: UncertaintyLevel::DontKnow(0.15),
            recommendation: HumilityRecommendation::Defer(
                "My confidence is below the reliable threshold",
            ),
            domain_boundary: None,
            unknown_unknown_risk: 0.3,
            composition_risk: 0.0,
        };
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        let expr = humility.humility_expression(&assessment);
        assert!(expr.contains("don't know"));
    }

    #[test]
    fn test_humility_expression_escalate() {
        let assessment = HumilityAssessment {
            raw_confidence: 0.05,
            calibrated_confidence: 0.03,
            uncertainty_level: UncertaintyLevel::DontKnow(0.03),
            recommendation: HumilityRecommendation::EscalateToHuman,
            domain_boundary: None,
            unknown_unknown_risk: 0.8,
            composition_risk: 0.0,
        };
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        let expr = humility.humility_expression(&assessment);
        assert!(expr.contains("human specialist"));
    }

    #[test]
    fn test_record_outcome_triggers_surprise_on_high_conf_failure() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.record_outcome("physics", 0.9, false, 1);
        assert!(humility.unknown_unknown_tracker.surprise_events.len() >= 1);
    }

    #[test]
    fn test_stats_basic() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        let stats = humility.stats();
        assert_eq!(stats.total_assessments, 0);
        assert_eq!(stats.total_deferrals, 0);
    }

    #[test]
    fn test_stats_after_records() {
        let config = HumilityConfig {
            domain_expertise_min_samples: 1,
            ..Default::default()
        };
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.record_outcome("math", 0.2, false, 1);
        humility.record_outcome("math", 0.3, true, 1);
        let stats = humility.stats();
        assert_eq!(stats.total_assessments, 2);
    }

    #[test]
    fn test_composition_degradation_updates() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        for _ in 0..10 {
            humility.record_outcome("logic", 0.8, true, 1);
        }
        for _ in 0..10 {
            humility.record_outcome("logic", 0.8, false, 5);
        }
        assert!(humility.composition_tracker.predicted_degradation >= 0.0);
    }

    #[test]
    fn test_domain_boundary_calibrates_after_min_samples() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        for i in 0..12 {
            let correct = i % 2 == 0;
            humility.record_outcome("chemistry", if correct { 0.8 } else { 0.4 }, correct, 1);
        }
        let boundary = humility.domain_boundaries.get("chemistry").unwrap();
        assert!(!boundary.is_outside_expertise);
    }

    #[test]
    fn test_estimate_unknown_unknown_risk_increases_with_confidence() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        let risk_low = humility.estimate_unknown_unknown_risk("test", 0.5);
        let risk_high = humility.estimate_unknown_unknown_risk("test", 0.95);
        assert!(risk_high >= risk_low);
    }

    #[test]
    fn test_estimate_composition_risk_increases_with_complexity() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let humility = EpistemicHumility::new(config, honesty);
        let risk_simple = humility.estimate_composition_risk(1);
        let risk_complex = humility.estimate_composition_risk(10);
        assert!(risk_complex > risk_simple);
    }

    #[test]
    fn test_assessment_contains_domain_boundary_when_known() {
        let config = HumilityConfig::default();
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.domain_boundaries.insert(
            "history".to_string(),
            EpistemicBoundary {
                domain: "history".to_string(),
                samples: 15,
                accuracy: 0.75,
                avg_confidence: 0.7,
                calibration_error: 0.1,
                is_outside_expertise: false,
                recommendation: "answer with caution".to_string(),
            },
        );
        let assessment = humility.assess("history", 0.7, 1);
        assert!(assessment.domain_boundary.is_some());
    }

    #[test]
    fn test_deferral_accuracy_in_stats() {
        let config = HumilityConfig {
            defer_threshold: 0.5,
            domain_expertise_min_samples: 1,
            ..Default::default()
        };
        let honesty = EpistemicHonesty::new(HonestyConfig::default());
        let mut humility = EpistemicHumility::new(config, honesty);
        humility.record_outcome("math", 0.2, false, 1);
        humility.record_outcome("math", 0.1, true, 1);
        let stats = humility.stats();
        assert!(stats.deferral_accuracy > 0.0);
    }

    #[test]
    fn test_serde_uncertainty_level() {
        let level = UncertaintyLevel::Uncertain(0.5);
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: UncertaintyLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}
