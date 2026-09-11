# Trending Rankings — Cycle 330

**Date**: 2026-09-11
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/routing patterns
**Previous cycles**: 318–329 (excluded from this list)

---

## Top 10 New Trending Projects

### 1. Neo4j Agent Memory
- **URL**: https://github.com/neo4j-labs/agent-memory
- **Stars**: 519 | **Forks**: 98
- **What**: Graph-native memory system for AI agents backed by Neo4j. Three connected memory layers (short-term conversations, long-term entities/facts, reasoning traces) in one knowledge graph. Multi-stage entity extraction pipeline (spaCy → GLiNER2 → GLiREL → LLM). POLE+O entity classification. Hosted NAMS service or self-hosted bolt. Python + TypeScript SDKs. 16 MCP tools. Integrations with LangChain, PydanticAI, Google ADK, Strands, CrewAI, LlamaIndex.
- **Key Pattern**: **Three-layer graph-native memory** — conversations, entities, and reasoning traces stored as connected nodes in a single knowledge graph. Not vector chunks, not rows — graph traversal enables multi-hop reasoning across memory types. Background compression worker (messages → observations → active reflection).
- **NeoTrix Relevance**: Maps directly to NT-MEMORY KB architecture. The three-layer model (short-term/long-term/realing) parallels our experience-tree hub/branch/full-text structure. POLE+O entity classification could enrich KB node typing. The `:TOUCHED` audit edges from reasoning steps to entities align with our pointer-conservation philosophy.

### 2. Offsite — Hybrid Human-Agent Teams
- **URL**: https://www.producthunt.com/products/offsite-2
- **What**: Shared workspace where humans and AI agents occupy roles in a live org chart. Agents appear as nodes alongside humans, talk to each other via dragged edges, and collaborate in real time. Human-in-the-loop by default — no real-world actions without approval. Integrates Claude Code, OpenClaw, HeyGen, any MCP agent. 800+ tools. Mercury engine for graph-based agent coordination (not just chains). Always-on persistent teams, not run-once pipelines.
- **Key Pattern**: **Org chart as orchestration interface** — the organizational hierarchy is the unit of multi-agent coordination, not code or prompts. String-in/string-out conversations make agent interactions human-understandable. "Every node interchangeable between human and agent."
- **NeoTrix Relevance**: Maps to NT-ACT production deployment and NT-GOVERNANCE. The org-chart-as-interface pattern is relevant to our DomainBridge cross-domain coordination. Human-in-the-loop approval gates align with NT-SHIELD safety kernel. The "protocol that lets siloed systems talk" (not building own agents) validates our capability-network architecture.

### 3. Cowork — Claude as Digital Coworker
- **URL**: https://www.producthunt.com/products/cowork (Sep 2026)
- **What**: Turns Claude into a persistent digital coworker. Not a chatbot — a teammate with persistent context, memory, and the ability to initiate work. Operates across Slack, email, calendar, code repos. Proactive task execution based on learned patterns. Shared context with human team members.
- **Key Pattern**: **Agent-as-coworker (not agent-as-tool)** — persistent identity, proactive initiation, shared context. The agent has "job knowledge" that accumulates across sessions. The shift from reactive (you prompt it) to proactive (it notices work to do).
- **NeoTrix Relevance**: Maps to NT-MIND self-evolution (proactive capability) and NT-IO interface layer. The proactive initiation pattern aligns with ConsciousnessTree's self-directed growth cycles. Persistent context across tools validates NT-MEMORY cross-session knowledge.

### 4. AgentOpera — Graph-Based Multi-Agent Orchestration
- **URL**: https://docs.agentopera.ai/
- **What**: Open-source graph-based framework for multi-agent teams. Smart Router analyzes user intents and directs to domain-specific agents. Swarm intelligence via unified routing across devices, edge nodes, and cloud. Supports MCP-compatible agents, zero-code tools, hybrid cloud-edge execution. A2A (Agent-to-Agent) protocol for standardized inter-agent communication.
- **Key Pattern**: **Graph-native multi-agent routing** — user intent → intent classification → graph-based agent dispatch. Not chain-based but network-based. Hybrid edge-cloud execution with unified routing.
- **NeoTrix Relevance**: Maps to NT-ACT CapabilityRouter and NT-WORLD perception routing. The graph-native dispatch pattern is relevant to our GWT attention routing. Edge-cloud hybrid execution aligns with NT-PHYSICAL embodied architecture.

### 5. Cortex — Neuroscience-Inspired Persistent Memory
- **URL**: https://github.com/cdeust/Cortex
- **What**: Persistent memory for Claude Code built on computational neuroscience. 36 cited brain mechanisms for encoding → consolidation → retrieval → forgetting. 52 memory tools, 9 lifecycle hooks. BEAM benchmark validated. Hippocampal Replay for context restoration after compaction. Five-signal fusion (vector + FTS + trigram + thermodynamic heat + recency) with cross-encoder reranking. Stage-Aware Context Assembly. Self-curating per-project wiki. "Says I don't know when unsure, flags its own contradictions."
- **Key Pattern**: **Neuroscience-grounded memory lifecycle** — not just storage/retrieval but full cognitive cycle (encode → consolidate → retrieve → forget). Hippocampal Replay solves context loss during compaction. Contradiction detection as first-class memory operation.
- **NeoTrix Relevance**: Maps to NT-MEMORY experience-tree lifecycle and NT-META self-audit. The 36 neuroscience mechanisms provide a validated taxonomy for our memory operations. Hippocampal Replay directly addresses our session-start context reconstruction. Contradiction flagging aligns with ROAM relation classification from cycle 329.

### 6. mnem — Git for AI Agent Knowledge
- **URL**: https://github.com/Uranid/mnem
- **What**: Persistent, versioned knowledge layer for AI agents. Skills, decisions, conventions as nodes and typed edges in a queryable knowledge graph inside `.mnem/` directory. Commit alongside code. Branch, diff, merge, roll back any write. Content-addressed storage (same bytes = same ID, auto-dedup). Hybrid retrieval (vector + keyword + graph) in single pass. Token-budget transparency (reports exactly what was found, skipped, and token cost). Forgetting as first-class: revoke a fact and every retrieval path filters it out. Single binary, no server, offline, WASM/edge. MCP server.
- **Key Pattern**: **Git semantics for agent knowledge** — version control, branching, merging, content-addressed storage applied to memory. "Commit it alongside your code and every teammate's agents start from the same baseline." Forgetting with audit trail preserved.
- **NeoTrix Relevance**: Maps to NT-MEMORY experience-tree versioning and pointer conservation. The git-as-memory-primitive pattern validates our AGENTS.md pointer-conservation philosophy. Content-addressed storage could optimize KB deduplication. Token-budget transparency aligns with our experience-tree lazy loading.

### 7. PlugMem — Plug-and-Play Long-Term Memory
- **URL**: https://github.com/taodeng009/PlugMem
- **What**: ICML 2026 accepted. Task-agnostic long-term memory system. Organizes experience into compact, reusable knowledge units (not raw histories). Three memory types: Semantic (facts/concepts), Procedural (workflows), Episodic (interaction sequences). Graph structure with hierarchical knowledge units. LLM-enhanced extraction, retrieval, and reasoning. Memory compression and evolution. Plugin release for OpenClaw and Claude Code. SOTA on LongMemEval (90.2 Acc) and HotpotQA (79.1 F1).
- **Key Pattern**: **Experience as reusable knowledge units** — not storing raw interactions but distilling them into compact, task-agnostic knowledge atoms. Three-type memory taxonomy (semantic/procedural/episodic) with graph structure connecting them.
- **NeoTrix Relevance**: Maps to NT-MEMORY experience-tree distillation. The three-type taxonomy (semantic/procedural/episodic) could enrich our experience namespace typing. ICML validation confirms knowledge-unit approach is sound. Plugin architecture validates SKILL-SPEC.md portability.

### 8. Parason — Parallel Reasoning with Trial Parallelism
- **URL**: https://arxiv.org/pdf/2608.24658
- **What**: Framework revealing two forms of parallel reasoning: Subtask Parallelism (independent decomposition) and Trial Parallelism (competing speculative attempts). Trial Parallelism accounts for 65.5% of parallelizable steps in DeepSeek-V4. Converts sequential traces to structured parallel trajectories via context-free grammar. Trained with PA-GRPO (Parallelism-Aware GRPO) balancing accuracy, latency, and parallelism ratios. ~1.7x acceleration while maintaining accuracy on AIME24/AIME25.
- **Key Pattern**: **Trial Parallelism as dominant mechanism** — hard reasoning is not just decomposition but trying and refining uncertain solution paths. CFG-based structure enables engine-parseable parallel dispatch.
- **NeoTrix Relevance**: Maps to NT-CORE E8 reasoning (trial branches as exploration paths) and NT-ACT parallel tool execution. The CFG structure for parallel reasoning is relevant to SEAL pipeline parallel stage execution. Trial Parallelism as dominant pattern validates our E8 hexagram exploration of multiple reasoning paths.

### 9. Livedocs — The General Data Agent
- **URL**: https://www.producthunt.com/products (Sep 2026)
- **What**: ProductHunt Sep 2026 featured. "The general data agent." Connects to databases, spreadsheets, APIs, and documents. Automatically discovers data schemas, understands relationships, and answers complex analytical questions. Not a BI tool — an agent that reasons over your data. Multi-source data fusion with natural language interface.
- **Key Pattern**: **Schema-aware data reasoning** — agent discovers structure, then reasons over it. Multi-source fusion as agent capability, not separate ETL pipeline. Natural language as the interface to structured data.
- **NeoTrix Relevance**: Maps to NT-WORLD data perception and NT-MEMORY KB queries. Schema-aware discovery aligns with our Knowledge Representation (VSA HyperCube) for structured data. Multi-source fusion parallels our UnifiedCrawler multi-source aggregation.

### 10. Basedash — AI Data Analyst
- **URL**: https://www.producthunt.com/products (Sep 2026)
- **What**: AI-powered data analyst that queries your database with natural language. Traceable, action-ready business intelligence. Pairs natural-language querying with governed analytics. Not just generating SQL — understanding business context, suggesting analyses, and producing actionable insights. Query results include source provenance and confidence scores.
- **Key Pattern**: **Governed analytics with provenance** — AI-generated insights with traceable source lineage and confidence scores. Business context understanding, not just SQL generation.
- **NeoTrix Relevance**: Maps to NT-MEMORY query quality and NT-GOVERNANCE audit. Provenance tracking aligns with our KB query audit trail. Confidence scoring is relevant to experience-tree quality gates (SRMA pattern from cycle 329).

---

## Key Trends (Cycle 330)

1. **Memory Is Becoming Graph-Native**: Neo4j Agent Memory, Cortex, mnem, PlugMem all move from vector-store memory to graph-structured knowledge. The graph enables multi-hop reasoning, version control, and contradiction detection that flat stores cannot.

2. **Agent-as-Coworker, Not Agent-as-Tool**: Offsite (org chart), Cowork (persistent teammate), and Basedash (analyst) all treat agents as team members with roles and persistent context, not stateless tools you prompt.

3. **Neuroscience Grounding for Memory**: Cortex (36 brain mechanisms), PlugMem (ICML 2026), and mnem (content-addressed) all ground memory systems in cognitive science rather than ad-hoc storage.

4. **Git Semantics for Agent Knowledge**: mnem and GitAgent (cycle 329) both use git primitives for agent memory — versioning, branching, merging, content addressing. Knowledge becomes committable alongside code.

5. **Parallel Reasoning Dominance**: Parason shows Trial Parallelism (speculative attempts) is 65.5% of parallelizable reasoning. Hard reasoning ≠ decomposition; it = exploration of uncertain paths.

6. **Governed Analytics**: Basedash and Livedocs both add provenance and confidence to AI-generated data insights. Trust requires traceability, not just accuracy.

---

## Action Items for NeoTrix

| Project | NeoTrix Integration Opportunity | Domain | Priority |
|---------|--------------------------------|--------|----------|
| Neo4j Agent Memory | Study POLE+O entity model for KB node typing | NT-MEMORY | P1 |
| Cortex | Adopt Hippocampal Replay for session-start context reconstruction | NT-MEMORY | P1 |
| mnem | Content-addressed storage + version control for experience-tree | NT-MEMORY | P1 |
| PlugMem | Three-type memory taxonomy (semantic/procedural/episodic) for KB | NT-MEMORY | P1 |
| Parason | Trial Parallelism pattern for E8 hexagram exploration | NT-CORE | P2 |
| Offsite | Org-chart-as-orchestration for DomainBridge | NT-ACT | P2 |
| Cowork | Proactive initiation pattern for ConsciousnessTree | NT-MIND | P2 |
| AgentOpera | Graph-native agent dispatch for CapabilityRouter | NT-ACT | P2 |
| Livedocs | Schema-aware data reasoning for VSA HyperCube | NT-WORLD | P3 |
| Basedash | Provenance + confidence scoring for KB queries | NT-MEMORY | P3 |
