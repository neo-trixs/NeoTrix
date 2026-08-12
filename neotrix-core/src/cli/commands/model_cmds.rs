use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

fn config_path() -> PathBuf {
    crate::config::NeoTrixConfig::path()
}

/// 读取当前 provider / default_model (config 优先, env 兜底)。
fn current_provider_model() -> (String, String) {
    let cfg = crate::config::NeoTrixConfig::load();
    let provider = cfg.provider
        .or_else(|| std::env::var("NEOTRIX_PROVIDER").ok())
        .unwrap_or_else(|| "auto".to_string());
    let model = cfg.default_model
        .or_else(|| std::env::var("NEOTRIX_MODEL").ok())
        .unwrap_or_else(|| "(not set)".to_string());
    (provider, model)
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
        "Set default model/provider (persisted): /model <name> | /model set <provider> [model] | /model list | /model current"
    }
    fn is_primary(&self) -> bool { false }


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
                msg.push_str("\nSet: /model set <provider> [model] | /model <model-name>");

                let (current_provider, current_model) = current_provider_model();
                msg.push_str(&format!("\nCurrent: {} / {}\n", current_provider, current_model));
                CommandOutput::ok(&msg)
            }
            "set" | "switch" if args.len() >= 2 => {
                let provider = &args[1];
                let model = args.get(2).map(|s| s.as_str()).unwrap_or("default");

                // 运行时生效 (env)
                set_env_var("NEOTRIX_PROVIDER", provider);
                if model != "default" {
                    set_env_var("NEOTRIX_MODEL", model);
                }

                // 持久化 (config.toml)
                let cfg = crate::config::NeoTrixConfig::load();
                if !cfg.save_field("provider", provider) {
                    return CommandOutput::err(&format!("Failed to persist provider to {}", config_path().display()));
                }
                let mut msg = format!("✅ Provider set to: {}\n", provider);
                if model != "default" {
                    if !cfg.save_field("default_model", model) {
                        return CommandOutput::err(&format!("Failed to persist model to {}", config_path().display()));
                    }
                    msg.push_str(&format!("   Model set to: {}\n", model));
                }
                msg.push_str(&format!("   Saved to: {}", config_path().display()));
                CommandOutput::ok(&msg)
            }
            "current" | "status" => {
                let (provider, model) = current_provider_model();
                let base_url = crate::config::NeoTrixConfig::load()
                    .custom_endpoint
                    .unwrap_or_else(|| "(not set)".to_string());
                CommandOutput::ok(&format!(
                    "Current LLM config:\n  Provider:    {}\n  Model:       {}\n  Base URL:    {}\n  Config file: {}\n",
                    provider, model, base_url,
                    config_path().display()
                ))
            }
            // 裸参数: /model <name> — 持久化默认模型名
            "" | "help" | "h" | "usage" => {
                CommandOutput::err("Usage:\n  /model <name>                   Set default model (persisted)\n  /model list                       List available models\n  /model set <provider> [model]     Set provider/model (persisted)\n  /model current                    Show current config")
            }
            name => {
                if name.starts_with('/') {
                    return CommandOutput::err(&format!("Unknown /model subcommand: {}", name));
                }
                let cfg = crate::config::NeoTrixConfig::load();
                if cfg.save_field("default_model", name) {
                    set_env_var("NEOTRIX_MODEL", name);
                    CommandOutput::ok(&format!(
                        "✅ Default model set to: {}\n   Saved to: {}",
                        name, config_path().display()
                    ))
                } else {
                    CommandOutput::err(&format!("Failed to persist model to {}", config_path().display()))
                }
            }
        }
    }
}
