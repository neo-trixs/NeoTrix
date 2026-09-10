

pub struct GpuGrid {
    width: usize,
    height: usize,
    cells: Vec<f32>,
    dirty: bool,
}

impl GpuGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, cells: vec![0.0; width * height], dirty: false }
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = value;
            self.dirty = true;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height { self.cells[y * self.width + x] } else { 0.0 }
    }

    /// CPU fallback: compute 3x3 blur
    pub fn blur_cpu(&mut self) {
        let old = self.cells.clone();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let mut sum = 0.0f32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        sum += old[((y as i32 + dy) as usize) * self.width + ((x as i32 + dx) as usize)];
                    }
                }
                self.cells[y * self.width + x] = sum / 9.0;
            }
        }
        self.dirty = true;
    }

    /// CPU fallback: compute 2D convolution
    pub fn convolve_cpu(&mut self, kernel: &[f32; 9]) {
        let old = self.cells.clone();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let mut sum = 0.0;
                for ky in 0..3 {
                    for kx in 0..3 {
                        sum += old[(y + ky - 1) * self.width + (x + kx - 1)] * kernel[ky * 3 + kx];
                    }
                }
                self.cells[y * self.width + x] = sum;
            }
        }
        self.dirty = true;
    }

    /// CPU fallback: cellular automaton step
    pub fn cellular_step_cpu(&mut self, threshold: f32) {
        let old = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let mut count = 0u32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = ((x as i32 + dx).max(0) as usize) % self.width;
                        let ny = ((y as i32 + dy).max(0) as usize) % self.height;
                        if old[ny * self.width + nx] > threshold { count += 1; }
                    }
                }
                let center = old[y * self.width + x];
                self.cells[y * self.width + x] = if count > 4 || (count == 3 && center > threshold) { 1.0 } else { 0.0 };
            }
        }
        self.dirty = true;
    }

    /// Batch update: set multiple cells at once
    pub fn batch_set(&mut self, updates: &[(usize, usize, f32)]) {
        for &(x, y, v) in updates {
            if x < self.width && y < self.height {
                self.cells[y * self.width + x] = v;
            }
        }
        self.dirty = true;
    }

    /// Get all non-zero cells as (x, y, value)
    pub fn non_zero(&self) -> Vec<(usize, usize, f32)> {
        self.cells.iter().enumerate()
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, &v)| (i % self.width, i / self.width, v))
            .collect()
    }

    /// Compute statistics
    pub fn stats(&self) -> GridStats {
        let sum: f32 = self.cells.iter().sum();
        let count = self.cells.len();
        let mean = sum / count as f32;
        let variance = self.cells.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / count as f32;
        let non_zero = self.cells.iter().filter(|&&v| v > 0.0).count();
        GridStats { mean, variance, non_zero, total: count }
    }

    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn clear_dirty(&mut self) { self.dirty = false; }
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}

#[derive(Debug, Clone)]
pub struct GridStats {
    pub mean: f32,
    pub variance: f32,
    pub non_zero: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_works() {
        let mut g = GpuGrid::new(10, 10);
        g.set(5, 5, 1.0);
        assert_eq!(g.get(5, 5), 1.0);
        assert_eq!(g.get(0, 0), 0.0);
    }

    #[test]
    fn blur_smooths() {
        let mut g = GpuGrid::new(5, 5);
        g.set(2, 2, 9.0);
        g.blur_cpu();
        assert!(g.get(2, 2) < 9.0);
        assert!(g.get(2, 2) > 0.0);
    }

    #[test]
    fn convolve_identity() {
        let mut g = GpuGrid::new(5, 5);
        g.set(2, 2, 5.0);
        let identity = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        g.convolve_cpu(&identity);
        assert!((g.get(2, 2) - 5.0).abs() < 0.01);
    }

    #[test]
    fn batch_set_works() {
        let mut g = GpuGrid::new(10, 10);
        g.batch_set(&[(0, 0, 1.0), (5, 5, 2.0), (9, 9, 3.0)]);
        assert_eq!(g.get(0, 0), 1.0);
        assert_eq!(g.get(5, 5), 2.0);
        assert_eq!(g.get(9, 9), 3.0);
    }

    #[test]
    fn non_zero_filters() {
        let mut g = GpuGrid::new(5, 5);
        g.set(1, 1, 1.0);
        g.set(3, 3, 1.0);
        assert_eq!(g.non_zero().len(), 2);
    }

    #[test]
    fn stats_correct() {
        let mut g = GpuGrid::new(3, 3);
        g.set(0, 0, 1.0);
        g.set(1, 1, 2.0);
        g.set(2, 2, 3.0);
        let s = g.stats();
        assert_eq!(s.non_zero, 3);
        // mean = (1+2+3)/9 = 0.666...
        assert!((s.mean - 6.0/9.0).abs() < 0.01);
    }

    #[test]
    fn dirty_tracking() {
        let mut g = GpuGrid::new(5, 5);
        assert!(!g.is_dirty());
        g.set(0, 0, 1.0);
        assert!(g.is_dirty());
        g.clear_dirty();
        assert!(!g.is_dirty());
    }

    #[test]
    fn out_of_bounds_safe() {
        let mut g = GpuGrid::new(5, 5);
        g.set(10, 10, 1.0);
        assert_eq!(g.get(10, 10), 0.0);
    }
}
