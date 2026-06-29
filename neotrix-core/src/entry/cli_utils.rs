use std::io::{self, Write};

use colored::Colorize;

use neotrix::neotrix::nt_io_mention::resolve_mentions;

use super::{err, info, init_brain, success, warn};

pub fn check_provider_config() -> bool {
    let cfg = crate::config::NeoTrixConfig::load();
    if let (Some(_), Some(api_key)) = (&cfg.provider, &cfg.api_key) {
        if !api_key.is_empty() {
            return true;
        }
    }
    false
}

pub fn run_provider_wizard() {
    log::info!("╔══════════════════════════════════════════╗");
    log::info!("║  NeoTrix — First-Time Provider Setup    ║");
    log::info!("╚══════════════════════════════════════════╝");
    log::info!("");

    log::info!("No LLM provider configured yet.");
    log::info!("");
    log::info!("Available providers:");
    log::info!("  1) opencode.ai (free tier available)");
    log::info!("  2) xiaohuxing (OpenAI-compatible proxy)");
    log::info!("  3) OpenAI");
    log::info!("  4) Anthropic");
    log::info!("  5) Custom (OpenAI-compatible)");
    log::info!("");

    let provider = loop {
        print!("Select provider [1-5]: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        match input.trim() {
            "1" => break "opencode",
            "2" => break "xiaohuxing",
            "3" => break "openai",
            "4" => break "anthropic",
            "5" => break "custom",
            _ => {
                log::info!("Invalid selection, try again.");
                continue;
            }
        };
    };

    print!("Enter your API key (or press Enter to skip): ");
    let _ = io::stdout().flush();
    let mut api_key = String::new();
    let _ = io::stdin().read_line(&mut api_key);
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
            let _ = io::stdin().read_line(&mut model);
            model.trim().to_string()
        }
        _ => "gpt-4o-mini".to_string(),
    };

    let config_path = crate::config::NeoTrixConfig::path();
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("failed to create config directory: {}", e);
        }
    }

    let custom_endpoint = match provider {
        "xiaohuxing" => Some("https://api.xiaohuxing.eu.org/v1".to_string()),
        "custom" => {
            print!("Enter custom base URL: ");
            let _ = io::stdout().flush();
            let mut url = String::new();
            let _ = io::stdin().read_line(&mut url);
            let url = url.trim().to_string();
            if url.is_empty() {
                None
            } else {
                Some(url)
            }
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

    let tmp = config_path.with_extension("tmp");
    if let Err(e) = std::fs::write(&tmp, content) {
        log::error!("failed to write config file: {}", e);
    }
    if let Err(e) = std::fs::rename(&tmp, &config_path) {
        log::error!("failed to rename config file: {}", e);
    }
    log::info!("");
    log::info!("✅ Configuration saved to: {}", config_path.display());
    log::info!("   Provider: {}", provider);
    if !api_key.is_empty() {
        log::info!(
            "   API Key: ****{}",
            &api_key[api_key.len().saturating_sub(4)..]
        );
    }
    log::info!("");
    log::info!("You can change these settings anytime by editing the config file.");
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
                log::error!("{}: {}", err("Read file error"), e);
                String::new()
            });
        }
        log::error!("{}: file not found: {}", err("Error"), f);
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
            log::error!("Error: empty prompt");
        }
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() && !json_output {
        log::info!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let prompt = prompt;
    let start = std::time::Instant::now();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };

    if json_output {
        use neotrix::cli::jsonl_stream::JsonlWriter;
        let mut writer = JsonlWriter::new();
        writer.emit_start(&prompt, None, None, None);

        let result = rt.block_on(async {
            let (brain, bank) = init_brain("default");
            let mut agent = neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
            agent.brain = brain;
            agent.reasoning_bank = bank;
            agent.init_reasoning_engine();

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let prompt_owned = prompt.to_string();
            let handle = tokio::task::spawn_blocking(move || {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt_owned)
                } else {
                    let task_type = neotrix::neotrix::nt_expert_routing::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            });
            tokio::time::timeout(timeout, handle).await
        });

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(Ok(response))) => {
                let tokens_used = (response.len() / 4) as u32;
                writer.emit_message("assistant", &response, Some(tokens_used));
                writer.emit_finish(&response, tokens_used, elapsed, 0);
            }
            Ok(Ok(Err(e))) => {
                let msg = e.to_string();
                writer.emit_error(&msg, Some("LLM_ERROR"), true);
                writer.emit_finish("", 0, elapsed, 1);
            }
            Ok(Err(join_err)) => {
                let msg = format!("Task panicked: {}", join_err);
                writer.emit_error(&msg, Some("PANIC"), true);
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
            let mut agent = neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
            agent.brain = brain;
            agent.reasoning_bank = bank;
            agent.init_reasoning_engine();

            let timeout = tokio::time::Duration::from_secs(timeout_secs);
            let prompt_owned = prompt.to_string();
            let handle = tokio::task::spawn_blocking(move || {
                if let Some(ref mut engine) = agent.reasoning_engine {
                    engine.reason(&prompt_owned)
                } else {
                    let task_type = neotrix::neotrix::nt_expert_routing::TaskType::General;
                    let r = agent.iterate(task_type);
                    Ok(format!(
                        "Learned: {:.3} → {:.3}",
                        r.score_before, r.score_after
                    ))
                }
            });
            tokio::time::timeout(timeout, handle).await
        });

        let _elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(Ok(response))) => {
                log::info!("{}", response);
            }
            Ok(Ok(Err(e))) => {
                log::error!("Error: {}", e);
            }
            Ok(Err(join_err)) => {
                log::error!("Task panicked: {}", join_err);
            }
            Err(_timeout) => {
                log::error!("Error: execution timed out after {}s", timeout_secs);
            }
        }
    }
}

pub fn run_one_shot(prompt: &str, format: Option<&str>, profile: &str) {
    if prompt.is_empty() {
        log::error!(
            "{}: usage: neotrix run <prompt> | neotrix reason <prompt>",
            err("Error")
        );
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let (prompt, mentions) = resolve_mentions(prompt, &cwd);
    if !mentions.is_empty() {
        log::info!("📎 Resolved {} file mention(s)", mentions.len());
    }
    let prompt = prompt;
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        agent.init_reasoning_engine();

        let pb = indicatif::ProgressBar::new(100);
        let bar_style = match indicatif::ProgressStyle::default_bar()
            .template("{spinner:.blue} [{bar:40.cyan/blue}] {percent}% {msg}")
        {
            Ok(s) => s.progress_chars("█▉▊▋▌▍▎▏ "),
            Err(e) => {
                log::error!("invalid progress bar template: {}", e);
                indicatif::ProgressStyle::default_bar()
            }
        };
        pb.set_style(bar_style);
        pb.set_message("reasoning...");

        let result = if let Some(ref mut engine) = agent.reasoning_engine {
            pb.inc(30);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let r = engine.reason(&prompt);
            pb.finish_with_message("done");
            r
        } else {
            let task_type = neotrix::neotrix::nt_expert_routing::TaskType::General;
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
                    log::info!(
                        "{}",
                        serde_json::to_string_pretty(&json).unwrap_or(response)
                    );
                } else {
                    log::info!("\n{}", response);
                }
            }
            Err(e) => {
                if format == Some("json") {
                    let json = serde_json::json!({
                        "success": false,
                        "error": e.to_string(),
                    });
                    log::error!(
                        "{}",
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    );
                } else {
                    log::error!("{}: {}", err("Reasoning error"), e);
                }
            }
        }
        let _ = agent.brain.save();
    });
}

pub fn show_status() {
    log::info!("{}", info("╭─ NeoTrix Status ───────────────────────╮"));
    log::info!(
        "│ {}  {:<40} │",
        info("Version:"),
        env!("CARGO_PKG_VERSION")
    );
    log::info!("│ {}  {:<40} │", info("VSA Dims:"), neotrix::VSA_DIM);
    log::info!("{}", info("╰─────────────────────────────────────────╯"));
}

pub fn generate_completions(shell: &str, cmd: &mut clap::Command) {
    use clap_complete::Shell;
    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        other => {
            log::error!(
                "{}: unsupported shell '{}'. Use: bash, zsh, fish, powershell",
                err("Error"),
                other
            );
            return;
        }
    };
    let mut stdout = std::io::stdout();
    clap_complete::generate(shell, cmd, "neotrix", &mut stdout);
}

pub fn run_benchmark(category: Option<&str>) {
    use neotrix::neotrix::nt_mind_benchmark::{BenchmarkReport, BenchmarkSuite};
    use neotrix::CapabilityVector;

    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix/brain.json");
    let cap: CapabilityVector = if path.exists() {
        let json = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&json).unwrap_or_else(|e| {
            log::warn!(
                "{}",
                warn(format!("failed to parse brain.json ({}), using default", e))
            );
            CapabilityVector::default()
        })
    } else {
        CapabilityVector::default()
    };

    let mut bank = neotrix::neotrix::nt_mind::memory::ReasoningBank::new(100);
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

    log::info!("{}", info("╭─ NeoTrix Benchmark ───────────────────╮"));
    log::info!("│ Category      | Test              | Score │");
    log::info!("├───────────────┼───────────────────┼───────┤");
    for r in &report.results {
        let name_display = if r.name.len() > 17 {
            format!("{}…", &r.name[..16])
        } else {
            r.name.clone()
        };
        log::info!(
            "│ {:<13} | {:<17} | {:.2}  │",
            r.category,
            name_display,
            r.score
        );
    }
    if !report.results.is_empty() {
        log::info!("├───────────────┼───────────────────┼───────┤");
    }
    log::info!(
        "│ OVERALL       │                   │ {:.2}  │",
        report.overall_score
    );
    log::info!("╰───────────────┴───────────────────┴───────╯");
}

pub fn run_browse(url: &str) {
    use neotrix::neotrix::nt_world_browse::BrowserCircuit;
    log::info!("{}", info("╭─ NeoTrix Browser ──────────────────────────╮"));
    log::info!("│ {} {}", info("Fetching:"), url);
    log::info!(
        "{}",
        info("╰────────────────────────────────────────────────╯")
    );
    let browser = BrowserCircuit::new();
    match browser.browse(url) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            log::info!(
                "\n{} ({} lines, ~{} chars):",
                info("Content"),
                lines.len(),
                text.len()
            );
            for line in lines.iter().take(60) {
                log::info!("  {}", line);
            }
            if lines.len() > 60 {
                log::info!(
                    "  {} ({})",
                    info("..."),
                    info(format!("{} more lines", lines.len() - 60))
                );
            }
        }
        Err(e) => log::error!("{}: {}", err("Error"), e),
    }
}

pub fn run_search(query: &str, count: usize) {
    use neotrix::neotrix::nt_world_search::WebSearchEngine;

    let engine = WebSearchEngine::default();
    log::info!("{} Searching for: {}", info("🔍"), query);
    log::info!("");

    match engine.search(query, count) {
        Ok(results) => {
            if results.is_empty() {
                log::info!("{} No results found.", warn("ℹ️"));
                return;
            }
            log::info!("{}", info(format!("Found {} results:\n", results.len())));
            for (i, result) in results.iter().enumerate() {
                log::info!("{}. {}", info(format!("{}", i + 1)), result.title.bold());
                log::info!("   {}", result.url.blue().underline());
                log::info!("   {}", result.snippet);
                log::info!("");
            }
        }
        Err(e) => {
            log::error!("{} {}", err("❌ Search error:"), e);
        }
    }
}

pub fn run_login(url: &str) {
    use neotrix::neotrix::nt_world_browse::BrowserCircuit;
    log::info!("{}", info("╭─ NeoTrix Login ────────────────────────────╮"));
    log::info!("│ {}: {}", info("URL"), url);
    log::info!("│ {}", info("A Chrome window will open. Log in, then"));
    log::info!("│ {}", info("close the window to save the session."));
    log::info!(
        "{}",
        info("╰─────────────────────────────────────────────╯")
    );
    let browser = BrowserCircuit::new();
    match browser.login(url) {
        Ok(_) => log::info!("{}", success("✅ Login session saved.")),
        Err(e) => log::error!("{}", err(format!("❌ Login error: {}", e))),
    }
}

pub fn run_update(check_only: bool) {
    log::info!("{} v{}", info("NeoTrix Update"), env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "self-update")]
    {
        use self_update::cargo_crate_version;
        if check_only {
            log::info!("{}", info("Checking for updates..."));
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
                        log::info!("{} {}", info("Current version:"), env!("CARGO_PKG_VERSION"));
                        log::info!("{} {}", info("Latest version:"), release.version);
                        if release.version != cargo_crate_version!() {
                            log::info!(
                                "{}",
                                success("✅ Update available! Run `neotrix update` to install.")
                            );
                        } else {
                            log::info!("{}", success("✅ You have the latest version."));
                        }
                    }
                    Err(e) => log::error!("{}: {}", err("Check failed"), e),
                },
                Err(e) => log::error!("{}: {}", err("Update config failed"), e),
            }
        } else {
            log::info!("{}", info("Updating NeoTrix..."));
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
                        log::info!("{} {}", success("✅ Update complete:"), status.version());
                    }
                    Err(e) => log::error!("{}: {}", err("Update failed"), e),
                },
                Err(e) => log::error!("{}: {}", err("Update config failed"), e),
            }
        }
    }
    #[cfg(not(feature = "self-update"))]
    {
        let _ = check_only;
        log::info!("{}", info("Self-update is not enabled in this build."));
        log::info!(
            "{}",
            info("Build with --features self-update or use your package manager.")
        );
    }
}
