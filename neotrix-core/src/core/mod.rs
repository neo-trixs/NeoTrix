// core/ — 晶体核心模块库
//
// 所有模块按层级组织，通过 pub use 桥接到各层。
// 模块物理位置保持在 core/，各层通过 re-export 暴露。
//
// 层级映射 (Unified Crystal Architecture):
//   L0 基质层   — error, hot_data
//   L1 行动层   — bank, graph, edit, embed, llm, task_dispatcher, ...
//   L2 感知层   — e8, hcube, sense, knowledge, vector_store
//   L3 具身层   — guard_chain
//   L4 情感层   — (nt_feel)
//   L5 认知层   — gwt, consciousness, reasoning, hex, gate, math, ...
//   L6 元认知层 — self, observer, absorb, scheduler, ...
//   基础设施     — event, traits, cache, di, ...

#![forbid(unsafe_code)]

// ═══════════════════════════════════════════════════════════════
// 已迁移到层级的模块 (通过 pub use 保持向后兼容)
// ═══════════════════════════════════════════════════════════════

// 从 l5_cognition 重导出 (已物理迁移的模块)
pub use crate::l5_cognition::nt_core::capability as l7_capability;
pub use crate::l5_cognition::nt_core_consciousness_core;
pub use crate::l5_cognition::nt_core_consciousness_tree;
pub use crate::l5_cognition::nt_core::nt_crt as nt_core_crt;
pub use crate::l5_cognition::nt_core::nt_forecast as nt_core_forecast;
pub use crate::l5_cognition::nt_core::nt_iit_phi as nt_core_iit_phi;
pub use crate::l5_cognition::nt_core::nt_meta as nt_core_meta;
pub use crate::l5_cognition::nt_core::nt_state_substrate as nt_core_state_substrate;
pub use crate::l5_cognition::nt_core::capability::types::CapabilityVector;
pub use crate::l5_cognition::nt_core::nt_crt::{CrtTimeScale, CrtPlan};

// 从 cli 重导出
pub use crate::cli::nt_conn as nt_core_conn;
pub use crate::cli::nt_router as nt_core_router;
pub use crate::cli::nt_subagent as nt_core_subagent;

// 从 neotrix 重导出
pub use crate::neotrix::nt_core_event_bus::EventBus;

// 层级重导出
pub use crate::l2_perception;
pub use crate::l3_embodiment as l1_body;

// ═══════════════════════════════════════════════════════════════
// L0 基质层 — 底层错误处理与热数据
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_error;
pub mod nt_core_hot_data;

// ═══════════════════════════════════════════════════════════════
// L1 行动层 — 工具/IO/记忆/任务调度
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_bank;           // 推理银行
pub mod nt_core_graph;          // 图结构
pub mod nt_core_memory_budget;  // 内存预算
pub mod nt_core_resource_pool;  // 资源池
pub mod nt_core_edit;           // 编辑操作
pub mod nt_core_embed;          // 嵌入
pub mod nt_core_llm;            // LLM 接口
pub mod nt_core_task_dispatcher; // 任务分发
pub mod nt_core_harness;        // 运行时外壳
pub mod nt_core_simulate_engine; // 模拟引擎

// ═══════════════════════════════════════════════════════════════
// L2 感知层 — 世界模型/知识/向量/E8/超几何
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_e8;             // E8 超几何
pub mod nt_core_e8_predictor;   // E8 预测器
pub mod nt_core_e8_vsa;         // E8-VSA 融合
pub mod nt_core_hcube;          // HyperCube 向量符号
pub mod nt_core_sense;          // 感官处理
pub mod nt_core_knowledge;      // 知识系统
pub mod nt_core_vector_store;   // 向量存储
pub mod nt_core_code_search;    // 代码搜索

// ═══════════════════════════════════════════════════════════════
// L3 具身层 — 安全/保护
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_guard_chain;    // 守护链

// ═══════════════════════════════════════════════════════════════
// L5 认知层 — 推理/意识/数学/策略
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_consciousness;  // 意识系统
pub mod nt_core_context;        // 上下文组装
pub mod nt_core_dispatch;       // 任务调度
pub mod nt_core_gwt;            // 全局工作空间理论
pub mod nt_core_echo_terminal;  // 回声终端
pub mod nt_core_reasoning;      // 推理引擎
pub mod nt_core_math;           // 数学工具
pub mod nt_core_hex;            // 推理六十四卦
pub mod nt_core_gate;           // 门控系统
pub mod nt_core_policy;         // E8 策略
pub mod nt_core_prm;            // PRM 推理
pub mod nt_core_cot_generator;  // CoT 生成器
pub mod nt_core_credit;         // 信用系统
pub mod nt_core_rule_memory;    // 规则记忆
pub mod nt_core_meaning;        // 语义系统
pub mod nt_core_paradigm;       // 范式系统
pub mod nt_core_aura;           // 氛围场
pub mod nt_core_walsh;          // Walsh 变换
pub mod nt_core_kron;           // Kronecker 积
pub mod nt_core_plan;           // 规划引擎
pub mod nt_core_kernel_types;   // 内核类型
pub mod nt_core_sae;            // 稀疏自编码器
pub mod nt_core_sae_bridge;     // SAE 桥接
pub mod nt_core_state;          // 状态系统
pub mod nt_core_narrative_types; // 叙事类型
pub mod nt_core_td;             // 时差学习
pub mod nt_core_trajectory_compress; // 轨迹压缩
pub mod nt_core_ttc;            // TTC 指标
pub mod nt_core_quantum_fusion; // 量子融合
pub mod nt_core_panic_recovery; // panic 恢复
pub mod nt_core_scoring_substrate; // 评分子基
pub mod nt_core_second_brain;   // 第二大脑
pub mod nt_core_orchestration_failure_taxonomy; // 编排故障分类
pub mod nt_core_cad_consciousness; // CAD 意识
pub mod nt_core_arch_diagram;   // 架构图
pub mod nt_core_arch_fitness;   // 架构适应度
pub mod nt_core_model_skills;   // 模型技能
pub mod nt_core_shared_types;   // 共享类型

// ═══════════════════════════════════════════════════════════════
// L6 元认知层 — 自我/观察/吸收/调度
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_self;           // 自我模型
pub mod nt_core_self_constitution; // 自我宪法
pub mod nt_core_aware;          // 意识感知
pub mod nt_core_observer;       // 观察者
pub mod nt_core_observer_error; // 观察者错误
pub mod nt_core_kb_primitives;  // KB 原语
pub mod nt_core_kb_types;       // KB 类型
pub mod nt_core_memory_asset;   // 记忆资产
pub mod nt_core_absorb;         // 吸收系统
pub mod nt_core_iter;           // 迭代器
pub mod nt_core_scheduler;      // 调度器
pub mod nt_core_self_review;    // 自我审查
pub mod nt_core_capability;     // 能力系统

// ═══════════════════════════════════════════════════════════════
// 基础设施 — 跨层通用组件
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_di;             // 依赖注入
pub mod nt_core_axiom_tree;     // 公理树
pub mod nt_core_cap;            // 能力原语
pub mod nt_core_event;          // 事件系统
pub mod nt_core_traits;         // 核心 trait
pub mod nt_core_ws;             // 工作空间
pub mod nt_core_cache;          // 缓存
pub mod nt_core_span;           // Span 追踪
pub mod nt_core_answer_engine;  // 答案引擎
pub mod nt_core_qtest;          // 快速测试
pub mod nt_core_platform;       // 平台初始化
pub mod nt_core_telemetry;      // 遥测
pub mod nt_core_schema_watchdog; // Schema 看门狗

// ═══════════════════════════════════════════════════════════════
// 类型重导出 — 供外部模块快速引用
// ═══════════════════════════════════════════════════════════════

// 共享类型
pub use nt_core_shared_types::*;

// 错误类型
pub use nt_core_error::NeoTrixError;

// 向量存储类型
pub use nt_core_vector_store::{DistanceMetric, VectorSearchResult, VectorRecord, create_default_store, create_store, StoreBackend};

// 感官类型
pub use nt_core_sense::{Sensor, SensorSample};

// 知识类型
pub use nt_core_knowledge::types::{KnowledgeSource, RewardSource, TaskType};
pub use nt_core_knowledge::SourceAccessTracker;

// SAE 类型
pub use nt_core_sae::{SaeFeature, SparseAutoencoder, SAE_INPUT_DIM};

// 推理类型
pub use nt_core_hex::{FullReasoningState, MetaState, strategy_matrix};
pub use nt_core_hex::ReasoningHexagram;
pub use nt_core_hex::optimal_starting_mode;
pub use nt_core_bank::bank::ReasoningBank;

// 策略类型
pub use nt_core_policy::E8Policy;
pub use nt_core_policy::E8TransitionLearner;

// GWT 类型
pub use nt_core_gwt::resonance::default_specialist_states;

// 工作空间
pub use nt_core_ws::WORKSPACE_MANAGER;

// 事件类型
pub use nt_core_event::CoreEvent;
