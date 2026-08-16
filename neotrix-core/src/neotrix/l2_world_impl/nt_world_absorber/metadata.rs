//! 多源元数据聚合 (P21, absorbed from bookorbit) — 纯函数、确定性融合。
//!
//! 从多个 provider (openlib / goodreads / amazon / google_books / douban /
//! worldcat / isbn / wikidata) 聚合书籍元数据到统一记录, 并支持三路版本同步
//! (PushLocal / PullRemote / Merge / NoOp)。零网络、零 I/O, 完全确定性 —
//! 相同输入恒得相同输出 (含并列决胜采用字典序)。

use crate::core::nt_core_self_test::SelfTest;

/// 单个元数据 provider 的静态描述。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetadataProvider {
    /// Provider 名 (与 `ProviderResult::provider` 对应)。
    pub name: &'static str,
    /// 聚合时该 provider 的基准权重。
    pub weight: f64,
    /// 该 provider 的历史典型延迟 (毫秒, 供调度/展示用, 不参与融合)。
    pub latency_ms: u64,
}

/// 内置 provider 静态表 (≥6, 与 task 契约对齐)。
pub static METADATA_PROVIDERS: [MetadataProvider; 8] = [
    MetadataProvider { name: "openlib", weight: 0.9, latency_ms: 120 },
    MetadataProvider { name: "goodreads", weight: 1.0, latency_ms: 200 },
    MetadataProvider { name: "amazon", weight: 0.7, latency_ms: 90 },
    MetadataProvider { name: "google_books", weight: 0.85, latency_ms: 150 },
    MetadataProvider { name: "douban", weight: 0.6, latency_ms: 110 },
    MetadataProvider { name: "worldcat", weight: 0.75, latency_ms: 300 },
    MetadataProvider { name: "isbn", weight: 0.5, latency_ms: 40 },
    MetadataProvider { name: "wikidata", weight: 0.95, latency_ms: 80 },
];

/// 单个 provider 返回的原始结果。
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderResult {
    pub provider: &'static str,
    pub title: Option<String>,
    pub author: Option<String>,
    pub rating: Option<f64>,
    pub confidence: f64,
}

impl ProviderResult {
    pub fn new(provider: &'static str) -> Self {
        Self {
            provider,
            title: None,
            author: None,
            rating: None,
            confidence: 1.0,
        }
    }
}

/// 聚合后的统一元数据记录。
#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedRecord {
    pub title: Option<String>,
    pub author: Option<String>,
    pub rating: Option<f64>,
    /// 实际贡献了至少一个字段的 provider 数量。
    pub sources_used: usize,
    /// 一致性 (0..=1): title+author 与选定值的符合比例。
    pub consensus: f64,
}

/// 多 provider 元数据聚合器 — 确定性加权融合。
#[derive(Debug, Clone)]
pub struct MetadataAggregator {
    providers: Vec<MetadataProvider>,
}

impl Default for MetadataAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataAggregator {
    pub fn new() -> Self {
        Self {
            providers: METADATA_PROVIDERS.to_vec(),
        }
    }

    /// 以自定义 provider 表构建 (主要用于测试与注入式配置)。
    pub fn with_providers(providers: Vec<MetadataProvider>) -> Self {
        Self { providers }
    }

    /// 确定性融合:
    /// - title/author 按 (provider weight × confidence) 加权投票, 并列取字典序最小 (输入序无关)。
    /// - rating 为加权均值, 权重 = provider weight × confidence。
    /// - consensus 为 title+author 各自符合比例的均值。
    pub fn aggregate(&self, results: &[ProviderResult]) -> AggregatedRecord {
        let sources_used = results
            .iter()
            .filter(|r| r.title.is_some() || r.author.is_some() || r.rating.is_some())
            .count();

        let title = self.weighted_field_vote(results, |r| r.title.clone());
        let author = self.weighted_field_vote(results, |r| r.author.clone());

        let mut num = 0.0;
        let mut den = 0.0;
        for r in results {
            if let Some(rating) = r.rating {
                let w = self.effective_weight(r);
                num += w * rating;
                den += w;
            }
        }
        let rating = if den > 0.0 { Some(num / den) } else { None };

        let consensus = self.compute_consensus(results, title.as_deref(), author.as_deref());

        AggregatedRecord {
            title,
            author,
            rating,
            sources_used,
            consensus,
        }
    }

    /// 共识分 (0..=1) — provider 在 title+author 上越一致, 分数越高。
    pub fn consensus_score(&self, record: &AggregatedRecord) -> f64 {
        record.consensus.clamp(0.0, 1.0)
    }

    fn provider_weight(&self, name: &str) -> f64 {
        self.providers
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.weight)
            .unwrap_or(1.0)
    }

    fn effective_weight(&self, r: &ProviderResult) -> f64 {
        self.provider_weight(r.provider) * r.confidence
    }

    /// 按有效权重对某字段候选值投票; 并列时取字典序最小, 保证输入序无关的确定性。
    fn weighted_field_vote<F>(&self, results: &[ProviderResult], field: F) -> Option<String>
    where
        F: Fn(&ProviderResult) -> Option<String>,
    {
        let mut scores: Vec<(String, f64)> = Vec::new();
        for r in results {
            if let Some(v) = field(r) {
                let score = self.effective_weight(r);
                if let Some(entry) = scores.iter_mut().find(|(k, _)| *k == v) {
                    entry.1 += score;
                } else {
                    scores.push((v, score));
                }
            }
        }
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.into_iter().next().map(|(v, _)| v)
    }

    /// 一致性: 分别计算 title / author 与选定值的符合比例, 取平均。
    fn compute_consensus(
        &self,
        results: &[ProviderResult],
        title: Option<&str>,
        author: Option<&str>,
    ) -> f64 {
        let mut t_num = 0.0;
        let mut t_den = 0.0;
        let mut a_num = 0.0;
        let mut a_den = 0.0;
        for r in results {
            if let Some(t) = r.title.as_deref() {
                t_den += 1.0;
                if title == Some(t) {
                    t_num += 1.0;
                }
            }
            if let Some(a) = r.author.as_deref() {
                a_den += 1.0;
                if author == Some(a) {
                    a_num += 1.0;
                }
            }
        }
        let t = if t_den > 0.0 { t_num / t_den } else { 0.0 };
        let a = if a_den > 0.0 { a_num / a_den } else { 0.0 };
        let fields = u8::from(t_den > 0.0) + u8::from(a_den > 0.0);
        if fields == 0 {
            0.0
        } else {
            (t + a) / fields as f64
        }
    }
}

/// 三路同步状态 (local / remote 各自版本 + 上次同步版本)。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThreeWayState {
    pub local: u32,
    pub remote: u32,
    pub last_sync: u32,
}

/// 三路同步动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    /// 本地较新, 推送本地。
    PushLocal,
    /// 远端较新, 拉取远端。
    PullRemote,
    /// 两侧都在 last_sync 之后有新版本, 合并 (计入 merges)。
    Merge,
    /// 无需动作。
    NoOp,
}

/// 同步动作计数报告。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub pushes: u32,
    pub pulls: u32,
    pub merges: u32,
    pub noops: u32,
}

impl SyncReport {
    fn record(&mut self, action: &SyncAction) {
        match action {
            SyncAction::PushLocal => self.pushes += 1,
            SyncAction::PullRemote => self.pulls += 1,
            SyncAction::Merge => self.merges += 1,
            SyncAction::NoOp => self.noops += 1,
        }
    }
}

/// 三路同步解析器 — 确定性版本比较, 并维护累计报告。
#[derive(Debug, Clone, Default)]
pub struct ThreeWaySync {
    pub report: SyncReport,
}

impl ThreeWaySync {
    pub fn new() -> Self {
        Self::default()
    }

    /// 解析一次同步请求, 更新累计报告。
    ///
    /// 规则:
    /// - last_sync 已不低于两侧版本 → NoOp (自上次同步无新版本)。
    /// - local == remote → NoOp (两侧已在同一版本, 无传输需要)。
    /// - 仅一侧在 last_sync 之后有新版本 → PushLocal / PullRemote。
    /// - 两侧都在 last_sync 之后有新版本 → Merge。
    pub fn resolve(&mut self, local_ver: u32, remote_ver: u32, last_sync_ver: u32) -> SyncAction {
        let action = if last_sync_ver >= local_ver && last_sync_ver >= remote_ver {
            SyncAction::NoOp
        } else if local_ver == remote_ver {
            SyncAction::NoOp
        } else if local_ver > remote_ver {
            if last_sync_ver >= remote_ver {
                SyncAction::PushLocal
            } else {
                SyncAction::Merge
            }
        } else if last_sync_ver >= local_ver {
            SyncAction::PullRemote
        } else {
            SyncAction::Merge
        };
        self.report.record(&action);
        action
    }

    /// 便捷入口: 对可变状态解析并推进 last_sync, 返回动作。
    pub fn sync(&mut self, state: &mut ThreeWayState) -> SyncAction {
        let action = self.resolve(state.local, state.remote, state.last_sync);
        state.last_sync = match action {
            SyncAction::PushLocal => state.local,
            SyncAction::PullRemote => state.remote,
            SyncAction::Merge => state.local.max(state.remote),
            SyncAction::NoOp => state.last_sync,
        };
        action
    }
}

impl SelfTest for MetadataAggregator {
    fn name(&self) -> &str {
        "nt_world_absorber_metadata_aggregate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let agg = Self::default();
        let mut failures: Vec<String> = Vec::new();

        let r = agg.aggregate(&[
            ProviderResult {
                provider: "goodreads",
                title: Some("Dune".into()),
                author: Some("Frank Herbert".into()),
                rating: Some(4.3),
                confidence: 0.9,
            },
            ProviderResult {
                provider: "amazon",
                title: Some("Dune".into()),
                author: Some("Frank Herbert".into()),
                rating: Some(4.2),
                confidence: 0.9,
            },
            ProviderResult {
                provider: "isbn",
                title: Some("DUNE".into()),
                author: Some("Herbert, Frank".into()),
                rating: Some(3.5),
                confidence: 0.5,
            },
        ]);
        if r.title.as_deref() != Some("Dune") {
            failures.push("weighted title vote failed".into());
        }
        if r.author.as_deref() != Some("Frank Herbert") {
            failures.push("weighted author vote failed".into());
        }
        if r.sources_used != 3 {
            failures.push("sources_used mismatch".into());
        }
        if !((r.consensus - 2.0 / 3.0).abs() < 1e-9) {
            failures.push("consensus mismatch".into());
        }

        let mut sync = ThreeWaySync::new();
        let _ = sync.resolve(4, 3, 2); // Merge
        let _ = sync.resolve(5, 3, 3); // PushLocal
        let _ = sync.resolve(2, 5, 2); // PullRemote
        let _ = sync.resolve(2, 2, 2); // NoOp
        if sync.report.merges != 1 {
            failures.push("merge count wrong".into());
        }
        if sync.report.pushes != 1 {
            failures.push("push count wrong".into());
        }
        if sync.report.pulls != 1 {
            failures.push("pull count wrong".into());
        }
        if sync.report.noops != 1 {
            failures.push("noop count wrong".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(
        provider: &'static str,
        title: Option<&str>,
        author: Option<&str>,
        rating: Option<f64>,
        confidence: f64,
    ) -> ProviderResult {
        ProviderResult {
            provider,
            title: title.map(|s| s.to_string()),
            author: author.map(|s| s.to_string()),
            rating,
            confidence,
        }
    }

    #[test]
    fn test_metadata_providers_list_minimum() {
        assert!(
            METADATA_PROVIDERS.len() >= 6,
            "must expose >=6 providers"
        );
        let names: Vec<&str> = METADATA_PROVIDERS.iter().map(|p| p.name).collect();
        for required in ["openlib", "goodreads", "amazon", "google_books", "douban", "worldcat", "isbn", "wikidata"] {
            assert!(names.contains(&required), "missing provider {}", required);
        }
    }

    #[test]
    fn test_aggregate_weighted_title_and_author_vote() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", Some("Dune"), Some("Frank Herbert"), Some(4.3), 0.9),
            res("amazon", Some("Dune"), Some("Frank Herbert"), Some(4.2), 0.9),
            res("isbn", Some("DUNE"), Some("Herbert, Frank"), Some(3.5), 0.5),
        ]);
        assert_eq!(r.title.as_deref(), Some("Dune"));
        assert_eq!(r.author.as_deref(), Some("Frank Herbert"));
        assert_eq!(r.sources_used, 3);
    }

    #[test]
    fn test_aggregate_author_weighted_vote_conflict() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", Some("Red Mars"), Some("Kim Stanley Robinson"), None, 0.8),
            res("amazon", Some("Red Mars"), Some("K. S. Robinson"), None, 0.8),
        ]);
        assert_eq!(r.title.as_deref(), Some("Red Mars"));
        assert_eq!(r.author.as_deref(), Some("Kim Stanley Robinson"));
    }

    #[test]
    fn test_aggregate_title_tie_break_deterministic() {
        // 两个 provider 权重相同 → 并列, 取字典序最小, 与输入顺序无关。
        let agg = MetadataAggregator::with_providers(vec![
            MetadataProvider { name: "a", weight: 1.0, latency_ms: 1 },
            MetadataProvider { name: "b", weight: 1.0, latency_ms: 1 },
        ]);
        let forward = agg.aggregate(&[
            res("a", Some("banana"), None, None, 1.0),
            res("b", Some("apple"), None, None, 1.0),
        ]);
        let reversed = agg.aggregate(&[
            res("b", Some("apple"), None, None, 1.0),
            res("a", Some("banana"), None, None, 1.0),
        ]);
        assert_eq!(forward.title.as_deref(), Some("apple"));
        assert_eq!(reversed.title.as_deref(), Some("apple"));
    }

    #[test]
    fn test_aggregate_rating_confidence_weighted_mean() {
        let agg = MetadataAggregator::new();
        // openlib w=0.9 c=1.0 → 0.9; goodreads w=1.0 c=0.5 → 0.5
        let r = agg.aggregate(&[
            res("openlib", Some("T"), None, Some(4.0), 1.0),
            res("goodreads", Some("T"), None, Some(5.0), 0.5),
        ]);
        let expected = (0.9 * 4.0 + 0.5 * 5.0) / (0.9 + 0.5);
        let got = r.rating.unwrap();
        assert!((got - expected).abs() < 1e-9, "got {}, want {}", got, expected);
    }

    #[test]
    fn test_aggregate_rating_zero_confidence_excluded() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", Some("T"), None, Some(4.0), 1.0),
            res("amazon", Some("T"), None, Some(1.0), 0.0),
        ]);
        assert!((r.rating.unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_consensus_score_high() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", Some("Dune"), Some("Frank Herbert"), None, 0.9),
            res("amazon", Some("Dune"), Some("Frank Herbert"), None, 0.9),
            res("openlib", Some("Dune"), Some("Frank Herbert"), None, 0.7),
        ]);
        assert_eq!(r.consensus, 1.0);
        assert_eq!(agg.consensus_score(&r), 1.0);
    }

    #[test]
    fn test_consensus_score_low() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", Some("Red Mars"), Some("Kim Stanley Robinson"), None, 0.9),
            res("amazon", Some("Blue Venus"), Some("K. S. Robinson"), None, 0.9),
        ]);
        // title 1/2, author 1/2 → mean 0.5
        assert!((r.consensus - 0.5).abs() < 1e-9);
        assert!((agg.consensus_score(&r) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_aggregate_empty_results() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[]);
        assert_eq!(r.title, None);
        assert_eq!(r.author, None);
        assert_eq!(r.rating, None);
        assert_eq!(r.sources_used, 0);
        assert_eq!(r.consensus, 0.0);
    }

    #[test]
    fn test_aggregate_all_none_results_excluded_from_sources() {
        let agg = MetadataAggregator::new();
        let r = agg.aggregate(&[
            res("goodreads", None, None, None, 1.0),
            res("isbn", None, None, None, 1.0),
        ]);
        assert_eq!(r.sources_used, 0);
        assert_eq!(r.consensus, 0.0);
    }

    #[test]
    fn test_sync_push_local() {
        let mut sync = ThreeWaySync::new();
        let action = sync.resolve(5, 3, 3);
        assert_eq!(action, SyncAction::PushLocal);
        assert_eq!(sync.report.pushes, 1);
    }

    #[test]
    fn test_sync_pull_remote() {
        let mut sync = ThreeWaySync::new();
        let action = sync.resolve(2, 5, 2);
        assert_eq!(action, SyncAction::PullRemote);
        assert_eq!(sync.report.pulls, 1);
    }

    #[test]
    fn test_sync_merge() {
        let mut sync = ThreeWaySync::new();
        let action = sync.resolve(5, 4, 3);
        assert_eq!(action, SyncAction::Merge);
        assert_eq!(sync.report.merges, 1);
    }

    #[test]
    fn test_sync_noop() {
        let mut sync = ThreeWaySync::new();
        assert_eq!(sync.resolve(3, 3, 3), SyncAction::NoOp);
        // equal but ahead of last_sync → still NoOp (already in sync)
        assert_eq!(sync.resolve(3, 3, 1), SyncAction::NoOp);
        // nothing new since last sync
        assert_eq!(sync.resolve(3, 2, 4), SyncAction::NoOp);
        assert_eq!(sync.report.noops, 3);
    }

    #[test]
    fn test_sync_merge_counting() {
        let mut sync = ThreeWaySync::new();
        assert_eq!(sync.resolve(4, 3, 2), SyncAction::Merge);
        assert_eq!(sync.resolve(6, 5, 4), SyncAction::Merge);
        assert_eq!(sync.resolve(7, 4, 4), SyncAction::PushLocal);
        assert_eq!(sync.report.merges, 2);
        assert_eq!(sync.report.pushes, 1);
    }

    #[test]
    fn test_sync_state_update() {
        let mut sync = ThreeWaySync::new();
        let mut st = ThreeWayState { local: 5, remote: 4, last_sync: 3 };
        assert_eq!(sync.sync(&mut st), SyncAction::Merge);
        assert_eq!(st.last_sync, 5);

        let mut st = ThreeWayState { local: 6, remote: 4, last_sync: 4 };
        assert_eq!(sync.sync(&mut st), SyncAction::PushLocal);
        assert_eq!(st.last_sync, 6);

        let mut st = ThreeWayState { local: 2, remote: 6, last_sync: 2 };
        assert_eq!(sync.sync(&mut st), SyncAction::PullRemote);
        assert_eq!(st.last_sync, 6);
    }

    #[test]
    fn test_selftest_metadata_aggregate() {
        let agg = MetadataAggregator::default();
        assert_eq!(agg.name(), "nt_world_absorber_metadata_aggregate");
        assert!(agg.self_test().is_ok());
    }
}
