use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CognitiveLoadConfig {
    pub load_history_size: usize,
    pub fast_mode_budget: f64,
    pub deep_mode_budget: f64,
    pub budget_recharge_rate: f64,
    pub step_load_factor: f64,
    pub deep_mode_load_threshold: f64,
    pub fast_mode_load_threshold: f64,
}

impl Default for CognitiveLoadConfig {
    fn default() -> Self {
        Self {
            load_history_size: 10,
            fast_mode_budget: 0.3,
            deep_mode_budget: 0.8,
            budget_recharge_rate: 0.05,
            step_load_factor: 0.1,
            deep_mode_load_threshold: 0.4,
            fast_mode_load_threshold: 0.7,
        }
    }
}

pub static COGNITIVE_LOAD_CONFIG: std::sync::LazyLock<CognitiveLoadConfig> =
    std::sync::LazyLock::new(CognitiveLoadConfig::default);

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
        Self::with_config(COGNITIVE_LOAD_CONFIG.clone())
    }

    pub fn with_config(config: CognitiveLoadConfig) -> Self {
        Self {
            recent_load: VecDeque::with_capacity(config.load_history_size),
            thinking_budget: config.deep_mode_budget,
            mode: ThinkingMode::Balanced,
            total_steps: 0,
            deep_steps: 0,
        }
    }

    pub fn record_step(&mut self, load: f64) {
        self.total_steps += 1;
        let clamped = load.clamp(0.0, 1.0);
        let cfg = &COGNITIVE_LOAD_CONFIG;
        self.recent_load.push_back(clamped);
        if self.recent_load.len() > cfg.load_history_size {
            self.recent_load.pop_front();
        }
        self.thinking_budget = (self.thinking_budget - clamped * cfg.step_load_factor + cfg.budget_recharge_rate).clamp(0.0, 1.0);
        self.update_mode();
    }

    pub fn record_deep_step(&mut self, load: f64) {
        self.deep_steps += 1;
        self.record_step(load);
    }

    fn update_mode(&mut self) {
        let cfg = &COGNITIVE_LOAD_CONFIG;
        if self.thinking_budget > cfg.deep_mode_budget * 0.5 && self.average_load() < cfg.deep_mode_load_threshold {
            self.mode = ThinkingMode::Deep;
        } else if self.thinking_budget < cfg.fast_mode_budget * 0.5 || self.average_load() > cfg.fast_mode_load_threshold {
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
        self.thinking_budget > COGNITIVE_LOAD_CONFIG.fast_mode_budget && self.mode != ThinkingMode::Fast
    }

    pub fn deep_ratio(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.deep_steps as f64 / self.total_steps as f64
    }

    pub fn reset(&mut self) {
        let cfg = &COGNITIVE_LOAD_CONFIG;
        self.recent_load.clear();
        self.thinking_budget = cfg.deep_mode_budget;
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
        assert!((m.thinking_budget() - COGNITIVE_LOAD_CONFIG.deep_mode_budget).abs() < 1e-9);
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

impl crate::core::nt_core_self_test::SelfTest for CognitiveLoadMonitor {
    fn name(&self) -> &str { "cognitive_load" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let cfg = &COGNITIVE_LOAD_CONFIG;
        if cfg.load_history_size == 0 {
            failures.push("cognitive_load: load_history_size must be > 0".into());
        }
        if cfg.fast_mode_budget >= cfg.deep_mode_budget {
            failures.push("cognitive_load: fast_mode_budget must be < deep_mode_budget".into());
        }
        if self.total_steps < self.deep_steps {
            failures.push("cognitive_load: deep_steps > total_steps".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}
