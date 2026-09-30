//! AutoMem 记忆架构优化 — 经验引导的架构搜索
//!
//! 通过搜索记忆组件的组合空间 (编码器×存储×检索×管理)，
//! 找到最适合当前任务分布的记忆架构。

use serde::{Deserialize, Serialize};


/// 编码器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncoderType {
    Extractive,
    Abstractive,
    Hybrid,
    Embedding,
    Keyword,
}

impl EncoderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Extractive => "extractive",
            Self::Abstractive => "abstractive",
            Self::Hybrid => "hybrid",
            Self::Embedding => "embedding",
            Self::Keyword => "keyword",
        }
    }
}

/// 存储类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoreType {
    VectorDB,
    KeyValue,
    GraphDB,
    DocumentDB,
    Hybrid,
}

impl StoreType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VectorDB => "vector_db",
            Self::KeyValue => "key_value",
            Self::GraphDB => "graph_db",
            Self::DocumentDB => "document_db",
            Self::Hybrid => "hybrid",
        }
    }
}

/// 检索器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrieverType {
    CosineSimilarity,
    BM25,
    Hybrid,
    Reranker,
    GraphTraversal,
}

impl RetrieverType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CosineSimilarity => "cosine_similarity",
            Self::BM25 => "bm25",
            Self::Hybrid => "hybrid",
            Self::Reranker => "reranker",
            Self::GraphTraversal => "graph_traversal",
        }
    }
}

/// 管理器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManagerType {
    LRU,
    LFU,
    TimeDecay,
    ImportanceBased,
    Adaptive,
}

impl ManagerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LRU => "lru",
            Self::LFU => "lfu",
            Self::TimeDecay => "time_decay",
            Self::ImportanceBased => "importance_based",
            Self::Adaptive => "adaptive",
        }
    }
}

/// 记忆架构 — 四组件组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryArchitecture {
    pub encoder: EncoderType,
    pub store: StoreType,
    pub retriever: RetrieverType,
    pub manager: ManagerType,
    pub label: String,
}

impl MemoryArchitecture {
    /// 生成架构的唯一签名
    pub fn signature(&self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.encoder.as_str(),
            self.store.as_str(),
            self.retriever.as_str(),
            self.manager.as_str()
        )
    }

    /// 所有可能的架构
    pub fn all_variants() -> Vec<MemoryArchitecture> {
        let encoders = [
            EncoderType::Extractive,
            EncoderType::Abstractive,
            EncoderType::Hybrid,
            EncoderType::Embedding,
            EncoderType::Keyword,
        ];
        let stores = [
            StoreType::VectorDB,
            StoreType::KeyValue,
            StoreType::GraphDB,
            StoreType::DocumentDB,
            StoreType::Hybrid,
        ];
        let retrievers = [
            RetrieverType::CosineSimilarity,
            RetrieverType::BM25,
            RetrieverType::Hybrid,
            RetrieverType::Reranker,
            RetrieverType::GraphTraversal,
        ];
        let managers = [
            ManagerType::LRU,
            ManagerType::LFU,
            ManagerType::TimeDecay,
            ManagerType::ImportanceBased,
            ManagerType::Adaptive,
        ];

        let mut archs = Vec::new();
        for enc in &encoders {
            for store in &stores {
                for ret in &retrievers {
                    for mgr in &managers {
                        let label = format!(
                            "{}/{}/{}/{}",
                            enc.as_str(),
                            store.as_str(),
                            ret.as_str(),
                            mgr.as_str()
                        );
                        archs.push(MemoryArchitecture {
                            encoder: enc.clone(),
                            store: store.clone(),
                            retriever: ret.clone(),
                            manager: mgr.clone(),
                            label,
                        });
                    }
                }
            }
        }
        archs
    }

    /// 计算架构的复杂度分数 (0-1)
    pub fn complexity(&self) -> f64 {
        let encoder_score = match self.encoder {
            EncoderType::Extractive | EncoderType::Keyword => 0.2,
            EncoderType::Abstractive | EncoderType::Embedding => 0.6,
            EncoderType::Hybrid => 0.8,
        };
        let store_score = match self.store {
            StoreType::KeyValue => 0.2,
            StoreType::VectorDB | StoreType::DocumentDB => 0.5,
            StoreType::GraphDB => 0.7,
            StoreType::Hybrid => 0.9,
        };
        let retriever_score = match self.retriever {
            RetrieverType::BM25 | RetrieverType::CosineSimilarity => 0.3,
            RetrieverType::Hybrid => 0.6,
            RetrieverType::Reranker => 0.7,
            RetrieverType::GraphTraversal => 0.8,
        };
        let manager_score = match self.manager {
            ManagerType::LRU | ManagerType::LFU => 0.2,
            ManagerType::TimeDecay => 0.4,
            ManagerType::ImportanceBased => 0.6,
            ManagerType::Adaptive => 0.8,
        };
        (encoder_score + store_score + retriever_score + manager_score) / 4.0
    }
}

/// AutoMem 记忆架构搜索器
pub struct AutoMemMemoryArchitect {
    pub search_space: Vec<MemoryArchitecture>,
    pub evaluated: Vec<(MemoryArchitecture, f64)>,
    pub best_architecture: Option<MemoryArchitecture>,
    pub best_score: f64,
    pub iteration: usize,
}

impl AutoMemMemoryArchitect {
    pub fn new() -> Self {
        Self {
            search_space: MemoryArchitecture::all_variants(),
            evaluated: Vec::new(),
            best_architecture: None,
            best_score: 0.0,
            iteration: 0,
        }
    }

    /// 经验引导的架构搜索
    ///
    /// 基于历史评估结果，选择最可能有效的架构:
    /// 1. 如果有高分历史，偏向复杂度相近的架构
    /// 2. 如果历史较少，随机探索
    /// 3. 惩罚已评估过的架构
    pub fn propose_architecture(
        &self,
        history: &[(MemoryArchitecture, f64)],
    ) -> MemoryArchitecture {
        if self.search_space.is_empty() {
            // 搜索空间耗尽，从已评估的中选最优
            return history
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(a, _)| a.clone())
                .unwrap_or_else(|| self.fallback_architecture());
        }

        let evaluated_sigs: std::collections::HashSet<String> = history
            .iter()
            .map(|(a, _)| a.signature())
            .collect();

        // 过滤已评估的
        let candidates: Vec<&MemoryArchitecture> = self
            .search_space
            .iter()
            .filter(|a| !evaluated_sigs.contains(&a.signature()))
            .collect();

        if candidates.is_empty() {
            return self.fallback_architecture();
        }

        if history.len() < 3 {
            // 探索阶段: 偏向高复杂度 (更有潜力)
            candidates
                .iter()
                .max_by(|a, b| {
                    a.complexity()
                        .partial_cmp(&b.complexity())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|a| (*a).clone())
                .unwrap_or_else(|| candidates[0].clone())
        } else {
            // 利用阶段: 偏向与高分架构复杂度相近的
            let avg_complexity: f64 = history
                .iter()
                .map(|(a, s)| a.complexity() * s)
                .sum::<f64>()
                / history.iter().map(|(_, s)| s).sum::<f64>().max(0.001);

            candidates
                .iter()
                .min_by(|a, b| {
                    let diff_a = (a.complexity() - avg_complexity).abs();
                    let diff_b = (b.complexity() - avg_complexity).abs();
                    diff_a.partial_cmp(&diff_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|a| (*a).clone())
                .unwrap_or_else(|| candidates[0].clone())
        }
    }

    /// 失败引导的模块诊断
    pub fn diagnose_failure(&self, arch: &MemoryArchitecture, failure: &str) -> String {
        let failure_lower = failure.to_lowercase();

        // 编码失败诊断
        if failure_lower.contains("encoding") || failure_lower.contains("tokenize") {
            return match arch.encoder {
                EncoderType::Embedding => {
                    "Embedding encoder may be OOM or dimension mismatch. \
                     Try Extractive encoder or reduce embedding dimension."
                        .into()
                }
                EncoderType::Abstractive => {
                    "Abstractive encoder requires significant compute. \
                     Consider Extractive for speed, or Hybrid for balance."
                        .into()
                }
                _ => {
                    "Encoder failure may be due to input format. \
                     Check input preprocessing pipeline."
                        .into()
                }
            };
        }

        // 存储失败诊断
        if failure_lower.contains("storage") || failure_lower.contains("write") || failure_lower.contains("persist") {
            return match arch.store {
                StoreType::VectorDB => {
                    "VectorDB may have index corruption. \
                     Try rebuilding index or switch to KeyValue for small datasets."
                        .into()
                }
                StoreType::GraphDB => {
                    "GraphDB cycles detected. \
                     Check for circular references or use DocumentDB instead."
                        .into()
                }
                _ => {
                    "Storage failure may be capacity-related. \
                     Check available disk space and memory."
                        .into()
                }
            };
        }

        // 检索失败诊断
        if failure_lower.contains("retrieval") || failure_lower.contains("recall") || failure_lower.contains("search") {
            return match arch.retriever {
                RetrieverType::CosineSimilarity => {
                    "Cosine similarity has low recall for keyword-heavy queries. \
                     Try BM25 or Hybrid retriever."
                        .into()
                }
                RetrieverType::BM25 => {
                    "BM25 has low recall for semantic queries. \
                     Try CosineSimilarity or Hybrid retriever."
                        .into()
                }
                RetrieverType::GraphTraversal => {
                    "Graph traversal may miss disconnected components. \
                     Add fallback CosineSimilarity retriever."
                        .into()
                }
                _ => {
                    "Retrieval failure may be indexing-related. \
                     Check index freshness and query preprocessing."
                        .into()
                }
            };
        }

        // 管理失败诊断
        if failure_lower.contains("management") || failure_lower.contains("eviction") || failure_lower.contains("cache") {
            return match arch.manager {
                ManagerType::LRU => {
                    "LRU may evict important one-time references. \
                     Try ImportanceBased or Adaptive manager."
                        .into()
                }
                ManagerType::LFU => {
                    "LFU may retain stale frequent items. \
                     Try TimeDecay or Adaptive manager."
                        .into()
                }
                ManagerType::TimeDecay => {
                    "TimeDecay may discard recent important items too quickly. \
                     Adjust decay rate or use ImportanceBased manager."
                        .into()
                }
                _ => {
                    "Management failure may be policy-related. \
                     Review eviction thresholds and memory budget."
                        .into()
                }
            };
        }

        // 默认诊断
        format!(
            "Unknown failure for architecture '{}'. \
             Consider switching to Hybrid variants for robustness.",
            arch.label
        )
    }

    /// 评估架构并记录
    pub fn evaluate(&mut self, arch: MemoryArchitecture, score: f64) {
        self.evaluated.push((arch.clone(), score));
        self.iteration += 1;

        if score > self.best_score {
            self.best_score = score;
            self.best_architecture = Some(arch);
        }
    }

    /// 获取当前最优架构
    pub fn best(&self) -> Option<&MemoryArchitecture> {
        self.best_architecture.as_ref()
    }

    /// 获取搜索空间统计
    pub fn stats(&self) -> ArchitectStats {
        let total = self.search_space.len();
        let evaluated_count = self.evaluated.len();
        let remaining = total.saturating_sub(evaluated_count);

        ArchitectStats {
            total_space: total,
            evaluated: evaluated_count,
            remaining,
            best_score: self.best_score,
            avg_score: if evaluated_count > 0 {
                self.evaluated.iter().map(|(_, s)| s).sum::<f64>() / evaluated_count as f64
            } else {
                0.0
            },
        }
    }

    /// 回退架构: 默认的稳健组合
    fn fallback_architecture(&self) -> MemoryArchitecture {
        MemoryArchitecture {
            encoder: EncoderType::Hybrid,
            store: StoreType::Hybrid,
            retriever: RetrieverType::Hybrid,
            manager: ManagerType::Adaptive,
            label: "hybrid/hybrid/hybrid/adaptive".into(),
        }
    }
}

impl Default for AutoMemMemoryArchitect {
    fn default() -> Self {
        Self::new()
    }
}

/// 架构搜索统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectStats {
    pub total_space: usize,
    pub evaluated: usize,
    pub remaining: usize,
    pub best_score: f64,
    pub avg_score: f64,
}
