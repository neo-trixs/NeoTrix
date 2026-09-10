// Heightmap - Procedural terrain generation using value noise
// Design: No external noise crate dependency - use simple value noise
// that can be replaced with Perlin/Simplex later. This keeps the
// crate dependency-light while providing real terrain generation.

use serde::{Serialize, Deserialize};

/// Simple 2D value noise (replaceable with Perlin later)
#[derive(Debug, Clone)]
pub struct ValueNoise {
    _seed: u64,
    grid_size: usize,
    values: Vec<Vec<f32>>,
}

impl ValueNoise {
    pub fn new(seed: u64, grid_size: usize) -> Self {
        let mut values = vec![vec![0.0f32; grid_size]; grid_size];
        // Simple LCG random for grid values
        let mut state = seed;
        for i in 0..grid_size {
            for j in 0..grid_size {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                values[i][j] = (state as f32 / u64::MAX as f32) * 2.0 - 1.0;
            }
        }
        Self { _seed: seed, grid_size, values }
    }

    /// Sample noise at (x, y) with bilinear interpolation
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let scale = self.grid_size as f32;
        let fx = x * scale;
        let fy = y * scale;

        let x0 = (fx.floor() as usize) % self.grid_size;
        let y0 = (fy.floor() as usize) % self.grid_size;
        let x1 = (x0 + 1) % self.grid_size;
        let y1 = (y0 + 1) % self.grid_size;

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();

        let v00 = self.values[y0][x0];
        let v10 = self.values[y0][x1];
        let v01 = self.values[y1][x0];
        let v11 = self.values[y1][x1];

        let top = v00 + (v10 - v00) * tx;
        let bottom = v01 + (v11 - v01) * tx;
        top + (bottom - top) * ty
    }

    /// Fractal Brownian Motion - layered noise for natural terrain
    pub fn fbm(&self, x: f32, y: f32, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;

        for _ in 0..octaves {
            value += amplitude * self.sample(x * frequency, y * frequency);
            amplitude *= gain;
            frequency *= lacunarity;
        }

        value
    }
}

/// Heightmap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightmapConfig {
    pub width: usize,
    pub height: usize,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub gain: f32,
    pub sea_level: f32,
    pub mountain_level: f32,
}

impl Default for HeightmapConfig {
    fn default() -> Self {
        Self {
            width: 128,
            height: 128,
            scale: 0.02,
            octaves: 6,
            lacunarity: 2.0,
            gain: 0.5,
            sea_level: 0.0,
            mountain_level: 0.6,
        }
    }
}

/// Generated heightmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heightmap {
    config: HeightmapConfig,
    data: Vec<Vec<f32>>,
}

impl Heightmap {
    pub fn generate(seed: u64, config: HeightmapConfig) -> Self {
        let noise = ValueNoise::new(seed, 64);
        let mut data = vec![vec![0.0f32; config.width]; config.height];

        for y in 0..config.height {
            for x in 0..config.width {
                let nx = x as f32 / config.width as f32;
                let ny = y as f32 / config.height as f32;

                // Base terrain
                let h = noise.fbm(
                    nx * config.scale * config.width as f32,
                    ny * config.scale * config.height as f32,
                    config.octaves,
                    config.lacunarity,
                    config.gain,
                );

                // Edge falloff (island effect)
                let dx = nx * 2.0 - 1.0;
                let dy = ny * 2.0 - 1.0;
                let dist = (dx * dx + dy * dy).sqrt();
                let falloff = 1.0 - (dist * 1.2).min(1.0).powi(2);

                data[y][x] = h * falloff;
            }
        }

        Self { config, data }
    }

    /// Get height at world position
    pub fn height_at(&self, wx: f32, wy: f32) -> f32 {
        let fx = (wx / self.config.scale).clamp(0.0, (self.config.width - 1) as f32);
        let fy = (wy / self.config.scale).clamp(0.0, (self.config.height - 1) as f32);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.config.width - 1);
        let y1 = (y0 + 1).min(self.config.height - 1);

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();

        let top = self.data[y0][x0] + (self.data[y0][x1] - self.data[y0][x0]) * tx;
        let bottom = self.data[y1][x0] + (self.data[y1][x1] - self.data[y1][x0]) * tx;
        top + (bottom - top) * ty
    }

    /// Check if position is water
    pub fn is_water(&self, wx: f32, wy: f32) -> bool {
        self.height_at(wx, wy) < self.config.sea_level
    }

    /// Check if position is mountain
    pub fn is_mountain(&self, wx: f32, wy: f32) -> bool {
        self.height_at(wx, wy) > self.config.mountain_level
    }

    pub fn config(&self) -> &HeightmapConfig {
        &self.config
    }

    pub fn data(&self) -> &[Vec<f32>] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_noise_sample_range() {
        let noise = ValueNoise::new(42, 32);
        for i in 0..100 {
            let x = i as f32 * 0.01;
            let v = noise.sample(x, x);
            assert!((-1.0..=1.0).contains(&v), "noise out of range: {v}");
        }
    }

    #[test]
    fn fbm_bounded() {
        let noise = ValueNoise::new(123, 32);
        let v = noise.fbm(0.5, 0.5, 6, 2.0, 0.5);
        assert!(v.abs() < 4.0, "fbm unexpectedly large: {v}");
    }

    #[test]
    fn heightmap_generate() {
        let hm = Heightmap::generate(42, HeightmapConfig::default());
        assert_eq!(hm.data().len(), 128);
        assert_eq!(hm.data()[0].len(), 128);
    }

    #[test]
    fn heightmap_water_and_mountain() {
        let hm = Heightmap::generate(42, HeightmapConfig::default());
        let mut has_water = false;
        let mut has_land = false;
        for y in 0..128 {
            for x in 0..128 {
                let wx = x as f32 * hm.config().scale;
                let wy = y as f32 * hm.config().scale;
                if hm.is_water(wx, wy) { has_water = true; }
                else { has_land = true; }
            }
        }
        assert!(has_water, "expected some water");
        assert!(has_land, "expected some land");
    }
}
