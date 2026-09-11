# Model Architecture Reverse Engineering — 10 Models

> Date: 2026-09-11 | Scope: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 reasoning, Yi-Lightning, Grok 3

---

## 1. GPT-4o (OpenAI)

| Attribute | Detail |
|-----------|--------|
| **Params** | Undisclosed (estimated ~200B) |
| **Architecture** | Autoregressive omni model, single unified transformer |
| **Modality** | Text + Audio + Image + Video (all modalities end-to-end) |
| **Context** | 128K tokens |
| **Latency** | 232ms median audio response |

**Key Innovations:**
1. **End-to-end multimodal training** — single neural network processes text, vision, audio jointly; no separate encoders. Collapsed ASR+LLM+TTS pipeline into one model.
2. **Neural audio codec tokenization** — learned discrete audio tokens at ~50-75 Hz (similar to Encodec/SoundStream), enabling real-time voice.
3. **Diffusion-based image decoding** — AR + diffusion hybrid: autoregressive text generation + diffusion head for image output (empirically validated by GPT-ImgEval).
4. **Joint cross-modal attention** — cross-modal relationships learned through self-attention within unified token stream, not separate cross-modal layers.

**NeoTrix Mapping:**
- **PerceptionBridge** → GPT-4o's unified attention across modalities mirrors GWT's cross-domain broadcasting. Both use single routing mechanism for multi-source signals.
- **EmotionLabel** → GPT-4o's end-to-end audio prosody parallels EmotionLabel's unified emotion representation across modalities.
- **Cost-Aware Routing (A1)** → GPT-4o achieved 50% cost reduction over GPT-4 Turbo via architectural efficiency, validating A1 axiom.

---

## 2. Claude 3.5 Sonnet (Anthropic)

| Attribute | Detail |
|-----------|--------|
| **Params** | Undisclosed |
| **Architecture** | Dense transformer (evolution of Claude 3 family) |
| **Modality** | Multimodal input (text+image) → text output |
| **Context** | 200K tokens |
| **Safety** | ASL-2, Constitutional AI trained |

**Key Innovations:**
1. **Computer use capability** — screenshot → GUI command generation; OSWorld 14.9% (human 72.36%). First model to natively interpret screen state.
2. **Agentic coding at scale** — SWE-bench Verified 49.0% pass@1; agentic loop with search/view/edit across 3-20 files.
3. **HHH alignment with capability preservation** — Constitutional AI + RSP safety framework; refusal calibration preserves underlying capability.
4. **Tool-augmented reasoning** — native function calling + JSON output for agentic workflows; self-correcting loops.

**NeoTrix Mapping:**
- **NT-ACT (行动执行者)** → Claude's agentic coding loop directly maps to NT-ACT's orchestration of tools, code editing, and self-correction.
- **NT-SHIELD (影卫)** → ASL safety levels and Constitutional AI align with NT-SHIELD's audit and safety governance (D1-D12).
- **Disclosure Ladder (P4)** → Claude's tiered model sizing (Haiku/Sonnet/Opus) mirrors the anchor-then-promote pattern.

---

## 3. Gemini 2.5 Pro (Google DeepMind)

| Attribute | Detail |
|-----------|--------|
| **Params** | Undisclosed (sparse MoE) |
| **Architecture** | Sparse MoE transformer, native multimodal |
| **Modality** | Text + Vision + Audio input/output |
| **Context** | 1M tokens (2M upcoming) |
| **Training** | TPUv5p, 8960-chip pods, multi-datacenter |

**Key Innovations:**
1. **Sparse MoE with training stability** — addressed MoE training instabilities (divergence, dead experts) with improved signal propagation and optimization dynamics.
2. **Controllable thinking budget** — user-configurable reasoning depth; performance scales linearly with budget tokens.
3. **K-sparse distillation** — approximate teacher distribution with top-k sparse vectors, reducing storage while preserving quality in smaller Flash models.
4. **3-hour video processing** — native long-video understanding via architectural changes to vision processing; not just frame sampling.
5. **Slice-granularity elasticity** — automatic recovery from TPU failures in tens of seconds vs. 10+ minutes; 97% throughput during recovery.

**NeoTrix Mapping:**
- **GWT Attention Routing** → Gemini's thinking budget is isomorphic to GWT salience modulation — more attention budget = deeper reasoning.
- **Cost-Aware Routing (A1)** → Gemini 2.X family covers full Pareto frontier: Flash-Lite (cheapest) → Flash → Pro (most capable). Exact A1 implementation.
- **Skill as Production Template (A3)** → K-sparse distillation enables large→small model knowledge transfer, analogous to skill crystallization.
- **SEAL Pipeline** → Four-stage training (cold-start → RL → mode fusion → distillation) parallels SEAL's exploration→distillation→absorption cycle.

---

## 4. Llama 4 Scout (Meta)

| Attribute | Detail |
|-----------|--------|
| **Params** | 17B active / 109B total (16 experts) |
| **Architecture** | Alternating dense + MoE layers, early fusion multimodal |
| **Modality** | Text + Image → Text + Code |
| **Context** | 10M tokens (industry-leading) |
| **Training** | ~40T tokens, MetaCLIP vision encoder |

**Key Innovations:**
1. **iRoPE architecture** — interleaved attention layers without positional embeddings + RoPE in remaining layers. Inference-time temperature scaling of attention for length generalization. Goal: "infinite" context.
2. **Early fusion multimodality** — text and vision tokens integrated into unified backbone during pretraining (not post-hoc CLIP-style projection). MetaCLIP encoder adapted jointly with frozen Llama.
3. **Single-H100 deployment** — 17B active params fit on single H100 with int4 quantization. Open-weight, FP8 quantized for quality preservation.
4. **Mid-training recipe** — specialized datasets for long-context extension post-pretraining; bridges pretraining and post-training phases.

**NeoTrix Mapping:**
- **KVMem (A2)** → Llama 4's 10M context via iRoPE directly addresses Context as Scarce Resource. Interleaved attention without PE is a breakthrough for KV cache efficiency.
- **CapabilityBridge** → Early fusion architecture mirrors CapabilityBridge's unified view of evolution (CapabilityTree) and runtime (CapabilityRegistry).
- **Rune Socketing** → Alternating dense/MoE layers implement a natural rune-like configuration: dense layers for core processing, MoE layers for expert routing.

---

## 5. DeepSeek V4.1 Flash (DeepSeek)

| Attribute | Detail |
|-----------|--------|
| **Params** | 284B total / 13B activated |
| **Architecture** | DeepSeekMoE + hybrid CSA/HCA attention + mHC |
| **Modality** | Text |
| **Context** | 1M tokens |
| **Precision** | FP4 routed experts + FP8 mixed |
| **Training** | 32T tokens, Muon optimizer |

**Key Innovations:**
1. **Hybrid CSA + HCA attention** — Compressed Sparse Attention (compress m tokens → 1 entry, then sparse top-k) interleaved with Heavily Compressed Attention (extreme m' >> m compression, dense attention). Achieves 10% FLOPs and 7% KV cache vs V3.2 at 1M context.
2. **Manifold-Constrained Hyper-Connections (mHC)** — constrain residual mapping onto specific manifold to enhance signal propagation stability while preserving expressivity. Superior to naive Hyper-Connections.
3. **Muon optimizer** — faster convergence and training stability over AdamW; adopted for majority of modules.
4. **FP4 routed experts** — ultra-low precision for MoE expert parameters; theoretical 3x efficiency on future hardware.
5. **Multi-Token Prediction (MTP)** — retained from V3, parallel next-token prediction heads.
6. **Domain-expert distillation pipeline** — independent SFT+RL (GRPO) per domain (math, code, agent), then on-policy distillation into unified model.

**NeoTrix Mapping:**
- **KVMem (A2)** → DeepSeek's hybrid attention architecture is the most aggressive KV cache optimization: 7-10% of V3.2's cache at 1M context. Directly maps to KVMem's paged KV virtualization.
- **GWT Attention** → CSA/HCA hybrid = attention budget allocation: cheap tokens get heavy compression (HCA), important tokens get sparse attention (CSA). GWT salience with cost weighting.
- **SEAL Pipeline** → Domain-expert cultivation → unified distillation is isomorphic to SEAL's skill crystallization → absorption.
- **Dark Forest** → FP4 expert pruning enforces compile+test+connect or delete.

---

## 6. Qwen 3 (Alibaba Qwen)

| Attribute | Detail |
|-----------|--------|
| **Params** | 235B total / 22B activated (MoE) + 6 dense variants (0.6B-32B) |
| **Architecture** | Dense + MoE, GQA, SwiGLU, RoPE, RMSNorm, QK-Norm |
| **Modality** | Text |
| **Context** | 32K-128K (dense) / 128K (MoE) |
| **Training** | 4-stage: Long-CoT cold start → Reasoning RL → Thinking mode fusion → Strong-to-weak distillation |

**Key Innovations:**
1. **Unified thinking/non-thinking mode** — single model handles both modes via chat template flags (/think, /no_think). No separate models needed. Thinking budget mechanism for latency-performance tradeoff.
2. **Strong-to-weak distillation** — off-policy distillation (teacher outputs) → on-policy distillation (student generates, aligns logits to teacher via KL divergence). 1/10 GPU hours vs full 4-stage training.
3. **QK-Norm for training stability** — removed QKV-bias, introduced QK-Norm (from Gemma) to stabilize MoE training.
4. **Global-batch load balancing loss** — encourages expert specialization; no shared experts (unlike Qwen2.5-MoE).
5. **119 languages** — expanded from 29 to 119 languages/dialects.

**NeoTrix Mapping:**
- **GWT Attention** → Thinking budget = attention allocation per task. /think flag = route to deep reasoning specialist, /no_think = route to fast response. GWT salience routing.
- **Disclosure Ladder (P4)** → Thinking budget naturally implements the anchor-then-promote pattern: start with minimal budget, promote to deeper reasoning as needed.
- **SEAL Pipeline** → 4-stage training parallels SEAL's Soil→Roots→Trunk→Branches growth cycle.
- **Skill Crystallization** → Strong-to-weak distillation = NT-MIND's skill crystallization from expert to lightweight.

---

## 7. Mistral Large 3 (Mistral AI)

| Attribute | Detail |
|-----------|--------|
| **Params** | 675B total / 41B active (granular MoE) |
| **Architecture** | Granular MoE + 2.5B vision encoder |
| **Modality** | Text + Image → Text |
| **Context** | 256K tokens |
| **Training** | 3000× H200 GPUs, Apache 2.0 license |
| **Deployment** | FP8 on single 8×B200/H200 node; NVFP4 on 8×H100/A100 |

**Key Innovations:**
1. **Granular MoE** — fine-grained expert routing with extreme sparsity ratio (675B/41B ≈ 16:1). More experts than typical MoE, smaller per-expert capacity.
2. **Native vision integration** — 2.5B vision encoder fused into model (not adapter). Enables OCR, document understanding, image QA within unified model.
3. **NVFP4 deployment** — first frontier model optimized for Blackwell NVL72 via NVFP4 quantization + speculative decoding.
4. **Prefill/decode disaggregated serving** — separate hardware for prefill vs decode phases; maximizes throughput on long-context workloads.

**NeoTrix Mapping:**
- **CapabilityBridge** → Granular MoE = fine-grained capability nodes. Each expert is a micro-specialist, mapped to CapabilityRegistry runtime IDs.
- **Rune Socketing 5-slot** → 5 rune colors map to MoE expert types: Crimson(data ingest), Indigo(transform), Obsidian(cache), Golden(error recovery), Alabaster(monitor).
- **Ordered Backend Router (P4)** → Disaggregated prefill/decode = ordered backend with hardware specialization. Single interface, different execution paths.

---

## 8. Phi-4 Reasoning (Microsoft Research)

| Attribute | Detail |
|-----------|--------|
| **Params** | 14B (dense decoder-only transformer) |
| **Architecture** | Phi-4 base + reasoning tokens + extended RoPE |
| **Modality** | Text |
| **Context** | 32K tokens (doubled from 16K base) |
| **Training** | SFT on 1.4M prompts + GRPO RL on 6.4K math problems |
| **Teacher** | o3-mini (medium/high effort) |

**Key Innovations:**
1. **"Teachable" prompt curation** — prompts selected at boundary of base model capability for maximum learning. Optimal complexity + diversity filtering.
2. **Reasoning as transferable meta-skill** — improvements transfer to domains not explicitly trained (code, planning, spatial). Reasoning is domain-agnostic.
3. **Thinking tokens** — repurposed placeholder tokens as `<think>`/`</think>` markers; enables structured reasoning blocks.
4. **GRPO with rule-based reward** — outcome-based RL without neural reward model (avoids reward hacking). 72K math seeds → 64 per iteration.
5. **Response length regulation** — RL produces 1.5x longer responses; accuracy-efficiency tradeoff controlled by reward shaping (repetition/length penalties).
6. **Additive domain property** — domains can be optimized independently then combined, enabling modular training.

**NeoTrix Mapping:**
- **SEAL Pipeline** → Teachable prompts = SEAL's Phase-0 convergence check (select tasks at capability boundary). GRPO RL = SEAL's Fruits→Core feedback loop.
- **ConsciousnessTree** → Reasoning transferability = cross-domain health propagation. Improvements in math transfer to coding, like NT-META learning propagates across modules.
- **E8 Hexagram** → Thinking tokens create structured reasoning states, analogous to E8's 64 hexagram reasoning states.
- **Experience-tree** → Additive domain property = experience branches can be independently distilled then merged.

---

## 9. Yi-Lightning (01.AI)

| Attribute | Detail |
|-----------|--------|
| **Params** | MoE (exact count undisclosed) |
| **Architecture** | Enhanced MoE + hybrid attention + cross-layer KV sharing |
| **Modality** | Text |
| **Context** | Extended (via KV cache optimization) |
| **Training** | FP8 on Hopper, 1200 TFLOPS/card |
| **Ranking** | Chatbot Arena #6 overall, #2-4 in Chinese/Math/Coding |

**Key Innovations:**
1. **Fine-grained expert segmentation** — partition each FFN into smaller units, increase activated experts per token. Balanced segmentation (not max) to preserve training throughput.
2. **3-layer load balancing** — Switch Transformer loss (per-expert) → EP group loss → Partitioned EP loss (PEP). Addresses All-to-All communication imbalance.
3. **Hybrid sliding window + full attention** — 3 sliding window layers + 1 full attention layer per block. Most heads focus on local, few on global. 82.8% memory reduction.
4. **Cross-layer KV cache reuse** — share KV states between consecutive full attention layers. Halves memory for full attention components.
5. **RAISE safety framework** — 4-component safety across pre-training, post-training, and serving phases.

**NeoTrix Mapping:**
- **KVMem (A2)** → Cross-layer KV reuse + hybrid attention = paged KV virtualization. 82.8% memory reduction validates A2 (Context as Scarce Resource).
- **GWT Attention** → 3:1 sliding/full ratio = attention budget allocation. Local heads (cheap) + global heads (expensive) = cost-aware routing.
- **Ordered Backend Router (P4)** → 3-layer load balancing = ordered fallback: try per-expert balance → EP group balance → partitioned balance.
- **NT-SHIELD** → RAISE framework parallels NT-SHIELD's audit dimensions (D1-D50) for safety governance.

---

## 10. Grok 3 (xAI)

| Attribute | Detail |
|-----------|--------|
| **Params** | ~1.2T (estimated, undisclosed) |
| **Architecture** | Transformer + neuro-symbolic hybrid, MoE with 128 experts |
| **Modality** | Text (API), with DeepSearch agentic capability |
| **Context** | 1M tokens (8x previous Grok) |
| **Training** | Colossus supercluster, 200K H100 GPUs, 10x compute of previous SOTA |
| **Ranking** | Chatbot Arena Elo 1402 |

**Key Innovations:**
1. **RL-scaled reasoning** — chain-of-thought refined via massive-scale reinforcement learning. Thinks for seconds to minutes, corrects errors, explores alternatives.
2. **Cross-expert attention gates** — MoE experts share knowledge via learned attention gates (not just routing). Prevents catastrophic interference while enabling specialization.
3. **DeepSearch agent** — agentic research mode: web+X real-time query → synthesize → reason about conflicts → distill report. First production reasoning agent from xAI.
4. **10x compute scaling** — largest training run at time of release; diminishing returns analysis not yet published.
5. **Think/DeepSearch modes** — explicit reasoning mode + agentic web search mode. Dual-mode operation.

**NeoTrix Mapping:**
- **ConsciousnessTree** → Grok's extended reasoning (seconds to minutes) parallels ConsciousnessTree's 6-stage feedback loop (Soil→Core). Both spend time "thinking" before responding.
- **GWT Attention** → DeepSearch = GWT's selective attention applied to real-world web corpus. Salience routing over external knowledge.
- **CapabilityBridge** → Cross-expert attention gates = runtime bridge between specialized capability nodes (CapabilityTree ↔ CapabilityRegistry).
- **E8 Hexagram** → Think mode's structured reasoning trace = E8's 64 hexagram reasoning states, where each step is a deliberate state transition.

---

## Cross-Model Innovation Matrix

| Innovation | Models | NeoTrix Component |
|------------|--------|-------------------|
| **Sparse MoE** | Gemini 2.5, Llama 4, DeepSeek V4, Qwen 3, Mistral 3, Yi-Lightning | Rune Socketing, CapabilityBridge |
| **Hybrid Attention (local+global)** | DeepSeek V4, Yi-Lightning | GWT salience + Cost-Aware Routing |
| **KV Cache Optimization** | DeepSeek V4, Yi-Lightning, Llama 4 | KVMem (A2), kv_cache_optimizer |
| **Thinking/Reasoning Budget** | Gemini 2.5, Qwen 3, DeepSeek V4, Grok 3 | GWT Attention budget, E8 Hexagram |
| **End-to-end Multimodal** | GPT-4o, Llama 4, Mistral 3 | PerceptionBridge, EmotionLabel |
| **Domain Expert Distillation** | DeepSeek V4, Qwen 3, Phi-4 | SEAL Pipeline, Skill Crystallization |
| **Agentic Tool Use** | Claude 3.5, Grok 3, Gemini 2.5 | NT-ACT, Ordered Backend Router |
| **Safety/Governance** | Claude 3.5, Yi-Lightning, GPT-4o | NT-SHIELD, Audit Dimensions |
| **Low-Precision Training** | DeepSeek V4 (FP4), Yi-Lightning (FP8), Mistral 3 (NVFP4) | Dark Forest, Rune Socketing |

---

## Axiom Validation (8-Source Batch from 2026-09-08)

| Axiom | Validation Source | Evidence |
|-------|-------------------|----------|
| **A1: Cost-Aware Routing** | Gemini 2.5 (Pareto frontier), DeepSeek V4 (10% FLOPs) | Models explicitly optimize cost-performance tradeoff |
| **A2: Context as Scarce Resource** | Llama 4 (10M ctx), DeepSeek V4 (7% KV cache), Yi-Lightning (82.8% reduction) | All top models invest heavily in KV cache efficiency |
| **A3: Skill as Production Template** | Qwen 3 (distillation), Phi-4 (teachable prompts), Gemini 2.5 (k-sparse distill) | Knowledge transfer from large→small is universal pattern |

## Cross-Source Pattern Validation

| Pattern | Validation | NeoTrix Implementation |
|---------|-----------|----------------------|
| **P1: Model Routing** | Gemini Flash-Lite→Flash→Pro; DeepSeek Flash→Pro | GWT salience + cost weight |
| **P2: Isolation-per-Task** | Phi-4's domain-specific training; DeepSeek domain experts | Worktree isolation + paged memory |
| **P3: Profile-Driven** | Qwen 3's thinking/non-thinking mode switching | SelfModel extension |
| **P4: Ordered Backend Fallback** | Yi-Lightning's 3-layer load balancing; DeepSeek CSA→HCA | Ordered Backend Router |
| **P5: Skill as Template** | Qwen 3 distillation pipeline; Phi-4 teachable prompts | SKILL-SPEC.md contract |

---

## Key Takeaway

The 2025-2026 LLM landscape converges on three architectural pillars:
1. **Sparse MoE** with fine-grained expert routing (8/128 or 22/235 activation ratios)
2. **Hybrid attention** with local/global head specialization for KV cache efficiency
3. **Thinking budget control** enabling user-configurable reasoning depth

All three pillars directly validate NeoTrix's core axioms (A1-A3) and architectural patterns (GWT, KVMem, SEAL). The most transferable innovation is **Domain Expert Distillation** (DeepSeek V4, Qwen 3, Phi-4) — which maps directly to NT-MIND's skill crystallization and the SEAL pipeline's distillation stage.
