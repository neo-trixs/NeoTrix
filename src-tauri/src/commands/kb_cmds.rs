use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use tauri::command;
use rusqlite::params;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_memory_kb::nt_memory_store;
use neotrix::neotrix::nt_memory_kb::nt_memory_store::{get_all_nodes, get_all_edges};
use neotrix::neotrix::nt_memory_kb::nt_memory_types::{KnowledgeEdge, KnowledgeNode, NodeType, RelationType};
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::{self, PackDecoder};
use neotrix::neotrix::nt_memory_kb::nt_memory_store as kbs;

fn kb_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".neotrix").join("knowledge.db")
}

fn open_kb_conn() -> Result<rusqlite::Connection, NeoTrixError> {
    rusqlite::Connection::open(kb_path())
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))
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

/// 地理索引记录 (geo_index 表) — 供前端 3D 地图渲染.
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

/// 导出地理索引点 (geo_index) — 前端地球知识世界仿真数据源.
/// `source` 可选过滤：如 "shanhai" 只返回 shanhai-peaks + shanhai-mappings，
/// 供前端把幻境数据分层叠加在真实地图上（真实层取城市点，幻境层取全部 shanhai）.
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

/// NT-Pack 数据源路径: 全量归档 + 冷层归档.
/// 偏好 `~/.neotrix/geo/geo_index.ntpack` (全量) ; source 有冷层文件时优先冷层.
fn geo_pack_paths() -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".neotrix")
        .join("geo");
    (dir.join("geo_index.ntpack"), dir)
}

/// 进程级 NT-Pack 解码缓存 (单槽): key = 解析后路径 + mtime + len + TTL.
/// 避免 GlobeView 8 路并发各触发一次 7MB 全量 read+decode (约 50-60ms/次).
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

/// B2 v0: 从 NT-Pack 高密度文件直接读地理点 (绕开 SQLite), 前端无感切换.
///
/// 读取全量归档 `~/.neotrix/geo/geo_index.ntpack`; `source` 指定且存在冷层文件
/// `geo_<source>.ntpack` 时优先冷层 (B1 透明层).NT-Pack 无 confidence, 补 0.0.
/// `limit` 硬上限 20k, 语义与 [`kb_geo_points`] 对齐.进程级缓存: 同路径+mtime+len
/// 在 TTL 内命中则只做源过滤 + 取 limit, 避免重复全量解码.
#[command]
pub fn kb_geo_points_pack(limit: Option<usize>, source: Option<String>) -> Result<Vec<GeoPoint>, NeoTrixError> {
    let (default_path, cold_dir) = geo_pack_paths();
    // 冷层优先: source 指定且地理目录有对应归档文件
    let path = source.as_ref().map(|s| cold_dir.join(format!("geo_{}.ntpack", s)))
        .filter(|p| p.exists())
        .unwrap_or(default_path);

    let limit = limit.unwrap_or(5000).clamp(1, 20_000);
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
        let mut slot = geo_pack_cache().lock().map_err(|e| NeoTrixError::Memory(format!("geo cache lock: {}", e)))?;
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
            .map(|c| c.points.iter().cloned().collect())
            .unwrap_or_default();
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

/// 地理索引统计.
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

/// 地图分层摘要 — 各数据源计数.前端据此决定加载策略
/// （幻境层全量拉取，真实层按预算采样），实现前后端分离.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLayerSummary {
    pub source: String,
    pub count: i64,
}

/// 返回 geo_index 各 source 的计数（真实层/幻境层分层摘要）.
///
/// 冷层感知 (B1): 合并 `~/.neotrix/geo/geo_*.ntpack` 冷层归档计数 —
/// 归档后热表行被删, 若不计冷层, 层计数骤减会扭曲前端加载策略
/// (GeoLayerSummary 契约不变, 前端零改动).
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

/// 海拔点记录 — geo_elevation 表 + geo_index 来源分类，供前端海拔渐变着色.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoElevationPoint {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub elevation_m: f64,
    pub source: String,
}

/// 导出海拔记录 (geo_elevation) — 前端按高度渐变着色.
/// 排除 geonames-cities 低价值点，返回海拔降序.
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

/// B1 Trajectory storage: 写入/查询轨迹记录 (GeoJSON LineString 坐标序列).
#[command]
pub fn kb_trajectory_add(
    id: String,
    name: String,
    kind: Option<String>,
    points: Vec<f64>, // 交替 lat/lng 数组
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    distance_km: f64,
) -> Result<(), NeoTrixError> {
    use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::insert_trajectory;
    let conn = kb_path();
    let conn = rusqlite::Connection::open(&conn)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    insert_trajectory(&conn, &id, &name, kind.as_deref(), &points, (west, south, east, north), distance_km)
        .map_err(|e| NeoTrixError::Memory(format!("insert_trajectory: {}", e)))?;
    Ok(())
}

/// B1 Trajectory storage: 查询所有轨迹记录.
#[command]
pub fn kb_trajectory_query() -> Result<Vec<serde_json::Value>, NeoTrixError> {
    use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::query_trajectories;
    let conn = kb_path();
    let conn = rusqlite::Connection::open(&conn)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;
    let rows = query_trajectories(&conn).map_err(|e| NeoTrixError::Memory(format!("query_trajectories: {}", e)))?;
    let vals: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "kind": r.kind,
                "bbox": [r.bbox_west, r.bbox_south, r.bbox_east, r.bbox_north],
                "distance_km": r.distance_km,
                "created_at": r.created_at,
            })
        })
        .collect();
    Ok(vals)
}

/// C2 离线地图包: 导出 bbox 区域为 NT-Pack (v2 chunked 格式).
#[command]
pub fn kb_geo_offline_pack(
    bbox: [f64; 4], // [west, south, east, north]
    name: String,
) -> Result<serde_json::Value, NeoTrixError> {
    use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::query_bbox_with_cold;
    use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack_chunked::{encode_chunked, CHUNK_SIZE};
    use std::env;

    let conn = kb_path();
    let conn = rusqlite::Connection::open(&conn)
        .map_err(|e| NeoTrixError::Memory(format!("Open DB: {}", e)))?;

    let [west, south, east, north] = bbox;
    let (records, cold_hits) = query_bbox_with_cold(&conn, south, west, north, east, 20000, &cold_dir())?;

    // 转换为 pack crate 的 GeoPoint
    let points: Vec<neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::GeoPoint> = records
        .into_iter()
        .map(|r| neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::GeoPoint {
            node_id: r.node_id,
            lat: r.lat,
            lng: r.lng,
            country: r.country,
            region: r.region,
            city: r.city,
            tags: r.tags,
            source: r.source,
        })
        .collect();

    let path = format!("{}/.neotrix/geo/{}.ntpack", env::var("HOME").unwrap_or(".".into()), name);
    let bytes = encode_chunked(&points, CHUNK_SIZE);
    std::fs::write(&path, &bytes).map_err(|e| NeoTrixError::Memory(format!("Write pack: {}", e)))?;

    Ok(serde_json::json!({
        "path": path,
        "count": points.len(),
        "bytes": bytes.len(),
        "cold_hits": cold_hits,
    }))
}

fn cold_dir() -> String {
    let home = std::env::var("HOME").unwrap_or(".".to_string());
    format!("{}/.neotrix/geo", home)
}

/// 文档入库: 幂等 (同 title 覆盖重建)。切片 → chunk 节点 + part_of 边。
#[command]
pub fn kb_doc_ingest(title: String, text: String, library: Option<String>) -> Result<KbDocIngestResult, NeoTrixError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(NeoTrixError::Memory("title 不能为空".into()));
    }
    let library = library.unwrap_or_else(|| "default".into());
    let conn = open_kb_conn()?;
    let ts = now_ts();

    // 幂等: 同标题旧 doc 先删 (含 chunks)
    let existing = find_doc_by_title(&conn, &title)?;
    let doc_id = match existing {
        Some(old) => {
            delete_chunks(&conn, &old)?;
            kbs::delete_node(&conn, &old)
                .map_err(|e| NeoTrixError::Memory(format!("old doc del: {}", e)))?;
            old
        }
        None => format!("kbdoc-{}", uuid::Uuid::new_v4().simple()),
    };

    let chunks = split_chunks(&text);
    let status = if chunks.is_empty() { "empty" } else { "ready" };
    let doc = make_doc_node(&doc_id, &title, &text.trim(), &library, ts);

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| NeoTrixError::Memory(format!("ingest tx: {}", e)))?;
    kbs::insert_node_rows(&tx, &doc)
        .map_err(|e| NeoTrixError::Memory(format!("doc insert: {}", e)))?;
    write_chunks(&tx, &doc_id, &chunks, ts)?;
    tx.commit().map_err(|e| NeoTrixError::Memory(format!("ingest commit: {}", e)))?;

    Ok(KbDocIngestResult { doc_id, title, library, chunk_count: chunks.len(), status: status.into() })
}

#[command]
pub fn kb_doc_list() -> Result<Vec<KbDocSummary>, NeoTrixError> {
    let conn = open_kb_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.created_at,
                    COALESCE(LENGTH(n.content),0),
                    COALESCE(json_extract(n.metadata,'$.library'),'default'),
                    (SELECT COUNT(*) FROM edges e WHERE e.relation_type='part_of' AND e.target_id=n.id)
             FROM nodes n
             WHERE n.metadata LIKE ?1
             ORDER BY n.created_at DESC",
        )
        .map_err(|e| NeoTrixError::Memory(format!("doc list prep: {}", e)))?;
    let pat = format!("%{}%", KB_DOC_META);
    let rows = stmt
        .query_map(params![pat], |row| {
            let chunk_count: i64 = row.get(5)?;
            Ok(KbDocSummary {
                doc_id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                total_chars: row.get(3)?,
                library: row.get(4)?,
                chunk_count,
                status: if chunk_count > 0 { "ready" } else { "empty" }.into(),
            })
        })
        .map_err(|e| NeoTrixError::Memory(format!("doc list: {}", e)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// 删除文档及其全部切片, 返回删除的切片数。
#[command]
pub fn kb_doc_delete(doc_id: String) -> Result<usize, NeoTrixError> {
    let conn = open_kb_conn()?;
    let deleted = delete_chunks(&conn, &doc_id)?;
    kbs::delete_node(&conn, &doc_id)
        .map_err(|e| NeoTrixError::Memory(format!("doc del: {}", e)))?;
    Ok(deleted)
}

/// 重索引: 从 doc.content 全文重建切片。返回新切片数。
#[command]
pub fn kb_doc_reindex(doc_id: String) -> Result<usize, NeoTrixError> {
    let conn = open_kb_conn()?;
    let text: Option<String> = {
        let mut stmt = conn
            .prepare("SELECT content FROM nodes WHERE id=?1")
            .map_err(|e| NeoTrixError::Memory(format!("reindex prep: {}", e)))?;
        stmt.query_row(params![doc_id], |r| r.get(0))
            .map_err(|_| NeoTrixError::Memory(format!("doc 不存在: {}", doc_id)))?
    };
    let text = text.unwrap_or_default();
    let chunks = split_chunks(&text);
    let ts = now_ts();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| NeoTrixError::Memory(format!("reindex tx: {}", e)))?;
    delete_chunks(&tx, &doc_id)?;
    let n = write_chunks(&tx, &doc_id, &chunks, ts)?;
    tx.commit().map_err(|e| NeoTrixError::Memory(format!("reindex commit: {}", e)))?;
    Ok(n)
}

/* ══════════════════════════════════════════════════
   KB 文档级 CRUD (B2, 吸收 Cherry Studio KB 模式)
   复用 nodes/edges/nodes_fts 存储 (R-P42 零平行表):
   - Doc   article + metadata{"kb_doc":true,"library":L}, content=全文
   - Chunk article + metadata{"kb_chunk":true,"doc_id":D,"idx":I}
   - 边    chunk --[part_of]--> doc
   状态派生: chunk_count>0 → ready / ==0 → empty
   ══════════════════════════════════════════════════ */

/// 文档摘要 (列表面契约)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbDocSummary {
    pub doc_id: String,
    pub title: String,
    pub library: String,
    pub chunk_count: i64,
    pub total_chars: i64,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbDocIngestResult {
    pub doc_id: String,
    pub title: String,
    pub library: String,
    pub chunk_count: usize,
    pub status: String,
}

/// 段落感知确定性切片: 目标 ~600 字符, 相邻切片 100 重叠。
fn split_chunks(text: &str) -> Vec<String> {
    const TARGET: usize = 600;
    const OVERLAP: usize = 100;
    let text = text.trim();
    if text.is_empty() {
        return vec![];
    }
    let paras: Vec<&str> = text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for para in paras {
        if para.chars().count() > TARGET * 2 {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            let chars: Vec<char> = para.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let end = (i + TARGET).min(chars.len());
                out.push(chars[i..end].iter().collect());
                if end >= chars.len() {
                    break;
                }
                i = end.saturating_sub(OVERLAP);
            }
            continue;
        }
        if !cur.is_empty() && cur.chars().count() + para.chars().count() > TARGET {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push_str("\n\n");
        }
        cur.push_str(para);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/* ── 内部写路径 (可测: 接受 &Connection) ── */

const KB_DOC_META: &str = "\"kb_doc\":true";
const KB_CHUNK_META_PREFIX: &str = "{\"kb_chunk\":true";

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn make_doc_node(id: &str, title: &str, text: &str, library: &str, ts: i64) -> KnowledgeNode {
    KnowledgeNode {
        id: id.into(),
        node_type: NodeType::Article,
        title: title.into(),
        summary: Some(text.chars().take(120).collect()),
        content: Some(text.into()),
        url: None,
        domain: Some("app-kb".into()),
        language: "zh".into(),
        confidence: 1.0,
        importance: 0.6,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: Some(serde_json::json!({"kb_doc": true, "library": library})),
        temporal: None,
        supersedes: None,
        source_episode: None,
    }
}

fn make_chunk_node(id: &str, doc_id: &str, idx: usize, text: &str, ts: i64) -> KnowledgeNode {
    let summary: String = text.chars().take(80).collect();
    KnowledgeNode {
        id: id.into(),
        node_type: NodeType::Article,
        title: format!("{} · 段{}", doc_id.trim_start_matches("kbdoc-"), idx + 1),
        summary: Some(summary),
        content: Some(text.into()),
        url: None,
        domain: Some("app-kb".into()),
        language: "zh".into(),
        confidence: 1.0,
        importance: 0.5,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: Some(serde_json::json!({"kb_chunk": true, "doc_id": doc_id, "idx": idx})),
        temporal: None,
        supersedes: None,
        source_episode: None,
    }
}

/// 写入 chunk 节点 + part_of 边 (事务)。返回切片数。
fn write_chunks(conn: &rusqlite::Connection, doc_id: &str, chunks: &[String], ts: i64) -> Result<usize, NeoTrixError> {
    for (idx, text) in chunks.iter().enumerate() {
        let cid = format!("{}-c{:03}", doc_id, idx);
        let node = make_chunk_node(&cid, doc_id, idx, text, ts);
        kbs::insert_node_rows(conn, &node)
            .map_err(|e| NeoTrixError::Memory(format!("chunk insert {}: {}", cid, e)))?;
        let edge = KnowledgeEdge {
            id: format!("{}-e{:03}", doc_id, idx),
            source_id: cid.clone(),
            target_id: doc_id.into(),
            relation_type: RelationType::PartOf,
            weight: 1.0,
            description: None,
            created_at: ts,
            metadata: None,
        };
        kbs::insert_edge(conn, &edge)
            .map_err(|e| NeoTrixError::Memory(format!("chunk edge {}: {}", cid, e)))?;
    }
    Ok(chunks.len())
}

/// 删除文档的全部 chunk (节点+边), 返回删除数。
fn delete_chunks(conn: &rusqlite::Connection, doc_id: &str) -> Result<usize, NeoTrixError> {
    let ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT source_id FROM edges WHERE relation_type='part_of' AND target_id=?1")
            .map_err(|e| NeoTrixError::Memory(format!("chunk q {}: {}", doc_id, e)))?;
        let rows = stmt
            .query_map(params![doc_id], |r| r.get::<_, String>(0))
            .map_err(|e| NeoTrixError::Memory(format!("chunk q map: {}", e)))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let n = ids.len();
    for cid in ids {
        // 两步删 (FTS 行先于节点行) — 不用 kbs::delete_node: 其内部
        // unchecked_transaction 在外层 reindex 事务中会嵌套炸 (测试抓到)
        conn.execute(
            "DELETE FROM nodes_fts WHERE rowid = (SELECT rowid FROM nodes WHERE id=?1)",
            params![cid],
        )
        .map_err(|e| NeoTrixError::Memory(format!("chunk fts del {}: {}", cid, e)))?;
        conn.execute("DELETE FROM nodes WHERE id=?1", params![cid])
            .map_err(|e| NeoTrixError::Memory(format!("chunk del {}: {}", cid, e)))?;
    }
    // 清理残留 part_of 边 (delete_node 可能不级联边)
    conn.execute(
        "DELETE FROM edges WHERE relation_type='part_of' AND target_id=?1",
        params![doc_id],
    )
    .map_err(|e| NeoTrixError::Memory(format!("chunk edges del: {}", e)))?;
    Ok(n)
}

/// 同 title 幂等查找已有 doc id。
fn find_doc_by_title(conn: &rusqlite::Connection, title: &str) -> Result<Option<String>, NeoTrixError> {
    let mut stmt = conn
        .prepare("SELECT id FROM nodes WHERE node_type='article' AND title=?1 AND metadata LIKE ?2 LIMIT 1")
        .map_err(|e| NeoTrixError::Memory(format!("doc find prep: {}", e)))?;
    let pat = format!("%{}%", KB_DOC_META);
    let mut rows = stmt
        .query_map(params![title, pat], |r| r.get::<_, String>(0))
        .map_err(|e| NeoTrixError::Memory(format!("doc find: {}", e)))?;
    match rows.next() {
        Some(Ok(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod doc_crud_tests {
    use super::*;

    /// 最小 KB schema (镜像 nt_memory_store 的 SQL 面) — 测试自足, 不依赖真实 knowledge.db
    fn test_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("mem db");
        conn.execute_batch(
            "CREATE TABLE nodes (
                id TEXT PRIMARY KEY, node_type TEXT NOT NULL, title TEXT NOT NULL,
                summary TEXT, content TEXT, url TEXT, domain TEXT, language TEXT DEFAULT 'zh',
                confidence REAL DEFAULT 1.0, importance REAL DEFAULT 0.5,
                created_at INTEGER, updated_at INTEGER, access_count INTEGER DEFAULT 0,
                metadata TEXT, data_tier TEXT DEFAULT 'core', temporal TEXT,
                supersedes TEXT, source_episode TEXT, tier TEXT DEFAULT 'warm'
             );
             CREATE VIRTUAL TABLE nodes_fts USING fts5(title, summary, content, domain);
             CREATE TABLE edges (
                id TEXT PRIMARY KEY, source_id TEXT NOT NULL, target_id TEXT NOT NULL,
                relation_type TEXT NOT NULL, weight REAL DEFAULT 1.0,
                description TEXT, created_at INTEGER, metadata TEXT
             );",
        )
        .expect("schema");
        conn
    }

    #[test]
    fn test_split_chunks_deterministic_and_bounded() {
        let long_para = "字".repeat(1500);
        let chunks = split_chunks(&format!("第一段短文\n\n{}\n\n结尾段", long_para));
        assert!(chunks.len() >= 3, "超长段应硬切: {}", chunks.len());
        // 确定性
        let again = split_chunks(&format!("第一段短文\n\n{}\n\n结尾段", long_para));
        assert_eq!(chunks, again);
        assert!(chunks.iter().all(|c| c.chars().count() <= 700), "切片不超过 ~600+overlap 上限");
    }

    #[test]
    fn test_split_chunks_empty() {
        assert!(split_chunks("").is_empty());
        assert!(split_chunks("   \n\n  ").is_empty());
    }

    fn ingest_into(conn: &rusqlite::Connection, title: &str, text: &str) -> KbDocIngestResult {
        let ts = now_ts();
        let doc_id = format!("kbdoc-test-{}", title);
        let chunks = split_chunks(text);
        let doc = make_doc_node(&doc_id, title, text.trim(), "default", ts);
        kbs::insert_node_rows(conn, &doc).expect("doc insert");
        write_chunks(conn, &doc_id, &chunks, ts).expect("chunks");
        KbDocIngestResult {
            doc_id, title: title.into(), library: "default".into(),
            chunk_count: chunks.len(), status: if chunks.is_empty() { "empty" } else { "ready" }.into(),
        }
    }

    #[test]
    fn test_doc_roundtrip_ingest_list_delete() {
        let conn = test_conn();
        let r = ingest_into(&conn, "设计文档", "## 一\n\n内容A\n\n内容B");
        assert_eq!(r.chunk_count, 1);
        assert_eq!(r.status, "ready");

        // list 可见且计数正确
        let n_chunks: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE relation_type='part_of' AND target_id=?1",
                params![r.doc_id], |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n_chunks, 1);

        // delete 级联清 chunk
        let deleted = delete_chunks(&conn, &r.doc_id).unwrap();
        assert_eq!(deleted, 1);
        kbs::delete_node(&conn, &r.doc_id).unwrap();
        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes WHERE id LIKE 'kbdoc-test-%'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(left, 0, "doc+chunk 节点应全部删除");
    }

    #[test]
    fn test_reindex_replaces_old_chunks() {
        let conn = test_conn();
        let r = ingest_into(&conn, "重建文档", "旧内容一\n\n旧内容二");
        assert!(r.chunk_count >= 1);

        // 模拟全文更新后重索引为单段
        let new_text = "全新单一内容";
        conn.execute(
            "UPDATE nodes SET content=?1 WHERE id=?2",
            params![new_text, r.doc_id],
        )
        .unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        delete_chunks(&tx, &r.doc_id).unwrap();
        let n = write_chunks(&tx, &r.doc_id, &split_chunks(new_text), now_ts()).unwrap();
        tx.commit().unwrap();
        assert_eq!(n, 1, "重索引后仅剩新切片 (单短段合并为 1)");

        let stale: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes WHERE id LIKE ?1", params![format!("{}-c%", r.doc_id)], |row| row.get(0))
            .unwrap();
        assert_eq!(stale, 1, "旧 chunk 不残留");
    }
}
