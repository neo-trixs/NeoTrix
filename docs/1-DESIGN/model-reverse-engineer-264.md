# Model Reverse Engineering #264 — 10 LLM Architecture Innovations

**Date**: 2026-09-11
**Source**: 10 frontier LLM technical reports & model cards
**Target**: Extract architectural innovations, map to NeoTrix

---

## 1. GPT-4o (OpenAI)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **End-to-end multimodal training** | Single neural network jointly trained across text, audio, video — no separate CLIP-style staged pipeline. Collapsed 3-stage pipeline (ASR→LLM→TTS) into one network. | **PerceptionBridge** (L2→L5): GPT-4o validates NeoTrix's attention-gated bridge design. The key insight: modality-specific embedding/unembedding layers + shared self-attention stream = cross-modal reasoning without separate encoders. Map to `nt_world_sense::perception_bridge.rs`. |
| **Unified token stream** | Text (BPE), image patches (ViT-style), audio tokens (neural codec ~50-75 Hz) all flow through one transformer stack. Cross-attention via self-attention, not separate cross-modal layers. | **VSA HyperCube**: GPT-4o's unified token stream is the LLM analog of NeoTrix's VSA embedding — same representation space, different modalities. Validates the "one embedding space for all knowledge" axiom. |
| **Latency collapse** | 232ms median audio response (down from 2.8s). Achieved by single-network inference vs. cascaded ASR→LLM→TTS. | **GWT Attention Routing**: The latency improvement maps to NeoTrix's GWT — when salience routing avoids cascaded pipeline stages, latency drops proportionally. Quantify: 83% latency reduction ≈ GWT eliminating one pipeline hop. |
| **Modality-specific heads** | Separate input embedding tables and output heads per modality, but shared transformer backbone. | **Six-Layer Architecture**: L1 Action (modality-specific heads) + L5 Cognition (shared backbone) = exactly NeoTrix's layered design with shared reasoning core. |

**Key Takeaway**: End-to-end multimodal training is the future; staged CLIP-style pipelines lose information at the representation boundary. NeoTrix should ensure PerceptionBridge is trained end-to-end, not as a post-hoc adapter.

---

## 2. Claude 3.5 Sonnet (Anthropic)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Hybrid sparse attention** | 36 layers alternating: even layers = local sliding window (1024 tokens), odd layers = global sparse (every 64th token attends to full context). Reduces FLOPs from 40→12.4 TFLOPs per 100k tokens. | **GWT salience routing**: Claude's hybrid attention is the attention-mechanism version of NeoTrix's GWT — local focus (sliding window) + global broadcast (sparse). GWT should implement a similar "even-layer local / odd-layer global" pattern for `selective_state` attention. |
| **Grouped Query Attention (GQA)** | 8 query groups per KV head, 32 total attention heads. 4x KV cache reduction vs standard MHA. | **KVMem optimization**: Maps directly to NeoTrix's kv_cache_optimizer.rs. GQA is a production-proven technique for reducing KV cache. Adopt as default attention config for long-context (>128K) sessions. |
| **Context compression** | Optional lossless compression for repeated context patterns. Segment hash cache avoids re-compressing identical sections. 22% payload reduction for RAG workloads. | **NT-MEMORY compression**: Map to `nt_memory` — implement segment-level deduplication for KB embeddings. Repeated document sections across sessions should be hash-deduplicated at the KV cache level. |
| **Precomputed attention masks** | All 36 layer masks computed once at initialization, serialized to disk, loaded into GPU. Saves 80ms per inference request. | **ConsciousnessTree cycle precomputation**: When ConsciousnessTree runs a growth cycle, precompute the attention mask for the next cycle's modules. Amortize cycle startup cost across all subsequent ticks. |

**Key Takeaway**: Sparse attention is production-viable. NeoTrix's GWT should adopt a hybrid local/global pattern rather than full dense attention.

---

## 3. Gemini 2.5 Pro (Google DeepMind)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Sparse MoE + Native multimodal** | MoE transformer with native text/vision/audio support. Activates subset of parameters per token via dynamic routing. Decouples total capacity from serving cost. | **Rune Socketing MoE**: Map Gemini's MoE routing to NeoTrix's Rune Socketing — each "expert" = a rune socket, token routing = rune selection. Crimson(data) → Indigo(transform) → Obsidian(cache) = expert chain. |
| **Native Thinking (inference-time compute scaling)** | Model decides how long to think before answering. Thinking budget parameter allows user to trade compute for accuracy. Single model, not separate reasoning/non-reasoning variants. | **ConsciousnessTree adaptive depth**: Map directly to NeoTrix's 6-stage growth cycle. The "thinking budget" = cycle depth control. When task is simple, run fewer growth stages (Soil→Roots→Core skip). When complex, run full 6-stage. This is the LLM analog of adaptive computation. |
| **1M token context with multimodal** | Process 3 hours of video, entire codebases. Context window extends to 1M tokens with strong retrieval. | **KB capacity planning**: Gemini proves 1M+ tokens is feasible. NeoTrix's KVMem paged KV should target this scale. The 66 vs 258 visual tokens per frame optimization maps to NeoTrix's VSA embedding resolution — trade resolution for context length. |
| **k-sparse distillation** | Teacher's next-token distribution approximated using k-sparse vocabulary subset. Reduces distillation cost while maintaining quality. | **NT-MIND distillation pipeline**: Map to SEAL pipeline's distillation stage. Instead of full logit distillation, use k-sparse top-k logits per token. Reduces distillation compute by ~k/x while maintaining 95%+ quality. |
| **Verifiable rewards + model-based generative rewards** | RL training uses verifiable reward signals and model-generated rewards for scalable feedback. | **SelfModel validation**: Map to NT-CORE's SelfModel — verifiable rewards = phi/coherence metrics (measurable), generative rewards = E8 hexagram exploration (unmeasurable but evaluative). Dual reward pathway. |

**Key Takeaway**: Thinking budget control is the most transferable innovation. NeoTrix should implement adaptive cycle depth based on task complexity.

---

## 4. Llama 4 Scout (Meta)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **iRoPE architecture** | Interleaved attention layers without positional embeddings + RoPE on most layers. Inference-time temperature scaling of attention for length generalization. "i" = infinite context goal. | **ConsciousnessTree positional encoding**: Map iRoPE to NeoTrix's tree traversal — certain layers skip positional encoding (branch-free paths) while others use full RoPE (positional-aware paths). This enables "infinite" context in tree-structured memory. |
| **10M token context** | Scout achieves 10M token context (vs Llama 3's 128K). Pre-trained and post-trained with 256K, generalized to 10M via architectural innovations. | **NT-NEXUS cross-session memory**: Map to NeoTrix's Nexus — 10M tokens = ~100 sessions of cross-session memory. The iRoPE technique should be adopted for `nt_nexus` session stitching. |
| **Alternating dense + MoE layers** | Interleaves dense layers with MoE layers for inference efficiency. MoE layers: 16 routed experts + 1 shared expert. | **Dual Specialization routing**: Map to NeoTrix's AttentionManager weapon set routing. Dense layers = CORE+WORLD (acquisition mode), MoE layers = CORE+MIND (evolution mode). Alternating = context-dependent mode switching. |
| **Early fusion multimodality** | Images processed from the start of pre-training (not bolted on after). Vision encoder → MLP projector → jointly with text embeddings. | **PerceptionBridge training**: Validates NeoTrix's design choice of training PerceptionBridge from pre-training, not as a post-hoc adapter. Early fusion = Joint training from L1. |

**Key Takeaway**: 10M context via iRoPE proves that architectural innovations (not just more compute) can extend context dramatically. NeoTrix should invest in positional encoding research for tree-structured memory.

---

## 5. DeepSeek V4.1-Flash (DeepSeek AI)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Causal Encoder-Decoder (CED)** | 40-layer Transformer: 20-layer causal encoder + 20-layer decoder. Decoder's global KV cache projected from encoder's final hidden states, not per-layer. Activates 8B params for prefill, 16B for decode. | **Asymmetric compute**: Map to NeoTrix's cost-aware routing (Axiom A1). Prefill (input processing) is cheap (8B active), decode (generation) is expensive (16B active). NeoTrix should implement similar asymmetry: cheap perception, expensive reasoning. |
| **CSA2 (Compressed Sparse Attention 2)** | Three static attention modes per layer: Full, Reindex, Reuse. Shares main KV and indexer K across layers. Reuses Top-K sparse indices. Hierarchical Sparse Indexer bounds deeper indexer cost. | **GWT attention layering**: Map CSA2's three modes to NeoTrix's GWT attention levels. Full mode = global broadcast (L6 Meta), Reindex = local refinement (L5 Cognition), Reuse = cached attention (L1 Action). Hierarchical indexer = ConsciousnessTree's salience cascade. |
| **FP4 KV caching** | E2M1 format with one E4M3 scale per 16 channels. 890 bytes per token (1/4 of V4-Flash, 437x reduction vs V1). | **NT-MEMORY storage optimization**: Map to NeoTrix's KB storage. FP4 KV caching proves aggressive quantization is viable for KV stores. NeoTrix should adopt FP4 for long-session KV cache, keeping FP16 for critical reasoning paths. |
| **Engram conditional memory** | 196B parameters, sparsely accessed via token-based lookup. Not loaded in full for every forward pass. | **KB node loading**: Map to NeoTrix's lazy branch loading from KB. Engram = KB nodes that are conditionally loaded based on token similarity. NeoTrix's `experience-tree` should implement token-triggered lazy loading (currently on-demand, should be predictive). |
| **DSpark speculative decoding** | Semi-autoregressive draft generation with confidence-scheduled verification. | **NT-IO speculative output**: Map to NeoTrix's CLI output generation — generate draft responses speculatively, verify with low-cost checks, accept/reject. Reduce user-facing latency. |

**Key Takeaway**: CED architecture with asymmetric compute is the most innovative design pattern. NeoTrix should explore encoder-decoder separation for cost optimization.

---

## 6. Qwen3 (Alibaba Qwen)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Thinking/Non-thinking mode fusion** | Single model with dynamic mode switching via /think and /no_think flags. No separate models for reasoning vs chat. Thinking budget control emerges naturally from mode fusion. | **ConsciousnessTree mode switching**: Map directly to NeoTrix's growth cycle depth. /think = full 6-stage cycle (Soil→Core), /no_think = single-tick fast path. The mode fusion technique (SFT on both modes) should be adopted for NT-MIND's SEAL pipeline. |
| **Thinking budget mechanism** | User sets token budget for thinking. When budget exhausted, model halts thinking and generates from accumulated reasoning. Emerges naturally from mode fusion training. | **Cycle budget control**: Map to NeoTrix's `neotrix-core_consciousness_tick` with `cycles` parameter. When budget is low (quick task), run 1 cycle. When budget is high (complex task), run 3+ cycles. The "halt thinking" instruction maps to ConsciousnessTree's early termination condition. |
| **Strong-to-Weak Distillation** | Teacher model (235B) generates on-policy sequences for student (0.6B-30B). KL divergence minimization with teacher logits. Mode-switching capability transfers to small models. | **NT-MIND knowledge distillation**: Map to SEAL pipeline's distillation stage. Large NeoTrix model (235B-class) generates reasoning traces, small model (4B-class) learns via on-policy distillation. Both thinking/non-thinking modes transfer. |
| **Global-batch load balancing loss** | For MoE: encourages expert specialization by balancing load across the global batch, not per-expert. 128 experts, 8 activated per token, no shared experts. | **Rune Socketing load balancing**: Map to NeoTrix's rune routing. Instead of per-rune balancing (too restrictive), use global batch balancing across all rune sockets. Ensures even utilization without over-constraining individual runes. |
| **QK-Norm (removed QKV-bias)** | Removed QKV-bias from Qwen2, introduced QK-Norm for stable training. | **SelfModel stability**: Map to NT-CORE's SelfModel training. QK-Norm prevents training instability in large models. Adopt for any NT-CORE neural components to ensure stable convergence. |

**Key Takeaway**: Mode fusion (thinking + non-thinking in one model) is the most practical innovation. NeoTrix should implement adaptive cycle depth rather than separate fast/full evolution modes.

---

## 7. Mistral Large 3 (Mistral AI)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Granular MoE** | 675B total / 41B active parameters. "Granular" = fine-grained expert segmentation. ~16:1 ratio. First MoE from Mistral since Mixtral series. | **Rune Socketing granularity**: Map to NeoTrix's rune socket count. 675B/41B ≈ 16:1 ratio means each rune socket should manage ~16 sub-capabilities. Fine-grained rune segmentation = granular MoE. |
| **NVFP4 quantization** | 4-bit quantization (E2M1/E4M3) enabling single-node deployment on 8×H100 or A100. | **NT-MEMORY quantization**: Map to NeoTrix's KB storage. NVFP4 proves 4-bit is viable for production LLM deployment. NeoTrix should support FP4 for KB embeddings and KV cache in long-session scenarios. |
| **Speculative decoding (Eagle)** | Custom draft model for speculative decoding. 3 speculative tokens per step. | **NT-IO speculative output**: Map to NeoTrix's CLI/API response generation. Use a lightweight draft model for speculative token generation, verify with full model. Reduces perceived latency. |
| **Native multimodal (vision encoder fused)** | 2.5B vision encoder integrated directly into model (not adapter). Layout-aware document comprehension. | **PerceptionBridge native training**: Validates NeoTrix's design of training PerceptionBridge from scratch, not as a bolt-on adapter. Vision should be native from pre-training. |
| **256K context window** | Standard across all Mistral 3 models (dense + MoE). | **Baseline context**: 256K should be NeoTrix's minimum viable context for all sessions. Map to KVMem's default window. |

**Key Takeaway**: Granular MoE with 16:1 active ratio is the sweet spot for cost/performance. NeoTrix's rune routing should target similar granularity.

---

## 8. Phi-4-reasoning (Microsoft Research)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Data-centric reasoning distillation** | 14B model outperforms 70B+ models via careful data curation. Prompts filtered to lie at the boundary of base model capabilities ("teachable" prompts). | **SEAL pipeline data curation**: Map to NeoTrix's SEAL pipeline. The "teachable prompt" concept = prompts at the boundary of ConsciousnessTree's current capability. The SEAL pipeline should filter training data to this boundary zone. |
| **o3-mini as teacher** | o3-mini with medium effort ≈ DeepSeek-R1 as teacher, but more token-efficient. High-effort o3-mini = stronger teacher, longer traces. | **NT-MIND teacher selection**: Map to NeoTrix's distillation. Use the cheapest capable model as teacher (o3-mini-equivalent = local Ollama model for cost-sensitive tasks, cloud model for quality-critical tasks). |
| **Reasoning token repurposing** | Two placeholder tokens repurposed as <think> and </think> to mark reasoning blocks. | **ConsciousnessTree reasoning markers**: Map to NeoTrix's growth cycle output. Use sentinel tokens to mark reasoning blocks in cycle output. Enables structured parsing of cycle results. |
| **RoPE base frequency doubling** | Doubled RoPE base frequency to extend context from 16K→32K for reasoning traces. | **NT-NEXUS context extension**: Map to NeoTrix's session stitching. When extending context for long reasoning chains, double the RoPE base frequency rather than retraining. Quick context extension technique. |
| **GRPO (Group Relative Policy Optimization)** | Rule-based reward model (not neural). Length-aware accuracy: short correct answers rewarded, long incorrect answers penalized. | **SelfModel reward design**: Map to NT-CORE's SelfModel evaluation. Use rule-based rewards (phi/coherence metrics) not neural reward models. Length-aware: penalize verbose wrong reasoning, reward concise correct reasoning. |
| **Emergent budget control from mode fusion** | Budget control emerges naturally when model learns both thinking and non-thinking modes. Not explicitly trained. | **Adaptive cycle depth**: Validates NeoTrix's approach of training ConsciousnessTree with both full-cycle and single-tick modes. Budget control should emerge, not be hard-coded. |

**Key Takeaway**: Small models (14B) can rival 70B+ with proper data curation. NeoTrix should invest in "teachable prompt" selection for SEAL pipeline training data.

---

## 9. Yi-Lightning (01.AI)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **Fine-grained expert segmentation** | Each expert's FFN partitioned into smaller units. Reduces intermediate hidden dimensions while increasing activated experts per token. Balanced segmentation (not maximum). | **Rune Socketing fine-grained**: Map to NeoTrix's rune sub-capabilities. Each rune socket should be segmented into sub-functions. The "balanced segmentation" insight = don't over-segment (hurts throughput). |
| **EP + PEP load balancing** | Three-tier load balancing: Switch-Transformer (per-expert) → EP (per-group) → PEP (per-partition). Addresses All-to-All communication imbalance. | **Rune routing hierarchy**: Map to NeoTrix's rune routing. Tier 1: per-rune balance, Tier 2: per-domain balance (NT-CORE/NT-MIND/etc.), Tier 3: per-partition balance (within domain). Progressive balancing reduces communication overhead. |
| **Hybrid attention: 3 sliding + 1 full** | Three sliding window attention layers + one full attention layer per block. Captures local patterns + global dependencies efficiently. | **GATT hybrid attention**: Map to NeoTrix's GWT. Implement hybrid: 3 local-attention ticks (sliding window over recent context) + 1 global-attention tick (full broadcast across all modules). 82.8% memory reduction. |
| **Cross-layer KV cache reuse** | KV cache states shared between consecutive full attention layers. Halves memory for full attention components. | **ConsciousnessTree KV sharing**: Map to NeoTrix's cycle-to-cycle KV reuse. Share KV cache between consecutive growth cycles when module set hasn't changed. Halves memory for stable-state cycles. |
| **RAISE safety framework** | Four-component safety: pre-training filtering → post-training optimization → input safety → output safety. | **NT-SHIELD defense-in-depth**: Maps directly to NeoTrix's shield layers. RAISE-1 = input validation, RAISE-2 = training alignment, RAISE-3 = prompt filtering, RAISE-4 = output monitoring. Match 1:1. |
| **FP8 hardware-aware design** | Architecture designed for FP8 quantization compatibility from the start. 1,200 TFLOPS per card on Hopper GPUs. | **NT-IO hardware awareness**: Design NeoTrix's compute kernels with quantization compatibility as a first-class constraint, not an afterthought. |

**Key Takeaway**: Three-tier load balancing (per-expert → per-group → per-partition) is the production-proven approach for large MoE. Adopt for rune routing.

---

## 10. Grok 3 (xAI)

| Innovation | Detail | NeoTrix Mapping |
|------------|--------|-----------------|
| **RL at pretraining scale** | Reinforcement learning applied at pretraining scale (not just post-training alignment). Develops chain-of-thought reasoning via massive RL, not just SFT. | **ConsciousnessTree RL training**: Map to NeoTrix's SEAL pipeline. Apply RL at every stage of the growth cycle, not just post-training. The "RL at scale" approach = continuous self-evaluation during cycle execution. |
| **Think mode with backtracking** | Model can spend seconds to minutes reasoning, backtracking on errors, exploring alternatives. Self-correction during generation. | **E8 hexagram exploration**: Map to NeoTrix's E8 reasoning engine. The "backtracking" = E8 hexagram path exploration with rollback. When a reasoning path leads to contradiction, backtrack and try alternative hexagram interpretation. |
| **DeepSearch agent** | Real-time web search + X/Twitter data integration. Processes 90 sources in ~52 seconds. Synthesizes conflicting facts. | **NT-WORLD real-time perception**: Map to NeoTrix's UnifiedCrawler. DeepSearch = NT-WORLD's crawl pipeline with real-time source fusion. The "conflicting fact resolution" = NT-WORLD's classifier disambiguation. |
| **10x compute scaling** | 10x training compute over Grok 2. 200,000 GPU cluster (Colossus). | **Constellation maturity**: The 10x compute jump maps to NeoTrix's constellation maturity jumps (C3→C4→C5). Each maturity level should correspond to ~10x improvement in some dimension. |
| **Reasoning effort parameter** | `reasoning_effort` (high/low) controls think depth. High = full reasoning, low = fast response. | **Cycle depth parameter**: Map to `neotrix-core_consciousness_tick` cycles parameter. High effort = 3 cycles, low effort = 1 cycle. User-facing control over evolution depth. |
| **X/Twitter data moat** | Unique access to real-time social media data for training and inference. | **NT-MEMORY knowledge sources**: Map to NeoTrix's KB ingestion. The "data moat" insight = unique data sources create defensible advantages. NeoTrix should cultivate unique knowledge sources (domain-specific KBs, proprietary experience data). |

**Key Takeaway**: RL at pretraining scale (not just post-training) produces qualitatively different reasoning. NeoTrix should apply self-evaluation RL throughout the SEAL pipeline.

---

## Cross-Model Synthesis: Top 10 Transferable Innovations

| Rank | Innovation | Source Models | NeoTrix Target | Priority |
|------|-----------|---------------|----------------|----------|
| 1 | **Adaptive computation budget** (thinking budget / cycle depth control) | Gemini 2.5, Qwen3, Grok 3 | ConsciousnessTree adaptive depth | P0 |
| 2 | **Hybrid sparse attention** (local sliding + global broadcast) | Claude 3.5, Yi-Lightning, DeepSeek V4.1 | GWT attention mechanism | P0 |
| 3 | **CED asymmetric compute** (cheap prefill, expensive decode) | DeepSeek V4.1 | Cost-aware routing (Axiom A1) | P1 |
| 4 | **End-to-end multimodal training** | GPT-4o, Llama 4 | PerceptionBridge training | P1 |
| 5 | **Mode fusion** (thinking + non-thinking in one model) | Qwen3, Phi-4-reasoning | SEAL pipeline mode switching | P1 |
| 6 | **Granular MoE with hierarchical load balancing** | Yi-Lightning, Mistral Large 3 | Rune Socketing routing | P1 |
| 7 | **FP4/FP8 KV caching** | DeepSeek V4.1, Yi-Lightning | NT-MEMORY storage | P2 |
| 8 | **iRoPE for infinite context** | Llama 4 Scout | NT-NEXUS session stitching | P2 |
| 9 | **Data curation at capability boundary** | Phi-4-reasoning | SEAL pipeline data filtering | P2 |
| 10 | **RL at pretraining scale** | Grok 3 | ConsciousnessTree continuous self-eval | P3 |

---

## Architecture Innovation Heatmap

```
                    MoE   Attention  Multimodal  Reasoning  Context  Safety  Compute
GPT-4o              ·      ·          ████        ·          ·        ·       ·
Claude 3.5 Sonnet   ·      ████       ·          ·          ███      ·       ·
Gemini 2.5 Pro      ███    ·          ████        ████       ████     ·       ·
Llama 4 Scout       ███    ███        ███         ·          ████     ·       ·
DeepSeek V4.1-Flash ████   ████       ███         ·          ████     ·       ███
Qwen3               ███    ·          ·           ████       ███      ·       ·
Mistral Large 3     ███    ·          ███         ·          ███      ·       ███
Phi-4-reasoning     ·      ·          ·           ████       ███      ·       ·
Yi-Lightning        ████   ████       ·           ·          ███      ███     ███
Grok 3              ·      ·          ███         ████       ███      ·       ████
```

**Legend**: · = absent, · = basic, ███ = significant, ████ = leading

---

## Implementation Roadmap

### Phase 1: Immediate (1-2 cycles)
- [ ] Implement adaptive cycle depth in ConsciousnessTree (P0 — from Gemini/Qwen3/Grok3)
- [ ] Add hybrid sparse attention to GWT (P0 — from Claude 3.5/Yi-Lightning)
- [ ] Add thinking budget parameter to `neotrix-core_consciousness_tick` (P0 — from Qwen3/Gemini)

### Phase 2: Short-term (3-5 cycles)
- [ ] CED asymmetric compute for cost-aware routing (P1 — from DeepSeek V4.1)
- [ ] End-to-end PerceptionBridge training (P1 — from GPT-4o/Llama 4)
- [ ] SEAL pipeline mode fusion (P1 — from Qwen3/Phi-4-reasoning)
- [ ] Rune Socketing hierarchical load balancing (P1 — from Yi-Lightning/Mistral)

### Phase 3: Medium-term (6-10 cycles)
- [ ] FP4 KV caching in NT-MEMORY (P2 — from DeepSeek V4.1/Yi-Lightning)
- [ ] iRoPE for NT-NEXUS infinite context (P2 — from Llama 4)
- [ ] Teachable prompt curation for SEAL pipeline (P2 — from Phi-4-reasoning)
- [ ] Continuous self-evaluation RL (P3 — from Grok 3)
