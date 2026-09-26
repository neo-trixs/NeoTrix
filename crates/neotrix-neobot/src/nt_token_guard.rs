//! `nt_token_guard` — Token 成本门（EVO-01，抄 claude-token-optimizer 思想）。
//!
//! 纯同步纯逻辑，不碰网络/文件：
//! 1. [`estimate_tokens`]：文本 token 估算（≈ chars/4，向上取整）；
//! 2. [`TokenMeter`]：输入/输出/缓存三计数；
//! 3. [`GuardPolicy`] + [`GuardVerdict`] + [`audit`]：warn/block 两阈值门；
//! 4. [`LearningInjector`]：recent-learnings 注入器（关键词→上下文片段，
//!    内存 HashMap，上限 50 条，超限逐出最旧）。
//!
//! 与 `nt_cost` 的分工：`nt_cost` 管“事后多少钱”（美元计价），
//! 本模块管“事前让不让过”（token 量门 + 上下文注入节流）。

use std::collections::{HashMap, VecDeque};

/// recent-learnings 上限条数。
pub const MAX_LEARNINGS: usize = 50;

/// 默认 warn 阈值（total tokens）。
pub const DEFAULT_WARN_AT: u64 = 2000;
/// 默认 block 阈值（total tokens）。
pub const DEFAULT_BLOCK_AT: u64 = 8000;

/// 估算文本 token 数：`chars/4` 向上取整（空串 = 0）。
///
/// 纯函数；按 Unicode scalar 计（`chars().count()`），CJK/emoji 与英文同权。
pub fn estimate_tokens(text: &str) -> u64 {
    let chars = text.chars().count() as u64;
    chars.saturating_add(3) / 4
}

/// Token 三计数表。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenMeter {
    /// 输入 token。
    pub input_tokens: u64,
    /// 输出 token。
    pub output_tokens: u64,
    /// 缓存命中 token（不重复计费/计量的部分，仍计入 total 做门控）。
    pub cached_tokens: u64,
}

impl TokenMeter {
    /// 构造（各计数直接给；调用方保证语义）。
    pub fn new(input_tokens: u64, output_tokens: u64, cached_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cached_tokens,
        }
    }

    /// 空表。
    pub fn empty() -> Self {
        Self::default()
    }

    /// 累加输入（饱和加，永不溢出）。
    pub fn add_input(&mut self, n: u64) {
        self.input_tokens = self.input_tokens.saturating_add(n);
    }

    /// 累加输出（饱和加）。
    pub fn add_output(&mut self, n: u64) {
        self.output_tokens = self.output_tokens.saturating_add(n);
    }

    /// 累加缓存命中（饱和加）。
    pub fn add_cached(&mut self, n: u64) {
        self.cached_tokens = self.cached_tokens.saturating_add(n);
    }

    /// 用两段文本估算并记入输入/输出。
    pub fn record_estimate(&mut self, input_text: &str, output_text: &str) {
        self.add_input(estimate_tokens(input_text));
        self.add_output(estimate_tokens(output_text));
    }

    /// 门控总量 = 输入 + 输出 + 缓存（饱和加）。
    pub fn total(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cached_tokens)
    }

    /// 去缓存后的净新增量 = 输入 + 输出（饱和加）。
    pub fn uncached_total(&self) -> u64 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}

/// 成本门策略：warn/block 两阈值（total tokens 比较）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardPolicy {
    /// ≥ 此值判 Warn。
    pub warn_at: u64,
    /// ≥ 此值判 Block（应 ≥ warn_at，构造时自动钳正）。
    pub block_at: u64,
}

impl GuardPolicy {
    /// 显式阈值构造；若 `warn > block` 则把 warn 钳到 block（永不 panic）。
    pub fn with_thresholds(warn_at: u64, block_at: u64) -> Self {
        let block = block_at;
        let warn = if warn_at > block { block } else { warn_at };
        Self {
            warn_at: warn,
            block_at: block,
        }
    }
}

impl Default for GuardPolicy {
    /// 默认 warn 2000 / block 8000。
    fn default() -> Self {
        Self {
            warn_at: DEFAULT_WARN_AT,
            block_at: DEFAULT_BLOCK_AT,
        }
    }
}

/// 门控裁决。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardVerdict {
    /// 放行。
    Pass { total: u64 },
    /// 放行但告警（含原因）。
    Warn { total: u64, reason: String },
    /// 拦截（含原因）。
    Block { total: u64, reason: String },
}

impl GuardVerdict {
    /// 门控总量。
    pub fn total(&self) -> u64 {
        match self {
            Self::Pass { total }
            | Self::Warn { total, .. }
            | Self::Block { total, .. } => *total,
        }
    }

    /// 是否放行（含 Warn）。
    pub fn is_pass(&self) -> bool {
        !self.is_block()
    }

    /// 是否拦截。
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block { .. })
    }

    /// 原因（Pass 无原因 → None）。
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Pass { .. } => None,
            Self::Warn { reason, .. } | Self::Block { reason, .. } => Some(reason.as_str()),
        }
    }
}

/// 一次估算：两段文本 + 已知缓存命中 → 计数表（纯函数）。
pub fn measure(input_text: &str, output_text: &str, cached_tokens: u64) -> TokenMeter {
    TokenMeter::new(
        estimate_tokens(input_text),
        estimate_tokens(output_text),
        cached_tokens,
    )
}

/// 门控审计：按 total 与两阈值比较（纯函数，`≥warn → Warn`，`≥block → Block`）。
pub fn audit(meter: &TokenMeter, policy: &GuardPolicy) -> GuardVerdict {
    let total = meter.total();
    if total >= policy.block_at {
        GuardVerdict::Block {
            total,
            reason: format!(
                "total {total} >= block {} (warn {})",
                policy.block_at, policy.warn_at
            ),
        }
    } else if total >= policy.warn_at {
        GuardVerdict::Warn {
            total,
            reason: format!(
                "total {total} >= warn {} (< block {})",
                policy.warn_at, policy.block_at
            ),
        }
    } else {
        GuardVerdict::Pass { total }
    }
}

/// recent-learnings 注入器：关键词→上下文片段（内存 HashMap，上限 50 条）。
///
/// - key 按原样存，命中按小写子串（含即中，query/output 两侧都转小写比较）；
/// - 超限逐出最旧（插入顺序 FIFO；已存在 key 的更新视为最新）。
#[derive(Debug, Clone, Default)]
pub struct LearningInjector {
    map: HashMap<String, String>,
    order: VecDeque<String>,
    capacity: usize,
}

impl LearningInjector {
    /// 新建（容量 [`MAX_LEARNINGS`]）。
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity: MAX_LEARNINGS,
        }
    }

    /// 当前条数。
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 容量上限。
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 插入/更新一条；超限逐出最旧；空 key 空片段不收（静默忽略）。
    pub fn insert(&mut self, keyword: &str, snippet: &str) {
        if keyword.is_empty() || snippet.is_empty() {
            return;
        }
        if self.capacity == 0 {
            return;
        }
        let key = keyword.to_string();
        if self.map.contains_key(&key) {
            self.map.insert(key.clone(), snippet.to_string());
            self.promote(&key);
            return;
        }
        while self.map.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, snippet.to_string());
    }

    /// 精确 key 查询（大小写敏感）。
    pub fn get(&self, keyword: &str) -> Option<&String> {
        self.map.get(keyword)
    }

    /// 按 query 子串命中返回相关片段（大小写不敏感；无命中 → 空 vec）。
    pub fn inject(&self, query: &str) -> Vec<String> {
        let q = query.to_ascii_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for key in self.order.iter() {
            let hit = q.contains(&key.to_ascii_lowercase());
            if hit {
                if let Some(snippet) = self.map.get(key) {
                    out.push(snippet.clone());
                }
            }
        }
        out
    }

    fn promote(&mut self, key: &str) {
        let mut rebuilt = VecDeque::new();
        for k in self.order.iter() {
            if k.as_str() != key {
                rebuilt.push_back(k.clone());
            }
        }
        rebuilt.push_back(key.to_string());
        self.order = rebuilt;
    }
}

#[cfg(test)]
mod tests {
    use super::{GuardPolicy, LearningInjector, MAX_LEARNINGS, audit, estimate_tokens, measure};

    #[test]
    fn estimate_rounds_up_per_four_chars() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("a"), 1);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn audit_threshold_boundaries() {
        let policy = GuardPolicy::with_thresholds(100, 200);
        // 99 pass / 100 warn / 199 warn / 200 block
        let mut m = super::TokenMeter::new(99, 0, 0);
        assert_eq!(audit(&m, &policy).total(), 99);
        assert!(matches!(audit(&m, &policy), super::GuardVerdict::Pass { .. }));
        m = super::TokenMeter::new(100, 0, 0);
        assert!(matches!(audit(&m, &policy), super::GuardVerdict::Warn { .. }));
        m = super::TokenMeter::new(150, 49, 0);
        assert!(matches!(audit(&m, &policy), super::GuardVerdict::Warn { .. }));
        m = super::TokenMeter::new(150, 50, 0);
        assert!(matches!(audit(&m, &policy), super::GuardVerdict::Block { .. }));
        // 缓存也计入 total
        m = super::TokenMeter::new(10, 10, 200);
        assert!(audit(&m, &policy).is_block());
    }

    #[test]
    fn audit_default_policy_warn_and_block() {
        let policy = GuardPolicy::default();
        assert_eq!((policy.warn_at, policy.block_at), (2000, 8000));
        // warn 段：用 measure 造 ~2000 tokens（8000 chars 输入）
        let input = "x".repeat(8000);
        let meter = measure(&input, "", 0);
        assert_eq!(meter.input_tokens, 2000);
        let verdict = audit(&meter, &policy);
        assert!(matches!(verdict, super::GuardVerdict::Warn { .. }));
        assert!(verdict.reason().is_some_and(|r| !r.is_empty()));
        // block 段
        let big = "y".repeat(32000);
        let meter = measure(&big, "", 0);
        let verdict = audit(&meter, &policy);
        assert!(verdict.is_block());
        assert!(!verdict.is_pass());
    }

    #[test]
    fn inverted_thresholds_clamped_without_panic() {
        let policy = GuardPolicy::with_thresholds(9000, 100);
        assert_eq!((policy.warn_at, policy.block_at), (100, 100));
        let m = super::TokenMeter::new(100, 0, 0);
        assert!(audit(&m, &policy).is_block());
    }

    #[test]
    fn injector_evicts_oldest_beyond_cap() {
        let mut inj = LearningInjector::new();
        assert_eq!(inj.capacity(), MAX_LEARNINGS);
        for i in 0..MAX_LEARNINGS {
            inj.insert(&format!("k{i}"), &format!("v{i}"));
        }
        assert_eq!(inj.len(), MAX_LEARNINGS);
        inj.insert("k_new", "v_new");
        assert_eq!(inj.len(), MAX_LEARNINGS);
        // 最旧 k0 被逐出
        assert_eq!(inj.get("k0"), None);
        assert_eq!(inj.get("k_new").map(String::as_str), Some("v_new"));
        assert_eq!(inj.get("k1").map(String::as_str), Some("v1"));
    }

    #[test]
    fn injector_keyword_match_and_empty_guards() {
        let mut inj = LearningInjector::new();
        inj.insert("", "nope");
        inj.insert("k", "");
        assert!(inj.is_empty());
        inj.insert("token", "budget snippet");
        inj.insert("cache", "cache snippet");
        let hits = inj.inject("how to save TOKEN today?");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits.first().map(String::as_str), Some("budget snippet"));
        assert!(inj.inject("").is_empty());
        assert!(inj.inject("unrelated query").is_empty());
    }
}
