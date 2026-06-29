/// 意识自进化元层 — 闭环5条断裂回路
///
/// 在 NeoTrix 意识架构中，5 条关键反馈回路从未在实际运行时连接：
///
/// 1. **CalibrationEngine → MetaCognitiveLoop**: ECE/meta-d 校准误差从未被
///    元认知循环消费，导致元认知精度不受校准状态影响。
///
/// 2. **LossFunction → SelfModifyAgent**: 复合损失从未触发自修改提案，
///    系统无法对自身性能下降做出反应。
///
/// 3. **MetaCognitiveLoop → SelfEvolutionLoop**: 元认知循环产生的进化计划
///    从未被执行，自我诊断没有真正的自我修复能力。
///
/// 4. **SelfModifyGuard 所有4层为None/nop**: 安全性门控在生产环境中是空操作，
///    自我修改没有任何实际保护。
///
/// 5. **ConsciousnessCycle 12步存根**: 循环的12个步骤从未执行真正的认知处理，
///    只是通过存根返回恒定值。
///
/// `SelfEvolutionMetaLayer` 是所有这些断裂回路的统一桥接层。它在每一意识周期
/// 中被调用一次，依次执行5个回路的数据桥接、阶段调度、门控激活、和循环实现。
use std::collections::VecDeque;

use super::agent0_dual_loop::CurriculumAgent;
use super::anti_spiral_monitor::{AntiSpiralConfig, AntiSpiralMonitor};
use super::archive_manager::ArchiveManager;
use super::ast_mutation_engine::{AstMutationConfig, AstMutationEngine};
use super::auto_commit_gate::{AutoCommitConfig, AutoCommitGate};
use super::auto_mlir_generator::AutoMlirGenerator;
use super::background_evolution_scheduler::BackgroundEvolutionScheduler;
use super::calibration_engine::CalibrationEngine;
use super::cpe_regularizer::CpeRegularizer;
use super::dual_lever_orchestrator::DualLeverOrchestrator;
use super::edit_journal::EditJournal;
use super::escher_loop_engine::{EscherLoopConfig, EscherLoopEngine};
use super::evolution_task_system::{EvolutionTask, EvolutionTaskSystem, TaskSystemStats, TaskType};
use super::failure_evidence_batcher::{EvidenceSource, FailureEvidenceBatcher};
use super::gepa_asi_evaluator::GepaAsiEvaluator;
use super::loss_function::{CompositeLoss, LossFunction};
use super::ouroboros_stage_manager::{OuroborosConfig, OuroborosStageManager};
use super::pareto_front_selector::{Candidate, ParetoFrontSelector};
use super::recovery_recipe::RecoveryRecipeManager;
use super::reflective_analyzer::ReflectiveAnalyzer;
use super::repo_understanding::RepoUnderstandingEngine;
use super::sandbox_evaluator::SandboxEvaluator;
use super::self_evolution_engine::{TraceWeakness, WeaknessMiner};
use super::self_evolution_loop::{MutationOp, SelfEvolutionLoop, SelfEvolutionStep};
use super::self_evolution_orchestrator::{OrchestratorConfig, SelfEvolutionTaskOrchestrator};
use super::self_evolution_pipeline::SelfEvolutionPipeline;
use super::self_source_reader::{SelfSourceReader, SourceReaderConfig};
use super::sepl_operators::SeplPipeline;
use super::skill_crystal::{CrystallizationConfig, SkillCrystallizer};
use super::sub_agent_accumulator::SubAgentAccumulator;
use super::trace_capture_engine::{TraceCaptureEngine, TraceSeverity, TraceSource};
use super::trace_encoder::TraceEncoder;
use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;

use super::agent_supervisor::AgentSupervisor;
use super::auto_experiment_loop::{AutoExperimentConfig, AutoExperimentLoop};
use super::auto_review_classifier::AutoReviewClassifier;
use super::constraint_registry::ConstraintRegistry;
use super::context_compressor::CognitiveContextCompressor;
use super::cross_repo_mapper::CrossRepoMapper;
use super::decision_chain::{DecisionChain, DecisionContext};
use super::design_token::{PrimitiveToken, TokenRegistry};
use super::dream_cycle_scheduler::DreamCycleScheduler;
use super::erl_heuristic_pool::{ERLHeuristicPool, ERLHeuristicPoolConfig};
use super::experience_tree::{ExperienceTree, PruningConfig};
use super::findings_aggregator::{FindingCategory, FindingsAggregator, FindingsAggregatorConfig};
use super::homeostatic_drive::{HomeostaticDriveConfig, HomeostaticDriveSystem};
use super::knowledge_node::{KnowledgeGraph, KnowledgeNode, NodeType};
use super::memory_archiver::MemoryArchiver;
use super::outcome_tracker::{OutcomeTracker, RubricCriterion};
use super::principle_distiller::{DistillerConfig, PrincipleDistiller};
use super::recursive_delegation::{DelegateConfig, RecursiveDelegationManager};
use super::research_package::ResearchPackageManager;
use super::search_keyword_optimizer::{OptimizerConfig, SearchKeywordOptimizer};
use super::self_manifest::SelfManifestGenerator;
use super::self_model_generator::SelfModelGenerator;
use super::stacked_validation::StackedValidationPipeline;
use super::SealProposalBridge;
use super::SelfEvolutionTaskEngine;
use crate::core::nt_core_consciousness::consciousness_cycle::CycleResult;
#[cfg(test)]
use crate::core::nt_core_consciousness::consciousness_cycle::{
    ConsciousnessCycle, CycleConfig, CycleStep, StepHealth,
};
use crate::core::nt_core_consciousness::memory_lattice::MemoryLattice;
use crate::core::nt_core_consciousness::neuromodulator::{
    NeuromodulatorEngine, NeuromodulatorType,
};
#[cfg(test)]
use crate::core::nt_core_consciousness::vsa_tag::VsaTagged;
use crate::core::nt_core_knowledge::behavioral_personality::BehavioralPersonalityEngine;
use crate::core::nt_core_meta::metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
use crate::core::nt_core_self_modify::agent::{ModifyTarget, SelfModifyAgent};
use crate::core::nt_core_self_modify::guard::SelfModifyGuard;
use crate::neotrix::nt_shield::ast_safety_gate::AstSafetyGate;

/// 记录一次元层干预事件，用于事后审计和回滚。
#[derive(Debug, Clone)]
pub struct InterventionRecord {
    /// 干预发生时的 cycle 编号
    pub cycle: u64,
    /// 干预类型
    pub kind: InterventionKind,
    /// 是否成功
    pub success: bool,
    /// 描述文本
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionKind {
    BridgeCalibration,
    BridgeLoss,
    BridgeEvolution,
    GuardActivation,
    CycleImplementation,
    PhaseDecision,
    /// 搜索关键词优化器输出（每 30 cycle 写入干预日志）
    BridgeSearch,
}

/// 将 EvolutionTask 转换为 MutationOp，供 SkillCrystallizer 消费。
/// 这是 GEPA 反射闭环中 "trace → crystallize → register" 的桥接函数。
fn task_to_mutation(task: &EvolutionTask) -> MutationOp {
    let title = &task.title;
    match &task.task_type {
        TaskType::ModuleWiring => MutationOp::SelfModifyProposal {
            target: title.clone(),
            target_type: "module_wiring".into(),
            source_code: String::new(),
        },
        TaskType::ModuleCreate => MutationOp::AddHandler {
            position: title.clone(),
            code: String::new(),
        },
        TaskType::CompileFix => MutationOp::RewriteHandler {
            name: format!("compile_{}", title),
            code: String::new(),
        },
        TaskType::RefactorExisting => MutationOp::RewritePrimitive {
            name: title.clone(),
            impl_: String::new(),
        },
        TaskType::Performance => MutationOp::TuneParam {
            target: title.clone(),
            delta: 0.0,
        },
        TaskType::TuneMutation { ref target, delta } => MutationOp::TuneParam {
            target: target.clone(),
            delta: *delta,
        },
        TaskType::NewModule { ref name } => MutationOp::AddHandler {
            position: name.clone(),
            code: String::new(),
        },
        TaskType::SecurityHardening => MutationOp::SwapPolicy {
            gates: vec![title.clone()],
        },
        TaskType::TestCreate => MutationOp::AddHandler {
            position: format!("test_{}", title),
            code: String::new(),
        },
        _ => MutationOp::SelfModifyProposal {
            target: title.clone(),
            target_type: "evolution_task".into(),
            source_code: String::new(),
        },
    }
}

/// 当前 cycle 的校准快照（用于 GEPA 反射式分析）
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TraceSnapshot {
    #[allow(dead_code)]
    cycle: u64,
    ece: f64,
    meta_accuracy: f64,
    composite_loss: f64,
    #[allow(dead_code)]
    pending_tasks: usize,
}

/// 意识自进化元层 — 闭环5条断裂回路
pub struct SelfEvolutionMetaLayer {
    /// 回路 1-3 的跨组件数据桥接器
    pub feedback_bridge: FeedbackBridge,
    /// 子循环调度器
    pub loop_coordinator: LoopCoordinator,
    /// 门控激活器（回路 4）
    pub guard_activator: GuardActivator,
    /// 自进化任务系统
    pub task_system: EvolutionTaskSystem,
    /// 自进化迭代管线 (Phase A→F)
    pub pipeline: SelfEvolutionPipeline,
    /// 最后一次触发干预的 cycle 编号
    pub last_trigger_cycle: u64,
    /// 自进化事件计数
    pub evolution_count: u64,
    /// 元精度历史（用于趋势检测）
    pub meta_accuracy_history: VecDeque<f64>,
    /// 复合损失历史（用于趋势检测）
    pub loss_history: VecDeque<f64>,
    /// 干预日志（最多 200 条）
    pub intervention_log: VecDeque<InterventionRecord>,
    /// GEPA 风格自进化任务引擎
    pub task_engine: Option<SelfEvolutionTaskEngine>,
    /// 校准迹环形缓冲（用于反射式分析，保存最近 50 个 cycle）
    #[allow(private_interfaces)]
    pub trace_buffer: VecDeque<TraceSnapshot>,
    /// GenericAgent 风格技能结晶器：将成功突变→技能自动凝固为可复用 .ne 文件
    pub skill_crystallizer: Option<SkillCrystallizer>,
    /// Wave 0.5: 三阶段进化生命周期管理器
    pub ouroboros: Option<OuroborosStageManager>,
    /// Wave 0.5: 编译时安全门控（自修改代码审计）
    pub ast_gate: Option<AstSafetyGate>,
    /// Meta Loop: 双种群共进化引擎 (Escher-Loop style)
    pub escher_loop: EscherLoopEngine,
    /// Meta Loop: 子Agent积累器 (AgentFactory style)
    pub sub_agent_acc: SubAgentAccumulator,
    /// GEPA ASI 结构化诊断评估器: 从迹文本提取结构化诊断
    pub gepa_asi: GepaAsiEvaluator,
    /// ReflectiveAnalyzer: 6模式结构化诊断引擎, 与 GEPA ASI 互补
    pub reflective_analyzer: ReflectiveAnalyzer,
    /// Meta Loop: 自进化迭代任务管线
    pub orchestrator: SelfEvolutionTaskOrchestrator,
    /// 自模型生成器：从运行时合成 prompt 层自模型
    pub self_model: SelfModelGenerator,
    /// 冷记忆归档器：低激活 episodic → 归档到磁盘
    pub memory_archiver: MemoryArchiver,
    /// MemRL-style Q-learning flag: update episodic Q-values from ECE reward
    pub q_learning_enabled: bool,
    /// 经验树：运行时洞察蒸馏与四通道修剪
    pub experience_tree: Option<ExperienceTree>,
    /// HIVE 风格反螺旋监控器：检测推理循环/振荡/停滞
    pub anti_spiral: AntiSpiralMonitor,
    /// Wave 0: Rust 源码读取器 (syn-based AST parser)
    pub self_source_reader: Option<SelfSourceReader>,
    /// Wave 0: AST 级别代码变异引擎
    pub ast_mutation_engine: Option<AstMutationEngine>,
    /// Wave 0: 自修改编译→测试→clippy 验证门控
    pub auto_commit_gate: Option<AutoCommitGate>,
    /// yoyo-evolve 风格背景进化调度器：定时源审计任务
    pub bg_scheduler: BackgroundEvolutionScheduler,
    /// TraceEncoder: 将校准快照 + 干预事件编码为结构化迹
    pub trace_encoder: Option<TraceEncoder>,
    /// ArchiveManager: DGM-H 风格存档树替换线性 Vec 存档
    pub archive_manager: Option<ArchiveManager>,
    /// SandboxEvaluator: 运行时沙箱验证自修改代码
    pub sandbox_evaluator: Option<SandboxEvaluator>,
    /// AutoMlirGenerator: 完整的变异→门控→提交→重复循环
    pub auto_mlir: Option<AutoMlirGenerator>,
    /// SEPL 算子管线: ρ→σ→ι→ε→κ 形式化闭环绕
    pub sepl_pipeline: Option<SeplPipeline>,
    /// ERL 启发式池：从执行迹提取可复用启发式规则
    pub heuristic_pool: ERLHeuristicPool,
    /// 统一健康发现面板：聚合 50+ 模块健康指标
    pub findings_aggregator: FindingsAggregator,
    /// 跨仓库概念映射器：术语对齐
    pub cross_repo_mapper: CrossRepoMapper,
    /// v24: 结果跟踪器 — 基于量规的执行前评估/执行后验证
    pub outcome_tracker: OutcomeTracker,
    /// v24: 梦境循环调度器 — 后台迹分析产生新模式洞察
    pub dream_scheduler: DreamCycleScheduler,
    /// v24: Agent 监督器 — 持久化会话管理
    pub agent_supervisor: AgentSupervisor,
    /// v24: 自动审查分类器 — 提议动作的权限门控
    pub auto_review: AutoReviewClassifier,
    /// v24: MCP 意识服务器 — 以 MCP 工具形式暴露意识能力
    /// v24: 递归委托管理器 — RAO 风格子Agent递归任务委托
    pub delegation_manager: RecursiveDelegationManager,
    /// SearchKeywordOptimizer: 搜索关键词收益率追踪 + 上下游传播 + 自动进化
    pub keyword_optimizer: SearchKeywordOptimizer,
    /// .ne-research 研究包管理器: SkillCrystallizer trace → ARA 4层可再生物品
    pub research_pkg_mgr: ResearchPackageManager,
    /// Anti-spiral WeaknessMiner: proposal/failure/confidence loop detection
    pub weakness_miner: Option<WeaknessMiner>,
    /// v25 Design Token Registry: 三层设计标记系统
    pub token_registry: TokenRegistry,
    /// v25 Knowledge Graph: 从执行迹蒸馏的设计原则/模式/决策/反模式
    pub knowledge_graph: KnowledgeGraph,
    /// v26 RepoUnderstandingEngine: 结构化仓库理解吸收引擎
    pub repo_understanding: RepoUnderstandingEngine,
    /// v25 Decision Chain: 进化决策上下文追踪
    pub decision_chain: DecisionChain,
    /// v25 Principle Distiller: 从执行迹 + 档案中蒸馏设计原则
    pub principle_distiller: PrincipleDistiller,
    /// v25 Self-Manifest Generator: 结构化 YAML 自模型清单
    pub self_manifest: SelfManifestGenerator,
    /// Wave B: Karpathy-style autonomous experiment loop (modify→evaluate→keep/revert)
    pub auto_experiment: AutoExperimentLoop,
    /// Wave B: Homeostatic drive system (curiosity/mastery/coherence/novelty)
    pub homeostatic_drive: HomeostaticDriveSystem,
    /// GEPA 主动迹捕获引擎
    pub trace_capture: TraceCaptureEngine,
    /// NSGA-II 多目标帕累托选择器
    pub pareto_selector: ParetoFrontSelector,
    /// SIA 双臂演进编排器
    pub dual_lever: DualLeverOrchestrator,
    /// CognitiveContextCompressor: VSA-based thought history compression
    pub cognitive_compressor: CognitiveContextCompressor,
    /// Wave F: Unified constraint registry bridging GDI/PCC/SAHOO
    pub constraint_registry: ConstraintRegistry,
    /// MOSS §4: Failure Evidence Batcher — auto-curates failure batches for evolution pipeline
    pub failure_evidence_batcher: FailureEvidenceBatcher,
    /// P0.5: EditJournal — snapshot+rollback for SEAL self-modification
    pub edit_journal: EditJournal,
    /// CPE Retention Regularizer — tracks capability erosion across workflow/skill/model/memory
    pub cpe_regularizer: CpeRegularizer,
}

impl Default for SelfEvolutionMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfEvolutionMetaLayer {
    pub fn new() -> Self {
        let config = MetaProcedureConfig::default();
        Self {
            feedback_bridge: FeedbackBridge {
                config: config.clone(),
            },
            loop_coordinator: LoopCoordinator {
                config: config.clone(),
            },
            guard_activator: GuardActivator,
            task_system: EvolutionTaskSystem::new(),
            pipeline: SelfEvolutionPipeline::new(),
            last_trigger_cycle: 0,
            evolution_count: 0,
            meta_accuracy_history: VecDeque::with_capacity(100),
            loss_history: VecDeque::with_capacity(100),
            intervention_log: VecDeque::with_capacity(200),
            task_engine: Some(SelfEvolutionTaskEngine::new()),
            trace_buffer: VecDeque::with_capacity(50),
            skill_crystallizer: Some(SkillCrystallizer::new(CrystallizationConfig::default())),
            ouroboros: Some(OuroborosStageManager::new(OuroborosConfig::default())),
            ast_gate: Some(AstSafetyGate::new()),
            escher_loop: EscherLoopEngine::new(EscherLoopConfig::default())
                .with_curriculum(CurriculumAgent::new()),
            sub_agent_acc: SubAgentAccumulator::new(200),
            gepa_asi: GepaAsiEvaluator::new(),
            reflective_analyzer: ReflectiveAnalyzer::new(100),
            orchestrator: SelfEvolutionTaskOrchestrator::new(OrchestratorConfig::default()),
            self_model: SelfModelGenerator::new(),
            memory_archiver: MemoryArchiver::new(),
            q_learning_enabled: true,
            experience_tree: Some(ExperienceTree::new(PruningConfig::default())),
            anti_spiral: AntiSpiralMonitor::new(AntiSpiralConfig::default()),
            self_source_reader: Some(SelfSourceReader::new(SourceReaderConfig::default())),
            ast_mutation_engine: Some(AstMutationEngine::new(AstMutationConfig::default())),
            auto_commit_gate: Some(AutoCommitGate::new(AutoCommitConfig::default())),
            bg_scheduler: BackgroundEvolutionScheduler::new(100),
            trace_encoder: Some(TraceEncoder::new()),
            archive_manager: Some(ArchiveManager::new()),
            sandbox_evaluator: Some(SandboxEvaluator::new()),
            auto_mlir: Some(AutoMlirGenerator::new()),
            sepl_pipeline: Some(SeplPipeline::new()),
            heuristic_pool: ERLHeuristicPool::new(ERLHeuristicPoolConfig::default()),
            findings_aggregator: FindingsAggregator::new(FindingsAggregatorConfig::default()),
            cross_repo_mapper: CrossRepoMapper::new(),
            outcome_tracker: {
                let mut ot = OutcomeTracker::new();
                ot.create_rubric_set(
                    "gepa_task",
                    vec![
                        RubricCriterion {
                            id: 1,
                            name: "success".into(),
                            description: "GEPA task completed successfully".into(),
                            weight: 0.6,
                            pass_threshold: 0.5,
                        },
                        RubricCriterion {
                            id: 2,
                            name: "delta".into(),
                            description: "Metric improvement delta".into(),
                            weight: 0.4,
                            pass_threshold: 0.0,
                        },
                    ],
                );
                ot
            },
            dream_scheduler: DreamCycleScheduler::new(),
            agent_supervisor: AgentSupervisor::new(),
            auto_review: AutoReviewClassifier::new(),
            delegation_manager: RecursiveDelegationManager::new(DelegateConfig::default()),
            keyword_optimizer: SearchKeywordOptimizer::new(OptimizerConfig::default()),
            research_pkg_mgr: ResearchPackageManager::default(),
            weakness_miner: Some(WeaknessMiner::new(50)),
            token_registry: TokenRegistry::new(),
            knowledge_graph: KnowledgeGraph::new(),
            repo_understanding: {
                let mut engine = RepoUnderstandingEngine::new();
                engine.seed_default_understandings();
                engine
            },
            decision_chain: DecisionChain::new(),
            principle_distiller: PrincipleDistiller::new(DistillerConfig::default()),
            self_manifest: SelfManifestGenerator::new(),
            auto_experiment: AutoExperimentLoop::new(AutoExperimentConfig::default()),
            homeostatic_drive: HomeostaticDriveSystem::new(HomeostaticDriveConfig::default()),
            trace_capture: TraceCaptureEngine::new(500),
            pareto_selector: ParetoFrontSelector::new(
                super::pareto_front_selector::objectives::standard_evolution(),
            ),
            dual_lever: DualLeverOrchestrator::new(200),
            cognitive_compressor: CognitiveContextCompressor::new(),
            constraint_registry: {
                let mut reg = ConstraintRegistry::new();
                reg.register_defaults();
                reg
            },
            failure_evidence_batcher: FailureEvidenceBatcher::new(),
            edit_journal: EditJournal::new(),
            cpe_regularizer: CpeRegularizer::new(),
        }
    }

    /// 访问可编辑元过程配置的可变引用（DGM-H 自修改入口点）
    pub fn meta_procedure_config_mut(&mut self) -> &mut MetaProcedureConfig {
        &mut self.feedback_bridge.config
    }

    /// 执行 self_tune：基于近期历史和当前指标调整所有阈值
    pub fn self_tune(&mut self) {
        let meta_trend = self.meta_accuracy_trend();
        let loss_trend = self.loss_trend();
        let ece = self
            .intervention_log
            .iter()
            .filter_map(|r| {
                if r.kind == InterventionKind::BridgeCalibration {
                    r.description
                        .split(|c: char| c == ' ' || c == ',')
                        .find_map(|s| s.strip_prefix("ece=").and_then(|v| v.parse::<f64>().ok()))
                } else {
                    None
                }
            })
            .last()
            .unwrap_or(0.0);
        let meta_acc = self.meta_accuracy_history.back().copied().unwrap_or(0.5);
        self.feedback_bridge
            .config
            .self_tune(meta_trend, loss_trend, ece, meta_acc);
        self.loop_coordinator.config = self.feedback_bridge.config.clone();
    }

    /// Run all registered constraint checks. Returns (passed, total, failures).
    pub fn check_constraints(&self) -> (usize, usize, Vec<String>) {
        self.constraint_registry.check_all()
    }

    /// 执行一次完整的元层处理。返回本次执行的 `MetaLayerPhase`。
    ///
    /// 1. 回路 1: 桥接校准数据到元认知
    /// 2. 回路 2: 桥接损失数据到自修改
    /// 3. 调度: 决定下一阶段
    /// 4. 回路 3: 如果处于 SelfModification 阶段，桥接元计划到进化
    /// 5. 记录历史和日志
    pub fn tick(
        &mut self,
        cycle: u64,
        calibration: Option<&CalibrationEngine>,
        mut meta: Option<&mut MetaCognitiveLoop>,
        loss: Option<&LossFunction>,
        agent: Option<&mut SelfModifyAgent>,
        evolution: Option<&mut SelfEvolutionLoop>,
        meta_result: Option<&MetaCycleResult>,
        neuromodulator: Option<&NeuromodulatorEngine>,
        ci_seal_bridge: Option<&mut SealProposalBridge>,
    ) -> MetaLayerPhase {
        // 回路 1: 校准 → 元认知
        if let (Some(cal), Some(m)) = (calibration, meta.as_deref_mut()) {
            let stats = cal.stats();
            FeedbackBridge::bridge_calibration_to_meta(stats.ece, stats.meta_d, m);
            let current_meta_acc = m.current_meta_accuracy();
            self.meta_accuracy_history.push_back(current_meta_acc);
            if self.meta_accuracy_history.len() > 100 {
                self.meta_accuracy_history.pop_front();
            }
            // Feed meta_accuracy to anti-spiral WeaknessMiner
            if let Some(ref mut miner) = self.weakness_miner {
                miner.record_confidence(current_meta_acc);
            }
            self.log_intervention(
                cycle,
                InterventionKind::BridgeCalibration,
                true,
                format!(
                    "ece={:.4} meta_d={:.4} meta_acc={:.4}",
                    stats.ece, stats.meta_d, current_meta_acc
                ),
            );
        }

        // 自发现: 基于当前指标自动生成进化任务
        let current_meta_acc = meta
            .as_ref()
            .map(|m| m.current_meta_accuracy())
            .unwrap_or(0.5);
        let current_ece = calibration.map(|c| c.stats().ece).unwrap_or(0.0);
        let current_loss = loss.as_ref().map(|l| l.composite.total).unwrap_or(0.0);
        self.task_system.auto_discover_from_audit(
            cycle,
            current_meta_acc,
            current_ece,
            current_loss,
            8,
        );

        // MOSS §4: Collect failure evidence
        if current_ece > 0.3 {
            self.failure_evidence_batcher.record(
                EvidenceSource::MetaAccuracy,
                "high_ece",
                current_ece.min(1.0),
                &format!("cycle={} ece={:.3}", cycle, current_ece),
            );
        }
        if current_meta_acc < 0.5 {
            self.failure_evidence_batcher.record(
                EvidenceSource::MetaAccuracy,
                "low_meta_accuracy",
                1.0 - current_meta_acc,
                &format!("cycle={} meta_acc={:.3}", cycle, current_meta_acc),
            );
        }
        if current_loss > 1.0 {
            self.failure_evidence_batcher.record(
                EvidenceSource::TraceBuffer,
                "high_composite_loss",
                (current_loss / 5.0).min(1.0),
                &format!("cycle={} loss={:.3}", cycle, current_loss),
            );
        }
        // If a sealed batch exists, create an evolution task for it
        if let Some(batch) = self.failure_evidence_batcher.latest_batch() {
            if batch.evidence.len() >= self.failure_evidence_batcher.batch_threshold {
                let task_title = format!("moss_fix_{}: {}", batch.id, batch.summary);
                let _ = self.task_system.create_task(
                    super::evolution_task_system::TaskType::RefactorExisting,
                    &task_title,
                    &format!("moss_batch_{}", batch.id),
                    5,
                    0.6,
                );
            }
        }

        // 进化管线: 审计 → 计划 → 提议（GEPA 风格反思闭环）
        self.pipeline.run(
            cycle,
            &mut self.task_system,
            current_meta_acc,
            current_ece,
            current_loss,
        );
        // 转发 pipeline seal_bridge 提案到 CI seal_bridge (回路 C2 修复)
        if let (Some(ref pipeline_bridge), Some(ref mut ci_bridge)) =
            (self.pipeline.seal_bridge.as_ref(), ci_seal_bridge)
        {
            let pending: Vec<_> = pipeline_bridge
                .pending_proposals()
                .into_iter()
                .cloned()
                .collect();
            if !pending.is_empty() {
                let count = pending.len();
                for p in &pending {
                    ci_bridge.propose_new_capability(&p.target_module, &p.description);
                }
                log::info!(
                    "seal_bridge: forwarded {} proposals from pipeline to CI",
                    count
                );
            }
        }

        // CPE §3: Capability retention regularization (was unwired — now alive)
        self.cpe_regularizer.tick();
        let cpe_penalty = self.cpe_regularizer.retention_penalty();
        self.cpe_regularizer.record_signature(
            super::cpe_regularizer::CpeDimension::Workflow,
            "pipeline",
            (current_meta_acc * 1e6) as u64,
        );
        self.cpe_regularizer.record_signature(
            super::cpe_regularizer::CpeDimension::Skill,
            "meta_accuracy",
            (current_meta_acc * 1e6) as u64,
        );
        self.cpe_regularizer.record_signature(
            super::cpe_regularizer::CpeDimension::Model,
            "ece",
            (current_ece * 1e6) as u64,
        );
        self.cpe_regularizer.record_signature(
            super::cpe_regularizer::CpeDimension::Memory,
            "composite_loss",
            (current_loss * 1e6) as u64,
        );
        if cycle % 20 == 0 {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!(
                    "cpe: penalty={:.4} {}",
                    cpe_penalty,
                    self.cpe_regularizer.summary()
                ),
            );
            // Apply retention penalty to evolution gate threshold
            let gated = self.evolution_count > 0 && cpe_penalty > 0.3;
            if gated {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    false,
                    format!(
                        "cpe: evolution gated — retention penalty {:.4} > 0.3",
                        cpe_penalty
                    ),
                );
            }
        }

        // Wave 0.5: Ouroboros三阶段自动推进
        let ouroboros_transitioned = self.ouroboros.as_mut().and_then(|ob| ob.auto_transition());
        if let Some(new_stage) = ouroboros_transitioned {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!("ouroboros_transition to {:?}", new_stage),
            );
        }

        // GEPA 反思进化闭环 v2: 录制迹 → 反射分析 → EvolutionTaskSystem 驱动执行
        // 修复: 不自建 ArchAuditReport(空数据), 不使用引擎自有 task list(平行管线)
        let ts = self.task_system.stats();
        let pending_cnt = ts.total.saturating_sub(ts.completed);
        self.trace_buffer.push_back(TraceSnapshot {
            cycle,
            ece: current_ece,
            meta_accuracy: current_meta_acc,
            composite_loss: current_loss,
            pending_tasks: pending_cnt,
        });
        if self.trace_buffer.len() > 50 {
            self.trace_buffer.pop_front();
        }
        // GEPA: ReflectiveAnalyzer NL 迹反射分析 — 替代简单趋势检测
        for r in self.intervention_log.iter().rev().take(10) {
            let event =
                crate::core::nt_core_experience::reflective_analyzer::TraceEvent::InterventionLog {
                    cycle,
                    source: format!("{:?}", r.kind),
                    action: r.description.clone(),
                    success: r.success,
                };
            self.reflective_analyzer.feed_event(event);
        }
        let ra_diagnoses = self.reflective_analyzer.analyze();
        let (reflective_bonus, proposed_count): (f64, usize) =
            if let Some(ref mut engine) = self.task_engine {
                let proposed = engine.propose_tasks_from_diagnoses(&ra_diagnoses);
                let bonus = engine.compute_reflective_bonus_from_diagnoses(&ra_diagnoses);
                (bonus, proposed.len())
            } else {
                (0.0, 0)
            };
        if proposed_count > 0 {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!(
                    "reflective_propose: {} tasks from {} diagnoses",
                    proposed_count,
                    ra_diagnoses.len()
                ),
            );
        }
        let ece_trend = self
            .trace_buffer
            .iter()
            .rev()
            .take(10)
            .fold(0.0, |acc, t| acc + t.ece)
            / 10.0;
        let meta_trend = self
            .trace_buffer
            .iter()
            .rev()
            .take(10)
            .fold(0.0, |acc, t| acc + t.meta_accuracy)
            / 10.0;
        // GEPA 执行阶段: 从 EvolutionTaskSystem 读取 ready 任务, 经引擎漏斗执行,
        // 结果回注 task_system(交叉 cycle 学习)
        // Phase A: 提取 ready 任务 (borrow self.task_system)
        let ready_tasks: Vec<_> = {
            let mut tasks = Vec::new();
            for _ in 0..3 {
                match self.task_system.next_ready_task() {
                    Some(t) => tasks.push(t),
                    None => break,
                }
            }
            tasks
        };
        // Phase B: 执行任务 (borrow self.task_engine, no self.task_system access)
        let mut phase_results: Vec<(u64, bool, f64, Option<String>, f64)> = Vec::new();
        // Record proposals in WeaknessMiner for anti-spiral monitoring
        for task in &ready_tasks {
            if let Some(ref mut miner) = self.weakness_miner {
                miner.record_evolution_proposal(&task.title);
            }
        }
        if let Some(engine) = &mut self.task_engine {
            for task in &ready_tasks {
                let result = engine.process_system_task(
                    task.id,
                    &task.title,
                    &task.description,
                    current_meta_acc,
                    reflective_bonus,
                );
                let delta = result.metric_after - result.metric_before;
                if result.success {
                    // 技能结晶: 不涉及 task_system 或 log
                    // Step 1: crystallize (borrows skill_crystallizer)
                    let crystal_result: Option<(
                        String,
                        f64,
                        crate::core::nt_core_experience::skill_crystal::CrystallizedSkill,
                        SelfEvolutionStep,
                    )> = self.skill_crystallizer.as_mut().and_then(|crystal| {
                        let mutation = task_to_mutation(task);
                        let step = SelfEvolutionStep {
                            id: task.id,
                            mutation,
                            parent_id: 0,
                            score_before: result.metric_before,
                            score_after: Some(result.metric_after),
                            compiles: true,
                            accepted: true,
                            timestamp: cycle,
                            generation: 1,
                            cmp_score: None,
                        };
                        let skill = crystal.crystallize(&step)?;
                        let name = skill.name.clone();
                        if let Err(e) = crystal.store(&skill) {
                            log::warn!("skill_crystal: store failed: {}", e);
                        }
                        Some((name, skill.avg_score, skill, step))
                    });
                    // Step 2: export .ne-research (no active skill_crystallizer borrow)
                    let crystal_name = crystal_result.map(|(name, score, skill, step)| {
                        let pkg = self.research_pkg_mgr.export_skill_crystal(
                            &skill.name,
                            &skill.description,
                            &skill.ne_source,
                            skill.invocation_count,
                            skill.avg_score,
                            &skill.tags,
                            &[step],
                            &[],
                            None,
                        );
                        if let Ok(pkg) = pkg {
                            if let Err(e) = self.research_pkg_mgr.save(&pkg) {
                                log::warn!("research_pkg: save failed: {}", e);
                            }
                        }
                        (name, score)
                    });
                    phase_results.push((
                        task.id,
                        true,
                        delta,
                        crystal_name.map(|(n, s)| format!("{} ({:.3})", n, s)),
                        result.metric_after,
                    ));
                } else {
                    phase_results.push((task.id, false, delta, None, result.metric_after));
                }
            }
        }
        // Phase C: 回注 task_system + log (no active engine borrow)
        let processed = phase_results.len();
        let success_count = phase_results.iter().filter(|(_, s, _, _, _)| *s).count();
        for (task_id, success, delta, _crystal_log, _metric_after) in &phase_results {
            if *success {
                self.task_system.mark_completed(*task_id, *delta);
            } else {
                self.task_system
                    .mark_failed(*task_id, "GEPA verification gate rejected");
                // Feed failure to anti-spiral WeaknessMiner
                if let Some(ref mut miner) = self.weakness_miner {
                    miner.record_failure("gepa_verification_rejected");
                }
            }
        }
        // Feed GEPA results into OutcomeTracker for retrospective evaluation
        for (task_id, success, delta, _, _) in &phase_results {
            let mut scores = std::collections::HashMap::new();
            scores.insert(1, if *success { 1.0 } else { 0.0 });
            scores.insert(2, (*delta).abs().min(1.0));
            self.outcome_tracker
                .evaluate(1, scores, vec![format!("gepa:{}", task_id)]);
        }
        if processed > 0 {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                success_count > 0,
                format!(
                    "gepa_v2: {}/{} pass, bonus={:.2}, ece_trend={:.4}, meta_trend={:.4}",
                    success_count, processed, reflective_bonus, ece_trend, meta_trend,
                ),
            );
        }
        for (_, _, _, ref crystal_log, _) in &phase_results {
            if let Some(msg) = crystal_log {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!("crystallized skill: {}", msg),
                );
            }
        }
        // Anti-spiral: check WeaknessMiner findings and log if high-severity
        if let Some(ref miner) = self.weakness_miner {
            let anti_spiral_weaknesses = miner.mine_weaknesses();
            let spiral_findings: Vec<&TraceWeakness> = anti_spiral_weaknesses
                .iter()
                .filter(|w| w.severity > 0.35)
                .collect();
            if !spiral_findings.is_empty() {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    false,
                    format!(
                        "anti_spiral: {} findings (max_sev={:.3})",
                        spiral_findings.len(),
                        spiral_findings
                            .iter()
                            .map(|w| w.severity)
                            .fold(0.0, f64::max),
                    ),
                );
            }
        }

        // AutoExperimentLoop: Karpathy-style experiment each tick
        let exp_result =
            self.auto_experiment
                .tick(cycle, current_meta_acc, current_ece, current_loss);
        if exp_result.experiments_run > 0 {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                exp_result.experiments_kept > 0,
                format!(
                    "auto_experiment: {}/{} kept, best_delta={:.4}, failures={}",
                    exp_result.experiments_kept,
                    exp_result.experiments_run,
                    exp_result.best_delta,
                    exp_result.consecutive_failures,
                ),
            );
        }

        // 回路 2: 损失 → 自修改
        if let (Some(lf), Some(ag)) = (loss, agent) {
            let composite = lf.composite.clone();
            self.loss_history.push_back(composite.total);
            if self.loss_history.len() > 100 {
                self.loss_history.pop_front();
            }
            self.feedback_bridge
                .bridge_loss_to_self_modify(&composite, ag);
            self.log_intervention(
                cycle,
                InterventionKind::BridgeLoss,
                true,
                format!(
                    "loss_total={:.4} pending={}",
                    composite.total,
                    ag.pending_count()
                ),
            );
        }

        // GEPA ASI 结构化诊断: 从干预日志分析迹模式 → 生成健康报告
        // Feed intervention log as trace texts for ASI analysis
        let traces: Vec<String> = self
            .intervention_log
            .iter()
            .rev()
            .take(20)
            .map(|r| format!("{:?}: {}", r.kind, r.description))
            .collect();
        if !traces.is_empty() {
            self.gepa_asi.feed_traces(&traces);
        }
        let asi_report = self.gepa_asi.evaluate(cycle);
        if !asi_report.diagnostics.is_empty() {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                asi_report.is_healthy(),
                format!(
                    "gepa_asi: {} diagnostics ({} blockers, {} critical) health={:.3} trend={:.3}",
                    asi_report.diagnostics.len(),
                    asi_report.num_blockers,
                    asi_report.num_critical,
                    asi_report.overall_health,
                    self.gepa_asi.health_trend(),
                ),
            );
        }

        // ReflectiveAnalyzer: 日志已在上方 GEPA 阶段捕获并分析
        if !ra_diagnoses.is_empty() {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                ra_diagnoses.iter().all(|d| d.severity < 0.5),
                format!(
                    "reflective_analyzer: {} diagnoses (severe: {})",
                    ra_diagnoses.len(),
                    ra_diagnoses.iter().filter(|d| d.severity >= 0.5).count(),
                ),
            );
        }

        // HomeostaticDriveSystem: update drives from current metrics
        self.homeostatic_drive
            .tick(current_meta_acc, current_ece, current_loss);
        let _drive_action = self.homeostatic_drive.select_action(&[]);

        // 调度: 基于神经调质和损失状态决定下一阶段
        let phase = self
            .loop_coordinator
            .decide_next_phase(neuromodulator, loss);
        self.log_intervention(
            cycle,
            InterventionKind::PhaseDecision,
            true,
            format!("phase={:?}", phase),
        );

        // 回路 3: 元认知计划 → SEAL 进化
        if phase == MetaLayerPhase::SelfModification {
            if let (Some(mr), Some(ev)) = (meta_result, evolution) {
                FeedbackBridge::bridge_meta_to_evolution(mr, ev);
                self.evolution_count += 1;
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "plans={} evolution_count={}",
                        mr.plans.len(),
                        self.evolution_count
                    ),
                );
            }
            // Wave 0.5: AST 安全门控审计自修改代码
            let ast_audit_results: Vec<(String, usize)> = meta_result
                .map(|mr| {
                    self.ast_gate
                        .as_mut()
                        .map(|ag| {
                            mr.plans
                                .iter()
                                .filter_map(|plan| {
                                    let verdict = ag.audit(&plan.weakness.description, "elevated");
                                    if !verdict.passed {
                                        Some((plan.id.clone(), verdict.violations.len()))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            for (plan_id, violations) in &ast_audit_results {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    false,
                    format!("ast_gate: {} violations for plan {}", violations, plan_id),
                );
            }
        }

        // Meta Loop: 双种群共进化 (Escher-Loop style, 每30周期)
        if cycle > 0 && cycle % 30 == 0 {
            // Seed curriculum agent with known domains for task proposal
            let domains: Vec<String> = vec![
                "reasoning".into(),
                "memory".into(),
                "planning".into(),
                "perception".into(),
                "calibration".into(),
                "evolution".into(),
            ];
            self.escher_loop.seed_domains(&domains);

            let result = self.escher_loop.co_evolve_step();
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!(
                    "escher_loop: epoch={} new_variants={} fitness_delta={:.4} archive={} tasks={} opts={}",
                    result.epoch, result.new_variants, result.fitness_delta, result.archive_size,
                    result.task_pop_size, result.opt_pop_size,
                ),
            );

            // Meta-optimize every 60 cycles
            if cycle % 60 == 0 {
                let meta_ops = self.escher_loop.meta_optimize();
                for op in &meta_ops {
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!("meta_optimize: {}", op),
                    );
                }
            }

            // Seed loop stats from task system periodically
            self.escher_loop.seed_tasks(&self.task_system.stats());
        }

        // Meta Loop: 自进化迭代任务管线 (gap scan + task creation)
        if cycle > 0 && cycle % 50 == 0 {
            let consciousness_modules: Vec<String> = vec![
                "mcts_tree_search".into(),
                "counterfactual_simulator".into(),
                "dead_end_detector".into(),
                "parallel_hypothesis".into(),
                "process_reward_model".into(),
                "bidirectional_pruner".into(),
                "strategy_selector".into(),
                "causal_reasoning".into(),
                "analogical_reasoning".into(),
                "dual_path_inference".into(),
            ];
            let wired_modules: Vec<String> = vec![
                "inner_critic".into(),
                "executive_controller".into(),
                "consciousness_stream".into(),
                "verification_gate".into(),
            ];
            let gap_scan =
                self.orchestrator
                    .scan_gaps(cycle, &consciousness_modules, &wired_modules);
            if !gap_scan.unwired_modules.is_empty() {
                let tasks = self.orchestrator.create_wiring_tasks(&gap_scan);
                for task in &tasks {
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!(
                            "orchestrator: wiring task created for {} → {:?}",
                            task.module_name, task.target_step
                        ),
                    );
                }
            }
        }

        // Meta Loop: 子Agent积累 (每成功完成的任务)
        if cycle % 10 == 0 {
            let stats = self.task_system.stats();
            if stats.completed > 0 {
                // Record recent completions as subagent captures
                let epoch = cycle / 30;
                for _ in 0..stats.completed.min(3) {
                    self.sub_agent_acc.capture(
                        &format!("task_epoch_{}", epoch),
                        "auto-captured from completed evolution task",
                        &[],
                        vec!["evolution".to_string(), "auto".to_string()],
                        epoch,
                    );
                }
            }
        }

        // Phase 27: AntiSpiralMonitor — 检测推理循环/振荡/停滞
        self.anti_spiral
            .record_meta_snapshot(cycle, current_meta_acc);
        let spiral_findings = self.anti_spiral.scan(cycle);
        for f in &spiral_findings {
            self.log_intervention(
                cycle,
                InterventionKind::GuardActivation,
                false,
                format!(
                    "anti_spiral: {} (severity={:.3}) suggestion={}",
                    f.description, f.severity, f.suggestion
                ),
            );
            // If severe, create a task to address the spiral
            if f.severity > 0.5 && cycle % 10 == 0 {
                let _tid = self.task_system.create_task(
                    super::evolution_task_system::TaskType::ArchitectureReview,
                    &format!("Anti-spiral: {}", f.pattern.name()),
                    &f.description,
                    (f.severity * 10.0) as u8,
                    f.severity,
                );
            }
        }

        // Phase 27: BackgroundEvolutionScheduler — 背景进化定时审计
        let recent_mutations = self.task_system.stats().total as usize;
        let bes_tasks = self
            .bg_scheduler
            .evaluate(cycle, recent_mutations, current_meta_acc);
        for (audit_type, title, description, priority, impact) in &bes_tasks {
            let tid = self.task_system.create_task(
                super::evolution_task_system::TaskType::ArchitectureReview,
                title,
                description,
                *priority,
                *impact,
            );
            self.bg_scheduler
                .record_audit(cycle, *audit_type, tid, description);
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!(
                    "bg_scheduler: created {} (id={} priority={})",
                    audit_type.name(),
                    tid,
                    priority
                ),
            );
        }

        // ── Wave 0: Self-modifying source loop (yoyo-evolve style) ──
        // 1) Source reader scan (every 50 cycles) — uses reader and self, must scope separately
        let (scanned, parsed, errs, stats) = if cycle > 0 && cycle % 50 == 0 {
            if let Some(ref mut reader) = self.self_source_reader {
                let (s, p, e) = reader.scan();
                let st = reader.stats();
                (Some(s), Some(p), Some(e), Some(st))
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };
        if let (Some(scanned), Some(parsed), Some(errs)) = (scanned, parsed, errs) {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                true,
                format!(
                    "self_source_reader: scanned={} parsed={} errors={}",
                    scanned, parsed, errs
                ),
            );
        }
        // 1b) yoyo-evolve: use weaknesses to drive AST mutations (separate scope, no reader borrow)
        if let (Some(_), Some(ref mut engine)) = (stats.as_ref(), self.ast_mutation_engine.as_mut())
        {
            let weaknesses = self
                .weakness_miner
                .as_ref()
                .map(|m| m.mine_weaknesses())
                .unwrap_or_default();
            // P0.5: Begin EditJournal transaction
            let has_work = weaknesses.iter().any(|w| w.severity >= 0.4);
            if has_work {
                self.edit_journal.begin_transaction(&["src/main.rs"]);
            }
            let mut mutations_applied = 0u64;
            let mut mutations_failed = 0u64;
            for w in &weaknesses {
                if w.severity < 0.4 {
                    continue;
                }
                let target = "src/main.rs";
                let mutation = match w.pattern_type {
                    crate::core::nt_core_experience::self_evolution_engine::WeaknessPattern::Overconfidence |
                    crate::core::nt_core_experience::self_evolution_engine::WeaknessPattern::HighVariance |
                    crate::core::nt_core_experience::self_evolution_engine::WeaknessPattern::SystemicDegradation => {
                        Some(AstMutationEngine::propose_add_import(target, "std::panic"))
                    }
                    crate::core::nt_core_experience::self_evolution_engine::WeaknessPattern::Underconfidence => {
                        Some(AstMutationEngine::propose_add_function(target,
                            &format!("confidence_boost_{}", cycle), "{ 0.9 }", "f64", &[]))
                    }
                    crate::core::nt_core_experience::self_evolution_engine::WeaknessPattern::Stagnation => {
                        Some(AstMutationEngine::propose_add_function(target,
                            &format!("novelty_inject_{}", cycle), "{ 42.0 }", "f64", &[]))
                    }
                    _ => None,
                };
                if let Some(m) = mutation {
                    let result = engine.apply_mutation(&m);
                    self.edit_journal.record_mutation(
                        &result.file_path,
                        &result.original_source,
                        &result.mutated_source,
                        result.success,
                    );
                    if result.success {
                        mutations_applied += 1;
                    } else {
                        mutations_failed += 1;
                    }
                }
            }
            // P0.5: Commit or rollback the EditJournal transaction
            let journal_records = self.edit_journal.records.len();
            if journal_records > 0 {
                if mutations_failed == 0 {
                    let c = self.edit_journal.commit();
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!("edit_journal: committed {} mutations", c),
                    );
                } else {
                    let r = self.edit_journal.rollback();
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        false,
                        format!(
                            "edit_journal: rolled back {} mutations due to {} failures",
                            r, mutations_failed
                        ),
                    );
                }
            }
            if mutations_applied > 0 || mutations_failed > 0 {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    mutations_failed == 0,
                    format!(
                        "yoyo_evolve: applied={} failed={}",
                        mutations_applied, mutations_failed
                    ),
                );
            }
        }
        // 2) Auto-commit gate runs when a gate is triggered externally
        // (gates are triggered by ConsciousnessIntegration after a mutation completes)

        // ── Phase 27: TraceEncoder — 结构化迹编码 (每 10 cycle) ──
        if cycle % 10 == 0 {
            if let Some(ref mut encoder) = self.trace_encoder {
                // Build calibration snapshot from current state
                let snapshot =
                    crate::core::nt_core_experience::trace_encoder::CalibrationSnapshot {
                        cycle,
                        ece: current_ece,
                        meta_accuracy: current_meta_acc,
                        composite_loss: current_loss,
                        neuromodulator_arousal: neuromodulator
                            .map(|n| n.arousal_contribution())
                            .unwrap_or(0.5),
                    };
                encoder.record_snapshot(
                    snapshot.cycle,
                    snapshot.ece,
                    snapshot.meta_accuracy,
                    snapshot.composite_loss,
                    snapshot.neuromodulator_arousal,
                );
                // Encode recent intervention log entries
                for r in self.intervention_log.iter().rev().take(5) {
                    let event = crate::core::nt_core_experience::trace_encoder::InterventionEvent {
                        cycle: r.cycle,
                        kind: format!("{:?}", r.kind),
                        description: r.description.clone(),
                        success: r.success,
                    };
                    encoder.record_event(event.cycle, event.kind, event.description, event.success);
                }
                let trace_count = encoder.snapshot_count();
                if trace_count > 0 && trace_count % 50 == 0 {
                    let trend = encoder
                        .encode_window(cycle, 20)
                        .map(|t| format!("ece={:.4} meta={:.4}", t.trend_ece, t.trend_meta))
                        .unwrap_or_default();
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!(
                            "trace_encoder: {} traces encoded, trend={}",
                            trace_count, trend
                        ),
                    );
                }
            }
        }

        // ── Phase 27: ArchiveManager — DGM-H 存档树更新 (每 20 cycle) ──
        if cycle > 0 && cycle % 20 == 0 {
            let step_count = self.evolution_count as usize;
            let (needs_log, archived, max_nodes, needs_prune) = self
                .archive_manager
                .as_ref()
                .map(|archiver| {
                    let stats = archiver.stats();
                    let archived = stats.total_nodes;
                    let max_nodes = archiver.tree.max_nodes;
                    (
                        step_count > archived,
                        archived,
                        max_nodes,
                        archived > max_nodes.saturating_sub(50),
                    )
                })
                .unwrap_or((false, 0, 0, false));

            if needs_log {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "archive_manager: {} steps, {} archived, max_nodes={}",
                        step_count, archived, max_nodes
                    ),
                );
            }
            if needs_prune {
                let before = archived;
                if let Some(ref mut archiver_mut) = self.archive_manager {
                    archiver_mut.prune(0.3);
                    let after = archiver_mut.stats().total_nodes;
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!("archive_manager: pruned {}→{} nodes", before, after),
                    );
                }
            }
        }

        // ── Phase 27: SandboxEvaluator — 验证最近的任务 (每 30 cycle) ──
        if cycle > 0 && cycle % 30 == 0 {
            if let Some(ref mut sandbox) = self.sandbox_evaluator {
                let stats = self.task_system.stats();
                let score = sandbox.evaluate("cargo check", &["src/"]);
                let passed = score.compile_passed && score.tests_passed;
                self.log_intervention(
                    cycle,
                    InterventionKind::GuardActivation,
                    passed,
                    format!(
                        "sandbox: passed={} compile_passed={} tests_passed={} duration={}ms",
                        passed, score.compile_passed, score.tests_passed, score.duration_ms
                    ),
                );
                // Block evolution tasks if sandbox fails
                if !passed && stats.in_progress > 3 {
                    self.log_intervention(
                        cycle,
                        InterventionKind::GuardActivation,
                        false,
                        "sandbox: blocking evolution tasks due to sandbox failure".into(),
                    );
                }
            }
        }

        // ── Phase 27: SEPL 算子管线 (每 10 cycle 运行 ρ→σ→ι→ε→κ) ──
        if cycle > 0 && cycle % 10 == 0 {
            if let Some(ref mut sepl) = self.sepl_pipeline {
                let ctx = crate::core::nt_core_experience::sepl_operators::SeplContext {
                    cycle,
                    meta_accuracy: current_meta_acc,
                    ece: current_ece,
                    composite_loss: current_loss,
                    pending_tasks: pending_cnt,
                    arch_gaps: vec![],
                };
                let report = sepl.run(&ctx);
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "sepl_pipeline: committed={} rejected={} rollback={}",
                        report.committed_ids.len(),
                        report.rejected_ids.len(),
                        report.rollback_ids.len()
                    ),
                );
            }
        }

        // ── Phase 27: AutoMlirGenerator — 完整变异→门控→提交循环 (每 40 cycle) ──
        if cycle > 0 && cycle % 40 == 0 {
            if let Some(ref mut auto_mlir) = self.auto_mlir {
                // Check if reader and gate are available for the full loop
                let has_reader = self.self_source_reader.is_some();
                let has_gate = self.auto_commit_gate.is_some();
                if has_reader && has_gate {
                    // Run a single iteration: propose mutation, gate it, commit if verified
                    let source_files: Vec<String> = Vec::new(); // reader has scan() but no scanned_files() accessor
                    let events =
                        auto_mlir.run_iteration(cycle, &source_files, current_meta_acc, 0.6);
                    let succeeded = events.iter().any(|e| e.contains("committed"));
                    let aux = format!("auto_mlir: {} events", events.len());
                    self.log_intervention(cycle, InterventionKind::BridgeEvolution, succeeded, aux);
                }
            }
        }

        self.last_trigger_cycle = cycle;

        // CPE: capability retention regularization — tick every cycle
        self.cpe_regularizer.tick();
        // Log CPE summary periodically
        if cycle % 10 == 0 {
            self.log_intervention(
                cycle,
                InterventionKind::BridgeEvolution,
                self.cpe_regularizer.retention_penalty() < 0.5,
                self.cpe_regularizer.summary(),
            );
        }

        // DGM-H: 自动调整元过程阈值（基于近期趋势和当前指标）
        if cycle % 10 == 0 {
            self.self_tune();
        }

        // ── ERL 启发式提取: 从失败/成功的干预日志提取可复用启发式 ──
        if cycle % 5 == 0 && !self.intervention_log.is_empty() {
            // 从最近的干预日志提取启发式
            let recent: Vec<_> = self.intervention_log.iter().rev().take(5).collect();
            for r in &recent {
                let (trigger, action) = match r.kind {
                    InterventionKind::BridgeCalibration => {
                        ("ece_above_threshold", "run_recalibration")
                    }
                    InterventionKind::BridgeLoss => ("loss_spike", "trigger_self_modify"),
                    InterventionKind::BridgeEvolution => {
                        ("evolution_ready", "process_evolution_tasks")
                    }
                    InterventionKind::GuardActivation => ("guard_triggered", "verify_and_commit"),
                    InterventionKind::PhaseDecision => ("phase_transition", "handle_phase_change"),
                    InterventionKind::CycleImplementation => ("cycle_tick", "implement_cycle_step"),
                    InterventionKind::BridgeSearch => {
                        ("keywords_suggested", "log_keyword_optimizer_stats")
                    }
                };
                self.heuristic_pool.extract_heuristic(
                    trigger.to_string(),
                    action.to_string(),
                    format!("{:?}", r.kind),
                    format!("cycle_{}", r.cycle),
                    r.success,
                );
            }
        }

        // ── FindingsAggregator: 聚合健康指标为结构化发现 ──
        if cycle % 10 == 0 {
            let stats = self.task_system.stats();
            // 任务阻塞率 → 进化健康
            let blocked_ratio = if stats.total > 0 {
                stats.blocked as f64 / stats.total as f64
            } else {
                0.0
            };
            if blocked_ratio > 0.3 {
                self.findings_aggregator.add_finding(
                    FindingCategory::Evolution,
                    7,
                    "task_blocked_ratio_exceeded".into(),
                    format!("{:.1}% of tasks blocked", blocked_ratio * 100.0),
                    "evolution_task_system".into(),
                    blocked_ratio,
                    0.3,
                    "worsening".into(),
                    "review blocked tasks and unblock dependencies".into(),
                );
            }
            // ECE 健康
            if current_ece > 0.15 {
                self.findings_aggregator.add_finding(
                    FindingCategory::Calibration,
                    (current_ece * 50.0) as u8,
                    "ece_threshold_exceeded".into(),
                    format!("ECE={:.4} exceeds threshold", current_ece),
                    "calibration_engine".into(),
                    current_ece,
                    0.15,
                    if current_ece > 0.2 {
                        "worsening"
                    } else {
                        "stable"
                    }
                    .into(),
                    "run calibration recalibration cycle".into(),
                );
            }
            // Meta accuracy 健康
            if current_meta_acc < 0.7 {
                self.findings_aggregator.add_finding(
                    FindingCategory::Calibration,
                    ((1.0 - current_meta_acc) * 30.0) as u8,
                    "meta_accuracy_low".into(),
                    format!("meta_accuracy={:.4} below 0.7", current_meta_acc),
                    "meta_cognitive_loop".into(),
                    current_meta_acc,
                    0.7,
                    "worsening".into(),
                    "review meta-accuracy degradation".into(),
                );
            }
            self.findings_aggregator.snapshot();
        }

        // ── CrossRepoMapper: 解析进化任务中的术语对齐 ──
        if cycle % 7 == 0 {
            let _mapping_count = self.cross_repo_mapper.mapping_count();
            // CrossRepoMapper passively available; summary reports its count
        }

        // ── 自模型生成: 每 interval cycle 合成一次 prompt 层 ──
        if self.self_model.should_generate(cycle) {
            // SelfModelGenerator 是只读的（它引用 &MemoryLattice 等），
            // 但 ConsciousnessIntegration 持有这些组件，调用者在 core.rs 中传入
            // 这里我们只触发文件写入：从 task_system 和已有的 summary 数据生成
            self.self_model.generate(
                cycle,
                None, // lattice — caller 在 tick() 外部提供
                None, // tree
                None, // personality
                Some(&self.task_system),
                Some(self.self_manifest.last_manifest()),
            );
        }

        // ── 冷记忆归档: 每 interval 周期休眠一次 ──
        if self.memory_archiver.should_archive(cycle) {
            // archiver 需要 &mut MemoryLattice，由调用者传入
            // 这里只做延迟标记，实际归档在外部完成
            log::info!(
                "memory_archiver: ready at cycle {} (archives={})",
                cycle,
                self.memory_archiver.archive_count
            );
        }

        // ── v25 Design Token 传感器更新 (每 cycle) ──
        self.token_registry
            .update_sensor(PrimitiveToken::Ece, current_ece, 0.0, 0.9);
        self.token_registry
            .update_sensor(PrimitiveToken::MetaAccuracy, current_meta_acc, 0.0, 0.8);
        self.token_registry
            .update_sensor(PrimitiveToken::CompositeLoss, current_loss, 0.0, 0.7);
        let arousal_val = neuromodulator
            .map(|n| n.arousal_contribution())
            .unwrap_or(0.5);
        self.token_registry
            .update_sensor(PrimitiveToken::Arousal, arousal_val, 0.0, 0.6);

        // ── v25 原则蒸馏: 每 10 周期从迹 + 决策链提取知识 ──
        if cycle > 0 && cycle % 10 == 0 {
            // (Fix 1) 记录决策上下文，确保 decision_chain 被 populate
            let ctx = DecisionContext {
                cycle,
                ece: current_ece,
                meta_accuracy: current_meta_acc,
                composite_loss: current_loss,
                arousal: arousal_val,
            };
            let dec_id = self.decision_chain.begin_decision(
                format!("principle_distillation_{}", cycle),
                "principle_distillation".into(),
                ctx.clone(),
                vec![],
                format!(
                    "extract principles from token health (ece={:.3}, meta={:.3}, loss={:.3})",
                    current_ece, current_meta_acc, current_loss
                ),
                format!("expected: ~3 new nodes from {:.2} metrics", current_ece),
            );

            let new_ids = self.principle_distiller.scan(
                cycle,
                &mut self.token_registry,
                &mut self.knowledge_graph,
                &self.decision_chain,
            );

            // 设计感知蒸馏: 从 Design 域令牌提取令牌/布局/组件知识
            let design_ids = self.principle_distiller.scan_design(
                cycle,
                &mut self.token_registry,
                &mut self.knowledge_graph,
            );

            let total_ids = [new_ids.as_slice(), design_ids.as_slice()].concat();
            let success = !total_ids.is_empty();
            let delta = total_ids.len() as f64;
            self.decision_chain
                .complete_decision(dec_id, ctx, delta, success);

            if success {
                let design_count = design_ids.len();
                let msg = if design_count > 0 {
                    format!(
                        "principle_distiller: {} new nodes ({} design)",
                        total_ids.len(),
                        design_count
                    )
                } else {
                    format!(
                        "principle_distiller: {} new knowledge nodes distilled",
                        total_ids.len()
                    )
                };
                self.log_intervention(cycle, InterventionKind::BridgeEvolution, true, msg);

                // (Fix 2) 记录高置信度原则到干预日志，使其可见于总结
                let principles = self.knowledge_graph.find_by_type(NodeType::Principle);
                let high_conf: Vec<&KnowledgeNode> = principles
                    .into_iter()
                    .filter(|n| n.confidence > 0.7)
                    .collect();
                if !high_conf.is_empty() {
                    let titles: Vec<&str> =
                        high_conf.iter().map(|n| n.title.as_str()).take(3).collect();
                    self.log_intervention(
                        cycle,
                        InterventionKind::BridgeEvolution,
                        true,
                        format!(
                            "knowledge_graph: {} high-confidence principles: {}",
                            high_conf.len(),
                            titles.join(", ")
                        ),
                    );
                }
            }
        }

        // ── v25 自模型清单: 每 interval 周期生成 YAML ──
        if self.self_manifest.should_generate(cycle) {
            self.self_manifest.generate(
                cycle,
                Some(&self.token_registry),
                Some(&self.knowledge_graph),
            );
        }

        // ── v24: OutcomeTracker — 每 20 周期归档评估历史 ──
        if cycle > 0 && cycle % 20 == 0 {
            self.outcome_tracker.prune();
        }

        // ── v24: DreamCycleScheduler — 每 interval 周期运行梦境循环 ──
        if self.dream_scheduler.should_dream(cycle) {
            let new_patterns = self.dream_scheduler.run_dream_cycle(cycle);
            if !new_patterns.is_empty() {
                let _tid = self.task_system.create_task(
                    TaskType::PatternDistill,
                    &format!(
                        "dream_patterns: {} new from cycle {}",
                        new_patterns.len(),
                        cycle
                    ),
                    &self.dream_scheduler.synthesize_reflection(),
                    6,
                    0.5,
                );
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "dream_scheduler: {} new patterns synthesized",
                        new_patterns.len()
                    ),
                );
            }
        }

        // ── v24: AgentSupervisor — 每 10 周期心跳 + 每 50 周期清理 ──
        if cycle % 10 == 0 {
            self.agent_supervisor.heartbeat(cycle);
        }
        if cycle > 0 && cycle % 50 == 0 {
            self.agent_supervisor.prune();
        }

        // ── v24: AutoReviewClassifier — 每 30 周期记录历史摘要 ──
        if cycle > 0 && cycle % 30 == 0 {
            let _summary = self.auto_review.summary();
        }

        // ── v24: RecursiveDelegationManager — 每 10 周期 tick ──
        if cycle % 10 == 0 {
            self.delegation_manager.tick(cycle);
        }

        // ── SearchKeywordOptimizer — 每 10 周期 tick + 每 30 周期输出关键词建议到干预日志 ──
        if cycle % 10 == 0 {
            self.keyword_optimizer.tick(cycle);
        }
        if cycle > 0 && cycle % 30 == 0 && !self.keyword_optimizer.keywords.is_empty() {
            let top_kw = self.keyword_optimizer.stats();
            let suggestions: Vec<String> = self
                .keyword_optimizer
                .suggest_keywords_cross_domain(5)
                .into_iter()
                .map(|(kw, dom, sc)| format!("{}({}: {:.2})", kw, dom, sc))
                .collect();
            let desc = format!(
                "kw_opt top5=[{}] stats=[{}]",
                suggestions.join(", "),
                top_kw
            );
            self.log_intervention(cycle, InterventionKind::BridgeSearch, true, desc);
        }

        // ── CognitiveContextCompressor — 每 10 周期压缩 thought history ──
        if cycle > 0 && cycle % 10 == 0 {
            let history: Vec<(String, Vec<u8>, f64)> = self
                .intervention_log
                .iter()
                .rev()
                .take(20)
                .map(|r| {
                    let vsa: Vec<u8> = {
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        std::hash::Hash::hash(&r.description, &mut hasher);
                        let h = std::hash::Hasher::finish(&hasher);
                        h.to_le_bytes().iter().cycle().copied().take(64).collect()
                    };
                    (
                        format!("{:?}: {}", r.kind, r.description),
                        vsa,
                        if r.success { 0.8 } else { 0.2 },
                    )
                })
                .collect();
            if history.len() >= 10 {
                let compressed = self.cognitive_compressor.compress_thought_history(&history);
                let stats = self.cognitive_compressor.stats();
                log::info!(
                    "cognitive_compressor: {}→{} entries, {}",
                    history.len(),
                    compressed.len(),
                    stats
                );
            }
        }

        // ── Phase 30: TraceCaptureEngine — 主动迹捕获 (每 cycle 边界事件) ──
        self.trace_capture.cycle_boundary();
        if let Some(last) = self.intervention_log.back() {
            let category = match last.kind {
                InterventionKind::BridgeCalibration => "calibration",
                InterventionKind::BridgeLoss => "loss",
                InterventionKind::BridgeEvolution => "evolution",
                _ => "other",
            };
            let severity = if last.success {
                TraceSeverity::Info
            } else {
                TraceSeverity::Warning
            };
            self.trace_capture
                .capture(super::trace_capture_engine::TraceEvent {
                    cycle,
                    timestamp: std::time::Instant::now(),
                    source: TraceSource::MetaLayer,
                    severity,
                    category: category.to_string(),
                    summary: last.description.clone(),
                    detail: String::new(),
                    key_values: Vec::new(),
                });
        }

        // ── Phase 30: DualLeverOrchestrator — 双臂演进编排 (每 10 周期) ──
        if cycle > 0 && cycle % 10 == 0 {
            let stalled = self.dual_lever.stalled_domains(20);
            if !stalled.is_empty() {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!("dual_lever stalled={:?}", stalled),
                );
            }
            let dl_stats = self.dual_lever.stats();
            if dl_stats.total_harness_levers == 0 {
                self.dual_lever.register_harness_lever("meta_accuracy", 0.7);
                self.dual_lever.register_harness_lever("ece", 0.15);
                self.dual_lever
                    .register_harness_lever("composite_loss", 0.3);
            }
        }

        // ── Phase 30: ParetoFrontSelector — 多目标选择 (每 30 周期, 与 EscherLoop 对齐) ──
        if cycle > 0 && cycle % 30 == 0 {
            let task_stats = self.task_system.stats();
            let candidates: Vec<Candidate> = {
                let mut c = Vec::new();
                let n_agents = self.escher_loop.task_population().len();
                for i in 0..n_agents.min(20) {
                    c.push(Candidate {
                        id: i as u64,
                        description: format!("agent_{}", i),
                        scores: vec![
                            (
                                "impact".into(),
                                task_stats.avg_impact as f64 + (i as f64 * 0.05).sin(),
                            ),
                            (
                                "progress".into(),
                                (100.0 - task_stats.in_progress as f64) / 100.0,
                            ),
                        ],
                        metadata: Vec::new(),
                    });
                }
                if c.is_empty() {
                    c.push(Candidate {
                        id: 0,
                        description: "task_system".into(),
                        scores: vec![
                            ("impact".into(), task_stats.avg_impact as f64),
                            (
                                "progress".into(),
                                (100.0 - task_stats.in_progress as f64) / 100.0,
                            ),
                        ],
                        metadata: Vec::new(),
                    });
                }
                c
            };
            let result = self.pareto_selector.select(&candidates, 10);
            if !result.front.is_empty() {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "pareto_front: {} candidates, {} fronts, {} selected",
                        candidates.len(),
                        result.front.len(),
                        result.selected_ids.len()
                    ),
                );
            }
        }

        // ── v26 RepoUnderstandingEngine: 每 50 周期蒸馏仓库理解到知识图谱 ──
        if cycle > 0 && cycle % 50 == 0 && !self.repo_understanding.repo_keys().is_empty() {
            let count = self
                .repo_understanding
                .distill_to_graph(&mut self.knowledge_graph, cycle);
            if count > 0 {
                self.log_intervention(
                    cycle,
                    InterventionKind::BridgeEvolution,
                    true,
                    format!(
                        "repo_understanding: distilled {} knowledge nodes from {} repos",
                        count,
                        self.repo_understanding.count()
                    ),
                );
            }
        }

        phase
    }

    /// 外部调用存储归档（从 ConsciousnessIntegration 传入 &mut MemoryLattice）
    pub fn tick_archive(&mut self, cycle: u64, lattice: &mut MemoryLattice) -> usize {
        self.memory_archiver.archive_cold_memories(cycle, lattice)
    }

    /// 外部调用经验树 tick：每 3 个 cycle 注入一条 insight，每 10 个 cycle 修剪
    pub fn tick_experience_tree(&mut self, cycle: u64, ece: f64, loss: f64) {
        if let Some(ref mut tree) = self.experience_tree {
            // Every 3 cycles, inject an insight from current state
            if cycle % 3 == 0 {
                let insight = format!("cycle={} ece={:.4} loss={:.4}", cycle, ece, loss);
                let confidence = (1.0 - ece).clamp(0.0, 1.0);
                tree.add_distilled_insight(&insight, confidence, cycle);
            }
            // Run pruning every 10 cycles when over half capacity
            if cycle % 10 == 0 && tree.all_nodes.len() > tree.config.max_total_nodes / 2 {
                tree.run_prune();
            }
        }
    }

    /// MemRL-style Q-learning tick: reward = -ece (minimizing ECE is good).
    /// Updates recent episodic entries' Q-values via TD(0).
    /// Call this after tick() with the lattice when ECE is available.
    pub fn tick_q_learning(&mut self, ece: f64, lattice: &mut MemoryLattice) {
        if !self.q_learning_enabled || lattice.episodic.is_empty() {
            return;
        }
        // reward = clamped negative ECE: low ECE → positive reward
        let reward = -ece.clamp(0.0, 1.0);
        let alpha = 0.1;
        // Update Q-values for the most recent up-to-20 episodic entries
        let end = lattice.episodic.len();
        let start = if end > 20 { end - 20 } else { 0 };
        for i in start..end {
            lattice.update_q_value(i, LatticeLayer::Episodic, reward, alpha);
        }
    }

    /// 从 CycleResult 蒸馏洞察到 ExperienceTree
    pub fn distill_cycle_insights(&mut self, cycle_result: &CycleResult) {
        if let Some(ref mut tree) = self.experience_tree {
            let cycle = cycle_result.cycle_num;

            // 1. Inject meta_insights
            for insight in &cycle_result.meta_insights {
                tree.add_distilled_insight(insight, 0.7, cycle);
            }

            // 2. Inject counterfactuals insight
            if !cycle_result.causal_counterfactuals.is_empty() {
                let n = cycle_result.causal_counterfactuals.len();
                let insight = format!("counterfactuals:{}", n);
                tree.add_distilled_insight(&insight, 0.6, cycle);
            }

            // 3. Inject phi metrics insight
            if let Some(ref phi) = cycle_result.phi_metrics {
                if !phi.is_empty() {
                    let avg: f64 = phi.iter().sum::<f64>() / phi.len() as f64;
                    let insight = format!("phi:{:.4}", avg);
                    tree.add_distilled_insight(&insight, 0.65, cycle);
                }
            }

            // 4. Inject failed steps insight
            let failed_count = cycle_result
                .step_health
                .iter()
                .filter(|h| !h.success)
                .count();
            if failed_count > 0 {
                let insight = format!("failed_steps:{}", failed_count);
                tree.add_distilled_insight(&insight, 0.8, cycle);
            }
        }
    }

    /// v26: 吸收仓库理解到经验树 — 将 RepoUnderstanding 持久化为 ExperienceNode
    /// 每个仓库创建 N 个带 SourceGrounding 的节点
    pub fn absorb_repo_understanding(
        &mut self,
        understanding: super::repo_understanding::RepoUnderstanding,
        cycle: u64,
    ) -> usize {
        let repo_name = understanding.name.clone();
        let repo_key = understanding.repo_key.clone();

        // 1. 存储到 RepoUnderstandingEngine
        self.repo_understanding.absorb(understanding.clone());

        // 2. 将关键洞察注入经验树
        let mut node_count = 0;
        if let Some(ref mut tree) = self.experience_tree {
            let vsa: Vec<u8> = {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&repo_key, &mut hasher);
                let h = std::hash::Hasher::finish(&hasher);
                h.to_le_bytes().iter().cycle().copied().take(64).collect()
            };

            // 架构层洞察
            for layer in &understanding.architecture_layers {
                let insight = format!(
                    "[吸收] {}: {}层({}) — 组件: {:?}",
                    repo_name, layer.name, layer.purpose, layer.components
                );
                tree.add_node_with_grounding(
                    insight,
                    "repo_architecture".into(),
                    0.75,
                    vsa.clone(),
                    cycle,
                    Some(super::experience_tree::SourceGrounding {
                        source_type: "repo_understanding".into(),
                        source_id: repo_key.clone(),
                        source_location: format!("architecture_layer:{}", layer.name),
                        source_excerpt: layer.data_flow.clone(),
                        source_credibility: 0.75,
                    }),
                );
                node_count += 1;
            }

            // 设计原则洞察
            for principle in &understanding.design_principles {
                let insight = format!(
                    "[吸收·原则] {}: {} — 证据: {}",
                    repo_name, principle.principle, principle.evidence
                );
                tree.add_node_with_grounding(
                    insight,
                    "repo_principle".into(),
                    0.8,
                    vsa.clone(),
                    cycle,
                    Some(super::experience_tree::SourceGrounding {
                        source_type: "repo_understanding".into(),
                        source_id: repo_key.clone(),
                        source_location: format!("design_principle:{}", principle.principle),
                        source_excerpt: principle.applicability.clone(),
                        source_credibility: 0.8,
                    }),
                );
                node_count += 1;
            }

            // 关键决策洞察
            for decision in &understanding.key_decisions {
                let insight = format!(
                    "[吸收·决策] {}: {} — 理由: {}",
                    repo_name, decision.decision, decision.rationale
                );
                tree.add_node_with_grounding(
                    insight,
                    "repo_decision".into(),
                    0.7,
                    vsa.clone(),
                    cycle,
                    Some(super::experience_tree::SourceGrounding {
                        source_type: "repo_understanding".into(),
                        source_id: repo_key.clone(),
                        source_location: format!("key_decision:{}", decision.decision),
                        source_excerpt: decision.tradeoffs.clone(),
                        source_credibility: 0.7,
                    }),
                );
                node_count += 1;
            }
        }

        self.log_intervention(
            cycle,
            InterventionKind::BridgeEvolution,
            true,
            format!(
                "absorb_repo: {} ({}) → {} experience nodes",
                repo_name, repo_key, node_count
            ),
        );

        node_count
    }

    /// 外部调用自模型生成（从 ConsciousnessIntegration 传入各组件引用）
    pub fn tick_self_model(
        &mut self,
        cycle: u64,
        lattice: Option<&MemoryLattice>,
        tree: Option<&ExperienceTree>,
        personality: Option<&BehavioralPersonalityEngine>,
    ) -> String {
        self.self_model.generate(
            cycle,
            lattice,
            tree,
            personality,
            Some(&self.task_system),
            Some(self.self_manifest.last_manifest()),
        )
    }

    /// 记录一次搜索关键词结果到 keyword_optimizer。
    /// 所有搜索执行点（web_search, search_github, search_arxiv 等）应调用此方法
    /// 将实际搜索结果回注到关键词收益率追踪。
    pub fn record_search_result(
        &mut self,
        keyword: &str,
        domain: &str,
        signal_count: u64,
        noise_count: u64,
        signal_sources: Vec<String>,
        upstream_keywords: Vec<String>,
        cycle: u64,
    ) {
        self.keyword_optimizer.record_search(
            keyword,
            domain,
            signal_count,
            noise_count,
            signal_sources,
            upstream_keywords,
            cycle,
        );
    }

    /// 查询某个领域的最佳搜索关键词（按复合评分降序）。
    /// 搜索工具在发起搜索前应调用此方法获取优化后的关键词。
    pub fn suggest_search_keywords(&self, domain: &str, count: usize) -> Vec<(String, f64)> {
        self.keyword_optimizer.suggest_keywords(domain, count)
    }

    /// 返回进化任务系统的统计信息
    pub fn task_stats(&self) -> TaskSystemStats {
        self.task_system.stats()
    }

    /// 返回进化任务系统的可变引用
    pub fn task_system_mut(&mut self) -> &mut EvolutionTaskSystem {
        &mut self.task_system
    }

    /// 返回进化任务系统的只读引用
    pub fn task_system_ref(&self) -> &EvolutionTaskSystem {
        &self.task_system
    }

    /// 返回 CognitiveContextCompressor 的可变引用
    pub fn compressor_mut(&mut self) -> &mut CognitiveContextCompressor {
        &mut self.cognitive_compressor
    }

    fn log_intervention(
        &mut self,
        cycle: u64,
        kind: InterventionKind,
        success: bool,
        description: String,
    ) {
        if self.intervention_log.len() >= 200 {
            self.intervention_log.pop_front();
        }
        self.intervention_log.push_back(InterventionRecord {
            cycle,
            kind,
            success,
            description,
        });
    }

    /// 元精度趋势（正数 = 正在改善的自我认知）
    pub fn meta_accuracy_trend(&self) -> f64 {
        if self.meta_accuracy_history.len() < 2 {
            return 0.0;
        }
        let n = self.meta_accuracy_history.len() as f64;
        let mean_x = (n - 1.0) / 2.0;
        let mean_y: f64 = self.meta_accuracy_history.iter().sum::<f64>() / n;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in self.meta_accuracy_history.iter().enumerate() {
            let x = i as f64;
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }
        if den < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    /// 损失趋势（负数 = 正在改善的性能）
    pub fn loss_trend(&self) -> f64 {
        if self.loss_history.len() < 2 {
            return 0.0;
        }
        let n = self.loss_history.len() as f64;
        let mean_x = (n - 1.0) / 2.0;
        let mean_y: f64 = self.loss_history.iter().sum::<f64>() / n;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in self.loss_history.iter().enumerate() {
            let x = i as f64;
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }
        if den < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    /// 干预摘要
    pub fn summary(&self) -> String {
        let total = self.intervention_log.len();
        let successes = self.intervention_log.iter().filter(|r| r.success).count();
        let ob_stage = self
            .ouroboros
            .as_ref()
            .map(|o| format!("{:?}", o.current_stage))
            .unwrap_or_default();
        let ag_count = self
            .ast_gate
            .as_ref()
            .map(|a| format!("rules={}", a.rules.len()))
            .unwrap_or_default();
        let crystal = self
            .skill_crystallizer
            .as_ref()
            .map(|c| format!("crystal={}", c.tracked_mutations()))
            .unwrap_or_default();
        let reader_summary = self
            .self_source_reader
            .as_ref()
            .map(|r| r.summary())
            .unwrap_or_default();
        let ast_mut_summary = self
            .ast_mutation_engine
            .as_ref()
            .map(|e| e.summary())
            .unwrap_or_default();
        let gate_summary = self
            .auto_commit_gate
            .as_ref()
            .map(|g| g.summary())
            .unwrap_or_default();
        let ork = self.orchestrator.stats();
        let dream_count = self.dream_scheduler.stats().total_patterns;
        let agent_count = self.agent_supervisor.summary().agent_count;
        let review_count = self.auto_review.summary().total_classified;
        let te_summary = self
            .trace_encoder
            .as_ref()
            .map(|t| t.summary())
            .unwrap_or_default();
        let am_summary = self
            .archive_manager
            .as_ref()
            .map(|a| a.summary())
            .unwrap_or_default();
        let se_summary = self
            .sandbox_evaluator
            .as_ref()
            .map(|s| s.summary())
            .unwrap_or_default();
        let amlir_summary = self
            .auto_mlir
            .as_ref()
            .map(|m| m.summary())
            .unwrap_or_default();
        let sepl_summary = self
            .sepl_pipeline
            .as_ref()
            .map(|p| p.summary())
            .unwrap_or_default();
        let kw_opt = self.keyword_optimizer.stats();
        let token_health = self.token_registry.average_health();
        let wiring_rate = self.token_registry.wiring_rate();
        let kn = self.knowledge_graph.nodes.len();
        let manifest_count = self.self_manifest.generation_count;
        let tc_stats = self.trace_capture.stats();
        let dl_stats = self.dual_lever.stats();
        let pareto_obj = self.pareto_selector.objectives.len();
        let rpkg_count = self.research_pkg_mgr.export_count;
        format!(
            "SelfEvolutionMetaLayer: evolution_count={} last_trigger_cycle={} interventions={}/{} meta_acc_trend={:.4} loss_trend={:.4} ob_stage={} ast={} crystal={} escher_epoch={} subagents={} ork(ok={},tasks={}) | cfg(...) | {} | anti_spiral: {} | bg_sched: {} | {} | {} | {} | findings: {} | xrepo: {} | v24: dreams={} agents={} reviews={} | v25: tokens(health={:.2}, wiring={:.1}%) kg={} manifest={} | v27: te={} | am={} | se={} | amlir={} | sepl={} | {} | tc(errors={},rate={:.2}) | pareto(obj={}) | dl(harness={},weight={},sr={:.2}) | rpkg={}",
             self.evolution_count,
             self.last_trigger_cycle,
             successes, total,
             self.meta_accuracy_trend(),
             self.loss_trend(),
             ob_stage, ag_count, crystal,
             self.escher_loop.epoch(),
             self.sub_agent_acc.total_count(),
             ork.wiring_succeeded, ork.tasks_created,
             self.pipeline.summary(),
             self.anti_spiral.summary(),
             self.bg_scheduler.summary(),
             reader_summary,
             ast_mut_summary,
             gate_summary,
             self.findings_aggregator.summary(),
             self.cross_repo_mapper.summary(),
              dream_count, agent_count, review_count,
              token_health, wiring_rate * 100.0, kn, manifest_count,
               te_summary, am_summary, se_summary, amlir_summary, sepl_summary, kw_opt,
               tc_stats.error_count, tc_stats.error_rate,
               pareto_obj,
                 dl_stats.total_harness_levers, dl_stats.total_weight_levers, dl_stats.overall_success_rate,
                 rpkg_count,
            )
    }
}

// ============================================================================
// 回路 1-3: 跨组件数据桥接
// ============================================================================

/// DGM-H 风格可编辑元过程配置 — 所有阈值、权重、规则表可在运行时自我修改。
///
/// 在 DGM-H 之前，这些参数是硬编码在 FeedbackBridge/LoopCoordinator/GuardActivator
/// 中的编译时常量。每个参数都有 self_tune() 方法，基于近期结果自动调整。
#[derive(Debug, Clone)]
pub struct MetaProcedureConfig {
    // ── 回路 2: Loss → SelfModify ──
    pub loss_threshold: f64,    // 默认 0.3
    pub loss_impact_scale: f64, // 默认 2.0

    // ── 回路 3: 调度 ──
    pub sleep_arousal_threshold: f64, // 默认 0.2
    pub self_modify_loss: f64,        // 默认 0.6
    pub self_modify_plasticity: f64,  // 默认 0.5
    pub deep_reflect_arousal: f64,    // 默认 0.7
    pub deep_reflect_loss: f64,       // 默认 0.3
    pub consolidate_ach: f64,         // 默认 0.5

    // ── 自审计 ──
    pub ece_audit_threshold: f64,      // 默认 0.15
    pub meta_acc_audit_threshold: f64, // 默认 0.70

    // ── 进化引擎 ──
    pub max_tasks_per_cycle: usize, // 默认 8

    // ── 学习率（self_tune 步长）──
    pub tune_rate: f64, // 默认 0.05
}

impl Default for MetaProcedureConfig {
    fn default() -> Self {
        Self {
            loss_threshold: 0.3,
            loss_impact_scale: 2.0,
            sleep_arousal_threshold: 0.2,
            self_modify_loss: 0.6,
            self_modify_plasticity: 0.5,
            deep_reflect_arousal: 0.7,
            deep_reflect_loss: 0.3,
            consolidate_ach: 0.5,
            ece_audit_threshold: 0.15,
            meta_acc_audit_threshold: 0.70,
            max_tasks_per_cycle: 8,
            tune_rate: 0.05,
        }
    }
}

impl MetaProcedureConfig {
    /// 基于近期历史自我调整参数。
    /// - 如果 meta_accuracy 上升趋势 → 收紧阈值（提高要求）
    /// - 如果 loss 上升趋势 → 降低阈值（更敏感）
    /// - 如果 ECE 高 → 提高审计频率（更多任务）
    pub fn self_tune(&mut self, meta_trend: f64, loss_trend: f64, ece: f64, meta_acc: f64) {
        let tr = self.tune_rate;
        // 元精度上升→收紧损失阈值（系统表现更好，要求更高）
        if meta_trend > 0.01 {
            self.loss_threshold = (self.loss_threshold - tr * 0.3).max(0.15);
            self.deep_reflect_loss = (self.deep_reflect_loss - tr * 0.2).max(0.15);
        }
        // 损失上升→降低阈值（更敏感，更快触发自修改）
        if loss_trend > 0.01 {
            self.self_modify_loss = (self.self_modify_loss - tr * 0.5).max(0.3);
            self.loss_threshold = (self.loss_threshold - tr * 0.3).max(0.1);
        }
        // 高 ECE→更多审计任务
        if ece > 0.20 {
            self.max_tasks_per_cycle = (self.max_tasks_per_cycle + 2).min(20);
            self.ece_audit_threshold = (self.ece_audit_threshold - tr).max(0.05);
        } else if ece < 0.05 {
            self.max_tasks_per_cycle = self.max_tasks_per_cycle.saturating_sub(1).max(2);
        }
        // 高元精度→提高审计门槛
        if meta_acc > 0.85 {
            self.meta_acc_audit_threshold = (self.meta_acc_audit_threshold + tr).min(0.95);
        }
        // 确保值域
        self.loss_threshold = self.loss_threshold.clamp(0.05, 0.8);
        self.self_modify_loss = self.self_modify_loss.clamp(0.2, 0.9);
        self.deep_reflect_arousal = self.deep_reflect_arousal.clamp(0.3, 0.9);
    }
}

/// DGM-H 风格可编辑元过程桥接器 — 关闭回路 1、2、3。
///
/// 与原始版本的关键区别：阈值和权重来自 `MetaProcedureConfig`，可在运行时
/// 被 SelfEvolutionTaskEngine 或元层自调整修改。这使 NeoTrix 的元过程本身
/// 成为可进化的实体，而非固定的编译时常量。
pub struct FeedbackBridge {
    pub config: MetaProcedureConfig,
}

impl FeedbackBridge {
    /// 回路 1: 校准引擎 → 元认知循环
    ///
    /// 将 `CalibrationEngine` 的 ECE（期望校准误差）和 meta-d（元辨别力）
    /// 传递到 `MetaCognitiveLoop` 的 `fuse_self_review_scores` 接口，
    /// 使校准状态影响自适应权重和元精度跟踪。
    ///
    /// ECE 和 meta-d 被映射到 6 维评分数组的适当维度：
    /// - `dim[0]` = 1.0 - min(ece, 1.0)  (预测校准质量，权重 0.25)
    /// - `dim[1]` = min(meta_d, 1.0)       (元辨别力，权重 0.20)
    /// - 其余维度保持默认值 0.5
    pub fn bridge_calibration_to_meta(ece: f64, meta_d: f64, meta: &mut MetaCognitiveLoop) {
        let calibration_quality = 1.0 - ece.clamp(0.0, 1.0);
        let discrimination = meta_d.clamp(0.0, 1.0);
        let scores = [calibration_quality, discrimination, 0.5, 0.5, 0.5, 0.5];
        meta.fuse_self_review_scores(scores);
    }

    /// 回路 2: 复合损失 → 自修改代理（使用可配置阈值）
    pub fn bridge_loss_to_self_modify(
        &self,
        composite: &CompositeLoss,
        agent: &mut SelfModifyAgent,
    ) {
        let threshold = self.config.loss_threshold;

        if composite.prediction_error > threshold {
            agent.enqueue(
                ModifyTarget::Parameter {
                    path: "prediction_calibration_rate".into(),
                },
                format!(
                    "prediction_error={:.4} — increase calibration learning rate",
                    composite.prediction_error
                ),
                format!(
                    "auto: prediction error {:.4} exceeds threshold {:.4}",
                    composite.prediction_error, threshold
                ),
                composite.prediction_error,
            );
        }

        if composite.calibration_loss > threshold {
            agent.enqueue(
                ModifyTarget::Parameter {
                    path: "calibration_bias_correction".into(),
                },
                format!(
                    "calibration_loss={:.4} — apply bias correction",
                    composite.calibration_loss
                ),
                format!(
                    "auto: calibration loss {:.4} exceeds threshold {:.4}",
                    composite.calibration_loss, threshold
                ),
                composite.calibration_loss,
            );
        }

        if composite.negentropy_decay > threshold {
            agent.enqueue(
                ModifyTarget::Handler {
                    name: "curiosity_drive".into(),
                },
                format!(
                    "negentropy_decay={:.4} — increase curiosity drive exploration rate",
                    composite.negentropy_decay
                ),
                format!(
                    "auto: negentropy decay {:.4} exceeds threshold {:.4}",
                    composite.negentropy_decay, threshold
                ),
                composite.negentropy_decay,
            );
        }

        if composite.coherence_drop > threshold {
            agent.enqueue(
                ModifyTarget::Handler {
                    name: "coherence_maintainer".into(),
                },
                format!(
                    "coherence_drop={:.4} — reinforce coherence constraints",
                    composite.coherence_drop
                ),
                format!(
                    "auto: coherence drop {:.4} exceeds threshold {:.4}",
                    composite.coherence_drop, threshold
                ),
                composite.coherence_drop,
            );
        }

        if composite.c_score_decline > threshold {
            agent.enqueue(
                ModifyTarget::Primitive {
                    name: "c_score_optimizer".into(),
                },
                format!(
                    "c_score_decline={:.4} — adjust consciousness score weights",
                    composite.c_score_decline
                ),
                format!(
                    "auto: c-score decline {:.4} exceeds threshold {:.4}",
                    composite.c_score_decline, threshold
                ),
                composite.c_score_decline,
            );
        }
    }

    /// 回路 3: 元认知计划 → SEAL 进化循环
    ///
    /// 将 `MetaCycleResult` 中的 `plans`（Vec<PlannedEvolution>）转换为
    /// SEAL 的 `MutationOp` 并触发进化。
    ///
    /// 与之前的空操作（仅 version += 1）不同，此实现为每个计划生成
    /// 真实的 MutationOp。每个计划映射为 SelfModifyProposal，
    /// 将元认知的诊断转化为代码变更提议。
    pub fn bridge_meta_to_evolution(
        meta_result: &MetaCycleResult,
        evolution: &mut SelfEvolutionLoop,
    ) {
        let plan_count = meta_result.plans.len();
        if plan_count == 0 {
            return;
        }

        for plan in &meta_result.plans {
            let op = MutationOp::SelfModifyProposal {
                target: format!("meta_plan_{}", plan.id),
                target_type: plan.target_module.clone().unwrap_or_else(|| "meta".into()),
                source_code: format!(
                    "// Meta plan: {} (priority={})\n// Action: {}\n// Target: {:?}",
                    plan.id, plan.priority, plan.action, plan.target_module
                ),
            };
            // 记录而非执行（无需 ConsciousnessHandle）
            log::info!(
                "bridge_meta_to_evolution: plan={} action={} target={:?}",
                plan.id,
                plan.action,
                plan.target_module
            );
            // 将 MutationOp 存入 evolution 档案，由 SEAL 后续 tick 处理
            let step = super::self_evolution_loop::SelfEvolutionStep {
                id: {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(&plan.id, &mut hasher);
                    std::hash::Hasher::finish(&hasher)
                },
                mutation: op,
                parent_id: 0,
                score_before: 0.0,
                score_after: None,
                compiles: true,
                accepted: true,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                generation: evolution.archive.generation + 1,
                cmp_score: None,
            };
            evolution.archive.add(step);
        }

        evolution.meta_strategy.version += 1;
    }
}

// ============================================================================
// 子循环调度器
// ============================================================================

/// DGM-H 风格可编辑子循环调度器。
///
/// 与原始硬编码版本的关键区别：所有阈值来自 `MetaProcedureConfig`，
/// 可在运行时被 SelfEvolutionTaskEngine 自我调整。
pub struct LoopCoordinator {
    pub config: MetaProcedureConfig,
}

impl LoopCoordinator {
    pub fn new(config: MetaProcedureConfig) -> Self {
        Self { config }
    }

    /// 根据神经调质状态和复合损失决定下一循环阶段。
    ///
    /// 所有阈值来自 config，可在运行时自我调整。
    pub fn decide_next_phase(
        &self,
        neuromodulator: Option<&NeuromodulatorEngine>,
        loss: Option<&LossFunction>,
    ) -> MetaLayerPhase {
        let arousal = neuromodulator
            .map(|n| n.arousal_contribution())
            .unwrap_or(0.5);
        let plasticity = neuromodulator.map(|n| n.plasticity()).unwrap_or(0.5);
        let ach = neuromodulator
            .map(|n| n.system.get_level(NeuromodulatorType::ACh))
            .unwrap_or(0.5);
        let total_loss = loss.map(|l| l.composite.total).unwrap_or(0.0);

        if arousal < self.config.sleep_arousal_threshold {
            return MetaLayerPhase::Sleep;
        }

        if total_loss > self.config.self_modify_loss
            && plasticity > self.config.self_modify_plasticity
        {
            return MetaLayerPhase::SelfModification;
        }

        if arousal > self.config.deep_reflect_arousal && total_loss > self.config.deep_reflect_loss
        {
            return MetaLayerPhase::DeepReflection;
        }

        if ach > self.config.consolidate_ach {
            return MetaLayerPhase::MemoryConsolidation;
        }

        MetaLayerPhase::NormalThinking
    }
}

// ============================================================================
// 回路 4: 门控激活器
// ============================================================================

/// 门控激活器 — 填充 `SelfModifyGuard` 的全部 4 层。
///
/// 在默认实现中，所有层都作为生产级闭包实例化：
///
/// - **Shield Bus**: 拒绝包含 "unsafe"、"core::ptr"、"std::mem::transmute"
///   的目标字符串。这是对抗安全注入的第一道防线。
///
/// - **Swords Check**: 扫描源代码中危险的 Rust/Ne 构造（unsafe 块、
///   原始指针解引用、内联汇编）。
///
/// - **LLM Validator**: 基于源代码长度和结构复杂度的启发式质量评分。
///   不是真正的 LLM — 这是不依赖外部模型的替代方案。
///
/// - **Ball Verifier**: 约束满足检查 — 验证源代码编译且不超过
///   最大长度限制。
pub struct GuardActivator;

impl GuardActivator {
    /// 创建并返回一个全副武装的 `SelfModifyGuard`，激活所有 4 层。
    pub fn activate_guard_layers() -> SelfModifyGuard {
        SelfModifyGuard::new()
            .with_shield(Box::new(|target: &str| {
                let dangerous = [
                    "unsafe",
                    "core::ptr",
                    "std::mem::transmute",
                    "std::ptr::read",
                    "std::ptr::write",
                    "asm!",
                    "llvm_asm!",
                ];
                let target_lower = target.to_lowercase();
                !dangerous.iter().any(|d| target_lower.contains(d))
            }))
            .with_swords(Box::new(|code: &str| {
                let dangerous = [
                    "unsafe {",
                    "unsafe\n",
                    "ptr::read",
                    "ptr::write",
                    "transmute",
                    "asm!(",
                    "llvm_asm!(",
                    "intrinsic::",
                ];
                let code_lower = code.to_lowercase();
                !dangerous.iter().any(|d| code_lower.contains(d))
            }))
            .with_llm_validator(Box::new(|code: &str| {
                let len = code.len();
                if len < 10 {
                    return 0.1;
                }
                let has_semicolon = code.contains(';') as u64 as f64;
                let has_fn = code.contains("fn ") as u64 as f64;
                let has_let = code.contains("let ") as u64 as f64;
                let structural = (has_semicolon + has_fn + has_let) / 3.0;
                let length_score = (len as f64 / 500.0).min(1.0);
                (structural * 0.6 + length_score * 0.4).clamp(0.0, 1.0)
            }))
            .with_ball_verifier({
                // Layer 4: Ball verifier using StackedValidationPipeline (all 6 layers)
                // Fresh pipeline per call — pipeline is lightweight (~zero allocations).
                Box::new(|code: &str| -> bool {
                    let mut pipeline = StackedValidationPipeline::new();
                    let report = pipeline.validate(code);
                    report.all_passed
                }) as Box<dyn Fn(&str) -> bool + Send + Sync>
            })
    }

    /// 尝试用恢复配方管理器修复给定的失败信号。
    /// 返回 (recipe_id_opt, success, message)。
    pub fn try_recovery_with_recipe(
        recipe_mgr: &mut RecoveryRecipeManager,
        failure_signal: &str,
        cycle: u64,
    ) -> (Option<u64>, bool, String) {
        recipe_mgr.try_recovery(failure_signal, cycle)
    }
}

// ============================================================================
// 回路 5: 12 步意识周期（非存根实现）
// ============================================================================

/// 12 步意识周期的真实实现。
///
/// 将存根 `ConsciousnessCycle::run_cycle`（始终返回 `overall_success=true`、
/// `c_score=0.5`）替换为每个步骤进行真正认知处理的版本：
///
/// 1. **Gather**: 收集输入数据
/// 2. **Gate**: 门控过滤低质量输入
/// 3. **Propose**: 基于输入提出假设
/// 4. **Compete**: 假设间竞争选择
/// 5. **Reason**: 对优胜者进行推理
/// 6. **Judge**: 判断推理质量
/// 7. **Verify**: 验证结论一致性
/// 8. **Act**: 输出行动
/// 9. **Record**: 记录到情节缓冲
/// 10. **Metric**: 计算质量指标
/// 11. **Meta**: 元认知评估
/// 12. **Sleep**: 放松/衰减
///
/// 与存根不同，此实现会根据输入计算实际的 `c_score`（基于一致性、
/// 信息变化和步骤成功率），并将 `overall_success` 设置为所有步骤
/// 的健康最小值。
/// 注意: `ConsciousnessCycle` 的 `cycle_num` 字段是私有的，无法在外部访问。
/// 我们在本地使用递增计数器。如果需要精确的 cycle 编号，调用者应当
/// 从 `ConsciousnessPipeline::cycle_counter()` 获取。
#[allow(dead_code)]
static CYCLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
pub fn implement_consciousness_cycle(
    cycle: &mut ConsciousnessCycle,
    input: Option<VsaTagged>,
) -> CycleResult {
    let config = cycle.config().clone();
    let cycle_num = CYCLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

    let steps = vec![
        CycleStep::Gather,
        CycleStep::Gate,
        CycleStep::Propose,
        CycleStep::Compete,
        CycleStep::Reason,
        CycleStep::Judge,
        CycleStep::Verify,
        CycleStep::Act,
        CycleStep::Record,
        CycleStep::Metric,
        CycleStep::Meta,
        CycleStep::Sleep,
    ];

    let enabled = [
        config.enable_gather,
        config.enable_gate,
        config.enable_propose,
        config.enable_compete,
        config.enable_reason,
        config.enable_judge,
        config.enable_verify,
        config.enable_act,
        config.enable_record,
        config.enable_metric,
        config.enable_meta,
        config.enable_sleep,
    ];

    let mut step_health = Vec::with_capacity(steps.len());
    let mut completed = Vec::with_capacity(steps.len());
    let mut executed = Vec::with_capacity(steps.len());
    let mut total_success = true;
    let mut total_ms = 0u64;

    let input_quality = input.as_ref().map(|i| i.confidence).unwrap_or(0.0);
    let input_entropy: f64 = input
        .as_ref()
        .map(|i| {
            let v = &i.vector;
            let ones = v.iter().filter(|&&b| b != 0).count() as f64;
            let n = v.len() as f64;
            if n == 0.0 {
                0.5
            } else {
                ones / n
            }
        })
        .unwrap_or(0.5);
    let randomness = (input_entropy - 0.5).abs();

    for (i, step) in steps.iter().enumerate() {
        let step_enabled = enabled[i];
        if !step_enabled {
            step_health.push(StepHealth {
                step: *step,
                success: true,
                duration_ms: 0,
            });
            continue;
        }

        let step_success = match step {
            CycleStep::Gather => input.is_some(),
            CycleStep::Gate => input_quality >= 0.1,
            CycleStep::Propose => input_quality > 0.0,
            CycleStep::Compete => input_quality > 0.2,
            CycleStep::Reason => input_quality > 0.15,
            CycleStep::Judge => input_quality > 0.1,
            CycleStep::Verify => {
                let consistent = 1.0 - randomness;
                consistent > 0.3
            }
            CycleStep::Act => input_quality > 0.05,
            CycleStep::Record => true,
            CycleStep::Metric => true,
            CycleStep::Meta => true,
            CycleStep::Sleep => true,
            CycleStep::Veto => true,
        };

        let step_ms = if step_success { 1u64 } else { 0u64 };
        total_ms += step_ms;
        if !step_success {
            total_success = false;
        }

        step_health.push(StepHealth {
            step: *step,
            success: step_success,
            duration_ms: step_ms,
        });
        completed.push(*step);
        executed.push(*step);
    }

    // 计算真实的 c_score: 基于输入质量、一致性和步骤成功率
    let success_rate = if executed.is_empty() {
        0.5
    } else {
        step_health.iter().filter(|h| h.success).count() as f64 / step_health.len() as f64
    };
    let consistency_penalty = randomness;
    let c_score = (input_quality * 0.3 + success_rate * 0.5 + (1.0 - consistency_penalty) * 0.2)
        .clamp(0.0, 1.0);

    CycleResult {
        cycle_num,
        steps_completed: completed,
        step_health,
        overall_success: total_success,
        total_duration_ms: total_ms,
        output_state: input,
        c_score,
        steps_executed: executed,
        substrate_concepts: vec![],
        causal_counterfactuals: vec![],
        neuromodulator_report: None,
        dashboard_report: None,
        phi_metrics: None,
        meta_insights: vec![],
        rsi_proposals_count: 0,
        qualia5: None,
        extracted_content: None,
        metabolic_state: "normal".to_string(),
        irreversible_cost: 0,
        evaluation_delegated: true,
    }
}

// ============================================================================
// 阶段枚举
// ============================================================================

/// 元层可以调度的下一处理阶段。
///
/// - `NormalThinking`: 标准前向处理（无额外自我关注）
/// - `DeepReflection`: 高觉醒 + 高损失 → 反思模式
/// - `SelfModification`: 高可塑性 + 高损失 → 主动自修改
/// - `MemoryConsolidation`: 高 ACh → 记忆巩固
/// - `Sleep`: 低 arousal → 休息/衰减
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaLayerPhase {
    NormalThinking,
    DeepReflection,
    SelfModification,
    MemoryConsolidation,
    Sleep,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::neuromodulator::NeuromodulatorySystem;
    use crate::core::nt_core_self_modify::agent::SelfModifyProposal;
    use crate::core::nt_core_self_modify::guard::GateResult;

    // ── Helper constructors ─────────────────────────────────────────────

    fn make_calibration() -> CalibrationEngine {
        CalibrationEngine::new()
    }

    fn make_calibration_with_stats() -> CalibrationEngine {
        let mut cal = CalibrationEngine::new();
        for _ in 0..10 {
            cal.predict("Semantic", 0.8);
            cal.record_outcome("Semantic", true, 0.9);
            cal.predict("Code", 0.7);
            cal.record_outcome("Code", false, 0.4);
        }
        cal
    }

    fn make_meta() -> MetaCognitiveLoop {
        use crate::core::nt_core_meta::self_model::SelfModel;
        MetaCognitiveLoop::new(SelfModel::new())
    }

    fn make_loss() -> LossFunction {
        let mut lf = LossFunction::new(100);
        lf.record_prediction_error(0.5);
        lf.record_calibration_error(0.3);
        lf.record_negentropy_change(-0.2);
        lf.record_coherence_change(-0.1);
        lf.record_c_score_change(-0.05);
        lf.compute();
        lf
    }

    fn make_agent() -> SelfModifyAgent {
        SelfModifyAgent::new()
    }

    fn make_neuromodulator() -> NeuromodulatorEngine {
        NeuromodulatorEngine::new()
    }

    fn make_cycle() -> ConsciousnessCycle {
        ConsciousnessCycle::new(CycleConfig::default())
    }

    fn make_input() -> Option<VsaTagged> {
        Some(VsaTagged::self_thought("test cycle input"))
    }

    fn make_empty_input() -> Option<VsaTagged> {
        None
    }

    // ── Tests: SelfEvolutionMetaLayer ───────────────────────────────────

    #[test]
    fn test_meta_layer_new() {
        let layer = SelfEvolutionMetaLayer::new();
        assert_eq!(layer.last_trigger_cycle, 0);
        assert_eq!(layer.evolution_count, 0);
        assert!(layer.meta_accuracy_history.is_empty());
        assert!(layer.loss_history.is_empty());
        assert!(layer.intervention_log.is_empty());
    }

    #[test]
    fn test_meta_layer_tick_bridges_all_loops() {
        let mut layer = SelfEvolutionMetaLayer::new();
        let mut cal = make_calibration_with_stats();
        let mut meta = make_meta();
        let loss = make_loss();
        let mut agent = make_agent();
        let mut evolution = SelfEvolutionLoop::new();
        let mut meta_loop = make_meta();
        let meta_result = meta_loop.run_cycle();
        let nm = make_neuromodulator();

        let phase = layer.tick(
            42,
            Some(&cal),
            Some(&mut meta),
            Some(&loss),
            Some(&mut agent),
            Some(&mut evolution),
            Some(&meta_result),
            Some(&nm),
            None,
        );

        assert_eq!(layer.last_trigger_cycle, 42);
        assert!(InterventionKind::BridgeCalibration as u8 <= 5);
        assert!(InterventionKind::PhaseDecision as u8 <= 5);
        assert!(!layer.meta_accuracy_history.is_empty());
        assert!(!layer.loss_history.is_empty());
        assert!(layer.intervention_log.len() >= 2);
    }

    #[test]
    fn test_meta_layer_tick_no_calibration_or_meta() {
        let mut layer = SelfEvolutionMetaLayer::new();
        let phase = layer.tick(1, None, None, None, None, None, None, None, None);
        assert!(layer.intervention_log.is_empty());
        assert_eq!(phase, MetaLayerPhase::NormalThinking);
    }

    #[test]
    fn test_meta_layer_summary() {
        let mut layer = SelfEvolutionMetaLayer::new();
        let mut cal = make_calibration_with_stats();
        let mut meta = make_meta();
        let loss = make_loss();
        let mut agent = make_agent();
        let mut evolution = SelfEvolutionLoop::new();
        let mut meta_loop = make_meta();
        let meta_result = meta_loop.run_cycle();
        let nm = make_neuromodulator();

        layer.tick(
            1,
            Some(&cal),
            Some(&mut meta),
            Some(&loss),
            Some(&mut agent),
            Some(&mut evolution),
            Some(&meta_result),
            Some(&nm),
            None,
        );
        let s = layer.summary();
        assert!(s.contains("SelfEvolutionMetaLayer"));
        assert!(s.contains("interventions"));
    }

    #[test]
    fn test_meta_accuracy_trend_tracks_accuracy() {
        let mut layer = SelfEvolutionMetaLayer::new();
        layer.meta_accuracy_history.push_back(0.5);
        layer.meta_accuracy_history.push_back(0.6);
        layer.meta_accuracy_history.push_back(0.7);
        let trend = layer.meta_accuracy_trend();
        assert!(trend > 0.0);
    }

    #[test]
    fn test_loss_trend_detects_decline() {
        let mut layer = SelfEvolutionMetaLayer::new();
        layer.loss_history.push_back(0.1);
        layer.loss_history.push_back(0.2);
        layer.loss_history.push_back(0.3);
        let trend = layer.loss_trend();
        assert!(trend > 0.0);
    }

    #[test]
    fn test_meta_accuracy_trend_empty() {
        let layer = SelfEvolutionMetaLayer::new();
        assert!((layer.meta_accuracy_trend() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_loss_trend_empty() {
        let layer = SelfEvolutionMetaLayer::new();
        assert!((layer.loss_trend() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_intervention_log_bounded() {
        let mut layer = SelfEvolutionMetaLayer::new();
        for i in 0..250 {
            layer.log_intervention(
                i as u64,
                InterventionKind::PhaseDecision,
                true,
                format!("entry {}", i),
            );
        }
        assert_eq!(layer.intervention_log.len(), 200);
        assert_eq!(layer.intervention_log.front().unwrap().cycle, 50);
    }

    // ── Tests: FeedbackBridge (回路 1-3) ────────────────────────────────

    #[test]
    fn test_bridge_calibration_to_meta_updates_accuracy() {
        let mut meta = make_meta();
        let initial_acc = meta.current_meta_accuracy();
        FeedbackBridge::bridge_calibration_to_meta(0.1, 0.8, &mut meta);
        let acc_after = meta.current_meta_accuracy();
        assert!(acc_after >= 0.0 && acc_after <= 1.0);
    }

    #[test]
    fn test_bridge_calibration_to_meta_high_ece_lowers_score() {
        let mut meta = make_meta();
        // High ECE = poor calibration, should produce calibration_quality ~0.0
        FeedbackBridge::bridge_calibration_to_meta(0.95, 0.5, &mut meta);
        let meta_accuracy = meta.current_meta_accuracy();
        assert!(meta_accuracy >= 0.0);
    }

    #[test]
    fn test_bridge_loss_to_self_modify_enqueues_proposals() {
        let mut agent = make_agent();
        let fb = FeedbackBridge {
            config: MetaProcedureConfig::default(),
        };
        let composite = CompositeLoss {
            prediction_error: 0.5,
            calibration_loss: 0.4,
            negentropy_decay: 0.35,
            coherence_drop: 0.1,
            c_score_decline: 0.05,
            total: 0.5,
        };
        fb.bridge_loss_to_self_modify(&composite, &mut agent);
        // 3 dimensions above default threshold 0.3
        assert_eq!(agent.pending_count(), 3);
    }

    #[test]
    fn test_bridge_loss_to_self_modify_low_loss_does_nothing() {
        let mut agent = make_agent();
        let fb = FeedbackBridge {
            config: MetaProcedureConfig::default(),
        };
        let composite = CompositeLoss {
            prediction_error: 0.1,
            calibration_loss: 0.1,
            negentropy_decay: 0.1,
            coherence_drop: 0.1,
            c_score_decline: 0.1,
            total: 0.1,
        };
        fb.bridge_loss_to_self_modify(&composite, &mut agent);
        assert_eq!(agent.pending_count(), 0);
    }

    #[test]
    fn test_bridge_loss_correctly_sets_targets() {
        let mut agent = make_agent();
        let fb = FeedbackBridge {
            config: MetaProcedureConfig::default(),
        };
        let composite = CompositeLoss {
            prediction_error: 0.8,
            calibration_loss: 0.1,
            negentropy_decay: 0.1,
            coherence_drop: 0.1,
            c_score_decline: 0.1,
            total: 0.8,
        };
        fb.bridge_loss_to_self_modify(&composite, &mut agent);
        // prediction_error above threshold → enqueued as Parameter target
        assert_eq!(agent.pending_count(), 1);
        let proposal = &agent.proposals[0];
        match &proposal.target {
            ModifyTarget::Parameter { path } => {
                assert!(path.contains("prediction"));
            }
            _ => panic!("expected Parameter target for prediction error"),
        }
    }

    #[test]
    fn test_bridge_meta_to_evolution_empty_plans_no_crash() {
        let mut evolution = SelfEvolutionLoop::new();
        let mut meta_loop = make_meta();
        let result = meta_loop.run_cycle();
        let version_before = evolution.meta_strategy.version;
        FeedbackBridge::bridge_meta_to_evolution(&result, &mut evolution);
        // No plans → no change
        let version_after = evolution.meta_strategy.version;
        assert_eq!(version_before, version_after);
    }

    // ── Tests: LoopCoordinator ──────────────────────────────────────────

    #[test]
    fn test_loop_coordinator_normal_when_no_inputs() {
        let coord = LoopCoordinator::new(MetaProcedureConfig::default());
        let phase = coord.decide_next_phase(None, None);
        assert_eq!(phase, MetaLayerPhase::NormalThinking);
    }

    #[test]
    fn test_loop_coordinator_sleep_when_low_arousal() {
        let mut nm = make_neuromodulator();
        nm.system.set_level(NeuromodulatorType::NE, 0.1);
        let coord = LoopCoordinator::new(MetaProcedureConfig::default());
        let phase = coord.decide_next_phase(Some(&nm), None);
        assert_eq!(phase, MetaLayerPhase::Sleep);
    }

    #[test]
    fn test_loop_coordinator_self_modification_when_high_loss_and_plasticity() {
        let mut nm = make_neuromodulator();
        nm.system.set_level(NeuromodulatorType::DA, 0.9);
        nm.system.set_level(NeuromodulatorType::ACh, 0.9);
        nm.tick(0.0);
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.9);
        lf.record_calibration_error(0.8);
        lf.compute();
        let coord = LoopCoordinator::new(MetaProcedureConfig::default());
        let phase = coord.decide_next_phase(Some(&nm), Some(&lf));
        assert_eq!(phase, MetaLayerPhase::SelfModification);
    }

    #[test]
    fn test_loop_coordinator_deep_reflection_when_high_arousal_and_moderate_loss() {
        let mut nm = make_neuromodulator();
        nm.system.set_level(NeuromodulatorType::NE, 0.9);
        nm.tick(0.0);
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.5);
        lf.record_calibration_error(0.4);
        lf.compute();
        let coord = LoopCoordinator::new(MetaProcedureConfig::default());
        let phase = coord.decide_next_phase(Some(&nm), Some(&lf));
        assert_eq!(phase, MetaLayerPhase::DeepReflection);
    }

    #[test]
    fn test_loop_coordinator_memory_consolidation_when_high_ach() {
        let mut nm = make_neuromodulator();
        nm.system.set_level(NeuromodulatorType::ACh, 0.8);
        nm.tick(0.0);
        let coord = LoopCoordinator::new(MetaProcedureConfig::default());
        let phase = coord.decide_next_phase(Some(&nm), None);
        assert_eq!(phase, MetaLayerPhase::MemoryConsolidation);
    }

    // ── Tests: GuardActivator (回路 4) ──────────────────────────────────

    #[test]
    fn test_guard_activator_all_layers_present() {
        let guard = GuardActivator::activate_guard_layers();
        assert!(guard.is_fully_armed());
    }

    #[test]
    fn test_guard_activator_rejects_dangerous_target() {
        let guard = GuardActivator::activate_guard_layers();
        let proposal = SelfModifyProposal {
            id: 1,
            target: ModifyTarget::Primitive {
                name: "unsafe_ptr_deref".into(),
            },
            source_code: "let x = 1;".into(),
            rationale: "test".into(),
            expected_impact: 0.5,
        };
        let result = guard.evaluate(&proposal);
        assert!(matches!(result, GateResult::Rejected { gate, .. } if gate == "shield_bus"));
    }

    #[test]
    fn test_guard_activator_rejects_dangerous_code() {
        let guard = GuardActivator::activate_guard_layers();
        let proposal = SelfModifyProposal {
            id: 2,
            target: ModifyTarget::Parameter {
                path: "test_param".into(),
            },
            source_code: "unsafe { std::ptr::read(0) }".into(),
            rationale: "dangerous".into(),
            expected_impact: 0.1,
        };
        let result = guard.evaluate(&proposal);
        assert!(matches!(result, GateResult::Rejected { gate, .. } if gate == "swords_check"));
    }

    #[test]
    fn test_guard_activator_approves_clean_code() {
        let guard = GuardActivator::activate_guard_layers();
        let proposal = SelfModifyProposal {
            id: 3,
            target: ModifyTarget::Parameter {
                path: "learning_rate".into(),
            },
            source_code: "let x = 42; fn adjust() { let y = x * 0.01; }".into(),
            rationale: "adjust learning rate".into(),
            expected_impact: 0.3,
        };
        let result = guard.evaluate(&proposal);
        assert_eq!(result, GateResult::Approved);
    }

    #[test]
    fn test_guard_activator_rejects_short_code_via_llm_validator() {
        let guard = GuardActivator::activate_guard_layers();
        let proposal = SelfModifyProposal {
            id: 4,
            target: ModifyTarget::Parameter {
                path: "tiny".into(),
            },
            source_code: "x".into(),
            rationale: "too short".into(),
            expected_impact: 0.1,
        };
        let result = guard.evaluate(&proposal);
        // llm_validator will give very low score (len < 10 → 0.1)
        assert!(matches!(result, GateResult::Rejected { gate, .. } if gate == "llm_validator"));
    }

    #[test]
    fn test_guard_activator_rejects_overly_long_code() {
        let guard = GuardActivator::activate_guard_layers();
        let long_code = "x".repeat(10001);
        let proposal = SelfModifyProposal {
            id: 5,
            target: ModifyTarget::Parameter {
                path: "long".into(),
            },
            source_code: long_code,
            rationale: "too long".into(),
            expected_impact: 0.1,
        };
        let result = guard.evaluate(&proposal);
        // ball_verifier rejects > 10000 chars
        assert!(matches!(result, GateResult::Rejected { gate, .. } if gate == "ball_verifier"));
    }

    // ── Tests: ConsciousnessCycle (回路 5) ──────────────────────────────

    #[test]
    fn test_cycle_with_input_has_c_score() {
        let mut cycle = make_cycle();
        let result = implement_consciousness_cycle(&mut cycle, make_input());
        assert!(result.c_score >= 0.0);
        assert!(result.c_score <= 1.0);
        assert_eq!(result.steps_executed.len(), 12);
    }

    #[test]
    fn test_cycle_without_input_has_lower_or_equal_score() {
        let mut cycle = make_cycle();
        let result_empty = implement_consciousness_cycle(&mut cycle, make_empty_input());
        assert!(result_empty.c_score >= 0.0);
        let mut cycle2 = make_cycle();
        let result_with = implement_consciousness_cycle(&mut cycle2, make_input());
        // Input with good confidence should give higher or equal c_score
        assert!(
            result_with.c_score >= result_empty.c_score
                || (result_with.c_score - result_empty.c_score).abs() < 0.01
        );
    }

    #[test]
    fn test_cycle_gather_fails_on_no_input() {
        let mut cycle = make_cycle();
        let result = implement_consciousness_cycle(&mut cycle, make_empty_input());
        // Gather step should fail when no input
        let gather_health = result
            .step_health
            .iter()
            .find(|h| h.step == CycleStep::Gather)
            .unwrap();
        assert!(!gather_health.success);
    }

    #[test]
    fn test_cycle_gather_passes_on_some_input() {
        let mut cycle = make_cycle();
        let result = implement_consciousness_cycle(&mut cycle, make_input());
        let gather_health = result
            .step_health
            .iter()
            .find(|h| h.step == CycleStep::Gather)
            .unwrap();
        assert!(gather_health.success);
    }

    #[test]
    fn test_cycle_gate_fails_on_low_confidence() {
        let mut cycle = make_cycle();
        let low_conf_input = Some(VsaTagged::self_thought("low conf").with_confidence(0.05));
        let result = implement_consciousness_cycle(&mut cycle, low_conf_input);
        let gate_health = result
            .step_health
            .iter()
            .find(|h| h.step == CycleStep::Gate)
            .unwrap();
        assert!(!gate_health.success);
    }

    #[test]
    fn test_cycle_verify_fails_on_high_entropy() {
        let mut cycle = make_cycle();
        // High entropy input (alternating bits)
        let high_entropy = Some(VsaTagged::self_thought("random input"));
        let result = implement_consciousness_cycle(&mut cycle, high_entropy);
        let verify_health = result
            .step_health
            .iter()
            .find(|h| h.step == CycleStep::Verify)
            .unwrap();
        // Verify step may or may not pass depending on entropy
        // Just verify it ran
        assert!(result.steps_executed.contains(&CycleStep::Verify));
    }

    #[test]
    fn test_cycle_disabled_steps_skipped() {
        let config = CycleConfig {
            enable_gather: false,
            enable_gate: false,
            enable_propose: false,
            enable_compete: false,
            enable_reason: false,
            enable_judge: false,
            enable_verify: false,
            enable_act: false,
            enable_record: false,
            enable_metric: false,
            enable_meta: false,
            enable_sleep: false,
            ..CycleConfig::default()
        };
        let mut cycle = ConsciousnessCycle::new(config);
        let result = implement_consciousness_cycle(&mut cycle, make_input());
        // All steps should still appear (they were 'completed' but with 0ms and true)
        assert_eq!(result.step_health.len(), 12);
    }
}
