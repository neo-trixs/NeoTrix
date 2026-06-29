// REVIVED Task 2 — dead_code removed
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

// === Spectral-domain VSA operations ===
// Bundle = linear superposition in freq domain,
// Bind = circular convolution (pointwise multiply in freq),
// Cleanup = low-pass filter in freq domain.

const ZERO_THRESHOLD: f64 = 1e-12;

#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    pub fn norm(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

/// Spectral VSA engine: forward/backward transforms in frequency domain.
#[derive(Debug, Clone)]
pub struct SpectralVSA {
    /// Pre-computed FFT twiddle factors (placeholder)
    pub dim: usize,
    /// Frequency-domain coefficients
    pub coeffs: Vec<Complex>,
}

impl Default for SpectralVSA {
    fn default() -> Self {
        Self {
            dim: VSA_DIM,
            coeffs: vec![Complex::new(0.0, 0.0); VSA_DIM],
        }
    }
}

impl SpectralVSA {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            coeffs: vec![Complex::new(0.0, 0.0); dim],
        }
    }

    /// Forward transform: map byte data to frequency-domain coefficients.
    pub fn forward(data: &[u8]) -> Self {
        let n = data.len().max(2);
        let scale = 1.0 / (n as f64).sqrt();
        let mut coeffs = Vec::with_capacity(n);
        for i in 0..n {
            let val = if i < data.len() { data[i] as f64 } else { 0.0 };
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let re = val * angle.cos() * scale;
            let im = val * angle.sin() * scale;
            coeffs.push(Complex::new(re, im));
        }
        Self { dim: n, coeffs }
    }

    /// Iterate over frequency coefficients.
    pub fn iter(&self) -> impl Iterator<Item = &Complex> {
        self.coeffs.iter()
    }

    /// Convert back to byte values via inverse transform.
    pub fn inverse(&self) -> Vec<u8> {
        let n = self.coeffs.len();
        let scale = (n as f64).sqrt();
        let mut out = Vec::with_capacity(n);
        for c in &self.coeffs {
            let val = (c.re * scale).round().max(0.0).min(255.0) as u8;
            out.push(val);
        }
        out
    }
}
