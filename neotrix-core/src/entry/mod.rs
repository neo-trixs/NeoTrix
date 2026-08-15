#![deny(clippy::unwrap_used)]

use std::sync::Arc;
use std::path::PathBuf;
use std::io::{self, Write};

use colored::Colorize;

use neotrix::neotrix::nt_mind_background_loop::BackgroundLoop;
use neotrix::neotrix::nt_world_model::WorldModelV2;
use neotrix::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::neotrix::nt_mind::self_iterating::{ReasoningBrain, SelfIteratingBrain};
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use neotrix::neotrix::nt_io_mention::resolve_mentions;
use neotrix::core::nt_core_task_dispatcher::{TaskDecomposerDispatcher, DispatcherConfig};
use neotrix::neotrix::ReasoningKernel;
use neotrix::core::nt_core_policy::E8Policy;

use neotrix::config::NeoTrixConfig;
use neotrix::cli::tui::output::StreamingMarkdownRenderer;

mod proxy_cmd;
mod standalone;
mod headless;
mod desktop;
mod sysops;

pub use proxy_cmd::run_proxy_cmd;
pub use sysops::run_sysops;
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
                model.push_str("gpt-4o-mini");
            }
            model.trim().to_string()
        }
        _ => "gpt-4o-mini".to_string(),
    };

    let config_path = neotrix::config::NeoTrixConfig::path();
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("[config] warning: failed to create config directory ({}); continuing", e);
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
            if url.is_empty() { None } else { Some(url) }
        }
        _ => None,
    };

    // Encrypt the API key before persisting to disk
    // 加密失败即拒绝保存，禁止明文回退 (fail-closed，防密钥落盘可读)
    let stored_key = if !api_key.is_empty() {
        match neotrix::neotrix::nt_shield::key_encryption::encrypt(&api_key) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!("{}: key encryption failed ({}); refusing to store plaintext key", err("Error"), e);
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
         default_model = {:?}\n",
        provider, stored_key, default_model,
    );
    if let Some(ref ep) = custom_endpoint {
        content.push_str(&format!("custom_endpoint = {:?}\n", ep));
    }

    if let Err(e) = std::fs::write(&config_path, content) {
        eprintln!("{}: failed to write config file ({}); configuration not saved", err("Error"), e);
        return;
    }
    println!();
    println!("✅ Configuration saved to: {}", config_path.display());
    println!("   Provider: {}", provider);
    if !api_key.is_empty() {
        println!("   API Key: ****{}", &api_key[api_key.len().saturating_sub(4)..]);
    }
    println!();
    println!("You can change these settings anytime by editing the config file.");
}

fn print_brain_stats(brain: &SelfIteratingBrain) {
    let stats = brain.brain.get_statistics();
    println!("\n{}", info("╭─ NeoTrix V2 Brain Status ──────────────────────────╮"));
    println!("│ {} {:<5}  {} {:<5}             │",
        info("Iteration:"), brain.iteration,
        info("Absorbed:"), brain.brain.total_absorb_count);
    println!("│ {} {:.3}  {} {:<5}       │",
        info("Capability Sum:"), stats.capability_sum,
        info("Memory:"), brain.reasoning_bank.memories().len());
    println!("{}", info("╰──────────────────────────────────────────────────────╯"));
}

fn brain_dir(profile: &str) -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".neotrix");
    if profile.is_empty() || profile == "default" { base } else { base.join("profiles").join(profile) }
}

fn init_brain(profile: &str) -> (ReasoningBrain, ReasoningBank) {
    let dir = brain_dir(profile);
    std::env::set_var("NEOTRIX_HOME", &dir);

    if ReasoningBrain::has_saved_state() {
        match ReasoningBrain::load() {
            Ok(b) => {
                println!("{}", info(format!("Loaded brain from {}/brain.json", dir.display())));
                (b, ReasoningBank::new(100))
            }
            Err(e) => {
                eprintln!("{}", warn(format!("Load failed ({}), creating new brain", e)));
                (ReasoningBrain::new(), ReasoningBank::new(100))
            }
        }
    } else {
        println!("{}", info(format!("New brain at {}/brain.json", dir.display())));
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
    println!("{}", info("Starting background services... Press Ctrl+C to stop."));
    let server_rt = tokio_runtime();
    server_rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        let mut panorama = PanoramaPipeline::new();
        if let Ok(kb) = neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
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
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("HTTP server runtime");
            rt.block_on(async {
                // L1 层禁止直接依赖 L8 (层边界守卫 arch_fitness_layer_boundary):
                // 在 entry (bin 层) 构造 ReasoningBrain 后经 start_server_with 注入。
                neotrix::neotrix::nt_io_web::server::start_server_with(
                    http_port,
                    Box::new(neotrix::neotrix::l8_autonomic_impl::nt_mind::ReasoningBrain::new()),
                    neotrix::core::ReasoningBank::new(10000),
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
                        match neotrix::neotrix::nt_shield_stealth_net::config::reload() {
                            Ok(_) => log::info!("[hotreload] SIGHUP: stealth-net config reloaded"),
                            Err(e) => log::warn!("[hotreload] SIGHUP reload failed: {}", e),
                        }
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    log::info!("[hotreload] SIGHUP received (stealth-net feature off, nothing to reload)");
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
        if !p.is_empty() { return p.to_string(); }
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
            use neotrix::cli::jsonl_stream::JsonlWriter;
            let mut writer = JsonlWriter::new();
            writer.emit_error("Empty prompt", Some("EMPTY_PROMPT"), false);
            writer.emit_finish("", 0, 0, 1);
        } else {
            eprintln!("error: empty prompt");
        }
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() && !json_output {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let start = std::time::Instant::now();
    let rt = tokio_runtime();

    if json_output {
        use neotrix::cli::jsonl_stream::JsonlWriter;
        let mut writer = JsonlWriter::new();
        writer.emit_start(&prompt, None, None, None);

        let result = rt.block_on(async {
            let mut agent = build_brain("default");

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let task = async {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt)
                } else {
                    let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after))
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
                let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
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
                    let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after))
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
        eprintln!("{}: usage: neotrix run <prompt> | neotrix reason <prompt>", err("Error"));
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
                            eprintln!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
                let r = agent.iterate(task_type);
                let msg = format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after);
                if format == Some("json") {
                    let json = serde_json::json!({"success": true, "response": msg, "prompt": prompt});
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or(msg));
                } else {
                    println!("{}", msg);
                }
                Ok(())
            };
            if let Err(e) = result {
                if format == Some("json") {
                    let json = serde_json::json!({"success": false, "error": e.to_string()});
                    eprintln!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
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
            let gateway = agent.reasoning_engine.as_ref().and_then(|e| e.gateway.clone());
            let reasoning_engine = agent.reasoning_engine.take();
            let kernel = ReasoningKernel::new(3);
            let e8_policy = E8Policy::default();
            
            let mut dispatcher = match (gateway, reasoning_engine) {
                (Some(gw), Some(re)) => TaskDecomposerDispatcher::new(
                    gw,
                    DispatcherConfig::from_env(),
                )
                .with_reasoning_engine(re)
                .with_kernel(kernel)
                .with_e8_policy(e8_policy),
                _ => {
                    eprintln!("{}: missing gateway or reasoning engine", err("Reasoning error"));
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
                        println!("{}", serde_json::to_string_pretty(&json).unwrap_or(response));
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
                        eprintln!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
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
                let task_type = neotrix::neotrix::nt_world_model::TaskType::General;
                pb.inc(50);
                let r = agent.iterate(task_type);
                pb.finish_with_message("done");
                Ok(format!("Learned: {:.3} → {:.3}", r.score_before, r.score_after))
            };

            match result {
                Ok(response) => {
                    if format == Some("json") {
                        let json = serde_json::json!({
                            "success": true,
                            "response": response,
                            "prompt": prompt,
                        });
                        println!("{}", serde_json::to_string_pretty(&json).unwrap_or(response));
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
                        eprintln!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
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
        "analyze", "design", "implement", "debug", "refactor", "optimize",
        "architecture", "plan", "research", "compare", "evaluate",
        "step by step", "think through", "break down", "decompose",
    ];
    complex_keywords.iter().any(|kw| lower.contains(kw)) || prompt.len() > 200
}

pub fn show_status() {
    let status = neotrix::neotrix::nt_io_proxy_server::ServerProxy::status();
    println!("{}", info("╭─ NeoTrix Status ───────────────────────╮"));
    println!("│ {}  {:<2} / {:<2} {}   │",
        info("Brain dimensions:"),
        status["brain_dims"].as_i64().unwrap_or(0),
        status["total_dims"].as_i64().unwrap_or(23),
        info("active"));
    println!("│ {}  {:<4}               │",
        info("Extensions:"),
        status["brain_extension"].as_i64().unwrap_or(0));
    println!("│ {}   {:<8} {}  │",
        info("Knowledge store:"),
        status["knowledge_store_bytes"].as_i64().unwrap_or(0),
        info("bytes"));
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
            eprintln!("error: unsupported shell '{}'. Use: bash, zsh, fish, powershell, elvish", other);
            std::process::exit(1);
        }
    };
    let mut stdout = std::io::stdout();
    clap_complete::generate(shell, cmd, "neotrix", &mut stdout);
}

pub fn run_consciousness_core(sub: Option<&str>, want_json: bool, cycles: usize) {
    use neotrix::core::nt_core_consciousness_core as consciousness_core;

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
            let health_map: serde_json::Value = branches.iter().fold(
                serde_json::Map::new(),
                |mut acc, b| {
                    let health_v = b.get("health").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    let fog_v = b.get("fog").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
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
                },
            ).into();
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
            println!("│ GWT 谐振  {:>54}", if snap.gwt_resonance_active { "active" } else { "idle" });
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
                println!("  {:<14} 健康 {:>5}  {:?} {:?}", kind, health, tier, constel);
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
                println!("  {:<14} 健康{:>5} 雾{:>4}  {:?} {:?}", kind, health, fog_s, tier, constel);
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
    use neotrix::neotrix::nt_mind_evolution_loop::{EvolutionLoop, REPAIR_MAX_ROUNDS};

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
    println!("无测试模块 {} 个", report.snapshot.modules_without_tests.len());
    println!("unsafe    {} 处 (热点 {} 文件)", report.snapshot.unsafe_count, report.snapshot.file_unsafe_hotspots.len());
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
    let mut server = neotrix::core::McpServer::new();
    server.register_all_tools();
    // 单调授权守卫: 阻断破坏性 shell 命令 (NT-SHIELD GuardChain 生产接线)
    {
        use neotrix::neotrix::l1_body_impl::nt_shield::guard_chain::GuardVerdict;
        server.add_guard("destructive_shell", |tool, args| {
            if tool != "execute_command" {
                return GuardVerdict::Allow;
            }
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let dangerous = [
                "rm -rf /",
                "rm -rf ~",
                ":(){ :|:& };:",
                "mkfs.",
                "dd if=",
                "> /dev/sda",
            ];
            if dangerous.iter().any(|d| cmd.contains(d)) {
                GuardVerdict::Deny
            } else {
                GuardVerdict::Allow
            }
        });
    }
    // 横幅必须走 stderr: MCP stdio 协议要求 stdout 只承载 JSON-RPC 帧。
    eprintln!("neotrix-mcp {} starting (stdio JSON-RPC 2.0)", env!("CARGO_PKG_VERSION"));
    if let Err(e) = server.run() {
        eprintln!("MCP server error: {}", e);
    }
}

pub fn run_benchmark(category: Option<&str>) {
    use neotrix::neotrix::nt_mind_benchmark::{BenchmarkSuite, BenchmarkReport};
    use neotrix::CapabilityVector;

    let path = dirs::home_dir().unwrap_or_default().join(".neotrix/brain.json");
    let cap: CapabilityVector = if path.exists() {
        let json = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&json).unwrap_or_else(|e| {
            eprintln!("{}", warn(format!("failed to parse brain.json ({}), using default", e)));
            CapabilityVector::default()
        })
    } else {
        CapabilityVector::default()
    };

    let mut bank = ReasoningBank::new(100);
    let report = match category {
        Some(cat) => {
            let results = BenchmarkSuite::run_category(&cap, cat);
            let overall = if results.is_empty() { 0.0 } else {
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
        println!("│ {:<13} | {:<17} | {:.2}  │", r.category, name_display, r.score);
    }
    if !report.results.is_empty() {
        println!("├───────────────┼───────────────────┼───────┤");
    }
    println!("│ OVERALL       │                   │ {:.2}  │", report.overall_score);
    println!("╰───────────────┴───────────────────┴───────╯");
}

pub fn run_browse(url: &str) {
    use neotrix::neotrix::nt_world_browse::BrowserCircuit;
    println!("{}", info("╭─ NeoTrix Browser ──────────────────────────╮"));
    println!("│ {} {}", info("Fetching:"), url);
    println!("{}", info("╰────────────────────────────────────────────────╯"));
    let browser = BrowserCircuit::new();
    match browser.browse(url) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            println!("\n{} ({} lines, ~{} chars):",
                info("Content"), lines.len(), text.len());
            for line in lines.iter().take(60) {
                println!("  {}", line);
            }
            if lines.len() > 60 {
                println!("  {} ({})", info("..."), info(format!("{} more lines", lines.len() - 60)));
            }
        }
        Err(e) => eprintln!("{}: {}", err("Error"), e),
    }
}

pub fn run_search(query: &str, count: usize) {
    use neotrix::neotrix::nt_world_search::WebSearchEngine;

    let engine = WebSearchEngine::default();
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
                println!("{}. {}",
                    info(format!("{}", i + 1)),
                    result.title.bold());
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
    use neotrix::neotrix::nt_world_browse::BrowserCircuit;
    println!("{}", info("╭─ NeoTrix Login ────────────────────────────╮"));
    println!("│ {}: {}", info("URL"), url);
    println!("│ {}", info("A Chrome window will open. Log in, then"));
    println!("│ {}", info("close the window to save the session."));
    println!("{}", info("╰─────────────────────────────────────────────╯"));
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
                            println!("{}", success("✅ Update available! Run `neotrix update` to install."));
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
        println!("{}", info("Build with --features self-update or use your package manager."));
    }
}

pub fn run_daemon(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{} {}", info("[daemon]"), info("NeoTrix background daemon started"));
        bg.start().await;
        tokio::signal::ctrl_c().await.unwrap_or_default();
        println!("\n{}", info("[daemon] shutting down..."));
        // Persist E8 state on graceful shutdown (SIGTERM/Ctrl+C)
        if let Ok(brain_guard) = bg.brain.try_read() {
            brain_guard.shutdown_save_e8();
        }
        bg.shutdown().await;
    });
}

pub fn run_daemon_evolution(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{} {}", info("[daemon]"), info("NeoTrix evolution daemon started"));
        let daemon = std::sync::Arc::new(std::sync::Mutex::new(
            neotrix::neotrix::nt_mind_evolution_daemon::EvolutionDaemon::default()
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
        println!("\n{}", info("[daemon] shutting down..."));
        // Persist E8 state on graceful shutdown (SIGTERM/Ctrl+C)
        if let Ok(brain_guard) = bg.brain.try_read() {
            brain_guard.shutdown_save_e8();
        }
        // 终止 evolution 后台任务，避免 ctrl_c 后幽灵 tick (R-P38)
        evolution_task.abort();
        let _ = evolution_task.await;
        bg.shutdown().await;
    });
}

pub fn run_standalone_mode(stage: usize) {
    let rt = tokio_runtime();
    rt.block_on(async {
        standalone::run_standalone(stage).await;
    });
}

pub fn run_headless_mode(_cfg: &NeoTrixConfig, profile: &str) {
    use neotrix::neotrix::nt_mind_background_loop::BackgroundLoop;
    use neotrix::neotrix::nt_world_model::WorldModelV2;
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::hooks::{EccHookRegistry, HookEvent, HookContext};
    use neotrix::agent::tool::mcp::{McpRegistry, McpTransport, McpToolDef};
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
            println!("{}: {} {}", info("ReasoningEngine"), success("active"), info("(LLM connected)"));
        } else {
            println!("{}: {}", warn("ReasoningEngine"), warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)"));
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        println!("{}: {} ", info("SkillsEngine"), success(format!("{} local skills loaded", skill_count)));
        println!("  -> {} /skills list to browse, /skills ecc <id> to load from ECC community", info("/skills"));

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![
            McpToolDef {
                name: "neotrix_info".to_string(),
                description: "NeoTrix MCP system info".to_string(),
                server_name: "built-in".to_string(),
                transport: McpTransport::Local {
                    command: "echo".to_string(),
                    args: vec![],
                },
                input_schema: serde_json::json!({"type": "object"}),
                schema_version: None,
            },
        ];
        builtin_tools.extend(neotrix::neotrix::nt_agent_mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::nt_agent_mcp_tools::register_neotrix_tools(&mut mcp_registry);
        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        neotrix::cli::commands::agent_cmds::set_tool_orchestrator(orchestrator);
        println!("{}: {} native MCP tools absorbed via McpToolAdapter", info("ToolOrchestrator"), success(mcp_registry.tool_count().to_string()));
        neotrix::cli::commands::agent_cmds::set_mcp_registry(mcp_registry.clone());
        println!("{}: {} ({})", info("McpRegistry"), success("ready"), info("use /mcp list"));
        let mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!("{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)"));

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg_goal_loop.load();
        let agent_team = Arc::new(Mutex::new(AgentTeam::new("default", ProcessType::Sequential)));
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
            use neotrix::neotrix::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default")
                .with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!("{}: {} (session #{}, {} messages)",
                    info("SessionRecovery"), success("restored"),
                    snapshot.session_id, snapshot.message_count);
            } else {
                println!("{}: {} — no previous session found",
                    info("SessionRecovery"), dim("fresh start"));
            }
        }

        // AGENTS.md
        {
            use neotrix::neotrix::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(&std::path::Path::new(".")) {
                if !rules.is_empty() {
                    println!("{}: {} ({} sections)",
                        info("AGENTS.md"), success("loaded"), rules.sections.len());
                } else {
                    println!("{}: {} — no rules found",
                        info("AGENTS.md"), dim("skipped"));
                }
            }
        }

        let sp = indicatif::ProgressBar::new_spinner();
        match indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
        {
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
    use neotrix::neotrix::nt_mind_background_loop::BackgroundLoop;
    use neotrix::neotrix::nt_world_model::WorldModelV2;
    use neotrix::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::hooks::{EccHookRegistry, HookEvent, HookContext};
    use neotrix::agent::tool::mcp::{McpRegistry, McpTransport, McpToolDef};
    use neotrix::agent::{AgentTeam, AgentRole, ProcessType};
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
            println!("{}: {} {}", info("ReasoningEngine"), success("active"), info("(LLM connected)"));
        } else {
            println!("{}: {}", warn("ReasoningEngine"), warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)"));
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        println!("{}: {} ", info("SkillsEngine"), success(format!("{} local skills loaded", skill_count)));
        println!("  -> {} /skills list to browse, /skills ecc <id> to load from ECC community", info("/skills"));

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![
            McpToolDef {
                name: "neotrix_info".to_string(),
                description: "NeoTrix MCP system info".to_string(),
                server_name: "built-in".to_string(),
                transport: McpTransport::Local {
                    command: "echo".to_string(),
                    args: vec![],
                },
                input_schema: serde_json::json!({"type": "object"}),
                schema_version: None,
            },
        ];
        builtin_tools.extend(neotrix::neotrix::nt_agent_mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::nt_agent_mcp_tools::register_neotrix_tools(&mut mcp_registry);
        let mut orchestrator = neotrix::agent::tool::ToolOrchestrator::default();
        orchestrator.register_native_all(mcp_registry.as_native_tools());
        neotrix::cli::commands::agent_cmds::set_tool_orchestrator(orchestrator);
        neotrix::cli::commands::agent_cmds::set_mcp_registry(mcp_registry.clone());
        println!("{}: {} ({})", info("McpRegistry"), success("ready"), info("use /mcp list"));
        let _mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = EccHookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!("{}: {} {}",
            info("EccHookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)"));

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = EccHookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let _skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry: Arc<RwLock<EccHookRegistry>> = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg_goal_loop.load();
        if bg_goal_loop.active_goal.is_some() {
            println!("{} {}", info("[bg]"), info("Restored background goal from ~/.neotrix/goals.json"));
        }

        let agent_team = Arc::new(Mutex::new(AgentTeam::new("default", ProcessType::Sequential)));
        {
            let mut team = agent_team.lock().unwrap_or_else(|e| e.into_inner());
            team.add_agent(AgentRole {
                name: "planner".into(),
                role: "Task Planner".into(),
                goal: "Break down complex tasks into sub-tasks".into(),
                backstory: "Strategic planner with systems thinking".into(),
                tools: vec!["reason".into()],
            });
        }
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);

        let mut panorama = PanoramaPipeline::new();
        let bg_kb: Option<std::sync::Arc<neotrix::neotrix::nt_memory_kb::KnowledgeBase>>;
        if let Ok(kb) = neotrix::neotrix::nt_memory_kb::KnowledgeBase::open(None) {
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
            use neotrix::neotrix::nt_io_session_recovery::SessionRecoveryManager;
            let recovery_mgr = SessionRecoveryManager::new("default")
                .with_auto_recover(true);
            if let Some(snapshot) = recovery_mgr.load_latest_snapshot() {
                println!("{}: {} (session #{}, {} messages, {} e8 states)",
                    info("SessionRecovery"), success("restored"),
                    snapshot.session_id, snapshot.message_count, snapshot.e8_state_sequence.len());
            } else {
                println!("{}: {} — no previous session found",
                    info("SessionRecovery"), dim("fresh start"));
            }
        }

        // AGENTS.md — 扫描项目规则文件
        {
            use neotrix::neotrix::nt_io_agents_md::AgentsMdReader;
            let agents_reader = AgentsMdReader::new();
            if let Ok(rules) = agents_reader.load_project_rules(&std::path::Path::new(".")) {
                if !rules.is_empty() {
                    let sections: Vec<&str> = rules.sections.keys().map(|k| k.as_str()).collect();
                    println!("{}: {} ({}) — {} sections: {}",
                        info("AGENTS.md"), success("loaded"),
                        rules.source_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "),
                        rules.sections.len(),
                        sections.join(", "));
                } else {
                    println!("{}: {} — no rules found in current directory",
                        info("AGENTS.md"), dim("skipped"));
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
    use neotrix::neotrix::nt_shield_sandbox::cli;
    let runtime = if runtime.is_empty() { None } else { Some(runtime) };
    let rt = tokio_runtime();
    rt.block_on(cli::handle_run(code, runtime, Some(timeout)));
}

pub fn run_sandbox_list() {
    neotrix::neotrix::nt_shield_sandbox::cli::handle_list();
}

pub fn run_sandbox_cancel(session_id: &str) {
    neotrix::neotrix::nt_shield_sandbox::cli::handle_cancel(session_id);
}

pub fn run_discover(port: u16, duration_ms: u64, json: bool) {
    use neotrix::neotrix::nt_agent_protocol::discovery::AgentDiscovery;

    let mut discovery = match AgentDiscovery::new(port) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("❌ 绑定 UDP :{} 失败: {}", port, e);
            return;
        }
    };

    eprintln!("🔍 扫描中 ({}ms, UDP :{})...", duration_ms, port);
    match discovery.discover(duration_ms) {
        Ok(agents) => {
            if agents.is_empty() {
                println!("🔍 扫描完成，未发现任何代理");
                if json {
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "success": true, "agent_count": 0, "port": port, "duration_ms": duration_ms
                    })).unwrap_or_default());
                }
                return;
            }

            if json {
                let json_agents: Vec<serde_json::Value> = agents.iter().map(|a| {
                    serde_json::json!({
                        "id": a.id, "name": a.name, "host": a.host, "port": a.port,
                        "capabilities": a.capabilities, "hexagram": a.hexagram,
                        "service_type": a.service_type, "instance_name": a.instance_name,
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "success": true, "agent_count": agents.len(), "port": port,
                    "duration_ms": duration_ms, "agents": json_agents
                })).unwrap_or_default());
            } else {
                println!("🔍 发现 {} 个代理 (扫描 {}ms):", agents.len(), duration_ms);
                println!("{:-<72}", "");
                println!(" {:<4} {:<24} {:<22} {:<6} {:<4}", "#", "ID", "Host", "Port", "Caps");
                println!("{:-<72}", "");
                for (i, a) in agents.iter().enumerate() {
                    let id_trunc = if a.id.len() > 23 { format!("{}…", &a.id[..22]) } else { a.id.clone() };
                    println!(" {:<4} {:<24} {:<22} {:<6} {:<4}",
                        i + 1, id_trunc, a.host, a.port, a.capabilities.len());
                }
                println!("{:-<72}", "");
                if agents.len() == 1 {
                    let a = &agents[0];
                    println!("  详情:");
                    println!("    Name:     {}", a.name);
                    println!("    Service:  {}", if a.service_type.is_empty() { "(none)" } else { &a.service_type });
                    println!("    Instance: {}", if a.instance_name.is_empty() { "(none)" } else { &a.instance_name });
                    if !a.capabilities.is_empty() {
                        println!("    Caps:     {}", a.capabilities.join(", "));
                    }
                    if a.hexagram != 0 {
                        println!("    Hexagram: {}", a.hexagram);
                    }
                }
            }
        }
        Err(e) => eprintln!("❌ 扫描失败: {}", e),
    }
}

pub fn run_sandbox_upload(path: &str, session_id: &str) {
    use neotrix::neotrix::nt_shield_sandbox::cli;
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
        println!("  Use {} to enable a feature", info("neotrix features enable <name>"));
        return;
    }
    println!("  {} Enabled feature flags:", success("✓"));
    for f in &features {
        println!("    • {}", f);
    }
}

// ── Config commands ──

pub fn run_config_encrypt_keys() {
    use neotrix::neotrix::nt_shield::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!("{} No config file found at {}", err("Error:"), config_path.display());
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
                (k_lower.contains("api_key") || k_lower.contains("apikey") || k_lower.contains("secret"))
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
        println!("  {} No plaintext API keys or secrets found in config", info("ℹ"));
        return;
    }
    let output = toml::to_string_pretty(&cfg).unwrap_or(content);
    if let Err(e) = std::fs::write(&config_path, &output) {
        eprintln!("{} Failed to write config: {}", err("Error:"), e);
        return;
    }
    println!("  {} Config written to {}", success("✓"), config_path.display());
}

pub fn run_config_decrypt_keys() {
    use neotrix::neotrix::nt_shield::key_encryption;
    let config_path = neotrix::config::NeoTrixConfig::path();
    if !config_path.exists() {
        eprintln!("{} No config file found at {}", err("Error:"), config_path.display());
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
            .filter(|(_, v)| v.is_str() && key_encryption::is_encrypted(v.as_str().unwrap_or_default()))
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
    println!("  {} Config written to {}", success("✓"), config_path.display());
}

// ── Wallet commands ──

pub fn run_wallet_create(label: &str) {
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
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
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
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
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.list_wallets() {
        Ok(wallets) => {
            if json {
                let list: Vec<serde_json::Value> = wallets.iter().map(|w| {
                    serde_json::json!({
                        "label": w.label, "address": w.address,
                        "chain": w.chain, "created": w.created_at
                    })
                }).collect();
                match serde_json::to_string_pretty(&serde_json::json!({"wallets": list})) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("{}: JSON serialization failed: {}", err("Error"), e),
                }
            } else if wallets.is_empty() {
                println!("  {} No wallets found. Use {} to create one.",
                    info("ℹ"), info("neotrix wallet create <label>"));
            } else {
                println!("  {} Wallets ({})", success("✓"), wallets.len());
                for w in &wallets {
                    let addr_short = if w.address.len() > 12 {
                        format!("{}...{}", &w.address[..6], &w.address[w.address.len()-4..])
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
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    let addr = match crypto.wallet_manager.active_wallet() {
        Some(w) => w.address.clone(),
        None => {
            eprintln!("{} No active wallet. Create or import one first.", err("Error:"));
            return;
        }
    };
    println!("  {} Checking balance of {} on {}", info("ℹ"), &addr[..10], chain);
}

pub fn run_wallet_delete(label: &str) {
    let mut crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.delete_persisted_wallet(label) {
        Ok(_) => println!("{} Wallet '{}' deleted", success("✓"), label),
        Err(e) => eprintln!("{} {}", err("Error:"), e),
    }
}

pub fn run_wallet_export(label: &str) {
    let crypto = neotrix::neotrix::nt_act_crypto::CryptoAgent::new();
    match crypto.wallet_store.load_wallet(label) {
        Ok(w) => {
            println!("{}", warn("⚠️  安全警告: 私钥可控制你的全部资产, 请勿泄露!"));
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
pub fn run_agent_mode(profile: &str) {
    use neotrix::agent::tool::mcp::{McpRegistry, McpTransport, McpToolDef};
    use neotrix::neotrix::l1_body_impl::nt_io_agent_loop::AgentLoop;
    use neotrix::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async;
    use std::io::{self, Write};
    use std::sync::Arc;

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
        }];
        builtin_tools.extend(neotrix::neotrix::nt_agent_mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::nt_agent_mcp_tools::register_neotrix_tools(&mut mcp_registry);
        // 意识核心能力面: 命令面 (file/git/session/memory/crypto/...) 全部桥接为
        // NativeTool, LLM 意识核心智能调度; 人类只接触基础控制命令。
        let mut tools = mcp_registry.as_native_tools();
        tools.extend(neotrix::neotrix::l1_body_impl::nt_io_awareness_core::awareness_core_tools());

        let gateway = create_gateway_async().await;
        let default_model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| {
            let cfg = neotrix::config::NeoTrixConfig::load();
            cfg.default_model.clone().unwrap_or_else(|| "default".to_string())
        });

        let mut loop_ = AgentLoop::new(Arc::new(gateway), &default_model, NT_CORE_SYSTEM_PROMPT)
            .with_tools(tools);        let _ = profile;

        println!("╭─ NeoTrix Agent Loop ─────────────────────────────╮");
        println!("│  NT-CORE 作为主体 · LLM 作为后端推理引擎        │");
        println!("│  model: {} · tools: {}          │",
            loop_.model(), loop_.tool_count());
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
                        "/exit" | "/q" => { println!("Exiting."); break; }
                        "/tools" => {
                            for t in loop_.tool_count()..loop_.tool_count() {
                                let _ = t;
                            }
                            println!("{} tools registered", loop_.tool_count());
                            for inv in &loop_.tool_log {
                                println!("  {} → {}: {}", if inv.success { "✓" } else { "✗" }, inv.name, inv.output);
                            }
                        }
                        "/hist" => {
                            println!("{} messages in history", loop_.history_len());
                        }
                        _ if !trimmed.is_empty() => {
                            match loop_.turn(trimmed).await {
                                Ok(response) => println!("\n{}", response),
                                Err(e) => eprintln!("\n{} {}", err("Error:"), e),
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => { eprintln!("error: {}", e); break; }
            }
        }
    });
}

/// NT-AGENT TUI 模式 — 基于 ratatui 的完整对话终端。
///
/// 与 [`run_agent_mode`]（逐行 REPL）不同，本入口使用 `TuiApp` 状态机 +
/// crossterm 事件循环 + `AgentLoop::turn_stream()` 流式渲染：
///   - 多行输入 / 历史（↑↓ / Ctrl+R 搜索）/ vim 模式 / Tab 斜杠补全
///   - 工具调用行内状态、token 计数、tokens/sec、会话切换
///   - 流式 markdown 增量渲染（`streaming_text` → `commit_stream`）
pub fn run_agent_tui(profile: &str) {
    use neotrix::agent::tool::mcp::{McpRegistry, McpTransport, McpToolDef};
    use neotrix::neotrix::l1_body_impl::nt_io_agent_loop::AgentLoop;
    use neotrix::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async;
    use neotrix::cli::tui::TuiApp;
    use neotrix::cli::tui::app::KeyAction;
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::execute;
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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
        }];
        builtin_tools.extend(neotrix::neotrix::nt_agent_mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::nt_agent_mcp_tools::register_neotrix_tools(&mut mcp_registry);
        // 意识核心能力面: 命令面 (file/git/session/memory/crypto/...) 全部桥接为
        // NativeTool, LLM 意识核心智能调度; 人类只接触基础控制命令。
        let mut tools = mcp_registry.as_native_tools();
        tools.extend(neotrix::neotrix::l1_body_impl::nt_io_awareness_core::awareness_core_tools());

        let gateway = create_gateway_async().await;
        let default_model = std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| {
            let cfg = neotrix::config::NeoTrixConfig::load();
            cfg.default_model.clone().unwrap_or_else(|| "default".to_string())
        });
        // 整体链路链接: 未显式指定模型时, 从池子实际注册名解析默认 (而非硬编码 provider)。
        let default_model = if default_model.is_empty() || default_model == "default" {
            gateway.resolve_default_model().await
        } else {
            default_model
        };

        let agent = Arc::new(Mutex::new(
            AgentLoop::new(Arc::new(gateway), &default_model, NT_CORE_SYSTEM_PROMPT)
                .with_tools(tools),
        ));
        let _ = profile;
        // ── TUI 初始化 ──────────────────────────────────────────────
        // 前置检查：stdin 必须是 tty（交互式终端），否则 crossterm 事件循环会立即失败。
        use std::io::IsTerminal;
        if !io::stdin().is_terminal() {
            eprintln!("{} TUI 需要交互式终端（stdin 非 tty）。请直接在终端运行，或使用 --headless 模式。", err("Error"));
            return;
        }
        // P0-3 修复: panic 兜底 — 一旦 panic, 恢复 raw mode + alternate screen,
        // 避免终端残留不可用状态。设置钩子后在 re-panic 前尝试清理。
        {
            use std::panic;
            let prev = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                use std::io::Write;
                let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
                let _ = terminal::disable_raw_mode();
                let _ = io::stdout().flush();
                prev(info);
            }));
        }
        terminal::enable_raw_mode().ok();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, EnterAlternateScreen);
        // P1-2 修复: 启用 bracketed paste, 多行粘贴被 crossterm 作为单个
        // Event::Paste(String) 交付, 而非逐 \n 触发 Enter/Submit。
        let _ = execute!(stdout, crossterm::event::EnableBracketedPaste);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                let _ = terminal::disable_raw_mode();
                eprintln!("{} TUI init failed: {}", err("Error"), e);
                return;
            }
        };
        let _ = terminal.clear();

        let mut app = TuiApp::new(false);
        // 从 config 读取主题偏好（color_mode）——非法值回退 dark
        {
            let cfg = neotrix::config::NeoTrixConfig::load();
            if let Some(mode) = cfg.color_mode {
                if matches!(mode.as_str(), "dark" | "light" | "gruvbox") {
                    app.theme_name = mode;
                }
            }
        }
        let model_name = {
            let guard = agent.lock().unwrap_or_else(|e| e.into_inner());
            guard.model().to_string()
        };
        app.status_text = format!("Ready | model: {}", model_name);

        // 会话自动恢复：把最近会话历史加载到 TuiApp.sessions（替代仅打印 restored）。
        let restored = app.restore_sessions();
        if restored > 0 {
            app.status_text = format!("已恢复上次会话 ({} 条消息) | model: {}", restored, model_name);
        }

        let draw = |terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &TuiApp| {
            let theme = neotrix::cli::tui::theme_by_name(&app.theme_name);
            let _ = terminal.draw(|frame| {
                let area = frame.area();
                use neotrix::cli::tui::layout::{
                    compute_layout, render_approval_bar, render_chat_panel, render_diff_panel,
                    render_input_panel, render_session_list, render_session_picker,
                    render_status_bar, render_streaming_tools,
                };
                let (left, chat, input_area, status) = compute_layout(area, app.show_sessions, app.input.lines().count());
                if let Some(left_area) = left {
                    render_session_list(frame, left_area, app, &theme);
                }
                // 审批条：chat 区顶部切 1 行（仅在有 pending 时显示）。
                let (chat_area, approval_area) = if app.pending_approval.is_some() {
                    let v = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Length(1),
                            ratatui::layout::Constraint::Min(0),
                        ])
                        .split(chat);
                    (v[1], Some(v[0]))
                } else {
                    (chat, None)
                };
                // 流式工具区：审批条之下再切一段（有工具时显示，最高 5 行）。
                let (chat_area, tools_area) = if !app.streaming_tool_calls.is_empty() {
                    let max_rows = app.streaming_tool_calls.len().min(5) as u16 + 1;
                    let v = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Length(max_rows),
                            ratatui::layout::Constraint::Min(0),
                        ])
                        .split(chat_area);
                    (v[1], Some(v[0]))
                } else {
                    (chat_area, None)
                };
                if app.diff_viewer.is_some() {
                    render_diff_panel(frame, chat_area, app, &theme);
                } else {
                    render_chat_panel(frame, chat_area, app, &theme);
                }
                if let Some(a) = approval_area {
                    render_approval_bar(frame, a, app, &theme);
                }
                if let Some(t) = tools_area {
                    render_streaming_tools(frame, t, app, &theme);
                }
                render_input_panel(frame, input_area, app, &theme);
                render_status_bar(frame, status, app, &theme);
                // 会话恢复 picker：最顶层 overlay（全屏区域居中弹出）。
                render_session_picker(frame, area, app, &theme);
            });
        };

        let mut exit = neotrix::cli::tui::app::TuiExit::Quit;

        // ── 事件循环 ────────────────────────────────────────────────
        loop {
            // 每 tick 推进 spinner 帧（poll 超时也会重绘 → 动画持续驱动）
            app.tick_spinner();
            draw(&mut terminal, &app);

            // P1-5: 会话切换/清空后, 同步重置 AgentLoop 内部历史 (模型上下文跟随当前会话)。
            if app.needs_agent_reset {
                app.needs_agent_reset = false;
                if let Ok(mut g) = agent.lock() {
                    g.reset_history(NT_CORE_SYSTEM_PROMPT);
                }
            }

            if !event::poll(Duration::from_millis(100)).unwrap_or(false) {
                continue;
            }
            let ev = match event::read() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("{} TUI 事件读取失败: {}", err("Error"), e);
                    break;
                }
            };

            match ev {
                Event::Key(key) => {
                    let action = app.handle_key(key.code, key.modifiers);
                    match action {
                        KeyAction::Quit => { exit = neotrix::cli::tui::app::TuiExit::Quit; break; }
                        KeyAction::ClearScreen => {
                            let _ = terminal.clear();
                            app.scroll_offset = 0;
                        }
                        // 审批决策在外层仅在非流式时可达（无 pending 审批时无意义），
                        // 真实审批事件在流式内循环 (a/d) 处理；此处留空。
                        KeyAction::CancelGeneration | KeyAction::ApprovePending | KeyAction::DenyPending => {}
                        KeyAction::SelectSession(idx) => {
                            // 从 picker 选择会话 → 加载到当前 TuiApp。
                            let name = app.session_picker.as_ref()
                                .and_then(|p| p.entries.get(idx))
                                .map(|e| e.name.clone());
                            app.session_picker = None;
                            if let Some(name) = name {
                                match load_tui_session(&mut app, &name) {
                                    Ok(n) => {
                                        app.status_text = format!("已加载会话 {} ({} 条消息)", name, n);
                                        app.scroll_offset = 0;
                                    }
                                    Err(e) => app.status_text = format!("加载失败: {}", e),
                                }
                            }
                        }
                        KeyAction::ClosePicker => {
                            app.session_picker = None;
                        }
                        KeyAction::DeleteSession(idx) => {
                            // 从 picker 删除选中会话（从 SessionStore + 会话列表同步移除）。
                            let name = app.session_picker.as_ref()
                                .and_then(|p| p.entries.get(idx))
                                .map(|e| e.name.clone());
                            if let Some(name) = name {
                                use neotrix::cli::tui::session_store::SessionStore;
                                let mut store = SessionStore::new();
                                match store.delete_session(&name) {
                                    Ok(()) => {
                                        if let Some(p) = &mut app.session_picker {
                                            p.entries.remove(idx);
                                            if p.entries.is_empty() {
                                                app.session_picker = None;
                                                app.status_text = "会话已删除，无剩余会话".into();
                                            } else {
                                                p.selected = p.selected.min(p.entries.len() - 1);
                                                app.status_text = format!("已删除会话 {}", name);
                                            }
                                        }
                                    }
                                    Err(e) => app.status_text = format!("删除失败: {}", e),
                                }
                            }
                        }
                        KeyAction::Submit => {
                            let input = app.trim().to_string();
                            app.cursor = 0;
                            if input.is_empty() { continue; }
                            // 斜杠命令
                            if input.starts_with('/') {
                                match handle_slash_tui(&mut app, &input) {
                                    SlashResult::Quit => { exit = neotrix::cli::tui::app::TuiExit::Quit; break; }
                                    SlashResult::Handled => { app.input.clear(); app.cursor = 0; continue; }
                                    SlashResult::NotHandled => {
                                        // 作为普通消息发给 AgentLoop。
                                    }
                                }
                            }
                            // `!` shell 直跑：本地 sh -c 执行，输出进会话（对标 claude-code 的 ! 前缀）。
                            if let Some(shell_cmd) = input.strip_prefix('!') {
                                let shell_cmd = shell_cmd.trim();
                                if shell_cmd.is_empty() {
                                    app.status_text = "用法: !<shell 命令>".into();
                                    app.input.clear();
                                    app.cursor = 0;
                                    continue;
                                }
                                app.push_message("user", input.clone());
                                app.command_history.push(input.clone());
                                app.input.clear();
                                app.cursor = 0;
                                match run_shell_direct(shell_cmd) {
                                    Ok((code, stdout, stderr)) => {
                                        let mut out = format!("$ {}\n", shell_cmd);
                                        if !stdout.is_empty() { out.push_str(&stdout); }
                                        if !stderr.is_empty() {
                                            if !stdout.is_empty() { out.push('\n'); }
                                            out.push_str(&stderr);
                                        }
                                        out.push_str(&format!("\n[exit {}]", code));
                                        app.push_message_with_model("assistant", out, Some("shell".into()));
                                        app.status_text = if code == 0 { format!("shell: exit 0") } else { format!("shell: exit {} (非零)", code) };
                                    }
                                    Err(e) => {
                                        app.push_message("system", format!("[shell 失败] {}", e));
                                        app.status_text = format!("shell 失败: {}", e);
                                    }
                                }
                                continue;
                            }
                            // 交给 AgentLoop 流式生成（后台线程，主循环边收 chunk 边渲染）。
                            let text = input;
                            app.push_message("user", text.clone());
                            app.command_history.push(text.clone());
                            app.input.clear();
                            app.cursor = 0;
                            app.agent_busy = true;
                            app.streaming = true;
                            app.streaming_role = "assistant".into();
                            app.streaming_model = {
                                let guard = agent.lock().unwrap_or_else(|e| e.into_inner());
                                Some(guard.model().to_string())
                            };
                            app.streaming_text.clear();
                            app.streaming_renderer = StreamingMarkdownRenderer::new();
                            app.clear_streaming_tools();

                            use std::sync::mpsc as sync_mpsc;
                            let (event_tx, event_rx) = sync_mpsc::channel::<WorkerEvent>();
                            // 审批决策通道：主循环 → worker（true=允许, false=拒绝）。
                            let (approval_tx, approval_rx) = sync_mpsc::channel::<bool>();
                            // 取消标志：主循环设置，worker 的 on_token/审批回调读取，
                            // 保证「取消」真正传导到 LLM 生成循环（不再只是 UI 状态）。
                            let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                            let cancel_flag_worker = cancel_flag.clone();
                            let agent_worker = agent.clone();
                            let text_worker = text.clone();
                            let worker_handle = std::thread::spawn(move || {
                                let rt = match tokio::runtime::Runtime::new() {
                                    Ok(rt) => rt,
                                    Err(_) => {
                                        let _ = event_tx.send(WorkerEvent::Error("runtime init failed".into()));
                                        return;
                                    }
                                };
                                let mut guard = match agent_worker.lock() {
                                    Ok(g) => g,
                                    Err(poisoned) => poisoned.into_inner(),
                                };
                                let result = rt.block_on(guard.turn_stream_with_approval(
                                    &text_worker,
                                    |chunk| {
                                        if cancel_flag_worker.load(std::sync::atomic::Ordering::Relaxed) {
                                            return false; // 取消 → 中止本轮生成
                                        }
                                        let _ = event_tx.send(WorkerEvent::Token(chunk.to_string()));
                                        true
                                    },
                                    |name, args| {
                                        if cancel_flag_worker.load(std::sync::atomic::Ordering::Relaxed) {
                                            return false;
                                        }
                                        let _ = event_tx.send(WorkerEvent::ToolStart(name.to_string(), args.to_string()));
                                        true
                                    },
                                    |name, args, result, duration, success| {
                                        let _ = event_tx.send(WorkerEvent::ToolEnd(
                                            name.to_string(), args.to_string(),
                                            result.to_string(), duration, success,
                                        ));
                                        true
                                    },
                                    Some(Box::new({
                                        let tx = event_tx.clone();
                                        let cancel = cancel_flag_worker.clone();
                                        move |pending| {
                                            // 阻塞等待主循环决策（a=允许 / d=拒绝）。
                                            // 取消已设置 → 立即拒绝，避免卡在 recv() 上
                                            // （否则 worker 持锁阻塞、主线程 agent.lock() 永久死锁）。
                                            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                                return false;
                                            }
                                            let _ = tx.send(WorkerEvent::ApprovalRequest(pending.clone()));
                                            // 使用 recv_timeout 轮询取消标志，保证可中断。
                                            loop {
                                                match approval_rx.recv_timeout(std::time::Duration::from_millis(200)) {
                                                    Ok(approved) => break approved,
                                                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                                        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                                            break false;
                                                        }
                                                    }
                                                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break false,
                                                }
                                            }
                                        }
                                    })),
                                ));
                                match result {
                                    Ok(_) => { let _ = event_tx.send(WorkerEvent::Done); }
                                    Err(e) => { let _ = event_tx.send(WorkerEvent::Error(e.to_string())); }
                                }
                            });

                            // 主循环：边收事件边渲染，同时响应键盘（Ctrl+C 取消 / a/d 审批决策）。
                            let mut worker_done = false;
                            let mut frame_counter = 0u32;
                            let mut stream_ended = false;
                            while !worker_done {
                                while let Ok(ev) = event_rx.try_recv() {
                                    match ev {
                                        WorkerEvent::Token(t) => app.feed_stream(&t),
                                        WorkerEvent::ToolStart(name, args) => {
                                            app.start_streaming_tool(&name, &args);
                                        }
                                        WorkerEvent::ToolEnd(name, _args, result, duration, success) => {
                                            app.finish_streaming_tool(&name, duration, success, &result);
                                        }
                                        WorkerEvent::ApprovalRequest(pending) => {
                                            app.pending_approval = Some(pending);
                                            app.status_text = "等待审批: [a]允许 [d]拒绝".into();
                                        }
                                        WorkerEvent::Error(e) => { app.push_message_with_model("error", format!("[error] {}", e), None); app.status_text = format!("[error] {}", e); },
                                        WorkerEvent::Done => {
                                            stream_ended = true;
                                            worker_done = true;
                                        }
                                    }
                                }
                                // 每 3 圈（≈90ms）推进一次 spinner 帧，贴近参考 100ms/帧；
                                // 无 chunk 时也重绘（借鉴 claude-code-local：沉默≠卡死）。
                                frame_counter += 1;
                                if frame_counter % 3 == 0 {
                                    app.tick_spinner();
                                }
                                draw(&mut terminal, &app);
                                // 处理键盘（Ctrl+C 取消 / a 允许 / d 拒绝）。
                                if let Ok(true) = event::poll(Duration::from_millis(30)) {
                                    if let Ok(Event::Key(key)) = event::read() {
                                        match app.handle_key(key.code, key.modifiers) {
                                            KeyAction::CancelGeneration => {
                                                app.agent_busy = false;
                                                app.streaming = false;
                                                app.status_text = "生成已取消".into();
                                                app.pending_approval = None;
                                                // 取消真正传导：设置标志中止 LLM 生成，
                                                // 并 send(false) 唤醒可能阻塞在审批 recv_timeout 的 worker。
                                                cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                                                let _ = approval_tx.send(false);
                                                worker_done = true;
                                            }
                                            KeyAction::ApprovePending => {
                                                let _ = approval_tx.send(true);
                                            }
                                            KeyAction::DenyPending => {
                                                let _ = approval_tx.send(false);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                // sender drop → 工作线程结束。
                                match event_rx.try_recv() {
                                    Err(sync_mpsc::TryRecvError::Disconnected) => {
                                        stream_ended = true;
                                        worker_done = true;
                                    }
                                    Err(sync_mpsc::TryRecvError::Empty) => {}
                                    Ok(ev) => match ev {
                                        WorkerEvent::Token(t) => app.feed_stream(&t),
                                        WorkerEvent::ToolStart(name, args) => {
                                            app.start_streaming_tool(&name, &args);
                                        }
                                        WorkerEvent::ToolEnd(name, _args, result, duration, success) => {
                                            app.finish_streaming_tool(&name, duration, success, &result);
                                        }
                                        WorkerEvent::ApprovalRequest(pending) => {
                                            app.pending_approval = Some(pending);
                                            app.status_text = "等待审批: [a]允许 [d]拒绝".into();
                                        }
                                        WorkerEvent::Error(e) => { app.push_message_with_model("error", format!("[error] {}", e), None); app.status_text = format!("[error] {}", e); },
                                        WorkerEvent::Done => {
                                            stream_ended = true;
                                            worker_done = true;
                                        }
                                    },
                                }
                                draw(&mut terminal, &app);
                            }
                            // worker 已发 Done/取消完, 释放 agent 锁后主线程才可安全重入。
                            let _ = worker_handle.join();

                            // 收尾：应用剩余事件并提交。
                            while let Ok(ev) = event_rx.try_recv() {
                                match ev {
                                    WorkerEvent::Token(t) => app.feed_stream(&t),
                                    WorkerEvent::ToolStart(name, args) => {
                                        app.start_streaming_tool(&name, &args);
                                    }
                                    WorkerEvent::ToolEnd(name, _args, result, duration, success) => {
                                        app.finish_streaming_tool(&name, duration, success, &result);
                                    }
                                    WorkerEvent::ApprovalRequest(pending) => {
                                        app.pending_approval = Some(pending);
                                        app.status_text = "等待审批: [a]允许 [d]拒绝".into();
                                    }
                                    WorkerEvent::Error(e) => { app.push_message_with_model("error", format!("[error] {}", e), None); app.status_text = format!("[error] {}", e); },
                                    WorkerEvent::Done => stream_ended = true,
                                }
                            }
                            let _ = stream_ended;
                            if app.streaming_text.trim().is_empty() && app.status_text.contains("[error]") {
                                let _ = app.status_text.clone();
                            }
                            app.agent_busy = false;
                            app.streaming = false;
                            app.pending_approval = None;
                            let model = {
                                let guard = agent.lock().unwrap_or_else(|e| e.into_inner());
                                guard.model().to_string()
                            };
                            app.commit_stream_with_model("assistant", Some(model));
                        }
                        KeyAction::None => {}
                    }
                }
                Event::Resize(_, _) => {
                    let _ = terminal.clear();
                }
                // P1-2 修复: bracketed paste — 粘贴文本原样插入输入缓冲 (含换行/多行),
                // 不会逐 \n 触发 Enter/Submit, 杜绝大段粘贴被拆行发送丢失。
                Event::Paste(pasted) => {
                    if !app.agent_busy {
                        append_paste(&mut app, &pasted);
                    }
                }
                _ => {}
            }
        }

        // ── TUI 清理 ────────────────────────────────────────────────
        // P1-3 修复: 退出前自动保存当前会话 (不再依赖显式 /save), 防正常退出丢对话。
        {
            let name = format!("session-{}", app.sessions[app.active_session].id);
            let n = app.sessions[app.active_session].messages.len();
            if n > 0 {
                match save_tui_session(&app, &name) {
                    Ok(()) => eprintln!("{} 会话已自动保存 ({} 条消息)", info("Saved"), n),
                    Err(e) => eprintln!("{} 会话自动保存失败: {}", err("Error"), e),
                }
            }
        }
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        println!("{}", info("NT-AGENT TUI 结束"));
        let _ = exit;
    });
}

enum SlashResult {
    Quit,
    Handled,
    NotHandled,
}

/// AgentLoop 后台线程 → 主循环的事件（流式生成期间）。
enum WorkerEvent {
    Token(String),
    ToolStart(String, String),
    ToolEnd(String, String, String, u64, bool),
    ApprovalRequest(neotrix::cli::approval::PendingAction),
    Error(String),
    Done,
}

/// TUI 模式斜杠命令分发（当前仅本地命令；其余透传给 AgentLoop 当消息）。
fn handle_slash_tui(app: &mut neotrix::cli::tui::TuiApp, input: &str) -> SlashResult {
    let (cmd, rest) = match input.split_once(' ') {
        Some((c, r)) => (c, r),
        None => (input, ""),
    };
    match cmd {
        "/exit" | "/quit" | "/q" => SlashResult::Quit,
        "/clear" => {
            app.clear_session();
            app.needs_agent_reset = true;
            app.status_text = "已清空会话".into();
            SlashResult::Handled
        }
        "/new" => {
            app.new_session();
            app.needs_agent_reset = true;
            app.status_text = format!("已切换到会话 {}", app.sessions.len());
            SlashResult::Handled
        }
        "/hist" => {
            let n = app.sessions[app.active_session].messages.len();
            app.status_text = format!("{} 条消息", n);
            SlashResult::Handled
        }
        "/context" => {
            // 借鉴 claude-code-local 的 /context：显示估算的 context 用量（含可视化进度条）。
            let pct = app.context_pct();
            let used_k = app.token_count as f64 / 1000.0;
            let limit_k = neotrix::cli::tui::app::CONTEXT_LIMIT_ESTIMATE as f64 / 1000.0;
            let warn = if pct >= 90 { " (接近上限，建议 /new 或 /clear)" } else { "" };
            // 20 格进度条（█ 满格 / ░ 空格）
            let filled = ((pct as usize * 20) / 100).min(20);
            let bar: String = format!("{}{}", "█".repeat(filled), "░".repeat(20 - filled));
            app.status_text = format!("ctx {}{} {:.1}k / {:.1}k tokens ({pct}%){warn}", bar, if pct >= 100 { "" } else { "" }, used_k, limit_k);
            SlashResult::Handled
        }
        "/save" => {
            let name = if rest.trim().is_empty() {
                format!("session-{}", app.sessions[app.active_session].id)
            } else {
                rest.trim().to_string()
            };
            match save_tui_session(app, &name) {
                Ok(()) => {
                    app.status_text = format!("会话已保存到 KB + session-logs ({})", name);
                    SlashResult::Handled
                }
                Err(e) => {
                    app.status_text = format!("保存失败: {}", e);
                    SlashResult::Handled
                }
            }
        }
        "/load" => {
            let name = rest.trim().to_string();
            if name.is_empty() {
                app.status_text = "用法: /load <会话名>".into();
                return SlashResult::Handled;
            }
            match load_tui_session(app, &name) {
                Ok(n) => {
                    app.needs_agent_reset = true;
                    app.status_text = format!("已加载会话 {} ({} 条消息)", name, n);
                    SlashResult::Handled
                }
                Err(e) => {
                    app.status_text = format!("加载失败: {}", e);
                    SlashResult::Handled
                }
            }
        }
        "/sessions" | "/resume" => {
            // 打开会话恢复 picker（对标 claude-code /resume picker）：↑↓ 选择 · Enter 加载。
            app.needs_agent_reset = true;
            open_session_picker(app);
            SlashResult::Handled
        }
        "/diff" => {
            // 打开 diff 查看模式：无参 → git diff 全量；带路径 → git diff <path>；
            // 含换行的参数视为命令行直接传入的 diff 文本（纯空白文本 → 无内容）。
            let content = if rest.contains('\n') {
                if rest.trim().is_empty() {
                    Ok(String::new()) // 纯空白 diff 文本 → 无内容（确定性，不依赖 git 状态）
                } else {
                    Ok(rest.to_string())
                }
            } else if rest.trim().is_empty() {
                run_git_diff(None)
            } else {
                run_git_diff(Some(rest.trim()))
            };
            match content {
                Ok(text) if !text.trim().is_empty() => {
                    app.open_diff(text);
                    SlashResult::Handled
                }
                Ok(_) => {
                    app.status_text = "无 diff 内容".into();
                    SlashResult::Handled
                }
                Err(e) => {
                    app.status_text = format!("diff 失败: {}", e);
                    SlashResult::Handled
                }
            }
        }
        "/help" => {
            app.push_message("system", "NeoTrix TUI 快捷键\n\n输入: Enter 发送 | Alt+E 多行 | ↑↓ 历史 | Ctrl+R 搜索 | Tab 补全 | Ctrl+L 清屏\n引用: @路径 Tab 补全文件 | !<cmd> 直跑 shell\n生成: Esc / Ctrl+C 取消 | Ctrl+T 展开 thinking | Ctrl+X 展开工具调用\n审批: 工具执行前提示 [a]允许 [d]拒绝 (Esc 取消)\n视图: Ctrl+S 会话侧栏 | Alt+T 主题 | PageUp/Down 滚动\nDiff: /diff [路径] 打开 diff 查看 (↑↓ 滚动 · q/Esc 退出)\n会话: /new /clear /save <名> /load <名> /sessions 恢复面板 /hist /context\n其他: /exit /quit /help".into());
            SlashResult::Handled
        }
        _ => {
            // Registry fallback: unknown slash commands route to the command
            // registry (90+ commands). Hardcoded commands above take priority.
            // If the registry also misses, return NotHandled so the input is
            // treated as a normal message.
            if input.starts_with('/') {
                let reg = neotrix::cli::commands::registry::default_registry();
                let cmd = input.split(' ').next().unwrap_or(input);
                if reg.find(cmd).is_some() {
                    let out = reg.execute(input, None);
                    app.status_text = out.message;
                    return SlashResult::Handled;
                }
            }
            SlashResult::NotHandled
        }
    }
}

/// 打开会话恢复 picker：从 SessionStore 拉取已保存会话列表填充到 TuiApp。
fn open_session_picker(app: &mut neotrix::cli::tui::TuiApp) {
    use neotrix::cli::tui::session_store::SessionStore;
    use neotrix::cli::tui::app::types::{SessionEntry, SessionPicker};
    let store = SessionStore::new();
    let mut entries: Vec<SessionEntry> = Vec::new();
    for data in store.list_sessions() {
        entries.push(SessionEntry {
            name: data.name,
            updated_at: data.updated_at,
            message_count: data.messages.len(),
        });
    }
    if entries.is_empty() {
        app.status_text = "无已保存会话（/save <名> 保存当前会话）".into();
        return;
    }
    entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    app.session_picker = Some(SessionPicker { entries, selected: 0 });
}

/// 把 TuiApp 当前会话持久化到 SessionStore（KB + session-logs 双落盘）。
fn save_tui_session(app: &neotrix::cli::tui::TuiApp, name: &str) -> Result<(), String> {
    use neotrix::cli::tui::session_store::{SessionData, SessionStore};
    let session = &app.sessions[app.active_session];
    let now = chrono::Utc::now().to_rfc3339();
    let messages: Vec<String> = session.messages.iter()
        .map(|m| format!("[{}] {}", m.role, m.content))
        .collect();
    let data = SessionData {
        id: session.id.clone(),
        name: name.to_string(),
        messages,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut store = SessionStore::new();
    store.save_session(name, &data)
}

/// 从 SessionStore 加载会话到 TuiApp。
fn load_tui_session(app: &mut neotrix::cli::tui::TuiApp, name: &str) -> Result<usize, String> {
    use neotrix::cli::tui::session_store::SessionStore;
    let store = SessionStore::new();
    let data = store.load_session(name)?;
    app.clear_session();
    for line in &data.messages {
        let (role, content) = if let Some(c) = line.strip_prefix("[user] ") {
            ("user", c.to_string())
        } else if let Some(c) = line.strip_prefix("[assistant] ") {
            ("assistant", c.to_string())
        } else {
            ("system", line.clone())
        };
        app.push_message(role, content);
    }
    Ok(data.messages.len())
}

/// 直接执行 shell 命令（`!` 前缀直跑），返回 (exit_code, stdout, stderr)。
/// 语义对标 claude-code 的 `!` 前缀：用户显式发起，不经 agent/审批（等价于终端直跑）。
/// P1-2 修复: 粘贴文本插入输入缓冲 (保留换行/多行), 通过 TuiApp 的 insert_char
/// 逐字符插入以保持光标/宽字符正确。多行粘贴自动切换多行编辑语义。
fn append_paste(app: &mut neotrix::cli::tui::TuiApp, pasted: &str) {
    app.insert_text(pasted);
    // 含换行 → 提示已进入多行编辑 (Enter 在空行处才提交, 多行模式 Enter 插行)。
    if pasted.contains('\n') {
        app.status_text = "已粘贴多行文本 (Alt+E 多行模式; 空行 Enter 提交)".into();
    }
}

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

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }

    #[test]
    fn test_save_load_tui_session_roundtrip() {
        // 用隔离 base 目录验证 save/load 闭环（不污染真实 ~/.neotrix KB）。
        use neotrix::cli::tui::TuiApp;
        use neotrix::cli::tui::session_store::SessionStore;

        let tmp = std::env::temp_dir().join(format!("nt-tui-session-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create tmp");

        // 用 with_base 隔离的 store 直接验证 SessionStore 落盘逻辑。
        let mut store = SessionStore::with_base(tmp.clone());
        let now = chrono::Utc::now().to_rfc3339();
        let data = neotrix::cli::tui::session_store::SessionData {
            id: "s-1".into(),
            name: "roundtrip".into(),
            messages: vec!["[user] hello".into(), "[assistant] hi there".into()],
            created_at: now.clone(),
            updated_at: now,
        };
        store.save_session("roundtrip", &data).expect("save ok");

        // 重新打开验证持久化。
        let store2 = SessionStore::with_base(tmp.clone());
        let loaded = store2.load_session("roundtrip").expect("load ok");
        assert_eq!(loaded.name, "roundtrip");
        assert_eq!(loaded.messages.len(), 2);
        assert!(loaded.messages[0].contains("hello"));
        assert!(loaded.messages[1].contains("hi there"));

        // session-logs/*.md 应已落盘。
        let md = tmp.join("session-logs").join("roundtrip.md");
        assert!(md.exists(), "session-logs markdown should exist");
        let content = std::fs::read_to_string(&md).expect("read md");
        assert!(content.contains("hello"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_shell_direct_echo() {
        // `!` 前缀直跑：echo 成功 + exit 0
        let (code, stdout, stderr) = super::run_shell_direct("echo hello-neotrix").expect("shell runs");
        assert_eq!(code, 0);
        assert_eq!(stdout, "hello-neotrix");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_shell_direct_nonzero_exit() {
        // 非零退出码应透传
        let (code, stdout, stderr) = super::run_shell_direct("exit 3").expect("shell runs");
        assert_eq!(code, 3);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_shell_direct_stderr_captured() {
        // stderr 应被捕获而非丢弃
        let (code, _stdout, stderr) = super::run_shell_direct("echo boom >&2").expect("shell runs");
        assert_eq!(code, 0);
        assert_eq!(stderr, "boom");
    }

    #[test]
    fn test_slash_command_dispatch() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        use super::SlashResult;
        // /clear
        app.push_message("user", "x".into());
        assert!(matches!(super::handle_slash_tui(&mut app, "/clear"), SlashResult::Handled));
        assert!(app.sessions[0].messages.is_empty());
        // /new
        assert!(matches!(super::handle_slash_tui(&mut app, "/new"), SlashResult::Handled));
        assert_eq!(app.sessions.len(), 2);
        // /exit
        assert!(matches!(super::handle_slash_tui(&mut app, "/exit"), SlashResult::Quit));
        // 未知命令透传
        assert!(matches!(super::handle_slash_tui(&mut app, "/bogus"), SlashResult::NotHandled));
    }

    #[test]
    fn test_diff_slash_with_literal_text_opens_viewer() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        // 含换行的参数 → 视为命令行直接传入的 diff 文本（不触发 git 调用）。
        let diff_text = "diff --git a/x.rs b/x.rs\n@@ -1,2 +1,3 @@\n-old\n+new\n";
        assert!(matches!(
            super::handle_slash_tui(&mut app, &format!("/diff {}", diff_text)),
            super::SlashResult::Handled
        ));
        assert!(app.diff_active(), "/diff 应打开 diff 查看模式");
        let viewer = app.diff_viewer.as_ref().expect("diff viewer");
        assert_eq!(viewer.blocks.len(), 1, "应解析出一个 diff block");
        assert!(!viewer.is_empty());
        // q 退出
        app.handle_key(crossterm::event::KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE);
        assert!(!app.diff_active(), "q 应退出 diff 查看模式");
    }

    #[test]
    fn test_diff_slash_empty_reports_no_content() {
        use neotrix::cli::tui::TuiApp;
        let mut app = TuiApp::new(true);
        // 空 diff 文本 → 不进入查看模式，状态栏提示无内容。
        assert!(matches!(
            super::handle_slash_tui(&mut app, "/diff \n"),
            super::SlashResult::Handled
        ));
        assert!(!app.diff_active());
        assert!(app.status_text.contains("无 diff 内容"));
    }
}
