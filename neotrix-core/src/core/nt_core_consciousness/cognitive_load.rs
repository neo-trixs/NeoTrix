use std::collections::VecDeque;

const LOAD_HISTORY_SIZE: usize = 10;
const FAST_MODE_BUDGET: f64 = 0.3;
const DEEP_MODE_BUDGET: f64 = 0.8;
const BUDGET_RECHARGE_RATE: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThinkingMode {
    Fast,
    Balanced,
    Deep,
}

impl ThinkingMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingMode::Fast => "fast",
            ThinkingMode::Balanced => "balanced",
            ThinkingMode::Deep => "deep",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveLoadMonitor {
    recent_load: VecDeque<f64>,
    thinking_budget: f64,
    mode: ThinkingMode,
    total_steps: u64,
    deep_steps: u64,
}

impl Default for CognitiveLoadMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveLoadMonitor {
    pub fn new() -> Self {
        Self {
            recent_load: VecDeque::with_capacity(LOAD_HISTORY_SIZE),
            thinking_budget: DEEP_MODE_BUDGET,
            mode: ThinkingMode::Balanced,
            total_steps: 0,
            deep_steps: 0,
        }
    }

    pub fn record_step(&mut self, load: f64) {
        self.total_steps += 1;
        let clamped = load.clamp(0.0, 1.0);
        self.recent_load.push_back(clamped);
        if self.recent_load.len() > LOAD_HISTORY_SIZE {
            self.recent_load.pop_front();
        }
        self.thinking_budget = (self.thinking_budget - clamped * 0.1 + BUDGET_RECHARGE_RATE).clamp(0.0, 1.0);
        self.update_mode();
    }

    pub fn record_deep_step(&mut self, load: f64) {
        self.deep_steps += 1;
        self.record_step(load);
    }

    fn update_mode(&mut self) {
        if self.thinking_budget > DEEP_MODE_BUDGET * 0.5 && self.average_load() < 0.4 {
            self.mode = ThinkingMode::Deep;
        } else if self.thinking_budget < FAST_MODE_BUDGET * 0.5 || self.average_load() > 0.7 {
            self.mode = ThinkingMode::Fast;
        } else {
            self.mode = ThinkingMode::Balanced;
        }
    }

    pub fn mode(&self) -> ThinkingMode {
        self.mode
    }

    pub fn thinking_budget(&self) -> f64 {
        self.thinking_budget
    }

    pub fn average_load(&self) -> f64 {
        if self.recent_load.is_empty() {
            return 0.0;
        }
        self.recent_load.iter().sum::<f64>() / self.recent_load.len() as f64
    }

    pub fn peak_load(&self) -> f64 {
        self.recent_load.iter().cloned().fold(0.0, f64::max)
    }

    pub fn can_do_deep_reasoning(&self) -> bool {
        self.thinking_budget > FAST_MODE_BUDGET && self.mode != ThinkingMode::Fast
    }

    pub fn deep_ratio(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.deep_steps as f64 / self.total_steps as f64
    }

    pub fn reset(&mut self) {
        self.recent_load.clear();
        self.thinking_budget = DEEP_MODE_BUDGET;
        self.mode = ThinkingMode::Balanced;
        self.total_steps = 0;
        self.deep_steps = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_monitor_starts_balanced() {
        let m = CognitiveLoadMonitor::new();
        assert_eq!(m.mode(), ThinkingMode::Balanced);
        assert!((m.thinking_budget() - DEEP_MODE_BUDGET).abs() < 1e-9);
        assert!((m.average_load() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_low_load_allows_deep() {
        let mut m = CognitiveLoadMonitor::new();
        for _ in 0..5 {
            m.record_step(0.1);
        }
        assert!(m.can_do_deep_reasoning());
    }

    #[test]
    fn test_high_load_triggers_fast() {
        let mut m = CognitiveLoadMonitor::new();
        for _ in 0..20 {
            m.record_step(0.9);
        }
        assert_eq!(m.mode(), ThinkingMode::Fast);
    }

    #[test]
    fn test_deep_ratio_tracking() {
        let mut m = CognitiveLoadMonitor::new();
        m.record_step(0.3);
        m.record_deep_step(0.5);
        m.record_step(0.2);
        assert!((m.deep_ratio() - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_peak_load() {
        let mut m = CognitiveLoadMonitor::new();
        m.record_step(0.2);
        m.record_step(0.8);
        m.record_step(0.3);
        assert!((m.peak_load() - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut m = CognitiveLoadMonitor::new();
        for _ in 0..10 {
            m.record_step(0.9);
        }
        m.reset();
        assert_eq!(m.mode(), ThinkingMode::Balanced);
        assert!((m.average_load() - 0.0).abs() < 1e-9);
        assert_eq!(m.total_steps, 0);
    }

    #[test]
    fn test_can_do_deep_when_budget_healthy() {
        let mut m = CognitiveLoadMonitor::new();
        m.thinking_budget = 0.5;
        m.mode = ThinkingMode::Balanced;
        assert!(m.can_do_deep_reasoning());
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(ThinkingMode::Fast.name(), "fast");
        assert_eq!(ThinkingMode::Deep.name(), "deep");
        assert_eq!(ThinkingMode::Balanced.name(), "balanced");
    }
}
