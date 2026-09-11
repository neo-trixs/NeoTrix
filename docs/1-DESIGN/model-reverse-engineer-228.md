# Model Reverse Engineering #228 — DeepSeek V4.1 Flash

**Date**: 2026-09-11
**Model**: DeepSeek-V4.1-Flash (552B backbone, 196B Engram)
**Source**: HuggingFace `deepseek-ai/DeepSeek-V4.1-Flash` + Tech Report
**NeoTrix Mapping**: KV cache compression → NT-IO/NT-CORE absorption

---

## 1. Model Overview

| Spec | Value |
|------|-------|
| **Total Params** | 552B backbone + 196B Engram conditional memory |
| **Active (Prefill)** | 8B per token |
| **Active (Decode)** | 16B per token |
| **Architecture** | Causal Encoder-Decoder (CED): 20-layer encoder + 20-layer decoder |
| **Context** | 1M tokens (YaRN ×16 from 65K base) |
| **Max Output** | 384K tokens |
| **MoE** | 384 routed experts + 1 shared, 6 activated per token |
| **Vision** | DeepSeek-ViT (2D-RoPE, 3×3 pixel-unshuffle) |
| **KV Cache** | 890 bytes/token global (FP4 E2M1) |
| **License** | MIT |
| **Training** | 45T tokens, 7:1 text:multimodal ratio |
| **Release** | 2026-09-10 |

---

## 2. Architecture Innovations

### 2.1 Causal Encoder-Decoder (CED) — Asymmetric Activation

The core architectural break from V3/V4. The 40-layer Transformer is split:
- **20-layer causal encoder**: processes input tokens, builds compressed representations
- **20-layer decoder**: generates output tokens, KV cache projected from encoder's final hidden states

**Key insight**: Decoder's global KV cache is NOT derived from each decoder layer's own hidden states — it's projected once from the encoder's output. This enables:
- **8B active parameters during prefill** (reading long prompts)
- **16B active parameters during decode** (generating tokens)
- Input-heavy agent workloads get the cheap half; generation gets the expensive half

**Why it matters**: Agent tasks = long context in, short output out. CED directly reduces cost on the expensive side of the ledger.

### 2.2 Compressed Sparse Attention 2 (CSA2)

Three static modes per attention layer:

| Mode | Function | KV State |
|------|----------|----------|
| **Full** | Computes main KV + indexer keys + Top-K sparse positions | Own KV |
| **Reindex** | Reuses Full layer's KV + indexer keys, rescans with own query | Shared KV |
| **Reuse** | Reuses both KV and Top-K indices from earlier layer | Shared KV + indices |

**Hierarchical Sparse Indexer** (decoder only):
- First Full Mode layer constructs candidate pool: 2,048 blocks × 8 positions = 16,384 positions
- Later indexing layers restricted to this pool
- Indexing cost independent of context length at 1M tokens

**Impact**: 256× context extension (4K → 1M) raises decode compute by only ~25%.

### 2.3 FP4 KV Cache — 890 Bytes/Token

- Main KV stored in FP4 (E2M1 format)
- One E4M3 scale factor per 16 channels
- **1/4** of DeepSeek-V4-Flash footprint
- **437×** smaller than original DeepSeek-V1

### 2.4 SWA Bounded Replay

Sliding Window Attention normally requires persisting KV state to SSD for context reconstruction. V4.1-Flash:
- Replays only the most recent `n_win` tokens (128-token window)
- No SSD round-trip needed
- Persistent KV cache footprint: **1/8** of V4-Flash
- SWA KV lives in 10% of host DRAM with minutes TTL
- Global KV guaranteed ≥72 hours in persistent cache

### 2.5 Engram Conditional Memory

- 196B parameters across two conditional-memory modules (layers 1 and 14)
- Token n-grams (lengths 2, 3, 4) with 8 hash heads
- Context-aware gating + host-memory prefetch over RDMA
- **Sparse access**: only loaded when needed, not every forward pass
- Separates memorization from dense model computation

### 2.6 DSpark Speculative Decoding

- 3 extra Transformer blocks draft 5 tokens semi-autoregressively
- Confidence head estimates how many drafted tokens survive verification
- Scheduler uses confidence + measured throughput to select verification length
- Separately trained (not alongside backbone pre-training like V3 MTP)

### 2.7 Single-Pass mHC (Hyper-Connections)

- Residual stream carried as 4 parallel copies
- Each sublayer derives pre/post/combine coefficients from the stream
- Combine matrix doubly stochastic via 20 Sinkhorn iterations
- Mega-mHC kernel halves activation memory traffic vs original 4-kernel version

### 2.8 MoE Configuration

- **384 routed experts + 1 shared** per MoE layer
- **6 routed experts activated** per token (~1.6% of routed pool)
- `sqrtsoftplus` scoring function (replaces V3's Sigmoid)
- Routed scaling factor: 1.5
- No auxiliary loss — bias-only load balancing (from V3)
- Expert weights: MXFP4; everything else: MXFP8

---

## 3. Training Method

### 3.1 Pre-training

| Phase | Detail |
|-------|--------|
| **Corpus** | 45T tokens, multimodal from day 1 |
| **Text:MM ratio** | 7:1 (after overlap replacement + dedup) |
| **Sparse attention** | Trained at 64K sequence length, no dense warmup |
| **Context extension** | 1M at 34T token mark via YaRN (factor 16) |
| **Optimizer** | Muon (orthogonalized momentum, replaces AdamW) |
| **Vision pretraining** | ~47B image-text pairs (contrastive), then 236B AR tokens |
| **Vision encoder** | DeepSeek-ViT trained from scratch, 2D-RoPE |

### 3.2 Post-training — SFT → RL → On-Policy Distillation

**No algorithmic innovations** — all gains from data pipeline:

1. **SFT**: Supervised fine-tuning on curated instruction data
2. **RL**: Large-scale automated synthesis of agent tasks and environments
   - Progressive scaling of data, tasks, and rollouts
   - Verifiable rewards
   - GRPO (Group Relative Policy Optimization)
3. **On-Policy Distillation (OPD)**: Domain expert traces → unified model

### 3.3 Continuously Controllable Reasoning Effort

- Integer setting 1–100 (low=50, high=75, max=100)
- Trades inference cost for accuracy
- No separate model variants needed

---

## 4. Inference Optimizations

| Optimization | Mechanism | Impact |
|--------------|-----------|--------|
| **CED Asymmetry** | 8B prefill / 16B decode | ~50% cost reduction for long-prompt tasks |
| **CSA2 Layer Sharing** | Full/Reindex/Reuse static modes | KV + indexer state shared across layers |
| **FP4 KV Cache** | E2M1 format, 1 scale/16 channels | 890 bytes/token (1/4 of V4-Flash) |
| **SWA Bounded Replay** | Replay recent window, no SSD | 1/8 persistent cache footprint |
| **Hierarchical Indexer** | 16K candidate pool bounded | Indexing cost independent of context length |
| **DSpark** | 3-stage draft, confidence-scheduled | Higher throughput without quality loss |
| **Mixed Precision** | MXFP4 experts + MXFP8 attention | 511 GB checkpoint on disk |
| **YaRN ×16** | 65K base → 1M native | No separate context extension phase |

---

## 5. NeoTrix Mapping

### 5.1 NT-CORE — Attention Architecture

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **CED asymmetric activation** | `nt_core_self::AttentionManager` | Dual activation mode: cheap input processing, expensive generation — mirrors Agent workloads |
| **CSA2 static mode assignment** | `nt_core::HyperCube` | Layer-level attention strategy: Full/Reindex/Reuse as module-level caching policies |
| **mHC residual mixing** | `nt_core::HyperCube` signal propagation | Doubly-stochastic combine for deep stack stability |
| **sqrtsoftplus MoE scoring** | `AttentionManager` routing | Replace sigmoid with sqrtsoftplus for expert affinity scoring |

### 5.2 NT-IO — KV Cache & Inference

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **FP4 KV cache (890 B/token)** | `nt_io::kv_cache_optimizer` | FP4 E2M1 format + per-16-channel scaling for extreme KV compression |
| **Hierarchical Sparse Indexer** | `nt_io::inference_engine` | Bounded indexing cost: candidate pool from first Full layer |
| **DSpark speculative decoding** | `nt_io::speculative_decoder` | Confidence-scheduled draft length, 3-stage semi-autoregressive |
| **YaRN ×16 context scaling** | `nt_io::context_manager` | Native 65K → 1M with RoPE theta=160K for compressed positions |
| **Mixed MXFP4/MXFP8** | `nt_io::quantization_engine` | Expert weights MXFP4, attention MXFP8, embeddings BF16 |

### 5.3 NT-MEMORY — Conditional Memory

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **Engram (196B sparse memory)** | `nt_memory::kv_store` | Token n-gram hash tables with context-aware gating |
| **Sparse access pattern** | `nt_memory::lazy_loader` | Load on-demand via token-based lookup, not every forward pass |
| **Host-memory RDMA prefetch** | `nt_memory::cache_tier` | GPU→Host→SSD tiered with prefetch for hot conditional memory |

### 5.4 NT-MIND — Training Pipeline

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **45T multimodal pretraining** | `nt_mind::seal_pipeline` | Multimodal data integration from day 1, 7:1 text:MM ratio |
| **SFT → RL → OPD** | `nt_mind::skill_engine` | Three-phase post-training: supervised → reinforcement → distillation |
| **Automated agent task synthesis** | `nt_mind::experience_generator` | Large-scale synthetic environments with progressive scaling |
| **Controllable reasoning effort (1-100)** | `nt_mind::effort_controller` | Continuous dial for inference cost vs accuracy tradeoff |
| **Muon optimizer** | `nt_mind::training_kernel` | Orthogonalized momentum for faster convergence at scale |

### 5.5 NT-WORLD — Multimodal Perception

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **DeepSeek-ViT (2D-RoPE)** | `nt_world::vision_encoder` | From-scratch ViT with 2D rotary embeddings |
| **3×3 pixel-unshuffle** | `nt_world::image_preprocessor` | Efficient downsampling before embedding |
| **MLP projector** | `nt_world::multimodal_bridge` | 2-layer MLP converting visual → language embedding space |
| **Joint pretraining** | `nt_world::perception_fusion` | Vision processed with text from pretraining start, not bolted on |

### 5.6 NT-ACT — Agent Task Synthesis

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **Progressive task scaling** | `nt_act::production_orchestrator` | Scale data volume, task variety, rollout count in stages |
| **Verifiable rewards** | `nt_act::quality_control` | RL rewards grounded in verifiable outcomes |
| **Domain expert isolation → distill** | `nt_act::skill_isolation` | Train specialists independently, then unify via OPD |

### 5.7 NT-SHIELD — Security Implications

| DeepSeek Innovation | NeoTrix Component | Absorption |
|---------------------|-------------------|------------|
| **MIT license** | `nt_shield::license_engine` | Permissive reuse, but inspect for IP contamination |
| **Engram memory tables** | `nt_shield::data_governance` | 196B parameter memory — verify no training data leakage |
| **Persistent KV cache (72h)** | `nt_shield::cache_security` | Multi-tenant cache isolation required for production |

---

## 6. Key Insights for NeoTrix

### 6.1 The KV Cache is the Bottleneck
- DeepSeek's entire V4.1 architecture is organized around KV cache compression
- 890 bytes/token makes 1M context economically viable
- **NeoTrix priority**: `kv_cache_optimizer` must support FP4 E2M1 + cross-layer reuse

### 6.2 Asymmetric Activation for Agent Workloads
- 8B prefill / 16B decode is purpose-built for long-prompt, short-output agent tasks
- **NeoTrix mapping**: `AttentionManager` should route input-heavy tasks to cheaper compute paths

### 6.3 Conditional Memory Separates Memorization from Computation
- Engram's 196B parameters are NOT activated every forward pass
- Token-based lookup loads only what's needed
- **NeoTrix mapping**: KB should implement sparse conditional loading, not eager full-load

### 6.4 Static Mode Assignment Beats Dynamic
- CSA2 assigns Full/Reindex/Reuse statically per layer, not dynamically per token
- Simpler hardware implementation, predictable memory profile
- **NeoTrix mapping**: Module-level caching policies (static) > token-level routing (dynamic) for memory-bound operations

### 6.5 Controllable Reasoning Effort is the UX Pattern
- Integer 1-100 dial replaces separate model variants
- **NeoTrix mapping**: `nt_mind::effort_controller` — continuous reasoning budget per task

### 6.6 Data Pipeline > Algorithm Innovation
- DeepSeek explicitly states "no algorithmic innovation" in post-training
- All gains from automated task synthesis + progressive scaling
- **NeoTrix mapping**: SEAL pipeline should invest in data generation infrastructure, not RL algorithm variants

---

## 7. Implementation Roadmap

### Phase 1: KV Cache Foundation (P0)
- [ ] FP4 E2M1 KV cache with per-16-channel scaling in `kv_cache_optimizer`
- [ ] Cross-layer KV sharing (Full/Reindex/Reuse static modes)
- [ ] Hierarchical Sparse Indexer for bounded indexing cost
- [ ] SWA Bounded Replay for sliding window reconstruction

### Phase 2: Asymmetric Activation (P1)
- [ ] CED-style split in `AttentionManager`: cheap input, expensive generation
- [ ] Dual activation paths for prefill vs decode
- [ ] Cost-aware routing based on task shape (long prompt → cheap path)

### Phase 3: Conditional Memory (P2)
- [ ] Engram-style sparse memory tables in KB
- [ ] Token n-gram hash lookup with context-aware gating
- [ ] GPU→Host→SSD tiered with RDMA prefetch

### Phase 4: Training Pipeline (P3)
- [ ] Automated agent task synthesis in SEAL pipeline
- [ ] Progressive scaling: data volume → task variety → rollout count
- [ ] Controllable reasoning effort (1-100) for skill execution
- [ ] On-Policy Distillation: domain expert → unified model

### Phase 5: Multimodal (P4)
- [ ] DeepSeek-ViT style vision encoder with 2D-RoPE
- [ ] Joint vision-text pretraining from day 1
- [ ] MLP projector for visual → language embedding bridge

---

## 8. Comparison with Previous Models

| Spec | DeepSeek-V3 | DeepSeek-V4-Flash | DeepSeek-V4.1-Flash |
|------|-------------|-------------------|---------------------|
| Total Params | 671B | 284B | 552B + 196B Engram |
| Active Params | 37B | 13B | 8B prefill / 16B decode |
| Attention | MLA (single) | CSA + HCA hybrid | CSA2 (3 static modes) |
| KV Cache (1M) | baseline | 10% of V3 | 890 B/token (1/4 of V4) |
| Context | 128K | 1M | 1M (YaRN ×16) |
| Residual | Standard | mHC | mHC (Mega kernel) |
| Speculative Decoding | MTP | MTP | DSpark (separate training) |
| Conditional Memory | None | None | 196B Engram |
| Post-training | SFT+RL | Domain expert→distill | SFT→RL→OPD |
| Vision | None | None | Native DeepSeek-ViT |

---

## 9. Sources

| Source | URL | Key Data |
|--------|-----|----------|
| HuggingFace Model Card | `deepseek-ai/DeepSeek-V4.1-Flash` | Architecture specs, config |
| DeepSeek Blog | deepseek.com/en/news/deepseek-v4-1-flash | Pricing, API details |
| DEV Community Deep Dive | dev.to/matthewhsu/deepseek-v41-flash-deep-dive | CED, CSA2, Engram analysis |
| MindStudio Analysis | mindstudio.ai/blog/deepseek-v4-1-flash-specs | KV cache compression |
| OrcaRouter Analysis | orcarouter.ai/blog/deepseek-v4-1-new-base-model | Family claim, naming |
| Progressive Robot | progressiverobot.com | Config details, vLLM recipe |
| vLLM Recipe | recipes.vllm.ai/deepseek-ai/DeepSeek-V4.1-Flash | Serving requirements |
| DeepSeek-V4 ArXiv | arxiv.org/html/2606.19348 | V4 architecture (parent) |

---

*Generated by NeoTrix Model Reverse Engineering Pipeline*
*Absorbed into NT-IO + NT-CORE domain knowledge*
