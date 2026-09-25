//! Community persist: `persist_to_kb*` materialization into KB.
//! Pure move from `nt_core_community_ingester.rs`; behavior unchanged.

use super::nt_community_data::CommunityDataIngester;

/// 20-hex md5 of a string, used to derive deterministic node/edge ids.
/// Mirrors the retired prototype `scripts/deep-absorb-fable5.py:ndig`. md5 here is used only for
/// storage-key derivation (not security).
fn qidian_hash(s: &str) -> String {
    use md5::{Digest, Md5};
    let mut h = Md5::new();
    h.update(s.as_bytes());
    let digest = h.finalize();
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    hex[..20].to_string()
}

impl CommunityDataIngester {
    /// Materialize the E8 community-dataset hub + per-dataset Concept nodes and
    /// edges into the KB. Faithful port of the retired `scripts/deep-absorb-fable5.py`:
    /// creates `community_e8_datasets_hub`, one `community_dataset_{name}` node
    /// per dataset, `contains` edges hub→dataset, `related` edges within themed
    /// groups, and cross-theme `related` edges with lower weight.
    ///
    /// Idempotent (INSERT OR IGNORE). Returns the number of nodes written.
    pub fn persist_to_kb(&self, conn: &rusqlite::Connection, now: i64) -> rusqlite::Result<usize> {
        let hub_id = "community_e8_datasets_hub";
        let hub_meta = serde_json::json!({
            "source": "fable5-absorption",
            "type": "dataset-hub",
            "count": self.datasets.len(),
            "domain": "community-datasets",
            "quality_score": 0.95,
        })
        .to_string();
        conn.execute(
            "INSERT OR IGNORE INTO nodes
             (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata)
             VALUES (?1,'Concept',?2,?3,'',?4,'neotrix.local','en',1.0,0.95,?5,?5,?6)",
            rusqlite::params![
                hub_id,
                "E8 Community Datasets",
                format!(
                    "Fable-5 / Open-SWE-Traces community datasets ({} datasets). Injected by Fable-5 deep absorption.",
                    self.datasets.len()
                ),
                "neotrix://community-datasets/e8",
                now,
                hub_meta,
            ],
        )?;

        let mut written = 1usize;
        let mut ids: std::collections::BTreeMap<&str, String> = std::collections::BTreeMap::new();
        for ds in &self.datasets {
            let ds_id = format!("community_dataset_{}", ds.name);
            let meta = serde_json::json!({
                "source": "fable5-absorption",
                "type": "community-dataset",
                "weight": ds.weight,
                "tags": [],
            })
            .to_string();
            conn.execute(
                "INSERT OR IGNORE INTO nodes
                 (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata)
                 VALUES (?1,'Concept',?2,?3,'',?4,'neotrix.local','en',1.0,?5,?6,?6,?7)",
                rusqlite::params![
                    ds_id,
                    ds.name,
                    ds.description,
                    format!("neotrix://community-datasets/{}", ds.name),
                    ds.weight,
                    now,
                    meta,
                ],
            )?;
            written += 1;
            ids.insert(&ds.name, ds_id);
        }

        // contains edges hub → dataset
        for ds in &self.datasets {
            let ds_id = &ids[ds.name.as_str()];
            let eid = format!("re-{}", qidian_hash(&format!("{hub_id}{ds_id}")));
            conn.execute(
                "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at)
                 VALUES (?1,?2,?3,'contains',?4,?5,?6)",
                rusqlite::params![
                    eid,
                    hub_id,
                    ds_id,
                    ds.weight,
                    format!("E8 Community Hub → {}", ds.name),
                    now,
                ],
            )?;
        }

        // related edges within themed groups (faithful group split)
        let ssm_papers = ["priming_hybrid_ssm_fable", "retrieval_aware_distill_ssm"];
        let fable_traces = [
            "fable5_sft_traces_kelexine_4k",
            "fable5_swarm_traces_sft_4k",
        ];
        let swe_related = [
            "nvidia_open_swe_traces_207k",
            "open_swe_agent_thinking_dual",
        ];
        for group in [&ssm_papers[..], &fable_traces[..], &swe_related[..]] {
            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    let (src, tgt) = (group[i], group[j]);
                    if let (Some(s), Some(t)) = (ids.get(src), ids.get(tgt)) {
                        let eid = format!("re-{}", qidian_hash(&format!("{s}{t}")));
                        conn.execute(
                            "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at)
                             VALUES (?1,?2,?3,'related',0.7,?4,?5)",
                            rusqlite::params![eid, s, t, format!("Thematic link: {src} ↔ {tgt}"), now],
                        )?;
                    }
                }
            }
        }

        // cross-theme edges with lower weight
        let cross: &[(&str, &str, f64, &str)] = &[
            (
                "nvidia_open_swe_traces_207k",
                "fable5_sft_traces_kelexine_4k",
                0.4,
                "SWE-bench ↔ Kelexine SFT",
            ),
            (
                "nvidia_open_swe_traces_207k",
                "fable5_swarm_traces_sft_4k",
                0.35,
                "SWE-bench ↔ Swarm-AI SFT",
            ),
            (
                "open_swe_agent_thinking_dual",
                "fable5_sft_traces_kelexine_4k",
                0.4,
                "Dual-mode ↔ Kelexine SFT",
            ),
            (
                "open_swe_agent_thinking_dual",
                "fable5_swarm_traces_sft_4k",
                0.4,
                "Dual-mode ↔ Swarm-AI SFT",
            ),
            (
                "priming_hybrid_ssm_fable",
                "fable5_sft_traces_kelexine_4k",
                0.3,
                "SSM ↔ Kelexine SFT",
            ),
            (
                "retrieval_aware_distill_ssm",
                "fable5_swarm_traces_sft_4k",
                0.3,
                "Distilled SSM ↔ Swarm-AI SFT",
            ),
        ];
        for (s, t, w, desc) in cross {
            if let (Some(src), Some(tgt)) = (ids.get(*s), ids.get(*t)) {
                let eid = format!("re-{}", qidian_hash(&format!("{src}{tgt}")));
                conn.execute(
                    "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at)
                     VALUES (?1,?2,?3,'related',?4,?5,?6)",
                    rusqlite::params![eid, src, tgt, w, desc, now],
                )?;
            }
        }

        Ok(written)
    }

    /// Persist the community dataset hub into the NeoTrix KnowledgeBase via
    /// the public `insert_node`/`insert_edge` API (idempotent, INSERT OR IGNORE).
    ///
    /// This is the production wiring that closes the "data → KB → 意识进化"
    /// loop: the 200G-scale community reasoning datasets (FABLE.5-2M,
    /// r1-distilled-100k, GLM-5.2-50k, ...) become real KB nodes/edges that
    /// the ConsciousnessTree soil can observe — instead of only seeding the
    /// E8 transition matrix. Returns the number of nodes written.
    pub fn persist_to_kb_store(
        &self,
        kb: &crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<usize, String> {
        use neotrix_types::knowledge_access::{KnowledgeNode, NodeType, RelationType};
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let hub_id = "community_e8_datasets_hub";
        let hub_meta = serde_json::json!({
            "source": "fable5-absorption",
            "type": "dataset-hub",
            "count": self.datasets.len(),
            "domain": "community-datasets",
            "quality_score": 0.95,
        });
        // 幂等: 已存在则跳过 (INSERT OR IGNORE 语义)
        if kb.get_node(hub_id).ok().flatten().is_none() {
            kb.insert_node(&KnowledgeNode {
                id: hub_id.into(),
                node_type: NodeType::Concept,
                title: "E8 Community Datasets".into(),
                summary: Some(format!(
                    "Community reasoning datasets ({} datasets, 200G+ traces). Injected by E8 community absorption.",
                    self.datasets.len()
                )),
                content: None,
                url: Some("neotrix://community-datasets/e8".into()),
                domain: Some("neotrix.local".into()),
                language: "en".to_string(),
                recall_weight: 1.0,
                confidence: 1.0,
                importance: 0.95,
                created_at: now,
                updated_at: now,
                access_count: 0,
                metadata: Some(hub_meta),
                temporal: None,
                supersedes: None,
                source_episode: None,
                parent_id: None,
                depth: 0,
                cluster_id: None,
            })?;
        }

        let mut written = 1usize;
        let mut ids: std::collections::BTreeMap<&str, String> = std::collections::BTreeMap::new();
        for ds in &self.datasets {
            let ds_id = format!("community_dataset_{}", ds.name);
            let meta = serde_json::json!({
                "source": "e8-absorption",
                "type": "community-dataset",
                "weight": ds.weight,
                "tags": [],
            });
            if kb.get_node(&ds_id).ok().flatten().is_none() {
                kb.insert_node(&KnowledgeNode {
                    id: ds_id.clone(),
                    node_type: NodeType::Dataset,
                    title: ds.name.clone(),
                    summary: Some(ds.description.clone()),
                    content: None,
                    url: Some(ds.source_url.clone()),
                    domain: Some("neotrix.local".into()),
                    language: "en".to_string(),
                    recall_weight: 1.0,
                    confidence: 1.0,
                    importance: ds.weight,
                    created_at: now,
                    updated_at: now,
                    access_count: 0,
                    metadata: Some(meta),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                    parent_id: None,
                    depth: 0,
                    cluster_id: None,
                })?;
            }
            written += 1;
            ids.insert(&ds.name, ds_id);
        }

        // contains edges hub → dataset (upsert_edge 幂等)
        for ds in &self.datasets {
            let ds_id = &ids[ds.name.as_str()];
            kb.upsert_edge(
                hub_id,
                ds_id,
                RelationType::References,
                ds.weight,
                Some(&format!("E8 Community Hub → {}", ds.name)),
            )?;
        }

        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_core_e8::nt_core_community_ingester::CommunityDataIngester;

    #[test]
    fn test_persist_to_kb_materializes_hub_and_datasets() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, summary TEXT, content TEXT, url TEXT, domain TEXT, language TEXT, confidence REAL, importance REAL, created_at INTEGER, updated_at INTEGER, metadata TEXT);
             CREATE TABLE edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, relation_type TEXT, weight REAL, description TEXT, created_at INTEGER);",
        )
        .unwrap();
        let ingester = CommunityDataIngester::default();
        let written = ingester.persist_to_kb(&conn, 1700000000).unwrap();
        assert!(written >= 1);

        let hub: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE id='community_e8_datasets_hub'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hub, 1);

        // The 6 deep-absorb-fable5 datasets must be materialized
        for name in [
            "nvidia_open_swe_traces_207k",
            "fable5_sft_traces_kelexine_4k",
            "fable5_swarm_traces_sft_4k",
            "priming_hybrid_ssm_fable",
            "retrieval_aware_distill_ssm",
            "open_swe_agent_thinking_dual",
        ] {
            let c: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes WHERE id=?1",
                    rusqlite::params![format!("community_dataset_{name}")],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(c, 1, "missing dataset node {name}");
        }

        // Idempotent second run
        let written2 = ingester.persist_to_kb(&conn, 1700000000).unwrap();
        assert_eq!(written2, written);
        let contains: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE relation_type='contains'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(contains >= 6);
    }

    #[test]
    fn test_persist_to_kb_store_writes_nodes_and_edges() {
        // 意识核心进化闭环 (数据→KB): persist_to_kb_store 应把社区数据集
        // 落盘为真实 KB 节点/边, 使 ConsciousnessTree soil 可观测。
        // 此前该方法无生产调用者, KB 全表 0 行。
        let tmp = std::env::temp_dir().join(format!("nt_kb_test_{}.db", std::process::id()));
        let kb = crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone()))
            .expect("open temp KB");
        let ingester = CommunityDataIngester::default();
        let n = ingester.persist_to_kb_store(&kb).expect("persist ok");
        // hub + 每个数据集 = 1 + datasets.len()
        assert_eq!(n, 1 + ingester.datasets.len(), "hub + all datasets written");
        // hub 节点存在
        let hub = kb
            .get_node("community_e8_datasets_hub")
            .expect("get hub")
            .expect("hub node");
        assert_eq!(
            hub.node_type,
            crate::l4_emotion::nt_memory::nt_memory_kb::NodeType::Concept
        );
        // 至少一个数据集节点存在且为 Dataset 类型
        let ds = kb
            .get_node(&format!("community_dataset_{}", ingester.datasets[0].name))
            .expect("get ds")
            .expect("first dataset node");
        assert_eq!(
            ds.node_type,
            crate::l4_emotion::nt_memory::nt_memory_kb::NodeType::Dataset
        );
        // 幂等: 再次落盘不重复
        let n2 = ingester.persist_to_kb_store(&kb).expect("persist again");
        assert_eq!(n2, 1 + ingester.datasets.len(), "idempotent");
        let _ = std::fs::remove_file(&tmp);
    }
}
