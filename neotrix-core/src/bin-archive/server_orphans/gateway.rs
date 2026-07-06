use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use axum::{
    Router, routing::{get, post}, response::Json, extract::State,
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::neotrix::nt_io_provider::{
    LlmProvider, LlmProviderType, ProviderConfig, create_provider,
    LlmRequest, Message, Role, FinishReason, sanitize_history,
};

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub addr: String,
    pub providers: Vec<ProviderConfig>,
}

pub struct OpenAICompatibleGateway {
    pub config: GatewayConfig,
    pub providers: HashMap<String, Box<dyn LlmProvider>>,
}

impl OpenAICompatibleGateway {
    pub fn new(config: GatewayConfig) -> Self {
        let mut providers: HashMap<String, Box<dyn LlmProvider>> = HashMap::new();
        for pc in &config.providers {
            let key = match pc.provider_type {
                LlmProviderType::OpenAI => "openai",
                LlmProviderType::Anthropic => "anthropic",
                LlmProviderType::Gemini => "gemini",
                LlmProviderType::Ollama => "ollama",
                LlmProviderType::FreeApi => "free",
            };
            if !providers.contains_key(key) {
                providers.insert(key.to_string(), create_provider(pc.clone()));
            }
        }
        Self { config, providers }
    }

    pub fn from_env() -> Self {
        let config = GatewayConfig {
            addr: std::env::var("NEOTRIX_GATEWAY_ADDR").unwrap_or_else(|_| "127.0.0.1:1242".to_string()),
            providers: vec![ProviderConfig::from_env()],
        };
        Self::new(config)
    }

    fn resolve_provider(&self, model: &str) -> Option<&str> {
        let lower = model.to_lowercase();
        if (lower.contains("gpt") || lower.contains("o1")) && self.providers.contains_key("openai") {
            return Some("openai");
        }
        if lower.contains("claude") && self.providers.contains_key("anthropic") {
            return Some("anthropic");
        }
        if lower.contains("gemini") && self.providers.contains_key("gemini") {
            return Some("gemini");
        }
        if (lower.contains("llama") || lower.contains("mistral") || lower.contains("qwen") || lower.contains("deepseek"))
            && self.providers.contains_key("ollama")
        {
            return Some("ollama");
        }
        self.providers.keys().next().map(|s| s.as_str())
    }

    fn get_provider(&self, model: &str) -> Option<&dyn LlmProvider> {
        let key = self.resolve_provider(model)?;
        self.providers.get(key).map(|b| b.as_ref())
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
}

#[derive(Clone)]
pub struct GatewayState {
    pub gateway: Arc<RwLock<OpenAICompatibleGateway>>,
}

async fn models_handler(State(state): State<GatewayState>) -> Json<ModelList> {
    let gateway = state.gateway.read().await;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let mut data: Vec<ModelInfo> = gateway.providers.keys().map(|key| {
        let model_id = match key.as_str() {
            "openai" => "gpt-4o",
            "anthropic" => "claude-3-5-sonnet-20241022",
            "gemini" => "gemini-2.0-flash",
            "ollama" => "llama-3.1-8b",
            "free" => "free-api",
            _ => "neotrix-model",
        };
        ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: now,
            owned_by: key.clone(),
        }
    }).collect();

    if data.is_empty() {
        data.push(ModelInfo {
            id: "neotrix-default".to_string(),
            object: "model".to_string(),
            created: now,
            owned_by: "neotrix".to_string(),
        });
    }

    Json(ModelList { object: "list".to_string(), data })
}

async fn chat_completions_handler(
    State(state): State<GatewayState>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let gateway = state.gateway.read().await;

    if req.stream.unwrap_or(false) {
        return Err((
            axum::http::StatusCode::NOT_IMPLEMENTED,
            Json(ApiError {
                error: ErrorDetail {
                    message: "Streaming is not yet supported. Use stream=false.".to_string(),
                    error_type: "invalid_request_error".to_string(),
                    code: "streaming_not_supported".to_string(),
                },
            }),
        ));
    }

    let provider = gateway.get_provider(&req.model).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(ApiError {
                error: ErrorDetail {
                    message: format!("Model '{}' is not supported by any configured provider", req.model),
                    error_type: "invalid_request_error".to_string(),
                    code: "model_not_found".to_string(),
                },
            }),
        )
    })?;

    let messages: Vec<Message> = req.messages.iter().map(|m| Message {
        role: match m.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            _ => Role::User,
        },
        content: m.content.clone(),
        tool_calls: None,
        tool_call_id: None,
    }).collect();

    let mut llm_request = LlmRequest {
        model: req.model.clone(),
        messages,
        temperature: req.temperature.unwrap_or(0.7),
        max_tokens: req.max_tokens.unwrap_or(4096),
        tools: vec![],
    };
    sanitize_history(&mut llm_request.messages);

    let llm_response = provider.complete(&llm_request).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: ErrorDetail {
                    message: format!("Provider error: {}", e),
                    error_type: "server_error".to_string(),
                    code: "provider_error".to_string(),
                },
            }),
        )
    })?;

    let finish_reason = match llm_response.finish_reason {
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::Tool => "tool_calls",
        FinishReason::ContentFilter => "content_filter",
        FinishReason::Unknown => "stop",
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("0000")),
        object: "chat.completion".to_string(),
        created: now,
        model: llm_response.model,
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: llm_response.content,
            },
            finish_reason: finish_reason.to_string(),
        }],
        usage: Usage {
            prompt_tokens: llm_response.usage.prompt_tokens,
            completion_tokens: llm_response.usage.completion_tokens,
            total_tokens: llm_response.usage.total_tokens,
        },
    };

    Ok(Json(response))
}

pub fn start_gateway(gateway: OpenAICompatibleGateway, addr: &str) {
    let state = GatewayState {
        gateway: Arc::new(RwLock::new(gateway)),
    };
    let app = Router::new()
        .route("/v1/models", get(models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .with_state(state);
    let addr_owned = addr.to_string();
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr_owned).await.expect("Failed to bind gateway");
        println!("OpenAI-compatible gateway listening on http://{}", addr_owned);
        axum::serve(listener, app).await.expect("Gateway error");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::LlmProviderType;

    #[test]
    fn test_model_routing() {
        let config = GatewayConfig {
            addr: "127.0.0.1:1242".to_string(),
            providers: vec![
                ProviderConfig {
                    provider_type: LlmProviderType::OpenAI,
                    api_key: Some("sk-test".to_string()),
                    model: Some("gpt-4o".to_string()),
                    ..Default::default()
                },
                ProviderConfig {
                    provider_type: LlmProviderType::Anthropic,
                    api_key: Some("sk-ant-test".to_string()),
                    model: Some("claude-3-5-sonnet".to_string()),
                    ..Default::default()
                },
                ProviderConfig {
                    provider_type: LlmProviderType::Gemini,
                    api_key: Some("google-test".to_string()),
                    model: Some("gemini-2.0-flash".to_string()),
                    ..Default::default()
                },
                ProviderConfig {
                    provider_type: LlmProviderType::Ollama,
                    api_key: None,
                    model: Some("llama-3.1-8b".to_string()),
                    ..Default::default()
                },
            ],
        };
        let gateway = OpenAICompatibleGateway::new(config);
        assert_eq!(gateway.resolve_provider("gpt-4o"), Some("openai"));
        assert_eq!(gateway.resolve_provider("gpt-4-turbo"), Some("openai"));
        assert_eq!(gateway.resolve_provider("o1-preview"), Some("openai"));
        assert_eq!(gateway.resolve_provider("o1-mini"), Some("openai"));
        assert_eq!(gateway.resolve_provider("claude-3-opus-20240229"), Some("anthropic"));
        assert_eq!(gateway.resolve_provider("claude-3-5-sonnet-20241022"), Some("anthropic"));
        assert_eq!(gateway.resolve_provider("gemini-2.0-flash"), Some("gemini"));
        assert_eq!(gateway.resolve_provider("gemini-1.5-pro"), Some("gemini"));
        assert_eq!(gateway.resolve_provider("llama-3.1-8b"), Some("ollama"));
        assert_eq!(gateway.resolve_provider("mistral-7b"), Some("ollama"));
        assert_eq!(gateway.resolve_provider("deepseek-v3"), Some("ollama"));
    }

    #[test]
    fn test_empty_provider_list() {
        let config = GatewayConfig {
            addr: "127.0.0.1:1242".to_string(),
            providers: vec![],
        };
        let gateway = OpenAICompatibleGateway::new(config);
        assert!(gateway.providers.is_empty());
        assert!(gateway.get_provider("gpt-4o").is_none());
        assert!(gateway.resolve_provider("gpt-4o").is_none());
    }

    #[test]
    fn test_model_list_serialization() {
        let list = ModelList {
            object: "list".to_string(),
            data: vec![
                ModelInfo {
                    id: "gpt-4o".to_string(),
                    object: "model".to_string(),
                    created: 1677652288,
                    owned_by: "openai".to_string(),
                },
                ModelInfo {
                    id: "claude-3-5-sonnet".to_string(),
                    object: "model".to_string(),
                    created: 1677652288,
                    owned_by: "anthropic".to_string(),
                },
            ],
        };
        let json = serde_json::to_value(&list).expect("model list should serialize to json");
        assert_eq!(json["object"], "list");
        assert_eq!(json["data"].as_array().expect("data should be an array").len(), 2);
        assert_eq!(json["data"][0]["id"], "gpt-4o");
        assert_eq!(json["data"][0]["object"], "model");
        assert_eq!(json["data"][0]["owned_by"], "openai");
        assert_eq!(json["data"][1]["id"], "claude-3-5-sonnet");
        assert_eq!(json["data"][1]["owned_by"], "anthropic");
    }

    #[test]
    fn test_request_parsing() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hello!"}
            ],
            "temperature": 0.5,
            "max_tokens": 100,
            "stream": false
        }"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).expect("request json should deserialize");
        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, "system");
        assert_eq!(req.messages[0].content, "You are helpful.");
        assert_eq!(req.messages[1].role, "user");
        assert_eq!(req.messages[1].content, "Hello!");
        assert_eq!(req.temperature, Some(0.5));
        assert_eq!(req.max_tokens, Some(100));
        assert_eq!(req.stream, Some(false));
    }

    #[test]
    fn test_chat_completion_response_serialization() {
        let response = ChatCompletionResponse {
            id: "chatcmpl-test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-4o".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ResponseMessage {
                    role: "assistant".to_string(),
                    content: "Hello! I am an AI assistant.".to_string(),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };
        let json = serde_json::to_value(&response).expect("response should serialize to json");
        assert_eq!(json["id"], "chatcmpl-test-id");
        assert_eq!(json["object"], "chat.completion");
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["choices"][0]["index"], 0);
        assert_eq!(json["choices"][0]["message"]["role"], "assistant");
        assert_eq!(json["choices"][0]["message"]["content"], "Hello! I am an AI assistant.");
        assert_eq!(json["choices"][0]["finish_reason"], "stop");
        assert_eq!(json["usage"]["prompt_tokens"], 10);
        assert_eq!(json["usage"]["completion_tokens"], 5);
        assert_eq!(json["usage"]["total_tokens"], 15);
    }

    #[test]
    fn test_error_serialization() {
        let err = ApiError {
            error: ErrorDetail {
                message: "Model 'unknown-model' is not supported".to_string(),
                error_type: "invalid_request_error".to_string(),
                code: "model_not_found".to_string(),
            },
        };
        let json = serde_json::to_value(&err).expect("error should serialize to json");
        assert_eq!(json["error"]["message"], "Model 'unknown-model' is not supported");
        assert_eq!(json["error"]["type"], "invalid_request_error");
        assert_eq!(json["error"]["code"], "model_not_found");
    }

    #[test]
    fn test_default_provider_fallback() {
        let config = GatewayConfig {
            addr: "127.0.0.1:1242".to_string(),
            providers: vec![
                ProviderConfig {
                    provider_type: LlmProviderType::OpenAI,
                    api_key: Some("sk-test".to_string()),
                    model: Some("gpt-4o".to_string()),
                    ..Default::default()
                },
            ],
        };
        let gateway = OpenAICompatibleGateway::new(config);
        assert_eq!(gateway.resolve_provider("unknown-model-xyz"), Some("openai"));
        assert_eq!(gateway.resolve_provider("some-random-model"), Some("openai"));
    }
}
