//! nt_memory_cortex_sync — 外置大脑双向同步边界 (对应 data-flow map G1–G5)。
//!
//! 让外置大脑 (`/Volumes/NeoTrixBrain`) 从"只读冷存档"升级为进化伙伴:
//! - 入向 (已存在): `register_cortex_brain` 注册 + `load_cortex_causal_graph` 消费因果图;
//! - 出向 (本模块新增): `export_delta` 把 live KB 的 SEAL 吸收产物 (`kv_store` `experience`
//!   namespace) 回写 volume 的 `working/nt_cortex_delta.jsonl`;
//! - 血缘 (G3): 每个 `cortex_brain` 节点 metadata 带 `lineage` 块, 记录 sha256/上次同步/循环;
//! - 版本门禁 (G2): 回写前比对因果图 sha256, 防止写入被替换/回滚的外置大脑;
//! - 挂载后才同步 (G4): 未挂载则拒绝回写。
//!
//! 设计见 `docs/cortex-data-flow-map.md` §3。仅新增, 不改既有 schema (lineage 存 metadata JSON)。

use std::io::Write;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use super::nt_memory_store::{get_node, insert_or_get_node_rows, update_node_metadata};
use super::nt_memory_types::NodeType;
use super::nt_normalizer::validate_node_type;
use crate::core::nt_core_e8::abduction::causal_graph::CausalGraph;

const SCHEMA_VERSION: u32 = 1;
/// 超过该体积的文件不计算 sha256 (避免 68GB corpus 卡死); 仅小文件 (因果图) 取指纹。
const SHA_SIZE_CAP: u64 = 100 * 1024 * 1024;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// 外置大脑节点的血缘块 (存于 `nodes.metadata` 的 `lineage` 字段, 增量、无 schema 迁移)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub schema_version: u32,
    pub external_snapshot_ts: Option<i64>,
    pub external_sha256: Option<String>,
    pub last_synced_at: Option<i64>,
    pub last_seal_cycle: Option<String>,
    pub direction: String, // "in" = 已消费; "out" = 已回写
    pub kb_nodes_derived: Option<usize>,
}

impl Default for Lineage {
    fn default() -> Self {
        Lineage {
            schema_version: SCHEMA_VERSION,
            external_snapshot_ts: None,
            external_sha256: None,
            last_synced_at: None,
            last_seal_cycle: None,
            direction: "in".to_string(),
            kb_nodes_derived: None,
        }
    }
}

/// 计算文件 sha256 (小文件); 超过 `SHA_SIZE_CAP` 返回 None (避免大档案卡死)。
fn maybe_sha256(path: &Path) -> Option<String> {
    let len = std::fs::metadata(path).ok()?.len();
    if len > SHA_SIZE_CAP {
        return None;
    }
    let data = std::fs::read(path).ok()?;
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(&data);
    Some(hex::encode(h.finalize()))
}

/// 把一个 `cortex_brain` 节点的 metadata 注入 `lineage` 块 (G3)。
///
/// `external_path` 指向实际外置文件时, 取其 sha256 与 json `ts` 作为血缘锚点; 大文件 (>100MB)
/// 跳过 sha。幂等: 重复调用只在既有 lineage 上更新, 不丢历史。
pub fn enrich_cortex_metadata(
    payload: &mut serde_json::Value,
    external_path: Option<&Path>,
    direction: &str,
) {
    let mut lineage = payload
        .get("lineage")
        .and_then(|l| serde_json::from_value::<Lineage>(l.clone()).ok())
        .unwrap_or_default();
    lineage.schema_version = SCHEMA_VERSION;
    lineage.direction = direction.to_string();
    if let Some(p) = external_path {
        if let Some(s) = maybe_sha256(p) {
            lineage.external_sha256 = Some(s);
        }
        if let Ok(txt) = std::fs::read_to_string(p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                if let Some(ts) = v.get("ts").and_then(|t| t.as_i64()) {
                    lineage.external_snapshot_ts = Some(ts);
                }
            }
        }
    }
    lineage.last_synced_at = Some(now());
    payload["lineage"] = serde_json::to_value(&lineage).expect("lineage serializes");
}

/// `/cortex lineage` 报告: 每个 cortex_brain 节点的血缘快照 (G3 可观测性, Phase 1)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageReport {
    pub url: String,
    pub kind: String,
    pub external_sha256: Option<String>,
    pub last_synced_at: Option<i64>,
    pub last_seal_cycle: Option<String>,
    pub direction: String,
}

pub fn report_lineage(conn: &Connection) -> Result<Vec<LineageReport>, String> {
    let mut stmt = conn
        .prepare("SELECT url, metadata FROM nodes WHERE node_type='cortex_brain'")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (url, meta_str) = row.map_err(|e| e.to_string())?;
        let meta: serde_json::Value =
            serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null);
        let kind = meta
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("?")
            .to_string();
        let lineage = meta
            .get("lineage")
            .and_then(|l| serde_json::from_value::<Lineage>(l.clone()).ok());
        out.push(LineageReport {
            url,
            kind,
            external_sha256: lineage.as_ref().and_then(|l| l.external_sha256.clone()),
            last_synced_at: lineage.as_ref().and_then(|l| l.last_synced_at),
            last_seal_cycle: lineage.as_ref().and_then(|l| l.last_seal_cycle.clone()),
            direction: lineage.map(|l| l.direction).unwrap_or_else(|| "in".to_string()),
        });
    }
    Ok(out)
}

/// `export_delta` 结果 (G1 出向 + G2 版本门禁结果)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaReport {
    pub entries: usize,
    pub bytes: u64,
    pub since: i64,
    pub dry_run: bool,
    pub version_ok: bool,
}

/// 出向回写 (G1): 把 live KB `experience` namespace 中 `updated_at > 上次同步` 的增量,
/// 以 append-only JSONL 写入外置大脑的 `working/nt_cortex_delta.jsonl` (Dark Forest: 连接不膨胀)。
///
/// - G4: 外置大脑未挂载 → 拒绝。
/// - G2 版本门禁: 若上次同步记录过因果图 sha256, 必须与当前文件一致, 否则拒绝 (防污染)。
/// - 成功后更新 cortex_brain 节点 lineage (`last_synced_at`/`direction=out`/`last_seal_cycle`)。
/// - `dry_run`: 仅统计增量条数, 不写盘 (沿用 `--force` 约定之外的 `--dry-run`)。
pub fn export_delta(conn: &Connection, root: &Path, dry_run: bool) -> Result<DeltaReport, String> {
    if !root.exists() {
        return Err(format!(
            "外置大脑未挂载: {} — 拒绝回写 (G4: 挂载后才同步)",
            root.display()
        ));
    }

    // 读取 causal_graph 节点的既有 lineage, 做 G2 版本门禁。
    let mut last_synced_at = 0i64;
    let mut last_sha: Option<String> = None;
    if let Some(l) = read_cortex_lineage(conn, "cortex_source://causal_graph")? {
        last_synced_at = l.last_synced_at.unwrap_or(0);
        last_sha = l.external_sha256.clone();
    }
    if let Some(sha) = last_sha {
        let causal = root.join("working").join("causal_graph.json");
        if causal.exists() {
            let cur = maybe_sha256(&causal)
                .ok_or_else(|| "无法计算因果图 sha256 (文件过大或不可读)".to_string())?;
            if cur != sha {
                return Err(
                    "版本门禁失败: 外置因果图 sha256 与上次同步记录不符 (可能被替换/回滚)。\
                     拒绝回写以免污染外置大脑 (G2)。重新 /cortex register 后可继续。"
                        .to_string(),
                );
            }
        }
    }

    // 查询上次同步以来的 experience 增量。
    let since = last_synced_at;
    let mut stmt = conn
        .prepare(
            "SELECT key, value, updated_at FROM kv_store \
             WHERE namespace='experience' AND updated_at > ?1 ORDER BY updated_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([since], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Vec<u8>>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    if dry_run {
        let mut count = 0usize;
        for row in rows {
            let _ = row.map_err(|e| e.to_string())?;
            count += 1;
        }
        return Ok(DeltaReport {
            entries: count,
            bytes: 0,
            since,
            dry_run: true,
            version_ok: true,
        });
    }

    let working = root.join("working");
    std::fs::create_dir_all(&working).map_err(|e| e.to_string())?;
    let delta_path = working.join("nt_cortex_delta.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&delta_path)
        .map_err(|e| e.to_string())?;

    let mut count = 0usize;
    let mut bytes = 0u64;
    use base64::engine::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    for row in rows {
        let (key, value, ts) = row.map_err(|e| e.to_string())?;
        let line = serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "exported_at": now(),
            "experience_key": key,
            "updated_at": ts,
            "value_b64": engine.encode(&value),
        });
        let s = serde_json::to_string(&line).map_err(|e| e.to_string())?;
        writeln!(file, "{s}").map_err(|e| e.to_string())?;
        bytes += s.len() as u64 + 1;
        count += 1;
    }
    file.flush().map_err(|e| e.to_string())?;

    // 更新 causal_graph 节点 lineage: 标记已回写 + 记录最新循环。
    let latest_cycle = latest_cycle_from_experience(conn)?;
    update_cortex_lineage(conn, "cortex_source://causal_graph", |l| {
        l.last_synced_at = Some(now());
        l.direction = "out".to_string();
        l.last_seal_cycle = latest_cycle;
    })?;

    Ok(DeltaReport {
        entries: count,
        bytes,
        since,
        dry_run: false,
        version_ok: true,
    })
}

// ── 内部 helper ──

fn read_cortex_lineage(conn: &Connection, url: &str) -> Result<Option<Lineage>, String> {
    let meta: Option<String> = conn
        .query_row(
            "SELECT metadata FROM nodes WHERE node_type='cortex_brain' AND url=?1",
            [url],
            |r| r.get::<_, String>(0),
        )
        .ok();
    Ok(meta.and_then(|m| {
        serde_json::from_str::<serde_json::Value>(&m)
            .ok()
            .and_then(|v| v.get("lineage").and_then(|l| serde_json::from_value::<Lineage>(l.clone()).ok()))
    }))
}

fn update_cortex_lineage<F>(conn: &Connection, url: &str, f: F) -> Result<(), String>
where
    F: FnOnce(&mut Lineage),
{
    let meta_str: Option<String> = conn
        .query_row(
            "SELECT metadata FROM nodes WHERE node_type='cortex_brain' AND url=?1",
            [url],
            |r| r.get::<_, String>(0),
        )
        .ok();
    let mut meta: serde_json::Value = match meta_str {
        Some(s) => serde_json::from_str(&s).unwrap_or(serde_json::json!({})),
        None => return Ok(()), // 节点不存在则无需更新
    };
    let mut lineage = meta
        .get("lineage")
        .and_then(|l| serde_json::from_value::<Lineage>(l.clone()).ok())
        .unwrap_or_default();
    f(&mut lineage);
    lineage.schema_version = SCHEMA_VERSION;
    meta["lineage"] = serde_json::to_value(&lineage).expect("lineage serializes");
    conn.execute(
        "UPDATE nodes SET metadata=?1, updated_at=?2 WHERE node_type='cortex_brain' AND url=?3",
        rusqlite::params![meta.to_string(), now(), url],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 从 experience namespace 推断最新循环标识 (key 形如 `cycle_NNN/...` 取 `cycle_NNN`)。
fn latest_cycle_from_experience(conn: &Connection) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT key FROM kv_store WHERE namespace='experience' AND key LIKE 'cycle_%' \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    let r = stmt.query_row([], |row| row.get::<_, String>(0)).ok();
    Ok(r.map(|k| k.split('/').next().unwrap_or(&k).to_string()))
}

/// Phase 1 有界采样摄取报告: 从外置大脑 corpus (SQLite 超集快照) 取样的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestReport {
    pub sampled: usize,
    pub activated: usize,
    pub already_live: usize,
    pub bytes: u64,
    pub node_types: Vec<String>,
}

/// Phase 1 有界采样摄取 (G1 入向 + G3 血缘 + G4 挂载检查)。
///
/// 外置大脑 corpus (`knowledge-archive-corpus-20260825.db`) 是 live KB `nodes` 的**超集快照**。
/// 本函数取样 `top_k` 个冷节点, 把 live KB 中尚缺的节点"激活"进 live KB (Dark Forest: 连接不膨胀),
/// 并写入 `lineage` (G3) + 原始 content/metadata。只读采样, 不改权重 — 把冷库变"可寻址/可检索"的活知识。
///
/// - G4: corpus 未挂载 → 拒绝。
/// - 复用既有 `insert_or_get_node_rows` (R-P42, 不平行造插入器), 仅以 UPDATE 补 content/lineage。
pub fn digest_sample(
    conn: &Connection,
    corpus_db: &Path,
    top_k: usize,
    domain_filter: Option<&str>,
) -> Result<DigestReport, String> {
    if !corpus_db.exists() {
        return Err(format!(
            "外置大脑 corpus 未挂载/不存在: {} (G4: 挂载后才摄取)",
            corpus_db.display()
        ));
    }
    let cdb = Connection::open(corpus_db).map_err(|e| e.to_string())?;
    let mut stmt = cdb
        .prepare(
            "SELECT id, node_type, title, summary, content, url, domain, metadata \
             FROM nodes ORDER BY rowid LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([top_k as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1).unwrap_or_default(),
                r.get::<_, String>(2).unwrap_or_default(),
                r.get::<_, Option<String>>(3).unwrap_or_default(),
                r.get::<_, Option<String>>(4).unwrap_or_default(),
                r.get::<_, String>(5).unwrap_or_default(),
                r.get::<_, Option<String>>(6).unwrap_or_default(),
                r.get::<_, Option<String>>(7).unwrap_or_default(),
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut rep = DigestReport {
        sampled: 0,
        activated: 0,
        already_live: 0,
        bytes: 0,
        node_types: Vec::new(),
    };

    for row in rows {
        let (cid, ntype, title, summary, content, url, domain, meta_str) = match row {
            Ok(t) => t,
            Err(_) => continue,
        };
        rep.sampled += 1;
        if let Some(f) = domain_filter {
            let hay = format!("{url} {domain:?} {ntype}").to_ascii_lowercase();
            if !hay.contains(&f.to_ascii_lowercase()) {
                continue;
            }
        }
        // 已 live 则跳过 (避免重复激活)
        let live: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes WHERE url=?", rusqlite::params![url.clone()], |r| r.get(0))
            .unwrap_or(0);
        if live > 0 {
            rep.already_live += 1;
            continue;
        }
        let ntype_enum = NodeType::from_str(validate_node_type(&ntype));
        let _id = insert_or_get_node_rows(
            conn,
            &title,
            ntype_enum,
            summary.as_deref(),
            Some(&url),
            domain.as_deref(),
        )
        .map_err(|e| e.to_string())?;
        // 写入 lineage + 原始 content/metadata (G3)
        let mut meta = match meta_str {
            Some(s) => serde_json::from_str::<serde_json::Value>(&s).unwrap_or(serde_json::Value::Null),
            None => serde_json::Value::Null,
        };
        if !meta.is_object() {
            meta = serde_json::json!({});
        }
        enrich_cortex_metadata(&mut meta, Some(corpus_db), "in");
        let content_str = content.clone().unwrap_or_default();
        conn.execute(
            "UPDATE nodes SET content=?, metadata=?, updated_at=? WHERE url=?",
            rusqlite::params![content_str, meta.to_string(), now(), url.clone()],
        )
        .map_err(|e| e.to_string())?;
        rep.activated += 1;
        rep.bytes += content_str.len() as u64;
        if !rep.node_types.contains(&ntype) {
            rep.node_types.push(ntype);
        }
        let _ = cid;
    }
    Ok(rep)
}

/// Phase 3 (G1+G3+G4): 把外置因果图的高信号节点蒸馏进 live KB。
///
/// 筛选高信号: confidence ≥ 0.8 或 关联边(入+出)数 ≥ 2。每个节点经既有
/// `insert_or_get_node_rows` 落盘 (url = `cortex_source://causal_graph#<id>`, 不平行造插入器, R-P42),
/// 并写 `lineage` 块 (G3)。同时更新伞节点 `cortex_source://causal_graph` 的 lineage sha,
/// 供 `export_delta` 的 G2 版本门禁做指纹比对。挂载门禁 (G4) 由 CLI 层保证。
pub fn ingest_causal_graph(
    conn: &Connection,
    graph: &CausalGraph,
    source_path: Option<&Path>,
) -> Result<usize, String> {
    let mut derived = 0usize;
    for node in &graph.nodes {
        let degree = graph
            .edges
            .iter()
            .filter(|e| e.from == node.id || e.to == node.id)
            .count();
        if node.confidence < 0.8 && degree < 2 {
            continue; // 低信号跳过, 防止噪音灌入 live KB
        }
        let url = format!("cortex_source://causal_graph#{}", node.id);
        let id = insert_or_get_node_rows(
            conn,
            &node.description,
            NodeType::Concept,
            Some(&node.description),
            Some(&url),
            Some("NT-CORE"),
        )
        .map_err(|e| e.to_string())?;
        // 写 lineage (G3): 用既有 metadata 增量更新, 不平行造写入器
        if let Ok(Some(existing)) = get_node(conn, &id) {
            let mut payload: serde_json::Value = existing
                .metadata
                .clone()
                .unwrap_or(serde_json::Value::Null);
            if !payload.is_object() {
                payload = serde_json::json!({});
            }
            enrich_cortex_metadata(&mut payload, source_path, "in");
            update_node_metadata(conn, &id, &payload).map_err(|e| e.to_string())?;
        }
        derived += 1;
    }
    // 伞节点 lineage (G2 版本门禁锚点): 复用既有插入器取 id 后写 sha
    let umbrella_id = insert_or_get_node_rows(
        conn,
        "causal_graph",
        NodeType::Concept,
        Some("外置大脑因果图蒸馏锚点"),
        Some("cortex_source://causal_graph"),
        Some("NT-CORE"),
    )
    .map_err(|e| e.to_string())?;
    if let Ok(Some(existing)) = get_node(conn, &umbrella_id) {
        let mut payload: serde_json::Value = existing
            .metadata
            .clone()
            .unwrap_or(serde_json::json!({}));
        enrich_cortex_metadata(&mut payload, source_path, "in");
        update_node_metadata(conn, &umbrella_id, &payload).map_err(|e| e.to_string())?;
    }
    Ok(derived)
}

/// Phase 6 (G5 双向 prune): 反向修剪外置 corpus 中已被 live KB 遗弃且陈旧的条目。
///
/// 安全约束 (避免破坏 68GB 冷存档): 仅当外置条目 **带 `lineage`** (说明曾被激活进 live KB)
/// 且当前 live KB 已无对应节点 (双向孤儿) 且 `last_synced_at` 超过 `stale_days` 时才候选。
/// 纯冷存档条目 (无 lineage) 永不修剪。默认 `dry_run` 只计数, 需显式 `dry_run=false` 才删行。
pub fn prune_external(
    conn: &Connection,
    corpus_db: &Path,
    stale_days: i64,
    dry_run: bool,
) -> Result<usize, String> {
    if !corpus_db.exists() {
        return Err(format!(
            "外置 corpus 未挂载/不存在: {} (G4: 挂载后才修剪)",
            corpus_db.display()
        ));
    }
    let cdb = Connection::open(corpus_db).map_err(|e| e.to_string())?;
    let mut stmt = cdb
        .prepare("SELECT id, url, metadata FROM nodes")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1).unwrap_or_default(),
                r.get::<_, Option<String>>(2).unwrap_or_default(),
            ))
        })
        .map_err(|e| e.to_string())?;
    let threshold = now() - stale_days * 86400;
    let mut to_prune: Vec<String> = Vec::new();
    for row in rows {
        let (id, url, meta_str) = match row {
            Ok(t) => t,
            Err(_) => continue,
        };
        // 仅带 lineage 的条目才可能成为双向修剪候选 (纯冷存档不碰)
        let meta: serde_json::Value = meta_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::Value::Null);
        let last = meta
            .get("lineage")
            .and_then(|l| l.get("last_synced_at"))
            .and_then(|v| v.as_i64());
        let Some(last) = last else {
            continue;
        };
        // live KB 已无对应节点 → 双向孤儿
        let live: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE url=?",
                rusqlite::params![url.clone()],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if live > 0 {
            continue;
        }
        if last < threshold {
            to_prune.push(id);
        }
    }
    if dry_run {
        return Ok(to_prune.len());
    }
    let tx = cdb.unchecked_transaction().map_err(|e| e.to_string())?;
    for id in &to_prune {
        tx.execute("DELETE FROM nodes WHERE id=?", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(to_prune.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn fixture_db() -> Connection {
        let conn = Connection::open_in_memory().expect("mem db");
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, content TEXT, \
             url TEXT, created_at INTEGER, updated_at INTEGER, metadata TEXT);
             CREATE TABLE kv_store (namespace TEXT, key TEXT, value BLOB, updated_at INTEGER);",
        )
        .expect("schema");
        // 一个模拟的 cortex_brain 因果图节点
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, content, url, created_at, updated_at, metadata) \
             VALUES ('c1', 'cortex_brain', 'g', 'g', 'cortex_source://causal_graph', 0, 0, '{}')",
            [],
        )
        .expect("insert");
        conn
    }

    #[test]
    fn enrich_adds_lineage_with_sha() -> Result<(), String> {
        let dir = tempfile::tempdir().expect("tmp");
        let cg = dir.path().join("causal_graph.json");
        std::fs::write(&cg, serde_json::json!({"ts": 1700000000, "nodes": []}).to_string())
            .expect("write");
        let mut payload = serde_json::json!({"kind": "cortex_causal_graph"});
        enrich_cortex_metadata(&mut payload, Some(&cg), "in");
        let lineage: Lineage = serde_json::from_value(payload["lineage"].clone()).expect("parse");
        assert_eq!(lineage.schema_version, SCHEMA_VERSION);
        assert_eq!(lineage.direction, "in");
        assert_eq!(lineage.external_snapshot_ts, Some(1700000000));
        assert!(lineage.external_sha256.is_some());
        Ok(())
    }

    #[test]
    fn enrich_skips_sha_on_huge_file() -> Result<(), String> {
        let dir = tempfile::tempdir().expect("tmp");
        let big = dir.path().join("corpus.db");
        // 写一个 > cap 的占位文件 (内容无关, 只测大小跳过)
        let f = std::fs::File::create(&big).expect("create");
        f.set_len(SHA_SIZE_CAP + 1).expect("truncate");
        let mut payload = serde_json::json!({"kind": "corpus_cold_archive"});
        enrich_cortex_metadata(&mut payload, Some(&big), "in");
        let lineage: Lineage = serde_json::from_value(payload["lineage"].clone()).expect("parse");
        assert!(lineage.external_sha256.is_none()); // 大文件不取指纹
        Ok(())
    }

    #[test]
    fn report_lineage_reads_node() -> Result<(), String> {
        let conn = fixture_db();
        let dir = tempfile::tempdir().expect("tmp");
        let cg = dir.path().join("causal_graph.json");
        std::fs::write(&cg, serde_json::json!({"ts": 1700000000}).to_string()).expect("write");
        let mut payload = serde_json::json!({"kind": "cortex_causal_graph"});
        enrich_cortex_metadata(&mut payload, Some(&cg), "in");
        conn.execute(
            "UPDATE nodes SET metadata=?1 WHERE url='cortex_source://causal_graph'",
            [payload.to_string()],
        )
        .expect("update");
        let reps = report_lineage(&conn)?;
        assert_eq!(reps.len(), 1);
        assert_eq!(reps[0].kind, "cortex_causal_graph");
        assert!(reps[0].external_sha256.is_some());
        Ok(())
    }

    #[test]
    fn export_delta_dry_run_counts_only() -> Result<(), String> {
        let conn = fixture_db();
        // 放两条 experience, 一条旧一条新
        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES ('experience','cycle_001/a',X'01',100)",
            [],
        )
        .expect("ins");
        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES ('experience','cycle_002/b',X'02',200)",
            [],
        )
        .expect("ins");
        let dir = tempfile::tempdir().expect("tmp");
        // 先给因果图节点写 lineage (last_synced_at=150), 避免版本门禁拦 (无 sha 记录时放行)
        let mut payload = serde_json::json!({"kind": "cortex_causal_graph"});
        enrich_cortex_metadata(&mut payload, None, "in");
        // 设 last_synced_at=150 模拟上一次同步
        update_cortex_lineage(&conn, "cortex_source://causal_graph", |l| {
            l.last_synced_at = Some(150);
        })?;
        let r = export_delta(&conn, dir.path(), true)?;
        assert!(r.dry_run);
        assert_eq!(r.entries, 1); // updated_at>150 仅 cycle_002 (200) 算, cycle_001 (100) 不算
        assert_eq!(r.since, 150);
        // dry-run 不写盘
        assert!(!dir.path().join("working").join("nt_cortex_delta.jsonl").exists());
        Ok(())
    }

    #[test]
    fn export_delta_writes_jsonl_and_updates_lineage() -> Result<(), String> {
        let conn = fixture_db();
        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES ('experience','cycle_001/a',X'deadbeef',200)",
            [],
        )
        .expect("ins");
        let dir = tempfile::tempdir().expect("tmp");
        update_cortex_lineage(&conn, "cortex_source://causal_graph", |l| {
            l.last_synced_at = Some(150);
        })?;
        let r = export_delta(&conn, dir.path(), false)?;
        assert!(!r.dry_run);
        assert_eq!(r.entries, 1);
        let jl = dir.path().join("working").join("nt_cortex_delta.jsonl");
        assert!(jl.exists());
        let txt = std::fs::read_to_string(&jl).expect("read");
        assert!(txt.contains("cycle_001/a"));
        assert!(txt.contains("value_b64"));
        // lineage 应更新为 out + cycle
        let reps = report_lineage(&conn)?;
        assert_eq!(reps[0].direction, "out");
        assert_eq!(reps[0].last_seal_cycle.as_deref(), Some("cycle_001"));
        Ok(())
    }

    #[test]
    fn export_delta_refuses_when_version_mismatch() -> Result<(), String> {
        let conn = fixture_db();
        let dir = tempfile::tempdir().expect("tmp");
        let cg = dir.path().join("working").join("causal_graph.json");
        std::fs::create_dir_all(cg.parent().expect("p")).expect("mkdir");
        std::fs::write(&cg, serde_json::json!({"ts": 1}).to_string()).expect("write");
        // 既有 lineage 记录了一个旧 sha, 与实际文件不符 → 应被门禁拒绝
        update_cortex_lineage(&conn, "cortex_source://causal_graph", |l| {
            l.external_sha256 = Some("deadbeef".to_string());
            l.last_synced_at = Some(150);
        })?;
        let err = export_delta(&conn, dir.path(), false);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("版本门禁"));
        Ok(())
    }

    #[test]
    fn export_delta_refuses_when_unmounted() -> Result<(), String> {
        let conn = fixture_db();
        let missing = std::path::PathBuf::from("/Volumes/NeoTrixBrain__definitely_not_mounted");
        let err = export_delta(&conn, &missing, false);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("未挂载"));
        Ok(())
    }

    #[test]
    fn digest_sample_activates_cold_nodes() -> Result<(), String> {
        let conn = Connection::open_in_memory().expect("mem");
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn)
            .map_err(|e| e.to_string())?;
        // 造一个 temp corpus SQLite (live KB 的超集快照)
        let cdir = tempfile::tempdir().expect("tmp");
        let cpath = cdir.path().join("corpus.db");
        let cdb = Connection::open(&cpath).map_err(|e| e.to_string())?;
        cdb.execute(
            "CREATE TABLE nodes (id TEXT, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, metadata TEXT)",
            [],
        )
        .map_err(|e| e.to_string())?;
        for i in 0..3u32 {
            cdb.execute(
                "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,metadata) \
                 VALUES (?1,'article',?2,'s',?3,?4,'example.com','{}')",
                [
                    format!("c{i}"),
                    format!("t{i}"),
                    format!("content {i}"),
                    format!("zimid://cold/{i}"),
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        let r = digest_sample(&conn, &cpath, 10, None)?;
        assert_eq!(r.sampled, 3);
        assert_eq!(r.activated, 3);
        assert_eq!(r.already_live, 0);
        // lineage 应写入激活节点的 metadata
        let meta: String = conn
            .query_row(
                "SELECT metadata FROM nodes WHERE url='zimid://cold/0'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&meta).map_err(|e| e.to_string())?;
        assert!(v.get("lineage").is_some(), "lineage 应存在: {meta}");
        Ok(())
    }

    #[test]
    fn digest_sample_skips_already_live() -> Result<(), String> {
        let conn = Connection::open_in_memory().expect("mem");
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn)
            .map_err(|e| e.to_string())?;
        // 先在 live KB 放一个已存在的节点 (复用既有插入器, R-P42)
        insert_or_get_node_rows(
            &conn,
            "causal",
            NodeType::from_str("cortex_brain"),
            Some("s"),
            Some("cortex_source://causal_graph"),
            Some("example.com"),
        )
        .map_err(|e| e.to_string())?;
        let cdir = tempfile::tempdir().expect("tmp");
        let cpath = cdir.path().join("corpus.db");
        let cdb = Connection::open(&cpath).map_err(|e| e.to_string())?;
        cdb.execute(
            "CREATE TABLE nodes (id TEXT, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, metadata TEXT)",
            [],
        )
        .map_err(|e| e.to_string())?;
        // url 已在 live KB (fixture 自带 cortex_source://causal_graph) → 应跳过
        cdb.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,metadata) \
             VALUES ('x','cortex_brain','causal','s','c','cortex_source://causal_graph','example.com','{}')",
            [],
        )
        .map_err(|e| e.to_string())?;
        let r = digest_sample(&conn, &cpath, 10, None)?;
        assert_eq!(r.sampled, 1);
        assert_eq!(r.activated, 0);
        assert_eq!(r.already_live, 1);
        Ok(())
    }

    #[test]
    fn ingest_causal_graph_distills_high_signal_only() -> Result<(), String> {
        use crate::core::nt_core_e8::abduction::causal_graph::CausalGraph;
        let conn = Connection::open_in_memory().expect("mem");
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn)
            .map_err(|e| e.to_string())?;
        let dir = tempfile::tempdir().expect("tmp");
        let cg = dir.path().join("causal_graph.json");
        std::fs::write(&cg, serde_json::json!({"ts": 1700000000, "nodes": []}).to_string())
            .expect("write");
        // 构造因果图: a(高置信) → c, b(低置信) → c; c 度数=2 (高信号), b 度数=1 且低置信 (应被跳过)
        let mut g = CausalGraph::new();
        let a = g.add_node("原则: 模块化降低耦合".to_string(), 0.9);
        let b = g.add_node("噪声节点".to_string(), 0.1);
        let c = g.add_node("被多条边指向的核心概念".to_string(), 0.5);
        g.add_edge(a, c, "causal".to_string(), 0.9);
        g.add_edge(b, c, "causal".to_string(), 0.7);
        let derived = ingest_causal_graph(&conn, &g, Some(&cg))?;
        assert_eq!(derived, 2, "a(高置信)+c(高度数) 应被蒸馏, b 跳过");
        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE url LIKE 'cortex_source://causal_graph#%'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        assert_eq!(rows, 2);
        // 伞节点应带 sha (G2 锚点)
        let meta: String = conn
            .query_row(
                "SELECT metadata FROM nodes WHERE url='cortex_source://causal_graph'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&meta).map_err(|e| e.to_string())?;
        assert!(
            v.get("lineage").and_then(|l| l.get("external_sha256")).is_some(),
            "伞节点应含 sha: {meta}"
        );
        Ok(())
    }

    #[test]
    fn prune_external_only_removes_stale_orphans_with_lineage() -> Result<(), String> {
        let conn = Connection::open_in_memory().expect("mem");
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_schema::initialize(&conn)
            .map_err(|e| e.to_string())?;
        let cdir = tempfile::tempdir().expect("tmp");
        let cpath = cdir.path().join("corpus.db");
        let cdb = Connection::open(&cpath).map_err(|e| e.to_string())?;
        cdb.execute(
            "CREATE TABLE nodes (id TEXT, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, metadata TEXT)",
            [],
        )
        .map_err(|e| e.to_string())?;
        // 候选: 带 lineage + 陈旧 + live 无对应 → 应被删
        cdb.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,metadata) \
             VALUES ('a','article','t','s','c','zimid://stale/0','d',?)",
            [serde_json::json!({"lineage": {"last_synced_at": 1}}).to_string()],
        )
        .map_err(|e| e.to_string())?;
        // 纯冷存档 (无 lineage) → 永不被删
        cdb.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,metadata) \
             VALUES ('b','article','t2','s','c','zimid://cold/1','d','{}')",
            [],
        )
        .map_err(|e| e.to_string())?;
        // 带 lineage 但 live 仍有对应节点 → 保留
        cdb.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,metadata) \
             VALUES ('c','article','t3','s','c','zimid://live/2','d',?)",
            [serde_json::json!({"lineage": {"last_synced_at": 1}}).to_string()],
        )
        .map_err(|e| e.to_string())?;
        insert_or_get_node_rows(
            &conn, "live", NodeType::from_str("article"), Some("s"), Some("zimid://live/2"), Some("d"),
        )
        .map_err(|e| e.to_string())?;
        // dry-run 先计数
        let dry = prune_external(&conn, &cpath, 0, true)?;
        assert_eq!(dry, 1, "dry-run 应只数到 1 个候选 (a)");
        let removed = prune_external(&conn, &cpath, 0, false)?;
        assert_eq!(removed, 1);
        let remain: i64 = cdb
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        assert_eq!(remain, 2, "b(冷存档) 与 c(live 仍引用) 应保留");
        Ok(())
    }
}
