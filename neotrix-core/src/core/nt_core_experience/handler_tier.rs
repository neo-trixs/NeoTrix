use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::trajectory_heuristics::Heuristic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadTier {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadStatus {
    Ready,
    NeedsInit,
    NeedsReload,
}

pub struct HandlerRegistry {
    tiers: HashMap<String, LoadTier>,
    load_counts: HashMap<String, u64>,
    last_access: HashMap<String, Instant>,
    warm_cache_ttl: Duration,
    success_counts: HashMap<String, u64>,
    failure_counts: HashMap<String, u64>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            tiers: HashMap::new(),
            load_counts: HashMap::new(),
            last_access: HashMap::new(),
            warm_cache_ttl: Duration::from_secs(300),
            success_counts: HashMap::new(),
            failure_counts: HashMap::new(),
        }
    }

    pub fn perf_snapshot(&self) -> HashMap<String, (u64, u64)> {
        let mut snap = HashMap::new();
        for name in self.tiers.keys() {
            let s = self.success_counts.get(name).copied().unwrap_or(0);
            let f = self.failure_counts.get(name).copied().unwrap_or(0);
            snap.insert(name.clone(), (s, f));
        }
        snap
    }

    pub fn perf_diff(
        &self,
        before: &HashMap<String, (u64, u64)>,
    ) -> HashMap<String, (i64, i64, f64)> {
        let mut diff = HashMap::new();
        for (name, (pre_s, pre_f)) in before {
            let cur_s = self.success_counts.get(name).copied().unwrap_or(0);
            let cur_f = self.failure_counts.get(name).copied().unwrap_or(0);
            let d_s = cur_s as i64 - *pre_s as i64;
            let d_f = cur_f as i64 - *pre_f as i64;
            let pre_total = *pre_s + *pre_f;
            let pre_rate = if pre_total == 0 {
                0.0
            } else {
                *pre_s as f64 / pre_total as f64
            };
            diff.insert(name.clone(), (d_s, d_f, pre_rate));
        }
        diff
    }

    pub fn register(&mut self, name: &str, tier: LoadTier) {
        self.tiers.insert(name.to_string(), tier);
    }

    pub fn register_many(&mut self, entries: &[(&str, LoadTier)]) {
        for (name, tier) in entries {
            self.tiers.insert(name.to_string(), *tier);
        }
    }

    pub fn tier(&self, name: &str) -> LoadTier {
        self.tiers.get(name).copied().unwrap_or(LoadTier::Warm)
    }

    pub fn record_access(&mut self, name: &str) -> LoadStatus {
        let now = Instant::now();
        let last = self.last_access.get(name).copied();
        let count = self.load_counts.entry(name.to_string()).or_insert(0);
        *count += 1;
        self.last_access.insert(name.to_string(), now);

        let tier = self.tier(name);
        match tier {
            LoadTier::Hot => LoadStatus::Ready,
            LoadTier::Warm => {
                if let Some(last_time) = last {
                    if now.duration_since(last_time) < self.warm_cache_ttl {
                        LoadStatus::Ready
                    } else {
                        LoadStatus::NeedsReload
                    }
                } else {
                    LoadStatus::NeedsInit
                }
            }
            LoadTier::Cold => LoadStatus::NeedsInit,
        }
    }

    pub fn mark_unloaded(&mut self, name: &str) {
        self.last_access.remove(name);
    }

    pub fn mark_loaded(&mut self, name: &str) {
        self.last_access
            .entry(name.to_string())
            .or_insert_with(Instant::now);
    }

    pub fn stale_handlers(&self, max_age: Duration) -> Vec<String> {
        let now = Instant::now();
        self.last_access
            .iter()
            .filter(|(name, _)| self.tier(name) == LoadTier::Cold)
            .filter(|(_, time)| now.duration_since(**time) > max_age)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn stats(&self) -> LoadTierStats {
        let mut hot = 0u64;
        let mut warm = 0u64;
        let mut cold = 0u64;
        for (_, tier) in &self.tiers {
            match tier {
                LoadTier::Hot => hot += 1,
                LoadTier::Warm => warm += 1,
                LoadTier::Cold => cold += 1,
            }
        }
        LoadTierStats {
            hot,
            warm,
            cold,
            total: hot + warm + cold,
        }
    }

    pub fn total_calls(&self) -> u64 {
        self.load_counts.values().sum()
    }

    pub fn handler_names(&self) -> Vec<String> {
        self.tiers.keys().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.tiers.len()
    }

    pub fn record_success(&mut self, name: &str) {
        *self.success_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn record_failure(&mut self, name: &str) {
        *self.failure_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn success_rate(&self, name: &str) -> Option<f64> {
        let success = self.success_counts.get(name).copied().unwrap_or(0);
        let failure = self.failure_counts.get(name).copied().unwrap_or(0);
        let total = success + failure;
        if total == 0 {
            return None;
        }
        Some(success as f64 / total as f64)
    }

    pub fn worst_handlers(&self, threshold: f64) -> Vec<String> {
        let mut result = Vec::new();
        for name in self.tiers.keys() {
            if let Some(rate) = self.success_rate(name) {
                if rate < threshold {
                    result.push(name.clone());
                }
            }
        }
        result
    }

    pub fn perf_report(&self) -> String {
        let mut all_names: Vec<&String> = self.success_counts.keys().collect();
        for name in self.failure_counts.keys() {
            if !self.success_counts.contains_key(name) {
                all_names.push(name);
            }
        }
        let mut parts: Vec<String> = Vec::new();
        for name in &all_names {
            let successes = self.success_counts.get(*name).copied().unwrap_or(0);
            let failures = self.failure_counts.get(*name).copied().unwrap_or(0);
            let total = successes + failures;
            if total > 0 {
                let rate = successes as f64 / total as f64 * 100.0;
                parts.push(format!("{}:{:.0}%({}/{})", name, rate, successes, total));
            }
        }
        if parts.is_empty() {
            "perf:no_data".to_string()
        } else {
            format!("perf:{}", parts.join("|"))
        }
    }

    pub fn promote(&mut self, name: &str) {
        self.tiers.insert(name.to_string(), LoadTier::Hot);
    }

    pub fn demote(&mut self, name: &str) {
        self.tiers.insert(name.to_string(), LoadTier::Cold);
    }

    pub fn set_tier(&mut self, name: &str, tier: LoadTier) {
        self.tiers.insert(name.to_string(), tier);
    }

    pub fn unregister(&mut self, name: &str) {
        self.tiers.remove(name);
        self.load_counts.remove(name);
        self.last_access.remove(name);
        self.success_counts.remove(name);
        self.failure_counts.remove(name);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tiers.contains_key(name)
    }

    pub fn zero_access_handlers(&self) -> Vec<String> {
        self.tiers
            .keys()
            .filter(|name| !self.load_counts.contains_key(name.as_str()))
            .cloned()
            .collect()
    }

    pub fn clear_stats(&mut self, name: &str) {
        self.success_counts.remove(name);
        self.failure_counts.remove(name);
        self.load_counts.remove(name);
    }

    pub fn prune_stale(&mut self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        let stale: Vec<String> = self
            .last_access
            .iter()
            .filter(|(_, last)| now.duration_since(**last) > max_age)
            .map(|(name, _)| name.clone())
            .collect();
        for name in stale {
            self.unregister(&name);
        }
    }

    pub fn handlers_by_tier(&self, tier: LoadTier) -> Vec<String> {
        self.tiers
            .iter()
            .filter(|(_, t)| **t == tier)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn total_success_rate(&self) -> f64 {
        let mut total_success: u64 = 0;
        let mut total_fail: u64 = 0;
        for name in self.tiers.keys() {
            if let Some(&s) = self.success_counts.get(name) {
                total_success += s;
            }
            if let Some(&f) = self.failure_counts.get(name) {
                total_fail += f;
            }
        }
        let total = total_success + total_fail;
        if total == 0 {
            1.0
        } else {
            total_success as f64 / total as f64
        }
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LoadTierStats {
    pub hot: u64,
    pub warm: u64,
    pub cold: u64,
    pub total: u64,
}

pub fn default_handler_tiers() -> HashMap<&'static str, LoadTier> {
    let mut m = HashMap::new();

    // Hot — every cycle (~50 handlers)
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
        "drive_selector",
        "memory_lattice",
        "memory_palace",
        "sub_agent_spawn",
        "sub_agent_tick",
        "sub_agent_collect",
        "lead_agent_plan",
        "lead_agent_execute",
        "preview_options",
        "ultra_review",
        "goal_manager_create",
        "goal_manager_execute",
        "goal_manager_status",
        "goal_manager_pause",
        "goal_manager_resume",
        "goal_manager_cancel",
        "permission_set_mode",
        "permission_check",
        "permission_override",
        "verify_check",
        "verify_toggle",
        "dispatch_pipeline_mode",
        "transcript_status",
        "transcript_flush",
        "transcript_set_path",
        "memory_summary",
        "memory_query",
        "memory_add_explicit",
        "memory_add_discovered",
        "memory_add_lesson",
        "memory_to_markdown",
        "daemon_status",
        "daemon_start",
        "daemon_stop",
        "daemon_inbox_read",
        "identity_cycle",
        "self_reason",
        "identity_persist",
    ] {
        m.insert(*name, LoadTier::Hot);
    }

    // Warm — every 5-15 cycles (~20)
    for name in &[
        "volition",
        "conformal_uq",
        "confidence_calibrate",
        "failure_trace",
        "hive",
        "imagination",
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
        "physics",
        "dgmh_plan_verify_execute",
        "reasoning_step",
        "sia_feedback",
        "spatial",
        "srcc_brain_dgm",
        "skill_evolution",
        "capability_synthesizer",
        "novelty_detection",
        "tool_discovery",
        "vsa_vocabulary",
        "metrics",
        "knowledge_base",
        "e8_training",
    ] {
        m.insert(*name, LoadTier::Warm);
    }

    // Cold — every 20-50 cycles (~8)
    for name in &[
        "e8_geometry",
        "ctm_inference",
        "async_delegate",
        "social_feed_absorb",
        "consciousness_pipeline",
        "moss_pipeline",
        "input_pipeline_batch",
        "archive_save",
        "ne_compile",
        "research_trajectory",
        "storm_perspective",
        "storm_conversation",
        "storm_synthesis",
        "storm_critique",
        "storm_status",
    ] {
        m.insert(*name, LoadTier::Cold);
    }

    // Gap 3 — Chinese content creation skills (Warm, every 10-20 cycles)
    for name in &[
        "humanizer",
        "business_diagnosis",
        "visual_planner",
        "research_writer",
        "self_play_guide",
    ] {
        m.insert(*name, LoadTier::Warm);
    }

    // Gap 4 — Self-evolution engines (Warm, every 20-30 cycles)
    for name in &[
        "self_harness",
        "self_harness_stats",
        "context_compressor",
        "context_compressor_stats",
        "egpo",
        "egpo_stats",
        "self_evolution",
        "evolution_coordinator",
    ] {
        m.insert(*name, LoadTier::Warm);
    }

    // Gap 5 — Unregistered dispatch arms (auto-WARM fallback)
    // Core consciousness & bridges
    for name in &[
        "bridge",
        "checkpoint",
        "ctm",
        "source_cognition",
        "vsa_input",
        "temporal_attention",
        "cross_modal",
        "narrative_self",
        "valence_axis",
        "first_person",
        "awakening",
        "constitution",
        "workspace",
        "dream_consolidator",
        "meta_cognition",
        "meta_cog_plan",
        "meta_cog_regulate",
        "calibration",
        "working_memory",
        "world_model",
        "layer_management",
        "trace_mining",
        "translate_engine",
        "memory_sync",
        "memory_reflector",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Cognitive dynamics & patterns
    for name in &[
        "skill_trend",
        "exploratory_gap",
        "signal_pattern",
        "resonance",
        "emergent_property",
        "concept_drift",
        "reflexivity",
        "cognitive_diversity",
        "story_generator",
        "mirror_buffer",
        "adapt_orch",
        "godel_round",
        "curiosity_drive",
        "exploration_orchestrate",
        "neuromodulate",
        "arena_round",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // HyperCube dispatch
    for name in &["selfref_meta", "memory_activation", "efe_curiosity_bridge"] {
        m.insert(*name, LoadTier::Warm);
    }

    // Adaptive processing & safety
    for name in &[
        "adaptive_vsa",
        "null_drift",
        "thdc",
        "sparse_vsa_attn",
        "vsa_moe",
        "pcc_safety",
        "ball_verifier",
        "progress_rag",
        "validity_crosscheck",
        "loss_recalibrate",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Ne evolution & self systems
    for name in &[
        "ne_evaluator",
        "ne_loader",
        "evolution_bridge",
        "design_token",
        "self_revision",
        "meta_agent",
        "ema_jepa",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Research
    for name in &[
        "research",
        "research_propose",
        "research_stats",
        "research_kg",
        "research_kg_submit",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Job queue
    for name in &[
        "job_queue",
        "job_queue_stats",
        "job_queue_submit",
        "architecture_report",
        "architecture_status",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Exporters & explorers
    for name in &[
        "okf_exporter",
        "native_explorer",
        "contrastive_reflection",
        "faithfulness_auditor",
        "entity_resolver",
        "dysib",
        "interaction_trace",
        "keyword_lexicon",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Data & security
    for name in &[
        "quant_data",
        "cdp_session",
        "fringe_mix",
        "factor_miner",
        "osint",
        "mcp_intel",
        "hubness",
        "remote_host",
        "security_gate",
        "browser_mcp",
        "koopman",
        "news_radar",
        "voice_synthesis",
        "html_presentation",
        "loop_templates",
        "cyber_threat",
        "introspection",
        "faithfulness",
        "motion_synthesizer",
        "decoder_learning",
        "mirror",
        "transcript_analysis",
        "induction",
        "belief_trajectory",
        "dgmh_meta",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Workflow
    for name in &[
        "workflow_execute",
        "workflow_list",
        "workflow_summary",
        "sandbox_execute",
        "sandbox_cleanup",
    ] {
        m.insert(*name, LoadTier::Warm);
    }
    // Health & geometry alias
    for name in &["skill_health", "e8_geometry_tick"] {
        m.insert(*name, LoadTier::Warm);
    }

    // Vision pipeline
    m.insert("vision_integrate", LoadTier::Warm);

    // XC-02: Dispatch arms previously missing tier registration
    m.insert("memory_consolidate", LoadTier::Warm);
    m.insert("goal_decompose", LoadTier::Hot);
    m.insert("storage_engine", LoadTier::Cold);
    m.insert("e8_cortical", LoadTier::Warm);

    // P1.01–P1.08: New disconnected subsystem handlers
    m.insert("evidence", LoadTier::Cold);
    m.insert("spread_activation", LoadTier::Cold);
    m.insert("consensus", LoadTier::Cold);
    m.insert("hypergraph", LoadTier::Cold);
    m.insert("storm_poll", LoadTier::Cold);
    m.insert("hypothesis_tree", LoadTier::Warm);
    m.insert("kb_maintenance", LoadTier::Cold);

    // N12 + O04: Three-Role Manager & Sub-Consciousness Manager (always-on Warm)
    m.insert("three_role", LoadTier::Warm);
    m.insert("sub_consciousness", LoadTier::Warm);

    m
}

pub struct HeuristicCapability {
    pub name: String,
    pub heuristic_source: Option<Heuristic>,
    pub trigger_pattern: String,
    pub handler_fn: Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
    pub confidence: f64,
    pub invocation_count: u64,
    pub success_count: u64,
}

impl std::fmt::Debug for HeuristicCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeuristicCapability")
            .field("name", &self.name)
            .field("heuristic_source", &self.heuristic_source)
            .field("trigger_pattern", &self.trigger_pattern)
            .field("confidence", &self.confidence)
            .field("invocation_count", &self.invocation_count)
            .field("success_count", &self.success_count)
            .finish()
    }
}

impl Clone for HeuristicCapability {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            heuristic_source: self.heuristic_source.clone(),
            trigger_pattern: self.trigger_pattern.clone(),
            handler_fn: self.handler_fn.clone(),
            confidence: self.confidence,
            invocation_count: self.invocation_count,
            success_count: self.success_count,
        }
    }
}

impl HeuristicCapability {
    pub fn new(
        name: &str,
        trigger_pattern: &str,
        handler_fn: Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
        confidence: f64,
    ) -> Self {
        Self {
            name: name.to_string(),
            heuristic_source: None,
            trigger_pattern: trigger_pattern.to_string(),
            handler_fn,
            confidence,
            invocation_count: 0,
            success_count: 0,
        }
    }

    pub fn matches(&self, context: &str) -> bool {
        let context_lower = context.to_lowercase();
        let pattern_lower = self.trigger_pattern.to_lowercase();
        let keywords: Vec<&str> = pattern_lower.split_whitespace().collect();
        if keywords.is_empty() {
            return false;
        }
        let matches = keywords
            .iter()
            .filter(|&&kw| context_lower.contains(kw))
            .count();
        matches as f64 / keywords.len() as f64 >= 0.3
    }

    pub fn execute(&mut self, context: &str) -> Result<String, String> {
        self.invocation_count += 1;
        let result = (self.handler_fn)(context);
        if result.is_ok() {
            self.success_count += 1;
            self.confidence = (self.confidence + 0.05).clamp(0.0, 1.0);
        } else {
            self.confidence = (self.confidence - 0.02).clamp(0.0, 1.0);
        }
        result
    }

    pub fn success_rate(&self) -> f64 {
        if self.invocation_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.invocation_count as f64
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    pub capabilities: Vec<HeuristicCapability>,
    max_capabilities: usize,
    pub(crate) min_confidence_to_keep: f64,
}

impl CapabilityRegistry {
    pub fn new(max_capabilities: usize) -> Self {
        Self {
            capabilities: Vec::with_capacity(max_capabilities),
            max_capabilities,
            min_confidence_to_keep: 0.1,
        }
    }

    pub fn register_from_heuristic(
        &mut self,
        heuristic: &Heuristic,
    ) -> Option<HeuristicCapability> {
        if self.capabilities.len() >= self.max_capabilities {
            self.prune_lowest_confidence();
        }
        if self.capabilities.len() >= self.max_capabilities {
            return None;
        }

        let name = format!(
            "heur_{}",
            heuristic.pattern.chars().take(20).collect::<String>()
        );
        let trigger = heuristic.pattern.clone();
        let principle = heuristic.principle.clone();
        let confidence = heuristic.confidence;

        if self
            .capabilities
            .iter()
            .any(|c| c.trigger_pattern == trigger)
        {
            return None;
        }

        let cap = HeuristicCapability {
            name,
            heuristic_source: Some(heuristic.clone()),
            trigger_pattern: trigger,
            handler_fn: Arc::new(move |ctx: &str| -> Result<String, String> {
                Ok(format!(
                    "Applied heuristic: {} (context: {})",
                    principle, ctx
                ))
            }),
            confidence,
            invocation_count: 0,
            success_count: 0,
        };

        self.capabilities.push(cap.clone());
        Some(cap)
    }

    pub fn find_matching(&self, context: &str) -> Vec<&HeuristicCapability> {
        self.capabilities
            .iter()
            .filter(|c| c.matches(context))
            .collect()
    }

    pub fn find_matching_mut(&mut self, context: &str) -> Vec<&mut HeuristicCapability> {
        self.capabilities
            .iter_mut()
            .filter(|c| c.matches(context))
            .collect()
    }

    pub fn evolve(&mut self, heuristics: &[Heuristic]) -> usize {
        let mut registered = 0;
        for h in heuristics {
            if self.register_from_heuristic(h).is_some() {
                registered += 1;
            }
        }
        self.prune_low_confidence();
        registered
    }

    pub fn prune_low_confidence(&mut self) {
        self.capabilities
            .retain(|c| c.confidence >= self.min_confidence_to_keep);
    }

    fn prune_lowest_confidence(&mut self) {
        if let Some(idx) = self
            .capabilities
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1.confidence
                    .partial_cmp(&b.1.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        {
            self.capabilities.swap_remove(idx);
        }
    }

    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    pub fn best_capabilities(&self, top_k: usize) -> Vec<&HeuristicCapability> {
        let mut sorted: Vec<&HeuristicCapability> = self.capabilities.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(top_k).collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_default_tiers_contain_all_categories() {
        let tiers = default_handler_tiers();
        let has_hot = tiers.values().any(|t| *t == LoadTier::Hot);
        let has_warm = tiers.values().any(|t| *t == LoadTier::Warm);
        let has_cold = tiers.values().any(|t| *t == LoadTier::Cold);
        assert!(has_hot, "should have hot entries");
        assert!(has_warm, "should have warm entries");
        assert!(has_cold, "should have cold entries");
    }

    #[test]
    fn test_record_access_returns_ready_for_hot() {
        let mut reg = HandlerRegistry::new();
        reg.register("hot_handler", LoadTier::Hot);
        assert_eq!(reg.record_access("hot_handler"), LoadStatus::Ready);
    }

    #[test]
    fn test_record_access_returns_needs_init_for_unregistered() {
        let mut reg = HandlerRegistry::new();
        assert_eq!(reg.record_access("unknown"), LoadStatus::NeedsInit);
    }

    #[test]
    fn test_warm_handler_returns_ready_on_second_access_within_ttl() {
        let mut reg = HandlerRegistry::new();
        reg.register("warm_handler", LoadTier::Warm);
        // First access: NeedsInit
        assert_eq!(reg.record_access("warm_handler"), LoadStatus::NeedsInit);
        // Second access immediately: Ready (within TTL)
        assert_eq!(reg.record_access("warm_handler"), LoadStatus::Ready);
    }

    #[tokio::test]
    async fn test_warm_handler_needs_reload_after_ttl() {
        let mut reg = HandlerRegistry::new();
        reg.warm_cache_ttl = Duration::from_millis(10);
        reg.register("warm_handler", LoadTier::Warm);
        reg.record_access("warm_handler");
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(reg.record_access("warm_handler"), LoadStatus::NeedsReload);
    }

    #[tokio::test]
    async fn test_stale_handlers_returns_cold_after_timeout() {
        let mut reg = HandlerRegistry::new();
        reg.register("cold_handler", LoadTier::Cold);
        reg.register("hot_handler", LoadTier::Hot);
        reg.record_access("cold_handler");
        reg.record_access("hot_handler");
        tokio::time::sleep(Duration::from_millis(20)).await;
        let stale = reg.stale_handlers(Duration::from_millis(15));
        assert!(
            stale.contains(&"cold_handler".to_string()),
            "cold handler should be stale"
        );
        assert!(
            !stale.contains(&"hot_handler".to_string()),
            "hot handler should not be stale"
        );
    }

    #[test]
    fn test_stats_returns_correct_counts() {
        let mut reg = HandlerRegistry::new();
        reg.register("h1", LoadTier::Hot);
        reg.register("h2", LoadTier::Hot);
        reg.register("w1", LoadTier::Warm);
        reg.register("c1", LoadTier::Cold);
        reg.register("c2", LoadTier::Cold);
        let stats = reg.stats();
        assert_eq!(stats.hot, 2);
        assert_eq!(stats.warm, 1);
        assert_eq!(stats.cold, 2);
        assert_eq!(stats.total, 5);
    }

    #[test]
    fn test_mark_unloaded_clears_last_access() {
        let mut reg = HandlerRegistry::new();
        reg.register("test", LoadTier::Cold);
        reg.record_access("test");
        reg.mark_unloaded("test");
        let stale = reg.stale_handlers(Duration::from_secs(0));
        assert!(
            !stale.contains(&"test".to_string()),
            "unloaded handler should not be stale"
        );
    }

    #[test]
    fn test_total_calls_aggregates_correctly() {
        let mut reg = HandlerRegistry::new();
        reg.register("a", LoadTier::Hot);
        reg.register("b", LoadTier::Warm);
        reg.record_access("a");
        reg.record_access("a");
        reg.record_access("a");
        reg.record_access("b");
        assert_eq!(reg.total_calls(), 4);
    }

    #[test]
    fn test_default_handler_tiers_has_at_least_50_entries() {
        let tiers = default_handler_tiers();
        assert!(
            tiers.len() >= 50,
            "expected at least 50 entries, got {}",
            tiers.len()
        );
    }

    #[test]
    fn test_record_access_increments_load_count() {
        let mut reg = HandlerRegistry::new();
        reg.register("c", LoadTier::Cold);
        reg.record_access("c");
        reg.record_access("c");
        reg.record_access("c");
        assert_eq!(*reg.load_counts.get("c").unwrap(), 3);
    }

    #[test]
    fn test_register_many_bulk() {
        let mut reg = HandlerRegistry::new();
        reg.register_many(&[("x", LoadTier::Hot), ("y", LoadTier::Cold)]);
        assert_eq!(reg.tier("x"), LoadTier::Hot);
        assert_eq!(reg.tier("y"), LoadTier::Cold);
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_cold_handler_always_needs_init() {
        let mut reg = HandlerRegistry::new();
        reg.register("c", LoadTier::Cold);
        assert_eq!(reg.record_access("c"), LoadStatus::NeedsInit);
        assert_eq!(reg.record_access("c"), LoadStatus::NeedsInit);
    }

    #[test]
    fn test_handler_names_returns_all_registered() {
        let mut reg = HandlerRegistry::new();
        reg.register("a", LoadTier::Hot);
        reg.register("b", LoadTier::Warm);
        let mut names = reg.handler_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
