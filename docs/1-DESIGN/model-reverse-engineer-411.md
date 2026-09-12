# Model Reverse Engineering — Cycle 411

**Date**: 2026-09-12
**Period**: Aug-Sep 2026
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory routing

---

## Paper 1: AgentInfer — Co-Design of Inference Architecture and System

**Source**: arXiv:2512.18337v2 (Feb 2026)
**Authors**: Weizhe Lin et al. (Huawei)
**Benchmarks**: BrowseComp-zh, DeepDiver

### Core Innovation
End-to-end agent acceleration framework with 4 synergistic components:
1. **AgentCollab**: Hierarchical dual-model reasoning (large + small model via dynamic role assignment)
2. **AgentSched**: Cache-aware hybrid scheduler for heterogeneous request patterns
3. **AgentSAM**: Suffix-automaton-based speculative decoding reusing multi-session semantic memory
4. **AgentCompress**: Semantic compression that asynchronously distills agent memory without disrupting reasoning

### Key Results
- 50%+ reduction in ineffective token consumption
- 1.8-2.5x overall speedup with preserved accuracy
- Self-Evolution Engine sustaining efficiency across long-horizon reasoning

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| AgentCollab (dual-model) | NT-CORE | GWT cost-aware routing (Axiom A1) — route to cheap/expensive model based on task complexity |
| AgentSched (cache-aware) | NT-ACT | Work scheduling with cache locality awareness |
| AgentSAM (speculative decode) | NT-MEMORY | Reuse semantic memory across sessions for inference acceleration — relates to KVMem delta reuse |
| AgentCompress (async distill) | NT-MIND | Background memory distillation without blocking reasoning — SEAL pipeline background cycle |
| Self-Evolution Engine | NT-META | ConsciousnessTree cycle with efficiency preservation |

### Absorbed Pattern
`agent-infer-codesign` — Joint optimization of inference architecture + system scheduling. Key insight: optimizing per-token throughput is insufficient; must optimize for agentic task completion. Maps to NT-CORE's GWT salience with cost weight (Axiom A1).

---

## Paper 2: Attention-MoA — Inter-Agent Semantic Attention

**Source**: arXiv:2601.16596 (Jan 2026)
**Authors**: Jianyu Wen et al.
**Benchmarks**: AlpacaEval 2.0, MT-Bench, FLASK

### Core Innovation
Mixture-of-Agents framework with inter-agent semantic attention:
- Agents attend to each other's semantic representations (not just concatenation)
- Inter-layer Residual Module with Adaptive Early Stopping
- Small open-source models ensemble outperforms Claude-4.5-Sonnet and GPT-4.1

### Key Results
- 91.15% Length-Controlled Win Rate on AlpacaEval 2.0
- MT-Bench score of 8.83 (ensemble of small models)
- Dominates 10/12 capabilities on FLASK

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Inter-agent semantic attention | NT-CORE | GWT broadcast with resonance-based routing — agents attend to salient signals from other modules |
| Residual connections | NT-CORE | HyperCube residual pathways for information preservation across layers |
| Adaptive early stopping | NT-META | ConsciousnessTree adaptive cycle depth — stop when convergence detected |
| Small model ensemble | NT-IO | Cost-aware model routing — ensemble cheap models instead of single expensive one |

### Absorbed Pattern
`attention-moa-ensemble` — Small model ensemble via inter-agent semantic attention can surpass large proprietary models. Maps to GWT's resonance-based routing where specialist modules attend to each other's representations. Validates NT-CORE's multi-module broadcast architecture.

---

## Paper 3: Three Roles, One Model — Inference-Time Scaffolding

**Source**: arXiv:2604.11465v2 (Apr 2026)
**Authors**: S. Aaron McClendon et al.
**Benchmarks**: AppWorld

### Core Innovation
Three-tier inference scaffolding using the SAME frozen model in three roles:
1. **Summarization Model**: Preserves critical artifacts while compressing dialogue history
2. **Main Agent Model**: Reasons over compressed context
3. **Correction Model**: Reviews and revises agent output without conversation history (breaks repetitive failure loops)

### Key Results
- Qwen3-8B raw: 5.4% task completion → scaffolded: 8.9% (FP16)
- 8B scaffolded model surpasses DeepSeek-Coder 33B (7.1%)
- Doubles performance on difficulty-1 tasks
- Single 24GB GPU deployment

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Summarization role | NT-MEMORY | Context compression — preserve critical artifacts, compress noise |
| Main agent role | NT-CORE | Core reasoning over compressed context |
| Correction role | NT-REPAIR | Self-healing: isolated correction model reviews and fixes output |
| Same-model multi-role | NT-IO | Role-based routing with single model — different conditioning, same weights |
| Breaking failure loops | NT-META | ConsciousnessTree detects and breaks repetitive reasoning patterns |

### Absorbed Pattern
`single-model-triple-role` — One frozen model serves three distinct roles via different conditioning. Key insight: structured inference-time interventions make 8B models competitive with 33B. Maps to NT-REPAIR's self-healing isolation pattern and NT-MEMORY's context compression.

---

## Paper 4: MemRouter — Memory-as-Embedding Routing

**Source**: arXiv:2605.00356 (May 2026)
**Authors**: Under review

### Core Innovation
Write-side memory router decoupling memory admission from downstream answer backbone:
- Embedding-based routing policy replaces per-turn LLM memory management
- Lightweight classification heads for operation classification (ADD/UPDATE/DELETE/NOOP)
- Bypasses autoregressive generation for memory decisions
- Route-then-compile design: intent routing → fixed memory path → answer generation

### Key Components
- **Router Agent**: Classifies query intent, dispatches to memory tier
- **Memory Agent**: Executes one of three tiers (Profile Lookup / Targeted Retrieval / Deep Reasoning)
- **Answer Agent**: Generates from compact, intent-matched evidence
- **Validator Agent**: Retries with heavier memory tier when response unsupported

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Embedding-based routing | NT-CORE | GWT salience scoring — route based on embedding similarity, not autoregressive |
| Write-side admission | NT-MEMORY | KB write gating — decide what enters persistent memory |
| Three memory tiers | NT-MEMORY | Tiered memory: hot (profile) / warm (retrieval) / cold (deep reasoning) |
| Route-then-compile | NT-ACT | Intent classification → fixed execution path → result assembly |
| Validator retry | NT-REPAIR | Self-healing: validate output, retry with heavier computation if needed |

### Absorbed Pattern
`embed-router-memory-admission` — Lightweight embedding-based router for memory write decisions, replacing expensive LLM-based memory management. Key insight: routing memory at the embedding level is faster and more reliable than autoregressive generation. Maps directly to NT-MEMORY KB write pipeline and GWT attention routing.

---

## Paper 5: Supra Cognitive Modes — Routed Architecture for Agent Memory

**Source**: arXiv:2607.19096 (Jul 2026)
**Authors**: Joshua Tobkin, David Yang (Supra Research)
**Benchmarks**: LoCoMo, LongMemEval, MemoryAgentBench

### Core Innovation
Routed architecture organizing different memory computation paths over shared substrate:
- Three procedure families: factual lookup, relation-chain reasoning, broad synthesis
- Latest-revision precedence for current-state queries
- Shared asynchronous substrate — all procedures read from one ingest substrate
- Per-query control interface for routing decisions

### Key Insight
Agent-memory workloads mix fundamentally different computation types (lookup vs reasoning vs synthesis). Routing the computation TYPE, not just the model, enables optimal resource allocation.

### NeoTrix Domain Mapping

| Component | NeoTrix Domain | Implementation |
|-----------|---------------|----------------|
| Factual lookup path | NT-MEMORY | Direct KB query (BM25 + vector search) |
| Relation-chain reasoning | NT-CORE | HyperCube traversal — chain reasoning over knowledge graph edges |
| Broad synthesis | NT-MIND | SEAL pipeline synthesis — distill across knowledge domains |
| Shared substrate | NT-MEMORY | Unified KB with multiple query interfaces |
| Per-query routing | NT-CORE | GWT salience routes to appropriate computation path |
| Latest-reversion precedence | NT-MEMORY | Temporal ordering in KB — most recent version takes priority |

### Absorbed Pattern
`cognitive-mode-routing` — Route memory queries to different computation paths based on intent (lookup/reasoning/synthesis). Key insight: one computation path cannot optimally handle all memory workloads. Maps to GWT's specialist module routing and NT-MEMORY's multi-index KB architecture.

---

## Cross-Paper Synthesis: Patterns for NeoTrix Integration

### Pattern 1: Hierarchical Dual-Model (AgentInfer)
**NeoTrix Integration**: NT-CORE GWT with cost-aware model routing (Axiom A1)
- Route simple perception tasks to cheap models (NT-WORLD/NT-ACT)
- Route complex reasoning to expensive models (NT-CORE/NT-MIND)
- Dynamic role assignment based on task complexity

### Pattern 2: Inter-Agent Semantic Attention (Attention-MoA)
**NeoTrix Integration**: NT-CORE HyperCube + GWT broadcast
- Specialist modules attend to each other's semantic representations
- Residual connections preserve information across attention layers
- Adaptive early stopping for convergence detection

### Pattern 3: Single-Model Triple-Role (Three Roles)
**NeoTrix Integration**: NT-REPAIR + NT-MEMORY + NT-CORE
- Same model serves different roles via different conditioning
- Isolated correction role prevents failure loop propagation
- Maps to ConsciousnessTree's self-healing isolation

### Pattern 4: Embedding-Level Memory Routing (MemRouter)
**NeoTrix Integration**: NT-MEMORY KB write pipeline
- Lightweight embedding router for memory admission decisions
- Three-tier memory: hot/warm/cold
- Validator agent for grounding verification

### Pattern 5: Cognitive Mode Routing (SCM)
**NeoTrix Integration**: NT-CORE GWT + NT-MEMORY multi-index
- Route to different computation paths based on query intent
- Shared substrate with per-query control interface
- Latest-revision precedence for temporal queries

---

## Priority Absorption Targets

| Priority | Pattern | Source | NeoTrix Domain | Effort |
|----------|---------|--------|----------------|--------|
| P0 | Embedding-level memory routing | MemRouter | NT-MEMORY | Medium |
| P0 | Cognitive mode routing | SCM | NT-CORE + NT-MEMORY | High |
| P1 | Hierarchical dual-model routing | AgentInfer | NT-CORE | Medium |
| P1 | Single-model triple-role | Three Roles | NT-REPAIR | Low |
| P2 | Inter-agent semantic attention | Attention-MoA | NT-CORE | High |
| P2 | Async semantic compression | AgentInfer | NT-MIND | Medium |

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MemRouter bypasses LLM for memory vs NeoTrix uses LLM for KB decisions | Adopt embedding routing for write-side admission; keep LLM for complex reasoning queries |
| AgentInfer async compression vs SEAL synchronous pipeline | Run compression as background task (already supported by SEAL Phase-4) |
| Three Roles uses same model vs NeoTrix uses different models | Implement as model-agnostic conditioning — same weights, different system prompts |
| SCM shared substrate vs NeoTrix multi-database | Unified KB (single substrate) with multiple query interfaces (already architecture) |
| Attention-MoA small model ensemble vs cost-aware single model | Ensemble only when quality critical; single model when cost constrained |
