mod config;
mod entry;

use clap::{Parser, Subcommand, CommandFactory};
use entry::*;

#[derive(Parser, Debug)]
#[command(name = "neotrix", version, about = "NeoTrix — Self-evolving reasoning engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true, long, value_name = "COLOR", help = "Color mode: auto|always|never")]
    color: Option<String>,

    #[arg(global = true, long, short = 's', help = "Run HTTP server mode (legacy flag)")]
    serve: bool,

    #[arg(global = true, long, help = "Run headless mode (legacy flag)")]
    headless: bool,

    #[arg(global = true, long, help = "Run standalone mode (no LLM)")]
    standalone: bool,

    #[arg(global = true, long, value_name = "ADDR", default_value_t = String::from("0.0.0.0:3000"), help = "Server address")]
    addr: String,

    #[arg(global = true, long, value_name = "STAGE", default_value_t = 18, help = "Reasoning stage count")]
    stage: usize,

    #[arg(global = true, long, default_value_t = String::from("default"), help = "Profile name for isolated state")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Non-interactive execution with structured output")]
    Exec {
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, help = "JSONL streaming output (one JSON object per line)")]
        json: bool,
        #[arg(long, value_name = "SCHEMA", help = "Output schema for structured validation (reserved)")]
        output_schema: Option<String>,
        #[arg(long, help = "Execution timeout in seconds", default_value_t = 60)]
        timeout: u64,
        #[arg(long, value_name = "DOLLARS", help = "Hard limit on total API spend in USD")]
        max_budget_usd: Option<f64>,
    },
    #[command(about = "Run interactive mode (TUI) or one-shot prompt")]
    Run {
        #[arg(long)]
        headless: bool,
        #[arg(help = "One-shot prompt")]
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, value_name = "FORMAT", help = "Output format: text|json")]
        format: Option<String>,
        #[arg(long, help = "Start in Suggest mode (approve all)")]
        suggest: bool,
        #[arg(long, help = "Start in AutoEdit mode (auto-approve file writes)")]
        auto_edit: bool,
        #[arg(long, help = "Start in FullAuto mode (no approvals, like Codex --yolo)")]
        full_auto: bool,
        #[arg(long, help = "Alias for --full-auto")]
        yolo: bool,
        #[arg(long, value_name = "DOLLARS", help = "Hard limit on total API spend in USD")]
        max_budget_usd: Option<f64>,
        #[arg(long, value_name = "MODE", default_value = "disabled", help = "Sandbox mode: disabled|read-only")]
        sandbox: String,
        #[arg(long, help = "Disposable session — do not save to disk")]
        ephemeral: bool,
    },
    #[command(about = "Start HTTP API server")]
    Serve { #[arg(long, default_value_t = String::from("0.0.0.0:3000"))] addr: String },
    #[command(about = "One-shot reasoning (non-interactive)")]
    Reason {
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, value_name = "FORMAT", help = "Output format: text|json")]
        format: Option<String>,
    },
    #[command(name = "bench", about = "Run benchmarks")]
    Bench { category: Option<String> },
    #[command(about = "Show brain/daemon status")]
    Status,
    #[command(about = "Start background daemon")]
    Daemon { #[arg(long)] evolve: bool },
    #[command(about = "Self-update the binary")]
    Update { #[arg(long)] check_only: bool },
    #[command(about = "Generate shell completions")]
    Completions { shell: String },
    #[command(about = "Browse a URL")]
    Browse { url: String },
    #[command(about = "Browser login")]
    Login { url: String },
    #[command(about = "Proxy daemon control (status|mode|start|stop|install)")]
    Proxy { args: Vec<String> },
    #[command(name = "mcp-server", about = "Run as MCP server (stdio JSON-RPC 2.0)")]
    McpServer,
    #[command(about = "Cloud/Docker sandbox commands")]
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },
    #[command(about = "Search the web")]
    Search {
        query: String,
        #[arg(long, short = 'n', default_value_t = 5, help = "Number of results")]
        count: usize,
    },
    #[command(about = "Scan network for NeoTrix agents via UDP discovery")]
    Discover {
        #[arg(long, short = 'p', default_value_t = 42069, help = "UDP port")]
        port: u16,
        #[arg(long, short = 'd', default_value_t = 3000, help = "Scan duration in ms")]
        duration: u64,
        #[arg(long, help = "JSON output")]
        json: bool,
    },
    #[command(about = "Manage runtime feature flags")]
    Features {
        #[command(subcommand)]
        command: FeaturesCommands,
    },
    #[command(about = "Wallet management (create, import, list, balance)")]
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },
}

#[derive(Subcommand, Debug)]
enum SandboxCommands {
    #[command(about = "Execute code in sandbox")]
    Run {
        #[arg(help = "Code to execute (reads from stdin if omitted)")]
        code: Option<String>,
        #[arg(long, short = 'r', default_value = "python3", help = "Runtime (python3, node18, rust, go1_21, linux)")]
        runtime: String,
        #[arg(long, short = 't', default_value_t = 300, help = "Max runtime in seconds")]
        timeout: u64,
    },
    #[command(about = "List active sandbox sessions")]
    List,
    #[command(about = "Cancel a sandbox session")]
    Cancel {
        #[arg(help = "Session ID")]
        session_id: String,
    },
    #[command(about = "Upload file to sandbox session")]
    Upload {
        #[arg(help = "Local file path")]
        path: String,
        #[arg(help = "Session ID (creates new if omitted)", default_value = "")]
        session_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum FeaturesCommands {
    #[command(about = "Enable a runtime feature flag")]
    Enable {
        #[arg(help = "Feature name to enable")]
        name: String,
    },
    #[command(about = "List all available feature flags and their status")]
    List,
}

#[derive(Subcommand, Debug)]
enum WalletCommands {
    #[command(about = "Create a new wallet")]
    Create {
        #[arg(help = "Wallet label")]
        label: String,
    },
    #[command(about = "Import wallet from private key")]
    Import {
        #[arg(help = "Wallet label")]
        label: String,
        #[arg(help = "Private key (hex with or without 0x)")]
        private_key: String,
    },
    #[command(about = "List all wallets")]
    List {
        #[arg(long, help = "JSON output")]
        json: bool,
    },
    #[command(about = "Check wallet balance")]
    Balance {
        #[arg(help = "Chain name (eth, bsc, polygon, etc.)", default_value = "eth")]
        chain: String,
    },
    #[command(about = "Delete a wallet")]
    Delete {
        #[arg(help = "Wallet label to delete")]
        label: String,
    },
    #[command(about = "Export private key (⚠️  security sensitive)")]
    Export {
        #[arg(help = "Wallet label")]
        label: String,
    },
}

fn main() {
    let _sentry_guard = neotrix::neotrix::sentry::init_sentry();
    let cli = Cli::parse();

    // First-run provider config wizard
    if !entry::check_provider_config() {
        entry::run_provider_wizard();
    }

    let cfg = config::NeoTrixConfig::load();

    let color_mode = cli.color.as_deref().or(cfg.color_mode.as_deref()).unwrap_or("auto");
    if color_mode == "never" {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(true);
    }

    if let Some(level) = &cfg.log_level {
        let _ = std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }

    match &cli.command {
        Some(Commands::Exec { prompt, file, pipe, json, output_schema: _, timeout, max_budget_usd }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::cli::cost_tracker::COST_TRACKER.lock().expect("COST_TRACKER lock").set_max_budget_usd(*limit);
            }
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            if resolved.is_empty() {
                eprintln!("Error: no prompt provided. Usage: neotrix exec <prompt>");
                std::process::exit(1);
            }
            run_exec(&resolved, *json, *timeout);
        }
        Some(Commands::Run { headless, prompt, file, pipe, format, suggest: _, auto_edit, full_auto, yolo, sandbox, max_budget_usd, ephemeral }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::cli::cost_tracker::COST_TRACKER.lock().expect("COST_TRACKER lock").set_max_budget_usd(*limit);
            }
            let approval_mode = if *yolo || *full_auto {
                neotrix::cli::approval::ApprovalMode::FullAuto
            } else if *auto_edit {
                neotrix::cli::approval::ApprovalMode::AutoEdit
            } else {
                neotrix::cli::approval::ApprovalMode::Suggest
            };
            neotrix::cli::approval::global_approval().lock().expect("global_approval lock").set_mode(approval_mode);
            neotrix::cli::sandbox::init_sandbox(neotrix::cli::sandbox::SandboxMode::from_str(sandbox.as_str()));
            if let Some(p) = prompt {
                let resolved = resolve_prompt(Some(p), file.as_deref(), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile);
            } else if let Some(f) = file {
                let resolved = resolve_prompt(None, Some(f), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile);
            } else if *pipe {
                let resolved = resolve_prompt(None, None, true);
                run_one_shot(&resolved, format.as_deref(), &cli.profile);
            } else if *headless {
                run_headless_mode(&cfg, &cli.profile);
            } else {
                run_interactive_with_ephemeral(&cfg, &cli.profile, *ephemeral);
            }
        }
        Some(Commands::Serve { addr }) => run_server_mode(addr, &cli.profile),
        Some(Commands::Reason { prompt, file, pipe, format }) => {
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            run_one_shot(&resolved, format.as_deref(), &cli.profile);
        }
        Some(Commands::Bench { category }) => run_benchmark(category.as_deref()),
        Some(Commands::Status) => show_status(),
        Some(Commands::Daemon { evolve }) => {
            if *evolve { run_daemon_evolution(&cli.profile); } else { run_daemon(&cli.profile); }
        }
        Some(Commands::Update { check_only }) => run_update(*check_only),
        Some(Commands::Completions { shell }) => generate_completions(shell, &mut Cli::command()),
        Some(Commands::Browse { url }) => run_browse(url),
        Some(Commands::Login { url }) => run_login(url),
        Some(Commands::Proxy { args }) => {
            let cmd_str = args.join(" ");
            let rt = tokio::runtime::Runtime::new().expect("tokio");
            rt.block_on(entry::run_proxy_cmd(&cmd_str));
        }
        Some(Commands::Sandbox { command }) => {
            match command {
                SandboxCommands::Run { code, runtime, timeout } => {
                    entry::run_sandbox_run(code.as_deref(), runtime, *timeout);
                }
                SandboxCommands::List => {
                    entry::run_sandbox_list();
                }
                SandboxCommands::Cancel { session_id } => {
                    entry::run_sandbox_cancel(session_id);
                }
                SandboxCommands::Upload { path, session_id } => {
                    entry::run_sandbox_upload(path, session_id);
                }
            }
        }
        Some(Commands::Search { query, count }) => {
            run_search(query, *count);
        }
        Some(Commands::Discover { port, duration, json }) => {
            run_discover(*port, *duration, *json);
        }
        Some(Commands::McpServer) => entry::run_mcp_server(),
        Some(Commands::Features { command }) => {
            match command {
                FeaturesCommands::Enable { name } => {
                    entry::run_features_enable(name);
                }
                FeaturesCommands::List => {
                    entry::run_features_list();
                }
            }
        }
        Some(Commands::Wallet { command }) => {
            match command {
                WalletCommands::Create { label } => {
                    entry::run_wallet_create(label);
                }
                WalletCommands::Import { label, private_key } => {
                    entry::run_wallet_import(label, private_key);
                }
                WalletCommands::List { json } => {
                    entry::run_wallet_list(*json);
                }
                WalletCommands::Balance { chain } => {
                    entry::run_wallet_balance(chain);
                }
                WalletCommands::Delete { label } => {
                    entry::run_wallet_delete(label);
                }
                WalletCommands::Export { label } => {
                    entry::run_wallet_export(label);
                }
            }
        }
        None => {
            if cli.standalone { run_standalone_mode(cli.stage); }
            else if cli.serve { run_server_mode(&cli.addr, &cli.profile); }
            else if cli.headless { run_headless_mode(&cfg, &cli.profile); }
            else { run_interactive(&cfg, &cli.profile); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let instance = ::new();
        assert!(true);
    }
}
