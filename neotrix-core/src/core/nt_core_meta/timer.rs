use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimerRegistry {
    timers: Vec<TimerEntry>,
}

#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub name: String,
    pub interval: Duration,
    pub last_fired: Instant,
    pub firing_count: u64,
    pub failures: u32,
    pub max_failures: u32,
    pub paused: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TimerStats {
    pub total_timers: usize,
    pub active_timers: usize,
    pub paused_timers: usize,
    pub total_failures: u32,
}

impl TimerRegistry {
    pub fn new() -> Self {
        Self { timers: Vec::new() }
    }

    pub fn register(&mut self, name: &str, interval: Duration, max_failures: u32) {
        self.timers.push(TimerEntry {
            name: name.to_string(),
            interval,
            last_fired: Instant::now(),
            firing_count: 0,
            failures: 0,
            max_failures,
            paused: false,
        });
    }

    pub fn due(&self) -> Vec<&TimerEntry> {
        let now = Instant::now();
        self.timers
            .iter()
            .filter(|t| !t.paused && now.duration_since(t.last_fired) >= t.interval)
            .collect()
    }

    pub fn tick(&mut self, name: &str) -> Result<(), String> {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.name == name) {
            timer.last_fired = Instant::now();
            timer.firing_count += 1;
            timer.failures = 0;
            Ok(())
        } else {
            Err(format!("timer '{}' not found", name))
        }
    }

    pub fn record_failure(&mut self, name: &str) -> Result<bool, String> {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.name == name) {
            timer.failures += 1;
            let paused = timer.failures >= timer.max_failures;
            if paused {
                timer.paused = true;
            }
            Ok(paused)
        } else {
            Err(format!("timer '{}' not found", name))
        }
    }

    pub fn pause(&mut self, name: &str) {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.name == name) {
            timer.paused = true;
        }
    }

    pub fn resume(&mut self, name: &str) {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.name == name) {
            timer.paused = false;
        }
    }

    pub fn stats(&self) -> TimerStats {
        TimerStats {
            total_timers: self.timers.len(),
            active_timers: self.timers.iter().filter(|t| !t.paused).count(),
            paused_timers: self.timers.iter().filter(|t| t.paused).count(),
            total_failures: self.timers.iter().map(|t| t.failures).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_empty() {
        let r = TimerRegistry::new();
        assert_eq!(r.stats().total_timers, 0);
    }

    #[test]
    fn test_register_adds_timer() {
        let mut r = TimerRegistry::new();
        r.register("test", Duration::from_secs(60), 3);
        assert_eq!(r.stats().total_timers, 1);
        assert_eq!(r.stats().active_timers, 1);
    }

    #[test]
    fn test_due_returns_empty_on_fresh() {
        let mut r = TimerRegistry::new();
        r.register("test", Duration::from_secs(3600), 3);
        assert!(r.due().is_empty());
    }

    #[test]
    fn test_tick_updates_last_fired() {
        let mut r = TimerRegistry::new();
        r.register("test", Duration::from_secs(60), 3);
        assert!(r.tick("test").is_ok());
    }

    #[test]
    fn test_tick_unknown_timer() {
        let mut r = TimerRegistry::new();
        assert!(r.tick("nonexistent").is_err());
    }

    #[test]
    fn test_failure_pauses_timer() {
        let mut r = TimerRegistry::new();
        r.register("test", Duration::from_secs(60), 2);
        r.record_failure("test").unwrap();
        r.record_failure("test").unwrap();
        assert!(r.stats().paused_timers == 1);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut r = TimerRegistry::new();
        r.register("test", Duration::from_secs(60), 3);
        r.pause("test");
        assert_eq!(r.stats().paused_timers, 1);
        r.resume("test");
        assert_eq!(r.stats().paused_timers, 0);
    }
}
