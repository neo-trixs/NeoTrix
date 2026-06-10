//! Core commands — Config / Help / Stats / Exit / Clear / Version / Completions

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

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

    fn help_detail(&self) -> Option<String> {
        Some("Manage NeoTrix configuration. Use 'show' to view current config, 'set <key> <value>' to modify settings. Configuration is persisted to ~/.config/neotrix/config.toml.".into())
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let config_path = dirs::home_dir().unwrap_or_default().join(".config").join("neotrix").join("config.toml");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
            let msg = if config_str.is_empty() {
                "Usage:\n  /config show              Show current configuration\n  /config set <key> <value>  Set a config key\n  /config --json             Output as JSON\n(no config file found at ~/.neotrix/config.toml)".to_string()
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
                let value = &args[2];
                // Read existing, update key, write back
                let mut config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
                let key_line = format!("{} = ", key);
                if config_str.lines().any(|l| l.trim().starts_with(&key_line)) {
                    // Replace existing
                    let mut new_lines: Vec<String> = config_str.lines().map(|l| {
                        if l.trim().starts_with(&key_line) {
                            if value.contains(' ') || value.contains('#') {
                                format!("{} = \"{}\"", key, value)
                            } else {
                                format!("{} = {}", key, value)
                            }
                        } else { l.to_string() }
                    }).collect();
                    if !config_str.ends_with('\n') { new_lines.push(String::new()); }
                    config_str = new_lines.join("\n");
                } else {
                    if !config_str.ends_with('\n') { config_str.push('\n'); }
                    if value.contains(' ') || value.contains('#') {
                        config_str.push_str(&format!("{} = \"{}\"\n", key, value));
                    } else {
                        config_str.push_str(&format!("{} = {}\n", key, value));
                    }
                }
                if let Some(dir) = config_path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                match std::fs::write(&config_path, &config_str) {
                    Ok(()) => CommandOutput::ok(&format!("Set {} = {} (saved to {})", key, value, config_path.display())),
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

    fn help_detail(&self) -> Option<String> {
        Some("Display help information for all available commands. Use '/help <command>' to get detailed help for a specific command.".into())
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        // Per-command help
        if args.len() >= 1 && args[0] != "--json" {
            let cmd_name = if args[0].starts_with('/') { &args[0] } else { args[0].as_str() };
            return match cmd_name {
                "/help" => CommandOutput::ok("Display this help. Usage: /help [command]"),
                "/stats" => CommandOutput::ok("Show reasoning statistics: capabilities, iterations, memory"),
                "/cost" => CommandOutput::ok("Track token usage and costs. Sub: detail, budget, reset"),
                "/save" => CommandOutput::ok("Save brain state to ~/.neotrix/brain.json"),
                "/absorb" => CommandOutput::ok("Absorb knowledge from a source URL or file"),
                "/evolve" => CommandOutput::ok("Run the SEAL self-evolution loop"),
                "/mem" => CommandOutput::ok("Browse and query reasoning memory"),
                "/agent" => CommandOutput::ok("Manage sub-agents. Sub: spawn, list, kill"),
                "/mcp" => CommandOutput::ok("MCP server management. Sub: list, add, auth, debug"),
                "/profile" => CommandOutput::ok("Permission profiles. Sub: list, switch, show, create, rm, set"),
                "/clear" => CommandOutput::ok("Clear the terminal screen"),
                "/version" => CommandOutput::ok("Show version and build info"),
                "/completions" => CommandOutput::ok("Generate shell completions (bash|zsh|fish|powershell)"),
                "/exit" => CommandOutput::ok("Exit the application"),
                "/read" => CommandOutput::ok("Read a file from the filesystem"),
                "/write" => CommandOutput::ok("Write content to a file"),
                "/create" => CommandOutput::ok("Create a new file"),
                "/edit" => CommandOutput::ok("Edit an existing file with text replacement"),
                "/patch" => CommandOutput::ok("Apply a diff patch to a file"),
                "/diff" => CommandOutput::ok("Show diff between files"),
                "/approval" => CommandOutput::ok("Approval mode: suggest|auto-edit|full-auto. Sub: status, list, approve, deny"),
                "/git" => CommandOutput::ok("Git operations. Sub: status, log, branch, diff"),
                "/commit" => CommandOutput::ok("Create a git commit"),
                "/pr" => CommandOutput::ok("Create a pull request"),
                "/session" => CommandOutput::ok("Session management. Sub: list, switch, delete, fork, export, import"),
                "/resume" => CommandOutput::ok("Resume a previous session by ID"),
                "/compact" => CommandOutput::ok("Compact context window. Sub: now"),
                "/context" => CommandOutput::ok("Context management. Sub: clear, model"),
                "/side" => CommandOutput::ok("Quick side question without switching context"),
                "/history" => CommandOutput::ok("Show command history"),
                "/search" => CommandOutput::ok("Search the web. Usage: /search <query> [-n <N>]"),
                "/schedule" => CommandOutput::ok("Schedule recurring tasks"),
                "/doctor" => CommandOutput::ok("Run system diagnostics"),
                "/theme" => CommandOutput::ok("Change TUI color theme"),
                "/review" => CommandOutput::ok("Code review: /review [path]"),
                _ => CommandOutput::err(&format!("No help available for '{}'", cmd_name)),
            };
        }

        let mut s = "Commands:\n".to_string();
        for (n, d) in &[
            ("/help","Show help for a command"),("/stats","Reasoning statistics"),
            ("/cost","Cost tracking"),("/save","Save brain state"),
            ("/absorb","Absorb knowledge"),("/evolve","SEAL evolution loop"),
            ("/mem","Memory nt_world_browse"),("/agent","Sub-agent management"),
            ("/mcp","MCP server management"),("/profile","Permission profiles"),
            ("/clear","Clear screen"),("/version","Show version"),
            ("/completions","Shell completions"),("/exit","Exit"),
            ("/read","Read file"),("/write","Write file"),
            ("/create","Create file"),("/edit","Edit file"),
            ("/patch","Apply patch"),("/diff","Show diff"),
            ("/approval","Approval mode"),("/git","Git operations"),
            ("/commit","Git commit"),("/pr","Pull request"),
            ("/session","Session management"),("/resume","Resume session"),
            ("/compact","Compress context"),("/context","Context settings"),
            ("/side","Side question"),("/history","Command history"),
            ("/search","Web search"),("/schedule","Schedule tasks"),
            ("/doctor","System diagnostics"),("/theme","Color theme"),
            ("/review","Code review"),
        ] {
            s.push_str(&format!("  {:15}{}\n", n, d));
        }
        s.push_str("\nRun /help <command> for detailed help on a specific command.\n");
        let out = CommandOutput::ok(&s);
        if want_json {
            let cmds: Vec<&str> = vec!["/help","/stats","/cost","/save","/absorb","/evolve","/mem","/agent","/mcp","/profile","/clear","/version","/completions","/exit","/read","/write","/create","/edit","/patch","/diff","/approval","/git","/commit","/pr","/session","/resume","/compact","/context","/side","/history","/search","/schedule","/doctor","/theme","/review"];
            out.with_json(serde_json::json!({"commands": cmds, "count": cmds.len()}))
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

    fn help_detail(&self) -> Option<String> {
        Some("Display current reasoning statistics including capability sum, iteration count, absorbed sources, and memory entries. Use --json for machine-readable output.".into())
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
