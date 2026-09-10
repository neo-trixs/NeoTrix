//! L5 Knowledge Store trait — abstraction over L1 KB store functions.
//!
//! Follows dependency inversion: L5 defines the interface, L1 implements.
//! L5 code uses this trait instead of importing L1 store functions directly.

use std::collections::HashMap;

/// Crawl queue item from KB
#[derive(Debug, Clone)]
pub struct CrawlQueueItem {
    pub id: String,
    pub url: String,
    pub depth: Option<i64>,
    pub domain: Option<String>,
    pub priority: Option<i64>,
    pub status: Option<String>,
    pub discovered_at: Option<i64>,
    pub last_attempt: Option<i64>,
    pub retry_count: Option<i64>,
    pub error_message: Option<String>,
}

/// Knowledge Store trait — L5 contract for KB store operations.
///
/// This trait abstracts the L1 `nt_memory_store` functions that L5 needs.
/// The actual implementation lives in L1 and is injected via DI.
pub trait KnowledgeStore: Send + Sync {
    /// Claim the next pending URL from the crawl queue.
    fn claim_next_crawl_url(&self) -> Result<Option<CrawlQueueItem>, String>;

    /// Mark a crawl queue item as completed or failed.
    fn mark_crawl_complete(&self, id: &str, success: bool, error: Option<&str>) -> Result<(), String>;

    /// Count nodes grouped by domain.
    fn count_nodes_by_domain(&self) -> Result<HashMap<String, usize>, String>;

    /// Enqueue seed URLs into the crawl queue.
    /// Each tuple is (url, priority, domain).
    fn enqueue_seed_urls(&self, urls: &[(&str, i64, &str)]) -> Result<usize, String>;

    /// Extract safe HTML content from raw HTML, returning (title, text).
    fn extract_html_content(&self, html: &str) -> (String, String);

    /// Check if a URL is safe to fetch (SSRF guard).
    fn is_safe_fetch_url(&self, url: &str) -> bool;
}

/// Wrapper that delegates to the L1 `nt_memory_store` implementation.
pub struct L1KnowledgeStore;

impl KnowledgeStore for L1KnowledgeStore {
    fn claim_next_crawl_url(&self) -> Result<Option<CrawlQueueItem>, String> {
        // This is a placeholder — the actual implementation requires a Connection.
        // In practice, KB methods that need a connection should be called through
        // the KnowledgeBase struct which holds the connection.
        Err("L1KnowledgeStore requires a KnowledgeBase connection".into())
    }

    fn mark_crawl_complete(&self, _id: &str, _success: bool, _error: Option<&str>) -> Result<(), String> {
        Err("L1KnowledgeStore requires a KnowledgeBase connection".into())
    }

    fn count_nodes_by_domain(&self) -> Result<HashMap<String, usize>, String> {
        Err("L1KnowledgeStore requires a KnowledgeBase connection".into())
    }

    fn enqueue_seed_urls(&self, _urls: &[(&str, i64, &str)]) -> Result<usize, String> {
        Err("L1KnowledgeStore requires a KnowledgeBase connection".into())
    }

    fn extract_html_content(&self, html: &str) -> (String, String) {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
    }

    fn is_safe_fetch_url(&self, url: &str) -> bool {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
    }
}

/// KB-backed KnowledgeStore that delegates to a KnowledgeBase instance.
pub struct KbKnowledgeStore {
    pub kb: crate::neotrix::nt_memory_kb::KnowledgeBase,
}

impl KnowledgeStore for KbKnowledgeStore {
    fn claim_next_crawl_url(&self) -> Result<Option<CrawlQueueItem>, String> {
        let conn = self.kb.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
        let item = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::claim_next_crawl_url(&conn)
            .map_err(|e| format!("claim_next_crawl_url: {}", e))?;
        Ok(item.map(|i| CrawlQueueItem {
            id: i.id,
            url: i.url,
            depth: i.depth,
            domain: i.domain,
            priority: i.priority,
            status: i.status,
            discovered_at: i.discovered_at,
            last_attempt: i.last_attempt,
            retry_count: i.retry_count,
            error_message: i.error_message,
        }))
    }

    fn mark_crawl_complete(&self, id: &str, success: bool, error: Option<&str>) -> Result<(), String> {
        let conn = self.kb.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::mark_crawl_complete(&conn, id, success, error)
            .map_err(|e| format!("mark_crawl_complete: {}", e))
    }

    fn count_nodes_by_domain(&self) -> Result<HashMap<String, usize>, String> {
        let conn = self.kb.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_domain(&conn)
            .map_err(|e| format!("count_nodes_by_domain: {}", e))
    }

    fn enqueue_seed_urls(&self, urls: &[(&str, i64, &str)]) -> Result<usize, String> {
        let conn = self.kb.conn.lock().map_err(|e| format!("KB lock: {}", e))?;
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::enqueue_seed_urls(&conn, urls)
            .map_err(|e| format!("enqueue_seed_urls: {}", e))
    }

    fn extract_html_content(&self, html: &str) -> (String, String) {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html)
    }

    fn is_safe_fetch_url(&self, url: &str) -> bool {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::is_safe_fetch_url(url)
    }
}
