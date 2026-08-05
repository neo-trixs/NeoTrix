//! neotrix-experience — Unified End-of-Conversation Absorption Engine (Rust native,
//! 生产路径; 历史原型为 Python 版 `absorb_session.py`, 已退役).
//! 单入口: 每次会话结束时运行 `neotrix-experience absorb <session.json>`
//!   统一数据层: ~/.neotrix/knowledge.db 的 kv_store, namespace='experience'.
//!   统一 schema: 每条经验含 {schema_version, type, session_id, cycle, ts, domain,
//!                content, evidence, source, verify_by}.
//!   verify_by 软过期软门槛 — 经验默认 verify_by = ts + VERIFY_DEFAULT_DAYS,
//!   超期仍可检索但被 `stale` 标记,触发复核而非静默信任.
//!
//! 五阶段协议:
//!   1. Snapshot 快照     — 记录会话开始/结束上下文 (snapshot / close)
//!   2. Distill 蒸馏     — 从对话提取 patterns/rules/defects/insights (absorb)
//!   3. Classify 分类     — 映射到已有节点: domain + evidence (file:line|url)
//!   4. Persist 落盘     — 写入 kv_store experience namespace + 更新 hub 索引
//!   5. Feedback 反馈     — 同步 route_table 使后续 session 可检索 (query)
//!
//! 神经概念层: 概念=神经元 (唯一去重存储), 分支=经验节点,
//!   突触=概念↔分支引用, Hebb 共现=概念↔概念二阶联想。
//! value 透明压缩: 魔数 NTZ1 + zlib (仅当压缩有收益时), 读取透明解压。
//!
//! Usage:
//!   cargo run -p neotrix --bin neotrix-experience snapshot --cycle NNN --task "..." [--domain X]
//!   cargo run -p neotrix --bin neotrix-experience absorb <session.json>
//!   cargo run -p neotrix --bin neotrix-experience close --cycle NNN
//!   cargo run -p neotrix --bin neotrix-experience query --kw "关键词" [--type T] [--domain D] [--limit N] [--no-hebb]
//!   cargo run -p neotrix --bin neotrix-experience list [--type T] [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience stale [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience hub
//!   cargo run -p neotrix --bin neotrix-experience route --kw KEYWORD --branch BK
//!   cargo run -p neotrix --bin neotrix-experience neuron TERM [--exact]
//!   cargo run -p neotrix --bin neotrix-experience backfill
//!   cargo run -p neotrix --bin neotrix-experience prune [--stop WORD ...] [--stale-isolated]
//!   cargo run -p neotrix --bin neotrix-experience hebb
//!   cargo run -p neotrix --bin neotrix-experience compress [--all]
//!   cargo run -p neotrix --bin neotrix-experience gen-index [--out FILE] [--limit N]

use chrono::TimeZone;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use rusqlite::types::Value as SqlValue;
use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use sha1::{Digest, Sha1};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{Read, Write};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const NS: &str = "experience";
const SCHEMA_VERSION: i64 = 1;

const DOMAINS: [&str; 11] = [
    "NT-CORE", "NT-MIND", "NT-MEMORY", "NT-WORLD", "NT-ACT", "NT-IO", "NT-SHIELD",
    "NT-META", "NT-REPAIR", "NT-GOVERNANCE", "NT-NEXUS",
];
const TYPES: [&str; 6] = ["pattern", "rule", "defect", "insight", "cycle", "artifact"];

// verify_by 软过期: 经验默认审核周期, 超出后标记 stale 待复核而非静默信任
const VERIFY_DEFAULT_DAYS: i64 = 90;
const DAY: i64 = 86400;

// 中文停用字 (单字无意义) 与英文停用词
const CN_STOP: &str = "的了是在与和就都而这于有也一个我你他她它其之为此对从地向到";
const EN_STOP: [&str; 66] = [
    "the", "and", "for", "with", "from", "that", "this", "into", "were", "was",
    "will", "have", "has", "had", "not", "are", "but", "its", "than", "then",
    "when", "where", "which", "while", "should", "would", "could", "can", "may",
    "must", "only", "over", "under", "also", "been", "being", "does", "did",
    "doing", "how", "what", "why", "our", "your", "their", "his", "her",
    "area", "tree", "pass", "note", "status", "check", "action",
    "these", "those", "them", "they", "there", "here", "each", "both", "some",
    "such", "very", "just",
];

const CONCEPT_MIN_LEN: usize = 3;
const CONCEPT_MAX_LEN: usize = 8;

fn en_stop() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| EN_STOP.iter().copied().collect())
}

fn cn_stop() -> &'static HashSet<char> {
    static SET: OnceLock<HashSet<char>> = OnceLock::new();
    SET.get_or_init(|| CN_STOP.chars().collect())
}

// ─── value 透明压缩层 (方案 D) ─────────────────────────────────────
const VALUE_MAGIC: &[u8] = b"NTZ1";

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn value_encode(text: &str) -> Vec<u8> {
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

fn value_decode(raw: &[u8]) -> Option<String> {
    if raw.len() >= 4 && &raw[..4] == VALUE_MAGIC {
        let mut dec = ZlibDecoder::new(&raw[4..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out).ok()?;
        String::from_utf8(out).ok()
    } else {
        String::from_utf8(raw.to_vec()).ok()
    }
}

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
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
fn sql_value_decode(v: &SqlValue) -> Option<String> {
    match v {
        SqlValue::Blob(b) => value_decode(b),
        SqlValue::Text(s) => Some(s.clone()),
        _ => None,
    }
}

fn kv_get(conn: &Connection, namespace: &str, key: &str) -> Option<String> {
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

fn kv_set(conn: &Connection, namespace: &str, key: &str, value: &str) {
    let encoded = value_encode(value);
    conn.execute(
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![namespace, key, encoded, now_ts()],
    )
    .expect("kv_set");
}

/// 批量扫描 namespace 下 key LIKE '<prefix>%' 的行, 统一透明解压。
fn scan_values(conn: &Connection, prefix: &str) -> Vec<(String, String)> {
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

fn load_json(conn: &Connection, namespace: &str, key: &str, default: Value) -> Value {
    match kv_get(conn, namespace, key) {
        Some(raw) => serde_json::from_str(&raw).unwrap_or(default),
        None => default,
    }
}

fn json_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Python `str(float)` 表示: 整值浮点带 `.0` (5.0 → "5.0", 5.5 → "5.5").
fn py_float_repr(f: f64) -> String {
    if f.fract() == 0.0 && f.is_finite() {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

/// Python `repr(dict)` 表示: `{'k': v, 'k2': v2}` (单引号, `: ` 与 `, ` 分隔).
fn py_dict_repr(v: &Value) -> String {
    match v {
        Value::Object(m) => {
            if m.is_empty() {
                return "{}".to_string();
            }
            let items: Vec<String> = m
                .iter()
                .map(|(k, val)| format!("'{}': {}", k, py_repr_scalar(val)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        _ => v.to_string(),
    }
}

fn py_repr_scalar(v: &Value) -> String {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                py_float_repr(f)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("'{}'", s),
        Value::Bool(b) => b.to_string(),
        Value::Null => "None".to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(py_repr_scalar).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(_) => v.to_string(),
    }
}

/// cycle 字段可能是 JSON 字符串或数字 (历史分支), 统一归一化为字符串 (对应 Python `str()`).
fn cycle_opt(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

// ────────────────────────────────────────────────────────────────
// 神经概念层 (Neural Concept Layer)
// ────────────────────────────────────────────────────────────────
fn concept_hash(term: &str) -> String {
    let mut h = Sha1::new();
    h.update(term.as_bytes());
    hex::encode(h.finalize())[..16].to_string()
}

/// 从经验正文抽取概念词集 — 神经网络化的关键一步。
fn extract_concepts(content: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if content.is_empty() {
        return found;
    }
    let en_re = regex::Regex::new(r"[A-Za-z][A-Za-z0-9_/\-]{3,}").unwrap();
    let cn_re = regex::Regex::new(r"[\u{4e00}-\u{9fff}]{3,12}").unwrap();
    for m in en_re.find_iter(content) {
        let tok = m.as_str().trim();
        if tok.is_empty() {
            continue;
        }
        let low = tok.to_lowercase();
        if en_stop().contains(low.as_str()) || tok.len() < 4 {
            continue;
        }
        found.insert(tok.to_string());
    }
    for m in cn_re.find_iter(content) {
        let tok = m.as_str().trim();
        if tok.is_empty() {
            continue;
        }
        let n = tok.chars().count();
        if n < CONCEPT_MIN_LEN {
            continue;
        }
        if n <= CONCEPT_MAX_LEN {
            if !tok.chars().all(|c| cn_stop().contains(&c)) {
                found.insert(tok.to_string());
            }
            continue;
        }
        // 超长中文短语: 重叠滑窗切 3-4 字词, 丢弃全停用窗口
        let chars: Vec<char> = tok.chars().collect();
        let step = 2;
        for win in [3usize, 4] {
            let mut i = 0;
            while i + win <= chars.len() {
                let sub: String = chars[i..i + win].iter().collect();
                if !sub.chars().all(|c| cn_stop().contains(&c)) {
                    found.insert(sub);
                }
                i += step;
            }
        }
    }
    found
}

fn load_concept(conn: &Connection, ch: &str) -> Option<Value> {
    let raw = kv_get(conn, NS, &format!("concept_{}", ch))?;
    serde_json::from_str(&raw).ok()
}

/// 概念去重落盘: 已存在 → 加引用+count; 不存在 → 新建神经元。
fn concept_from_branch(conn: &Connection, term: &str, branch_key: &str, domain: &str) -> String {
    let ch = concept_hash(&term.to_lowercase());
    let key = format!("concept_{}", ch);
    let mut c = load_json(conn, NS, &key, Value::Null);
    if c.is_null() {
        c = json!({
            "schema_version": 1,
            "type": "concept",
            "id": ch,
            "term": term,
            "count": 0,
            "branches": [],
            "co": [],
            "co_w": {},
            "domains": {},
            "ts": now_ts(),
        });
    }
    let branches = c.get_mut("branches").unwrap().as_array_mut().unwrap();
    if !branches.iter().any(|b| b.as_str() == Some(branch_key)) {
        branches.push(json!(branch_key));
    }
    c["count"] = json!(branches.len());
    let mut domains = c
        .get("domains")
        .and_then(|d| d.as_object())
        .cloned()
        .unwrap_or_default();
    let cur = domains.get(domain).and_then(|v| v.as_i64()).unwrap_or(0);
    domains.insert(domain.to_string(), json!(cur + 1));
    c["domains"] = json!(domains);
    kv_set(conn, NS, &key, &c.to_string());
    ch
}

/// 归一化读取概念共现映射 → {other_hash: weight} (兼容新旧格式)。
fn co_full(c: &Value) -> HashMap<String, f64> {
    let mut full = HashMap::new();
    match c.get("co") {
        Some(Value::Array(arr)) => {
            for h in arr {
                if let Some(s) = h.as_str() {
                    full.insert(s.to_string(), 1.0);
                }
            }
        }
        Some(Value::Object(map)) => {
            for (k, v) in map {
                full.insert(k.clone(), v.as_f64().unwrap_or(1.0));
            }
        }
        _ => {}
    }
    if let Some(cow) = c.get("co_w").and_then(|m| m.as_object()) {
        for (k, v) in cow {
            full.insert(k.clone(), v.as_f64().unwrap_or(1.0));
        }
    }
    full
}

/// 为概念 a 的共现映射累加 b 的权重 (对称调用, 双侧各自存储保持 O(1) 读取)。
fn co_bump(conn: &Connection, a: &str, b: &str) {
    let Some(mut ca) = load_concept(conn, a) else { return };
    let mut cow = ca
        .get_mut("co_w")
        .and_then(|m| m.as_object_mut())
        .map(|m| m.clone())
        .unwrap_or_default();
    let mut co = ca
        .get_mut("co")
        .and_then(|c| c.as_array_mut())
        .map(|a| a.clone())
        .unwrap_or_default();
    if let Some(w) = cow.get(b).and_then(|v| v.as_i64()) {
        cow.insert(b.to_string(), json!(w + 1));
    } else if co.iter().any(|v| v.as_str() == Some(b)) {
        co.retain(|v| v.as_str() != Some(b));
        cow.insert(b.to_string(), json!(2));
    } else {
        co.push(json!(b));
    }
    let obj = ca.as_object_mut().unwrap();
    if co.is_empty() {
        obj.remove("co");
    } else {
        obj.insert("co".to_string(), json!(co));
    }
    if cow.is_empty() {
        obj.remove("co_w");
    } else {
        obj.insert("co_w".to_string(), json!(cow));
    }
    kv_set(conn, NS, &format!("concept_{}", a), &ca.to_string());
}

/// Hebb 共现突触: 对同一分支内的概念两两累加共现权重 (对称)。
fn hebb_cooccurrence(conn: &Connection, hashes: &[String]) {
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            co_bump(conn, &hashes[i], &hashes[j]);
            co_bump(conn, &hashes[j], &hashes[i]);
        }
    }
}

/// 内存版 co_bump (cmd_hebb 全量重建用, 避免逐对 commit 的 O(n) DB 往返)。
fn co_bump_in_mem(concepts: &mut HashMap<String, Value>, x: &str, y: &str) {
    let Some(c) = concepts.get_mut(x) else { return };
    let obj = c.as_object_mut().unwrap();
    let in_cow = obj.get("co_w").and_then(|m| m.get(y)).cloned();
    if let Some(w) = in_cow {
        obj.get_mut("co_w")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(y.to_string(), json!(w.as_i64().unwrap_or(1) + 1));
        return;
    }
    let in_co = obj
        .get("co")
        .map(|v| {
            v.as_array()
                .map(|a| a.iter().any(|v| v.as_str() == Some(y)))
                .unwrap_or(false)
        })
        .unwrap_or(false);
    if in_co {
        obj.get_mut("co")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .retain(|v| v.as_str() != Some(y));
        obj.get_mut("co_w")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(y.to_string(), json!(2));
    } else {
        obj.get_mut("co")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(json!(y));
    }
}

// ────────────────────────────────────────────────────────────────
// Hub 引导与维护
// ────────────────────────────────────────────────────────────────
fn new_hub() -> Value {
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

fn ensure_hub(conn: &Connection) -> Value {
    match kv_get(conn, NS, "hub") {
        Some(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| new_hub()),
        None => new_hub(),
    }
}

fn save_hub(conn: &Connection, hub: &Value) {
    let mut h = hub.clone();
    h["schema_version"] = json!(SCHEMA_VERSION);
    h["hub"]["last_updated"] = json!(now_ts());
    kv_set(conn, NS, "hub", &h.to_string());
}

/// 自愈: 从实际分支全量重建 cycles/cycles 索引 (幂等), 消除幽灵/低估索引。
fn refresh_hub_metrics(conn: &Connection, hub: &mut Value) {
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
        let cmeta = cmeta.as_object_mut().unwrap();
        let count = cmeta.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
        cmeta.insert("count".to_string(), json!(count + 1));
        let types = cmeta.entry("types".to_string()).or_insert_with(|| json!([]));
        if !types
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some(t))
        {
            types.as_array_mut().unwrap().push(json!(t));
        }
        let domains = cmeta
            .entry("domains".to_string())
            .or_insert_with(|| json!([]));
        if !domains
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some(d))
        {
            domains.as_array_mut().unwrap().push(json!(d));
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
fn norm_verify_by(v: &Value, ts: i64) -> Option<i64> {
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

fn is_stale(vb: Option<&Value>, now: i64) -> bool {
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

fn fmt_ts(ts: i64) -> String {
    chrono::Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string())
}

// ────────────────────────────────────────────────────────────────
// 1. Snapshot 快照 / Close
// ────────────────────────────────────────────────────────────────
fn cmd_snapshot(conn: &Connection, cycle: &str, task: &str, domain: &str) {
    ensure_hub(conn);
    let now = now_ts();
    let sid = format!("sess_{}_{}", now, &uuid_hex(8));
    let snap = json!({
        "type": "cycle",
        "session_id": sid,
        "cycle": cycle,
        "ts": now,
        "started_at": now,
        "ended_at": Value::Null,
        "task": task,
        "domain": domain,
        "content": task,
        "evidence": "",
        "source": "dialogue",
        "duration_s": Value::Null,
    });
    kv_set(conn, NS, &format!("snapshot_{}", sid), &snap.to_string());
    println!("[snapshot] {} (cycle={})", sid, cycle);
}

fn cmd_close(conn: &Connection, cycle: &str) {
    let rows = scan_values(conn, "snapshot_");
    let now = now_ts();
    let mut closed = 0;
    for (key, value) in rows {
        let Ok(mut v) = serde_json::from_str::<Value>(&value) else { continue };
        if v.get("cycle").and_then(|c| c.as_str()) == Some(cycle) && v.get("ended_at").is_none() {
            let started = v.get("started_at").and_then(|s| s.as_i64()).unwrap_or(now);
            v["ended_at"] = json!(now);
            v["duration_s"] = json!(now - started);
            kv_set(conn, NS, &key, &v.to_string());
            closed += 1;
        }
    }
    println!("[close] {} snapshot(s) closed for cycle={}", closed, cycle);
}

fn uuid_hex(len: usize) -> String {
    uuid::Uuid::new_v4().simple().to_string()[..len].to_string()
}

// ────────────────────────────────────────────────────────────────
// 2-4. Absorb 蒸馏 → 分类 → 落盘
// ────────────────────────────────────────────────────────────────
fn validate_entry(e: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    match e.get("content") {
        Some(v) if json_truthy(v) => {}
        _ => errors.push("content is required".to_string()),
    }
    if let Some(t) = e.get("type").and_then(|x| x.as_str()) {
        if !TYPES.contains(&t) {
            errors.push(format!("type must be one of {:?}", TYPES));
        }
    }
    if let Some(d) = e.get("domain").and_then(|x| x.as_str()) {
        if !DOMAINS.contains(&d) {
            errors.push(format!("domain must be one of {:?}", DOMAINS));
        }
    }
    errors
}

fn cmd_absorb(conn: &Connection, session_path: &str) {
    let mut hub = ensure_hub(conn);
    let raw = std::fs::read_to_string(session_path).expect("read session.json");
    let session: Value = serde_json::from_str(&raw).expect("session.json is valid JSON");

    let sid = session
        .get("session_id")
        .and_then(|s| s.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("sess_{}_{}", now_ts(), uuid_hex(8)));
    let cycle = cycle_opt(session.get("cycle")).unwrap_or_else(|| "unknown".to_string());
    let ts = session.get("ts").and_then(|t| t.as_i64()).unwrap_or_else(now_ts);

    let mut entries: Vec<Value> = session
        .get("entries")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();
    if entries.is_empty() && session.get("content").is_some() {
        entries.push(json!({
            "type": "cycle",
            "content": session.get("content").cloned().unwrap_or(Value::Null),
            "domain": session.get("domain").cloned(),
            "evidence": session.get("evidence").cloned().unwrap_or_else(|| json!("")),
            "source": session.get("source").cloned().unwrap_or_else(|| json!("dialogue")),
        }));
    }

    let mut written = 0;
    for (i, raw_entry) in entries.iter().enumerate() {
        let mut e = json!({
            "schema_version": SCHEMA_VERSION,
            "type": raw_entry.get("type").cloned().unwrap_or_else(|| json!("insight")),
            "session_id": sid,
            "cycle": cycle,
            "ts": ts,
            "domain": raw_entry.get("domain").or_else(|| session.get("domain"))
                .cloned().unwrap_or_else(|| json!("unknown")),
            "content": raw_entry.get("content").cloned().unwrap_or_else(|| json!("")),
            "evidence": raw_entry.get("evidence").cloned().unwrap_or_else(|| json!("")),
            "source": raw_entry.get("source").or_else(|| session.get("source"))
                .cloned().unwrap_or_else(|| json!("dialogue")),
        });
        if let Some(vb) = norm_verify_by(&e, ts) {
            e["verify_by"] = json!(vb);
        }
        let errors = validate_entry(&e);
        if !errors.is_empty() {
            println!("[absorb] ✗ entry #{}: {:?}", i, errors);
            continue;
        }
        let key = format!("branch_{}_{}_{}", cycle, i, uuid_hex(6));
        // 神经网络化: 提取概念 → 去重神经元 → 分支保存概念引用 (内容词不重复落盘)
        let concepts = extract_concepts(
            e.get("content").and_then(|c| c.as_str()).unwrap_or(""),
        );
        let mut chs = Vec::new();
        let domain = e.get("domain").and_then(|d| d.as_str()).unwrap_or("unknown");
        for term in concepts {
            chs.push(concept_from_branch(conn, &term, &key, domain));
        }
        e["concepts"] = json!(chs);
        kv_set(conn, NS, &key, &e.to_string());
        // Hebb 共现突触: 同分支概念两两强化关联 (fire together, wire together)
        hebb_cooccurrence(conn, &chs);
        // 更新 hub cycle 索引
        let cycles = hub["hub"]["cycles"].as_object_mut().unwrap();
        let cmeta = cycles
            .entry(cycle.clone())
            .or_insert_with(|| json!({"count": 0, "types": [], "domains": []}));
        let cmeta = cmeta.as_object_mut().unwrap();
        let count = cmeta.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
        cmeta.insert("count".to_string(), json!(count + 1));
        let types = cmeta.entry("types".to_string()).or_insert_with(|| json!([]));
        let ty = e.get("type").cloned().unwrap_or_else(|| json!("insight"));
        if !types.as_array().unwrap().contains(&ty) {
            types.as_array_mut().unwrap().push(ty);
        }
        let domains = cmeta
            .entry("domains".to_string())
            .or_insert_with(|| json!([]));
        let dom = e
            .get("domain")
            .cloned()
            .unwrap_or_else(|| json!("unknown"));
        if !domains.as_array().unwrap().contains(&dom) {
            domains.as_array_mut().unwrap().push(dom);
        }
        written += 1;
    }

    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("[absorb] {} entries from {} (cycle={})", written, sid, cycle);
}

// ────────────────────────────────────────────────────────────────
// 5. Feedback 反馈 / 查询
// ────────────────────────────────────────────────────────────────
/// 突触联想检索: 输入词 → 命中概念神经元 → 沿突触扩散到分支(1阶, 主结果) →
/// Hebb 共现扩散到关联概念(2阶, 仅二阶且权重衰减, 作为相关推荐)。
fn neural_associative(conn: &Connection, kws: &[String], hebb: bool) -> Option<Vec<(String, f64, i64)>> {
    if kws.is_empty() {
        return None;
    }
    // 1. 词 → 概念哈希, 直接命中神经元 (去重: 概念只存一份)
    let mut neuron_hits: HashSet<String> = HashSet::new();
    for kw in kws {
        neuron_hits.insert(concept_hash(kw));
    }
    // 1b. 词 → 子串模糊匹配概念 (CJK 长短语不可达修复), 并入激活集合
    let fuzzy: Vec<&String> = kws
        .iter()
        .filter(|k| !k.is_ascii() || k.len() >= 4)
        .collect();
    if !fuzzy.is_empty() {
        for (_, value) in scan_values(conn, "concept_") {
            let Ok(c) = serde_json::from_str::<Value>(&value) else { continue };
            let term = c
                .get("term")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_lowercase();
            if !term.is_empty() && fuzzy.iter().any(|k| term.contains(k.as_str())) {
                if let Some(id) = c.get("id").and_then(|i| i.as_str()) {
                    neuron_hits.insert(id.to_string());
                }
            }
        }
    }
    // 一阶: 直接命中神经元 → 其分支 (无扩散)
    let mut order1: HashMap<String, i64> = HashMap::new();
    let mut first_neurons: Vec<Value> = Vec::new();
    for ch in &neuron_hits {
        let Some(c) = load_concept(conn, ch) else { continue };
        first_neurons.push(c.clone());
        if let Some(bs) = c.get("branches").and_then(|b| b.as_array()) {
            for b in bs {
                if let Some(s) = b.as_str() {
                    *order1.entry(s.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    if !hebb {
        if order1.is_empty() {
            return None;
        }
        let mut v: Vec<(String, f64, i64)> = order1
            .into_iter()
            .map(|(b, s)| (b, s as f64, 1))
            .collect();
        v.sort_by(|x, y| {
            y.1.partial_cmp(&x.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| x.0.cmp(&y.0))
        });
        return Some(v);
    }
    // 二阶: Hebb 共现扩散 — 与命中概念同现的关联概念 (只一跳), 其分支获得衰减权重
    let mut order2: HashMap<String, f64> = HashMap::new();
    for c in &first_neurons {
        let co = co_full(c);
        if co.is_empty() {
            continue;
        }
        let co_max = co.values().cloned().fold(1.0f64, f64::max);
        for (oth_ch, w) in co {
            if neuron_hits.contains(&oth_ch) {
                continue;
            }
            let Some(oth) = load_concept(conn, &oth_ch) else { continue };
            let boost = 0.5 * (w / co_max);
            if let Some(bs) = oth.get("branches").and_then(|b| b.as_array()) {
                for b in bs {
                    if let Some(s) = b.as_str() {
                        *order2.entry(s.to_string()).or_insert(0.0) += boost;
                    }
                }
            }
        }
    }
    // 合并: 一阶优先, 二阶作为相关推荐 (分数 *0.1 压后, 避免淹没直接命中)
    let mut merged: HashMap<String, (f64, i64)> = HashMap::new();
    for (b, s) in order1 {
        merged.insert(b, (s as f64, 1));
    }
    for (b, s) in order2 {
        if let Some((s1, _)) = merged.get_mut(&b) {
            *s1 += s * 0.1;
        } else {
            merged.insert(b, (s, 2));
        }
    }
    if merged.is_empty() {
        return None;
    }
    let mut out: Vec<(String, f64, i64)> = merged
        .into_iter()
        .map(|(b, (s, o))| (b, s, o))
        .collect();
    out.sort_by(|x, y| {
        y.2.cmp(&x.2)
            .then_with(|| {
                y.1.partial_cmp(&x.1).unwrap_or(Ordering::Equal)
            })
            .then_with(|| x.0.cmp(&y.0))
    });
    Some(out)
}

#[derive(Clone)]
struct QueryResult {
    cycle: Option<String>,
    ty: String,
    domain: String,
    content: String,
    evidence: String,
    key: String,
    verify_by: Option<Value>,
    score: f64,
    order: i64,
}

fn cmd_query(conn: &Connection, kw: &str, ty: Option<&str>, domain: Option<&str>, limit: usize, no_hebb: bool) {
    ensure_hub(conn);
    let kws: Vec<String> = kw
        .split_whitespace()
        .map(|k| k.to_lowercase())
        .collect();
    let rows = scan_values(conn, "branch_");
    let mut cache: HashMap<String, Option<Value>> = HashMap::new();
    for (key, value) in rows {
        cache.insert(key, serde_json::from_str(&value).ok());
    }
    let mut results: Vec<QueryResult> = Vec::new();
    let synapse = neural_associative(conn, &kws, !no_hebb);
    if let Some(syn) = synapse {
        for (key, score, order) in syn {
            let Some(Some(v)) = cache.get(&key) else { continue };
            if let Some(t) = ty {
                if v.get("type").and_then(|x| x.as_str()) != Some(t) {
                    continue;
                }
            }
            if let Some(d) = domain {
                if v.get("domain").and_then(|x| x.as_str()) != Some(d) {
                    continue;
                }
            }
            results.push(QueryResult {
                cycle: cycle_opt(v.get("cycle")),
                ty: v.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                domain: v.get("domain").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                content: truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 100),
                evidence: v.get("evidence").and_then(|e| e.as_str()).unwrap_or("").to_string(),
                key,
                verify_by: v.get("verify_by").cloned(),
                score,
                order,
            });
        }
        results.sort_by(|a, b| {
            a.order
                .cmp(&b.order)
                .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal))
                .then_with(|| {
                    a.cycle
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.cycle.as_deref().unwrap_or(""))
                })
        });
    } else {
        for (key, v) in &cache {
            let Some(v) = v else { continue };
            if !kws.is_empty() {
                let blob = format!(
                    "{} {} {}",
                    v.get("content").and_then(|c| c.as_str()).unwrap_or(""),
                    v.get("domain").and_then(|d| d.as_str()).unwrap_or(""),
                    v.get("evidence").and_then(|e| e.as_str()).unwrap_or("")
                )
                .to_lowercase();
                if !kws.iter().all(|k| blob.contains(k)) {
                    continue;
                }
            }
            if let Some(t) = ty {
                if v.get("type").and_then(|x| x.as_str()) != Some(t) {
                    continue;
                }
            }
            if let Some(d) = domain {
                if v.get("domain").and_then(|x| x.as_str()) != Some(d) {
                    continue;
                }
            }
            results.push(QueryResult {
                cycle: cycle_opt(v.get("cycle")),
                ty: v.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                domain: v.get("domain").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                content: truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 100),
                evidence: v.get("evidence").and_then(|e| e.as_str()).unwrap_or("").to_string(),
                key: key.clone(),
                verify_by: v.get("verify_by").cloned(),
                score: 0.0,
                order: 0,
            });
        }
        results.sort_by(|a, b| {
            a.order
                .cmp(&b.order)
                .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal))
                .then_with(|| {
                    a.cycle
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.cycle.as_deref().unwrap_or(""))
                })
        });
    }
    let now = now_ts();
    let shown = results.len().min(limit);
    for r in &results[..shown] {
        let stale_mark = if is_stale(r.verify_by.as_ref(), now) {
            "[STALE] "
        } else {
            ""
        };
        let act = if r.score != 0.0 {
            format!(" μ={}", py_float_repr(r.score))
        } else {
            String::new()
        };
        let hop = match r.order {
            0 => "",
            2 => "[关联]",
            _ => "",
        };
        println!(
            "[{}] {:8} {:16} {}{}{}{}",
            r.cycle.as_deref().unwrap_or("None"),
            r.ty,
            r.domain,
            stale_mark,
            hop,
            act,
            r.content
        );
        if !r.evidence.is_empty() {
            println!("        evidence: {}", r.evidence);
        }
        println!("        key: {}", r.key);
    }
    println!(
        "[query] {} match(es), showing {}",
        results.len(),
        shown
    );
}

fn cmd_list(conn: &Connection, ty: Option<&str>, domain: Option<&str>) {
    ensure_hub(conn);
    let rows = scan_values(conn, "branch_");
    let mut count = 0;
    for (_, value) in rows {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        if let Some(t) = ty {
            if v.get("type").and_then(|x| x.as_str()) != Some(t) {
                continue;
            }
        }
        if let Some(d) = domain {
            if v.get("domain").and_then(|x| x.as_str()) != Some(d) {
                continue;
            }
        }
        count += 1;
        println!(
            "[{}] {:8} {:16} {}",
            cycle_opt(v.get("cycle")).unwrap_or_else(|| "?".to_string()),
            v.get("type").and_then(|x| x.as_str()).unwrap_or("?"),
            v.get("domain").and_then(|x| x.as_str()).unwrap_or("?"),
            truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 80)
        );
    }
    println!("[list] {} entries", count);
}

/// 列出已过期 (verify_by < now) 的分支 — 复核清单而非删除。
fn cmd_stale(conn: &Connection, domain: Option<&str>) {
    ensure_hub(conn);
    let now = now_ts();
    let rows = scan_values(conn, "branch_");
    let mut stale: Vec<Value> = Vec::new();
    for (_, value) in rows {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        if let Some(d) = domain {
            if v.get("domain").and_then(|x| x.as_str()) != Some(d) {
                continue;
            }
        }
        if is_stale(v.get("verify_by"), now) {
            stale.push(v);
        }
    }
    stale.sort_by(|a, b| {
        let av = a.get("verify_by").and_then(|x| x.as_i64()).unwrap_or(now);
        let bv = b.get("verify_by").and_then(|x| x.as_i64()).unwrap_or(now);
        av.cmp(&bv)
    });
    for v in &stale {
        let vb = v.get("verify_by").and_then(|x| x.as_i64()).unwrap_or(now);
        let days = (now - vb) / DAY;
        println!(
            "[{}] {:8} {:16} +{}d overdue  v_by={}  {}",
            cycle_opt(v.get("cycle")).unwrap_or_else(|| "?".to_string()),
            v.get("type").and_then(|x| x.as_str()).unwrap_or("?"),
            v.get("domain").and_then(|x| x.as_str()).unwrap_or("?"),
            days,
            vb,
            truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 70)
        );
    }
    println!(
        "[stale] {} branch(es) past verify_by ({})",
        stale.len(),
        fmt_ts(now)
    );
}

fn cmd_hub(conn: &Connection) {
    let mut hub = ensure_hub(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("{}", serde_json::to_string_pretty(&hub).unwrap());
}

fn cmd_route(conn: &Connection, kw: &str, branch: &str) {
    let mut hub = ensure_hub(conn);
    let mut list = hub["hub"]["route_table"]
        .as_object_mut()
        .expect("route_table object")
        .entry(kw.to_string())
        .or_insert_with(|| json!([]))
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !list.iter().any(|b| b.as_str() == Some(branch)) {
        list.push(json!(branch));
    }
    hub["hub"]["route_table"][kw] = json!(list);
    save_hub(conn, &hub);
    println!("[route] '{}' → {}", kw, json!(list));
}

/// 神经概念图检视: 显示概念神经元的突触 (引用它的分支) 与联想扩散。
fn cmd_neuron(conn: &Connection, term: &str, exact: bool) {
    let mut c: Option<Value> = None;
    if exact {
        let ch = concept_hash(&term.to_lowercase());
        let val = load_json(conn, NS, &format!("concept_{}", ch), Value::Null);
        if !val.is_null() {
            c = Some(val);
        }
    }
    if c.is_none() {
        let rows = scan_values(conn, "concept_");
        let mut cands: Vec<Value> = Vec::new();
        let tl = term.to_lowercase();
        for (_, value) in rows {
            let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
            if v.get("term")
                .and_then(|t| t.as_str())
                .map(|t| t.to_lowercase().contains(&tl))
                .unwrap_or(false)
            {
                cands.push(v);
            }
        }
        cands.sort_by(|a, b| {
            b.get("count")
                .and_then(|x| x.as_i64())
                .unwrap_or(0)
                .cmp(&a.get("count").and_then(|x| x.as_i64()).unwrap_or(0))
        });
        if cands.is_empty() {
            println!("[neuron] 未找到概念 '{}'", term);
            return;
        }
        if cands.len() > 1 {
            println!(
                "[neuron] '{}' 匹配 {} 个概念, 选最高激活: '{}'",
                term,
                cands.len(),
                cands[0].get("term").and_then(|t| t.as_str()).unwrap_or("")
            );
        }
        c = Some(cands.remove(0));
    }
    let c = c.unwrap();
    let term_disp = c.get("term").and_then(|t| t.as_str()).unwrap_or("");
    let id = c.get("id").and_then(|i| i.as_str()).unwrap_or("");
    let branches = c.get("branches").and_then(|b| b.as_array()).cloned().unwrap_or_default();
    println!("神经元: {}  (id={})", term_disp, id);
    println!(
        "  激活度(count): {}  引用分支: {}",
        c.get("count").and_then(|x| x.as_i64()).unwrap_or(0),
        branches.len()
    );
    println!(
        "  域分布: {}",
        py_dict_repr(&c.get("domains").cloned().unwrap_or_else(|| json!({})))
    );
    for (i, b) in branches.iter().enumerate() {
        if i >= 20 {
            break;
        }
        let Some(bk) = b.as_str() else { continue };
        let Some(raw) = kv_get(conn, NS, bk) else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else { continue };
        println!(
            "   ↳ {}  [{}] {} {} {}",
            bk,
            cycle_opt(v.get("cycle")).unwrap_or_else(|| "None".to_string()),
            v.get("type").and_then(|t| t.as_str()).unwrap_or(""),
            v.get("domain").and_then(|d| d.as_str()).unwrap_or(""),
            truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 80)
        );
    }
}

/// 回填: 为已存在的 (无 concepts) 分支重建概念神经元与突触链路 (幂等)。
fn cmd_backfill(conn: &Connection) {
    let rows = scan_values(conn, "branch_");
    let mut built = 0;
    let mut skipped = 0;
    for (key, value) in rows {
        let Ok(mut v) = serde_json::from_str::<Value>(&value) else { continue };
        if v.get("concepts")
            .map(json_truthy)
            .unwrap_or(false)
        {
            skipped += 1;
            continue;
        }
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let domain = v.get("domain").and_then(|d| d.as_str()).unwrap_or("unknown");
        let mut chs = Vec::new();
        for term in extract_concepts(content) {
            chs.push(concept_from_branch(conn, &term, &key, domain));
        }
        v["concepts"] = json!(chs);
        kv_set(conn, NS, &key, &v.to_string());
        built += 1;
    }
    let mut hub = ensure_hub(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!(
        "[backfill] {} branch(es) rebuilt synapse graph; {} already had concepts",
        built, skipped
    );
}

/// 神经网络清理 (Neural Pruning): 移除停用词污染产生的低价值概念神经元 (幂等)。
fn cmd_prune(conn: &Connection, extra_stop: &[String], stale_isolated: bool) {
    let rows = scan_values(conn, "concept_");
    let mut stop: HashSet<String> = en_stop().iter().map(|s| s.to_string()).collect();
    for s in extra_stop {
        stop.insert(s.to_lowercase());
    }
    let mut removed = 0;
    let mut dropped = 0;
    let mut branches: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in &rows {
        let Ok(c) = serde_json::from_str::<Value>(value) else { continue };
        let term = c.get("term").and_then(|t| t.as_str()).unwrap_or("");
        let low = term.to_lowercase();
        let del = if stop.contains(&low) {
            if let Some(bs) = c.get("branches").and_then(|b| b.as_array()) {
                for b in bs {
                    if let Some(s) = b.as_str() {
                        let id = c.get("id").and_then(|i| i.as_str()).unwrap_or("");
                        branches.entry(s.to_string()).or_default().push(id.to_string());
                    }
                }
            }
            removed += 1;
            true
        } else if stale_isolated && c.get("count").and_then(|x| x.as_i64()).unwrap_or(0) < 1 {
            dropped += 1;
            true
        } else {
            false
        };
        if del {
            conn.execute(
                "DELETE FROM kv_store WHERE namespace=?1 AND key=?2",
                params![NS, key],
            )
            .ok();
        }
    }
    // 从引用分支的 concepts 数组摘除已删哈希
    for (bk, hashes) in &branches {
        let Some(raw) = kv_get(conn, NS, bk) else { continue };
        let Ok(mut v) = serde_json::from_str::<Value>(&raw) else { continue };
        let hset: HashSet<String> = hashes.iter().cloned().collect();
        if let Some(arr) = v.get_mut("concepts").and_then(|c| c.as_array_mut()) {
            arr.retain(|h| !hset.contains(h.as_str().unwrap_or("")));
        }
        kv_set(conn, NS, bk, &v.to_string());
    }
    if removed > 0 || dropped > 0 {
        let mut hub = ensure_hub(conn);
        refresh_hub_metrics(conn, &mut hub);
        save_hub(conn, &hub);
    }
    println!(
        "[prune] 清理 {} 噪声神经元 + {} 孤立神经元; {} 分支突触已摘除",
        removed,
        dropped,
        branches.len()
    );
}

/// 重建 Hebb 共现突触网络 (幂等 — 先清空 co 再全量重建)。
/// 全库 O(n²) 需内存批量: 全部概念载入 → 内存计共现 → 单事务写回。
fn cmd_hebb(conn: &mut Connection) {
    let mut branch_hashes: Vec<Vec<String>> = Vec::new();
    for (_, value) in scan_values(conn, "branch_") {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        let chs: Vec<String> = v
            .get("concepts")
            .and_then(|c| c.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if !chs.is_empty() {
            branch_hashes.push(chs);
        }
    }
    // 全量载入概念 → 内存 map {ch: concept}
    let mut concepts: HashMap<String, Value> = HashMap::new();
    for (_, value) in scan_values(conn, "concept_") {
        let Ok(mut c) = serde_json::from_str::<Value>(&value) else { continue };
        let id = c.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        if let Some(o) = c.as_object_mut() {
            o.remove("co");
            o.remove("co_w");
            o.insert("co".to_string(), json!([]));
            o.insert("co_w".to_string(), json!({}));
        }
        concepts.insert(id, c);
    }
    // 内存计共现 (完整对称, 保 O(1) 读取)
    let mut pairs: i64 = 0;
    for chs in &branch_hashes {
        let n = chs.len();
        pairs += (n * n.saturating_sub(1) / 2) as i64;
        for i in 0..n {
            for j in (i + 1)..n {
                co_bump_in_mem(&mut concepts, &chs[i], &chs[j]);
                co_bump_in_mem(&mut concepts, &chs[j], &chs[i]);
            }
        }
    }
    // 单事务批量写回 (空 co 数组不落盘, 少存冗余字段)
    let now = now_ts();
    let tx = conn.transaction().expect("hebb tx");
    for (ch, c) in &concepts {
        let mut c = c.clone();
        if let Some(o) = c.as_object_mut() {
            if o.get("co")
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true)
            {
                o.remove("co");
            }
            if o.get("co_w")
                .and_then(|v| v.as_object())
                .map(|a| a.is_empty())
                .unwrap_or(true)
            {
                o.remove("co_w");
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![NS, format!("concept_{}", ch), value_encode(&c.to_string()), now],
        )
        .expect("hebb write");
    }
    tx.commit().expect("hebb commit");
    println!(
        "[hebb] {} 分支共现网络重建, {} 神经元, 累计 {} 概念对",
        branch_hashes.len(),
        concepts.len(),
        pairs
    );
}

/// 存量 value 透明压缩迁移 (幂等 — 已压缩魔数行跳过)。
fn cmd_compress(conn: &mut Connection, all: bool) {
    let nss: Vec<String> = if all {
        let mut stmt = conn
            .prepare("SELECT DISTINCT namespace FROM kv_store")
            .expect("compress nss prepare");
        stmt.query_map([], |r| r.get::<_, String>(0))
            .expect("compress nss map")
            .filter_map(|r| r.ok())
            .collect()
    } else {
        vec![NS.to_string()]
    };
    let mut total_rows: u64 = 0;
    let mut total_in: u64 = 0;
    let mut total_out: u64 = 0;
    let now = now_ts();
    for ns in &nss {
        let mut stmt = conn
            .prepare("SELECT key, value FROM kv_store WHERE namespace=?1")
            .expect("compress prepare");
        let rows = stmt
            .query_map(params![ns], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<SqlValue>>(1)?))
            })
            .expect("compress query_map");
        let mut batch: Vec<(String, Vec<u8>)> = Vec::new();
        for row in rows {
            let Ok((key, value)) = row else { continue };
            let Some(value) = value else { continue };
            let raw: Vec<u8> = match &value {
                SqlValue::Blob(b) => b.clone(),
                SqlValue::Text(s) => s.as_bytes().to_vec(),
                _ => continue,
            };
            if raw.len() >= 4 && &raw[..4] == VALUE_MAGIC {
                total_rows += 1;
                continue;
            }
            let Ok(text) = String::from_utf8(raw) else { continue };
            let encoded = value_encode(&text);
            total_in += text.len() as u64;
            total_out += encoded.len() as u64;
            total_rows += 1;
            batch.push((key, encoded));
        }
        if !batch.is_empty() {
            drop(stmt);
            let tx = conn.transaction().expect("compress tx");
            for (key, encoded) in &batch {
                tx.execute(
                    "UPDATE kv_store SET value=?1, updated_at=?2 WHERE namespace=?3 AND key=?4",
                    params![encoded, now, ns, key],
                )
                .expect("compress update");
            }
            tx.commit().expect("compress commit");
            println!("[compress] {}: {} 行迁移压缩", ns, batch.len());
        }
    }
    if total_in > 0 {
        let pct = (1.0 - total_out as f64 / total_in as f64) * 100.0;
        println!(
            "[compress] 总 {} 行: {} B -> {} B (省 {:.1}%)",
            total_rows, total_in, total_out, pct
        );
    } else {
        println!("[compress] 总 {} 行: 无待压缩明文", total_rows);
    }
}

// ────────────────────────────────────────────────────────────────
// 生成 Experience Index (派生生成物)
// ────────────────────────────────────────────────────────────────
fn cycle_sort_key(c: &str) -> (i64, String) {
    let mut num = String::new();
    let mut suf = c.to_string();
    for (i, ch) in c.chars().enumerate() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else {
            suf = c[i..].to_string();
            break;
        }
    }
    (num.parse().unwrap_or(0), suf)
}

fn cycle_summary(conn: &Connection, cycle: &str) -> String {
    let mut stmt = conn
        .prepare("SELECT key, value FROM kv_store WHERE namespace=?1 AND key LIKE ?2")
        .expect("cycle_summary prepare");
    let rows = stmt
        .query_map(params![NS, format!("branch_{}_%", cycle)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<SqlValue>>(1)?))
        })
        .expect("cycle_summary query_map");
    let mut all: Vec<Value> = Vec::new();
    for row in rows {
        let Ok((_, value)) = row else { continue };
        let Some(value) = value else { continue };
        let Some(decoded) = sql_value_decode(&value) else { continue };
        if let Ok(v) = serde_json::from_str::<Value>(&decoded) {
            all.push(v);
        }
    }
    let mut best = String::new();
    // 优先取 "## Experience Tree — Cycle ..." 标题行
    for v in &all {
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("## ") && line.contains("Cycle") {
                best = line.trim_start_matches('#').trim().to_string();
                break;
            }
        }
        if !best.is_empty() {
            break;
        }
    }
    // 回退: 第一个非空简短 content
    if best.is_empty() {
        for v in &all {
            let mut content = v
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if content.chars().count() > 80 {
                content = format!("{}…", truncate(&content, 80));
            }
            if !content.is_empty() {
                best = content;
                break;
            }
        }
    }
    if best.chars().count() > 100 {
        best = format!("{}…", truncate(&best, 100));
    }
    if best.is_empty() {
        best = "—".to_string();
    }
    best
}

fn cmd_gen_index(conn: &Connection, out: &str, limit: usize) {
    let hub = ensure_hub(conn);
    let cycles = hub["hub"]
        .get("cycles")
        .and_then(|c| c.as_object())
        .cloned()
        .unwrap_or_default();
    let mut ordered: Vec<String> = cycles.keys().cloned().collect();
    ordered.sort_by(|a, b| cycle_sort_key(b).cmp(&cycle_sort_key(a)));
    ordered.truncate(limit);
    let header = format!(
        "# Experience Index (自动生成 — 勿手工编辑)\n\n\
         > 本文件由 `neotrix-experience gen-index` 从 KB hub 自动生成 (派生生成物)。\n\
         > 手工追加会在下一次生成时被覆盖。经验全文在 KB, 此处仅保留最近 {} cycle 指针。\n",
        ordered.len()
    );
    let mut lines = vec![header, "| Cycle | Session |".to_string(), "|-------|----------|".to_string()];
    for c in &ordered {
        let summary = cycle_summary(conn, c);
        lines.push(format!("| {} | {} |", c, summary));
    }
    let body = lines.join("\n");
    if out == "-" {
        println!("{}", body);
    } else {
        std::fs::write(out, format!("{}\n", body)).expect("write gen-index");
        println!("[gen-index] {} cycle pointers written to {}", ordered.len(), out);
    }
}

// ────────────────────────────────────────────────────────────────
// CLI
// ────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "neotrix-experience", about = "Unified end-of-conversation absorption engine")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 会话开始快照
    Snapshot {
        #[arg(long, required = true)]
        cycle: String,
        #[arg(long, default_value = "")]
        task: String,
        #[arg(long, default_value = "unknown")]
        domain: String,
    },
    /// 关闭会话快照
    Close {
        #[arg(long, required = true)]
        cycle: String,
    },
    /// 会话结束吸收
    Absorb {
        session: String,
    },
    /// 检索匹配分支
    Query {
        #[arg(long, default_value = "")]
        kw: String,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_hebb: bool,
    },
    /// 列出条目
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        domain: Option<String>,
    },
    /// 列出过期 (verify_by) 分支 — 复核清单
    Stale {
        #[arg(long)]
        domain: Option<String>,
    },
    /// 查看 hub
    Hub,
    /// 更新 route_table
    Route {
        #[arg(long, required = true)]
        kw: String,
        #[arg(long, required = true)]
        branch: String,
    },
    /// 神经概念图检视 (突触链路)
    Neuron {
        term: String,
        #[arg(long)]
        exact: bool,
    },
    /// 为旧分支重建概念神经元与突触链路
    Backfill,
    /// 清理停用词污染与孤立概念神经元 (幂等自愈)
    Prune {
        #[arg(long, num_args = 0..)]
        stop: Vec<String>,
        #[arg(long)]
        stale_isolated: bool,
    },
    /// 重建 Hebb 共现突触网络
    Hebb,
    /// 存量 value 透明压缩迁移 (zlib, 魔数标记)
    Compress {
        #[arg(long)]
        all: bool,
    },
    /// 从 KB 生成 Experience Index 派生文件
    GenIndex {
        #[arg(long, default_value = "-")]
        out: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut conn = open_kb();
    match cli.cmd {
        Cmd::Snapshot { cycle, task, domain } => cmd_snapshot(&conn, &cycle, &task, &domain),
        Cmd::Close { cycle } => cmd_close(&conn, &cycle),
        Cmd::Absorb { session } => cmd_absorb(&conn, &session),
        Cmd::Query { kw, r#type, domain, limit, no_hebb } => {
            cmd_query(&conn, &kw, r#type.as_deref(), domain.as_deref(), limit, no_hebb)
        }
        Cmd::List { r#type, domain } => cmd_list(&conn, r#type.as_deref(), domain.as_deref()),
        Cmd::Stale { domain } => cmd_stale(&conn, domain.as_deref()),
        Cmd::Hub => cmd_hub(&conn),
        Cmd::Route { kw, branch } => cmd_route(&conn, &kw, &branch),
        Cmd::Neuron { term, exact } => cmd_neuron(&conn, &term, exact),
        Cmd::Backfill => cmd_backfill(&conn),
        Cmd::Prune { stop, stale_isolated } => cmd_prune(&conn, &stop, stale_isolated),
        Cmd::Hebb => cmd_hebb(&mut conn),
        Cmd::Compress { all } => cmd_compress(&mut conn, all),
        Cmd::GenIndex { out, limit } => cmd_gen_index(&conn, &out, limit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_roundtrip() {
        let long = "x".repeat(2000);
        let encoded = value_encode(&long);
        assert!(encoded.len() >= 4 && &encoded[..4] == VALUE_MAGIC);
        assert_eq!(value_decode(&encoded).unwrap(), long);
        let short = "abc";
        assert_eq!(value_encode(short), short.as_bytes());
    }

    #[test]
    fn test_concept_hash_stable() {
        assert_eq!(concept_hash("neotrix"), concept_hash("neotrix"));
        assert_eq!(concept_hash("hebb").len(), 16);
        assert_ne!(concept_hash("a"), concept_hash("b"));
    }

    #[test]
    fn test_extract_concepts_en() {
        let s = "DelegateEngine wired to E8 with area and note and this and check";
        let c = extract_concepts(s);
        assert!(c.contains("DelegateEngine"));
        assert!(!c.contains("area"));
        assert!(!c.contains("this"));
        assert!(!c.contains("check"));
    }

    #[test]
    fn test_extract_concepts_cn() {
        let c = extract_concepts("神经网络化 是 关键一步");
        assert!(c.contains("神经网络化"));
    }

    #[test]
    fn test_extract_concepts_long_cn_window() {
        let c = extract_concepts("这是一个非常长的中文概念短语用于测试滑窗切词行为");
        // 至少应切出一些 3-4 字窗口且不含纯停用字
        assert!(c.iter().any(|t| t.chars().count() == 3 || t.chars().count() == 4));
    }

    #[test]
    fn test_cycle_sort_key() {
        assert_eq!(cycle_sort_key("160d"), (160, "d".to_string()));
        assert_eq!(cycle_sort_key("201"), (201, "201".to_string()));
        assert!(cycle_sort_key("201b") < cycle_sort_key("201c"));
    }

    #[test]
    fn test_is_stale_and_norm() {
        let ts = now_ts();
        let v = json!({"verify_by": ts - 100});
        assert!(is_stale(v.get("verify_by"), ts));
        let v2 = json!({"verify_by": ts + 100});
        assert!(!is_stale(v2.get("verify_by"), ts));
        let v3 = json!({"verify_by": "not-a-date"});
        assert!(!is_stale(v3.get("verify_by"), ts));
        let e = json!({"verify_by": Value::Null});
        assert_eq!(
            norm_verify_by(&e, ts),
            Some(ts + VERIFY_DEFAULT_DAYS * DAY)
        );
    }

    #[test]
    fn test_scan_values_handles_text_and_blob() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE kv_store (namespace TEXT, key TEXT, value BLOB, updated_at INTEGER)",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO kv_store VALUES ('experience','concept_a','plain text',1)",
            [],
        )
        .unwrap();
        let encoded = value_encode("compressed content here");
        conn.execute(
            "INSERT INTO kv_store (namespace,key,value,updated_at) VALUES ('experience','concept_b',?1,1)",
            params![encoded],
        )
        .unwrap();
        let rows = scan_values(&conn, "concept_");
        assert_eq!(rows.len(), 2, "scan must read both TEXT and BLOB rows");
        assert!(rows.iter().any(|(k, v)| k == "concept_a" && v == "plain text"));
        assert!(
            rows.iter()
                .any(|(k, v)| k == "concept_b" && v == "compressed content here")
        );
        assert_eq!(kv_get(&conn, NS, "concept_a").as_deref(), Some("plain text"));
        assert_eq!(
            kv_get(&conn, NS, "concept_b").as_deref(),
            Some("compressed content here")
        );
    }
}
