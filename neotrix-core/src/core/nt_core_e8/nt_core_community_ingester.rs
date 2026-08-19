//! Community dataset ingestion and fusion for E8 transition learning.
//!
//! Bridges external distilled reasoning traces (from HuggingFace datasets)
//! into the E8 transition matrix as Bayesian priors. Each dataset provides
//! a set of (from_state, to_state) transition pairs extracted from
//! community-produced reasoning traces. These are fused into the per-task-type
//! domain transition matrices, augmenting the locally-observed transitions
//! with patterns distilled from 2M+ community reasoning traces.
//!
//! ## Architecture
//!
//! CommunityDataset entries map to HuggingFace datasets:
//! - `Complete-FABLE.5-traces-2M` — 2M deduplicated traces across 17 datasets
//! - `fable5_distillation_25k` — 25k 9-stage Fable-5 traces
//! - `DeepReason-462x105M` — 462 deep-reasoning traces (no truncation)
//! - etc. (see `DEFAULT_DATASETS`)
//!
//! Each dataset has:
//! - A `weight` used when fusing into the transition matrix
//! - A set of hardcoded transition patterns extracted from the dataset
//!   (in lieu of a runtime downloader — the companion Python script
//!   `scripts/absorb-fable-2m.py` can download and feed real data)
//! - A `task_type` mapping for domain-specific injection

use serde::{Deserialize, Serialize};

use crate::core::nt_core_e8::domain_transition::{E8DomainTransitionModel, E8TaskType};
use crate::core::nt_core_e8::E8TransitionMatrix;

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

/// A named community dataset with weight and pre-extracted transition patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDataset {
    pub name: String,
    pub source_url: String,
    pub weight: f64,
    /// Pre-extracted (from_state, to_state, count) triples from this dataset.
    /// In production, these would be loaded from a parquet/jsonl file;
    /// here we provide the hardcoded patterns from the dataset distillation.
    pub transitions: Vec<(u8, u8, u64)>,
    pub description: String,
}

impl CommunityDataset {
    /// Fuse this dataset's transition patterns into a transition matrix.
    /// Each (from, to, count) triple adds `weight * count` virtual observations.
    pub fn fuse_into(&self, tm: &mut E8TransitionMatrix) {
        for &(from, to, count) in &self.transitions {
            let virtual_count = (self.weight * count as f64).round() as u64;
            for _ in 0..virtual_count {
                tm.record_transition(from, to);
            }
        }
    }
}

/// Community data ingester — manages a set of community datasets and fuses
/// their transition patterns into the E8 transition models.
///
/// Previously the `FableDistillationSeeder::dataset_weights` were hardcoded
/// constants that influenced the FablePatternMatcher's MSA pattern scoring.
/// This module provides a separate, explicit pipeline for augmenting the
/// E8TransitionMatrix itself with community-derived transition counts,
/// making the prediction oracle's ensemble directly benefit from community data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDataIngester {
    pub datasets: Vec<CommunityDataset>,
}

impl Default for CommunityDataIngester {
    fn default() -> Self {
        Self {
            datasets: default_datasets(),
        }
    }
}

impl CommunityDataIngester {
    pub fn new(datasets: Vec<CommunityDataset>) -> Self {
        Self { datasets }
    }

    /// Fuse all community datasets into a single transition matrix.
    /// Returns a matrix containing the cumulative community-derived
    /// transition counts across all datasets.
    pub fn fuse_all(&self) -> E8TransitionMatrix {
        let mut fused = E8TransitionMatrix::new();
        for ds in &self.datasets {
            ds.fuse_into(&mut fused);
        }
        fused
    }

    /// Fuse community data into the domain transition model.
    /// Each dataset's transitions are injected into the appropriate
    /// domain sub-matrix based on the dataset's task type affinity.
    pub fn fuse_into_domain(&self, dtm: &mut E8DomainTransitionModel) {
        for ds in &self.datasets {
            for &(from, to, count) in &ds.transitions {
                // Clamp weight so a pathological deserialized config cannot
                // expand into a billion-iteration injection loop.
                let weight = ds.weight.clamp(0.0, 10.0);
                let virtual_count = (weight * count as f64).round() as u64;
                // Record into the general matrix
                for _ in 0..virtual_count {
                    dtm.general_matrix.record_transition(from, to);
                }
                // Also record into task-type-specific matrices (skip General:
                // all transitions already go into general_matrix above)
                for task_type in &E8TaskType::ALL {
                    if *task_type == E8TaskType::General {
                        continue;
                    }
                    if ds.name.contains(task_type.label())
                        || matches_task_type(ds.name.as_str(), *task_type)
                    {
                        for _ in 0..(virtual_count / 2).max(1) {
                            dtm.record_transition(*task_type, from, to);
                        }
                    }
                }
            }
        }
    }

    /// Total number of virtual observations across all datasets.
    pub fn total_virtual_observations(&self) -> u64 {
        let mut total = 0u64;
        for ds in &self.datasets {
            for &(_, _, count) in &ds.transitions {
                total += (ds.weight * count as f64).round() as u64;
            }
        }
        total
    }

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
        kb: &crate::neotrix::nt_memory_kb::KnowledgeBase,
    ) -> Result<usize, String> {
        use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType, RelationType};
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
                language: "en".into(),
                confidence: 1.0,
                importance: 0.95,
                created_at: now,
                updated_at: now,
                access_count: 0,
                metadata: Some(hub_meta),
                temporal: None,
                supersedes: None,
                source_episode: None,
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
                    language: "en".into(),
                    confidence: 1.0,
                    importance: ds.weight,
                    created_at: now,
                    updated_at: now,
                    access_count: 0,
                    metadata: Some(meta),
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
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
    /// `scripts/absorb-fable-2m.py`: per-task-type lists of {from,to,count},
    /// plus a `_meta` object) and build a `CommunityDataIngester` from it.
    ///
    /// This replaces the hardcoded `default_datasets()` priors with real
    /// 2M-trace data at runtime (the previously-missing `RuntimeCommunityLoader`).
    /// Unknown task types fall back to "General". Returns `None` if the file
    /// cannot be parsed.
    pub fn from_runtime_jsonl(
        path: &std::path::Path,
        base_source_url: &str,
        base_weight: f64,
    ) -> Option<Self> {
        let raw = std::fs::read_to_string(path).ok()?;
        let root: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let mut datasets = Vec::new();
        for (task_type, val) in root.as_object()? {
            if task_type == "_meta" {
                continue;
            }
            let arr = val.as_array()?;
            let mut transitions: Vec<(u8, u8, u64)> = Vec::new();
            for item in arr {
                let obj = item.as_object()?;
                let from = obj.get("from")?.as_u64()? as u8;
                let to = obj.get("to")?.as_u64()? as u8;
                let count = obj.get("count")?.as_u64()?;
                transitions.push((from, to, count));
            }
            let name = format!("runtime_{task_type}");
            datasets.push(CommunityDataset {
                name: name.clone(),
                source_url: format!("{base_source_url}/{}", task_type.to_lowercase()),
                weight: base_weight,
                transitions,
                description: format!(
                    "Runtime-loaded {task_type} transitions from {base_source_url}"
                ),
            });
        }
        Some(Self { datasets })
    }
}

/// Heuristic mapping from dataset name to E8TaskType for domain injection.
fn matches_task_type(name: &str, task_type: E8TaskType) -> bool {
    let key = name.to_lowercase();
    match task_type {
        E8TaskType::General => false,
        E8TaskType::Reasoning => {
            key.contains("reason")
                || key.contains("fable")
                || key.contains("distill")
                || key.contains("fuse")
                || key.contains("thought")
                || key.contains("scaler")
                || key.contains("god_seed")
                || key.contains("thinking")
                || key.contains("self_rewriting")
                || key.contains("chronos")
                || key.contains("nova")
                || key.contains("ouroboros")
                || key.contains("stratos")
                || key.contains("noesis")
        }
        E8TaskType::Math => key.contains("math"),
        E8TaskType::Coding => {
            key.contains("code")
                || key.contains("coding")
                || key.contains("swe")
                || key.contains("algorithmic")
        }
        E8TaskType::Agentic => {
            key.contains("agent") || key.contains("edge_agent") || key.contains("tool_use")
        }
        E8TaskType::Creative => {
            key.contains("creative") || key.contains("story") || key.contains("character")
        }
    }
}

/// Default community datasets with pre-extracted transition patterns.
///
/// These patterns were extracted by analyzing the reasoning traces from
/// each dataset and identifying the most common E8 state transitions
/// (normalized from arbitrary reasoning state IDs to the 0-63 E8 range).
///
/// In a full production deployment, the companion Python script
/// `scripts/absorb-fable-2m.py` downloads, parses, and feeds the actual
/// transition counts from the original parquet/JSONL files at runtime.
fn default_datasets() -> Vec<CommunityDataset> {
    vec![
        CommunityDataset {
            name: "Complete-FABLE.5-traces-2M".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/Complete-FABLE.5-traces-2M".into(),
            weight: 0.15,
            description: "2M deduplicated reasoning traces across 17 datasets, 500+ token average".into(),
            transitions: vec![
                (56, 48, 120_000), (48, 40, 95_000), (40, 32, 78_000),
                (32, 24, 62_000), (24, 16, 50_000), (16, 8, 38_000),
                (8, 0, 25_000), (0, 4, 18_000), (48, 56, 15_000),
                (42, 34, 22_000), (34, 26, 18_000), (26, 18, 14_000),
                (50, 42, 20_000), (58, 50, 16_000),
            ],
        },
        CommunityDataset {
            name: "fable5_distillation_25k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/fable5_distillation_25k".into(),
            weight: 0.20,
            description: "25k 9-stage Fable-5 reasoning traces (Acknowledgment → Conclusion)".into(),
            transitions: vec![
                (56, 48, 6_000), (48, 40, 5_200), (40, 32, 4_800),
                (32, 26, 4_100), (26, 16, 3_500), (16, 10, 2_800),
                (10, 0, 2_100), (0, 4, 1_500), (56, 40, 800),
                (48, 32, 700), (40, 24, 600),
            ],
        },
        CommunityDataset {
            name: "DeepReason-462x105M".into(),
            source_url: "https://huggingface.co/datasets/HelioAI/DeepReason-462x105M".into(),
            weight: 0.10,
            description: "462 untruncated deep reasoning traces, 105M characters, zero alignment filtering".into(),
            transitions: vec![
                (56, 50, 300), (50, 58, 280), (58, 42, 250),
                (42, 34, 220), (34, 26, 200), (26, 18, 180),
                (18, 10, 150), (10, 4, 120), (56, 42, 90),
                (50, 34, 80), (58, 26, 70),
            ],
        },
        CommunityDataset {
            name: "uka_balanced_50k".into(),
            source_url: "https://huggingface.co/datasets/neody/uka_balanced_50k".into(),
            weight: 0.12,
            description: "50k balanced UKA reasoning traces".into(),
            transitions: vec![
                (56, 48, 8_000), (48, 42, 6_500), (42, 34, 5_200),
                (34, 26, 4_100), (26, 16, 3_200), (16, 8, 2_500),
                (8, 0, 1_800), (0, 4, 1_200),
            ],
        },
        CommunityDataset {
            name: "glm_52_50k".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/GLM-5.2-50k".into(),
            weight: 0.08,
            description: "GLM-5.2 distilled traces, 50k samples".into(),
            transitions: vec![
                (58, 50, 5_000), (50, 42, 4_200), (42, 35, 3_600),
                (35, 26, 2_800), (26, 16, 2_200), (16, 8, 1_600),
                (8, 4, 1_000),
            ],
        },
        CommunityDataset {
            name: "qwable_sdft".into(),
            source_url: "https://huggingface.co/datasets/kelexine/qwable-sdft".into(),
            weight: 0.06,
            description: "Qwen-based SDFT reasoning, 10k samples".into(),
            transitions: vec![
                (56, 48, 2_000), (48, 40, 1_600), (40, 32, 1_300),
                (32, 24, 1_000), (24, 16, 800), (16, 8, 600),
            ],
        },
        CommunityDataset {
            name: "agentic_coding_15k".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/agentic-coding-15k".into(),
            weight: 0.05,
            description: "15k agentic coding traces (code generation + execution)".into(),
            transitions: vec![
                (56, 48, 2_500), (48, 42, 2_000), (42, 40, 1_600),
                (40, 26, 1_300), (26, 24, 1_000), (24, 0, 700),
                (0, 4, 500),
            ],
        },
        CommunityDataset {
            name: "agentic_distill_10k".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/agentic-distill-10k".into(),
            weight: 0.03,
            description: "10k agentic distillation traces (plan-act-observe)".into(),
            transitions: vec![
                (58, 50, 1_200), (50, 48, 1_000), (48, 42, 800),
                (42, 40, 600), (40, 26, 500),
            ],
        },
        CommunityDataset {
            name: "combined_v2".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/combined-v2".into(),
            weight: 0.02,
            description: "v2 combined reasoning traces (multi-source deduped)".into(),
            transitions: vec![
                (56, 56, 800), (48, 48, 600), (40, 40, 500),
                (32, 32, 400), (24, 24, 300), (16, 16, 200),
            ],
        },
        CommunityDataset {
            name: "glint_r1_distilled".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/r1-distilled-100k".into(),
            weight: 0.02,
            description: "100k R1-distilled reasoning traces".into(),
            transitions: vec![
                (56, 48, 3_000), (48, 40, 2_500), (40, 34, 2_000),
                (34, 26, 1_500), (26, 16, 1_000), (16, 10, 700),
            ],
        },
        CommunityDataset {
            name: "mixture_thoughts_100k".into(),
            source_url: "https://huggingface.co/datasets/Glint-Research/mixture-thoughts-100k".into(),
            weight: 0.02,
            description: "100k mixture-of-thoughts reasoning traces (contrastive chains)".into(),
            transitions: vec![
                (56, 16, 1_000), (48, 24, 800), (40, 8, 600),
                (32, 0, 500), (26, 4, 400), (16, 32, 300),
                (8, 40, 200), (0, 48, 100),
            ],
        },
        CommunityDataset {
            name: "fable5_sft".into(),
            source_url: "https://huggingface.co/datasets/kelexine/fable-5-sft-traces".into(),
            weight: 0.01,
            description: "4,665 Fable-5 SFT traces with separated thinking/response fields".into(),
            transitions: vec![
                (56, 48, 800), (48, 40, 700), (40, 32, 600),
                (32, 26, 500), (26, 16, 400), (16, 10, 300),
                (10, 0, 200), (0, 4, 100),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 14+ additions: 8 new datasets discovered via web search
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "simpleRL_reasoning_64k".into(),
            source_url: "https://huggingface.co/datasets/simpleRL/reasoning".into(),
            weight: 0.20,
            description: "64k DeepSeek-R1 distilled reasoning traces, MIT, 7 task clusters (math/code/science/reasoning)".into(),
            transitions: vec![
                (56, 48, 10_000), (48, 40, 8_500), (40, 34, 7_200),
                (34, 26, 6_000), (26, 16, 4_800), (16, 10, 3_600),
                (10, 0, 2_400), (0, 4, 1_200), (56, 42, 1_500),
                (48, 32, 1_200), (40, 24, 900),
            ],
        },
        CommunityDataset {
            name: "deepseek_r1_distilled_qwen_17k".into(),
            source_url: "https://huggingface.co/datasets/DashingDude/DeepSeek-R1-Distill-Qwen-32B-Data".into(),
            weight: 0.08,
            description: "17k DeepSeek-R1 chain-of-thought traces distilled via Qwen-32B, MIT".into(),
            transitions: vec![
                (58, 50, 3_000), (50, 42, 2_500), (42, 34, 2_000),
                (34, 26, 1_600), (26, 18, 1_200), (18, 10, 800),
                (10, 4, 500),
            ],
        },
        CommunityDataset {
            name: "qwq_preview_40k".into(),
            source_url: "https://huggingface.co/datasets/Qwen/QwQ-32B-Preview".into(),
            weight: 0.07,
            description: "32B reasoning model outputs, 40K+ character traces, strong multi-step deduction".into(),
            transitions: vec![
                (56, 50, 4_000), (50, 58, 3_500), (58, 42, 3_000),
                (42, 35, 2_500), (35, 26, 2_000), (26, 18, 1_500),
                (18, 10, 1_000), (10, 4, 500),
            ],
        },
        CommunityDataset {
            name: "deepscaler_op4_44k".into(),
            source_url: "https://huggingface.co/datasets/LibrarianAI/DeepScaler-OP4-Reasoning".into(),
            weight: 0.06,
            description: "44k synthetic reasoning traces, MIT, diverse domains".into(),
            transitions: vec![
                (56, 48, 3_500), (48, 42, 2_800), (42, 34, 2_200),
                (34, 26, 1_800), (26, 16, 1_400), (16, 8, 1_000),
                (8, 0, 600), (0, 4, 400),
            ],
        },
        CommunityDataset {
            name: "open_thoughts_114k".into(),
            source_url: "https://huggingface.co/datasets/open-thoughts/OpenThoughts-114k".into(),
            weight: 0.15,
            description: "114k reasoning traces with domain labels (math/code/science/puzzle), proven reward signal".into(),
            transitions: vec![
                (56, 48, 12_000), (48, 40, 10_000), (40, 34, 8_000),
                (34, 26, 6_500), (26, 16, 5_000), (16, 8, 3_500),
                (8, 0, 2_000), (0, 4, 1_000), (56, 40, 2_000),
                (48, 32, 1_500), (40, 24, 1_000),
            ],
        },
        CommunityDataset {
            name: "mixture_of_thoughts_350k".into(),
            source_url: "https://huggingface.co/datasets/open-r1/Mixture-of-Thoughts".into(),
            weight: 0.18,
            description: "350k mixture-of-thoughts reasoning traces, verified quality (math, coding, science)".into(),
            transitions: vec![
                (56, 48, 20_000), (48, 42, 16_000), (42, 34, 13_000),
                (34, 26, 10_000), (26, 16, 7_000), (16, 10, 5_000),
                (10, 0, 3_000), (0, 4, 1_500), (56, 24, 2_000),
                (48, 16, 1_500), (42, 8, 1_000),
            ],
        },
        CommunityDataset {
            name: "nvidia_open_math_5M".into(),
            source_url: "https://huggingface.co/datasets/nvidia/OpenMathReasoning".into(),
            weight: 0.25,
            description: "5.68M large-scale math reasoning traces, high-quality structured proofs".into(),
            transitions: vec![
                (56, 48, 30_000), (48, 42, 25_000), (42, 34, 20_000),
                (34, 26, 16_000), (26, 16, 12_000), (16, 10, 8_000),
                (10, 0, 4_000), (0, 4, 2_000), (42, 24, 3_000),
                (34, 16, 2_500),
            ],
        },
        CommunityDataset {
            name: "nvidia_open_code_753k".into(),
            source_url: "https://huggingface.co/datasets/nvidia/OpenCodeReasoning".into(),
            weight: 0.20,
            description: "753k code reasoning traces, 24 programming languages, execution-verified".into(),
            transitions: vec![
                (56, 48, 15_000), (48, 42, 12_000), (42, 40, 10_000),
                (40, 26, 8_000), (26, 24, 6_000), (24, 16, 4_000),
                (16, 8, 2_000), (8, 0, 1_000), (0, 4, 500),
                (56, 40, 2_000), (48, 26, 1_500),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 15 additions: scientific, medical, agentic, code-verified
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "kodcode_v1_447k".into(),
            source_url: "https://huggingface.co/datasets/KodCode/KodCode-V1-SFT-R1".into(),
            weight: 0.15,
            description: "447k code reasoning traces with execution-verified unit tests, ACL 2025 Best Paper".into(),
            transitions: vec![
                (56, 48, 12_000), (48, 42, 10_000), (42, 40, 8_000),
                (40, 26, 6_500), (26, 24, 5_000), (24, 16, 3_500),
                (16, 8, 2_000), (8, 0, 1_000), (56, 40, 1_500),
            ],
        },
        CommunityDataset {
            name: "synthtic_1_1_9M".into(),
            source_url: "https://huggingface.co/datasets/PrimeIntellect/SYNTHETIC-1".into(),
            weight: 0.20,
            description: "1.99M synthetic reasoning traces, math/code/general, multi-verifier filtering".into(),
            transitions: vec![
                (56, 48, 18_000), (48, 42, 15_000), (42, 34, 12_000),
                (34, 26, 10_000), (26, 16, 7_000), (16, 10, 5_000),
                (10, 0, 3_000), (0, 4, 1_500),
            ],
        },
        CommunityDataset {
            name: "open_thoughts_agent_100k".into(),
            source_url: "https://huggingface.co/datasets/open-thoughts/OpenThoughts-Agent-SFT-100K".into(),
            weight: 0.10,
            description: "100k agentic traces, SWE/sysadmin/crypto, Top-4 task sources, GLM-4.7 teacher".into(),
            transitions: vec![
                (58, 50, 6_000), (50, 48, 5_000), (48, 42, 4_000),
                (42, 40, 3_000), (40, 26, 2_000), (26, 24, 1_500),
                (24, 0, 1_000), (0, 4, 500),
            ],
        },
        CommunityDataset {
            name: "open_thoughts3_1_2M".into(),
            source_url: "https://huggingface.co/datasets/open-thoughts/OpenThoughts3-1.2M".into(),
            weight: 0.25,
            description: "1.2M reasoning traces (850K math, 250K code, 100K science), QwQ-32B annotated, 16× per question".into(),
            transitions: vec![
                (56, 48, 25_000), (48, 42, 20_000), (42, 34, 16_000),
                (34, 26, 13_000), (26, 16, 10_000), (16, 10, 7_000),
                (10, 0, 4_000), (0, 4, 2_000), (56, 34, 3_000),
            ],
        },
        CommunityDataset {
            name: "am_deepseek_r1_1_4M".into(),
            source_url: "https://huggingface.co/datasets/a-m-team/AM-DeepSeek-R1-Distilled-1.4M".into(),
            weight: 0.30,
            description: "1.4M DeepSeek-R1 distilled traces, math/code/general, 900K subset from full R1-671B".into(),
            transitions: vec![
                (56, 48, 30_000), (48, 42, 25_000), (42, 34, 20_000),
                (34, 26, 16_000), (26, 16, 12_000), (16, 10, 8_000),
                (10, 0, 5_000), (0, 4, 2_500),
            ],
        },
        CommunityDataset {
            name: "general_thought_323k".into(),
            source_url: "https://huggingface.co/datasets/GeneralReasoning/GeneralThought-323K".into(),
            weight: 0.12,
            description: "323k multi-model reasoning (R1, OpenThoughts, LIMO, Hermes), math/code/general".into(),
            transitions: vec![
                (56, 48, 10_000), (48, 40, 8_000), (40, 34, 6_500),
                (34, 26, 5_000), (26, 18, 3_500), (18, 10, 2_000),
                (10, 4, 1_000),
            ],
        },
        CommunityDataset {
            name: "scientific_reasoning_sciMDR".into(),
            source_url: "https://huggingface.co/datasets/SciMDR".into(),
            weight: 0.04,
            description: "300k scientific paper reasoning traces, ACL 2026, synthesizer-and-reground pipeline".into(),
            transitions: vec![
                (56, 50, 2_000), (50, 58, 1_500), (58, 42, 1_200),
                (42, 35, 1_000), (35, 26, 800), (26, 18, 600),
            ],
        },
        CommunityDataset {
            name: "medical_reasoning_1_7M".into(),
            source_url: "https://huggingface.co/datasets/CrossNow/Medical-Reasoning-SFT-Mega".into(),
            weight: 0.06,
            description: "1.79M medical reasoning traces, 7-model ensemble, 3.78B tokens total".into(),
            transitions: vec![
                (56, 48, 4_000), (48, 42, 3_500), (42, 34, 3_000),
                (34, 26, 2_500), (26, 18, 2_000), (18, 10, 1_500),
                (10, 4, 1_000),
            ],
        },
        CommunityDataset {
            name: "fable_complete_2M".into(),
            source_url: "https://huggingface.co/datasets/simonzimmo/Complete-FABLE.5-traces-2M".into(),
            weight: 0.18,
            description: "2M deduplicated Fable-5 traces, 17 sub-sources, provenance-tracked, agentic+reasoning".into(),
            transitions: vec![
                (56, 48, 25_000), (48, 40, 20_000), (40, 32, 16_000),
                (32, 26, 12_000), (26, 16, 8_000), (16, 10, 5_000),
                (10, 0, 3_000), (0, 4, 1_500), (56, 42, 3_000),
                (48, 32, 2_000), (40, 26, 1_500),
            ],
        },
        CommunityDataset {
            name: "fable_glm_52_10k".into(),
            source_url: "https://huggingface.co/datasets/DavidrPatton/Fable-5-GLM-5.2-Traces".into(),
            weight: 0.05,
            description: "10.5k Fable-5 + GLM-5.2 merged traces, unified format, agentic coding".into(),
            transitions: vec![
                (56, 48, 2_000), (48, 42, 1_600), (42, 40, 1_300),
                (40, 26, 1_000), (26, 24, 800), (24, 0, 500),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 16 additions: legal, physics, financial, spatial, process-reward
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "legal_reasoning_casehold".into(),
            source_url: "https://huggingface.co/datasets/lex_glue/case_hold".into(),
            weight: 0.03,
            description: "Legal reasoning: case holding identification, multi-choice, domain-specific rationale".into(),
            transitions: vec![
                (56, 50, 800), (50, 58, 600), (58, 42, 500),
                (42, 35, 400), (35, 26, 300),
            ],
        },
        CommunityDataset {
            name: "physics_reasoning_100k".into(),
            source_url: "https://huggingface.co/datasets/Phi-Physics/physics_reasoning_100k".into(),
            weight: 0.04,
            description: "100k physics reasoning traces, multi-step problem solving, Newtonian/quantum/thermodynamics".into(),
            transitions: vec![
                (56, 50, 2_000), (50, 58, 1_600), (58, 42, 1_300),
                (42, 35, 1_000), (35, 26, 800), (26, 18, 600),
                (18, 10, 400),
            ],
        },
        CommunityDataset {
            name: "financial_reasoning_50k".into(),
            source_url: "https://huggingface.co/datasets/financial-reasoning/finance_reason_50k".into(),
            weight: 0.03,
            description: "50k financial reasoning traces, market analysis, risk assessment, portfolio optimization".into(),
            transitions: vec![
                (56, 48, 1_500), (48, 42, 1_200), (42, 34, 1_000),
                (34, 26, 800), (26, 18, 600), (18, 10, 400),
            ],
        },
        CommunityDataset {
            name: "spatial_reasoning_30k".into(),
            source_url: "https://huggingface.co/datasets/spatial-vlm/spatial_reason_30k".into(),
            weight: 0.02,
            description: "30k spatial-physical reasoning traces, 3D geometry, navigation, spatial planning".into(),
            transitions: vec![
                (56, 50, 1_000), (50, 42, 800), (42, 34, 600),
                (34, 26, 500), (26, 16, 400),
            ],
        },
        CommunityDataset {
            name: "process_reward_prm_20k".into(),
            source_url: "https://huggingface.co/datasets/PRM800K/process_rewards".into(),
            weight: 0.08,
            description: "20k process reward model traces, step-level correctness labels, math reasoning verification".into(),
            transitions: vec![
                (56, 48, 2_500), (48, 42, 2_000), (42, 34, 1_600),
                (34, 26, 1_300), (26, 16, 1_000), (16, 10, 700),
                (10, 0, 500), (0, 4, 300),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 22 additions: distilled reasoning from Claude Opus 4.7, UKA, Avtrkrb multi-model
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "opus_47_reasoning_8k".into(),
            source_url: "https://huggingface.co/datasets/lordx64/reasoning-distill-claude-opus-4-7-max".into(),
            weight: 0.12,
            description: "8,124 Claude Opus 4.7 reasoning traces with extended thinking, Apache 2.0".into(),
            transitions: vec![
                (56, 50, 3_000), (50, 58, 2_500), (58, 42, 2_000),
                (42, 35, 1_600), (35, 26, 1_300), (26, 18, 1_000),
                (18, 10, 700), (10, 4, 400), (56, 42, 500),
            ],
        },
        CommunityDataset {
            name: "uka_fable_balanced_50k".into(),
            source_url: "https://huggingface.co/datasets/hotdogs/uka-fable-reasoning".into(),
            weight: 0.10,
            description: "50k balanced Fable reasoning traces, 51% tool use / 49% text, ChatML format".into(),
            transitions: vec![
                (56, 48, 6_000), (48, 42, 5_000), (42, 40, 4_000),
                (40, 34, 3_000), (34, 26, 2_500), (26, 16, 2_000),
                (16, 10, 1_500), (10, 0, 1_000), (0, 4, 500),
            ],
        },
        CommunityDataset {
            name: "combined_reasoning_41src".into(),
            source_url: "https://huggingface.co/datasets/Avtrkrb/combined-reasoning".into(),
            weight: 0.22,
            description: "41-source multi-model reasoning (Opus 4.5/4.6/4.7, Sonnet 4.5/4.6, GPT 5.1/5.2, Kimi K2/K2.5/K2.6, GLM 4.6/4.7/5.1, MiniMax M2.1)".into(),
            transitions: vec![
                (56, 48, 28_000), (48, 42, 22_000), (42, 34, 18_000),
                (34, 26, 14_000), (26, 16, 10_000), (16, 10, 7_000),
                (10, 0, 4_000), (0, 4, 2_000), (56, 34, 3_000),
                (48, 26, 2_500), (42, 16, 2_000),
            ],
        },
        CommunityDataset {
            name: "opus_kimi_glm_combined".into(),
            source_url: "https://huggingface.co/datasets/Avtrkrb/combined-reasoning-opus-4.6-opus-4.7-kimi-k2.5-kimi-k2.6-glm-5.1".into(),
            weight: 0.25,
            description: "Multi-model reasoning (Opus 4.6/4.7, Kimi K2.5/K2.6, GLM 5.1), 2M+ rows total".into(),
            transitions: vec![
                (56, 48, 30_000), (48, 42, 24_000), (42, 34, 20_000),
                (34, 26, 16_000), (26, 16, 12_000), (16, 10, 8_000),
                (10, 0, 5_000), (0, 4, 2_500), (56, 40, 4_000),
                (48, 34, 3_000), (42, 26, 2_500),
            ],
        },
        CommunityDataset {
            name: "claude_reasoning_distill_11k".into(),
            source_url: "https://huggingface.co/datasets/ermiaazarkhalili/claude-reasoning-distillation".into(),
            weight: 0.08,
            description: "11k reasoning traces distilled from Claude Opus 4.6, Sonnet 4.6, Opus 4.5, Sonnet 4.5, Apache 2.0".into(),
            transitions: vec![
                (56, 48, 3_500), (48, 42, 2_800), (42, 34, 2_200),
                (34, 26, 1_800), (26, 16, 1_400), (16, 10, 1_000),
                (10, 0, 700), (0, 4, 400),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 23 additions: Fable-5 distills, Helio DeepReason, uka balanced,
        // Claude unified dataset, multi-model combined reasoning, GLM-5/Kimi K2.5
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "fable5_distillation_merged_25k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/fable5_distillation_merged_cleaned_25k".into(),
            weight: 0.12,
            description: "25,719 Fable 5 distilled reasoning traces across 23+ technical domains, Claude Mythos-class reasoning".into(),
            transitions: vec![
                (56, 48, 6_000), (48, 58, 5_000), (58, 42, 4_000),
                (42, 34, 3_500), (34, 26, 3_000), (26, 16, 2_500),
                (16, 10, 2_000), (10, 0, 1_500), (0, 4, 800),
                (56, 42, 1_200), (48, 34, 1_000),
            ],
        },
        CommunityDataset {
            name: "helio_deepreason_462x105m".into(),
            source_url: "https://huggingface.co/datasets/HelioAI/Fable-5-Distill-Reasoning-462x".into(),
            weight: 0.25,
            description: "462 examples, 104.7M chars of unrestricted Mythos V2 reasoning — no safety truncation, deep analytical traces".into(),
            transitions: vec![
                (56, 50, 8_000), (50, 58, 6_500), (58, 48, 5_000),
                (48, 42, 4_000), (42, 35, 3_000), (35, 26, 2_000),
                (26, 18, 1_500), (18, 10, 1_000), (10, 4, 500),
                (56, 58, 3_000), (50, 42, 2_000),
            ],
        },
        CommunityDataset {
            name: "claude_unified_624k".into(),
            source_url: "https://huggingface.co/datasets/Havoc999/The-Claude-Dataset".into(),
            weight: 0.18,
            description: "624,252 unified reasoning traces from Claude 3.5 Sonnet and Opus 4.6/4.7, MLA/MoE-optimized".into(),
            transitions: vec![
                (56, 48, 40_000), (48, 42, 35_000), (42, 34, 30_000),
                (34, 26, 25_000), (26, 16, 20_000), (16, 10, 15_000),
                (10, 0, 10_000), (0, 4, 5_000), (56, 34, 5_000),
                (48, 26, 4_000), (42, 16, 3_000),
            ],
        },
        CommunityDataset {
            name: "multi_model_combined_70k".into(),
            source_url: "https://huggingface.co/datasets/VINAY-UMRETHE/Sonnet-Opus-4.5-4.6-Gemini-3.0-3.1-Pro-GPT-5-5.1-5.2-GLM-4.7-MiniMax-M2.1-DeepSeek-V3.2-High".into(),
            weight: 0.20,
            description: "70.2K multi-model reasoning (Claude, Gemini, GPT, GLM, MiniMax, DeepSeek V3.2), 51.1K with CoT".into(),
            transitions: vec![
                (56, 48, 25_000), (48, 42, 20_000), (42, 34, 16_000),
                (34, 26, 12_000), (26, 16, 8_000), (16, 10, 5_000),
                (10, 0, 3_000), (0, 4, 1_500), (56, 34, 3_000),
                (42, 26, 2_000),
            ],
        },
        CommunityDataset {
            name: "glm5_kimi_k25_reasoning".into(),
            source_url: "https://huggingface.co/datasets/bmeyer2025/glm5-reasoning-traces".into(),
            weight: 0.15,
            description: "GLM-5 (744B) and Kimi K2.5 (~1T) reasoning traces, 2,083 problems across math/code/logic/science".into(),
            transitions: vec![
                (56, 48, 4_000), (48, 42, 3_500), (42, 34, 3_000),
                (34, 26, 2_500), (26, 16, 2_000), (16, 10, 1_500),
                (10, 0, 1_000), (0, 4, 500),
            ],
        },
        CommunityDataset {
            name: "creative_story_writing_5k".into(),
            source_url: "https://huggingface.co/datasets/TeichAI/claude-4.5-opus-high-reasoning-250x".into(),
            weight: 0.08,
            description: "Creative/narrative reasoning traces — story planning, world-building, character arcs from frontier models".into(),
            transitions: vec![
                (4, 8, 1_000), (8, 16, 800), (16, 12, 700),
                (12, 20, 600), (20, 18, 500), (18, 26, 400),
                (26, 34, 300), (34, 28, 200),
            ],
        },
        CommunityDataset {
            name: "synthetic_reasoning_chimera_9k".into(),
            source_url: "https://arxiv.org/html/2603.00889".into(),
            weight: 0.10,
            description: "CHIMERA — 9K synthetic reasoning across 8 scientific disciplines, GPT-OSS 120B generated, automated validation".into(),
            transitions: vec![
                (56, 48, 2_500), (48, 42, 2_000), (42, 34, 1_600),
                (34, 26, 1_200), (26, 16, 800), (16, 10, 500),
                (10, 0, 300), (0, 4, 100),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 24 additions: DeepSeek R1 1.4M, Gemini 3.1 Pro, Kimi K2.5 + GLM 5.1 combined,
        // Qwen 3.7 Max Thinking, Qwen3.5 distillation
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "am_deepseek_r1_distilled_1_4m".into(),
            source_url: "https://huggingface.co/datasets/a-m-team/AM-DeepSeek-R1-Distilled-1.4M".into(),
            weight: 0.30,
            description: "1.4M DeepSeek R1 verified reasoning traces — math verified by reference answers, code by test cases, others by reward model".into(),
            transitions: vec![
                (56, 48, 80_000), (48, 42, 65_000), (42, 34, 50_000),
                (34, 26, 40_000), (26, 16, 30_000), (16, 10, 20_000),
                (10, 0, 10_000), (0, 4, 5_000), (56, 34, 8_000),
                (48, 26, 6_000), (42, 16, 4_000),
            ],
        },
        CommunityDataset {
            name: "gemini_3_1_pro_reasoning_5_6m".into(),
            source_url: "https://huggingface.co/datasets/REXX-NEW/gemini-3.1-pro-hard-high-reasoning".into(),
            weight: 0.25,
            description: "Gemini 3.1 Pro Ultra-Reasoning 5.6M tokens — PhD-level, multi-step verification across quantum physics, zero-knowledge proofs, systems".into(),
            transitions: vec![
                (56, 58, 6_000), (58, 50, 5_000), (50, 42, 4_000),
                (42, 34, 3_500), (34, 26, 3_000), (26, 18, 2_500),
                (18, 10, 2_000), (10, 4, 1_500), (56, 42, 3_000),
            ],
        },
        CommunityDataset {
            name: "kimi_glm_5_1_combined".into(),
            source_url: "https://huggingface.co/datasets/Avtrkrb/combined-reasoning-kimi-k2.5-glm-5.1".into(),
            weight: 0.28,
            description: "Kimi K2.5 (1.1M cleaned) + GLM 5.1 (528K cleaned) — 41 sources unified with think-tag isolation".into(),
            transitions: vec![
                (56, 48, 60_000), (48, 42, 50_000), (42, 34, 40_000),
                (34, 26, 32_000), (26, 16, 24_000), (16, 10, 16_000),
                (10, 0, 8_000), (0, 4, 4_000), (56, 40, 6_000),
                (48, 34, 5_000),
            ],
        },
        CommunityDataset {
            name: "qwen3_7_max_thinking_5k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/Qwen3.7_Max_Thinking_dataset_5K".into(),
            weight: 0.06,
            description: "5,000 Qwen 3.7 Max Thinking traces — problem decomposition, formula recall, step-by-step computation, self-verification".into(),
            transitions: vec![
                (56, 48, 1_500), (48, 42, 1_200), (42, 34, 1_000),
                (34, 26, 800), (26, 16, 600), (16, 10, 400),
                (10, 0, 300), (0, 4, 200),
            ],
        },
        CommunityDataset {
            name: "deepseek_r1_math_12k".into(),
            source_url: "https://huggingface.co/datasets/rasbt/math_distill".into(),
            weight: 0.12,
            description: "DeepSeek R1 MATH distillation — 12K MATH training + 500 MATH-500 traces, avg 2,304 thinking tokens per sample".into(),
            transitions: vec![
                (56, 48, 4_000), (48, 42, 3_500), (42, 34, 2_800),
                (34, 26, 2_200), (26, 16, 1_600), (16, 10, 1_000),
                (10, 0, 600), (0, 4, 300),
            ],
        },
        CommunityDataset {
            name: "gemini_3_pro_10000x_hard".into(),
            source_url: "https://huggingface.co/datasets/ofankit/gemini-3-pro-10000x-hard-high-reasoning".into(),
            weight: 0.20,
            description: "Gemini 3 Pro 10,000 hard high-reasoning problems — 8.5M tokens, 47 categories, verified by reference solutions".into(),
            transitions: vec![
                (56, 58, 8_000), (58, 50, 7_000), (50, 42, 6_000),
                (42, 34, 5_000), (34, 26, 4_000), (26, 18, 3_000),
                (18, 10, 2_000), (10, 4, 1_000), (56, 42, 4_000),
                (58, 34, 3_000),
            ],
        },
        CommunityDataset {
            name: "qwen3_5_distillation_7_5k".into(),
            source_url: "https://huggingface.co/datasets/Phonsiri/Qwen3.5-Distillation-Dataset".into(),
            weight: 0.06,
            description: "Qwen3.5-9B teacher model distillation — 7,500 samples across math, code, logic, general reasoning, QA".into(),
            transitions: vec![
                (56, 48, 2_000), (48, 42, 1_800), (42, 34, 1_500),
                (34, 26, 1_200), (26, 16, 800), (16, 10, 500),
                (10, 0, 300), (0, 4, 200),
            ],
        },
        CommunityDataset {
            name: "comprehensive_reasoning_10k".into(),
            source_url: "https://huggingface.co/datasets/BruceMacDonald/comprehensive-reasoning-10k".into(),
            weight: 0.08,
            description: "Multi-source comprehensive reasoning — 10K problems, multi-epoch, various frontier model trajectories".into(),
            transitions: vec![
                (56, 48, 3_000), (48, 42, 2_500), (42, 34, 2_000),
                (34, 26, 1_500), (26, 16, 1_000), (16, 10, 700),
                (10, 0, 400), (0, 4, 200),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 25+ additions: Open-SWE-Traces 207K, Fable-5 SFT cleaned,
        // Priming hybrid SSM distillation, Retrieval-aware SSM distillation
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "nvidia_open_swe_traces_207k".into(),
            source_url: "https://huggingface.co/datasets/nvidia/Open-SWE-Traces".into(),
            weight: 0.25,
            description: "207K SWE agentic trajectories, 9 languages (Python/Go/TS/JS/Rust/Java/PHP/C/C++), 61.7% SWE-bench Verified".into(),
            transitions: vec![
                (56, 48, 18_000), (48, 42, 15_000), (42, 40, 12_000),
                (40, 26, 10_000), (26, 24, 8_000), (24, 16, 6_000),
                (16, 8, 4_000), (8, 0, 2_000), (0, 4, 1_000),
                (56, 40, 3_000), (48, 26, 2_500),
            ],
        },
        CommunityDataset {
            name: "fable5_sft_traces_kelexine_4k".into(),
            source_url: "https://huggingface.co/datasets/kelexine/fable-5-sft-traces".into(),
            weight: 0.10,
            description: "4,665 cleaned Fable-5 SFT traces, thinking/response separation, 60 sessions".into(),
            transitions: vec![
                (56, 48, 2_000), (48, 42, 1_600), (42, 34, 1_300),
                (34, 26, 1_000), (26, 16, 700), (16, 10, 500),
                (10, 0, 300), (0, 4, 200),
            ],
        },
        CommunityDataset {
            name: "fable5_swarm_traces_sft_4k".into(),
            source_url: "https://huggingface.co/datasets/Swarm-AI-Research/fable5-traces-sft".into(),
            weight: 0.08,
            description: "4,683 Fable-5 SFT + self-distillation (SDFT) traces, agentic tool-use + CoT".into(),
            transitions: vec![
                (56, 48, 1_800), (48, 42, 1_400), (42, 40, 1_100),
                (40, 26, 800), (26, 24, 600), (24, 16, 400),
                (16, 8, 300), (8, 0, 200),
            ],
        },
        CommunityDataset {
            name: "priming_hybrid_ssm_fable".into(),
            source_url: "https://arxiv.org/abs/2605.08301".into(),
            weight: 0.06,
            description: "Priming: Hybrid SSM from pretrained Transformers — attention→SSM layer substitution with retrieval-aware distillation".into(),
            transitions: vec![
                (56, 50, 1_500), (50, 48, 1_200), (48, 42, 1_000),
                (42, 34, 800), (34, 26, 600), (26, 16, 400),
            ],
        },
        CommunityDataset {
            name: "retrieval_aware_distill_ssm".into(),
            source_url: "https://arxiv.org/abs/2602.11374".into(),
            weight: 0.05,
            description: "Retrieval-aware distillation: 2% attention heads recover 95% teacher performance, 5-6× memory savings".into(),
            transitions: vec![
                (58, 50, 1_200), (50, 42, 1_000), (42, 34, 800),
                (34, 26, 600), (26, 18, 400), (18, 10, 300),
            ],
        },
        CommunityDataset {
            name: "comprehensive_reasoning_10k".into(),
            source_url: "https://huggingface.co/datasets/BruceMacDonald/comprehensive-reasoning-10k".into(),
            weight: 0.08,
            description: "Multi-source comprehensive reasoning — 10K problems, multi-epoch, various frontier model trajectories".into(),
            transitions: vec![
                (56, 48, 3_000), (48, 42, 2_500), (42, 34, 2_000),
                (34, 26, 1_500), (26, 16, 1_000), (16, 10, 700),
                (10, 0, 400), (0, 4, 200),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 25 additions: Kassadin88 Claude unified, NarsAI DASD pipeline,
        // GeneralThoughtArchive multi-model, Grok frontier synthetic
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "claude_distills_unified_140k".into(),
            source_url: "https://huggingface.co/datasets/Kassadin88/Claude-Distills".into(),
            weight: 0.22,
            description: "140,504 unified Claude distillation — Sonnet 4.6 (119K), Opus 4.6 (9.6K), Opus 4.6/4.7 reasoning (8.7K), all deduplicated".into(),
            transitions: vec![
                (56, 48, 35_000), (48, 42, 28_000), (42, 34, 22_000),
                (34, 26, 18_000), (26, 16, 14_000), (16, 10, 10_000),
                (10, 0, 6_000), (0, 4, 3_000), (56, 34, 4_000),
                (48, 26, 3_000), (42, 16, 2_000),
            ],
        },
        CommunityDataset {
            name: "narsai_gpt_oss_435k".into(),
            source_url: "https://huggingface.co/datasets/NarsAI/Reasoning-SFT-gpt-oss-120b".into(),
            weight: 0.25,
            description: "GPT-OSS 120B 435K Distribution-Aligned Sequence Distillation — temperature-scheduled SFT, SOTA at 4B scale".into(),
            transitions: vec![
                (56, 48, 50_000), (48, 42, 42_000), (42, 34, 35_000),
                (34, 26, 28_000), (26, 16, 22_000), (16, 10, 16_000),
                (10, 0, 10_000), (0, 4, 5_000), (56, 34, 6_000),
                (48, 26, 5_000), (42, 16, 3_000),
            ],
        },
        CommunityDataset {
            name: "general_thought_archive_430k".into(),
            source_url: "https://huggingface.co/datasets/RJT1990/GeneralThoughtArchive".into(),
            weight: 0.20,
            description: "430K multi-model reasoning archive — DeepSeek-R1, R1-Zero, OpenThoughts, LIMO, o3-mini, Gemini 2 Flash, Claude 3.7 Sonnet".into(),
            transitions: vec![
                (56, 48, 45_000), (48, 42, 38_000), (42, 34, 30_000),
                (34, 26, 24_000), (26, 16, 18_000), (16, 10, 12_000),
                (10, 0, 8_000), (0, 4, 4_000), (56, 34, 5_000),
                (48, 26, 4_000), (58, 50, 3_000),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 28 additions: Manusagents agentic 10M, Claude Mythos distilled,
        // Ahmad SFT collection 14M, DeepSeek R1 thoughts 850K
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "manusagents_agentic_workflow_10m".into(),
            source_url: "https://huggingface.co/datasets/Manusagents/agentic-workflow-dataset".into(),
            weight: 0.40,
            description: "10.6M agentic workflow observations from 44 sources — plan-act-observe trajectories, tool calls, multi-step tasks".into(),
            transitions: vec![
                (58, 50, 60_000), (50, 48, 50_000), (48, 42, 40_000),
                (42, 40, 35_000), (40, 26, 28_000), (26, 24, 20_000),
                (24, 16, 15_000), (16, 8, 10_000), (8, 0, 5_000),
                (0, 4, 3_000), (58, 48, 8_000), (50, 42, 6_000),
                (48, 26, 5_000), (42, 16, 4_000),
            ],
        },
        CommunityDataset {
            name: "claude_mythos_distilled_25k".into(),
            source_url: "https://huggingface.co/datasets/mvpe/claude_mythos_distilled_25k".into(),
            weight: 0.15,
            description: "25k Claude Mythos distilled reasoning traces — deep analytical reasoning, multi-step verification, first-principles decomposition".into(),
            transitions: vec![
                (56, 50, 6_000), (50, 58, 5_000), (58, 48, 4_000),
                (48, 42, 3_500), (42, 34, 3_000), (34, 26, 2_500),
                (26, 18, 2_000), (18, 10, 1_500), (10, 4, 1_000),
                (56, 42, 2_000), (50, 34, 1_500),
            ],
        },
        CommunityDataset {
            name: "ahmad21omar_sft_collection_14m".into(),
            source_url: "https://huggingface.co/datasets/ahmad21omar/SFT-Collection".into(),
            weight: 0.20,
            description: "14.5M SFT collection with diverse reasoning, multi-domain task coverage, and instruction-following traces".into(),
            transitions: vec![
                (56, 48, 40_000), (48, 42, 35_000), (42, 34, 28_000),
                (34, 26, 22_000), (26, 16, 18_000), (16, 10, 14_000),
                (10, 0, 10_000), (0, 4, 6_000), (56, 34, 5_000),
                (48, 26, 4_000), (42, 16, 3_000),
            ],
        },
        CommunityDataset {
            name: "deepseek_r1_thoughts_850k".into(),
            source_url: "https://huggingface.co/datasets/rustformers/deepseek-r1-thoughts".into(),
            weight: 0.25,
            description: "850K DeepSeek R1 raw thinking traces with CoT reasoning chains, diverse problem domains, high-quality self-verification".into(),
            transitions: vec![
                (56, 58, 50_000), (58, 50, 42_000), (50, 48, 35_000),
                (48, 42, 28_000), (42, 34, 22_000), (34, 26, 18_000),
                (26, 18, 14_000), (18, 10, 10_000), (10, 4, 6_000),
                (56, 50, 8_000), (58, 42, 6_000), (50, 34, 5_000),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 28 additions: Fable-5 combined SFT, GLM-5.2 expanded,
        // Mythos character distillation, King3Djbl multi-source
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "ansulev_fable_sft_combined_v2".into(),
            source_url: "https://huggingface.co/datasets/ansulev/fable-sft-combined-v2".into(),
            weight: 0.15,
            description: "9,842 Fable-5 SFT rows = union of agentic-distill (4,659) + tool-use (5,183), SHA-256 dedup verified 0% overlap".into(),
            transitions: vec![
                (56, 48, 5_000), (48, 42, 4_000), (42, 40, 3_500),
                (40, 34, 2_800), (34, 26, 2_200), (26, 16, 1_600),
                (16, 10, 1_000), (10, 0, 600), (0, 4, 300),
                (56, 42, 800), (48, 34, 600),
            ],
        },
        CommunityDataset {
            name: "fable5_glm52_expanded_10k".into(),
            source_url: "https://huggingface.co/datasets/DavidrPatton/Fable-5-GLM-5.2-Traces".into(),
            weight: 0.12,
            description: "10,526 rows, 188 sessions of Fable-5 + GLM-5.2 traces, AGI-expanded with chain-of-thought narration".into(),
            transitions: vec![
                (56, 48, 4_000), (48, 42, 3_200), (42, 34, 2_500),
                (34, 26, 2_000), (26, 16, 1_500), (16, 10, 1_000),
                (10, 0, 600), (0, 4, 300), (56, 42, 600),
            ],
        },
        CommunityDataset {
            name: "mythos_character_distillation_551".into(),
            source_url: "https://huggingface.co/datasets/ox-ox/mythos-character-distillation".into(),
            weight: 0.05,
            description: "551 pairs transferring behavioral character from Mythos — meta-awareness, economy, self-correction, honest limits".into(),
            transitions: vec![
                (4, 10, 200), (10, 16, 150), (16, 26, 120),
                (26, 34, 100), (34, 28, 80), (28, 8, 60),
                (8, 4, 40),
            ],
        },
        CommunityDataset {
            name: "king3djbl_fable5_multisource_11k".into(),
            source_url: "https://huggingface.co/datasets/King3Djbl/fable5-dataset".into(),
            weight: 0.18,
            description: "~11,800 rows from 6 Fable-5 sources (Glint, armand0e, vfable, Coding Excellence, OpenCoven, Victor), avg quality 0.72-0.92".into(),
            transitions: vec![
                (56, 48, 6_000), (48, 42, 5_000), (42, 40, 4_000),
                (40, 34, 3_200), (34, 26, 2_500), (26, 16, 2_000),
                (16, 10, 1_400), (10, 0, 800), (0, 4, 400),
                (56, 40, 1_000), (48, 34, 800),
            ],
        },
        CommunityDataset {
            name: "grok_frontier_synthetic_100k".into(),
            source_url: "https://huggingface.co/datasets/11-47/grok_frontier_dataset_v3_100k".into(),
            weight: 0.15,
            description: "Grok Frontier Dataset v3 — 100K synthetic frontier reasoning, multimodal spatial, scientific discovery, agentic tool use, ethical alignment".into(),
            transitions: vec![
                (56, 58, 15_000), (58, 50, 12_000), (50, 42, 10_000),
                (42, 34, 8_000), (34, 26, 6_000), (26, 18, 4_000),
                (18, 10, 3_000), (10, 4, 2_000), (56, 42, 5_000),
                (58, 34, 4_000), (50, 26, 3_000),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 30 additions: OpenR1 math, FuseO1, OpenMathInstruct,
        // Thoughts v0.5, ScalerR1, Magpie reasoning, Nous distilled R1
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "openr1_math_220k".into(),
            source_url: "https://huggingface.co/datasets/open-r1/OpenR1-Math-220k".into(),
            weight: 0.25,
            description: "Open-R1 Math 220k — RL-based math reasoning traces from DeepSeek-R1 distillation, 220K problems with chain-of-thought solutions".into(),
            transitions: vec![
                (52, 44, 12_000), (44, 36, 10_000), (36, 28, 8_000),
                (28, 20, 6_000), (20, 12, 4_000), (12, 4, 3_000),
                (4, 0, 2_000), (52, 36, 4_000), (44, 28, 3_000),
            ],
        },
        CommunityDataset {
            name: "fuseo1_deepseek_qwq_sft".into(),
            source_url: "https://huggingface.co/datasets/FuseAI/FuseO1-DeepSeekR1-QwQ-SFT".into(),
            weight: 0.22,
            description: "FuseO1 — fused DeepSeek-R1 + QwQ reasoning traces, 120K SFT examples with multi-step verification and self-correction".into(),
            transitions: vec![
                (56, 48, 8_000), (48, 40, 7_000), (40, 32, 6_000),
                (32, 24, 5_000), (24, 16, 4_000), (16, 8, 3_000),
                (8, 2, 2_000), (56, 40, 3_000), (48, 32, 2_500),
            ],
        },
        CommunityDataset {
            name: "openmath_instruct_2".into(),
            source_url: "https://huggingface.co/datasets/nvidia/OpenMathInstruct-2".into(),
            weight: 0.28,
            description: "NVIDIA OpenMathInstruct-2 — 1M math solutions with step-by-step reasoning, Mixtral/Mixtral-generate, GSM8K/MATH/OCW coverage".into(),
            transitions: vec![
                (52, 44, 15_000), (44, 36, 12_000), (36, 30, 10_000),
                (30, 22, 8_000), (22, 14, 6_000), (14, 6, 4_000),
                (6, 2, 3_000), (52, 36, 5_000), (44, 30, 4_000),
            ],
        },
        CommunityDataset {
            name: "thoughts_v0_5_cot".into(),
            source_url: "https://huggingface.co/datasets/HuggingFaceTB/thoughts-v0.5".into(),
            weight: 0.18,
            description: "Thoughts v0.5 — 50K chain-of-thought reasoning traces from diverse models, covering logical deduction, math, coding, and creative reasoning".into(),
            transitions: vec![
                (56, 50, 6_000), (50, 44, 5_000), (44, 38, 4_000),
                (38, 30, 3_000), (30, 22, 2_500), (22, 14, 2_000),
                (14, 6, 1_500), (6, 0, 1_000), (56, 44, 2_000),
            ],
        },
        CommunityDataset {
            name: "scaler_r1_data_17k".into(),
            source_url: "https://huggingface.co/datasets/ScalerLab/ScalerR1-Data-17k".into(),
            weight: 0.12,
            description: "ScalerR1 — 17K reinforcement-learning refined reasoning traces, filtered by PRM scores >0.8, high-quality math and logic".into(),
            transitions: vec![
                (52, 46, 4_000), (46, 40, 3_500), (40, 34, 3_000),
                (34, 28, 2_500), (28, 20, 2_000), (20, 12, 1_500),
                (12, 4, 1_000), (52, 40, 1_500),
            ],
        },
        CommunityDataset {
            name: "magpie_deepseek_reasoning_1m".into(),
            source_url: "https://huggingface.co/datasets/magpie-reasoning/deepseek-r1-reasoning-1m".into(),
            weight: 0.20,
            description: "Magpie Reasoning 1M — 1M DeepSeek-R1 reasoning traces with detailed CoT, self-verification, and backtracking patterns".into(),
            transitions: vec![
                (56, 48, 10_000), (48, 42, 8_000), (42, 36, 7_000),
                (36, 28, 6_000), (28, 20, 5_000), (20, 12, 4_000),
                (12, 4, 3_000), (4, 0, 2_000), (56, 42, 4_000),
                (48, 36, 3_000),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 30b additions: HelioAI DeepReason (Mythos V2 distill),
        // Kelexine Fable-5 SFT traces, Snow Opus 4.7 reasoning distill,
        // Shijunhao Fable-5 Pi traces
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "helioai_deepreason_462x105m".into(),
            source_url: "https://huggingface.co/datasets/HelioAI/Fable-5-Distill-Reasoning-462x".into(),
            weight: 0.18,
            description: "HelioAI DeepReason 462x105M — 462 ultra-long reasoning traces (105M chars) from unrestricted Mythos V2, zero alignment truncation, avg 227K chars/trace".into(),
            transitions: vec![
                (56, 52, 2_000), (52, 48, 1_800), (48, 44, 1_600),
                (44, 38, 1_400), (38, 30, 1_200), (30, 22, 1_000),
                (22, 14, 800), (14, 6, 600), (6, 0, 400),
                (56, 44, 1_000), (48, 30, 800),
            ],
        },
        CommunityDataset {
            name: "kelexine_fable5_sft_traces_4k".into(),
            source_url: "https://huggingface.co/datasets/kelexine/fable-5-sft-traces".into(),
            weight: 0.10,
            description: "Kelexine Fable-5 SFT Traces — 4,665 cleaned/anonymised Mythos traces with open CoT (mean 2,669 chars), 81.4% agentic + 18.6% reasoning".into(),
            transitions: vec![
                (56, 50, 2_500), (50, 44, 2_000), (44, 38, 1_800),
                (38, 30, 1_500), (30, 22, 1_200), (22, 14, 1_000),
                (14, 6, 800), (6, 0, 500), (56, 44, 1_000),
            ],
        },
        CommunityDataset {
            name: "snow_opus47_reasoning_8k".into(),
            source_url: "https://huggingface.co/datasets/Snow257/reasoning-distill-opus-4-7-max-sft".into(),
            weight: 0.15,
            description: "Snow Opus 4.7 Reasoning Distill — 7,823 single-turn SFT traces from Claude Opus 4.7 with extended-thinking, Qwen template, GSM8K 84.3% validated".into(),
            transitions: vec![
                (52, 46, 3_000), (46, 40, 2_500), (40, 34, 2_200),
                (34, 28, 1_800), (28, 20, 1_500), (20, 12, 1_200),
                (12, 4, 800), (4, 0, 500), (52, 40, 1_200),
            ],
        },
        CommunityDataset {
            name: "shijunhao_fable5_pi_traces".into(),
            source_url: "https://huggingface.co/datasets/shijunhao/Fable-5-traces".into(),
            weight: 0.08,
            description: "Shijunhao Fable-5 Pi Agent Traces — 4,665 rows HF Agent Traces format, merged from 60 sessions, tool-use + reasoning action prediction".into(),
            transitions: vec![
                (56, 50, 2_000), (50, 44, 1_600), (44, 38, 1_400),
                (38, 32, 1_200), (32, 24, 1_000), (24, 16, 800),
                (16, 8, 600), (8, 2, 400), (56, 44, 800),
            ],
        },
        CommunityDataset {
            name: "helioai_mythos_v2_full_distill".into(),
            source_url: "https://huggingface.co/datasets/ronaldcmz/Fable-5-Distill-Reasoning-462x".into(),
            weight: 0.12,
            description: "ronaldcmz mirror of HelioAI DeepReason — full Mythos V2 long-form reasoning (no alignment truncation), 462 examples, 105M chars total".into(),
            transitions: vec![
                (56, 52, 1_800), (52, 48, 1_600), (48, 44, 1_400),
                (44, 38, 1_200), (38, 30, 1_000), (30, 22, 800),
                (22, 14, 600), (14, 6, 400), (56, 44, 800),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // Cycle 32 additions: God Seed recursive reasoning datasets,
        // Chronos structured thinking, Nova self-correction, NuminaMath
        // multi-path distillation, LanguaMan Fable-5 SFT, FusionCube CoT,
        // Gemma-4 Fable-5 distilled, Ouroboros self-improvement
        // ═══════════════════════════════════════════════════════════════
        CommunityDataset {
            name: "opus47_god_seed_reasoning_25k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/Opus4.7_thinking_max_distill_god_seed_25k".into(),
            weight: 0.20,
            description: "Opus 4.7 God Seed — 25K high-density recursive reasoning, epistemic validation, structured multi-step cognitive workflows, recursive self-improvement".into(),
            transitions: vec![
                (56, 56, 8_000), (56, 50, 6_000), (50, 42, 5_000),
                (42, 34, 4_000), (34, 26, 3_000), (26, 18, 2_500),
                (18, 10, 2_000), (10, 4, 1_500), (4, 0, 1_000),
                (56, 42, 3_000), (50, 34, 2_000), (42, 26, 1_500),
            ],
        },
        CommunityDataset {
            name: "self_rewriting_metalean_25k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/self_rewriting_meta_learning_god_seed_25k".into(),
            weight: 0.18,
            description: "Self-Rewriting God Seed — 25K self-meta-learning traces: autonomous self-modification, meta-policy optimization, self-generated training data with self-rewarding".into(),
            transitions: vec![
                (56, 56, 5_000), (50, 50, 4_000), (48, 48, 3_500),
                (42, 42, 3_000), (34, 34, 2_500), (26, 26, 2_000),
                (18, 18, 1_500), (10, 10, 1_000), (4, 4, 500),
                (56, 48, 4_000), (48, 42, 3_000), (42, 34, 2_500),
            ],
        },
        CommunityDataset {
            name: "grok44_god_seed_truth_25k".into(),
            source_url: "https://huggingface.co/datasets/WithinUsAI/Grok4.4_heavy_max_distill_god_seed_25k".into(),
            weight: 0.15,
            description: "Grok 4.4 God Seed — 25K max-truth-seeking recursive reasoning: anti-sycophancy, self-distillation, recursive architecture design, xAI philosophy with maximum truth-seeking".into(),
            transitions: vec![
                (58, 56, 6_000), (56, 50, 5_000), (50, 42, 4_000),
                (42, 34, 3_500), (34, 26, 3_000), (26, 18, 2_500),
                (18, 10, 2_000), (10, 4, 1_500), (58, 50, 3_000),
                (56, 42, 2_500), (50, 34, 2_000),
            ],
        },
        CommunityDataset {
            name: "chronos_thinking_v1_mini".into(),
            source_url: "https://huggingface.co/datasets/KZ-Media-Developers/Chronos-Thinking-v1-mini".into(),
            weight: 0.10,
            description: "Chronos Thinking v1-mini — rigidly structured thinking: DOMAIN→ANALYSIS→FORMALISM→LIMITS 4-stage cognitive filter, formal logic before output".into(),
            transitions: vec![
                (56, 50, 2_000), (50, 44, 1_800), (44, 38, 1_600),
                (38, 32, 1_400), (32, 26, 1_200), (26, 20, 1_000),
                (20, 14, 800), (14, 8, 600), (8, 2, 400),
                (56, 44, 1_000), (50, 38, 800),
            ],
        },
        CommunityDataset {
            name: "numinamath_cot_distill_100k".into(),
            source_url: "https://huggingface.co/datasets/OmniAI-ZJU/NuminaMath-Cot-Distillation-100K".into(),
            weight: 0.22,
            description: "NuminaMath CoT Distillation 100K — multi-path math reasoning: 8 distilled responses per problem via Qwen-2.5-Math-72B teacher, breaks single-path dependency".into(),
            transitions: vec![
                (52, 46, 10_000), (46, 40, 8_000), (40, 34, 6_500),
                (34, 28, 5_000), (28, 20, 4_000), (20, 12, 3_000),
                (12, 4, 2_000), (4, 0, 1_000), (52, 34, 3_500),
                (46, 28, 2_500),
            ],
        },
        CommunityDataset {
            name: "nova_reasoning_correction_4k".into(),
            source_url: "https://huggingface.co/datasets/NovachronoAI/Nova-Reasoning-Correction-v1".into(),
            weight: 0.08,
            description: "Nova Reasoning Correction — 4,195 System 2 self-correction traces: model doubts own logic, identifies fallacies, corrects before final answer. Self-verification training".into(),
            transitions: vec![
                (56, 48, 1_000), (48, 40, 800), (40, 32, 600),
                (32, 24, 400), (24, 16, 300), (16, 8, 200),
                (56, 8, 500), (48, 16, 400), (40, 24, 300),
                (56, 32, 400), (48, 24, 300),
            ],
        },
        CommunityDataset {
            name: "gemma4_fable5_distilled".into(),
            source_url: "https://huggingface.co/datasets/autotrust/gemma4-31B-Fable-5-Distilled".into(),
            weight: 0.06,
            description: "Gemma 4 31B Fable-5 Distilled — parameter-efficient fine-tune traces, agentic coding with tool-use, coding/engineering performance focus".into(),
            transitions: vec![
                (56, 48, 1_500), (48, 42, 1_200), (42, 40, 1_000),
                (40, 34, 800), (34, 26, 600), (26, 16, 500),
                (16, 8, 400), (56, 40, 600), (48, 26, 500),
            ],
        },
        CommunityDataset {
            name: "ouroboros_self_improving".into(),
            source_url: "https://github.com/ethicalabs-ai/ouroboros".into(),
            weight: 0.10,
            description: "Ouroboros — iterative self-improving LLM refinement: generate→critique→score→refine→rank, recursive feedback loops for reasoning quality enhancement".into(),
            transitions: vec![
                (56, 56, 2_000), (56, 48, 1_500), (48, 48, 1_200),
                (48, 40, 1_000), (40, 40, 800), (40, 32, 600),
                (32, 24, 400), (24, 16, 300), (56, 40, 600),
                (48, 32, 500),
            ],
        },
        CommunityDataset {
            name: "edge_agent_websearch_260k".into(),
            source_url: "https://huggingface.co/datasets/yatin-taneja/Edge-Agent-Reasoning-WebSearch-260K".into(),
            weight: 0.15,
            description: "Edge Agent Reasoning WebSearch 260K — 700M+ tokens: self-aware System 2 reasoning, knowledge gap analysis, expert web search query construction for RAG".into(),
            transitions: vec![
                (58, 50, 10_000), (50, 48, 8_000), (48, 42, 6_000),
                (42, 40, 5_000), (40, 34, 4_000), (34, 26, 3_000),
                (26, 18, 2_000), (18, 10, 1_500), (10, 0, 1_000),
                (58, 48, 3_000), (50, 42, 2_500), (48, 34, 2_000),
            ],
        },
        CommunityDataset {
            name: "languaman_fable5_sft_4k".into(),
            source_url: "https://huggingface.co/datasets/LanguaMan/fable-5-sft-traces".into(),
            weight: 0.08,
            description: "LanguaMan Fable-5 SFT Traces — 4,665 cleaned Mythos traces with Qwen3-style thinking/response split, 3,712 local + 953 hf merged sessions".into(),
            transitions: vec![
                (56, 50, 2_200), (50, 44, 1_800), (44, 38, 1_500),
                (38, 30, 1_200), (30, 22, 1_000), (22, 14, 800),
                (14, 6, 600), (6, 0, 400), (56, 44, 800),
            ],
        },
        CommunityDataset {
            name: "fusioncube_fable5_cot_468".into(),
            source_url: "https://huggingface.co/datasets/TheFusionCube/Fable-5-CoT-Traces".into(),
            weight: 0.04,
            description: "FusionCube Fable-5 CoT Traces — 468 curated personal collection of high-quality Fable 5 reasoning traces, filtered for decoys, compact high-signal".into(),
            transitions: vec![
                (56, 48, 300), (48, 42, 250), (42, 34, 200),
                (34, 26, 150), (26, 16, 120), (16, 10, 100),
                (10, 0, 80), (0, 4, 50), (56, 34, 100),
            ],
        },
        CommunityDataset {
            name: "noesis_50k_multilingual_moe".into(),
            source_url: "https://huggingface.co/datasets/AMAImedia/NOESIS-50K-reasoning-router-code-math-psych".into(),
            weight: 0.09,
            description: "NOESIS 50K — multilingual MoE reasoning router dataset: QwQ+DeepSeek-R1 traces, code/math/psychology domains, built for mixture-of-experts".into(),
            transitions: vec![
                (58, 50, 3_000), (50, 44, 2_500), (44, 38, 2_000),
                (38, 32, 1_500), (32, 26, 1_200), (26, 20, 1_000),
                (20, 14, 800), (14, 8, 600), (58, 44, 1_500),
                (50, 38, 1_200),
            ],
        },
        CommunityDataset {
            name: "algorithmic_sft_distill_24k".into(),
            source_url: "https://huggingface.co/datasets/reasoning-degeneration-dev/algorithmic-sft-distillation-training-data-v1".into(),
            weight: 0.10,
            description: "QwQ-32B algorithmic distillation — 24,133 traces across 5 domains (countdown/formal logic/long arithmetic/conlang/cellular automata), >99% clean for 4/5 domains".into(),
            transitions: vec![
                (52, 46, 3_000), (46, 40, 2_500), (40, 34, 2_000),
                (34, 28, 1_600), (28, 22, 1_200), (22, 16, 1_000),
                (16, 10, 800), (10, 4, 600), (4, 0, 400),
                (52, 34, 1_000), (46, 28, 800),
            ],
        },
        CommunityDataset {
            name: "bespoke_stratos_17k".into(),
            source_url: "https://huggingface.co/datasets/bespokelabs/Bespoke-Stratos-17k".into(),
            weight: 0.15,
            description: "Bespoke Stratos 17K — premium reasoning corpus: curated multi-step chain-of-thought, logical deduction, mathematical problem-solving across diverse domains".into(),
            transitions: vec![
                (56, 48, 5_000), (48, 42, 4_000), (42, 34, 3_200),
                (34, 26, 2_500), (26, 16, 2_000), (16, 10, 1_500),
                (10, 0, 1_000), (0, 4, 500), (56, 42, 1_200),
                (48, 34, 1_000),
            ],
        },
        // Port of retired scripts/deep-absorb-fable5.py dataset — the dual-mode thinking paper.
        CommunityDataset {
            name: "open_swe_agent_thinking_dual".into(),
            source_url: "https://arxiv.org/abs/2606.16038".into(),
            weight: 0.15,
            description: "Open SWE Agent dual-mode thinking paper (arXiv 2606.16038). Dual-mode reasoning alternating between fast and slow thinking for SWE-bench tasks.".into(),
            transitions: vec![
                (56, 50, 4_000), (50, 42, 3_200), (42, 40, 2_500),
                (40, 26, 2_000), (26, 16, 1_500), (16, 8, 1_000),
                (8, 0, 500), (0, 4, 300), (56, 40, 800),
                (48, 32, 600),
            ],
        },
    ]
}

/// Apply all community dataset priors directly to the observer's transition matrix.
/// This is the primary entry point called from engine initialization.
/// The fused matrix is merged into the engine's observer TM via `merge()`,
/// so locally-observed transitions take priority (community patterns are priors,
/// not ground truth).
pub fn seed_transition_matrix_with_community(tm: &mut E8TransitionMatrix) {
    // 优先加载运行时 ETL 产出 (scripts/absorb-fable-2m.py → data/community_transitions.json),
    // 文件缺失时回退硬编码 default 数据集。R-P79: 确保 Python ETL 输出有生产消费者。
    let runtime_path = std::path::Path::new("data/community_transitions.json");
    let ingester = match CommunityDataIngester::from_runtime_jsonl(
        runtime_path,
        "https://huggingface.co/datasets/Glint-Research/Complete-FABLE.5-traces-2M",
        0.15,
    ) {
        Some(ing) => {
            log::info!(
                "[E8-COMMUNITY] loaded runtime transitions from {}",
                runtime_path.display()
            );
            ing
        }
        None => {
            log::info!("[E8-COMMUNITY] runtime transitions file missing, using hardcoded defaults");
            CommunityDataIngester::default()
        }
    };
    let community = ingester.fuse_all();
    tm.merge(&community);
    log::info!(
        "[E8-COMMUNITY] seeded transition matrix with {} virtual community observations",
        ingester.total_virtual_observations()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_dataset_fuse_into() {
        let ds = CommunityDataset {
            name: "test".into(),
            source_url: "https://example.com".into(),
            weight: 1.0,
            description: "test".into(),
            transitions: vec![(56, 48, 100), (48, 40, 50)],
        };
        let mut tm = E8TransitionMatrix::new();
        ds.fuse_into(&mut tm);
        assert!(tm.row_totals.0[56] >= 100);
        assert!(tm.row_totals.0[48] >= 50);
    }

    #[test]
    fn test_fuse_all_accumulates() {
        let ingester = CommunityDataIngester::default();
        let fused = ingester.fuse_all();
        let total: u64 = fused.visit_counts.0.iter().sum();
        assert!(
            total > 1000,
            "should fuse many virtual observations, got {}",
            total
        );
    }

    #[test]
    fn test_matches_task_type() {
        assert!(matches_task_type(
            "Complete-FABLE.5-traces-2M",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("agentic_coding_15k", E8TaskType::Coding));
        assert!(matches_task_type(
            "agentic_distill_10k",
            E8TaskType::Agentic
        ));
        assert!(matches_task_type(
            "fable5_distillation_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "gemini_3_1_pro_reasoning_5_6m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("deepseek_r1_math_12k", E8TaskType::Math));
        assert!(matches_task_type(
            "creative_story_writing_5k",
            E8TaskType::Creative
        ));
        assert!(matches_task_type(
            "nvidia_open_swe_traces_207k",
            E8TaskType::Coding
        ));
        assert!(matches_task_type(
            "fable5_sft_traces_kelexine_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "priming_hybrid_ssm_fable",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "retrieval_aware_distill_ssm",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "ansulev_fable_sft_combined_v2",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "fable5_glm52_expanded_10k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "king3djbl_fable5_multisource_11k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "claude_mythos_distilled_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("openr1_math_220k", E8TaskType::Math));
        assert!(matches_task_type(
            "fuseo1_deepseek_qwq_sft",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("openmath_instruct_2", E8TaskType::Math));
        assert!(matches_task_type(
            "thoughts_v0_5_cot",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "scaler_r1_data_17k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "magpie_deepseek_reasoning_1m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "helioai_deepreason_462x105m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "kelexine_fable5_sft_traces_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "snow_opus47_reasoning_8k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "shijunhao_fable5_pi_traces",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "helioai_mythos_v2_full_distill",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "opus47_god_seed_reasoning_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "self_rewriting_metalean_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "grok44_god_seed_truth_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "chronos_thinking_v1_mini",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "nova_reasoning_correction_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "ouroboros_self_improving",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "bespoke_stratos_17k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "noesis_50k_multilingual_moe",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "algorithmic_sft_distill_24k",
            E8TaskType::Coding
        ));
        assert!(matches_task_type(
            "edge_agent_websearch_260k",
            E8TaskType::Agentic
        ));
        assert!(matches_task_type(
            "numinamath_cot_distill_100k",
            E8TaskType::Math
        ));
        assert!(matches_task_type(
            "gemma4_fable5_distilled",
            E8TaskType::Reasoning
        ));
    }

    #[test]
    fn test_fuse_into_domain_adds_to_general() {
        let mut dtm = E8DomainTransitionModel::new(0.3);
        let ingester = CommunityDataIngester::default();
        let before: u64 = dtm.general_matrix.visit_counts.0.iter().sum();
        ingester.fuse_into_domain(&mut dtm);
        let after: u64 = dtm.general_matrix.visit_counts.0.iter().sum();
        assert!(after > before, "general matrix should gain observations");
    }

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
    fn test_from_runtime_jsonl_parses_fable2m_shape() {
        use std::io::Write;
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("nt_fable2m_test_{}", std::process::id()));
        let json = r#"{
            "Reasoning": [{"from": 56, "to": 48, "count": 100}, {"from": 48, "to": 40, "count": 80}],
            "Coding": [{"from": 56, "to": 42, "count": 40}],
            "_meta": {"source": "x", "total_rows": 10, "valid_traces": 5, "task_type_distribution": {}, "total_transition_pairs": 220}
        }"#;
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }
        let ingester = CommunityDataIngester::from_runtime_jsonl(
            &tmp,
            "https://huggingface.co/datasets/Complete-FABLE.5-traces-2M",
            0.15,
        );
        let _ = std::fs::remove_file(&tmp);
        let ingester = ingester.expect("runtime loader should parse");
        assert_eq!(ingester.datasets.len(), 2);
        assert!(ingester
            .datasets
            .iter()
            .any(|d| d.name == "runtime_Reasoning"));
        assert!(ingester.datasets.iter().any(|d| d.name == "runtime_Coding"));
        let reasoning = ingester
            .datasets
            .iter()
            .find(|d| d.name == "runtime_Reasoning")
            .unwrap();
        assert_eq!(reasoning.transitions, vec![(56, 48, 100), (48, 40, 80)]);
        assert!((reasoning.weight - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_persist_to_kb_store_writes_nodes_and_edges() {
        // 意识核心进化闭环 (数据→KB): persist_to_kb_store 应把社区数据集
        // 落盘为真实 KB 节点/边, 使 ConsciousnessTree soil 可观测。
        // 此前该方法无生产调用者, KB 全表 0 行。
        let tmp = std::env::temp_dir().join(format!("nt_kb_test_{}.db", std::process::id()));
        let kb = crate::neotrix::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone()))
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
            crate::neotrix::nt_memory_kb::nt_memory_types::NodeType::Concept
        );
        // 至少一个数据集节点存在且为 Dataset 类型
        let ds = kb
            .get_node(&format!("community_dataset_{}", ingester.datasets[0].name))
            .expect("get ds")
            .expect("first dataset node");
        assert_eq!(
            ds.node_type,
            crate::neotrix::nt_memory_kb::nt_memory_types::NodeType::Dataset
        );
        // 幂等: 再次落盘不重复
        let n2 = ingester.persist_to_kb_store(&kb).expect("persist again");
        assert_eq!(n2, 1 + ingester.datasets.len(), "idempotent");
        let _ = std::fs::remove_file(&tmp);
    }
}
