//! 检索后处理: recency 时间衰减 (D1) + MMR 多样性去冗余 (D9)
//!
//! 参照: supermemory (时间感知检索区分当前/过去) + the-librarian (MMR + recency decay
//! + brainstorm wildcards)。两个缺陷共用同一个后处理管线, 挂在 hybrid_search 输出端。
//! 纯函数设计便于无 DB 单测。

use super::nt_memory_embed::cosine_similarity;
use super::nt_memory_types::{KnowledgeNode, SearchResult};

/// 当前 unix 秒 — 供 recency 计算用基准
pub fn now_unix_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// 时间衰减半衰期 (秒) — 约 30 天。超过后旧文档相关度减半。
const RECENCY_HALF_LIFE_SECS: f64 = 30.0 * 24.0 * 3600.0;

/// MMR lambda — 1.0 纯相关度, 0.0 纯多样性。默认偏向相关度。
const MMR_LAMBDA: f64 = 0.7;

/// 按 recency 对结果重排 (D1): 相关度 × 时间衰减。
/// `now_secs` 为基准时间 (unix 秒)。分数越高越新且相关。
pub fn apply_recency_decay(mut results: Vec<SearchResult>, now_secs: i64) -> Vec<SearchResult> {
    for r in results.iter_mut() {
        let age = (now_secs.saturating_sub(r.node.created_at)).max(0) as f64;
        let decay = 0.5f64.powf(age / RECENCY_HALF_LIFE_SECS);
        r.score *= decay;
    }
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// MMR 多样性重排 (D9): 贪心选择最大化 [λ·相关度 − (1−λ)·与已选的最大相似度]。
/// `similarity(a, b)` 由调用方提供 (可用 VSA/embedding 余弦), 缺省 0 → 纯相关度排序。
/// 返回 top-k 且去冗余的结果。
pub fn diversify_mmr<F>(results: Vec<SearchResult>, k: usize, similarity: F) -> Vec<SearchResult>
where
    F: Fn(&KnowledgeNode, &KnowledgeNode) -> f64,
{
    if results.is_empty() || k == 0 {
        return Vec::new();
    }
    let mut selected: Vec<SearchResult> = Vec::new();
    let mut pool: Vec<SearchResult> = results;

    while !pool.is_empty() && selected.len() < k {
        let mut best_idx = 0;
        let mut best_val = f64::NEG_INFINITY;
        for (i, cand) in pool.iter().enumerate() {
            let rel = cand.score;
            let max_sim = selected.iter()
                .map(|s| similarity(&s.node, &cand.node))
                .fold(0.0f64, f64::max);
            let val = MMR_LAMBDA * rel - (1.0 - MMR_LAMBDA) * max_sim;
            if val > best_val {
                best_val = val;
                best_idx = i;
            }
        }
        selected.push(pool.remove(best_idx));
    }
    selected
}

/// 便捷: 融合 recency + MMR。`embeddings` 为 id → 向量 映射, 用于相似度;
/// 无向量的结果相似度按 0 处理 (退化为相关度排序)。
pub fn rerank_with_recency_and_mmr(
    results: Vec<SearchResult>,
    now_secs: i64,
    k: usize,
    embeddings: &std::collections::HashMap<String, Vec<f32>>,
) -> Vec<SearchResult> {
    let recency_sorted = apply_recency_decay(results, now_secs);
    diversify_mmr(recency_sorted, k, |a, b| {
        match (embeddings.get(&a.id), embeddings.get(&b.id)) {
            (Some(va), Some(vb)) => cosine_similarity(va, vb),
            _ => 0.0,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType;

    fn node(id: &str, created_at: i64) -> KnowledgeNode {
        KnowledgeNode {
            id: id.into(),
            node_type: NodeType::Concept,
            title: id.into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 0.9,
            importance: 0.5,
            created_at,
            updated_at: created_at,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        }
    }

    fn result(id: &str, created_at: i64, score: f64) -> SearchResult {
        SearchResult {
            node: node(id, created_at),
            score,
            matched_on: Vec::new(),
            signals: None,
        }
    }

    #[test]
    fn recency_boost_newer_when_scores_tie() {
        let now = 1_800_000_000i64;
        let old = result("old", now - 2 * 30 * 24 * 3600, 0.5);
        let new = result("new", now - 1000, 0.5);
        let mut results = vec![old, new];
        results = apply_recency_decay(results, now);
        assert_eq!(results[0].node.id, "new", "newer node must rank first on tie");
        assert!(results[0].score > results[1].score, "decay must lower old score");
    }

    #[test]
    fn recency_does_not_invert_large_gap() {
        let now = 1_800_000_000i64;
        // 一个半衰期 (30 天) 的旧文档 — decay=0.5, 高相关度仍应胜过弱新
        let strong_old = result("strong-old", now - 30 * 24 * 3600, 1.0);
        let weak_new = result("weak-new", now - 1000, 0.1);
        let mut results = vec![weak_new, strong_old];
        results = apply_recency_decay(results, now);
        assert_eq!(results[0].node.id, "strong-old", "strong relevance must survive recency decay");
    }

    #[test]
    fn mmr_drops_near_duplicate() {
        let a = result("a", 0, 0.9);
        let b = result("b", 0, 0.88);
        let c = result("c", 0, 0.5);
        // a 与 b 高度相似, 与 c 完全不同 — 多样性应把 c 提到近重复 b 之前
        let sim = |x: &KnowledgeNode, y: &KnowledgeNode| -> f64 {
            match (x.id.as_str(), y.id.as_str()) {
                ("a", "b") | ("b", "a") => 0.99,
                ("a", "c") | ("c", "a") => 0.1,
                ("b", "c") | ("c", "b") => 0.1,
                _ => 0.0,
            }
        };
        let out = diversify_mmr(vec![a, b, c], 3, sim);
        let ids: Vec<&str> = out.iter().map(|r| r.node.id.as_str()).collect();
        // 期望 c 被挤到前面 (去冗余), b 与 a 太相似被后置
        assert!(ids.iter().position(|&x| x == "c").unwrap() < ids.iter().position(|&x| x == "b").unwrap(),
            "MMR must promote diverse c before near-duplicate b");
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn mmr_empty_and_k_zero() {
        assert!(diversify_mmr(Vec::new(), 5, |_, _| 0.0).is_empty());
        let r = vec![result("a", 0, 0.5)];
        assert!(diversify_mmr(r, 0, |_, _| 0.0).is_empty());
    }
}
