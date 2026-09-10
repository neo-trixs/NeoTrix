// TickSchedule - Multi-timescale execution scheduler
//
// Organizes simulation steps into frequency tiers, inspired by Project Sid/PIANO:
// Reflexes (~50ms), Fast (~250ms), Medium (~1s), Slow (~5s), Background (~25s).

use serde::{Serialize, Deserialize};

/// Execution tier for multi-timescale processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TickTier {
    /// Every tick (~50ms) — reflexes, metabolism, spatial grid
    Reflex,
    /// Every 5 ticks (~250ms) — perception, action execution
    Fast,
    /// Every 20 ticks (~1s) — decision making, social interaction
    Medium,
    /// Every 100 ticks (~5s) — consciousness metrics, reflection
    Slow,
    /// Every 500 ticks (~25s) — evolution, planning, consolidation
    Background,
}

impl TickTier {
    pub fn name(&self) -> &'static str {
        match self {
            TickTier::Reflex => "reflex",
            TickTier::Fast => "fast",
            TickTier::Medium => "medium",
            TickTier::Slow => "slow",
            TickTier::Background => "background",
        }
    }

    pub fn default_interval(&self) -> u64 {
        match self {
            TickTier::Reflex => 1,
            TickTier::Fast => 5,
            TickTier::Medium => 20,
            TickTier::Slow => 100,
            TickTier::Background => 500,
        }
    }
}

/// Multi-timescale tick scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSchedule {
    pub tick: u64,
    pub reflex_interval: u64,
    pub fast_interval: u64,
    pub medium_interval: u64,
    pub slow_interval: u64,
    pub background_interval: u64,
}

impl Default for TickSchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl TickSchedule {
    pub fn new() -> Self {
        Self {
            tick: 0,
            reflex_interval: 1,
            fast_interval: 5,
            medium_interval: 20,
            slow_interval: 100,
            background_interval: 500,
        }
    }

    /// Advance the tick counter by one
    pub fn advance(&mut self) {
        self.tick += 1;
    }

    /// Check if a given tier should execute this tick
    pub fn should_run(&self, tier: TickTier) -> bool {
        let interval = match tier {
            TickTier::Reflex => self.reflex_interval,
            TickTier::Fast => self.fast_interval,
            TickTier::Medium => self.medium_interval,
            TickTier::Slow => self.slow_interval,
            TickTier::Background => self.background_interval,
        };
        self.tick % interval == 0
    }

    /// Get the current tick number
    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    /// Get the interval for a given tier
    pub fn interval_for(&self, tier: TickTier) -> u64 {
        match tier {
            TickTier::Reflex => self.reflex_interval,
            TickTier::Fast => self.fast_interval,
            TickTier::Medium => self.medium_interval,
            TickTier::Slow => self.slow_interval,
            TickTier::Background => self.background_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflex_runs_every_tick() {
        let mut sched = TickSchedule::new();
        for i in 1..=10 {
            sched.advance();
            assert!(sched.should_run(TickTier::Reflex), "Reflex should run on tick {}", i);
        }
    }

    #[test]
    fn fast_runs_every_5_ticks() {
        let mut sched = TickSchedule::new();
        for _ in 0..5 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Fast));

        // tick 6 should not
        sched.advance();
        assert!(!sched.should_run(TickTier::Fast));

        // tick 10 should
        for _ in 0..4 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Fast));
    }

    #[test]
    fn medium_runs_every_20_ticks() {
        let mut sched = TickSchedule::new();
        for _ in 0..20 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Medium));

        sched.advance();
        assert!(!sched.should_run(TickTier::Medium));
    }

    #[test]
    fn slow_runs_every_100_ticks() {
        let mut sched = TickSchedule::new();
        for _ in 0..100 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Slow));

        sched.advance();
        assert!(!sched.should_run(TickTier::Slow));
    }

    #[test]
    fn background_runs_every_500_ticks() {
        let mut sched = TickSchedule::new();
        for _ in 0..500 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Background));

        sched.advance();
        assert!(!sched.should_run(TickTier::Background));
    }

    #[test]
    fn tick_counter_advances_correctly() {
        let mut sched = TickSchedule::new();
        assert_eq!(sched.current_tick(), 0);

        sched.advance();
        assert_eq!(sched.current_tick(), 1);

        for _ in 0..9 {
            sched.advance();
        }
        assert_eq!(sched.current_tick(), 10);
    }

    #[test]
    fn all_tiers_run_on_tick_100() {
        let mut sched = TickSchedule::new();
        for _ in 0..100 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Reflex));
        assert!(sched.should_run(TickTier::Fast));
        assert!(sched.should_run(TickTier::Medium));
        assert!(sched.should_run(TickTier::Slow));
        assert!(!sched.should_run(TickTier::Background));
    }

    #[test]
    fn all_tiers_run_on_tick_500() {
        let mut sched = TickSchedule::new();
        for _ in 0..500 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Reflex));
        assert!(sched.should_run(TickTier::Fast));
        assert!(sched.should_run(TickTier::Medium));
        assert!(sched.should_run(TickTier::Slow));
        assert!(sched.should_run(TickTier::Background));
    }

    #[test]
    fn tier_names() {
        assert_eq!(TickTier::Reflex.name(), "reflex");
        assert_eq!(TickTier::Fast.name(), "fast");
        assert_eq!(TickTier::Medium.name(), "medium");
        assert_eq!(TickTier::Slow.name(), "slow");
        assert_eq!(TickTier::Background.name(), "background");
    }

    #[test]
    fn default_intervals_match() {
        assert_eq!(TickTier::Reflex.default_interval(), 1);
        assert_eq!(TickTier::Fast.default_interval(), 5);
        assert_eq!(TickTier::Medium.default_interval(), 20);
        assert_eq!(TickTier::Slow.default_interval(), 100);
        assert_eq!(TickTier::Background.default_interval(), 500);
    }

    #[test]
    fn custom_intervals() {
        let mut sched = TickSchedule {
            tick: 0,
            reflex_interval: 1,
            fast_interval: 3,
            medium_interval: 10,
            slow_interval: 50,
            background_interval: 200,
        };

        for _ in 0..3 {
            sched.advance();
        }
        assert!(sched.should_run(TickTier::Fast));
        assert!(!sched.should_run(TickTier::Medium));
    }
}
