//! kb_core — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。

use rusqlite::Connection;
use std::path::PathBuf;

use super::KnowledgeBase;
use super::nt_absorb_mapper;
use super::nt_memory_adaptive_rag;
use super::nt_memory_agent_driven;
use super::nt_memory_crawl;
use super::nt_memory_graph_cache;
use super::nt_memory_knowledge_assets;
use super::nt_memory_lifecycle;
use super::nt_memory_schema;
use super::nt_memory_search;
use super::nt_memory_store;
use super::{AdaptiveRetrieval, AgentMemory, AgentSessionManager, CommunityAwareSearch, CommunityDetector, ConfidenceStore, CrawlCycleReport, DecayConfig, EmbeddingCommitmentStore, EmbeddingConfig, FeedbackStore, GwtRouter, GwtRouterConfig, MemoryProficiency, PrivacyConfig, PrivacyEnforcer, SvafGate, TechProfile, TechReserveEntry, TechReserveStore, TemporalFactLedger, VsaAssociativeExpander};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};
use std::fs::{File, OpenOptions};
use std::num::NonZeroUsize;
use lru::LruCache;
use fs2::FileExt;
use log::warn;
use super::bm25;
use log::error;

impl KnowledgeBase {
    /// Shared field initialization for `open()` and `_from_conn()`.
    pub(crate) fn init_fields(
        conn: Connection,
        db_path: PathBuf,
        db_file: Option<File>,
        bm25_dirty: bool,
    ) -> Self {
        let commitment_store = EmbeddingCommitmentStore::new(10000, None);
        let confidence_store = ConfidenceStore::new(DecayConfig::default());
        let community_search = CommunityAwareSearch::new(CommunityDetector::default());
        let privacy = PrivacyEnforcer::new(PrivacyConfig::default());
        let temporal_ledger = TemporalFactLedger::open(Some(&db_path)).unwrap_or_else(|e| {
            log::warn!(
                "[KB] temporal ledger open failed ({}), using isolated in-memory ledger",
                e
            );
            TemporalFactLedger::open(Some(std::path::Path::new(":memory:")))
                .expect("in-memory temporal ledger")
        });
        Self {
            conn: Mutex::new(conn),
            db_path,
            db_file,
            file_lock_held: AtomicBool::new(false),
            bm25: RwLock::new(None),
            bm25_dirty: RwLock::new(bm25_dirty),
            embedding_config: RwLock::new(None),
            fused_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(100).expect("non-zero cache capacity"),
            )),
            adaptive: AdaptiveRetrieval::new(nt_memory_adaptive_rag::AdaptiveRagConfig::default()),
            commitment_store: RwLock::new(commitment_store),
            confidence_store: RwLock::new(confidence_store),
            community_search: RwLock::new(community_search),
            privacy: RwLock::new(privacy),
            vector_adapter: RwLock::new(None),
            agent_memory: RwLock::new(AgentMemory::new(
                nt_memory_agent_driven::MemoryConfig::default(),
            )),
            agent_session: RwLock::new(false),
            svaf_gate: RwLock::new(SvafGate::default()),
            proficiency: RwLock::new(MemoryProficiency::new()),
            graphrag_store: RwLock::new(None),
            tech_reserve: RwLock::new(TechReserveStore::new()),
            skills_library: RwLock::new(nt_memory_knowledge_assets::SkillsLibrary::new()),
            graph_cache: RwLock::new(nt_memory_graph_cache::GraphCache::empty()),
            feedback_store: RwLock::new(FeedbackStore::new(0.05)),
            gwt_router: RwLock::new(GwtRouter::new(GwtRouterConfig::default())),
            vsa_expander: RwLock::new(VsaAssociativeExpander::default()),
            retrieval_evolver: RwLock::new(nt_memory_search::RetrievalEvolver::new()),
            temporal_ledger: Mutex::new(temporal_ledger),
            lifecycle: RwLock::new(nt_memory_lifecycle::MemoryLifecycle::default()),
            absorb_scanner: RwLock::new(None),
            receipt_emitter: RwLock::new(None),
        }
    }

    pub fn open(path: Option<PathBuf>) -> Result<Self, String> {
        let db_path = path.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".neotrix").join("knowledge.db")
        });
        // 2026-09-27 根因修复：拦截 SQLite 惯用哨兵 ":memory:"。
        // 此前 `Connection::open(PathBuf::from(":memory:"))` 会在磁盘上**真的**创建
        // 名为 `:memory:` 的库文件，随后下面 `with_extension("lock")` 的 flock 侧车
        // 再以 `create(true)` 造出 `:memory:.lock` —— 两者都落在进程 CWD（实测即
        // 仓库根，0 字节 `:memory:.lock` 生成于 2026-09-27 22:22）。
        // 全仓 9 处测试用 `Some(PathBuf::from(":memory:"))` 表达"内存库"，这是 Rust
        // 生态惯例写法，故在被调用方拦截：一处修好全部调用点。
        // 同族 `TemporalFactLedger::open`(nt_temporal_facts.rs:63) 已有同样处理，
        // 此处对齐之，保持两处写法一致。内存库无跨进程共享，flock 侧车本无意义。
        let is_memory = db_path.to_string_lossy() == ":memory:";
        let conn = if is_memory {
            Connection::open_in_memory().map_err(|e| format!("Failed to open in-memory KB: {}", e))?
        } else {
            Connection::open(&db_path).map_err(|e| format!("Failed to open KB: {}", e))?
        };
        nt_memory_schema::initialize(&conn)
            .map_err(|e| format!("Failed to initialize KB: {}", e))?;
        if is_memory {
            return Ok(Self::init_fields(conn, db_path, None, true));
        }
        // 锁侧车文件（2026-09-25 根因修复）：flock 绝不能直接下在 sqlite 库文件上。
        // macOS 上 WAL 模式的 sqlite 持有与 flock 互斥的锁（`kb_flocktest` S5 实证：
        // WAL-idle 下 try_lock 必败 WouldBlock），open 期 try_lock 失败 → 写前阻塞锁
        // 等自己，构成进程内自死锁（`kb_probe` v2 stage A + sample 2667 帧全卡 flock 实锤）。
        // 侧车 `<db>.lock` 仅 flock 用户可见、sqlite 永不触碰：跨进程互斥语义不变，自冲突归零。
        let lock_path = db_path.with_extension("lock");
        let db_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
            .ok();
        let mut locked = false;
        if let Some(ref f) = db_file {
            match f.try_lock_exclusive() {
                Ok(()) => {
                    log::info!("[KB] acquired exclusive file lock on knowledge.db");
                    locked = true;
                }
                Err(e) => {
                    log::warn!("[KB] file lock busy/failed ({e}), continue without exclusive lock");
                }
            }
        }
        let kb = Self::init_fields(conn, db_path.clone(), db_file, true);
        // 如实记录：只有真正拿到锁才标持有（之前无条件置 true 会误导
        // lock_before_write 跳过加锁；拿不到时写前会再试）。
        kb.file_lock_held.store(locked, Ordering::Relaxed);
        let db_path_str = db_path.display().to_string();
        log::info!("[KB] opened at {db_path_str} — graph_cache lazy (rebuilt by background loop on demand); BM25/tech-reserve lazy");

        if std::env::var("NEOTRIX_EMBEDDING_API_KEY").is_err() {
            log::warn!(
                "[KB] NEOTRIX_EMBEDDING_API_KEY not set — semantic search disabled. \
                Set it to enable vector embedding support."
            );
        }

        Ok(kb)
    }

    pub fn init_agent_session(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::ensure_tables(&conn).map_err(|e| format!("Table init: {}", e))?;
        *self
            .agent_session
            .write()
            .map_err(|e| format!("Lock: {}", e))? = true;
        Ok(())
    }

    /// Build minimal KB from an existing Connection (for fallback paths).
    pub(crate) fn _from_conn(conn: Connection, db_path: PathBuf) -> Self {
        Self::init_fields(conn, db_path, None, false)
    }

    pub fn rebuild_skills_library(&self) -> Result<usize, String> {
        let mut lib = self
            .skills_library
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        lib.rebuild_from_kb(self)
    }

    pub fn rebuild_graph_cache(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let mut cache = self
            .graph_cache
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        *cache = nt_memory_graph_cache::GraphCache::new(&conn)
            .unwrap_or_else(|_| nt_memory_graph_cache::GraphCache::empty());
        log::info!(
            "[KB] graph_cache rebuilt: {} edges, {} nodes",
            cache.edge_count,
            cache.node_count
        );
        Ok(())
    }

    pub fn embedding_available(&self) -> bool {
        self.embedding_config.read().is_ok_and(|c| c.is_some())
    }

    pub fn integrity_check(&self) -> Vec<String> {
        let mut issues: Vec<String> = Vec::new();
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                issues.push(format!("Lock error: {}", e));
                return issues;
            }
        };
        // 1. Check dangling edges
        if let Ok(mut dangling) = conn.prepare(
            "SELECT e.id FROM edges e LEFT JOIN nodes n ON e.source_id = n.id WHERE n.id IS NULL \
             UNION ALL SELECT e.id FROM edges e LEFT JOIN nodes n ON e.target_id = n.id WHERE n.id IS NULL"
        ) {
            let count: usize = dangling.query_map([], |_| Ok(()))
                .map(|r| r.count())
                .unwrap_or(0);
            if count > 0 {
                issues.push(format!("Dangling edges: {} edges reference non-existent nodes", count));
            }
        }
        // 2. Check orphan nodes (no edges)
        if let Ok(mut orphans) = conn.prepare(
            "SELECT COUNT(*) FROM nodes n WHERE n.id NOT IN (SELECT source_id FROM edges) \
             AND n.id NOT IN (SELECT target_id FROM edges)",
        ) {
            if let Ok(count) = orphans.query_row([], |row| row.get::<_, usize>(0)) {
                if count > 0 {
                    issues.push(format!("Orphan nodes: {} nodes have no connections", count));
                }
            }
        }
        // 3. Check stale data (nodes created >30 days ago, never accessed)
        let stale_threshold = chrono::Utc::now().timestamp() - 2_592_000;
        if let Ok(mut stale) =
            conn.prepare("SELECT COUNT(*) FROM nodes WHERE created_at < ?1 AND last_accessed < ?1")
        {
            if let Ok(count) = stale.query_row([stale_threshold], |row| row.get::<_, usize>(0)) {
                if count > 0 {
                    issues.push(format!(
                        "Stale nodes: {} nodes untouched for >30 days",
                        count
                    ));
                }
            }
        }
        // 4. Check embedding count vs node count
        if let Ok(emb_count) = conn.query_row(
            "SELECT COUNT(*) FROM embeddings WHERE embedding IS NOT NULL",
            [],
            |row| row.get::<_, usize>(0),
        ) {
            if let Ok(node_count) = conn.query_row("SELECT COUNT(*) FROM nodes", [], |row| {
                row.get::<_, usize>(0)
            }) {
                if node_count > 0 && emb_count < node_count / 10 {
                    issues.push(format!(
                        "Embedding gap: {}/{} nodes have embeddings (<10%)",
                        emb_count, node_count
                    ));
                }
            }
        }
        issues
    }

    /// 物理压缩数据库（VACUUM）— 回收删除/更新产生的空闲页，缩小文件体积。
    ///
    /// 这是对一次性运维脚本（如 cleanup-opencode.sh）的正式能力沉淀：
    /// 数据库维护/压缩应作为 KB 一等公民能力，而非每次手写脚本。
    ///
    /// `prune_stale_days` 若 >0，先删除超过该天数且从未访问的节点（回收逻辑空间），
    /// 再 VACUUM 回收物理空间。返回 `(pruned_nodes, freed_bytes)`。
    pub fn compact(&self, prune_stale_days: Option<u32>) -> Result<(usize, i64), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("KB compact lock: {}", e))?;

        // 1. 可选：清理过期节点（逻辑空间回收）
        let mut pruned = 0usize;
        if let Some(days) = prune_stale_days {
            let threshold = chrono::Utc::now().timestamp() - (days as i64) * 86_400;
            let deleted = conn
                .execute(
                    "DELETE FROM nodes WHERE created_at < ?1 AND access_count = 0",
                    [threshold],
                )
                .map_err(|e| format!("KB compact prune: {}", e))?;
            pruned = deleted;
            // 清理孤儿边（被删节点的边）
            let _ = conn.execute(
                "DELETE FROM edges WHERE source_id NOT IN (SELECT id FROM nodes) \
                 OR target_id NOT IN (SELECT id FROM nodes)",
                [],
            );
        }

        // 2. 记录压缩前文件大小
        let before = std::fs::metadata(&self.db_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        // 3. VACUUM 物理压缩
        conn.execute_batch("VACUUM;")
            .map_err(|e| format!("KB compact vacuum: {}", e))?;

        let after = std::fs::metadata(&self.db_path)
            .map(|m| m.len() as i64)
            .unwrap_or(before);
        let freed = before.saturating_sub(after);

        log::info!(
            "[KB] compact: pruned={} nodes, size {} -> {} (freed {} bytes)",
            pruned,
            before,
            after,
            freed
        );
        Ok((pruned, freed))
    }

    /// Acquire a locked reference to the underlying SQLite connection
    pub fn raw_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.conn.lock().map_err(|e| format!("KB lock: {}", e))
    }

    /// Open a clone connection to the same DB (for sharing across subsystems)
    pub fn clone_connection(&self) -> Self {
        Self::open(Some(self.db_path.clone())).unwrap_or_else(|e| {
            warn!("[neotrix] clone_connection: KB::open({}) failed: {}. Trying default path.", self.db_path.display(), e);
            Self::open(None).unwrap_or_else(|e| {
                warn!("[neotrix] clone_connection: default path also failed: {}. Creating in-memory KB.", e);
                let Ok(conn) = Connection::open_in_memory() else {
                    error!("[neotrix] FATAL: cannot create in-memory SQLite database");
                    std::process::abort();
                };
                let _ = nt_memory_schema::initialize(&conn);
                Self::_from_conn(conn, PathBuf::from(":memory:"))
            })
        })
    }

    pub fn with_embedding(self, config: EmbeddingConfig) -> Self {
        if let Ok(mut c) = self.embedding_config.write() {
            *c = Some(config);
        }
        self
    }

    // ── BM25 ──

    pub fn mark_bm25_dirty(&self) {
        let _ = self.bm25_dirty.write().map(|mut d| *d = true);
        let _ = self.fused_cache.lock().map(|mut c| c.clear());
    }

    pub fn rebuild_bm25(&self) {
        let needs_rebuild = self.bm25_dirty.read().map(|d| *d).unwrap_or(false);
        if !needs_rebuild {
            return;
        }

        let total = {
            let conn = match self.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[KB] rebuild_bm25 lock: {}", e);
                    return;
                }
            };
            nt_memory_store::count_nodes(&conn).unwrap_or(0)
        };
        if total == 0 {
            if let Ok(mut d) = self.bm25_dirty.write() {
                *d = false;
            }
            return;
        }

        use crate::l1_action::nt_core_memory_budget;
        let budget = nt_core_memory_budget::global();
        let page_size = budget.check().suggested_batch_size().max(100);

        let mut index = bm25::Bm25Index::empty();
        let mut offset = 0;
        let mut processed = 0;
        loop {
            if budget.should_throttle() {
                log::warn!(
                    "[KB] rebuild_bm25 throttled at {} docs — resuming later",
                    processed
                );
                return;
            }
            let conn = match self.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[KB] rebuild_bm25 lock: {}", e);
                    break;
                }
            };
            let page = match nt_memory_store::get_nodes_page(&conn, offset, page_size) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("[KB] rebuild_bm25 page: {}", e);
                    break;
                }
            };
            drop(conn);
            if page.is_empty() {
                break;
            }
            for node in &page {
                let text = format!(
                    "{} {} {}",
                    node.title,
                    node.summary.as_deref().unwrap_or(""),
                    node.content.as_deref().unwrap_or(""),
                );
                index.add_document(&bm25::Bm25Document {
                    id: node.id.clone(),
                    text,
                });
            }
            processed += page.len();
            offset += page.len();
            if page.len() < page_size {
                break;
            }
        }

        if let Ok(mut bm25) = self.bm25.write() {
            *bm25 = Some(index);
        }
        if let Ok(mut d) = self.bm25_dirty.write() {
            *d = false;
        }
        log::info!(
            "[KB] BM25 index rebuilt: {} docs (page_size={})",
            processed,
            page_size
        );
    }

    /// Rebuild tech reserve index from all KB nodes (streaming, page-by-page).
    pub fn rebuild_tech_reserve(&self) {
        use crate::l1_action::nt_core_memory_budget;
        let budget = nt_core_memory_budget::global();
        let page_size = budget.check().suggested_batch_size().max(100);

        let total = {
            let conn = match self.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[KB] rebuild_tech_reserve lock: {}", e);
                    return;
                }
            };
            nt_memory_store::count_nodes(&conn).unwrap_or(0)
        };
        if total == 0 {
            return;
        }

        {
            if let Ok(mut tr) = self.tech_reserve.write() {
                tr.clear();
            }
        }

        let mut offset = 0;
        let mut processed = 0;
        loop {
            if budget.should_throttle() {
                log::warn!("[KB] rebuild_tech_reserve throttled at {} nodes", processed);
                if let Ok(mut tr) = self.tech_reserve.write() {
                    tr.clear();
                }
                return;
            }
            let conn = match self.conn.lock() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[KB] rebuild_tech_reserve lock: {}", e);
                    break;
                }
            };
            let page =
                nt_memory_store::get_nodes_page(&conn, offset, page_size).unwrap_or_default();
            drop(conn);
            if page.is_empty() {
                break;
            }
            if let Ok(mut tr) = self.tech_reserve.write() {
                for node in &page {
                    tr.add_node(node);
                }
            }
            processed += page.len();
            offset += page.len();
            if page.len() < page_size {
                break;
            }
        }
        if let Ok(tr) = self.tech_reserve.read() {
            log::info!("[KB] Tech reserve rebuilt: {} entries across {} dimensions (streamed, page_size={})",
                tr.entry_count(), tr.stats_by_dimension().len(), page_size);
        }
    }

    /// Run a crawl cycle and refresh tech reserve afterward.
    pub fn run_crawl_cycle_and_refresh(
        &self,
        max_items: usize,
    ) -> Result<CrawlCycleReport, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let report = nt_memory_crawl::run_crawl_cycle(&conn, max_items)?;
        drop(conn);
        self.rebuild_tech_reserve();
        Ok(report)
    }

    /// Query the tech reserve for mature products in a domain.
    pub fn query_tech_reserve(&self, domain: &str, top_k: usize) -> Vec<TechReserveEntry> {
        let tr = self.tech_reserve.read().unwrap_or_else(|e| e.into_inner());
        tr.latest_mature_products(domain, top_k)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get full 4D tech profile for a technology.
    pub fn tech_profile(&self, tech_name: &str) -> TechProfile {
        let tr = self.tech_reserve.read().unwrap_or_else(|e| e.into_inner());
        tr.full_tech_profile(tech_name)
    }

    /// 读取全部已吸收能力 `(branch_str, capability)` 对 (Cycle 206 R-P79 闭环)。
    pub fn absorbed_capabilities(&self) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_absorb_mapper::load_absorbed_capabilities(&conn).map_err(|e| e.to_string())
    }

    // ── close ──

    pub fn close(self) -> Result<(), String> {
        // Connection is dropped; nothing else to do
        Ok(())
    }
}
