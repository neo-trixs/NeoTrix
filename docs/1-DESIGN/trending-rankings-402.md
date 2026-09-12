# Trending Rankings — Cycle 402

**Date:** 2026-09-12
**Scope:** AI agents, LLM tools, reasoning frameworks, memory systems, attention/routing innovations

---

## 1. OpenViking — Context Database for AI Agents
- **Repo:** volcengine/OpenViking (36K+ ⭐, 2.7K forks)
- **Pattern:** Self-evolving context database using filesystem paradigm (`viking://` protocol). Three-tier context loading (L0 abstract → L1 overview → L2 details). Memory, resources, skills unified as browsable directories.
- **NeoTrix Relevance:** Directly mirrors NT-MEMORY KB design — tiered context loading, session-to-memory extraction, observable retrieval trajectories. The `viking://` URI scheme parallels our `kv_store` namespace architecture. Potential integration pattern for NeoTrix memory subsystem.
- **Domain:** NT-MEMORY

## 2. Attention-MoA (Mixture-of-Agents)
- **Paper:** arXiv:2601.16596 (Jan 2026)
- **Pattern:** Inter-agent semantic attention + inter-layer residual synthesis with adaptive early stopping. Small open-source model ensemble outperforms Claude-4.5-Sonnet (MT-Bench 8.83, AlpacaEval 2.0 LC 77.36%).
- **NeoTrix Relevance:** Maps to GWT salience routing — agents attend to each other's semantic representations, not just raw outputs. Residual connections prevent information degradation in deep agent layers. Applicable to NT-CORE multi-agent coordination.
- **Domain:** NT-CORE (GWT routing)

## 3. CacheWise — Agent-Aware KVCache Management
- **Paper:** arXiv:2606.16824 (Jun 2026)
- **Pattern:** Prefix-aware request scheduling + predictive KVCache eviction for coding agent workloads. Reuses KV cache across successive requests within a session. Reduces overhead from recomputation.
- **NeoTrix Relevance:** Directly applicable to NeoTrix's KVMem paged KV strategy. Session-aware cache reuse aligns with our step-level scheduling insight (A2: Context as Scarce Resource). Could optimize `kv_cache_optimizer.rs`.
- **Domain:** NT-CORE (reasoning infrastructure)

## 4. MemDecay — Region-Aware KV Cache Eviction
- **Paper:** arXiv:2607.10582 (Jul 2026)
- **Pattern:** Semantic region-aware eviction — system tokens have half-lives 148-190 steps vs scratchpad tokens 14-16 steps. Training-free, region-specific base priorities + decay rates. Critical regions pinned.
- **NeoTrix Relevance:** Validates our context-as-scarce-resource axiom (A2). System instruction tokens should be pinned in NeoTrix sessions while working memory decays. Direct extension of KVMem tiered strategy.
- **Domain:** NT-CORE (memory architecture)

## 5. Explicit Trait Inference (ETI) for Multi-Agent Coordination
- **Paper:** arXiv:2604.19278 (ACL 2026)
- **Pattern:** Agents infer partner traits along warmth/competence dimensions from interaction histories. Reduces payoff loss 45-77% in economic games, improves performance 3-29% on MultiAgentBench.
- **NeoTrix Relevance:** Maps to NT-MIND SelfModel dynamics — agents build trait profiles of collaborators. Could enhance NT-ACT tool routing by tracking provider reliability (warmth) and capability (competence) dimensions.
- **Domain:** NT-MIND (SelfModel) + NT-ACT (routing)

## 6. MagiCrew — Open-Source Multi-Agent Workforce Platform
- **Repo:** magi-crew (ProductHunt #1 Sep 3, 2026)
- **Pattern:** Deploy specialized digital workers that research, analyze, create reports. Multi-agent collaboration with enterprise controls and deliverable-ready outputs.
- **NeoTrix Relevance:** Validates NT-ACT's production orchestration pattern — agents as specialized workers, not generic chatbots. Enterprise control layer aligns with NT-GOVERNANCE policy enforcement.
- **Domain:** NT-ACT (orchestration)

## 7. Nanobot — Ultra-Lightweight Personal AI Agent
- **Repo:** HKUDS/nanobot (47.7K ⭐, MIT)
- **Pattern:** Python agent framework with WebUI, MCP, multi-agent workflows, model routing, scheduled automation. OpenAI-compatible API. Persistent workflows survive long-running tasks.
- **NeoTrix Relevance:** Lightweight agent runtime pattern — contrasts with NeoTrix's heavy architecture but validates the "small core + composable skills" approach. Model routing with fallback chains mirrors our Ordered Backend Router pattern (P4).
- **Domain:** NT-IO (interface/runtime)

## 8. AgentInfer — Co-Design of Inference Architecture and System
- **Paper:** arXiv:2512.18337 (Dec 2025, v2 Feb 2026)
- **Pattern:** 4-component acceleration: AgentCollab (hierarchical dual-model), AgentSched (cache-aware hybrid scheduler), AgentSAM (speculative decoding with semantic memory), AgentCompress (async memory distillation). 50%+ token reduction, 1.8-2.5x speedup.
- **NeoTrix Relevance:** AgentSAM's multi-session semantic memory reuse directly maps to our SEAL pipeline's experience crystallization. AgentCompress's async distillation mirrors our experience-tree absorption protocol. The hierarchical dual-model pattern validates our dual specialization (Weapon Set I/II).
- **Domain:** NT-MIND (SEAL) + NT-CORE (reasoning)

## 9. HugAgentOS — Self-Evolving AgentOS
- **Repo:** ZJU-REAL/HugAgentOS (440 ⭐)
- **Pattern:** Ontology-grounded trustworthy reasoning for self-evolving agent operating systems. Desktop-app agent harness with trust verification.
- **NeoTrix Relevance:** Trust-grounded agent OS pattern aligns with NT-SHIELD security domain. Ontology-based reasoning maps to our VSA HyperCube knowledge representation. Self-evolution with trust verification validates our SEAL pipeline's self-test gates.
- **Domain:** NT-SHIELD + NT-MIND

## 10. Flare — Graph-First IDE for Agentic Coding
- **Repo:** Flare (ProductHunt launched Aug 2026)
- **Pattern:** Interactive map for agentic coding — visualizes code relationships as a graph. Agents navigate codebases through graph traversal rather than linear file reading.
- **NeoTrix Relevance:** Graph-based code navigation maps to our DependencyGraph and capability tree visualization. Could enhance NT-CORE's E8 reasoning engine by representing module relationships as navigable graphs.
- **Domain:** NT-CORE (reasoning visualization)

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|-------------------|
| **Tiered Context Loading** | OpenViking, MemDecay | KV cache tiered strategy (A2 axiom) |
| **Agent-as-Worker** | MagiCrew, Nanobot | NT-ACT production orchestration |
| **Semantic Memory Reuse** | CacheWise, AgentInfer | SEAL experience crystallization |
| **Trait-Based Coordination** | ETI | SelfModel dynamic profiling |
| **Graph-Based Navigation** | Flare | E8 reasoning visualization |
| **Lightweight Runtimes** | Nanobot | Model routing with fallback |

---

## Actionable Absorptions (Priority)

1. **MemDecay region-aware eviction** → Extend `kv_cache_optimizer.rs` with semantic region classification
2. **ETI trait inference** → Add warmth/competence tracking to SelfModel for provider routing
3. **CacheWise session-aware caching** → Implement prefix-aware scheduling in KVMem
4. **AgentInfer AgentSAM** → Integrate semantic memory reuse into SEAL speculative execution
