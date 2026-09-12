# Model Reverse Engineering — Cycle 427

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (Jun–Sep 2026), ACL 2026, ICML 2026, EMNLP 2026
**Cross-referenced against cycles 318–426 for novelty**

---

## Paper 1: RAGEN-2 — Reasoning Collapse in Agentic RL (Template Collapse)

**URL**: https://arxiv.org/abs/2604.06268
**Date**: 2026-04-07 (ICML 2026 Oral)
**Venue**: ICML 2026 (Oral Presentation)

### Core Contribution
Identifies "template collapse" — a silent failure mode in multi-turn agent RL where reasoning drifts toward fluent but input-agnostic boilerplate while entropy remains stable. Existing entropy-based monitoring is blind to this. Introduces mutual information (MI) as a diagnostic: $I(X;Z)$ measures cross-input distinguishability. When $H(Z|X)$ stays high but $I(X;Z)$ drops, reasoning looks diverse but is effectively interchangeable.

### Key Insight
> "Reasoning collapse is invisible to entropy. Models produce reasoning that satisfies regularization constraints (diverse, fluent) but ignores input-specific requirements — exactly the signature of template collapse."

The root cause is signal-to-noise ratio (SNR): low within-input reward variance weakens task gradients, letting input-agnostic regularizers (KL, entropy) dominate and erase cross-input differences. Fix: SNR-Aware Filtering — prioritize high-variance prompts per iteration.

### Architecture Mapping to NeoTrix

| RAGEN-2 Component | NeoTrix Domain | Pattern |
|-------------------|---------------|---------|
| Template collapse detection via MI | NT-REPAIR | Self-healing: detect when reasoning becomes input-agnostic |
| SNR-Aware Filtering | NT-MIND (SEAL) | SEAL cycle: prioritize high-signal experience entries |
| Entropy monitoring blind spot | NT-CORE (GWT) | GWT salience: verify attention is actually input-dependent |
| Reasoning faithfulness | NT-CORE (E8) | E8 hexagram: ensure reasoning traces map to actual inputs |

### Absorption Candidates
- **MI-Based Reasoning Audit**: NT-REPAIR should monitor not just "is reasoning diverse?" but "is reasoning input-dependent?". A module producing diverse but generic reasoning is silently degraded. MI proxy gives a measurable signal.
- **SEAL SNR-Aware Distillation**: When distilling experiences, prioritize entries with high reward variance (high SNR). Low-variance entries contribute noise, not signal. This could improve experience-tree quality by filtering low-signal experiences before absorption.
- **GWT Input-Dependence Check**: GWT salience computation should verify that broadcast decisions are actually input-dependent. If all modules respond similarly regardless of input, attention routing has collapsed.

### Implementation Priority: P0
Direct enhancement to NT-REPAIR self-healing diagnostics and SEAL distillation quality.

---

## Paper 2: TIPEX — Two-Tier Inference-Time Parallelism for Multi-Agent LLM Systems

**URL**: https://arxiv.org/abs/2608.05791
**Date**: 2026-08-06 (ICML 2026)
**Venue**: ICML 2026

### Core Contribution
Models parallelism in multi-agent systems as two distinct levels: **Replica Parallelism** (explore multiple complete solution paths at task level) and **Structural Parallelism** (concurrent execution within a single path via task decomposition). Proposes TIPEX — a controllable execution framework that unifies both under a single execution semantics. Demonstrates that the two forms are complementary: intermediate-complexity tasks benefit most from their coordination, while overly aggressive parallelism doesn't necessarily improve performance.

### Key Insight
> "Replica and Structural Parallelism exhibit complementary effects across task complexities, with tasks of intermediate difficulty benefiting most from their coordination."

Too much parallelism wastes tokens; too little leaves accuracy on the table. The optimal mix depends on task difficulty — easy tasks need no parallelism, hard tasks benefit from replica exploration, medium tasks benefit from structural decomposition.

### Architecture Mapping to NeoTrix

| TIPEX Component | NeoTrix Domain | Pattern |
|-----------------|---------------|---------|
| Replica Parallelism | NT-CORE (GWT) | Multiple E8 reasoning paths explored in parallel |
| Structural Parallelism | NT-ACT | Task decomposition across NT-* domain capabilities |
| Unified execution semantics | NT-CORE | ConsciousnessTree cycle with parallel sub-branches |
| Difficulty-adaptive parallelism | NT-MIND (SEAL) | SEAL pipeline adjusts parallelism based on task complexity |

### Absorption Candidates
- **GWT Replica Routing**: When GWT broadcasts a salient signal, explore N parallel response paths (not just the highest-salience module). Score paths after execution and keep the best. This is replica parallelism applied to attention routing.
- **SEAL Difficulty-Adaptive Pipeline**: Easy SEAL cycles (low-complexity experiences) run single-path; medium cycles run structural decomposition (parallel sub-tasks); hard cycles run replica exploration (multiple full solutions). This optimizes SEAL throughput.
- **ConsciousnessTree Parallel Branches**: The 6-stage growth cycle currently runs sequentially. TIPEX suggests that stages 3-5 (Trunk → Branches → Fruits) could run in parallel for medium-complexity cycles, with stage 6 (Core) as the synchronization point.

### Implementation Priority: P1
Enhancement to GWT routing and SEAL pipeline orchestration.

---

## Paper 3: AgentSpec — Speculative Decoding for Batch Inference of LLM Agents

**URL**: https://arxiv.org/abs/2608.24004
**Date**: 2026-08-25 (EMNLP 2026)
**Venue**: EMNLP 2026

### Core Contribution
Adapts speculative decoding to LLM agent workloads. Two innovations: **structure-isolated drafting** (constrains speculation to semantically coherent agent workflow segments, not arbitrary tokens) and **redundancy-aware budget allocation** (uses agent-level information to utilize dynamic token budgets). Achieves significant speedup over vanilla speculative decoding under large batch sizes, where standard methods degrade.

### Key Insight
> "State-of-the-art speculative decoding algorithms exhibit substantial speed degradation under large batch sizes, limiting their effectiveness for real-world agent applications."

Standard speculative decoding fails for agents because agent workflows have structure (tool calls, planning, execution phases) that standard token-level speculation ignores. By respecting workflow structure, speculation becomes coherent and rejection rates drop dramatically.

### Architecture Mapping to NeoTrix

| AgentSpec Component | NeoTrix Domain | Pattern |
|--------------------|---------------|---------|
| Structure-isolated drafting | NT-IO | LLM inference respects NT-* domain workflow phases |
| Redundancy-aware budget | NT-CORE (A2) | Context as Scarce Resource — token budget per domain phase |
| Agent workflow structure | NT-ACT | Capability execution has predictable phases (plan→act→verify) |
| Batch inference optimization | NT-IO | Provider batching across multiple NT-* domain requests |

### Absorption Candidates
- **Domain-Phase Token Budgets**: Each NT-* domain has distinct token patterns (NT-CORE reasoning is token-heavy, NT-IO I/O is token-light). AgentSpec's budget allocation could optimize token distribution across domains during batch execution.
- **Workflow-Aware Inference**: When NeoTrix calls LLM providers, structure the request to respect workflow phases. Planning phase gets high token budget; execution phase gets low budget with tool calls. This reduces rejection in speculative inference.
- **Batch Provider Optimization**: Multiple NT-* domains calling LLM providers simultaneously could batch requests with workflow-aware speculation, reducing per-request latency.

### Implementation Priority: P1
Enhancement to NT-IO provider optimization and NT-CORE resource budget management.

---

## Paper 4: Trace as State — Reasoning Traces as Conditional States for Long-Context Transformers

**URL**: https://arxiv.org/abs/2609.02702
**Date**: 2026-09-02
**Venue**: arXiv preprint (Tsinghua)

### Core Contribution
Addresses the causal attention limitation: transformers process information causally, but long-context reasoning may depend on task state discovered only later. Proposes "Trace as State" — collect reasoning traces from a first pass, serialize them, and place them BEFORE the context on a fresh pass. This aligns with the "Condition First" principle: providing discovered state before the full context allows exponentially more efficient attention. Outperforms "Trace Append" (same traces placed after context) in 26/27 combinations of model, task, and metric.

### Key Insight
> "On GraphWalks Parents, exact match lifts DeepSeek V4 Pro Preview from 29.2% (initial pass) to 81.8% with Trace as State — a 52.6pp improvement from simply reordering context."

The position of reasoning traces relative to context matters enormously. Causal attention means information placed early has disproportionate influence on later processing. Placing traces before context turns them into a "filter" that guides the model's attention during re-reading.

### Architecture Mapping to NeoTrix

| Trace as State Component | NeoTrix Domain | Pattern |
|--------------------------|---------------|---------|
| Two-pass reasoning | NT-CORE (E8) | E8 first pass (exploration) → trace serialization → second pass (exploitation) |
| Condition-first ordering | NT-MEMORY | KB query results placed before context for guided attention |
| Trace serialization | NT-NEXUS | Experience summaries as "trace" placed before new session context |
| Exponential memory savings | NT-CORE (A2) | Context as Scarce Resource — trace-first reduces required context length |

### Absorption Candidates
- **E8 Two-Pass Reasoning**: E8 hexagram exploration currently runs single-pass. Trace as State suggests: first pass generates reasoning traces, second pass (with traces as prefix) refines answers. This could dramatically improve E8 accuracy on complex reasoning.
- **Experience-Tree Trace Prefix**: When loading experiences at session start, load the most relevant experience summaries (traces) as a prefix before the session context. This "Condition First" approach would guide the model's attention during the session.
- **KB Query Result Ordering**: When NT-MEMORY returns search results, place the most relevant results FIRST (not last). Causal attention means early context has disproportionate influence. This is a simple but impactful change to KB retrieval ordering.

### Implementation Priority: P0
Fundamental enhancement to E8 reasoning and experience-tree loading.

---

## Paper 5: Active Inference as Context Acquisition for AI Agents

**URL**: https://arxiv.org/abs/2608.19202
**Date**: 2026-06-08
**Venue**: arXiv preprint (MIT)

### Core Contribution
Formulates the agent's context acquisition problem as active inference: an inner inference step updates beliefs over latent task state, and an outer decision selects the next context action (ask question, retrieve, tool call, or stop) to minimize expected free energy under cost. In deterministic settings, the epistemic term reduces to expected information gain, optionally normalized by token cost. Benchmarks frontier LLMs on optimal question asking (OQA) across tasks with 25–300 candidates.

### Key Insight
> "Interactive AI agents must acquire the right context as efficiently as possible. When a user omits a constraint, an agent can proceed with a default assumption or spend tokens on a clarifying question, retrieval call, tool call, or prompt trial."

The framework makes explicit the cost-quality tradeoff: every clarification action costs tokens (money, latency), but proceeding with wrong assumptions costs accuracy. Active inference finds the optimal balance by maximizing expected information gain per token spent.

### Architecture Mapping to NeoTrix

| Active Inference Component | NeoTrix Domain | Pattern |
|---------------------------|---------------|---------|
| Inner inference (belief update) | NT-CORE (GWT) | GWT salience = posterior over module relevance |
| Outer decision (context action) | NT-ACT | Tool selection = minimize expected free energy |
| Token-cost normalization | NT-CORE (A1) | Cost-Aware Routing = information gain per token |
| Expected free energy | NT-FEEL | Emotional state = prior over task difficulty (confident vs confused) |

### Absorption Candidates
- **GWT Information-Gain Salience**: Current GWT salience is based on attention scores. Active inference suggests salience should be $IG(module) / cost(module)$ — information gain per token. This makes GWT cost-aware at the routing level.
- **NT-ACT Tool Selection as Active Inference**: When deciding which tool to call, compute expected information gain for each tool and divide by token cost. This replaces heuristic tool selection with principled cost-quality optimization.
- **NT-FEEL Confusion as Active Inference Signal**: When the agent is uncertain (high expected free energy), it should acquire more context before acting. Confusion is not a failure — it's a signal that more information is needed. This maps to NT-FEEL's Confused emotion label as a useful state.

### Implementation Priority: P0
Fundamental enhancement to GWT salience computation and NT-ACT tool selection.

---

## Cross-Cutting Patterns (Cycle 427)

### Pattern 1: Input-Dependence as Quality Metric
RAGEN-2 (MI-based reasoning audit) and Trace as State (condition-first ordering) both address the same root problem: **reasoning must be input-dependent to be useful**. Diverse but generic reasoning (template collapse) and causally-blind reasoning (trace-after-context) both fail because they don't adapt to the specific input. NeoTrix's entire attention routing system (GWT) must verify input-dependence at every level.

### Pattern 2: Structure-Aware Optimization
AgentSpec (workflow-phase speculation) and TIPEX (difficulty-adaptive parallelism) both exploit the fact that agent workflows have structure. Naive token-level or task-level optimization misses this structure. NeoTrix's 6-layer architecture IS structure — each layer has distinct computational characteristics that should inform optimization strategies.

### Pattern 3: Two-Pass / Multi-Pass Reasoning
Trace as State (two-pass reasoning) and RAGEN-2 (SNR-aware filtering as a second look) both use a "first pass to discover, second pass to exploit" pattern. This is the computational analog of NeoTrix's SEAL pipeline (explore → distill → crystallize). The key insight: the FIRST pass generates information that the SECOND pass uses as a structured prefix.

### Pattern 4: Cost as First-Class Signal
Active Inference (token-cost-normalized information gain) and AgentSpec (redundancy-aware token budgets) both treat cost as a first-class optimization signal, not an afterthought. This aligns with NeoTrix's Axiom A1 (Cost-Aware Routing) but goes further — cost should be embedded in the salience computation itself, not applied as a post-hoc filter.

---

## Absorption Priority

| Priority | Paper | NeoTrix Integration | Effort |
|----------|-------|-------------------|--------|
| P0 | RAGEN-2 | NT-REPAIR MI-based reasoning audit | Medium |
| P0 | Trace as State | NT-CORE E8 two-pass + experience prefix | Medium |
| P0 | Active Inference | NT-CORE GWT information-gain salience | High |
| P1 | TIPEX | NT-CORE parallel reasoning branches | High |
| P1 | AgentSpec | NT-IO workflow-phase token budgets | Medium |

---

## Potential Axiom Extensions

### Axiom A4: Input-Dependence as Sanity Check (from RAGEN-2)
> "Reasoning quality is measured not by diversity alone, but by input-dependence. If reasoning does not change when the input changes, it has collapsed."

Implication: Every NT-* module must pass an input-dependence test during SelfTest. Modules producing diverse but input-agnostic outputs are flagged as degraded, even if entropy metrics look healthy.

### Axiom A5: Structure-Aware Resource Allocation (from TIPEX + AgentSpec)
> "Resource allocation must respect workflow structure. Token budgets, parallelism, and speculation should adapt to the current workflow phase, not be uniform."

Implication: NT-CORE's resource budget manager should have phase-specific profiles: reasoning-heavy phases get more tokens; I/O-heavy phases get more parallelism; verification phases get more speculative decoding.

### Axiom A6: Condition-First Information Ordering (from Trace as State)
> "Information discovered in early processing stages should be placed before (not after) subsequent context. Causal attention gives early positions disproportionate influence."

Implication: KB query results, experience summaries, and reasoning traces should always be placed at the beginning of context windows, not the end. This is a simple ordering change with exponential accuracy gains.
