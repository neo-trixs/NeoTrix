use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct HonestyConfig {
    pub calibration_window: usize,
    pub unknown_unknown_prior: f64,
    pub overconfidence_penalty: f64,
    pub underconfidence_boost: f64,
}

impl Default for HonestyConfig {
    fn default() -> Self {
        Self {
            calibration_window: 100,
            unknown_unknown_prior: 0.05,
            overconfidence_penalty: 1.5,
            underconfidence_boost: 0.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub correct: usize,
    pub accuracy: f64,
}

impl CalibrationBin {
    fn new(lower: f64, upper: f64) -> Self {
        Self {
            lower,
            upper,
            count: 0,
            correct: 0,
            accuracy: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpistemicReport {
    pub calibration_error: f64,
    pub overconfidence_rate: f64,
    pub underconfidence_rate: f64,
    pub unknown_unknowns: f64,
    pub total_predictions: usize,
    pub ece: f64,
    pub meta_d: f64,
    pub m_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct EpistemicHonesty {
    bins: Vec<CalibrationBin>,
    predictions: Vec<(f64, bool)>,
    config: HonestyConfig,
    unknown_unknown_score: f64,
    step: u64,
}

impl EpistemicHonesty {
    pub fn new(config: HonestyConfig) -> Self {
        let bins = (0..10)
            .map(|i| CalibrationBin::new(i as f64 / 10.0, (i + 1) as f64 / 10.0))
            .collect();
        Self {
            bins,
            predictions: Vec::new(),
            config,
            unknown_unknown_score: 0.0,
            step: 0,
        }
    }

    pub fn calibrate(&mut self, confidence: f64, correct: bool) {
        let clamped = confidence.clamp(0.0, 0.999);
        let bin_idx = (clamped * 10.0).min(9.0) as usize;
        let bin = &mut self.bins[bin_idx];
        bin.count += 1;
        if correct {
            bin.correct += 1;
        }
        bin.accuracy = bin.correct as f64 / bin.count as f64;
        self.predictions.push((confidence, correct));
        self.step += 1;
    }

    pub fn honest_confidence(&self, raw_confidence: f64) -> f64 {
        let clamped = raw_confidence.clamp(0.0, 0.999);
        let bin_idx = (clamped * 10.0).min(9.0) as usize;
        let bin = &self.bins[bin_idx];
        let accuracy = if bin.count > 0 {
            bin.accuracy
        } else {
            raw_confidence
        };
        let adjusted = if raw_confidence > accuracy {
            accuracy - (raw_confidence - accuracy) * self.config.overconfidence_penalty
        } else {
            raw_confidence + (accuracy - raw_confidence) * self.config.underconfidence_boost
        };
        adjusted.clamp(0.0, 1.0)
    }

    pub fn unknown_unknown_estimate(&self, context_novelty: f64) -> f64 {
        self.config.unknown_unknown_prior * (1.0 + context_novelty)
            / (1.0 + self.predictions.len() as f64 * 0.01)
    }

    pub fn epistemic_risk(&self, decision_confidence: f64) -> f64 {
        (1.0 - self.honest_confidence(decision_confidence)) * (1.0 + self.unknown_unknown_score)
    }

    pub fn detect_self_deception(&self, claim: &[u8], evidence: &[u8]) -> f64 {
        1.0 - QuantizedVSA::similarity(claim, evidence)
    }

    pub fn report(&self) -> EpistemicReport {
        let n = self.predictions.len();
        if n == 0 {
            return EpistemicReport {
                calibration_error: 0.0,
                overconfidence_rate: 0.0,
                underconfidence_rate: 0.0,
                unknown_unknowns: self.unknown_unknown_score,
                total_predictions: 0,
                ece: 0.0,
                meta_d: 0.0,
                m_ratio: 0.0,
            };
        }
        let window = self.config.calibration_window.min(n);
        let recent = &self.predictions[n - window..];
        let cal_error_sum: f64 = recent
            .iter()
            .map(|(conf, _)| {
                let clamped = conf.clamp(0.0, 0.999);
                let bin_idx = (clamped * 10.0).min(9.0) as usize;
                let bin = &self.bins[bin_idx];
                let bin_acc = if bin.count > 0 { bin.accuracy } else { *conf };
                (conf - bin_acc).abs()
            })
            .sum();
        let calibration_error = cal_error_sum / window as f64;
        let over_count = recent
            .iter()
            .filter(|(conf, _)| {
                let clamped = conf.clamp(0.0, 0.999);
                let bin_idx = (clamped * 10.0).min(9.0) as usize;
                let bin = &self.bins[bin_idx];
                let bin_acc = if bin.count > 0 { bin.accuracy } else { *conf };
                *conf > bin_acc + 0.1
            })
            .count();
        let under_count = recent
            .iter()
            .filter(|(conf, _)| {
                let clamped = conf.clamp(0.0, 0.999);
                let bin_idx = (clamped * 10.0).min(9.0) as usize;
                let bin = &self.bins[bin_idx];
                let bin_acc = if bin.count > 0 { bin.accuracy } else { *conf };
                *conf < bin_acc - 0.1
            })
            .count();
        let overconfidence_rate = over_count as f64 / window as f64;
        let underconfidence_rate = under_count as f64 / window as f64;
        let total = n as f64;
        let ece: f64 = self
            .bins
            .iter()
            .map(|b| {
                let midpoint = (b.lower + b.upper) / 2.0;
                (b.accuracy - midpoint).abs() * (b.count as f64 / total)
            })
            .sum();
        EpistemicReport {
            calibration_error,
            overconfidence_rate,
            underconfidence_rate,
            unknown_unknowns: self.unknown_unknown_score,
            total_predictions: n,
            ece,
            meta_d: self.compute_meta_d(),
            m_ratio: self.compute_m_ratio(),
        }
    }

    pub fn reset_calibration(&mut self) {
        for bin in &mut self.bins {
            bin.count = 0;
            bin.correct = 0;
            bin.accuracy = 0.0;
        }
        self.predictions.clear();
        self.step = 0;
    }

    /// Compute type-2 d' (meta-d'): metacognitive sensitivity.
    /// Uses the confidence-accuracy relationship: how well confidence
    /// predicts correctness. Higher = better metacognition.
    pub fn compute_meta_d(&self) -> f64 {
        let n = self.predictions.len();
        if n < 10 {
            return 0.0;
        }
        let window = self.config.calibration_window.min(n);
        let recent = &self.predictions[n - window..];

        let mut type2_hits = 0usize;
        let mut total_correct = 0usize;
        let mut type2_fa = 0usize;
        let mut total_incorrect = 0usize;
        let conf_threshold = 0.7;

        for (conf, correct) in recent {
            if *correct {
                total_correct += 1;
                if *conf > conf_threshold {
                    type2_hits += 1;
                }
            } else {
                total_incorrect += 1;
                if *conf > conf_threshold {
                    type2_fa += 1;
                }
            }
        }

        let hr = type2_hits as f64 / total_correct.max(1) as f64;
        let far = type2_fa as f64 / total_incorrect.max(1) as f64;

        let hr_clamped = hr.clamp(0.01, 0.99);
        let far_clamped = far.clamp(0.01, 0.99);

        let z_hr = gaussian_quantile(hr_clamped);
        let z_far = gaussian_quantile(far_clamped);

        let meta_d = z_hr - z_far;
        if meta_d.is_finite() {
            meta_d
        } else {
            0.0
        }
    }

    /// Compute M-ratio = meta-d' / d'.
    /// M-ratio < 1 means metacognitive inefficiency (over/underconfidence).
    /// M-ratio ≈ 1 means optimal metacognition.
    pub fn compute_m_ratio(&self) -> f64 {
        let d_prim = self.compute_d_prime();
        if d_prim < 0.01 {
            return 0.0;
        }
        let meta_d = self.compute_meta_d();
        let ratio = meta_d / d_prim;
        if ratio.is_finite() && ratio >= 0.0 {
            ratio
        } else {
            0.0
        }
    }

    /// Compute d' (type-1 sensitivity): how well the system performs overall.
    fn compute_d_prime(&self) -> f64 {
        let n = self.predictions.len();
        if n < 10 {
            return 0.0;
        }
        let window = self.config.calibration_window.min(n);
        let recent = &self.predictions[n - window..];

        let total = recent.len();
        let total_correct = recent.iter().filter(|(_, c)| *c).count();
        let total_incorrect = total - total_correct;

        let hr = total_correct as f64 / total.max(1) as f64;
        let far = total_incorrect as f64 / total.max(1) as f64;

        let hr_clamped = hr.clamp(0.01, 0.99);
        let far_clamped = far.clamp(0.01, 0.99);

        let z_hr = gaussian_quantile(hr_clamped);
        let z_far = gaussian_quantile(far_clamped);

        let d = z_hr - z_far;
        if d.is_finite() {
            d
        } else {
            0.0
        }
    }
}

/// Approximation of the Gaussian quantile function (inverse CDF).
/// Uses the rational approximation (Peter Acklam, 2003).
fn gaussian_quantile(p: f64) -> f64 {
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    if p < 0.0 || p > 1.0 {
        return 0.0;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    let t = if p < 0.02425 {
        let q = (-2.0 * p.ln()).sqrt();
        let num = c[0] + q * (c[1] + q * (c[2] + q * (c[3] + q * (c[4] + q * c[5]))));
        let den = 1.0 + q * (d[0] + q * (d[1] + q * (d[2] + q * d[3])));
        (q, num / den)
    } else if p > 0.97575 {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        let num = c[0] + q * (c[1] + q * (c[2] + q * (c[3] + q * (c[4] + q * c[5]))));
        let den = 1.0 + q * (d[0] + q * (d[1] + q * (d[2] + q * d[3])));
        (q, -num / den)
    } else {
        let q = p - 0.5;
        let r = q * q;
        let num = a[0] + r * (a[1] + r * (a[2] + r * (a[3] + r * (a[4] + r * a[5]))));
        let den = 1.0 + r * (b[0] + r * (b[1] + r * (b[2] + r * (b[3] + r * b[4]))));
        (q, q * num / den)
    };
    t.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_report() {
        let eh = EpistemicHonesty::new(HonestyConfig::default());
        let r = eh.report();
        assert_eq!(r.total_predictions, 0);
        assert!((r.ece - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibrate_basic() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..20 {
            eh.calibrate(0.8, true);
        }
        let bin = &eh.bins[8];
        assert_eq!(bin.count, 20);
        assert_eq!(bin.correct, 20);
        assert!((bin.accuracy - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_honest_confidence_overconfident() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..10 {
            eh.calibrate(0.9, false);
        }
        let adjusted = eh.honest_confidence(0.9);
        assert!(adjusted < 0.9);
    }

    #[test]
    fn test_honest_confidence_underconfident() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..10 {
            eh.calibrate(0.3, true);
        }
        let adjusted = eh.honest_confidence(0.3);
        assert!(adjusted > 0.3);
    }

    #[test]
    fn test_epistemic_risk() {
        let eh = EpistemicHonesty::new(HonestyConfig::default());
        let risk = eh.epistemic_risk(0.9);
        assert!(risk > 0.0);
    }

    #[test]
    fn test_self_deception() {
        let eh = EpistemicHonesty::new(HonestyConfig::default());
        let claim = vec![0u8; 64];
        let evidence = vec![1u8; 64];
        let score = eh.detect_self_deception(&claim, &evidence);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_report_after_predictions() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..50 {
            eh.calibrate(0.7, true);
        }
        let r = eh.report();
        assert!(r.total_predictions > 0);
        assert!(r.ece >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..50 {
            eh.calibrate(0.9, true);
        }
        assert!(eh.report().total_predictions > 0);
        eh.reset_calibration();
        let r = eh.report();
        assert_eq!(r.total_predictions, 0);
        assert!((r.ece - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unknown_unknown() {
        let eh = EpistemicHonesty::new(HonestyConfig::default());
        let est = eh.unknown_unknown_estimate(0.5);
        assert!(est > 0.0);
        assert!(est < 1.0);
    }

    #[test]
    fn test_meta_d_with_perfect_calibration() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for _ in 0..50 {
            eh.calibrate(0.9, true);
        }
        let md = eh.compute_meta_d();
        assert!(md >= 0.0, "meta-d' should be non-negative, got {}", md);
    }

    #[test]
    fn test_meta_d_near_zero_for_random() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for i in 0..50 {
            let correct = i % 2 == 0;
            eh.calibrate(0.5, correct);
        }
        let md = eh.compute_meta_d();
        assert!(
            md < 1.0,
            "rand confidence should give low meta-d', got {}",
            md
        );
    }

    #[test]
    fn test_m_ratio_non_negative() {
        let mut eh = EpistemicHonesty::new(HonestyConfig::default());
        for i in 0..50 {
            let correct = i % 3 != 0;
            let conf = if correct { 0.8 } else { 0.4 };
            eh.calibrate(conf, correct);
        }
        let mr = eh.compute_m_ratio();
        assert!(mr >= 0.0, "M-ratio should be non-negative, got {}", mr);
    }

    #[test]
    fn test_gaussian_quantile_symmetry() {
        let z05 = gaussian_quantile(0.5);
        assert!((z05).abs() < 0.1, "p=0.5 should give ~0, got {}", z05);
        let z025 = gaussian_quantile(0.025);
        let z975 = gaussian_quantile(0.975);
        assert!((z025 + z975).abs() < 0.1, "z(0.025) should ≈ -z(0.975)");
    }

    #[test]
    fn test_gaussian_quantile_extremes() {
        let z001 = gaussian_quantile(0.001);
        assert!(z001 < 0.0, "p<0.5 gives negative z, got {}", z001);
        let z099 = gaussian_quantile(0.999);
        assert!(z099 > 0.0, "p>0.5 gives positive z, got {}", z099);
    }
}
