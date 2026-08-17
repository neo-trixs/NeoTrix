// Batch C 吸收 (2026-08-15 sweep): P6 / P15 / P16 / P17
// 单一文件内聚合四个记忆层能力, 统一注入 nt_memory_kb 域 (R-P42: 强化既有节点):
//   P6  KvCacheMemory      (arXiv 2602.23592 KEEP — KV 缓存 → 长程记忆)
//   P15 MutationGuard      (iwe 内存突变保护)
//   P16 RetrievalMatrix    (rag-from-scratch 检索矩阵: 语义+BM25 混合)
//   P17 SingleFileMemory   (claude-brain 单文件记忆栈)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ────────────────────────────────────────────────────────────────
// P6: KvCacheMemory — KV 缓存即记忆 (KEEP)
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: Vec<f64>,
    pub access_count: u64,
    pub last_access: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KvCacheMemory {
    entries: HashMap<String, CacheEntry>,
    capacity: usize,
    clock: u64,
}

impl KvCacheMemory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity,
            clock: 0,
        }
    }

    /// 写入 (KEEP 语义: 高访问率条目晋升为持久记忆, 不被 LRU 驱逐)。
    pub fn put(&mut self, key: impl Into<String>, value: Vec<f64>) {
        self.clock += 1;
        let key = key.into();
        if let Some(e) = self.entries.get_mut(&key) {
            e.value = value;
            e.access_count += 1;
            e.last_access = self.clock;
            return;
        }
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }
        self.entries.insert(
            key.clone(),
            CacheEntry {
                key,
                value,
                access_count: 1,
                last_access: self.clock,
            },
        );
    }

    fn evict_lru(&mut self) {
        if let Some((k, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| (e.access_count, e.last_access))
            .map(|(k, e)| (k.clone(), e.access_count))
        {
            self.entries.remove(&k);
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&[f64]> {
        self.clock += 1;
        if let Some(e) = self.entries.get_mut(key) {
            e.access_count += 1;
            e.last_access = self.clock;
            Some(&e.value)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 晋升率: 访问密集条目占比 (KEEP 信号)。
    pub fn promotion_ratio(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let hot = self
            .entries
            .values()
            .filter(|e| e.access_count >= 3)
            .count();
        hot as f64 / self.entries.len() as f64
    }
}

// ────────────────────────────────────────────────────────────────
// P15: MutationGuard — 记忆突变保护 (iwe)
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationKind {
    Insert,
    Update,
    Delete,
    Rewrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEvent {
    pub kind: MutationKind,
    pub target: String,
    pub checksum: u64,
    pub origin: String,
    pub size_bytes: usize,
}

impl MutationEvent {
    pub fn new(kind: MutationKind, target: impl Into<String>, origin: impl Into<String>) -> Self {
        Self {
            kind,
            target: target.into(),
            checksum: 0,
            origin: origin.into(),
            size_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationGuard {
    max_daily_bytes: usize,
    daily_bytes: HashMap<String, usize>,
    blocked: u64,
    allowed: u64,
}

impl MutationGuard {
    pub fn with_budget(max_daily_bytes: usize) -> Self {
        Self {
            max_daily_bytes,
            daily_bytes: HashMap::new(),
            blocked: 0,
            allowed: 0,
        }
    }

    pub fn default_budget() -> Self {
        Self::with_budget(50 * 1024 * 1024)
    }

    /// 突变审批: 预算耗尽 → 拦截 (iwe 防失控写入)。
    pub fn approve(&mut self, event: &MutationEvent) -> bool {
        let day = event.target.clone();
        let used = self.daily_bytes.entry(day.clone()).or_insert(0);
        if *used + event.size_bytes > self.max_daily_bytes {
            self.blocked += 1;
            return false;
        }
        *used += event.size_bytes;
        self.allowed += 1;
        true
    }

    pub fn usage(&self, target: &str) -> usize {
        self.daily_bytes.get(target).copied().unwrap_or(0)
    }

    pub fn blocked_count(&self) -> u64 {
        self.blocked
    }
}

// ────────────────────────────────────────────────────────────────
// P16: RetrievalMatrix — 语义 + BM25 混合检索 (rag-from-scratch)
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalChannel {
    Semantic,
    Keyword,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    pub doc_id: String,
    pub score: f64,
    pub channel: RetrievalChannel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetrievalMatrix {
    semantic_index: HashMap<String, Vec<f64>>,
    keyword_index: HashMap<String, Vec<(String, u32)>>,
}

// ────────────────────────────────────────────────────────────────
// P0-3: UntrustedFence (nt_memory_untrusted_fence) — 未信任记忆隔离围栏。
// 从外部/爬虫/非审核源吸收的记忆 (untrusted) 与内部可信记忆 (trusted) 物理隔离。
// 检索时默认不混入 untrusted, 除非显式请求 (is_fenced_doc 谓词)。
// 机制来源: wms data 清洗/隔离 (R-P79 生产接线: RetrievalMatrix 消费)。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceKind {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone)]
pub struct FencedContent {
    pub doc_id: String,
    pub kind: FenceKind,
    pub source_url: Option<String>,
}

impl FencedContent {
    pub fn trusted(doc_id: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            kind: FenceKind::Trusted,
            source_url: None,
        }
    }

    pub fn untrusted(doc_id: impl Into<String>, source_url: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            kind: FenceKind::Untrusted,
            source_url: Some(source_url.into()),
        }
    }

    pub fn is_trusted(&self) -> bool {
        self.kind == FenceKind::Trusted
    }

    pub fn is_untrusted(&self) -> bool {
        self.kind == FenceKind::Untrusted
    }
}

#[derive(Debug, Clone, Default)]
pub struct UntrustedFence {
    /// doc_id → 围栏元数据 (trusted/untrusted + 来源)。
    fences: HashMap<String, FencedContent>,
    /// 围栏边界标记 — untrusted 内容写入时包上, 读取时校验剥离。
    /// (文本围栏语义: 未闭合即拒绝消费, 防部分提取污染)。
    pub marker_open: &'static str,
    pub marker_close: &'static str,
}

pub const UNTRUSTED_OPEN: &str = "\u{FFFD}__NT_UNTRUSTED__\u{FFFD}";
pub const UNTRUSTED_CLOSE: &str = "\u{FFFD}__NT_UNTRUSTED_END__\u{FFFD}";

impl UntrustedFence {
    pub fn new() -> Self {
        Self {
            fences: HashMap::new(),
            marker_open: UNTRUSTED_OPEN,
            marker_close: UNTRUSTED_CLOSE,
        }
    }

    /// 围栏写入: 把 untrusted 内容包上边界标记。
    pub fn fence(&mut self, doc_id: impl Into<String>, content: &str, source_url: impl Into<String>) -> String {
        let id = doc_id.into();
        self.fences.insert(
            id.clone(),
            FencedContent::untrusted(id, source_url),
        );
        format!("{}{}{}", self.marker_open, content, self.marker_close)
    }

    /// 围栏读取: 剥离边界标记。未闭合 → None (拒绝消费)。
    pub fn unwrap(&self, content: &str) -> Option<String> {
        if content.starts_with(self.marker_open) && content.ends_with(self.marker_close) {
            Some(content[self.marker_open.len()..content.len() - self.marker_close.len()].to_string())
        } else if content.contains(self.marker_open) || content.contains(self.marker_close) {
            None
        } else {
            Some(content.to_string())
        }
    }

    /// 是否为围栏文档 (fence 内登记过)。
    pub fn is_fenced(&self, doc_id: &str) -> bool {
        self.fences.contains_key(doc_id)
    }

    pub fn fence_kind(&self, doc_id: &str) -> Option<FenceKind> {
        self.fences.get(doc_id).map(|f| f.kind)
    }

    pub fn source_url(&self, doc_id: &str) -> Option<&str> {
        self.fences.get(doc_id).and_then(|f| f.source_url.as_deref())
    }

    pub fn fence_count(&self) -> usize {
        self.fences.len()
    }
}

// ────────────────────────────────────────────────────────────────
// P0-4: QualityRanker (nt_memory_quality_rank) — 检索质量排序器。
// 按来源分级 (peer-reviewed > official docs > blog > social > crowdsourced),
// 引用权威度 + 语义中心度加权, 撤回 (retracted) 文档强制降权到垫底。
// 机制来源: content 质量分级 (R-P79 生产接线: hybrid_search_quality 消费)。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceTier {
    PeerReviewed,
    OfficialDocs,
    Blog,
    Social,
    Crowdsourced,
}

impl SourceTier {
    pub fn weight(&self) -> f64 {
        match self {
            SourceTier::PeerReviewed => 1.0,
            SourceTier::OfficialDocs => 0.9,
            SourceTier::Blog => 0.6,
            SourceTier::Social => 0.4,
            SourceTier::Crowdsourced => 0.3,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SourceTier::PeerReviewed => "peer-reviewed",
            SourceTier::OfficialDocs => "official",
            SourceTier::Blog => "blog",
            SourceTier::Social => "social",
            SourceTier::Crowdsourced => "crowdsourced",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QualityRanker {
    /// doc_id → 来源分级。
    tiers: HashMap<String, SourceTier>,
    /// doc_id → 引用权威度 [0,1]。
    citation_authority: HashMap<String, f64>,
    /// doc_id → 语义中心度 [0,1]。
    centrality: HashMap<String, f64>,
    /// 已撤回 (retracted) 文档集。
    retracted: std::collections::HashSet<String>,
}

impl QualityRanker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tier(&mut self, doc_id: impl Into<String>, tier: SourceTier) {
        self.tiers.insert(doc_id.into(), tier);
    }

    pub fn set_citation_authority(&mut self, doc_id: impl Into<String>, a: f64) {
        self.citation_authority.insert(doc_id.into(), a.clamp(0.0, 1.0));
    }

    pub fn set_centrality(&mut self, doc_id: impl Into<String>, c: f64) {
        self.centrality.insert(doc_id.into(), c.clamp(0.0, 1.0));
    }

    pub fn retract(&mut self, doc_id: impl Into<String>) {
        self.retracted.insert(doc_id.into());
    }

    /// 质量分 = tier_weight × (0.5 + 0.3×authority + 0.2×centrality)。
    /// 撤回文档 → 直接返回 0.0 (垫底, 永不出现在顶部)。
    pub fn score(&self, doc_id: &str) -> f64 {
        if self.retracted.contains(doc_id) {
            return 0.0;
        }
        let tier_w = self.tiers.get(doc_id).map(|t| t.weight()).unwrap_or(0.5);
        let authority = self.citation_authority.get(doc_id).copied().unwrap_or(0.5);
        let centrality = self.centrality.get(doc_id).copied().unwrap_or(0.5);
        tier_w * (0.5 + 0.3 * authority + 0.2 * centrality)
    }

    /// 质量感知重排: 在基础分数上乘质量系数 (撤回 → 0)。
    pub fn rerank(&self, hits: &mut Vec<RetrievalHit>) {
        for h in hits.iter_mut() {
            h.score *= self.score(&h.doc_id);
        }
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

impl RetrievalMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_semantic(&mut self, doc_id: impl Into<String>, embedding: Vec<f64>) {
        self.semantic_index.insert(doc_id.into(), embedding);
    }

    pub fn index_keywords(&mut self, doc_id: impl Into<String>, terms: Vec<(String, u32)>) {
        self.keyword_index.insert(doc_id.into(), terms);
    }

    pub fn semantic_search(&self, query: &[f64], k: usize) -> Vec<RetrievalHit> {
        let mut hits: Vec<RetrievalHit> = self
            .semantic_index
            .iter()
            .map(|(id, emb)| RetrievalHit {
                doc_id: id.clone(),
                score: cosine(query, emb),
                channel: RetrievalChannel::Semantic,
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(k);
        hits
    }

    /// 简化 BM25: 命中词数 / 文档词数。
    pub fn keyword_search(&self, query_terms: &[&str], k: usize) -> Vec<RetrievalHit> {
        let mut hits = Vec::new();
        for (id, terms) in &self.keyword_index {
            let total: u32 = terms.iter().map(|(_, c)| c).sum();
            let matched: u32 = terms
                .iter()
                .filter(|(t, _)| query_terms.contains(&t.as_str()))
                .map(|(_, c)| c)
                .sum();
            let matched_terms: usize = terms
                .iter()
                .filter(|(t, _)| query_terms.contains(&t.as_str()))
                .count();
            if matched > 0 && total > 0 {
                hits.push(RetrievalHit {
                    doc_id: id.clone(),
                    // 匹配词数优先 (更多词命中排更高), 频率比做次级 tiebreak
                    score: matched_terms as f64 * 1000.0 + (matched as f64 / total as f64),
                    channel: RetrievalChannel::Keyword,
                });
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(k);
        hits
    }

    /// 混合检索: 语义分 0.6 + 关键词分 0.4 (RRF 简化)。
    pub fn hybrid_search(
        &self,
        query: &[f64],
        query_terms: &[&str],
        k: usize,
    ) -> Vec<RetrievalHit> {
        let mut scores: HashMap<String, (f64, f64)> = HashMap::new();
        for hit in self.semantic_search(query, usize::MAX) {
            scores.entry(hit.doc_id.clone()).or_insert((0.0, 0.0)).0 = hit.score;
        }
        for hit in self.keyword_search(query_terms, usize::MAX) {
            scores.entry(hit.doc_id.clone()).or_insert((0.0, 0.0)).1 = hit.score;
        }
        let mut hits: Vec<RetrievalHit> = scores
            .into_iter()
            .map(|(doc_id, (s, k))| RetrievalHit {
                doc_id,
                score: 0.6 * s + 0.4 * k,
                channel: RetrievalChannel::Hybrid,
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(k);
        hits
    }

    pub fn semantic_len(&self) -> usize {
        self.semantic_index.len()
    }
}

impl RetrievalMatrix {
    /// P0-4 接线 (R-P79): 质量感知混合检索 — 基础 hybrid_search 结果经
    /// QualityRanker 重排 (tier/authority/centrality 加权, 撤回垫底)。
    /// UntrustedFence 登记过的 untrusted 文档默认过滤 (除非 include_untrusted)。
    pub fn hybrid_search_quality(
        &self,
        query: &[f64],
        query_terms: &[&str],
        k: usize,
        ranker: &QualityRanker,
        fence: &UntrustedFence,
        include_untrusted: bool,
    ) -> Vec<RetrievalHit> {
        let mut hits = self.hybrid_search(query, query_terms, usize::MAX);
        if !include_untrusted {
            hits.retain(|h| !fence.is_fenced(&h.doc_id));
        }
        ranker.rerank(&mut hits);
        hits.truncate(k);
        hits
    }

    /// P0-3 接线 (R-P79): 是否为围栏文档 (untrusted 隔离谓词, 供上层检索过滤)。
    pub fn is_fenced_doc(&self, doc_id: &str, fence: &UntrustedFence) -> bool {
        fence.is_fenced(doc_id)
    }
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

// ────────────────────────────────────────────────────────────────
// P17: SingleFileMemory — 单文件记忆栈 (claude-brain)
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTierKind {
    Permanent,
    Working,
    Ephemeral,
}

impl MemoryTierKind {
    pub fn retention(&self) -> u64 {
        match self {
            MemoryTierKind::Permanent => u64::MAX,
            MemoryTierKind::Working => 30 * 24 * 3600,
            MemoryTierKind::Ephemeral => 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub tier: MemoryTierKind,
    pub content: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SingleFileMemory {
    records: Vec<MemoryRecord>,
    next_id: u64,
}

impl SingleFileMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, tier: MemoryTierKind, content: impl Into<String>) -> &MemoryRecord {
        let id = format!("mem-{}", self.next_id);
        self.next_id += 1;
        let record = MemoryRecord {
            id: id.clone(),
            tier,
            content: content.into(),
            created_at: now_secs(),
        };
        self.records.push(record);
        self.records.last().expect("record just pushed")
    }

    pub fn recall(&self, query: &str) -> Vec<&MemoryRecord> {
        let q = query.to_lowercase();
        let mut hits: Vec<&MemoryRecord> = self
            .records
            .iter()
            .filter(|r| r.content.to_lowercase().contains(&q))
            .collect();
        hits.sort_by_key(|r| std::cmp::Reverse(r.created_at));
        hits
    }

    pub fn by_tier(&self, tier: MemoryTierKind) -> Vec<&MemoryRecord> {
        self.records.iter().filter(|r| r.tier == tier).collect()
    }

    /// 过期清理 (Ephemeral/Working 到期)。
    pub fn purge_expired(&mut self, now: u64) -> usize {
        let before = self.records.len();
        self.records
            .retain(|r| now.saturating_sub(r.created_at) < r.tier.retention());
        before - self.records.len()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ────────────────────────────────────────────────────────────────
// SelfTest 聚合 (T1): 单一 SelfTest 覆盖四能力
// ────────────────────────────────────────────────────────────────

pub struct SweepMemoryCapabilitiesSelfTest;

impl crate::core::nt_core_self_test::SelfTest for SweepMemoryCapabilitiesSelfTest {
    fn name(&self) -> &str {
        "nt_memory_kb_sweep_capabilities"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut cache = KvCacheMemory::with_capacity(4);
        cache.put("a", vec![1.0]);
        if cache.len() != 1 {
            return Err(vec!["kv cache should hold 1".into()]);
        }

        let mut guard = MutationGuard::with_budget(10);
        let ev = MutationEvent::new(MutationKind::Insert, "node:x", "selftest");
        let mut ev = ev;
        ev.size_bytes = 5;
        if !guard.approve(&ev) {
            return Err(vec!["small mutation should pass".into()]);
        }

        let mut matrix = RetrievalMatrix::new();
        matrix.index_semantic("d1", vec![1.0, 0.0]);
        if matrix.semantic_len() != 1 {
            return Err(vec!["matrix should index 1 doc".into()]);
        }

        let mut sf = SingleFileMemory::new();
        sf.push(MemoryTierKind::Permanent, "alpha");
        if sf.recall("alpha").len() != 1 {
            return Err(vec!["single-file memory recall failed".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    // ── P6 ──
    #[test]
    fn test_kv_put_get() {
        let mut cache = KvCacheMemory::with_capacity(4);
        cache.put("k", vec![0.5, 0.5]);
        assert_eq!(cache.get("k"), Some(&[0.5, 0.5][..]));
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn test_kv_evicts_lru() {
        let mut cache = KvCacheMemory::with_capacity(2);
        cache.put("a", vec![1.0]);
        cache.put("b", vec![2.0]);
        cache.get("a"); // a 热
        cache.put("c", vec![3.0]); // b 被驱逐
        assert!(cache.get("b").is_none());
        assert!(cache.get("a").is_some());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_kv_promotion_ratio() {
        let mut cache = KvCacheMemory::with_capacity(10);
        cache.put("hot", vec![1.0]);
        for _ in 0..5 {
            cache.get("hot");
        }
        cache.put("cold", vec![1.0]);
        let ratio = cache.promotion_ratio();
        assert!(ratio > 0.0 && ratio <= 1.0);
    }

    // ── P15 ──
    #[test]
    fn test_mutation_budget_blocks_overflow() {
        let mut guard = MutationGuard::with_budget(10);
        let mut ev = MutationEvent::new(MutationKind::Insert, "t", "origin");
        ev.size_bytes = 8;
        assert!(guard.approve(&ev));
        ev.size_bytes = 8;
        assert!(!guard.approve(&ev), "over budget must block");
        assert_eq!(guard.blocked_count(), 1);
        assert_eq!(guard.usage("t"), 8);
    }

    #[test]
    fn test_mutation_budget_per_target() {
        let mut guard = MutationGuard::with_budget(10);
        let mut ev = MutationEvent::new(MutationKind::Rewrite, "t1", "o");
        ev.size_bytes = 6;
        assert!(guard.approve(&ev));
        let mut ev2 = MutationEvent::new(MutationKind::Rewrite, "t2", "o");
        ev2.size_bytes = 6;
        assert!(guard.approve(&ev2), "independent targets share no budget");
    }

    #[test]
    fn test_mutation_budget_exact_boundary_allows() {
        let mut guard = MutationGuard::with_budget(10);
        let mut ev = MutationEvent::new(MutationKind::Insert, "t", "origin");
        ev.size_bytes = 10;
        assert!(guard.approve(&ev), "exact budget boundary must allow");
        assert_eq!(guard.usage("t"), 10);
        // 再进 1 字节 → 超预算拦截
        ev.size_bytes = 1;
        assert!(!guard.approve(&ev));
    }

    #[test]
    fn test_mutation_budget_approve_accumulates_allowed() {
        let mut guard = MutationGuard::with_budget(100);
        let mut ev = MutationEvent::new(MutationKind::Rewrite, "t", "o");
        ev.size_bytes = 4;
        assert!(guard.approve(&ev));
        assert!(guard.approve(&ev));
        assert_eq!(guard.blocked_count(), 0);
        assert_eq!(guard.usage("t"), 8, "allowed bytes accumulate per target");
        // blocked_count 语义: 仅拦截计 (供审计/自愈触发器)
        ev.size_bytes = 500;
        assert!(!guard.approve(&ev));
        assert_eq!(guard.blocked_count(), 1);
    }

    // ── P16 ──
    #[test]
    fn test_matrix_semantic_rank() {
        let mut matrix = RetrievalMatrix::new();
        matrix.index_semantic("near", vec![1.0, 0.0]);
        matrix.index_semantic("far", vec![0.0, 1.0]);
        let hits = matrix.semantic_search(&[0.9, 0.1], 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, "near");
    }

    #[test]
    fn test_matrix_keyword_match() {
        let mut matrix = RetrievalMatrix::new();
        matrix.index_keywords("doc1", vec![("rust".into(), 3), ("memory".into(), 1)]);
        matrix.index_keywords("doc2", vec![("rust".into(), 1)]);
        let hits = matrix.keyword_search(&["rust", "memory"], 2);
        assert_eq!(hits[0].doc_id, "doc1", "more matches rank higher");
    }

    #[test]
    fn test_matrix_hybrid_combines() {
        let mut matrix = RetrievalMatrix::new();
        matrix.index_semantic("both", vec![1.0, 0.0]);
        matrix.index_keywords("both", vec![("rust".into(), 2)]);
        matrix.index_semantic("sem", vec![0.9, 0.1]);
        let hits = matrix.hybrid_search(&[1.0, 0.0], &["rust"], 2);
        assert_eq!(hits[0].doc_id, "both");
    }

    // ── P17 ──
    #[test]
    fn test_single_file_push_recall() {
        let mut sf = SingleFileMemory::new();
        sf.push(MemoryTierKind::Working, "hello world");
        sf.push(MemoryTierKind::Working, "rust is great");
        assert_eq!(sf.recall("world").len(), 1);
        assert_eq!(sf.recall("rust").len(), 1);
    }

    #[test]
    fn test_single_file_tiers() {
        let mut sf = SingleFileMemory::new();
        sf.push(MemoryTierKind::Permanent, "perm");
        sf.push(MemoryTierKind::Ephemeral, "eph");
        assert_eq!(sf.by_tier(MemoryTierKind::Permanent).len(), 1);
        assert_eq!(sf.by_tier(MemoryTierKind::Ephemeral).len(), 1);
    }

    #[test]
    fn test_single_file_purge_expired() {
        let mut sf = SingleFileMemory::new();
        sf.push(MemoryTierKind::Ephemeral, "old");
        let purged = sf.purge_expired(now_secs() + 7200);
        assert_eq!(purged, 1, "ephemeral (1h) expired at +2h");
        assert!(sf.is_empty());
    }

    // ── SelfTest ──
    #[test]
    fn test_aggregate_selftest() {
        let t = SweepMemoryCapabilitiesSelfTest;
        assert!(t.self_test().is_ok());
    }

    // ── P0-3 UntrustedFence ──
    #[test]
    fn test_untrusted_fence_roundtrip() {
        let mut fence = UntrustedFence::new();
        let wrapped = fence.fence("d1", "external raw content", "https://example.com/x");
        assert!(wrapped.starts_with(UNTRUSTED_OPEN));
        assert!(fence.is_fenced("d1"));
        assert_eq!(fence.fence_kind("d1"), Some(FenceKind::Untrusted));
        assert_eq!(fence.source_url("d1"), Some("https://example.com/x"));
        let inner = fence.unwrap(&wrapped);
        assert_eq!(inner.as_deref(), Some("external raw content"));
    }

    #[test]
    fn test_untrusted_fence_rejects_unclosed() {
        let fence = UntrustedFence::new();
        let broken = format!("{}{}", UNTRUSTED_OPEN, "no close marker");
        assert!(fence.unwrap(&broken).is_none(), "未闭合 → 拒绝消费");
        assert_eq!(fence.unwrap("plain"), Some("plain".to_string()));
    }

    #[test]
    fn test_retrieval_matrix_fenced_filter() {
        let mut matrix = RetrievalMatrix::new();
        let mut fence = UntrustedFence::new();
        matrix.index_keywords("trusted", vec![("a".into(), 3)]);
        matrix.index_keywords("untrusted", vec![("a".into(), 3)]);
        fence.fence("untrusted", "raw", "http://x");
        let hits = matrix.hybrid_search_quality(&[], &["a"], 10, &QualityRanker::new(), &fence, false);
        assert!(hits.iter().all(|h| h.doc_id != "untrusted"), "untrusted 默认过滤");
        assert!(matrix.is_fenced_doc("untrusted", &fence));
        let all = matrix.hybrid_search_quality(&[], &["a"], 10, &QualityRanker::new(), &fence, true);
        assert_eq!(all.len(), 2, "include_untrusted 时全部返回");
    }

    // ── P0-4 QualityRanker ──
    #[test]
    fn test_quality_ranker_peer_reviewed_top() {
        let mut r = QualityRanker::new();
        r.set_tier("a", SourceTier::PeerReviewed);
        r.set_tier("b", SourceTier::Crowdsourced);
        assert!(r.score("a") > r.score("b"));
    }

    #[test]
    fn test_quality_ranker_retracted_bottom() {
        let mut r = QualityRanker::new();
        r.set_tier("a", SourceTier::PeerReviewed);
        r.set_citation_authority("a", 1.0);
        r.retract("a");
        assert_eq!(r.score("a"), 0.0, "撤回 → 0");
    }
}
