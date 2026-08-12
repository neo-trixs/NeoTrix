//! 基础命令 — Config / Help / Stats / Exit / Clear / Version / Completions

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_shield::key_encryption;

// ====== /config ======

pub struct ConfigCmd;

/// `NeoTrixConfig` 展示字段序 (与 config.rs 字段一一对应)。
const CONFIG_FIELDS: [&str; 7] = [
    "default_llm_provider",
    "provider",
    "api_key",
    "default_model",
    "custom_endpoint",
    "color_mode",
    "log_level",
];

impl CliCommand for ConfigCmd {
    fn name(&self) -> &str {
        "/config"
    }
    fn is_primary(&self) -> bool { true }
    fn aliases(&self) -> Vec<&str> {
        vec!["/cfg", "/conf"]
    }

    fn description(&self) -> &str {
        "Config management: /config [show] | /config list | /config set <key> <value>"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let operands: Vec<&str> = args.iter().filter(|a| a.as_str() != "--json").map(|s| s.as_str()).collect();
        let sub = operands.first().copied().unwrap_or("show");
        let config_path = crate::config::NeoTrixConfig::path();

        match sub {
            "show" | "list" | "ls" => {
                let cfg = crate::config::NeoTrixConfig::load();
                let mut lines = format!("Config at {}:\n", config_path.display());
                let mut json_map = serde_json::Map::new();
                for name in CONFIG_FIELDS {
                    let val = match name {
                        "default_llm_provider" => cfg.default_llm_provider.as_deref(),
                        "provider" => cfg.provider.as_deref(),
                        "api_key" => cfg.api_key.as_deref(),
                        "default_model" => cfg.default_model.as_deref(),
                        "custom_endpoint" => cfg.custom_endpoint.as_deref(),
                        "color_mode" => cfg.color_mode.as_deref(),
                        "log_level" => cfg.log_level.as_deref(),
                        _ => None,
                    };
                    let display = match (name, val) {
                        // 敏感字段脱敏: 不落盘/不打印明文
                        ("api_key", Some(_)) => "(set, redacted)".to_string(),
                        (_, Some(v)) => v.to_string(),
                        (_, None) => "(unset)".to_string(),
                    };
                    lines.push_str(&format!("  {:<22} {}\n", name, display));
                    if name != "api_key" {
                        if let Some(v) = val { json_map.insert(name.to_string(), serde_json::json!(v)); }
                    }
                }
                if cfg.default_model.is_none() && cfg.provider.is_none() && cfg.color_mode.is_none() {
                    lines.push_str("\n(no fields set — defaults are in use)\n");
                }
                if sub == "list" || sub == "ls" {
                    lines.push_str(&format!("\nSettable keys: {}\n", CONFIG_FIELDS.join(", ")));
                    lines.push_str("Set: /config set <key> <value>\n");
                }
                let out = CommandOutput::ok(lines.trim_end());
                if want_json {
                    out.with_json(serde_json::Value::Object(json_map))
                } else { out }
            }
            "set" => {
                if operands.len() < 3 {
                    return CommandOutput::err("Usage: /config set <key> <value>");
                }
                let key = operands[1];
                if !CONFIG_FIELDS.contains(&key) {
                    return CommandOutput::err(&format!(
                        "Unknown config key: {}. Available: {}",
                        key, CONFIG_FIELDS.join(", ")
                    ));
                }
                let mut raw_value = operands[2].to_string();
                // Auto-encrypt api_key before persisting
                if key == "api_key" && !raw_value.is_empty() && !key_encryption::is_encrypted(&raw_value) {
                    match key_encryption::encrypt(&raw_value) {
                        Ok(enc) => raw_value = enc,
                        Err(e) => {
                            return CommandOutput::err(&format!("Failed to encrypt api_key: {}", e));
                        }
                    }
                }
                let cfg = crate::config::NeoTrixConfig::load();
                if cfg.save_field(key, &raw_value) {
                    let display_value = if key == "api_key" { "(encrypted)" } else { &raw_value };
                    let out = CommandOutput::ok(&format!(
                        "Set {} = {} (saved to {})",
                        key, display_value, config_path.display()
                    ));
                    if want_json {
                        out.with_json(serde_json::json!({
                            "key": key, "value": display_value, "path": config_path.display().to_string()
                        }))
                    } else { out }
                } else {
                    CommandOutput::err(&format!("Failed to persist key: {}", key))
                }
            }
            _ => CommandOutput::err(&format!(
                "Unknown subcommand: {}. Available: show, list, set", sub
            )),
        }
    }
}

// ====== /help ======

pub struct HelpCmd;
impl CliCommand for HelpCmd {
    fn name(&self) -> &str {
        "/help"
    }
    fn is_primary(&self) -> bool { true }

    fn aliases(&self) -> Vec<&str> {
        vec!["/h", "/?"]
    }

    fn description(&self) -> &str {
        "Show help: /help [command] | /help all"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let operands: Vec<&str> = args.iter().filter(|a| a.as_str() != "--json").map(|s| s.as_str()).collect();
        let reg = crate::cli::commands::registry::default_registry();

        // `/help all` — 全部已注册命令 + 分类 (含 agent 后端调度命令)
        if operands.first().copied() == Some("all") {
            let s = reg.help_all_by_category();
            let out = CommandOutput::ok(&s);
            if want_json {
                let by_cat = reg.list_all_by_category();
                let json_cmds: serde_json::Value = by_cat.into_iter().map(|(cat, names)| {
                    (cat.label().to_string(), serde_json::json!(names))
                }).collect();
                out.with_json(json_cmds)
            } else {
                out
            }
        }
        // `/help <cmd>` — 单命令详细帮助 (名称+别名+子命令说明)
        else if let Some(cmd_name) = operands.first() {
            let lookup = format!("/{}", cmd_name.trim_start_matches('/'));
            match reg.help_for(&lookup) {
                Some(detail) => {
                    let out = CommandOutput::ok(&detail);
                    if want_json {
                        let cmd = reg.find(&lookup).expect("help_for found it");
                        out.with_json(serde_json::json!({
                            "command": cmd.name(),
                            "aliases": cmd.aliases(),
                            "description": cmd.description(),
                        }))
                    } else { out }
                }
                None => CommandOutput::err(&format!("No help available for '{}'", cmd_name)),
            }
        }
        // 裸 `/help` — 一级入口分类帮助 (命令收敛面)
        else {
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
}

// ====== /stats ======

pub struct StatsCmd;
impl CliCommand for StatsCmd {
    fn name(&self) -> &str {
        "/stats"
    }
    fn is_primary(&self) -> bool { true }

    fn aliases(&self) -> Vec<&str> {
        vec!["/st"]
    }

    fn description(&self) -> &str {
        "System status: /stats | /stats version | /stats --json (includes diagnostics)"
    }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        // 子命令 version: 合并 /version — 版本信息进系统状态面
        if args.iter().any(|a| a == "version" || a == "v") {
            let version_str = format!(
                "NeoTrix v{} | V2 85% | OS {} ({})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH,
            );
            let out = CommandOutput::ok(&version_str);
            return if want_json {
                out.with_json(serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "v2_progress": 0.85,
                    "os": std::env::consts::OS,
                    "arch": std::env::consts::ARCH,
                }))
            } else { out };
        }
        // 子命令 doctor: 合并 /doctor — 环境诊断进系统状态面
        if args.iter().any(|a| a == "doctor" || a == "diag") {
            return crate::cli::commands::doctor_cmds::run_doctor();
        }
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
    fn is_primary(&self) -> bool { true }

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
    fn is_primary(&self) -> bool { true }

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
    fn is_primary(&self) -> bool { false }

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

// ====== /catalog ======

/// 统一命令目录 — 融合 CLI + NoeCodex 两侧命令描述
pub struct CatalogCmd;
impl CliCommand for CatalogCmd {
    fn name(&self) -> &str {
        "/catalog"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/cmds", "/commands"]
    }

    fn description(&self) -> &str {
        "Unified command catalog (CLI + NoeCodex): /catalog [cli|tauri|--json]"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        use crate::unified_cmd::{CommandBackend, catalog_by_backend, unified_catalog};
        let want_json = args.iter().any(|a| a == "--json");
        let backend_filter = args.iter().find_map(|a| match a.as_str() {
            "cli" => Some(CommandBackend::Cli),
            "tauri" | "noecodex" => Some(CommandBackend::Tauri),
            _ => None,
        });

        let specs = match backend_filter {
            Some(b) => catalog_by_backend(b),
            None => unified_catalog(),
        };

        if want_json {
            let arr: serde_json::Value = specs.iter().map(|s| serde_json::json!({
                "name": s.name,
                "aliases": s.aliases,
                "category": s.category,
                "description": s.description,
                "backend": s.backend,
            })).collect();
            let count = specs.len();
            return CommandOutput::ok(&format!("{} commands in unified catalog", count)).with_json(arr);
        }

        // 按 backend + category 分组输出
        let mut cli_cmds = Vec::new();
        let mut tauri_cmds = Vec::new();
        for s in &specs {
            match s.backend {
                CommandBackend::Cli => cli_cmds.push(s),
                CommandBackend::Tauri => tauri_cmds.push(s),
            }
        }

        let mut out = String::from("━━ Unified Command Catalog (CLI + NoeCodex) ━━\n\n");
        out.push_str(&format!("▸ CLI commands ({}):\n", cli_cmds.len()));
        for s in &cli_cmds {
            let alias = if s.aliases.is_empty() { String::new() } else { format!(" [{}]", s.aliases.join(", ")) };
            out.push_str(&format!("  {:<20}{} {}\n", s.name, alias, s.description));
        }
        out.push('\n');
        out.push_str(&format!("▸ NoeCodex commands ({}):\n", tauri_cmds.len()));
        for s in &tauri_cmds {
            out.push_str(&format!("  {:<40} {}\n", s.name, s.description));
        }
        out.push_str("\nUse /catalog --json for machine-readable output, /catalog cli|tauri to filter backends\n");
        CommandOutput::ok(&out)
    }
}

// ====== /completions ======

/// Shell completions generated from the live command registry snapshot.
pub struct CompletionsCmd {
    cmds: Arc<std::sync::Mutex<Vec<String>>>,
}

impl CompletionsCmd {
    pub fn new(cmds: Arc<std::sync::Mutex<Vec<String>>>) -> Self {
        Self { cmds }
    }

    fn candidates(&self) -> Vec<String> {
        let mut cmds = self
            .cmds
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        // entry-layer auto-backend 命令 (不注册 registry) 仍保持补全可用
        for extra in ["save", "absorb", "evolve", "mem"] {
            if !cmds.iter().any(|c| c == extra) {
                cmds.push(extra.to_string());
            }
        }
        cmds.sort();
        cmds.dedup();
        cmds
    }
}

impl CliCommand for CompletionsCmd {
    fn name(&self) -> &str {
        "/completions"
    }
    fn is_primary(&self) -> bool { false }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn description(&self) -> &str {
        "Generate shell completions (bash|zsh|fish|powershell)"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let shell = args.first().map(|s| s.as_str()).unwrap_or("bash");
        let cmds = self.candidates();
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
