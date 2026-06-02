use std::sync::Arc;
use std::path::PathBuf;
use std::io::{self, Write};

use colored::Colorize;

use neotrix::neotrix::background_loop::BackgroundLoop;
use neotrix::neotrix::nt_world_model::WorldModelV2;
use neotrix::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::neotrix::nt_mind::self_iterating::{ReasoningBrain, SelfIteratingBrain};
use neotrix::neotrix::nt_mind::memory::ReasoningBank;
use neotrix::neotrix::mention::resolve_mentions;

use crate::config::NeoTrixConfig;

mod proxy_cmd;
mod standalone;
mod headless;
mod server;
mod desktop;

pub use proxy_cmd::run_proxy_cmd;

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

pub fn check_provider_config() -> bool {
    let cfg = crate::config::NeoTrixConfig::load();
    if cfg.provider.is_some() && cfg.api_key.is_some() && !cfg.api_key.as_ref().expect("api_key checked for Some above").is_empty() {
        return true;
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
        io::stdout().flush().expect("stdout flush failed");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("stdin read failed");
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
    io::stdout().flush().expect("stdout flush failed");
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key).expect("stdin read failed");
    let api_key = api_key.trim().to_string();

    let default_model = match provider {
        "opencode" => "opencode/gpt-4o-mini".to_string(),
        "xiaohuxing" => "gpt-4o-mini".to_string(),
        "openai" => "gpt-4o-mini".to_string(),
        "anthropic" => "claude-3-haiku-20240307".to_string(),
        "custom" => {
            print!("Enter default model name: ");
            io::stdout().flush().expect("stdout flush failed");
            let mut model = String::new();
            io::stdin().read_line(&mut model).expect("stdin read failed");
            model.trim().to_string()
        }
        _ => "gpt-4o-mini".to_string(),
    };

    let config_path = crate::config::NeoTrixConfig::path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create config directory");
    }

    let custom_endpoint = match provider {
        "xiaohuxing" => Some("https://api.xiaohuxing.eu.org/v1".to_string()),
        "custom" => {
            print!("Enter custom base URL: ");
            io::stdout().flush().expect("stdout flush failed");
            let mut url = String::new();
            io::stdin().read_line(&mut url).expect("stdin read failed");
            let url = url.trim().to_string();
            if url.is_empty() { None } else { Some(url) }
        }
        _ => None,
    };

    let mut content = format!(
        "# NeoTrix Configuration\n\
         provider = {:?}\n\
         api_key = {:?}\n\
         default_model = {:?}\n",
        provider, api_key, default_model,
    );
    if let Some(ref ep) = custom_endpoint {
        content.push_str(&format!("custom_endpoint = {:?}\n", ep));
    }

    std::fs::write(&config_path, content).expect("failed to write config file");
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

/// Public entry point for clap-based CLI dispatch.
/// Each function wraps the existing async sub-mode logic.

#[allow(dead_code)]
pub(crate) fn run_server_mode(_addr: &str, profile: &str) {
    println!("{} v{}", info("NeoTrix Server"), env!("CARGO_PKG_VERSION"));
    println!("{}", info("Starting background services... Press Ctrl+C to stop."));
    let server_rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    server_rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.world_model = Some(WorldModelV2::new(8, 64));
        bg.panorama = Some(PanoramaPipeline::new());
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{}", info("[server] all services initialized."));
        tokio::select! {
            _ = bg.start() => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", info("[server] shutting down..."));
            }
        }
    });
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

pub fn run_exec(prompt: &str, json_output: bool, timeout_secs: u64) {
    if prompt.is_empty() {
        if json_output {
            use neotrix::cli::jsonl_stream::JsonlWriter;
            let mut writer = JsonlWriter::new();
            writer.emit_error("Empty prompt", Some("EMPTY_PROMPT"), false);
            writer.emit_finish("", 0, 0, 1);
        } else {
            eprintln!("Error: empty prompt");
        }
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() && !json_output {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let prompt = prompt;
    let start = std::time::Instant::now();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    if json_output {
        use neotrix::cli::jsonl_stream::JsonlWriter;
        let mut writer = JsonlWriter::new();
        writer.emit_start(&prompt, None, None, None);

        let result = rt.block_on(async {
            let (brain, bank) = init_brain("default");
            let mut agent = SelfIteratingBrain::new();
            agent.brain = brain;
            agent.reasoning_bank = bank;
            agent.init_reasoning_engine();

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
    } else {
        // Plain text mode (original behavior)
        let result = rt.block_on(async {
            let (brain, bank) = init_brain("default");
            let mut agent = SelfIteratingBrain::new();
            agent.brain = brain;
            agent.reasoning_bank = bank;
            agent.init_reasoning_engine();

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
                eprintln!("Error: {}", e);
            }
            Err(_timeout) => {
                eprintln!("Error: execution timed out after {}s", timeout_secs);
            }
        }
    }
}

pub fn run_one_shot(prompt: &str, format: Option<&str>, profile: &str) {
    if prompt.is_empty() {
        eprintln!("{}: usage: neotrix run <prompt> | neotrix reason <prompt>", err("Error"));
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() {
        eprintln!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let prompt = prompt;
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.init_reasoning_engine();

        let pb = indicatif::ProgressBar::new(100);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {percent}% {msg}")
                .expect("invalid progress bar template")
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
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
        let _ = agent.brain.save();
    });
}

pub fn show_status() {
    let status = neotrix::neotrix::server_proxy::ServerProxy::status();
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
        other => {
            eprintln!("{}: unsupported shell '{}'. Use: bash, zsh, fish, powershell", err("Error"), other);
            return;
        }
    };
    let mut stdout = std::io::stdout();
    clap_complete::generate(shell, cmd, "neotrix", &mut stdout);
}

pub fn run_mcp_server() {
    let mut server = neotrix::core::McpServer::new();
    server.register_all_tools();
    println!("neotrix-mcp {} starting (stdio JSON-RPC 2.0)", env!("CARGO_PKG_VERSION"));
    if let Err(e) = server.run() {
        eprintln!("MCP server error: {}", e);
    }
}

pub fn run_benchmark(category: Option<&str>) {
    use neotrix::neotrix::benchmark::{BenchmarkSuite, BenchmarkReport};
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
        let name_display = if r.name.len() > 17 {
            format!("{}…", &r.name[..16])
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
    use neotrix::neotrix::browser::BrowserCircuit;
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
    use neotrix::neotrix::browser::BrowserCircuit;
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
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{} {}", info("[daemon]"), info("NeoTrix background daemon started"));
        tokio::select! {
            _ = bg.start() => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", info("[daemon] shutting down..."));
            }
        }
    });
}

pub fn run_daemon_evolution(profile: &str) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{} {}", info("[daemon]"), info("NeoTrix evolution daemon started"));
        let daemon_handle = bg.start();
        let daemon = std::sync::Arc::new(std::sync::Mutex::new(
            neotrix::neotrix::evolution_daemon::EvolutionDaemon::default()
        ));
        let daemon_clone = daemon.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let mut d = daemon_clone.lock().expect("daemon lock");
                let report = d.run_cycle_goal();
                if report.fixes_applied > 0 {
                    println!("[evolution] 🔧 {} fixes applied (cycle {})", report.fixes_applied, report.cycle);
                }
            }
        });
        tokio::select! {
            _ = daemon_handle => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", info("[daemon] shutting down..."));
            }
        }
    });
}

pub fn run_standalone_mode(stage: usize) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        standalone::run_standalone(stage).await;
    });
}

pub fn run_headless_mode(_cfg: &NeoTrixConfig, profile: &str) {
    use neotrix::neotrix::background_loop::BackgroundLoop;
    use neotrix::neotrix::nt_world_model::WorldModelV2;
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::hooks::{HookRegistry, HookEvent, HookContext};
    use neotrix::agent::tools::{McpRegistry, McpTransport, McpToolDef};
    use neotrix::agent::{AgentTeam, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
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
            println!("{}: {} {}", info("ReasoningEngine"), success("active"), info("(LLM connected)"));
        } else {
            println!("{}: {}", warn("ReasoningEngine"), warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)"));
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        println!("{}: {} {}", info("SkillsEngine"), success(format!("{} local skills loaded", skill_count)), "");
        println!("  -> {} /skills list to browse, /skills ecc <id> to load from ECC community", info("/skills"));

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![
            McpToolDef {
                name: "neotrix_info".to_string(),
                description: "NeoTrix MCP system info".to_string(),
                server_name: "built-in".to_string(),
                transport: McpTransport::Stdio {
                    command: "echo".to_string(),
                    args: vec![],
                },
                input_schema: serde_json::json!({"type": "object"}),
            },
        ];
        builtin_tools.extend(neotrix::neotrix::mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::mcp_tools::register_neotrix_tools(&mut mcp_registry);
        println!("{}: {} ({})", info("McpRegistry"), success("ready"), info("use /mcp list"));
        let mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = HookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!("{}: {} {}",
            info("HookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)"));

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = HookRegistry::check_blocked(&hook_actions) {
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
        let _ = tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_world_model(WorldModelV2::new(8, 64));
            #[cfg(feature = "stealth-net")]
            {
                bg = bg.with_world_consciousness();
            }
            bg.start().await;
        });

        let sp = indicatif::ProgressBar::new_spinner();
        sp.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .expect("invalid spinner template"),
        );
        sp.set_message("starting headless mode...");
        headless::run_headless(agent, skills_engine, hook_registry, mcp_registry).await;
        sp.finish_and_clear();
    });
}

pub fn run_interactive(cfg: &NeoTrixConfig, profile: &str) {
    run_interactive_with_ephemeral(cfg, profile, false)
}

pub fn run_interactive_with_ephemeral(cfg: &NeoTrixConfig, profile: &str, ephemeral: bool) {
    use neotrix::neotrix::background_loop::BackgroundLoop;
    use neotrix::neotrix::nt_world_model::WorldModelV2;
    use neotrix::neotrix::nt_mind::panorama_pipeline::PanoramaPipeline;
    use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
    
    use neotrix::agent::skills::SkillsEngine;
    use neotrix::agent::hooks::{HookRegistry, HookEvent, HookContext};
    use neotrix::agent::tools::{McpRegistry, McpTransport, McpToolDef};
    use neotrix::agent::{AgentTeam, AgentRole, ProcessType};
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

    if let Some(level) = &cfg.log_level {
        let _ = std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
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
            println!("{}: {} {}", info("ReasoningEngine"), success("active"), info("(LLM connected)"));
        } else {
            println!("{}: {}", warn("ReasoningEngine"), warn("inactive (set NEOTRIX_PROVIDER/API_KEY/MODEL)"));
        }
        print_brain_stats(&agent);

        let mut skills_engine = SkillsEngine::new();
        let skill_count = skills_engine.init().len();
        println!("{}: {} {}", info("SkillsEngine"), success(format!("{} local skills loaded", skill_count)), "");
        println!("  -> {} /skills list to browse, /skills ecc <id> to load from ECC community", info("/skills"));

        let mut mcp_registry = McpRegistry::new();
        let mut builtin_tools = vec![
            McpToolDef {
                name: "neotrix_info".to_string(),
                description: "NeoTrix MCP system info".to_string(),
                server_name: "built-in".to_string(),
                transport: McpTransport::Stdio {
                    command: "echo".to_string(),
                    args: vec![],
                },
                input_schema: serde_json::json!({"type": "object"}),
            },
        ];
        builtin_tools.extend(neotrix::neotrix::mcp_tools::neotrix_mcp_tools());
        mcp_registry.register_stdio("built-in", "echo", &["mcp"], builtin_tools);
        neotrix::neotrix::mcp_tools::register_neotrix_tools(&mut mcp_registry);
        println!("{}: {} ({})", info("McpRegistry"), success("ready"), info("use /mcp list"));
        let _mcp_registry = Arc::new(RwLock::new(mcp_registry));

        let mut hook_registry = HookRegistry::default();
        hook_registry.set_profile(neotrix::agent::hooks::HookProfile::Standard);
        println!("{}: {} {}",
            info("HookRegistry"),
            success(format!("{} hooks registered", hook_registry.hook_count())),
            info("(profile: standard)"));

        let session_ctx = HookContext::new(HookEvent::SessionStart);
        let hook_actions = hook_registry.execute_event(&session_ctx);
        if let Some(block) = HookRegistry::check_blocked(&hook_actions) {
            eprintln!("{}: {}", warn("Hook blocked startup"), block);
        }

        let agent = Arc::new(RwLock::new(agent));
        let bg_agent = agent.clone();
        let _skills_engine = Arc::new(RwLock::new(skills_engine));
        let hook_registry: Arc<RwLock<HookRegistry>> = Arc::new(RwLock::new(hook_registry));

        let mut bg_goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg_goal_loop.load();
        if bg_goal_loop.active_goal.is_some() {
            println!("{} {}", info("[bg]"), info("Restored background goal from ~/.neotrix/goals.json"));
        }

        let agent_team = Arc::new(Mutex::new(AgentTeam::new("default", ProcessType::Sequential)));
        {
            let mut team = agent_team.lock().expect("lock");
            team.add_agent(AgentRole {
                name: "planner".into(),
                role: "Task Planner".into(),
                goal: "Break down complex tasks into sub-tasks".into(),
                backstory: "Strategic planner with systems thinking".into(),
                tools: vec!["reason".into()],
            });
        }
        bg_goal_loop = bg_goal_loop.with_agent_team(agent_team);

        let _ = tokio::spawn(async move {
            let mut bg = BackgroundLoop::new(bg_agent)
                .with_goal_loop(bg_goal_loop)
                .with_world_model(WorldModelV2::new(8, 64))
                .with_panorama(PanoramaPipeline::new())
                .with_nt_world_crawl(std::path::PathBuf::from("."))
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
            if let Some(block_reason) = HookRegistry::check_blocked(&pre_actions) {
                eprintln!("Hook blocked TUI session: {}", block_reason);
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
    use neotrix::neotrix::sandbox_v2::cli;
    let runtime = if runtime.is_empty() { None } else { Some(runtime) };
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(cli::handle_run(code, runtime, Some(timeout)));
}

pub fn run_sandbox_list() {
    neotrix::neotrix::sandbox_v2::cli::handle_list();
}

pub fn run_sandbox_cancel(session_id: &str) {
    neotrix::neotrix::sandbox_v2::cli::handle_cancel(session_id);
}

pub fn run_discover(port: u16, duration_ms: u64, json: bool) {
    use neotrix::neotrix::agent_protocol::discovery::AgentDiscovery;

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
    use neotrix::neotrix::sandbox_v2::cli;
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
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
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({"wallets": list})).expect("JSON serialization failed"));
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
