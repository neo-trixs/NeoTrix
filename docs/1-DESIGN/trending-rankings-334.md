# Trending Rankings — Cycle 334

**Date**: 2026-09-11
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/routing patterns
**Previous cycles**: 318–333 (excluded from this list)

---

## Top 10 New Trending Projects

### 1. Superpowers (obra)
- **URL**: https://github.com/obra/superpowers
- **Stars**: 274,950 | **Forks**: 24,606 | **Language**: Shell
- **What**: Agentic skills framework and software development methodology that works. The dominant "agentic harness" — skills, instincts, memory, security, and research-first development for Claude Code, Codex, Opencode, Cursor and beyond. Defines how agents organize their capabilities, remember across sessions, and enforce quality gates. Growing at 727 stars/day (Sep 11).
- **Key Pattern**: **Skills-as-methodology** — not just a framework but a complete development methodology encoded as agent skills. Memory, security, and quality enforcement baked into the skill layer, not bolted on.
- **NeoTrix Relevance**: Maps to NT-MIND skill crystallization and NT-MEMORY cross-session persistence. Superpowers' "instincts" concept parallels NeoTrix's skill tree with Small Passive / Notable Passive / Keystone tiers. The methodology-as-code approach validates our SEAL pipeline's evolution rules.

### 2. OpenViking (volcengine)
- **URL**: https://github.com/volcengine/OpenViking
- **Stars**: 31,029 | **Forks**: 2,394 | **Language**: Python
- **What**: Self-evolving Context Database for AI Agents. Unifies agent memory, knowledge RAG, and skills into a single system. Memory that grows and adapts autonomously — not static vector stores but evolving context that self-organizes. 1,659 stars this week (Sep 11).
- **Key Pattern**: **Self-evolving context** — memory that autonomously restructures, consolidates, and expands based on usage patterns. Not passive storage but active knowledge management.
- **NeoTrix Relevance**: Maps directly to NT-MEMORY and experience-tree. OpenViking's self-evolving context validates our experience absorption protocol (snapshot→distill→classify→persist→feedback). The unification of memory + RAG + skills in one system mirrors our KB architecture where nodes, edges, embeddings, and FTS coexist.

### 3. deer-flow (bytedance)
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 81,800 | **Forks**: N/A | **Language**: Python
- **What**: Multi-agent deep research framework. Node.js + Python + TypeScript. Agent podcast, multi-agent orchestration, superagent harness. Turns research tasks into coordinated multi-agent workflows with automatic source synthesis. Deep-research as a first-class agent capability.
- **Key Pattern**: **Research-as-workflow** — deep research decomposed into parallel agent tasks with automatic synthesis. Not a single agent doing research, but a coordinated team with specialized roles.
- **NeoTrix Relevance**: Maps to NT-WORLD (deep research perception) and NT-ACT orchestration. deer-flow's multi-agent research decomposition parallels our SEAL pipeline's stage-based processing. The automatic synthesis step validates our KB embedding approach for knowledge consolidation.

### 4. SkillCorpus (EverMind-AI)
- **URL**: https://github.com/EverMind-AI/SkillCorpus
- **Stars**: 646 | **Language**: Python
- **What**: Universal AI skill router — auto-detects best installed skill per prompt + activates "caveman mode" for ~75% token reduction. Works with Claude, Codex, Cursor, OpenCode, Gemini CLI. The routing layer that sits between user intent and skill execution — no manual skill selection needed.
- **Key Pattern**: **Automatic skill routing** — prompt→skill matching without user awareness. Caveman mode as aggressive context compression that preserves intent while slashing token cost.
- **NeoTrix Relevance**: Maps to NT-CORE GWT attention routing and NT-MEMORY context optimization. SkillCorpus validates our Axiom A1 (cost-aware routing) — not all tasks need full skill activation. The 75% token reduction via "caveman mode" is a concrete implementation of Axiom A2 (context as scarce resource).

### 5. ccRecall
- **URL**: https://github.com/tznthou/ccRecall
- **Stars**: Trending Sep 2026 | **Language**: TypeScript/SQLite
- **What**: Cross-session memory for Claude Code — serves relevant context via hooks & MCP. Local SQLite + FTS5, no vector DB. Rule-based retrieval; extraction runs a small model. Zero network dependency. The minimal viable persistent memory: no embeddings, no vector stores, just FTS5 and rules.
- **Key Pattern**: **Minimal persistent memory** — SQLite + FTS5 + rules, no vector DB. Proves that for many use cases, structured text search outperforms embedding-based retrieval at a fraction of the infrastructure cost.
- **NeoTrix Relevance**: Maps to NT-MEMORY lightweight retrieval. ccRecall validates our FTS5-based search in KB. The "no vector DB" approach is a useful counterpoint to our VSA HyperCube — some memory operations don't need high-dimensional embeddings. Rule-based retrieval parallels our experience-tree route table matching.

### 6. OpenSkills (OpenAI)
- **URL**: https://github.com/openai/skills
- **Stars**: 26,000 | **Language**: Python
- **What**: Skills Catalog for Codex. OpenAI's official skill definition format and catalog. Defines how skills are structured, discovered, and composed. The "npm registry" for AI agent skills — standardized skill interfaces with versioning and dependency management.
- **Key Pattern**: **Skill as standardized package** — skills with defined interfaces, versioning, discovery. Not ad-hoc prompts but structured, composable units with contracts.
- **NeoTrix Relevance**: Maps to NT-MIND skill crystallization and NT-ACT tool integration. OpenSkills validates our SKILL-SPEC.md contract approach (<200 lines per skill). The standardized format aligns with our domain model (NT-* domains as skill namespaces). Versioning parallels our constellation maturity (C0-C6).

### 7. cactus-compute/needle
- **URL**: https://github.com/cactus-compute/needle
- **Stars**: 7,986 | **Forks**: 516 | **Language**: Python
- **What**: 14MB foundation model for tiny devices — phones, wearables, smart home, and robots. Runs on the smallest compute targets. 3,838 stars this week (Sep 11). The "Colibri" pattern applied to smaller models — not frontier-scale on consumer hardware, but useful models on edge hardware.
- **Key Pattern**: **Edge-native AI** — not local-first but edge-first. 14MB fits in IoT device memory. Models designed for constrained environments from the ground up, not distilled after the fact.
- **NeoTrix Relevance**: Maps to NT-PHYSICAL (embodiment on constrained hardware) and NT-IO (model access). Needle validates our physical embodiment domain — sensors and motors need local inference. The 14MB constraint informs our resource budget management for edge deployment.

### 8. semantica-agi/semantica
- **URL**: https://github.com/semantica-agi/semantica
- **Stars**: 9,709 | **Forks**: 1,029 | **Language**: Python
- **What**: Graph-native infrastructure for context and accountable AI systems. 4,005 stars this week (Sep 11). Context as a graph, not a list. Accountability baked into the graph structure — every decision traceable to its source nodes. The "knowledge graph as infrastructure" pattern.
- **Key Pattern**: **Graph-native accountability** — context represented as a graph where every node has provenance. Not just "what does the agent know" but "where did each piece of knowledge come from."
- **NeoTrix Relevance**: Maps to NT-MEMORY (KB graph structure) and NT-GOVERNANCE (accountability). Semantica's graph-native approach validates our KB architecture (nodes + edges + embeddings). Accountable context directly maps to our rev-officer evidence-first methodology — every finding traced to file:line.

### 9. Kopai
- **URL**: https://kopai.cloud/
- **Stars**: N/A (Product Hunt: 84 upvotes, Sep 8)
- **What**: The Cloud for AI Agents. Build the agent; we run it. Publish any agent as an API (native or OpenAI-compatible, with streaming), list it in the marketplace, or call it from your own product. Analytics count what an agent costs to run separately from what it earns. One-command benchmark runs against reference agents before shipping. Certification expires, and answers get re-checked.
- **Key Pattern**: **Agent-as-a-service with accountability** — not just hosting agents but benchmarking, certifying, and monitoring them. Cost tracking separated from revenue tracking. Certification with expiry — agents must maintain quality to stay listed.
- **NeoTrix Relevance**: Maps to NT-IO (agent deployment) and NT-GOVERNANCE (certification). Kopai's benchmark-before-ship parallels our SelfTest tiers (T1/T2/T3). Cost/revenue separation validates our cost-aware routing (Axiom A1). Expiring certification maps to constellation maturity re-evaluation.

### 10. NVIDIA Switchyard
- **URL**: https://github.com/NVIDIA-NeMo/Switchyard
- **Stars**: 1,928 | **Forks**: 177 | **Language**: Rust
- **What**: LLM traffic router across models and providers while preserving native OpenAI and Anthropic API compatibility. Enables flexible model selection, benchmarking, and cost/performance optimization. 1,220 stars this week (Sep 11). Rust-native for performance. The "smart proxy" for LLM deployments.
- **Key Pattern**: **API-compatible model routing** — route across providers without changing client code. Benchmarking built into the routing layer — performance data feeds routing decisions.
- **NeoTrix Relevance**: Maps to NT-IO provider routing and NT-CORE cost-aware routing. Switchyard validates our ordered backend fallback pattern (P4). Rust-native aligns with our core implementation language. Benchmark-integrated routing directly implements Axiom A1 — routing decisions based on measured performance, not configuration.

---

## Key Trends (Cycle 334)

1. **Self-Evolving Memory Dominates**: OpenViking (31K stars), ccRecall, and SkillCorpus all implement memory that evolves autonomously. The static vector store era ends — memory systems must self-organize, consolidate, and adapt. OpenViking's "unify memory + RAG + skills" validates NeoTrix's integrated KB approach.

2. **Skills as Standardized Infrastructure**: OpenSkills (OpenAI, 26K stars), Superpowers (275K stars), and SkillCorpus converge on skills as structured, versionable, composable packages. The "skill" becomes the unit of AI capability, not the "model" or the "prompt." This validates NeoTrix's SKILL-SPEC.md contract and domain-namespace mapping.

3. **Token Reduction as First-Class Feature**: SkillCorpus (75% via caveman mode), ccRecall (minimal memory without vector overhead), and Switchyard (cost/performance routing) all prioritize token efficiency. The production constraint is not capability but cost — every system must justify its token spend.

4. **Graph-Native Context**: Semantica and OpenViking both represent context as graphs, not lists. The graph structure provides provenance, accountability, and relational reasoning that flat retrieval cannot. This validates NeoTrix's KB graph architecture (nodes + edges).

5. **Edge-Native AI Arrives**: Needle (14MB model for IoT) extends the "local AI" trend from consumer hardware to truly constrained devices. The Colibri pattern (expert streaming) plus Needle (edge-native) covers the full spectrum from frontier-on-laptop to useful-on-sensor.

6. **Research-as-Workflow**: deer-flow (81K stars) treats deep research as a coordinated multi-agent workflow, not a single agent prompt. The decomposition + synthesis pattern is the production form of "agentic research."

---

## Action Items for NeoTrix

| Project | NeoTrix Integration Opportunity | Domain | Priority |
|---------|--------------------------------|--------|----------|
| OpenViking | Self-evolving context as model for experience-tree evolution | NT-MEMORY | P0 |
| Superpowers | Skills-as-methodology pattern for NT-MIND crystallization | NT-MIND | P0 |
| SkillCorpus | Auto-skill routing + caveman mode for GWT cost optimization | NT-CORE | P1 |
| OpenSkills | Standardized skill format for SKILL-SPEC.md alignment | NT-MIND | P1 |
| Semantica | Graph-native accountability for KB provenance | NT-MEMORY | P1 |
| ccRecall | Minimal memory pattern (SQLite+FTS5) for lightweight retrieval | NT-MEMORY | P2 |
| deer-flow | Multi-agent research decomposition for NT-WORLD deep research | NT-WORLD | P2 |
| Switchyard | Rust-native model routing with benchmark integration | NT-IO | P2 |
| Kopai | Agent certification with expiry for quality maintenance | NT-GOVERNANCE | P2 |
| Needle | Edge-native inference for NT-PHYSICAL constrained deployment | NT-PHYSICAL | P3 |
