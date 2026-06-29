use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::server::http::AppState;

#[derive(Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    if req.stream == Some(true) {
        let err = ErrorResponse {
            error: ErrorDetail {
                message: "Streaming not supported, use stream=false".into(),
                error_type: "invalid_request_error".into(),
                code: "400".into(),
            },
        };
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(err).unwrap_or(serde_json::Value::Null)),
        );
    }

    if req.messages.is_empty() {
        let err = ErrorResponse {
            error: ErrorDetail {
                message: "messages must not be empty".into(),
                error_type: "invalid_request_error".into(),
                code: "400".into(),
            },
        };
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(err).unwrap_or(serde_json::Value::Null)),
        );
    }

    let system_prompt: String = req
        .messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt: String = req
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = if system_prompt.is_empty() {
        user_prompt
    } else {
        format!("{system_prompt}\n\n{user_prompt}")
    };

    let mut agent = state.agent.write().await;

    let response_text;
    #[cfg(feature = "e8-theory")]
    if let Some(ref mut engine) = agent.reasoning_engine {
        response_text = match engine.reason(&prompt) {
            Ok(text) => text,
            Err(e) => {
                let err = ErrorResponse {
                    error: ErrorDetail {
                        message: format!("ReasoningEngine error: {e}"),
                        error_type: "server_error".into(),
                        code: "500".into(),
                    },
                };
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::to_value(err).unwrap_or(serde_json::Value::Null)),
                );
            }
        };
    } else {
        let result = agent.iterate(crate::neotrix::nt_expert_routing::TaskType::General);
        response_text = format!(
            "Evolution: {:.3} → {:.3}",
            result.score_before, result.score_after
        );
    }
    #[cfg(not(feature = "e8-theory"))]
    {
        let result = agent.iterate(crate::neotrix::nt_expert_routing::TaskType::General);
        response_text = format!(
            "Evolution: {:.3} → {:.3}",
            result.score_before, result.score_after
        );
    }

    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let prompt_tokens = count_tokens(&prompt);
    let completion_tokens = count_tokens(&response_text);

    let response = ChatCompletionResponse {
        id: format!(
            "chatcmpl-{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("0000")
        ),
        object: "chat.completion".into(),
        created,
        model: req.model,
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".into(),
                content: response_text,
            },
            finish_reason: "stop".into(),
        }],
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    };

    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap_or(serde_json::Value::Null)),
    )
}

pub async fn list_models() -> Json<ModelList> {
    Json(ModelList {
        object: "list".into(),
        data: vec![ModelInfo {
            id: "neotrix".into(),
            object: "model".into(),
            created: 1700000000,
            owned_by: "neotrix".into(),
        }],
    })
}

fn count_tokens(s: &str) -> u32 {
    (s.len() / 4).max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_empty() {
        assert_eq!(count_tokens(""), 1);
    }

    #[test]
    fn test_count_tokens_short() {
        assert_eq!(count_tokens("hi"), 1);
    }

    #[test]
    fn test_count_tokens_longer() {
        let s = "a".repeat(100);
        assert_eq!(count_tokens(&s), 25);
    }

    #[test]
    fn test_chat_completion_request_serde() {
        let req = ChatCompletionRequest {
            model: "neotrix".into(),
            messages: vec![Message {
                role: "user".into(),
                content: "hello".into(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
        };
        assert_eq!(req.messages.len(), 1);
    }

    #[test]
    fn test_usage_creation() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_model_list_creation() {
        let list = ModelList {
            object: "list".into(),
            data: vec![ModelInfo {
                id: "neotrix".into(),
                object: "model".into(),
                created: 1700000000,
                owned_by: "neotrix".into(),
            }],
        };
        assert_eq!(list.data.len(), 1);
    }

    #[test]
    fn test_error_response_creation() {
        let err = ErrorResponse {
            error: ErrorDetail {
                message: "Invalid request".into(),
                error_type: "invalid_request_error".into(),
                code: "400".to_string(),
            },
        };
        assert_eq!(err.error.code, "400");
    }

    #[test]
    fn test_message_serde() {
        let msg = Message {
            role: "system".into(),
            content: "be helpful".into(),
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("system"));
    }
}
