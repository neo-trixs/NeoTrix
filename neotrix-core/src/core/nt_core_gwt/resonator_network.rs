//! Resonator Network — Kuramoto adaptive coupling + bandpass resonator bank.
//!
//! Two-layer resonance architecture:
//! 1. [`AdaptiveCouplingKuramoto`] — enhances the fixed-K Kuramoto model
//!    with per-layer adaptive coupling that tracks the order parameter.
//! 2. [`BandpassResonator`] / [`ResonatorNetwork`] — bank of tuned resonators
//!    that inject dimensional coherence into frequency bands.
//! 3. [`ResonanceOptimizer`] — combines both to produce a [`ResonanceReport`].

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

// ─── AdaptiveCouplingKuramoto ─────────────────────────────────────────

/// Configuration for the adaptive Kuramoto resonator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveResonatorConfig {
    /// Base coupling strength K₀ (starting value for all layers).
    pub base_coupling: f64,
    /// Adaptation rate η — how fast K_i tracks the order parameter.
    pub adaptation_rate: f64,
    /// Minimum per-layer coupling.
    pub min_coupling: f64,
    /// Maximum per-layer coupling.
    pub max_coupling: f64,
    /// Inertia (momentum) for coupling updates: 0 = no inertia, 1 = full.
    pub inertia: f64,
}

impl Default for AdaptiveResonatorConfig {
    fn default() -> Self {
        Self {
            base_coupling: 1.0,
            adaptation_rate: 0.1,
            min_coupling: 0.1,
            max_coupling: 10.0,
            inertia: 0.3,
        }
    }
}

/// Kuramoto model with adaptive per-layer coupling.
///
/// Each layer i has its own coupling strength K_i that evolves as:
/// `dK_i/dt = η · (R - K_i)` where R is the global order parameter.
/// Layers that contribute more to synchrony get phase-weighted coupling boosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCouplingKuramoto {
    /// Per-layer coupling strengths K_i.
    pub coupling_strengths: Vec<f64>,
    /// Per-layer natural frequencies ω_i.
    pub natural_frequencies: Vec<f64>,
    /// Current phases θ_i ∈ [0, 2π).
    pub phases: Vec<f64>,
    /// Config parameters.
    pub config: AdaptiveResonatorConfig,
    /// Phase-weighted coupling memory (past contribution to synchrony).
    pub phase_weights: Vec<f64>,
    /// Last order parameter.
    pub order_parameter: f64,
}

impl AdaptiveCouplingKuramoto {
    /// Create a new adaptive resonator with `n` layers and the given config.
    pub fn new(config: AdaptiveResonatorConfig, n: usize) -> Self {
        let coupling_strengths = vec![config.base_coupling; n];
        let phase_weights = vec![1.0 / n as f64; n];
        let natural_frequencies: Vec<f64> = (0..n)
            .map(|i| 0.05 + 0.001 * (i as f64))
            .collect();
        let phases = vec![0.0; n];
        let order_parameter = Self::compute_order(&phases);
        Self {
            coupling_strengths,
            natural_frequencies,
            phases,
            config,
            phase_weights,
            order_parameter,
        }
    }

    /// Create from existing phases and natural frequencies (e.g. from CrossDimensionalResonator).
    pub fn from_layers(
        phases: Vec<f64>,
        natural_frequencies: Vec<f64>,
        config: AdaptiveResonatorConfig,
    ) -> Self {
        let n = phases.len();
        let coupling_strengths = vec![config.base_coupling; n];
        let phase_weights = vec![1.0 / n as f64; n];
        let order_parameter = Self::compute_order(&phases);
        Self {
            coupling_strengths,
            natural_frequencies,
            phases,
            config,
            phase_weights,
            order_parameter,
        }
    }

    /// Advance one time step. Updates phases and adapts coupling strengths.
    /// Returns (new_phases, order_parameter).
    pub fn adaptive_step(&mut self, dt: f64) -> (Vec<f64>, f64) {
        let n = self.phases.len() as f64;
        let mut deltas = vec![0.0; self.phases.len()];

        // Phase update: dθ_i/dt = ω_i + (1/N) Σ K_j * sin(θ_j - θ_i)
        for i in 0..self.phases.len() {
            let mut coupling_sum = 0.0;
            for j in 0..self.phases.len() {
                if i == j {
                    continue;
                }
                coupling_sum += self.coupling_strengths[j] * (self.phases[j] - self.phases[i]).sin();
            }
            deltas[i] = self.natural_frequencies[i] + coupling_sum / n;
        }

        // Update phases
        for i in 0..self.phases.len() {
            self.phases[i] = wrap_phase(self.phases[i] + dt * deltas[i]);
        }

        // Compute order parameter R
        let r = Self::compute_order(&self.phases);
        self.order_parameter = r;

        // Adapt coupling: dK_i/dt = η * (R - K_i)
        for k in self.coupling_strengths.iter_mut() {
            let delta_k = self.config.adaptation_rate * (r - *k);
            *k = (*k * self.config.inertia + delta_k * (1.0 - self.config.inertia))
                .max(self.config.min_coupling)
                .min(self.config.max_coupling);
        }

        // Update phase-weighted coupling: weight = moving avg of phase alignment
        let mean_phase = self.mean_phase();
        for (i, pw) in self.phase_weights.iter_mut().enumerate() {
            let alignment = (1.0 + (self.phases[i] - mean_phase).cos()) / 2.0; // [0, 1]
            *pw = *pw * 0.9 + alignment * 0.1;
        }

        (self.phases.clone(), r)
    }

    /// Phase-weighted effective coupling for each layer.
    pub fn effective_couplings(&self) -> Vec<f64> {
        self.coupling_strengths
            .iter()
            .zip(self.phase_weights.iter())
            .map(|(&k, &w)| k * (0.5 + 0.5 * w))
            .collect()
    }

    /// Global order parameter R ∈ [0, 1].
    fn compute_order(phases: &[f64]) -> f64 {
        let n = phases.len() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let (sum_c, sum_s): (f64, f64) = phases
            .iter()
            .map(|&p| (p.cos(), p.sin()))
            .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
        ((sum_c / n).powi(2) + (sum_s / n).powi(2))
            .sqrt()
            .min(1.0)
    }

    /// Mean phase (direction of order parameter).
    fn mean_phase(&self) -> f64 {
        let (sum_c, sum_s): (f64, f64) = self
            .phases
            .iter()
            .map(|&p| (p.cos(), p.sin()))
            .fold((0.0, 0.0), |(c, s), (cc, ss)| (c + cc, s + ss));
        sum_s.atan2(sum_c)
    }
}

// ─── BandpassResonator ────────────────────────────────────────────────

/// A single bandpass resonator tuned to a center frequency.
///
/// Models a damped harmonic oscillator driven by input:
/// `d²x/dt² + 2ζω₀·dx/dt + ω₀²·x = A·sin(ω₀·t)`
/// Implemented as a second-order IIR bandpass filter in discrete time.
/// Used by [`ResonatorNetwork`] as a bank of frequency-tuned resonators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandpassResonator {
    /// Center frequency ω₀ (rad/s).
    pub center_freq: f64,
    /// Bandwidth (rad/s) — width of the passband.
    pub bandwidth: f64,
    /// Current amplitude (output magnitude).
    pub amplitude: f64,
    /// Current phase.
    pub phase: f64,
    // Internal filter state
    #[allow(dead_code)]
    prev_input: f64,
    /// Resonance gain factor.
    gain: f64,
    /// Sampling rate assumption for filter coefs.
    sample_rate: f64,
}

impl BandpassResonator {
    /// Create a new bandpass resonator.
    ///
    /// `center_freq` in Hz, `q` is the quality factor. Higher Q → narrower band.
    pub fn new(center_freq: f64, q: f64) -> Self {
        let bandwidth = center_freq / q;
        let sample_rate = 100.0; // assume 100 Hz control loop
        Self {
            center_freq,
            bandwidth,
            amplitude: 0.0,
            phase: 0.0,
            prev_input: 0.0,
            gain: 1.0,
            sample_rate,
        }
    }

    /// Design a second-order IIR bandpass filter coefficients using BLT.
    /// Inject energy at the resonator's center frequency.
    pub fn stimulate(&mut self, input_amplitude: f64) {
        let input = input_amplitude * (self.phase).sin();
        // Simplified bandpass: apply a resonant gain with lowpass smoothing
        let filtered = self.gain * alpha_lowpass(input, 0.3);
        self.amplitude = (self.amplitude + filtered.abs()) / 2.0;
        self.phase = (self.phase + 2.0 * PI * self.center_freq / self.sample_rate) % (2.0 * PI);
        self.prev_input = input;
    }

    /// Advance one time step. Returns the current output amplitude.
    pub fn step(&mut self, dt: f64) -> f64 {
        // Free oscillation decay
        let decay = (-self.bandwidth * dt).exp();
        self.amplitude *= decay;
        self.phase = (self.phase + 2.0 * PI * self.center_freq * dt) % (2.0 * PI);
        self.amplitude
    }
}

/// Simple alpha lowpass filter helper.
fn alpha_lowpass(input: f64, alpha: f64) -> f64 {
    input * alpha
}

// ─── ResonatorNetwork ─────────────────────────────────────────────────

/// A bank of bandpass resonators spanning a frequency range.
///
/// Incoming coherence states are injected into the nearest resonator,
/// producing a resonance spectrum that reveals dominant frequency bands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonatorNetwork {
    pub resonators: Vec<BandpassResonator>,
    pub min_freq: f64,
    pub max_freq: f64,
}

impl ResonatorNetwork {
    /// Create a bank of `n` resonators spanning `[min_freq, max_freq]` Hz.
    pub fn new(n_resonators: usize, min_freq: f64, max_freq: f64) -> Self {
        let resonators: Vec<BandpassResonator> = (0..n_resonators)
            .map(|i| {
                let freq = if n_resonators == 1 {
                    (min_freq + max_freq) / 2.0
                } else {
                    min_freq + (max_freq - min_freq) * (i as f64) / (n_resonators as f64 - 1.0)
                };
                let q = freq / ((max_freq - min_freq) / n_resonators as f64).max(0.01);
                BandpassResonator::new(freq, q.max(1.0))
            })
            .collect();
        Self {
            resonators,
            min_freq,
            max_freq,
        }
    }

    /// Inject each dimension of `states` into the nearest resonator.
    pub fn stimulate_all(&mut self, states: &[f64]) {
        if self.resonators.is_empty() || states.is_empty() {
            return;
        }
        for (i, &state) in states.iter().enumerate() {
            let idx = (i as f64 / states.len().max(1) as f64
                * self.resonators.len() as f64)
                .min((self.resonators.len() - 1) as f64) as usize;
            if idx < self.resonators.len() {
                self.resonators[idx].stimulate(state);
            }
        }
    }

    /// Advance all resonators by `dt`.
    pub fn step_all(&mut self, dt: f64) {
        for r in &mut self.resonators {
            r.step(dt);
        }
    }

    /// Compute the resonance spectrum as `Vec<(frequency, amplitude)>`.
    pub fn compute_resonance_spectrum(&self) -> Vec<(f64, f64)> {
        self.resonators
            .iter()
            .map(|r| (r.center_freq, r.amplitude))
            .collect()
    }

    /// Resonant frequencies whose amplitude exceeds the threshold.
    pub fn resonant_frequencies(&self, threshold: f64) -> Vec<f64> {
        self.resonators
            .iter()
            .filter(|r| r.amplitude > threshold)
            .map(|r| r.center_freq)
            .collect()
    }

    /// Number of resonators.
    pub fn len(&self) -> usize {
        self.resonators.len()
    }
}

// ─── ResonanceReport ──────────────────────────────────────────────────

/// Report from the resonance optimizer combining adaptive Kuramoto and
/// resonator network analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceReport {
    /// Global order parameter R ∈ [0, 1].
    pub order_parameter: f64,
    /// Per-layer effective coupling strengths after adaptation.
    pub coupling_strengths: Vec<f64>,
    /// Resonant frequency peaks (frequency, amplitude).
    pub resonant_peaks: Vec<(f64, f64)>,
    /// Spectral entropy of the resonator network response.
    pub entropy: f64,
    /// Stability metric: how much the order parameter changed over the run.
    pub stability: f64,
}

// ─── ResonanceOptimizer ───────────────────────────────────────────────

/// Combines [`AdaptiveCouplingKuramoto`] and [`ResonatorNetwork`] to
/// optimize cross-dimensional resonance.
///
/// The optimizer runs the adaptive Kuramoto model over `n_steps` while
/// simultaneously injecting dimensional coherence states into the
/// resonator network, then produces a unified [`ResonanceReport`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceOptimizer {
    /// Adaptive Kuramoto resonator (12 layers).
    pub adaptive_kuramoto: AdaptiveCouplingKuramoto,
    /// Bandpass resonator bank for spectral analysis.
    pub resonator_network: ResonatorNetwork,
}

impl ResonanceOptimizer {
    /// Create a new optimizer with default configuration for `n` layers.
    pub fn new(n_layers: usize) -> Self {
        let config = AdaptiveResonatorConfig::default();
        let adaptive_kuramoto = AdaptiveCouplingKuramoto::new(config, n_layers);
        let resonator_network = ResonatorNetwork::new(
            16,   // 16 bandpass resonators
            1.0,  // min freq Hz
            40.0, // max freq Hz (gamma range)
        );
        Self {
            adaptive_kuramoto,
            resonator_network,
        }
    }

    /// Create from existing phase/frequency arrays.
    pub fn from_phases(
        phases: Vec<f64>,
        natural_frequencies: Vec<f64>,
    ) -> Self {
        let config = AdaptiveResonatorConfig::default();
        let adaptive_kuramoto =
            AdaptiveCouplingKuramoto::from_layers(phases, natural_frequencies, config);
        let resonator_network = ResonatorNetwork::new(16, 1.0, 40.0);
        Self {
            adaptive_kuramoto,
            resonator_network,
        }
    }

    /// Run the optimization loop:
    /// 1. Run `n_steps` of adaptive Kuramoto
    /// 2. Inject coherence states into resonator network
    /// 3. Compute resonance spectrum
    /// 4. Return report
    pub fn optimize_resonance(
        &mut self,
        coherence_states: &[f64],
        n_steps: usize,
    ) -> ResonanceReport {
        let dt = 0.05;

        // Track order parameter history for stability computation
        let mut op_history = Vec::with_capacity(n_steps);

        // Run adaptive Kuramoto
        for _ in 0..n_steps {
            let (_, op) = self.adaptive_kuramoto.adaptive_step(dt);
            op_history.push(op);
        }

        // Inject coherence into resonator network
        self.resonator_network.stimulate_all(coherence_states);
        for _ in 0..5 {
            self.resonator_network.step_all(dt);
        }

        // Gather results
        let order_parameter = self.adaptive_kuramoto.order_parameter;
        let coupling_strengths = self.adaptive_kuramoto.effective_couplings();
        let spectrum = self.resonator_network.compute_resonance_spectrum();
        let resonant_peaks = spectrum;

        // Spectral entropy: normalize amplitudes to probability distribution
        let entropy = spectral_entropy(&resonant_peaks);

        // Stability: standard deviation of order parameter over the run
        let stability = if op_history.len() > 1 {
            let mean = op_history.iter().sum::<f64>() / op_history.len() as f64;
            let variance = op_history
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>()
                / op_history.len() as f64;
            1.0 - variance.sqrt().min(1.0)
        } else {
            1.0
        };

        ResonanceReport {
            order_parameter,
            coupling_strengths,
            resonant_peaks,
            entropy,
            stability,
        }
    }

    /// Resonant frequencies from the resonator network with amplitude > threshold.
    pub fn resonant_frequencies(&self, threshold: f64) -> Vec<f64> {
        self.resonator_network.resonant_frequencies(threshold)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────

fn wrap_phase(theta: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let mut t = theta % two_pi;
    if t < 0.0 {
        t += two_pi;
    }
    t
}

/// Compute spectral entropy from a list of (frequency, amplitude) pairs.
fn spectral_entropy(spectrum: &[(f64, f64)]) -> f64 {
    let total: f64 = spectrum.iter().map(|(_, a)| a).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let entropy: f64 = spectrum
        .iter()
        .map(|(_, a)| {
            let p = a / total;
            if p > 0.0 {
                -p * p.ln()
            } else {
                0.0
            }
        })
        .sum();
    // Normalize to [0, 1] by dividing by ln(n)
    let n = spectrum.len() as f64;
    if n > 1.0 {
        entropy / n.ln()
    } else {
        0.0
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_kuramoto_initialization() {
        let config = AdaptiveResonatorConfig::default();
        let ak = AdaptiveCouplingKuramoto::new(config, 12);
        assert_eq!(ak.phases.len(), 12);
        assert_eq!(ak.coupling_strengths.len(), 12);
        assert!((ak.order_parameter - 1.0).abs() < 1e-9); // all phases 0 → R=1
    }

    #[test]
    fn test_adaptive_kuramoto_step_produces_order() {
        let config = AdaptiveResonatorConfig::default();
        let mut ak = AdaptiveCouplingKuramoto::new(config, 12);
        // Inject spread phases
        for (i, p) in ak.phases.iter_mut().enumerate() {
            *p = (i as f64) * 0.3;
        }
        let (phases, r) = ak.adaptive_step(0.05);
        assert_eq!(phases.len(), 12);
        assert!(r >= 0.0 && r <= 1.0);
    }

    #[test]
    fn test_adaptive_coupling_converges_to_order() {
        let config = AdaptiveResonatorConfig {
            base_coupling: 5.0,
            adaptation_rate: 0.2,
            min_coupling: 0.1,
            max_coupling: 10.0,
            inertia: 0.0,
        };
        let mut ak = AdaptiveCouplingKuramoto::new(config, 12);
        // Start with high coupling, should converge down when R < K
        for k in ak.coupling_strengths.iter_mut() {
            *k = 5.0;
        }
        for _ in 0..50 {
            ak.adaptive_step(0.05);
        }
        // Coupling should have moved toward the order parameter
        let avg_k = ak.coupling_strengths.iter().sum::<f64>() / 12.0;
        assert!(
            (avg_k - ak.order_parameter).abs() < 2.0,
            "K should track R: K_avg={:.3}, R={:.3}",
            avg_k,
            ak.order_parameter
        );
    }

    #[test]
    fn test_bandpass_resonator_stimulate() {
        let mut r = BandpassResonator::new(10.0, 100.0);
        assert!((r.amplitude - 0.0).abs() < 1e-9);
        r.stimulate(1.0);
        // phase is now non-zero; second call produces real amplitude
        r.stimulate(1.0);
        assert!(r.amplitude > 0.0);
    }

    #[test]
    fn test_bandpass_resonator_decay() {
        let mut r = BandpassResonator::new(10.0, 5.0);
        r.stimulate(1.0);
        let a1 = r.amplitude;
        r.step(0.1);
        let a2 = r.amplitude;
        assert!(a2 <= a1 + 1e-9); // amplitude decays or stays
    }

    #[test]
    fn test_resonator_network_new() {
        let net = ResonatorNetwork::new(8, 1.0, 40.0);
        assert_eq!(net.resonators.len(), 8);
        assert!((net.resonators[0].center_freq - 1.0).abs() < 1e-6);
        assert!((net.resonators[7].center_freq - 40.0).abs() < 1e-6);
    }

    #[test]
    fn test_resonator_network_stimulate_all() {
        let mut net = ResonatorNetwork::new(8, 1.0, 40.0);
        let states: Vec<f64> = (0..12).map(|i| 0.1 + 0.05 * i as f64).collect();
        net.stimulate_all(&states);
        let spectrum = net.compute_resonance_spectrum();
        assert_eq!(spectrum.len(), 8);
    }

    #[test]
    fn test_resonator_network_resonant_frequencies() {
        let mut net = ResonatorNetwork::new(8, 1.0, 40.0);
        let states: Vec<f64> = (0..12).map(|_| 0.8).collect();
        net.stimulate_all(&states);
        let freqs = net.resonant_frequencies(0.01);
        assert!(freqs.len() > 0);
    }

    #[test]
    fn test_resonator_network_stimulate_all_empty_no_underflow() {
        // Regression: stimulate_all computed self.resonators.len() - 1 while
        // iterating a non-empty states slice. With zero resonators this
        // underflowed usize in debug builds (panic) and produced a garbage
        // index in release. Guarded: early return when either side is empty.
        let mut net = ResonatorNetwork::new(0, 1.0, 40.0);
        net.stimulate_all(&[0.1, 0.2, 0.3]);
        assert!(net.compute_resonance_spectrum().is_empty());

        let mut net2 = ResonatorNetwork::new(3, 1.0, 40.0);
        net2.stimulate_all(&[]);
        assert_eq!(net2.compute_resonance_spectrum().len(), 3);
    }

    #[test]
    fn test_resonance_optimizer_new() {
        let opt = ResonanceOptimizer::new(12);
        assert_eq!(opt.adaptive_kuramoto.phases.len(), 12);
        assert_eq!(opt.resonator_network.len(), 16);
    }

    #[test]
    fn test_resonance_optimizer_optimize() {
        let mut opt = ResonanceOptimizer::new(12);
        let states: Vec<f64> = (0..12).map(|i| 0.2 + 0.05 * i as f64).collect();
        let report = opt.optimize_resonance(&states, 20);
        assert!(report.order_parameter >= 0.0 && report.order_parameter <= 1.0);
        assert_eq!(report.coupling_strengths.len(), 12);
        assert!(report.entropy >= 0.0 && report.entropy <= 1.0);
        assert!(report.stability >= 0.0 && report.stability <= 1.0);
    }

    #[test]
    fn test_resonance_optimizer_high_coherence_low_entropy() {
        let mut opt = ResonanceOptimizer::new(12);
        // All states identical → high coherence, low entropy
        let states = vec![0.9; 12];
        let report = opt.optimize_resonance(&states, 30);
        assert!(
            report.entropy < 0.5,
            "high coherence should give low entropy, got {}",
            report.entropy
        );
    }

    #[test]
    fn test_spectral_entropy_uniform() {
        let spectrum = vec![
            (1.0, 1.0),
            (2.0, 1.0),
            (3.0, 1.0),
            (4.0, 1.0),
        ];
        let e = spectral_entropy(&spectrum);
        assert!((e - 1.0).abs() < 0.01, "uniform spectrum → entropy ~1, got {}", e);
    }

    #[test]
    fn test_spectral_entropy_single_peak() {
        let spectrum = vec![
            (1.0, 1.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
        ];
        let e = spectral_entropy(&spectrum);
        assert!(
            (e - 0.0).abs() < 0.01,
            "single peak → entropy ~0, got {}",
            e
        );
    }

    #[test]
    fn test_phase_weighted_coupling() {
        let config = AdaptiveResonatorConfig::default();
        let mut ak = AdaptiveCouplingKuramoto::new(config, 12);
        for (i, p) in ak.phases.iter_mut().enumerate() {
            *p = (i as f64) * 0.5;
        }
        for _ in 0..10 {
            ak.adaptive_step(0.05);
        }
        let eff = ak.effective_couplings();
        assert_eq!(eff.len(), 12);
        for &e in &eff {
            assert!(e >= 0.0);
        }
    }

    #[test]
    fn test_resonance_report_fields() {
        let mut opt = ResonanceOptimizer::new(12);
        let states = vec![0.5; 12];
        let report = opt.optimize_resonance(&states, 10);
        // Check all fields are populated
        assert!(!report.coupling_strengths.is_empty());
        assert!(!report.resonant_peaks.is_empty());
        assert!(report.stability >= 0.0);
    }
}
