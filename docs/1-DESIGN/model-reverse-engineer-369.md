# Model Reverse Engineering — Cycle 369

Date: 2026-09-12
Scope: Recent papers on efficient inference, attention, agent coordination

## 5 Models/Papers Analyzed

---

### 1. ReActNet: Inference-Time Graph Engineering for Multi-Agent LLM Workflows

**Paper**: [2609.05774] Inference-Time Graph Engineering for Multi-Agent LLM Workflows
**Date**: 2026-09-04 | **Venue**: arXiv

#### Core Innovation
ReActNet compiles a query and a set of role-specialized agents into a **sequence of directed communication graphs** — one per reasoning stage. Each edge carries a natural-language instruction specifying what message the source agent should provide to the target. The temporal graph is executed through structured message passing: agents update reasoning states by integrating previous states with messages from controller-assigned neighbors.

Key separation: **graph compilation** (offline, task-conditioned) from **graph execution** (online, message passing). No RL or gradient-based topology optimization needed.

#### Key Results
- Consistently improves over fixed-topology and learned-topology baselines
- Maintains competitive inference cost
- Works across knowledge reasoning, math, code generation, and GAIA assistant tasks

#### Reverse Engineering the Pattern
```
Query + Agent Set
  ↓ (compile)
Temporal Communication Graph [G1, G2, ..., Gn]
  Each Gi = (V, Ei) where edges carry NL instructions
  ↓ (execute)
Structured Message Passing per stage
  Agent state_i+1 = f(state_i, messages from neighbors)
  ↓ (aggregate)
Final Answer
```

The key insight: **multi-agent coordination is not about which agents communicate, but engineering executable workflow graphs that encode when, why, and how information flows during reasoning**.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | ReActNet's task-conditioned graph compilation ≈ GWT's salience-based routing. Both dynamically select which specialists receive broadcast. | GWT could adopt ReActNet's edge-as-instruction pattern: instead of generic "broadcast to all salient", specify "agent X should tell agent Y exactly Z" |
| **NT-MIND** (SEAL) | ReActNet's temporal graph as multi-stage reasoning mirrors SEAL pipeline's phased evolution. Each SEAL phase could compile its own communication graph. | SEAL pipeline stages (Soil→Roots→Trunk→Branches→Fruits→Core) could each define agent communication topology |
| **NT-ACT** (orchestration) | ReActNet's role-specialized agents map to NeoTrix's domain agents (NT-CORE, NT-MIND, etc.). Graph compilation = agent selection per reasoning stage. | CapabilityRegistry could use ReActNet-style temporal graphs for multi-step task orchestration |

#### Reuse Potential: HIGH
ReActNet is training-free and directly applicable. The "edge carries NL instruction" pattern could enhance GWT's broadcast mechanism: instead of sending generic salient signals, send task-specific instructions to selected agents.

---

### 2. AGAO: Adaptive Goal-aware Attention Orchestration

**Paper**: [2607.23678] Focus Is All You Need: Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems
**Date**: 2026-07-26 | **Venue**: arXiv

#### Core Innovation
AGAO introduces **Attention Engineering** — extending attention from token-level representation learning to workflow-level agent coordination. Three complementary mechanisms:

1. **Goal-aware Attention**: measures semantic relevance between user goals and agent capabilities
2. **Topology-aware Attention**: incorporates structural dependencies within agent graphs
3. **Resource-aware Attention**: dynamically allocates computational budgets and execution priorities

Key idea: treat agents as **dynamically selectable computational units** rather than fixed workflow operators. Attention scores drive execution decisions.

#### Key Results
- Improves task effectiveness while reducing unnecessary computation
- Reduces latency and token consumption vs. static graph execution
- Adaptive Graph Routing: attention distributions updated per execution feedback

#### Reverse Engineering the Pattern
```
User Goal
  ↓
Goal-aware Attention: score(agent_i, goal) → relevance_i
  ↓
Topology-aware Attention: score(agent_i, graph_deps) → structural_i
  ↓
Resource-aware Attention: score(agent_i, budget) → priority_i
  ↓
Combined: attention_i = f(relevance_i, structural_i, priority_i)
  ↓
Execution: route to top-k agents, skip low-attention nodes
  ↓
Feedback loop: update attention from execution results
```

This is **workflow-level sparse attention** — the same principle as token-level sparse attention (only compute on relevant tokens), applied to agent-level execution (only compute on relevant agents).

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | AGAO's three attention mechanisms map directly to GWT's salience scoring. Goal-aware = task relevance, topology-aware = graph structure, resource-aware = cost weight. | GWT salience could be decomposed into AGAO's three components: goal relevance + structural position + resource cost |
| **NT-MEMORY** (KB) | AGAO's feedback loop ≈ experience-tree's cycle feedback. Attention distributions updated from execution results = experience distilled into routing weights. | Experience-tree hub index could store AGAO-style attention distributions per task type |
| **NT-ACT** (orchestration) | AGAO's dynamic agent selection ≈ CapabilityRegistry's runtime routing. Skip low-attention nodes = Dark Forest (delete unreachable modules). | CapabilityRegistry could implement AGAO's three-score routing for task assignment |

#### Reuse Potential: HIGH
AGAO provides the theoretical foundation for GWT's attention mechanism. The three-score decomposition (goal + topology + resource) is directly implementable. The feedback loop aligns with experience-tree's cycle-based learning.

---

### 3. SpecBox: Speculative Sandbox Scheduling for Efficient LLM Agent Serving

**Paper**: [2607.23933] SpecBox: Speculative Sandbox Scheduling for Efficient LLM Agent Serving
**Date**: 2026-07-27 | **Venue**: arXiv

#### Core Innovation
SpecBox addresses the **cold-start penalty** for LLM agent sandbox execution. When agents invoke tools that require isolated sandboxes (MCP-style), persistent reservations waste memory while on-demand instantiation causes latency.

SpecBox introduces:
- **Intent-driven prewarming**: keyword matching + streaming semantic embedding to predict sandbox needs mid-token-generation
- **Context-aware stochastic prefetching**: probabilistic forecasting of future sandbox switches via dependency graph
- **Semantic result cache**: prunes redundant repeated sandbox invocations
- **Zero-copy shared-memory transport**: bypasses network serialization for artifact transfers

#### Key Results
- P99 end-to-end latency reduced by 2.9× vs. on-demand baseline
- Peak memory consumption reduced by 45.9% vs. permanent reservation
- Works on high-concurrency multi-turn agent traces

#### Reverse Engineering the Pattern
```
Agent token generation
  ↓ (simultaneously)
Intent Detection: keyword + streaming embedding → predict tool calls
  ↓
Sandbox Prewarming: boot predicted sandboxes during inference
  ↓
Execution: sandbox ready when tool call arrives (zero wait)
  ↓
Result Cache: skip re-execution for repeated identical calls
  ↓
Zero-Copy Transfer: artifacts via shared memory, not network
```

This is **speculative execution for agent infrastructure** — the same principle as CPU branch prediction, applied to sandbox lifecycle management.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** (tool execution) | SpecBox's intent-driven prewarming ≈ NeoTrix's tool preloading. Predict which tools will be needed, preload their sandboxes. | CapabilityRegistry could implement SpecBox-style speculative tool prewarming |
| **NT-SHIELD** (sandbox) | SpecBox's semantic result cache ≈ NT-SHIELD's sandbox caching. Reuse validated execution environments. | Sandbox manager could use SpecBox's intent detection for pre-allocation |
| **NT-IO** (provider) | SpecBox's zero-copy transport ≈ NT-IO's data transfer optimization. Shared-memory for inter-component artifact passing. | IO layer could adopt SpecBox's shared-memory transport for large artifact transfers |

#### Reuse Potential: MEDIUM-HIGH
SpecBox's intent-driven prewarming is applicable to NeoTrix's tool execution pipeline. The zero-copy transport pattern is particularly relevant for large file operations (PDF processing, image super-resolution). Semantic result cache could reduce redundant tool invocations.

---

### 4. PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents

**Paper**: [2609.06702] PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents
**Date**: 2026-09-06 | **Venue**: arXiv

#### Core Innovation
PARSER **decouples reading from reasoning**. A bank of lightweight subagents each bound to a single chunk read the entire document in parallel. A lead agent reasons in depth through iterative scatter-gather rounds:

1. Lead broadcasts query to all subagents
2. Subagents return evidence from their chunks
3. Lead aggregates evidence, formulates deeper follow-up query
4. Repeat until sufficient depth achieved

Lead agent is optimized with RL; subagents remain frozen off-the-shelf models.

#### Key Results
- 4B backbone outperforms strongest sequential baseline by 5.7 pts (avg), 12.0 pts at 896K tokens
- 9B backbone surpasses DeepSeek-V4-Pro by 6.3 pts
- Robust to evidence position, order, and distance perturbations
- Up to 11× latency reduction vs. sequential methods

#### Reverse Engineering the Pattern
```
Long Document (896K tokens)
  ↓ (split)
Chunks [C1, C2, ..., Cn]
  ↓ (parallel read)
Subagents [S1, S2, ..., Sn] — each reads one chunk
  ↓ (scatter-gather)
Round 1: Lead broadcasts query Q1 → Subagents return evidence E1
  ↓
Lead synthesizes: deeper query Q2 = f(Q1, E1)
  ↓
Round 2: Lead broadcasts Q2 → Subagents return evidence E2
  ↓
... (iterative deepening)
  ↓
Final Answer = lead's accumulated reasoning
```

Key insight: **reading is embarrassingly parallel; reasoning is inherently sequential**. Separating them enables both parallelism (fast reading) and depth (iterative reasoning).

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** (KB) | PARSER's scatter-gather ≈ KB embedding search. Parallel chunk reading = vector similarity search across KB namespaces. Lead agent = query reformulation. | KB search could adopt PARSER's iterative scatter-gather: query → evidence → deeper query → more evidence |
| **NT-CORE** (reasoning) | PARSER's lead agent with RL optimization ≈ GWT's salience computation. Both score which information is relevant to the current reasoning state. | GWT could use PARSER's iterative deepening: broadcast initial salience, gather evidence, refine salience scores |
| **NT-WORLD** (perception) | PARSER's parallel chunk reading ≈ NT-WORLD's parallel crawl/parse pipeline. Both split work across subagents for parallel processing. | NT-WORLD's crawl pipeline could adopt PARSER's scatter-gather for parallel document processing |

#### Reuse Potential: HIGH
PARSER's scatter-gather pattern is directly applicable to NeoTrix's KB search and NT-WORLD's crawl pipeline. The iterative deepening (query → evidence → deeper query) is a natural fit for GWT's multi-stage attention refinement.

---

### 5. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention

**Paper**: [2609.07237] CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
**Date**: 2026-09-07 | **Venue**: arXiv

#### Core Innovation
CEDAR addresses the fundamental flaw of hard sparse attention: when a chunk is dropped, its information is **completely lost**. CEDAR introduces a **residual attention path**:

1. Each semantic chunk contributes a cheap key-value summary to a residual path
2. Chunks with high estimated approximation error are expanded to exact token attention
3. Exact and summarized contributions are combined in a single softmax normalization
4. Refinement replaces, rather than duplicates, coarse evidence

Key insight: **soft selection with error bounds** beats **hard selection with fixed budgets**. The error bound governs refinement budget allocation.

#### Key Results
- Residual summaries reduce reconstruction error by 98%+ vs. hard dropping
- Recovers most quality lost by hard sparse routing
- Maintains ~3× kernel speedup at 128K context
- Output-error bound governed by within-chunk key/value dispersion

#### Reverse Engineering the Pattern
```
Query Q over context [C1, C2, ..., Cn]
  ↓
Stage 1 (Coarse): cheap KV summary for each chunk → approximate attention
  ↓
Error Estimation: compute approximation error per chunk
  ↓
Decision: if error_i > threshold → expand C_i to exact tokens
           else → keep summarized contribution
  ↓
Stage 2 (Refine): exact attention on high-error chunks
  ↓
Combine: single softmax over (exact + summarized) contributions
  ↓
Output = refined attention (no duplication)
```

This is **coarse-to-fine attention with provable error bounds** — the same principle as multigrid methods in numerical analysis, applied to neural attention.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | CEDAR's residual path ≈ GWT's background attention. Even when a module isn't "salient," it still contributes a summary. No module is completely silenced. | GWT could adopt CEDAR's residual path: non-salient modules contribute summaries, not silence |
| **NT-MEMORY** (KV) | CEDAR's error-bounded refinement ≈ KV cache tiering. Hot KV = exact tokens, cold KV = summaries. Error bound determines when to promote from cold to hot. | kv_cache_optimizer.rs could use CEDAR's error-bounded promotion: only promote KV pairs whose summary error exceeds threshold |
| **NT-MIND** (evolution) | CEDAR's coarse-to-fine refinement ≈ SEAL pipeline's phased distillation. Coarse summary first, then refine based on importance. | SEAL pipeline could adopt CEDAR's error-bounded refinement: summarize first, refine only high-error components |

#### Reuse Potential: HIGH
CEDAR's residual path is the most directly applicable pattern for NeoTrix. The "no module is completely silenced" principle aligns with GWT's broadcast mechanism: every module contributes a summary, salient modules get exact attention. The error-bounded refinement is applicable to KV cache management and experience-tree summarization.

---

## Cross-Paper Synthesis

### Emerging Pattern: Two-Phase Attention Across Scales

All five papers converge on a **two-phase attention** pattern:

| Phase | Token Level (CEDAR/SANTA) | Agent Level (ReActNet/AGAO) | System Level (PARSER) |
|-------|---------------------------|----------------------------|----------------------|
| **Coarse** | KV summary / residual path | Goal-relevance scoring | Parallel chunk reading |
| **Refine** | Exact token attention on high-error chunks | Topology + resource routing | Iterative scatter-gather |
| **Budget Control** | Error bound governs refinement | Attention score governs execution | Query depth governs iteration |

This is a **universal design pattern**: coarse screening → error/importance estimation → targeted refinement. It appears at every scale from individual attention heads to multi-agent systems.

### NeoTrix Integration Priority

| Pattern | Source | Target | Priority |
|---------|--------|--------|----------|
| Residual attention (no silence) | CEDAR | GWT salience scoring | P0 |
| Iterative scatter-gather | PARSER | KB search + crawl pipeline | P0 |
| Three-score agent routing | AGAO | GWT decomposition | P1 |
| Temporal communication graphs | ReActNet | SEAL pipeline stages | P1 |
| Speculative tool prewarming | SpecBox | CapabilityRegistry | P2 |
| Error-bounded KV promotion | CEDAR | kv_cache_optimizer.rs | P2 |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| ReActNet (compile graphs offline) vs. AGAO (route online with feedback) | Hybrid: compile initial graph offline (ReActNet), adapt routing online with feedback (AGAO) |
| CEDAR (residual summaries) vs. hard sparse (drop non-salient) | CEDAR wins for quality; hard sparse wins for speed. Use CEDAR for high-quality mode, hard sparse for speed-critical mode |
| PARSER (parallel subagents) vs. sequential memory agents | PARSER for parallel-friendly tasks (document QA); sequential for dependency-heavy tasks (code reasoning) |
| SpecBox (predictive prewarming) vs. on-demand (lazy instantiation) | SpecBox for high-concurrency production; on-demand for low-concurrency development |

## Priority Absorption Candidates

| Priority | Paper | Why |
|----------|-------|-----|
| P0 | **CEDAR** (residual attention) | "No module silenced" principle for GWT — every module contributes summary |
| P0 | **PARSER** (scatter-gather) | Iterative deepening for KB search and NT-WORLD crawl pipeline |
| P1 | **AGAO** (three-score routing) | Theoretical foundation for GWT's attention decomposition |
| P1 | **ReActNet** (temporal graphs) | Multi-stage communication topology for SEAL pipeline |
| P2 | **SpecBox** (speculative prewarming) | Tool execution optimization for CapabilityRegistry |
