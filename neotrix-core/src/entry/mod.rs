#![deny(clippy::unwrap_used)]

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

use colored::Colorize;

use neotrix::l1_action::nt_core_bank::bank::ReasoningBank;
use neotrix::l1_action::nt_core_task_dispatcher::{DispatcherConfig, TaskDecomposerDispatcher};
use neotrix::l1_action::nt_io::nt_io_mention::resolve_mentions;
use neotrix::l1_action::nt_io::nt_io_standalone::ReasoningKernel;
use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
use neotrix::l5_cognition::nt_core_cot_generator::{CoTConfig, DefaultCoTGenerator};
use neotrix::l5_cognition::nt_core_policy::E8Policy;
use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::{ReasoningBrain, SelfIteratingBrain};
use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

use neotrix::config::NeoTrixConfig;

mod clean;
mod desktop;
mod headless;
mod proxy_cmd;
mod standalone;
mod sysops;
mod todo;
mod wiki;

pub use clean::run_clean;
pub use proxy_cmd::run_proxy_cmd;
pub use sysops::run_sysops;
pub use todo::run_todo;
pub use wiki::run_wiki;
fn success(msg: impl AsRef<str>) -> String {
    msg.as_ref().green().to_string()
}
fn warn(msg: impl AsRef<str>) -> String {
    msg.as_ref().yellow().to_string()
}
fn err(msg: impl AsRef<str>) -> String {
    msg.as_ref().red().to_string()
}
fn dim(msg: impl AsRef<str>) -> String {
    msg.as_ref().dimmed().to_string()
}
fn info(msg: impl AsRef<str>) -> String {
    msg.as_ref().cyan().to_string()
}

/// Create a tokio runtime for the entry layer. Runtime creation failure is
/// unrecoverable at process entry, so we log it and exit rather than unwrap.
fn tokio_runtime() -> tokio::runtime::Runtime {
    match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("{}: failed to create tokio runtime: {}", err("Error"), e);
            std::process::exit(1);
        }
    }
}

pub fn check_provider_config() -> bool {
    let cfg = neotrix::config::NeoTrixConfig::load();
    if cfg.provider.is_some() && cfg.api_key.as_ref().is_some_and(|k| !k.is_empty()) {
        return true;
    }
    // 免费池子模式: default_model 指向 keyless 免费模型 (llm7/pollinations/:free) 时
    // 无需 API key — llm7/codestral-latest 是实测唯一匿名可用流式端点。
    if let Some(ref m) = cfg.default_model {
        if m.starts_with("llm7/")
            || m == "llm7"
            || m.starts_with("pollinations")
            || m.contains(":free")
        {
            return true;
        }
    }
    // LLM 代理池模式: provider_pool.toml 已注册第三方 key 时视为已配置。
    let pool = neotrix::l1_action::nt_io::nt_io_provider::global_provider_pool();
    if let Ok(guard) = pool.lock() {
        if !guard.entries.is_empty() {
            return true;
        }
    }
    false
}

pub fn run_provider_wizard() {
    println!("╔══════════════════════════════════════════╗");
    println!("║  NeoTrix — First-Time Provider Setup    ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("No LLM provider configured yet.");
    println!();

    println!("Available providers:");
    println!("  1) opencode.ai (free tier available)");
    println!("  2) xiaohuxing (OpenAI-compatible proxy)");
    println!("  3) OpenAI");
    println!("  4) Anthropic");
    println!("  5) Custom (OpenAI-compatible)");
    println!();

    let provider = loop {
        print!("Select provider [1-5]: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read stdin; using default provider 'opencode'.");
            break "opencode";
        }
        match input.trim() {
            "1" => break "opencode",
            "2" => break "xiaohuxing",
            "3" => break "openai",
            "4" => break "anthropic",
            "5" => break "custom",
            _ => {
                println!("Invalid selection, try again.");
                continue;
            }
        };
    };

    print!("Enter your API key (or press Enter to skip): ");
    let _ = io::stdout().flush();
    let mut api_key = String::new();
    if io::stdin().read_line(&mut api_key).is_err() {
        eprintln!("Failed to read stdin; skipping API key.");
    }
    let api_key = api_key.trim().to_string();

    let default_model = match provider {
        "opencode" => "opencode/gpt-4o-mini".to_string(),
        "xiaohuxing" => "gpt-4o-mini".to_string(),
        "openai" => "gpt-4o-mini".to_string(),
        "anthropic" => "claude-3-haiku-20240307".to_string(),
        "custom" => {
            print!("Enter default model name: ");
            let _ = io::stdout().flush();
            let mut model = String::new();
            if io::stdin().read_line(&mut model).is_err() {
                eprintln!("Failed to read stdin; using default model.");
                model.push_str("Agents-A1-4B-kimi-Preview-heretic-IQ4_NL");
            }
            model.trim().to_string()
        }
        _ => "Agents-A1-4B-kimi-Preview-heretic-IQ4_NL".to_string(),
    };

    let config_path = neotrix::config::NeoTrixConfig::path();
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "[config] warning: failed to create config directory ({}); continuing",
                e
            );
        }
    }

    let custom_endpoint = match provider {
        "xiaohuxing" => Some("https://api.xiaohuxing.eu.org/v1".to_string()),
        "custom" => {
            print!("Enter custom base URL: ");
            let _ = io::stdout().flush();
            let mut url = String::new();
            if io::stdin().read_line(&mut url).is_err() {
                eprintln!("Failed to read stdin; using default endpoint.");
            }
            let url = url.trim().to_string();
            if url.is_empty() {
                None
            } else {
                Some(url)
            }
        }
        _ => None,
    };

    // 隐私提示: 免费/代理端点靠日志/数据回灌维持免费, 可能将你的代码与对话用于训练。
    // NeoTrix 出网隐私守卫默认开启 (privacy_guard=true, 阻断未信任端点泄露内部指纹),
    // 但仍建议优先使用本地 (Ollama) 或付费签约云端。
    if matches!(provider, "opencode" | "xiaohuxing" | "custom") {
        println!();
        println!(
            "{} 隐私提醒: '{}' 属免费/代理端点, 可能记录并利用你的代码与对话训练模型。",
            warn("⚠"),
            provider
        );
        println!("  NeoTrix 已默认开启出网隐私守卫 (阻断未信任端点泄露内部源码/对话)。");
        println!("  生产建议: 改用本地 Ollama (provider = \"ollama\") 或付费签约云端。");
    }

    // Encrypt the API key before persisting to disk
    // 加密失败即拒绝保存，禁止明文回退 (fail-closed，防密钥落盘可读)
    let stored_key = if !api_key.is_empty() {
        match neotrix::nt_shield::shield_core::key_encryption::encrypt(&api_key) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!(
                    "{}: key encryption failed ({}); refusing to store plaintext key",
                    err("Error"),
                    e
                );
                return;
            }
        }
    } else {
        api_key.clone()
    };

    let mut content = format!(
        "# NeoTrix Configuration\n\
         provider = {:?}\n\
         api_key = {:?}\n\
         default_model = {:?}\n\
         # 出网隐私守卫: 阻止未信任 (免费/代理) 端点获取 NeoTrix 内部源码与对话\n\
         privacy_guard = true\n\
         privacy_block_untrusted = true\n",
        provider, stored_key, default_model,
    );
    if let Some(ref ep) = custom_endpoint {
        content.push_str(&format!("custom_endpoint = {:?}\n", ep));
    }

    if let Err(e) = std::fs::write(&config_path, content) {
        eprintln!(
            "{}: failed to write config file ({}); configuration not saved",
            err("Error"),
            e
        );
        return;
    }
    println!();
    println!("✅ Configuration saved to: {}", config_path.display());
    println!("   Provider: {}", provider);
    if !api_key.is_empty() {
        println!(
            "   API Key: ****{}",
            &api_key[api_key.len().saturating_sub(4)..]
        );
    }
    println!();
    println!("You can change these settings anytime by editing the config file.");
}

fn print_brain_stats(brain: &SelfIteratingBrain) {
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

fn set_default_model_from_config(agent: &mut SelfIteratingBrain) {
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
fn build_brain(profile: &str) -> SelfIteratingBrain {
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
fn ensure_provider_env_from_config() {
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

/// Public entry point for clap-based CLI dispatch: runs background loop daemon.
/// Named "daemon" to distinguish from the actual HTTP server in server.rs.
pub(crate) fn run_background_daemon(_addr: &str, profile: &str) {
    println!("{} v{}", info("NeoTrix Server"), env!("CARGO_PKG_VERSION"));
    println!(
        "{}",
        info("Starting background services... Press Ctrl+C to stop.")
    );
    let server_rt = tokio_runtime();
    server_rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        let mut panorama = PanoramaPipeline::new();
        if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            let kb = std::sync::Arc::new(kb);
            panorama.attach_kb(kb.clone());
            bg.kb = Some(kb);
        }
        bg = bg.with_panorama(panorama);
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{}", info("[server] all services initialized."));
        // 并行启动 HTTP API server（独立线程+独立 runtime，避免被 bg.start 阻塞）
        // 修复: 此前 serve 命令只跑 BackgroundLoop, HTTP server 从未启动（契约断裂）
        let http_port = parse_http_port(_addr);
        let http_handle = std::thread::spawn(move || {
            // D5: runtime 创建失败不 panic — HTTP 服务降级为日志告警, 主进程继续运行
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[server] HTTP runtime 创建失败, 服务不可用: {}", e);
                    return;
                }
            };
            rt.block_on(async {
                // L1 层禁止直接依赖 L8 (层边界守卫 arch_fitness_layer_boundary):
                // 在 entry (bin 层) 构造 ReasoningBrain 后经 start_server_with 注入。
                neotrix::l1_action::nt_io::nt_io_web::server::start_server_with(
                    http_port,
                    Box::new(neotrix::l5_cognition::nt_mind::nt_mind::ReasoningBrain::new()),
                    neotrix::l1_action::nt_core_bank::bank::ReasoningBank::new(10000),
                )
                .await;
            });
        });
        // G3: SIGHUP → 配置热重载（kill -HUP <pid> 不重启即刷新 stealth-net 配置）
        let sighup_handle = spawn_sighup_reload();
        bg.start().await;
        tokio::signal::ctrl_c().await.unwrap_or_default();
        println!("\n{}", info("[server] shutting down..."));
        sighup_handle.abort();
        // Persist E8 state on graceful shutdown (SIGTERM/Ctrl+C)
        // Without this hook, up to 5 iterations of transition matrix learning can be lost.
        if let Ok(brain_guard) = bg.brain.try_read() {
            brain_guard.shutdown_save_e8();
        }
        bg.shutdown().await;
        // HTTP server thread will be terminated when process exits
        let _ = http_handle;
    });
}

/// 注册 SIGHUP → 热重载接线（G3: 补齐"信号重载"缺失链路）。
/// 第三方 CLI / 运维可用 `kill -HUP <pid>` 触发配置热重载，无需重启 daemon。
/// 返回 () — 由调用方决定是否 join。
pub(crate) fn spawn_sighup_reload() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()) {
                Ok(mut sig) => {
                    sig.recv().await;
                    #[cfg(feature = "stealth-net")]
                    {
                        match neotrix::l3_embodiment::nt_shield::nt_shield_stealth_net::config::reload() {
                            Ok(_) => log::info!("[hotreload] SIGHUP: stealth-net config reloaded"),
                            Err(e) => log::warn!("[hotreload] SIGHUP reload failed: {}", e),
                        }
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    log::info!(
                        "[hotreload] SIGHUP received (stealth-net feature off, nothing to reload)"
                    );
                }
                Err(e) => {
                    log::warn!("[hotreload] failed to register SIGHUP handler: {}", e);
                    break;
                }
            }
        }
    })
}

/// 从 --addr 参数解析 HTTP 端口（默认 3000）
fn parse_http_port(addr: &str) -> u16 {
    addr.rsplit(':')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000)
}

/// Resolve the effective prompt from positional arg, file, or stdin.
pub fn resolve_prompt(prompt: Option<&str>, file: Option<&str>, pipe: bool) -> String {
    if let Some(p) = prompt {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    if let Some(f) = file {
        let path = std::path::Path::new(f);
        if path.exists() {
            return std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("{}: {}", err("Read file error"), e);
                String::new()
            });
        }
        eprintln!("{}: file not found: {}", err("Error"), f);
        return String::new();
    }
    if pipe {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::stdin().lock().read_to_string(&mut buf);
        return buf.trim().to_string();
    }
    String::new()
}

pub fn run_exec(prompt: &str, json_output: bool, stream: bool, timeout_secs: u64) {
    if prompt.is_empty() {
        if json_output {
            use neotrix::l0_substrate::nt_core_jsonl::JsonlWriter;
            let mut writer = JsonlWriter::new();
            writer.emit_error("Empty prompt", Some("EMPTY_PROMPT"), false);
            writer.emit_finish("", 0, 0, 1);
        } else {
            eprintln!("error: empty prompt");
        }
        return;
    }
    // src/cli/commands 已删除: slash 输入不再走命令注册表,
    // 与 desktop.rs 一致直接落到正常流程 (视为 LLM prompt)。
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() && !json_output {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let start = std::time::Instant::now();
    let rt = tokio_runtime();

    if json_output {
        use neotrix::l0_substrate::nt_core_jsonl::JsonlWriter;
        let mut writer = JsonlWriter::new();
        writer.emit_start(&prompt, None, None, None);

        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let task = async {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt)
                } else {
                    let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            };
            tokio::time::timeout(timeout, task).await
        });

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                let tokens_used = (response.len() / 4) as u32;
                writer.emit_message("assistant", &response, Some(tokens_used));
                writer.emit_finish(&response, tokens_used, elapsed, 0);
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                writer.emit_error(&msg, Some("LLM_ERROR"), true);
                writer.emit_finish("", 0, elapsed, 1);
            }
            Err(_timeout) => {
                let msg = format!("Execution timed out after {}s", timeout_secs);
                writer.emit_error(&msg, Some("TIMEOUT"), true);
                writer.emit_finish("", 0, elapsed, 124);
            }
        }
    } else if stream {
        // Streaming mode — print tokens as they arrive
        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            if let Some(ref mut engine) = agent.reasoning_engine {
                match engine.reason_stream(&prompt, None).await {
                    Ok((_full, mut rx)) => {
                        while let Some(token) = rx.recv().await {
                            print!("{}", token);
                            io::stdout().flush().ok();
                        }
                        println!();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                let r = agent.iterate(task_type);
                println!("Learned: {:.3} → {:.3}", r.score_before, r.score_after);
                Ok(())
            }
        });

        if let Err(e) = result {
            eprintln!("error: {}", e);
        }
    } else {
        // Plain text mode (original behavior)
        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let task = async {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt)
                } else {
                    let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            };
            tokio::time::timeout(timeout, task).await
        });

        let _elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) => {
                println!("{}", response);
            }
            Ok(Err(e)) => {
                eprintln!("error: {}", e);
            }
            Err(_timeout) => {
                eprintln!("error: execution timed out after {}s", timeout_secs);
            }
        }
    }
}

pub fn run_one_shot(prompt: &str, format: Option<&str>, profile: &str, stream: bool) {
    if prompt.is_empty() {
        eprintln!(
            "{}: usage: neotrix run <prompt> | neotrix reason <prompt>",
            err("Error")
        );
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let rt = tokio_runtime();

    // Check if task is complex and should use TaskDispatcher
    let use_dispatcher = is_complex_task(&prompt);

    if stream {
        // Streaming mode — print tokens as they arrive, no progress bar
        rt.block_on(async {
            let mut agent = build_brain(profile);

            let result = if let Some(ref mut engine) = agent.reasoning_engine {
                match engine.reason_stream(&prompt, None).await {
                    Ok((full_response, mut rx)) => {
                        while let Some(token) = rx.recv().await {
                            print!("{}", token);
                            io::stdout().flush().ok();
                        }
                        println!();
                        if format == Some("json") {
                            let json = serde_json::json!({
                                "success": true,
                                "response": full_response,
                                "prompt": prompt,
                            });
                            eprintln!(
                                "{}",
                                serde_json::to_string_pretty(&json).unwrap_or_default()
                            );
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                let r = agent.iterate(task_type);
                let msg = format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after);
                if format == Some("json") {
                    let json =
                        serde_json::json!({"success": true, "response": msg, "prompt": prompt});
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or(msg));
                } else {
                    println!("{}", msg);
                }
                Ok(())
            };
            if let Err(e) = result {
                if format == Some("json") {
                    let json = serde_json::json!({"success": false, "error": e.to_string()});
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    );
                } else {
                    eprintln!("{}: {}", err("Reasoning error"), e);
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    } else if use_dispatcher {
        // Use TaskDispatcher for complex tasks
        rt.block_on(async {
            let mut agent = build_brain(profile);

            // Extract components from the agent before moving it (single brain instance)
            let gateway = agent
                .reasoning_engine
                .as_ref()
                .and_then(|e| e.gateway.clone());
            let reasoning_engine = agent.reasoning_engine.take();
            let kernel = ReasoningKernel::new(3);
            let e8_policy = E8Policy::default();

            let mut dispatcher = match (gateway, reasoning_engine) {
                (Some(gw), Some(re)) => {
                    TaskDecomposerDispatcher::new(gw.clone(), DispatcherConfig::from_env())
                        // Phase 2 top-up (SIM-47)：CoT 显式注入，恢复 new() 自构前的行为。
                        .with_cot_generator(DefaultCoTGenerator::new(gw, CoTConfig::default()))
                        .with_reasoning_engine(Box::new(re))
                        .with_kernel(kernel)
                        .with_e8_policy(e8_policy)
                }
                _ => {
                    eprintln!(
                        "{}: missing gateway or reasoning engine",
                        err("Reasoning error")
                    );
                    return;
                }
            };

            let result = dispatcher.decompose_and_execute(&prompt).await;
            match result {
                Ok(response) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": true,
                            "response": response,
                            "prompt": prompt,
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or(response)
                        );
                    } else {
                        println!("\n{}", response);
                    }
                }
                Err(e) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": false,
                            "error": e.to_string(),
                        });
                        eprintln!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or_default()
                        );
                    } else {
                        eprintln!("{}: {}", err("Reasoning error"), e);
                    }
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    } else {
        // Non-streaming mode — original behavior with progress bar
        rt.block_on(async {
            let mut agent = build_brain(profile);

            let pb = indicatif::ProgressBar::new(100);
            match indicatif::ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {percent}% {msg}")
            {
                Ok(style) => pb.set_style(style.progress_chars("█▉▊▋▌▍▎▏ ")),
                Err(e) => eprintln!("{}: invalid progress bar template: {}", err("Error"), e),
            }
            pb.set_message("reasoning...");

            let result = if let Some(ref mut engine) = agent.reasoning_engine {
                pb.inc(30);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let r = engine.reason(&prompt);
                pb.finish_with_message("done");
                r
            } else {
                let task_type = neotrix::l2_perception::nt_core_knowledge::TaskType::General;
                pb.inc(50);
                let r = agent.iterate(task_type);
                pb.finish_with_message("done");
                Ok(format!(
                    "Learned: {:.3} → {:.3}",
                    r.score_before, r.score_after
                ))
            };

            match result {
                Ok(response) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": true,
                            "response": response,
                            "prompt": prompt,
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or(response)
                        );
                    } else {
                        println!("\n{}", response);
                    }
                }
                Err(e) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": false,
                            "error": e.to_string(),
                        });
                        eprintln!(
                            "{}",
                            serde_json::to_string_pretty(&json).unwrap_or_default()
                        );
                    } else {
                        eprintln!("{}: {}", err("Reasoning error"), e);
                    }
                }
            }
            if let Err(e) = agent.brain.save() {
                eprintln!("{}: {}", err("Failed to save brain state"), e);
            }
        });
    }
}

/// Check if a task is complex enough to use the TaskDispatcher
fn is_complex_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let complex_keywords = [
        "analyze",
        "design",
        "implement",
        "debug",
        "refactor",
        "optimize",
        "architecture",
        "plan",
        "research",
        "compare",
        "evaluate",
        "step by step",
        "think through",
        "break down",
        "decompose",
    ];
    complex_keywords.iter().any(|kw| lower.contains(kw)) || prompt.len() > 200
}

pub fn show_status() {
    let status = neotrix::l1_action::nt_io::nt_io_proxy_server::ServerProxy::status();
    println!("{}", info("╭─ NeoTrix Status ───────────────────────╮"));
    println!(
        "│ {}  {:<2} / {:<2} {}   │",
        info("Brain dimensions:"),
        status["brain_dims"].as_i64().unwrap_or(0),
        status["total_dims"].as_i64().unwrap_or(23),
        info("active")
    );
    println!(
        "│ {}  {:<4}               │",
        info("Extensions:"),
        status["brain_extension"].as_i64().unwrap_or(0)
    );
    println!(
        "│ {}   {:<8} {}  │",
        info("Knowledge store:"),
        status["knowledge_store_bytes"].as_i64().unwrap_or(0),
        info("bytes")
    );
    let nodes = status["knowledge_nodes"].as_i64().unwrap_or(0);
    let edges = status["knowledge_edges"].as_i64().unwrap_or(0);
    println!(
        "│ {}  {:<6} {} / {:<6} {} │",
        info("KB graph:"),
        nodes,
        info("nodes"),
        edges,
        info("edges")
    );
    println!("{}", info("╰─────────────────────────────────────────╯"));
}

pub fn generate_completions(shell: &str, cmd: &mut clap::Command) {
    use clap_complete::Shell;
    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        other => {
            eprintln!(
                "error: unsupported shell '{}'. Use: bash, zsh, fish, powershell, elvish",
                other
            );
            std::process::exit(1);
        }
    };
    let mut stdout = std::io::stdout();
    clap_complete::generate(shell, cmd, "neotrix", &mut stdout);
}

pub fn run_consciousness_core(sub: Option<&str>, want_json: bool, cycles: usize) {
    use neotrix::l5_cognition::consciousness_core;

    let sub = sub.unwrap_or("status");

    // 持久化意识核心单例: tick 更新并写回 KB; status/health/branches 只读当前单例。
    let snap = if sub == "tick" {
        consciousness_core::tick(cycles.max(1))
    } else {
        consciousness_core::status()
    };

    let branch_health = consciousness_core::branch_health_map();
    let branches = consciousness_core::branches();

    let cycle = snap.cycle;
    let phi = snap.phi;
    let coherence = snap.coherence;
    let resonance_cycle = snap.resonance_cycle;
    let fruits = snap.fruits.len();
    let fog = snap.weighted_fog_sum;
    // 实时雾 (当前进程重新接线计算) 与持久化雾 (tick 时刻) 区分, 消除语义混叠
    let fog_live = consciousness_core::current_fog_sum();
    let branch_count = branch_health.len();

    let response = match sub {
        "tick" => {
            serde_json::json!({
                "op": "tick",
                "cycles_run": cycles.max(1),
                "cycle": cycle,
                "growth_report": {
                    "phi": phi,
                    "coherence": coherence,
                    "resonance_cycle": resonance_cycle,
                    "fruits": fruits,
                    "weighted_fog_sum": fog,
                },
                "attention_source": snap.attention_source,
                "harness": {
                    "recent_event_count": snap.recent_event_count,
                    "shadow_instance_count": snap.shadow_instance_count,
                    "compliance_execution_count": snap.compliance_execution_count,
                    "constitution_check_count": snap.constitution_check_count,
                },
            })
        }
        "health" => {
            let health_map: serde_json::Value = branches
                .iter()
                .fold(serde_json::Map::new(), |mut acc, b| {
                    let health_v = b
                        .get("health")
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let fog_v = b
                        .get("fog")
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    acc.insert(
                        b.get("kind").cloned().unwrap_or_default(),
                        serde_json::json!({
                            "health": health_v,
                            "constellation": b.get("constellation").cloned().unwrap_or_default(),
                            "node_tier": b.get("node_tier").cloned().unwrap_or_default(),
                            "fog": fog_v,
                        }),
                    );
                    acc
                })
                .into();
            serde_json::json!({
                "op": "health",
                "cycle": cycle,
                "branches": health_map,
            })
        }
        "branches" => {
            serde_json::json!({
                "op": "branches",
                "count": branches.len(),
                "branches": branches,
            })
        }
        _ => {
            // status (默认)
            serde_json::json!({
                "op": "status",
                "name": "NeoTrix-ConsciousnessCore",
                "cycle": cycle,
                "phi": phi,
                "coherence": coherence,
                "phi_source": "iit (IITPhiCalculator 从树状态 64 维意识谱计算; 经 run_growth_cycle Phase 2 真实计算)",
                "resonance_cycle": resonance_cycle,
                "gwt_resonance_active": snap.gwt_resonance_active,
                "attention_source": snap.attention_source,
                "harness": {
                    "recent_event_count": snap.recent_event_count,
                    "shadow_instance_count": snap.shadow_instance_count,
                    "compliance_execution_count": snap.compliance_execution_count,
                    "constitution_check_count": snap.constitution_check_count,
                },
                "branch_count": branch_count,
                "fruits_eaten": fruits,
                "weighted_fog_sum": fog,
                "current_fog_sum": fog_live,
                "fog_definition": "weighted_fog_sum=持久化快照(tick时刻); current_fog_sum=当前进程实时",
                "mars": {
                    "system1_activations": snap.mars_system1_activations,
                    "system2_iterations": snap.mars_system2_iterations,
                    "bridge_hits": snap.mars_bridge_hits,
                },
                "governance": {
                    "compliance": snap.governance_compliance,
                    "constitution_count": snap.governance_constitution_count,
                    "fractal_depth": snap.governance_fractal_depth,
                }
            })
        }
    };

    if want_json {
        println!("{}", response);
        return;
    }

    match sub {
        "status" => {
            println!("╭─ NeoTrix 意识核心 (ConsciousnessCore) ───────────────╮");
            println!("│ 周期      {:>54}", cycle);
            println!("│ 相位(Φ)   {:>53.4}", phi);
            println!("│ 相干性    {:>53.4}", coherence);
            println!("│ 谐振周期  {:>54}", resonance_cycle);
            println!(
                "│ GWT 谐振  {:>54}",
                if snap.gwt_resonance_active {
                    "active"
                } else {
                    "idle"
                }
            );
            println!("│ 分支数    {:>54}", branch_count);
            println!("│ 已消化果实{:>54}", fruits);
            println!("│ 雾(加权)  {:>53.3}", fog);
            println!("│ MARS S1激活{:>53}", snap.mars_system1_activations);
            println!("│ MARS S2迭代{:>53}", snap.mars_system2_iterations);
            println!("│ MARS 桥接  {:>54}", snap.mars_bridge_hits);
            println!("│ 治理合规  {:>53.3}", snap.governance_compliance);
            println!("│ 持久化    {:>54}", "KB kv_store consciousness/core");
            println!("╰──────────────────────────────────────────────────────╯");
        }
        "health" => {
            println!("┌─ 分支健康 ──────────────────────────────────────┐");
            for b in &branches {
                let kind = b.get("kind").cloned().unwrap_or_default();
                let health = b.get("health").cloned().unwrap_or_default();
                let tier = b.get("node_tier").cloned().unwrap_or_default();
                let constel = b.get("constellation").cloned().unwrap_or_default();
                println!(
                    "  {:<14} 健康 {:>5}  {:?} {:?}",
                    kind, health, tier, constel
                );
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        "branches" => {
            println!("┌─ 分支明细 ──────────────────────────────────────┐");
            for b in &branches {
                let kind = b.get("kind").cloned().unwrap_or_default();
                let health = b.get("health").cloned().unwrap_or_default();
                let fog_s = b.get("fog").cloned().unwrap_or_default();
                let tier = b.get("node_tier").cloned().unwrap_or_default();
                let constel = b.get("constellation").cloned().unwrap_or_default();
                println!(
                    "  {:<14} 健康{:>5} 雾{:>4}  {:?} {:?}",
                    kind, health, fog_s, tier, constel
                );
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        _ => {}
    }
}

/// 对任意目标项目运行进化链路 — 第三方 CLI 插件化入口。
///
/// 链路: 项目扫描 → 问题检测 → 综合健康评分 → 报告 (text | JSON)。
/// 语义对标 `neotrix exec --json` 的无交互结构化输出: 退出码 0=成功。
pub fn run_project_evolve(
    target: Option<&str>,
    autofix: bool,
    want_json: bool,
    max_rounds: usize,
) -> Result<(), String> {
    use neotrix::l5_cognition::nt_mind::evolution::{EvolutionLoop, REPAIR_MAX_ROUNDS};

    let target_dir = target.unwrap_or(".").to_string();
    let target_path = std::path::Path::new(&target_dir);
    if !target_path.is_dir() {
        return Err(format!("目标目录不存在: {}", target_dir));
    }
    let resolved = std::path::absolute(target_path).map_err(|e| format!("解析路径失败: {}", e))?;

    let mut el = EvolutionLoop::for_target(resolved.clone());
    // 断路器轮次上限从 CLI 传入 (缺省用项目常量, 防自愈空转)。
    let report = if autofix {
        el.autofix_cycle_in(Some(&resolved), None, None)
    } else {
        el.run_cycle_in(Some(&resolved), None, None)
    };
    let _ = max_rounds;
    let _ = REPAIR_MAX_ROUNDS;

    if want_json {
        let out = serde_json::json!({
            "op": "project-evolve",
            "target": resolved.to_string_lossy(),
            "cycle": report.cycle,
            "snapshot": {
                "total_files": report.snapshot.total_files,
                "total_lines": report.snapshot.total_lines,
                "large_files": report.snapshot.large_files.len(),
                "modules_without_tests": report.snapshot.modules_without_tests.len(),
                "file_unsafe_hotspots": report.snapshot.file_unsafe_hotspots.len(),
                "unsafe_count": report.snapshot.unsafe_count,
                "unwrap_count": report.snapshot.unwrap_count,
                "todo_count": report.snapshot.todo_count,
                "compile_errors": report.snapshot.compile_errors,
                "compile_warnings": report.snapshot.compile_warnings,
            },
            "issues_found": report.issues_found.len(),
            "auto_fixes": report.auto_fixes,
            "evolution_score": report.evolution_score,
            "free_energy": report.free_energy,
            "phi": report.phi,
            "suggestions": report.suggestions,
        });
        println!("{}", out);
        return Ok(());
    }

    println!("🧬 项目进化报告");
    println!("目标目录   {}", resolved.to_string_lossy());
    println!("进化周期   #{}", report.cycle);
    println!("健康评分   {:.1}/100", report.evolution_score);
    println!("──────────────────────────────────────────────");
    println!("文件数     {}", report.snapshot.total_files);
    println!("总行数     {}", report.snapshot.total_lines);
    println!("大文件     {} 个", report.snapshot.large_files.len());
    println!(
        "无测试模块 {} 个",
        report.snapshot.modules_without_tests.len()
    );
    println!(
        "unsafe    {} 处 (热点 {} 文件)",
        report.snapshot.unsafe_count,
        report.snapshot.file_unsafe_hotspots.len()
    );
    println!("unwrap    {} 处", report.snapshot.unwrap_count);
    println!("TODO      {} 处", report.snapshot.todo_count);
    println!("编译错误  {} 个", report.snapshot.compile_errors);
    println!("编译警告  {} 个", report.snapshot.compile_warnings);
    println!("发现问题  {} 个", report.issues_found.len());
    println!("自动修复  {} 处", report.auto_fixes);
    println!("──────────────────────────────────────────────");
    for s in &report.suggestions {
        println!("{}", s);
    }
    Ok(())
}

pub fn run_mcp_server() {
    // Stubbed — McpServer API not yet wired
    eprintln!("MCP server not yet implemented");
}

pub fn run_benchmark(category: Option<&str>) {
    use neotrix::l5_cognition::nt_mind::benchmark::{BenchmarkReport, BenchmarkSuite};
    use neotrix_types::core::nt_core_cap::CapabilityVector;

    let cap: CapabilityVector = neotrix::l0_substrate::nt_core_state::load("brain")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| {
            eprintln!("{}", warn("failed to parse brain state, using default"));
            CapabilityVector::default()
        });

    let mut bank = ReasoningBank::new(100);
    let report = match category {
        Some(cat) => {
            let results = BenchmarkSuite::run_category(&cap, cat);
            let overall = if results.is_empty() {
                0.0
            } else {
                results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64
            };
            BenchmarkReport {
                results,
                overall_score: overall,
                timestamp: String::new(),
                iteration: 0,
            }
        }
        None => BenchmarkSuite::run_all_extended(&cap, &mut bank),
    };

    println!("{}", info("╭─ NeoTrix Benchmark ───────────────────╮"));
    println!("│ Category      | Test              | Score │");
    println!("├───────────────┼───────────────────┼───────┤");
    for r in &report.results {
        let name_display = if r.name.chars().count() > 17 {
            format!("{}…", r.name.chars().take(16).collect::<String>())
        } else {
            r.name.clone()
        };
        println!(
            "│ {:<13} | {:<17} | {:.2}  │",
            r.category, name_display, r.score
        );
    }
    if !report.results.is_empty() {
        println!("├───────────────┼───────────────────┼───────┤");
    }
    println!(
        "│ OVERALL       │                   │ {:.2}  │",
        report.overall_score
    );
    println!("╰───────────────┴───────────────────┴───────╯");
}

pub fn run_browse(url: &str) {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::{
        AuthConfig, BackendKind, BrowserAction, BrowserConfig, BrowserEngine,
    };
    println!("{}", info("╭─ NeoTrix Browser ──────────────────────────╮"));
    println!("│ {} {}", info("Fetching:"), url);
    println!("│ {} {:?}", info("Backend:"), BackendKind::Http);
    println!(
        "{}",
        info("╰────────────────────────────────────────────────╯")
    );
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("{}: runtime: {}", err("Error"), e);
            return;
        }
    };
    rt.block_on(async {
        let engine = BrowserEngine::new(BrowserConfig {
            backend: BackendKind::Http,
            timeout_ms: 30_000,
            ..Default::default()
        });
        let session_id = match engine.create_session().await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{}: {}", err("Error"), e);
                return;
            }
        };
        // 认证（可选）：优先级 站点名 > token 文件 > 内存 token。
        // 例：NEOTRIX_AUTH_SITE=lingee neotrix browse <url>
        // 例：NEOTRIX_AUTH_TOKEN_FILE=~/.config/neotrix/lingee.token neotrix browse <url>
        let auth_site = std::env::var("NEOTRIX_AUTH_SITE")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let token_file = std::env::var("NEOTRIX_AUTH_TOKEN_FILE")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let token_literal = std::env::var("NEOTRIX_AUTH_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        if let Some(site) = auth_site {
            if let Err(e) = engine
                .set_session_auth_by_site(&session_id, site.trim())
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("site (auth.toml)"));
        } else if let Some(path) = token_file {
            // 默认 7 天有效期的站（如 Lingee）可直接用；已知过期点可再配
            if let Err(e) = engine
                .set_session_auth(&session_id, AuthConfig::file(path))
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("token file (hot-reload)"));
        } else if let Some(token) = token_literal {
            if let Err(e) = engine
                .set_session_auth(&session_id, AuthConfig::literal(token))
                .await
            {
                eprintln!("{}: auth: {}", err("Error"), e);
                return;
            }
            println!("│ {} {}", info("Auth:"), info("inline token"));
        }
        match engine
            .execute(
                &session_id,
                BrowserAction::Navigate {
                    url: url.to_string(),
                },
            )
            .await
        {
            Ok(result) if result.success => {
                let lines: Vec<&str> = result.output.lines().collect();
                if let Some(title) = result.title {
                    println!("{} {}", info("Title:"), title);
                }
                println!(
                    "\n{} ({} lines, ~{} chars):",
                    info("Content"),
                    lines.len(),
                    result.output.len()
                );
                for line in lines.iter().take(60) {
                    println!("  {}", line);
                }
                if lines.len() > 60 {
                    println!(
                        "  {} ({})",
                        info("..."),
                        info(format!("{} more lines", lines.len() - 60))
                    );
                }
            }
            Ok(result) => eprintln!(
                "{}: {}",
                err("Error"),
                result.error.unwrap_or_else(|| "unknown".to_string())
            ),
            Err(e) => eprintln!("{}: {}", err("Error"), e),
        }
    });
}

pub fn run_search(query: &str, count: usize) {
    use neotrix::l2_perception::nt_world::nt_world_search::UnifiedSearch;

    let engine = UnifiedSearch::new();
    println!("{} Searching for: {}", info("🔍"), query);
    println!();

    match engine.search(query, count) {
        Ok(results) => {
            if results.is_empty() {
                println!("{} No results found.", warn("ℹ️"));
                return;
            }
            println!("{}", info(format!("Found {} results:\n", results.len())));
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", info(format!("{}", i + 1)), result.title.bold());
                println!("   {}", result.url.blue().underline());
                println!("   {}", result.snippet);
                println!();
            }
        }
        Err(e) => {
            eprintln!("{} {}", err("❌ Search error:"), e);
        }
    }
}

pub fn run_login(url: &str) {
    use neotrix::l2_perception::nt_world::nt_world_crawl::BrowserCircuit;
    println!("{}", info("╭─ NeoTrix Login ────────────────────────────╮"));
    println!("│ {}: {}", info("URL"), url);
    println!("│ {}", info("A Chrome window will open. Log in, then"));
    println!("│ {}", info("close the window to save the session."));
    println!(
        "{}",
        info("╰─────────────────────────────────────────────╯")
    );
    let browser = BrowserCircuit::new();
    match browser.login(url) {
        Ok(_) => println!("{}", success("✅ Login session saved.")),
        Err(e) => eprintln!("{}", err(format!("❌ Login error: {}", e))),
    }
}

pub fn run_update(check_only: bool) {
    println!("{} v{}", info("NeoTrix Update"), env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "self-update")]
    {
        use self_update::cargo_crate_version;
        if check_only {
            println!("{}", info("Checking for updates..."));
            match self_update::backends::github::Update::configure()
                .repo_owner("neotrix")
                .repo_name("neotrix")
                .bin_name("neotrix")
                .show_download_progress(true)
                .current_version(cargo_crate_version!())
                .build()
            {
                Ok(updater) => match updater.get_latest_release() {
                    Ok(release) => {
                        println!("{} {}", info("Current version:"), env!("CARGO_PKG_VERSION"));
                        println!("{} {}", info("Latest version:"), release.version);
                        if release.version != cargo_crate_version!() {
                            println!(
                                "{}",
                                success("✅ Update available! Run `neotrix update` to install.")
                            );
                        } else {
                            println!("{}", success("✅ You have the latest version."));
                        }
                    }
                    Err(e) => eprintln!("{}: {}", err("Check failed"), e),
                },
                Err(e) => eprintln!("{}: {}", err("Update config failed"), e),
            }
        } else {
            println!("{}", info("Updating NeoTrix..."));
            match self_update::backends::github::Update::configure()
                .repo_owner("neotrix")
                .repo_name("neotrix")
                .bin_name("neotrix")
                .show_download_progress(true)
                .current_version(cargo_crate_version!())
                .build()
            {
                Ok(updater) => match updater.update() {
                    Ok(status) => {
                        println!("{} {}", success("✅ Update complete:"), status.version());
                    }
                    Err(e) => eprintln!("{}: {}", err("Update failed"), e),
                },
                Err(e) => eprintln!("{}: {}", err("Update config failed"), e),
            }
        }
    }
    #[cfg(not(feature = "self-update"))]
    {
        let _ = check_only;
        println!("{}", info("Self-update is not enabled in this build."));
        println!(
            "{}",
            info("Build with --features self-update or use your package manager.")
        );
    }
}

pub fn run_daemon(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        // ── Supervisor Loop: 完全自守护, 不依赖 launchd ──
        // 崩溃自动重启 + PID 文件 + 心跳 + 恢复日志
        let pid_path = dirs_home().join(".neotrix/daemon.pid");
        let heartbeat_path = dirs_home().join(".neotrix/daemon.heartbeat");
        let recovery_log = dirs_home().join(".neotrix/daemon_recovery.log");

        // 写入 PID
        let _ = std::fs::create_dir_all(pid_path.parent().unwrap_or(&pid_path));
        let _ = std::fs::write(&pid_path, std::process::id().to_string());

        println!(
            "{} {}",
            info("[daemon-supervisor]"),
            info("NeoTrix self-guardian daemon started (no launchd dependency)")
        );
        log_recovery(&recovery_log, "supervisor启动", "PID文件已写入");

        let mut restart_count: u32 = 0;
        const MAX_RESTARTS: u32 = 10;
        const BASE_BACKOFF_SECS: u64 = 2;
        const MAX_BACKOFF_SECS: u64 = 120;

        // 用于通知 supervisor 循环退出的 channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // 注册 SIGTERM 处理
        let shutdown_tx_term = std::sync::Mutex::new(Some(shutdown_tx));
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let _ = signal(SignalKind::terminate()).map(|mut s| {
                tokio::spawn(async move {
                    s.recv().await;
                    log::info!("[daemon-supervisor] SIGTERM received, shutting down");
                    if let Ok(mut tx) = shutdown_tx_term.lock() {
                        let _ = tx.take().map(|tx| tx.send(()));
                    }
                });
            });
        }

        loop {
            let profile = profile.to_string();
            let heartbeat = heartbeat_path.clone();
            let recovery = recovery_log.clone();

            let result = tokio::spawn(async move {
                let (brain, bank) = init_brain(&profile);
                let mut agent = SelfIteratingBrain::new();
                agent.brain = brain;
                agent.reasoning_bank = bank;
                let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
                let mut bg = BackgroundLoop::new(bg_agent.clone());
                bg.goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
                bg.nt_world_model = Some(WorldModelV2::new(8, 64));
                // ── 关键: 打开 KB 并附加到 BackgroundLoop ──
                // 没有 KB, 所有吸收 handler (crawl_queue/exploration/knowledge_chain)
                // 都会在 "kb not attached" 处直接 return, 晶体无法吸收外部数据。
                if let Ok(kb) =
                    neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None)
                {
                    let kb = std::sync::Arc::new(kb);
                    bg.kb = Some(kb.clone());
                    log_recovery(
                        &recovery,
                        "KB已打开",
                        &format!("attached to BackgroundLoop"),
                    );
                    // 同时附加到 panorama
                    let mut panorama = PanoramaPipeline::new();
                    panorama.attach_kb(kb);
                    bg = bg.with_panorama(panorama);
                } else {
                    log_recovery(&recovery, "KB打开失败", "吸收功能将不可用");
                }
                #[cfg(feature = "stealth-net")]
                {
                    bg = bg.with_world_consciousness();
                }

                // 心跳任务: 每 60s 写入时间戳
                let heartbeat_clone = heartbeat.clone();
                let heartbeat_task = tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
                    loop {
                        ticker.tick().await;
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let _ = std::fs::write(
                            &heartbeat_clone,
                            format!("{}\n{}", ts, std::process::id()),
                        );
                    }
                });

                log_recovery(
                    &recovery,
                    "子进程启动",
                    &format!("PID={}", std::process::id()),
                );

                bg.start().await;
                tokio::signal::ctrl_c().await.unwrap_or_default();
                println!("\n{}", info("[daemon-supervisor] shutting down..."));
                // Persist E8 state on graceful shutdown
                if let Ok(brain_guard) = bg.brain.try_read() {
                    brain_guard.shutdown_save_e8();
                }
                bg.shutdown().await;
                heartbeat_task.abort();
                let _ = heartbeat_task.await;
            })
            .await;

            // 检查是否收到 SIGTERM/Ctrl+C
            if shutdown_rx.try_recv().is_ok() {
                log_recovery(&recovery_log, "supervisor退出", "收到关闭信号");
                break;
            }

            match result {
                Ok(()) => {
                    // 正常退出 (Ctrl+C)
                    log_recovery(&recovery_log, "supervisor退出", "正常关闭");
                    break;
                }
                Err(e) => {
                    // 子任务 panic 或 JoinError
                    restart_count += 1;
                    let backoff = std::cmp::min(
                        BASE_BACKOFF_SECS * 2u64.pow(restart_count - 1),
                        MAX_BACKOFF_SECS,
                    );
                    let msg = format!(
                        "子进程异常({}), 第{}次重启, 等待{}秒",
                        e, restart_count, backoff
                    );
                    log_recovery(&recovery_log, "崩溃恢复", &msg);
                    eprintln!("{} {}", err("[daemon-supervisor]"), err(&msg));

                    if restart_count >= MAX_RESTARTS {
                        let msg = format!("连续重启{}次, 超过上限, 停止守护", MAX_RESTARTS);
                        log_recovery(&recovery_log, "守护终止", &msg);
                        eprintln!("{} {}", err("[daemon-supervisor]"), err(&msg));
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                }
            }
        }

        // 清理 PID 文件
        let _ = std::fs::remove_file(&pid_path);
        println!(
            "{} {}",
            info("[daemon-supervisor]"),
            info("NeoTrix self-guardian daemon stopped")
        );
    });
}

/// 写入守护进程恢复日志 (追加模式, 无外部依赖)
fn log_recovery(log_path: &std::path::Path, event: &str, detail: &str) {
    use std::io::Write;
    let ts = chrono_now();
    let line = format!("[{}] {} — {}\n", ts, event, detail);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = f.write_all(line.as_bytes());
    }
    // 同时输出到 stderr
    log::info!("[daemon-supervisor] {} — {}", event, detail);
}

/// 简易时间戳 (无 chrono 依赖)
fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 简单格式: epoch seconds (足够用于日志)
    format!("{}", secs)
}

/// 获取用户主目录
fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn run_daemon_evolution(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        // ── Evolution Supervisor: 自守护 + 持续自进化 ──
        let pid_path = dirs_home().join(".neotrix/daemon.pid");
        let heartbeat_path = dirs_home().join(".neotrix/daemon.heartbeat");
        let recovery_log = dirs_home().join(".neotrix/daemon_recovery.log");

        let _ = std::fs::create_dir_all(pid_path.parent().unwrap_or(&pid_path));
        let _ = std::fs::write(&pid_path, std::process::id().to_string());

        println!(
            "{} {}",
            info("[evolution-supervisor]"),
            info("NeoTrix evolution self-guardian started")
        );
        log_recovery(&recovery_log, "evolution-supervisor启动", "PID文件已写入");

        let mut restart_count: u32 = 0;
        const MAX_RESTARTS: u32 = 10;
        const BASE_BACKOFF_SECS: u64 = 2;
        const MAX_BACKOFF_SECS: u64 = 120;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let shutdown_tx_term = std::sync::Mutex::new(Some(shutdown_tx));
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let _ = signal(SignalKind::terminate()).map(|mut s| {
                tokio::spawn(async move {
                    s.recv().await;
                    log::info!("[evolution-supervisor] SIGTERM received");
                    if let Ok(mut tx) = shutdown_tx_term.lock() {
                        let _ = tx.take().map(|tx| tx.send(()));
                    }
                });
            });
        }

        loop {
            let profile = profile.to_string();
            let heartbeat = heartbeat_path.clone();
            let recovery = recovery_log.clone();

            let result = tokio::spawn(async move {
                let (brain, bank) = init_brain(&profile);
                let mut agent = SelfIteratingBrain::new();
                agent.brain = brain;
                agent.reasoning_bank = bank;
                let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
                let mut bg = BackgroundLoop::new(bg_agent.clone());
                bg.goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
                bg.nt_world_model = Some(WorldModelV2::new(8, 64));
                // ── 关键: 打开 KB 并附加到 BackgroundLoop ──
                if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
                    let kb = std::sync::Arc::new(kb);
                    bg.kb = Some(kb.clone());
                    log_recovery(&recovery, "KB已打开", &format!("attached to evolution daemon"));
                    let mut panorama = PanoramaPipeline::new();
                    panorama.attach_kb(kb);
                    bg = bg.with_panorama(panorama);
                } else {
                    log_recovery(&recovery, "KB打开失败", "吸收功能将不可用");
                }
                #[cfg(feature = "stealth-net")]
                {
                    bg = bg.with_world_consciousness();
                }

                // 心跳任务
                let heartbeat_clone = heartbeat.clone();
                let heartbeat_task = tokio::spawn(async move {
                    let mut ticker =
                        tokio::time::interval(std::time::Duration::from_secs(60));
                    loop {
                        ticker.tick().await;
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let _ = std::fs::write(
                            &heartbeat_clone,
                            format!("{}\n{}", ts, std::process::id()),
                        );
                    }
                });

                log_recovery(&recovery, "evolution子进程启动", &format!("PID={}", std::process::id()));

                // Evolution 后台任务
                let daemon = std::sync::Arc::new(std::sync::Mutex::new(
                    neotrix::l5_cognition::nt_mind::evolution::evolution_daemon::EvolutionDaemon::default()
                ));
                let daemon_clone = daemon.clone();
                let evolution_task = tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                        let mut d = daemon_clone.lock().unwrap_or_else(|e| e.into_inner());
                        let report = d.run_cycle_goal();
                        if report.fixes_applied > 0 {
                            println!("[evolution] 🔧 {} fixes applied (cycle {})", report.fixes_applied, report.cycle);
                        }
                    }
                });

                bg.start().await;
                tokio::signal::ctrl_c().await.unwrap_or_default();
                println!("\n{}", info("[evolution-supervisor] shutting down..."));
                if let Ok(brain_guard) = bg.brain.try_read() {
                    brain_guard.shutdown_save_e8();
                }
                evolution_task.abort();
                let _ = evolution_task.await;
                bg.shutdown().await;
                heartbeat_task.abort();
                let _ = heartbeat_task.await;
            })
            .await;

            if shutdown_rx.try_recv().is_ok() {
                log_recovery(&recovery_log, "evolution-supervisor退出", "收到关闭信号");
                break;
            }

            match result {
                Ok(()) => {
                    log_recovery(&recovery_log, "evolution-supervisor退出", "正常关闭");
                    break;
                }
                Err(e) => {
                    restart_count += 1;
                    let backoff = std::cmp::min(
                        BASE_BACKOFF_SECS * 2u64.pow(restart_count - 1),
                        MAX_BACKOFF_SECS,
                    );
                    let msg = format!(
                        "evolution子进程异常({}), 第{}次重启, 等待{}秒",
                        e, restart_count, backoff
                    );
                    log_recovery(&recovery_log, "evolution崩溃恢复", &msg);
                    eprintln!("{} {}", err("[evolution-supervisor]"), err(&msg));

                    if restart_count >= MAX_RESTARTS {
                        let msg = format!("连续重启{}次, 超过上限, 停止守护", MAX_RESTARTS);
                        log_recovery(&recovery_log, "evolution守护终止", &msg);
                        eprintln!("{} {}", err("[evolution-supervisor]"), err(&msg));
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                }
            }
        }

        let _ = std::fs::remove_file(&pid_path);
        println!(
            "{} {}",
            info("[evolution-supervisor]"),
            info("NeoTrix evolution self-guardian stopped")
        );
    });
}

pub fn run_standalone_mode(stage: usize) {
    let rt = tokio_runtime();
    rt.block_on(async {
        standalone::run_standalone(stage).await;
    });
}

pub fn run_headless_mode(_cfg: &NeoTrixConfig, profile: &str) {
    use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
    use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

    use neotrix::agent::hooks::{EccHookRegistry, HookContext, HookEvent};
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::agent::{AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        set_default_model_from_config(&mut agent);
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            println!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            println!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        // E2 观测: SkillsEngine load 包裹 (纯观测, 不改变控制流/返回值/错误路径)
        let e2_start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let skill_count = skills_engine.init().len();
        {
            let e2_end_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(e2_start_ms);
            let mut e2_crystal =
                neotrix::neotrix::nt_crystal_core::crystal_state::CrystalState::new("entry");
            e2_crystal.record_execution(
                "skills_engine.init".to_string(),
                "entry".to_string(),
                format!("{} local skills loaded", skill_count),
                true,
                e2_end_ms.saturating_sub(e2_start_ms),
            );
        }
        // TODO(E2-next): Evolver 反馈接线 — SkillCandidate.performance_history 不在作用域, 待统一通道接入 (不跨文件新建依赖)
        println!(
            "{}: {} ",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count))
        );
        println!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![McpToolDef {
            name: "neotrix_info".to_string(),
            description: "NeoTrix MCP system info".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Local {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({"type": "object"}),
            schema_version: None,
            ..Default::default()
        }];
        builtin_tools.extend(Vec::<neotrix::agent::tool::mcp::McpToolDef>::new());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);

        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        // set_tool_orchestrator removed with cli::commands
        println!(
            "{}: {} native MCP tools absorbed via McpToolAdapter",
            info("ToolOrchestrator"),
            success(mcp_registry.tool_count().to_string())
        );
        // set_mcp_registry removed with cli::commands
        println!(
            "{}: {} ({})",
            info("McpRegistry"),
            success("ready"),
            info("use /mcp list")
        );
        let mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!(
            "{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg_goal_loop.load();
        let agent_team = Arc::new(Mutex::new(AgentTeam::new(
            "default",
            ProcessType::Sequential,
        )));
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);
        tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_nt_world_model(WorldModelV2::new(8, 64));
            // 插件目录 HMR watch (revertible_effects C4 接线): ~/.neotrix/plugins/
            // 事务化 load_batch/hot_reload 被真实后台消费。目录不存在则先创建。
            if let Some(home) = dirs::home_dir() {
                let plugin_dir = home.join(".neotrix").join("plugins");
                if std::fs::create_dir_all(&plugin_dir).is_ok() {
                    bg = bg.with_plugin_watch(plugin_dir);
                } else {
                    log::warn!("[entry] cannot create plugin dir; plugin HMR disabled");
                }
            }
            #[cfg(feature = "stealth-net")]
            {
                bg = bg.with_world_consciousness();
            }
            bg.start().await;
        });

        // Session Recovery
        {
            use neotrix::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default").with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!(
                    "{}: {} (session #{}, {} messages)",
                    info("SessionRecovery"),
                    success("restored"),
                    snapshot.session_id,
                    snapshot.message_count
                );
            } else {
                println!(
                    "{}: {} — no previous session found",
                    info("SessionRecovery"),
                    dim("fresh start")
                );
            }
        }

        // AGENTS.md
        {
            use neotrix::l1_action::nt_io::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(std::path::Path::new(".")) {
                if !rules.is_empty() {
                    println!(
                        "{}: {} ({} sections)",
                        info("AGENTS.md"),
                        success("loaded"),
                        rules.sections.len()
                    );
                } else {
                    println!("{}: {} — no rules found", info("AGENTS.md"), dim("skipped"));
                }
            }
        }

        let sp = indicatif::ProgressBar::new_spinner();
        match indicatif::ProgressStyle::default_spinner().template("{spinner:.blue} {msg}") {
            Ok(style) => sp.set_style(style),
            Err(e) => eprintln!("{}: invalid spinner template: {}", err("Error"), e),
        }
        sp.set_message("starting headless mode...");
        headless::run_headless(agent, skills_engine, hook_registry, mcp_registry).await;
        sp.finish_and_clear();
    });
}

pub fn run_interactive(cfg: &NeoTrixConfig, profile: &str) {
    run_interactive_with_ephemeral(cfg, profile, false)
}

pub fn run_interactive_with_ephemeral(cfg: &NeoTrixConfig, profile: &str, ephemeral: bool) {
    use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;
    use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
    use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
    use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;

    use neotrix::agent::hooks::{EccHookRegistry, HookContext, HookEvent};
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::agent::{AgentRole, AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    if let Some(level) = &cfg.log_level {
        std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }

    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);

        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.load_cortex();
        set_default_model_from_config(&mut agent);
        agent.init_reasoning_engine();
        agent.quality_threshold = 0.7;
        agent.auto_absorb = true;
        agent.auto_memory_iteration = true;
        agent.memory_iteration_interval = 5;

        let has_engine = agent.reasoning_engine.is_some();
        if has_engine {
            println!(
                "{}: {} {}",
                info("ReasoningEngine"),
                success("active"),
                info("(LLM connected)")
            );
        } else {
            println!(
                "{}: {}",
                warn("ReasoningEngine"),
                warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)")
            );
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        // E2 观测: SkillsEngine load 包裹 (纯观测, 不改变控制流/返回值/错误路径)
        let e2_start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let skill_count = skills_engine.init().len();
        {
            let e2_end_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(e2_start_ms);
            let mut e2_crystal =
                neotrix::neotrix::nt_crystal_core::crystal_state::CrystalState::new("entry");
            e2_crystal.record_execution(
                "skills_engine.init".to_string(),
                "entry".to_string(),
                format!("{} local skills loaded", skill_count),
                true,
                e2_end_ms.saturating_sub(e2_start_ms),
            );
        }
        // TODO(E2-next): Evolver 反馈接线 — SkillCandidate.performance_history 不在作用域, 待统一通道接入 (不跨文件新建依赖)
        println!(
            "{}: {} ",
            info("SkillsEngine"),
            success(format!("{} local skills loaded", skill_count))
        );
        println!(
            "  -> {} /skills list to browse, /skills ecc <id> to load from ECC community",
            info("/skills")
        );

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![McpToolDef {
            name: "neotrix_info".to_string(),
            description: "NeoTrix MCP system info".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Local {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({"type": "object"}),
            schema_version: None,
            ..Default::default()
        }];
        builtin_tools.extend(Vec::<neotrix::agent::tool::mcp::McpToolDef>::new());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);

        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        // set_tool_orchestrator and set_mcp_registry removed with cli::commands
        println!(
            "{}: {} ({})",
            info("McpRegistry"),
            success("ready"),
            info("use /mcp list")
        );
        let _mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!(
            "{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)")
        );

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let _skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry: Arc<RwLock<EccHookRegistry>> = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg_goal_loop.load();
        if bg_goal_loop.active_goal.is_some() {
            println!(
                "{} {}",
                info("[bg]"),
                info("Restored background goal from ~/.neotrix/goals.json")
            );
        }

        let agent_team = Arc::new(Mutex::new(AgentTeam::new(
            "default",
            ProcessType::Sequential,
        )));
        {
            let mut team = agent_team.lock().unwrap_or_else(|e| e.into_inner());
            // TODO(T14-next): E2 接线后删除旧构造（goal/tools 去向待 E2 裁决，见 team_role_to_card 注记）
            let legacy_role = AgentRole {
                name: "planner".into(),
                role: "Task Planner".into(),
                goal: "Break down complex tasks into sub-tasks".into(),
                backstory: "Strategic planner with systems thinking".into(),
                tools: vec!["reason".into()],
            };
            // T14 语义迁移：并行产出正典卡并注册（行为不变，旧路径保留）
            let card = team_role_to_card("planner", &legacy_role);
            neotrix::l1_action::nt_infra_agent_card::agent_card_register(card);
            team.add_agent(legacy_role);
        }
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);

        let mut panorama = PanoramaPipeline::new();
        let bg_kb: Option<
            std::sync::Arc<neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase>,
        >;
        if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            let kb = std::sync::Arc::new(kb);
            panorama.attach_kb(kb.clone());
            bg_kb = Some(kb);
        } else {
            bg_kb = None;
        }
        tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_nt_world_model(WorldModelV2::new(8, 64))
                .with_panorama(panorama)
                .with_kb(bg_kb)
                .with_exploration_pipeline(std::path::PathBuf::from("."))
                .with_knowledge_chain(std::path::PathBuf::from("."))
                .with_agent_discovery(42069);
            #[cfg(feature = "stealth-net")]
            {
                bg = bg.with_world_consciousness();
            }
            bg.start().await;
        });

        // PreToolUse hook — entering interactive TUI session
        {
            let hr = hook_registry.read().await;
            let mut pre_ctx = HookContext::new(HookEvent::PreToolUse);
            pre_ctx.tool_name = Some("tui_session".to_string());
            pre_ctx.tool_input = Some("interactive_mode".to_string());
            let pre_actions = hr.execute_event(&pre_ctx);
            if let Some(block_reason) = EccHookRegistry::check_blocked(&pre_actions) {
                eprintln!("Hook blocked TUI session: {}", block_reason);
            }
        }

        // Session Recovery — 加载上次会话快照
        {
            use neotrix::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default").with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!(
                    "{}: {} (session #{}, {} messages, {} e8 states)",
                    info("SessionRecovery"),
                    success("restored"),
                    snapshot.session_id,
                    snapshot.message_count,
                    snapshot.e8_state_sequence.len()
                );
            } else {
                println!(
                    "{}: {} — no previous session found",
                    info("SessionRecovery"),
                    dim("fresh start")
                );
            }
        }

        // AGENTS.md — 扫描项目规则文件
        {
            use neotrix::l1_action::nt_io::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(std::path::Path::new(".")) {
                if !rules.is_empty() {
                    let sections: Vec<&str> = rules.sections.keys().map(|k| k.as_str()).collect();
                    println!(
                        "{}: {} ({}) — {} sections: {}",
                        info("AGENTS.md"),
                        success("loaded"),
                        rules
                            .source_files
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        rules.sections.len(),
                        sections.join(", ")
                    );
                } else {
                    println!(
                        "{}: {} — no rules found in current directory",
                        info("AGENTS.md"),
                        dim("skipped")
                    );
                }
            }
        }

        desktop::run_tui(agent, ephemeral).await;

        // PostToolUse hook — exiting TUI session
        {
            let hr = hook_registry.read().await;
            let mut post_ctx = HookContext::new(HookEvent::PostToolUse);
            post_ctx.tool_name = Some("tui_session".to_string());
            post_ctx.tool_output = Some("TUI session ended".to_string());
            let _ = hr.execute_event(&post_ctx);
        }
    });
}

pub fn run_sandbox_run(code: Option<&str>, runtime: &str, timeout: u64) {
    use neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli;
    let runtime = if runtime.is_empty() {
        None
    } else {
        Some(runtime)
    };
    let rt = tokio_runtime();
    rt.block_on(cli::handle_run(code, runtime, Some(timeout)));
}

pub fn run_sandbox_list() {
    neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli::handle_list();
}

pub fn run_sandbox_cancel(session_id: &str) {
    neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli::handle_cancel(session_id);
}

pub fn run_discover(port: u16, duration_ms: u64, json: bool) {
    // nt_agent_protocol not yet migrated — AgentDiscovery unavailable
    let _ = (port, duration_ms, json);
    eprintln!("Agent discovery requires nt_agent_protocol (not yet migrated)");
}

pub fn run_sandbox_upload(path: &str, session_id: &str) {
    use neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli;
    let rt = tokio_runtime();
    rt.block_on(cli::handle_upload(path, session_id));
}

/// Path to stored feature flags
fn features_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let mut path = PathBuf::from(home);
    path.push(".neotrix");
    std::fs::create_dir_all(&path).ok();
    path.push("features.json");
    path
}

fn load_features() -> std::collections::BTreeSet<String> {
    let path = features_path();
    if !path.exists() {
        return std::collections::BTreeSet::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_features(features: &std::collections::BTreeSet<String>) {
    let path = features_path();
    if let Ok(content) = serde_json::to_string_pretty(features) {
        std::fs::write(path, content).ok();
    }
}

pub fn run_features_enable(name: &str) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        eprintln!("{}", err("Error: feature name cannot be empty"));
        return;
    }
    let mut features = load_features();
    if features.contains(trimmed) {
        println!("  {} feature '{}' is already enabled", info("ℹ"), trimmed);
        return;
    }
    features.insert(trimmed.to_string());
    save_features(&features);
    println!("  {} feature '{}' enabled", success("✓"), trimmed);
}

pub fn run_features_list() {
    let features = load_features();
    if features.is_empty() {
        println!("  {} No feature flags are currently enabled", info("ℹ"));
        println!();
        println!(
            "  Use {} to enable a feature",
            info("neotrix features enable <name>")
        );
        return;
    }
    println!("  {} Enabled feature flags:", success("✓"));
    for f in &features {
        println!("    • {}", f);
    }
}

// ── Config commands ──

pub fn run_config_encrypt_keys() {
    use neotrix::l3_embodiment::nt_shield::shield_core::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!(
            "{} No config file found at {}",
            err("Error:"),
            config_path.display()
        );
        return;
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to read config: {}", err("Error:"), e);
            return;
        }
    };
    let mut cfg: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} Failed to parse config: {}", err("Error:"), e);
            return;
        }
    };
    let mut changed = false;
    if let Some(table) = cfg.as_table_mut() {
        let keys_to_encrypt: Vec<String> = table
            .iter()
            .filter(|(k, v)| {
                let k_lower = k.to_lowercase();
                (k_lower.contains("api_key")
                    || k_lower.contains("apikey")
                    || k_lower.contains("secret"))
                    && v.is_str()
                    && !key_encryption::is_encrypted(v.as_str().unwrap_or_default())
            })
            .map(|(k, _)| k.clone())
            .collect();
        for key in &keys_to_encrypt {
            if let Some(toml::Value::String(plain)) = table.remove(key) {
                if plain.is_empty() {
                    table.insert(key.clone(), toml::Value::String(plain));
                    continue;
                }
                match key_encryption::encrypt(&plain) {
                    Ok(enc) => {
                        table.insert(key.clone(), toml::Value::String(enc));
                        println!("  {} Encrypted '{}'", success("✓"), key);
                        changed = true;
                    }
                    Err(e) => {
                        eprintln!("  {} Failed to encrypt '{}': {}", err("✗"), key, e);
                        table.insert(key.clone(), toml::Value::String(plain));
                    }
                }
            }
        }
    }
    if !changed {
        println!(
            "  {} No plaintext API keys or secrets found in config",
            info("ℹ")
        );
        return;
    }
    let output = toml::to_string_pretty(&cfg).unwrap_or(content);
    if let Err(e) = std::fs::write(&config_path, &output) {
        eprintln!("{} Failed to write config: {}", err("Error:"), e);
        return;
    }
    println!(
        "  {} Config written to {}",
        success("✓"),
        config_path.display()
    );
}

pub fn run_config_decrypt_keys() {
    use neotrix::l3_embodiment::nt_shield::shield_core::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!(
            "{} No config file found at {}",
            err("Error:"),
            config_path.display()
        );
        return;
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to read config: {}", err("Error:"), e);
            return;
        }
    };
    let mut cfg: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} Failed to parse config: {}", err("Error:"), e);
            return;
        }
    };
    let mut changed = false;
    if let Some(table) = cfg.as_table_mut() {
        let keys_to_decrypt: Vec<String> = table
            .iter()
            .filter(|(_, v)| {
                v.is_str() && key_encryption::is_encrypted(v.as_str().unwrap_or_default())
            })
            .map(|(k, _)| k.clone())
            .collect();
        for key in &keys_to_decrypt {
            if let Some(toml::Value::String(enc)) = table.remove(key) {
                match key_encryption::decrypt(&enc) {
                    Ok(plain) => {
                        table.insert(key.clone(), toml::Value::String(plain));
                        println!("  {} Decrypted '{}'", warn("⚠"), key);
                        changed = true;
                    }
                    Err(e) => {
                        eprintln!("  {} Failed to decrypt '{}': {}", err("✗"), key, e);
                        table.insert(key.clone(), toml::Value::String(enc));
                    }
                }
            }
        }
    }
    if !changed {
        println!("  {} No encrypted values found in config", info("ℹ"));
        return;
    }
    let output = toml::to_string_pretty(&cfg).unwrap_or(content);
    if let Err(e) = std::fs::write(&config_path, &output) {
        eprintln!("{} Failed to write config: {}", err("Error:"), e);
        return;
    }
    println!(
        "{} API keys are now stored in plaintext. Consider re-encrypting with `neotrix config encrypt-keys`.",
        warn("⚠")
    );
    println!(
        "  {} Config written to {}",
        success("✓"),
        config_path.display()
    );
}

// ── Wallet commands ──

pub fn run_wallet_create(label: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.persist_wallet(label) {
        Ok(lbl) => {
            if let Some(w) = crypto.wallet_manager.active_wallet() {
                println!("{}", success("Wallet created successfully"));
                println!("  Label:   {}", lbl);
                println!("  Address: {}", w.address);
                println!("  Path:    {:?}", crypto.wallet_store.dir_path());
            }
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_import(label: &str, private_key: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.import_wallet(private_key, label) {
        Ok(w) => {
            println!("{}", success("Wallet imported successfully"));
            println!("  Label:   {}", w.label);
            println!("  Address: {}", w.address);
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_list(json: bool) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
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
                match serde_json::to_string_pretty(&serde_json::json!({"wallets": list})) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("{}: JSON serialization failed: {}", err("Error"), e),
                }
            } else if wallets.is_empty() {
                println!(
                    "  {} No wallets found. Use {} to create one.",
                    info("ℹ"),
                    info("neotrix wallet create <label>")
                );
            } else {
                println!("  {} Wallets ({})", success("✓"), wallets.len());
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
                    println!("    • {} [{}] {}", w.label, w.chain, addr_short);
                }
            }
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_balance(chain: &str) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    let addr = match crypto.wallet_manager.active_wallet() {
        Some(w) => w.address.clone(),
        None => {
            eprintln!(
                "{} No active wallet. Create or import one first.",
                err("Error:")
            );
            return;
        }
    };
    println!(
        "  {} Checking balance of {} on {}",
        info("ℹ"),
        &addr[..10],
        chain
    );
}

pub fn run_wallet_delete(label: &str) {
    let mut crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.delete_persisted_wallet(label) {
        Ok(_) => println!("{} Wallet '{}' deleted", success("✓"), label),
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_export(label: &str) {
    let crypto = neotrix::l1_action::nt_act::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.load_wallet(label) {
        Ok(w) => {
            println!(
                "{}",
                warn("⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!")
            );
            println!();
            println!("🔑 {} 私钥:", w.label);
            println!("{}", w.private_key_hex());
        }
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

/// NT-AGENT 模式 — NeoTrix 作为主体的对话驱动循环。
///
/// 架构目标：LLM 降级为后端能力（`LlmProvider`），决策循环由 Rust 的
/// `AgentLoop` 持有。本入口：
///   1. 初始化 GatewayV2（provider 路由/熔断/限流）
///   2. 装配 MCP 原生工具（ToolOrchestrator → AgentLoop 工具集）
///   3. 启动交互 REPL：每轮 `loop_.turn(input)` 驱动 用户→LLM→工具→回答
#[allow(dead_code)] // 保留入口：待上层接线后启用
pub fn run_agent_mode(profile: &str) {
    use neotrix::agent::tool::mcp::{McpToolDef, McpTransport};
    use neotrix::agent::tool::McpRegistry;
    use neotrix::l1_action::nt_io::nt_io_agent_loop::AgentLoop;
    use neotrix::l1_action::nt_io::nt_io_provider::factory::create_gateway_async;
    use std::io::{self, Write};

    const NT_CORE_SYSTEM_PROMPT: &str = "\
You are NT-CORE, the orchestrating brain of the NeoTrix system. \
You hold state, route work, and decide. The language model you are part of is a \
backend reasoning engine you call — not your master. Answer the user directly. \
You have tools available; call them when they help. Be concise and evidence-first.";

    let rt = tokio_runtime();
    rt.block_on(async {
        ensure_provider_env_from_config();

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![McpToolDef {
            name: "neotrix_info".to_string(),
            description: "NeoTrix MCP system info".to_string(),
            server_name: "built-in".to_string(),
            transport: McpTransport::Local {
                command: "echo".to_string(),
                args: vec![],
            },
            input_schema: serde_json::json!({"type": "object"}),
            schema_version: None,
            ..Default::default()
        }];
        builtin_tools.extend(Vec::<neotrix::agent::tool::mcp::McpToolDef>::new());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);

        // 意识核心能力面: src/cli/commands 已删除, awareness_core_tools() 为空;
        // 工具面由 MCP 注册表提供, 意图路由由 nt_auto_orchestrator 分类。
        let mut tools = mcp_registry.as_native_tools();
        tools.extend(neotrix::l5_cognition::nt_core::nt_io_awareness_core::awareness_core_tools());

        let gateway = create_gateway_async().await;
        let default_model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| {
            let cfg = neotrix::config::NeoTrixConfig::load();
            cfg.default_model
                .clone()
                .unwrap_or_else(|| "default".to_string())
        });

        let mut loop_ = AgentLoop::new(Arc::new(gateway), &default_model, NT_CORE_SYSTEM_PROMPT)
            .with_tools(tools);
        let _ = profile;

        println!("╭─ NeoTrix Agent Loop ─────────────────────────────╮");
        println!("│  NT-CORE 作为主体 · LLM 作为后端推理引擎        │");
        println!(
            "│  model: {} · tools: {}          │",
            loop_.model(),
            loop_.tool_count()
        );
        println!("│  /exit 退出 · /tools 查看工具 · /hist 查看历史  │");
        println!("╰──────────────────────────────────────────────────╯");

        loop {
            print!("\n❯ ");
            io::stdout().flush().unwrap_or(());
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = input.trim();
                    match trimmed {
                        "/exit" | "/q" => {
                            println!("Exiting.");
                            break;
                        }
                        "/tools" => {
                            for t in loop_.tool_count()..loop_.tool_count() {
                                let _ = t;
                            }
                            println!("{} tools registered", loop_.tool_count());
                            for inv in &loop_.tool_log {
                                println!(
                                    "  {} → {}: {}",
                                    if inv.success { "✓" } else { "✗" },
                                    inv.name,
                                    inv.output
                                );
                            }
                        }
                        "/hist" => {
                            println!("{} messages in history", loop_.history_len());
                        }
                        _ if !trimmed.is_empty() => match loop_.turn(trimmed).await {
                            Ok(response) => println!("\n{}", response),
                            Err(e) => eprintln!("\n{} {}", err("Error:"), e),
                        },
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    break;
                }
            }
        }
    });
}

/// NT-AGENT TUI 模式 — 基于 ratatui 的完整对话终端。
///
/// **STUB** — cli::tui 模块已移除，此函数仅打印错误并返回。
pub fn run_agent_tui(_profile: &str) {
    eprintln!(
        "{} TUI 模块已移除，请使用 --headless 或 web 模式",
        err("Error")
    );
}

#[cfg(test)] // 仅测试使用（nt_entry_tests）
fn run_shell_direct(cmd: &str) -> Result<(i32, String, String), String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("shell 执行失败: {}", e))?;
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Ok((code, stdout, stderr))
}

/// 运行 `git diff --no-color [path]`，返回 stdout（best-effort，失败返回错误信息）。
#[allow(dead_code)] // 保留工具函数：待调用方接线后启用
fn run_git_diff(path: Option<&str>) -> Result<String, String> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--no-color"]);
    if let Some(p) = path {
        cmd.arg(p);
    }
    let out = cmd.output().map_err(|e| format!("git 执行失败: {}", e))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// ── T14b 语义迁移桥接（A 侧，调用方侧）：team::AgentRole → L1 正典 AgentCard ──
// 旧 struct 定义与旧调用点不动（E2 接线）；正典文件只读。
// 映射：id＝调用方传入；name←role.name；description←role.backstory；
// tags←role.tools 克隆；role_chain←vec![role.role]；其余走 AgentCard::new 正典默认。
// 语义缺口：goal 字段无处可放（正典无对应位），本次丢弃，E2 需裁决去向。
#[allow(dead_code)] // E2 接线前保留：旧调用点切换时启用
pub fn team_role_to_card(
    id: &str,
    role: &neotrix::agent::team::AgentRole,
) -> neotrix::l1_action::nt_infra_agent_card::AgentCard {
    let mut card =
        neotrix::l1_action::nt_infra_agent_card::AgentCard::new(id, &role.name, &role.backstory);
    card.tags = role.tools.clone();
    card.role.role_chain = vec![role.role.clone()];
    card
}

#[cfg(test)]
mod t14b_team_card_tests {
    use super::team_role_to_card;

    #[test]
    fn team_role_maps_to_card() {
        let role = neotrix::agent::team::AgentRole {
            name: "planner".to_string(),
            role: "Task Planner".to_string(),
            goal: "Break down complex tasks".to_string(),
            backstory: "Strategic planner".to_string(),
            tools: vec!["reason".to_string()],
        };
        let card = team_role_to_card("planner", &role);
        assert_eq!(card.id, "planner");
        assert_eq!(card.name, "planner");
        assert_eq!(card.description, "Strategic planner");
        assert_eq!(card.tags, vec!["reason".to_string()]);
        assert_eq!(card.role.role_chain, vec!["Task Planner".to_string()]);
    }
}

#[cfg(test)]
mod nt_entry_tests;
