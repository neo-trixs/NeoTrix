//! # L7 — Capability (能力层)
//!
//! 能力注册、调度、成熟度进化、星脉通信协议。

pub mod capability;
pub mod memory; // 记忆层（孤儿接线 A51）
pub mod knowledge; // 知识图谱层（孤儿接线 A51）
pub mod nt_crt;
pub mod nt_core_cost_ladder; // 成本阶梯（孤儿接线 T45+2）
// ⛔ `multi_agent` **本轮不接线**，理由是实测证据（2026-09-30）：
// 尝试接线后立即暴露 **8 个编译错误**（3 个是真 bug：`E0382` move 后使用、
// `E0308` 类型不匹配、`E0277` unsized `str`），分布在
// `graph_orch/{scheduler,dag}.rs`、`coordinator/{load_balancer,monitor}.rs`、`crew.rs`。
// ⇒ 这 4,655 行**从未被编译过**，「历史绿灯」是假的；接线它等于
//    **立刻改动一批从未验证过的代码**，属另一个范围的工作。
// ⛔ 故本轮只保留取证，不接线。完整记录见
//    docs/architecture/MIRROR-BANK-2026-09-30.md §9.6。

pub mod nt_iit_phi;
pub mod nt_meta;
pub mod nt_forecast;
pub mod nt_state_substrate;

// 子目录模块
pub mod reasoning;
pub mod visual;
pub mod info_theory;
pub mod cuda;
pub mod other;

// 意识核心 (从 neotrix/ 迁移)
pub mod nt_consciousness_core;

// 保留的独立模块
pub mod context_assembly;
pub mod nt_core_intra_reflection;
pub mod nt_core_parallel;
pub mod seal;
pub mod persona_routing;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod awareness_monitor;

pub use awareness_monitor::SelfAwarenessMonitor;

// 从 L1 nt_io 迁移过来的模块
pub mod nt_io_awareness_core;
pub mod nt_io_context_mgmt;

// JSON-first declarative agent configuration (crewAI pattern)
pub mod nt_core_agent_config;

/// Trade Intelligence — market analysis, strategy recommendation engine.
pub mod trade_intelligence;

pub mod multi_agent;
