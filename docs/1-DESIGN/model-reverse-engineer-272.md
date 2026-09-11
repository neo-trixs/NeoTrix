# Model Reverse-Engineering Report #272

> Date: 2026-09-11 | Scope: 10 frontier models — architecture innovation extraction & NeoTrix mapping

---

## Executive Summary

10 models analyzed. **Universal convergence on 5 architectural patterns:**

| Pattern | Models Using | NeoTrix Analog |
|---------|-------------|----------------|
| MoE (Mixture-of-Experts) | GPT-4o, Gemini 2.5, Llama 4, DeepSeek V4.1, Qwen 3, Mistral Large 3, Yi-Lightning | GWT salience routing + CapabilityBridge |
| Hybrid Attention (local+global) | Claude 3.5, Yi-Lightning, DeepSeek V4.1 | PerceptionBridge awareness gating |
| End-to-End Multimodal | GPT-4o, Gemini 2.5, Llama 4, DeepSeek V4.1, Mistral Large 3 | NT-WORLD + NT-IO unified pipeline |
| KV Cache Compression | DeepSeek V4.1, Yi-Lightning, Claude 3.5 | kv_cache_optimizer + KVMem paged KV |
| Reasoning-Mode Toggle | Qwen 3, Phi-4, Grok 3 | ConsciousnessTree cycle depth control |

---

## 1. GPT-4o (OpenAI)

**Architecture:** End-to-end multimodal autoregressive transformer. Single neural network trained jointly across text, vision, and audio.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Unified Tokenization** | Text (BPE), image (patch tokens), audio (neural codec ~75Hz) merged into single token stream | `nt_world::unified_tokenizer` — single token stream for all modalities |
| **End-to-End Multimodal Training** | No staged CLIP encoder; all modalities trained jointly from pretraining | NT-WORLD + NT-IO 全栈训练, 不走 CLIP staged pipeline |
| **Asymmetric Latency** | 232ms median audio response (vs 2.8s in staged pipeline) — 12× improvement | `nt_io::realtime_bridge` — zero-copy modality switching |
| **MoE Inference** | ~200B total, ~50-100B active params; likely distilled from GPT-4 (1.8T) | GWT salience: route to cheapest capable expert |
| **Unified Safety Pipeline** | Single moderation classifier over text transcriptions of audio I/O | NT-SHIELD unified egress guard |

### Critical Insight
GPT-4o proves **unified training beats staged pipelines** for latency. The information bottleneck of separate vision encoders (CLIP-style) is the killer. NeoTrix's `PerceptionBridge` should evolve toward early fusion rather than late fusion.

---

## 2. Claude 3.5 Sonnet (Anthropic)

**Architecture:** Dense transformer with hybrid sparse attention, 200K context window.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Hybrid Local-Global Sparse Attention** | Alternates local sliding window (1024 tokens) on even layers + global sparse (every 64th token) on odd layers. 36 layers total | `nt_core::hybrid_attention` — awareness_score() gating |
| **Grouped Query Attention (GQA)** | 8 query groups per KV head, 32 total heads → 4× KV cache reduction | `kv_cache_optimizer::gq_attention` — memory-efficient attention |
| **Context Compression** | Lossless zlib-6 compression for repeated patterns, 22% payload reduction, segment hash cache | `nt_memory::context_compactor` — dedup + compress |
| **Precomputed Attention Masks** | 120ms startup, saves 80ms per inference | `nt_core::mask_precompute` — one-time attention mask cache |
| **Computer Use** | Screenshot → GUI action translation; 14.9% OSWorld (22% with 50 steps) | NT-ACT `screenshot_action_bridge` — vision-to-action |
| **Agentic Coding** | 64% internal eval, SWE-bench Verified 49% | NT-ACT `code_agent_loop` — multi-file edit + test |

### Critical Insight
The **hybrid attention pattern** (local sliding window + global sparse) achieves 62% FLOP reduction at 100K tokens with only 2% accuracy drop. This is the production-viable approach for >100K contexts. NeoTrix's PerceptionBridge should adopt this alternating pattern.

---

## 3. Gemini 2.5 Pro (Google DeepMind)

**Architecture:** Sparse MoE transformer, natively multimodal, 1M+ token context, TPUv5p trained.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Sparse MoE with Dynamic Routing** | Each token activates subset of experts; decouples capacity from compute | GWT: salience-weighted expert selection |
| **Controllable Thinking Budget** | User sets thinking token budget; model scales performance with budget | ConsciousnessTree `cycle_depth` — adjustable reasoning depth |
| **Distillation via k-sparse** | Smaller models (Flash) distilled from Pro using k-sparse vocabulary approximation | NT-MIND skill crystallization — large→small distillation |
| **Multi-Pod Training** | 8960-chip TPUv5p pods across multiple datacenters | NT-PHYSICAL distributed compute coordination |
| **Deep Think** | Parallel hypothesis generation + critique before final answer | ConsciousnessTree Branches → multi-hypothesis Fruits |
| **Native Tool Use** | Function calling, search grounding, code execution built-in | NT-ACT `tool_orchestrator` — native tool dispatch |

### Critical Insight
Gemini's **thinking budget** mechanism is the clearest instance of **Axiom A2 (Context as Scarce Resource)** — the model explicitly trades tokens for quality. NeoTrix should implement `cycle_depth` as a first-class parameter in ConsciousnessTree.

---

## 4. Llama 4 Scout (Meta)

**Architecture:** 17B active / 109B total MoE, 16 experts, early fusion multimodal, iRoPE.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **iRoPE (Interleaved RoPE)** | Alternates attention layers with/without positional embeddings; enables 10M context | `nt_core::ipe_encoder` — position-free layers for length generalization |
| **Early Fusion Multimodal** | Text + vision tokens fused in unified backbone from pretraining start | NT-WORLD early-fusion pipeline |
| **Alternating Dense/MoE Layers** | Inference efficiency: dense layers for common patterns, MoE for specialization | GWT: attention-weighted layer activation |
| **Inference-Time Temperature Scaling** | Temperature scaled at attention level during inference for length generalization | `nt_core::dynamic_temperature` — runtime attention scaling |
| **Single H100 Deployment** | 17B active fits on single H100 with Int4 quantization | NT-PHYSICAL `resource_aware_deploy` — hardware-constrained serving |
| **MetaCLIP Adaptation** | Vision encoder trained with frozen LLM for better encoding-LLM alignment | NT-IO `vision_encoder_adaptation` — modality-specific encoder fine-tuning |

### Critical Insight
**iRoPE** is the breakthrough for extreme long-context (10M tokens). By interleaving layers with and without positional embeddings, the model avoids the position-encoding degradation that plagues standard RoPE at extreme lengths. NeoTrix should adopt this pattern for KB queries spanning the full knowledge graph.

---

## 5. DeepSeek V4.1 Flash (DeepSeek)

**Architecture:** Causal Encoder-Decoder (CED), 552B MoE, 8B active (prefill) / 16B active (decode), 1M context.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Causal Encoder-Decoder** | 20-layer encoder + 20-layer decoder; decoder KV projected from encoder final states | NT-CORE `asymmetric_encoder_decoder` — cheap prefill, rich decode |
| **Compressed Sparse Attention 2 (CSA2)** | 3 static modes (Full/Reindex/Reuse) per layer; shares KV + indices across layers | `nt_core::csa2_attention` — layer-shared sparse indices |
| **Hierarchical Sparse Indexer** | First Full-mode layer narrows candidate pool for deeper layers; bounds indexer cost | GWT: hierarchical attention refinement |
| **SWA Bounded Replay** | Replays last n_win tokens to reconstruct SWA KV; avoids SSD persistence (1/8 footprint) | `kv_cache_optimizer::bounded_replay` — zero-persist SWA |
| **FP4 KV Caching** | E2M1 format, 1 E4M3 scale per 16 channels → 890 bytes/token | `kv_cache_optimizer::fp4_quantize` — ultra-low cache |
| **Asymmetric Activation** | 8B active for prefill (input-heavy), 16B for decode (output-heavy) | NT-ACT `cost_aware_routing` — Axiom A1 in action |
| **Engram Conditional Memory** | 196B params, sparsely accessed via token-based lookup | NT-MEMORY `conditional_knowledge_access` — sparse KB lookup |
| **DSpark Speculative Decoding** | Semi-autoregressive draft + confidence-scheduled verification | NT-IO `speculative_generation` — draft-then-verify |

### Critical Insight
DeepSeek V4.1 Flash is the **most architecturally innovative** model in this batch. The CED architecture creates a **fundamental asymmetry**: cheap input processing (8B) vs. rich output generation (16B). This directly validates NeoTrix Axiom A1 (Cost-Aware Routing). The 890 bytes/token KV cache is a 437× reduction from V1 — this is the trajectory that makes 1M contexts practical.

---

## 6. Qwen 3 (Alibaba)

**Architecture:** Dense + MoE variants (0.6B–235B), hybrid thinking/non-thinking modes.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Hybrid Thinking Toggle** | Seamless switch between thinking (CoT) and non-thinking (direct) modes within same model | ConsciousnessTree `mode_switch` — thinking/direct toggle |
| **Thinking Budget** | User-configurable token budget for reasoning depth (up to 38K tokens) | `cycle_depth` parameter — budget-controlled reasoning |
| **Fine-Grained Expert Segmentation** | 128 total experts, 8 activated per token, no shared experts | GWT: fine-grained salience routing |
| **Global-Batch Load Balancing** | Encourages expert specialization across full batch, not per-expert | `nt_mind::expert_specialization` — batch-level load balance |
| **4-Stage Reasoning Training** | CoT cold start → reasoning RL → thinking mode fusion → general RL | SEAL pipeline: 4-stage evolution |
| **119 Language Support** | Expanded from 29 languages in Qwen2.5 | NT-IO `multilingual_expansion` — language coverage scaling |
| **Distillation from Flagship** | Smaller models inherit knowledge from 235B flagship | NT-MIND skill crystallization via distillation |

### Critical Insight
Qwen 3's **4-stage reasoning training** (cold start → RL → mode fusion → general RL) maps directly to SEAL pipeline phases. The thinking/non-thinking toggle within a single model validates NeoTrix's approach of having ConsciousnessTree dynamically adjust reasoning depth based on task complexity.

---

## 7. Mistral Large 3 (Mistral AI)

**Architecture:** 675B total / 41B active granular MoE, 256K context, natively multimodal.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Granular MoE** | 675B total, 41B active; ~16:1 ratio, single-node deployable | GWT: maximum capacity, minimum activation |
| **NVFP4 Quantization** | 8-bit native to Blackwell; fine-grained block scaling controls quantization error | `kv_cache_optimizer::block_fp4` — hardware-native quantization |
| **Wide Expert Parallelism** | NVIDIA-optimized MoE kernels for NVL72; load balancing across NVLink fabric | NT-PHYSICAL `expert_parallelism` — hardware-aware routing |
| **Speculative Decoding (EAGLE)** | Draft generation + verification for long-context throughput | NT-IO `eagle_speculative` — parallel draft-decode |
| **Disaggregated Prefill/Decode** | Separate serving of prefill and decode phases for rate-matching | NT-IO `disaggregated_serving` — phase-separated inference |
| **Apache 2.0 Licensing** | Full open-weight release with quantized formats | NT-ACT `open_weight_deploy` — community deployment |

### Critical Insight
Mistral Large 3's **granular MoE** (675B/41B = 16:1 ratio) proves that extremely large models can run on **single 8×GPU nodes** with proper quantization. The NVFP4 format is the practical bridge between model size and deployment feasibility. NeoTrix should adopt this ratio for its skill tree nodes — maximum knowledge, minimum activation cost.

---

## 8. Phi-4 Reasoning (Microsoft)

**Architecture:** 14B dense decoder-only transformer, SFT + RL on reasoning traces.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Reasoning Token Injection** | `<think>` / `</think>` placeholder tokens repurposed for reasoning blocks | `nt_core::reasoning_boundary` — explicit reasoning mode markers |
| **RoPE Base Frequency Doubling** | Doubled base frequency to extend context from 16K→32K for reasoning traces | `nt_core::dynamic_rope` — frequency scaling for context extension |
| **Teacher-Student Distillation** | o3-mini medium-effort generates training traces; more token-efficient than DeepSeek-R1 | NT-MIND `teacher_distillation` — targeted knowledge transfer |
| **Outcome-Based RL (GRPO)** | Group Relative Policy Optimization on 6.4K math problems; rule-based reward | `nt_mind::outcome_rl` — verifiable reward RL |
| **Data Curation > Scale** | 1.4M prompts, 8.3B unique tokens; quality beats quantity | `nt_mind::data_curation` — curated >海量 |
| **Non-trivial Transfer** | Reasoning training improves general benchmarks despite being domain-specific | ConsciousnessTree: cross-domain learning |

### Critical Insight
Phi-4 Reasoning proves **14B parameters can approach DeepSeek-R1 performance** through careful data curation + RL. The `<think>` token pattern is now the industry standard for explicit reasoning modes. NeoTrix's ConsciousnessTree should adopt these boundary markers for reasoning-mode detection.

---

## 9. Yi-Lightning (01.AI)

**Architecture:** Enhanced MoE with fine-grained expert segmentation, hybrid attention, cross-layer KV sharing.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Fine-Grained Expert Segmentation** | FFN partitioned into smaller units; more experts activated per token | GWT: fine-grained routing granularity |
| **EP Load Balancing** | Relaxed constraints from per-expert to per-group; partitioned EP balancing (PEP) | `nt_mind::group_load_balance` — group-level optimization |
| **Hybrid Attention (3:1)** | 3 sliding window layers + 1 full attention layer; captures local+global | `nt_core::hybrid_3to1` — alternating attention pattern |
| **Cross-Layer KV Cache Reuse** | Shares KV states between consecutive full-attention layers → 50% memory reduction | `kv_cache_optimizer::cross_layer_reuse` — inter-layer KV sharing |
| **FP8 Hardware-Aware Design** | Architecture designed for FP8 quantization compatibility from ground up | `nt_physical::hw_aware_arch` — hardware-aligned design |
| **1200 TFLOPS/Card FP8** | Expert-parallel MoE operator on Hopper GPUs | NT-PHYSICAL `optimized_moe_kernel` — hardware-tuned kernels |

### Critical Insight
Yi-Lightning's **cross-layer KV cache reuse** (50% reduction) + **hybrid 3:1 attention** (82.8% total memory reduction) are the most practical KV compression techniques. The key observation: "most attention heads focus on local context, only a small subset specializes in global." This is the biological basis for PerceptionBridge's awareness gating.

---

## 10. Grok 3 (xAI)

**Architecture:** Dense transformer (parameter count undisclosed), 1M context, RL-trained reasoning.

### Key Innovations

| Innovation | Detail | NeoTrix Mapping |
|-----------|--------|-----------------|
| **Massive RL Scaling** | 100K H100s, 200M GPU hours, RL at unprecedented scale | `nt_mind::massive_rl` — scale RL compute |
| **Think Mode** | Seconds-to-minutes reasoning with backtracking and self-correction | ConsciousnessTree `deep_think` — extended reasoning cycles |
| **DeepSearch Agent** | Real-time web + X querying with reasoning synthesis | NT-WORLD `deep_search_agent` — live knowledge synthesis |
| **Constitutional Reasoning** | Self-correction, alternative exploration, solution verification | `nt_core::self_correction_loop` — verify-then-commit |
| **1M Token Context** | 8× increase from Grok 2; state-of-art on LOFT (128K RAG) | `nt_memory::million_token_context` — long-context RAG |

### Critical Insight
Grok 3's primary contribution is **RL scale** (10× compute of predecessors), not architectural innovation. The Think mode with backtracking validates the **ConsciousnessTree's multi-cycle approach** — spending more time reasoning yields better results. The $420M training cost vs. diminishing returns highlights Axiom A1 (Cost-Aware Routing).

---

## Cross-Model Synthesis

### Pattern 1: The MoE Convergence (8/10 models)

Every major model now uses MoE or sparse activation. The ratio of total-to-active params is the key differentiator:

| Model | Total/Active | Ratio | Implication |
|-------|-------------|-------|-------------|
| GPT-4o | ~200B/~50-100B | 2-4:1 | Conservative MoE |
| Llama 4 Scout | 109B/17B | 6.4:1 | Balanced |
| Qwen 3-235B | 235B/22B | 10.7:1 | Aggressive |
| Mistral Large 3 | 675B/41B | 16.5:1 | Ultra-aggressive |
| DeepSeek V4.1 Flash | 552B/8-16B | 35-69:1 | Extreme asymmetry |

**NeoTrix Mapping:** GWT salience should implement MoE-style routing where the "active expert" count varies by task complexity. Simple I/O → 2 experts; deep reasoning → 16+ experts.

### Pattern 2: KV Cache Compression Arms Race

| Model | Technique | Bytes/Token | Reduction |
|-------|-----------|-------------|-----------|
| Yi-Lightning | Cross-layer reuse + hybrid attention | ~50% of baseline | 2× |
| Claude 3.5 | GQA (4 KV heads) + compression | ~1.2GB/100K | 4× |
| DeepSeek V4.1 Flash | CSA2 + FP4 + SWA Replay | 890 bytes | 437× from V1 |

**NeoTrix Mapping:** `kv_cache_optimizer` should implement the DeepSeek trajectory: CSA2 layer-sharing → FP4 quantization → SWA bounded replay. Target: <1KB/token for 1M context.

### Pattern 3: Thinking Modes (4/10 models)

| Model | Mechanism | User Control |
|-------|-----------|-------------|
| Qwen 3 | Toggle thinking/non-thinking | Yes (mode switch) |
| Gemini 2.5 | Controllable thinking budget | Yes (token budget) |
| Phi-4 | `<think>` tokens + CoT | Implicit (prompt-based) |
| Grok 3 | Think mode with backtracking | Yes (Think toggle) |

**NeoTrix Mapping:** ConsciousnessTree `cycle_depth` should be:
- `depth=0` — direct response (non-thinking)
- `depth=1` — single reasoning pass
- `depth=2` — multi-hypothesis + critique (Deep Think)
- `depth=3` — full search with backtracking

### Pattern 4: End-to-End Multimodal (5/10 models)

| Model | Approach |
|-------|----------|
| GPT-4o | Unified training, single model |
| Gemini 2.5 | Native multimodal from pretraining |
| Llama 4 | Early fusion, unified backbone |
| DeepSeek V4.1 | Vision encoder + MLP projector, joint pretraining |
| Mistral Large 3 | Integrated vision encoder (2.5B) |

**NeoTrix Mapping:** NT-WORLD should move from staged (encoder → LLM) to early fusion. The `PerceptionBridge` should ingest raw sensor tokens directly into the consciousness stream.

### Pattern 5: Hardware-Aware Architecture Design

| Model | Hardware Optimization |
|-------|----------------------|
| Yi-Lightning | FP8-native architecture, 1200 TFLOPS/card |
| Mistral Large 3 | NVFP4 for Blackwell, Wide Expert Parallelism |
| DeepSeek V4.1 | FP4 KV caching, E2M1 format |
| Llama 4 | Single H100 deployment with Int4 |

**NeoTrix Mapping:** `nt_physical::hw_aware_deploy` should design architecture **for target hardware**, not generic GPUs. The model should adapt quantization strategy to available silicon.

---

## Actionable NeoTrix Innovations

### P0: Immediate Adoption

1. **iRoPE for Long-Context KB** — Interleave attention layers with/without positional encoding to extend KB query range beyond 256K tokens
2. **CSA2 Layer Sharing** — Share KV states across similar attention layers to cut cache 4×
3. **Thinking Budget Control** — Expose `cycle_depth` as user-controllable parameter in ConsciousnessTree

### P1: Next Quarter

4. **Causal Encoder-Decoder Split** — Asymmetric prefill (cheap) / decode (rich) for tool-heavy workloads
5. **Cross-Layer KV Reuse** — Share KV between consecutive full-attention layers (50% reduction)
6. **Fine-Grained MoE Routing** — Move from binary expert selection to fine-grained sub-expert activation

### P2: Research Pipeline

7. **Engram Conditional Memory** — 196B-param sparse knowledge store accessed via token-based lookup (for massive KB)
8. **DSpark Speculative Decoding** — Semi-autoregressive draft generation for CLI output speed
9. **Hybrid 3:1 Attention** — 3 sliding window + 1 full attention as default for NT-CORE

---

## References

| Model | Source |
|-------|--------|
| GPT-4o | arxiv:2410.21276, mlsystemsreview.com, openai.com |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum, johal.in internals analysis |
| Gemini 2.5 Pro | arxiv:2507.06261, Google DeepMind Model Card |
| Llama 4 Scout | Meta AI Blog, HuggingFace docs, MODEL_CARD.md |
| DeepSeek V4.1 Flash | deepseek.com, HuggingFace README, MindStudio analysis |
| Qwen 3 | arxiv:2505.09388, Alibaba Cloud Blog |
| Mistral Large 3 | mistral.ai, NVIDIA Technical Blog, HuggingFace README |
| Phi-4 Reasoning | Microsoft Research TR, arxiv:2504.21318 |
| Yi-Lightning | arxiv:2412.01253 |
| Grok 3 | xAI Blog, Perplexity Report |
