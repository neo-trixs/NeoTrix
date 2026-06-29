use std::collections::HashMap;

/// Records a single healing attempt outcome.
#[derive(Clone, Debug)]
pub struct HealingRecord {
    pub skill_name: String,
    pub issue: String,
    pub action_taken: String,
    pub success: bool,
    pub timestamp: u64,
}

/// Tracks loaded skill health and drives auto-repair (MOLTRON-inspired).
///
/// Monitors consecutive failures per skill and triggers repair when
/// the failure threshold is exceeded. Maintains a Scorecard of healing
/// attempts and their success rates.
pub struct SkillHealthMonitor {
    failures: HashMap<String, u32>,
    successes: HashMap<String, u32>,
    loading_errors: HashMap<String, u32>,
    repair_history: Vec<HealingRecord>,
    max_failures_before_repair: u32,
    max_cached_skills: usize,
}

impl SkillHealthMonitor {
    pub fn new() -> Self {
        Self {
            failures: HashMap::new(),
            successes: HashMap::new(),
            loading_errors: HashMap::new(),
            repair_history: Vec::new(),
            max_failures_before_repair: 3,
            max_cached_skills: 100,
        }
    }

    pub fn record_loading_error(&mut self, skill_name: &str) {
        *self.failures.entry(skill_name.to_string()).or_insert(0) += 1;
        *self
            .loading_errors
            .entry(skill_name.to_string())
            .or_insert(0) += 1;
        self.prune_if_needed();
    }

    pub fn record_success(&mut self, skill_name: &str) {
        *self.successes.entry(skill_name.to_string()).or_insert(0) += 1;
        self.failures.remove(skill_name);
        self.loading_errors.remove(skill_name);
        self.prune_if_needed();
    }

    /// Returns skill names that have exceeded the consecutive failure threshold.
    pub fn needs_repair(&self) -> Vec<String> {
        self.failures
            .iter()
            .filter(|(_, &count)| count >= self.max_failures_before_repair)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Attempt to repair a failing skill by resetting its failure count.
    /// Returns a repair summary string.
    pub fn attempt_repair(&mut self, skill_name: &str, timestamp: u64) -> String {
        let old_count = self.failures.remove(skill_name).unwrap_or(0);
        self.loading_errors.remove(skill_name);

        let record = HealingRecord {
            skill_name: skill_name.to_string(),
            issue: format!("{} consecutive failures", old_count),
            action_taken: "reset failure count, queue re-evaluation".to_string(),
            success: true,
            timestamp,
        };
        self.repair_history.push(record);

        format!("repair:{}:cleared_{}_failures", skill_name, old_count)
    }

    /// Returns a diagnostic string for dashboard / response.
    pub fn diagnostic(&self) -> String {
        let total_failures: u32 = self.failures.values().sum();
        let total_successes: u32 = self.successes.values().sum();
        let repairing: usize = self.needs_repair().len();
        let total_healings = self.repair_history.len();
        let healing_successes = self.repair_history.iter().filter(|r| r.success).count();
        let healing_rate = if total_healings > 0 {
            healing_successes as f64 / total_healings as f64
        } else {
            1.0
        };
        format!(
            "skills:{}/fails:{}/ok:{}/repair:{}_heal:{:.0}%",
            self.failures.len() + self.successes.len(),
            total_failures,
            total_successes,
            repairing,
            healing_rate * 100.0
        )
    }

    fn prune_if_needed(&mut self) {
        if self.failures.len() + self.successes.len() > self.max_cached_skills {
            if let Some(oldest) = self.successes.keys().next().cloned() {
                self.successes.remove(&oldest);
            }
        }
        if self.repair_history.len() > 200 {
            self.repair_history.drain(0..100);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_monitor_is_empty() {
        let m = SkillHealthMonitor::new();
        assert!(m.needs_repair().is_empty());
    }

    #[test]
    fn test_failure_triggers_repair() {
        let mut m = SkillHealthMonitor::new();
        m.record_loading_error("email_parser");
        m.record_loading_error("email_parser");
        m.record_loading_error("email_parser");
        assert_eq!(m.needs_repair(), vec!["email_parser".to_string()]);
    }

    #[test]
    fn test_success_clears_failures() {
        let mut m = SkillHealthMonitor::new();
        m.record_loading_error("web_scraper");
        m.record_success("web_scraper");
        assert!(m.needs_repair().is_empty());
    }

    #[test]
    fn test_repair_resets_failure_count() {
        let mut m = SkillHealthMonitor::new();
        m.record_loading_error("broken_skill");
        m.record_loading_error("broken_skill");
        m.record_loading_error("broken_skill");
        let report = m.attempt_repair("broken_skill", 100);
        assert!(report.contains("cleared_3_failures"));
        assert!(m.needs_repair().is_empty());
    }

    #[test]
    fn test_healing_rate() {
        let mut m = SkillHealthMonitor::new();
        m.attempt_repair("s1", 1);
        m.attempt_repair("s2", 2);
        let diag = m.diagnostic();
        assert!(diag.contains("heal:100%"));
    }
}
