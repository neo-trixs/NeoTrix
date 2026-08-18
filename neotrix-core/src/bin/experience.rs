//! neotrix-experience — Unified End-of-Conversation Absorption Engine (Rust native,
//! 生产路径; 历史原型为 Python 版 `absorb_session.py`, 已退役).
#![allow(clippy::unwrap_used)] // CLI bin 自持 JSON/DB 结构, 结构非法即 panic 报错优于静默降级
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
//!   cargo run -p neotrix --bin neotrix-experience query --kw "关键词" [--type T] [--domain D] [--limit N] [--no-hebb] [--include-distilled]
//!   cargo run -p neotrix --bin neotrix-experience list [--type T] [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience stale [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience hub
//!   cargo run -p neotrix --bin neotrix-experience route --kw KEYWORD --branch BK
//!   cargo run -p neotrix --bin neotrix-experience neuron TERM [--exact]
//!   cargo run -p neotrix --bin neotrix-experience backfill
//!   cargo run -p neotrix --bin neotrix-experience prune [--stop WORD ...] [--stale-isolated]
//!   cargo run -p neotrix --bin neotrix-experience hebb
//!   cargo run -p neotrix --bin neotrix-experience distill [--domain D] [--min-group N] [--dry-run]
//!   cargo run -p neotrix --bin neotrix-experience compress [--all]
//!   cargo run -p neotrix --bin neotrix-experience gen-index [--out FILE] [--limit N]

#![forbid(unsafe_code)]
use chrono::TimeZone;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;
use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
use neotrix::neotrix::l8_autonomic_impl::nt_mind_guard::{MapeGate, MapeGateConfig, MetricEval};
use neotrix::core::nt_core_hcube::ghrr_vsa::{
    ghrr_bundle, ghrr_random_vector_dim, ghrr_similarity,
};
use neotrix::core::nt_core_hcube::{PersistentHomology, PointCloud};
use rusqlite::types::Value as SqlValue;
use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use sha1::{Digest, Sha1};
use sha2::Sha256;
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

/// UNBP URL 规范化: 去锚点/尾斜杠/域名小写 (用于 absorb-node 去重)
/// 锚点 (#zh-full/#RealEarth4D) 是视角标记, 非 URL 唯一性的一部分
fn normalize_url(url: &str) -> String {
    let mut u = url.trim().to_string();
    if let Some(idx) = u.find('#') {
        u.truncate(idx);
    }
    u = u.trim_end_matches('/').to_string();
    // 域名小写 (仅 http/https)
    if let Some(pos) = u.find("://") {
        let rest = &u[pos + 3..];
        if let Some(slash) = rest.find('/') {
            let (host, path) = rest.split_at(slash);
            u = format!("{}://{}{}", &u[..pos], host.to_lowercase(), path);
        } else {
            let host = rest;
            u = format!("{}://{}", &u[..pos], host.to_lowercase());
        }
    }
    u
}

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

fn kb_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.neotrix", home)
}

fn open_kb() -> Connection {
    let db_path = format!("{}/knowledge.db", kb_dir());
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

/// 数据库忙时重试写操作: 并发进程 (如 absorb_guji) 持写锁时,
/// busy_timeout 可能失效或超时不足 (实测 DatabaseBusy panic at experience.rs:178),
/// 显式指数退避重试 (max 5 次, 总等待 ≤~3s), 仍失败则 panic 保留可见错误。
fn kv_set_retry(conn: &Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, rusqlite::Error> {
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

fn kv_set(conn: &Connection, namespace: &str, key: &str, value: &str) {
    let encoded = value_encode(value);
    kv_set_retry(
        conn,
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        &[&namespace, &key, &encoded, &now_ts()],
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
/// 从字符串派生确定性 u64 种子 (供 ghrr 确定性向量)。
fn seed_from_str(s: &str) -> u64 {
    let mut h = Sha1::new();
    h.update(s.as_bytes());
    let d = h.finalize();
    u64::from_be_bytes(d[..8].try_into().unwrap())
}

/// CJK 2-gram / ASCII 空白分词的 token 列表 (VSA 词袋)。
fn vsa_tokens(s: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii() {
            let mut j = i;
            while j < chars.len() && !chars[j].is_ascii_whitespace() {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            if !word.is_empty() {
                toks.push(word.to_lowercase());
            }
            i = j;
        } else if (0x4e00..=0x9fff).contains(&(c as u32)) {
            let mut bigram = String::new();
            bigram.push(c);
            if i + 1 < chars.len() {
                bigram.push(chars[i + 1]);
            }
            toks.push(bigram);
            i += 1;
        } else {
            i += 1;
        }
    }
    toks
}

/// 文本 → GHRR 确定性向量 (词袋 bundle, 同 token 重叠 → 语义相近)。
/// 使用全局 token 向量 memo: 同一 token (bigram/词) 只生成一次向量, 跨文档复用,
/// 避免 per-token StdRng 高维生成爆炸。返回 (向量, 本文本 token 数)。
fn text_doc_vector(s: &str, dim: usize, memo: &mut HashMap<String, Vec<f64>>) -> (Vec<f64>, usize) {
    let toks = vsa_tokens(s);
    if toks.is_empty() {
        let v = ghrr_random_vector_dim(dim, 0);
        return (v, 0);
    }
    let mut vecs: Vec<Vec<f64>> = Vec::with_capacity(toks.len());
    for t in &toks {
        if let Some(v) = memo.get(t) {
            vecs.push(v.clone());
        } else {
            let v = ghrr_random_vector_dim(dim, seed_from_str(t));
            memo.insert(t.clone(), v.clone());
            vecs.push(v);
        }
    }
    let refs: Vec<&[f64]> = vecs.iter().map(|v| v.as_slice()).collect();
    (ghrr_bundle(&refs), toks.len())
}

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
        // 结束判定: snapshot 存 ended_at: null (字段存在但为 null) → 视为未关闭。
        let not_ended = match v.get("ended_at") {
            None => true,
            Some(x) => x.is_null(),
        };
        if v.get("cycle").and_then(|c| c.as_str()) == Some(cycle) && not_ended {
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

/// P0-2 G3 质量门置信度 (SEA/SimpleMem absorb): 综合 verified_by / evidence /
/// source / not 负例 → [0,1]。吸收时作为初始 confidence, 供质量门过滤低信号噪声。
fn confidence_of(entry: &Value) -> f64 {
    let mut c = 0.0f64;
    if entry
        .get("verified_by")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        c += 0.25;
    }
    if entry
        .get("evidence")
        .and_then(|ev| ev.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        c += 0.25;
    }
    if entry.get("not").is_some() && entry.get("not").map(|n| n.is_string() || n.is_array()).unwrap_or(false) {
        c += 0.10;
    }
    match entry.get("source").and_then(|s| s.as_str()) {
        Some("experiment") | Some("code") | Some("trace") => c += 0.30,
        Some("dialogue") => c += 0.15,
        _ => c += 0.10,
    }
    c.min(1.0)
}

fn cmd_absorb(conn: &mut Connection, session_path: &str) {
    let mut hub = ensure_hub(conn);
    let raw = std::fs::read_to_string(session_path).expect("read session.json");
    let session: Value = serde_json::from_str(&raw).expect("session.json is valid JSON");

    let sid = session
        .get("session_id")
        .and_then(|s| s.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("sess_{}_{}", now_ts(), uuid_hex(8)));
    // 幂等门禁: 同一 session_id 已落盘则拒绝重复吸收 (防分支重复, cycle 218 教训)。
    let dup = scan_values(conn, "branch_").into_iter().find(|(_, v)| {
        serde_json::from_str::<Value>(v)
            .map(|b| b.get("session_id").and_then(|s| s.as_str()) == Some(sid.as_str()))
            .unwrap_or(false)
    });
    if let Some((key, _)) = dup {
        println!(
            "[absorb] 拒绝重复吸收 session_id={} (已存在于 {}) — 如需重吸先删除旧分支",
            sid, key
        );
        return;
    }
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
    // D20 (aihot-skill/workflow_templates 参照): 吸收审计轨迹 — 记录每条 entry 的
    // 决策 (written / redundant / invalid) 与内容哈希, 供事后核对吸收质量。
    let mut audit_log: Vec<Value> = Vec::new();
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
            // P0-2: 负例字段 (NOT: 不该做什么)
            "not": raw_entry.get("not").cloned().unwrap_or(Value::Null),
            // P0-1: 独立审计者字段
            "verified_by": raw_entry.get("verified_by").cloned().unwrap_or(Value::Null),
            "verification_status": raw_entry.get("verification_status").cloned().unwrap_or(Value::Null),
            // P0-2 G3: 经验三元组 (SEA/SimpleMem absorb): (context, decision, feedback)
            "context": raw_entry.get("context").cloned().unwrap_or_else(|| {
                json!(format!(
                    "domain={} type={}",
                    raw_entry.get("domain").or_else(|| session.get("domain"))
                        .and_then(|d| d.as_str()).unwrap_or("unknown"),
                    raw_entry.get("type").and_then(|t| t.as_str()).unwrap_or("insight")
                ))
            }),
            "decision": raw_entry.get("decision").cloned()
                .unwrap_or_else(|| raw_entry.get("content").cloned().unwrap_or(json!(""))),
            "feedback": json!({"success": 0, "failure": 0, "reuse": 0}),
            // P0-2 G3 质量门 confidence: verified_by + evidence + source 综合置信度
            "confidence": confidence_of(raw_entry),
        });
        // D20: manifest — 内容 SHA-256, 落盘后可按哈希核对吸收内容未被篡改/漂移
        let raw_content = e.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let content_hash = {
            let mut hasher = Sha256::new();
            hasher.update(raw_content.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        e["content_hash"] = json!(content_hash.clone());
        if let Some(vb) = norm_verify_by(&e, ts) {
            e["verify_by"] = json!(vb);
        }
        let errors = validate_entry(&e);
        if !errors.is_empty() {
            println!("[absorb] ✗ entry #{}: {:?}", i, errors);
            audit_log.push(json!({
                "idx": i, "decision": "invalid", "reason": format!("{:?}", errors),
                "content_hash": content_hash, "ts": ts,
            }));
            continue;
        }
        let dom = e.get("domain").and_then(|d| d.as_str()).unwrap_or("unknown");
        // 写入前语义过滤 (SRMU 启示, 记忆大脑设计 §4.1a): 与同 domain 已有分支算 VSA
        // 词袋相似度, 高冗余 (sim≥0.65, 校准自 2026-08-06 sim 分布: 精确=1.0, 改写≈0.75,
        // 部分重叠≈0.02, 无关≈0) → 拒绝落盘, 防重复吸收冗余。
        // 仅同 domain 比较 (跨 domain 不同语义面, 不裁), 且只在有已存分支时生效。
        let new_content = e.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if !new_content.trim().is_empty() {
            let dim = 2048usize;
            let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
            let (qvec, _) = text_doc_vector(&new_content.to_lowercase(), dim, &mut memo);
            // 性能优化 (cycle 387 根因): 原实现对全量 branch_ 逐条算 VSA 向量,
            // 随 KB 增长退化为 O(n²) (1851 条时单次 absorb ≈168s)。
            // 改为仅同 domain 最近 SIM_COMPARE_MAX 条比对 (保留冗余过滤能力,
            // 近期重复捕获足够; 跨期重复由幂等 session_id 门禁 + 内容哈希兜底)。
            const SIM_COMPARE_MAX: usize = 200;
            let mut best_sim = 0.0f64;
            let mut candidates: Vec<(String, String)> = scan_values(conn, "branch_")
                .into_iter()
                .rev() // scan 按 key 升序 (cycle 前缀近似时间序), 取最近
                .filter(|(_, v)| {
                    serde_json::from_str::<Value>(v)
                        .map(|b| b.get("domain").and_then(|d| d.as_str()) == Some(dom))
                        .unwrap_or(false)
                })
                .take(SIM_COMPARE_MAX)
                .collect();
            candidates.reverse(); // 恢复时间正序
            for (_, value) in candidates {
                let Ok(b) = serde_json::from_str::<Value>(&value) else { continue };
                let bc = b
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if bc.is_empty() {
                    continue;
                }
                let (dvec, _) = text_doc_vector(&bc, dim, &mut memo);
                let sim = ghrr_similarity(&qvec, &dvec);
                if sim > best_sim {
                    best_sim = sim;
                }
            }
            if best_sim >= 0.65 {
                println!(
                    "[absorb] 跳过冗余 entry #{} (sim={:.3} ≥0.65, 同 domain={}) — 防重复吸收",
                    i, best_sim, dom
                );
                audit_log.push(json!({
                    "idx": i, "decision": "redundant", "reason": format!("sim={:.3}", best_sim),
                    "content_hash": content_hash, "ts": ts,
                }));
                continue;
            }
        }
        // P0-2 G3 质量过滤门 (SEA/SimpleMem absorb): 低置信度 + 无证据 + 未验证
        // → 纯噪声, 拒绝落盘 (审计决策 quality_gate)。
        let confidence = e.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
        let has_evidence = e.get("evidence").and_then(|ev| ev.as_str())
            .map(|s| !s.trim().is_empty()).unwrap_or(false);
        let has_verifier = e.get("verified_by").and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty()).unwrap_or(false);
        if confidence < 0.15 && !has_evidence && !has_verifier {
            println!(
                "[absorb] ✗ quality gate 拒绝 entry #{} (confidence={:.2}, 无证据/无验证) — 低信号噪声",
                i, confidence
            );
            audit_log.push(json!({
                "idx": i, "decision": "quality_gate", "reason": format!("confidence={:.2}", confidence),
                "content_hash": content_hash, "ts": ts,
            }));
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
        audit_log.push(json!({
            "idx": i, "decision": "written", "key": key,
            "content_hash": content_hash, "ts": ts,
        }));
    }

    // D20: audit jsonl 落盘 — 与 KB 同一目录, 供审计核对每次吸收的决策轨迹
    let kb_dir = kb_dir();
    let audit_path = format!(
        "{}/audit_{}_{}.jsonl",
        kb_dir,
        cycle.replace(['/', '\\', ' '], "_"),
        sid.replace(['/', '\\', ' ', ':'], "_")
    );
    if std::fs::create_dir_all(&kb_dir).is_ok() {
        let mut blob = String::new();
        for rec in &audit_log {
            blob.push_str(&serde_json::to_string(rec).unwrap_or_default());
            blob.push('\n');
        }
        let _ = std::fs::write(&audit_path, blob);
    }
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("[absorb] {} entries from {} (cycle={})", written, sid, cycle);
    // 自动消退蒸馏: 吸收后若未蒸馏分支累积超阈值, 自动触发 distill
    // (经验无限追加 → 维度膨胀 → 自动收敛为能力模式, "始终处于最优解状态")
    auto_distill_if_over_threshold(conn, &mut hub);
}

// ────────────────────────────────────────────────────────────────
// P0-2 G3+G8: 经验反馈环 — reuse 结果记录 → MapeGate 多指标 burn-in 门
// (SEA/SimpleMem absorb)。feedback success/failure 反向下调 confidence,
// 触发主动回滚 (status=failing, confidence 减半) 或晋升 (status=stable)。
// ────────────────────────────────────────────────────────────────
fn cmd_feedback(conn: &mut Connection, key: &str, outcome: &str) {
    let Some(value) = kv_get(conn, NS, key) else {
        println!("[feedback] ✗ 分支 {key} 不存在 (kv_store experience namespace)");
        return;
    };
    let Ok(mut v) = serde_json::from_str::<Value>(&value) else {
        println!("[feedback] ✗ 分支 {key} 损坏 (非 JSON)");
        return;
    };
    let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
    if content.trim().is_empty() {
        println!("[feedback] ✗ 分支 {key} 无 content, 拒绝反馈");
        return;
    }

    let obj = v.as_object_mut().expect("branch 是 JSON 对象");
    let fb = obj.entry("feedback").or_insert_with(|| json!({"success": 0, "failure": 0, "reuse": 0}));
    let mut success = fb.get("success").and_then(|x| x.as_i64()).unwrap_or(0);
    let mut failure = fb.get("failure").and_then(|x| x.as_i64()).unwrap_or(0);
    let mut reuse = fb.get("reuse").and_then(|x| x.as_i64()).unwrap_or(0);
    match outcome {
        "success" => {
            success += 1;
            reuse += 1;
        }
        "failure" => failure += 1,
        other => {
            println!("[feedback] ✗ outcome 必须为 success|failure, 收到: {other}");
            return;
        }
    }
    fb["success"] = json!(success);
    fb["failure"] = json!(failure);
    fb["reuse"] = json!(reuse);

    // 反向下调/上调 confidence (feedback 负例 → 该经验可信度下降)
    let total = (success + failure) as f64;
    let base_conf = v.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5);
    let conf = if total > 0.0 {
        let ratio = success as f64 / total;
        (base_conf * 0.5 + ratio * 0.5).clamp(0.0, 1.0)
    } else {
        base_conf
    };
    v["confidence"] = json!(conf);

    // G8 MapeGate 多指标验证门 (burn-in 20): 指标 = confidence / feedback / reuse
    let mut gate = MapeGate::new(MapeGateConfig::default());
    let metrics = vec![
        MetricEval { name: "confidence".into(), score: conf, passed: conf >= 0.5 },
        MetricEval {
            name: "feedback".into(),
            score: if total > 0.0 { success as f64 / total } else { 1.0 },
            passed: failure == 0 || (success as f64 / total) >= 0.5,
        },
        MetricEval { name: "reuse".into(), score: reuse as f64, passed: reuse >= 3 },
    ];
    let verdict = gate.evaluate(key, metrics);
    if verdict.rollback {
        v["status"] = json!("failing");
        v["rollback"] = json!(true);
        let c = v.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5) * 0.5;
        v["confidence"] = json!(c);
        println!(
            "[feedback] ↺ 回滚 {key} (evaluations={}, reason: {}) → status=failing, confidence={:.2}",
            verdict.evaluations, verdict.note, c
        );
    } else if verdict.promoted {
        v["status"] = json!("stable");
        v["rollback"] = json!(false);
        println!(
            "[feedback] ⬆ 晋升 {key} → stable (evaluations={}, {})",
            verdict.evaluations, verdict.note
        );
    } else {
        println!(
            "[feedback] ◌ burn-in (evaluations={}, {}) confidence={:.2}, success={}, failure={}, reuse={}",
            verdict.evaluations, verdict.note, conf, success, failure, reuse
        );
    }
    kv_set(conn, NS, key, &v.to_string());
    println!("[feedback] 已更新 {key} feedback={success}成功/{failure}失败 reuse={reuse}");
}

/// 吸收后自动蒸馏: 未蒸馏分支数 ≥ 阈值时触发 distill (min_group=3 默认)。
/// 幂等 — distill 自身跳过已蒸馏条目; 阈值防频繁触发 (每 cycle 吸收 6 条
/// 左右, 阈值 60 ≈ 10 cycle 一次收敛)。
const AUTO_DISTILL_THRESHOLD: usize = 60;
fn auto_distill_if_over_threshold(conn: &mut Connection, hub: &mut Value) {
    let mut undistilled = 0;
    for (_, value) in scan_values(conn, "branch_") {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        if !v.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false) {
            undistilled += 1;
        }
    }
    if undistilled < AUTO_DISTILL_THRESHOLD {
        return;
    }
    println!(
        "[absorb] 未蒸馏分支 {} 条 ≥ 阈值 {}, 自动触发维度蒸馏...",
        undistilled, AUTO_DISTILL_THRESHOLD
    );
    cmd_distill(conn, None, 3, false);
    refresh_hub_metrics(conn, hub);
    save_hub(conn, hub);
}

// ────────────────────────────────────────────────────────────────
// R-P97: 批量节点吸收 (Python insert_node 的 Rust port — 知识写入单一事实源)
// ────────────────────────────────────────────────────────────────
/// 批量节点吸收: 输入 JSON (节点数组或单对象) → URL 去重 → nodes/nodes_fts 双写。
/// 语义对齐 scripts/kb_batch_absorb.py:insert_node:
///   - 去重: SELECT 1 FROM nodes WHERE url=? (URL 为唯一键, 幂等)
///   - FTS: 显式 INSERT INTO nodes_fts (非 external-content 表, rebuild 不会拉新数据)
///   - capability: --apply-capability 写 metadata.absorbed_capability 四元组 (R-P79 闭环)
///
/// node id 派生: batch_{ts}_{sha1(url)[:8]} (sha1 仅作存储键派生, 非安全用途)。
fn cmd_absorb_node(conn: &Connection, input: &str, dry_run: bool, apply_capability: bool) {
    // 1. 读取输入 (文件或 stdin)
    let raw = if input == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .expect("read stdin");
        buf
    } else {
        std::fs::read_to_string(input).unwrap_or_else(|e| {
            panic!("read node json {}: {}", input, e)
        })
    };
    let v: Value = serde_json::from_str(&raw).expect("node json is valid JSON");

    // 2. 归一化为节点数组 (单对象 → [对象])
    let nodes: Vec<Value> = match v {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![v],
        _ => panic!("input must be a JSON object or array of objects"),
    };

    // 3. 逐个处理
    let mut inserted = 0usize;
    let mut duplicated = 0usize;
    let mut mapped = 0usize;
    let now = now_ts();
    // 管道写端连接复用于整批, 避免每节点重复 open KB (开销大)
    let kb = KnowledgeBase::open(None).expect("open KB");
    for (i, n) in nodes.iter().enumerate() {
        let url = n
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .trim();
        if url.is_empty() {
            println!("[absorb-node] ✗ node #{}: missing url — skipped", i);
            continue;
        }
        let title = n
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or(url)
            .to_string();
        let summary = n
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let content = n
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let node_type = n
            .get("node_type")
            .and_then(|t| t.as_str())
            .unwrap_or("article")
            .to_string();
        let language = n
            .get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("en")
            .to_string();
        let domain = n
            .get("domain")
            .and_then(|d| d.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                url.split("//")
                    .nth(1)
                    .and_then(|rest| rest.split('/').next())
                    .unwrap_or("")
                    .trim_start_matches("www.")
                    .to_string()
            });
        let importance = n
            .get("importance")
            .and_then(|im| im.as_f64())
            .unwrap_or(0.5);

        // 4. URL 规范化 (锚点/尾斜杠/域名小写) — 幂等与去重统一交由管道 absorb_core
        //    (管道按 entry.url 精确匹配, 规范化后传入保证重复 URL 幂等 + 补 hub 边)
        let norm_url = normalize_url(url);
        if dry_run {
            println!("[absorb-node] would_insert #{}: {}", i, url);
            inserted += 1;
            continue;
        }

        // 5. (node id 由管道 absorb_core 内部派生, 见 nt_memory_pipeline)

        // 6. metadata: 保留输入 meta 字段 + enriched_at
        let mut meta = match n.get("meta") {
            Some(Value::Object(m)) => Value::Object(m.clone()),
            _ => json!({}),
        };
        if meta.get("enriched_at").is_none() {
            meta["enriched_at"] = json!(now);
        }

        // 7. 走最短路径管道写入 (absorb_core: nodes + FTS + 域枢纽 BelongsTo 边)
        //    原裸 SQL 双写 (PA011 desync 防护) 已内化为 nt_memory_pipeline::absorb_core,
        //    意识体/CLI 共用同一写端, 防逻辑分叉 (R-P42 强化现有节点)。
        let entry = AbsorbEntry {
            title: title.clone(),
            summary: if summary.is_empty() { None } else { Some(summary) },
            content: if content.is_empty() { None } else { Some(content) },
            node_type: node_type.clone(),
            domain: Some(domain),
            url: Some(norm_url.to_string()),
            language: Some(language.clone()),
            importance: Some(importance),
            relations: vec![],
        };
        let report = kb.absorb_core(&entry).expect("absorb_core pipeline");
        if report.created {
            inserted += 1;
        } else {
            duplicated += 1;
        }
        println!(
            "[absorb-node] {} #{}: {} ({}, lang={}, cap={}, hub={}, edges={})",
            if report.created { "inserted" } else { "duplicate" },
            i,
            url,
            node_type,
            language,
            if apply_capability { "apply" } else { "-" },
            report.hub_linked,
            report.edges_added,
        );
        let eid = report.node_id;

        // 8. capability 映射 (R-P79 闭环: metadata.absorbed_capability 四元组)
        if apply_capability {
            if let (Some(branch), Some(capability)) = (
                n.get("capability")
                    .and_then(|c| c.get("branch"))
                    .and_then(|b| b.as_str()),
                n.get("capability")
                    .and_then(|c| c.get("capability"))
                    .and_then(|c| c.as_str()),
            ) {
                let evidence = n
                    .get("capability")
                    .and_then(|c| c.get("evidence"))
                    .and_then(|e| e.as_str())
                    .unwrap_or("");
                let mapped_at = {
                    // 本地时间 YYYY-MM-DDTHH:MM:SS
                    let secs = now;
                    let dt = chrono::DateTime::from_timestamp(secs, 0)
                        .unwrap_or_else(|| {
                            chrono::DateTime::from_timestamp(0, 0).expect("epoch timestamp 必合法")
                        });
                    let local = dt.with_timezone(&chrono::Local);
                    local.format("%Y-%m-%dT%H:%M:%S").to_string()
                };
                meta["absorbed_capability"] = json!({
                    "branch": branch,
                    "capability": capability,
                    "evidence": evidence,
                    "mapped_at": mapped_at,
                });
                conn.execute(
                    "UPDATE nodes SET metadata=?1 WHERE id=?2",
                    params![meta.to_string(), eid],
                )
                .expect("update node metadata");
                mapped += 1;
            }
        }
    }

    println!(
        "[absorb-node] done: {} inserted, {} duplicated, {} mapped (dry_run={})",
        inserted, duplicated, mapped, dry_run
    );
}

// ────────────────────────────────────────────────────────────────
// R-P97: 节点 metadata 批量更新 (absorb_to_capability.py 写回路径的 Rust port)
// ────────────────────────────────────────────────────────────────
/// 批量更新已有节点的 metadata: 输入 JSON 数组 [{node_id, patch: {key: value}}],
/// 读原 metadata JSON → 合并 patch → 写回。patch 值可为任意 JSON
/// (如 absorbed_capability 四元组 / knowledge_source 本源溯源对象)。
/// 语义对齐 absorb_to_capability.py:678 UPDATE nodes SET metadata=? — 单一事实源。
fn cmd_update_node_metadata(conn: &Connection, input: &str, dry_run: bool) {
    let raw = if input == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .expect("read stdin");
        buf
    } else {
        std::fs::read_to_string(input).unwrap_or_else(|e| panic!("read {}: {}", input, e))
    };
    let v: Value = serde_json::from_str(&raw).expect("update list is valid JSON");
    let updates: Vec<Value> = match v {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![v],
        _ => panic!("input must be a JSON array of {{node_id, patch}} objects"),
    };

    let mut updated = 0usize;
    let mut missing = 0usize;
    for (i, u) in updates.iter().enumerate() {
        let nid = u.get("node_id").and_then(|n| n.as_str()).unwrap_or("");
        let patch = match u.get("patch") {
            Some(Value::Object(p)) => p.clone(),
            _ => {
                println!("[update-node-metadata] ✗ #{}: missing patch — skipped", i);
                continue;
            }
        };
        if nid.is_empty() {
            println!("[update-node-metadata] ✗ #{}: missing node_id — skipped", i);
            continue;
        }
        // 读原 metadata
        let meta_raw: Option<String> = conn
            .query_row(
                "SELECT metadata FROM nodes WHERE id=?1",
                params![nid],
                |r| r.get(0),
            )
            .ok();
        let Some(meta_raw) = meta_raw else {
            missing += 1;
            println!("[update-node-metadata] ✗ #{}: node not found ({})", i, nid);
            continue;
        };
        let mut meta: Map<String, Value> = if meta_raw.trim().is_empty() {
            Map::new()
        } else {
            serde_json::from_str(&meta_raw).unwrap_or_else(|_| Map::new())
        };
        // 合并 patch
        for (k, val) in &patch {
            meta.insert(k.clone(), val.clone());
        }
        if dry_run {
            println!(
                "[update-node-metadata] would_update #{}: {} (keys: {:?})",
                i,
                nid,
                patch.keys().collect::<Vec<_>>()
            );
            updated += 1;
            continue;
        }
        conn.execute(
            "UPDATE nodes SET metadata=?1, updated_at=?2 WHERE id=?3",
            params![serde_json::to_string(&Value::Object(meta)).unwrap_or_default(), now_ts(), nid],
        )
        .expect("update node metadata");
        updated += 1;
        println!(
            "[update-node-metadata] updated #{}: {} (keys: {:?})",
            i,
            nid,
            patch.keys().collect::<Vec<_>>()
        );
    }
    println!(
        "[update-node-metadata] done: {} updated, {} missing (dry_run={})",
        updated, missing, dry_run
    );
}

// ────────────────────────────────────────────────────────────────
// 5. Feedback 反馈 / 查询
// ────────────────────────────────────────────────────────────────
/// 突触联想检索: 输入词 → 命中概念神经元 → 沿突触扩散到分支(1阶, 主结果) →
/// Hebb 共现多跳扩散到关联概念 (BFS, 每跳衰减, 记忆大脑设计 §3.3 图信号) →
/// 关联概念分支获得加权分数 (作为相关推荐)。
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
    // 二阶+: Hebb 共现多跳扩散 (BFS, 每跳衰减 decay 系数) — 与命中概念关联的概念
    // (经一跳及以上), 其分支获得衰减权重。hops 受限避免扩散爆炸: 每跳只保留
    // top-K 高激活概念 (frontier 剪枝), 与设计文档 §3.3 图信号一致。
    const HOP_LIMIT: usize = 3;
    const DECAY: f64 = 0.5;
    const FRONTIER_K: usize = 12;
    let mut order2: HashMap<String, f64> = HashMap::new();
    let mut frontier: Vec<(String, f64)> = first_neurons
        .iter()
        .filter_map(|c| {
            c.get("id")
                .and_then(|i| i.as_str())
                .map(|id| (id.to_string(), 1.0))
        })
        .collect();
    let mut seen: HashSet<String> = neuron_hits.clone();
    for _ in 0..HOP_LIMIT {
        if frontier.is_empty() {
            break;
        }
        let mut next: HashMap<String, f64> = HashMap::new();
        for (ch, act) in &frontier {
            let Some(c) = load_concept(conn, ch) else { continue };
            let co = co_full(&c);
            if co.is_empty() {
                continue;
            }
            let co_max = co.values().cloned().fold(1.0f64, f64::max);
            for (oth_ch, w) in co {
                if seen.contains(&oth_ch) {
                    continue;
                }
                let boost = act * DECAY * (w / co_max);
                *next.entry(oth_ch.clone()).or_insert(0.0) += boost;
            }
        }
        // Frontier 剪枝: 保留 top-K 高激活, 收集其分支; 同时标记 seen 防回环
        let mut ranked: Vec<(String, f64)> = next.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        let mut keep: Vec<(String, f64)> = Vec::new();
        for (ch, act) in ranked.into_iter().take(FRONTIER_K) {
            if let Some(oth) = load_concept(conn, &ch) {
                if let Some(bs) = oth.get("branches").and_then(|b| b.as_array()) {
                    for b in bs {
                        if let Some(s) = b.as_str() {
                            *order2.entry(s.to_string()).or_insert(0.0) += act;
                        }
                    }
                }
            }
            seen.insert(ch.clone());
            keep.push((ch, act));
        }
        frontier = keep;
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
    semantic: f64,
}

#[allow(clippy::too_many_arguments)] // CLI 子命令参数面, 直白优于 struct
fn cmd_query(conn: &Connection, kw: &str, ty: Option<&str>, domain: Option<&str>, limit: usize, no_hebb: bool, json: bool, semantic: bool, include_distilled: bool) -> usize {
    ensure_hub(conn);
    // 蒸馏降权: 默认过滤已蒸馏原始条目 (模式已升维), --include-distilled 保留溯源
    let allow_distilled = |v: &Value| include_distilled
        || !v.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false);
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
                semantic: 0.0,
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
            if !allow_distilled(v) {
                continue;
            }
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
                semantic: 0.0,
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
    // 语义信号 (第三路混合): 与 query 的 VSA 词袋相似度, 重加权排序。
    // 权重: 语义 0.4 / 原 FTS 图扩散 0.6 (研究 §6.3.1; 初始硬编码, C3 校准)。
    // 分层 (TiMem/HiGMem 锚点思想): FTS 命中时只在 top-K 候选上 refine;
    // FTS 0 命中时回退到全库扫描 (预算截断, 语义作 recall 补充, 非重排)。
    // token 向量 memo 跨文档复用, 避免 per-token RNG 生成爆炸。
    if semantic && !kw.trim().is_empty() {
        let dim = 2048usize;
        let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
        let (qvec, _) = text_doc_vector(&kw.to_lowercase(), dim, &mut memo);
        if !results.is_empty() {
            // 阶段A: FTS 命中 → 只 refine top-K
            let k = (limit * 4).clamp(8, 64);
            let mut ranked: Vec<QueryResult> = results.clone();
            ranked.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| a.key.cmp(&b.key))
            });
            let candidates: Vec<QueryResult> = ranked.into_iter().take(k).collect();
            let mut scored: Vec<(f64, f64, String)> = Vec::with_capacity(candidates.len());
            for r in &candidates {
                let full = cache
                    .get(&r.key)
                    .and_then(|v| v.as_ref())
                    .and_then(|v| v.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or(&r.content)
                    .to_lowercase();
                let (dvec, _) = text_doc_vector(&full, dim, &mut memo);
                let sim = ghrr_similarity(&qvec, &dvec);
                let fused = 0.4 * sim + 0.6 * r.score;
                scored.push((fused, sim, r.key.clone()));
            }
            let order_map: HashMap<String, (f64, f64)> = scored
                .into_iter()
                .map(|(f, s, k)| (k, (f, s)))
                .collect();
            for r in &mut results {
                if let Some(&(fused, sim)) = order_map.get(&r.key) {
                    r.score = fused;
                    r.order = 2; // 语义候选最高优先
                    r.semantic = sim;
                } else {
                    r.order = 0; // 未进 top-K 预算的候选排最后
                }
            }
            // 语义排序: order 降序 (语义候选 2 优先), 融合分高者先; 未进预算者殿后。
            results.sort_by(|a, b| {
                b.order
                    .cmp(&a.order)
                    .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal))
                    .then_with(|| a.key.cmp(&b.key))
            });
        } else {
            // 阶段B: FTS 0 命中 → 全库语义召回, 预算截断 (取 k 条最高 sim), 输出 fused=sim。
            let k = (limit * 4).clamp(8, 64);
            let mut scored: Vec<(f64, f64, String)> = Vec::new();
            for (key, v) in &cache {
                let Some(v) = v else { continue };
                let full = v
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if full.is_empty() {
                    continue;
                }
                let (dvec, _) = text_doc_vector(&full, dim, &mut memo);
                let sim = ghrr_similarity(&qvec, &dvec);
                if sim > 0.0 {
                    scored.push((sim, sim, key.clone()));
                }
            }
            scored.sort_by(|a, b| {
                b.0.partial_cmp(&a.0)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| a.2.cmp(&b.2))
            });
            results = scored
                .into_iter()
                .take(k)
                .filter_map(|(fused, sim, key)| {
                    let v = cache.get(&key).and_then(|x| x.as_ref())?;
                    Some(QueryResult {
                        cycle: cycle_opt(v.get("cycle")),
                        ty: v.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        domain: v.get("domain").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        content: truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 100),
                        evidence: v.get("evidence").and_then(|e| e.as_str()).unwrap_or("").to_string(),
                        key,
                        verify_by: v.get("verify_by").cloned(),
                        score: fused,
                        order: 2,
                        semantic: sim,
                    })
                })
                .collect();
        }
    }
    let now = now_ts();
    let shown = results.len().min(limit);
    if json {
        // 机器可读输出: 顶层数组, 每元素 {key, cycle, type, domain, content, evidence}
        // key 直接来自 kv_store branch_% —— 天然真实存在, 取代 Python 正则提取/二次校验。
        let arr: Vec<Value> = results[..shown]
            .iter()
            .map(|r| {
                json!({
                    "key": r.key,
                    "cycle": r.cycle.as_deref().unwrap_or(""),
                    "type": r.ty,
                    "domain": r.domain,
                    "content": r.content,
                    "evidence": r.evidence
                })
            })
            .collect();
        println!("{}", json!(arr));
        return results.len();
    }
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
    results.len()
}

fn cmd_list(conn: &Connection, ty: Option<&str>, domain: Option<&str>, cycle: Option<&str>) {
    ensure_hub(conn);
    let rows = scan_values(conn, "branch_");
    let mut count = 0;
    for (key, value) in rows {
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
        if let Some(c) = cycle {
            if v.get("cycle").and_then(|x| x.as_str()) != Some(c) {
                continue;
            }
        }
        count += 1;
        println!(
            "[{}] {:8} {:16} {}  (key={})",
            cycle_opt(v.get("cycle")).unwrap_or_else(|| "?".to_string()),
            v.get("type").and_then(|x| x.as_str()).unwrap_or("?"),
            v.get("domain").and_then(|x| x.as_str()).unwrap_or("?"),
            truncate(v.get("content").and_then(|c| c.as_str()).unwrap_or(""), 80),
            truncate(&key, 40)
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
    // 门禁: 分支必须真实存在于 kv_store (branch_% key), 否则拒绝 route 防 ghost ROUTE
    // (root cause fix: ghost branches 来自 route 命令接受任意字符串且不回显校验)
    let branch_key = branch.trim_end_matches('/');
    if !branch_key.starts_with("branch_")
        || kv_get(conn, NS, branch_key).is_none()
    {
        eprintln!(
            "[route] 拒绝 ghost branch '{}': 不存在于 kv_store (branch_% key). 先用 query --kw 或 get 确认真实 key.",
            branch_key
        );
        std::process::exit(1);
    }
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

/// 巡检 route_table: 校验每条路由指向的 branch_% key 真实存在于 kv_store。
/// --clean 移除 ghost 路由 (否则仅报告)。替代手工 SQL 编辑 (P1 修补的运维侧)。
fn cmd_route_verify(conn: &Connection, clean: bool) {
    let mut hub = ensure_hub(conn);
    let mut new_rt: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut ghost_total = 0usize;
    {
        let rt = hub["hub"]["route_table"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        for (kw, arr) in rt {
            let mut keep: Vec<Value> = Vec::new();
            let mut ghosts: Vec<String> = Vec::new();
            if let Some(list) = arr.as_array() {
                for b in list {
                    let key = b.as_str().unwrap_or("");
                    if key.starts_with("branch_") && kv_get(conn, NS, key).is_some() {
                        keep.push(b.clone());
                    } else {
                        ghost_total += 1;
                        ghosts.push(key.to_string());
                    }
                }
            }
            if !ghosts.is_empty() {
                eprintln!("[route-verify] ghost '{}' → {:?}", kw, ghosts);
            }
            if !keep.is_empty() {
                new_rt.insert(kw, json!(keep));
            }
        }
    }
    hub["hub"]["route_table"] = json!(new_rt);
    if clean {
        save_hub(conn, &hub);
        println!(
            "[route-verify] cleaned {} ghost route(s), {} routes remain",
            ghost_total,
            hub["hub"]["route_table"].as_object().map(|m| m.len()).unwrap_or(0)
        );
    } else {
        println!(
            "[route-verify] {} ghost route(s) found (use --clean to remove), {} routes",
            ghost_total,
            hub["hub"]["route_table"].as_object().map(|m| m.len()).unwrap_or(0)
        );
    }
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

/// 清理重复分支 (幂等): 内容空白归一化后完全相同 → 保留 cycle 最旧的一份,
/// 删除其余。同步三处: 删 kv_store 行 / 从引用分支的 concepts 摘除 / 概念图
/// branches 摘引用 / hub 指标刷新。--dry-run 只报告不删。
fn cmd_dedup(conn: &Connection, dry_run: bool) {
    let rows = scan_values(conn, "branch_");
    // key → (归一化 content, 原始 value, cycle)
    let mut norm: HashMap<String, Vec<(String, Value, String)>> = HashMap::new();
    for (key, value) in &rows {
        let Ok(v) = serde_json::from_str::<Value>(value) else { continue };
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let n = content.split_whitespace().collect::<String>().to_lowercase();
        if n.len() < 30 {
            continue;
        }
        let cycle = match v.get("cycle") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        };        norm.entry(n).or_default().push((key.clone(), v, cycle));
    }

    let mut to_delete: Vec<String> = Vec::new();
    let mut groups = 0;
    for group in norm.values() {
        if group.len() < 2 {
            continue;
        }
        groups += 1;
        // 保留 cycle 最旧 (数值最小) 且 key 字典序最小的
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| {
            let ca = parse_cycle(&a.2);
            let cb = parse_cycle(&b.2);
            ca.cmp(&cb).then_with(|| a.0.cmp(&b.0))
        });
        for (key, _, _) in sorted.iter().skip(1) {
            to_delete.push(key.clone());
        }
        println!(
            "  [dedup] 组: {} 份 (保留 {}) — 删 {}",
            sorted.len(),
            sorted[0].0,
            sorted
                .iter()
                .skip(1)
                .map(|x| x.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if to_delete.is_empty() {
        println!("[dedup] 无重复分支");
        return;
    }
    println!("[dedup] 发现 {} 组重复, 将删除 {} 条", groups, to_delete.len());
    if dry_run {
        println!("[dedup] (dry-run) 未删除 — 加 --dry-run 去掉则执行");
        return;
    }

    // 1. 从引用分支的 concepts 摘除: 被删分支本身若被 concept 引用, 概念图 branches 需摘
    let del_set: HashSet<String> = to_delete.iter().cloned().collect();
    // 2. 概念图: 每个 concept 的 branches 摘除被删分支
    for (ckey, cvalue) in scan_values(conn, "concept_") {
        let Ok(mut c) = serde_json::from_str::<Value>(&cvalue) else { continue };
        let mut changed = false;
        if let Some(bs) = c.get_mut("branches").and_then(|b| b.as_array_mut()) {
            let before = bs.len();
            bs.retain(|b| !del_set.contains(b.as_str().unwrap_or("")));
            changed = bs.len() != before;
        }
        if changed {
            kv_set(conn, NS, &ckey, &c.to_string());
        }
    }
    // 3. 删除 kv_store 行
    for key in &to_delete {
        conn.execute(
            "DELETE FROM kv_store WHERE namespace=?1 AND key=?2",
            params![NS, key],
        )
        .ok();
    }
    // 4. hub 指标刷新
    let mut hub = ensure_hub(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("[dedup] 已删除 {} 条重复分支, 概念图摘引用完成, hub 已刷新", to_delete.len());
}

fn parse_cycle(c: &str) -> i64 {
    // cycle 可能是 "186" / "105" / "161k" 等, 取前缀数字
    c.chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse::<i64>()
        .unwrap_or(i64::MAX)
}

/// 维度蒸馏 — 消退蒸馏核心: 细枝末节经验 → 能力网/意识体维度模式。
///
/// 协议 (三阶段):
///   1. 按 domain 分组 (可选 --domain 限定单域)。
///   2. 组内按主题关键词聚类: 提取每条 content 的高信号词 (非停用词),
///      词共现相似的两条归为一簇。
///   3. 对 ≥ min_group 条的簇: 生成一条 pattern 类型蒸馏条目 (distilled_from
///      记录溯源 keys), 原始条目标记 distilled:true 降权 (保留, 不删除)。
///
/// 设计依据: 经验无限追加会维度膨胀 — 高信号模式沉没在细枝末节中。
/// 蒸馏使经验维度向"能力级模式"收敛 (能力网维度), 原始条目降权为溯源证据。
fn cmd_distill(conn: &mut Connection, domain: Option<&str>, min_group: usize, dry_run: bool) {
    let rows = scan_values(conn, "branch_");

    // 1. 按 domain 分组 (跳过已蒸馏条目 — 幂等, 不重复蒸馏)
    let mut by_domain: HashMap<String, Vec<(String, Value)>> = HashMap::new();
    for (key, value) in &rows {
        let Ok(v) = serde_json::from_str::<Value>(value) else { continue };
        if v.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false) {
            continue;
        }
        let d = v.get("domain").and_then(|x| x.as_str()).unwrap_or("unknown");
        if let Some(want) = domain {
            if d != want {
                continue;
            }
        }
        by_domain.entry(d.to_string()).or_default().push((key.clone(), v));
    }

    // 2. 组内主题聚类 — 高信号词袋 Jaccard (并查集合并共享 ≥2 关键词的条目)
    let mut distilled: Vec<(String, String, Vec<String>)> = Vec::new(); // (domain, pattern_content, src_keys)
    let mut marked: Vec<String> = Vec::new(); // 标记 distilled 的 key
    for (d, items) in &by_domain {
        // 条目 → 高信号词集合 (限 top 12, 避免长条目过度重叠)
        let mut item_kws: HashMap<String, HashSet<String>> = HashMap::new();
        for (key, v) in items {
            let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
            let kws: HashSet<String> = high_signal_words(content).into_iter().take(12).collect();
            item_kws.insert(key.clone(), kws);
        }
        // 并查集: 两两共享 ≥3 个关键词 → 合并 (强主题信号, 避免过度合并)
        let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
        let mut parent: HashMap<String, String> = HashMap::new();
        for k in &keys {
            parent.insert(k.clone(), k.clone());
        }
        fn find(p: &mut HashMap<String, String>, x: &str) -> String {
            let root = p.get(x).cloned().unwrap_or_else(|| x.to_string());
            if root != x {
                let r = find(p, &root);
                p.insert(x.to_string(), r.clone());
                r
            } else {
                root
            }
        }
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let a = &keys[i];
                let b = &keys[j];
                let ka = item_kws.get(a).cloned().unwrap_or_default();
                let kb = item_kws.get(b).cloned().unwrap_or_default();
                let shared = ka.intersection(&kb).count();
                if shared >= 3 {
                    let ra = find(&mut parent, a);
                    let rb = find(&mut parent, b);
                    if ra != rb {
                        let rra = find(&mut parent, &ra);
                        parent.insert(rb.clone(), rra.clone());
                    }
                }
            }
        }
        // 收集簇
        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
        for k in &keys {
            let r = find(&mut parent, k);
            clusters.entry(r).or_default().push(k.clone());
        }
        for cluster in clusters.values() {
            if cluster.len() < min_group {
                continue;
            }
            let mut contents: Vec<String> = Vec::new();
            for k in cluster {
                if let Some(v) = items.iter().find(|(kk, _)| kk == k) {
                    if let Some(c) = v.1.get("content").and_then(|x| x.as_str()) {
                        contents.push(c.to_string());
                    }
                }
            }
            if contents.is_empty() {
                continue;
            }
            let pattern_content = distill_pattern(d, &contents);
            distilled.push((d.clone(), pattern_content, cluster.clone()));
            marked.extend(cluster.iter().cloned());
        }
    }

    if distilled.is_empty() {
        println!("[distill] 无满足条件 (min_group={}) 的蒸馏组", min_group);
        return;
    }
    println!("[distill] 发现 {} 个蒸馏组 (共标记 {} 条原始经验)", distilled.len(), marked.len());
    if dry_run {
        for (d, pc, src) in &distilled {
            println!(
                "  [dry-run] {} | {} ← {} 条",
                d,
                pc.chars().take(80).collect::<String>(),
                src.len()
            );
        }
        println!("[distill] (dry-run) 未落盘 — 去 --dry-run 则执行");
        return;
    }

    // 3. 落盘: 蒸馏模式 + 原始条目降权标记
    let now = now_ts();
    let tx = conn.transaction().expect("distill tx");
    for (d, pc, src) in &distilled {
        let branch_key = format!("branch_distill_{}_{}", d, now);
        let entry = json!({
            "schema_version": 1,
            "type": "pattern",
            "session_id": "distill",
            "cycle": "distill",
            "ts": now,
            "domain": d,
            "content": pc,
            "evidence": "",
            "source": "distill",
            "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            "distilled_from": src,
            // P0-2: 负例字段
            "not": Value::Null,
            // P0-1: 独立审计者字段
            "verified_by": Value::Null,
            "verification_status": Value::Null,
        });
        tx.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![NS, branch_key, value_encode(&entry.to_string()), now],
        )
        .expect("distill insert");
    }
    for key in &marked {
        if let Some((_, value)) = rows.iter().find(|(k, _)| k == key) {
            let Ok(mut v) = serde_json::from_str::<Value>(value) else { continue };
            if let Some(o) = v.as_object_mut() {
                o.insert("distilled".to_string(), json!(true));
            }
            tx.execute(
                "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .expect("distill mark");
        }
    }
    tx.commit().expect("distill commit");

    // 4. 意识体维度蒸馏: 聚合本次蒸馏的元认知信号 → 意识体维度 insight
    //    (ConsciousnessTree/GWT 消费: 域健康、主题演化、蒸馏收敛度)
    let mut dom_counts: HashMap<String, usize> = HashMap::new();
    for (d, _, src) in &distilled {
        *dom_counts.entry(d.clone()).or_default() += src.len();
    }
    let mut dom_v: Vec<(String, usize)> = dom_counts.into_iter().collect();
    dom_v.sort_by(|a, b| b.1.cmp(&a.1));
    let dom_str = dom_v
        .iter()
        .map(|(d, n)| format!("{}:{}", d, n))
        .collect::<Vec<_>>()
        .join(", ");
    let consciousness_entry = json!({
        "schema_version": 1,
        "type": "insight",
        "session_id": "distill",
        "cycle": "distill",
        "ts": now,
        "domain": "NT-META",
        "content": format!(
            "[意识体蒸馏] 本轮收敛 {} 组经验为 {} 条能力模式, 标记 {} 条原始经验。\
             跨域分布: {}. 意识体维度信号: 经验维度向能力网模式收敛, \
             细枝末节降权为溯源证据。",
            distilled.len(),
            distilled.len(),
            marked.len(),
            dom_str
        ),
        "evidence": "neotrix-experience distill",
        "source": "distill",
        "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
        "dimension": "consciousness",
        "distilled_from": marked.clone(),
        // P0-2: 负例字段
        "not": Value::Null,
        // P0-1: 独立审计者字段
        "verified_by": Value::Null,
        "verification_status": Value::Null,
    });
    let ckey = format!("branch_consciousness_{}", now);
    conn.execute(
        "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![NS, ckey, value_encode(&consciousness_entry.to_string()), now],
    )
    .expect("consciousness distill insert");

    // 5. 高信号提升: 蒸馏出的能力模式 → 能力树迭代目标 (经验升维到能力网维度)
    //    bridge 将每个蒸馏模式路由为 Strengthen/Bud 计划, 写入能力树 registry 文件的
    //    "experience_targets" 建议区 — 由 neotrix-capability scan --apply 消费执行
    let bridge_result = distill_promote_to_capability(&distilled);
    if !bridge_result.is_empty() {
        println!("[distill] 高信号提升: {} 条能力模式提升为能力树迭代目标", bridge_result.len());
        for line in bridge_result.iter().take(5) {
            println!("  [promote] {}", line);
        }
        if bridge_result.len() > 5 {
            println!("  ... 其余 {} 条", bridge_result.len() - 5);
        }
    }

    // 6. hub 指标刷新
    let mut hub = ensure_hub(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!(
        "[distill] 已落盘 {} 条能力模式 + 1 条意识体维度 insight, {} 条原始经验标记 distilled",
        distilled.len(),
        marked.len()
    );
}

/// 把蒸馏出的能力模式提升为能力树迭代目标 (经验升维: 细枝末节 → 能力网节点)。
/// 返回提升建议的行描述 (实际写入 capability_registry.json 的 experience_targets 区)。
///
/// 接入点: 蒸馏模式 (domain, pattern, src_keys) → ExperienceRouter.route_experience
///   → 能力标签路由 → EvolutionPlan (Strengthen 已有节点 / Bud 新节点建议)
fn distill_promote_to_capability(distilled: &[(String, String, Vec<String>)]) -> Vec<String> {
    use neotrix::neotrix::nt_capability_bridge::{
        ExperienceDimension, ExperienceEntry, ExperienceRouter, promote_to_file,
    };
    let mut promoted = Vec::new();
    let mut dims = Vec::new();
    for (d, pc, src) in distilled {
        let entry = ExperienceEntry {
            id: format!("distill_{}", src.first().cloned().unwrap_or_else(|| "?".to_string())),
            entry_type: "pattern".to_string(),
            domain_name: d.clone(),
            content: pc.clone(),
            not: None,
            confidence: 0.8,       // 蒸馏聚合模式, 信号高
            importance: 0.7,
            verified_by: None,
            verification_status: None,
        };
        let dim = ExperienceRouter::route_experience(&entry);
        match &dim {
            ExperienceDimension::CapabilityNetwork {
                domain,
                capability_tag,
                rationale,
                signal,
                ..
            } => {
                promoted.push(format!(
                    "{} → {} (signal={:.2}) | {}",
                    domain.as_str(), capability_tag, signal, rationale
                ));
            }
            ExperienceDimension::ConsciousnessAwakening { layer, signal, .. } => {
                promoted.push(format!(
                    "意识体觉醒 → {} (signal={:.2}) | {}",
                    layer, signal, pc.chars().take(60).collect::<String>()
                ));
            }
        }
        dims.push(dim);
    }
    // 写入能力树 registry 的 experience_targets 区 (经验 → 能力节点迭代目标闭环)
    // 优先 cwd (项目内, 被 git 追踪, 存在); home 目录为回退。两个都写, 确保闭环真实落盘。
    let cwd_registry = std::path::PathBuf::from(".neotrix/capability_registry.json");
    let home_registry = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".neotrix/capability_registry.json");
    let mut written_total = 0usize;
    for path in [&cwd_registry, &home_registry] {
        if path.exists() {
            written_total += promote_to_file(path, &dims);
        }
    }
    // 两个文件都不存在 → 创建 cwd 文件再写
    if written_total == 0 && !cwd_registry.exists() {
        if let Some(parent) = cwd_registry.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        written_total += promote_to_file(&cwd_registry, &dims);
    }
    if written_total > 0 {
        promoted.push(format!("写入 {} 条迭代目标到 {}", written_total, cwd_registry.display()));
    }
    promoted
}

/// 提取高信号词: 非停用词、非纯数字、长度 ≥3 的 ASCII 词 (小写去重)。
fn high_signal_words(content: &str) -> Vec<String> {
    let stop: HashSet<String> = en_stop().iter().map(|s| s.to_string()).collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for w in content.split(|c: char| !c.is_alphanumeric()) {
        let w = w.trim();
        if w.len() < 3 {
            continue;
        }
        let wl = w.to_lowercase();
        if wl.chars().any(|c| c.is_numeric()) {
            continue;
        }
        if stop.contains(&wl) {
            continue;
        }
        if seen.insert(wl.clone()) {
            out.push(wl);
        }
    }
    out
}

/// 蒸馏模式合成: 簇内经验 → 能力网维度模式。
/// 启发式: 最长 content 做骨架, 附簇规模 + 词频信号。
fn distill_pattern(domain: &str, contents: &[String]) -> String {
    let mut longest = String::new();
    for c in contents {
        if c.len() > longest.len() {
            longest = c.clone();
        }
    }
    let mut freq: HashMap<String, usize> = HashMap::new();
    for c in contents {
        for w in high_signal_words(c) {
            *freq.entry(w).or_default() += 1;
        }
    }
    let mut freq_v: Vec<(String, usize)> = freq.into_iter().collect();
    freq_v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_kws: Vec<String> = freq_v
        .iter()
        .filter(|(_, n)| *n >= 2)
        .take(8)
        .map(|(w, _)| w.clone())
        .collect();
    let kw_str = if top_kws.is_empty() {
        String::new()
    } else {
        format!(" [关键词: {}]", top_kws.join(", "))
    };
    format!(
        "[蒸馏-{}] 聚合 {} 条经验的模式: {}{}",
        domain,
        contents.len(),
        longest.chars().take(180).collect::<String>(),
        kw_str
    )
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
    for (i, ch) in c.char_indices() {
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
    ordered.sort_by_key(|b| std::cmp::Reverse(cycle_sort_key(b)));
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
    /// P0-2 G3+G8: 记录经验复用反馈 → MapeGate burn-in 门 (晋升/回滚)
    Feedback {
        key: String,
        /// success | failure
        #[arg(long, default_value = "success")]
        outcome: String,
    },
    /// 批量节点吸收 (R-P97: Python insert_node 的 Rust port — 知识写入单一事实源)
    /// 输入: JSON 文件 (节点数组或单节点), 每条含 node_type/title/summary/content/url/domain/
    ///       language/importance/meta; 可含 capability {branch,capability,evidence} 四元组。
    /// 语义: URL 去重 + nodes/nodes_fts 双写 (FTS 显式插入, 防 PA011 desync)。
    AbsorbNode {
        /// 节点 JSON 文件 (数组或单对象), 或 "-" 读 stdin
        #[arg(default_value = "-")]
        input: String,
        #[arg(long)]
        dry_run: bool,
        /// 写 metadata.absorbed_capability 四元组 (R-P79 闭环)
        #[arg(long)]
        apply_capability: bool,
    },
    /// 批量更新已有节点的 metadata (R-P97: absorb_to_capability.py 写回路径的 Rust port)
    /// 输入: JSON 数组, 每项 {node_id, patch: {key: value}} — 读原 metadata JSON →
    ///       合并 patch → 写回。patch 值可为任意 JSON (对象如 absorbed_capability 四元组)。
    UpdateNodeMetadata {
        /// 更新清单 JSON 文件, 或 "-" 读 stdin
        #[arg(default_value = "-")]
        input: String,
        #[arg(long)]
        dry_run: bool,
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
        #[arg(long)]
        json: bool,
        /// 增加 VSA 语义近邻信号: 检索后按嵌入相似度加权重排 (混合检索第三路)
        #[arg(long)]
        semantic: bool,
        /// 包含已蒸馏 (distilled) 的原始经验 — 默认过滤 (模式已升维, 原始条目仅作溯源)
        #[arg(long)]
        include_distilled: bool,
    },
    /// 列出条目
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        cycle: Option<String>,
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
    /// 巡检 route_table 幽灵路由 (--clean 移除)
    RouteVerify {
        #[arg(long)]
        clean: bool,
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
    /// 清理重复分支: 内容归一化相同 → 保留一份, 删其余 (含概念图摘引用 + hub 刷新)
    Dedup {
        #[arg(long)]
        dry_run: bool,
    },
    /// 维度蒸馏: 按域+主题聚类细枝末节经验 → 升维为能力网/意识体维度模式,
    /// 原始条目标记 distilled 降权 (保留溯源, 不删除)。消退蒸馏核心。
    Distill {
        #[arg(long)]
        domain: Option<String>,
        /// 组内最少条目数才蒸馏 (默认 3)
        #[arg(long, default_value_t = 3)]
        min_group: usize,
        #[arg(long)]
        dry_run: bool,
    },
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
    /// 测量两段文本的 VSA 词袋相似度 (阈值校准工具)
    Sim {
        #[arg(long, default_value = "")]
        a: String,
        #[arg(long, default_value = "")]
        b: String,
        #[arg(long, default_value_t = 2048)]
        dim: usize,
    },
    /// 记忆星系拓扑报告: 经验嵌入点云 → 持续同调 (Betti 数) + 记忆簇
    Topology {
        #[arg(long, default_value_t = 2048)]
        dim: usize,
        #[arg(long, default_value_t = 10)]
        steps: usize,
        /// 点云采样上限 (O(n³) 三角形计数防爆炸, 0=不限)
        #[arg(long, default_value_t = 400)]
        max_points: usize,
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut conn = open_kb();
    match cli.cmd {
        Cmd::Snapshot { cycle, task, domain } => cmd_snapshot(&conn, &cycle, &task, &domain),
        Cmd::Close { cycle } => cmd_close(&conn, &cycle),
        Cmd::Absorb { session } => cmd_absorb(&mut conn, &session),
        Cmd::Feedback { key, outcome } => cmd_feedback(&mut conn, &key, &outcome),
        Cmd::AbsorbNode { input, dry_run, apply_capability } => {
            cmd_absorb_node(&conn, &input, dry_run, apply_capability)
        }
        Cmd::UpdateNodeMetadata { input, dry_run } => {
            cmd_update_node_metadata(&conn, &input, dry_run)
        }
        Cmd::Query { kw, r#type, domain, limit, no_hebb, json, semantic, include_distilled } => {
            cmd_query(&conn, &kw, r#type.as_deref(), domain.as_deref(), limit, no_hebb, json, semantic, include_distilled);
        }
        Cmd::List { r#type, domain, cycle } => cmd_list(&conn, r#type.as_deref(), domain.as_deref(), cycle.as_deref()),
        Cmd::Stale { domain } => cmd_stale(&conn, domain.as_deref()),
        Cmd::Hub => cmd_hub(&conn),
        Cmd::Route { kw, branch } => cmd_route(&conn, &kw, &branch),
        Cmd::RouteVerify { clean } => cmd_route_verify(&conn, clean),
        Cmd::Neuron { term, exact } => cmd_neuron(&conn, &term, exact),
        Cmd::Backfill => cmd_backfill(&conn),
        Cmd::Prune { stop, stale_isolated } => cmd_prune(&conn, &stop, stale_isolated),
        Cmd::Hebb => cmd_hebb(&mut conn),
        Cmd::Dedup { dry_run } => cmd_dedup(&conn, dry_run),
        Cmd::Distill { domain, min_group, dry_run } => {
            cmd_distill(&mut conn, domain.as_deref(), min_group, dry_run)
        }
        Cmd::Compress { all } => cmd_compress(&mut conn, all),
        Cmd::GenIndex { out, limit } => cmd_gen_index(&conn, &out, limit),
        Cmd::Sim { a, b, dim } => cmd_sim(&a, &b, dim),        Cmd::Topology {
            dim,
            steps,
            max_points,
            json,
        } => cmd_topology(&conn, dim, steps, max_points, json),
    }
}

/// 测量两段文本的 VSA 词袋相似度 — 阈值校准工具。
fn cmd_sim(a: &str, b: &str, dim: usize) {
    let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
    let (va, _) = text_doc_vector(&a.to_lowercase(), dim, &mut memo);
    let (vb, _) = text_doc_vector(&b.to_lowercase(), dim, &mut memo);
    let sim = if va.is_empty() || vb.is_empty() {
        0.0
    } else {
        ghrr_similarity(&va, &vb)
    };
    println!("sim(a,b) = {:.6}  (dim={})", sim, dim);
    println!("  len(a)={} tokens, len(b)={} tokens", vsa_tokens(a).len(), vsa_tokens(b).len());
}

/// 记忆星系拓扑报告: 全量分支 → VSA 文档向量 (归一化) → 持续同调点云。
/// 输出 Betti 曲线 (β₀=记忆簇/组件, β₁=环路=反复出现的模式链, β₂=填充四面体≈高密度凸起)
/// + 积分估计 (Φ 代理) + 持久熵 + 选定尺度下的记忆簇成员 (凸起映射回真实分支)。
///
/// 归一化向量欧氏距离: 语义相关 ≈0.4-0.6, 无关 ≈1.0-1.4 → scale_max=0.8 已覆盖相关区。
/// O(n³) 三角形计数 → max_points 分层采样 (按 domain 均摊) 防爆炸。
#[allow(clippy::needless_range_loop)] // 矩阵双索引 (dists[i][j]) 迭代器改写不可读
fn cmd_topology(conn: &Connection, dim: usize, steps: usize, max_points: usize, json: bool) {
    ensure_hub(conn);
    let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
    let mut entries: Vec<(String, String, String, Vec<f64>)> = Vec::new();
    for (key, value) in scan_values(conn, "branch_") {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        let content = v
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_lowercase();
        if content.trim().is_empty() {
            continue;
        }
        let (raw, _) = text_doc_vector(&content, dim, &mut memo);
        let norm = l2_norm(&raw);
        let vec: Vec<f64> = if norm > 1e-12 {
            raw.iter().map(|x| x / norm).collect()
        } else {
            raw
        };
        entries.push((
            key,
            v.get("domain").and_then(|d| d.as_str()).unwrap_or("").to_string(),
            truncate(
                v.get("content").and_then(|c| c.as_str()).unwrap_or(""),
                60,
            ),
            vec,
        ));
    }
    if entries.len() < 2 {
        println!("[topology] 至少需要 2 个分支 (当前 {})", entries.len());
        return;
    }

    // 分层采样: 按 domain 均摊到 max_points, 保持域多样性
    if max_points > 0 && entries.len() > max_points {
        let mut by_domain: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            by_domain.entry(e.1.clone()).or_default().push(i);
        }
        let n_domains = by_domain.len();
        let per = (max_points / n_domains).max(1);
        let mut picked: Vec<usize> = Vec::with_capacity(max_points);
        for idxs in by_domain.values() {
            picked.extend(idxs.iter().take(per));
        }
        if picked.len() < max_points {
            let mut rest: Vec<usize> = (0..entries.len()).filter(|i| !picked.contains(i)).collect();
            rest.sort_by_key(|i| std::cmp::Reverse((entries[*i].1.len(), 0)));
            picked.extend(rest.into_iter().take(max_points - picked.len()));
        }
        let mut filtered: Vec<(String, String, String, Vec<f64>)> = Vec::with_capacity(picked.len());
        for &i in &picked {
            filtered.push(entries[i].clone());
        }
        entries = filtered;
    }

    let mut cloud = PointCloud::new("memory-galaxy");
    for e in &entries {
        cloud.add_point(e.3.clone());
    }

    let scale_max = 0.8f64;
    let ph = PersistentHomology::compute(&cloud, scale_max, steps);

    // 报告 Betti 曲线 (采样几个代表性尺度)
    let mut curve_lines = Vec::new();
    for (s, b) in &ph.betti_curves {
        curve_lines.push(format!(
            "  scale={:.2} β0={} β1={} β2={}",
            s, b.beta_0, b.beta_1, b.beta_2
        ));
    }
    let phi = ph
        .simplified_betti()
        .integration_estimate();
    let entropy = ph.persistence_entropy();

    // 记忆簇: 在 scale 0.6 (语义相近距离) 做 union-find 聚类, 输出≥2 成员的簇
    let mut parent: Vec<usize> = (0..entries.len()).collect();
    let dists = cloud_distance_matrix(&cloud);
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            if dists[i][j] <= 0.6 {
                cluster_union(&mut parent, i, j);
            }
        }
    }
    let mut roots: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..parent.len() {
        let r = cluster_find(&mut parent, i);
        roots.entry(r).or_default().push(i);
    }
    let mut clusters: Vec<Vec<usize>> = roots.into_values().filter(|c| c.len() >= 2).collect();
    clusters.sort_by_key(|c| std::cmp::Reverse(c.len()));

    if json {
        let mut cluster_json = Vec::new();
        for c in &clusters {
            let members: Vec<Value> = c
                .iter()
                .map(|&i| {
                    json!({
                        "key": entries[i].0,
                        "domain": entries[i].1,
                        "content": entries[i].2,
                    })
                })
                .collect();
            cluster_json.push(json!({ "size": c.len(), "members": members }));
        }
        let out = json!({
            "dim": dim,
            "points": entries.len(),
            "betti": ph.betti_curves.iter().map(|(s, b)| json!({
                "scale": s, "beta_0": b.beta_0, "beta_1": b.beta_1, "beta_2": b.beta_2
            })).collect::<Vec<_>>(),
            "integration_estimate": phi,
            "persistence_entropy": entropy,
            "clusters": cluster_json,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("[topology] 记忆星系拓扑 — {} 个分支, dim={}", entries.len(), dim);
        println!("Betti 曲线 (scale∈[0,{}], {} 步):", scale_max, steps);
        for l in curve_lines.iter().take(6) {
            println!("{}", l);
        }
        if steps > 6 {
            println!("  ... 共 {} 步, 中间省略 ...", steps + 1);
            for l in curve_lines.iter().skip(curve_lines.len() - 2) {
                println!("{}", l);
            }
        }
        println!(
            "integration_estimate (Φ 代理) = {:.4}, persistence_entropy = {:.4}",
            phi, entropy
        );
        println!("\n记忆簇 (scale≤0.6, 语义相近 ≥2 分支): {} 个", clusters.len());
        for (ci, c) in clusters.iter().enumerate().take(10) {
            println!(" 簇 #{} ({} 分支):", ci + 1, c.len());
            for &i in c.iter().take(5) {
                println!("   · [{}] {} — {}", entries[i].1, entries[i].0, entries[i].2);
            }
            if c.len() > 5 {
                println!("   ... 其余 {} 分支", c.len() - 5);
            }
        }
    }
}

fn l2_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

#[allow(clippy::needless_range_loop)] // 矩阵双索引 (dists[i][j]) 迭代器改写不可读
fn cloud_distance_matrix(cloud: &PointCloud) -> Vec<Vec<f64>> {
    let n = cloud.n();
    let mut dists = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = l2_distance(&cloud.points[i], &cloud.points[j]);
            dists[i][j] = d;
            dists[j][i] = d;
        }
    }
    dists
}

fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f64>()
        .sqrt()
}

fn cluster_find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn cluster_union(parent: &mut [usize], a: usize, b: usize) {
    let ra = cluster_find(parent, a);
    let rb = cluster_find(parent, b);
    if ra != rb {
        parent[ra] = rb;
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

    // ─── R-P97: absorb-node 测试 ──────────────────────────────
    fn node_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (
                id TEXT PRIMARY KEY, node_type TEXT NOT NULL, title TEXT NOT NULL,
                summary TEXT, content TEXT, url TEXT, domain TEXT,
                language TEXT DEFAULT 'en', confidence REAL DEFAULT 1.0,
                importance REAL DEFAULT 0.5, created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL, access_count INTEGER DEFAULT 0,
                metadata TEXT, data_tier TEXT NOT NULL DEFAULT 'core',
                temporal TEXT, supersedes TEXT, source_episode TEXT,
                tier TEXT NOT NULL DEFAULT 'warm');
             CREATE VIRTUAL TABLE nodes_fts USING fts5(title, summary, content, domain);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_absorb_node_insert_and_fts() {
        let conn = node_test_db();
        let node = json!({
            "url": "https://example.github.io/demo/",
            "title": "Demo Page",
            "summary": "A test article",
            "content": "This is a test article body with enough length to be meaningful for the FTS index.",
            "node_type": "article",
            "language": "en",
            "domain": "example.github.io",
            "importance": 0.7,
        });
        // 写临时文件 (cmd_absorb_node 读文件)
        let path = std::env::temp_dir().join("nt_test_node.json");
        std::fs::write(&path, node.to_string()).unwrap();
        let dry_run = false;
        let apply_cap = false;
        cmd_absorb_node(&conn, path.to_str().unwrap(), dry_run, apply_cap);
        // 节点已写入
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "node inserted");
        // FTS 已同步 (防 PA011 desync)
        let fts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fts_count, 1, "FTS row inserted");
        // id 前缀 batch_
        let id: String = conn
            .query_row("SELECT id FROM nodes LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert!(id.starts_with("batch_"), "id prefix batch_: {}", id);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_absorb_node_duplicate_dedup() {
        let conn = node_test_db();
        let node = json!({
            "url": "https://example.github.io/demo/",
            "title": "Demo Page",
            "content": "Same URL must be deduplicated.",
            "node_type": "article",
        });
        let path = std::env::temp_dir().join("nt_test_node2.json");
        std::fs::write(&path, node.to_string()).unwrap();
        cmd_absorb_node(&conn, path.to_str().unwrap(), false, false);
        cmd_absorb_node(&conn, path.to_str().unwrap(), false, false);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "duplicate URL must not double-insert");
        let fts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fts_count, 1, "FTS also deduplicated");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_absorb_node_dry_run_and_capability() {
        let conn = node_test_db();
        let node = json!({
            "url": "https://example.github.io/cap/",
            "title": "Cap Page",
            "content": "Capability mapping test node with sufficient content length.",
            "node_type": "article",
            "capability": {"branch": "NT-MIND", "capability": "generate", "evidence": "test"},
        });
        let path = std::env::temp_dir().join("nt_test_node3.json");
        std::fs::write(&path, node.to_string()).unwrap();
        // dry-run: 不写入
        cmd_absorb_node(&conn, path.to_str().unwrap(), true, false);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "dry-run must not insert");
        // 实际写入 + capability
        cmd_absorb_node(&conn, path.to_str().unwrap(), false, true);
        let meta: String = conn
            .query_row("SELECT metadata FROM nodes WHERE url='https://example.github.io/cap/'",
                       [], |r| r.get(0))
            .unwrap();
        let m: Value = serde_json::from_str(&meta).unwrap();
        assert_eq!(m["absorbed_capability"]["branch"], "NT-MIND");
        assert_eq!(m["absorbed_capability"]["capability"], "generate");
        assert_eq!(m["absorbed_capability"]["evidence"], "test");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_update_node_metadata_merge_and_dry_run() {
        let conn = node_test_db();
        // 先插入一个带初始 metadata 的节点
        let node = json!({
            "url": "https://example.github.io/meta/",
            "title": "Meta Page",
            "content": "Metadata update test node with sufficient content length.",
            "node_type": "article",
            "meta": {"existing": "keep-me"},
        });
        let path = std::env::temp_dir().join("nt_test_meta_node.json");
        std::fs::write(&path, node.to_string()).unwrap();
        cmd_absorb_node(&conn, path.to_str().unwrap(), false, false);
        let nid: String = conn
            .query_row("SELECT id FROM nodes LIMIT 1", [], |r| r.get(0))
            .unwrap();
        std::fs::remove_file(&path).ok();

        // dry-run: 不写入
        let updates = json!([{
            "node_id": nid,
            "patch": {"absorbed_capability": {"branch": "NT-ACT", "capability": "execute"}}
        }]);
        let up = std::env::temp_dir().join("nt_test_update.json");
        std::fs::write(&up, updates.to_string()).unwrap();
        cmd_update_node_metadata(&conn, up.to_str().unwrap(), true);
        let meta: String = conn
            .query_row("SELECT metadata FROM nodes WHERE id=?1", params![nid], |r| r.get(0))
            .unwrap();
        let m: Value = serde_json::from_str(&meta).unwrap();
        assert_eq!(m["existing"], "keep-me", "dry-run must not modify metadata");
        assert!(m.get("absorbed_capability").is_none(), "dry-run must not add capability");

        // 实际写入: 合并 patch, 保留既有字段
        cmd_update_node_metadata(&conn, up.to_str().unwrap(), false);
        let meta: String = conn
            .query_row("SELECT metadata FROM nodes WHERE id=?1", params![nid], |r| r.get(0))
            .unwrap();
        let m: Value = serde_json::from_str(&meta).unwrap();
        assert_eq!(m["existing"], "keep-me", "existing metadata preserved");
        assert_eq!(m["absorbed_capability"]["branch"], "NT-ACT");
        assert_eq!(m["absorbed_capability"]["capability"], "execute");
        std::fs::remove_file(&up).ok();
    }

    #[test]
    fn test_high_signal_words() {
        let ws = high_signal_words("the neural network training on GPU failed");
        // 停用词 the/on 剔除, 数字剔除, 短词剔除
        assert!(!ws.contains(&"the".to_string()));
        assert!(!ws.contains(&"on".to_string()));
        assert!(ws.contains(&"neural".to_string()));
        assert!(ws.contains(&"training".to_string()));
        // 去重
        let ws2 = high_signal_words("error error error retry");
        assert_eq!(ws2.iter().filter(|w| *w == "error").count(), 1);
        assert!(ws2.contains(&"retry".to_string()));
    }

    #[test]
    fn test_distill_pattern_aggregates() {
        let contents = vec![
            "neural network training failed on GPU memory".to_string(),
            "neural network training needs more GPU memory".to_string(),
            "neural network training error GPU memory overflow".to_string(),
        ];
        let p = distill_pattern("NT-CORE", &contents);
        assert!(p.starts_with("[蒸馏-NT-CORE]"));
        assert!(p.contains("聚合 3 条经验"));
        // 高频词 neural/network/training 应出现在关键词区
        assert!(p.contains("neural"));
        assert!(p.contains("training"));
    }

    #[test]
    fn test_cmd_distill_dry_run_marks_nothing() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        // 插入 3 条同主题经验 (NT-CORE)
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "source": "test",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        // dry-run: 不落盘蒸馏, 不标记
        cmd_distill(&mut conn, Some("NT-CORE"), 3, true);
        let cnt: i64 = conn
            .query_row("SELECT COUNT(*) FROM kv_store WHERE namespace=?1", params![NS], |r| r.get(0))
            .unwrap();
        assert_eq!(cnt, 3, "dry-run must not add distilled pattern");
        let marked: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kv_store WHERE namespace=?1 AND value LIKE '%distilled%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(marked, 0, "dry-run must not mark distilled");
    }

    #[test]
    fn test_cmd_distill_creates_pattern_and_marks() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        cmd_distill(&mut conn, Some("NT-CORE"), 3, false);
        // 新增 1 条蒸馏 pattern (key 前缀 branch_distill_)
        let patterns: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kv_store WHERE namespace=?1 AND key LIKE 'branch_distill_%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(patterns, 1, "应生成 1 条蒸馏模式");
        // 3 条原始经验标记 distilled (解码后检查)
        let mut marked = 0;
        for (_, value) in scan_values(&conn, "branch_test_") {
            let j: Value = serde_json::from_str(&value).unwrap();
            if j.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false) {
                marked += 1;
            }
        }
        assert_eq!(marked, 3, "3 条原始经验应标记 distilled");
    }

    #[test]
    fn test_cmd_distill_generates_consciousness_entry() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        cmd_distill(&mut conn, Some("NT-CORE"), 3, false);
        // 意识体维度条目生成
        let ckey: String = conn
            .query_row(
                "SELECT key FROM kv_store WHERE namespace=?1 AND key LIKE 'branch_consciousness_%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        let (_, cvalue) = scan_values(&conn, "branch_consciousness_")
            .into_iter()
            .next()
            .unwrap();
        let c: Value = serde_json::from_str(&cvalue).unwrap();
        assert_eq!(c["dimension"], "consciousness");
        assert_eq!(c["type"], "insight");
        assert!(c["content"].as_str().unwrap().contains("意识体蒸馏"));
        assert!(c["distilled_from"].as_array().unwrap().len() >= 3);
        let _ = ckey;
    }

    #[test]
    fn test_query_filters_distilled_by_default() {
        let conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        // 2 条普通 + 1 条 distilled
        let mut entries = vec![
            ("branch_a_1", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training tip one", "evidence": "f:1",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
            ("branch_a_2", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training tip two", "evidence": "f:2",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
            ("branch_a_3", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training distilled old", "evidence": "f:3",
                "distilled": true, "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
        ];
        for (k, v) in entries.drain(..) {
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, k, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        // 默认过滤 distilled → 2 条
        let res = cmd_query(&conn, "neural", None, None, 10, false, false, false, false);
        assert_eq!(res, 2, "默认应过滤 distilled 条目, 得到 {}", res);
        // include_distilled → 3 条
        let res2 = cmd_query(&conn, "neural", None, None, 10, false, false, false, true);
        assert_eq!(res2, 3, "include_distilled 应含原始条目");
    }
}
