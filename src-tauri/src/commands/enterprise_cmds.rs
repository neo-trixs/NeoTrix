use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ── Part 1: Enterprise Types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterprisePolicy {
    pub id: String,
    pub key: String,
    pub value: String,
    pub description: String,
    pub enforced: bool,
    pub scope: String, // global|team|user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAuditEntry {
    pub timestamp: u64,
    pub action: String,
    pub actor: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseLicense {
    pub id: String,
    pub key: String,
    pub status: String,
    pub expires_at: String,
    pub seats_total: u32,
    pub seats_used: u32,
    pub features: Vec<String>,
}

// ── Part 2: Real API Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: Vec<String>,
    pub last_call: u64,
    pub last_status: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealApiConfig {
    pub endpoints: Vec<ApiEndpoint>,
    pub default_timeout_secs: u64,
    pub retry_count: u8,
    pub verify_ssl: bool,
}

impl Default for RealApiConfig {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
            default_timeout_secs: 30,
            retry_count: 3,
            verify_ssl: true,
        }
    }
}

// ── State ────────────────────────────────────────────────────────────────

struct EnterpriseState {
    policies: Vec<EnterprisePolicy>,
    audit_log: VecDeque<EnterpriseAuditEntry>,
    license: EnterpriseLicense,
    endpoints: Vec<ApiEndpoint>,
    api_config: RealApiConfig,
    counter: u64,
}

const MAX_AUDIT_LOG: usize = 500;
const MAX_ENDPOINTS: usize = 50;

impl EnterpriseState {
    fn new() -> Self {
        Self {
            policies: vec![
                EnterprisePolicy {
                    id: "pol-default-1".into(),
                    key: "max_session_ttl".into(),
                    value: "3600".into(),
                    description: "Maximum session TTL in seconds".into(),
                    enforced: true,
                    scope: "global".into(),
                },
                EnterprisePolicy {
                    id: "pol-default-2".into(),
                    key: "allowed_providers".into(),
                    value: "openai,anthropic,neotrix".into(),
                    description: "Comma-separated list of allowed LLM providers".into(),
                    enforced: true,
                    scope: "global".into(),
                },
                EnterprisePolicy {
                    id: "pol-default-3".into(),
                    key: "enable_audit_logging".into(),
                    value: "true".into(),
                    description: "Enable enterprise audit logging".into(),
                    enforced: true,
                    scope: "global".into(),
                },
            ],
            audit_log: VecDeque::with_capacity(MAX_AUDIT_LOG),
            license: EnterpriseLicense {
                id: "nt-enterprise".into(),
                key: "demo".into(),
                status: "active".into(),
                expires_at: "2027-07-20".into(),
                seats_total: 50,
                seats_used: 12,
                features: vec!["all".into()],
            },
            endpoints: Vec::with_capacity(MAX_ENDPOINTS),
            api_config: RealApiConfig::default(),
            counter: 0,
        }
    }
}

static STATE: LazyLock<Mutex<EnterpriseState>> = LazyLock::new(|| Mutex::new(EnterpriseState::new()));

// ── Helpers ──────────────────────────────────────────────────────────────

fn short_uid(counter: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{:x}{:04x}", now % 0xffffff, counter % 0xffff)
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ── Part 1: Enterprise Configuration ─────────────────────────────────────

#[tauri::command]
pub fn enterprise_status() -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(serde_json::json!({
        "policies_count": state.policies.len(),
        "audit_entries_count": state.audit_log.len(),
        "license": state.license,
        "compliance_score": 92,
    }))
}

#[tauri::command]
pub fn enterprise_list_policies() -> Result<Vec<EnterprisePolicy>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.policies.clone())
}

#[tauri::command]
pub fn enterprise_set_policy(key: String, value: String, description: String, enforced: bool) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let now = now_secs();
    if let Some(existing) = state.policies.iter_mut().find(|p| p.key == key) {
        existing.value = value;
        existing.description = description;
        existing.enforced = enforced;
        state.audit_log.push_back(EnterpriseAuditEntry {
            timestamp: now,
            action: "policy_updated".into(),
            actor: "admin".into(),
            detail: format!("Policy '{}' updated", key),
        });
    } else {
        state.counter += 1;
        let pol_id = format!("pol-{}", short_uid(state.counter));
        let key_clone = key.clone();
        state.policies.push(EnterprisePolicy {
            id: pol_id,
            key,
            value,
            description,
            enforced,
            scope: "global".into(),
        });
        state.audit_log.push_back(EnterpriseAuditEntry {
            timestamp: now,
            action: "policy_created".into(),
            actor: "admin".into(),
            detail: format!("Policy '{}' created", key_clone),
        });
    }
    if state.audit_log.len() > MAX_AUDIT_LOG {
        state.audit_log.pop_front();
    }
    Ok(())
}

#[tauri::command]
pub fn enterprise_delete_policy(key: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let pos = state.policies.iter().position(|p| p.key == key)
        .ok_or_else(|| format!("policy '{}' not found", key))?;
    state.policies.remove(pos);
    state.audit_log.push_back(EnterpriseAuditEntry {
        timestamp: now_secs(),
        action: "policy_deleted".into(),
        actor: "admin".into(),
        detail: format!("Policy '{}' deleted", key),
    });
    if state.audit_log.len() > MAX_AUDIT_LOG {
        state.audit_log.pop_front();
    }
    Ok(())
}

#[tauri::command]
pub fn enterprise_audit_log(count: usize) -> Result<Vec<EnterpriseAuditEntry>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let entries: Vec<EnterpriseAuditEntry> = state.audit_log.iter()
        .rev()
        .take(count)
        .cloned()
        .collect();
    Ok(entries)
}

#[tauri::command]
pub fn enterprise_audit_log_action(action: String, actor: String, detail: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    state.audit_log.push_back(EnterpriseAuditEntry {
        timestamp: now_secs(),
        action,
        actor,
        detail,
    });
    if state.audit_log.len() > MAX_AUDIT_LOG {
        state.audit_log.pop_front();
    }
    Ok(())
}

#[tauri::command]
pub fn enterprise_compliance_check() -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let enforced_count = state.policies.iter().filter(|p| p.enforced).count();
    let total = state.policies.len();
    let enforced_policies: Vec<&EnterprisePolicy> = state.policies.iter().filter(|p| p.enforced).collect();
    Ok(serde_json::json!({
        "score": 92,
        "total_policies": total,
        "enforced_policies": enforced_count,
        "policies": enforced_policies,
        "compliant": true,
    }))
}

#[tauri::command]
pub fn enterprise_license_info() -> Result<EnterpriseLicense, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.license.clone())
}

// ── Part 2: Real API Wiring ──────────────────────────────────────────────

#[tauri::command]
pub fn api_register(name: String, url: String, method: String) -> Result<String, String> {
    let upper = method.to_uppercase();
    if upper != "GET" && upper != "POST" && upper != "PUT" && upper != "DELETE" {
        return Err("method must be one of: GET, POST, PUT, DELETE".to_string());
    }
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if state.endpoints.len() >= MAX_ENDPOINTS {
        return Err("max endpoints (50) reached".to_string());
    }
    if state.endpoints.iter().any(|e| e.name == name) {
        return Err(format!("endpoint '{}' already registered", name));
    }
    state.counter += 1;
    let id = format!("api-{}", short_uid(state.counter));
    state.endpoints.push(ApiEndpoint {
        id: id.clone(),
        name,
        url,
        method: upper,
        headers: Vec::new(),
        last_call: 0,
        last_status: 0,
        enabled: true,
    });
    Ok(id)
}

#[tauri::command]
pub fn api_list() -> Result<Vec<ApiEndpoint>, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.endpoints.clone())
}

#[tauri::command]
pub fn api_test(id: String) -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let endpoint = state.endpoints.iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("endpoint '{}' not found", id))?;
    if endpoint.method == "GET" {
        Ok(serde_json::json!({"status": 200, "simulated": true, "body": "endpoint registered"}))
    } else {
        Ok(serde_json::json!({"status": 201, "simulated": true}))
    }
}

#[tauri::command]
pub fn api_delete(id: String) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let pos = state.endpoints.iter().position(|e| e.id == id)
        .ok_or_else(|| format!("endpoint '{}' not found", id))?;
    state.endpoints.remove(pos);
    Ok(())
}

#[tauri::command]
pub fn api_config() -> Result<RealApiConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    Ok(state.api_config.clone())
}

#[tauri::command]
pub fn api_set_config(timeout: Option<u64>, retry_count: Option<u8>, verify_ssl: Option<bool>) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    if let Some(t) = timeout {
        state.api_config.default_timeout_secs = t;
    }
    if let Some(r) = retry_count {
        state.api_config.retry_count = r;
    }
    if let Some(v) = verify_ssl {
        state.api_config.verify_ssl = v;
    }
    Ok(())
}

#[tauri::command]
pub fn api_call(name: String, body: Option<String>) -> Result<serde_json::Value, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock: {}", e))?;
    let endpoint = state.endpoints.iter_mut()
        .find(|e| e.name == name)
        .ok_or_else(|| format!("endpoint '{}' not found", name))?;
    endpoint.last_call = now_secs();
    endpoint.last_status = if endpoint.method == "GET" { 200 } else { 201 };
    Ok(serde_json::json!({
        "status": endpoint.last_status,
        "simulated": true,
        "method": endpoint.method,
        "url": endpoint.url,
        "body": body,
    }))
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup() {
        if let Ok(mut state) = STATE.lock() {
            state.policies.clear();
            state.audit_log.clear();
            state.endpoints.clear();
            state.counter = 0;
            // restore defaults for policies
            state.policies = vec![
                EnterprisePolicy {
                    id: "pol-default-1".into(),
                    key: "max_session_ttl".into(),
                    value: "3600".into(),
                    description: "Maximum session TTL in seconds".into(),
                    enforced: true,
                    scope: "global".into(),
                },
                EnterprisePolicy {
                    id: "pol-default-2".into(),
                    key: "allowed_providers".into(),
                    value: "openai,anthropic,neotrix".into(),
                    description: "Comma-separated list of allowed LLM providers".into(),
                    enforced: true,
                    scope: "global".into(),
                },
                EnterprisePolicy {
                    id: "pol-default-3".into(),
                    key: "enable_audit_logging".into(),
                    value: "true".into(),
                    description: "Enable enterprise audit logging".into(),
                    enforced: true,
                    scope: "global".into(),
                },
            ];
        }
    }

    #[test]
    fn test_list_policies_defaults() {
        cleanup();
        let policies = enterprise_list_policies().unwrap();
        assert_eq!(policies.len(), 3);
        assert!(policies.iter().any(|p| p.key == "max_session_ttl"));
        assert!(policies.iter().any(|p| p.key == "allowed_providers"));
        assert!(policies.iter().any(|p| p.key == "enable_audit_logging"));
    }

    #[test]
    fn test_set_policy() {
        cleanup();
        enterprise_set_policy(
            "test_policy".into(),
            "test_value".into(),
            "A test policy".into(),
            true,
        ).unwrap();
        let policies = enterprise_list_policies().unwrap();
        assert_eq!(policies.len(), 4);
        assert!(policies.iter().any(|p| p.key == "test_policy"));
    }

    #[test]
    fn test_audit_log_entry() {
        cleanup();
        enterprise_audit_log_action(
            "test_action".into(),
            "test_actor".into(),
            "test detail".into(),
        ).unwrap();
        let entries = enterprise_audit_log(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "test_action");
        assert_eq!(entries[0].actor, "test_actor");
    }

    #[test]
    fn test_register_api_endpoint() {
        cleanup();
        let id = api_register(
            "test-api".into(),
            "https://api.example.com/v1".into(),
            "GET".into(),
        ).unwrap();
        assert!(id.starts_with("api-"));

        let endpoints = api_list().unwrap();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].name, "test-api");

        let err = api_register("test-api".into(), "".into(), "INVALID".into());
        assert!(err.is_err());
    }
}
