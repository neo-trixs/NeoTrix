//! L1 Action Layer - IO Modules

// LLM Provider system
pub mod nt_io_provider;

// Web server and API
pub mod nt_io_web;

// Plugin system
pub mod nt_io_plugin;

// Hot reload
pub mod nt_io_hotreload;

// ============================================================================
// Specialized IO modules
// ============================================================================

pub mod nt_io_agent_loop;

pub mod nt_io_agents_md;

pub mod nt_io_avatar_channel;

pub mod nt_io_awareness_core;

pub mod nt_io_digital_human;

pub mod nt_io_http_factory;

pub mod nt_io_logging;

pub mod nt_io_mention;

pub mod nt_io_multimodal_transform;

pub mod nt_io_neocodex;

pub mod nt_io_notify;

pub mod nt_io_output_style;

pub mod nt_io_proxy_server;

pub mod nt_io_session_recovery;

pub mod nt_io_standalone;

#[cfg(feature = "telemetry")]
pub mod nt_io_telemetry;

pub mod nt_io_user_avatar;

// 上下文沙箱 — 工具输出压缩层
pub mod context_sandbox;

pub mod nt_io_messaging;
pub mod nt_io_context_mgmt;
pub mod nt_l1_error;

// 三拍子 Hook 系统 — 基于 Grok Build 模式
pub mod hooks;

// ACP (Agent Client Protocol) — IDE 集成协议
pub mod acp;

// L3 vendor skill modules (read-only capability branches)
pub mod nt_io_ai_image_prompts;
pub mod nt_io_cozyclay;
pub mod nt_io_excalidraw;
pub mod nt_io_generative_media_skills;
pub mod nt_io_hermes_community;
pub mod nt_io_hermes_quota;
pub mod nt_io_pi_agent_desktop;
pub mod nt_io_promo_bgm;
pub mod nt_io_show_me;
pub mod nt_io_unslop;
pub mod nt_io_video_shotcraft;

// MCP server — 将 NT 能力暴露为 MCP 工具
pub mod mcp_server;

// 平台适配器模块 (Seedance2.0/Runtime 等多平台适配)
pub mod platform_adapter;

// 快速入门向导
pub mod quick_start_guide;

// 一致性控制接口
pub mod consistency_adapter;

// 参考生视频模式
pub mod reference_video_mode;

// 通用能力模块 (从漫剧专用重构为通用)
pub mod platform_gateway;     // 平台网关 (原 platform_adapter)
pub mod model_adapter;        // 模型适配器 (原 consistency_adapter)
pub mod reference_generation; // 基于参考的生成 (原 reference_video_mode)

// 多模型路由
pub mod model_routing;

// Cache-Aware Compaction — 基于缓存命中率的智能压缩
pub mod cache_compaction;

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

#[cfg(feature = "desktop")]
pub mod nt_io_desktop;
