# Model Architecture Reverse-Engineering #297

> 10 frontier models — architecture, innovations, NeoTrix mapping
> Date: 2026-09-11 | Batch: 297

---

## Executive Summary

| # | Model | Params (Total/Active) | Architecture | Key Innovation | NeoTrix Pattern |
|---|-------|----------------------|-------------|----------------|-----------------|
| 1 | GPT-4o | Undisclosed | Dense Transformer + Diffusion Head | Omni-modal end-to-end, AR+diffusion hybrid | PerceptionBridge + Egress Guard |
| 2 | Claude 3.5 Sonnet | Undisclosed | Dense Transformer | Computer Use, Constitutional AI, Agentic coding | NT-ACT agent loop + NT-SHIELD safety |
| 3 | Gemini 2.5 Pro | Undisclosed (MoE) | Sparse MoE Transformer | Deep Think (parallel thinking), 1M+ context, TPUv5p | GWT salience + ConsciousnessTree |
| 4 | Llama 4 Scout | 17B active / 109B total (16E) | MoE + iRoPE | 10M context, interleaved attention without positional embeddings | NT-NEXUS long memory + KVMem paged KV |
| 5 | DeepSeek V4.1 Flash | 552B total, 8B/16B active | CED (Causal Encoder-Decoder) MoE | CSA2 sparse attention, 890B/token KV cache, 384 routed experts | NT-MEMORY KV optimization + NT-ACT efficiency |
| 6 | Qwen 3 | 0.6B-235B (MoE + Dense) | MoE (128E/8A) + Dense | Thinking/non-thinking unified mode, thinking budget | GWT cost-aware routing + SEAL pipeline |
| 7 | Mistral Large 3 | 675B total, 41B active | Granular MoE | 256K context, NVFP4 quantization, native vision | NT-WORLD multimodal + NT-SHIELD deployment |
| 8 | Phi-4 Reasoning | 14B | Dense Transformer | Teachable prompt curation, o3-mini distillation, GRPO RL | NT-MIND distillation + experience-tree |
| 9 | Yi-Lightning | Undisclosed (MoE) | Enhanced MoE | PEP load balancing, cross-layer KV cache sharing (82.8% reduction) | NT-MEMORY KV sharing + NT-ACT routing |
| 10 | Grok 3 | Undisclosed (~300-400B est.) | Dense/MoE (undisclosed) | RL-at-scale reasoning, DeepSearch agent, 200K H100 training | NT-WORLD DeepSearch + NT-ACT agent tools |

---

## 1. GPT-4o (OpenAI)

### Architecture

- **Type**: Autoregressive omni model — end-to-end across text, vision, and audio
- **Modalities**: Input (text + audio + image + video), Output (text + audio + image)
- **Key structural evidence**: Hybrid AR backbone + diffusion-based generation head (empirically confirmed via GPT-ImgEval classifier analysis)
- **Response latency**: 232ms minimum, 320ms average (speech-to-speech)
- **Pricing**: 50% cheaper than GPT-4 Turbo

### Key Innovations

1. **Omni-modal end-to-end**: Single neural network processes all input/output modalities — no separate encoders stitched together
2. **AR + Diffusion hybrid**: Uses autoregressive transformer for semantic understanding, diffusion head for image decoding (confirmed by classifier distinguishing AR vs diffusion outputs)
3. **Continuous visual tokens**: Likely avoids VQ (vector quantization) for images, using continuous token representations to preserve comprehension (similar to MAR approach)
4. **Audio as first-class modality**: Native speech understanding and generation without ASR→LLM→TTS pipeline

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Omni-modal end-to-end | `PerceptionBridge` (L2) | Attention-gated bridge connecting sensory integration with consciousness — same principle of unified multimodal flow |
| AR + Diffusion hybrid | `GWT` attention routing | Selective routing between semantic (AR) and generative (diffusion) pathways mirrors GWT's salience-based broadcast |
| Egress privacy guard | `nt_core_llm::egress_privacy_guard` | GPT-4o's safety evaluations on voice modality directly validate NeoTrix's Trust-tiered egress policy design |
| Real-time audio | `nt_feel` emotion engine | Speech prosody → emotion mapping is a direct input channel for NT-FEEL's EmotionLabel (11 variants) |

---

## 2. Claude 3.5 Sonnet (Anthropic)

### Architecture

- **Type**: Dense Transformer (undisclosed parameters)
- **Modalities**: Text + image input; text output
- **Context**: 200K tokens
- **Key structural element**: Constitutional AI (CAI) training methodology
- **Speed**: 2x faster than Claude 3 Opus

### Key Innovations

1. **Computer Use (GUI grounding)**: Interprets screenshots and generates tool calls — OSWorld 14.9% (screenshot-only), 22% with 50 steps
2. **Agentic coding loop**: 64% on internal eval, 49% on SWE-bench Verified — model iterates: search→view→edit→test→submit
3. **Constitutional AI**: Principle-based training replacing RLHF, enabling self-correction without human labelers
4. **Responsible Scaling Policy (RSP)**: ASL-level gating system (ASL-2/3/4) with automatic capability thresholds triggering safety protocols
5. **Haiku efficiency**: Smaller model matching larger predecessor performance — same architecture, better training

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Computer Use (GUI) | `nt_act` MCP tools | Claude's screenshot→action loop is the production reference for NT-ACT's tool-calling architecture |
| Agentic coding loop | `nt_act::orchestration` | Multi-step search→edit→test mirrors NT-ACT's orchestration pipeline with self-correction |
| Constitutional AI | `NT-SHIELD` + `NT-GOVERNANCE` | RSP/ASL gating maps directly to NT-SHIELD risk classification + NT-GOVERNANCE policy enforcement |
| RSP thresholds | `HeartbeatAggregator` | Automatic capability→safety threshold detection parallels HeartbeatAggregator's time-decay health signals |

---

## 3. Gemini 2.5 Pro (Google DeepMind)

### Architecture

- **Type**: Sparse MoE Transformer with native multimodal support
- **Training**: TPUv5p (first model family on this architecture), 8960-chip pods across datacenters
- **Context**: 1M+ tokens (2M coming soon)
- **Modalities**: Text, audio, images, video (up to 3 hours)
- **Family**: 2.5 Pro (thinking), 2.5 Flash (hybrid), 2.0 Flash (fast), 2.0 Flash-Lite (cheapest)

### Key Innovations

1. **Deep Think**: Parallel thinking technique — generates multiple hypotheses, critiques them, then selects best. Blends parallel thinking during response generation
2. **Thinking budget control**: User-configurable thinking budget (token count) allowing quality/latency/cost tradeoff
3. **K-sparse distillation**: Approximates teacher's next-token distribution using top-k sparse vocabulary, reducing storage by factor of k
4. **Training stability innovations**: Signal propagation improvements, optimization dynamics enhancements for MoE at scale
5. **Pareto frontier coverage**: Four model tiers spanning capability vs cost — user chooses their operating point

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Deep Think (parallel hypotheses) | `ConsciousnessTree` (Soil→Core loop) | Multi-hypothesis generation + critique mirrors ConsciousnessTree's 6-stage feedback loop |
| Thinking budget control | `GWT` salience routing | User-controlled compute allocation = GWT's task-type-based attention allocation with cost weight (Axiom A1) |
| K-sparse distillation | `nt_mind::distillation` | Sparse vocabulary approximation for smaller models = NT-MIND's skill crystallization with information compression |
| Pareto frontier | `Ordered Backend Router` | Multi-tier model selection mirrors NT-WORLD's ordered fallback chain across backends |
| TPUv5p distributed training | `NT-PHYSICAL` infrastructure | Multi-datacenter pod training = NT-PHYSICAL's power management and hardware awareness |

---

## 4. Llama 4 Scout (Meta)

### Architecture

- **Type**: MoE with early fusion for native multimodality
- **Parameters**: 17B active / 109B total (16 routed experts)
- **Context**: 10M tokens (industry-leading)
- **Training**: ~40T tokens, multimodal (text + image)
- **Deployment**: Single H100 GPU with Int4 quantization

### Key Innovations

1. **iRoPE (Interleaved RoPE)**: Alternating attention layers — some with RoPE positional embeddings, some without. "i" = interleaved for "infinite" context aspiration
2. **Inference-time temperature scaling of attention**: Enhances length generalization beyond training context
3. **Early fusion multimodality**: Images and text fused from first layer — no separate vision encoder pipeline
4. **10M context window**: 80x increase from Llama 3's 128K, enabling multi-document summarization and codebase-wide reasoning
5. **Alternating dense/MoE layers**: Inference efficiency through layer-type alternation

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| iRoPE (position-free layers) | `NT-NEXUS` cross-session memory | Position-free attention enables infinite context = NT-NEXUS's session-spanning memory without positional decay |
| 10M context | `KVMem paged KV` (Axiom A2) | Meta's context scaling validates NeoTrix's KVMem paged KV virtualization approach for >256K sessions |
| Early fusion | `PerceptionBridge` | End-to-end multimodal fusion from first layer = PerceptionBridge's direct L2→L5 attention-gated connection |
| Single-H100 deployment | `nt_io` provider routing | Efficient single-GPU serving = NT-IO's cost-aware provider selection (Axiom A1) |

---

## 5. DeepSeek V4.1 Flash (DeepSeek AI)

### Architecture

- **Type**: Causal Encoder-Decoder (CED) MoE
- **Parameters**: 552B total, 8B active (prefill) / 16B active (decode)
- **Experts**: 384 routed + 1 shared per MoE layer, 6 activated per token
- **Context**: 1M tokens
- **KV Cache**: 890 bytes/token (1/4 of V4-Flash, 437x reduction from V1)

### Key Innovations

1. **Causal Encoder-Decoder (CED)**: 40-layer Transformer split into 20-layer encoder + 20-layer decoder. Decoder's KV cache projected from encoder's final hidden states — not derived per-layer. This is the key architectural breakthrough
2. **Compressed Sparse Attention 2 (CSA2)**: Three static modes per attention layer (Full/Reindex/Reuse) sharing KV data and sparse-attention indices across layers
3. **Hierarchical Sparse Indexer**: Constrains deeper indexing to candidate pool from first Full layer — bounds indexer cost independent of context length
4. **FP4 KV caching**: E2M1 format with E4M3 scale per 16 channels — further shrinks memory without separate quantization pass
5. **SWA Bounded Replay**: Replays only recent n_win tokens to reconstruct sliding window attention states — avoids SSD round-trip entirely
6. **Engram conditional memory**: 196B parameters, sparsely accessed via token-based lookup — loaded on-demand, not resident
7. **DSpark speculative decoding**: Semi-autoregressive draft generation with confidence-scheduled verification
8. **Controllable reasoning effort**: Integer 1-100 setting trades inference cost for accuracy

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| CED architecture | `CapabilityBridge` | Encoder→decoder projection = CapabilityBridge mapping evolution view to runtime view |
| CSA2 static mode sharing | `NT-MEMORY` KV sharing | Cross-layer KV reuse = NT-MEMORY's embedding deduplication and index sharing |
| FP4 KV caching | `kv_cache_optimizer.rs` | Direct implementation target — DeepSeek's 890B/token benchmark is the optimization goal |
| Engram conditional memory | `experience-tree lazy loading` | 196B sparsely-accessed memory = experience-tree's on-demand branch loading via route table |
| Controllable reasoning effort (1-100) | `GWT` salience + cost weight | Dial-a-compute = GWT's task-complexity-based attention allocation (Axiom A1: Cost-Aware Routing) |
| DSpark speculative decoding | `NT-ACT` parallel task | Draft-verify pattern = NT-ACT's ParallelTaskManager speculative execution |

---

## 6. Qwen 3 (Alibaba Cloud)

### Architecture

- **Type**: Dense (0.6B-32B) + MoE (30B-A3B, 235B-A22B)
- **MoE config**: 128 total experts, 8 activated per token, no shared experts
- **Dense config**: GQA, SwiGLU, RoPE, RMSNorm, QK-Norm (removed QKV-bias from Qwen2)
- **Context**: 32K (small) to 128K (large)
- **Languages**: 119 (expanded from 29 in Qwen2.5)

### Key Innovations

1. **Unified thinking/non-thinking mode**: Single model switches between reasoning (long CoT) and fast-response modes — eliminates need for separate models (chat vs reasoning)
2. **Thinking budget mechanism**: User controls computational resources during inference — adaptive allocation per task complexity
3. **4-stage training pipeline**: (1) CoT cold start → (2) Reasoning RL → (3) Mode fusion (thinking + non-thinking) → (4) General RL
4. **Global-batch load balancing loss**: Encourages expert specialization across full batch (not per-expert constraint)
5. **No shared experts**: Unlike Qwen2.5-MoE — all experts are routed, simplifying architecture
6. **Weak distillation**: Flagship model knowledge distilled to smaller models — 4B model matches Qwen2.5-72B

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Unified thinking/non-thinking | `GWT` adaptive routing | Mode switching = GWT's task-type-based attention routing between fast (I/O) and slow (reasoning) pathways |
| Thinking budget | `Axiom A1: Cost-Aware Routing` | Direct implementation — budget control maps to GWT salience with token cost weight |
| 4-stage training | `SEAL pipeline` | Cold start→RL→fusion→general maps to SEAL's exploration→distillation→self-test→absorption |
| Global-batch load balancing | `ParallelTaskManager` | Cross-task expert balancing = NT-ACT's multi-device load balancing |
| Weak distillation | `nt_mind::distillation` | Flagship→smaller model transfer = NT-MIND's skill crystallization |

---

## 7. Mistral Large 3 (Mistral AI)

### Architecture

- **Type**: Granular Sparse MoE + Vision Encoder (2.5B)
- **Parameters**: 675B total, 41B active
- **Context**: 256K tokens
- **Training**: 3,000 NVIDIA H200 GPUs (Hopper), from scratch
- **Precision**: NVFP4, FP8, BF16 — single 8×H100 node deployment
- **Modalities**: Text + image input; text output

### Key Innovations

1. **Granular MoE**: Fine-grained expert segmentation — more experts, fewer parameters per expert, better activation combinations
2. **NVFP4 quantization**: Native 4-bit format for Blackwell GPUs — 675B model runs on single 8×H100 node
3. **Native vision encoder**: 2.5B vision module fused into architecture — not adapter-based
4. **Apache 2.0 license**: Full open-weight with base + instruct + reasoning variants
5. **Speculative decoding**: NVIDIA Blackwell-optimized with prefill/decode disaggregated serving

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Granular MoE | `Skill Tree` node granularity | Fine-grained experts = fine-grained skill nodes (Small/Notable/Keystone tiers) |
| NVFP4 deployment | `NT-SHIELD` resource optimization | Single-node frontier deployment = NT-SHIELD's stealth-efficient resource usage |
| Native vision | `NT-WORLD` perception | Integrated vision encoder = NT-WORLD's UnifiedCrawler native multimodal parsing |
| Open-weight variants | `Constellation C0-C6` | Base→Instruct→Reasoning maturity = Constellation C0→C6 progression |

---

## 8. Phi-4 Reasoning (Microsoft Research)

### Architecture

- **Type**: Dense decoder-only Transformer (same as Phi-4 base)
- **Parameters**: 14B
- **Context**: 32K tokens (extended from 16K via doubled RoPE base frequency)
- **Training**: 32 NVIDIA H100 GPUs, 2.5 days, 16B tokens

### Key Innovations

1. **Teachable prompt curation**: Prompts selected at boundary of base model capability — "optimal complexity" for maximum learning
2. **o3-mini as teacher**: Synthetic reasoning traces generated by o3-mini (medium effort preferred over DeepSeek-R1 for token efficiency)
3. **Reasoning tokens**: Placeholder tokens repurposed as `<think>`/`</think>` — explicit thinking blocks
4. **Additive domain optimization**: Independently optimize data sources per domain, then combine — linear scaling of domain expertise
5. **GRPO reinforcement learning**: Group Relative Policy Optimization on just 6,400 math problems → significant accuracy boost with 1.5x longer reasoning traces
6. **Non-trivial transfer**: Improvements from reasoning training transfer to general benchmarks not seen during training

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| Teachable prompt curation | `experience-tree` curation | Boundary-difficulty selection = experience-tree's distillation phase selecting high-signal experiences |
| o3-mini teacher distillation | `nt_mind::distillation` | Teacher→student transfer = NT-MIND's skill crystallization pipeline |
| Reasoning tokens (think/end think) | `ConsciousnessTree` stage markers | Explicit thinking blocks = ConsciousnessTree's Soil→Roots→Trunk→Branches→Fruits→Core markers |
| Additive domain optimization | `Dual Specialization` | Independent domain optimization = Weapon Set I/II independent specialization |
| GRPO on small dataset | `experience-tree` feedback | Small dataset RL = experience-tree's high-signal, low-volume absorption |
| Transfer learning | `Cross-domain audit` | Unplanned capability transfer = CrossModuleAudit discovering emergent cross-domain effects |

---

## 9. Yi-Lightning (01.AI)

### Architecture

- **Type**: Enhanced MoE
- **Parameters**: Undisclosed (fine-grained expert segmentation)
- **Key features**: Sliding window + full attention hybrid, cross-layer KV cache sharing

### Key Innovations

1. **Partitioned EP load balancing (PEP)**: Splits experts within EP groups into smaller partitions — ensures balanced token distribution across All-to-All communication
2. **Cross-layer KV cache sharing**: Shares KV states between consecutive full attention layers — 82.8% memory reduction
3. **Hybrid attention blocks**: 3 sliding window attention layers + 1 full attention layer — captures both local patterns and global dependencies
4. **FP8 hardware-aware design**: Architecture precisely aligned with NVIDIA Hopper specifications — 1,200 TFLOPS per card
5. **Three-level load balancing**: L_ST (per-expert) + L_EP (EP group) + L_PEP (partitioned) — cascading balance constraints

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| PEP load balancing | `NT-ACT::ParallelTaskManager` | Cascading balance = NT-ACT's multi-tier load balancing across devices |
| Cross-layer KV sharing | `NT-MEMORY` deduplication | 82.8% memory reduction = NT-MEMORY's embedding dedup and index sharing strategy |
| Hybrid attention (SW + Full) | `GWT` local/global routing | Sliding window (local) + full (global) = GWT's selective broadcast between local specialists and global workspace |
| Three-level balancing | `HeartbeatAggregator` cascading | L_ST→L_EP→L_PEP = HeartbeatAggregator's per-module→per-domain→system-wide health aggregation |
| FP8 hardware-aware | `NT-PHYSICAL` | Architecture-hardware co-design = NT-PHYSICAL's sensor/motor hardware awareness |

---

## 10. Grok 3 (xAI)

### Architecture

- **Type**: Transformer (dense or MoE — undisclosed, estimated 300-400B active)
- **Context**: 131K tokens
- **Training**: 200,000 NVIDIA H100 GPUs (Colossus), 12.8T tokens, 10x compute over Grok 2
- **Modalities**: Text + image input; text output
- **Data**: Web + proprietary X (Twitter) data — only frontier lab with real-time X training access

### Key Innovations

1. **RL-at-scale reasoning**: Reinforcement learning applied at pretraining scale — not just post-training alignment. Think mode learns to backtrack, self-correct, explore alternatives
2. **DeepSearch agent**: Real-time web + X search synthesized into cited, structured reports. Processes 90+ sources in 52 seconds
3. **Colossus infrastructure**: 200K H100s assembled in 122 days — largest single-cluster training disclosed at launch
4. **Reasoning time scaling**: Model spends seconds to minutes on hard problems, generating visible thinking traces
5. **Native tool integration**: Grok 4 (successor) trained for native use of web browsing, code execution, search — tool calling learned during training, not bolted on

### NeoTrix Mapping

| Innovation | NeoTrix Component | Connection |
|-----------|-------------------|------------|
| RL-at-scale reasoning | `SEAL pipeline` | Pretraining-scale RL = SEAL's exploration→distillation at architecture level, not just post-training |
| DeepSearch agent | `NT-WORLD::UnifiedCrawler` | Real-time multi-source synthesis = NT-WORLD's crawl→parse→classify→extract pipeline |
| Colossus 200K GPU | `NT-PHYSICAL` power management | Massive cluster = NT-PHYSICAL's power budget and thermal awareness |
| Reasoning time scaling | `ConsciousnessTree` cycle depth | Seconds-to-minutes thinking = ConsciousnessTree's adaptive cycle depth based on task complexity |
| Native tool training | `NT-ACT::MCP tools` | Tools learned during training = NT-ACT's tool-coding integration in capability network (L1) |

---

## Cross-Model Innovation Synthesis

### Tier 1: Universal Patterns (appeared in 8+ models)

| Pattern | Models | NeoTrix Component |
|---------|--------|-------------------|
| **MoE with fine-grained experts** | Gemini, Llama, DeepSeek, Qwen, Mistral, Yi | `Skill Tree` node decomposition |
| **Thinking budget / compute control** | Gemini, Qwen, DeepSeek, GPT-4o | `GWT` cost-aware routing (Axiom A1) |
| **KV cache optimization** | DeepSeek, Yi, Llama | `kv_cache_optimizer.rs` + KVMem (Axiom A2) |
| **Native multimodal fusion** | GPT-4o, Gemini, Llama, Mistral | `PerceptionBridge` |

### Tier 2: Emerging Patterns (3-7 models)

| Pattern | Models | NeoTrix Component |
|---------|--------|-------------------|
| **RL-at-scale reasoning** | Grok, DeepSeek, Qwen, Phi-4 | `SEAL pipeline` |
| **Speculative decoding** | DeepSeek, Mistral | `NT-ACT::ParallelTaskManager` |
| **Controllable reasoning effort** | DeepSeek, Qwen, Gemini | `GWT` salience dial |
| **Cross-layer KV sharing** | Yi, DeepSeek | `NT-MEMORY` deduplication |

### Tier 3: Differentiating Patterns (1-2 models)

| Pattern | Models | NeoTrix Component |
|---------|--------|-------------------|
| **CED asymmetric architecture** | DeepSeek only | `CapabilityBridge` |
| **iRoPE (position-free layers)** | Llama 4 only | `NT-NEXUS` infinite memory |
| **PEP cascading load balance** | Yi only | `NT-ACT::ParallelTaskManager` |
| **Teachable prompt curation** | Phi-4 only | `experience-tree` curation |

---

## Actionable Absorption Targets for NeoTrix

### P0 — Immediate Implementation

1. **DeepSeek CSA2 → `kv_cache_optimizer.rs`**: Implement Full/Reindex/Reuse static mode sharing. Target: 890 bytes/token baseline
2. **Qwen thinking budget → GWT salience**: Add user-configurable thinking budget parameter to GWT attention routing
3. **Phi-4 teachable prompts → experience-tree**: Implement boundary-difficulty selection for experience curation

### P1 — Next Sprint

4. **DeepSeek CED → CapabilityBridge**: Asymmetric encoder-decoder projection for evolution→runtime bridge
5. **Yi PEP → ParallelTaskManager**: Cascading three-level load balancing (expert→group→partition)
6. **Llama iRoPE → NT-NEXUS**: Interleaved position-free attention for cross-session memory

### P2 — Architecture Phase

7. **Grok RL-at-scale → SEAL pipeline**: Extend SEAL exploration to pretraining-scale RL
8. **Gemini Deep Think → ConsciousnessTree**: Parallel hypothesis generation with critique-and-select
9. **Mistral NVFP4 → NT-SHIELD deployment**: Single-node frontier model deployment optimization

---

## References

| Model | Source | Date |
|-------|--------|------|
| GPT-4o | arXiv:2410.21276 (System Card) | 2024-10 |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum | 2024-06/10 |
| Gemini 2.5 Pro | arXiv:2507.06261 | 2025-07 |
| Llama 4 Scout | Meta AI Blog + MODEL_CARD.md | 2025-04 |
| DeepSeek V4.1 Flash | HuggingFace README + Tech Report | 2026-09 |
| Qwen 3 | arXiv:2505.09388 | 2025-05 |
| Mistral Large 3 | Mistral Docs + HuggingFace | 2025-12 |
| Phi-4 Reasoning | Microsoft Research (arXiv:2504.21318) | 2025-04 |
| Yi-Lightning | arXiv:2412.01253 | 2024-12 |
| Grok 3 | xAI News + ChatForest Review | 2025-02 |
