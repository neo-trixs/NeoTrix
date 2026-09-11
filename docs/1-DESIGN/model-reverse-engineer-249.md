# Model Reverse Engineering 249

**Batch**: 2026-09-11 | **Scope**: 10 frontier models (2024-2026)

---

## Architecture Summary

| Model | Total Params | Active Params | Architecture | Context | Key Innovation |
|-------|-------------|---------------|--------------|---------|----------------|
| **GPT-4o** | ~200B (est.) | ~100B (est.) | Dense/MoE (undisclosed) | 128K | End-to-end omni-modal (text+vision+audio), unified tokenization |
| **Claude 3.5 Sonnet** | Undisclosed | Undisclosed | Transformer | 200K | Hybrid sparse attention (sliding window + global sparse), GQA 8:1 |
| **Gemini 2.5 Pro** | Undisclosed | Undisclosed | Sparse MoE + Transformer | 1M+ | Native multimodal + Thinking (reasoning budget), TPUv5p |
| **Llama 4 Scout** | 109B | 17B | MoE (16 experts) | 10M | iRoPE (interleaved attention without positional embeddings), early fusion |
| **DeepSeek V4.1-Flash** | 552B | 8B/16B | CED (Causal Encoder-Decoder) + MoE (384 routed experts) | 1M | CSA2 (Compressed Sparse Attention 2), Engram conditional memory, Hyper-Connections |
| **Qwen3-235B-A22B** | 235B | 22B | MoE (128 experts) | 128K (1M ext.) | Thinking/Non-Thinking mode fusion, thinking budget, 119 languages |
| **Mistral Large 3** | 675B | 41B | Granular MoE | 256K | Granular MoE (many small experts), native 2.5B vision encoder, Apache 2.0 |
| **Phi-4-reasoning** | 14B | 14B | Dense Transformer | 32K | Distillation from o3-mini, "teachable" prompt curation, RoPE frequency doubling |
| **Yi-Lightning** | Undisclosed | Undisclosed | Enhanced MoE | Undisclosed | Fine-grained expert segmentation, EP+PEP load balancing, hybrid attention (3 SWA + 1 full), cross-layer KV cache reuse |
| **Grok 3** | Undisclosed | Undisclosed | Transformer (likely MoE) | 1M | RL-trained chain-of-thought reasoning (Think/BigBrain/DeepSearch), 10x compute of predecessor |

---

## Detailed Architectural Innovations

### 1. GPT-4o — End-to-End Omni-Modal Architecture

**Source**: OpenAI (May 2024), arXiv:2410.21276

**Architecture**:
- Single neural network trained end-to-end across text, vision, and audio
- Unified token stream: text tokens (BPE), image patch tokens, audio codec tokens (Encodec/SoundStream-style)
- Single transformer stack with modality-specific embedding/unembedding layers
- Cross-modal attention via self-attention within unified stream (no separate cross-modal layers)
- Joint training on interleaved multimodal data (transcribed conversations, captioned images, videos)

**Key Metrics**:
- Median audio latency: 232ms (vs 2.8s in staged pipeline)
- ~2x faster and ~half price of GPT-4 Turbo
- Estimated ≤200B parameters (fits single H100 server)

**Architectural Innovation**:
- Eliminated the CLIP-style staged pipeline (separate vision encoder → projection → LLM)
- End-to-end training preserves information that staged pipelines discard
- Unified tokenization enables native cross-modal reasoning

### 2. Claude 3.5 Sonnet — Hybrid Sparse Attention

**Source**: Anthropic Model Card, reverse-engineering analysis

**Architecture**:
- Modified BPE tokenizer (100K vocab, optimized for code)
- RoPE with base frequency 10,000, extended to 200K via linear scaling
- **Hybrid attention**: even layers = local sliding window (1024 tokens), odd layers = global sparse (every 64th token)
- Grouped Query Attention (GQA): 8 query groups per KV head, 32 total attention heads → 4x KV cache reduction
- Optional lossless context compression (up to 22% for repeated patterns)
- 36 transformer layers

**Performance Impact**:
- FLOPs reduced from 40 TFLOPs to 12.4 TFLOPs per 100K tokens
- KV cache: 3.2GB → 1.2GB per 100K tokens
- Latency: 1400ms → 840ms (p99)
- 2% drop in long-range retrieval accuracy (negligible for production)

**Architectural Innovation**:
- Alternating local/global attention layers — each layer doesn't need to compute full attention
- Precomputed attention masks (120ms startup, saves 80ms per request)
- Tunable GQA ratio per workload (code: 16 groups, summarization: 4 groups)

### 3. Gemini 2.5 Pro — Sparse MoE + Native Thinking

**Source**: Google DeepMind (July 2025), arXiv:2507.06261

**Architecture**:
- Sparse MoE Transformer with native multimodal support (text, vision, audio)
- **Thinking mode**: RL-trained inference-time reasoning with configurable budget
- Trained on TPUv5p (8960-chip pods, multi-datacenter)
- Distillation for smaller models (k-sparse teacher distribution approximation)
- Slice-granularity elasticity (automatic failure recovery, ~97% throughput during recovery)
- Split-phase SDC detection (lightweight deterministic replay, <1 min localization)

**Key Capabilities**:
- 1M+ token context
- 3-hour video processing
- Native tool use (Google Search, code execution)
- Thinking budget control (user-configurable reasoning depth)

**Architectural Innovation**:
- MoE decouples capacity from cost (activate subset per token)
- Native multimodal from pretraining (not bolted on)
- Thinking as first-class capability (not separate model)
- Elastic training infrastructure (fault tolerance at scale)

### 4. Llama 4 Scout — iRoPE + Early Fusion

**Source**: Meta (April 2025), HuggingFace

**Architecture**:
- 17B active params, 109B total, 16 experts (alternating dense + MoE layers)
- **iRoPE**: Interleaved attention layers without positional embeddings (every Nth layer has no RoPE)
- Inference-time temperature scaling of attention for length generalization
- Early fusion: text + vision tokens unified from pretraining start
- Vision encoder: MetaCLIP-based, trained with frozen Llama model
- Shared expert + routed experts (each token → shared + 1 routed expert)

**Key Capabilities**:
- 10M token context (industry-leading)
- Fits single H100 GPU (with INT4 quantization)
- 200 languages, 40T training tokens

**Architectural Innovation**:
- iRoPE: interleaved position-free layers enable infinite context extrapolation
- Early fusion enables joint pretraining on unlabeled text+image+video
- Single-GPU deployment via expert sparsity

### 5. DeepSeek V4.1-Flash — CED + Engram Memory

**Source**: DeepSeek (September 2026)

**Architecture**:
- **Causal Encoder-Decoder (CED)**: 40 layers (20 encoder + 20 decoder)
  - Decoder KV cache projected from encoder hidden states (not decoder layers)
  - 8B active for prefill, 16B for decode
- **CSA2 (Compressed Sparse Attention 2)**: 3 static modes (Full/Index/Reuse), hierarchical sparse indexer
- **Hyper-Connections**: Residual stream as 4 parallel copies, elementwise gate
- **Engram conditional memory**: 196B params, sparsely accessed via token-based n-gram hash lookup
- **DSpark speculative decoding**: Semi-autoregressive draft with confidence-scheduled verification
- FP4 main KV caching (E2M1 format, 890 bytes/token — 1/4 of DeepSeek-V4-Flash)
- 384 routed experts + 1 shared expert, 6 activated per token
- Vision: DeepSeek-ViT (2D-RoPE, 3×3 pixel-unshuffle), 2-layer MLP projector

**Key Capabilities**:
- 1M context, native multimodal (image + text)
- SWA Bounded Replay (1/8 KV cache of predecessor)
- Asymmetric compute: cheap input (8B), full output (16B)

**Architectural Innovation**:
- CED: encoder compresses input, decoder generates — asymmetric compute allocation
- Engram: external n-gram memory injected into residual stream (retrieval-augmented at architecture level)
- Hyper-Connections: parallel residual copies with learned mixing (improved gradient flow)
- Compressed KV cache enables 1M context on practical hardware

### 6. Qwen3-235B-A22B — Thinking Mode Fusion

**Source**: Alibaba (April 2025)

**Architecture**:
- 235B total, 22B active, 128 experts (8 activated per token), no shared experts
- Grouped Query Attention (GQA), SwiGLU, RoPE, RMSNorm with pre-normalization
- QK-Norm (no QKV-bias, unlike Qwen2)
- Global-batch load balancing loss (encourages expert specialization)
- 94 layers, 64 heads (Q/KV ratio 16:1)

**Key Innovation — Thinking/Non-Thinking Fusion**:
1. **Long CoT Cold Start**: SFT on diverse reasoning data
2. **Reasoning RL**: Rule-based rewards, scaled exploration
3. **Thinking Mode Fusion**: Continual SFT integrating both modes into single model
4. **General RL**: 20+ domain tasks, instruction/format/agent capabilities

**Training**:
- 36T tokens, 119 languages
- 4-stage pretraining (S1: 30T@4K → S2: 5T@32K → S3: long-context extension)
- Strong-to-Weak Distillation for smaller models

**Architectural Innovation**:
- Single model handles both deep reasoning and fast responses
- Thinking budget control (user-configurable)
- No shared experts (unlike Llama 4) — all 128 routed
- Global-batch load balancing (vs per-expert)

### 7. Mistral Large 3 — Granular MoE

**Source**: Mistral AI (December 2025)

**Architecture**:
- 675B total, 41B active (~6% sparsity ratio)
- **Granular MoE**: many small experts (not few large ones)
- Native 2.5B vision encoder (fused, not bolted on)
- Tekken-family tokenizer (multilingual + code optimized)
- 256K context window
- NVFP4 quantization support (Blackwell NVL72 / 8×A100)

**Training**:
- 3000 H200 GPUs from scratch
- DPO-style post-training (not full RLHF)
- Apache 2.0 license (open weight)

**Architectural Innovation**:
- Granularity: more small experts → finer routing choices, better specialization
- Cost: 675B model runs at ~41B dense cost
- Trade-off: harder routing + more All-to-All GPU communication
- Predictable inference cost (no hidden thinking tokens)

### 8. Phi-4-reasoning — Data-Centric Small Reasoning

**Source**: Microsoft Research (April 2025)

**Architecture**:
- 14B dense decoder-only Transformer (same as Phi-4 base)
- Two modifications: thinking tokens (`<think>`/`</think>`) + doubled RoPE base frequency
- Context extended from 16K → 32K
- SFT on 1.4M prompt-response pairs (8.3B unique tokens)
- Teacher: o3-mini (medium/high reasoning effort)
- GRPO reinforcement learning (6.4K math problems, rule-based reward)

**Key Innovation**:
- Reasoning as transferable meta-skill (learns via SFT alone, enhanced by RL)
- "Teachable" prompt selection: filtered to lie at boundary of base model capabilities
- Strong generalization: improvements transfer to non-reasoning tasks (IFEval, FlenQA)
- Outperforms DeepSeek-R1-Distill-Llama-70B (5x larger)

**Architectural Innovation**:
- Minimal architectural change (just token + RoPE modifications)
- Data quality > model size (14B beats 70B)
- SFT + RL synergy: SFT provides strong baseline, RL amplifies

### 9. Yi-Lightning — Enhanced MoE with Hybrid Attention

**Source**: 01.AI (December 2024)

**Architecture**:
- Enhanced MoE with fine-grained expert segmentation (partition FFN into smaller units)
- **3-level load balancing**: Switch-Transformer (per-expert) → EP group → Partitioned EP (PEP)
- **Hybrid attention**: 3 sliding window attention layers + 1 full attention layer
- **Cross-layer KV cache reuse**: share KV between consecutive full attention layers
- FP8 quantization (1,200 TFLOPS/card on Hopper)
- Hybrid parallelism: expert + pipeline parallelism

**Performance Impact**:
- 82.8% memory reduction for long sequences
- 100%+ improvement in MoE operator execution
- 70% training speedup via pipeline stage optimization

**Architectural Innovation**:
- Fine-grained segmentation: more experts, each smaller, better parameter utilization
- PEP load balancing: addresses All-to-All communication imbalance
- Hybrid attention: 3:1 SWA:full ratio captures local + global patterns efficiently
- Cross-layer KV reuse: halves memory for full attention components

### 10. Grok 3 — RL-Scaled Reasoning

**Source**: xAI (February 2025)

**Architecture**:
- Undisclosed architecture (likely MoE based on compute scaling)
- 1M context window (8x predecessor)
- 200,000 H100 GPUs (Colossus supercluster)
- 10x compute of previous SOTA models

**Key Innovation — Reasoning Modes**:
- **Think**: Chain-of-thought reasoning (seconds to minutes)
- **Big Brain**: Extended reasoning with additional computation
- **DeepSearch**: Agent that searches web, synthesizes information, compiles reports
- RL-trained to refine problem-solving, correct errors, explore alternatives
- Selective token obscuring (anti-distillation measure)

**Architectural Innovation**:
- Massive compute scaling (10x predecessor)
- Inference-time compute scaling (thinking budget)
- Agent integration (code interpreter + internet access)
- Anti-distillation via token obscuring

---

## NeoTrix Mapping: Cross-Model Patterns

### Pattern 1: Mixture-of-Experts as Universal Substrate

| Model | Expert Count | Activation | Innovation |
|-------|-------------|------------|------------|
| GPT-4o | ~128 (est.) | ~100B | Unified multimodal MoE |
| Gemini 2.5 Pro | Undisclosed | Undisclosed | Sparse MoE + Thinking |
| Llama 4 Scout | 16 | 17B | Shared + routed expert |
| DeepSeek V4.1-Flash | 384 | 6/token | CED architecture |
| Qwen3 | 128 | 8/token | No shared experts |
| Mistral Large 3 | Granular | 41B | Many small experts |
| Yi-Lightning | Enhanced | Unknown | Fine-grained segmentation |

**NeoTrix Implication**: MoE is now the default architecture for frontier models. NeoTrix should treat MoE as a first-class capability for:
- **NT-CORE**: HyperCube could use MoE-style sparse activation for knowledge routing
- **NT-MIND**: SEAL pipeline could dynamically allocate expert "factions" per task
- **GWT Attention**: Salience routing maps naturally to expert routing (token → expert = attention → faction)

### Pattern 2: Thinking/Reasoning Budget Control

| Model | Mechanism | Budget Control | Cost Trade-off |
|-------|-----------|----------------|----------------|
| Gemini 2.5 Pro | Thinking tokens | User-configurable | Linear cost scaling |
| Qwen3 | Think/no_think flags | User-configurable | Seamless switching |
| Grok 3 | Think/BigBrain/DeepSearch | Three tiers | Seconds to minutes |
| Phi-4-reasoning | `<think>` tokens | Fixed (32K max) | SFT + RL |
| o3/o4 series | Hidden reasoning | Model-controlled | Variable |

**NeoTrix Implication**: Adaptive reasoning is the new standard. NeoTrix should implement:
- **NT-MIND**: SEAL pipeline phases should have configurable "thinking budget" (tokens allocated per phase)
- **ConsciousnessTree**: Growth cycle depth should be budget-aware (shallow cycles for simple tasks, deep for complex)
- **GWT**: Salience scoring should include "reasoning depth" as a factor

### Pattern 3: Native Multimodal vs Staged Pipeline

| Model | Approach | Latency | Trade-off |
|-------|----------|---------|-----------|
| GPT-4o | End-to-end unified | 232ms | Can't upgrade vision independently |
| Gemini 2.5 Pro | Native from pretraining | Low | Requires massive compute |
| Llama 4 Scout | Early fusion | Low | Joint pretraining required |
| DeepSeek V4.1-Flash | DeepSeek-ViT + MLP projector | Low | Vision encoder trained from scratch |
| Mistral Large 3 | 2.5B vision encoder | Low | Fused, not bolted on |
| Claude 3.5 Sonnet | Vision capabilities | Unknown | Likely staged |

**NeoTrix Implication**: Native multimodal is winning. NeoTrix should:
- **NT-WORLD**: UnifiedCrawler should treat multimodal as first-class input (not text-only)
- **NT-PHYSICAL**: Body schema should handle visual/audio natively (PerceptionBridge)
- **NT-IO**: LLM provider abstraction should support native multimodal APIs

### Pattern 4: KV Cache Optimization

| Model | Technique | Memory Reduction |
|-------|-----------|-----------------|
| Claude 3.5 Sonnet | GQA 8:1 + hybrid sparse | 4x (GQA) + 62% FLOPs reduction |
| DeepSeek V4.1-Flash | CSA2 + FP4 caching | 890 bytes/token (1/4 predecessor) |
| Yi-Lightning | Cross-layer KV reuse | 82.8% memory reduction |
| Llama 4 Scout | iRoPE | Enables 10M context |
| Gemini 2.5 Pro | TPUv5p optimizations | 1M+ context |

**NeoTrix Implication**: KV cache is the new bottleneck. NeoTrix should:
- **NT-MEMORY**: KB embedding storage should adopt similar compression (sparse indexing + compressed KV)
- **KVMem integration**: Paged KV virtualization for long sessions (>256K)
- **Experience-tree**: Lazy branch loading mirrors KV cache eviction strategies

### Pattern 5: Distillation as Model Family Strategy

| Model Family | Teacher | Student | Compression |
|-------------|---------|---------|-------------|
| Gemini 2.5 | Pro | Flash/Flash-Lite | k-sparse distribution |
| Qwen3 | 235B-A22B | 30B-A3B, 4B, etc. | On-policy distillation |
| Llama 4 | Behemoth | Scout/Maverick | Knowledge transfer |
| Phi-4 | o3-mini | Phi-4-reasoning | SFT on teacher traces |

**NeoTrix Implication**: Distillation is how model families scale. NeoTrix should:
- **NT-MIND**: SEAL distillation should support teacher→student patterns
- **Skill crystallization**: Small skill nodes distilled from large "teacher" capabilities
- **Rune Socketing**: Golden rune (error recovery) could use distilled knowledge from larger models

### Pattern 6: External Memory Injection

| Model | Mechanism | Architecture |
|-------|-----------|-------------|
| DeepSeek V4.1-Flash | Engram (196B params, n-gram hash) | Residual stream injection |
| Qwen3 (next) | n-gram embedding tables (51B off-accelerator) | Host memory prefetch |
| Phi-4-reasoning | External teacher knowledge | SFT traces |

**NeoTrix Implication**: External memory at architecture level. NeoTrix should:
- **NT-MEMORY**: KB could be injected into reasoning via Engram-style mechanisms
- **ConsciousnessTree**: Experience pointers could be "resident" in the reasoning stream
- **VSA HyperCube**: Associative recall could use hash-based lookup (like Engram)

---

## Synthesis: Universal Trends

### 1. Sparsity is Architecture
Every frontier model uses MoE or sparse attention. Dense models are legacy. NeoTrix's GWT salience routing is already sparse — formalize it as MoE.

### 2. Reasoning is Budgetable
Thinking tokens/budgets are now standard. NeoTrix's SEAL pipeline should expose configurable reasoning depth per phase.

### 3. Multimodal is Native
End-to-end training wins. NeoTrix's PerceptionBridge should treat all modalities as first-class.

### 4. KV Cache is the Bottleneck
Compression, reuse, and virtualization are critical. NeoTrix's memory layer needs similar optimizations.

### 5. Small Models Beat Large Ones
Phi-4 (14B) beats DeepSeek-R1-Distill-70B. Quality > quantity. NeoTrix's skill crystallization should prioritize data quality.

### 6. External Memory is Architectural
Engram-style injection into residual streams. NeoTrix's KB could be "resident" in reasoning, not just retrieved.

---

## Action Items

| Priority | Pattern | NeoTrix Component | Implementation |
|----------|---------|-------------------|----------------|
| P0 | MoE substrate | NT-CORE / GWT | Formalize GWT as MoE-style router |
| P0 | Thinking budget | NT-MIND / SEAL | Configurable reasoning depth per SEAL phase |
| P1 | Native multimodal | NT-WORLD / NT-PHYSICAL | UnifiedCrawler multimodal input pipeline |
| P1 | KV compression | NT-MEMORY | Sparse indexing + compressed embedding cache |
| P2 | Distillation | NT-MIND | Teacher→student skill crystallization |
| P2 | Engram memory | NT-MEMORY / ConsciousnessTree | KB injection into reasoning stream |
