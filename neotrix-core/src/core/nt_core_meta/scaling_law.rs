use std::f64;

const KAPLAN_ALPHA: f64 = 0.34;
const KAPLAN_BETA: f64 = 0.28;
const CHINCHILLA_ALPHA: f64 = 0.46;
const CHINCHILLA_BETA: f64 = 0.54;

#[derive(Debug, Clone, Copy)]
pub struct PowerLaw {
    pub c: f64,
    pub alpha: f64,
    pub b: f64,
}

impl PowerLaw {
    pub fn new(c: f64, alpha: f64, b: f64) -> Self {
        Self { c, alpha, b }
    }

    pub fn evaluate(&self, x: f64) -> f64 {
        self.c * x.powf(-self.alpha) + self.b
    }

    pub fn fit(data: &[(f64, f64)]) -> Self {
        let n = data.len();
        if n < 2 {
            return Self { c: 1.0, alpha: 0.5, b: 0.0 };
        }
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;
        for (x, y) in data {
            let lx = x.ln();
            let ly = y.ln();
            sum_x += lx;
            sum_y += ly;
            sum_xx += lx * lx;
            sum_xy += lx * ly;
        }
        let xm = sum_x / n as f64;
        let ym = sum_y / n as f64;
        let num = sum_xy - n as f64 * xm * ym;
        let den = sum_xx - n as f64 * xm * xm;
        if den.abs() < 1e-15 {
            return Self { c: 1.0, alpha: 0.5, b: 0.0 };
        }
        let alpha = -num / den;
        let log_c = ym + alpha * xm;
        Self { c: log_c.exp(), alpha, b: 0.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChinchillaLaw {
    pub a: f64,
    pub b: f64,
    pub e: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl Default for ChinchillaLaw {
    fn default() -> Self {
        Self::new()
    }
}

impl ChinchillaLaw {
    pub fn new() -> Self {
        Self { a: 406.4, b: 410.7, e: 1.69, alpha: CHINCHILLA_ALPHA, beta: CHINCHILLA_BETA }
    }

    pub fn with_coefficients(a: f64, b: f64, e: f64, alpha: f64, beta: f64) -> Self {
        Self { a, b, e, alpha, beta }
    }

    pub fn loss(&self, n_params: f64, n_data: f64) -> f64 {
        self.e + self.a / n_params.powf(self.alpha) + self.b / n_data.powf(self.beta)
    }

    pub fn optimal_allocation(&self, budget_flops: f64) -> (f64, f64) {
        let alpha = self.alpha;
        let beta = self.beta;
        let k = ((alpha * self.a) / (beta * self.b)).powf(1.0 / (alpha + beta));
        let scale = (budget_flops / 6.0).powf(beta / (alpha + beta));
        let n_opt = k * scale;
        let d_opt = (budget_flops / 6.0) / n_opt;
        (n_opt, d_opt)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KaplanLaw {
    pub a: f64,
    pub b: f64,
    pub e: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl Default for KaplanLaw {
    fn default() -> Self {
        Self::new()
    }
}

impl KaplanLaw {
    pub fn new() -> Self {
        Self { a: 0.15, b: 0.18, e: 0.88, alpha: KAPLAN_ALPHA, beta: KAPLAN_BETA }
    }

    pub fn with_coefficients(a: f64, b: f64, e: f64, alpha: f64, beta: f64) -> Self {
        Self { a, b, e, alpha, beta }
    }

    pub fn loss(&self, n_params: f64, n_data: f64) -> f64 {
        self.e + self.a / n_params.powf(self.alpha) + self.b / n_data.powf(self.beta)
    }
}

#[derive(Debug, Clone)]
pub struct ScaleReport {
    pub n_params: f64,
    pub n_data: f64,
    pub predicted_loss: f64,
    pub predicted_perf: f64,
    pub optimal_params: Option<f64>,
    pub optimal_data: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ScalingLawPredictor {
    pub kaplan: KaplanLaw,
    pub chinchilla: ChinchillaLaw,
}

impl Default for ScalingLawPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScalingLawPredictor {
    pub fn new() -> Self {
        Self { kaplan: KaplanLaw::new(), chinchilla: ChinchillaLaw::new() }
    }

    pub fn with_laws(kaplan: KaplanLaw, chinchilla: ChinchillaLaw) -> Self {
        Self { kaplan, chinchilla }
    }

    pub fn predict_loss_from_params(&self, n_params: f64) -> f64 {
        self.kaplan.loss(n_params, 1e15)
    }

    pub fn predict_loss(&self, n_params: f64, n_data: f64) -> f64 {
        self.kaplan.loss(n_params, n_data)
    }

    pub fn predict_optimal_allocation(&self, budget_flops: f64) -> (f64, f64) {
        self.chinchilla.optimal_allocation(budget_flops)
    }

    pub fn compute_loss_curve(&self, param_range: &[f64], data_range: &[f64]) -> Vec<Vec<f64>> {
        param_range.iter().map(|&n| {
            data_range.iter().map(|&d| self.kaplan.loss(n, d)).collect()
        }).collect()
    }

    pub fn fit_from_observations(&mut self, data: &[(f64, f64, f64)]) {
        let n = data.len();
        if n < 3 {
            return;
        }
        let alpha = self.kaplan.alpha;
        let beta = self.kaplan.beta;

        let mut xtx = [[0.0; 3]; 3];
        let mut xty = [0.0; 3];

        for &(np, nd, loss) in data {
            let x1 = 1.0;
            let x2 = np.powf(-alpha);
            let x3 = nd.powf(-beta);

            xtx[0][0] += x1 * x1;
            xtx[0][1] += x1 * x2;
            xtx[0][2] += x1 * x3;
            xtx[1][0] += x2 * x1;
            xtx[1][1] += x2 * x2;
            xtx[1][2] += x2 * x3;
            xtx[2][0] += x3 * x1;
            xtx[2][1] += x3 * x2;
            xtx[2][2] += x3 * x3;

            xty[0] += x1 * loss;
            xty[1] += x2 * loss;
            xty[2] += x3 * loss;
        }

        if let Some([e, a, b]) = solve_3x3(xtx, xty) {
            self.kaplan.e = e.max(0.0);
            self.kaplan.a = a.max(0.0);
            self.kaplan.b = b.max(0.0);
        }
    }

    pub fn generate_report(&self, n_params: f64, n_data: f64) -> ScaleReport {
        let loss = self.predict_loss(n_params, n_data);
        let perf = (-loss).exp();
        let (opt_n, opt_d) = self.predict_optimal_allocation(6.0 * n_params * n_data);
        ScaleReport {
            n_params,
            n_data,
            predicted_loss: loss,
            predicted_perf: perf,
            optimal_params: Some(opt_n),
            optimal_data: Some(opt_d),
        }
    }

    pub fn kaplan_mut(&mut self) -> &mut KaplanLaw {
        &mut self.kaplan
    }
}

fn solve_3x3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let mut m = [
        [a[0][0], a[0][1], a[0][2], b[0]],
        [a[1][0], a[1][1], a[1][2], b[1]],
        [a[2][0], a[2][1], a[2][2], b[2]],
    ];

    for col in 0..3 {
        let mut max_row = col;
        for row in (col + 1)..3 {
            if m[row][col].abs() > m[max_row][col].abs() {
                max_row = row;
            }
        }
        m.swap(col, max_row);

        if m[col][col].abs() < 1e-15 {
            return None;
        }

        for row in (col + 1)..3 {
            let factor = m[row][col] / m[col][col];
            for c in col..4 {
                m[row][c] -= factor * m[col][c];
            }
        }
    }

    let mut x = [0.0; 3];
    for i in (0..3).rev() {
        x[i] = m[i][3];
        for j in (i + 1)..3 {
            x[i] -= m[i][j] * x[j];
        }
        x[i] /= m[i][i];
    }

    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_law_evaluate() {
        let pl = PowerLaw::new(2.0, 0.5, 1.0);
        let v = pl.evaluate(4.0);
        assert!((v - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_power_law_fit() {
        let data = vec![(1.0, 2.0), (4.0, 1.0), (9.0, 2.0 / 3.0)];
        let pl = PowerLaw::fit(&data);
        assert!((pl.c - 2.0).abs() < 0.5);
        assert!((pl.alpha - 0.5).abs() < 0.3);
    }

    #[test]
    fn test_kaplan_loss() {
        let kl = KaplanLaw::new();
        let loss = kl.loss(1e9, 1e12);
        assert!(loss > 0.0);
        assert!(loss < 10.0);
    }

    #[test]
    fn test_chinchilla_loss() {
        let cl = ChinchillaLaw::new();
        let loss = cl.loss(1e9, 1e12);
        assert!(loss > 0.0);
        assert!(loss < 10.0);
    }

    #[test]
    fn test_predict_loss_from_params() {
        let pred = ScalingLawPredictor::new();
        let loss = pred.predict_loss_from_params(1e9);
        assert!(loss > 0.0);
        assert!(loss < 5.0);
    }

    #[test]
    fn test_optimal_allocation() {
        let cl = ChinchillaLaw::new();
        let budget = 6.0 * 1e9 * 1e12;
        let (n_opt, d_opt) = cl.optimal_allocation(budget);
        assert!(n_opt > 0.0);
        assert!(d_opt > 0.0);
        let product = n_opt * d_opt;
        assert!((product - budget / 6.0).abs() / (budget / 6.0) < 1e-6);
    }

    #[test]
    fn test_compute_loss_curve() {
        let pred = ScalingLawPredictor::new();
        let curve = pred.compute_loss_curve(&[1e8, 1e9], &[1e11, 1e12]);
        assert_eq!(curve.len(), 2);
        assert_eq!(curve[0].len(), 2);
    }

    #[test]
    fn test_fit_from_observations() {
        let mut pred = ScalingLawPredictor::new();
        let data = vec![
            (1e8, 1e11, 1.8),
            (1e9, 1e12, 1.2),
            (1e10, 1e13, 0.95),
        ];
        pred.fit_from_observations(&data);
        assert!(pred.kaplan.e >= 0.0);
        assert!(pred.kaplan.a >= 0.0);
        assert!(pred.kaplan.b >= 0.0);
    }

    #[test]
    fn test_generate_report() {
        let pred = ScalingLawPredictor::new();
        let report = pred.generate_report(1e9, 1e12);
        assert!(report.predicted_loss > 0.0);
        assert!(report.predicted_perf > 0.0);
        assert!(report.optimal_params.unwrap() > 0.0);
    }

    #[test]
    fn test_scale_report_default() {
        let pred = ScalingLawPredictor::new();
        let report = pred.generate_report(7e9, 2e12);
        assert_eq!(report.n_params, 7e9);
        assert_eq!(report.n_data, 2e12);
    }

    #[test]
    fn test_power_law_insufficient_data() {
        let pl = PowerLaw::fit(&[(1.0, 1.0)]);
        assert!((pl.c - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_chinchilla_default() {
        let cl: ChinchillaLaw = Default::default();
        assert!((cl.alpha - CHINCHILLA_ALPHA).abs() < 1e-6);
    }

    #[test]
    fn test_kaplan_default() {
        let kl: KaplanLaw = Default::default();
        assert!((kl.alpha - KAPLAN_ALPHA).abs() < 1e-6);
    }

    #[test]
    fn test_solve_3x3_identity() {
        let a = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let b = [3.0, 5.0, 7.0];
        let x = solve_3x3(a, b).unwrap();
        assert!((x[0] - 3.0).abs() < 1e-10);
        assert!((x[1] - 5.0).abs() < 1e-10);
        assert!((x[2] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_3x3_singular() {
        let a = [[0.0; 3]; 3];
        let b = [1.0, 2.0, 3.0];
        assert!(solve_3x3(a, b).is_none());
    }

    #[test]
    fn test_predict_loss_scales_with_params() {
        let pred = ScalingLawPredictor::new();
        let loss_small = pred.predict_loss_from_params(1e8);
        let loss_large = pred.predict_loss_from_params(1e10);
        assert!(loss_large < loss_small);
    }

    #[test]
    fn test_kaplan_mut() {
        let mut pred = ScalingLawPredictor::new();
        let kl = pred.kaplan_mut();
        kl.a = 1.0;
        assert!((pred.kaplan.a - 1.0).abs() < 1e-10);
    }
}
