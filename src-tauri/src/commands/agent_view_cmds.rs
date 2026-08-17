use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentViewSession {
    pub id: String,
    pub name: String,
    pub surface: String,
    pub status: String,
    pub current_action: String,
    pub progress_pct: f64,
    pub started_at: u64,
    pub last_active_at: u64,
    pub cpu_pct: f64,
    pub memory_mb: f64,
    pub tokens_used: u64,
    pub tasks_completed: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentViewSummary {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub waiting_input: usize,
    pub completed_today: usize,
    pub failed_today: usize,
    pub avg_cpu: f64,
    pub avg_memory: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentViewEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentViewConfig {
    pub enabled: bool,
    pub poll_interval_ms: u64,
    pub max_sessions: usize,
    pub show_completed: bool,
    pub group_by: String,
}

impl Default for AgentViewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval_ms: 5000,
            max_sessions: 20,
            show_completed: true,
            group_by: "status".into(),
        }
    }
}

const MAX_EVENTS: usize = 500;

struct AgentViewState {
    sessions: Vec<AgentViewSession>,
    events: VecDeque<AgentViewEvent>,
    config: AgentViewConfig,
    tick_count: u64,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn init_sessions() -> Vec<AgentViewSession> {
    let now = now_secs();
    vec![
        AgentViewSession {
            id: "s-cli-001".into(),
            name: "CLI Session".into(),
            surface: "cli".into(),
            status: "running".into(),
            current_action: "Refactoring auth module".into(),
            progress_pct: 62.0,
            started_at: now - 1800,
            last_active_at: now - 2,
            cpu_pct: 34.0,
            memory_mb: 128.0,
            tokens_used: 15234,
            tasks_completed: 7,
            error_count: 1,
        },
        AgentViewSession {
            id: "s-desk-002".into(),
            name: "Desktop Chat".into(),
            surface: "desktop".into(),
            status: "running".into(),
            current_action: "Analyzing codebase".into(),
            progress_pct: 45.0,
            started_at: now - 3600,
            last_active_at: now - 5,
            cpu_pct: 52.0,
            memory_mb: 256.0,
            tokens_used: 28901,
            tasks_completed: 12,
            error_count: 0,
        },
        AgentViewSession {
            id: "s-web-003".into(),
            name: "Web Review".into(),
            surface: "web".into(),
            status: "waiting_input".into(),
            current_action: "Waiting for PR review approval".into(),
            progress_pct: 88.0,
            started_at: now - 7200,
            last_active_at: now - 120,
            cpu_pct: 2.0,
            memory_mb: 64.0,
            tokens_used: 4567,
            tasks_completed: 3,
            error_count: 0,
        },
        AgentViewSession {
            id: "s-mob-004".into(),
            name: "Mobile Task".into(),
            surface: "mobile".into(),
            status: "completed".into(),
            current_action: "Bug fix deployed".into(),
            progress_pct: 100.0,
            started_at: now - 14400,
            last_active_at: now - 600,
            cpu_pct: 0.0,
            memory_mb: 32.0,
            tokens_used: 8901,
            tasks_completed: 5,
            error_count: 0,
        },
        AgentViewSession {
            id: "s-bg-005".into(),
            name: "Background Crawler".into(),
            surface: "background".into(),
            status: "running".into(),
            current_action: "Crawling documentation sites".into(),
            progress_pct: 34.0,
            started_at: now - 28800,
            last_active_at: now - 8,
            cpu_pct: 78.0,
            memory_mb: 512.0,
            tokens_used: 0,
            tasks_completed: 23,
            error_count: 2,
        },
        AgentViewSession {
            id: "s-bg-006".into(),
            name: "Code Review Agent".into(),
            surface: "background".into(),
            status: "idle".into(),
            current_action: "Scheduled review pending".into(),
            progress_pct: 0.0,
            started_at: now - 43200,
            last_active_at: now - 3600,
            cpu_pct: 1.0,
            memory_mb: 48.0,
            tokens_used: 0,
            tasks_completed: 18,
            error_count: 0,
        },
        AgentViewSession {
            id: "s-bg-007".into(),
            name: "Deploy Pipeline".into(),
            surface: "background".into(),
            status: "failed".into(),
            current_action: "Deploy failed at test stage".into(),
            progress_pct: 67.0,
            started_at: now - 600,
            last_active_at: now - 300,
            cpu_pct: 0.0,
            memory_mb: 16.0,
            tokens_used: 0,
            tasks_completed: 0,
            error_count: 1,
        },
        AgentViewSession {
            id: "s-bg-008".into(),
            name: "Research Agent".into(),
            surface: "background".into(),
            status: "running".into(),
            current_action: "Researching vector databases".into(),
            progress_pct: 55.0,
            started_at: now - 5400,
            last_active_at: now - 3,
            cpu_pct: 45.0,
            memory_mb: 192.0,
            tokens_used: 32109,
            tasks_completed: 9,
            error_count: 0,
        },
    ]
}

fn init_state() -> AgentViewState {
    let now = now_secs();
    let sessions = init_sessions();
    let mut events = VecDeque::with_capacity(MAX_EVENTS);
    for s in &sessions {
        events.push_back(AgentViewEvent {
            timestamp: now,
            session_id: s.id.clone(),
            kind: "started".into(),
            detail: format!("Session '{}' started", s.name),
        });
        if s.status == "completed" {
            events.push_back(AgentViewEvent {
                timestamp: now + 1,
                session_id: s.id.clone(),
                kind: "completed".into(),
                detail: format!("Session '{}' completed successfully", s.name),
            });
        }
        if s.status == "failed" {
            events.push_back(AgentViewEvent {
                timestamp: now + 1,
                session_id: s.id.clone(),
                kind: "failed".into(),
                detail: format!("Session '{}' failed: {}", s.name, s.current_action),
            });
        }
    }
    AgentViewState {
        sessions,
        events,
        config: AgentViewConfig::default(),
        tick_count: 0,
    }
}

static STATE: LazyLock<Mutex<AgentViewState>> = LazyLock::new(|| Mutex::new(init_state()));

fn compute_summary(state: &AgentViewState) -> AgentViewSummary {
    let total = state.sessions.len();
    let active = state.sessions.iter().filter(|s| s.status == "running").count();
    let waiting = state.sessions.iter().filter(|s| s.status == "waiting_input").count();
    let completed = state.sessions.iter().filter(|s| s.status == "completed").count();
    let failed = state.sessions.iter().filter(|s| s.status == "failed").count();
    let avg_cpu = if total > 0 {
        state.sessions.iter().map(|s| s.cpu_pct).sum::<f64>() / total as f64
    } else {
        0.0
    };
    let avg_memory = if total > 0 {
        state.sessions.iter().map(|s| s.memory_mb).sum::<f64>() / total as f64
    } else {
        0.0
    };
    AgentViewSummary {
        total_sessions: total,
        active_sessions: active,
        waiting_input: waiting,
        completed_today: completed,
        failed_today: failed,
        avg_cpu,
        avg_memory,
    }
}

#[tauri::command]
pub fn agent_view_summary() -> Result<AgentViewSummary, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(compute_summary(&state))
}

#[tauri::command]
pub fn agent_view_list() -> Result<Vec<AgentViewSession>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.sessions.clone())
}

#[tauri::command]
pub fn agent_view_get(id: String) -> Result<AgentViewSession, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.sessions.iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| format!("session '{}' not found", id))
}

#[tauri::command]
pub fn agent_view_events(session_id: String, count: Option<usize>) -> Result<Vec<AgentViewEvent>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let n = count.unwrap_or(50);
    let result: Vec<AgentViewEvent> = state.events.iter()
        .filter(|e| e.session_id == session_id)
        .rev()
        .take(n)
        .cloned()
        .collect();
    Ok(result)
}

#[tauri::command]
pub fn agent_view_pause(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let idx = state.sessions.iter().position(|s| s.id == session_id)
        .ok_or_else(|| format!("session '{}' not found", session_id))?;
    let name = state.sessions[idx].name.clone();
    if state.sessions[idx].status == "running" {
        state.sessions[idx].status = "idle".into();
        state.events.push_back(AgentViewEvent {
            timestamp: now_secs(),
            session_id,
            kind: "input_required".into(),
            detail: format!("Session '{}' paused", name),
        });
        if state.events.len() > MAX_EVENTS {
            state.events.pop_front();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_view_resume(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let idx = state.sessions.iter().position(|s| s.id == session_id)
        .ok_or_else(|| format!("session '{}' not found", session_id))?;
    let name = state.sessions[idx].name.clone();
    if state.sessions[idx].status == "idle" || state.sessions[idx].status == "waiting_input" {
        state.sessions[idx].status = "running".into();
        state.events.push_back(AgentViewEvent {
            timestamp: now_secs(),
            session_id,
            kind: "started".into(),
            detail: format!("Session '{}' resumed", name),
        });
        if state.events.len() > MAX_EVENTS {
            state.events.pop_front();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_view_cancel(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let idx = state.sessions.iter().position(|s| s.id == session_id)
        .ok_or_else(|| format!("session '{}' not found", session_id))?;
    let name = state.sessions[idx].name.clone();
    if state.sessions[idx].status == "running" || state.sessions[idx].status == "waiting_input" || state.sessions[idx].status == "idle" {
        state.sessions[idx].status = "failed".into();
        state.events.push_back(AgentViewEvent {
            timestamp: now_secs(),
            session_id,
            kind: "failed".into(),
            detail: format!("Session '{}' cancelled by user", name),
        });
        if state.events.len() > MAX_EVENTS {
            state.events.pop_front();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_view_config() -> Result<AgentViewConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn agent_view_set_config(config: AgentViewConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.config = config;
    Ok(())
}

fn pseudo_rand(seed: &mut u64, max: f64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let val = (*seed >> 33) as f64;
    let div = (1u64 << 31) as f64;
    (val / div) * max
}

fn pseudo_rand_int(seed: &mut u64, max: u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 33) % max
}

#[tauri::command]
pub fn agent_view_tick() -> Result<AgentViewSummary, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.tick_count += 1;
    let now = now_secs();
    let mut seed = now.wrapping_add(state.tick_count * 12345);
    let mut new_events: Vec<AgentViewEvent> = Vec::new();

    for session in state.sessions.iter_mut() {
        let sid = session.id.clone();
        let sname = session.name.clone();
        match session.status.as_str() {
            "running" => {
                session.progress_pct = (session.progress_pct + pseudo_rand(&mut seed, 5.0)).min(100.0);
                session.cpu_pct = (session.cpu_pct + pseudo_rand(&mut seed, 16.0) - 8.0).clamp(0.0, 100.0);
                session.memory_mb = (session.memory_mb + pseudo_rand(&mut seed, 32.0) - 16.0).max(0.0);
                // 演示模拟器: 本视图为种子/伪随机数据 (progress/cpu/memory 均为漂移),
                // token 统计同样是演示值, 不代表真实用量; 真实 token 统计以 Gateway
                // Usage (LlmResponse.usage.total_tokens) 为准。
                session.tokens_used += pseudo_rand_int(&mut seed, 500);
                session.last_active_at = now;

                if session.progress_pct >= 100.0 {
                    session.status = "completed".into();
                    session.progress_pct = 100.0;
                    new_events.push(AgentViewEvent {
                        timestamp: now,
                        session_id: sid,
                        kind: "completed".into(),
                        detail: format!("Session '{}' completed", sname),
                    });
                } else if pseudo_rand_int(&mut seed, 100) < 3 {
                    session.status = "failed".into();
                    session.error_count += 1;
                    new_events.push(AgentViewEvent {
                        timestamp: now,
                        session_id: sid,
                        kind: "failed".into(),
                        detail: format!("Session '{}' encountered an error", sname),
                    });
                } else if pseudo_rand_int(&mut seed, 100) < 5 {
                    session.status = "waiting_input".into();
                    new_events.push(AgentViewEvent {
                        timestamp: now,
                        session_id: sid,
                        kind: "input_required".into(),
                        detail: format!("Session '{}' requires user input", sname),
                    });
                }
            },
            "idle" => {
                if pseudo_rand_int(&mut seed, 100) < 10 {
                    session.status = "running".into();
                    new_events.push(AgentViewEvent {
                        timestamp: now,
                        session_id: sid,
                        kind: "started".into(),
                        detail: format!("Session '{}' resumed from idle", sname),
                    });
                }
            },
            "waiting_input" => {
                if pseudo_rand_int(&mut seed, 100) < 8 {
                    session.status = "running".into();
                    new_events.push(AgentViewEvent {
                        timestamp: now,
                        session_id: sid,
                        kind: "started".into(),
                        detail: format!("Session '{}' received input, resuming", sname),
                    });
                }
            },
            _ => {},
        }
    }

    for ev in new_events {
        state.events.push_back(ev);
        if state.events.len() > MAX_EVENTS {
            state.events.pop_front();
        }
    }

    Ok(compute_summary(&state))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        if let Ok(mut state) = STATE.lock() {
            *state = init_state();
        }
    }

    #[test]
    fn test_summary_has_eight_sessions() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let summary = agent_view_summary().unwrap();
        assert_eq!(summary.total_sessions, 8);
    }

    #[test]
    fn test_list_returns_sessions() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let list = agent_view_list().unwrap();
        assert_eq!(list.len(), 8);
        assert!(list.iter().any(|s| s.name == "CLI Session"));
        assert!(list.iter().any(|s| s.name == "Desktop Chat"));
        assert!(list.iter().any(|s| s.name == "Web Review"));
        assert!(list.iter().any(|s| s.name == "Research Agent"));
    }

    #[test]
    fn test_get_finds_session() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let session = agent_view_get("s-cli-001".into()).unwrap();
        assert_eq!(session.name, "CLI Session");
        assert_eq!(session.surface, "cli");
        assert_eq!(session.status, "running");
    }

    #[test]
    fn test_get_returns_error_for_missing() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let result = agent_view_get("nonexistent".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_changes_status() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        agent_view_pause("s-cli-001".into()).unwrap();
        let session = agent_view_get("s-cli-001".into()).unwrap();
        assert_eq!(session.status, "idle");
    }

    #[test]
    fn test_pause_nonexistent_returns_error() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let result = agent_view_pause("nonexistent".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_changes_status() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        agent_view_pause("s-cli-001".into()).unwrap();
        agent_view_resume("s-cli-001".into()).unwrap();
        let session = agent_view_get("s-cli-001".into()).unwrap();
        assert_eq!(session.status, "running");
    }

    #[test]
    fn test_cancel_changes_status() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        agent_view_cancel("s-cli-001".into()).unwrap();
        let session = agent_view_get("s-cli-001".into()).unwrap();
        assert_eq!(session.status, "failed");
    }

    #[test]
    fn test_events_returns_log() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let events = agent_view_events("s-cli-001".into(), Some(10)).unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].session_id, "s-cli-001");
    }

    #[test]
    fn test_config_default() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let config = agent_view_config().unwrap();
        assert!(config.enabled);
        assert_eq!(config.poll_interval_ms, 5000);
        assert_eq!(config.group_by, "status");
    }

    #[test]
    fn test_set_config() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let new_config = AgentViewConfig {
            enabled: false,
            poll_interval_ms: 10000,
            max_sessions: 50,
            show_completed: false,
            group_by: "surface".into(),
        };
        agent_view_set_config(new_config).unwrap();
        let config = agent_view_config().unwrap();
        assert!(!config.enabled);
        assert_eq!(config.poll_interval_ms, 10000);
        assert_eq!(config.group_by, "surface");
    }

    #[test]
    fn test_tick_advances_progress() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let before = agent_view_get("s-desk-002".into()).unwrap();
        let progress_before = before.progress_pct;
        agent_view_tick().unwrap();
        let after = agent_view_get("s-desk-002".into()).unwrap();
        assert!(after.progress_pct >= progress_before, "tick 应推进或保持进度");
        assert!(after.last_active_at >= before.last_active_at);
    }
}
