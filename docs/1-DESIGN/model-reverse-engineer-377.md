# Model Reverse Engineering — Cycle 377

**Date:** 2026-09-12
**Focus:** Recent models/papers on efficient inference, attention, agent coordination

---

## 1. Muse Spark 1.3 (Meta) — Agentic Efficiency via Contemplating Mode

**Source:** Meta Superintelligence Labs, Sep 2 2026

### Key Claims
- ~20% fewer tool calls, ~25% fewer tokens than Muse Spark 1.2 for equivalent tasks
- 1M token context window, 943K max output
- "Contemplating" mode: parallel specialized sub-agents cross-check logic before committing
- Closed weights, $1.25/$4.25 per M tokens

### Architecture Insights
- **Contemplating Mode**: Runs multiple specialized sub-agents in parallel to verify logic, cross-reference code, simulate outcomes before final answer. Trades compute for quality in a structured way.
- **Efficiency vs. Capability Trade**: Meta deliberately cut tool calls/tokens in one part of the pipeline to fund deeper reasoning in another (Contemplating).
- **Long-horizon Training**: Trained on more long-horizon coding tasks → fewer wasted turns, cleaner style.
- **Multimodal Native**: Text + image + video input, visual chain-of-thought through real execution environment.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Contemplating Mode → GWT attention | Parallel sub-agents verify before broadcast. Maps to GWT salience modulation with multi-path evaluation |
| NT-ACT | Tool-call efficiency | 20% fewer tool calls via better planning. Aligns with NT-ACT orchestration optimization |
| NT-MIND | Self-awareness calibration | Model knows what it can't do, asks for help. Maps to SelfModel uncertainty awareness |
| NT-IO | Reasoning tiers | xhigh/max reasoning configs. Maps to Cost-Aware Routing (Axiom A1) |

### Absorption
- **Contemplating pattern**: Implement as a "VerifyBeforeCommit" GWT modifier — before broadcasting salient info, run parallel verification sub-agents
- **Efficiency framing**: Measure tool-call efficiency as a first-class metric in SEAL pipeline evolution tracking

---

## 2. K2 Horizon MoVA (IFM) — Sparse Attention via Mixture-of-Value-Attention

**Source:** Institute of Foundation Models, Sep 3 2026

### Key Claims
- 36B total params, 4B active per token (MoE + MoVA)
- Frontier-class results at 4B active parameters
- 512K native context
- Fully open: weights + training data + code + intermediate checkpoints

### Architecture Insights
- **MoVA (Mixture-of-Value Attention)**: Extends MoE sparsity from feed-forward layers INTO attention. Router activates subset of attention experts per token.
- **Dual Sparsity**: MoE in FFN + MoVA in attention = two independent sparsity axes for capacity scaling
- **Token-efficient tool presentation**: Markdown tool definitions are 18.5% more token-efficient than JSON
- **Development tree post-training**: Different branches specialize in reasoning/coding/tool-use/agentic while sharing base checkpoints
- **Uno inference**: Diffusion-based lossless inference speedup via LoRA adapter, no draft model needed

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | MoVA sparse attention | GWT attention routing with sparse expert selection — only activate relevant attention heads per task |
| NT-MIND | Development tree post-training | SEAL pipeline branching — different evolution branches for different specializations sharing common base |
| NT-WORLD | Token-efficient formats | Choose Markdown over JSON for tool definitions in GWT attention payloads |
| NT-CORE | Uno lossless speedup | KV-cache optimizer extension — diffusion-based parallel generation without quality loss |

### Absorption
- **MoVA pattern**: Implement sparse attention in HyperCube — not all attention heads need to fire for every query. Could reduce GWT routing overhead
- **Development tree**: Model SEAL pipeline evolution as a tree with shared base and specialized branches
- **Uno adapter**: Investigate for KV-cache optimizer — lossless speedup via diffusion parameters

---

## 3. Looped Flows — Recurrent Reasoning via Probability Flow

**Source:** arXiv:2609.11801, Sep 10 2026

### Key Claims
- 58.8% on ARC-AGI-1 (vs TRM 44.6%, GRAM 52.0%)
- 12.2% on ARC-AGI-2
- 5-7M params, trains with local denoising objectives (no full BPTT)
- Inference scales with finer temporal grid (more steps → better accuracy)

### Architecture Insights
- **Temporal Alignment**: Train recurrence with local denoising objectives at decreasing noise levels. Shared noise across timesteps creates implicit curriculum.
- **Stateful Denoiser**: Maintains recurrent state z_t that carries computation forward across flow steps
- **ODE/SDE Inference**: Deterministic (ODE) or stochastic (SDE) integration. SDE adds exploration for multi-solution problems.
- **Inference-time scaling**: Use finer grid than training (n ≥ k). Performance improves with more steps.
- **Best-Q ensembling**: Choose prediction with highest success probability from multiple samples.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Recurrent reasoning | E8 hexagram reasoning as a flow — each step refines the solution through recurrent state updates |
| NT-MIND | Temporal alignment training | SEAL pipeline stages with decreasing "noise" (uncertainty) — early stages explore, later stages exploit |
| NT-CORE | Adaptive inference scaling | ConsciousnessTree adjusts computation depth based on task difficulty (like Loop-ViT's Dynamic Exit) |
| NT-MEMORY | Multi-solution reasoning | SDE exploration for generating multiple valid solution paths from same problem |

### Absorption
- **Looped reasoning pattern**: E8 reasoning engine could use flow-based recurrence — each hexagram iteration refines the solution state
- **Temporal alignment**: Map to SEAL pipeline — early phases (Soil/Roots) are high-noise exploration, later phases (Fruits/Core) are low-noise exploitation
- **Adaptive halting**: ConsciousnessTree cycle could use entropy-based halting — stop when coherence stabilizes

---

## 4. Gated-Memory Routing (EMNLP 2026) — Adaptive Multi-Agent Orchestration

**Source:** arXiv:2609.00237, Aug 31 2026

### Key Claims
- +2.44 points best average accuracy across 5 benchmarks
- 31.9% inference cost reduction vs strongest baseline
- Adaptive Halting Controller stops when memory has sufficient evidence
- Accepted EMNLP 2026 Main Conference

### Architecture Insights
- **Memory Write Gate**: Commits only non-redundant reasoning steps to memory (deduplication at write time)
- **Retrieval Gate**: Supplies each agent a compact, relevant subset (not full history)
- **Adaptive Halting**: Stops execution once memory contains sufficient evidence for answering
- **Execution-history overload**: Complete history inflates cost; compact memory state is key
- **Two-gate design**: Write gate (what to remember) + Read gate (what to retrieve) = clean information flow

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Write Gate | KB write filtering — only non-redundant experience enters knowledge base |
| NT-MEMORY | Retrieval Gate | KB query filtering — compact, relevant subset per query |
| NT-CORE | Adaptive Halting | ConsciousnessTree stops cycle when sufficient evidence accumulated |
| NT-ACT | Multi-agent routing | Route tasks based on memory state, not just query |
| NT-MIND | Execution-history compression | Experience-tree distillation — compress raw trajectories into compact summaries |

### Absorption
- **Two-gate memory**: Implement Write Gate + Retrieval Gate in KB pipeline. Write Gate = experience-tree distillation filter. Retrieval Gate = GWT attention filtering.
- **Adaptive halting**: Map to SEAL pipeline — stop evolution cycle when phi/coherence reaches threshold
- **Memory-state routing**: Route tasks based on accumulated memory state, not just current query

---

## 5. ExoMind — Extended-Mind Scientific Reasoning Agent

**Source:** HuggingFace AI4SGI, Aug 28 2026

### Key Claims
- 8-benchmark average: 68.3 (vs Claude-Opus-4.8 54.2, GPT-5.5 53.0)
- First #1 on 6 out of 8 scientific benchmarks
- 262K context, fine-tuned from Qwen3.5-35B-A3B
- Training-value-aware data engineering + Progressive Chain-of-Interaction training

### Architecture Insights
- **Extended Mind Architecture**: LLM + interaction objects + autonomous interaction processes as ONE system (not separate components)
- **Chain-of-Interaction (CoI)**: Source discovery → evidence grounding → verification → observation integration as composable objects
- **Training-Value-Aware Data**: Identifies challenging, learnable problems → routes to pure-reasoning or interaction-reasoning data
- **Progressive CoI Training**: Jointly develops intrinsic reasoning and autonomous interaction using few thousand high-quality trajectories
- **Efficient frontier**: 1-2 days on 8×H200 for full-parameter SFT

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Extended Mind | Unified system (not separate modules) — E8+GWT+HyperCube as one cognitive system |
| NT-MIND | Training-value-aware data | SEAL pipeline data selection — route tasks to appropriate reasoning mode |
| NT-WORLD | Composable interaction objects | Perception modules as composable objects (source discovery, evidence grounding) |
| NT-ACT | Progressive training | Capability progression C0→C6 with staged training |
| NT-MEMORY | Chain-of-Interaction | KB query chain: discover → ground → verify → integrate |

### Absorption
- **Extended Mind pattern**: Reinforce NeoTrix's integrated architecture — modules communicate as one system, not isolated components
- **Training-value-aware routing**: SEAL pipeline should assess task difficulty before choosing reasoning strategy
- **Composable interaction objects**: Model NT-WORLD perception as composable objects (fetch→parse→classify→extract)

---

## Cross-Cutting Patterns

### 1. Adaptive Computation (all 5 models)
All models implement some form of "spend more compute on harder problems":
- Muse Spark: Contemplating mode (parallel verification)
- K2 Horizon: MoVA (sparse attention — less compute for easy tokens)
- Looped Flows: Finer temporal grid for harder problems
- Gated-Memory: Adaptive halting when sufficient evidence
- ExoMind: Training-value-aware routing

**NeoTrix**: ConsciousnessTree should implement adaptive computation depth — easy tasks get shallow cycles, hard tasks get deep cycles.

### 2. Write-Time Filtering (Gated-Memory + ExoMind + Muse Spark)
Both Gated-Memory and ExoMind emphasize filtering at write time:
- Gated-Memory: Write Gate commits only non-redundant steps
- ExoMind: Training-value-aware data selection
- Muse Spark: Fewer tool calls via better planning

**NeoTrix**: Experience-tree absorption should filter at write time — only non-redundant, high-value experience enters KB.

### 3. Memory as Active Participant (Gated-Memory + ExoMind + Looped Flows)
Memory is not passive storage — it actively shapes reasoning:
- Gated-Memory: Memory state routes next decision
- ExoMind: Interaction objects are part of the reasoning system
- Looped Flows: Recurrent state carries computation forward

**NeoTrix**: KB should be an active participant in GWT attention — not just queried, but actively modulates salience.
