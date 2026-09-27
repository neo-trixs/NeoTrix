//! `nt_cmd_tasks` — 任务/审计/账本.

use super::{NeobotAuditItem, NeobotCostActorRow, NeobotCostRow, NeobotTaskItem, load_config, open_store};
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_tasks() -> Result<Vec<NeobotTaskItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let tasks = store.list_tasks(50).map_err(|err| err.to_string())?;
    Ok(tasks
        .into_iter()
        .map(|task| NeobotTaskItem {
            id: task.id,
            title: task.title,
            status: task.status.as_str().to_owned(),
            claimed_by: task.claimed_by,
            visibility: task.visibility,
            conversation_id: task.conversation_id,
            attempts: task.attempts,
        })
        .collect())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_claim(task_id: String, actor_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .claim_task(&task_id, &actor_id)
        .map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_release(task_id: String, actor_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .release_task(&task_id, &actor_id)
        .map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_visibility(task_id: String, visibility: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .set_task_visibility(&task_id, &visibility)
        .map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_cancel(task_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.cancel_task(&task_id).map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_retry(task_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.retry_task(&task_id).map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_rename(task_id: String, title: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.rename_task(&task_id, &title).map_err(|err| err.to_string())
}
/// 对应 store 原子方法

#[tauri::command]
pub fn neobot_task_delete(task_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.delete_task(&task_id).map_err(|err| err.to_string())
}
/// list_audit(50)，明细不出前端

#[tauri::command]
pub fn neobot_audit() -> Result<Vec<NeobotAuditItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let events = store.list_audit(50).map_err(|err| err.to_string())?;
    Ok(events
        .into_iter()
        .map(|event| NeobotAuditItem {
            at: event.at,
            actor: event.actor,
            tool: event.tool,
            decision: event.decision.as_str().to_owned(),
            rule: event.rule,
        })
        .collect())
}
/// ledger 聚合

#[tauri::command]
pub fn neobot_cost_ledger() -> Result<Vec<NeobotCostRow>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let rows = store.ledger_sums().map_err(|err| err.to_string())?;
    Ok(rows
        .into_iter()
        .map(|(engine, model, in_tokens, out_tokens, cost_usd)| NeobotCostRow {
            engine,
            model,
            in_tokens,
            out_tokens,
            cost_usd,
        })
        .collect())
}
/// ledger 聚合

#[tauri::command]
pub fn neobot_cost_ledger_by_actor() -> Result<Vec<NeobotCostActorRow>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let rows = store.ledger_sums_by_actor().map_err(|err| err.to_string())?;
    Ok(rows
        .into_iter()
        .map(|(engine, model, actor, in_tokens, out_tokens, cost_usd)| NeobotCostActorRow {
            engine,
            model,
            actor,
            in_tokens,
            out_tokens,
            cost_usd,
        })
        .collect())
}
