use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use super::super::types::Message;

/// G: Response Caching — LRU 响应缓存, key 为 (model_id, messages) 的哈希。
/// 容量默认 256 条, 超出按最久未使用 (LRU) 驱逐; 命中/未命中计数器暴露给遥测。
#[derive(Debug)]
pub struct ResponseCache {
    entries: HashMap<u64, (String, u64)>,
    capacity: usize,
    tick: u64,
    hit_count: u64,
    miss_count: u64,
    /// G26 分层 expert 缓存 (colibri 吸收): 热集 pin — 被 pin 的 key 永不参与
    /// LRU 驱逐, 保持高频专家响应常驻。
    pinned: HashSet<u64>,
    /// prefetch 命中计数 (分层缓存 prefetch 遥测)。
    prefetch_hits: u64,
}

impl ResponseCache {
    /// 默认容量 (条目数)
    pub const DEFAULT_CAPACITY: usize = 256;
    /// 热集 pin 上限 (防 pin 过多挤掉冷条目的生存空间)
    pub const MAX_PINNED: usize = 32;

    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity: capacity.max(1),
            tick: 0,
            hit_count: 0,
            miss_count: 0,
            pinned: HashSet::new(),
            prefetch_hits: 0,
        }
    }

    /// 构造 (model_id, messages) → 规范化 key 字符串 (供内部哈希使用)
    pub fn key_for(model_id: &str, messages: &[Message]) -> String {
        let body = messages
            .iter()
            .map(|m| format!("{:?}:{}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}|{}", model_id, body)
    }

    /// 指纹硬化 key: model + 请求语义指纹 (max_tokens/tools/structured_output)。
    /// 修复: 旧 key_for 仅拼 messages, 同提示词不同 max_tokens/工具集会错误共享
    /// 缓存 → 可能命中截断输出 (Length) 或带 tool_calls 的响应。
    pub fn key_for_request(model_id: &str, fingerprint: &str) -> String {
        format!("{}|fp={}", model_id, fingerprint)
    }

    /// 确定性哈希 (std DefaultHasher, 无外部依赖)
    fn hash_key(key: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// 查询缓存 — 命中刷新 LRU 时间戳并返回克隆的响应文本
    pub fn cache(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        if let Some((resp, last_used)) = self.entries.get_mut(&hash) {
            self.tick += 1;
            *last_used = self.tick;
            self.hit_count += 1;
            return Some(resp.clone());
        }
        self.miss_count += 1;
        None
    }

    /// 写入缓存 — 已存在则刷新, 满容量驱逐最久未使用条目 (pin 条目不参与驱逐)
    pub fn insert(&mut self, key: &str, response: String) {
        let hash = Self::hash_key(key);
        self.tick += 1;
        if let Some(entry) = self.entries.get_mut(&hash) {
            *entry = (response, self.tick);
            return;
        }
        if self.entries.len() >= self.capacity {
            let lru_key = self
                .entries
                .iter()
                .filter(|(k, _)| !self.pinned.contains(k))
                .min_by_key(|(_, (_, t))| *t)
                .map(|(k, _)| *k);
            if let Some(k) = lru_key {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(hash, (response, self.tick));
    }

    /// G26 热集 pin (colibri 吸收): pin 一个 key, 使其免于 LRU 驱逐。
    /// 返回是否真正 pin (false = 已 pin 或已达上限)。
    pub fn pin(&mut self, key: &str) -> bool {
        if self.pinned.len() >= Self::MAX_PINNED {
            return false;
        }
        self.pinned.insert(Self::hash_key(key))
    }

    /// 解除 pin。
    pub fn unpin(&mut self, key: &str) {
        self.pinned.remove(&Self::hash_key(key));
    }

    pub fn pinned_count(&self) -> usize {
        self.pinned.len()
    }

    /// G26 prefetch (colibri 吸收): 预取已缓存的专家响应 — 命中(存在)则刷新
    /// LRU 时间戳并计入 prefetch_hits, 未命中仅刷新计数 (无响应返回)。
    /// 用于在下一轮高概率请求前预热热集, 避免 miss 时重新调用 provider。
    pub fn prefetch(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        let exists = self.entries.contains_key(&hash);
        if !exists {
            return None;
        }
        self.tick += 1;
        if let Some((_, t)) = self.entries.get_mut(&hash) {
            *t = self.tick;
        }
        self.prefetch_hits += 1;
        self.entries.get(&hash).map(|(resp, _)| resp.clone())
    }

    pub fn prefetch_hit_count(&self) -> u64 {
        self.prefetch_hits
    }

    /// P0-7 lookahead 预取 (OasisKV 吸收): 由 speculative 提示词流预测未来访问 key,
    /// 提前把条目刷新到热层 (staging), 而非仅刷新已缓存项 (原 prefetch 是反应式)。
    /// hints 是"未来可能访问的 key 列表" — 对已缓存者刷新 LRU 防驱逐, 对缺失者
    /// 由调用方判定是否值得热加载 (返回缺失列表供 fetch)。
    ///
    /// 返回 (预取命中数, 缺失的 lookahead key 列表)。
    pub fn prefetch_lookahead(&mut self, hints: &[String]) -> (usize, Vec<String>) {
        let mut hits = 0;
        let mut missing = Vec::new();
        for hint in hints {
            let hash = Self::hash_key(hint);
            if self.entries.contains_key(&hash) {
                self.tick += 1;
                if let Some((_, t)) = self.entries.get_mut(&hash) {
                    *t = self.tick;
                }
                self.prefetch_hits += 1;
                hits += 1;
            } else {
                missing.push(hint.clone());
            }
        }
        (hits, missing)
    }

    /// P0-7 辅助: 低开销 lookahead 提示 — 从当前 key 派生相邻 key 候选
    /// (如同一 model_id 下的相邻温度/指纹变体)。纯字符串启发, 供调用方
    /// 作为 prefetch_lookahead 的 hints 输入。
    pub fn lookahead_hints(&self, key: &str) -> Vec<String> {
        let mut hints = Vec::new();
        if let Some((model, _)) = key.split_once('|') {
            hints.push(format!("{}|fp=lookahead:1", model));
            hints.push(format!("{}|fp=lookahead:2", model));
        }
        hints
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn hit_count(&self) -> u64 {
        self.hit_count
    }

    pub fn miss_count(&self) -> u64 {
        self.miss_count
    }
}