# Model Reverse Engineering — Cycle 328

**Date**: 2026-09-11
**Papers**: 5 recent papers on KV scheduling, agent routing, skill diversity, and game-theoretic coordination

---

## 1. UNISON: Co-Designed Near-Memory Scheduler for Session KV Residency

**Source**: arXiv:2609.09643, Sep 2026
**Domain**: KV cache management for LLM agent loops

### Core Contribution
Agent loops (plan → tool call → resume) stress memory hierarchies differently than multi-turn chat. Sessions hold growing KV prefixes across tool waits; multiple sessions share SRAM/HBM. UNISON is an event-driven near-memory scheduler that jointly optimizes eviction (who leaves) and tiering (who sits in fast tier) using a unified live ranking.

### Key Mechanisms
1. **SPEAR (Survival-Penalty Eviction for Agent Return-gap)**: Selects eviction candidates based on gap average and turn-indexed hazard — penalizes sessions returning from long tool waits
2. **TIDE (Tiering in Idle-window DMA Events)**: Spends observed wait time as a DMA budget for fast-tier placement — idle sessions get tiered out proportionally to their expected idle duration
3. **Unified Ranking**: SPEAR and TIDE share one live ranking rather than independent eviction/tiering decisions
4. **28nm CMOS Core**: 0.169mm² at 13.6mW — scheduling overhead negligible relative to KV hierarchy it manages

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY (KVMem)** | Session-aware KV eviction — agent loops have different access patterns than chat | `nt_memory::kvmem::agent_session_scheduler` |
| **Axiom A2 (Context as Scarce)** | KV residency budget = context budget at hardware level. Tiering decisions = context compaction vs paging decisions | `kv_cache_optimizer.rs` |
| **NT-PHYSICAL** | Hardware-software co-design for KV management. Near-memory scheduling = scheduling close to where data lives | `nt_physical::memory_management` |
| **ConsciousnessTree** | Session-level health monitoring — which agent sessions are alive, which are idle, which should be evicted | `nt_meta::session_health` |

### Actionable Insight
Our KVMem compaction-vs-paging decision should incorporate agent-loop awareness. Sessions returning from tool waits (long gaps) are different from continuous chat sessions. The SPEAR gap-average mechanism directly applies to our `kv_cache_optimizer.rs` tiering policy.

---

## 2. TRIAGE: Three-Level Routing with Trajectory-as-a-Skill

**Source**: arXiv:2609.01428, Sep 2026
**Domain**: Token-efficient agent execution via trajectory reuse

### Core Contribution
Every ReAct query triggers a complete reasoning loop from scratch. TRIAGE classifies queries into three levels: (1) Direct Reuse — identical queries, 0 tokens; (2) Skill Substitution — similar queries, 0 tokens via deterministic parameter substitution; (3) Full ReAct — novel queries, automatically stored for future reuse. The core innovation is TaaS (Trajectory-as-a-Skill) — abstracting historical execution trajectories into reusable skills.

### Key Mechanisms
1. **Three-Level Routing**: L1 (exact match, 0 tokens) → L2 (parameter substitution, 0 tokens) → L3 (full ReAct, stored for future)
2. **TaaS (Trajectory-as-a-Skill)**: High-frequency trajectory patterns distilled into deterministic skills — "experience as a service"
3. **Online Learning**: L2 hit rate rises from 0% to 57% within first 100 queries; average token cost drops from 198 to 74.7
4. **Cross-Domain Validation**: 76.3% token reduction on ToolBench (15 domains, 345 queries)

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MIND (SEAL)** | Trajectory crystallization = skill node creation from execution traces. TaaS = SEAL distillation stage | `seal::trajectory_distillation` |
| **NT-MEMORY** | Skill storage as KB entries with parameter substitution schemas | `experience-tree::skill_entries` |
| **Axiom A1 (Cost-Aware)** | Three-level routing = progressive cost optimization. L1/L2 = zero-cost, L3 = full cost | `nt_core_gwt::cost_routing` |
| **NT-ACT** | Trajectory reuse = tool execution pattern caching. Parameter substitution = tool argument templating | `nt_act::tool_caching` |

### Actionable Insight
Our experience-tree absorption should implement TaaS-style trajectory crystallization. When an execution trajectory repeats with high frequency, automatically distill it into a parameterized skill node. The three-level routing (exact → parameterized → full) should be the default cost optimization for NT-ACT tool execution.

---

## 3. HeRo: History-Aware Routing with Router Memory

**Source**: arXiv:2609.08189, Sep 2026
**Domain**: Dynamic layer routing for efficient LLM inference

### Core Contribution
Dynamic layer routing skips layers per-token to reduce inference cost, but existing methods treat each routing decision as local (conditioned only on current hidden state). HeRo introduces a router memory mechanism — explicit routing state maintained across model depth via linear attention. The router conditions jointly on accumulated routing history and current hidden representation to select the executed branch.

### Key Mechanisms
1. **Router Memory**: Linear attention mechanism that incrementally aggregates preceding routing scores and induced residual updates into compact history representation
2. **Joint Conditioning**: Each routed layer conditions on both accumulated routing state AND current hidden representation
3. **Performance Retention**: On Llama 3.1-8B, bypasses 26.87% of parameters while achieving 100.24% of dense performance across 7 benchmarks
4. **Ablation**: Removing routing history consistently degrades performance, especially on multistep reasoning and code generation

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **GWT (NT-CORE)** | Router memory = GWT attention history — what was broadcast before influences what is broadcast next | `nt_core_gwt::attention_memory` |
| **ConsciousnessTree** | Cross-cycle state propagation — routing decisions at cycle N inform cycle N+1 | `nt_meta::cycle_state` |
| **Axiom A2 (Context as Scarce)** | Compact routing history = minimal state carried across decisions. Linear attention aggregation keeps memory bounded | `nt_core_self::routing_state` |
| **NT-MEMORY** | Router memory = execution memory for routing decisions. Not full trajectory, but compact routing-relevant history | `nt_memory::routing_cache` |

### Actionable Insight
Our GWT salience mechanism should maintain routing history across attention cycles. Currently, each GWT broadcast is independent. HeRo shows that accumulating routing history via linear attention improves downstream decisions. Add a compact routing memory to `AttentionManager` that aggregates previous salience scores.

---

## 4. Diverse Skill Routing (DSR): Beyond Top-k Retrieval

**Source**: arXiv:2609.05824, Sep 2026
**Domain**: Skill selection for LLM agents with redundancy awareness

### Core Contribution
Existing skill routers rank candidates independently by query relevance, wasting context budget on redundant skills. DSR uses a Determinantal Point Process (DPP) to balance relevance and non-redundancy. Introduces a query-residual diversity kernel that penalizes redundant skill overlap while reducing penalties caused only by shared query relevance.

### Key Mechanisms
1. **DPP-Based Reranking**: Determinantal Point Process balances relevance (diagonal) and diversity (off-diagonal) in skill selection
2. **Query-Residual Diversity Kernel**: Penalizes skill overlap that is NOT explained by shared query relevance — two skills are only "redundant" if they overlap beyond what the query demands
3. **Complementary Set Selection**: Skill routing reframed from relevance ranking to complementary set selection — the selected set should cover the query without redundancy
4. **Multi-Skill Query Gains**: Larger improvements on queries requiring multiple complementary skills

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MIND (Skill Engine)** | Skill selection as DPP optimization — not just top-k but diversity-aware | `nt_mind_skill_engine::diversity_routing` |
| **GWT (NT-CORE)** | Attention broadcast should be diversity-aware — don't broadcast redundant salient signals | `nt_core_gwt::diversity_broadcast` |
| **CapabilityRegistry** | Skill node selection = complementary set construction, not independent ranking | `nt_core_capability::diversity_selection` |
| **Axiom A2 (Context as Scarce)** | Diversity-aware selection keeps context compact — no redundant skill descriptions consuming context budget | `context_budget::diversity_optimizer` |

### Actionable Insight
Our CapabilityRegistry skill selection should adopt DPP-based diversity-aware reranking. Currently, skill nodes are selected by relevance alone. DSR shows that selecting a complementary set (relevance + diversity) outperforms top-k selection, especially for multi-capability tasks. The query-residual kernel is the key insight — penalize overlap only beyond what the query requires.

---

## 5. Bilevel Coordinated Reflection: Game-Theoretic Multi-Agent Memory

**Source**: arXiv:2609.02750, Sep 2026
**Domain**: Multi-agent LLM coordination via game theory and reflective memory

### Core Contribution
Models orchestrator-worker interaction as a bilevel coordination game. Workers' local-update game is an approximate potential game whose equilibrium slack is controlled by decomposition quality. Reflection analyzed as stochastic movement over semantic memory states. Introduces SRMA (Stochastic Reflective Memory Ascent) — accepts candidate memory only after grounded evaluation risk strictly decreases.

### Key Mechanisms
1. **Bilevel Coordination Game**: Upper level (orchestrator decomposition) controls lower level (worker execution quality). Equilibrium slack = decomposition quality gap
2. **Reflection as Stochastic Drift**: Free-form reflection modeled as movement over semantic memory states. Finite-time convergence bounds proven.
3. **Information-Theoretic Impossibility**: No gate observing only generated transcript can improve uniformly — environment-grounded evaluation required
4. **SRMA (Stochastic Reflective Memory Ascent)**: Accept memory only when grounded evaluation risk strictly decreases. Converges geometrically or polynomially under calibration.
5. **SWE-bench Result**: Kimi-based system resolves 72.2% vs 70.8% baseline

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** | SRMA = experience-tree quality gate — accept new experience only when retrieval risk decreases | `experience-tree::srm_gate` |
| **NT-MIND (SEAL)** | Reflection as stochastic drift — SEAL feedback loop can be modeled as movement in semantic space | `seal::reflection_dynamics` |
| **NT-GOVERNANCE** | Bilevel game = orchestrator-worker governance. Decomposition quality controls worker performance | `nt_governance::bilevel_control` |
| **NT-CORE (ConsciousnessTree)** | Environment-grounded evaluation — ConsciousnessTree health should be grounded in execution outcomes, not self-assessment | `nt_meta::grounded_evaluation` |

### Actionable Insight
Our experience-tree absorption should implement SRMA-style acceptance gating. Before committing new experience to KB, evaluate whether accepting it reduces future retrieval risk (not just whether it's accurate). The information-theoretic impossibility result is critical — our quality gates must be grounded in execution outcomes, not just transcript coherence.

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Memory as Active Control Signal (UNISON, HeRo, SRMA)
All three papers treat memory not as passive storage but as active control:
- UNISON: KV residency decisions control which sessions survive
- HeRo: Router memory controls which layers execute
- SRMA: Memory acceptance controls knowledge quality

**NeoTrix Mapping**: Our KB should be an active control signal, not just a store. Experience entries should influence routing decisions (GWT), not just be retrieved on demand.

### Pattern 2: Progressive Cost Optimization (TRIAGE, DSR, HeRo)
All papers implement progressive cost reduction:
- TRIAGE: 3-level routing (0 tokens → 0 tokens → full)
- DPP: Diversity-aware selection reduces redundant context
- HeRo: History-aware routing reduces parameter execution

**NeoTrix Mapping**: Axiom A1 (Cost-Aware Routing) should be implemented as progressive layers: exact match → parameterized reuse → full execution. Each layer has a different cost profile.

### Pattern 3: Verify-Before-Commit (SRMA, UNISON)
Both papers verify before committing state changes:
- SRMA: Accept memory only when retrieval risk decreases
- UNISON: Tiering decisions verified against session survival metrics

**NeoTrix Mapping**: Experience-tree absorption should adopt verify-before-commit. Generate retrieval probes against candidate experience entries. Only commit when probe success rate exceeds threshold.

### Pattern 4: History-Aware Decision Making (HeRo, TRIAGE)
Both papers show that routing decisions improve with history:
- HeRo: Router memory across depth improves layer routing
- TRIAGE: Trajectory history enables skill crystallization

**NeoTrix Mapping**: GWT salience should maintain routing history. ConsciousnessTree should carry cross-cycle state. SEAL pipeline should learn optimal stage transitions from execution history.

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | TRIAGE | Implement TaaS trajectory crystallization in experience-tree | experience-tree SKILL |
| P0 | SRMA | Add retrieval-risk acceptance gate to experience absorption | experience-tree SKILL |
| P1 | HeCo | Add router memory to GWT salience mechanism | nt_core_gwt |
| P1 | DSR | Implement DPP diversity-aware skill selection | nt_mind_skill_engine |
| P2 | UNISON | Add agent-loop-aware KV eviction to kv_cache_optimizer | nt_memory::kvmem |
