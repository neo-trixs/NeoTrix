#![forbid(unsafe_code)]

//! Incremental tick computation for ConsciousnessTree.
//!
//! Inspired by the SpanCache pattern: instead of recomputing all modules every
//! tick, only recompute modules marked as "dirty". Cached results are reused
//! for unchanged modules, giving O(dirty) cost per tick instead of O(all).

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Identifies a ConsciousnessTree module / branch that participates in tick computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    NtMeta,
    NtRepair,
    NtGovernance,
    NtNexus,
    NtCore,
    NtMind,
    NtMemory,
    NtWorld,
    NtAct,
    NtIo,
    NtFeel,
}

impl ModuleId {
    /// All variants in deterministic order.
    pub const ALL: &'static [ModuleId] = &[
        ModuleId::NtCore,
        ModuleId::NtMind,
        ModuleId::NtMemory,
        ModuleId::NtWorld,
        ModuleId::NtAct,
        ModuleId::NtIo,
        ModuleId::NtFeel,
        ModuleId::NtMeta,
        ModuleId::NtRepair,
        ModuleId::NtGovernance,
        ModuleId::NtNexus,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ModuleId::NtMeta => "NT-META",
            ModuleId::NtRepair => "NT-REPAIR",
            ModuleId::NtGovernance => "NT-GOVERNANCE",
            ModuleId::NtNexus => "NT-NEXUS",
            ModuleId::NtCore => "NT-CORE",
            ModuleId::NtMind => "NT-MIND",
            ModuleId::NtMemory => "NT-MEMORY",
            ModuleId::NtWorld => "NT-WORLD",
            ModuleId::NtAct => "NT-ACT",
            ModuleId::NtIo => "NT-IO",
            ModuleId::NtFeel => "NT-FEEL",
        }
    }
}

/// Health snapshot for a single module, produced by `compute_module`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub module_id: ModuleId,
    /// Normalised health score in \[0.0, 1.0\].
    pub health_score: f64,
    /// Wall-clock instant when this result was computed.
    #[serde(skip)]
    pub last_computed: Instant,
    /// Whether the result is stale (module was dirty when last ticked).
    pub is_stale: bool,
}

/// Result of a single incremental or full tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickResult {
    /// Number of modules actually recomputed.
    pub recomputed: usize,
    /// Number of modules served from cache.
    pub cache_hits: usize,
    /// Total modules considered.
    pub total_modules: usize,
    /// Whether this was a full tick.
    pub was_full: bool,
    /// Per-module health results for this tick.
    pub results: HashMap<ModuleId, ModuleHealth>,
}

/// Incremental tick engine.
///
/// Maintains a dirty set and a result cache. On each tick, only dirty modules
/// are recomputed; clean modules are served from cache. A full tick forces
/// recomputation of every module regardless of dirty state.
#[derive(Debug)]
pub struct IncrementalTick {
    /// Modules that need recomputation on the next incremental tick.
    dirty: HashSet<ModuleId>,
    /// Cached health results from the last computation.
    cache: HashMap<ModuleId, ModuleHealth>,
    /// Number of full ticks executed.
    full_tick_count: u64,
    /// Number of incremental ticks executed.
    incremental_tick_count: u64,
    /// Total recomputation cost accumulated (module × tick).
    total_recomputed: u64,
    /// Total cache hits accumulated.
    total_cache_hits: u64,
}

impl Default for IncrementalTick {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalTick {
    pub fn new() -> Self {
        Self {
            dirty: HashSet::new(),
            cache: HashMap::new(),
            full_tick_count: 0,
            incremental_tick_count: 0,
            total_recomputed: 0,
            total_cache_hits: 0,
        }
    }

    /// Mark a module as needing recomputation on the next incremental tick.
    pub fn mark_dirty(&mut self, module: ModuleId) {
        self.dirty.insert(module);
    }

    /// Mark multiple modules as dirty.
    pub fn mark_dirty_many(&mut self, modules: impl IntoIterator<Item = ModuleId>) {
        self.dirty.extend(modules);
    }

    /// Remove a module from the dirty set (e.g. after external validation confirms no change).
    pub fn mark_clean(&mut self, module: ModuleId) {
        self.dirty.remove(&module);
    }

    /// Execute a tick.
    ///
    /// - `full = true`: recompute every module, clear dirty set.
    /// - `full = false`: recompute only dirty modules, serve the rest from cache.
    pub fn tick(&mut self, full: bool) -> TickResult {
        let mut results: HashMap<ModuleId, ModuleHealth> = HashMap::new();
        let mut recomputed = 0usize;
        let mut cache_hits = 0usize;

        let modules_to_compute: Vec<ModuleId> = if full {
            ModuleId::ALL.to_vec()
        } else {
            self.dirty.iter().copied().collect()
        };

        // Recompute dirty / all modules.
        for &id in &modules_to_compute {
            let health = Self::compute_module(id);
            results.insert(id, health);
            self.cache.insert(id, results[&id].clone());
            recomputed += 1;
        }

        // Serve clean modules from cache.
        for &id in ModuleId::ALL {
            if results.contains_key(&id) {
                continue;
            }
            if let Some(cached) = self.cache.get(&id) {
                let mut served = cached.clone();
                served.is_stale = false;
                results.insert(id, served);
                cache_hits += 1;
            } else {
                // First tick or module never computed — compute it.
                let health = Self::compute_module(id);
                results.insert(id, health);
                self.cache.insert(id, results[&id].clone());
                recomputed += 1;
            }
        }

        if full {
            self.full_tick_count += 1;
        } else {
            self.incremental_tick_count += 1;
        }

        self.total_recomputed += recomputed as u64;
        self.total_cache_hits += cache_hits as u64;

        if !full {
            self.dirty.clear();
        }

        TickResult {
            recomputed,
            cache_hits,
            total_modules: ModuleId::ALL.len(),
            was_full: full,
            results,
        }
    }

    /// Compute health for a single module (stub — real implementation delegates
    /// to each domain's self-test / heartbeat).
    fn compute_module(id: ModuleId) -> ModuleHealth {
        // Stub: return a deterministic score based on module identity.
        // In production this calls the module's SelfTest::run() or equivalent.
        let score = match id {
            ModuleId::NtCore => 0.95,
            ModuleId::NtMind => 0.88,
            ModuleId::NtMemory => 0.91,
            ModuleId::NtWorld => 0.84,
            ModuleId::NtAct => 0.87,
            ModuleId::NtIo => 0.90,
            ModuleId::NtFeel => 0.86,
            ModuleId::NtMeta => 0.92,
            ModuleId::NtRepair => 0.89,
            ModuleId::NtGovernance => 0.93,
            ModuleId::NtNexus => 0.85,
        };

        ModuleHealth {
            module_id: id,
            health_score: score,
            last_computed: Instant::now(),
            is_stale: false,
        }
    }

    /// Percentage of modules served from cache over all ticks (0.0 – 1.0).
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.total_recomputed + self.total_cache_hits;
        if total == 0 {
            0.0
        } else {
            self.total_cache_hits as f64 / total as f64
        }
    }

    /// Number of modules currently marked dirty.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Whether a module is dirty.
    pub fn is_dirty(&self, module: ModuleId) -> bool {
        self.dirty.contains(&module)
    }

    /// Total number of full ticks executed.
    pub fn full_tick_count(&self) -> u64 {
        self.full_tick_count
    }

    /// Total number of incremental ticks executed.
    pub fn incremental_tick_count(&self) -> u64 {
        self.incremental_tick_count
    }

    /// Read a cached module health (if available).
    pub fn cached_health(&self, module: ModuleId) -> Option<&ModuleHealth> {
        self.cache.get(&module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_id_all_has_11_variants() {
        assert_eq!(ModuleId::ALL.len(), 11);
    }

    #[test]
    fn module_id_labels_are_unique() {
        let labels: Vec<&str> = ModuleId::ALL.iter().map(|m| m.label()).collect();
        let mut deduped = labels.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(labels.len(), deduped.len(), "duplicate labels found");
    }

    #[test]
    fn new_engine_is_clean() {
        let engine = IncrementalTick::new();
        assert_eq!(engine.dirty_count(), 0);
        assert_eq!(engine.full_tick_count(), 0);
        assert_eq!(engine.incremental_tick_count(), 0);
        assert_eq!(engine.cache_hit_rate(), 0.0);
    }

    #[test]
    fn mark_dirty_adds_to_set() {
        let mut engine = IncrementalTick::new();
        engine.mark_dirty(ModuleId::NtCore);
        engine.mark_dirty(ModuleId::NtMind);
        assert!(engine.is_dirty(ModuleId::NtCore));
        assert!(engine.is_dirty(ModuleId::NtMind));
        assert_eq!(engine.dirty_count(), 2);
    }

    #[test]
    fn mark_dirty_many() {
        let mut engine = IncrementalTick::new();
        engine.mark_dirty_many([ModuleId::NtMeta, ModuleId::NtRepair, ModuleId::NtGovernance]);
        assert_eq!(engine.dirty_count(), 3);
    }

    #[test]
    fn mark_clean_removes() {
        let mut engine = IncrementalTick::new();
        engine.mark_dirty(ModuleId::NtCore);
        assert!(engine.is_dirty(ModuleId::NtCore));
        engine.mark_clean(ModuleId::NtCore);
        assert!(!engine.is_dirty(ModuleId::NtCore));
        assert_eq!(engine.dirty_count(), 0);
    }

    #[test]
    fn full_tick_recomputes_all() {
        let mut engine = IncrementalTick::new();
        let result = engine.tick(true);
        assert!(result.was_full);
        assert_eq!(result.recomputed, 11);
        assert_eq!(result.cache_hits, 0);
        assert_eq!(result.total_modules, 11);
        assert_eq!(engine.full_tick_count(), 1);
        assert_eq!(engine.incremental_tick_count(), 0);
    }

    #[test]
    fn incremental_tick_only_dirty_modules() {
        let mut engine = IncrementalTick::new();
        // First full tick to populate cache.
        engine.tick(true);

        // Mark two modules dirty.
        engine.mark_dirty(ModuleId::NtCore);
        engine.mark_dirty(ModuleId::NtIo);

        let result = engine.tick(false);
        assert!(!result.was_full);
        assert_eq!(result.recomputed, 2);
        assert_eq!(result.cache_hits, 9);
        assert_eq!(engine.incremental_tick_count(), 1);
        // Dirty set should be cleared after incremental tick.
        assert_eq!(engine.dirty_count(), 0);
    }

    #[test]
    fn incremental_tick_with_no_cache_uses_fallback() {
        let mut engine = IncrementalTick::new();
        engine.mark_dirty(ModuleId::NtCore);

        let result = engine.tick(false);
        // NtCore was computed, other 10 had no cache so they're also computed.
        assert_eq!(result.recomputed, 11);
        assert_eq!(result.cache_hits, 0);
    }

    #[test]
    fn cache_hit_rate_after_mixed_ticks() {
        let mut engine = IncrementalTick::new();

        // Full tick: 11 recomputed, 0 hits.
        engine.tick(true);

        // Incremental tick with 1 dirty: 1 recomputed, 10 hits.
        engine.mark_dirty(ModuleId::NtMind);
        engine.tick(false);

        // Total: 12 recomputed, 10 hits.
        let rate = engine.cache_hit_rate();
        assert!((rate - 10.0 / 22.0).abs() < 1e-10, "rate was {rate}");
    }

    #[test]
    fn cached_health_returns_previous_result() {
        let mut engine = IncrementalTick::new();
        engine.tick(true);
        let cached = engine.cached_health(ModuleId::NtCore);
        assert!(cached.is_some());
        let health = cached.unwrap();
        assert_eq!(health.module_id, ModuleId::NtCore);
        assert!(health.health_score > 0.0 && health.health_score <= 1.0);
    }

    #[test]
    fn cached_health_none_when_not_computed() {
        let engine = IncrementalTick::new();
        assert!(engine.cached_health(ModuleId::NtCore).is_none());
    }

    #[test]
    fn full_tick_resets_dirty_set() {
        let mut engine = IncrementalTick::new();
        engine.mark_dirty(ModuleId::NtMeta);
        engine.mark_dirty(ModuleId::NtNexus);
        assert_eq!(engine.dirty_count(), 2);

        engine.tick(true);
        assert_eq!(engine.dirty_count(), 0);
    }

    #[test]
    fn compute_module_health_in_range() {
        for &id in ModuleId::ALL {
            let health = IncrementalTick::compute_module(id);
            assert!(
                health.health_score >= 0.0 && health.health_score <= 1.0,
                "module {:?} score {} out of range",
                id,
                health.health_score
            );
            assert_eq!(health.module_id, id);
        }
    }

    #[test]
    fn consecutive_incremental_ticks_accumulate_counts() {
        let mut engine = IncrementalTick::new();

        for i in 0..5 {
            engine.mark_dirty(ModuleId::NtWorld);
            engine.tick(false);
            assert_eq!(engine.incremental_tick_count(), i + 1);
        }

        assert_eq!(engine.full_tick_count(), 0);
    }

    #[test]
    fn default_trait_works() {
        let engine = IncrementalTick::default();
        assert_eq!(engine.dirty_count(), 0);
    }

    #[test]
    fn tick_result_cloning() {
        let mut engine = IncrementalTick::new();
        let result = engine.tick(true);
        let cloned = result.clone();
        assert_eq!(cloned.recomputed, result.recomputed);
        assert_eq!(cloned.cache_hits, result.cache_hits);
        assert_eq!(cloned.results.len(), result.results.len());
    }
}
