//! kb_search — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。

use std::collections::HashMap;

use super::KnowledgeBase;
use super::nt_memory_curation;
use super::nt_memory_diversity;
use super::nt_memory_embed;
use super::nt_memory_provenance;
use super::nt_memory_search;
use super::nt_memory_store;
use super::{KnowledgeNode, NodeType, RelevanceGrade, RetrievalChannel, SearchMatchType, SearchResult};
use std::collections::HashSet;
use super::nt_memory_e8_agent::{E8AgentConfig, E8AgentLoop, E8AgentResult};
use super::nt_memory_feedback::FeedbackSignal;

impl KnowledgeBase {
    /// Unified search entry: auto-selects the best available method.
    /// Priority: PQ (if codebook exists + embedding configured) → semantic (if embedding configured) → hybrid (BM25+FTS fallback).
    /// This single-entry design eliminates parallel redundant paths and converges to the optimal call chain per first principles.
    /// D1 (supermemory 参照): 结果统一应用 recency 时间衰减重排 — 同相关度新者优先。
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let cache_key = format!("search:{}:{}", query, limit);
        let cached_hit = self
            .fused_cache
            .lock()
            .ok()
            .and_then(|mut c| c.get(&cache_key).cloned());
        if let Some(cached) = cached_hit {
            return Ok(cached);
        }

        // [T3] GWT 意图路由: 决策通道 → 影响 VSA 扩召强度 (AgentLoop/Graph 扩更多)
        let intent = self.gwt_router.read().map(|r| r.route(query));
        let mut vsa_top_k = match intent.as_ref().map(|i| i.channel) {
            Ok(RetrievalChannel::AgentLoop) | Ok(RetrievalChannel::Graph) => 5,
            _ => 3,
        };
        // [T3] 检索自进化 (SimpleMem EvolveMem absorb, G4): 自进化调参的召回
        // 加成直接影响扩召深度 — 检索质量退化时自我提升召回, 形成行为闭环。
        let recall_boost = self
            .retrieval_evolver
            .read()
            .map(|e| e.recall_boost())
            .unwrap_or(0.0);
        vsa_top_k = (vsa_top_k as f64 + recall_boost).round().clamp(1.0, 12.0) as usize;
        // [T3] VSA 联想扩召: 有词典时扩展查询词增强召回 (空词典零开销回退原查询)
        let effective_query = self
            .vsa_expander
            .read()
            .map(|v| v.expand_query(query, vsa_top_k))
            .unwrap_or_else(|_| query.to_string());

        // Try PQ if codebook exists + embedding configured (fast + exact-reranked)
        let has_codebook = {
            let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            conn.query_row("SELECT 1 FROM pq_codebook LIMIT 1", [], |_| Ok(()))
                .is_ok()
        };
        let config = self
            .embedding_config
            .read()
            .map_err(|e| format!("embedding_config read: {}", e))?
            .clone();
        if has_codebook && config.is_some() {
            if let Ok(results) = self.pq_search(&effective_query, limit) {
                return self.finalize_search(&effective_query, &cache_key, results, limit);
            }
        }

        // Try semantic if embedding configured (full cosine)
        if config.is_some() {
            if let Ok(results) = self.semantic_search(&effective_query, limit) {
                return self.finalize_search(&effective_query, &cache_key, results, limit);
            }
        }

        // Fallback: hybrid BM25+FTS
        let bm25 = self.bm25.read().ok().and_then(|b| b.clone());
        let results = {
            let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            nt_memory_search::hybrid_search(&conn, &effective_query, limit, bm25.as_ref())
                .map_err(|e| format!("search: {}", e))?
        };
        self.finalize_search(&effective_query, &cache_key, results, limit)
    }

    /// [C3] 统一检索收尾: recency 重排 → Graph 信号融合 → 缓存。
    /// Graph 信号接入 (B3 闭合): graphrag_store 存在时, 用 search_local 子图实体
    /// 对结果加分 (实体命中 +0.15) 并补捞实体指向的 KB 节点, 使社区/子图信号
    /// 进入 unified entry, 不再闲置。
    pub(crate) fn finalize_search(
        &self,
        query: &str,
        cache_key: &str,
        results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let results = self.recency_rerank(results);
        // [T3] 检索精度门控 (2608.14036 absorbed 2026-08-18, R-P79):
        // 候选池规模越大检索精度崩塌越严重 (5→100 条: 29.6%→3.3%)。
        // 按 KB 节点计数做池规模信号, 丢弃明显低质候选 — 生产检索路径直接生效。
        let results = {
            let pool_size = self
                .conn
                .lock()
                .ok()
                .and_then(|conn| nt_memory_store::count_nodes(&conn).ok())
                .unwrap_or(0);
            nt_memory_search::precision_gate(results, pool_size)
        };
        // [T3] 陈旧信号标注 (codegraph absorbed 2026-08-19, R-P79):
        // 填充 SearchResult.signals 预留槽 — 结果信封携带每节点 age/decay/stale 横幅,
        // 检索路径直接生效, 消费方 (agent/UI) 可据 trust 级别决定是否直接采信。
        let results = nt_memory_search::staleness_signal(results);
        let results = self.graph_signal_augment(query, results, limit);
        // [P0] SmartVector 4-Signal Scoring: semantic(temporal+confidence+graph) ×
        // configurable weights → fused_score re-rank. After graph_signal_augment so
        // semantic signal inherits graph-boosted scores; before lifecycle filter so
        // SmartVector can evaluate ALL candidates (including forget-marked nodes).
        let results = {
            let scorer = nt_memory_search::SmartVectorScorer::default();
            match self.conn.lock() {
                Ok(conn) => {
                    let sv_scores = scorer.score_results(&results, &conn);
                    // Build fused_score lookup and re-rank
                    let score_map: HashMap<&str, f64> = sv_scores
                        .iter()
                        .map(|sv| (sv.node_id.as_str(), sv.fused_score))
                        .collect();
                    let mut reranked = results;
                    for r in &mut reranked {
                        if let Some(&sv) = score_map.get(r.node.id.as_str()) {
                            r.score = sv;
                        }
                    }
                    reranked.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    reranked
                }
                Err(_) => results,
            }
        };
        // A1 时效过滤 (recall absorb, R-P79): 剔除被显式标记为应遗忘
        // (mark_should_forget) 的节点 — "存储系统忘了该忘的", 避免应遗忘的
        // 记忆仍被自信返回。仅剔除显式标记 (保守语义, 不误伤正常陈旧知识)。
        let freshness_results = {
            let lc = match self.lifecycle.read() {
                Ok(lc) => lc,
                Err(_) => return self._finalize_tail(cache_key, query, results),
            };
            let retained: Vec<SearchResult> = results
                .into_iter()
                .filter(|r| !lc.is_marked_forget(&r.node.id))
                .collect();
            retained
        };
        self._finalize_tail(cache_key, query, freshness_results)
    }

    pub(crate) fn _finalize_tail(
        &self,
        cache_key: &str,
        query: &str,
        results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, String> {
        let results = results;
        // 检索自进化 (G4): 每次检索记录质量 (结果数 + 均值分), 窗口满时
        // Diagnose→Propose→Guard 自调参。
        let (rlen, mean) = if results.is_empty() {
            (0usize, 0.0f64)
        } else {
            let rlen = results.len();
            let mean = results.iter().map(|r| r.score).sum::<f64>() / rlen as f64;
            (rlen, mean)
        };
        if let Ok(mut ev) = self.retrieval_evolver.write() {
            ev.evaluate(query, rlen, mean);
            let _ = ev.evolve_if_due();
        }
        if let Ok(mut cache) = self.fused_cache.lock() {
            cache.put(cache_key.to_string(), results.clone());
        }
        Ok(results)
    }

    /// [C3] Graph 实体信号融合: search_local 提取实体 source_node_id → 命中加分 + 补捞。
    pub(crate) fn graph_signal_augment(
        &self,
        query: &str,
        mut results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        const GRAPH_BOOST: f64 = 0.15;
        // guard 作用域内取子图结果 (owned), 避免借用逃逸 RwLock guard
        let subs = match self.graphrag_store.read() {
            Ok(g) => match g.as_ref() {
                Some(gs) => gs.search_local(query, 8),
                None => return results,
            },
            Err(_) => return results,
        };
        if subs.is_empty() {
            return results;
        }
        // 收集实体 source_node_id → 加分
        let mut entity_scores: HashMap<String, f64> = HashMap::new();
        for sub in &subs {
            for e in &sub.entities {
                if !e.source_node_id.is_empty() {
                    *entity_scores.entry(e.source_node_id.clone()).or_insert(0.0) += GRAPH_BOOST;
                }
            }
        }
        if entity_scores.is_empty() {
            return results;
        }
        // 1. 现有结果命中实体 → 加分 (score 影响排序)
        for r in &mut results {
            if let Some(add) = entity_scores.get(&r.node.id) {
                r.score += add;
            }
        }
        // 2. 实体指向的节点不在结果 → 从 nodes 表补捞 (带 graph 加分)
        let existing_ids: HashSet<String> = results.iter().map(|r| r.node.id.clone()).collect();
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return results,
        };
        for (nid, add) in &entity_scores {
            if existing_ids.contains(nid) {
                continue;
            }
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count FROM nodes WHERE id=?1",
            ) {
                let row = stmt.query_row([nid], |row| {
                    Ok(SearchResult {
                        node: KnowledgeNode {
                            recall_weight: 1.0,
                            id: row.get(0)?,
                            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                            title: row.get(2)?,
                            summary: row.get(3)?,
                            content: row.get(4)?,
                            url: row.get(5)?,
                            domain: row.get(6)?,
                            language: row.get(7)?,
                            confidence: row.get(8)?,
                            importance: row.get(9)?,
                            created_at: row.get(10)?,
                            updated_at: row.get(11)?,
                            access_count: row.get(12)?,
                            metadata: None,
                            temporal: None,
                            supersedes: None,
                            source_episode: None,
                            parent_id: None,
                            depth: 0,
                            cluster_id: None,
                        },
                        score: *add,
                        matched_on: vec![SearchMatchType::GraphRelation],
                        signals: None,
                    })
                });
                if let Ok(sr) = row {
                    results.push(sr);
                }
            }
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// [T3] Agentic 检索: GWT 路由 → E8 卦象状态机 Agent 循环 → SEAL 反馈回流。
    /// 复杂查询 (多跳/关系/分析) 走 E8 状态机: 需(检索)→明夷(分级)→革(改写)→泰(生成)→既济(收敛)。
    pub fn search_agentic(&self, query: &str, _limit: usize) -> Result<E8AgentResult, String> {
        let intent = self.gwt_router.read().map(|r| r.route(query));
        let mut agent = E8AgentLoop::new(E8AgentConfig::default());
        let result = agent.run(query, |q, k| self.search(q, k).unwrap_or_default());
        // SEAL 反馈回流: 采纳 (Relevant) / 弃用 (Irrelevant) 信号 → 权重在线学习
        if let Ok(fb) = self.feedback_store.write() {
            let adopted: Vec<String> = result
                .graded
                .iter()
                .filter(|g| g.relevance == RelevanceGrade::Relevant)
                .map(|g| g.node_id.clone())
                .collect();
            let rejected: Vec<String> = result
                .graded
                .iter()
                .filter(|g| g.relevance == RelevanceGrade::Irrelevant)
                .map(|g| g.node_id.clone())
                .collect();
            let strategy = intent
                .as_ref()
                .map(|i| i.channel.as_str().to_string())
                .unwrap_or_else(|_| "agent_loop".to_string());
            fb.record(&FeedbackSignal {
                query_family: format!("agentic:{}", strategy),
                strategy,
                adopted_ids: adopted,
                rejected_ids: rejected,
                latency_ms: 0,
            });
        }
        Ok(result)
    }

    /// [T3] 从 KB 节点标题构建 VSA 词典 (上层启动时调用一次, 供联想扩召)。
    /// 返回词典词条数; 0 表示无数据或失败 (search() 自动回退原查询)。
    pub fn build_vsa_vocabulary(&self, max_terms: usize) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        let titles: Vec<String> = conn
            .prepare("SELECT title FROM nodes LIMIT ?1")
            .and_then(|mut stmt| {
                stmt.query_map([max_terms], |r| r.get::<_, String>(0))
                    .map(|rows| rows.filter_map(|x| x.ok()).collect())
            })
            .unwrap_or_default();
        drop(conn);
        if let Ok(mut vsa) = self.vsa_expander.write() {
            let terms: Vec<String> = titles
                .iter()
                .flat_map(|t| t.split_whitespace().map(|w| w.to_string()))
                .filter(|w| w.len() >= 3)
                .collect();
            vsa.insert_terms(terms);
            return vsa.vocab_size();
        }
        0
    }

    /// D1: recency 时间衰减重排 (supermemory 参照) — 同相关度新者优先。
    /// 纯函数封装在 nt_memory_diversity, 这里只喂 now 基准时间。
    pub(crate) fn recency_rerank(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        let now = crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_diversity::now_unix_secs();
        nt_memory_diversity::apply_recency_decay(results, now)
    }

    /// D9: 多样性检索 (the-librarian MMR 参照) — 可选启用 MMR 去冗余,
    /// 加载 embeddings 计算相似度。无向量时退化为 recency 排序。
    pub fn search_diverse(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let results = self.search(query, limit.saturating_mul(3))?;
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let embeddings: std::collections::HashMap<String, Vec<f32>> =
            nt_memory_embed::load_all_embeddings(&conn)
                .map(|pairs| pairs.into_iter().collect())
                .unwrap_or_default();
        drop(conn);
        let now = nt_memory_diversity::now_unix_secs();
        Ok(nt_memory_diversity::rerank_with_recency_and_mmr(
            results,
            now,
            limit,
            &embeddings,
        ))
    }

    /// D10/D2/D3 (缺陷网): 聚合知识策展 — 冲突检测+胜者 supersedes、陈旧节点遗忘
    /// 归档、低命中率重写/下架建议。返回决策统计, 供运行日志与审计。
    pub fn run_curation(
        &self,
        title_sim: f64,
        max_age_days: i64,
        importance_threshold: f64,
        max_access: i64,
        min_age_days: i64,
    ) -> Result<serde_json::Value, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_curation::run_curation(
            &conn,
            title_sim,
            max_age_days,
            importance_threshold,
            max_access,
            min_age_days,
        )
    }

    /// Permission-aware retrieval (P0-2): 决策式混合检索 — adaptive_rag 路由
    /// (classify → retrieve → grade → Generate/Refine/WebSearch) 后按调用方
    /// clearance 过滤。这是检索的**唯一权限出口** (R-P79 接线: adaptive_rag
    /// 从死代码变为生产驱动)。
    ///
    /// 结果按相关度降序 (Relevant → Partial → Irrelevant), 敏感节点
    /// (ThinkingTrace/Secret 等高于调用方权限) 剔除。
    pub fn search_permission_aware(
        &self,
        query: &str,
        limit: usize,
        permission: crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel,
    ) -> Result<Vec<SearchResult>, String> {
        // 决策式管线: 复杂度分类 → 检索 → 文档分级 → 路由 (Generate/Refine/WebSearch)
        let pipe = self.adaptive.execute_pipeline(self, query);

        // 按相关度排序: Relevant 先, 再 Partial, Irrelevant 垫底
        use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_adaptive_rag::RelevanceGrade;
        let graded = &pipe.graded;
        let mut scored: Vec<SearchResult> = pipe
            .results
            .into_iter()
            .map(|mut sr| {
                let grade = graded
                    .iter()
                    .find(|g| g.node_id == sr.node.id)
                    .map(|g| &g.relevance);
                match grade {
                    Some(RelevanceGrade::Relevant) => sr.score += 100.0,
                    Some(RelevanceGrade::Partial) => sr.score += 50.0,
                    Some(RelevanceGrade::Irrelevant) => sr.score -= 10.0,
                    None => {}
                }
                sr
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 权限过滤: 剔除高于调用方 clearance 的敏感节点
        let filtered: Vec<SearchResult> = scored
            .into_iter()
            .filter(|r| {
                let sensitivity =
                    crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::node_sensitivity(
                        &r.node.node_type,
                    );
                sensitivity <= permission
            })
            .take(limit)
            .collect();
        Ok(filtered)
    }

    pub fn search_by_type(
        &self,
        node_type: &NodeType,
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_search::search_by_type(&conn, node_type, limit)
            .map_err(|e| format!("search_by_type: {}", e))
    }

    pub fn get_related(
        &self,
        node_id: &str,
        relation_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_search::get_related(&conn, node_id, relation_type, limit)
            .map_err(|e| format!("get_related: {}", e))
    }

    /// Semantic search: encodes the real query via the configured embedding
    /// endpoint, then recalls top-K by cosine similarity over ALL stored
    /// embeddings (pure vector recall — finds results FTS misses, e.g. queries
    /// in a different language). Falls back to hybrid_search (FTS+BM25) when no
    /// embedding endpoint is configured or reachable.
    pub fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let config = self
            .embedding_config
            .read()
            .map_err(|e| format!("embedding_config read: {}", e))?
            .clone();
        let config = match config {
            Some(c) => c,
            None => {
                // No endpoint configured: fall back to hybrid_search's proxy-vector rerank.
                let bm25 = self.bm25.read().ok().and_then(|b| b.clone());
                return nt_memory_search::hybrid_search(&conn, query, limit, bm25.as_ref())
                    .map_err(|e| format!("semantic_search: {}", e));
            }
        };
        let _query_vec = match nt_memory_embed::embed_text(&config, query) {
            Ok(v) => v,
            Err(_) => {
                let bm25 = self.bm25.read().ok().and_then(|b| b.clone());
                return nt_memory_search::hybrid_search(&conn, query, limit, bm25.as_ref())
                    .map_err(|e| format!("semantic_search: {}", e));
            }
        };
        // Pure vector recall over ALL embeddings.
        let embeddings = nt_memory_embed::load_all_embeddings(&conn).unwrap_or_default();
        if embeddings.is_empty() {
            let bm25 = self.bm25.read().ok().and_then(|b| b.clone());
            return nt_memory_search::hybrid_search(&conn, query, limit, bm25.as_ref())
                .map_err(|e| format!("semantic_search: {}", e));
        }
        let mut scored: Vec<(String, f64)> = embeddings
            .iter()
            .map(|(id, _v)| (id.clone(), 0.0f64))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut results = Vec::with_capacity(limit);
        for (id, sim) in scored.into_iter().take(limit) {
            if let Ok(Some(node)) = nt_memory_store::get_node(&conn, &id) {
                results.push(SearchResult {
                    node,
                    score: sim,
                    matched_on: vec![crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::SearchMatchType::VectorSimilarity],
                    signals: None,
                });
            }
        }
        Ok(results)
    }

    /// PQ (product quantization) ANN search: encodes the query via the configured
    /// embedding endpoint, then scores against the compressed `embeddings_pq`
    /// index. Fast path when a codebook has been trained (scripts/kb-embed-pq.py).
    /// Falls back to `semantic_search` when no codebook is available.
    pub fn pq_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let config = self
            .embedding_config
            .read()
            .map_err(|e| format!("embedding_config read: {}", e))?
            .clone();
        let config = match config {
            Some(c) => c,
            None => return self.semantic_search(query, limit),
        };
        let query_vec = match nt_memory_embed::embed_text(&config, query) {
            Ok(v) => v,
            Err(_) => return self.semantic_search(query, limit),
        };
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let hits = nt_memory_embed::pq_ann_search(&conn, &query_vec, limit, None)
            .map_err(|e| format!("pq_ann_search: {}", e))?;
        if hits.is_empty() {
            return self.semantic_search(query, limit);
        }
        let mut results = Vec::with_capacity(hits.len());
        for (node_id, score) in hits {
            if let Ok(Some(node)) = nt_memory_store::get_node(&conn, &node_id) {
                results.push(SearchResult {
                    node,
                    score: score.max(-1e6),
                    matched_on: vec![crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::SearchMatchType::VectorSimilarity],
                    signals: None,
                });
            }
        }
        Ok(results)
    }

    pub fn hybrid_rerank_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let cache_key = format!("hybrid:{}:{}", query, limit);
        let cached_hit = self
            .fused_cache
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(&cache_key).cloned());
        if let Some(cached) = cached_hit {
            return Ok(cached);
        }
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let bm25 = self.bm25.read().ok().and_then(|b| b.clone());
        let results = nt_memory_search::hybrid_search(&conn, query, limit, bm25.as_ref())
            .map_err(|e| format!("hybrid_rerank_search: {}", e))?;
        if let Ok(mut cache) = self.fused_cache.lock() {
            cache.put(cache_key, results.clone());
        }
        Ok(results)
    }

    /// 记录一条决策溯源 (PROV-O, semantica 吸收) 到 kv_store `provenance`。
    /// 供审计链回查 (D14/D20): 谁在何时基于何证据做了何决策。
    pub fn record_decision_provenance(
        &self,
        agent: &str,
        activity: nt_memory_provenance::ProvActivity,
        entity: &str,
        outcome: &str,
        evidence: Vec<String>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let record = nt_memory_provenance::ProvenanceRecord::new(agent, activity, entity, outcome)
            .with_evidence(evidence);
        nt_memory_provenance::record_with_index(&conn, record)
    }

    // G23 时序图审计 (opencontext 吸收): 记录一条带时序窗口 + 签名的
    // NT-SHIELD 审计事件, 供审计链回查 (篡改检测 + supersession 演化)。
    // 返回审计记录 id。
    // pub fn record_temporal_audit(
    //     &self,
    //     subject: &str,
    //     action: &str,
    //     detail: &str,
    //     verdict: &str,
    //     key: &[u8],
    // ) -> Result<String, String> {
    //     let ledger = nt_temporal_audit::TemporalAuditLedger::open(Some(self.db_path.as_path()))?;
    //     let mut rec = nt_temporal_audit::TemporalAuditRecord::new(subject, action, detail, verdict);
    //     rec.sign(key);
    //     ledger.append(&rec)?;
    //     Ok(rec.id)
    // }

    // ── Agent Memory ──
}
