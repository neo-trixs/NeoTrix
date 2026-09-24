//! exp_util — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use serde_json::Value;
use std::collections::HashSet;
use super::{CN_STOP, EN_STOP};
use sha1::{Digest, Sha1};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::TimeZone;

pub(crate) fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() { return 1; }
    let mut tokens = 0.0f64;
    for c in text.chars() {
        tokens += char_token_cost(c);
    }
    (tokens.ceil() as usize).max(1)
}

/// 单字符 token 成本 (CJK ≈ 1.3, ASCII alnum ≈ 0.25, 其他 ≈ 0.5)。


pub(crate) fn char_token_cost(c: char) -> f64 {
    if c.is_ascii_alphanumeric() { 0.25 }
    else if c.is_ascii_punctuation() || c.is_ascii_whitespace() { 0.2 }
    else if c as u32 >= 0x4E00 && c as u32 <= 0x9FFF { 1.3 }
    else if c as u32 >= 0x3040 && c as u32 <= 0x30FF { 1.0 }
    else if c as u32 >= 0xAC00 && c as u32 <= 0xD7AF { 1.0 }
    else { 0.5 }
}


pub(crate) fn en_stop() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| EN_STOP.iter().copied().collect())
}


pub(crate) fn cn_stop() -> &'static HashSet<char> {
    static SET: OnceLock<HashSet<char>> = OnceLock::new();
    SET.get_or_init(|| CN_STOP.chars().collect())
}

// ─── value 透明压缩层 (方案 D) ─────────────────────────────────────
pub(crate) const VALUE_MAGIC: &[u8] = b"NTZ1";

// normalize_url 已统一到 neotrix::l0_substrate::nt_core_math::normalize_url


pub(crate) fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}


pub(crate) fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Python `str(float)` 表示: 整值浮点带 `.0` (5.0 → "5.0", 5.5 → "5.5").


pub(crate) fn py_float_repr(f: f64) -> String {
    if f.fract() == 0.0 && f.is_finite() {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

/// Python `repr(dict)` 表示: `{'k': v, 'k2': v2}` (单引号, `: ` 与 `, ` 分隔).


pub(crate) fn py_dict_repr(v: &Value) -> String {
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


pub(crate) fn py_repr_scalar(v: &Value) -> String {
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


pub(crate) fn cycle_opt(v: Option<&Value>) -> Option<String> {
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


pub(crate) fn seed_from_str(s: &str) -> u64 {
    let mut h = Sha1::new();
    h.update(s.as_bytes());
    let d = h.finalize();
    u64::from_be_bytes(d[..8].try_into().expect("SHA1 digest >= 8 bytes"))
}

/// CJK 2-gram / ASCII 空白分词的 token 列表 (VSA 词袋)。


pub(crate) fn fmt_ts(ts: i64) -> String {
    chrono::Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string())
}

// ────────────────────────────────────────────────────────────────
// 1. Snapshot 快照 / Close
// ────────────────────────────────────────────────────────────────


pub(crate) fn uuid_hex(len: usize) -> String {
    uuid::Uuid::new_v4().simple().to_string()[..len].to_string()
}

// ────────────────────────────────────────────────────────────────
// 2-4. Absorb 蒸馏 → 分类 → 落盘
// ────────────────────────────────────────────────────────────────


pub(crate) fn parse_cycle(c: &str) -> i64 {
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


pub(crate) fn cycle_sort_key(c: &str) -> (i64, String) {
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
