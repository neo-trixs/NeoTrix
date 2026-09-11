# Model Reverse Engineering — Cycle 352 (2026-09-12)

## Selection Criteria

Papers/models from Sep 2026 on efficient inference, attention mechanisms, agent coordination, and memory systems. Focus on patterns transferable to NeoTrix's 7-domain architecture. Excluded papers present in cycles 318–351.

---

## Paper 1: CEDAR — Error-Bounded Residual Routing for Long-Context Attention

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2609.07237 (Sep 7, 2026) |
| **Title** | CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention |
| **Authors** | — |

### Core Idea

Post-hoc sparse attention with coarse-to-fine refinement. Each semantic chunk contributes a cheap key-value summary to a residual attention path. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in single softmax — refinement replaces rather than duplicates coarse evidence. Derives output-error bound governed by within-chunk key/value dispersion to allocate variable refinement budget.

### Key Insight

Hard sparse routing (zero probability for omitted chunks) loses recoverable information. Residual summaries reduce reconstruction error by 98%+ vs hard dropping at equal budgets. The error bound enables adaptive refinement — easy queries get cheap summaries, ambiguous queries get exact attention.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-CORE** (GWT) | Error-aware attention routing | GWT salience could incorporate approximation error bounds — route easy queries cheaply, expensive queries exactly |
| **NT-MEMORY** (KB) | Tiered retrieval精度 | KB retrieval could use residual summaries for initial scan, expand to full retrieval only for high-error queries |
| **NT-IO** (providers) | Adaptive precision routing | Route simple tasks to cheap models (summary), hard tasks to expensive models (exact) |

### Implementation Sketch

```rust
// Error-bounded tiered retrieval for KB
fn cedar_retrieve(query: &Query, kb: &KB) -> RetrievalResult {
    // Tier 1: Cheap residual summaries
    let summaries = kb.summarized_chunks(query, budget: 64);
    let error_estimates = estimate_approximation_error(&summaries, query);

    // Tier 2: Expand high-error chunks to exact retrieval
    let mut result = summaries.low_error;
    for chunk in summaries.high_error {
        let exact = kb.exact_retrieve(query, chunk, budget: 256);
        result.merge(exact);  // replaces summary, doesn't duplicate
    }

    // Single normalization pass
    result.normalize()  // error-bounded output
}
```

### Risk Assessment

- **Low risk**: Additive to existing retrieval — can start with simple error estimation
- **Validation**: Test on multi-hop QA where retrieval precision matters
- **Extension**: Could integrate with GWT for attention-budget-aware routing

---

## Paper 2: UNISON — Session KV Scheduling for LLM Agents

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2609.09643 (Sep 9, 2026) |
| **Title** | UNISON: A Co-Designed Near-Memory Scheduler of Session KV Residency for LLM Agents |
| **Authors** | — |

### Core Idea

Event-driven near-memory scheduler that treats KV residency as a session-level efficiency problem. Two mechanisms share one live ranking:
- **SPEAR** (Survival-Penalty Eviction for Agent Return-gap): Selects who leaves based on gap average and turn-indexed hazard
- **TIDE** (Tiering in Idle-window DMA Events): Spends observed wait as DMA budget for who sits in fast tier

28nm CMOS scheduling core: 0.169mm², 13.6mW, 150MHz. Raises hit rate 0.3-23.1%, reduces AMAT 22-51%, lowers TTFT 58-89% on long-horizon traces.

### Key Insight

Agent loops press shared memory harder than chat because they hold growing KV prefix across tool waits. Existing eviction proxies (recency, timeout, identity) miss loop mechanism — they treat a live wait as cold/discardable. Session-level scheduling is structurally necessary, not decomposable into independent components.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-MEMORY** (KB) | Session-aware cache management | KB cache eviction should account for agent loop patterns, not just recency |
| **NT-PHYSICAL** (embodiment) | Near-memory scheduling for edge | Edge devices need efficient KV management — SPEAR/TIDE patterns for constrained hardware |
| **NT-CORE** (reasoning) | Tool-wait-aware context retention | When agent calls tools, keep KV alive for resumption — don't evict during waits |

### Implementation Sketch

```rust
// Session-aware KV eviction for NT-MEMORY
struct SessionKVManager {
    active_sessions: HashMap<SessionId, SessionState>,
    eviction_ranker: SPEARRanker,
    tier_manager: TIDETierManager,
}

impl SessionKVManager {
    fn on_tool_wait(&mut self, session: &SessionId) {
        // Don't evict — mark as "return-gap" pending
        self.eviction_ranker.mark_return_gap(session);
    }

    fn on_resume(&mut self, session: &SessionId) {
        // Boost priority — session is active again
        self.tier_manager.promote_to_fast_tier(session);
    }

    fn evict(&mut self) -> Option<KVBlock> {
        // SPEAR: gap-aware + hazard-aware eviction
        self.eviction_ranker.select_victim(
            gap_threshold: GAP_AVG,
            hazard_factor: TURN_INDEX
        )
    }
}
```

### Risk Assessment

- **Medium risk**: Requires deep integration with KV cache management
- **Mitigation**: Start with software-only SPEAR, add TIDE after profiling
- **Validation**: Benchmark on multi-tool agent tasks with long waits

---

## Paper 3: Codebook Agent — Amortized Topology Design for Multi-Agent Systems

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2609.02264 (Sep 2, 2026) |
| **Title** | Codebook Agent: Amortized Topology Design for LLM Multi-Agent Systems |
| **Authors** | — |

### Core Idea

Vector-quantized autoencoder compresses successful topologies into a query-independent 16-entry codebook. Reward-weighted MLP maps query embedding to distribution over codes. MLP proxy reads flattened adjacency, regressed on measured utility and per-task normalized token cost, reranks top candidates in single batched forward pass. No iterative search, no message passing at test time.

### Key Insight

Three empirical facts motivate this: (1) topologies collapse to ~6 distinct graphs even at 64 capacity; (2) edge count is negatively correlated with token consumption (Pearson r≈-0.4), so sparsifying makes inference more expensive; (3) message-passing scorer is adjacency-invariant when agents share profiles. Codebook approach is 21.9-33.2% more token-efficient than learned-topology methods.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-CORE** (GWT) | Topology-aware attention routing | GWT broadcasts salient info — Codebook selects optimal communication graph per query |
| **NT-ACT** (orchestration) | Amortized multi-agent coordination | Instead of optimizing topology per-task, use codebook lookup — 2.4ms vs iterative search |
| **NT-MIND** (evolution) | Topology evolution via codebook | SEAL stages could evolve codebook entries as new task patterns emerge |

### Implementation Sketch

```rust
// Codebook-based topology selection for GWT
struct TopologyCodebook {
    entries: [AdjacencyMatrix; 16],  // 16-entry codebook
    query_encoder: MLP,              // query → code distribution
    reranker: MLP,                   // code → utility+cost score
}

impl TopologyCodebook {
    fn select_topology(&self, query: &Query, agents: &[Agent]) -> Topology {
        // 1. Encode query to code distribution
        let codes = self.query_encoder.encode(query);

        // 2. Rerank top candidates by utility + token cost
        let candidates = codes.top_k(3);
        let ranked = self.reranker.rerank(candidates, query);

        // 3. Return best topology (2.4ms total)
        self.entries[ranked.best]
    }
}
```

### Risk Assessment

- **Low risk**: Codebook is pre-computed, no online training needed
- **Validation**: Test on multi-agent coordination tasks with varying complexity
- **Extension**: Could evolve codebook entries via SEAL pipeline feedback

---

## Paper 4: Bilevel Coordinated Reflection — Game-Theoretic Multi-Agent Systems

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2609.02750 (Sep 2, 2026) |
| **Title** | Bilevel Coordinated Reflection: A Game-Theoretic Approach to Multi-Agent LLM Systems |
| **Authors** | — |

### Core Idea

Models orchestrator-worker interaction as a bilevel coordination game. Workers' local-update game is an approximate potential game whose equilibrium slack is controlled by decomposition quality. Reflection is analyzed as stochastic movement over semantic memory states. Introduces SRMA (Stochastic Reflective Memory Ascent) — accepts candidate memory only after grounded evaluation risk strictly decreases. On 500 SWE-bench instances: 72.2% resolution vs 70.8% reference.

### Key Insight

Information-theoretic impossibility: no gate observing only generated transcript can improve uniformly over text-indistinguishable environments. Environment-grounded gate CAN improve — motivating grounded evaluation. SRMA convergence is geometric or polynomial, both order-tight.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-CORE** (GWT) | Bilevel attention coordination | GWT could model attention as bilevel game — orchestrator allocates, workers optimize locally |
| **NT-MIND** (SEAL) | SRMA for experience acceptance | experience-tree could use grounded evaluation before accepting new experiences — risk must decrease |
| **NT-SHIELD** (security) | Grounded evaluation gate | Memory updates should pass environment-grounded checks, not just transcript-based review |

### Implementation Sketch

```rust
// SRMA for experience-tree absorption
struct SRMAGate {
    risk_evaluator: GroundedEvaluator,
    calibration: CalibrationModel,
}

impl SRMAGate {
    fn should_absorb(&self, candidate: &Experience, context: &Context) -> bool {
        // 1. Grounded evaluation (not transcript-only)
        let risk_before = self.risk_evaluator.evaluate(context);
        let risk_after = self.risk_evaluator.evaluate(&context.with(candidate));

        // 2. Accept only if risk strictly decreases
        if risk_after >= risk_before {
            return false;  // reject — no improvement
        }

        // 3. Confidence gating for stochastic evaluation
        let confidence = self.calibration.confidence(risk_before, risk_after);
        confidence > CONFIDENCE_THRESHOLD
    }
}
```

### Risk Assessment

- **Low risk**: Additive to existing absorption pipeline — adds evaluation gate
- **Validation**: Test on 100+ sessions to verify risk reduction metric
- **Extension**: Could integrate with指针守恒 — structural invariants as risk factors

---

## Paper 5: Random Attention — Signal-Free KV Cache Eviction

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2609.03430 (Sep 3, 2026) |
| **Title** | Random Attention: Rethinking KV Cache Eviction for Efficient Reasoning |
| **Authors** | Huan Wang et al. |

### Core Idea

Minimalist signal-free eviction: (1) protect the entire prompt (pin in memory, never evict); (2) evict uniformly at random within each attention head for reasoning tokens. No scoring pass needed. Across 4 models and 6 reasoning tasks, matches strongest prior evictor while serving 32-43% higher throughput in vLLM. On H200: 1.6-2.7× full-attention throughput.

### Key Insight

The perceived success of complex eviction methods was largely due to how they implicitly handled the initial prompt. When all methods protect the prompt equally, accuracy gaps shrink dramatically. Reasoning traces protect themselves via redundancy: (1) the model restates what it still needs in text; (2) each attention head keeps its own copy. Random draw statistically preserves enough copies across heads.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-MEMORY** (KB) | Simplified cache eviction | KB cache could use random eviction for reasoning traces — simpler, faster, same quality |
| **NT-CORE** (reasoning) | Redundancy-based robustness | Self-reducing reasoning traces are inherently robust to partial eviction |
| **NT-PHYSICAL** (embodiment) | Zero-overhead eviction for edge | Edge devices can't afford scoring passes — random eviction is O(1) per token |

### Implementation Sketch

```rust
// Random Attention eviction for KB cache
struct RandomEvictor {
    prompt_pin: PromptPinning,  // never evict prompt
    head_rng: Vec<ThreadRng>,   // per-head RNG
}

impl RandomEvictor {
    fn evict(&mut self, cache: &mut KVCache, head: usize) {
        // 1. Prompt is pinned — skip
        let non_prompt_range = self.prompt_pin.non_prompt_range(cache);

        // 2. Uniform random eviction within head
        let victim_idx = self.head_rng[head].gen_range(non_prompt_range);
        cache.evict(head, victim_idx);
    }

    // No scoring pass — O(1) per eviction
    // Throughput: 32-43% higher than scored methods
}
```

### Risk Assessment

- **Low risk**: Zero-complexity eviction — can deploy immediately
- **Validation**: Benchmark on reasoning tasks with long chains of thought
- **Key insight for NeoTrix**: Prompt protection is critical; reasoning traces are self-protecting

---

## Cross-Paper Synthesis

### Unified Pattern: Error-Aware Adaptive Resource Allocation

All five papers converge on the same insight: **static allocation of attention, memory, and coordination resources is wasteful — adaptive, error-aware allocation achieves same quality at lower cost**.

```
┌─────────────────────────────────────────┐
│        Error Estimation (CEDAR/UNISON)   │
│  Approximation error + Session hazard    │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│     Adaptive Allocation (Codebook/SRMA) │
│  Topology codebook + Risk-gated memory  │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│     Simple Fallback (Random Attention)   │
│  When error estimation is expensive,    │
│  random + prompt protection suffices     │
└─────────────────────────────────────────┘
```

### NeoTrix Integration Priority

| Priority | Paper | Impact | Effort |
|----------|-------|--------|--------|
| **P0** | Random Attention | Simplify KB cache eviction — random + prompt pin | Very low — zero new code |
| **P0** | SRMA Gate | Grounded evaluation for experience absorption | Low — add evaluation step |
| **P1** | CEDAR | Error-bounded tiered retrieval for KB | Medium — need error estimation |
| **P1** | Codebook | Amortized topology selection for GWT | Medium — need codebook training |
| **P2** | UNISON | Session-aware KV scheduling | High — deep integration with cache |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Random Attention (simple) vs CEDAR (complex) | Use Random for routine tasks, CEDAR for precision-critical retrieval |
| UNISON (hardware) vs NeoTrix (software) | Implement SPEAR/TIDE logic in software first, hardware acceleration later |
| Codebook (pre-computed) vs SEAL (online evolution) | Codebook is static per deployment, SEAL evolves codebook entries periodically |
| SRMA (risk-averse) vs experience-tree (absorb aggressively) | SRMA for high-stakes experiences (production incidents), aggressive for routine learnings |
| Prompt protection (Random) vs context compaction (existing) | Pin prompt separately, compact only reasoning traces — complementary mechanisms |

---

## Action Items

1. **Immediate** (cycle 352): Implement Random Attention eviction for KB cache (random + prompt pin)
2. **Immediate** (cycle 352): Add SRMA grounded evaluation gate to experience-tree absorption
3. **Short-term** (cycle 353-355): Implement CEDAR error-bounded tiered retrieval for KB
4. **Medium-term** (cycle 356-360): Build Codebook topology selector for GWT attention routing
5. **Long-term** (cycle 360+): Design UNISON-inspired session-aware KV scheduling for edge deployment
