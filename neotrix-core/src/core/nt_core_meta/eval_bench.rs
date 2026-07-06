#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BenchmarkMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone)]
pub struct EvalResult {
    pub benchmark_id: String,
    pub passed: bool,
    pub score: f64,
    pub metrics: Vec<BenchmarkMetric>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Benchmark {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub expected_min_score: f64,
    pub version: u32,
}

impl Benchmark {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        expected_min_score: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            description: description.into(),
            expected_min_score: expected_min_score.max(0.0).min(1.0),
            version: 1,
        }
    }

    pub fn evaluate(&self, score: f64, metrics: Vec<BenchmarkMetric>) -> EvalResult {
        let passed = score >= self.expected_min_score;
        EvalResult {
            benchmark_id: self.id.clone(),
            passed,
            score: score.max(0.0).min(1.0),
            metrics,
            errors: Vec::new(),
            duration_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub name: String,
    pub benchmarks: Vec<Benchmark>,
    pub results: Vec<EvalResult>,
}

impl BenchmarkSuite {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            benchmarks: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn add_benchmark(&mut self, benchmark: Benchmark) {
        self.benchmarks.push(benchmark);
    }

    pub fn run_all(&mut self) -> &[EvalResult] {
        self.results.clear();
        for benchmark in &self.benchmarks {
            let result = benchmark.evaluate(0.0, Vec::new());
            self.results.push(result);
        }
        &self.results
    }

    pub fn record_result(&mut self, result: EvalResult) {
        self.results.push(result);
    }

    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count() as f64;
        passed / self.results.len() as f64
    }

    pub fn avg_score(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().map(|r| r.score).sum::<f64>() / self.results.len() as f64
    }

    pub fn category_breakdown(&self) -> HashMap<String, (u32, u32)> {
        let mut breakdown: HashMap<String, (u32, u32)> = HashMap::new();
        for result in &self.results {
            if let Some(bench) = self.benchmarks.iter().find(|b| b.id == result.benchmark_id) {
                let entry = breakdown.entry(bench.category.clone()).or_insert((0, 0));
                entry.0 += 1;
                if result.passed {
                    entry.1 += 1;
                }
            }
        }
        breakdown
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    pub suites: HashMap<String, BenchmarkSuite>,
    pub history: Vec<EvalResult>,
    max_history: usize,
}

impl BenchmarkRunner {
    pub fn new(max_history: usize) -> Self {
        Self {
            suites: HashMap::new(),
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn register_suite(&mut self, suite: BenchmarkSuite) {
        self.suites.insert(suite.name.clone(), suite);
    }

    pub fn get_suite(&self, name: &str) -> Option<&BenchmarkSuite> {
        self.suites.get(name)
    }

    pub fn get_suite_mut(&mut self, name: &str) -> Option<&mut BenchmarkSuite> {
        self.suites.get_mut(name)
    }

    pub fn run_suite(&mut self, name: &str) -> Option<&[EvalResult]> {
        let suite = self.suites.get_mut(name)?;
        let results = suite.run_all().to_vec();
        for r in results {
            if self.history.len() >= self.max_history {
                self.history.remove(0);
            }
            self.history.push(r);
        }
        self.suites.get(name).map(|s| s.results.as_slice())
    }

    pub fn all_time_pass_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let passed = self.history.iter().filter(|r| r.passed).count() as f64;
        passed / self.history.len() as f64
    }

    pub fn trending_score(&self, window: usize) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let recent: Vec<&EvalResult> = self.history.iter().rev().take(window).collect();
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|r| r.score).sum::<f64>() / recent.len() as f64
    }

    pub fn stats(&self) -> RunnerStats {
        RunnerStats {
            suites: self.suites.len() as u32,
            total_benchmarks: self.suites.values().map(|s| s.benchmarks.len() as u32).sum(),
            total_runs: self.history.len() as u32,
            pass_rate: self.all_time_pass_rate(),
            avg_score: if self.history.is_empty() { 0.0 } else { self.history.iter().map(|r| r.score).sum::<f64>() / self.history.len() as f64 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunnerStats {
    pub suites: u32,
    pub total_benchmarks: u32,
    pub total_runs: u32,
    pub pass_rate: f64,
    pub avg_score: f64,
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_evaluate_pass_fail() {
        let bench = Benchmark::new("b1", "accuracy", "nlp", "test", 0.8);
        let result = bench.evaluate(0.9, Vec::new());
        assert!(result.passed);
        let result = bench.evaluate(0.5, Vec::new());
        assert!(!result.passed);
    }

    #[test]
    fn test_suite_pass_rate() {
        let mut suite = BenchmarkSuite::new("test");
        suite.add_benchmark(Benchmark::new("b1", "b1", "cat", "desc", 0.5));
        suite.add_benchmark(Benchmark::new("b2", "b2", "cat", "desc", 0.9));
        suite.record_result(EvalResult {
            benchmark_id: "b1".into(), passed: true, score: 0.8,
            metrics: Vec::new(), errors: Vec::new(), duration_ms: 0,
        });
        suite.record_result(EvalResult {
            benchmark_id: "b2".into(), passed: false, score: 0.3,
            metrics: Vec::new(), errors: Vec::new(), duration_ms: 0,
        });
        assert!((suite.pass_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_runner_tracks_history() {
        let mut runner = BenchmarkRunner::new(100);
        let mut suite = BenchmarkSuite::new("core");
        suite.add_benchmark(Benchmark::new("b1", "b1", "cat", "desc", 0.5));
        runner.register_suite(suite);
        runner.run_suite("core");
        assert!(runner.all_time_pass_rate() >= 0.0);
        let stats = runner.stats();
        assert_eq!(stats.suites, 1);
    }

    #[test]
    fn test_category_breakdown() {
        let mut suite = BenchmarkSuite::new("test");
        suite.add_benchmark(Benchmark::new("b1", "b1", "reasoning", "desc", 0.5));
        suite.add_benchmark(Benchmark::new("b2", "b2", "memory", "desc", 0.5));
        suite.record_result(EvalResult {
            benchmark_id: "b1".into(), passed: true, score: 0.9,
            metrics: Vec::new(), errors: Vec::new(), duration_ms: 0,
        });
        suite.record_result(EvalResult {
            benchmark_id: "b2".into(), passed: false, score: 0.3,
            metrics: Vec::new(), errors: Vec::new(), duration_ms: 0,
        });
        let breakdown = suite.category_breakdown();
        assert!(breakdown.contains_key("reasoning"));
        assert!(breakdown.contains_key("memory"));
    }
}
