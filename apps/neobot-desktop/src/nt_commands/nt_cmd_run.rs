//! `nt_cmd_run` — 跑轮（阻塞/流式）+ 引擎解析 + 回复标签.
//!
//! 每轮返回 `NeobotRunResult { status, labels }`（前端气泡顶部 chips 直消）：
//! `model` 来自分辨出的引擎模型名，`mode` 为 direct/passthrough/fallback 三态，
//! `tools` 取本轮 task 的 `steps` 工具列（`nt_reply_tag` 口径过滤），
//! `usage` 取账本前后差值 + `nt_cost` 同律计价（未知不猜）。

use super::{EngineSelection, NeobotRunResult, load_config, open_store, resolve_engine};
use neotrix_neobot::{
    Actor, EngineAdapter, HttpEngine, LocalEchoEngine, NeobotConfig, NeobotStore, OpencodeEngine,
    ReplyMode, TurnLabels, labels_for_turn, run_local_turn_as, run_local_turn_stream_as,
};
/// spawn_blocking → run_local_turn(_stream)_as(Person)

#[tauri::command]
pub async fn neobot_run(
    title: String,
    text: String,
    actor_name: String,
    convo_id: Option<String>,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<NeobotRunResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = load_config()?;
        let store = open_store(&config)?;
        let me = actor_name.trim();
        let me = if me.is_empty() { "owner" } else { me };
        let convo = convo_id.as_deref();
        let (engine, mode, model) =
            resolve_run_engine(&store, &config, model_provider, model_name)?;
        let before = ledger_totals(&store);
        let status = run_local_turn_as(
            &store, &config, engine.as_ref(), Actor::Person, me, &title, &text, convo,
        )
        .map_err(|err| err.to_string())?;
        let labels = collect_labels(&store, &model, mode, before);
        Ok(NeobotRunResult {
            status: status.as_str().to_owned(),
            labels,
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

fn resolve_run_engine(
    store: &NeobotStore,
    config: &NeobotConfig,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<(Box<dyn EngineAdapter>, ReplyMode, String), String> {
    // 模型走内部（架构律）：配对且在线时，一律先走晶体核心（任务拆解→能力网分发→聚合），
    // 模型名透传由晶体池内解析。例外：`env` 本地直连舱（Ollama/LM Studio，无晶体等价）；
    // 晶体离线/未配对 → 回落旧链（显式端点直连 → CLI 配对 → 配置引擎），不断档。
    // 路由三态：核心路径=passthrough，显式/配置直连=direct，本地回显=fallback。
    let explicit_env = matches!(&model_provider, Some(p) if p == "env");
    if !explicit_env {
        let memory = neotrix_neobot::nt_memory::memory_for_config(config);
        // 选中模型名透传（池子模型由晶体池内解析；空即配对模型）。
        let wanted = model_name.as_deref().filter(|m| !m.trim().is_empty());
        if let Ok(Some(http)) = neotrix_neobot::core_engine_with_model(store, wanted) {
            let name = http.model_name().trim();
            let model = if name.is_empty() {
                wanted
                    .unwrap_or(neotrix_neobot::nt_core::CORE_MODEL)
                    .trim()
                    .to_owned()
            } else {
                name.to_owned()
            };
            return Ok((
                Box::new(http.with_memory_context(memory)),
                ReplyMode::Passthrough,
                model,
            ));
        }
    }
    if let Some((engine, model)) = resolve_model_engine(store, config, model_provider, model_name)? {
        return Ok((engine, ReplyMode::Direct, model));
    }
    let memory = neotrix_neobot::nt_memory::memory_for_config(config);
    // 灵魂嵌入：无显式指定且核心活着 → 走晶体核心（桌面/CLI 同律）。
    if let Ok(Some(http)) = neotrix_neobot::core_engine(store) {
        let name = http.model_name().trim();
        let model = if name.is_empty() {
            neotrix_neobot::nt_core::CORE_MODEL.to_owned()
        } else {
            name.to_owned()
        };
        return Ok((
            Box::new(http.with_memory_context(memory)),
            ReplyMode::Passthrough,
            model,
        ));
    }
    // CLI 通道配对（Zen）：配置引擎之外，配对行即路由（opencode 坏了就地回落）。
    // 本地 CLI 直驱，口径记 direct（非晶体服务端透传）。
    if let Ok(Some(pair)) = store.get_core_pair() {
        if pair.via == "cli" {
            if let Ok(engine) = OpencodeEngine::new(&pair.model) {
                return Ok((Box::new(engine), ReplyMode::Direct, pair.model));
            }
        }
    }
    match resolve_engine(config)? {
        EngineSelection::Echo => Ok((
            Box::new(LocalEchoEngine),
            ReplyMode::Fallback,
            "echo".to_owned(),
        )),
        EngineSelection::Cli(engine) => {
            let model = engine.engine_id().to_owned();
            Ok((Box::new(engine), ReplyMode::Direct, model))
        }
        EngineSelection::Opencode(engine) => {
            let model = engine.model().to_owned();
            Ok((Box::new(engine), ReplyMode::Direct, model))
        }
        EngineSelection::Http(engine) => {
            let model = engine.model_name().to_owned();
            Ok((
                Box::new(engine.with_memory_context(memory)),
                ReplyMode::Direct,
                model,
            ))
        }
    }
}
/// spawn_blocking → run_local_turn(_stream)_as(Person)

#[tauri::command]
pub async fn neobot_run_stream(
    title: String,
    text: String,
    actor_name: String,
    convo_id: Option<String>,
    model_provider: Option<String>,
    model_name: Option<String>,
    on_event: tauri::ipc::Channel<StreamEvent>,
) -> Result<NeobotRunResult, String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamEvent>(128);
    let run = tokio::task::spawn_blocking(move || -> Result<NeobotRunResult, String> {
        let config = load_config()?;
        let store = open_store(&config)?;
        let me = actor_name.trim().to_owned();
        let me = if me.is_empty() { "owner".to_owned() } else { me };
        let convo = convo_id.as_deref();
        let (engine, mode, model) =
            resolve_run_engine(&store, &config, model_provider, model_name)?;
        let before = ledger_totals(&store);
        let mut emit_delta = |delta: &str| {
            let _ = tx.blocking_send(StreamEvent::Delta { text: delta.to_owned() });
        };
        let mut emit_step = |tool: &str, ok: bool, output: &str| {
            let _ = tx.blocking_send(StreamEvent::Step {
                tool: tool.to_owned(),
                ok,
                output: output.to_owned(),
            });
        };
        let status = run_local_turn_stream_as(
            &store,
            &config,
            engine.as_ref(),
            Actor::Person,
            &me,
            &title,
            &text,
            convo,
            &mut emit_delta,
            Some(&mut emit_step),
        )
        .map_err(|err| err.to_string())?;
        let _ = tx.blocking_send(StreamEvent::Done { status: status.as_str().to_owned() });
        let labels = collect_labels(&store, &model, mode, before);
        Ok(NeobotRunResult {
            status: status.as_str().to_owned(),
            labels,
        })
    });
    while let Some(event) = rx.recv().await {
        on_event.send(event).map_err(|err| err.to_string())?;
    }
    run.await.map_err(|err| err.to_string())?
}

/// 流式事件（delta 增量 / step 工具行 / done 终态）。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamEvent {
    Delta { text: String },
    Step { tool: String, ok: bool, output: String },
    Done { status: String },
}

/// 账本总量快照（engine+model 全聚合；失败回零，不挡主流程）。
fn ledger_totals(store: &NeobotStore) -> (i64, i64, f64) {
    let sums = store.ledger_sums().unwrap_or_default();
    let mut in_tokens = 0i64;
    let mut out_tokens = 0i64;
    let mut cost = 0.0f64;
    for (_, _, inn, out, c) in sums {
        in_tokens = in_tokens.saturating_add(inn.max(0));
        out_tokens = out_tokens.saturating_add(out.max(0));
        if c.is_finite() && c > 0.0 {
            cost += c;
        }
    }
    (in_tokens, out_tokens, cost)
}

/// 本轮标签组装（账本差值 + 最新 task 步骤工具；任一步失败即降级空工具/零耗）。
fn collect_labels(
    store: &NeobotStore,
    model: &str,
    mode: ReplyMode,
    before: (i64, i64, f64),
) -> TurnLabels {
    let after = ledger_totals(store);
    let in_tokens = after.0.saturating_sub(before.0).max(0);
    let out_tokens = after.1.saturating_sub(before.1).max(0);
    let cost = if after.2.is_finite() && before.2.is_finite() {
        (after.2 - before.2).max(0.0)
    } else {
        0.0
    };
    let step_tools = store
        .list_tasks(1)
        .unwrap_or_default()
        .first()
        .map(|task| store.list_step_tools(&task.id).unwrap_or_default())
        .unwrap_or_default();
    labels_for_turn(model, mode, step_tools, in_tokens, out_tokens, cost)
}

fn resolve_model_engine(
    store: &NeobotStore,
    config: &NeobotConfig,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<Option<(Box<dyn EngineAdapter>, String)>, String> {
    let (provider, model) = match (model_provider, model_name) {
        (Some(provider), Some(model)) if !model.trim().is_empty() => (provider, model),
        _ => return Ok(None),
    };
    let memory = neotrix_neobot::nt_memory::memory_for_config(config);
    let display = model.trim().to_owned();
    let engine = if provider == "env" {
        let base = std::env::var("NEOBOT_BASE_URL")
            .unwrap_or_else(|_| neotrix_neobot::nt_http_engine::DEFAULT_BASE_URL.to_owned());
        let key = std::env::var("NEOBOT_API_KEY").unwrap_or_default();
        let http_config = neotrix_neobot::HttpEngineConfig {
            base_url: base.trim().trim_end_matches('/').to_owned(),
            model: model.trim().to_owned(),
            timeout_secs: neotrix_neobot::nt_http_engine::DEFAULT_TIMEOUT_SECS,
        };
        HttpEngine::new(http_config, key).map_err(|err| err.to_string())?
    } else {
        let item = store
            .get_provider(&provider)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("no such provider '{provider}'"))?;
        if !item.enabled {
            return Err(format!("provider '{provider}' is off"));
        }
        item.http_engine(Some(&model)).map_err(|err| err.to_string())?
    };
    Ok(Some((
        Box::new(engine.with_memory_context(memory)),
        display,
    )))
}
