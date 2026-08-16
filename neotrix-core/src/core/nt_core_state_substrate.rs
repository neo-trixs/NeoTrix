use std::collections::HashMap;

use crate::core::nt_core_self_test::SelfTest;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThinkingMode {
    Deep,
    Fast,
    Balanced,
}

impl ThinkingMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingMode::Deep => "deep",
            ThinkingMode::Fast => "fast",
            ThinkingMode::Balanced => "balanced",
        }
    }
}

#[derive(Clone)]
pub struct MetricEntry {
    values: Vec<f64>,
    max_len: usize,
}

impl MetricEntry {
    pub fn new(max_len: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_len),
            max_len,
        }
    }

    pub fn record(&mut self, value: f64) {
        if self.values.len() >= self.max_len {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    pub fn latest(&self) -> Option<f64> {
        self.values.last().copied()
    }

    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }
}

pub struct StateSubstrate {
    pub active_mode: ThinkingMode,
    pub free_energy: f64,
    metrics: HashMap<String, MetricEntry>,
    tick_count: u64,
}

impl StateSubstrate {
    pub fn new() -> Self {
        Self {
            active_mode: ThinkingMode::Fast,
            free_energy: 0.5,
            metrics: HashMap::new(),
            tick_count: 0,
        }
    }

    pub fn mode(&self) -> ThinkingMode {
        self.active_mode
    }

    pub fn set_mode(&mut self, mode: ThinkingMode) {
        self.active_mode = mode;
    }

    pub fn metric(&self, name: &str) -> Option<&MetricEntry> {
        self.metrics.get(name)
    }

    pub fn record_metric(&mut self, name: &str, value: f64) {
        let entry = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| MetricEntry::new(100));
        entry.record(value);
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        let load = self
            .metrics
            .get("load")
            .and_then(|m| m.latest())
            .unwrap_or(0.5);
        self.free_energy = (self.free_energy * 0.9 + load * 0.1).max(0.0).min(1.0);
        self.active_mode = if self.free_energy > 0.7 {
            ThinkingMode::Deep
        } else {
            ThinkingMode::Fast
        };
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}

impl Default for StateSubstrate {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for StateSubstrate {
    fn name(&self) -> &'static str {
        "StateSubstrate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.free_energy < 0.0 || self.free_energy > 1.0 {
            failures.push(format!("free_energy out of range: {}", self.free_energy));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}
