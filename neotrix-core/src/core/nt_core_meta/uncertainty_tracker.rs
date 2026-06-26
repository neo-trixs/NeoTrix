// REVIVED Evo 3 — dead_code removed
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncertaintyType {
    Epistemic,
    Aleatoric,
    Ambiguity,
    ModelLimitation,
    Conflict,
    TimePressure,
}

#[derive(Debug, Clone)]
pub struct PredictionOutcome {
    pub predicted_confidence: f64,
    pub was_correct: bool,
    pub uncertainty_types: Vec<UncertaintyType>,
    pub timestamp: String,
}

impl PredictionOutcome {
    pub fn new(predicted_confidence: f64, was_correct: bool) -> Self {
        Self {
            predicted_confidence,
            was_correct,
            uncertainty_types: Vec::new(),
            timestamp: timestamp_now(),
        }
    }

    pub fn with_uncertainties(
        predicted_confidence: f64,
        was_correct: bool,
        uncertainty_types: Vec<UncertaintyType>,
    ) -> Self {
        Self {
            predicted_confidence,
            was_correct,
            uncertainty_types,
            timestamp: timestamp_now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalibratedConfidence {
    pub raw_confidence: f64,
    pub calibrated_confidence: f64,
    pub uncertainty_types: Vec<UncertaintyType>,
    pub calibration_quality: f64,
    pub evidence_count: u32,
    pub contradictory_count: u32,
    pub calibration_curve: Vec<(f64, f64)>,
}

impl CalibratedConfidence {
    pub fn new(raw: f64) -> Self {
        Self {
            raw_confidence: raw.clamp(0.0, 1.0),
            calibrated_confidence: raw.clamp(0.0, 1.0),
            uncertainty_types: Vec::new(),
            calibration_quality: 1.0,
            evidence_count: 0,
            contradictory_count: 0,
            calibration_curve: Vec::new(),
        }
    }

    pub fn overconfidence(&self) -> f64 {
        (self.raw_confidence - self.calibrated_confidence).max(0.0)
    }

    pub fn underconfidence(&self) -> f64 {
        (self.calibrated_confidence - self.raw_confidence).max(0.0)
    }

    pub fn is_reliable(&self, threshold: f64) -> bool {
        self.calibrated_confidence >= threshold && self.calibration_quality > 0.5
    }
}

pub struct ConfidenceCalibrator {
    pub history: Vec<PredictionOutcome>,
    pub bin_count: usize,
    pub min_samples_per_bin: u32,
}

impl ConfidenceCalibrator {
    pub fn new(bin_count: usize) -> Self {
        Self {
            history: Vec::new(),
            bin_count: bin_count.max(2),
            min_samples_per_bin: 5,
        }
    }

    pub fn calibrate(
        &mut self,
        raw_confidence: f64,
        evidence_count: u32,
        contradictory_count: u32,
    ) -> CalibratedConfidence {
        let raw = raw_confidence.clamp(0.0, 1.0);
        let curve = self.calibration_curve();
        let quality = self.calibration_quality();

        let calibrated = if self.history.is_empty() {
            let adj = if raw > 0.7 {
                raw * 0.9
            } else if raw < 0.3 {
                raw + 0.05
            } else {
                raw
            };
            adj.clamp(0.0, 1.0)
        } else {
            let bin_idx = ((raw * self.bin_count as f64) as usize).min(self.bin_count - 1);
            let bin_width = 1.0 / self.bin_count as f64;
            let lower = bin_idx as f64 * bin_width;
            let upper = lower + bin_width;
            let bin_center = (lower + upper) / 2.0;

            let (_, actual_acc, count) = curve
                .iter()
                .find(|(m, _, _)| (*m - bin_center).abs() < bin_width / 2.0 + 0.01)
                .copied()
                .unwrap_or((bin_center, raw, 0));

            if count < self.min_samples_per_bin {
                raw * 0.9
            } else {
                actual_acc.clamp(0.0, 1.0)
            }
        };

        let evidence_factor = (evidence_count as f64).atan() / (std::f64::consts::FRAC_PI_2);
        let contradiction_penalty = if contradictory_count > 0 {
            (contradictory_count as f64).atan() / (std::f64::consts::FRAC_PI_2) * 0.3
        } else {
            0.0
        };

        let combined = calibrated * (0.5 + 0.5 * evidence_factor) - contradiction_penalty;
        let calibrated = combined.max(0.05).min(0.99);

        let curve_pairs: Vec<(f64, f64)> = curve.iter().map(|(m, a, _)| (*m, *a)).collect();

        let mut uncertainty_types = Vec::new();
        if evidence_count < 3 {
            uncertainty_types.push(UncertaintyType::Epistemic);
        }
        if contradictory_count > 0 {
            uncertainty_types.push(UncertaintyType::Conflict);
        }

        CalibratedConfidence {
            raw_confidence: raw,
            calibrated_confidence: calibrated,
            uncertainty_types,
            calibration_quality: quality,
            evidence_count,
            contradictory_count,
            calibration_curve: curve_pairs,
        }
    }

    pub fn record_outcome(&mut self, predicted_confidence: f64, was_correct: bool) {
        self.history
            .push(PredictionOutcome::new(predicted_confidence, was_correct));
    }

    pub fn record_outcome_with_types(
        &mut self,
        predicted_confidence: f64,
        was_correct: bool,
        uncertainty_types: Vec<UncertaintyType>,
    ) {
        self.history.push(PredictionOutcome::with_uncertainties(
            predicted_confidence,
            was_correct,
            uncertainty_types,
        ));
    }

    pub fn calibration_curve(&self) -> Vec<(f64, f64, u32)> {
        if self.history.is_empty() {
            return Vec::new();
        }
        let bin_width = 1.0 / self.bin_count as f64;
        let mut bins: Vec<Vec<f64>> = (0..self.bin_count).map(|_| Vec::new()).collect();

        for outcome in &self.history {
            let idx = ((outcome.predicted_confidence * self.bin_count as f64) as usize)
                .min(self.bin_count - 1);
            bins[idx].push(if outcome.was_correct { 1.0 } else { 0.0 });
        }

        bins.iter()
            .enumerate()
            .filter_map(|(i, accs)| {
                if accs.is_empty() {
                    return None;
                }
                let mean_accuracy = accs.iter().sum::<f64>() / accs.len() as f64;
                let mid = (i as f64 + 0.5) * bin_width;
                Some((mid, mean_accuracy, accs.len() as u32))
            })
            .collect()
    }

    pub fn ece(&self) -> f64 {
        let curve = self.calibration_curve();
        if curve.is_empty() {
            return 0.0;
        }
        let total_samples: u32 = curve.iter().map(|(_, _, c)| c).sum();
        if total_samples == 0 {
            return 0.0;
        }
        let weighted_sum: f64 = curve
            .iter()
            .map(|(mid, acc, count)| {
                let diff = (mid - acc).abs();
                diff * *count as f64
            })
            .sum();
        weighted_sum / total_samples as f64
    }

    pub fn mce(&self) -> f64 {
        let curve = self.calibration_curve();
        if curve.is_empty() {
            return 0.0;
        }
        curve
            .iter()
            .map(|(mid, acc, _)| (mid - acc).abs())
            .fold(0.0_f64, f64::max)
    }

    pub fn calibration_quality(&self) -> f64 {
        if self.history.len() < 5 {
            return 1.0;
        }
        let ece_val = self.ece();
        (1.0 - ece_val).max(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct EvidencePiece {
    pub source: String,
    pub content_summary: String,
    pub reliability: f64,
    pub supports_hypothesis: bool,
    pub strength: f64,
}

impl EvidencePiece {
    pub fn new(
        source: &str,
        summary: &str,
        reliability: f64,
        supports: bool,
        strength: f64,
    ) -> Self {
        Self {
            source: source.to_string(),
            content_summary: summary.to_string(),
            reliability: reliability.clamp(0.0, 1.0),
            supports_hypothesis: supports,
            strength: strength.clamp(0.0, 1.0),
        }
    }
}

pub struct UncertaintyDetector;

impl UncertaintyDetector {
    pub fn detect(
        evidence: &[EvidencePiece],
        reasoning_steps: u32,
        time_taken_ms: u64,
    ) -> Vec<UncertaintyType> {
        let mut types = Vec::new();

        if Self::has_conflict(evidence) {
            types.push(UncertaintyType::Conflict);
        }

        if Self::evidence_sufficiency(evidence.len() as u32) < 0.3 {
            types.push(UncertaintyType::Epistemic);
        }

        if Self::time_pressure_assessment(time_taken_ms, 1000).is_some() {
            types.push(UncertaintyType::TimePressure);
        }

        let avg_reliability =
            evidence.iter().map(|e| e.reliability).sum::<f64>() / (evidence.len() as f64).max(1.0);
        if avg_reliability < 0.4 && !evidence.is_empty() {
            types.push(UncertaintyType::Aleatoric);
        }

        if reasoning_steps > 20 {
            types.push(UncertaintyType::ModelLimitation);
        }

        types
    }

    pub fn has_conflict(evidence: &[EvidencePiece]) -> bool {
        let mut supporting = 0;
        let mut opposing = 0;
        for e in evidence {
            if e.supports_hypothesis {
                supporting += 1;
            } else {
                opposing += 1;
            }
        }
        supporting > 0 && opposing > 0
    }

    pub fn time_pressure_assessment(
        time_taken_ms: u64,
        optimal_ms: u64,
    ) -> Option<UncertaintyType> {
        if optimal_ms > 0 && time_taken_ms < optimal_ms / 2 {
            Some(UncertaintyType::TimePressure)
        } else {
            None
        }
    }

    pub fn evidence_sufficiency(evidence_count: u32) -> f64 {
        let count = evidence_count as f64;
        (count / 10.0).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub decision_id: u32,
    pub description: String,
    pub raw_confidence: f64,
    pub calibrated: CalibratedConfidence,
    pub final_outcome: Option<bool>,
    pub context_summary: String,
    pub timestamp: String,
}

pub struct DecisionTracker {
    pub decisions: Vec<DecisionRecord>,
    pub max_history: usize,
    next_id: u32,
}

impl DecisionTracker {
    pub fn new(max: usize) -> Self {
        Self {
            decisions: Vec::with_capacity(max),
            max_history: max.max(10),
            next_id: 1,
        }
    }

    pub fn record_decision(
        &mut self,
        desc: &str,
        raw_confidence: f64,
        calibrated: CalibratedConfidence,
        context: &str,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        if self.decisions.len() >= self.max_history {
            self.decisions.remove(0);
        }

        self.decisions.push(DecisionRecord {
            decision_id: id,
            description: desc.to_string(),
            raw_confidence: raw_confidence.clamp(0.0, 1.0),
            calibrated,
            final_outcome: None,
            context_summary: context.to_string(),
            timestamp: timestamp_now(),
        });
        id
    }

    pub fn record_outcome(&mut self, decision_id: u32, was_correct: bool) {
        if let Some(d) = self
            .decisions
            .iter_mut()
            .rev()
            .find(|d| d.decision_id == decision_id)
        {
            d.final_outcome = Some(was_correct);
        }
    }

    pub fn uncertainty_report(&self) -> UncertaintyReport {
        let total = self.decisions.len() as u32;
        let evaluated: Vec<&DecisionRecord> = self
            .decisions
            .iter()
            .filter(|d| d.final_outcome.is_some())
            .collect();
        let evaluated_count = evaluated.len() as u32;

        if total == 0 {
            return UncertaintyReport {
                total_decisions: 0,
                evaluated_decisions: 0,
                mean_raw_confidence: 0.0,
                mean_calibrated_confidence: 0.0,
                mean_accuracy: 0.0,
                ece: 0.0,
                mce: 0.0,
                overconfident_count: 0,
                underconfident_count: 0,
            };
        }

        let mean_raw: f64 =
            self.decisions.iter().map(|d| d.raw_confidence).sum::<f64>() / total as f64;
        let mean_cal: f64 = self
            .decisions
            .iter()
            .map(|d| d.calibrated.calibrated_confidence)
            .sum::<f64>()
            / total as f64;

        let mut ece_sum = 0.0;
        let mut mce_val: f64 = 0.0;
        let mut over = 0;
        let mut under = 0;
        let mut correct = 0;

        for d in &evaluated {
            let Some(outcome) = d.final_outcome else { continue; };
            if outcome {
                correct += 1;
            }
            let diff = (d.calibrated.calibrated_confidence - if outcome { 1.0 } else { 0.0 }).abs();
            ece_sum += diff;
            mce_val = mce_val.max(diff);

            if d.calibrated.calibrated_confidence > 0.7 && !outcome {
                over += 1;
            }
            if d.calibrated.calibrated_confidence < 0.3 && outcome {
                under += 1;
            }
        }

        UncertaintyReport {
            total_decisions: total,
            evaluated_decisions: evaluated_count,
            mean_raw_confidence: mean_raw,
            mean_calibrated_confidence: mean_cal,
            mean_accuracy: if evaluated_count > 0 {
                correct as f64 / evaluated_count as f64
            } else {
                0.0
            },
            ece: if evaluated_count > 0 {
                ece_sum / evaluated_count as f64
            } else {
                0.0
            },
            mce: mce_val,
            overconfident_count: over,
            underconfident_count: under,
        }
    }

    pub fn mean_calibration_error(&self) -> f64 {
        self.uncertainty_report().ece
    }

    pub fn decisions_above_threshold(&self, threshold: f64) -> Vec<&DecisionRecord> {
        self.decisions
            .iter()
            .filter(|d| d.calibrated.calibrated_confidence >= threshold)
            .collect()
    }

    pub fn decisions_below_threshold(&self, threshold: f64) -> Vec<&DecisionRecord> {
        self.decisions
            .iter()
            .filter(|d| d.calibrated.calibrated_confidence < threshold)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct UncertaintyReport {
    pub total_decisions: u32,
    pub evaluated_decisions: u32,
    pub mean_raw_confidence: f64,
    pub mean_calibrated_confidence: f64,
    pub mean_accuracy: f64,
    pub ece: f64,
    pub mce: f64,
    pub overconfident_count: u32,
    pub underconfident_count: u32,
}

fn timestamp_now() -> String {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("ts:{}", start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrated_confidence_construction() {
        let c = CalibratedConfidence::new(0.8);
        assert!((c.raw_confidence - 0.8).abs() < 1e-6);
        assert!((c.calibrated_confidence - 0.8).abs() < 1e-6);
        assert!(c.calibration_quality - 1.0 < 1e-6);
        assert_eq!(c.evidence_count, 0);
        assert_eq!(c.contradictory_count, 0);
        assert!(c.uncertainty_types.is_empty());
    }

    #[test]
    fn test_calibrator_new() {
        let c = ConfidenceCalibrator::new(10);
        assert_eq!(c.bin_count, 10);
        assert!(c.history.is_empty());
        assert_eq!(c.min_samples_per_bin, 5);
    }

    #[test]
    fn test_calibrate_high_confidence() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        let result = calibrator.calibrate(0.95, 5, 0);
        assert!(result.calibrated_confidence < 0.95);
        assert!(result.calibrated_confidence > 0.5);
        assert!((result.raw_confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_calibrate_low_confidence() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        let result = calibrator.calibrate(0.1, 5, 0);
        assert!(result.calibrated_confidence > 0.1);
        assert!(result.calibrated_confidence <= 0.99);
    }

    #[test]
    fn test_record_outcome_updates_curve() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        calibrator.record_outcome(0.8, true);
        calibrator.record_outcome(0.8, true);
        calibrator.record_outcome(0.8, false);
        let curve = calibrator.calibration_curve();
        assert!(!curve.is_empty());
        let (_, acc, count) = curve[0];
        assert!(count >= 3);
        assert!((acc - 2.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn test_calibration_curve_construction() {
        let mut calibrator = ConfidenceCalibrator::new(5);
        for i in 0..10 {
            let conf = 0.1 + i as f64 * 0.08;
            let correct = i % 2 == 0;
            calibrator.record_outcome(conf, correct);
        }
        let curve = calibrator.calibration_curve();
        assert!(!curve.is_empty());
        assert!(curve.len() <= 5);
        for &(mid, acc, count) in &curve {
            assert!(mid >= 0.0);
            assert!(acc >= 0.0 && acc <= 1.0);
            assert!(count > 0);
        }
    }

    #[test]
    fn test_ece_perfect_calibration() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        for _ in 0..20 {
            calibrator.record_outcome(0.9, true);
            calibrator.record_outcome(0.7, true);
            calibrator.record_outcome(0.5, true);
        }
        let ece_val = calibrator.ece();
        assert!(ece_val < 0.3);
    }

    #[test]
    fn test_ece_poor_calibration() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        for _ in 0..20 {
            calibrator.record_outcome(0.9, false);
        }
        let ece_val = calibrator.ece();
        assert!(ece_val > 0.3);
    }

    #[test]
    fn test_mce_computation() {
        let mut calibrator = ConfidenceCalibrator::new(5);
        calibrator.record_outcome(0.9, false);
        calibrator.record_outcome(0.9, false);
        calibrator.record_outcome(0.9, false);
        let mce_val = calibrator.mce();
        assert!(mce_val > 0.0);
    }

    #[test]
    fn test_uncertainty_detector_conflict() {
        let evidence = vec![
            EvidencePiece::new("src1", "supports A", 0.9, true, 0.8),
            EvidencePiece::new("src2", "opposes A", 0.8, false, 0.7),
        ];
        assert!(UncertaintyDetector::has_conflict(&evidence));
        let types = UncertaintyDetector::detect(&evidence, 5, 500);
        assert!(types.contains(&UncertaintyType::Conflict));
    }

    #[test]
    fn test_uncertainty_detector_time_pressure() {
        let assessment = UncertaintyDetector::time_pressure_assessment(200, 1000);
        assert!(assessment.is_some());
        assert_eq!(assessment.unwrap(), UncertaintyType::TimePressure);

        let no_pressure = UncertaintyDetector::time_pressure_assessment(800, 1000);
        assert!(no_pressure.is_none());
    }

    #[test]
    fn test_uncertainty_detector_sufficient_evidence() {
        let suf = UncertaintyDetector::evidence_sufficiency(10);
        assert!((suf - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_uncertainty_detector_insufficient_evidence() {
        let suf = UncertaintyDetector::evidence_sufficiency(1);
        assert!((suf - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_decision_tracker_record_and_outcome() {
        let mut tracker = DecisionTracker::new(100);
        let cal = CalibratedConfidence::new(0.75);
        let id = tracker.record_decision("test decision", 0.75, cal, "test context");
        assert_eq!(id, 1);
        assert_eq!(tracker.decisions.len(), 1);
        assert_eq!(tracker.decisions[0].final_outcome, None);

        tracker.record_outcome(id, true);
        assert_eq!(tracker.decisions[0].final_outcome, Some(true));
    }

    #[test]
    fn test_decision_tracker_uncertainty_report() {
        let mut tracker = DecisionTracker::new(100);
        for i in 0..5 {
            let cal = CalibratedConfidence::new(0.5 + i as f64 * 0.1);
            let id =
                tracker.record_decision(&format!("dec_{}", i), 0.5 + i as f64 * 0.1, cal, "ctx");
            tracker.record_outcome(id, i < 3);
        }
        let report = tracker.uncertainty_report();
        assert_eq!(report.total_decisions, 5);
        assert_eq!(report.evaluated_decisions, 5);
        assert!(report.mean_raw_confidence > 0.0);
        assert!(report.mean_calibrated_confidence > 0.0);
        assert!(report.ece >= 0.0);
        assert!(report.mce >= 0.0);
    }

    #[test]
    fn test_decision_tracker_overconfident_detection() {
        let mut tracker = DecisionTracker::new(100);
        for i in 0..5 {
            let cal = CalibratedConfidence::new(0.8);
            let id = tracker.record_decision(&format!("dec_{}", i), 0.8, cal, "ctx");
            tracker.record_outcome(id, i == 0);
        }
        let report = tracker.uncertainty_report();
        assert!(report.overconfident_count >= 3);
    }

    #[test]
    fn test_decision_tracker_underconfident_detection() {
        let mut tracker = DecisionTracker::new(100);
        for i in 0..5 {
            let cal = CalibratedConfidence::new(0.2);
            let id = tracker.record_decision(&format!("dec_{}", i), 0.2, cal, "ctx");
            tracker.record_outcome(id, true);
        }
        let report = tracker.uncertainty_report();
        assert!(report.underconfident_count >= 3);
    }

    #[test]
    fn test_uncertainty_report_accuracy() {
        let mut tracker = DecisionTracker::new(100);
        for i in 0..10 {
            let cal = CalibratedConfidence::new(0.5);
            let id = tracker.record_decision(&format!("dec_{}", i), 0.5, cal, "ctx");
            tracker.record_outcome(id, i < 7);
        }
        let report = tracker.uncertainty_report();
        assert!((report.mean_accuracy - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_calibrate_with_contradiction() {
        let mut calibrator = ConfidenceCalibrator::new(10);
        let result = calibrator.calibrate(0.8, 10, 5);
        assert!(result.contradictory_count == 5);
    }

    #[test]
    fn test_decision_tracker_thresholds() {
        let mut tracker = DecisionTracker::new(100);
        for i in 0..5 {
            let cal = CalibratedConfidence::new(0.3 + i as f64 * 0.15);
            tracker.record_decision(&format!("dec_{}", i), 0.3 + i as f64 * 0.15, cal, "ctx");
        }
        let above = tracker.decisions_above_threshold(0.5);
        let below = tracker.decisions_below_threshold(0.5);
        assert_eq!(above.len() + below.len(), tracker.decisions.len());
    }

    #[test]
    fn test_evidence_piece_construction() {
        let e = EvidencePiece::new("source_a", "content", 0.8, true, 0.9);
        assert_eq!(e.source, "source_a");
        assert!((e.reliability - 0.8).abs() < 1e-6);
        assert!(e.supports_hypothesis);
        assert!((e.strength - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_uncertainty_detector_no_conflict() {
        let evidence = vec![
            EvidencePiece::new("src1", "supports A", 0.9, true, 0.8),
            EvidencePiece::new("src2", "also supports A", 0.8, true, 0.7),
        ];
        assert!(!UncertaintyDetector::has_conflict(&evidence));
    }

    #[test]
    fn test_empty_calibration_curve() {
        let calibrator = ConfidenceCalibrator::new(10);
        let curve = calibrator.calibration_curve();
        assert!(curve.is_empty());
        assert!((calibrator.ece() - 0.0).abs() < 1e-6);
        assert!((calibrator.mce() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_prediction_outcome_construction() {
        let p = PredictionOutcome::new(0.7, true);
        assert!((p.predicted_confidence - 0.7).abs() < 1e-6);
        assert!(p.was_correct);
        assert!(p.uncertainty_types.is_empty());

        let p2 =
            PredictionOutcome::with_uncertainties(0.3, false, vec![UncertaintyType::Epistemic]);
        assert!((p2.predicted_confidence - 0.3).abs() < 1e-6);
        assert!(!p2.was_correct);
        assert_eq!(p2.uncertainty_types.len(), 1);
    }

    #[test]
    fn test_overconfidence_underconfidence() {
        let mut cal = CalibratedConfidence::new(0.9);
        cal.calibrated_confidence = 0.6;
        assert!((cal.overconfidence() - 0.3).abs() < 1e-6);
        assert!((cal.underconfidence() - 0.0).abs() < 1e-6);

        let mut cal2 = CalibratedConfidence::new(0.3);
        cal2.calibrated_confidence = 0.7;
        assert!((cal2.underconfidence() - 0.4).abs() < 1e-6);
        assert!((cal2.overconfidence() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_reliable() {
        let mut cal = CalibratedConfidence::new(0.8);
        cal.calibrated_confidence = 0.85;
        assert!(cal.is_reliable(0.5));
        cal.calibration_quality = 0.3;
        assert!(!cal.is_reliable(0.5));
    }
}
