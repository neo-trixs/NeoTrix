// REVIVED Task 1 — dead_code removed 2026-06-24

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Appraisal dimensions — the core axes used by OCC theory
/// to evaluate an event's emotional significance.
#[derive(Debug, Clone, Copy)]
pub struct AppraisalDimensions {
    /// Alignment with goals [0, 1]
    pub desirability: f64,
    /// Subjective probability of outcome [0, 1]
    pub likelihood: f64,
    /// Expected cognitive / physical effort [0, 1]
    pub effort: f64,
    /// Predictability of outcome [0, 1]
    pub certainty: f64,
    /// Personal influence possible [0, 1]
    pub controllability: f64,
    /// -1 = other, 0 = circumstance, +1 = self
    pub agency: f64,
    /// Alignment with norms / legitimacy [0, 1]
    pub legitimacy: f64,
}

impl Default for AppraisalDimensions {
    fn default() -> Self {
        Self {
            desirability: 0.5,
            likelihood: 0.5,
            effort: 0.5,
            certainty: 0.5,
            controllability: 0.5,
            agency: 0.0,
            legitimacy: 0.5,
        }
    }
}

/// The 22 OCC emotion labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmotionLabel {
    Joy,
    Distress,
    Hope,
    Fear,
    Pride,
    Shame,
    Admiration,
    Reproach,
    Gratitude,
    Anger,
    Gratification,
    Remorse,
    Love,
    Hate,
    Relief,
    Disappointment,
    Satisfaction,
    FearsConfirmed,
    HappyFor,
    Resentment,
    Gloating,
    Pity,
}

impl fmt::Display for EmotionLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Joy => write!(f, "Joy"),
            Self::Distress => write!(f, "Distress"),
            Self::Hope => write!(f, "Hope"),
            Self::Fear => write!(f, "Fear"),
            Self::Pride => write!(f, "Pride"),
            Self::Shame => write!(f, "Shame"),
            Self::Admiration => write!(f, "Admiration"),
            Self::Reproach => write!(f, "Reproach"),
            Self::Gratitude => write!(f, "Gratitude"),
            Self::Anger => write!(f, "Anger"),
            Self::Gratification => write!(f, "Gratification"),
            Self::Remorse => write!(f, "Remorse"),
            Self::Love => write!(f, "Love"),
            Self::Hate => write!(f, "Hate"),
            Self::Relief => write!(f, "Relief"),
            Self::Disappointment => write!(f, "Disappointment"),
            Self::Satisfaction => write!(f, "Satisfaction"),
            Self::FearsConfirmed => write!(f, "FearsConfirmed"),
            Self::HappyFor => write!(f, "HappyFor"),
            Self::Resentment => write!(f, "Resentment"),
            Self::Gloating => write!(f, "Gloating"),
            Self::Pity => write!(f, "Pity"),
        }
    }
}

impl EmotionLabel {
    fn all() -> [Self; 22] {
        [
            Self::Joy,
            Self::Distress,
            Self::Hope,
            Self::Fear,
            Self::Pride,
            Self::Shame,
            Self::Admiration,
            Self::Reproach,
            Self::Gratitude,
            Self::Anger,
            Self::Gratification,
            Self::Remorse,
            Self::Love,
            Self::Hate,
            Self::Relief,
            Self::Disappointment,
            Self::Satisfaction,
            Self::FearsConfirmed,
            Self::HappyFor,
            Self::Resentment,
            Self::Gloating,
            Self::Pity,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Joy => "Joy",
            Self::Distress => "Distress",
            Self::Hope => "Hope",
            Self::Fear => "Fear",
            Self::Pride => "Pride",
            Self::Shame => "Shame",
            Self::Admiration => "Admiration",
            Self::Reproach => "Reproach",
            Self::Gratitude => "Gratitude",
            Self::Anger => "Anger",
            Self::Gratification => "Gratification",
            Self::Remorse => "Remorse",
            Self::Love => "Love",
            Self::Hate => "Hate",
            Self::Relief => "Relief",
            Self::Disappointment => "Disappointment",
            Self::Satisfaction => "Satisfaction",
            Self::FearsConfirmed => "FearsConfirmed",
            Self::HappyFor => "HappyFor",
            Self::Resentment => "Resentment",
            Self::Gloating => "Gloating",
            Self::Pity => "Pity",
        }
    }
}

/// A complete appraisal record for a single event.
#[derive(Debug, Clone)]
pub struct AppraisalEvent {
    pub event_vsa: Vec<u8>,
    pub dimensions: AppraisalDimensions,
    pub emotions: Vec<(EmotionLabel, f64)>,
    pub timestamp: u64,
}

/// Classifies appraisal dimensions into OCC emotion labels.
/// Uses rule-based OCC mapping and maintains prototype VSA vectors.
#[derive(Debug, Clone)]
pub struct EmotionClassifier {
    pub prototype_vectors: HashMap<String, Vec<u8>>,
}

impl Default for EmotionClassifier {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_to_seed(name: &str) -> u64 {
    let bytes: Vec<u8> = name.bytes().collect();
    bytes
        .iter()
        .fold(42u64, |acc, b| acc.wrapping_mul(31).wrapping_add(*b as u64))
}

impl EmotionClassifier {
    pub fn new() -> Self {
        let mut prototype_vectors = HashMap::new();
        for emotion in EmotionLabel::all() {
            let seed = hash_to_seed(emotion.name());
            let vec = QuantizedVSA::seeded_random(seed, VSA_DIM);
            prototype_vectors.insert(emotion.name().to_string(), vec);
        }
        Self { prototype_vectors }
    }

    pub fn classify(&self, dim: &AppraisalDimensions) -> Vec<(EmotionLabel, f64)> {
        let mut candidates: Vec<(EmotionLabel, f64)> = Vec::new();

        let d = dim.desirability;
        let l = dim.likelihood;
        let c = dim.certainty;
        let co = dim.controllability;
        let a = dim.agency;
        let le = dim.legitimacy;

        // Core valenced responses
        if d > 0.6 {
            candidates.push((EmotionLabel::Joy, d));
        }
        if d < 0.3 {
            candidates.push((EmotionLabel::Distress, 1.0 - d));
        }

        // Prospect-based
        if d > 0.6 && l > 0.6 {
            candidates.push((EmotionLabel::Hope, (d + l) * 0.5));
        }
        if d < 0.3 && l > 0.6 {
            candidates.push((EmotionLabel::Fear, ((1.0 - d) + l) * 0.5));
        }

        // Agency-based — self
        if a > 0.5 && d > 0.6 && le > 0.5 {
            candidates.push((EmotionLabel::Pride, (a + d + le) / 3.0));
        }
        if a > 0.5 && d < 0.3 && le < 0.4 {
            candidates.push((EmotionLabel::Remorse, (a + (1.0 - d) + (1.0 - le)) / 3.0));
        }
        if a > 0.5 && d < 0.3 {
            candidates.push((EmotionLabel::Shame, (a + (1.0 - d)) * 0.5));
        }

        // Agency-based — other
        if a < -0.5 && d > 0.6 {
            candidates.push((EmotionLabel::Admiration, ((-a) + d) * 0.5));
        }
        if a < -0.5 && d < 0.3 && le < 0.4 {
            candidates.push((EmotionLabel::Anger, ((-a) + (1.0 - d) + (1.0 - le)) / 3.0));
        }
        if a < -0.5 && d < 0.3 {
            candidates.push((EmotionLabel::Reproach, ((-a) + (1.0 - d)) * 0.5));
        }
        if a < -0.5 && d > 0.6 && le > 0.5 {
            candidates.push((EmotionLabel::Gratitude, ((-a) + d + le) / 3.0));
        }

        // Compound attributions
        if a > 0.5 && d > 0.6 && le > 0.6 {
            candidates.push((EmotionLabel::Gratification, a * 0.3 + d * 0.4 + le * 0.3));
        }

        // Well-being compound
        if d > 0.7 && a > 0.0 && co < 0.5 {
            candidates.push((
                EmotionLabel::Love,
                d * 0.5 + (1.0 - co) * 0.3 + a.max(0.0) * 0.2,
            ));
        }
        if d < 0.2 && a < -0.2 && co < 0.3 {
            candidates.push((
                EmotionLabel::Hate,
                (1.0 - d) * 0.5 + (-a.min(0.0)) * 0.3 + (1.0 - co) * 0.2,
            ));
        }

        // Certainty + outcome
        if d > 0.6 && c > 0.6 && l < 0.3 {
            candidates.push((EmotionLabel::Relief, (d + c + (1.0 - l)) / 3.0));
        }
        if d < 0.3 && c > 0.5 && l > 0.5 {
            candidates.push((EmotionLabel::Disappointment, ((1.0 - d) + c + l) / 3.0));
        }
        if d < 0.3 && l > 0.7 && c > 0.5 {
            candidates.push((EmotionLabel::FearsConfirmed, ((1.0 - d) + l + c) / 3.0));
        }

        // Controllability-based
        if co > 0.6 && c > 0.3 && d > 0.5 {
            candidates.push((EmotionLabel::Satisfaction, (co + c + d) / 3.0));
        }
        if co > 0.6 && c > 0.3 && d <= 0.5 {
            candidates.push((EmotionLabel::Resentment, (co + c + (1.0 - d)) / 3.0));
        }

        // Fortunes-of-others
        if d > 0.5 && a > 0.0 && le > 0.6 {
            candidates.push((EmotionLabel::HappyFor, (d + a.max(0.0) + le) / 3.0));
        }
        if d > 0.5 && a > 0.5 && le < 0.3 {
            candidates.push((EmotionLabel::Gloating, d * 0.5 + a * 0.3 + (1.0 - le) * 0.2));
        }
        if d < 0.3 && a.abs() < 0.5 && l < 0.5 {
            candidates.push((
                EmotionLabel::Pity,
                (1.0 - d) * 0.5 + (1.0 - a.abs()) * 0.3 + (1.0 - l) * 0.2,
            ));
        }

        // Deduplicate: keep highest confidence per label
        let mut merged: HashMap<EmotionLabel, f64> = HashMap::new();
        for (label, conf) in candidates {
            let entry = merged.entry(label).or_insert(0.0);
            *entry = (*entry).max(conf);
        }

        let mut result: Vec<(EmotionLabel, f64)> =
            merged.into_iter().filter(|(_, conf)| *conf > 0.1).collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(3);
        result
    }

    pub fn emotion_count() -> usize {
        22
    }

    pub fn prototype_for(&self, emotion: &EmotionLabel) -> Option<&[u8]> {
        self.prototype_vectors
            .get(emotion.name())
            .map(|v| v.as_slice())
    }
}

/// Assessment of coping potential based on appraisal dimensions.
#[derive(Debug, Clone)]
pub struct CopingAssessment {
    pub problem_focused_score: f64,
    pub emotion_focused_score: f64,
    pub avoidance_potential: f64,
    pub recommended_strategy: String,
}

impl CopingAssessment {
    pub fn assess(dimensions: &AppraisalDimensions) -> Self {
        let problem_focused = dimensions.controllability * 0.7 + dimensions.certainty * 0.3;
        let emotion_focused =
            (1.0 - dimensions.controllability) * 0.6 + dimensions.desirability.abs() * 0.4;
        let avoidance = dimensions.likelihood * 0.5 + (1.0 - dimensions.controllability) * 0.5;

        let strategy = if problem_focused > emotion_focused && problem_focused > avoidance {
            "Act".to_string()
        } else if emotion_focused > problem_focused && emotion_focused > avoidance {
            "Reappraise".to_string()
        } else if avoidance > problem_focused && avoidance > emotion_focused {
            "Avoid".to_string()
        } else if problem_focused >= emotion_focused {
            "Act".to_string()
        } else {
            "Reappraise".to_string()
        };

        Self {
            problem_focused_score: problem_focused.clamp(0.0, 1.0),
            emotion_focused_score: emotion_focused.clamp(0.0, 1.0),
            avoidance_potential: avoidance.clamp(0.0, 1.0),
            recommended_strategy: strategy,
        }
    }
}

/// Cognitive strategy modulator driven by emotional state.
#[derive(Debug, Clone)]
pub struct StrategyModulator {
    /// Modification to exploration tendency [-1, 1]
    pub exploration_bonus: f64,
    /// Modification to risk acceptance [-1, 1]
    pub risk_tolerance: f64,
    /// Shift toward System1 (intuitive) processing [0, 1]
    pub system1_bias: f64,
    /// Shift toward detailed / analytical processing [0, 1]
    pub detail_bias: f64,
}

impl Default for StrategyModulator {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyModulator {
    pub fn new() -> Self {
        Self {
            exploration_bonus: 0.0,
            risk_tolerance: 0.0,
            system1_bias: 0.5,
            detail_bias: 0.5,
        }
    }

    pub fn modulate(&mut self, emotions: &[(EmotionLabel, f64)]) {
        self.exploration_bonus = 0.0;
        self.risk_tolerance = 0.0;
        self.system1_bias = 0.5;
        self.detail_bias = 0.5;

        let mut total_weight = 0.0;
        for (label, conf) in emotions {
            let w = *conf;
            total_weight += w;
            match label {
                EmotionLabel::Joy => {
                    self.exploration_bonus += 0.2 * w;
                    self.risk_tolerance += 0.1 * w;
                    self.system1_bias += 0.2 * w;
                }
                EmotionLabel::Hope => {
                    self.exploration_bonus += 0.15 * w;
                    self.risk_tolerance += 0.15 * w;
                }
                EmotionLabel::Fear => {
                    self.exploration_bonus += -0.2 * w;
                    self.risk_tolerance += -0.3 * w;
                    self.detail_bias += 0.3 * w;
                }
                EmotionLabel::Anger => {
                    self.risk_tolerance += 0.3 * w;
                    self.detail_bias += -0.2 * w;
                }
                EmotionLabel::Pride | EmotionLabel::Gratification => {
                    self.system1_bias += 0.15 * w;
                }
                EmotionLabel::Shame | EmotionLabel::Remorse => {
                    self.detail_bias += 0.2 * w;
                    self.exploration_bonus += -0.1 * w;
                }
                EmotionLabel::Disappointment | EmotionLabel::Distress => {
                    self.exploration_bonus += -0.1 * w;
                    self.detail_bias += 0.15 * w;
                }
                EmotionLabel::Relief => {
                    self.exploration_bonus += 0.1 * w;
                    self.system1_bias += 0.1 * w;
                }
                _ => {}
            }
        }

        if total_weight > 0.0 {
            self.exploration_bonus /= total_weight.max(1.0);
            self.risk_tolerance /= total_weight.max(1.0);
            self.system1_bias = 0.5 + (self.system1_bias - 0.5) / total_weight.max(1.0);
            self.detail_bias = 0.5 + (self.detail_bias - 0.5) / total_weight.max(1.0);
        }

        self.exploration_bonus = self.exploration_bonus.clamp(-1.0, 1.0);
        self.risk_tolerance = self.risk_tolerance.clamp(-1.0, 1.0);
        self.system1_bias = self.system1_bias.clamp(0.0, 1.0);
        self.detail_bias = self.detail_bias.clamp(0.0, 1.0);
    }
}

/// OCC appraisal engine — the main orchestrator that evaluates events
/// through the full appraisal → emotion → coping → strategy pipeline.
#[derive(Debug, Clone)]
pub struct AppraisalEngine {
    pub classifier: EmotionClassifier,
    pub last_assessment: Option<CopingAssessment>,
    pub last_modulation: Option<StrategyModulator>,
    pub event_history: VecDeque<AppraisalEvent>,
    max_history: usize,
    next_timestamp: u64,
}

impl Default for AppraisalEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AppraisalEngine {
    pub fn new() -> Self {
        Self {
            classifier: EmotionClassifier::new(),
            last_assessment: None,
            last_modulation: None,
            event_history: VecDeque::with_capacity(64),
            max_history: 256,
            next_timestamp: 1,
        }
    }

    pub fn evaluate(
        &mut self,
        event_vsa: Vec<u8>,
        dimensions: AppraisalDimensions,
    ) -> AppraisalEvent {
        let emotions = self.classifier.classify(&dimensions);
        let assessment = CopingAssessment::assess(&dimensions);
        let mut modulation = StrategyModulator::new();
        modulation.modulate(&emotions);

        let timestamp = self.next_timestamp;
        self.next_timestamp += 1;

        let event = AppraisalEvent {
            event_vsa,
            dimensions,
            emotions,
            timestamp,
        };

        self.last_assessment = Some(assessment);
        self.last_modulation = Some(modulation);
        self.event_history.push_back(event.clone());

        while self.event_history.len() > self.max_history {
            self.event_history.pop_front();
        }

        event
    }

    pub fn assess_coping(&self, dimensions: &AppraisalDimensions) -> CopingAssessment {
        CopingAssessment::assess(dimensions)
    }

    pub fn modulate_strategy(&self, emotions: &[(EmotionLabel, f64)]) -> StrategyModulator {
        let mut modulator = StrategyModulator::new();
        modulator.modulate(emotions);
        modulator
    }

    pub fn dominant_emotion(&self) -> Option<(EmotionLabel, f64)> {
        self.event_history
            .back()
            .and_then(|event| event.emotions.first().copied())
    }

    pub fn emotional_state_vector(&self) -> HashMap<String, f64> {
        let mut state = HashMap::new();
        if let Some(last) = self.event_history.back() {
            for (label, conf) in &last.emotions {
                state.insert(label.to_string(), *conf);
            }
        }
        state
    }

    pub fn event_count(&self) -> usize {
        self.event_history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa() -> Vec<u8> {
        QuantizedVSA::seeded_random(999, VSA_DIM)
    }

    // 1
    #[test]
    fn test_appraisal_dimensions_default() {
        let d = AppraisalDimensions::default();
        assert!((d.desirability - 0.5).abs() < 1e-9);
        assert!((d.likelihood - 0.5).abs() < 1e-9);
        assert!((d.effort - 0.5).abs() < 1e-9);
        assert!((d.certainty - 0.5).abs() < 1e-9);
        assert!((d.controllability - 0.5).abs() < 1e-9);
        assert!((d.agency - 0.0).abs() < 1e-9);
        assert!((d.legitimacy - 0.5).abs() < 1e-9);
    }

    // 2
    #[test]
    fn test_emotion_label_display() {
        let labels = [
            (EmotionLabel::Joy, "Joy"),
            (EmotionLabel::Distress, "Distress"),
            (EmotionLabel::Hope, "Hope"),
            (EmotionLabel::Fear, "Fear"),
            (EmotionLabel::Pride, "Pride"),
            (EmotionLabel::Shame, "Shame"),
            (EmotionLabel::Admiration, "Admiration"),
            (EmotionLabel::Reproach, "Reproach"),
            (EmotionLabel::Gratitude, "Gratitude"),
            (EmotionLabel::Anger, "Anger"),
            (EmotionLabel::Gratification, "Gratification"),
            (EmotionLabel::Remorse, "Remorse"),
            (EmotionLabel::Love, "Love"),
            (EmotionLabel::Hate, "Hate"),
            (EmotionLabel::Relief, "Relief"),
            (EmotionLabel::Disappointment, "Disappointment"),
            (EmotionLabel::Satisfaction, "Satisfaction"),
            (EmotionLabel::FearsConfirmed, "FearsConfirmed"),
            (EmotionLabel::HappyFor, "HappyFor"),
            (EmotionLabel::Resentment, "Resentment"),
            (EmotionLabel::Gloating, "Gloating"),
            (EmotionLabel::Pity, "Pity"),
        ];
        for (label, expected) in &labels {
            assert_eq!(format!("{}", label), *expected);
        }
    }

    // 3
    #[test]
    fn test_emotion_classifier_new() {
        let c = EmotionClassifier::new();
        assert_eq!(c.prototype_vectors.len(), 22);
        for emotion in EmotionLabel::all() {
            let proto = c.prototype_for(&emotion);
            assert!(proto.is_some(), "Missing prototype for {}", emotion);
            assert_eq!(proto.unwrap().len(), VSA_DIM);
        }
    }

    // 4
    #[test]
    fn test_emotion_classifier_joy() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.85,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions.iter().any(|(l, _)| matches!(l, EmotionLabel::Joy)));
    }

    // 5
    #[test]
    fn test_emotion_classifier_distress() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.1,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Distress)));
    }

    // 6
    #[test]
    fn test_emotion_classifier_hope() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.8,
            likelihood: 0.75,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Hope)));
    }

    // 7
    #[test]
    fn test_emotion_classifier_fear() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.1,
            likelihood: 0.85,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Fear)));
    }

    // 8
    #[test]
    fn test_emotion_classifier_pride() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.85,
            agency: 0.7,
            legitimacy: 0.8,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Pride)));
    }

    // 9
    #[test]
    fn test_emotion_classifier_shame() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.1,
            agency: 0.7,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Shame)));
    }

    // 10
    #[test]
    fn test_coping_assessment_controllable() {
        let dim = AppraisalDimensions {
            controllability: 0.9,
            certainty: 0.8,
            ..Default::default()
        };
        let ca = CopingAssessment::assess(&dim);
        assert!(ca.problem_focused_score > ca.emotion_focused_score);
        assert_eq!(ca.recommended_strategy, "Act");
    }

    // 11
    #[test]
    fn test_coping_assessment_uncontrollable() {
        let dim = AppraisalDimensions {
            controllability: 0.1,
            desirability: 0.5,
            certainty: 0.2,
            ..Default::default()
        };
        let ca = CopingAssessment::assess(&dim);
        assert!(ca.emotion_focused_score > ca.problem_focused_score);
    }

    // 12
    #[test]
    fn test_strategy_modulator_joy() {
        let mut sm = StrategyModulator::new();
        sm.modulate(&[(EmotionLabel::Joy, 1.0)]);
        assert!(sm.exploration_bonus > 0.0);
        assert!(sm.system1_bias > 0.5);
    }

    // 13
    #[test]
    fn test_strategy_modulator_fear() {
        let mut sm = StrategyModulator::new();
        sm.modulate(&[(EmotionLabel::Fear, 1.0)]);
        assert!(sm.detail_bias > 0.5);
        assert!(sm.exploration_bonus < 0.0);
        assert!(sm.risk_tolerance < 0.0);
    }

    // 14
    #[test]
    fn test_appraisal_engine_new() {
        let engine = AppraisalEngine::new();
        assert_eq!(engine.event_count(), 0);
        assert!(engine.last_assessment.is_none());
        assert!(engine.last_modulation.is_none());
    }

    // 15
    #[test]
    fn test_appraisal_engine_evaluate() {
        let mut engine = AppraisalEngine::new();
        let vsa = test_vsa();
        let dim = AppraisalDimensions {
            desirability: 0.9,
            likelihood: 0.3,
            certainty: 0.8,
            controllability: 0.7,
            ..Default::default()
        };
        let event = engine.evaluate(vsa, dim);
        assert!(event.timestamp > 0);
        assert!(!event.emotions.is_empty());
        assert!(engine.last_assessment.is_some());
        assert!(engine.last_modulation.is_some());
    }

    // 16
    #[test]
    fn test_dominant_emotion() {
        let mut engine = AppraisalEngine::new();
        let vsa = test_vsa();
        let dim = AppraisalDimensions {
            desirability: 0.9,
            ..Default::default()
        };
        engine.evaluate(vsa, dim);
        let dominant = engine.dominant_emotion();
        assert!(dominant.is_some());
        let (label, conf) = dominant.unwrap();
        assert!(conf > 0.0);
        // Joy should be dominant for high desirability
        assert_eq!(label, EmotionLabel::Joy);
    }

    // 17
    #[test]
    fn test_event_history_cap() {
        let mut engine = AppraisalEngine::new();
        // Override max_history for test
        engine.max_history = 5;
        for _ in 0..10 {
            let vsa = test_vsa();
            let dim = AppraisalDimensions::default();
            engine.evaluate(vsa, dim);
        }
        assert_eq!(engine.event_count(), 5);
        // Oldest should be gone; timestamp advances from 1, so after 10 events
        // with max_history=5 we have timestamps 6..10
        let first_ts = engine.event_history.front().unwrap().timestamp;
        assert!(
            first_ts > 5,
            "Expected old events to be evicted, got timestamp {}",
            first_ts
        );
    }

    // 18
    #[test]
    fn test_emotional_state_vector() {
        let mut engine = AppraisalEngine::new();
        let vsa = test_vsa();
        let dim = AppraisalDimensions {
            desirability: 0.9,
            ..Default::default()
        };
        engine.evaluate(vsa, dim);
        let state = engine.emotional_state_vector();
        assert!(!state.is_empty());
        assert!(state.contains_key("Joy"));
    }

    // 19
    #[test]
    fn test_event_count() {
        let mut engine = AppraisalEngine::new();
        assert_eq!(engine.event_count(), 0);
        for _ in 0..3 {
            let vsa = test_vsa();
            engine.evaluate(vsa, AppraisalDimensions::default());
        }
        assert_eq!(engine.event_count(), 3);
    }

    // 20
    #[test]
    fn test_emotion_classifier_anger() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.1,
            agency: -0.7,
            legitimacy: 0.2,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(
            emotions
                .iter()
                .any(|(l, _)| matches!(l, EmotionLabel::Anger)),
            "Expected Anger in {:?}",
            emotions
        );
    }

    // 21
    #[test]
    fn test_emotion_classifier_gratitude() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            desirability: 0.8,
            agency: -0.7,
            legitimacy: 0.8,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Gratitude)));
    }

    // 22
    #[test]
    fn test_emotion_classifier_satisfaction() {
        let c = EmotionClassifier::new();
        let dim = AppraisalDimensions {
            controllability: 0.8,
            certainty: 0.7,
            desirability: 0.75,
            ..Default::default()
        };
        let emotions = c.classify(&dim);
        assert!(emotions
            .iter()
            .any(|(l, _)| matches!(l, EmotionLabel::Satisfaction)));
    }

    // 23
    #[test]
    fn test_strategy_modulator_anger() {
        let mut sm = StrategyModulator::new();
        sm.modulate(&[(EmotionLabel::Anger, 1.0)]);
        assert!(sm.risk_tolerance > 0.0);
        assert!(sm.detail_bias < 0.5);
    }

    // 24
    #[test]
    fn test_strategy_modulator_shame() {
        let mut sm = StrategyModulator::new();
        sm.modulate(&[(EmotionLabel::Shame, 1.0)]);
        assert!(sm.detail_bias > 0.5);
        assert!(sm.exploration_bonus < 0.0);
    }

    // 25
    #[test]
    fn test_coping_assessment_avoid() {
        let dim = AppraisalDimensions {
            likelihood: 0.9,
            controllability: 0.1,
            desirability: 0.1,
            ..Default::default()
        };
        let ca = CopingAssessment::assess(&dim);
        // avoidance = 0.9*0.5 + 0.9*0.5 = 0.9
        // problem_focused = 0.1*0.7 + 0.5*0.3 = 0.22
        // emotion_focused = 0.9*0.6 + 0.1*0.4 = 0.58
        // But we need avoidance > emotion_focused, so likelihood needs to be high enough
        assert_eq!(
            ca.recommended_strategy, "Avoid",
            "Expected Avoid strategy, got {}",
            ca.recommended_strategy
        );
    }

    // 26
    #[test]
    fn test_emotion_count() {
        assert_eq!(EmotionClassifier::emotion_count(), 22);
    }

    // 27
    #[test]
    fn test_prototype_vectors_stable_seeds() {
        let c1 = EmotionClassifier::new();
        let c2 = EmotionClassifier::new();
        let proto1 = c1.prototype_for(&EmotionLabel::Joy).unwrap().to_vec();
        let proto2 = c2.prototype_for(&EmotionLabel::Joy).unwrap().to_vec();
        assert_eq!(proto1, proto2, "Prototype vectors must be deterministic");
    }
}
