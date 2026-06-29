use std::collections::VecDeque;

use crate::core::nt_core_consciousness::{
    confidence_calibrator::ConfidenceCalibrator,
    epistemic_honesty::{EpistemicHonesty, HonestyConfig},
    vsa_tag::{OutcomeRecord, PredictionRecord},
};
use crate::core::nt_core_experience::epistemic::{EpistemicConfig, EpistemicSelfModel};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_self::attention_head::AttentionDomain;

pub struct CalibrationEngine {
    pub epistemic: EpistemicSelfModel,
    pub confidence: ConfidenceCalibrator,
    pub honesty: EpistemicHonesty,
    pub predictions: Vec<PredictionRecord>,
    pub outcomes: Vec<OutcomeRecord>,
    pub pre_post_pairs: VecDeque<(PredictionRecord, OutcomeRecord)>,
    pub rsi: RsiMetrics,
}

#[derive(Debug, Clone)]
pub struct CalibrationStats {
    pub meta_d: f64,
    pub m_ratio: f64,
    pub ece: f64,
    pub calibration_error: f64,
    pub pair_count: usize,
}

fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xe8e8_e8e8_e8e8_e8e8u64;
    for b in s.bytes() {
        h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
        h ^= b as u64;
        h = h.rotate_left(11);
    }
    h
}

fn domain_to_attention(domain: &str) -> AttentionDomain {
    match domain.to_lowercase().as_str() {
        "semantic" | "semantics" => AttentionDomain::Semantic,
        "code" | "programming" => AttentionDomain::Code,
        "planning" | "plan" => AttentionDomain::Planning,
        "creativity" | "creative" => AttentionDomain::Creativity,
        "temporal" | "time" => AttentionDomain::Temporal,
        "selfreflection" | "reflection" | "self" => AttentionDomain::SelfReflection,
        "tooluse" | "tool" => AttentionDomain::ToolUse,
        "goal" | "goalalignment" | "alignment" => AttentionDomain::GoalAlignment,
        "risk" | "riskassessment" => AttentionDomain::RiskAssessment,
        "memory" | "mem" => AttentionDomain::Memory,
        "social" | "society" => AttentionDomain::Social,
        "emotional" | "emotion" | "affect" => AttentionDomain::Emotional,
        _ => AttentionDomain::PatternMatch,
    }
}

impl Default for CalibrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// RSI (Recursive Self-Improvement) empirical metrics tracker.
///
/// Tracks the three key RSI metrics identified in empirical literature:
/// - `code_auto_rate`: percentage of code generated autonomously (0.0–1.0)
/// - `engineer_multiplier`: output per unit time (baseline = 1.0)
/// - `task_autonomy_hours`: time between human interventions (hours)
///
/// These metrics are tracked as exponential moving averages for smooth
/// trending. Each record updates the EMA with configurable alpha.
#[derive(Debug, Clone)]
pub struct RsiMetrics {
    pub code_auto_rate: f64,
    pub engineer_multiplier: f64,
    pub task_autonomy_hours: f64,
    pub alpha: f64,
    pub samples: u64,
    pub last_updated: u64,
}

impl RsiMetrics {
    pub fn new() -> Self {
        Self {
            code_auto_rate: 0.0,
            engineer_multiplier: 1.0,
            task_autonomy_hours: 0.0,
            alpha: 0.3,
            samples: 0,
            last_updated: 0,
        }
    }

    /// Record an autonomous code generation event.
    /// `autonomous`: number of operations completed without human intervention
    /// `total`: total operations in this session
    pub fn record_code_auto(&mut self, autonomous: u64, total: u64) {
        let rate = if total > 0 {
            autonomous as f64 / total as f64
        } else {
            0.0
        };
        self.code_auto_rate = self.ema(self.code_auto_rate, rate);
        self.samples += 1;
        self.last_updated = self.now();
    }

    /// Record engineer multiplier (output per unit time).
    /// `output`: measure of work completed (e.g. lines of code, tasks)
    /// `hours`: elapsed time in hours
    pub fn record_engineer_mult(&mut self, output: f64, hours: f64) {
        let mult = if hours > 0.0 { output / hours } else { 1.0 };
        self.engineer_multiplier = self.ema(self.engineer_multiplier, mult);
        self.samples += 1;
        self.last_updated = self.now();
    }

    /// Record task autonomy in hours.
    /// `hours_since_intervention`: wall clock time since last human intervention
    pub fn record_task_autonomy(&mut self, hours_since_intervention: f64) {
        self.task_autonomy_hours = self.ema(self.task_autonomy_hours, hours_since_intervention);
        self.samples += 1;
        self.last_updated = self.now();
    }

    fn ema(&self, prev: f64, current: f64) -> f64 {
        if self.samples == 0 {
            current
        } else {
            prev * (1.0 - self.alpha) + current * self.alpha
        }
    }

    fn now(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn report(&self) -> String {
        format!(
            "RSI: auto_rate={:.1}% mult={:.2}x autonomy={:.1}h samples={}",
            self.code_auto_rate * 100.0,
            self.engineer_multiplier,
            self.task_autonomy_hours,
            self.samples
        )
    }
}

impl Default for RsiMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationEngine {
    pub fn new() -> Self {
        Self {
            epistemic: EpistemicSelfModel::new(EpistemicConfig::default()),
            confidence: ConfidenceCalibrator::new(),
            honesty: EpistemicHonesty::new(HonestyConfig::default()),
            predictions: Vec::with_capacity(100),
            outcomes: Vec::with_capacity(100),
            pre_post_pairs: VecDeque::with_capacity(200),
            rsi: RsiMetrics::new(),
        }
    }

    /// Predict outcome for a domain, returning (predicted_success, averaged_confidence).
    pub fn predict(&mut self, domain: &str, quality: f64) -> (f64, f64) {
        let context_vec = QuantizedVSA::seeded_random(stable_hash(domain), 4096);
        let predicted_success = self.epistemic.predict_success(&context_vec);
        let calibrated = self.confidence.calibrate(predicted_success);
        let honesty_adjusted = self.honesty.honest_confidence(calibrated);
        let avg_confidence = (predicted_success + calibrated + honesty_adjusted) / 3.0;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.predictions.push(PredictionRecord {
            predicted_success,
            predicted_quality: quality,
            confidence: avg_confidence,
            domain: domain.to_string(),
            timestamp: now,
        });

        if self.predictions.len() > 1000 {
            self.predictions.remove(0);
        }

        (predicted_success, avg_confidence)
    }

    /// Record the actual outcome, returning surprise = |predicted - actual|.
    pub fn record_outcome(&mut self, domain: &str, success: bool, quality: f64) -> f64 {
        let domain_key = domain_to_attention(domain);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let pred = self.predictions.iter().rev().find(|p| p.domain == domain);
        let actual_success_f = if success { 1.0 } else { 0.0 };
        let surprise = match pred {
            Some(p) => (p.predicted_success - actual_success_f).abs(),
            None => 1.0,
        };
        let pred_confidence = pred.map(|p| p.confidence).unwrap_or(0.5);

        let outcome = OutcomeRecord {
            actual_success: success,
            actual_quality: quality,
            outcome_detail: format!("domain={}, surprise={:.3}", domain, surprise),
            timestamp: now,
        };

        self.outcomes.push(outcome.clone());
        if self.outcomes.len() > 1000 {
            self.outcomes.remove(0);
        }

        self.epistemic
            .calibrate(&domain_key, pred_confidence, success);
        self.confidence.record_prediction(pred_confidence, success);
        self.honesty.calibrate(pred_confidence, success);

        if let Some(p) = pred {
            if self.pre_post_pairs.len() >= 200 {
                self.pre_post_pairs.pop_front();
            }
            self.pre_post_pairs.push_back((p.clone(), outcome));
        }

        surprise
    }

    pub fn stats(&self) -> CalibrationStats {
        let report = self.honesty.report();
        CalibrationStats {
            meta_d: report.meta_d,
            m_ratio: report.m_ratio,
            ece: self.ece(),
            calibration_error: report.calibration_error,
            pair_count: self.pre_post_pairs.len(),
        }
    }

    /// Expected Calibration Error from the confidence calibrator.
    pub fn ece(&self) -> f64 {
        self.confidence.ece()
    }

    /// Access the RSI metrics tracker.
    pub fn rsi_metrics(&self) -> &RsiMetrics {
        &self.rsi
    }

    pub fn rsi_metrics_mut(&mut self) -> &mut RsiMetrics {
        &mut self.rsi
    }

    pub fn last_n_predictions(&self, n: usize) -> &[PredictionRecord] {
        let start = self.predictions.len().saturating_sub(n);
        &self.predictions[start..]
    }

    pub fn prediction_count(&self) -> usize {
        self.predictions.len()
    }

    pub fn outcome_count(&self) -> usize {
        self.outcomes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_empty() {
        let engine = CalibrationEngine::new();
        assert_eq!(engine.prediction_count(), 0);
        assert_eq!(engine.outcome_count(), 0);
        let s = engine.stats();
        assert!((s.meta_d - 0.0).abs() < f64::EPSILON);
        assert!((s.m_ratio - 0.0).abs() < f64::EPSILON);
        assert!((s.ece - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_predict_returns_sane_values() {
        let mut engine = CalibrationEngine::new();
        let (pred, conf) = engine.predict("Semantic", 0.8);
        assert!(pred >= 0.0 && pred <= 1.0);
        assert!(conf >= 0.0 && conf <= 1.0);
        assert_eq!(engine.prediction_count(), 1);
    }

    #[test]
    fn test_record_outcome_returns_surprise() {
        let mut engine = CalibrationEngine::new();
        engine.predict("Semantic", 0.8);
        let surprise = engine.record_outcome("Semantic", true, 0.9);
        assert!(surprise >= 0.0 && surprise <= 1.0);
        assert_eq!(engine.outcome_count(), 1);
    }

    #[test]
    fn test_record_outcome_creates_pair() {
        let mut engine = CalibrationEngine::new();
        engine.predict("Semantic", 0.8);
        engine.record_outcome("Semantic", true, 0.9);
        assert_eq!(engine.pre_post_pairs.len(), 1);
    }

    #[test]
    fn test_stats_after_full_cycle() {
        let mut engine = CalibrationEngine::new();
        for _ in 0..20 {
            engine.predict("Semantic", 0.8);
            engine.record_outcome("Semantic", true, 0.9);
        }
        let s = engine.stats();
        assert!(s.pair_count >= 20);
    }

    #[test]
    fn test_last_n_predictions() {
        let mut engine = CalibrationEngine::new();
        for _ in 0..10 {
            engine.predict("Code", 0.7);
            engine.record_outcome("Code", true, 0.8);
        }
        let last = engine.last_n_predictions(5);
        assert_eq!(last.len(), 5);
    }

    #[test]
    fn test_different_domains_tracked() {
        let mut engine = CalibrationEngine::new();
        engine.predict("Semantic", 0.8);
        engine.predict("Code", 0.7);
        engine.predict("Planning", 0.6);
        assert_eq!(engine.prediction_count(), 3);
    }

    // ── RsiMetrics tests ───────────────────────────────────────────────────

    #[test]
    fn test_rsi_metrics_new() {
        let rsi = RsiMetrics::new();
        assert!((rsi.code_auto_rate - 0.0).abs() < 1e-9);
        assert!((rsi.engineer_multiplier - 1.0).abs() < 1e-9);
        assert!((rsi.task_autonomy_hours - 0.0).abs() < 1e-9);
        assert_eq!(rsi.samples, 0);
    }

    #[test]
    fn test_rsi_record_code_auto() {
        let mut rsi = RsiMetrics::new();
        rsi.record_code_auto(80, 100);
        assert!(rsi.code_auto_rate > 0.0);
        assert_eq!(rsi.samples, 1);
        assert!(rsi.last_updated > 0);
    }

    #[test]
    fn test_rsi_record_engineer_mult() {
        let mut rsi = RsiMetrics::new();
        rsi.record_engineer_mult(50.0, 2.0);
        assert!(rsi.engineer_multiplier > 1.0);
        assert_eq!(rsi.samples, 1);
    }

    #[test]
    fn test_rsi_record_task_autonomy() {
        let mut rsi = RsiMetrics::new();
        rsi.record_task_autonomy(4.5);
        assert!((rsi.task_autonomy_hours - 4.5).abs() < 1e-9);
        assert_eq!(rsi.samples, 1);
    }

    #[test]
    fn test_rsi_ema_convergence() {
        let mut rsi = RsiMetrics::new();
        for _ in 0..10 {
            rsi.record_code_auto(100, 100);
        }
        // After many identical samples, EMA should converge to 1.0
        assert!((rsi.code_auto_rate - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_rsi_report_format() {
        let mut rsi = RsiMetrics::new();
        rsi.record_code_auto(75, 100);
        rsi.record_engineer_mult(30.0, 1.0);
        let report = rsi.report();
        assert!(report.contains("RSI:"));
        assert!(report.contains("auto_rate="));
        assert!(report.contains("mult="));
    }

    #[test]
    fn test_rsi_accessible_via_calibration_engine() {
        let engine = CalibrationEngine::new();
        let rsi = engine.rsi_metrics();
        assert_eq!(rsi.samples, 0);
    }

    #[test]
    fn test_rsi_mut_accessible_via_calibration_engine() {
        let mut engine = CalibrationEngine::new();
        engine.rsi_metrics_mut().record_code_auto(90, 100);
        assert!(engine.rsi_metrics().code_auto_rate > 0.0);
    }

    #[test]
    fn test_domain_to_attention_maps_all() {
        assert_eq!(domain_to_attention("semantic"), AttentionDomain::Semantic);
        assert_eq!(domain_to_attention("code"), AttentionDomain::Code);
        assert_eq!(domain_to_attention("planning"), AttentionDomain::Planning);
        assert_eq!(domain_to_attention("reasoning"), AttentionDomain::Reasoning);
        assert_eq!(domain_to_attention("memory"), AttentionDomain::Memory);
        assert_eq!(
            domain_to_attention("creativity"),
            AttentionDomain::Creativity
        );
        assert_eq!(domain_to_attention("social"), AttentionDomain::Social);
        assert_eq!(domain_to_attention("emotional"), AttentionDomain::Emotional);
        assert_eq!(
            domain_to_attention("unknown"),
            AttentionDomain::PatternMatch
        );
    }
}
