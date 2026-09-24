//! kb_ingest — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_discovery_github_topics;
use super::nt_discovery_orchestrator;
use super::nt_discovery_sources;
use super::nt_memory_crawl;
use super::nt_memory_embed;
use super::nt_memory_knowledge_assets;
use super::nt_memory_seed;
use super::nt_memory_store;
use super::nt_memory_wiki;
use super::{CrawlCycleReport, DiscoveryCycleConfig, DiscoveryCycleReport, DiscoveryPipelineConfig, GithubDiscoveryStats};

impl KnowledgeBase {
    pub fn seed_foundational(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_seed::seed_foundational_knowledge(&conn).map_err(|e| format!("seed: {}", e))
    }

    // ── Wiki ──

    pub fn wiki_sync(
        &self,
        dir: &std::path::Path,
        prefix: &str,
    ) -> Result<nt_memory_wiki::WikiSyncReport, String> {
        nt_memory_wiki::sync_directory(self, dir, prefix)
    }

    pub fn wiki_graph_html(&self) -> Result<String, String> {
        nt_memory_wiki::generate_graph_html(self)
    }

    pub fn wiki_query(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<nt_memory_wiki::WikiSearchResult>, String> {
        nt_memory_wiki::query(self, query, limit)
    }

    // ── Knowledge Assets ──

    pub fn import_knowledge_assets(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_knowledge_assets(self, path)
    }

    pub fn import_review_findings(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_review_findings(self, path)
    }

    pub fn import_brain_state(
        &self,
        base_path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_brain_state(self, base_path)
    }

    pub fn import_absorption_report(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_absorption_report(self, path)
    }

    pub fn import_knowledge_engine(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_knowledge_engine(self, path)
    }

    pub fn import_reasoning_memories(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_reasoning_memories(self, path)
    }

    pub fn import_bandit_data(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_bandit_data(self, path)
    }

    pub fn import_e8_state(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_e8_state(self, path)
    }

    pub fn import_avatar_chain(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_avatar_chain(self, path)
    }

    pub fn import_proxy_pool(
        &self,
        path: &std::path::Path,
    ) -> Result<nt_memory_knowledge_assets::ImportReport, String> {
        nt_memory_knowledge_assets::import_proxy_pool(self, path)
    }

    // ── Embeddings ──

    pub fn ensure_embeddings(&self) -> Result<usize, String> {
        let config = self
            .embedding_config
            .read()
            .map_err(|e| format!("embedding_config read: {}", e))?
            .clone();
        let config = match config {
            Some(c) => c,
            None => return Ok(0),
        };
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let missing = nt_memory_embed::find_nodes_missing_embeddings(&conn)
            .map_err(|e| format!("find_missing: {}", e))?;
        let count = missing.len();
        let mut local_fallback = 0usize;
        let mut http_err: Option<String> = None;
        for node_id in &missing {
            if let Ok(Some(node)) = nt_memory_store::get_node(&conn, node_id) {
                let text = nt_memory_embed::build_node_text(
                    &node.title,
                    node.summary.as_deref(),
                    node.content.as_deref(),
                );
                // Http 模式失败 → 自动降级本地 hash-kernel (Cycle 207 R-P79:
                // embedding 链路不依赖外部 server, 保证零依赖可跑)。
                if config.mode == nt_memory_embed::EmbedMode::Http {
                    match nt_memory_embed::embed_text(&config, &text) {
                        Ok(vec) => {
                            if let Err(e) = nt_memory_embed::store_embedding(
                                &conn,
                                node_id,
                                &vec,
                                &config.model,
                            ) {
                                log::warn!("[KB] store embedding for {}: {}", node_id, e);
                            }
                            continue;
                        }
                        Err(e) => {
                            if http_err.is_none() {
                                http_err = Some(e);
                            }
                            let local_cfg = nt_memory_embed::EmbeddingConfig {
                                mode: nt_memory_embed::EmbedMode::Local,
                                ..config.clone()
                            };
                            if let Ok(vec) = nt_memory_embed::embed_text(&local_cfg, &text) {
                                if nt_memory_embed::store_embedding(
                                    &conn,
                                    node_id,
                                    &vec,
                                    "hash-kernel-384",
                                )
                                .is_ok()
                                {
                                    local_fallback += 1;
                                }
                            }
                        }
                    }
                } else if let Ok(vec) = nt_memory_embed::embed_text(&config, &text) {
                    if let Err(e) =
                        nt_memory_embed::store_embedding(&conn, node_id, &vec, &config.model)
                    {
                        log::warn!("[KB] store embedding for {}: {}", node_id, e);
                    }
                }
            }
        }
        if let Some(e) = http_err {
            log::warn!(
                "[KB] ensure_embeddings: embedding API unavailable ({}); fell back to local hash-kernel for {} nodes",
                e,
                local_fallback
            );
        }
        Ok(count)
    }

    // ── Crawl / Ingest ──

    pub fn enqueue_seed_urls(&self, urls: &[(&str, i64, &str)]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::enqueue_seed_urls(&conn, urls)
            .map_err(|e| format!("enqueue_seed_urls: {}", e))
    }

    pub fn run_crawl_cycle(&self, max_items: usize) -> Result<CrawlCycleReport, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::run_crawl_cycle(&conn, max_items)
    }

    pub fn ingest_wikipedia(&self, topic: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::ingest_from_wikipedia(&conn, topic)
    }

    pub fn ingest_arxiv(&self, id: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::ingest_from_arxiv(&conn, id)
    }

    pub fn ingest_alphaxiv_feed(
        &self,
        pages: usize,
        page_size: usize,
        categories: &str,
    ) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::ingest_from_alphaxiv_feed(&conn, pages, page_size, categories)
    }

    pub fn ingest_github(&self, owner: &str, repo: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::ingest_from_github(&conn, owner, repo)
    }

    pub fn ingest_hf_dataset(&self, dataset_ref: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::ingest_from_hf_dataset(&conn, dataset_ref)
    }

    pub fn run_hf_queue_batch(&self, max_items: usize) -> Result<(usize, usize), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_crawl::run_hf_queue_batch(&conn, max_items)
    }

    // ── Discovery (GitHub Topics / External Sources) ──

    pub fn run_github_topics_discovery(
        &self,
        config: &DiscoveryPipelineConfig,
    ) -> Result<GithubDiscoveryStats, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_discovery_github_topics::run_github_topics_discovery(&conn, config)
    }

    pub fn run_discovery_cycle(&self, config: &DiscoveryCycleConfig) -> DiscoveryCycleReport {
        match self.conn.lock() {
            Ok(conn) => nt_discovery_orchestrator::run_discovery_cycle(&conn, config),
            Err(e) => {
                let mut report = DiscoveryCycleReport::default();
                report.errors.push(("lock".into(), format!("Mutex: {}", e)));
                report
            }
        }
    }

    /// 外部知识自动获取 — 按任务摘要调度 discover_* 源 (Semantic Scholar / ArXiv /
    /// 技术文档) 摄入 KB。全部源失败不 panic, 返回成功摄入数。D3: 该编排原驻留
    /// NT-CORE consciousness_core, 下沉至 NT-MEMORY (KB 持 conn 的域能力)。
    pub fn acquire_external_sources(&self, query: &str) -> usize {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return 0,
        };
        let mut ingested = 0usize;
        if let Ok(s) = nt_discovery_sources::discover_semantic_scholar(&conn, query, 5) {
            ingested += s.resources_ingested;
        }
        if let Ok(s) = nt_discovery_sources::discover_arxiv_papers(&conn, query, 5) {
            ingested += s.resources_ingested;
        }
        if let Ok(s) = nt_discovery_sources::discover_technical_docs(&conn, query) {
            ingested += s.resources_ingested;
        }
        ingested
    }

    // ── Unified Store (KV / Config / Secrets / etc.) ──
}
