//! 规则记忆体 (Rule Memory) — 存储单位从"字节"升级为"可执行生成规则"。
//!
//! KB 作为意识体大脑的重构层 (L3 Memory): 经验实例 (experience namespace)
//! 经结晶流水线 detect→compile→verify→replace 晋升为 RuleRecord 四元组:
//!
//! ```text
//! Rule = (触发签名, 生成器, 验证器, 出处)
//! ```
//!
//! - **触发签名**: 文本 + 字符三元组特征散列双极超向量 (VSA), 余弦匹配路由
//! - **生成器**: 安全模板 DSL (`{{slot}}` + `|trim |upper |lower`), 无任意代码执行
//! - **验证器**: round-trip 重放 — 从实例提取槽值再渲染必须还原原文。
//!   同形状组经对齐编译后数学上保证无损还原 (保真对象为空白归一化文本);
//!   Quarantined 状态由事后 drift 检测 (verify_rule) 驱动。
//! - **出处**: experience namespace 的源 key 集合 (可回溯/可撤销/幂等去重)
//!
//! 语义: 记忆=程序, 回忆=执行。字节损坏静默; 规则损坏在下次执行时抛错 —
//! 唯一一种"会自我报错"的存储介质。GC 遵循 Dark Forest: 无消费者或验证
//! 连续失败的规则直接删除。

use crate::core::nt_core_kb_primitives::{kv_delete, kv_get, kv_list, kv_set, now};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// rule namespace — KB kv_store 中的规则记忆命名空间 (单一事实源)。
pub const NS_RULE: &str = "rule";
/// 默认超向量维度 (存储经济性与区分度平衡)。
pub const HV_DIM: usize = 512;
/// exec 路由默认余弦阈值。
pub const DEFAULT_MATCH_THRESHOLD: f64 = 0.18;

// ─── 状态与记录 ─────────────────────────────────────────────────────────────

/// 规则生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    /// 验证通过, 可被 exec 路由命中
    Active,
    /// drift 验证失败, 只能查询不能执行
    Quarantined,
    /// Dark Forest 标记: 无消费者的闲置规则, 下轮 GC 删除
    Retired,
}

/// RuleRecord 四元组 — 可执行记忆的存储单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRecord {
    pub id: String,
    pub domain: String,
    /// 人类可读触发描述 (exec 路由用其超向量匹配)
    pub trigger_text: String,
    /// 触发签名超向量 (bipolar i8, 维度 hv_dim)
    pub trigger_hv: Vec<i8>,
    pub hv_dim: usize,
    /// 生成器模板 (安全 DSL: {{slot}} 变量替换)
    pub generator: String,
    /// 模板槽名 (有序去重)
    pub slots: Vec<String>,
    /// 出处: experience namespace 源 key 集合
    pub provenance_keys: Vec<String>,
    pub confidence: f64,
    pub status: RuleStatus,
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_verified_at: Option<i64>,
    pub exec_count: u64,
    pub fail_count: u64,
}

impl RuleRecord {
    pub fn new(id: String, domain: String, trigger_text: String, generator: String, now_ts: i64) -> Self {
        let trigger_hv = text_hypervector(&trigger_text, HV_DIM);
        let slots = template_slots(&generator).unwrap_or_default();
        Self {
            id,
            domain,
            trigger_text,
            trigger_hv,
            hv_dim: HV_DIM,
            generator,
            slots,
            provenance_keys: Vec::new(),
            confidence: 0.0,
            status: RuleStatus::Quarantined,
            version: 1,
            created_at: now_ts,
            updated_at: now_ts,
            last_verified_at: None,
            exec_count: 0,
            fail_count: 0,
        }
    }
}

// ─── 安全模板 DSL: 统一分段解析 ─────────────────────────────────────────────

/// 模板段: 字面量或带变换链的变量。
#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Lit(String),
    Var { name: String, transforms: Vec<String> },
}

/// 解析模板为段序列。未闭合的 `{{` 视为语法错误 (fail-loud, 不静默吞掉记忆片段)。
fn parse_template(template: &str) -> Result<Vec<Segment>, String> {
    let mut out = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            return Err(format!("模板存在未闭合的 '{{{{': near '{}'", {
                let take: String = after.chars().take(20).collect();
                take
            }));
        };
        if start > 0 {
            out.push(Segment::Lit(rest[..start].to_string()));
        }
        let expr = &after[..end];
        let mut parts = expr.split('|');
        let name = parts.next().unwrap_or("").trim().to_string();
        if name.is_empty() {
            return Err("空变量名 {{}}".into());
        }
        out.push(Segment::Var {
            name,
            transforms: parts.map(|p| p.trim().to_string()).collect(),
        });
        rest = &after[end + 2..];
    }
    if !rest.is_empty() {
        out.push(Segment::Lit(rest.to_string()));
    }
    Ok(out)
}

const KNOWN_TRANSFORMS: [&str; 3] = ["trim", "upper", "lower"];

/// 提取模板槽名 (有序去重)。语法非法返回 Err。
pub fn template_slots(template: &str) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for seg in parse_template(template)? {
        if let Segment::Var { name, .. } = seg {
            if seen.insert(name.clone()) {
                out.push(name);
            }
        }
    }
    Ok(out)
}

/// 应用变换链。未知变换报错 (白名单制, 杜绝注入面)。
fn apply_transforms(mut val: String, transforms: &[String]) -> Result<String, String> {
    for t in transforms {
        match t.as_str() {
            "trim" => val = val.trim().to_string(),
            "upper" => val = val.to_uppercase(),
            "lower" => val = val.to_lowercase(),
            other => return Err(format!("未知变换 '|{other}' (允许: {KNOWN_TRANSFORMS:?})")),
        }
    }
    Ok(val)
}

/// 渲染模板: 缺失变量返回 Err (fail-loud, 不静默产出半成品记忆)。
pub fn render_template(template: &str, vars: &BTreeMap<String, String>) -> Result<String, String> {
    let segments = parse_template(template)?;
    for seg in &segments {
        if let Segment::Var { name, .. } = seg {
            if !vars.contains_key(name) {
                return Err(format!("缺少变量 '{name}'"));
            }
        }
    }
    let mut out = String::with_capacity(template.len());
    for seg in segments {
        match seg {
            Segment::Lit(s) => out.push_str(&s),
            Segment::Var { name, transforms } => {
                let raw = vars.get(&name).cloned().unwrap_or_default();
                out.push_str(&apply_transforms(raw, &transforms)?);
            }
        }
    }
    Ok(out)
}

// ─── 触发签名: 文本 → 双极超向量 ────────────────────────────────────────────

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// splitmix64 终态雪崩 — FNV 对顺序小整数输入游程相关, 必须再打散。
fn avalanche(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// 全局共享随机基座 (模块加载时由固定种子生成, 所有向量复用)。
/// 确保三元组命中相同维度, 中心化余弦可消除基座偏置。
static BASE_HV: std::sync::OnceLock<Vec<i8>> = std::sync::OnceLock::new();
fn get_base_hv(dim: usize) -> &'static [i8] {
    BASE_HV.get_or_init(|| {
        (0..dim).map(|i| {
            let h = avalanche(i as u64);
            if (h >> 63) == 1 { -1 } else { 1 }
        }).collect()
    })
}

/// 字符三元组特征散列 → 双极超向量 (feature hashing, 确定性无随机源)。
///
/// 所有文本共享同一全局基座, 三元组命中同维度累积票数。
/// 中心化余弦可精确消除基座偏置, 仅保留共享三元组的相关信号。
pub fn text_hypervector(text: &str, dim: usize) -> Vec<i8> {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let base = get_base_hv(dim);
    let mut acc: Vec<i32> = base.iter().map(|&x| x as i32).collect();
    for w in chars.windows(3) {
        let mut buf = Vec::with_capacity(12);
        for c in w {
            let mut enc = [0u8; 4];
            buf.extend_from_slice(c.encode_utf8(&mut enc).as_bytes());
        }
        let h = avalanche(fnv1a(&buf));
        let idx = (h % dim as u64) as usize;
        acc[idx] += if (h >> 63) == 1 { -2 } else { 2 };
    }
    acc.iter().map(|&x| if x >= 0 { 1 } else { -1 }).collect()
}

/// 双极超向量原始余弦 (归一化点积)。
///
/// 注意: 稀疏特征散列向量存在公共基座偏置 (未命中维对所有向量相同),
/// 原始余弦对短文本判别力有限。路由匹配请用 `text_jaccard` (精确 Jaccard)。
pub fn hv_similarity(a: &[i8], b: &[i8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: i32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as i32) * (*y as i32))
        .sum();
    dot as f64 / a.len() as f64
}

/// 字符三元组集合 (归一化: 小写、去空白), 用于精确 Jaccard 匹配。
pub fn text_trigrams(text: &str) -> std::collections::HashSet<u64> {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let mut set = std::collections::HashSet::new();
    for w in chars.windows(3) {
        let mut buf = Vec::with_capacity(12);
        for c in w {
            let mut enc = [0u8; 4];
            buf.extend_from_slice(c.encode_utf8(&mut enc).as_bytes());
        }
        set.insert(fnv1a(&buf));
    }
    set
}

/// 三元组 Jaccard 相似度 (精确、无几何偏置、短文本判别力最强)。
pub fn text_jaccard(a: &str, b: &str) -> f64 {
    let sa = text_trigrams(a);
    let sb = text_trigrams(b);
    if sa.is_empty() && sb.is_empty() {
        return 1.0;
    }
    let inter = sa.intersection(&sb).count();
    let union = sa.union(&sb).count();
    inter as f64 / union as f64
}

// 中心化余弦在共享基座下无效 — 保留签名兼容旧测试, 文档说明弃用。
#[deprecated(note = "共享基座下中心化无效, 请用 text_jaccard")]
pub fn hv_similarity_centered(a: &[i8], b: &[i8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let n = a.len() as f64;
    let ma = a.iter().map(|&x| x as f64).sum::<f64>() / n;
    let mb = b.iter().map(|&x| x as f64).sum::<f64>() / n;
    let (mut dot, mut na, mut nb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b.iter()) {
        let dx = *x as f64 - ma;
        let dy = *y as f64 - mb;
        dot += dx * dy;
        na += dx * dx;
        nb += dy * dy;
    }
    if na <= 1e-12 || nb <= 1e-12 {
        return 0.0;
    }
    (dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0)
}

// ─── 结晶流水线: 实例 → 规则 ────────────────────────────────────────────────

/// 词元类别 — 形状分组依据。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TokenClass {
    Number,
    Word,
    Punct,
    Mixed,
}

fn classify(tok: &str) -> TokenClass {
    let has_digit = tok.chars().any(|c| c.is_ascii_digit());
    let has_alpha = tok.chars().any(|c| c.is_alphabetic());
    let is_punct = !tok.is_empty() && tok.chars().all(|c| c.is_ascii_punctuation());
    if is_punct {
        TokenClass::Punct
    } else if has_digit && !has_alpha {
        TokenClass::Number
    } else if has_alpha && !has_digit {
        TokenClass::Word
    } else {
        TokenClass::Mixed
    }
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize_norm(norm: &str) -> Vec<&str> {
    norm.split(' ').collect()
}

/// 形状 = 词元类别序列。形状相同是对齐编译的前提。
fn shape_of(norm: &str) -> Vec<TokenClass> {
    tokenize_norm(norm).iter().map(|t| classify(t)).collect()
}

/// 从同形状实例组对齐编译模板: 全等位 → 字面量, 差异位 → 槽 s{n}。
/// 返回 (template, 每实例槽参数)。同形状保证 round-trip 无损还原。
pub fn compile_template(
    instances: &[String],
) -> Result<(String, Vec<BTreeMap<String, String>>), String> {
    let norms: Vec<String> = instances.iter().map(|s| normalize(s)).collect();
    if norms.is_empty() {
        return Err("空实例组".into());
    }
    let shape = shape_of(&norms[0]);
    if shape.is_empty() {
        return Err("空实例".into());
    }
    for n in &norms[1..] {
        if shape_of(n) != shape {
            return Err("实例形状不一致 (词元类别序列不同)".into());
        }
    }
    let toks: Vec<Vec<&str>> = norms.iter().map(|n| tokenize_norm(n)).collect();
    let n_tok = shape.len();
    let mut parts: Vec<String> = Vec::with_capacity(n_tok);
    let mut params_per_inst: Vec<BTreeMap<String, String>> = vec![BTreeMap::new(); toks.len()];
    let mut slot_idx = 0usize;
    for pos in 0..n_tok {
        let first = toks[0][pos];
        if toks.iter().all(|t| t[pos] == first) {
            parts.push(first.to_string());
        } else {
            let name = format!("s{}", slot_idx);
            slot_idx += 1;
            parts.push(format!("{{{{{name}}}}}"));
            for (i, t) in toks.iter().enumerate() {
                params_per_inst[i].insert(name.clone(), t[pos].to_string());
            }
        }
    }
    Ok((parts.join(" "), params_per_inst))
}

/// 结晶结果 — 编译+holdout 切分的完整产物。
#[derive(Debug)]
pub struct Crystallized {
    pub record: RuleRecord,
    /// train 组渲染参数 (供外部重放审计)
    pub train_params: Vec<BTreeMap<String, String>>,
    /// holdout 组渲染参数 (只测不编)
    pub holdout_params: Vec<BTreeMap<String, String>>,
}

/// 单组实例的完整结晶: compile → holdout 切分 → round-trip 验证。
///
/// holdout 策略: 组 ≥5 时末尾 ceil(20%) 实例只测不编 (防背答案式伪压缩)。
/// 保真对象是**空白归一化文本** — 高熵自由叙事不应结晶 (回退字节阶梯)。
pub fn crystallize_group(
    domain: &str,
    trigger_text: &str,
    instances: &[String],
    min_group: usize,
    provenance_keys: Vec<String>,
    id_salt: &str,
) -> Result<Crystallized, String> {
    if instances.len() < min_group.max(2) {
        return Err(format!(
            "实例数 {} < min_group {min_group}",
            instances.len()
        ));
    }
    let (template, all_params) = compile_template(instances)?;
    let split_at = if instances.len() >= 5 {
        instances.len() - (((instances.len() as f64) * 0.2).ceil() as usize).max(1)
    } else {
        instances.len()
    };
    let train: Vec<BTreeMap<String, String>> = all_params[..split_at].to_vec();
    let holdout: Vec<BTreeMap<String, String>> = all_params[split_at..].to_vec();
    // round-trip 验证: 同形状组应全过; 不过即内部缺陷, 落 Quarantined
    let mut train_pass = 0usize;
    for (i, p) in train.iter().enumerate() {
        if render_template(&template, p)? == normalize(&instances[i]) {
            train_pass += 1;
        }
    }
    let mut holdout_pass = 0usize;
    for (i, p) in holdout.iter().enumerate() {
        if render_template(&template, p)? == normalize(&instances[split_at + i]) {
            holdout_pass += 1;
        }
    }
    let now_ts = now();
    let mut rec = RuleRecord::new(
        format!("rule_{}_{:x}", domain, fnv1a(id_salt.as_bytes())),
        domain.to_string(),
        trigger_text.to_string(),
        template,
        now_ts,
    );
    rec.provenance_keys = provenance_keys;
    let train_rate = train_pass as f64 / train.len().max(1) as f64;
    // 空 holdout = vacuous pass (小组不因无留出而被误隔离)
    let holdout_rate = if holdout.is_empty() {
        1.0
    } else {
        holdout_pass as f64 / holdout.len() as f64
    };
    rec.confidence = 0.7 * train_rate + 0.3 * holdout_rate;
    rec.status = if train_rate == 1.0 && holdout_rate == 1.0 {
        RuleStatus::Active
    } else {
        RuleStatus::Quarantined
    };
    rec.last_verified_at = Some(now_ts);
    Ok(Crystallized {
        record: rec,
        train_params: train,
        holdout_params: holdout,
    })
}

// ─── KB 持久化 (namespace=rule) ─────────────────────────────────────────────

fn kv_set_retry(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    let mut last = String::new();
    for attempt in 0..3 {
        match kv_set(conn, NS_RULE, key, value) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = e;
                std::thread::sleep(std::time::Duration::from_millis(50 * (attempt as u64 + 1)));
            }
        }
    }
    Err(last)
}

fn rule_key(id: &str) -> String {
    format!("rule:{id}")
}

pub fn rule_put(conn: &Connection, rec: &RuleRecord) -> Result<(), String> {
    let json = serde_json::to_string(rec).map_err(|e| e.to_string())?;
    kv_set_retry(conn, &rule_key(&rec.id), &json)
}

pub fn rule_get(conn: &Connection, id: &str) -> Result<Option<RuleRecord>, String> {
    let Some(json) = kv_get(conn, NS_RULE, &rule_key(id))? else {
        return Ok(None);
    };
    serde_json::from_str(&json)
        .map(Some)
        .map_err(|e| format!("rule {id} 反序列化失败: {e}"))
}

pub fn rule_list(conn: &Connection) -> Result<Vec<RuleRecord>, String> {
    let mut out = Vec::new();
    for (_k, v) in kv_list(conn, NS_RULE)? {
        if let Ok(rec) = serde_json::from_str::<RuleRecord>(&v) {
            out.push(rec);
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub fn rule_delete(conn: &Connection, id: &str) -> Result<bool, String> {
    kv_delete(conn, NS_RULE, &rule_key(id))
}

// ─── 执行 (回忆=运行) ───────────────────────────────────────────────────────

/// 触发路由: query 超向量 vs 全部 Active 规则触发签名, 取最高余弦且 ≥ threshold。
pub fn rule_match(
    conn: &Connection,
    query: &str,
    threshold: f64,
) -> Result<Option<(RuleRecord, f64)>, String> {
    let mut best: Option<(RuleRecord, f64)> = None;
    for rec in rule_list(conn)? {
        if rec.status != RuleStatus::Active {
            continue;
        }
        let sim = text_jaccard(query, &rec.trigger_text);
        if sim >= threshold && best.as_ref().is_none_or(|(_, s)| sim > *s) {
            best = Some((rec, sim));
        }
    }
    Ok(best)
}

/// 执行规则: 校验状态/变量 → 渲染。计数器由调用方经 rule_put 回写。
pub fn rule_exec(rec: &RuleRecord, vars: &BTreeMap<String, String>) -> Result<String, String> {
    match rec.status {
        RuleStatus::Retired => Err("规则已退役".into()),
        RuleStatus::Quarantined => Err("规则处于隔离态 (drift 验证未过), 拒绝执行".into()),
        RuleStatus::Active => render_template(&rec.generator, vars),
    }
}

// ─── 验证 / GC ──────────────────────────────────────────────────────────────

/// 从模板+归一化实例反推槽参数: 字面量段精确对齐, 槽消费到下一字面量锚点。
pub fn extract_params(
    template: &str,
    instance_norm: &str,
) -> Result<BTreeMap<String, String>, String> {
    let segments = parse_template(template)?;
    let inst_tokens = tokenize_norm(instance_norm);
    let mut params = BTreeMap::new();
    let mut ti = 0usize;
    for (si, seg) in segments.iter().enumerate() {
        match seg {
            Segment::Lit(text) => {
                for tok in text.split_whitespace() {
                    if ti >= inst_tokens.len() || inst_tokens[ti] != tok {
                        return Err(format!(
                            "字面量不匹配: 期望 '{tok}' @token{ti} (实例共 {} token)",
                            inst_tokens.len()
                        ));
                    }
                    ti += 1;
                }
            }
            Segment::Var { name, .. } => {
                // 找下一个字面量段的首 token 作为右锚点
                let anchor = segments[si + 1..].iter().find_map(|s| match s {
                    Segment::Lit(t) => t.split_whitespace().next().map(str::to_string),
                    _ => None,
                });
                let end = match anchor {
                    Some(a) => {
                        let mut j = ti;
                        while j < inst_tokens.len() && inst_tokens[j] != a {
                            j += 1;
                        }
                        if j >= inst_tokens.len() {
                            return Err(format!("锚点 '{a}' 未在实例中找到"));
                        }
                        j
                    }
                    None => inst_tokens.len(),
                };
                params.insert(name.clone(), inst_tokens[ti..end].join(" "));
                ti = end;
            }
        }
    }
    if ti != inst_tokens.len() {
        return Err(format!(
            "实例尾部多余 {} token",
            inst_tokens.len() - ti
        ));
    }
    Ok(params)
}

/// 单规则重验证: 重放出处实例的 round-trip。
/// sources: provenance_key → 原始实例文本 (缺失源跳过, 全缺失判失败)。
pub fn verify_rule(
    conn: &Connection,
    id: &str,
    sources: &HashMap<String, String>,
) -> Result<Option<RuleRecord>, String> {
    let Some(mut rec) = rule_get(conn, id)? else {
        return Ok(None);
    };
    let now_ts = now();
    let mut pass = 0u64;
    let mut total = 0u64;
    for key in &rec.provenance_keys {
        let Some(text) = sources.get(key) else {
            continue;
        };
        total += 1;
        let norm = normalize(text);
        if let Ok(params) = extract_params(&rec.generator, &norm) {
            if render_template(&rec.generator, &params).map(|r| r == norm).unwrap_or(false) {
                pass += 1;
            }
        }
    }
    let rate = pass as f64 / total.max(1) as f64;
    rec.last_verified_at = Some(now_ts);
    rec.updated_at = now_ts;
    rec.version += 1;
    rec.status = if total > 0 && rate == 1.0 {
        RuleStatus::Active
    } else {
        rec.fail_count += 1;
        RuleStatus::Quarantined
    };
    rule_put(conn, &rec)?;
    Ok(Some(rec))
}

/// GC 报告。
#[derive(Debug, Default, serde::Serialize)]
pub struct GcReport {
    pub deleted: Vec<String>,
    pub retired: Vec<String>,
    pub kept: usize,
}

/// Dark Forest GC:
/// - Retired → 直接删除
/// - Quarantined 且 fail_count ≥ 3 → 删除 (连续验证失败即死代码)
/// - Active 且 exec_count == 0 且闲置超过 max_idle_days → 降级 Retired
pub fn gc_rules(conn: &Connection, max_idle_days: u64) -> Result<GcReport, String> {
    let now_ts = now();
    let mut report = GcReport::default();
    for rec in rule_list(conn)? {
        match rec.status {
            RuleStatus::Retired => {
                rule_delete(conn, &rec.id)?;
                report.deleted.push(rec.id.clone());
            }
            RuleStatus::Quarantined if rec.fail_count >= 3 => {
                rule_delete(conn, &rec.id)?;
                report.deleted.push(rec.id.clone());
            }
            RuleStatus::Active
                if rec.exec_count == 0
                    && now_ts - rec.created_at > (max_idle_days as i64 * 86400) =>
            {
                let mut r = rec.clone();
                r.status = RuleStatus::Retired;
                r.updated_at = now_ts;
                rule_put(conn, &r)?;
                report.retired.push(rec.id.clone());
            }
            _ => report.kept += 1,
        }
    }
    Ok(report)
}

// ─── 经验扫描结晶 (detect: experience namespace → rule namespace) ───────────

/// 扫描配置。
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 最小组实例数
    pub min_group: usize,
    /// 限定域 (None = 全域)
    pub domain: Option<String>,
    /// 单次扫描最大实例数 (后台循环防阻塞)
    pub limit: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self { min_group: 3, domain: None, limit: 200 }
    }
}

/// 扫描报告。
#[derive(Debug, Default, serde::Serialize)]
pub struct ScanReport {
    pub scanned_instances: usize,
    pub candidate_groups: usize,
    pub rules_created: Vec<String>,
    pub rules_quarantined: Vec<String>,
    pub skipped_existing: usize,
    /// 结晶失败组诊断 (可见性链: 静默跳过会掩盖系统性形状漂移)
    pub errors: Vec<String>,
}

/// 从 experience namespace 扫描实例并结晶 (detect→compile→verify→replace)。
///
/// 实例契约: kv_store namespace='experience' 的 JSON 条目, 含 `domain` 与
/// `content` 字段 (neotrix-experience absorb 落盘格式)。幂等性: 任一源 key
/// 已被现有规则的 provenance 覆盖 → 该组跳过 (不重复结晶)。
pub fn crystallize_scan(conn: &Connection, cfg: &ScanConfig) -> Result<ScanReport, String> {
    let mut report = ScanReport::default();
    let rows = kv_list(conn, "experience")?;
    // 已覆盖源 key (幂等)
    let covered: HashSet<String> = rule_list(conn)?
        .into_iter()
        .flat_map(|r| r.provenance_keys)
        .collect();
    // 按 (domain, 形状签名) 分组 — detect 阶段
    let mut groups: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    for (key, value) in rows.iter().take(cfg.limit) {
        if covered.contains(key) {
            report.skipped_existing += 1;
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(value) else {
            continue;
        };
        if v.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false) {
            continue; // 已蒸馏降权的条目不再结晶
        }
        let Some(content) = v.get("content").and_then(|c| c.as_str()) else {
            continue;
        };
        let domain = v.get("domain").and_then(|d| d.as_str()).unwrap_or("unknown");
        if let Some(want) = &cfg.domain {
            if domain != want {
                continue;
            }
        }
        report.scanned_instances += 1;
        let norm = normalize(content);
        if norm.is_empty() || !norm.contains(' ') {
            continue; // 单 token 无结构可结晶 (熵地板)
        }
        let shape_key = shape_of(&norm)
            .iter()
            .map(|c| format!("{c:?}"))
            .collect::<Vec<_>>()
            .join(",");
        groups
            .entry((domain.to_string(), shape_key))
            .or_default()
            .push((key.clone(), norm));
    }
    // 逐组编译验证 (compile→verify→replace)
    let mut id_counter = 0u64;
    for ((domain, _shape), mut members) in groups {
        if members.len() < cfg.min_group.max(2) {
            continue;
        }
        members.sort();
        let keys: Vec<String> = members.iter().map(|(k, _)| k.clone()).collect();
        let texts: Vec<String> = members.into_iter().map(|(_, t)| t).collect();
        report.candidate_groups += 1;
        let trigger = trigger_from_group(&domain, &texts[0]);
        id_counter += 1;
        match crystallize_group(
            &domain,
            &trigger,
            &texts,
            cfg.min_group,
            keys,
            &format!("{domain}:{trigger}:{id_counter}"),
        ) {
            Ok(out) => {
                let quarantined = out.record.status == RuleStatus::Quarantined;
                rule_put(conn, &out.record)?;
                if quarantined {
                    report.rules_quarantined.push(out.record.id);
                } else {
                    report.rules_created.push(out.record.id);
                }
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("组 [{domain}] 结晶失败: {e}"));
                continue;
            }
        }
    }
    Ok(report)
}

/// 组触发描述: 域 + 首实例骨架预览 (字面量为主, 槽位折叠为 ◇)。
fn trigger_from_group(domain: &str, sample_norm: &str) -> String {
    let toks: Vec<String> = tokenize_norm(sample_norm)
        .iter()
        .map(|t| {
            if classify(t) == TokenClass::Number {
                "◇".to_string()
            } else {
                t.to_string()
            }
        })
        .collect();
    let preview: Vec<String> = toks.iter().take(8).cloned().collect();
    format!("{domain} ▸ {}", preview.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("mem conn");
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).expect("schema");
        conn
    }

    #[test]
    fn test_template_slots_ordered_dedup() {
        assert_eq!(
            template_slots("{{a}} x {{b}} y {{a}} {{ c }}").unwrap(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_template_unclosed_brace_fails_loud() {
        assert!(parse_template("hi {{oops").is_err());
        assert!(parse_template("hi {{}}").is_err());
    }

    #[test]
    fn test_render_transforms_and_missing_var_fails_loud() {
        let mut vars = BTreeMap::new();
        vars.insert("name".into(), "  neo ".into());
        assert_eq!(render_template("hi {{name|trim}}!", &vars).unwrap(), "hi neo!");
        // 链式变换: trim → upper
        assert_eq!(render_template("{{name|trim|upper}}", &vars).unwrap(), "NEO");
        assert_eq!(render_template("{{name|upper}}", &vars).unwrap(), "  NEO ");
        let err = render_template("hi {{missing}}", &vars).unwrap_err();
        assert!(err.contains("缺少变量"), "got: {err}");
    }

    #[test]
    fn test_render_rejects_unknown_transform() {
        let mut vars = BTreeMap::new();
        vars.insert("a".into(), "x".into());
        let err = render_template("{{a|exec}}", &vars).unwrap_err();
        assert!(err.contains("未知变换"), "transform whitelist must reject injection");
    }

    #[test]
    fn test_render_preserves_utf8_multibyte() {
        let mut vars = BTreeMap::new();
        vars.insert("域".into(), "核心".into());
        assert_eq!(render_template("NT-{{域}} 记忆层 ✓", &vars).unwrap(), "NT-核心 记忆层 ✓");
    }

    #[test]
    fn test_hv_deterministic_and_similar_text_scores_higher() {
        let a = text_hypervector("cargo build 失败 重试", HV_DIM);
        let b = text_hypervector("cargo build 失败 重试", HV_DIM);
        assert_eq!(hv_similarity(&a, &b), 1.0);
        let near = text_hypervector("cargo build 失败 回滚", HV_DIM);
        let far = text_hypervector("量子引力 波函数 坍缩", HV_DIM);
        assert!(hv_similarity(&a, &near) > hv_similarity(&a, &far));
    }

    #[test]
    fn test_compile_alignment_literals_and_slots() {
        let insts = vec![
            "部署 v1 到 生产 环境".to_string(),
            "部署 v2 到 生产 环境".to_string(),
            "部署 v3 到 测试 环境".to_string(),
        ];
        let (tpl, params) = compile_template(&insts).unwrap();
        assert_eq!(tpl, "部署 {{s0}} 到 {{s1}} 环境");
        assert_eq!(params[0]["s0"], "v1");
        assert_eq!(params[2]["s1"], "测试");
        for (inst, p) in insts.iter().zip(&params) {
            assert_eq!(render_template(&tpl, p).unwrap(), normalize(inst));
        }
    }

    #[test]
    fn test_compile_shape_mismatch_and_empty_rejected() {
        assert!(compile_template(&["abc def".into(), "123 456 789".into()]).is_err());
        assert!(compile_template(&[]).is_err());
    }

    #[test]
    fn test_crystallize_active_on_clean_group_with_holdout() {
        let insts: Vec<String> = (0..5)
            .map(|i| format!("合并 分支 feature/{i} 到 main"))
            .collect();
        let out =
            crystallize_group("NT-ACT", "git 合并分支操作", &insts, 3, vec!["k1".to_string(); 5], "salt")
                .unwrap();
        assert_eq!(out.record.status, RuleStatus::Active);
        assert!(out.record.confidence > 0.99);
        assert_eq!(out.record.provenance_keys.len(), 5);
        assert_eq!(out.holdout_params.len(), 1, "≥5 实例必须留出 holdout");
        assert_eq!(out.train_params.len(), 4);
    }

    #[test]
    fn test_crystallize_min_group_enforced() {
        let insts = vec!["a 1".to_string()];
        assert!(crystallize_group("d", "t", &insts, 3, vec![], "s").is_err());
    }

    #[test]
    fn test_store_roundtrip_and_list_ordering() {
        let conn = mem_conn();
        let mut rec = RuleRecord::new(
            "rule_x".into(),
            "d".into(),
            "触发 描述".into(),
            "值 {{s0}}".into(),
            now(),
        );
        rec.status = RuleStatus::Active;
        rule_put(&conn, &rec).unwrap();
        let got = rule_get(&conn, "rule_x").unwrap().unwrap();
        assert_eq!(got.trigger_text, "触发 描述");
        assert_eq!(got.hv_dim, HV_DIM);
        assert_eq!(rule_list(&conn).unwrap().len(), 1);
        assert!(rule_delete(&conn, "rule_x").unwrap());
        assert!(rule_get(&conn, "rule_x").unwrap().is_none());
    }

    #[test]
    fn test_match_routes_to_active_only_above_threshold() {
        let conn = mem_conn();
        let mut hit = RuleRecord::new(
            "hit".into(),
            "d".into(),
            "合并 git 分支 操作".into(),
            "{{s0}}".into(),
            now(),
        );
        hit.status = RuleStatus::Active;
        let mut miss = hit.clone();
        miss.id = "miss".into();
        miss.status = RuleStatus::Quarantined;
        rule_put(&conn, &hit).unwrap();
        rule_put(&conn, &miss).unwrap();
        let m = rule_match(&conn, "合并 git 分支", DEFAULT_MATCH_THRESHOLD).unwrap();
        assert!(m.is_some());
        assert_eq!(m.unwrap().0.id, "hit");
        let none = rule_match(&conn, "蛋白质 折叠 预测", 0.9).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_exec_status_gating() {
        let rec = RuleRecord::new("q".into(), "d".into(), "t".into(), "{{s0}}!".into(), now());
        let mut vars = BTreeMap::new();
        vars.insert("s0".into(), "x".into());
        let mut q = rec.clone();
        q.status = RuleStatus::Retired;
        assert!(rule_exec(&q, &vars).is_err());
        q.status = RuleStatus::Quarantined;
        assert!(rule_exec(&q, &vars).is_err());
        q.status = RuleStatus::Active;
        assert_eq!(rule_exec(&q, &vars).unwrap(), "x!");
    }

    #[test]
    fn test_extract_params_multi_token_slots() {
        let tpl = "把 {{src}} 迁移到 {{dst}} 完成";
        let p = extract_params(tpl, "把 a/b/c 目录 迁移到 目标 位置 x 完成").unwrap();
        assert_eq!(p["src"], "a/b/c 目录");
        assert_eq!(p["dst"], "目标 位置 x");
        assert_eq!(render_template(tpl, &p).unwrap(), "把 a/b/c 目录 迁移到 目标 位置 x 完成");
    }

    #[test]
    fn test_verify_rule_drift_flips_to_quarantined() {
        let conn = mem_conn();
        let out = crystallize_group(
            "d",
            "部署 服务",
            &[
                "部署 v1 到 生产 环境".to_string(),
                "部署 v2 到 生产 环境".to_string(),
                "部署 v3 到 生产 环境".to_string(),
            ],
            3,
            vec!["k1".into()],
            "salt",
        )
        .unwrap();
        assert_eq!(out.record.status, RuleStatus::Active);
        rule_put(&conn, &out.record).unwrap();

        // 未漂移: 全过保持 Active
        let mut sources = HashMap::new();
        sources.insert("k1".to_string(), "部署 v1 到 生产 环境".to_string());
        let rec = verify_rule(&conn, &out.record.id, &sources).unwrap().unwrap();
        assert_eq!(rec.status, RuleStatus::Active);

        // 漂移: 字面量改变 → Quarantined + fail_count 递增
        sources.insert("k1".to_string(), "部署 v9 到 生产 磁盘".to_string());
        let rec = verify_rule(&conn, &out.record.id, &sources).unwrap().unwrap();
        assert_eq!(rec.status, RuleStatus::Quarantined);
        assert_eq!(rec.fail_count, 1);
    }

    #[test]
    fn test_gc_dark_forest_lifecycle() {
        let conn = mem_conn();
        let base = now();
        let mut r1 = RuleRecord::new("dead".into(), "d".into(), "t1".into(), "x".into(), base);
        r1.status = RuleStatus::Retired;
        let mut r2 = RuleRecord::new("sick".into(), "d".into(), "t2".into(), "y".into(), base + 1);
        r2.status = RuleStatus::Quarantined;
        r2.fail_count = 3;
        let mut r3 = RuleRecord::new(
            "idle".into(),
            "d".into(),
            "t3".into(),
            "z".into(),
            base + 2,
        );
        r3.status = RuleStatus::Active;
        r3.created_at = base - 40 * 86400;
        rule_put(&conn, &r1).unwrap();
        rule_put(&conn, &r2).unwrap();
        rule_put(&conn, &r3).unwrap();
        let rep = gc_rules(&conn, 30).unwrap();
        assert!(rep.deleted.contains(&"dead".to_string()));
        assert!(rep.deleted.contains(&"sick".to_string()));
        assert_eq!(rep.retired, vec!["idle".to_string()]);
        assert_eq!(
            rule_get(&conn, "idle").unwrap().unwrap().status,
            RuleStatus::Retired
        );
        // 二次 GC: retired idle 也被删 (Dark Forest 收敛)
        let rep2 = gc_rules(&conn, 30).unwrap();
        assert!(rep2.deleted.contains(&"idle".to_string()));
    }

    fn seed_experience(conn: &Connection, key: &str, domain: &str, content: &str) {
        let entry = serde_json::json!({
            "schema_version": 1,
            "type": "branch",
            "domain": domain,
            "content": content,
        });
        crate::core::nt_core_kb_primitives::kv_set(conn, "experience", key, &entry.to_string())
            .unwrap();
    }

    #[test]
    fn test_crystallize_scan_detects_groups_and_idempotent() {
        let conn = mem_conn();
        for i in 0..4 {
            seed_experience(
                &conn,
                &format!("branch_a_{i}"),
                "NT-ACT",
                &format!("回滚 发布 release/{i} 到 上一个 版本"),
            );
        }
        // 噪声: 单 token 与孤立条目不应结晶
        seed_experience(&conn, "branch_noise", "NT-ACT", "hello");
        let rep = crystallize_scan(
            &conn,
            &ScanConfig { min_group: 3, domain: None, limit: 200 },
        )
        .unwrap();
        assert_eq!(rep.scanned_instances, 5);
        assert_eq!(rep.candidate_groups, 1);
        assert_eq!(rep.rules_created.len(), 1);
        let rules = rule_list(&conn).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].status, RuleStatus::Active);
        assert!(rules[0].trigger_text.starts_with("NT-ACT ▸"));

        // 幂等: 二次扫描已覆盖 key 跳过; 未覆盖噪声条目重扫但不成组
        let rep2 = crystallize_scan(
            &conn,
            &ScanConfig { min_group: 3, domain: None, limit: 200 },
        )
        .unwrap();
        assert_eq!(rep2.rules_created.len(), 0);
        assert_eq!(rep2.skipped_existing, 4, "已覆盖源 key 必须跳过");
        assert_eq!(rule_list(&conn).unwrap().len(), 1);
    }

    #[test]
    fn test_crystallize_scan_domain_filter() {
        let conn = mem_conn();
        for i in 0..3 {
            seed_experience(
                &conn,
                &format!("branch_b_{i}"),
                "NT-MIND",
                &format!("蒸馏 经验 batch/{i} 成 模式"),
            );
        }
        let rep = crystallize_scan(
            &conn,
            &ScanConfig { min_group: 3, domain: Some("NT-CORE".into()), limit: 200 },
        )
        .unwrap();
        assert_eq!(rep.candidate_groups, 0, "域过滤必须挡住 NT-MIND 组");
        let rep2 = crystallize_scan(
            &conn,
            &ScanConfig { min_group: 3, domain: Some("NT-MIND".into()), limit: 200 },
        )
        .unwrap();
        assert_eq!(rep2.rules_created.len(), 1);
    }
}
