use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LoadSample {
    pub token_count: usize,
    pub active_modules: usize,
    pub recursion_depth: usize,
    pub elapsed_ms: u64,
}

pub struct CognitiveLoadMonitor {
    samples: VecDeque<LoadSample>,
    max_samples: usize,
    current_tokens: usize,
    current_modules: usize,
    current_depth: usize,
    throttle_threshold: f64,
    overload_count: usize,
}

impl CognitiveLoadMonitor {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(128),
            max_samples: 128,
            current_tokens: 0,
            current_modules: 0,
            current_depth: 0,
            throttle_threshold: 0.8,
            overload_count: 0,
        }
    }

    pub fn begin_cycle(&mut self) {
        self.current_tokens = 0;
        self.current_modules = 0;
        self.current_depth = 0;
    }

    pub fn record_token(&mut self) {
        self.current_tokens += 1;
    }

    pub fn record_module(&mut self) {
        self.current_modules += 1;
    }

    pub fn record_depth(&mut self, depth: usize) {
        if depth > self.current_depth {
            self.current_depth = depth;
        }
    }

    pub fn end_cycle(&mut self, elapsed_ms: u64) {
        let sample = LoadSample {
            token_count: self.current_tokens,
            active_modules: self.current_modules,
            recursion_depth: self.current_depth,
            elapsed_ms,
        };
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        if self.load() > self.throttle_threshold {
            self.overload_count += 1;
        }
    }

    pub fn load(&self) -> f64 {
        if self.samples.len() < 3 {
            return 0.0;
        }
        let recent: Vec<_> = self.samples.iter().rev().take(10).collect();
        let max_tokens = 100_000.0;
        let max_modules = 20.0;
        let max_depth = 10.0;
        let max_time = 30_000.0;
        let avg_token_ratio: f64 = recent.iter().map(|s| s.token_count as f64 / max_tokens).sum::<f64>() / recent.len() as f64;
        let avg_module_ratio: f64 = recent.iter().map(|s| s.active_modules as f64 / max_modules).sum::<f64>() / recent.len() as f64;
        let avg_depth_ratio: f64 = recent.iter().map(|s| s.recursion_depth as f64 / max_depth).sum::<f64>() / recent.len() as f64;
        let avg_time_ratio: f64 = recent.iter().map(|s| s.elapsed_ms as f64 / max_time).sum::<f64>() / recent.len() as f64;
        (avg_token_ratio * 0.3 + avg_module_ratio * 0.3 + avg_depth_ratio * 0.2 + avg_time_ratio * 0.2).clamp(0.0, 1.0)
    }

    pub fn should_throttle(&self) -> bool {
        self.load() > self.throttle_threshold
    }

    pub fn set_throttle_threshold(&mut self, threshold: f64) {
        self.throttle_threshold = threshold.clamp(0.1, 1.0);
    }

    pub fn overload_ratio(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.overload_count as f64 / self.samples.len() as f64
    }

    pub fn current_tokens(&self) -> usize { self.current_tokens }
    pub fn current_modules(&self) -> usize { self.current_modules }
    pub fn current_depth(&self) -> usize { self.current_depth }
}

impl Default for CognitiveLoadMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_monitor_initial() {
        let m = CognitiveLoadMonitor::new();
        assert!((m.load() - 0.0).abs() < 1e-6);
        assert!(!m.should_throttle());
    }

    #[test]
    fn test_load_monitor_accumulates() {
        let mut m = CognitiveLoadMonitor::new();
        for i in 0..15 {
            m.begin_cycle();
            for _ in 0..1000 {
                m.record_token();
            }
            m.record_module();
            m.record_depth(5);
            m.end_cycle(i * 100);
        }
        assert!(m.load() > 0.0);
        assert!(m.current_tokens() > 0);
    }
}
