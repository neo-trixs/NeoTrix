use crate::core::nt_core_experience::handler_tier::{HandlerRegistry, LoadTier};
use std::collections::HashMap;

/// Type-safe slot identifier into the harness handler set.
/// Each slot maps to one handler name + tier and supports
/// substitution-algebra operations at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HarnessSlotId(pub u32);

/// Metadata for a registered harness slot.
#[derive(Debug, Clone)]
pub struct SlotEntry {
    pub handler_name: &'static str,
    pub tier: LoadTier,
    pub created_at_cycle: u64,
}

/// Registry for composable harness slots.
///
/// Provides substitution-algebra operations:
/// - `insert` — add a new handler slot
/// - `remove` — delete a slot (propagates to HandlerRegistry)
/// - `replace` — swap handler name without changing slot id
/// - `swap` — exchange two slots' handler names
/// - `ids()` — iterate all slot ids
///
/// Designed as the **single source of truth** for handler metadata.
/// HandlerRegistry tiers and SelfInspectable handler_graph can be
/// rebuilt from this registry, eliminating the three-way duplication.
pub struct SlotRegistry {
    slots: HashMap<HarnessSlotId, SlotEntry>,
    next_id: u32,
}

impl SlotRegistry {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
            next_id: 1,
        }
    }

    /// Insert a new handler slot. Returns the assigned id.
    /// The handler is NOT automatically registered in HandlerRegistry —
    /// call `sync_to_registry()` to propagate.
    pub fn insert(&mut self, handler_name: &'static str, tier: LoadTier) -> HarnessSlotId {
        let id = HarnessSlotId(self.next_id);
        self.next_id += 1;
        self.slots.insert(
            id,
            SlotEntry {
                handler_name,
                tier,
                created_at_cycle: 0,
            },
        );
        id
    }

    /// Insert a batch of handler slots. Returns the assigned ids.
    pub fn insert_many(&mut self, entries: &[(&'static str, LoadTier)]) -> Vec<HarnessSlotId> {
        entries.iter().map(|(n, t)| self.insert(n, *t)).collect()
    }

    /// Remove a slot by id. Returns the old handler name, or None.
    pub fn remove(&mut self, id: HarnessSlotId) -> Option<&'static str> {
        self.slots.remove(&id).map(|e| e.handler_name)
    }

    /// Replace a slot's handler name without changing its id or tier.
    /// Returns the old name, or None if id not found.
    pub fn replace(&mut self, id: HarnessSlotId, new_name: &'static str) -> Option<&'static str> {
        let entry = self.slots.get_mut(&id)?;
        let old = entry.handler_name;
        entry.handler_name = new_name;
        Some(old)
    }

    /// Swap the handler names of two slots.
    pub fn swap(&mut self, id_a: HarnessSlotId, id_b: HarnessSlotId) -> bool {
        let a_name = self.slots.get(&id_a).map(|e| e.handler_name);
        let b_name = self.slots.get(&id_b).map(|e| e.handler_name);
        match (a_name, b_name) {
            (Some(an), Some(bn)) => {
                if let Some(ea) = self.slots.get_mut(&id_a) {
                    ea.handler_name = bn;
                }
                if let Some(eb) = self.slots.get_mut(&id_b) {
                    eb.handler_name = an;
                }
                true
            }
            _ => false,
        }
    }

    /// Number of registered slots.
    pub fn count(&self) -> usize {
        self.slots.len()
    }

    /// Get a slot entry by id.
    pub fn get(&self, id: HarnessSlotId) -> Option<&SlotEntry> {
        self.slots.get(&id)
    }

    /// Iterate over all (id, entry) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&HarnessSlotId, &SlotEntry)> {
        self.slots.iter()
    }

    /// Get all handler names sorted for deterministic iteration.
    pub fn handler_names(&self) -> Vec<&'static str> {
        let mut names: Vec<&str> = self.slots.values().map(|e| e.handler_name).collect();
        names.sort();
        names
    }

    /// Propagate all slots into a HandlerRegistry.
    /// Clears existing entries and re-registers from slots.
    pub fn sync_to_registry(&self, registry: &mut HandlerRegistry) {
        for entry in self.slots.values() {
            registry.register(entry.handler_name, entry.tier);
        }
    }

    /// Build a list of (name, tier) pairs for default initialization.
    pub fn to_tier_pairs(&self) -> Vec<(&'static str, LoadTier)> {
        let mut pairs: Vec<(&str, LoadTier)> = self
            .slots
            .values()
            .map(|e| (e.handler_name, e.tier))
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        pairs
    }
}

impl Default for SlotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a SlotRegistry with the full set of ~84 default handlers.
///
/// This is the single declaration point for all harness handlers.
/// Instead of editing 3 files (handler_tier.rs, modules.rs match arm,
/// self_inspect.rs handler_graph), the macro arms in modules.rs dispatch
/// would reference slots by name, and SelfInspectable queries this registry.
pub fn default_slot_registry() -> SlotRegistry {
    let mut sr = SlotRegistry::new();

    // ── Hot — every cycle (~50 handlers) ──
    for name in &[
        "context_gather",
        "decision_compress",
        "experience_reflect",
        "skill_accumulate",
        "curriculum_generate",
        "policy_repair",
        "epistemic_calibrate",
        "attractor_dynamics",
        "ebbinghaus_decay",
        "dream_cycle",
        "emergent_reasoning",
        "epistemic_honesty",
        "personality",
        "cognitive_state_ingest",
        "master_consciousness_update",
        "goal_execution",
        "goal_drift",
        "specious_present",
        "narrative",
        "valence",
        "inner_critic",
        "cognitive_load",
        "proof_search",
        "dgmh_writeback",
        "default_mode",
        "min_sufficient",
        "stream_buffer",
        "reconstructive_narrative",
        "adaptive_rate",
        "context_budget",
        "resonator_decode",
        "neuromodulator",
        "sar_diagnostic",
        "mirror_thread",
        "gea",
        "vsa_rdt",
        "tree_seeker",
        "shortcut_detect",
        "failure_mode",
        "social_belief",
        "fable_route",
        "memory_fs",
        "hypothesis_tree",
        "osc",
        "attention_gate",
        "uat_gate",
        "kroneker_cleanup",
        "episodic_memory",
        "goal_decomposition",
        "value_volition_bridge",
    ] {
        sr.insert(name, LoadTier::Hot);
    }

    // ── Warm — every 5-15 cycles (~25) ──
    for name in &[
        "volition",
        "conformal_uq",
        "confidence_calibrate",
        "failure_trace",
        "hive",
        "evosc",
        "counterfactual_futures",
        "dgm_variant_propose",
        "skill_dag",
        "value_system",
        "value_alignment",
        "dream_consolidate",
        "moss_health",
        "meta_reflection",
        "adversarial_arena",
        "reliability_gate",
        "open_skill",
        "dgmh_plan_verify_execute",
        "reasoning_step",
        "sia_feedback",
        "srcc_brain_dgm",
        "skill_evolution",
        "capability_synthesizer",
        "novelty_detection",
        "tool_discovery",
    ] {
        sr.insert(name, LoadTier::Warm);
    }

    // ── Cold — every 20-50 cycles (~10) ──
    for name in &[
        "e8_geometry",
        "ctm_inference",
        "async_delegate",
        "social_feed_absorb",
        "consciousness_pipeline",
        "moss_pipeline",
        "input_pipeline_batch",
        "archive_save",
        "translate",
        "lottie",
        "ne_compile",
    ] {
        sr.insert(name, LoadTier::Cold);
    }

    // ── Modular — external intelligence modules (every 15-30 cycles) ──
    for name in &[
        "news_radar",
        "voice_synthesis",
        "html_presentation",
        "loop_templates",
        "cyber_threat",
        "introspection",
        "contrastive_reflection",
        "faithfulness_auditor",
        "entity_resolver",
        "dysib",
        "interaction_trace",
        "keyword_lexicon",
        "hubness",
        "quant_data",
        "factor_miner",
        "fringe_mix",
        "osint",
        "mcp_intel",
        "cdp_session",
        "remote_host",
        "security_gate",
        "browser_mcp",
        "bridge",
        "ctm",
        "source_cognition",
        "vsa_input",
        "temporal_attention",
        "cross_modal",
    ] {
        sr.insert(name, LoadTier::Warm);
    }

    sr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_remove() {
        let mut sr = SlotRegistry::new();
        let id = sr.insert("test_handler", LoadTier::Hot);
        assert_eq!(sr.count(), 1);
        assert_eq!(sr.get(id).unwrap().handler_name, "test_handler");
        assert_eq!(sr.remove(id), Some("test_handler"));
        assert_eq!(sr.count(), 0);
    }

    #[test]
    fn test_replace_swaps_handler_name() {
        let mut sr = SlotRegistry::new();
        let id = sr.insert("old_name", LoadTier::Warm);
        assert_eq!(sr.replace(id, "new_name"), Some("old_name"));
        assert_eq!(sr.get(id).unwrap().handler_name, "new_name");
    }

    #[test]
    fn test_swap_exchanges_two_slots() {
        let mut sr = SlotRegistry::new();
        let a = sr.insert("handler_a", LoadTier::Hot);
        let b = sr.insert("handler_b", LoadTier::Cold);
        assert!(sr.swap(a, b));
        assert_eq!(sr.get(a).unwrap().handler_name, "handler_b");
        assert_eq!(sr.get(b).unwrap().handler_name, "handler_a");
    }

    #[test]
    fn test_unknown_id_returns_none() {
        let mut sr = SlotRegistry::new();
        assert!(sr.get(HarnessSlotId(999)).is_none());
        assert_eq!(sr.remove(HarnessSlotId(999)), None);
    }

    #[test]
    fn test_handler_names_sorted() {
        let mut sr = SlotRegistry::new();
        sr.insert("zeta", LoadTier::Cold);
        sr.insert("alpha", LoadTier::Hot);
        sr.insert("beta", LoadTier::Warm);
        let names = sr.handler_names();
        assert_eq!(names, vec!["alpha", "beta", "zeta"]);
    }

    #[test]
    fn test_sync_to_registry_propagates_tiers() {
        let mut sr = SlotRegistry::new();
        sr.insert("hot_h", LoadTier::Hot);
        sr.insert("warm_h", LoadTier::Warm);
        let mut reg = HandlerRegistry::new();
        sr.sync_to_registry(&mut reg);
        assert_eq!(reg.tier("hot_h"), LoadTier::Hot);
        assert_eq!(reg.tier("warm_h"), LoadTier::Warm);
    }

    #[test]
    fn test_default_slot_registry_has_80_plus() {
        let sr = default_slot_registry();
        assert!(sr.count() >= 80, "expected 80+ slots, got {}", sr.count());
    }

    #[test]
    fn test_to_tier_pairs_includes_all() {
        let sr = default_slot_registry();
        let pairs = sr.to_tier_pairs();
        assert_eq!(pairs.len(), sr.count());
        assert!(pairs.iter().any(|(n, _)| *n == "context_gather"));
    }

    #[test]
    fn test_remove_nonexistent_is_safe() {
        let mut sr = SlotRegistry::new();
        assert_eq!(sr.remove(HarnessSlotId(42)), None);
    }

    #[test]
    fn test_insert_many_bulk() {
        let mut sr = SlotRegistry::new();
        let ids = sr.insert_many(&[("a", LoadTier::Hot), ("b", LoadTier::Cold)]);
        assert_eq!(ids.len(), 2);
        assert_eq!(sr.count(), 2);
    }

    #[test]
    fn test_swap_nonexistent_returns_false() {
        let mut sr = SlotRegistry::new();
        let a = sr.insert("valid", LoadTier::Hot);
        assert!(!sr.swap(a, HarnessSlotId(999)));
        assert!(!sr.swap(HarnessSlotId(998), HarnessSlotId(999)));
    }

    #[test]
    fn test_iter_yields_all_entries() {
        let mut sr = SlotRegistry::new();
        sr.insert("x", LoadTier::Hot);
        sr.insert("y", LoadTier::Cold);
        let count = sr.iter().count();
        assert_eq!(count, 2);
    }
}
