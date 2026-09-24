//! exp_concept — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};

pub(crate) fn vsa_tokens(s: &str) -> Vec<String> {
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


pub(crate) fn text_doc_vector(s: &str, dim: usize, memo: &mut HashMap<String, Vec<f64>>) -> (Vec<f64>, usize) {
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


pub(crate) fn concept_hash(term: &str) -> String {
    let mut h = Sha1::new();
    h.update(term.as_bytes());
    hex::encode(h.finalize())[..16].to_string()
}

/// 从经验正文抽取概念词集 — 神经网络化的关键一步。


pub(crate) fn extract_concepts(content: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if content.is_empty() {
        return found;
    }
    let en_re = regex::Regex::new(r"[A-Za-z][A-Za-z0-9_/\-]{3,}").expect("valid regex");
    let cn_re = regex::Regex::new(r"[\u{4e00}-\u{9fff}]{3,12}").expect("valid regex");
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

/// ────────────────────────────────────────────────────────────────
/// Extractor 管线 (Mastra OM 吸收, 2026-08-24):
/// - zod-like schema 驱动 (serde_json Value 作为 schema 描述)
/// - 失败隔离: 单个 extractor 失败不阻塞管线, 错误记录在 extracted.errors
/// - on_extracted 钩子: 可选的后处理 (normalize/react/signal)
/// - 内置: current_task, suggested_response, thread_title
/// ────────────────────────────────────────────────────────────────
use std::collections::HashMap as StdHashMap;
use super::{CONCEPT_MAX_LEN, CONCEPT_MIN_LEN, NS};
use super::exp_store::{kv_get, kv_set, load_json};
use super::exp_util::{cn_stop, en_stop, now_ts, seed_from_str};
use sha1::{Digest, Sha1};
use neotrix::l2_perception::nt_core_hcube::ghrr_vsa::{ghrr_bundle, ghrr_random_vector_dim};

/// Extractor 定义: name + prompt/schema + 可选 on_extracted 后处理闭包。
/// 为避免闭包序列化/跨线程问题, on_extracted 用 fn(&mut Value) -> Result<(), String> 形式。
pub(crate) type ExtractorFn = fn(&str, &Value) -> Result<Value, String>;

#[derive(Clone)]
pub(crate) struct Extractor {
    name: String,
    schema: Value,          // zod-like: {"type": "object", "properties": {...}}
    extract: ExtractorFn,
    on_extracted: Option<fn(&mut Value) -> Result<(), String>>,
}

impl Extractor {
    fn new(name: &str, schema: Value, extract: ExtractorFn) -> Self {
        Self { name: name.to_string(), schema, extract, on_extracted: None }
    }
    // (with_hook 零调用已删除; on_extracted 字段保留, run_extractors 照常执行钩子)
}

/// 运行所有 extractors, 失败隔离 + on_extracted 钩子执行。


pub(crate) fn run_extractors(content: &str, extractors: &[Extractor]) -> Value {
    let mut results = StdHashMap::new();
    let mut errors = Vec::new();
    for ext in extractors {
        match (ext.extract)(content, &ext.schema) {
            Ok(mut val) => {
                if let Some(hook) = ext.on_extracted {
                    if let Err(e) = hook(&mut val) {
                        errors.push(format!("{}: on_extracted hook failed: {}", ext.name, e));
                    }
                }
                results.insert(ext.name.clone(), val);
            }
            Err(e) => {
                errors.push(format!("{}: {}", ext.name, e));
            }
        }
    }
    if !errors.is_empty() {
        results.insert("errors".to_string(), json!(errors));
    }
    json!(results)
}

/// 内置 extractor: current_task — 从内容推断当前任务 (启发式: 首行/动词/关键词)


pub(crate) fn extract_current_task(content: &str, _schema: &Value) -> Result<Value, String> {
    let first_line = content.lines().next().unwrap_or("").trim();
    let verbs = ["实现", "修复", "重构", "分析", "设计", "测试", "部署", "吸收", "蒸馏", "审查", "调试"];
    let mut task = first_line.to_string();
    for v in &verbs {
        if content.contains(v) {
            task = format!("{} ({})", v, first_line.chars().take(40).collect::<String>());
            break;
        }
    }
    Ok(json!({ "task": task, "confidence": 0.6 }))
}

/// 内置 extractor: suggested_response — 推荐下一步动作


pub(crate) fn extract_suggested_response(content: &str, _schema: &Value) -> Result<Value, String> {
    let has_error = content.to_lowercase().contains("error") || content.contains("失败") || content.contains("报错");
    let has_test = content.contains("测试") || content.contains("test");
    let mut actions = Vec::new();
    if has_error { actions.push("diagnose_root_cause"); }
    if has_test { actions.push("run_tests"); }
    if actions.is_empty() { actions.push("continue_monitoring"); }
    Ok(json!({ "actions": actions, "priority": if has_error { "high" } else { "normal" } }))
}

/// 内置 extractor: thread_title — 生成线程标题


pub(crate) fn extract_thread_title(content: &str, _schema: &Value) -> Result<Value, String> {
    let words: Vec<&str> = content.split_whitespace().take(8).collect();
    let title = if words.is_empty() { "untitled".to_string() } else { words.join(" ") };
    Ok(json!({ "title": title }))
}

/// 内置 extractors 集合 (可配置/扩展)


pub(crate) fn builtin_extractors() -> Vec<Extractor> {
    vec![
        Extractor::new("current_task", json!({"type": "object", "properties": {"task": {"type": "string"}, "confidence": {"type": "number"}}}), extract_current_task),
        Extractor::new("suggested_response", json!({"type": "object", "properties": {"actions": {"type": "array", "items": {"type": "string"}}, "priority": {"type": "string"}}}), extract_suggested_response),
        Extractor::new("thread_title", json!({"type": "object", "properties": {"title": {"type": "string"}}}), extract_thread_title),
    ]
}


pub(crate) fn load_concept(conn: &Connection, ch: &str) -> Option<Value> {
    let raw = kv_get(conn, NS, &format!("concept_{}", ch))?;
    serde_json::from_str(&raw).ok()
}

/// 概念去重落盘: 已存在 → 加引用+count; 不存在 → 新建神经元。


pub(crate) fn concept_from_branch(conn: &Connection, term: &str, branch_key: &str, domain: &str) -> String {
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
    let Some(branches) = c.get_mut("branches").and_then(|v| v.as_array_mut()) else { return Default::default(); };
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


pub(crate) fn co_full(c: &Value) -> HashMap<String, f64> {
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


pub(crate) fn co_bump(conn: &Connection, a: &str, b: &str) {
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
    let Some(obj) = ca.as_object_mut() else { return; };
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


pub(crate) fn hebb_cooccurrence(conn: &Connection, hashes: &[String]) {
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            co_bump(conn, &hashes[i], &hashes[j]);
            co_bump(conn, &hashes[j], &hashes[i]);
        }
    }
}

/// 内存版 co_bump (cmd_hebb 全量重建用, 避免逐对 commit 的 O(n) DB 往返)。


pub(crate) fn co_bump_in_mem(concepts: &mut HashMap<String, Value>, x: &str, y: &str) {
    let Some(c) = concepts.get_mut(x) else { return };
    let Some(obj) = c.as_object_mut() else { return };
    let in_cow = obj.get("co_w").and_then(|m| m.get(y)).cloned();
    if let Some(w) = in_cow {
        if let Some(cow) = obj.get_mut("co_w").and_then(|v| v.as_object_mut()) {
            cow.insert(y.to_string(), json!(w.as_i64().unwrap_or(1) + 1));
        }
        return;
    }
    let in_co = obj
        .get("co")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().any(|v| v.as_str() == Some(y)))
        .unwrap_or(false);
    if in_co {
        if let Some(arr) = obj.get_mut("co").and_then(|v| v.as_array_mut()) {
            arr.retain(|v| v.as_str() != Some(y));
        }
        if let Some(cow) = obj.get_mut("co_w").and_then(|v| v.as_object_mut()) {
            cow.insert(y.to_string(), json!(2));
        }
    } else if let Some(arr) = obj.get_mut("co").and_then(|v| v.as_array_mut()) {
        arr.push(json!(y));
    }
}
