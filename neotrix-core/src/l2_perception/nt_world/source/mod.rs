//! NT-WORLD Media Source - 媒体源治理
//!
//! 媒体数据源的审计追踪、数据保留与删除策略
//! 域: NT-WORLD (虚空探索者)
//! 层: L2 Perception

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

// ============================================================================
// Audit
// ============================================================================

/// 审计结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Pass,
    Fail,
    Warn,
    Skip,
}

/// 单条审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: SystemTime,
    pub source_id: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub detail: String,
}

/// 审计轨迹 — 按 source_id 聚合的审计日志
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn recent(&self, n: usize) -> &[AuditEntry] {
        let len = self.entries.len();
        if n >= len {
            &self.entries
        } else {
            &self.entries[len - n..]
        }
    }

    pub fn filter_by_source(&self, source_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.source_id == source_id)
            .collect()
    }

    pub fn filter_by_outcome(&self, outcome: AuditOutcome) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.outcome == outcome)
            .collect()
    }
}

// ============================================================================
// Retention
// ============================================================================

/// 删除策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionStrategy {
    /// 软删除 — 标记为已删除，保留数据
    Soft,
    /// 彻底删除 — 不可恢复
    Hard,
    /// 归档后删除 — 先归档再删除
    ArchiveThenDelete,
}

/// 单条保留规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRule {
    pub data_type: String,
    pub max_age: Duration,
    pub strategy: DeletionStrategy,
}

/// 数据保留策略 — 规则集合 + 默认策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub rules: Vec<RetentionRule>,
    pub default_strategy: DeletionStrategy,
    pub default_max_age: Duration,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_strategy: DeletionStrategy::Soft,
            default_max_age: Duration::from_secs(90 * 24 * 3600), // 90 days
        }
    }
}

impl RetentionPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: RetentionRule) {
        self.rules.push(rule);
    }

    pub fn match_rule(&self, data_type: &str) -> Option<&RetentionRule> {
        self.rules.iter().find(|r| r.data_type == data_type)
    }

    pub fn strategy_for(&self, data_type: &str) -> DeletionStrategy {
        self.match_rule(data_type)
            .map(|r| r.strategy)
            .unwrap_or(self.default_strategy)
    }

    pub fn max_age_for(&self, data_type: &str) -> Duration {
        self.match_rule(data_type)
            .map(|r| r.max_age)
            .unwrap_or(self.default_max_age)
    }
}

// ============================================================================
// Data Record
// ============================================================================

/// 受治理管辖的数据记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: String,
    pub source_id: String,
    pub data_type: String,
    pub created_at: SystemTime,
    pub size_bytes: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl DataRecord {
    pub fn age(&self) -> Option<Duration> {
        self.created_at.elapsed().ok()
    }

    pub fn is_expired(&self, policy: &RetentionPolicy) -> bool {
        let max_age = policy.max_age_for(&self.data_type);
        self.age().map_or(false, |age| age > max_age)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_trail_record_and_filter() {
        let mut trail = AuditTrail::new();
        trail.record(AuditEntry {
            timestamp: SystemTime::now(),
            source_id: "s1".into(),
            action: "fetch".into(),
            outcome: AuditOutcome::Pass,
            detail: "ok".into(),
        });
        trail.record(AuditEntry {
            timestamp: SystemTime::now(),
            source_id: "s2".into(),
            action: "parse".into(),
            outcome: AuditOutcome::Fail,
            detail: "err".into(),
        });
        assert_eq!(trail.entries().len(), 2);
        assert_eq!(trail.filter_by_source("s1").len(), 1);
        assert_eq!(trail.filter_by_outcome(AuditOutcome::Fail).len(), 1);
    }

    #[test]
    fn test_retention_policy_defaults() {
        let policy = RetentionPolicy::new();
        assert_eq!(policy.default_strategy, DeletionStrategy::Soft);
        assert_eq!(policy.strategy_for("unknown"), DeletionStrategy::Soft);
    }

    #[test]
    fn test_retention_policy_rule_matching() {
        let mut policy = RetentionPolicy::new();
        policy.add_rule(RetentionRule {
            data_type: "image".into(),
            max_age: Duration::from_secs(30 * 24 * 3600),
            strategy: DeletionStrategy::Hard,
        });
        assert_eq!(policy.strategy_for("image"), DeletionStrategy::Hard);
        assert_eq!(policy.strategy_for("video"), DeletionStrategy::Soft);
    }
}
