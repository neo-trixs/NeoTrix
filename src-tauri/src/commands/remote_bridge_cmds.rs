use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Data Types ──

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub last_seen: u64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: String,
    pub payload: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemotePairing {
    pub code: String,
    pub expires_at: u64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteSession {
    pub paired_devices: Vec<RemoteDevice>,
    pub messages: VecDeque<RemoteMessage>,
    pub pending_pairing: Option<RemotePairing>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteBridgeConfig {
    pub port: u16,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteBridgeState {
    pub session: RemoteSession,
    pub config: RemoteBridgeConfig,
    pub history: VecDeque<RemoteMessage>,
}

impl Default for RemoteBridgeState {
    fn default() -> Self {
        Self {
            session: RemoteSession {
                paired_devices: Vec::new(),
                messages: VecDeque::new(),
                pending_pairing: None,
            },
            config: RemoteBridgeConfig { port: 9876, enabled: true },
            history: VecDeque::with_capacity(100),
        }
    }
}

// ── State ──

static BRIDGE_STATE: LazyLock<Mutex<RemoteBridgeState>> = LazyLock::new(|| {
    Mutex::new(RemoteBridgeState::default())
});

// ── Helpers ──

fn generate_id(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", prefix, now)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_pairing_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let mut code = String::with_capacity(6);
    let mut s = seed;
    for _ in 0..6 {
        let idx = (s % chars.len() as u128) as usize;
        code.push(chars[idx]);
        s /= chars.len() as u128;
    }
    code
}

fn enforce_max_devices(state: &mut RemoteBridgeState) {
    if state.session.paired_devices.len() > 10 {
        state.session.paired_devices.sort_by(|a, b| a.last_seen.cmp(&b.last_seen));
        state.session.paired_devices.drain(0..state.session.paired_devices.len().saturating_sub(10));
    }
}

fn push_history(state: &mut RemoteBridgeState, msg: &RemoteMessage) {
    if state.history.len() >= 100 {
        state.history.pop_front();
    }
    state.history.push_back(msg.clone());
}

// ── Commands ──

#[tauri::command]
pub fn remote_bridge_status() -> Result<serde_json::Value, String> {
    let state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    Ok(serde_json::json!({
        "device_count": state.session.paired_devices.len(),
        "devices": state.session.paired_devices,
        "config": state.config,
        "has_pending_pairing": state.session.pending_pairing.is_some(),
    }))
}

#[tauri::command]
pub fn remote_bridge_pair() -> Result<String, String> {
    let mut state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    let code = generate_pairing_code();
    let expires_at = now_secs() + 300;
    state.session.pending_pairing = Some(RemotePairing {
        code: code.clone(),
        expires_at,
        status: "pending".into(),
    });
    Ok(format!("{{\"code\":\"{}\",\"expires_at\":{}}}", code, expires_at))
}

#[tauri::command]
pub fn remote_bridge_connect(device_name: String, device_type: String, code: String) -> Result<String, String> {
    let mut state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let pairing = state.session.pending_pairing.as_ref()
        .ok_or_else(|| "No pending pairing".to_string())?;

    if pairing.status != "pending" {
        return Err("Pairing already used or expired".into());
    }
    if pairing.code != code {
        return Err("Invalid pairing code".into());
    }
    let now = now_secs();
    if now > pairing.expires_at {
        state.session.pending_pairing = None;
        return Err("Pairing code expired".into());
    }

    let device_id = generate_id("dev");
    let device = RemoteDevice {
        id: device_id.clone(),
        name: device_name,
        device_type,
        last_seen: now,
        status: "online".into(),
    };

    if state.session.paired_devices.iter().any(|d| d.id == device.id) {
        return Err("Device already paired".into());
    }

    state.session.paired_devices.push(device);
    state.session.pending_pairing = None;
    enforce_max_devices(&mut state);

    Ok(format!("Connected with device_id: {}", device_id))
}

#[tauri::command]
pub fn remote_bridge_disconnect(device_id: String) -> Result<(), String> {
    let mut state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;
    let len_before = state.session.paired_devices.len();
    state.session.paired_devices.retain(|d| d.id != device_id);
    if state.session.paired_devices.len() == len_before {
        return Err(format!("Device {} not found", device_id));
    }
    Ok(())
}

#[tauri::command]
pub fn remote_bridge_send(device_id: String, kind: String, payload: String) -> Result<(), String> {
    let mut state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    if !state.session.paired_devices.iter().any(|d| d.id == device_id) {
        return Err(format!("Device {} not found", device_id));
    }

    let msg = RemoteMessage {
        id: generate_id("msg"),
        from: "neotrix".into(),
        to: device_id,
        kind,
        payload,
        timestamp: now_secs(),
    };

    state.session.messages.push_back(msg.clone());
    push_history(&mut state, &msg);

    Ok(())
}

#[tauri::command]
pub fn remote_bridge_broadcast(kind: String, payload: String) -> Result<(), String> {
    let mut state = BRIDGE_STATE.lock().map_err(|e| format!("State lock failed: {}", e))?;

    let device_ids: Vec<String> = state.session.paired_devices.iter().map(|d| d.id.clone()).collect();
    if device_ids.is_empty() {
        return Err("No paired devices".into());
    }

    let now = now_secs();
    for device_id in &device_ids {
        let msg = RemoteMessage {
            id: generate_id("msg"),
            from: "neotrix".into(),
            to: device_id.clone(),
            kind: kind.clone(),
            payload: payload.clone(),
            timestamp: now,
        };
        state.session.messages.push_back(msg.clone());
        push_history(&mut state, &msg);
    }

    Ok(())
}

#[tauri::command]
pub fn remote_bridge_poll(device_id: String) -> Vec<RemoteMessage> {
    let mut state = match BRIDGE_STATE.lock() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut pending: Vec<RemoteMessage> = Vec::new();
    let mut remaining: VecDeque<RemoteMessage> = VecDeque::new();

    for msg in state.session.messages.drain(..) {
        if msg.to == device_id {
            pending.push(msg);
        } else {
            remaining.push_back(msg);
        }
    }

    state.session.messages = remaining;
    pending
}

#[tauri::command]
pub fn remote_bridge_devices() -> Vec<RemoteDevice> {
    let state = match BRIDGE_STATE.lock() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    state.session.paired_devices.clone()
}

#[tauri::command]
pub fn remote_bridge_history(count: usize) -> Vec<RemoteMessage> {
    let state = match BRIDGE_STATE.lock() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let effective = count.min(state.history.len());
    state.history.iter().rev().take(effective).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_state() {
        let mut state = BRIDGE_STATE.lock().unwrap();
        *state = RemoteBridgeState::default();
    }

    fn pair_and_connect(name: &str, device_type: &str) -> String {
        let pair_resp = remote_bridge_pair().unwrap();
        let code: String = serde_json::from_str::<serde_json::Value>(&pair_resp)
            .unwrap()["code"].as_str().unwrap().to_string();
        let connect_resp = remote_bridge_connect(name.into(), device_type.into(), code).unwrap();
        connect_resp.split("device_id: ").nth(1).unwrap().to_string()
    }

    /// Single sequential test — parallel tests would race on LazyLock<Mutex<RemoteBridgeState>>.
    #[test]
    fn test_remote_bridge_full_flow() {
        // 1. Pair code generation
        reset_state();
        let result = remote_bridge_pair().expect("pair should succeed");
        assert!(result.contains("\"code\":\""));
        assert!(result.contains("\"expires_at\":"));

        {
            let state = BRIDGE_STATE.lock().unwrap();
            let pairing = state.session.pending_pairing.as_ref().unwrap();
            assert_eq!(pairing.code.len(), 6);
            assert_eq!(pairing.status, "pending");
        }

        // 2. Connect validates code
        let pair_resp2 = remote_bridge_pair().unwrap();
        let code: String = serde_json::from_str::<serde_json::Value>(&pair_resp2)
            .unwrap()["code"].as_str().unwrap().to_string();

        let connect_result = remote_bridge_connect("iPhone".into(), "phone".into(), code);
        assert!(connect_result.is_ok());
        assert!(connect_result.unwrap().contains("Connected with device_id: dev-"));

        {
            let state = BRIDGE_STATE.lock().unwrap();
            assert_eq!(state.session.paired_devices.len(), 1);
            assert_eq!(state.session.paired_devices[0].name, "iPhone");
            assert_eq!(state.session.paired_devices[0].device_type, "phone");
        }

        // Wrong code should fail
        let bad_result = remote_bridge_connect("Wrong".into(), "phone".into(), "XXXXXX".into());
        assert!(bad_result.is_err());

        // 3. Disconnect removes device
        reset_state();
        let dev_id = pair_and_connect("iPad", "tablet");

        assert!(remote_bridge_disconnect(dev_id.clone()).is_ok());
        assert!(remote_bridge_disconnect(dev_id).is_err());

        {
            let state = BRIDGE_STATE.lock().unwrap();
            assert_eq!(state.session.paired_devices.len(), 0);
        }

        // 4. Send/receive roundtrip
        reset_state();
        let dev_id = pair_and_connect("MacBook", "laptop");

        assert!(remote_bridge_send(dev_id.clone(), "notification".into(), "Hello".into()).is_ok());

        let msgs = remote_bridge_poll(dev_id.clone());
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].payload, "Hello");
        assert_eq!(msgs[0].kind, "notification");

        // Second poll should be empty (messages cleared)
        let msgs2 = remote_bridge_poll(dev_id);
        assert!(msgs2.is_empty());

        // 5. History bounded
        reset_state();
        let dev_id = pair_and_connect("Server", "server");

        for i in 0..5 {
            remote_bridge_send(dev_id.clone(), "test".into(), format!("msg-{}", i)).unwrap();
        }

        // Poll to clear from messages queue
        remote_bridge_poll(dev_id);

        let h = remote_bridge_history(3);
        assert_eq!(h.len(), 3, "history should be bounded by count param");
    }
}
