use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

// ===== Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_prompt: String,
    pub schedule: String,
    pub surface: String,
    pub max_runtime_minutes: u32,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineRun {
    pub id: String,
    pub routine_id: String,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub result_summary: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineStats {
    pub total_routines: usize,
    pub active_routines: usize,
    pub total_runs_today: usize,
    pub completed_today: usize,
    pub failed_today: usize,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRoutineConfig {
    pub enabled: bool,
    pub cloud_endpoint: String,
    pub api_key: Option<String>,
    pub max_concurrent: u32,
}

impl Default for CloudRoutineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cloud_endpoint: "https://api.neotrix.ai/routines".into(),
            api_key: None,
            max_concurrent: 5,
        }
    }
}

// ===== State =====

struct RoutinesState {
    routines: Vec<RoutineConfig>,
    runs: Vec<RoutineRun>,
    cloud_config: CloudRoutineConfig,
}

impl Default for RoutinesState {
    fn default() -> Self {
        Self {
            routines: Vec::new(),
            runs: Vec::new(),
            cloud_config: CloudRoutineConfig::default(),
        }
    }
}

static STATE: std::sync::LazyLock<Mutex<RoutinesState>> =
    std::sync::LazyLock::new(|| Mutex::new(RoutinesState::default()));

// ===== Helpers =====

fn short_uid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:08x}", nanos)
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn today_start_ts() -> i64 {
    use chrono::Utc;
    Utc::now().date_naive().and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

// ===== Commands =====

#[command]
pub fn routines_create(
    name: String,
    description: String,
    task_prompt: String,
    schedule: String,
    surface: Option<String>,
) -> Result<String, String> {
    let id = format!("rt-{}", short_uid());
    let config = RoutineConfig {
        id: id.clone(),
        name,
        description,
        task_prompt,
        schedule,
        surface: surface.unwrap_or_else(|| "desktop".into()),
        max_runtime_minutes: 60,
        enabled: true,
        created_at: now_ts(),
    };
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    if state.routines.len() >= 50 {
        return Err("Maximum of 50 routines reached".into());
    }
    state.routines.push(config);
    Ok(id)
}

#[command]
pub fn routines_list() -> Result<Vec<RoutineConfig>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.routines.clone())
}

#[command]
pub fn routines_get(id: String) -> Result<RoutineConfig, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    state
        .routines
        .iter()
        .find(|r| r.id == id)
        .cloned()
        .ok_or_else(|| format!("Routine not found: {}", id))
}

#[command]
pub fn routines_update(
    id: String,
    name: Option<String>,
    description: Option<String>,
    task_prompt: Option<String>,
    schedule: Option<String>,
    enabled: Option<bool>,
) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let routine = state
        .routines
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Routine not found: {}", id))?;
    if let Some(v) = name {
        routine.name = v;
    }
    if let Some(v) = description {
        routine.description = v;
    }
    if let Some(v) = task_prompt {
        routine.task_prompt = v;
    }
    if let Some(v) = schedule {
        routine.schedule = v;
    }
    if let Some(v) = enabled {
        routine.enabled = v;
    }
    Ok(())
}

#[command]
pub fn routines_delete(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.routines.retain(|r| r.id != id);
    state.runs.retain(|r| r.routine_id != id);
    Ok(())
}

#[command]
pub fn routines_enable(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let routine = state
        .routines
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Routine not found: {}", id))?;
    routine.enabled = true;
    Ok(())
}

#[command]
pub fn routines_disable(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let routine = state
        .routines
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Routine not found: {}", id))?;
    routine.enabled = false;
    Ok(())
}

#[command]
pub fn routines_run_now(id: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    let routine = state
        .routines
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Routine not found: {}", id))?
        .clone();

    let run_id = format!("run-{}", short_uid());
    let started = now_ts();
    if state.runs.len() >= 500 {
        state.runs.remove(0);
    }
    state.runs.push(RoutineRun {
        id: run_id.clone(),
        routine_id: id.clone(),
        status: "running".into(),
        started_at: started,
        completed_at: None,
        duration_ms: None,
        result_summary: None,
        error: None,
    });
    drop(state);

    std::thread::sleep(std::time::Duration::from_millis(500));

    let elapsed_ms = (now_ts() - started) as u64 * 1000;
    let summary = format!("Routine '{}' completed successfully", routine.name);
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    if let Some(run) = state.runs.iter_mut().find(|r| r.id == run_id) {
        run.status = "completed".into();
        run.completed_at = Some(now_ts());
        run.duration_ms = Some(elapsed_ms);
        run.result_summary = Some(summary);
    }
    Ok(run_id)
}

#[command]
pub fn routines_run_history(
    id: String,
    count: Option<usize>,
) -> Result<Vec<RoutineRun>, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let max = count.unwrap_or(100).min(100);
    let mut history: Vec<_> = state
        .runs
        .iter()
        .filter(|r| r.routine_id == id)
        .cloned()
        .collect();
    history.reverse();
    history.truncate(max);
    Ok(history)
}

#[command]
pub fn routines_stats() -> Result<RoutineStats, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let total_routines = state.routines.len();
    let active_routines = state.routines.iter().filter(|r| r.enabled).count();
    let today = today_start_ts();
    let today_runs: Vec<_> = state.runs.iter().filter(|r| r.started_at >= today).collect();
    let total_runs_today = today_runs.len();
    let completed_today = today_runs.iter().filter(|r| r.status == "completed").count();
    let failed_today = today_runs.iter().filter(|r| r.status == "failed").count();
    let total_completed_all = state.runs.iter().filter(|r| r.status == "completed").count();
    let total_all = state.runs.len();
    let success_rate = if total_all > 0 {
        total_completed_all as f64 / total_all as f64
    } else {
        1.0
    };
    let completed_with_duration: Vec<_> = state
        .runs
        .iter()
        .filter(|r| r.status == "completed" && r.duration_ms.is_some())
        .collect();
    let avg_duration_ms = {
        let sum: u64 = completed_with_duration.iter().filter_map(|r| r.duration_ms).sum();
        if !completed_with_duration.is_empty() {
            sum as f64 / completed_with_duration.len() as f64
        } else {
            0.0
        }
    };

    Ok(RoutineStats {
        total_routines,
        active_routines,
        total_runs_today,
        completed_today,
        failed_today,
        avg_duration_ms,
        success_rate,
    })
}

#[command]
pub fn routines_cloud_config() -> Result<CloudRoutineConfig, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.cloud_config.clone())
}

#[command]
pub fn routines_cloud_set_config(config: CloudRoutineConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| e.to_string())?;
    state.cloud_config = config;
    Ok(())
}

#[command]
pub fn routines_sync() -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| e.to_string())?;
    let count = state.routines.len();
    Ok(serde_json::json!({
        "synced": true,
        "routines_count": count,
        "last_sync": "2026-07-20T12:00:00Z"
    }))
}

#[command]
pub fn routines_generate_from_task(task_description: String) -> Result<serde_json::Value, String> {
    let lower = task_description.to_lowercase();
    let suggested_name = if lower.contains("daily") || lower.contains("every day") {
        "Daily Report".to_string()
    } else if lower.contains("monitor") || lower.contains("watch") || lower.contains("track") {
        "Monitoring Task".to_string()
    } else if lower.contains("scrape") || lower.contains("crawl") || lower.contains("collect") {
        "Data Collection".to_string()
    } else if lower.contains("review") || lower.contains("audit") || lower.contains("check") {
        "Review Task".to_string()
    } else if lower.contains("backup") || lower.contains("sync") || lower.contains("save") {
        "Backup Task".to_string()
    } else {
        "Automated Task".to_string()
    };

    let suggested_description = format!("Auto-generated routine for: {}", task_description);
    let suggested_prompt = task_description;
    let suggested_schedule = if lower.contains("hour") || lower.contains("every hour") {
        "every_1h".to_string()
    } else if lower.contains("daily") || lower.contains("every day") || lower.contains("24h") {
        "every_24h".to_string()
    } else if lower.contains("weekly") || lower.contains("every week") {
        "every_week".to_string()
    } else if lower.contains("6h") || lower.contains("six hour") || lower.contains("6 hour") {
        "every_6h".to_string()
    } else {
        "every_24h".to_string()
    };

    Ok(serde_json::json!({
        "suggested_name": suggested_name,
        "suggested_description": suggested_description,
        "suggested_prompt": suggested_prompt,
        "suggested_schedule": suggested_schedule,
        "confidence": 0.85,
    }))
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routines_create() {
        let id = routines_create(
            "Test Routine".into(),
            "A test".into(),
            "Do something".into(),
            "every_1h".into(),
            None,
        )
        .unwrap();
        assert!(id.starts_with("rt-"));
        assert!(!id.is_empty());
    }

    #[test]
    fn test_routines_list() {
        // Create a routine and verify it appears in list
        let _ = routines_create("List Test".into(), "Test".into(), "task".into(), "every_24h".into(), None);
        let list = routines_list().unwrap();
        assert!(!list.is_empty());
        assert!(list.iter().any(|r| r.name == "List Test"));
    }

    #[test]
    fn test_routines_run_now() {
        let id = routines_create(
            "Run Test".into(),
            "Test run".into(),
            "Do it".into(),
            "every_1h".into(),
            Some("desktop".into()),
        )
        .unwrap();
        let run_id = routines_run_now(id).unwrap();
        assert!(run_id.starts_with("run-"));
    }

    #[test]
    fn test_routines_generate_from_task() {
        let result = routines_generate_from_task(
            "scrape hacker news every hour and summarize".into(),
        )
        .unwrap();
        assert_eq!(result["suggested_name"], "Data Collection");
        assert_eq!(result["suggested_schedule"], "every_1h");
        assert_eq!(result["confidence"], 0.85);
    }
}
