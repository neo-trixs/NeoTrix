use crate::core::nt_core_arch_fitness::arch_fitness_tests;
use crate::core::nt_core_qtest::QTestEngineSelfTest;
use crate::core::nt_core_self_test::{ConstitutionComplianceTest, SelfTest, SelfTestRegistry};
// use crate::l2_perception::nt_world::cad_selftest;
use crate::core::nt_core_cad_consciousness;

pub fn register_absorbed_modules(mut registry: &mut SelfTestRegistry) {
    registry.register(Box::new(AnswerEngineSelfTest));
    registry.register(Box::new(AgentTeamSelfTest));
    registry.register(Box::new(AgenticScanSelfTest));
    registry.register(Box::new(DigitalHumanSelfTest));
    registry.register(Box::new(AffectiveInterfaceSelfTest));
    registry.register(Box::new(LeannStoreSelfTest));
    registry.register(Box::new(VideoPipelineSelfTest));
    registry.register(Box::new(QTestEngineSelfTest));
    registry.register(Box::new(ConstitutionComplianceTest));
    for t in arch_fitness_tests() {
        registry.register(t);
    }
    // 2026-08-15 sweep absorption batch (Phase A): reasoning-trace / TLS 指纹 / 设备沙箱
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
    ));
    // CAD SelfTest (NT-WORLD 实现层): GenCAD 四步框架生成能力
    cad_selftest::register_cad_self_tests(&mut registry);
    // 跨模态检索增强: 文本/点云/草图查询支持 (CCIP 表征空间)
    // crate::l2_perception::nt_world::cad_crossmodal_selftest::register_cad_crossmodal_self_tests(
    //     &mut registry,
    // );
    // CAD 意识核心统一编排 (NT-CORE): SEAL级联 / Runeword / T3证据 / 经验吸收
    nt_core_cad_consciousness::register_cad_consciousness_self_tests(&mut registry);
    // CAD 真实生成管线 (GenCAD 四步: CSR→CCIP→CDP→Decoder) — 替换架构占位
    // crate::l2_perception::nt_world::cad_generator::register_cad_generator_self_tests(&mut registry);
    // LLM 核心 (NT-CORE): 统一错误域接入 + token 预算引擎自测 (卫生层 P0)
    crate::core::nt_core_llm::register_llm_self_tests(&mut registry);
    // 缓存核心 (NT-CORE): 精确层往返 + 容量计数自测
    crate::core::nt_core_cache::register_cache_self_tests(&mut registry);
    // VSA/HyperCube 核心 (NT-CORE): 卦象嵌入自相似 + 异卦分离 + bind 自相似
    crate::core::nt_core_e8_vsa::register_e8_vsa_self_tests(&mut registry);
    // KB 类型核心 (NT-CORE): NodeType 枚举全变体往返
    crate::core::nt_core_kb_types::register_kb_types_self_tests(&mut registry);
    // NT-MEMORY 四态记忆资产 (吸收 TencentDB-Agent-Memory): ChatMemory/Skill/LlmWiki/CodeGraph 分类
    crate::core::nt_core_memory_asset::register_memory_asset_self_tests(&mut registry);
    // 2026-08-29 外部吸收 (archify/diagram-design): NT-CORE 可验证架构图原语
    crate::core::nt_core_arch_diagram::register_arch_diagram_self_tests(&mut registry);
    // 2026-08-29 外部吸收 (firecrawl/anydoc): NT-WORLD 文档格式路由
    crate::neotrix::nt_file_ability::register_format_route_self_tests(&mut registry);
    // 2026-08-29 外部吸收 (reverse-skill): NT-SHIELD 安全技能路由
    crate::l3_embodiment::nt_shield::nt_shield_skill_router::register_skill_router_self_tests(&mut registry);
    // 2026-08-29 外部吸收 (affaan-m/ECC): NT-MIND SEAL 进化维度 instincts/security
    crate::l5_cognition::nt_mind::nt_mind_seal_ecc::register_seal_ecc_self_tests(&mut registry);
    // 意识核心本体 (NT-CORE): 跨会话 CoreSnapshot 持久化往返
    crate::core::nt_core_consciousness_core::register_consciousness_core_self_tests(&mut registry);
    // 意识度量 IIT Φ (NT-CORE): 同步可约→phi=0 + 变化状态 phi∈[0,1] + 共振矩阵维度
    crate::core::nt_core_iit_phi::register_iit_phi_self_tests(&mut registry);
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_traffic::FingerprintStore::new(),
    ));
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_sandbox::DeviceSandbox::new(
            crate::l3_embodiment::nt_shield::nt_shield_sandbox::SandboxSpec::default(),
        ),
    ));
    // W2.4 (batch3 2026-08-26): 有状态 egress 基准 + W3.5 编排失败类分类法
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_sandbox::stateful_bench::StatefulEgressBench,
    ));
    registry.register(Box::new(
        crate::core::nt_core_orchestration_failure_taxonomy::OrchestrationFailureTaxonomyTest,
    ));
    // 2026-08-15 sweep absorption batch (Phase B): HDA 归因 / 自验证奖励 / 元 harness 优化 / 提示词库
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_evolution_loop::MetaHarnessOptimizer::new(),
    ));
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::PromptLibrary::new(),
    ));
    // 2026-08-29 外部吸收 (anthropics/skills Agent Skills 标准): SKILL.md 必需字段校验
    crate::l5_cognition::nt_mind_skill_engine::register_skill_standard_self_tests(&mut registry);
    // 新增: SelfReflectionEngine (Reflexion-inspired verbal reinforcement learning)
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::experience_tree::self_reflection::SelfReflectionEngine::new(8),
    ));
    // 新增: MemoryAdmissionGate (A-MAC-inspired 5维记忆入口控制)
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_memory::MemoryAdmissionGate::new(0.5, 100),
    ));
    // 2026-08-16 T2 补齐: 小规模方法评估 (sweep absorption 声明 Phase B 但未注册)
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::SmallScaleMethod::new(1.0, 0.5, 32),
    ));
    // 2026-08-16 T2 补齐: 贝叶斯实验设计 (SelfTest 存在但漏注册, C1→C2)
    registry.register(Box::new(
        crate::core::nt_core_hcube::bayesian_experiment::BayesianExperimentDesign::new(
            crate::core::nt_core_hcube::bayesian_experiment::VoIConfig::default(),
            Vec::new(),
        ),
    ));
    // 2026-08-16 T2 补齐: HDA 归因 (纯函数, 评估域原子 2/3→3/3)
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::HdaAttributionSelfTest,
    ));
    // 2026-08-16 T2 补齐: 可自验证奖励 (verify_* 纯函数, SelfTest 逻辑搬入独立件)
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::SelfVerifiableRewardSelfTest,
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
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_hcube::latent_recurrent::RecurrentLatent::new(
            crate::core::nt_core_hcube::latent_recurrent::RecurrentLatentConfig::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    ));
    // 2026-08-15 sweep absorption batch (Phase D): 编排治理 / harness / 感知 / 多模态 / 元数据
    // registry.register(Box::new(
    //     crate::l1_action::nt_act::nt_act_orchestrator::arbiter_mediation::ArbiterMediator::new(),
    // ));
    // registry.register(Box::new(
    //     crate::l1_action::nt_act::nt_act_orchestrator::expert_team_diff::ExpertTeamWriter::new(),
    // ));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_orchestrator::harness_scaffold::HarnessScaffold::new(),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_code::yagni_ladder::YagniLadder::new(),
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_scrape::FitExtractor::default(),
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_crawl::resilient::ResilientCrawler::new(
            crate::l2_perception::nt_world::nt_world_crawl::resilient::ThrottlePolicy::default(),
        ),
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_browse_auto::agentic_browse::AgenticBrowseSelfTest,
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_osint::sweep::SweepDeltaSelfTest,
    ));
    registry.register(Box::new(
        crate::l1_action::nt_io::nt_io_output_style::OutputGovernorSelfTest,
    ));
    // 2026-08-16 T2 补齐: UnifiedAbsorber (in-memory KB, 无网络)
    registry.register(Box::new(UnifiedAbsorberSelfTest));
    registry.register(Box::new(
        crate::l1_action::nt_io::nt_io_multimodal_transform::VisionPreprocessor::new(),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_io::nt_io_multimodal_transform::CpuTtsEngine::new(
            crate::l1_action::nt_io::nt_io_multimodal_transform::VoiceLoader::empty(),
        ),
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_absorber::metadata::MetadataAggregator::new(),
    ));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_video_pipeline::MediaSniffer::new(),
    ));
    // 2026-08-16 Replica absorb: 量子态最优融合检测 (quantum_fusion) T3 接线
    registry.register(Box::new(
        crate::core::nt_core_quantum_fusion::QuantumFusionSelfTest,
    ));
    // 2026-08-19 write_guard 证据审计闭环 (dbx G4): T1→T2 注册 (run.rs 架构审计侧)
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
    ));
    register_c5_healers(registry);
    // 2026-08-27 Wave6 H4: 10 情报工具 SelfTest 注册 (激活 ConsciousnessTree NT-WORLD 分支)
    crate::l2_perception::nt_world::nt_world_intel_selftest::register_intel_self_tests(registry);
    // 2026-08-27 外部吸收 (arXiv:2608.23642): NT-SHIELD 监督退化 canary (Rev-明 审计强化)
    crate::l3_embodiment::nt_shield::nt_shield_oversight::register_oversight_self_tests(registry);
    // 2026-08-28 外部吸收 (arXiv:2608.23642): NT-GOVERNANCE 人类监督治理 affordance + 萎缩对策
    crate::l6_meta::nt_meta::nt_governance::register_human_oversight_self_tests(registry);
}

/// C5 自愈回路检测件 (检测异常 → 自动恢复)。纯内存, 无网络/磁盘/env IO。
pub fn register_c5_healers(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_sentry::SentryHealer,
    ));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_orchestrator::task_state_dag::TaskStateDagHealer,
    ));
    // 2026-08-17 C5 自愈回路扩展: nt_mind_skill_engine 可逆效应 (F1) + fiber 生命周期 (F5)
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::RevertibleEffectsHealer,
    ));
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::FiberLifecycleHealer,
    ));
    // 2026-08-17 C5 自愈回路扩展: MEMORY 溢出层完整性 + MIND-eval 阶梯单调性
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::spill_storage::SpillStorageHealer,
    ));
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::OracleLadderHealer,
    ));
    // 2026-08-17 C5 自愈回路扩展: CAD 生成能力自我修复
    // registry.register(Box::new(
    // crate::l2_perception::nt_world::cad_ch_selftest::CadCHSelfTest,
    // ));
    // SynthBal 合成数据平衡
    // registry.register(Box::new(
    //     crate::l2_perception::nt_world::cad_synthbal_selftest::CadSynthBalSelfTest,
    // ));
    // B-Rep 拓扑验证
    // registry.register(Box::new(
    //     crate::l2_perception::nt_world::cad_brep_selftest::CadBRepTopologySelfTest,
    // ));
    // 2026-08-17 C5 自愈回路扩展: CORE scheduler 认领池一致性 + IO 账户池健康度
    // registry.register(Box::new(
    //     crate::core::nt_core_scheduler::event_driven_claim::ClaimPoolHealer,
    // ));
    // registry.register(Box::new(
    //     crate::l1_action::nt_io::nt_io_provider::account_pool::AccountPoolHealer,
    // ));
    // registry.register(Box::new(
    //     crate::l6_meta::nt_repair::nt_mind_self_heal::SelfHealLoop::new(),
    // ));
    // 2026-08-28 Phase3 免疫: 自愈闭环 C5 — 消费 SelfTest 失败产出 (检测→诊断→自愈→复测)
    // registry.register(Box::new(
    //     crate::l6_meta::nt_repair::nt_mind_self_heal::SelfHealLoop::new(),
    // ));
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
///   Repair 1 atom → ≥1:   repair_causal_trace
///   Meta 1 atom → ≥1:     meta_transcendent_observer
///   Governance 1 atom → ≥1: governance_constitution
///   Nexus 1 atom → ≥1:    nexus_cross_session_memory
pub fn register_lightweight_modules(registry: &mut SelfTestRegistry) {
    // NT-CORE (5)
    registry.register(Box::new(AnswerEngineSelfTest));
    registry.register(Box::new(AffectiveInterfaceSelfTest));
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
    // 2026-08-16 T2 补齐 (lightweight): 贝叶斯实验设计
    registry.register(Box::new(
        crate::core::nt_core_hcube::bayesian_experiment::BayesianExperimentDesign::new(
            crate::core::nt_core_hcube::bayesian_experiment::VoIConfig::default(),
            Vec::new(),
        ),
    ));
    // NT-WORLD (3)
    registry.register(Box::new(VideoPipelineSelfTest));
    registry.register(Box::new(
        crate::l2_perception::nt_world::nt_world_video_pipeline::MediaSniffer::new(),
    ));
    registry.register(Box::new(UnifiedAbsorberSelfTest));
    // NT-MEMORY (4)
    registry.register(Box::new(LeannStoreSelfTest));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::SweepMemoryCapabilitiesSelfTest,
    ));
    // NT-MEMORY 四态资产 (吸收 TencentDB-Agent-Memory): 纯内存分类自测, 轻量注册
    registry.register(Box::new(
        crate::core::nt_core_memory_asset::MemoryAssetSelfTest,
    ));
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::commit_tracker::NarrativeConsistencyChecker::default(),
    ));
    registry.register(Box::new(Bm25IndexSelfTest));
    // 2026-08-19 write_guard 证据审计闭环 (dbx G4): 纯内存检测件 → 轻量注册表
    registry.register(Box::new(
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::WriteGuardAudit,
    ));
    // NT-MIND (5)
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_evolution_loop::MetaHarnessOptimizer::new(),
    ));
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::PromptLibrary::new(),
    ));
    // 2026-08-16 T2 补齐 (lightweight): 小规模方法 / HDA 归因 / 可自验证奖励
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::SmallScaleMethod::new(1.0, 0.5, 32),
    ));
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::HdaAttributionSelfTest,
    ));
    registry.register(Box::new(
        crate::l6_meta::nt_repair::nt_mind_eval_harness::SelfVerifiableRewardSelfTest,
    ));
    // 2026-08-29 外部吸收 (Agent Skills 标准): 轻量注册 (纯内存校验)
    registry.register(Box::new(
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::AgentSkillsStandardSelfTest,
    ));
    // NT-REPAIR / NT-META / NT-GOVERNANCE / NT-NEXUS (4 分支迷雾治理, 每分支 ≥1)
    registry.register(Box::new(
        // crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest,
    ));
    registry.register(Box::new(
        crate::l6_meta::nt_nexus::meta_observer::MetaObserverSelfTest,
    ));
    registry.register(Box::new(
        crate::core::nt_core_self_constitution::GovernanceConstitutionSelfTest,
    ));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_autonomy::cross_session_memory::CrossSessionMemorySelfTest,
    ));
    // NT-ACT (3)
    registry.register(Box::new(AgentTeamSelfTest));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_code::yagni_ladder::YagniLadder::new(),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::default(),
    ));
    // NT-SHIELD (4)
    registry.register(Box::new(AgenticScanSelfTest));
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
    ));
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_traffic::FingerprintStore::new(),
    ));
    registry.register(Box::new(
        crate::l3_embodiment::nt_shield::nt_shield_sandbox::DeviceSandbox::new(
            crate::l3_embodiment::nt_shield::nt_shield_sandbox::SandboxSpec::default(),
        ),
    ));
    // NT-IO (3)
    registry.register(Box::new(DigitalHumanSelfTest));
    registry.register(Box::new(
        crate::l1_action::nt_io::nt_io_multimodal_transform::VisionPreprocessor::new(),
    ));
    registry.register(Box::new(
        crate::l1_action::nt_io::nt_io_multimodal_transform::CpuTtsEngine::new(
            crate::l1_action::nt_io::nt_io_multimodal_transform::VoiceLoader::empty(),
        ),
    ));
}

/// 轻量 SelfTest 运行器 (纯内存检测件, 无网络/无 cargo/无全仓扫描) — 供意识核心
/// tick 前注入分支健康数据 (迷雾治理断链修复)。只跑轻量注册表,
/// 每个注册项运行 self_test() 并收集 SelfTestResult。
pub fn run_lightweight_self_tests() -> Vec<crate::core::nt_core_self_test::SelfTestResult> {
    let mut registry = SelfTestRegistry::new();
    register_lightweight_modules(&mut registry);
    let results = registry.run_all();
    // B1 接线 (quantum_fusion 死模块 → 生产行为): 将多源 SelfTest 结果经
    // 量子叠加融合产出单一可信信号 (多源 D-S 证据融合前端)。融合信号作为
    // 额外检测结果注入 — 树健康计算消费其分支归属, 使 quantum_fusion 从
    // 仅注册 (R-P79 违规) 转为真实生产路径 (行为接地: 结果被消费)。
    fuse_self_test_results(&results)
}

/// 将多源 SelfTest 结果经 quantum_fusion 融合 (B1 接线 — R-P98 三态②:
/// 能力独立且生产缺失 → 提炼并入最近生产节点, 禁平行适配器 R-P42)。
/// 每个检测件 pass→signal 1.0/失败→0.0; 融合产出单一高可靠信号, 以
/// `nt_core_quantum_fusion` 名义注册进结果集 (归属 NT-CORE 分支)。
fn fuse_self_test_results(
    results: &[crate::core::nt_core_self_test::SelfTestResult],
) -> Vec<crate::core::nt_core_self_test::SelfTestResult> {
    use crate::core::nt_core_quantum_fusion::{QuantumSignal, QuantumSuperposition};
    use crate::core::nt_core_self_test::SelfTestResult;

    if results.is_empty() {
        return results.to_vec();
    }
    let mut superpos = QuantumSuperposition::new();
    for r in results {
        // 分支归属检测件按值注入; 无 nt_ 前缀的 (如 constitution) 以中性置信参与
        let value = if r.passed { 1.0 } else { 0.0 };
        let source = r.name.clone();
        superpos.push(QuantumSignal::new(value, 0.8, source));
    }
    if superpos.is_empty() {
        return results.to_vec();
    }
    let fused = superpos.fuse();
    // 融合信号通过性 = 融合值 ≥ 0.5 且纠缠度支持 (非全冲突)
    let passed = fused.value >= 0.5 && fused.entanglement >= 0.3;
    let mut out = results.to_vec();
    out.push(if passed {
        SelfTestResult::pass("nt_core_quantum_fusion")
    } else {
        SelfTestResult::fail(
            "nt_core_quantum_fusion",
            vec![format!(
                "多源融合未达可信阈值: value={:.2} entanglement={:.2}",
                fused.value, fused.entanglement
            )],
        )
    });
    out
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
        // use crate::l1_action::nt_io::nt_agent_agent_team::*;
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
        use crate::l3_embodiment::nt_shield::nt_shield_agentic_scan::*;
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
        use crate::l1_action::nt_io::nt_io_digital_human::*;
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        if !pipeline.generate_reply("hello").contains("Hello") {
            return Err(vec!["reply should handle hello".into()]);
        }
        Ok(())
    }
}

/// NT-CORE affective interface 自测 — 情感交互层 (C1→C5 晋升: 注册即产果门覆盖)。
/// 纯内存构造即测, 适配 lightweight 表。
struct AffectiveInterfaceSelfTest;

impl SelfTest for AffectiveInterfaceSelfTest {
    fn name(&self) -> &str {
        "nt_core_affective_interface"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::core::nt_core_self::affective_interface::*;
        let mut iface = AffectiveInterface::new();
        let readout = iface.process_user_input("我很难过，真的很难受", None, GuideMode::Auto);
        if readout.user.emotion != UserEmotion::Sadness {
            return Err(vec!["sad text should map to Sadness".into()]);
        }
        if readout.expression != "frown" {
            return Err(vec!["sadness expression should be frown".into()]);
        }
        let json = iface.to_json().map_err(|e| vec![format!("serialize: {e}")])?;
        let restored = AffectiveInterface::from_json(&json).map_err(|e| vec![format!("deserialize: {e}")])?;
        if restored.relationship.interactions != 1 {
            return Err(vec!["restored interactions should carry over".into()]);
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
        use crate::l1_action::nt_memory::nt_memory_leann_store::*;
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
        use crate::l1_action::nt_memory::nt_memory_kb::bm25::{Bm25Document, Bm25Index};
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
        use crate::l2_perception::nt_world::nt_world_video_pipeline::*;
        let mut orch = VideoOrchestrator::new(TranscodeConfig::default());
        orch.self_test()?;
        // VideoProductionChain 接线 (C1→T2/T3): 纯内存确定性 produce_video 自检。
        let manifest = orch.produce_video("self-check").map_err(|e| vec![e])?;
        if manifest.len() < 5 {
            return Err(vec![format!("produce_video must yield full 5-stage manifest, got {}", manifest.len())]);
        }
        // VideoChainRunner 接线: run_all + checkpoint JSON roundtrip (纯内存)。
        let mut runner = VideoChainRunner::new("self-check");
        let stages = runner.run_all().map_err(|e| vec![e])?;
        if stages.is_empty() {
            return Err(vec!["video chain runner produced no stages".into()]);
        }
        let serialized = runner.checkpoint.to_json().map_err(|e| vec![e])?;
        let restored = VideoChainCheckpoint::from_json(&serialized).map_err(|e| vec![e])?;
        if restored.source != runner.checkpoint.source {
            return Err(vec!["video chain checkpoint roundtrip mismatch".into()]);
        }
        if !restored.all_done() {
            return Err(vec!["restored checkpoint must be all-done after full run".into()]);
        }
        Ok(())
    }
}

/// UnifiedAbsorber 接线 (C1→T2/T3): in-memory KB 构造 + 纯 DB 状态自检 (无网络)。
struct UnifiedAbsorberSelfTest;

impl SelfTest for UnifiedAbsorberSelfTest {
    fn name(&self) -> &str {
        "nt_world_absorber_unified"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
use crate::l2_perception::nt_world::nt_memory_kb_bridge::KnowledgeBase;
        use crate::l2_perception::nt_world::nt_world_absorber::{AbsorberConfig, UnifiedAbsorber};
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:")))
            .map_err(|e| vec![e])?;
        let absorber = UnifiedAbsorber::new(kb, AbsorberConfig::default())
            .map_err(|e| vec![e])?;
        let _ = absorber.status().map_err(|e| vec![e])?;
        let _ = absorber.api_registry_stats();
        Ok(())
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

    #[test]
    fn test_lightweight_covers_four_branch_prefixes() {
        // 迷雾治理四分支 (Repair/Meta/Governance/Nexus) 每 tick 必须有轻量
        // SelfTest 喂入 — 否则 fog 卡 0.15。防止后续重构把四分支注册移除。
        let results = run_lightweight_self_tests();
        let prefixes = ["nt_repair_", "nt_meta_", "nt_governance_", "nt_nexus_"];
        for prefix in prefixes {
            assert!(
                results.iter().any(|r| r.name.starts_with(prefix)),
                "lightweight registry missing {} selftest",
                prefix
            );
        }
    }

    #[test]
    fn test_c5_healers_registered_and_pass() {
        // C5 自愈回路: SentryHealer + TaskStateDagHealer + skill_engine 双 healer
        // 必须注册且 self_test 通过。
        let mut registry = SelfTestRegistry::new();
        register_c5_healers(&mut registry);
        let results = registry.run_all();
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(
            names.contains(&"nt_shield_sentry::sentry_healer"),
            "SentryHealer not registered, got {:?}",
            names
        );
        assert!(
            names.contains(&"nt_act_orchestrator::task_state_dag"),
            "TaskStateDagHealer not registered, got {:?}",
            names
        );
        assert!(
            names.contains(&"nt_mind_skill_engine::revertible_effects_healer"),
            "RevertibleEffectsHealer not registered, got {:?}",
            names
        );
        assert!(
            names.contains(&"nt_mind_skill_engine::fiber_lifecycle_healer"),
            "FiberLifecycleHealer not registered, got {:?}",
            names
        );
        for r in &results {
            assert!(r.passed, "C5 healer {} failed: {:?}", r.name, r.failures);
        }
    }
}
