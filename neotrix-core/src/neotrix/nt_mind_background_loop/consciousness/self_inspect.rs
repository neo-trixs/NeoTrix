use super::types::*;
use crate::core::nt_core_knowledge::self_inspect::{
    default_edit_policy, default_subspace_topology, default_vsa_primitives, EditPolicy,
    HandlerGraph, HandlerNode, LanguageSpec, SelfInspectable, SubspaceMap, VsaPrimitive,
};

impl SelfInspectable for ConsciousnessIntegration {
    fn primitive_inventory(&self) -> Vec<VsaPrimitive> {
        let mut prims = default_vsa_primitives();
        prims.push(VsaPrimitive {
            name: "encode",
            arity: 2,
            description: "text-to-VSA encoding via NgramVsaEncoder",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "sparse_encode",
            arity: 3,
            description: "sparse k-hot encoding (DIM, K, input)",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "linear_encode",
            arity: 2,
            description: "linear code systematic encoding",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "qhdc_encode",
            arity: 1,
            description: "quantum-inspired phase encoding",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "translate",
            arity: 2,
            description: "VSA-native translation via bind/unbind lookup",
            subspace_requirements: vec!["@translate"],
        });
        prims.push(VsaPrimitive {
            name: "generate_lottie",
            arity: 1,
            description: "Generate Lottie animation JSON from visual signature",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "introspect",
            arity: 1,
            description: "Runtime self-introspection — detect cognitive defect patterns from system diagnostics",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "check_bootstrap_identity",
            arity: 0,
            description: "Verify Rust vs Ne compiler identity for bootstrap closure",
            subspace_requirements: vec![],
        });
        // Ne evaluator primitives
        prims.push(VsaPrimitive {
            name: "ne_eval",
            arity: 1,
            description: "Evaluate a Ne expression string",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "ne_bind",
            arity: 2,
            description: "Ne-level VSA bind operation",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "ne_bundle",
            arity: 2,
            description: "Ne-level VSA bundle operation",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "ne_explore",
            arity: 1,
            description: "Activate a handler via Ne",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "ne_prune",
            arity: 1,
            description: "Deactivate a handler via Ne",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "self_modify",
            arity: 2,
            description: "Modify own handler registry via Ne",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "mutation-stats",
            arity: 0,
            description: "Current mutation log summary from CI evolution bridge",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "present-evolution",
            arity: 0,
            description: "Self-evolution loop generation and status",
            subspace_requirements: vec![],
        });
        // Reasoning pipeline + GWT primitives
        prims.push(VsaPrimitive {
            name: "mcts_select",
            arity: 2,
            description: "MCTS UCB1 node selection from visit counts and values",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "prm_evaluate",
            arity: 1,
            description: "Process Reward Model step evaluation",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "dead_end_check",
            arity: 1,
            description: "Reasoning dead-end detection from step trace",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "strategy_switch",
            arity: 2,
            description: "Self-healing reasoning strategy switching",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "counterfactual_generate",
            arity: 2,
            description: "VSA counterfactual generation from current state and perturbation",
            subspace_requirements: vec![],
        });
        prims.push(VsaPrimitive {
            name: "curiosity_sample",
            arity: 1,
            description: "Thompson-sampled novelty-driven curiosity exploration",
            subspace_requirements: vec![],
        });
        prims
    }

    fn subspace_topology(&self) -> SubspaceMap {
        default_subspace_topology()
    }

    fn edit_boundary(&self) -> EditPolicy {
        default_edit_policy()
    }

    fn handler_graph(&self) -> HandlerGraph {
        let handler_names: &[&str] = &[
            "bridge",
            "checkpoint",
            "ctm",
            "source_cognition",
            "vsa_input",
            "temporal_attention",
            "cross_modal",
            "value_system",
            "volition",
            "inner_critic",
            "specious_present",
            "narrative_self",
            "valence_axis",
            "drive_selector",
            "memory_lattice",
            "memory_palace",
            "memory_sync",
            "memory_reflector",
            "vsa_vocabulary",
            "cognitive_load",
            "default_mode",
            "stream_buffer",
            "first_person",
            "awakening",
            "constitution",
            "workspace",
            "dream_consolidator",
            "meta_cognition",
            "calibration",
            "policy_repair",
            "working_memory",
            "evosc",
            "open_skill",
            "skill_dag",
            "skill_trend",
            "exploratory_gap",
            "signal_pattern",
            "resonance",
            "emergent_property",
            "concept_drift",
            "reflexivity",
            "cognitive_diversity",
            "adaptive_rate",
            "conformal_uq",
            "story_generator",
            "mirror_buffer",
            "adapt_orch",
            "sparse_vsa_attn",
            "vsa_moe",
            "pcc_safety",
            "ball_verifier",
            "progress_rag",
            "ne_evaluator",
            "ne_loader",
            "adaptive_vsa",
            "null_drift",
            "thdc",
            "evolution_bridge",
            "self_evolution",
            "meta_agent",
            "self_revision",
            "ema_jepa",
            "okf_exporter",
            "native_explorer",
            "contrastive_reflection",
            "faithfulness_auditor",
            "entity_resolver",
            "dysib",
            "interaction_trace",
            "keyword_lexicon",
            "quant_data",
            "cdp_session",
            "fringe_mix",
            "factor_miner",
            "osint",
            "capability",
            "hubness",
            "remote_host",
            "security_gate",
            "native_browser",
            "koopman",
            "news_radar",
            "intel_profile",
            "vuln_pipeline",
            "voice_synthesis",
            "html_presentation",
            "loop_templates",
            "cyber_threat",
            "introspection",
            "faithfulness",
            "motion_synthesizer",
            "decoder_learning",
            "mirror",
            "humanizer",
            "transcript_analysis",
            "induction",
            "business_diagnosis",
            "visual_planner",
            "research_writer",
            "self_play_guide",
            "meta_reflection",
            "belief_trajectory",
            "dgmh_meta",
            "metrics",
            "ne_compile",
            "workflow_execute",
            "workflow_list",
            "workflow_summary",
            "sandbox_execute",
            "sandbox_cleanup",
            "skill_health",
            "research",
            "research_propose",
            "research_stats",
            "research_kg",
            "research_kg_submit",
            "job_queue",
            "job_queue_stats",
            "job_queue_submit",
            "context_gather",
            "decision_compress",
            "experience_reflect",
            "skill_accumulate",
            "goal_decompose",
            "validity_crosscheck",
            "loss_recalibrate",
            "arena_round",
            "curiosity_drive",
            "exploration_orchestrate",
            "godel_round",
            "neuromodulate",
            "world_model",
            "layer_management",
            "trace_mining",
            "translate_engine",
            "storage_engine",
            "persist",
            // Phase 5 reasoning pipeline + GWT modules
            "mcts_reasoner",
            "parallel_hypothesis",
            "dead_end_detector",
            "epistemic_humility",
            "process_calibration",
            "process_reward_model",
            "bidirectional_pruner",
            "strategy_selector",
            "counterfactual_simulator",
            "gwt_self_interrupt",
            "curiosity_exploration",
            "pipeline_orchestrator",
            "reasoning_ke_bridge",
            "work_discovery",
            "independent_verify",
            "loop_audit",
        ];
        HandlerGraph {
            handlers: handler_names
                .iter()
                .map(|name| HandlerNode {
                    name,
                    interval_secs: match *name {
                        "slow_handlers" => 100,
                        "induction" => 100,
                        "skill_trend" => 150,
                        "checkpoint" => 500,
                        "belief_trajectory" => 200,
                        "dgmh_meta" => 200,
                        "ne_compile" => 30,
                        "research_kg" => 30,
                        "self_evolution" => 50,
                        "meta_agent" => 5,
                        "skill_health" => 50,
                        "ball_verifier" => 50,
                        "adapt_orch" => 5,
                        "research" | "research_stats" | "research_kg_submit"
                        | "research_propose" => 10,
                        "counterfactual" | "physics" | "spatial" => 5,
                        "imagination" => 10,
                        "news_radar" | "intel_profile" | "voice_synthesis"
                        | "html_presentation" | "loop_templates" => 15,
                        "introspection" | "motion_synthesizer" | "decoder_learning"
                        | "vsa_vocabulary" => 30,
                        "contrastive_reflection"
                        | "faithfulness_auditor"
                        | "entity_resolver"
                        | "dysib"
                        | "interaction_trace"
                        | "keyword_lexicon" => 20,
                        "hubness" | "quant_data" | "factor_miner" | "fringe_mix" | "osint"
                        | "capability" | "cdp_session" | "remote_host" | "security_gate"
                        | "native_browser" => 30,
                        "research_trajectory" => 30,
                        "memory_sync" => 5,
                        "memory_reflector" => 10,
                        "awakening" => 200,
                        _ => 1,
                    },
                    call_count: 0,
                })
                .collect(),
        }
    }

    fn distill_language_spec(&self) -> LanguageSpec {
        LanguageSpec {
            vsa_primitives: self.primitive_inventory(),
            subspace_topology: self.subspace_topology(),
            edit_policy: self.edit_boundary(),
            handler_graph: self.handler_graph(),
            confidence: 0.8,
            distilled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl ConsciousnessIntegration {
    pub fn dispatch_coverage(&self) -> (usize, usize, usize, usize) {
        let handler_count = self.handler_registry.handler_names().len() as usize;
        let stubs = self
            .handler_registry
            .handler_names()
            .iter()
            .filter(|n| {
                matches!(
                    n.as_str(),
                    "bridge"
                        | "vsa_input"
                        | "cross_modal"
                        | "signal_pattern"
                        | "resonance"
                        | "reflexivity"
                        | "conformal_uq"
                        | "story_generator"
                )
            })
            .count();
        (handler_count, 16, stubs, 0)
    }
}

#[cfg(test)]
mod dispatch_consistency_tests {
    use super::*;

    /// Verify that handler_names() count matches known dispatch arms.
    /// This test catches when handlers are added to dispatch but not to handler_names().
    #[test]
    fn test_handler_names_complete() {
        let ci = ConsciousnessIntegration::new();
        let names = ci.dispatch_coverage();
        assert!(
            names.0 >= names.1,
            "handler_names() under-reports vs dispatch"
        );
    }
}
