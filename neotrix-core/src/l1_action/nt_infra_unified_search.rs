//! Unified Search API — single entry point for web, code, and KB search.
//!
//! Routes queries to the appropriate backend based on `SearchScope`.
//! `SearchScope::All` fans out to all backends in parallel and fuses results
//! via Reciprocal Rank Fusion (RRF).
//!
//! Backends are injected as trait objects so this module has zero direct
//! dependencies on L2 web search or concrete DB handles.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// Types
// ════════════════════════════════════════════════════════════════

/// Which backend(s) to query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    /// Open-internet search via web backends.
    Web,
    /// Source-code search via ripgrep + symbol index.
    Code,
    /// Knowledge-base search via FTS5 + vector hybrid.
    Knowledge,
    /// Parallel fan-out to all backends with RRF fusion.
    All,
}

/// A single search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Free-text query.
    pub query: String,
    /// Which backend(s) to target.
    pub scope: SearchScope,
    /// Maximum results per backend (default 10).
    pub max_results: usize,
    /// Optional key-value filters (e.g. `file_ext` for code, `node_type` for KB).
    #[serde(default)]
    pub filters: HashMap<String, String>,
}

impl SearchQuery {
    pub fn web(query: impl Into<String>) -> Self {
        Self { query: query.into(), scope: SearchScope::Web, max_results: 10, filters: HashMap::new() }
    }
    pub fn code(query: impl Into<String>, path: impl Into<String>) -> Self {
        let mut filters = HashMap::new();
        filters.insert("path".to_string(), path.into());
        Self { query: query.into(), scope: SearchScope::Code, max_results: 10, filters }
    }
    pub fn knowledge(query: impl Into<String>) -> Self {
        Self { query: query.into(), scope: SearchScope::Knowledge, max_results: 10, filters: HashMap::new() }
    }
    pub fn all(query: impl Into<String>) -> Self {
        Self { query: query.into(), scope: SearchScope::All, max_results: 10, filters: HashMap::new() }
    }
}

/// Where a result came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSource {
    Web,
    Code,
    Knowledge,
}

/// A single unified result regardless of origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    /// Title or symbol name.
    pub title: String,
    /// URL (web), file path (code), or node id (KB).
    pub location: String,
    /// Text snippet / content preview.
    pub snippet: String,
    /// Fused relevance score (higher = more relevant).
    pub score: f64,
    /// Which backend produced this result.
    pub source: SearchSource,
    /// Source-specific extra data.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

// ════════════════════════════════════════════════════════════════
// Backend traits
// ════════════════════════════════════════════════════════════════

/// Trait that web search backends implement.
pub trait WebSearchBackend: Send + Sync {
    fn search(&self, query: &str, max_results: usize) -> Vec<UnifiedSearchResult>;
}

/// Trait that code search backends implement.
pub trait CodeSearchBackend: Send + Sync {
    fn search(&self, query: &str, path: &Path, max_results: usize) -> Vec<UnifiedSearchResult>;
}

/// Trait that knowledge-base search backends implement.
pub trait KnowledgeSearchBackend: Send + Sync {
    fn search(&self, query: &str, max_results: usize) -> Vec<UnifiedSearchResult>;
}

// ════════════════════════════════════════════════════════════════
// Adapters — wrap existing search engines into backend traits
// ════════════════════════════════════════════════════════════════

/// Adapter: wraps `crate::l2_perception::nt_core_code_search::CodeSearchEngine`.
pub struct CodeSearchAdapter;

impl CodeSearchBackend for CodeSearchAdapter {
    fn search(&self, query: &str, path: &Path, max_results: usize) -> Vec<UnifiedSearchResult> {
        crate::l1_action::nt_action_facade::nt_core_code_search::CodeSearchEngine::search(query, path)
            .into_iter()
            .take(max_results)
            .enumerate()
            .map(|(i, r)| UnifiedSearchResult {
                title: format!("{}:{}", r.file, r.line),
                location: r.file.clone(),
                snippet: r.content,
                score: 1.0 / (60.0 + (i + 1) as f64),
                source: SearchSource::Code,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("line".to_string(), serde_json::json!(r.line));
                    m.insert("column".to_string(), serde_json::json!(r.column));
                    m
                },
            })
            .collect()
    }
}

/// Adapter: wraps `super::nt_memory::nt_memory_kb::nt_memory_search::search_fts`.
///
/// Requires a `rusqlite::Connection` to be injected.
pub struct KnowledgeSearchAdapter {
    conn: Arc<std::sync::Mutex<rusqlite::Connection>>,
}

impl KnowledgeSearchAdapter {
    pub fn new(conn: rusqlite::Connection) -> Self {
        Self { conn: Arc::new(std::sync::Mutex::new(conn)) }
    }
}

impl KnowledgeSearchBackend for KnowledgeSearchAdapter {
    fn search(&self, query: &str, max_results: usize) -> Vec<UnifiedSearchResult> {
        let Ok(conn) = self.conn.lock() else { return Vec::new() };
        super::nt_memory::nt_memory_kb::nt_memory_search::search_fts(&conn, query, max_results)
            .map(|results| {
                results
                    .into_iter()
                    .map(|r| UnifiedSearchResult {
                        title: r.node.title,
                        location: r.node.id.clone(),
                        snippet: r.node.summary.unwrap_or_default(),
                        score: r.score,
                        source: SearchSource::Knowledge,
                        metadata: {
                            let mut m = HashMap::new();
                            m.insert("node_type".to_string(), serde_json::json!(format!("{:?}", r.node.node_type)));
                            m.insert("confidence".to_string(), serde_json::json!(r.node.confidence));
                            m
                        },
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ════════════════════════════════════════════════════════════════
// UnifiedSearch — the main entry point
// ════════════════════════════════════════════════════════════════

/// Unified search orchestrator.
///
/// Holds optional backend references. Missing backends are silently skipped
/// for their scope (never panic, never block).
pub struct UnifiedSearch {
    web: Option<Arc<dyn WebSearchBackend>>,
    code: Option<Arc<dyn CodeSearchBackend>>,
    knowledge: Option<Arc<dyn KnowledgeSearchBackend>>,
}

impl Default for UnifiedSearch {
    fn default() -> Self {
        Self { web: None, code: None, knowledge: None }
    }
}

impl UnifiedSearch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: attach a web search backend.
    pub fn with_web(mut self, backend: Arc<dyn WebSearchBackend>) -> Self {
        self.web = Some(backend);
        self
    }

    /// Builder: attach a code search backend.
    pub fn with_code(mut self, backend: Arc<dyn CodeSearchBackend>) -> Self {
        self.code = Some(backend);
        self
    }

    /// Builder: attach a knowledge-base search backend.
    pub fn with_knowledge(mut self, backend: Arc<dyn KnowledgeSearchBackend>) -> Self {
        self.knowledge = Some(backend);
        self
    }

    /// Primary search entry point.
    pub fn search(&self, query: &SearchQuery) -> Vec<UnifiedSearchResult> {
        match query.scope {
            SearchScope::Web => self.search_web(&query.query, query.max_results),
            SearchScope::Code => self.search_code(&query),
            SearchScope::Knowledge => self.search_knowledge(&query.query, query.max_results),
            SearchScope::All => self.fan_out_search(query),
        }
    }

    // ── Single-scope searches ──

    fn search_web(&self, query: &str, max: usize) -> Vec<UnifiedSearchResult> {
        self.web
            .as_ref()
            .map(|b| b.search(query, max))
            .unwrap_or_default()
    }

    fn search_code(&self, query: &SearchQuery) -> Vec<UnifiedSearchResult> {
        let path = query
            .filters
            .get("path")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        self.code
            .as_ref()
            .map(|b| b.search(&query.query, &path, query.max_results))
            .unwrap_or_default()
    }

    fn search_knowledge(&self, query: &str, max: usize) -> Vec<UnifiedSearchResult> {
        self.knowledge
            .as_ref()
            .map(|b| b.search(query, max))
            .unwrap_or_default()
    }

    // ── Parallel fan-out + RRF ──

    /// Fan out to all available backends in parallel and fuse via RRF.
    fn fan_out_search(&self, query: &SearchQuery) -> Vec<UnifiedSearchResult> {
        let per_backend = query.max_results;

        // Collect results from each available backend.
        let mut web_results = Vec::new();
        let mut code_results = Vec::new();
        let mut kb_results = Vec::new();

        if let Some(web) = &self.web {
            web_results = web.search(&query.query, per_backend);
        }
        if let Some(code) = &self.code {
            let path = query
                .filters
                .get("path")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            code_results = code.search(&query.query, &path, per_backend);
        }
        if let Some(kb) = &self.knowledge {
            kb_results = kb.search(&query.query, per_backend);
        }

        // Build ranked-id lists per source for RRF.
        let web_ids: Vec<String> = web_results.iter().map(|r| r.location.clone()).collect();
        let code_ids: Vec<String> = code_results.iter().map(|r| r.location.clone()).collect();
        let kb_ids: Vec<String> = kb_results.iter().map(|r| r.location.clone()).collect();

        // Merge all results keyed by location.
        let mut by_location: HashMap<String, UnifiedSearchResult> = HashMap::new();
        for r in web_results.into_iter().chain(code_results).chain(kb_results) {
            by_location.entry(r.location.clone()).or_insert(r);
        }

        // RRF fusion over ranked lists.
        let lists: Vec<&[String]> = [&web_ids[..], &code_ids[..], &kb_ids[..]]
            .iter()
            .filter(|l| !l.is_empty())
            .copied()
            .collect();

        if lists.is_empty() {
            return Vec::new();
        }

        let rrf_scores = rrf_fuse(&lists, 60.0);

        // Apply RRF scores and sort.
        //
        // ⚠️ 此处曾编译不过（E0382 borrow of moved value）：
        //   下面 `.map(|mut r| … hit_score(&rrf_scores, …) …)` 在
        //   `rrf_scores.into_iter()` **消费**了 `rrf_scores` 的闭包里
        //   又按引用借用它 ⇒ 移动与借用冲突。
        //
        // ✅ 修法：`RrfHit` 是 `Clone`，而 `hit_score` 只需 `&[RrfHit]`。
        //   ⇒ 先把分数查成一张 `HashMap<id, score>`（`hit_score` 本来就是
        //     线性查找，每次 O(n)），闭包里查 map，不再借用已被移动的向量。
        //   这同时把 O(n²) 的重复查找降为 O(n)。
        let rrf_by_id: std::collections::HashMap<String, f64> = rrf_scores
            .iter()
            .map(|h| (h.id.clone(), h.score))
            .collect();
        let mut fused: Vec<UnifiedSearchResult> = rrf_scores
            .into_iter()
            .filter_map(|hit| by_location.remove(&hit.id))
            .map(|mut r| {
                // Blend original score with RRF score.
                let fused_score = rrf_by_id.get(&r.location).copied().unwrap_or(0.0);
                r.score = r.score * 0.3 + fused_score * 0.7;
                r
            })
            .collect();

        fused.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        fused.truncate(query.max_results);
        fused
    }
}

// ════════════════════════════════════════════════════════════════
// RRF helpers
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct RrfHit {
    id: String,
    score: f64,
}

/// Reciprocal Rank Fusion — merge multiple ranked lists into a single
/// score per item.  `k` is the RRF constant (default 60, as used in
/// `nt_core_code_search`).
fn rrf_fuse(lists: &[&[String]], k: f64) -> Vec<RrfHit> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    for list in lists {
        for (rank, id) in list.iter().enumerate() {
            let r = (rank + 1) as f64;
            *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + r);
        }
    }
    let mut hits: Vec<RrfHit> = scores
        .into_iter()
        .map(|(id, score)| RrfHit { id, score })
        .collect();
    hits.sort_by(|a, b| {
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

/// 按 id 取 RRF 分数（未命中记 0）。
///
/// ⚠️ 已无生产调用方：原唯一调用点在 `rrf_fuse` 之后的 `.map()` 闭包里，
///   那里 `rrf_scores` 已被 `into_iter()` 移动 ⇒ 无法再借用（E0382）。
///   现改为调用前先建 `HashMap<id, score>`。
///
/// 保留本函数而非直接删：它是「查不到就记 0」这条语义的**可执行说明**，
/// 而 map 版把这语义写成了一行 `.copied().unwrap_or(0.0)`。
#[allow(dead_code)]
fn hit_score(hits: &[RrfHit], id: &str) -> f64 {
    hits.iter().find(|h| h.id == id).map(|h| h.score).unwrap_or(0.0)
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWeb;
    impl WebSearchBackend for MockWeb {
        fn search(&self, query: &str, max: usize) -> Vec<UnifiedSearchResult> {
            (0..max)
                .map(|i| UnifiedSearchResult {
                    title: format!("Web {} for {}", i, query),
                    location: format!("https://example.com/{}", i),
                    snippet: format!("snippet {}", i),
                    score: 1.0 - i as f64 * 0.1,
                    source: SearchSource::Web,
                    metadata: HashMap::new(),
                })
                .collect()
        }
    }

    struct MockCode;
    impl CodeSearchBackend for MockCode {
        fn search(&self, query: &str, _path: &Path, max: usize) -> Vec<UnifiedSearchResult> {
            (0..max)
                .map(|i| UnifiedSearchResult {
                    title: format!("src/main.rs:{}", i + 1),
                    location: "src/main.rs".to_string(),
                    snippet: format!("// {} line {}", query, i),
                    score: 0.9 - i as f64 * 0.1,
                    source: SearchSource::Code,
                    metadata: HashMap::new(),
                })
                .collect()
        }
    }

    struct MockKB;
    impl KnowledgeSearchBackend for MockKB {
        fn search(&self, query: &str, max: usize) -> Vec<UnifiedSearchResult> {
            (0..max)
                .map(|i| UnifiedSearchResult {
                    title: format!("KB node {}", i),
                    location: format!("node_{}", i),
                    snippet: format!("KB content for {}", query),
                    score: 0.8 - i as f64 * 0.1,
                    source: SearchSource::Knowledge,
                    metadata: HashMap::new(),
                })
                .collect()
        }
    }

    #[test]
    fn test_single_scope_web() {
        let search = UnifiedSearch::new().with_web(Arc::new(MockWeb));
        let q = SearchQuery::web("rust");
        let results = search.search(&q);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.source == SearchSource::Web));
    }

    #[test]
    fn test_single_scope_code() {
        let search = UnifiedSearch::new().with_code(Arc::new(MockCode));
        let q = SearchQuery::code("fn main", ".");
        let results = search.search(&q);
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.source == SearchSource::Code));
    }

    #[test]
    fn test_fan_out_rrf() {
        let search = UnifiedSearch::new()
            .with_web(Arc::new(MockWeb))
            .with_code(Arc::new(MockCode))
            .with_knowledge(Arc::new(MockKB));
        let q = SearchQuery::all("test");
        let results = search.search(&q);
        assert!(!results.is_empty());
        // All three sources should appear.
        let has_web = results.iter().any(|r| r.source == SearchSource::Web);
        let has_code = results.iter().any(|r| r.source == SearchSource::Code);
        let has_kb = results.iter().any(|r| r.source == SearchSource::Knowledge);
        assert!(has_web);
        assert!(has_code);
        assert!(has_kb);
    }

    #[test]
    fn test_rrf_fuse_basic() {
        let a = vec!["x".into(), "y".into(), "z".into()];
        let b = vec!["y".into(), "z".into(), "w".into()];
        let lists: Vec<&[String]> = vec![&a, &b];
        let fused = rrf_fuse(&lists, 60.0);
        // "y" and "z" appear in both lists → higher score.
        let y_score = fused.iter().find(|h| h.id == "y").unwrap().score;
        let x_score = fused.iter().find(|h| h.id == "x").unwrap().score;
        assert!(y_score > x_score);
    }

    #[test]
    fn test_missing_backend_returns_empty() {
        let search = UnifiedSearch::new(); // no backends attached
        let q = SearchQuery::all("anything");
        let results = search.search(&q);
        assert!(results.is_empty());
    }

    #[test]
    fn test_max_results_respected() {
        let search = UnifiedSearch::new().with_web(Arc::new(MockWeb));
        let mut q = SearchQuery::web("test");
        q.max_results = 3;
        let results = search.search(&q);
        assert_eq!(results.len(), 3);
    }
}
