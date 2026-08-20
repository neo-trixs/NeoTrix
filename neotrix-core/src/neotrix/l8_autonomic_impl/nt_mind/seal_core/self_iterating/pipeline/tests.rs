    use super::*;

    #[test]
    fn test_placeholder() {
        let brain = crate::neotrix::l8_autonomic_impl::nt_mind::self_iterating::brain_core::ReasoningBrain::new();
        let task_type = crate::neotrix::nt_world_model::TaskType::General;
        let snapshot = BrainSnapshot::new(&brain, &task_type);
        assert!(snapshot.learning_rate >= 0.0);
    }

    #[test]
    fn test_process_wrapper_consumes_consciousness_fruits() {
        // B5 (缺陷4修复) 验证: ProcessWrapperStage 应消费意识树果实,
        // 使 extract_from_consciousness_tree 的果实 trace 进入 process buffer。
        // 此前果实从不被 SEAL 消费 (extract_from_consciousness_tree 无生产调用者)。
        let mut brain = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::loop_impl::core::SelfIteratingBrain::new();
        // 注入一个意识树果实 (quality 0.9)
        let fruit = crate::core::nt_core_consciousness_tree::EvolutionFruit {
            name: "NT-CORE-evo-fruit-1".into(),
            source_branch: crate::core::nt_core_consciousness_tree::BranchKind::Core,
            description: "Evolution capability from NT-CORE".into(),
            produced_at_cycle: 1,
            quality: 0.9,
            claim: "Branch Core produces capability at maturity 0.9".into(),
            evidence: crate::core::nt_core_consciousness_tree::EvidenceChain {
                run_id: Some("fruit-run-1".into()),
                ..crate::core::nt_core_consciousness_tree::EvidenceChain::default()
            },
            stop_rule: crate::core::nt_core_consciousness_tree::StopRule::default(),
            benchmark: crate::core::nt_core_consciousness_tree::ProviderBenchmark::default(),
            generation: 0,
        };
        brain._consciousness_fruits = vec![fruit];
        // 给 tool_traces 一个步骤, 使 process 有基础样本
        brain.tool_traces.push(("search".into(), 10, true));

        let stage = ProcessWrapperStage::new();
        let decision = stage.process(&mut brain).expect("process ok");
        match decision {
            StageDecision::Continue => {}
            other => panic!("expected Continue, got {:?}", other),
        }
        // 果实 trace 应进入 process buffer (至少 1 条来自 ConsciousnessTree)
        assert!(
            brain
                ._process_stage
                .buffer
                .traces
                .iter()
                .any(|t| t.source == TraceSource::ConsciousnessTree),
            "consciousness fruit trace consumed by SEAL process: {:?}",
            brain
                ._process_stage
                .buffer
                .traces
                .iter()
                .map(|t| t.source.clone())
                .collect::<Vec<_>>()
        );
        // 缺陷2修复 (自我运转实际情况): 果实消费后应清除, 防止同一批果实被
        // SEAL 反复消费 (1h 注入 vs 10min 消费时序错配 → 重复污染 process 学习)。
        assert!(
            brain._consciousness_fruits.is_empty(),
            "fruits cleared after consumption (one-shot): {}",
            brain._consciousness_fruits.len()
        );
        // 再次 process: 无果实可消费, buffer 不新增 ConsciousnessTree trace
        let before = brain
            ._process_stage
            .buffer
            .traces
            .iter()
            .filter(|t| t.source == TraceSource::ConsciousnessTree)
            .count();
        let _ = stage.process(&mut brain).expect("process again ok");
        let after = brain
            ._process_stage
            .buffer
            .traces
            .iter()
            .filter(|t| t.source == TraceSource::ConsciousnessTree)
            .count();
        assert_eq!(before, after, "no re-consumption after clear");
    }
