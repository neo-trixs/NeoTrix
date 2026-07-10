#![forbid(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::cognitive_engine::ContentItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyDrive {
    pub current_entropy: f64,
    pub target_entropy: f64,
    pub exploration_rate: f64,
    pub entropy_history: VecDeque<f64>,
    max_history: usize,
}

impl EntropyDrive {
    pub fn new(target_entropy: f64) -> Self {
        Self {
            current_entropy: 0.0,
            target_entropy,
            exploration_rate: 0.5,
            entropy_history: VecDeque::with_capacity(20),
            max_history: 20,
        }
    }

    pub fn compute_entropy(items: &[ContentItem]) -> f64 {
        let total: f64 = items.iter().map(|i| i.salience).sum();
        if total == 0.0 {
            return 0.0;
        }
        let entropy: f64 = items
            .iter()
            .map(|i| i.salience / total)
            .map(|p| -p * (p + 1e-10).ln())
            .sum();
        entropy
    }

    pub fn drive_signal(&self) -> f64 {
        let diff = self.target_entropy - self.current_entropy;
        let signal = diff / self.target_entropy.max(1e-10);
        signal.max(0.0).min(1.0)
    }

    pub fn update(&mut self, items: &[ContentItem]) {
        let entropy = Self::compute_entropy(items);
        self.entropy_history.push_back(entropy);
        if self.entropy_history.len() > self.max_history {
            self.entropy_history.pop_front();
        }
        self.current_entropy = entropy;
        self.exploration_rate = self.drive_signal();
    }

    pub fn boost_low_salience(&self, items: &mut [ContentItem]) {
        if items.is_empty() {
            return;
        }
        let mut sorted: Vec<f64> = items.iter().map(|i| i.salience).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];
        let boost = 1.0 + self.drive_signal();
        for item in items.iter_mut() {
            if item.salience < median {
                item.salience = (item.salience * boost).max(0.0).min(1.0);
            }
        }
    }

    pub fn entropy_trend(&self) -> f64 {
        if self.entropy_history.len() < 2 {
            return 0.0;
        }
        let first = self.entropy_history.front().copied().unwrap_or(0.0);
        let last = self.entropy_history.back().copied().unwrap_or(0.0);
        let len = self.entropy_history.len() as f64;
        (last - first) / len
    }

    pub fn reset(&mut self) {
        self.entropy_history.clear();
        self.current_entropy = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(salience: f64) -> ContentItem {
        ContentItem {
            id: 0,
            content: String::new(),
            salience,
            source_agent: 0,
            timestamp: 0,
        }
    }

    #[test]
    fn test_new_initialization() {
        let ed = EntropyDrive::new(0.5);
        assert!((ed.current_entropy - 0.0).abs() < 1e-9);
        assert!((ed.target_entropy - 0.5).abs() < 1e-9);
        assert!((ed.exploration_rate - 0.5).abs() < 1e-9);
        assert!(ed.entropy_history.is_empty());
    }

    #[test]
    fn test_compute_entropy_uniform() {
        let items = vec![make_item(1.0), make_item(1.0), make_item(1.0)];
        let e = EntropyDrive::compute_entropy(&items);
        assert!((e - 1.0986122886681096).abs() < 1e-6);
    }

    #[test]
    fn test_compute_entropy_skewed() {
        let items = vec![make_item(1.0), make_item(0.0), make_item(0.0)];
        let e = EntropyDrive::compute_entropy(&items);
        assert!(e < 0.1);
    }

    #[test]
    fn test_compute_entropy_all_zero() {
        let items = vec![make_item(0.0), make_item(0.0)];
        let e = EntropyDrive::compute_entropy(&items);
        assert!((e - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_drive_signal_low_entropy() {
        let ed = EntropyDrive {
            current_entropy: 0.0,
            target_entropy: 1.0,
            exploration_rate: 0.5,
            entropy_history: VecDeque::new(),
            max_history: 20,
        };
        let signal = ed.drive_signal();
        assert!((signal - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_drive_signal_high_entropy() {
        let ed = EntropyDrive {
            current_entropy: 1.0,
            target_entropy: 0.5,
            exploration_rate: 0.5,
            entropy_history: VecDeque::new(),
            max_history: 20,
        };
        let signal = ed.drive_signal();
        assert!((signal - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_boost_low_salience() {
        let ed = EntropyDrive {
            current_entropy: 0.1,
            target_entropy: 1.0,
            exploration_rate: 0.5,
            entropy_history: VecDeque::new(),
            max_history: 20,
        };
        let mut items = vec![make_item(0.9), make_item(0.1), make_item(0.5)];
        ed.boost_low_salience(&mut items);
        assert!(items[1].salience > 0.1);
    }

    #[test]
    fn test_entropy_trend_detection() {
        let mut ed = EntropyDrive::new(0.5);
        ed.entropy_history.push_back(0.1);
        ed.entropy_history.push_back(0.2);
        ed.entropy_history.push_back(0.3);
        let trend = ed.entropy_trend();
        assert!(trend > 0.0);
    }

    #[test]
    fn test_entropy_trend_insufficient_data() {
        let mut ed = EntropyDrive::new(0.5);
        ed.entropy_history.push_back(0.5);
        let trend = ed.entropy_trend();
        assert!((trend - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_update_cycle() {
        let mut ed = EntropyDrive::new(0.8);
        let items = vec![make_item(0.5), make_item(0.5)];
        ed.update(&items);
        assert!((ed.current_entropy - 0.6931471805599453).abs() < 1e-6);
        assert_eq!(ed.entropy_history.len(), 1);
    }

    #[test]
    fn test_reset() {
        let mut ed = EntropyDrive::new(0.5);
        ed.entropy_history.push_back(0.3);
        ed.entropy_history.push_back(0.7);
        ed.current_entropy = 0.7;
        ed.reset();
        assert!((ed.current_entropy - 0.0).abs() < 1e-9);
        assert!(ed.entropy_history.is_empty());
    }

    #[test]
    fn test_boost_empty_items() {
        let ed = EntropyDrive::new(0.5);
        let mut items: Vec<ContentItem> = vec![];
        ed.boost_low_salience(&mut items);
        assert!(items.is_empty());
    }
}
