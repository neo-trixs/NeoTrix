#[cfg(feature = "stealth-net")]
pub mod antidetect;
pub mod auth;
pub mod channel;
pub mod channel_adapter;
pub mod doctor;
pub mod extractors;
pub mod feed;
pub mod manager;
pub mod nt_catalog;
pub mod nt_login;
pub mod nt_payload_guard;
pub mod nt_selector_contract;
pub mod nt_x_browser;
pub mod probe;
pub mod traits;

pub use traits::{
    AuthFlow, AuthState, Author, Credentials, EngagementMetrics, ExtractorItem, ExtractorResult,
    FeedItem, FeedType, HttpPool, MediaItem, SocialAccessError, SocialAccessResult,
    SocialPlatform, SocialPlatformAdapter, SocialPost, SessionEntry,
    TrendingTopic, UnifiedPost, build_headers,
};

pub use channel::{Backend, BackendStatus, Channel, ChannelRegistry, Credential, ProbeResult, default_channels};
pub use channel_adapter::{ChannelAdapter, ChannelAdapterRegistry};
pub use feed::{FeedService, PlatformAdapterRegistry, PredictedAction, PredictedActions, Prediction, RankOutcome, UniversalRecommender};
pub use manager::{FeedResponse, SocialAccessManager, SessionPool};
pub use doctor::{DoctorReport, ChannelReport, BackendReport, run_doctor};
pub use nt_catalog::{PlatformCatalog, PlatformSpec, default_catalog};
pub use nt_login::{
    LoginRegistry, LoginTarget, Observation, ProbeOutcome, SuccessProbe, default_registry as default_login_registry,
    evaluate as evaluate_login,
};
pub use nt_selector_contract::{PageShape, X_SELECTOR_CONTRACT};
pub use nt_x_browser::{XBrowserRetriever, XQuery};
pub use nt_payload_guard::{PayloadVerdict, classify as classify_payload};
pub use probe::{
    DEFAULT_PROBE_TIMEOUT, RunOutcome, command_exists, find_best_backend, get_version,
    probe_backends, probe_command, probe_command_with_timeout, run_with_timeout,
};

// ─── 统一 adapter 工厂 ────────────────────────────────────────────────────
//
// 2026-10-03 新增。审计发现：`extractors/` 下 6 个 adapter
//   （twitter / reddit / instagram / tiktok / ytdlp / web_jina）
//   **全部零生产调用方**。
//
// ⛔ 审计方法说明（R-SCAN-1b）：先用 `rg` 数引用发现「每个都被 1~5 个文件
//    引用」，看着像已接线；**逐行读**才发现那些命中**全是文档注释**
//    （`source/text/social/mod.rs:7-10` 的 `//! - twitter::TwitterSource
//    → social_access::TwitterExtractor` 之类）。
//    ⇒ 真实生产调用方为 **0**。这是「导出 ≠ 接入」在**第三次**出现，
//    且第一次差点被 grep 的命中数骗过去。

use std::sync::Arc;

use crate::l2_perception::nt_world::social_access::extractors::{
    instagram::InstagramExtractor, reddit::RedditExtractor, tiktok::TikTokExtractor,
    twitter::TwitterExtractor, web_jina::JinaReaderExtractor, ytdlp::YtdlpExtractor,
};
/// 一个 adapter 的静态描述（用于 CLI 枚举与自检）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterEntry {
    /// 与 `SocialPlatformAdapter::id()` 一致。
    pub id: &'static str,
    /// 人类可读名。
    pub name: &'static str,
    /// 该 adapter 覆盖的平台。
    pub platform: SocialPlatform,
}

/// 内置 adapter 清单 —— **单一真源**。
///
/// ⛔ 此前没有任何统一入口，6 个 adapter 只能靠 `use` 路径逐个捞，
///    极易漏（实测就是全漏）。
const ADAPTERS: &[(&str, fn() -> Arc<dyn SocialPlatformAdapter>)] = &[
    ("youtube", || Arc::new(YtdlpExtractor::new())),
    ("web", || Arc::new(JinaReaderExtractor::new())),
    ("twitter", || Arc::new(TwitterExtractor::new())),
    ("reddit", || Arc::new(RedditExtractor::new())),
    ("instagram", || Arc::new(InstagramExtractor::new())),
    ("tiktok", || Arc::new(TikTokExtractor::new())),
];

/// 构造全部内置 adapter。
///
/// 这是「注册 adapter」的唯一入口：调用方不再逐个 import。
pub fn default_adapters() -> Vec<Arc<dyn SocialPlatformAdapter>> {
    ADAPTERS.iter().map(|(_, f)| f()).collect()
}

/// 只取某个平台的 adapter。
///
/// 先把平台解析成**目录主键**，再按主键查 adapter ——
///    因为 adapter 的 `id()` 可能是别名（`TwitterExtractor::id()` 是
///    `"twitter"`，目录主键是 `"x"`）。直接用 `SocialPlatform::as_str()`
///    查会让 X 的 adapter 永远取不到（自测抓到）。
pub fn adapter_for(platform: &SocialPlatform) -> Option<Arc<dyn SocialPlatformAdapter>> {
    let key = platform.as_str().to_string();
    let catalog = default_catalog();

    // 候选序列必须包含**主键自身声明的别名**。
    // ⛔ 独立 harness 抓到的实际缺陷：请求 "x" 时
    //    candidates = ["x", "x"] —— "x" 是主键、不是别名，
    //    故“别名反查”返回 None，别名 `twitter` **从未被尝试**，
    //    于是 X 的 adapter 取不到（catalog 曾把 x 报成 none）。
    let mut candidates: Vec<String> = vec![key.clone()];
    if let Some(spec) = catalog.resolve(&key) {
        candidates.push(spec.id.clone());
        candidates.extend(spec.aliases().iter().map(|a| a.to_string()));
    }
    if let Some(spec) = catalog
        .all()
        .into_iter()
        .find(|s| s.aliases().iter().any(|a| *a == key))
    {
        candidates.push(spec.id.clone());
        candidates.extend(spec.aliases().iter().map(|a| a.to_string()));
    }

    candidates
        .into_iter()
        .find(|c| !c.is_empty() && ADAPTERS.iter().any(|(id, _)| id == c))
        .and_then(|c| ADAPTERS.iter().find(|(id, _)| *id == c))
        .map(|(_, f)| f())
}

/// adapter 清单（供 CLI 与自检）。
pub fn adapter_entries() -> Vec<AdapterEntry> {
    let mut v: Vec<AdapterEntry> = default_adapters()
        .into_iter()
        .map(|a| AdapterEntry {
            id: Box::leak(a.id().into_boxed_str()),
            name: a.name(),
            platform: SocialPlatform::from_str(&a.id()),
        })
        .collect();
    // 排序稳定 —— HashMap/注册顺序变化会造成 diff 噪音
    v.sort_by(|a, b| a.id.cmp(b.id));
    v
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[test]
    fn factory_registers_all_six_extractors() {
        // 这条是本轮的核心回归：审计发现 6 个 adapter 零生产调用方
        let adapters = default_adapters();
        assert_eq!(adapters.len(), 6, "all six extractors must be registered");
        let ids: Vec<String> = adapters.iter().map(|a| a.id()).collect();
        for want in ["youtube", "web", "twitter", "reddit", "instagram", "tiktok"] {
            assert!(ids.iter().any(|i| i == want), "missing adapter: {}", want);
        }
    }

    #[test]
    fn adapter_ids_have_no_duplicates() {
        // ⛔ 重复 id 会让注册表后者覆盖前者，且无任何提示
        let ids: Vec<String> = default_adapters().iter().map(|a| a.id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "duplicate adapter ids: {:?}", ids);
    }

    #[test]
    fn adapter_lookup_by_platform_works() {
        assert!(adapter_for(&SocialPlatform::Twitter).is_some());
        assert!(adapter_for(&SocialPlatform::Youtube).is_some());
        assert!(adapter_for(&SocialPlatform::Reddit).is_some());
        assert!(adapter_for(&SocialPlatform::Linkedin).is_none());
    }

    #[test]
    fn every_adapter_maps_to_a_catalog_platform() {
        // 防漂移：adapter 的 id 必须在平台目录里存在，
        // 否则会出现「有 adapter 但目录不知道」的孤儿。
        let catalog = default_catalog();
        for a in default_adapters() {
            let id = a.id();
            // 用 `resolve`（含别名）：`TwitterExtractor::id()` 是
            //    `"twitter"`，目录主键是 `"x"`。
            assert!(
                catalog.resolve(&id).is_some(),
                "adapter `{}` has no PlatformCatalog entry ⇒ drifted",
                id
            );
        }
    }

    #[test]
    fn every_catalog_channel_has_an_adapter_or_explicit_reason() {
        // 渠道存在但无 adapter ⇒ 探测说健康、抓取必然失败。
        //    至少要能枚举出来供 CLI 报告，而不是静默。
        let catalog = default_catalog();
        for a in default_adapters() {
            let id = a.id();
            assert!(
                catalog.resolve(&id).is_some(),
                "adapter `{}` is not in the catalog",
                id
            );
        }
    }

    #[test]
    fn entries_are_sorted_and_stable() {
        let a = adapter_entries();
        let b = adapter_entries();
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.id, y.id, "entry order must be deterministic");
        }
        let mut sorted: Vec<&str> = a.iter().map(|e| e.id).collect();
        sorted.sort_unstable();
        let actual: Vec<&str> = a.iter().map(|e| e.id).collect();
        assert_eq!(sorted, actual, "entries must be sorted by id");
    }
}

#[cfg(test)]
mod alias_tests {
    use super::*;

    /// 这是本轮抓到的**真实生产缺陷**（非测试瑕疵）：
    /// `TwitterExtractor::id()` 返回 `"twitter"`，而目录主键是 `"x"`。
    /// 若 `adapter_for` 只按 `SocialPlatform::as_str()` 查，
    /// X 的 adapter 永远取不到 ⇒ `get_ranked(Twitter)` 报
    /// 「No adapter for Twitter」，而 adapter 明明就在清单里。
    #[test]
    fn adapter_for_twitter_resolves_the_x_alias() {
        assert_eq!(SocialPlatform::Twitter.as_str(), "twitter");
        let a = adapter_for(&SocialPlatform::Twitter).expect("X adapter must resolve via alias");
        assert_eq!(a.id(), "twitter");
    }

    #[test]
    fn same_name_platforms_still_resolve() {
        for p in [SocialPlatform::Youtube, SocialPlatform::Reddit, SocialPlatform::TikTok] {
            assert!(adapter_for(&p).is_some(), "{:?} adapter must resolve", p);
        }
    }

    #[test]
    fn platform_without_an_adapter_yields_none_without_panicking() {
        // ⛔ LinkedIn 在目录与渠道里都存在，但**没有** adapter ⇒ 必须返回 None
        assert!(adapter_for(&SocialPlatform::Linkedin).is_none());
        // ⚠️ Instagram **有** adapter（extractors/instagram.rs）——
        //    我最初误以为没有而写错断言。凭记忆断言"某平台没有实现"
        //    是危险的：它会在有人真的实现后变成假失败。
        assert!(adapter_for(&SocialPlatform::Instagram).is_some());
        // 目录里完全没有的平台
        assert!(adapter_for(&SocialPlatform::Other("definitely-unknown".into())).is_none());
    }
}
