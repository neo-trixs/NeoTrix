//! Ext Plugin — 扩展：远程桥接、频道、协作、通知

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static EXT_STATE: LazyLock<Mutex<HashMap<String, serde_json::Value>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("connections".into(), serde_json::json!({}));
    m.insert("channels".into(), serde_json::json!({}));
    m.insert(
        "cowork".into(),
        serde_json::json!({
            "active": false,
            "session_id": null,
            "participants": [],
        }),
    );
    m.insert("notifications".into(), serde_json::json!([]));
    Mutex::new(m)
});

pub struct ExtPlugin;

#[async_trait]
impl DomainPlugin for ExtPlugin {
    fn name(&self) -> &str {
        "ext"
    }
    fn description(&self) -> &str {
        "扩展：远程桥接、频道、协作、通知"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "remote_connect",
            "remote_disconnect",
            "channel_send",
            "channel_list",
            "cowork_start",
            "cowork_stop",
            "notify",
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
            "remote_connect" => {
                let host = args.get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "remote_connect requires 'host' argument".into(),
                        recoverable: false,
                    })?
                    .to_string();
                let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let device_id = args
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();
                let protocol = args
                    .get("protocol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tcp")
                    .to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let connections = state
                    .get_mut("connections")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Connections state corrupted".into(),
                        recoverable: true,
                    })?;
                let conn_id = format!("{}:{}", host, port);
                connections.insert(
                    conn_id.clone(),
                    serde_json::json!({
                        "host": host,
                        "port": port,
                        "device_id": device_id,
                        "protocol": protocol,
                        "connected_at": now,
                        "status": "connected",
                    }),
                );
                Ok(serde_json::json!({
                    "ok": true,
                    "connection_id": conn_id,
                    "connected_at": now,
                }))
            }
            "remote_disconnect" => {
                let host = args.get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "remote_disconnect requires 'host' argument".into(),
                        recoverable: false,
                    })?
                    .to_string();
                let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let connections = state
                    .get_mut("connections")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Connections state corrupted".into(),
                        recoverable: true,
                    })?;
                let conn_id = format!("{}:{}", host, port);
                let removed = connections.remove(&conn_id);
                Ok(serde_json::json!({
                    "ok": true,
                    "connection_id": conn_id,
                    "was_connected": removed.is_some(),
                }))
            }
            "channel_send" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();
                let message = args
                    .get("message")
                    .cloned()
                    .unwrap_or(serde_json::json!(null));
                let sender = args
                    .get("sender")
                    .and_then(|v| v.as_str())
                    .unwrap_or("local")
                    .to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let channels = state
                    .get_mut("channels")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Channels state corrupted".into(),
                        recoverable: true,
                    })?;
                let msg = serde_json::json!({
                    "sender": sender,
                    "message": message,
                    "timestamp": now,
                });
                let messages = channels
                    .entry(channel.clone())
                    .or_insert_with(|| serde_json::json!([]))
                    .as_array_mut()
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Channel messages corrupted".into(),
                        recoverable: true,
                    })?;
                messages.push(msg.clone());
                Ok(serde_json::json!({
                    "ok": true,
                    "channel": channel,
                    "message": msg,
                }))
            }
            "channel_list" => {
                let state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let channels = state
                    .get("channels")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Channels state corrupted".into(),
                        recoverable: true,
                    })?;
                let channel_list: Vec<serde_json::Value> = channels
                    .iter()
                    .map(|(name, msgs)| {
                        let count = msgs.as_array().map(|a| a.len()).unwrap_or(0);
                        serde_json::json!({
                            "name": name,
                            "message_count": count,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "channels": channel_list,
                    "count": channel_list.len(),
                }))
            }
            "cowork_start" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();
                let participants: Vec<String> = args
                    .get("participants")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                state.insert(
                    "cowork".into(),
                    serde_json::json!({
                        "active": true,
                        "session_id": session_id,
                        "participants": participants,
                        "started_at": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    }),
                );
                Ok(serde_json::json!({ "ok": true, "session_id": session_id }))
            }
            "cowork_stop" => {
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                state.insert(
                    "cowork".into(),
                    serde_json::json!({
                        "active": false,
                        "session_id": null,
                        "participants": [],
                    }),
                );
                Ok(serde_json::json!({ "ok": true }))
            }
            "notify" => {
                let title = args
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Notification")
                    .to_string();
                let body = args
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let level = args
                    .get("level")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info")
                    .to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mut state = EXT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let notifications = state
                    .get_mut("notifications")
                    .and_then(|v| v.as_array_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Notifications state corrupted".into(),
                        recoverable: true,
                    })?;
                let notification = serde_json::json!({
                    "title": title,
                    "body": body,
                    "level": level,
                    "timestamp": now,
                    "read": false,
                });
                notifications.push(notification.clone());
                Ok(serde_json::json!({ "ok": true, "notification": notification }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
