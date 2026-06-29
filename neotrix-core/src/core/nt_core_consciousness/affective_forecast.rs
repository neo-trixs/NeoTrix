// REVIVED Task 1 — dead_code removed 2026-06-24

use crate::core::nt_core_consciousness::AppraisalDimensions;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct ForecastEmotion {
    pub emotion_label: String,
    pub expected_intensity: f64,
    pub expected_duration_ms: u64,
    pub corrected_intensity: f64,
    pub corrected_duration_ms: u64,
    pub confidence: f64,
    pub bias_corrections_applied: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ForecastRecord {
    pub event_vsa: Vec<u8>,
    pub predicted: ForecastEmotion,
    pub actual: Option<(String, f64, u64)>,
    pub prediction_error: Option<f64>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct AffectiveForecastEngine {
    pub records: VecDeque<ForecastRecord>,
    pub max_records: usize,
    pub impact_bias_factor: f64,
    pub duration_neglect_factor: f64,
    pub focalism_correction: f64,
    pub intensity_decay_rate: f64,
    pub default_duration_ms: u64,
    pub cycle_count: u64,
    pub adaptation_rate: f64,
    pub prototype_library: HashMap<String, Vec<u8>>,
}

impl Default for AffectiveForecastEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AffectiveForecastEngine {
    pub fn new() -> Self {
        let mut library = HashMap::new();
        for (i, label) in ["joy", "sadness", "fear", "anger", "surprise", "disgust"]
            .iter()
            .enumerate()
        {
            let seed = (i as u64 + 1) * 0x9E3779B97F4A7C15;
            library.insert(
                label.to_string(),
                QuantizedVSA::seeded_random(seed, VSA_DIM),
            );
        }
        Self {
            records: VecDeque::with_capacity(200),
            max_records: 200,
            impact_bias_factor: 0.5,
            duration_neglect_factor: 0.4,
            focalism_correction: 0.2,
            intensity_decay_rate: 0.1,
            default_duration_ms: 3_600_000,
            cycle_count: 0,
            adaptation_rate: 0.05,
            prototype_library: library,
        }
    }

    pub fn forecast(
        &mut self,
        event_vsa: Vec<u8>,
        dimensions: AppraisalDimensions,
    ) -> ForecastEmotion {
        self.cycle_count += 1;

        let mut scores: Vec<(String, f64)> = self
            .prototype_library
            .iter()
            .map(|(label, proto)| {
                let sim = QuantizedVSA::similarity(&event_vsa, proto);
                (label.clone(), sim)
            })
            .collect();

        for (label, score) in scores.iter_mut() {
            let dim_weight = match label.as_str() {
                "joy" => dimensions.desirability.max(0.1),
                "sadness" => (1.0 - dimensions.desirability).max(0.1),
                "fear" => {
                    (1.0 - dimensions.desirability) * (1.0 - dimensions.controllability * 0.5)
                }
                "anger" => (1.0 - dimensions.desirability) * dimensions.agency.abs(),
                "surprise" => (1.0 - dimensions.certainty).max(0.1),
                "disgust" => (1.0 - dimensions.desirability) * (1.0 - dimensions.legitimacy),
                _ => 0.5,
            };
            *score *= dim_weight.max(0.05);
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let best_label = scores
            .first()
            .map(|(l, _)| l.clone())
            .unwrap_or_else(|| "joy".to_string());
        let best_score = scores.first().map(|(_, s)| *s).unwrap_or(0.5);
        let second_score = scores.get(1).map(|(_, s)| *s).unwrap_or(0.0);

        let intensity = best_score.clamp(0.0, 1.0);
        let confidence = if best_score + second_score > 0.0 {
            (best_score - second_score) / (best_score + second_score).max(1e-10)
        } else {
            0.5
        };
        let confidence = confidence.clamp(0.0, 1.0);

        let mut forecast = ForecastEmotion {
            emotion_label: best_label,
            expected_intensity: intensity,
            expected_duration_ms: self.default_duration_ms,
            corrected_intensity: intensity,
            corrected_duration_ms: self.default_duration_ms,
            confidence,
            bias_corrections_applied: Vec::new(),
        };

        self.correct_impact_bias(&mut forecast);
        self.correct_duration_neglect(&mut forecast);
        self.correct_focalism(&mut forecast);

        let record = ForecastRecord {
            event_vsa,
            predicted: forecast.clone(),
            actual: None,
            prediction_error: None,
            timestamp: self.cycle_count,
        };
        self.records.push_back(record);
        if self.records.len() > self.max_records {
            self.records.pop_front();
        }

        forecast
    }

    fn correct_impact_bias(&self, forecast: &mut ForecastEmotion) {
        let reduction =
            self.impact_bias_factor * (1.0 - forecast.confidence) * forecast.expected_intensity;
        forecast.corrected_intensity = (forecast.expected_intensity - reduction).max(0.0);
        forecast
            .bias_corrections_applied
            .push("impact_bias".to_string());
    }

    fn correct_duration_neglect(&self, forecast: &mut ForecastEmotion) {
        let reduction_ratio = self.duration_neglect_factor * forecast.corrected_intensity;
        let corrected_duration =
            (forecast.expected_duration_ms as f64 * (1.0 - reduction_ratio)).max(1.0);
        forecast.corrected_duration_ms = corrected_duration as u64;
        forecast
            .bias_corrections_applied
            .push("duration_neglect".to_string());
    }

    fn correct_focalism(&self, forecast: &mut ForecastEmotion) {
        let reduction = forecast.corrected_intensity * self.focalism_correction;
        forecast.corrected_intensity = (forecast.corrected_intensity - reduction).max(0.0);
        forecast
            .bias_corrections_applied
            .push("focalism".to_string());
    }

    pub fn correct_impact_bias_public(&self, forecast: &mut ForecastEmotion) {
        self.correct_impact_bias(forecast);
    }

    pub fn correct_duration_neglect_public(&self, forecast: &mut ForecastEmotion) {
        self.correct_duration_neglect(forecast);
    }

    pub fn record_outcome(
        &mut self,
        event_vsa: &[u8],
        actual_label: &str,
        actual_intensity: f64,
        actual_duration: u64,
    ) {
        let mut best_idx = None;
        let mut best_sim = 0.0;
        for (i, record) in self.records.iter().enumerate() {
            if record.actual.is_some() {
                continue;
            }
            let sim = QuantizedVSA::similarity(&record.event_vsa, event_vsa);
            if sim > best_sim {
                best_sim = sim;
                best_idx = Some(i);
            }
        }

        if let Some(idx) = best_idx {
            let record = &mut self.records[idx];
            let actual = (actual_label.to_string(), actual_intensity, actual_duration);
            let intensity_error = (record.predicted.corrected_intensity - actual_intensity).abs();
            let duration_error = if actual_duration > 0 {
                (record.predicted.corrected_duration_ms as f64 - actual_duration as f64).abs()
                    / actual_duration as f64
            } else {
                0.0
            };
            let prediction_error = intensity_error * 0.7 + duration_error * 0.3;
            record.actual = Some(actual);
            record.prediction_error = Some(prediction_error);

            let avg_error = self.prediction_error_trend();
            if avg_error > 0.3 {
                let adjustment = self.adaptation_rate * (avg_error - 0.3);
                self.impact_bias_factor = (self.impact_bias_factor + adjustment).clamp(0.0, 0.95);
                self.duration_neglect_factor =
                    (self.duration_neglect_factor + adjustment * 0.5).clamp(0.0, 0.9);
            } else if avg_error < 0.1 {
                let reduction = self.adaptation_rate * 0.5;
                self.impact_bias_factor = (self.impact_bias_factor - reduction).max(0.0);
                self.duration_neglect_factor =
                    (self.duration_neglect_factor - reduction * 0.5).max(0.0);
            }
        }
    }

    pub fn prediction_error_trend(&self) -> f64 {
        let recent: Vec<f64> = self
            .records
            .iter()
            .rev()
            .take(20)
            .filter_map(|r| r.prediction_error)
            .collect();
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().sum::<f64>() / recent.len() as f64
    }

    pub fn set_bias_factor(&mut self, factor: &str, value: f64) {
        match factor {
            "impact_bias" => self.impact_bias_factor = value.clamp(0.0, 1.0),
            "duration_neglect" => self.duration_neglect_factor = value.clamp(0.0, 1.0),
            "focalism" => self.focalism_correction = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.records.clear();
        self.cycle_count = 0;
        self.impact_bias_factor = 0.5;
        self.duration_neglect_factor = 0.4;
        self.focalism_correction = 0.2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::AppraisalDimensions;

    fn make_event_vsa() -> Vec<u8> {
        QuantizedVSA::random_vector()
    }

    #[test]
    fn test_new_engine_defaults() {
        let engine = AffectiveForecastEngine::new();
        assert_eq!(engine.max_records, 200);
        assert_eq!(engine.impact_bias_factor, 0.5);
        assert_eq!(engine.duration_neglect_factor, 0.4);
        assert_eq!(engine.focalism_correction, 0.2);
        assert_eq!(engine.default_duration_ms, 3_600_000);
        assert_eq!(engine.prototype_library.len(), 6);
        assert!(engine.prototype_library.contains_key("joy"));
        assert!(engine.prototype_library.contains_key("fear"));
    }

    #[test]
    fn test_forecast_returns_prediction() {
        let mut engine = AffectiveForecastEngine::new();
        let event = make_event_vsa();
        let dims = AppraisalDimensions {
            desirability: 0.9,
            likelihood: 0.8,
            certainty: 0.7,
            ..AppraisalDimensions::default()
        };
        let result = engine.forecast(event, dims);
        assert!(!result.emotion_label.is_empty());
        assert!(result.expected_intensity >= 0.0);
        assert!(result.confidence >= 0.0);
        assert!(result.bias_corrections_applied.len() >= 2);
    }

    #[test]
    fn test_forecast_intensity_within_bounds() {
        let mut engine = AffectiveForecastEngine::new();
        let dims = AppraisalDimensions::default();
        let result = engine.forecast(make_event_vsa(), dims);
        assert!(result.expected_intensity >= 0.0);
        assert!(result.expected_intensity <= 1.0);
        assert!(result.corrected_intensity >= 0.0);
        assert!(result.corrected_intensity <= 1.0);
    }

    #[test]
    fn test_forecast_duration_default() {
        let mut engine = AffectiveForecastEngine::new();
        let dims = AppraisalDimensions::default();
        let result = engine.forecast(make_event_vsa(), dims);
        assert_eq!(result.expected_duration_ms, 3_600_000);
        assert!(result.corrected_duration_ms > 0);
        assert!(result.corrected_duration_ms <= 3_600_000);
    }

    #[test]
    fn test_impact_bias_correction_reduces_intensity() {
        let engine = AffectiveForecastEngine::new();
        let mut fe = ForecastEmotion {
            emotion_label: "joy".to_string(),
            expected_intensity: 0.9,
            expected_duration_ms: 3_600_000,
            corrected_intensity: 0.9,
            corrected_duration_ms: 3_600_000,
            confidence: 0.3,
            bias_corrections_applied: Vec::new(),
        };
        engine.correct_impact_bias_public(&mut fe);
        assert!(
            fe.corrected_intensity < 0.9,
            "impact bias should reduce intensity"
        );
    }

    #[test]
    fn test_duration_neglect_correction_reduces_duration() {
        let engine = AffectiveForecastEngine::new();
        let mut fe = ForecastEmotion {
            emotion_label: "joy".to_string(),
            expected_intensity: 0.8,
            expected_duration_ms: 3_600_000,
            corrected_intensity: 0.8,
            corrected_duration_ms: 3_600_000,
            confidence: 0.5,
            bias_corrections_applied: Vec::new(),
        };
        engine.correct_duration_neglect_public(&mut fe);
        assert!(
            fe.corrected_duration_ms < 3_600_000,
            "duration neglect should reduce duration"
        );
    }

    #[test]
    fn test_correct_impact_bias_high_confidence_less_correction() {
        let engine = AffectiveForecastEngine::new();
        let mut fe_low = ForecastEmotion {
            emotion_label: "joy".to_string(),
            expected_intensity: 0.8,
            expected_duration_ms: 3_600_000,
            corrected_intensity: 0.8,
            corrected_duration_ms: 3_600_000,
            confidence: 0.2,
            bias_corrections_applied: Vec::new(),
        };
        let mut fe_high = fe_low.clone();
        fe_high.confidence = 0.9;
        fe_high.corrected_intensity = 0.8;

        engine.correct_impact_bias_public(&mut fe_low);
        engine.correct_impact_bias_public(&mut fe_high);

        assert!(
            fe_high.corrected_intensity > fe_low.corrected_intensity,
            "high confidence should result in less correction"
        );
    }

    #[test]
    fn test_record_outcome_matches_forecast() {
        let mut engine = AffectiveForecastEngine::new();
        let event = make_event_vsa();
        let dims = AppraisalDimensions::default();
        let _ = engine.forecast(event.clone(), dims);
        assert_eq!(engine.records.len(), 1);
        assert!(engine.records[0].actual.is_none());

        engine.record_outcome(&event, "joy", 0.7, 1_800_000);
        assert!(engine.records[0].actual.is_some());
        let (ref label, intensity, duration) = engine.records[0].actual.as_ref().unwrap();
        assert_eq!(label, "joy");
        assert_eq!(*intensity, 0.7);
        assert_eq!(*duration, 1_800_000);
    }

    #[test]
    fn test_record_outcome_computes_prediction_error() {
        let mut engine = AffectiveForecastEngine::new();
        let event = make_event_vsa();
        let dims = AppraisalDimensions::default();
        let _ = engine.forecast(event.clone(), dims);
        engine.record_outcome(&event, "joy", 0.7, 1_800_000);
        assert!(engine.records[0].prediction_error.is_some());
        let error = engine.records[0].prediction_error.unwrap();
        assert!(error >= 0.0);
    }

    #[test]
    fn test_prediction_error_trend() {
        let mut engine = AffectiveForecastEngine::new();
        let event = make_event_vsa();

        for _ in 0..25 {
            let dims = AppraisalDimensions::default();
            let e = event.clone();
            let _ = engine.forecast(e, dims);
            engine.record_outcome(
                &event,
                "joy",
                engine.records.back().unwrap().predicted.corrected_intensity,
                engine
                    .records
                    .back()
                    .unwrap()
                    .predicted
                    .corrected_duration_ms,
            );
        }

        let trend = engine.prediction_error_trend();
        assert!(
            (trend - 0.0).abs() < 0.15,
            "near-perfect matches should yield near-zero error, got {}",
            trend
        );
    }

    #[test]
    fn test_set_bias_factor_impact() {
        let mut engine = AffectiveForecastEngine::new();
        engine.set_bias_factor("impact_bias", 0.75);
        assert_eq!(engine.impact_bias_factor, 0.75);
    }

    #[test]
    fn test_set_bias_factor_duration() {
        let mut engine = AffectiveForecastEngine::new();
        engine.set_bias_factor("duration_neglect", 0.6);
        assert_eq!(engine.duration_neglect_factor, 0.6);
    }

    #[test]
    fn test_forecast_uses_emotion_prototypes() {
        let mut engine = AffectiveForecastEngine::new();

        let high_desire = AppraisalDimensions {
            desirability: 0.95,
            certainty: 0.9,
            controllability: 0.9,
            ..AppraisalDimensions::default()
        };
        let low_desire = AppraisalDimensions {
            desirability: 0.05,
            certainty: 0.9,
            controllability: 0.1,
            ..AppraisalDimensions::default()
        };

        let event1 = make_event_vsa();
        let result1 = engine.forecast(event1, high_desire);

        let event2 = make_event_vsa();
        let result2 = engine.forecast(event2, low_desire);

        let emotions_differ = result1.emotion_label != result2.emotion_label
            || (result1.corrected_intensity - result2.corrected_intensity).abs() > 0.1;
        assert!(
            emotions_differ,
            "different appraisal dimensions should produce different predictions"
        );
    }

    #[test]
    fn test_reset() {
        let mut engine = AffectiveForecastEngine::new();
        let event = make_event_vsa();
        let dims = AppraisalDimensions::default();
        let _ = engine.forecast(event, dims);
        engine.set_bias_factor("impact_bias", 0.9);
        assert_eq!(engine.records.len(), 1);
        assert_eq!(engine.cycle_count, 1);
        assert_eq!(engine.impact_bias_factor, 0.9);

        engine.reset();
        assert_eq!(engine.records.len(), 0);
        assert_eq!(engine.cycle_count, 0);
        assert_eq!(engine.impact_bias_factor, 0.5);
        assert_eq!(engine.duration_neglect_factor, 0.4);
        assert_eq!(engine.focalism_correction, 0.2);
    }
}
