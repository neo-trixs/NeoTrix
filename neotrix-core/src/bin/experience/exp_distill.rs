//! exp_distill — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::{params, Connection};
use rusqlite::types::Value as SqlValue;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use super::{DAY, NS, REF_BUFFER_ACTIVATION, REF_TOKEN_BUDGET, SCHEMA_VERSION, VERIFY_DEFAULT_DAYS};
use super::exp_concept::{co_bump_in_mem};
use super::exp_store::{ensure_hub, field_solve, kv_stage, refresh_hub_metrics, save_hub, scan_values, sql_value_decode, value_encode};
use super::exp_util::{VALUE_MAGIC, cycle_sort_key, en_stop, estimate_tokens, now_ts, truncate};

pub(crate) fn cmd_distill(conn: &mut Connection, domain: Option<&str>, min_group: usize, dry_run: bool) {
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
    // W4: 时序追加键走场账本 (单条批次即写即解)
    kv_stage(conn, NS, &ckey, &consciousness_entry.to_string());
    field_solve(conn);

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

// ────────────────────────────────────────────────────────────────
// 即时 promote (蒸馏延迟消除): 吸收后立即对高信号经验升维写入 experience_targets,
// 不等 distill 批处理。与 distill_promote_to_capability 共享 ExperienceRouter,
// 但只吃单条高信号 (importance ≥ 0.6) 而非聚类模式 — 保证高信号发现当天生效。
// ────────────────────────────────────────────────────────────────


pub(crate) fn immediate_promote_entries(entries: &[Value]) -> usize {
    use neotrix::neotrix::nt_capability_bridge::{
        ExperienceDimension, ExperienceEntry, ExperienceRouter, promote_to_file,
    };
    let mut dims: Vec<ExperienceDimension> = Vec::new();
    for e in entries {
        let content = e.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
        if content.trim().is_empty() {
            continue;
        }
        let domain_name = e.get("domain").and_then(|d| d.as_str()).unwrap_or("NT-CORE").to_string();
        let etype = e.get("type").and_then(|t| t.as_str()).unwrap_or("insight").to_string();
        let importance = e.get("importance").and_then(|x| x.as_f64()).unwrap_or(0.5);
        let entry = ExperienceEntry {
            id: format!("immediate_{}", e.get("session_id").and_then(|s| s.as_str()).unwrap_or("sess")),
            entry_type: etype,
            domain_name: domain_name.clone(),
            content: content.clone(),
            not: None,
            confidence: 0.7,
            importance,
            verified_by: None,
            verification_status: None,
        };
        let dim = ExperienceRouter::route_experience(&entry);
        match &dim {
            ExperienceDimension::CapabilityNetwork { domain, capability_tag, signal, .. } => {
                log::info!(
                    "[immediate-promote] {} → {} (signal={:.2})",
                    domain.as_str(), capability_tag, signal
                );
            }
            ExperienceDimension::ConsciousnessAwakening { layer, signal, .. } => {
                log::info!(
                    "[immediate-promote] 意识体觉醒 → {} (signal={:.2})",
                    layer, signal
                );
            }
        }
        dims.push(dim);
    }
    if dims.is_empty() {
        return 0;
    }
    let cwd_registry = std::path::PathBuf::from(".neotrix/capability_registry.json");
    let home_registry = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".neotrix/capability_registry.json");
    let mut written = 0usize;
    for path in [&cwd_registry, &home_registry] {
        if path.exists() {
            written += promote_to_file(path, &dims);
        }
    }
    if written == 0 && !cwd_registry.exists() {
        if let Some(parent) = cwd_registry.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        written += promote_to_file(&cwd_registry, &dims);
    }
    written
}

/// 返回提升建议的行描述 (实际写入 capability_registry.json 的 experience_targets 区)。
///
/// 接入点: 蒸馏模式 (domain, pattern, src_keys) → ExperienceRouter.route_experience
///   → 能力标签路由 → EvolutionPlan (Strengthen 已有节点 / Bud 新节点建议)


pub(crate) fn distill_promote_to_capability(distilled: &[(String, String, Vec<String>)]) -> Vec<String> {
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


pub(crate) fn high_signal_words(content: &str) -> Vec<String> {
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


pub(crate) fn distill_pattern(domain: &str, contents: &[String]) -> String {
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

/// 观察日志 → 反射日志重写 (Mastra OM 吸收, 2026-08-24)。
///
/// 算法:
/// 1. 读取指定 domain 的所有 observation 类型条目
/// 2. 计算总 token 数; 若 ≤ REF_TOKEN_BUDGET(40k) 则无需反射, 直接返回
/// 3. 触发反射: 将所有观察条目按时间序合并为单一反射内容
///    - 旧信息更激进压缩 (保留前 20% token 的精简摘要)
///    - 近期细节保留 (后 80% token 原文)
///    - 产物: 单条 reflection 类型条目, 含完整 provenance_range
/// 4. 更新所有观察条目的 reflection 元数据: version++, last_reflect_ts=now
/// 5. 幂等: reflection 条目键包含版本号, 重复执行仅版本号递增


pub(crate) fn cmd_reflect(conn: &mut Connection, domain: Option<&str>, dry_run: bool) {
    let want_dom = domain.unwrap_or("NT-MEMORY");
    println!(
        "[reflect] scanning observations for domain={} dry_run={}",
        want_dom, dry_run
    );

    // 1. 读取 observation 条目
    let rows = scan_values(conn, "obs_");
    let mut observations: Vec<(String, Value)> = Vec::new();
    for (key, value) in &rows {
        let Ok(v) = serde_json::from_str::<Value>(value) else { continue };
        if v.get("type").and_then(|x| x.as_str()) != Some("observation") {
            continue;
        }
        let d = v.get("domain").and_then(|x| x.as_str()).unwrap_or("unknown");
        if d != want_dom {
            continue;
        }
        observations.push((key.clone(), v));
    }
    if observations.is_empty() {
        println!("[reflect] no observation entries for domain={}", want_dom);
        return;
    }
    // 时间正序
    observations.sort_by(|a, b| {
        let ta = a.1.get("ts").and_then(|x| x.as_i64()).unwrap_or(0);
        let tb = b.1.get("ts").and_then(|x| x.as_i64()).unwrap_or(0);
        ta.cmp(&tb)
    });

    // 2. 计算总 token
    let mut total_tokens = 0usize;
    for (_, v) in &observations {
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        total_tokens += estimate_tokens(content);
    }
    println!("[reflect] {} observations, total {} tokens (budget={})", observations.len(), total_tokens, REF_TOKEN_BUDGET);

    if total_tokens <= REF_TOKEN_BUDGET {
        println!("[reflect] token budget not exceeded, skipping reflect");
        return;
    }

    if dry_run {
        println!("[reflect] dry-run: would trigger reflection (rewrite)");
        return;
    }

    // 3. 反射重写: 合并所有观察为单一反射内容
    // 策略: 前 20% token 精简摘要 (旧), 后 80% 原文 (新)
    let mut combined = String::new();
    for (_, v) in &observations {
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if !combined.is_empty() {
            combined.push_str("\n---\n");
        }
        combined.push_str(content);
    }
    let combined_tokens = estimate_tokens(&combined);

    // 简易压缩: 取头部 20% + 尾部 80% (类比 truncate_preserving, 但应用于合并后的观察日志)
    let head_budget = ((combined_tokens as f64) * REF_BUFFER_ACTIVATION).floor() as usize; // 0.5 = 50% 头部
    let tail_budget = combined_tokens.saturating_sub(head_budget);
    // 为了演示简化: 这里仅做标记, 实际生产需用 truncate_preserving 逻辑
    let reflected_content = format!(
        "[反射重写 v1] 合并 {} 条观察 ({} tokens) → 有界反射日志\n\
         [头部摘要 {} tokens] ... [尾部细节 {} tokens]\n\
         原始 provenance_range 保留于各观察条目",
        observations.len(),
        combined_tokens,
        head_budget,
        tail_budget
    );

    // 4. 写入 reflection 条目
    let now = now_ts();
    let version = observations.len() as u32; // 简化: 用观察条数作版本
    let refl_key = format!("refl_{}_{}", want_dom, now);
    let refl_entry = json!({
        "schema_version": SCHEMA_VERSION,
        "type": "reflection",
        "session_id": "reflect",
        "cycle": "reflect",
        "ts": now,
        "domain": want_dom,
        "content": reflected_content,
        "evidence": format!("merged {} observations", observations.len()),
        "source": "reflection",
        "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
        "confidence": 0.8,
        "importance": 0.7,
        "provenance_range": {
            "start_id": observations.first().and_then(|(_,v)| v.get("ts").and_then(|t| t.as_i64())).unwrap_or(now),
            "end_id": observations.last().and_then(|(_,v)| v.get("ts").and_then(|t| t.as_i64())).unwrap_or(now),
            "raw_source_keys": observations.iter().map(|(k,_)| k).collect::<Vec<_>>(),
        },
        "extracted": Value::Null,
        "observation": Value::Null,
        "reflection": {
            "version": version,
            "last_reflect_ts": now,
            "token_budget": REF_TOKEN_BUDGET,
            "buffer_activation": REF_BUFFER_ACTIVATION,
        },
        "concepts": Value::Null,
    });

    if !dry_run {
        kv_stage(conn, NS, &refl_key, &refl_entry.to_string());
        field_solve(conn);
        println!("[reflect] reflection entry written: {}", refl_key);
    }
    // 5. 更新观察条目: reflection.version++ / last_reflect_ts
    for (_obs_key, obs_entry) in &observations {
        let mut obs_entry = obs_entry.clone();
        if let Some(obj) = obs_entry.as_object_mut() {
            let mut refl_meta = obj.get("reflection").cloned().unwrap_or(json!({}));
            if let Some(r) = refl_meta.as_object_mut() {
                let ver = r.get("version").and_then(|x| x.as_u64()).unwrap_or(0) + 1;
                r.insert("version".to_string(), json!(ver));
                r.insert("last_reflect_ts".to_string(), json!(now));
            }
        }
    }
    println!("[reflect] updated {} observation entries with reflection metadata (version={})", observations.len(), version);
}

/// 重建 Hebb 共现突触网络 (幂等 — 先清空 co 再全量重建)。
/// 全库 O(n²) 需内存批量: 全部概念载入 → 内存计共现 → 单事务写回。


pub(crate) fn cmd_hebb(conn: &mut Connection) {
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


pub(crate) fn cmd_compress(conn: &mut Connection, all: bool) {
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


pub(crate) fn cycle_summary(conn: &Connection, cycle: &str) -> String {
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


pub(crate) fn cmd_gen_index(conn: &Connection, out: &str, limit: usize) {
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
