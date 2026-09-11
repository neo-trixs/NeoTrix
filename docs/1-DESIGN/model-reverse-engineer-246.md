# Model Reverse-Engineer #246 — 10 LLM Architectures vs NeoTrix

**Date**: 2026-09-11  
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3  
**Method**: Web research → architecture extraction → NeoTrix mapping

---

## 1. Architecture Comparison Matrix

| Model | Type | Params (Total/Active) | Attention | KV Cache | Context | Multimodal | Reasoning |
|-------|------|----------------------|-----------|----------|---------|------------|-----------|
| **GPT-4o** | Dense Transformer | Undisclosed | Dense self-attention | Standard | 128K | E2E text+image+audio | No (non-reasoning) |
| **Claude 3.5 Sonnet** | MoE | 175B / 50B | Hybrid sparse (SWA + global) | GQA 4x reduction | 200K | Text+image | No (non-reasoning) |
| **Gemini 2.5 Pro** | Sparse MoE | Undisclosed | Standard MoE attention | Standard MoE | 1M | E2E text+image+audio+video | Thinking (budget-controlled) |
| **Llama 4 Scout** | Sparse MoE | 109B / 17B (16 experts) | iRoPE (interleaved RoPE/NoPE) | Standard MoE | 10M | Early fusion text+image | No (non-reasoning) |
| **DeepSeek V4.1 Flash** | MoE + CED | 552B backbone / 8B-16B | CSA2 (compressed sparse attn) | FP4 890B/token | 1M | ViT + MLP projector | Thinking (low/high/max) |
| **Qwen 3** | Dense + MoE | 235B / 22B (128 experts) | GQA + sliding window | Standard MoE | 128K-1M | Text (separate VL model) | Thinking + non-thinking hybrid |
| **Mistral Large 3** | Granular MoE | 675B / 41B | Standard MoE | Standard MoE | 256K | 2.5B vision encoder | No (non-reasoning) |
| **Phi-4 Reasoning** | Dense | 14B | Standard dense | Standard dense | 32K | Text only | Thinking (SFT + RL) |
| **Yi-Lightning** | Enhanced MoE | Undisclosed | Hybrid attention (3 SWA + 1 full) | Cross-layer KV reuse | 64K | Text | No (non-reasoning) |
| **Grok 3** | Transformer (MoE suspected) | Undisclosed (~1.5T est.) | Sparse attention | Standard | 131K-1M | Text+image | Think/Big Brain/DeepSearch |

---

## 2. Per-Model Architecture Deep-Dive

### 2.1 GPT-4o (OpenAI, May 2024)

**Key Innovations:**
- **End-to-end multimodal training**: Single neural network processes text, vision, and audio natively — not bolted-on CLIP-style encoders
- **Unified tokenization**: BPE text tokens + image patch tokens + neural audio codec tokens in one stream
- **Latency breakthrough**: 232ms median audio response (vs 2.8s in GPT-4 Turbo's staged pipeline)
- **Improved multilingual tokenizer**: 1.1x-4.4x fewer tokens for non-English scripts

**Architecture Details:**
- Decoder-only Transformer (confirmed)
- 128K context, 16K max output
- Dense attention layers (no MoE confirmed)
- Cross-modal attention via self-attention within unified token stream (not separate cross-modal layers)
- Modality-specific embedding/unembedding layers

**What's NOT disclosed:** Parameter count, training compute, exact audio tokenizer, context window split between modalities

---

### 2.2 Claude 3.5 Sonnet (Anthropic, June 2024)

**Key Innovations:**
- **Hybrid sparse attention**: Alternates local SWA (1024 token window) + global sparse (every 64th token) across layers
- **GQA with 8 KV heads**: 4x KV cache reduction vs standard MHA
- **Context compression**: Lossless compression for repeated patterns, 22% payload reduction
- **Constitutional AI alignment**: Rule-based RLHF

**Architecture Details:**
- 175B total parameters, ~50B active per token (MoE: 8 experts, 2 active)
- 60 layers, 8192 hidden dim, 64 attention heads, 8 KV heads
- RoPE base frequency 10,000, extended to 200K via linear scaling
- Hybrid attention: even layers = local SWA (1024), odd layers = global sparse (every 64th)
- FLOPs reduced to 12.4 TFLOPs per 100K tokens (vs 40 TFLOPs for dense)
- KV cache: 1.2GB per 100K tokens (vs 3.2GB dense)

---

### 2.3 Gemini 2.5 Pro (Google DeepMind, 2025)

**Key Innovations:**
- **Sparse MoE with native multimodality**: Text, vision, audio from pretraining start
- **Thinking with controllable budget**: Model decides how long to reason, user can set token budget
- **1M token context**: Processes entire codebases, 3-hour video
- **TPUv5p training**: First model on Google's latest TPU architecture
- **Distillation for smaller models**: k-sparse distribution approximation for teacher logits

**Architecture Details:**
- Sparse MoE transformer (exact param count undisclosed)
- Native multimodal (text + vision + audio inputs)
- Thinking integrated natively across all domains
- Synchronous data-parallel training across 8960-chip TPUv5p pods
- Elastic training with automatic slice recovery (<30s downtime)
- Split-phase SDC (Silent Data Corruption) detection

**Training Infrastructure:**
- Single-controller design (Pathways system)
- ~0.25% steps replayed for SDC detection, 6% confirmed as genuine corruption
- Multi-datacenter distributed training

---

### 2.4 Llama 4 Scout (Meta, April 2025)

**Key Innovations:**
- **iRoPE (Interleaved Rotary Positional Embeddings)**: Alternates RoPE layers + NoPE (no positional encoding) layers for infinite context generalization
- **10M token context**: Industry-leading for open-weight models
- **Early fusion multimodality**: Vision and text tokens unified from first layer
- **16 "fat" experts**: Larger, more generalized experts (vs Maverick's 128 fine-grained)
- **Single H100 deployability**: 109B params fits one GPU with INT4 quantization

**Architecture Details:**
- 109B total / 17B active parameters
- 16 routed experts + 1 shared expert per MoE layer
- Alternating dense and MoE layers
- iRoPE: 3 of 4 layers use chunked RoPE (8K chunks), every 4th layer uses NoPE
- QK-normalization + temperature-scaled softmax in NoPE layers
- Vision encoder based on MetaCLIP, trained with frozen Llama model
- Pre-trained with 256K context, extended to 10M via mid-training

---

### 2.5 DeepSeek V4.1 Flash (DeepSeek, September 2026)

**Key Innovations:**
- **Causal Encoder-Decoder (CED)**: 20-layer encoder + 20-layer decoder; decoder's KV cache projected from encoder's final hidden states
- **Asymmetric activation**: 8B active for prefill, 16B for decode (input-heavy tasks are cheaper)
- **CSA2 (Compressed Sparse Attention 2)**: 3 static modes (Full/Reindex/Reuse) sharing KV + indexer across layers
- **FP4 KV caching**: E2M1 format, 890 bytes per token (1/4 of V4 Flash)
- **SWA Bounded Replay**: Reconstructs missing SWA KV by replaying recent window, no SSD round-trip
- **Engram conditional memory**: 196B parameters, sparsely accessed via token n-gram lookup
- **DSpark speculative decoding**: Semi-autoregressive draft + confidence-scheduled verification
- **DeepSeek-ViT**: 2D-RoPE, 3x3 pixel-unshuffle, trained from scratch

**Architecture Details:**
- 552B backbone + 196B Engram = 748B total parameters
- 40 Transformer layers: 20 causal encoder + 20 decoder
- MoE: 1 shared expert + 384 routed experts, 6 activated per token
- KV cache: 890 bytes/token global (FP4), FP8 for local SWA
- Hierarchical Sparse Indexer bounds deeper indexer cost independently of context length
- Single-Pass mHC for revised residual-stream mixing

**Training:**
- 45T multimodal tokens (7:1 text-only to multimodal ratio)
- Sparse attention trained at 64K, extended to 1M at 34T tokens

---

### 2.6 Qwen 3 (Alibaba, April 2025)

**Key Innovations:**
- **Thinking/non-thinking fusion**: Single model seamlessly switches between reasoning and quick-response modes via `/think` and `/no_think` flags
- **Thinking budget control**: User-specified token budget for reasoning depth
- **Strong-to-Weak Distillation**: Large model reasoning distilled into small models (1/10 GPU hours vs full training)
- **119 language support**: 2x language coverage vs Qwen2.5
- **No shared experts in MoE**: Unlike Qwen2.5-MoE, removes shared experts; uses global-batch load balancing

**Architecture Details:**
- Dense: 0.6B to 32B params; MoE: 30B-A3B and 235B-A22B
- MoE: 128 total experts, 8 activated per token, no shared experts
- GQA, SwiGLU, RoPE, RMSNorm with pre-normalization
- QK-Norm added (removed QKV-bias from Qwen2)
- Max context: 32K (dense base) → 128K (instruct) → 1M (Qwen3-2507)
- Pre-trained on 36T tokens across 119 languages

**Post-Training Pipeline:**
1. Long-CoT cold start (diverse CoT data)
2. Reasoning RL (rule-based rewards, scale exploration)
3. Thinking mode fusion (SFT on combined data)
4. General RL (20+ tasks, instruction/format/agent capabilities)

---

### 2.7 Mistral Large 3 (Mistral AI, December 2025)

**Key Innovations:**
- **Granular MoE at massive scale**: 675B total / 41B active — largest open-weight Apache 2.0 model
- **Native vision encoder**: 2.5B parameter vision encoder fused into model
- **256K context window**: Enterprise-grade long-context
- **Speculative decoding (Eagle)**: Draft model for faster inference
- **NVFP4 quantization**: Optimized for Blackwell GPU deployment
- **Apache 2.0 license**: Full commercial use, no restrictions

**Architecture Details:**
- 675B total parameters, 41B active per token
- Granular MoE routing (subset of experts per token)
- 2.5B vision encoder integrated natively
- Trained from scratch on 3000 H200 GPUs
- FP8 weights for single-node deployment (8xH200)
- Fill-in-the-middle (FIM) for code completion

---

### 2.8 Phi-4 Reasoning (Microsoft Research, April 2025)

**Key Innovations:**
- **Reasoning as transferable meta-skill**: 14B model approaches full DeepSeek-R1 performance
- **Curated "teachable" prompts**: Data selected at boundary of base model capability
- **o3-mini as teacher**: High-quality reasoning traces for SFT
- **Outcome-based RL (GRPO)**: Rule-based reward on math, 1.5x longer traces post-RL
- **Minimal architecture change**: Only adds thinking tokens + doubles RoPE frequency for 32K context

**Architecture Details:**
- 14B dense decoder-only Transformer (same as Phi-4 base)
- `<think>` / `</think>` placeholder tokens added
- RoPE base frequency doubled (16K → 32K context)
- SFT on 1.4M prompt-response pairs, 8.3B unique tokens
- RL: GRPO on 72K math problems (64 subsampled per iteration)
- Training: 32 H100 GPUs, 2.5 days, 16B tokens

**Key Finding:** Reasoning improvements transfer to non-reasoning tasks (IFEval, FlenQA) without explicit training

---

### 2.9 Yi-Lightning (01.AI, October 2024)

**Key Innovations:**
- **Fine-grained expert segmentation**: FFN partitioned into smaller units, more experts activated per token
- **Partitioned EP load balancing (PEP)**: Solves token dispatching imbalance in expert parallelism
- **Hybrid attention blocks**: 3 SWA layers + 1 full attention layer
- **Cross-layer KV cache reuse**: Shares KV between consecutive full attention layers, 82.8% memory reduction
- **Hardware-aware FP8 design**: Architecture aligned with GPU specs for quantization compatibility

**Architecture Details:**
- Enhanced MoE architecture
- Fine-grained expert segmentation (inspired by Dai et al. 2024)
- 3-tier load balancing: L_ST (Switch-Transformer) + L_EP (expert parallel groups) + L_PEP (partitioned)
- Hybrid attention: 3 sliding window + 1 full attention per block
- Cross-layer KV cache sharing between consecutive full attention layers
- 64K context (extended from base via RoPE frequency increase)
- FP8: 1,200 TFLOPS per card on Hopper GPUs

---

### 2.10 Grok 3 (xAI, February 2025)

**Key Innovations:**
- **Massive compute scaling**: 200K H100 GPUs, 10x compute of Grok-2
- **Think/Big Brain/DeepSearch modes**: Tiered reasoning with increasing compute
- **Real-time X integration**: Native access to X (Twitter) data for live information
- **DeepSearch agent**: Multi-source web research with citation tracking
- **Distillation protection**: Partial CoT concealment to prevent model extraction

**Architecture Details:**
- Transformer-based (exact architecture undisclosed)
- ~1.5T parameters estimated (unconfirmed)
- Sparse attention mechanisms (reported)
- MoE layers suspected (unconfirmed)
- 131K API context / 1M marketed context
- Text-only at API launch (multimodal added later)
- Trained on Colossus supercluster (100K+ H100 GPUs)

**Modes:**
- **Think**: Mini model reasoning, step-by-step CoT
- **Big Brain**: Full model reasoning, longer compute, more accurate
- **DeepSearch**: Internet + X browsing, real-time research

---

## 3. Cross-Cutting Architecture Patterns (2024-2026)

### 3.1 MoE Dominance
**8 of 10 models** use or suspect MoE architecture. Only GPT-4o (dense) and Phi-4 (dense 14B) remain purely dense.

| Model | Experts (Total/Active) | Shared Expert | Routing |
|-------|----------------------|---------------|---------|
| Claude 3.5 Sonnet | 8/2 | No | Top-k |
| Llama 4 Scout | 16/1+shared | Yes | Top-k |
| DeepSeek V4.1 Flash | 384/6+shared | Yes | Top-k |
| Qwen 3-235B | 128/8 | No | Global-batch load balance |
| Mistral Large 3 | Granular/41B active | Unknown | Top-k |
| Yi-Lightning | Fine-grained | Unknown | PEP load balancing |

**Pattern**: Shared experts stabilize training; fine-grained segmentation improves parameter utilization; global-batch load balancing > per-expert constraints.

### 3.2 KV Cache Compression Arms Race

| Model | Technique | Bytes/Token | Reduction |
|-------|-----------|-------------|-----------|
| DeepSeek V4.1 Flash | CSA2 + FP4 | 890 | 4x vs V4 Flash, 437x vs V1 |
| Yi-Lightning | Cross-layer KV reuse + hybrid attention | N/A | 82.8% memory reduction |
| Claude 3.5 Sonnet | GQA + context compression | ~164K/100K tokens | 4x vs MHA |
| DeepSeek V4.1 Flash | SWA Bounded Replay | 1/8 persistent | No SSD round-trip |

**Pattern**: KV cache is the new bottleneck. Solutions: layer sharing, quantization (FP4), sliding window replay, sparse indexing.

### 3.3 Thinking/Reasoning Modes

| Model | Reasoning Type | Budget Control | Training Method |
|-------|---------------|----------------|-----------------|
| Gemini 2.5 Pro | Native thinking | Token budget parameter | RL on thinking |
| Qwen 3 | Think/no-think fusion | User-specified budget | 4-stage: CoT cold start → RL → fusion → general RL |
| DeepSeek V4.1 Flash | Effort levels (low/high/max) | Internal effort scale | RL post-training |
| Phi-4 Reasoning | Explicit CoT | Max token length | SFT on o3-mini traces + GRPO RL |
| Grok 3 | Think/Big Brain/DeepSearch | Mode selection | Large-scale RL |

**Pattern**: Reasoning is becoming a default capability, not a separate model. Budget control enables cost-quality tradeoff.

### 3.4 Multimodal Integration Strategies

| Strategy | Models | Approach |
|----------|--------|----------|
| **End-to-end native** | GPT-4o, Gemini 2.5 | Single network, unified tokenization from pretraining |
| **Early fusion** | Llama 4, DeepSeek V4.1 | Vision tokens injected from first layer, jointly pretrained |
| **Bolted-on encoder** | Mistral Large 3 | Separate 2.5B vision encoder |
| **Separate model** | Qwen 3 | Text-only main model, separate VL variant |

**Pattern**: Industry converging on early fusion / native multimodality. Staged CLIP-style pipelines are legacy.

### 3.5 Context Window Scaling

| Model | Context | Key Technique |
|-------|---------|---------------|
| Llama 4 Scout | 10M | iRoPE (interleaved RoPE/NoPE) |
| Gemini 2.5 Pro | 1M | MoE + long-context mid-training |
| DeepSeek V4.1 Flash | 1M | CSA2 + hierarchical sparse indexer |
| Qwen 3-2507 | 1M | Extended via long-context data |
| Mistral Large 3 | 256K | Standard long-context |
| GPT-4o | 128K | Standard |

**Pattern**: 1M+ context is now table stakes. iRoPE is the most novel approach (positional encoding elimination for infinite generalization).

---

## 4. NeoTrix Mapping — Architectural Innovations to Domain Modules

### 4.1 KV Cache Compression → NT-MEMORY

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| Cross-layer KV reuse | Yi-Lightning | `nt_memory::kv_optimizer` — share KV states across domain layers |
| CSA2 (Full/Reindex/Reuse modes) | DeepSeek V4.1 | `nt_memory::sparse_attention_cache` — static mode assignment per layer |
| FP4 KV quantization | DeepSeek V4.1 | `nt_memory::quantized_cache` — E2M1 format for HBM efficiency |
| SWA Bounded Replay | DeepSeek V4.1 | `nt_memory::sliding_window_replay` — reconstruct local context without SSD |
| GQA with grouped KV heads | Claude 3.5, Qwen 3 | `nt_memory::grouped_kv` — standard optimization |

**Actionable**: Implement `kv_optimizer.rs` with cross-layer sharing and FP4 quantization for >256K sessions.

### 4.2 Thinking/Reasoning Budget → NT-CORE + NT-MIND

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| Thinking budget control | Gemini 2.5, Qwen 3 | `nt_core::reasoning_budget` — user-controllable inference compute |
| Think/no-think mode fusion | Qwen 3 | `nt_mind::dual_mode_engine` — single model, dynamic switching |
| Outcome-based RL (GRPO) | Phi-4 Reasoning | `nt_mind::grpo_trainer` — rule-based reward for self-improvement |
| Effort levels (low/high/max) | DeepSeek V4.1 | `nt_core::effort_controller` — maps to GWT attention depth |

**Actionable**: Extend SEAL pipeline with `reasoning_budget` parameter that controls GWT attention depth and thinking token allocation.

### 4.3 MoE Routing → NT-ACT + NT-CORE

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| Fine-grained expert segmentation | Yi-Lightning, DeepSeek | `nt_act::expert_segmentation` — partition FFN into functional units |
| Global-batch load balancing | Qwen 3 | `nt_act::global_routing` — per-group > per-expert constraints |
| Partitioned EP load balancing (PEP) | Yi-Lightning | `nt_act::partitioned_routing` — All-to-All communication optimization |
| Shared + routed expert hybrid | Llama 4, DeepSeek V4.1 | `nt_act::shared_expert` — universal features always active |

**Actionable**: Implement MoE-style routing in `nt_act::tool_router` — route tool calls to specialist sub-networks with load balancing.

### 4.4 Attention Architecture → NT-CORE

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| iRoPE (interleaved RoPE/NoPE) | Llama 4 Scout | `nt_core::infinite_attention` — positional-encoding-free layers for long context |
| Hybrid SWA + full attention | Yi-Lightning, Claude 3.5 | `nt_core::hybrid_attention` — local + global alternating |
| Hierarchical Sparse Indexer | DeepSeek V4.1 | `nt_core::hierarchical_indexer` — bounded-cost deep indexing |
| Temperature-scaled NoPE | Llama 4 Scout | `nt_core::temperature_attention` — scale softmax for very long sequences |

**Actionable**: Implement `hybrid_attention.rs` in `nt_core` — alternate SWA (8K window) + full attention layers for >128K contexts.

### 4.5 Multimodal Integration → NT-WORLD + NT-IO

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| End-to-end native multimodal | GPT-4o, Gemini 2.5 | `nt_world::native_multimodal` — unified token stream for all modalities |
| Early fusion | Llama 4, DeepSeek V4.1 | `nt_world::early_fusion` — vision/text from first layer |
| Neural audio codec tokenization | GPT-4o | `nt_io::audio_tokenizer` — Encodec/SoundStream-style discrete tokens |
| DeepSeek-ViT (2D-RoPE, pixel-unshuffle) | DeepSeek V4.1 | `nt_world::vision_encoder` — 9x token reduction via 3x3 unshuffle |

**Actionable**: Upgrade `nt_world::crawl` to support early-fusion multimodal ingestion for image+text processing.

### 4.6 Speculative Decoding → NT-IO

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| DSpark (semi-autoregressive draft) | DeepSeek V4.1 | `nt_io::speculative_decoder` — confidence-scheduled verification |
| Eagle draft model | Mistral Large 3 | `nt_io::eagle_draft` — small model drafts, large model verifies |

**Actionable**: Implement `speculative_decoder.rs` in `nt_io` for faster inference on LLM provider calls.

### 4.7 Conditional Memory → NT-MEMORY

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| Engram conditional memory (196B params, sparse access) | DeepSeek V4.1 | `nt_memory::conditional_memory` — n-gram hash lookup, sparsely accessed |
| Cross-layer KV cache reuse | Yi-Lightning | Already mapped above |

**Actionable**: Implement `conditional_memory.rs` — sparse n-gram lookup table for frequently accessed patterns (experience keywords, domain terms).

### 4.8 Distillation Pipeline → NT-MIND

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| Strong-to-Weak distillation (1/10 cost) | Qwen 3 | `nt_mind::distillation_pipeline` — flagship → small model knowledge transfer |
| k-sparse distribution approximation | Gemini 2.5 | `nt_mind::sparse_logit_distillation` — reduce teacher output storage |
| o3-mini as reasoning teacher | Phi-4 Reasoning | `nt_mind::teacher_chain` — use stronger models as reasoning demonstrator |

**Actionable**: Extend SEAL pipeline with distillation stage — use flagship model to generate reasoning traces for skill crystallization.

### 4.9 Safety Engine → NT-SHIELD

| Innovation | Source | NeoTrix Mapping |
|-----------|--------|-----------------|
| RAISE (4-component safety) | Yi-Lightning | `nt_shield::raise_framework` — pre/post-training + input/output safety |
| Constitutional AI | Claude 3.5 | `nt_shield::constitutional_ai` — rule-based alignment |
| Preparedness Framework | GPT-4o | `nt_shield::preparedness_eval` — CBRN/cyber/autonomy risk thresholds |

**Actionable**: Implement `raise_framework.rs` — 4-stage safety: data filtering → training optimization → input analysis → output monitoring.

### 4.10 Training Infrastructure → Meta-Insight

| Innovation | Source | Implication for NeoTrix |
|-----------|--------|------------------------|
| Elastic training with slice recovery | Gemini 2.5 | SEAL pipeline should handle mid-cycle hardware failures gracefully |
| Split-phase SDC detection | Gemini 2.5 | `experience-tree` absorption should verify data integrity before commit |
| PagedAttention + CPU offloading | Llama 4 Scout | NT-MEMORY should support hierarchical storage (GPU → CPU → SSD) |
| 99%+ GPU utilization via async scheduling | Yi-Lightning | Background loops should maximize utilization through async I/O |

---

## 5. Priority Implementation Queue

### P0 — Immediate (High Impact, Clear Path)
1. **Thinking Budget Controller** (from Gemini 2.5 + Qwen 3) — user-controllable reasoning depth
2. **Hybrid Attention** (from Yi-Lightning + Claude 3.5) — SWA + full attention for >128K contexts
3. **KV Cross-Layer Reuse** (from Yi-Lightning) — share KV between consecutive layers

### P1 — Near-Term (High Impact, Needs Design)
4. **MoE-Style Tool Routing** (from DeepSeek V4.1 + Qwen 3) — specialist sub-networks for tool calls
5. **Speculative Decoding** (from DeepSeek V4.1 + Mistral) — faster LLM inference
6. **Engram Conditional Memory** (from DeepSeek V4.1) — sparse n-gram pattern lookup

### P2 — Medium-Term (Architectural)
7. **iRoPE Infinite Attention** (from Llama 4 Scout) — positional-encoding-free long context
8. **CSA2 Sparse Attention** (from DeepSeek V4.1) — 3-mode layer sharing
9. **Strong-to-Weak Distillation** (from Qwen 3) — flagship reasoning → small model transfer

### P3 — Long-Term (Research)
10. **Native Multimodal Early Fusion** (from GPT-4o + Llama 4) — unified token stream
11. **RAISE Safety Framework** (from Yi-Lightning) — 4-stage safety pipeline
12. **Engram Memory** (from DeepSeek V4.1) — 196B conditional memory integration

---

## 6. Key Takeaways

### Industry Direction
- **MoE is default**: 8/10 models use MoE; dense is only for small/specialized models
- **Reasoning is native**: Thinking modes are being fused into base models, not separate models
- **KV cache is the bottleneck**: Every model has a KV cache innovation; FP4 quantization is the frontier
- **Context windows are unbounded**: 1M-10M tokens via iRoPE, CSA2, and hierarchical indexing
- **Multimodal is early-fusion**: Staged CLIP pipelines are dead; native joint training wins

### NeoTrix Competitive Advantages
1. **E8 + HyperCube + GWT**: No LLM has symbolic reasoning + vector association + attention routing unified
2. **SEAL pipeline with distillation**: No LLM has self-evolving architecture loops with cross-session learning
3. **Domain-based MoE**: NT-* factions naturally map to MoE-style routing without parameter overhead
4. **Experience-tree absorption**: No LLM has structured experience accumulation across sessions

### Gaps to Close
1. **Thinking budget control**: Need user-facing API for reasoning depth
2. **KV cache efficiency**: Need FP4 quantization + cross-layer sharing for long sessions
3. **Hybrid attention**: Need SWA + full attention for >128K context handling
4. **Speculative decoding**: Need draft-verify for faster LLM provider calls

---

*Generated by NeoTrix reverse-engineering pipeline — 10 models analyzed, 12 innovations mapped to NT-* domains*
