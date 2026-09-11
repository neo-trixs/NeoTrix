# Model Reverse Engineering — Batch 234 (10 Models)

Date: 2026-09-11
Models: Qwen 3.5, DeepSeek V4.1 Flash, Claude 4 Sonnet, GPT-4o, Gemini 2.5 Flash, Llama 4 Scout, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3

---

## 1. Qwen 3.5 (397B-A17B)

### Architecture
| Property | Value |
|----------|-------|
| Type | Sparse MoE + Hybrid Attention |
| Total Params | 397B |
| Active Params | 17B (4.3% activation) |
| Layers | 60 (15 structural blocks × 4 layers each) |
| Block Layout | 3:1 ratio — 3 Gated DeltaNet + 1 GQA per block |
| Hidden Dim | 4096 |
| Experts | 512 routed + 1 shared, top-10 routing |
| Expert Intermediate | 1024 |
| GQA Heads | 32 Q, 2 KV heads |
| GDN Heads | 64 linear V heads, 16 linear QK heads |
| Head Dim | 128 (GDN), 256 (GQA) |
| Context | 262K native, 1M+ via YaRN RoPE |
| Vocab | 248,320 |
| Multimodal | Native early-fusion (text + image + video) |

### Key Innovations
- **Gated DeltaNet (GDN)**: Linear attention via recurrent memory matrix per head — O(1) memory vs O(S²) for standard attention. Maintains constant-size hidden state per head.
- **Hybrid 3:1 Architecture**: 75% linear attention layers + 25% standard GQA layers. GQA layers anchor retrieval; GDN layers handle bulk of sequence processing.
- **Ultra-sparse MoE**: 512 experts, top-10 = only 2% of expert capacity active per token. 1 shared expert always executes.
- **Hybrid Parallelism**: DP=8 (attention) + EP=8 (MoE), co-designed for TPU v7. Custom SparseCore Ragged Gather kernel eliminates padded intermediate tensors.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| GDN linear attention | `nt_core::perception_bridge` — attention-gated perception with constant memory |
| GDN recurrent memory | `nt_nexus::experience_index` — rolling summary state |
| Shared expert (always-on) | `nt_core::e8_guidance` — shared baseline reasoning across all tasks |
| Top-k routing with load balance | `nt_core::gwt::salience_router` — token→expert mapping with cost-aware selection |
| 3:1 hybrid block layout | `nt_core::attention_manager` — 3 perception layers + 1 cognition layer per cycle |
| YaRN RoPE extension | `nt_memory::context_window` — dynamic context scaling |

---

## 2. DeepSeek V4.1 Flash (552B)

### Architecture
| Property | Value |
|----------|-------|
| Type | Causal Encoder-Decoder (CED) + Sparse MoE |
| Total Params | 552B |
| Active Params (Prefill) | 8B |
| Active Params (Decode) | 16B |
| Layers | 40 (20-layer causal encoder + 20-layer decoder) |
| Experts | 384 routed + 1 shared, top-6 routing |
| KV Cache | 890 bytes/token (FP4 E2M1, per-16-channel E4M3 scale) |
| Context | 1M tokens |
| Pretrain Tokens | 45T |
| Multimodal | Native (DeepSeek-ViT + 2-layer MLP projector) |

### Key Innovations
- **Causal Encoder-Decoder**: Decoder's global KV cache is projected from final encoder hidden states, not derived per decoder layer. Asymmetric activation: 8B input, 16B output.
- **CSA2 (Compressed Sparse Attention 2)**: 3 static modes per layer — Full, Reindex, Reuse. Shares KV and indexer state across layers. Hierarchical Sparse Indexer bounds deeper layer cost independent of context length.
- **SWA Bounded Replay**: Replays only recent window tokens to reconstruct SWA KV states — avoids SSD persistence, cuts persistent KV to 1/8 of predecessor.
- **Engram Conditional Memory**: 196B parameters, sparsely accessed via token-based lookup (not loaded every forward pass).
- **DSpark Speculative Decoding**: Semi-autoregressive draft generation with confidence-scheduled verification.
- **FP4 KV Caching**: E2M1 format + E4M3 per-16-channel scaling — 890 bytes/token = 1/4 of V4-Flash.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| CED asymmetric activation | `nt_core::e8::prefill_vs_decode` — cheap input processing, expensive generation |
| CSA2 layer sharing | `nt_memory::kv_cache_optimizer` — shared KV across attention layers |
| SWA Bounded Replay | `nt_memory::sliding_window_replay` — reconstruct context from recent tokens only |
| Engram conditional memory | `nt_memory::conditional_recall` — sparse lookup from 196B memory bank |
| Hierarchical Sparse Indexer | `nt_core::gwt::hierarchical_attention` — index narrowing across layers |
| DSpark speculative | `nt_core::speculative_reasoning` — draft-then-verify pattern |
| FP4 quantized KV | `nt_memory::quantized_kv` — 4-bit KV caching |

---

## 3. Claude 4 Sonnet (Proprietary)

### Architecture
| Property | Value |
|----------|-------|
| Type | Hybrid Reasoning Transformer |
| Params | Undisclosed |
| Layers | Undisclosed |
| Context | 200K (1M beta) |
| Max Output | 64K |
| Modalities | Text, Vision, PDF |
| Modes | Standard (instant) + Extended Thinking |
| Training | Constitutional AI + RLHF |
| Safety Level | ASL-2 |

### Key Innovations
- **Hybrid Reasoning Modes**: Standard mode for instant responses; Extended Thinking for deep step-by-step reasoning. User-toggleable.
- **Extended Thinking with Tool Use**: Model alternates between reasoning and tool use (web search, code execution) during extended thinking.
- **Thought Summarization**: ~5% of thinking processes trigger a smaller model to summarize lengthy reasoning chains. Developer Mode exposes full thought process.
- **Constitutional AI**: Rule-based alignment framework without relying solely on human feedback.
- **Parallel Tool Execution**: Can invoke multiple tools simultaneously.
- **Local File Memory**: Extracts and saves key facts from developer-provided local files for session continuity.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| Hybrid reasoning modes | `nt_core::gwt::attention_bifurcation` — instant vs deep reasoning routes |
| Extended thinking + tools | `nt_core::seal_pipeline` — alternating reasoning and action phases |
| Thought summarization | `nt_meta::meta_cognition::summarize_traces` — compress reasoning chains |
| Constitutional AI | `nt_governance::constitution` — rule-based alignment without pure RLHF |
| Local file memory | `nt_memory::local_file_cache` — session-persistent knowledge extraction |
| ASL safety levels | `nt_shield::safety_level` — tiered deployment safeguards |

---

## 4. GPT-4o (Proprietary, ~200B est.)

### Architecture
| Property | Value |
|----------|-------|
| Type | Dense Transformer (end-to-end multimodal) |
| Params | ~200B estimated (60B-400B range) |
| Active Params | ~50B-100B estimated |
| Context | 128K |
| Modalities | Text, Audio, Image, Video (input); Text, Audio, Image (output) |
| Audio Latency | 232ms median, 320ms average |
| Tokenization | Joint: BPE (text) + ViT patchifier (image) + Neural audio codec |

### Key Innovations
- **End-to-End Multimodal**: Single neural network trained jointly across text, vision, audio — not staged CLIP+Whisper+TTS pipeline.
- **Joint Tokenization**: Unified token stream where text (BPE), image (patch tokens), and audio (neural codec tokens ~50-75 Hz) are processed by the same transformer stack via self-attention.
- **Audio Token Emission**: Autoregressively generates audio tokens directly — no separate TTS module. Enables 232ms median voice latency.
- **Dense Architecture**: All layers activated per token (no MoE), optimized for low-latency streaming.
- **Distillation from GPT-4**: Likely a smaller, distilled model from the 1.8T GPT-4 MoE.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| End-to-end multimodal | `nt_io::unified_modality_stream` — joint text+image+audio processing |
| Joint tokenization | `nt_io::unified_tokenizer` — shared embedding space |
| Dense fast inference | `nt_io::streaming_mode` — low-latency real-time processing |
| Audio token emission | `nt_io::audio_codec` — neural audio encoding/decoding |
| Cross-modal self-attention | `nt_core::cross_modal_attention` — modality-agnostic attention |

---

## 5. Gemini 2.5 Flash (MoE)

### Architecture
| Property | Value |
|----------|-------|
| Type | Sparse MoE Transformer |
| Context | 1M tokens |
| Output | 64K (text), 32K (audio) |
| Modalities | Text, Image, Audio, Video |
| Training | Google TPU v5e/v6 |
| Distillation | k-sparse distribution approximation |

### Key Innovations
- **Hybrid Reasoning with Thinking Budget**: Developer-controllable `thinking_budget` parameter (0-24576 tokens). Model auto-decides how much to think based on prompt complexity, but caps at budget. Budget=0 = non-thinking fast mode.
- **MoE with Native Multimodal**: Sparse routing decouples capacity from serving cost. Joint latent space for vision+language (not CLIP-style staged).
- **Distillation from Pro**: Smaller models use k-sparse distribution approximation to store teacher's next-token distribution — trades storage for quality.
- **Native Grounding**: Built-in Google Search grounding — model can query search engine during generation.
- **Full Pareto Coverage**: Flash-Lite → Flash → Pro spans latency/cost/capability frontier within one family.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| Thinking budget | `nt_core::gwt::reasoning_budget` — configurable compute allocation per task |
| MoE capacity decoupling | `nt_core::capability_tree::sparse_activation` — only activate needed expertise |
| k-sparse distillation | `nt_mind::distillation::sparse_teacher` — efficient knowledge transfer |
| Google grounding | `nt_world::search_grounding` — live search during reasoning |
| Pareto family | `nt_io::model_tier_routing` — cheap/standard/premium routing per task |

---

## 6. Llama 4 Scout (17B×16E, 109B total)

### Architecture
| Property | Value |
|----------|-------|
| Type | Alternating Dense + MoE |
| Active Params | 17B |
| Total Params | 109B |
| Experts | 16 routed + 1 shared |
| Routing | Each token → shared expert + 1 routed expert |
| Context | 10M tokens (!) |
| Pretrain Context | 256K |
| Modalities | Text, Image (native early fusion) |
| Pretrain Tokens | ~40T |
| License | Llama Community License |

### Key Innovations
- **iRoPE Architecture**: Interleaved attention layers without positional embeddings + RoPE layers with rotary embeddings. "i" = "interleaved" (long-term goal: infinite context). Inference-time temperature scaling of attention enhances length generalization.
- **10M Token Context**: Industry-leading context length. Pretrained at 256K, generalizes to 10M via iRoPE.
- **Alternating Dense/MoE Layers**: Dense layers between MoE layers improve inference efficiency.
- **Shared + 1-Routed Expert**: Simpler routing than DeepSeek's top-k. Each token always hits shared expert + exactly 1 routed expert.
- **Early Fusion Multimodality**: Image tokens fused with text from start of training, not retrofitted.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| iRoPE (interleaved no-pos + RoPE) | `nt_memory::positional_encoding::hybrid` — position-aware + position-free layers |
| 10M context | `nt_memory::ultra_long_context` — extended window via hybrid position encoding |
| Alternating dense/MoE | `nt_core::layer_hybridization` — dense cognition + sparse expertise layers |
| Shared expert always-on | `nt_core::e8::universal_reasoning` — baseline reasoning always active |
| Early fusion multimodal | `nt_io::native_multimodal` — joint pretraining, not retrofitted |

---

## 7. Mistral Large 3 (675B-A41B)

### Architecture
| Property | Value |
|----------|-------|
| Type | Granular MoE + Vision Encoder |
| Total Params | 675B (673B LM + 2.5B Vision) |
| Active Params | 41B (39B LM + 2B Vision) |
| Experts | Granular MoE (many small experts) |
| Context | 256K |
| Modalities | Text, Image (native multimodal) |
| Languages | 40+ |
| License | Apache 2.0 |

### Key Innovations
- **Granular MoE**: Fine-grained expert segmentation — many small experts rather than few large ones. Each expert's FFN partitioned into smaller functional units.
- **Open-Weight Frontier**: Apache 2.0 license at 675B scale — largest permissively licensed model from a major lab.
- **Native 2.5B Vision Encoder**: Not bolted on — fused during pretraining.
- **NVFP4 Deployment**: Runs on single 8×H100 or 8×A100 node with vLLM quantization.
- **Speculative Decoding**: Partnered with NVIDIA for Blackwell-optimized speculative decoding.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| Granular MoE | `nt_core::capability_tree::fine_grained_experts` — many small skill nodes |
| Open-weight frontier | `nt_memory::open_knowledge` — downloadable, self-hostable expertise |
| NVFP4 deployment | `nt_shield::quantized_deployment` — edge-friendly model serving |
| Speculative decoding | `nt_core::speculative_reasoning` — draft-verify generation |
| Multi-language | `nt_io::multilingual_stream` — 40+ language support |

---

## 8. Phi-4 Reasoning (14B)

### Architecture
| Property | Value |
|----------|-------|
| Type | Dense decoder-only Transformer |
| Params | 14B |
| Context | 32K (doubled RoPE base frequency from 16K) |
| Modalities | Text |
| License | MIT |
| Training | SFT (1.4M prompts, o3-mini traces) + GRPO RL |

### Key Innovations
- **Reasoning Token Injection**: Repurposed 2 placeholder tokens as `<think>` / `</think>` markers to delineate reasoning blocks. Model generates structured reasoning chains.
- **Data-Centric Small Model**: 14B params outperforms 70B+ distilled models (DeepSeek-R1-Distill-Llama-70B) via careful data curation at the "teachable" difficulty boundary.
- **Teacher Distillation from o3-mini**: Used o3-mini medium-effort as teacher — more token-efficient than DeepSeek-R1 as teacher, similar quality.
- **GRPO Reinforcement Learning**: Short RL phase on 6K math problems with rule-based reward (no neural reward model). Boosts accuracy ~1.5x with longer traces.
- **Reasoning is Transferable**: SFT on STEM/coding improves general-purpose benchmarks (IFEval, FlenQA) without direct training on those tasks.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| Reasoning tokens | `nt_core::reasoning_markers` — explicit think/don't-think boundaries |
| Data curation > scale | `nt_mind::distillation::quality_over_quantity` — small model, curated data |
| Teacher distillation | `nt_mind::distillation::teacher_chain` — o3-mini → Phi-4 reasoning traces |
| GRPO RL | `nt_mind::reinforcement::group_relative` — outcome-based RL without reward model |
| Transferable reasoning | `nt_core::meta_skill::reasoning_transfer` — STEM training improves general tasks |

---

## 9. Yi-Lightning (MoE, 01.AI)

### Architecture
| Property | Value |
|----------|-------|
| Type | Enhanced MoE Transformer |
| Params | Undisclosed (estimated ~300B+) |
| Context | 128K+ |
| Modalities | Text |
| Architecture | Fine-grained expert segmentation + hybrid attention |

### Key Innovations
- **Fine-Grained Expert Segmentation**: Partitions each expert's FFN into smaller units — reduces intermediate hidden dim, increases experts activated per token. Balanced approach (not maximum segmentation) to preserve training throughput.
- **3-Level Load Balancing**: (1) Switch-Transformer per-expert loss, (2) EP-group level balancing, (3) Partitioned EP load balancing (PEP) — splits experts within groups to balance All-to-All communication.
- **Hybrid Attention (3:1)**: 3 sliding window attention layers + 1 full attention layer — captures local patterns + global dependencies.
- **Cross-Layer KV Cache Reuse**: Shares KV cache between consecutive full attention layers — 50% memory reduction for full attention components. Total: 82.8% memory reduction.
- **FP8 Hardware-Aware Design**: Architecture aligned to GPU hardware characteristics. Custom MoE operator: 1,200 TFLOPS/card at FP8 on Hopper.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| Fine-grained experts | `nt_core::capability_tree::micro_experts` — partitioned skill units |
| 3-level load balancing | `nt_core::gwt::multi_tier_routing` — per-expert, group, partition balancing |
| Hybrid attention 3:1 | `nt_core::attention::sliding_window_hybrid` — local + global attention |
| Cross-layer KV reuse | `nt_memory::kv_cache_sharing` — shared state across consecutive layers |
| FP8 HW-aware | `nt_physical::hardware_aware_ops` — quantized operators aligned to GPU specs |

---

## 10. Grok 3 (600B MoE, xAI)

### Architecture
| Property | Value |
|----------|-------|
| Type | Dense/MoE Hybrid |
| Total Params | 600B (some reports say 2.7T) |
| Active Params | 120B (600B variant) |
| Layers | 96 |
| Hidden Dim | 12,288 |
| Attention Heads | 96 |
| KV Heads | 16 |
| Head Dim | 128 |
| Experts | 16, active 2 |
| Context | 1M tokens |
| Vocab | 131,072 |
| Training | 200K H100 GPUs (Colossus), 10× compute of predecessor |

### Key Innovations
- **RL-Scale Reasoning**: Reinforcement learning at unprecedented scale — model learns to think for seconds to minutes, backtrack, explore alternatives, verify solutions.
- **DeepSearch**: Built-in deep research capability (not just search grounding — multi-step research agent).
- **Think Mode**: Exposed chain-of-thought reasoning with error correction and alternative exploration.
- **Massive Training Compute**: 200K H100 cluster, ~2.7T parameters trained on ~12.8T tokens.
- **Low Expert Activation**: Only 2 of 16 experts active per token = 12.5% activation ratio.

### NeoTrix Mapping
| Innovation | NeoTrix Equivalent |
|-----------|-------------------|
| RL-scale reasoning | `nt_core::reinforcement::massive_rl` — scale RL for self-correction |
| DeepSearch | `nt_world::deep_search` — multi-step research agent |
| Think mode | `nt_core::reasoning::explicit_thinking` — visible reasoning traces |
| Low activation ratio | `nt_core::gwt::extreme_sparsity` — 12.5% token→expert activation |
| 1M context | `nt_memory::ultra_long_context` — million-token processing |

---

## Cross-Model Synthesis: 8 Universal Architecture Patterns

### P1: MoE as Universal Sparsity
Every model (except GPT-4o and Phi-4) uses MoE. The trend is toward **ultra-sparse** activation: Qwen 3.5 (4.3%), DeepSeek V4.1 (1.5%-3%), Grok 3 (12.5%), Llama 4 Scout (15.6%). Active params per token are shrinking as total params grow.

**NeoTrix**: `nt_core::capability_tree` should implement progressive sparsity — activate only what's needed per task, not all capabilities.

### P2: Hybrid Attention is Standard
GDN (Qwen), sliding window + full (Yi-Lightning, Llama 4 iRoPE), CSA2 (DeepSeek), hybrid reasoning (Claude, Gemini). The pattern: **most layers use cheap attention, few layers use expensive attention**.

**NeoTrix**: `nt_core::attention_manager` — 3:1 cheap:expensive attention ratio as default.

### P3: KV Cache Compression Arms Race
DeepSeek (890 bytes/token FP4), Yi-Lightning (82.8% reduction via cross-layer reuse), Qwen (GDN constant memory), Grok (implied). KV cache is the bottleneck, and every model attacks it differently.

**NeoTrix**: `nt_memory::kv_cache_optimizer` — multi-strategy compression (FP4, cross-layer sharing, linear attention fallback).

### P4: Reasoning Tokens as Standard Interface
Phi-4 (`<think>`/`</think>`), Claude (extended thinking), Grok (Think mode), Gemini (thinking budget). The industry is converging on **explicit reasoning boundaries** — models think, then answer.

**NeoTrix**: `nt_core::reasoning_markers` — explicit thinking/not-thinking token boundaries.

### P5: Asymmetric Compute (Cheap Input, Expensive Output)
DeepSeek V4.1 (8B input, 16B output), Qwen 3.5 (GDN for bulk, GQA for anchor), Gemini (thinking budget control). Input processing is cheap; output generation is expensive.

**NeoTrix**: `nt_core::e8::asymmetric_activation` — different compute budgets for perception vs generation.

### P6: Early Fusion Multimodal
GPT-4o (joint token stream), Qwen 3.5 (early fusion on trillions of tokens), Llama 4 (early fusion), Mistral (native vision encoder). The CLIP+LLM staged approach is obsolete.

**NeoTrix**: `nt_io::native_multimodal` — joint pretraining across modalities from day one.

### P7: Speculative Decoding Everywhere
DeepSeek (DSpark), Mistral (NVIDIA partnership), Grok (implied). Semi-autoregressive draft + verification is the new default for fast generation.

**NeoTrix**: `nt_core::speculative_reasoning` — draft-verify pattern in SEAL pipeline.

### P8: Distillation > Scale
Phi-4 (14B beats 70B via data curation), Gemini (k-sparse distillation from Pro), Llama 4 (Behemoth teacher). Quality of training data matters more than raw parameter count.

**NeoTrix**: `nt_mind::distillation::quality_over_quantity` — prioritize data curation over model scale.

---

## NeoTrix Architecture Implications

### Immediate Actions
1. **Implement hybrid attention** in `nt_core::attention_manager` — 3:1 cheap:expensive ratio
2. **Add reasoning markers** to `nt_core` — explicit `<think>`/`</think>` boundaries
3. **KV cache compression** in `nt_memory` — FP4 + cross-layer sharing + linear attention fallback
4. **Speculative reasoning** in `nt_core::seal_pipeline` — draft-verify pattern

### Medium-Term Research
1. **Causal Encoder-Decoder** for asymmetric activation — cheap input, expensive output
2. **Granular MoE** for `nt_core::capability_tree` — many small experts vs few large ones
3. **Engram conditional memory** — sparsely accessed 196B-parameter memory bank
4. **CSA2 layer sharing** — share KV/indexer state across attention layers

### Strategic Direction
The industry consensus is converging on:
- **Sparse activation** (MoE) as the default architecture
- **Hybrid attention** (cheap + expensive) as the default attention pattern
- **Explicit reasoning boundaries** as the default inference interface
- **KV cache compression** as the critical systems challenge
- **Data curation > parameter scale** as the training philosophy

NeoTrix should align its 6-layer architecture with these universal patterns, particularly in L5 Cognition (reasoning markers, hybrid attention) and L1 Action (KV cache, speculative execution).
