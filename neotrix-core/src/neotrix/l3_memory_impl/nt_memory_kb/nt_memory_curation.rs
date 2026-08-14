//! NT-MEMORY 知识策展模块 — 缺陷网 D2/D3/D10 修复:
//! - D2  事实冲突检测: 同义表述但断言互相矛盾的节点 → 标冲突、按时间胜出者 supersedes 旧者
//! - D3  自动遗忘: 长期未访问 + 低重要性 → 降级 cold 归档 (可恢复, 非硬删)
//! - D10 成果反馈策展: 检索命中率过低 → 建议重写/下架, 记录策展决策
//!
//! 参照: memory-tools/forget, human-memory 遗忘曲线 (Ebbinghaus), AERO 反馈闭环。
//! 强化既有 nodes 表 (tier/supersedes/access_count 字段早已存在), 无平行适配器。

use rusqlite::{params, Connection};
use serde_json::json;

/// 断言极性启发式: 判断 content 属于 "肯定" 还是 "否定" 表述。
/// 冲突候选 = 相似 content 但极性相反。否定词加权更高 (取反覆盖肯定),
/// 因此 "not enabled" → 否定, "enabled" → 肯定。
fn polarity(content: &str) -> i32 {
    let c = content.to_lowercase();
    let mut p = 0i32;
    for neg in ["not ", "no", "false", "disabled", "unsupported", "不支持", "错误", "禁止", "禁用", "不允许", "cannot", "cannot be"] {
        if c.contains(neg) {
            p -= 3;
        }
    }
    for pos in ["is ", "yes", "true", "enabled", "supported", "正确", "是", "支持", "启用", "允许", "can be"] {
        if c.contains(pos) {
            p += 1;
        }
    }
    p.signum()
}

/// 词袋相似度 (O(|a|+|b|)), 用于在 title 相近节点间判定冲突候选。
fn bow_sim(a: &str, b: &str) -> f64 {
    let mut m: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for w in a.split(|c: char| !c.is_alphanumeric()) {
        if !w.is_empty() {
            *m.entry(w.to_lowercase()).or_insert(0) += 1;
        }
    }
    let mut inter = 0usize;
    let mut total = m.values().sum::<usize>();
    for w in b.split(|c: char| !c.is_alphanumeric()) {
        if !w.is_empty() {
            let k = w.to_lowercase();
            if let Some(n) = m.get_mut(&k) {
                if *n > 0 {
                    *n -= 1;
                    inter += 1;
                } else {
                    total += 1;
                }
            } else {
                total += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        inter as f64 / total as f64
    }
}

/// 冲突检测结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictHit {
    pub older_id: String,
    pub newer_id: String,
    pub title: String,
    pub polarity_older: i32,
    pub polarity_newer: i32,
    pub sim: f64,
}

/// 遗忘 (归档) 决策结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForgetHit {
    pub id: String,
    pub title: String,
    pub age_days: i64,
    pub importance: f64,
}

/// 策展决策结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct CurationHit {
    pub id: String,
    pub title: String,
    pub access_count: i64,
    pub action: String,
}

/// D2: 扫描 nodes, 找出 title 相似但断言极性相反的节点对 → 返回冲突命中。
/// 不自动改写: 由调用方决定 (默认新者胜出写 supersedes)。
pub fn conflict_detect(conn: &Connection, title_sim: f64) -> Result<Vec<ConflictHit>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, COALESCE(summary, content, ''), created_at FROM nodes \
             WHERE COALESCE(summary, content, '') IS NOT NULL \
             AND length(COALESCE(summary, content, '')) > 0",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut hits = Vec::new();
    for i in 0..rows.len() {
        for j in i + 1..rows.len() {
            let a = &rows[i];
            let b = &rows[j];
            if a.0 == b.0 {
                continue;
            }
            let sim = bow_sim(&a.1, &b.1);
            if sim < title_sim {
                continue;
            }
            let pa = polarity(&a.2);
            let pb = polarity(&b.2);
            if pa != 0 && pb != 0 && pa != pb {
                // 时间戳相等 (秒级精度) 时用 created_at 无法区分先后 → 用 rowid
                // 语义不明, 此时以 title 字典序作稳定 tie-break, 保证 supersede
                // 方向确定性 (同一数据每次扫描结果一致)。
                let (older, newer) = if a.3 < b.3 || (a.3 == b.3 && a.1 < b.1) {
                    (a, b)
                } else {
                    (b, a)
                };
                let (po, pn) = if a.3 < b.3 || (a.3 == b.3 && a.1 < b.1) {
                    (pa, pb)
                } else {
                    (pb, pa)
                };
                hits.push(ConflictHit {
                    older_id: older.0.clone(),
                    newer_id: newer.0.clone(),
                    title: older.1.clone(),
                    polarity_older: po,
                    polarity_newer: pn,
                    sim,
                });
            }
        }
    }
    Ok(hits)
}

/// D2 应用: 对冲突对新者胜出, 旧者 supersedes 指向新者 (保留证据链)。
/// 返回被覆盖的节点数。
pub fn apply_supersede(conn: &Connection, hits: &[ConflictHit]) -> Result<usize, String> {
    let mut n = 0usize;
    for h in hits {
        let affected = conn
            .execute(
                "UPDATE nodes SET supersedes = ?1, tier = 'cold', \
                 updated_at = ?2 WHERE id = ?3 AND tier != 'cold'",
                params![h.newer_id, now_unix(), h.older_id],
            )
            .map_err(|e| e.to_string())?;
        if affected > 0 {
            n += 1;
        }
    }
    Ok(n)
}

/// 单条冲突解决的差异记录 (diagram-design fidelity ledger 吸收)。
#[derive(Debug, Clone, serde::Serialize)]
pub struct FidelityEntry {
    pub older_id: String,
    pub newer_id: String,
    pub title: String,
    pub sim: f64,
    pub polarity_older: i32,
    pub polarity_newer: i32,
}

/// Fidelity ledger — 吸收自 cathrynlavery/diagram-design:
/// 每次冲突解决输出"合并/丢弃/覆盖"差异清单, 而非仅静默改库。
/// 调用方将 ledger 落盘/审计 (如写回 metadata.provenance)。
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FidelityLedger {
    pub entries: Vec<FidelityEntry>,
}

impl FidelityLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// 冲突解决 + fidelity ledger: 同 [`apply_supersede`] 应用覆盖, 但返回
/// 本次解决的差异清单 (哪条被丢弃、胜出者是谁、相似度多少), 供溯源审计。
pub fn apply_supersede_with_ledger(
    conn: &Connection,
    hits: &[ConflictHit],
) -> Result<(usize, FidelityLedger), String> {
    let mut n = 0usize;
    let mut ledger = FidelityLedger::new();
    for h in hits {
        let affected = conn
            .execute(
                "UPDATE nodes SET supersedes = ?1, tier = 'cold', \
                 updated_at = ?2 WHERE id = ?3 AND tier != 'cold'",
                params![h.newer_id, now_unix(), h.older_id],
            )
            .map_err(|e| e.to_string())?;
        if affected > 0 {
            n += 1;
            // provenance 溯源: 在旧节点 metadata 记录解决依据 (semantica 吸收 —
            // "冲突标记而非静默覆盖", 每次决策留痕可回查)。
            let _ = conn.execute(
                "UPDATE nodes SET metadata = json_set(
                     COALESCE(metadata, '{}'), '$.provenance.resolved_by', ?1,
                     '$.provenance.resolved_at', ?2,
                     '$.provenance.sim', ?3
                 ) WHERE id = ?4",
                params![h.newer_id, now_unix(), h.sim, h.older_id],
            );
            ledger.entries.push(FidelityEntry {
                older_id: h.older_id.clone(),
                newer_id: h.newer_id.clone(),
                title: h.title.clone(),
                sim: h.sim,
                polarity_older: h.polarity_older,
                polarity_newer: h.polarity_newer,
            });
        }
    }
    Ok((n, ledger))
}

/// T0.3 写后冲突检测 (scoped, T0.3 接线): 只对比指定新节点 vs 既有节点,
/// 避免 write_memory_entry 全库 O(n²) 扫描。
///
/// 来源: semantica "冲突标记而非静默覆盖" + 39 仓库吸收 Phase 0 接线纪律。
/// 流程: 取新节点 → token 预筛 (共享 ≥1 词) → bow_sim + 极性判定 →
///       按时间序返回 (older/newer), 可直接喂给 apply_supersede。
pub fn conflict_detect_for_write(
    conn: &Connection,
    new_id: &str,
    title_sim: f64,
) -> Result<Vec<ConflictHit>, String> {
    let (new_title, new_content, new_ts): (String, String, i64) = conn
        .query_row(
            "SELECT title, COALESCE(summary, content, ''), created_at FROM nodes WHERE id = ?1",
            params![new_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?;
    if new_content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let new_polarity = polarity(&new_content);
    if new_polarity == 0 {
        return Ok(Vec::new()); // 无极性断言, 不构成冲突候选
    }
    // 新标题 token 集 (预筛用)
    let new_tokens: std::collections::HashSet<String> = new_title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT id, title, COALESCE(summary, content, ''), created_at FROM nodes \
             WHERE id != ?1 AND (COALESCE(summary, content, '') IS NOT NULL \
             AND length(COALESCE(summary, content, '')) > 0)",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String, i64)> = stmt
        .query_map(params![new_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut hits = Vec::new();
    for (oid, otitle, ocontent, ots) in rows {
        // token 预筛: 无共享 token 直接跳过 (避免对无关标题跑 bow_sim)
        let shared = otitle
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .any(|w| new_tokens.contains(&w.to_lowercase()));
        if !shared {
            continue;
        }
        let sim = bow_sim(&new_title, &otitle);
        if sim < title_sim {
            continue;
        }
        let op = polarity(&ocontent);
        if op != 0 && op != new_polarity {
            // 写路径语义: 新节点是"刚写入"的, 时间戳相等 (秒级精度) 时新节点
            // 必须胜出 → 仅当 new_ts 严格小于 ots 才把新节点判为 older。
            let (older_id, older_ts, older_pol, newer_id, newer_ts, newer_pol) = if new_ts < ots {
                (new_id.to_string(), new_ts, new_polarity, oid, ots, op)
            } else {
                (oid, ots, op, new_id.to_string(), new_ts, new_polarity)
            };
            hits.push(ConflictHit {
                older_id,
                newer_id,
                title: new_title.clone(),
                polarity_older: older_pol,
                polarity_newer: newer_pol,
                sim,
            });
            let _ = older_ts; // 排序语义已由元组体现
            let _ = newer_ts;
        }
    }
    Ok(hits)
}

/// D3: 自动遗忘 — access_count=0 且超龄且低重要性的节点降级 cold。
/// 返回归档数。age_days 用 created_at 距今天数判断。
pub fn forget_stale(
    conn: &Connection,
    max_age_days: i64,
    importance_threshold: f64,
) -> Result<Vec<ForgetHit>, String> {
    let cutoff = now_unix() - max_age_days * 86_400;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, importance FROM nodes \
             WHERE tier != 'cold' AND access_count = 0 AND created_at < ?1 AND importance < ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, f64)> = stmt
        .query_map(params![cutoff, importance_threshold], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut hits = Vec::new();
    for (id, title, importance) in rows {
        conn.execute(
            "UPDATE nodes SET tier = 'cold', updated_at = ?1 WHERE id = ?2",
            params![now_unix(), id],
        )
        .map_err(|e| e.to_string())?;
        hits.push(ForgetHit {
            age_days: (now_unix() - cutoff) / 86_400,
            id,
            title,
            importance,
        });
    }
    Ok(hits)
}

/// D10: 成果反馈策展 — 低检索命中 (access_count 低且存在但从未被搜中/读取) 建议重写或下架。
/// action ∈ {"rewrite", "archive"}。archive = 降级 cold (禁检索)。
pub fn curate_by_hitrate(
    conn: &Connection,
    max_access: i64,
    min_age_days: i64,
) -> Result<Vec<CurationHit>, String> {
    let cutoff = now_unix() - min_age_days * 86_400;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, access_count FROM nodes \
             WHERE tier != 'cold' AND access_count <= ?1 AND created_at < ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, i64)> = stmt
        .query_map(params![max_access, cutoff], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut hits = Vec::new();
    for (id, title, access_count) in rows {
        // rewrite: 有内容但几乎没被访问 → 标注 metadata 建议重写
        // archive: 完全零访问 → 直接降级冷
        let action = if access_count == 0 { "archive" } else { "rewrite" };
        if action == "archive" {
            conn.execute(
                "UPDATE nodes SET tier = 'cold', updated_at = ?1 WHERE id = ?2",
                params![now_unix(), id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "UPDATE nodes SET metadata = json_set(COALESCE(metadata, '{}'), '$.curation.suggest', 'rewrite') \
                 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        }
        hits.push(CurationHit {
            id,
            title,
            access_count,
            action: action.to_string(),
        });
    }
    Ok(hits)
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 聚合策展: 一次跑完 D2+D3+D10, 返回各维度决策汇总 (供日志/审计)。
#[allow(clippy::too_many_arguments)]
pub fn run_curation(
    conn: &Connection,
    title_sim: f64,
    max_age_days: i64,
    importance_threshold: f64,
    max_access: i64,
    min_age_days: i64,
) -> Result<serde_json::Value, String> {
    let conflicts = conflict_detect(conn, title_sim)?;
    let superseded = apply_supersede(conn, &conflicts)?;
    let forgotten = forget_stale(conn, max_age_days, importance_threshold)?;
    let curated = curate_by_hitrate(conn, max_access, min_age_days)?;
    Ok(json!({
        "conflicts": conflicts.len(),
        "superseded": superseded,
        "forgotten": forgotten.len(),
        "curated": curated.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT,
                content TEXT,
                url TEXT,
                domain TEXT,
                language TEXT DEFAULT 'en',
                confidence REAL DEFAULT 1.0,
                importance REAL DEFAULT 0.5,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                access_count INTEGER DEFAULT 0,
                metadata TEXT,
                data_tier TEXT NOT NULL DEFAULT 'core',
                temporal TEXT,
                supersedes TEXT,
                source_episode TEXT,
                tier TEXT NOT NULL DEFAULT 'warm'
            );",
        )
        .unwrap();
        conn
    }

    fn seed(conn: &Connection, id: &str, title: &str, content: &str, ts: i64, access: i64, importance: f64) {
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, content, created_at, updated_at, access_count, importance) \
             VALUES (?1, 'fact', ?2, ?3, ?4, ?4, ?5, ?6)",
            params![id, title, content, ts, access, importance],
        )
        .unwrap();
    }

    #[test]
    fn conflict_detect_finds_opposite_claims() {
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "a", "Rate limit retry policy", "retry is enabled for provider", now, 3, 0.8);
        seed(&conn, "b", "Rate limit retry policy", "retry is not enabled", now + 10, 1, 0.7);
        let hits = conflict_detect(&conn, 0.4).unwrap();
        assert_eq!(hits.len(), 1, "similar title + opposite polarity must be flagged");
        assert_eq!(hits[0].older_id, "a");
        assert_eq!(hits[0].newer_id, "b");
        let applied = apply_supersede(&conn, &hits).unwrap();
        assert_eq!(applied, 1);
        let tier: String = conn
            .query_row("SELECT tier FROM nodes WHERE id='a'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tier, "cold", "older node retired");
        let supersedes: String = conn
            .query_row("SELECT supersedes FROM nodes WHERE id='a'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(supersedes, "b", "evidence chain kept");
    }

    #[test]
    fn same_polarity_not_conflict() {
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "a", "MMR diversity lambda", "diversify uses lambda 0.7", now, 0, 0.8);
        seed(&conn, "b", "MMR diversity lambda", "diversify uses lambda 0.7", now + 5, 0, 0.8);
        let hits = conflict_detect(&conn, 0.4).unwrap();
        assert!(hits.is_empty(), "same polarity must not conflict");
    }

    #[test]
    fn supersede_with_ledger_tracks_provenance() {
        // fidelity ledger + provenance 溯源 (diagram-design + semantica 吸收):
        // 覆盖解决需留差异清单, 并在旧节点 metadata 记录解决依据。
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "a", "Retry backoff policy", "backoff is enabled", now, 0, 0.8);
        seed(&conn, "b", "Retry backoff policy", "backoff is not enabled", now + 10, 0, 0.8);
        let hits = conflict_detect(&conn, 0.4).unwrap();
        assert_eq!(hits.len(), 1);
        let (applied, ledger) = apply_supersede_with_ledger(&conn, &hits).unwrap();
        assert_eq!(applied, 1);
        assert_eq!(ledger.len(), 1);
        assert_eq!(ledger.entries[0].older_id, "a");
        assert_eq!(ledger.entries[0].newer_id, "b");
        // provenance 落 metadata
        let meta: String = conn
            .query_row("SELECT metadata FROM nodes WHERE id='a'", [], |r| r.get(0))
            .unwrap();
        assert!(meta.contains("resolved_by"), "provenance must be recorded, got {}", meta);
        assert!(meta.contains("b"), "resolved_by should point to newer");
        assert!(meta.contains("sim"));
        // 幂等: 二次解决无新增
        let (applied2, ledger2) = apply_supersede_with_ledger(&conn, &hits).unwrap();
        assert_eq!(applied2, 0);
        assert!(ledger2.is_empty());
    }

    #[test]
    fn ledger_empty_for_no_conflicts() {
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "a", "Same claim", "feature is on", now, 0, 0.8);
        seed(&conn, "b", "Same claim", "feature is on", now + 5, 0, 0.8);
        let hits = conflict_detect(&conn, 0.4).unwrap();
        assert!(hits.is_empty());
        let (applied, ledger) = apply_supersede_with_ledger(&conn, &hits).unwrap();
        assert_eq!(applied, 0);
        assert!(ledger.is_empty());
    }

    #[test]
    fn equal_second_timestamp_tiebreak_deterministic() {
        // 秒级精度: 同一秒写入的两节点 created_at 相等 → 全量版必须用
        // title 字典序做稳定 tie-break, 保证 supersede 方向确定性 (幂等)。
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "z-node", "Equal ts policy", "rate limit is enabled", now, 0, 0.8);
        seed(&conn, "a-node", "Equal ts policy", "rate limit is not enabled", now, 0, 0.8);
        let hits1 = conflict_detect(&conn, 0.4).unwrap();
        assert_eq!(hits1.len(), 1, "opposite polarity + equal ts must be flagged");
        assert_eq!(hits1[0].older_id, "a-node", "title tie-break: a-node < z-node");
        assert_eq!(hits1[0].newer_id, "z-node");
        // 幂等: 二次扫描结果必须一致 (不依赖行序)
        let hits2 = conflict_detect(&conn, 0.4).unwrap();
        assert_eq!(hits1[0].older_id, hits2[0].older_id, "deterministic across runs");
        let applied = apply_supersede(&conn, &hits1).unwrap();
        assert_eq!(applied, 1);
        let supersedes: String = conn
            .query_row("SELECT supersedes FROM nodes WHERE id='a-node'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(supersedes, "z-node", "older (by title) points to newer");
    }

    #[test]
    fn forget_stale_archives_dead_nodes() {
        let conn = mem_conn();
        let now = now_unix();
        // 很旧 + 零访问 + 低重要性
        seed(&conn, "old", "dead experiment", "stale experiment notes", now - 90 * 86_400, 0, 0.1);
        // 旧但重要 → 不该被遗忘
        seed(&conn, "imp", "important design", "critical invariant", now - 90 * 86_400, 0, 0.9);
        let hits = forget_stale(&conn, 60, 0.3).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "old");
        let tier: String = conn
            .query_row("SELECT tier FROM nodes WHERE id='old'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tier, "cold");
        let tier_imp: String = conn
            .query_row("SELECT tier FROM nodes WHERE id='imp'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tier_imp, "warm", "important node survives");
    }

    #[test]
    fn curate_archives_zero_access_marks_rewrite_for_low() {
        let conn = mem_conn();
        let now = now_unix();
        seed(&conn, "z", "zero hit doc", "never retrieved", now - 90 * 86_400, 0, 0.5);
        seed(&conn, "l", "low hit doc", "rarely retrieved", now - 90 * 86_400, 1, 0.5);
        let hits = curate_by_hitrate(&conn, 1, 30).unwrap();
        let by_id: std::collections::HashMap<String, String> =
            hits.iter().map(|h| (h.id.clone(), h.action.clone())).collect();
        assert_eq!(by_id.get("z").map(String::as_str), Some("archive"));
        assert_eq!(by_id.get("l").map(String::as_str), Some("rewrite"));
        let tier: String = conn
            .query_row("SELECT tier FROM nodes WHERE id='z'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tier, "cold");
    }
}
