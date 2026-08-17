use crate::anthropic::types::*;
use keyring::Entry;
use reqwest::Client;
use serde_json::Value;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.get_password().map_err(|e| format!("no api key: {}", e))
}

pub fn save_api_key(key: &str) -> Result<(), String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.set_password(key).map_err(|e| format!("failed to save key: {}", e))
}

pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new("novachat", "anthropic_api_key")
        .map_err(|e| format!("keyring error: {}", e))?;
    entry.delete_credential().map_err(|e| format!("failed to delete key: {}", e))
}

pub fn has_api_key() -> bool {
    get_api_key().map(|k| !k.is_empty()).unwrap_or(false)
}

#[allow(dead_code)]
pub async fn send_message_blocking(
    api_key: &str,
    model: &str,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
) -> Result<String, String> {
    let client = Client::new();

    let request = MessageRequest {
        model: model.to_string(),
        max_tokens,
        stream: false,
        messages,
        system: system.filter(|s| !s.is_empty()),
    };

    let body = serde_json::to_value(&request).map_err(|e| format!("serialize error: {}", e))?;

    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status.as_u16(), error_text));
    }

    let res: Value = response.json().await.map_err(|e| format!("parse error: {}", e))?;
    let content = res["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c["text"].as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}
