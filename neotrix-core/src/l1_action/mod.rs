pub mod traits;
pub mod nt_act;
pub mod nt_io;
pub mod nt_media; // 统一媒体能力 (detect/router/streaming)

/// L1 → L0 error conversions (moved from l0_substrate to respect L0 ← L1 direction)
pub mod error_conversions;
pub mod nt_io_download; // 下载引擎 (自研，无外部依赖)

pub mod nt_memory_spatial; // moved from L2 (no L2 deps, spatial storage belongs in L1)
pub mod nt_action_facade; // L1 行动层唯一门面 (sole facade)
pub mod nt_conn; // migrated from cli/nt_conn
pub mod nt_router; // migrated from cli/nt_router

// 从 core/ 迁移的 L1 模块
pub mod nt_core_bank;
pub mod nt_core_graph;
pub mod nt_core_memory_budget;
pub mod nt_core_resource_pool;
pub mod nt_core_edit;
pub mod nt_core_embed;
pub mod nt_core_llm;
pub mod nt_core_task_dispatcher;
pub mod nt_crystal_llm_bridge; // L1 Provider → 晶体 NtLlmAsk（dispatcher 直调晶体闭环）
pub mod nt_model_cli; // 模型 CLI 问答桥 → 晶体 NtLlmAsk（外部命令仅为资源）
pub mod nt_free_pool; // 池免费模型智能调用 → 晶体 NtLlmAsk（轮转+故障转移+冷却）
pub mod nt_stdin_human; // 终端里的人 → 晶体 NtHumanChannel（窗口回话）
pub mod nt_dialogue_tui; // 对话终端 TUI 形态 → 晶体 NtHumanChannel（借鉴 Claude/opencode）
pub mod nt_tui_app; // v2 事件驱动会话应用（工作线程 + 实时渲染 + Esc 取消）
pub mod nt_core_harness;
pub mod nt_harness;
pub mod nt_core_simulate_engine;

// L1 基础设施层
pub mod nt_infra_tracing;
pub mod nt_infra_breaker;
pub mod nt_infra_semantic_router; // moved from L2 (no L2 deps, pure L1 infrastructure)
pub mod nt_infra_agent_card;
pub mod nt_infra_scatter_gather;
pub mod nt_infra_persistence;
pub mod nt_infra_learning;
pub mod nt_infra_integration;
/// AI Infrastructure: isolated environments, fast boot, OCI runtime (integrated from arcboxlabs/arcbox)
pub mod nt_infra_ai;

// ============================================================================
// Absorbed — 从 neotrix-sim 吸收
// ============================================================================
/// Graph Memory with PageRank — absorbed from neotrix-sim
pub mod nt_core_graph_memory;

// Backward-compatible re-export: nt_memory moved to L4 but binaries still reference l1_action::nt_memory
pub use crate::l4_emotion::nt_memory;
