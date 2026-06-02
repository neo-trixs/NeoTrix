//! KB Web API — axum handlers for KnowledgeBase queries
//!
//! Provides REST endpoints for searching, querying, and managing the
//! KnowledgeBase via a dedicated axum router. Merge into the main
//! server router with `build_kb_router()`.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::{KnowledgeBase, SearchResult, SearchMatchType};

/// Shared state for KB API handlers
#[derive(Clone)]
pub struct KbApiState {
    pub kb: Arc<Mutex<KnowledgeBase>>,
}

impl KbApiState {
    /// Open the default KB at ~/.neotrix/knowledge.db
    pub fn try_open_default() -> Option<Self> {
        let kb = KnowledgeBase::open(None).ok()?;
        Some(Self { kb: Arc::new(Mutex::new(kb)) })
    }
}

/// Build a standalone axum Router for all KB API routes.
pub fn build_kb_router(state: KbApiState) -> Router {
    Router::new()
        .route("/api/kb/search", get(search_handler))
        .route("/api/kb/node/{id}", get(node_handler))
        .route("/api/kb/stats", get(stats_handler))
        .route("/api/kb/query", post(advanced_query_handler))
        .route("/api/kb/graph/{node_id}", get(graph_handler))
        .route("/api/kb/e8/{mode}", get(e8_query_handler))
        .route("/api/kb/specialist/{name}", get(specialist_query_handler))
        .route("/api/kb/node", post(create_node_handler))
        .route("/api/kb/edge", post(create_edge_handler))
        .with_state(state)
}

// ─── Helper functions ───

fn json_ok<T: Serialize>(v: T) -> Json<serde_json::Value> {
    Json(serde_json::json!(v))
}

fn json_err(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})))
}

fn internal_err(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": msg})))
}

// ─── Query Parameter Types ───

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct GraphParams {
    depth: Option<usize>,
}

#[derive(Deserialize)]
pub struct AdvancedQueryBody {
    pub text: Option<String>,
    pub node_type: Option<String>,
    pub domain: Option<String>,
    pub min_importance: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateNodeBody {
    pub title: String,
    pub node_type: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateEdgeBody {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: Option<f64>,
    pub description: Option<String>,
}

// ─── Handlers ───

/// GET /api/kb/search?q=<query>&limit=10
pub async fn search_handler(
    State(state): State<KbApiState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let limit = params.limit.unwrap_or(10).min(100);
    let results = kb.search(&params.q, limit).map_err(|e| internal_err(&e))?;
    Ok(json_ok(results))
}

/// GET /api/kb/node/<id>
pub async fn node_handler(
    State(state): State<KbApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    match kb.get_node(&id).map_err(|e| internal_err(&e))? {
        Some(node) => Ok(json_ok(node)),
        None => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Node not found"})))),
    }
}

/// GET /api/kb/stats
pub async fn stats_handler(
    State(state): State<KbApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let stats = kb.stats().map_err(|e| internal_err(&e))?;
    Ok(json_ok(stats))
}

/// POST /api/kb/query — advanced query with text/type/domain filters
pub async fn advanced_query_handler(
    State(state): State<KbApiState>,
    Json(body): Json<AdvancedQueryBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let limit = body.limit.unwrap_or(10).min(100);

    let results: Vec<SearchResult> = if let Some(text) = &body.text {
        kb.hybrid_rerank_search(text, limit * 2).map_err(|e| internal_err(&e))?
    } else if let Some(nt_str) = &body.node_type {
        let nt = super::NodeType::from_str(nt_str);
        let nodes = kb.search_by_type(&nt, limit * 2).map_err(|e| internal_err(&e))?;
        nodes.into_iter().map(|n| SearchResult {
            node: n,
            score: 0.0,
            matched_on: vec![SearchMatchType::FtsTitle],
        }).collect()
    } else {
        return Err(json_err("At least one of 'text' or 'node_type' is required"));
    };

    let filtered: Vec<SearchResult> = results.into_iter()
        .filter(|r| {
            if let Some(domain) = &body.domain {
                r.node.domain.as_deref() == Some(domain.as_str())
            } else {
                true
            }
        })
        .filter(|r| {
            if let Some(min_imp) = body.min_importance {
                r.node.importance >= min_imp
            } else {
                true
            }
        })
        .collect();

    let offset = body.offset.unwrap_or(0);
    let paginated: Vec<SearchResult> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(json_ok(paginated))
}

/// GET /api/kb/graph/<node_id>?depth=2 — subgraph around a node
pub async fn graph_handler(
    State(state): State<KbApiState>,
    Path(node_id): Path<String>,
    Query(params): Query<GraphParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let depth = params.depth.unwrap_or(2).min(5);
    let (nodes, edges) = kb.subgraph(&node_id, depth).map_err(|e| internal_err(&e))?;
    Ok(json_ok(serde_json::json!({"nodes": nodes, "edges": edges})))
}

/// GET /api/kb/e8/<mode>?limit=10 — query by E8 reasoning mode
pub async fn e8_query_handler(
    State(state): State<KbApiState>,
    Path(mode): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let limit = params.limit.unwrap_or(10).min(100);
    let results = kb.recommend_for_e8_mode(&mode, limit).map_err(|e| internal_err(&e))?;
    Ok(json_ok(results))
}

/// GET /api/kb/specialist/<name>?limit=10 — query by GWT specialist module
pub async fn specialist_query_handler(
    State(state): State<KbApiState>,
    Path(name): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let limit = params.limit.unwrap_or(10).min(100);
    let results = kb.hybrid_rerank_search(&name, limit).map_err(|e| internal_err(&e))?;
    Ok(json_ok(results))
}

/// POST /api/kb/node — create a new knowledge node
pub async fn create_node_handler(
    State(state): State<KbApiState>,
    Json(body): Json<CreateNodeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let nt = super::NodeType::from_str(&body.node_type);
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let id = kb.insert_or_get_node(
        &body.title,
        nt,
        body.summary.as_deref(),
        body.url.as_deref(),
        body.domain.as_deref(),
    ).map_err(|e| internal_err(&e))?;
    Ok(json_ok(serde_json::json!({"id": id})))
}

/// POST /api/kb/edge — create a new edge between nodes
pub async fn create_edge_handler(
    State(state): State<KbApiState>,
    Json(body): Json<CreateEdgeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let rt = super::RelationType::from_str(&body.relation_type);
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    kb.upsert_edge(
        &body.source_id,
        &body.target_id,
        rt,
        body.weight.unwrap_or(1.0),
        body.description.as_deref(),
    ).map_err(|e| internal_err(&e))?;
    Ok(json_ok(serde_json::json!({"created": true})))
}
