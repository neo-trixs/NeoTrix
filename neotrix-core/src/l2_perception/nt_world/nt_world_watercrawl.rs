//! WaterCrawl — Web 爬虫内容提取 (C2)
//!
//! 吸收 github.com/watercrawl/WaterCrawl: 通用 Web 爬虫 + 内容提取能力。
//! C2 接线: `fetch` 升级为真实 reqwest 同步 HTTP 客户端调用；
//! `parse` 用 regex 从 HTML 提取 `<title>` 与 `<a href>` 链接 (无 scraper 依赖)。
//! API key 优先读 `WATERCRAWL_API_KEY` 环境变量，回退到 `NeoTrixConfig.api_key`，
//! 存在时以 `Authorization: Bearer` 注入 (兼容 WaterCrawl 托管 API)。

use crate::config::NeoTrixConfig;
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use regex::Regex;
use std::sync::OnceLock;

/// Web 页面原始抓取产物
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawledPage {
    pub url: String,
    pub status: u16,
    pub raw_html: String,
}

/// 内容提取结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedContent {
    pub title: String,
    pub text: String,
    pub links: Vec<String>,
}

/// Web 爬虫内容提取契约 (WaterCrawl 抽象)
pub trait WebCrawlerExtraction {
    /// 抓取给定 URL 的原始页面 (真实 HTTP 调用)
    fn fetch(&self, url: &str) -> CrawledPage;
    /// 从原始 HTML 中提取结构化正文
    fn parse(&self, page: &CrawledPage) -> ExtractedContent;
}

/// 编译期样例 HTML，供离线 parse 测试 / SelfTest 使用
const SAMPLE_HTML: &str = r#"<!doctype html><html><head><title>Sample Page</title></head>
<body><a href="https://example.com/a">A</a><a href="/b">B</a></body></html>"#;

fn title_re() -> &'static OnceLock<Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE
}

fn link_re() -> &'static OnceLock<Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE
}

/// 真实 HTTP 客户端实现 — 单元结构体，客户端在调用处惰性构建
#[derive(Default, Clone, Copy)]
pub struct WaterCrawlExtractor;

impl WaterCrawlerExtraction for WaterCrawlExtractor {
    fn fetch(&self, url: &str) -> CrawledPage {
        let client = reqwest::blocking::Client::new();
        let mut builder = client.get(url);
        if let Some(key) = resolve_api_key() {
            builder = builder.bearer_auth(key);
        }
        match builder.send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let raw_html = resp.text().unwrap_or_default();
                CrawledPage {
                    url: url.to_string(),
                    status,
                    raw_html,
                }
            }
            Err(_) => CrawledPage {
                url: url.to_string(),
                status: 0,
                raw_html: String::new(),
            },
        }
    }

    fn parse(&self, page: &CrawledPage) -> ExtractedContent {
        let title_re = title_re()
            .get_or_init(|| Regex::new(r"(?i)<title[^>]*>(.*?)</title>").unwrap());
        let link_re = link_re()
            .get_or_init(|| Regex::new(r#"(?i)href\s*=\s*["']([^"']+)["']"#).unwrap());

        let title = title_re
            .captures(&page.raw_html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| page.url.clone());

        let links = link_re
            .captures_iter(&page.raw_html)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .collect();

        ExtractedContent {
            title,
            text: page.raw_html.clone(),
            links,
        }
    }
}

/// 解析 API key: 优先 `WATERCRAWL_API_KEY` 环境变量，回退 `NeoTrixConfig.api_key`
fn resolve_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("WATERCRAWL_API_KEY") {
        if !k.is_empty() {
            return Some(k);
        }
    }
    NeoTrixConfig::load().api_key
}

/// SelfTest (T1): WaterCrawl 提取契约 — 离线 (合成页面) 验证 parse 正确性
pub struct WaterCrawlSelfTest;

impl SelfTest for WaterCrawlSelfTest {
    fn name(&self) -> &str {
        "nt_world_watercrawl"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let crawler = WaterCrawlExtractor;
        let page = CrawledPage {
            url: "https://example.com".into(),
            status: 200,
            raw_html: SAMPLE_HTML.into(),
        };
        let content = crawler.parse(&page);
        if content.title != "Sample Page" {
            return Err(vec![format!(
                "watercrawl: expected title 'Sample Page', got '{}'",
                content.title
            )]);
        }
        if !content.links.contains(&"https://example.com/a".to_string()) {
            return Err(vec!["watercrawl: parse should extract absolute links".into()]);
        }
        Ok(())
    }
}

/// 注册 WaterCrawl SelfTest
pub fn register_watercrawl_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(WaterCrawlSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    /// 起一个临时 HTTP server 返回固定 HTML，验证 fetch + parse 闭环
    fn spawn_mock_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(1) {
                if let Ok(mut s) = stream {
                    let body = SAMPLE_HTML;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = s.write_all(resp.as_bytes());
                }
            }
        });
        format!("http://{}", addr)
    }

    #[test]
    fn test_fetch_and_parse_via_mock_server() {
        let url = spawn_mock_server();
        let crawler = WaterCrawlExtractor;
        let page = crawler.fetch(&url);
        assert_eq!(page.status, 200);
        assert!(page.raw_html.contains("Sample Page"));
        let content = crawler.parse(&page);
        assert_eq!(content.title, "Sample Page");
        assert!(content.links.contains(&"https://example.com/a".to_string()));
        assert!(content.links.contains(&"/b".to_string()));
    }

    #[test]
    fn test_parse_offline() {
        let crawler = WaterCrawlExtractor;
        let page = CrawledPage {
            url: "https://example.com".into(),
            status: 200,
            raw_html: SAMPLE_HTML.into(),
        };
        let content = crawler.parse(&page);
        assert_eq!(content.title, "Sample Page");
        assert!(!content.links.is_empty());
    }

    #[test]
    fn test_selftest_passes() {
        let t = WaterCrawlSelfTest;
        assert_eq!(t.name(), "nt_world_watercrawl");
        assert!(t.self_test().is_ok());
    }

    #[test]
    #[ignore = "requires network access"]
    fn test_fetch_real_site() {
        let crawler = WaterCrawlExtractor;
        let page = crawler.fetch("https://example.com");
        assert_eq!(page.status, 200);
    }
}
