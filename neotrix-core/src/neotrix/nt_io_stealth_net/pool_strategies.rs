use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::pool_types::NodeSelectionStrategy;

const MIN_DATA_FOR_ADAPTIVE: u64 = 20;

/// Beta distribution sampler (Thompson Sampling core)
fn sample_beta(alpha: f64, beta: f64) -> f64 {
    let x = rand::thread_rng().gen::<f64>();
    let y = rand::thread_rng().gen::<f64>();
    let ga = sample_gamma(alpha, x);
    let gb = sample_gamma(beta, y);
    if ga + gb < 1e-10 {
        0.5
    } else {
        ga / (ga + gb)
    }
}

/// Standard Gamma distribution sampler (Marsaglia & Tsang approximation)
fn sample_gamma(shape: f64, uniform: f64) -> f64 {
    if shape < 1.0 {
        return sample_gamma(shape + 1.0, uniform) * uniform.powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    d * (1.0 + c * 0.5).powi(3)
}

#[derive(Serialize, Deserialize)]
pub struct StrategyLearner {
    counts: HashMap<String, HashMap<String, (f64, f64)>>,
    epsilon: f64,
    alpha: f64,
    pub record_count: u64,
    #[serde(skip)]
    persist_path: std::path::PathBuf,
}

impl Default for StrategyLearner {
    fn default() -> Self {
        Self {
            counts: HashMap::new(),
            epsilon: 0.3,
            alpha: 1.0,
            record_count: 0,
            persist_path: std::env::home_dir()
                .unwrap_or_default()
                .join(".neotrix")
                .join("strategy_q.json"),
        }
    }
}

impl StrategyLearner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.persist_path = path;
        self
    }

    fn domain_key(host: &str) -> String {
        let host = host.trim_start_matches("www.");
        let parts: Vec<&str> = host.split('.').collect();
        let n = parts.len();
        if n < 2 {
            return parts[0].to_string();
        }
        let maybe_2part = format!("{}.{}", parts[n - 2], parts[n - 1]);
        let two_part_tlds = [
            "co.uk", "com.au", "co.jp", "or.jp", "ne.jp", "ac.uk", "gov.uk", "org.uk", "net.uk",
            "com.br", "org.br",
        ];
        if n >= 3 && two_part_tlds.contains(&maybe_2part.as_str()) {
            parts[n - 3].to_string()
        } else {
            parts[n - 2].to_string()
        }
    }

    fn actions() -> Vec<NodeSelectionStrategy> {
        vec![
            NodeSelectionStrategy::Fastest,
            NodeSelectionStrategy::LeastLatency,
            NodeSelectionStrategy::LeastFailure,
            NodeSelectionStrategy::WeightedRandom,
        ]
    }

    pub fn select_strategy(&self, host: &str) -> NodeSelectionStrategy {
        let dk = Self::domain_key(host);
        let domain_counts = self.counts.get(&dk);
        let eps = self.effective_epsilon();

        if rand::thread_rng().gen::<f64>() < eps {
            let actions = Self::actions();
            let idx = rand::thread_rng().gen_range(0..actions.len());
            return actions[idx].clone();
        }

        let actions = Self::actions();
        let mut best_action = NodeSelectionStrategy::Fastest;
        let mut best_score = f64::NEG_INFINITY;

        for action in &actions {
            let name = action.as_str();
            let score = if let Some(counts) = domain_counts {
                let (alpha, beta) = counts.get(name).copied().unwrap_or((1.0, 1.0));
                sample_beta(alpha, beta)
            } else {
                0.5
            };
            if score > best_score {
                best_score = score;
                best_action = action.clone();
            }
        }
        best_action
    }

    pub fn has_enough_data(&self) -> bool {
        self.record_count >= MIN_DATA_FOR_ADAPTIVE
    }

    pub fn record_reward(&mut self, host: &str, strategy: &NodeSelectionStrategy, success: bool) {
        let dk = Self::domain_key(host);
        let name = strategy.as_str();
        let entry = self.counts.entry(dk).or_default();
        let (alpha, beta) = entry.get(name).copied().unwrap_or((1.0, 1.0));
        if success {
            entry.insert(name.to_string(), (alpha + self.alpha, beta));
        } else {
            entry.insert(name.to_string(), (alpha, beta + self.alpha));
        }
        self.record_count += 1;
        if self.record_count % 10 == 0 {
            let progress = (self.record_count as f64 / 500.0).min(1.0);
            self.epsilon = 0.3 * (1.0 - progress) + 0.05 * progress;
        }
    }

    pub fn effective_epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let parent = self
                .persist_path
                .parent()
                .unwrap_or(std::path::Path::new(""));
            let _ = std::fs::create_dir_all(parent);
            let tmp = self.persist_path.with_extension("tmp");
            let _ = std::fs::write(&tmp, &json);
            let _ = std::fs::rename(&tmp, &self.persist_path);
        } else {
            log::warn!("[strategy-learner] failed to serialize");
        }
    }

    pub fn load(path: &std::path::Path) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(mut learner) = serde_json::from_str::<StrategyLearner>(&content) {
                learner.persist_path = path.to_path_buf();
                return learner;
            }
        }
        Self::default().with_path(path.to_path_buf())
    }

    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn set_epsilon(&mut self, eps: f64) {
        self.epsilon = eps.clamp(0.0, 1.0);
    }

    pub fn domain_stats(&self, host: &str) -> Vec<(String, f64, u64)> {
        let dk = Self::domain_key(host);
        let Some(domain_counts) = self.counts.get(&dk) else {
            return vec![];
        };
        let mut stats: Vec<_> = domain_counts
            .iter()
            .map(|(name, (a, b))| {
                let total = (a + b - 2.0).max(0.0) as u64;
                let rate = a / (a + b);
                (name.clone(), rate, total)
            })
            .collect();
        stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        stats
    }

    pub fn total_entries(&self) -> usize {
        self.counts.values().map(|m| m.len()).sum()
    }

    pub fn domain_count(&self) -> usize {
        self.counts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_key_simple() {
        let k = StrategyLearner::domain_key("example.com");
        assert_eq!(k, "example");
    }

    #[test]
    fn test_domain_key_two_part_tld() {
        let k = StrategyLearner::domain_key("sub.example.co.uk");
        assert_eq!(k, "example");
    }

    #[test]
    fn test_domain_key_www() {
        let k = StrategyLearner::domain_key("www.google.com");
        assert_eq!(k, "google");
    }

    #[test]
    fn test_learner_has_enough_data() {
        let mut learner = StrategyLearner::new();
        assert!(!learner.has_enough_data());
        for _ in 0..25 {
            learner.record_reward("example.com", &NodeSelectionStrategy::Fastest, true);
        }
        assert!(learner.has_enough_data());
    }

    #[test]
    fn test_learner_epsilon_anneals() {
        let mut learner = StrategyLearner::new();
        let initial = learner.effective_epsilon();
        assert!(
            (initial - 0.3).abs() < 0.01,
            "initial epsilon should be ~0.3, got {initial}"
        );
        for _ in 0..100 {
            learner.record_reward("example.com", &NodeSelectionStrategy::Fastest, true);
        }
        let annealed = learner.effective_epsilon();
        assert!(
            annealed <= 0.25,
            "epsilon should anneal <= 0.25, got {annealed}"
        );
        assert!(
            annealed >= 0.05,
            "epsilon should not go below 0.05, got {annealed}"
        );
    }

    #[test]
    fn test_learner_domain_stats() {
        let mut learner = StrategyLearner::new();
        learner.record_reward("example.com", &NodeSelectionStrategy::Fastest, true);
        learner.record_reward("example.com", &NodeSelectionStrategy::Fastest, true);
        learner.record_reward("example.com", &NodeSelectionStrategy::LeastLatency, false);
        let stats = learner.domain_stats("example.com");
        assert!(!stats.is_empty());
        assert_eq!(stats[0].0, "fastest");
        assert!(stats[0].1 > 0.5);
    }

    #[test]
    fn test_learner_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("neotrix_test_q");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_q.json");
        let _ = std::fs::remove_file(&path);

        {
            let mut learner = StrategyLearner::new();
            learner.record_reward("test.com", &NodeSelectionStrategy::Fastest, true);
            learner.persist_path = path.clone();
            learner.save();
        }

        let loaded = StrategyLearner::load(&path);
        assert!(loaded.total_entries() > 0);
        let _ = std::fs::remove_file(&path);
    }
}
