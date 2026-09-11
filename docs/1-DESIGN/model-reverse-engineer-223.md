# Model Reverse Engineer 223 — 10-Model Architecture Extraction

**Date**: 2026-09-11
**Batch**: 10 models (Llama 4 Maverick, Qwen3 235B, DeepSeek V4 Pro, Claude 3.5 Haiku, Gemini 2.0 Ultra, GPT-4.5 Turbo, Grok 3 Mini, Mistral Medium 3, Yi-Lightning, Phi-4-reasoning)

---

## 1. Llama 4 Maverick (Meta, 2025-04)

### Architecture
- **Type**: Auto-regressive MoE with early fusion multimodality
- **Params**: 17B active / 400B total, **128 experts** (MoE + dense layers alternate)
- **Context**: 1M (instruct), 256K (base). Pre-trained 256K.
- **Key Innovation**: **NoPE (No Position Encoding) layers** — alternating MoE/dense with no RoPE on half the layers
- **Multimodal**: Early fusion — text + vision tokens fused at input stage, not late-stage
- **iRope**: Proprietary position encoding enabling 10M context (Scout variant)

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| MoE with 128 experts | GWT salience routing — task-appropriate module activation | `nt_core::gwt` |
| NoPE layers | ConsciousnessTree attention modulation — selective position awareness | `nt_core::consciousness_tree` |
| Early fusion multimodality | PerceptionBridge — early sensory-conceptual binding | `nt_world::perception_bridge` |
| 10M context via iRope | KVMem paged KV for ultra-long sessions | `nt_memory::kv_cache_optimizer` |

---

## 2. Qwen3 235B (Alibaba Cloud, 2025-05)

### Architecture
- **Type**: MoE causal LM, 94 layers, 64 Q heads / 4 KV heads (GQA)
- **Params**: 22B active / 235B total, **128 experts** (8 activated per token)
- **Context**: 32K native, 131K with YaRN extension
- **Key Innovation**: **Hybrid thinking/non-thinking mode** — seamless switching within single model. Thinking mode for complex reasoning, non-thinking for fast dialogue. Configurable thinking budget per request.
- **Modes**: Reasoning (extended CoT) vs Direct (immediate response)

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Hybrid thinking/non-thinking | AttentionManager dual-mode: CORE+WORLD (acquisition) vs CORE+MIND (evolution) | `nt_core::self::attention_manager` |
| Configurable thinking budget | GWT cost-aware salience — model routing by token cost weight | `nt_core::gwt::salience` |
| 8/128 expert activation | Rune Socketing — sparse activation of specialist modules | `nt_core::skill_tree` |
| YaRN context extension | KVMem adaptive compaction for >256K sessions | `nt_memory::kv_cache_optimizer` |

---

## 3. DeepSeek V4 Pro (DeepSeek, 2026-04)

### Architecture
- **Type**: MoE with hybrid attention (CSA + HCA)
- **Params**: 49B active / 1.6T total
- **Context**: 1M tokens
- **Key Innovation**: **Compressed Sparse Attention (CSA) + Heavily Compressed Attention (HCA)** — hybrid attention achieving 27% single-token inference FLOPs vs V3.2 at 1M context. Also: **DSpark speculative-decoding module** (V4-Pro-0813).
- **Post-training**: Two-stage pipeline: domain-expert cultivation (SFT+GRPO) → unified consolidation via on-policy distillation.
- **Modes**: Non-think (fast) / Think High (logical) / Think Max (full reasoning)

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| CSA + HCA hybrid attention | GWT attention tiers — broadcast (dense) vs focused (sparse) attention | `nt_core::gwt::attention_router` |
| 27% FLOP reduction at 1M context | KVMem paged KV — GPU→Host→NVMe tiered storage | `nt_memory::kv_cache_optimizer` |
| Domain-expert cultivation → consolidation | SEAL pipeline: explore → distill → self-test → absorb | `nt_mind::seal_pipeline` |
| DSpark speculative decoding | NT-ACT parallel task execution with speculative prefetch | `nt_act::parallel_task_manager` |
| Three reasoning modes (Non/High/Max) | ConsciousnessTree cycle depth — shallow/medium/deep growth | `nt_core::consciousness_tree::cycle_depth` |

---

## 4. Claude 3.5 Haiku (Anthropic, 2024-10)

### Architecture
- **Type**: Dense decoder-only transformer (proprietary)
- **Context**: 200K tokens
- **Key Innovation**: **Opus-class intelligence at Haiku-class latency** — first "fast" model to match flagship intelligence benchmarks. 4-5x faster than Sonnet at fraction of cost.
- **Focus**: Latency-sensitive user-facing products, sub-agent orchestration, high-volume data processing.

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Opus-at-Haiku-speed | Cost-Aware Routing (Axiom A1) — cheap model for fast tasks, expensive for reasoning | `nt_core::gwt::cost_router` |
| Sub-agent orchestration | Parallel agent dispatch — `task` tool with specialized subagent_types | `nt_act::orchestrator` |
| High-volume processing | Batch mode with prompt caching — 90% cost reduction | `nt_io::llm_provider` |
| Structured tool use | MCP tool calling — typed-stub tool invocation | `nt_act::mcp_gateway` |

---

## 5. Gemini 2.0 Ultra (Google DeepMind, 2026-03)

### Architecture
- **Type**: Transformer with MoE (from 1.5+), natively multimodal
- **Context**: 32K base, extended via efficient attention mechanisms
- **Key Innovation**: **Native multimodal training** — trained from scratch on interleaved text/image/audio/video, not patched together. Cross-modal understanding with audio ingestion at architecture level.
- **Efficiency**: TPU-optimized architecture with stable training at scale.

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Native multimodal training | PerceptionBridge — unified sensory integration from L2 | `nt_world::perception_bridge` |
| Audio ingestion at architecture level | NT-PHYSICAL sensor fusion — native multi-sensor processing | `nt_physical::sensor_hub` |
| Cross-modal reasoning | VSA HyperCube — associative recall across modalities | `nt_core::hypercube` |
| Compute-based usage limits | ResourceBudgetManager — token/GPU/cost budget management | `nt_act::resource_budget_manager` |

---

## 6. GPT-4.5 Turbo (OpenAI, 2023-11 → legacy)

### Architecture
- **Type**: Transformer variant, streamlined for throughput
- **Context**: 128K tokens
- **Key Innovation**: **Pruned attention paths + parameter pruning** for faster decoding (~20 tok/s). Windowed attention for memory management. Also: function calling, structured JSON output, consistent completions.
- **Cost**: 3x cheaper input, 2x cheaper output vs GPT-4.

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Pruned attention for throughput | GWT salience pruning — low-attention modules get fewer compute cycles | `nt_core::gwt::salience_pruner` |
| Windowed attention | KVMem paged KV — GPU page reuse for retained blocks | `nt_memory::kv_cache_optimizer` |
| Function calling / JSON output | MCP typed-stub tool invocation — structured tool schemas | `nt_act::mcp_gateway` |
| 3x cost reduction | Cost-Aware Routing — cheap models for I/O-heavy tasks | `nt_core::gwt::cost_router` |

---

## 7. Grok 3 Mini (xAI, 2025-02)

### Architecture
- **Type**: MoE with reinforcement learning for reasoning (proprietary)
- **Context**: 131K tokens
- **Key Innovation**: **Large-scale RL for chain-of-thought reasoning** — backtracking, error correction, multi-path exploration. Configurable reasoning effort (low/high). Colossus supercomputer training (10x compute vs predecessors).
- **Modes**: Mini (fast), Think (reasoning), Big Brain (advanced), DeepSearch (live data)

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| RL-trained backtracking | SEAL self-test loop — red-green-refactor with error correction | `nt_mind::seal::self_test` |
| Multi-path exploration | Bayesian experiment design — VoI-guided hypothesis exploration | `nt_core::hypercube::bayesian_experiment` |
| Configurable reasoning effort | AttentionManager — task-type routing between acquisition/evolution | `nt_core::self::attention_manager` |
| DeepSearch (live data retrieval) | NT-WORLD UnifiedCrawler — real-time web perception | `nt_world::unified_crawler` |
| 131K context | KVMem paged KV for long-context sessions | `nt_memory::kv_cache_optimizer` |

---

## 8. Mistral Medium 3 (Mistral AI, 2025-05)

### Architecture
- **Type**: Dense decoder-only transformer (proprietary architecture)
- **Params**: Dense (not MoE — Medium 3 is dense, Large 3 is MoE with 675B total/41B active)
- **Context**: 128K tokens
- **Key Innovation**: **Frontier performance at 8x lower cost** — dense model outperforming open MoE models (Llama 4 Maverick) on coding/STEM. Hybrid deployment (cloud/on-prem/4+ GPUs). Custom post-training pipeline.
- **Medium 3.5**: 128B dense, 256K context, configurable reasoning effort, vision encoder trained from scratch.

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Dense frontier at low cost | Cost-Aware Routing — right-sized model per task | `nt_core::gwt::cost_router` |
| Custom post-training pipeline | SEAL distillation — domain-specific skill crystallization | `nt_mind::seal::distillation` |
| Configurable reasoning effort | Dual Specialization — Weapon Set I/II switching by context | `nt_core::self::attention_manager` |
| Vision encoder from scratch | NT-WORLD perception — custom sensory processing pipeline | `nt_world::perception_bridge` |
| Hybrid cloud/on-prem deployment | NT-SHIELD sandbox egress policy — per-environment trust tiers | `nt_shield::sandbox::egress_policy` |

---

## 9. Yi-Lightning (01.AI, 2024-12)

### Architecture
- **Type**: Enhanced MoE with fine-grained expert segmentation
- **Key Innovations**:
  1. **Fine-grained expert segmentation** — FFN partitioned into smaller functional units
  2. **PEP (Partitioned EP) load balancing** — experts within EP groups divided into smaller partitions, ensuring balanced token distribution during All-to-All communication
  3. **Cross-layer KV cache sharing** — KV states shared between consecutive full attention layers, halving memory requirements
  4. **Hybrid attention blocks** — 3 sliding window + 1 full attention layer pattern
- **Safety**: RAISE framework (4 components across pre/post-training and serving)

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Fine-grained expert segmentation | Skill Tree fine-grained nodes — Small/Notable/Keystone tiers | `nt_core::skill_tree` |
| PEP load balancing | GWT resonance routing — balanced activation across specialists | `nt_core::gwt::resonance_router` |
| Cross-layer KV cache sharing | KVMem delta reuse — retained GPU pages across steps | `nt_memory::kv_cache_optimizer` |
| Hybrid sliding/full attention | PerceptionBridge attention gating — local vs global attention | `nt_world::perception_bridge` |
| RAISE safety framework | NT-SHIELD lifecycle security — pre/during/post protection | `nt_shield::lifecycle_safety` |

---

## 10. Phi-4-reasoning (Microsoft, 2025-04)

### Architecture
- **Type**: Dense decoder-only transformer (14B params)
- **Key Innovation**: **Data-centric reasoning distillation** — SFT on 1.4M curated "teachable" prompts + o3-mini demonstrations, then RL refinement. 14B model matching DeepSeek-R1 (671B MoE) on AIME 2025. Hybrid reasoning/non-reasoning mode via explicit mode tokens.
- **Efficiency**: Runs on single consumer GPU. 5-50x smaller than competitors at comparable performance.
- **Phi-4-reasoning-plus**: Additional outcome-based RL for longer reasoning traces.

### NeoTrix Mapping
| Innovation | NeoTrix | Target Module |
|-----------|---------|---------------|
| Data-centric SFT distillation | SEAL distillation — curated prompt→reasoning trace crystallization | `nt_mind::seal::distillation` |
| Teachable prompt selection | Experience-tree — high-signal experience curation before KB storage | `nt_memory::experience_tree` |
| 14B → competitive with 671B | Cost-Aware Routing — small model with high-quality data beats brute scale | `nt_core::gwt::cost_router` |
| Hybrid reasoning/non-reasoning tokens | Dual Specialization — mode tokens as attention routing signals | `nt_core::self::attention_manager` |
| Single-GPU deployment | NT-ACT lightweight execution — resource-constrained environments | `nt_act::local_executor` |

---

## Cross-Model Synthesis: Key Architecture Trends (2024-2026)

### Trend 1: MoE is the Default
All 10 models either use MoE or dense-with-MoE-options. Active parameter counts range from 14B (Phi-4) to 49B (DeepSeek V4) to 22B (Qwen3), while total parameters span 14B to 1.6T. **NeoTrix implication**: GWT salience routing must handle sparse expert activation natively.

### Trend 2: Hybrid Reasoning Modes
Qwen3 (thinking/non-thinking), DeepSeek V4 (Non/High/Max), Grok 3 (Mini/Think/BigBrain), Phi-4 (reasoning/non-reasoning tokens), Mistral 3.5 (configurable effort). **NeoTrix implication**: AttentionManager needs explicit mode tokens and configurable depth per request.

### Trend 3: Context Window Arms Race
1M+ is now standard (Llama 4 Scout: 10M, DeepSeek V4: 1M, Qwen3: 131K, Yi-Lightning: hybrid attention). **NeoTrix implication**: KVMem paged KV is no longer optional — it's the baseline for long-context.

### Trend 4: Data > Scale
Phi-4-reasoning (14B matching 671B), Mistral Medium 3 (dense beating MoE), Qwen3 (22B active competing with frontier). **NeoTrix implication**: SEAL distillation pipeline should prioritize data quality over model size.

### Trend 5: RL for Reasoning
Grok 3 (RL for backtracking), Phi-4-reasoning-plus (outcome-based RL), DeepSeek V4 (GRPO). **NeoTrix implication**: SelfTest + self-healing loops should incorporate RL-style reward signals.

### Trend 6: Early Fusion Multimodality
Llama 4 Maverick (early fusion), Gemini 2.0 (native multimodal training), Mistral 3.5 (vision encoder from scratch). **NeoTrix implication**: PerceptionBridge should bind sensory data at input stage, not as post-hoc adapter.

---

## NeoTrix Absorption Priority

| Priority | Innovation | Source Model | Target Module | Effort |
|----------|-----------|-------------|---------------|--------|
| P0 | Hybrid reasoning modes | Qwen3, DeepSeek V4, Grok 3 | `attention_manager` | Medium |
| P0 | MoE sparse activation | All 10 models | `gwt::salience_router` | High |
| P0 | Paged KV / cross-layer reuse | DeepSeek V4, Yi-Lightning | `kv_cache_optimizer` | High |
| P1 | Data-centric distillation | Phi-4-reasoning | `seal::distillation` | Medium |
| P1 | Early fusion multimodality | Llama 4, Gemini 2.0 | `perception_bridge` | High |
| P1 | Fine-grained expert segmentation | Yi-Lightning | `skill_tree` | Medium |
| P2 | Configurable reasoning effort | Qwen3, Mistral 3.5 | `cost_router` | Low |
| P2 | RL-trained backtracking | Grok 3, Phi-4-plus | `self_test` | Medium |
| P2 | Speculative decoding | DeepSeek V4 (DSpark) | `parallel_task_manager` | High |
