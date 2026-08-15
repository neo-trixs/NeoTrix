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
}

/// 轻量 SelfTest 运行器 (纯内存检测件, 无网络/无全仓扫描) — 供意识核心
/// tick 前注入分支健康数据 (迷雾治理断链修复)。复用吸收模块注册表,
/// 每个注册项运行 self_test() 并收集 SelfTestResult。
pub fn run_lightweight_self_tests() -> Vec<crate::core::nt_core_self_test::SelfTestResult> {
    let mut registry = SelfTestRegistry::new();
    register_absorbed_modules(&mut registry);
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
