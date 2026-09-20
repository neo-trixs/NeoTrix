pub mod chat;
pub mod context;
pub mod file;
pub mod kb;
pub mod llamacpp;
pub mod memory;
pub mod session;
pub mod stubs;
pub mod workflow;
pub mod world;

pub mod ai_orchestration;
pub mod folder_instructions;
pub mod im;
pub(crate) mod r#macro;
pub mod mcp_extension;
pub mod session_sync;
pub mod unified_surface;
pub mod voice_agent;

pub use chat::ChatPlugin;
pub use context::ContextPlugin;
pub use file::FilePlugin;
pub use kb::KbPlugin;
pub use llamacpp::LlamacppPlugin;
pub use memory::MemoryPlugin;
pub use session::SessionPlugin;
pub use stubs::{
    AgentPlugin, CliPlugin, ExtPlugin, GitPlugin, PluginPlugin, SecurityPlugin, SystemPlugin,
    ToolPlugin,
};
pub use workflow::WorkflowPluginImpl;
pub use world::WorldPlugin;

pub use ai_orchestration::AiOrchestrationPlugin;
pub use folder_instructions::FolderInstructionsPlugin;
pub use im::ImPlugin;
pub use mcp_extension::McpExtensionPlugin;
pub use session_sync::SessionSyncPlugin;
pub use unified_surface::UnifiedSurfacePlugin;
pub use voice_agent::VoiceAgentPlugin;

#[cfg(test)]
pub mod im_test;
