# Model Architecture Reverse Engineering #307

**Date**: 2026-09-11
**Scope**: 10 frontier LLM architectures — innovation extraction + NeoTrix mapping
**Sources**: Official technical reports, system cards, model cards, blog posts

---

## Executive Summary

This batch covers 10 models spanning 2024-2026: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3. Key industry trends: **MoE dominance** (8/10 models), **native multimodality** (7/10), **thinking/reasoning modes** (6/10), **extreme context windows** (up to 10M tokens), **KV cache compression** (DS V4, Yi-Lightning), **data-centric training** (Phi-4, Qwen 3).

---

## 1. GPT-4o (OpenAI, May 2024)

### Architecture
- **Type**: Autoregressive omni model, end-to-end multimodal
- **Parameters**: Undisclosed (estimated ~200B dense or MoE)
- **Context**: 128K tokens
- **Modalities**: Text + Audio + Image + Video (all in, all out)
- **Key innovation**: Single neural network trained jointly across text, vision, audio — no staged CLIP pipeline

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Unified Token Stream** | Text BPE + image patch tokens + audio codec tokens (neural audio codec, ~75Hz) consumed by single transformer stack |
| **End-to-End Multimodal Training** | Joint training across all modalities instead of staged encoder→decoder pipeline |
| **232ms Audio Latency** | Median audio response time comparable to human conversation latency (vs 2.8s in prior staged pipeline) |
| **Modality-Specific Embedding/Unembedding** | Separate input embedding tables per modality, shared transformer backbone |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Unified token stream | `nt_world_sense::perception_bridge` | Attention-gated bridge between perception and consciousness — same principle of unified representation |
| End-to-end multimodal | `nt_core::e8_hexagram` | E8's hexagram grid as universal reasoning substrate across modalities |
| Audio latency optimization | `nt_io::platform_gateway` | Ordered backend router for inference path selection |
| Joint modality training | `nt_mind::seal_pipeline` | SEAL's cross-domain distillation mirrors joint multimodal training |

---

## 2. Claude 3.5 Sonnet (Anthropic, Jun 2024)

### Architecture
- **Type**: Dense transformer, multimodal input (text + image), text output
- **Parameters**: Undisclosed
- **Context**: 200K tokens (up to 1M tested)
- **Modalities**: Text + Image input, Text output
- **Key innovation**: Constitutional AI alignment + agentic coding (64% on internal SWE eval)

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Constitutional AI (CAI)** | Rule-based alignment using explicit principles (UN Declaration, disability rights) instead of pure RLHF |
| **Agentic Coding** | 64% on real-world PR tasks (search, view, edit 3-20 files, run tests, self-correct) |
| **Computer Use** | Screenshot interpretation → GUI command generation (14.9% OSWorld, 22% with more steps) |
| **ASL-2 Safety Framework** | Tiered safety classification with quantitative thresholds of concern |
| **200K Context** | Production context window with strong NIAH and QuaLITY performance |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Constitutional AI | `nt_governance::steward` | Gov-Steward enforces principle-level rules — identical governance pattern |
| Agentic coding | `nt_act::implementer` | Dev-Implementer skill node — TDD + agentic loop |
| Computer use | `nt_world::unified_crawler` | GUI interaction as perception-action loop |
| Safety framework | `nt_shield::risk_assessor` | RiskAssessor with RiskLevel grading (Safe/Moderate/Risky/Protected) |
| Long context | `nt_memory::kv_cache_optimizer` | KVMem paged KV for >256K sessions |

---

## 3. Gemini 2.5 Pro (Google DeepMind, Mar 2025)

### Architecture
- **Type**: Sparse MoE transformer, natively multimodal
- **Parameters**: Undisclosed (MoE, activate subset per token)
- **Context**: 1M+ tokens
- **Modalities**: Text + Audio + Image + Video input
- **Key innovation**: Thinking model with controllable budget, 3-hour video processing, TPUv5p training

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Sparse MoE + Native Multimodal** | Dynamic routing to subset of parameters per token, native text/audio/video input |
| **Thinking Budget Control** | User-configurable inference-time compute allocation, scaling accuracy with budget |
| **1M+ Token Context** | Process entire codebases, 3-hour videos, full books |
| **TPUv5p Training** | 8960-chip pods, slice-granularity elasticity (97% throughput during recovery), split-phase SDC detection |
| **K-Sparse Distillation** | Approximate teacher distribution with k-sparse vocabulary for smaller models |
| **Pathways System** | Single-controller design enabling elastic training across datacenters |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| MoE routing | `nt_core::gwt` | GWT salience-based attention routing — same dynamic selection principle |
| Thinking budget | `nt_core::consciousness_tree` | 6-stage feedback loop with attention modulation |
| 1M context | `nt_memory::kv_cache_optimizer` | KVMem paged KV virtualization for >256K |
| TPU elasticity | `nt_physical::power_management` | Resource budget management with fault tolerance |
| Distillation | `nt_mind::skill_crystallization` | SEAL pipeline distillation → skill crystallization |

---

## 4. Llama 4 Scout (Meta, Apr 2025)

### Architecture
- **Type**: Sparse MoE with early fusion multimodality
- **Parameters**: 17B active, 109B total (16 experts)
- **Context**: 10M tokens (!)
- **Modalities**: Multilingual text + Image input, Text + Code output
- **Key innovation**: iRoPE architecture for infinite context, 10M context window, single H100 deployment

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **iRoPE Architecture** | Interleaved attention layers without positional embeddings + inference-time temperature scaling for length generalization |
| **Early Fusion Multimodality** | Joint pre-training on unlabeled text/image/video via early token fusion (not CLIP-style staged) |
| **10M Token Context** | Industry-leading context length via mid-training extension from 256K |
| **Alternating Dense/MoE Layers** | Alternating dense and MoE layers for inference efficiency |
| **128 Routed Experts + 1 Shared Expert** | Each token → shared expert + 1 of 128 routed experts |
| **FP8/Int4 Quantization** | Single H100 deployment via on-the-fly int4 quantization |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| iRoPE | `nt_core::hypercube` | VSA HyperCube's high-dimensional encoding — position-free representation |
| Early fusion | `nt_world::perception_bridge` | Attention-gated bridge unifying perception streams |
| 10M context | `nt_memory::kv_cache_optimizer` | KVMem paged KV + step-level scheduling |
| MoE routing | `nt_core::gwt` | GWT broadcasts salient info across specialist modules |
| Quantization | `nt_physical::power_management` | Resource-aware compute allocation |

---

## 5. DeepSeek V4 Flash (DeepSeek, Apr 2026)

### Architecture
- **Type**: Sparse MoE (DeepSeekMoE) with hybrid attention
- **Parameters**: 284B total, 13B activated
- **Context**: 1M tokens
- **Modalities**: Text
- **Key innovation**: CSA+HCA hybrid attention, Muon optimizer, mHC residual connections, FP4 routed experts

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **CSA+HCA Hybrid Attention** | Compressed Sparse Attention (compress m tokens→1 entry, then top-k sparse) + Heavily Compressed Attention (extreme compression m'>>m) — interleaved for 1M context at 10% KV cache of V3.2 |
| **Manifold-Constrained Hyper-Connections (mHC)** | Constrain residual mapping onto manifold for stable signal propagation across layers |
| **Muon Optimizer** | Faster convergence + training stability vs AdamW |
| **Multi-Token Prediction (MTP)** | Predict multiple future tokens per step (inherited from V3) |
| **FP4 Routed Experts** | MoE expert parameters at FP4 precision, future hardware 3x efficiency |
| **Domain Expert Isolation** | Independent SFT+RL per domain (math/code/agent), then on-policy distillation consolidation |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| CSA+HCA hybrid attention | `nt_memory::kv_cache_optimizer` | KVMem's GPU→Host→NVMe tiered KV — same compression philosophy |
| mHC | `nt_core::e8_hexagram` | E8's stable manifold geometry for signal propagation |
| Muon optimizer | `nt_mind::seal_pipeline` | SEAL's optimization dynamics |
| MTP | `nt_core::consciousness_tree` | Tree's forward-looking branch prediction |
| FP4 precision | `nt_physical::power_management` | Resource-aware compute allocation |
| Domain expert isolation | `nt_mind::skill_crystallization` | Per-domain skill nodes with cross-domain synthesis |

---

## 6. Qwen 3 (Alibaba, May 2025)

### Architecture
- **Type**: Dense + MoE variants, 0.6B to 235B parameters
- **Parameters**: 235B total / 22B activated (flagship MoE)
- **Context**: 128K tokens (up to 1M in later versions)
- **Modalities**: Text, 119 languages
- **Key innovation**: Unified thinking/non-thinking modes with budget control, strong-to-weak distillation

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Dual-Mode Architecture** | Single model with thinking (CoT) + non-thinking (direct) modes, switchable via chat template |
| **Thinking Budget Control** | User-specified token budget for reasoning depth, smooth performance scaling |
| **Strong-to-Weak Distillation** | Flagship (235B) → small models (0.6B-14B) via on-policy KL divergence minimization |
| **128 Experts, 8 Activated** | MoE with global-batch load balancing loss (no shared experts) |
| **36T Token Pre-training** | 2x predecessor data, 119 languages, 3-stage curriculum (S1: basic, S2: knowledge, S3: long context) |
| **QK-Norm** | Remove QKV-bias, add QK-Norm for training stability |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Dual-mode thinking | `nt_core::consciousness_tree` | 6-stage loop with thinking/non-thinking routing |
| Thinking budget | `nt_core::gwt` | GWT salience modulation — allocate attention by task complexity |
| Strong-to-weak distillation | `nt_mind::skill_crystallization` | SEAL distillation pipeline |
| Global-batch load balancing | `nt_act::parallel_task` | ParallelTaskManager load balancing across GPUs |
| Multi-language | `nt_world::unified_crawler` | Multi-language content processing |

---

## 7. Mistral Large 3 (Mistral AI, Dec 2025)

### Architecture
- **Type**: Granular sparse MoE with native vision
- **Parameters**: 675B total, 41B active
- **Context**: 256K tokens
- **Modalities**: Text + Image input, Text output
- **Key innovation**: Granular MoE + 2.5B vision encoder + NVFP4 deployment on single node

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Granular MoE** | Extremely fine-grained expert routing with 675B total / 41B active (~16:1 ratio) |
| **Native Vision Encoder** | 2.5B parameter vision encoder fused into model (not adapter) |
| **NVFP4 Deployment** | Full 675B model on single 8×H100 node via FP4 quantization |
| **Blackwell-Optimized Kernels** | NVIDIA co-designed attention + MoE kernels for GB200 NVL72 |
| **Speculative Decoding** | Inference acceleration via draft model |
| **Apache 2.0 Open Weight** | Frontier-level model fully open-sourced |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Granular MoE | `nt_core::gwt` | GWT's specialist module routing — same selection principle |
| Native vision | `nt_world::perception_bridge` | PerceptionBridge connecting sensory input to consciousness |
| NVFP4 deployment | `nt_physical::power_management` | Resource-aware compute allocation |
| Speculative decoding | `nt_core::consciousness_tree` | Tree's branch prediction — speculatively explore paths |
| Open weight | `nt_memory::kb` | KB as open, persistent knowledge store |

---

## 8. Phi-4 Reasoning (Microsoft, Apr 2025)

### Architecture
- **Type**: Dense decoder-only transformer (14B)
- **Parameters**: 14B
- **Context**: 32K tokens (extended from 16K)
- **Modalities**: Text
- **Key innovation**: Small model punching above weight via data curation + o3-mini distillation + GRPO RL

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Teachable Prompt Selection** | Prompts selected at boundary of base model capability for maximum learning |
| **o3-mini Distillation** | High-quality reasoning traces from o3-mini as SFT supervision |
| **Thinking Tokens** | Dedicated `<think>` / `</think>` markers repurposed from placeholder tokens |
| **RoPE Base Frequency Doubling** | Extend context from 16K→32K for reasoning traces |
| **GRPO Reinforcement Learning** | Outcome-based RL on 6K math problems → 1.5x longer responses, higher accuracy |
| **Reasoning as Transferable Meta-skill** | Improvements transfer to out-of-domain tasks (planning, spatial, algorithmic) |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Teachable prompts | `nt_mind::experience_tree` | Experience selection at capability boundary — same frontier principle |
| Distillation | `nt_mind::skill_crystallization` | SEAL distillation from flagship → smaller nodes |
| Thinking tokens | `nt_core::consciousness_tree` | ConsciousnessTree's explicit reasoning stages |
| GRPO RL | `nt_mind::seal_pipeline` | SEAL's self-test + feedback loop |
| Transferable reasoning | `nt_core::gwt` | GWT broadcasts insights across specialist modules |

---

## 9. Yi-Lightning (01.AI, Dec 2024)

### Architecture
- **Type**: Enhanced sparse MoE
- **Parameters**: Undisclosed (fine-grained expert segmentation)
- **Context**: Not specified (extended via KV cache optimization)
- **Modalities**: Text
- **Key innovation**: Fine-grained expert segmentation + PEP load balancing + 82.8% KV cache reduction

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Fine-Grained Expert Segmentation** | Partition each expert's FFN into smaller units, reduce hidden dims, increase activated experts per token |
| **PEP Load Balancing** | Partitioned Expert Parallel balancing within EP groups — addresses All-to-All communication imbalance |
| **Hybrid Attention Blocks** | 3 sliding window attention layers + 1 full attention layer per block |
| **Cross-Layer KV Cache Reuse** | Share KV cache between consecutive full attention layers → 50% memory reduction |
| **82.8% Memory Reduction** | Combined hybrid attention + KV cache reuse for long sequences |
| **RAISE Safety Engine** | 4-component safety: pre-training filter + post-training optimization + input safety + output safety |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Expert segmentation | `nt_core::skill_tree` | Skill tree's Small/Notable/Keystone node tiers — granular capability decomposition |
| PEP load balancing | `nt_act::parallel_task` | ParallelTaskManager GPU load balancing |
| Hybrid attention | `nt_core::gwt` | GWT's salience-based routing — local vs global attention |
| KV cache reuse | `nt_memory::kv_cache_optimizer` | KVMem's paged KV virtualization |
| RAISE safety | `nt_shield::risk_assessor` | RiskAssessor + CleanupCoordinator safety framework |

---

## 10. Grok 3 (xAI, Feb 2025)

### Architecture
- **Type**: Transformer-based LLM (dense or hybrid dense/MoE)
- **Parameters**: Undisclosed (~1.2T estimated by some analyses)
- **Context**: 131K tokens (API), marketed 1M capability
- **Modalities**: Text (Grok 3), Text+Image (later versions)
- **Key innovation**: Massive RL-scale training on Colossus (100K+ H100s), Think mode, DeepSearch agent

### Key Innovations
| Innovation | Description |
|-----------|-------------|
| **Colossus Training** | 100K+ H100 GPUs, 10x compute of predecessors, 200M GPU hours |
| **Think Mode** | Chain-of-thought reasoning with self-correction, 93.3% AIME 2025 |
| **DeepSearch Agent** | Agentic web+X research with real-time synthesis |
| **Cross-Expert Attention Gates** | Knowledge sharing between MoE experts without catastrophic interference |
| **1M Context Marketing** | Long-context capability for document/code processing |
| **Elo 1402** | Top Chatbot Arena performance |

### NeoTrix Mapping
| Innovation | NeoTrix Component | Mapping |
|-----------|-------------------|---------|
| Colossus scale | `nt_physical::power_management` | ResourceBudgetManager for massive compute orchestration |
| Think mode | `nt_core::consciousness_tree` | 6-stage tree with explicit reasoning phases |
| DeepSearch | `nt_world::unified_crawler` | UnifiedCrawler with multi-source intelligence synthesis |
| Cross-expert gates | `nt_core::gwt` | GWT's cross-module broadcast mechanism |
| 1M context | `nt_memory::kv_cache_optimizer` | KVMem paged KV + step-level scheduling |

---

## Cross-Model Innovation Matrix

| Innovation | GPT-4o | Claude 3.5 | Gemini 2.5 | Llama 4 | DS V4 | Qwen 3 | Mistral L3 | Phi-4r | Yi-Ltng | Grok 3 |
|-----------|--------|------------|------------|---------|-------|--------|------------|--------|---------|--------|
| **MoE Architecture** | ? | - | ✅ | ✅ | ✅ | ✅ | ✅ | - | ✅ | ? |
| **Native Multimodal** | ✅ | Input | ✅ | ✅ | - | - | Input | - | - | Partial |
| **Thinking/Reasoning Mode** | - | - | ✅ | - | ✅ | ✅ | - | ✅ | - | ✅ |
| **>256K Context** | - | ✅ | ✅ | ✅(10M) | ✅(1M) | ✅(1M) | ✅(256K) | - | - | Partial |
| **KV Cache Compression** | - | - | - | - | ✅ | - | - | - | ✅ | - |
| **Data Curation Focus** | - | - | - | - | - | ✅ | - | ✅ | - | - |
| **Open Weights** | - | - | - | ✅ | ✅ | ✅ | ✅ | ✅ | - | - |
| **Agentic Capabilities** | - | ✅ | ✅ | - | ✅ | ✅ | ✅ | - | - | ✅ |

---

## Industry Trend Synthesis (Batch 307)

### Trend 1: MoE is the Default
8/10 models use or are suspected MoE. The industry has converged on sparse activation for compute efficiency. Qwen 3 and Mistral Large 3 show MoE can be open-sourced at frontier scale.

### Trend 2: Thinking is a Feature, Not a Model
Gemini 2.5, Qwen 3, and DeepSeek V4 all offer thinking/non-thinking modes in a single model with budget control. This eliminates the need for separate reasoning models (contrasting with OpenAI's o-series separation).

### Trend 3: Context Windows are Exploding
Llama 4 Scout at 10M, DeepSeek V4 at 1M, Gemini at 1M+. The bottleneck has shifted from context length to **efficient attention** (CSA/HCA, hybrid sliding window).

### Trend 4: Small Models Can Reason
Phi-4 Reasoning (14B) outperforms DeepSeek-R1-Distill-Llama-70B. Data curation + distillation > raw scale. Qwen 3-4B rivals Qwen2.5-72B-Instruct.

### Trend 5: Safety is Structuring
Anthropic's ASL framework, Yi-Lightning's RAISE, and Qwen 3's multi-stage alignment all show safety moving from ad-hoc to structured frameworks with quantitative thresholds.

---

## NeoTrix Absorption Targets

Based on this batch, the following NeoTrix components should absorb external innovations:

| Priority | Innovation | Source | Target Component | Action |
|----------|-----------|--------|-----------------|--------|
| **P0** | Hybrid CSA+HCA attention | DeepSeek V4 | `kv_cache_optimizer.rs` | Implement tiered compression |
| **P0** | Thinking budget control | Qwen 3, Gemini 2.5 | `consciousness_tree.rs` | Add budget-aware attention modulation |
| **P1** | Fine-grained expert segmentation | Yi-Lightning | `skill_tree.rs` | Granular node decomposition |
| **P1** | mHC residual connections | DeepSeek V4 | `e8_hexagram.rs` | Manifold-constrained signal propagation |
| **P1** | Teachable prompt selection | Phi-4 Reasoning | `experience_tree.rs` | Frontier-boundary experience selection |
| **P2** | PEP load balancing | Yi-Lightning | `parallel_task.rs` | Partitioned EP group balancing |
| **P2** | Domain expert isolation + distillation | DeepSeek V4 | `skill_crystallization.rs` | Per-domain training → consolidation |
| **P2** | Cross-expert attention gates | Grok 3 | `gwt.rs` | Cross-module knowledge sharing gates |

---

*Generated by NeoTrix model-reverse-engineer pipeline, batch #307*
*Sources: GPT-4o System Card (arXiv:2410.21276), Claude 3 Model Card, Gemini 2.5 Technical Report (arXiv:2507.06261), Llama 4 Model Card, DeepSeek-V4 Paper (arXiv:2606.19348), Qwen3 Technical Report (arXiv:2505.09388), Mistral Large 3 Documentation, Phi-4-reasoning Technical Report (arXiv:2504.21318), Yi-Lightning Technical Report (arXiv:2412.01253), Grok 3 Announcement (x.ai)*
