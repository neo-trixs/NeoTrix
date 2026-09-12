# Trending Rankings — Cycle 398 (2026-09-12)

## Signal Sources
GitHub Trending (llm-agents, llm-tools, ai-agent-tools topics), ProductHunt Q2 2026 report, arXiv cs.AI/cs.CL/cs.LG, GitTrend AI agent rankings, awesome-ai-agents-2026 curated list.

---

## T1 — Multi-Agent Coordination (NT-CORE / NT-ACT)

### 1. Open Multi-Agent (OMA)
- **URL**: https://github.com/open-multi-agent/open-multi-agent
- **Stars**: 6,872 | **Language**: TypeScript | **License**: MIT
- **What**: Multi-agent orchestration — describe the goal, not the graph. Coordinator plans task DAG at runtime, deterministic scheduler executes across team. 13 built-in providers. Run viewer with DAG + span waterfall.
- **Novel Pattern**: Runtime DAG generation from natural language goals. No static pipeline definition. Data-first execution — every run leaves a verifiable, replayable record.
- **NeoTrix Mapping**: NT-ACT orchestration (replaces static workflow graphs). NT-CORE GWT attention could enhance task routing. Aligns with **P2: Isolation-per-Task**.
- **Cycle Gap Check**: Not in cycles 318-397 trending lists.

### 2. AgensFlow
- **URL**: https://arxiv.org/abs/2605.27466 | **Code**: open-source (mentioned)
- **Stars**: N/A (paper) | **Venue**: arXiv cs.MA, May 2026
- **What**: Coordination-policy substrate for multi-agent systems. Treats orchestration as online policy-learning under partial observability. Inspectable policy graph over skills, models, and topology.
- **Novel Pattern**: Learned routing beats fixed pipelines on coordination-heavy tasks. Warm-started policy graphs reduce exploration cost. Reward-signal auditability as first-class design.
- **NeoTrix Mapping**: NT-MIND SEAL pipeline (policy learning). NT-CORE GWT salience routing. Directly supports **A1: Cost-Aware Routing**.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## T2 — LLM Routing & Cost Optimization (NT-IO / NT-CORE)

### 3. xrouter-llm
- **URL**: https://github.com/xorbitsai/xrouter-llm
- **Stars**: 38 | **Language**: Python | **License**: Xagent Source License
- **What**: Prompt-aware LLM routing — predicts which models can complete a request, selects cheapest capable. 53.2% cost reduction, +1.9 pts completion improvement on tested dataset.
- **Novel Pattern**: Intent-classification routing with trained router artifact. IRT (Item Response Theory) model profiles per LLM. Does NOT call LLMs — pure routing decision service.
- **NeoTrix Mapping**: NT-IO provider routing. Validates **A1: Cost-Aware Routing** axiom. Complements NT-IO gateway with trained classifier approach.
- **Cycle Gap Check**: Not in cycles 318-397.

### 4. ToolRank
- **URL**: https://toolrank.dev
- **Stage**: Unfunded, Founded 2026
- **What**: Platform optimizing AI agent tool discovery and selection. Scores tool definitions across findability, clarity, precision, efficiency. LLM selection tournaments + runtime reliability testing.
- **Novel Pattern**: Tool-as-Product scoring — treats tool definitions like SEO: findability metrics, rewrite proposals, category rankings. Agent framework SDK for integration.
- **NeoTrix Mapping**: NT-ACT tool registry optimization. MCP tool quality scoring for capability discovery. Could replace manual tool descriptions in CapabilityRegistry.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## T3 — Semantic Memory & Agent Recall (NT-MEMORY)

### 5. MOSS (Memory-Orchestrated Semantic System)
- **URL**: https://arxiv.org/abs/2607.04391
- **What**: Auditable agentic memory over structured relational DB. Agent drives retrieval — no embedding similarity, purely symbolic and reproducible. Year-long production deployment (44M tokens, 163K documents, 569 concepts).
- **Novel Pattern**: Rejects RAG embedding paradigm entirely. Derives vocabulary from corpus (inductive concepts). Every retrieval step logged and inspectable. Model-agnostic, storage-agnostic.
- **NeoTrix Mapping**: NT-MEMORY KB architecture validation. Supports KB-as-source-of-truth. Inverse approach to VSA HyperCube — worth benchmarking against embedding retrieval.
- **Cycle Gap Check**: Not in cycles 318-397.

### 6. Mneme
- **URL**: https://github.com/dlrik/mneme-ai
- **Stars**: 1 | **Language**: Python | **License**: MIT
- **What**: Semantic memory layer combining vector search (ChromaDB + Voyage AI 1024-dim), knowledge graph (entity triples + forward-chaining), episodic tracking. Context injector with token-budget awareness.
- **Novel Pattern**: Triple-layer memory: facts (SQLite, importance-decay) + entity graph (SPO triples) + episodic (session grouping). Prompt-ready context injection with token budgets.
- **NeoTrix Mapping**: NT-MEMORY knowledge representation. Entity graph maps to KB edges. Importance-decay aligns with experience-tree branch relevance scoring.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## T4 — Token Compression & Efficiency (NT-CORE / NT-MEMORY)

### 7. Claw Compactor
- **URL**: https://github.com/open-compress/claw-compactor
- **What**: 14-stage Fusion Pipeline for LLM token compression. Reversible compression, AST-aware code analysis, intelligent content routing. Zero LLM inference cost.
- **Novel Pattern**: 14-stage deterministic pipeline — no LLM calls for compression itself. AST awareness preserves code structure. Reversible: can reconstruct original from compressed.
- **NeoTrix Mapping**: NT-CORE context compression for long sessions. Complements Axiom A2 (Context as Scarce Resource). Could optimize GWT broadcast payload size.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## T5 — Code Intelligence (NT-ACT / NT-IO)

### 8. open-codebase-index
- **URL**: https://github.com/Helweg/open-codebase-index
- **Stars**: 185 | **Language**: Rust + TypeScript | **License**: MIT
- **What**: Semantic codebase indexing powered by Rust + tree-sitter. Hybrid retrieval (embeddings + BM25 + call graph). Branch-aware incremental indexing. 859 commits.
- **Novel Pattern**: Call graph navigation through `implementation_lookup`, `call_graph`, `call_graph_path`. Low-token discovery via `codebase_context` and `codebase_peek`. Multiple embedding providers (Ollama, OpenAI, Google).
- **NeoTrix Mapping**: NT-ACT code analysis capability. Tree-sitter integration for multi-language parsing. Call graph aligns with module dependency analysis.
- **Cycle Gap Check**: Not in cycles 318-397.

### 9. semantic-code-search
- **URL**: https://github.com/gaurav-oberoi/semantic-code-search
- **Stars**: 0 | **Language**: TypeScript | **License**: MIT
- **What**: Search code by meaning using local embeddings (transformers.js). No API key, no cloud. all-MiniLM-L6-v2 ONNX. Cosine similarity ranking.
- **Novel Pattern**: Fully local semantic search — one-time model download, offline operation. Overlapping line-window chunking with configurable overlap. Deterministic fake embedder for CI.
- **NeoTrix Mapping**: NT-ACT local code understanding. Validates fully-local inference pattern for privacy-sensitive environments.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## T6 — Agent Memory Persistence (NT-MEMORY)

### 10. agent-memory (by OctavianTocan)
- **URL**: https://github.com/OctavianTocan/agent-memory
- **Stars**: 2 | **Language**: Python + Node.js | **License**: MIT
- **What**: Persistent memory for coding agents. SQLite + semantic search + hooks. Cross-agent shared database — Claude Code, Cline, Gemini CLI, Codex, Aider, Cursor, Windsurf all share one DB.
- **Novel Pattern**: Agent-agnostic shared memory layer. Auto-extracts structured facts from conversations. Three layers: facts, soul (preferences), daily_logs. Zero dependencies beyond Python 3.9+ and SQLite.
- **NeoTrix Mapping**: NT-MEMORY cross-session persistence. Shared DB model maps to KB namespace pattern. Preference tracking aligns with SelfModel extension.
- **Cycle Gap Check**: Not in cycles 318-397.

---

## Meta-Analysis

### Theme Consolidation

| Theme | Count | NeoTrix Domain |
|-------|-------|----------------|
| Multi-agent coordination | 2 | NT-CORE, NT-ACT |
| LLM routing / cost | 2 | NT-IO, NT-CORE |
| Semantic memory | 2 | NT-MEMORY |
| Token compression | 1 | NT-CORE |
| Code intelligence | 2 | NT-ACT |
| Agent persistence | 1 | NT-MEMORY |

### Cross-Cutting Pattern: Learned Routing > Static Pipelines
Three independent projects (OMA, AgensFlow, xrouter-llm) converge on the same insight: **runtime-learned routing outperforms static pipeline definitions**. This validates NeoTrix GWT salience-based attention routing as a first-principles design choice, not just an implementation detail.

### Axiom Validation
- **A1 (Cost-Aware Routing)**: xrouter-llm achieves 53.2% cost reduction via prompt-aware routing. Direct empirical evidence.
- **A2 (Context as Scarce Resource)**: Claw Compactor's 14-stage pipeline + Mneme's token-budget-aware context injection both address context compression.
- **A3 (Skill as Production Template)**: ToolRank's tool scoring treats tool definitions as quality-optimizable products.

### Frontier Observation: The Memory wars
Six of 10 trending projects relate to agent memory/recall. The field is converging on three approaches:
1. **Embedding-based** (MOSS rejects this — "opaque by construction")
2. **Structured relational** (MOSS, Mneme entity graph)
3. **Hybrid** (Mneme: vectors + graph + episodes)

NeoTrix VSA HyperCube is uniquely positioned: symbolic + distributed + compositionally productive. No trending project combines all three.

---

*Generated: 2026-09-12 | Cycle: 398 | Agent: opencode/mimo-v2.5-free*
