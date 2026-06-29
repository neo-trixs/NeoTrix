use std::collections::BTreeSet;
use std::path::PathBuf;

use colored::Colorize;
use futures::FutureExt;

use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

use crate::config::NeoTrixConfig;
use neotrix::core::nt_core_util::A2A_DEFAULT_PORT;

mod desktop;
mod headless;
mod proxy_cmd;
mod standalone;

pub use proxy_cmd::run_proxy_cmd;

mod cli_utils;
mod daemon;
mod interactive;
mod sandbox;

pub use cli_utils::generate_completions;
pub use cli_utils::resolve_prompt;
pub use cli_utils::run_benchmark;
pub use cli_utils::run_browse;
pub use cli_utils::run_exec;
pub use cli_utils::run_login;
pub use cli_utils::run_one_shot;
pub use cli_utils::run_search;
pub use cli_utils::run_update;
pub use cli_utils::show_status;

#[cfg(test)]
pub use cli_utils::check_provider_config;
pub use daemon::run_daemon;
pub use daemon::run_daemon_evolution;
pub use interactive::run_interactive;
pub use interactive::run_interactive_with_ephemeral;
pub use sandbox::run_sandbox_cancel;
pub use sandbox::run_sandbox_list;
pub use sandbox::run_sandbox_run;
pub use sandbox::run_sandbox_upload;

pub fn format_token_value(value: &neotrix::core::nt_core_design_token::TokenValue) -> String {
    use neotrix::core::nt_core_design_token::TokenValue;
    match value {
        TokenValue::Color { r, g, b, a } => {
            format!(
                "rgba({:.0},{:.0},{:.0},{:.2})",
                r * 255.0,
                g * 255.0,
                b * 255.0,
                a
            )
        }
        TokenValue::Spacing(v) => format!("{}px", v),
        TokenValue::Easing { x1, y1, x2, y2 } => {
            format!("cubic-bezier({},{},{},{})", x1, y1, x2, y2)
        }
        TokenValue::Shadow {
            offset_x,
            offset_y,
            blur,
            spread,
            r,
            g,
            b,
            a,
        } => {
            format!(
                "inset? {}px {}px {}px {}px rgba({:.0},{:.0},{:.0},{:.2})",
                offset_x,
                offset_y,
                blur,
                spread,
                r * 255.0,
                g * 255.0,
                b * 255.0,
                a
            )
        }
        TokenValue::Motion {
            duration_ms,
            stiffness,
            damping,
        } => {
            format!(
                "{}ms spring(stiffness={},damping={})",
                duration_ms, stiffness, damping
            )
        }
        TokenValue::Font {
            family,
            size,
            weight,
        } => {
            format!("{} {}px weight={}", family, size, weight)
        }
        TokenValue::Radius(v) => format!("{}px", v),
        TokenValue::Opacity(v) => format!("{:.2}", v),
    }
}

fn info(msg: impl AsRef<str>) -> String {
    msg.as_ref().blue().to_string()
}
fn success(msg: impl AsRef<str>) -> String {
    msg.as_ref().green().to_string()
}
fn warn(msg: impl AsRef<str>) -> String {
    msg.as_ref().yellow().to_string()
}
fn err(msg: impl AsRef<str>) -> String {
    msg.as_ref().red().to_string()
}

fn print_brain_stats(brain: &SelfIteratingBrain) {
    let stats = brain.brain.get_statistics();
    log::info!(
        "\n{}",
        info("╭─ NeoTrix V2 Brain Status ──────────────────────────╮")
    );
    log::info!(
        "│ {} {:<5}  {} {:<5}             │",
        info("Iteration:"),
        brain.iteration,
        info("Absorbed:"),
        brain.brain.total_absorb_count
    );
    log::info!(
        "│ {} {:.3}  {} {:<5}       │",
        info("Capability Sum:"),
        stats.capability_sum,
        info("Memory:"),
        brain.reasoning_bank.memories().len()
    );
    log::info!(
        "{}",
        info("╰──────────────────────────────────────────────────────╯")
    );
}

fn brain_dir(profile: &str) -> PathBuf {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix");
    if profile.is_empty() || profile == "default" {
        base
    } else {
        base.join("profiles").join(profile)
    }
}

fn init_brain(profile: &str) -> (ReasoningBrain, ReasoningBank) {
    let dir = brain_dir(profile);
    // SAFETY: NEOTRIX_HOME is never read by any Rust code in this codebase.
    // Called once during startup inside rt.block_on(async {…}) before any
    // concurrent tasks are spawned — no data race possible.
    std::env::set_var("NEOTRIX_HOME", &dir);

    if ReasoningBrain::has_saved_state() {
        match ReasoningBrain::load() {
            Ok(b) => {
                log::info!(
                    "{}",
                    info(format!("Loaded brain from {}/brain.json", dir.display()))
                );
                (b, ReasoningBank::new(100))
            }
            Err(e) => {
                log::warn!(
                    "{}",
                    warn(format!("Load failed ({}), creating new brain", e))
                );
                (ReasoningBrain::new(), ReasoningBank::new(100))
            }
        }
    } else {
        log::info!(
            "{}",
            info(format!("New brain at {}/brain.json", dir.display()))
        );
        (ReasoningBrain::new(), ReasoningBank::new(100))
    }
}

pub fn run_standalone_mode(stage: usize) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        standalone::run_standalone(stage).await;
    });
}

pub fn run_server_mode(addr: &str, profile: &str) {
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::server::http::start_server;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.init_reasoning_engine();
        start_server(Arc::new(RwLock::new(agent)), addr).await;
    });
}

pub fn run_headless_mode(_cfg: &NeoTrixConfig, profile: &str) {
    use neotrix::neotrix::nt_expert_routing::WorldModelV2;
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::neotrix::nt_mind_background_loop::{BackgroundLoop, ConsciousnessIntegration};

    use neotrix::agent::hooks::{HookContext, HookEvent, HookRegistry};
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::{AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            log::info!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            log::info!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        log::info!(
            "{}: {} {}",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count)),
            ""
        );
        log::info!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

        let mut hook_registry = HookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        log::info!(
            "{}: {} {}",
            info("HookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = HookRegistry::check_blocked(&hook_actions) {
            log::warn!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg_goal_loop.load();
        let agent_team = Arc::new(Mutex::new(AgentTeam::new(
            "default",
            ProcessType::Sequential,
        )));
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);
        let bg_handle = tokio::spawn(async move {
            if let Err(e) = std::panic::AssertUnwindSafe(async {
                let mut bg = BackgroundLoop::new(bg_agent)
                    .with_consciousness(ConsciousnessIntegration::new())
                    .with_goal_loop(bg_goal_loop)
                    .with_nt_world_model(WorldModelV2::new(8, 64))
                    .with_a2a_server_default(A2A_DEFAULT_PORT)
                    .with_adaptive_controller_default();
                #[cfg(feature = "stealth-net")]
                {
                    bg = bg.with_world_consciousness();
                }
                bg.start().await;
            })
            .catch_unwind()
            .await
            {
                log::error!("[headless] background loop panicked: {:?}", e);
            }
        });

        let sp = indicatif::ProgressBar::new_spinner();
        let spinner_style =
            match indicatif::ProgressStyle::default_spinner().template("{spinner:.blue} {msg}") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("invalid spinner template: {}", e);
                    indicatif::ProgressStyle::default_spinner()
                }
            };
        sp.set_style(spinner_style);
        sp.set_message("starting headless mode...");
        headless::run_headless(agent, skills_engine, hook_registry).await;
        bg_handle.abort();
        sp.finish_and_clear();
    });
}

pub fn run_discover(port: u16, duration_ms: u64, json: bool) {
    use neotrix::neotrix::nt_agent_protocol::discovery::AgentDiscovery;

    let mut discovery = match AgentDiscovery::new(port) {
        Ok(d) => d,
        Err(e) => {
            log::error!("❌ 绑定 UDP :{} 失败: {}", port, e);
            return;
        }
    };

    log::info!("🔍 扫描中 ({}ms, UDP :{})...", duration_ms, port);
    match discovery.discover(duration_ms) {
        Ok(agents) => {
            if agents.is_empty() {
                log::info!("🔍 扫描完成，未发现任何代理");
                if json {
                    log::info!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "success": true, "agent_count": 0, "port": port, "duration_ms": duration_ms
                    })).unwrap_or_default());
                }
                return;
            }

            if json {
                let json_agents: Vec<serde_json::Value> = agents
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "id": a.id, "name": a.name, "host": a.host, "port": a.port,
                            "capabilities": a.capabilities, "hexagram": a.hexagram,
                            "service_type": a.service_type, "instance_name": a.instance_name,
                        })
                    })
                    .collect();
                log::info!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "success": true, "agent_count": agents.len(), "port": port,
                        "duration_ms": duration_ms, "agents": json_agents
                    }))
                    .unwrap_or_default()
                );
            } else {
                log::info!("🔍 发现 {} 个代理 (扫描 {}ms):", agents.len(), duration_ms);
                log::info!("{:-<72}", "");
                log::info!(
                    " {:<4} {:<24} {:<22} {:<6} {:<4}",
                    "#",
                    "ID",
                    "Host",
                    "Port",
                    "Caps"
                );
                log::info!("{:-<72}", "");
                for (i, a) in agents.iter().enumerate() {
                    let id_trunc = if a.id.len() > 23 {
                        format!("{}…", &a.id[..22])
                    } else {
                        a.id.clone()
                    };
                    log::info!(
                        " {:<4} {:<24} {:<22} {:<6} {:<4}",
                        i + 1,
                        id_trunc,
                        a.host,
                        a.port,
                        a.capabilities.len()
                    );
                }
                log::info!("{:-<72}", "");
                if agents.len() == 1 {
                    let a = &agents[0];
                    log::info!("  详情:");
                    log::info!("    Name:     {}", a.name);
                    log::info!(
                        "    Service:  {}",
                        if a.service_type.is_empty() {
                            "(none)"
                        } else {
                            &a.service_type
                        }
                    );
                    log::info!(
                        "    Instance: {}",
                        if a.instance_name.is_empty() {
                            "(none)"
                        } else {
                            &a.instance_name
                        }
                    );
                    if !a.capabilities.is_empty() {
                        log::info!("    Caps:     {}", a.capabilities.join(", "));
                    }
                    if a.hexagram != 0 {
                        log::info!("    Hexagram: {}", a.hexagram);
                    }
                }
            }
        }
        Err(e) => log::error!("❌ 扫描失败: {}", e),
    }
}

// ── Feature flags ──

/// Path to stored feature flags
fn features_path() -> PathBuf {
    let mut path = neotrix::core::nt_core_util::home_dir();
    path.push(".neotrix");
    if let Err(e) = std::fs::create_dir_all(&path) {
        log::warn!("[features] failed to create dir {}: {}", path.display(), e);
    }
    path.push("features.json");
    path
}

fn load_features() -> BTreeSet<String> {
    let path = features_path();
    if !path.exists() {
        return BTreeSet::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_features(features: &BTreeSet<String>) {
    let path = features_path();
    if let Ok(content) = serde_json::to_string_pretty(features) {
        let p = path.as_path();
        let tmp = p.with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp, content) {
            log::warn!("[features] failed to write config: {}", e);
        }
        if let Err(e) = std::fs::rename(&tmp, p) {
            log::warn!("[features] failed to rename config: {}", e);
        }
    }
}

pub fn run_features_enable(name: &str) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        log::error!("{}", err("Error: feature name cannot be empty"));
        return;
    }
    let mut features = load_features();
    if features.contains(trimmed) {
        log::info!("  {} feature '{}' is already enabled", info("ℹ"), trimmed);
        return;
    }
    features.insert(trimmed.to_string());
    save_features(&features);
    log::info!("  {} feature '{}' enabled", success("✓"), trimmed);
}

pub fn run_features_list() {
    let features = load_features();
    if features.is_empty() {
        log::info!("  {} No feature flags are currently enabled", info("ℹ"));
        log::info!("");
        log::info!(
            "  Use {} to enable a feature",
            info("neotrix features enable <name>")
        );
        return;
    }
    log::info!("  {} Enabled feature flags:", success("✓"));
    for f in &features {
        log::info!("    • {}", f);
    }
}

// ── Wallet commands ──

pub fn run_wallet_create(label: &str) {
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.persist_wallet(label) {
        Ok(lbl) => {
            if let Some(w) = crypto.wallet_manager.active_wallet() {
                log::info!("{}", success("Wallet created successfully"));
                log::info!("  Label:   {}", lbl);
                log::info!("  Address: {}", w.address);
                log::info!("  Path:    {:?}", crypto.wallet_store.dir_path());
            }
        }
        Err(e) => log::error!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_import(label: &str, private_key: &str) {
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.import_wallet(private_key, label) {
        Ok(w) => {
            log::info!("{}", success("Wallet imported successfully"));
            log::info!("  Label:   {}", w.label);
            log::info!("  Address: {}", w.address);
        }
        Err(e) => log::error!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_list(json: bool) {
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.list_wallets() {
        Ok(wallets) => {
            if json {
                let list: Vec<serde_json::Value> = wallets
                    .iter()
                    .map(|w| {
                        serde_json::json!({
                            "label": w.label, "address": w.address,
                            "chain": w.chain, "created": w.created_at
                        })
                    })
                    .collect();
                log::info!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({"wallets": list}))
                        .unwrap_or_default()
                );
            } else if wallets.is_empty() {
                log::info!(
                    "  {} No wallets found. Use {} to create one.",
                    info("ℹ"),
                    info("neotrix wallet create <label>")
                );
            } else {
                log::info!("  {} Wallets ({})", success("✓"), wallets.len());
                for w in &wallets {
                    let addr_short = if w.address.len() > 12 {
                        format!(
                            "{}...{}",
                            &w.address[..6],
                            &w.address[w.address.len() - 4..]
                        )
                    } else {
                        w.address.clone()
                    };
                    log::info!("    • {} [{}] {}", w.label, w.chain, addr_short);
                }
            }
        }
        Err(e) => log::error!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_balance(chain: &str) {
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    let addr = match crypto.wallet_manager.active_wallet() {
        Some(w) => w.address.clone(),
        None => {
            log::error!(
                "{} No active wallet. Create or import one first.",
                err("Error:")
            );
            return;
        }
    };
    log::info!(
        "  {} Checking balance of {} on {}",
        info("ℹ"),
        &addr[..10],
        chain
    );
}

pub fn run_wallet_delete(label: &str) {
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.delete_persisted_wallet(label) {
        Ok(_) => log::info!("{} Wallet '{}' deleted", success("✓"), label),
        Err(e) => log::error!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_export(label: &str) {
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.load_wallet(label) {
        Ok(w) => {
            log::info!(
                "{}",
                warn("⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!")
            );
            log::info!("");
            let pk = w.private_key_hex();
            log::info!(
                "🔑 {} 私钥 (first 8 chars): {}... (use `neotrix wallet export` for full key)",
                w.label,
                &pk[..8]
            );
        }
        Err(e) => log::error!("{} {}", err("Error:"), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_defaults() {
        let cfg = NeoTrixConfig::load();
        assert!(cfg.provider.is_none());
        assert!(cfg.api_key.is_none());
        assert!(cfg.default_model.is_none());
        assert!(cfg.custom_endpoint.is_none());
        assert!(cfg.color_mode.is_none());
        assert!(cfg.log_level.is_none());
        assert!(cfg.default_llm_provider.is_none());
    }

    #[test]
    fn test_check_provider_config_no_config() {
        assert!(!check_provider_config());
    }

    #[test]
    fn test_brain_dir_default() {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(brain_dir("default"), home.join(".neotrix"));
        assert_eq!(brain_dir(""), home.join(".neotrix"));
    }

    #[test]
    fn test_brain_dir_custom_profile() {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(
            brain_dir("work"),
            home.join(".neotrix").join("profiles").join("work")
        );
    }

    #[test]
    fn test_resolve_prompt_from_arg() {
        assert_eq!(resolve_prompt(Some("hello"), None, false), "hello");
    }

    #[test]
    fn test_resolve_prompt_empty_arg() {
        assert_eq!(resolve_prompt(Some(""), None, false), "");
    }

    #[test]
    fn test_resolve_prompt_arg_precedence() {
        assert_eq!(
            resolve_prompt(Some("explicit"), Some("/nonexistent"), false),
            "explicit"
        );
    }

    #[test]
    fn test_resolve_prompt_no_input() {
        assert_eq!(resolve_prompt(None, None, false), "");
    }

    #[test]
    fn test_config_path_no_panic() {
        let _ = NeoTrixConfig::path();
    }
}
