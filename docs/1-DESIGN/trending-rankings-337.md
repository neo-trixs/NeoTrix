# Trending Rankings — Cycle 337

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt, ossinsight.io, arXiv

## 10 New Projects (Not in Cycles 318-336)

### 1. OpenViking (ByteDance)
- **URL**: https://github.com/volcengine/OpenViking
- **Stars**: 36K+ | **License**: AGPLv3
- **Category**: Context Database for AI Agents
- **Pattern**: File-system paradigm for agent memory — `viking://` URIs, L0/L1/L2 tiered loading, directory recursive retrieval. Sessions auto-commit to long-term memory. 80-83% accuracy on LoCoMo benchmarks (vs 33-57% native).
- **NeoTrix Mapping**: NT-MEMORY — aligns with `experience-tree` KB hub pattern. Tiered loading (L0/L1/L2) is structurally identical to NeoTrix's lazy branch loading. **Absorption candidate**: filesystem-as-context-interface paradigm.

### 2. Superpowers (obra)
- **URL**: https://github.com/obra/superpowers
- **Stars**: 282K+ | **License**: MIT
- **Category**: Agentic Skills Framework & Development Methodology
- **Pattern**: Composable skills library (brainstorming → writing-plans → subagent-driven-development → TDD → code-review → branch-finish). Mandatory workflows, not suggestions. Subagents dispatched per task with two-stage review. Cross-session worktree isolation.
- **NeoTrix Mapping**: NT-ACT — mirrors NeoTrix skill node architecture (Small Passive / Notable Passive / Keystone tiers). Two-stage review maps to NT-SHIELD audit. **Absorption candidate**: mandatory-workflow-skills pattern — skills that auto-trigger based on context, not user invocation.

### 3. GAAI Framework (digipulse-engineering)
- **URL**: https://github.com/Fr-e-d/GAAI-framework
- **Stars**: 157 | **Category**: Governed Agentic AI Infrastructure
- **Pattern**: `.gaai/` folder governance layer for AI coding tools. Discovery (what to build) → Delivery (autonomous execution until criteria pass). Cross-session memory, scope authorization, QA gates. Daemon mode for parallel delivery.
- **NeoTrix Mapping**: NT-SHIELD + NT-GOVERNANCE — governance layer between task intake and code deployment. Maps to GAIE (Governed AI-Assisted Engineering) framework's three-tier oversight model. **Absorption candidate**: governance-as-folder convention.

### 4. Cortex MCP (SKYNETLAB / cortex-mcp)
- **URL**: https://github.com/saketh12e/cortex-mcp
- **Stars**: 1 (early) | **License**: MIT
- **Category**: Persistent MCP Memory Layer
- **Pattern**: Local-first MCP server — one SQLite file shared across all agents. Origin-tagged memories (who wrote it, from which client). Hybrid search: BM25 + cosine semantic fused with RRF. Deduplication (exact hash + near-duplicate cosine ≥ 0.95). Soft-delete + audit trail.
- **NeoTrix Mapping**: NT-MEMORY — maps to KB pipeline with BM25 index. Origin-tagging maps to experience attribution. **Absorption candidate**: hybrid RRF fusion for memory search, dedup-by-cosine.

### 5. RAGEN-2 / StarPO (mll-lab-nu)
- **URL**: https://github.com/mll-lab-nu/RAGEN
- **Stars**: ~500 | **Category**: Agent RL Training Framework
- **Pattern**: StarPO (State-Thinking-Actions-Reward Policy Optimization) for trajectory-level agent RL. Identifies "template collapse" — reasoning looks diverse but is input-agnostic. Diagnosed via mutual information (MI) proxy. SNR-Aware Filtering selects high-signal prompts per iteration.
- **NeoTrix Mapping**: NT-MIND — maps to SEAL pipeline self-evolution. Template collapse = loss of input-specific reasoning in ConsciousnessTree. MI diagnostic = cross-domain attention quality metric. **Absorption candidate**: MI-based reasoning quality diagnostic for ConsciousnessTree health.

### 6. IQ Routing (ProductHunt)
- **URL**: producthunt.com/products/iq-routing
- **Category**: Trajectory-Aware LLM Routing
- **Pattern**: Routes agent queries to cheapest capable model by analyzing execution trajectories, not just current query. Cuts agent cost significantly.
- **NeoTrix Mapping**: NT-CORE (GWT) — extends cost-aware routing axiom (A1). Trajectory-aware routing = history-conditioned salience. **Absorption candidate**: trajectory-history as routing signal.

### 7. Mozaik
- **URL**: producthunt.com/products/mozaik
- **Category**: TypeScript Runtime for Concurrent AI Agents
- **Pattern**: TypeScript runtime enabling concurrent agent execution. Agents run as isolated execution contexts with shared state management.
- **NeoTrix Mapping**: NT-ACT — maps to production orchestrator (BatchProductionManager). Concurrent agent execution = worktree isolation pattern. **Absorption candidate**: typed-runtime for agent coordination.

### 8. DELTA (Training-Free Sparse Attention)
- **URL**: https://github.com/hoenza/DELTA
- **Category**: Efficient Attention Mechanism
- **Pattern**: Partitions transformer layers into three groups: initial full-attention layers → Δ-layers that identify salient tokens via aggregated head-level attention scores → sparse-attention layers attending only to selected subset. Training-free. 4.25x fewer attended tokens, 1.54x speedup.
- **NeoTrix Mapping**: NT-CORE (GWT attention) — maps to selective attention routing. Δ-layers = consciousness-level filtering. **Absorption candidate**: three-tier attention partition (full → identify → sparse).

### 9. LightMem (ICLR 2026)
- **URL**: https://proceedings.iclr.cc/paper_files/paper/2026/file/a05b72653ec5b473732129829ae04195-Paper-Conference.pdf
- **Category**: Lightweight Memory Framework
- **Pattern**: Three-stage memory inspired by Atkinson-Shiffrin model: Sensory (fast filtering) → Short-term (organized buffer) → Long-term (consolidated storage). Soft updates during test time preserve global information. Reduces memory system overhead while maintaining performance.
- **NeoTrix Mapping**: NT-MEMORY — directly maps to experience-tree write path. Three-stage = sensory → working → consolidated. Soft updates = non-destructive memory evolution. **Absorption candidate**: Atkinson-Shiffrin-inspired memory pipeline with soft-update policy.

### 10. EvoRoute (ACL 2026)
- **URL**: https://aclanthology.org/2026.acl-long.1771
- **Category**: Self-Evolving Model Routing
- **Pattern**: Self-evolving routing paradigm for Agent System Trilemma (performance vs cost vs latency). Uses expanding knowledge base of prior experience + Pareto-optimal selection. Dynamically selects LLM backbone per step. 80% cost reduction, 70%+ latency reduction.
- **NeoTrix Mapping**: NT-CORE (GWT + SelfModel) — maps to cost-aware routing (A1) + experience-driven adaptation (P3). Pareto selection = multi-objective GWT salience. **Absorption candidate**: experience-weighted Pareto routing.

## Cross-Cutting Themes (Cycle 337)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Context-as-Filesystem** | OpenViking, GAAI | Reinforces viking:// URI paradigm for NT-MEMORY |
| **Mandatory Skills** | Superpowers, GAAI | Auto-triggering workflows > manual invocation |
| **Hybrid Memory Search** | Cortex MCP, LightMem | BM25 + semantic RRF fusion for KB |
| **Template Collapse Detection** | RAGEN-2 | MI-based quality diagnostic for ConsciousnessTree |
| **Trajectory-Aware Routing** | IQ Routing, EvoRoute | History-conditioned GWT salience |
| **Governed Autonomy** | GAAI, GAIE | Three-tier oversight for agent actions |

## Prioritization

| Priority | Project | Action |
|----------|---------|--------|
| P0 | OpenViking | Deep-dive filesystem-as-context architecture |
| P0 | RAGEN-2 | Implement MI diagnostic for ConsciousnessTree |
| P1 | Superpowers | Map skill auto-trigger patterns to NT skill nodes |
| P1 | LightMem | Integrate Atkinson-Shiffrin memory pipeline |
| P2 | EvoRoute | Extend GWT with Pareto routing |
| P2 | Cortex MCP | Hybrid RRF search for KB |
| P3 | GAAI | Governance folder convention study |
| P3 | IQ Routing | Trajectory-aware cost routing |
| P3 | Mozaik | Concurrent agent runtime reference |
| P3 | DELTA | Three-tier attention partition study |
