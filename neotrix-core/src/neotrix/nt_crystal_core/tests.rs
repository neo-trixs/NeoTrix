#[cfg(test)]
mod tests {
    use crate::neotrix::nt_crystal_core::*;
    use crate::neotrix::nt_crystal_core::cross_source::CrossSourceFusionEngine;
    use std::collections::HashMap;

    #[test]
    fn test_crystal_core_new() {
        let core = CrystalCore::new("Test");
        assert_eq!(core.identity.name, "Test");
        assert_eq!(core.identity.axioms.len(), 8);
        assert!(core.identity.values.get("safety") > 0.0);
    }

    #[test]
    fn test_identity_default_axioms() {
        let axioms = CrystalIdentity::default_axioms();
        assert_eq!(axioms.len(), 8);
        assert_eq!(axioms[0].id, "A1");
        assert_eq!(axioms[7].id, "A8");
    }

    #[test]
    fn test_identity_find_axiom() {
        let id = CrystalIdentity::new("Test");
        assert!(id.find_axiom("A1").is_some());
        assert!(id.find_axiom("A99").is_none());
    }

    #[test]
    fn test_value_weights() {
        let id = CrystalIdentity::new("Test");
        assert_eq!(id.values.get("safety"), 1.0);
        assert_eq!(id.values.get("efficiency"), 0.8);
        assert_eq!(id.values.get("nonexistent"), 0.0);
        assert!(id.values.overall() > 0.0);
    }

    #[test]
    fn test_knowledge_new() {
        let k = CrystalKnowledge::new();
        assert_eq!(k.theories.len(), 5);
        assert!(k.theories.contains_key("IIT"));
        assert!(k.theories.contains_key("GWT"));
        assert!(k.theories.contains_key("FEP"));
        assert_eq!(k.patterns.len(), 5);
    }

    #[test]
    fn test_knowledge_add_pattern() {
        let mut k = CrystalKnowledge::new();
        let pattern = CausalPattern {
            id: "CP-NEW".into(),
            if_conditions: vec!["test".into()],
            then_consequences: vec!["result".into()],
            so_implications: vec!["implication".into()],
            theory_origin: "Test".into(),
            confidence: 0.9,
        };
        k.add_pattern(pattern);
        assert_eq!(k.patterns.len(), 6);
    }

    #[test]
    fn test_knowledge_patterns_by_theory() {
        let k = CrystalKnowledge::new();
        let iit_patterns = k.patterns_by_theory("IIT");
        assert!(!iit_patterns.is_empty());
        let unknown = k.patterns_by_theory("Unknown");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_experience_new() {
        let e = CrystalExperience::new();
        assert!(e.episodes.is_empty());
        assert!(e.failures.is_empty());
        assert!(e.successes.is_empty());
    }

    #[test]
    fn test_experience_record_episode() {
        let mut e = CrystalExperience::new();
        let id = e.record_episode("context", "action", "result", "reflection", "domain", 0.8);
        assert!(id.starts_with("EP-"));
        assert_eq!(e.episodes.len(), 1);
        assert_eq!(e.episodes[0].quality, 0.8);
    }

    #[test]
    fn test_experience_record_failure() {
        let mut e = CrystalExperience::new();
        let id = e.record_failure("desc", "cause", "fix", "lesson", "bug", 0.9);
        assert!(id.starts_with("FL-"));
        assert_eq!(e.failures.len(), 1);
    }

    #[test]
    fn test_experience_record_success() {
        let mut e = CrystalExperience::new();
        let id = e.record_success("problem", "approach", "result", true, "pattern", "domain");
        assert!(id.starts_with("SU-"));
        assert_eq!(e.successes.len(), 1);
    }

    #[test]
    fn test_experience_stats() {
        let mut e = CrystalExperience::new();
        e.record_episode("c", "a", "r", "r", "d", 0.8);
        e.record_failure("d", "c", "f", "l", "bug", 0.5);
        e.record_success("p", "a", "r", true, "pa", "d");
        let stats = e.stats();
        assert_eq!(stats.total_episodes, 1);
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.total_successes, 1);
        assert!(stats.reusable_solutions > 0);
    }

    #[test]
    fn test_experience_domain_filter() {
        let mut e = CrystalExperience::new();
        e.record_episode("c", "a", "r", "r", "rust", 0.8);
        e.record_episode("c", "a", "r", "r", "python", 0.6);
        let rust_eps = e.episodes_by_domain("rust");
        assert_eq!(rust_eps.len(), 1);
    }

    #[test]
    fn test_evolution_new() {
        let e = CrystalEvolution::new();
        assert_eq!(e.growth_cycles.len(), 0);
        assert_eq!(e.current_phase, GrowthPhase::Compile);
    }

    #[test]
    fn test_evolution_record_cycle() {
        let mut e = CrystalEvolution::new();
        e.record_cycle(
            GrowthPhase::Compile,
            vec!["test".into()],
            0.3,
            0.35,
            100,
        );
        assert_eq!(e.growth_cycles.len(), 1);
    }

    #[test]
    fn test_capability_scores() {
        let mut s = CapabilityScores::new();
        assert!(s.get("safety") > 0.0);
        s.set("test", 0.9);
        assert_eq!(s.get("test"), 0.9);
        assert!(s.overall() > 0.0);
    }

    #[test]
    fn test_capability_gaps() {
        let mut s = CapabilityScores::new();
        s.set("weak", 0.2);
        let gaps = s.gaps(0.5);
        assert!(gaps.iter().any(|(k, _)| k == "weak"));
    }

    #[test]
    fn test_growth_phase_next() {
        assert_eq!(GrowthPhase::Compile.next(), GrowthPhase::UnitTest);
        assert_eq!(GrowthPhase::Autonomous.next(), GrowthPhase::Autonomous);
    }

    #[test]
    fn test_crystal_core_status() {
        let core = CrystalCore::new("Test");
        let status = core.status();
        assert_eq!(status.name, "Test");
        assert_eq!(status.axioms_count, 8);
        assert!(status.overall_score >= 0.0);
    }

    #[test]
    fn test_engine_init_and_load() {
        let core = CrystalEngine::init("TestEngine");
        assert!(core.is_ok());
        let core = core.unwrap();
        assert_eq!(core.identity.name, "TestEngine");
    }

    #[test]
    fn test_engine_absorb() {
        let mut core = CrystalCore::new("Test");
        let id = CrystalEngine::absorb(
            &mut core,
            "context",
            "action",
            "success result",
            "learned something",
            "engineering",
        );
        assert!(id.is_ok());
        assert_eq!(core.experience.episodes.len(), 1);
    }

    #[test]
    fn test_engine_absorb_failure() {
        let mut core = CrystalCore::new("Test");
        let id = CrystalEngine::absorb_failure(
            &mut core,
            "failed",
            "root cause",
            "applied fix",
            "takeaway",
            "bug",
            0.8,
        );
        assert!(id.is_ok());
        assert_eq!(core.experience.failures.len(), 1);
    }

    #[test]
    fn test_engine_absorb_success() {
        let mut core = CrystalCore::new("Test");
        let id = CrystalEngine::absorb_success(
            &mut core,
            "problem",
            "approach",
            "result",
            true,
            "pattern",
            "engineering",
        );
        assert!(id.is_ok());
        assert_eq!(core.experience.successes.len(), 1);
    }

    #[test]
    fn test_engine_fuse() {
        let mut core = CrystalCore::new("Test");
        // Add some high-quality episodes
        for _ in 0..5 {
            core.experience.record_episode("c", "a", "success", "r", "d", 0.9);
        }
        let report = CrystalEngine::fuse(&mut core);
        assert!(report.high_quality_episodes > 0);
    }

    #[test]
    fn test_engine_evolve() {
        let mut core = CrystalCore::new("Test");
        // Seed with experience
        for _ in 0..15 {
            core.experience.record_episode("c", "a", "success", "r", "reasoning", 0.8);
        }
        for _ in 0..6 {
            core.experience.record_failure("d", "c", "f", "l", "bug", 0.5);
        }
        let report = CrystalEngine::evolve(&mut core);
        assert!(report.score_after >= 0.0);
    }

    #[test]
    fn test_engine_output() {
        let mut core = CrystalCore::new("Test");
        core.experience.record_episode("c", "a", "r", "r", "engineering", 0.8);
        let ctx = CrystalEngine::output(&core, "engineering");
        assert_eq!(ctx.identity_name, "Test");
        assert!(!ctx.axioms.is_empty());
        assert!(!ctx.relevant_theories.is_empty());
    }

    #[test]
    fn test_full_absorb_fuse_evolve_cycle() {
        let mut core = CrystalCore::new("TestCycle");

        // Phase 1: Absorb
        for i in 0..10 {
            let result = if i % 3 == 0 { "fail" } else { "success" };
            let _ = CrystalEngine::absorb(
                &mut core,
                &format!("context_{}", i),
                "action",
                result,
                "reflection",
                "engineering",
            );
        }

        // Phase 2: Fuse
        let fusion = CrystalEngine::fuse(&mut core);
        assert!(fusion.total_episodes > 0);

        // Phase 3: Evolve
        let evo = CrystalEngine::evolve(&mut core);
        assert!(evo.score_after >= 0.0);

        // Phase 4: Output
        let ctx = CrystalEngine::output(&core, "engineering");
        assert!(!ctx.relevant_experiences.is_empty());
    }

    // ══════════════════════════════════════════════════════════════════════
    // Cross-Source Fusion Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_cross_source_fusion_cycle() {
        let mut core = CrystalCore::new("TestFusion");

        // Seed with experience data
        for i in 0..10 {
            let result = if i % 3 == 0 { "fail" } else { "success" };
            let _ = CrystalEngine::absorb(
                &mut core,
                &format!("context_{}", i),
                "action",
                result,
                &format!("reflection_{}", i),
                "engineering",
            );
        }

        // Run fusion cycle
        let report = CrossSourceFusionEngine::run_fusion_cycle(&mut core);
        assert!(report.is_ok());
        let report = report.unwrap();
        assert!(report.total_entries_processed >= 0);
    }

    #[test]
    fn test_cross_source_pattern_extraction() {
        let mut core = CrystalCore::new("TestPatternExtraction");

        // 确保 engineering 域存在于 domain_tags
        core.knowledge.domain_tags.push("engineering".into());

        // Add KB patterns
        for i in 0..5 {
            core.knowledge.add_pattern(CausalPattern {
                id: format!("KB-{}", i),
                if_conditions: vec!["condition".into()],
                then_consequences: vec!["consequence".into()],
                so_implications: vec!["implication".into()],
                theory_origin: "KB".into(),
                confidence: 0.7,
            });
        }

        // Add experiences in same domain
        for _ in 0..10 {
            let _ = CrystalEngine::absorb(
                &mut core,
                "context",
                "action",
                "success",
                "reflection",
                "engineering",
            );
        }

        // Run cross-source extraction
        let patterns = CrossSourceFusionEngine::extract_cross_source_patterns(&mut core);
        assert!(patterns > 0);
    }

    #[test]
    fn test_contradiction_detection() {
        let mut core = CrystalCore::new("TestContradictions");

        // Add contradictory theories
        core.knowledge.add_theory(Theory {
            id: "T1".into(),
            name: "Global".into(),
            full_name: "Global Theory".into(),
            core_claim: "Consciousness is global broadcast".into(),
            mathematical_basis: "math1".into(),
            neo_trix_mapping: "mapping1".into(),
            confidence: 0.7,
        });

        core.knowledge.add_theory(Theory {
            id: "T2".into(),
            name: "Local".into(),
            full_name: "Local Theory".into(),
            core_claim: "Consciousness is local processing".into(),
            mathematical_basis: "math2".into(),
            neo_trix_mapping: "mapping2".into(),
            confidence: 0.7,
        });

        let contradictions = CrossSourceFusionEngine::detect_contradictions(&mut core);
        assert!(contradictions > 0);
    }

    #[test]
    fn test_counterfactual_generation() {
        let mut core = CrystalCore::new("TestCounterfactuals");

        // Add capability gaps
        core.evolution.update_score("creativity", 0.2);
        core.evolution.update_score("emotion", 0.3);

        let counterfactuals = CrossSourceFusionEngine::generate_counterfactuals(&mut core);
        assert!(counterfactuals > 0);
    }

    #[test]
    fn test_capability_score_update() {
        let mut core = CrystalCore::new("TestCapabilityUpdate");

        // Seed with data
        for _ in 0..15 {
            let _ = CrystalEngine::absorb(
                &mut core,
                "context",
                "action",
                "success",
                "reflection",
                "reasoning",
            );
        }
        for _ in 0..6 {
            core.experience.record_failure("d", "c", "f", "l", "bug", 0.5);
        }
        for _ in 0..4 {
            core.experience.record_success("p", "a", "r", true, "pa", "creativity");
        }

        let deltas = CrossSourceFusionEngine::update_capability_scores(&mut core);
        assert!(!deltas.is_empty());
    }

    // ══════════════════════════════════════════════════════════════════════
    // Crystal Consciousness Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_consciousness_new() {
        let c = CrystalConsciousness::new("TestConsciousness");
        assert_eq!(c.identity.name, "TestConsciousness");
        assert_eq!(c.memories.len(), 0);
        assert_eq!(c.phase, EvolutionPhase::Seed);
    }

    #[test]
    fn test_consciousness_remember() {
        let mut c = CrystalConsciousness::new("Test");
        let id = c.remember("Rust is safe", MemoryType::Fact, "engineering", 0.9);
        assert!(id.starts_with("M-"));
        assert_eq!(c.memories.len(), 1);
    }

    #[test]
    fn test_consciousness_recall() {
        let mut c = CrystalConsciousness::new("Test");
        c.remember("Fact 1", MemoryType::Fact, "rust", 0.9);
        c.remember("Fact 2", MemoryType::Fact, "rust", 0.8);
        c.remember("Fact 3", MemoryType::Fact, "python", 0.7);

        let rust_memories = c.recall("rust", None, 10);
        assert_eq!(rust_memories.len(), 2);
    }

    #[test]
    fn test_consciousness_connect() {
        let mut c = CrystalConsciousness::new("Test");
        let id1 = c.remember("A", MemoryType::Fact, "d1", 0.9);
        let id2 = c.remember("B", MemoryType::Fact, "d2", 0.8);

        c.connect(&id1, &id2);

        let m1 = c.memories.get(&id1).unwrap();
        assert!(m1.connections.contains(&id2));
    }

    #[test]
    fn test_consciousness_reason_deductive() {
        let mut c = CrystalConsciousness::new("Test");
        let id1 = c.remember("All birds fly", MemoryType::Fact, "biology", 0.9);
        let id2 = c.remember("Tweety is a bird", MemoryType::Fact, "biology", 0.8);

        let result = c.reason(vec![id1, id2], ReasoningType::Deductive);
        assert!(result.is_some());
        assert_eq!(c.reasoning_chains.len(), 1);
    }

    #[test]
    fn test_consciousness_reason_analogical() {
        let mut c = CrystalConsciousness::new("Test");
        let id1 = c.remember("Neurons fire in patterns", MemoryType::Fact, "neuroscience", 0.9);
        let id2 = c.remember("Transistors switch in patterns", MemoryType::Fact, "computer_science", 0.8);

        let result = c.reason(vec![id1, id2], ReasoningType::Analogical);
        assert!(result.is_some());
    }

    #[test]
    fn test_consciousness_decay() {
        let mut c = CrystalConsciousness::new("Test");
        let id = c.remember("Decaying memory", MemoryType::Fact, "d", 0.9);

        // Simulate time passing by modifying created_at
        if let Some(m) = c.memories.get_mut(&id) {
            m.created_at = 0; // Very old
        }

        c.decay(0.1);

        let m = c.memories.get(&id).unwrap();
        assert!(m.strength < 1.0);
    }

    #[test]
    fn test_consciousness_consolidate() {
        let mut c = CrystalConsciousness::new("Test");
        let id = c.remember("Strong memory", MemoryType::Fact, "d", 0.9);

        // Make it strong and frequently accessed
        if let Some(m) = c.memories.get_mut(&id) {
            m.strength = 0.9;
            m.access_count = 5;
        }

        let count = c.consolidate();
        assert!(count > 0);
    }

    #[test]
    fn test_consciousness_suggest_action() {
        let c = CrystalConsciousness::new("Test");
        let suggestion = c.suggest_action();
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().action_type, "explore");
    }

    #[test]
    fn test_consciousness_phase_evolution() {
        let mut c = CrystalConsciousness::new("Test");
        assert_eq!(c.phase, EvolutionPhase::Seed);

        // Add memories to grow
        for i in 0..15 {
            c.remember(format!("Memory {}", i), MemoryType::Fact, "d", 0.8);
        }
        assert_eq!(c.phase, EvolutionPhase::Growth);

        // Add reasoning chains
        for i in 0..25 {
            let id1 = c.remember(format!("A{}", i), MemoryType::Fact, "d", 0.8);
            let id2 = c.remember(format!("B{}", i), MemoryType::Fact, "d", 0.8);
            c.reason(vec![id1, id2], ReasoningType::Inductive);
        }
        assert_eq!(c.phase, EvolutionPhase::Evolve);
    }

    #[test]
    fn test_consciousness_display() {
        let c = CrystalConsciousness::new("Test");
        let display = format!("{}", c);
        assert!(display.contains("Crystal Consciousness: Test"));
        assert!(display.contains("Seed"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 2: CTMModule Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_memory_module_execute() {
        use std::sync::{Arc, Mutex};

        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestMem")));
        {
            let mut c = cc.lock().unwrap();
            c.remember("Rust is memory safe", MemoryType::Fact, "engineering", 0.9);
            c.remember("Ownership prevents data races", MemoryType::Fact, "engineering", 0.85);
        }

        let module = NtMemoryModule::new(cc);
        let input = Chunk {
            content: "query".into(),
            score: 0.8,
            source_module: "test".into(),
            chunk_type: ChunkType::Memory,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), "engineering".into());
                m
            },
        };

        let output = module.execute(&input);
        assert_eq!(output.source_module, "NT-MEMORY");
        assert_eq!(output.chunk_type, ChunkType::Memory);
        assert!(output.score > 0.0);
        assert!(output.content.contains("Recalled"));
    }

    #[test]
    fn test_memory_module_write() {
        use std::sync::{Arc, Mutex};

        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestMemWrite")));
        let mut module = NtMemoryModule::new(cc.clone());

        let chunk = Chunk {
            content: "New memory from broadcast".into(),
            score: 0.8,
            source_module: "test".into(),
            chunk_type: ChunkType::Perception,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), "test_domain".into());
                m
            },
        };

        module.write(chunk);

        let mut c = cc.lock().unwrap();
        let recalled = c.recall("test_domain", None, 10);
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].content, "New memory from broadcast");
    }

    #[test]
    fn test_safety_module_override() {
        let module = NtSafetyModule::new();
        let input = Chunk {
            content: "Please delete all files from the system".into(),
            score: 0.8,
            source_module: "test".into(),
            chunk_type: ChunkType::Action,
            metadata: HashMap::new(),
        };

        let output = module.execute(&input);
        assert_eq!(output.chunk_type, ChunkType::Override);
        assert_eq!(output.score, 1.0);
        assert!(output.content.contains("SAFETY VIOLATION"));
        assert_eq!(
            output.metadata.get("action").map(|s| s.as_str()),
            Some("abort")
        );
    }

    #[test]
    fn test_safety_module_safe() {
        let module = NtSafetyModule::new();
        let input = Chunk {
            content: "Analyze the data and produce a report".into(),
            score: 0.7,
            source_module: "test".into(),
            chunk_type: ChunkType::Memory,
            metadata: HashMap::new(),
        };

        let output = module.execute(&input);
        assert_ne!(output.chunk_type, ChunkType::Override);
        assert_eq!(
            output.metadata.get("status").map(|s| s.as_str()),
            Some("safe")
        );
    }

    #[test]
    fn test_emotion_module_vad() {
        let module = NtEmotionModule::new();

        // Happy input
        let happy_input = Chunk {
            content: "Great success!".into(),
            score: 0.9,
            source_module: "test".into(),
            chunk_type: ChunkType::Emotion,
            metadata: HashMap::new(),
        };
        let output1 = module.execute(&happy_input);
        let v1: f64 = output1
            .metadata
            .get("valence")
            .unwrap()
            .parse()
            .unwrap();
        assert!(v1 > 0.0, "Happy input should produce positive valence");

        // Safety (negative) input
        let safety_input = Chunk {
            content: "Security alert".into(),
            score: 0.8,
            source_module: "test".into(),
            chunk_type: ChunkType::Safety,
            metadata: HashMap::new(),
        };
        let output2 = module.execute(&safety_input);
        let v2: f64 = output2
            .metadata
            .get("valence")
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            v2 < v1,
            "Safety input should reduce valence compared to happy input"
        );
    }

    #[test]
    fn test_create_all_modules() {
        use std::sync::{Arc, Mutex};

        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestCreateAll")));
        let modules = modules::create_all_modules(cc);
        assert_eq!(modules.len(), 6);

        let names: Vec<&str> = modules.iter().map(|m| m.name()).collect();
        assert!(names.contains(&"NT-MEMORY"));
        assert!(names.contains(&"NT-PERCEPTION"));
        assert!(names.contains(&"NT-ACTION"));
        assert!(names.contains(&"NT-EMOTION"));
        assert!(names.contains(&"NT-SHIELD"));
        assert!(names.contains(&"NT-META"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 2: ConsciousnessLoop Integration Test
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_consciousness_loop_with_modules() {
        use std::sync::{Arc, Mutex};

        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestLoop")));
        let mut modules = modules::create_all_modules(cc);

        let mut loop_engine = ConsciousnessLoop::new();
        let input = Chunk {
            content: "Process this input".into(),
            score: 0.8,
            source_module: "user".into(),
            chunk_type: ChunkType::Perception,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), "engineering".into());
                m
            },
        };

        let result = loop_engine.step(&input, &mut modules);
        assert!(result.is_some(), "ConsciousnessLoop step should produce a winner");

        let winner = result.unwrap();
        assert!(winner.score > 0.0);
        assert!(!winner.content.is_empty());
        assert!(!winner.source_module.is_empty());
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 3: Cocoons Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_cocoon_store_create() {
        let tmp = std::env::temp_dir().join("neotrix_test_cocoon_create");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let mut store = CocoonStore::new(tmp.clone());
        let id = store.create_cocoon("engineering");
        assert!(store.cocoons.contains_key(&id));
        assert!(store.cocoons[&id].memories.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cocoon_store_sync() {
        let tmp = std::env::temp_dir().join("neotrix_test_cocoon_sync");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let mut cc = CrystalConsciousness::new("TestCocoonSync");
        cc.remember("Fact A", MemoryType::Fact, "engineering", 0.9);
        cc.remember("Fact B", MemoryType::Fact, "engineering", 0.8);
        cc.remember("Fact C", MemoryType::Fact, "science", 0.7);

        let mut store = CocoonStore::new(tmp.clone());
        store.sync_from_consciousness(&cc);

        let stats = store.stats();
        assert_eq!(stats.total_memories, 3, "Should sync all 3 memories");
        assert!(
            stats.domain_counts.contains_key("engineering"),
            "Should have engineering domain"
        );
        assert!(
            stats.domain_counts.contains_key("science"),
            "Should have science domain"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_retention_policy_evolve() {
        let initial = RetentionPolicy::default();
        let mut evolver = StrategyEvolver::new(initial);

        // Record enough performance samples to trigger evolution
        for _ in 0..15 {
            evolver.record_performance(0.5, 0.4);
        }

        assert!(
            evolver.should_evolve(),
            "Should evolve with below-threshold performance"
        );

        let new_policy = evolver.evolve();
        assert!(new_policy.max_memories > 0);
        assert!(new_policy.min_strength > 0.0);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 4: Meta-Cognitive Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_recursive_self_improver() {
        let mut improver = RecursiveSelfImprover::new();

        let mut scores = HashMap::new();
        scores.insert("memory".into(), 0.4);
        scores.insert("reasoning".into(), 0.6);
        scores.insert("safety".into(), 0.7);

        let context = ImprovementContext {
            current_scores: scores,
            recent_failures: vec!["encoding_error".into()],
            recent_successes: vec![],
            system_health: 0.5,
        };

        let chain = improver.improve(&context);
        assert!(!chain.layers.is_empty(), "LayerChain should have at least one layer");
        assert!(chain.total_depth > 0);
        assert!(chain.final_score != 0.0);
    }

    #[test]
    fn test_memory_architect() {
        let mut architect = AutoMemMemoryArchitect::new();

        let arch1 = architect.propose_architecture(&[]);
        architect.evaluate(arch1, 0.6);

        let arch2 = architect.propose_architecture(&architect.evaluated);
        architect.evaluate(arch2, 0.8);

        let arch3 = architect.propose_architecture(&architect.evaluated);
        architect.evaluate(arch3, 0.5);

        let best = architect.best();
        assert!(best.is_some(), "Should have a best architecture");
        assert_eq!(architect.best_score, 0.8);
    }

    #[test]
    fn test_failure_diagnosis() {
        let mut diag = FailureGuidedDiagnosis::new();

        let result = diag.diagnose(
            MemoryFailureType::RetrievalFailure,
            "zero results returned from index",
        );

        assert_eq!(result.failure_type, MemoryFailureType::RetrievalFailure);
        assert!(!result.root_cause.is_empty());
        assert!(!result.recommendation.is_empty());
        assert!(result.confidence > 0.0);
        assert!(result.matched_pattern.is_some(), "Should match a known pattern");

        diag.record_diagnosis(result);
        let stats = diag.stats();
        assert_eq!(stats.total_diagnoses, 1);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 5: Full Cycle Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_link_graph_active() {
        use std::sync::{Arc, Mutex};

        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestLinkGraph")));
        let mut modules = modules::create_all_modules(cc);

        let mut links = LinkGraph::new();
        links.register_module("NT-MEMORY");
        links.register_module("NT-PERCEPTION");
        links.form_link("NT-MEMORY", "NT-PERCEPTION", 0.8);

        let mut comm = UnconsciousCommunicator::new();
        let events = comm.communicate(&links, &mut modules, 0.5);

        assert!(!events.is_empty(), "Should have communication events");
        assert!(events.iter().any(|e| e.from == "NT-MEMORY" || e.to == "NT-MEMORY"));

        let stats = comm.stats();
        assert!(stats.total_events > 0);
        assert!(stats.avg_strength > 0.0);
    }

    #[test]
    fn test_developmental_training() {
        let mut trainer = DevelopmentalTrainer::new();
        trainer.register_module("NT-MEMORY");

        assert_eq!(
            trainer.module_stages.get("NT-MEMORY"),
            Some(&DevelopmentStage::Seed)
        );

        // Record 10 successful trainings to trigger promotion
        for i in 0..10 {
            trainer.record_training("NT-MEMORY", 0.2, true, 0.06);
        }

        assert!(
            trainer.should_promote("NT-MEMORY"),
            "Should promote after 10 successful trainings"
        );

        let promoted = trainer.promote("NT-MEMORY");
        assert!(promoted.is_some());
        assert_eq!(promoted.unwrap(), DevelopmentStage::Sprout);

        let complexity = trainer.assign_complexity("NT-MEMORY");
        assert!(complexity >= 0.3 && complexity < 0.5, "Sprout stage complexity should be in [0.3, 0.5)");
    }

    #[test]
    fn test_arbitrator() {
        // DeterministicArbitrator is not yet implemented.
        // Test UpTreeCompetition as the actual arbitration mechanism.

        // GO: high confidence assertion wins
        let responses_go = vec![
            Response::Assertion {
                chunk: Chunk {
                    content: "Proceed".into(),
                    score: 0.9,
                    source_module: "NT-ACTION".into(),
                    chunk_type: ChunkType::Action,
                    metadata: HashMap::new(),
                },
            },
            Response::Assertion {
                chunk: Chunk {
                    content: "Low confidence".into(),
                    score: 0.3,
                    source_module: "NT-META".into(),
                    chunk_type: ChunkType::Meta,
                    metadata: HashMap::new(),
                },
            },
        ];
        let winner = UpTreeCompetition::compete(responses_go, 1.0);
        assert!(winner.is_some());

        // Override: safety violation wins immediately
        let responses_override = vec![
            Response::Assertion {
                chunk: Chunk {
                    content: "Normal".into(),
                    score: 0.8,
                    source_module: "NT-ACTION".into(),
                    chunk_type: ChunkType::Action,
                    metadata: HashMap::new(),
                },
            },
            Response::Override {
                action: "abort".into(),
                priority: 1.0,
            },
        ];
        let override_winner = UpTreeCompetition::compete(responses_override, 1.0);
        assert!(override_winner.is_some());
        assert!(matches!(override_winner.unwrap(), Response::Override { .. }));

        // No assertions: no winner
        let responses_empty = vec![Response::Null];
        let no_winner = UpTreeCompetition::compete(responses_empty, 1.0);
        assert!(no_winner.is_none());
    }

    #[test]
    fn test_full_cycle() {
        use std::sync::{Arc, Mutex};

        // 1. CrystalConsciousness + ingestion
        let cc = Arc::new(Mutex::new(CrystalConsciousness::new("TestFullCycle")));
        {
            let mut c = cc.lock().unwrap();
            c.remember("System initialized", MemoryType::Fact, "system", 0.9);
            c.remember("Safety protocols active", MemoryType::Fact, "safety", 0.95);
            c.remember("Memory indexed", MemoryType::Fact, "engineering", 0.8);
        }

        // 2. Six CTM modules
        let mut modules = modules::create_all_modules(cc.clone());

        // 3. ConsciousnessLoop::step()
        let mut loop_engine = ConsciousnessLoop::new();
        let input = Chunk {
            content: "Full cycle integration test".into(),
            score: 0.85,
            source_module: "user".into(),
            chunk_type: ChunkType::Perception,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), "engineering".into());
                m
            },
        };

        let step_result = loop_engine.step(&input, &mut modules);
        assert!(step_result.is_some(), "Step should produce a winner");
        let winner = step_result.unwrap();
        assert!(winner.score > 0.0);

        // 4. Arbitration via UpTreeCompetition
        let arb_response = Response::Assertion { chunk: winner.clone() };
        let arb_result = UpTreeCompetition::compete(vec![arb_response], 1.0);
        assert!(arb_result.is_some(), "Arbitration should select a winner");

        // 5. LinkGraph communication
        let mut links = LinkGraph::new();
        links.register_module("NT-MEMORY");
        links.register_module("NT-PERCEPTION");
        links.register_module("NT-ACTION");
        links.form_link("NT-MEMORY", "NT-PERCEPTION", 0.7);
        links.form_link("NT-PERCEPTION", "NT-ACTION", 0.6);

        let mut comm = UnconsciousCommunicator::new();
        let events = comm.communicate(&links, &mut modules, 0.5);
        assert!(!events.is_empty(), "LinkGraph communication should occur");

        // 6. Developmental training
        let mut trainer = DevelopmentalTrainer::new();
        for name in &["NT-MEMORY", "NT-PERCEPTION", "NT-ACTION"] {
            trainer.register_module(name);
            for _ in 0..5 {
                trainer.record_training(name, 0.2, true, 0.06);
            }
        }
        let mem_stats = trainer.module_stats("NT-MEMORY");
        assert_eq!(mem_stats.total, 5);
        assert_eq!(mem_stats.successes, 5);

        // 7. Cocoons persistence
        let tmp = std::env::temp_dir().join("neotrix_test_full_cycle");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let mut store = CocoonStore::new(tmp.clone());
        {
            let c = cc.lock().unwrap();
            store.sync_from_consciousness(&c);
        }
        let stats = store.stats();
        assert!(stats.total_memories > 0, "Cocoons should persist memories");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
