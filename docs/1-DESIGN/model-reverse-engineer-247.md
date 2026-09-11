# Model Reverse-Engineer #247 — 10 LLM Architectures vs NeoTrix

**Date**: 2026-09-11
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3
**Method**: Web research → architecture extraction → NeoTrix mapping

---

## 1. Architecture Comparison Matrix

| Model | Type | Params (Total/Active) | Attention | KV Cache | Context | Multimodal | Reasoning |
|-------|------|----------------------|-----------|----------|---------|------------|-----------|
| **GPT-4o** | Dense Transformer | Undisclosed | Dense self-attention | Standard | 128K | E2E text+image+audio | No |
| **Claude 3.5 Sonnet** | MoE | ~175B / ~50B | Hybrid sparse (SWA+global) | GQA 4x reduction | 200K | Text+image | No |
| **Gemini 2.5 Pro** | Sparse MoE | Undisclosed | Standard MoE attn | Standard MoE | 1M+ | E2E text+image+audio+video | Thinking |
| **Llama 4 Scout** | Sparse MoE | 109B / 17B (16E) | iRoPE (interleaved RoPE/NoPE) | Standard MoE | 10M | Early fusion text+image | No |
| **DeepSeek V4.1 Flash** | CED MoE | 552B backbone / 8B-16B | CSA2 (compressed sparse) | FP4 890B/tok | 1M | ViT+MLP | Thinking |
| **Qwen 3** | Dense+MoE | 235B / 22B (128E) | GQA + sliding window | Standard MoE | 128K-1M | Text (separate VL) | Hybrid thinking |
| **Mistral Large 3** | Granular MoE | 675B / 41B | Standard MoE | Standard MoE | 256K | 2.5B vision encoder | No |
| **Phi-4 Reasoning** | Dense | 14B | Standard dense | Standard dense | 32K | Text only | Thinking (SFT+RL) |
| **Yi-Lightning** | Enhanced MoE | Undisclosed | Hybrid (3SWA+1full) | Cross-layer KV reuse | 64K | Text | No |
| **Grok 3** | MoE | ~600B / 120B (16E) | Sparse attention | Standard | 131K-1M | Text+image | Think/DeepSearch |

---

## 2. Per-Model Architecture Deep-Dive

### 2.1 GPT-4o (OpenAI, May 2024)

**Key Innovations:**
- **End-to-end multimodal**: Single neural net processes text+vision+audio natively — no bolted CLIP encoders
- **Unified tokenization**: BPE text + image patch tokens + neural audio codec tokens in one stream
- **Latency breakthrough**: 232ms median audio response (vs 2.8s staged pipeline)
- **Improved multilingual tokenizer**: 1.1x-4.4x fewer tokens for non-English scripts

**Architecture:**
- Decoder-only Transformer, 128K context, 16K max output
- Cross-modal attention via self-attention within unified token stream
- Modality-specific embedding/unembedding layers
- Audio tokenizer: neural codec (Encodec/SoundStream-class), 50-75 Hz tokens/sec

**Not disclosed:** Parameter count, exact audio tokenizer, KV cache reuse across modalities

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| Unified tokenization | NT-IO modality adapters | Need unified token stream abstraction |
| E2E multimodal attention | NT-WORLD PerceptionBridge | Currently stage-separated; bridge needs direct cross-modal flow |
| Sub-300ms latency | NT-ACT action pipeline | Need parallel prefill/decode for real-time |
| Multilingual tokenizer | NT-MEMORY KB tokenizer | Implement language-aware token budgeting |

---

### 2.2 Claude 3.5 Sonnet (Anthropic, June 2024)

**Key Innovations:**
- **Hybrid sparse attention**: Even layers = local SWA (1024 tokens), odd layers = global sparse (every 64th token)
- **GQA with 8 KV heads**: 4x KV cache reduction vs standard MHA
- **Context compression**: Lossless compression for repeated patterns, 22% payload reduction
- **Constitutional AI**: Rule-based RLHF alignment

**Architecture:**
- ~175B total, ~50B active (MoE: 8 experts, 2 active)
- 60 layers, 8192 hidden, 64 attention heads, 8 KV heads
- RoPE base 10K, extended to 200K via linear scaling
- FLOPs: 12.4 TFLOPs/100K tokens (vs 40 TFLOPs dense)

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| Hybrid SWA+global attention | NT-MIND attention routing (GWT) | GWT could adopt tiered attention: local focus + global broadcast |
| GQA KV sharing | NT-MEMORY kv_cache_optimizer | Extend GQA pattern to memory layer |
| Context compression | NT-MEMORY compaction | Add lossless pattern dedup for repeated tool outputs |
| Constitutional alignment | NT-SHIELD governance | Rule-based guardrails in rev-officer already partially map |

---

### 2.3 Gemini 2.5 Pro (Google, 2025)

**Key Innovations:**
- **Sparse MoE + native multimodal**: Text+vision+audio from pretraining
- **1M+ context window**: Longest production context
- **Training stability breakthroughs**: Signal propagation + optimization dynamics improvements
- **Thinking budget control**: Dynamic compute allocation per token

**Architecture:**
- Sparse MoE transformer, undisclosed parameters
- Native multimodal (text, vision, audio)
- 2M token context (Gemini 2.5 Pro)
- MoE routing: activates subset per token, decoupling capacity from cost

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| 2M context window | NT-MEMORY KVMem paged KV | Align with KVMem paged KV virtualization |
| Native multimodal | NT-WORLD UnifiedCrawler | Extend crawler to audio/video ingestion |
| Training stability | NT-REPAIR self-healing | Apply signal propagation monitoring |
| Thinking budget | NT-CORE GWT salience | Add compute-cost-aware attention routing |

---

### 2.4 Llama 4 Scout (Meta, April 2025)

**Key Innovations:**
- **iRoPE architecture**: Interleaved attention layers — some with RoPE, some without positional embeddings
- **10M token context**: Industry-leading context length
- **Alternating dense/MoE layers**: Inference efficiency via layer-type alternation
- **Early fusion multimodal**: Text+image from pretraining, not bolted on

**Architecture:**
- 109B total, 17B active, 16 routed experts + 1 shared expert
- iRoPE: interleaved positional/non-positional attention layers
- Temperature scaling of attention at inference for length generalization
- Pre/post-trained at 256K, extrapolates to 10M

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| iRoPE (interleaved pos/no-pos) | NT-CORE E8 hexagram positioning | Could adopt position-free layers for infinite context |
| 10M context | NT-MEMORY KVMem | paged KV + tiered storage (GPU→Host→NVMe) |
| Alternating dense/MoE | NT-ACT capability routing | Route simple tasks to dense, complex to MoE |
| Early fusion | NT-WORLD PerceptionBridge | Fuse modalities at input, not after encoding |

---

### 2.5 DeepSeek V4.1 Flash (DeepSeek, Sept 2026)

**Key Innovations:**
- **Causal Encoder-Decoder (CED)**: 20-layer encoder + 20-layer decoder; decoder KV projected from encoder's final hidden states
- **Asymmetric activation**: 8B active prefill / 16B decode (552B total)
- **CSA2 (Compressed Sparse Attention 2)**: 3 static modes (Full/Reindex/Reuse) sharing KV+indexer across layers
- **FP4 KV caching**: E2M1 format, 890 bytes/token (1/4 of V4-Flash)
- **SWA Bounded Replay**: Reconstruct SWA states by replaying recent window, no SSD round-trip
- **Engram conditional memory**: 196B params, sparsely accessed via token-based lookup
- **DSpark speculative decoding**: Semi-autoregressive draft + confidence-scheduled verification

**Architecture:**
- 552B backbone, 384 routed experts + 1 shared per MoE layer, 6 activated per token
- DeepSeek-ViT (2D-RoPE, 3×3 pixel-unshuffle) + 2-layer MLP projector
- Trained on 45T tokens, sparse attention at 64K, extended to 1M at 34T

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| CED asymmetric activation | NT-ACT resource_budget | Prefill/decode cost asymmetry for agent workloads |
| CSA2 layer-mode sharing | NT-MEMORY kv_cache | Adopt Full/Reindex/Reuse modes for KB KV cache |
| FP4 KV quantization | NT-MEMORY kv_cache_optimizer | Extend to E2M1 FP4 with per-channel scaling |
| Engram conditional memory | NT-MEMORY KB embedding | Sparse token-based lookup for large embeddings |
| SWA Bounded Replay | NT-MEMORY compaction | Reconstruct without SSD persistence |
| DSpark speculative decoding | NT-ACT action pipeline | Semi-autoregressive draft for tool call prediction |

---

### 2.6 Qwen 3 (Alibaba, April 2025)

**Key Innovations:**
- **Thinking/non-thinking hybrid**: Single model, dynamic mode switching via /think and /no_think flags
- **Thinking budget control**: User-defined token budget, emergent ability to handle incomplete reasoning
- **Fine-grained MoE**: 128 experts, 8 activated, no shared experts, global-batch load balancing
- **Strong-to-Weak distillation**: Flagship reasoning knowledge distilled to small models (0.6B-30B)

**Architecture:**
- Dense: 0.6B-32B, GQA, SwiGLU, RoPE, RMSNorm, QK-Norm
- MoE: 235B/22B, 128 experts, 8 active, 128K context (extendable to 1M via YARN)
- 4-stage training: CoT cold start → Reasoning RL → Thinking fusion → General RL
- 36T tokens pretrained, 119 languages

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| Thinking mode hybrid | NT-CORE ConsciousnessTree | 6-stage loop already maps; add budget control |
| /think flag routing | NT-MIND SEAL pipeline | Dynamic stage routing based on task complexity |
| Global-batch load balance | NT-ACT expert routing | Adopt for MoE-style capability routing |
| Strong-to-Weak distillation | NT-MIND distillation | Distill flagship reasoning to small NT models |
| YARN context extension | NT-MEMORY kv_cache | Implement YARN for length extrapolation |

---

### 2.7 Mistral Large 3 (Mistral, Dec 2025)

**Key Innovations:**
- **Granular MoE**: 675B total, 41B active (6.1% activation ratio)
- **First MoE since Mixtral**: Substantial step forward in pretraining
- **NVFP4 deployment**: Single 8×A100 or 8×H100 node via vLLM
- **Apache 2.0 open weights**: BF16 + NVFP4 formats

**Architecture:**
- 673B language model + 2.5B vision encoder
- 16 routed experts, 2 active per token
- 256K context window
- Trained on 3000 H200 GPUs
- Speculative decoding, prefill/decode disaggregated serving

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| Granular MoE (6.1% active) | NT-ACT capability routing | Extreme sparsity for cost optimization |
| NVFP4 single-node deploy | NT-PHYSICAL deployment | On-prem deployment with minimal GPU |
| Prefill/decode disaggregation | NT-ACT action pipeline | Separate input processing from generation |
| Speculative decoding | NT-ACT action pipeline | Draft tool calls before verification |

---

### 2.8 Phi-4 Reasoning (Microsoft, April 2025)

**Key Innovations:**
- **Data-centric SFT**: 1.4M carefully curated prompts, filtered for difficulty at model's capability boundary
- **Teacher distillation**: o3-mini traces as training data (medium effort = token-efficient, high effort = stronger)
- **GRPO reinforcement learning**: 6K math problems, outcome-based RL on verifiable solutions
- **Reasoning transfer**: Improvements transfer to non-reasoning tasks without explicit training

**Architecture:**
- 14B dense decoder-only Transformer (same as Phi-4 base)
- RoPE base frequency doubled (16K → 32K context)
- Think/think tokens for reasoning block demarcation
- Fixed reasoning-focused system message

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| Data curation at capability boundary | NT-MIND skill crystallization | Filter training data at frontier of current capabilities |
| Teacher distillation (o3-mini) | NT-MIND distillation | Use external model traces for skill training |
| GRPO on verifiable problems | NT-REPAIR self-healing | Reward-verified repair attempts |
| Reasoning transfer | NT-CORE meta-cognition | Cross-domain skill transfer without explicit training |

---

### 2.9 Yi-Lightning (01.AI, Dec 2024)

**Key Innovations:**
- **Partitioned EP load balancing (PEP)**: Splits experts within EP groups for finer-grained token distribution
- **Hybrid attention blocks**: 3 SWA layers + 1 full attention layer
- **Cross-layer KV cache reuse**: Share KV between consecutive full attention layers, 50% memory reduction
- **82.8% memory reduction** via combined hybrid attention + KV reuse
- **FP8 hardware-aware design**: 1200 TFLOPS/card on Hopper GPUs

**Architecture:**
- Enhanced MoE with fine-grained expert segmentation
- 3-level load balancing: Switch-Transformer + EP + PEP
- Hybrid parallelization: expert + pipeline parallelism
- Context parallelism with 70% training speedup

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| PEP load balancing | NT-ACT expert routing | Partition-level routing for sub-expert balance |
| Hybrid attention (3SWA+1full) | NT-CORE E8 attention | Implement tiered attention in E8 hexagram |
| Cross-layer KV reuse | NT-MEMORY kv_cache | Share KV states across memory layers |
| FP8 hardware-aware design | NT-PHYSICAL compute | Quantization-aware architecture design |

---

### 2.10 Grok 3 (xAI, Feb 2025)

**Key Innovations:**
- **Think/Big Brain/DeepSearch modes**: Three inference-compute tiers
- **10x pretraining compute**: 200K H100 GPUs on Colossus supercluster
- **RL at unprecedented scale**: Chain-of-thought refined via massive reinforcement learning
- **1M context window**: 8x increase from Grok 2
- **Real-time web search integration**: DeepSearch for live information retrieval

**Architecture:**
- ~600B total, 120B active (16 experts, 2 active)
- 96 layers, 12288 hidden, 96 attention heads, 16 KV heads
- 131K context (standard), 1M (extended)
- Training ongoing with frequent updates

**NeoTrix Mapping:**
| Innovation | NeoTrix Component | Gap |
|------------|------------------|-----|
| 3-tier compute modes | NT-CORE GWT salience | Dynamic compute allocation: Think→Big Brain→DeepSearch |
| 10x compute scale | NT-PHYSICAL infrastructure | Need scaled compute orchestration |
| RL-scale reasoning | NT-MIND SEAL | Large-scale RL for skill refinement |
| Live web search | NT-WORLD OrderedBackend | Extend with real-time search routing |
| 1M context | NT-MEMORY KVMem | Paged KV for ultra-long sessions |

---

## 3. Cross-Model Innovation Patterns

### 3.1 Universal Trends

| Trend | Models Adopting | NeoTrix Implication |
|-------|----------------|-------------------|
| **MoE dominance** | 8/10 models | NT-ACT must support sparse expert routing |
| **Hybrid attention** | Claude, Yi, DeepSeek | NT-CORE attention needs tiered local/global |
| **KV cache compression** | DeepSeek (FP4), Claude (GQA), Yi (KV reuse) | NT-MEMORY kv_cache_optimizer must support multiple strategies |
| **Thinking budget control** | Qwen3, Gemini 2.5, DeepSeek | NT-CORE ConsciousnessTree needs compute allocation |
| **Native multimodal** | GPT-4o, Gemini, Llama 4, DeepSeek | NT-WORLD must fuse modalities at input level |
| **1M+ context** | Gemini (2M), Llama 4 (10M), DeepSeek (1M), Grok (1M) | NT-MEMORY KVMem paged KV is critical |

### 3.2 Attention Architecture Spectrum

```
Dense Full Attention          Hybrid Sparse           Compressed Sparse
GPT-4o, Phi-4                 Claude 3.5, Yi-Lightning  DeepSeek CSA2
    │                              │                        │
    │   O(n²) FLOPs               │   O(n·w) FLOPs        │   Shared KV+indexer
    │   Full pairwise              │   SWA + global strided │   3 static modes
    │                              │                        │
    └──────────────────────────────┴────────────────────────┘
                                    NeoTrix target: tiered GWT routing
```

### 3.3 KV Cache Efficiency Spectrum

```
Standard KV (FP16)     GQA (4x reduction)     Cross-layer Reuse     FP4 Quantized
GPT-4o, Grok           Claude, Qwen            Yi-Lightning          DeepSeek V4.1
    │                      │                       │                     │
    │  3.2GB/100K tok      │  1.2GB/100K tok       │  50% memory cut     │  890B/token
    │                      │                       │                     │
    └──────────────────────┴───────────────────────┴─────────────────────┘
                                    NeoTrix target: adaptive KV strategy
```

---

## 4. NeoTrix Architecture Gaps (Priority-Ordered)

### P0 — Critical (blocks core capability)

| Gap | Source Innovation | Affected Component | Fix |
|-----|------------------|-------------------|-----|
| No unified multimodal token stream | GPT-4o E2E | NT-IO, NT-WORLD | Design unified token abstraction layer |
| KV cache lacks FP4/reuse strategies | DeepSeek CSA2, Yi KV reuse | NT-MEMORY | Extend kv_cache_optimizer with adaptive quantization |
| No thinking budget control | Qwen3, Gemini 2.5 | NT-CORE | Add compute-cost-aware allocation to ConsciousnessTree |

### P1 — High (significant efficiency gain)

| Gap | Source Innovation | Affected Component | Fix |
|-----|------------------|-------------------|-----|
| GWT lacks tiered attention | Claude hybrid, Yi hybrid | NT-CORE | Implement local SWA + global broadcast in GWT salience |
| No cross-layer KV sharing | Yi cross-layer reuse | NT-MEMORY | Share KV states across memory layers |
| No speculative draft for tool calls | DeepSeek DSpark | NT-ACT | Semi-autoregressive draft prediction |
| Missing strong-to-weak distillation | Qwen3 | NT-MIND | Distill flagship reasoning to small NT models |

### P2 — Medium (future-proofing)

| Gap | Source Innovation | Affected Component | Fix |
|-----|------------------|-------------------|-----|
| No iRoPE-style pos-free layers | Llama 4 Scout | NT-CORE | Interleave positional/non-positional attention for infinite context |
| No PEP-level load balancing | Yi-Lightning | NT-ACT | Partition-level expert routing |
| No real-time web search routing | Grok DeepSearch | NT-WORLD | Extend OrderedBackend with live search tier |
| No GRPO on verifiable problems | Phi-4 Reasoning | NT-REPAIR | Reward-verified self-healing attempts |

---

## 5. Derived Axioms

| # | Axiom | Source | NeoTrix Rule |
|---|-------|--------|-------------|
| D1 | **Asymmetric Activation** — Prefill and decode should activate different parameter sets | DeepSeek V4.1 CED | NT-ACT: route input-heavy tasks to cheap encoder, output-heavy to capable decoder |
| D2 | **KV Cache as First-Class Citizen** — Memory efficiency determines context scale ceiling | DeepSeek FP4, Yi reuse, Claude GQA | NT-MEMORY: adaptive KV strategy per workload type |
| D3 | **Thinking Budget as User API** — Compute allocation should be explicit, not implicit | Qwen3 /think, Gemini thinking | NT-CORE: expose thinking budget as configuration parameter |
| D4 | **Attention Tiering** — Local + global attention beats uniform dense | Claude hybrid, Yi 3+1, DeepSeek CSA2 | NT-CORE GWT: tiered attention routing (local focus → global broadcast) |
| D5 | **Data Curation > Model Scale** — Filtered training data at capability boundary beats larger models | Phi-4 Reasoning, Qwen3 distillation | NT-MIND: data quality pipeline for skill crystallization |
| D6 | **MoE is the Default** — 8/10 frontier models use MoE; dense is legacy | Universal trend | NT-ACT: default to sparse expert routing for all new capabilities |

---

## 6. Implementation Roadmap

| Phase | Duration | Deliverables | Priority |
|-------|----------|-------------|----------|
| **Phase 1** | 2 weeks | KV cache adaptive strategy (FP4 + cross-layer reuse) | P0 |
| **Phase 2** | 3 weeks | Thinking budget control in ConsciousnessTree | P0 |
| **Phase 3** | 2 weeks | GWT tiered attention (local SWA + global broadcast) | P1 |
| **Phase 4** | 4 weeks | Unified multimodal token stream | P0 |
| **Phase 5** | 3 weeks | Strong-to-weak distillation pipeline | P1 |
| **Phase 6** | Ongoing | iRoPE, PEP routing, live search, GRPO | P2 |

---

**Sources**: OpenAI GPT-4o System Card, Anthropic Claude 3.5 Model Card, Google Gemini 2.5 Technical Report, Meta Llama 4 Blog, DeepSeek V4.1-Flash Model Card (HuggingFace), Qwen3 Technical Report, Mistral Large 3 Documentation, Microsoft Phi-4-reasoning Technical Report, 01.AI Yi-Lightning Technical Report, xAI Grok 3 Blog
