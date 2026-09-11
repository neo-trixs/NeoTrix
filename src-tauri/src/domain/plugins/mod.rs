pub mod session;
pub mod chat;
pub mod file;
pub mod kb;
pub mod llamacpp;
pub mod memory;
pub mod world;
pub mod workflow;
pub mod stubs;
pub mod context;

pub use session::SessionPlugin;
pub use chat::ChatPlugin;
pub use file::FilePlugin;
pub use kb::KbPlugin;
pub use llamacpp::LlamacppPlugin;
pub use memory::MemoryPlugin;
pub use world::WorldPlugin;
pub use workflow::WorkflowPluginImpl;
pub use context::ContextPlugin;
pub use stubs::{
    AgentPlugin, PluginPlugin,
    ToolPlugin, SystemPlugin, SecurityPlugin, ExtPlugin,
    GitPlugin, CliPlugin,
};
pub use chat::set_app_handle;
