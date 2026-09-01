//! V2 Tauri 命令 — NeoTrix V2 架构的 Tauri 后端
//!
//! 当前只保留 unified + PTY 命令，其他旧命令模块暂不编译（依赖 neotrix crate）。

#![allow(dead_code)]

pub mod pty;
pub mod unified;
pub mod domain_cmd;

// ========== Types (shared) ==========

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigPayload {
    pub id: String,
    pub name: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub learning_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub mode: String,
    pub port: u16,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self { running: false, mode: "off".into(), port: 11080 }
    }
}

// ========== Stub re-exports for old commands ==========

// 旧命令模块的类型 stubs，保持兼容性
pub type NeoTrixError = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffBlock {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoOld {
    pub id: String,
    pub name: String,
    pub created: i64,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestOld {
    pub id: String,
    pub action: String,
    pub target: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub running: bool,
    pub current_task: Option<String>,
}

pub fn parse_git_diff(input: &str) -> Vec<DiffBlock> {
    let mut blocks = Vec::new();
    for line in input.lines() {
        if let Some(rest) = line.strip_prefix('+') {
            blocks.push(DiffBlock { r#type: "added".into(), content: rest.into() });
        } else if let Some(rest) = line.strip_prefix('-') {
            blocks.push(DiffBlock { r#type: "removed".into(), content: rest.into() });
        } else if !line.starts_with("diff ") && !line.starts_with("index ")
            && !line.starts_with("---") && !line.starts_with("+++") && !line.starts_with("@@")
            && !line.starts_with("\\")
        {
            blocks.push(DiffBlock { r#type: "unchanged".into(), content: line.into() });
        }
    }
    blocks
}

pub fn session_create(name: String) -> SessionInfoOld {
    SessionInfoOld { id: format!("s-{}", &uuid::Uuid::new_v4().to_string()[..8]), name, created: chrono::Utc::now().timestamp(), message_count: 0 }
}

pub fn session_list() -> Vec<SessionInfoOld> {
    vec![SessionInfoOld { id: "default".into(), name: "默认会话".into(), created: 0, message_count: 0 }]
}

pub fn payload_to_provider_config(payload: &ProviderConfigPayload) -> ProviderConfigPayload { payload.clone() }

// cmd_* stubs
use std::sync::LazyLock;
use std::collections::HashMap;

static CMD_SESSIONS: LazyLock<std::sync::Mutex<HashMap<String, SessionInfoOld>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

pub fn cmd_session_create(name: String) -> Result<String, String> {
    let id = format!("s-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    CMD_SESSIONS.lock().unwrap().insert(id.clone(), session_create(name));
    Ok(id)
}

pub fn cmd_session_list() -> Result<Vec<SessionInfoOld>, String> {
    Ok(CMD_SESSIONS.lock().unwrap().values().cloned().collect())
}

pub fn cmd_session_switch(id: String) -> Result<(), String> {
    if CMD_SESSIONS.lock().unwrap().contains_key(&id) { Ok(()) } else { Err("not found".into()) }
}

pub fn cmd_session_delete(id: String) -> Result<(), String> {
    CMD_SESSIONS.lock().unwrap().remove(&id); Ok(())
}

static CMD_AGENT: LazyLock<std::sync::Mutex<AgentStatus>> =
    LazyLock::new(|| std::sync::Mutex::new(AgentStatus { running: false, current_task: None }));

pub fn cmd_agent_start(task: String) -> Result<(), String> {
    let mut s = CMD_AGENT.lock().unwrap(); s.running = true; s.current_task = Some(task); Ok(())
}

pub fn cmd_agent_stop() -> Result<(), String> {
    let mut s = CMD_AGENT.lock().unwrap(); s.running = false; s.current_task = None; Ok(())
}

pub fn cmd_agent_status() -> Result<AgentStatus, String> { Ok(CMD_AGENT.lock().unwrap().clone()) }

static CMD_PERM: LazyLock<std::sync::Mutex<HashMap<String, PermissionRequestOld>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

pub fn cmd_permission_request(action: String, target: String) -> Result<PermissionRequestOld, String> {
    let req = PermissionRequestOld { id: format!("perm-{}", &uuid::Uuid::new_v4().to_string()[..8]), action, target, timestamp: chrono::Utc::now().timestamp() };
    CMD_PERM.lock().unwrap().insert(req.id.clone(), req.clone()); Ok(req)
}

pub fn cmd_permission_approve(id: String) -> Result<(), String> {
    CMD_PERM.lock().unwrap().remove(&id).map(|_| ()).ok_or("not found".into())
}

pub fn cmd_permission_deny(id: String) -> Result<(), String> { cmd_permission_approve(id) }
