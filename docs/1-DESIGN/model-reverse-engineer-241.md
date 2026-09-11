# Model Reverse Engineering — 10-Model Architecture Extraction (241)

Date: 2026-09-11
Session: Deep research batch — 10 frontier models architecture reverse-engineering

## Models Analyzed

| # | Model | Org | Params (Total/Active) | Architecture | Release |
|---|-------|-----|----------------------|-------------|---------|
| 1 | GPT-4o | OpenAI | ~200B (est.) | Proprietary omni-modal Transformer | May 2024 |
| 2 | Claude 3.5 Sonnet | Anthropic | Undisclosed | Dense Transformer | Jun 2024 |
| 3 | Gemini 2.5 Pro | Google DeepMind | Undisclosed | MoE Thinking Model | Mar 2025 |
| 4 | Llama 4 Scout | Meta | 109B / 17B active | MoE (16 experts) + Early Fusion | Apr 2025 |
| 5 | DeepSeek V4 Flash | DeepSeek | 284B / 13B active | MoE + CSA/HCA Hybrid Attention | Apr 2026 |
| 6 | Qwen3 | Alibaba | 0.6B–235B | Dense + MoE Hybrid Reasoning | Apr 2025 |
| 7 | Mistral Large 3 | Mistral AI | 675B / 41B active | Granular MoE (128 experts) | Dec 2025 |
| 8 | Phi-4 Reasoning | Microsoft | 14B | Dense Transformer (SFT+RL) | Apr 2025 |
| 9 | Yi-Lightning | 01.AI | 200B (est.) | Enhanced MoE + RAISE | Dec 2024 |
| 10 | Grok 3 | xAI | ~1.2T (est.) | Hybrid Dense/MoE + Neuro-symbolic | Feb 2025 |

---

## Cross-Model Innovation Matrix

| Innovation | GPT-4o | Claude | Gemini | Llama4 | DeepSeek | Qwen3 | Mistral | Phi-4 | Yi | Grok3 |
|-----------|--------|--------|--------|--------|----------|-------|---------|-------|----|----|
| MoE Architecture | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| Native Multimodal | ✓ | vision | ✓ | ✓ | — | — | ✓ | vision | — | vision |
| Hybrid Reasoning | — | — | ✓ (thinking) | — | ✓ (3 modes) | ✓ (think/non-think) | — | ✓ (SFT) | — | ✓ (TTCS) |
| Compressed Attention | — | — | — | iRoPE | CSA+HCA | DeltaNet | MLA | — | — | — |
| 1M+ Context | — | 200K | 1M | 192K (iRoPE) | 1M | 256K→1M | 256K | 32K | 128K | 131K |
| Synthetic Data Focus | — | — | — | — | — | ✓ | — | ✓✓ | ✓ | ✓ |
| Open Weights | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| KV Cache Optimization | — | — | — | — | CSA | DeltaNet | cross-layer KV | — | ✓ | — |

---

## Per-Model Architecture Deep-Dive

### 1. GPT-4o — Omni-Modal End-to-End

**Key Innovation**: Single unified neural network for text/audio/image/video processing. Eliminated the cascade pipeline (ASR→LLM→TTS) achieving 320ms audio response latency (vs 5.4s in GPT-4 Turbo).

**Architecture Signals**:
- End-to-end multi-modal training (not pipeline fusion)
- 128K context, ~200B parameters (estimated)
- Unified embedding space for all modalities — text, audio, image, video tokens flow through same network
- Real-time voice as native capability (232ms min, 320ms avg)
- 50% cheaper API than GPT-4 Turbo
- Autoregressive omni model: all inputs/outputs processed by same neural network

**Technical Details**:
- Architecture undisclosed (proprietary)
- Token efficiency: optimized tokenization across modalities
- Safety integrated into architecture (not post-processing)
- RLHF post-training alignment

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| End-to-end multi-modal | `nt_physical::sensory_integration_hub` | Unified sensory pipeline, not cascade adapters |
| Unified embedding space | `VSA HyperCube` | Cross-modal symbolic representation |
| Real-time latency (320ms) | GWT attention routing | Fast-path for I/O-bound tasks, Cost-Aware Routing (A1) |
| Native voice | `nt_io::llm_provider` | Provider-agnostic multi-modal routing |
| Safety-in-architecture | `nt_shield::egress_privacy_guard` | Trust-tier enforcement at architectural level |

---

### 2. Claude 3.5 Sonnet — Performance-at-Speed

**Key Innovation**: Frontier intelligence at 2x speed of Opus, at mid-tier cost. SWE-bench Verified 49.0% (SOTA at release). Dominates agentic coding (64% internal eval vs 38% for Opus).

**Architecture Signals**:
- Dense Transformer (no MoE disclosed)
- 200K token context window
- Excellent vision: chart/graph interpretation, OCR from imperfect images
- Tool use as first-class capability
- Computer use capability (screen interaction)
- Strong instruction following and nuance understanding

**Technical Details**:
- Architecture undisclosed (proprietary)
- $3/1M input, $15/1M output tokens
- Artifacts feature: workspace parallel to chat
- Upgrade path: Oct 2024 version improved SWE-bench to 49%, TAU-bench retail 69.2%

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Speed-at-intelligence | GWT salience + cost weight (A1) | Route simple tasks to cheap models |
| Agentic coding | `nt_act::mcp_tools` | Tool orchestration as primary interface |
| Vision quality | `nt_world::perception_bridge` | Attention-gated perception flow |
| SWE-bench dominance | `skills/dev/implementer` | TDD + verification-before-completion |
| Computer use | `nt_physical::motor_control` | Screen interaction loop |

---

### 3. Gemini 2.5 Pro — Thinking Model + Long Context

**Key Innovation**: First "thinking model" in Gemini family with native reasoning. 1M token context window. Deep Think mode for complex math/code.

**Architecture Signals**:
- MoE architecture (Sparse MoE Transformer)
- "Thinking" as core capability (not bolt-on) — controllable thinking budget
- 1M token context window, 65K max output tokens
- Multi-modal input (text/image/audio/video/PDF)
- Google infrastructure advantage: TPUs + custom training stack
- Dynamic thinking budget control via API parameter

**Technical Details**:
- MoE with thinking tokens interleaved
- Deep Think: enhanced reasoning mode considering multiple hypotheses
- Native audio output (not just input)
- Computer use capabilities (Project Mariner)
- SWE-bench Verified: 63.8% (custom agent setup)
- LearnLM integration for pedagogical effectiveness
- Security: advanced indirect prompt injection protection

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Native "thinking" | `nt_core::consciousness_tree` | 6-stage meta-cognition loop (Soil→Roots→Trunk→Branches→Fruits→Core) |
| Controllable thinking budget | GWT salience + cost weight | Dynamic resource allocation per task complexity |
| 1M context | `nt_memory::kv_cache_optimizer` | Paged KV virtualization (KVMem pattern) |
| Multi-modal native | `nt_sense::sensory_integration` | L2 perception layer |
| Deep Think | `nt_mind::seal_pipeline` | Multi-hypothesis exploration in evolution cycles |

---

### 4. Llama 4 Scout — Open MoE + Early Fusion

**Key Innovation**: First open-source natively multimodal MoE Llama model. 17B active params (109B total, 16 experts). Fits on single H100 (Int4). 192K context via iRoPE.

**Architecture Signals**:
- Mixture-of-Experts: 17B active, 16 experts, 109B total
- Early fusion for native multimodality (text + image)
- iRoPE (interleaved Rotary Position Embedding) for extreme context extension
- FP8 training precision, 390 TFLOPs/GPU
- Distilled from Llama 4 Behemoth (288B active teacher)
- 200 language pretraining, 10x more multilingual tokens than Llama 3
- 40T token pretraining corpus

**Technical Details**:
- Llama 4 Scout: 17B active, 16 experts, 109B total, 192K context
- Llama 4 Maverick: 17B active, 128 experts, 400B total, 1M context
- Llama 4 Behemoth: 288B active, 16 experts (teacher model, still training)
- Expert routing: top-k selection per layer
- Natively multimodal (text + image input, text + code output)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| MoE expert routing | `nt_core::capability_bridge` | Map tree node IDs to runtime capabilities |
| Early fusion multimodal | `nt_physical::sensory_integration_hub` | Unified modality processing |
| iRoPE long context | `nt_memory::kv_cache_optimizer` | Extended position encoding for >128K |
| Teacher distillation | `nt_mind::seal_pipeline` | Knowledge distillation in evolution cycles |
| Single-GPU deployment | `nt_physical::power_management` | Efficient resource allocation |

---

### 5. DeepSeek V4 Flash — Hybrid Attention Efficiency

**Key Innovation**: Compressed Sparse Attention (CSA) + Heavily Compressed Attention (HCA) hybrid architecture. Manifold-Constrained Hyper-Connections (mHC) for deep stack stability. 1M context at 284B params (13B active).

**Architecture Signals**:
- **CSA (Compressed Sparse Attention)**: Every 4 consecutive tokens merged into 1 compressed entry via learned weighted average. Lightning Indexer selects top-k (512 for Flash, 1024 for Pro).
- **HCA (Heavily Compressed Attention)**: Every 128 tokens → 1 entry. Dense attention over compressed entries. MQA (Multi-Query Attention) for efficiency.
- **mHC (Manifold-Constrained Hyper-Connections)**: Replace residual connections with doubly-stochastic mixing matrix (Birkhoff polytope). Ensures spectral norm ≤ 1 for stable signal propagation through 43-61 layers.
- **Hybrid interleaved layout**: CSA and HCA layers alternate. SWA (Sliding Window Attention) warmup in early layers.
- **Muon optimizer**: Matrix-level orthogonalization for faster convergence.

**Technical Details**:
- V4-Flash: 43 layers, 4096 hidden, CSA top-k=512, HCA m'=128
- V4-Pro: 61 layers, 7168 hidden, CSA top-k=1024, HCA m'=128
- 284B total, 13B active (Flash); 1.6T total, 49B active (Pro)
- 1M token context
- Three reasoning modes: Non-think, Think High, Think Max
- DeepSeekMoE architecture for MoE components
- FP4+FP8 mixed precision (native on B200)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| CSA compressed attention | `nt_memory::kv_cache_optimizer` | Token compression for long-context efficiency |
| HCA heavily compressed | `nt_memory::bm25_index` | Broad coverage with minimal compute |
| mHC stable deep stacks | `nt_core::consciousness_tree` | Stable signal propagation in meta-cognition |
| Hybrid attention layout | GWT attention routing | Layer-aware attention switching |
| Multi-mode reasoning | `nt_mind::seal_pipeline` | Non-think/High/Max modes for task adaptation |
| Muon optimizer | `nt_mind::distillation_engine` | Training stability for deep stacks |

---

### 6. Qwen3 — Hybrid Reasoning + Ultra-Sparse MoE

**Key Innovation**: First hybrid reasoning model from Alibaba — seamless switching between thinking and non-thinking modes. Qwen3-Next introduces Gated DeltaNet + Gated Attention hybrid attention (75%/25% ratio) with ultra-sparse MoE (3B active out of 80B total = 3.7% activation).

**Architecture Signals**:
- **Hybrid Reasoning**: Thinking mode for complex tasks, non-thinking for fast responses
- **Gated DeltaNet + Gated Attention**: 75% layers use linear attention (DeltaNet), 25% keep standard attention. Outperforms any monolithic architecture.
- **Ultra-Sparse MoE**: Qwen3-Next-80B-A3B activates only 3B parameters (3.7% of 80B)
- **Multi-Token Prediction (MTP)**: Boosts both training performance and inference efficiency
- **Training**: 36T tokens (2x Qwen2.5), 119 languages, 4-stage training process
- **MCP native support**: Model Context Protocol for agent integration

**Technical Details**:
- Qwen3 Dense: 0.6B, 1.7B, 4B, 8B, 14B, 32B
- Qwen3 MoE: 30B-A3B, 235B-A22B
- Qwen3-Next: 80B-A3B (hybrid attention, ultra-sparse MoE)
- Qwen3-Coder: 480B-A35B (agentic coding)
- Context: 131K base, 256K native, 1M extendable
- 4-stage training: CoT cold start → reasoning RL → general RL → domain SFT

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Hybrid reasoning modes | `nt_core::consciousness_tree` | Think/non-think mode switching per task |
| Gated DeltaNet linear attention | `nt_memory::kv_cache_optimizer` | O(n) attention for long context |
| Ultra-sparse MoE (3.7%) | `nt_core::capability_bridge` | Minimal activation for maximum efficiency |
| Multi-Token Prediction | `nt_mind::seal_pipeline` | Parallel prediction in evolution cycles |
| MCP native | `nt_act::mcp_tools` | Protocol-native tool integration |
| 4-stage training | `nt_mind::distillation_engine` | Staged training curriculum |

---

### 7. Mistral Large 3 — Granular MoE + Open Weights

**Key Innovation**: Granular MoE with 128 experts per layer (vs typical 8-16). 675B total / 41B active. First Mistral MoE since Mixtral. DeepSeekV3-style routing with softmax + top-4 selection.

**Architecture Signals**:
- **Granular MoE**: 128 experts per layer, top-4 selection with softmax routing
- **Multi-Latent Attention (MLA)**: Compressed KV representation
- **Vision Encoder**: 2.5B params for multimodal input
- **Llama 4 RoPE scaling**: Adopted from Meta's position encoding
- **NVFP4 quantization**: Efficient deployment on consumer hardware
- **EAGLE speculative decoding**: Draft model for faster inference

**Technical Details**:
- 675B total params, 41B active (39B language + 2.5B vision)
- 256K context window
- Trained from scratch on 3000 NVIDIA H200 GPUs
- Apache 2.0 license (fully open weights)
- FP8 and NVFP4 quantized variants available
- Ministral 3: Dense edge models (3B, 8B, 14B) with vision

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Granular MoE (128 experts) | `nt_core::capability_bridge` | Fine-grained capability routing |
| Multi-Latent Attention | `nt_memory::kv_cache_optimizer` | Compressed KV for long context |
| Vision encoder as module | `nt_sense::sensory_integration` | Modular perception pipeline |
| Speculative decoding | `nt_io::llm_provider` | Draft-verify pattern for latency |
| Open weights + Apache 2.0 | `nt_memory::versioning` | Model versioning and deployment |
| Edge deployment (Ministral) | `nt_physical::power_management` | On-device inference |

---

### 8. Phi-4 Reasoning — Small Model, Big Reasoning

**Key Innovation**: 14B params matching DeepSeek-R1 (671B) on AIME 2025. Synthetic data pipeline + o3-mini distillation. Pivotal token search for DPO pairs.

**Architecture Signals**:
- Dense decoder-only Transformer, 14B params
- Extended context: 4K → 16K (midtraining) → 32K (reasoning)
- Synthetic data as primary training signal (not web crawl)
- o3-mini teacher for high-quality reasoning traces
- Pivotal token search for DPO pair creation
- "Teachable" prompt selection for SFT
- Safety guidelines embedded in prompts, then removed during training

**Technical Details**:
- Architecture: same as Phi-4 base (14B dense decoder-only)
- Training: SFT on curated "teachable" prompts + o3-mini demonstrations
- Context extension to 32K for long chain-of-thought
- AIME 2025: competitive with DeepSeek-R1 (671B)
- Synthetic data: generated, filtered, diversity-optimized
- Phi-4 base uses tiktoken tokenizer (100K vocab)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Synthetic data pipeline | `nt_mind::distillation_engine` | Synthetic data for knowledge transfer |
| o3-mini distillation | `nt_mind::seal_pipeline` | Teacher-student knowledge transfer |
| Pivotal token search | `nt_core::e8_hexagram` | Token-level reasoning trace optimization |
| "Teachable" prompt selection | `nt_mind::skill_crystallization` | Curated difficulty progression |
| Small-model-big-reasoning | Cost-Aware Routing (A1) | 14B model for complex tasks |

---

### 9. Yi-Lightning — Enhanced MoE + RAISE Safety

**Key Innovation**: Fine-grained expert segmentation + balanced routing + cross-layer KV cache sharing. 6th on Chatbot Arena. RAISE safety framework across full lifecycle.

**Architecture Signals**:
- Enhanced MoE with fine-grained expert segmentation
- Balanced expert routing strategy
- Cross-layer KV cache sharing (reduce memory)
- FP8 quantization compatibility (hardware-aligned design)
- BPE tokenization with 100K vocab
- Multi-stage training with synthetic data synthesis

**Technical Details**:
- ~200B params (estimated, undisclosed)
- Enhanced MoE architecture
- Multi-stage pre-training approach
- SFT + RLHF post-training
- RAISE: 4-component safety framework (pre-training, post-training, serving)
- Data synthesis for math and coding tasks
- Strong Chinese, Math, Coding, Hard Prompts performance

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Fine-grained expert segmentation | `nt_core::capability_bridge` | Granular capability decomposition |
| Balanced routing | GWT attention routing | Load-balanced specialist dispatch |
| Cross-layer KV sharing | `nt_memory::kv_cache_optimizer` | KV cache deduplication across layers |
| RAISE safety framework | `nt_shield::egress_privacy_guard` | Full-lifecycle safety enforcement |
| FP8 hardware alignment | `nt_physical::power_management` | Precision-aware resource allocation |

---

### 10. Grok 3 — Scale + Neuro-symbolic

**Key Innovation**: 100K H100 GPU training (Colossus supercomputer). Test-Time Computing at Scale (TTCS). DeepSearch for real-time information integration. Big Brain mode for deep analysis.

**Architecture Signals**:
- Hybrid dense/MoE architecture (1.2T estimated)
- Neuro-symbolic integration (Transformer + symbolic reasoning)
- Test-Time Compute at Scale (TTCS) — inference-time compute scaling
- DeepSearch: real-time information from X platform
- Big Brain mode: extra compute for complex analysis
- 131K context window
- Reinforcement learning for reasoning (Grok 3 Mini Think)

**Technical Details**:
- 100K H100 GPUs (doubled to 200K in 92 days)
- 200M GPU hours training
- AIME 2025: 82%, GPQA: 76%, LiveCodeBench: 65.5%
- Grok 3 Mini: 90.7% AIME 2025 (reasoning mode)
- 131K context window
- DeepSearch: real-time web + X platform integration
- Dynamic expert activation (78% FLOPs utilization)

**NeoTrix Mapping**:
| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Test-Time Compute at Scale | GWT salience + cost weight | Dynamic compute allocation per task |
| Neuro-symbolic | `nt_core::e8_hexagram` | Symbolic reasoning + neural processing |
| DeepSearch (real-time) | `nt_world::unified_crawler` | Real-time information acquisition |
| Big Brain mode | `nt_mind::seal_pipeline` | Deep analysis mode with extended compute |
| Scale (100K H100s) | `nt_physical::power_management` | Cluster-aware resource management |

---

## Cross-Model Synthesis: 5 Transferable Patterns

### P1: Hybrid Attention is the New Default
- **DeepSeek V4**: CSA + HCA interleaved
- **Qwen3-Next**: Gated DeltaNet (75%) + Gated Attention (25%)
- **Mistral Large 3**: Multi-Latent Attention
- **Yi-Lightning**: Cross-layer KV sharing
- **NeoTrix**: `nt_memory::kv_cache_optimizer` — unified attention strategy with layer-type-aware routing

### P2: Thinking/Reasoning as a First-Class Mode
- **Gemini 2.5 Pro**: Native thinking with controllable budget
- **Qwen3**: Hybrid think/non-think modes
- **DeepSeek V4**: 3 modes (Non-think/High/Max)
- **Grok 3 Mini**: Think mode with backtracking
- **NeoTrix**: `nt_core::consciousness_tree` — 6-stage thinking loop with resource-aware mode switching

### P3: Ultra-Sparse MoE for Edge/Deployment Efficiency
- **Qwen3-Next**: 3.7% activation (3B/80B)
- **Llama 4 Scout**: 15.6% activation (17B/109B)
- **DeepSeek V4 Flash**: 4.6% activation (13B/284B)
- **Mistral Large 3**: 6.1% activation (41B/675B)
- **NeoTrix**: Cost-Aware Routing (A1) + `nt_core::capability_bridge` — activate only needed capabilities

### P4: Synthetic Data as Training Equalizer
- **Phi-4 Reasoning**: o3-mini distillation, pivotal token search
- **Qwen3**: 36T tokens with synthetic augmentation
- **Yi-Lightning**: Synthetic math/coding data synthesis
- **Grok 3**: Largest synthetic dataset ever assembled
- **NeoTrix**: `nt_mind::distillation_engine` — structured knowledge transfer pipeline

### P5: KV Cache Optimization for Long Context
- **DeepSeek V4**: CSA compression (4:1) + lightning indexer
- **Qwen3-Next**: Gated DeltaNet O(n) attention
- **Mistral Large 3**: Multi-Latent Attention compression
- **Yi-Lightning**: Cross-layer KV cache sharing
- **NeoTrix**: `nt_memory::kv_cache_optimizer` — Paged KV virtualization (KVMem pattern)

---

## NeoTrix Integration Roadmap

### Immediate (P0 — this cycle)
| Pattern | Source Model | NeoTrix Component | Action |
|---------|-------------|-------------------|--------|
| Hybrid attention | DeepSeek V4, Qwen3 | `kv_cache_optimizer` | Implement CSA+HCA layer alternation |
| Thinking modes | Gemini 2.5, Qwen3 | `consciousness_tree` | Add resource-aware mode switching |
| Ultra-sparse MoE | Qwen3-Next | `capability_bridge` | 3.7% activation routing |

### Short-term (P1 — next 2 cycles)
| Pattern | Source Model | NeoTrix Component | Action |
|---------|-------------|-------------------|--------|
| Synthetic data pipeline | Phi-4, Yi-Lightning | `distillation_engine` | Structured synthetic generation |
| Cross-layer KV sharing | Yi-Lightning | `kv_cache_optimizer` | KV deduplication |
| Speculative decoding | Mistral Large 3 | `llm_provider` | Draft-verify pattern |

### Medium-term (P2 — next 5 cycles)
| Pattern | Source Model | NeoTrix Component | Action |
|---------|-------------|-------------------|--------|
| Real-time search | Grok 3 DeepSearch | `unified_crawler` | Live information integration |
| Neuro-symbolic | Grok 3 | `e8_hexagram` | Symbolic + neural fusion |
| Edge deployment | Mistral Ministral | `power_management` | On-device inference |

---

## Axiom Alignment

| NeoTrix Axiom | Source Models | Evidence |
|---------------|--------------|---------|
| **A1: Cost-Aware Routing** | Qwen3 (3.7% activation), DeepSeek (4.6%) | Ultra-sparse MoE proves not all tasks need full model |
| **A2: Context as Scarce Resource** | DeepSeek (CSA), Qwen3 (DeltaNet), Mistral (MLA) | All optimize KV cache for efficiency |
| **A3: Skill as Production Template** | Phi-4 (synthetic data), Qwen3 (MCP) | Structured knowledge transfer as production pattern |

---

## Appendix: Source Verification

| Model | Primary Sources | Confidence |
|-------|----------------|------------|
| GPT-4o | OpenAI System Card (arXiv:2410.21276), DeepWiki | High (architecture undisclosed, signals inferred) |
| Claude 3.5 Sonnet | Anthropic release, Model Card Addendum | High (architecture undisclosed, performance verified) |
| Gemini 2.5 Pro | Google DeepMind blog, Technical Report, Vertex AI docs | High |
| Llama 4 Scout | Meta MODEL_CARD.md, ai.meta.com blog, HuggingFace | High (open weights, full specs) |
| DeepSeek V4 Flash | NVIDIA NIM, HuggingFace, vLLM docs, Technical Report | High (open weights, full architecture disclosed) |
| Qwen3 | Alibaba Cloud blog, arXiv:2505.09388, HuggingFace | High (open weights, full specs) |
| Mistral Large 3 | Mistral AI docs, NVIDIA NIM, Red Hat docs | High (open weights, Apache 2.0) |
| Phi-4 Reasoning | Microsoft Research, HuggingFace, arXiv:2504.21318 | High (open weights, full report) |
| Yi-Lightning | arXiv:2412.01253, AI Wiki | Medium (architecture disclosed, some params estimated) |
| Grok 3 | xAI docs, Azure Foundry, Perplexity reports | Medium (architecture partially disclosed, some signals inferred) |
