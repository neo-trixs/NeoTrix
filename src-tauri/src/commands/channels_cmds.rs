use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "channelType")]
    pub channel_type: String,
    #[serde(rename = "webhookUrl")]
    pub webhook_url: String,
    pub enabled: bool,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub from: String,
    pub content: String,
    pub timestamp: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSession {
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "linkedAt")]
    pub linked_at: u64,
    pub active: bool,
}

// ── State ───────────────────────────────────────────────────────────────

struct ChannelsState {
    channels: Vec<ChannelConfig>,
    messages: VecDeque<ChannelMessage>,
    session_links: Vec<ChannelSession>,
    slack_config: Option<(String, String, String)>,
    msg_counter: u64,
}

impl ChannelsState {
    fn new() -> Self {
        Self {
            channels: Vec::new(),
            messages: VecDeque::with_capacity(200),
            session_links: Vec::new(),
            slack_config: None,
            msg_counter: 0,
        }
    }
}

static CHANNELS: LazyLock<Mutex<ChannelsState>> = LazyLock::new(|| Mutex::new(ChannelsState::new()));

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn generate_id(prefix: &str) -> String {
    let ts = now_secs();
    let rand = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() % 9999)
        .unwrap_or(0);
    format!("{}-{}-{}", prefix, ts, rand)
}

fn push_message(state: &mut ChannelsState, channel_id: String, from: String, content: String, kind: String) {
    state.msg_counter += 1;
    state.messages.push_back(ChannelMessage {
        id: format!("msg-{}-{}", now_secs(), state.msg_counter),
        channel_id,
        from,
        content,
        timestamp: now_secs(),
        kind,
    });
    while state.messages.len() > 200 {
        state.messages.pop_front();
    }
}

// ── Channel Commands ────────────────────────────────────────────────────

#[tauri::command]
pub fn channels_list() -> Result<Vec<ChannelConfig>, String> {
    let state = CHANNELS.lock().map_err(|e| e.to_string())?;
    Ok(state.channels.clone())
}

#[tauri::command]
pub fn channels_add(channel_type: String, name: String, webhook_url: String) -> Result<String, String> {
    let id = generate_id("ch");
    let secret = format!("sec-{}-{}", now_secs(), id);
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    state.channels.push(ChannelConfig {
        id: id.clone(),
        name,
        channel_type,
        webhook_url,
        enabled: true,
        secret,
    });
    Ok(id)
}

#[tauri::command]
pub fn channels_remove(id: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let before = state.channels.len();
    state.channels.retain(|c| c.id != id);
    state.session_links.retain(|l| l.channel_id != id);
    if state.channels.len() == before {
        return Err(format!("Channel '{}' not found", id));
    }
    Ok(())
}

#[tauri::command]
pub fn channels_enable(id: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let channel = state.channels.iter_mut().find(|c| c.id == id);
    match channel {
        Some(c) => { c.enabled = true; Ok(()) }
        None => Err(format!("Channel '{}' not found", id)),
    }
}

#[tauri::command]
pub fn channels_disable(id: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let channel = state.channels.iter_mut().find(|c| c.id == id);
    match channel {
        Some(c) => { c.enabled = false; Ok(()) }
        None => Err(format!("Channel '{}' not found", id)),
    }
}

#[tauri::command]
pub fn channels_send(channel_id: String, content: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let channel = state.channels.iter().find(|c| c.id == channel_id).cloned();
    match channel {
        Some(ref ch) => {
            let msg = format!("[{}] Sending message to {} ({}): {}",
                ch.channel_type, ch.name, channel_id, content);
            log::info!("{}", msg);
            push_message(&mut state, channel_id, "system".into(), content, "outgoing".into());
            Ok(())
        }
        None => Err(format!("Channel '{}' not found", channel_id)),
    }
}

#[tauri::command]
pub fn channels_receive(channel_id: String) -> Result<Vec<ChannelMessage>, String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let msgs: Vec<ChannelMessage> = state.messages.iter()
        .filter(|m| m.channel_id == channel_id)
        .cloned()
        .collect();
    state.messages.retain(|m| m.channel_id != channel_id);
    Ok(msgs)
}

#[tauri::command]
pub fn channels_link_session(channel_id: String, session_id: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    if !state.channels.iter().any(|c| c.id == channel_id) {
        return Err(format!("Channel '{}' not found", channel_id));
    }
    if let Some(existing) = state.session_links.iter_mut().find(|l| l.channel_id == channel_id) {
        existing.session_id = session_id;
        existing.linked_at = now_secs();
        existing.active = true;
    } else {
        state.session_links.push(ChannelSession {
            channel_id,
            session_id,
            linked_at: now_secs(),
            active: true,
        });
    }
    Ok(())
}

#[tauri::command]
pub fn channels_unlink_session(channel_id: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let before = state.session_links.len();
    state.session_links.retain(|l| l.channel_id != channel_id);
    if state.session_links.len() == before {
        return Err(format!("No session link for channel '{}'", channel_id));
    }
    Ok(())
}

#[tauri::command]
pub fn channels_history(count: usize) -> Result<Vec<ChannelMessage>, String> {
    let state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let msgs: Vec<ChannelMessage> = state.messages.iter().rev().take(count).cloned().collect();
    Ok(msgs)
}

// ── Slack Commands ──────────────────────────────────────────────────────

#[tauri::command]
pub fn slack_config() -> Result<serde_json::Value, String> {
    let state = CHANNELS.lock().map_err(|e| e.to_string())?;
    match &state.slack_config {
        Some((workspace, channel, _token)) => Ok(json!({
            "configured": true,
            "enabled": state.channels.iter().any(|c| c.channel_type == "slack" && c.enabled),
            "workspace": workspace,
            "channel": channel,
        })),
        None => Ok(json!({
            "configured": false,
            "enabled": false,
        })),
    }
}

#[tauri::command]
pub fn slack_configure(workspace: String, channel: String, token: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    state.slack_config = Some((workspace.clone(), channel.clone(), token.clone()));

    let exists = state.channels.iter().any(|c| c.channel_type == "slack");
    if !exists {
        let id = generate_id("slack");
        state.channels.push(ChannelConfig {
            id,
            name: format!("Slack #{}/{} ", channel, workspace),
            channel_type: "slack".into(),
            webhook_url: format!("https://{}.slack.com/", workspace),
            enabled: true,
            secret: token,
        });
    } else {
        for c in state.channels.iter_mut().filter(|c| c.channel_type == "slack") {
            c.webhook_url = format!("https://{}.slack.com/", workspace);
            c.enabled = true;
            c.name = format!("Slack #{}/{} ", channel, workspace);
        }
    }
    log::info!("Slack configured: {} #{} ", workspace, channel);
    Ok(())
}

#[tauri::command]
pub fn slack_send(message: String) -> Result<(), String> {
    let mut state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let slack_ch = state.channels.iter().find(|c| c.channel_type == "slack").cloned();
    match slack_ch {
        Some(ref ch) => {
            log::info!("[Slack] Sending to {}: {}", ch.name, message);
            push_message(&mut state, ch.id.clone(), "slack".into(), message, "outgoing".into());
            Ok(())
        }
        None => Err("Slack is not configured. Call slack_configure first.".to_string()),
    }
}

#[tauri::command]
pub fn slack_status() -> Result<serde_json::Value, String> {
    let state = CHANNELS.lock().map_err(|e| e.to_string())?;
    let configured = state.slack_config.is_some();
    let enabled = state.channels.iter().any(|c| c.channel_type == "slack" && c.enabled);
    Ok(json!({
        "configured": configured,
        "enabled": enabled,
    }))
}


// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channels_add_and_list() {
        let id = channels_add("webhook".into(), "Test Channel".into(), "https://hook.example.com".into()).unwrap();
        assert!(id.starts_with("ch-"));
        let list = channels_list().unwrap();
        assert!(list.iter().any(|c| c.id == id));
    }

    #[test]
    fn test_channels_list_returns_added_channels() {
        let before = channels_list().unwrap().len();
        channels_add("discord".into(), "Discord Alerts".into(), "https://discord.example.com".into()).unwrap();
        let after = channels_list().unwrap().len();
        assert!(after > before);
    }

    #[test]
    fn test_channels_enable_disable() {
        let id = channels_add("telegram".into(), "Telegram Bot".into(), "https://t.me/bot".into()).unwrap();
        channels_disable(id.clone()).unwrap();
        let list = channels_list().unwrap();
        let ch = list.iter().find(|c| c.id == id).unwrap();
        assert!(!ch.enabled);

        channels_enable(id.clone()).unwrap();
        let list = channels_list().unwrap();
        let ch = list.iter().find(|c| c.id == id).unwrap();
        assert!(ch.enabled);
    }

    #[test]
    fn test_slack_configure_and_status() {
        let status_before = slack_status().unwrap();
        assert!(!status_before["configured"].as_bool().unwrap());

        slack_configure("neotrix".into(), "general".into(), "xoxb-token".into()).unwrap();

        let status_after = slack_status().unwrap();
        assert!(status_after["configured"].as_bool().unwrap());

        slack_send("Hello from tests".into()).unwrap();
        let hist = channels_history(10).unwrap();
        assert!(hist.iter().any(|m| m.content.contains("Hello from tests")));
    }
}
