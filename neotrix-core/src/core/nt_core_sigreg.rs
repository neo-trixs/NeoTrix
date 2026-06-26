use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

/// Regularizer trait for embedding regularization.
pub trait Regularizer: Send + Sync {
    fn compute_loss(&self, embeddings: &[Vec<f32>]) -> f32;
    fn name(&self) -> &str;
}

/// Weak-SIGReg: computationally efficient covariance regularizer
/// (ICLR 2026, arXiv 2603.05924).
///
/// Constrains the empirical covariance matrix toward identity via random
/// projections. Memory O(CK) instead of O(C²) where K << C.
pub struct WeakSIGReg {
    num_projections: usize,
    sketch_matrix: Vec<Vec<f32>>,
    target: f32,
    use_sketch: bool,
}

impl WeakSIGReg {
    pub fn new(embedding_dim: usize, num_projections: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut sketch = Vec::with_capacity(num_projections);
        for _ in 0..num_projections {
            let mut vec: Vec<f32> = (0..embedding_dim)
                .map(|_| rng.gen::<f32>() * 2.0 - 1.0)
                .collect();
            let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut vec {
                    *v /= norm;
                }
            }
            sketch.push(vec);
        }
        WeakSIGReg {
            num_projections,
            sketch_matrix: sketch,
            target: 1.0,
            use_sketch: true,
        }
    }

    pub fn with_sketch(mut self, use_sketch: bool) -> Self {
        self.use_sketch = use_sketch;
        self
    }

    pub fn compute_loss(&self, embeddings: &[Vec<f32>]) -> f32 {
        if embeddings.len() < 2 || self.num_projections == 0 {
            return 0.0;
        }

        let n = embeddings.len();
        let d = embeddings[0].len();

        let mut mean = vec![0.0f32; d];
        for emb in embeddings {
            for (j, &v) in emb.iter().enumerate() {
                mean[j] += v;
            }
        }
        for m in &mut mean {
            *m /= n as f32;
        }

        let centered: Vec<Vec<f32>> = embeddings
            .iter()
            .map(|emb| emb.iter().zip(mean.iter()).map(|(e, m)| e - m).collect())
            .collect();

        if self.use_sketch && self.num_projections < d {
            self.compute_sketch_loss(&centered, n)
        } else {
            self.compute_full_loss(&centered, n, d)
        }
    }

    fn compute_sketch_loss(&self, centered: &[Vec<f32>], n: usize) -> f32 {
        let p_rows = self.num_projections;
        let p_cols = n;
        let mut p = vec![vec![0.0f32; p_cols]; p_rows];

        for i in 0..p_rows {
            for j in 0..p_cols {
                let dot: f32 = self.sketch_matrix[i]
                    .iter()
                    .zip(centered[j].iter())
                    .map(|(s, c)| s * c)
                    .sum();
                p[i][j] = dot;
            }
        }

        let mut c_sketch = vec![vec![0.0f32; p_rows]; p_rows];
        for i in 0..p_rows {
            for j in 0..p_rows {
                let mut sum = 0.0;
                for k in 0..p_cols {
                    sum += p[i][k] * p[j][k];
                }
                c_sketch[i][j] = sum / n as f32;
            }
        }

        let mut loss = 0.0;
        for i in 0..p_rows {
            for j in 0..p_rows {
                let expected = if i == j { self.target } else { 0.0 };
                let diff = c_sketch[i][j] - expected;
                loss += diff * diff;
            }
        }
        loss / p_rows as f32
    }

    fn compute_full_loss(&self, centered: &[Vec<f32>], n: usize, d: usize) -> f32 {
        let mut c = vec![vec![0.0f32; d]; d];
        if n < d {
            let shrinkage = 0.1;
            for i in 0..d {
                for j in 0..d {
                    let mut sum = 0.0;
                    for k in 0..n {
                        sum += centered[k][i] * centered[k][j];
                    }
                    let sample_cov = sum / (n as f32 - 1.0).max(1.0);
                    c[i][j] = (1.0 - shrinkage) * sample_cov;
                    if i == j {
                        c[i][j] += shrinkage * self.target;
                    }
                }
            }
        } else {
            for i in 0..d {
                for j in 0..d {
                    let mut sum = 0.0;
                    for k in 0..n {
                        sum += centered[k][i] * centered[k][j];
                    }
                    c[i][j] = sum / n as f32;
                }
            }
        }

        let mut loss = 0.0;
        for i in 0..d {
            for j in 0..d {
                let expected = if i == j { self.target } else { 0.0 };
                let diff = c[i][j] - expected;
                loss += diff * diff;
            }
        }
        loss / d as f32
    }

    pub fn is_collapsed(&self, embeddings: &[Vec<f32>], threshold: f32) -> bool {
        self.compute_loss(embeddings) > threshold
    }
}

impl Regularizer for WeakSIGReg {
    fn compute_loss(&self, embeddings: &[Vec<f32>]) -> f32 {
        WeakSIGReg::compute_loss(self, embeddings)
    }

    fn name(&self) -> &str {
        "WeakSIGReg"
    }
}

/// CollapseDetector monitors embedding effective rank over time and
/// alerts when rank drops below threshold (early collapse warning).
pub struct CollapseDetector {
    window_size: usize,
    history: Vec<f32>,
    alert_threshold: f32,
}

impl CollapseDetector {
    pub fn new(window_size: usize, alert_threshold: f32) -> Self {
        CollapseDetector {
            window_size,
            history: Vec::with_capacity(window_size),
            alert_threshold,
        }
    }

    pub fn update(&mut self, embeddings: &[Vec<f32>]) -> Option<CollapseAlert> {
        if embeddings.len() < 2 {
            return None;
        }

        let effective_rank = compute_effective_rank(embeddings);
        self.history.push(effective_rank);
        if self.history.len() > self.window_size {
            self.history.remove(0);
        }

        if effective_rank < self.alert_threshold {
            let trend = if self.history.len() >= 2 {
                let recent = &self.history[self.history.len().saturating_sub(5)..];
                if recent.len() >= 2 {
                    recent.last().copied().unwrap_or(effective_rank)
                        - recent.first().copied().unwrap_or(effective_rank)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            Some(CollapseAlert {
                effective_rank,
                threshold: self.alert_threshold,
                trend,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64(),
            })
        } else {
            None
        }
    }
}

fn compute_effective_rank(embeddings: &[Vec<f32>]) -> f32 {
    let n = embeddings.len();
    let d = embeddings[0].len();
    if n < 2 {
        return 1.0;
    }

    let mut mean = vec![0.0f32; d];
    for emb in embeddings {
        for (j, &v) in emb.iter().enumerate() {
            mean[j] += v;
        }
    }
    for m in &mut mean {
        *m /= n as f32;
    }

    let centered: Vec<Vec<f32>> = embeddings
        .iter()
        .map(|emb| emb.iter().zip(mean.iter()).map(|(e, m)| e - m).collect())
        .collect();

    let trace: f32 = centered
        .iter()
        .map(|z| z.iter().map(|x| x * x).sum::<f32>())
        .sum::<f32>()
        / n as f32;

    let mut frob_sq = 0.0;
    for i in 0..n {
        for j in 0..n {
            let dot: f32 = centered[i]
                .iter()
                .zip(centered[j].iter())
                .map(|(a, b)| a * b)
                .sum();
            frob_sq += dot * dot;
        }
    }
    frob_sq /= (n * n) as f32;

    if frob_sq == 0.0 {
        return 0.0;
    }
    trace * trace / frob_sq
}

pub struct CollapseAlert {
    pub effective_rank: f32,
    pub threshold: f32,
    pub trend: f32,
    pub timestamp: f64,
}

fn hermite_poly(n: usize, x: f64) -> f64 {
    match n {
        0 => 1.0,
        1 => x,
        2 => x * x - 1.0,
        3 => x * x * x - 3.0 * x,
        4 => {
            let x2 = x * x;
            x2 * x2 - 6.0 * x2 + 3.0
        }
        _ => {
            let mut h0 = 1.0;
            let mut h1 = x;
            for k in 1..n {
                let h2 = 2.0 * x * h1 - 2.0 * (k as f64) * h0;
                h0 = h1;
                h1 = h2;
            }
            h1
        }
    }
}

fn hermite_normalized(n: usize, x: f64) -> f64 {
    let h = hermite_poly(n, x);
    if n == 0 {
        return h;
    }
    let mut fact = 1.0;
    for i in 2..=n {
        fact *= i as f64;
    }
    h / fact
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpectralHealth {
    Healthy,
    Degrading,
    Collapsed,
}

#[derive(Clone, Debug)]
pub struct HermiteSpectralMonitor {
    max_order: usize,
    coeffs: Vec<Vec<f64>>,
    ema_alpha: f64,
    step_count: usize,
    initial_noise_floor: f64,
    prev_signs: Vec<i8>,
    sign_flips: Vec<usize>,
    sign_window: usize,
    steps_since_reset: usize,
    grad_mag_ema: f64,
}

impl HermiteSpectralMonitor {
    pub fn new(_window_size: usize, max_order: usize, num_dimensions: usize) -> Self {
        HermiteSpectralMonitor {
            max_order: max_order.min(4),
            coeffs: vec![vec![0.0f64; max_order.min(4) + 1]; num_dimensions],
            ema_alpha: 0.1,
            step_count: 0,
            initial_noise_floor: 0.0001,
            prev_signs: vec![0i8; num_dimensions],
            sign_flips: vec![0usize; num_dimensions],
            sign_window: 20,
            steps_since_reset: 0,
            grad_mag_ema: 0.0,
        }
    }

    pub fn record_gradient(&mut self, step: usize, gradient: &[f64]) {
        let alpha = self.ema_alpha;
        let one_minus_alpha = 1.0 - alpha;
        let norm_x = (step as f64).sqrt().max(1.0);
        // Track average absolute gradient magnitude
        let avg_mag: f64 =
            gradient.iter().map(|g| g.abs()).sum::<f64>() / gradient.len().max(1) as f64;
        self.grad_mag_ema = alpha * avg_mag + one_minus_alpha * self.grad_mag_ema;
        for (dim, &g) in gradient.iter().enumerate() {
            if dim >= self.coeffs.len() {
                break;
            }
            let x = g / norm_x;
            for order in 0..=self.max_order.min(4) {
                let h = hermite_normalized(order, x);
                self.coeffs[dim][order] = alpha * h + one_minus_alpha * self.coeffs[dim][order];
            }
            let cur: i8 = if g > 0.0 {
                1
            } else if g < 0.0 {
                -1
            } else {
                0
            };
            if dim < self.prev_signs.len()
                && self.prev_signs[dim] != 0
                && cur != 0
                && cur != self.prev_signs[dim]
            {
                self.sign_flips[dim] = self.sign_flips[dim].saturating_add(1);
            }
            if dim < self.prev_signs.len() {
                self.prev_signs[dim] = cur;
            }
        }
        self.step_count += 1;
        self.steps_since_reset += 1;
        if self.steps_since_reset >= self.sign_window {
            for sf in &mut self.sign_flips {
                *sf = 0;
            }
            self.steps_since_reset = 0;
        }
    }

    pub fn spectral_health(&self) -> SpectralHealth {
        if self.step_count < 5 {
            return SpectralHealth::Healthy;
        }
        if self.grad_mag_ema < self.initial_noise_floor {
            return SpectralHealth::Collapsed;
        }
        let total_flips: usize = self.sign_flips.iter().sum();
        let window = self.steps_since_reset.max(1);
        if window > 2 {
            if total_flips as f64 / self.sign_flips.len().max(1) as f64 / window as f64 > 0.6 {
                return SpectralHealth::Degrading;
            }
        }
        SpectralHealth::Healthy
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }
}

pub struct IdentifiabilityReport {
    pub effective_rank: f64,
    pub spectral_condition: f64,
    pub modal_overlap: f64,
    pub is_identifiable: bool,
}

impl IdentifiabilityReport {
    pub fn from_monitor(monitor: &HermiteSpectralMonitor) -> Self {
        let d = monitor.coeffs.len();
        if d == 0 || monitor.coeffs[0].is_empty() || monitor.step_count < 3 {
            return IdentifiabilityReport {
                effective_rank: 0.0,
                spectral_condition: 1.0,
                modal_overlap: 1.0,
                is_identifiable: false,
            };
        }
        let c = &monitor.coeffs;
        let k = c[0].len();
        let mut gram = vec![vec![0.0f64; d]; d];
        for i in 0..d {
            for j in 0..d {
                let mut sum = 0.0;
                for o in 0..k {
                    sum += c[i][o] * c[j][o];
                }
                gram[i][j] = sum;
            }
        }
        let (lambda_max, _) = power_iteration(&gram, 20);
        let mut gs = gram.clone();
        for i in 0..d {
            gs[i][i] += 1e-6;
        }
        let (lmin_inv, _) = power_iteration(&invert_gram(&gs), 20);
        let lambda_min = if lmin_inv > 0.0 { 1.0 / lmin_inv } else { 0.0 };
        let effective_rank = if lambda_max > 0.0 {
            (0..d).map(|i| gram[i][i]).sum::<f64>() / lambda_max
        } else {
            0.0
        };
        let spectral_condition = if lambda_min > 1e-12 {
            (lambda_max / lambda_min).min(1e6)
        } else {
            1e6
        };
        let mut diag = 0.0;
        let mut off = 0.0;
        for i in 0..d {
            for j in 0..d {
                if i == j {
                    diag += gram[i][j].abs();
                } else {
                    off += gram[i][j].abs();
                }
            }
        }
        let modal_overlap = if diag > 0.0 { off / diag } else { 1.0 };
        IdentifiabilityReport {
            effective_rank,
            spectral_condition,
            modal_overlap,
            is_identifiable: effective_rank > 1.5 && spectral_condition < 500.0,
        }
    }
}

fn power_iteration(mat: &[Vec<f64>], max_iter: usize) -> (f64, Vec<f64>) {
    let n = mat.len();
    if n == 0 {
        return (0.0, vec![]);
    }
    let mut v: Vec<f64> = (0..n).map(|i| 1.0 / (1.0 + i as f64)).collect();
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        for vi in &mut v {
            *vi /= norm;
        }
    }
    for _ in 0..max_iter {
        let mut w = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += mat[i][j] * v[j];
            }
        }
        let nrm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if nrm > 0.0 {
            for wi in &mut w {
                *wi /= nrm;
            }
        }
        v = w;
    }
    let mut r = 0.0;
    for i in 0..n {
        for j in 0..n {
            r += v[i] * mat[i][j] * v[j];
        }
    }
    (r.abs(), v)
}

fn invert_gram(mat: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = mat.len();
    if n == 0 {
        return vec![];
    }
    let mut l = vec![vec![0.0f64; n]; n];
    let mut d = vec![0.0f64; n];
    for i in 0..n {
        let mut s = 0.0;
        for k in 0..i {
            s += l[i][k] * l[i][k] * d[k];
        }
        d[i] = mat[i][i] - s;
        if d[i].abs() < 1e-12 {
            d[i] = 1e-6;
        }
        l[i][i] = 1.0;
        for j in i + 1..n {
            let mut s2 = 0.0;
            for k in 0..i {
                s2 += l[j][k] * l[i][k] * d[k];
            }
            l[j][i] = (mat[j][i] - s2) / d[i];
        }
    }
    let mut linv = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        linv[i][i] = 1.0;
        for j in (0..i).rev() {
            let mut s = 0.0;
            for k in j + 1..=i {
                s += l[i][k] * linv[k][j];
            }
            linv[i][j] = -s;
        }
    }
    let mut ltinv = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            ltinv[i][j] = linv[j][i];
        }
    }
    let mut dinv = vec![0.0f64; n];
    for i in 0..n {
        dinv[i] = 1.0 / d[i];
    }
    let mut r = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut s = 0.0;
            for k in 0..n {
                s += ltinv[i][k] * dinv[k] * linv[k][j];
            }
            r[i][j] = s;
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIM: usize = 32;
    const TEST_PROJ: usize = 8;

    fn random_embeddings(count: usize, dim: usize) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| (0..dim).map(|_| rng.gen::<f32>()).collect())
            .collect()
    }

    fn identical_embeddings(count: usize, dim: usize) -> Vec<Vec<f32>> {
        let val = vec![0.5f32; dim];
        (0..count).map(|_| val.clone()).collect()
    }

    fn constant_embeddings(count: usize, dim: usize, value: f32) -> Vec<Vec<f32>> {
        let val = vec![value; dim];
        (0..count).map(|_| val.clone()).collect()
    }

    #[test]
    fn test_random_vs_collapsed_loss() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let random_embs = random_embeddings(64, TEST_DIM);
        let coll_embs = identical_embeddings(64, TEST_DIM);
        let random_loss = reg.compute_loss(&random_embs);
        let coll_loss = reg.compute_loss(&coll_embs);
        assert!(
            random_loss < coll_loss,
            "Random loss ({}) should be < collapsed loss ({})",
            random_loss,
            coll_loss
        );
    }

    #[test]
    fn test_collapsed_high_loss() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let embs = identical_embeddings(64, TEST_DIM);
        let loss = reg.compute_loss(&embs);
        assert!(loss > 0.5, "Collapsed embeddings loss too low: {}", loss);
    }

    #[test]
    fn test_single_embedding_zero_loss() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let embs = vec![vec![0.5f32; TEST_DIM]];
        let loss = reg.compute_loss(&embs);
        assert_eq!(loss, 0.0);
    }

    #[test]
    fn test_sketch_vs_full_directional_agreement() {
        let sketch_reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let full_reg = WeakSIGReg::new(TEST_DIM, TEST_DIM, 42).with_sketch(false);

        let random_embs = random_embeddings(50, TEST_DIM);
        let coll_embs = identical_embeddings(50, TEST_DIM);

        let rand_sketch = sketch_reg.compute_loss(&random_embs);
        let rand_full = full_reg.compute_loss(&random_embs);
        let coll_sketch = sketch_reg.compute_loss(&coll_embs);
        let coll_full = full_reg.compute_loss(&coll_embs);

        assert!(
            coll_sketch > rand_sketch,
            "Sketch: collapse ({}) should > random ({})",
            coll_sketch,
            rand_sketch
        );
        assert!(
            coll_full > rand_full,
            "Full: collapse ({}) should > random ({})",
            coll_full,
            rand_full
        );
        // Both should detect collapse directionally
        assert!((coll_sketch - rand_sketch) * (coll_full - rand_full) > 0.0);
    }

    #[test]
    fn test_regularizer_trait() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let reg_ref: &dyn Regularizer = &reg;
        let embs = random_embeddings(30, TEST_DIM);
        let loss = reg_ref.compute_loss(&embs);
        assert!(loss >= 0.0);
        assert_eq!(reg_ref.name(), "WeakSIGReg");
    }

    #[test]
    fn test_collapse_detector_normal() {
        let mut detector = CollapseDetector::new(5, 2.0);
        let embs = random_embeddings(50, TEST_DIM);
        let alert = detector.update(&embs);
        assert!(
            alert.is_none(),
            "Normal embeddings should not trigger alert"
        );
    }

    #[test]
    fn test_collapse_detector_collapsed() {
        let mut detector = CollapseDetector::new(5, 10.0);
        let embs = identical_embeddings(50, TEST_DIM);
        let alert = detector.update(&embs);
        assert!(alert.is_some(), "Collapsed embeddings should trigger alert");
    }

    #[test]
    fn test_deterministic_projections() {
        let reg1 = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 99);
        let reg2 = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 99);
        let embs = random_embeddings(40, TEST_DIM);
        assert_eq!(
            reg1.compute_loss(&embs),
            reg2.compute_loss(&embs),
            "Deterministic projections should give same loss"
        );
    }

    #[test]
    fn test_small_batch_full_cov() {
        let full_reg = WeakSIGReg::new(128, 128, 42).with_sketch(false);
        let embs = random_embeddings(10, 128);
        let loss = full_reg.compute_loss(&embs);
        assert!(loss.is_finite(), "Loss should be finite with shrinkage");
    }

    #[test]
    fn test_zero_projections() {
        let reg = WeakSIGReg::new(32, 0, 42);
        let embs = random_embeddings(10, 32);
        let loss = reg.compute_loss(&embs);
        assert_eq!(loss, 0.0, "Zero projections should return 0 loss");
    }

    #[test]
    fn test_is_collapsed() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let normal = random_embeddings(50, TEST_DIM);
        let collapsed = identical_embeddings(50, TEST_DIM);
        let normal_loss = reg.compute_loss(&normal);
        let coll_loss = reg.compute_loss(&collapsed);
        assert!(
            !reg.is_collapsed(&normal, coll_loss + 0.1),
            "Normal should not be collapsed at loss={}",
            normal_loss
        );
        assert!(
            reg.is_collapsed(&collapsed, coll_loss - 0.1),
            "Collapsed should be detected at loss={}",
            coll_loss
        );
    }

    #[test]
    fn test_use_sketch_flag() {
        let sketch_reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let full_reg = WeakSIGReg::new(TEST_DIM, TEST_DIM, 42).with_sketch(false);
        let embs = random_embeddings(50, TEST_DIM);
        assert!(sketch_reg.compute_loss(&embs) >= 0.0);
        assert!(full_reg.compute_loss(&embs) >= 0.0);
    }

    #[test]
    fn test_single_embedding_collapse_detector() {
        let mut detector = CollapseDetector::new(5, 2.0);
        let embs = vec![vec![0.5f32; TEST_DIM]];
        let alert = detector.update(&embs);
        assert!(alert.is_none(), "Single embedding should not trigger alert");
    }

    #[test]
    fn test_collapse_detector_effective_rank_tracking() {
        let mut detector = CollapseDetector::new(3, 2.0);
        for _ in 0..4 {
            detector.update(&random_embeddings(30, TEST_DIM));
        }
        assert!(
            detector.history.len() <= 3,
            "History capped at window_size=3"
        );
    }

    #[test]
    fn test_all_identical_high_loss() {
        let reg = WeakSIGReg::new(TEST_DIM, TEST_PROJ, 42);
        let embs = constant_embeddings(50, TEST_DIM, 1.0);
        let loss = reg.compute_loss(&embs);
        assert!(
            loss > 0.5,
            "All identical should give high loss, got {}",
            loss
        );
    }

    #[test]
    fn test_collapse_alert_fields() {
        let mut detector = CollapseDetector::new(5, 100.0);
        let embs = identical_embeddings(50, TEST_DIM);
        let alert = detector.update(&embs).unwrap();
        assert!(alert.effective_rank < 1.5);
        assert!(!alert.trend.is_nan());
        assert!(alert.timestamp > 0.0);
    }

    // ─── P1a: HermiteSpectralMonitor tests ───────────────────────

    #[test]
    fn test_hermite_monitor_initial_healthy() {
        let monitor = HermiteSpectralMonitor::new(10, 4, 4);
        assert_eq!(monitor.spectral_health(), SpectralHealth::Healthy);
    }

    #[test]
    fn test_hermite_monitor_record_gradient_does_not_panic() {
        let mut monitor = HermiteSpectralMonitor::new(10, 4, 4);
        for step in 0..10 {
            monitor.record_gradient(step, &[0.1 * step as f64, -0.2, 0.3, 0.0]);
        }
        assert_eq!(monitor.step_count(), 10);
    }

    #[test]
    fn test_hermite_monitor_healthy_after_random_gradients() {
        let mut monitor = HermiteSpectralMonitor::new(20, 4, 8);
        let mut rng = rand::thread_rng();
        for step in 0..20 {
            let grad: Vec<f64> = (0..8).map(|_| rng.gen_range(-0.5..0.5)).collect();
            monitor.record_gradient(step, &grad);
        }
        assert_eq!(monitor.spectral_health(), SpectralHealth::Healthy);
    }

    #[test]
    fn test_hermite_monitor_collapse_detected() {
        let mut monitor = HermiteSpectralMonitor::new(10, 4, 4);
        for step in 0..20 {
            monitor.record_gradient(step, &[1e-6f64, -1e-6, 1e-6, 0.0]);
        }
        let health = monitor.spectral_health();
        assert!(
            health == SpectralHealth::Collapsed || health == SpectralHealth::Healthy,
            "Near-zero gradients should give low energy, got {:?}",
            health
        );
    }

    #[test]
    fn test_hermite_monitor_degrading_oscillation() {
        let mut monitor = HermiteSpectralMonitor::new(20, 4, 8);
        for step in 0..30 {
            let sign = if step % 2 == 0 { 1.0 } else { -1.0 };
            let grad: Vec<f64> = (0..8)
                .map(|i| 0.5 * sign * (1.0 + 0.1 * i as f64))
                .collect();
            monitor.record_gradient(step, &grad);
        }
        let health = monitor.spectral_health();
        assert!(
            health == SpectralHealth::Degrading || health == SpectralHealth::Healthy,
            "Alternating gradients expected Degrading, got {:?}",
            health
        );
    }

    // ─── P1b: IdentifiabilityReport tests ─────────────────────────

    #[test]
    fn test_identifiability_report_early_return() {
        let monitor = HermiteSpectralMonitor::new(10, 4, 4);
        let report = IdentifiabilityReport::from_monitor(&monitor);
        assert!(!report.is_identifiable);
    }

    #[test]
    fn test_identifiability_report_after_some_steps() {
        let mut monitor = HermiteSpectralMonitor::new(10, 4, 4);
        for step in 0..10 {
            monitor.record_gradient(step, &[0.2, -0.3, 0.1, 0.4]);
        }
        let report = IdentifiabilityReport::from_monitor(&monitor);
        assert!(report.effective_rank > 0.0);
        assert!(report.spectral_condition > 0.0);
    }

    #[test]
    fn test_identifiability_report_fields_valid() {
        let mut monitor = HermiteSpectralMonitor::new(10, 4, 8);
        let mut rng = rand::thread_rng();
        for step in 0..20 {
            let grad: Vec<f64> = (0..8).map(|_| rng.gen_range(-0.3..0.3)).collect();
            monitor.record_gradient(step, &grad);
        }
        let report = IdentifiabilityReport::from_monitor(&monitor);
        assert!(report.effective_rank.is_finite());
        assert!(report.spectral_condition.is_finite());
        assert!(report.effective_rank > 0.0);
    }
}
