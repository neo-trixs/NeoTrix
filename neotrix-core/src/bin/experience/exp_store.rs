//! exp_store — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::{params, Connection};
use rusqlite::types::Value as SqlValue;
use serde_json::{json, Map, Value};
use std::io::Write;
use std::io::Read as _;
use super::{DAY, NS, SCHEMA_VERSION, VERIFY_DEFAULT_DAYS};
use super::exp_util::{VALUE_MAGIC, cycle_opt, now_ts};
use neotrix::l1_action::nt_memory::nt_memory_kb::nt_field_ledger;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use neotrix::l1_action::nt_memory::nt_memory_kb::nt_memory_schema;

pub(crate) fn value_encode(text: &str) -> Vec<u8> {
    let b = text.as_bytes();
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::new(6));
    let _ = enc.write_all(b);
    let c = enc.finish().expect("zlib encode");
    if c.len() + 4 < b.len() {
        let mut out = Vec::with_capacity(c.len() + 4);
        out.extend_from_slice(VALUE_MAGIC);
        out.extend_from_slice(&c);
        out
    } else {
        b.to_vec()
    }
}


pub(crate) fn value_decode(raw: &[u8]) -> Option<String> {
    if raw.len() >= 4 && &raw[..4] == VALUE_MAGIC {
        let mut dec = ZlibDecoder::new(&raw[4..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out).ok()?;
        String::from_utf8(out).ok()
    } else {
        String::from_utf8(raw.to_vec()).ok()
    }
}


pub(crate) fn kb_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.neotrix", home)
}


pub(crate) fn open_kb() -> Connection {
    let dir = kb_dir();
    // 新鲜 HOME (如刚装机) 无 .neotrix 目录时建目录, 否则首跑即 panic。
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[kb] 创建目录 {} 失败: {e}", dir);
        std::process::exit(1);
    }
    let db_path = format!("{}/knowledge.db", dir);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    conn.busy_timeout(std::time::Duration::from_secs(60)).ok();
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA busy_timeout=60000; PRAGMA synchronous=NORMAL;",
    )
    .ok();
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    conn
}

// ─── KV 基础读写 (透明压缩) ────────────────────────────────────────
/// 解码 SQL 值: TEXT 明文直接返回; BLOB 走透明压缩解压 (兼容 Python 双存储类型)。


pub(crate) fn sql_value_decode(v: &SqlValue) -> Option<String> {
    match v {
        SqlValue::Blob(b) => value_decode(b),
        SqlValue::Text(s) => Some(s.clone()),
        _ => None,
    }
}


pub(crate) fn kv_get(conn: &Connection, namespace: &str, key: &str) -> Option<String> {
    let row: rusqlite::Result<Option<SqlValue>> = conn.query_row(
        "SELECT value FROM kv_store WHERE namespace=?1 AND key=?2",
        params![namespace, key],
        |r| r.get(0),
    );
    match row {
        Ok(Some(v)) => sql_value_decode(&v),
        _ => None,
    }
}

/// 数据库忙时重试写操作: 并发进程 (如 absorb_guji) 持写锁时,
/// busy_timeout 可能失效或超时不足 (实测 DatabaseBusy panic at experience.rs:178),
/// 显式指数退避重试 (max 5 次, 总等待 ≤~3s), 仍失败则 panic 保留可见错误。


pub(crate) fn kv_set_retry(conn: &Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, rusqlite::Error> {
    let mut attempt = 0;
    loop {
        match conn.execute(sql, params) {
            Ok(n) => return Ok(n),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::DatabaseBusy
                    || e.code == rusqlite::ErrorCode::DatabaseLocked =>
            {
                attempt += 1;
                if attempt >= 5 {
                    return Err(rusqlite::Error::SqliteFailure(e, None));
                }
                let wait_ms = 100u64 << attempt; // 200,400,800,1600
                std::thread::sleep(std::time::Duration::from_millis(wait_ms));
            }
            Err(e) => return Err(e),
        }
    }
}


pub(crate) fn kv_set(conn: &Connection, namespace: &str, key: &str, value: &str) {
    let encoded = value_encode(value);
    kv_set_retry(
        conn,
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        &[&namespace, &key, &encoded, &now_ts()],
    )
    .expect("kv_set");
}

// ─── G1 场账本写路径 (W4): 追加型经验写入先 stage 暂存, 命令收尾统一 field_tick
// 一批一解求解下一版本 (哈希链审计 + 并发可交换合并)。覆盖型键保持 kv_set 直写 —
// 合并语义不匹配 (字典序最大 ≠ 最新状态), 属 G4 共识帧后续工作。
/// 场账本写者锚点 (field_journal 审计用)。
pub(crate) const FIELD_WRITER: &str = "absorption";

/// 追加型写入 → 场暂存 (不入正式状态)。暂存失败回退直写 — 数据不丢优先于审计完备。


pub(crate) fn kv_stage(conn: &Connection, namespace: &str, key: &str, value: &str) {
    if let Err(e) = nt_field_ledger::field_stage(conn, namespace, key, value, FIELD_WRITER) {
        eprintln!("[field] stage 失败回退直写 ({e}): {namespace}/{key}");
        kv_set(conn, namespace, key, value);
    }
}

/// 一批一解: 统一求解当前全部暂存条目 (空集幂等返回 None)。
/// 失败不致命 — 暂存留存于 field_staging, 下个命令的 tick 兜底重放。


pub(crate) fn field_solve(conn: &Connection) {
    match nt_field_ledger::field_tick(conn) {
        Ok(Some(r)) => eprintln!(
            "[field] tick v{} drained={} applied={}",
            r.version, r.drained, r.applied
        ),
        Ok(None) => {}
        Err(e) => eprintln!("[field] tick 延后 ({e}) — 暂存留存待下轮求解"),
    }
}

/// 收尾观测: 当前场版本号 (诊断)。


pub(crate) fn field_observe(conn: &Connection) {
    if let Ok(v) = nt_field_ledger::field_version(conn) {
        eprintln!("[field] field_version={v}");
    }
}

/// 批量扫描 namespace 下 key LIKE '<prefix>%' 的行, 统一透明解压。


pub(crate) fn scan_values(conn: &Connection, prefix: &str) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM kv_store WHERE namespace=?1 AND key LIKE ?2")
        .expect("scan_values prepare");
    let rows = stmt
        .query_map(params![NS, format!("{}%", prefix)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<SqlValue>>(1)?))
        })
        .expect("scan_values query_map");
    let mut out = Vec::new();
    for row in rows {
        let Ok((key, value)) = row else { continue };
        if let Some(v) = value {
            if let Some(decoded) = sql_value_decode(&v) {
                out.push((key, decoded));
            }
        }
    }
    out
}


pub(crate) fn load_json(conn: &Connection, namespace: &str, key: &str, default: Value) -> Value {
    match kv_get(conn, namespace, key) {
        Some(raw) => serde_json::from_str(&raw).unwrap_or(default),
        None => default,
    }
}


pub(crate) fn json_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}


// ────────────────────────────────────────────────────────────────
// Hub 引导与维护
// ────────────────────────────────────────────────────────────────
pub(crate) fn new_hub() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "hub": {
            "cycles": {},
            "branches": {},
            "dimensions": {},
            "route_table": {},
            "last_updated": 0,
        },
        "metrics": {
            "total_entries": 0,
            "by_type": {},
            "by_domain": {},
            "by_source": {},
        },
        "legacy_sources": {
            "absorption_cycle": true,
            "absorption": true,
            "meta_cognition": true,
        },
    })
}


pub(crate) fn ensure_hub(conn: &Connection) -> Value {
    match kv_get(conn, NS, "hub") {
        Some(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| new_hub()),
        None => new_hub(),
    }
}


pub(crate) fn save_hub(conn: &Connection, hub: &Value) {
    let mut h = hub.clone();
    h["schema_version"] = json!(SCHEMA_VERSION);
    h["hub"]["last_updated"] = json!(now_ts());
    kv_set(conn, NS, "hub", &h.to_string());
}

/// 自愈: 从实际分支全量重建 cycles/cycles 索引 (幂等), 消除幽灵/低估索引。


pub(crate) fn refresh_hub_metrics(conn: &Connection, hub: &mut Value) {
    let rows = scan_values(conn, "branch_");
    let ncells: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM kv_store WHERE namespace=?1 AND key LIKE 'concept_%'",
            params![NS],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let mut by_type: Map<String, Value> = Map::new();
    let mut by_domain: Map<String, Value> = Map::new();
    let mut by_source: Map<String, Value> = Map::new();
    let mut cycles: Map<String, Value> = Map::new();
    for (_, value) in &rows {
        let Ok(v) = serde_json::from_str::<Value>(value) else { continue };
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("unknown");
        let d = v.get("domain").and_then(|x| x.as_str()).unwrap_or("unknown");
        let s = v.get("source").and_then(|x| x.as_str()).unwrap_or("unknown");
        *by_type.entry(t.to_string()).or_insert(json!(0)) =
            json!(by_type.get(t).and_then(|x| x.as_i64()).unwrap_or(0) + 1);
        *by_domain.entry(d.to_string()).or_insert(json!(0)) =
            json!(by_domain.get(d).and_then(|x| x.as_i64()).unwrap_or(0) + 1);
        *by_source.entry(s.to_string()).or_insert(json!(0)) =
            json!(by_source.get(s).and_then(|x| x.as_i64()).unwrap_or(0) + 1);
        let cycle = cycle_opt(v.get("cycle")).unwrap_or_else(|| "unknown".to_string());
        let cmeta = cycles
            .entry(cycle.to_string())
            .or_insert_with(|| json!({"count": 0, "types": [], "domains": []}));
        let cmeta = cmeta.as_object_mut().expect("JSON object");
        let count = cmeta.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
        cmeta.insert("count".to_string(), json!(count + 1));
        let types = cmeta.entry("types".to_string()).or_insert_with(|| json!([]));
        if let Some(arr) = types.as_array() {
            if !arr.iter().any(|x| x.as_str() == Some(t)) {
                if let Some(arr) = types.as_array_mut() {
                    arr.push(json!(t));
                }
            }
        }
        let domains = cmeta
            .entry("domains".to_string())
            .or_insert_with(|| json!([]));
        if let Some(arr) = domains.as_array() {
            if !arr.iter().any(|x| x.as_str() == Some(d)) {
                if let Some(arr) = domains.as_array_mut() {
                    arr.push(json!(d));
                }
            }
        }
    }
    hub["hub"]["cycles"] = json!(cycles);
    hub["metrics"] = json!({
        "total_entries": rows.len(),
        "concepts": ncells,
        "by_type": by_type,
        "by_domain": by_domain,
        "by_source": by_source,
    });
}

// ────────────────────────────────────────────────────────────────
// verify_by 软过期
// ────────────────────────────────────────────────────────────────


pub(crate) fn norm_verify_by(v: &Value, ts: i64) -> Option<i64> {
    match v.get("verify_by") {
        None | Some(Value::Null) => Some(ts + VERIFY_DEFAULT_DAYS * DAY),
        Some(Value::Number(n)) => Some(n.as_i64().unwrap_or(ts + VERIFY_DEFAULT_DAYS * DAY)),
        Some(Value::String(s)) => {
            let s = s.trim();
            if s.is_empty() {
                return None;
            }
            if s.chars().all(|c| c.is_ascii_digit()) {
                return s.parse::<i64>().ok();
            }
            // 尝试 ISO 日期 %Y-%m-%d
            if let Ok(naive) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return naive
                    .and_hms_opt(0, 0, 0)
                    .map(|d| d.and_utc().timestamp());
            }
            Some(ts + VERIFY_DEFAULT_DAYS * DAY)
        }
        _ => None,
    }
}


pub(crate) fn is_stale(vb: Option<&Value>, now: i64) -> bool {
    let Some(vb) = vb else { return false };
    match vb {
        Value::Null => false,
        Value::String(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) {
                s.parse::<i64>().map(|n| n < now).unwrap_or(false)
            } else {
                false
            }
        }
        Value::Number(n) => n.as_i64().map(|n| n < now).unwrap_or(false),
        _ => false,
    }
}
