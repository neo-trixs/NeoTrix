# Model Reverse Engineering — 2026 Q3 Frontier Architecture Survey

**Date**: 2026-09-11
**Models**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3

---

## 1. Architecture Overview Matrix

| Model | Total Params | Active Params | Architecture | Context | Key Innovation |
|-------|-------------|---------------|-------------|---------|----------------|
| **GPT-4o** | ~200B (est.) | ~200B (Dense) | Dense Transformer | 128K | End-to-end omni-modal (text+audio+image+video in one forward pass) |
| **Claude 3.5 Sonnet** | Undisclosed | Undisclosed | Dense Transformer | 200K | Constitutional AI + RLHF safety alignment; no architectural disclosure |
| **Gemini 2.5 Pro** | Undisclosed | Undisclosed | MoE (undisclosed) | 1M (1,048,576) | Hybrid reasoning modes; TPUv5p training; 2M+ with thinking budget |
| **Llama 4 Scout** | 109B | 17B | MoE (16 experts) | 10M (iRope) | Native multimodal via early fusion; iRope for near-infinite context |
| **DeepSeek V4.1 Flash** | 522B | 8-16B | MoE + Hybrid Attention | 1M | Engram n-gram memory; CED (Causal Encoder-Decoder); Hyper-Connections |
| **Qwen 3** | 235B (flagship) | 22B | MoE (128 experts, 8 active) | 128K | Hybrid thinking/non-thinking mode; no shared experts; global load balance |
| **Mistral Large 3** | 675B | 41B (39B LM + 2.5B vision) | Granular MoE (128 experts) | 256K | Granular MoE with top-4 routing; Multi-Latent Attention; vision encoder |
| **Phi-4 Reasoning** | 14B | 14B (Dense) | Dense Transformer | 128K | Data-centric SFT distillation from o3-mini; competitive with 50x larger models |
| **Yi-Lightning** | ~100B | ~12B (est.) | MoE (fine-grained) | 32K | Fine-grained expert segmentation; cross-layer KV cache sharing |
| **Grok 3** | Undisclosed | Undisclosed | Transformer (likely MoE) | 131K | Colossus 10x compute training; DeepSearch real-time X integration |

---

## 2. Per-Model Deep Analysis

### 2.1 GPT-4o — End-to-End Omni-Modal

**Architecture Decisions:**
- Single neural network processes text, audio, image, video — no pipeline chaining
- 320ms average audio response (vs 5.4s in GPT-4 Turbo pipeline)
- Improved tokenizer: Gujarati 4.4x fewer tokens, English 1.1-1.3x fewer
- Estimated ~200B params (closed weights, likely MoE or dense)

**Key Innovation**: Eliminating modality boundaries. Previous approach: ASR → LLM → TTS (3 models, 3 latency hops). GPT-4o: 1 model, 1 forward pass.

**NeoTrix Mapping:**
- → **PerceptionBridge**: GPT-4o's omni-modal fusion = SensoryIntegrationHub × SelectiveState without attention gate
- → **Six-Layer Architecture**: GPT-4o collapses L1-L3 into a single forward pass — NeoTrix separates them for modularity
- → **EmotionLabel**: GPT-4o processes sentiment/tone natively; NeoTrix uses explicit EmotionEngine for regulation
- → **GWT**: GWT salience could learn from GPT-4o's attention-gated modality selection

---

### 2.2 Claude 3.5 Sonnet — Safety-First Dense Model

**Architecture Decisions:**
- Dense transformer (no MoE disclosed), 200K context
- Built on Claude 3 Opus base with post-training refinement
- 2x speed improvement over Opus at Sonnet cost tier
- 64% problem-solving on internal agentic coding eval (vs Opus 38%)

**Key Innovation**: Constitutional AI training + safety alignment at scale. Architecture is deliberately conservative; innovation is in the training methodology and alignment.

**NeoTrix Mapping:**
- → **NT-SHIELD**: Claude's safety-first alignment maps to NT-SHIELD's governance model
- → **Rev-明 (rev-officer)**: Claude's self-awareness of limitations parallels Fractal Review Loops
- → **SelfTest T3**: Claude's agentic coding eval = SelfTest T3 production wiring (detection influences behavior)

---

### 2.3 Gemini 2.5 Pro — Hybrid Reasoning at 1M Scale

**Architecture Decisions:**
- MoE architecture (details undisclosed), 1M token context
- First model trained on TPUv5p (synchronous data-parallel, multi-datacenter)
- Hybrid reasoning: configurable "thinking" budget for transparent chain-of-thought
- Architecture improvements over Gemini 1.5: better vision processing, longer video handling

**Key Innovation**: Thinking budget mechanism — user controls compute allocation per query. AIME 2025: 88% (vs Gemini 1.5's 17.5%).

**NeoTrix Mapping:**
- → **GWT salience + Cost-Aware Routing (A1)**: Thinking budget = salience-weighted compute allocation
- → **SEAL Pipeline**: Gemini's multi-stage reasoning mirrors SEAL's exploration→distillation cycle
- → **ConsciousnessTree**: Gemini's "thinking" modes parallel the 6-stage feedback loop (configurable depth)
- → **AttentionManager**: Gemini's budget mechanism = dual specialization compute switching

---

### 2.4 Llama 4 Scout — MoE + Native Multimodal + iRope

**Architecture Decisions:**
- MoE: 17B active / 109B total, 16 experts
- **Early fusion** for native multimodality (text + image tokens fused early in pipeline, not adapter-based)
- **iRope**: Novel positional encoding for 10M token context (near-infinite)
- Trained on 40T tokens (mix of public, licensed, Meta product data)
- FP4 quantization fits single B200 GPU; int4 fits single H100

**Key Innovation**: iRope for 10M context + early fusion multimodal. No adapter overhead — text and vision tokens trained jointly from pre-training.

**NeoTrix Mapping:**
- → **KVMem (A2)**: Llama 4's iRope 10M context validates Context as Scarce Resource axiom
- → **PerceptionBridge**: Early fusion = direct SensoryIntegrationHub → SelectiveState without adapter latency
- → **Rune Socketing**: 16 experts = 5-rune socket system scaled up (specialist activation per token)
- → **CapabilityBridge**: iRope's continuous context = runtime view (CapabilityRegistry) without evolution view gap

---

### 2.5 DeepSeek V4.1 Flash — Hybrid Attention + Engram Memory

**Architecture Decisions:**
- MoE: 522B total, 8B active (prompt) / 16B active (output)
- **Causal Encoder-Decoder (CED)**: 20-layer encoder on 20-layer decoder (40 total)
- **Two-tier sparse attention**: 128-token sliding window + compressed KV latents reaching further back
- **Engram n-gram memory**: 384M-row hash tables at layers 1 and 14, 4-gram lookup, gated into residual stream
- **Hyper-Connections (mHC)**: Constrained to doubly stochastic matrix manifold (Birkhoff polytope)
- **DSpark**: Multi-token draft head for speculative decoding
- Only 4 layers (2, 8, 14, 20) compress their own KV; rest read from compressed cache

**Key Innovation**: Engram memory is a novel architectural primitive — deterministic n-gram hash lookup injected into the residual stream via learned gates. This is essentially a **lookup-table memory** within the transformer, not just attention.

**NeoTrix Mapping:**
- → **nt_nexus (Cross-Session Memory)**: Engram memory = KB `kv_store` with hash-based retrieval + learned fusion gate
- → **experience-tree**: Engram's 4-gram hash → route table matching for lazy branch loading
- → **Hyper-Connections ↔ mHC**: NeoTrix's trait-based layer connections (traits.rs) could adopt doubly-stochastic constraints for stability
- → **VSA HyperCube**: Engram's compressed latent representation maps to VSA embedding with associative recall
- → **CED ↔ Six-Layer Architecture**: DeepSeek's encoder-decoder separation mirrors NeoTrix's L1-L6 layer separation (encoder = perception, decoder = action)
- → **KVMem (A2)**: V4.1 achieves 1M context at 10% KV cache vs V3 — validates paged KV virtualization

---

### 2.6 Qwen 3 — Hybrid Thinking + Fine-Grained MoE

**Architecture Decisions:**
- MoE: 235B total, 22B active, **128 experts, 8 activated per token** (no shared experts)
- Dense variants: 0.6B to 32B, all with QK-Norm (replacing QKV-bias)
- **Hybrid thinking/non-thinking mode**: Dynamic switching based on query complexity
- **Thinking budget**: User-controlled compute allocation per inference
- **Global-batch load balancing loss**: Encourages expert specialization (vs per-sample loss)
- 36T tokens, 119 languages
- **Qwen3-Next** (preview): 512 routed experts + 1 shared, hybrid linear+GQA attention every 4th layer

**Key Innovation**: No shared experts in MoE (unlike DeepSeek V3). Global-batch load balancing loss for better specialization. Hybrid thinking is a training-level innovation, not just inference prompting.

**NeoTrix Mapping:**
- → **GWT + Cost-Aware Routing (A1)**: Thinking budget = GWT salience × token cost weight
- → **Dual Specialization**: Thinking/non-thinking = Weapon Set I (acquisition) / Weapon Set II (evolution) switching
- → **Skill Tree**: 128 experts with 8 active = domain nodes with attention-gated activation
- → **Constellation maturity**: Qwen3-Next hybrid linear attention = C4→C5 leap (efficiency innovation)

---

### 2.7 Mistral Large 3 — Granular MoE + Multi-Latent Attention

**Architecture Decisions:**
- Granular MoE: 675B total, 41B active (39B LM + 2.5B vision encoder)
- **128 experts per layer**, top-4 routing with softmax
- **Multi-Latent Attention** (MLA): Key innovation from DeepSeek-V3 lineage
- Vision encoder integrated (2.5B), not adapter-based
- 256K context window
- Apache 2.0 license
- Llama 4 RoPE scaling applied

**Key Innovation**: Granular MoE — many small experts vs few large experts. Top-4 routing with softmax (not top-2 like Mixtral). MLA compresses KV representation.

**NeoTrix Mapping:**
- → **Rune Socketing**: 128 granular experts = high-granularity rune configuration (5 slots × many sub-runes)
- → **Skill Tree (3 tiers)**: Small Passive (granular expert) → Notable Passive (expert cluster) → Keystone (routing decision)
- → **CapabilityBridge**: MLA's latent compression = bridge between evolution view and runtime view
- → **GWT**: Multi-Latent Attention = attention mechanism with compressed KV — maps to GWT's salience broadcast with bounded KV

---

### 2.8 Phi-4 Reasoning — Data-Centric Small Model

**Architecture Decisions:**
- Dense transformer, 14B parameters, 128K context
- SFT from Phi-4 base using **1.4M curated "teachable" prompts**
- Reasoning traces generated by **o3-mini** (teacher distillation)
- Phi-4-reasoning-plus: additional RL phase (outcome-based RL)
- Competitive with DeepSeek-R1 (671B MoE) and o1-mini at 14B scale
- Phi-4-reasoning-vision-15B: mid-fusion multimodal with SigLIP-2 encoder

**Key Innovation**: **Data quality > model size**. 14B model outperforming 671B model through carefully curated SFT data and teacher distillation. No architectural innovation — pure training methodology.

**NeoTrix Mapping:**
- → **SEAL Pipeline**: Phi-4's SFT distillation = SEAL Phase 3 (distillation) with o3-mini as teacher
- → **experience-tree**: "Teachable" prompt curation = experience distillation (Stage 2: Distillation)
- → **R-P42/R-P79**: Phi-4 proves small models can compete — validates NeoTrix's "absorb not adapt" (R-P42)
- → **Cost-Aware Routing (A1)**: Phi-4 at 14B = optimal cheap model for simple tasks; o3-mini for hard tasks
- → **Skill Tree**: Small Passive node achieving Notable Passive capability through training, not scale

---

### 2.9 Yi-Lightning — Fine-Grained MoE with KV Optimization

**Architecture Decisions:**
- MoE with **fine-grained expert segmentation** (many small experts)
- Balanced expert routing strategy
- **Cross-layer KV cache sharing**: Multiple layers share the same KV cache (reduces memory)
- 100K+ token vocabulary for multilingual
- Number decomposition for numerical understanding
- Ranked 6th on Chatbot Arena, 2nd in Chinese/Math/Coding

**Key Innovation**: Cross-layer KV cache sharing — adjacent layers share KV representations, reducing memory by ~50% while maintaining quality.

**NeoTrix Mapping:**
- → **KVMem (A2)**: Yi-Lightning's KV sharing = Paged KV Virtualization (shared pages across layers)
- → **Obsidian Rune (cache)**: KV sharing = Obsidian rune for cache optimization
- → **Heartbeat Aggregator**: Yi's balanced routing = health signal aggregation with time-decay
- → **Dark Forest**: Yi's number decomposition = module must "connect" (numeric understanding) or be deleted

---

### 2.10 Grok 3 — Compute Scaling + Real-Time Integration

**Architecture Decisions:**
- Architecture undisclosed (likely MoE based on xAI's compute focus)
- Trained on **Colossus supercluster** with 10x compute vs previous SOTA
- **DeepSearch**: Real-time X (Twitter) integration for current information
- "Think" and "Big Brain" modes for reasoning compute scaling
- 131K context window
- RL-trained reasoning chain-of-thought at unprecedented scale

**Key Innovation**: 10x compute scaling + real-time social media integration (DeepSearch). Reasoning via RL at massive scale.

**NeoTrix Mapping:**
- → **NT-WORLD (虚空探索者)**: DeepSearch = NT-WORLD's real-time perception with X as data source
- → **GWT**: "Think" / "Big Brain" modes = GWT salience scaling compute based on task difficulty
- → **SEAL Pipeline**: Grok's RL reasoning = SEAL's exploration→distillation with massive compute
- → **Axiom A1 (Cost-Aware Routing)**: Grok's modes = explicit cost-quality tradeoff per query

---

## 3. Cross-Model Pattern Extraction

### 3.1 MoE is the Dominant Architecture (8/10 models)

| Model | Experts | Active/Token | Sparsity |
|-------|---------|-------------|----------|
| Llama 4 Scout | 16 | 17B/109B | 84% |
| DeepSeek V4.1 Flash | ~128+ | 8-16B/522B | 97-98% |
| Qwen 3-235B | 128 | 22B/235B | 91% |
| Mistral Large 3 | 128 | 41B/675B | 94% |
| Yi-Lightning | undisclosed | ~12B/100B | ~88% |

**NeoTrix Implication**: Rune Socketing should support **adaptive expert activation** — not just 5 static slots but dynamic MoE-like routing within the capability network.

### 3.2 Context Window Arms Race

| Context | Models | Technique |
|---------|--------|-----------|
| 10M | Llama 4 Scout | iRope positional encoding |
| 1M | DeepSeek V4, Gemini 2.5 Pro | Compressed sparse attention + KV compression |
| 256K | Mistral Large 3 | Standard + MLA compression |
| 200K | Claude 3.5 Sonnet | Standard transformer |
| 128K | GPT-4o, Qwen 3, Phi-4 | Standard transformer |

**NeoTrix Implication**: KB embedding must support >1M token contexts. KVMem paged KV is the right approach (Axiom A2).

### 3.3 Hybrid Reasoning is Standard

| Model | Mechanism | User Control |
|-------|-----------|-------------|
| DeepSeek V4 | Non-think / Think High / Think Max | Yes (mode selection) |
| Qwen 3 | Thinking / Non-thinking | Yes (budget) |
| Gemini 2.5 Pro | Thinking budget | Yes (configurable) |
| Grok 3 | Think / Big Brain | Yes (mode selection) |
| Phi-4 Reasoning | Always-on CoT (SFT) | No (fixed) |

**NeoTrix Implication**: AttentionManager should support **configurable reasoning depth** — GWT salience × thinking budget = compute allocation per task.

### 3.4 Attention Mechanism Innovations

| Innovation | Models | Description |
|-----------|--------|-------------|
| Engram n-gram memory | DeepSeek V4 | Hash-table memory injected into residual stream |
| Hyper-Connections (mHC) | DeepSeek V4 | Doubly-stochastic constrained residual connections |
| Multi-Latent Attention (MLA) | Mistral Large 3 | Compressed KV representation |
| Cross-layer KV sharing | Yi-Lightning | Multiple layers share KV cache |
| Hybrid linear+GQA | Qwen3-Next | Linear attention layers + GQA every 4th layer |
| iRope | Llama 4 Scout | Positional encoding for 10M context |

**NeoTrix Implication**: These are all variations of the same principle — **bounded memory with learned routing**. Maps directly to GWT's attention-gated information flow.

### 3.5 Data Quality > Model Size

| Model | Params | Training Data | Innovation |
|-------|--------|--------------|-----------|
| Phi-4 Reasoning | 14B | 1.4M curated prompts | Teacher distillation from o3-mini |
| Llama 4 Scout | 17B active | 40T tokens | Early fusion multimodal |
| Qwen 3 | 22B active | 36T tokens | Hybrid thinking training |

**NeoTrix Implication**: SEAL Pipeline's distillation phase is the highest-leverage stage. Experience-tree's 5-stage absorption is the right approach.

---

## 4. NeoTrix Mapping Summary

### 4.1 Direct Architectural Mappings

| External Innovation | NeoTrix Component | Mapping Strength |
|--------------------|-------------------|-----------------|
| MoE expert routing | Rune Socketing 5-slot system | ★★★★ (scale to dynamic) |
| Engram n-gram memory | KB kv_store + experience-tree | ★★★★★ (near-identical) |
| Hyper-Connections | traits.rs layer connections | ★★★ (adopt constraints) |
| Hybrid reasoning modes | AttentionManager dual specialization | ★★★★ |
| Early fusion multimodal | PerceptionBridge | ★★★★ |
| KV cache compression | KVMem paged KV (A2) | ★★★★★ |
| iRope long context | KB embedding + KVMem | ★★★ (adopt encoding) |
| Thinking budget | GWT salience × cost weight (A1) | ★★★★★ |
| Teacher distillation | SEAL Phase 3 + experience-tree Stage 2 | ★★★★ |
| Real-time data integration | NT-WORLD crawl pipeline | ★★★ |

### 4.2 Axiom Validation

| Axiom | External Evidence | Status |
|-------|------------------|--------|
| **A1: Cost-Aware Routing** | DeepSeek V4 modes, Qwen3 thinking budget, Grok Think/Big Brain | ✅ Validated |
| **A2: Context as Scarce Resource** | Llama 4 iRope (10M), DeepSeek KV compression (10%), Yi KV sharing | ✅ Validated |
| **A3: Skill as Production Template** | Qwen3-Next 512 experts, Phi-4 14B outperforming 671B | ✅ Validated |

### 4.3 Cross-Source Patterns (Updated)

| Pattern | Previous Definition | 2026 Update |
|---------|-------------------|-------------|
| P1: Model Routing | Route to cheapest capable model | + Reasoning depth budget per task |
| P2: Isolation-per-Task | Each task gets isolated context | + KV compression for shared contexts |
| P3: Profile-Driven | Persistent profile shapes behavior | + Hybrid thinking as profile dimension |
| P4: Ordered Backend Fallback | Single interface, ordered fallback | + Attention mechanism as fallback chain |
| P5: Skill as Reusable Template | Skills are composable atoms | + Engram memory as skill-level knowledge store |

---

## 5. Recommendations for NeoTrix

### 5.1 P0 (Immediate吸收)

1. **Engram Memory Pattern** (DeepSeek V4): Implement hash-based n-gram lookup in KB kv_store with learned gate fusion. This is the closest architectural analog to experience-tree's route table matching.

2. **Hybrid Reasoning Budget** (Qwen3/Gemini): Add configurable thinking depth to AttentionManager. Map to GWT salience × cost weight (Axiom A1).

3. **KV Cache Compression** (DeepSeek V4): Extend KVMem with compressed sparse attention layers. 10% KV cache at 1M context is achievable.

### 5.2 P1 (Next Cycle)

4. **Hyper-Connections** (DeepSeek V4): Adopt doubly-stochastic constraint on traits.rs layer connections for stability.

5. **Early Fusion Multimodal** (Llama 4): Redesign PerceptionBridge for direct token-level fusion instead of adapter-based modality bridging.

6. **Granular MoE** (Mistral Large 3): Extend Rune Socketing to support 128+ fine-grained sub-runes with learned routing.

### 5.3 P2 (Exploration)

7. **iRope Positional Encoding** (Llama 4): Research for >10M context support in KB embedding.

8. **Teacher Distillation at Scale** (Phi-4): Build SEAL Phase 3 distillation pipeline using frontier models as teachers for small NeoTrix-specific modules.

---

## 6. References

| Model | Source | Date |
|-------|--------|------|
| GPT-4o | OpenAI System Card, arXiv:2410.21276 | May 2024 |
| Claude 3.5 Sonnet | Anthropic announcement | Jun 2024 |
| Gemini 2.5 Pro | Google Tech Report, arXiv:2507.06261 | Jul 2025 |
| Llama 4 Scout | HuggingFace Model Card, NVIDIA NIM | Apr 2025 |
| DeepSeek V4.1 Flash | vLLM Recipes, HuggingFace | Apr 2026 |
| Qwen 3 | arXiv:2505.09388 | May 2025 |
| Mistral Large 3 | Mistral Docs, NVIDIA NIM | Dec 2025 |
| Phi-4 Reasoning | arXiv:2504.21318, Microsoft Research | Apr 2025 |
| Yi-Lightning | arXiv:2412.01253 | Dec 2024 |
| Grok 3 | xAI announcement, apxml.com | Feb 2025 |
