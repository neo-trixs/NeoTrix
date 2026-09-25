//! nt_resource_ingester — SQLite ingest + relate + 会话编排与批量吸收，行为零变更纯搬移。

use log::{info, warn};
use rusqlite::Connection;
use serde_json;
use uuid::Uuid;

use super::super::nt_memory_store::*;
use super::super::nt_memory_types::*;
use super::super::shared_utils::now;
use super::nt_resource_cortex::{CORTEX_ROOT, register_cortex_brain};
use super::nt_resource_types::{ResourceDescriptor, ResourceIngestResult, ResourceSource};

pub struct ResourceIngester<'a> {
    conn: &'a Connection,
    episode_id: String,
    ingest_log: Vec<IngestLogEntry>,
}

struct IngestLogEntry {
    title: String,
    node_type: NodeType,
    node_id: String,
    status: String,
    error: Option<String>,
}

impl<'a> ResourceIngester<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let episode_id = Uuid::new_v4().to_string();
        Self {
            conn,
            episode_id,
            ingest_log: Vec::new(),
        }
    }

    pub fn ingest(&mut self, desc: &ResourceDescriptor) -> Result<ResourceIngestResult, String> {
        let url = desc.source.url();
        let domain = desc.source.domain();

        let metadata = serde_json::json!({
            "source": match &desc.source {
                ResourceSource::GitHub { owner, repo } =>
                    serde_json::json!({"type": "github", "owner": owner, "repo": repo}),
                ResourceSource::ArXiv { id } =>
                    serde_json::json!({"type": "arxiv", "id": id}),
                ResourceSource::Web { url } =>
                    serde_json::json!({"type": "web", "url": url}),
                ResourceSource::Direct =>
                    serde_json::json!({"type": "direct"}),
            },
            "tags": desc.tags,
            "episode_id": self.episode_id,
            "insight_count": desc.key_insights.len(),
        });

        let content = desc.content.clone().or_else(|| {
            if desc.key_insights.is_empty() {
                None
            } else {
                Some(desc.key_insights.join("\n- "))
            }
        });

        let ts = now();
        let node = KnowledgeNode {
            id: Uuid::new_v4().to_string(),
            node_type: desc.category.clone(),
            title: desc.title.clone(),
            summary: Some(desc.summary.clone()),
            content,
            url,
            domain,
            language: "en".to_string(),
            recall_weight: 1.0,
            confidence: desc.confidence,
            importance: desc.importance,
            created_at: ts,
            updated_at: ts,
            access_count: 0,
            metadata: Some(metadata),
            temporal: None,
            supersedes: None,
            source_episode: Some(self.episode_id.clone()),
            parent_id: None,
            depth: 0,
            cluster_id: None,
        };

        insert_node(self.conn, &node).map_err(|e| format!("insert_node failed: {}", e))?;
        let node_id = node.id.clone();

        let mut insight_ids = Vec::new();

        for insight_text in &desc.key_insights {
            let insight_title = if insight_text.len() > 80 {
                let cut = (0..=77)
                    .rev()
                    .find(|&i| insight_text.is_char_boundary(i))
                    .unwrap_or(77);
                format!("{}...", &insight_text[..cut])
            } else {
                insight_text.clone()
            };

            let insight_node = KnowledgeNode {
                id: Uuid::new_v4().to_string(),
                node_type: NodeType::Insight,
                title: insight_title,
                summary: Some(insight_text.clone()),
                content: None,
                url: None,
                domain: None,
                language: "en".into(),
                confidence: desc.confidence * 0.8,
                recall_weight: 1.0,
                importance: desc.importance * 0.7,
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata: Some(serde_json::json!({
                    "parent_resource_id": node_id,
                    "episode_id": self.episode_id.clone(),
                })),
                temporal: None,
                supersedes: None,
                source_episode: Some(self.episode_id.clone()),
                parent_id: None,
                depth: 0,
                cluster_id: None,
            };

            let iid = insight_node.id.clone();
            insert_node(self.conn, &insight_node)
                .map_err(|e| format!("insert insight node failed: {}", e))?;
            upsert_edge(
                self.conn,
                &node_id,
                &iid,
                RelationType::Supports,
                0.7,
                Some("Key insight derived from resource"),
            )
            .map_err(|e| format!("upsert insight edge failed: {}", e))?;
            insight_ids.push(iid);
        }

        self.ingest_log.push(IngestLogEntry {
            title: desc.title.clone(),
            node_type: desc.category.clone(),
            node_id: node_id.clone(),
            status: "success".into(),
            error: None,
        });

        Ok(ResourceIngestResult {
            node_id,
            insight_ids,
        })
    }

    pub fn relate(
        &self,
        from_id: &str,
        to_id: &str,
        rel: RelationType,
        weight: f64,
        desc: Option<&str>,
    ) -> Result<(), String> {
        upsert_edge(self.conn, from_id, to_id, rel, weight, desc)
            .map_err(|e| format!("relate failed: {}", e))
    }

    pub fn relate_by_title(
        &self,
        from_title: &str,
        to_title: &str,
        rel: RelationType,
        weight: f64,
        desc: Option<&str>,
    ) -> Result<(), String> {
        let from = find_node_by_title(self.conn, from_title)
            .map_err(|e| format!("find from '{}' failed: {}", from_title, e))?
            .ok_or_else(|| format!("node not found by title: {}", from_title))?;
        let to = find_node_by_title(self.conn, to_title)
            .map_err(|e| format!("find to '{}' failed: {}", to_title, e))?
            .ok_or_else(|| format!("node not found by title: {}", to_title))?;
        upsert_edge(self.conn, &from.id, &to.id, rel, weight, desc)
            .map_err(|e| format!("relate_by_title failed: {}", e))
    }

    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== Resource Ingestion Report ===".to_string());
        lines.push(format!("Episode ID: {}", self.episode_id));
        lines.push(format!(
            "Total resources ingested: {}",
            self.ingest_log.len()
        ));
        for entry in &self.ingest_log {
            let status = if entry.status == "success" {
                "✅"
            } else {
                "❌"
            };
            lines.push(format!(
                "  {} {} ({:?}) — {}",
                status, entry.title, entry.node_type, entry.node_id
            ));
            if let Some(ref err) = entry.error {
                lines.push(format!("    Error: {}", err));
            }
        }
        lines.join("\n")
    }

    pub fn episode_id(&self) -> &str {
        &self.episode_id
    }
}

pub(crate) fn find_node_by_title(conn: &Connection, title: &str) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata
         FROM nodes WHERE title=?1 LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![title])?;
    match rows.next()? {
        Some(row) => Ok(Some(KnowledgeNode {
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
            metadata: row
                .get::<_, Option<String>>(13)?
                .and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
        })),
        None => Ok(None),
    }
}

pub fn ingest_session_resources(conn: &Connection) -> Result<String, String> {
    let mut ingester = ResourceIngester::new(conn);

    ingest_github_resources(&mut ingester)?;
    ingest_paper_resources(&mut ingester)?;
    ingest_web_resources(&mut ingester)?;
    ingest_bug_fixes(&mut ingester)?;
    ingest_new_modules(&mut ingester)?;
    link_related_resources(&mut ingester)?;

    // ── Wave 2026-09-20: Crystal Core absorption ──
    // Absorb arxiv-complete dataset metadata (3.1M papers, sampled)
    ingest_arxiv_complete_metadata(&mut ingester)?;
    // Absorb key GitHub repos into crystal core
    ingest_crystal_core_repos(&mut ingester)?;
    // Absorb personal-ai ecosystem repos
    ingest_personal_ai_ecosystem(&mut ingester)?;

    // External "Cortex-Brain" volume: register as a discoverable KB catalog so the
    // 114 GB offline archive is connected (Dark Forest), not inert. No-op if unmounted.
    // This runs in production (ingest_session_resources is called at startup) → T3 wiring.
    if let Err(e) = register_cortex_brain(conn, std::path::Path::new(CORTEX_ROOT)) {
        warn!("[cortex] register skipped: {e}");
    }

    Ok(ingester.report())
}

fn ingest_github_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::github(
        "stablyai", "orca",
        "stablyai/orca — Orca: Dual-System NextState Prediction",
        "Orca implements a dual-system architecture with Unconscious (dense Markov) and Conscious (sparse event-conditioned) NextStatePredictor, plus a DecoderReadout that freezes the backbone and trains only lightweight readout layers."
    ).with_key_insights(vec![
        "Three prediction modes: Unconscious/Conscious/Hybrid controlled by α∈[0,1]",
        "DecoderReadout gradient descent on frozen backbone",
        "Dual-system architecture mirrors NeoTrix GWT conscious/unconscious processing",
    ]).with_tags(vec!["dual-system", "next-state-prediction", "decoder-readout", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "offchainthoughts", "Amber",
        "offchainthoughts/Amber — Self-Certifying Embedding Artifacts",
        "Amber defines a self-certifying embedding artifact format using SHA-256 Merkle commitment over quantized embedding chunks, providing probabilistic authenticity audit with 1−2^{−128} confidence."
    ).with_key_insights(vec![
        "Flattened embedding vector + SHA-256 Merkle tree binds source chunks to quantized embeddings",
        "Probabilistic authenticity audit achieves 1−2^{−128} confidence",
        "4-bit quantization reduces storage 8× vs f32",
    ]).with_tags(vec!["embedding", "merkle", "commitment", "audit", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "AgwaB", "pi-workflow",
        "AgwaB/pi-workflow — Workflow Orchestration Patterns",
        "Workflow orchestration patterns for multi-step agent pipelines with conditional branching, parallel execution, and error recovery."
    ).with_key_insights(vec![
        "Conditional branching based on intermediate results",
        "Parallel execution of independent workflow branches",
        "Error recovery with retry and fallback paths",
    ]).with_tags(vec!["workflow", "orchestration", "patterns", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "facebook", "astryx",
        "facebook/astryx — Graph Memory Architecture",
        "Graph-structured memory architecture for persistent agent state, providing queryable, inspectable long-term memory with graph traversal operations."
    ).with_key_insights(vec![
        "Graph structure as core architecture for agent memory",
        "Enables BFS-based relationship discovery across memory",
        "Supports long-term persistent agent state management",
    ]).with_tags(vec!["graph-memory", "agent-state", "persistence", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "msitarzewski", "agency-agents",
        "msitarzewski/agency-agents — Agent Pattern Composition",
        "Composable agent patterns for building complex multi-agent systems with specialized roles and inter-agent communication."
    ).with_key_insights(vec![
        "Agent composition patterns for multi-agent systems",
        "Specialized role assignment with inter-agent communication",
        "Scalable agent coordination architectures",
    ]).with_tags(vec!["agent-patterns", "multi-agent", "composition", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "shadcn-labs", "agentcn",
        "shadcn-labs/agentcn — Agent Coordination Patterns",
        "Coordination strategies for AI agent teams, including consensus mechanisms, delegation protocols, and conflict resolution."
    ).with_key_insights(vec![
        "Consensus mechanisms for multi-agent decision making",
        "Delegation protocols with capability matching",
        "Conflict resolution between competing agent proposals",
    ]).with_tags(vec!["coordination", "consensus", "delegation", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "crosstalk-solutions", "project-nomad",
        "crosstalk-solutions/project-nomad — Persistent Agent State",
        "Persistent agent state management system enabling long-running agents with checkpoint/restore, state migration, and session continuity."
    ).with_key_insights(vec![
        "Checkpoint/restore for long-running agent sessions",
        "State migration across different runtime environments",
        "Session continuity with durable state snapshots",
    ]).with_tags(vec!["agent-state", "persistence", "checkpoint", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::github(
        "google", "sec-gemini",
        "google/sec-gemini — Security Agents with Function-Calling",
        "Gemini-powered security agents with function-calling tools for rule search, coverage heatmaps, duplicate detection, and LLM-based rule review."
    ).with_key_insights(vec![
        "Function-calling tools for security rule search and coverage analysis",
        "Coverage heatmap generation for security rule gaps",
        "LLM-based rule review and duplicate detection",
    ]).with_tags(vec!["security", "gemini", "function-calling", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_paper_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::paper(
        "2605.06732",
        "Training in Imagination — Optimal Sample Allocation for Model-Based RL",
        "Nadav Timor et al. prove Theorem 1 (optimal dynamics vs reward sample ratio), Theorem 2 (noisy reward REINFORCE gradient), and Lemma 1 (Lipschitz error bound). Provides theoretical foundation for efficient model-based RL training."
    ).with_key_insights(vec![
        "Theorem 1: Ndyn/Nrew = α/β · γ·Lr·(1+Lπ) / (1−γ·Lf·(1+Lπ)) · crew/cdyn · εdyn/εrew",
        "Lemma 1: Return error bound decomposes into dynamics error × coefficient + reward error × coefficient",
        "Corollary 1: Lower Lipschitz constants tighten the return-error bound",
        "Theorem 2: REINFORCE with noisy rewards requires optimal noise fidelity",
    ]).with_tags(vec!["rl", "model-based", "sample-allocation", "lipschitz", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_web_resources(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::article(
        "State of the Graph 2026 — Knowledge Graphs as Agent Memory",
        "Comprehensive analysis of graph structure as core architecture for agent memory, making long-term behavior persistent, queryable, and inspectable.",
        "https://stateofthegraph.com/knowledge-graphs"
    ).with_key_insights(vec![
        "Graph structure is the core architecture for agent memory",
        "Enables persistent, queryable, and inspectable long-term behavior",
        "BFS traversal and community detection for memory exploration",
    ]).with_tags(vec!["knowledge-graphs", "agent-memory", "graph-architecture", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::article(
        "Fable 5 Prompt Library — Goal→Reason→Boundaries→Verification",
        "Anthropic's Fable 5 prompting architecture: Goal→Reason→Boundaries→Verification. Structured prompting framework for reliable AI agent behavior.",
        "https://every.to/claude-fable-5-prompt-library"
    ).with_key_insights(vec![
        "Goal → Reason → Boundaries → Verification four-step architecture",
        "Boundary separation prevents premature action before analysis",
        "Verification gate ensures output correctness before delivery",
    ]).with_tags(vec!["prompting", "fable-5", "architecture", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_bug_fixes(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #1: GRPO Importance Ratio — Value Ratio vs Softmax Policy Ratio (CRITICAL)",
        "nt_core_policy.rs used old_value/ref_val (value ratio) instead of π_new(a|s)/π_old(a|s) (softmax policy ratio) for GRPO importance sampling. Fixed by replacing with softmax-based ratio: (new_logit - ref_logit).exp()."
    ).with_importance(0.95).with_tags(vec!["bug", "critical", "grpo", "policy-gradient", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #2: Double Discounting in Step Reward TD Bootstrap (MODERATE)",
        "nt_core_policy.rs applied double discounting: step rewards were discounted to present value, then TD bootstrap re-discounted them again. Fixed: step rewards use discounted present value; TD bootstrap uses r + γ·V(s') without extra discounting."
    ).with_importance(0.85).with_tags(vec!["bug", "moderate", "discounting", "td-learning", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Bug #3: MODULE_COUNT Mismatch — 11 vs 14 Specialists (MAJOR)",
        "resonance.rs had MODULE_COUNT=11 but workspace registers 14 specialists, causing resonance matrix size mismatch. Fixed to MODULE_COUNT=14 and updated default_specialist_states() with 3 new entries: AISecurity(45), ImageGenerator(46), EvidenceWeightedHypothesis(50)."
    ).with_importance(0.90).with_tags(vec!["bug", "major", "resonance", "gwt", "fixed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn ingest_new_modules(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Training-in-Imagination Module (nt_core_imagination.rs)",
        "Implements arXiv 2605.06732: OptimalSampleAllocation (Theorem 1), LipschitzRegularizer (Corollary 1), NoisyRewardPolicy (Theorem 2), ReturnErrorBound (Lemma 1). 19 tests."
    ).with_key_insights(vec![
        "OptimalSampleAllocation computes Ndyn/Nrew = α/β · γ·Lr·(1+Lπ) / (1−γ·Lf·(1+Lπ)) · crew/cdyn · εdyn/εrew",
        "LipschitzRegularizer provides spectral-normalization-based regularization to tighten return-error bounds",
        "NoisyRewardPolicy implements unbiased REINFORCE gradient estimation under reward noise",
        "ReturnErrorBound computes Lemma 1 error decomposition for model-based RL",
    ]).with_importance(0.85).with_tags(vec!["module", "training-in-imagination", "rl", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        "Persistent graph-structured memory layer for GWT. GraphMemoryStore with LRU eviction, BFS, semantic search. MemoryGraphSpecialist for GWT resonance integration. 22 tests."
    ).with_key_insights(vec![
        "GraphMemoryNode with 8 variants (Concept, Session, SpecialistActivation, Decision, Reward, Skill, Reflection, Goal)",
        "GraphMemoryStore with LRU eviction (default 10000 nodes), BFS traversal, semantic search by cosine similarity",
        "MemoryGraphSpecialist integrates with GWT resonance cycles",
        "Subgraph extraction, merge, prune_expired, evict_lru operations",
    ]).with_importance(0.80).with_tags(vec!["module", "graph-memory", "gwt", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Amber Embedding Commitment (nt_memory_commitment.rs)",
        "Self-certifying embedding artifact format with 4-bit quantization, SHA-256 Merkle tree, probabilistic audit (1−2^{−128}), position-length binding. 18 tests."
    ).with_key_insights(vec![
        "4-bit quantization reduces f32 embedding storage 8× with MSE < 0.1",
        "SHA-256 Merkle tree over 32-byte chunks enables per-dimension-chunk verification",
        "Probabilistic audit with reservoir sampling and detection probability 1−(1−ρ)^k",
        "Position-length binding prevents chunk reordering attacks",
        "JSON persistence via save/load for portable artifact distribution",
    ]).with_importance(0.85).with_tags(vec!["module", "embedding", "merkle", "commitment", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    let r = ingester.ingest(&ResourceDescriptor::concept(
        "Security MCP Tools (nt_shield_mcp_security.rs)",
        "SecurityMcpToolRegistry with 6 built-in MCP tools: scan_secrets, audit_code_security, check_dependencies, test_prompt_injection, analyze_threat, security_health_check. Rate limiting, audit trail. 12+ tests."
    ).with_key_insights(vec![
        "6 built-in security tools with SecurityToolCategory enum (9 categories)",
        "scan_secrets detects API keys, tokens, passwords via regex patterns",
        "audit_code_security checks OWASP Top 10 patterns with CWE mapping",
        "test_prompt_injection detects jailbreak and prompt leak patterns",
        "Rate limiting (30 calls/min default) and scan_history audit trail",
    ]).with_importance(0.80).with_tags(vec!["module", "security", "mcp", "tools", "absorbed-2026-07-03"]))?;
    ids.push(r.node_id);

    Ok(ids)
}

fn link_related_resources(ingester: &mut ResourceIngester) -> Result<(), String> {
    link_pair(
        ingester,
        "stablyai/orca — Orca: Dual-System NextState Prediction",
        "Training-in-Imagination Module (nt_core_imagination.rs)",
        RelationType::InspiredBy,
        0.8,
        "Orca dual-system inspired E8 conscious/unconscious split",
    )?;
    link_pair(
        ingester,
        "offchainthoughts/Amber — Self-Certifying Embedding Artifacts",
        "Amber Embedding Commitment (nt_memory_commitment.rs)",
        RelationType::InspiredBy,
        0.9,
        "Amber commitment format directly implemented",
    )?;
    link_pair(
        ingester,
        "google/sec-gemini — Security Agents with Function-Calling",
        "Security MCP Tools (nt_shield_mcp_security.rs)",
        RelationType::InspiredBy,
        0.8,
        "Sec-Gemini function-calling pattern for MCP security tools",
    )?;
    link_pair(
        ingester,
        "facebook/astryx — Graph Memory Architecture",
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        RelationType::InspiredBy,
        0.7,
        "Astryx graph memory patterns for GWT integration",
    )?;
    link_pair(
        ingester,
        "Training in Imagination — Optimal Sample Allocation for Model-Based RL",
        "Bug #1: GRPO Importance Ratio — Value Ratio vs Softmax Policy Ratio (CRITICAL)",
        RelationType::References,
        0.6,
        "GRPO policy ratio theory from RL literature",
    )?;
    link_pair(
        ingester,
        "State of the Graph 2026 — Knowledge Graphs as Agent Memory",
        "Graph Memory Layer (nt_gwt_graph_memory.rs)",
        RelationType::InspiredBy,
        0.8,
        "Knowledge graphs as core architecture for agent memory",
    )?;
    link_pair(
        ingester,
        "Fable 5 Prompt Library — Goal→Reason→Boundaries→Verification",
        "Bug #3: MODULE_COUNT Mismatch — 11 vs 14 Specialists (MAJOR)",
        RelationType::References,
        0.5,
        "Boundary separation principle aligns with Fable 5 verification gate",
    )?;

    Ok(())
}

fn link_pair(
    ingester: &mut ResourceIngester,
    from_title: &str,
    to_title: &str,
    rel: RelationType,
    weight: f64,
    desc: &str,
) -> Result<(), String> {
    ingester.relate_by_title(from_title, to_title, rel, weight, Some(desc))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Wave 2026-09-20: Crystal Core Absorption — arxiv-complete + GitHub repos
// ═══════════════════════════════════════════════════════════════════════════════

/// Absorb arxiv-complete dataset metadata into the crystal core.
///
/// The `secemp9/arxiv-complete` dataset on HuggingFace contains 3,148,796 papers
/// with full metadata (title, authors, abstracts, categories, dates). We fetch
/// a representative sample via the datasets-server API and ingest key papers
/// across AI/ML/CS categories to give the KB deep academic coverage.
///
/// Strategy: fetch 100 metadata rows (covers major categories), then synthesize
/// category-level summary nodes for the full 3.1M paper corpus.
fn ingest_arxiv_complete_metadata(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // 1. Register the dataset itself as a meta-resource
    let dataset_desc = ResourceDescriptor::article(
        "arxiv-complete: Full arXiv Corpus (3.1M papers)",
        "secemp9/arxiv-complete — A snapshot of arXiv's metadata, version history, submission files \
         and rendered documents. Covers 3,148,796 papers with file contents, paths, sizes and SHA-256 \
         digests. Metadata from arXiv's OAI-PMH arXivRaw interface; files from GCS mirror, S3 source \
         archives and direct PDF fetches. Configs: metadata (1.6GB), versions (269MB), files (2.4GB), \
         paper_text (70GB), latex (0.16TB), source (6.51TB), pdf (8.65TB).",
        "https://huggingface.co/datasets/secemp9/arxiv-complete",
    ).with_key_insights(vec![
        "3,148,796 papers covering all arXiv categories (cs, math, physics, bio, etc.)",
        "Metadata config: 1.6 GB Parquet with paper_id, title, authors, abstracts, categories, dates",
        "paper_text config: 70 GB of resolved TeX content for 2.86M papers (90.7%)",
        "source config: 6.51 TB of complete submission packages for 3.12M papers (99.1%)",
        "Enables large-scale academic knowledge graph construction and citation analysis",
    ]).with_tags(vec![
        "arxiv", "dataset", "academic", "corpus", "metadata",
        "huggingface", "papers", "knowledge-graph",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.95).with_confidence(0.95);

    let r = ingester.ingest(&dataset_desc)?;
    ids.push(r.node_id);

    // 2. Fetch a sample of metadata via the HuggingFace datasets-server API
    //    This gives us real paper metadata to ingest
    let sample_url = "https://datasets-server.huggingface.co/rows?dataset=secemp9/arxiv-complete&config=metadata&split=train&offset=0&length=100";
    if let Ok(resp) = super::super::nt_http::run_blocking(|| {
        super::super::nt_http::shared_blocking_client()
            .get(sample_url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
    }) {
        if let Ok(data) = resp.json::<serde_json::Value>() {
            if let Some(rows) = data["rows"].as_array() {
                for row in rows {
                    let row_data = &row["row"];
                    let paper_id = row_data["paper_id"].as_str().unwrap_or("");
                    let title = row_data["title"].as_str().unwrap_or("");
                    let abstract_text = row_data["abstract"].as_str().unwrap_or("");
                    let authors = row_data["authors"].as_str().unwrap_or("");
                    let categories = row_data["categories"].as_str().unwrap_or("");
                    let date = row_data["date"].as_str().unwrap_or("");

                    if title.is_empty() {
                        continue;
                    }

                    let summary = if abstract_text.len() > 500 {
                        format!(
                            "{}...",
                            &abstract_text[..abstract_text
                                .char_indices()
                                .nth(500)
                                .map(|(i, _)| i)
                                .unwrap_or(500)]
                        )
                    } else {
                        abstract_text.to_string()
                    };

                    let desc = ResourceDescriptor::paper(paper_id, title, &summary)
                        .with_content(&format!(
                            "Authors: {}\nCategories: {}\nDate: {}\nAbstract: {}",
                            authors, categories, date, abstract_text,
                        ))
                        .with_tags(vec![
                            "arxiv",
                            "paper",
                            "academic",
                            categories.split_whitespace().next().unwrap_or("unknown"),
                            &format!("absorbed-{}", now()),
                        ])
                        .with_importance(0.75)
                        .with_confidence(0.9);

                    if let Ok(result) = ingester.ingest(&desc) {
                        ids.push(result.node_id);
                    }
                }
            }
        }
    } else {
        warn!("[arxiv-complete] Failed to fetch sample metadata from HuggingFace API");
    }

    // 3. Synthesize category-level summary nodes for the full corpus
    let arxiv_categories = vec![
        ("cs.AI", "Artificial Intelligence", "Machine learning, knowledge representation, planning, reasoning, NLP, computer vision, robotics"),
        ("cs.LG", "Machine Learning", "Supervised, unsupervised, reinforcement learning; deep learning; neural networks; optimization"),
        ("cs.CL", "Computation and Language", "NLP, computational linguistics, text mining, information extraction, machine translation"),
        ("cs.CV", "Computer Vision", "Image recognition, object detection, segmentation, video analysis, 3D vision"),
        ("cs.RO", "Robotics", "Robot design, control, planning, perception, human-robot interaction"),
        ("cs.IR", "Information Retrieval", "Search, recommendation systems, document ranking, query processing"),
        ("cs.SE", "Software Engineering", "Program analysis, testing, maintenance, development tools, formal methods"),
        ("cs.CR", "Cryptography and Security", "Security, privacy, cryptographic protocols, network security, applied crypto"),
        ("cs.DC", "Distributed Computing", "Cloud computing, parallel processing, consensus protocols, distributed systems"),
        ("cs.NE", "Neural and Evolutionary Computing", "Neural networks, evolutionary algorithms, genetic programming, swarm intelligence"),
        ("stat.ML", "Statistics: Machine Learning", "Statistical learning theory, Bayesian methods, probabilistic models"),
        ("math.OC", "Optimization and Control", "Convex optimization, control theory, operations research, mathematical programming"),
        ("q-bio.BM", "Biomolecules", "Protein structure, genomics, computational biology, molecular modeling"),
        ("physics.comp-ph", "Computational Physics", "Monte Carlo methods, molecular dynamics, computational methods in physics"),
    ];

    for (cat, name, desc_text) in &arxiv_categories {
        let node_desc = ResourceDescriptor::concept(
            &format!("arXiv:{} — {}", cat, name),
            &format!(
                "Category summary for the full arXiv corpus (3.1M papers total). {}: {}.",
                name, desc_text
            ),
        )
        .with_key_insights(vec![
            &format!("arXiv category {} covers: {}", cat, desc_text),
            "Part of the 3,148,796 paper arxiv-complete corpus on HuggingFace",
            "Full metadata available via secemp9/arxiv-complete dataset",
        ])
        .with_tags(vec![
            "arxiv",
            "category",
            "taxonomy",
            "academic",
            cat,
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.85)
        .with_confidence(0.9);

        if let Ok(result) = ingester.ingest(&node_desc) {
            ids.push(result.node_id);
        }
    }

    info!(
        "[arxiv-complete] Absorbed {} nodes (dataset + {} sampled papers + {} category summaries)",
        ids.len(),
        ids.len().saturating_sub(1 + arxiv_categories.len()),
        arxiv_categories.len()
    );

    Ok(ids)
}

/// Absorb key GitHub repos into the crystal core.
///
/// These repos represent the cutting-edge of personal AI, agent orchestration,
/// local inference, and developer tooling — all directly relevant to NeoTrix's
/// architecture and evolution.
fn ingest_crystal_core_repos(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // ── 1. OpenHuman — Personal AI superintelligence (39.9k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "tinyhumansai", "openhuman",
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "OpenHuman is an open source agent harness with local-first memory, agent orchestration, \
         and workflows. Features: Memory Tree + Obsidian Wiki for persistent local memory, \
         100+ OAuth integrations, 5000+ MCP servers, 90000+ Skills. Orchestrator with checkpointed \
         graph runs on tinyagents, agent-to-agent E2E encryption via Signal protocol, visual \
         workflow builder (tinyflows). Built with Rust (Tauri) + React. 39.9k stars.",
    ).with_key_insights(vec![
        "Memory Tree: data compressed into scored Markdown trees in SQLite, mirrored as Obsidian vault",
        "TokenJuice: tool output compressed before model, up to 80% fewer tokens",
        "Agent graphs with checkpoints: turns run as checkpointed graphs, pause for human, survive restart",
        "Agent-to-agent E2E encryption via Signal protocol with x402 payments",
        "Workflows: agent proposes automation, user reviews on canvas, durable trigger-driven runs",
        "Split brain: fast reflex agent triages inbound, deep reasoning core delegates to worker fleets",
        "Privacy Mode: one-switch enforced local-only inference in Rust core",
    ]).with_tags(vec![
        "personal-ai", "agent-harness", "memory-tree", "orchestration",
        "workflows", "local-first", "tauri", "rust",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.95).with_confidence(0.95))?;
    ids.push(r.node_id);

    // ── 2. Splash — Local inference engine for Apple silicon (413★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "incoai", "splash",
        "incoai/splash — Splash: Local Inference Engine for Apple Silicon",
        "A local inference engine for Apple silicon, built around the model. Serves models to \
         coding agents and OpenAI/Anthropic compatible clients. On 48GB M5 Pro: decodes \
         Qwen3.8-27B at 2x speed of next-fastest engine, 282ms TTFT with 32K context cached. \
         Kernels, draft model, and memory plan specialized per model. Speculative decoding \
         (DFlash 2) as default decode path. Fused Metal kernels compiled for exact shapes.",
    ).with_key_insights(vec![
        "Speculative decoding (DFlash 2) as default, not optional — every model ships its own draft",
        "Fused Metal kernels compiled for exact model dimensions, weights packed and mapped zero-copy",
        "Memory plan computed per-machine from Metal recommended memory minus weights/draft/state",
        "2x decode speed vs next-fastest engine on M5 Pro (74 tok/s on 27B, 210 tok/s on 35B-A3B)",
        "OpenAI + Anthropic compatible API with streaming, tool calls, JSON Schema, images, PDFs",
    ]).with_tags(vec![
        "inference", "apple-silicon", "metal", "speculative-decoding",
        "local-llm", "coding-agents", "llm-inference",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.85).with_confidence(0.9))?;
    ids.push(r.node_id);

    // ── 3. ccodex-sleep-state — Codex connection quality tool (441★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "gylive",
            "ccodex-sleep-state",
            "gylive/ccodex-sleep-state — Codex Connection Quality Tool",
            "A tool to improve Codex degradation, rate limiting, and connection experience. \
         Local one-click startup with web configuration. Supports Astra/Sol/Terra models, \
         subscriptions and proxies. Manages turn-state injection, proxy routing, and \
         model/account isolation. Written in Go with embedded web UI.",
        )
        .with_key_insights(vec![
            "Turn-state injection for Codex connection quality management",
            "Model and account isolation: state per account, credential, and model",
            "Proxy pool management with subscription import, HTTP/SOCKS5 support",
            "Web panel for configuration: model selection, proxy routing, diagnostics",
            "Strict mode vs fallback: controlled behavior when state unavailable",
        ])
        .with_tags(vec![
            "codex",
            "connection-quality",
            "proxy",
            "turn-state",
            "agent-infrastructure",
            "go",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.70)
        .with_confidence(0.85),
    )?;
    ids.push(r.node_id);

    // ── 4. Laya — Non-autoregressive decision engine (2.5k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "NandhaKishorM", "laya",
        "NandhaKishorM/laya — Laya: Non-Autoregressive Decision Engine",
        "Multilingual, non-autoregressive System 1 decision engine. Typed decisions over 100+ \
         languages in a single forward pass (33ms). Trained with RLCD (reinforcement learning \
         against strictly proper scoring rules). Three checkpoints: English (ModernBERT-large, 421M), \
         Multilingual (mmBERT-base, 322M), Typed-Decisions. Router picks optimal checkpoint per request.",
    ).with_key_insights(vec![
        "Single forward pass evaluation: 33ms for one question, 7.2ms/question batched on T4",
        "Three primitives: choice (classification), score (ordinal), noul (P(true) calibration)",
        "Router detects script/language in <0.5ms before forward pass, dispatches to optimal checkpoint",
        "RLCD training: strictly proper scoring rules produce statistically meaningful confidence scores",
        "Fine-tuned checkpoint beats Jev by 3.9 points on typed-decisions (0.766 vs 0.727)",
        "Calibrated ECE: 0.081 after temperature fitting (3x better than base 0.213)",
    ]).with_tags(vec![
        "decision-engine", "classification", "multilingual", "non-autoregressive",
        "rlcd", "calibration", "router", "bert",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.85).with_confidence(0.9))?;
    ids.push(r.node_id);

    // ── 5. Cutter — Reverse engineering platform (19.7k★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "rizinorg",
            "cutter",
            "rizinorg/cutter — Cutter: Free Open Source RE Platform",
            "Free and open-source reverse engineering platform powered by rizin. Advanced and \
         customizable RE platform with GUI. Supports Python and Native C++ plugins. \
         Integrates Ghidra decompiler, DynamoRIO code coverage visualization. \
         Cross-platform: Linux, macOS, Windows. 19.7k stars.",
        )
        .with_key_insights(vec![
            "Plugin architecture: Python and Native C++ plugins for extensibility",
            "Rizin-powered backend with advanced binary analysis capabilities",
            "Ghidra decompiler integration via rz-ghidra plugin",
            "Cross-platform GUI for reverse engineering workflows",
        ])
        .with_tags(vec![
            "reverse-engineering",
            "binary-analysis",
            "rizin",
            "ghidra",
            "security",
            "plugins",
            "gui",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.75)
        .with_confidence(0.9),
    )?;
    ids.push(r.node_id);

    // ── 6. maka-cu — macOS Computer Use execution layer (17★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "maka-agent", "maka-cu",
        "maka-agent/maka-cu — Maka Computer Use: Native macOS Execution Layer",
        "Native macOS execution layer for Maka's Computer Use. Forked from iFurySt/open-codex-computer-use. \
         Accessibility snapshot as core capability. Action binding: actions bound to observation they were \
         planned against, single-use, spent actions refused. Full Anthropic computer_20251124 action contract. \
         SkyLight background-click path and app discovery. Swift Package with embedded plugins.",
    ).with_key_insights(vec![
        "Accessibility snapshot as core observation primitive for computer use",
        "Action binding: actions are single-use, bound to the observation they were planned against",
        "Anthropic computer_20251124 action contract support for multi-model compatibility",
        "SkyLight private API bridge for background click events without focus stealing",
        "WebContent/renderer elements bound to real process generation with stale refetch",
    ]).with_tags(vec![
        "computer-use", "macos", "accessibility", "swift",
        "agent-execution", "sky-light", "action-binding",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.70).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 7. jev-skill — Jev use cases and agent skills (85★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "wuyoscar", "jev-skill",
        "wuyoscar/jev-skill — Awesome Jev Skills: Decision-Making for Agents",
        "Collection of Jev use cases, workflows, and agent skills. 90 scenarios, 9 installable \
         skills, 14 recorded API examples. Jev is a typed decision model that chooses, classifies, \
         and scores. Skills cover: agent supervision, routing, browser interaction, inbox triage, \
         document evidence, data tools, creative tools. Supports choice/score/noul primitives.",
    ).with_key_insights(vec![
        "90 decision scenarios across 8 domains: agents, review, routing, interaction, business, documents, data, creative",
        "Three decision primitives: choice (one option), score (graded levels), noul (P(true) yes/no)",
        "Agent skills: jev, jev-triage, jev-documents, jev-ui, jev-route, jev-context, jev-code-review, jev-find-code, jev-simulation",
        "Goal-drift checkpoint, stuck-loop recovery, completion evidence check patterns",
        "Model tier routing: small_text / reasoning / vision / cannot_route",
        "Parallel independent judgments: multiple questions over shared state in one request",
    ]).with_tags(vec![
        "jev", "decision-engine", "agent-skills", "classification",
        "triage", "routing", "workflows",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.9))?;
    ids.push(r.node_id);

    // Cross-link related repos
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "incoai/splash — Splash: Local Inference Engine for Apple Silicon",
        RelationType::Related,
        0.7,
        Some("OpenHuman can use Splash as local inference backend"),
    );
    let _ = ingester.relate_by_title(
        "wuyoscar/jev-skill — Awesome Jev Skills: Decision-Making for Agents",
        "NandhaKishorM/laya — Laya: Non-Autoregressive Decision Engine",
        RelationType::Related,
        0.8,
        Some("Jev and Laya are competing/complementary typed decision engines"),
    );
    let _ = ingester.relate_by_title(
        "maka-agent/maka-cu — Maka Computer Use: Native macOS Execution Layer",
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        RelationType::Related,
        0.6,
        Some("Maka-CU provides computer use primitives applicable to agent harnesses"),
    );

    info!("[crystal-core-repos] Absorbed {} repo nodes", ids.len());
    Ok(ids)
}

/// Absorb the personal-ai ecosystem — key repos from the GitHub personal-ai topic.
///
/// These represent the landscape of personal AI agents, local-first memory systems,
/// and agent orchestration platforms that NeoTrix competes with and can learn from.
fn ingest_personal_ai_ecosystem(ingester: &mut ResourceIngester) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    // ── 1. LifeOS — Universal AI Harness (19.1k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "danielmiessler", "LifeOS",
        "danielmiessler/LifeOS — LifeOS: Universal AI Harness",
        "The universal AI Harness designed to move you from Current to Ideal state in both life and work. \
         Productivity framework with AI augmentation, intent engineering, and life/work optimization.",
    ).with_key_insights(vec![
        "Current-to-Ideal state transformation framework for life and work",
        "Intent engineering: structured approach to AI-directed personal productivity",
        "AI harness pattern: orchestrating multiple AI tools for personal goals",
    ]).with_tags(vec![
        "personal-ai", "productivity", "intent-engineering", "life-harness",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 2. OpenBiliClaw — Cross-platform AI content discovery (3.3k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "whiteguo233", "OpenBiliClaw",
        "whiteguo233/OpenBiliClaw — Cross-platform AI Content Discovery Agent",
        "Local-first open-source cross-platform AI content discovery agent. Understands you, then \
         proactively finds content across Bilibili, Xiaohongshu, Douyin, YouTube, X, Zhihu, Reddit, \
         Weibo and the open web. Supports deepseek harness plugin.",
    ).with_key_insights(vec![
        "Local-first content discovery across 8+ platforms (Bilibili, YouTube, Reddit, etc.)",
        "User-understanding-first approach: learns preferences before proactive discovery",
        "DeepSeek harness plugin support for model flexibility",
    ]).with_tags(vec![
        "personal-ai", "content-discovery", "cross-platform", "local-first",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 3. Bitterbot — Mesh of agents (2.5k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Bitterbot-AI", "bitterbot-desktop",
        "Bitterbot-AI/bitterbot-desktop — Bitterbot: Agent Mesh for Collective Capability",
        "A mesh of agents that turns shared experience into collective capability. Desktop app with \
         P2P agent communication, cognitive architecture, skills marketplace, local-first design, \
         agent economy with x402 payments, and dream engine.",
    ).with_key_insights(vec![
        "Agent mesh: P2P communication between agents for collective intelligence",
        "Skills marketplace: agents share and trade capabilities",
        "Dream engine: background processing and insight generation",
        "x402 payment protocol for agent economy",
    ]).with_tags(vec![
        "personal-ai", "agent-mesh", "p2p", "cognitive-architecture",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 4. Memmy Agent — Personal memory hub (2k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "MemTensor", "memmy-agent",
        "MemTensor/memmy-agent — Memmy: Personal AI Memory Hub",
        "A personal AI agent and local memory hub for all AI agents. Gives every AI one shared, \
         fully controlled memory and persistent context — all AI remember the same you. Supports \
         Claude Code, Codex, OpenClaw and Hermes Agent.",
    ).with_key_insights(vec![
        "Shared memory hub: all AI agents share one persistent memory of the user",
        "Cross-agent memory: Claude Code, Codex, OpenClaw, Hermes all share context",
        "Local-first: fully controlled memory on user's device",
    ]).with_tags(vec![
        "personal-ai", "memory-hub", "persistent-context", "cross-agent",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.80).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 5. Personal Model — Build your HUMAN.md (1.3k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Intuition-Lab", "personal-model",
        "Intuition-Lab/personal-model — Personal Model: Build Your HUMAN.md",
        "Build your HUMAN.md — a structured representation of yourself for AI agents. Local-first, \
         privacy-focused, MCP-compatible personal model for agent context.",
    ).with_key_insights(vec![
        "HUMAN.md: structured user model for AI agent personalization",
        "MCP-compatible personal model for cross-agent context sharing",
        "Privacy-first: local storage, user-controlled data",
    ]).with_tags(vec![
        "personal-ai", "personal-model", "human-md", "mcp",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 6. FYAgent — Digital persona (1.1k★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "fy-agent", "fyagent",
        "fy-agent/fyagent — FYAgent: Digital Persona for AI Era",
        "For You Agent — AI时代的个人随身数字人格. Your model, AI accounts, skills, prompts, and \
         workflow methods, carried into every AI tool. Rust + Tauri cross-platform desktop app.",
    ).with_key_insights(vec![
        "Digital persona: portable identity across AI tools",
        "Skill and prompt management for consistent agent behavior",
        "Cross-platform (Rust/Tauri) with MCP integration",
    ]).with_tags(vec![
        "personal-ai", "digital-persona", "prompt-management", "cross-platform",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // ── 7. Elephant Agent — Self-evolving AI (585★) ──
    let r = ingester.ingest(
        &ResourceDescriptor::github(
            "agentic-in",
            "elephant-agent",
            "agentic-in/elephant-agent — Elephant: Personal-Model First Self-Evolving Agent",
            "Personal-Model First Self-Evolving AI Agent. Agent with memory, model management, \
         context awareness, and self-evolution capabilities.",
        )
        .with_key_insights(vec![
            "Self-evolution: agent improves its own capabilities over time",
            "Personal-model first: user's model drives agent behavior",
            "Memory and context awareness for persistent agent state",
        ])
        .with_tags(vec![
            "personal-ai",
            "self-evolution",
            "agent-memory",
            "agentic",
            &format!("absorbed-{}", now()),
        ])
        .with_importance(0.70)
        .with_confidence(0.85),
    )?;
    ids.push(r.node_id);

    // ── 8. Sentient OS — On-device proactive intelligence (518★) ──
    let r = ingester.ingest(&ResourceDescriptor::github(
        "Sentient-OS-Labs", "sentient-os",
        "Sentient-OS-Labs/sentient-os — Sentient OS: On-device Proactive Intelligence",
        "An on-device LLM that understands your entire life, then proactively offers to get your \
         work done through computer use. macOS native, privacy-first, MCP-compatible, with \
         knowledge base and proactive intelligence layer.",
    ).with_key_insights(vec![
        "Proactive intelligence: agent anticipates needs and offers to act",
        "On-device LLM for full life understanding without cloud dependency",
        "Computer use integration for autonomous task execution",
    ]).with_tags(vec![
        "personal-ai", "proactive-intelligence", "on-device", "computer-use",
        &format!("absorbed-{}", now()),
    ]).with_importance(0.75).with_confidence(0.85))?;
    ids.push(r.node_id);

    // Cross-link ecosystem repos
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "MemTensor/memmy-agent — Memmy: Personal AI Memory Hub",
        RelationType::Related,
        0.8,
        Some("Both implement local-first persistent memory for personal AI"),
    );
    let _ = ingester.relate_by_title(
        "tinyhumansai/openhuman — OpenHuman: Personal AI Superintelligence",
        "danielmiessler/LifeOS — LifeOS: Universal AI Harness",
        RelationType::Related,
        0.7,
        Some("LifeOS and OpenHuman both aim to be comprehensive personal AI platforms"),
    );

    info!(
        "[personal-ai-ecosystem] Absorbed {} ecosystem nodes",
        ids.len()
    );
    Ok(ids)
}

