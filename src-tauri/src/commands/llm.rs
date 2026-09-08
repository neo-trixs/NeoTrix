use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LlmProvider {
    pub name: String,
    pub provider_type: String,
    pub model: String,
    pub api_base: Option<String>,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: Option<u32>,
}

#[tauri::command]
pub async fn list_providers() -> Result<Vec<LlmProvider>, String> {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("llm-providers.json");

    if config_path.exists() {
        let data = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Read providers: {e}"))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("Parse providers: {e}"))
    } else {
        Ok(vec![
            LlmProvider {
                name: "Ollama (Local)".into(),
                provider_type: "ollama".into(),
                model: "llama3.2".into(),
                api_base: Some("http://localhost:11434".into()),
                is_active: true,
            },
            LlmProvider {
                name: "OpenAI".into(),
                provider_type: "openai".into(),
                model: "gpt-4o".into(),
                api_base: None,
                is_active: false,
            },
        ])
    }
}

#[tauri::command]
pub async fn send_message(request: ChatRequest) -> Result<ChatResponse, String> {
    let providers = list_providers().await?;
    let active_provider = providers.iter().find(|p| p.is_active)
        .ok_or("No active LLM provider configured")?;

    match active_provider.provider_type.as_str() {
        "ollama" => {
            let api_base = active_provider.api_base.as_deref()
                .unwrap_or("http://localhost:11434");
            let url = format!("{}/api/chat", api_base);

            let body = serde_json::json!({
                "model": request.model.as_deref().unwrap_or(&active_provider.model),
                "messages": request.messages,
                "stream": false,
                "options": {
                    "temperature": request.temperature.unwrap_or(0.7),
                    "num_predict": request.max_tokens.unwrap_or(2048),
                }
            });

            let client = reqwest::Client::new();
            let resp = client.post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Ollama request failed: {e}"))?;

            let json: serde_json::Value = resp.json()
                .await
                .map_err(|e| format!("Parse Ollama response: {e}"))?;

            let content = json["message"]["content"]
                .as_str()
                .unwrap_or("No response")
                .to_string();

            Ok(ChatResponse {
                content,
                model: active_provider.model.clone(),
                tokens_used: json["eval_count"].as_u64().map(|n| n as u32),
            })
        }
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY not set".to_string())?;
            let url = "https://api.openai.com/v1/chat/completions";

            let messages: Vec<serde_json::Value> = request.messages.iter()
                .map(|m| serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                }))
                .collect();

            let body = serde_json::json!({
                "model": request.model.as_deref().unwrap_or(&active_provider.model),
                "messages": messages,
                "temperature": request.temperature.unwrap_or(0.7),
                "max_tokens": request.max_tokens.unwrap_or(2048),
            });

            let client = reqwest::Client::new();
            let resp = client.post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("OpenAI request failed: {e}"))?;

            let json: serde_json::Value = resp.json()
                .await
                .map_err(|e| format!("Parse OpenAI response: {e}"))?;

            let content = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("No response")
                .to_string();

            let tokens = json["usage"]["total_tokens"].as_u64().map(|n| n as u32);

            Ok(ChatResponse {
                content,
                model: active_provider.model.clone(),
                tokens_used: tokens,
            })
        }
        _ => Err(format!("Unsupported provider type: {}", active_provider.provider_type)),
    }
}
