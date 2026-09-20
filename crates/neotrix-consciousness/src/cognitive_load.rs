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
    config: CognitiveLoadConfig,
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
            config,
        }
    }

    pub fn record_step(&mut self, load: f64) {
        self.total_steps += 1;
        let clamped = load.clamp(0.0, 1.0);
        let cfg = &self.config;
        self.recent_load.push_back(clamped);
        if self.recent_load.len() > cfg.load_history_size {
            self.recent_load.pop_front();
        }
        self.thinking_budget = (self.thinking_budget - clamped * cfg.step_load_factor
            + cfg.budget_recharge_rate)
            .clamp(0.0, 1.0);
        self.update_mode();
    }

    pub fn record_deep_step(&mut self, load: f64) {
        self.deep_steps += 1;
        self.record_step(load);
    }

    fn update_mode(&mut self) {
        let cfg = &self.config;
        if self.thinking_budget > cfg.deep_mode_budget * 0.5
            && self.average_load() < cfg.deep_mode_load_threshold
        {
            self.mode = ThinkingMode::Deep;
        } else if self.thinking_budget < cfg.fast_mode_budget * 0.5
            || self.average_load() > cfg.fast_mode_load_threshold
        {
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
        self.thinking_budget > self.config.fast_mode_budget && self.mode != ThinkingMode::Fast
    }

    pub fn deep_ratio(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.deep_steps as f64 / self.total_steps as f64
    }

    pub fn reset(&mut self) {
        let cfg = &self.config;
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

    #[test]
    fn test_custom_config() {
        let config = CognitiveLoadConfig {
            load_history_size: 5,
            fast_mode_budget: 0.2,
            deep_mode_budget: 0.6,
            budget_recharge_rate: 0.1,
            step_load_factor: 0.05,
            deep_mode_load_threshold: 0.3,
            fast_mode_load_threshold: 0.6,
        };
        let mut m = CognitiveLoadMonitor::with_config(config);
        assert!((m.thinking_budget() - 0.6).abs() < 1e-9);
        m.record_step(0.1);
        // budget = (0.6 - 0.1 * 0.05 + 0.1).clamp(0, 1) = (0.6 - 0.005 + 0.1) = 0.695
        assert!((m.thinking_budget() - 0.695).abs() < 1e-9);
    }

    #[test]
    fn test_load_clamped() {
        let mut m = CognitiveLoadMonitor::new();
        m.record_step(-0.5);
        assert!((m.average_load()).abs() < 1e-9);
        m.record_step(1.5);
        assert!((m.peak_load() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_history_size_limit() {
        let config = CognitiveLoadConfig {
            load_history_size: 3,
            ..Default::default()
        };
        let mut m = CognitiveLoadMonitor::with_config(config);
        for i in 0..5 {
            m.record_step(i as f64 * 0.1);
        }
        assert_eq!(m.recent_load.len(), 3);
    }

    #[test]
    fn test_average_load_empty() {
        let m = CognitiveLoadMonitor::new();
        assert!((m.average_load() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_peak_load_empty() {
        let m = CognitiveLoadMonitor::new();
        assert!((m.peak_load() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_deep_ratio_zero_steps() {
        let m = CognitiveLoadMonitor::new();
        assert!((m.deep_ratio() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_mode_transitions_low_to_high() {
        let mut m = CognitiveLoadMonitor::new();
        m.record_step(0.1);
        assert_eq!(m.mode(), ThinkingMode::Deep);
        for _ in 0..30 {
            m.record_step(0.9);
        }
        assert_eq!(m.mode(), ThinkingMode::Fast);
    }

    #[test]
    fn test_budget_clamped_to_one() {
        let config = CognitiveLoadConfig {
            budget_recharge_rate: 0.9,
            step_load_factor: 0.0,
            deep_mode_budget: 0.9,
            ..Default::default()
        };
        let mut m = CognitiveLoadMonitor::with_config(config);
        for _ in 0..10 {
            m.record_step(0.0);
        }
        assert!(m.thinking_budget() <= 1.0);
    }

    #[test]
    fn test_budget_clamped_to_zero() {
        let config = CognitiveLoadConfig {
            budget_recharge_rate: 0.0,
            step_load_factor: 0.5,
            deep_mode_budget: 0.5,
            ..Default::default()
        };
        let mut m = CognitiveLoadMonitor::with_config(config);
        for _ in 0..10 {
            m.record_step(1.0);
        }
        assert!(m.thinking_budget() >= 0.0);
    }

    #[test]
    fn test_total_steps_counting() {
        let mut m = CognitiveLoadMonitor::new();
        assert_eq!(m.total_steps, 0);
        m.record_step(0.5);
        m.record_step(0.3);
        assert_eq!(m.total_steps, 2);
    }

    #[test]
    fn test_deep_steps_counting() {
        let mut m = CognitiveLoadMonitor::new();
        m.record_step(0.5);
        m.record_deep_step(0.3);
        m.record_deep_step(0.2);
        assert_eq!(m.total_steps, 3);
        assert_eq!(m.deep_steps, 2);
    }

    #[test]
    fn test_default_config_values() {
        let config = CognitiveLoadConfig::default();
        assert_eq!(config.load_history_size, 10);
        assert!((config.fast_mode_budget - 0.3).abs() < 1e-9);
        assert!((config.deep_mode_budget - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_cannot_do_deep_in_fast_mode() {
        let mut m = CognitiveLoadMonitor::new();
        for _ in 0..20 {
            m.record_step(0.9);
        }
        assert!(!m.can_do_deep_reasoning());
    }

    #[test]
    fn test_balanced_mode_can_do_deep() {
        let mut m = CognitiveLoadMonitor::new();
        for _ in 0..5 {
            m.record_step(0.5);
        }
        if m.mode() == ThinkingMode::Balanced && m.thinking_budget() > 0.3 {
            assert!(m.can_do_deep_reasoning());
        }
    }
}
