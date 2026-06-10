#[allow(dead_code)]
const REVERBERATION_CYCLE_MS: u64 = 500;
const IDLE_THRESHOLD_MS: u64 = 2000;
const MAX_REVERBERATION_STEPS: usize = 20;

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DMNActivity {
    Idle,
    Reverberating,
    Consolidating,
    Exploring,
}

impl DMNActivity {
    pub fn name(&self) -> &'static str {
        match self {
            DMNActivity::Idle => "idle",
            DMNActivity::Reverberating => "reverberating",
            DMNActivity::Consolidating => "consolidating",
            DMNActivity::Exploring => "exploring",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReverberationSample {
    pub timestamp: Instant,
    pub coherence: f64,
    pub novelty: f64,
}

#[derive(Debug, Clone)]
pub struct DefaultModeNetwork {
    pub activity: DMNActivity,
    pub last_external_input: Instant,
    pub reverberation_samples: Vec<ReverberationSample>,
    pub reverberation_step: usize,
    pub idle_since: Option<Instant>,
    pub total_reverberations: u64,
    pub novelty_threshold: f64,
}

impl Default for DefaultModeNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultModeNetwork {
    pub fn new() -> Self {
        Self {
            activity: DMNActivity::Idle,
            last_external_input: Instant::now(),
            reverberation_samples: Vec::with_capacity(MAX_REVERBERATION_STEPS),
            reverberation_step: 0,
            idle_since: None,
            total_reverberations: 0,
            novelty_threshold: 0.3,
        }
    }

    pub fn tick(&mut self, has_external_input: bool) -> DMNActivity {
        if has_external_input {
            self.last_external_input = Instant::now();
            self.idle_since = None;
            self.activity = DMNActivity::Idle;
            self.reverberation_step = 0;
            return DMNActivity::Idle;
        }

        let idle_duration = Instant::now().duration_since(self.last_external_input);
        if idle_duration < Duration::from_millis(IDLE_THRESHOLD_MS) {
            self.activity = DMNActivity::Idle;
            return DMNActivity::Idle;
        }

        if self.idle_since.is_none() {
            self.idle_since = Some(Instant::now());
        }

        if self.reverberation_step < MAX_REVERBERATION_STEPS {
            self.activity = DMNActivity::Reverberating;
            self.reverberation_step += 1;
        } else {
            self.activity = DMNActivity::Consolidating;
        }

        self.activity
    }

    pub fn record_reverberation(&mut self, coherence: f64, novelty: f64) {
        if self.reverberation_samples.len() >= MAX_REVERBERATION_STEPS {
            self.reverberation_samples.remove(0);
        }
        self.reverberation_samples.push(ReverberationSample {
            timestamp: Instant::now(),
            coherence,
            novelty,
        });
        self.total_reverberations += 1;
    }

    pub fn average_reverberation_coherence(&self) -> f64 {
        if self.reverberation_samples.is_empty() {
            return 0.0;
        }
        self.reverberation_samples.iter().map(|s| s.coherence).sum::<f64>()
            / self.reverberation_samples.len() as f64
    }

    pub fn average_novelty(&self) -> f64 {
        if self.reverberation_samples.is_empty() {
            return 0.0;
        }
        self.reverberation_samples.iter().map(|s| s.novelty).sum::<f64>()
            / self.reverberation_samples.len() as f64
    }

    pub fn is_idle(&self) -> bool {
        self.activity == DMNActivity::Idle
    }

    pub fn is_reverberating(&self) -> bool {
        self.activity == DMNActivity::Reverberating
    }

    pub fn reset(&mut self) {
        self.activity = DMNActivity::Idle;
        self.last_external_input = Instant::now();
        self.reverberation_samples.clear();
        self.reverberation_step = 0;
        self.idle_since = None;
    }

    pub fn idle_duration(&self) -> Duration {
        self.idle_since
            .map(|t| Instant::now().duration_since(t))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dmn_starts_idle() {
        let dmn = DefaultModeNetwork::new();
        assert_eq!(dmn.activity, DMNActivity::Idle);
        assert!(dmn.is_idle());
    }

    #[test]
    fn test_external_input_keeps_idle() {
        let mut dmn = DefaultModeNetwork::new();
        let activity = dmn.tick(true);
        assert_eq!(activity, DMNActivity::Idle);
    }

    #[test]
    fn test_no_input_triggers_reverberation() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        let activity = dmn.tick(false);
        assert_eq!(activity, DMNActivity::Reverberating);
    }

    #[test]
    fn test_reverberation_steps_limited() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        for _ in 0..MAX_REVERBERATION_STEPS {
            dmn.tick(false);
        }
        assert_eq!(dmn.activity, DMNActivity::Consolidating);
    }

    #[test]
    fn test_record_reverberation() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.record_reverberation(0.8, 0.3);
        assert_eq!(dmn.reverberation_samples.len(), 1);
        assert_eq!(dmn.total_reverberations, 1);
    }

    #[test]
    fn test_average_coherence() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.record_reverberation(0.8, 0.2);
        dmn.record_reverberation(0.9, 0.3);
        assert!((dmn.average_reverberation_coherence() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut dmn = DefaultModeNetwork::new();
        dmn.last_external_input = Instant::now() - Duration::from_millis(IDLE_THRESHOLD_MS + 100);
        dmn.tick(false);
        dmn.record_reverberation(0.8, 0.3);
        dmn.reset();
        assert!(dmn.is_idle());
        assert_eq!(dmn.reverberation_samples.len(), 0);
    }

    #[test]
    fn test_dmn_activity_names() {
        assert_eq!(DMNActivity::Idle.name(), "idle");
        assert_eq!(DMNActivity::Reverberating.name(), "reverberating");
        assert_eq!(DMNActivity::Exploring.name(), "exploring");
    }
}
