use std::collections::HashMap;
use std::sync::Arc;

use agent_core::card::{AgentCard, SkillDecl};
use agent_core::registry::RegistryClient;
use agent_core::task::{
    A2AMessage, A2APart, A2APartType, A2ATask, NegotiationOffer, NegotiationResponse,
    SendTaskRequest, SendTaskResponse, TaskState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        Json, Sse,
    },
    routing::{get, post},
    Router,
};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::engine;
use crate::state::Store;

// ── Shared State ───────────────────────────────────────

pub struct AppContext {
    pub config: Config,
    pub state_store: Store,
    pub tasks: RwLock<HashMap<String, A2ATask>>,
    pub registry_client: Option<RegistryClient>,
    pub http_client: reqwest::Client,
}

// ── Agent Card Construction ─────────────────────────────

pub fn build_agent_card(config: &crate::config::Config) -> AgentCard {
    AgentCard::new(
        &config.agent_name,
        "Personal IP publishing agent — analyze, synthesize, publish, track.",
        &format!("http://0.0.0.0:{}", config.http_port),
        &config.agent_version,
        vec![
            SkillDecl {
                id: "open-source-analysis".into(),
                name: "Open-Source Project Analysis".into(),
                description: "Analyze a GitHub repo: architecture patterns, gaps, synthesis.".into(),
                tags: vec!["analysis".into(), "pattern-extraction".into()],
                examples: vec!["analyze https://github.com/user/repo".into()],
            },
            SkillDecl {
                id: "content-generation".into(),
                name: "Cross-Platform Content Generation".into(),
                description: "Generate platform-native content from analysis or topic.".into(),
                tags: vec!["content".into(), "writing".into()],
                examples: vec!["generate content for x about topic Y".into()],
            },
            SkillDecl {
                id: "publishing".into(),
                name: "Multi-Platform Publishing".into(),
                description: "Publish content to Chinese and Western platforms.".into(),
                tags: vec!["publishing".into(), "social-media".into()],
                examples: vec!["publish to x,zhihu the content".into()],
            },
            SkillDecl {
                id: "geo-audit".into(),
                name: "GEO Visibility Audit".into(),
                description: "Check AI citability score for a URL.".into(),
                tags: vec!["geo".into(), "audit".into()],
                examples: vec!["geo audit https://github.com/user/repo".into()],
            },
            SkillDecl {
                id: "status".into(),
                name: "Agent Status & KPI".into(),
                description: "Return current state, history, and KPIs.".into(),
                tags: vec!["monitoring".into()],
                examples: vec!["status".into()],
            },
        ],
    )
}

// ── Handlers ───────────────────────────────────────────

async fn agent_card_handler(
    State(ctx): State<Arc<AppContext>>,
) -> Json<AgentCard> {
    Json(build_agent_card(&ctx.config))
}

async fn negotiate_handler(
    State(_ctx): State<Arc<AppContext>>,
    Json(offer): Json<NegotiationOffer>,
) -> Json<NegotiationResponse> {
    let selected = offer.versions.first().cloned().unwrap_or("1.0".into());
    Json(NegotiationResponse {
        selected_version: selected,
        negotiation_id: offer.negotiation_id,
        accepted: true,
    })
}

async fn send_task_handler(
    State(ctx): State<Arc<AppContext>>,
    Json(req): Json<SendTaskRequest>,
) -> Json<SendTaskResponse> {
    let task_id = req.id.clone();
    let initial_task = A2ATask {
        id: task_id.clone(),
        session_id: req.session_id.clone(),
        status: TaskState::Submitted,
        messages: req.messages.clone(),
        artifacts: Vec::new(),
        metadata: req.metadata.clone(),
    };
    ctx.tasks.write().await.insert(task_id.clone(), initial_task.clone());

    let instruction = req
        .messages
        .first()
        .and_then(|m| m.parts.first())
        .and_then(|p| p.text.clone())
        .unwrap_or_default()
        .trim()
        .to_string();

    // Process in background — handler returns immediately
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        let result = process_instruction(&ctx_clone, &instruction).await;
        let (status, response_msg) = match result {
            Ok(text) => (TaskState::Completed, text),
            Err(e) => (TaskState::Failed, format!("Error: {e}")),
        };
        let mut tasks = ctx_clone.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.status = status;
            task.messages.push(A2AMessage {
                role: "agent".into(),
                parts: vec![A2APart {
                    r#type: A2APartType::Text,
                    text: Some(response_msg),
                    data: None,
                }],
            });
        }
    });

    Json(SendTaskResponse { task: initial_task })
}

async fn get_task_handler(
    State(ctx): State<Arc<AppContext>>,
    Path(task_id): Path<String>,
) -> Result<Json<A2ATask>, StatusCode> {
    let tasks = ctx.tasks.read().await;
    tasks.get(&task_id).cloned().ok_or(StatusCode::NOT_FOUND).map(Json)
}

async fn stream_task_handler(
    State(_ctx): State<Arc<AppContext>>,
    Path(task_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = futures::stream::once(async move {
        Ok(Event::default()
            .data(format!("{{ \"event\": \"status\", \"taskId\": \"{task_id}\" }}")))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ── Instruction Router ─────────────────────────────────

pub(crate) async fn process_instruction(ctx: &Arc<AppContext>, instruction: &str) -> Result<String, String> {
    let instruction = instruction.to_lowercase();

    if instruction.starts_with("analyze ") || instruction.starts_with("分析") {
        let url = instruction
            .strip_prefix("analyze ")
            .or_else(|| instruction.strip_prefix("分析"))
            .unwrap_or("")
            .trim();
        if url.is_empty() {
            return Err("provide a GitHub URL: analyze https://github.com/user/repo".into());
        }
        let (patterns, capabilities, angles, report_path) =
            engine::analyze_repo(&ctx.config, url, &ctx.http_client).await?;
        let repo_name = url.trim_end_matches('/').rsplit('/').next().unwrap_or(url);
        ctx.state_store
            .record_analysis(url, repo_name, patterns.clone(), capabilities.clone(), angles, report_path)
            .await;
        Ok(format!(
            "Analysis complete for {url}\nPatterns: {}\nCapabilities: {}",
            patterns.len(),
            capabilities.len()
        ))
    } else if instruction.starts_with("generate ") || instruction.starts_with("生成") {
        let rest = instruction
            .strip_prefix("generate ")
            .or_else(|| instruction.strip_prefix("生成"))
            .unwrap_or("");
        let parts: Vec<&str> = rest.split(" about ").collect();
        if parts.len() < 2 {
            return Err("usage: generate content for <platform> about <topic>".into());
        }
        let platform = parts[0].trim();
        let topic = parts[1].trim();
        let content = engine::generate_content(&ctx.config, topic, platform, &ctx.http_client).await?;
        Ok(content)
    } else if instruction.starts_with("publish ") || instruction.starts_with("发布") {
        let rest = instruction
            .strip_prefix("publish to ")
            .or_else(|| instruction.strip_prefix("发布到"))
            .unwrap_or("");
        let parts: Vec<&str> = rest.splitn(2, " the ").collect();
        if parts.len() < 2 {
            return Err("usage: publish to <platform> the <content>".into());
        }
        let platform = parts[0].trim();
        let content = parts[1].trim();
        let result = engine::publish_to_platform(&ctx.config, platform, content, &ctx.http_client).await?;
        ctx.state_store.record_publication(platform, &content[..content.len().min(80)], None, "published").await;
        Ok(format!("Published to {platform}: {result}"))
    } else if instruction.starts_with("geo") {
        Ok("GEO audit: run manually:\n  uvx --from geo-optimizer-skill geo audit --url <url>\n(automated audit coming in v1.1)".into())
    } else if instruction == "status" || instruction == "状态" {
        let state = ctx.state_store.read().await;
        Ok(format!(
            "GhostMVP v{} | {} analyses | {} publications | {} patterns | {} capabilities",
            ctx.config.agent_version,
            state.total_analyses,
            state.total_publications,
            state.total_patterns,
            state.total_capabilities,
        ))
    } else if instruction.starts_with("help") || instruction == "?" || instruction == "帮助" {
        Ok([
            "GhostMVP A2A Agent — available commands:",
            "  analyze <url>         — analyze a GitHub repository",
            "  generate <platform> about <topic> — generate content",
            "  publish to <platform> the <content> — publish content",
            "  geo <url>             — GEO audit a URL",
            "  status                — agent status and KPIs",
            "  help                  — this message",
        ]
        .join("\n"))
    } else {
        Ok(format!("Unknown command: {instruction}\nType 'help' for available commands."))
    }
}

// ── Router ─────────────────────────────────────────────

pub fn build_router(ctx: Arc<AppContext>) -> Router {
    Router::new()
        .route("/.well-known/agent-card", get(agent_card_handler))
        .route("/.well-known/negotiate", post(negotiate_handler))
        .route("/tasks/send", post(send_task_handler))
        .route("/tasks/{task_id}", get(get_task_handler))
        .route("/tasks/{task_id}/stream", get(stream_task_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(ctx)
}
