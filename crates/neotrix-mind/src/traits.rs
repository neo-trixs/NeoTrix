pub trait ToolExecutor: Send + Sync {
    fn web_search(&self, query: &str) -> (String, bool);
    fn web_fetch(&self, url: &str) -> (String, bool);
    fn file_read(&self, path: &str) -> (String, bool);
    fn file_write(&self, path: &str, content: &str) -> (String, bool);
    fn file_edit(&self, path: &str, old: &str, new: &str) -> (String, bool);
    fn bash(&self, cmd: &str) -> (String, bool);
    fn glob(&self, pattern: &str) -> (String, bool);
    fn grep(&self, pattern: &str, path: &str) -> (String, bool);
}

pub trait ConsciousnessHandle {
    fn apply_ne_edit(&mut self, target: &str, value: f64) -> String;
    fn stats_c_score(&self) -> f64;
    fn cognitive_load(&self) -> f64;
    fn self_evolution_best_score(&self) -> f64;
    fn eval_ne_string(&mut self, expr: &str) -> Result<String, String>;
    fn set_self_evolution_archive(&mut self, best_score: f64);
}

pub struct SealResult {
    pub score_before: f64,
    pub score_after: f64,
    pub delta: f64,
    pub iterations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_result_construction() {
        let r = SealResult { score_before: 0.5, score_after: 0.8, delta: 0.3, iterations: 5 };
        assert!((r.delta - 0.3).abs() < 1e-10);
        assert_eq!(r.iterations, 5);
    }

    #[test]
    fn test_seal_result_zero_delta() {
        let r = SealResult { score_before: 1.0, score_after: 1.0, delta: 0.0, iterations: 0 };
        assert!((r.delta).abs() < 1e-10);
    }
}
