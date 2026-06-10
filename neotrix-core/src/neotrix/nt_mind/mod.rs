//! 统一推理大脑模块
//!
//! 拆分自 nt_mind.rs (882行 → 7个子模块)
//! 架构: CapabilityVector + ReasoningBrain + SelfIteratingBrain
//! 新增: SelfEvolver (S-06) - 外部信息自我进化
//!
//! 模块组织（6 域，中间模块提供层次化访问）:
//!   1. core_reasoning    — 核心推理类型、引擎、自迭代
//!   2. memory_knowledge  — 记忆、知识挖掘/引擎/持久化
//!   3. self_improvement  — 自我进化、元认知、创造
//!   4. connectivity      — HyperCube / GWT 桥接、监控
//!   5. specialized_tools — 垂直引擎（审查/目标/构建/归档）
//!   6. dev_tools         — 开发者工具（LSP 客户端）

// ============================================================================
// Domain module declarations (hierarchical organization layer)
// ============================================================================
pub mod core_reasoning;
pub mod evolution_seed;
pub mod memory_knowledge;
pub mod self_improvement;
pub mod connectivity;
pub mod specialized_tools;
pub mod dev_tools;

// ============================================================================
// Domain 1: 核心推理系统（Core Reasoning）
// ============================================================================
pub mod core;                    // CapabilityVector, KnowledgeSource, AbsorptionRecord
pub mod reasoning_engine;        // ReasoningEngine: LLM + Knowledge + Capability 统一入口
pub mod reasoning_types;         // ReasoningMethod, PerspectiveLens, ReasoningType, ReasoningTrace
pub mod self_edit;               // SelfEdit, MicroEdit, ToolCall, generate_self_edit()
pub mod self_iterating;          // ReasoningBrain, SelfIteratingBrain, EvaluationRecord
pub mod perception_evolution;    // Perception Evolution — adaptive sensing and attention
pub mod stats;                   // BrainStats, BrainReport, IterationResult
pub mod model_router;            // 智能模型分级路由 (T0-T4)
pub mod tier_prompts;            // Tier-aware system prompts based on ContextTier
pub mod lora;                    // LoRA 低秩适配 (S-12)
pub mod multi_brain;             // 多 ReasoningBrain 协同 (S-13)
pub mod bm25;                    // BM25 关键词索引 + RRF 融合检索（借鉴 agentmemory）
pub mod embedding;               // TEXT embedding 模块（EMB-1）

// ============================================================================
// Domain 2: 记忆与知识管理（Memory & Knowledge）
// ============================================================================
pub mod memory;                  // ReasoningBank 记忆机制
pub mod cortex_memory;           // 类人脑多维度记忆架构
pub mod knowledge_engine;        // 结构化知识引擎（文献搜索+持久化+关系网络）
pub mod knowledge_miner;         // 自动知识挖掘器
pub mod knowledge_chain;         // 知识链连接挖掘→验证→吸收→存储
pub mod web_miner;               // 统一网页知识挖掘 wiki/arXiv/GitHub/URL
pub mod knowledge_maturity;      // TENSA multi-fidelity maturity tracking (L-05)
pub mod impact_matrix;           // ImpactMatrix: 能力维度 → 任务类型影响权重
pub mod seal_algebra;            // SEALAlgebra: 验证 T_seal 的谱半径和收敛性
pub mod exploration_pipeline;    // Unified exploration pipeline — external knowledge absorption
pub mod goal_register;           // ExplicitGoalRegister: goal tracking + progress quantification
pub mod exploration_seeds;       // URL seed data extracted from exploration_pipeline
pub mod change_archive;          // Structured Archive + Delta Spec + 冲突检测
pub mod context_artifacts;       // Context Artifacts — index non-code knowledge alongside source code
pub mod export_import;           // v2.0: ReasoningBank 导出/导入（借鉴 MapCN）

// ============================================================================
// Domain 3: 自我改进与进化（Self-Improvement & Evolution）
// ============================================================================
pub mod self_evolver;            // SelfEvolver: 从外部链接自我进化 (S-06)
pub mod auto_crystallizer;       // AutoCrystallizer: 吸收知识自动结晶为 SkillCrystal
pub mod skill_evolution;         // SkillEvolver: ActionSequenceMiner + SkillDiagnoser + SkillRepairer
pub mod curiosity_drive;         // CuriosityDrive: 知识缺口→好奇心信号→主动探索
pub mod knowledge_aging;         // KnowledgeAging: 知识老化评分 + 自动重新扫描
pub mod stagnation;              // StagnationDetector: 防止无效循环
pub mod thinking_bridge;         // 硅基思维桥接: SiliconSelfModel → SEAL loop + skill crystallization
pub mod cognitive_map;           // LLM→NeoTrix 认知映射表
pub mod distillation;            // 经验蒸馏 + 对比反思
pub mod experiment;              // 实验设计引擎 (Route D - A/B 测试+假设框架)
pub mod causal_inventor;         // 因果发明引擎(跨域创新)
pub mod creation_engine;         // 造物引擎(从理论到实体工具)
pub mod dao_engine;              // 道引擎(从本源规则逆推万物)

// ============================================================================
// Domain 4: 连接与桥接（Connectivity & Bridges）
// ============================================================================
pub mod hypercube_bridge;                // Bridge: cortex_memory + knowledge_engine → core/hypercube
pub mod hypercube_attention_bridge;      // Attention → KnowledgeHyperCube 关联召回
pub mod consciousness_bridge;            // HC-06: GWT attention router ↔ SEAL loop
pub mod attention_router;                // AttentionRouter: GWT + HyperCube → ReasoningEngine
pub mod sleep;                           // SleepEngine: 离线递归记忆巩固 (arXiv:2605.26099v2)
pub mod panorama_pipeline;               // PanoramaPipeline: 超维度记忆知识库全景集成
pub mod bbrain_monitor;                  // B-Brain unified monitoring (P0-2)
pub mod element;                         // Plugin Element architecture (Phase 1)
pub mod code_graph;                      // CodeGraphEngine: 代码依赖图 + 影响分析（借鉴 GitNexus）
pub mod graph_types;                     // CodeGraph type definitions (NodeKind, EdgeKind, etc.)
pub mod graph_build;                     // CodeGraph parsing utilities
pub mod graph_analysis;                  // CodeGraph analysis methods (traverse, impact, etc.)
pub mod code_graph_executor;             // Tool call tracking + CountingExecutor for SEAL reward signals
pub mod wifi_sensing;                    // WiFi 感知引擎：空间特征 → WorldModel（借鉴 RuView）
pub mod cortex_types;                    // PredictiveCortex struct, types, constants
pub mod cortex_core;                     // PredictiveCortex core prediction logic
pub mod predictive_cortex;               // PredictiveCortex: constructor + self-repair + re-exports
pub mod meta_pattern_extractor;           // MetaPatternExtractor: ThinkingTrace → 元模式 → 意识进化

// ============================================================================
// Domain 5: 专用引擎与工具（Specialized Tools）
// ============================================================================
pub mod benchmark;               // E8→GWT→SelfIteration 管线基准测试
pub mod code_review;             // 代码审查引擎 (P2-9.3)
pub mod ux_review;               // UX 审查引擎 (Route E - 平行代码审查)
pub mod react_doctor;            // ReactDoctor: React 代码健康分析引擎
pub mod goal_loop;               // Goal Loop — 24/7 自主目标追求引擎 (Codex /goal + Ralph loop)
pub mod pipeline;                // Context Offloading + L1→L2→L3 渐进式记忆提取管线
pub mod build_context;
pub mod case_study;              // Case Study Writer (Problem→Process→Result)
pub mod stakeholder_comm;        // Stakeholder Communicator (3 audience types)
pub mod group_contracts;         // 多仓库 Group / Contract 系统
pub mod kronecker_cleanup;       // L-02: Kronecker-structured O(N log N) cleanup
pub mod open_source_benchmark;
#[cfg(test)]
pub mod context_integration_test;
pub mod side_git;                // Side-git world memory — independent git snapshot system
pub mod theory_of_mind;          // Theory of Mind — user mental model inference
pub mod tests;                   // 所有测试

// ============================================================================
// Domain 6: 开发者工具（Developer Tools）
// ============================================================================
pub mod lsp_client;
pub mod rule_engine;                     // 声明式路由规则引擎

// ============================================================================
// Orphaned modules — reintegrated from flat files
// ============================================================================
pub mod brain_event_bus;
pub mod clawbench;
pub mod clawbench_bench;
pub mod consciousness_reasoner;
pub mod content_extractor;
pub mod credential_manager;
pub mod deep_reflexion;
pub mod discovery_agent;
pub mod eval_monitor;
pub mod full_dimension_evolver;
pub mod gap_analyzer;
pub mod knowledge_absorber;
pub mod panoramic;
pub mod prediction_trainer;
pub mod reasoning_bench;
pub mod self_distillation;
pub mod session_context;
pub mod webapp_agent;
pub mod self_questioning;
pub mod meta_agent;
pub mod task_driver;

// Domain grouping sub-modules (reorganize flat 62 modules into 6 domains)
pub(crate) mod seal_core;
pub(crate) mod reason;
pub(crate) mod knowledge;
pub(crate) mod consciousness;
pub(crate) mod evolution;
pub(crate) mod infrastructure;

// ============================================================================
// Re-exports（按域分组）
// ============================================================================

// --- Core Reasoning ---
pub use core::{CapabilityVector, KnowledgeSource, AbsorptionRecord};
pub use self_edit::{SelfEdit, ToolCall};
pub use self_iterating::{ReasoningBrain, SelfIteratingBrain, EvaluationRecord, SelfIteration, EvoStats};
pub use self_iterating::brain_impl::{RLAlgorithm, WeightUpdateRecord};
pub use stats::{BrainStats, BrainReport, IterationResult};
pub use reasoning_engine::ReasoningEngine;
pub use reasoning_types::{ReasoningMethod, PerspectiveLens, ReasoningType, ReasoningTrace};

// --- Memory & Knowledge ---
pub use memory::{ReasoningBank, ReasoningMemory, ReasoningBankStats, MemoryDetailedStats, MemoryIterationResult};
pub use knowledge_miner::{KnowledgeMiner, MinedKnowledge, MinedRoundResult};
pub use knowledge_chain::{KnowledgeChain, KnowledgeChainPhase, KnowledgeChainStatus, ChainRunResult};
pub use web_miner::{WebKnowledgeMiner, WebMinedKnowledge, WebSourceType, WebMineResult};
pub use cortex_memory::{CortexMemory, MemoryTrace, DimensionTag, Modality, CortexStats, MemoryLayer, CmsConfig, CmsResult};
pub use export_import::ReasoningBankExporter;
pub use knowledge_engine::{KnowledgeEngine, KnowledgeEntry, KnowledgeRelation, RelationType, SourceType, LiteratureSearcher, KnowledgeEngineStats};
pub use change_archive::{ChangeArchive, DeltaChange, DeltaKind, ArchiveEntry, ConflictWarning, generate_delta_spec_report};

// --- Self-Improvement ---
pub use self_evolver::SelfEvolver;
pub use auto_crystallizer::AutoCrystallizer;
pub use skill_evolution::SkillEvolver;
pub use curiosity_drive::CuriosityDrive;
pub use knowledge_aging::{KnowledgeAging, KnowledgeFreshness, AgingReport};

// --- Connectivity ---
pub use consciousness_bridge::ConsciousnessBridge;
pub use attention_router::AttentionRouter;
pub use rule_engine::{RuleEngine, RuleAction, RulePattern, RoutingRule};
pub use stagnation::{StagnationDetector, StagnationSignal, StagnationStats};
pub use thinking_bridge::SkillBridge;
pub use bbrain_monitor::{BMonitor, BMonitorReport, AlertLevel};
pub use element::registry::ElementRegistry;
pub use panorama_pipeline::{PanoramaPipeline, PanoramaReport};

// --- Specialized Tools ---
pub use open_source_benchmark::{OpenSourceBenchmarker, BenchmarkReport};
pub use react_doctor::{ReactDoctorEngine, ReactHealthReport, ReactDiagnostic, ReactRuleCategory, RuleSeverity, SuppressionAnalysis};
pub use goal_loop::{GoalLoop, GoalState, GoalConfig, GoalTracker, GoalIterationRecord};
pub use theory_of_mind::{TheoryOfMind, MentalModel, InferredIntent};
