#![allow(dead_code)]

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

/// A single autonomy test event
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub tick: u64,
    pub action: String,
    pub success: bool,
    pub duration_ms: f64,
}

/// Result of an autonomy test run
#[derive(Debug, Clone)]
pub struct TestRunResult {
    pub total_ticks: u64,
    pub successes: u64,
    pub failures: u64,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
    pub autonomy_level: AutonomyLevel,
    pub competence: f64,
    pub events: Vec<TestEvent>,
}

/// Autonomy test harness — runs N ticks and measures behavior
pub struct AutonomyHarness {
    pub tick_interval_ms: u64,
    pub max_ticks: u64,
    pub competence: f64,
    pub level: AutonomyLevel,
    events: Vec<TestEvent>,
    tick_count: u64,
    start_time: Option<Instant>,
}

impl AutonomyHarness {
    pub fn new(max_ticks: u64) -> Self {
        AutonomyHarness {
            tick_interval_ms: 50,
            max_ticks,
            competence: 0.0,
            level: AutonomyLevel::L1,
            events: Vec::with_capacity(max_ticks as usize),
            tick_count: 0,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn tick(&mut self, action: &str, success: bool, duration_ms: f64) {
        if self.tick_count >= self.max_ticks {
            return;
        }
        self.events.push(TestEvent {
            tick: self.tick_count,
            action: action.into(),
            success,
            duration_ms,
        });
        self.tick_count += 1;
        self.update_competence();
    }

    pub fn run_battery(&mut self, actions: &[(&str, bool, f64)]) -> TestRunResult {
        self.start();
        for (action, success, duration) in actions {
            self.tick(action, *success, *duration);
        }
        self.result()
    }

    pub fn is_running(&self) -> bool {
        self.tick_count < self.max_ticks
    }

    pub fn progress(&self) -> f64 {
        if self.max_ticks == 0 {
            return 1.0;
        }
        self.tick_count as f64 / self.max_ticks as f64
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.map_or(0.0, |t| t.elapsed().as_secs_f64())
    }

    pub fn result(&self) -> TestRunResult {
        let total = self.events.len() as u64;
        let successes = self.events.iter().filter(|e| e.success).count() as u64;
        let failures = total - successes;
        let avg_dur = if total > 0 {
            self.events.iter().map(|e| e.duration_ms).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let success_rate = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };
        TestRunResult {
            total_ticks: total,
            successes,
            failures,
            avg_duration_ms: avg_dur,
            success_rate,
            autonomy_level: self.level,
            competence: self.competence,
            events: self.events.clone(),
        }
    }

    pub fn summary(&self) -> String {
        let r = self.result();
        format!(
            "AutonomyHarness: {}/{}/{} ticks (ok/fail/total), rate={:.1}%, competence={:.2}, level={:?}, elapsed={:.1}s",
            r.successes, r.failures, r.total_ticks, r.success_rate * 100.0, r.competence, r.autonomy_level, self.elapsed_secs(),
        )
    }

    fn update_competence(&mut self) {
        let total = self.events.len() as f64;
        if total < 2.0 {
            return;
        }
        let successes = self.events.iter().filter(|e| e.success).count() as f64;
        self.competence = (successes / total).clamp(0.0, 1.0);
        self.level = if self.competence > 0.9 {
            AutonomyLevel::L7
        } else if self.competence > 0.75 {
            AutonomyLevel::L6
        } else if self.competence > 0.6 {
            AutonomyLevel::L5
        } else if self.competence > 0.45 {
            AutonomyLevel::L4
        } else if self.competence > 0.3 {
            AutonomyLevel::L3
        } else if self.competence > 0.15 {
            AutonomyLevel::L2
        } else {
            AutonomyLevel::L1
        };
    }
}

/// Generate a simulated autonomy run with configurable noise
pub fn simulate_autonomy_run(ticks: u64, competence: f64, tick_ms: u64) -> AutonomyHarness {
    let mut harness = AutonomyHarness::new(ticks);
    harness.tick_interval_ms = tick_ms;
    harness.start();
    for i in 0..ticks {
        let success = rand::random::<f64>() < competence;
        let duration = tick_ms as f64 * (0.5 + rand::random::<f64>());
        harness.tick(&format!("action_{}", i), success, duration);
    }
    harness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_harness_starts_empty() {
        let h = AutonomyHarness::new(100);
        assert_eq!(h.tick_count, 0);
        assert!(h.events.is_empty());
    }

    #[test]
    fn test_tick_increments_counter() {
        let mut h = AutonomyHarness::new(10);
        h.tick("test", true, 5.0);
        assert_eq!(h.tick_count, 1);
        assert_eq!(h.events.len(), 1);
    }

    #[test]
    fn test_max_ticks_enforced() {
        let mut h = AutonomyHarness::new(3);
        for i in 0..10 {
            h.tick(&format!("a{}", i), true, 1.0);
        }
        assert_eq!(h.tick_count, 3);
    }

    #[test]
    fn test_run_battery() {
        let mut h = AutonomyHarness::new(5);
        let actions = vec![("a", true, 1.0), ("b", false, 2.0), ("c", true, 1.5)];
        let result = h.run_battery(&actions);
        assert_eq!(result.total_ticks, 3);
        assert_eq!(result.successes, 2);
        assert_eq!(result.failures, 1);
    }

    #[test]
    fn test_competence_equals_success_rate() {
        let mut h = AutonomyHarness::new(10);
        let actions: Vec<(&str, bool, f64)> = (0..10).map(|i| ("act", true, 1.0)).collect();
        h.run_battery(&actions);
        assert!((h.competence - 1.0).abs() < 0.01);
        assert_eq!(h.level, AutonomyLevel::L7);
    }

    #[test]
    fn test_low_competence_low_level() {
        let mut h = AutonomyHarness::new(20);
        let actions: Vec<(&str, bool, f64)> = (0..20).map(|i| ("act", false, 1.0)).collect();
        h.run_battery(&actions);
        assert_eq!(h.level, AutonomyLevel::L1);
    }

    #[test]
    fn test_summary_contains_info() {
        let mut h = AutonomyHarness::new(100);
        h.tick("test", true, 5.0);
        let s = h.summary();
        assert!(s.contains("AutonomyHarness"));
    }

    #[test]
    fn test_progress() {
        let mut h = AutonomyHarness::new(10);
        assert!((h.progress() - 0.0).abs() < 0.01);
        for _ in 0..5 {
            h.tick("act", true, 1.0);
        }
        assert!((h.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_running() {
        let mut h = AutonomyHarness::new(3);
        assert!(h.is_running());
        for _ in 0..3 {
            h.tick("act", true, 1.0);
        }
        assert!(!h.is_running());
    }

    #[test]
    fn test_simulate_run() {
        let h = simulate_autonomy_run(1000, 0.85, 50);
        assert_eq!(h.tick_count, 1000);
        assert!(h.competence > 0.7);
    }

    #[test]
    fn test_result_struct() {
        let r = TestRunResult {
            total_ticks: 100,
            successes: 80,
            failures: 20,
            avg_duration_ms: 5.0,
            success_rate: 0.8,
            autonomy_level: AutonomyLevel::L5,
            competence: 0.8,
            events: vec![],
        };
        assert_eq!(r.autonomy_level, AutonomyLevel::L5);
    }
}
