//! exp_absorb — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use super::{DAY, DOMAINS, NS, OBS_BUFFER_ACTIVATION, OBS_BUFFER_RATIO, OBS_TOKEN_BUDGET, REF_TOKEN_BUDGET, SCHEMA_VERSION, TYPES, VERIFY_DEFAULT_DAYS};
use super::exp_concept::{builtin_extractors, concept_from_branch, extract_concepts, hebb_cooccurrence, run_extractors, text_doc_vector};
use super::exp_distill::{cmd_distill, immediate_promote_entries};
use super::exp_store::{ensure_hub, field_observe, field_solve, json_truthy, kv_get, kv_set, kv_stage, norm_verify_by, refresh_hub_metrics, save_hub, scan_values};
use super::exp_util::{cycle_opt, estimate_tokens, now_ts, uuid_hex};
use neotrix::l2_perception::nt_core_hcube::ghrr_vsa::ghrr_similarity;
use neotrix::l0_substrate::nt_core_math::normalize_url;
use neotrix::l5_cognition::nt_mind::foundation::guardian::{MapeGate, MapeGateConfig, MetricEval};
use sha2::{Digest, Sha256};

pub(crate) fn cmd_snapshot(conn: &Connection, cycle: &str, task: &str, domain: &str) {
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
    kv_stage(conn, NS, &format!("snapshot_{}", sid), &snap.to_string());
    field_solve(conn);
    field_observe(conn);
    println!("[snapshot] {} (cycle={})", sid, cycle);
}


pub(crate) fn cmd_close(conn: &Connection, cycle: &str) {
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


pub(crate) fn validate_entry(e: &Value) -> Vec<String> {
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


pub(crate) fn confidence_of(entry: &Value) -> f64 {
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


pub(crate) fn cmd_absorb(conn: &mut Connection, input: &str) {
    let mut hub = ensure_hub(conn);
    // 输入直通: "-" 读 stdin (与 absorb-node 对称, 支持直写 KB 跳过本地文件),
    // 否则按文件路径读取。
    let raw = if input == "-" {
        let mut buf = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf) {
            eprintln!("[absorb] 读取 stdin 失败: {e}");
            std::process::exit(1);
        }
        buf
    } else {
        match std::fs::read_to_string(input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[absorb] 读取 {} 失败: {e}", input);
                std::process::exit(1);
            }
        }
    };
    let session: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[absorb] session.json 不是合法 JSON: {e}");
            std::process::exit(1);
        }
    };

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
    // 已实际落盘的高信号条目 (供即时 promote, 拒绝冗余/质量门过滤噪声)
    let mut written_high_signal: Vec<Value> = Vec::new();
    // P0-1 观察阶段收集: (branch_key, entry_json, domain)
    let mut written_entries: Vec<(String, Value, String)> = Vec::new();
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
            // 即时 promote 依赖的信号字段 (importance ≥ 0.6 → 立即升维)
            "importance": raw_entry.get("importance").and_then(|x| x.as_f64())
                .unwrap_or(0.5),
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
        kv_stage(conn, NS, &key, &e.to_string());
        // Hebb 共现突触: 同分支概念两两强化关联 (fire together, wire together)
        hebb_cooccurrence(conn, &chs);
        // 更新 hub cycle 索引
        let cycles = hub["hub"]["cycles"].as_object_mut().expect("JSON object");
        let cmeta = cycles
            .entry(cycle.clone())
            .or_insert_with(|| json!({"count": 0, "types": [], "domains": []}));
        let cmeta = cmeta.as_object_mut().expect("JSON object");
        let count = cmeta.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
        cmeta.insert("count".to_string(), json!(count + 1));
        let types = cmeta.entry("types".to_string()).or_insert_with(|| json!([]));
        let ty = e.get("type").cloned().unwrap_or_else(|| json!("insight"));
        if let Some(arr) = types.as_array() {
            if !arr.contains(&ty) {
                if let Some(arr) = types.as_array_mut() {
                    arr.push(ty);
                }
            }
        }
        let domains = cmeta
            .entry("domains".to_string())
            .or_insert_with(|| json!([]));
        let dom = e
            .get("domain")
            .cloned()
            .unwrap_or_else(|| json!("unknown"));
        if let Some(arr) = domains.as_array() {
            if !arr.contains(&dom) {
                if let Some(arr) = domains.as_array_mut() {
                    arr.push(dom);
                }
            }
        }
        written += 1;
        if e.get("importance").and_then(|x| x.as_f64()).unwrap_or(0.0) >= 0.6 {
            written_high_signal.push(e.clone());
        }
        // 收集写入成功的条目用于观察阶段 (P0-1 Mastra OM 吸收)
        let dom = e.get("domain").and_then(|d| d.as_str()).unwrap_or("unknown").to_string();
        let entry_for_obs = e.clone();
        written_entries.push((key.clone(), entry_for_obs, dom));
        audit_log.push(json!({
            "idx": i, "decision": "written", "key": key,
            "content_hash": content_hash, "ts": ts,
        }));
    }

    // D20: 吸收审计轨迹 → kv_store `audit` 命名空间 (Phase 1 KB 直写迁移)。
    // 替代本地 audit_*.jsonl 文件写入 — 每条 decision 一条记录, 附 session_id/cycle,
    // 可经 `query`/`list` 检索; 历史 audit_*.jsonl 文件保留只读 (遗产), 不再新写。
    for rec in &audit_log {
        let mut r = rec.clone();
        r["session_id"] = json!(sid);
        r["cycle"] = json!(cycle);
        let idx = r.get("idx").and_then(|x| x.as_u64()).unwrap_or(0);
        let key = format!(
            "audit_{}_{}_{}",
            cycle.replace(['/', '\\', ' '], "_"),
            sid.replace(['/', '\\', ' ', ':'], "_"),
            idx
        );
        kv_stage(conn, "audit", &key, &r.to_string());
    }
    // W4 一批一解: 先求解分支+审计批次, 再重建 hub 指标 — refresh_hub_metrics
    // 全量重读 kv_store, 若暂存未落账会被 save_hub 持久化少计的指标。
    field_solve(conn);
    refresh_hub_metrics(conn, &mut hub);
    save_hub(conn, &hub);
    println!("[absorb] {} entries from {} (cycle={})", written, sid, cycle);
    // 即时 promote (蒸馏延迟消除): 对已实际写入且 importance ≥ 0.6 的高信号经验立即
    // 路由并写入 experience_targets, 不等 distill 批处理 — 高信号发现当天生效,
    // 而不是等未蒸馏分支累积超阈值才被批量升维 (cycle 1191 教训: 接线完整性
    // 门禁当时靠用户提问才被强化, 说明即时生效比批处理更可靠)。
    // 只取 written_high_signal (已落盘 + 高信号), 冗余/质量门拒绝的条目不提升。
    if !written_high_signal.is_empty() {
        let n = immediate_promote_entries(&written_high_signal);
        if n > 0 {
            println!("[absorb] 即时 promote {} 条高信号经验 → experience_targets (不等 distill)", n);
        }
    }
    // 自动消退蒸馏: 吸收后若未蒸馏分支累积超阈值, 自动触发 distill
    // (经验无限追加 → 维度膨胀 → 自动收敛为能力模式, "始终处于最优解状态")
    auto_distill_if_over_threshold(conn, &mut hub);

    // ── P0-1 观察阶段 (Mastra OM 吸收, 2026-08-24) ──
    // 将已写入的 branch_ 条目按 token 预算分组为观察块,
    // 每块写一个 observation 类型条目, 含 provenance_range 溯源回原始 branch_ keys,
    // 并通过 Extractor 管线 (zod schema + 失败隔离 + on_extracted 钩子) 写 extracted 字段。
    if !written_entries.is_empty() {
        let extractors = builtin_extractors();
        let mut obs_chunks: Vec<Vec<(String, Value, String)>> = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0usize;
        for (bkey, entry, dom) in &written_entries {
            let content = entry.get("content").and_then(|c| c.as_str()).unwrap_or("");
            let tok = estimate_tokens(content);
            if current_tokens + tok > OBS_TOKEN_BUDGET && !current_chunk.is_empty() {
                obs_chunks.push(current_chunk);
                current_chunk = Vec::new();
                current_tokens = 0;
            }
            current_tokens += tok;
            current_chunk.push((bkey.clone(), entry.clone(), dom.clone()));
        }
        if !current_chunk.is_empty() {
            obs_chunks.push(current_chunk);
        }

        // 写入观察条目 + Extractor 管线
        let now = now_ts();
        for (chunk_idx, chunk) in obs_chunks.iter().enumerate() {
            let mut obs_content = String::new();
            let mut source_keys = Vec::new();
            let mut total_tokens = 0usize;
            let mut domains: BTreeSet<String> = BTreeSet::new();
            let mut first_ts = now;
            let mut last_ts = now;
            for (bkey, entry, dom) in chunk {
                let content = entry.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if !obs_content.is_empty() {
                    obs_content.push_str("\n---\n");
                }
                obs_content.push_str(content);
                source_keys.push(bkey.clone());
                total_tokens += estimate_tokens(content);
                domains.insert(dom.clone());
                let ts = entry.get("ts").and_then(|t| t.as_i64()).unwrap_or(now);
                if ts < first_ts { first_ts = ts; }
                if ts > last_ts { last_ts = ts; }
            }

            // Extractor 管线: zod schema 驱动 + 失败隔离 + on_extracted 钩子
            let extracted = run_extractors(&obs_content, &extractors);

            let obs_entry = json!({
                "schema_version": SCHEMA_VERSION,
                "type": "observation",
                "session_id": sid,
                "cycle": cycle,
                "ts": now,
                "domain": "NT-MEMORY",
                "content": obs_content,
                "evidence": format!("provenance: {} branch keys", source_keys.len()),
                "source": "observation",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
                "confidence": 0.7,
                "importance": 0.5,
                "provenance_range": {
                    "start_id": first_ts,
                    "end_id": last_ts,
                    "raw_source_keys": source_keys,
                },
                "extracted": extracted,
                "observation": {
                    "buffered": true,
                    "token_budget": OBS_TOKEN_BUDGET,
                    "buffer_ratio": OBS_BUFFER_RATIO,
                    "buffer_activation": OBS_BUFFER_ACTIVATION,
                    "activate_after_idle": "auto",
                    "activate_on_provider_change": true,
                    "extractors": ["current_task", "suggested_response", "thread_title"],
                },
                "reflection": Value::Null,
                "concepts": Value::Null,
            });
            let obs_key = format!("obs_{}_{}_{}", cycle, chunk_idx, uuid_hex(6));
            kv_stage(conn, NS, &obs_key, &obs_entry.to_string());
            // 更新 hub
            let cycles = hub["hub"]["cycles"].as_object_mut().expect("JSON object");
            let cmeta = cycles.entry(cycle.clone()).or_insert_with(|| json!({"count": 0, "types": [], "domains": []}));
            let cmeta = cmeta.as_object_mut().expect("JSON object");
            let count = cmeta.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
            cmeta.insert("count".to_string(), json!(count + 1));
            let types = cmeta.entry("types".to_string()).or_insert_with(|| json!([]));
            if let Some(arr) = types.as_array() {
                if !arr.contains(&json!("observation")) {
                    if let Some(arr) = types.as_array_mut() {
                        arr.push(json!("observation"));
                    }
                }
            }
            let doms = cmeta.entry("domains".to_string()).or_insert_with(|| json!([]));
            if let Some(arr) = doms.as_array() {
                if !arr.contains(&json!("NT-MEMORY")) {
                    if let Some(arr) = doms.as_array_mut() {
                        arr.push(json!("NT-MEMORY"));
                    }
                }
            }
            println!("[absorb] observation chunk {} written: {} tokens, {} sources, extracted={}", chunk_idx, total_tokens, chunk.len(), extracted.as_object().map(|o| o.len()).unwrap_or(0));
        }
    }

    // ── P0-1 反射阶段触发检查 (异步建议: observe 写入后可立即触发 reflect) ──
    // 这里仅打印建议; 实际生产由后台循环或手动 `neotrix-experience reflect` 触发
    if !written_entries.is_empty() {
        let total_obs_tokens: usize = written_entries.iter().map(|(_, e, _)| {
            e.get("content").and_then(|c| c.as_str()).map(|s| estimate_tokens(s)).unwrap_or(0)
        }).sum();
        if total_obs_tokens > REF_TOKEN_BUDGET {
            println!("[absorb] hint: observation tokens {} > REF_TOKEN_BUDGET ({}), 建议运行 `neotrix-experience reflect --domain NT-MEMORY` 触发重写", total_obs_tokens, REF_TOKEN_BUDGET);
        }
    }

    // W4: 观察批次收尾求解 + 场版本观测
    field_solve(conn);
    field_observe(conn);
}

// ────────────────────────────────────────────────────────────────
// P0-2 G3+G8: 经验反馈环 — reuse 结果记录 → MapeGate 多指标 burn-in 门
// (SEA/SimpleMem absorb)。feedback success/failure 反向下调 confidence,
// 触发主动回滚 (status=failing, confidence 减半) 或晋升 (status=stable)。
// ────────────────────────────────────────────────────────────────


pub(crate) fn cmd_feedback(conn: &mut Connection, key: &str, outcome: &str) {
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
pub(crate) const AUTO_DISTILL_THRESHOLD: usize = 60;


pub(crate) fn auto_distill_if_over_threshold(conn: &mut Connection, hub: &mut Value) {
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


pub(crate) fn cmd_absorb_node(conn: &Connection, input: &str, dry_run: bool, apply_capability: bool) {
    // 1. 读取输入 (文件或 stdin)
    let raw = if input == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .expect("read stdin");
        buf
    } else {
        std::fs::read_to_string(input).unwrap_or_else(|e| {
            eprintln!("[absorb-node] ✗ failed to read {}: {}", input, e);
            std::process::exit(1);
        })
    };
    let v: Value = serde_json::from_str(&raw).expect("node json is valid JSON");
    absorb_node_value(conn, v, dry_run, apply_capability);
}

/// 直接从 URL 生成 `nt_self_forge` 的候选节点并走同一条 absorb_node 路径。
///
/// 这是 `absorb-node --url ...` 的生产入口：候选 JSON 与 `nt_self_forge::forge_from_url`
/// 写入的 payload 同构，避免两条「自我锻造」JSON 口径漂移。
pub(crate) fn cmd_absorb_node_from_url(
    conn: &Connection,
    url: &str,
    dry_run: bool,
    apply_capability: bool,
) {
    let v = match neotrix_neobot::nt_self_forge::absorb_node_candidate(url) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[absorb-node] ✗ self_forge candidate: {e}");
            std::process::exit(1);
        }
    };
    absorb_node_value(conn, v, dry_run, apply_capability);
}

fn absorb_node_value(conn: &Connection, v: Value, dry_run: bool, apply_capability: bool) {

    // 2. 归一化为节点数组 (单对象 → [对象])
    let nodes: Vec<Value> = match v {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![v],
        _ => {
            eprintln!("[absorb-node] ✗ input must be a JSON object or array of objects");
            std::process::exit(1);
        }
    };

    // 3. 逐个处理
    let mut inserted = 0usize;
    let mut duplicated = 0usize;
    let mut mapped = 0usize;
    let now = now_ts();
    // FIX: 使用 main 传入的 conn (已 open_kb 初始化), 不再二次 open KnowledgeBase
    // (二次 open 会 flock + schema_initialize 造成死锁/长时间阻塞)。
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

        // 7. 直接用 conn 写入 (nodes + FTS + 域枢纽边), 避免 KnowledgeBase 二次 open
        let node_id = {
            let _ts = now;
            let id = format!("batch_{}_{}", now, {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut h = DefaultHasher::new();
                norm_url.hash(&mut h);
                format!("{:x}", h.finish())
            });

            // 幂等: URL 去重
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM nodes WHERE url=?1 LIMIT 1",
                    params![norm_url],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if exists {
                duplicated += 1;
                let nid: String = conn
                    .query_row(
                        "SELECT id FROM nodes WHERE url=?1 LIMIT 1",
                        params![norm_url],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                println!(
                    "[absorb-node] duplicate #{}: {} (id={})",
                    i, norm_url, nid
                );
                continue;
            }

            // 插入 nodes + FTS
            let concepts = {
                let text = format!("{} {} {}", title, summary, content);
                let words: Vec<&str> = text.split_whitespace().filter(|w| w.len() > 3).collect();
                let mut seen = std::collections::HashSet::new();
                words.into_iter()
                    .filter(|w| seen.insert(*w))
                    .take(24)
                    .map(|s| s.to_lowercase())
                    .collect::<Vec<_>>()
            };
            let meta_val = if concepts.is_empty() {
                json!({"enriched_at": now}).to_string()
            } else {
                json!({"ingest_index": {"concepts": concepts, "compiled_at": now}, "enriched_at": now}).to_string()
            };

            let tx = conn.unchecked_transaction().expect("tx begin");
            tx.execute(
                "INSERT INTO nodes (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count, metadata, data_tier, temporal, supersedes, source_episode, tier, recall_weight, norm_title, valid_start_time, valid_end_time, transaction_time, parent_id, depth, cluster_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13, 'core', NULL, NULL, NULL, 'warm', 1.0, ?14, 0, 0, 0, NULL, 0, NULL)",
                params![
                    id, node_type, title,
                    summary.is_empty().then(|| None).unwrap_or(Some(&summary)),
                    content.is_empty().then(|| None).unwrap_or(Some(&content)),
                    norm_url, domain, language,
                    0.9_f64, importance, now, now,
                    meta_val,
                    title.to_lowercase(),
                ],
            ).expect("insert node");
            // FTS 同步
            tx.execute(
                "INSERT INTO nodes_fts (rowid, title, summary, content, domain)
                 SELECT rowid, title, summary, content, domain FROM nodes WHERE id=?1",
                params![id],
            ).expect("fts sync");
            // 域枢纽 BelongsTo 边
            let hub_title = format!("domain:{}", domain);
            let hub_exists: bool = tx
                .query_row(
                    "SELECT 1 FROM nodes WHERE title=?1 AND node_type='concept' LIMIT 1",
                    params![hub_title],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if !hub_exists {
                let hub_id = format!("hub_{}", now);
                tx.execute(
                    "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, url, domain, language, confidence, importance, created_at, updated_at, data_tier, tier, recall_weight, transaction_time, depth)
                     VALUES (?1, 'concept', ?2, ?2, '', ?3, 'en', 0.9, 0.7, ?4, ?4, 'core', 'warm', 1.0, 0, 0)",
                    params![hub_id, hub_title, domain, now],
                ).ok();
                tx.execute(
                    "INSERT OR IGNORE INTO nodes_fts (rowid, title, summary, content, domain)
                     SELECT rowid, title, summary, content, domain FROM nodes WHERE id=?1",
                    params![hub_id],
                ).ok();
                tx.execute(
                    "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at, transaction_time)
                     VALUES (?1, ?2, ?3, 'belongs_to', 1.0, ?4, ?5, 0)",
                    params![format!("edge_{}_{}", now, i), id, hub_id, format!("{} → {}", title, domain), now],
                ).ok();
            } else {
                let hub_id: String = tx
                    .query_row(
                        "SELECT id FROM nodes WHERE title=?1 AND node_type='concept' LIMIT 1",
                        params![hub_title],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                tx.execute(
                    "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at, transaction_time)
                     VALUES (?1, ?2, ?3, 'belongs_to', 1.0, ?4, ?5, 0)",
                    params![format!("edge_{}_{}", now, i), id, hub_id, format!("{} → {}", title, domain), now],
                ).ok();
            }
            tx.commit().expect("tx commit");
            id
        };
        inserted += 1;
        // 保留输入 meta 字段
        let mut meta = match n.get("meta") {
            Some(Value::Object(m)) => Value::Object(m.clone()),
            _ => json!({}),
        };
        if meta.get("enriched_at").is_none() {
            meta["enriched_at"] = json!(now);
        }
        conn.execute(
            "UPDATE nodes SET metadata=?1 WHERE id=?2",
            params![Value::Object(meta.as_object().cloned().unwrap_or_default()).to_string(), node_id],
        ).ok();
        let _report_created = true;
        let report_node_id = node_id.clone();
        println!(
            "[absorb-node] {} #{}: {} ({}, lang={})",
            "inserted",
            i,
            norm_url,
            node_type,
            language,
        );
        let eid = report_node_id;

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


pub(crate) fn cmd_update_node_metadata(conn: &Connection, input: &str, dry_run: bool) {
    let raw = if input == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .expect("read stdin");
        buf
    } else {
        std::fs::read_to_string(input).unwrap_or_else(|e| {
            eprintln!("[update-metadata] ✗ failed to read {}: {}", input, e);
            std::process::exit(1);
        })
    };
    let v: Value = serde_json::from_str(&raw).expect("update list is valid JSON");
    let updates: Vec<Value> = match v {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![v],
        _ => {
            eprintln!("[update-metadata] ✗ input must be a JSON array of {{node_id, patch}} objects");
            std::process::exit(1);
        }
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
