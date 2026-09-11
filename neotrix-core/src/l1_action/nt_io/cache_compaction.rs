//! Cache-Aware Compaction — 基于缓存命中率的智能压缩
//!
//! 当缓存命中率高时，保留缓存相关内容；命中率低时，激进压缩。

/// 缓存条目
#[derive(Clone, Debug)]

pub struct CacheEntry {
    pub key: String,
    pub content: String,
    pub token_count: usize,
    pub hit_count: u32,
    pub last_hit: i64,
    pub created_at: i64,
}

/// 压缩策略

#[derive(Clone, Debug)]
pub enum CompactionStrategy {
    /// 保守：只压缩低命中内容
    Conservative,
    /// 平衡：按命中率加权压缩
    Balanced,
    /// 激进：最大压缩
    Aggressive,
}

/// 压缩结果

#[derive(Debug)]
pub struct CompactionResult {
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub entries_compacted: usize,
    pub entries_kept: usize,
    pub hit_rate_preserved: f64,
}

/// Cache-Aware 压缩器

#[derive(Debug)]
pub struct CacheCompactor {
    entries: Vec<CacheEntry>,
    strategy: CompactionStrategy,
    max_tokens: usize,
}


impl CacheCompactor {
    pub fn new(strategy: CompactionStrategy, max_tokens: usize) -> Self {
        Self {
            entries: Vec::new(),
            strategy,
            max_tokens,
        }
    }

    /// 添加缓存条目
    pub fn add_entry(&mut self, key: &str, content: &str, token_count: usize) {
        self.entries.push(CacheEntry {
            key: key.to_string(),
            content: content.to_string(),
            token_count,
            hit_count: 0,
            last_hit: now_ts(),
            created_at: now_ts(),
        });
    }

    /// 记录缓存命中
    pub fn record_hit(&mut self, key: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.hit_count += 1;
            entry.last_hit = now_ts();
        }
    }

    /// 执行压缩
    pub fn compact(&self) -> CompactionResult {
        let before_tokens: usize = self.entries.iter().map(|e| e.token_count).sum();

        let mut sorted = self.entries.clone();
        sorted.sort_by(|a, b| b.hit_count.cmp(&a.hit_count));

        let mut kept_tokens = 0;
        let mut kept = 0;
        let mut compacted = 0;

        for entry in &sorted {
            match self.strategy {
                CompactionStrategy::Conservative => {
                    if entry.hit_count > 2 || kept_tokens + entry.token_count <= self.max_tokens {
                        kept_tokens += entry.token_count;
                        kept += 1;
                    } else {
                        compacted += 1;
                    }
                }
                CompactionStrategy::Balanced => {
                    if entry.hit_count > 0 || kept_tokens + entry.token_count <= self.max_tokens / 2
                    {
                        kept_tokens += entry.token_count;
                        kept += 1;
                    } else {
                        compacted += 1;
                    }
                }
                CompactionStrategy::Aggressive => {
                    if kept_tokens + entry.token_count <= self.max_tokens / 4 {
                        kept_tokens += entry.token_count;
                        kept += 1;
                    } else {
                        compacted += 1;
                    }
                }
            }
        }

        CompactionResult {
            before_tokens,
            after_tokens: kept_tokens,
            entries_compacted: compacted,
            entries_kept: kept,
            hit_rate_preserved: if !self.entries.is_empty() {
                kept as f64 / self.entries.len() as f64
            } else {
                0.0
            },
        }
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let total_hits: u32 = self.entries.iter().map(|e| e.hit_count).sum();
        total_hits as f64 / self.entries.len() as f64
    }
}


fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
