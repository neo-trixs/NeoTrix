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

use super::{
    diff_snapshots, kb_write_guard, nt_memory_embed, record_write_evidence, snapshot_from_file,
    snapshot_kb, WriteGuardVerdict, KnowledgeBase, SearchResult, SearchMatchType,
};

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
        .route("/api/kb/embeddings/status", get(embeddings_status_handler))
        .route("/api/kb/embeddings/backfill", post(embeddings_backfill_handler))
        .route("/api/kb/snapshot", get(snapshot_handler))
        .route("/api/kb/diff", post(diff_handler))
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

/// 写前门 (G3): 运行确定性 kb_write_guard。Reject → 400 (含拒绝原因);
/// RequiresApproval → 403 (人工审批); Allow → Ok(verdict)。裁决落证据到 kv_store。
async fn gate_write(
    state: &KbApiState,
    action: &str,
    payload: &serde_json::Value,
) -> Result<WriteGuardVerdict, (StatusCode, Json<serde_json::Value>)> {
    let verdict = kb_write_guard(action, payload);
    match &verdict {
        WriteGuardVerdict::Allow => {}
        WriteGuardVerdict::RequiresApproval => {
            if let Ok(kb) = state.kb.lock() {
                record_write_evidence(&kb, action, payload, &verdict, false);
            }
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": format!(
                    "{} 需要人工审批 (RequiresApproval)",
                    action
                )})),
            ));
        }
        WriteGuardVerdict::Reject(reasons) => {
            if let Ok(kb) = state.kb.lock() {
                record_write_evidence(&kb, action, payload, &verdict, false);
            }
            return Err(json_err(&format!("{} 拒绝: {}", action, reasons.join("; "))));
        }
    }
    Ok(verdict)
}

// ─── Query Parameter Types ───

#[derive(Deserialize)]
pub(crate) struct SearchParams {
    q: String,
    limit: Option<usize>,
    /// 兼容字段：HTTP 公开面强制 Public clearance（C-2 加固），
    /// 客户端传入的 permission 被忽略——保留字段仅为 serde 兼容旧客户端。
    #[allow(dead_code)]
    permission: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct PaginationParams {
    limit: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct GraphParams {
    depth: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct AdvancedQueryBody {
    pub text: Option<String>,
    pub node_type: Option<String>,
    pub domain: Option<String>,
    pub min_importance: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct CreateNodeBody {
    pub title: String,
    pub node_type: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CreateEdgeBody {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: Option<f64>,
    pub description: Option<String>,
}

/// POST /api/kb/diff 请求体 — base_path/other_path 至少提供一个文件路径;
/// 缺省的一方使用当前实时库快照。
#[derive(Deserialize)]
pub(crate) struct DiffBody {
    pub base_path: Option<String>,
    pub other_path: Option<String>,
}

// ─── Handlers ───

/// GET /api/kb/search?q=<query>&limit=10
pub async fn search_handler(
    State(state): State<KbApiState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let limit = params.limit.unwrap_or(10).min(100);
    // 安全加固（C-2）：HTTP 公开搜索面强制 Public clearance，
    // 不接受客户端传入的 permission 参数——否则调用方可自选 "secret"
    // 读取 ThinkingTrace/Secret 等敏感节点。
    let permission = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public;
    let results = kb.search_permission_aware(&params.q, limit, permission).map_err(|e| internal_err(&e))?;
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
            signals: None,
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
    let payload = serde_json::json!({
        "title": body.title,
        "node_type": body.node_type,
        "summary": body.summary,
        "url": body.url,
        "domain": body.domain,
    });
    let verdict = gate_write(&state, "node:create", &payload).await?;
    let nt = super::NodeType::from_str(
        payload.get("node_type").and_then(|v| v.as_str()).unwrap_or(""),
    );
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let id = kb.insert_or_get_node(
        &body.title,
        nt,
        body.summary.as_deref(),
        body.url.as_deref(),
        body.domain.as_deref(),
    ).map_err(|e| {
        record_write_evidence(&kb, "node:create", &payload, &WriteGuardVerdict::Reject(vec![e.clone()]), false);
        internal_err(&e)
    })?;
    record_write_evidence(&kb, "node:create", &payload, &verdict, true);
    Ok(json_ok(serde_json::json!({"id": id})))
}

/// POST /api/kb/edge — create a new edge between nodes
pub async fn create_edge_handler(
    State(state): State<KbApiState>,
    Json(body): Json<CreateEdgeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let payload = serde_json::json!({
        "source_id": body.source_id,
        "target_id": body.target_id,
        "relation_type": body.relation_type,
        "weight": body.weight,
        "description": body.description,
    });
    let verdict = gate_write(&state, "edge:upsert", &payload).await?;
    let rt = super::RelationType::from_str(
        payload.get("relation_type").and_then(|v| v.as_str()).unwrap_or(""),
    );
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    kb.upsert_edge(
        &body.source_id,
        &body.target_id,
        rt,
        body.weight.unwrap_or(1.0),
        body.description.as_deref(),
    ).map_err(|e| {
        record_write_evidence(&kb, "edge:upsert", &payload, &WriteGuardVerdict::Reject(vec![e.clone()]), false);
        internal_err(&e)
    })?;
    record_write_evidence(&kb, "edge:upsert", &payload, &verdict, true);
    Ok(json_ok(serde_json::json!({"created": true})))
}

/// GET /api/kb/embeddings/status — report embedding provider config + corpus state
///
/// Convenience/introspection endpoint mirroring the "provider=lite" responsibilities of
/// `scripts/kb-embed-server.py`. Real inference stays external (MiniLM server); this just
/// surfaces reachability, dimension, and how far backfill has to go.
pub async fn embeddings_status_handler(
    State(state): State<KbApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (model, dimension, base_url, vector_count, missing) = {
        let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
        let config = kb.embedding_config.read()
            .map_err(|e| internal_err(&format!("embedding_config read: {}", e)))?
            .clone();
        let vectorized = {
            let conn = kb.conn.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
            nt_memory_embed::embedding_count(&conn).unwrap_or(0)
        };
        let missing = {
            let conn = kb.conn.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
            nt_memory_embed::find_nodes_missing_embeddings(&conn).unwrap_or_default().len()
        };
        match config {
            Some(c) => (c.model, c.dimension, c.base_url, vectorized, missing),
            None => ("(none)".to_string(), 0, "(not configured)".to_string(), vectorized, missing),
        }
    };

    // Cheap reachability probe on the configured provider, off the async runtime.
    let reachable = tokio::task::spawn_blocking({
        let base_url = base_url.clone();
        move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .ok();
            let Some(client) = client else { return false };
            client.get(format!("{base_url}/models")).send().map(|r| r.status().is_success()).unwrap_or(false)
        }
    }).await.unwrap_or(false);

    Ok(json_ok(serde_json::json!({
        "model": model,
        "dimension": dimension,
        "base_url": base_url,
        "provider_reachable": reachable,
        "vectors_stored": vector_count,
        "nodes_missing_vectors": missing,
    })))
}

/// POST /api/kb/embeddings/backfill — materialize missing node embeddings via the provider
///
/// Rust-side equivalent of `scripts/kb-embed-server.py`'s backfill loop. Requires an
/// embedding provider on `base_url` (see `EmbeddingConfig::default`). Runs the (blocking)
/// `ensure_embeddings` off the async runtime via `spawn_blocking`.
pub async fn embeddings_backfill_handler(
    State(state): State<KbApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // G3: 全量嵌入重建 = 批量重写, 一律 RequiresApproval (dbx high_risk_write 类)。
    let payload = serde_json::json!({"action": "embedding:backfill"});
    gate_write(&state, "embedding:backfill", &payload).await?;
    let kb = state.kb.clone();
    let processed = tokio::task::spawn_blocking(move || {
        kb.lock().map_err(|e| format!("Lock: {}", e))?.ensure_embeddings()
    })
        .await
        .map_err(|e| internal_err(&format!("Backfill task: {e}")))?
        .map_err(|e| internal_err(&e))?;
    Ok(json_ok(serde_json::json!({"processed": processed})))
}

/// GET /api/kb/snapshot — 返回当前 KB 全量快照 (只读, G5)。
pub async fn snapshot_handler(
    State(state): State<KbApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
    let snap = snapshot_kb(&kb).map_err(|e| internal_err(&e))?;
    Ok(json_ok(snap))
}

/// POST /api/kb/diff — 比较快照文件与/或当前库 (只读, G5)。
/// body: {"base_path": "snapA.json"?, "other_path": "snapB.json"?}
/// 双方皆缺省 → base=other=当前库, 恒空; 至少一个路径才有意义。
pub async fn diff_handler(
    State(state): State<KbApiState>,
    Json(body): Json<DiffBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let load = |path: &Option<String>| -> Result<Option<super::KbSnapshot>, (StatusCode, Json<serde_json::Value>)> {
        match path {
            Some(p) => snapshot_from_file(std::path::Path::new(p))
                .map(Some)
                .map_err(|e| internal_err(&e)),
            None => Ok(None),
        }
    };
    let base = match load(&body.base_path)? {
        Some(s) => s,
        None => {
            let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
            snapshot_kb(&kb).map_err(|e| internal_err(&e))?
        }
    };
    let other = match load(&body.other_path)? {
        Some(s) => s,
        None => {
            let kb = state.kb.lock().map_err(|e| internal_err(&format!("Lock: {}", e)))?;
            snapshot_kb(&kb).map_err(|e| internal_err(&e))?
        }
    };
    let diff = diff_snapshots(&base, &other);
    Ok(json_ok(serde_json::json!({
        "base_nodes": base.nodes.len(),
        "base_edges": base.edges.len(),
        "other_nodes": other.nodes.len(),
        "other_edges": other.edges.len(),
        "diff": diff,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_kb() -> (KbApiState, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("nt_kb_api_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let db_path = dir.join(format!("test_api_kb_{}.db", std::thread::current().name().unwrap_or("t")));
        let kb = KnowledgeBase::open(Some(db_path.clone())).expect("open kb");
        (KbApiState { kb: Arc::new(Mutex::new(kb)) }, db_path)
    }

    #[test]
    fn test_embeddings_status_reports_config() {
        let (state, _) = temp_kb();
        // Config present → status reports model/dim without panicking even if no provider up.
        {
            let kb = state.kb.lock().expect("lock");
            *kb.embedding_config.write().expect("rwlock") = Some(nt_memory_embed::EmbeddingConfig {
                api_key: "local".into(),
                base_url: "http://127.0.0.1:8237/v1".into(),
                model: "all-MiniLM-L6-v2".into(),
                dimension: 384,
                mode: nt_memory_embed::EmbedMode::Http,
            });
        }
        let body = futures_block_on(embeddings_status_handler(State(state))).expect("status ok");
        let v = body.0;
        assert_eq!(v["model"], "all-MiniLM-L6-v2");
        assert_eq!(v["dimension"], 384);
        assert_eq!(v["base_url"], "http://127.0.0.1:8237/v1");
        // No nodes, no vectors — reachability just reports false (no provider).
        assert_eq!(v["vectors_stored"], 0);
        assert_eq!(v["nodes_missing_vectors"], 0);
    }

    #[test]
    fn test_embeddings_status_without_config() {
        let (state, _) = temp_kb();
        let body = futures_block_on(embeddings_status_handler(State(state))).expect("status ok");
        assert_eq!(body.0["model"], "(none)");
        assert_eq!(body.0["provider_reachable"], false);
    }

    #[test]
    fn test_embeddings_backfill_without_provider_is_noop() {
        let (state, _) = temp_kb();
        // G3 gate: embedding:backfill 一律 RequiresApproval → 403, 不执行。
        let err = futures_block_on(embeddings_backfill_handler(State(state))).unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_create_node_gate_rejects_bad_payload() {
        let (state, _) = temp_kb();
        // 空 title → guard Reject → 400, 不写库。
        let body = CreateNodeBody {
            title: "  ".into(),
            node_type: "concept".into(),
            summary: None,
            url: Some("ftp://x".into()),
            domain: None,
        };
        let err = futures_block_on(create_node_handler(State(state.clone()), Json(body))).unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        // 库应保持空 (无节点写入)。
        let kb = state.kb.lock().unwrap();
        let stats = kb.stats().unwrap();
        assert_eq!(stats.total_nodes, 0);
    }

    #[test]
    fn test_create_edge_gate_rejects_self_loop() {
        let (state, _) = temp_kb();
        let body = CreateEdgeBody {
            source_id: "a".into(),
            target_id: "a".into(),
            relation_type: "related".into(),
            weight: Some(1.0),
            description: None,
        };
        let err = futures_block_on(create_edge_handler(State(state), Json(body))).unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_snapshot_handler_reports_current_kb() {
        let (state, _) = temp_kb();
        // 写入一个节点后, 快照应包含该节点。
        {
            let kb = state.kb.lock().unwrap();
            kb.insert_or_get_node("G5 snap", crate::neotrix::nt_memory_kb::NodeType::Concept, None, None, None).unwrap();
        }
        let body = futures_block_on(snapshot_handler(State(state))).expect("snapshot ok");
        let v = body.0;
        assert_eq!(v["format"], crate::neotrix::nt_memory_kb::SNAPSHOT_FORMAT);
        assert_eq!(v["nodes"].as_array().map(|a| a.len()).unwrap_or(0), 1);
        assert_eq!(v["edges"].as_array().map(|a| a.len()).unwrap_or(0), 0);
    }

    #[test]
    fn test_diff_handler_against_self_is_empty() {
        let (state, _) = temp_kb();
        let body = futures_block_on(diff_handler(State(state), Json(DiffBody { base_path: None, other_path: None })))
            .expect("diff ok");
        assert_eq!(body.0["diff"]["nodes_added"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_diff_handler_file_vs_current() {
        let (state, _) = temp_kb();
        // 空库 → 快照文件 (含 1 节点) → 删除后 diff 应报告 removed 1。
        let node_id = {
            let kb = state.kb.lock().unwrap();
            kb.insert_or_get_node("G5 file", crate::neotrix::nt_memory_kb::NodeType::Concept, None, None, None)
                .unwrap()
        };
        let snap = {
            let kb = state.kb.lock().unwrap();
            snapshot_kb(&kb).unwrap()
        };
        let dir = std::env::temp_dir().join(format!("nt_kb_api_snap_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("snap.json");
        crate::neotrix::nt_memory_kb::snapshot_to_file(&snap, &path).unwrap();
        // 删除节点 → 库变空, 与文件快照对比应报告 removed 1。
        {
            let kb = state.kb.lock().unwrap();
            let deleted = kb.delete_node(&node_id).unwrap();
            assert!(deleted, "删除应生效");
        }
        let body = futures_block_on(diff_handler(
            State(state),
            Json(DiffBody { base_path: Some(path.to_string_lossy().into_owned()), other_path: None }),
        ))
        .expect("diff ok");
        assert_eq!(body.0["diff"]["nodes_removed"].as_array().unwrap().len(), 1);
    }

    fn futures_block_on<F: std::future::Future>(fut: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(fut)
    }
}
