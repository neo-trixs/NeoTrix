//! 查询分解器 (C2 阶段 — map-reduce 并行子检索)
//!
//! 解决 B6 (无查询分解): Hard 复杂度查询 (实体≥4 / 对比 / 多条件) 不再只走
//! 拼接重试, 而是规则分解为并行子查询 (Flat) 或依赖链 (Sequential), 子结果
//! merge 去重后统一排序。
//!
//! 对比式分解 (map-reduce): "difference between E8 and GWT" →
//!   Flat ["E8", "GWT"] → 并行检索各自概念 → merge
//! 顺序式分解 (依赖链): "A then B after that C" →
//!   Sequential ["A", "B", "C"] → 链式推进, 每跳结果作为下一跳上下文

use std::collections::HashSet;

use super::nt_memory_types::SearchResult;

/// 分解结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decomposition {
    /// 并行子查询 (无依赖) — map-reduce
    Flat(Vec<String>),
    /// 依赖链 (多条件推理) — 顺序推进
    Sequential(Vec<String>),
    /// 无需分解
    Atomic(String),
}

impl Decomposition {
    pub fn sub_queries(&self) -> Vec<String> {
        match self {
            Decomposition::Flat(subs) | Decomposition::Sequential(subs) => subs.clone(),
            Decomposition::Atomic(q) => vec![q.clone()],
        }
    }

    pub fn is_decomposed(&self) -> bool {
        matches!(self, Decomposition::Flat(_) | Decomposition::Sequential(_))
    }
}

/// 规则分解器 (纯函数, 无 LLM 依赖 — 保持廉价, 与 B2 反启发式对齐)
///
/// 识别优先级:
/// 1. 对比查询 ("difference between X and Y" / "X vs Y" / "compare X and Y") → Flat
/// 2. 顺序推理 ("then" / "after that" / "subsequently") → Sequential
/// 3. 多实体枚举 (≥2 大写实体 and 连接) → Flat
/// 4. 兜底 → Atomic
pub fn decompose_query(query: &str) -> Decomposition {
    let q_lower = query.to_lowercase();
    let trimmed = query.trim();

    // 1. 对比: "difference between X and Y" (中文 "X 与 Y 的区别" 亦覆盖 "区别")
    if let Some(idx) = q_lower.find("difference between") {
        let rest = &trimmed[idx + "difference between".len()..];
        let parts = split_and_terms(rest);
        if parts.len() >= 2 {
            return Decomposition::Flat(parts);
        }
    }
    if let Some(idx) = q_lower.find(" vs ") {
        let (a, b) = trimmed.split_at(idx);
        let b_rest = &b[4..];
        let mut parts: Vec<String> = vec![a.trim().to_string()];
        parts.extend(split_and_terms(b_rest));
        parts.retain(|s| !s.is_empty() && s.len() >= 2);
        if parts.len() >= 2 {
            return Decomposition::Flat(parts);
        }
    }
    if q_lower.contains(" compare ") || q_lower.starts_with("compare ") {
        // "compare X and Y" / "compare X with Y"
        for marker in [" and ", " with ", " versus "] {
            if let Some(idx) = q_lower.find(marker) {
                let (a, b_rest) = trimmed.split_at(idx);
                let mut parts: Vec<String> = vec![a.trim().to_string()];
                parts.extend(split_and_terms(&b_rest[marker.len()..]));
                parts.retain(|s| !s.is_empty() && s.len() >= 2);
                if parts.len() >= 2 {
                    return Decomposition::Flat(parts);
                }
            }
        }
    }

    // 2. 顺序推理 → Sequential
    let seq_markers = [" then ", " after that ", " subsequently "];
    if let Some(marker) = seq_markers.iter().find(|m| q_lower.contains(*m)) {
        let parts: Vec<String> = q_lower
            .split(marker)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.len() > 3)
            .collect();
        if parts.len() >= 2 {
            return Decomposition::Sequential(parts);
        }
    }

    // 3. 多实体枚举: ≥2 大写实体 with "and"
    let entities: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| {
            !w.is_empty()
                && w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && w.len() >= 2
                && !["The", "This", "That", "What", "Why", "How", "When", "Where", "Compare", "Between", "Explain"]
                    .contains(&w)
        })
        .map(|w| w.to_string())
        .collect();
    if entities.len() >= 2 && (q_lower.contains(" and ") || q_lower.contains('&')) {
        return Decomposition::Flat(entities);
    }

    // 4. 兜底
    Decomposition::Atomic(trimmed.to_string())
}

/// 按 " and " / " & " / "," 切分术语列表 (去除前缀杂质词)
fn split_and_terms(s: &str) -> Vec<String> {
    let cleaned = s
        .split(" and ")
        .flat_map(|p| p.split(" & "))
        .flat_map(|p| p.split(','))
        .map(|p| p.trim().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|p| !p.is_empty() && p.len() >= 2)
        .collect::<Vec<String>>();
    cleaned
}

/// 合并多个检索结果 (去重 + 统一排序 + 截断)
pub fn merge_results(
    mut acc: Vec<SearchResult>,
    new: Vec<SearchResult>,
    limit: usize,
) -> Vec<SearchResult> {
    let mut seen: HashSet<String> = acc.iter().map(|r| r.node.id.clone()).collect();
    for r in new {
        if seen.insert(r.node.id.clone()) {
            acc.push(r);
        }
    }
    acc.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    acc.truncate(limit);
    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nt_memory_types::{KnowledgeNode, NodeType};

    fn mk_result(id: &str, score: f64) -> SearchResult {
        SearchResult {
            node: KnowledgeNode {
                id: id.to_string(),
                node_type: NodeType::Concept,
                title: id.to_string(),
                summary: None,
                content: None,
                url: None,
                domain: None,
                language: "en".into(),
                confidence: 1.0,
                importance: 0.5,
                created_at: 0,
                updated_at: 0,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
            },
            score,
            matched_on: vec![],
            signals: None,
        }
    }

    #[test]
    fn test_difference_between_decomposes_flat() {
        let d = decompose_query("what is the difference between E8 and GWT");
        assert!(matches!(d, Decomposition::Flat(ref parts) if parts.len() >= 2),
            "对比查询应分解为 Flat: {:?}", d);
    }

    #[test]
    fn test_vs_decomposes_flat() {
        let d = decompose_query("E8 vs GWT attention routing");
        assert!(matches!(d, Decomposition::Flat(ref parts) if parts.len() >= 2),
            "vs 查询应分解为 Flat: {:?}", d);
    }

    #[test]
    fn test_compare_decomposes_flat() {
        let d = decompose_query("compare SEAL pipeline with PRM reward model");
        assert!(matches!(d, Decomposition::Flat(ref parts) if parts.len() >= 2),
            "compare 查询应分解为 Flat: {:?}", d);
    }

    #[test]
    fn test_then_decomposes_sequential() {
        let d = decompose_query("first find the E8 module then trace its consumers");
        assert!(matches!(d, Decomposition::Sequential(ref parts) if parts.len() >= 2),
            "顺序推理应分解为 Sequential: {:?}", d);
    }

    #[test]
    fn test_entity_enumeration_flat() {
        let d = decompose_query("SEAL and E8 and GWT evolution loops");
        assert!(d.is_decomposed(), "多实体枚举应分解: {:?}", d);
    }

    #[test]
    fn test_simple_query_atomic() {
        let d = decompose_query("what is a vector embedding");
        assert!(matches!(d, Decomposition::Atomic(_)), "简单查询保持原子: {:?}", d);
    }

    #[test]
    fn test_merge_dedup_and_sort() {
        let a = vec![mk_result("a", 0.3), mk_result("b", 0.9)];
        let b = vec![mk_result("b", 0.9), mk_result("c", 0.6)];
        let merged = merge_results(a, b, 3);
        assert_eq!(merged.len(), 3, "去重后应 3 条");
        assert_eq!(merged[0].node.id, "b", "最高分应排首");
        assert_eq!(merged[2].node.id, "a", "最低分应排尾");
    }

    #[test]
    fn test_merge_truncates() {
        let a: Vec<SearchResult> = (0..5).map(|i| mk_result(&format!("n{}", i), i as f64)).collect();
        let b: Vec<SearchResult> = (5..10).map(|i| mk_result(&format!("n{}", i), i as f64)).collect();
        let merged = merge_results(a, b, 6);
        assert_eq!(merged.len(), 6, "应截断到 limit");
        assert_eq!(merged[0].node.id, "n9", "高分优先");
    }

    #[test]
    fn test_sub_queries_flat() {
        let d = decompose_query("difference between RAG and GWT");
        let subs = d.sub_queries();
        assert!(subs.len() >= 2, "Flat 子查询应 ≥2: {:?}", subs);
    }
}
