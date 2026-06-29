use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::sse::KeepAlive;
use axum::response::Sse;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use futures::stream::unfold;
use futures::stream::Stream;
use tokio::sync::broadcast;

use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::response::Response;

use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_agent::message::AgentId;
use crate::core::nt_core_agent::message::AgentMessage;
use crate::core::nt_core_agent::message::MessageContent;
use crate::core::nt_core_agent::message::MessagePriority;
use crate::core::nt_core_hive::signed_card::SignedAgentCard;
use crate::neotrix::nt_agent_protocol::a2a_auth::{
    auth_middleware, A2AAuthConfig, A2AMiddlewareState, RateLimiter,
};
use crate::neotrix::nt_agent_protocol::a2a_negotiation::{
    A2ANegotiator, NegotiationOffer, NegotiationResponse,
};

use super::types::{A2A_PROTOCOL_VERSION, A2AMessage, A2APart, A2APartType, TaskEvent};
use super::{
    A2ATask, AgentCapabilities, AgentCard, AgentInterface, CancelTaskResponse, GetTaskResponse,
    SendTaskRequest, SendTaskResponse, SkillDecl, TaskState,
};

// ── A2A Server ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct A2AState {
    agent_card: Arc<AgentCard>,
    negotiator: Arc<A2ANegotiator>,
    bus: Arc<Mutex<AgentCommunicationBus>>,
    self_id: Arc<AgentId>,
    tasks: Arc<Mutex<HashMap<String, A2ATask>>>,
    event_channels: Arc<Mutex<HashMap<String, broadcast::Sender<TaskEvent>>>>,
    signed_card: Option<SignedAgentCard>,
    auth_config: A2AAuthConfig,
}

pub struct A2AServer {
    agent_card: AgentCard,
    negotiator: A2ANegotiator,
    bus: Arc<Mutex<AgentCommunicationBus>>,
    self_id: AgentId,
    port: u16,
    signing_key: Option<k256::ecdsa::SigningKey>,
    signer_name: String,
    auth_config: A2AAuthConfig,
}

impl A2AServer {
    pub fn new(
        name: &str,
        description: &str,
        port: u16,
        bus: AgentCommunicationBus,
        self_id: AgentId,
    ) -> Self {
        let url = format!("http://0.0.0.0:{}", port);
        let negotiator = A2ANegotiator::default_v1_2();
        let card = AgentCard {
            name: name.to_string(),
            description: description.to_string(),
            url: url.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: AgentCapabilities::default(),
            skills: vec![SkillDecl {
                id: "neotrix-general".into(),
                name: "NeoTrix General Agent".into(),
                description: "General-purpose cognitive agent with multi-step reasoning and knowledge representation".into(),
                tags: vec!["reasoning".into(), "knowledge".into(), "planning".into()],
            }],
            supported_interfaces: vec![AgentInterface {
                url,
                protocol_binding: "HTTP+JSON".into(),
                protocol_version: "1.0".into(),
            }],
            negotiation_endpoint: Some("/.well-known/negotiate".into()),
            key_id: None,
            signature: None,
            default_input_modes: vec!["text/plain".into(), "application/json".into()],
            default_output_modes: vec!["text/plain".into(), "application/json".into()],
        };
        Self {
            agent_card: card,
            negotiator,
            bus: Arc::new(Mutex::new(bus)),
            self_id,
            port,
            signing_key: None,
            signer_name: name.to_string(),
            auth_config: A2AAuthConfig::default(),
        }
    }

    pub fn with_signing_key(mut self, signing_key: k256::ecdsa::SigningKey) -> Self {
        self.signing_key = Some(signing_key);
        self
    }

    pub fn with_auth(mut self, config: A2AAuthConfig) -> Self {
        self.auth_config = config;
        self
    }

    pub async fn start(self) -> Result<u16, String> {
        let signed_card = self
            .signing_key
            .as_ref()
            .map(|key| {
                let card_json = serde_json::to_vec(&self.agent_card)
                    .map_err(|e| format!("serialize card: {}", e))?;
                SignedAgentCard::sign(card_json, key, &self.signer_name)
                    .map_err(|e| format!("sign card: {}", e))
            })
            .transpose()?;

        // Also create a JWS-signed AgentCard (inline signature in the JSON body)
        let agent_card = if let Some(ref key) = self.signing_key {
            self.agent_card.to_signed_card(key)?
        } else {
            self.agent_card.clone()
        };

        let mw_state = Arc::new(A2AMiddlewareState {
            config: self.auth_config.clone(),
            rate_limiter: tokio::sync::Mutex::new(RateLimiter::new(60)),
        });

        let state = A2AState {
            agent_card: Arc::new(agent_card),
            negotiator: Arc::new(self.negotiator),
            bus: self.bus,
            self_id: Arc::new(self.self_id),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            event_channels: Arc::new(Mutex::new(HashMap::new())),
            signed_card,
            auth_config: self.auth_config,
        };

        let app = Router::new()
            .route("/.well-known/agent-card", get(get_agent_card))
            .route("/.well-known/negotiate", post(negotiate_handler))
            .route("/a2a/discover", get(discover_handler))
            .route("/tasks/send", post(send_task_handler))
            .route("/tasks/{id}", get(get_task_handler))
            .route("/tasks/{id}/stream", get(stream_task_handler))
            .route("/tasks/{id}/cancel", post(cancel_task_handler))
            .layer(from_fn_with_state(mw_state, auth_middleware))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(("0.0.0.0", self.port))
            .await
            .map_err(|e| format!("A2A bind: {}", e))?;
        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("addr: {}", e))?
            .port();
        // No inner tokio::spawn here: start() is already called inside a
        // spawned task (BackgroundLoop::spawn), so axum::serve runs in
        // that tracked task. On BackgroundLoop shutdown, the outer task is
        // aborted → axum::serve future is dropped → server is cancelled.
        #[allow(clippy::let_and_return)]
        let port = actual_port;
        log::info!("[a2a] server listening on :{}", port);
        if let Err(e) = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        {
            log::error!("[a2a] server error: {:?}", e);
        }
        Ok(port)
    }
}

async fn discover_handler(State(state): State<A2AState>) -> impl IntoResponse {
    let card = (*state.agent_card).clone();
    let capabilities = card.capabilities.clone();
    let supported_bindings: Vec<String> = card
        .supported_interfaces
        .iter()
        .map(|i| i.protocol_binding.clone())
        .collect();

    let discovery = serde_json::json!({
        "agentCard": card,
        "capabilities": capabilities,
        "protocolVersion": A2A_PROTOCOL_VERSION,
        "supportedBindings": supported_bindings,
    });

    Json(discovery).into_response()
}

async fn get_agent_card(State(state): State<A2AState>) -> Response {
    let card = (*state.agent_card).clone();
    let mut resp = Json(card).into_response();
    if let Some(ref signed) = state.signed_card {
        let headers = resp.headers_mut();
        if let Ok(val) = hex::encode(&signed.signature).parse::<HeaderValue>() {
            headers.insert("x-a2a-signature", val);
        }
        if let Ok(val) = hex::encode(&signed.signer_pubkey).parse::<HeaderValue>() {
            headers.insert("x-a2a-signer-pubkey", val);
        }
        if let Ok(val) = signed.signer_name.parse::<HeaderValue>() {
            headers.insert("x-a2a-signer-name", val);
        }
    }
    resp
}

async fn negotiate_handler(
    State(state): State<A2AState>,
    Json(offer): Json<NegotiationOffer>,
) -> Json<NegotiationResponse> {
    Json(state.negotiator.negotiate(&offer))
}

async fn send_task_handler(
    State(state): State<A2AState>,
    Json(req): Json<SendTaskRequest>,
) -> Result<Json<SendTaskResponse>, (StatusCode, String)> {
    // ── Session-level concurrent task limit ─────────────────────────────
    {
        let tasks = state.tasks.lock().map_err(|e: std::sync::PoisonError<_>| {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
        let active = tasks
            .values()
            .filter(|t| t.session_id == req.session_id && !t.status.is_terminal())
            .count();
        if active >= state.auth_config.max_concurrent_tasks_per_session as usize {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                serde_json::to_string(&serde_json::json!({
                    "error": "too many concurrent tasks for this session"
                }))
                .unwrap_or_else(|_| "too many concurrent tasks".into()),
            ));
        }
    }

    let session_id = req.session_id.clone();
    let task = A2ATask {
        id: req.id.clone(),
        session_id,
        status: TaskState::Submitted,
        messages: req.messages.clone(),
        artifacts: Vec::new(),
        error_message: None,
        metadata: req.metadata.clone(),
    };

    let task_id = task.id.clone();
    state
        .tasks
        .lock()
        .map_err(|e: std::sync::PoisonError<_>| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .insert(task_id.clone(), task);

    let msg_content = MessageContent::TaskRequest {
        description: req
            .messages
            .first()
            .and_then(|m| m.parts.first())
            .and_then(|p| p.text.clone())
            .unwrap_or_default(),
        domain: "a2a".into(),
        constraints: vec![],
    };
    let agent_msg = AgentMessage::new(
        (*state.self_id).clone(),
        vec![],
        msg_content,
        MessagePriority::Normal,
        Duration::from_secs(300),
    );
    if let Ok(mut bus) = state.bus.lock() {
        if let Err(e) = bus.send(agent_msg) {
            log::warn!("[a2a] bus send failed: {}", e);
        }
        let delivered = bus.deliver();
        if !delivered.is_empty() {
            if let Some(mut tasks) = state.tasks.lock().ok() {
                if let Some(t) = tasks.get_mut(&task_id) {
                    let t: &mut A2ATask = t;
                    t.status = TaskState::Working;
                    let _ = emit_event_state(
                        &state,
                        &task_id,
                        TaskEvent {
                            task_id: task_id.clone(),
                            event_type: "status_update".into(),
                            status: TaskState::Working,
                            message: None,
                            artifact: None,
                            error: None,
                        },
                    );
                }
            }
            let response_text = delivered
                .iter()
                .map(|m| format!("{:?}", m.content))
                .collect::<Vec<_>>()
                .join("\n");
            if let Some(mut tasks) = state.tasks.lock().ok() {
                if let Some(t) = tasks.get_mut(&task_id) {
                    let t: &mut A2ATask = t;
                    t.messages.push(A2AMessage {
                        role: "assistant".into(),
                        parts: vec![A2APart {
                            part_type: A2APartType::Text,
                            text: Some(response_text),
                            mime_type: Some("text/plain".into()),
                            file_uri: None,
                            data: None,
                        }],
                    });
                    t.status = TaskState::Completed;
                }
            }
        }
    }
    if let Some(tasks) = state.tasks.lock().ok() {
        if let Some(task) = tasks.get(&task_id) {
            let task: &A2ATask = task;
            return Ok(Json(SendTaskResponse { task: task.clone() }));
        }
    }
    Err((StatusCode::INTERNAL_SERVER_ERROR, "task not found".into()))
}

async fn get_task_handler(
    State(state): State<A2AState>,
    Path(id): Path<String>,
) -> Result<Json<GetTaskResponse>, (StatusCode, String)> {
    let tasks = state.tasks.lock().map_err(|e: std::sync::PoisonError<_>| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    let task = tasks
        .get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "task not found".into()))?;
    Ok(Json(GetTaskResponse { task: task.clone() }))
}

async fn stream_task_handler(
    State(state): State<A2AState>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)>
{
    let rx = {
        let mut channels = state
            .event_channels
            .lock()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let tx = channels
            .entry(id.clone())
            .or_insert_with(|| {
                let (new_tx, _) = broadcast::channel(64);
                new_tx
            })
            .clone();
        tx.subscribe()
    };

    let stream = unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => {
                let json = serde_json::to_string(&event).unwrap_or_default();
                Some((Ok(Event::default().data(json)), rx))
            }
            Err(broadcast::error::RecvError::Closed) => None,
            Err(broadcast::error::RecvError::Lagged(_)) => {
                Some((Ok(Event::default().data("{}")), rx))
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn cancel_task_handler(
    State(state): State<A2AState>,
    Path(id): Path<String>,
) -> Result<Json<CancelTaskResponse>, (StatusCode, String)> {
    let mut tasks = state.tasks.lock().map_err(|e: std::sync::PoisonError<_>| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    if let Some(task) = tasks.get_mut(&id) {
        task.status = TaskState::Canceled;
        return Ok(Json(CancelTaskResponse { task: task.clone() }));
    }
    Err((StatusCode::NOT_FOUND, "task not found".into()))
}

pub(crate) fn emit_event(
    channels: &Arc<Mutex<HashMap<String, broadcast::Sender<TaskEvent>>>>,
    task_id: &str,
    event: TaskEvent,
) {
    if let Ok(chans) = channels.lock() {
        if let Some(tx) = chans.get(task_id) {
            if tx.send(event).is_err() {
                log::warn!(
                    "a2a emit_event send failed for task {}: channel closed",
                    task_id
                );
            }
        }
    }
}

fn emit_event_state(state: &A2AState, task_id: &str, event: TaskEvent) {
    emit_event(&state.event_channels, task_id, event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_agent::bus::AgentCommunicationBus;

    fn test_bus() -> AgentCommunicationBus {
        let bus = AgentCommunicationBus::new(100);
        bus
    }

    fn test_agent_id() -> AgentId {
        AgentId::with_random_instance("a2a-test", "1.0")
    }

    #[test]
    fn test_a2a_server_creation() {
        let bus = test_bus();
        let self_id = test_agent_id();
        let server = A2AServer::new("test-agent", "test description", 0, bus, self_id);
        assert_eq!(server.agent_card.name, "test-agent");
        assert_eq!(server.agent_card.skills.len(), 1);
    }
}
