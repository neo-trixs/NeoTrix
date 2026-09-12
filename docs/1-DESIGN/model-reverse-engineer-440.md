# Model Reverse Engineering — Cycle 440

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures
**Source**: arxiv (Sep 2026), ACL 2026 Findings

---

## 5 Papers/Models Analyzed

### 1. Declarative Attention (DA) — Self-Sparse Attention

**Paper**: arXiv:2609.02737 — "Language Models Can Control Their Own Attention" (Sep 2, 2026)

**Core Mechanism**:
- Models declare *where* to attend via chain-of-thought tokens: `<global>`, `<focus>`, `<local>` modes
- Inference engine parses declarations like tool calls, skipping most KV cache reads
- Zero-shot on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B)
- Reduces total attended tokens by 52% (Gemma-4) / 31% (Qwen-3.6) with modest accuracy drops (1.27pp, 2.75pp)

**Key Insight**: The model already knows which context is relevant — let it declare rather than scanning externally.

**NeoTrix Domain Mapping**:
| Domain | Application | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Declarative attention as self-modulated salience — E8 hexagram nodes declare focus regions |
| **NT-MEMORY** | KB query optimization | `<focus>` mode for targeted experience retrieval, `<local>` for session-local cache |
| **NT-IO** | Context window management | Skip KV cache for irrelevant provider responses |
| **Axiom A2** | Context as scarce resource | Direct implementation of context-efficient attention |

**Absorption Difficulty**: Medium — requires model-side training for optimal mode selection, but zero-shot works on existing models.

---

### 2. RecurTrace — Adaptive Latent Reasoning with Loop-Time Memory

**Paper**: arXiv:2609.03379 — "RecurTrace: Adaptive Latent Reasoning with Loop-Time Memory" (Sep 3, 2026)

**Core Mechanism**:
- Repeating middle layers increases effective inference depth without parameters/tokens
- **Loop Memory Attention**: Each looped layer attends to its own states from *previous iterations* along the loop-time axis
- **Halting Head**: Predicts whether to continue based on loop state, supervised by oracle
- 56.9% accuracy on MathQA with avg 2.0 loops, exceeding best fixed depth by 2.2 points
- Outperforms ACT, PonderNet, CALM at matched compute

**Key Insight**: Loop-time axis as a new dimension for attention — models can revisit earlier computations, not just the latest state. Adaptive halting wastes no compute on easy problems.

**NeoTrix Domain Mapping**:
| Domain | Application | Pattern |
|--------|-------------|---------|
| **NT-CORE** | E8 reasoning loops | Loop Memory Attention for hexagram state persistence across reasoning iterations |
| **NT-MIND** | SEAL pipeline adaptive depth | Halting head for self-evolution: stop refining when sufficient |
| **NT-FEEL** | Emotion trajectory tracking | Loop-time attention over emotion state history |
| **NT-META** | ConsciousnessTree cycle control | Adaptive cycle depth: skip phases when already converged |

**Absorption Difficulty**: Low — architectural pattern applicable to existing reasoning loops in NT-CORE.

---

### 3. CEDAR — Error-Bounded Residual Routing for Long-Context Attention

**Paper**: arXiv:2609.07237 — "CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention" (Sep 7, 2026)

**Core Mechanism**:
- Coarse-to-fine: each semantic chunk contributes cheap KV summary to residual attention path
- Chunks with high estimated approximation error are expanded to exact token attention
- Exact and combined in single softmax normalization — refinement replaces coarse evidence
- Output-error bound governed by within-chunk key/value dispersion
- 3x kernel speedup at 128K context, recovers most quality lost by hard sparse routing

**Key Insight**: Don't hard-select or hard-drop — use residual summaries as fallback, expand only when error budget is exceeded. Variable refinement budget per query.

**NeoTrix Domain Mapping**:
| Domain | Application | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT salience routing | Residual attention: broadcast coarse summaries, expand high-salience branches |
| **NT-MEMORY** | KB retrieval | Coarse summaries for all nodes, expand high-error-relevance nodes to full detail |
| **NT-WORLD** | Crawl pipeline | Chunk-level summarization with error-bounded expansion for content extraction |
| **NT-SHIELD** | Security scanning | Fast coarse scan → expand suspicious chunks for deep inspection |

**Absorption Difficulty**: Low — error-bounded routing is a drop-in enhancement for existing chunked retrieval.

---

### 4. PARSER — Parallel Subagents for Long-Context LLM Agents

**Paper**: arXiv:2609.06702 — "PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents" (Sep 6, 2026)

**Core Mechanism**:
- Decouples reading from reasoning: bank of lightweight subagents read chunks in parallel
- Lead agent reasons in depth through iterative scatter-gather rounds
- At each round: broadcast query → subagents return evidence → aggregate → formulate deeper query
- All learnable behavior in lead agent (RL-trained), subagents frozen off-the-shelf models
- 4B backbone outperforms sequential baselines by 5.7 points avg, 12.0 points at 896K tokens
- 9B backbone surpasses DeepSeek-V4-Pro by 6.3 points

**Key Insight**: Parallel subagents with frozen weights handle reading; one RL-trained lead handles reasoning depth. Scatter-gather rounds enable iterative refinement without sequential dependency.

**NeoTrix Domain Mapping**:
| Domain | Application | Pattern |
|--------|-------------|---------|
| **NT-ACT** | MCP tool orchestration | Subagent bank for parallel tool execution, lead agent for aggregation |
| **NT-WORLD** | Crawl pipeline | Parallel fetchers (frozen) → NT-CORE lead for content synthesis |
| **NT-MEMORY** | KB parallel retrieval | Subagents scan KB shards in parallel, ConsciousnessTree aggregates |
| **NT-IO** | Multi-provider routing | Frozen provider adapters → lead agent selects best response |

**Absorption Difficulty**: Low — scatter-gather pattern directly applicable to existing parallel execution infrastructure.

---

### 5. AERA — Adaptive Evidence Residual Allocation for Test-Time Scaling

**Paper**: arXiv:2608.27964 — "AERA: Adaptive Evidence Residual Allocation for Efficient Test-Time Reasoning" (Aug 28, 2026)

**Core Mechanism**:
- Sequential controller that learns whether additional computation will recover a better answer
- Uses checkpoint-observable evidence: answer-distribution, temporal, re-solving, semantic, compute features
- Key finding: correctness evolves **non-monotonically** — evidence may strengthen before collapse or weaken before recovery
- Estimate **future value of computation** rather than equating present confidence with correctness
- 92.61% accuracy vs 93.01% with 128 responses while reducing completion tokens by **96%**

**Key Insight**: Don't stop when confident — stop when evidence suggests further computation won't improve the answer. Non-monotonic correctness means current confidence is a poor stopping signal.

**NeoTrix Domain Mapping** | Domain | Application | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Reasoning loop termination | AERA controller for E8 reasoning: allocate depth based on expected improvement |
| **NT-MIND** | SEAL pipeline stopping | Don't over-refine: predict whether next iteration improves product |
| **NT-META** | ConsciousnessTree cycle control | Adaptive cycle count based on expected evolution value |
| **Axiom A1** | Cost-aware routing | Compute budget allocation per problem difficulty |

**Absorption Difficulty**: Medium — requires training the controller, but the feature set (answer-distribution + temporal + semantic) is extractable from existing outputs.

---

## Cross-Paper Synthesis

### Convergent Theme: "The Model Already Knows"
All five papers share a meta-insight: **the model has internal signals about its own state that are underutilized**.
- DA: model knows which context matters → let it declare
- RecurTrace: model knows when to stop looping → halting head
- CEDAR: model knows which chunks are approximated well → error-bounded expansion
- PARSER: model knows what to ask next → iterative scatter-gather
- AERA: model knows if more compute helps → future-value estimation

**NeoTrix implication**: NT-CORE's E8 reasoning engine should expose **meta-signals** (confidence trajectory, error bounds, focus declarations) that GWT uses for attention routing. Current architecture treats reasoning as opaque; these papers show it should be **self-aware and self-modulating**.

### Pattern: Residual Fallback > Hard Selection
CEDAR and DA both reject hard selection (pick/drop) in favor of residual/declarative fallbacks. The trend: **always maintain a degraded path, never zero-out information completely**.

**NeoTrix implication**: GWT should never fully suppress a module's signal — use residual summaries (CEDAR) or mode declarations (DA) instead of binary attention.

### Pattern: Frozen Readers + Trained Orchestrator
PARSER's separation of frozen subagents (reading) from RL-trained lead (reasoning) mirrors the emerging consensus: **specialize the coordinator, generalize the workers**.

**NeoTrix implication**: NT-ACT tool execution should use frozen adapters (MCP servers) with NT-CORE as the RL-trained orchestrator. Already partially implemented; formalize the separation.

### Pattern: Non-Monotonic Correctness
AERA's key finding — correctness doesn't monotonically improve with more compute — challenges confidence-based stopping. TRACE (ACL 2026) corroborates: single-step confidence is unreliable for multi-step reasoning.

**NeoTrix implication**: ConsciousnessTree cycle control must use **temporal aggregation** (TRACE's answer consistency + confidence trajectory) not instantaneous confidence. NT-FEEL's emotion state trajectory also exhibits non-monotonic patterns — same mechanism applies.

---

## Absorption Priority Matrix

| Paper | NeoTrix Target | Difficulty | Impact | Priority |
|-------|---------------|------------|--------|----------|
| Declarative Attention | GWT self-modulation | Medium | High — reduces context cost | P1 |
| RecurTrace | E8 loop-time memory | Low | High — adaptive reasoning depth | P1 |
| CEDAR | KB residual retrieval | Low | Medium — chunked query optimization | P1 |
| PARSER | Parallel subagent orchestration | Low | Medium — scatter-gather pattern | P2 |
| AERA | SEAL stopping criterion | Medium | High — 96% token reduction potential | P1 |

---

## Actionable Next Steps

1. **GWT + Declarative Attention**: Add `<global>/<focus>/<local>` mode declarations to E8 hexagram reasoning output. GWT uses these to skip KV cache for irrelevant branches.

2. **RecurTrace Loop Memory**: Add loop-time attention persistence to E8 reasoning cycles. Each iteration writes to a loop-time axis buffer; next iteration attends to prior states.

3. **CEDAR Residual Retrieval**: Replace hard chunk selection in KB retrieval with error-bounded residual summaries. Expand only when within-chunk dispersion exceeds threshold.

4. **TRACE Convergence Detection**: Implement sliding-window answer consistency + confidence trajectory for ConsciousnessTree cycle termination. Replace single-step confidence.

5. **PARSER Scatter-Gather**: Formalize NT-ACT parallel tool execution as frozen subagent bank + NT-CORE lead agent with iterative query refinement.
