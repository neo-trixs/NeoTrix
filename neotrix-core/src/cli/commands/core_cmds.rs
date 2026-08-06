//! 基础命令 — Config / Help / Stats / Exit / Clear / Version / Completions

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_shield::key_encryption;

// ====== /config ======

pub struct ConfigCmd;
impl CliCommand for ConfigCmd {
    fn name(&self) -> &str {
        "/config"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/cfg", "/conf"]
    }

    fn description(&self) -> &str {
        "Config management: /config show | /config set <key> <value>"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let config_path = dirs::home_dir().unwrap_or_default().join(".config").join("neotrix").join("config.toml");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
            let msg = if config_str.is_empty() {
                format!("Usage:\n  /config show              Show current configuration\n  /config set <key> <value>  Set a config key\n  /config --json             Output as JSON\n(no config file found at {})", config_path.display())
            } else {
                format!("Config at {}:\n{}", config_path.display(), config_str)
            };
            let out = CommandOutput::ok(&msg);
            return if want_json {
                let parsed: serde_json::Value = config_str.parse().unwrap_or(serde_json::json!({"note": "parse failed"}));
                out.with_json(parsed)
            } else { out };
        }

        let sub = args[0].as_str();
        match sub {
            "show" => {
                let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
                if config_str.is_empty() {
                    CommandOutput::ok("No config file found. Defaults will be used.")
                } else {
                    CommandOutput::ok(&format!("Config at {}:\n{}", config_path.display(), config_str))
                }
            }
            "set" => {
                if args.len() < 3 { return CommandOutput::err("Usage: /config set <key> <value>"); }
                let key = &args[1];
                let mut raw_value = args[2].clone();
                // Auto-encrypt api_key before persisting
                if key == "api_key" && !raw_value.is_empty() && !key_encryption::is_encrypted(&raw_value) {
                    match key_encryption::encrypt(&raw_value) {
                        Ok(enc) => raw_value = enc,
                        Err(e) => {
                            return CommandOutput::err(&format!("Failed to encrypt api_key: {}", e));
                        }
                    }
                }
                // Read existing, update key, write back
                let mut config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
                let key_line = format!("{} = ", key);
                if config_str.lines().any(|l| l.trim().starts_with(&key_line)) {
                    // Replace existing
                    let mut new_lines: Vec<String> = config_str.lines().map(|l| {
                        if l.trim().starts_with(&key_line) {
                            if raw_value.contains(' ') || raw_value.contains('#') {
                                format!("{} = \"{}\"", key, raw_value)
                            } else {
                                format!("{} = {}", key, raw_value)
                            }
                        } else { l.to_string() }
                    }).collect();
                    if !config_str.ends_with('\n') { new_lines.push(String::new()); }
                    config_str = new_lines.join("\n");
                } else {
                    if !config_str.ends_with('\n') { config_str.push('\n'); }
                    if raw_value.contains(' ') || raw_value.contains('#') {
                        config_str.push_str(&format!("{} = \"{}\"\n", key, raw_value));
                    } else {
                        config_str.push_str(&format!("{} = {}\n", key, raw_value));
                    }
                }
                if let Some(dir) = config_path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let display_key = if key == "api_key" { "api_key" } else { key };
                let display_value = if key == "api_key" { "(encrypted)" } else { &raw_value };
                match std::fs::write(&config_path, &config_str) {
                    Ok(()) => CommandOutput::ok(&format!("Set {} = {} (saved to {})", display_key, display_value, config_path.display())),
                    Err(e) => CommandOutput::err(&format!("Failed to write config: {}", e)),
                }
            }
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Available: show, set", sub)),
        }
    }
}

// ====== /help ======

pub struct HelpCmd;
impl CliCommand for HelpCmd {
    fn name(&self) -> &str {
        "/help"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/h", "/?"]
    }

    fn description(&self) -> &str {
        "Show help: /help [command]"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        // Per-command detail — delegate to the command's own description
        if !args.is_empty() && args[0] != "--json" && args[0] != "all" {
            let reg = crate::cli::commands::registry::default_registry();
            let lookup = format!("/{}", args[0].trim_start_matches('/'));
            return match reg.find(&lookup) {
                Some(cmd) => CommandOutput::ok(&format!("{} — {}", cmd.name(), cmd.description())),
                None => CommandOutput::err(&format!("No help available for '{}'", args[0])),
            };
        }

        // Categorized full help
        let reg = crate::cli::commands::registry::default_registry();
        let s = reg.help_by_category();
        let out = CommandOutput::ok(&s);
        if want_json {
            let by_cat = reg.list_by_category();
            let json_cmds: serde_json::Value = by_cat.into_iter().map(|(cat, names)| {
                (cat.label().to_string(), serde_json::json!(names))
            }).collect();
            out.with_json(json_cmds)
        } else {
            out
        }
    }
}

// ====== /stats ======

pub struct StatsCmd;
impl CliCommand for StatsCmd {
    fn name(&self) -> &str {
        "/stats"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/st"]
    }

    fn description(&self) -> &str {
        "Show reasoning stats: capabilities, iterations, memory"
    }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        if let Some(b) = brain {
            let a = b.blocking_read();
            let stats = a.brain.get_statistics();
            let msg = format!("Capabilities: {:.3} | Iterations: {} | Absorb: {} | Memory: {}",
                stats.capability_sum, a.iteration, a.brain.total_absorb_count,
                a.reasoning_bank.memories().len());
            let out = CommandOutput::ok(&msg);
            return if want_json { out.with_json(serde_json::json!({
                "capability_sum": stats.capability_sum,
                "iteration": a.iteration,
                "absorb_count": a.brain.total_absorb_count,
                "memory_count": a.reasoning_bank.memories().len(),
                "learning_rate": a.brain.learning_rate,
            }))} else { out };
        }
        let out = CommandOutput::ok("23-dim | learning_rate 0.05 | ready");
        if want_json {
            out.with_json(serde_json::json!({
                "capability_dimensions": 23, "learning_rate": 0.05, "status": "ready"
            }))
        } else {
            out
        }
    }
}

// ====== /exit ======

pub struct ExitCmd;
impl CliCommand for ExitCmd {
    fn name(&self) -> &str {
        "/exit"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/q", "/quit"]
    }

    fn description(&self) -> &str {
        "Exit the application"
    }

    fn execute(&self, _: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        CommandOutput::ok("Goodbye")
    }
}

// ====== /clear ======

pub struct ClearCmd;
impl CliCommand for ClearCmd {
    fn name(&self) -> &str {
        "/clear"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn description(&self) -> &str {
        "Clear the terminal screen"
    }

    fn execute(&self, _: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        CommandOutput::ok(&"\n".repeat(50))
    }
}

// ====== /version ======

pub struct VersionCmd;
impl CliCommand for VersionCmd {
    fn name(&self) -> &str {
        "/version"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/v"]
    }

    fn description(&self) -> &str {
        "Show version and build info"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let out = CommandOutput::ok("NeoTrix v0.3.0 | V2 85%");
        if want_json {
            out.with_json(serde_json::json!({"version": "0.3.0", "v2_progress": 0.85}))
        } else {
            out
        }
    }
}

// ====== /completions ======

pub struct CompletionsCmd;
impl CliCommand for CompletionsCmd {
    fn name(&self) -> &str {
        "/completions"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn description(&self) -> &str {
        "Generate shell completions (bash|zsh|fish|powershell)"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
        let cmds = vec!["help","stats","save","absorb","evolve","mem","agent","mcp","clear","version","completions","exit"];
        match shell {
            "bash" => {
                let mut s = String::new();
                s.push_str("_neotrix() {\n  local cur=${COMP_WORDS[COMP_CWORD]}\n");
                s.push_str(&format!("  COMPREPLY=($(compgen -W \"{} --json\" -- \"$cur\"))\n}}\n", cmds.join(" ")));
                s.push_str("complete -F _neotrix neotrix\n");
                CommandOutput::ok(&s)
            }
            "zsh" => {
                let mut s = String::new();
                s.push_str("#compdef neotrix\n");
                s.push_str(&format!("_arguments \\\n  '(-):command:({})' \\\n  '--json[(output as JSON)]'\n", cmds.join(" ")));
                CommandOutput::ok(&s)
            }
            "fish" => {
                let mut s = String::new();
                for cmd in &cmds {
                    s.push_str(&format!("complete -c neotrix -a '{}' -d '{} command'\n", cmd, cmd));
                }
                s.push_str("complete -c neotrix -l json -d 'Output as JSON'\n");
                CommandOutput::ok(&s)
            }
            "powershell" => {
                let s = format!(
                    "Register-ArgumentCompleter -Native -CommandName neotrix -ScriptBlock {{\n  param($wordToComplete)\n  @({})\n}}",
                    cmds.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(", ")
                );
                CommandOutput::ok(&s)
            }
            _ => CommandOutput::err(&format!("Unsupported shell: {}. Supported: bash, zsh, fish, powershell", shell)),
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
