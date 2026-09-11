# Model Reverse Engineering — 10-Model Architecture Extraction (240)

Date: 2026-09-11

## Models Analyzed

| # | Model | Org | Params (Total/Active) | Architecture | Release |
|---|-------|-----|----------------------|-------------|---------|
| 1 | GPT-4o | OpenAI | ~200B (est.) | Proprietary omni-modal Transformer | May 2024 |
| 2 | Claude 3.5 Sonnet | Anthropic | Undisclosed | Dense Transformer | Jun 2024 |
| 3 | Gemini 2.5 Pro | Google DeepMind | Undisclosed | MoE Transformer | Mar 2025 |
| 4 | Llama 4 Scout | Meta | 109B / 17B active | MoE (16 experts) | Apr 2025 |
| 5 | DeepSeek V4.1 Flash | DeepSeek | 552B / 13B active | MoE + CED + CSA/HCA | Sep 2026 |
| 6 | Qwen3 | Alibaba | 0.6B–235B | Dense + MoE variants | May 2025 |
| 7 | Mistral Large 3 | Mistral AI | 675B / 41B active | Granular MoE (128 experts) | Dec 2025 |
| 8 | Phi-4 Reasoning | Microsoft | 14B | Dense Transformer (SFT+RL) | Apr 2025 |
| 9 | Yi-Lightning | 01.AI | 200B | Enhanced MoE | Dec 2024 |
| 10 | Grok 3 | xAI | ~1.2T (est.) | MoE + Neuro-symbolic | Feb 2025 |

---

## Per-Model Architecture Deep-Dive

### 1. GPT-4o — Omni-Modal End-to-End

**Key Innovation**: Single unified neural network for text/audio/image/video. Eliminated cascade (ASR→LLM→TTS) → 320ms audio response (vs 5.4s in GPT-4 Turbo).

**Architecture Signals**:
- End-to-end multi-modal training (not pipeline fusion)
- 128K context, ~200B parameters (estimated)
- Unified embedding space for all modalities
- Real-time voice as native capability

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| End-to-end multi-modal | `nt_physical::sensory_integration_hub` | Unified sensory pipeline, not cascade adapters |
| Unified embedding space | `VSA HyperCube` | Cross-modal symbolic representation |
| Real-time latency (320ms) | GWT attention routing | Fast-path for I/O-bound tasks |
| Native voice | `nt_io::llm_provider` | Provider-agnostic multi-modal routing |

---

### 2. Claude 3.5 Sonnet — Performance-at-Speed

**Key Innovation**: Frontier intelligence at 2x speed of Opus, at mid-tier cost. SWE-bench Verified 49.0% (SOTA at release).

**Architecture Signals**:
- Dense Transformer (no MoE disclosed)
- Strong agentic coding: 64% internal eval vs 38% for Opus
- Excellent vision: chart/graph interpretation, OCR from imperfect images
- Tool use as first-class capability

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Speed-at-intelligence | GWT salience + cost weight (A1) | Route simple tasks to cheap models |
| Agentic coding | `nt_act::mcp_tools` | Tool orchestration as primary interface |
| Vision quality | `nt_world::perception_bridge` | Attention-gated perception flow |
| SWE-bench dominance | `skills/dev/implementer` | TDD + verification-before-completion |

---

### 3. Gemini 2.5 Pro — Thinking Model + Long Context

**Key Innovation**: First "thinking model" in Gemini family. 1M token context (planned 2M). Native multi-modal (text/image/audio/video).

**Architecture Signals**:
- MoE architecture (Sparse MoE Transformer)
- "Thinking" capability as core (not bolt-on)
- 1M token context window
- Multi-modal input (text/image/audio/video/PDF)
- Google infrastructure advantage

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Native "thinking" | `nt_core::consciousness_tree` | 6-stage meta-cognition loop |
| 1M context | `nt_memory::kv_cache_optimizer` | Paged KV virtualization (KVMem pattern) |
| Multi-modal native | `nt_sense::sensory_integration` | L2 perception layer |
| Thinking budget | GWT attention routing | Cost-aware routing (A1) |
| Structured outputs | `nt_io::llm_provider` | Provider-level structured output |

---

### 4. Llama 4 Scout — MoE + Early Fusion + iRoPE

**Key Innovation**: MoE with early fusion for native multimodality. iRoPE for 10M token context. 109B total / 17B active.

**Architecture Signals**:
- 16 experts, top-k routing
- Early fusion: text + vision tokens merged at input (not late fusion)
- iRoPE: rotary position embeddings for extreme context length
- Trained on 40T tokens
- Open weights (Apache 2.0 family)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| MoE routing | `nt_core::gwt` | Expert selection = attention routing |
| Early fusion | `nt_sense::perception_bridge` | L2→L5 early attention gating |
| iRoPE (10M context) | `nt_memory::kv_cache_optimizer` | Infinite context goal |
| Open weights | `nt_io::llm_provider` | Provider fallback chain |
| Expert specialization | `nt_core::skill_tree` | Domain-specific capability nodes |

---

### 5. DeepSeek V4.1 Flash — KV Cache Compression Master

**Key Innovation**: Compressed Sparse Attention (CSA) + Heavily Compressed Attention (HCA). Causal Encoder-Decoder (CED) for KV cache compression. 552B backbone, 13B active, 1M context.

**Architecture Signals**:
- CED: 20-layer causal encoder on top of 20-layer decoder
- Ratio-4 (selective) + Ratio-128 (heavy) compressed attention
- Raw 128-token sliding window + compressed historical KV
- Multi-Token Prediction (MTP) for speculative decoding
- MXFP4 quantization support
- $0.15/1M input tokens — extreme cost efficiency

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| CSA + HCA | `nt_memory::kv_cache_optimizer` | Tiered KV: hot (raw) → warm (ratio-4) → cold (ratio-128) |
| CED architecture | `nt_memory::kb_pipeline` | Encoder for indexing, decoder for generation |
| MTP speculative decoding | `nt_core::consciousness_tick` | Lookahead reasoning (speculative branches) |
| MXFP4 quantization | `nt_shield::resource_monitor` | Hardware-aware precision switching |
| Cost efficiency ($0.15/M) | A1: Cost-Aware Routing | Cheapest provider for simple tasks |

---

### 6. Qwen3 — Thinking + Non-Thinking Unified

**Key Innovation**: Single model with thinking mode (chain-of-thought) and non-thinking mode (fast response). Thinking budget mechanism. 36T token pretraining.

**Architecture Signals**:
- Dense (0.6B–32B) + MoE (30B-A3B, 235B-A22B) variants
- QK-Norm (replaced QKV-bias) for training stability
- Global-batch load balancing loss for expert specialization
- 119 language support (vs 29 in Qwen2.5)
- BBPE tokenizer, 151,669 vocab

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Thinking/non-thinking toggle | `nt_core::attention_manager` | Dual specialization: CORE+WORLD vs CORE+MIND |
| Thinking budget | GWT salience | Adaptive compute allocation |
| QK-Norm | `nt_core::e8_engine` | Training stability for reasoning |
| Multi-tier sizes | A1: Cost-Aware Routing | Match model size to task complexity |
| 119 languages | `nt_io::llm_provider` | Multi-lingual provider support |

---

### 7. Mistral Large 3 — Granular MoE + MLA

**Key Innovation**: Granular MoE (128 experts, top-4 selection, softmax routing) + Multi-Latent Attention (MLA). 675B total / 41B active.

**Architecture Signals**:
- DeepSeekV3-style MoE with fewer, larger experts
- Softmax-based routing (vs top-k binary)
- Llama 4 RoPE scaling
- Multi-Latent Attention for KV cache reduction
- Vision Encoder (2.5B) for multimodal
- Apache 2.0 open weights
- Trained on 3000 H200 GPUs

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Granular MoE (128 experts) | `nt_core::skill_tree` | Fine-grained capability nodes |
| Softmax routing | GWT salience | Probabilistic attention (not binary) |
| MLA (KV cache reduction) | `nt_memory::kv_cache_optimizer` | Multi-latent compression |
| Vision Encoder | `nt_sense::perception_bridge` | Dedicated perception module |
| Open weights + Apache 2.0 | `nt_io::llm_provider` | Self-hosted provider option |

---

### 8. Phi-4 Reasoning — Small-but-Mighty Distillation

**Key Innovation**: 14B model outperforming 5-50x larger models on reasoning. SFT on o3-mini traces + short RL phase. Synthetic data as primary training signal.

**Architecture Signals**:
- Dense decoder-only Transformer (14B)
- SFT on 1.4M curated "teachable" prompts
- o3-mini reasoning traces as training data
- Phi-4-reasoning-plus: additional outcome-based RL
- Runs on single GPU (laptop-capable)
- 32K context

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Small model = big reasoning | A1: Cost-Aware Routing | Route reasoning to distillated small models |
| Synthetic data pipeline | `nt_mind::seal_pipeline` | Distillation as core capability |
| Teachable prompt selection | `nt_mind::skill_engine` | Difficulty-adaptive skill selection |
| Single GPU inference | `nt_physical::power_management` | Edge deployment |
| RL enhancement | `nt_mind::self_evolution` | Post-training self-improvement |

---

### 9. Yi-Lightning — MoE + Cross-Layer KV Sharing

**Key Innovation**: Fine-grained expert segmentation + cross-layer KV cache sharing (82.8% memory reduction). Hardware-aware FP8 quantization design.

**Architecture Signals**:
- Fine-grained expert FFN segmentation
- Expert Parallel (EP) + Partitioned EP (PEP) load balancing
- Cross-layer KV cache sharing between consecutive full-attention layers
- 82.8% memory reduction for long sequences
- FP8 quantization-compatible architecture design
- Hybrid expert architecture (dense + sparse)
- RAISE safety framework (4-component)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Cross-layer KV sharing | `nt_memory::kv_cache_optimizer` | Shared cache across layers |
| Fine-grained expert segmentation | `nt_core::skill_tree` | Micro-node → Notable → Keystone tiers |
| FP8 hardware-aware design | `nt_physical::resource_monitor` | Hardware-architecture co-design |
| RAISE safety | `nt_shield::stealth_net` | Multi-layer safety framework |
| 82.8% memory reduction | KVMem pattern (A2) | Context as scarce resource |

---

### 10. Grok 3 — Massive Scale + Neuro-Symbolic

**Key Innovation**: 1.2T parameter MoE + neuro-symbolic integration. Colossus supercomputer (200K H100 GPUs). Test-time compute at scale (TTCS).

**Architecture Signals**:
- MoE + symbolic reasoning modules
- 12 input modalities (text, image, audio, 3D point clouds, etc.)
- Hierarchical ViT with adaptive patch sizing
- Constrained decoding for 2.1% hallucination rate (vs 3.4% competitors)
- Three inference modes: Think / Big Brain / DeepSearch
- Adversarial debiasing in intermediate representations

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Neuro-symbolic | `nt_core::e8_engine` | Symbolic reasoning (E8 hexagram) |
| TTCS (test-time compute) | `nt_core::consciousness_tick` | Adaptive compute per task |
| 12 modalities | `nt_sense::sensory_integration` | Universal sensory input |
| Constrained decoding | `nt_shield::egress_guard` | Output safety/filtering |
| DeepSearch mode | `nt_world::unified_crawler` | Agent-mediated web research |
| Think/Big Brain modes | `nt_core::attention_manager` | Mode switching by task depth |

---

## Cross-Model Architecture Patterns

### Pattern Matrix

| Pattern | Models Using | Frequency | NeoTrix Mapping |
|---------|-------------|-----------|-----------------|
| **MoE (Mixture-of-Experts)** | Gemini, Llama4, DS4, Qwen3, Mistral3, Yi, Grok3 | 7/10 | GWT salience routing |
| **Native Multi-Modal** | GPT-4o, Gemini, Llama4, Mistral3, Grok3 | 5/10 | `nt_sense::perception_bridge` |
| **KV Cache Optimization** | DS4, Yi, Mistral3, Llama4 | 4/10 | `nt_memory::kv_cache_optimizer` |
| **Thinking/Reasoning Modes** | Gemini, Qwen3, Grok3, Phi4 | 4/10 | `nt_core::attention_manager` |
| **Context >256K** | Gemini(1M), Llama4(10M), DS4(1M), Mistral3(256K) | 4/10 | KVMem paged KV (A2) |
| **Open Weights** | Llama4, DS4, Qwen3, Mistral3, Phi4 | 5/10 | `nt_io::llm_provider` fallback |
| **Speculative Decoding** | DS4 (MTP), Mistral3 (Eagle) | 2/10 | `nt_core::consciousness_tick` |
| **Cost Efficiency** | DS4 ($0.15/M), Phi4 (14B), Llama4 (17B active) | 3/10 | A1: Cost-Aware Routing |

### Dominant Trend: MoE is the New Default

7 out of 10 models use MoE. The industry has converged on:
- **Sparse activation**: Only a fraction of parameters active per token
- **Expert routing**: Gating functions select specialists
- **KV cache compression**: Essential for long-context efficiency

### Emergent Pattern: Thinking as First-Class

Models are no longer "text completion" — they are "reasoning engines":
- Gemini 2.5 Pro: "thinking model" as core design
- Qwen3: Unified thinking/non-thinking toggle
- Grok 3: Think / Big Brain / DeepSearch modes
- Phi-4: Reasoning chain as primary output format

### Emergent Pattern: Context as Scarce Resource (A2 Validated)

| Model | Context | KV Strategy |
|-------|---------|-------------|
| Gemini 2.5 Pro | 1M (planned 2M) | In-context learning |
| Llama 4 Scout | 10M | iRoPE positional encoding |
| DeepSeek V4.1 | 1M | CSA + HCA compression |
| Mistral Large 3 | 256K | MLA (Multi-Latent Attention) |

---

## NeoTrix Absorption Actions

### Immediate (P0) — Implement This Cycle

| # | Source | Innovation | Target Component | Action |
|---|--------|-----------|-----------------|--------|
| 1 | DeepSeek V4.1 | CSA+HCA tiered KV | `kv_cache_optimizer.rs` | Implement 3-tier: raw(128) → compressed(4) → heavy(128) |
| 2 | Qwen3 | Thinking/non-thinking toggle | `attention_manager.rs` | Add mode switching: fast-response vs deep-reasoning |
| 3 | Phi-4 | Small model distillation | `seal_pipeline.rs` | Add distillation stage: large→small with teachable prompts |
| 4 | Yi-Lightning | Cross-layer KV sharing | `kv_cache_optimizer.rs` | Share KV state between consecutive layers (82.8% savings) |

### Near-Term (P1) — Next Cycle

| # | Source | Innovation | Target Component | Action |
|---|--------|-----------|-----------------|--------|
| 5 | Llama 4 | Early fusion (text+vision at input) | `perception_bridge.rs` | Merge modalities at L2, not late fusion |
| 6 | Mistral 3 | Softmax routing (probabilistic) | `gwt.rs` | Replace binary gating with softmax attention |
| 7 | Grok 3 | Neuro-symbolic integration | `e8_engine.rs` | Strengthen E8↔symbolic reasoning bridge |
| 8 | GPT-4o | End-to-end multi-modal | `sensory_integration_hub.rs` | Unified pipeline, not cascade adapters |

### Strategic (P2) — Architecture Evolution

| # | Source | Innovation | Target Component | Action |
|---|--------|-----------|-----------------|--------|
| 9 | All MoE models | MoE routing efficiency | `gwt.rs` | GWT as MoE-style expert router for NT-* domains |
| 10 | Gemini/Qwen3/Grok3 | Thinking budget mechanism | `consciousness_tick.rs` | Adaptive compute: allocate thinking tokens by task complexity |
| 11 | DS4+Yi | MXFP4/FP8 quantization | `resource_monitor.rs` | Hardware-aware precision switching |
| 12 | Grok 3 | 12-modality support | `perception_bridge.rs` | Extend beyond text/image to audio/video/3D |

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MoE vs Dense: MoE needs more memory, Dense simpler to deploy | **Dual path**: Dense for edge (Phi-4 pattern), MoE for cloud (Qwen3 pattern) |
| Open vs Closed: Open models catching up to closed | **Provider fallback**: Self-hosted open models as fallback for paid APIs |
| Thinking vs Speed: Deep reasoning = slow | **Thinking budget**: Let users choose depth (Qwen3/Grok3 pattern) |
| KV Cache vs Context Length: More context = more memory | **Tiered compression**: CSA+HCA (DS4) or MLA (Mistral3) |
| Small Model vs Quality: Phi-4 proves size ≠ quality | **Distillation pipeline**: Train small on large's traces (Phi-4 pattern) |

---

## Summary

The 2024-2026 LLM architecture landscape has converged on three pillars:

1. **MoE as default**: 7/10 models use sparse expert routing — validated as the efficiency frontier
2. **Thinking as first-class**: Models are reasoning engines, not text completers — thinking modes are mandatory
3. **Context as scarce resource**: KV cache compression and paged memory are critical for >1M context

NeoTrix's architecture already maps well to these trends:
- **GWT** ↔ MoE routing (expert selection)
- **ConsciousnessTree** ↔ Thinking/reasoning loop
- **KV Cache Optimizer** ↔ Context compression (needs CSA/HCA implementation)
- **Cost-Aware Routing (A1)** ↔ Small/large model selection
- **E8 Engine** ↔ Neuro-symbolic reasoning

The highest-impact absorption targets are DeepSeek V4.1's tiered KV compression and Qwen3's thinking mode toggle — both directly enhance NeoTrix's memory and reasoning subsystems.
