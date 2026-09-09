# MiniMind Absorption Plan into NeoTrix Consciousness Kernel

## 1. Project Overview

**MiniMind** (https://github.com/jingyaogong/minimind) is an ultra-lightweight LLM training framework that trains a 64M-parameter language model from scratch in 2 hours for ~¥3. It provides complete end-to-end training pipelines covering Pretrain → SFT → LoRA → DPO → RLAIF (PPO/GRPO/CISPO) → Tool Use → Agentic RL → Adaptive Thinking → Model Distillation.

**Key Characteristics:**
- Pure PyTorch implementation, no third-party abstractions
- Qwen3/Qwen3-MoE architecture aligned
- MoE with 4 experts / top-1 routing
- JSONL data format throughout
- OpenAI-compatible API serving
- Adaptive thinking with `<think>` tags

**License:** Apache 2.0 (compatible with NeoTrix)

---

## 2. Transferable Patterns Analysis

### Pattern 1: MoE Auxiliary Load Balancing Loss

**MiniMind Implementation:**
- `MOEFeedForward` with router gate computing expert scores
- Auxiliary loss: `load * scores.mean(0)).sum() * num_experts * aux_loss_coef`
- Normalized top-k probabilities for stable training
- Balances expert utilization during training

**NeoTrix Applicability:**
- GWT attention routing currently uses resonance-based routing
- MoE load balancing can prevent attention collapse in specialist modules
- Could improve Constellation maturity progression by ensuring balanced capability development

**Target Domain:** NT-CORE (GWT attention routing enhancement)

---

### Pattern 2: DPO Preference Optimization

**MiniMind Implementation:**
- Reference model (frozen) vs policy model comparison
- Log-probability ratio computation for chosen vs rejected responses
- Beta-controlled divergence from reference
- Native PyTorch implementation, no RLHF framework dependency

**NeoTrix Applicability:**
- SEAL pipeline self-evaluation can use DPO-style preference learning
- ConsciousnessTree growth decisions can be optimized via preference signals
- Module selection in GWT can use preference-based routing

**Target Domain:** NT-MIND (SEAL pipeline self-evaluation)

---

### Pattern 3: Agentic RL for Tool Use

**MiniMind Implementation:**
- Multi-turn tool-use training with GRPO/CISPO
- Rollout engine decoupled from training loop
- OpenAI-style tool_call format in training data
- Adaptive thinking with reasoning content

**NeoTrix Applicability:**
- NT-ACT tool calling can benefit from RL-trained tool selection
- Multi-step task execution can use agentic RL for planning
- Tool use patterns can be learned from experience

**Target Domain:** NT-ACT (tool calling patterns)

---

### Pattern 4: Checkpoint Resume Pattern

**MiniMind Implementation:**
- Auto-detect and resume from checkpoints
- Cross-GPU recovery (automatic step adjustment)
- Wandb/SwanLab run continuity
- State dict includes model, optimizer, scaler, epoch, step

**NeoTrix Applicability:**
- KB persistence layer can adopt checkpoint pattern
- SEAL pipeline stages can be checkpointed
- Cross-session state recovery

**Target Domain:** NT-MEMORY (state persistence)

---

### Pattern 5: Minimalist Architecture Principle

**MiniMind Implementation:**
- 64M parameters (1/2700 of GPT-3)
- Dim=768, n_layers=8 optimized for small models
- MobileLLM research: deeper > wider for small models
- Vocabulary compression (6400 tokens) for parameter efficiency

**NeoTrix Applicability:**
- Aligns with NeoTrix "大道至简" philosophy
- Module design should favor depth over width
- Capability nodes can be small but deep

**Target Domain:** Cross-cutting (architecture philosophy)

---

### Pattern 6: JSONL Data Format Standardization

**MiniMind Implementation:**
- Unified JSONL format for all training stages
- Pretrain: `{"text": "..."}`
- SFT: `{"conversations": [...]}`
- DPO: `{"chosen": [...], "rejected": [...]}`
- Tool call: OpenAI multi-turn message format

**NeoTrix Applicability:**
- KB experience storage can adopt JSONL format
- Cross-module data exchange standardized
- Training data pipeline for self-evolution

**Target Domain:** NT-MEMORY (KB data format)

---

### Pattern 7: Adaptive Thinking (<think> Tags)

**MiniMind Implementation:**
- `<think>` reasoning content in training data
- `open_thinking` flag for inference control
- Reasoning content streamed separately from response
- Enables chain-of-thought without explicit prompting

**NeoTrix Applicability:**
- ConsciousnessTree can emit reasoning traces
- GWT attention can expose reasoning paths
- Self-audit can use structured thinking

**Target Domain:** NT-META (meta-cognition reasoning traces)

---

### Pattern 8: Top-k / Top-p Sampling with Repetition Penalty

**MiniMind Implementation:**
- Temperature scaling for logits
- Top-k filtering with dynamic threshold
- Top-p (nucleus) sampling with cumulative probability
- Repetition penalty for seen tokens

**NeoTrix Applicability:**
- LLM provider inference optimization in NT-IO
- Model routing can use sampling parameters
- Response quality control

**Target Domain:** NT-IO (LLM inference)

---

## 3. Absorption Plan by Domain

### NT-CORE: MoE Load Balancing for GWT

**What to absorb:** MoE auxiliary loss mechanism for attention routing balance

**Files to create/modify:**
- `neotrix-core/src/l5_cognition/nt_core/gwt_router.rs` — Add load balancing loss
- `neotrix-core/src/l5_cognition/nt_core/gwt_attention.rs` — Expert utilization tracking

**Implementation approach:**
```
1. Add router gate to GWT attention modules
2. Compute auxiliary loss during training: load * scores.mean(0) * num_modules * coef
3. Normalize top-k probabilities for stable routing
4. Log expert utilization metrics for ConsciousnessTree
```

**Risk:** Low — extends existing GWT routing without breaking changes
**Mitigation:** GWT already has salience-based routing; this adds load balancing signal

---

### NT-MIND: DPO Self-Evaluation for SEAL

**What to absorb:** DPO preference optimization for self-improvement decisions

**Files to create/modify:**
- `neotrix-core/src/l5_cognition/nt_mind/seal/dpo_evaluator.rs` — DPO loss computation
- `neotrix-core/src/l5_cognition/nt_mind/seal/preference_store.rs` — Chosen/rejected pairs

**Implementation approach:**
```
1. Store SEAL stage outputs as chosen/rejected pairs
2. Compute log-probability ratios for preference learning
3. Use beta-controlled divergence from reference behavior
4. Update module selection based on preference signals
```

**Risk:** Medium — requires reference model (previous session state)
**Mitigation:** Use KB experience history as reference

---

### NT-ACT: Agentic RL Tool Patterns

**What to absorb:** Multi-turn tool-use training patterns and rollout engine

**Files to create/modify:**
- `neotrix-core/src/l1_action/nt_act/tool_rl_trainer.rs` — RL training for tool selection
- `neotrix-core/src/l1_action/nt_act/rollout_engine.rs` — Decoupled rollout

**Implementation approach:**
```
1. Record tool use sequences as trajectories
2. Compute rewards based on task completion
3. Use GRPO/CISPO for policy optimization
4. Decouple rollout from training for flexibility
```

**Risk:** Medium — tool use patterns are domain-specific
**Mitigation:** Start with simple tool sequences, expand gradually

---

### NT-MEMORY: Checkpoint Resume Pattern

**What to absorb:** Auto-resume and cross-session state recovery

**Files to create/modify:**
- `neotrix-core/src/l1_action/nt_memory/checkpoint.rs` — Checkpoint management
- `neotrix-core/src/l1_action/nt_memory/resume.rs` — Resume logic

**Implementation approach:**
```
1. Auto-save state at SEAL stage boundaries
2. Store model weights, optimizer state, step counts
3. Resume from last valid checkpoint
4. Support cross-session recovery via KB
```

**Risk:** Low — aligns with existing KB persistence
**Mitigation:** Use existing SQLite backend for checkpoint storage

---

### NT-META: Reasoning Trace Emission

**What to absorb:** `<think>` tag pattern for structured reasoning

**Files to create/modify:**
- `neotrix-core/src/l6_meta/nt_meta/reasoning_trace.rs` — Trace emission
- `neotrix-core/src/l6_meta/nt_meta/thinking_buffer.rs` — Buffer management

**Implementation approach:**
```
1. Emit reasoning traces during consciousness processing
2. Buffer thinking content separately from responses
3. Expose reasoning paths for audit and debugging
4. Support adaptive thinking with toggle
```

**Risk:** Low — extends existing meta-cognition
**Mitigation:** Traces are optional, don't affect core processing

---

### NT-IO: Inference Sampling Optimization

**What to absorb:** Top-k/top-p sampling with repetition penalty

**Files to create/modify:**
- `neotrix-core/src/l1_action/nt_io/sampling.rs` — Sampling parameters
- `neotrix-core/src/l1_action/nt_io/inference.rs` — Inference optimization

**Implementation approach:**
```
1. Add temperature, top-k, top-p parameters to inference
2. Implement repetition penalty for seen tokens
3. Support streaming with reasoning content separation
4. OpenAI-compatible API response format
```

**Risk:** Low — extends existing LLM provider interface
**Mitigation:** Parameters are optional, backward compatible

---

## 4. Priority Order

| Priority | Pattern | Domain | Effort | Impact |
|----------|---------|--------|--------|--------|
| P0 | MoE Load Balancing | NT-CORE | Medium | High — improves GWT routing |
| P1 | Checkpoint Resume | NT-MEMORY | Low | High — reliability |
| P1 | JSONL Data Format | NT-MEMORY | Low | Medium — standardization |
| P2 | DPO Self-Evaluation | NT-MIND | High | High — self-improvement |
| P2 | Reasoning Traces | NT-META | Medium | Medium — auditability |
| P3 | Agentic RL Tools | NT-ACT | High | Medium — tool learning |
| P3 | Sampling Optimization | NT-IO | Low | Low — inference quality |

---

## 5. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| MoE routing instability | Medium | Start with small num_experts, monitor aux_loss |
| DPO reference drift | Medium | Use KB history as stable reference |
| Tool RL sample inefficiency | Low | Start with simple tool sequences |
| Checkpoint storage overhead | Low | Compress state dict, periodic cleanup |
| Reasoning trace overhead | Low | Make traces optional, lazy emission |

---

## 6. NeoTrix-Specific Considerations

### R-P42 Compliance
All patterns strengthen existing nodes:
- MoE → GWT (existing node, not new module)
- DPO → SEAL (existing pipeline, not new pipeline)
- Checkpoint → KB (existing persistence, not new storage)

### R-P79 Compliance
Same-session wiring:
- Each pattern is wired to production path in implementation
- No dead code or deferred integration

### Shared Language Alignment
- Use NT-* domain names, not generic terms
- Distinguish "MoE routing" from "GWT attention routing"
- Use "DPO preference" not "RLHF training"

---

## 7. Next Steps

1. **Phase 1 (P0):** Implement MoE load balancing in GWT
2. **Phase 2 (P1):** Add checkpoint resume to KB
3. **Phase 3 (P2):** Implement DPO self-evaluation in SEAL
4. **Phase 4 (P3):** Add reasoning traces to meta-cognition

Each phase requires:
- Implementation in target domain
- Unit tests (C1 constellation)
- Integration with existing modules (C2)
- Production wiring verification (T3)
