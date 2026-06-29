#[derive(Debug, Clone, serde::Serialize)]
pub struct HandlerInfo {
    pub name: String,
    pub phase: String,
    pub called: bool,
    pub call_count: u64,
}

impl<'de> serde::Deserialize<'de> for HandlerInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // HandlerInfo is only deserialized for diagnostics; name/phase are empty
        #[derive(serde::Deserialize)]
        struct Info {
            called: bool,
            call_count: u64,
        }
        let info = Info::deserialize(deserializer)?;
        Ok(Self {
            name: String::new(),
            phase: String::new(),
            called: info.called,
            call_count: info.call_count,
        })
    }
}

impl HandlerInfo {
    pub fn new(name: &str, phase: &str) -> Self {
        Self {
            name: name.to_string(),
            phase: phase.to_string(),
            called: false,
            call_count: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandlerDiscovery {
    pub handlers: Vec<HandlerInfo>,
    pub coverage_history: Vec<f64>,
}

impl HandlerDiscovery {
    pub fn new() -> Self {
        let handlers = vec![
            HandlerInfo::new("context_gather", "Execute"),
            HandlerInfo::new("decision_compress", "Execute"),
            HandlerInfo::new("experience_reflect", "Execute"),
            HandlerInfo::new("skill_accumulate", "Execute"),
            HandlerInfo::new("curriculum_generate", "Execute"),
            HandlerInfo::new("policy_repair", "Execute"),
            HandlerInfo::new("epistemic_calibrate", "Execute"),
            HandlerInfo::new("attractor_dynamics", "Execute"),
            HandlerInfo::new("ebbinghaus_decay", "Execute"),
            HandlerInfo::new("dream_cycle", "Execute"),
            HandlerInfo::new("emergent_reasoning", "Execute"),
            HandlerInfo::new("reflexive", "Execute"),
            HandlerInfo::new("epistemic_honesty", "Execute"),
            HandlerInfo::new("personality_update", "Execute"),
            HandlerInfo::new("cognitive_state_ingest", "Execute"),
            HandlerInfo::new("master_consciousness_update", "Execute"),
            HandlerInfo::new("vs_advantage_learn", "Execute"),
            HandlerInfo::new("sleep_consolidation", "Execute"),
            HandlerInfo::new("goal_execution", "Execute"),
            HandlerInfo::new("specious_present_feed", "Execute"),
            HandlerInfo::new("narrative_tick", "Execute"),
            HandlerInfo::new("valence_update", "Execute"),
            HandlerInfo::new("inner_critic", "Verify"),
            HandlerInfo::new("cognitive_load_tick", "Decide"),
            HandlerInfo::new("proof_search_tick", "Execute"),
            HandlerInfo::new("dgmh_writeback_tick", "Persist"),
            HandlerInfo::new("self_protection_tick", "Discover"),
            HandlerInfo::new("spatial_scene", "Execute"),
            HandlerInfo::new("physics_reasoning", "Execute"),
            HandlerInfo::new("novelty_detection_tick", "Discover"),
            HandlerInfo::new("tool_discovery_tick", "Discover"),
            HandlerInfo::new("goal_decomposition_tick", "Assign"),
            HandlerInfo::new("episodic_memory_tick", "Execute"),
            HandlerInfo::new("reasoning_step", "Execute"),
            HandlerInfo::new("moss_pipeline", "Execute"),
            HandlerInfo::new("sia_feedback", "Decide"),
            HandlerInfo::new("input_pipeline_batch", "Execute"),
            HandlerInfo::new("ctm_inference", "Execute"),
            HandlerInfo::new("sar_diagnostic_tick", "Execute"),
            HandlerInfo::new("reliability_gate_tick", "Verify"),
        ];
        Self {
            handlers,
            coverage_history: Vec::new(),
        }
    }

    pub fn record_call(&mut self, name: &str) {
        if let Some(h) = self.handlers.iter_mut().find(|h| h.name == name) {
            h.called = true;
            h.call_count += 1;
        }
    }

    pub fn coverage_report(&self) -> CoverageReport {
        let total = self.handlers.len();
        let called = self.handlers.iter().filter(|h| h.called).count();
        let pct = if total > 0 {
            (called as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let uncalled: Vec<String> = self
            .handlers
            .iter()
            .filter(|h| !h.called)
            .map(|h| h.name.clone())
            .collect();

        CoverageReport {
            total,
            called,
            coverage_pct: pct,
            uncalled,
        }
    }

    pub fn uncalled_handlers(&self) -> Vec<&HandlerInfo> {
        self.handlers.iter().filter(|h| !h.called).collect()
    }
}

pub struct CoverageReport {
    pub total: usize,
    pub called: usize,
    pub coverage_pct: f64,
    pub uncalled: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_new() {
        let d = HandlerDiscovery::new();
        assert!(d.handlers.len() > 30);
        assert!(d.coverage_history.is_empty());
    }

    #[test]
    fn test_discovery_record_call() {
        let mut d = HandlerDiscovery::new();
        d.record_call("context_gather");
        let h = d
            .handlers
            .iter()
            .find(|h| h.name == "context_gather")
            .unwrap();
        assert!(h.called);
        assert_eq!(h.call_count, 1);
    }

    #[test]
    fn test_discovery_coverage_report() {
        let mut d = HandlerDiscovery::new();
        let r = d.coverage_report();
        assert_eq!(r.called, 0);
        assert!(r.coverage_pct < 1.0);

        d.record_call("context_gather");
        let r = d.coverage_report();
        assert_eq!(r.called, 1);
        assert!(r.coverage_pct > 0.0);
    }

    #[test]
    fn test_discovery_uncalled() {
        let d = HandlerDiscovery::new();
        let uncalled = d.uncalled_handlers();
        assert_eq!(uncalled.len(), d.handlers.len());
    }

    #[test]
    fn test_discovery_record_unknown_name() {
        let mut d = HandlerDiscovery::new();
        d.record_call("nonexistent_handler"); // should not panic
        let r = d.coverage_report();
        assert_eq!(r.called, 0);
    }
}
