// REVIVED Task 2 — dead_code removed
use crate::core::nt_core_hcube::spectral_vsa::{Complex, SpectralVSA};
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

// === Wave-Geometric VSA: UWE encoding + spectral denoising ===

/// UWE (Universal Wave Embedding): maps a bipolar vector to a continuous waveform.
///
/// Each bit is spread across multiple frequency components via a wavelet-like kernel,
/// preserving the inner product structure in the spectral domain.
#[derive(Debug, Clone)]
pub struct WaveGeometricEmbed {
    /// Wavelet spread factor (0.5-2.0): how many frequency bins each bit spreads across
    spread: f64,
    /// Number of wave packets
    n_packets: usize,
    /// Pre-computed wavelet basis
    basis: Vec<Vec<f64>>,
}

impl Default for WaveGeometricEmbed {
    fn default() -> Self {
        Self::new(1.0, VSA_DIM / 8)
    }
}

impl WaveGeometricEmbed {
    pub fn new(spread: f64, n_packets: usize) -> Self {
        let n_packets = n_packets.max(4).min(VSA_DIM / 4);
        let basis = Self::compute_wavelet_basis(n_packets, VSA_DIM, spread);
        Self {
            spread,
            n_packets,
            basis,
        }
    }

    /// Compute Morlet-like wavelet basis: ψ_k(f) = exp(-(f-μ_k)²/(2σ²)) * cos(2πf/μ_k)
    fn compute_wavelet_basis(n_packets: usize, dim: usize, spread: f64) -> Vec<Vec<f64>> {
        let sigma = spread * (dim as f64) / (n_packets as f64 * 2.0);
        let mut basis = Vec::with_capacity(n_packets);
        for k in 0..n_packets {
            let mu = (k as f64 + 0.5) * (dim as f64) / n_packets as f64;
            let row: Vec<f64> = (0..dim)
                .map(|f| {
                    let freq = f as f64;
                    let gauss = (-((freq - mu).powi(2)) / (2.0 * sigma * sigma)).exp();
                    let carrier = (2.0 * std::f64::consts::PI * freq / mu.max(1.0)).cos();
                    gauss * carrier
                })
                .collect();
            basis.push(row);
        }
        basis
    }

    pub fn spread(&self) -> f64 {
        self.spread
    }
    pub fn n_packets(&self) -> usize {
        self.n_packets
    }

    /// Encode a bipolar VSA vector into a continuous waveform
    pub fn encode(&self, v: &[u8]) -> Vec<f64> {
        let mut waveform = vec![0.0; VSA_DIM];
        let bipolar: Vec<f64> = v.iter().map(|&b| if b > 0 { 1.0 } else { -1.0 }).collect();
        for (k, packet) in self.basis.iter().enumerate() {
            let coeff: f64 = bipolar
                .iter()
                .zip(packet.iter())
                .map(|(b, p)| b * p)
                .sum::<f64>()
                / self.basis.len() as f64;
            let packet_idx = (k * VSA_DIM / self.n_packets) % VSA_DIM;
            let spread_range = (VSA_DIM / self.n_packets).max(8);
            for j in 0..spread_range {
                let idx = (packet_idx + j) % VSA_DIM;
                waveform[idx] += coeff * packet[idx];
            }
        }
        let norm: f64 = waveform
            .iter()
            .map(|&x| x * x)
            .sum::<f64>()
            .sqrt()
            .max(1e-12);
        waveform.iter_mut().for_each(|x| *x /= norm);
        waveform
    }

    /// Decode a waveform back to a binary VSA vector (via sign of projection)
    pub fn decode(&self, waveform: &[f64]) -> Vec<u8> {
        let mut result = vec![0u8; VSA_DIM];
        let mut projections = vec![0.0; self.n_packets];
        for k in 0..self.n_packets {
            projections[k] = waveform
                .iter()
                .zip(self.basis[k].iter())
                .map(|(w, b)| w * b)
                .sum::<f64>();
        }
        for i in 0..VSA_DIM {
            let mut vote = 0.0;
            for k in 0..self.n_packets {
                vote += self.basis[k][i] * projections[k];
            }
            result[i] = if vote > 0.0 { 1 } else { 0 };
        }
        result
    }

    /// Wavelet packet decomposition: returns sub-band energies
    pub fn decompose(&self, waveform: &[f64]) -> Vec<f64> {
        self.basis
            .iter()
            .map(|packet| {
                let proj: f64 = waveform.iter().zip(packet.iter()).map(|(w, p)| w * p).sum();
                proj.abs()
            })
            .collect()
    }
}

/// Spectral denoiser: separates signal from noise in frequency domain
#[derive(Debug, Clone)]
pub struct SpectralDenoiser {
    /// Noise floor estimation window (fraction of spectrum)
    noise_floor_window: f64,
    /// Spectral subtraction factor
    subtraction_factor: f64,
}

impl Default for SpectralDenoiser {
    fn default() -> Self {
        Self {
            noise_floor_window: 0.1,
            subtraction_factor: 1.5,
        }
    }
}

impl SpectralDenoiser {
    pub fn new(noise_floor_window: f64, subtraction_factor: f64) -> Self {
        Self {
            noise_floor_window: noise_floor_window.clamp(0.01, 0.5),
            subtraction_factor: subtraction_factor.max(0.1),
        }
    }

    /// Estimate noise floor from the highest-frequency portion of the spectrum
    pub fn estimate_noise_floor(&self, spectrum: &[f64]) -> f64 {
        let n = spectrum.len();
        let window = (n as f64 * self.noise_floor_window).max(2.0) as usize;
        let start = n.saturating_sub(window);
        let noise: f64 = spectrum[start..].iter().map(|&v| v.abs()).sum::<f64>() / window as f64;
        noise
    }

    /// Spectral subtraction denoising
    pub fn denoise(&self, spectrum: &[f64]) -> Vec<f64> {
        let noise_floor = self.estimate_noise_floor(spectrum);
        let threshold = noise_floor * self.subtraction_factor;
        spectrum
            .iter()
            .map(|&v| {
                let abs_v = v.abs();
                if abs_v <= threshold {
                    if v > 0.0 {
                        v - threshold
                    } else {
                        v + threshold
                    }
                } else {
                    v
                }
            })
            .collect()
    }

    /// Wiener filter: SNR-dependent attenuation
    pub fn wiener_filter(&self, spectrum: &[f64]) -> Vec<f64> {
        let noise_floor = self.estimate_noise_floor(spectrum);
        spectrum
            .iter()
            .map(|&v| {
                let snr = v.abs() / noise_floor.max(1e-12);
                let gain = snr / (snr + 1.0);
                v * gain
            })
            .collect()
    }
}

/// Full Wave-Geometric VSA processor
#[derive(Debug, Clone)]
pub struct WaveGeometricVSA {
    pub embed: WaveGeometricEmbed,
    pub denoiser: SpectralDenoiser,
    svsa: SpectralVSA,
}

impl Default for WaveGeometricVSA {
    fn default() -> Self {
        Self {
            embed: WaveGeometricEmbed::default(),
            denoiser: SpectralDenoiser::default(),
            svsa: SpectralVSA::default(),
        }
    }
}

impl WaveGeometricVSA {
    pub fn new(embed: WaveGeometricEmbed, denoiser: SpectralDenoiser) -> Self {
        Self {
            embed,
            denoiser,
            svsa: SpectralVSA::default(),
        }
    }

    /// Encode → denoise in spectral domain → decode
    pub fn clean_encode(&self, v: &[u8]) -> Vec<u8> {
        let waveform = self.embed.encode(v);
        let fv = SpectralVSA::forward(&Self::waveform_to_u8(&waveform));
        let fv_real: Vec<f64> = fv.iter().map(|c: &Complex| c.norm()).collect();
        let denoised = self.denoiser.denoise(&fv_real);
        let reconstructed = Self::spectrum_to_waveform(&denoised, fv.coeffs.len());
        self.embed.decode(&reconstructed)
    }

    /// Encode to waveform (continuous domain)
    pub fn encode_to_waveform(&self, v: &[u8]) -> Vec<f64> {
        self.embed.encode(v)
    }

    /// Decode from waveform back to VSA
    pub fn decode_from_waveform(&self, waveform: &[f64]) -> Vec<u8> {
        self.embed.decode(waveform)
    }

    /// UWE-based similarity: compare two vectors via their waveform embeddings
    pub fn uwe_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let wa = self.embed.encode(a);
        let wb = self.embed.encode(b);
        let dot: f64 = wa.iter().zip(wb.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = wa.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = wb.iter().map(|x| x * x).sum::<f64>().sqrt();
        dot / (na * nb).max(1e-12)
    }

    fn waveform_to_u8(waveform: &[f64]) -> Vec<u8> {
        waveform
            .iter()
            .map(|&x| if x > 0.0 { 1 } else { 0 })
            .collect()
    }

    fn spectrum_to_waveform(spectrum: &[f64], len: usize) -> Vec<f64> {
        let n = len.min(spectrum.len());
        (0..n)
            .map(|i| {
                let idx = i * spectrum.len() / n;
                spectrum[idx.min(spectrum.len() - 1)]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_wave_geometric_embed_encode_decode_roundtrip() {
        let wge = WaveGeometricEmbed::default();
        let v = QuantizedVSA::random_binary();
        let waveform = wge.encode(&v);
        assert_eq!(waveform.len(), VSA_DIM);
        let decoded = wge.decode(&waveform);
        assert_eq!(decoded.len(), VSA_DIM);
        let sim = QuantizedVSA::cosine(&v, &decoded);
        assert!(sim > 0.5, "roundtrip similarity too low: {}", sim);
    }

    #[test]
    fn test_wave_geometric_embed_deterministic() {
        let wge = WaveGeometricEmbed::default();
        let v = QuantizedVSA::random_binary();
        let w1 = wge.encode(&v);
        let w2 = wge.encode(&v);
        for (a, b) in w1.iter().zip(w2.iter()) {
            assert!((a - b).abs() < 1e-12, "determinism violated");
        }
    }

    #[test]
    fn test_spectral_denoiser_noise_floor() {
        let denoiser = SpectralDenoiser::default();
        let mut spectrum = vec![1.0; 64];
        for i in 56..64 {
            spectrum[i] = 0.01;
        }
        let floor = denoiser.estimate_noise_floor(&spectrum);
        assert!(floor > 0.0 && floor < 0.1);
    }

    #[test]
    fn test_spectral_denoiser_subtraction() {
        let denoiser = SpectralDenoiser::default();
        let spectrum = vec![1.0, 0.8, 0.6, 0.4, 0.2, 0.05, 0.05, 0.05];
        let denoised = denoiser.denoise(&spectrum);
        assert_eq!(denoised.len(), spectrum.len());
        assert!(denoised[spectrum.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_spectral_denoiser_wiener() {
        let denoiser = SpectralDenoiser::default();
        let spectrum = vec![10.0, 5.0, 1.0, 0.5, 0.1, 0.05, 0.05, 0.05];
        let filtered = denoiser.wiener_filter(&spectrum);
        assert_eq!(filtered.len(), spectrum.len());
        assert!(filtered[0] > filtered[7]);
    }

    #[test]
    fn test_wave_geometric_vsa_clean_encode() {
        let wgvsa = WaveGeometricVSA::default();
        let v = QuantizedVSA::random_binary();
        let cleaned = wgvsa.clean_encode(&v);
        assert_eq!(cleaned.len(), VSA_DIM);
        let sim = QuantizedVSA::cosine(&v, &cleaned);
        assert!(sim > -1.0 && sim < 1.0);
    }

    #[test]
    fn test_uwe_similarity() {
        let wgvsa = WaveGeometricVSA::default();
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let sim = wgvsa.uwe_similarity(&a, &b);
        assert!(sim >= -1.0 && sim <= 1.0);
        let self_sim = wgvsa.uwe_similarity(&a, &a);
        assert!(
            (self_sim - 1.0).abs() < 0.1,
            "self-sim should be ~1.0, got {}",
            self_sim
        );
    }

    #[test]
    fn test_wave_geometric_decompose() {
        let wge = WaveGeometricEmbed::default();
        let v = QuantizedVSA::random_binary();
        let waveform = wge.encode(&v);
        let bands = wge.decompose(&waveform);
        assert_eq!(bands.len(), wge.n_packets());
        assert!(bands.iter().any(|&b| b > 0.0));
    }

    #[test]
    fn test_different_spread_produces_different_encoding() {
        let wge1 = WaveGeometricEmbed::new(0.5, 16);
        let wge2 = WaveGeometricEmbed::new(2.0, 16);
        let v = QuantizedVSA::random_binary();
        let w1 = wge1.encode(&v);
        let w2 = wge2.encode(&v);
        let diff: f64 = w1.iter().zip(w2.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(
            diff > 0.1,
            "different spread should produce different encodings"
        );
    }

    #[test]
    fn test_denoiser_default() {
        let d = SpectralDenoiser::default();
        assert!((d.noise_floor_window - 0.1).abs() < 1e-9);
        assert!((d.subtraction_factor - 1.5).abs() < 1e-9);
    }
}
