pub mod core;
pub mod types;
pub use core::AdversarialArena;
pub use core::{fast_rng, tournament_pick};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn dummy_score_fn(agent: &AgentGenotype) -> f64 {
        agent.traits.get("speed").copied().unwrap_or(0.5)
            + agent.traits.get("accuracy").copied().unwrap_or(0.5)
    }

    #[test]
    fn test_genotype_creation() {
        let mut traits = HashMap::new();
        traits.insert("speed".into(), 0.8);
        let agent = AgentGenotype::new("test-agent", traits);
        assert_eq!(agent.id, "test-agent");
        assert!((agent.traits.get("speed").copied().unwrap_or(0.0) - 0.8).abs() < 1e-9);
        assert_eq!(agent.generation, 0);
    }

    #[test]
    fn test_mutation() {
        let mut traits = HashMap::new();
        traits.insert("x".into(), 0.5);
        traits.insert("y".into(), 0.5);
        let agent = AgentGenotype::new("parent", traits);
        let mut rng = fast_rng(123);
        let mutant = agent.mutate(&mut rng, 1.0, 0.5);
        assert_ne!(mutant.id, agent.id);
        assert_eq!(mutant.generation, 1);
        assert_eq!(mutant.parent_ids, vec!["parent"]);
    }

    #[test]
    fn test_crossover() {
        let mut t1 = HashMap::new();
        t1.insert("a".into(), 1.0);
        t1.insert("b".into(), 0.0);
        let mut t2 = HashMap::new();
        t2.insert("a".into(), 0.0);
        t2.insert("b".into(), 1.0);
        let parent_a = AgentGenotype::new("p1", t1);
        let parent_b = AgentGenotype::new("p2", t2);
        let mut rng = fast_rng(456);
        let child = parent_a.crossover(&parent_b, &mut rng, 1.0);
        assert_eq!(child.parent_ids.len(), 2);
        assert!(child.generation > 0);
    }

    #[test]
    fn test_arena_creation() {
        let config = ArenaConfig {
            population_size: 10,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        };
        let arena = AdversarialArena::new(config);
        assert!(arena.population.is_empty());
        assert_eq!(arena.generation, 0);
    }

    #[test]
    fn test_seed_population() {
        let mut arena = AdversarialArena::new(ArenaConfig::default());
        arena.seed_population(&["speed", "accuracy"], 0.3);
        assert_eq!(arena.population.len(), 20);
        for agent in &arena.population {
            assert!(agent.traits.contains_key("speed"));
            assert!(agent.traits.contains_key("accuracy"));
            assert!(agent.id.starts_with("gen0-"));
        }
    }

    #[test]
    fn test_run_round() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 10,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        });
        arena.seed_population(&["speed", "accuracy"], 0.3);
        let result = arena.run_round("test-task", &mut dummy_score_fn);
        assert_eq!(arena.history.len(), 1);
        assert!(result.top_fitness > 0.0);
        assert!(result.avg_fitness > 0.0);
    }

    #[test]
    fn test_evolve_generation() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 10,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        });
        arena.seed_population(&["speed", "accuracy"], 0.3);

        for _ in 0..3 {
            arena.run_round("bench", &mut dummy_score_fn);
            arena.evolve(42);
        }

        assert!(arena.history.len() >= 3);
        assert_eq!(arena.generation, 3);
        let report = arena.full_report();
        assert!(report.contains("Adversarial Co-Evolution"));
        assert!(report.contains("Gen 0"));
        assert!(report.contains("Gen 2"));
    }

    #[test]
    fn test_diversity_computation() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 5,
            tournament_size: 2,
            mutation_rate: 0.0,
            crossover_rate: 0.0,
            mutation_sigma: 0.0,
            elite_count: 0,
        });
        arena.seed_population(&["trait"], 0.0);
        let div = arena.compute_diversity();
        assert!(
            (div - 0.0).abs() < 1e-6,
            "identical agents should have zero diversity"
        );
    }

    #[test]
    fn test_empty_arena_round() {
        let mut arena = AdversarialArena::new(ArenaConfig::default());
        let result = arena.run_round("no-ops", &mut dummy_score_fn);
        assert_eq!(result.population_size, 0);
    }

    #[test]
    fn test_trait_bounds() {
        let mut traits = HashMap::new();
        traits.insert("val".into(), 2.5);
        let agent = AgentGenotype::new("bound-test", traits);
        let mut rng = fast_rng(789);
        let mutant = agent.mutate(&mut rng, 1.0, 10.0);
        let v = mutant.traits.get("val").copied().unwrap_or(0.0);
        assert!(
            v >= 0.0 && v <= 1.0,
            "value {} should be clamped to [0,1]",
            v
        );
    }

    #[test]
    fn test_summary_report() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 5,
            tournament_size: 2,
            mutation_rate: 0.0,
            crossover_rate: 0.0,
            mutation_sigma: 0.0,
            elite_count: 1,
        });
        arena.seed_population(&["x"], 0.1);
        arena.run_round("work", &mut dummy_score_fn);

        let summary = arena.summary();
        assert!(summary.contains("pop="));
        assert!(summary.contains("top_fit="));
    }

    #[test]
    fn test_mutate_rate_zero() {
        let mut traits = HashMap::new();
        traits.insert("x".into(), 0.5);
        let agent = AgentGenotype::new("fixed", traits);
        let mut rng = fast_rng(111);
        let mutant = agent.mutate(&mut rng, 0.0, 0.5);
        assert_eq!(
            mutant.traits.get("x").copied().unwrap_or(0.0),
            agent.traits.get("x").copied().unwrap_or(0.0),
            "zero mutation rate should preserve traits"
        );
    }

    #[test]
    fn test_crossover_rate_zero() {
        let mut t1 = HashMap::new();
        t1.insert("x".into(), 1.0);
        let mut t2 = HashMap::new();
        t2.insert("x".into(), 0.0);
        let a = AgentGenotype::new("a", t1);
        let b = AgentGenotype::new("b", t2);
        let mut rng = fast_rng(222);
        let child = a.crossover(&b, &mut rng, 0.0);
        assert_eq!(child.traits.get("x").copied().unwrap_or(0.0), 1.0);
    }

    // ── Gödel Agent Tests ────────────────────────────────────────────────

    fn dummy_gödel_score_fn(agent: &GödelAgent) -> f64 {
        agent.traits.get("speed").copied().unwrap_or(0.5)
            + agent.traits.get("accuracy").copied().unwrap_or(0.5)
            + agent.traits.get("plasticity").copied().unwrap_or(0.5)
    }

    fn dummy_code_mutator(code: &str, rng: &mut dyn FnMut() -> f64) -> String {
        let mut result = code.to_string();
        if rng() < 0.5 {
            result.push_str(&format!("\n// mutate-{}", (rng() * 1000.0) as u64));
        }
        result
    }

    #[test]
    fn test_gödel_agent_creation() {
        let mut traits = HashMap::new();
        traits.insert("speed".into(), 0.8);
        traits.insert("accuracy".into(), 0.7);
        let code = "fn agent() -> f64 { 42.0 }".to_string();
        let agent = GödelAgent {
            id: "gödel-0".into(),
            parent_ids: Vec::new(),
            generation: 0,
            fitness: 0.0,
            code: code.clone(),
            traits,
            archive_entries: Vec::new(),
            base_strategy: code,
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        assert_eq!(agent.id, "gödel-0");
        assert_eq!(agent.generation, 0);
        assert!((agent.fitness - 0.0).abs() < 1e-9);
        assert!(agent.code.contains("fn agent"));
    }

    #[test]
    fn test_seed_gödel_population() {
        let mut arena = AdversarialArena::new(ArenaConfig::default());
        arena.seed_gödel_population("fn agent_{id}() -> f64 {{ {seed} }}", 10, 0.3);
        assert_eq!(arena.gödel_population.len(), 10);
        for agent in &arena.gödel_population {
            assert!(agent.id.starts_with("gödel-"));
            assert!(agent.code.contains("fn agent_"));
            assert!(agent.traits.contains_key("speed"));
            assert!(agent.traits.contains_key("plasticity"));
        }
    }

    #[test]
    fn test_gödel_agent_mutation() {
        let mut arena = AdversarialArena::new(ArenaConfig::default());
        arena.seed_gödel_population("fn agent() -> f64 { 1.0 }", 5, 0.3);
        let _old_code = arena.gödel_population[0].code.clone();
        let result = arena.run_gödel_round(
            "self-modify",
            &mut dummy_gödel_score_fn,
            &mut |code, _rng| format!("{} // mutated", code),
        );
        assert!(result.generation > 0 || arena.history.len() == 1);
        assert!(!arena.gödel_population.is_empty());
    }

    #[test]
    fn test_run_gödel_round_basic() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 20,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        });
        arena.seed_gödel_population("fn think() -> f64 {{ negentropy() }}", 10, 0.4);
        let result = arena.run_gödel_round(
            "reasoning-task",
            &mut dummy_gödel_score_fn,
            &mut dummy_code_mutator,
        );
        assert_eq!(arena.history.len(), 1);
        assert!(result.top_fitness > 0.0);
        assert!(result.avg_fitness > 0.0);
        assert_eq!(result.population_size, 10);
    }

    #[test]
    fn test_archive_evolution() {
        let arena = AdversarialArena::new(ArenaConfig::default());
        let mut archive = vec![
            ArchiveEntry {
                agent_id: "a".into(),
                fitness: 0.5,
                code_snapshot: "code1".into(),
                generation: 0,
            },
            ArchiveEntry {
                agent_id: "b".into(),
                fitness: 0.9,
                code_snapshot: "code2".into(),
                generation: 1,
            },
            ArchiveEntry {
                agent_id: "c".into(),
                fitness: 0.3,
                code_snapshot: "code3".into(),
                generation: 0,
            },
            ArchiveEntry {
                agent_id: "a".into(),
                fitness: 0.8,
                code_snapshot: "code4".into(),
                generation: 2,
            },
        ];
        arena.archive_evolution(&mut archive, 2);
        assert_eq!(archive.len(), 2);
        assert!(archive[0].fitness >= archive[1].fitness);
        assert_ne!(archive[0].agent_id, archive[1].agent_id);
    }

    #[test]
    fn test_clade_metaproductivity() {
        let agents = vec![
            GödelAgent {
                id: "g0".into(),
                parent_ids: vec![],
                generation: 0,
                fitness: 0.8,
                code: String::new(),
                traits: HashMap::new(),
                archive_entries: vec![],
                base_strategy: String::new(),
                weight_deltas: HashMap::new(),
                harness_version: 0,
            },
            GödelAgent {
                id: "g1".into(),
                parent_ids: vec![],
                generation: 1,
                fitness: 0.6,
                code: String::new(),
                traits: HashMap::new(),
                archive_entries: vec![],
                base_strategy: String::new(),
                weight_deltas: HashMap::new(),
                harness_version: 0,
            },
            GödelAgent {
                id: "g2".into(),
                parent_ids: vec![],
                generation: 2,
                fitness: 0.4,
                code: String::new(),
                traits: HashMap::new(),
                archive_entries: vec![],
                base_strategy: String::new(),
                weight_deltas: HashMap::new(),
                harness_version: 0,
            },
        ];
        let cmp = AdversarialArena::clade_metaproductivity(&agents);
        assert!((cmp - 0.6).abs() < 1e-9, "CMP should be 0.6, got {}", cmp);
    }

    #[test]
    fn test_clade_metaproductivity_empty() {
        let cmp = AdversarialArena::clade_metaproductivity(&[]);
        assert!((cmp - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_self_consistency_pass() {
        let code = "fn main() { let x = 1; if x > 0 { println!(); } }";
        let invariants = &["fn ", "{"];
        let violations = AdversarialArena::self_consistency_check(code, invariants);
        assert!(
            violations.is_empty(),
            "Expected no violations, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_self_consistency_fail() {
        let code = "weird stuff without proper structure";
        let invariants = &["fn ", "struct ", "impl "];
        let violations = AdversarialArena::self_consistency_check(code, invariants);
        assert!(
            !violations.is_empty(),
            "Expected violations for code without required patterns"
        );
        let has_fn = violations.iter().any(|v| v.contains("fn"));
        assert!(has_fn, "Should complain about missing 'fn'");
    }

    #[test]
    fn test_self_consistency_braces() {
        let code = "fn main() { let x = 1; ";
        let invariants = &[];
        let violations = AdversarialArena::self_consistency_check(code, invariants);
        assert!(violations.iter().any(|v| v.contains("Unbalanced braces")));
    }

    #[test]
    fn test_gödel_evolve_generation() {
        let mut arena = AdversarialArena::new(ArenaConfig {
            population_size: 20,
            tournament_size: 3,
            mutation_rate: 0.2,
            crossover_rate: 0.3,
            mutation_sigma: 0.1,
            elite_count: 2,
        });
        arena.seed_gödel_population("fn evolve() -> f64 {{ 0.0 }}", 8, 0.3);

        for _ in 0..3 {
            arena.run_gödel_round(
                "evolve-task",
                &mut dummy_gödel_score_fn,
                &mut dummy_code_mutator,
            );
        }

        assert!(arena.history.len() >= 3);
        let distinct_ids: std::collections::HashSet<String> = arena
            .gödel_population
            .iter()
            .map(|a| a.id.clone())
            .collect();
        assert!(distinct_ids.len() > 1 || arena.gödel_population.len() > 0);
    }

    #[test]
    fn test_archive_preserves_diversity() {
        let arena = AdversarialArena::new(ArenaConfig::default());
        let mut archive = vec![
            ArchiveEntry {
                agent_id: "x".into(),
                fitness: 0.9,
                code_snapshot: "v1".into(),
                generation: 0,
            },
            ArchiveEntry {
                agent_id: "x".into(),
                fitness: 0.8,
                code_snapshot: "v2".into(),
                generation: 1,
            },
            ArchiveEntry {
                agent_id: "y".into(),
                fitness: 0.7,
                code_snapshot: "v3".into(),
                generation: 0,
            },
            ArchiveEntry {
                agent_id: "z".into(),
                fitness: 0.6,
                code_snapshot: "v4".into(),
                generation: 0,
            },
        ];
        arena.archive_evolution(&mut archive, 3);
        assert_eq!(archive.len(), 3);
        let ids: std::collections::HashSet<String> =
            archive.iter().map(|e| e.agent_id.clone()).collect();
        assert_eq!(ids.len(), 3, "Archive should have 3 distinct agent IDs");
        assert!(ids.contains("x"));
        assert!(ids.contains("y"));
        assert!(ids.contains("z"));
    }

    // ── SelfReferenceEngine tests ─────────────────────────────────────────

    #[test]
    fn test_causal_sleeper_cell_new() {
        let cell = CausalSleeperCell::new();
        assert_eq!(cell.step, 0);
        assert!(cell.trace_key.iter().all(|&x| x == 0));
        assert!(cell.delayed_self.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_causal_sleeper_cell_tick() {
        let mut cell = CausalSleeperCell::new();
        cell.tick(&[1u8; 64]);
        assert_eq!(cell.step, 1);
        // After tick, delayed_self = old trace_key (zeros), trace_key = XOR(zeros, [1;64]) = [1;64]
        assert_eq!(cell.trace_key, vec![1u8; 64]);
        assert!(cell.delayed_self.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_causal_sleeper_cell_self_similarity_zero_on_init() {
        let cell = CausalSleeperCell::new();
        // Both trace_key and delayed_self are zeros → zero similarity
        let sim = cell.self_similarity();
        assert!((sim - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_self_reference_engine_new() {
        let engine = SelfReferenceEngine::new(4);
        assert_eq!(engine.cells.len(), 4);
        assert!(engine.reflection_log.is_empty());
        assert!((engine.coherence - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_self_reference_engine_reflect() {
        let mut engine = SelfReferenceEngine::new(2);
        let traces = vec![vec![1u8; 64], vec![2u8; 64]];
        engine.reflect("test-thought", &traces);
        assert_eq!(engine.reflection_log.len(), 1);
        assert_eq!(engine.reflection_log[0].label, "test-thought");
        assert!(engine.coherence > 0.0 || engine.coherence == 0.0);
    }

    #[test]
    fn test_self_reference_engine_report() {
        let mut engine = SelfReferenceEngine::new(2);
        let traces = vec![vec![1u8; 64], vec![2u8; 64]];
        engine.reflect("thought-1", &traces);
        let report = engine.report();
        assert!(report.contains("SelfRefEngine:"));
        assert!(report.contains("cells=2"));
    }

    #[test]
    fn test_empty_gödel_round() {
        let mut arena = AdversarialArena::new(ArenaConfig::default());
        let result =
            arena.run_gödel_round("empty", &mut dummy_gödel_score_fn, &mut dummy_code_mutator);
        assert_eq!(result.population_size, 0);
    }

    // ── Harness+Weights Dual Update Tests ─────────────────────────────────

    #[test]
    fn test_apply_delta_update() {
        let mut agent = GödelAgent {
            id: "test".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "base".into(),
            traits: HashMap::from([("speed".into(), 0.5)]),
            archive_entries: vec![],
            base_strategy: "base".into(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        let update = HarnessWeightsUpdate {
            delta_updates: vec![("speed".into(), 0.3)],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 0.8,
        };
        agent.apply_dual_update(update);
        assert!((agent.weight_deltas.get("speed").copied().unwrap_or(0.0) - 0.3).abs() < 1e-9);
        assert_eq!(agent.harness_version, 0);
    }

    #[test]
    fn test_apply_harness_update() {
        let mut agent = GödelAgent {
            id: "test".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "old".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "old".into(),
            weight_deltas: HashMap::from([("x".into(), 0.2)]),
            harness_version: 0,
        };
        let update = HarnessWeightsUpdate {
            delta_updates: vec![],
            is_harness_update: true,
            new_base_strategy: Some("new_harness".into()),
            confidence: 0.9,
        };
        agent.apply_dual_update(update);
        assert_eq!(agent.base_strategy, "new_harness");
        assert_eq!(agent.harness_version, 1);
        assert!(agent.weight_deltas.is_empty());
    }

    #[test]
    fn test_effective_trait_stacking() {
        let mut traits = HashMap::new();
        traits.insert("speed".into(), 0.6);
        let mut deltas = HashMap::new();
        deltas.insert("speed".into(), 0.3);
        let agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: String::new(),
            traits,
            archive_entries: vec![],
            base_strategy: String::new(),
            weight_deltas: deltas,
            harness_version: 0,
        };
        let eff = agent.effective_trait("speed");
        assert!((eff - 0.9).abs() < 1e-9, "expected 0.9, got {}", eff);
    }

    #[test]
    fn test_effective_trait_clamped() {
        let mut traits = HashMap::new();
        traits.insert("x".into(), 0.9);
        let mut deltas = HashMap::new();
        deltas.insert("x".into(), 0.3);
        let agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: String::new(),
            traits,
            archive_entries: vec![],
            base_strategy: String::new(),
            weight_deltas: deltas,
            harness_version: 0,
        };
        assert!((agent.effective_trait("x") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_harness_update_cost_decay() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: String::new(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: String::new(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        assert!((agent.harness_update_cost() - 1.0).abs() < 1e-9);
        agent.harness_version = 1;
        assert!((agent.harness_update_cost() - 0.5).abs() < 1e-9);
        agent.harness_version = 4;
        assert!((agent.harness_update_cost() - 0.2).abs() < 1e-9);
    }

    #[test]
    fn test_suggest_updates_basic() {
        let mut traits = HashMap::new();
        traits.insert("speed".into(), 0.5);
        let agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: String::new(),
            traits,
            archive_entries: vec![],
            base_strategy: String::new(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        let perf = vec![("speed".into(), 0.2)];
        let config = DualUpdateConfig::default();
        let updates = agent.suggest_updates(&perf, &config);
        assert!(!updates.is_empty(), "should suggest at least one update");
        assert!(!updates[0].is_harness_update, "should be a weight update");
        assert_eq!(updates[0].delta_updates.len(), 1);
        assert_eq!(updates[0].delta_updates[0].0, "speed");
    }

    #[test]
    fn test_dual_update_config_defaults() {
        let config = DualUpdateConfig::default();
        assert!((config.weight_lr - 0.1).abs() < 1e-9);
        assert!((config.harness_update_prob - 0.05).abs() < 1e-9);
        assert!((config.max_weight_sum - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_weight_clamping() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "base".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "base".into(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        let update = HarnessWeightsUpdate {
            delta_updates: vec![("x".into(), 10.0)],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 0.5,
        };
        agent.apply_dual_update(update);
        let val = agent.weight_deltas.get("x").copied().unwrap_or(0.0);
        assert!(val <= 0.5, "delta should be clamped to 0.5, got {}", val);
        assert!(val >= 0.0);
    }

    #[test]
    fn test_empty_deltas() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "base".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "base".into(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        let update = HarnessWeightsUpdate {
            delta_updates: vec![],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 0.0,
        };
        agent.apply_dual_update(update);
        assert!(agent.weight_deltas.is_empty());
        assert_eq!(agent.code, "base");
    }

    #[test]
    fn test_harness_version_increment() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "v0".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "v0".into(),
            weight_deltas: HashMap::new(),
            harness_version: 5,
        };
        agent.apply_dual_update(HarnessWeightsUpdate {
            delta_updates: vec![],
            is_harness_update: true,
            new_base_strategy: Some("v6".into()),
            confidence: 1.0,
        });
        assert_eq!(agent.harness_version, 6);
        assert_eq!(agent.base_strategy, "v6");
    }

    #[test]
    fn test_multiple_delta_accumulation() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "base".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "base".into(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        agent.apply_dual_update(HarnessWeightsUpdate {
            delta_updates: vec![("a".into(), 0.2), ("b".into(), 0.3)],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 0.5,
        });
        assert!((agent.weight_deltas.get("a").copied().unwrap_or(0.0) - 0.2).abs() < 1e-9);
        assert!((agent.weight_deltas.get("b").copied().unwrap_or(0.0) - 0.3).abs() < 1e-9);
        // Second accumulation
        agent.apply_dual_update(HarnessWeightsUpdate {
            delta_updates: vec![("a".into(), 0.1)],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 0.3,
        });
        assert!((agent.weight_deltas.get("a").copied().unwrap_or(0.0) - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_delta_accumulation_clamp_sum() {
        let mut agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: "base".into(),
            traits: HashMap::new(),
            archive_entries: vec![],
            base_strategy: "base".into(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        // Total abs sum 0.5+0.5+0.5+0.5 = 2.0 — at the cap, no clamping
        agent.apply_dual_update(HarnessWeightsUpdate {
            delta_updates: vec![
                ("a".into(), 0.5),
                ("b".into(), 0.5),
                ("c".into(), 0.5),
                ("d".into(), 0.5),
            ],
            is_harness_update: false,
            new_base_strategy: None,
            confidence: 1.0,
        });
        let total: f64 = agent.weight_deltas.values().map(|v| v.abs()).sum();
        assert!(
            total <= 2.0 + 1e-9,
            "total abs sum {} exceeds cap 2.0",
            total
        );
    }

    #[test]
    fn test_suggest_updates_high_perf_no_update() {
        let mut traits = HashMap::new();
        traits.insert("speed".into(), 0.5);
        let agent = GödelAgent {
            id: "t".into(),
            parent_ids: vec![],
            generation: 0,
            fitness: 0.0,
            code: String::new(),
            traits,
            archive_entries: vec![],
            base_strategy: String::new(),
            weight_deltas: HashMap::new(),
            harness_version: 0,
        };
        let perf = vec![("speed".into(), 0.9)];
        let config = DualUpdateConfig::new(0.1, 0.05, 2.0);
        let updates = agent.suggest_updates(&perf, &config);
        let weight_updates: Vec<_> = updates.iter().filter(|u| !u.is_harness_update).collect();
        assert!(
            weight_updates.is_empty(),
            "high perf should not suggest weight deltas"
        );
    }
}
