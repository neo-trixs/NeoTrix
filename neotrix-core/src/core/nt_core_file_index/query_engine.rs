//! # QueryEngine — 三层融合查询调度器
//!
//! 接收自然语言查询 → 并行派发到 L1/L2/L3 → RRF 融合排序
//!
//! ## 调度策略
//! - 模块名 + 关键词 → 优先 L1 PathIndex（O(1) 哈希）
//! - 代码结构查询（"找 struct Foo"）→ 优先 L2 StructureIndex
//! - 内容语义查询（"哪里处理 auth"）→ 优先 L3 ContentIndex
//!
//! 默认三层并行，结果经 RRF 融合后返回 top-k。

use std::path::PathBuf;

use super::{FileQuery, ScoredFile};

/// RRF 融合系数
const RRF_K: f64 = 60.0;

/// 三层查询结果融合排序
pub fn fuse_results(
    l1: Vec<ScoredFile>,
    l2: Vec<ScoredFile>,
    l3: Vec<ScoredFile>,
    top_k: usize,
) -> Vec<ScoredFile> {
    let mut rank_map: std::collections::HashMap<PathBuf, RankAccum> =
        std::collections::HashMap::new();

    // L1: 权重 1.0
    for (pos, r) in l1.iter().enumerate() {
        let entry = rank_map.entry(r.path.clone()).or_default();
        entry.rrf_score += 1.0 / (RRF_K + pos as f64);
        entry.source_layers.push("L1");
        entry.best_score = entry.best_score.max(r.score);
        if r.snippet.is_some() {
            entry.snippet = r.snippet.clone();
            entry.line = r.line;
        }
    }

    // L2: 权重 1.2（结构信息更精确）
    for (pos, r) in l2.iter().enumerate() {
        let entry = rank_map.entry(r.path.clone()).or_default();
        entry.rrf_score += 1.2 / (RRF_K + pos as f64);
        entry.source_layers.push("L2");
        entry.best_score = entry.best_score.max(r.score);
        if r.snippet.is_some() {
            entry.snippet = r.snippet.clone();
            entry.line = r.line;
        }
    }

    // L3: 权重 1.0
    for (pos, r) in l3.iter().enumerate() {
        let entry = rank_map.entry(r.path.clone()).or_default();
        entry.rrf_score += 1.0 / (RRF_K + pos as f64);
        entry.source_layers.push("L3");
        entry.best_score = entry.best_score.max(r.score);
        if r.snippet.is_some() {
            entry.snippet = r.snippet.clone();
            entry.line = r.line;
        }
    }

    let mut fused: Vec<ScoredFile> = rank_map
        .into_iter()
        .map(|(path, acc)| {
            let layer_label = if acc.source_layers.contains(&"L2") {
                "L1+L2"
            } else if acc.source_layers.len() > 1 {
                "Fused"
            } else {
                acc.source_layers.first().unwrap_or(&"?")
            };
            ScoredFile {
                path,
                score: acc.rrf_score,
                source_layer: layer_label,
                snippet: acc.snippet,
                line: acc.line,
            }
        })
        .collect();

    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused.truncate(top_k);
    fused
}

/// 查询意图分类器
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    /// 内容语义（默认）
    Semantic,
    /// 结构（函数/类型）
    Structural,
    /// 路径/模块
    ByPath,
    /// 所有三层
    All,
}

/// 从查询关键词推断意图
pub fn classify_intent(keywords: &[String]) -> QueryIntent {
    let kw_set: std::collections::HashSet<&str> = keywords.iter().map(|s| s.as_str()).collect();

    let structural_hints = [
        "struct", "fn", "trait", "impl", "enum", "type", "const", "macro",
    ];
    let path_hints = ["module", "path", "dir", "file", "src", "in"];

    let has_structural = kw_set.iter().any(|k| structural_hints.contains(k));
    let has_path = kw_set.iter().any(|k| path_hints.contains(k));

    if has_structural && !has_path {
        QueryIntent::Structural
    } else if has_path && !has_structural {
        QueryIntent::ByPath
    } else if has_structural && has_path {
        QueryIntent::All
    } else {
        QueryIntent::Semantic
    }
}

/// 智能查询调度器
pub struct QueryEngine;

impl QueryEngine {
    /// 根据意图选择最优调度策略
    pub fn dispatch(
        intent: &QueryIntent,
        _q: &FileQuery,
        l1: &[ScoredFile],
        l2: &[ScoredFile],
        l3: &[ScoredFile],
        top_k: usize,
    ) -> Vec<ScoredFile> {
        match intent {
            QueryIntent::ByPath => {
                // 路径查询优先 L1
                let mut results = l1.to_vec();
                results.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                results.truncate(top_k);
                results
            }
            QueryIntent::Structural => {
                // 结构查询优先 L2，补充 L1
                let mut r2 = l2.to_vec();
                r2.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let existing: std::collections::HashSet<PathBuf> =
                    r2.iter().map(|r| r.path.clone()).collect();
                let mut results = r2;
                for r in l1.iter().filter(|r| !existing.contains(&r.path)) {
                    results.push(r.clone());
                }
                results.truncate(top_k);
                results
            }
            _ => fuse_results(l1.to_vec(), l2.to_vec(), l3.to_vec(), top_k),
        }
    }
}

#[derive(Debug, Default)]
struct RankAccum {
    rrf_score: f64,
    best_score: f64,
    source_layers: Vec<&'static str>,
    snippet: Option<String>,
    line: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuse_results_empty() {
        let r = fuse_results(vec![], vec![], vec![], 10);
        assert!(r.is_empty());
    }

    #[test]
    fn test_fuse_results_ranks_l1_first() {
        let l1 = vec![ScoredFile {
            path: PathBuf::from("/a.rs"),
            score: 1.0,
            source_layer: "L1:Path",
            snippet: None,
            line: None,
        }];
        let l3 = vec![ScoredFile {
            path: PathBuf::from("/b.rs"),
            score: 0.9,
            source_layer: "L3:Content",
            snippet: None,
            line: None,
        }];
        let r = fuse_results(l1, vec![], l3, 5);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_intent_classifier() {
        assert_eq!(
            classify_intent(&["find".into(), "struct".into(), "Foo".into()]),
            QueryIntent::Structural
        );
        assert_eq!(
            classify_intent(&["file".into(), "module".into(), "core".into()]),
            QueryIntent::ByPath
        );
        assert_eq!(
            classify_intent(&["auth".into(), "handler".into()]),
            QueryIntent::Semantic
        );
    }
}
