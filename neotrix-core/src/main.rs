mod entry;

use clap::{Parser, Subcommand, CommandFactory};
use entry::*;
// config 模块由 lib 提供 (neotrix::config), 避免与 lib.rs 重复定义。

#[derive(Parser, Debug)]
#[command(
    name = "neotrix",
    version,
    about = "NeoTrix — Self-evolving reasoning engine",
    after_help = "\
EXAMPLES:
  neotrix run \"explain this codebase\"          Interactive / one-shot reasoning
  neotrix exec --json \"summarize the diff\"    Structured non-interactive execution
  neotrix reason -f prompt.txt                 Reason from a file
  neotrix status                               Show brain/daemon status
  neotrix completions bash > /etc/bash_completion.d/neotrix
  neotrix search \"rust async runtime\" -n 10   Web search
  neotrix discover --json                      Scan for NeoTrix agents on LAN
  neotrix features list                        List runtime feature flags
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true, long, value_name = "COLOR", help = "Color mode: auto|always|never")]
    color: Option<String>,

    #[arg(global = true, long, help = "Suppress non-error log output")]
    quiet: bool,

    #[arg(global = true, long, short = 's', help = "Run HTTP server mode (legacy flag)")]
    serve: bool,

    #[arg(global = true, long, help = "Run headless mode (legacy flag)")]
    headless: bool,

    #[arg(global = true, long, help = "Run standalone mode (no LLM)")]
    standalone: bool,

    #[arg(global = true, long, help = "Run Agent Loop mode (NeoTrix as subject, LLM as backend)")]
    agent: bool,

    #[arg(global = true, long, value_name = "ADDR", default_value_t = String::from("0.0.0.0:3000"), help = "Server address")]
    addr: String,

    #[arg(global = true, long, value_name = "STAGE", default_value_t = 18, help = "Reasoning stage count")]
    stage: usize,

    #[arg(global = true, long, default_value_t = String::from("default"), help = "Profile name for isolated state")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ── Core: LLM 交互 ──
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
        #[arg(long, short = 'S', help = "Stream output in real-time (text mode only)")]
        stream: bool,
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
        #[arg(long, short = 'S', help = "Stream output in real-time (text mode only)")]
        stream: bool,
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
        #[arg(long, short = 'S', help = "Stream output in real-time")]
        stream: bool,
    },
    #[command(name = "mcp-server", about = "Run as MCP server (stdio JSON-RPC 2.0)")]
    McpServer,

    // ── Knowledge & Memory ──
    #[command(name = "evidence", about = "EWHR evidence management (list|get|calibrate|export|stats)")]
    Evidence {
        #[command(subcommand)]
        command: neotrix::cli::commands::evidence_cmds::EvidenceCommand,
    },
    #[command(name = "wiki", about = "Wiki KB: generate|status|sync|graph|query")]
    Wiki {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "todo", about = "TODO smart-sync (sync_todos.py replacement): sync|status|allocate [max]|import <path>")]
    Todo {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Search the web")]
    Search {
        query: String,
        #[arg(long, short = 'n', default_value_t = 5, help = "Number of results")]
        count: usize,
    },

    // ── System & Ops ──
    #[command(about = "Run benchmarks")]
    Bench { category: Option<String> },
    #[command(about = "Show brain/daemon status")]
    Status,
    #[command(about = "Start background daemon")]
    Daemon { #[arg(long)] evolve: bool },
    #[command(about = "Self-update the binary")]
    Update { #[arg(long)] check_only: bool },
    #[command(about = "Generate shell completions")]
    Completions { shell: String },
    #[command(about = "Manage runtime feature flags")]
    Features {
        #[command(subcommand)]
        command: FeaturesCommands,
    },
    #[command(about = "Manage config file (encrypt/decrypt API keys)")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    #[command(about = "NeoTrix 系统运维 (统一安装/守护/卸载, 替代分散 sh 脚本): daemons|uninstall|status")]
    Sysops {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    // ── Network & Agents ──
    #[command(about = "Browse a URL")]
    Browse { url: String },
    #[command(about = "Browser login")]
    Login { url: String },
    #[command(about = "Proxy daemon control (status|mode|start|stop|install)")]
    Proxy { args: Vec<String> },
    #[command(about = "Scan network for NeoTrix agents via UDP discovery")]
    Discover {
        #[arg(long, short = 'p', default_value_t = 42069, help = "UDP port")]
        port: u16,
        #[arg(long, short = 'd', default_value_t = 3000, help = "Scan duration in ms")]
        duration: u64,
        #[arg(long, help = "JSON output")]
        json: bool,
    },
    #[command(about = "Cloud/Docker sandbox commands")]
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },

    // ── Finance & Wallet ──
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
enum ConfigCommands {
    #[command(about = "Encrypt all plaintext API keys in the config file")]
    EncryptKeys,
    #[command(about = "Decrypt all encrypted API keys in the config file (use with caution)")]
    DecryptKeys,
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
    // 智能命令整合: clap 解析失败时, 未知子命令回退到交互式命令注册表
    // (60+ 命令: /kb /goal /wiki /evidence ...), 使它们可直接从命令行调用。
    // 注意: try_parse 必须在 init_tracing 之前, 这样回退路径设置的
    // RUST_LOG 才能在 tracing subscriber 初始化时生效 (抑制 KB 等 INFO 日志)。
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::InvalidSubcommand {
                if let Some(unknown) = extract_unknown_subcommand(&e) {
                    let reg = neotrix::cli::commands::registry::default_registry();
                    let lookup = if unknown.starts_with('/') {
                        unknown.clone()
                    } else {
                        format!("/{}", unknown)
                    };
                    if reg.find(&lookup).is_some() {
                        // 对标主流 CLI: 回退命令输出保持干净, 抑制 INFO/WARN 日志
                        std::env::set_var("RUST_LOG", "neotrix=error");
                        let raw: Vec<String> = std::env::args().skip(1).collect();
                        let input = if unknown.starts_with('/') {
                            raw.join(" ")
                        } else {
                            format!("/{}", raw.join(" "))
                        };
                        let out = reg.execute(&input, None);
                        if !out.message.is_empty() {
                            println!("{}", out.message);
                        }
                        std::process::exit(if out.success { 0 } else { 1 });
                    }
                }
            }
            e.exit()
        }
    };

    neotrix::neotrix::nt_io_logging::init_tracing();
    let _sentry_guard = neotrix::neotrix::nt_shield_sentry::init_sentry();

    // --quiet: 必须在 NeoTrixConfig::load() 之前设置环境变量,
    // 否则 load() 的 "[config] loaded" 诊断已在 quiet 之前输出。
    if cli.quiet {
        std::env::set_var("RUST_LOG", "neotrix=error");
        std::env::set_var("NEOTRIX_QUIET", "1");
    }

    // First-run provider config wizard (skip for pure ops commands)
    // 对标主流 CLI: 纯本地命令 (help/status/completions/features/config/wallet/
    // evidence/wiki/todo/sysops/bench/discover/proxy/sandbox/update/browse/login)
    // 不依赖 LLM provider, 不应被交互式 wizard 阻塞。
    let is_ops_cmd = matches!(
        cli.command,
        Some(Commands::Sysops { .. })
            | Some(Commands::Status)
            | Some(Commands::Completions { .. })
            | Some(Commands::Features { .. })
            | Some(Commands::Config { .. })
            | Some(Commands::Wallet { .. })
            | Some(Commands::Evidence { .. })
            | Some(Commands::Wiki { .. })
            | Some(Commands::Todo { .. })
            | Some(Commands::Bench { .. })
            | Some(Commands::Discover { .. })
            | Some(Commands::Proxy { .. })
            | Some(Commands::Sandbox { .. })
            | Some(Commands::Update { .. })
            | Some(Commands::Browse { .. })
            | Some(Commands::Login { .. })
    ) || cli.agent
        || cli.standalone
        || cli.headless;
    if !is_ops_cmd && !entry::check_provider_config() {
        entry::run_provider_wizard();
    }

    let cfg = neotrix::config::NeoTrixConfig::load();

    let color_mode = cli.color.as_deref().or(cfg.color_mode.as_deref()).unwrap_or("auto");
    if color_mode == "never" {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(true);
    }

    if let Some(level) = &cfg.log_level {
        std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }
    // (--quiet 已在 parse 后提前设置, 此处不再重复)

    match &cli.command {
        Some(Commands::Exec { prompt, file, pipe, json, output_schema: _, timeout, max_budget_usd, stream }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::cli::cost_tracker::COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner()).set_max_budget_usd(*limit);
            }
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            if resolved.is_empty() {
                // 用法错误 → 退出码 2 (对标 clap 约定: 0=成功 / 1=运行时错误 / 2=用法错误)
                eprintln!("error: no prompt provided. Usage: neotrix exec <prompt>");
                std::process::exit(2);
            }
            run_exec(&resolved, *json, *stream, *timeout);
        }
        Some(Commands::Run { headless, prompt, file, pipe, format, suggest: _, auto_edit, full_auto, yolo, sandbox, max_budget_usd, ephemeral, stream }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::cli::cost_tracker::COST_TRACKER.lock().unwrap_or_else(|e| e.into_inner()).set_max_budget_usd(*limit);
            }
            let approval_mode = if *yolo || *full_auto {
                neotrix::cli::approval::ApprovalMode::FullAuto
            } else if *auto_edit {
                neotrix::cli::approval::ApprovalMode::AutoEdit
            } else {
                neotrix::cli::approval::ApprovalMode::Suggest
            };
            neotrix::cli::approval::global_approval().lock().unwrap_or_else(|e| e.into_inner()).set_mode(approval_mode);
            neotrix::cli::sandbox::init_sandbox(neotrix::cli::sandbox::SandboxMode::from_str(sandbox.as_str()));
            if let Some(p) = prompt {
                let resolved = resolve_prompt(Some(p), file.as_deref(), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if let Some(f) = file {
                let resolved = resolve_prompt(None, Some(f), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if *pipe {
                let resolved = resolve_prompt(None, None, true);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if *headless {
                run_headless_mode(&cfg, &cli.profile);
            } else {
                run_interactive_with_ephemeral(&cfg, &cli.profile, *ephemeral);
            }
        }
        Some(Commands::Serve { addr }) => run_background_daemon(addr, &cli.profile),
        Some(Commands::Reason { prompt, file, pipe, format, stream }) => {
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
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
        Some(Commands::Config { command }) => {
            match command {
                ConfigCommands::EncryptKeys => {
                    entry::run_config_encrypt_keys();
                }
                ConfigCommands::DecryptKeys => {
                    entry::run_config_decrypt_keys();
                }
            }
        }
        Some(Commands::Evidence { command }) => {
            if let Err(e) = neotrix::cli::commands::evidence_cmds::handle_evidence_command(command) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Wiki { args }) => {
            use neotrix::cli::commands::types::CliCommand;
            let cmd = neotrix::cli::commands::wiki_cmds::WikiCmd;
            let out = cmd.execute(&args, None);
            if out.success {
                println!("{}", out.message);
            } else {
                eprintln!("error: {}", out.message);
                std::process::exit(1);
            }
        }
        Some(Commands::Todo { args }) => {
            use neotrix::cli::commands::types::CliCommand;
            let cmd = neotrix::cli::commands::kanban_cmds::BoardCmd;
            let out = cmd.execute(&args, None);
            if out.success {
                println!("{}", out.message);
            } else {
                eprintln!("error: {}", out.message);
                std::process::exit(1);
            }
        }
        Some(Commands::Sysops { args }) => {
            entry::run_sysops(args);
        }
        None => {
            if cli.standalone { run_standalone_mode(cli.stage); }
            else if cli.agent { entry::run_agent_tui(&cli.profile); }
            else if cli.serve { run_background_daemon(&cli.addr, &cli.profile); }
            else if cli.headless { run_headless_mode(&cfg, &cli.profile); }
            else { run_interactive(&cfg, &cli.profile); }
        }
    }
}

/// 从 clap 错误中提取未知子命令名。
/// clap 格式: `error: unrecognized subcommand 'xxx'`
fn extract_unknown_subcommand(e: &clap::Error) -> Option<String> {
    let msg = e.to_string();
    let start = msg.find('\'')? + 1;
    let rest = &msg[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_unknown_subcommand() {
        let err = clap::Error::raw(
            clap::error::ErrorKind::InvalidSubcommand,
            "error: unrecognized subcommand 'kbl'\n\nUsage: neotrix <COMMAND>\n",
        );
        assert_eq!(extract_unknown_subcommand(&err).as_deref(), Some("kbl"));
    }
}
