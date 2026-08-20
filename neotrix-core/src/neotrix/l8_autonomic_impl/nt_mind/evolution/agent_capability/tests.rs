    use super::*;

    fn mem_agent() -> MemoryAgent {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_memagent_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = KnowledgeBase::open(Some(tmp.into())).expect("open kb");
        MemoryAgent { kb: std::sync::Arc::new(kb) }
    }

    #[test]
    fn capability_write_and_retrieve() {
        let agent = mem_agent();
        let w = agent.capability_write("AgentTest", "NeoTrix agent capability wiring", "test").unwrap();
        assert!(matches!(w, CapabilityOutcome::Text(_)));
        let r = agent.capability_retrieve("NeoTrix agent", 5).unwrap();
        match r {
            CapabilityOutcome::Hits(n, first) => {
                assert!(n >= 1);
                assert_eq!(first, "AgentTest");
            }
            _ => panic!("expected hits"),
        }
    }

    #[test]
    fn capability_consolidate_counts() {
        let agent = mem_agent();
        let c = agent.capability_consolidate().unwrap();
        assert!(matches!(c, CapabilityOutcome::Count(_)));
    }

    #[test]
    fn capability_evidence_empty_ok() {
        let agent = mem_agent();
        let e = agent.capability_evidence().unwrap();
        assert!(matches!(e, CapabilityOutcome::Count(_)));
    }

    #[test]
    fn meta_shell_routes_via_attention() {
        let mut shell = MetaAgentShell::new("planning");
        shell.stimulate(AttentionDomain::Planning, 0.9);
        let r = shell.decide_and_run();
        assert!(r.is_some());
        assert_eq!(shell.iterations_run, 1);
    }

    #[test]
    fn meta_shell_no_dominant_no_run() {
        let mut shell = MetaAgentShell::new("planning");
        // 无任何域被刺激 → dominant_domain() 返回 None → 不空转
        let r = shell.decide_and_run();
        assert!(r.is_none());
        assert_eq!(shell.iterations_run, 0);
    }

    #[test]
    fn capability_kind_domain_mapping() {
        assert_eq!(
            MemoryCapabilityKind::Consolidate.attention_domain(),
            AttentionDomain::Semantic
        );
        assert_eq!(
            MemoryCapabilityKind::Retrieve.attention_domain(),
            AttentionDomain::PatternMatch
        );
        assert_eq!(MemoryCapabilityKind::Write.attention_domain(), AttentionDomain::Planning);
    }

    #[test]
    fn meta_shell_dispatches_catalog_profile() {
        // 派单桥回归: decide_and_run 后 last_dispatched 应记录 AgentCatalog 档案名。
        // 刺激 SelfReflection → 应派单 verifier (以判据回滚/防自确认陷阱)。
        let mut shell = MetaAgentShell::new("planning");
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let r = shell.decide_and_run();
        assert!(r.is_some());
        assert_eq!(shell.last_dispatched, Some("verifier"));
    }

    #[test]
    fn meta_shell_route_maps_all_supported_domains() {
        // route_to_catalog 对每个受支持域都返回内置档案名 (星系派单可全映射)。
        let mut shell = MetaAgentShell::new("planning");
        for domain in [
            AttentionDomain::PatternMatch,
            AttentionDomain::Planning,
            AttentionDomain::Code,
            AttentionDomain::Temporal,
            AttentionDomain::SelfReflection,
            AttentionDomain::Semantic,
        ] {
            shell.stimulate(domain, 0.9);
            assert!(
                shell.route_to_catalog().is_some(),
                "domain {:?} should resolve to a catalog agent",
                domain
            );
        }
    }

    #[test]
    fn route_learner_cold_start_uses_static() {
        // 冷启动 (无证据): 学习器不应覆盖静态映射。
        let learner = RouteLearner::new();
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "verifier");
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert!(learner.rates(AttentionDomain::SelfReflection).is_empty());
    }

    #[test]
    fn route_learner_overrides_static_with_evidence() {
        // D3: 该域历史证据显示 generalist 成功率更高 → 学习器应覆盖静态 verifier。
        let mut learner = RouteLearner::new();
        for _ in 0..5 {
            learner.record(AttentionDomain::SelfReflection, "generalist", true);
        }
        for _ in 0..5 {
            learner.record(AttentionDomain::SelfReflection, "verifier", false);
        }
        assert!(learner.has_enough_evidence(AttentionDomain::SelfReflection));
        let routed = learner.route(AttentionDomain::SelfReflection, "verifier");
        assert_eq!(routed, "generalist", "learner should override static mapping");
        // 诊断视图可用
        let rates = learner.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 2);
    }

    #[test]
    fn route_learner_below_evidence_keeps_static() {
        // 证据不足时即使有成功率差异也不覆盖 (防冷启动噪声)。
        let mut learner = RouteLearner::new();
        learner.record(AttentionDomain::SelfReflection, "generalist", true);
        learner.record(AttentionDomain::SelfReflection, "verifier", false);
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "verifier");
    }

    #[test]
    fn route_learner_cold_start_covers_under_evidenced_arms() {
        // P5: 纯利用会饿死备选档案 — 一旦某档案达标即锁定, 未达标备选再无机会
        // 被观察 (死锁)。冷启动覆盖应探索尝试最少的臂, 保证每臂都被采样。
        let mut learner = RouteLearner::with_config(RouteLearnerConfig { min_evidence: 2 });
        learner.record(AttentionDomain::PatternMatch, "researcher", false); // 达标臂, 0 成功
        learner.record(AttentionDomain::PatternMatch, "researcher", false);
        learner.record(AttentionDomain::PatternMatch, "explorer", true); // 未达标备选 (1 次)
        assert!(learner.has_enough_evidence(AttentionDomain::PatternMatch));
        // 旧行为: 纯利用 → researcher (唯一达标臂), explorer 永远不再被采样。
        // 新行为: 探索未达标的 explorer。
        assert_eq!(
            learner.route(AttentionDomain::PatternMatch, "researcher"),
            "explorer",
            "under-evidenced alternative must stay observable"
        );
        // 备选也达标后 → 纯利用, 选成功率最高的臂。
        learner.record(AttentionDomain::PatternMatch, "explorer", true);
        assert_eq!(
            learner.route(AttentionDomain::PatternMatch, "researcher"),
            "explorer",
            "exploit highest-rate arm once all seen arms have evidence"
        );
    }

    #[test]
    fn route_learner_config_is_calibratable() {
        // P1: min_evidence 进配置, 默认 3 保持既有行为; 调低后证据阈值随之变化。
        let cfg = RouteLearnerConfig::default();
        assert_eq!(cfg.min_evidence, 3);
        // 自定义配置构造生效: min_evidence=1 → 1 次成功即覆盖静态映射。
        let mut learner = RouteLearner::with_config(RouteLearnerConfig { min_evidence: 1 });
        learner.record(AttentionDomain::SelfReflection, "generalist", true);
        assert!(learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "generalist");
        // 默认配置下同场景仍不足 (1 < 3)。
        let mut default = RouteLearner::new();
        default.record(AttentionDomain::SelfReflection, "generalist", true);
        assert!(!default.has_enough_evidence(AttentionDomain::SelfReflection));
    }

    #[test]
    fn route_learner_persist_load_round_trip() {
        // P1: 行为统计经 KB kv_store 持久化, 新实例可完整恢复 → 派单学习跨会话存活。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));

        let mut original = RouteLearner::new();
        for _ in 0..5 {
            original.record(AttentionDomain::SelfReflection, "generalist", true);
        }
        for _ in 0..3 {
            original.record(AttentionDomain::SelfReflection, "verifier", false);
        }
        original.persist(&kb).expect("persist");

        // 全新实例 (冷启动) 加载后应还原证据 → 路由覆盖生效。
        let mut restored = RouteLearner::new();
        restored.load(&kb).expect("load");
        assert!(restored.has_enough_evidence(AttentionDomain::SelfReflection));
        let rates = restored.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 2, "both agents restored");
        // generalist 5/5 成功 > verifier 0/3 → 学习后覆盖静态 verifier。
        assert_eq!(restored.route(AttentionDomain::SelfReflection, "verifier"), "generalist");
    }

    #[test]
    fn route_learner_load_missing_archive_stays_cold() {
        // P1: 无存档时 load 为空操作, 不崩溃、不产生证据。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_cold_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut learner = RouteLearner::new();
        learner.load(&kb).expect("load empty should be Ok");
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::PatternMatch, "explorer"), "explorer");
    }

    #[test]
    fn route_learner_persist_drops_unknown_agents() {
        // P1: 持久化仅保留规范档案名 (researcher/explorer/planner/...), 未知名丢弃。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_unk_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut learner = RouteLearner::new();
        learner.record(AttentionDomain::PatternMatch, "researcher", true);
        learner.record(AttentionDomain::PatternMatch, "not-a-real-agent", true);
        learner.persist(&kb).expect("persist");
        let mut restored = RouteLearner::new();
        restored.load(&kb).expect("load");
        let rates = restored.rates(AttentionDomain::PatternMatch);
        assert_eq!(rates.len(), 1, "unknown agent dropped on restore");
        assert_eq!(rates[0].0, "researcher");
    }

    #[test]
    fn dialogue_absorb_config_is_calibratable() {
        // D5: 参数集中可调, 默认值保持既有行为 (8 / 0.1)。
        let cfg = DialogueAbsorbConfig::default();
        assert_eq!(cfg.max_entries, 8);
        assert!((cfg.min_importance - 0.1).abs() < 1e-9);
        assert!((cfg.boost_base - 0.5).abs() < 1e-9);
        // 自定义参数构造生效
        let custom = DialogueAbsorbConfig {
            max_entries: 3,
            min_importance: 0.5,
            ..Default::default()
        };
        assert_eq!(custom.max_entries, 3);
    }

    #[test]
    fn dialogue_absorb_outcome_signals_positive() {
        // D1/D2: is_positive 要求 吸收 + 批评器接受 + 能力面变好 三条件齐备。
        let positive = DialogueAbsorbOutcome {
            absorbed: 2,
            critic_accepted: true,
            score_before: 0.4,
            score_after: 0.6,
            score_delta: 0.2,
        };
        assert!(positive.is_positive());
        // 批评器回滚 → 非正向 (自欺防御)
        let rolled_back = DialogueAbsorbOutcome {
            absorbed: 2,
            critic_accepted: false,
            score_before: 0.5,
            score_after: 0.5,
            score_delta: 0.0,
        };
        assert!(!rolled_back.is_positive());
        // 空结果
        assert!(!DialogueAbsorbOutcome::empty().is_positive());
        assert_eq!(DialogueAbsorbOutcome::empty().absorbed, 0);
    }

    #[test]
    fn dialogue_bridge_derive_vector_content_aware() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let cv = bridge.derive_vector("unit tests assert verify memory recall kb retention");
        assert!(cv.verification() > 0.0, "verification should be boosted");
        assert!(cv.semantic_layer() > 0.0, "semantic_layer should be boosted");
        assert!(cv.analysis() == 0.0, "no analysis keyword → stays 0");
    }

    #[test]
    fn dialogue_bridge_recent_experiences_filters() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_exp_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-test",
            NodeType::Session,
            Some("user asked about seal loop iteration"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        kb.write_memory_entry(
            "not-a-session",
            NodeType::Concept,
            Some("regular node"),
            None,
            Some("test"),
            None,
        )
        .expect("write concept node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let exps = bridge.recent_experiences();
        assert_eq!(exps.len(), 1, "only Session-type node should be collected");
        assert_eq!(exps[0].title, "session-test");
    }

    #[test]
    fn dialogue_bridge_absorb_pending_moves_capability() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_abs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-1",
            NodeType::Session,
            Some("user taught testing verification discipline"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_pending(&mut brain);
        assert!(outcome.absorbed >= 1, "at least one dialogue experience should absorb");
        // 能力面应朝对话信号移动: 初始全零, 吸收后 verification 维度被提升。
        assert!(
            brain.brain.capability.verification() > 0.0,
            "verification dimension should move toward dialogue signal"
        );
        assert!(
            brain.brain.total_absorb_count > 0,
            "absorb count should increase"
        );
        // 源身份已记录 — DialogueExperience 通道写入吸收历史。
        let has_dialogue = brain
            .brain
            .absorption_history
            .iter()
            .any(|rec| rec.source == crate::core::KnowledgeSource::DialogueExperience);
        assert!(has_dialogue, "DialogueExperience should be in absorption history");
    }

    #[test]
    fn dialogue_bridge_absorbs_multiple_sessions() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_multi_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-alpha",
            NodeType::Session,
            Some("testing verification discipline"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        kb.write_memory_entry(
            "session-beta",
            NodeType::Session,
            Some("planning and memory retrieval strategy"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_pending(&mut brain);
        assert_eq!(outcome.absorbed, 2, "two distinct sessions should both absorb");
        // 原则级共振: 两条会话分别在 verification 与 semantic_layer 留下信号。
        assert!(brain.brain.capability.verification() > 0.0);
        assert!(brain.brain.capability.semantic_layer() > 0.0);
        // 行为化指标: outcome 记录了批评器判决 + 实测能力差, 不再是无意义计数。
        assert!(
            outcome.score_before >= 0.0 && outcome.score_after >= 0.0,
            "behavioral score must be measurable"
        );
    }

    fn research_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "NeoTrix Search Backend Fallback".into(),
                url: "https://example.com/search-fallback".into(),
                snippet: "web search method with evidence source citation for verification and synthesis".into(),
                evidence: None,
            },
            SearchResult {
                title: "Ordered Backend Routing Research".into(),
                url: "https://example.com/routing".into(),
                snippet: "research paper arxiv study aggregating benchmark metrics and findings".into(),
                evidence: None,
            },
        ]
    }

    #[test]
    fn research_bridge_derive_research_vector_content_aware() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_vec_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let cv = bridge.derive_research_vector(
            "web search method with evidence source citation paper arxiv benchmark synthesis finding",
        );
        assert!(cv.semantic_layer() > 0.0, "search/web keywords should boost semantic_layer");
        assert!(cv.inference_depth() > 0.0, "paper/research keywords should boost inference_depth");
        assert!(cv.verification() > 0.0, "evidence/source keywords should boost verification");
        // 与对话域映射区分: 无 dialogue 关键词时 inference_depth 不应被对话映射污染。
        let dialogue_cv = bridge.derive_vector("just a chat");
        assert_eq!(dialogue_cv.inference_depth(), 0.0);
    }

    #[test]
    fn dispatch_bridge_derive_dispatch_vector_content_aware() {
        // P6: 派单域关键词映射 — 路由/拓扑/策略/检索/进化信号提升对应维度。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dispatch_vec_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let cv = bridge.derive_dispatch_vector(
            "topology repair revision route agent strategy bandit retrieval hit",
        );
        assert!(cv.ai_native_states() > 0.0, "route/agent keywords should boost ai_native_states");
        assert!(cv.compound_composition() > 0.0, "topology/repair keywords should boost compound_composition");
        assert!(cv.analysis() > 0.0, "strategy/bandit keywords should boost analysis");
        assert!(cv.semantic_layer() > 0.0, "retrieval/hit keywords should boost semantic_layer");
    }

    #[test]
    fn dispatch_bridge_absorb_experiences_moves_capability_and_dedups() {
        // P6: coevo 经验子图 → 大脑吸收闭环 — 水位去重防反复吸收, 新经验继续可吸收。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dispatch_abs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb.clone());
        let mut brain = SelfIteratingBrain::new();

        let mut coevo = CoEvolutionLoop::new();
        coevo.record_reward("dialogue", AttentionDomain::PatternMatch, "explorer", "balanced", true, "retrieved 5 hits");
        coevo.record_reward("dialogue", AttentionDomain::PatternMatch, "researcher", "conservative", false, "search returned empty");

        // 首次吸收: 新经验被消费, 大脑能力面被推动。
        let outcome = bridge.absorb_dispatch_experiences(&mut brain, &mut coevo);
        assert!(outcome.absorbed > 0, "dispatch experiences should be absorbed");
        // 水位推进 → 同一批不再重复吸收。
        assert!(coevo.new_memories_since_watermark().is_empty());
        let again = bridge.absorb_dispatch_experiences(&mut brain, &mut coevo);
        assert_eq!(again.absorbed, 0, "watermark must dedup already-consumed experiences");

        // 新经验 → 下一轮可吸收。
        coevo.record_reward("dialogue", AttentionDomain::Code, "generalist", "exploratory", true, "combined 3 hits, 2 evidence");
        assert_eq!(coevo.new_memories_since_watermark().len(), 1);
        let next = bridge.absorb_dispatch_experiences(&mut brain, &mut coevo);
        assert!(next.absorbed > 0, "new dispatch experience should be absorbable next cycle");
        assert!(coevo.new_memories_since_watermark().is_empty());
    }

    #[test]
    fn research_bridge_findings_absorb_moves_capability() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_abs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb.clone());
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_research_findings(&mut brain, "neotrix search", &research_results());
        // 结论落 KB: 2 条 Source + 1 条 Insight = 3 节点。
        assert_eq!(outcome.absorbed, 3, "2 sources + 1 insight should be written to KB");
        // 能力面吸收信号: 研究域关键词应提升 semantic_layer / inference_depth。
        assert!(
            brain.brain.capability.semantic_layer() > 0.0,
            "research absorb should move semantic_layer"
        );
        // 源身份已记录 — ResearchFindings 通道写入吸收历史。
        let has_research = brain
            .brain
            .absorption_history
            .iter()
            .any(|rec| rec.source == crate::core::KnowledgeSource::ResearchFindings);
        assert!(has_research, "ResearchFindings should be in absorption history");
        // KB 落盘可溯源: 结论以 Source 节点存在。
        let nodes = kb.all_nodes().expect("read kb");
        assert!(
            nodes.iter().any(|n| n.node_type == NodeType::Source),
            "research findings should be stored as Source nodes"
        );
    }

    #[test]
    fn research_bridge_empty_results_noop() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_empty_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        // 空结论 → 不落 KB 不吸收, graceful no-op。
        let outcome = bridge.absorb_research_findings(&mut brain, "query", &[]);
        assert_eq!(outcome.absorbed, 0);
        assert!(!outcome.is_positive());
        assert_eq!(brain.brain.total_absorb_count, 0);
    }

    #[test]
    fn research_source_registered_in_catalog() {
        // KnowledgeSource::ResearchFindings 完整登记: name / all / source_weight / 向量。
        let s = crate::core::KnowledgeSource::ResearchFindings;
        assert_eq!(s.name(), "neotrix-research-findings");
        assert!((s.source_weight() - 0.84).abs() < 1e-9);
        assert!(crate::core::KnowledgeSource::all().contains(&s));
        let cv = s.capability_vector();
        assert!(cv.verification() > 0.0, "research source should carry a real vector");
    }

    // ──────────────────────────────────────────────────────────────
    // P0 派单执行桥测试
    // ──────────────────────────────────────────────────────────────

    /// 探针执行器 — 记录被调用的 (agent, task), 返回可控结果。
    struct ProbeExecutor {
        pub calls: std::cell::RefCell<Vec<(String, String)>>,
        pub respond: AgentExecutionOutcome,
    }

    impl AgentExecutor for ProbeExecutor {
        fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome {
            self.calls.borrow_mut().push((agent.to_string(), task.to_string()));
            self.respond.clone()
        }
    }

    #[test]
    fn domains_for_goal_multi_domain_stimulation() {
        // P0 缺陷 #3 修复: 不同目标语义刺激不同注意力域, 不再永远 SelfReflection。
        let research = domains_for_goal("research the latest papers on agent evolution");
        assert!(
            research.iter().any(|(d, _)| *d == AttentionDomain::PatternMatch),
            "research goal should stimulate PatternMatch"
        );
        let code = domains_for_goal("implement the refactor for core loop");
        assert!(
            code.iter().any(|(d, _)| *d == AttentionDomain::Code),
            "code goal should stimulate Code"
        );
        let review = domains_for_goal("review the diff and verify changes");
        assert!(
            review.iter().any(|(d, _)| *d == AttentionDomain::RiskAssessment),
            "review goal should stimulate RiskAssessment"
        );
        // 空文本 → 弱自省兜底 (防 10 域全空空转)。
        let empty = domains_for_goal("   ");
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].0, AttentionDomain::SelfReflection);
    }

    #[test]
    fn tree_branch_weakness_derived_from_health_fog_constellation() {
        // P2: 分支薄弱度 = (1-health)*0.4 + fog*0.4 + (1-constellation)*0.2。
        // 健康分支 (满 health, 低 fog, 满 constellation) → 0。
        let mut healthy = CapabilityBranch::new(BranchKind::Memory);
        healthy.health = 1.0;
        healthy.fog = crate::core::nt_core_consciousness_tree::FogLevel {
            wired: true,
            consumer_count: 3,
            has_tests: true,
            level: 0.05,
        };
        healthy.constellation = crate::core::nt_core_consciousness_tree::Constellation {
            level: 5,
            c0_compiles: true,
            c1_unit_tests: true,
            c2_integration: true,
            c3_benchmark: true,
            c4_pipeline: true,
            c5_self_healing: true,
            c6_adaptive: true,
        };
        let w_healthy = branch_weakness(&healthy);
        assert!(w_healthy < 0.1, "healthy branch should be ~0, got {w_healthy}");
        // 薄弱分支 (health 0, 高 fog, C0) → 接近 1。
        let mut weak = CapabilityBranch::new(BranchKind::Memory);
        weak.health = 0.0;
        let w_weak = branch_weakness(&weak);
        assert!(w_weak > 0.7, "weak branch should be high, got {w_weak}");
    }

    #[test]
    fn tree_branch_stimuli_skips_healthy_drives_weak() {
        // P2: 树作为控制面 — 健康分支不产生刺激, 薄弱分支驱动对应注意力域。
        let mut tree = ConsciousnessTree::new();
        // Memory 分支设健康 (低薄弱度) → 不应产生 Semantic 刺激。
        if let Some(mem) = tree.branches.get_mut(&BranchKind::Memory) {
            mem.health = 1.0;
            mem.fog = crate::core::nt_core_consciousness_tree::FogLevel {
                wired: true,
                consumer_count: 3,
                has_tests: true,
                level: 0.05,
            };
            mem.constellation = crate::core::nt_core_consciousness_tree::Constellation {
                level: 5,
                c0_compiles: true,
                c1_unit_tests: true,
                c2_integration: true,
                c3_benchmark: true,
                c4_pipeline: true,
                c5_self_healing: true,
                c6_adaptive: true,
            };
        }
        // Shield 分支保持默认 (health 0, fog 0.85, C0) → 薄弱 → 应刺激 RiskAssessment。
        // Nexus 分支默认薄弱且映射 Semantic → 也设为健康, 使断言聚焦"健康分支不刺激"。
        if let Some(nx) = tree.branches.get_mut(&BranchKind::Nexus) {
            nx.health = 1.0;
            nx.fog = crate::core::nt_core_consciousness_tree::FogLevel {
                wired: true,
                consumer_count: 3,
                has_tests: true,
                level: 0.05,
            };
            nx.constellation = crate::core::nt_core_consciousness_tree::Constellation {
                level: 5,
                c0_compiles: true,
                c1_unit_tests: true,
                c2_integration: true,
                c3_benchmark: true,
                c4_pipeline: true,
                c5_self_healing: true,
                c6_adaptive: true,
            };
        }
        let stimuli = tree_branch_stimuli(&tree);
        assert!(
            stimuli.iter().any(|(d, _)| *d == AttentionDomain::RiskAssessment),
            "weak Shield branch should stimulate RiskAssessment"
        );
        assert!(
            !stimuli.iter().any(|(d, _)| *d == AttentionDomain::Semantic),
            "healthy Memory branch should not stimulate Semantic"
        );
    }

    #[test]
    fn tree_branch_stimuli_fuses_with_goal_dispatch() {
        // P2: 端到端 — 薄弱分支刺激 + 派单执行闭环。Memory 薄弱 → Semantic 刺激 →
        // dispatch_and_execute 派单 watcher (Semantic→watcher 静态映射)。
        // 其余分支全部健康, 保证 Semantic 是唯一强刺激 → 派单确定性。
        let mut tree = ConsciousnessTree::new();
        for (kind, branch) in tree.branches.iter_mut() {
            if kind == &BranchKind::Memory {
                branch.health = 0.0; // 薄弱
            } else {
                branch.health = 1.0;
                branch.fog = crate::core::nt_core_consciousness_tree::FogLevel {
                    wired: true,
                    consumer_count: 3,
                    has_tests: true,
                    level: 0.05,
                };
            }
        }
        let mut shell = MetaAgentShell::new("dialogue");
        for (domain, amount) in tree_branch_stimuli(&tree) {
            shell.stimulate(domain, amount);
        }
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("consolidate ran".into()),
        };
        let (agent, _) = shell.dispatch_and_execute(&probe, "general_dialogue_tick").expect("dispatch");
        // Semantic 刺激应让 watcher 成为主导派单 (Memory 分支薄弱被树驱动)。
        assert_eq!(agent, "watcher");
    }

    #[test]
    fn branch_attention_domains_covers_all_kinds() {
        // P2: 7 星系分支均有非空注意力域映射 (控制面完整性)。
        for kind in BranchKind::all() {
            let domains = branch_attention_domains(&kind);
            assert!(!domains.is_empty(), "branch {kind:?} should map to >=1 attention domain");
        }
    }

    #[test]
    fn topology_task_conditioned_init() {
        // P3/MANTA: 任务条件化初始化 — research 任务 PatternMatch→researcher,
        // 一般任务 PatternMatch→explorer。
        let research = DispatchTopology::for_task_type("research_study");
        assert_eq!(research.agent_for(AttentionDomain::PatternMatch), "researcher");
        assert_eq!(research.agent_for(AttentionDomain::SelfReflection), "verifier");
        let general = DispatchTopology::for_task_type("dialogue");
        assert_eq!(general.agent_for(AttentionDomain::PatternMatch), "explorer");
        assert_eq!(general.agent_for(AttentionDomain::Semantic), "watcher");
        assert_eq!(general.agent_for(AttentionDomain::Code), "generalist");
    }

    #[test]
    fn topology_audit_proposes_repair_on_underperformance() {
        // P3/MANTA: trace 审计 — 某域当前档案长期低成功率, 候选档案明显更优 →
        // 提议有界结构修复。
        let mut learner = RouteLearner::new();
        // Semantic 域: watcher (当前, 静态) 0/5 成功; planner 5/5 成功。
        for _ in 0..5 {
            learner.record(AttentionDomain::Semantic, "watcher", false);
        }
        for _ in 0..5 {
            learner.record(AttentionDomain::Semantic, "planner", true);
        }
        let topology = DispatchTopology::for_task_type("dialogue");
        let repairs = topology.audit(&learner);
        assert_eq!(repairs.len(), 1, "one edge should be repaired");
        assert_eq!(repairs[0].domain, AttentionDomain::Semantic);
        assert_eq!(repairs[0].from_agent, "watcher");
        assert_eq!(repairs[0].to_agent, "planner");
        assert!(repairs[0].to_success_rate > repairs[0].from_success_rate);
        // 健康域不产生修复: Code 域 generalist 5/5 成功 → 无替换提议。
        let mut healthy = RouteLearner::new();
        for _ in 0..5 {
            healthy.record(AttentionDomain::Code, "generalist", true);
        }
        let repairs_healthy = topology.audit(&healthy);
        assert!(repairs_healthy.is_empty(), "healthy edge should not be repaired");
    }

    #[test]
    fn topology_audit_with_experience_fuses_coevo_evidence() {
        // P7/D45: 跨域能量流 — coevo 失败警示 + 候选档案掌握度显著更高时提议修复;
        // 证据不足 (reward < min_evidence) 或候选未达 mastery_gate 时不动结构。
        let topology = DispatchTopology::for_task_type("dialogue");
        let learner = RouteLearner::new(); // 空 trace — 传统 audit 不提议任何修复。

        // 证据不足: 无 reward → 仅返回空 (learner 无观测)。
        let empty_coevo = CoEvolutionLoop::new();
        let no_reward = topology.audit_with_experience(&learner, &empty_coevo, "dialogue");
        assert!(no_reward.is_empty(), "insufficient evidence must not repair");

        // 有证据: PatternMatch 当前 explore 失败多次, 候选 researcher mastery 高。
        let mut coevo = CoEvolutionLoop::new();
        for _ in 0..3 {
            coevo.record_reward(
                "dialogue", AttentionDomain::PatternMatch, "explorer", "balanced", false,
                "missed relevant hits",
            );
        }
        for _ in 0..5 {
            coevo.record_reward(
                "dialogue", AttentionDomain::PatternMatch, "researcher", "conservative", true,
                "search returned evidence",
            );
        }
        let repairs = topology.audit_with_experience(&learner, &coevo, "dialogue");
        assert!(!repairs.is_empty(), "coevo failure warning + mastery gap should repair");
        // 修复边界是规范的: 提议的 to_agent 落在规范档案内。
        for r in &repairs {
            assert!(canonical_agent_ok(r.to_agent), "repair target must be canonical");
        }
        // 无当前档案失败的任务类型 → 不应触发经验修复。
        let innocent = topology.audit_with_experience(&learner, &coevo, "research_study");
        for r in &innocent {
            assert_ne!(r.domain, AttentionDomain::PatternMatch,
                "no failure warning for task means no experience-driven repair on its edge");
        }
    }

    /// 测试辅助: 校验规范档案 (对私有 canonical_agent 的只读镜像)。
    fn canonical_agent_ok(s: &str) -> bool {
        matches!(s, "researcher" | "explorer" | "planner" | "generalist" | "verifier" | "watcher")
    }

    #[test]
    fn topology_apply_repair_is_bounded() {
        // P3/MANTA: 有界修复 — 只接受规范档案名, 保持 agent 预算; 未知档案拒绝。
        let mut topology = DispatchTopology::for_task_type("dialogue");
        let repair = TopologyRepair {
            domain: AttentionDomain::Semantic,
            from_agent: "watcher",
            to_agent: "planner",
            from_success_rate: 0.0,
            to_success_rate: 1.0,
            evidence_attempts: 5,
        };
        assert!(topology.apply_repair(&repair));
        assert_eq!(topology.agent_for(AttentionDomain::Semantic), "planner");
        assert_eq!(topology.revision, 1);
        assert!(topology.last_repair.is_some());
        // 未知档案 (越界, 注入防护) → 拒绝。
        let evil = TopologyRepair {
            domain: AttentionDomain::Semantic,
            from_agent: "planner",
            to_agent: "root_backdoor",
            from_success_rate: 0.0,
            to_success_rate: 1.0,
            evidence_attempts: 5,
        };
        assert!(!topology.apply_repair(&evil));
        assert_eq!(topology.revision, 1, "rejected repair must not bump revision");
    }

    #[test]
    fn topology_persist_load_round_trip() {
        // P3/MANTA: 跨轮 playbook — 修复后的拓扑落盘 KB, 新实例可恢复。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_topology_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut topology = DispatchTopology::for_task_type("dialogue");
        topology.apply_repair(&TopologyRepair {
            domain: AttentionDomain::Semantic,
            from_agent: "watcher",
            to_agent: "planner",
            from_success_rate: 0.0,
            to_success_rate: 1.0,
            evidence_attempts: 5,
        });
        topology.persist(&kb).expect("persist");
        // 全新实例恢复 → 边与修订号还原。
        let mut restored = DispatchTopology::for_task_type("research_study");
        assert_eq!(restored.agent_for(AttentionDomain::PatternMatch), "researcher");
        restored.load(&kb).expect("load");
        assert_eq!(restored.agent_for(AttentionDomain::Semantic), "planner", "repair survives restart");
        assert_eq!(restored.revision, 1);
    }

    #[test]
    fn shell_audit_repair_uses_real_learner_trace() {
        // P3 端到端: 派单执行 → learner 积累 trace → audit_and_repair_topology
        // 自动修复拓扑边。模拟 Semantic 域 watcher 连续失败。
        let mut shell = MetaAgentShell::new("dialogue");
        let fail_probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Failure("consolidate unavailable".into()),
        };
        let _success_probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("search returned".into()),
        };
        // 5 次 Semantic 刺激 → watcher 连续失败。
        for _ in 0..5 {
            shell.stimulate(AttentionDomain::Semantic, 0.9);
            shell.dispatch_and_execute(&fail_probe, "monitor health");
        }
        // 再让 explorer/planner 在 Semantic 上成功 (模拟其他域派单误入 Semantic 的修正路径)。
        shell.learner.record(AttentionDomain::Semantic, "planner", true);
        shell.learner.record(AttentionDomain::Semantic, "planner", true);
        shell.learner.record(AttentionDomain::Semantic, "planner", true);
        shell.learner.record(AttentionDomain::Semantic, "planner", true);
        shell.learner.record(AttentionDomain::Semantic, "planner", true);
        let repairs = shell.audit_and_repair_topology();
        assert!(
            repairs.iter().any(|r| r.domain == AttentionDomain::Semantic),
            "Semantic edge should be repaired after underperformance"
        );
        assert_eq!(shell.topology.agent_for(AttentionDomain::Semantic), "planner");
        assert!(shell.topology.revision >= 1);
    }

    #[test]
    fn dispatch_and_execute_feeds_learner_with_real_result() {
        // 派单不再只 eprintln — 执行结果真实喂回 RouteLearner。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("probe did work".into()),
        };
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let (agent, outcome) = shell.dispatch_and_execute(&probe, "review the codebase").expect("dispatch");
        // SelfReflection → verifier 档案被激活, 且探针确实被调用。
        assert_eq!(agent, "verifier");
        assert!(outcome.is_success());
        assert_eq!(shell.last_dispatched, Some("verifier"));
        // 执行器被调用了一次, 参数正确。
        let calls = probe.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "verifier");
        // learner 已记录: verifier 在 SelfReflection 域上的一次成功。
        assert_eq!(shell.learner.rates(AttentionDomain::SelfReflection).len(), 1);
    }

    #[test]
    fn dispatch_and_execute_failure_does_not_reinforce() {
        // 执行失败 → 不强化该档案 (行为信号防自欺)。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Failure("backend down".into()),
        };
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let (agent, outcome) = shell.dispatch_and_execute(&probe, "audit").expect("dispatch");
        assert_eq!(agent, "verifier");
        assert!(!outcome.is_success());
        // 失败记录仍入 learner (attempts+1), 但成功计数不增。
        let rates = shell.learner.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].2, 1, "one attempt recorded");
        assert_eq!(rates[0].1, 0.0, "zero success rate");
    }

    #[test]
    fn dispatch_drives_coevo_same_reward_stream() {
        // P4/MAGE: 同一次派单执行结果构成单一 reward 流 — 技能级路由 bandit (learner)
        // 与四子图共进化循环 (coevo) 被同一 reward 同时更新。
        let mut shell = MetaAgentShell::with_coevo_config("dialogue", CoEvoConfig {
            epsilon: 0.0,
            max_memories: 50,
            min_evidence: 2,
            mastery_gate: 0.5,
        });
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("probe did work".into()),
        };
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let (_agent, outcome) = shell.dispatch_and_execute(&probe, "review the codebase").expect("dispatch");
        assert!(outcome.is_success());
        // 技能级 bandit: learner 记录了 verifier@SelfReflection 一次成功。
        assert_eq!(shell.learner.rates(AttentionDomain::SelfReflection).len(), 1);
        // 共进化循环: 同一 reward 更新四子图 + 任务级搜索 bandit。
        assert_eq!(shell.coevo.total_rewards, 1);
        assert_eq!(shell.coevo.evolution_revision, 1);
        assert_eq!(shell.coevo.graph.memories.len(), 1);
        assert!(shell.coevo.graph.memories[0].is_success());
        // capability 子图: verifier 对 SelfReflection 掌握度 1.0。
        assert_eq!(shell.coevo.mastery(AttentionDomain::SelfReflection, "verifier"), 1.0);
        // experience 子图: 双记忆成功索引可检索到该记忆。
        let g: Vec<_> = shell.coevo.guidance("dialogue", 5);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].agent, "verifier");
        // task 子图 bandit: dialogue 任务已累积 bandit 观测。
        assert_eq!(shell.coevo.bandit().stats("dialogue").len(), 1);
    }

    #[test]
    fn coevo_persist_load_round_trip() {
        // P4/MAGE: 四子图 + 任务级搜索 bandit 落盘 KB, 新实例恢复后继续共进化。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_coevo_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut loop_ = CoEvolutionLoop::with_config(CoEvoConfig {
            epsilon: 0.0,
            max_memories: 50,
            min_evidence: 2,
            mastery_gate: 0.5,
        });
        loop_.record_reward(
            "research",
            AttentionDomain::PatternMatch,
            "explorer",
            "exploratory",
            true,
            "deep dive found hits",
        );
        loop_.record_reward(
            "research",
            AttentionDomain::PatternMatch,
            "explorer",
            "exploratory",
            false,
            "second dive empty",
        );
        assert_eq!(loop_.evolution_revision, 2);
        loop_.persist(&kb).expect("persist");
        // 全新实例恢复 → 图谱、bandit 观测与修订号还原。
        let mut restored = CoEvolutionLoop::new();
        restored.load(&kb).expect("load");
        assert_eq!(restored.evolution_revision, 2);
        assert_eq!(restored.total_rewards, 2);
        assert_eq!(restored.graph.memories.len(), 2);
        assert_eq!(restored.mastery(AttentionDomain::PatternMatch, "explorer"), 0.5);
        let stats = restored.bandit().stats("research");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].strategy, "exploratory");
        assert_eq!(stats[0].attempts, 2);
        // append-only 语义在恢复后继续: 新 reward 递增 id, 不重写历史。
        restored.record_reward(
            "research",
            AttentionDomain::PatternMatch,
            "explorer",
            "exploratory",
            true,
            "third dive",
        );
        assert_eq!(restored.graph.memories.len(), 3);
        assert_eq!(restored.graph.memories[2].id, 2, "monotonic id survives restart");
    }

    #[test]
    fn production_executor_strategy_aware_explorer() {
        // P4/MAGE: 任务级搜索 bandit 的策略经 execute_with_strategy 注入 confidence 检索缝。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_prod_strategy_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        // 预置一条知识节点, 让 confidence 检索有可命中的内容。
        kb.write_memory_entry(
            "neotrix knowledge node",
            NodeType::Concept,
            Some("co-evolution knowledge graph marker"),
            None,
            Some("test"),
            None,
        )
        .expect("write");
        let executor = crate::neotrix::nt_mind::ProductionAgentExecutor::new(kb);
        // balanced → 常规执行 (默认路径, 不改变行为)。
        let outcome = executor.execute_with_strategy("explorer", "knowledge", "balanced");
        assert!(outcome.is_success(), "default path still works: {}", outcome.summary());
        // exploratory → 走 confidence 检索缝, 策略名出现在结果里。
        let strategic = executor.execute_with_strategy("explorer", "knowledge", "exploratory");
        assert!(
            strategic.summary().contains("strategy=exploratory"),
            "strategy should reach confidence seam: {}",
            strategic.summary()
        );
    }

    #[test]
    fn dispatch_research_task_routes_to_researcher() {
        // P0 缺陷 #4 修复: PatternMatch 域 + research 文本 → researcher (网络研究),
        // 而非一律 explorer (KB 检索) — 融合关键词路由与注意力静态映射。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("research done".into()),
        };
        // 刺激 PatternMatch (检索域), 目标是研究任务。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, outcome) = shell
            .dispatch_and_execute(&probe, "research the latest papers on agents")
            .expect("dispatch");
        assert_eq!(agent, "researcher", "research text should route to researcher");
        assert!(outcome.is_success());
        // 纯探索文本 → explorer (KB 检索)。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, _) = shell
            .dispatch_and_execute(&probe, "explore the codebase structure")
            .expect("dispatch");
        assert_eq!(agent, "explorer", "explore text should route to explorer");
        // 越界关键词 (非 PatternMatch 语义) → 退回注意力静态映射。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, _) = shell
            .dispatch_and_execute(&probe, "implement a refactor for core")
            .expect("dispatch");
        assert_eq!(agent, "explorer", "code text outside PatternMatch semantics falls back to explorer");
    }

    #[test]
    fn production_executor_maps_profiles_to_real_actions() {
        // P0 断点 1/2/8 修复: 档案名映射到真实子系统动作。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_executor_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let exec = ProductionAgentExecutor::new(kb);
        // watcher → KB 规模监控 (确定性, 无网络依赖)。
        match exec.execute("watcher", "health probe") {
            AgentExecutionOutcome::Success(s) => assert!(s.contains("KB nodes"), "watcher should probe KB: {}", s),
            other => panic!("watcher should succeed offline: {:?}", other),
        }
        // verifier → 证据溯源计数 (确定性)。
        match exec.execute("verifier", "evidence audit") {
            AgentExecutionOutcome::Success(s) => assert!(s.contains("evidence"), "verifier should trace evidence: {}", s),
            other => panic!("verifier should succeed offline: {:?}", other),
        }
        // planner → 无副作用规划占位 (规划由 metacog cycle 产出)。
        match exec.execute("planner", "design plan") {
            AgentExecutionOutcome::NoOp(s) => assert!(s.contains("planning"), "planner should be NoOp: {}", s),
            other => panic!("planner should be NoOp: {:?}", other),
        }
        // 未知档案 → NoOp (不崩溃, 可审计)。
        match exec.execute("ghost", "whatever") {
            AgentExecutionOutcome::NoOp(s) => assert!(s.contains("ghost"), "unknown agent should NoOp: {}", s),
            other => panic!("unknown agent should NoOp: {:?}", other),
        }
    }

    #[test]
    fn execution_outcome_summary_human_readable() {
        assert!(AgentExecutionOutcome::Success("done".into()).summary().contains("success"));
        assert!(AgentExecutionOutcome::NoOp("none".into()).summary().contains("noop"));
        assert!(AgentExecutionOutcome::Failure("boom".into()).summary().contains("failed"));
        assert!(AgentExecutionOutcome::Success("x".into()).is_success());
        assert!(!AgentExecutionOutcome::Failure("x".into()).is_success());
    }
