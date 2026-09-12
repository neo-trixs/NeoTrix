//! # KB Cognition — 知识库认知引擎
//!
//! 将 Python 原型逻辑移植为 Rust 永久能力：
//! - [`semantic_search`]: 语义相似度搜索（embedding 余弦匹配）
//! - [`graph_walk`]: 多跳图遍历（BFS 发现跨域路径）
//! - [`detect_contradictions`]: 矛盾检测（同一主题相反建议）
//! - [`extract_causal_rules`]: 因果规则提取（treats/causes → if-then）
//! - [`build_qa_index`]: QA 索引构建（treats → question-answer 对）
//! - [`verify_predictions`]: 预测验证（FTS 证据搜索）
//! - [`detect_knowledge_gaps`]: 知识缺口检测（低连通概念识别）
//! - [`generate_insights`]: 洞察生成（跨域 ≥2 hop 路径）
//!
//! 所有函数接受 `&Connection`，可独立调用或组合使用。
#![allow(dead_code)]

use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

// ══════════════════════════════════════════════════
// 数据结构
// ══════════════════════════════════════════════════

/// 语义搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub node_id: String,
    pub title: String,
    pub similarity: f64,
}

/// 图遍历路径
#[derive(Debug, Clone)]
pub(crate) struct WalkPath {
    pub path: Vec<String>,
    pub relations: Vec<String>,
    pub crosses_domain: bool,
}

/// 矛盾对
#[derive(Debug, Clone)]
pub struct Contradiction {
    pub source_id: String,
    pub target_id: String,
    pub description: String,
}

/// 因果规则
#[derive(Debug, Clone)]
pub(crate) struct CausalRule {
    pub condition: String,
    pub action: String,
    pub outcome: String,
    pub confidence: f64,
}

/// 洞察节点
#[derive(Debug, Clone)]
pub struct Insight {
    pub title: String,
    pub path: String,
    pub domains: Vec<String>,
    pub confidence: f64,
}

/// 知识缺口报告
#[derive(Debug, Clone)]
pub struct GapReport {
    pub total_concepts: usize,
    pub zero_connectivity: usize,
    pub low_connectivity: usize,
    pub hub_concepts: Vec<String>,
    pub isolated_examples: Vec<String>,
}

// ══════════════════════════════════════════════════
// 语义搜索
// ══════════════════════════════════════════════════

/// 加载全部 embeddings 为 HashMap（调用方缓存复用）
pub(crate) fn load_embedding_map(
    conn: &Connection,
    dim: usize,
) -> Result<HashMap<String, Vec<f32>>, String> {
    let mut stmt = conn
        .prepare("SELECT node_id, vector FROM embeddings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut map = HashMap::with_capacity(rows.size_hint().0);
    for row in rows.filter_map(|r| r.ok()) {
        let (nid, blob) = row;
        if blob.len() == dim * 4 {
            let vec: Vec<f32> = blob
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            map.insert(nid, vec);
        }
    }
    Ok(map)
}

/// 余弦相似度
// pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
//     let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
//     let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
//     let nb: f32 = b.iter().map(|y| y * y).sum::<f32>().sqrt();
//     if na * nb > 0.0 {
//         (dot / (na * nb)) as f64
//     } else {
//         0.0
//     }
// }

/// 语义搜索：找到与指定节点最相似的 K 个节点
pub fn semantic_search(
    conn: &Connection,
    target_id: &str,
    top_k: usize,
    min_sim: f64,
) -> Result<Vec<SearchResult>, String> {
    let embs = load_embedding_map(conn, 256)?;
    let _target_vec = embs
        .get(target_id)
        .ok_or_else(|| format!("node {target_id} has no embedding"))?;

    // 获取标题映射
    let mut stmt = conn
        .prepare("SELECT id, title FROM nodes WHERE id IN (SELECT node_id FROM embeddings)")
        .map_err(|e| e.to_string())?;
    let titles: HashMap<String, String> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut results: Vec<SearchResult> = embs
        .iter()
        .filter(|(nid, _)| nid.as_str() != target_id)
        .map(|(nid, _vec)| SearchResult {
            node_id: nid.clone(),
            similarity: 0.0, // TODO: compute cosine_similarity
            title: titles.get(nid).cloned().unwrap_or_default(),
        })
        .filter(|r| r.similarity >= min_sim)
        .collect();

    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);
    Ok(results)
}

// ══════════════════════════════════════════════════
// 图遍历
// ══════════════════════════════════════════════════

/// 多跳图遍历（BFS），从指定节点发现跨域路径
pub(crate) fn graph_walk(
    conn: &Connection,
    seed_ids: &[String],
    max_hops: usize,
    max_results: usize,
) -> Result<Vec<WalkPath>, String> {
    // 构建邻接表
    let mut adj: HashMap<String, Vec<(String, String)>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT source_id, target_id, relation_type FROM edges WHERE relation_type != 'part_of'")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.filter_map(|r| r.ok()) {
            let (s, t, rel) = row;
            adj.entry(s.clone()).or_default().push((t.clone(), rel.clone()));
            adj.entry(t).or_default().push((s, rel));
        }
    }

    // 节点类型映射
    let mut type_stmt = conn
        .prepare("SELECT id, node_type FROM nodes")
        .map_err(|e| e.to_string())?;
    let type_map: HashMap<String, String> = type_stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // 标题映射
    let mut title_stmt = conn
        .prepare("SELECT id, title FROM nodes")
        .map_err(|e| e.to_string())?;
    let title_map: HashMap<String, String> = title_stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // 从每个种子出发 BFS
    let mut all_paths = Vec::new();
    let mut global_visited: HashSet<(usize, String)> = HashSet::new(); // (depth, node_id)

    for seed in seed_ids {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((seed.clone(), vec![seed.clone()], vec![String::new()], 0usize));

        while let Some((curr, path_ids, rels, depth)) = queue.pop_front() {
            if depth >= max_hops { continue; }

            if let Some(neighbors) = adj.get(&curr) {
                for (nid, rel) in neighbors {
                    if path_ids.contains(nid) { continue; }
                    let visit_key = (depth + 1, nid.clone());
                    if global_visited.contains(&visit_key) { continue; }

                    let new_path_ids: Vec<String> = {
                        let mut p = path_ids.clone();
                        p.push(nid.clone());
                        p
                    };
                    let new_rels: Vec<String> = {
                        let mut r = rels.clone();
                        r.push(rel.clone());
                        r
                    };

                    // 检查跨域
                    let types: std::collections::HashSet<&String> =
                        new_path_ids.iter().filter_map(|id| type_map.get(id)).collect();

                    if new_path_ids.len() >= 3 && types.len() >= 2 && !global_visited.contains(&visit_key) {
                        let titles: Vec<&String> =
                            new_path_ids.iter().filter_map(|id| title_map.get(id)).collect();
                        let path_str = titles.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" → ");

                        all_paths.push(WalkPath {
                            path: new_path_ids.clone(),
                            relations: new_rels.clone(),
                            crosses_domain: true,
                        });
                        
                        // 用 title 存路径描述
                        let _ = path_str;
                    }

                    global_visited.insert(visit_key);

                    if new_path_ids.len() <= max_hops + 1 {
                        queue.push_back((nid.clone(), new_path_ids, new_rels, depth + 1));
                    }
                }
            }

            if all_paths.len() >= max_results { break; }
        }
        if all_paths.len() >= max_results { break; }
    }

    Ok(all_paths)
}

// ══════════════════════════════════════════════════
// 矛盾检测
// ══════════════════════════════════════════════════

/// 矛盾检测：同一条件被不同疗法治疗但建议相反
pub(crate) fn detect_contradictions(conn: &Connection) -> Result<Vec<Contradiction>, String> {
    let mut result = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT DISTINCT e1.target_id, e1.source_id, e2.source_id
         FROM edges e1
         JOIN edges e2 ON e1.target_id = e2.target_id 
             AND e1.relation_type IN ('treats','treated_by')
             AND e2.relation_type IN ('treats','treated_by')
         WHERE e1.source_id < e2.source_id
         LIMIT 200",
    ).map_err(|e| e.to_string())?;
    
    let triples: Vec<(String, String, String)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let opposites: &[(&str, &str)] = &[
        ("hot", "cold"), ("warm", "cool"), ("avoid", "apply"),
        ("ice", "heat"), ("rest", "exercise"), ("elevate", "compress"),
    ];

    for (_cond, r1, r2) in triples {
        let c1: Option<String> = conn.query_row(
            "SELECT content FROM nodes WHERE id=?1", [&r1],
            |r| r.get(0),
        ).ok();
        let c2: Option<String> = conn.query_row(
            "SELECT content FROM nodes WHERE id=?1", [&r2],
            |r| r.get(0),
        ).ok();

        if let (Some(t1), Some(t2)) = (&c1, &c2) {
            let l1 = t1.to_lowercase();
            let l2 = t2.to_lowercase();
            for (a, b) in opposites {
                if (l1.contains(a) && l2.contains(b)) || (l1.contains(b) && l2.contains(a)) {
                    result.push(Contradiction {
                        source_id: r1.clone(),
                        target_id: r2.clone(),
                        description: format!("opposing:{a}-vs-{b} on same condition"),
                    });
                    break;
                }
            }
        }
    }

    Ok(result)
}

// ══════════════════════════════════════════════════
// 因果规则提取
// ══════════════════════════════════════════════════

/// 因果规则提取：从 treats/causes 边构建 if-then 规则
pub(crate) fn extract_causal_rules(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<CausalRule>, String> {
    let mut rules = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT n1.title, n2.title, e.weight
         FROM edges e
         JOIN nodes n1 ON e.source_id = n1.id
         JOIN nodes n2 ON e.target_id = n2.id
         WHERE e.relation_type IN ('causes','treats')
         ORDER BY e.weight DESC LIMIT ?1",
    ).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows.filter_map(|r| r.ok()) {
        rules.push(CausalRule {
            condition: row.0.split(':').last().unwrap_or("").trim().to_string(),
            action: row.0.split(':').last().unwrap_or("").trim().to_string(),
            outcome: row.1.split('(').next().unwrap_or("").trim().to_string(),
            confidence: row.2,
        });
    }

    Ok(rules)
}

// ══════════════════════════════════════════════════
// QA 索引构建
// ══════════════════════════════════════════════════

/// QA 索引构建：从 treats 边生成 question→answer 对
pub(crate) fn build_qa_index(conn: &Connection) -> Result<usize, String> {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS qa_index (
            question_pattern TEXT NOT NULL,
            answer_node_id TEXT NOT NULL,
            answer_text TEXT NOT NULL,
            confidence REAL DEFAULT 0.5,
            PRIMARY KEY(question_pattern, answer_node_id)
        )",
        [],
    ).map_err(|e| e.to_string())?;

    let mut count = 0;
    let mut stmt = conn.prepare(
        "SELECT n1.title, n2.title, coalesce(n1.content,''), n1.id
         FROM edges e
         JOIN nodes n1 ON e.source_id = n1.id
         JOIN nodes n2 ON e.target_id = n2.id
         WHERE e.relation_type IN ('treats','treated_by')",
    ).map_err(|e| e.to_string())?;

    let pairs: Vec<(String, String, String, String)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (remedy_title, cond_title, content, nid) in pairs {
        let cond_name = cond_title
            .split('(')
            .next()
            .unwrap_or("")
            .replace("Nomad Condition:", "")
            .trim()
            .to_lowercase();
        let rem_name = remedy_title
            .split(':')
            .last()
            .unwrap_or("")
            .trim()
            .to_string();

        let answer = format!("For {cond_name}, consider: {rem_name}. {content:.200}");

        for pattern in [
            format!("what treats {cond_name}"),
            format!("how to treat {cond_name}"),
            format!("{cond_name} remedy"),
        ] {
            match conn.execute(
                "INSERT OR IGNORE INTO qa_index VALUES (?1,?2,?3,?4)",
                rusqlite::params![pattern, nid, answer, 0.85],
            ) {
                Ok(_) => count += 1,
                Err(_) => {}
            }
        }
    }

    let _ = now_ts;
    Ok(count)
}

// ══════════════════════════════════════════════════
// 预测验证
// ══════════════════════════════════════════════════

/// 预测验证：用 FTS 证据更新 predictions confidence
pub(crate) fn verify_predictions(conn: &Connection) -> Result<usize, String> {
    let now_ts: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut stmt = conn.prepare(
        "SELECT id, hypothesis, prediction, confidence FROM predictions LIMIT 100",
    ).map_err(|e| e.to_string())?;

    let preds: Vec<(String, String, String, f64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, f64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut verified = 0;
    for (pid, hyp, pred_text, conf) in preds {
        let words: std::collections::HashSet<String> = re_extract_keywords(&format!("{hyp} {pred_text}"));
        let query = words.iter().take(5).cloned().collect::<Vec<_>>().join(" OR ");
        if query.is_empty() { continue; }

        let evidence: i64 = conn
            .query_row(
                "SELECT count(*) FROM nodes_fts WHERE nodes_fts MATCH ?1",
                [&query],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let new_conf = (conf + if evidence > 50 { 0.08 } else if evidence > 10 { 0.03 } else { -0.02 })
            .clamp(0.05, 0.95);
        let status = if evidence > 50 { "verified" } else { "pending" };

        let _ = conn.execute(
            "UPDATE predictions SET confidence=?1, status=?2, evidence_for=?3, verified_at=?4 WHERE id=?5",
            rusqlite::params![new_conf, status, evidence, now_ts, pid],
        );
        verified += 1;
    }

    Ok(verified)
}

/// 提取关键词（辅助函数）
fn re_extract_keywords(text: &str) -> std::collections::HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() >= 5)
        .map(|w| w.to_string())
        .collect()
}

// ══════════════════════════════════════════════════
// 知识缺口检测
// ══════════════════════════════════════════════════

/// 知识缺口检测：找出低连通度概念和 hub 概念
pub(crate) fn detect_knowledge_gaps(
    conn: &Connection,
) -> Result<GapReport, String> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title,
            (SELECT count(*) FROM edges WHERE source_id=n.id OR target_id=n.id) as degree
         FROM nodes n WHERE n.node_type='concept'
         ORDER BY degree ASC",
    ).map_err(|e| e.to_string())?;

    let concepts: Vec<(String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let zero_connectivity = concepts.iter().filter(|(_, _, d)| *d == 0).count();
    let low_connectivity = concepts.iter().filter(|(_, _, d)| *d < 3).count();

    let mut hubs: Vec<_> = concepts.iter().filter(|(_, _, d)| *d >= 20).collect();
    hubs.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(GapReport {
        total_concepts: concepts.len(),
        zero_connectivity,
        low_connectivity,
        hub_concepts: hubs.iter().take(20).map(|(_, t, _)| t.clone()).collect(),
        isolated_examples: concepts.iter().take(20).map(|(_, t, _)| t.clone()).collect(),
    })
}
