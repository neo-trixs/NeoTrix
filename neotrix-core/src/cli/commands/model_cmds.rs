use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("neotrix").join("config.toml")
}

fn read_config_toml() -> toml::Value {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = content.parse::<toml::Value>() {
                return cfg;
            }
        }
    }
    toml::Value::Table(toml::value::Table::new())
}

fn write_config_toml(cfg: &toml::Value) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let output = toml::to_string_pretty(cfg).map_err(|e| format!("Serialization error: {}", e))?;
    std::fs::write(&path, output).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

fn set_env_var(name: &str, val: &str) {
    std::env::set_var(name, val);
}

pub struct ModelCmd;

impl CliCommand for ModelCmd {
    fn name(&self) -> &str {
        "/model"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/provider", "/llm"]
    }

    fn description(&self) -> &str {
        "Switch model/provider: /model list | /model set <provider> [model] | /model current"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");
        match sub {
            "list" | "ls" => {
                let providers = vec![
                    ("openai",     vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"]),
                    ("anthropic",  vec!["claude-3-opus", "claude-3-sonnet", "claude-3-haiku", "claude-3.5-sonnet"]),
                    ("gemini",     vec!["gemini-2.0-pro", "gemini-2.0-flash", "gemini-1.5-pro"]),
                    ("deepseek",   vec!["deepseek-chat", "deepseek-reasoner"]),
                    ("openrouter", vec!["auto"]),
                    ("ollama",     vec!["llama3", "mistral", "codellama"]),
                ];
                let mut msg = String::from("📋 Available providers and models:\n\n");
                for (prov, models) in &providers {
                    msg.push_str(&format!("  {}:\n", prov));
                    for m in models {
                        msg.push_str(&format!("    - {}\n", m));
                    }
                }
                msg.push_str("\nSet: /model set <provider> [model]");

                let cfg = read_config_toml();
                let current_provider = cfg.get("provider").and_then(|v| v.as_str()).unwrap_or("(not set)");
                let current_model = cfg.get("model").and_then(|v| v.as_str()).unwrap_or("default");
                msg.push_str(&format!("\nCurrent: {} / {}\n", current_provider, current_model));
                CommandOutput::ok(&msg)
            }
            "set" | "switch" if args.len() >= 2 => {
                let provider = &args[1];
                let model = args.get(2).map(|s| s.as_str()).unwrap_or("default");

                set_env_var("NEOTRIX_PROVIDER", provider);
                if model != "default" {
                    set_env_var("NEOTRIX_MODEL", model);
                }

                let mut cfg = read_config_toml();
                if let Some(table) = cfg.as_table_mut() {
                    table.insert("provider".to_string(), toml::Value::String(provider.to_string()));
                    if model != "default" {
                        table.insert("model".to_string(), toml::Value::String(model.to_string()));
                    }
                }
                match write_config_toml(&cfg) {
                    Ok(()) => {
                        let mut msg = format!("✅ Provider set to: {}\n", provider);
                        if model != "default" {
                            msg.push_str(&format!("   Model set to: {}\n", model));
                        }
                        msg.push_str(&format!("   Written to: {}", config_path().display()));
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&format!("Failed to write config: {}", e)),
                }
            }
            "current" | "status" => {
                let cfg = read_config_toml();
                let provider = cfg.get("provider").and_then(|v| v.as_str()).unwrap_or("(not set)");
                let model = cfg.get("model").and_then(|v| v.as_str()).unwrap_or("(not set)");
                let base_url = cfg.get("base_url").and_then(|v| v.as_str()).unwrap_or("(not set)");
                CommandOutput::ok(&format!(
                    "Current LLM config:\n  Provider: {}\n  Model:    {}\n  Base URL: {}\n  Config:   {}\n",
                    provider, model, base_url,
                    config_path().display()
                ))
            }
            _ => {
                CommandOutput::err("Usage:\n  /model list                       List available models\n  /model set <provider> [model]     Set provider/model (persisted)\n  /model current                    Show current config")
            }
        }
    }
}
