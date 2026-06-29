// REVIVED Task 1 — dead_code removed 2026-06-24

use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolverMethod {
    Euler,
    RK4,
}

#[derive(Debug, Clone)]
pub struct ODEConfig {
    pub n: usize,
    pub dt: f64,
    pub tau: f64,
    pub solver: SolverMethod,
    pub noise_std: f64,
    pub recurrent_scale: f64,
}

impl Default for ODEConfig {
    fn default() -> Self {
        ODEConfig {
            n: 128,
            dt: 0.05,
            tau: 1.0,
            solver: SolverMethod::RK4,
            noise_std: 0.01,
            recurrent_scale: 0.5,
        }
    }
}

pub trait OdeDynamicalSystem: Send {
    fn dynamics(&self, y: &[f64], t: f64) -> Vec<f64>;
    fn dim(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct LiquidODE {
    pub config: ODEConfig,
    pub state: Vec<f64>,
    pub time: f64,
    pub weight_matrix: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

impl LiquidODE {
    pub fn new(config: ODEConfig) -> Self {
        let n = config.n;
        let mut rng = rand::thread_rng();
        let mut weights = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    weights[i][j] = rng.gen_range(-0.5..0.5) * config.recurrent_scale;
                }
            }
        }
        let bias: Vec<f64> = (0..n).map(|_| rng.gen_range(-0.1..0.1)).collect();
        let state = vec![0.0; n];
        LiquidODE {
            config,
            state,
            time: 0.0,
            weight_matrix: weights,
            bias,
        }
    }

    pub fn with_weights(config: ODEConfig, weights: Vec<Vec<f64>>, bias: Vec<f64>) -> Self {
        let state = vec![0.0; config.n];
        LiquidODE {
            config,
            state,
            time: 0.0,
            weight_matrix: weights,
            bias,
        }
    }

    pub fn init_state(&mut self, init: &[f64]) {
        let n = self.config.n;
        self.state = init.iter().take(n).map(|&v| v.clamp(0.0, 1.0)).collect();
        while self.state.len() < n {
            self.state.push(0.0);
        }
    }

    pub fn step(&mut self) {
        match self.config.solver {
            SolverMethod::Euler => self.step_euler(),
            SolverMethod::RK4 => self.step_rk4(),
        }
        self.time += self.config.dt;
    }

    fn dynamics_func(&self, y: &[f64]) -> Vec<f64> {
        let n = self.config.n;
        let tau = self.config.tau;
        let mut dy = vec![0.0; n];
        for i in 0..n {
            let mut sum_w = 0.0;
            for j in 0..n {
                sum_w += self.weight_matrix[i][j] * sigmoid(y[j]);
            }
            dy[i] = (-y[i] + sum_w + self.bias[i]) / tau;
        }
        dy
    }

    fn step_euler(&mut self) {
        let dt = self.config.dt;
        let dy = self.dynamics_func(&self.state);
        for i in 0..self.config.n {
            self.state[i] += dt * dy[i];
            self.state[i] = self.state[i].clamp(0.0, 1.0);
        }
        self.add_noise(self.config.noise_std);
    }

    fn step_rk4(&mut self) {
        let dt = self.config.dt;
        let n = self.config.n;
        let y_copy = self.state.clone();

        let k1 = self.dynamics_func(&y_copy);

        let mut y2 = vec![0.0; n];
        for i in 0..n {
            y2[i] = y_copy[i] + 0.5 * dt * k1[i];
        }
        let k2 = self.dynamics_func(&y2);

        let mut y3 = vec![0.0; n];
        for i in 0..n {
            y3[i] = y_copy[i] + 0.5 * dt * k2[i];
        }
        let k3 = self.dynamics_func(&y3);

        let mut y4 = vec![0.0; n];
        for i in 0..n {
            y4[i] = y_copy[i] + dt * k3[i];
        }
        let k4 = self.dynamics_func(&y4);

        for i in 0..n {
            self.state[i] = y_copy[i] + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            self.state[i] = self.state[i].clamp(0.0, 1.0);
        }

        self.add_noise(self.config.noise_std);
    }

    pub fn vsa_readout(&self) -> Vec<u8> {
        self.state
            .iter()
            .map(|&v| if v > 0.5 { 1u8 } else { 0u8 })
            .collect()
    }

    pub fn mean_activation(&self) -> f64 {
        if self.state.is_empty() {
            return 0.0;
        }
        self.state.iter().sum::<f64>() / self.state.len() as f64
    }

    pub fn sync_index(&self) -> f64 {
        let n = self.state.len();
        if n < 2 {
            return 0.0;
        }
        let mean = self.mean_activation();
        let var = self.state.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        if var.abs() < 1e-10 {
            return 0.0;
        }
        let mut pair_cov = 0.0;
        let mut count = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                pair_cov += (self.state[i] - mean) * (self.state[j] - mean);
                count += 1;
            }
        }
        if count == 0 {
            return 0.0;
        }
        let avg_cov = pair_cov / count as f64;
        let r = avg_cov / var;
        r.abs().min(1.0)
    }

    pub fn inject_input(&mut self, input: &[f64]) {
        let n = self.config.n.min(input.len());
        for i in 0..n {
            self.state[i] = (self.state[i] + input[i] * 0.1).clamp(0.0, 1.0);
        }
    }

    pub fn add_noise(&mut self, std: f64) {
        if std <= 0.0 {
            return;
        }
        let mut rng = rand::thread_rng();
        for v in self.state.iter_mut() {
            *v = (*v + rng.gen_range(-std..std)).clamp(0.0, 1.0);
        }
    }

    pub fn run_steps(&mut self, steps: usize, record_every: usize) -> Vec<Vec<f64>> {
        let mut snapshots = Vec::new();
        for step in 0..steps {
            self.step();
            if record_every > 0 && step % record_every == 0 {
                snapshots.push(self.state.clone());
            }
        }
        snapshots
    }

    pub fn run_duration(&mut self, seconds: f64, record_every: usize) -> Vec<Vec<f64>> {
        let steps = (seconds / self.config.dt).ceil() as usize;
        self.run_steps(steps, record_every)
    }

    pub fn compute_energy(&self) -> f64 {
        let n = self.config.n;
        let mut e = 0.0;
        for i in 0..n {
            e += 0.5 * self.state[i].powi(2);
            for j in 0..n {
                if i != j {
                    e -= 0.5
                        * self.weight_matrix[i][j]
                        * sigmoid(self.state[i])
                        * sigmoid(self.state[j]);
                }
            }
            e -= self.bias[i] * sigmoid(self.state[i]);
        }
        e
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Debug, Clone)]
pub struct ODEStateObserver {
    pub activation_history: Vec<Vec<f64>>,
    pub mean_activation: Vec<f64>,
    pub sync_history: Vec<f64>,
    pub energy: Vec<f64>,
    max_len: usize,
}

impl ODEStateObserver {
    pub fn new(max_len: usize) -> Self {
        ODEStateObserver {
            activation_history: Vec::with_capacity(max_len.min(1000)),
            mean_activation: Vec::with_capacity(max_len.min(1000)),
            sync_history: Vec::with_capacity(max_len.min(1000)),
            energy: Vec::with_capacity(max_len.min(1000)),
            max_len,
        }
    }

    pub fn record(&mut self, ode: &LiquidODE) {
        while self.activation_history.len() >= self.max_len {
            self.activation_history.remove(0);
        }
        while self.mean_activation.len() >= self.max_len {
            self.mean_activation.remove(0);
        }
        while self.sync_history.len() >= self.max_len {
            self.sync_history.remove(0);
        }
        while self.energy.len() >= self.max_len {
            self.energy.remove(0);
        }
        self.activation_history.push(ode.state.clone());
        self.mean_activation.push(ode.mean_activation());
        self.sync_history.push(ode.sync_index());
        self.energy.push(ode.compute_energy());
    }

    pub fn mean_activation_trend(&self) -> f64 {
        let n = self.mean_activation.len();
        if n < 2 {
            return 0.0;
        }
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let vals = &self.mean_activation;
        let mean_i = indices.iter().sum::<f64>() / n as f64;
        let mean_v = vals.iter().sum::<f64>() / n as f64;
        let num: f64 = indices
            .iter()
            .zip(vals.iter())
            .map(|(&i, &v)| (i - mean_i) * (v - mean_v))
            .sum();
        let den: f64 = indices.iter().map(|&i| (i - mean_i).powi(2)).sum();
        if den.abs() < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    pub fn entropy_estimate(&self) -> f64 {
        let n = self.activation_history.len();
        if n == 0 {
            return 0.0;
        }
        let latest = &self.activation_history[n - 1];
        let dim = latest.len();
        if dim == 0 {
            return 0.0;
        }
        let mean: f64 = latest.iter().sum::<f64>() / dim as f64;
        let var: f64 = latest.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / dim as f64;
        if var < 1e-10 {
            return 0.0;
        }
        0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * var)
            .ln()
            .max(0.0)
    }

    pub fn energy_profile(&self) -> &[f64] {
        &self.energy
    }

    pub fn current_mean_activation(&self) -> f64 {
        self.mean_activation.last().copied().unwrap_or(0.0)
    }

    pub fn current_sync(&self) -> f64 {
        self.sync_history.last().copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "|{} - {}| < {}", a, b, eps);
    }

    #[test]
    fn test_ode_creation() {
        let config = ODEConfig {
            n: 64,
            ..Default::default()
        };
        let ode = LiquidODE::new(config.clone());
        assert_eq!(ode.state.len(), 64);
        assert_eq!(ode.weight_matrix.len(), 64);
        assert_eq!(ode.bias.len(), 64);
        assert_eq!(ode.config.n, 64);
    }

    #[test]
    fn test_euler_step_no_nan() {
        let config = ODEConfig {
            n: 64,
            solver: SolverMethod::Euler,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 64]);
        for _ in 0..10 {
            ode.step();
            for &v in &ode.state {
                assert!(!v.is_nan(), "state contains NaN after Euler step");
                assert!(!v.is_infinite(), "state contains Inf after Euler step");
            }
        }
    }

    #[test]
    fn test_rk4_step_no_nan() {
        let config = ODEConfig {
            n: 64,
            solver: SolverMethod::RK4,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 64]);
        for _ in 0..10 {
            ode.step();
            for &v in &ode.state {
                assert!(!v.is_nan(), "state contains NaN after RK4 step");
                assert!(!v.is_infinite(), "state contains Inf after RK4 step");
            }
        }
    }

    #[test]
    fn test_vsa_readout_binary() {
        let config = ODEConfig {
            n: 128,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 128]);
        ode.step();
        let readout = ode.vsa_readout();
        assert_eq!(readout.len(), 128);
        for &v in &readout {
            assert!(v == 0 || v == 1, "VSA readout must be binary, got {}", v);
        }
    }

    #[test]
    fn test_vsa_readout_length() {
        let config = ODEConfig {
            n: 64,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.2; 64]);
        ode.step();
        assert_eq!(ode.vsa_readout().len(), 64);
    }

    #[test]
    fn test_inject_input_changes_state() {
        let config = ODEConfig {
            n: 16,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.0; 16]);
        let before = ode.state.clone();
        ode.inject_input(&vec![1.0; 16]);
        let after = ode.state;
        let changed = before
            .iter()
            .zip(after.iter())
            .any(|(a, b)| (a - b).abs() > 1e-6);
        assert!(changed, "input injection should change state");
    }

    #[test]
    fn test_run_steps_correct_count() {
        let config = ODEConfig {
            n: 16,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 16]);
        let snapshots = ode.run_steps(100, 10);
        assert_eq!(snapshots.len(), 10);
    }

    #[test]
    fn test_sync_index_bounds() {
        let config = ODEConfig {
            n: 32,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 32]);
        for _ in 0..5 {
            ode.step();
            let sync = ode.sync_index();
            assert!(sync >= 0.0, "sync index must be >= 0, got {}", sync);
            assert!(sync <= 1.0, "sync index must be <= 1, got {}", sync);
        }
    }

    #[test]
    fn test_euler_vs_rk4_consistent() {
        let config_euler = ODEConfig {
            n: 8,
            dt: 0.001,
            solver: SolverMethod::Euler,
            noise_std: 0.0,
            ..Default::default()
        };
        let config_rk4 = ODEConfig {
            n: 8,
            dt: 0.001,
            solver: SolverMethod::RK4,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode_e = LiquidODE::new(config_euler);
        let mut ode_r = LiquidODE::new(config_rk4);
        ode_e.init_state(&vec![0.5; 8]);
        ode_r.init_state(&vec![0.5; 8]);
        for _ in 0..20 {
            ode_e.step();
            ode_r.step();
        }
        let diff: f64 = ode_e
            .state
            .iter()
            .zip(ode_r.state.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            diff < 1.0,
            "Euler and RK4 should be roughly consistent, diff={}",
            diff
        );
    }

    #[test]
    fn test_mean_activation_bounds() {
        let config = ODEConfig {
            n: 32,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 32]);
        for _ in 0..10 {
            ode.step();
            let m = ode.mean_activation();
            assert!(m >= 0.0, "mean activation >= 0, got {}", m);
            assert!(m <= 1.0, "mean activation <= 1, got {}", m);
        }
    }

    #[test]
    fn test_observer_records_activation() {
        let config = ODEConfig {
            n: 16,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 16]);
        let mut observer = ODEStateObserver::new(10);
        for _ in 0..5 {
            ode.step();
            observer.record(&ode);
        }
        assert_eq!(observer.mean_activation.len(), 5);
        assert_eq!(observer.sync_history.len(), 5);
        assert_eq!(observer.activation_history.len(), 5);
    }

    #[test]
    fn test_observer_trend() {
        let config = ODEConfig {
            n: 8,
            noise_std: 0.0,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.1; 8]);
        let mut observer = ODEStateObserver::new(20);
        for _ in 0..10 {
            ode.inject_input(&vec![0.05; 8]);
            ode.step();
            observer.record(&ode);
        }
        let trend = observer.mean_activation_trend();
        assert!(
            trend >= -1.0 && trend <= 1.0,
            "trend should be bounded, got {}",
            trend
        );
    }

    #[test]
    fn test_entropy_estimate_non_negative() {
        let config = ODEConfig {
            n: 16,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 16]);
        let mut observer = ODEStateObserver::new(10);
        for _ in 0..3 {
            ode.step();
            observer.record(&ode);
        }
        let entropy = observer.entropy_estimate();
        assert!(entropy >= 0.0, "entropy must be >= 0, got {}", entropy);
    }

    #[test]
    fn test_ode_step_deterministic_no_noise() {
        let config = ODEConfig {
            n: 8,
            dt: 0.01,
            solver: SolverMethod::RK4,
            noise_std: 0.0,
            recurrent_scale: 0.0,
            ..Default::default()
        };
        let mut ode1 = LiquidODE::with_weights(config.clone(), vec![vec![0.0; 8]; 8], vec![0.0; 8]);
        let mut ode2 = LiquidODE::with_weights(config, vec![vec![0.0; 8]; 8], vec![0.0; 8]);
        ode1.init_state(&vec![0.5; 8]);
        ode2.init_state(&vec![0.5; 8]);
        for _ in 0..10 {
            ode1.step();
            ode2.step();
        }
        for (a, b) in ode1.state.iter().zip(ode2.state.iter()) {
            assert_approx_eq(*a, *b, 1e-10);
        }
    }

    #[test]
    fn test_energy_finite() {
        let config = ODEConfig {
            n: 16,
            ..Default::default()
        };
        let mut ode = LiquidODE::new(config);
        ode.init_state(&vec![0.5; 16]);
        ode.step();
        let e = ode.compute_energy();
        assert!(
            !e.is_nan() && !e.is_infinite(),
            "energy must be finite, got {}",
            e
        );
    }
}
