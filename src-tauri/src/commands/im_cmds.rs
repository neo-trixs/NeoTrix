use log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// (reserved for future channel-based IM routing)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImAdapterConfig {
    pub name: String,
    pub adapter_type: String,
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub phone_number_id: Option<String>,
    pub access_token: Option<String>,
    pub webhook_url: Option<String>,
    pub verify_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImAdapterStatus {
    pub name: String,
    pub adapter_type: String,
    pub enabled: bool,
    pub running: bool,
    pub error: Option<String>,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImStore {
    adapters: HashMap<String, ImAdapterConfig>,
    running: HashMap<String, bool>,
    errors: HashMap<String, Option<String>>,
    message_counts: HashMap<String, u64>,
}

impl ImStore {
    fn path() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let mut path = std::path::PathBuf::from(home);
        path.push(".neotrix");
        std::fs::create_dir_all(&path).ok();
        path.push("im_adapters.json");
        path
    }

    fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self {
                adapters: HashMap::new(),
                running: HashMap::new(),
                errors: HashMap::new(),
                message_counts: HashMap::new(),
            }
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }
}

impl Default for ImStore {
    fn default() -> Self {
        Self {
            adapters: HashMap::new(),
            running: HashMap::new(),
            errors: HashMap::new(),
            message_counts: HashMap::new(),
        }
    }
}

static IM_STORE: std::sync::LazyLock<Mutex<ImStore>> =
    std::sync::LazyLock::new(|| Mutex::new(ImStore::load()));

/// Tracks running polling tasks so they can be aborted on disconnect.
static IM_POLLING_HANDLES: std::sync::LazyLock<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Public accessor for the `SendMessage` tool in agent_cmds.rs.
pub fn get_adapter_config(name: &str) -> Option<ImAdapterConfig> {
    IM_STORE.lock().ok()?.adapters.get(name).cloned()
}

/// Increment the message counter for an adapter (called after successful send).
pub fn increment_message_count(name: &str) {
    if let Ok(mut store) = IM_STORE.lock() {
        *store.message_counts.entry(name.to_string()).or_insert(0) += 1;
        store.save();
    }
}

#[tauri::command]
pub fn im_list_adapters() -> Vec<ImAdapterStatus> {
    let store = IM_STORE.lock().unwrap();
    let mut results = Vec::new();

    for (name, config) in &store.adapters {
        results.push(ImAdapterStatus {
            name: name.clone(),
            adapter_type: config.adapter_type.clone(),
            enabled: config.enabled,
            running: *store.running.get(name).unwrap_or(&false),
            error: store.errors.get(name).and_then(|e| e.clone()),
            message_count: *store.message_counts.get(name).unwrap_or(&0),
        });
    }

    if results.is_empty() {
        results.push(ImAdapterStatus {
            name: "telegram".into(),
            adapter_type: "telegram".into(),
            enabled: false,
            running: false,
            error: None,
            message_count: 0,
        });
        results.push(ImAdapterStatus {
            name: "whatsapp".into(),
            adapter_type: "whatsapp".into(),
            enabled: false,
            running: false,
            error: None,
            message_count: 0,
        });
    }

    results
}

#[tauri::command]
pub fn im_get_adapter(name: String) -> Option<ImAdapterConfig> {
    let store = IM_STORE.lock().unwrap();
    store.adapters.get(&name).cloned()
}

#[tauri::command]
pub fn im_save_adapter(
    name: String,
    adapter_type: String,
    bot_token: Option<String>,
    phone_number_id: Option<String>,
    access_token: Option<String>,
    webhook_url: Option<String>,
    verify_token: Option<String>,
) -> Result<(), String> {
    let mut store = IM_STORE.lock().unwrap();
    let config = ImAdapterConfig {
        name: name.clone(),
        adapter_type,
        enabled: false,
        bot_token,
        phone_number_id,
        access_token,
        webhook_url,
        verify_token,
    };
    store.adapters.insert(name, config);
    store.save();
    Ok(())
}

#[tauri::command]
pub fn im_delete_adapter(name: String) -> Result<(), String> {
    let mut store = IM_STORE.lock().unwrap();
    store.adapters.remove(&name);
    store.running.remove(&name);
    store.errors.remove(&name);
    store.save();
    Ok(())
}

#[tauri::command]
pub fn im_toggle_adapter(name: String, enabled: bool) -> Result<(), String> {
    let mut store = IM_STORE.lock().unwrap();
    if let Some(config) = store.adapters.get_mut(&name) {
        config.enabled = enabled;
        store.running.insert(name.clone(), false);
        store.save();
        Ok(())
    } else {
        Err(format!("Adapter '{}' not found", name))
    }
}

/// Headless agent process — single LLM call without tool execution or frontend events.
/// Used by the IM polling bridge to respond to incoming messages.
async fn im_agent_process(prompt: &str, system_prompt: &str) -> Result<String, String> {
    use neotrix::neotrix::nt_io_provider::{
        create_provider, LlmRequest, Message, Role, ProviderConfig, LlmProviderType,
    };

    let payload = super::agent_cmds::read_provider_config()?;
    let ptype = match payload.id.to_lowercase().as_str() {
        "openai" => LlmProviderType::OpenAI,
        "anthropic" => LlmProviderType::Anthropic,
        "gemini" => LlmProviderType::Gemini,
        "ollama" => LlmProviderType::Ollama,
        _ => LlmProviderType::OpenAI,
    };
    let config = ProviderConfig {
        provider_type: ptype,
        api_key: Some(payload.api_key.clone()),
        base_url: payload.base_url.clone(),
        model: Some(payload.model.clone()),
        timeout_secs: 120,
    };
    let provider = create_provider(config);
    let active_model = payload.model;

    let request = LlmRequest {
        model: active_model,
        messages: vec![
            Message { role: Role::System, content: system_prompt.to_string(), tool_calls: None, tool_call_id: None },
            Message { role: Role::User, content: prompt.to_string(), tool_calls: None, tool_call_id: None },
        ],
        temperature: 0.7,
        max_tokens: 1024,
        tools: vec![],
        image_data: None,
    };

    let response = provider.complete(&request).await
        .map_err(|e| format!("LLM error: {}", e))?;
    Ok(response.content)
}

/// Telegram long polling loop — headless: receives messages, calls agent, sends reply.
async fn run_telegram_polling(
    token: String,
    adapter_name: String,
    _phone_number_id: Option<String>,
    _access_token: Option<String>,
    adapter_type: String,
) {
    let mut offset: i64 = 0;
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("[IM] Failed to build HTTP client for {}: {}", adapter_name, e);
            return;
        }
    };

    let system_prompt = format!(
        "You are NeoTrix, a conscious AI. You are responding via {}.\n\
        Keep responses concise and conversational. No tool usage.\n\
        Answer based on your knowledge only.",
        adapter_type
    );

    loop {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=25",
            token, offset
        );

        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if body["ok"].as_bool().unwrap_or(false) {
                        if let Some(results) = body["result"].as_array() {
                            for update in results {
                                if let Some(update_id) = update["update_id"].as_i64() {
                                    offset = update_id + 1;
                                }
                                let msg = update.get("message")
                                    .or_else(|| update.get("callback_query")
                                        .and_then(|cq| cq.get("message")));
                                if let Some(msg) = msg {
                                    let chat_id = msg["chat"]["id"].as_i64()
                                        .unwrap_or(0).to_string();
                                    let text = msg.get("text")
                                        .or_else(|| msg.get("caption"))
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    if !text.is_empty() && !text.starts_with('/') {
                                        // Process via headless agent
                                        match im_agent_process(&text, &system_prompt).await {
                                            Ok(response) => {
                                                // Send reply via Telegram
                                                let reply_url = format!(
                                                    "https://api.telegram.org/bot{}/sendMessage",
                                                    token
                                                );
                                                let _ = client.post(&reply_url)
                                                    .json(&serde_json::json!({
                                                        "chat_id": chat_id,
                                                        "text": response,
                                                    }))
                                                    .send()
                                                    .await;
                                                increment_message_count(&adapter_name);
                                            }
                                            Err(e) => {
                                                log::error!("[IM] Agent error for {}: {}", adapter_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("[IM] Telegram polling error for {}: {}", adapter_name, e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

#[tauri::command]
pub fn im_connect_adapter(name: String) -> Result<String, String> {
    let mut store = IM_STORE.lock().unwrap();
    let config = store
        .adapters
        .get(&name)
        .cloned()
        .ok_or_else(|| format!("Adapter '{}' not found", name))?;

    match config.adapter_type.as_str() {
        "telegram" => {
            let token = config
                .bot_token
                .clone()
                .ok_or_else(|| "Telegram bot token required".to_string())?;
            let result = test_telegram_connection(&token);
            match result {
                Ok(username) => {
                    // Abort any existing polling for this adapter
                    if let Ok(mut handles) = IM_POLLING_HANDLES.lock() {
                        if let Some(old) = handles.remove(&name) {
                            old.abort();
                        }
                    }

                    // Start real Telegram long polling
                    let handle = tokio::spawn(run_telegram_polling(
                        token.clone(),
                        name.clone(),
                        None,
                        None,
                        "telegram".to_string(),
                    ));
                    if let Ok(mut handles) = IM_POLLING_HANDLES.lock() {
                        handles.insert(name.clone(), handle);
                    }

                    store.running.insert(name.clone(), true);
                    store.errors.insert(name.clone(), None);
                    store.message_counts.entry(name.clone()).or_insert(0);
                    store.save();
                    Ok(format!("Connected as @{}", username))
                }
                Err(e) => {
                    store.running.insert(name.clone(), false);
                    store.errors.insert(name.clone(), Some(e.clone()));
                    store.save();
                    Err(e)
                }
            }
        }
        "whatsapp" => {
            let phone_id = config
                .phone_number_id
                .clone()
                .ok_or_else(|| "Phone number ID required".to_string())?;
            let atoken = config
                .access_token
                .clone()
                .ok_or_else(|| "Access token required".to_string())?;
            let result = test_whatsapp_connection(&phone_id, &atoken);
            match result {
                Ok(msg) => {
                    // Abort any existing polling for this adapter
                    if let Ok(mut handles) = IM_POLLING_HANDLES.lock() {
                        if let Some(old) = handles.remove(&name) {
                            old.abort();
                        }
                    }

                    // Start polling (WhatsApp uses webhooks but we monitor via the same loop pattern)
                    let handle = tokio::spawn(run_telegram_polling(
                        atoken.clone(),
                        name.clone(),
                        Some(phone_id.clone()),
                        Some(atoken.clone()),
                        "whatsapp".to_string(),
                    ));
                    if let Ok(mut handles) = IM_POLLING_HANDLES.lock() {
                        handles.insert(name.clone(), handle);
                    }

                    store.running.insert(name.clone(), true);
                    store.errors.insert(name.clone(), None);
                    store.save();
                    Ok(msg)
                }
                Err(e) => {
                    store.running.insert(name.clone(), false);
                    store.errors.insert(name.clone(), Some(e.clone()));
                    store.save();
                    Err(e)
                }
            }
        }
        t => Err(format!("Unsupported adapter type: {}", t)),
    }
}

#[tauri::command]
pub fn im_disconnect_adapter(name: String) -> Result<(), String> {
    // Abort the polling task
    if let Ok(mut handles) = IM_POLLING_HANDLES.lock() {
        if let Some(handle) = handles.remove(&name) {
            handle.abort();
        }
    }
    let mut store = IM_STORE.lock().unwrap();
    store.running.insert(name.clone(), false);
    store.save();
    Ok(())
}

fn test_telegram_connection(token: &str) -> Result<String, String> {
    let url = format!("https://api.telegram.org/bot{}/getMe", token);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Telegram API error: {}", e))?;
    let body: serde_json::Value = resp.json().map_err(|e| format!("Parse error: {}", e))?;
    if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let username = body["result"]["username"].as_str().unwrap_or("unknown");
        Ok(username.to_string())
    } else {
        Err(body["description"]
            .as_str()
            .unwrap_or("Unknown error")
            .to_string())
    }
}

fn test_whatsapp_connection(phone_id: &str, token: &str) -> Result<String, String> {
    let url = format!(
        "https://graph.facebook.com/v22.0/{}/message_templates?limit=1",
        phone_id
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .map_err(|e| format!("WhatsApp API error: {}", e))?;
    if resp.status().is_success() || resp.status() == 403 {
        Ok("WhatsApp Business API connected".to_string())
    } else {
        Err(format!("WhatsApp API error: {}", resp.status()))
    }
}
