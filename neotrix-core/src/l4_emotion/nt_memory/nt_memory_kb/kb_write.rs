//! kb_write — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_blocks;
use super::nt_memory_curation;
use super::nt_memory_graphrag;
use super::nt_memory_svaf_gate;
use super::{NodeType, RelationType};

/// 从 url/domain 推导 SVAF source_type (T0.4 validated writeback)。
/// 优先级: url 域名特征 > domain 字段 > unknown。
fn derive_source_type(url: Option<&str>, domain: Option<&str>) -> String {
    if let Some(u) = url {
        let lower = u.to_lowercase();
        for (pat, ty) in [
            ("arxiv", "arxiv"),
            ("wikipedia", "wikipedia"),
            ("github", "github"),
            ("blog", "blog"),
            ("news", "news"),
            ("forum", "forum"),
        ] {
            if lower.contains(pat) {
                return ty.to_string();
            }
        }
    }
    domain.unwrap_or("unknown").to_string()
}

impl KnowledgeBase {

    /// 子方法: 主库写入 — 插入或复用节点 + 时序事实记账。
    pub(crate) fn write_memory_entry_core(
        &self,
        title: &str,
        node_type: NodeType,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        let node_id = self.insert_or_get_node(title, node_type, content, url, domain)?;
        // 时序事实记账 (TemporalFactLedger 接线, R-P79): 事实型节点 (有正文)
        // 写入 append-only temporal_facts, 知识变更获得 point-in-time 语义。
        if let Ok(Some(node)) = self.get_node(&node_id) {
            self.record_node_fact(&node);
        }
        Ok(node_id)
    }

    /// 子方法: 代际版本化 — generation stamp 递增 + written_at 时间戳 + 类型化块分块统计。
    pub(crate) fn write_memory_entry_versioned(
        &self,
        node_id: &str,
        content: Option<&str>,
    ) -> Result<(), String> {
        // 代际版本化: 同源重写 (同 URL/同标题) 每次经统一弧落库都在 metadata
        // 递增 generation, 形成可审计的版本链。
        let mut meta = self
            .get_node(node_id)?
            .and_then(|n| n.metadata.clone())
            .unwrap_or_else(|| serde_json::json!({}));
        let generation = meta.get("generation").and_then(|g| g.as_u64()).unwrap_or(0) + 1;
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("generation".to_string(), serde_json::json!(generation));
            obj.insert(
                "written_at".to_string(),
                serde_json::json!(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64),
            );
        }
        self.update_node_metadata(node_id, &meta)?;

        // 类型化块分块 (typed block stats) — 保留表格/公式/代码/标题结构。
        let text_now = content.unwrap_or_default();
        if !text_now.is_empty() {
            let blocks = nt_memory_blocks::split_typed_blocks(text_now);
            let stats = nt_memory_blocks::block_stats(&blocks);
            let mut meta2 = self
                .get_node(node_id)?
                .and_then(|n| n.metadata.clone())
                .unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = meta2.as_object_mut() {
                obj.insert("block_types".to_string(), serde_json::json!(stats));
            }
            self.update_node_metadata(node_id, &meta2)?;
        }
        Ok(())
    }

    /// 子方法: evidence 元数据 — 将 source_id + run_id + sha256 落到主节点 metadata。
    pub(crate) fn write_memory_entry_evidence(
        &self,
        node_id: &str,
        evidence: &serde_json::Value,
    ) -> Result<(), String> {
        let mut meta = self
            .get_node(node_id)?
            .and_then(|n| n.metadata.clone())
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("evidence".to_string(), evidence.clone());
        }
        self.update_node_metadata(node_id, &meta)
    }

    /// 子方法: SVAF 写入门禁 — 评估写入质量, 记录决策到 metadata, 返回 gate_passed 标志。
    /// Reject/Redundant → 跳过后续 graphrag 派生 (不污染语义图), 但基础节点保留 (证据链完整)。
    pub(crate) fn write_memory_entry_svaf(
        &self,
        node_id: &str,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<bool, String> {
        let svaf_eval = self.evaluate_write_gate(content, url, domain);
        let mut svaf_meta = self
            .get_node(node_id)?
            .and_then(|n| n.metadata.clone())
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = svaf_meta.as_object_mut() {
            obj.insert(
                "svaf".to_string(),
                serde_json::json!({
                    "decision": format!("{:?}", svaf_eval.decision),
                    "novelty": svaf_eval.novelty,
                    "coherence": svaf_eval.coherence,
                    "relevance": svaf_eval.relevance,
                    "authority": svaf_eval.authority,
                    "reason": svaf_eval.reason,
                }),
            );
        }
        self.update_node_metadata(node_id, &svaf_meta)?;
        let gate_passed = !matches!(
            svaf_eval.decision,
            nt_memory_svaf_gate::SvafDecision::Reject
                | nt_memory_svaf_gate::SvafDecision::Redundant
        );
        Ok(gate_passed)
    }

    /// 子方法: 冲突检测 — 写后校验新节点与既有相似标题节点, 执行 supersede + PROV-O 溯源。
    /// 与 SVAF 门禁解耦: 事实一致性独立于质量评估, 无论门禁结果都执行。
    pub(crate) fn write_memory_entry_conflict(
        &self,
        node_id: &str,
        content: Option<&str>,
    ) -> Result<(), String> {
        let text = match content {
            Some(t) if !t.trim().is_empty() => t,
            _ => return Ok(()),
        };
        let _ = text; // 仅用于空值守卫

        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let conflicts =
            nt_memory_curation::conflict_detect_for_write(&conn, node_id, 0.4).unwrap_or_default();
        drop(conn);

        if conflicts.is_empty() {
            return Ok(());
        }

        // fidelity ledger (diagram-design 吸收): 冲突解决留差异清单
        // + provenance 溯源, 供审计回查, 而非仅静默覆盖。
        let (applied, ledger) = {
            let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            match nt_memory_curation::apply_supersede_with_ledger(&conn, &conflicts) {
                Ok(ok) => ok,
                Err(e) => {
                    log::warn!("[curation] supersede ledger failed: {}", e);
                    (0, nt_memory_curation::FidelityLedger::new())
                }
            }
        };

        if ledger.is_empty() {
            return Ok(());
        }

        log::info!(
            "[curation] superseded {} nodes w/ provenance ledger: {}",
            applied,
            ledger.len()
        );

        // PROV-O 决策溯源 (semantica 吸收): 每次覆盖解决记录谁/做什么/基于什么证据。
        // 锁已释放, 避免非重入 Mutex 死锁。
        for e in &ledger.entries {
            let _ = self.record_decision_provenance(
                "nt_memory_curation",
                crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_provenance::ProvActivity::Supersede,
                &e.older_id,
                &format!("superseded by {}", e.newer_id),
                vec![e.newer_id.clone(), format!("sim={:.3}", e.sim)],
            );
            // 时序事实账本 (R-P79): 旧节点事实沿版本链 supersede
            // (append-only 更正), 新对象 = 胜出新节点正文。
            if let Ok(Some(newer)) = self.get_node(&e.newer_id) {
                if let Some(obj) = newer.content.as_ref().filter(|c| !c.trim().is_empty()) {
                    let obj: String = obj.chars().take(2048).collect();
                    if let Err(terr) = self
                        .temporal_ledger
                        .lock()
                        .map_err(|x| format!("{x}"))
                        .and_then(|lg| {
                            lg.supersede_node_fact(&e.older_id, &obj, None, "nt_memory_curation")
                                .map_err(|x| x.to_string())
                        })
                    {
                        log::debug!(
                            "[temporal] supersede skip {} -> {}: {}",
                            e.older_id,
                            e.newer_id,
                            terr
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// 统一写入弧 (Unified Ingestion Bus) — 记忆写入的唯一入口。
    ///
    /// Onyx/ai-knowledge-graph 吸收: 任何写记忆动作先落主库 (nodes), 再从主库
    /// 派生 graph (graphrag_extract → entities/relations edges) 与 evidence
    /// (source_id + run_id 元数据), 杜绝 5 条平行写入管线互不相通。
    ///
    /// 返回主库节点 id (已存在则复用, 不重复建点)。
    pub fn write_memory_entry(
        &self,
        title: &str,
        node_type: NodeType,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
        evidence: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        // 1. 主库写入 (插入或复用) + 时序事实记账
        let node_id = self.write_memory_entry_core(title, node_type, content, url, domain)?;

        // 2. 代际版本化 + 类型化块分块统计
        self.write_memory_entry_versioned(&node_id, content)?;

        // 3. evidence 元数据 (source_id + run_id + sha256) 落到主节点
        if let Some(ev) = evidence {
            self.write_memory_entry_evidence(&node_id, ev)?;
        }

        // 4. SVAF 写入门禁 — 评估质量, 决定是否跳过 graphrag 派生
        let gate_passed = self.write_memory_entry_svaf(&node_id, content, url, domain)?;

        // 5. 冲突检测 — 写后校验, supersede 旧节点 + PROV-O 溯源
        self.write_memory_entry_conflict(&node_id, content)?;

        // 6. GraphRAG 派生: 实体/关系从主库内容提取, 落 graphrag_store。
        //    门禁未过 (Reject/Redundant) → 跳过派生 (不污染语义图), 基础节点保留。
        let text = content.unwrap_or_default();
        if gate_passed && !text.is_empty() {
            // 惰性初始化 graphrag store (首次写入自动建立, 无需前置 init)
            if self
                .graphrag_store
                .read()
                .map(|gs| gs.as_ref().is_none())
                .unwrap_or(false)
            {
                let _ = self.init_graphrag(nt_memory_graphrag::GraphRagConfig::default());
            }
            if let Ok((entities, relations)) = self.graphrag_extract(text, &node_id) {
                // 关系边回写主库: node → entity (graphrag 实体作为主库概念点)
                for rel in relations.iter().take(32) {
                    let target_title = rel.target_entity.clone();
                    if let Ok(target_id) = self.insert_or_get_node(
                        &target_title,
                        NodeType::Concept,
                        None,
                        None,
                        domain,
                    ) {
                        let rtype = RelationType::from_str(&rel.relation_type);
                        // T0.1 类型化边: 结构化溯源进 metadata (evidence/source/extractor),
                        // description 保留人类可读证据。
                        let edge_meta = serde_json::json!({
                            "evidence": rel.evidence,
                            "source": domain.unwrap_or("unknown"),
                            "extractor": "graphrag",
                        });
                        // 2026-09-30: 原为 `let _ = self.upsert_edge_with_metadata(…)`。
                        // 该文件 :229-235 的文档把 `write_memory_entry` 定义为
                        // 「记忆写入的**唯一入口**」并承诺「杜绝平行写入管线」。
                        // 丢弃这批边 ⇒ 调用方照常拿到成功的 node_id，
                        // 而语义图缺边 ⇒ **GraphRAG 多跳查询静默少召回，
                        // 且没有任何指标下降可观测**。
                        // 同函数 :176 有 `log::warn!`、:216 有 `log::debug!`
                        // ⇒ 这里是这一段唯一无日志的失败点。照它们改。
                        let mut dropped_edges = 0usize;
                        if let Err(e) = self.upsert_edge_with_metadata(
                            &node_id,
                            &target_id,
                            rtype,
                            rel.weight,
                            Some(&rel.evidence),
                            Some(edge_meta),
                        ) {
                            dropped_edges += 1;
                            log::warn!(
                                "[kb-write] GraphRAG 边写入失败 {node_id}->{target_id}: {e}"
                            );
                        }
                        if dropped_edges > 0 {
                            log::warn!(
                                "[kb-write] 本次写入有 {dropped_edges} 条 GraphRAG 边未落库 —— 多跳检索将少召回"
                            );
                        }
                    }
                }
                let _ = entities.len(); // 实体已入 graphrag_store, 主库边来自关系
            }
        }

        Ok(node_id)
    }

    /// SVAF 写入门禁评估 (T0.4): 从 url/domain 推导 source_type, 走 content-only 门禁
    /// (廉价, 不触发全库 embedding 扫描)。无内容写入默认 Accept (标题型节点)。
    pub(crate) fn evaluate_write_gate(
        &self,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> nt_memory_svaf_gate::SvafEvaluation {
        let text = content.unwrap_or_default();
        if text.trim().is_empty() {
            return nt_memory_svaf_gate::SvafEvaluation {
                decision: nt_memory_svaf_gate::SvafDecision::Accept,
                novelty: 0.5,
                coherence: 0.5,
                relevance: 0.5,
                authority: 0.5,
                reason: "no content, default accept".into(),
            };
        }
        let source_type = derive_source_type(url, domain);
        self.gate_content_only(text, &source_type)
    }
}
