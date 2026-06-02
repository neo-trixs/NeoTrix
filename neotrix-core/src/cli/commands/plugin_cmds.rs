use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::plugin::PluginRegistry;
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct PluginCmd;
impl CliCommand for PluginCmd {
    fn name(&self) -> &str { "/plugin" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str {
        "Plugin management: /plugin list | /plugin load <path> | /plugin unload <name> | /plugin info <name>"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");
        match sub {
            "list" | "ls" => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let registry = PluginRegistry::new();
                let plugins = rt.block_on(registry.list());
                if plugins.is_empty() {
                    return CommandOutput::ok("No plugins registered. Use /plugin load <path> to load from a directory.");
                }
                let mut msg = format!("Registered plugins ({}):\n", plugins.len());
                for p in &plugins {
                    msg.push_str(&format!("  {} v{} [{}] — {}\n", p.name, p.version, p.status, p.source));
                }
                CommandOutput::ok(&msg)
            }
            "load" => {
                let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if path.is_empty() {
                    return CommandOutput::err("Usage: /plugin load <path>");
                }
                let dir = PathBuf::from(path);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let registry = PluginRegistry::new();
                match rt.block_on(registry.load_from_dir(&dir)) {
                    Ok(loaded) => {
                        let count: usize = loaded.len();
                        CommandOutput::ok(&format!("Scanned {}. Found {} plugin files (loading pending WASM/DynamicLib support).", path, count))
                    }
                    Err(e) => CommandOutput::err(&format!("Failed to load from directory: {}", e)),
                }
            }
            "unload" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /plugin unload <name>");
                }
                let rt = tokio::runtime::Runtime::new().unwrap();
                let registry = PluginRegistry::new();
                match rt.block_on(registry.unregister(name)) {
                    Ok(()) => CommandOutput::ok(&format!("Plugin '{}' unregistered.", name)),
                    Err(e) => CommandOutput::err(&format!("Failed to unregister '{}': {}", name, e)),
                }
            }
            "info" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /plugin info <name>");
                }
                let rt = tokio::runtime::Runtime::new().unwrap();
                let registry = PluginRegistry::new();
                let plugins = rt.block_on(registry.list());
                match plugins.iter().find(|p| p.name == name) {
                    Some(p) => {
                        let mut msg = format!("Plugin: {} v{}\n", p.name, p.version);
                        msg.push_str(&format!("  Source: {}\n", p.source));
                        msg.push_str(&format!("  Status: {}\n", p.status));
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::err(&format!("Plugin '{}' not found.", name)),
                }
            }
            _ => {
                CommandOutput::err("Usage:\n  /plugin list               List registered plugins\n  /plugin load <path>        Scan directory for plugins\n  /plugin unload <name>      Unregister a plugin\n  /plugin info <name>        Show plugin details")
            }
        }
    }
}
