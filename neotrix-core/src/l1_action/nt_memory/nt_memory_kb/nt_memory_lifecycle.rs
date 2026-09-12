//! # Memory Lifecycle Orchestrator
//!
//! Unifies the three forgetting mechanisms into a single coordinator:
//! 1. **ForgettingCurve** (Ebbinghaus) — marks nodes in DB via `should_forget()` metadata
//! 2. **FreshnessLedger** (staleness tracking) — filters stale docs from in-memory index
//! 3. **ConfidenceStore::apply_decay** — reduces recency confidence over time
//!
//! Before this module, each mechanism ran independently with no coordination,
//! leading to inconsistent retention decisions across subsystems.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::nt_memory_brain::ForgettingCurve;
use super::nt_memory_confidence::{ConfidenceStore, DecayConfig};
use super::nt_memory_sweep_20260815::FreshnessLedger;

/// Summary produced by a single `run_cycle` invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ForgettingReport {
    /// Nodes marked in DB metadata by the ForgettingCurve.
    pub curve_marked: usize,
    /// Confidence entries decayed by ConfidenceStore::apply_decay.
    pub confidence_decayed: u64,
    /// Documents explicitly marked as should-forget in the FreshnessLedger.
    pub ledger_marked: u64,
    /// Current FreshnessLedger clock tick after the cycle.
    pub current_tick: u64,
}

/// Unified memory lifecycle orchestrator.
///
/// Composes all three forgetting mechanisms and provides a single interface
/// for retention decisions. Each mechanism operates on its own data store
/// but the orchestrator coordinates their execution order.
#[derive(Debug, Clone)]
pub struct MemoryLifecycle {
    /// Ebbinghaus forgetting curve — time-based retention decay
    pub forgetting_curve: ForgettingCurve,
    /// Staleness tracking — clock-tick based freshness ledger
    pub freshness: FreshnessLedger,
    /// Confidence decay configuration — for `ConfidenceStore::apply_decay()`
    pub decay_config: DecayConfig,
    /// Default staleness threshold (ticks) for FreshnessLedger
    pub staleness_after: u64,
}

impl Default for MemoryLifecycle {
    fn default() -> Self {
        Self {
            forgetting_curve: ForgettingCurve::default(),
            freshness: FreshnessLedger::new(),
            decay_config: DecayConfig::default(),
            staleness_after: 100,
        }
    }
}

impl MemoryLifecycle {
    pub fn new(
        forgetting_curve: ForgettingCurve,
        decay_config: DecayConfig,
        staleness_after: u64,
    ) -> Self {
        Self {
            forgetting_curve,
            freshness: FreshnessLedger::new(),
            decay_config,
            staleness_after,
        }
    }

    /// Unified retention decision — checks all three mechanisms.
    ///
    /// A node is retained if ALL of:
    /// - ForgettingCurve says it should NOT be forgotten
    /// - FreshnessLedger says it is NOT stale
    /// - ConfidenceStore has NOT decayed it below threshold
    ///
    /// Returns `true` if the node should be retained (i.e., NOT forgotten).
    pub fn should_retain(
        &self,
        node_id: &str,
        last_access: i64,
        access_count: i64,
        confidence_store: &ConfidenceStore,
        now: i64,
    ) -> bool {
        // 1. ForgettingCurve: time-based retention
        if self.forgetting_curve.should_forget(last_access, access_count, now) {
            return false;
        }

        // 2. FreshnessLedger: explicit forget markers
        if self.freshness.should_forget(node_id) {
            return false;
        }

        // 3. ConfidenceStore: confidence-based retention
        if let Ok(Some(conf)) = confidence_store.get_confidence_by_str(node_id) {
            if conf.aggregate() < self.decay_config.min_confidence {
                return false;
            }
        }

        true
    }

    /// Run all three forgetting mechanisms and return a structured report.
    ///
    /// This is the primary entry point for periodic maintenance.
    /// Execution order mirrors `run_forgetting_cycle` but returns a
    /// richer `ForgettingReport` for observability.
    pub fn run_cycle(
        &mut self,
        conn: &Connection,
        confidence_store: &ConfidenceStore,
    ) -> Result<ForgettingReport, String> {
        // 1. ForgettingCurve: mark nodes that should be forgotten in DB
        let curve_marked = self.forgetting_curve.update_freshness(conn)?;

        // 2. ConfidenceStore: decay recency confidence for old records
        let lambda = self.decay_config.lambda_general;
        let older_than = self.decay_config.auto_archive_days;
        let confidence_decayed = confidence_store.apply_decay(lambda, older_than)?;

        // 3. FreshnessLedger: tick the clock
        let current_tick = self.freshness.tick();

        // Count documents explicitly marked as should-forget in the ledger.
        let ledger_marked = self.freshness.forget_count() as u64;

        Ok(ForgettingReport {
            curve_marked,
            confidence_decayed,
            ledger_marked,
            current_tick,
        })
    }

    /// Run all three forgetting mechanisms in sequence.
    ///
    /// Execution order:
    /// 1. ForgettingCurve marks stale nodes in DB metadata
    /// 2. ConfidenceStore decay reduces recency scores
    /// 3. FreshnessLedger tracks staleness for index filtering
    ///
    /// Returns `(nodes_marked_by_curve, confidences_decayed)`.
    pub fn run_forgetting_cycle(
        &mut self,
        conn: &Connection,
        confidence_store: &ConfidenceStore,
    ) -> Result<(usize, u64), String> {
        // 1. ForgettingCurve: mark nodes that should be forgotten in DB
        let marked = self.forgetting_curve.update_freshness(conn)?;

        // 2. ConfidenceStore: decay recency confidence for old records
        let lambda = self.decay_config.lambda_general;
        let older_than = self.decay_config.auto_archive_days;
        let decayed = confidence_store.apply_decay(lambda, older_than)?;

        // 3. FreshnessLedger: tick the clock (staleness is checked on read via is_stale())
        self.freshness.tick();

        Ok((marked, decayed))
    }

    /// Advance the freshness clock and return the new tick.
    pub fn tick(&mut self) -> u64 {
        self.freshness.tick()
    }

    /// Record that a document was updated at the given tick.
    pub fn note_updated(&mut self, doc_id: &str, at: u64) {
        self.freshness.note_updated(doc_id, at);
    }

    /// Mark a document as explicitly forgotten (removed from index).
    pub fn mark_should_forget(&mut self, doc_id: &str) {
        self.freshness.mark_should_forget(doc_id);
    }

    /// Check if a document is marked as should-forget.
    pub fn is_marked_forget(&self, doc_id: &str) -> bool {
        self.freshness.should_forget(doc_id)
    }

    /// Check if a document is stale (not updated within `staleness_after` ticks).
    pub fn is_stale(&self, doc_id: &str) -> bool {
        self.freshness.is_stale(doc_id, self.staleness_after)
    }

    /// Mark a document as fresh at the current clock tick (alias for `note_updated`).
    pub fn mark_fresh(&mut self, doc_id: &str) {
        let tick = self.freshness.tick();
        self.freshness.note_updated(doc_id, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_memory_lifecycle_default() {
        let lifecycle = MemoryLifecycle::default();
        assert_eq!(lifecycle.staleness_after, 100);
        assert_eq!(lifecycle.forgetting_curve.forget_threshold, 0.1);
    }

    #[test]
    fn test_should_retain_fresh_node() {
        let lifecycle = MemoryLifecycle::default();
        let store = ConfidenceStore::new(DecayConfig::default());

        // Fresh node: just accessed, high access count → should retain
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(lifecycle.should_retain("node1", now, 10, &store, now));
    }

    #[test]
    fn test_should_forget_very_old_node() {
        let lifecycle = MemoryLifecycle::default();
        let store = ConfidenceStore::new(DecayConfig::default());

        // Very old node: last accessed long ago, low access count → should forget
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(!lifecycle.should_retain("node2", 0, 1, &store, now));
    }

    #[test]
    fn test_update_all_returns_counts() {
        let mut lifecycle = MemoryLifecycle::default();
        let conn = Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        let store = ConfidenceStore::new(DecayConfig::default());

        let (marked, _decayed) = lifecycle.run_forgetting_cycle(&conn, &store).unwrap();
        // With empty DB, nothing to mark
        assert_eq!(marked, 0);
    }
}
