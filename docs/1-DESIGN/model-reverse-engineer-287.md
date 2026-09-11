# Model Reverse Engineering #287 — 10-State-of-the-Art LLM Architecture Extraction

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

### Key Innovation: **End-to-End Omni-Modal Training**
- Single model trained jointly on text, audio, image, video — no pipeline chaining
- Eliminated the ASR→LLM→TTS pipeline: 320ms audio latency (vs 5.4s previously, 16x improvement)
- Native cross-modal reasoning without modality-specific adapters

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Omni-modal training | `nt_world_sense::PerceptionBridge` | Extend PerceptionBridge to handle unified audio-visual-text embeddings; use E8 attention gating for cross-modal salience |
| 320ms latency target | `nt_io::LlmRouter` | Implement GWT salience-based routing with latency SLA per modality tier |
| Pipeline elimination | `nt_act::ProductionOrchestrator` | Collapse multi-step agent pipelines into single-pass calls where possible |

---

## 2. Claude 3.5 Sonnet (Anthropic)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | 200K |
| Modality | Text + vision |
| Training | Constitutional AI + RLHF + synthetic data |
| Speed | 2x Claude 3 Opus |
| Cost | $3/$15 per M tokens |

### Key Innovation: **Constitutional AI + Architectural Tweaks for Speed**
- Safety alignment baked into training via constitutional principles (not post-hoc filtering)
- Architecture optimized for the speed-intelligence Pareto frontier: "mid-tier cost, flagship intelligence"
- Agentic coding: 64% SWE-bench pass rate (vs Opus 38%) via tool-use fine-tuning
- Artifacts: structured output workspace (not just text generation)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Constitutional AI | `nt_shield::EgressPrivacyGuard` | Extend guard to Constitutional-style principle injection at inference time |
| Speed-intelligence Parefo | `nt_core_self::AttentionManager` | Dual-weapon-set routing (Ascendancy) with cost-latency-aware switching |
| Agentic tool-use training | `nt_act::McpToolRegistry` | Fine-tune tool-calling schemas; structured artifact output as first-class |
| Artifacts workspace | `nt_io::ACP` (Agent Communication Protocol) | Structured output containers with live editing state |

---

## 3. Gemini 2.5 Pro (Google)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Context | 1M tokens |
| Modality | Text + image + audio + video |
| Training | TPUv5p, synchronous data-parallel |
| Key Feature | "Thinking" (reasoning trace) |
| Output | 65K tokens |

### Key Innovation: **1M-Token Context with Native Thinking + TPUv5p Training**
- Sparse MoE Transformer (inherited from Gemini 1.5 architecture lineage)
- Native "thinking" mode: internal chain-of-thought before response
- Thinking budgets: developer-controlled compute allocation
- 1M context with improved vision: 3-hour video understanding, video-to-code conversion
- TPUv5p multi-datacenter training at pod scale (8960-chip pods)

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| 1M context | `nt_memory::KB` + `kv_cache_optimizer` | Extend KB embedding pipeline with hierarchical context compression (inspired by Gemini's long-context approach) |
| Native thinking | `nt_core::ConsciousnessTree` | Map Gemini's thinking traces to ConsciousnessTree's 6-stage feedback loop; thinking budget = growth cycle budget |
| Thinking budgets | `nt_core_self::DynamicParams` | Expose `thinking_budget` as a DynamicParams (speed/amplitude/frequency) for caller-controlled compute |
| Multi-datacenter training | `nt_physical::ParallelTaskManager` | Extend with TPU-style pod-aware scheduling |
| Video-to-code | `nt_world::NarrativeStructuring` | Add video frame extraction → structured narrative → code generation pipeline |

---

## 4. Llama 4 Scout (Meta)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 17B active / 109B total |
| Experts | 16 |
| Context | 10M tokens (Scout) / 1M (Maverick) |
| Training | ~40T tokens |
| Modality | Native multimodal (early fusion) |

### Key Innovation: **10M-Token Context via MoE + Early Fusion**
- MoE architecture: 16 experts, only 17B active per token
- Early fusion: multimodal tokens fused at input embedding level (not late fusion adapters)
- 10M token context: longest publicly claimed context window
- Efficient single-GPU deployment via int4 quantization (fits 1x H100)
- iRoPE (interleaved RoPE) for position encoding at extreme lengths

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| MoE with 16 experts | `nt_core::E8 Hexagram` | Map expert routing to E8 hexagram state transitions; each hexagram = expert configuration |
| Early fusion multimodal | `nt_world_sense::SensoryIntegrationHub` | Fuse modalities at embedding level before GWT attention routing |
| 10M context | `nt_memory::KB` + `nt_nexus::SessionBridge` | Hierarchical context: hot (GPU KV) → warm (SSD) → cold (KB embeddings) with automatic tiering |
| iRoPE | `nt_core::SelfModule` | Implement positional encoding variant for ultra-long session memory |

---

## 5. DeepSeek V4.1 Flash (DeepSeek)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 552B total / 8B input-active / 16B output-active |
| Context | 1M tokens |
| Modality | Native multimodal vision |
| Architecture | Causal Encoder-Decoder (CED) + CSA2 + Engram |
| KV Cache | 890 bytes/token (1/4 of V4-Flash) |

### Key Innovation: **Causal Encoder-Decoder + KV Cache Compression to 890 bytes/token**
- **CED Architecture**: 20-layer encoder + 20-layer decoder. Prefill only computes encoder; decoder KV is projected from encoder final state. Nearly halves prefill FLOPs.
- **CSA2 (Compressed Sparse Attention 2)**: Three modes (Full/Reindex/Reuse). Cross-layer KV reuse. Compression ratios 1 and 2.
- **Engram Memory**: 196B-parameter conditional memory module. Hash-table lookup (384M rows × 256 dims). Written into residual stream via learned gate.
- **SWA Bounded Replay**: Persistent KV cache reduced to 1/8 of previous gen by replaying only recent window tokens.
- **FP4 KV Caching**: MXFP4 format for main KV cache in HBM.
- **Controllable Reasoning Effort**: Scalar 1-100 in system prompt, exponential token penalty during RL.
- **mHC (Manifold-Constrained Hyper-Connections)**: Replaces residual stream. 4x wider inter-layer pathways, doubly-stochastic mixing matrix.

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| CED architecture | `nt_core::E8 Hexagram` | Encoder = perception branch (L1-L2 layers), Decoder = generation branch (L3-L6 layers). KV projection across consciousness layers. |
| CSA2 three-mode attention | `nt_core_gwt::SelectiveState` | Map Full/Reindex/Reuse to GWT attention modes: broadcast (Full), focus (Reindex), sustain (Reuse) |
| Engram memory | `nt_memory::KB` + `nt_nexus::CrossSessionMemory` | KB as hash-table: key=n-gram hash, value=experience embedding. Learned gate = attention-based retrieval weighting |
| SWA Bounded Replay | `nt_memory::SessionBridge` | For session resume: replay last N turns instead of full context. Maps to experience-tree lazy branch loading |
| FP4 KV caching | `nt_memory::KB` + `kv_cache_optimizer` | Implement 4-bit quantized KB embeddings with QAT-aware training |
| Controllable reasoning effort | `nt_core_self::DynamicParams` | `effort_level` parameter (1-100) controlling ConsciousnessTree cycle depth |
| mHC replacing residual | `l5_cognition::traits` | Replace simple residual connections between cognitive layers with multi-pathway doubly-stochastic mixing |

---

## 6. Qwen 3 (Alibaba)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 0.6B to 235B (dense) / 30B-3B-active, 235B-22B-active (MoE) |
| Context | 131K |
| Modality | Text (dense), multimodal (MoE) |
| Key Feature | Hybrid thinking/non-thinking mode |
| Training | 36T tokens, 4-stage process |

### Key Innovation: **Hybrid Thinking/Non-Thinking Mode + Ultra-Sparse MoE**
- **Hybrid Reasoning**: Seamlessly switches between thinking mode (deep CoT) and non-thinking mode (fast response). API control via thinking duration (up to 38K tokens).
- **4-Stage Training**: (1) Long CoT cold start, (2) Reasoning RL, (3) Thinking mode fusion, (4) General RL
- **Ultra-Sparse MoE** (Qwen3-Next): 80B params, only 3B activated (3.7%). Hybrid attention: Gated DeltaNet + Gated Attention.
- **Multi-Token Prediction (MTP)**: Predicts multiple future tokens per step, boosting both training efficiency and inference speed.
- **MCP-native**: Natively supports Model Context Protocol for tool calling.

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Hybrid thinking/non-thinking | `nt_core_self::AttentionManager` | Map to Ascendancy dual-weapon-set: Thinking mode = CORE+WORLD (acquisition), Non-thinking = CORE+MIND (evolution) |
| 4-stage training | `seal::Pipeline` | Map SEAL stages to Qwen's training: Soil→Roots (CoT cold start), Trunk (reasoning RL), Branches (mode fusion), Fruits (general RL) |
| Ultra-sparse MoE | `nt_core::E8 Hexagram` | Sparse expert activation: only ~4% of Hexagram states active per token. Maps to Constellation maturity gating. |
| Multi-Token Prediction | `nt_mind::SkillEngine` | Speculative execution: predict next 2-3 skill transitions, verify with small cost |
| MCP-native | `nt_act::McpToolRegistry` | Already implemented; strengthen with Qwen's structured function-calling schemas |

---

## 7. Mistral Large 3 (Mistral AI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 41B active / 675B total |
| Experts | 128 per layer |
| Context | 256K |
| Modality | Multimodal (text + image) |
| Architecture | Granular MoE + Vision Encoder (673B LM + 2.5B vision) |
| License | Apache 2.0 |

### Key Innovation: **Granular MoE with 128 Experts + Multi-Latent Attention**
- **Granular MoE**: 128 experts per layer (vs typical 8-16). Top-4 softmax routing. Fewer, larger experts than DeepSeekV3-style.
- **Multi-Latent Attention (MLA)**: Inherited from DeepSeek lineage, compresses KV cache into low-rank latent.
- **Llama 4 RoPE Scaling**: Cross-pollination of position encoding techniques.
- **Speculative Decoding**: Eagle draft model for fast token generation.
- **Edge Deployment**: Ministral 3 series (3B/8B/14B) for on-device with vision.

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| 128-expert granularity | `nt_core::E8 Hexagram` | 64 hexagrams × 2 states = 128 micro-states. Granular routing via E8 adjacency matrix. |
| MLA KV compression | `nt_memory::KB` + `kv_cache_optimizer` | Low-rank KV latent compression for KB embedding storage. Already partially implemented in mHC design. |
| Speculative decoding | `nt_mind::SkillEngine` | Draft-verify pattern: small model predicts next skill, large model verifies. Maps to SEAL speculative execution. |
| Edge deployment | `nt_physical::BodySchema` | Lightweight model variants for edge/body deployment (sensor processing, local reasoning) |

---

## 8. Phi-4 Reasoning (Microsoft)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 14B |
| Context | 32K (extensible) |
| Architecture | Dense decoder-only Transformer |
| Key Feature | Reasoning via synthetic SFT + RL |
| Training | 16B tokens SFT, 32x H100, 2.5 days |

### Key Innovation: **Small Model Reasoning via "Teachable" Prompt Curation + Synthetic Traces**
- **Teachable Prompt Selection**: Curated prompts with "right level of complexity and diversity" — not random SFT data
- **Synthetic Reasoning Traces**: Generated by o3-mini, then used for SFT. The small model learns to mimic frontier model reasoning chains.
- **Phi-4-reasoning-plus**: Short RL phase after SFT, producing longer reasoning traces (1.5x more tokens, higher accuracy)
- **Pivotal Token Search**: Novel DPO pair construction via identifying tokens that change outcome
- **Outperforms 5-50x larger models** on math, science, coding tasks

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Teachable prompt curation | `nt_mind::SkillEngine` | Curate "teachable" skill-training examples: not random, but at optimal complexity frontier |
| Synthetic reasoning traces | `nt_mind::Distillation` | Use frontier model traces (o3-mini equivalent) to train local reasoning chains. Already partially in SEAL distillation. |
| Pivotal token search | `nt_meta::CrossModuleAudit` | Identify "pivotal" decisions in session history that changed outcome. Weight these in experience absorption. |
| Small-model-beats-large | `nt_core::E8 Hexagram` | Validate: can 14B-class local models handle NeoTrix reasoning tasks? Route simple tasks to small models (Axiom A1: Cost-Aware Routing) |

---

## 9. Yi-Lightning (01.AI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Params | 100B MoE |
| Context | 128K |
| Architecture | Enhanced MoE with fine-grained segmentation |
| Key Feature | Chatbot Arena #6 overall, #1 in China |
| License | Proprietary |

### Key Innovation: **Fine-Grained Expert Segmentation + Cross-Layer KV Cache Sharing**
- **Fine-Grained Expert Segmentation**: Experts split into smaller sub-experts, enabling more granular routing decisions
- **Balanced Expert Routing**: Load-balancing strategy that prevents expert collapse (some experts never used)
- **Cross-Layer KV Cache Sharing**: KV states shared across adjacent layers, reducing memory without quality loss
- **FP8 Hardware Co-Design**: Architecture designed specifically for FP8 quantization compatibility
- **RAISE Safety Framework**: 4-component safety across pre-training, post-training, serving

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Fine-grained expert segmentation | `nt_core::E8 Hexagram` | Subdivide each hexagram into micro-states. 64 hexagrams × 4 micro-states = 256 routing options. |
| Balanced expert routing | `nt_core_self::AttentionManager` | Ensure E8 states are evenly visited (prevent "dark forest" collapse where some states never activate) |
| Cross-layer KV sharing | `nt_memory::KB` | Share embeddings across related KB namespaces (e.g., experience + domain share KV in GWT attention) |
| FP8 co-design | `nt_physical::ParallelTaskManager` | Design NeoTrix inference pipeline for FP4/FP8 hardware-native execution |
| RAISE safety | `nt_shield::EgressPrivacyGuard` | Extend with 4-component safety: input filtering, training alignment, output guard, serving audit |

---

## 10. Grok 3 (xAI)

### Architecture Facts
| Attribute | Value |
|-----------|-------|
| Training | 100K H100 GPUs (200K post-upgrade), 200M GPU-hours |
| Context | 131K |
| Modality | Text + vision (image gen via separate model) |
| Key Feature | Think / Big Brain / DeepSearch modes |
| Architecture | Hybrid dense/MoE (undisclosed details) |

### Key Innovation: **Multi-Tier Reasoning Effort + Massive Compute Scaling**
- **Three Reasoning Tiers**: (1) Think — deep reasoning, (2) Big Brain — extra compute, (3) DeepSearch — web research agent
- **TTCS (Test-Time Compute at Scale)**: Dynamically allocate more inference compute for harder problems
- **Partial Chain-of-Thought Visibility**: Shows some reasoning tokens, hides others (anti-distillation)
- **DeepSearch Agent**: Web search + report compilation as a first-class model capability
- **Colossus Supercomputer**: 122-day build, 200K H100s — brute-force compute scaling

### NeoTrix Mapping
| Innovation | NeoTrix Component | Transfer Method |
|-----------|-------------------|-----------------|
| Three reasoning tiers | `nt_core_self::DynamicParams` | Map to 3 Ascendancy modes: Think (CORE+WORLD), Big Brain (CORE+MIND deep), DeepSearch (NT-WORLD crawl) |
| TTCS | `nt_core::ConsciousnessTree` | Dynamic cycle depth: easy tasks = 1 cycle, hard tasks = up to 3 cycles (current max). Already partially implemented. |
| Partial CoT visibility | `nt_shield::EgressPrivacyGuard` | Selective reasoning trace disclosure: show high-level reasoning to user, keep detailed traces internal |
| DeepSearch agent | `nt_world::UnifiedCrawler` | Already implemented as crawl pipeline. Strengthen with Grok's multi-query follow-up pattern. |
| Compute scaling | `nt_physical::ParallelTaskManager` | Design for massive GPU cluster调度 (100K+ class). Current ParallelTaskManager handles multi-GPU but not pod-scale. |

---

## Cross-Model Synthesis: 12 Transferable Architecture Patterns

### Pattern 1: **Hybrid Attention / MoE is Universal**
All 10 models use some form of MoE or sparse activation. The era of dense-only is over.
- **NeoTrix Impact**: E8 Hexagram must support sparse state activation. Only ~4-10% of states active per token.

### Pattern 2: **KV Cache Compression is the Bottleneck Frontier**
DeepSeek (890 bytes/token), Yi-Lightning (cross-layer sharing), Gemini (long-context optimization) all prioritize KV efficiency.
- **NeoTrix Impact**: `kv_cache_optimizer.rs` is critical path. Implement FP4 KV, cross-namespace sharing, SWA replay.

### Pattern 3: **Thinking Budgets / Controllable Compute**
Gemini (thinking budgets), Qwen3 (thinking duration), DeepSeek (effort 1-100), Grok (3 tiers) — all expose compute control.
- **NeoTrix Impact**: `DynamicParams.effort_level` must be a first-class parameter routed through GWT salience.

### Pattern 4: **End-to-End Multimodal > Pipeline Chaining**
GPT-4o (16x latency reduction), Llama 4 (early fusion), Gemini (native audio) — pipelines are dead.
- **NeoTrix Impact**: `PerceptionBridge` must fuse modalities at embedding level, not post-processing.

### Pattern 5: **Small Models with Large-Model Reasoning**
Phi-4 (14B beats 70B), Qwen3-Next (3.7% activation), Mistral Ministral (3B-14B edge) — size is not destiny.
- **NeoTrix Impact**: Validate Axiom A1 — route simple tasks to small local models, complex to frontier. Cost-aware routing.

### Pattern 6: **Engram / Hash-Table Memory is the New Paradigm**
DeepSeek's Engram (384M-row hash table), KB's BM25 index — conditional memory via lookup is replacing full attention.
- **NeoTrix Impact**: `nt_memory::KB` namespace retrieval is the Engram equivalent. Strengthen with learned gating.

### Pattern 7: **Manifold-Constrained Connections Replace Residual Streams**
DeepSeek's mHC (4x wider, doubly-stochastic) keeps 61-layer models stable. Residual streams are insufficient for deep stacks.
- **NeoTrix Impact**: ConsciousnessTree 6-stage feedback loop uses inter-stage connections. Apply mHC-style mixing.

### Pattern 8: **Speculative Execution is Table Stakes**
Mistral (Eagle draft), Qwen3 (MTP), DeepSeek (controllable effort) — predict-then-verify is standard.
- **NeoTrix Impact**: `nt_mind::SkillEngine` should speculatively predict next skill transition, verify with small cost.

### Pattern 9: **Agentic Tool Use is a First-Class Training Objective**
Claude 3.5 (64% SWE-bench), Qwen3 (MCP-native), Grok 3 (DeepSearch) — agents aren't post-hoc, they're trained in.
- **NeoTrix Impact**: `nt_act::McpToolRegistry` integration should be a training-time objective, not runtime-only.

### Pattern 10: **Safety as Architecture, Not Filter**
Constitutional AI (Claude), RAISE (Yi), Egress Guard (NeoTrix) — safety must be structural, not bolt-on.
- **NeoTrix Impact**: Extend `nt_shield` with pre-training alignment principles, not just runtime filtering.

### Pattern 11: **Cross-Layer State Sharing**
Yi-Lightning (KV sharing), DeepSeek (CSA reuse), Qwen3-Next (Gated DeltaNet) — adjacent layers share computation.
- **NeoTrix Impact**: Layer traits (`l1_action/traits.rs` through `l6_meta/traits.rs`) should allow state projection across layers.

### Pattern 12: **Compute-Aware Model Routing**
GPT-4o (omni-modal routing), Grok (3-tier effort), NeoTrix Axiom A1 (Cost-Aware Routing) — not all tasks need the strongest model.
- **NeoTrix Impact**: GWT salience must include model-cost dimension. Cheap models for I/O, expensive for reasoning.

---

## NeoTrix Architecture Gap Analysis

| Gap | Severity | Closest Model Pattern | Recommended Action |
|-----|----------|----------------------|-------------------|
| No MoE/expert routing in E8 | HIGH | All 10 models | Implement sparse Hexagram activation |
| KV cache not FP4-quantized | HIGH | DeepSeek V4.1 | Extend `kv_cache_optimizer` to FP4 |
| No thinking budget control | MEDIUM | Gemini/Qwen3/DeepSeek | Add `effort_level` to DynamicParams |
| Pipeline chaining in agent flows | MEDIUM | GPT-4o/Llama 4 | Collapse multi-step to single-pass where possible |
| No cross-layer state sharing | MEDIUM | Yi-Lightning/DeepSeek | Add mHC-style mixing to layer traits |
| No speculative skill execution | LOW | Mistral/Qwen3 | Add draft-verify to SkillEngine |
| Edge model variants missing | LOW | Mistral Ministral | Design body-layer models for edge deployment |

---

## Priority Absorption Queue (R-P42/R-P79/R-P80)

| Priority | Pattern | Source Models | NeoTrix Target | Session Required |
|----------|---------|---------------|----------------|-----------------|
| P0 | KV Cache FP4 + SWA Replay | DeepSeek V4.1 | `kv_cache_optimizer` | Current |
| P0 | Controllable Reasoning Effort | DeepSeek/Gemini/Qwen3 | `DynamicParams.effort_level` | Current |
| P1 | Causal Encoder-Decoder | DeepSeek V4.1 | `E8 Hexagram` encoder-decoder split | Next |
| P1 | Fine-Grained Expert Segmentation | Yi-Lightning/Mistral | `E8 Hexagram` micro-states | Next |
| P2 | End-to-End Multimodal Fusion | GPT-4o/Llama 4 | `PerceptionBridge` fusion | Future |
| P2 | Speculative Skill Execution | Mistral/Qwen3 | `SkillEngine` draft-verify | Future |
| P3 | mHC Layer Connections | DeepSeek V4.1 | Layer traits mixing | Future |

---

*Generated: 2026-09-11 | Source: 10 LLM technical reports + community analysis | NeoTrix mapping: architecture team*
