pub mod nt_memory_types;
pub mod nt_memory_store;
pub mod nt_memory_search;
pub mod nt_memory_graph;
pub mod nt_memory_crawl;
pub mod nt_memory_seed;
pub mod nt_memory_integration;
pub mod nt_memory_gwtq;
pub mod nt_memory_embed;
pub mod nt_memory_ingest;
pub mod nt_memory_api;
pub mod vector_adapter;
mod nt_memory_schema;

use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use rusqlite::Connection;

use crate::neotrix::nt_mind::bm25::{Bm25Document, Bm25Index};

pub use nt_memory_types::*;

pub struct KnowledgeBase {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
    bm25: RwLock<Option<Bm25Index>>,
    bm25_dirty: RwLock<bool>,
    pub embedding_config: RwLock<Option<nt_memory_embed::EmbeddingConfig>>,
}

impl KnowledgeBase {
    pub fn open(path: Option<PathBuf>) -> Result<Self, String> {
        let db_path = path.unwrap_or_else(|| {
            let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push(".neotrix");
            std::fs::create_dir_all(&p).ok();
            p.push("knowledge.db");
            p
        });

        let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open DB: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {}", e))?;

        nt_memory_schema::initialize(&conn).map_err(|e| format!("Failed to init schema: {}", e))?;

        Ok(KnowledgeBase {
            conn: Mutex::new(conn),
            db_path,
            bm25: RwLock::new(None),
            bm25_dirty: RwLock::new(true),
            embedding_config: RwLock::new(None),
        })
    }

    pub fn insert_node(&self, node: &KnowledgeNode) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::insert_node(&conn, node).map_err(|e| format!("Insert error: {}", e))?;
        self.mark_bm25_dirty();
        Ok(())
    }

    pub fn get_node(&self, id: &str) -> Result<Option<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::get_node(&conn, id).map_err(|e| format!("Get error: {}", e))
    }

    pub fn get_node_history(&self, id: &str) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::get_node_history(&conn, id).map_err(|e| format!("History error: {}", e))
    }

    pub fn find_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::find_node_by_url(&conn, url).map_err(|e| format!("Find error: {}", e))
    }

    pub fn update_node(&self, node: &KnowledgeNode) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::update_node(&conn, node).map_err(|e| format!("Update error: {}", e))
    }

    pub fn delete_node(&self, id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_store::delete_node(&conn, id).map_err(|e| format!("Delete error: {}", e))?;
        if r { self.mark_bm25_dirty(); }
        Ok(r)
    }

    pub fn insert_edge(&self, edge: &KnowledgeEdge) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::insert_edge(&conn, edge).map_err(|e| format!("Edge insert error: {}", e))
    }

    pub fn insert_or_get_node(&self, title: &str, node_type: NodeType, summary: Option<&str>, url: Option<&str>, domain: Option<&str>) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let id = nt_memory_store::insert_or_get_node(&conn, title, node_type, summary, url, domain)
            .map_err(|e| format!("Insert/get error: {}", e))?;
        self.mark_bm25_dirty();
        Ok(id)
    }

    pub fn upsert_edge(&self, source_id: &str, target_id: &str, relation_type: RelationType, weight: f64, description: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::upsert_edge(&conn, source_id, target_id, relation_type, weight, description)
            .map_err(|e| format!("Edge upsert error: {}", e))
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut results = nt_memory_search::hybrid_search(&conn, query, limit)
            .map_err(|e| format!("Search error: {}", e))?;

        if results.len() < limit {
            drop(conn);
            self.rebuild_bm25()?;
            let bm25_results = {
                let cache = self.bm25.read().map_err(|e| format!("Lock error: {}", e))?;
                cache.as_ref().map(|bm25| bm25.search(query, limit)).unwrap_or_default()
            };
            let existing: std::collections::HashSet<String> =
                results.iter().map(|r| r.node.id.clone()).collect();
            for (score, id) in &bm25_results {
                if existing.contains(id) { continue; }
                let conn2 = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
                if let Ok(Some(node)) = nt_memory_store::get_node(&conn2, id) {
                    drop(conn2);
                    results.push(SearchResult {
                        node,
                        score: *score,
                        matched_on: vec![SearchMatchType::Bm25],
                    });
                }
                if results.len() >= limit * 2 { break; }
            }
        }

        results.truncate(limit);
        Ok(results)
    }

    pub fn search_by_type(&self, node_type: &NodeType, limit: usize) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_search::search_by_type(&conn, node_type, limit).map_err(|e| format!("Search error: {}", e))
    }

    pub fn get_related(&self, node_id: &str, relation_type: Option<&str>, limit: usize) -> Result<Vec<SearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_search::get_related(&conn, node_id, relation_type, limit).map_err(|e| format!("Related error: {}", e))
    }

    pub fn get_edges(&self, node_id: &str) -> Result<Vec<KnowledgeEdge>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::get_edges_for_node(&conn, node_id).map_err(|e| format!("Edges error: {}", e))
    }

    pub fn shortest_path(&self, from_id: &str, to_id: &str, max_depth: usize) -> Result<Option<GraphPath>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_graph::shortest_path(&conn, from_id, to_id, max_depth).map_err(|e| format!("Path error: {}", e))
    }

    pub fn subgraph(&self, center_id: &str, depth: usize) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_graph::subgraph(&conn, center_id, depth).map_err(|e| format!("Subgraph error: {}", e))
    }

    pub fn stats(&self) -> Result<KnowledgeStats, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_store::get_stats(&conn).map_err(|e| format!("Stats error: {}", e))
    }

    pub fn enqueue_seed_urls(&self, urls: &[(&str, i64, &str)]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::enqueue_seed_urls(&conn, urls).map_err(|e| format!("Enqueue error: {}", e))?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn discover_wiki_category(&self, category: &str, max_pages: usize, max_depth: u32, enqueue: bool) -> Result<(usize, usize, usize), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::discover_wiki_category_members(&conn, category, max_pages, max_depth, enqueue)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn enqueue_search_results(&self, query: &str, max_results: usize, priority: i64) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::enqueue_search_results_from_engine(&conn, query, max_results, priority)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn run_crawl_cycle(&self, max_items: usize) -> Result<nt_memory_crawl::CrawlCycleReport, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::run_crawl_cycle(&conn, max_items);
        self.mark_bm25_dirty();
        r
    }

    pub fn reset_stuck_items(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_crawl::reset_stuck_items(&conn)
    }

    pub fn purge_skip_domains(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_crawl::purge_skip_domains(&conn)
    }

    pub fn purge_all_skip_patterns(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        nt_memory_crawl::purge_all_skip_patterns(&conn)
    }

    pub fn validate_urls(&self, num_workers: usize) -> Result<(usize, usize), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let db_path = self.db_path.to_string_lossy().to_string();
        nt_memory_crawl::validate_urls_parallel(&conn, &db_path, num_workers)
    }

    pub fn run_crawl_cycle_parallel(&self, max_items: usize, num_workers: usize, fetch_links: bool) -> Result<nt_memory_crawl::CrawlCycleReport, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let db_path = self.db_path.to_string_lossy().to_string();
        let r = nt_memory_crawl::run_crawl_cycle_parallel(&conn, &db_path, max_items, num_workers, fetch_links);
        self.mark_bm25_dirty();
        r
    }

    pub fn seed_foundational(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_seed::seed_foundational_knowledge(&conn).map_err(|e| format!("Seed error: {}", e))?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn ingest_wikipedia(&self, topic: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::ingest_from_wikipedia(&conn, topic)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn ingest_arxiv(&self, arxiv_id: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::ingest_from_arxiv(&conn, arxiv_id)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn ingest_github(&self, owner: &str, repo: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::ingest_from_github(&conn, owner, repo)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn persist_mined(&self, title: &str, summary: &str, url: &str, source_type: &str, confidence: f64, edits: &[(String, f64)], insights: &[String]) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_integration::persist_mined_knowledge(&conn, title, summary, url, source_type, confidence, edits, insights);
        self.mark_bm25_dirty();
        r
    }

    pub fn import_knowledge_engine(&self, entries: &[super::nt_mind::knowledge_engine::KnowledgeEntry], relations: &[super::nt_mind::knowledge_engine::KnowledgeRelation]) -> Result<(usize, usize), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_integration::import_from_knowledge_engine(&conn, entries, relations);
        self.mark_bm25_dirty();
        r
    }

    pub fn ingest_openlibrary_search(&self, query: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::ingest_from_openlibrary_search(&conn, query)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn ingest_github_search(&self, query: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_crawl::ingest_github_search(&conn, query)?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    pub fn dedup_nodes(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let r = nt_memory_store::dedup_nodes(&conn).map_err(|e| format!("Dedup error: {}", e))?;
        self.mark_bm25_dirty();
        Ok(r)
    }

    fn mark_bm25_dirty(&self) {
        if let Ok(mut dirty) = self.bm25_dirty.write() {
            *dirty = true;
        }
    }

    fn rebuild_bm25(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let nodes = nt_memory_store::get_all_nodes(&conn).map_err(|e| format!("Rebuild BM25 error: {}", e))?;
        let docs: Vec<Bm25Document> = nodes.iter().map(|n| {
            let text = format!("{} {} {}",
                n.title,
                n.summary.as_deref().unwrap_or(""),
                n.content.as_deref().unwrap_or(""));
            Bm25Document { id: n.id.clone(), text }
        }).collect();
        let index = Bm25Index::build(&docs);
        if let Ok(mut cache) = self.bm25.write() {
            *cache = Some(index);
        }
        if let Ok(mut dirty) = self.bm25_dirty.write() {
            *dirty = false;
        }
        Ok(())
    }

    /// Set the embedding config. Call before using semantic_search.
    pub fn with_embedding(&self, config: nt_memory_embed::EmbeddingConfig) {
        if let Ok(mut cfg) = self.embedding_config.write() {
            *cfg = Some(config);
        }
    }

    /// Ensure all nodes have embeddings. Scans for missing ones and generates them via the API.
    /// DB lock is only held briefly per read/write — never across API calls.
    pub fn ensure_embeddings(&self) -> Result<usize, String> {
        let config = {
            let r = self.embedding_config.read().map_err(|e| format!("Lock: {}", e))?;
            r.clone()
        };
        let config = config.ok_or_else(|| "Embedding config not set. Call with_embedding() first.".to_string())?;

        // Phase 1: collect pending nodes (brief lock)
        let pending: Vec<(String, String)> = {
            let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
            let missing = nt_memory_embed::find_nodes_missing_embeddings(&conn)
                .map_err(|e| format!("DB: {}", e))?;
            if missing.is_empty() { return Ok(0); }
            missing.iter().filter_map(|id| {
                let node = nt_memory_store::get_node(&conn, id).ok()??;
                Some((id.clone(), nt_memory_embed::build_node_text(
                    &node.title, node.summary.as_deref(), node.content.as_deref(),
                )))
            }).collect()
            // conn lock dropped here
        };

        // Phase 2: batch-embed all texts (no DB lock held)
        let texts: Vec<&str> = pending.iter().map(|(_, t)| t.as_str()).collect();
        let results = match nt_memory_embed::embed_text_batch(&config, &texts) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[KB] batch embed failed, trying single requests: {}", e);
                // fallback: one-by-one
                let mut vecs = Vec::with_capacity(pending.len());
                for (id, text) in &pending {
                    match nt_memory_embed::embed_text(&config, text) {
                        Ok(v) => vecs.push(v),
                        Err(e2) => {
                            log::warn!("[KB] embed failed for {}: {}", id, e2);
                            vecs.push(Vec::new());
                        }
                    }
                }
                vecs
            }
        };

        // Phase 3: store results (brief lock)
        let mut total = 0usize;
        {
            let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
            for ((id, _), embedding) in pending.iter().zip(results.iter()) {
                if !embedding.is_empty() {
                    if nt_memory_embed::store_embedding(&conn, id, embedding, &config.model).is_ok() {
                        total += 1;
                    }
                }
            }
        }
        Ok(total)
    }

    /// Semantic search: embed the query, then find the nearest neighbors by cosine similarity.
    /// Returns results scored by similarity (higher = more relevant).
    pub fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let config = {
            let r = self.embedding_config.read().map_err(|e| format!("Lock: {}", e))?;
            r.clone()
        };
        let config = config.ok_or_else(|| "Embedding config not set".to_string())?;

        let query_vec = nt_memory_embed::embed_text(&config, query)?;

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let all = nt_memory_embed::load_all_embeddings(&conn)
            .map_err(|e| format!("DB: {}", e))?;

        let mut scored: Vec<(f64, String)> = all.iter()
            .map(|(id, vec)| (nt_memory_embed::cosine_similarity(&query_vec, vec), id.clone()))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        let mut results = Vec::with_capacity(scored.len());
        for (score, id) in &scored {
            if let Ok(Some(node)) = nt_memory_store::get_node(&conn, id) {
                results.push(SearchResult {
                    node,
                    score: *score,
                    matched_on: vec![SearchMatchType::VectorSimilarity],
                });
            }
        }
        Ok(results)
    }

    /// Hybrid search: FTS5 recall top-3N → embedding cosine rerank → final top-N.
    /// Falls back to pure FTS5/BM25 when embedding config is not set.
    pub fn hybrid_rerank_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let has_embeddings = {
            let r = self.embedding_config.read().map_err(|e| format!("Lock: {}", e))?;
            r.is_some()
        };
        if !has_embeddings {
            return self.search(query, limit);
        }

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // 1. FTS5 recall top-3N
        let fts_results = nt_memory_search::search_fts(&conn, query, limit * 3)
            .map_err(|e| format!("FTS error: {}", e))?;
        if fts_results.is_empty() {
            drop(conn);
            return self.search(query, limit);
        }

        // 2. Load all embeddings
        let all_embeddings = nt_memory_embed::load_all_embeddings(&conn)
            .map_err(|e| format!("DB: {}", e))?;
        drop(conn);

        let config = {
            let r = self.embedding_config.read().map_err(|e| format!("Lock: {}", e))?;
            r.clone().unwrap()
        };

        // 3. Embed the query
        let query_vec = match nt_memory_embed::embed_text(&config, query) {
            Ok(v) => v,
            Err(_) => {
                // Embedding API failed, fall back to plain FTS5
                let mut r = fts_results;
                r.truncate(limit);
                return Ok(r);
            }
        };

        // 4. Build embedding map for fast lookup
        let emb_map: std::collections::HashMap<&str, &[f32]> = all_embeddings.iter()
            .map(|(id, v)| (id.as_str(), v.as_slice()))
            .collect();

        // 5. Rerank FTS results by weighted score: 0.3 × FTS + 0.7 × cosine
        let mut hybrid: Vec<(f64, SearchResult)> = fts_results.into_iter()
            .map(|r| {
                let cos = emb_map.get(r.node.id.as_str())
                    .map(|v| nt_memory_embed::cosine_similarity(&query_vec, v))
                    .unwrap_or(0.2);
                let combined = 0.3 * r.score + 0.7 * cos;
                (combined, r)
            })
            .collect();
        hybrid.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        hybrid.truncate(limit);

        let mut results: Vec<SearchResult> = hybrid.into_iter().map(|(_, r)| r).collect();
        for r in &mut results {
            r.matched_on.push(SearchMatchType::VectorSimilarity);
        }
        Ok(results)
    }

    /// Batch hybrid search: embed all queries at once, then rerank FTS results for each.
    pub fn hybrid_rerank_search_batch(
        &self, queries: &[&str], limit: usize,
    ) -> Result<Vec<Vec<SearchResult>>, String> {
        if queries.is_empty() { return Ok(Vec::new()); }

        let config = {
            let r = self.embedding_config.read().map_err(|e| format!("Lock: {}", e))?;
            r.clone()
        };
        let config = match config {
            Some(c) => c,
            None => {
                return queries.iter().map(|q| self.search(q, limit)).collect();
            }
        };

        // 1. Batch embed all query texts in one API call
        let query_vecs = match nt_memory_embed::embed_text_batch(&config, queries) {
            Ok(v) => v,
            Err(_) => {
                return queries.iter().map(|q| self.search(q, limit)).collect();
            }
        };

        // 2. Load embeddings once
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let all_embeddings = nt_memory_embed::load_all_embeddings(&conn)
            .map_err(|e| format!("DB: {}", e))?;
        drop(conn);

        let emb_map: std::collections::HashMap<&str, &[f32]> = all_embeddings.iter()
            .map(|(id, v)| (id.as_str(), v.as_slice()))
            .collect();

        // 3. For each query, run FTS5 + cosine rerank
        let mut all_results = Vec::with_capacity(queries.len());
        for (i, query_vec) in query_vecs.iter().enumerate() {
            let q = queries[i];
            let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
            let fts_results = nt_memory_search::search_fts(&conn, q, limit * 3)
                .unwrap_or_default();
            drop(conn);

            if fts_results.is_empty() {
                all_results.push(Vec::new());
                continue;
            }

            let mut hybrid: Vec<(f64, SearchResult)> = fts_results.into_iter()
                .map(|r| {
                    let cos = emb_map.get(r.node.id.as_str())
                        .map(|v| nt_memory_embed::cosine_similarity(query_vec, v))
                        .unwrap_or(0.2);
                    (0.3 * r.score + 0.7 * cos, r)
                })
                .collect();
            hybrid.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            hybrid.truncate(limit);

            let mut results: Vec<SearchResult> = hybrid.into_iter().map(|(_, r)| r).collect();
            for r in &mut results {
                r.matched_on.push(SearchMatchType::VectorSimilarity);
            }
            all_results.push(results);
        }

        Ok(all_results)
    }

    /// Store a conversation record as an Insight node for evolution training.
    /// Returns the node ID.
    pub fn store_conversation_record(&self, record: &nt_memory_types::ConversationRecord) -> Result<String, String> {
        let json = serde_json::to_string(record).map_err(|e| format!("Serialize: {}", e))?;
        let summary = format!("[Evolution] {} — {} — effectiveness: {:.2}",
            record.task_description, record.outcome, record.effectiveness);
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let id = nt_memory_store::insert_or_get_node(
            &conn, &format!("EVO:{}", record.id), NodeType::Insight,
            Some(&summary), None, Some("conversation_evolution"),
        ).map_err(|e| format!("Insert node: {}", e))?;
        // Store full record as metadata
        conn.execute(
            "UPDATE nodes SET metadata = ?1 WHERE id = ?2",
            rusqlite::params![json, id],
        ).map_err(|e| format!("Update metadata: {}", e))?;
        self.mark_bm25_dirty();
        Ok(id)
    }

    /// Store an evolution record (pattern extracted from conversation).
    pub fn store_evolution_record(&self, record: &nt_memory_types::EvolutionRecord) -> Result<String, String> {
        let json = serde_json::to_string(record).map_err(|e| format!("Serialize: {}", e))?;
        let summary = format!("[{:?}] {} — gain: {:.2}", record.pattern_type, record.description, record.effectiveness_gain);
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let id = nt_memory_store::insert_or_get_node(
            &conn, &format!("EVO_PATTERN:{}", record.id), NodeType::Insight,
            Some(&summary), None, Some("evolution_pattern"),
        ).map_err(|e| format!("Insert node: {}", e))?;
        conn.execute(
            "UPDATE nodes SET metadata = ?1 WHERE id = ?2",
            rusqlite::params![json, id],
        ).map_err(|e| format!("Update metadata: {}", e))?;
        self.mark_bm25_dirty();
        Ok(id)
    }

    /// Query recent evolution records for meta-cognitive analysis.
    pub fn get_evolution_history(&self, limit: usize) -> Result<Vec<nt_memory_types::ConversationRecord>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT metadata FROM nodes WHERE domain = 'conversation_evolution' ORDER BY updated_at DESC LIMIT ?1"
        ).map_err(|e| format!("Prepare: {}", e))?;
        let results = stmt.query_map(rusqlite::params![limit as i64], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| format!("Query: {}", e))?;
        let mut records = Vec::new();
        for row in results {
            if let Ok(json_str) = row {
                if let Ok(rec) = serde_json::from_str::<nt_memory_types::ConversationRecord>(&json_str) {
                    records.push(rec);
                }
            }
        }
        Ok(records)
    }

    /// Query recent evolution pattern records for meta-cognitive analysis.
    pub fn get_evolution_patterns(&self, limit: usize) -> Result<Vec<nt_memory_types::EvolutionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT metadata FROM nodes WHERE domain = 'evolution_pattern' ORDER BY updated_at DESC LIMIT ?1"
        ).map_err(|e| format!("Prepare: {}", e))?;
        let results = stmt.query_map(rusqlite::params![limit as i64], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| format!("Query: {}", e))?;
        let mut records = Vec::new();
        for row in results {
            if let Ok(json_str) = row {
                if let Ok(rec) = serde_json::from_str::<nt_memory_types::EvolutionRecord>(&json_str) {
                    records.push(rec);
                }
            }
        }
        Ok(records)
    }

    /// Distill a session (messages) into a ConversationRecord and store it.
    /// Accepts messages as (role, content) pairs for loose coupling with TUI types.
    pub fn distill_session(&self, session_id: &str, name: &str, messages: &[(String, String)]) -> Result<String, String> {
        let user_msgs: Vec<&str> = messages.iter().filter(|(r, _)| r == "user").map(|(_, c)| c.as_str()).collect();
        let assistant_msgs: Vec<&str> = messages.iter().filter(|(r, _)| r == "assistant").map(|(_, c)| c.as_str()).collect();
        let user_intent = user_msgs.first().unwrap_or(&"").chars().take(200).collect::<String>();
        let last_response = assistant_msgs.last().map(|s| s.to_string()).unwrap_or_default();

        let outcome = if last_response.contains("error") || last_response.contains("failed") {
            "failure".to_string()
        } else if assistant_msgs.is_empty() {
            "no_response".to_string()
        } else {
            "success".to_string()
        };

        let analysis = format!("Session '{}' with {} user messages, {} assistant messages",
            name, user_msgs.len(), assistant_msgs.len());

        let record = nt_memory_types::ConversationRecord {
            id: session_id.to_string(),
            session_id: session_id.to_string(),
            task_description: analysis,
            user_intent,
            strategy_used: "auto".to_string(),
            e8_mode: "core_first".to_string(),
            specialist_winner: "default".to_string(),
            actions_taken: vec![],
            obstacles_encountered: vec![],
            fix_patterns: vec![],
            outcome,
            effectiveness: if !assistant_msgs.is_empty() { 0.5 } else { 0.0 },
            reasoning_iterations: 0,
            error_count: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0),
        };
        self.store_conversation_record(&record)
    }

    pub fn close(self) -> Result<(), String> {
        let conn = self.conn.into_inner().map_err(|e| format!("Mutex error: {}", e))?;
        conn.close().map_err(|(_conn, e)| format!("Close error: {}", e))?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
