use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::engine::general_purpose::URL_SAFE as BASE64_URL;
use base64::Engine;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;

use super::crawler_core::*;
use super::crawler_parse::*;
use super::http_client::{Response, StealthHttpClient};

// ── Main TorCrawler ──────────────────────────────────────────────────

pub struct TorCrawler {
    queue: RwLock<VecDeque<CrawlJob>>,
    visited: RwLock<HashSet<String>>,
    index: RwLock<OnionIndex>,
    store_path: PathBuf,
    socks5_addr: String,
    concurrency: usize,
    stats: RwLock<CrawlerStats>,
    running: AtomicBool,
    search_count: AtomicU64,
    discovery_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    stealth_client: Arc<StealthHttpClient>,
}
impl TorCrawler {
    pub fn new(socks5_addr: String, store_path: PathBuf) -> Self {
        let path = store_path.join("tor_crawl");
        for sub in &["pages", "queue", "index"] {
            if let Err(e) = std::fs::create_dir_all(path.join(sub)) {
                log::warn!(
                    "[tor-nt_world_crawl] create dir {:?}: {}",
                    path.join(sub),
                    e
                );
            }
        }

        let socks = socks5_addr.clone();
        let stealth_client = Arc::new(StealthHttpClient::with_socks5(&format!(
            "socks5://{}",
            socks
        )));

        let nt_world_crawl = Self {
            queue: RwLock::new(VecDeque::new()),
            visited: RwLock::new(HashSet::new()),
            index: RwLock::new(OnionIndex::new()),
            store_path: path,
            socks5_addr,
            concurrency: MAX_CONCURRENCY,
            stats: RwLock::new(CrawlerStats {
                queued: 0,
                visited: 0,
                stored: 0,
                errors: 0,
                indexed: 0,
                search_engines_queried: 0,
                last_crawl: None,
            }),
            running: AtomicBool::new(false),
            search_count: AtomicU64::new(0),
            discovery_handle: Arc::new(Mutex::new(None)),
            stealth_client,
        };

        nt_world_crawl.load_state();
        nt_world_crawl
    }

    pub fn socks5_addr(&self) -> &str {
        &self.socks5_addr
    }

    // ── HTTP client via Tor SOCKS5 (StealthHttpClient) ────────────

    async fn stealth_fetch(&self, url: &str) -> Result<Response, String> {
        self.stealth_client.fetch(url).await
    }

    // ── Queue management ───────────────────────────────────────────

    pub async fn enqueue(&self, url: &str, depth: u32, priority: CrawlPriority) {
        let mut visited = self.visited.write().await;
        if visited.contains(url) {
            return;
        }
        if visited.len() >= MAX_VISITED {
            let to_remove: Vec<String> = visited.iter().take(MAX_VISITED / 2).cloned().collect();
            for u in to_remove {
                visited.remove(&u);
            }
        }
        visited.insert(url.to_string());
        drop(visited);

        let mut queue = self.queue.write().await;
        if queue.len() >= MAX_QUEUE {
            return;
        }
        queue.push_back(CrawlJob {
            url: url.to_string(),
            depth,
            priority,
            added_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    async fn dequeue(&self) -> Option<CrawlJob> {
        let mut queue = self.queue.write().await;
        // Sort by priority before popping (prioritize search results)
        if queue.len() > 1 {
            let mut items: Vec<CrawlJob> = queue.drain(..).collect();
            items.sort_by_key(|j| j.priority);
            for item in items {
                queue.push_back(item);
            }
        }
        queue.pop_front()
    }

    fn queue_len(&self) -> usize {
        // Best-effort read
        self.queue.try_read().map(|q| q.len()).unwrap_or(0)
    }

    // ── Search engine queries ──────────────────────────────────────

    /// Search a specific dark web search engine for a keyword
    /// Returns discovered .onion URLs
    pub async fn search_engine(&self, engine_url: &str, query: &str) -> Vec<String> {
        let mut found = Vec::new();

        // Build search URL depending on the search engine
        let search_url = if engine_url.contains("ahmia") || engine_url.contains("juhanurmi") {
            format!(
                "{}/search/?q={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        } else if engine_url.contains("torch") {
            format!(
                "{}/search?query={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        } else if engine_url.contains("duckduckgo") {
            format!(
                "{}/html/?q={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        } else if engine_url.contains("darksearch") {
            format!(
                "{}/search?q={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        } else if engine_url.contains("onionland") {
            format!(
                "{}/search?query={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        } else {
            format!(
                "{}/search?q={}",
                engine_url.trim_end_matches('/'),
                urlencoding(query)
            )
        };

        match self.stealth_fetch(&search_url).await {
            Ok(resp) if resp.status >= 200 && resp.status < 300 => {
                let body = resp.text().unwrap_or_default();
                let links = extract_onion_links(&body, engine_url);
                for link in links {
                    if !found.contains(&link) {
                        found.push(link);
                    }
                }
                // Update stats
                self.search_count.fetch_add(1, Ordering::Relaxed);
                let mut stats = self.stats.write().await;
                stats.search_engines_queried += 1;
            }
            Ok(resp) => {
                log::debug!(
                    "[tor-nt_world_crawl] search engine {} returned HTTP {}",
                    engine_url,
                    resp.status
                );
            }
            Err(e) => {
                log::debug!(
                    "[tor-nt_world_crawl] search engine {} error: {}",
                    engine_url,
                    e
                );
            }
        }

        found
    }

    /// Multi-engine search: query all known dark web search engines for keywords
    /// Returns combined unique discovered .onion URLs
    pub async fn search(&self, query: &str) -> Vec<String> {
        let mut all_found = Vec::new();
        for engine in SEARCH_ENGINES {
            let results = self.search_engine(engine, query).await;
            for url in results {
                if !all_found.contains(&url) {
                    all_found.push(url);
                }
            }
            // Be polite to search engines
            sleep(SEARCH_DELAY).await;
        }

        // Enqueue discovered URLs as priority CrawlResult
        for url in &all_found {
            self.enqueue(url, 0, CrawlPriority::CrawlResult).await;
        }

        all_found
    }

    // ── Page crawling ──────────────────────────────────────────────

    async fn crawl_page(&self, job: &CrawlJob) -> Result<CrawledPage, String> {
        let resp = self
            .stealth_fetch(&job.url)
            .await
            .map_err(|e| format!("GET {} failed: {}", job.url, e))?;

        let status = resp.status;
        if status >= 400 {
            return Err(format!("HTTP {}", status));
        }

        let body = resp.body;

        if body.len() > MAX_PAGE_BYTES {
            return Err(format!("page too large ({} bytes)", body.len()));
        }

        let html = String::from_utf8_lossy(&body);
        let title = extract_title(&html).unwrap_or_default();
        let body_text = extract_body_text(&html, 5000);
        let text_snippet = body_text.chars().take(300).collect::<String>();
        let links = extract_onion_links(&html, &job.url);
        let keywords = extract_keywords(&body_text, &title);

        let category = detect_category(&title, &body_text, &job.url);
        Ok(CrawledPage {
            url: job.url.clone(),
            title,
            text_snippet,
            body_text,
            links,
            keywords,
            content_type_hint: format!("{:?}", category),
            crawled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            depth: job.depth,
            http_status: status,
        })
    }

    // ── Storage ────────────────────────────────────────────────────

    fn store_page(&self, page: &CrawledPage) -> Result<(), String> {
        let url_hash = BASE64_URL.encode(page.url.as_bytes());
        let path = self
            .store_path
            .join("pages")
            .join(format!("{}.json", url_hash));
        let json =
            serde_json::to_string_pretty(page).map_err(|e| format!("serialize failed: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json)
            .map_err(|e| format!("write {} failed: {}", tmp.display(), e))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| format!("rename {} failed: {}", path.display(), e))?;
        Ok(())
    }

    fn index_page(&self, page: &CrawledPage) {
        let Ok(mut idx) = self.index.try_write() else {
            return;
        };
        idx.insert(page);
    }

    fn save_queue(&self) {
        let path = self.store_path.join("queue").join("pending.json");
        if let Ok(queue) = self.queue.try_read().map(|q| q) {
            let jobs: Vec<&CrawlJob> = queue.iter().collect();
            if let Ok(json) = serde_json::to_string(&jobs) {
                let tmp = path.with_extension("tmp");
                let _ = std::fs::write(&tmp, &json);
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    fn save_index(&self) {
        if let Ok(idx) = self.index.try_read() {
            idx.save(&self.store_path);
        }
    }

    fn load_state(&self) {
        // Load queue
        let path = self.store_path.join("queue").join("pending.json");
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(jobs) = serde_json::from_str::<Vec<CrawlJob>>(&data) {
                for j in jobs {
                    if let Ok(mut visited) = self.visited.try_write() {
                        if visited.len() < MAX_VISITED {
                            visited.insert(j.url.clone());
                        }
                    }
                    if let Ok(mut queue) = self.queue.try_write() {
                        if queue.len() < MAX_QUEUE {
                            queue.push_back(j);
                        }
                    }
                }
            }
        }
        // Load index
        if let Ok(mut idx) = self.index.try_write() {
            idx.load(&self.store_path);
        }
        // Recalculate stats
        let idx_len = self.index.try_read().map(|i| i.len()).unwrap_or(0);
        let visited_len = self.visited.try_read().map(|v| v.len()).unwrap_or(0);
        let queue_len = self.queue.try_read().map(|q| q.len()).unwrap_or(0);
        if let Ok(mut s) = self.stats.try_write() {
            s.indexed = idx_len;
            s.visited = visited_len;
            s.queued = queue_len;
        }
    }

    // ── Main crawl loop ────────────────────────────────────────────

    pub async fn run(self: Arc<Self>) {
        self.running.store(true, Ordering::Relaxed);

        // Enqueue default seeds
        for seed in DEFAULT_SEEDS {
            self.enqueue(seed, 0, CrawlPriority::Seed).await;
        }

        let mut last_save = Instant::now();
        let mut last_flush = Instant::now();
        let sem = Arc::new(Semaphore::new(self.concurrency));

        // Seed search engines with high-value queries for discovery
        let nt_world_crawl_clone = self.clone();
        *self.discovery_handle.lock().unwrap_or_else(|e| e.into_inner()) = Some(tokio::spawn(async move {
            sleep(Duration::from_secs(30)).await;
            let discovery_queries = [
                "onion directory",
                "hidden wiki",
                "tor links",
                "dark web index",
                "onion search",
            ];
            for q in &discovery_queries {
                let results = nt_world_crawl_clone.search(q).await;
                log::info!(
                    "[tor-nt_world_crawl] discovery query '{}' found {} .onion URLs",
                    q,
                    results.len()
                );
                sleep(Duration::from_secs(60)).await;
            }
        }));

        loop {
            if !self.running.load(Ordering::Relaxed) {
                log::info!("[tor-nt_world_crawl] stopping (running=false)");
                // Abort background discovery task so it doesn't outlive the crawl loop
                if let Some(h) = self.discovery_handle.lock().unwrap_or_else(|e| e.into_inner()).take() {
                    h.abort();
                }
                break;
            }
            let job = self.dequeue().await;

            match job {
                None => {
                    // Queue empty: do a discovery search to find more seeds
                    if self.search_count.load(Ordering::Relaxed) < 3 {
                        self.search("onion directory").await;
                    }
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
                Some(job) => {
                    let nt_world_crawl = self.clone();
                    let sem_clone = sem.clone();

                    tokio::spawn(async move {
                        let _permit = sem_clone.acquire_owned().await;

                        match nt_world_crawl.crawl_page(&job).await {
                            Ok(page) => {
                                let links = page.links.clone();
                                if let Err(e) = nt_world_crawl.store_page(&page) {
                                    log::warn!("[tor-nt_world_crawl] store failed: {}", e);
                                }
                                nt_world_crawl.index_page(&page);

                                // Enqueue discovered links
                                let next_priority = match job.priority {
                                    CrawlPriority::CrawlResult => CrawlPriority::Discovered,
                                    CrawlPriority::Seed => CrawlPriority::Discovered,
                                    CrawlPriority::Discovered if job.depth >= 3 => {
                                        CrawlPriority::Deep
                                    }
                                    _ => CrawlPriority::Discovered,
                                };
                                let next_depth = job.depth + 1;
                                for link in &links {
                                    nt_world_crawl
                                        .enqueue(link, next_depth, next_priority)
                                        .await;
                                }

                                let mut s = nt_world_crawl.stats.write().await;
                                s.visited += 1;
                                s.stored += 1;
                                s.indexed = nt_world_crawl
                                    .index
                                    .try_read()
                                    .map(|i| i.len())
                                    .unwrap_or(0);
                                s.last_crawl = Some(page.url.clone());
                                if s.visited % 10 == 0 {
                                    log::info!(
                                        "[tor-nt_world_crawl] crawled {} pages, idx {}, queue {}",
                                        s.visited,
                                        s.indexed,
                                        nt_world_crawl.queue_len()
                                    );
                                }
                            }
                            Err(e) => {
                                let mut s = nt_world_crawl.stats.write().await;
                                s.errors += 1;
                                if s.errors % 10 == 0 {
                                    log::warn!(
                                        "[tor-nt_world_crawl] {} errors (last: {}): {}",
                                        s.errors,
                                        job.url,
                                        e
                                    );
                                }
                            }
                        }
                    });

                    sleep(CRAWL_DELAY).await;
                }
            }

            // Periodic persistence
            if last_save.elapsed() > QUEUE_SAVE_INTERVAL {
                self.save_queue();
                last_save = Instant::now();
            }
            if last_flush.elapsed() > INDEX_FLUSH_INTERVAL {
                self.save_index();
                last_flush = Instant::now();
            }
        }
    }

    /// Search the local index for previously crawled content
    pub async fn search_index(&self, query: &str, max: usize) -> Vec<CrawlResult> {
        let idx = self.index.read().await;
        idx.search(query, max)
    }

    pub async fn stats(&self) -> CrawlerStats {
        let s = self.stats.read().await;
        let mut s = s.clone();
        s.queued = self.queue.try_read().map(|q| q.len()).unwrap_or(0);
        s
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        assert_eq!(extract_title(html), Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_title_empty() {
        assert_eq!(extract_title("<html><body>No title</body></html>"), None);
    }

    #[test]
    fn test_extract_body_text_removes_tags() {
        let html = "<html><body><p>Hello <b>World</b></p></body></html>";
        let text = extract_body_text(html, 50);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<b>"));
    }

    #[test]
    fn test_onion_links() {
        let html =
            r#"<a href="http://abc123.onion/">link</a><a href="http://def456.onion/">link2</a>"#;
        let links = extract_onion_links(html, "http://base.onion/");
        assert_eq!(links.len(), 2);
        assert!(links[0].contains("abc123.onion"));
    }

    #[test]
    fn test_onion_links_relative() {
        let html = r#"<a href="/page">link</a>"#;
        let links = extract_onion_links(html, "http://abc.onion/");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_onion_links_dedup() {
        let html = r#"<a href="http://abc.onion/">a</a><a href="http://abc.onion/">a</a>"#;
        let links = extract_onion_links(html, "");
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn test_onion_links_empty() {
        let links = extract_onion_links("<html></html>", "http://base.onion/");
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let body = "This is a test about onions and tor routing and privacy technology";
        let title = "Tor Routing Test";
        let keywords = extract_keywords(body, title);
        assert!(
            keywords.contains(&"routing".to_string()) || keywords.contains(&"routing".to_string())
        );
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_crawl_job_serde() {
        let job = CrawlJob {
            url: "http://test.onion/".into(),
            depth: 1,
            priority: CrawlPriority::Seed,
            added_at: 12345,
        };
        let json = serde_json::to_string(&job).expect("serialize");
        let deser: CrawlJob = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.url, job.url);
        assert_eq!(deser.priority, CrawlPriority::Seed);
    }

    #[test]
    fn test_crawled_page_serde() {
        let page = CrawledPage {
            url: "http://test.onion/".into(),
            title: "Test".into(),
            text_snippet: "snippet".into(),
            body_text: "body".into(),
            links: vec!["http://other.onion/".into()],
            keywords: vec!["test".into()],
            content_type_hint: "General".into(),
            crawled_at: 12345,
            depth: 1,
            http_status: 200,
        };
        let json = serde_json::to_string(&page).expect("serialize");
        let deser: CrawledPage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.url, page.url);
    }

    #[test]
    fn test_detect_category_marketplace() {
        let cat = detect_category(
            "Buy cheap stuff",
            "market prices escrow",
            "http://market.onion/",
        );
        assert_eq!(cat, ContentCategory::Marketplace);
    }

    #[test]
    fn test_detect_category_forum() {
        let cat = detect_category(
            "Discussion board",
            "forum thread comment",
            "http://forum.onion/",
        );
        assert_eq!(cat, ContentCategory::Forum);
    }

    #[test]
    fn test_onion_index_search() {
        let mut idx = OnionIndex::new();
        idx.insert(&CrawledPage {
            url: "http://abc.onion/".into(),
            title: "Test Page About Privacy".into(),
            text_snippet: "privacy and anonymity".into(),
            body_text: "".into(),
            links: vec![],
            keywords: vec!["privacy".into(), "anonymity".into()],
            content_type_hint: "General".into(),
            crawled_at: 100,
            depth: 0,
            http_status: 200,
        });
        let results = idx.search("privacy", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "http://abc.onion/");
    }

    #[test]
    fn test_onion_index_empty_search() {
        let idx = OnionIndex::new();
        assert!(idx.search("anything", 10).is_empty());
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("test"), "test");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(CrawlPriority::CrawlResult < CrawlPriority::Seed);
        assert!(CrawlPriority::Seed < CrawlPriority::Discovered);
        assert!(CrawlPriority::Discovered < CrawlPriority::Deep);
    }
}
