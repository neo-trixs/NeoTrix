use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream as RxStream;

use super::analyzer::TrafficAnalyzer;
use crate::neotrix::l1_body_impl::nt_io_provider::gateway::GatewayV2;
use crate::neotrix::l1_body_impl::nt_io_provider::types::{
    FinishReason, LlmRequest, Message, Role, Tool,
};

#[derive(Debug, Clone)]
pub struct ApiProxyConfig {
    pub listen_addr: String,
    pub upstream_timeout: Duration,
    pub debug: bool,
}

impl Default for ApiProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8082".into(),
            upstream_timeout: Duration::from_secs(360),
            debug: false,
        }
    }
}

#[derive(Clone)]
struct AppState {
    gateway: Arc<GatewayV2>,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
}

pub struct ApiProxy {
    config: ApiProxyConfig,
    gateway: Arc<GatewayV2>,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
}

impl ApiProxy {
    pub fn new(config: ApiProxyConfig, gateway: Arc<GatewayV2>) -> Self {
        Self {
            config,
            gateway,
            analyzer: Arc::new(Mutex::new(TrafficAnalyzer::new())),
        }
    }

    pub fn analyzer(&self) -> Arc<Mutex<TrafficAnalyzer>> {
        self.analyzer.clone()
    }

    pub async fn start(&self) -> Result<(), String> {
        let state = AppState {
            gateway: self.gateway.clone(),
            analyzer: self.analyzer.clone(),
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/v1/models", get(models_handler))
            .route("/v1/messages", post(messages_handler))
            .with_state(state);

        let addr = self.config.listen_addr.clone();
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("bind {}: {}", addr, e))?;

        println!("[api-proxy] L7 API proxy on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| format!("serve: {}", e))
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(default)]
    tools: Vec<AnthropicTool>,
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    system: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    metadata: Option<HashMap<String, String>>,
    #[serde(default)]
    #[allow(dead_code)]
    stop_sequences: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicTool {
    name: String,
    description: Option<String>,
    #[serde(default)]
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    resp_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Serialize)]
struct AnthropicModel {
    #[serde(rename = "type")]
    model_type: String,
    id: String,
    display_name: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    uptime: String,
    providers: usize,
    sessions: usize,
    version: String,
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let sessions = {
        let a = state.analyzer.lock().await;
        a.total_sessions()
    };

    Json(HealthResponse {
        status: "ok".into(),
        uptime: "running".into(),
        providers: 0,
        sessions,
        version: "neotrix-api-proxy/0.1".into(),
    })
}

async fn models_handler(State(_state): State<AppState>) -> Json<Vec<AnthropicModel>> {
    let models = vec![
        AnthropicModel {
            model_type: "model".into(),
            id: "claude-sonnet-4-6".into(),
            display_name: "Claude Sonnet 4.6".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        },
        AnthropicModel {
            model_type: "model".into(),
            id: "claude-opus-4-7".into(),
            display_name: "Claude Opus 4.7".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        },
        AnthropicModel {
            model_type: "model".into(),
            id: "claude-haiku-4-5".into(),
            display_name: "Claude Haiku 4.5".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        },
    ];
    Json(models)
}

fn to_internal_role(role: &str) -> Role {
    match role {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        _ => Role::User,
    }
}

fn to_anthropic_request(req: AnthropicRequest) -> LlmRequest {
    let mut messages: Vec<Message> = req
        .messages
        .into_iter()
        .map(|m| Message::new(to_internal_role(&m.role), &m.content))
        .collect();

    if let Some(sys) = req.system {
        messages.insert(
            0,
            Message {
                role: Role::System,
                content: sys,
                tool_calls: None,
                tool_call_id: None,
            },
        );
    }

    let tools: Vec<Tool> = req
        .tools
        .into_iter()
        .map(|t| Tool {
            name: t.name,
            description: t.description.unwrap_or_default(),
            input_schema: t.input_schema,
        })
        .collect();

    LlmRequest {
        model: req.model,
        messages,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        tools,
        image_data: None,
        thinking_budget: None,
        provider_params: HashMap::new(),
        constraint_json: None,
        structured_output: None,
        cacheable_prefix_tokens: None,
    }
}

fn to_anthropic_response(resp: &crate::neotrix::l1_body_impl::nt_io_provider::types::LlmResponse, model: &str) -> AnthropicResponse {
    let stop_reason = match resp.finish_reason {
        FinishReason::Stop => Some("end_turn".into()),
        FinishReason::Length => Some("max_tokens".into()),
        FinishReason::Tool => Some("tool_use".into()),
        _ => None,
    };

    AnthropicResponse {
        id: format!("msg_{:x}", rand::random::<u64>()),
        resp_type: "message".into(),
        role: "assistant".into(),
        content: vec![AnthropicContent {
            content_type: "text".into(),
            text: Some(resp.content.clone()),
        }],
        model: model.to_string(),
        stop_reason,
        stop_sequence: None,
        usage: AnthropicUsage {
            input_tokens: resp.usage.prompt_tokens,
            output_tokens: resp.usage.completion_tokens,
        },
    }
}

async fn messages_handler(
    State(state): State<AppState>,
    Json(body): Json<AnthropicRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let is_stream = body.stream;
    let model = body.model.clone();

    let internal = to_anthropic_request(body);

    let session_id = {
        let mut a = state.analyzer.lock().await;
        let body_text = format!("{:?}", internal);
        let headers = vec![
            ("x-api-key".into(), "***".into()),
            ("content-type".into(), "application/json".into()),
        ];
        a.capture_request(
            &format!("api-proxy:{}", internal.model),
            8082,
            "POST",
            "/v1/messages",
            &headers,
            body_text.as_bytes(),
        )
    };

    if is_stream {
        return handle_stream(state, internal, model, session_id).await;
    }

    match state.gateway.complete_with_selection(&internal).await {
        Ok(resp) => {
            let mut a = state.analyzer.lock().await;
            let resp_text = serde_json::to_string(&resp).unwrap_or_default();
            a.capture_response(session_id, 200, "OK", &[], resp_text.as_bytes());

            let anthropic = to_anthropic_response(&resp, &model);
            Ok(Json(anthropic).into_response())
        }
        Err(e) => {
            let mut a = state.analyzer.lock().await;
            a.capture_response(session_id, 500, "Error", &[], format!("{:?}", e).as_bytes());

            let err_body = serde_json::json!({
                "error": {
                    "type": "api_error",
                    "message": format!("{:?}", e)
                }
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err_body)))
        }
    }
}

async fn handle_stream(
    state: AppState,
    request: LlmRequest,
    model: String,
    session_id: u64,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    match state.gateway.stream_complete_with_selection(&request).await {
        Ok(mut rx) => {
            let a2 = state.analyzer.clone();

            let (tx, rx_out) = tokio::sync::mpsc::channel::<Result<Event, std::convert::Infallible>>(64);

            tokio::spawn(async move {
                let _ = tx.send(Ok(Event::default().event("message_start").data(
                    r#"{"type":"message_start","message":{"id":"msg_","type":"message","role":"assistant","content":[],"model":""#.to_string()
                    + &serde_json::to_string(&model).unwrap_or_default() + r#""}}"#,
                ))).await;

                let mut full_content = String::new();

                while let Some(chunk) = rx.recv().await {
                    match chunk {
                        Ok(partial) => {
                            full_content.push_str(&partial.content);
                            let _ = tx.send(Ok(Event::default().event("content_block_delta").data(
                                serde_json::json!({
                                    "type": "content_block_delta",
                                    "index": 0,
                                    "delta": {
                                        "type": "text_delta",
                                        "text": partial.content
                                    }
                                }).to_string(),
                            ))).await;
                        }
                        Err(e) => {
                            let _ = tx.send(Ok(Event::default().event("error").data(
                                format!("{{\"type\":\"error\",\"error\":\"{:?}\"}}", e),
                            ))).await;
                            break;
                        }
                    }
                }

                let _ = tx.send(Ok(Event::default().event("message_delta").data(
                    serde_json::json!({
                        "type": "message_delta",
                        "delta": {
                            "stop_reason": "end_turn",
                            "stop_sequence": null
                        },
                        "usage": {
                            "output_tokens": 0
                        }
                    }).to_string(),
                ))).await;

                {
                    let mut a = a2.lock().await;
                    a.capture_response(session_id, 200, "OK", &[], full_content.as_bytes());
                }
            });

            let stream = Sse::new(RxStream::new(rx_out))
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(15)),
                );

            Ok(stream.into_response())
        }
        Err(e) => {
            let err_body = serde_json::json!({
                "error": { "type": "api_error", "message": format!("{:?}", e) }
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err_body)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_internal_role() {
        assert_eq!(to_internal_role("user"), Role::User);
        assert_eq!(to_internal_role("assistant"), Role::Assistant);
        assert_eq!(to_internal_role("system"), Role::User);
    }

    #[test]
    fn test_to_anthropic_req() {
        let ext = AnthropicRequest {
            model: "claude-sonnet-4-6".into(),
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            max_tokens: 4096,
            tools: vec![],
            stream: false,
            temperature: Some(0.7),
            system: Some("be helpful".into()),
            metadata: None,
            stop_sequences: vec![],
        };
        let internal = to_anthropic_request(ext);
        assert_eq!(internal.model, "claude-sonnet-4-6");
        assert_eq!(internal.messages.len(), 2);
        assert!(internal.messages[0].content.contains("helpful"));
        assert_eq!(internal.messages[1].content, "hello");
    }

    #[test]
    fn test_api_proxy_config_default() {
        let cfg = ApiProxyConfig::default();
        assert_eq!(cfg.listen_addr, "127.0.0.1:8082");
        assert_eq!(cfg.upstream_timeout, Duration::from_secs(360));
    }

    #[test]
    fn test_to_anthropic_response() {
        let resp = crate::neotrix::l1_body_impl::nt_io_provider::types::LlmResponse {
            content: "Hello, I'm Claude.".into(),
            finish_reason: FinishReason::Stop,
            usage: crate::neotrix::l1_body_impl::nt_io_provider::types::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            model: "claude-sonnet-4-6".into(),
            tool_calls: None,
        };
        let anthropic = to_anthropic_response(&resp, "claude-sonnet-4-6");
        assert_eq!(anthropic.role, "assistant");
        assert_eq!(anthropic.stop_reason.as_deref(), Some("end_turn"));
        assert!(!anthropic.content.is_empty());
        assert_eq!(anthropic.content[0].text.as_deref(), Some("Hello, I'm Claude."));
    }
}
