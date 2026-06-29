#[cfg(test)]
mod tests {
    #[test]
    fn test_attention_to_gwt_to_hypercube_chain() {
        // TODO(GWT-MIGRATION): replace with GlobalLatentWorkspace equivalent — calls bridge.broadcast_attention_to_gwt(&mut gwt), gwt.active_specialists()
        use crate::core::nt_core_gwt::module_def::SpecialistType;
        use crate::core::nt_core_hcube::axis::DimensionAxis;
        use crate::core::nt_core_hcube::coord::HyperCoord;
        use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
        use crate::core::nt_core_self::AttentionDomain;
        use crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace;
        use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;

        let mut bridge = ThinkingBridge::new("/tmp");
        let mut gwt = GlobalWorkspace::new(0.3);

        bridge
            .silicon
            .attention_manager
            .stimulate_domain(AttentionDomain::Code, 0.9);
        bridge
            .silicon
            .attention_manager
            .stimulate_domain(AttentionDomain::Planning, 0.7);

        let active = bridge.silicon.attention_manager.active_heads();
        assert!(active.len() >= 2);

        let results = bridge.broadcast_attention_to_gwt(&mut gwt);
        assert!(results.len() >= 2);

        let active_specialists = gwt.active_specialists();
        let has_code = active_specialists
            .iter()
            .any(|m| m.specialist_type == SpecialistType::CodeAnalyzer);
        let has_planner = active_specialists
            .iter()
            .any(|m| m.specialist_type == SpecialistType::Planner);
        assert!(has_code, "GWT should have CodeAnalyzer specialist");
        assert!(has_planner, "GWT should have Planner specialist");

        let mut cube = KnowledgeHyperCube::new();
        let coord = HyperCoord::with(DimensionAxis::CodeUnderstanding, 0.9);
        cube.insert(&coord, "test", "Rust ownership system");
        assert!(!cube.is_empty());

        let items = bridge.recall_from_attention(Some(&cube));
        assert!(
            !items.is_empty(),
            "should recall items for active Code domain"
        );
        assert!(items
            .iter()
            .any(|item| item.domain == AttentionDomain::Code));
    }

    #[test]
    fn test_full_evolution_cycle_affects_archive() {
        use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;

        let mut bridge = ThinkingBridge::new("/tmp");
        let initial = bridge.archive.snapshots.len();

        for _ in 0..5 {
            bridge.run_reflection_cycle();
        }

        bridge.auto_snapshot();
        assert!(bridge.archive.snapshots.len() > initial);
    }

    #[test]
    fn test_trace_rewards_after_evolution_cycle() {
        use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;

        let mut bridge = ThinkingBridge::new("/tmp");
        bridge.run_full_evolution_cycle();
        assert!(!bridge.silicon.thinking_traces.is_empty());
    }

    #[test]
    fn test_cognitive_health_after_multiple_cycles() {
        use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;

        let mut bridge = ThinkingBridge::new("/tmp");

        for _ in 0..5 {
            bridge.run_reflection_cycle();
        }

        let report = bridge.evaluate_cognitive_health();
        assert!(report.attention_health >= 0.0 && report.attention_health <= 1.0);
        assert!(report.stability_score >= 0.0 && report.stability_score <= 1.0);
    }

    #[test]
    fn test_intrinsic_motivation_drives_goal_priority() {
        use crate::neotrix::nt_mind::thinking_bridge::ThinkingBridge;
        use crate::neotrix::nt_mind::GoalLoop;

        let mut bridge = ThinkingBridge::new("/tmp");
        let mut goal_loop = GoalLoop::new();

        bridge.run_reflection_cycle();
        let mot = bridge.compute_motivation();

        goal_loop.set_motivation(mot);

        goal_loop.prioritize_from_motivation();
    }

    #[test]
    fn test_metacognition_quick_scan() {
        let bridge = crate::neotrix::nt_mind::distillation::MetaCognitionBridge::new(".");
        let report = bridge.quick_scan();
        assert!(report.summary.total_count > 0, "Should detect some files");
    }

    #[test]
    fn test_metacognition_full_cycle() {
        let mut bridge = crate::neotrix::nt_mind::distillation::MetaCognitionBridge::new(".");
        let result = bridge.run_full_cycle();
        assert!(result.iteration > 0);
        assert!(result.report.summary.total_count > 0);
        assert!(!result.plans.is_empty() || result.report.summary.total_count == 0);
        assert!(result.health_check.compilation_ok || result.health_check.test_count == 0);
    }

    #[test]
    fn test_metacognition_weakness_patterns() {
        use crate::core::nt_core_meta::{ModuleInfo, SelfModel, WeaknessAnalyzer};
        let analyzer = WeaknessAnalyzer::new();
        let mut model = SelfModel::new();
        model.modules.push(ModuleInfo {
            name: "big_mod".into(),
            path: "src/big/".into(),
            file_count: 1,
            total_lines: 2000,
            test_count: 0,
            has_tests: false,
            unsafe_count: 10,
            unwrap_count: 30,
            todo_count: 10,
            public_api_count: 20,
            description: "".into(),
        });
        let report = analyzer.analyze(&model);
        assert!(
            report.summary.total_count >= 4,
            "Large module + missing tests + unsafe + unwrap"
        );
        assert!(
            report.summary.critical_count >= 1,
            "Should have critical issues"
        );
    }

    #[test]
    fn test_metacognition_planner_prioritization() {
        use crate::core::nt_core_meta::weakness::Weakness;
        use crate::core::nt_core_meta::EvolutionPlanner;
        let mut planner = EvolutionPlanner::new();
        let weaknesses = vec![
            Weakness {
                pattern_id: "MISSING_TESTS".into(),
                target_module: None,
                file: None,
                line: None,
                severity: crate::core::nt_core_meta::DebtSeverity::Critical,
                description: "no tests".into(),
                impact: "risk".into(),
                suggestion: "add tests".into(),
            },
            Weakness {
                pattern_id: "LARGE_FILE".into(),
                target_module: None,
                file: None,
                line: None,
                severity: crate::core::nt_core_meta::DebtSeverity::Minor,
                description: "big file".into(),
                impact: "hard to maintain".into(),
                suggestion: "split".into(),
            },
        ];
        let plans = planner.plan_from_weaknesses(weaknesses);
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].weakness.pattern_id, "MISSING_TESTS");
        assert_eq!(plans[1].weakness.pattern_id, "LARGE_FILE");
    }

    #[test]
    fn test_metacognition_bridge_reuses_across_cycles() {
        let mut bridge = crate::neotrix::nt_mind::distillation::MetaCognitionBridge::new(".");
        let r1 = bridge.run_full_cycle();
        let r2 = bridge.run_full_cycle();
        assert_eq!(r1.iteration, 1);
        assert_eq!(r2.iteration, 2);
        let summary = bridge.status_summary();
        assert!(summary.contains("MetaCognition Cycle"));
    }
}
