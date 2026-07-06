//! NeoTrix 外部探索引擎 — 持续爬取多源信息 → KB 归档 → 全景更新
//!
//! 数据源:
//!   - GitHub: trending 仓库 / topic 发现
//!   - ArXiv: 最新论文
//!   - Wikipedia: 相关主题
//!   - 网页: 通用文章
//!   - 种子 URL: 用户配置的探索源
//!
//! 循环: 发现 → 抓取 → 蒸馏 → KB 存储 → 全景更新

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::neotrix::l2_world_impl::nt_memory_kb_bridge::{KnowledgeBase, NodeType};

/// 探索数据源类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExplorationSource {
    GitHubTrending,
    GitHubTopic,
    ArXiv,
    Wikipedia,
    WebSearch,
    SeedUrls,
    Custom(String),
}

impl ExplorationSource {
    pub fn label(&self) -> &str {
        match self {
            ExplorationSource::GitHubTrending => "GitHub Trending",
            ExplorationSource::GitHubTopic => "GitHub Topic",
            ExplorationSource::ArXiv => "ArXiv",
            ExplorationSource::Wikipedia => "Wikipedia",
            ExplorationSource::WebSearch => "Web Search",
            ExplorationSource::SeedUrls => "Seed URLs",
            ExplorationSource::Custom(t) => t.as_str(),
        }
    }
}

/// 探索条目 (单次发现的结果)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationEntry {
    pub url: String,
    pub title: String,
    pub source: ExplorationSource,
    pub summary: String,
    pub discovered_at: i64,
    pub ingested: bool,
    pub priority: u8,        // 0-10
}

/// 探索周期报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationCycleReport {
    pub discovered: usize,
    pub ingested: usize,
    pub skipped: usize,
    pub failed: usize,
    pub by_source: HashMap<String, usize>,
    pub total_in_kb: usize,
}

/// 探索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationConfig {
    pub github_topics: Vec<String>,
    pub arxiv_categories: Vec<String>,
    pub seed_urls: Vec<String>,
    pub web_search_queries: Vec<String>,
    pub max_per_cycle: usize,
    pub cooldown_hours: u64,       // 同一 URL 冷却时间
}

impl Default for ExplorationConfig {
    fn default() -> Self {
        Self {
            github_topics: vec![
                "rust".into(), "machine-learning".into(), "artificial-intelligence".into(),
                "llm".into(), "agent".into(), "compiler".into(),
            ],
            arxiv_categories: vec!["cs.AI".into(), "cs.LG".into(), "cs.CL".into()],
            seed_urls: vec![],
            web_search_queries: vec![
                "state of AI 2026".into(), "new programming language".into(),
                "Rust ecosystem updates".into(),
            ],
            max_per_cycle: 10,
            cooldown_hours: 24,
        }
    }
}

// ============================================================
// 探索引擎
// ============================================================

pub struct ExplorationEngine {
    pub config: ExplorationConfig,
    pub kb: Option<KnowledgeBase>,
    discovered: Vec<ExplorationEntry>,
    ingested_urls: HashMap<String, i64>,  // URL → 上次摄入时间
    total_cycles: usize,
}

impl ExplorationEngine {
    pub fn new(config: ExplorationConfig) -> Self {
        Self {
            config,
            kb: None,
            discovered: Vec::new(),
            ingested_urls: HashMap::new(),
            total_cycles: 0,
        }
    }

    pub fn attach_kb(&mut self, kb: KnowledgeBase) {
        self.kb = Some(kb);
    }

    /// 执行一次探索循环: 发现 → 去重 → 摄入
    pub fn run_cycle(&mut self) -> ExplorationCycleReport {
        self.total_cycles += 1;
        let mut report = ExplorationCycleReport {
            discovered: 0, ingested: 0, skipped: 0, failed: 0,
            by_source: HashMap::new(), total_in_kb: self.ingested_urls.len(),
        };

        // 1. 发现新 URL
        let new_entries = self.discover();
        self.discovered.extend(new_entries);
        report.discovered = self.discovered.len();

        // 2. 去重 + 冷却检查
        let now = Utc::now().timestamp();
        let mut to_ingest: Vec<ExplorationEntry> = Vec::new();
        self.discovered.retain(|e| {
            if e.ingested { return false; }
            let cooldown = (self.config.cooldown_hours * 3600) as i64;
            let last = self.ingested_urls.get(&e.url).copied().unwrap_or(0);
            if now - last < cooldown {
                report.skipped += 1;
                return false;
            }
            if to_ingest.len() >= self.config.max_per_cycle {
                report.skipped += 1;
                return false;
            }
            true
        });
        to_ingest = self.discovered.drain(..self.config.max_per_cycle.min(self.discovered.len())).collect();

        // 3. 摄入到 KB
        for entry in &to_ingest {
            match self.ingest(entry) {
                Ok(()) => {
                    report.ingested += 1;
                    *report.by_source.entry(format!("{:?}", entry.source)).or_insert(0) += 1;
                    self.ingested_urls.insert(entry.url.clone(), now);
                }
                Err(e) => {
                    log::warn!("[exploration] 摄入失败 {}: {}", entry.url, e);
                    report.failed += 1;
                }
            }
        }

        report.total_in_kb = self.ingested_urls.len();
        report
    }

    /// 发现新资源
    fn discover(&self) -> Vec<ExplorationEntry> {
        let mut entries = Vec::new();
        let now = Utc::now().timestamp();

        // GitHub Trending
        if let Ok(trending) = self.discover_github_trending() {
            entries.extend(trending);
        }

        // ArXiv
        for cat in &self.config.arxiv_categories {
            if let Ok(papers) = self.discover_arxiv(cat) {
                entries.extend(papers);
            }
        }

        // 种子 URL
        for url in &self.config.seed_urls {
            entries.push(ExplorationEntry {
                url: url.clone(),
                title: url.clone(),
                source: ExplorationSource::SeedUrls,
                summary: String::new(),
                discovered_at: now,
                ingested: false,
                priority: 5,
            });
        }

        entries
    }

    /// 发现 GitHub Trending 仓库
    fn discover_github_trending(&self) -> Result<Vec<ExplorationEntry>, String> {
        let mut entries = Vec::new();
        let client = reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build().map_err(|e| format!("HTTP: {}", e))?;

        // GitHub API: 按 stars 搜索本周热门
        for topic in &self.config.github_topics {
            let query = format!("topic:{} stars:>100 pushed:>2026-01-01", topic);
            let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=5",
                urlencoding(&query));

            match client.get(&url).send() {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                            for item in items {
                                let repo_url = item.get("clone_url")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let name = item.get("full_name")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let desc = item.get("description")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string();
                                if !repo_url.is_empty() {
                                    entries.push(ExplorationEntry {
                                        url: repo_url,
                                        title: name,
                                        source: ExplorationSource::GitHubTrending,
                                        summary: desc,
                                        discovered_at: Utc::now().timestamp(),
                                        ingested: false,
                                        priority: 8,
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(resp) => log::warn!("[exploration] GitHub search API {}: {}", resp.status(), url),
                Err(e) => log::warn!("[exploration] GitHub search 请求失败: {}", e),
            }
        }

        Ok(entries)
    }

    /// 发现 ArXiv 最新论文
    fn discover_arxiv(&self, category: &str) -> Result<Vec<ExplorationEntry>, String> {
        let mut entries = Vec::new();
        let url = format!("http://export.arxiv.org/api/query?search_query=cat:{}&sortBy=submittedDate&sortOrder=descending&max_results=5",
            category);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build().map_err(|e| format!("HTTP: {}", e))?;

        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                let text = resp.text().unwrap_or_default();
                // 简单解析 XML
                for entry in text.split("<entry>").skip(1) {
                    let id = extract_xml(entry, "id").unwrap_or_default();
                    let title = extract_xml(entry, "title").unwrap_or_default();
                    let summary = extract_xml(entry, "summary").unwrap_or_default();
                    if !id.is_empty() {
                        entries.push(ExplorationEntry {
                            url: id,
                            title: title.trim().to_string(),
                            source: ExplorationSource::ArXiv,
                            summary: summary.chars().take(200).collect(),
                            discovered_at: Utc::now().timestamp(),
                            ingested: false,
                            priority: 7,
                        });
                    }
                }
            }
            Ok(resp) => log::warn!("[exploration] ArXiv API {}: {}", resp.status(), url),
            Err(e) => log::warn!("[exploration] ArXiv 请求失败: {}", e),
        }

        Ok(entries)
    }

    /// 摄入单个条目到 KB
    fn ingest(&self, entry: &ExplorationEntry) -> Result<(), String> {
        match &self.kb {
            Some(kb) => {
                let domain = match entry.source {
                    ExplorationSource::ArXiv => "arxiv.org",
                    ExplorationSource::GitHubTrending | ExplorationSource::GitHubTopic => "github.com",
                    _ => "web",
                };
                let summary = if entry.summary.is_empty() {
                    format!("Auto-discovered via {}", entry.source.label())
                } else {
                    format!("Auto-discovered via {}: {}", entry.source.label(), entry.summary)
                };
                let node_id = kb.insert_or_get_node(
                    &entry.title,
                    NodeType::Concept,
                    Some(&summary),
                    Some(&entry.url),
                    Some(domain),
                ).map_err(|e| format!("KB insert_or_get_node: {e}"))?;
                log::info!("[exploration] {} → KB node {}", entry.title, node_id);
            }
            None => {
                log::info!("[exploration] 发现但未摄入 (KB未连接): {} ({})", entry.title, entry.url);
            }
        }
        Ok(())
    }

    /// 已发现但未摄入的条目数
    pub fn pending_count(&self) -> usize {
        self.discovered.iter().filter(|e| !e.ingested).count()
    }

    pub fn stats(&self) -> ExplorationStats {
        ExplorationStats {
            total_cycles: self.total_cycles,
            discovered: self.discovered.len(),
            ingested_urls: self.ingested_urls.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationStats {
    pub total_cycles: usize,
    pub discovered: usize,
    pub ingested_urls: usize,
}

// ============================================================
// 工具函数
// ============================================================

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        ' ' => "%20".into(),
        ':' => "%3A".into(),
        '>' => "%3E".into(),
        '<' => "%3C".into(),
        _ => c.to_string(),
    }).collect()
}

fn extract_xml(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    text.find(&open).and_then(|start| {
        let content_start = start + open.len();
        text[content_start..].find(&close).map(|end| {
            text[content_start..content_start + end].to_string()
        })
    })
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exploration_engine_new() {
        let engine = ExplorationEngine::new(ExplorationConfig::default());
        assert_eq!(engine.stats().total_cycles, 0);
    }

    #[test]
    fn test_url_encoding() {
        let encoded = urlencoding("topic:rust stars:>100");
        assert!(encoded.contains("%3A"));
        assert!(encoded.contains("%3E"));
        assert!(encoded.contains("%20"));
    }

    #[test]
    fn test_xml_extract() {
        let xml = "<entry><id>1234</id><title>Test Paper</title></entry>";
        assert_eq!(extract_xml(xml, "id"), Some("1234".into()));
        assert_eq!(extract_xml(xml, "title"), Some("Test Paper".into()));
    }

    #[test]
    fn test_discover_returns_entries() {
        let engine = ExplorationEngine::new(ExplorationConfig {
            seed_urls: vec!["https://github.com/rust-lang/rust".into()],
            ..Default::default()
        });
        let entries = engine.discover();
        assert!(!entries.is_empty(), "should have at least seed URLs");
        assert!(entries.iter().any(|e| e.source == ExplorationSource::SeedUrls));
    }

    #[test]
    fn test_run_cycle_counts() {
        let mut engine = ExplorationEngine::new(ExplorationConfig {
            seed_urls: vec!["https://github.com/neotrix".into()],
            max_per_cycle: 5,
            ..Default::default()
        });
        let report = engine.run_cycle();
        assert!(report.ingested > 0 || report.skipped > 0);
    }
}
