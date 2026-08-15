//! nt_world_absorber — 统一外部知识源吸收编排模块
//!
//! 将所有外部数据源（GitHub / Web / Papers / 知识库）的摄取
//! 统一到一个管道中，路由到 KB 存储，支持调度、去重、增量更新。

pub mod self_curriculum;
pub mod api_registry;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::nt_world_github_absorber::{GitHubAbsorbReport, GitHubAbsorber};
use crate::neotrix::l2_world_impl::nt_memory_kb_bridge::{
    CrawlCycleReport, KnowledgeBase, KnowledgeNode, NodeType,
};
use rusqlite::Connection;

// ── Types ──

/// Source types the absorber can handle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AbsorbSource {
    GitHub { owner: String, repo: String },
    GitHubUrl(String),
    ArXiv(String),
    Wikipedia(String),
    WebPage(String),
    DiscoveryTopic(String),
}

impl AbsorbSource {
    pub fn from_url(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.contains("github.com") {
            return Some(AbsorbSource::GitHubUrl(url.to_string()));
        }
        if url.contains("arxiv.org") {
            let id = url.trim_end_matches('/').split('/').next_back().unwrap_or(url);
            return Some(AbsorbSource::ArXiv(id.to_string()));
        }
        if url.contains("wikipedia.org") {
            let topic = url.split('/').next_back().unwrap_or(url).replace('_', " ");
            return Some(AbsorbSource::Wikipedia(topic));
        }
        Some(AbsorbSource::WebPage(url.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbCycleReport {
    pub sources_attempted: usize,
    pub sources_succeeded: usize,
    pub sources_failed: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub github_repos: Vec<GitHubAbsorbReport>,
    pub web_pages: Vec<WebPageReport>,
    pub errors: Vec<(String, String)>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPageReport {
    pub url: String,
    pub title: String,
    pub nodes_created: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberConfig {
    pub max_github_stars: i64,
    pub max_source_files: usize,
    pub enable_readme: bool,
    pub enable_deps: bool,
    pub enable_insights: bool,
    pub auto_refresh_days: i64,
    pub max_concurrent: usize,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            max_github_stars: 0,
            max_source_files: 30,
            enable_readme: true,
            enable_deps: true,
            enable_insights: true,
            auto_refresh_days: 7,
            max_concurrent: 3,
        }
    }
}

// ── UnifiedAbsorber ──

pub struct UnifiedAbsorber {
    kb: KnowledgeBase,
    github: GitHubAbsorber,
    /// G30 API/服务注册表 (public-apis/free-for-dev 吸收) — 发现种子源。
    api_registry: api_registry::ApiRegistry,
}

impl UnifiedAbsorber {
    pub fn new(kb: KnowledgeBase, _config: AbsorberConfig) -> Result<Self, String> {
        Ok(Self {
            github: GitHubAbsorber::new(kb.clone_connection()?),
            kb,
            api_registry: api_registry::ApiRegistry::new(),
        })
    }

    /// G30 注册一个 API 条目 (发现种子)。
    pub fn register_api(&mut self, entry: api_registry::ApiEntry) {
        self.api_registry.register(entry);
    }

    /// G30 把注册的 API 条目 URL 作为发现种子注入 crawl queue (供后续摄取)。
    /// 返回成功注入的种子数。
    pub fn seed_api_discovery(&self, conn: &Connection) -> Result<usize, String> {
        let seeds = self.api_registry.seed_urls();
        let mut injected = 0usize;
        for url in seeds {
            let domain = crate::neotrix::l2_world_impl::nt_world_crawl::frontier::extract_domain(&url);
            if let Err(e) = nt_memory_store::upsert_crawl_queue(conn, &url, 0, &domain, 50, now()) {
                log::warn!("[absorber] seed api discovery {}: {}", url, e);
                continue;
            }
            injected += 1;
        }
        Ok(injected)
    }

    pub fn api_registry_stats(&self) -> (usize, f64, Vec<(String, usize)>) {
        self.api_registry.stats()
    }

    /// Absorb a single source, automatically detecting type
    pub fn absorb(&self, source: &AbsorbSource) -> Result<AbsorbCycleReport, String> {
        let mut report = AbsorbCycleReport {
            sources_attempted: 1,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        match source {
            AbsorbSource::GitHub { owner, repo } => {
                match self.github.absorb(owner, repo) {
                    Ok(gr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += gr.nodes_created;
                        report.edges_created += gr.edges_created;
                        report.github_repos.push(gr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("{}/{}", owner, repo), e));
                    }
                }
            }
            AbsorbSource::GitHubUrl(url) => {
                match self.github.absorb_url(url) {
                    Ok(gr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += gr.nodes_created;
                        report.edges_created += gr.edges_created;
                        report.github_repos.push(gr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((url.clone(), e));
                    }
                }
            }
            AbsorbSource::ArXiv(id) => {
                match self.kb.ingest_arxiv(id) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("arxiv:{}", id), e));
                    }
                }
            }
            AbsorbSource::Wikipedia(topic) => {
                match self.kb.ingest_wikipedia(topic) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("wiki:{}", topic), e));
                    }
                }
            }
            AbsorbSource::WebPage(url) => {
                match self.absorb_webpage(url) {
                    Ok(wr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += wr.nodes_created;
                        report.web_pages.push(wr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((url.clone(), e));
                    }
                }
            }
            AbsorbSource::DiscoveryTopic(topic) => {
                match self.kb.ingest_wikipedia(topic) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                        // Also crawl references
                        let conn = match self.kb.conn.lock() {
                            Ok(c) => c,
                            Err(e) => { report.errors.push(("lock".into(), format!("{}", e))); return Ok(report); }
                        };
                        let _ = nt_memory_kb_crawl::discover_from_seed(&conn, topic);
                        report.nodes_created += 1;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("discover:{}", topic), e));
                    }
                }
            }
        }

        self.persist_cycle_report(&report)?;
        Ok(report)
    }

    /// Absorb multiple sources in batch
    pub fn absorb_batch(&self, sources: &[AbsorbSource]) -> Result<AbsorbCycleReport, String> {
        let mut aggregated = AbsorbCycleReport {
            sources_attempted: sources.len(),
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        for source in sources {
            match self.absorb(source) {
                Ok(report) => {
                    aggregated.sources_succeeded += report.sources_succeeded;
                    aggregated.sources_failed += report.sources_failed;
                    aggregated.nodes_created += report.nodes_created;
                    aggregated.edges_created += report.edges_created;
                    aggregated.github_repos.extend(report.github_repos);
                    aggregated.web_pages.extend(report.web_pages);
                    aggregated.errors.extend(report.errors);
                }
                Err(e) => {
                    aggregated.sources_failed += 1;
                    aggregated.errors.push((format!("{:?}", source), e));
                }
            }
        }

        self.persist_cycle_report(&aggregated)?;
        Ok(aggregated)
    }

    /// Run a full exploration cycle: discover new sources, absorb them, check for updates
    pub fn run_cycle(&self, topics: &[&str]) -> Result<AbsorbCycleReport, String> {
        let mut report = AbsorbCycleReport {
            sources_attempted: 0,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        // Phase 1: Discover from seed topics
        for topic in topics {
            match self.absorb(&AbsorbSource::DiscoveryTopic(topic.to_string())) {
                Ok(r) => {
                    report.sources_succeeded += r.sources_succeeded;
                    report.sources_failed += r.sources_failed;
                    report.nodes_created += r.nodes_created;
                    report.edges_created += r.edges_created;
                    report.errors.extend(r.errors);
                }
                Err(e) => report.errors.push((topic.to_string(), e)),
            }
            report.sources_attempted += 1;
        }

        // Phase 2: GitHub topic discovery
        let discovery_cfg = nt_memory_kb_discovery::DiscoveryPipelineConfig::default();
        match self.kb.run_github_topics_discovery(&discovery_cfg) {
            Ok(stats) => {
                report.nodes_created += stats.repos_ingested;
                report.sources_succeeded += 1;
            }
            Err(e) => report.errors.push(("github_topics_discovery".into(), e)),
        }

        // Phase 3: Run KB crawl queue
        let conn = match self.kb.conn.lock() {
            Ok(c) => c,
            Err(e) => { report.errors.push(("lock".into(), format!("{}", e))); return Ok(report); }
        };
        if let Ok(crawl_report) = nt_memory_kb_crawl::run_crawl_cycle(&conn, 10) {
            report.nodes_created += crawl_report.nodes_created;
            report.edges_created += crawl_report.edges_created;
            report.sources_attempted += crawl_report.attempted;
            report.sources_succeeded += crawl_report.completed;
            report.sources_failed += crawl_report.failed;
            for (url, err) in &crawl_report.errors {
                let url: String = url.clone();
                let err: String = err.clone();
                report.errors.push((url, err));
            }
        }
        drop(conn);

        // Phase 4: Refresh stale GitHub repos
        let repos = self.kb.find_repositories("github.com", None).unwrap_or_default();
        for node in repos {
            let stale = node.metadata.as_ref()
                .map(|m| {
                    let pushed = m.get("pushed_at").and_then(|v| v.as_i64()).unwrap_or(0);
                    let absorbed = m.get("last_absorbed").and_then(|v| v.as_i64()).unwrap_or(0);
                    pushed > absorbed
                })
                .unwrap_or(false);
            if stale {
                let parts: Vec<&str> = node.title.split('/').collect();
                if parts.len() == 2 {
                    if let Ok(gr) = self.github.refresh(parts[0], parts[1]) {
                        if gr.is_update {
                            report.github_repos.push(gr);
                        }
                    }
                }
            }
        }

        self.persist_cycle_report(&report)?;
        Ok(report)
    }

    /// Status of the absorption pipeline
    pub fn status(&self) -> Result<AbsorberStatus, String> {
        let repos = self.kb.find_repositories("github.com", None).unwrap_or_default();
        let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let total_nodes = nt_memory_store::count_nodes(&conn).map_err(|e| format!("count: {}", e))?;
        let paper_count = nt_memory_store::count_nodes_by_type(&conn, "Paper").map_err(|e| format!("count_papers: {}", e))?;
        let article_count = nt_memory_store::count_nodes_by_type(&conn, "Article").map_err(|e| format!("count_articles: {}", e))?;
        let concept_count = nt_memory_store::count_nodes_by_type(&conn, "Concept").map_err(|e| format!("count_concepts: {}", e))?;
        let code_count = nt_memory_store::count_nodes_by_type(&conn, "CodeSnippet").map_err(|e| format!("count_code: {}", e))?;
        let insight_count = nt_memory_store::count_nodes_by_type(&conn, "Insight").map_err(|e| format!("count_insights: {}", e))?;
        let stale_repos = repos.iter().filter(|n| {
            n.metadata.as_ref()
                .map(|m| {
                    let pushed = m.get("pushed_at").and_then(|v| v.as_i64()).unwrap_or(0);
                    let absorbed = m.get("last_absorbed").and_then(|v| v.as_i64()).unwrap_or(0);
                    pushed > absorbed
                })
                .unwrap_or(false)
        }).count();
        drop(conn);

        Ok(AbsorberStatus {
            total_nodes,
            repositories: repos.len(),
            papers: paper_count,
            articles: article_count,
            concepts: concept_count,
            code_snippets: code_count,
            insights: insight_count,
            stale_repos,
            last_cycle: self.kb.kv_get("absorber", "last_cycle").unwrap_or(None),
        })
    }

    // ── Private ──

    fn absorb_webpage(&self, url: &str) -> Result<WebPageReport, String> {
        let (html, host) = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_http::fetch_safe_http(url)?;
        let (title, text) = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_html_content(&html);
        if text.is_empty() {
            return Err("Empty content".into());
        }
        let summary = text.chars().take(2000).collect::<String>();
        let domain = if host.is_empty() { "unknown" } else { &host };
        let _node_id = self.kb.insert_or_get_node(
            &if title.is_empty() { url.to_string() } else { title.clone() },
            NodeType::Article,
            Some(&summary),
            Some(url),
            Some(domain),
        )?;

        // Extract and enqueue links
        let links = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_links(&html, url);
        let ts = now();
        for link in links.iter().take(20) {
            let link_domain = link.split('/').nth(2).unwrap_or("").trim_start_matches("www.").to_string();
            if link_domain == domain || link_domain.is_empty() { continue; }
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            let _ = nt_memory_store::upsert_crawl_queue(&conn, link, 1, &link_domain, 0, ts);
            drop(conn);
        }

        Ok(WebPageReport {
            url: url.to_string(),
            title,
            nodes_created: 1,
            success: true,
            error: None,
        })
    }

    fn persist_cycle_report(&self, report: &AbsorbCycleReport) -> Result<(), String> {
        let json = serde_json::to_string(report).map_err(|e| format!("serde: {}", e))?;
        let _ = self.kb.kv_set("absorber", "last_cycle", &json);
        Ok(())
    }

    /// G19/G20 视频生产全链产物入 KB: 将 production manifest 落盘为
    /// Resource 节点 (nvda 领域), 供下游检索与审计。
    pub fn absorb_video_production(
        &self,
        topic: &str,
        manifest: &[(crate::neotrix::l2_world_impl::nt_world_video_pipeline::ProductionStage, String)],
        asset_stats: (usize, usize, usize),
    ) -> Result<String, String> {
        let mut summary = format!("video production: {}", topic);
        for (stage, artifact) in manifest {
            summary.push_str(&format!("\n- {}: {}", stage.label(), artifact));
        }
        summary.push_str(&format!(
            "\nasset enrichment: total={} dup={} kept={}",
            asset_stats.0, asset_stats.1, asset_stats.2
        ));
        let _node_id = self.kb.insert_or_get_node(
            &format!("video-production-{}", topic),
            NodeType::Resource,
            Some(&summary),
            None,
            Some("nvda"),
        )?;
        let _ = self.kb.kv_set("absorber", "last_video_production", &summary);
        Ok(summary)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberStatus {
    pub total_nodes: usize,
    pub repositories: usize,
    pub papers: usize,
    pub articles: usize,
    pub concepts: usize,
    pub code_snippets: usize,
    pub insights: usize,
    pub stale_repos: usize,
    pub last_cycle: Option<String>,
}

// ── Module-level forwarders for KB sub-module access ──

mod nt_memory_kb_crawl {
    use rusqlite::Connection;
    pub fn run_crawl_cycle(conn: &Connection, max: usize) -> Result<super::CrawlCycleReport, String> {
        let r = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::run_crawl_cycle(conn, max)?;
        Ok(super::CrawlCycleReport {
            attempted: r.attempted,
            completed: r.completed,
            failed: r.failed,
            nodes_created: r.nodes_created,
            edges_created: r.edges_created,
            urls_processed: r.urls_processed,
            errors: r.errors,
            by_domain: r.by_domain,
        })
    }
    pub fn discover_from_seed(conn: &Connection, topic: &str) -> Result<usize, String> {
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::discover_from_seed(conn, topic)
    }
}

mod nt_memory_kb_discovery {
    pub use crate::neotrix::l2_world_impl::nt_memory_kb_bridge::DiscoveryPipelineConfig;
}

mod nt_memory_store {
    use rusqlite::Connection;
    #[allow(dead_code)]
    fn bridge_node(n: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::KnowledgeNode) -> super::KnowledgeNode {
        use crate::neotrix::l2_world_impl::nt_memory_kb_bridge as bridge;
        let nt = bridge::from_real_node_type(&n.node_type);
        super::KnowledgeNode {
            id: n.id,
            title: n.title,
            node_type: nt,
            content: n.content,
            summary: n.summary,
            url: n.url,
            domain: n.domain,
            language: n.language,
            confidence: n.confidence,
            importance: n.importance,
            access_count: n.access_count,
            metadata: n.metadata,
            created_at: n.created_at,
            updated_at: n.updated_at,
        }
    }
    #[allow(dead_code)]
    pub fn get_all_nodes(conn: &Connection) -> rusqlite::Result<Vec<super::KnowledgeNode>> {
        let nodes = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::get_all_nodes(conn)?;
        Ok(nodes.into_iter().map(bridge_node).collect())
    }
    pub fn upsert_crawl_queue(conn: &Connection, url: &str, depth: i64, domain: &str, priority: i64, ts: i64) -> rusqlite::Result<()> {
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::upsert_crawl_queue(conn, url, depth, domain, priority, ts)
    }
    pub fn count_nodes(conn: &Connection) -> rusqlite::Result<usize> {
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_nodes(conn)
    }
    pub fn count_nodes_by_type(conn: &Connection, node_type: &str) -> rusqlite::Result<usize> {
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_nodes_by_type(conn, node_type)
    }
}

// ── HTTP (单一配置源在 nt_http,此处无独立 client) ──

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_from_url() {
        assert_eq!(
            AbsorbSource::from_url("https://github.com/rust-lang/rust").unwrap(),
            AbsorbSource::GitHubUrl("https://github.com/rust-lang/rust".into())
        );
        assert_eq!(
            AbsorbSource::from_url("https://arxiv.org/abs/2301.12345").unwrap(),
            AbsorbSource::ArXiv("2301.12345".into())
        );
        assert_eq!(
            AbsorbSource::from_url("https://en.wikipedia.org/wiki/Artificial_intelligence").unwrap(),
            AbsorbSource::Wikipedia("Artificial intelligence".into())
        );
        assert!(
            AbsorbSource::from_url("https://example.com/page").is_some(),
            "Plain webpage should be caught as WebPage"
        );
    }

    #[test]
    fn test_extract_html() {
        let (title, text) = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_html_content(
            "<html><title>Test</title><body><p>Hello world</p></body></html>");
        assert_eq!(title, "Test");
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_extract_html_script_and_style_stripping() {
        let html = r#"<html><title>Page</title><body>
<script>alert("xss")</script>
<style>.cls{color:red}</style>
<p>Visible content</p>
</body></html>"#;
        let (title, text) = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_html_content(html);
        assert_eq!(title, "Page");
        assert!(text.contains("Visible content"), "Visible text should survive");
        assert!(!text.contains("alert"), "Script content should be stripped");
        assert!(!text.contains(".cls"), "Style content should be stripped");
    }

    #[test]
    fn test_extract_links() {
        // 使用公网 IP 字面量 (8.8.8.8) 而非域名 — SSRF 守卫对域名做实时 DNS 解析,
        // 离线测试环境 DNS 失败会误过滤全部链接 (预存失败修复)。
        let html = r#"<a href="http://8.8.8.8/page1">Link 1</a><a href="http://8.8.8.8/page2">Link 2</a>"#;
        let links = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.contains("page1")));
        assert!(links.iter().any(|l| l.contains("page2")));
    }

    #[test]
    fn test_extract_links_deduplication() {
        let html = r#"<a href="http://8.8.8.8/page">Link</a><a href="http://8.8.8.8/page">Dup</a>"#;
        let links = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
        assert_eq!(links.len(), 1, "Duplicate links should be deduped");
    }

    #[test]
    fn test_absorb_source_from_url_edge_cases() {
        // GitHub URL with .git suffix
        let result = AbsorbSource::from_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(result, AbsorbSource::GitHubUrl("https://github.com/user/repo.git".into()));

        // ArXiv abstract URL
        let result = AbsorbSource::from_url("https://arxiv.org/abs/2301.12345v2").unwrap();
        assert_eq!(result, AbsorbSource::ArXiv("2301.12345v2".into()));
    }

    #[test]
    fn test_absorb_cycle_report_default_values() {
        let report = AbsorbCycleReport {
            sources_attempted: 0,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };
        assert_eq!(report.sources_attempted, 0);
        assert_eq!(report.sources_succeeded, 0);
        assert_eq!(report.nodes_created, 0);
    }

    #[test]
    fn test_absorber_status_unknown_domain() {
        let status = AbsorberStatus {
            total_nodes: 10,
            repositories: 3,
            papers: 1,
            articles: 2,
            concepts: 4,
            code_snippets: 0,
            insights: 0,
            stale_repos: 0,
            last_cycle: None,
        };
        assert_eq!(status.total_nodes, 10);
        assert_eq!(status.repositories, 3);
        assert_eq!(status.papers, 1);
        assert!(status.last_cycle.is_none());
    }

    #[test]
    fn test_api_registry_register_and_stats_wired() {
        // G30 discovery_registry_meta: 注册表接线到 UnifiedAbsorber
        let entry = api_registry::ApiEntry {
            name: "OpenAI".into(),
            description: "AI".into(),
            auth: "apiKey".into(),
            https: true,
            category: "Machine Learning".into(),
            cors: true,
            url: "https://api.openai.com".into(),
        };
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap();
        let mut absorber = UnifiedAbsorber::new(kb, AbsorberConfig {
            max_github_stars: 100,
            max_source_files: 10,
            enable_readme: true,
            enable_deps: false,
            enable_insights: false,
            auto_refresh_days: 7,
            max_concurrent: 4,
        }).unwrap();
        absorber.register_api(entry);
        let (n, https, _auths) = absorber.api_registry_stats();
        assert_eq!(n, 1);
        assert_eq!(https, 1.0);
    }

    #[test]
    fn test_absorb_video_production_writes_kb_node() {
        // G19/G20 视频生产全链产物入 KB
        use crate::neotrix::l2_world_impl::nt_world_video_pipeline::ProductionStage;
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap();
        let absorber = UnifiedAbsorber::new(kb, AbsorberConfig {
            max_github_stars: 100,
            max_source_files: 10,
            enable_readme: true,
            enable_deps: false,
            enable_insights: false,
            auto_refresh_days: 7,
            max_concurrent: 4,
        }).unwrap();
        let manifest = vec![
            (ProductionStage::Script, "script-x".into()),
            (ProductionStage::Compose, "out/x-final.mp4".into()),
            (ProductionStage::Publish, "published/x-final.mp4".into()),
        ];
        let summary = absorber.absorb_video_production("X", &manifest, (3, 1, 2)).unwrap();
        assert!(summary.contains("video production: X"));
        assert!(summary.contains("asset enrichment: total=3 dup=1 kept=2"));
    }
}
