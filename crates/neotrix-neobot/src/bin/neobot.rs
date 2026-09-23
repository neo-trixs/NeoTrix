//! `neobot` — 本地独立 App CLI.
//!
//! ```sh
//! neobot init                 # 建 ~/.neobot + workspace + config.json
//! neobot doctor               # 自检 (目录/DB/引擎探活)
//! neobot run -t 标题 --text 内容   # 跑一轮本地任务 (默认 echo 引擎, 零模型)
//! neobot task list            # 列任务
//! neobot audit list           # 列审计
//! ```

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use neotrix_neobot::{
    CliEngine, EngineAdapter, EngineKind, HttpEngine, LocalEchoEngine, NeobotConfig, NeobotStore,
    NtBotError, PolicyMode, run_local_turn, run_local_turn_stream,
};

#[derive(Debug, Parser)]
#[command(name = "neobot", version, about = "NeoBot local-first agent app")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// 初始化数据目录与配置.
    Init,
    /// 自检.
    Doctor,
    /// 跑一轮本地任务.
    Run {
        /// 任务标题.
        #[arg(short, long, default_value = "local task")]
        title: String,
        /// 用户输入.
        #[arg(short = 'x', long, default_value = "hello neobot")]
        text: String,
        /// 引擎: `echo` | 本机命令名 (如 `claude`) | `http` (OpenAI 兼容, 读 NEOBOT_* env).
        #[arg(long, default_value = "echo")]
        engine: String,
        /// 流式输出增量内容 (仅 http 引擎).
        #[arg(long, default_value_t = false)]
        stream: bool,
    },
    /// 任务管理.
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// 审计管理.
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// 列出模型池 (`GET /v1/models`; neotrix serve 默认 http://127.0.0.1:3000/v1).
    Models,
}

#[derive(Debug, Subcommand)]
enum TaskCmd {
    List,
}

#[derive(Debug, Subcommand)]
enum AuditCmd {
    List,
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("neobot: {err}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), NtBotError> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init => cmd_init(),
        Cmd::Doctor => cmd_doctor(),
        Cmd::Run { title, text, engine, stream } => cmd_run(&title, &text, &engine, stream),
        Cmd::Task { cmd: TaskCmd::List } => cmd_task_list(),
        Cmd::Audit { cmd: AuditCmd::List } => cmd_audit_list(),
        Cmd::Models => cmd_models(),
    }
}

fn load_config() -> Result<NeobotConfig, NtBotError> {
    let cfg = NeobotConfig::from_env()?;
    Ok(cfg)
}

fn open_store(cfg: &NeobotConfig) -> Result<NeobotStore, NtBotError> {
    let path = cfg.db_path();
    let path_str = path.to_string_lossy().into_owned();
    NeobotStore::open(&path_str)
}

fn cmd_init() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let config_path = cfg.data_dir.join("config.json");
    if !config_path.exists() {
        let json = serde_json::to_string_pretty(&cfg)?;
        std::fs::write(&config_path, json)?;
    }
    let _ = open_store(&cfg)?;
    println!("neobot init ok: {}", cfg.data_dir.to_string_lossy());
    Ok(())
}

fn cmd_doctor() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let engine_info = match &cfg.engine {
        EngineKind::Echo => LocalEchoEngine.probe()?,
        EngineKind::Cli { command } => CliEngine::new(command)?.probe()?,
        EngineKind::Http { .. } => HttpEngine::from_env()?.probe()?,
    };
    let tasks = store.list_tasks(1)?;
    println!(
        "neobot doctor ok: data={} policy={} engine={} tasks={}",
        cfg.data_dir.to_string_lossy(),
        match cfg.policy_mode {
            PolicyMode::Enforce => "enforce",
            PolicyMode::DryRun => "dry_run",
        },
        engine_info,
        tasks.len()
    );
    Ok(())
}

fn cmd_run(title: &str, text: &str, engine_name: &str, stream: bool) -> Result<(), NtBotError> {
    let mut cfg = load_config()?;
    let trimmed = engine_name.trim();
    if trimmed == "http" {
        let (http, _) = neotrix_neobot::HttpEngineConfig::from_env()?;
        cfg.engine = EngineKind::Http {
            base_url: http.base_url,
            model: http.model,
        };
    } else if !trimmed.is_empty() && trimmed != "echo" {
        cfg.engine = EngineKind::Cli {
            command: trimmed.to_owned(),
        };
    }
    let store = open_store(&cfg)?;
    let mut emit = |delta: &str| {
        print!("{delta}");
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
    };
    let status = match &cfg.engine {
        EngineKind::Echo => {
            if stream {
                run_local_turn_stream(&store, &cfg, &LocalEchoEngine, title, text, &mut emit)?
            } else {
                run_local_turn(&store, &cfg, &LocalEchoEngine, title, text)?
            }
        }
        EngineKind::Cli { command } => {
            let engine = CliEngine::new(command)?;
            if stream {
                run_local_turn_stream(&store, &cfg, &engine, title, text, &mut emit)?
            } else {
                run_local_turn(&store, &cfg, &engine, title, text)?
            }
        }
        EngineKind::Http { .. } => {
            let engine = HttpEngine::from_env()?;
            if stream {
                println!("--- stream ---");
                let status =
                    run_local_turn_stream(&store, &cfg, &engine, title, text, &mut emit)?;
                println!("\n--- end ---");
                status
            } else {
                run_local_turn(&store, &cfg, &engine, title, text)?
            }
        }
    };
    println!("neobot run ok: status={}", status.as_str());
    Ok(())
}

fn cmd_task_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for task in store.list_tasks(20)? {
        println!("{} [{}] {}", task.id, task.status.as_str(), task.title);
    }
    Ok(())
}

fn cmd_audit_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for event in store.list_audit(20)? {
        let rule = event.rule.as_deref().unwrap_or("-");
        println!(
            "{} {} {} rule={}",
            event.at,
            event.tool,
            event.decision.as_str(),
            rule
        );
    }
    Ok(())
}

fn cmd_models() -> Result<(), NtBotError> {
    let engine = HttpEngine::for_listing()?;
    let mut models = engine.list_models()?;
    models.sort();
    for (id, owner) in models {
        println!("{id}  (owner={owner})");
    }
    Ok(())
}
