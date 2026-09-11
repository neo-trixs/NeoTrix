# Model Reverse Engineering #288 — 10-State-of-the-Art LLM Architecture Extraction

**Date**: 2026-09-11
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3
**Goal**: Extract architectural innovations from each model, map to NeoTrix subsystems, identify transferable patterns.

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
| Speed-intelligence Parefo | `nt_core_self::AttentionManager` | Dual-weapon-set routing (Ascendancy) with cost-latency-aware switching |
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

## 5. DeepSeek V4.1 Flash (DeepSeek)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | 1M tokens |
| Max Output | 384K tokens |
| Modality | Text + vision |
| Thinking | Both thinking and non-thinking modes |
| Pricing | $0.15/M input (off-peak), $0.60/M output |
| Key Feature | Vision support, thinking mode, ultra-low cost |

### Key Innovation: **Ultra-Low Cost + Dual Thinking Modes + Vision**
- Extremely low pricing: $0.15/M input tokens (off-peak) — 10-50x cheaper than competitors
- Dual thinking modes: non-thinking (fast) and thinking (reasoning) per request
- Vision support integrated into flash model
- Off-peak pricing: 50% discount during non-peak hours
- V4.1 Flash surpasses V4 Pro in performance, cost, and speed
- 384K max output tokens (longest output window)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Dual thinking modes | `nt_mind::SEAL_Pipeline` | Adaptive SEAL depth: thinking mode triggers deep SEAL phases, non-thinking uses shallow fast path |
| Ultra-low cost routing | `nt_io::LlmRouter` | GWT salience + cost weight: flash-tier routing for I/O-heavy tasks (Axiom A1) |
| 384K output | `nt_memory::KB` | Long-form experience logging: full session transcripts stored without truncation |
| Off-peak pricing | `nt_act::ResourceBudget` | Time-aware cost optimization: schedule non-urgent tasks during off-peak windows |

---

## 6. Qwen 3 (Alibaba)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| MoE Flagship | 235B total / 22B active (128 experts) |
| MoE Small | 30B total / 3B active (128 experts) |
| Dense Sizes | 32B, 14B, 8B, 4B, 1.7B, 0.6B |
| Context | 128K (MoE), 32K-128K (dense) |
| Languages | 119 languages/dialects |
| Training Data | ~36 trillion tokens |
| Key Feature | Hybrid thinking modes (thinking + non-thinking) |

### Key Innovation: **Hybrid Thinking + 4-Stage Post-Training + Scale Efficiency**
- Hybrid thinking: `/think` and `/no_think` commands for dynamic mode switching per turn
- 4-stage post-training: (1) CoT cold start, (2) Reasoning RL, (3) Thinking mode fusion, (4) General RL
- Qwen3-30B-A3B rivals QwQ-32B with 10x fewer active parameters
- Qwen3-4B rivals Qwen2.5-72B-Instruct (18x parameter efficiency)
- 119 languages: broadest multilingual coverage
- Budget-controlled reasoning: smooth performance scaling with compute budget

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Hybrid thinking modes | `nt_mind::SEAL_Pipeline` | Dynamic SEAL depth control: thinking mode = full 6-stage, non-thinking = Soil→Fruits fast path |
| 4-stage post-training | `nt_mind::SkillEngine` | Crystallization pipeline: cold start → RL refinement → mode fusion → general polish |
| Budget-controlled reasoning | `nt_core_self::AttentionManager` | GWT budget allocator: smooth salience scaling with token budget |
| 128-expert MoE | `nt_core::E8_Hexagram` | Expert routing as hexagram selection: 128 expert nodes mapped to 64-hexagram grid |
| 119 language support | `nt_io::LlmRouter` | Multi-language routing: language detection → model selection based on linguistic complexity |

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

## 8. Phi-4 Reasoning (Microsoft)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 14B |
| Architecture | Dense decoder-only Transformer |
| Context | 32K |
| Training Data | 16B tokens (~8.3B unique) |
| Training Time | 2.5 days on 32 H100-80G GPUs |
| License | MIT |
| Key Feature | Reasoning via CoT + RL on small model |

### Key Innovation: **Small Model Reasoning via CoT + RL Distillation**
- 14B params achieves reasoning competitive with DeepSeek-R1 (671B) and o1-mini
- AIME 2024: 75.3% (Phi-4-reasoning) vs 74.6% (o1) — 14B vs unbounded params
- Training: SFT on CoT traces + rule-based RL (math, science, code focused)
- Thought/Solution structure: explicit reasoning chain before answer
- Extreme efficiency: 2.5 days training on 32 GPUs (vs months for frontier models)
- MIT license: full commercial use, smallest reasoning model at SOTA level

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| CoT distillation | `nt_mind::SEAL_Pipeline` | Experience distillation: compress reasoning traces into compact skill nodes |
| Small model reasoning | `nt_io::LlmRouter` | Edge reasoning: route complex reasoning to Phi-4 locally, simple tasks to flash models |
| Thought/Solution structure | `nt_core::E8_Hexagram` | Structured reasoning output: Hexagram → Thought (exploration) → Solution (consensus) |
| 2.5-day training | `nt_mind::SkillEngine` | Rapid skill crystallization: fast training loops for domain-specific reasoning skills |
| Rule-based RL | `nt_meta::CrossModuleAudit` | Automated audit RL: rule-based reward for cross-module consistency checks |

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
| Model | Type | Active/Total | Experts |
|-------|------|-------------|---------|
| Gemini 2.5 Pro | MoE | undisclosed | undisclosed |
| Llama 4 Scout | MoE | 17B/109B | 16 |
| Qwen 3-235B | MoE | 22B/235B | 128 |
| Qwen 3-30B | MoE | 3B/30B | 128 |
| DeepSeek V4.1 | MoE | undisclosed | undisclosed |

**NeoTrix Implication**: MoE routing is now standard. GWT salience should incorporate expert-routing logic. The `AttentionManager` needs dynamic expert selection analogous to MoE top-k routing.

### Pattern 2: Hybrid Thinking is Universal
| Model | Thinking Implementation |
|-------|------------------------|
| Gemini 2.5 Pro | Developer-controlled thinking budgets |
| DeepSeek V4.1 | Thinking/non-thinking per request |
| Qwen 3 | `/think` `/no_think` turn-level switching |
| Grok 3 | Think mode + Big Brain mode |
| Phi-4-reasoning | Thought/Solution structured output |

**NeoTrix Implication**: SEAL Pipeline needs adaptive depth. `enable_thinking` equivalent: shallow (Soil→Fruits) vs deep (full 6-stage) based on task complexity.

### Pattern 3: Context Windows are Exploding
| Model | Context |
|-------|---------|
| Llama 4 Scout | 10M |
| Gemini 2.5 Pro | 1M |
| DeepSeek V4.1 | 1M |
| Qwen 3 | 128K |
| Grok 4 | 2M |

**NeoTrix Implication**: KVMem paged KV virtualization is critical. Experience-tree lazy branch loading aligns with this: load only relevant context nodes, not full history.

### Pattern 4: Cost-Aware Routing is Mandatory
| Model | Input Cost/M | Output Cost/M |
|-------|-------------|---------------|
| DeepSeek V4.1 | $0.15 | $0.60 |
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
| Llama 4 Scout | Early fusion (text+image) |
| Claude 3.5 | Vision via separate encoder |
| Phi-4-reasoning | Text only |

**NeoTrix Implication**: PerceptionBridge should adopt early fusion for text+image. Audio pipeline needs native integration (GPT-4o style) for real-time agent communication.

---

## NeoTrix Transfer Priority Matrix

### P0 — Immediate Absorption (this cycle)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| Dual thinking modes | Qwen 3 / DeepSeek V4.1 | SEAL Pipeline adaptive depth | Universal pattern, directly applicable |
| Cost-aware routing | All models | GWT salience + cost weight | Axiom A1 confirmed by 10/10 models |
| MoE expert routing | Gemini / Llama / Qwen | E8 Hexagram expert selection | Fundamental architecture shift |

### P1 — Short-Term Integration (1-2 cycles)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| Paged KV for 1M+ context | Gemini / DeepSeek / Llama | KVMem integration | Context explosion requires it |
| Early fusion multimodal | GPT-4o / Llama 4 | PerceptionBridge upgrade | Unified modality processing |
| Prompt caching 90% | Claude 3.5 | KVCache tiering | Cost reduction, already proven |
| CoT distillation | Phi-4-reasoning | Skill crystallization pipeline | Small-model efficiency |

### P2 — Medium-Term Research (2-4 cycles)
| Innovation | Source | Target | Rationale |
|-----------|--------|--------|-----------|
| 10M+ context sessions | Llama 4 Scout | KVMem + experience-tree | Ultra-long session support |
| 4-stage post-training | Qwen 3 | SEAL Pipeline stages | Skill training pipeline refinement |
| Real-time social integration | Grok 3 | UnifiedCrawler + KB | Live knowledge acquisition |
| Budget-controlled reasoning | Qwen 3 | AttentionManager | Smooth compute allocation |

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MoE efficiency vs Dense quality (Claude 3.5) | Both viable: Dense for speed-critical, MoE for capacity-critical. NeoTrix routes accordingly. |
| Thinking mode latency vs accuracy | Budget-aware: short tasks → non-thinking, long tasks → thinking. Adaptive SEAL depth. |
| Open-weight (Llama/Qwen) vs Proprietary (GPT/Claude) | Ordered Backend Router: open-weight for local/sensitive, proprietary for frontier capability |
| 10M context vs KV cost | Tiered storage: hot 128K in GPU, warm 1M in host, cold 10M on NVMe. KVMem paged virtualization. |
| Small model reasoning (Phi-4 14B) vs Frontier (GPT-4o ~200B) | Cost-aware delegation: Phi-4 for edge reasoning, frontier for complex multi-step tasks |
