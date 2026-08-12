use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use tauri::command;
use rusqlite::params;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_memory_kb::nt_memory_store;
use neotrix::neotrix::nt_memory_kb::nt_memory_store::{get_all_nodes, get_all_edges};
use neotrix::neotrix::nt_memory_kb::nt_memory_types::{KnowledgeNode, KnowledgeEdge};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::{self, PackDecoder};

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

/// NT-Pack 数据源路径: 全量归档 + 冷层归档。
/// 偏好 `~/.neotrix/geo/geo_index.ntpack` (全量) ; source 有冷层文件时优先冷层。
fn geo_pack_paths() -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".neotrix")
        .join("geo");
    (dir.join("geo_index.ntpack"), dir)
}

/// 进程级 NT-Pack 解码缓存 (单槽): key = 解析后路径 + mtime + len + TTL。
/// 避免 GlobeView 8 路并发各触发一次 7MB 全量 read+decode (约 50-60ms/次)。
struct GeoPackCache {
    key_file: PathBuf,
    key_len: u64,
    key_mtime: SystemTime,
    born: Instant,
    points: std::sync::Arc<Vec<nt_memory_pack::GeoPoint>>,
}

type GeoPackSlot = Mutex<Option<GeoPackCache>>;
fn geo_pack_cache() -> &'static GeoPackSlot {
    static CACHE: OnceLock<GeoPackSlot> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

const GEO_PACK_TTL: Duration = Duration::from_secs(60);

/// B2 v0: 从 NT-Pack 高密度文件直接读地理点 (绕开 SQLite), 前端无感切换。
///
/// 读取全量归档 `~/.neotrix/geo/geo_index.ntpack`; `source` 指定且存在冷层文件
/// `geo_<source>.ntpack` 时优先冷层 (B1 透明层)。NT-Pack 无 confidence, 补 0.0。
/// `limit` 硬上限 20k, 语义与 [`kb_geo_points`] 对齐。进程级缓存: 同路径+mtime+len
/// 在 TTL 内命中则只做源过滤 + 取 limit, 避免重复全量解码。
#[command]
pub fn kb_geo_points_pack(limit: Option<usize>, source: Option<String>) -> Result<Vec<GeoPoint>, NeoTrixError> {
    let (default_path, cold_dir) = geo_pack_paths();
    // 冷层优先: source 指定且地理目录有对应归档文件
    let path = source.as_ref().map(|s| cold_dir.join(format!("geo_{}.ntpack", s)))
        .filter(|p| p.exists())
        .unwrap_or(default_path);

    let limit = (limit.unwrap_or(5000) as usize).clamp(1, 20_000);
    let filtered = |points: Vec<nt_memory_pack::GeoPoint>, limit: usize| {
        points
            .into_iter()
            .filter(|p| match source.as_deref() {
                Some("shanhai") => p.source == "shanhai-peaks" || p.source == "shanhai-mappings",
                Some(s) => p.source == s,
                None => true,
            })
            .map(|p| GeoPoint {
                node_id: p.node_id,
                lat: p.lat,
                lng: p.lng,
                country: p.country,
                region: p.region,
                city: p.city,
                tags: p.tags,
                source: p.source,
                confidence: 0.0,
            })
            .take(limit)
            .collect()
    };

    // 缓存命中: 路径+mtime+len 相同且未过期 → 只过滤 + clone (≤limit 个)
    if let Ok(meta) = std::fs::metadata(&path) {
        let len = meta.len();
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let mut slot = geo_pack_cache().lock().unwrap();
        if let Some(c) = slot.as_ref() {
            if c.key_file == path && c.key_len == len && c.key_mtime == mtime
                && c.born.elapsed() < GEO_PACK_TTL
            {
                let pts: Vec<nt_memory_pack::GeoPoint> = c.points.iter().cloned().collect();
                return Ok(filtered(pts, limit));
            }
        }
        // 锁内只放解码（慢路径），命中路径零全量 decode
        let bytes = std::fs::read(&path)
            .map_err(|e| NeoTrixError::Memory(format!("ntpack 读 {}: {}", path.display(), e)))?;
        let (dec, points) = PackDecoder::decode(&bytes)
            .map_err(|e| NeoTrixError::Memory(format!("ntpack decode: {}", e)))?;
        let _ = dec;
        *slot = Some(GeoPackCache {
            key_file: path.clone(),
            key_len: len,
            key_mtime: mtime,
            born: Instant::now(),
            points: std::sync::Arc::new(points),
        });
        let pts: Vec<nt_memory_pack::GeoPoint> = slot
            .as_ref()
            .unwrap()
            .points
            .iter()
            .cloned()
            .collect();
        return Ok(filtered(pts, limit));
    }

    // 文件不存在: 回退旧逻辑 (返回空)
    let bytes = std::fs::read(&path)
        .map_err(|e| NeoTrixError::Memory(format!("ntpack 读 {}: {}", path.display(), e)))?;
    let (dec, points) = PackDecoder::decode(&bytes)
        .map_err(|e| NeoTrixError::Memory(format!("ntpack decode: {}", e)))?;
    let _ = dec;
    Ok(filtered(points, limit))
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
///
/// 冷层感知 (B1): 合并 `~/.neotrix/geo/geo_*.ntpack` 冷层归档计数 —
/// 归档后热表行被删, 若不计冷层, 层计数骤减会扭曲前端加载策略
/// (GeoLayerSummary 契约不变, 前端零改动)。
#[command]
pub fn kb_geo_layers() -> Result<Vec<GeoLayerSummary>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let mut stmt = conn
        .prepare("SELECT source, COUNT(*) FROM geo_index GROUP BY source")
        .map_err(|e| NeoTrixError::Memory(format!("geo layers prep: {}", e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| NeoTrixError::Memory(format!("geo layers query: {}", e)))?;
    // HashMap: source → count (热表)
    let mut counts: std::collections::HashMap<String, i64> = rows
        .filter_map(|r| r.ok())
        .collect();

    // 合并冷层计数: 枚举 geo_*.ntpack, decode 取条数 (冷层为少量小文件, 成本可忽略)
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".neotrix").join("geo");
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname == "geo_index.ntpack" { continue } // 全量文件非冷层
            let Some(rest) = fname.strip_prefix("geo_").map(|s| s.to_owned()) else { continue };
            if !rest.ends_with(".ntpack") { continue }
            let source = rest.trim_end_matches(".ntpack").to_string();
            if let Ok(bytes) = std::fs::read(entry.path()) {
                if let Ok((_, pts)) = neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::PackDecoder::decode(&bytes) {
                    *counts.entry(source).or_insert(0) += pts.len() as i64;
                }
            }
        }
    }

    let mut out: Vec<GeoLayerSummary> = counts
        .into_iter()
        .map(|(source, count)| GeoLayerSummary { source, count })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count));
    Ok(out)
}

/// 海拔点记录 — geo_elevation 表 + geo_index 来源分类，供前端海拔渐变着色。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoElevationPoint {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub elevation_m: f64,
    pub source: String,
}

/// 导出海拔记录 (geo_elevation) — 前端按高度渐变着色。
/// 排除 geonames-cities 低价值点，返回海拔降序。
#[command]
pub fn kb_geo_elevations(limit: Option<usize>) -> Result<Vec<GeoElevationPoint>, NeoTrixError> {
    let path = kb_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let limit = (limit.unwrap_or(2000) as i64).clamp(1, 20_000);
    let mut stmt = conn
        .prepare(
            "SELECT e.node_id, e.lat, e.lng, e.elevation_m, g.source
             FROM geo_elevation e
             LEFT JOIN geo_index g ON g.node_id = e.node_id
             ORDER BY e.elevation_m DESC
             LIMIT ?1",
        )
        .map_err(|e| NeoTrixError::Memory(format!("geo elevations prep: {}", e)))?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(GeoElevationPoint {
                node_id: row.get(0)?,
                lat: row.get(1)?,
                lng: row.get(2)?,
                elevation_m: row.get(3)?,
                source: row.get(4).unwrap_or_default(),
            })
        })
        .map_err(|e| NeoTrixError::Memory(format!("geo elevations query: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
