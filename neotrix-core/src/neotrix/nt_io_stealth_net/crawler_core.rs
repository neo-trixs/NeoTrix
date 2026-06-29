use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────

pub(crate) const MAX_CONCURRENCY: usize = 5;
pub(crate) const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub(crate) const CRAWL_DELAY: std::time::Duration = std::time::Duration::from_millis(1500);
pub(crate) const MAX_PAGE_BYTES: usize = 2_097_152;
pub(crate) const QUEUE_SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(120);
pub(crate) const INDEX_FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(300);
pub(crate) const MAX_QUEUE: usize = 5_000;
pub(crate) const MAX_VISITED: usize = 20_000;
pub(crate) const MAX_INDEX_SIZE: usize = 50_000;
pub(crate) const SEARCH_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) const SEARCH_ENGINE_AHMIA: &str =
    "http://juhanurmihxlp77nkq76byazcldy2hlmovfu2epvl5ankdibsot4csyd.onion";
pub(crate) const SEARCH_ENGINE_TORCH: &str =
    "http://xmh57jrknzkhv6y3ls3ubitzfqnkrwxhopf5aygthi7d6rplyvk3noyd.onion";
pub(crate) const SEARCH_ENGINE_DUCKDUCKGO: &str =
    "http://duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion";
pub(crate) const SEARCH_ENGINE_TORLINKS: &str =
    "http://torlinksge6enmcyyuxjpjkoouw4oorgdgeo7ftnq3zodj7g2zxi3kyd.onion";
pub(crate) const SEARCH_ENGINE_DARKSEARCH: &str =
    "http://darksearchivrio6kz5zqk6zl7qvbrtd4pwh3p3tr2da7ixy5z7j4z6yd.onion";
pub(crate) const SEARCH_ENGINE_ONIONLAND: &str =
    "http://onionland3dg4f3k7k6hqzr7vz3z5kjk2zqkz5kjn7yzfjz7zq7p7ad.onion";

pub(crate) const SEARCH_ENGINES: &[&str] = &[
    SEARCH_ENGINE_AHMIA,
    SEARCH_ENGINE_TORCH,
    SEARCH_ENGINE_DUCKDUCKGO,
    SEARCH_ENGINE_TORLINKS,
    SEARCH_ENGINE_DARKSEARCH,
    SEARCH_ENGINE_ONIONLAND,
];

pub(crate) const DEFAULT_SEEDS: &[&str] = &[
    "http://danielas3rtn54uwmofdo3qx2lvb3o37p3cskve4lw5l3k7ekpcid.onion/",
    "http://thehiddenwiki6ndgfkmh3zq5kq6zqk5zqk6zqk5zqk6zqk5zqk6zqkid.onion/",
];

// ── Priority levels ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrawlPriority {
    CrawlResult = 0,
    Seed = 1,
    Discovered = 2,
    Deep = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlJob {
    pub url: String,
    pub depth: u32,
    pub priority: CrawlPriority,
    pub added_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    pub url: String,
    pub title: String,
    pub text_snippet: String,
    pub body_text: String,
    pub links: Vec<String>,
    pub keywords: Vec<String>,
    pub content_type_hint: String,
    pub crawled_at: u64,
    pub depth: u32,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionIndexEntry {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub keywords: Vec<String>,
    pub depth: u32,
    pub crawled_at: u64,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerStats {
    pub queued: usize,
    pub visited: usize,
    pub stored: usize,
    pub errors: usize,
    pub indexed: usize,
    pub search_engines_queried: u64,
    pub last_crawl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub crawled_at: u64,
}

// ── Content category hints ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentCategory {
    General,
    Forum,
    Marketplace,
    News,
    Wiki,
    Search,
    File,
    Unknown,
}

pub(crate) fn detect_category(title: &str, body: &str, url: &str) -> ContentCategory {
    let combined = format!("{} {} {}", title, body, url).to_lowercase();
    if combined.contains("market")
        || combined.contains("shop")
        || combined.contains("buy")
        || combined.contains("escrow")
        || combined.contains("price")
    {
        ContentCategory::Marketplace
    } else if combined.contains("forum")
        || combined.contains("board")
        || combined.contains("thread")
        || combined.contains("discuss")
        || combined.contains("comment")
    {
        ContentCategory::Forum
    } else if combined.contains("news")
        || combined.contains("article")
        || combined.contains("blog")
        || combined.contains("press")
    {
        ContentCategory::News
    } else if combined.contains("wiki") || combined.contains("wikipedia") {
        ContentCategory::Wiki
    } else if combined.contains("search")
        || combined.contains("crawl")
        || combined.contains("index")
    {
        ContentCategory::Search
    } else if url.ends_with(".pdf")
        || url.ends_with(".zip")
        || url.ends_with(".tar.gz")
        || combined.contains("download")
    {
        ContentCategory::File
    } else {
        ContentCategory::General
    }
}

// ── OnionIndex ────────────────────────────────────────────────────────

pub(crate) struct OnionIndex {
    entries: Vec<OnionIndexEntry>,
    keyword_map: HashMap<String, Vec<usize>>,
    url_map: HashMap<String, usize>,
}

impl OnionIndex {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            keyword_map: HashMap::new(),
            url_map: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, page: &CrawledPage) {
        if self.url_map.contains_key(&page.url) {
            return;
        }
        let entry = OnionIndexEntry {
            url: page.url.clone(),
            title: page.title.clone(),
            snippet: page.text_snippet.clone(),
            keywords: page.keywords.clone(),
            depth: page.depth,
            crawled_at: page.crawled_at,
            http_status: page.http_status,
        };
        let idx = self.entries.len();
        self.entries.push(entry);
        self.url_map.insert(page.url.clone(), idx);
        for kw in &page.keywords {
            self.keyword_map.entry(kw.clone()).or_default().push(idx);
        }
        if self.entries.len() > MAX_INDEX_SIZE {
            let excess = self.entries.len() - MAX_INDEX_SIZE;
            for _ in 0..excess {
                if let Some(old) = self.entries.first().cloned() {
                    self.url_map.remove(&old.url);
                    self.entries.remove(0);
                }
            }
        }
    }

    pub(crate) fn search(&self, query: &str, max: usize) -> Vec<CrawlResult> {
        let terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|t| t.len() > 2)
            .map(|t| t.to_string())
            .collect();
        if terms.is_empty() {
            return vec![];
        }

        let mut scores: Vec<(f64, usize)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let mut score = 0.0;
                let title_lower = e.title.to_lowercase();
                let snippet_lower = e.snippet.to_lowercase();
                for term in &terms {
                    if title_lower.contains(term) {
                        score += 10.0;
                    }
                    if e.keywords.iter().any(|k| k.contains(term)) {
                        score += 5.0;
                    }
                    if snippet_lower.contains(term) {
                        score += 2.0;
                    }
                }
                (score, i)
            })
            .collect();
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(max);

        scores
            .into_iter()
            .map(|(score, i)| {
                let e = &self.entries[i];
                CrawlResult {
                    url: e.url.clone(),
                    title: e.title.clone(),
                    snippet: e.snippet.clone(),
                    score,
                    crawled_at: e.crawled_at,
                }
            })
            .collect()
    }

    pub(crate) fn save(&self, path: &PathBuf) {
        let file_path = path.join("tor_index.json");
        if let Ok(json) = serde_json::to_string(&self.entries) {
            let tmp = file_path.with_extension("tmp");
            let _ = std::fs::write(&tmp, &json);
            let _ = std::fs::rename(&tmp, &file_path);
        }
    }

    pub(crate) fn load(&mut self, path: &PathBuf) {
        let file_path = path.join("tor_index.json");
        let data = std::fs::read_to_string(&file_path).ok();
        let entries: Vec<OnionIndexEntry> = data
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        for e in entries {
            if self.url_map.contains_key(&e.url) {
                continue;
            }
            let _idx = self.entries.len();
            self.entries.push(e);
        }
        self.keyword_map.clear();
        self.url_map.clear();
        for (i, e) in self.entries.iter().enumerate() {
            self.url_map.insert(e.url.clone(), i);
            for kw in &e.keywords {
                self.keyword_map.entry(kw.clone()).or_default().push(i);
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}
