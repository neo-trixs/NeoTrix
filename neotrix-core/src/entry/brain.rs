//! brain — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::path::PathBuf;

use super::{info, warn};
use neotrix::l1_action::nt_core_bank::bank::ReasoningBank;
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::{ReasoningBrain, SelfIteratingBrain};

pub(crate) fn print_brain_stats(brain: &SelfIteratingBrain) {
    let stats = brain.brain.get_statistics();
    println!(
        "\n{}",
        info("╭─ NeoTrix V2 Brain Status ──────────────────────────╮")
    );
    println!(
        "│ {} {:<5}  {} {:<5}             │",
        info("Iteration:"),
        brain.iteration,
        info("Absorbed:"),
        brain.brain.total_absorb_count
    );
    println!(
        "│ {} {:.3}  {} {:<5}       │",
        info("Capability Sum:"),
        stats.capability_sum,
        info("Memory:"),
        brain.reasoning_bank.memories().len()
    );
    println!(
        "{}",
        info("╰──────────────────────────────────────────────────────╯")
    );
}

pub(crate) fn brain_dir(profile: &str) -> PathBuf {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix");
    if profile.is_empty() || profile == "default" {
        base
    } else {
        base.join("profiles").join(profile)
    }
}

pub(crate) fn init_brain(profile: &str) -> (ReasoningBrain, ReasoningBank) {
    let dir = brain_dir(profile);
    std::env::set_var("NEOTRIX_HOME", &dir);

    if ReasoningBrain::has_saved_state() {
        match ReasoningBrain::load() {
            Ok(b) => {
                println!(
                    "{}",
                    info(format!("Loaded brain from {}/brain.json", dir.display()))
                );
                (b, ReasoningBank::new(100))
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    warn(format!("Load failed ({}), creating new brain", e))
                );
                (ReasoningBrain::new(), ReasoningBank::new(100))
            }
        }
    } else {
        println!(
            "{}",
            info(format!("New brain at {}/brain.json", dir.display()))
        );
        (ReasoningBrain::new(), ReasoningBank::new(100))
    }
}

pub(crate) fn set_default_model_from_config(agent: &mut SelfIteratingBrain) {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if let Some(ref model) = cfg.default_model {
        if !model.is_empty() {
            agent.default_model = model.clone();
        }
    }
}

/// 构建自进化 brain — 抽取 7 处重复初始化样板 (审计 R-P99 去重)。
///
/// 核心 5 步: init_brain → SelfIteratingBrain::new → 挂载 brain/reasoning_bank
/// → set_default_model_from_config → ensure_provider_env_from_config → init_reasoning_engine。
/// 带 load_cortex 的变体 (run_daemon/evolution) 不共用, 因顺序不同。
pub(crate) fn build_brain(profile: &str) -> SelfIteratingBrain {
    let (brain, bank) = init_brain(profile);
    let mut agent = SelfIteratingBrain::new();
    agent.brain = brain;
    agent.reasoning_bank = bank;
    set_default_model_from_config(&mut agent);
    ensure_provider_env_from_config();
    agent.init_reasoning_engine();
    agent
}

/// 将 config.toml 中的 provider/api_key 提升为环境变量，使 GatewayV2 能发现
pub(crate) fn ensure_provider_env_from_config() {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if let (Some(provider), Some(api_key)) = (&cfg.provider, &cfg.api_key) {
        if !api_key.is_empty() {
            match provider.as_str() {
                "openai" => {
                    if std::env::var("OPENAI_API_KEY").is_err() {
                        std::env::set_var("OPENAI_API_KEY", api_key);
                    }
                }
                "anthropic" => {
                    if std::env::var("ANTHROPIC_API_KEY").is_err() {
                        std::env::var("ANTHROPIC_API_KEY").ok();
                        std::env::set_var("ANTHROPIC_API_KEY", api_key);
                    }
                }
                "custom" => {
                    if std::env::var("NEOTRIX_API_KEY").is_err() {
                        std::env::set_var("NEOTRIX_API_KEY", api_key);
                    }
                    if let Some(ref endpoint) = cfg.custom_endpoint {
                        if std::env::var("NEOTRIX_BASE_URL").is_err() {
                            std::env::set_var("NEOTRIX_BASE_URL", endpoint);
                        }
                    }
                    if let Some(ref model) = cfg.default_model {
                        if std::env::var("NEOTRIX_MODEL").is_err() {
                            std::env::set_var("NEOTRIX_MODEL", model);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
