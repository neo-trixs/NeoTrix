//! Cleanup Coordinator - 清理协调器
//!
//! 协调清理操作，管理策略和优先级
//! 域: NT-META (元吸收者)
//! 层: L6 Meta-Cognition

use crate::l1_action::nt_act::nt_act_cleanup::shared::*;
use std::collections::HashMap;

/// 清理事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupEventType {
    ScanStarted,
    ScanCompleted,
    CleanStarted,
    CleanCompleted,
    CleanFailed,
    ArchiveCreated,
    BackupCreated,
}

/// 清理事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupEvent {
    pub timestamp: String,
    pub event_type: CleanupEventType,
    pub details: String,
    pub size_bytes: Option<u64>,
}

pub struct CleanupCoordinator {
    current_strategy: CleanupStrategy,
    cleanup_plan: CleanupPlan,
    stats: CleanupStats,
    rule_weights: HashMap<String, f64>,
    event_log: Vec<CleanupEvent>,
}

impl CleanupCoordinator {
    pub fn new() -> Self {
        Self {
            current_strategy: CleanupStrategy::Balanced,
            cleanup_plan: CleanupPlan {
                strategy: CleanupStrategy::Balanced,
                categories: vec!["system".into(), "developer".into(), "browser".into()],
                max_risk_level: "moderate".into(),
                dry_run: false, scheduled: false, interval_hours: None,
            },
            stats: CleanupStats { total_scanned: 0, total_cleaned: 0, total_skipped: 0, total_errors: 0, bytes_freed: 0, duration_ms: 0 },
            rule_weights: HashMap::new(),
            event_log: Vec::new(),
        }
    }

    pub fn set_strategy(&mut self, strategy: CleanupStrategy) {
        match &strategy {
            CleanupStrategy::Conservative => { self.cleanup_plan.max_risk_level = "safe".into(); self.cleanup_plan.categories = vec!["system".into()]; }
            CleanupStrategy::Balanced => { self.cleanup_plan.max_risk_level = "moderate".into(); self.cleanup_plan.categories = vec!["system".into(), "developer".into(), "browser".into()]; }
            CleanupStrategy::Aggressive => { self.cleanup_plan.max_risk_level = "risky".into(); self.cleanup_plan.categories = vec!["system".into(), "developer".into(), "browser".into(), "file".into()]; }
            CleanupStrategy::Custom => {}
        }
        self.current_strategy = strategy;
        self.cleanup_plan.strategy = self.current_strategy.clone();
    }

    pub fn get_strategy(&self) -> &CleanupStrategy { &self.current_strategy }
    pub fn get_plan(&self) -> &CleanupPlan { &self.cleanup_plan }

    pub fn calculate_priority(&self, category: &str, size_bytes: u64, age_days: u32) -> f64 {
        let mut p = match category {
            "system" => 1.0, "developer" => 0.8, "browser" => 0.6, "file" => 0.4, _ => 0.2,
        };
        p += (size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)) * 2.0;
        p += age_days as f64 / 30.0;
        if let Some(w) = self.rule_weights.get(category) { p *= w; }
        p
    }

    pub fn sort_by_priority(&self, items: &mut Vec<CleanupItem>) {
        items.sort_by(|a, b| {
            let pa = self.calculate_priority(&a.category, a.size_bytes, a.age_days);
            let pb = self.calculate_priority(&b.category, b.size_bytes, b.age_days);
            pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn update_stats(&mut self, scanned: usize, cleaned: usize, skipped: usize, errors: usize, bytes_freed: u64) {
        self.stats.total_scanned += scanned;
        self.stats.total_cleaned += cleaned;
        self.stats.total_skipped += skipped;
        self.stats.total_errors += errors;
        self.stats.bytes_freed += bytes_freed;
    }

    pub fn get_stats(&self) -> &CleanupStats { &self.stats }
    pub fn set_rule_weight(&mut self, category: &str, weight: f64) { self.rule_weights.insert(category.into(), weight); }

    pub fn log_event(&mut self, event_type: CleanupEventType, details: String, size_bytes: Option<u64>) {
        self.event_log.push(CleanupEvent {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type, details, size_bytes,
        });
    }

    pub fn get_event_log(&self) -> &[CleanupEvent] { &self.event_log }

    pub fn generate_report(&self) -> CleanupReport {
        let mut recs = Vec::new();
        if self.stats.total_errors > 0 { recs.push("有清理错误发生".into()); }
        if self.stats.bytes_freed > 1024 * 1024 * 1024 { recs.push("释放超过1GB".into()); }
        if self.stats.total_cleaned == 0 { recs.push("未清理任何内容".into()); }
        CleanupReport { strategy: self.current_strategy.clone(), stats: self.stats.clone(), recommendations: recs }
    }
}

impl Default for CleanupCoordinator { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_new() {
        let c = CleanupCoordinator::new();
        assert!(matches!(c.current_strategy, CleanupStrategy::Balanced));
    }

    #[test]
    fn test_set_strategy() {
        let mut c = CleanupCoordinator::new();
        c.set_strategy(CleanupStrategy::Aggressive);
        assert!(matches!(c.current_strategy, CleanupStrategy::Aggressive));
        assert_eq!(c.cleanup_plan.max_risk_level, "risky");
    }

    #[test]
    fn test_calculate_priority() {
        let c = CleanupCoordinator::new();
        let p = c.calculate_priority("system", 1024 * 1024 * 1024, 30);
        assert!(p > 0.0);
    }

    #[test]
    fn test_log_event() {
        let mut c = CleanupCoordinator::new();
        c.log_event(CleanupEventType::ScanStarted, "扫描开始".into(), None);
        assert_eq!(c.event_log.len(), 1);
    }

    #[test]
    fn test_generate_report() {
        let mut c = CleanupCoordinator::new();
        c.update_stats(100, 50, 40, 10, 1024 * 1024);
        let r = c.generate_report();
        assert_eq!(r.stats.total_scanned, 100);
    }
}
