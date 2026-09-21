//! Security Plugin — 安全：扫描、权限、隐身、企业合规

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static SECURITY_STATE: LazyLock<Mutex<HashMap<String, serde_json::Value>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("audit_log".into(), serde_json::json!([]));
    m.insert(
        "policies".into(),
        serde_json::json!({
            "require_confirmation": true,
            "quarantine_on_threat": true,
            "auto_scan": false,
        }),
    );
    m.insert("quarantine".into(), serde_json::json!([]));
    Mutex::new(m)
});

fn security_add_audit_event(event_type: &str, detail: &str) {
    if let Ok(mut state) = SECURITY_STATE.lock() {
        if let Some(log) = state.get_mut("audit_log").and_then(|v| v.as_array_mut()) {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            log.push(serde_json::json!({
                "timestamp": ts,
                "type": event_type,
                "detail": detail,
            }));
        }
    }
}

pub struct SecurityPlugin;

#[async_trait]
impl DomainPlugin for SecurityPlugin {
    fn name(&self) -> &str {
        "security"
    }
    fn description(&self) -> &str {
        "安全：扫描、权限、隐身、企业合规"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "scan",
            "audit",
            "quarantine",
            "permission_request",
            "permission_respond",
            "stealth_status",
            "audit_log",
            "policy_list",
            "policy_set",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "scan" => {
                let target = args
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("system")
                    .to_string();
                security_add_audit_event(
                    "scan",
                    &format!("Security scan triggered for: {}", target),
                );
                // Basic scan: check for common sensitive files
                let mut findings = vec![];
                let sensitive_patterns = vec![
                    ".env",
                    "credentials.json",
                    "secrets.yml",
                    ".ssh/id_rsa",
                    ".aws/credentials",
                    ".npmrc",
                ];
                let home = dirs::home_dir().unwrap_or_default();
                for pattern in &sensitive_patterns {
                    let path = home.join(pattern);
                    if path.exists() {
                        findings.push(serde_json::json!({
                            "severity": "warning",
                            "type": "sensitive_file",
                            "path": path.to_string_lossy(),
                            "message": format!("Sensitive file found: {}", pattern),
                        }));
                    }
                }
                Ok(serde_json::json!({
                    "ok": true,
                    "target": target,
                    "findings_count": findings.len(),
                    "findings": findings,
                    "clean": findings.is_empty(),
                }))
            }
            "audit" => {
                let detail = args
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general audit")
                    .to_string();
                security_add_audit_event("audit", &detail);
                let state = SECURITY_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let log = state
                    .get("audit_log")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                Ok(serde_json::json!({
                    "ok": true,
                    "detail": detail,
                    "total_events": log.as_array().map(|a| a.len()).unwrap_or(0),
                }))
            }
            "quarantine" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let reason = args
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("manual quarantine")
                    .to_string();
                if path.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 path 参数".into(),
                        recoverable: true,
                    });
                }
                security_add_audit_event(
                    "quarantine",
                    &format!("Quarantined: {} ({})", path, reason),
                );
                let mut state = SECURITY_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                if let Some(q) = state.get_mut("quarantine").and_then(|v| v.as_array_mut()) {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    q.push(serde_json::json!({
                        "path": path,
                        "reason": reason,
                        "timestamp": ts,
                    }));
                }
                Ok(serde_json::json!({ "ok": true, "path": path }))
            }
            "permission_request" | "permission_respond" => {
                security_add_audit_event(action, &args.to_string());
                Ok(serde_json::json!({ "ok": true, "action": action }))
            }
            "stealth_status" => Ok(serde_json::json!({
                "stealth_enabled": false,
                "proxy_active": false,
                "fingerprint_masked": false,
            })),
            "audit_log" => {
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                let state = SECURITY_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let log = state
                    .get("audit_log")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                let entries = log
                    .as_array()
                    .map(|a| {
                        let start = if a.len() > limit { a.len() - limit } else { 0 };
                        a[start..].to_vec()
                    })
                    .unwrap_or_default();
                Ok(serde_json::json!({
                    "entries": entries,
                    "count": entries.len(),
                }))
            }
            "policy_list" => {
                let state = SECURITY_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let policies = state
                    .get("policies")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                Ok(serde_json::json!({ "policies": policies }))
            }
            "policy_set" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let value = args
                    .get("value")
                    .cloned()
                    .unwrap_or(serde_json::json!(null));
                if key.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 key 参数".into(),
                        recoverable: true,
                    });
                }
                security_add_audit_event("policy_set", &format!("{} = {}", key, value));
                let mut state = SECURITY_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                if let Some(policies) = state.get_mut("policies").and_then(|v| v.as_object_mut()) {
                    policies.insert(key.clone(), value.clone());
                }
                Ok(serde_json::json!({ "ok": true, "key": key, "value": value }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
