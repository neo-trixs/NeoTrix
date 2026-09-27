//! neobot Tauri 命令 — 本地 agent App 的桌面入口.
//!
//! 薄封装 `neotrix-neobot` (SQLite + 网关 + 引擎), 不含业务逻辑:
//! 读操作走同步命令, `run` (可长达分钟级) 经 `spawn_blocking`
//! 避免占住 Tauri 异步运行时. 错误统一为 `String` 给前端.

use neotrix_neobot::{
    CliEngine, EngineAdapter, EngineKind, HttpEngine, LocalEchoEngine, NeobotConfig, NeobotStore,
    OpencodeEngine, run_local_turn,
};

/// 前端任务 DTO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotTaskItem {
    pub id: String,
    pub title: String,
    pub status: String,
}

/// 前端模型 DTO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotModelItem {
    pub id: String,
    pub owner: String,
}

/// 前端审计 DTO (明细已由核心脱敏).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotAuditItem {
    pub at: String,
    pub tool: String,
    pub decision: String,
    pub rule: Option<String>,
}

fn load_config() -> Result<NeobotConfig, String> {
    NeobotConfig::from_env().map_err(|err| err.to_string())
}

fn open_store(config: &NeobotConfig) -> Result<NeobotStore, String> {
    let path = config.db_path().to_string_lossy().into_owned();
    NeobotStore::open(&path).map_err(|err| err.to_string())
}

fn resolve_engine(config: &NeobotConfig) -> Result<EngineSelection, String> {
    match &config.engine {
        EngineKind::Echo => Ok(EngineSelection::Echo),
        EngineKind::Cli { command } => CliEngine::new(command)
            .map(EngineSelection::Cli)
            .map_err(|err| err.to_string()),
        EngineKind::Http { .. } => HttpEngine::from_env()
            .map(EngineSelection::Http)
            .map_err(|err| err.to_string()),
        EngineKind::Opencode { model } => OpencodeEngine::new(model)
            .map(EngineSelection::Opencode)
            .map_err(|err| err.to_string()),
    }
}

enum EngineSelection {
    Echo,
    Cli(CliEngine),
    Http(HttpEngine),
    Opencode(OpencodeEngine),
}

impl EngineSelection {
    fn run_turn(
        &self,
        store: &NeobotStore,
        config: &NeobotConfig,
        title: &str,
        text: &str,
    ) -> Result<String, String> {
        let local_echo;
        let status = match self {
            Self::Echo => {
                local_echo = LocalEchoEngine;
                run_local_turn(store, config, &local_echo, title, text)
            }
            Self::Cli(engine) => run_local_turn(store, config, engine, title, text),
            Self::Http(engine) => run_local_turn(store, config, engine, title, text),
            Self::Opencode(engine) => run_local_turn(store, config, engine, title, text),
        }
        .map_err(|err| err.to_string())?;
        Ok(status.as_str().to_owned())
    }
}

/// 自检 (目录/DB/引擎探活).
#[tauri::command]
pub fn neobot_doctor() -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let engine_info = match resolve_engine(&config)? {
        EngineSelection::Echo => LocalEchoEngine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Cli(engine) => engine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Http(engine) => engine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Opencode(engine) => engine.probe().map_err(|err| err.to_string())?,
    };
    let tasks = store.list_tasks(1).map_err(|err| err.to_string())?;
    Ok(format!("doctor ok: engine={engine_info} tasks={}", tasks.len()))
}

/// 跑一轮本地任务 (阻塞, 前端请配超时 ≥120s).
#[tauri::command]
pub async fn neobot_run(title: String, text: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = load_config()?;
        let store = open_store(&config)?;
        resolve_engine(&config)?.run_turn(&store, &config, &title, &text)
    })
    .await
    .map_err(|err| err.to_string())?
}

/// 列任务 (近 20).
#[tauri::command]
pub fn neobot_tasks() -> Result<Vec<NeobotTaskItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let tasks = store.list_tasks(20).map_err(|err| err.to_string())?;
    Ok(tasks
        .into_iter()
        .map(|task| NeobotTaskItem {
            id: task.id,
            title: task.title,
            status: task.status.as_str().to_owned(),
        })
        .collect())
}

/// 列审计 (近 20).
#[tauri::command]
pub fn neobot_audit() -> Result<Vec<NeobotAuditItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let events = store.list_audit(20).map_err(|err| err.to_string())?;
    Ok(events
        .into_iter()
        .map(|event| NeobotAuditItem {
            at: event.at,
            tool: event.tool,
            decision: event.decision.as_str().to_owned(),
            rule: event.rule,
        })
        .collect())
}

/// 列模型池 (`GET /v1/models`, 需要 HTTP 端点可达).
#[tauri::command]
pub fn neobot_models() -> Result<Vec<NeobotModelItem>, String> {
    let engine = HttpEngine::for_listing().map_err(|err| err.to_string())?;
    let mut models = engine.list_models().map_err(|err| err.to_string())?;
    models.sort();
    Ok(models
        .into_iter()
        .map(|(id, owner)| NeobotModelItem { id, owner })
        .collect())
}
