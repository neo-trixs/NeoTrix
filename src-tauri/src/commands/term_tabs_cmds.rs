//! Multi-Terminal Tabs per Thread — Codex Desktop-style terminal tabs within sessions
//!
//! Each session/thread can have multiple terminal tabs running independently.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTab {
    pub id: String,
    pub session_id: String,
    pub name: String,
    pub index: u32,
    pub cwd: String,
    pub shell: String,
    pub is_active: bool,
    pub created_at: i64,
    pub last_used_at: i64,
    pub color: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTabLayout {
    pub session_id: String,
    pub tabs: Vec<TerminalTab>,
    pub active_tab_id: Option<String>,
    pub layout: String,
    pub split_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTabGroup {
    pub id: String,
    pub name: String,
    pub session_id: String,
    pub tab_ids: Vec<String>,
    pub is_collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTabConfig {
    pub default_shell: String,
    pub default_columns: u32,
    pub default_rows: u32,
    pub max_tabs_per_session: u32,
    pub enable_colors: bool,
    pub enable_groups: bool,
    pub scrollback_lines: u32,
}

impl Default for TerminalTabConfig {
    fn default() -> Self {
        Self {
            default_shell: if cfg!(target_os = "windows") { "powershell".into() } else { "zsh".into() },
            default_columns: 120,
            default_rows: 40,
            max_tabs_per_session: 20,
            enable_colors: true,
            enable_groups: true,
            scrollback_lines: 10000,
        }
    }
}

struct TermTabState {
    sessions: HashMap<String, Vec<TerminalTab>>,
    groups: HashMap<String, TerminalTabGroup>,
    layouts: HashMap<String, TerminalTabLayout>,
    config: TerminalTabConfig,
}

impl TermTabState {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            groups: HashMap::new(),
            layouts: HashMap::new(),
            config: TerminalTabConfig::default(),
        }
    }
}

static STATE: LazyLock<Mutex<TermTabState>> = LazyLock::new(|| Mutex::new(TermTabState::new()));

fn generate_tab_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("tab-{:x}", nanos)
}

fn generate_group_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("grp-{:x}", nanos)
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

#[tauri::command]
pub fn term_tabs_create(
    session_id: String,
    name: Option<String>,
    cwd: Option<String>,
    shell: Option<String>,
    color: Option<String>,
) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let max_tabs = state.config.max_tabs_per_session as usize;
    let default_shell = state.config.default_shell.clone();
    let default_cols = state.config.default_columns;
    let default_rows = state.config.default_rows;

    let tabs = state.sessions.entry(session_id.clone()).or_default();

    if tabs.len() >= max_tabs {
        return Err(format!("Max tabs per session ({}) reached", max_tabs));
    }

    let idx = tabs.len() as u32;
    let tab_name = name.unwrap_or_else(|| format!("Terminal {}", idx + 1));
    let tab_shell = shell.unwrap_or(default_shell);
    let tab_cwd = cwd.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".into())
    });

    let id = generate_tab_id();
    let now = now_secs();
    let is_active = tabs.is_empty();

    let tab = TerminalTab {
        id: id.clone(),
        session_id,
        name: tab_name,
        index: idx,
        cwd: tab_cwd,
        shell: tab_shell,
        is_active,
        created_at: now,
        last_used_at: now,
        color,
        env_vars: HashMap::new(),
        columns: default_cols,
        rows: default_rows,
    };

    tabs.push(tab);
    Ok(id)
}

#[tauri::command]
pub fn term_tabs_list(session_id: String) -> Result<Vec<TerminalTab>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.sessions.get(&session_id).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn term_tabs_get(tab_id: String) -> Result<TerminalTab, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    for tabs in state.sessions.values() {
        if let Some(tab) = tabs.iter().find(|t| t.id == tab_id) {
            return Ok(tab.clone());
        }
    }
    Err(format!("Tab '{}' not found", tab_id))
}

#[tauri::command]
pub fn term_tabs_rename(tab_id: String, name: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    for tabs in state.sessions.values_mut() {
        if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.name = name;
            tab.last_used_at = now_secs();
            return Ok(());
        }
    }
    Err(format!("Tab '{}' not found", tab_id))
}

#[tauri::command]
pub fn term_tabs_close(tab_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;

    let mut session_key: Option<String> = None;
    let mut was_active = false;
    for (sid, tabs) in &state.sessions {
        if let Some(tab) = tabs.iter().find(|t| t.id == tab_id) {
            was_active = tab.is_active;
            session_key = Some(sid.clone());
            break;
        }
    }

    let sid = session_key.ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
    let tabs = state.sessions.get_mut(&sid).unwrap();
    tabs.retain(|t| t.id != tab_id);

    if was_active {
        if let Some(first) = tabs.first_mut() {
            first.is_active = true;
        }
    }

    state.groups.retain(|_, g| {
        g.tab_ids.retain(|id| id != &tab_id);
        !g.tab_ids.is_empty()
    });

    Ok(())
}

#[tauri::command]
pub fn term_tabs_activate(tab_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    for tabs in state.sessions.values_mut() {
        let mut found = false;
        for tab in tabs.iter_mut() {
            if tab.id == tab_id {
                tab.is_active = true;
                tab.last_used_at = now_secs();
                found = true;
            } else {
                tab.is_active = false;
            }
        }
        if found {
            return Ok(());
        }
    }
    Err(format!("Tab '{}' not found", tab_id))
}

#[tauri::command]
pub fn term_tabs_reorder(session_id: String, tab_ids: Vec<String>) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let tabs = state.sessions.get_mut(&session_id)
        .ok_or_else(|| format!("Session '{}' not found", session_id))?;

    if tab_ids.len() != tabs.len() {
        return Err(format!(
            "tab_ids length ({}) != current tabs ({})",
            tab_ids.len(),
            tabs.len()
        ));
    }

    let mut reordered: Vec<TerminalTab> = Vec::with_capacity(tab_ids.len());
    for id in &tab_ids {
        let pos = tabs.iter().position(|t| t.id == *id)
            .ok_or_else(|| format!("Tab '{}' not found in session", id))?;
        reordered.push(tabs.remove(pos));
    }

    for (i, tab) in reordered.iter_mut().enumerate() {
        tab.index = i as u32;
    }

    *tabs = reordered;
    Ok(())
}

#[tauri::command]
pub fn term_tabs_set_color(tab_id: String, color: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    for tabs in state.sessions.values_mut() {
        if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.color = Some(color);
            return Ok(());
        }
    }
    Err(format!("Tab '{}' not found", tab_id))
}

#[tauri::command]
pub fn term_tabs_layout(session_id: String) -> Result<TerminalTabLayout, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    if let Some(layout) = state.layouts.get(&session_id) {
        return Ok(layout.clone());
    }
    let tabs = state.sessions.get(&session_id).cloned().unwrap_or_default();
    let active_id = tabs.iter().find(|t| t.is_active).map(|t| t.id.clone());
    let lt = if tabs.len() > 1 { "grid".into() } else { "horizontal".into() };
    Ok(TerminalTabLayout {
        session_id,
        tabs,
        active_tab_id: active_id,
        layout: lt,
        split_pct: 50.0,
    })
}

#[tauri::command]
pub fn term_tabs_set_layout(
    session_id: String,
    layout: String,
    split_pct: Option<f64>,
) -> Result<(), String> {
    if !["horizontal", "vertical", "grid"].contains(&layout.as_str()) {
        return Err(format!(
            "Invalid layout '{}'. Must be horizontal, vertical, or grid",
            layout
        ));
    }

    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let tabs = state.sessions.get(&session_id).cloned().unwrap_or_default();
    let active_id = tabs.iter().find(|t| t.is_active).map(|t| t.id.clone());

    let tab_layout = state.layouts.entry(session_id.clone()).or_insert_with(|| {
        TerminalTabLayout {
            session_id: session_id.clone(),
            tabs: Vec::new(),
            active_tab_id: None,
            layout: "horizontal".into(),
            split_pct: 50.0,
        }
    });

    tab_layout.tabs = tabs;
    tab_layout.active_tab_id = active_id;
    tab_layout.layout = layout;
    if let Some(pct) = split_pct {
        tab_layout.split_pct = pct.max(10.0).min(90.0);
    }

    Ok(())
}

#[tauri::command]
pub fn term_tabs_group_create(
    name: String,
    session_id: String,
    tab_ids: Vec<String>,
) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;

    let tabs = state.sessions.get(&session_id)
        .ok_or_else(|| format!("Session '{}' not found", session_id))?;
    for id in &tab_ids {
        if !tabs.iter().any(|t| t.id == *id) {
            return Err(format!("Tab '{}' not found in session '{}'", id, session_id));
        }
    }

    let id = generate_group_id();
    let group = TerminalTabGroup {
        id: id.clone(),
        name,
        session_id,
        tab_ids,
        is_collapsed: false,
    };

    state.groups.insert(id.clone(), group);
    Ok(id)
}

#[tauri::command]
pub fn term_tabs_group_list(session_id: String) -> Result<Vec<TerminalTabGroup>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.groups.values()
        .filter(|g| g.session_id == session_id)
        .cloned()
        .collect())
}

#[tauri::command]
pub fn term_tabs_group_delete(group_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.groups.remove(&group_id)
        .ok_or_else(|| format!("Group '{}' not found", group_id))?;
    Ok(())
}

#[tauri::command]
pub fn term_tabs_config() -> Result<TerminalTabConfig, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn term_tabs_set_config(config: TerminalTabConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn term_tabs_stats() -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let total_tabs: usize = state.sessions.values().map(|t| t.len()).sum();
    let total_groups = state.groups.len();
    let session_count = state.sessions.len();
    let avg = if session_count > 0 { total_tabs as f64 / session_count as f64 } else { 0.0 };
    Ok(serde_json::json!({
        "total_tabs": total_tabs,
        "total_groups": total_groups,
        "avg_tabs_per_session": avg,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        if let Ok(mut state) = STATE.lock() {
            state.sessions.clear();
            state.groups.clear();
            state.layouts.clear();
            state.config = TerminalTabConfig::default();
        }
    }

    #[test]
    fn test_term_tabs_create_and_list() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = term_tabs_create("ts-session".into(), None, None, None, None).unwrap();
        assert!(id.starts_with("tab-"));

        let tabs = term_tabs_list("ts-session".into()).unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].name, "Terminal 1");
        assert!(tabs[0].is_active);
    }

    #[test]
    fn test_term_tabs_rename_and_activate() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = term_tabs_create("ra-test".into(), None, None, None, None).unwrap();
        term_tabs_rename(id.clone(), "MyTab".into()).unwrap();

        let tab = term_tabs_get(id.clone()).unwrap();
        assert_eq!(tab.name, "MyTab");

        let id2 = term_tabs_create("ra-test".into(), Some("Second".into()), None, None, None).unwrap();
        term_tabs_activate(id.clone()).unwrap();

        let tabs = term_tabs_list("ra-test".into()).unwrap();
        assert!(tabs.iter().find(|t| t.id == id).unwrap().is_active);
        assert!(!tabs.iter().find(|t| t.id == id2).unwrap().is_active);
    }

    #[test]
    fn test_term_tabs_close_activates_next() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id1 = term_tabs_create("ct-test".into(), Some("A".into()), None, None, None).unwrap();
        let id2 = term_tabs_create("ct-test".into(), Some("B".into()), None, None, None).unwrap();
        term_tabs_activate(id1.clone()).unwrap();
        term_tabs_close(id1.clone()).unwrap();

        let tabs = term_tabs_list("ct-test".into()).unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].id, id2);
        assert!(tabs[0].is_active);
    }

    #[test]
    fn test_term_tabs_group_create_and_delete() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id1 = term_tabs_create("grp-test".into(), Some("A".into()), None, None, None).unwrap();
        let id2 = term_tabs_create("grp-test".into(), Some("B".into()), None, None, None).unwrap();

        let gid = term_tabs_group_create("MyGroup".into(), "grp-test".into(), vec![id1, id2]).unwrap();
        assert!(gid.starts_with("grp-"));

        let groups = term_tabs_group_list("grp-test".into()).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "MyGroup");

        term_tabs_group_delete(gid).unwrap();
        assert!(term_tabs_group_list("grp-test".into()).unwrap().is_empty());
    }
}
