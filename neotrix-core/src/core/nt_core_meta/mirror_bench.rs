use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct MirrorBenchmark {
    pub num_episodes: usize,
    pub overconfidence_threshold: f64,
    pub underconfidence_threshold: f64,
    pub results: Vec<MirrorEpisode>,
}

#[derive(Debug, Clone)]
pub struct MirrorEpisode {
    pub predicted_performance: f64,
    pub actual_performance: f64,
    pub ece: f64,
    pub overconfident: bool,
    pub underconfident: bool,
}

#[derive(Debug, Clone)]
pub struct MirrorReport {
    pub ece_mean: f64,
    pub ece_std: f64,
    pub overconfidence_rate: f64,
    pub underconfidence_rate: f64,
    pub accuracy: f64,
    pub passes_mirror: bool,
}

impl Default for MirrorBenchmark {
    fn default() -> Self {
        Self {
            num_episodes: 100,
            overconfidence_threshold: 0.8,
            underconfidence_threshold: 0.3,
            results: Vec::new(),
        }
    }
}

impl MirrorBenchmark {
    pub fn new(num_episodes: usize) -> Self {
        Self {
            num_episodes,
            overconfidence_threshold: 0.8,
            underconfidence_threshold: 0.3,
            results: Vec::with_capacity(num_episodes),
        }
    }

    pub fn with_thresholds(mut self, over: f64, under: f64) -> Self {
        self.overconfidence_threshold = over;
        self.underconfidence_threshold = under;
        self
    }

    pub fn run_benchmark(&mut self, metacognitive_fn: impl Fn() -> (f64, f64)) -> MirrorReport {
        self.results.clear();
        for _ in 0..self.num_episodes {
            let (predicted, actual) = metacognitive_fn();
            let ece = (predicted - actual).abs();
            let overconfident = predicted >= self.overconfidence_threshold && actual < 0.8;
            let underconfident = predicted <= self.underconfidence_threshold && actual > 0.7;
            self.results.push(MirrorEpisode {
                predicted_performance: predicted,
                actual_performance: actual,
                ece,
                overconfident,
                underconfident,
            });
        }
        self.report()
    }

    fn report(&self) -> MirrorReport {
        let n = self.results.len();
        if n == 0 {
            return MirrorReport {
                ece_mean: 0.0,
                ece_std: 0.0,
                overconfidence_rate: 0.0,
                underconfidence_rate: 0.0,
                accuracy: 0.0,
                passes_mirror: true,
            };
        }
        let nf = n as f64;
        let ece_mean = self.results.iter().map(|e| e.ece).sum::<f64>() / nf;
        let ece_std = (self
            .results
            .iter()
            .map(|e| (e.ece - ece_mean).powi(2))
            .sum::<f64>()
            / nf)
            .sqrt();
        let over_rate = self.results.iter().filter(|e| e.overconfident).count() as f64 / nf;
        let under_rate = self.results.iter().filter(|e| e.underconfident).count() as f64 / nf;
        let accurate = self
            .results
            .iter()
            .filter(|e| (e.predicted_performance - e.actual_performance).abs() < 0.1)
            .count() as f64
            / nf;
        MirrorReport {
            ece_mean,
            ece_std,
            overconfidence_rate: over_rate,
            underconfidence_rate: under_rate,
            accuracy: accurate,
            passes_mirror: over_rate < 0.2,
        }
    }
}

/// Simulates an overconfident agent: predicts high regardless of actual difficulty.
/// awareness is high (0.8-1.0), pass_rate is pulled down when arch_penalty is high.
pub fn metacognitive_scenario_overconfident() -> (f64, f64) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f64;
    let seed = (t * 1.6180339887).fract();
    let arch_penalty = (seed * 2.0 * PI).sin().abs();
    let awareness = 0.85 + seed * 0.15;
    let pass_rate = 0.9 - arch_penalty * 0.6;
    (awareness, pass_rate.clamp(0.0, 1.0))
}

/// Simulates an underconfident agent: predicts low even when doing well.
pub fn metacognitive_scenario_underconfident() -> (f64, f64) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f64;
    let seed = (t * 2.7182818284).fract();
    let noise = (seed * PI).sin() * 0.3;
    let actual = 0.75 + (seed * 3.0).fract() * 0.2;
    let predicted = (0.3 + noise).clamp(0.0, 0.45);
    (predicted, actual.clamp(0.0, 1.0))
}

/// Default scenario: mixes overconfident and underconfident episodes with a
/// synthetic metacognitive loop returning (awareness, pass_rate).
/// Overconfident when arch_penalty is high but awareness ignores it.
/// Underconfident when arch_penalty is low but awareness reads noise.
pub fn metacognitive_scenario_default() -> (f64, f64) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f64;
    let seed = (t * 0.5772156649).fract();
    let arch_penalty = (seed * 3.0 * PI).sin().abs();
    let noise = (seed * 7.0 * PI).sin() * 0.15;
    let awareness = if arch_penalty > 0.5 {
        0.9 + noise * 0.5
    } else {
        0.3 + noise.abs() * 0.5
    };
    let pass_rate = 0.9 - arch_penalty * 0.5 + noise * 0.1;
    (awareness.clamp(0.0, 1.0), pass_rate.clamp(0.0, 1.0))
}

/// Scaffolded scenario: architecture constraints reduce overconfidence.
/// Returns (awareness, pass_rate) where awareness is constrained to track
/// arch_penalty, mimicking MIRROR's finding that architecture constraints work.
pub fn metacognitive_scenario_scaffolded() -> (f64, f64) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f64;
    let seed = (t * 0.6180339887).fract();
    let arch_penalty = (seed * 3.0 * PI).sin().abs();
    let noise = (seed * 11.0 * PI).sin() * 0.05;
    let awareness = (1.0 - arch_penalty) + noise;
    let pass_rate = 0.9 - arch_penalty * 0.5 + noise * 0.1;
    (awareness.clamp(0.0, 1.0), pass_rate.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_fixed_benchmark(scenario: impl Fn() -> (f64, f64)) -> MirrorReport {
        let mut bench = MirrorBenchmark::new(500);
        bench.run_benchmark(scenario)
    }

    #[test]
    fn default_scenario_produces_overconfidence() {
        let report = run_fixed_benchmark(metacognitive_scenario_default);
        assert!(report.overconfidence_rate > 0.0);
        assert!(report.ece_mean >= 0.0);
    }

    #[test]
    fn overconfident_scenario_high_overconfidence() {
        let report = run_fixed_benchmark(metacognitive_scenario_overconfident);
        assert!(report.overconfidence_rate > 0.15);
    }

    #[test]
    fn underconfident_scenario_high_underconfidence() {
        let report = run_fixed_benchmark(metacognitive_scenario_underconfident);
        assert!(report.underconfidence_rate > 0.15);
    }

    #[test]
    fn scaffolded_scenario_reduces_overconfidence() {
        let report_default = run_fixed_benchmark(metacognitive_scenario_default);
        let report_scaffolded = run_fixed_benchmark(metacognitive_scenario_scaffolded);
        assert!(
            report_scaffolded.overconfidence_rate < report_default.overconfidence_rate,
            "scaffolded ({}) should be lower than default ({})",
            report_scaffolded.overconfidence_rate,
            report_default.overconfidence_rate
        );
    }

    #[test]
    fn report_format_correct() {
        let mut bench = MirrorBenchmark::new(50);
        let report = bench.run_benchmark(|| (0.9, 0.85));
        assert!((report.ece_mean - 0.05).abs() < 1e-10);
        assert!((report.accuracy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn passes_mirror_when_low_overconfidence() {
        let mut bench = MirrorBenchmark::new(100);
        let report = bench.run_benchmark(|| (0.7, 0.72));
        assert!(report.passes_mirror);
        assert!(report.overconfidence_rate < 1e-10);
    }

    #[test]
    fn fails_mirror_when_high_overconfidence() {
        let report = run_fixed_benchmark(metacognitive_scenario_overconfident);
        assert!(report.overconfidence_rate > 0.0);
    }

    #[test]
    fn accuracy_metric() {
        let mut bench = MirrorBenchmark::new(100);
        let report = bench.run_benchmark(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as f64;
            let seed = (t * 0.42).fract();
            (0.5 + seed * 0.4, 0.5 + seed * 0.4)
        });
        assert!(report.accuracy > 0.9);
    }

    #[test]
    fn threshold_configuration() {
        let mut bench = MirrorBenchmark::new(50).with_thresholds(0.6, 0.4);
        let report = bench.run_benchmark(|| (0.95, 0.5));
        assert!(report.overconfidence_rate > 0.0);
    }

    #[test]
    fn zero_episodes_does_not_panic() {
        let mut bench = MirrorBenchmark::new(0);
        let report = bench.run_benchmark(|| (0.5, 0.5));
        assert!((report.ece_mean - 0.0).abs() < 1e-10);
        assert!(report.passes_mirror);
    }
}
