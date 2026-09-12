# Trending Rankings — Cycle 390 (2026-09-12)

## 10 New Projects (Not in Cycles 318–389)

### 1. ADHD — Parallel Divergent Ideation Skill for Coding Agents
- **URL**: https://github.com/UditAkhourii/adhd
- **Stars**: 4,059 | **Forks**: 278
- **What**: Tree-of-thought with pruning for coding agents. Spawns N isolated reasoning processes under deliberately distorted cognitive frames with zero shared context during divergence, then runs a separate critic pass to score, cluster, prune traps, and deepen survivors. Solves "give me a few ways to…" problems better than linear CoT.
- **Novel Pattern**: Generator-critic separation with cognitive-frame branching. 6 isolated frames → 30+ ideas → 20 traps flagged → non-obvious picks. The New Stack featured it; 50+ agent integrations.
- **NeoTrix Mapping**: NT-CORE (parallel reasoning under GWT), NT-MIND (divergent→convergent evolution), ConsciousnessTree (cognitive frame as awareness modulation).

### 2. Sem — Semantic Version Control for Coding Agents
- **URL**: https://github.com/Ataraxy-Labs/sem
- **Stars**: 3,328 | **Forks**: 101
- **What**: Entity-level diffs, blame, and impact analysis on top of git. Parses code with tree-sitter (28 languages), extracts every function/class/method as an entity, diffs at entity level instead of lines. Cross-file dependency graph shows blast radius. JSON output for AI agents.
- **Novel Pattern**: Structural hashing + rename detection + word-level inline highlights at entity granularity. `sem setup` replaces `git diff` globally. Pre-commit hook shows entity-level blast radius of staged changes.
- **NeoTrix Mapping**: NT-WORLD (semantic code perception), NT-ACT (agent-native version control), NT-MEMORY (entity-level knowledge graph).

### 3. Claw Compactor — 14-Stage Fusion Pipeline for Token Compression
- **URL**: https://github.com/open-compress/claw-compactor
- **Stars**: N/A (PyPI, new) | **Language**: Python
- **What**: 14-stage Fusion Pipeline for LLM token compression — 15-82% reduction depending on content. Zero LLM inference cost. Reversible compression with AST-aware code analysis, JSON statistical sampling, simhash deduplication. 1600+ tests. ROUGE-L @0.3 = 0.653 vs LLMLingua-2's 0.346 (+88.2%).
- **Novel Pattern**: Content-type-aware compression stages (QuantumLock → Cortex → Photon → RLE → SemanticDedup → Ionizer → Neurosyntax → Nexus → TokenOpt → Abbrev). Rewind markers allow LLM to retrieve originals by marker ID.
- **NeoTrix Mapping**: NT-IO (context compression at pipeline boundaries), NT-MEMORY (RAG pre-processing), GWT (salience-aware token budgeting), Axiom A2 (Context as Scarce Resource).

### 4. ToolRank — Agent Tool Optimization (ATO) Platform
- **URL**: https://toolrank.dev
- **Stars**: N/A (new platform) | **Language**: TypeScript/Python
- **What**: The "PageRank for AI agent tools." Scores tool definitions across 4 dimensions (Findability 25%, Clarity 35%, Precision 25%, Efficiency 15%). 97.1% of MCP tools have quality defects. Optimized tools get selected 3.6x more often. Scans 4,000+ MCP servers daily. ATO = SEO → LLMO → ATO progression for the agent economy.
- **Novel Pattern**: LLM selection tournaments + runtime reliability testing + rule-based scoring. Agent framework SDK for runtime tool evaluation.
- **NeoTrix Mapping**: NT-ACT (tool discovery optimization), NT-IO (MCP interface quality), NT-SHIELD (tool trust scoring), GWT (salience-based tool selection).

### 5. Nex — Claude Cowork for High-Volume GTM Workflows
- **URL**: https://nex.ai (YC S26)
- **Stars**: N/A (YC-backed) | **Team**: ex-HubSpot
- **What**: AI that builds and runs complex, high-volume GTM workflows across 1000+ integrations. Agent writes code for deterministic parts (fast, cheap) and reserves AI judgment for hard parts. CRM cleanup, lead scoring, re-engagement, outbound — all from goal description.
- **Novel Pattern**: "AI writes code for predictable parts" — hybrid deterministic+LLM execution. Knowledge graph synthesizes signals across fragmented tools (HubSpot, Clay, Outreach, Gong, Slack).
- **NeoTrix Mapping**: NT-ACT (workflow orchestration), NT-WORLD (multi-tool signal fusion), NT-MEMORY (knowledge graph), NT-IO (1000+ integration interface).

### 6. KVMem — Virtualizing Million-Token Agent Workspaces on Consumer GPU
- **URL**: arXiv:2609.04852 (Sep 2026)
- **Stars**: N/A (paper + open-source engine)
- **What**: KV-context virtualization system preserving overflowed workspace history as paged KV state across GPU→Host→NVMe. 24GB RTX 5090 laptop runs Qwen3.8-27B at ~50 tokens/s with 1M-token workspace (4× native 256K window). Attention-space indexes (Mean-K vectors) select relevant historical blocks per agent step.
- **Novel Pattern**: GPU→Host→NVMe tiered KV paging with query-conditioned retrieval. Working set decomposed into Retained/Incoming/Outgoing blocks. GPU pages of retained blocks reused directly across steps.
- **NeoTrix Mapping**: NT-MEMORY (paged KV as memory virtualization), NT-IO (inference optimization), NT-CORE (GWT salience for block selection), Axiom A2 (Context as Scarce Resource — breaks memory wall).

### 7. Attention-MoA — Inter-Agent Semantic Attention for Mixture-of-Agents
- **URL**: arXiv:2601.16596
- **Stars**: N/A (paper)
- **What**: Redefines MoA collaboration through Inter-agent Semantic Attention — deep semantic interaction between agents, not just output aggregation. Inter-layer Residual Module with Adaptive Early Stopping prevents information degradation. Small open-source models ensemble outperforms Claude-4.5-Sonnet and GPT-4.1 (MT-Bench 8.83, AlpacaEval 2.0 LC Win Rate 77.36%).
- **Novel Pattern**: Agents attend to each other's semantic representations (not just outputs). Residual connections prevent degradation in deep agent layers. Adaptive early stopping for efficiency.
- **NeoTrix Mapping**: NT-CORE (agent attention routing = GWT refinement), NT-ACT (multi-agent coordination), ConsciousnessTree (inter-agent semantic awareness).

### 8. Explicit Trait Inference (ETI) — Multi-Agent Coordination via Trait Tracking
- **URL**: arXiv:2604.19278 (ACL 2026)
- **Stars**: N/A (ACL 2026 accepted)
- **What**: Psychologically grounded method for multi-agent coordination. Agents infer and track partner characteristics along warmth (trust) and competence (skill) dimensions from interaction histories. Reduces payoff loss 45-77% in economic games, improves performance 3-29% on MultiAgentBench.
- **Novel Pattern**: Lightweight trait inference (warmth + competence) from interaction history guides agent decisions. First systematic evidence LLM agents can reliably infer others' traits and leverage structured awareness for coordination.
- **NeoTrix Mapping**: NT-CORE (agent self-model tracking), NT-ACT (multi-agent trust/competence routing), NT-FEEL (warmth dimension), ConsciousnessTree (meta-cognitive awareness of partner traits).

### 9. Airtop — Compiled Agent Builder with Self-Healing
- **URL**: https://www.airtop.ai
- **Stars**: N/A (SOC 2 Type II certified)
- **What**: Agent Builder compiles automations into reusable code — up to 6x faster, 1% the cost at scale vs LLM-per-step agents. Built-in self-healing: broken runs diagnose failure, update draft, test fix. Cloud execution with full traces + video recordings. Browser automation for any website (APIs + logins + legacy portals).
- **Novel Pattern**: "Build once, run like software" — compile natural language → deterministic code → test → deploy. Self-healing on failure. 10-100x cheaper than LLM agents at scale.
- **NeoTrix Mapping**: NT-ACT (skill crystallization via compilation), NT-SHIELD (SOC 2 compliance, audit trails), NT-IO (browser automation interface), SEAL (distillation stage — NL→compiled skill).

### 10. Nex Agent Builder — Agent Skill Marketplace for Coding Agents
- **URL**: https://www.airtop.ai/connect/claude-code (skill: `airtop-ai/airtop-skill`)
- **Stars**: N/A (ProductHunt #1 Sep 4, 2026)
- **What**: One skill to run and manage agents that automate GTM workflows from terminal. Agent Builder + Mark (vibe automation for marketers). Integrates with Claude Code, Codex, Cursor, 40+ coding agents. SOC 2 + HIPAA compliant.
- **Novel Pattern**: Skill-as-MCP-server pattern — one `npx skills add` installs browser automation into any coding agent. Compiled agents run like software, not LLM per-step.
- **NeoTrix Mapping**: NT-ACT (skill distribution via MCP), NT-IO (multi-agent connector), NT-SHIELD (compliance), NT-WORLD (web automation).

---

## Meta-Trends (Cycle 390)

| Trend | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Parallel Divergent Reasoning** | ADHD, Attention-MoA | Isolated cognitive frames + semantic attention → GWT multi-path salience |
| **Entity-Level Code Understanding** | Sem, Claw Compactor | Structural hashing + AST-aware compression → NT-WORLD code perception |
| **Token Compression at Scale** | Claw Compactor, KVMem | 15-82% compression + 1M-token virtualization → Axiom A2 fulfillment |
| **Agent Tool Optimization** | ToolRank, Airtop | ATO as new optimization surface → NT-ACT tool discovery |
| **Compiled Agent Execution** | Airtop, Nex | NL→compiled code + self-healing → SEAL skill crystallization |
| **Paged KV Virtualization** | KVMem | GPU→Host→NVMe tiered KV → NT-MEMORY memory virtualization |
| **Trait-Based Agent Coordination** | ETI, Attention-MoA | Warmth/competence tracking + semantic attention → NT-CORE agent self-model |
| **Deterministic+LLM Hybrid** | Nex, Airtop | Code for predictable parts, LLM for hard parts → GWT cost-aware routing |
