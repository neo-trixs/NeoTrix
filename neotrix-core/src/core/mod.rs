// core/ — 跨层基础设施模块
//
// 这些模块是跨层通用的基础设施，不属于任何特定层级。
// 各层模块已迁移到对应的 lN_* 目录。

#![forbid(unsafe_code)]

// ═══════════════════════════════════════════════════════════════
// 基础设施 — 跨层通用组件
// ═══════════════════════════════════════════════════════════════
pub mod nt_core_di;             // 依赖注入
pub mod nt_core_axiom_tree;     // 公理树
pub mod nt_core_cap;            // 能力原语
pub mod nt_core_event;          // 事件系统
pub mod nt_core_traits;         // 核心 trait
pub mod nt_core_span;           // Span 追踪
pub mod nt_core_platform;       // 平台初始化
pub mod nt_core_telemetry;      // 遥测
pub mod nt_core_schema_watchdog; // Schema 看门狗
pub mod nt_core_ws;              // Workspace 管理
pub mod nt_core_cache;           // 语义缓存
pub mod nt_core_qtest;           // 自测引擎
pub mod nt_core_answer_engine;   // 应答引擎
