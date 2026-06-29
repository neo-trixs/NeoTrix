use crate::core::nt_core_hcube::spectral_vsa::SpectralVSA;
use std::sync::Arc;

/// Frequency band for spectral rule filtering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrequencyBand {
    LowPass(f64),
    BandPass(f64, f64),
    HighPass(f64),
    Notch(f64, f64),
}

/// A spectral logic rule: formula → spectral template + frequency band
#[derive(Debug, Clone)]
pub struct SpectralRule {
    pub template: Vec<f64>,
    pub band: FrequencyBand,
    pub weight: f64,
}

/// Graph Laplacian for spectral graph analysis
#[derive(Debug, Clone)]
pub struct GraphLaplacian {
    dim: usize,
    laplacian: Vec<Vec<f64>>,
    eigenvalues: Vec<f64>,
    eigenvectors: Vec<Vec<f64>>,
}

impl GraphLaplacian {
    pub fn new(adjacency: &[Vec<f64>]) -> Self {
        let n = adjacency.len();
        let mut deg = vec![0.0; n];
        for i in 0..n {
            deg[i] = adjacency[i].iter().sum();
        }
        let mut lap = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    lap[i][j] = deg[i];
                } else {
                    lap[i][j] = -adjacency[i][j];
                }
            }
        }
        let (eigvals, eigvecs) = eigen_decompose(&lap, n);
        Self {
            dim: n,
            laplacian: lap,
            eigenvalues: eigvals,
            eigenvectors: eigvecs,
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn laplacian(&self) -> &[Vec<f64>] {
        &self.laplacian
    }

    pub fn eigenvalues(&self) -> &[f64] {
        &self.eigenvalues
    }

    pub fn eigenvectors(&self) -> &[Vec<f64>] {
        &self.eigenvectors
    }

    /// Filter graph signal: keep components in [low_freq, high_freq] eigen-range [0,1]
    pub fn filter_signal(&self, signal: &[f64], low_ratio: f64, high_ratio: f64) -> Vec<f64> {
        let n = signal.len().min(self.dim);
        let low = (self.dim as f64 * low_ratio.clamp(0.0, 1.0)) as usize;
        let high = (self.dim as f64 * high_ratio.clamp(0.0, 1.0)) as usize;
        let mut result = vec![0.0; n];
        for k in low..high.min(self.dim) {
            let coeff: f64 = signal
                .iter()
                .zip(self.eigenvectors[k].iter())
                .take(n)
                .map(|(s, v)| s * v)
                .sum();
            for i in 0..n {
                result[i] += coeff * self.eigenvectors[k][i];
            }
        }
        result
    }

    /// Compute graph Fourier transform of a signal
    pub fn gft(&self, signal: &[f64]) -> Vec<f64> {
        let n = signal.len().min(self.dim);
        (0..self.dim)
            .map(|k| {
                signal
                    .iter()
                    .zip(self.eigenvectors[k].iter())
                    .take(n)
                    .map(|(s, v)| s * v)
                    .sum()
            })
            .collect()
    }

    /// Inverse graph Fourier transform
    pub fn igft(&self, spectrum: &[f64]) -> Vec<f64> {
        let n = self.dim;
        (0..n)
            .map(|i| {
                spectrum
                    .iter()
                    .zip(self.eigenvectors.iter())
                    .map(|(s, ev)| s * ev[i])
                    .sum()
            })
            .collect()
    }
}

fn eigen_decompose(matrix: &[Vec<f64>], n: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut eigvals = vec![0.0; n];
    let mut eigvecs = vec![vec![0.0; n]; n];
    for i in 0..n {
        eigvals[i] = matrix[i][i] - matrix[i].iter().filter(|&&v| v < 0.0).sum::<f64>();
        eigvecs[i][i] = 1.0;
    }
    eigvals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for j in 0..n {
        let idx = eigvals.iter().position(|&v| {
            (v - (matrix[j][j] - matrix[j].iter().filter(|&&v| v < 0.0).sum::<f64>())).abs() < 1e-9
        });
        if let Some(k) = idx {
            eigvecs[k][j] = 1.0;
        }
    }
    (eigvals, eigvecs)
}

/// Spectral filter: frequency-domain processing of VSA vectors
#[derive(Debug, Clone)]
pub struct SpectralFilter {
    cutoff_ratio: f64,
    filter_order: usize,
}

impl Default for SpectralFilter {
    fn default() -> Self {
        Self {
            cutoff_ratio: 0.3,
            filter_order: 2,
        }
    }
}

impl SpectralFilter {
    pub fn new(cutoff_ratio: f64, filter_order: usize) -> Self {
        Self {
            cutoff_ratio: cutoff_ratio.clamp(0.0, 1.0),
            filter_order: filter_order.max(1),
        }
    }

    pub fn cutoff_ratio(&self) -> f64 {
        self.cutoff_ratio
    }

    /// Apply a Butterworth-style low-pass filter in frequency domain
    pub fn lowpass(&self, spectrum: &[f64]) -> Vec<f64> {
        let n = spectrum.len();
        let cutoff_idx = (n as f64 * self.cutoff_ratio) as usize;
        spectrum
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                if i < cutoff_idx {
                    val
                } else {
                    let ratio = (i - cutoff_idx) as f64 / (n - cutoff_idx).max(1) as f64;
                    val * (-(self.filter_order as f64) * ratio).exp()
                }
            })
            .collect()
    }

    /// Band-pass filter
    pub fn bandpass(&self, spectrum: &[f64], low: f64, high: f64) -> Vec<f64> {
        let n = spectrum.len();
        let low_idx = (n as f64 * low.clamp(0.0, 1.0)) as usize;
        let high_idx = (n as f64 * high.clamp(0.0, 1.0)) as usize;
        spectrum
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                if i >= low_idx && i <= high_idx {
                    val
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Notch filter: suppress a specific frequency band
    pub fn notch(&self, spectrum: &[f64], center: f64, width: f64) -> Vec<f64> {
        let n = spectrum.len();
        let c = (n as f64 * center.clamp(0.0, 1.0)) as usize;
        let w = (n as f64 * width.clamp(0.0, 1.0)) as usize;
        let lo = c.saturating_sub(w);
        let hi = (c + w).min(n);
        spectrum
            .iter()
            .enumerate()
            .map(|(i, &val)| if i >= lo && i <= hi { val * 0.1 } else { val })
            .collect()
    }
}

/// Mixture of Spectral Experts
pub struct MoSpectralExperts {
    pub experts: Vec<Arc<dyn SpectralExpert + Send>>,
    pub gate_weights: Vec<f64>,
}

impl std::fmt::Debug for MoSpectralExperts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MoSpectralExperts({} experts)", self.experts.len())
    }
}

impl Clone for MoSpectralExperts {
    fn clone(&self) -> Self {
        Self {
            experts: self.experts.clone(),
            gate_weights: self.gate_weights.clone(),
        }
    }
}

impl MoSpectralExperts {
    pub fn new() -> Self {
        Self {
            experts: Vec::new(),
            gate_weights: Vec::new(),
        }
    }

    pub fn register_expert(&mut self, expert: Arc<dyn SpectralExpert + Send>, weight: f64) {
        self.experts.push(expert);
        self.gate_weights.push(weight);
    }

    pub fn apply(&self, spectrum: &[f64]) -> Vec<f64> {
        if self.experts.is_empty() {
            return spectrum.to_vec();
        }
        let total: f64 = self.gate_weights.iter().sum();
        let weights: Vec<f64> = if total > 0.0 {
            self.gate_weights.iter().map(|w| w / total).collect()
        } else {
            vec![1.0 / self.experts.len() as f64; self.experts.len()]
        };
        let mut result = vec![0.0; spectrum.len()];
        for (expert, &w) in self.experts.iter().zip(weights.iter()) {
            let filtered = expert.transform(spectrum);
            for (r, f) in result.iter_mut().zip(filtered.iter()) {
                *r += w * f;
            }
        }
        result
    }
}

pub trait SpectralExpert: Send + Sync {
    fn name(&self) -> &str;
    fn transform(&self, spectrum: &[f64]) -> Vec<f64>;
}

/// Low-pass spectral expert
#[derive(Debug, Clone)]
pub struct LowPassExpert {
    pub filter: SpectralFilter,
}

impl SpectralExpert for LowPassExpert {
    fn name(&self) -> &str {
        "lowpass"
    }
    fn transform(&self, spectrum: &[f64]) -> Vec<f64> {
        self.filter.lowpass(spectrum)
    }
}

/// Band-pass spectral expert
#[derive(Debug, Clone)]
pub struct BandPassExpert {
    pub low: f64,
    pub high: f64,
}

impl SpectralExpert for BandPassExpert {
    fn name(&self) -> &str {
        "bandpass"
    }
    fn transform(&self, spectrum: &[f64]) -> Vec<f64> {
        let n = spectrum.len();
        let low = (n as f64 * self.low.clamp(0.0, 1.0)) as usize;
        let high = (n as f64 * self.high.clamp(0.0, 1.0)) as usize;
        spectrum
            .iter()
            .enumerate()
            .map(|(i, &val)| if i >= low && i <= high { val } else { 0.0 })
            .collect()
    }
}

/// High-pass spectral expert
#[derive(Debug, Clone)]
pub struct HighPassExpert {
    pub cutoff: f64,
}

impl SpectralExpert for HighPassExpert {
    fn name(&self) -> &str {
        "highpass"
    }
    fn transform(&self, spectrum: &[f64]) -> Vec<f64> {
        let n = spectrum.len();
        let cutoff = (n as f64 * self.cutoff.clamp(0.0, 1.0)) as usize;
        spectrum
            .iter()
            .enumerate()
            .map(|(i, &val)| if i >= cutoff { val } else { 0.0 })
            .collect()
    }
}

/// Full Spectral NSR (Neuro-Symbolic Reasoning) engine
#[derive(Debug, Clone)]
pub struct SpectralNSR {
    pub laplacian: Option<GraphLaplacian>,
    pub rules: Vec<SpectralRule>,
    pub filter: SpectralFilter,
    pub experts: MoSpectralExperts,
    svsa: SpectralVSA,
}

impl Default for SpectralNSR {
    fn default() -> Self {
        Self {
            laplacian: None,
            rules: Vec::new(),
            filter: SpectralFilter::default(),
            experts: MoSpectralExperts::new(),
            svsa: SpectralVSA::default(),
        }
    }
}

impl SpectralNSR {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_laplacian(mut self, adjacency: &[Vec<f64>]) -> Self {
        self.laplacian = Some(GraphLaplacian::new(adjacency));
        self
    }

    pub fn add_rule(&mut self, template: Vec<f64>, band: FrequencyBand, weight: f64) {
        self.rules.push(SpectralRule {
            template,
            band,
            weight,
        });
    }

    pub fn add_expert(&mut self, expert: Arc<dyn SpectralExpert + Send>, weight: f64) {
        self.experts.register_expert(expert, weight);
    }

    /// Convert a graph signal to spectral domain via Laplacian eigenbasis
    pub fn graph_to_spectral(&self, signal: &[f64]) -> Vec<f64> {
        match &self.laplacian {
            Some(lap) => lap.gft(signal),
            None => signal.to_vec(),
        }
    }

    /// Apply spectral rules to a spectrum
    pub fn apply_spectral_rules(&self, spectrum: &[f64]) -> Vec<f64> {
        let mut result = spectrum.to_vec();
        for rule in &self.rules {
            let n = result.len().min(rule.template.len());
            match rule.band {
                FrequencyBand::LowPass(cutoff) => {
                    let cutoff_idx = (n as f64 * cutoff.clamp(0.0, 1.0)) as usize;
                    for i in 0..n {
                        if i < cutoff_idx {
                            result[i] += rule.weight * rule.template[i];
                        }
                    }
                }
                FrequencyBand::BandPass(lo, hi) => {
                    let lo_idx = (n as f64 * lo.clamp(0.0, 1.0)) as usize;
                    let hi_idx = (n as f64 * hi.clamp(0.0, 1.0)) as usize;
                    for i in lo_idx..hi_idx.min(n) {
                        result[i] += rule.weight * rule.template[i];
                    }
                }
                FrequencyBand::HighPass(cutoff) => {
                    let cutoff_idx = (n as f64 * cutoff.clamp(0.0, 1.0)) as usize;
                    for i in cutoff_idx..n {
                        result[i] += rule.weight * rule.template[i];
                    }
                }
                FrequencyBand::Notch(center, width) => {
                    let c = (n as f64 * center.clamp(0.0, 1.0)) as usize;
                    let w = (n as f64 * width.clamp(0.0, 1.0)) as usize;
                    let lo = c.saturating_sub(w);
                    let hi = (c + w).min(n);
                    for i in lo..hi {
                        result[i] -= rule.weight * rule.template[i];
                    }
                }
            }
        }
        result
    }

    /// Apply mixture of spectral experts
    pub fn apply_experts(&self, spectrum: &[f64]) -> Vec<f64> {
        self.experts.apply(spectrum)
    }

    /// Inverse graph Fourier transform back to signal domain
    pub fn spectral_to_signal(&self, spectrum: &[f64]) -> Vec<f64> {
        match &self.laplacian {
            Some(lap) => lap.igft(spectrum),
            None => spectrum.to_vec(),
        }
    }

    /// Full reasoning pipeline: signal → spectral → rules → experts → signal
    pub fn reason_vsa(&self, vsa_input: &[u8]) -> Vec<u8> {
        let signal: Vec<f64> = vsa_input
            .iter()
            .map(|&b| (b as f64 / 127.5) - 1.0)
            .collect();
        let spectrum = self.graph_to_spectral(&signal);
        let ruled = self.apply_spectral_rules(&spectrum);
        let filtered = self.apply_experts(&ruled);
        let output_signal = self.spectral_to_signal(&filtered);
        output_signal
            .iter()
            .map(|&v| {
                let clamped = v.max(-1.0).min(1.0);
                if clamped > 0.0 {
                    1
                } else {
                    0
                }
            })
            .collect()
    }

    pub fn svsa(&self) -> &SpectralVSA {
        &self.svsa
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

    fn make_adjacency(n: usize) -> Vec<Vec<f64>> {
        let mut adj = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j && (i as isize - j as isize).abs() <= 2 {
                    adj[i][j] = 1.0;
                }
            }
        }
        adj
    }

    #[test]
    fn test_graph_laplacian_construction() {
        let adj = make_adjacency(8);
        let lap = GraphLaplacian::new(&adj);
        assert_eq!(lap.dim(), 8);
        assert_eq!(lap.eigenvalues().len(), 8);
        assert_eq!(lap.eigenvectors().len(), 8);
    }

    #[test]
    fn test_laplacian_gft_igft_roundtrip() {
        let adj = make_adjacency(8);
        let lap = GraphLaplacian::new(&adj);
        let signal = vec![1.0, 0.5, 0.0, -0.5, -1.0, 0.0, 0.5, 1.0];
        let spectrum = lap.gft(&signal);
        assert_eq!(spectrum.len(), 8);
        let restored = lap.igft(&spectrum);
        for (s, r) in signal.iter().zip(restored.iter()) {
            assert!((s - r).abs() < 1e-9, "GFT roundtrip error: {} vs {}", s, r);
        }
    }

    #[test]
    fn test_spectral_filter_lowpass() {
        let filter = SpectralFilter::new(0.5, 2);
        let spectrum = vec![1.0, 0.8, 0.6, 0.4, 0.2, 0.0, -0.2, -0.4];
        let filtered = filter.lowpass(&spectrum);
        assert_eq!(filtered.len(), spectrum.len());
        assert!(filtered[0] == spectrum[0]);
        assert!(filtered[filtered.len() - 1].abs() < spectrum[spectrum.len() - 1].abs());
    }

    #[test]
    fn test_spectral_filter_bandpass() {
        let filter = SpectralFilter::default();
        let spectrum = vec![1.0; 16];
        let band = filter.bandpass(&spectrum, 0.25, 0.5);
        assert_eq!(band.len(), 16);
        let zeros_before = band[..4].iter().all(|&v| v == 0.0);
        assert!(zeros_before);
    }

    #[test]
    fn test_spectral_filter_notch() {
        let filter = SpectralFilter::default();
        let spectrum = vec![1.0; 16];
        let notched = filter.notch(&spectrum, 0.5, 0.1);
        assert_eq!(notched.len(), 16);
        let center_val = notched[8];
        assert!(center_val < 1.0);
    }

    #[test]
    fn test_mo_spectral_experts() {
        let mut moe = MoSpectralExperts::new();
        moe.register_expert(
            Arc::new(LowPassExpert {
                filter: SpectralFilter::new(0.3, 2),
            }),
            1.0,
        );
        moe.register_expert(Arc::new(HighPassExpert { cutoff: 0.5 }), 1.0);
        let spectrum = vec![1.0; 16];
        let result = moe.apply(&spectrum);
        assert_eq!(result.len(), 16);
        assert!(result.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_spectral_nsr_reason_vsa() {
        let adj = make_adjacency(8);
        let vsa = QuantizedVSA::random_binary();
        let nsr = SpectralNSR::new().with_laplacian(&adj);
        let output = nsr.reason_vsa(&vsa);
        assert_eq!(output.len(), VSA_DIM);
    }

    #[test]
    fn test_spectral_nsr_with_rules() {
        let mut nsr = SpectralNSR::new();
        nsr.add_rule(vec![1.0; VSA_DIM], FrequencyBand::LowPass(0.3), 0.5);
        nsr.add_rule(vec![0.5; VSA_DIM], FrequencyBand::HighPass(0.7), 0.3);
        let vsa = QuantizedVSA::random_binary();
        let output = nsr.reason_vsa(&vsa);
        assert_eq!(output.len(), VSA_DIM);
    }

    #[test]
    fn test_spectral_nsr_with_experts() {
        let mut nsr = SpectralNSR::new();
        nsr.add_expert(
            Arc::new(LowPassExpert {
                filter: SpectralFilter::new(0.3, 2),
            }),
            1.0,
        );
        nsr.add_expert(
            Arc::new(BandPassExpert {
                low: 0.2,
                high: 0.6,
            }),
            0.5,
        );
        let vsa = QuantizedVSA::random_binary();
        let output = nsr.reason_vsa(&vsa);
        assert_eq!(output.len(), VSA_DIM);
    }

    #[test]
    fn test_spectral_nsr_empty_rules() {
        let nsr = SpectralNSR::new();
        let vsa = vec![1u8; VSA_DIM];
        let output = nsr.reason_vsa(&vsa);
        assert_eq!(output.len(), VSA_DIM);
    }

    #[test]
    fn test_frequency_band_variants() {
        let low = FrequencyBand::LowPass(0.3);
        let band = FrequencyBand::BandPass(0.2, 0.6);
        let high = FrequencyBand::HighPass(0.7);
        let notch = FrequencyBand::Notch(0.5, 0.1);
        match low {
            FrequencyBand::LowPass(v) => assert!((v - 0.3).abs() < 1e-9),
            _ => panic!("unexpected FrequencyBand variant, expected LowPass"),
        }
        match band {
            FrequencyBand::BandPass(lo, hi) => {
                assert!(lo < hi);
            }
            _ => panic!("unexpected FrequencyBand variant, expected BandPass"),
        }
        match high {
            FrequencyBand::HighPass(v) => assert!((v - 0.7).abs() < 1e-9),
            _ => panic!("unexpected FrequencyBand variant, expected HighPass"),
        }
        match notch {
            FrequencyBand::Notch(c, w) => {
                assert!(c > w);
            }
            _ => panic!("unexpected FrequencyBand variant, expected Notch"),
        }
    }

    #[test]
    fn test_graph_laplacian_filter_signal() {
        let adj = make_adjacency(8);
        let lap = GraphLaplacian::new(&adj);
        let signal = vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let filtered = lap.filter_signal(&signal, 0.0, 0.5);
        assert_eq!(filtered.len(), 8);
    }
}
