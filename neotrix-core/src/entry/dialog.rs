//! 对话面（合二为一：neobot 即 neotrix 对外对话的一部分）。
//!
//! `neotrix dialog <run|agent|models|provider|core|convo|task>` 与
//! `neobot` 二进制同律（同 lib 入口 `load_config/open_store` + 同聚合 `pool_models`），
//! 薄封装，一字不差。桌面 App 与 CLI 走同一套。

use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum DialogCmd {
    /// 跑一轮对话（配对在线先走晶体，选中模型透传；env 本地直连舱除外）。
    Say {
        /// 说的话.
        text: String,
        /// 走注册端点（覆盖默认路由；key 读其 key_env）。
        #[arg(long)]
        provider: Option<String>,
        /// 覆盖模型名（透传晶体池或端点）。
        #[arg(long)]
        model: Option<String>,
        /// 归属会话 id（缺省自动建）。
        #[arg(long)]
        convo: Option<String>,
    },
    /// 服务端 agent 跑一轮（任务拆解→能力网分发→聚合，需配对）。
    Agent {
        /// 目标.
        goal: String,
        /// 上下文（可选透传；显式 ops JSON 即确定性工具直调）。
        #[arg(long)]
        context: Option<String>,
        /// 步数上限 1-32（缺省 8）。
        #[arg(long)]
        steps: Option<i64>,
    },
    /// 模型池（聚合启用的端点；不可达给 fallback 行）。
    Models {
        /// 只看指定端点.
        #[arg(long)]
        provider: Option<String>,
    },
    /// 端点管理：list（脱敏）.
    Provider {
        /// 子命令：list（缺省）.
        #[arg(default_value = "list")]
        action: String,
    },
    /// 核心配对：status|pair|unpair.
    Core {
        /// 子命令：status（缺省）|pair|unpair.
        #[arg(default_value = "status")]
        action: String,
        /// pair 用：base_url（缺省 127.0.0.1:3000/v1）。
        #[arg(long)]
        base_url: Option<String>,
        /// pair 用：模型名.
        #[arg(long)]
        model: Option<String>,
        /// pair 用：token 环境变量名.
        #[arg(long)]
        key_env: Option<String>,
    },
    /// 会话列表.
    Convo {
        /// 子命令：list（缺省）.
        #[arg(default_value = "list")]
        action: String,
    },
    /// 任务列表.
    Task {
        /// 子命令：list（缺省）.
        #[arg(default_value = "list")]
        action: String,
    },
}

pub fn run_dialog(cmd: DialogCmd) -> Result<(), String> {
    match cmd {
        DialogCmd::Say { text, provider, model, convo } => say(&text, provider.as_deref(), model.as_deref(), convo.as_deref()),
        DialogCmd::Agent { goal, context, steps } => agent(&goal, context.as_deref(), steps),
        DialogCmd::Models { provider } => models(provider.as_deref()),
        DialogCmd::Provider { action } => provider_cmd(&action),
        DialogCmd::Core { action, base_url, model, key_env } => {
            core(&action, base_url.as_deref(), model.as_deref(), key_env.as_deref())
        }
        DialogCmd::Convo { action } => convo(&action),
        DialogCmd::Task { action } => task(&action),
    }
}

fn say(
    text: &str,
    provider_name: Option<&str>,
    model_override: Option<&str>,
    convo_id: Option<&str>,
) -> Result<(), String> {
    use neotrix_neobot::{Actor, EngineAdapter, LocalEchoEngine};
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    let memory = neotrix_neobot::nt_memory::memory_for_config(&cfg);
    // 路由律与桌面 `resolve_run_engine` 同构：env 直连舱除外，一律先晶体。
    let engine: Box<dyn EngineAdapter> = match provider_name {
        Some("env") => {
            let base = std::env::var("NEOBOT_BASE_URL").unwrap_or_else(|_| {
                neotrix_neobot::nt_http_engine::DEFAULT_BASE_URL.to_owned()
            });
            let key = std::env::var("NEOBOT_API_KEY").unwrap_or_default();
            let model = model_override.unwrap_or("").trim();
            if model.is_empty() {
                return Err("env provider needs --model".to_owned());
            }
            let http_config = neotrix_neobot::HttpEngineConfig {
                base_url: base.trim().trim_end_matches('/').to_owned(),
                model: model.to_owned(),
                timeout_secs: neotrix_neobot::nt_http_engine::DEFAULT_TIMEOUT_SECS,
            };
            let http = neotrix_neobot::HttpEngine::new(http_config, key).map_err(|e| e.to_string())?;
            Box::new(http.with_memory_context(memory))
        }
        _ => {
            let wanted = model_override.map(str::trim).filter(|m| !m.is_empty());
            if let Ok(Some(http)) = neotrix_neobot::core_engine_with_model(&store, wanted) {
                Box::new(http.with_memory_context(memory))
            } else if let Some(name) = provider_name {
                let provider = store
                    .get_provider(name)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("no such provider '{name}'"))?;
                Box::new(
                    provider
                        .http_engine(model_override)
                        .map_err(|e| e.to_string())?
                        .with_memory_context(memory),
                )
            } else {
                Box::new(LocalEchoEngine)
            }
        }
    };
    let title = text.chars().take(24).collect::<String>();
    let status = neotrix_neobot::run_local_turn_as(
        &store,
        &cfg,
        engine.as_ref(),
        Actor::Person,
        "owner",
        &title,
        text,
        convo_id,
    )
    .map_err(|e| e.to_string())?;
    println!("neotrix dialog: status={}", status.as_str());
    Ok(())
}

fn agent(goal: &str, context: Option<&str>, steps: Option<i64>) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    let result = neotrix_neobot::agent_run_with_steps(
        &store,
        goal,
        context,
        steps.unwrap_or(neotrix_neobot::AGENT_DEFAULT_STEPS),
    )
    .map_err(|e| e.to_string())?;
    println!("neotrix dialog agent: status={} model={}", result.status, result.model_used);
    println!("{}", result.output);
    for step in &result.trace {
        println!("- [{}] {}", step.kind, step.detail);
    }
    Ok(())
}

fn models(only: Option<&str>) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    // legacy env 端点（与 neobot models §1 同律）。
    if only.is_none() {
        if let Ok(engine) = neotrix_neobot::HttpEngine::for_listing() {
            if let Ok(list) = engine.list_models() {
                for (id, owner) in list {
                    println!("{id}  (owner={owner} provider=env)");
                }
            }
        }
    }
    let (mut pooled, unreachable) = neotrix_neobot::pool_models(&store);
    if let Some(name) = only {
        pooled.retain(|m| m.provider == name);
    }
    for name in &unreachable {
        if only.map(|n| n == name).unwrap_or(true) {
            eprintln!("neotrix dialog: 端点 '{name}' 不可达（已用 fallback 行）");
        }
    }
    if let Some(name) = only {
        if !pooled.iter().any(|m| m.provider == name)
            && store.get_provider(name).map_err(|e| e.to_string())?.is_none()
        {
            return Err(format!("no such provider '{name}'"));
        }
    }
    for m in pooled {
        println!("{}  (owner={} provider={})", m.id, m.owner, m.provider);
    }
    Ok(())
}

fn provider_cmd(action: &str) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    match action {
        "list" => {
            for p in store.list_providers().map_err(|e| e.to_string())? {
                // key_env 脱敏：只显变量名，泄露形态告警不打印值（与 neobot 同律）。
                let key_show = if p.key_env.is_empty() {
                    "-".to_owned()
                } else if neotrix_neobot::Provider::looks_like_secret(&p.key_env) {
                    "!!PLAINTEXT-LEAK!!(export 变量后重填)".to_owned()
                } else {
                    p.key_env.clone()
                };
                println!(
                    "{} {} model={} key_env={} {}",
                    if p.enabled { "[on]" } else { "[off]" },
                    p.name,
                    if p.model.is_empty() { "-" } else { &p.model },
                    key_show,
                    p.base_url,
                );
            }
            Ok(())
        }
        other => Err(format!("unknown provider action '{other}' (list)")),
    }
}

fn core(
    action: &str,
    base_url: Option<&str>,
    model: Option<&str>,
    key_env: Option<&str>,
) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    match action {
        "status" => match neotrix_neobot::core_status(&store).map_err(|e| e.to_string())? {
            neotrix_neobot::CoreStatus::Unpaired => {
                println!("neotrix dialog core: unpaired (local-only)");
                Ok(())
            }
            neotrix_neobot::CoreStatus::Offline { pair, reason } => {
                println!(
                    "neotrix dialog core: paired but offline (model={} at {}, reason={})",
                    pair.model, pair.base_url, reason
                );
                Ok(())
            }
            neotrix_neobot::CoreStatus::Online { pair, models, latency_ms, crystal_version, tool_count } => {
                println!(
                    "neotrix dialog core: soul online ({} models, model={} at {}, {}ms, crystal_version={} tools={})",
                    models, pair.model, pair.base_url, latency_ms, crystal_version, tool_count
                );
                Ok(())
            }
        },
        "pair" => {
            let base = base_url.unwrap_or("http://127.0.0.1:3000/v1");
            let (pair, count) = neotrix_neobot::pair_core(
                &store,
                base,
                model.unwrap_or("neotrix-crystal"),
                key_env.unwrap_or("CRYSTAL_TOKEN"),
            )
            .map_err(|e| e.to_string())?;
            println!(
                "neotrix dialog core paired: soul embedded ({} models at {}, model={})",
                count, pair.base_url, pair.model
            );
            Ok(())
        }
        "unpair" => {
            let removed = neotrix_neobot::unpair_core(&store).map_err(|e| e.to_string())?;
            println!(
                "neotrix dialog core unpaired: {}",
                if removed { "soul removed, back to local-only" } else { "already local-only" }
            );
            Ok(())
        }
        other => Err(format!("unknown core action '{other}' (status|pair|unpair)")),
    }
}

fn convo(action: &str) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    match action {
        "list" => {
            for c in store.list_conversations().map_err(|e| e.to_string())? {
                println!("{} [{}] {} members={}", c.id, c.kind, c.title, c.members.join(","));
            }
            Ok(())
        }
        other => Err(format!("unknown convo action '{other}' (list)")),
    }
}

fn task(action: &str) -> Result<(), String> {
    let cfg = neotrix_neobot::load_config().map_err(|e| e.to_string())?;
    let store = neotrix_neobot::open_store(&cfg).map_err(|e| e.to_string())?;
    match action {
        "list" => {
            for t in store.list_tasks(50).map_err(|e| e.to_string())? {
                println!("{} [{}] {}", t.id, t.status.as_str(), t.title);
            }
            Ok(())
        }
        other => Err(format!("unknown task action '{other}' (list)")),
    }
}
