use rusqlite::{params, Connection};

use super::super::bm25;
use super::super::nt_memory_embed::load_all_embeddings;
use super::super::nt_memory_types::*;
use crate::l0_substrate::nt_core_math::{cosine_similarity_f32, cosine_similarity_f64};
#[cfg(feature = "full")]
use log::debug;

// Tests cross-module refs: re-export so existing `super::X` in moved tests keeps resolving.
// Production code has no cycle (these are test-only).
#[cfg(test)]
use super::nt_cache::MaterializedNeighborCacheSelfTest;
#[cfg(test)]
use super::nt_gate_optimizer::{Fts5OptimizerConfig, RetrievalEvolver, RetrievalTuning};
#[cfg(test)]
use super::nt_scoring::cosine_f32;

// ── Pure search fns (moved from nt_memory_search.rs, pure move) ──

pub fn search_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<SearchResult>> {
    // FTS5 rank = bm25 分数, 越大越相关 (大文档 term 密度低, rank 偏小 → 需标题加权纠正)
    // ORDER BY rank DESC: 修正原实现 ASC + score=1.0-rank 的双重反向缺陷
    // 标题加权在 SQL 层: title 与查询词完全相等 → 排最前 (LIMIT 前生效, 防大文档被截断)
    let mut stmt = conn.prepare(
"SELECT n.id, n.node_type, n.title, n.summary, COALESCE(n.content, n.summary, ''), n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata,
                rank,
                CASE WHEN trim(n.title) = ?1 THEN 1
                     WHEN trim(n.title) LIKE ?1 || '%' THEN 0.5
                     ELSE 0 END as title_boost
         FROM nodes_fts f
         JOIN nodes n ON n.rowid = f.rowid
         WHERE nodes_fts MATCH ?2
         ORDER BY title_boost DESC, rank DESC
         LIMIT ?3",
    )?;

    // 标题加权: 查询词与节点标题精确相等 → 该节点是"本体"而非"引用者",
    // 给予强加权 (score 拉高)。前缀相等 (标题以查询词开头) → 弱加权。
    let query_trim = query.trim();
    let exact_title = |title: &str| -> bool {
        let t = title.trim();
        t == query_trim
    };
    let prefix_title = |title: &str| -> bool {
        let t = title.trim();
        t.starts_with(query_trim) && t.len() > query_trim.len()
    };

    let rows = stmt.query_map(params![query, query, limit as i64], |row| {
        let title: String = row.get(2)?;
        let rank: f64 = row.get(14)?;
        // score = rank + 偏移 (rank 为负值, 加 1.0 归一避免全负)
        let mut score = rank + 1.0;
        // 标题加权: 精确命中 +1.0 (远高于引用书的 rank 差异), 前缀命中 +0.3
        let is_prefix = prefix_title(&title);
        let matched_on = if exact_title(&title) {
            score += 1.0;
            SearchMatchType::FtsTitle
        } else if is_prefix {
            score += 0.3;
            SearchMatchType::FtsTitle
        } else {
            SearchMatchType::FtsContent
        };
        Ok(SearchResult {
            node: KnowledgeNode {
                recall_weight: 1.0,
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                title,
                summary: row.get(3)?,
                content: row.get(4)?,
                url: row.get(5)?,
                domain: row.get(6)?,
                language: row.get(7)?,
                confidence: row.get(8)?,
                importance: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                access_count: row.get(12)?,
                metadata: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|m| serde_json::from_str(&m).ok()),
                temporal: None,
                supersedes: None,
                source_episode: None,
                parent_id: None,
                depth: 0,
                cluster_id: None,
            },
            score,
            matched_on: vec![matched_on],
            signals: None,
        })
    })?;

    // 标题加权后按 (FtsTitle > FtsContent, score) 重排:
    // ⚠️ 不能用 score 降序 — rank 是 FTS5 负值, 大文档(原书全文) rank 极负,
    // 标题加权 +1.0 不足以抵消 (-11 vs -0.01), 会把原书压到引用书后面。
    // 正确排序键: 先精确标题命中(原书), 再按 score。
    let title_pri = |r: &SearchResult| -> u8 {
        if r.matched_on
            .iter()
            .any(|m| matches!(m, SearchMatchType::FtsTitle))
        {
            2
        } else {
            1
        }
    };
    let mut results: Vec<SearchResult> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    // FTS5 unicode61 精确词匹配不到 CJK 子串 ("最短路径管道" vs "最短路径管道测试" 单词元) 时，
    // MATCH 返回空但节点已正确双写提交。LIKE 子串兜底保证写↔读闭环。
    if results.is_empty() && !query_trim.is_empty() {
        let escaped = query_trim
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        if let Ok(mut like_stmt) = conn.prepare(
            "SELECT id, node_type, title, summary, COALESCE(content, summary, ''), url, domain,
                    language, confidence, importance, created_at, updated_at, access_count, metadata
             FROM nodes
             WHERE title LIKE ?1 ESCAPE '\\' OR summary LIKE ?1 ESCAPE '\\' OR content LIKE ?1 ESCAPE '\\'
             LIMIT ?2",
        ) {
            if let Ok(like_rows) = like_stmt.query_map(params![pattern, limit as i64], |row| {
                let title: String = row.get(2)?;
                let is_title_hit: bool = title.contains(query_trim);
                Ok(SearchResult {
                    node: KnowledgeNode {
                        recall_weight: 1.0,
                        id: row.get(0)?,
                        node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                        title,
                        summary: row.get(3)?,
                        content: row.get(4)?,
                        url: row.get(5)?,
                        domain: row.get(6)?,
                        language: row.get(7)?,
                        confidence: row.get(8)?,
                        importance: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                        access_count: row.get(12)?,
                        metadata: row
                            .get::<_, Option<String>>(13)?
                            .and_then(|m| serde_json::from_str(&m).ok()),
                        temporal: None,
                        supersedes: None,
                        source_episode: None,
                        parent_id: None,
                        depth: 0,
                        cluster_id: None,
                    },
                    score: if is_title_hit { 0.6 } else { 0.4 },
                    matched_on: vec![if is_title_hit {
                        SearchMatchType::FtsTitle
                    } else {
                        SearchMatchType::FtsContent
                    }],
                    signals: None,
                })
            }) {
                for r in like_rows.filter_map(|r| r.ok()) {
                    results.push(r);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
    }
    // W1.4 (batch3 2026-08-26, arxiv 2608.20845 ingest-time compilation):
    // 查询词与节点摄取时编译的概念索引求交 → 命中加分 (每词 +0.05, 封顶 +0.25)。
    // 纯增量修正: 无 ingest_index 的存量节点行为不变。
    let query_concepts = super::super::nt_memory_pipeline::compile_ingest_index(query, "", "");
    if !query_concepts.is_empty() {
        // 概念命中语义: 相等, 或双向包含且短侧 ≥3 字 (CJK 复合词窗口匹配)
        let hit = |q: &str, concepts: &[&str]| -> bool {
            concepts.iter().any(|c| {
                *c == q
                    || (q.chars().count().min(c.chars().count()) >= 3
                        && (c.contains(q) || q.contains(c)))
            })
        };
        for r in results.iter_mut() {
            let Some(meta) = r.node.metadata.as_ref() else {
                continue;
            };
            let Some(idx) = meta
                .get("ingest_index")
                .and_then(|v| v.get("concepts"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };
            let idx_terms: Vec<&str> = idx.iter().filter_map(|c| c.as_str()).collect();
            if idx_terms.is_empty() {
                continue;
            }
            let hits = query_concepts
                .iter()
                .filter(|q| hit(q.as_str(), &idx_terms))
                .count();
            if hits > 0 {
                r.score += 0.05 * (hits as f64).min(5.0);
                r.matched_on.push(SearchMatchType::IngestConcept);
            }
        }
    }
    results.sort_by(|a, b| {
        let pa = title_pri(a);
        let pb = title_pri(b);
        // 优先级大的排前 (FtsTitle=2 > FtsContent=1): 用 pb.cmp(&pa) 实现降序
        pb.cmp(&pa).then(
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    #[cfg(feature = "full")]
    debug!(
        "[search_fts] q={} rows={} first10={:?}",
        query,
        results.len(),
        results
            .iter()
            .take(10)
            .map(|r| format!("{}|{:.2}|{:?}", r.node.title, r.score, r.matched_on))
            .collect::<Vec<_>>()
    );
    Ok(results)
}

pub fn search_by_type(
    conn: &Connection,
    node_type: &NodeType,
    limit: usize,
) -> rusqlite::Result<Vec<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, COALESCE(content, summary, ''), url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata
         FROM nodes
         WHERE node_type=?1
         ORDER BY importance DESC, access_count DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![node_type.as_str(), limit as i64], |row| {
        Ok(KnowledgeNode {
            recall_weight: 1.0,
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row
                .get::<_, Option<String>>(13)?
                .and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
        })
    })?;

    rows.collect()
}

pub fn get_related(
    conn: &Connection,
    node_id: &str,
    relation_type: Option<&str>,
    limit: usize,
) -> rusqlite::Result<Vec<SearchResult>> {
    let (sql, has_relation) = if let Some(_rt) = relation_type {
        ("SELECT n.id, n.node_type, n.title, n.summary, COALESCE(n.content, n.summary, ''), n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata, e.weight as score
             FROM edges e
             JOIN nodes n ON n.id = CASE WHEN e.source_id=?1 THEN e.target_id ELSE e.source_id END
             WHERE (e.source_id=?1 OR e.target_id=?1) AND e.relation_type=?2
             ORDER BY e.weight DESC
             LIMIT ?3".to_string(), true)
    } else {
        ("SELECT n.id, n.node_type, n.title, n.summary, COALESCE(n.content, n.summary, ''), n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata, e.weight as score
             FROM edges e
             JOIN nodes n ON n.id = CASE WHEN e.source_id=?1 THEN e.target_id ELSE e.source_id END
             WHERE e.source_id=?1 OR e.target_id=?1
             ORDER BY e.weight DESC
             LIMIT ?2".to_string(), false)
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<SearchResult> = if has_relation {
        stmt.query_map(params![node_id, relation_type, limit as i64], |row| {
            Ok(SearchResult {
                node: KnowledgeNode {
                    recall_weight: 1.0,
                    id: row.get(0)?,
                    node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    content: row.get(4)?,
                    url: row.get(5)?,
                    domain: row.get(6)?,
                    language: row.get(7)?,
                    confidence: row.get(8)?,
                    importance: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    access_count: row.get(12)?,
                    metadata: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|m| serde_json::from_str(&m).ok()),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                    parent_id: None,
                    depth: 0,
                    cluster_id: None,
                },
                score: row.get(14)?,
                matched_on: vec![SearchMatchType::GraphRelation],
                signals: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(params![node_id, limit as i64], |row| {
            Ok(SearchResult {
                node: KnowledgeNode {
                    recall_weight: 1.0,
                    id: row.get(0)?,
                    node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    content: row.get(4)?,
                    url: row.get(5)?,
                    domain: row.get(6)?,
                    language: row.get(7)?,
                    confidence: row.get(8)?,
                    importance: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    access_count: row.get(12)?,
                    metadata: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|m| serde_json::from_str(&m).ok()),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                    parent_id: None,
                    depth: 0,
                    cluster_id: None,
                },
                score: row.get(14)?,
                matched_on: vec![SearchMatchType::GraphRelation],
                signals: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

/// BM25 search helper — queries an in-memory Bm25Index (built by rebuild_bm25).
/// Returns results with score normalized to [0,1] for fusion with FTS scores.
pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    limit: usize,
    bm25: Option<&bm25::Bm25Index>,
) -> rusqlite::Result<Vec<SearchResult>> {
    let fts_results = search_fts(conn, query, limit * 3)?;
    let bm25_results: Vec<(f64, String)> = if let Some(idx) = bm25 {
        idx.search(query, limit * 3)
    } else {
        Vec::new()
    };

    // Convert FTS results to (score, id) pairs for RRF fusion
    let fts_pairs: Vec<(f64, String)> = fts_results
        .iter()
        .map(|r| (r.score, r.node.id.clone()))
        .collect();

    let mut ranklists: Vec<Vec<(f64, String)>> = Vec::new();
    if !fts_pairs.is_empty() {
        ranklists.push(fts_pairs);
    }
    if !bm25_results.is_empty() {
        ranklists.push(bm25_results);
    }

    // Walsh 正交通道 (nt_core_walsh 接线 — 能力网维度升维):
    // 用 Hadamard 正交编码对查询与候选文档做语义比对, 生成第三 ranklist 加入 RRF 融合。
    // 正交表示与 FTS/BM25 的词汇统计互补, 提升检索语义多样性 (cycle 251 经验: 检索排序三层缺陷)。
    let walsh_ranklist = build_walsh_ranklist(conn, query, &fts_results, limit * 3);
    if !walsh_ranklist.is_empty() {
        ranklists.push(walsh_ranklist);
    }

    // RRF fusion
    let mut fused = if ranklists.len() >= 2 {
        bm25::rrf_fuse(&ranklists)
    } else if ranklists.is_empty() {
        Vec::new()
    } else {
        ranklists.into_iter().next().expect("non-empty ranklists")
    };

    // 缺陷7修复 (真实运转): RRF 融合只按排名位置融合, 丢弃 search_fts 的标题加权分数,
    // 导致标题精确匹配的"本体"节点 (如《史记》book) 被内容高频引用它的 concept 节点
    // (BM25/内容命中排名靠前) 挤到后面。融合后恢复标题加权: FtsTitle 精确命中 +1.0,
    // 前缀命中 +0.3 (与 search_fts 内部加权一致), 再重排。
    {
        let title_boost: std::collections::HashMap<&str, f64> = fts_results
            .iter()
            .filter(|r| {
                r.matched_on
                    .iter()
                    .any(|m| matches!(m, SearchMatchType::FtsTitle))
            })
            .map(|r| {
                let t = r.node.title.trim();
                let boost = if t == query.trim() { 1.0 } else { 0.3 };
                (r.node.id.as_str(), boost)
            })
            .collect();
        for (score, id) in fused.iter_mut() {
            if let Some(b) = title_boost.get(id.as_str()) {
                *score += *b;
            }
        }
        fused.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    }
    #[cfg(feature = "full")]
    debug!(
        "[hybrid] fused={} first5={:?}",
        fused.len(),
        fused
            .iter()
            .take(5)
            .map(|(s, id)| format!("{:.2}|{}", s, id.chars().take(24).collect::<String>()))
            .collect::<Vec<_>>()
    );

    // Fetch full node data for fused IDs
    let mut fused_ids: Vec<String> = Vec::new();
    let mut fused_scores: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for (score, id) in fused.into_iter().take(limit) {
        fused_ids.push(id.clone());
        fused_scores.insert(id, score);
    }
    let mut results: Vec<SearchResult> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    if !fused_ids.is_empty() {
        let placeholders: Vec<String> = fused_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "SELECT id, node_type, title, summary, COALESCE(content, summary, ''), url, domain, language,
                confidence, importance, created_at, updated_at, access_count, metadata
             FROM nodes WHERE id IN ({})",
            placeholders.join(",")
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let params: Vec<&dyn rusqlite::types::ToSql> = fused_ids
                .iter()
                .map(|id| id as &dyn rusqlite::types::ToSql)
                .collect();
            if let Ok(rows) = stmt.query_map(params.as_slice(), |row| {
                let id: String = row.get(0)?;
                let score = fused_scores.get(&id).copied().unwrap_or(0.5);
                Ok(SearchResult {
                    node: KnowledgeNode {
                        recall_weight: 1.0,
                        id,
                        node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                        title: row.get(2)?,
                        summary: row.get(3)?,
                        content: row.get(4)?,
                        url: row.get(5)?,
                        domain: row.get(6)?,
                        language: row.get(7)?,
                        confidence: row.get(8)?,
                        importance: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                        access_count: row.get(12)?,
                        metadata: row
                            .get::<_, Option<String>>(13)?
                            .and_then(|m| serde_json::from_str(&m).ok()),
                        temporal: None,
                        supersedes: None,
                        source_episode: None,
                        parent_id: None,
                        depth: 0,
                        cluster_id: None,
                    },
                    score,
                    matched_on: vec![SearchMatchType::Bm25],
                    signals: None,
                })
            }) {
                for r in rows.filter_map(|r| r.ok()) {
                    if seen_ids.insert(r.node.id.clone()) {
                        results.push(r);
                    }
                }
            }
        }
    }

    // fetch 后按 fused 排名恢复顺序 (WHERE id IN 不保证顺序)
    results.sort_by(|a, b| {
        let ia = fused_ids
            .iter()
            .position(|x| *x == a.node.id)
            .unwrap_or(usize::MAX);
        let ib = fused_ids
            .iter()
            .position(|x| *x == b.node.id)
            .unwrap_or(usize::MAX);
        ia.cmp(&ib)
    });

    // agentmemory 吸收接线 (cycle 1188): confidence × 时间衰减重排。
    // 对标 agentmemory 的 confidence scoring + decay/auto-forget:
    //   - confidence 权重: 高置信节点 (知识经验证/确认) 应优先于低置信 (模糊抓取/占位)。
    //   - 时间衰减: 距离更新越久 (7 天半衰期), 排名越低 — 过期知识自动沉底 (auto-forget)。
    // 仅作同权重内二级排序键, 不覆盖 FTS/RRF/Walsh 的相关性主排序。
    if results.len() >= 2 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let half_life: i64 = 7 * 24 * 3600; // 7 天半衰期
        results.sort_by(|a, b| {
            // 主: fused 原排名
            let ia = fused_ids
                .iter()
                .position(|x| *x == a.node.id)
                .unwrap_or(usize::MAX);
            let ib = fused_ids
                .iter()
                .position(|x| *x == b.node.id)
                .unwrap_or(usize::MAX);
            // 仅当两结果处于相邻近排名 (差距 ≤ 1) 时, 才用置信×衰减做二级修正,
            // 避免颠覆强相关性 (标题精确命中/高分 BM25) 的既有排序。
            if ia.abs_diff(ib) <= 1 {
                let ra =
                    a.node.confidence.max(0.1) * decay_factor(now - a.node.updated_at, half_life);
                let rb =
                    b.node.confidence.max(0.1) * decay_factor(now - b.node.updated_at, half_life);
                rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                ia.cmp(&ib)
            }
        });
    }

    if results.len() >= limit {
        results.truncate(limit);
        return Ok(results);
    }

    // Tier 2: FTS5 title-only fallback — catches nodes missed by RRF fusion
    let remaining = limit - results.len();
    let fts_title_query: String = query
        .trim()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" OR ");
    let fts_rows = conn.prepare(
        "SELECT n.id, n.node_type, n.title, n.summary, COALESCE(n.content, n.summary, ''), n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata
         FROM nodes n
         JOIN nodes_fts f ON n.rowid = f.rowid
         WHERE nodes_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )
    .and_then(|mut stmt| {
        stmt.query_map(params![fts_title_query, remaining as i64], |row| {
            Ok(SearchResult {
                node: KnowledgeNode {
                    recall_weight: 1.0,
                    id: row.get(0)?,
                    node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    content: row.get(4)?,
                    url: row.get(5)?,
                    domain: row.get(6)?,
                    language: row.get(7)?,
                    confidence: row.get(8)?,
                    importance: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    access_count: row.get(12)?,
                    metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                    parent_id: None,
                    depth: 0,
                    cluster_id: None,
                },
                score: 0.1,
                matched_on: vec![SearchMatchType::FtsTitle],
                signals: None,
            })
        })?.collect::<Result<Vec<_>, _>>()
    });

    if let Ok(rows) = fts_rows {
        for r in rows {
            if seen_ids.insert(r.node.id.clone()) {
                results.push(r);
            }
        }
    }

    // Tier 3: embedding cosine rerank. If query embedding available, boost
    // results whose stored embedding is similar to the query embedding.
    if let Ok(embeddings) = load_all_embeddings(conn) {
        if !embeddings.is_empty() {
            // Build a simple avg-word-embedding from query words as proxy
            let query_embedding = query_to_avg_embedding(query, &embeddings);
            // DistilVDR 蒸馏学生 (R-P79 接线): 已训练学生存在则用蒸馏分数,
            // 否则退化回裸余弦。学生 = 两塔对角线性, 从余弦 teacher 点级回归。
            let student = super::super::nt_memory_distill::load_student();
            let mut scored: Vec<(SearchResult, f64)> = Vec::new();
            for r in &results {
                let emb_score =
                    if let Some(emb) = embeddings.iter().find(|(id, _)| *id == r.node.id) {
                        match &student {
                            Some(s) => s.score(&query_embedding, &emb.1),
                            None => cosine_similarity_f32(&query_embedding, &emb.1),
                        }
                    } else {
                        0.0
                    };
                let combined = r.score * 0.7 + emb_score * 0.3;
                scored.push((r.clone(), combined));
            }
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results = scored.into_iter().take(limit).map(|(r, _)| r).collect();
        }
    }

    Ok(results)
}

/// 时间衰减因子 (agentmemory decay/auto-forget 接线, cycle 1188)。
/// age ≥ 0; 半衰期 half_life 秒后衰减到 0.5, 之后指数递减 → 永远沉底但不消失。
pub(crate) fn decay_factor(age: i64, half_life: i64) -> f64 {
    if age <= 0 || half_life <= 0 {
        return 1.0;
    }
    0.5_f64.powf(age as f64 / half_life as f64)
}

/// 构建 Walsh 正交 ranklist (nt_core_walsh 接线 — 能力网维度升维)。
///
/// 用 Hadamard 正交编码对查询与候选文档做语义比对, 生成 (score, id) 列表
/// 加入 hybrid_search 的 RRF 融合。正交表示与 FTS/BM25 的词汇统计互补,
/// 提升检索语义多样性。纯增量: 无 Walsh 索引时返回空, 不影响既有融合。
fn build_walsh_ranklist(
    _conn: &Connection,
    query: &str,
    fts_results: &[SearchResult],
    limit: usize,
) -> Vec<(f64, String)> {
    use crate::l4_emotion::nt_feel_facade::WalshMemoryIndex;

    if fts_results.is_empty() {
        return Vec::new();
    }
    let walsh = WalshMemoryIndex::new();
    let query_vec = walsh.encode(query);
    if query_vec.iter().all(|x| *x == 0.0) {
        return Vec::new();
    }

    let mut scored: Vec<(f64, String)> = Vec::new();
    for r in fts_results.iter().take(limit) {
        // 用 title + summary 作为文档表示 (避免 content 过长)
        let doc_text = format!(
            "{} {}",
            r.node.title,
            r.node.summary.as_deref().unwrap_or("")
        );
        let doc_vec = walsh.encode(&doc_text);
        let sim = cosine_similarity_f64(&query_vec, &doc_vec);
        if sim > 0.0 {
            scored.push((sim, r.node.id.clone()));
        }
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Build a proxy query embedding by averaging stored embeddings of nodes
/// whose title or content matches query words.
fn query_to_avg_embedding(query: &str, all_embeddings: &[(String, Vec<f32>)]) -> Vec<f32> {
    if all_embeddings.is_empty() {
        return Vec::new();
    }
    let dim = all_embeddings[0].1.len();
    let q = query.to_lowercase();
    let matching: Vec<&[f32]> = all_embeddings
        .iter()
        .filter(|(id, _)| id.to_lowercase().contains(&q))
        .map(|(_, emb)| emb.as_slice())
        .collect();
    if matching.is_empty() {
        return vec![0.0_f32; dim];
    }
    let sum: Vec<f32> = (0..dim)
        .map(|i| matching.iter().map(|e| e[i]).sum::<f32>())
        .collect();
    let n = matching.len() as f32;
    sum.into_iter().map(|v| v / n).collect()
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_fts5_config_defaults() {
        let cfg = super::Fts5OptimizerConfig::default();
        assert_eq!(cfg.cache_size, -256000);
        assert_eq!(cfg.synchronous, "NORMAL");
        assert!(cfg.auto_analyze);
    }

    #[test]
    fn test_contentless_schema_const() {
        let schema = super::Fts5OptimizerConfig::CONTENTLESS_FTS_SCHEMA;
        assert!(schema.contains("fts5"));
        assert!(schema.contains("content='nodes'"));
    }

    #[test]
    fn test_cosine_similarity_f64() {
        // 相同向量 → 1.0
        let a = vec![1.0, 0.0, 0.0];
        assert!((super::cosine_similarity_f64(&a, &a) - 1.0).abs() < 1e-9);
        // 正交向量 → 0.0
        let b = vec![0.0, 1.0, 0.0];
        assert!(super::cosine_similarity_f64(&a, &b).abs() < 1e-9);
        // 空向量 → 0.0
        assert_eq!(super::cosine_similarity_f64(&[], &[]), 0.0);
        // 长度不同 → 0.0
        assert_eq!(super::cosine_similarity_f64(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_build_walsh_ranklist_empty_fts() {
        // 无 FTS 结果 → 空 ranklist (不阻塞融合)
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let empty: Vec<super::SearchResult> = Vec::new();
        let rank = super::build_walsh_ranklist(&conn, "query", &empty, 10);
        assert!(rank.is_empty());
    }

    #[test]
    fn test_walsh_ranklist_ranks_similar_higher() {
        use crate::l4_emotion::nt_feel_facade::WalshMemoryIndex;
        let walsh = WalshMemoryIndex::new();
        // 语义相似文档应比不相关文档得分更高
        let q = walsh.encode("neural network training");
        let similar = walsh.encode("neural network training methods");
        let unrelated = walsh.encode("cooking recipes pasta");
        let sim = super::cosine_similarity_f64(&q, &similar);
        let unrel = super::cosine_similarity_f64(&q, &unrelated);
        assert!(
            sim > unrel,
            "相似文档应得分更高: sim={} unrel={}",
            sim,
            unrel
        );
    }

    /// 缺陷7 回归测试 (真实运转): hybrid_search 的 RRF 融合只按排名位置融合,
    /// 丢弃 search_fts 的标题加权分数 — 标题精确匹配的"本体"节点 (如《史记》book)
    /// 被内容高频引用它的 concept 节点挤到后面。修复: 融合后对 FtsTitle 精确匹配节点加权。
    #[test]
    fn test_hybrid_search_preserves_title_exact_match() {
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (
                id TEXT PRIMARY KEY, node_type TEXT, title TEXT, summary TEXT,
                content TEXT, url TEXT, domain TEXT, language TEXT,
                confidence REAL, importance REAL, created_at INTEGER,
                updated_at INTEGER, access_count INTEGER, metadata TEXT
            );
            CREATE VIRTUAL TABLE nodes_fts USING fts5(title, summary, content, domain);",
        )
        .unwrap();
        // 本体: 标题精确匹配"史记" (FtsTitle)
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, summary, content, domain, language,
                                confidence, importance, created_at, updated_at, access_count, metadata)
             VALUES ('book-shiji', 'book', '史记', '史记 司马迁 本纪 列传',
                     '史记 司马迁 本纪 列传 世家 表 书 八书 十表 十二本纪 三十世家',
                     'guji', 'zh', 0.9, 0.9, 1, 1, 0, NULL)",
            [],
        ).unwrap();
        // 引用者: 标题含"史记"但内容高频重复 (BM25/内容命中会把它排前)
        let spam = "史记 ".repeat(200);
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, summary, content, domain, language,
                                confidence, importance, created_at, updated_at, access_count, metadata)
             VALUES ('concept-shiji', 'concept', '史记——史家之绝唱', '评论',
                     ?1, 'guji', 'zh', 0.5, 0.5, 2, 2, 0, NULL)",
            [spam],
        ).unwrap();
        // 同步 FTS (content 亦入索引)
        conn.execute_batch(
            "INSERT INTO nodes_fts(rowid, title, summary, content, domain)
             SELECT rowid, title, summary, content, domain FROM nodes;",
        )
        .unwrap();

        let results = super::hybrid_search(&conn, "史记", 5, None).unwrap();
        assert!(!results.is_empty(), "应检索到结果");
        let first_title = results[0].node.title.clone();
        assert_eq!(
            first_title,
            "史记",
            "标题精确匹配的本体应排第一, 实际: {} | {:?}",
            first_title,
            results
                .iter()
                .map(|r| format!("{}[{:.2}]", r.node.title, r.score))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_evolver_evaluate_and_window_mean() {
        let mut ev = super::RetrievalEvolver::new();
        ev.evaluate("rust 生命周期", 10, 0.8);
        ev.evaluate("async", 5, 0.6);
        assert_eq!(ev.window_len(), 2);
        let mean = ev.window_mean().unwrap();
        assert!((mean - 0.7).abs() < 1e-9);
        // 窗口滚动上限
        for i in 0..300 {
            ev.evaluate(&format!("q{i}"), 1, 0.5);
        }
        assert!(ev.window_len() <= 128);
    }

    #[test]
    fn test_evolver_diagnose_flags_degraded_classes() {
        let mut ev = super::RetrievalEvolver::new();
        // 大量空结果查询 → empty 类退化
        for _ in 0..10 {
            ev.evaluate("extremely obscure term xyzzy", 0, 0.0);
        }
        for _ in 0..5 {
            ev.evaluate("常见词", 8, 0.9);
        }
        let diag = ev.diagnose();
        let empty = diag.iter().find(|d| d.class == "empty").unwrap();
        assert!(empty.degraded);
    }

    #[test]
    fn test_evolver_propose_raises_boost_on_empty_queries() {
        let mut ev = super::RetrievalEvolver::new();
        for _ in 0..10 {
            ev.evaluate("unmatched term", 0, 0.0);
        }
        let proposal = ev.propose();
        assert!(proposal.is_some(), "空结果占比高应提出调参");
        assert!(proposal.unwrap().boost > 0.0);
    }

    #[test]
    fn test_evolver_guard_monotonic_commit_and_reject() {
        let mut ev = super::RetrievalEvolver::new();
        for _ in 0..8 {
            ev.evaluate("term", 10, 0.9);
        }
        let proposal = super::RetrievalTuning {
            boost: 1.0,
            committed_at: 0,
        };
        assert!(ev.guard(&proposal), "窗口均值高应接受调参");
        assert_eq!(ev.recall_boost(), 1.0);
        // 之后检索质量下滑 → 新调参被拒绝, 保留已提交的 boost
        for _ in 0..8 {
            ev.evaluate("noise", 0, 0.05);
        }
        let proposal2 = super::RetrievalTuning {
            boost: 2.0,
            committed_at: 0,
        };
        assert!(!ev.guard(&proposal2), "均值下滑应拒绝");
        assert_eq!(ev.recall_boost(), 1.0, "拒绝后保留原 tuning");
    }

    #[test]
    fn test_evolver_recall_boost_clamped() {
        let mut ev = super::RetrievalEvolver::new();
        ev.tuning.boost = 99.0;
        assert_eq!(ev.recall_boost(), 4.0);
        ev.tuning.boost = -99.0;
        assert_eq!(ev.recall_boost(), -2.0);
    }
}

#[cfg(test)]
mod precision_gate_tests {
    use super::super::nt_gate_optimizer::{precision_gate, staleness_signal};
    use super::super::nt_pure_fns::decay_factor;
    use super::*;
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::KnowledgeNode;

    fn node(id: &str) -> KnowledgeNode {
        KnowledgeNode {
            id: id.into(),
            node_type:
                crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Concept,
            title: id.into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".to_string(),
            recall_weight: 1.0,
            confidence: 0.0,
            importance: 0.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
        }
    }

    fn hit(id: &str, score: f64) -> SearchResult {
        SearchResult {
            node: node(id),
            score,
            matched_on: vec![SearchMatchType::Bm25],
            signals: None,
        }
    }

    #[test]
    fn test_gate_empty_returns_empty() {
        assert!(precision_gate(vec![], 100).is_empty());
    }

    #[test]
    fn test_small_pool_keeps_all() {
        let results = vec![hit("a", 0.9), hit("b", 0.5)];
        let gated = precision_gate(results.clone(), 5);
        assert_eq!(gated.len(), 2);
    }

    #[test]
    fn test_large_pool_drops_low_scores() {
        // pool=200 → log2(201)≈7.65 → ratio≈0.25+0.765=1.01→min(0.9)=0.9
        // top=1.0 → cutoff=0.9 → 0.5 被丢弃
        let results = vec![hit("a", 1.0), hit("b", 0.5)];
        let gated = precision_gate(results, 200);
        assert_eq!(gated.len(), 1);
        assert_eq!(gated[0].node.id, "a");
    }

    #[test]
    fn test_pool_size_monotonicity() {
        // 池越大保留的越少
        let results = vec![hit("a", 1.0), hit("b", 0.8), hit("c", 0.7), hit("d", 0.6)];
        let small = precision_gate(results.clone(), 10).len();
        let large = precision_gate(results.clone(), 500).len();
        assert!(large <= small, "池越大门槛越高");
    }

    #[test]
    fn test_gate_preserves_order() {
        // pool=10 → log2(11)≈3.46 → ratio≈0.596 → cutoff≈0.596: a/b 保留, c 丢弃, 顺序不变
        let results = vec![hit("a", 1.0), hit("b", 0.7), hit("c", 0.3)];
        let gated = precision_gate(results, 10);
        let ids: Vec<&str> = gated.iter().map(|r| r.node.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    // ========== codegraph 陈旧信号标注接线测试 (2026-08-19) ==========

    #[test]
    fn test_staleness_signal_fills_reserved_slot() {
        let mut n = node("old");
        n.updated_at = chrono::Utc::now().timestamp() - 90 * 86400; // 90 天前
        let results = vec![SearchResult {
            node: n,
            score: 1.0,
            matched_on: vec![SearchMatchType::Bm25],
            signals: None,
        }];
        let annotated = staleness_signal(results);
        assert!(annotated[0].signals.is_some(), "signals 槽必须被填充");
        let sig = annotated[0].signals.unwrap();
        assert_eq!(sig[3], 1.0, ">30 天应打 stale 横幅");
        assert!(sig[0] < 0.5, "90 天 ≈ 12.8 半衰期, decay 应接近 0");
        assert!(sig[1] >= 89.0, "age_days 约 90");
    }

    #[test]
    fn test_staleness_fresh_node_no_banner() {
        let mut n = node("fresh");
        n.updated_at = chrono::Utc::now().timestamp(); // 刚刚更新
        let results = vec![SearchResult {
            node: n,
            score: 1.0,
            matched_on: vec![SearchMatchType::Bm25],
            signals: None,
        }];
        let annotated = staleness_signal(results);
        let sig = annotated[0].signals.unwrap();
        assert_eq!(sig[3], 0.0, "新节点不打横幅");
        assert!((sig[0] - 1.0).abs() < 1e-9, "age=0 → decay=1.0");
    }

    #[test]
    fn test_staleness_signal_uses_existing_updated_at() {
        let mut n = node("half");
        n.updated_at = chrono::Utc::now().timestamp() - 7 * 24 * 3600; // 恰 1 个半衰期
        let results = vec![SearchResult {
            node: n,
            score: 0.5,
            matched_on: vec![SearchMatchType::FtsTitle],
            signals: None,
        }];
        let annotated = staleness_signal(results);
        let sig = annotated[0].signals.unwrap();
        assert!((sig[0] - 0.5).abs() < 1e-6, "1 半衰期 → decay=0.5");
        assert_eq!(sig[3], 0.0, "7 天 < 30 天, 不打横幅");
    }

    // ========== agentmemory 接线测试 (cycle 1188: confidence × 时间衰减) ==========

    #[test]
    fn test_decay_factor_half_life() {
        let half_life = 7 * 24 * 3600; // 7 天
                                       // age=0 → 1.0 (全新)
        assert!((decay_factor(0, half_life) - 1.0).abs() < 1e-9);
        // age=half_life → 0.5 (一个半衰期)
        assert!((decay_factor(half_life, half_life) - 0.5).abs() < 1e-9);
        // age=2*half_life → 0.25 (两个半衰期)
        assert!((decay_factor(2 * half_life, half_life) - 0.25).abs() < 1e-9);
        // 负 age (时钟回拨) → 1.0
        assert_eq!(decay_factor(-5, half_life), 1.0);
        // half_life=0 → 1.0
        assert_eq!(decay_factor(100, 0), 1.0);
    }

    #[test]
    fn test_decay_factor_monotonic_decreasing() {
        let half_life = 3600;
        let older = decay_factor(7200, half_life);
        let newer = decay_factor(3600, half_life);
        assert!(older < newer, "越旧衰减越强, 分数越低");
    }
}

// ========== Phase 2: 物化邻居缓存 + p95 延迟基准测试 ==========
// 不依赖真实 389K DB: 在测试内生成合成向量 (确定性 LCG), 构建缓存并测 p95 查询延迟。

#[cfg(test)]
mod materialized_neighbors_tests {
    use super::super::nt_cache::MaterializedNeighborCache;

    /// 确定性伪随机向量生成 (LCG), 保证测试可复现。
    fn synth_embeddings(count: usize, dim: usize) -> Vec<(String, Vec<f32>)> {
        let mut state: u64 = 0x9E3779B97F4A7C15;
        let mut next = || {
            // xorshift64
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let v: Vec<f32> = (0..dim)
                .map(|_| {
                    let raw = next();
                    ((raw as f64 / u64::MAX as f64) as f32) - 0.5
                })
                .collect();
            out.push((format!("node-{i}"), v));
        }
        out
    }

    #[test]
    fn test_cosine_f32_basic() {
        let a = vec![1.0_f32, 0.0, 0.0];
        assert!((super::cosine_f32(&a, &a) - 1.0).abs() < 1e-6);
        let b = vec![0.0_f32, 1.0, 0.0];
        assert!(super::cosine_f32(&a, &b).abs() < 1e-6);
        // 长度不匹配 → 0.0
        assert_eq!(super::cosine_f32(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_materialized_cache_get_is_consistent() {
        // 小库: top-1 邻居应为彼此最相似的节点 (自身除外)。
        let emb = synth_embeddings(50, 32);
        let cache = MaterializedNeighborCache::build(&emb, 5);
        assert_eq!(cache.len(), 50);
        // 每个节点的 top-1 邻居与该节点余弦应为所有其他节点中最大。
        for (i, (id, v)) in emb.iter().enumerate() {
            let top = cache.get(id).expect("应有物化邻居");
            assert!(!top.is_empty(), "节点 {} 应有邻居", i);
            // 验证 top[0] 确实是真正的近邻: 对所有 j != i 求最大余弦
            let mut best_j = 0usize;
            let mut best_s = f32::MIN;
            for j in 0..emb.len() {
                if j == i {
                    continue;
                }
                let s = super::cosine_f32(v, &emb[j].1);
                if s > best_s {
                    best_s = s;
                    best_j = j;
                }
            }
            assert_eq!(top[0].0, emb[best_j].0, "top-1 应为真实近邻");
        }
    }

    #[test]
    fn test_materialized_search_uses_buffer_reuse() {
        let emb = synth_embeddings(200, 32);
        let cache = MaterializedNeighborCache::build(&emb, 10);
        let mut buf = Vec::new();
        // 查询匹配已缓存节点 → 直接查表, 跳过扫描
        let r1 = cache.search(&emb[3].1, Some("node-3"), &emb, 10, &mut buf);
        assert_eq!(r1.len(), 10);
        // 未命中节点 → 单次扫描
        let q = vec![0.1_f32; 32];
        let r2 = cache.search(&q, None, &emb, 10, &mut buf);
        assert_eq!(r2.len(), 10);
        assert!(r2[0].1 >= r2[1].1, "应按相似度降序");
    }

    /// 基准: 合成 10K 向量 (dim=64) 构建物化缓存, 测量相似查询 p95 延迟。
    /// 目标 sanity: 单次查询 (10K 向量全扫) 远低于 Phase-1 269ms 量级 (纯扫描 < 5ms)。
    #[ignore = "基准测试(10k x 64d 物化邻居缓存 + 200 次查询), 默认套件不该跑; 用 cargo bench 或 --ignored"]
    fn bench_p95_search_latency_10k() {
        const N: usize = 10_000;
        const DIM: usize = 64;
        const K: usize = 10;
        let emb = synth_embeddings(N, DIM);
        let cache = MaterializedNeighborCache::build(&emb, K);

        const QUERIES: usize = 200;
        let mut latencies: Vec<f64> = Vec::with_capacity(QUERIES);
        let mut buf = Vec::new();
        for i in 0..QUERIES {
            let q = &emb[i % N].1;
            let start = std::time::Instant::now();
            let res = cache.search(q, None, &emb, K, &mut buf);
            let el = start.elapsed().as_secs_f64() * 1000.0;
            latencies.push(el);
            assert_eq!(res.len(), K, "每次查询返回 top-k");
        }
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = latencies[(QUERIES as f64 * 0.95) as usize];
        log::info!(
            "[bench] 10K 向量, dim={DIM}: median={:.3}ms p95={:.3}ms (queries={QUERIES})",
            latencies[QUERIES / 2],
            p95
        );
        // sanity gate: 单次扫描查询应在合理延迟内 (远高于 HNSW 目标但远优于 269ms 暴力)
        assert!(
            p95 < 50.0,
            "p95 查询延迟应远低于 Phase-1 基线, 实际 {p95:.3}ms"
        );
        // 缓存命中 (已建节点) 路径应更便宜 (O(k) 查表)
        let mut hit_lat: Vec<f64> = Vec::with_capacity(QUERIES);
        for i in 0..QUERIES {
            let start = std::time::Instant::now();
            let _ = cache.search(
                &emb[i % N].1,
                Some(&format!("node-{}", i % N)),
                &emb,
                K,
                &mut buf,
            );
            hit_lat.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        hit_lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let hit_p95 = hit_lat[(QUERIES as f64 * 0.95) as usize];
        log::info!("[bench] 缓存命中路径 p95={:.4}ms", hit_p95);
        assert!(hit_p95 < p95, "缓存命中应快于全扫描");
    }
}

#[cfg(test)]
mod selftest_tests {
    use crate::l0_substrate::nt_core_self_test::SelfTest;

    #[test]
    fn test_nt_memory_search_self_test_passes() {
        let t = super::MaterializedNeighborCacheSelfTest;
        assert!(
            t.self_test().is_ok(),
            "MaterializedNeighborCacheSelfTest failed: {:?}",
            t.self_test().err()
        );
    }
}
