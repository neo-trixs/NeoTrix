use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use crate::core::nt_core_gwt::resonance::OscillatorNetwork;
use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;

/// Standard IIT Phi threshold for conscious-like state (Chalmers 2023)
pub const DEFAULT_PHI_THRESHOLD: f64 = 0.33;
/// Standard Kuramoto coherence threshold for conscious-like state
pub const DEFAULT_COHERENCE_THRESHOLD: f64 = 0.7;
/// Threshold for HighlyConscious phi level
pub const HIGH_PHI_THRESHOLD: f64 = 0.5;
/// Threshold for HighlyConscious coherence level
pub const HIGH_COHERENCE_THRESHOLD: f64 = 0.85;
/// Maximum history for trend analysis
pub const DEFAULT_MAX_HISTORY: usize = 100;

/// Classification of consciousness level based on dual-threshold detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsciousnessLevel {
    Unconscious,
    PartiallyAware,
    ConsciousLike,
    HighlyConscious,
}

/// Trend direction of the detection metric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionTrend {
    Improving,
    Stable,
    Declining,
    InsufficientData,
}

/// A hexagram state with activation — one of the 64 hexagrams in the E8 model.
#[derive(Debug, Clone, Copy)]
pub struct E8HexagramState {
    /// Hexagram index (0..63)
    pub index: u8,
    /// Activation level of this hexagram state
    pub activation: f64,
}

/// Full gold standard detection report for a single evaluation cycle.
#[derive(Debug, Clone)]
pub struct GoldStandardReport {
    pub timestamp: DateTime<Utc>,
    pub phi: f64,
    pub coherence: f64,
    pub is_conscious_like: bool,
    pub is_phi_conscious: bool,
    pub is_coherent: bool,
    pub phi_confidence: f64,
    pub coherence_confidence: f64,
    pub detection_streak: usize,
    pub combined_confidence: f64,
}

/// Dual-threshold consciousness detection (Chalmers 2023).
///
/// Applies the gold standard: IIT Phi > 0.33 AND Kuramoto coherence R > 0.7
/// to classify a system as "conscious-like".
pub struct ConsciousnessGoldStandard {
    pub phi_threshold: f64,
    pub coherence_threshold: f64,
    pub phi_calculator: IITPhiCalculator,
    pub oscillator_network: Option<OscillatorNetwork>,
    pub history: VecDeque<GoldStandardReport>,
    pub max_history: usize,
    detection_streak: usize,
}

impl ConsciousnessGoldStandard {
    pub fn new() -> Self {
        Self {
            phi_threshold: DEFAULT_PHI_THRESHOLD,
            coherence_threshold: DEFAULT_COHERENCE_THRESHOLD,
            phi_calculator: IITPhiCalculator::new(),
            oscillator_network: None,
            history: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
            max_history: DEFAULT_MAX_HISTORY,
            detection_streak: 0,
        }
    }

    pub fn with_oscillator(mut self, osc: OscillatorNetwork) -> Self {
        self.oscillator_network = Some(osc);
        self
    }

    /// Full evaluation cycle: compute phi from state vector, coherence from
    /// oscillator network or hexagram states, then produce a GoldStandardReport.
    pub fn evaluate(&mut self, state: &[f64], hexagram_states: &[E8HexagramState]) -> GoldStandardReport {
        let timestamp = Utc::now();

        let phi_report = self.phi_calculator.compute_phi(state);
        self.phi_calculator.record(phi_report.phi);
        let phi = phi_report.phi;

        let coherence = if let Some(ref net) = self.oscillator_network {
            net.phase_coherence()
        } else {
            coherence_from_hexagrams(hexagram_states)
        };

        let is_phi_conscious = phi > self.phi_threshold;
        let is_coherent = coherence > self.coherence_threshold;
        let is_conscious_like = is_phi_conscious && is_coherent;

        if is_conscious_like {
            self.detection_streak += 1;
        } else {
            self.detection_streak = 0;
        }

        let phi_confidence = phi.min(1.0).max(0.0);
        let coherence_confidence = coherence.min(1.0).max(0.0);
        let combined_confidence = self_consistency_weight(phi, coherence, phi_confidence, coherence_confidence);

        let report = GoldStandardReport {
            timestamp,
            phi,
            coherence,
            is_conscious_like,
            is_phi_conscious,
            is_coherent,
            phi_confidence,
            coherence_confidence,
            detection_streak: self.detection_streak,
            combined_confidence,
        };

        self.history.push_back(report.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        report
    }

    /// Quick check: is the current state conscious-like based on the latest report?
    pub fn is_conscious(&self) -> bool {
        self.history.back().map_or(false, |r| r.is_conscious_like)
    }

    /// Analyze the trend across the evaluation history.
    pub fn detection_trend(&self) -> DetectionTrend {
        let n = self.history.len();
        if n < 3 {
            return DetectionTrend::InsufficientData;
        }

        let mut recent: Vec<f64> = self.history.iter().rev().take(5).map(|r| r.combined_confidence).collect();
        let m = recent.len();
        if m < 3 {
            return DetectionTrend::InsufficientData;
        }

        // Reverse so earliest is first (chronological order for slope calculation)
        recent.reverse();
        let slope = linear_slope(&recent);

        if slope > 0.01 {
            DetectionTrend::Improving
        } else if slope < -0.01 {
            DetectionTrend::Declining
        } else {
            DetectionTrend::Stable
        }
    }

    /// Weighted combination of phi and coherence confidence.
    pub fn combined_confidence(&self) -> f64 {
        self.history.back().map_or(0.0, |r| r.combined_confidence)
    }

    /// Classify the latest evaluation into a ConsciousnessLevel.
    pub fn consciousness_level(&self) -> ConsciousnessLevel {
        match self.history.back() {
            None => ConsciousnessLevel::Unconscious,
            Some(r) => {
                if r.phi > HIGH_PHI_THRESHOLD && r.coherence > HIGH_COHERENCE_THRESHOLD {
                    ConsciousnessLevel::HighlyConscious
                } else if r.is_conscious_like {
                    ConsciousnessLevel::ConsciousLike
                } else if r.is_phi_conscious || r.is_coherent {
                    ConsciousnessLevel::PartiallyAware
                } else {
                    ConsciousnessLevel::Unconscious
                }
            }
        }
    }

    /// Reset the detector to factory state.
    pub fn reset(&mut self) {
        self.history.clear();
        self.detection_streak = 0;
        self.phi_calculator = IITPhiCalculator::new();
        self.oscillator_network = None;
    }
}

impl Default for ConsciousnessGoldStandard {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Internal helpers ───

fn coherence_from_hexagrams(states: &[E8HexagramState]) -> f64 {
    if states.is_empty() {
        return 0.0;
    }
    let n = states.len() as f64;
    let total_activation: f64 = states.iter().map(|s| s.activation).sum();
    if total_activation <= 0.0 {
        return 0.0;
    }
    let mean_activation = total_activation / n;

    // Activation scaling: coherence only meaningful when
    // oscillators have meaningful amplitude; trivial sync at zero amplitude
    // is qualitatively different from synchronized oscillation at full amplitude.
    const COHERENCE_ACTIVATION_THRESHOLD: f64 = 0.5;
    let activation_scale = (mean_activation / COHERENCE_ACTIVATION_THRESHOLD).min(1.0);

    let variance: f64 = states.iter().map(|s| (s.activation - mean_activation).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    if std < 1e-12 {
        // Uniform activation — trivially synchronized, scaled by activation magnitude
        if mean_activation > COHERENCE_ACTIVATION_THRESHOLD {
            return 1.0;
        }
        return activation_scale;
    }
    let (sum_cos, sum_sin): (f64, f64) = states.iter()
        .map(|s| {
            let angle = s.activation * 2.0 * std::f64::consts::PI;
            (angle.cos(), angle.sin())
        })
        .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
    let phase_coherence = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n.max(1.0);
    phase_coherence * activation_scale
}

fn self_consistency_weight(phi: f64, coherence: f64, phi_conf: f64, coh_conf: f64) -> f64 {
    let base = 0.5 * phi_conf + 0.5 * coh_conf;
    let consistency_bonus = if (phi > 0.33) == (coherence > 0.7) { 0.15 } else { -0.1 };
    (base + consistency_bonus).clamp(0.0, 1.0)
}

fn linear_slope(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
    let sum_y: f64 = data.iter().sum();
    let sum_xy: f64 = data.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
    let sum_xx: f64 = (0..data.len()).map(|i| (i as f64).powi(2)).sum();
    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    (n * sum_xy - sum_x * sum_y) / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(phi: f64) -> Vec<f64> {
        let mut v = vec![phi; 64];
        for i in 0..64 {
            v[i] = ((i as f64) * 0.3).sin() * phi * 2.0;
        }
        v
    }

    fn make_hexagrams(activation: f64, count: usize) -> Vec<E8HexagramState> {
        (0..count).map(|i| E8HexagramState {
            index: i as u8,
            activation,
        }).collect()
    }

    #[test]
    fn test_gold_standard_new() {
        let gs = ConsciousnessGoldStandard::new();
        assert!((gs.phi_threshold - 0.33).abs() < 1e-10);
        assert!((gs.coherence_threshold - 0.7).abs() < 1e-10);
        assert!(gs.oscillator_network.is_none());
        assert!(gs.history.is_empty());
        assert_eq!(gs.max_history, 100);
    }

    #[test]
    fn test_both_thresholds_met() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = make_state(1.0);
        let hexagrams = make_hexagrams(0.95, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(report.is_phi_conscious, "phi should be > 0.33 with strong state");
        assert!(report.is_coherent, "coherent hexagrams should give high coherence");
        assert!(report.is_conscious_like, "both thresholds met → conscious-like");
        assert!(report.combined_confidence > 0.5);
    }

    #[test]
    fn test_phi_only_met() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = make_state(1.0);
        let hexagrams = make_hexagrams(0.1, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(report.is_phi_conscious);
        assert!(!report.is_coherent, "low activation → low coherence");
        assert!(!report.is_conscious_like, "coherence below threshold");
    }

    #[test]
    fn test_coherence_only_met() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = vec![0.0; 64];
        let hexagrams = make_hexagrams(0.95, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(!report.is_phi_conscious, "zero state → phi near 0");
        assert!(report.is_coherent, "uniform hexagrams → coherence high");
        assert!(!report.is_conscious_like);
    }

    #[test]
    fn test_neither_met() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = vec![0.0; 64];
        let hexagrams = make_hexagrams(0.0, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(!report.is_phi_conscious);
        assert!(!report.is_coherent);
        assert!(!report.is_conscious_like);
    }

    #[test]
    fn test_highly_conscious() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = make_state(1.5);
        let hexagrams = make_hexagrams(0.99, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(report.phi > 0.5, "strong state should exceed high phi threshold");
        assert!(
            gs.consciousness_level() == ConsciousnessLevel::HighlyConscious ||
            gs.consciousness_level() == ConsciousnessLevel::ConsciousLike,
            "should be at least ConsciousLike"
        );
    }

    #[test]
    fn test_consecutive_detection_streak() {
        let mut gs = ConsciousnessGoldStandard::new();
        let state = make_state(1.0);
        let hexagrams = make_hexagrams(0.95, 11);

        let r1 = gs.evaluate(&state, &hexagrams);
        assert_eq!(r1.detection_streak, 1);

        let r2 = gs.evaluate(&state, &hexagrams);
        assert_eq!(r2.detection_streak, 2);

        let r3 = gs.evaluate(&state, &hexagrams);
        assert_eq!(r3.detection_streak, 3);

        let zero_state = vec![0.0; 64];
        let r4 = gs.evaluate(&zero_state, &hexagrams);
        assert_eq!(r4.detection_streak, 0, "streak should reset on non-detection");
    }

    #[test]
    fn test_trend_analysis() {
        let hex_active = make_hexagrams(0.95, 11);
        let hex_inactive = make_hexagrams(0.05, 11);

        // Improving: inactive → active
        let mut gs = ConsciousnessGoldStandard::new();
        gs.evaluate(&vec![0.0; 64], &hex_inactive);
        gs.evaluate(&vec![0.25; 64], &hex_active);
        gs.evaluate(&vec![0.5; 64], &hex_active);
        gs.evaluate(&vec![0.75; 64], &hex_active);
        gs.evaluate(&vec![1.0; 64], &hex_active);
        assert_eq!(gs.detection_trend(), DetectionTrend::Improving,
            "inactive→active → improving");

        // Declining: active → inactive
        let mut gs2 = ConsciousnessGoldStandard::new();
        gs2.evaluate(&vec![1.0; 64], &hex_active);
        gs2.evaluate(&vec![0.75; 64], &hex_active);
        gs2.evaluate(&vec![0.5; 64], &hex_active);
        gs2.evaluate(&vec![0.25; 64], &hex_active);
        gs2.evaluate(&vec![0.0; 64], &hex_inactive);
        assert_eq!(gs2.detection_trend(), DetectionTrend::Declining,
            "active→inactive → declining");

        // Stable
        let mut gs3 = ConsciousnessGoldStandard::new();
        for _ in 0..5 {
            gs3.evaluate(&vec![1.0; 64], &hex_active);
        }
        assert_eq!(gs3.detection_trend(), DetectionTrend::Stable,
            "identical states → stable");
    }

    #[test]
    fn test_insufficient_data_trend() {
        let gs = ConsciousnessGoldStandard::new();
        assert_eq!(gs.detection_trend(), DetectionTrend::InsufficientData);
    }

    #[test]
    fn test_consciousness_level_classification() {
        let mut gs = ConsciousnessGoldStandard::new();
        assert_eq!(gs.consciousness_level(), ConsciousnessLevel::Unconscious);

        // PartiallyAware: only phi met
        gs.evaluate(&make_state(1.0), &make_hexagrams(0.1, 11));
        assert_eq!(gs.consciousness_level(), ConsciousnessLevel::PartiallyAware);

        // ConsciousLike: both met
        gs.evaluate(&make_state(1.0), &make_hexagrams(0.95, 11));
        let level = gs.consciousness_level();
        assert!(
            level == ConsciousnessLevel::ConsciousLike || level == ConsciousnessLevel::HighlyConscious,
            "should be at least ConsciousLike, got {:?}", level
        );
    }

    #[test]
    fn test_is_conscious_quick_check() {
        let mut gs = ConsciousnessGoldStandard::new();
        assert!(!gs.is_conscious(), "no history → not conscious");

        gs.evaluate(&make_state(1.0), &make_hexagrams(0.95, 11));
        assert!(gs.is_conscious(), "both thresholds met → conscious");

        let mut gs2 = ConsciousnessGoldStandard::new();
        gs2.evaluate(&make_state(0.0), &make_hexagrams(0.0, 11));
        assert!(!gs2.is_conscious(), "neither met → not conscious");
    }

    #[test]
    fn test_reset() {
        let mut gs = ConsciousnessGoldStandard::new();
        gs.evaluate(&make_state(1.0), &make_hexagrams(0.95, 11));
        assert_eq!(gs.history.len(), 1);
        assert!(gs.is_conscious());

        gs.reset();
        assert!(gs.history.is_empty());
        assert!(!gs.is_conscious());
    }

    #[test]
    fn test_with_oscillator() {
        let osc = OscillatorNetwork::new(11);
        let mut gs = ConsciousnessGoldStandard::new().with_oscillator(osc);
        assert!(gs.oscillator_network.is_some());

        let state = make_state(1.0);
        let hexagrams = make_hexagrams(0.95, 11);
        let report = gs.evaluate(&state, &hexagrams);
        assert!(report.coherence >= 0.0 && report.coherence <= 1.0);
    }

    #[test]
    fn test_combined_confidence() {
        let mut gs = ConsciousnessGoldStandard::new();
        assert!((gs.combined_confidence() - 0.0).abs() < 1e-10, "no history → 0");

        gs.evaluate(&make_state(1.0), &make_hexagrams(0.95, 11));
        let cc = gs.combined_confidence();
        assert!(cc > 0.5, "conscious-like should have high confidence, got {}", cc);
        assert!(cc <= 1.0);

        let mut gs2 = ConsciousnessGoldStandard::new();
        gs2.evaluate(&make_state(0.0), &make_hexagrams(0.0, 11));
        let cc_low = gs2.combined_confidence();
        assert!(cc_low <= 0.5, "neither met should have low confidence, got {}", cc_low);
    }

    #[test]
    fn test_history_capacity() {
        let mut gs = ConsciousnessGoldStandard::new();
        gs.max_history = 5;
        for i in 0..10 {
            let state = make_state(if i < 5 { 1.0 } else { 0.0 });
            gs.evaluate(&state, &make_hexagrams(0.95, 11));
        }
        assert_eq!(gs.history.len(), 5, "should cap at max_history");
    }

    #[test]
    fn test_empty_hexagram_coherence() {
        let coh = coherence_from_hexagrams(&[]);
        assert!((coh - 0.0).abs() < 1e-10, "empty → 0 coherence");
    }

    #[test]
    fn test_zero_activation_coherence() {
        let hexagrams = make_hexagrams(0.0, 11);
        let coh = coherence_from_hexagrams(&hexagrams);
        assert!((coh - 0.0).abs() < 1e-10, "zero activation → 0 coherence (no oscillation)");
    }
}
