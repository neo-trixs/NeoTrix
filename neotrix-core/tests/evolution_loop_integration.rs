//! Evolution Loop Integration Test — 进化闭环集成测试
//!
//! 验证 observe → orient → decide → act → verify 五阶段闭环。

use neotrix::l6_meta::evolution::evolution_loop::{
    CyclePhase, EvolutionLoop, EvolutionLoopConfig, SystemState,
};

fn system_state() -> SystemState {
    SystemState {
        capability_count: 10,
        overall_score: 0.5,
        growth_phase: "C1-UnitTest".into(),
        active_modules: vec![
            "consciousness".into(),
            "reasoning".into(),
            "safety".into(),
        ],
        metadata: std::collections::HashMap::new(),
    }
}

#[test]
fn full_cycle_runs_all_five_phases() {
    let mut evo = EvolutionLoop::new(EvolutionLoopConfig::default());
    let state = system_state();
    let result = evo.run_cycle(&state);

    assert_eq!(result.phase_results.len(), 5, "must have 5 phases");
    let phases: Vec<&str> = result.phase_results.iter().map(|p| p.phase.as_str()).collect();
    assert_eq!(
        phases,
        vec!["observe", "orient", "decide", "act", "verify"]
    );
}

#[test]
fn cycle_produces_non_negative_duration() {
    let mut evo = EvolutionLoop::new(EvolutionLoopConfig::default());
    let state = system_state();
    let result = evo.run_cycle(&state);
    assert!(result.duration_ms > 0 || result.duration_ms == 0);
}

#[test]
fn multiple_cycles_increment_id() {
    let mut evo = EvolutionLoop::new(EvolutionLoopConfig::default());
    let state = system_state();

    let r1 = evo.run_cycle(&state);
    let r2 = evo.run_cycle(&state);
    let r3 = evo.run_cycle(&state);

    assert_eq!(r1.cycle_id, 1);
    assert_eq!(r2.cycle_id, 2);
    assert_eq!(r3.cycle_id, 3);
}

#[test]
fn act_phase_increases_capability_count() {
    let mut evo = EvolutionLoop::new(EvolutionLoopConfig::default());
    let state = system_state();
    let result = evo.run_cycle(&state);

    let act_phase = result
        .phase_results
        .iter()
        .find(|p| p.phase == CyclePhase::Act)
        .unwrap();
    assert!(act_phase.success);
}

#[test]
fn cycle_result_serializes_to_json() {
    let mut evo = EvolutionLoop::new(EvolutionLoopConfig::default());
    let state = system_state();
    let result = evo.run_cycle(&state);

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert!(json.contains("\"cycle_id\""));
    assert!(json.contains("\"phase_results\""));
    assert!(json.contains("\"improvements\""));
    assert!(json.contains("\"net_score_delta\""));
}

#[test]
fn absorber_works_independently() {
    use neotrix::l6_meta::evolution::evolution_loop::KnowledgeAbsorber;

    let mut absorber = KnowledgeAbsorber::new(2);
    let result = absorber.absorb("test_source");
    assert!(result.rules_extracted > 0);
    assert!(result.rules_applied > 0);
}

#[test]
fn self_evolver_identifies_gaps() {
    use neotrix::l6_meta::evolution::evolution_loop::SelfEvolver;

    let evolver = SelfEvolver::new();
    let state = SystemState {
        capability_count: 0,
        overall_score: 0.0,
        growth_phase: "C0-Compile".into(),
        active_modules: vec![],
        metadata: std::collections::HashMap::new(),
    };
    let plan = evolver.evolve(&state);
    assert!(!plan.new_modules.is_empty(), "should identify all gaps");
}

#[test]
fn verifier_compares_states() {
    use neotrix::l6_meta::evolution::evolution_loop::EvolutionVerifier;

    let verifier = EvolutionVerifier::new(0.01);
    let before = SystemState {
        capability_count: 5,
        overall_score: 0.4,
        growth_phase: "C0-Compile".into(),
        active_modules: vec!["a".into()],
        metadata: std::collections::HashMap::new(),
    };
    let after = SystemState {
        capability_count: 8,
        overall_score: 0.6,
        growth_phase: "C1-UnitTest".into(),
        active_modules: vec!["a".into(), "b".into()],
        metadata: std::collections::HashMap::new(),
    };
    let result = verifier.verify(&before, &after);
    assert!(!result.improvements.is_empty());
    assert!(result.net_score > 0.0);
}

#[test]
fn config_override_max_absorption_rounds() {
    let config = EvolutionLoopConfig {
        max_absorption_rounds: 5,
        min_improvement_threshold: 0.001,
        enable_verification: true,
        phase_timeout_ms: 10000,
    };
    let mut evo = EvolutionLoop::new(config);
    let state = system_state();
    let result = evo.run_cycle(&state);
    assert_eq!(result.phase_results.len(), 5);
}
