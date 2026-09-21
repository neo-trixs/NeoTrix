//! Stub domain plugins — one module per plugin.
//!
//! Re-exports preserve the original `stubs::{AgentPlugin, ...}` paths.

pub mod agent;
pub mod cli;
pub mod ext;
pub mod git;
pub mod plugin_meta;
pub mod security;
pub mod system;
pub mod tool;

mod common;

pub use agent::AgentPlugin;
pub use cli::CliPlugin;
pub use ext::ExtPlugin;
pub use git::GitPlugin;
pub use plugin_meta::PluginPlugin;
pub use security::SecurityPlugin;
pub use system::SystemPlugin;
pub use tool::ToolPlugin;
