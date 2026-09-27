//! `nt_cmd_sys` — 模型池/providers/routines/skills/记忆/接管/在线/自检/窗口.

use super::{DaemonMap, EngineSelection, NeobotPresenceItem, PresenceMap, load_config, now_epoch, now_secs, open_store, resolve_engine};
use super::{NeobotModelItem, NeobotProviderItem, NeobotRoutineItem, NeobotSkillItem};
use neotrix_neobot::{EngineAdapter, HttpEngine, LocalEchoEngine, fire_routine, sweep_routines};
/// probe + 回收计数

#[tauri::command]
pub fn neobot_doctor() -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let engine_info = match resolve_engine(&config)? {
        EngineSelection::Echo => LocalEchoEngine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Cli(engine) => engine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Opencode(engine) => engine.probe().map_err(|err| err.to_string())?,
        EngineSelection::Http(engine) => engine.probe().map_err(|err| err.to_string())?,
    };
    let tasks = store.list_tasks(1).map_err(|err| err.to_string())?;
    Ok(format!("doctor ok: engine={engine_info} tasks={}", tasks.len()))
}
/// HttpEngine::for_listing + providers 聚合

#[tauri::command]
pub fn neobot_models() -> Result<Vec<NeobotModelItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let mut out: Vec<NeobotModelItem> = Vec::new();
    // 1) legacy env 端点
    if let Ok(engine) = HttpEngine::for_listing() {
        if let Ok(models) = engine.list_models() {
            for (id, owner) in models {
                out.push(NeobotModelItem { id, owner, source: "env".to_owned() });
            }
        }
    }
    // 2) 注册端点（三端同律 `pool_models`；桌面静默跳过不可达提示）。
    let (models, _) = neotrix_neobot::pool_models(&store);
    for model in models {
        out.push(NeobotModelItem {
            id: model.id,
            owner: model.owner,
            source: model.provider,
        });
    }
    out.sort_by(|a, b| (&a.source, &a.id).cmp(&(&b.source, &b.id)));
    Ok(out)
}
/// providers 注册表（key 只存变量名；preset 经 CLI）

#[tauri::command]
pub fn neobot_providers() -> Result<Vec<NeobotProviderItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let providers = store.list_providers().map_err(|err| err.to_string())?;
    Ok(providers
        .into_iter()
        .map(|p| NeobotProviderItem {
            name: p.name,
            base_url: p.base_url,
            key_env: p.key_env,
            model: p.model,
            enabled: p.enabled,
        })
        .collect())
}
/// providers 注册表（key 只存变量名；preset 经 CLI）

#[tauri::command]
pub fn neobot_provider_add(
    name: String,
    base_url: String,
    key_env: String,
    model: String,
) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .upsert_provider(&neotrix_neobot::Provider {
            name: name.trim().to_owned(),
            base_url: base_url.trim().to_owned(),
            key_env: key_env.trim().to_owned(),
            model: model.trim().to_owned(),
            enabled: true,
        })
        .map_err(|err| err.to_string())
}
/// providers 注册表（key 只存变量名；preset 经 CLI）

#[tauri::command]
pub fn neobot_provider_remove(name: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.remove_provider(&name).map_err(|err| err.to_string())
}
/// providers 注册表（key 只存变量名；preset 经 CLI）

#[tauri::command]
pub fn neobot_provider_toggle(name: String, enabled: bool) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .set_provider_enabled(&name, enabled)
        .map_err(|err| err.to_string())
}
/// 8K 上限

#[tauri::command]
pub fn neobot_memory_get() -> Result<String, String> {
    let config = load_config()?;
    Ok(neotrix_neobot::nt_memory::read_memory(&config.data_dir))
}
/// 8K 上限

#[tauri::command]
pub fn neobot_memory_set(text: String) -> Result<bool, String> {
    let config = load_config()?;
    neotrix_neobot::nt_memory::append_memory(&config.data_dir, &text)
        .map_err(|err| err.to_string())
}
/// store + fire/sweep_routines

#[tauri::command]
pub fn neobot_routine_list() -> Result<Vec<NeobotRoutineItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let routines = store.list_routines().map_err(|err| err.to_string())?;
    Ok(routines
        .into_iter()
        .map(|r| NeobotRoutineItem {
            name: r.name,
            interval_secs: r.interval_secs,
            owner: r.owner,
            failures: r.failures,
            disabled: r.disabled,
            next_run_at: r.next_run_at,
            instruction: r.instruction,
        })
        .collect())
}
/// store + fire/sweep_routines

#[tauri::command]
pub async fn neobot_routine_fire(
    name: String,
    gate: tauri::State<'_, DaemonMap>,
) -> Result<String, String> {
    let gate = gate.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let config = load_config()?;
        let store = open_store(&config)?;
        let instruction = store
            .list_routines()
            .map_err(|err| err.to_string())?
            .into_iter()
            .find(|r| r.name == name)
            .map(|r| r.instruction)
            .unwrap_or_default();
        let key = format!("fire:{name}");
        let mut gate = gate.lock().map_err(|err| format!("gate lock: {err}"))?;
        if !gate.should_wake(&key) {
            return Ok(format!("{name} debounced (burst merged)"));
        }
        if !gate.triage(&key, &instruction) {
            return Ok(format!("{name} dropped (empty/duplicate instruction)"));
        }
        drop(gate);
        let now = now_epoch();
        let status = match resolve_engine(&config)? {
            EngineSelection::Echo => fire_routine(&store, &config, &LocalEchoEngine, &name, now),
            EngineSelection::Cli(engine) => {
                fire_routine(&store, &config, &engine, &name, now)
            }
            EngineSelection::Opencode(engine) => {
                fire_routine(&store, &config, &engine, &name, now)
            }
            EngineSelection::Http(engine) => fire_routine(&store, &config, &engine, &name, now),
        }
        .map_err(|err| err.to_string())?;
        Ok(format!(
            "{} disabled={}",
            status.0.as_str(),
            status.1
        ))
    })
    .await
    .map_err(|err| err.to_string())?
}
/// store + fire/sweep_routines

#[tauri::command]
pub async fn neobot_routine_sweep(
    gate: tauri::State<'_, DaemonMap>,
) -> Result<Vec<String>, String> {
    let gate = gate.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        {
            let mut gate = gate.lock().map_err(|err| format!("gate lock: {err}"))?;
            if !gate.should_wake("sweep") {
                return Ok(vec!["sweep debounced (burst merged)".to_owned()]);
            }
        }
        let config = load_config()?;
        let store = open_store(&config)?;
        let now = now_epoch();
        let fired = match resolve_engine(&config)? {
            EngineSelection::Echo => {
                sweep_routines(&store, &config, &LocalEchoEngine, now)
            }
            EngineSelection::Cli(engine) => sweep_routines(&store, &config, &engine, now),
            EngineSelection::Opencode(engine) => sweep_routines(&store, &config, &engine, now),
            EngineSelection::Http(engine) => sweep_routines(&store, &config, &engine, now),
        }
        .map_err(|err| err.to_string())?;
        Ok(fired
            .into_iter()
            .map(|(name, status, disabled)| {
                format!("{name}: {}{}", status.as_str(), if disabled { " (OFF)" } else { "" })
            })
            .collect())
    })
    .await
    .map_err(|err| err.to_string())?
}
/// scan_skills（~/.neobot/skills）

#[tauri::command]
pub fn neobot_skills() -> Result<Vec<NeobotSkillItem>, String> {
    let config = load_config()?;
    let (skills, _) = neotrix_neobot::nt_skills::scan_skills(&config.data_dir);
    Ok(skills
        .into_iter()
        .map(|s| NeobotSkillItem {
            name: s.name,
            description: s.description,
        })
        .collect())
}
/// control 表 + 交接审计

#[tauri::command]
pub fn neobot_control_status() -> Result<Option<String>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.control_holder().map_err(|err| err.to_string())
}
/// control 表 + 交接审计

#[tauri::command]
pub fn neobot_control_take(holder: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store.take_control(&holder).map_err(|err| err.to_string())
}
/// control 表 + 交接审计

#[tauri::command]
pub fn neobot_control_release(holder: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .release_control(&holder)
        .map_err(|err| err.to_string())
}
/// 第二窗口 settings.html

#[tauri::command]
pub async fn neobot_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager as _;
    if let Some(window) = app.get_webview_window("settings") {
        window.set_focus().map_err(|err| err.to_string())?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("设置 · NeoBot")
    .inner_size(720.0, 540.0)
    .min_inner_size(620.0, 460.0)
    .center()
    .build()
    .map_err(|err| err.to_string())?;
    Ok(())
}
/// PresenceMap 内存（45s/120s）

#[tauri::command]
pub fn neobot_roster_heartbeat(
    member_id: String,
    kind: String,
    map: tauri::State<'_, PresenceMap>,
) -> Result<(), String> {
    let mut guard = map.0.lock().map_err(|err| err.to_string())?;
    guard.insert(member_id, (kind, now_secs()));
    Ok(())
}
/// PresenceMap 内存（45s/120s）

#[tauri::command]
pub fn neobot_roster_list(
    map: tauri::State<'_, PresenceMap>,
) -> Result<Vec<NeobotPresenceItem>, String> {
    let guard = map.0.lock().map_err(|err| err.to_string())?;
    let now = now_secs();
    let mut out: Vec<NeobotPresenceItem> = guard
        .iter()
        .map(|(id, (kind, seen))| {
            let age = now.saturating_sub(*seen);
            let presence = if age < 45 {
                "active"
            } else if age < 120 {
                "idle"
            } else {
                "off"
            };
            NeobotPresenceItem {
                id: id.clone(),
                kind: kind.clone(),
                presence: presence.to_owned(),
                last_seen_secs: *seen,
            }
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
