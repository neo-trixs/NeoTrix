//! 持久记忆茧 — 跨会话持久化
//!
//! Cocoon 是记忆的持久化单元，按领域组织，支持衰减、巩固和合并。
//! 提供从 CrystalConsciousness 双向同步的能力。

use super::consciousness::{CrystalConsciousness, Memory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 持久记忆茧 — 跨会话持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCocoon {
    pub id: String,
    pub memories: Vec<Memory>,
    pub retention_policy: RetentionPolicy,
    pub meta_metrics: CocoonMetrics,
    pub last_accessed: u64,
    pub created_at: u64,
}

/// 策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_memories: usize,
    pub min_strength: f64,
    pub domain_weights: HashMap<String, f64>,
    pub decay_strategy: DecayStrategy,
}

/// 衰减策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayStrategy {
    Exponential { rate: f64 },
    Linear { step: f64 },
    StepFunction { thresholds: Vec<(f64, f64)> },
}

/// 茧性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoonMetrics {
    pub total_recall_attempts: u64,
    pub successful_recalls: u64,
    pub avg_relevance_score: f64,
    pub memory_utilization: f64,
}

/// 茧存储管理器
pub struct CocoonStore {
    pub cocoons: HashMap<String, PersistentCocoon>,
    pub store_path: PathBuf,
    pub strategy: RetentionPolicy,
}

impl CocoonStore {
    /// 创建新的茧存储管理器
    pub fn new(store_path: PathBuf) -> Self {
        let default_strategy = RetentionPolicy {
            max_memories: 1000,
            min_strength: 0.1,
            domain_weights: HashMap::new(),
            decay_strategy: DecayStrategy::Exponential { rate: 0.01 },
        };

        Self {
            cocoons: HashMap::new(),
            store_path,
            strategy: default_strategy,
        }
    }

    /// 从磁盘加载
    pub fn load() -> Self {
        let store_path = Self::default_store_path();
        if !store_path.exists() {
            return Self::new(store_path);
        }

        let content = match std::fs::read_to_string(&store_path) {
            Ok(c) => c,
            Err(_) => return Self::new(store_path),
        };

        let data: StoreData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(_) => StoreData {
                cocoons: HashMap::new(),
                strategy: RetentionPolicy::default(),
            },
        };

        Self {
            cocoons: data.cocoons,
            store_path,
            strategy: data.strategy,
        }
    }

    /// 保存到磁盘
    pub fn save(&self) -> Result<(), String> {
        let data = StoreData {
            cocoons: self.cocoons.clone(),
            strategy: self.strategy.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(&self.store_path, json).map_err(|e| format!("Failed to write: {}", e))
    }

    /// 创建新茧
    pub fn create_cocoon(&mut self, domain: &str) -> String {
        let id = format!("cocoon-{}-{}", domain, timestamp_now());
        let now = timestamp_now();

        let cocoon = PersistentCocoon {
            id: id.clone(),
            memories: Vec::new(),
            retention_policy: self.strategy.clone(),
            meta_metrics: CocoonMetrics::default(),
            last_accessed: now,
            created_at: now,
        };

        self.cocoons.insert(id.clone(), cocoon);
        id
    }

    /// 存储记忆到指定茧
    pub fn store_memory(&mut self, cocoon_id: &str, memory: Memory) -> Result<(), String> {
        let cocoon = self
            .cocoons
            .get_mut(cocoon_id)
            .ok_or_else(|| format!("Cocoon not found: {}", cocoon_id))?;

        // 检查容量限制
        if cocoon.memories.len() >= cocoon.retention_policy.max_memories {
            return Err("Cocoon memory capacity reached".into());
        }

        // 检查强度阈值
        if memory.strength < cocoon.retention_policy.min_strength {
            return Err("Memory strength below threshold".into());
        }

        cocoon.memories.push(memory);
        cocoon.last_accessed = timestamp_now();
        Ok(())
    }

    /// 按领域和查询召回记忆
    pub fn recall(&self, domain: &str, query: &str, limit: usize) -> Vec<Memory> {
        let mut recalled: Vec<Memory> = self
            .cocoons
            .values()
            .filter(|_c| {
                let domain_weight = self
                    .strategy
                    .domain_weights
                    .get(domain)
                    .copied()
                    .unwrap_or(1.0);
                domain_weight > 0.0
            })
            .flat_map(|c| &c.memories)
            .filter(|m| m.domain == domain)
            .filter(|m| {
                // 简单的查询匹配
                query.is_empty() || m.content.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .collect();

        // 按强度和访问次数排序
        recalled.sort_by(|a, b| {
            let score_a = a.strength * (1.0 + a.access_count as f64 * 0.1);
            let score_b = b.strength * (1.0 + b.access_count as f64 * 0.1);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recalled.truncate(limit);
        recalled
    }

    /// 合并小茧
    pub fn consolidate(&mut self) {
        let min_memories = 10;
        let mut to_merge: Vec<String> = Vec::new();
        let mut domain_groups: HashMap<String, Vec<String>> = HashMap::new();

        // 按领域分组
        for (id, _cocoon) in &self.cocoons {
            // 从茧ID中提取领域（假设格式为 cocoon-{domain}-{timestamp}）
            let domain = id.split('-').nth(1).unwrap_or("unknown").to_string();
            domain_groups.entry(domain).or_default().push(id.clone());
        }

        // 找出需要合并的茧
        for (_domain, ids) in &domain_groups {
            let small_cocoons: Vec<String> = ids
                .iter()
                .filter(|id| {
                    self.cocoons
                        .get(*id)
                        .map_or(false, |c| c.memories.len() < min_memories)
                })
                .cloned()
                .collect();

            if small_cocoons.len() > 1 {
                to_merge.extend(small_cocoons);
            }
        }

        if to_merge.is_empty() {
            return;
        }

        // 收集所有需要合并的记忆
        let mut merged_memories: Vec<Memory> = Vec::new();
        for id in &to_merge {
            if let Some(cocoon) = self.cocoons.remove(id) {
                merged_memories.extend(cocoon.memories);
            }
        }

        // 创建新茧
        let now = timestamp_now();
        let new_id = format!("cocoon-merged-{}", now);
        let new_cocoon = PersistentCocoon {
            id: new_id.clone(),
            memories: merged_memories,
            retention_policy: self.strategy.clone(),
            meta_metrics: CocoonMetrics::default(),
            last_accessed: now,
            created_at: now,
        };

        self.cocoons.insert(new_id, new_cocoon);
    }

    /// 淘汰弱记忆
    pub fn prune(&mut self) {
        let now = timestamp_now();
        for cocoon in self.cocoons.values_mut() {
            cocoon.memories.retain(|m| {
                // 保留满足以下条件的记忆：
                // 1. 强度大于阈值
                // 2. 最近被访问过（30天内）
                let age_days = (now - m.last_accessed) as f64 / 86400.0;
                m.strength >= cocoon.retention_policy.min_strength && age_days < 30.0
            });
        }

        // 移除空茧
        self.cocoons.retain(|_, c| !c.memories.is_empty());
    }

    /// 从晶体意识同步数据到茧
    pub fn sync_from_consciousness(&mut self, consciousness: &CrystalConsciousness) {
        // 按领域组织记忆
        let mut domain_memories: HashMap<String, Vec<Memory>> = HashMap::new();

        for memory in consciousness.memories.values() {
            domain_memories
                .entry(memory.domain.clone())
                .or_default()
                .push(memory.clone());
        }

        // 为每个领域创建或更新茧
        for (domain, memories) in domain_memories {
            // 查找现有茧或创建新的
            let cocoon_id = self
                .cocoons
                .values()
                .find(|c| c.id.starts_with(&format!("cocoon-{}-", domain)))
                .map(|c| c.id.clone())
                .unwrap_or_else(|| self.create_cocoon(&domain));

            if let Some(cocoon) = self.cocoons.get_mut(&cocoon_id) {
                // 合并记忆（避免重复）
                for memory in memories {
                    if !cocoon.memories.iter().any(|m| m.id == memory.id) {
                        cocoon.memories.push(memory);
                    }
                }
            }
        }
    }

    /// 同步茧数据到晶体意识
    pub fn sync_to_consciousness(&self, consciousness: &mut CrystalConsciousness) {
        for cocoon in self.cocoons.values() {
            for memory in &cocoon.memories {
                // 只同步意识中不存在的记忆
                if !consciousness.memories.contains_key(&memory.id) {
                    consciousness
                        .memories
                        .insert(memory.id.clone(), memory.clone());
                }
            }
        }
    }

    /// 获取所有茧的统计信息
    pub fn stats(&self) -> StoreStats {
        let total_memories: usize = self.cocoons.values().map(|c| c.memories.len()).sum();

        let domain_counts: HashMap<String, usize> = self
            .cocoons
            .values()
            .flat_map(|c| &c.memories)
            .fold(HashMap::new(), |mut acc, m| {
                *acc.entry(m.domain.clone()).or_insert(0) += 1;
                acc
            });

        StoreStats {
            cocoon_count: self.cocoons.len(),
            total_memories,
            domain_counts,
        }
    }

    /// 默认存储路径
    fn default_store_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("crystal_core")
            .join("cocoons.json")
    }
}

/// 存储数据（用于序列化）
#[derive(Debug, Serialize, Deserialize)]
struct StoreData {
    cocoons: HashMap<String, PersistentCocoon>,
    strategy: RetentionPolicy,
}

/// 存储统计信息
#[derive(Debug)]
pub struct StoreStats {
    pub cocoon_count: usize,
    pub total_memories: usize,
    pub domain_counts: HashMap<String, usize>,
}

/// 默认实现
impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_memories: 1000,
            min_strength: 0.1,
            domain_weights: HashMap::new(),
            decay_strategy: DecayStrategy::Exponential { rate: 0.01 },
        }
    }
}

impl Default for CocoonMetrics {
    fn default() -> Self {
        Self {
            total_recall_attempts: 0,
            successful_recalls: 0,
            avg_relevance_score: 0.0,
            memory_utilization: 0.0,
        }
    }
}

/// 时间戳生成
fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl std::fmt::Display for CocoonStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats();
        writeln!(f, "=== Cocoon Store ===")?;
        writeln!(f, "Store path: {}", self.store_path.display())?;
        writeln!(f, "Cocoons: {}", stats.cocoon_count)?;
        writeln!(f, "Total memories: {}", stats.total_memories)?;
        writeln!(f, "Domains: {:?}", stats.domain_counts)?;
        Ok(())
    }
}
