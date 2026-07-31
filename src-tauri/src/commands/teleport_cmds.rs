use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ── Teleport Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportSession {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub session_data: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportCode {
    pub code: String,
    pub session_id: String,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportConfig {
    pub enabled: bool,
    pub max_sessions: usize,
    pub default_ttl_secs: u64,
}

impl Default for TeleportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_sessions: 20,
            default_ttl_secs: 300,
        }
    }
}

// ── Agent Team Types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeam {
    pub id: String,
    pub name: String,
    pub description: String,
    pub strategy: String,
    pub created_at: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeamMember {
    pub id: String,
    pub team_id: String,
    pub role: String,
    pub task: String,
    pub status: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeamMessage {
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeamResult {
    pub team_id: String,
    pub overall_status: String,
    pub member_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub duration_ms: u64,
}

// ── State ──────────────────────────────────────────────────────────────

pub struct TeleportState {
    pub teleport_sessions: Vec<TeleportSession>,
    pub teleport_codes: Vec<TeleportCode>,
    pub config: TeleportConfig,
    pub teams: Vec<AgentTeam>,
    pub team_members: Vec<AgentTeamMember>,
    pub team_messages: VecDeque<AgentTeamMessage>,
    session_counter: u64,
    team_counter: u64,
    member_counter: u64,
}

impl TeleportState {
    fn new() -> Self {
        Self {
            teleport_sessions: Vec::with_capacity(20),
            teleport_codes: Vec::new(),
            config: TeleportConfig::default(),
            teams: Vec::new(),
            team_members: Vec::new(),
            team_messages: VecDeque::with_capacity(500),
            session_counter: 0,
            team_counter: 0,
            member_counter: 0,
        }
    }
}

static STATE: LazyLock<Mutex<TeleportState>> = LazyLock::new(|| Mutex::new(TeleportState::new()));

const MAX_TELEPORT_SESSIONS: usize = 20;
const MAX_TEAM_MESSAGES: usize = 500;

// ── Helpers ────────────────────────────────────────────────────────────

fn short_uid(counter: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{:x}{:04x}", now % 0xffffff, counter % 0xffff)
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn generate_code() -> String {
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut code = String::with_capacity(6);
    let mut seed = now;
    for _ in 0..6 {
        let idx = (seed % 36) as usize;
        code.push(chars[idx] as char);
        seed /= 36;
    }
    code
}

// ── PART 1: Teleport ──────────────────────────────────────────────────

#[tauri::command]
pub fn teleport_create(source: String, session_data: String) -> Result<TeleportCode, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if !state.config.enabled {
        return Err("teleport disabled".to_string());
    }
    if state.teleport_sessions.len() >= state.config.max_sessions {
        return Err("max teleport sessions reached".to_string());
    }
    state.session_counter += 1;
    let session_id = format!("tp-{}", short_uid(state.session_counter));
    let now = now_secs();
    let ttl = state.config.default_ttl_secs;
    let code_str = generate_code();

    state.teleport_sessions.push(TeleportSession {
        id: session_id.clone(),
        source: source.clone(),
        destination: String::new(),
        session_data: session_data.clone(),
        created_at: now,
        expires_at: now + ttl,
        claimed: false,
    });
    state.teleport_codes.push(TeleportCode {
        code: code_str.clone(),
        session_id: session_id.clone(),
        expires_at: now + ttl,
        used: false,
    });
    Ok(TeleportCode {
        code: code_str,
        session_id,
        expires_at: now + ttl,
        used: false,
    })
}

#[tauri::command]
pub fn teleport_claim(code: String, destination: String) -> Result<TeleportSession, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let now = now_secs();
    let session_id = {
        let code_entry = state.teleport_codes.iter_mut()
            .find(|c| c.code == code)
            .ok_or_else(|| "code not found".to_string())?;
        if code_entry.used {
            return Err("code already used".to_string());
        }
        if now > code_entry.expires_at {
            return Err("code expired".to_string());
        }
        code_entry.used = true;
        code_entry.session_id.clone()
    };
    let session = state.teleport_sessions.iter_mut()
        .find(|s| s.id == session_id)
        .ok_or_else(|| "session not found".to_string())?;
    session.destination = destination;
    session.claimed = true;
    Ok(session.clone())
}

#[tauri::command]
pub fn teleport_list() -> Result<Vec<TeleportSession>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let mut sessions: Vec<TeleportSession> = state.teleport_sessions.iter()
        .filter(|s| !s.claimed)
        .cloned()
        .collect();
    sessions.truncate(20);
    Ok(sessions)
}

#[tauri::command]
pub fn teleport_config() -> Result<TeleportConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn teleport_set_config(config: TeleportConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn teleport_revoke(session_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let pos = state.teleport_sessions.iter().position(|s| s.id == session_id)
        .ok_or_else(|| format!("session {} not found", session_id))?;
    state.teleport_sessions.remove(pos);
    state.teleport_codes.retain(|c| c.session_id != session_id);
    Ok(())
}

// ── PART 2: Agent Teams ────────────────────────────────────────────────

#[tauri::command]
pub fn agent_team_create(name: String, description: String, strategy: String) -> Result<String, String> {
    if strategy != "sequential" && strategy != "parallel" && strategy != "leader" {
        return Err("strategy must be one of: sequential, parallel, leader".to_string());
    }
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.team_counter += 1;
    let id = format!("team-{}", short_uid(state.team_counter));
    state.teams.push(AgentTeam {
        id: id.clone(),
        name,
        description,
        strategy,
        created_at: now_secs(),
        status: "idle".to_string(),
    });
    Ok(id)
}

#[tauri::command]
pub fn agent_team_list() -> Result<Vec<AgentTeam>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.teams.clone())
}

#[tauri::command]
pub fn agent_team_get(id: String) -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let team = state.teams.iter()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("team {} not found", id))?;
    let members: Vec<&AgentTeamMember> = state.team_members.iter()
        .filter(|m| m.team_id == id)
        .collect();
    Ok(serde_json::json!({
        "team": team,
        "members": members,
        "member_count": members.len(),
    }))
}

#[tauri::command]
pub fn agent_team_add_member(team_id: String, role: String, task: String) -> Result<String, String> {
    if role != "lead" && role != "worker" && role != "reviewer" && role != "observer" {
        return Err("role must be one of: lead, worker, reviewer, observer".to_string());
    }
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if !state.teams.iter().any(|t| t.id == team_id) {
        return Err(format!("team {} not found", team_id));
    }
    state.member_counter += 1;
    let id = format!("mbr-{}", short_uid(state.member_counter));
    state.team_members.push(AgentTeamMember {
        id: id.clone(),
        team_id,
        role,
        task,
        status: "pending".to_string(),
        result: None,
    });
    Ok(id)
}

#[tauri::command]
pub fn agent_team_remove_member(team_id: String, member_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let pos = state.team_members.iter().position(|m| m.id == member_id && m.team_id == team_id)
        .ok_or_else(|| format!("member {} not found in team {}", member_id, team_id))?;
    state.team_members.remove(pos);
    Ok(())
}

#[tauri::command]
pub fn agent_team_start(team_id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let team = state.teams.iter_mut()
        .find(|t| t.id == team_id)
        .ok_or_else(|| format!("team {} not found", team_id))?;
    team.status = "running".to_string();
    for m in state.team_members.iter_mut().filter(|m| m.team_id == team_id) {
        if m.status == "pending" {
            m.status = "in_progress".to_string();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_team_complete_member(member_id: String, result: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let member = state.team_members.iter_mut()
        .find(|m| m.id == member_id)
        .ok_or_else(|| format!("member {} not found", member_id))?;
    member.status = "completed".to_string();
    member.result = Some(result);

    let team_id = member.team_id.clone();

    let any_failed = state.team_members.iter()
        .filter(|m| m.team_id == team_id)
        .any(|m| m.status == "failed");
    let all_done = state.team_members.iter()
        .filter(|m| m.team_id == team_id)
        .all(|m| m.status == "completed" || m.status == "failed");
    if all_done {
        if let Some(team) = state.teams.iter_mut().find(|t| t.id == team_id) {
            team.status = if any_failed { "failed".to_string() } else { "completed".to_string() };
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_team_fail_member(member_id: String, error: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let member = state.team_members.iter_mut()
        .find(|m| m.id == member_id)
        .ok_or_else(|| format!("member {} not found", member_id))?;
    member.status = "failed".to_string();
    member.result = Some(error);

    let team_id = member.team_id.clone();

    let all_done = state.team_members.iter()
        .filter(|m| m.team_id == team_id)
        .all(|m| m.status == "completed" || m.status == "failed");
    if all_done {
        if let Some(team) = state.teams.iter_mut().find(|t| t.id == team_id) {
            team.status = "failed".to_string();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn agent_team_status(team_id: String) -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let team = state.teams.iter()
        .find(|t| t.id == team_id)
        .ok_or_else(|| format!("team {} not found", team_id))?;
    let members: Vec<&AgentTeamMember> = state.team_members.iter()
        .filter(|m| m.team_id == team_id)
        .collect();
    let total = members.len();
    let completed = members.iter().filter(|m| m.status == "completed").count();
    let failed = members.iter().filter(|m| m.status == "failed").count();
    let in_progress = members.iter().filter(|m| m.status == "in_progress").count();
    let pending = members.iter().filter(|m| m.status == "pending").count();
    Ok(serde_json::json!({
        "team_id": team_id,
        "status": team.status,
        "total": total,
        "completed": completed,
        "failed": failed,
        "in_progress": in_progress,
        "pending": pending,
    }))
}

#[tauri::command]
pub fn agent_team_messages(team_id: String) -> Result<Vec<AgentTeamMessage>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let msgs: Vec<AgentTeamMessage> = state.team_messages.iter()
        .filter(|m| m.from == team_id || m.to == team_id || m.to == "*")
        .cloned()
        .collect();
    Ok(msgs.into_iter().take(200).collect())
}

#[tauri::command]
pub fn agent_team_send_message(team_id: String, to: String, content: String, kind: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if !state.teams.iter().any(|t| t.id == team_id) {
        return Err(format!("team {} not found", team_id));
    }
    if state.team_messages.len() >= MAX_TEAM_MESSAGES {
        state.team_messages.pop_front();
    }
    state.team_messages.push_back(AgentTeamMessage {
        from: team_id,
        to,
        content,
        timestamp: now_secs(),
        kind,
    });
    Ok(())
}

#[tauri::command]
pub fn agent_team_result(team_id: String) -> Result<AgentTeamResult, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let team = state.teams.iter()
        .find(|t| t.id == team_id)
        .ok_or_else(|| format!("team {} not found", team_id))?;
    let start = team.created_at;
    let members: Vec<&AgentTeamMember> = state.team_members.iter()
        .filter(|m| m.team_id == team_id)
        .collect();
    let total = members.len();
    let completed = members.iter().filter(|m| m.status == "completed").count();
    let failed = members.iter().filter(|m| m.status == "failed").count();
    let duration_ms = (now_secs().saturating_sub(start)) * 1000;
    Ok(AgentTeamResult {
        team_id: team_id.clone(),
        overall_status: team.status.clone(),
        member_count: total,
        completed_count: completed,
        failed_count: failed,
        duration_ms,
    })
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup_teleport() {
        if let Ok(mut state) = STATE.lock() {
            state.teleport_sessions.clear();
            state.teleport_codes.clear();
            state.config = TeleportConfig::default();
        }
    }

    fn cleanup_teams() {
        if let Ok(mut state) = STATE.lock() {
            state.teams.clear();
            state.team_members.clear();
            state.team_messages.clear();
        }
    }

    #[test]
    fn test_teleport_create_code() {
        cleanup_teleport();
        let code = teleport_create("cli".into(), "session-data".into()).unwrap();
        assert_eq!(code.code.len(), 6);
        assert!(!code.used);
        assert!(code.expires_at > 0);
        assert!(code.session_id.starts_with("tp-"));
    }

    #[test]
    fn test_teleport_claim() {
        cleanup_teleport();
        let code = teleport_create("desktop".into(), "claim-test".into()).unwrap();
        let session = teleport_claim(code.code.clone(), "mobile".into()).unwrap();
        assert!(session.claimed);
        assert_eq!(session.destination, "mobile");
        assert_eq!(session.source, "desktop");
        assert_eq!(session.session_data, "claim-test");

        let err = teleport_claim(code.code.clone(), "web".into());
        assert!(err.is_err());
    }

    #[test]
    fn test_agent_team_create() {
        cleanup_teams();
        let id = agent_team_create("test-team".into(), "a test team".into(), "parallel".into()).unwrap();
        assert!(id.starts_with("team-"));

        let err = agent_team_create("bad".into(), "".into(), "invalid".into());
        assert!(err.is_err());
    }

    #[test]
    fn test_agent_team_add_member() {
        cleanup_teams();
        let team_id = agent_team_create("dev-team".into(), "dev".into(), "sequential".into()).unwrap();
        let m1 = agent_team_add_member(team_id.clone(), "lead".into(), "design".into()).unwrap();
        assert!(m1.starts_with("mbr-"));
        let m2 = agent_team_add_member(team_id.clone(), "worker".into(), "implement".into()).unwrap();
        assert!(m2.starts_with("mbr-"));

        let err = agent_team_add_member(team_id.clone(), "invalid".into(), "x".into());
        assert!(err.is_err());

        let err = agent_team_add_member("nonexistent".into(), "worker".into(), "x".into());
        assert!(err.is_err());
    }

    #[test]
    fn test_agent_team_start() {
        cleanup_teams();
        let team_id = agent_team_create("start-team".into(), "start test".into(), "parallel".into()).unwrap();
        let m1 = agent_team_add_member(team_id.clone(), "worker".into(), "task-1".into()).unwrap();
        let m2 = agent_team_add_member(team_id.clone(), "worker".into(), "task-2".into()).unwrap();
        assert!(agent_team_start(team_id.clone()).is_ok());

        let status = agent_team_status(team_id.clone()).unwrap();
        assert_eq!(status["status"], "running");
        assert_eq!(status["in_progress"], 2);

        assert!(agent_team_complete_member(m1.clone(), "done".into()).is_ok());
        assert!(agent_team_fail_member(m2.clone(), "error".into()).is_ok());

        let result = agent_team_result(team_id.clone()).unwrap();
        assert_eq!(result.completed_count, 1);
        assert_eq!(result.failed_count, 1);

        let status2 = agent_team_status(team_id).unwrap();
        assert_eq!(status2["completed"], 1);
        assert_eq!(status2["failed"], 1);
    }
}
