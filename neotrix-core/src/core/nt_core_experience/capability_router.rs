use std::collections::HashMap;

use super::handler_tier::LoadTier;

/// Capability OID — typed identifier for what a handler provides
pub type CapabilityOid = &'static str;

/// Route entry mapping capability OID to handler dispatch name
pub struct RouteEntry {
    pub oid: CapabilityOid,
    pub handler_name: &'static str,
    pub tier: LoadTier,
}

/// RoutingTable — replaces flat string-match dispatch with OID-based routing
pub struct CapabilityRouter {
    /// oid → handler_name mapping
    table: HashMap<&'static str, &'static str>,
    /// reverse: handler_name → oid(s)
    reverse: HashMap<&'static str, Vec<&'static str>>,
    /// ordered list of OIDs for dispatch priority
    order: Vec<&'static str>,
}

impl CapabilityRouter {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            reverse: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn register(&mut self, oid: CapabilityOid, handler_name: &'static str, _tier: LoadTier) {
        if self.table.contains_key(oid) {
            return;
        }
        self.table.insert(oid, handler_name);
        self.reverse.entry(handler_name).or_default().push(oid);
        self.order.push(oid);
    }

    pub fn register_many(&mut self, entries: &[RouteEntry]) {
        for entry in entries {
            self.register(entry.oid, entry.handler_name, entry.tier);
        }
    }

    pub fn resolve(&self, oid: &str) -> Option<&'static str> {
        self.table.get(oid).copied()
    }

    pub fn oids_for_handler(&self, handler_name: &str) -> Option<&[&'static str]> {
        self.reverse.get(handler_name).map(|v| v.as_slice())
    }

    /// Hot-swap handler for a given OID
    pub fn substitute(
        &mut self,
        oid: &'static str,
        new_handler: &'static str,
    ) -> Result<bool, String> {
        if !self.table.contains_key(oid) {
            return Ok(false);
        }
        let old = self.table.get(oid).copied().ok_or_else(|| {
            format!(
                "substitute: oid '{}' missing despite contains_key check",
                oid
            )
        })?;
        if old == new_handler {
            return Ok(true);
        }
        self.table.insert(oid, new_handler);

        if let Some(oids) = self.reverse.get_mut(old) {
            oids.retain(|o| *o != oid);
        }
        self.reverse.entry(new_handler).or_default().push(oid);
        Ok(true)
    }

    /// Dispatch through a closure — the closure receives the resolved handler name
    pub fn dispatch<F>(&self, oid: &str, mut dispatch_fn: F) -> Option<String>
    where
        F: FnMut(&str) -> String,
    {
        let handler_name = self.table.get(oid)?;
        Some(dispatch_fn(handler_name))
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Unique handler names
    pub fn handler_names(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.reverse.keys().copied().collect();
        names.sort();
        names
    }
}

pub fn default_capability_routes() -> Vec<RouteEntry> {
    vec![
        // ── consciousness.core.* ──
        RouteEntry {
            oid: "consciousness.core.first_person",
            handler_name: "first_person",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.specious_present",
            handler_name: "specious_present",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.narrative_self",
            handler_name: "narrative_self",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.stream_buffer",
            handler_name: "stream_buffer",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.awakening",
            handler_name: "awakening",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.volition",
            handler_name: "volition",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.inner_critic",
            handler_name: "inner_critic",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.cognitive_load",
            handler_name: "cognitive_load",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.default_mode",
            handler_name: "default_mode",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.valence_axis",
            handler_name: "valence_axis",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.source_cognition",
            handler_name: "source_cognition",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.vsa_input",
            handler_name: "vsa_input",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.bridge",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.core.ctm",
            handler_name: "ctm",
            tier: LoadTier::Cold,
        },
        // ── consciousness.attention.* ──
        RouteEntry {
            oid: "consciousness.attention.temporal",
            handler_name: "temporal_attention",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.attention.cross_modal",
            handler_name: "cross_modal",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.attention.mirror_buffer",
            handler_name: "mirror_buffer",
            tier: LoadTier::Warm,
        },
        // ── consciousness.value.* ──
        RouteEntry {
            oid: "consciousness.value.system",
            handler_name: "value_system",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.value.alignment",
            handler_name: "value_system",
            tier: LoadTier::Warm,
        },
        // ── consciousness.meta.* ──
        RouteEntry {
            oid: "consciousness.meta.introspection",
            handler_name: "introspection",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.meta.meta_cognition",
            handler_name: "meta_cognition",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.meta.calibration",
            handler_name: "calibration",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.meta.mirror",
            handler_name: "mirror",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "consciousness.meta.ne_evaluator",
            handler_name: "ne_evaluator",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "consciousness.meta.adaptive_vsa",
            handler_name: "adaptive_vsa",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "consciousness.meta.adapt_orch",
            handler_name: "adapt_orch",
            tier: LoadTier::Warm,
        },
        // ── consciousness.safety.* ──
        RouteEntry {
            oid: "consciousness.safety.gate",
            handler_name: "safety_gate",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.safety.pcc",
            handler_name: "pcc_safety",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.safety.ball_verifier",
            handler_name: "ball_verifier",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.safety.edit",
            handler_name: "safety_gate",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "consciousness.safety.security_gate",
            handler_name: "security_gate",
            tier: LoadTier::Hot,
        },
        // ── consciousness.neuro.* ──
        RouteEntry {
            oid: "consciousness.neuro.neuromodulator",
            handler_name: "neuromodulate",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "consciousness.neuro.emergent_reasoning",
            handler_name: "emergent_reasoning",
            tier: LoadTier::Warm,
        },
        // ── consciousness.sleep.* ──
        RouteEntry {
            oid: "consciousness.sleep.dream_consolidator",
            handler_name: "dream_consolidator",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "consciousness.sleep.consolidation",
            handler_name: "dream_consolidator",
            tier: LoadTier::Cold,
        },
        // ── experience.* ──
        RouteEntry {
            oid: "experience.evosc",
            handler_name: "evosc",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "experience.open_skill",
            handler_name: "open_skill",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "experience.skill_dag",
            handler_name: "skill_dag",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "experience.failure_classifier",
            handler_name: "failure_classifier",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.failure_trace",
            handler_name: "exploratory_gap",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.curriculum",
            handler_name: "calibration",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.reflection",
            handler_name: "contrastive_reflection",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.faithfulness",
            handler_name: "faithfulness_auditor",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.reflexivity",
            handler_name: "reflexivity",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.evolution_bridge",
            handler_name: "evolution_bridge",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.self_evolution",
            handler_name: "self_evolution",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.self_revision",
            handler_name: "self_revision",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.native_explorer",
            handler_name: "native_explorer",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "experience.okf_exporter",
            handler_name: "okf_exporter",
            tier: LoadTier::Cold,
        },
        // ── experience.detection.* ──
        RouteEntry {
            oid: "experience.detection.signal_pattern",
            handler_name: "signal_pattern",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.detection.resonance",
            handler_name: "resonance",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.detection.emergent_property",
            handler_name: "emergent_property",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.detection.concept_drift",
            handler_name: "concept_drift",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.detection.cognitive_diversity",
            handler_name: "cognitive_diversity",
            tier: LoadTier::Warm,
        },
        // ── experience.analysis.* ──
        RouteEntry {
            oid: "experience.analysis.adaptive_rate",
            handler_name: "adaptive_rate",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.conformal_uq",
            handler_name: "conformal_uq",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.story_generator",
            handler_name: "story_generator",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.humanizer",
            handler_name: "humanizer",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.business_diagnosis",
            handler_name: "business_diagnosis",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.visual_planner",
            handler_name: "visual_planner",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.research_writer",
            handler_name: "research_writer",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "experience.analysis.self_play_guide",
            handler_name: "self_play_guide",
            tier: LoadTier::Warm,
        },
        // ── knowledge.* ──
        RouteEntry {
            oid: "knowledge.entity_resolver",
            handler_name: "entity_resolver",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "knowledge.keyword_lexicon",
            handler_name: "keyword_lexicon",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "knowledge.fringe_mix",
            handler_name: "fringe_mix",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "knowledge.hubness",
            handler_name: "hubness",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "knowledge.progress_rag",
            handler_name: "progress_rag",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "knowledge.dysib",
            handler_name: "dysib",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "knowledge.null_drift",
            handler_name: "null_drift",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "knowledge.thdc",
            handler_name: "thdc",
            tier: LoadTier::Cold,
        },
        // ── agent.* ──
        RouteEntry {
            oid: "agent.adversarial_arena",
            handler_name: "adversarial_arena",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "agent.hyperagent",
            handler_name: "hyperagent",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "agent.mcp_intel",
            handler_name: "mcp_intel",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "agent.remote_host",
            handler_name: "remote_host",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "agent.browser_mcp",
            handler_name: "browser_mcp",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "agent.cdp_session",
            handler_name: "cdp_session",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "agent.quant_data",
            handler_name: "quant_data",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "agent.factor_miner",
            handler_name: "factor_miner",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "agent.osint",
            handler_name: "osint",
            tier: LoadTier::Cold,
        },
        RouteEntry {
            oid: "agent.koopman",
            handler_name: "koopman",
            tier: LoadTier::Cold,
        },
        // ── shield.* ──
        RouteEntry {
            oid: "shield.pcc_safety",
            handler_name: "pcc_safety",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "shield.ball_verifier",
            handler_name: "ball_verifier",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "shield.security_gate",
            handler_name: "security_gate",
            tier: LoadTier::Hot,
        },
        // ── module.* (Phase 55-58 external modules) ──
        RouteEntry {
            oid: "module.news_radar",
            handler_name: "news_radar",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "module.voice_synthesis",
            handler_name: "voice_synthesis",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "module.html_presentation",
            handler_name: "html_presentation",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "module.loop_templates",
            handler_name: "loop_templates",
            tier: LoadTier::Warm,
        },
        RouteEntry {
            oid: "module.cyber_threat",
            handler_name: "cyber_threat",
            tier: LoadTier::Warm,
        },
        // ── pipeline.* (core pipeline steps, mapped to existing handlers) ──
        RouteEntry {
            oid: "pipeline.context_gather",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "pipeline.decision_compress",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "pipeline.experience_reflect",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "pipeline.skill_accumulate",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "pipeline.goal_decompose",
            handler_name: "bridge",
            tier: LoadTier::Hot,
        },
        // ── health.* ──
        RouteEntry {
            oid: "health.health_patrol",
            handler_name: "health_patrol",
            tier: LoadTier::Hot,
        },
        RouteEntry {
            oid: "health.ema_jepa",
            handler_name: "ema_jepa",
            tier: LoadTier::Cold,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve() {
        let mut router = CapabilityRouter::new();
        router.register("test.one", "handler_a", LoadTier::Hot);
        assert_eq!(router.resolve("test.one"), Some("handler_a"));
        assert_eq!(router.resolve("test.missing"), None);
    }

    #[test]
    fn test_register_many_batch() {
        let mut router = CapabilityRouter::new();
        let entries = vec![
            RouteEntry {
                oid: "alpha",
                handler_name: "h1",
                tier: LoadTier::Hot,
            },
            RouteEntry {
                oid: "beta",
                handler_name: "h2",
                tier: LoadTier::Warm,
            },
            RouteEntry {
                oid: "gamma",
                handler_name: "h1",
                tier: LoadTier::Cold,
            },
        ];
        router.register_many(&entries);
        assert_eq!(router.len(), 3);
        assert_eq!(router.resolve("alpha"), Some("h1"));
        assert_eq!(router.resolve("beta"), Some("h2"));
        assert_eq!(router.resolve("gamma"), Some("h1"));
    }

    #[test]
    fn test_substitute_hot_swap() {
        let mut router = CapabilityRouter::new();
        router.register("test.one", "old_handler", LoadTier::Hot);
        assert_eq!(router.resolve("test.one"), Some("old_handler"));

        let swapped = router.substitute("test.one", "new_handler").unwrap();
        assert!(swapped);
        assert_eq!(router.resolve("test.one"), Some("new_handler"));
    }

    #[test]
    fn test_substitute_unregistered_oid_returns_false() {
        let mut router = CapabilityRouter::new();
        let swapped = router.substitute("does_not_exist", "anything").unwrap();
        assert!(!swapped);
    }

    #[test]
    fn test_reverse_lookup() {
        let mut router = CapabilityRouter::new();
        router.register("test.a", "handler_x", LoadTier::Hot);
        router.register("test.b", "handler_x", LoadTier::Warm);
        router.register("test.c", "handler_y", LoadTier::Cold);

        let oids = router.oids_for_handler("handler_x").unwrap();
        assert_eq!(oids.len(), 2);
        assert!(oids.contains(&"test.a"));
        assert!(oids.contains(&"test.b"));

        assert!(router.oids_for_handler("handler_z").is_none());
    }

    #[test]
    fn test_dispatch_through_closure() {
        let mut router = CapabilityRouter::new();
        router.register("test.echo", "echo_handler", LoadTier::Hot);

        let result = router.dispatch("test.echo", |name| format!("dispatched:{}", name));
        assert_eq!(result, Some("dispatched:echo_handler".to_string()));

        let missing = router.dispatch("test.missing", |name| format!("should_not_run:{}", name));
        assert_eq!(missing, None);
    }

    #[test]
    fn test_default_capability_routes_count_at_least_80() {
        let routes = default_capability_routes();
        assert!(
            routes.len() >= 80,
            "expected >= 80 routes, got {}",
            routes.len()
        );
    }

    #[test]
    fn test_len_matches_registration_count() {
        let mut router = CapabilityRouter::new();
        assert_eq!(router.len(), 0);

        let entries = default_capability_routes();
        let count = entries.len();
        router.register_many(&entries);
        assert_eq!(router.len(), count);
    }

    #[test]
    fn test_handler_names_uniqueness() {
        let mut router = CapabilityRouter::new();
        router.register("a", "handler_x", LoadTier::Hot);
        router.register("b", "handler_x", LoadTier::Warm);
        router.register("c", "handler_y", LoadTier::Cold);

        let names = router.handler_names();
        let expected = vec!["handler_x", "handler_y"];
        assert_eq!(names, expected);
    }

    #[test]
    fn test_register_duplicate_oid_ignored() {
        let mut router = CapabilityRouter::new();
        router.register("dup.oid", "first", LoadTier::Hot);
        router.register("dup.oid", "second", LoadTier::Warm);
        assert_eq!(router.len(), 1);
        assert_eq!(router.resolve("dup.oid"), Some("first"));
    }

    #[test]
    fn test_substitute_updates_reverse_index() {
        let mut router = CapabilityRouter::new();
        router.register("swapper", "old_h", LoadTier::Hot);
        assert!(router.oids_for_handler("old_h").is_some());

        router.substitute("swapper", "new_h").unwrap();
        assert!(router.oids_for_handler("old_h").unwrap().is_empty());
        assert!(router
            .oids_for_handler("new_h")
            .unwrap()
            .contains(&"swapper"));
    }

    #[test]
    fn test_default_routes_all_oid_unique() {
        let routes = default_capability_routes();
        let mut seen = std::collections::HashSet::new();
        for entry in &routes {
            assert!(seen.insert(entry.oid), "duplicate OID: {}", entry.oid);
        }
    }

    #[test]
    fn test_default_routes_all_handler_name_in_known_set() {
        let known_handlers: std::collections::HashSet<&str> = [
            "first_person",
            "specious_present",
            "narrative_self",
            "stream_buffer",
            "awakening",
            "volition",
            "inner_critic",
            "cognitive_load",
            "default_mode",
            "valence_axis",
            "source_cognition",
            "vsa_input",
            "bridge",
            "ctm",
            "temporal_attention",
            "cross_modal",
            "mirror_buffer",
            "value_system",
            "introspection",
            "meta_cognition",
            "calibration",
            "mirror",
            "ne_evaluator",
            "adaptive_vsa",
            "adapt_orch",
            "safety_gate",
            "pcc_safety",
            "ball_verifier",
            "security_gate",
            "neuromodulate",
            "emergent_reasoning",
            "dream_consolidator",
            "evosc",
            "open_skill",
            "skill_dag",
            "failure_classifier",
            "exploratory_gap",
            "contrastive_reflection",
            "faithfulness_auditor",
            "reflexivity",
            "evolution_bridge",
            "self_evolution",
            "self_revision",
            "native_explorer",
            "okf_exporter",
            "signal_pattern",
            "resonance",
            "emergent_property",
            "concept_drift",
            "cognitive_diversity",
            "adaptive_rate",
            "conformal_uq",
            "story_generator",
            "humanizer",
            "business_diagnosis",
            "visual_planner",
            "research_writer",
            "self_play_guide",
            "entity_resolver",
            "keyword_lexicon",
            "fringe_mix",
            "hubness",
            "progress_rag",
            "dysib",
            "null_drift",
            "thdc",
            "adversarial_arena",
            "hyperagent",
            "mcp_intel",
            "remote_host",
            "browser_mcp",
            "cdp_session",
            "quant_data",
            "factor_miner",
            "osint",
            "koopman",
            "news_radar",
            "voice_synthesis",
            "html_presentation",
            "loop_templates",
            "cyber_threat",
            "health_patrol",
            "ema_jepa",
        ]
        .into_iter()
        .collect();

        let routes = default_capability_routes();
        for entry in &routes {
            assert!(
                known_handlers.contains(entry.handler_name),
                "unexpected handler name '{}' for OID '{}'",
                entry.handler_name,
                entry.oid
            );
        }
    }

    #[test]
    fn test_is_empty() {
        let router = CapabilityRouter::new();
        assert!(router.is_empty());

        let mut router2 = CapabilityRouter::new();
        router2.register("foo", "bar", LoadTier::Hot);
        assert!(!router2.is_empty());
    }

    #[test]
    fn test_substitute_same_handler() {
        let mut router = CapabilityRouter::new();
        router.register("same", "handler_x", LoadTier::Hot);
        assert!(router.substitute("same", "handler_x").unwrap());
        assert_eq!(router.resolve("same"), Some("handler_x"));
    }
}
