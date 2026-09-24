//! exp_query — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use super::{DAY, NS};
use super::exp_concept::{co_full, concept_from_branch, concept_hash, extract_concepts, load_concept, text_doc_vector};
use super::exp_store::{ensure_hub, is_stale, json_truthy, kv_get, kv_set, load_json, refresh_hub_metrics, save_hub, scan_values};
use super::exp_util::{cycle_opt, en_stop, fmt_ts, now_ts, parse_cycle, py_dict_repr, py_float_repr, truncate};
use std::cmp::Ordering;
use neotrix::l2_perception::nt_core_hcube::ghrr_vsa::ghrr_similarity;

/// 突触联想检索: 输入词 → 命中概念神经元 → 沿突触扩散到分支(1阶, 主结果) →
/// Hebb 共现多跳扩散到关联概念 (BFS, 每跳衰减, 记忆大脑设计 §3.3 图信号) →
/// 关联概念分支获得加权分数 (作为相关推荐)。
pub(crate) fn neural_associative(conn: &Connection, kws: &[String], hebb: bool) -> Option<Vec<(String, f64, i64)>> {
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
pub(crate) struct QueryResult {
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


pub(crate) fn cmd_query(conn: &Connection, kw: &str, ty: Option<&str>, domain: Option<&str>, limit: usize, no_hebb: bool, json: bool, semantic: bool, include_distilled: bool) -> usize {
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


pub(crate) fn cmd_list(conn: &Connection, ty: Option<&str>, domain: Option<&str>, cycle: Option<&str>) {
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


pub(crate) fn cmd_stale(conn: &Connection, domain: Option<&str>) {
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


pub(crate) fn cmd_hub(conn: &Connection) {
    let mut hub = ensure_hub(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("{}", serde_json::to_string_pretty(&hub).expect("hub JSON serialization"));
}


pub(crate) fn cmd_route(conn: &Connection, kw: &str, branch: &str) {
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


pub(crate) fn cmd_route_verify(conn: &Connection, clean: bool) {
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


pub(crate) fn cmd_neuron(conn: &Connection, term: &str, exact: bool) {
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
    let c = c.expect("concept found");
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


pub(crate) fn cmd_backfill(conn: &Connection) {
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


pub(crate) fn cmd_prune(conn: &Connection, extra_stop: &[String], stale_isolated: bool) {
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


pub(crate) fn cmd_dedup(conn: &Connection, dry_run: bool) {
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
