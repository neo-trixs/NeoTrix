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

pub mod nt_io_messaging;
pub mod nt_io_context_mgmt;
pub use nt_io_messaging::{
    MessagingRegistry, MessagingRouter, MessagingBridge,
    MessageTemplate, TemplateVariable, TemplateCategory,
    Channel, MessageDirection, Conversation, ConversationStatus,
    WhatsAppProvider, EmailProvider, ExtendedMessage, Attachment,
    trade_templates,
};

#[cfg(feature = "desktop")]
pub mod nt_io_desktop;
