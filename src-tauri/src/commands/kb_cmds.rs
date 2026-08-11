use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use tauri::command;
use rusqlite::params;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_memory_kb::nt_memory_store;
use neotrix::neotrix::nt_memory_kb::nt_memory_store::{get_all_nodes, get_all_edges};
use neotrix::neotrix::nt_memory_kb::nt_memory_types::{KnowledgeNode, KnowledgeEdge};

fn kb_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".neotrix").join("knowledge.db")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbNode {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub confidence: f64,
    pub importance: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbSearchResult {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbGraphResponse {
    pub nodes: Vec<KbNode>,
    pub edges: Vec<KbEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbStatsResponse {
    pub total_nodes: i64,
    pub total_edges: i64,
    pub by_type: Vec<(String, i64)>,
}

fn map_node(n: &KnowledgeNode) -> KbNode {
    KbNode {
        id: n.id.clone(),
        node_type: n.node_type.as_str().to_string(),
        title: n.title.clone(),
        summary: n.summary.clone(),
        content: n.content.clone(),
        url: n.url.clone(),
        domain: n.domain.clone(),
        confidence: n.confidence,
        importance: n.importance,
        metadata: n.metadata.clone(),
    }
}

fn map_edge(e: &KnowledgeEdge) -> KbEdge {
    KbEdge {
        id: e.id.clone(),
        source_id: e.source_id.clone(),
        target_id: e.target_id.clone(),
        relation_type: e.relation_type.as_str().to_string(),
        weight: e.weight,
    }
}

#[command]
pub fn get_knowledge_graph() -> Result<KbGraphResponse, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let nodes = get_all_nodes(&conn).map_err(|e| NeoTrixError::Memory(format!("get_all_nodes: {}", e)))?;
    let edges = get_all_edges(&conn).map_err(|e| NeoTrixError::Memory(format!("get_all_edges: {}", e)))?;
    Ok(KbGraphResponse {
        nodes: nodes.iter().map(map_node).collect(),
        edges: edges.iter().map(map_edge).collect(),
    })
}

#[command]
pub fn get_knowledge_stats() -> Result<KbStatsResponse, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let stats = nt_memory_store::get_stats(&conn)
        .map_err(|e| NeoTrixError::Memory(format!("stats: {}", e)))?;
    Ok(KbStatsResponse {
        total_nodes: stats.total_nodes,
        total_edges: stats.total_edges,
        by_type: stats.by_type,
    })
}

#[command]
pub fn kb_search(query: String, limit: Option<usize>) -> Result<Vec<KbSearchResult>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let limit = limit.unwrap_or(10);
    let mut stmt = conn.prepare(
        "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain, n.confidence, n.importance, n.created_at
         FROM nodes n
         INNER JOIN nodes_fts fts ON fts.rowid = n.rowid
         WHERE nodes_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    ).map_err(|e| NeoTrixError::Memory(format!("search prep: {}", e)))?;
    let results = stmt.query_map(params![query, limit as i64], |row| {
        Ok(KbSearchResult {
            id: row.get(0)?,
            node_type: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            confidence: row.get(7)?,
            importance: row.get(8)?,
            created_at: row.get(9)?,
        })
    }).map_err(|e| NeoTrixError::Memory(format!("search query: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();
    Ok(results)
}

#[command]
pub fn kb_get_node(id: String) -> Result<Option<KbNode>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, confidence, importance, metadata FROM nodes WHERE id = ?1"
    ).map_err(|e| NeoTrixError::Memory(format!("get_node prep: {}", e)))?;
    let result = stmt.query_row(params![id], |row| {
        Ok(KbNode {
            id: row.get(0)?,
            node_type: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            confidence: row.get(7)?,
            importance: row.get(8)?,
            metadata: row.get::<_, Option<String>>(9)?.and_then(|s| serde_json::from_str(&s).ok()),
        })
    });
    match result {
        Ok(node) => Ok(Some(node)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(NeoTrixError::Memory(format!("get_node: {}", e))),
    }
}

#[command]
pub fn kb_get_related(id: String, relation_type: Option<String>, limit: Option<usize>) -> Result<Vec<KbSearchResult>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let limit = limit.unwrap_or(10);
    let results: Vec<KbSearchResult> = if let Some(rt) = relation_type {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain, n.confidence, n.importance, n.created_at
             FROM nodes n
             INNER JOIN edges e ON (e.source_id = n.id OR e.target_id = n.id)
             WHERE (e.source_id = ?1 OR e.target_id = ?1) AND n.id != ?1 AND e.relation_type = ?3
             ORDER BY e.weight DESC
             LIMIT ?2"
        ).map_err(|e| NeoTrixError::Memory(format!("get_related prep: {}", e)))?;
        let rows = stmt.query_map(params![id, limit as i64, rt], |row| {
            Ok(KbSearchResult {
                id: row.get(0)?, node_type: row.get(1)?, title: row.get(2)?,
                summary: row.get(3)?, content: row.get(4)?, url: row.get(5)?,
                domain: row.get(6)?, confidence: row.get(7)?, importance: row.get(8)?, created_at: row.get(9)?,
            })
        }).map_err(|e| NeoTrixError::Memory(format!("get_related query: {}", e)))?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain, n.confidence, n.importance, n.created_at
             FROM nodes n
             INNER JOIN edges e ON (e.source_id = n.id OR e.target_id = n.id)
             WHERE (e.source_id = ?1 OR e.target_id = ?1) AND n.id != ?1
             ORDER BY e.weight DESC
             LIMIT ?2"
        ).map_err(|e| NeoTrixError::Memory(format!("get_related prep: {}", e)))?;
        let rows = stmt.query_map(params![id, limit as i64], |row| {
            Ok(KbSearchResult {
                id: row.get(0)?, node_type: row.get(1)?, title: row.get(2)?,
                summary: row.get(3)?, content: row.get(4)?, url: row.get(5)?,
                domain: row.get(6)?, confidence: row.get(7)?, importance: row.get(8)?, created_at: row.get(9)?,
            })
        }).map_err(|e| NeoTrixError::Memory(format!("get_related query: {}", e)))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    Ok(results)
}

#[command]
pub fn kb_feed(limit: Option<usize>, offset: Option<usize>, sort: Option<String>) -> Result<Vec<KbSearchResult>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let order = match sort.as_deref() {
        Some("confidence") => "n.confidence DESC, n.created_at DESC",
        Some("importance") => "n.importance DESC, n.created_at DESC",
        _ => "n.created_at DESC, n.confidence DESC",
    };
    let sql = format!(
        "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain, n.confidence, n.importance, n.created_at
         FROM nodes n
         ORDER BY {}
         LIMIT ?1 OFFSET ?2", order
    );
    let mut stmt = conn.prepare(&sql)
        .map_err(|e| NeoTrixError::Memory(format!("feed prep: {}", e)))?;
    let results = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(KbSearchResult {
            id: row.get(0)?, node_type: row.get(1)?, title: row.get(2)?,
            summary: row.get(3)?, content: row.get(4)?, url: row.get(5)?,
            domain: row.get(6)?, confidence: row.get(7)?, importance: row.get(8)?, created_at: row.get(9)?,
        })
    }).map_err(|e| NeoTrixError::Memory(format!("feed query: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();
    Ok(results)
}

/// 地理索引记录 (geo_index 表) — 供前端 3D 地图渲染。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub country: String,
    pub region: String,
    pub city: String,
    pub tags: String,
    pub source: String,
    pub confidence: f64,
}

/// 导出地理索引点 (geo_index) — 前端地球知识世界仿真数据源。
/// `source` 可选过滤：如 "shanhai" 只返回 shanhai-peaks + shanhai-mappings，
/// 供前端把幻境数据分层叠加在真实地图上（真实层取城市点，幻境层取全部 shanhai）。
#[command]
pub fn kb_geo_points(limit: Option<usize>, source: Option<String>) -> Result<Vec<GeoPoint>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    // limit 硬上限：防止前端误传超大值拉全量 117k 点拖垮 IPC
    let limit = (limit.unwrap_or(5000) as i64).clamp(1, 20_000);
    // stmt 提升到 match 外：Rows 借用 stmt，若在分支内创建会在分支末尾 drop
    let mut stmt = conn
        .prepare(match source.as_deref() {
            Some("shanhai") => {
                "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
                 FROM geo_index WHERE source IN ('shanhai-peaks', 'shanhai-mappings')
                 ORDER BY confidence DESC LIMIT ?1"
            }
            Some(_) => {
                "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
                 FROM geo_index WHERE source = ?1 ORDER BY confidence DESC LIMIT ?2"
            }
            None => {
                "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
                 FROM geo_index ORDER BY confidence DESC LIMIT ?1"
            }
        })
        .map_err(|e| NeoTrixError::Memory(format!("geo points prep: {}", e)))?;
    let rows = match source.as_deref() {
        Some("shanhai") | None => stmt
            .query_map(params![limit], map_geo_row)
            .map_err(|e| NeoTrixError::Memory(format!("geo points query: {}", e)))?,
        Some(s) => stmt
            .query_map(params![s, limit], map_geo_row)
            .map_err(|e| NeoTrixError::Memory(format!("geo points query: {}", e)))?,
    };
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn map_geo_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GeoPoint> {
    Ok(GeoPoint {
        node_id: row.get(0)?,
        lat: row.get(1)?,
        lng: row.get(2)?,
        country: row.get(3)?,
        region: row.get(4)?,
        city: row.get(5)?,
        tags: row.get(6)?,
        source: row.get(7)?,
        confidence: row.get(8)?,
    })
}

/// 地理索引统计。
#[command]
pub fn kb_geo_stats() -> Result<(i64, i64), NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM geo_index", [], |r| r.get(0))
        .map_err(|e| NeoTrixError::Memory(format!("geo stats: {}", e)))?;
    let with_country: i64 = conn
        .query_row("SELECT COUNT(*) FROM geo_index WHERE country != ''", [], |r| r.get(0))
        .map_err(|e| NeoTrixError::Memory(format!("geo stats country: {}", e)))?;
    Ok((total, with_country))
}

/// 地图分层摘要 — 各数据源计数。前端据此决定加载策略
/// （幻境层全量拉取，真实层按预算采样），实现前后端分离。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLayerSummary {
    pub source: String,
    pub count: i64,
}

/// 返回 geo_index 各 source 的计数（真实层/幻境层分层摘要）。
#[command]
pub fn kb_geo_layers() -> Result<Vec<GeoLayerSummary>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let mut stmt = conn
        .prepare("SELECT source, COUNT(*) FROM geo_index GROUP BY source ORDER BY COUNT(*) DESC")
        .map_err(|e| NeoTrixError::Memory(format!("geo layers prep: {}", e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(GeoLayerSummary {
                source: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| NeoTrixError::Memory(format!("geo layers query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
