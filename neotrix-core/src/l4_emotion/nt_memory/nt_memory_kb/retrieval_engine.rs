#![forbid(unsafe_code)]

//! 检索引擎 — 统一检索管线编排器
//!
//! 核心抽象:
//! - `HybridRetriever`: BM25 + 向量 + 图谱三路融合检索
//! - `QueryDecomposer`: 复杂查询拆解为子查询
//! - `RetrievalIntent`: 查询意图分类 (fact/overview/compare/exploration)
//!
//! 设计: 纯算法层, 持有引用到各子索引 (Bm25Index / VectorAdapter / GraphCache),
//! 不持有 DB 连接。调用方通过 `HybridRetriever::new()` 组装。

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Retrieval Intent — 查询意图
// ═══════════════════════════════════════════════════════════════════

/// 查询意图分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrievalIntent {
    /// 事实查找: "Rust 的所有权规则是什么"
    Fact,
    /// 概览总结: "给我总结一下 NeoTrix 的架构"
    Overview,
    /// 比较分析: "BM25 和向量检索的优劣"
    Compare,
    /// 探索发现: "有哪些相关的开源项目"
    Exploration,
    /// 混合意图
    Hybrid,
}

impl RetrievalIntent {
    /// 从查询文本推断意图 (简单规则, 可扩展)
    pub fn classify(query: &str) -> Self {
        let q = query.to_ascii_lowercase();
        if q.contains("总结") || q.contains("概述") || q.contains("summary") || q.contains("overview") {
            Self::Overview
        } else if q.contains("比较") || q.contains("对比") || q.contains("compare") || q.contains("vs") {
            Self::Compare
        } else if q.contains("哪些") || q.contains("什么") || q.contains("推荐") || q.contains("explore") || q.contains("find") {
            Self::Exploration
        } else {
            Self::Fact
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Query Decomposer — 查询拆解
// ═══════════════════════════════════════════════════════════════════

/// 拆解后的子查询
#[derive(Debug, Clone)]
pub struct DecomposedQuery {
    pub original: String,
    pub sub_queries: Vec<String>,
    pub strategy: DecompositionStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompositionStrategy {
    /// 不拆解, 直接检索
    Direct,
    /// 按关键词拆解
    KeywordSplit,
    /// 按意图拆解 (如: "X 和 Y 的区别" → query("X 特点") + query("Y 特点") + query("X vs Y"))
    IntentSplit,
}

impl DecomposedQuery {
    /// 拆解查询
    pub fn decompose(query: &str) -> Self {
        let intent = RetrievalIntent::classify(query);

        match intent {
            RetrievalIntent::Compare => {
                // "X 和 Y 的区别" → ["X 特点", "Y 特点", "X vs Y"]
                let parts: Vec<&str> = query
                    .split(&['和', '&', ','][..])
                    .flat_map(|s| s.split(" vs "))
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() >= 2 {
                    let mut subs: Vec<String> = parts
                        .iter()
                        .map(|p| format!("{} 特点", p))
                        .collect();
                    subs.push(format!("{} vs {}", parts[0], parts[1]));
                    return Self {
                        original: query.to_string(),
                        sub_queries: subs,
                        strategy: DecompositionStrategy::IntentSplit,
                    };
                }
            }
            RetrievalIntent::Overview => {
                // 概览: 拆为 "X 核心组件" + "X 架构" + "X 设计"
                let keywords = ["核心", "架构", "组件", "设计"];
                let subs: Vec<String> = keywords
                    .iter()
                    .map(|k| format!("{} {}", query.trim_end_matches("总结").trim(), k))
                    .collect();
                return Self {
                    original: query.to_string(),
                    sub_queries: subs,
                    strategy: DecompositionStrategy::IntentSplit,
                };
            }
            _ => {}
        }

        // 默认: 直接检索
        Self {
            original: query.to_string(),
            sub_queries: vec![query.to_string()],
            strategy: DecompositionStrategy::Direct,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Hybrid Retriever — 三路融合
// ═══════════════════════════════════════════════════════════════════

/// 单条检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub node_id: String,
    pub title: String,
    pub score: f64,
    pub source: RetrievalSource,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrievalSource {
    BM25,
    Vector,
    Graph,
    Fused,
}

/// RRF (Reciprocal Rank Fusion) 融合参数
#[derive(Debug, Clone)]
pub struct RrfConfig {
    /// RRF 常数 k (通常 60)
    pub k: usize,
    /// BM25 权重
    pub bm25_weight: f64,
    /// 向量权重
    pub vector_weight: f64,
    /// 图谱权重
    pub graph_weight: f64,
}

impl Default for RrfConfig {
    fn default() -> Self {
        Self {
            k: 60,
            bm25_weight: 1.0,
            vector_weight: 1.0,
            graph_weight: 0.5,
        }
    }
}

/// 混合检索器: BM25 + Vector + Graph 三路融合
pub struct HybridRetriever {
    pub rrf_config: RrfConfig,
}

impl HybridRetriever {
    pub fn new(rrf_config: RrfConfig) -> Self {
        Self { rrf_config }
    }

    pub fn with_default() -> Self {
        Self::new(RrfConfig::default())
    }

    /// RRF 融合多路排序结果
    ///
    /// 输入: 各路独立排序结果 (越靠前分数越高)
    /// 输出: 融合排序 (RRF score 递减)
    pub fn rrf_fuse(&self, ranked_lists: Vec<Vec<String>>) -> Vec<(String, f64)> {
        let mut scores: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let weights = [
            self.rrf_config.bm25_weight,
            self.rrf_config.vector_weight,
            self.rrf_config.graph_weight,
        ];

        for (list_idx, list) in ranked_lists.iter().enumerate() {
            let w = weights[list_idx.min(weights.len() - 1)];
            for (rank, node_id) in list.iter().enumerate() {
                let rrf_score = w / (self.rrf_config.k as f64 + rank as f64 + 1.0);
                *scores.entry(node_id.clone()).or_insert(0.0) += rrf_score;
            }
        }

        let mut sorted: Vec<(String, f64)> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    /// 从 RRF 融合结果构建最终检索结果
    pub fn build_results(
        &self,
        fused: Vec<(String, f64)>,
        node_titles: &std::collections::HashMap<String, String>,
        snippets: &std::collections::HashMap<String, String>,
        limit: usize,
    ) -> Vec<RetrievalResult> {
        fused
            .into_iter()
            .take(limit)
            .map(|(id, score)| RetrievalResult {
                title: node_titles.get(&id).cloned().unwrap_or_default(),
                node_id: id.clone(),
                score,
                source: RetrievalSource::Fused,
                snippet: snippets.get(&id).cloned(),
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 检索管线编排 — 完整流程
// ═══════════════════════════════════════════════════════════════════

/// 检索管线配置
#[derive(Debug, Clone)]
pub struct RetrievalPipelineConfig {
    /// 最大返回结果数
    pub max_results: usize,
    /// 是否启用查询拆解
    pub enable_decomposition: bool,
    /// 是否启用 RRF 融合
    pub enable_rrf_fusion: bool,
}

impl Default for RetrievalPipelineConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            enable_decomposition: true,
            enable_rrf_fusion: true,
        }
    }
}

/// 检索管线: query → decompose → multi-channel retrieve → fuse → rank
pub struct RetrievalPipeline {
    pub retriever: HybridRetriever,
    pub config: RetrievalPipelineConfig,
}

impl RetrievalPipeline {
    pub fn new(retriever: HybridRetriever, config: RetrievalPipelineConfig) -> Self {
        Self { retriever, config }
    }

    /// 执行完整检索管线 (纯数据流, 不涉及 IO)
    ///
    /// 调用方提供各路已排序的 node_id 列表。
    pub fn execute(
        &self,
        query: &str,
        bm25_ranked: Vec<String>,
        vector_ranked: Vec<String>,
        graph_ranked: Vec<String>,
        node_titles: &std::collections::HashMap<String, String>,
        snippets: &std::collections::HashMap<String, String>,
    ) -> Vec<RetrievalResult> {
        let _decomposed = if self.config.enable_decomposition {
            DecomposedQuery::decompose(query)
        } else {
            DecomposedQuery {
                original: query.to_string(),
                sub_queries: vec![query.to_string()],
                strategy: DecompositionStrategy::Direct,
            }
        };

        // 当前实现: 只用原始查询的三路结果
        // 子查询扩展: 可以对每个 sub_query 分别检索再合并
        let ranked_lists = if self.config.enable_rrf_fusion {
            vec![bm25_ranked, vector_ranked, graph_ranked]
        } else {
            // 不融合: 直接用 BM25
            vec![bm25_ranked]
        };

        let fused = self.retriever.rrf_fuse(ranked_lists);
        self.retriever
            .build_results(fused, node_titles, snippets, self.config.max_results)
    }
}

// ═══════════════════════════════════════════════════════════════════
// 辅助
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_classification() {
        assert_eq!(RetrievalIntent::classify("Rust 的所有权"), RetrievalIntent::Fact);
        assert_eq!(RetrievalIntent::classify("总结一下架构"), RetrievalIntent::Overview);
        assert_eq!(RetrievalIntent::classify("BM25 vs 向量检索"), RetrievalIntent::Compare);
        assert_eq!(RetrievalIntent::classify("有哪些相关项目"), RetrievalIntent::Exploration);
    }

    #[test]
    fn test_rrf_fusion() {
        let retriever = HybridRetriever::with_default();
        let bm25 = vec!["a".into(), "b".into(), "c".into()];
        let vector = vec!["b".into(), "a".into(), "d".into()];
        let graph = vec!["a".into(), "d".into(), "e".into()];
        let fused = retriever.rrf_fuse(vec![bm25, vector, graph]);
        assert!(!fused.is_empty());
        // "a" 出现在三路前列, 应该排第一
        assert_eq!(fused[0].0, "a");
    }

    #[test]
    fn test_decompose_compare() {
        let dq = DecomposedQuery::decompose("BM25 和 向量检索 的区别");
        assert_eq!(dq.strategy, DecompositionStrategy::IntentSplit);
        assert!(dq.sub_queries.len() >= 2);
    }

    #[test]
    fn test_pipeline_end_to_end() {
        let pipeline = RetrievalPipeline::new(
            HybridRetriever::with_default(),
            RetrievalPipelineConfig::default(),
        );
        let mut titles = std::collections::HashMap::new();
        titles.insert("a".into(), "Title A".into());
        titles.insert("b".into(), "Title B".into());
        titles.insert("c".into(), "Title C".into());
        let results = pipeline.execute(
            "test query",
            vec!["a".into(), "b".into(), "c".into()],
            vec!["b".into(), "a".into()],
            vec!["a".into(), "c".into()],
            &titles,
            &std::collections::HashMap::new(),
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].source, RetrievalSource::Fused);
    }
}
