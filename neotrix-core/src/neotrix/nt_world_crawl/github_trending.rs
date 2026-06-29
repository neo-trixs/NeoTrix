use crate::core::nt_core_data_types::{DataSourceRecord, DataSourceType};
use crate::core::nt_core_time::unix_now;

/// Pure knowledge module — no HTTP, no duplicate client.
/// Raw fetch is done by data_connector.rs::fetch_github_trending().
pub struct GitHubTrending;

impl GitHubTrending {
    pub fn key_insights_section() -> &'static [(&'static str, &'static str, f64)] {
        &[
            ("codebase-memory-mcp: persistent knowledge graph",
             "C-based, zero-dependency MCP server indexing codebases into SQLite knowledge graph via tree-sitter AST. 158 languages, 120x fewer tokens than file-by-file search. 11 agent auto-detection. arXiv:2603.27277",
             0.92),
            ("headroom: token compression layer",
             "Compresses LLM context 60-95% via 6 algorithms (JSON/AST/ML). Library, proxy, MCP server. CacheAligner for KV cache hits. Reversible CCR. cross-agent memory. headroom learn mines failures -> CLAUDE.md corrections.",
             0.90),
            ("ponytail: lazy senior dev pattern",
             "JavaScript plugin achieving -54% LOC, -22% tokens, -20% cost, -27% time, 100% safe. 5-rung ladder: YAGNI->stdlib->native->one-liner->minimum. Works with 14 agents. Benchmark-verified on real FastAPI+React repo.",
             0.88),
            ("MiMo-Code: persistent memory coding agent",
             "Xiaomi's OpenCode fork with SQLite FTS5 cross-session memory, subagent orchestration (build/plan/compose), intelligent context management, goal/stop conditions with independent judge, dream/distill for self-improvement.",
             0.85),
            ("NVIDIA/SkillSpector: agent security scanner",
             "Scans AI agent skills for security vulnerabilities, running in CI/CD pipeline. +4,633★/week at 6.9K total. Directly relevant to NeoTrix's shield and safety layers.",
             0.82),
            ("Agent-Reach: universal web extraction",
             "CLI tool for agents to read/search Twitter, Reddit, YouTube, GitHub, Bilibili, RSS. Multi-source extraction directly applicable to NeoTrix crawl pipeline.",
             0.80),
        ]
    }

    pub fn architecture_patterns() -> &'static [(&'static str, &'static str)] {
        &[
            ("knowledge-graph-for-agents",
             "codebase-memory-mcp proves persistent knowledge graphs are the missing layer for AI coding agents. Pure C single binary, tree-sitter AST + Hybrid LSP, SQLite backend. NeoTrix should absorb: extend existing KnowledgeEngine with tree-sitter-like structural indexing, not just text embedding."),
            ("compression-proxy-before-llm",
             "headroom shows a compression proxy between agent and LLM is viable at scale. 6 algorithms, reversible CCR, cross-agent memory, failure learning. NeoTrix should absorb: add compression layer in nt_shield between consciousness output and LLM calls."),
            ("yagni-code-generation",
             "ponytail proves code reduction is measurable and safe. -54% LOC, 100% safe via structured guardrails. NeoTrix should absorb: integrate ponytail's 5-rung ladder into Ne compiler's codegen pipeline."),
            ("persistent-memory-fork",
             "MiMo-Code validates persistent cross-session memory as the key differentiator for coding agents. SQLite FTS5, token-budgeted context injection, dream/distill self-improvement. NeoTrix should absorb: extend HyperCube VSA memory with MiMo's checkpoint architecture."),
        ]
    }

    pub fn feed_text() -> Vec<String> {
        vec![
            format!(
                "[GitHub Trending Absorbed] Top weekly velocity repos: \
                 agent-skills +11,088★, headroom +10,660★, apple/container +10,541★, \
                 last30days-skill +9,676★, Agent-Reach +5,873★, NVIDIA/SkillSpector +4,633★. \
                 New this week: ponytail 38K★ (54% less code), MiMo-Code 8.7K★ (persistent memory)."
            ),
            format!(
                "[Architecture Insight] 4 key patterns absorbed: \
                 (1) Knowledge graphs for agents via tree-sitter AST - extends VSA-based knowledge engine, \
                 (2) Token compression proxy before LLM reduces costs 60-95%, \
                 (3) YAGNI code generation reduces code 54% while maintaining 100% safety, \
                 (4) Persistent cross-session memory via SQLite FTS5 enables agent continuity."
            ),
            format!(
                "[Ecosystem Signal] The 'skill as product surface' paradigm has matured. \
                 agent-skills (61K★), ponytail (38K★), and NVIDIA SkillSpector (6.9K★) \
                 show the market is standardizing on composable agent capabilities. \
                 NeoTrix's CapabilitySynthesizer is aligned with this direction."
            ),
        ]
    }

    pub fn into_feed_records() -> Vec<DataSourceRecord> {
        let now = unix_now();
        Self::key_insights_section()
            .iter()
            .map(|(title, summary, score)| DataSourceRecord {
                title: title.to_string(),
                summary: summary.to_string(),
                url: String::new(),
                source_type: DataSourceType::GitHubTrending,
                topics: vec!["trending".to_string(), "insight".to_string()],
                score: *score,
                timestamp: now,
            })
            .collect()
    }
}
