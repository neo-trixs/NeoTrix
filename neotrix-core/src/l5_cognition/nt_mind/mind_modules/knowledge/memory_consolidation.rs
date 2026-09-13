//! Memory Consolidation — 记忆整合压缩
//!
//! 吸收 KB 经验:
//! - 短期→长期记忆迁移
//! - 记忆压缩/摘要
//! - 遗忘曲线
//! - 记忆巩固

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 记忆整合管理器
pub struct MemoryConsolidation {
    short_term: _ShortTermMemory,
    long_term: _LongTermMemory,
    consolidation_queue: Vec<MemoryItem>,
    config: ConsolidationConfig,
    stats: _ConsolidationStats,
}

/// 整合配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    pub short_term_capacity: usize,
    pub consolidation_threshold: f64,
    pub forgetting_rate: f64,
    pub consolidation_interval: u64,
    pub compression_ratio: f64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            short_term_capacity: 100,
            consolidation_threshold: 0.7,
            forgetting_rate: 0.1,
            consolidation_interval: 3600,
            compression_ratio: 0.5,
        }
    }
}

/// 记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub importance: f64,
    pub recency: f64,
    pub frequency: u32,
    pub associations: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub embedding: Option<Vec<f32>>,
}

/// 记忆类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Episodic,    // 情景记忆
    Semantic,    // 语义记忆
    Procedural,  // 程序记忆
    Working,     // 工作记忆
}

/// 短期记忆
pub struct _ShortTermMemory {
    items: Vec<MemoryItem>,
    capacity: usize,
}

/// 长期记忆
pub struct _LongTermMemory {
    items: HashMap<String, MemoryItem>,
    categories: HashMap<String, Vec<String>>,
}

/// 整合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub items_consolidated: usize,
    pub items_forgotten: usize,
    pub items_compressed: usize,
    pub new_long_term_items: Vec<MemoryItem>,
    pub compressed_summaries: Vec<String>,
}

/// 整合统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ConsolidationStats {
    pub total_consolidations: u64,
    pub total_items_consolidated: u64,
    pub total_items_forgotten: u64,
    pub avg_consolidation_time: f64,
    pub memory_utilization: f64,
}

impl _ShortTermMemory {
    /// 创建新的短期记忆
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
        }
    }

    /// 添加记忆项
    pub fn add(&mut self, item: MemoryItem) -> Option<MemoryItem> {
        if self.items.len() >= self.capacity {
            // 移除最不重要的项
            let least_important_idx = self.items.iter()
                .enumerate()
                .min_by(|a, b| {
                    let score_a = a.1.importance * 0.5 + a.1.recency * 0.3 + (a.1.frequency as f64) * 0.2;
                    let score_b = b.1.importance * 0.5 + b.1.recency * 0.3 + (b.1.frequency as f64) * 0.2;
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx);

            if let Some(idx) = least_important_idx {
                let removed = self.items.remove(idx);
                self.items.push(item);
                Some(removed)
            } else {
                None
            }
        } else {
            self.items.push(item);
            None
        }
    }

    /// 获取所有记忆项
    pub fn get_all(&self) -> &[MemoryItem] {
        &self.items
    }

    /// 获取可整合的记忆项 (超过阈值)
    pub(crate) fn _get_consolidatable(&self, threshold: f64) -> Vec<MemoryItem> {
        self.items.iter()
            .filter(|item| {
                let score = item.importance * 0.5 + item.recency * 0.3 + (item.frequency as f64) * 0.2;
                score >= threshold
            })
            .cloned()
            .collect()
    }

    /// 移除已整合的项
    pub(crate) fn _remove_consolidated(&mut self, ids: &[String]) {
        self.items.retain(|item| !ids.contains(&item.id));
    }
}

impl _LongTermMemory {
    /// 创建新的长期记忆
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// 添加记忆项
    pub fn add(&mut self, item: MemoryItem) {
        self.items.insert(item.id.clone(), item.clone());

        // 按类型分类
        let category = format!("{:?}", item.memory_type);
        self.categories.entry(category).or_insert_with(Vec::new).push(item.id.clone());
    }

    /// 获取记忆项
    pub fn get(&self, id: &str) -> Option<&MemoryItem> {
        self.items.get(id)
    }

    /// 搜索记忆
    pub fn search(&self, query: &str) -> Vec<&MemoryItem> {
        self.items.values()
            .filter(|item| item.content.contains(query))
            .collect()
    }

    /// 应用遗忘曲线
    pub(crate) fn _apply_forgetting(&mut self, rate: f64) {
        let now = chrono::Utc::now();
        let ids_to_forget: Vec<String> = self.items.iter()
            .filter(|(_, item)| {
                let time_since_access = now.signed_duration_since(item.last_accessed).num_hours() as f64;
                let forgetting_factor = (-rate * time_since_access).exp();
                item.importance * forgetting_factor < 0.1
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in ids_to_forget {
            self.items.remove(&id);
        }
    }

    /// 获取大小
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl MemoryConsolidation {
    /// 创建新的记忆整合管理器
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            short_term: _ShortTermMemory::new(config.short_term_capacity),
            long_term: _LongTermMemory::new(),
            consolidation_queue: Vec::new(),
            config,
            stats: _ConsolidationStats {
                total_consolidations: 0,
                total_items_consolidated: 0,
                total_items_forgotten: 0,
                avg_consolidation_time: 0.0,
                memory_utilization: 0.0,
            },
        }
    }

    /// 添加到短期记忆
    pub(crate) fn _add_memory(&mut self, item: MemoryItem) {
        if let Some(evicted) = self.short_term.add(item) {
            self.consolidation_queue.push(evicted);
        }
    }

    /// 执行整合
    ///
    /// Note: Consolidation pipeline: (1) select short-term items exceeding importance threshold,
    /// (2) compress content, (3) promote to long-term memory with boosted importance,
    /// (4) remove consolidated items from short-term, (5) apply forgetting curve to long-term.
    ///
    /// Real implementation needs:
    /// - Semantic deduplication before promotion (avoid duplicate long-term entries)
    /// - Ebbinghaus forgetting curve with spaced repetition scheduling
    /// - Cross-memory association building (episodic ↔ semantic links)
    /// - Integration with KB for persistent storage beyond in-memory
    pub fn consolidate(&mut self) -> ConsolidationResult {
        let mut items_consolidated = 0;
        let items_forgotten = 0;
        let mut items_compressed = 0;
        let mut new_long_term_items = Vec::new();
        let mut compressed_summaries = Vec::new();

        // 从短期记忆中获取可整合的项
        let consolidatable = self.short_term._get_consolidatable(self.config.consolidation_threshold);

        for item in consolidatable {
            // 压缩内容
            let compressed_content = self.compress_content(&item.content);
            if compressed_content.len() < item.content.len() {
                items_compressed += 1;
                compressed_summaries.push(compressed_content.clone());
            }

            // 创建长期记忆项
            let mut long_term_item = item.clone();
            long_term_item.content = compressed_content;
            long_term_item.importance *= 1.2; // 巩固后重要性增加

            self.long_term.add(long_term_item.clone());
            new_long_term_items.push(long_term_item);
            items_consolidated += 1;
        }

        // 移除已整合的项
        let consolidated_ids: Vec<String> = new_long_term_items.iter().map(|i| i.id.clone()).collect();
        self.short_term._remove_consolidated(&consolidated_ids);

        // 应用遗忘曲线
        self.long_term._apply_forgetting(self.config.forgetting_rate);

        // 更新统计
        self.stats.total_consolidations += 1;
        self.stats.total_items_consolidated += items_consolidated as u64;
        self.stats.total_items_forgotten += items_forgotten as u64;
        self.stats.memory_utilization = self.short_term.items.len() as f64 / self.config.short_term_capacity as f64;

        ConsolidationResult {
            items_consolidated,
            items_forgotten,
            items_compressed,
            new_long_term_items,
            compressed_summaries,
        }
    }

    /// 压缩内容
    ///
    /// STUB: Simple truncation with "[compressed]" marker — no real summarization.
    /// Real implementation needs:
    /// - LLM-based abstractive summarization (e.g., use local Ollama model)
    /// - Extractive summarization with sentence importance scoring
    /// - Preserve key facts (entity names, dates, technical terms)
    /// - Adaptive compression ratio based on content complexity
    fn compress_content(&self, content: &str) -> String {
        // 简化版: 截断并添加摘要标记
        let max_length = (content.len() as f64 * self.config.compression_ratio) as usize;
        if content.len() <= max_length {
            content.to_string()
        } else {
            format!("{}... [compressed]", &content[..max_length])
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_ConsolidationStats {
        &self.stats
    }

    /// 获取短期记忆
    pub fn short_term(&self) -> &_ShortTermMemory {
        &self.short_term
    }

    /// 获取长期记忆
    pub fn long_term(&self) -> &_LongTermMemory {
        &self.long_term
    }
}
