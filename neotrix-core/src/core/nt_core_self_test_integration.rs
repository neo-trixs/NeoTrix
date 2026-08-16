use crate::core::nt_core_arch_fitness::arch_fitness_tests;
use crate::core::nt_core_qtest::QTestEngineSelfTest;
use crate::core::nt_core_self_test::{ConstitutionComplianceTest, SelfTest, SelfTestRegistry};

pub fn register_absorbed_modules(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(AnswerEngineSelfTest));
    registry.register(Box::new(AgentTeamSelfTest));
    registry.register(Box::new(AgenticScanSelfTest));
    registry.register(Box::new(DigitalHumanSelfTest));
    registry.register(Box::new(LeannStoreSelfTest));
    registry.register(Box::new(VideoPipelineSelfTest));
    registry.register(Box::new(QTestEngineSelfTest));
    registry.register(Box::new(ConstitutionComplianceTest));
    for t in arch_fitness_tests() {
        registry.register(t);
    }
    // 2026-08-15 sweep absorption batch (Phase A): reasoning-trace / TLS 指纹 / 设备沙箱
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_audit::ReasoningTraceGuard::default(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_traffic::FingerprintStore::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_sandbox::DeviceSandbox::new(
            crate::neotrix::l1_body_impl::nt_shield_sandbox::SandboxSpec::default(),
        ),
    ));
    // 2026-08-15 sweep absorption batch (Phase B): HDA 归因 / 自验证奖励 / 元 harness 优化 / 提示词库
    registry.register(Box::new(
        crate::neotrix::l8_autonomic_impl::nt_mind_evolution_loop::MetaHarnessOptimizer::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::PromptLibrary::new(),
    ));
    // 2026-08-15 sweep absorption batch (Phase C): 模式路由 / 潜循环 / 记忆四能力
    registry.register(Box::new(
        crate::core::nt_core_gwt::mode_router::ModeRouter::new(),
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::neotrix::l3_memory_impl::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    // 2026-08-15 sweep absorption batch (Phase D): 编排治理 / harness / 感知 / 多模态 / 元数据
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_agent_orchestrator::arbiter_mediation::ArbiterMediator::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_agent_orchestrator::expert_team_diff::ExpertTeamWriter::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_act_orchestrator::harness_scaffold::HarnessScaffold::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_act_code::yagni_ladder::YagniLadder::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_scrape::FitExtractor::default(),
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_crawl::resilient::ResilientCrawler::new(
            crate::neotrix::l2_world_impl::nt_world_crawl::resilient::ThrottlePolicy::default(),
        ),
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_browse_auto::agentic_browse::AgenticBrowseSelfTest,
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_osint::sweep::SweepDeltaSelfTest,
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_io_multimodal_transform::VisionPreprocessor::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_io_multimodal_transform::CpuTtsEngine::new(
            crate::neotrix::l1_body_impl::nt_io_multimodal_transform::VoiceLoader::empty(),
        ),
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_absorber::metadata::MetadataAggregator::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_video_pipeline::MediaSniffer::new(),
    ));
    // 2026-08-16 Replica absorb: 量子态最优融合检测 (quantum_fusion) T3 接线
    registry.register(Box::new(
        crate::core::nt_core_quantum_fusion::QuantumFusionSelfTest,
    ));
}

/// 轻量 SelfTest 注册表 (纯内存检测件, 无网络/无 cargo check/无全仓扫描) —
/// 供意识核心 tick 前注入分支健康数据 (迷雾治理断链修复 + 果实产出覆盖)。
///
/// 与 `register_absorbed_modules` (全量) 的差异:
/// - 全量含 DeadCodeFitness (cargo check ~30s) / QTestEngine (全仓扫描) /
///   爬虫/OSINT 检测件 (真实 IO 语义) → tick 曾退化至 69s。
/// - 本表只含构造即测的纯内存件, 保证 tick 秒级 (Dark Forest: 轻量路径必须快)。
///
/// 每分支覆盖目标: mandatory atoms 的 ≥50% (产果门 self_test_coverage >= 0.5)。
///   Core 10 atoms → ≥5:   answer_engine / gwt_mode_router / hcube_latent_recurrent / quantum_fusion / constitution
///   World 4 atoms → ≥2:   video_pipeline / media_sniff
///   Memory 7 atoms → ≥4:  leann_store / kb_sweep_capabilities / narrative_checker / bm25_index
///   Mind 3 atoms → ≥2:    meta_harness_optimizer / prompt_library
///   Act 3 atoms → ≥2:     agent_team / yagni_ladder / nt_act_sandbox
///   Shield 5 atoms → ≥3:  agentic_scan / reasoning_trace_guard / tls_fingerprint / device_sandbox
///   Io 4 atoms → ≥2:      digital_human / vision_preprocess / cpu_tts
pub fn register_lightweight_modules(registry: &mut SelfTestRegistry) {
    // NT-CORE (5)
    registry.register(Box::new(AnswerEngineSelfTest));
    registry.register(Box::new(
        crate::core::nt_core_gwt::mode_router::ModeRouter::new(),
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::core::nt_core_quantum_fusion::QuantumFusionSelfTest,
    ));
    registry.register(Box::new(ConstitutionComplianceTest));
    // NT-WORLD (2)
    registry.register(Box::new(VideoPipelineSelfTest));
    registry.register(Box::new(
        crate::neotrix::l2_world_impl::nt_world_video_pipeline::MediaSniffer::new(),
    ));
    // NT-MEMORY (4)
    registry.register(Box::new(LeannStoreSelfTest));
    registry.register(Box::new(
        crate::neotrix::l3_memory_impl::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::default(),
    ));
    registry.register(Box::new(Bm25IndexSelfTest));
    // NT-MIND (2)
    registry.register(Box::new(
        crate::neotrix::l8_autonomic_impl::nt_mind_evolution_loop::MetaHarnessOptimizer::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::PromptLibrary::new(),
    ));
    // NT-ACT (3)
    registry.register(Box::new(AgentTeamSelfTest));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_act_code::yagni_ladder::YagniLadder::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_act_sandbox::ActionSandbox::default(),
    ));
    // NT-SHIELD (4)
    registry.register(Box::new(AgenticScanSelfTest));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_audit::ReasoningTraceGuard::default(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_traffic::FingerprintStore::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_shield_sandbox::DeviceSandbox::new(
            crate::neotrix::l1_body_impl::nt_shield_sandbox::SandboxSpec::default(),
        ),
    ));
    // NT-IO (3)
    registry.register(Box::new(DigitalHumanSelfTest));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_io_multimodal_transform::VisionPreprocessor::new(),
    ));
    registry.register(Box::new(
        crate::neotrix::l1_body_impl::nt_io_multimodal_transform::CpuTtsEngine::new(
            crate::neotrix::l1_body_impl::nt_io_multimodal_transform::VoiceLoader::empty(),
        ),
    ));
}

/// 轻量 SelfTest 运行器 (纯内存检测件, 无网络/无 cargo/无全仓扫描) — 供意识核心
/// tick 前注入分支健康数据 (迷雾治理断链修复)。只跑轻量注册表,
/// 每个注册项运行 self_test() 并收集 SelfTestResult。
pub fn run_lightweight_self_tests() -> Vec<crate::core::nt_core_self_test::SelfTestResult> {
    let mut registry = SelfTestRegistry::new();
    register_lightweight_modules(&mut registry);
    registry.run_all()
}

struct AnswerEngineSelfTest;

impl SelfTest for AnswerEngineSelfTest {
    fn name(&self) -> &str {
        "nt_core_answer_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let engine = crate::core::nt_core_answer_engine::AnswerEngine::with_mode(
            crate::core::nt_core_answer_engine::AnswerMode::Balanced,
        );
        let config = engine.config();
        if config.max_sources == 0 {
            return Err(vec!["config not initialized".into()]);
        }
        Ok(())
    }
}

struct AgentTeamSelfTest;

impl SelfTest for AgentTeamSelfTest {
    fn name(&self) -> &str {
        "nt_act_agent_team"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_agent_agent_team::*;
        let mut team = AgentTeam::new("test-team");
        team.add_member(AgentProfile::new(AgentRole::Lead, "alice"));
        team.add_member(AgentProfile::new(AgentRole::Coder, "bob"));
        if team.member_count() != 2 {
            return Err(vec!["expected 2 members".into()]);
        }
        let t1 = team.create_task("implement feature", AgentRole::Coder, 1);
        let assigned = team.assign_tasks();
        if assigned.is_empty() {
            return Err(vec!["no tasks assigned".into()]);
        }
        team.complete_task(t1);
        if (team.progress() - 1.0).abs() > 0.01 {
            return Err(vec!["progress should be 1.0".into()]);
        }
        Ok(())
    }
}

struct AgenticScanSelfTest;

impl SelfTest for AgenticScanSelfTest {
    fn name(&self) -> &str {
        "nt_shield_agentic_scan"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_shield_agentic_scan::*;
        let scanner = AgenticScanner::new(ScanConfig::default());
        if scanner.current_stage() != ScanStage::Recon {
            return Err(vec!["initial stage should be Recon".into()]);
        }
        let report = scanner.recon_scan("http://test.com");
        if report.estimated_files == 0 {
            return Err(vec!["recon should estimate files".into()]);
        }
        Ok(())
    }
}

struct DigitalHumanSelfTest;

impl SelfTest for DigitalHumanSelfTest {
    fn name(&self) -> &str {
        "nt_io_digital_human"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_io_digital_human::*;
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        if !pipeline.generate_reply("hello").contains("Hello") {
            return Err(vec!["reply should handle hello".into()]);
        }
        Ok(())
    }
}

struct LeannStoreSelfTest;

impl SelfTest for LeannStoreSelfTest {
    fn name(&self) -> &str {
        "nt_memory_leann_store"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l3_memory_impl::nt_memory_leann_store::*;
        let store = LeannGraphStore::new(LeannConfig::default());
        if store.node_count() != 0 {
            return Err(vec!["fresh store should have 0 nodes".into()]);
        }
        Ok(())
    }
}

/// NT-MEMORY 原子能力覆盖 (第 4 件): BM25 索引检索 (纯内存, 接线现有 bm25 模块)。
struct Bm25IndexSelfTest;

impl SelfTest for Bm25IndexSelfTest {
    fn name(&self) -> &str {
        "nt_memory_bm25_index"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::bm25::{Bm25Document, Bm25Index};
        let docs = vec![
            Bm25Document {
                id: "d1".into(),
                text: "rust knowledge graph embedding".into(),
            },
            Bm25Document {
                id: "d2".into(),
                text: "vector symbolic architecture reasoning".into(),
            },
            Bm25Document {
                id: "d3".into(),
                text: "attention routing global workspace".into(),
            },
        ];
        let index = Bm25Index::build(&docs);
        let results = index.search("knowledge", 2);
        if results.is_empty() {
            return Err(vec!["bm25 search returned empty for known term".into()]);
        }
        if results[0].1 != "d1" {
            return Err(vec![format!("expected d1 first, got {:?}", results[0])]);
        }
        Ok(())
    }
}

struct VideoPipelineSelfTest;

impl SelfTest for VideoPipelineSelfTest {
    fn name(&self) -> &str {
        "nt_world_video_pipeline"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l2_world_impl::nt_world_video_pipeline::*;
        let orch = VideoOrchestrator::new(TranscodeConfig::default());
        orch.self_test()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all() {
        let mut registry = SelfTestRegistry::new();
        register_absorbed_modules(&mut registry);
        // 动态断言: 至少包含基础 8 检测件 + 6 arch_fitness 守卫 (≥14)。
        // 不硬编码精确计数 — sweep 吸收批 (2026-08-15 Phase A/B/C) 会持续
        // 增补检测件, 魔法数字会被并行会话反复改破 (拉锯)。
        assert!(
            registry.count() >= 14,
            "注册器应至少含 8 基础 + 6 arch_fitness, got {}",
            registry.count()
        );
    }

    #[test]
    fn test_answer_engine_st() {
        assert!(AnswerEngineSelfTest.self_test().is_ok());
    }

    #[test]
    fn test_all_have_names() {
        // 动态遍历注册表验证全部检测件都有非空名称 — 不依赖精确数组长度,
        // 后台增补检测件时无需同步此测试 (Dark Forest: 防漂移)。
        let mut registry = SelfTestRegistry::new();
        register_absorbed_modules(&mut registry);
        for r in registry.run_all() {
            assert!(!r.name.is_empty());
        }
    }
}
