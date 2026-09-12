# Model Reverse Engineering — Cycle 417 (2026-09-12)

**Sources**: arXiv (Sep 2026), ACL 2026, Stanford/ICLR 2026
**Focus**: Efficient inference, attention mechanisms, agent coordination
**Mapping**: 7 NeoTrix domains

---

## Model 1: CEDAR — Error-Bounded Residual Routing for Long-Context Attention

**Paper**: [2609.07237] CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
**Authors**: Anonymous | **Date**: 2026-09-07
**URL**: https://arxiv.org/abs/2609.07237

### Core Innovation
Coarse-to-fine error-aware dynamic attention routing. Each semantic chunk contributes a cheap KV summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Refinement replaces rather than duplicates coarse evidence.

### Technical Details
- **Output-error bound**: Governed by within-chunk key/value dispersion, enables variable refinement budget per chunk
- **Residual summaries**: Reduce reconstruction error by >98% relative to hard dropping
- **Speedup**: ~3× kernel speedup at 128K context
- **Key insight**: Exact and summarized contributions combined in single softmax — refinement replaces coarse evidence, never duplicates

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Error-aware salience routing | Allocate attention budget proportional to approximation error |
| **NT-MEMORY** | Two-tier KV cache | Summary tier (cheap) → exact tier (expensive) based on error bound |
| **NT-WORLD** | Content triage | Cheap summaries for all, deep analysis only where error exceeds threshold |
| **NT-PHYSICAL** | Memory hierarchy | Residual summaries as hot tier, exact KV as cold tier |

### Actionable Insight
CEDAR's error-bound refinement budget is directly transferable to NT-MEMORY's KVMem: maintain cheap summaries for all memory blocks, expand to full KV only when estimation error exceeds threshold. This is a **formal cost-quality tradeoff** for memory tiers, replacing heuristic pruning with bounded error guarantees.

---

## Model 2: Declarative Attention — Model-Controlled Sparse Attention

**Paper**: [2609.02737] Language Models Can Control Their Own Attention
**Authors**: Anonymous | **Date**: 2026-09-02
**URL**: https://arxiv.org/abs/2609.02737

### Core Innovation
Declarative Attention (DA): model declares where it needs to attend via `<global>`, `<focus>`, and `<local>` tags in chain-of-thought. Inference engine parses declarations like tool calls, skips most KV cache reads. Intrinsic approach — model knows what's relevant.

### Technical Details
- **Three modes**: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only)
- **Token savings**: 52.0% on Gemma-4-31B, 31.1% on Qwen-3.6-27B
- **Accuracy drops**: 1.27pp, 2.75pp — shrink with model scale
- **Zero-shot**: Works on off-the-shelf models without training

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Self-directed attention | Model declares its own attention routing — intrinsic salience |
| **NT-MEMORY** | Selective memory recall | Model declares which memory tier to activate per query |
| **NT-WORLD** | Focus-based perception | `<focus>` mode for targeted content extraction |
| **NT-IO** | Token budget optimization | Reduce inference cost via model-declared attention regions |

### Actionable Insight
DA proves LLMs can self-declare attention needs at inference time. For NT-CORE's GWT: instead of external salience scoring, let the model declare `<focus>` regions during reasoning. This is **intrinsic attention routing** — the model becomes its own attention controller. Zero-shot capability means no training overhead.

---

## Model 3: AgentFlow — In-the-Flow Agentic Optimization (ICLR 2026 Oral)

**Paper**: AgentFlow: In-the-Flow Agentic System Optimization
**Authors**: Zhuofeng Li et al. (Stanford/Texas A&M/UCSD/Lambda) | **Venue**: ICLR 2026 Oral (Top 1.1%)
**URL**: https://agentflow.stanford.edu/

### Core Innovation
Trainable agentic framework coordinating Planner + Executor + Verifier + Generator through evolving memory. Flow-GRPO converts multi-turn optimization into tractable single-turn policy updates. Online RL inside the live multi-turn loop.

### Technical Details
- **4 modules**: Planner (πθ trainable), Executor, Verifier, Generator — coordinated by shared memory
- **Flow-GRPO**: Broadcasts single trajectory-level outcome to every turn, aligning local planner decisions with global success
- **Results**: 7B backbone outperforms GPT-4o on search (+14.9%), agentic (+14.0%), math (+14.5%), scientific (+4.1%) tasks
- **Key insight**: SFT causes catastrophic 19% collapse; online RL gives 17.2% improvement on same architecture

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Adaptive planner | Trajectory-level salience signals flow back to planner decisions |
| **NT-MIND** (SEAL) | Online self-evolution | Flow-GRPO for in-the-flow optimization — train inside production loop |
| **NT-ACT** | Modular execution | Planner/Executor/Verifier/Generator as composable capability modules |
| **NT-MEMORY** | Evolving memory | Shared memory state that evolves across multi-turn interactions |

### Actionable Insight
AgentFlow's Flow-GRPO solves the credit assignment problem for multi-turn agent optimization. For NT-MIND's SEAL pipeline: instead of offline distillation, run **online RL inside the live loop** — each turn broadcasts trajectory-level outcome to all prior turns. This eliminates the train-serve distribution shift that plagues offline methods. The 17.2% gain from online RL vs 19% collapse from SFT is a strong signal.

---

## Model 4: PARSER — Parallel Read, Deep Reasoning for Long-Context

**Paper**: [2609.06702] PARSER: Read in Parallel, Reason in Depth for Long-Context LLM Agents
**Authors**: Anonymous | **Date**: 2026-09-06
**URL**: https://arxiv.org/abs/2609.06702

### Core Innovation
Decouples reading from reasoning. Lightweight subagents each bound to one chunk read the document in parallel; lead agent reasons in depth through iterative scatter-gather rounds. Lead agent optimized via RL; subagents frozen.

### Technical Details
- **Architecture**: Lead agent + N frozen subagents (one per chunk)
- **Scatter-gather**: Lead broadcasts query → subagents return evidence → lead aggregates → deeper follow-up query
- **Results**: 4B backbone outperforms strongest sequential baseline by 5.7pp avg, 12.0pp at 896K tokens. 9B surpasses DeepSeek-V4-Pro by 6.3pp
- **Latency**: 11× reduction vs sequential methods
- **Robustness**: Immune to evidence position/order/distance perturbations

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Lead-subagent attention | Lead agent = GWT broadcaster, subagents = specialist receivers |
| **NT-WORLD** | Parallel perception | Each subagent = independent sensor reading one data chunk |
| **NT-MEMORY** | Parallel retrieval | Scatter-gather for multi-hop reasoning across memory shards |
| **NT-ACT** | Task decomposition | Lead agent orchestrates frozen specialist subagents |

### Actionable Insight
PARSER's architecture is a direct blueprint for NT-CORE's GWT: the lead agent is the global workspace broadcaster, frozen subagents are specialist modules. The scatter-gather pattern eliminates sequential evidence accumulation — critical for 896K+ token contexts. For NT-MEMORY: parallel chunk retrieval with iterative deepening replaces sequential scan. 11× latency reduction validates the parallel-read-deep-reason pattern.

---

## Model 5: CondenseFlow — Semantic Compression for Multi-Agent Latent Communication

**Paper**: CondenseFlow: Scalable Latent Space Collaboration via Semantic Compression
**Authors**: Anonymous | **Venue**: ACL 2026 Findings
**URL**: https://aclanthology.org/2026.findings-acl.669.pdf

### Core Innovation
Latent Thought Condenser (LTC) compresses variable-length KV caches into fixed-size representations using learnable semantic probes via cross-attention. O(1) communication complexity regardless of context length. End-to-end learnable compression (not heuristic pruning).

### Technical Details
- **Memory reduction**: >99% vs dense latent transfer
- **Latency reduction**: ~20% vs dense transfer
- **Accuracy**: <2% degradation vs dense, 1.7pp gain over text-based methods
- **Compression dimension**: K=64 (optimal based on effective rank analysis)
- **Regularization**: Coverage (uniform probe attention) + orthogonality (complementary probes)

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MEMORY** | KV cache compression | Fixed-size semantic anchors for cross-session memory transfer |
| **NT-CORE** (GWT) | Compressed broadcast | GWT broadcasts compressed semantic representations, not full state |
| **NT-ACT** | Agent-to-agent | O(1) inter-agent communication regardless of context length |
| **NT-PHYSICAL** | Memory efficiency | 99% KV cache reduction enables larger agent fleets on same hardware |

### Actionable Insight
CondenseFlow solves the linear memory scaling problem for multi-agent collaboration. For NT-MEMORY: replace full KV cache transfer between sessions with LTC-compressed semantic anchors. K=64 compression dimension with learned probes preserves task-relevant information while achieving 99% memory reduction. The learnable probes (vs heuristic pruning) are key — they discover which information patterns matter for downstream reasoning through end-to-end training. This is the missing piece for NeoTrix's agent fleet scaling.

---

## Cross-Model Synthesis (Cycle 417)

| Pattern | Models | NeoTrix Integration |
|---------|--------|---------------------|
| **Error-bounded attention routing** | CEDAR, Declarative Attention | NT-CORE GWT with formal cost-quality guarantees |
| **Parallel read, deep reason** | PARSER, CondenseFlow | NT-WORLD parallel perception + NT-MEMORY scatter-gather |
| **In-the-flow optimization** | AgentFlow Flow-GRPO | NT-MIND SEAL with online RL inside production loop |
| **Fixed-size semantic compression** | CondenseFlow, CEDAR summaries | NT-MEMORY two-tier KV: summaries (hot) → exact (cold) |
| **Self-declared attention** | Declarative Attention | NT-CORE intrinsic salience — model as own attention controller |

### Key Meta-Insight (Cycle 417)
The September 2026 papers converge on one theme: **attention is no longer just a neural mechanism — it's an engineering primitive**. CEDAR treats it as a routing problem with error bounds. Declarative Attention makes it a first-class protocol the model controls. AgentFlow optimizes it through in-the-flow RL. PARSER parallelizes it across subagents. CondenseFlow compresses it into fixed-size semantic anchors. For NeoTrix, this means GWT should evolve from a fixed attention router to an **adaptive, self-declared, error-bounded, compressible** attention system — combining all five patterns into a unified framework.
