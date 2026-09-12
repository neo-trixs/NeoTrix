# Trending Rankings — Cycle 420 (2026-09-12)

## 10 New Projects (Not in Cycles 318-419)

| # | Project | Stars | Focus | NeoTrix Relevance |
|---|---------|-------|-------|-------------------|
| 1 | **ToolRank** | ~1.2K | AI agent tool discovery & scoring — findability, clarity, precision, efficiency across 4 dimensions. LLM selection tournaments + runtime reliability testing. | NT-ACT: tool registration scoring, CapabilityRegistry health metrics. Complements ToolRegistry self-test. |
| 2 | **claw-compactor** | ~3.5K | 14-stage Fusion Pipeline for LLM token compression — reversible, AST-aware, zero LLM inference cost. | NT-MEMORY: context compression for >256K sessions. Aligns with KVMem paged KV — compression as pre-filter before paging. |
| 3 | **Flare** | ~2.1K | Graph-first IDE and interactive map for agentic coding. Visual dependency graph as primary navigation. | NT-CORE: E8 reasoning engine visualized. Maps to CapabilityTree→CapabilityRegistry bridge — spatial reasoning over code. |
| 4 | **AgentInfer** (Paper + Code) | ~1.8K | Unified framework: AgentCollab (dual-model reasoning) + AgentSched (cache-aware) + AgentSAM (speculative decoding via semantic memory) + AgentCompress (async distillation). | NT-MIND: SEAL pipeline optimization — AgentSAM maps to experience compression, AgentCompress to distillation stage. |
| 5 | **verl-agent** | ~2.3K | RL training framework for LLM/VLM agents. Group-in-Group Policy Optimization (GiGPO). | NT-MIND: SEAL pipeline could adopt GiGPO for self-evolution reward shaping. |
| 6 | **Dropstone** | ~4.5K | "AI runtime that remembers, learns, and acts everywhere." Persistent cross-session memory + action layer. | NT-MEMORY + NT-NEXUS: cross-session memory weaving, experience persistence. |
| 7 | **HarnessRouter** | ~1.8K | Open-source unified interface for agent harnesses — orchestrates multiple agent runtimes. | NT-IO: multi-provider routing, aligns with Ordered Backend Router pattern (P4). |
| 8 | **Gated-Memory Routing** (EMNLP 2026) | ~900 | Learned Memory Write Gate + Retrieval Gate + Adaptive Halting. Compact state for multi-agent reasoning. | NT-MEMORY: gated memory architecture maps to KB write filtering + experience-tree lazy loading. |
| 9 | **R2-Router** | ~1.5K | "Routing as Reasoning" — treats each LLM as a quality-cost curve (not a point). Budget-constrained output length. | NT-IO: cost-aware routing (Axiom A1). Extends GWT salience with token budget optimization. |
| 10 | **Construct Computer** | ~3.2K | "Your AI coworker gets a computer." Full computer-use agent with desktop environment. | NT-ACT + NT-PHYSICAL: agent embodiment in desktop space, aligns with physical module motor control. |

## Category Breakdown

### AI Agents & Orchestration
- **ToolRank** — tool discovery scoring
- **Construct Computer** — desktop computer-use
- **HarnessRouter** — multi-harness orchestration
- **Dropstone** — persistent memory agent

### LLM Inference & Efficiency
- **claw-compactor** — token compression (14-stage)
- **AgentInfer** — co-designed inference architecture
- **PackInfer** — batched attention kernel (from arxiv)
- **R2-Router** — reasoning-based routing

### Reasoning & Memory
- **Gated-Memory Routing** — compact multi-agent state
- **Flare** — graph-first IDE for agentic coding

### Training & Evolution
- **verl-agent** — RL agent training (GiGPO)

## Top 3 Actionable Insights for NeoTrix

1. **Token compression as pre-filter** (claw-compactor) — Before KVMem paging, apply 14-stage reversible compression. Zero inference cost, 50%+ token reduction.

2. **Routing as reasoning** (R2-Router) — Extend GWT salience from "which model" to "which model + what budget." Each LLM modeled as quality-cost curve, not static point.

3. **Gated memory write** (Gated-Memory Routing) — Memory Write Gate for KB commits: only non-redundant reasoning steps persist. Retrieval Gate for experience-tree lazy loading.

## Sources

- GitHub Topics: llm-tools, llm-agents, agents (2026-09-12)
- Product Hunt Q2 2026 State of Tech Discovery
- arXiv: 2605.18597, 2512.18337, 2602.06072, 2609.00237, 2602.02823
- GitTrend AI Agent rankings
- AI Hunt weekly best (2026-09-07 to 2026-09-13)
