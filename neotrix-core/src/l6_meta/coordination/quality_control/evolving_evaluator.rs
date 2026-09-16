#[derive(Debug, Clone)]
pub struct EvalMetric {
    pub name: String,
    pub baseline: f64,
    pub current: f64,
    pub weight: f64,
}

pub struct EvolvingEvaluator {
    metrics: Vec<EvalMetric>,
    history: Vec<Vec<f64>>,
}

impl EvolvingEvaluator {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            history: Vec::new(),
        }
    }
    pub fn add_metric(&mut self, name: &str, baseline: f64, weight: f64) {
        self.metrics.push(EvalMetric {
            name: name.to_string(),
            baseline,
            current: baseline,
            weight,
        });
    }
    pub fn update(&mut self, name: &str, value: f64) {
        if let Some(m) = self.metrics.iter_mut().find(|m| m.name == name) {
            m.current = value;
        }
    }
    pub fn score(&self) -> f64 {
        let tw: f64 = self.metrics.iter().map(|m| m.weight).sum();
        if tw == 0.0 {
            0.0
        } else {
            self.metrics
                .iter()
                .map(|m| (m.current / m.baseline).min(2.0) * m.weight)
                .sum::<f64>()
                / tw
        }
    }
    pub fn snapshot(&mut self) {
        self.history
            .push(self.metrics.iter().map(|m| m.current).collect());
    }
    pub fn trend(&self, name: &str) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let idx = self
            .metrics
            .iter()
            .position(|m| m.name == name)
            .unwrap_or(0);
        self.history.last().unwrap()[idx] - self.history[self.history.len() - 2][idx]
    }
    pub fn count(&self) -> usize {
        self.metrics.len()
    }
}
impl Default for EvolvingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_score() {
        let mut e = EvolvingEvaluator::new();
        e.add_metric("acc", 100.0, 1.0);
        e.update("acc", 90.0);
        assert!(e.score() > 0.8);
    }
    #[test]
    fn test_trend() {
        let mut e = EvolvingEvaluator::new();
        e.add_metric("a", 100.0, 1.0);
        e.update("a", 80.0);
        e.snapshot();
        e.update("a", 90.0);
        e.snapshot();
        assert!(e.trend("a") > 0.0);
    }
}
