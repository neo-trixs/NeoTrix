use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::core::nt_core_agent::bus::AgentCommunicationBus;
use crate::core::nt_core_agent::message::AgentId;
use crate::core::nt_core_agent::message::AgentMessage;
use crate::core::nt_core_agent::message::MessageContent;
use crate::core::nt_core_agent::message::MessagePriority;

use super::super::a2a::{
    emit_event, A2AMessage, A2APart, A2APartType, A2ATask, AgentCard, TaskEvent, TaskState,
};
use super::super::a2a_auth::{A2AAuthConfig, RateLimiter};
use super::super::a2a_negotiation::{A2ANegotiator, NegotiationOffer};
use super::super::agent_card_v12::{
    build_agent_card_payload, AuthConfig, SignedAgentCardV12, SkillEntryV12,
    A2A_V12_PROTOCOL_VERSION,
};
use super::conversion::{convert_to_grpc_task, uuid_v4};
use super::types::{
    sign_agent_card_hmac, AgentEndpoint, GrpcCancelTaskRequest, GrpcFrame, GrpcGetTaskRequest,
    GrpcListTasksRequest, GrpcListTasksResponse, GrpcMethod, GrpcSendMessageRequest,
    GrpcSendMessageResponse, GrpcStreamResponse, JwsSignature, MultiTenantRegistry,
    SignedAgentCard,
};

/// Response wrapper for the A2A agent card endpoint, returning a
/// `SignedAgentCard` with JWS-compliant HMAC-SHA256 signature(s) per
/// A2A v1.0.1 spec (RFC 7515).
#[derive(Debug, Clone, Serialize)]
struct AgentCardSignedResponse {
    card: AgentCard,
    /// JWS signatures keyed by algorithm ("HS256").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    signatures: Vec<JwsSignature>,
    /// Multi-tenant agent list (included when `?tenants=true`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tenants: Vec<AgentCard>,
}

struct GrpcServerState {
    bus: Arc<Mutex<AgentCommunicationBus>>,
    self_id: Arc<AgentId>,
    tasks: Arc<Mutex<HashMap<String, A2ATask>>>,
    event_channels: Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<TaskEvent>>>>,
    agent_card: Arc<AgentCard>,
    /// Signed version of the agent card (JWS HMAC-SHA256).
    signed_card: Arc<Option<SignedAgentCard>>,
    /// A2A v1.2 JWT-signed agent card string (for .well-known/agent-card JWT response).
    signed_card_v12: Arc<Option<String>>,
    /// Multi-tenant agent registry (A2A v1.0.1).
    tenants: Arc<Mutex<MultiTenantRegistry>>,
    negotiator: A2ANegotiator,
    auth_config: A2AAuthConfig,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    shutdown: CancellationToken,
}

pub struct A2AGrpcServer {
    bus: Arc<Mutex<AgentCommunicationBus>>,
    self_id: AgentId,
    port: u16,
    tasks: Arc<Mutex<HashMap<String, A2ATask>>>,
    event_channels: Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<TaskEvent>>>>,
    agent_card: AgentCard,
    /// Signed AgentCard (JWS-style, populated when auth is configured).
    signed_card: Option<SignedAgentCard>,
    /// A2A v1.2 JWT-signed agent card as a compact JWT string.
    signed_card_v12: Option<String>,
    /// Multi-tenant agent registry (A2A v1.0.1).
    tenants: MultiTenantRegistry,
    negotiator: A2ANegotiator,
    auth_config: A2AAuthConfig,
    shutdown: CancellationToken,
}

impl A2AGrpcServer {
    pub fn new(
        port: u16,
        bus: AgentCommunicationBus,
        self_id: AgentId,
        shutdown: CancellationToken,
    ) -> Self {
        let url = format!("http://0.0.0.0:{port}");
        let negotiator = A2ANegotiator::default_v1_2();
        let agent_card = AgentCard {
            name: format!("a2a-grpc:{}", self_id.name),
            description: "A2A gRPC agent".into(),
            url: url.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: Default::default(),
            skills: vec![],
            supported_interfaces: vec![super::super::a2a::AgentInterface {
                url,
                protocol_binding: "GRPC".into(),
                protocol_version: "1.0".into(),
            }],
            negotiation_endpoint: Some("/.well-known/negotiate".into()),
            key_id: None,
            signature: None,
            default_input_modes: vec!["text/plain".into(), "application/json".into()],
            default_output_modes: vec!["text/plain".into(), "application/json".into()],
        };
        let signed_card = None;
        let signed_card_v12 = None;
        Self {
            bus: Arc::new(Mutex::new(bus)),
            self_id,
            port,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            event_channels: Arc::new(Mutex::new(HashMap::new())),
            agent_card,
            signed_card,
            signed_card_v12,
            tenants: MultiTenantRegistry::new(),
            negotiator,
            auth_config: A2AAuthConfig::default(),
            shutdown,
        }
    }

    pub fn with_auth(mut self, config: A2AAuthConfig) -> Self {
        let api_key = config.api_key.clone();
        self.auth_config = config;
        if let Some(ref key) = api_key {
            self.signed_card = Some(sign_agent_card_hmac(&self.agent_card, key));
            self.signed_card_v12 = Some(self.build_v12_jwt(key));
        }
        self
    }

    pub fn with_signed_card_v12(mut self, jwt: String) -> Self {
        self.signed_card_v12 = Some(jwt);
        self
    }

    pub fn with_agent_card(mut self, card: AgentCard) -> Self {
        let api_key = self.auth_config.api_key.clone();
        if let Some(ref key) = api_key {
            self.signed_card = Some(sign_agent_card_hmac(&card, key));
            self.signed_card_v12 = Some(self.build_v12_jwt(key));
        }
        self.agent_card = card;
        self
    }

    /// Build a JWT-signed Agent Card v1.2 from the current agent_card.
    fn build_v12_jwt(&self, api_key: &str) -> String {
        let payload = build_agent_card_payload(
            &self.agent_card.name,
            &self.agent_card.description,
            &self.agent_card.url,
            A2A_V12_PROTOCOL_VERSION,
            vec!["streaming".into(), "a2a-v12".into()],
            self.agent_card
                .skills
                .iter()
                .map(|s| SkillEntryV12 {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    description: s.description.clone(),
                    input_schema: std::collections::HashMap::new(),
                    output_schema: std::collections::HashMap::new(),
                })
                .collect(),
            AuthConfig {
                scheme: "bearer".into(),
                credentials: Some(api_key.to_string()),
            },
            86400,
        );
        SignedAgentCardV12::sign(payload, api_key.as_bytes()).unwrap_or_else(|e| {
            log::error!("[a2a-grpc] failed to sign v1.2 card: {e}");
            String::new()
        })
    }

    pub fn with_negotiator(mut self, negotiator: A2ANegotiator) -> Self {
        self.negotiator = negotiator;
        self
    }

    /// Register a tenant agent in the multi-tenant registry.
    /// The agent's card is signed with the configured API key.
    pub fn with_tenant(mut self, agent_id: &str, card: AgentCard) -> Self {
        if let Some(ref api_key) = self.auth_config.api_key {
            let signed = sign_agent_card_hmac(&card, api_key);
            self.tenants.register(AgentEndpoint {
                agent_id: agent_id.to_string(),
                card: signed,
                handler: None,
            });
        }
        self
    }

    pub async fn start(self) -> Result<u16, String> {
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", self.port))
            .await
            .map_err(|e| format!("gRPC bind: {e}"))?;
        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("addr: {e}"))?
            .port();

        let state = GrpcServerState {
            bus: self.bus,
            self_id: Arc::new(self.self_id),
            tasks: self.tasks,
            event_channels: self.event_channels,
            agent_card: Arc::new(self.agent_card),
            signed_card: Arc::new(self.signed_card),
            signed_card_v12: Arc::new(self.signed_card_v12),
            tenants: Arc::new(Mutex::new(self.tenants)),
            negotiator: self.negotiator,
            auth_config: self.auth_config.clone(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(
                self.auth_config.rate_limit_per_min,
            ))),
            shutdown: self.shutdown.clone(),
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = state.shutdown.cancelled() => {
                        log::info!("[a2a-grpc] server shutting down");
                        break;
                    }
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                let state = state.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_grpc_connection(stream, state).await {
                                        log::debug!("[a2a-grpc] connection {addr} error: {e}");
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("[a2a-grpc] accept error: {e}");
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(actual_port)
    }
}

impl Clone for GrpcServerState {
    fn clone(&self) -> Self {
        Self {
            bus: self.bus.clone(),
            self_id: self.self_id.clone(),
            tasks: self.tasks.clone(),
            event_channels: self.event_channels.clone(),
            agent_card: self.agent_card.clone(),
            signed_card: self.signed_card.clone(),
            signed_card_v12: self.signed_card_v12.clone(),
            tenants: self.tenants.clone(),
            negotiator: self.negotiator.clone(),
            auth_config: self.auth_config.clone(),
            rate_limiter: self.rate_limiter.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

fn check_grpc_http_request(
    headers: &HashMap<String, String>,
    config: &A2AAuthConfig,
    rate_limiter: &mut RateLimiter,
    peer_addr: Option<SocketAddr>,
) -> Result<(), (u16, String)> {
    if let Some(ref api_key) = config.api_key {
        let auth = headers.get("authorization");
        let valid = auth.is_some_and(|v| v == &format!("Bearer {api_key}"));
        if !valid {
            return Err((401, "missing or invalid API key".into()));
        }
    }
    if let Some(cl) = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
    {
        if cl > config.max_request_size {
            return Err((413, "request body too large".into()));
        }
    }
    if let Some(addr) = peer_addr {
        if !rate_limiter.check_and_record(addr) {
            return Err((429, "rate limit exceeded".into()));
        }
    }
    Ok(())
}

async fn handle_grpc_connection(
    mut stream: tokio::net::TcpStream,
    state: GrpcServerState,
) -> Result<(), String> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("read: {e}"))?;

    if n == 0 {
        return Err("connection closed".into());
    }

    let data = &buf[..n];

    let header_end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "no header boundary".to_string())?;

    let header_part = &data[..header_end];
    let header_str =
        std::str::from_utf8(header_part).map_err(|_| "invalid header utf8".to_string())?;

    let request_line = header_str
        .lines()
        .next()
        .ok_or_else(|| "empty request line".to_string())?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("invalid request line".into());
    }
    let http_method = parts[0];
    let method_path = parts[1];

    let headers: HashMap<String, String> = header_str
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            let key = parts.next()?.trim().to_lowercase();
            let value = parts.next()?.trim().to_string();
            Some((key, value))
        })
        .collect();

    let peer_addr = stream.peer_addr().ok();
    let auth_error: Option<(u16, String)> = {
        let mut rl = state.rate_limiter.lock().map_err(|e| e.to_string())?;
        match check_grpc_http_request(&headers, &state.auth_config, &mut rl, peer_addr) {
            Err(e) => Some(e),
            Ok(()) => None,
        }
    };
    if let Some((status, msg)) = auth_error {
        let body = serde_json::json!({"error": msg}).to_string();
        let resp = format!(
            "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            status, body.len(), body
        );
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;
        stream.flush().await.map_err(|e| format!("flush: {e}"))?;
        return Ok(());
    }

    match (http_method, method_path) {
        ("GET", "/.well-known/agent-card.jwt") => {
            // A2A v1.2: Return compact JWT-signed agent card
            let jwt = &*state.signed_card_v12;
            if let Some(ref jwt_str) = jwt {
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/jwt\r\nContent-Length: {}\r\n\r\n{}",
                    jwt_str.len(),
                    jwt_str,
                );
                stream
                    .write_all(resp.as_bytes())
                    .await
                    .map_err(|e| format!("write: {e}"))?;
                stream.flush().await.map_err(|e| format!("flush: {e}"))?;
            } else {
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                stream
                    .write_all(resp.as_bytes())
                    .await
                    .map_err(|e| format!("write: {e}"))?;
                stream.flush().await.map_err(|e| format!("flush: {e}"))?;
            }
            return Ok(());
        }
        ("GET", "/.well-known/agent-card") => {
            let card = &*state.agent_card;
            let signed_card = &*state.signed_card;
            let signed_card_v12 = &*state.signed_card_v12;

            let signatures = signed_card
                .as_ref()
                .map(|sc| sc.signatures.clone())
                .unwrap_or_default();

            // Collect tenant cards within a tight lock scope to avoid holding
            // MutexGuard across .await (Send constraint).
            let tenant_cards: Vec<AgentCard> = {
                let tenants = state.tenants.lock().map_err(|e| e.to_string())?;
                tenants.agents().map(|e| e.card.card.clone()).collect()
            };

            let resp_json;
            if let Some(ref jwt) = signed_card_v12 {
                let extended = serde_json::json!({
                    "card": card,
                    "signatures": signatures,
                    "tenants": tenant_cards,
                    "jwt_v12": jwt,
                    "protocol_version": "1.2",
                });
                resp_json = serde_json::to_string(&extended).map_err(|e| format!("json: {e}"))?;
            } else {
                let response = AgentCardSignedResponse {
                    card: card.clone(),
                    signatures,
                    tenants: tenant_cards,
                };
                resp_json = serde_json::to_string(&response).map_err(|e| format!("json: {e}"))?;
            }

            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                resp_json.len(),
                resp_json,
            );
            stream
                .write_all(resp.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            stream.flush().await.map_err(|e| format!("flush: {e}"))?;
            return Ok(());
        }
        ("POST", "/.well-known/negotiate") => {
            let body = &data[header_end + 4..n];
            let offer: NegotiationOffer =
                serde_json::from_slice(body).map_err(|e| format!("parse offer: {e}"))?;
            let response = state.negotiator.negotiate(&offer);
            let resp_json = serde_json::to_string(&response).map_err(|e| format!("json: {e}"))?;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                resp_json.len(),
                resp_json,
            );
            stream
                .write_all(resp.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            stream.flush().await.map_err(|e| format!("flush: {e}"))?;
            return Ok(());
        }
        _ => {}
    }

    let body = &data[header_end + 4..n];

    if body.is_empty() {
        return Err("empty gRPC body".into());
    }

    let mut remaining = body;
    let mut resp_frames = Vec::new();

    loop {
        if remaining.is_empty() {
            break;
        }
        let (frame, rest) = GrpcFrame::decode(remaining)?;
        remaining = rest;

        let response = dispatch_grpc_method(method_path, &frame.payload, &state).await?;
        let resp_frame = GrpcFrame::encode_raw(&response);
        resp_frames.push(resp_frame);
    }

    let mut response_body = Vec::new();
    for frame in &resp_frames {
        response_body.extend_from_slice(frame);
    }

    let http_response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/grpc\r\n\
         Content-Length: {}\r\n\
         \r\n",
        response_body.len()
    );

    stream
        .write_all(http_response.as_bytes())
        .await
        .map_err(|e| format!("write headers: {e}"))?;
    stream
        .write_all(&response_body)
        .await
        .map_err(|e| format!("write body: {e}"))?;
    stream.flush().await.map_err(|e| format!("flush: {e}"))?;

    Ok(())
}

async fn dispatch_grpc_method(
    method_path: &str,
    payload: &[u8],
    state: &GrpcServerState,
) -> Result<Vec<u8>, String> {
    let method = GrpcMethod::from_path(method_path)
        .ok_or_else(|| format!("unknown method: {method_path}"))?;

    match method {
        GrpcMethod::SendMessage => {
            let req: GrpcSendMessageRequest =
                serde_json::from_slice(payload).map_err(|e| format!("parse request: {e}"))?;

            // Multi-tenancy: resolve tenant agent if specified
            let tenant_id = req.tenant.as_deref();
            if let Ok(tenants) = state.tenants.lock() {
                if let Some(endpoint) = tenants.resolve(tenant_id) {
                    log::debug!("[a2a-grpc] routing to tenant: {}", endpoint.agent_id);
                }
            }

            let task_id = format!("grpc-{}", uuid_v4());
            let text = req
                .message
                .parts
                .first()
                .and_then(|p| p.text.clone())
                .unwrap_or_default();

            let task = A2ATask {
                id: task_id.clone(),
                session_id: String::new(),
                status: TaskState::Submitted,
                messages: vec![A2AMessage {
                    role: req.message.role.clone(),
                    parts: req
                        .message
                        .parts
                        .iter()
                        .map(|p| A2APart {
                            part_type: A2APartType::Text,
                            text: p.text.clone(),
                            mime_type: p.mime_type.clone(),
                            file_uri: None,
                            data: None,
                        })
                        .collect(),
                }],
                artifacts: Vec::new(),
                error_message: None,
                metadata: req.metadata,
            };

            state
                .tasks
                .lock()
                .map_err(|e| e.to_string())?
                .insert(task_id.clone(), task);

            let msg_content = MessageContent::TaskRequest {
                description: text,
                domain: "a2a-grpc".into(),
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
                    log::warn!("[a2a_grpc] bus send failed: {}", e);
                }
                let delivered = bus.deliver();
                if !delivered.is_empty() {
                    if let Some(mut tasks) = state.tasks.lock().ok() {
                        if let Some(t) = tasks.get_mut(&task_id) {
                            t.status = TaskState::Working;
                            let _ = emit_event(
                                &state.event_channels,
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

            let task_resp = state
                .tasks
                .lock()
                .ok()
                .and_then(|t| t.get(&task_id).cloned());
            let grpc_task = task_resp.as_ref().map(convert_to_grpc_task);

            let resp = GrpcSendMessageResponse {
                task: grpc_task,
                message: None,
            };
            serde_json::to_vec(&resp).map_err(|e| format!("json: {e}"))
        }

        GrpcMethod::GetTask => {
            let req: GrpcGetTaskRequest =
                serde_json::from_slice(payload).map_err(|e| format!("parse: {e}"))?;
            let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
            let task = tasks
                .get(&req.id)
                .ok_or_else(|| format!("task not found: {}", req.id))?;
            let grpc_task = convert_to_grpc_task(task);
            serde_json::to_vec(&grpc_task).map_err(|e| format!("json: {e}"))
        }

        GrpcMethod::CancelTask => {
            let req: GrpcCancelTaskRequest =
                serde_json::from_slice(payload).map_err(|e| format!("parse: {e}"))?;
            let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
            let task = tasks
                .get_mut(&req.id)
                .ok_or_else(|| format!("task not found: {}", req.id))?;
            task.status = TaskState::Canceled;
            let grpc_task = convert_to_grpc_task(task);
            serde_json::to_vec(&grpc_task).map_err(|e| format!("json: {e}"))
        }

        GrpcMethod::ListTasks => {
            let _req: GrpcListTasksRequest =
                serde_json::from_slice(payload).map_err(|e| format!("parse: {e}"))?;
            let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
            let grpc_tasks: Vec<_> = tasks.values().map(convert_to_grpc_task).collect();
            let resp = GrpcListTasksResponse {
                tasks: grpc_tasks,
                next_page_token: None,
            };
            serde_json::to_vec(&resp).map_err(|e| format!("json: {e}"))
        }

        GrpcMethod::SubscribeToTask => {
            let req: GrpcGetTaskRequest =
                serde_json::from_slice(payload).map_err(|e| format!("parse: {e}"))?;

            let _rx = {
                let mut channels = state.event_channels.lock().map_err(|e| e.to_string())?;
                let tx = channels
                    .entry(req.id.clone())
                    .or_insert_with(|| {
                        let (new_tx, _) = tokio::sync::broadcast::channel(64);
                        new_tx
                    })
                    .clone();
                tx.subscribe()
            };

            let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
            let task = tasks
                .get(&req.id)
                .cloned()
                .ok_or_else(|| format!("task not found: {}", req.id))?;
            drop(tasks);

            let stream_resp = GrpcStreamResponse {
                task: Some(convert_to_grpc_task(&task)),
                message: None,
                status_update: None,
                artifact_update: None,
            };
            serde_json::to_vec(&stream_resp).map_err(|e| format!("json: {e}"))
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::net::SocketAddr;

    use super::check_grpc_http_request;
    use crate::neotrix::nt_agent_protocol::a2a_auth::{A2AAuthConfig, RateLimiter};

    fn make_auth_config(api_key: Option<&str>) -> A2AAuthConfig {
        A2AAuthConfig {
            api_key: api_key.map(|s| s.to_string()),
            rate_limit_per_min: 5,
            max_request_size: 1024,
            max_concurrent_tasks_per_session: 10,
        }
    }

    // ── check_grpc_http_request ───────────────────────────────────────────

    #[test]
    fn test_check_request_no_auth_config() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(60);
        let headers = HashMap::new();
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
    }

    #[test]
    fn test_check_request_with_valid_bearer() {
        let config = make_auth_config(Some("sk-secret"));
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "Bearer sk-secret".into());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
    }

    #[test]
    fn test_check_request_with_invalid_bearer() {
        let config = make_auth_config(Some("sk-secret"));
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "Bearer wrong-key".into());
        let err = check_grpc_http_request(&headers, &config, &mut rl, None);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().0, 401);
    }

    #[test]
    fn test_check_request_missing_bearer() {
        let config = make_auth_config(Some("sk-secret"));
        let mut rl = RateLimiter::new(60);
        let headers = HashMap::new();
        let err = check_grpc_http_request(&headers, &config, &mut rl, None);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().0, 401);
    }

    #[test]
    fn test_check_request_bearer_prefix_mismatch() {
        let config = make_auth_config(Some("sk-secret"));
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "sk-secret".into());
        let err = check_grpc_http_request(&headers, &config, &mut rl, None);
        assert!(err.is_err());
    }

    #[test]
    fn test_check_request_content_length_ok() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("content-length".into(), "500".into());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
    }

    #[test]
    fn test_check_request_content_length_too_large() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("content-length".into(), "2000".into());
        let err = check_grpc_http_request(&headers, &config, &mut rl, None);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().0, 413);
    }

    #[test]
    fn test_check_request_content_length_unparseable() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(60);
        let mut headers = HashMap::new();
        headers.insert("content-length".into(), "not-a-number".into());
        // Should pass because parse fails, so the check is skipped
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
    }

    #[test]
    fn test_check_request_rate_limit_exceeded() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(2);
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let headers = HashMap::new();
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(addr)).is_ok());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(addr)).is_ok());
        let err = check_grpc_http_request(&headers, &config, &mut rl, Some(addr));
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().0, 429);
    }

    #[test]
    fn test_check_request_rate_limiting_independent_ips() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(1);
        let a: SocketAddr = "10.0.0.1:1111".parse().unwrap();
        let b: SocketAddr = "10.0.0.2:2222".parse().unwrap();
        let headers = HashMap::new();
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(a)).is_ok());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(b)).is_ok());
        // Both IPs get 1 call each — only the next call should be blocked
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(b)).is_err());
    }

    #[test]
    fn test_check_request_auth_and_size_and_rate_combined() {
        let config = make_auth_config(Some("key"));
        let mut rl = RateLimiter::new(3);
        let addr: SocketAddr = "10.0.0.1:3333".parse().unwrap();
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "Bearer key".into());
        headers.insert("content-length".into(), "100".into());
        // All checks should pass
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(addr)).is_ok());
    }

    #[test]
    fn test_check_request_auth_failure_before_rate_limit() {
        // Auth is checked before rate limit, so wrong key = 401, not 429
        let config = make_auth_config(Some("key"));
        let mut rl = RateLimiter::new(1);
        let addr: SocketAddr = "10.0.0.1:4444".parse().unwrap();
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "Bearer wrong".into());
        let err = check_grpc_http_request(&headers, &config, &mut rl, Some(addr));
        assert_eq!(err.unwrap_err().0, 401);
    }

    #[test]
    fn test_check_request_valid_auth_uses_rate_quota() {
        // Valid auth should consume a rate limit slot
        let config = make_auth_config(Some("key"));
        let mut rl = RateLimiter::new(1);
        let addr: SocketAddr = "10.0.0.1:5555".parse().unwrap();
        let mut headers = HashMap::new();
        headers.insert("authorization".into(), "Bearer key".into());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, Some(addr)).is_ok());
        // Second request should be rate limited
        let err = check_grpc_http_request(&headers, &config, &mut rl, Some(addr));
        assert_eq!(err.unwrap_err().0, 429);
    }

    #[test]
    fn test_check_request_no_peer_addr_skips_rate_limit() {
        let config = make_auth_config(None);
        let mut rl = RateLimiter::new(1);
        let headers = HashMap::new();
        // Without peer addr, rate limiter is not invoked
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
        assert!(check_grpc_http_request(&headers, &config, &mut rl, None).is_ok());
    }
}
