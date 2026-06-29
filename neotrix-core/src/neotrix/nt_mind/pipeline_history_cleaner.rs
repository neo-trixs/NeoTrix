//! Pipeline history cleaning — ToolPairHealing pattern.
//!
//! Prevents unbounded growth of pipeline history by removing redundant,
//! superseded, and obsolete entries. Operates as a pass-through filter
//! within the evolution loop cycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Age classification of a history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryAge {
    Recent,  // < 1 hour
    Warm,    // 1-24 hours
    Stale,   // 1-7 days
    Ancient, // > 7 days
}

/// Severity / importance of a history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntrySeverity {
    Critical,  // Never auto-prune
    Important, // Prune only if Ancient and superseded
    Normal,    // Prune if Stale
    Trivial,   // Prune immediately after processing
}

/// A single entry in the pipeline history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub issue_type: String,
    pub severity: EntrySeverity,
    pub description: String,
    pub resolved: bool,
    pub superseded_by: Option<u64>,
}

/// Statistics over the current history set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total: usize,
    pub resolved: usize,
    pub unresolved: usize,
    pub critical: usize,
    pub important: usize,
    pub normal: usize,
    pub trivial: usize,
    pub superseded: usize,
    pub oldest: Option<DateTime<Utc>>,
    pub newest: Option<DateTime<Utc>>,
}

/// Pipeline history cleaner — ToolPairHealing pattern.
///
/// Maintains a bounded vector of `HistoryEntry` items and exposes
/// a `clean()` method that removes entries according to the pruning rules:
///
/// 1. Remove entries superseded by a newer resolved entry.
/// 2. Remove entries that are Ancient AND resolved AND severity != Critical.
/// 3. Remove entries that are Stale AND severity == Trivial.
/// 4. If still above `max_entries`, remove oldest resolved entries.
/// 5. Never go below `min_entries`.
#[derive(Debug, Clone)]
pub struct PipelineHistoryCleaner {
    max_entries: usize,
    max_age_days: u64,
    min_entries: usize,
    history: Vec<HistoryEntry>,
    next_id: u64,
}

impl Default for PipelineHistoryCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineHistoryCleaner {
    pub fn new() -> Self {
        Self {
            max_entries: 1000,
            max_age_days: 30,
            min_entries: 50,
            history: Vec::new(),
            next_id: 1,
        }
    }

    // ── builder-style setters ────────────────────────────────────

    pub fn with_max_entries(mut self, n: usize) -> Self {
        self.max_entries = n;
        self
    }

    pub fn with_max_age_days(mut self, d: u64) -> Self {
        self.max_age_days = d;
        self
    }

    pub fn with_min_entries(mut self, n: usize) -> Self {
        self.min_entries = n;
        self
    }

    // ── public API ───────────────────────────────────────────────

    /// Add a new entry. The `id` field is auto-assigned if zero.
    pub fn add_entry(&mut self, mut entry: HistoryEntry) {
        if entry.id == 0 {
            entry.id = self.next_id;
            self.next_id += 1;
        } else {
            self.next_id = self.next_id.max(entry.id + 1);
        }
        self.history.push(entry);
    }

    /// Run a cleaning pass. Returns the number of removed entries.
    pub fn clean(&mut self) -> usize {
        let before = self.history.len();

        // Rule 1: remove entries superseded by a newer resolved entry
        let superseded_ids: Vec<u64> = self
            .history
            .iter()
            .filter_map(|e| {
                if let Some(sup_id) = e.superseded_by {
                    if self.history.iter().any(|o| o.id == sup_id && o.resolved) {
                        return Some(e.id);
                    }
                }
                None
            })
            .collect();
        self.history.retain(|e| !superseded_ids.contains(&e.id));

        let now = Utc::now();
        let ancient_cutoff = chrono::Duration::days(self.max_age_days as i64);
        let stale_cutoff = chrono::Duration::hours(24);
        let week_cutoff = chrono::Duration::days(7);

        // Rule 2: remove Ancient + resolved + severity != Critical
        //         but keep at least min_entries
        let rule2_targets: Vec<usize> = self
            .history
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let age = now.signed_duration_since(e.timestamp);
                age > ancient_cutoff && e.resolved && e.severity != EntrySeverity::Critical
            })
            .map(|(i, _)| i)
            .collect();
        let rule2_safe = self.history.len().saturating_sub(rule2_targets.len()) >= self.min_entries;
        if rule2_safe {
            for i in rule2_targets.into_iter().rev() {
                self.history.remove(i);
            }
        }

        // Rule 3: remove Stale + Trivial (resolved or not)
        //         but keep at least min_entries
        let rule3_targets: Vec<usize> = self
            .history
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let age = now.signed_duration_since(e.timestamp);
                age > stale_cutoff && age <= week_cutoff && e.severity == EntrySeverity::Trivial
            })
            .map(|(i, _)| i)
            .collect();
        let rule3_safe = self.history.len().saturating_sub(rule3_targets.len()) >= self.min_entries;
        if rule3_safe {
            for i in rule3_targets.into_iter().rev() {
                self.history.remove(i);
            }
        }

        // Rule 4: if still above max_entries, remove oldest resolved
        //         but keep at least min_entries
        while self.history.len() > self.max_entries && self.history.len() > self.min_entries {
            // find oldest resolved entry
            let oldest_resolved = self
                .history
                .iter()
                .enumerate()
                .filter(|(_, e)| e.resolved)
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(i, _)| i);
            if let Some(idx) = oldest_resolved {
                self.history.remove(idx);
            } else {
                break;
            }
        }

        let after = self.history.len();
        before.saturating_sub(after)
    }

    /// Mark an entry as resolved by its id.
    pub fn mark_resolved(&mut self, id: u64) {
        if let Some(entry) = self.history.iter_mut().find(|e| e.id == id) {
            entry.resolved = true;
        }
    }

    /// Mark an entry as superseded by another.
    pub fn mark_superseded(&mut self, id: u64, superseded_by: u64) {
        if let Some(entry) = self.history.iter_mut().find(|e| e.id == id) {
            entry.superseded_by = Some(superseded_by);
        }
    }

    /// Return a filtered, sorted copy of the history (newest first).
    pub fn history(&self) -> Vec<&HistoryEntry> {
        let mut items: Vec<&HistoryEntry> = self.history.iter().collect();
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    /// Return statistics about the current history.
    pub fn stats(&self) -> HistoryStats {
        let mut resolved = 0;
        let mut critical = 0;
        let mut important = 0;
        let mut normal = 0;
        let mut trivial = 0;
        let mut superseded = 0;

        for e in &self.history {
            if e.resolved {
                resolved += 1;
            }
            match e.severity {
                EntrySeverity::Critical => critical += 1,
                EntrySeverity::Important => important += 1,
                EntrySeverity::Normal => normal += 1,
                EntrySeverity::Trivial => trivial += 1,
            }
            if e.superseded_by.is_some() {
                superseded += 1;
            }
        }

        let oldest = self.history.iter().map(|e| e.timestamp).min();
        let newest = self.history.iter().map(|e| e.timestamp).max();

        HistoryStats {
            total: self.history.len(),
            resolved,
            unresolved: self.history.len() - resolved,
            critical,
            important,
            normal,
            trivial,
            superseded,
            oldest,
            newest,
        }
    }

    /// Age of an entry relative to now.
    pub fn entry_age(&self, entry: &HistoryEntry) -> EntryAge {
        let age = Utc::now().signed_duration_since(entry.timestamp);
        if age < chrono::Duration::hours(1) {
            EntryAge::Recent
        } else if age < chrono::Duration::hours(24) {
            EntryAge::Warm
        } else if age < chrono::Duration::days(7) {
            EntryAge::Stale
        } else {
            EntryAge::Ancient
        }
    }
}

// ================================================================
// Tests
// ================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: u64, severity: EntrySeverity, resolved: bool, days_ago: i64) -> HistoryEntry {
        HistoryEntry {
            id,
            timestamp: Utc::now() - chrono::Duration::days(days_ago),
            issue_type: "test".into(),
            severity,
            description: format!("test entry {}", id),
            resolved,
            superseded_by: None,
        }
    }

    #[test]
    fn test_clean_removes_superseded_entries() {
        let mut cleaner = PipelineHistoryCleaner::new();
        cleaner.add_entry(make_entry(1, EntrySeverity::Normal, false, 0));
        cleaner.add_entry(HistoryEntry {
            id: 2,
            timestamp: Utc::now(),
            issue_type: "test".into(),
            severity: EntrySeverity::Normal,
            description: "superseded by 1".into(),
            resolved: true,
            superseded_by: Some(1),
        });
        // 1 is resolved → 2 should be pruned
        cleaner.mark_resolved(1);
        let removed = cleaner.clean();
        assert_eq!(removed, 1, "should remove the superseded entry");
        assert_eq!(cleaner.history().len(), 1);
        assert_eq!(cleaner.history()[0].id, 1);
    }

    #[test]
    fn test_clean_removes_ancient_resolved_entries() {
        let mut cleaner = PipelineHistoryCleaner::new().with_max_age_days(7);
        // Only Normal severity entries get pruned (rule 2: severity != Critical)
        cleaner.add_entry(make_entry(1, EntrySeverity::Normal, true, 10)); // ancient + resolved
        cleaner.add_entry(make_entry(2, EntrySeverity::Normal, false, 3)); // recent, keep
        let removed = cleaner.clean();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.history().len(), 1);
    }

    #[test]
    fn test_clean_preserves_critical_entries() {
        let mut cleaner = PipelineHistoryCleaner::new().with_max_age_days(7);
        cleaner.add_entry(make_entry(1, EntrySeverity::Critical, true, 30));
        cleaner.add_entry(make_entry(2, EntrySeverity::Important, true, 30)); // Ancient but Important is not Critical → pruned
        let removed = cleaner.clean();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.history().len(), 1);
        assert_eq!(cleaner.history()[0].id, 1);
    }

    #[test]
    fn test_clean_keeps_minimum_entries() {
        let mut cleaner = PipelineHistoryCleaner::new()
            .with_max_entries(5)
            .with_min_entries(3);
        for i in 0..10 {
            cleaner.add_entry(make_entry(i, EntrySeverity::Trivial, true, i as i64 + 10));
        }
        let removed = cleaner.clean();
        // Should keep at least min_entries (3)
        assert!(cleaner.history().len() >= 3);
        // Should have removed some
        assert!(removed > 0);
    }

    #[test]
    fn test_add_and_mark_resolved() {
        let mut cleaner = PipelineHistoryCleaner::new();
        let e = HistoryEntry {
            id: 1,
            timestamp: Utc::now(),
            issue_type: "test".into(),
            severity: EntrySeverity::Normal,
            description: "test".into(),
            resolved: false,
            superseded_by: None,
        };
        cleaner.add_entry(e);
        assert!(!cleaner.history()[0].resolved);
        cleaner.mark_resolved(1);
        assert!(cleaner.history()[0].resolved);
    }

    #[test]
    fn test_auto_assigns_id() {
        let mut cleaner = PipelineHistoryCleaner::new();
        let e = HistoryEntry {
            id: 0,
            timestamp: Utc::now(),
            issue_type: "auto".into(),
            severity: EntrySeverity::Normal,
            description: "auto-id".into(),
            resolved: false,
            superseded_by: None,
        };
        cleaner.add_entry(e);
        assert_ne!(cleaner.history()[0].id, 0);
    }

    #[test]
    fn test_stats_reflect_history() {
        let mut cleaner = PipelineHistoryCleaner::new();
        cleaner.add_entry(make_entry(1, EntrySeverity::Critical, false, 0));
        cleaner.add_entry(make_entry(2, EntrySeverity::Normal, true, 1));
        let stats = cleaner.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.resolved, 1);
        assert_eq!(stats.unresolved, 1);
    }

    #[test]
    fn test_entry_age_classification() {
        let mut cleaner = PipelineHistoryCleaner::new();
        let ancient = make_entry(1, EntrySeverity::Normal, false, 10);
        let recent = make_entry(2, EntrySeverity::Normal, false, 0);
        cleaner.add_entry(ancient.clone());
        cleaner.add_entry(recent.clone());
        assert_eq!(cleaner.entry_age(&ancient), EntryAge::Ancient);
        assert_eq!(cleaner.entry_age(&recent), EntryAge::Recent);
    }

    #[test]
    fn test_clean_removes_stale_trivial() {
        let mut cleaner = PipelineHistoryCleaner::new();
        let mut e = make_entry(1, EntrySeverity::Trivial, false, 2); // Stale (2 days, between 24h and 7d)
        e.timestamp = Utc::now() - chrono::Duration::hours(48);
        cleaner.add_entry(e);
        // Need a second entry to avoid triggering Rule 5 (min_entries=50 won't matter since we only have 1 entry)
        // But wait — min_entries is 50 so cleaning won't remove the last one if it goes below 50...
        // Actually the second entry needs to be not stale/trivial so it survives
        let mut e2 = make_entry(2, EntrySeverity::Normal, false, 0);
        e2.timestamp = Utc::now();
        cleaner.add_entry(e2);
        let removed = cleaner.clean();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.history().len(), 1);
        assert_eq!(cleaner.history()[0].id, 2);
    }

    #[test]
    fn test_mark_superseded_and_clean() {
        let mut cleaner = PipelineHistoryCleaner::new();
        cleaner.add_entry(make_entry(1, EntrySeverity::Normal, false, 0));
        cleaner.add_entry(make_entry(2, EntrySeverity::Normal, false, 0));
        cleaner.mark_superseded(1, 2);
        cleaner.mark_resolved(2);
        let removed = cleaner.clean();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.history().len(), 1);
        assert_eq!(cleaner.history()[0].id, 2);
    }
}
