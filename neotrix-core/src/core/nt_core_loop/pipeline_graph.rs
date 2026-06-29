use std::collections::HashMap;

/// Group classification for consciousness_batch handlers.
/// Determines conditional routing in the pipeline DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NodeGroup {
    Gathering,     // context_gather, decision_encode/compress
    Reflection,    // experience_reflect, skill_accumulate, curriculum, policy_repair
    Calibration,   // epistemic_calibrate, attractor, ebbinghaus, dream
    Consciousness, // emergent, reflexive, epistemic_honesty, personality, cognitive_state, master
    Learning,      // vs_advantage, failure_trace, dream_consolidator
    Synthesis,     // narrative, valence, inner_critic, cognitive_load, proof
    Bridge, // dgmh_writeback, dmn, min_sufficient, stream_buffer, narrative, adaptive, budget, resonator
    HeavyMetric, // CTM, spatial, physics
    Exploration, // novelty, tool_discovery, goal_decomposition, episodic
    Persistence, // archive_save, checkpoint
    Schedule, // volition, conformal, confidence, value, e8_geometry, social
    Meta,   // reasoning_step, moss_pipeline, input_pipeline
}

impl NodeGroup {
    pub fn label(&self) -> &'static str {
        match self {
            NodeGroup::Gathering => "gathering",
            NodeGroup::Reflection => "reflection",
            NodeGroup::Calibration => "calibration",
            NodeGroup::Consciousness => "consciousness",
            NodeGroup::Learning => "learning",
            NodeGroup::Synthesis => "synthesis",
            NodeGroup::Bridge => "bridge",
            NodeGroup::HeavyMetric => "heavy_metric",
            NodeGroup::Exploration => "exploration",
            NodeGroup::Persistence => "persistence",
            NodeGroup::Schedule => "schedule",
            NodeGroup::Meta => "meta",
        }
    }
}

/// Runtime condition snapshot for pipeline routing decisions.
#[derive(Debug, Clone)]
pub struct PipelineConditions {
    pub da_level: f64,
    pub fast_mode: bool,
    pub has_epistemic_gaps: bool,
    pub critique_passed: bool,
    pub cognitive_load: f64,
    pub storm_score: f64,
    pub archive_size: usize,
}

/// Static node definition in the pipeline graph.
#[derive(Debug, Clone)]
pub struct PipelineNodeData {
    pub name: &'static str,
    pub group: NodeGroup,
    pub compute_cost: u8,
}

/// Conditional routing graph for the consciousness_batch pipeline.
///
/// Maps each handler to a NodeGroup. Groups have conditional routing rules:
/// - Core groups (Gathering, Consciousness, Reflection) always run
/// - HeavyMetric / Exploration skip when DA is low
/// - Calibration skips when no epistemic gaps exist
/// - Schedule groups run only periodically
///
/// Tracks execution vs skip counts for observability.
#[derive(Debug, Clone)]
pub struct PipelineGraph {
    nodes: Vec<PipelineNodeData>,
    execution_counts: HashMap<String, u64>,
    skip_counts: HashMap<String, u64>,
    total_cycles: u64,
}

impl PipelineGraph {
    /// Build the static pipeline graph with all registered handlers and their groups.
    pub fn new() -> Self {
        let nodes = Self::default_nodes();
        Self {
            nodes,
            execution_counts: HashMap::new(),
            skip_counts: HashMap::new(),
            total_cycles: 0,
        }
    }

    /// Define all pipeline nodes with their group classification.
    fn default_nodes() -> Vec<PipelineNodeData> {
        vec![
            // Core — always run
            ("context_gather", NodeGroup::Gathering, 3),
            ("decision_compress", NodeGroup::Gathering, 4),
            ("experience_reflect", NodeGroup::Reflection, 3),
            ("skill_accumulate", NodeGroup::Reflection, 3),
            ("curriculum_generate", NodeGroup::Reflection, 3),
            ("policy_repair", NodeGroup::Reflection, 4),
            ("epistemic_calibrate", NodeGroup::Calibration, 3),
            ("attractor_dynamics", NodeGroup::Calibration, 2),
            ("ebbinghaus_decay", NodeGroup::Calibration, 1),
            ("dream_cycle", NodeGroup::Calibration, 3),
            ("emergent_reasoning", NodeGroup::Consciousness, 2),
            ("reflexive", NodeGroup::Consciousness, 1),
            ("epistemic_honesty", NodeGroup::Consciousness, 2),
            ("personality_update", NodeGroup::Consciousness, 1),
            ("cognitive_state_ingest", NodeGroup::Consciousness, 1),
            ("master_consciousness_update", NodeGroup::Consciousness, 2),
            ("vs_advantage_learn", NodeGroup::Learning, 4),
            ("sleep_consolidation", NodeGroup::Learning, 3),
            ("goal_execution", NodeGroup::Learning, 3),
            // Synthesis
            ("specious_present_feed", NodeGroup::Synthesis, 1),
            ("narrative_tick", NodeGroup::Synthesis, 1),
            ("valence_update", NodeGroup::Synthesis, 1),
            ("inner_critic", NodeGroup::Synthesis, 1),
            ("cognitive_load_tick", NodeGroup::Synthesis, 1),
            ("proof_search_tick", NodeGroup::Synthesis, 2),
            // Bridge — orphan handlers
            ("dgmh_writeback_tick", NodeGroup::Bridge, 2),
            ("dmn_reverberate", NodeGroup::Bridge, 2),
            ("stream_buffer_feed", NodeGroup::Bridge, 1),
            ("reconstructive_narrative_tick", NodeGroup::Bridge, 2),
            ("adaptive_rate_tick", NodeGroup::Bridge, 1),
            ("context_budget_tick", NodeGroup::Bridge, 1),
            ("resonator_decode", NodeGroup::Bridge, 2),
            ("self_protection_tick", NodeGroup::Bridge, 1),
            // Heavy metric — DA-gated
            ("ctm_inference", NodeGroup::HeavyMetric, 5),
            ("spatial_scene", NodeGroup::HeavyMetric, 4),
            ("physics_reasoning", NodeGroup::HeavyMetric, 4),
            // Exploration — DA-gated
            ("novelty_detection_tick", NodeGroup::Exploration, 2),
            ("tool_discovery_tick", NodeGroup::Exploration, 3),
            ("goal_decomposition_tick", NodeGroup::Exploration, 3),
            ("episodic_memory_tick", NodeGroup::Exploration, 2),
            // Persistence
            ("archive_save", NodeGroup::Persistence, 1),
            // Schedule — periodic
            ("volition_tick", NodeGroup::Schedule, 2),
            ("conformal_uq_tick", NodeGroup::Schedule, 2),
            ("confidence_calibrate", NodeGroup::Schedule, 2),
            ("value_system_tick", NodeGroup::Schedule, 2),
            ("value_alignment_tick", NodeGroup::Schedule, 2),
            ("e8_geometry_tick", NodeGroup::Schedule, 3),
            ("social_feed_absorb", NodeGroup::Schedule, 2),
            // Diagnostics
            ("sar_diagnostic_tick", NodeGroup::Synthesis, 1),
            ("reliability_gate_tick", NodeGroup::Bridge, 1),
            // Meta — periodic
            ("reasoning_step", NodeGroup::Meta, 3),
            ("moss_pipeline", NodeGroup::Meta, 4),
            ("input_pipeline_batch", NodeGroup::Meta, 2),
            ("kroneker_cleanup", NodeGroup::Meta, 2),
            ("uat_gate", NodeGroup::Meta, 2),
            ("dgm_variant_propose", NodeGroup::Meta, 2),
        ]
        .into_iter()
        .map(|(name, group, cost)| PipelineNodeData {
            name,
            group,
            compute_cost: cost,
        })
        .collect()
    }

    /// Check whether a group should run this cycle given current conditions.
    pub fn should_run_group(&self, group: NodeGroup, cond: &PipelineConditions) -> bool {
        match group {
            // Core groups always run
            NodeGroup::Gathering
            | NodeGroup::Reflection
            | NodeGroup::Consciousness
            | NodeGroup::Learning
            | NodeGroup::Synthesis
            | NodeGroup::Bridge => true,

            // Calibration: skip when no epistemic gaps (narrow focused mode)
            NodeGroup::Calibration => cond.has_epistemic_gaps || cond.archive_size > 0,

            // Heavy metric: DA >= 0.3 and not in fast mode
            NodeGroup::HeavyMetric => cond.da_level >= 0.3 && !cond.fast_mode,

            // Exploration: DA >= 0.3
            NodeGroup::Exploration => cond.da_level >= 0.3,

            // Persistence: only when archive is non-empty
            NodeGroup::Persistence => cond.archive_size > 0,

            // Schedule: DA >= 0.3 (low motivation skips planning steps)
            NodeGroup::Schedule => cond.da_level >= 0.3,

            // Meta: always runs (includes cleanup, verification)
            NodeGroup::Meta => true,
        }
    }

    /// Record that handler `name` executed this cycle.
    pub fn record_execution(&mut self, name: &str) {
        *self.execution_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Record that handler `name` was skipped this cycle.
    pub fn record_skip(&mut self, name: &str) {
        *self.skip_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    /// End a cycle.
    pub fn end_cycle(&mut self) {
        self.total_cycles += 1;
    }

    /// Get the group for a named handler.
    pub fn group_for(&self, name: &str) -> Option<NodeGroup> {
        self.nodes.iter().find(|n| n.name == name).map(|n| n.group)
    }

    /// All handler names registered in the graph.
    pub fn all_handlers(&self) -> Vec<&'static str> {
        self.nodes.iter().map(|n| n.name).collect()
    }

    /// Graph statistics for observability.
    pub fn stats(&self) -> GraphStats {
        let total_executions: u64 = self.execution_counts.values().sum();
        let total_skips: u64 = self.skip_counts.values().sum();
        GraphStats {
            node_count: self.nodes.len(),
            total_cycles: self.total_cycles,
            total_executions,
            total_skips,
            skip_ratio: if total_executions + total_skips > 0 {
                total_skips as f64 / (total_executions + total_skips) as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for PipelineGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub total_cycles: u64,
    pub total_executions: u64,
    pub total_skips: u64,
    pub skip_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_handlers_registered() {
        let g = PipelineGraph::new();
        assert!(!g.all_handlers().is_empty());
        assert!(g.all_handlers().contains(&"context_gather"));
        assert!(g.all_handlers().contains(&"ctm_inference"));
        assert!(g.all_handlers().contains(&"archive_save"));
    }

    #[test]
    fn test_core_groups_always_run() {
        let g = PipelineGraph::new();
        let cond = PipelineConditions {
            da_level: 0.1,
            fast_mode: true,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.9,
            storm_score: 0.0,
            archive_size: 0,
        };
        assert!(g.should_run_group(NodeGroup::Gathering, &cond));
        assert!(g.should_run_group(NodeGroup::Reflection, &cond));
        assert!(g.should_run_group(NodeGroup::Consciousness, &cond));
    }

    #[test]
    fn test_heavy_metric_gated_by_da() {
        let g = PipelineGraph::new();
        let low_da = PipelineConditions {
            da_level: 0.1,
            fast_mode: false,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.5,
            storm_score: 0.0,
            archive_size: 5,
        };
        let high_da = PipelineConditions {
            da_level: 0.7,
            fast_mode: false,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.5,
            storm_score: 0.0,
            archive_size: 5,
        };
        assert!(!g.should_run_group(NodeGroup::HeavyMetric, &low_da));
        assert!(g.should_run_group(NodeGroup::HeavyMetric, &high_da));
    }

    #[test]
    fn test_exploration_gated_by_da() {
        let g = PipelineGraph::new();
        let low = PipelineConditions {
            da_level: 0.2,
            fast_mode: false,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.5,
            storm_score: 0.0,
            archive_size: 0,
        };
        let high = PipelineConditions {
            da_level: 0.5,
            fast_mode: false,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.5,
            storm_score: 0.0,
            archive_size: 0,
        };
        assert!(!g.should_run_group(NodeGroup::Exploration, &low));
        assert!(g.should_run_group(NodeGroup::Exploration, &high));
    }

    #[test]
    fn test_heavy_metric_blocked_by_fast_mode() {
        let g = PipelineGraph::new();
        let fast = PipelineConditions {
            da_level: 0.8,
            fast_mode: true,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.5,
            storm_score: 0.0,
            archive_size: 0,
        };
        assert!(!g.should_run_group(NodeGroup::HeavyMetric, &fast));
    }

    #[test]
    fn test_group_for_known_handler() {
        let g = PipelineGraph::new();
        assert_eq!(g.group_for("ctm_inference"), Some(NodeGroup::HeavyMetric));
        assert_eq!(g.group_for("context_gather"), Some(NodeGroup::Gathering));
        assert_eq!(g.group_for("nonexistent"), None);
    }

    #[test]
    fn test_tracking_counts() {
        let mut g = PipelineGraph::new();
        g.record_execution("context_gather");
        g.record_execution("context_gather");
        g.record_skip("ctm_inference");
        g.end_cycle();
        let s = g.stats();
        assert_eq!(s.total_executions, 2);
        assert_eq!(s.total_skips, 1);
        assert!((s.skip_ratio - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_heavy_metric_in_fast_mode_low_da() {
        let g = PipelineGraph::new();
        // Fast mode + low DA = definitely blocked
        let cond = PipelineConditions {
            da_level: 0.1,
            fast_mode: true,
            has_epistemic_gaps: false,
            critique_passed: true,
            cognitive_load: 0.9,
            storm_score: 0.0,
            archive_size: 0,
        };
        assert!(!g.should_run_group(NodeGroup::HeavyMetric, &cond));
        assert!(!g.should_run_group(NodeGroup::Exploration, &cond));
    }
}
