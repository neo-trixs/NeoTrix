use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde_json;
use uuid::Uuid;

use super::nt_memory_store::*;
use super::nt_memory_types::*;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone)]
pub enum ResourceSource {
    GitHub { owner: String, repo: String },
    ArXiv { id: String },
    Web { url: String },
    Direct, // conceptual / built-in
}

impl ResourceSource {
    pub fn url(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { owner, repo } => {
                Some(format!("https://github.com/{}/{}", owner, repo))
            }
            ResourceSource::ArXiv { id } => Some(format!("https://arxiv.org/abs/{}", id)),
            ResourceSource::Web { url } => Some(url.clone()),
            ResourceSource::Direct => None,
        }
    }

    pub fn domain(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { .. } => Some("github.com".into()),
            ResourceSource::ArXiv { .. } => Some("arxiv.org".into()),
            ResourceSource::Web { url } => {
                url.split('/').nth(2).or(url.split('/').nth(0)).map(|d| d.to_string())
            }
            ResourceSource::Direct => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub category: NodeType,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub source: ResourceSource,
    pub key_insights: Vec<String>,
    pub tags: Vec<String>,
    pub importance: f64,
    pub confidence: f64,
}

impl ResourceDescriptor {
    pub fn github(owner: &str, repo: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Repository,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::GitHub { owner: owner.to_string(), repo: repo.to_string() },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.7,
            confidence: 0.9,
        }
    }

    pub fn paper(arxiv_id: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Paper,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::ArXiv { id: arxiv_id.to_string() },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.8,
            confidence: 0.9,
        }
    }

    pub fn article(title: &str, summary: &str, url: &str) -> Self {
        Self {
            category: NodeType::Article,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Web { url: url.to_string() },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn tool(name: &str, summary: &str, source: ResourceSource) -> Self {
        Self {
            category: NodeType::Tool,
            title: name.to_string(),
            summary: summary.to_string(),
            content: None,
            source,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn concept(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Concept,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn insight(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Insight,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn with_key_insights(mut self, insights: Vec<&str>) -> Self {
        self.key_insights = insights.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }
}

#[derive(Debug)]
pub struct ResourceIngestResult {
    pub node_id: String,
    pub insight_ids: Vec<String>,
}

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
        Self { conn, episode_id, ingest_log: Vec::new() }
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
            language: "en".into(),
            confidence: desc.confidence,
            importance: desc.importance,
            created_at: ts,
            updated_at: ts,
            access_count: 0,
            metadata: Some(metadata),
            temporal: None,
            supersedes: None,
            source_episode: Some(self.episode_id.clone()),
        };

        insert_node(self.conn, &node).map_err(|e| format!("insert_node failed: {}", e))?;
        let node_id = node.id.clone();

        let mut insight_ids = Vec::new();

        for insight_text in &desc.key_insights {
            let insight_title = if insight_text.len() > 80 {
                let cut = (0..=77).rev().find(|&i| insight_text.is_char_boundary(i)).unwrap_or(77);
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
                importance: desc.importance * 0.7,
                created_at: ts,
                updated_at: ts,
                access_count: 0,
                metadata: Some(serde_json::json!({
                    "parent_resource_id": node_id,
                    "episode_id": self.episode_id,
                })),
                temporal: None,
                supersedes: None,
                source_episode: Some(self.episode_id.clone()),
            };

            let iid = insight_node.id.clone();
            insert_node(self.conn, &insight_node)
                .map_err(|e| format!("insert insight node failed: {}", e))?;
            upsert_edge(self.conn, &node_id, &iid, RelationType::Supports, 0.7,
                Some("Key insight derived from resource"))
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

        Ok(ResourceIngestResult { node_id, insight_ids })
    }

    pub fn relate(&self, from_id: &str, to_id: &str, rel: RelationType, weight: f64, desc: Option<&str>) -> Result<(), String> {
        upsert_edge(self.conn, from_id, to_id, rel, weight, desc)
            .map_err(|e| format!("relate failed: {}", e))
    }

    pub fn relate_by_title(&self, from_title: &str, to_title: &str, rel: RelationType, weight: f64, desc: Option<&str>) -> Result<(), String> {
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
        lines.push(format!("Total resources ingested: {}", self.ingest_log.len()));
        for entry in &self.ingest_log {
            let status = if entry.status == "success" { "✅" } else { "❌" };
            lines.push(format!("  {} {} ({:?}) — {}", status, entry.title, entry.node_type, entry.node_id));
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

fn find_node_by_title(conn: &Connection, title: &str) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata
         FROM nodes WHERE title=?1 LIMIT 1"
    )?;
    let mut rows = stmt.query(rusqlite::params![title])?;
    match rows.next()? {
        Some(row) => Ok(Some(KnowledgeNode {
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
            metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: None,
            source_episode: None,
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
    link_pair(ingester, "stablyai/orca — Orca: Dual-System NextState Prediction", "Training-in-Imagination Module (nt_core_imagination.rs)", RelationType::InspiredBy, 0.8, "Orca dual-system inspired E8 conscious/unconscious split")?;
    link_pair(ingester, "offchainthoughts/Amber — Self-Certifying Embedding Artifacts", "Amber Embedding Commitment (nt_memory_commitment.rs)", RelationType::InspiredBy, 0.9, "Amber commitment format directly implemented")?;
    link_pair(ingester, "google/sec-gemini — Security Agents with Function-Calling", "Security MCP Tools (nt_shield_mcp_security.rs)", RelationType::InspiredBy, 0.8, "Sec-Gemini function-calling pattern for MCP security tools")?;
    link_pair(ingester, "facebook/astryx — Graph Memory Architecture", "Graph Memory Layer (nt_gwt_graph_memory.rs)", RelationType::InspiredBy, 0.7, "Astryx graph memory patterns for GWT integration")?;
    link_pair(ingester, "Training in Imagination — Optimal Sample Allocation for Model-Based RL", "Bug #1: GRPO Importance Ratio — Value Ratio vs Softmax Policy Ratio (CRITICAL)", RelationType::References, 0.6, "GRPO policy ratio theory from RL literature")?;
    link_pair(ingester, "State of the Graph 2026 — Knowledge Graphs as Agent Memory", "Graph Memory Layer (nt_gwt_graph_memory.rs)", RelationType::InspiredBy, 0.8, "Knowledge graphs as core architecture for agent memory")?;
    link_pair(ingester, "Fable 5 Prompt Library — Goal→Reason→Boundaries→Verification", "Bug #3: MODULE_COUNT Mismatch — 11 vs 14 Specialists (MAJOR)", RelationType::References, 0.5, "Boundary separation principle aligns with Fable 5 verification gate")?;

    Ok(())
}

fn link_pair(ingester: &mut ResourceIngester, from_title: &str, to_title: &str, rel: RelationType, weight: f64, desc: &str) -> Result<(), String> {
    ingester.relate_by_title(from_title, to_title, rel, weight, Some(desc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_ingest_github_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("testowner", "testrepo", "Test Repo", "A test repository for testing.");
        let result = ingester.ingest(&desc).unwrap();
        assert!(!result.node_id.is_empty());
        assert!(result.insight_ids.is_empty());

        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Repo");
        assert_eq!(fetched.node_type, NodeType::Repository);
    }

    #[test]
    fn test_ingest_with_insights() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Repo With Insights", "Desc")
            .with_key_insights(vec!["Insight one", "Insight two", "Insight three"]);
        let result = ingester.ingest(&desc).unwrap();
        assert_eq!(result.insight_ids.len(), 3);

        for iid in &result.insight_ids {
            let fetched = get_node(&conn, iid).unwrap().unwrap();
            assert_eq!(fetched.node_type, NodeType::Insight);
        }

        let edges = get_edges_for_node(&conn, &result.node_id).unwrap();
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_ingest_paper_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("1234.56789", "Test Paper", "A test paper abstract.")
            .with_tags(vec!["test", "paper"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Paper);
        assert!(fetched.url.unwrap().contains("arxiv.org"));
    }

    #[test]
    fn test_ingest_article_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::article("Test Article", "Summary", "https://example.com/article");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Article);
        assert_eq!(fetched.url.unwrap(), "https://example.com/article");
    }

    #[test]
    fn test_ingest_concept_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Test Concept", "A conceptual insight.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Concept);
        assert!(fetched.url.is_none());
    }

    #[test]
    fn test_ingest_tool_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::tool("Test Tool", "A tool description.",
            ResourceSource::GitHub { owner: "test".into(), repo: "tool".into() });
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Tool);
    }

    #[test]
    fn test_ingest_with_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("0000.00000", "Paper With Content", "Abstract")
            .with_content("Full paper content here\nwith multiple lines.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.unwrap().contains("Full paper content"));
    }

    #[test]
    fn test_ingest_metadata_includes_episode() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Episode Concept", "Has episode tracking.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["episode_id"], serde_json::json!(ingester.episode_id()));
        assert_eq!(fetched.source_episode, None);
    }

    #[test]
    fn test_ingest_multiple_and_relate() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let r1 = ingester.ingest(&ResourceDescriptor::concept("Concept A", "First concept.")).unwrap();
        let r2 = ingester.ingest(&ResourceDescriptor::concept("Concept B", "Second concept.")).unwrap();

        ingester.relate(&r1.node_id, &r2.node_id, RelationType::References, 0.8, Some("A references B")).unwrap();

        let edges = get_edges_for_node(&conn, &r1.node_id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_id, r2.node_id);
        assert_eq!(edges[0].relation_type, RelationType::References);
    }

    #[test]
    fn test_ingest_with_importance_and_confidence() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Important Concept", "Very important.")
            .with_importance(0.95)
            .with_confidence(0.99);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!((fetched.importance - 0.95).abs() < 0.01);
        assert!((fetched.confidence - 0.99).abs() < 0.01);
    }

    #[test]
    fn test_ingest_report_format() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester.ingest(&ResourceDescriptor::concept("Report Test", "Testing report.")).unwrap();
        let report = ingester.report();
        assert!(report.contains("Report Test"));
        assert!(report.contains("Episode ID"));
        assert!(report.contains("1"));
    }

    #[test]
    fn test_ingest_domain_from_source() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Domain Test", "Testing domain.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.domain.unwrap(), "github.com");
    }

    #[test]
    fn test_ingest_source_url() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("owner", "repo", "URL Test", "Testing URL.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.url.unwrap().contains("github.com/owner/repo"));
    }

    #[test]
    fn test_ingest_with_tags_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("9999.99999", "Tagged Paper", "Abstract")
            .with_tags(vec!["tag1", "tag2", "absorbed-2026-07-03"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        let tags: Vec<String> = serde_json::from_value(meta["tags"].clone()).unwrap();
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"absorbed-2026-07-03".to_string()));
    }

    #[test]
    fn test_relate_by_title() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester.ingest(&ResourceDescriptor::concept("Source Node", "Source.")).unwrap();
        ingester.ingest(&ResourceDescriptor::concept("Target Node", "Target.")).unwrap();
        ingester.relate_by_title("Source Node", "Target Node", RelationType::DependsOn, 0.9, Some("depends")).unwrap();

        let src = find_node_by_title(&conn, "Source Node").unwrap().unwrap();
        let edges = get_edges_for_node(&conn, &src.id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation_type, RelationType::DependsOn);
    }

    #[test]
    fn test_find_node_by_title_not_found() {
        let conn = test_conn();
        let result = find_node_by_title(&conn, "NonExistentNode").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ingest_session_resources_creates_all_nodes() {
        let conn = test_conn();
        let report = ingest_session_resources(&conn).unwrap();
        assert!(report.contains("Episode ID"));
        let all_nodes = get_all_nodes(&conn).unwrap();
        assert!(all_nodes.len() >= 17);
    }

    #[test]
    fn test_ingest_insight_counts_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("8888.88888", "Insight Count Test", "Abstract")
            .with_key_insights(vec!["A", "B", "C", "D", "E"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["insight_count"], 5);
    }

    #[test]
    fn test_ingest_concept_without_insights_no_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("No Insights", "Just a concept.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.is_none());
    }

    #[test]
    fn test_ingest_long_insight_title_truncated() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let long = "A very long insight text that exceeds eighty characters in total length for truncation testing purposes";
        let desc = ResourceDescriptor::github("o", "r", "Truncation Test", "Summary")
            .with_key_insights(vec![long]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.insight_ids[0]).unwrap().unwrap();
        assert!(fetched.title.len() <= 80);
        assert!(fetched.title.ends_with("..."));
        assert_eq!(fetched.summary.unwrap(), long);
    }
}
