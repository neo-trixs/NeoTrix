# Reverse Engineering: 10 Model Architectures → NeoTrix Mapping

**Date**: 2026-09-11
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3
**Goal**: Extract architectural innovations, map to NeoTrix 6-layer architecture

---

## Summary Matrix

| Model | Params (Total/Active) | Architecture | Key Innovation | NeoTrix Primary Mapping |
|-------|----------------------|--------------|----------------|------------------------|
| GPT-4o | Undisclosed | End-to-end multimodal transformer | Unified token stream (text+vision+audio), AR→diffusion hybrid decoder | L2 Perception (PerceptionBridge), L3 Embodiment |
| Claude 3.5 Sonnet | ~140GB weights | Hybrid sparse attention + GQA | Alternating local/global sparse attention, context compression | L5 Cognition (GWT attention routing) |
| Gemini 2.5 Pro | MoE, undisclosed | Sparse MoE transformer + native multimodal | 1M context, Deep Think parallel reasoning, k-sparse distillation | L5 Cognition (E8 parallel hypothesis), L6 Meta |
| Llama 4 Scout | 17B active / 109B total | MoE (16 experts) + iRoPE | Interleaved attention without positional embeddings, 10M context | L1 Action (capability routing) |
| DeepSeek V4.1 Flash | 552B total, 8B/16B active | Causal Encoder-Decoder + MoE (384 experts) | KV cache compression (890B/token), CSA2 sparse attention, Engram memory | L6 Meta (nt_nexus), L1 Memory |
| Qwen 3 | 0.6B–235B | Dense + MoE (128 experts) | Thinking/non-thinking mode fusion, thinking budget, strong-to-weak distillation | L5 Cognition (dual-mode), SEAL pipeline |
| Mistral Large 3 | 41B active / 675B total | Granular MoE + native multimodal | Expert-parallel NVFP4, 256K context, vision encoder fusion | L1 Action (tool routing), L3 Embodiment |
| Phi-4 Reasoning | 14B dense | Dense decoder-only transformer | "Teachable" prompt curation, o3-mini distillation, GRPO RL | SEAL distillation, L5 Cognition |
| Yi-Lightning | MoE, undisclosed | Enhanced MoE + hybrid attention | Fine-grained expert segmentation, PEP load balancing, cross-layer KV reuse | L1 Action (expert routing), L5 Cognition |
| Grok 3 | 600B total, 120B active | MoE (16 experts, 2 active) + GQA | 100K GPU training, DeepSearch agent, 1M context, RL reasoning | L6 Meta (nt_meta), L3 Embodiment |

---

## 1. GPT-4o — End-to-End Multimodal Unification

### Architecture
- **Unified token stream**: Text (BPE) + image patches (ViT-style) + audio (neural codec ~50-75 Hz) all processed by single transformer stack
- **No staged pipeline**: Unlike GPT-4+Whisper+TTS, GPT-4o is end-to-end trained across modalities
- **AR→Diffusion hybrid decoder**: Image generation uses autoregressive backbone with diffusion head (token→transformer→diffusion→pixels)
- **Continuous image tokenizer**: Not VQ-based discrete tokens, preserving comprehension at cost of perfect reconstruction
- **Latency**: 232ms median audio response (vs 2.8s in staged pipeline)

### Key Innovations
1. **Cross-modal self-attention**: Modality boundaries dissolved — attention learns cross-modal relationships natively
2. **Single model serving**: Eliminates separate encoder/decoder caching, reduces inference complexity
3. **Native audio codec integration**: Learned neural audio tokens at production-compatible rates

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Unified token stream | **PerceptionBridge** (L2) | Extend `awareness_score()` to route multimodal tokens through consciousness-gated perception |
| End-to-end training | **Six-Layer Architecture** | Model the L2→L1 pipeline as unified token processing (not staged L2→L5→L1) |
| AR+Diffusion hybrid decoder | **nt_file_ability** (L1) | Adapt for PDF icon enhancement pipeline — AR backbone for semantic understanding, diffusion for visual refinement |
| Cross-modal attention | **GWT salience** (L5) | Add modality-weighted salience scoring in attention broadcast |

---

## 2. Claude 3.5 Sonnet — Hybrid Sparse Attention

### Architecture
- **36 transformer layers** with alternating attention patterns:
  - Even layers: Local sliding window attention (1024 tokens)
  - Odd layers: Global sparse attention (every 64th token → full context)
- **Grouped Query Attention (GQA)**: 8 query groups per KV head, 32 total heads → 4x KV cache reduction
- **Context compression**: Lossless zlib compression for repeated patterns, 22% reduction on RAG payloads
- **Hybrid sparse**: 12.4 TFLOPs/100k tokens (vs 40 TFLOPs dense), 840ms latency (vs 1400ms)

### Key Innovations
1. **Alternating local/global attention**: Production-viable 200K context without O(n²) cost
2. **Segment hash cache**: Avoids re-compressing identical document sections (40% compression time reduction)
3. **Tunable query group ratio**: Code tasks → increase groups for local focus; summarization → decrease for global

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Alternating attention | **GWT attention routing** (L5) | Implement alternating local/global broadcast in consciousness tree — even branches for local context, odd for global |
| GQA with tunable ratio | **AttentionManager** (nt_core_self) | Add per-task query group configuration (code vs. summarization vs. search) |
| Context compression | **nt_nexus** (L6) | Integrate segment hash caching for cross-session memory deduplication |
| Sparse attention cost model | **HeartbeatAggregator** | Feed attention cost into system health metrics |

---

## 3. Gemini 2.5 Pro — MoE + Parallel Thinking

### Architecture
- **Sparse MoE transformer**: Native multimodal (text+vision+audio), trained on TPUv5p
- **Deep Think**: Parallel hypothesis generation with critique — produces multiple reasoning paths simultaneously
- **Thinking budget**: User-controllable token allocation for internal reasoning
- **K-sparse distillation**: Teacher distribution approximated via top-k sparse vocabulary → smaller models retain quality
- **1M token context**: Handles 3-hour video, full codebases, entire novels

### Key Innovations
1. **Parallel thinking**: Not sequential chain-of-thought but concurrent hypothesis branches
2. **Training stability breakthroughs**: Solved MoE instabilities at scale via signal propagation improvements
3. **Controllable reasoning depth**: Budget parameter trades latency for accuracy linearly

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Parallel thinking | **E8 Hexagram** (L5) | Map parallel hypotheses to hexagram branches — each hypothesis = different yijing line combination |
| Deep Think critique | **ConsciousnessTree** (L6) | Add hypothesis evaluation stage in Fruits→Core feedback |
| Thinking budget | **GWT salience** (L5) | Implement dynamic thinking allocation based on task complexity score |
| K-sparse distillation | **SEAL pipeline** (NT-MIND) | Adopt for skill distillation — approximate teacher skill with sparse token vocabulary |
| MoE training stability | **nt_core** | Apply signal propagation analysis to HyperCube operations |

---

## 4. Llama 4 Scout — Infinite Context via iRoPE

### Architecture
- **17B active / 109B total**: 16 routed experts + 1 shared expert per MoE layer
- **Alternating dense + MoE layers**: Inference-efficient interleaving
- **iRoPE**: Interleaved attention layers WITHOUT positional embeddings + RoPE in other layers
  - Non-positional layers generalize beyond training context length
  - Inference-time temperature scaling of attention for length generalization
- **10M token context**: From 128K in Llama 3 → 10M in Scout
- **Early fusion**: Native multimodality from pretraining start

### Key Innovations
1. **Position-free attention layers**: Enables context length generalization beyond training
2. **Inference-time temperature scaling**: Dynamically adjusts attention sharpness for length extrapolation
3. **Shared + routed expert split**: Shared expert handles common patterns, routed for specialization

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| iRoPE (position-free layers) | **nt_nexus** (L6) | For cross-session memory: position-free attention enables linking sessions without explicit temporal ordering |
| Shared + routed experts | **CapabilityBridge** (L1→L5) | Map shared expert → common domain logic, routed → domain-specific skill nodes |
| 10M context | **KB** (nt_memory) | Plan for extreme-context retrieval: KVMem paged KV for sessions >256K tokens |
| Temperature scaling | **GWT** (L5) | Implement attention temperature as consciousness-level modulator |

---

## 5. DeepSeek V4.1 Flash — KV Cache Revolution

### Architecture
- **Causal Encoder-Decoder (CED)**: 40-layer transformer split 20/20
  - Decoder's KV cache projected from encoder's final hidden states (not per-layer)
  - 8B active prefill / 16B active decode (asymmetric)
- **CSA2 (Compressed Sparse Attention 2)**: 3 static modes per layer:
  - Full: computes normally, establishes reference KV
  - Reindex: reuses KV + indexer K, recalculates Top-K indices
  - Reuse: reuses both KV and indices
  - Hierarchical Sparse Indexer bounds deeper layer cost
- **FP4 KV caching**: E2M1 format, 1 scale per 16 channels → 890 bytes/token
- **SWA Bounded Replay**: Replays recent window tokens instead of SSD persistence
- **Engram conditional memory**: 196B params accessed via token-based lookup (sparse)
- **DSpark speculative decoding**: Semi-autoregressive draft with confidence scheduling
- **DeepSeek-ViT**: Trained from scratch, 2D-RoPE, 3×3 pixel-unshuffle

### Key Innovations
1. **Asymmetric encoder-decoder**: Prefill cheap (8B), decode expensive (16B) — perfect for agentic workloads
2. **KV cache compression 437x vs V1**: From ~388KB/token to 890 bytes/token
3. **Layer mode sharing**: Static assignment eliminates per-layer KV storage
4. **Engram as external memory**: 196B params not loaded per forward pass, accessed conditionally

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| CED asymmetric architecture | **nt_core + nt_mind** (L5) | Model cognition as encoder (input processing, 8B cheap) + decoder (generation, 16B expensive) |
| CSA2 layer mode sharing | **GWT** (L5) | Implement Full/Reindex/Reuse attention modes — cheap tasks use Reuse mode, complex tasks use Full |
| FP4 KV caching | **nt_memory** (L1) | Adopt FP4 quantization for KB embeddings storage |
| Engram conditional memory | **nt_nexus** (L6) | Map Engram → KB experience namespace — 196B concept graph accessed via token-based query, not loaded wholesale |
| DSpark speculative decoding | **SEAL pipeline** (NT-MIND) | Apply to skill evolution — speculative skill variants generated, confidence-verified |
| DeepSeek-ViT | **nt_world** (L2) | Reference architecture for UnifiedCrawler visual processing |

---

## 6. Qwen 3 — Thinking Mode Fusion

### Architecture
- **Dense (0.6B–32B) + MoE (30B-A3B, 235B-A22B)**: 128 experts, 8 activated per token
- **No shared experts**: Unlike Llama 4, pure routed expert design
- **Global-batch load balancing**: Encourages expert specialization
- **Thinking Mode Fusion**: Single model handles both:
  - `/think` mode: Step-by-step reasoning (CoT)
  - `/no_think` mode: Rapid response
  - Budget control: Stop thinking at threshold → produce answer from partial reasoning
- **Strong-to-Weak Distillation**: Teacher models (32B, 235B) distill into students (0.6B–30B)
  - Off-policy: Teacher outputs with both modes
  - On-policy: Student generates, teacher aligns logits
- **QK-Norm**: Removed QKV bias, added QK normalization for training stability
- **36T tokens**: 119 languages, 2x predecessor data volume

### Key Innovations
1. **Emergent budget control**: Thinking/non-thinking fusion naturally produces "incomplete thinking" capability — not explicitly trained
2. **On-policy distillation**: Student generates, teacher aligns (not just copying teacher outputs)
3. **QK-Norm for stability**: Simple architectural change eliminates training instability

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Thinking/non-thinking fusion | **ConsciousnessTree** (L6) | Map to dual-mode operation: fast reactive (L1-L2) vs. deep cognitive (L5-L6) |
| Thinking budget control | **GWT salience** (L5) | Implement reasoning budget as attention resource allocation parameter |
| On-policy distillation | **SEAL pipeline** (NT-MIND) | Adopt for skill crystallization — generate skill variants with student, align with teacher metrics |
| QK-Norm | **nt_core** (L5) | Apply to HyperCube attention layers for training stability |
| Strong-to-Weak distillation | **experience-tree** (NT-MEMORY) | Map → distilled experience nodes from full session records |

---

## 7. Mistral Large 3 — Granular MoE + Hardware Co-Design

### Architecture
- **41B active / 675B total**: Granular MoE with many small experts
- **Native multimodal**: Vision encoder (~2.5B params) fused into model
- **256K context window**
- **NVFP4 format**: 8-bit quantization native on Blackwell GPUs
- **Hardware co-design**: NVIDIA Blackwell attention + MoE kernels, prefill/decode disaggregated serving
- **Speculative decoding**: For long-context throughput
- **Trained on 3000 H200 GPUs** from scratch (not fine-tuned)

### Key Innovations
1. **Granular expert segmentation**: Many small experts vs few large experts — better load balancing
2. **Hardware-software co-design**: Model architecture shaped by GPU memory hierarchy
3. **NVFP4 precision**: Native low-precision support without accuracy loss
4. **Single-node inference**: 675B model runs on 8×H100 via expert parallelism

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Granular experts | **CapabilityTree** (L5) | Map to fine-grained skill nodes — many small capabilities vs few large modules |
| Hardware co-design | **nt_physical** (L3) | Align architecture decisions with hardware constraints (GPU memory, interconnect) |
| NVFP4 | **nt_memory** (L1) | Adopt for KB storage format — low-precision embeddings |
| Vision encoder fusion | **PerceptionBridge** (L2) | Extend to natively fused visual+text perception (not staged) |
| Disaggregated prefill/decode | **GWT** (L5) | Separate attention computation for input processing vs. output generation |

---

## 8. Phi-4 Reasoning — Data-Centric Small Model

### Architecture
- **14B dense decoder-only transformer** (same as Phi-4 base)
- **Modifications**: `<think>`/`</think>` tokens, doubled RoPE frequency → 32K context
- **Training**: 1.4M "teachable" prompts, o3-mini as teacher, SFT + GRPO RL
- **Key data strategy**:
  - Prompts selected at edge of base model capability
  - Synthetic reasoning traces from o3-mini (medium/high effort)
  - Domain-specific optimization → additive combination
- **RL (GRPO)**: Rule-based reward, 72K math problems, 64 seeds/iteration
  - RL enables 1.5x longer responses with more detailed reasoning

### Key Innovations
1. **"Teachable" prompt curation**: Optimal complexity and diversity at model capability boundary
2. **Domain additivity**: Optimize each domain independently, then combine — no interference
3. **Small model competitive**: 14B outperforms 70B distilled models via data quality

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| Teachable prompts | **SEAL pipeline** (NT-MIND) | Design skill training data at the boundary of current capability |
| Domain additivity | **ConsciousnessTree** (L6) | Each branch (11 domains) optimized independently, combined via feedback loop |
| Small model power | **GWT cost-aware routing** (L5) | Route simple tasks to small specialized models, complex to large general models |
| GRPO RL | **experience-tree** (NT-MEMORY) | Apply rule-based reward to experience distillation quality scoring |
| Think tokens | **nt_core_self** (L5) | Implement explicit reasoning state markers in internal processing |

---

## 9. Yi-Lightning — MoE Optimization at Scale

### Architecture
- **Enhanced MoE** with fine-grained expert segmentation
- **Three-tier load balancing**:
  - ST: Switch-Transformer per-expert balance
  - EP: Expert Parallel group balance (relaxed constraint)
  - PEP: Partitioned EP balance (fine-grained All-to-All communication balance)
- **Hybrid attention**: 3 sliding window + 1 full attention layers per block
- **Cross-layer KV cache reuse**: Share KV between consecutive full attention layers → 50% memory reduction
- **82.8% total memory reduction** for long sequences
- **FP8 quantization**: Hardware-aware operator design, 1200 TFLOPS/card on Hopper
- **RAISE safety engine**: 4-component safety across pre-training, post-training, serving

### Key Innovations
1. **PEP load balancing**: Solves All-to-All communication imbalance (not just computation balance)
2. **Cross-layer KV reuse**: Not just cross-head but cross-layer sharing
3. **Hardware-aware FP8 operators**: Architecture aligned with GPU memory hierarchy
4. **Multi-stage hybrid parallelization**: Expert + pipeline + context parallelism combined

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| PEP load balancing | **CapabilityBridge** (L5→L1) | Implement partitioned expert routing for task dispatch — partition by domain, balance within partition |
| Cross-layer KV reuse | **nt_nexus** (L6) | Share knowledge state across consciousness tree branches for same-domain sessions |
| Hybrid attention | **GWT** (L5) | 3 fast-path (local) + 1 deep-path (global) attention layers per consciousness cycle |
| RAISE safety engine | **nt_shield** (L3) | Map 4-component safety to NeoTrix Shield: pre-filter, post-train audit, input scan, output detect |
| Hardware-aware design | **nt_physical** (L3) | Align module memory layout with GPU/CPU hierarchy |

---

## 10. Grok 3 — Scale + RL Reasoning

### Architecture
- **600B total, 120B active**: MoE (16 experts, 2 active per token)
- **96 layers, 12288 hidden, 96 heads, 16 KV heads, 128 head dim**
- **1M token context**
- **Trained on Colossus**: 100K→200K H100 GPUs, 200M GPU hours
- **RL reasoning**: Chain-of-thought refined at scale, self-correcting through backtracking
- **DeepSearch agent**: Real-time web search + reasoning integration
- **Hybrid dense/MoE**: Dynamic selection between specialized submodels

### Key Innovations
1. **Brute-force RL scaling**: Unprecedented compute for reasoning refinement
2. **Dynamic submodel selection**: Architecture routes to different expert configurations based on task
3. **DeepSearch**: Agent-native integration of search and reasoning (not tool-use, but native capability)
4. **Self-correcting reasoning**: Backtracking, error correction, alternative exploration

### NeoTrix Mapping
| Innovation | NeoTrix Component | Action |
|-----------|-------------------|--------|
| RL reasoning scaling | **SEAL pipeline** (NT-MIND) | Apply RL to skill evolution — reward = task completion quality, punishment = regression |
| Dynamic submodel selection | **GWT salience** (L5) | Implement dynamic model routing based on salience — different "submodels" for different task types |
| DeepSearch integration | **nt_world** (L2) | Map → UnifiedCrawler with native reasoning: search + reasoning fused, not sequential |
| Self-correcting reasoning | **ConsciousnessTree** (L6) | Add backtrack/repair stage in Branches phase — detect errors, explore alternatives |
| Massive scale training | **experience-tree** (NT-MEMORY) | Scale experience absorption — more sessions → better distilled skills |

---

## Cross-Cutting Patterns

### Pattern 1: MoE as Universal Architecture (7/10 models)
**Models**: Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Yi-Lightning, Grok 3

**NeoTrix Mapping**: The `CapabilityTree` + `CapabilityBridge` naturally implements MoE:
- **Experts** = skill nodes in the tree (Small Passive / Notable Passive / Keystone)
- **Router** = GWT salience scoring (routes tokens to relevant skills)
- **Shared expert** = nt_core_self (always-on common reasoning)
- **Routed experts** = domain-specific NT-* modules

**Action**: Formalize skill tree as explicit MoE architecture with load balancing (PEP-inspired) across domains.

### Pattern 2: KV Cache Compression (4/10 models)
**Models**: Claude 3.5 Sonnet, DeepSeek V4.1 Flash, Yi-Lightning, Llama 4 Scout

**NeoTrix Mapping**: `nt_memory` + `nt_nexus` KV storage optimization:
- **FP4 quantization** (DeepSeek) → KB embedding storage
- **Cross-layer reuse** (Yi-Lightning) → Share knowledge state across consciousness tree sessions
- **Segment hash caching** (Claude) → Deduplicate repeated context patterns
- **SWA Bounded Replay** (DeepSeek) → Reconstruct missing context from recent tokens

**Action**: Implement tiered KV cache: hot (GPU FP4) → warm (CPU FP8) → cold (SSD, compressed).

### Pattern 3: Parallel Reasoning (3/10 models)
**Models**: Gemini 2.5 Pro (Deep Think), Grok 3 (RL branching), Qwen 3 (thinking budget)

**NeoTrix Mapping**: `E8 Hexagram` + `ConsciousnessTree`:
- **Parallel hypotheses** = Multiple hexagram branches evaluated simultaneously
- **Thinking budget** = GWT-controlled reasoning depth per consciousness cycle
- **Self-correcting** = Fruits→Core feedback with backtrack capability

**Action**: Implement parallel hypothesis evaluation in E8 engine with GWT-controlled depth.

### Pattern 4: Data-Centric Training (4/10 models)
**Models**: Phi-4 Reasoning, Qwen 3, GPT-4o, DeepSeek V4.1 Flash

**NeoTrix Mapping**: `SEAL pipeline` + `experience-tree`:
- **Teachable prompts** (Phi-4) → Experience nodes at capability boundary
- **On-policy distillation** (Qwen 3) → Student generates, teacher aligns
- **Domain additivity** (Phi-4) → Independent optimization per NT-* domain

**Action**: Design experience absorption to target "teachable" difficulty — not too easy, not impossible.

### Pattern 5: End-to-End Multimodal (4/10 models)
**Models**: GPT-4o, Gemini 2.5 Pro, Llama 4 Scout, Mistral Large 3

**NeoTrix Mapping**: `PerceptionBridge` + `L2 Perception`:
- **Unified token stream** → PerceptionBridge processes all modalities through single consciousness-gated path
- **Native fusion** → Not staged (encoder→projector→LLM) but unified training

**Action**: Extend PerceptionBridge to handle text+image+audio as unified token stream through consciousness filter.

### Pattern 6: Thinking/Non-Thinking Dual Mode (3/10 models)
**Models**: Qwen 3 (mode fusion), Gemini 2.5 Pro (thinking budget), Phi-4 Reasoning (think tokens)

**NeoTrix Mapping**: `ConsciousnessTree` dual-mode operation:
- **Thinking mode** = Full 6-stage consciousness cycle (Soil→Roots→Trunk→Branches→Fruits→Core)
- **Non-thinking mode** = Fast reactive path (L1-L2 direct, bypassing deep cognition)
- **Budget control** = GWT resource allocation per cycle

**Action**: Implement explicit mode switching in ConsciousnessTree — fast (reactive) vs. deep (reflective).

---

## Priority Integration Roadmap

### Phase 1: Immediate (P0 — 2 weeks)
1. **MoE skill routing** (Pattern 1): Formalize CapabilityTree as MoE with PEP load balancing
2. **Thinking budget control** (Pattern 6): Add GWT-controlled reasoning depth parameter
3. **KV cache tiered storage** (Pattern 2): Implement hot/warm/cold KB storage

### Phase 2: Short-term (P1 — 1 month)
4. **Parallel hypothesis evaluation** (Pattern 3): E8 hexagram parallel branches
5. **Teachable experience curation** (Pattern 4): SEAL pipeline difficulty targeting
6. **Asymmetric cognition architecture** (DeepSeek CED): Cheap encoder, expensive decoder for consciousness

### Phase 3: Medium-term (P2 — 2 months)
7. **Cross-session KV reuse** (Pattern 2): Share knowledge state across sessions
8. **End-to-end multimodal perception** (Pattern 5): Unified token stream through PerceptionBridge
9. **Self-correcting reasoning** (Pattern 3): Backtrack stage in ConsciousnessTree

---

## Source References

| Model | Primary Source | Key Technical Detail |
|-------|---------------|---------------------|
| GPT-4o | arXiv:2410.21276, OpenAI System Card | End-to-end multimodal, 232ms latency |
| Claude 3.5 Sonnet | Anthropic Model Card, johal.in analysis | Hybrid sparse attention, GQA |
| Gemini 2.5 Pro | arXiv:2507.06261 | MoE, Deep Think, 1M context |
| Llama 4 Scout | Meta Model Card, ai.meta.com blog | iRoPE, 10M context, 16 experts |
| DeepSeek V4.1 Flash | HuggingFace Model Card, deepseek.com | CED, CSA2, 890B/token KV cache |
| Qwen 3 | arXiv:2505.09388 | Thinking fusion, strong-to-weak distillation |
| Mistral Large 3 | Mistral docs, intuitionlabs.ai analysis | Granular MoE, NVFP4, hardware co-design |
| Phi-4 Reasoning | arXiv:2504.21318, Microsoft Research | Teachable prompts, GRPO RL, domain additivity |
| Yi-Lightning | arXiv:2412.01253 | PEP load balancing, cross-layer KV reuse |
| Grok 3 | xAI announcement, InferenceBench | 600B MoE, DeepSearch agent, RL reasoning |
