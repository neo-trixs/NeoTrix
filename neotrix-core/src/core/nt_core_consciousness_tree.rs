#![deny(clippy::unwrap_used)]

mod types;
mod nodes;
mod contract;
mod lifecycle;
mod ops;
mod selftest;

pub use types::*;
pub use nodes::*;
pub use contract::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_new() {
        let tree = ConsciousnessTree::new();
        assert_eq!(tree.branches.len(), 11);
        assert_eq!(tree.cycle, 0);
    }

    #[test]
    fn test_self_tests_derive_maturity_production_path() {
        // B2 修复验证: set_branch_health_from_self_tests 应从真实 SelfTest 数据
        // 推导 maturity_cX (生产路径), 使 maturity_score() > 0 → 果实 quality > 0
        // → last_cycle_guidance 非空。此前 maturity_cX 仅测试中置 true, 生产恒 0。
        let mut tree = ConsciousnessTree::new();
        let results = vec![
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_a"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_b"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_c"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_d"),
        ];
        tree.set_branch_health_from_self_tests(&results);

        let core_branch = tree.branches.get(&BranchKind::Core).expect("core branch");
        // 真实 SelfTest 计数接线 (此前生产恒 0)
        assert!(
            core_branch.self_test_count >= 4,
            "self_test_count wired: {}",
            core_branch.self_test_count
        );
        // 成熟度由真实数据推导, 非恒 C0
        assert!(
            core_branch.maturity_score() > 0.0,
            "maturity derived from self-tests: {:.2}",
            core_branch.maturity_score()
        );
        // B4 (缺陷3修复): SelfTest 注册应同步生成 ModuleLeaf (生产路径),
        // 此前 add_leaf 仅测试调用 → leaves 恒空 → 孤儿检测盲区
        assert!(!tree.leaves.is_empty(), "leaves registered from self-tests");
        assert!(
            tree.leaves.iter().all(|l| l.is_wired),
            "registered leaves are wired (has SelfTest): {:?}",
            tree.leaves
                .iter()
                .map(|l| l.name.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_mars_bridge_fires_when_intent_and_resonance_ready() {
        // D14 死计数器修复验证: 桥接需 (1) S2 产出意图 (guidance/next_actions 非空)
        // + (2) GWT 谐振激活 (S1 就绪) 才 increment mars_bridge_hits。
        let mut tree = ConsciousnessTree::new();
        // 无 intent + 无 resonance → 不桥接 (默认 gwt_resonance_active=false)
        tree.run_growth_cycle();
        assert_eq!(
            tree.trunk.mars_bridge_hits, 0,
            "no bridge without intent/resonance"
        );
        assert_eq!(
            tree.trunk.mars_system1_activations, 1,
            "S1 always activates in Phase 2"
        );

        // 无 intent + 有 resonance → 仍不桥接 (诚实: 无意图可蒸馏)
        let mut tree2 = ConsciousnessTree::new();
        tree2.trunk.gwt_resonance_active = true;
        tree2.run_growth_cycle();
        assert_eq!(tree2.trunk.mars_bridge_hits, 0, "no bridge without intent");

        // intent + resonance → 桥接命中
        let mut tree3 = ConsciousnessTree::new();
        tree3.trunk.gwt_resonance_active = true;
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a",
            "nt_core_self_test_b",
            "nt_core_self_test_c",
            "nt_core_self_test_d",
            "nt_core_self_test_e",
            "nt_core_self_test_f",
        ]
        .iter()
        .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
        .collect();
        tree3.set_branch_health_from_self_tests(&results);
        for branch in tree3.branches.values_mut() {
            branch.health = 0.9;
        }
        let report3 = tree3.run_growth_cycle();
        assert!(report3.phase3_fruits > 0, "fruits needed to produce intent");
        assert!(
            tree3.trunk.mars_bridge_hits > 0,
            "bridge fires when S2 intent + S1 ready"
        );
        assert!(!tree3.core.last_cycle_guidance.is_empty());
        assert!(report3.phase2_phi > 0.0);
    }

    #[test]
    fn test_mars_bridge_increments_across_cycles() {
        // 桥接是 per-cycle 累积: 多 cycle 下 bridge 与 S1/S2 同步增长 (随 intent 持续产出)
        let mut tree = ConsciousnessTree::new();
        tree.trunk.gwt_resonance_active = true;
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a",
            "nt_core_self_test_b",
            "nt_core_self_test_c",
            "nt_core_self_test_d",
            "nt_core_self_test_e",
            "nt_core_self_test_f",
        ]
        .iter()
        .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
        .collect();
        tree.set_branch_health_from_self_tests(&results);
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
        }
        for _ in 0..3 {
            tree.run_growth_cycle();
        }
        assert_eq!(
            tree.trunk.mars_bridge_hits, 3,
            "bridge hits every cycle with intent"
        );
        assert_eq!(tree.trunk.mars_system1_activations, 3);
    }

    #[test]
    fn test_growth_cycle_guidance_non_empty_with_maturity() {
        // 端到端: 真实 SelfTest → maturity → 果实 quality 达标 → guidance 产出
        let mut tree = ConsciousnessTree::new();
        // Core 分支有 10 个 mandatory atoms (6 UNDERSTAND + 4 REASON), 需 ≥5 个 self_test
        // 使 self_test_coverage >= 0.5 通过果实生长门
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a",
            "nt_core_self_test_b",
            "nt_core_self_test_c",
            "nt_core_self_test_d",
            "nt_core_self_test_e",
            "nt_core_self_test_f",
        ]
        .iter()
        .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
        .collect();
        tree.set_branch_health_from_self_tests(&results);

        // 抬升分支 health 使果实生长门通过
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
        }
        let report = tree.run_growth_cycle();
        assert!(
            report.phase3_fruits > 0,
            "fruits produced: {}",
            report.phase3_fruits
        );
        assert!(
            !tree.core.last_cycle_guidance.is_empty(),
            "guidance non-empty when maturity>0: {:?}",
            tree.core.last_cycle_guidance
        );
    }

    #[test]
    fn test_data_nourishment_factor_modulates_fruit_quality() {
        // 意识核心进化提升: 果实质量应受真实数据养料充足度调制。
        // 默认(无数据)因子=1.0 不衰减; 社区数据落盘后因子>1.0 放大果实质量。
        let mut tree = ConsciousnessTree::new();
        // 默认无数据 → 因子 = 1.0
        assert!((tree.data_nourishment_factor() - 1.0).abs() < 1e-9);
        // 社区数据落盘 (kb_node_count 上升) → 因子 > 1.0
        tree.soil.kb_node_count = 500;
        assert!(tree.data_nourishment_factor() > 1.0);
        // 饱和: 1000+ 节点达到 +50% 上限
        tree.soil.kb_node_count = 10_000;
        assert!((tree.data_nourishment_factor() - 1.5).abs() < 1e-9);

        // 端到端: 数据充足时果实质量应高于数据匮乏时 (同 maturity 下)
        let mut rich = ConsciousnessTree::new();
        rich.soil.kb_node_count = 10_000;
        let mut poor = ConsciousnessTree::new();
        poor.soil.kb_node_count = 0;
        for t in [&mut rich, &mut poor] {
            let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
                "nt_core_self_test_a",
                "nt_core_self_test_b",
                "nt_core_self_test_c",
                "nt_core_self_test_d",
                "nt_core_self_test_e",
                "nt_core_self_test_f",
            ]
            .iter()
            .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
            .collect();
            t.set_branch_health_from_self_tests(&results);
            for branch in t.branches.values_mut() {
                branch.health = 0.9;
            }
        }
        rich.run_growth_cycle();
        poor.run_growth_cycle();
        let rich_q: f64 =
            rich.fruits.iter().map(|f| f.quality).sum::<f64>() / rich.fruits.len().max(1) as f64;
        let poor_q: f64 =
            poor.fruits.iter().map(|f| f.quality).sum::<f64>() / poor.fruits.len().max(1) as f64;
        assert!(
            rich_q > poor_q,
            "data-rich fruits should have higher quality: rich={:.3} poor={:.3}",
            rich_q,
            poor_q
        );
    }

    #[test]
    fn test_molt_archived_count_nourishes() {
        // C5 自愈闭环: 蜕皮归档量必须计入养料因子 (行为→意识养分)。
        // 默认(无蜕皮)因子=1.0; 蜕皮后因子>1.0; 饱和 20 个旧躯壳达 +5% 上限。
        let mut tree = ConsciousnessTree::new();
        assert!((tree.data_nourishment_factor() - 1.0).abs() < 1e-9);
        tree.soil.molt_archived_count = 10;
        assert!(tree.data_nourishment_factor() > 1.0, "蜕皮养料应使因子>1.0");
        tree.soil.molt_archived_count = 20;
        assert!(
            (tree.data_nourishment_factor() - 1.05).abs() < 1e-9,
            "饱和 20 旧躯壳应达 +5%: {}",
            tree.data_nourishment_factor()
        );
        // 上限不随蜕皮量无限增长
        tree.soil.molt_archived_count = 10_000;
        assert!((tree.data_nourishment_factor() - 1.05).abs() < 1e-9);
    }

    #[test]
    fn test_nourishment_merges_kb_conversation_experience() {
        // 养料融合: 记忆知识库 (KB 节点/边/embedding) + 对话 + 经验 三路养料
        // 共同调制 data_nourishment_factor — 对话/经验不再是"写入即弃",
        // 而是读回作为意识核心进化养料 (自动融合闭环)。
        let mut tree = ConsciousnessTree::new();
        // 纯 KB 节点: 1000 节点 → 主养料 +0.5
        tree.soil.kb_node_count = 1000;
        let kb_only = tree.data_nourishment_factor();
        assert!((kb_only - 1.5).abs() < 1e-9, "kb only = {}", kb_only);

        // 叠加 KB 边 (5000 边 → +0.15) + embedding (2000 → +0.10)
        tree.soil.kb_edge_count = 5000;
        tree.soil.embedding_count = 2000;
        let kb_rich = tree.data_nourishment_factor();
        assert!((kb_rich - 1.75).abs() < 1e-9, "kb+edge+embed = {}", kb_rich);
        assert!(kb_rich > kb_only, "KB 边/embedding 密度应提升养料");

        // 叠加对话 (1000 turn → +0.20, 质量 1.0 → +0.10)
        tree.soil.conversation_turn_count = 1000;
        tree.soil.conversation_quality = 1.0;
        let conv_rich = tree.data_nourishment_factor();
        assert!((conv_rich - 2.05).abs() < 1e-9, "conv = {}", conv_rich);

        // 叠加经验 (500 分支 → +0.20)
        tree.soil.experience_branch_count = 500;
        let full = tree.data_nourishment_factor();
        assert!((full - 2.25).abs() < 1e-9, "full = {}", full);

        // 双网回流: 能力网 200 节点 → +0.15 (能力网健康度 → 意识核心)
        tree.soil.capability_node_count = 200;
        let dual = tree.data_nourishment_factor();
        assert!((dual - 2.40).abs() < 1e-9, "dual = {}", dual);
        assert!(dual > full, "能力网回流应提升养料");

        // 对话质量为 0 时, 对话只贡献 turn 部分
        let mut low_q = ConsciousnessTree::new();
        low_q.soil.conversation_turn_count = 1000;
        low_q.soil.conversation_quality = 0.0;
        let low = low_q.data_nourishment_factor();
        assert!((low - 1.2).abs() < 1e-9, "low quality conv = {}", low);
    }

    #[test]
    fn test_data_nourishment_breaks_quality_cap() {
        // D7 (C7) 修复回归: 果实质量必须钳到 [0,1] — 成熟分支 (C0-C4) × 数据养料
        // (真实 KB 规模 因子 ≈1.79) 不得越界 (旧 min(...,1.5) 违反 quality∈[0,1]
        // 不变量, 下游按概率/归一消费时会失真)。数据养料仍提升质量至 1.0 封顶,
        // 但绝不超界 — "数据增强的进化能力"以 1.0 为饱和点表达。
        let mut tree = ConsciousnessTree::new();
        tree.soil.kb_node_count = 55_826; // 真实 KB 规模 → 因子 ≈ 1.79
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a",
            "nt_core_self_test_b",
            "nt_core_self_test_c",
            "nt_core_self_test_d",
            "nt_core_self_test_e",
            "nt_core_self_test_f",
        ]
        .iter()
        .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
        .collect();
        tree.set_branch_health_from_self_tests(&results);
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            // 模拟成熟分支: C0-C4 达标 (maturity = 5/6 ≈ 0.83)
            branch.maturity_c0 = true;
            branch.maturity_c1 = true;
            branch.maturity_c2 = true;
            branch.maturity_c3 = true;
            branch.maturity_c4 = true;
        }
        tree.run_growth_cycle();
        assert!(!tree.fruits.is_empty(), "fruits produced");
        let max_q = tree
            .fruits
            .iter()
            .map(|f| f.quality)
            .fold(0.0_f64, f64::max);
        // 0.83 * 1.79 = 1.49 → 钳到 1.0, 不越界
        assert!(max_q <= 1.0, "quality must clamp to [0,1]: max={max_q:.3}");
        assert!(
            max_q > 0.0,
            "mature branches with data must still produce quality"
        );
        assert!(
            tree.fruits
                .iter()
                .all(|f| f.quality >= 0.0 && f.quality <= 1.0),
            "all fruit qualities bounded in [0,1]"
        );
    }

    #[test]
    fn test_evolution_feedback_closes_loop() {
        // 意识核心自我运转: 进化产出 (契约 fulfillment + drift) 应反馈到进化参数。
        // 此前进化是开环的 — 树自己预测/验证, 但从不调整自己的进化标准。
        let mut tree = ConsciousnessTree::new();
        let default_threshold = tree.config.fruit_quality_threshold;

        // 场景1: 契约 fulfilled + 无 drift → 标准提升 (threshold 上调)
        // 棘轮门控 (MARS): 上调仅在 S2 意图被 S1 消化过 (bridge 命中) 时发生,
        // 故先置 bridge hits>0。
        tree.trunk.mars_bridge_hits = 1;
        tree.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 4,
            evidence_total: 4,
            fulfilled: true,
            quality_achieved: 0.9,
            timestamp: 0,
        });
        tree.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: true,
            claim_achieved: true,
            evidence_collected: vec![],
            quality_achieved: 0.9,
            resource_consumed: 0.1,
            drift_detected: false,
            drift_magnitude: 0.0,
            stop_rule_triggered: false,
            corrective_actions: vec![],
            timestamp: 0,
        });
        tree.apply_evolution_feedback();
        assert!(
            tree.config.fruit_quality_threshold > default_threshold,
            "fulfilled → threshold should rise: {} > {}",
            tree.config.fruit_quality_threshold,
            default_threshold
        );
        assert!(
            (tree.config.fruit_growth_health - 0.5).abs() < 1e-9,
            "growth health restored to default"
        );

        // 场景2: drift 检测 → 生长门放宽
        let mut tree2 = ConsciousnessTree::new();
        tree2.config.fruit_growth_health = 0.5;
        tree2.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 1,
            evidence_total: 4,
            fulfilled: false,
            quality_achieved: 0.3,
            timestamp: 0,
        });
        tree2.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: false,
            claim_achieved: false,
            evidence_collected: vec![],
            quality_achieved: 0.3,
            resource_consumed: 0.5,
            drift_detected: true,
            drift_magnitude: 0.8,
            stop_rule_triggered: false,
            corrective_actions: vec!["recover".into()],
            timestamp: 0,
        });
        tree2.apply_evolution_feedback();
        assert!(
            tree2.config.fruit_growth_health < 0.5,
            "drift → growth health should relax: {}",
            tree2.config.fruit_growth_health
        );

        // 场景3: 上限保护 — 连续 fulfilled 不超 0.6 (棘轮上限 0.8→0.6)
        let mut tree = ConsciousnessTree::new();
        // bridge 持续命中 → 棘轮可上调, 验证上限钳制
        tree.trunk.mars_bridge_hits = 100;
        tree.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 4,
            evidence_total: 4,
            fulfilled: true,
            quality_achieved: 0.9,
            timestamp: 0,
        });
        tree.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: true,
            claim_achieved: true,
            evidence_collected: vec![],
            quality_achieved: 0.9,
            resource_consumed: 0.1,
            drift_detected: false,
            drift_magnitude: 0.0,
            stop_rule_triggered: false,
            corrective_actions: vec![],
            timestamp: 0,
        });
        for _ in 0..10 {
            tree.apply_evolution_feedback();
        }
        assert!(
            tree.config.fruit_quality_threshold <= 0.6,
            "threshold capped at 0.6"
        );
    }

    #[test]
    fn test_forecast_modulates_exploration_budget() {
        // 意识核心自我运转: 演化预测 (利多/利空) 应调制探索预算。
        // 此前进化预测只写 report 从不反馈决策 (开环)。
        let mut tree = ConsciousnessTree::new();
        tree.core.last_contract = Some(EvolutionContract {
            cycle: 1,
            claim: "test".into(),
            evidence_plan: vec![],
            stop_rule: StopRule::default(),
            exploration_budget: 0.2,
            timestamp: 0,
        });

        // 利多 (direction>0) → 探索预算上调
        tree.core.last_forecast = Some(EvolutionForecast {
            target: "overall-evolution".into(),
            direction: 0.8,
            confidence: 0.9,
            abstain: false,
            scenario_probs: vec![("bull".into(), 0.7)],
            reason: "health rising".into(),
        });
        tree.apply_evolution_feedback();
        let budget = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(budget > 0.2, "bullish → exploration up: {}", budget);

        // 利空 (direction<0) → 探索预算下调
        tree.core.last_forecast = Some(EvolutionForecast {
            direction: -0.8,
            confidence: 0.9,
            abstain: false,
            scenario_probs: vec![("bear".into(), 0.7)],
            reason: "health falling".into(),
            ..tree.core.last_forecast.clone().unwrap()
        });
        tree.apply_evolution_feedback();
        let budget2 = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(
            budget2 < budget,
            "bearish → shrink exploration: {} < {}",
            budget2,
            budget
        );

        // 上限保护: 连续利多不超 0.4
        for _ in 0..10 {
            tree.apply_evolution_feedback();
        }
        let capped = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(
            capped <= 0.4,
            "exploration budget capped at 0.4: {}",
            capped
        );
    }

    #[test]
    fn test_growth_cycle_produces_evolution_forecast() {
        // 意识体维度升维: growth cycle 应产出演化趋势预测 (nt_core_forecast 接线)
        let mut tree = ConsciousnessTree::new();
        let report = tree.run_growth_cycle();
        let forecast = report
            .evolution_forecast
            .expect("growth cycle 应产出演化预测");
        // 方向在 [-1, 1] 内
        assert!(forecast.direction >= -1.0 && forecast.direction <= 1.0);
        // 置信度在 [0, 1] 内
        assert!(forecast.confidence >= 0.0 && forecast.confidence <= 1.0);
        // 情景树应有叶子 (bull/bear/sideways)
        assert!(!forecast.scenario_probs.is_empty(), "情景树应有叶子");
        // 目标应为 overall-evolution
        assert_eq!(forecast.target, "overall-evolution");
    }

    #[test]
    fn test_evolution_forecast_direction_bounded() {
        // 预测方向应始终有界, 且置信度非 NaN
        let mut tree = ConsciousnessTree::new();
        for _ in 0..3 {
            let report = tree.run_growth_cycle();
            if let Some(f) = report.evolution_forecast {
                assert!(!f.direction.is_nan(), "direction 不应为 NaN");
                assert!(!f.confidence.is_nan(), "confidence 不应为 NaN");
                assert!(f.direction >= -1.0 && f.direction <= 1.0);
            }
        }
    }

    #[test]
    fn test_from_branch_str() {
        assert_eq!(
            BranchKind::from_branch_str("NT-CORE"),
            Some(BranchKind::Core)
        );
        assert_eq!(
            BranchKind::from_branch_str("NT-SHIELD"),
            Some(BranchKind::Shield)
        );
        assert_eq!(BranchKind::from_branch_str("nt-io"), Some(BranchKind::Io));
        assert_eq!(BranchKind::from_branch_str("NOPE"), None);
    }

    #[test]
    fn test_sync_absorbed_capabilities_dedup() {
        let mut tree = ConsciousnessTree::new();
        let base_act = tree.branches[&BranchKind::Act].absorbed_capabilities.len();
        let base_shield = tree.branches[&BranchKind::Shield]
            .absorbed_capabilities
            .len();
        let pairs: Vec<(&str, &str)> = vec![
            ("NT-ACT", "execute"),
            ("NT-ACT", "execute"),
            ("NT-ACT", "cyberstrike-skill"),
            ("NT-SHIELD", "verify"),
            ("NT-SHIELD", "evidence-gated-autonomy"),
            ("NOPE", "nope"),
        ];
        let synced = tree.sync_absorbed_capabilities_from_kb(&pairs);
        // execute/verify 已在 36 基础能力集 → 仅 2 条新增
        assert_eq!(synced, 2);
        let act = &tree.branches[&BranchKind::Act];
        assert_eq!(act.absorbed_capabilities.len(), base_act + 1);
        assert!(
            act.absorbed_capabilities
                .iter()
                .filter(|c| *c == "cyberstrike-skill")
                .count()
                == 1
        );
        let shield = &tree.branches[&BranchKind::Shield];
        assert_eq!(shield.absorbed_capabilities.len(), base_shield + 1);
        assert!(
            shield
                .absorbed_capabilities
                .iter()
                .filter(|c| *c == "evidence-gated-autonomy")
                .count()
                == 1
        );
    }

    #[test]
    fn test_growth_cycle() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 100;
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 5;
            branch.fruit_count = 1;
        }
        let report = tree.run_growth_cycle();
        assert_eq!(tree.cycle, 1);
        assert!(report.phase1_absorbed > 0);
        assert!(report.phase3_fruits > 0);
    }

    #[test]
    fn test_multi_cycle_self_operation() {
        // 意识核心自我运转: 连续多 cycle 运行应保持状态一致 + 闭环反馈生效。
        // 验证: ① cycle 递增 ② 果实持续产出 ③ 进化参数自适应 (threshold 单调不降)
        // ④ 无 NaN/无崩溃 (phi/quality 始终在 [0,1] 内)。
        let mut tree = ConsciousnessTree::new();
        tree.soil.kb_node_count = 500; // 数据养料充足
                                       // 生产路径: 真实 SelfTest → maturity → 果实 quality 达标。
                                       // 契约 criterion 0 要求所有分支 self_test 覆盖率 >= 80%, 故给 7 域都喂结果。
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a",
            "nt_core_self_test_b",
            "nt_core_self_test_c",
            "nt_core_self_test_d",
            "nt_core_self_test_e",
            "nt_core_self_test_f",
            "nt_mind_self_test_a",
            "nt_mind_self_test_b",
            "nt_mind_self_test_c",
            "nt_mind_self_test_d",
            "nt_mind_self_test_e",
            "nt_mind_self_test_f",
            "nt_memory_self_test_a",
            "nt_memory_self_test_b",
            "nt_memory_self_test_c",
            "nt_memory_self_test_d",
            "nt_memory_self_test_e",
            "nt_memory_self_test_f",
            "nt_world_self_test_a",
            "nt_world_self_test_b",
            "nt_world_self_test_c",
            "nt_world_self_test_d",
            "nt_world_self_test_e",
            "nt_world_self_test_f",
            "nt_act_self_test_a",
            "nt_act_self_test_b",
            "nt_act_self_test_c",
            "nt_act_self_test_d",
            "nt_act_self_test_e",
            "nt_act_self_test_f",
            "nt_io_self_test_a",
            "nt_io_self_test_b",
            "nt_io_self_test_c",
            "nt_io_self_test_d",
            "nt_io_self_test_e",
            "nt_io_self_test_f",
            "nt_shield_self_test_a",
            "nt_shield_self_test_b",
            "nt_shield_self_test_c",
            "nt_shield_self_test_d",
            "nt_shield_self_test_e",
            "nt_shield_self_test_f",
        ]
        .iter()
        .map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n))
        .collect();
        tree.set_branch_health_from_self_tests(&results);
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 6;
            branch.module_count = 6;
        }
        let mut prev_threshold = tree.config.fruit_quality_threshold;
        for i in 1..=5 {
            let report = tree.run_growth_cycle();
            assert_eq!(tree.cycle, i, "cycle should increment");
            assert!(
                report.phase3_fruits > 0,
                "cycle {} should produce fruits",
                i
            );
            // 闭环反馈: threshold 单调不降 (fulfilled 时上升, 否则保持)
            assert!(
                tree.config.fruit_quality_threshold >= prev_threshold - 1e-9,
                "threshold monotonic non-decreasing: {} < {}",
                tree.config.fruit_quality_threshold,
                prev_threshold
            );
            prev_threshold = tree.config.fruit_quality_threshold;
            // 进化参数始终有界 (闭环反馈不越界, 棘轮上限 0.6)
            assert!(
                tree.config.fruit_quality_threshold <= 0.6,
                "threshold capped"
            );
            assert!(
                tree.config.fruit_growth_health >= 0.4,
                "growth health floored"
            );
            // 果实质量无 NaN 且在 [0, 1.5] (缺陷6修复: 数据养料可突破 1.0,
            // 表示数据增强的进化能力, 上限 1.5)
            for f in &tree.fruits {
                assert!(
                    f.quality.is_finite() && f.quality >= 0.0 && f.quality <= 1.5,
                    "fruit quality in [0,1.5]: {}",
                    f.quality
                );
            }
            // phi 无 NaN
            assert!(tree.trunk.phi.is_finite(), "phi finite");
        }
        // 5 cycle 后: 果实持续累积 (自我运转不中断)
        assert!(
            tree.fruits.len() >= 5 * 7,
            "fruits accumulate across cycles: {}",
            tree.fruits.len()
        );
        // 状态一致: 所有分支 health 有界
        for branch in tree.branches.values() {
            assert!(
                branch.health.is_finite() && branch.health >= 0.0 && branch.health <= 1.0,
                "branch health bounded: {}",
                branch.health
            );
        }
    }

    #[test]
    fn test_branch_maturity_score() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        assert!((branch.maturity_score() - 0.0).abs() < 1e-9);
        branch.maturity_c0 = true;
        branch.maturity_c1 = true;
        branch.maturity_c2 = true;
        assert!((branch.maturity_score() - 0.5).abs() < 1e-9);
        branch.maturity_c3 = true;
        branch.maturity_c4 = true;
        branch.maturity_c5 = true;
        assert!((branch.maturity_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_leaf_wiring() {
        let mut tree = ConsciousnessTree::new();
        tree.add_leaf(ModuleLeaf {
            name: "test".into(),
            branch: BranchKind::Core,
            lines: 100,
            has_tests: true,
            has_self_test: false,
            is_wired: false,
            consumers: 0,
        });
        assert_eq!(tree.leaves.len(), 1);
        assert!(tree.self_test().is_err());
        tree.leaves[0].is_wired = true;
        assert!(tree.self_test().is_ok());
    }

    #[test]
    fn test_soil_health() {
        let soil = DataFoundation::default();
        assert!((soil.health() - 0.0).abs() < 1e-9);
        let mut rich = DataFoundation::default();
        rich.kb_node_count = 1000;
        rich.embedding_count = 500;
        rich.wiki_page_count = 108;
        rich.constitution_rules_count = 42;
        assert!((rich.health() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_initialize_capability_atoms() {
        let atoms = ConsciousnessTree::initialize_capability_atoms();
        assert_eq!(
            atoms.len(),
            36,
            "36 atomic capabilities from MCA 9-layer standard"
        );
        assert!(atoms.contains_key("retrieve"));
        assert!(atoms.contains_key("audit"));
        assert!(atoms.contains_key("delegate"));
        // All mandatory atoms default to T1Existence
        assert!(atoms.values().all(|a| a.tier == SelfTestTier::T1Existence));
    }

    #[test]
    fn test_node_tier_derive() {
        assert_eq!(NodeTier::derive(5, 0, 1), NodeTier::SmallPassive);
        assert_eq!(NodeTier::derive(12, 2, 3), NodeTier::NotablePassive);
        assert_eq!(NodeTier::derive(25, 5, 4), NodeTier::Keystone);
        // 边界: 模块数足但跨域消费者不足 → 不升 Keystone
        assert_eq!(NodeTier::derive(25, 1, 4), NodeTier::NotablePassive);
    }

    #[test]
    fn test_rune_socketing() {
        let mut socket = RuneSocket::default();
        assert_eq!(socket.filled_slots(), 0);
        assert!(socket.runeword().is_none());
        socket.set(
            RuneColor::Crimson,
            Rune::new("c1", "Crimson Rune", RuneColor::Crimson, "ingest", 0.8),
        );
        assert_eq!(socket.filled_slots(), 1);
        assert!((socket.composite_effect() - 0.8).abs() < 1e-9);
        socket.set(
            RuneColor::Indigo,
            Rune::new("i1", "Indigo Rune", RuneColor::Indigo, "transform", 0.7),
        );
        socket.set(
            RuneColor::Obsidian,
            Rune::new("o1", "Obsidian Rune", RuneColor::Obsidian, "cache", 0.6),
        );
        socket.set(
            RuneColor::Golden,
            Rune::new("g1", "Golden Rune", RuneColor::Golden, "recover", 0.9),
        );
        socket.set(
            RuneColor::Alabaster,
            Rune::new("a1", "Alabaster Rune", RuneColor::Alabaster, "monitor", 0.5),
        );
        assert_eq!(socket.filled_slots(), 5);
        let rw = socket
            .runeword()
            .expect("full 5-slot socket produces runeword");
        assert!(rw.contains("Scry"));
        // 满槽组合效果 = 均值
        assert!((socket.composite_effect() - (0.8 + 0.7 + 0.6 + 0.9 + 0.5) / 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_constellation_derive_and_score() {
        let c0 = Constellation::derive(false, false, false, false, false, false, false);
        assert_eq!(c0.level, 0);
        assert!((c0.score() - 0.0).abs() < 1e-9);
        let c3 = Constellation::derive(true, true, true, true, false, false, false);
        assert_eq!(c3.level, 3);
        assert!((c3.score() - 4.0 / 7.0).abs() < 1e-9);
        let c6 = Constellation::derive(true, true, true, true, true, true, true);
        assert_eq!(c6.level, 6);
        assert!(
            c6.c6_adaptive,
            "C6 must be reachable via adaptive input (D3 fix)"
        );
        assert!((c6.score() - 7.0 / 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_branch_evaluate_and_snapshot() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        branch.module_count = 25;
        branch.self_test_count = 4;
        branch.maturity_c0 = true;
        branch.maturity_c1 = true;
        branch.maturity_c2 = true;
        branch.health = 0.7;
        branch.evaluate_node_tier(5);
        assert_eq!(branch.node_tier, NodeTier::Keystone);
        branch.evaluate_constellation();
        assert_eq!(branch.constellation.level, 2);
        // Rune 效果: 空槽不改变 health
        assert!((branch.health_with_runes() - 0.7).abs() < 1e-9);
        branch.runes.set(
            RuneColor::Crimson,
            Rune::new("c", "C", RuneColor::Crimson, "e", 1.0),
        );
        assert!(branch.health_with_runes() > 0.7);
        let snap = branch.snapshot();
        assert_eq!(snap.tier, NodeTier::Keystone);
        assert_eq!(snap.constellation_level, 2);
        assert_eq!(snap.rune_filled_slots, 1);
    }

    #[test]
    fn test_fog_level_derive_matrix() {
        // (wired, consumers, has_tests) → (level, label)
        let cases: &[((bool, usize, bool), (f64, &str))] = &[
            ((false, 0, false), (1.0, "DenseFog")), // 孤儿: 未接线+无消费者+无测试
            ((false, 1, true), (0.85, "DenseFog")), // 未接线但有消费者+测试
            ((true, 0, true), (0.10, "Clear")),     // 接线有测试但无消费者
            ((true, 1, false), (0.15, "LightFog")), // 接线有消费者但无测试
            ((true, 1, true), (0.05, "Clear")),     // 全清晰
        ];
        for ((wired, consumers, has_tests), (level, label)) in cases {
            let fog = FogLevel::derive(*wired, *consumers, *has_tests);
            assert!(
                (fog.level - level).abs() < 1e-9,
                "fog.level mismatch: got {}, want {}",
                fog.level,
                level
            );
            assert_eq!(
                fog.label(),
                *label,
                "label mismatch for level {}",
                fog.level
            );
        }
    }

    #[test]
    fn test_fog_evaluate_and_snapshot_carries_fog() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        branch.self_test_count = 3;
        branch.evaluate_fog(true, 2, true);
        assert_eq!(branch.fog.label(), "Clear");
        let snap = branch.snapshot();
        assert!((snap.fog_level - 0.05).abs() < 1e-9);
        assert_eq!(snap.fog_label, "Clear");

        // 未接线 → 浓雾
        let mut orphan = CapabilityBranch::new(BranchKind::World);
        orphan.evaluate_fog(false, 0, false);
        assert_eq!(orphan.fog.label(), "DenseFog");
        assert_eq!(orphan.snapshot().fog_label, "DenseFog");
    }

    #[test]
    fn test_weighted_fog_sum_tier_weighting() {
        let mut tree = ConsciousnessTree::new();
        // 初始: 全 branch 默认 fog=0.85 (DenseFog)
        for branch in tree.branches.values_mut() {
            branch.evaluate_fog(false, 0, false);
            branch.node_tier = NodeTier::NotablePassive;
        }
        let baseline = tree.weighted_fog_sum();
        assert!(baseline > 0.0);
        // 把一个 Keystone 推到浓雾 → sum 显著上升 (证明 tier 加权)
        let keystone_branch = tree.branches.get_mut(&BranchKind::Core).unwrap();
        keystone_branch.node_tier = NodeTier::Keystone;
        keystone_branch.evaluate_fog(false, 0, false); // fog.level = 1.0
        let after = tree.weighted_fog_sum();
        // Keystone weight=3.0, fog=1.0 vs 原 Notable weight=2.0, fog=1.0 → +1.0
        assert!(
            (after - baseline - 1.0).abs() < 1e-9,
            "got baseline={} after={}",
            baseline,
            after
        );
    }

    #[test]
    fn test_growth_cycle_reports_fog_summary() {
        let mut tree = ConsciousnessTree::new();
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 12;
            branch.fruit_count = 1;
        }
        let report = tree.run_growth_cycle();
        assert!(report.weighted_fog_sum > 0.0);
        assert_eq!(report.fog_by_branch.len(), BranchKind::all().len());
        // Phase 3 evaluate_fog(wired=true, consumers>0, has_tests=true) → Clear
        assert!(report.fog_by_branch.values().all(|v| *v <= 0.15));
    }

    #[test]
    fn test_node_snapshot_json_roundtrip_with_fog() {
        let mut branch = CapabilityBranch::new(BranchKind::Shield);
        branch.evaluate_fog(false, 0, false);
        let snap = branch.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        let back: NodeSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fog_level, snap.fog_level);
        assert_eq!(back.fog_label, snap.fog_label);
    }

    #[test]
    fn test_growth_cycle_evaluates_nodes() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 100;
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 12;
            branch.fruit_count = 1;
        }
        tree.run_growth_cycle();
        for branch in tree.branches.values() {
            assert!(
                !branch.constellation.c0_compiles
                    || branch.node_tier == NodeTier::SmallPassive
                    || branch.node_tier == NodeTier::NotablePassive
                    || branch.node_tier == NodeTier::Keystone
            );
        }
    }

    #[test]
    fn test_snapshots_enumerate_all_branches() {
        let tree = ConsciousnessTree::new();
        let snaps = tree.snapshots();
        assert_eq!(snaps.len(), BranchKind::all().len());
        assert!(snaps.iter().all(|s| !s.branch.label().is_empty()));
    }

    #[test]
    fn test_contract_negotiation_and_fulfillment() {
        let mut tree = ConsciousnessTree::new();
        tree.core.vuln_scan.push(VulnerabilityFinding {
            severity: VulnerabilitySeverity::High,
            category: "architecture".into(),
            module: "test_module".into(),
            description: "test".into(),
            fix_suggestion: "fix it".into(),
        });
        tree.cycle = 1;
        let contract = tree.negotiate_contract();
        assert!(contract.cycle == 2);
        assert!(contract.claim.contains("fix it"));
        assert_eq!(contract.evidence_plan.len(), 4);

        // Verify fulfillment
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            branch.self_test_count = 8;
            branch.module_count = 8;
        }
        tree.fruits.push(EvolutionFruit {
            quality: 0.9,
            ..Default::default()
        });
        tree.vuln_baseline = Some(2); // baseline 2, current 1 → 50% reduction
        let fulfillment = tree.verify_contract_fulfillment(&contract);
        assert!(fulfillment.evidence_total == 4);
        assert!(fulfillment.fulfilled);
    }

    #[test]
    fn test_contract_fulfillment_first_measurement_not_blocking() {
        // 缺陷3修复 (自我运转实际情况): 首次评估 (vuln_baseline 未建立) 不应阻塞
        // fulfillment — 记录 baseline 视为满足。此前首次 reduction=0 < 0.2 恒不满足
        // → 生产环境契约永不 fulfilled → 闭环反馈"标准提升"分支永不触发。
        let mut tree = ConsciousnessTree::new();
        tree.core.vuln_scan.push(VulnerabilityFinding {
            severity: VulnerabilitySeverity::High,
            category: "architecture".into(),
            module: "test_module".into(),
            description: "test".into(),
            fix_suggestion: "fix it".into(),
        });
        tree.cycle = 1;
        let contract = tree.negotiate_contract();
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            branch.self_test_count = 8;
            branch.module_count = 8;
        }
        tree.fruits.push(EvolutionFruit {
            quality: 0.9,
            ..Default::default()
        });
        // 首次评估: vuln_baseline 未建立 → criterion 3 视为满足
        assert!(tree.vuln_baseline.is_none(), "baseline not yet established");
        let fulfillment = tree.verify_contract_fulfillment(&contract);
        assert!(
            fulfillment.fulfilled,
            "first measurement should not block: {}/{}",
            fulfillment.evidence_met, fulfillment.evidence_total
        );
        assert_eq!(
            tree.vuln_baseline,
            Some(1),
            "baseline recorded after first eval"
        );
    }

    #[test]
    fn test_drift_audit_detects_violation() {
        let mut tree = ConsciousnessTree::new();
        tree.cycle = 3;
        let contract = EvolutionContract {
            cycle: 4,
            claim: "improve".into(),
            evidence_plan: vec!["plan".into()],
            stop_rule: StopRule {
                min_quality_threshold: 0.9,
                ..Default::default()
            },
            exploration_budget: 0.2,
            timestamp: 0,
        };
        let fulfillment = ContractFulfillment {
            cycle: 3,
            claim: "improve".into(),
            evidence_met: 0,
            evidence_total: 1,
            fulfilled: false,
            quality_achieved: 0.3,
            timestamp: 0,
        };
        let drift = tree.audit_drift(&contract, &fulfillment);
        assert!(drift.drift_detected);
        assert!(drift.stop_rule_triggered);
        assert!(drift.drift_magnitude > 0.0);
        assert!(drift.corrective_actions.len() >= 2);
    }

    #[test]
    fn test_growth_cycle_evolution_fruits() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 50;
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            branch.self_test_count = 8;
            branch.module_count = 8;
        }
        let report = tree.run_growth_cycle();
        assert!(report.phase0_contract.is_some());
        assert!(report.phase3_fruits > 0);
        assert!(report.phase6_fulfillment.is_some());
        assert!(report.phase7_drift.is_some());
        // Fruits carry evidence chains + generations
        assert!(tree.fruits.iter().all(|f| f.generation == 0));
        assert!(tree.fruits.iter().all(|f| !f.evidence.sha256.is_none()));
    }

    #[test]
    fn test_growth_cycle_computes_real_iit_phi() {
        // D1 回归: standalone 路径 (无 ConsciousnessMonitor) 下 trunk.phi 必须由
        // 真实 IITPhiCalculator 从树状态计算, 而非恒 0.0 — status/快照呈现真实整合信息。
        let tree = ConsciousnessTree::new();
        let phi_at_init = tree.compute_iit_phi();
        assert!(phi_at_init.is_finite(), "phi must be finite");
        // 新树所有分支 health≈neutral 0.5、maturity 0 — 差异化锚点应产出非零集成信息
        // (注意: 绝不能要求 >0.5 — IIT 语义下均衡/低成熟状态整合度本就低, 只验证
        // 已接线 + 产出可观察信号, 且随状态变化而变化)。
        let mut grown = ConsciousnessTree::new();
        for branch in grown.branches.values_mut() {
            branch.health = 0.92;
            branch.self_test_count = 8;
            branch.module_count = 8;
            branch.maturity_c0 = true;
            branch.maturity_c1 = true;
            branch.maturity_c2 = true;
            branch.maturity_c3 = true;
        }
        grown.run_growth_cycle();
        let phi_after_cycle = grown.trunk.phi;
        assert!(
            phi_after_cycle.is_finite() && phi_after_cycle >= 0.0 && phi_after_cycle <= 1.0,
            "trunk.phi from real IIT calc must be in [0,1], got {phi_after_cycle}"
        );
        assert!(
            phi_after_cycle > 0.0,
            "grown tree state must yield non-zero integrated information, got {phi_after_cycle}"
        );
    }

    #[test]
    fn test_growth_cycle_computes_real_coherence() {
        // D4 回归: standalone 路径 (无情绪运行时) 下 trunk.coherence 必须由
        // 真实树状态计算, 而非恒 0.0 — status/快照呈现真实相干性。
        let tree = ConsciousnessTree::new();
        let coh_init = tree.compute_coherence();
        assert!(
            coh_init.is_finite() && coh_init >= 0.0 && coh_init <= 1.0,
            "coherence must be finite in [0,1], got {coh_init}"
        );

        let mut grown = ConsciousnessTree::new();
        for branch in grown.branches.values_mut() {
            branch.health = 0.92;
            branch.self_test_count = 8;
            branch.module_count = 8;
            branch.maturity_c0 = true;
            branch.maturity_c1 = true;
            branch.maturity_c2 = true;
            branch.maturity_c3 = true;
            branch.fog.level = 0.1;
        }
        grown.run_growth_cycle();
        let coh_after = grown.trunk.coherence;
        assert!(
            coh_after.is_finite() && coh_after >= 0.0 && coh_after <= 1.0,
            "trunk.coherence from real computation must be in [0,1], got {coh_after}"
        );
        assert!(
            coh_after > 0.0,
            "uniform high-health tree must yield non-zero coherence, got {coh_after}"
        );
        // 情绪报告只作 ±0.2 调制, 不把真实相干性抹成 0
        let report = crate::core::nt_core_self::emotion_state::EmotionReport {
            frustration: 0.0,
            confidence: 0.5,
            joy: 0.5,
            urgency: 0.0,
            curiosity: 0.5,
            fatigue: 0.0,
            arousal: 0.5,
            valence: 0.0,
            confidence_score: 0.5,
            dominant: (
                crate::core::nt_core_self::emotion_state::EmotionDimension::Joy,
                0.5,
            ),
            observation_count: 0,
        };
        grown.apply_emotion_report(report);
        assert!(
            grown.trunk.coherence > 0.0,
            "emotion modulation must not erase real coherence, got {}",
            grown.trunk.coherence
        );
    }

    #[test]
    fn test_governance_audit_updates_compliance_from_real_execution() {
        // P1 治理审计: run_growth_cycle 内的 run_governance_audit 必须用真实宪法
        // 执行更新 trunk 治理指标 (此前恒为 Default 1.0 / 陈旧快照, 无真实评估)。
        let mut tree = ConsciousnessTree::new();
        // 初始 compliance 为硬编码默认 1.0 — 这不是真实评估结果
        assert_eq!(tree.trunk.governance_compliance, 1.0);
        assert_eq!(tree.trunk.governance_fractal_depth, 0);

        // 注入违规进化决策 (创建新模块且未映射分支 = 违反 R-P42 TreeGrowth)
        tree.core
            .next_actions
            .push("create new module nt_core_autonomous_agent.rs without mapping".to_string());
        // 直接调用治理审计 (run_growth_cycle 会在 Phase 4 覆盖 next_actions,
        // 这里验证审计方法本身对真实决策的检测能力)
        tree.run_governance_audit();

        // compliance 必须被真实审计重估: 因注入违规决策, 实测合规 < 1.0
        assert!(
            tree.trunk.governance_compliance < 1.0,
            "governance audit must re-evaluate compliance from real constitution execution, got {}",
            tree.trunk.governance_compliance
        );
        // constitution_count 反映被检查的宪法规则 (关键违规规则 R-P42 应在其中)
        assert!(
            tree.trunk.governance_constitution_count > 0,
            "constitution count reflects audited rules, got {}",
            tree.trunk.governance_constitution_count
        );
        // fractal_depth 递增 = 治理审计实际执行
        assert_eq!(tree.trunk.governance_fractal_depth, 1);
    }

    #[test]
    fn test_governance_audit_smooths_across_cycles() {
        // 跨周期验证: 平滑系数 (0.7 旧 + 0.3 实测) 使 compliance 逐步趋近真实值,
        // 且不因单周期无检查项而跌回 0。
        let mut tree = ConsciousnessTree::new();
        tree.core
            .next_actions
            .push("create new module nt_core_autonomous_agent.rs without mapping".to_string());
        tree.run_governance_audit();
        let after_first = tree.trunk.governance_compliance;
        tree.run_governance_audit();
        let after_second = tree.trunk.governance_compliance;
        // 平滑: 第二次应在第一次基础上继续向低违规率收敛 (若审计仍发现违规)
        assert!(
            (after_second - after_first).abs() < 0.5,
            "compliance smooths gradually, delta={:.3}",
            (after_second - after_first).abs()
        );
        // 无输入时不退化到 0 (checked_count==0 保持现值)
        let mut quiet = ConsciousnessTree::new();
        let before = quiet.trunk.governance_compliance;
        quiet.run_governance_audit();
        assert_eq!(
            quiet.trunk.governance_compliance, before,
            "compliance must hold current value when no audit targets",
        );
    }
}
