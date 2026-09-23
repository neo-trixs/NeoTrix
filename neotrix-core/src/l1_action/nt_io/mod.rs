//! L1 Action Layer - IO Modules

// ============================================================================
// Provider — 模型提供商与适配层
// ============================================================================
pub mod nt_io_provider;
pub mod model_adapter;
pub mod model_routing;
pub mod universal_model;

// ============================================================================
// Protocol — 通信协议桥接
// ============================================================================
/// MCP Bridge — Model Context Protocol 桥接
pub mod nt_io_mcp_bridge;
/// ACP (Agent Client Protocol) — IDE 集成协议
pub mod acp;
/// MCP server — 将 NT 能力暴露为 MCP 工具
pub mod mcp_server;

// ============================================================================
// Agent — Agent 循环与编排
// ============================================================================
pub mod nt_io_agent_loop;
/// HiveAgentLoop — AgentLoop wrapper with Hive coordination (inbox/outbox/blackboard)
pub mod nt_io_hive_agent_loop;
pub mod nt_io_neocodex;
pub mod nt_io_standalone;

// ============================================================================
// Media — 多模态与 LLM 处理
// ============================================================================
pub mod nt_io_media;
/// Unified LLM Module — simplified trait + registry
pub mod nt_io_llm;
pub mod nt_io_multimodal_transform;
pub mod nt_io_digital_human;

// ============================================================================
// System — Web/插件/平台网关
// ============================================================================
pub mod nt_io_web;
pub mod nt_io_plugin;
pub mod platform_gateway;
pub mod nt_io_http_factory;
pub mod universal_browser;
/// 浏览器网络捕获（XHR URL＋响应体＋文本/坐标点击 JS＋登录壳检查，WSD 实战融合）
pub mod nt_io_browser_capture;
pub mod nt_io_proxy_server;
#[cfg(feature = "desktop")]
pub mod nt_io_desktop;

// ============================================================================
// Infrastructure — 会话/日志/缓存/推理
// ============================================================================
pub mod nt_io_session_recovery;
pub mod nt_io_logging;
pub mod nt_io_inference;
pub mod cache_compaction;
#[cfg(feature = "telemetry")]
pub mod nt_io_telemetry;
pub mod nt_io_contract;

// ============================================================================
// Messaging — 消息通道与通知
// ============================================================================
pub mod nt_io_messaging;
pub mod nt_io_notify;
pub mod nt_io_mention;
pub mod nt_io_avatar_channel;
pub mod nt_io_user_avatar;
pub mod nt_io_output_style;

// ============================================================================
// Extensions — Hook/沙箱/厂商技能/向导
// ============================================================================
// 上下文沙箱 — 工具输出压缩层
pub mod context_sandbox;
// 三拍子 Hook 系统 — 基于 Grok Build 模式
pub mod hooks;
// L3 厂商技能 (read-only capability branches)
pub mod l3_vendor_skills;
// 快速入门向导
pub mod quick_start_guide;
// 基于参考的生成
pub mod reference_generation;
pub mod nt_io_hotreload;
pub mod nt_io_agents_md;
pub mod nt_l1_error;

// ============================================================================
// Re-exports
// ============================================================================
pub use cache_compaction::{CacheCompactor, CompactionStrategy, CacheEntry, CompactionResult};

// 向后兼容别名
pub use platform_gateway::PlatformAdapter;
pub use model_adapter::ConsistencyAdapter;
pub use reference_generation::ReferenceVideoMode;

pub use nt_io_messaging::{
    MessagingRegistry, MessagingRouter, MessagingBridge,
    MessageTemplate, TemplateVariable, TemplateCategory,
    Channel, MessageDirection, Conversation, ConversationStatus,
    WhatsAppProvider, EmailProvider, ExtendedMessage, Attachment,
    trade_templates,
};
