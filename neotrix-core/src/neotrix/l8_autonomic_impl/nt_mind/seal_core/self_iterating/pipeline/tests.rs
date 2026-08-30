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

    #[test]
    fn test_reward_calc_affective_guided_external() {
        // Q2 P3: 外部奖励 + 情感候选 (高愉悦 + Bond + 充足交互) → reward 被引导抬高
        use crate::core::nt_core_knowledge::{
            publish_affective_observation, take_affective_observation, AffectiveFeedback,
        };
        let mut brain = SelfIteratingBrain::new();
        brain._external_reward = Some(0.5);
        publish_affective_observation(Some(AffectiveFeedback {
            valence: 0.9,
            arousal: 0.2,
            stage: 4,
            interactions: 10,
            signal_weight: 0.3,
        }));
        let stage = RewardCalculationStage::new();
        stage.process(&mut brain).expect("reward calc ok");
        let boosted = brain._reward();
        assert!(boosted > 0.5, "affective 引导应抬高, got {boosted}");
        assert_eq!(brain._reward_source(), crate::core::RewardSource::External);
        assert!(take_affective_observation().is_none(), "情感观测被消费即取走");
    }

    #[test]
    fn test_reward_calc_negative_external_preserved() {
        // Q2 P3 防回归: 负外部奖励原样保留 (触发 snapshot restore rewind), 不夹取到 0
        let mut brain = SelfIteratingBrain::new();
        brain._external_reward = Some(-0.3);
        let stage = RewardCalculationStage::new();
        stage.process(&mut brain).expect("reward calc ok");
        assert!((brain._reward() + 0.3).abs() < 1e-9, "负奖励应保留, got {}", brain._reward());
        assert_eq!(brain._reward_source(), crate::core::RewardSource::External);
    }

    #[test]
    fn test_reward_calc_external_unchanged_without_affective() {
        // Q2 P3 防回归: 无情感候选时外部奖励原样透传
        let mut brain = SelfIteratingBrain::new();
        brain._external_reward = Some(0.5);
        let stage = RewardCalculationStage::new();
        stage.process(&mut brain).expect("reward calc ok");
        assert!((brain._reward() - 0.5).abs() < 1e-9, "应原样透传, got {}", brain._reward());
    }

// ── W3.6 (batch3 2026-08-26) 步级信用分歧审计验收 ──

#[test]
fn constant_rewards_yield_zero_divergence() {
    let rewards: Vec<(usize, f64)> = (0..10).map(|i| (i, 0.5)).collect();
    let r = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::pipeline::compute_credit_divergence(&rewards, 0.95);
    assert!(r.divergent_steps.is_empty(), "{:?}", r.divergent_steps);
    assert!(r.max_divergence < 1e-9);
}

#[test]
fn late_spike_flagged_as_unsung_hero() {
    // 前 8 步低奖励, 最后一步高奖励 → 回传信用把前期步抬为铺垫 (unsung_hero)
    let mut rewards: Vec<(usize, f64)> = (0..9).map(|i| (i, 0.1)).collect();
    rewards.push((9, 1.0));
    let r = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::pipeline::compute_credit_divergence(&rewards, 0.95);
    assert!(!r.divergent_steps.is_empty());
    assert!(
        r.divergent_steps.iter().any(|d| d.label == "unsung_hero"),
        "{:?}",
        r.divergent_steps
    );
}

#[test]
fn early_spike_only_flags_lucky_start_on_itself() {
    // 仅首步高奖励且后续为负 → 首步 raw_z 高但 backprop 被负未来拖低
    let mut rewards: Vec<(usize, f64)> = vec![(0, 1.0)];
    rewards.extend((1..10).map(|i| (i, -0.2)));
    let r = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::pipeline::compute_credit_divergence(&rewards, 0.95);
    assert!(
        r.divergent_steps.iter().any(|d| d.step_idx == 0 && d.label == "lucky_start"),
        "{:?}",
        r.divergent_steps
    );
}

#[test]
fn step_credit_audit_stage_runs_and_persists() {
    let mut brain = SelfIteratingBrain::new();
    brain._prm_step_rewards = (0..12).map(|i| (i, if i == 11 { 1.0 } else { 0.1 })).collect();
    let stage = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::pipeline::StepCreditAuditStage::new();
    assert_eq!(stage.name(), "step_credit_audit");
    let out = stage.process(&mut brain).expect("stage runs");
    assert!(matches!(out, crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::pipeline::StageDecision::Continue));
}
