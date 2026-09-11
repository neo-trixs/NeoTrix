# Model Reverse Engineering #289 — 10-State-of-the-Art LLM Architecture Extraction (v2)

**Date**: 2026-09-11
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3
**Goal**: Extract architectural innovations from each model, map to NeoTrix subsystems, identify transferable patterns.
**Sources**: Official blogs, HuggingFace model cards, API docs, arXiv papers, DeepSeek V4.1 tech report.
**Delta vs #288**: DeepSeek V4.1 Flash updated with CED architecture, CSA2, Engram, DSpark from official tech report. Qwen 3 updated with full 4-stage post-training details. Phi-4-reasoning updated with training specifics.

---

## 1. GPT-4o (OpenAI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | ~200B (estimated) |
| Context | 128K |
| Modality | Omni-modal (text+audio+image+video) |
| Training | End-to-end across all modalities |
| Architecture | Undisclosed (MoE suspected) |
| Latency | 320ms avg audio response |

### Key Innovation: **End-to-End Omni-Modal Training**
- Single neural network trained jointly on text, audio, image, video — no pipeline chaining
- Eliminated the ASR→LLM→TTS pipeline: 320ms audio latency (vs 5.4s previously, 16x improvement)
- Native cross-modal reasoning without modality-specific adapters
- Unified tokenization: Korean 1.7x fewer tokens, Vietnamese 1.5x fewer tokens vs GPT-4
- 50% cheaper than GPT-4 Turbo with matching text/code performance

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Omni-modal training | `nt_world_sense::PerceptionBridge` | Extend PerceptionBridge to unified audio-visual-text embeddings; E8 attention gating for cross-modal salience |
| 320ms latency target | `nt_io::LlmRouter` | GWT salience-based routing with latency SLA per modality tier |
| Pipeline elimination | `nt_act::ProductionOrchestrator` | Collapse multi-step agent pipelines into single-pass calls where possible |
| Unified tokenization | `nt_memory::KB` | Unified token budget across modalities in KB queries |

---

## 2. Claude 3.5 Sonnet (Anthropic)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | 200K |
| Architecture | Dense transformer (confirmed via apxml) |
| Modality | Text + vision |
| Speed | 2x Claude 3 Opus |
| Cost | $3/$15 per M tokens |
| SWE-bench | 64% (vs Opus 38%) |

### Key Innovation: **Dense Architecture + Constitutional AI + Agentic Fine-Tuning**
- Dense (non-MoE) architecture optimized for speed-intelligence Pareto frontier
- Constitutional AI: safety alignment baked into training via constitutional principles (not post-hoc filtering)
- Agentic coding: tool-use fine-tuning achieves 64% SWE-bench (38% → 64% improvement)
- Artifacts: structured output workspace with live editing state
- Prompt caching: up to 90% cost reduction for repeated context

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Constitutional AI | `nt_shield::EgressPrivacyGuard` | Extend guard to Constitutional-style principle injection at inference time |
| Speed-intelligence Pareto | `nt_core_self::AttentionManager` | Dual-weapon-set routing (Ascendancy) with cost-latency-aware switching |
| Agentic tool-use training | `nt_act::McpToolRegistry` | Fine-tune tool-calling schemas; structured artifact output as first-class |
| Prompt caching 90% | `nt_memory::KVCache` | Tiered caching: hot prompts in GPU KV, warm in host, cold on NVMe |

---

## 3. Gemini 2.5 Pro (Google)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | 1,048,576 tokens (1M) |
| Architecture | Sparse MoE Transformer |
| Modality | Text + image + audio + video (native) |
| Output | 65,536 tokens |
| Training | TPUv5p, 8960-chip pods |
| Key Feature | "Thinking" mode + Deep Think |

### Key Innovation: **1M-Token Context + Sparse MoE + Thinking Budgets**
- Sparse MoE: activate subset of parameters per input token, decouple capacity from compute cost
- Native "thinking" mode: internal chain-of-thought with developer-controlled compute budgets
- Deep Think: multi-hypothesis generation with critique before final answer
- 1M context: 3-hour video understanding, video-to-code conversion
- Architecture refinements from Gemini 1.5 → 2.5: improved training stability and computational efficiency
- LiveCodeBench: 30.5% (1.5 Pro) → 74.2% (2.5 Pro)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Sparse MoE routing | `nt_core_self::AttentionManager` | Dynamic expert selection: route tokens to specialized sub-networks based on GWT salience |
| Thinking budgets | `nt_mind::SEAL_Pipeline` | Budget-aware reasoning: allocate SEAL phases based on task complexity budget |
| Deep Think | `nt_core::E8_Hexagram` | Multi-hypothesis exploration in E8 reasoning: generate parallel hexagram paths, critique, select |
| 1M context | `nt_memory::KB` + `kv_cache_optimizer.rs` | Paged KV virtualization for long-context sessions (KVMem integration) |
| Vision improvements | `nt_world_sense::PerceptionBridge` | Enhanced visual encoding in perception bridge; video frame sampling strategy |

---

## 4. Llama 4 Scout (Meta)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Active Params | 17B |
| Total Params | 109B |
| Experts | 16 (MoE) |
| Context | 10M tokens |
| Training Data | ~40 trillion tokens |
| Modality | Text + image (native multimodal via early fusion) |
| License | Llama 4 Community License |
| Deployment | Single H100 GPU (int4 quantized) |

### Key Innovation: **Ultra-Efficient MoE + 10M Context + Early Fusion**
- 17B active / 109B total: only 15.6% parameters active per token
- Early fusion for native multimodality: text and images processed in same token stream
- 10M token context window (largest open-weight model context)
- Single H100 GPU deployment with int4 quantization
- 40T token pretraining (largest open-weight dataset)
- 200 language support with 12 language fine-tuning

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| 17B/109B MoE ratio | `nt_io::LlmRouter` | Cost-aware routing: use 17B-equivalent for simple tasks, full 109B for complex reasoning |
| Early fusion | `nt_world_sense::PerceptionBridge` | Unified token stream for text+image in perception; no separate vision encoder |
| 10M context | `nt_memory::KVMem` | Paged KV virtualization with tiered storage (GPU→Host→NVMe) for ultra-long sessions |
| Single-GPU deployment | `nt_act::ResourceBudget` | Quantization-aware scheduling: int4 for edge, bf16 for cloud |
| 40T token training | `nt_mind::SEAL_Pipeline` | Data scaling strategy: synthetic data generation pipeline for domain-specific pretraining |

---

## 5. DeepSeek V4.1 Flash (DeepSeek) — UPDATED

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Backbone Params | 552B (763B total) |
| Active Params | 8B (prefill) / 16B (decode) — asymmetric |
| Architecture | Causal Encoder-Decoder (CED): 20L encoder + 20L decoder |
| Context | 1M tokens |
| Experts | 384 routed + 1 shared per layer, 6 routed active/token |
| KV Cache | 890 bytes/token (1/4 of V4-Flash, 1/437 of V1) |
| Training Data | 45T tokens (multimodal) |
| Vision | DeepSeek-ViT (2D-RoPE, 3×3 pixel-unshuffle) |
| Reasoning Effort | Continuously controllable 1-100 |
| License | MIT |

### Key Innovation: **Causal Encoder-Decoder + KV Cache Compression + Asymmetric Compute**
- **CED Architecture**: 40-layer Transformer as 20L encoder + 20L decoder. Decoder's global KV cache is projected from encoder final hidden states, not from each decoder layer. This enables asymmetric activation: 8B for input (prefill), 16B for output (decode).
- **Compressed Sparse Attention 2 (CSA2)**: Three static modes per layer (Full, Reindex, Reuse) sharing main KV and indexer K across layers. Hierarchical Sparse Indexer restricts deeper layers to candidate pool from first Full layer, bounding cost independent of context length. FP4 main KV caching (E2M1 format, 1 E4M3 scale per 16 channels).
- **SWA Bounded Replay**: Reconstructs missing sliding-window attention KV states by replaying only the most recent n_win tokens. Avoids SSD persistence for SWA KV, reducing persistent cache footprint to ~1/8 of V4-Flash.
- **Engram Conditional Memory**: 196B parameters sparsely accessed via token-based lookup — a form of external memory augmentation.
- **DSpark Speculative Decoding**: Semi-autoregressive draft generation with confidence-scheduled verification for inference acceleration.
- **DeepSeek-ViT**: Vision encoder trained from scratch with 2D-RoPE and pixel-unshuffle downsampling. Two-layer MLP projector for image→text embedding alignment.
- **Reasoning Effort Control**: Integer 1-100 slider trading inference cost for accuracy, enabling continuous budget allocation.
- **Post-training**: SFT → RL → on-policy distillation (OPD). Large-scale automated synthesis of agent tasks with progressive scaling.

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| CED asymmetric compute | `nt_io::LlmRouter` | Asymmetric routing: lightweight encoder for perception (8B-equiv), heavier decoder for generation (16B-equiv) |
| CSA2 sparse attention | `nt_core::E8_Hexagram` | Hierarchical attention gating: Full/Reindex/Reuse modes as E8 hexagram attention tiers |
| FP4 KV cache (890 B/token) | `nt_memory::KVMem` | Aggressive KV compression: FP4 quantization for hot KV, FP8 for warm, BF16 for cold |
| SWA Bounded Replay | `nt_memory::KVMem` | Sliding-window reconstruction: replay only recent n tokens for session-local context |
| Engram conditional memory | `nt_memory::KB` | Token-based KB lookup: sparse external memory accessed by content hash, not address |
| DSpark speculative decoding | `nt_core_self::AttentionManager` | Speculative reasoning: draft hypotheses with confidence-scheduled verification in E8 |
| Reasoning effort 1-100 | `nt_mind::SEAL_Pipeline` | Continuous SEAL depth: effort slider maps to number of SEAL phases executed |
| DeepSeek-ViT | `nt_world_sense::PerceptionBridge` | Native vision encoder: 2D-RoPE for spatial reasoning in perception bridge |
| Agent task synthesis | `nt_mind::SkillEngine` | Automated skill generation: synthetic agent tasks for RL-based skill crystallization |

---

## 6. Qwen 3 (Alibaba) — UPDATED

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| MoE Flagship | 235B total / 22B active (128 experts, 8 activated) |
| MoE Small | 30B total / 3B active (128 experts, 8 activated) |
| Dense Sizes | 32B, 14B, 8B, 4B, 1.7B, 0.6B |
| Context | 128K (MoE), 32K-128K (dense) |
| Languages | 119 languages/dialects |
| Training Data | ~36 trillion tokens |
| KV Heads | 4 (GQA for MoE), 8 (GQA for dense) |
| Training Stages | 3-stage pre-training + 4-stage post-training |

### Key Innovation: **Hybrid Thinking + 4-Stage Post-Training + Scale Efficiency**
- Hybrid thinking: `/think` and `/no_think` commands for dynamic mode switching per turn
- **4-stage post-training**: (1) CoT cold start on diverse long CoT data, (2) Reasoning RL with rule-based rewards, (3) Thinking mode fusion (blend thinking + non-thinking data), (4) General RL across 20+ tasks
- **3-stage pre-training**: S1 (30T tokens, 4K context) → S2 (5T tokens, knowledge-intensive) → S3 (long-context extension to 32K)
- Qwen3-30B-A3B rivals QwQ-32B with 10x fewer active parameters
- Qwen3-4B rivals Qwen2.5-72B-Instruct (18x parameter efficiency)
- 119 languages: broadest multilingual coverage
- Budget-controlled reasoning: smooth performance scaling with compute budget
- 1 shared expert + 8 activated routed experts per MoE layer

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Hybrid thinking modes | `nt_mind::SEAL_Pipeline` | Dynamic SEAL depth control: thinking mode = full 6-stage, non-thinking = Soil→Fruits fast path |
| 4-stage post-training | `nt_mind::SkillEngine` | Crystallization pipeline: CoT cold start → RL refinement → mode fusion → general polish |
| Budget-controlled reasoning | `nt_core_self::AttentionManager` | GWT budget allocator: smooth salience scaling with token budget |
| 128-expert MoE | `nt_core::E8_Hexagram` | Expert routing as hexagram selection: 128 expert nodes mapped to 64-hexagram grid |
| 119 language support | `nt_io::LlmRouter` | Multi-language routing: language detection → model selection based on linguistic complexity |
| 3-stage pre-training | `nt_mind::SEAL_Pipeline` | Curriculum learning: progressive data quality and context length scaling |

---

## 7. Mistral Large 3 (Mistral AI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| License | Apache 2.0 |
| Modality | Multimodal (text + vision) |
| Context | Not disclosed (likely 128K) |
| Type | Open-weight, general-purpose |
| Release | v25.12 (December 2025) |
| Predecessor | Mistral Large 2.0/2.1 |

### Key Innovation: **Open-Weight Frontier + Apache 2.0 + Ecosystem Integration**
- Open-weight under Apache 2.0: full commercial use permitted
- State-of-the-art general-purpose multimodal model
- Part of comprehensive Mistral ecosystem: Studio (platform), Forge (training), Vibe (agent)
- Modular model lineup: Small 4 (hybrid instruct/reasoning), Medium 3.5 (frontier agentic), Large 3 (general)
- Enterprise-first: hybrid deployment (cloud + on-prem + self-hosted)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Open-weight Apache 2.0 | `nt_io::LlmRouter` | Local model hosting: self-hosted Mistral for Egress Privacy Guard compliance |
| Modular model lineup | `nt_io::LlmRouter` | Multi-tier routing: Small (I/O), Medium (reasoning), Large (complex) — cost-aware delegation |
| Enterprise hybrid deploy | `nt_shield::Sandbox` | Hybrid trust zones: local Mistral for sensitive data, cloud for general tasks |
| Ecosystem integration | `nt_act::ProductionOrchestrator` | Platform-agnostic orchestration: abstract Mistral/OpenAI/Anthropic behind unified interface |

---

## 8. Phi-4 Reasoning (Microsoft) — UPDATED

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 14B |
| Architecture | Dense decoder-only Transformer (same as Phi-4 base) |
| Context | 32K |
| Training Data | 16B tokens (~8.3B unique) |
| Training Time | 2.5 days on 32 H100-80G GPUs |
| License | MIT |
| Training Period | January 2025 – April 2025 |
| Base Model | microsoft/phi-4 (14B dense) |
| Tensor Type | BF16 |
| Model Size | 15B params (safetensors) |

### Key Innovation: **Small Model Reasoning via CoT + RL Distillation**
- 14B params achieves reasoning competitive with DeepSeek-R1 (671B) and o1-mini
- AIME 2024: 75.3% (Phi-4-reasoning) vs 74.6% (o1) — 14B vs unbounded params
- Training: SFT on CoT traces + rule-based RL (math, science, code focused)
- **Thought/Solution structure**: explicit `<think>` reasoning chain before `</think>` answer
- Extreme efficiency: 2.5 days training on 32 GPUs (vs months for frontier models)
- MIT license: full commercial use, smallest reasoning model at SOTA level
- Synthetic data pipeline: filtered high-quality web data + LLM-generated CoT traces
- Safety via SFT: synthetic prompts with LLM-generated safety-aligned responses
- Phi-4-reasoning-plus variant achieves AIME 2025: 78.0% (14B params)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| CoT distillation | `nt_mind::SEAL_Pipeline` | Experience distillation: compress reasoning traces into compact skill nodes |
| Small model reasoning | `nt_io::LlmRouter` | Edge reasoning: route complex reasoning to Phi-4 locally, simple tasks to flash models |
| Thought/Solution structure | `nt_core::E8_Hexagram` | Structured reasoning output: Hexagram → Thought (exploration) → Solution (consensus) |
| 2.5-day training | `nt_mind::SkillEngine` | Rapid skill crystallization: fast training loops for domain-specific reasoning skills |
| Rule-based RL | `nt_meta::CrossModuleAudit` | Automated audit RL: rule-based reward for cross-module consistency checks |
| Synthetic data pipeline | `nt_mind::SEAL_Pipeline` | Synthetic experience generation: LLM-crafted training data for skill nodes |

---

## 9. Yi-Lightning (01.ai)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | Not publicly detailed |
| Type | Competitive multimodal model |
| Key Feature | High performance at competitive cost |
| Ecosystem | Part of Yi model family |

### Key Innovation: **Competitive Performance + Cost Optimization**
- Competitive with frontier models on major benchmarks
- Focus on cost-effective deployment
- Part of 01.ai's strategy to provide accessible frontier-level AI
- Multimodal capabilities for text and vision tasks

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Cost-effective frontier | `nt_io::LlmRouter` | Cost-performance routing: Yi-Lightning as mid-tier option between flash and frontier |
| Accessible deployment | `nt_act::ResourceBudget` | Budget-aware model selection: right-size model to task complexity |

---

## 10. Grok 3 (xAI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Training Compute | 10x Grok-2 (200K GPU cluster "Colossus") |
| Context | 128K |
| Key Feature | Think mode (reasoning), Big Brain mode |
| Benchmarks | AIME 2025, GPQA competitive with o3-mini |
| Pricing | $3/$15 per M tokens (API) |
| Training Data | Expanded dataset including legal filings |

### Key Innovation: **Massive Compute Scaling + Think/Big Brain Modes + Real-Time Integration**
- 10x compute over predecessor: 200K GPU Colossus cluster
- Think mode: reasoning trace for complex problems (similar to o3)
- Big Brain mode: extra compute allocation for hardest problems (never publicly released)
- DeepSearch: internet + X scanning for real-time information retrieval
- X platform integration: real-time social media context
- Grok 4 (successor): 1.5T parameter V9 foundation, 2M context window

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Think/Big Brain modes | `nt_mind::SEAL_Pipeline` | Tiered reasoning: Think = 3-stage SEAL, Big Brain = full 6-stage with resource boost |
| 200K GPU training | `nt_mind::SkillEngine` | Large-scale skill training: allocate GPU clusters for domain-specific skill crystallization |
| DeepSearch | `nt_world::UnifiedCrawler` | Real-time knowledge acquisition: integrate X/social media as first-class crawl source |
| X platform context | `nt_memory::KB` | Social context enrichment: real-time social signals integrated into KB node metadata |
| 2M context (Grok 4) | `nt_memory::KVMem` | Extended paged KV: 2M token sessions with GPU→Host→NVMe tiering |

---

## Cross-Model Architecture Patterns

### Pattern 1: MoE is the Default
| Model | Type | Active/Total | Experts | Active Ratio |
|-------|------|-------------|---------|-------------|
| DeepSeek V4.1 Flash | CED MoE | 8B-16B/552B | 384+1 | 1.5%-2.9% |
| Gemini 2.5 Pro | MoE | undisclosed | undisclosed | — |
| Llama 4 Scout | MoE | 17B/109B | 16 | 15.6% |
| Qwen 3-235B | MoE | 22B/235B | 128 | 9.4% |
| Qwen 3-30B | MoE | 3B/30B | 128 | 10.0% |

**NeoTrix Implication**: MoE routing is now standard. GWT salience should incorporate expert-routing logic. The `AttentionManager` needs dynamic expert selection analogous to MoE top-k routing. DeepSeek V4.1 Flash pushes the envelope: only 1.5% of params active during prefill.

### Pattern 2: Hybrid Thinking is Universal
| Model | Thinking Implementation |
|-------|------------------------|
| Gemini 2.5 Pro | Developer-controlled thinking budgets |
| DeepSeek V4.1 Flash | Continuously controllable 1-100 reasoning effort |
| Qwen 3 | `/think` `/no_think` turn-level switching |
| Grok 3 | Think mode + Big Brain mode |
| Phi-4-reasoning | Thought/Solution structured output |

**NeoTrix Implication**: SEAL Pipeline needs adaptive depth. `enable_thinking` equivalent: shallow (Soil→Fruits) vs deep (full 6-stage) based on task complexity. DeepSeek's 1-100 slider is the most granular — map to SEAL phase count.

### Pattern 3: Context Windows are Exploding
| Model | Context |
|-------|---------|
| Llama 4 Scout | 10M |
| Grok 4 (successor) | 2M |
| Gemini 2.5 Pro | 1M |
| DeepSeek V4.1 Flash | 1M |
| Qwen 3 | 128K |

**NeoTrix Implication**: KVMem paged KV virtualization is critical. Experience-tree lazy branch loading aligns with this: load only relevant context nodes, not full history.

### Pattern 4: Cost-Aware Routing is Mandatory
| Model | Input Cost/M | Output Cost/M |
|-------|-------------|---------------|
| DeepSeek V4.1 Flash | $0.15 (off-peak) | $0.60 |
| Llama 4 Scout | Self-hosted | Self-hosted |
| Phi-4-reasoning | Self-hosted | Self-hosted |
| Mistral Large 3 | Self-hosted | Self-hosted |
| GPT-4o | $2.50 | $10.00 |
| Claude 3.5 Sonnet | $3.00 | $15.00 |
| Grok 3 | $3.00 | $15.00 |

**NeoTrix Implication**: Axiom A1 (Cost-Aware Routing) confirmed. GWT salience must include cost weight. Ordered Backend Router should include price-per-token in routing decisions.

### Pattern 5: Native Multimodality Wins
| Model | Multimodal Approach |
|-------|-------------------|
| GPT-4o | End-to-end omni (text+audio+image+video) |
| Gemini 2.5 Pro | Native multimodal (text+image+audio+video) |
| DeepSeek V4.1 Flash | Native ViT encoder + text (image+text) |
| Llama 4 Scout | Early fusion (text+image) |
| Claude 3.5 | Vision via separate encoder |
| Phi-4-reasoning | Text only |

**NeoTrix Implication**: PerceptionBridge should adopt early fusion for text+image. DeepSeek-ViT with 2D-RoPE provides a reference architecture for native vision in perception bridge.

### Pattern 6: KV Cache Compression is a Frontier (NEW)
| Model | KV Cache Strategy | Size/Token |
|-------|------------------|------------|
| DeepSeek V4.1 Flash | FP4 KV + CSA2 + SWA Bounded Replay | 890 bytes |
| DeepSeek V4-Flash | Previous gen | 3,560 bytes (4x) |
| DeepSeek V1 | Baseline | 389K bytes (437x) |

**NeoTrix Implication**: Aggressive KV compression is critical for long-context cost efficiency. KVMem should implement FP4 quantization for hot KV pages, CSA2-style sparse attention for GWT routing, and SWA replay for session-local context.

### Pattern 7: Asymmetric Encoder-Decoder (NEW — DeepSeek V4.1)
- CED architecture: encoder handles perception (8B active), decoder handles generation (16B active)
- KV cache projected from encoder, not decoder layers — 4x compression
- Separates perception cost from generation cost

**NeoTrix Implication**: PerceptionBridge (L2) and ActionLayer (L1) should have different compute budgets. Encoder-equivalent for perception can be 2x lighter than decoder-equivalent for generation.

---

## NeoTrix Transfer Priority Matrix

### P0 — Immediate Absorption (this cycle)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| Asymmetric CED compute | DeepSeek V4.1 | L2/L1 layer budget separation | Fundamental: perception 2x cheaper than generation |
| FP4 KV compression | DeepSeek V4.1 | KVMem 890 bytes/token | 4x cost reduction for long sessions |
| Continuous reasoning effort 1-100 | DeepSeek V4.1 | SEAL Pipeline depth control | Most granular thinking budget control |
| Dual thinking modes | Qwen 3 / DeepSeek | SEAL Pipeline adaptive depth | Universal pattern, directly applicable |
| Cost-aware routing | All models | GWT salience + cost weight | Axiom A1 confirmed by 10/10 models |
| MoE expert routing | Gemini / Llama / Qwen / DeepSeek | E8 Hexagram expert selection | Fundamental architecture shift |

### P1 — Short-Term Integration (1-2 cycles)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| CSA2 sparse attention | DeepSeek V4.1 | GWT attention gating | Hierarchical indexing reduces attention cost |
| Engram conditional memory | DeepSeek V4.1 | KB token-based lookup | Sparse external memory augmentation |
| SWA Bounded Replay | DeepSeek V4.1 | KVMem session-local context | Avoid SSD persistence for sliding window |
| Early fusion multimodal | GPT-4o / Llama 4 / DeepSeek | PerceptionBridge upgrade | Unified modality processing |
| Prompt caching 90% | Claude 3.5 | KVCache tiering | Cost reduction, already proven |
| CoT distillation | Phi-4-reasoning | Skill crystallization pipeline | Small-model efficiency |
| DSpark speculative decoding | DeepSeek V4.1 | AttentionManager | Draft hypotheses with confidence verification |

### P2 — Medium-Term Research (2-4 cycles)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| 10M+ context sessions | Llama 4 Scout | KVMem + experience-tree | Ultra-long session support |
| 4-stage post-training | Qwen 3 | SEAL Pipeline stages | Skill training pipeline refinement |
| Real-time social integration | Grok 3 | UnifiedCrawler + KB | Live knowledge acquisition |
| Budget-controlled reasoning | Qwen 3 / DeepSeek | AttentionManager | Smooth compute allocation |
| 2D-RoPE vision encoding | DeepSeek-ViT | PerceptionBridge | Spatial reasoning in perception |
| DeepSearch real-time crawl | Grok 3 | UnifiedCrawler | Internet + social as first-class source |

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MoE efficiency vs Dense quality (Claude 3.5) | Both viable: Dense for speed-critical, MoE for capacity-critical. NeoTrix routes accordingly. |
| Thinking mode latency vs accuracy | Budget-aware: short tasks → non-thinking, long tasks → thinking. Adaptive SEAL depth (1-100 slider). |
| Open-weight (Llama/Qwen) vs Proprietary (GPT/Claude) | Ordered Backend Router: open-weight for local/sensitive, proprietary for frontier capability |
| 10M context vs KV cost | Tiered storage: hot 128K in GPU, warm 1M in host, cold 10M on NVMe. FP4 KV compression (890 bytes/token). |
| Small model reasoning (Phi-4 14B) vs Frontier (GPT-4o ~200B) | Cost-aware delegation: Phi-4 for edge reasoning, frontier for complex multi-step tasks |
| Asymmetric compute (8B prefill / 16B decode) vs symmetric design | PerceptionBridge (L2) gets lighter budget, ActionLayer (L1) gets heavier budget — mirrors CED |
| KV cache growth vs compression | CSA2 + FP4 + SWA Bounded Replay: compress aggressively, reconstruct cheaply |

---

## Deep Dive: DeepSeek V4.1 Flash Architecture (NEW)

This section provides detailed analysis of the most architecturally novel model in this batch.

### Causal Encoder-Decoder (CED) — Why It Matters
The traditional Transformer decoder uses its own hidden states for KV cache at every layer. DeepSeek V4.1 Flash projects the decoder's global KV cache from the **encoder's final hidden states** instead. This means:
- Encoder processes input with 8B active params (lightweight)
- Decoder generates output with 16B active params (heavier)
- KV cache size is determined by encoder, not decoder depth
- Result: 890 bytes/token vs 3,560 bytes/token (V4-Flash)

**NeoTrix Analogy**: The PerceptionBridge (L2) is the "encoder" — it processes sensory input with lighter compute. The ActionLayer (L1) is the "decoder" — it generates actions with heavier compute. The KB "KV cache" should be sized based on perception depth, not action depth.

### CSA2 — Hierarchical Sparse Attention
Three static modes per attention layer:
- **Full**: Full attention (establishes global context)
- **Reindex**: Reuses indexer from Full layer, reindexes with local context
- **Reuse**: Reuses both KV and indexer from previous layers

The Hierarchical Sparse Indexer restricts deeper layers to a candidate pool constructed by the first Full layer, bounding cost independent of context length. Combined with FP4 caching, this achieves 890 bytes/token.

**NeoTrix Analogy**: E8 Hexagram attention tiers — Full (all 64 hexagrams), Reindex (local cluster), Reuse (cached results). The GWT refinement layer should implement this hierarchical attention.

### Engram — External Memory by Content
196B parameters accessed sparsely via token-based lookup. Not a traditional attention mechanism — it's more like a differentiable hash map where tokens are keys and stored knowledge is values.

**NeoTrix Analogy**: This is exactly what KB nodes are — content-addressed storage accessed by semantic similarity. The `neotrix-experience query --kw` mechanism is a non-differentiable Engram. Making KB lookups differentiable (soft attention over nodes) would be a direct adoption.

### DSpark — Speculative Draft + Verify
Semi-autoregressive draft generation: produce multiple draft tokens in parallel, then verify with the full model. Confidence-scheduled: verify only when draft confidence is below threshold.

**NeoTrix Analogy**: E8 multi-hypothesis exploration — generate parallel hexagram paths, verify only the promising ones. The AttentionManager should implement confidence-gated verification to avoid wasting compute on obvious paths.
