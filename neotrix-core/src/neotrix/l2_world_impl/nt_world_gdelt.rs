#![forbid(unsafe_code)]

//! GDELT DOC 2.0 API 接线 — Wave6 H4 首个情报工具 (P1, 10工具中首个)
//!
//! - **数据源**: GDELT 2.0 DOC API (`api.gdeltproject.org`, 免费无key)
//! - **端点**: `https://api.gdeltproject.org/api/v2/doc/doc?query=...&mode=ArtList&format=json`
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `GdeltBackend`
//!   作为有序路由第三后端 (DDG → Wikipedia → GDELT)，亦可独立作为 `GdeltFetcher`
//!   直喂 intel-watch (R-P79 具名消费者)
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Article, domain=gdelt)
//! - **Egress**: `nt_shield_sandbox::INTEL_GDELT_HOST` allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定
//!
//! GDELT ArtList 响应形如：`{"articles":[{"title","url","seendate","domain","language"}]}`

use serde::{Deserialize, Serialize};

// ── GDELT 文章模型 ───────────────────────────────────────────────

/// 单篇 GDELT 文章 (ArtList 模式)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltArticle {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub seendate: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub language: String,
    #[serde(default, alias = "sourcecountry")]
    pub sourcecountry: String,
    #[serde(default, alias = "socialimage")]
    pub socialimage: String,
}

/// GDELT DOC 2.0 响应 — 仅解析需要的 `articles` 字段，余下忽略。
#[derive(Debug, Deserialize)]
struct GdeltDocResponse {
    #[serde(default)]
    articles: Vec<GdeltArticle>,
}

/// 兜底：GDELT 有时返回 `{"status":...,"articles":[]}` 或顶层数组，直接容错。
fn parse_gdelt_json(raw: &str) -> Result<Vec<GdeltArticle>, String> {
    // 1. 标准对象 {"articles":[...]}
    if let Ok(doc) = serde_json::from_str::<GdeltDocResponse>(raw) {
        if !doc.articles.is_empty() || raw.contains("\"articles\"") {
            return Ok(doc.articles);
        }
    }
    // 2. 顶层数组 [...]
    if let Ok(arr) = serde_json::from_str::<Vec<GdeltArticle>>(raw) {
        return Ok(arr);
    }
    // 3. 空或错误体 → 空列表
    if raw.trim().is_empty() || raw.contains("\"status\"") {
        return Ok(vec![]);
    }
    Err(format!("GDELT parse failed: {}", &raw.chars().take(300).collect::<String>()))
}

// ── Fetch 层 ───────────────────────────────────────────────────

/// GDELT DOC 2.0 Fetcher — 免费无key，直喂 intel-watch。
///
/// `base_url` 默认为 `https://api.gdeltproject.org`，测试可注入本地 fixture URL。
pub struct GdeltFetcher {
    base_url: String,
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for GdeltFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl GdeltFetcher {
    /// 生产构造 — 指向真实 GDELT。
    pub fn new() -> Self {
        Self::with_base_url("https://api.gdeltproject.org")
    }

    /// 测试/注入构造 — 指向任意 base (如 mock server)。
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: std::sync::OnceLock::new(),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (GDELT intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    /// 构造请求 URL。
    pub fn build_url(&self, query: &str, max_records: usize) -> String {
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        format!(
            "{}/api/v2/doc/doc?query={}&mode=ArtList&format=json&maxrecords={}&sort=HybridRel",
            self.base_url, encoded, max_records.max(1).min(250)
        )
    }

    /// 抓取并解析 (网络依赖)。
    pub fn fetch(&self, query: &str, max_records: usize) -> Result<Vec<GdeltArticle>, String> {
        let url = self.build_url(query, max_records);
        let resp = self
            .client()
            .get(&url)
            .send()
            .map_err(|e| format!("GDELT request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GDELT returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("GDELT read body: {}", e))?;
        parse_gdelt_json(&text)
    }

    /// 纯解析 (无网络，用于 fixture/单测)。
    pub fn parse_articles(json: &str) -> Result<Vec<GdeltArticle>, String> {
        parse_gdelt_json(json)
    }

    /// 从给定 JSON 字符串解析并入库 (无网络，用于 fixture E2E)。
    pub fn ingest_from_json(
        &self,
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        json: &str,
        query: &str,
    ) -> Result<GdeltIngestReport, String> {
        let articles = parse_gdelt_json(json)?;
        Self::ingest_articles(kb, &articles, query)
    }

    /// E2E 入库：fetch → 解析 → KB `insert_or_get_node` (Article, domain=gdelt)。
    pub fn ingest(
        &self,
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        query: &str,
        max_records: usize,
    ) -> Result<GdeltIngestReport, String> {
        let articles = self.fetch(query, max_records)?;
        Self::ingest_articles(kb, &articles, query)
    }

    /// 将已解析的 articles 入库 — 可复用 (fetch/parse 解耦)。
    pub fn ingest_articles(
        kb: &crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
        articles: &[GdeltArticle],
        query: &str,
    ) -> Result<GdeltIngestReport, String> {
        let mut report = GdeltIngestReport {
            query: query.to_string(),
            articles_fetched: articles.len(),
            ..Default::default()
        };
        for art in articles {
            if art.url.trim().is_empty() || art.title.trim().is_empty() {
                report.errors.push(format!("skip empty url/title: {:?}", art.url));
                continue;
            }
            // 去重：同 URL 复用 (insert_or_get 语义)
            let summary = format!(
                "{} | {} | {} | via {} ({})",
                art.title, art.seendate, art.language, art.domain, art.sourcecountry
            );
            // KB 入库：NodeType::Article，domain=gdelt，便于检索回查
            let existing = kb.find_node_by_url(&art.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb
                .insert_or_get_node(&art.title, crate::core::nt_core_kb_types::NodeType::Article, Some(&summary), Some(&art.url), Some("gdelt"))
                .map_err(|e| format!("KB ingest failed for {}: {}", art.url, e))?;
            if is_new {
                report.nodes_created += 1;
            } else {
                report.nodes_reused += 1;
            }
        }
        Ok(report)
    }

    /// 转 SearchResult (供 Ordered Backend Router 复用)。
    pub fn to_search_results(articles: &[GdeltArticle]) -> Vec<crate::neotrix::l2_world_impl::nt_world_search::SearchResult> {
        articles
            .iter()
            .map(|a| crate::neotrix::l2_world_impl::nt_world_search::SearchResult {
                title: a.title.clone(),
                url: a.url.clone(),
                snippet: format!("{} | {} {}", a.seendate, a.domain, a.language),
                evidence: None,
            })
            .collect()
    }
}

// ── Egress 登记 ─────────────────────────────────────────────────

/// GDELT Egress 主机 — 单一事实源在 `nt_shield_sandbox::INTEL_GDELT_HOST` (P2)。
pub const GDELT_HOST: &str = crate::neotrix::l1_body_impl::nt_shield_sandbox::INTEL_GDELT_HOST;
/// GDELT Egress allow 规则 (deny-wins 体系中的 allow 分支)。
pub fn gdelt_egress_rule() -> crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressRule {
    crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressRule::allow(GDELT_HOST, "443")
}
/// GDELT 专用 Egress Policy (deny_all 基线 + 单条 allow) — 委托 `intel_egress_policy`。
pub fn gdelt_egress_policy() -> crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressPolicy {
    crate::neotrix::l1_body_impl::nt_shield_sandbox::intel_egress_policy()
}

// ── 入库报告 ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GdeltIngestReport {
    pub query: String,
    pub articles_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend 适配 (R-P42 强化 Ordered Backend Router) ──────

/// GDELT 作为有序搜索后端 (P1 首个情报后端，免费无key)。
pub struct GdeltBackend {
    fetcher: GdeltFetcher,
}

impl Default for GdeltBackend {
    fn default() -> Self {
        Self { fetcher: GdeltFetcher::new() }
    }
}

impl GdeltBackend {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_base_url(base: &str) -> Self {
        Self { fetcher: GdeltFetcher::with_base_url(base) }
    }
}

impl crate::neotrix::l2_world_impl::nt_world_search::SearchBackend for GdeltBackend {
    fn name(&self) -> &str {
        "gdelt"
    }
    fn search(&self, query: &str, count: usize) -> Result<Vec<crate::neotrix::l2_world_impl::nt_world_search::SearchResult>, String> {
        let articles = self.fetcher.fetch(query, count)?;
        Ok(GdeltFetcher::to_search_results(&articles))
    }
}

// ── Fixture (测试共用) ─────────────────────────────────────────

/// 最小 ArtList fixture：2 条 artificial intelligence 文章。
pub const GDELT_FIXTURE_JSON: &str = r#"{
  "articles": [
    {
      "title": "AI breakthrough transforms healthcare",
      "url": "https://example.com/ai-health-2026",
      "seendate": "20260826T120000Z",
      "domain": "reuters.com",
      "language": "English",
      "sourcecountry": "United States",
      "socialimage": "https://example.com/img1.jpg"
    },
    {
      "title": "Machine learning reaches new milestone",
      "url": "https://example.com/ml-milestone-2026",
      "seendate": "20260826T080000Z",
      "domain": "bloomberg.com",
      "language": "English",
      "sourcecountry": "United Kingdom",
      "socialimage": "https://example.com/img2.jpg"
    }
  ]
}"#;

// ── 测试 ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let arts = GdeltFetcher::parse_articles(GDELT_FIXTURE_JSON).expect("parse fixture");
        assert_eq!(arts.len(), 2, "fixture should yield 2 articles");
        assert_eq!(arts[0].title, "AI breakthrough transforms healthcare");
        assert_eq!(arts[0].url, "https://example.com/ai-health-2026");
        assert_eq!(arts[0].domain, "reuters.com");
        assert_eq!(arts[1].title, "Machine learning reaches new milestone");
    }

    #[test]
    fn test_parse_empty_articles() {
        let arts = GdeltFetcher::parse_articles(r#"{"articles":[]}"#).expect("empty ok");
        assert!(arts.is_empty());
    }

    #[test]
    fn test_build_url_encodes_query() {
        let f = GdeltFetcher::new();
        let url = f.build_url("artificial intelligence", 10);
        assert!(url.contains("artificial+intelligence") || url.contains("artificial%20intelligence"), "url: {url}");
        assert!(url.contains("mode=ArtList"));
        assert!(url.contains("format=json"));
        assert!(url.contains("maxrecords=10"));
    }

    #[test]
    fn test_egress_policy_allows_gdelt_denies_other() {
        let policy = gdelt_egress_policy();
        assert!(policy.check(GDELT_HOST, 443), "gdelt host should be allowed on 443");
        assert!(!policy.check(GDELT_HOST, 80), "wrong port should be denied");
        assert!(!policy.check("evil.com", 443), "non-gdelt host denied");
        // deny-wins: 叠加 deny 规则应覆盖 allow
        let mut with_deny = gdelt_egress_policy();
        with_deny.rules.push(crate::neotrix::l1_body_impl::nt_shield_sandbox::EgressRule::deny(GDELT_HOST, "443"));
        assert!(!with_deny.check(GDELT_HOST, 443), "explicit deny wins over allow");
    }

    #[test]
    fn test_to_search_results() {
        let arts = GdeltFetcher::parse_articles(GDELT_FIXTURE_JSON).unwrap();
        let results = GdeltFetcher::to_search_results(&arts);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "AI breakthrough transforms healthcare");
        assert_eq!(results[0].url, "https://example.com/ai-health-2026");
        assert!(results[0].snippet.contains("reuters.com"));
    }

    #[test]
    fn test_ingest_fixture_to_kb_and_requery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test_gdelt_kb.db");
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
        let fetcher = GdeltFetcher::new();
        // 用 fixture 纯内存 ingest，无网络
        let report = fetcher.ingest_from_json(&kb, GDELT_FIXTURE_JSON, "artificial intelligence").expect("ingest");
        assert_eq!(report.articles_fetched, 2);
        assert_eq!(report.nodes_created, 2, "first ingest creates 2 nodes");
        assert_eq!(report.nodes_reused, 0);

        // 回查：URL 精确命中
        let node = kb.find_node_by_url("https://example.com/ai-health-2026").expect("find").expect("found");
        assert_eq!(node.title, "AI breakthrough transforms healthcare");
        assert_eq!(node.domain.as_deref(), Some("gdelt"));
        // FTS 搜索可回查 (经 nodes_fts)
        let hits = kb.search_permission_aware("healthcare", 10, crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
        assert!(!hits.is_empty(), "FTS should recall ingested article by keyword");

        // 幂等：二次 ingest 同 fixture → nodes_reused
        let report2 = fetcher.ingest_from_json(&kb, GDELT_FIXTURE_JSON, "artificial intelligence").expect("re-ingest");
        assert_eq!(report2.nodes_created, 0, "second ingest reuses nodes");
        assert_eq!(report2.nodes_reused, 2);
    }

    #[test]
    fn test_gdelt_backend_name() {
        use crate::neotrix::l2_world_impl::nt_world_search::SearchBackend;
        let b = GdeltBackend::new();
        assert_eq!(SearchBackend::name(&b), "gdelt");
    }

    #[test]
    fn test_ingest_skips_empty_url() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test_gdelt_skip.db");
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
        let fetcher = GdeltFetcher::new();
        let bad_json = r#"{"articles":[{"title":"has title but no url","url":"","seendate":"20260826T000000Z","domain":"x.com","language":"English"}]}"#;
        let report = fetcher.ingest_from_json(&kb, bad_json, "x").expect("ingest bad");
        assert_eq!(report.nodes_created, 0);
        assert_eq!(report.errors.len(), 1);
    }
}