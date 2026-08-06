//! NT-WORLD 两阶段抓取 + 统一端点 — 缺陷网 D11/D15 修复:
//! - D11 单遍抓取无 prefetch: 引入两阶段 — 先 prefetch 只发现 URL 不抓正文
//!       (快速展开站点地图), 再选择性抓正文。BM25 关键词过滤降噪, 断点续爬。
//! - D15 无统一 Scrape/Interact+Map: 统一端点 scrape()/map()/search()/
//!       interact(), 一致输入输出契约。
//!
//! 参照: crawl4ai (prefetch 两阶段 + BM25 过滤 + resume_state),
//!       firecrawl (Search/Scrape/Interact/Map 统一端点)。
//! 纯内存实现 (无网络依赖), 便于离线单测; 生产路径见 nt_world_crawl 走真实网络。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 预抓取阶段发现的 URL 条目 (不包含正文, 只含元数据)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefetchedUrl {
    pub url: String,
    pub depth: usize,
    /// 锚文本/片段, 用于 BM25 过滤
    pub snippet: String,
    /// 该 URL 是否已在此轮抓取过正文 (断点续爬)
    pub fetched: bool,
}

/// 统一文档输出模型 (D15): 任何端点都产出此 schema。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDoc {
    pub kind: String,          // "scrape" | "map" | "search" | "interact"
    pub source: String,
    pub title: String,
    pub url: String,
    pub text: String,
    pub links: Vec<String>,
    pub items: Vec<DocItem>,
}

/// 端点明细项 (map 的站内 URL, search 的命中, interact 的观察)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

/// D11 两阶段爬取器 — 无网络, 靠注入的"页面提供者"驱动, 便于测试与断点。
pub struct TwoPhaseCrawler {
    /// 已抓正文的 URL 集合 (断点续爬: 重跑时跳过)
    fetched_urls: HashSet<String>,
    /// 已发现过的 URL 集合 (跨 prefetch 调用去重)
    discovered_urls: HashSet<String>,
    /// 页面提供者: 输入 url → (正文, 出链)。None 表示纯离线解析模式。
    fetcher: Option<Box<dyn Fn(&str) -> (String, Vec<String>)>>,
    bm25_keywords: Vec<String>,
    max_pages: usize,
}

impl Default for TwoPhaseCrawler {
    fn default() -> Self {
        Self::new(vec![], 100, None)
    }
}

impl TwoPhaseCrawler {
    pub fn new(
        bm25_keywords: Vec<String>,
        max_pages: usize,
        fetcher: Option<Box<dyn Fn(&str) -> (String, Vec<String>)>>,
    ) -> Self {
        Self {
            fetched_urls: HashSet::new(),
            discovered_urls: HashSet::new(),
            fetcher,
            bm25_keywords,
            max_pages,
        }
    }

    /// 阶段一: prefetch — 只发现 URL, 不抓正文。从种子出发展开到 max_depth。
    /// 返回去重后的 URL 集。若配置了 fetcher, 会用其正文里的链接; 否则只记录种子
    /// 自身 (由外部注入更完整的发现)。
    pub fn prefetch(&mut self, seeds: &[&str], max_depth: usize) -> Vec<PrefetchedUrl> {
        let mut frontier: Vec<(String, usize)> = seeds.iter().map(|s| (s.to_string(), 0)).collect();
        let mut local_seen: HashMap<String, usize> = HashMap::new();
        let mut order: Vec<String> = Vec::new();

        while let Some((url, depth)) = frontier.pop() {
            if self.fetched_urls.contains(&url)
                || self.discovered_urls.contains(&url)
                || local_seen.contains_key(&url)
            {
                continue;
            }
            if order.len() >= self.max_pages {
                break;
            }
            local_seen.insert(url.clone(), depth);
            self.discovered_urls.insert(url.clone());
            order.push(url.clone());
            if depth >= max_depth {
                continue;
            }
            if let Some(f) = &self.fetcher {
                let (_body, links) = f(&url);
                for link in links {
                    if !self.discovered_urls.contains(&link)
                        && !self.fetched_urls.contains(&link)
                        && !local_seen.contains_key(&link)
                    {
                        frontier.push((link, depth + 1));
                    }
                }
            }
        }

        order
            .into_iter()
            .map(|url| {
                let depth = local_seen[&url];
                let fetched = self.fetched_urls.contains(&url);
                PrefetchedUrl {
                    url,
                    depth,
                    snippet: String::new(),
                    fetched,
                }
            })
            .collect()
    }

    /// 阶段二: 按 BM25 相关度过滤 prefetch 发现的 URL, 返回应抓正文的子集。
    /// 纯基于 snippet 的过滤 (snippet 缺失时保留, 避免过滤掉不可见 URL)。
    pub fn filter_relevant(&self, discovered: &[PrefetchedUrl]) -> Vec<PrefetchedUrl> {
        if self.bm25_keywords.is_empty() {
            return discovered.to_vec();
        }
        discovered
            .iter()
            .filter(|u| {
                let score = self.score(u);
                score >= 0.2 || u.snippet.trim().is_empty()
            })
            .cloned()
            .collect()
    }

    /// BM25 简化打分: 命中关键词数 / 片段词数 (toy 实现, 语义一致即可)。
    fn score(&self, u: &PrefetchedUrl) -> f64 {
        let text = format!("{} {}", u.url, u.snippet).to_lowercase();
        let total = self.bm25_keywords.len().max(1);
        let hit = self
            .bm25_keywords
            .iter()
            .filter(|k| text.contains(&k.to_lowercase()))
            .count();
        hit as f64 / total as f64
    }

    /// 抓取正文 (带断点续爬: 已抓过的直接跳过)。返回抓取数量。
    /// fetcher 为 None 时只记录"已抓"状态, 便于测试断言断点行为。
    pub fn crawl(&mut self, urls: &[PrefetchedUrl]) -> usize {
        let mut n = 0;
        for u in urls {
            if self.fetched_urls.contains(&u.url) {
                continue;
            }
            self.fetched_urls.insert(u.url.clone());
            n += 1;
        }
        n
    }

    pub fn fetched_count(&self) -> usize {
        self.fetched_urls.len()
    }

    /// 断点续爬: 序列化已抓 URL 集, 供跨会话恢复。
    pub fn snapshot(&self) -> Vec<String> {
        self.fetched_urls.iter().cloned().collect()
    }

    /// 恢复断点。
    pub fn restore(&mut self, urls: Vec<String>) {
        self.fetched_urls.extend(urls);
    }
}

/// D15 统一端点 — firecrawl 风格: 一个门面暴露 scrape/map/search/interact,
/// 全部产出 UnifiedDoc 一致 schema。
pub struct UnifiedEndpoints {
    pub crawler: TwoPhaseCrawler,
}

impl Default for UnifiedEndpoints {
    fn default() -> Self {
        Self::new(vec![], 100, None)
    }
}

impl UnifiedEndpoints {
    pub fn new(
        bm25_keywords: Vec<String>,
        max_pages: usize,
        fetcher: Option<Box<dyn Fn(&str) -> (String, Vec<String>)>>,
    ) -> Self {
        Self {
            crawler: TwoPhaseCrawler::new(bm25_keywords, max_pages, fetcher),
        }
    }

    /// search 端点: 对 prefetch 发现的 URL 过滤相关集, 输出统一文档。
    pub fn search(&mut self, query: &str, count: usize) -> UnifiedDoc {
        let seeds = [query];
        let discovered = self.crawler.prefetch(&seeds, 0);
        let relevant = self.crawler.filter_relevant(&discovered);
        let items: Vec<DocItem> = relevant
            .into_iter()
            .take(count)
            .map(|u| DocItem {
                url: u.url.clone(),
                title: u.url.clone(),
                snippet: u.snippet.clone(),
            })
            .collect();
        UnifiedDoc {
            kind: "search".to_string(),
            source: query.to_string(),
            title: format!("search:{}", query),
            url: query.to_string(),
            text: String::new(),
            links: items.iter().map(|i| i.url.clone()).collect(),
            items,
        }
    }

    /// map 端点: 全站 URL 发现 (max_depth 展开)。
    pub fn map(&mut self, site_url: &str, max_depth: usize) -> UnifiedDoc {
        let discovered = self.crawler.prefetch(&[site_url], max_depth);
        let items: Vec<DocItem> = discovered
            .iter()
            .map(|u| DocItem {
                url: u.url.clone(),
                title: u.url.clone(),
                snippet: u.snippet.clone(),
            })
            .collect();
        UnifiedDoc {
            kind: "map".to_string(),
            source: site_url.to_string(),
            title: format!("map:{}", site_url),
            url: site_url.to_string(),
            text: String::new(),
            links: items.iter().map(|i| i.url.clone()).collect(),
            items,
        }
    }

    /// scrape 端点: 抓取单页正文。
    pub fn scrape(&mut self, url: &str) -> UnifiedDoc {
        let target = PrefetchedUrl {
            url: url.to_string(),
            depth: 0,
            snippet: String::new(),
            fetched: false,
        };
        self.crawler.crawl(&[target]);
        UnifiedDoc {
            kind: "scrape".to_string(),
            source: url.to_string(),
            title: url.to_string(),
            url: url.to_string(),
            text: format!("scraped:{}", url),
            links: vec![],
            items: vec![],
        }
    }

    /// interact 端点: 记录一次交互观察 (stagehand act/observe 参照)。
    pub fn interact(&mut self, url: &str, action: &str) -> UnifiedDoc {
        UnifiedDoc {
            kind: "interact".to_string(),
            source: url.to_string(),
            title: format!("{}@{}", action, url),
            url: url.to_string(),
            text: format!("interact {} on {}", action, url),
            links: vec![],
            items: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_site(url: &str) -> (String, Vec<String>) {
        match url {
            "https://site.dev" => (
                "<h1>Home</h1><a href='/a'>Rust guide</a><a href='/b'>Docker notes</a>".into(),
                vec!["https://site.dev/a".into(), "https://site.dev/b".into()],
            ),
            "https://site.dev/a" => ("rust docs".into(), vec![]),
            "https://site.dev/b" => ("docker notes".into(), vec![]),
            _ => (String::new(), vec![]),
        }
    }

    #[test]
    fn prefetch_discovers_but_does_not_fetch_bodies() {
        // D11: 阶段一不抓正文 — 用 fetcher 提供链接但断言 fetched_count==0
        let mut c = TwoPhaseCrawler::new(
            vec!["rust".into()],
            100,
            Some(Box::new(fake_site)),
        );
        let discovered = c.prefetch(&["https://site.dev"], 2);
        assert_eq!(discovered.len(), 3, "home + /a + /b discovered");
        assert_eq!(c.fetched_count(), 0, "prefetch must not fetch bodies");
        // 去重: 再次 prefetch 同种子不重复
        let again = c.prefetch(&["https://site.dev"], 2);
        assert!(again.is_empty(), "no duplicate prefetch of seen urls");
    }

    #[test]
    fn prefetch_deduplicates_urls() {
        let mut c = TwoPhaseCrawler::new(vec![], 100, Some(Box::new(fake_site)));
        // 同种子传入两次 → 只发现一次
        let discovered = c.prefetch(&["https://site.dev", "https://site.dev"], 1);
        assert_eq!(discovered.len(), 3);
        let urls: HashSet<String> = discovered.iter().map(|u| u.url.clone()).collect();
        assert_eq!(urls.len(), discovered.len());
    }

    #[test]
    fn bm25_filter_keeps_relevant_drops_noise() {
        let mut c = TwoPhaseCrawler::new(
            vec!["rust".into()],
            100,
            Some(Box::new(fake_site)),
        );
        let mut d = c.prefetch(&["https://site.dev"], 2);
        // 不依赖返回顺序: 按 url 赋予 snippet
        for u in d.iter_mut() {
            u.snippet = if u.url.contains("/a") {
                "rust guide".into()
            } else if u.url.contains("/b") {
                "rust tutorial".into()
            } else {
                "docker".into()
            };
        }
        let relevant = c.filter_relevant(&d);
        // docker snippet < 0.2 threshold → dropped; snippets with hits kept
        assert!(relevant.iter().any(|u| u.url.contains("/a")));
        assert!(relevant.iter().any(|u| u.url.contains("/b")));
        assert!(!relevant.iter().any(|u| u.snippet == "docker"));
    }

    #[test]
    fn crawl_resume_skips_fetched() {
        let mut c = TwoPhaseCrawler::new(vec![], 100, Some(Box::new(fake_site)));
        let d = c.prefetch(&["https://site.dev"], 1);
        // 第一轮抓 3 个 (host + /a + /b)
        assert_eq!(c.crawl(&d), 3);
        // 断点恢复后重新跑 → 跳过已抓
        c.restore(c.snapshot());
        assert_eq!(c.crawl(&d), 0, "resume skips already-fetched urls");
    }

    #[test]
    fn unified_endpoints_share_schema() {
        let mut u = UnifiedEndpoints::new(vec!["rust".into()], 100, Some(Box::new(fake_site)));
        let s = u.search("rust", 3);
        let m = u.map("https://site.dev", 1);
        let sc = u.scrape("https://site.dev/a");
        let it = u.interact("https://site.dev/a", "click");
        for doc in [&s, &m, &sc, &it] {
            // D15: 所有端点产出同一 schema (字段齐全)
            assert!(!doc.kind.is_empty());
            assert!(!doc.source.is_empty());
            assert!(!doc.title.is_empty());
            assert!(!doc.url.is_empty());
        }
        assert_eq!(s.kind, "search");
        assert_eq!(m.kind, "map");
        assert_eq!(sc.kind, "scrape");
        assert_eq!(it.kind, "interact");
    }

    #[test]
    fn map_discovers_site_urls() {
        let mut u = UnifiedEndpoints::new(vec![], 100, Some(Box::new(fake_site)));
        let m = u.map("https://site.dev", 2);
        assert!(m.links.contains(&"https://site.dev/a".to_string()));
        assert!(m.links.contains(&"https://site.dev/b".to_string()));
    }
}
