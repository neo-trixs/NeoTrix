/// Spectral diffusion enhancement via DCT-2D + k*(t) cutoff schedule.
/// Provides frequency-domain operations for VSA-based diffusion models.

const PI: f64 = std::f64::consts::PI;

/// 2D DCT-II transform (type II, orthonormal)
pub struct DCT2D;

impl DCT2D {
    pub fn forward(data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }
        let m = data[0].len();
        let mut out = vec![vec![0.0_f64; m]; n];

        // DCT-II rows
        let mut rows = vec![vec![0.0_f64; m]; n];
        for i in 0..n {
            for k in 0..m {
                let mut sum = 0.0;
                for j in 0..m {
                    sum += data[i][j] * ((PI * k as f64 * (j as f64 + 0.5)) / m as f64).cos();
                }
                let ck = if k == 0 {
                    (1.0 / m as f64).sqrt()
                } else {
                    (2.0 / m as f64).sqrt()
                };
                rows[i][k] = ck * sum;
            }
        }

        // DCT-II columns
        for k in 0..m {
            for l in 0..n {
                let mut sum = 0.0;
                for i in 0..n {
                    sum += rows[i][k] * ((PI * l as f64 * (i as f64 + 0.5)) / n as f64).cos();
                }
                let cl = if l == 0 {
                    (1.0 / n as f64).sqrt()
                } else {
                    (2.0 / n as f64).sqrt()
                };
                out[l][k] = cl * sum;
            }
        }
        out
    }

    pub fn inverse(coeff: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = coeff.len();
        if n == 0 {
            return Vec::new();
        }
        let m = coeff[0].len();
        let mut out = vec![vec![0.0_f64; m]; n];

        // IDCT columns first
        let mut cols = vec![vec![0.0_f64; m]; n];
        for k in 0..m {
            for i in 0..n {
                let mut sum = 0.0;
                for l in 0..n {
                    let cl = if l == 0 {
                        (1.0 / n as f64).sqrt()
                    } else {
                        (2.0 / n as f64).sqrt()
                    };
                    sum += cl * coeff[l][k] * ((PI * l as f64 * (i as f64 + 0.5)) / n as f64).cos();
                }
                cols[i][k] = sum;
            }
        }

        // IDCT rows
        for i in 0..n {
            for j in 0..m {
                let mut sum = 0.0;
                for k in 0..m {
                    let ck = if k == 0 {
                        (1.0 / m as f64).sqrt()
                    } else {
                        (2.0 / m as f64).sqrt()
                    };
                    sum += ck * cols[i][k] * ((PI * k as f64 * (j as f64 + 0.5)) / m as f64).cos();
                }
                out[i][j] = sum;
            }
        }
        out
    }
}

/// Spectral forcing schedule with time-dependent cutoff k*(t).
/// Higher t -> higher cutoff -> more high frequencies allowed.
#[derive(Debug, Clone)]
pub struct SpectralForcing {
    pub initial_cutoff: usize,
    pub max_cutoff: usize,
    pub schedule_type: ScheduleType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleType {
    Linear,
    Exponential,
    Cosine,
    Step { steps: Vec<(usize, usize)> },
}

impl SpectralForcing {
    pub fn new(initial_cutoff: usize, max_cutoff: usize) -> Self {
        Self {
            initial_cutoff,
            max_cutoff,
            schedule_type: ScheduleType::Cosine,
        }
    }

    pub fn cutoff_at(&self, t: usize, total_steps: usize) -> usize {
        if total_steps == 0 {
            return self.initial_cutoff;
        }
        let progress = t as f64 / total_steps as f64;
        let range = self.max_cutoff.saturating_sub(self.initial_cutoff) as f64;
        let advance = match self.schedule_type {
            ScheduleType::Linear => range * progress,
            ScheduleType::Exponential => range * (progress * 3.0).exp().min(1.0),
            ScheduleType::Cosine => range * (1.0 - (PI * progress / 2.0).cos()),
            ScheduleType::Step { ref steps } => {
                let mut v = 0.0;
                for (step_t, step_val) in steps {
                    if t >= *step_t {
                        v = *step_val as f64;
                    }
                }
                v - self.initial_cutoff as f64
            }
        };
        let raw = self.initial_cutoff as f64 + advance;
        (raw.round() as usize).clamp(self.initial_cutoff, self.max_cutoff)
    }

    pub fn apply_mask(&self, coeff: &[Vec<f64>], cutoff: usize) -> Vec<Vec<f64>> {
        let n = coeff.len();
        let m = if n > 0 { coeff[0].len() } else { 0 };
        let mut masked = coeff.to_vec();
        for i in 0..n {
            for j in 0..m {
                if i + j > cutoff {
                    masked[i][j] = 0.0;
                }
            }
        }
        masked
    }
}

impl Default for SpectralForcing {
    fn default() -> Self {
        Self {
            initial_cutoff: 4,
            max_cutoff: 32,
            schedule_type: ScheduleType::Cosine,
        }
    }
}

/// Diffusion enhancer — applies spectral forcing to VSA diffusion steps
#[derive(Debug, Clone)]
pub struct DiffusionEnhancer {
    forcing: SpectralForcing,
    total_steps: usize,
}

impl DiffusionEnhancer {
    pub fn new(forcing: SpectralForcing, total_steps: usize) -> Self {
        Self {
            forcing,
            total_steps,
        }
    }

    /// Apply spectral diffusion enhancement to a 2D grid
    pub fn enhance(&self, grid: &[Vec<f64>], step: usize) -> Vec<Vec<f64>> {
        let coeff = DCT2D::forward(grid);
        let k = self.forcing.cutoff_at(step, self.total_steps);
        let masked = self.forcing.apply_mask(&coeff, k);
        DCT2D::inverse(&masked)
    }

    /// VSA-specific: diffuse a 1D VSA vector through spectral domain
    pub fn enhance_vsa(&self, vsa: &[u8], step: usize) -> Vec<u8> {
        let n = vsa.len();
        let size = (n as f64).sqrt().ceil() as usize;
        let mut grid = vec![vec![0.0_f64; size]; size];
        for (idx, &val) in vsa.iter().enumerate() {
            if idx < size * size {
                grid[idx / size][idx % size] = val as f64 / 255.0;
            }
        }
        let enhanced = self.enhance(&grid, step);
        let mut out = Vec::with_capacity(n);
        for (idx, &val) in vsa.iter().enumerate() {
            if idx < size * size {
                let v = (enhanced[idx / size][idx % size] * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                out.push(v);
            } else {
                out.push(val);
            }
        }
        out
    }
}

impl Default for DiffusionEnhancer {
    fn default() -> Self {
        Self {
            forcing: SpectralForcing::default(),
            total_steps: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dct_roundtrip() {
        let data = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 10.0, 11.0, 12.0],
            vec![13.0, 14.0, 15.0, 16.0],
        ];
        let coeff = DCT2D::forward(&data);
        let reconstructed = DCT2D::inverse(&coeff);
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (reconstructed[i][j] - data[i][j]).abs() < 1e-10,
                    "Mismatch at ({i},{j}): expected {}, got {}",
                    data[i][j],
                    reconstructed[i][j]
                );
            }
        }
    }

    #[test]
    fn test_spectral_forcing_cutoff() {
        let sf = SpectralForcing::new(2, 16);
        assert_eq!(sf.cutoff_at(0, 100), 2);
        assert_eq!(sf.cutoff_at(100, 100), 16);
        assert!(sf.cutoff_at(50, 100) > 2);
        assert!(sf.cutoff_at(50, 100) < 16);
    }

    #[test]
    fn test_apply_mask_low_cutoff() {
        let coeff = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let sf = SpectralForcing::new(0, 2);
        let masked = sf.apply_mask(&coeff, 1);
        assert_eq!(masked[0][0], 1.0);
        assert_eq!(masked[0][1], 0.0);
        assert_eq!(masked[1][0], 0.0);
    }

    #[test]
    fn test_diffusion_enhancer_basic() {
        let sf = SpectralForcing::new(2, 32);
        let enhancer = DiffusionEnhancer::new(sf, 50);
        let vsa = vec![128u8; 64];
        let result = enhancer.enhance_vsa(&vsa, 10);
        assert_eq!(result.len(), 64);
    }
}
