# Model Reverse Engineering #253 — 10-Model Architecture Survey

> **Date**: 2026-09-11  
> **Models**: GPT-4o · Claude 3.5 Sonnet · Gemini 2.5 Pro · Llama 4 Scout · DeepSeek V4 Flash · Qwen 3 · Mistral Large 3 · Phi-4 Reasoning · Yi-Lightning · Grok 3

---

## 1. Model Architecture Overview

| Model | Architecture | Params (Total/Active) | Experts | Context | Multimodal | Key Innovation |
|-------|-------------|----------------------|---------|---------|------------|----------------|
| **GPT-4o** | Dense Transformer (MoE rumored) | ~1.8T est. / undisclosed | undisclosed | 128K | Native text+audio+video | End-to-end omni-modal single model |
| **Claude 3.5 Sonnet** | Dense Transformer | undisclosed | none | 200K | Text+image | Constitutional AI + thinking tokens |
| **Gemini 2.5 Pro** | Sparse MoE Transformer | undisclosed | undisclosed | 1M (2M coming) | Native text+vision+audio | Sparse MoE + 1M context + thinking |
| **Llama 4 Scout** | Sparse MoE Transformer | 109B / 17B active | 16 routed + 1 shared | 10M | Native early-fusion multimodal | 10M context, fits single H100 |
| **DeepSeek V4 Flash** | Sparse MoE Transformer | 284B / 13B active | MoE (top-k routed) | 1M | Text | Hybrid attention (CSA+HCA), mHC, Muon optimizer |
| **Qwen 3** | Sparse MoE Transformer | 235B / 22B active | MoE | 128K (extendable) | Native multimodal | Hybrid thinking mode, MCP protocol native |
| **Mistral Large 3** | Granular Sparse MoE | 675B / 41B active | 128 | 256K | Text+vision (vision encoder) | Granular MoE + EAGLE speculative decoding |
| **Phi-4 Reasoning** | Dense Transformer | 14B | none | 16K | Text+vision (mid-fusion) | Synthetic data distillation from o3-mini |
| **Yi-Lightning** | Sparse MoE Transformer | 100B (est.) / undisclosed | fine-grained experts | 200K | Text | Fine-grained expert segmentation + KV cache sharing |
| **Grok 3** | Sparse MoE (neuro-symbolic rumored) | undisclosed | undisclosed | 131K (200K extended) | Text+image | RL-trained chain-of-thought, DeepSearch agent |

---

## 2. Architectural Innovations Extracted

### 2.1 MoE Routing Strategies

| Model | Routing | Expert Count | Activation | Special |
|-------|---------|-------------|------------|---------|
| Gemini 2.5 Pro | Sparse top-k | undisclosed | dynamic per-token | Decouples capacity from compute |
| Llama 4 Scout | Top-k + shared expert | 16 + 1 shared | 17B/109B | Shared expert always active |
| DeepSeek V4 Flash | Top-k + hash bootstrap | MoE | 13B/284B | First 3 layers use static tid→eid hash table |
| Qwen 3 | Top-k | 22B active | MoE | Hybrid thinking mode |
| Mistral Large 3 | Granular top-k | 128 | 41B/675B | Granular expert splitting |
| Yi-Lightning | Balanced routing | fine-grained | undisclosed | Cross-layer KV cache sharing |

**NeoTrix Mapping**:
- Llama 4's "shared expert + routed experts" → **GWT AttentionManager** routing with a "common reasoning baseline" (E8 anchor) + specialized domain dispatch
- DeepSeek V4's hash-bootstrap MoE → **SEAL pipeline Phase-0**: static first-layer mapping for bootstrapping, learned routing for refinement
- Mistral's granular MoE → **Skill Tree node granularity**: fine-grained capability nodes (Small Passive → Notable Passive → Keystone) activated selectively

### 2.2 Attention Mechanisms

| Model | Attention Type | KV Cache Innovation |
|-------|---------------|-------------------|
| Gemini 2.5 Pro | Standard + thinking | 1M context via MoE efficiency |
| Llama 4 Scout | Grouped-Query Attention (GQA) | 10M context via mid-training extension |
| DeepSeek V4 Flash | **Hybrid: CSA + HCA** | 73% KV cache reduction vs V3.2; CSA compresses along sequence, HCA heavy pool |
| Qwen 3 | **Hybrid: Gated DeltaNet + Gated Attention** | Every 4th layer GQA, rest linear attention |
| Mistral Large 3 | Standard | EAGLE speculative decoding for latency |
| Yi-Lightning | GQA | Cross-layer KV cache sharing |

**NeoTrix Mapping**:
- DeepSeek V4's CSA+HCA hybrid → **KVMem paged KV**: tiered memory (GPU→Host→NVMe) matching compressed attention tiers
- Qwen 3's Gated DeltaNet linear attention → **ConsciousnessTree cycle boundary**: linear attention for "ambient awareness", full attention for "focused consciousness"
- Yi-Lightning's cross-layer KV sharing → **KB embedding reuse**: shared embeddings across domain modules reduce redundant computation

### 2.3 Reasoning & Chain-of-Thought

| Model | Reasoning Approach | Special Modes |
|-------|-------------------|--------------|
| GPT-4o | Unified omni-modal reasoning | 232ms audio response latency |
| Claude 3.5 Sonnet | Thinking tokens + Constitutional AI | Artifacts (code/document side-channel) |
| Gemini 2.5 Pro | Native thinking model | Dynamic thinking budget |
| DeepSeek V4 Flash | 3 modes: Non-think / Think High / Think Max | Reasoning effort configurable per call |
| Qwen 3 | Hybrid Thinking Mode | Think/non-think toggle |
| Phi-4 Reasoning | SFT distillation from o3-mini | `<think>` / `</think>` tokens |
| Grok 3 | RL-trained CoT | Think / Big Brain / DeepSearch modes |

**NeoTrix Mapping**:
- DeepSeek V4's 3-tier reasoning → **GWT salience routing**: cheap tasks → fast model, hard tasks → reasoning model, complex → deep analysis
- Phi-4's synthetic distillation → **NT-MIND distillation**: train small models on traces from large models
- Grok 3's DeepSearch → **NT-WORLD crawl pipeline**: agent-style research with web search + synthesis

### 2.4 Efficiency & Scaling

| Model | Training Compute | Efficiency Innovation |
|-------|-----------------|---------------------|
| GPT-4o | undisclosed | 50% cheaper than GPT-4 Turbo |
| Gemini 2.5 Pro | TPUv5p, 8960-chip pods | MoE decouples capacity from cost |
| Llama 4 Scout | undisclosed | 17B active fits single H100 (INT4) |
| DeepSeek V4 Flash | 32T tokens | 10% FLOPs, 7% KV cache vs V3.2; FP4 expert precision |
| Mistral Large 3 | 3000 H200s | NVFP4 quantization, EAGLE speculative decoding |
| Qwen 3 Next | undisclosed | 80B total, 3B active (3.7% activation rate) |

**NeoTrix Mapping**:
- DeepSeek V4's FP4 expert precision → **Rune Socketing**: precision tiers for different module importance
- Llama 4's single-GPU deployment → **NT-PHYSICAL embodied constraint**: optimize for edge/resource-limited deployment
- Qwen 3's 3.7% activation → **Constellation maturity**: C0 modules use minimal compute, C5 modules use full

---

## 3. Cross-Cutting Patterns (2026)

### Pattern 1: Hybrid Attention is the New Default
Every major model now uses some form of hybrid attention — mixing sparse/compressed/local attention with full attention. Pure dense attention is extinct at scale.

**NeoTrix**: `ConsciousnessTree` already models this as "ambient vs focused" attention. Formalize into `nt_world_sense::perception_bridge` with configurable compression tiers.

### Pattern 2: MoE + Shared Expert
The "shared expert + routed experts" pattern (Llama 4, DeepSeek V4, Mistral Large 3) ensures baseline capability while specializing per-domain.

**NeoTrix**: `E8 Hexagram` reasoning already has "anchor hexagrams" (shared) + "domain hexagrams" (routed). Align naming to industry convention.

### Pattern 3: Configurable Reasoning Budget
DeepSeek V4 (3 modes), Qwen 3 (think/non-think), Grok 3 (Think/Big Brain/DeepSearch), Gemini 2.5 (dynamic thinking) — all expose reasoning effort as a runtime parameter.

**NeoTrix**: GWT salience should accept a `reasoning_budget` parameter that maps to model selection + thinking token allocation.

### Pattern 4: Synthetic Data + Distillation at Scale
Phi-4 Reasoning (14B beating 70B models), DeepSeek V4 (32T tokens), and every model uses synthetic data generation pipelines.

**NeoTrix**: `NT-MIND` distillation pipeline should formalize "teachable prompt" selection + reasoning trace generation from frontier models.

### Pattern 5: 1M+ Context is Table Stakes
Gemini (1M), DeepSeek V4 (1M), Llama 4 Scout (10M), Mistral (256K), Yi-Lightning (200K). Long context is no longer differentiating — it's baseline.

**NeoTrix**: KVMem paged KV is correctly positioned. The differentiator becomes *what you do with long context*, not *how long*.

### Pattern 6: Agent-Native Architectures
Grok 3 (DeepSearch), Claude (computer use), Gemini (agentic video), Qwen 3 (MCP protocol). Models are being designed for tool use from the ground up.

**NeoTrix**: NT-ACT's MCP gateway + NT-IO's tool interfaces are well-positioned. The gap is in agent-native training (RL on tool use), not just tool calling APIs.

---

## 4. NeoTrix Architecture Alignment

### Strengths (Already Aligned)

| NeoTrix Component | Industry Pattern | Status |
|-------------------|-----------------|--------|
| GWT Attention Routing | Cost-aware model routing (A1) | ✅ Implemented |
| KVMem Paged KV | Long context memory (A2) | ✅ Implemented |
| SEAL Pipeline | Multi-stage training/distillation | ✅ Implemented |
| NT-ACT MCP Gateway | Agent-native tool calling | ✅ Implemented |
| ConsciousnessTree | Hybrid attention (ambient/focused) | ✅ Implemented |
| Six-Layer Architecture | Layered capability separation | ✅ Implemented |

### Gaps (To Absorb)

| Industry Innovation | NeoTrix Gap | Priority | Mapping |
|--------------------|-------------|----------|---------|
| **Shared Expert + Routed Expert MoE** | No formal "baseline + specialized" routing pattern | P0 | E8 anchor hexagram + domain dispatch |
| **Configurable Reasoning Budget** | GWT salience lacks runtime reasoning_effort param | P0 | Add `reasoning_budget` to GWT broadcast |
| **Hash Bootstrap MoE** | SEAL Phase-0 uses learned routing from start | P1 | Add static tid→eid bootstrap for first layers |
| **Hybrid Linear+Full Attention** | ConsciousnessTree has concept but no implementation | P1 | Gated DeltaNet-style linear attention for ambient awareness |
| **Synthetic Data Pipeline** | NT-MIND distillation exists but no "teachable prompt" curation | P1 | Formalize prompt selection + trace generation |
| **Fine-Grained Expert Segmentation** | Skill Tree has 3 tiers but no sub-tier granularity | P2 | Add sub-tier nodes within each Skill Tree tier |
| **Cross-Layer KV Sharing** | KB embeddings not shared across modules | P2 | Implement cross-domain embedding reuse |
| **EAGLE Speculative Decoding** | No speculative decoding in NT-IO | P3 | Add draft model for LLM inference acceleration |

---

## 5. Research Targets (Next Session)

1. **DeepSeek V4 mHC (Manifold-Constrained Hyper-Connections)**: Constrain residual connections to Birkhoff polytope — could stabilize ConsciousnessTree signal propagation
2. **Qwen 3 Next's Gated DeltaNet**: Linear attention variant — candidate for NT-WORLD perception pipeline
3. **Mistral Large 3's Granular MoE**: Expert splitting strategy — candidate for Skill Tree refinement
4. **Grok 3's RL-trained CoT at scale**: RL methodology for reasoning — candidate for NT-MIND SEAL pipeline enhancement
5. **Phi-4's synthetic data recipe**: "Teachable prompt" selection algorithm — candidate for NT-MIND data curation

---

## Sources

| Model | Primary Source |
|-------|---------------|
| GPT-4o | openai.com/index/hello-gpt-4o, IBM Think |
| Claude 3.5 Sonnet | anthropic.com/news/claude-3-5-sonnet, Anthropic Model Card |
| Gemini 2.5 Pro | arxiv.org/abs/2507.06261 (Google Technical Report) |
| Llama 4 Scout | meta.com/blog/llama-4-multimodal-intelligence, HuggingFace Model Card |
| DeepSeek V4 Flash | arxiv.org/abs/2606.19348 (DeepSeek Technical Report) |
| Qwen 3 | qwen-3.com, Alibaba Cloud Blog |
| Mistral Large 3 | mistral.ai/news/mistral-3, NVIDIA Technical Blog |
| Phi-4 Reasoning | arxiv.org/abs/2504.21318 (Microsoft Technical Report) |
| Yi-Lightning | arxiv.org/abs/2412.01253 (01.AI Technical Report) |
| Grok 3 | x.ai/news/grok-3, DeepLearning.ai Batch |
