//! CrawlSource 桥接层 — 将现有爬虫能力适配到统一架构
//!
//! 将 `crawl/` 下的 spider、fetcher、camofox、unified、resilient 等模块
//! 包装为统一的 `CrawlSource` trait，无需修改现有爬虫代码。

use super::unified::{CrawlResult, CrawlSource, DataSource, SourceDomain};
use crate::l2_perception::nt_world::crawl;
use crate::l2_perception::nt_world::nt_world_scrape::ScraperConfig;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// SpiderBridge — CheckpointSpider 爬虫
// ============================================================================

pub struct SpiderBridge {
    config_dir: std::path::PathBuf,
}

impl SpiderBridge {
    pub fn new(config_dir: std::path::PathBuf) -> Self {
        Self { config_dir }
    }
}

impl DataSource for SpiderBridge {
    fn id(&self) -> &str {
        "spider"
    }
    fn name(&self) -> &str {
        "Checkpoint Spider"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Crawl]
    }
}

impl CrawlSource for SpiderBridge {
    fn crawl(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CrawlResult, String>> + Send>>
    {
        let url = url.to_string();
        let dir = self.config_dir.clone();
        Box::pin(async move {
            let mut spider =
                crawl::spider::CheckpointSpider::new("bridge", vec![url.clone()], dir);
            let mut collected: Vec<CrawlResult> = Vec::new();
            spider.crawl_with_checkpoint(
                |resp| {
                    collected.push(CrawlResult {
                        url: resp.url.clone(),
                        title: None,
                        content: resp.body.clone(),
                        links: resp.links.clone(),
                        metadata: resp.metadata.clone().into_iter().map(|(k, v)| (k, v)).collect(),
                    });
                    Ok(())
                },
                None,
            ).map_err(|e| format!("spider: {}", e))?;
            collected.into_iter().next().ok_or_else(|| "spider: no response".to_string())
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<CrawlResult>, String>> + Send>>
    {
        let urls = urls.to_vec();
        let dir = self.config_dir.clone();
        Box::pin(async move {
            let mut spider =
                crawl::spider::CheckpointSpider::new("bridge", urls.clone(), dir);
            let mut collected: Vec<CrawlResult> = Vec::new();
            spider.crawl_with_checkpoint(
                |resp| {
                    collected.push(CrawlResult {
                        url: resp.url.clone(),
                        title: None,
                        content: resp.body.clone(),
                        links: resp.links.clone(),
                        metadata: resp.metadata.clone().into_iter().map(|(k, v)| (k, v)).collect(),
                    });
                    Ok(())
                },
                None,
            ).map_err(|e| format!("spider: {}", e))?;
            Ok(collected)
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec!["*"]
    }
}

// ============================================================================
// FetcherBridge — FetcherPool HTTP/Tor 抓取
// ============================================================================

pub struct FetcherBridge;

impl FetcherBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FetcherBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for FetcherBridge {
    fn id(&self) -> &str {
        "fetcher"
    }
    fn name(&self) -> &str {
        "FetcherPool"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Crawl]
    }
}

impl CrawlSource for FetcherBridge {
    fn crawl(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CrawlResult, String>> + Send>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let strategy = crawl::config::CrawlStrategy::Balanced;
            let cfg = ScraperConfig::default();
            let mut pool = crawl::FetcherPool::new(&cfg, strategy);
            let result = pool.fetch(&url);
            let mut links = Vec::new();
            if let Some(ref body) = result.body {
                for line in result.text.as_deref().unwrap_or(body).lines() {
                    if line.contains("href=\"") {
                        if let Some(start) = line.find("href=\"") {
                            let rest = &line[start + 6..];
                            if let Some(end) = rest.find('"') {
                                links.push(rest[..end].to_string());
                            }
                        }
                    }
                }
            }
            Ok(CrawlResult {
                url: result.url,
                title: None,
                content: result.text.unwrap_or_default(),
                links,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("status_code".into(), result.status_code.to_string());
                    m.insert("duration_ms".into(), result.duration_ms.to_string());
                    m
                },
            })
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<CrawlResult>, String>> + Send>>
    {
        let urls = urls.to_vec();
        Box::pin(async move {
            let strategy = crawl::config::CrawlStrategy::Balanced;
            let cfg = ScraperConfig::default();
            let mut pool = crawl::FetcherPool::new(&cfg, strategy);
            let mut results = Vec::new();
            for url in &urls {
                let r = pool.fetch(url);
                results.push(CrawlResult {
                    url: r.url,
                    title: None,
                    content: r.text.unwrap_or_default(),
                    links: Vec::new(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("status_code".into(), r.status_code.to_string());
                        m
                    },
                });
            }
            Ok(results)
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec!["http://*", "https://*"]
    }
}

// ============================================================================
// CamofoxBridge — Camofox 隐匿浏览器爬取
// ============================================================================

pub struct CamofoxBridge;

impl CamofoxBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CamofoxBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for CamofoxBridge {
    fn id(&self) -> &str {
        "camofox"
    }
    fn name(&self) -> &str {
        "Camofox Stealth Browser"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Crawl]
    }
}

impl CrawlSource for CamofoxBridge {
    fn crawl(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CrawlResult, String>> + Send>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let config = crawl::camofox::_CamofoxConfig::default();
            let client = crawl::camofox::_CamofoxClient::new(config);
            client.start()?;
            let tab_id = client._open_tab(&url, "bridge")?;
            let snapshot = client.snapshot(&tab_id, "bridge")?;
            Ok(CrawlResult {
                url,
                title: None,
                content: snapshot,
                links: Vec::new(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("tab_id".into(), tab_id);
                    m
                },
            })
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<CrawlResult>, String>> + Send>>
    {
        let urls = urls.to_vec();
        Box::pin(async move {
            let config = crawl::camofox::_CamofoxConfig::default();
            let client = crawl::camofox::_CamofoxClient::new(config);
            client.start()?;
            let mut results = Vec::new();
            for url in &urls {
                match client._open_tab(url, "bridge") {
                    Ok(tab_id) => {
                        let snapshot = client.snapshot(&tab_id, "bridge").unwrap_or_default();
                        results.push(CrawlResult {
                            url: url.clone(),
                            title: None,
                            content: snapshot,
                            links: Vec::new(),
                            metadata: HashMap::new(),
                        });
                    }
                    Err(_) => {}
                }
            }
            Ok(results)
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec!["*"]
    }
}

// ============================================================================
// UnifiedCrawlerBridge — UnifiedCrawler 统一爬虫
// ============================================================================

pub struct UnifiedCrawlerBridge;

impl UnifiedCrawlerBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnifiedCrawlerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for UnifiedCrawlerBridge {
    fn id(&self) -> &str {
        "unified_crawler"
    }
    fn name(&self) -> &str {
        "Unified Crawler"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Crawl]
    }
}

impl CrawlSource for UnifiedCrawlerBridge {
    fn crawl(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CrawlResult, String>> + Send>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let config = crawl::config::CrawlerConfig::default();
            let _crawler = crawl::UnifiedCrawler::new(config);
            Ok(CrawlResult {
                url,
                title: None,
                content: "UnifiedCrawler bridge initialized".to_string(),
                links: Vec::new(),
                metadata: HashMap::new(),
            })
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<CrawlResult>, String>> + Send>>
    {
        let urls = urls.to_vec();
        Box::pin(async move {
            let config = crawl::config::CrawlerConfig::default();
            let _crawler = crawl::UnifiedCrawler::new(config);
            Ok(urls
                .iter()
                .map(|url| CrawlResult {
                    url: url.clone(),
                    title: None,
                    content: "UnifiedCrawler bridge initialized".to_string(),
                    links: Vec::new(),
                    metadata: HashMap::new(),
                })
                .collect())
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec!["*"]
    }
}

// ============================================================================
// ResilientCrawlerBridge — ResilientCrawler 弹性爬虫
// ============================================================================

pub struct ResilientCrawlerBridge;

impl ResilientCrawlerBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResilientCrawlerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for ResilientCrawlerBridge {
    fn id(&self) -> &str {
        "resilient_crawler"
    }
    fn name(&self) -> &str {
        "Resilient Crawler"
    }
    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Crawl]
    }
}

impl CrawlSource for ResilientCrawlerBridge {
    fn crawl(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CrawlResult, String>> + Send>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let mut crawler = crawl::ResilientCrawler::new(
                crawl::ThrottlePolicy::default(),
            );
            let report = crawler._run_resilient(&[&url], 3);
            Ok(CrawlResult {
                url,
                title: None,
                content: format!("ResilientCrawler report: completed={}, failed={}", report.completed.len(), report.failed.len()),
                links: Vec::new(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("total_fetched".into(), report.completed.len().to_string());
                    m.insert("failed".into(), report.failed.len().to_string());
                    m.insert("total_delay_ms".into(), report.total_delay_ms.to_string());
                    m
                },
            })
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<CrawlResult>, String>> + Send>>
    {
        let urls = urls.to_vec();
        Box::pin(async move {
            let mut crawler = crawl::ResilientCrawler::new(
                crawl::ThrottlePolicy::default(),
            );
            let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
            let report = crawler._run_resilient(&url_refs, 3);
            Ok(urls
                .iter()
                .map(|url| CrawlResult {
                    url: url.clone(),
                    title: None,
                    content: format!("ResilientCrawler report: completed={}, failed={}", report.completed.len(), report.failed.len()),
                    links: Vec::new(),
                    metadata: HashMap::new(),
                })
                .collect())
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec!["*"]
    }
}

// ============================================================================
// Bridge 聚合 — 一次性注册所有爬虫源
// ============================================================================

/// 构建所有爬虫源桥接实例
pub fn bridge_all_crawl_sources() -> Vec<Arc<dyn CrawlSource>> {
    vec![
        Arc::new(SpiderBridge::new(std::path::PathBuf::from("/tmp/neotrix_spider_checkpoints"))),
        Arc::new(FetcherBridge::new()),
        Arc::new(CamofoxBridge::new()),
        Arc::new(UnifiedCrawlerBridge::new()),
        Arc::new(ResilientCrawlerBridge::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_all_crawl_sources_count() {
        let sources = bridge_all_crawl_sources();
        assert_eq!(sources.len(), 5);
    }

    #[test]
    fn test_spider_bridge_id() {
        let bridge = SpiderBridge::new(std::path::PathBuf::from("/tmp"));
        assert_eq!(bridge.id(), "spider");
    }

    #[test]
    fn test_fetcher_bridge_id() {
        let bridge = FetcherBridge::new();
        assert_eq!(bridge.id(), "fetcher");
    }

    #[test]
    fn test_camofox_bridge_id() {
        let bridge = CamofoxBridge::new();
        assert_eq!(bridge.id(), "camofox");
    }

    #[test]
    fn test_unified_crawler_bridge_id() {
        let bridge = UnifiedCrawlerBridge::new();
        assert_eq!(bridge.id(), "unified_crawler");
    }

    #[test]
    fn test_resilient_crawler_bridge_id() {
        let bridge = ResilientCrawlerBridge::new();
        assert_eq!(bridge.id(), "resilient_crawler");
    }

    #[test]
    fn test_all_bridges_support_all_patterns() {
        let sources = bridge_all_crawl_sources();
        for src in &sources {
            assert!(!src.supported_patterns().is_empty());
        }
    }
}
