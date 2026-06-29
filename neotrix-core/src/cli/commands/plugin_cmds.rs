//! Plugin management commands — /plugin list / load / unload / info

use std::sync::Mutex;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_plugin::PluginRegistry;

/// Shared mutable registry accessible from CLI commands.
static PLUGIN_REGISTRY: std::sync::LazyLock<Mutex<PluginRegistry>> =
    std::sync::LazyLock::new(|| Mutex::new(PluginRegistry::new()));

pub struct PluginCmd;
impl CliCommand for PluginCmd {
    fn name(&self) -> &str {
        "/plugin"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Plugin management: /plugin list | /plugin load <path> | /plugin unload <name> | /plugin info <name>"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<
            &std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>,
        >,
    ) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");
        match sub {
            "list" | "ls" => {
                let guard = match PLUGIN_REGISTRY.lock() {
                    Ok(g) => g,
                    Err(_) => return CommandOutput::err("plugin registry lock poisoned"),
                };
                let plugins = guard.list();
                if plugins.is_empty() {
                    return CommandOutput::ok("No plugins registered. Use /plugin load <path> to scan a directory.");
                }
                let mut msg = format!("Registered plugins ({}):\n", plugins.len());
                for p in &plugins {
                    let state_str = match &p.state {
                        crate::core::nt_core_plugin::types::PluginState::Loaded => "loaded",
                        crate::core::nt_core_plugin::types::PluginState::Unloaded => "unloaded",
                        crate::core::nt_core_plugin::types::PluginState::Error(e) =>
                            return CommandOutput::ok(&format!("  {} v{} [error: {}] — {}\n", p.name, p.version, e, p.description)),
                    };
                    msg.push_str(&format!("  {} v{} [{}] — {}\n", p.name, p.version, state_str, p.description));
                }
                CommandOutput::ok(&msg)
            }
            "load" => {
                let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if path.is_empty() {
                    return CommandOutput::err("Usage: /plugin load <path>");
                }
                let load_path = std::path::PathBuf::from(path);

                let mut guard = match PLUGIN_REGISTRY.lock() {
                    Ok(g) => g,
                    Err(_) => return CommandOutput::err("plugin registry lock poisoned"),
                };

                // Single .wasm file path
                if load_path.is_file() && path.ends_with(".wasm") {
                    match guard.load_wasm(&load_path) {
                        Ok(()) => {
                            let info = guard.info(
                                &load_path.file_stem().unwrap_or_default().to_string_lossy(),
                            );
                            let name = info.map(|i| i.name).unwrap_or_else(|| path.to_string());
                            CommandOutput::ok(&format!("WASM plugin '{}' loaded.", name))
                        }
                        Err(e) => CommandOutput::err(&e),
                    }
                } else if load_path.is_dir() {
                    // Scan directory for .wasm / .so / .dylib / .dll files
                    let entries = match std::fs::read_dir(&load_path) {
                        Ok(e) => e,
                        Err(e) => {
                            return CommandOutput::err(&format!(
                                "Failed to read directory: {}",
                                e
                            ))
                        }
                    };
                    let mut wasm_loaded = 0usize;
                    let mut native_found = 0usize;
                    let mut errors: Vec<String> = Vec::new();
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        let fpath = entry.path();
                        if fname.ends_with(".wasm") {
                            match guard.load_wasm(&fpath) {
                                Ok(()) => wasm_loaded += 1,
                                Err(e) => errors.push(format!("  {}: {}", fname, e)),
                            }
                        } else if fname.ends_with(".so")
                            || fname.ends_with(".dylib")
                            || fname.ends_with(".dll")
                        {
                            native_found += 1;
                        }
                    }
                    let mut msg = format!(
                        "Scanned {}. Loaded {} WASM plugin(s).",
                        path, wasm_loaded
                    );
                    if native_found > 0 {
                        msg.push_str(&format!(
                            " {} native plugin file(s) found (not yet supported).",
                            native_found
                        ));
                    }
                    for e in &errors {
                        msg.push_str("\n");
                        msg.push_str(e);
                    }
                    CommandOutput::ok(&msg)
                } else {
                    CommandOutput::err(&format!("Not a directory or .wasm file: {}", path))
                }
            }
            "unload" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /plugin unload <name>");
                }
                let mut guard = match PLUGIN_REGISTRY.lock() {
                    Ok(g) => g,
                    Err(_) => return CommandOutput::err("plugin registry lock poisoned"),
                };
                match guard.unregister(name) {
                    Ok(()) => CommandOutput::ok(&format!("Plugin '{}' unloaded.", name)),
                    Err(e) => CommandOutput::err(&format!("Failed to unload '{}': {}", name, e)),
                }
            }
            "info" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /plugin info <name>");
                }
                let guard = match PLUGIN_REGISTRY.lock() {
                    Ok(g) => g,
                    Err(_) => return CommandOutput::err("plugin registry lock poisoned"),
                };
                match guard.info(name) {
                    Some(p) => {
                        let state_str = match &p.state {
                            crate::core::nt_core_plugin::types::PluginState::Loaded => "loaded",
                            crate::core::nt_core_plugin::types::PluginState::Unloaded => "unloaded",
                            crate::core::nt_core_plugin::types::PluginState::Error(e) => &e,
                        };
                        let mut msg = format!("Plugin: {} v{}\n", p.name, p.version);
                        msg.push_str(&format!("  Description: {}\n", p.description));
                        msg.push_str(&format!("  Source:      {}\n", p.source));
                        msg.push_str(&format!("  State:       {}\n", state_str));
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::err(&format!("Plugin '{}' not found.", name)),
                }
            }
            _ => {
                CommandOutput::err("Usage:\n  /plugin list               List registered plugins\n  /plugin load <path>        Scan directory for plugins\n  /plugin unload <name>      Unload a plugin\n  /plugin info <name>        Show plugin details")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add #[serial] to any new tests that use global singletons
    #[test]
    fn test_plugin_cmd_impl_cli_command() {
        let cmd = PluginCmd;
        assert_eq!(cmd.name(), "/plugin");
        assert!(cmd.description().contains("list"));
    }
}
