use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSession {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub surface: String,
    pub status: String,
    pub project: Option<String>,
    pub started_at: i64,
    pub last_active_at: i64,
    pub active_duration_minutes: u32,
    pub command_count: u32,
    pub file_changes: u32,
    pub error_count: u32,
    pub remote_host: Option<String>,
    pub remote_location: Option<String>,
    pub sync_status: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSessionSummary {
    pub total_sessions: u32,
    pub active_local: u32,
    pub active_remote: u32,
    pub active_teleport: u32,
    pub total_active: u32,
    pub total_idle: u32,
    pub total_paused: u32,
    pub total_errors: u32,
    pub most_active_project: Option<String>,
    pub avg_duration_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSessionFilter {
    pub types: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub projects: Option<Vec<String>>,
    pub surfaces: Option<Vec<String>>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSessionGroup {
    pub group_by: String,
    pub groups: Vec<GroupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEntry {
    pub key: String,
    pub sessions: Vec<UnifiedSession>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSessionStats {
    pub total_sessions_all_time: u32,
    pub total_active_hours: f64,
    pub most_used_surface: String,
    pub most_used_type: String,
    pub sessions_per_day: f64,
    pub peak_hour: u32,
}

struct UnifiedSessionState {
    sessions: Vec<UnifiedSession>,
}

impl UnifiedSessionState {
    fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sessions: vec![
                UnifiedSession {
                    id: "local-1".into(), name: "CLI Workspace".into(), r#type: "local".into(), surface: "cli".into(), status: "active".into(), project: Some("neotrix-core".into()),
                    started_at: now - 7200, last_active_at: now - 60, active_duration_minutes: 45, command_count: 128, file_changes: 34, error_count: 2,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["dev".into(), "cli".into()],
                },
                UnifiedSession {
                    id: "local-2".into(), name: "Desktop Dev".into(), r#type: "local".into(), surface: "desktop".into(), status: "active".into(), project: Some("neotrix-tauri".into()),
                    started_at: now - 14400, last_active_at: now - 120, active_duration_minutes: 120, command_count: 340, file_changes: 89, error_count: 5,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["dev".into(), "tauri".into(), "rust".into()],
                },
                UnifiedSession {
                    id: "local-3".into(), name: "Mobile Review".into(), r#type: "local".into(), surface: "mobile".into(), status: "idle".into(), project: Some("neotrix-mobile".into()),
                    started_at: now - 36000, last_active_at: now - 5400, active_duration_minutes: 180, command_count: 67, file_changes: 12, error_count: 1,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["review".into(), "mobile".into()],
                },
                UnifiedSession {
                    id: "local-4".into(), name: "Background Crawler".into(), r#type: "local".into(), surface: "background".into(), status: "active".into(), project: Some("neotrix-crawler".into()),
                    started_at: now - 86400, last_active_at: now - 30, active_duration_minutes: 480, command_count: 1200, file_changes: 0, error_count: 23,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["crawler".into(), "background".into()],
                },
                UnifiedSession {
                    id: "remote-1".into(), name: "Server Deploy".into(), r#type: "remote".into(), surface: "desktop".into(), status: "active".into(), project: Some("infra".into()),
                    started_at: now - 3600, last_active_at: now - 10, active_duration_minutes: 55, command_count: 89, file_changes: 23, error_count: 0,
                    remote_host: Some("ec2-us-west".into()), remote_location: Some("us-west-2".into()), sync_status: "synced".into(), tags: vec!["deploy".into(), "aws".into(), "infra".into()],
                },
                UnifiedSession {
                    id: "remote-2".into(), name: "DB Migration".into(), r#type: "remote".into(), surface: "cli".into(), status: "paused".into(), project: Some("data-migration".into()),
                    started_at: now - 7200, last_active_at: now - 3600, active_duration_minutes: 30, command_count: 45, file_changes: 8, error_count: 3,
                    remote_host: Some("do-droplet".into()), remote_location: Some("sfo3".into()), sync_status: "pending".into(), tags: vec!["db".into(), "migration".into()],
                },
                UnifiedSession {
                    id: "remote-3".into(), name: "Log Analysis".into(), r#type: "remote".into(), surface: "background".into(), status: "active".into(), project: Some("analytics".into()),
                    started_at: now - 43200, last_active_at: now - 300, active_duration_minutes: 240, command_count: 890, file_changes: 0, error_count: 12,
                    remote_host: Some("hetzner".into()), remote_location: Some("hel1".into()), sync_status: "synced".into(), tags: vec!["logging".into(), "analysis".into()],
                },
                UnifiedSession {
                    id: "teleport-1".into(), name: "Code Review".into(), r#type: "teleport".into(), surface: "desktop".into(), status: "active".into(), project: Some("neotrix-core".into()),
                    started_at: now - 1800, last_active_at: now - 45, active_duration_minutes: 25, command_count: 56, file_changes: 15, error_count: 1,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["review".into(), "teleport".into()],
                },
                UnifiedSession {
                    id: "teleport-2".into(), name: "Bug Bash".into(), r#type: "teleport".into(), surface: "cli".into(), status: "active".into(), project: Some("neotrix-test".into()),
                    started_at: now - 900, last_active_at: now - 5, active_duration_minutes: 14, command_count: 210, file_changes: 42, error_count: 8,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["testing".into(), "bugs".into()],
                },
                UnifiedSession {
                    id: "teleport-3".into(), name: "Pair Debug".into(), r#type: "teleport".into(), surface: "web".into(), status: "idle".into(), project: Some("neotrix-core".into()),
                    started_at: now - 10800, last_active_at: now - 6000, active_duration_minutes: 90, command_count: 178, file_changes: 31, error_count: 4,
                    remote_host: None, remote_location: None, sync_status: "synced".into(), tags: vec!["debug".into(), "pair".into()],
                },
                UnifiedSession {
                    id: "proxy-1".into(), name: "API Testing".into(), r#type: "proxy".into(), surface: "cli".into(), status: "idle".into(), project: Some("api-gateway".into()),
                    started_at: now - 21600, last_active_at: now - 9000, active_duration_minutes: 60, command_count: 320, file_changes: 5, error_count: 6,
                    remote_host: Some("staging-proxy".into()), remote_location: Some("proxy-nyc".into()), sync_status: "conflict".into(), tags: vec!["api".into(), "testing".into()],
                },
                UnifiedSession {
                    id: "proxy-2".into(), name: "Security Audit".into(), r#type: "proxy".into(), surface: "desktop".into(), status: "active".into(), project: Some("security".into()),
                    started_at: now - 28800, last_active_at: now - 900, active_duration_minutes: 150, command_count: 670, file_changes: 120, error_count: 2,
                    remote_host: Some("vpn-gateway".into()), remote_location: Some("gateway-fra".into()), sync_status: "synced".into(), tags: vec!["security".into(), "audit".into()],
                },
            ],
        }
    }
}

static STATE: LazyLock<Mutex<UnifiedSessionState>> =
    LazyLock::new(|| Mutex::new(UnifiedSessionState::new()));

fn matches_filter(session: &UnifiedSession, filter: &UnifiedSessionFilter) -> bool {
    if let Some(ref types) = filter.types {
        if !types.contains(&session.r#type) {
            return false;
        }
    }
    if let Some(ref statuses) = filter.statuses {
        if !statuses.contains(&session.status) {
            return false;
        }
    }
    if let Some(ref projects) = filter.projects {
        if !session.project.as_ref().is_some_and(|p| projects.contains(p)) {
            return false;
        }
    }
    if let Some(ref surfaces) = filter.surfaces {
        if !surfaces.contains(&session.surface) {
            return false;
        }
    }
    if let Some(ref search) = filter.search {
        let q = search.to_lowercase();
        if !session.name.to_lowercase().contains(&q)
            && !session.project.as_ref().is_some_and(|p| p.to_lowercase().contains(&q))
            && !session.tags.iter().any(|t| t.to_lowercase().contains(&q))
        {
            return false;
        }
    }
    true
}

#[tauri::command]
pub fn unified_session_list(filter: Option<UnifiedSessionFilter>) -> Result<Vec<UnifiedSession>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let filter = filter.unwrap_or(UnifiedSessionFilter {
        types: None, statuses: None, projects: None, surfaces: None, search: None,
    });
    Ok(state.sessions.iter().filter(|s| matches_filter(s, &filter)).cloned().collect())
}

#[tauri::command]
pub fn unified_session_get(id: String) -> Result<UnifiedSession, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state.sessions.iter().find(|s| s.id == id).cloned()
        .ok_or_else(|| format!("Session not found: {}", id))
}

#[tauri::command]
pub fn unified_session_summary() -> Result<UnifiedSessionSummary, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let total = state.sessions.len() as u32;
    let active_local = state.sessions.iter().filter(|s| s.r#type == "local" && s.status == "active").count() as u32;
    let active_remote = state.sessions.iter().filter(|s| s.r#type == "remote" && s.status == "active").count() as u32;
    let active_teleport = state.sessions.iter().filter(|s| s.r#type == "teleport" && s.status == "active").count() as u32;
    let total_active = state.sessions.iter().filter(|s| s.status == "active").count() as u32;
    let total_idle = state.sessions.iter().filter(|s| s.status == "idle").count() as u32;
    let total_paused = state.sessions.iter().filter(|s| s.status == "paused").count() as u32;
    let total_errors = state.sessions.iter().filter(|s| s.status == "error").count() as u32;
    let mut project_counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    for s in &state.sessions {
        if let Some(ref p) = s.project {
            *project_counts.entry(p).or_insert(0) += 1;
        }
    }
    let most_active_project = project_counts.into_iter().max_by_key(|(_, c)| *c).map(|(p, _)| p.to_string());
    let avg_duration = if total > 0 {
        state.sessions.iter().map(|s| s.active_duration_minutes as f64).sum::<f64>() / total as f64
    } else { 0.0 };
    Ok(UnifiedSessionSummary {
        total_sessions: total, active_local, active_remote, active_teleport,
        total_active, total_idle, total_paused, total_errors,
        most_active_project, avg_duration_minutes: avg_duration,
    })
}

#[tauri::command]
pub fn unified_session_group_by(field: String) -> Result<UnifiedSessionGroup, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let mut groups: std::collections::HashMap<String, Vec<UnifiedSession>> = std::collections::HashMap::new();
    for session in &state.sessions {
        let key = match field.as_str() {
            "project" => session.project.clone().unwrap_or_else(|| "none".into()),
            "type" => session.r#type.clone(),
            "status" => session.status.clone(),
            "surface" => session.surface.clone(),
            _ => return Err(format!("Invalid group_by field: {}", field)),
        };
        groups.entry(key).or_default().push(session.clone());
    }
    let mut entries: Vec<GroupEntry> = groups.into_iter()
        .map(|(key, sessions)| {
            let count = sessions.len() as u32;
            GroupEntry { key, sessions, count }
        })
        .collect();
    entries.sort_by(|a, b| b.count.cmp(&a.count));
    Ok(UnifiedSessionGroup { group_by: field, groups: entries })
}

#[tauri::command]
pub fn unified_session_search(query: String) -> Result<Vec<UnifiedSession>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let q = query.to_lowercase();
    Ok(state.sessions.iter().filter(|s| {
        s.name.to_lowercase().contains(&q)
            || s.project.as_ref().is_some_and(|p| p.to_lowercase().contains(&q))
            || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
            || s.r#type.to_lowercase().contains(&q)
            || s.surface.to_lowercase().contains(&q)
            || s.remote_host.as_ref().is_some_and(|h| h.to_lowercase().contains(&q))
    }).cloned().collect())
}

#[tauri::command]
pub fn unified_session_stats() -> Result<UnifiedSessionStats, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let total = state.sessions.len() as u32;
    let total_hours: f64 = state.sessions.iter().map(|s| s.active_duration_minutes as f64 / 60.0).sum();
    let mut surface_counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    let mut type_counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    for s in &state.sessions {
        *surface_counts.entry(&s.surface).or_insert(0) += 1;
        *type_counts.entry(&s.r#type).or_insert(0) += 1;
    }
    let most_used_surface = surface_counts.into_iter().max_by_key(|(_, c)| *c).map(|(s, _)| s.to_string()).unwrap_or_else(|| "none".into());
    let most_used_type = type_counts.into_iter().max_by_key(|(_, c)| *c).map(|(t, _)| t.to_string()).unwrap_or_else(|| "none".into());
    Ok(UnifiedSessionStats {
        total_sessions_all_time: total,
        total_active_hours: f64::round(total_hours * 100.0) / 100.0,
        most_used_surface,
        most_used_type,
        sessions_per_day: 4.2,
        peak_hour: 14,
    })
}

#[tauri::command]
pub fn unified_session_connect(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.iter_mut().find(|s| s.id == id)
        .ok_or_else(|| format!("Session not found: {}", id))?;
    session.status = "active".into();
    session.last_active_at = chrono::Utc::now().timestamp();
    Ok(())
}

#[tauri::command]
pub fn unified_session_disconnect(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.iter_mut().find(|s| s.id == id)
        .ok_or_else(|| format!("Session not found: {}", id))?;
    session.status = "idle".into();
    Ok(())
}

#[tauri::command]
pub fn unified_session_tag(id: String, tags: Vec<String>) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.iter_mut().find(|s| s.id == id)
        .ok_or_else(|| format!("Session not found: {}", id))?;
    for tag in tags {
        if !session.tags.contains(&tag) {
            session.tags.push(tag);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn unified_session_untag(id: String, tags: Vec<String>) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.iter_mut().find(|s| s.id == id)
        .ok_or_else(|| format!("Session not found: {}", id))?;
    session.tags.retain(|t| !tags.contains(t));
    Ok(())
}

#[tauri::command]
pub fn unified_session_export() -> Result<String, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&state.sessions).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unified_session_import(data: String) -> Result<usize, String> {
    let sessions: Vec<UnifiedSession> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let count = sessions.len();
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    for s in sessions {
        if !state.sessions.iter().any(|existing| existing.id == s.id) {
            state.sessions.push(s);
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn unified_session_refresh() -> Result<UnifiedSessionSummary, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let statuses = ["active", "idle", "active", "active", "paused"];
    for (i, session) in state.sessions.iter_mut().enumerate() {
        let pseudo_seed = (now as usize).wrapping_mul(6364136223846793005).wrapping_add(i);
        session.last_active_at = now - ((pseudo_seed % 3600) as i64).max(5);
        session.status = statuses[pseudo_seed % statuses.len()].into();
        session.active_duration_minutes = session.active_duration_minutes.saturating_add((pseudo_seed % 10) as u32 + 1);
    }
    drop(state);
    unified_session_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_session_list_all() {
        let sessions = unified_session_list(None).unwrap();
        assert_eq!(sessions.len(), 12);
    }

    #[test]
    fn test_unified_session_filter_by_type() {
        let filter = UnifiedSessionFilter {
            types: Some(vec!["local".into()]),
            statuses: None, projects: None, surfaces: None, search: None,
        };
        let sessions = unified_session_list(Some(filter)).unwrap();
        assert_eq!(sessions.len(), 4);
        assert!(sessions.iter().all(|s| s.r#type == "local"));
    }

    #[test]
    fn test_unified_session_summary_counts() {
        let summary = unified_session_summary().unwrap();
        assert_eq!(summary.total_sessions, 12);
        assert!(summary.total_active >= summary.active_local + summary.active_remote + summary.active_teleport);
        assert!(summary.total_active > 0);
    }

    #[test]
    fn test_unified_session_connect_and_disconnect() {
        let id = "local-1".to_string();
        unified_session_disconnect(id.clone()).unwrap();
        let session = unified_session_get(id.clone()).unwrap();
        assert_eq!(session.status, "idle");
        unified_session_connect(id.clone()).unwrap();
        let session = unified_session_get(id.clone()).unwrap();
        assert_eq!(session.status, "active");
    }
}
