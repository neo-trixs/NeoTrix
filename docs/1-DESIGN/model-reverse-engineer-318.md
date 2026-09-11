# Model Reverse Engineering — Cycle 318 (2026-09-11)

## 5 New Models/Papers to Reverse-Engineer

---

### 1. Tandem — Collaborative LLM-SLM Reasoning (ACL 2026 Findings)
**Paper**: arXiv:2604.23623
**Authors**: Fu et al. (ACL 2026 Findings)

#### Core Architecture
- **Two-tier collaboration**: Large LLM generates compact "critical reasoning insights" → Small SLM executes full reasoning with those insights
- **Cost-aware termination**: Sufficiency classifier determines when enough guidance has been accumulated; adaptively stops LLM generation
- **Cross-domain transfer**: Sufficiency classifier trained on one domain (math) transfers to others (code) without retraining
- **Result**: ~40% cost reduction vs standalone LLM, competitive performance

#### NeoTrix Domain Mapping
| Domain | Mapping |
|--------|---------|
| **NT-CORE** | GWT attention routing — LLM = "strategic coordinator" generating high-salience insights |
| **NT-MIND** | Distillation pattern — LLM output compressed to actionable signals |
| **NT-IO** | Cost-Aware Routing (Axiom A1) — route cheap tasks to SLM, hard tasks to LLM |
| **NT-MEMORY** | Sufficiency classifier as reusable skill node |

#### Fusion Opportunities
- **GWT salience + cost weight**: The sufficiency classifier is exactly a "sufficiency detector" — when GWT salience exceeds threshold, stop expensive model, switch to cheap model
- **NT-MIND distillation**: LLM insight generation → SLM execution = crystal distillation pattern
- **Dual Specialization**: LLM as Weapon Set I (acquisition), SLM as Weapon Set II (execution)
- **Action**: Implement `SufficiencyClassifier` in `nt_core_self` for adaptive model switching

---

### 2. DUET — Dual-Model Efficient Two-Stage Inference (arXiv:2605.01111)
**Paper**: arXiv:2605.01111
**Authors**: Chen et al. (2026)

#### Core Architecture
- **Two-stage decomposition**: Capable model produces "reasoning signal" → Lightweight model interprets signal to generate answer
- **Length-penalized joint training**: Forces capable model to transmit only sufficient information
- **Key insight**: Reasoning-intensive computation handled by capable model; non-reasoning components delegated to lightweight model
- **Result**: 60% output token savings on AIME/GPQA benchmarks

#### NeoTrix Domain Mapping
| Domain | Mapping |
|--------|---------|
| **NT-CORE** | E8 Hexagram reasoning — capable model = full hexagram analysis, lightweight = partial pattern match |
| **NT-IO** | Provider routing — heavy tasks → expensive model, light tasks → cheap model |
| **NT-ACT** | Task decomposition — separate reasoning from execution |
| **NT-SHIELD** | Reasoning signal as "capability envelope" — minimal info disclosure |

#### Fusion Opportunities
- **E8 reasoning decomposition**: Full hexagram (64 states) for hard problems, 3-line trigram for simple ones — test-time compute scaling
- **GWT broadcasting**: "Reasoning signal" = salient broadcast that lightweight modules can consume
- **Cost optimization**: Track reasoning complexity → auto-route to appropriate tier
- **Action**: Add complexity estimator to GWT salience scoring

---

### 3. CAT — Confidence-Adaptive Thinking (ACL 2026 Industry Track)
**Paper**: arXiv:2607.00862
**Authors**: Jiang et al. (ACL 2026 Industry Track)

#### Core Architecture
- **Self-certainty signals**: Model's intrinsic confidence modulates reasoning length
- **Overthinking prevention**: Confident responses compressed; uncertain ones get full deliberation
- **Preference optimization**: Confidence integrated into RLHF-style training
- **Key insight**: "Autonomously modulates reasoning lengths based on problem difficulty"
- **Result**: Consistently outperforms SOTA baselines on reasoning accuracy across multiple benchmarks

#### NeoTrix Domain Mapping
| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Phi (IIT integration score) — confidence = integration strength |
| **NT-FEEL** | Emotion regulation — confidence maps to Trust/Fear axis |
| **NT-MIND** | SEAL pipeline adaptive depth — easy tasks skip stages |
| **NT-REPAIR** | Self-healing — low confidence triggers repair path |

#### Fusion Opportunities
- **Emotion-driven compute allocation**: High Trust → compress reasoning (Joy path); high Fear → extend reasoning (Caution path)
- **SEAL pipeline adaptive**: Skip distillation stages when confidence high; full pipeline when low
- **GWT salience modulation**: Confidence as input to attention weighting — confident signals broadcast wider
- **Action**: Add confidence estimator to `nt_core_self::AttentionManager`

---

### 4. Sheaf-ADMM — Multi-Agent Coordination via Cellular Sheaves (ICML 2026)
**Paper**: arXiv:2605.31005
**Authors**: Seely, Cupiał, Jones (ICML 2026)

#### Core Architecture
- **Cellular sheaf coordination**: Inter-agent constraints specified by a sheaf — determines which aspects of neighboring solutions must agree
- **ADMM optimization**: Alternating Direction Method of Multipliers with sheaf constraints
- **Heterogeneous consensus**: Different agents can have different "notions of global consensus"
- **Explainable dynamics**: Primal, consensus, and dual state variables are exposed for direct analysis
- **Key insight**: "Heterogeneous notions of global consensus" — agents don't need to agree on everything, just the relevant subset

#### NeoTrix Domain Mapping
| Domain | Mapping |
|--------|---------|
| **NT-CORE** | GWT attention — sheaf defines which modules see which broadcasts |
| **NT-MEMORY** | KB namespace sharing — sheaf defines cross-namespace consistency rules |
| **NT-ACT** | Multi-agent orchestration — sheaf = coordination protocol |
| **NT-SHIELD** | Trust boundaries — sheaf constraints enforce security invariants |

#### Fusion Opportunities
- **GWT sheaf routing**: Instead of broadcast-everything, use sheaf to define per-module attention scope — reduces noise
- **Cross-domain coordination**: ConsciousnessTree branches as sheaf nodes — partial consensus (not full alignment)
- **KB consistency**: Sheaf constraints for cross-namespace data integrity
- **Action**: Model NT-* domain coordination as cellular sheaf over domain graph

---

### 5. NeuralFSM — Finite-State Multi-Agent Coordination (ACL 2026)
**Paper**: ACL Anthology 2026.acl-long.1543
**Authors**: Wang et al. (ACL 2026)

#### Core Architecture
- **State-driven execution**: Multi-agent problem solving as finite-state execution process
- **Temporal Coordination Controller**: Learns state transition distribution AND inter-agent communication weights from interaction traces
- **Task-context modulation**: Transitions and routing decisions driven by task context (not manual protocol design)
- **Key insight**: Coordination emerges from learned state transitions, not handcrafted rules

#### NeoTrix Domain Mapping
| Domain | Mapping |
|--------|---------|
| **NT-CORE** | ConsciousnessTree 6-stage loop = finite state machine |
| **NT-MIND** | SEAL pipeline stages = state transitions |
| **NT-ACT** | Agent orchestration — FSM as coordination protocol |
| **NT-REPAIR** | Self-healing — state machine detects stuck states, triggers repair |

#### Fusion Opportunities
- **ConsciousnessTree as FSM**: Formalize 6 stages (Soil→Roots→Trunk→Branches→Fruits→Core) as finite states with learned transitions
- **SEAL pipeline optimization**: Learn optimal stage transitions from execution traces (not fixed)
- **Cross-branch coordination**: Temporal Coordination Controller for NT-* domain handoffs
- **Action**: Model ConsciousnessTree as NeuralFSM, train transition distribution from KB execution logs

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Adaptive Compute Allocation
All 5 papers share the insight: **allocate compute proportional to difficulty**.
- Tandem: Cost-aware termination
- DUET: Length-penalized training
- CAT: Confidence-adaptive thinking
- Sheaf-ADMM: Heterogeneous consensus (different effort per relationship)
- NeuralFSM: Task-context modulation

**NeoTrix Mapping**: GWT salience should incorporate difficulty/confidence weighting. Easy tasks → cheap models, few reasoning tokens. Hard tasks → expensive models, full deliberation.

### Pattern 2: Decomposed Reasoning
Papers 1-3 decompose reasoning into stages:
- Signal generation → Execution (Tandem, DUET)
- Confidence assessment → Adaptive depth (CAT)

**NeoTrix Mapping**: E8 Hexagram analysis could decompose: trigram pattern match (fast) → full hexagram analysis (slow). Test-time compute scaling.

### Pattern 3: Learned Coordination > Handcrafted Rules
Papers 4-5 replace handcrafted coordination with learned protocols:
- Sheaf constraints learned from data
- FSM transitions learned from traces

**NeoTrix Mapping**: ConsciousnessTree branch coordination should be learned from KB execution patterns, not hardcoded. SEAL pipeline stages should adapt based on task type.

## Action Items

| # | Action | Domain | Priority |
|---|--------|--------|----------|
| 1 | Implement `SufficiencyClassifier` for adaptive model switching | NT-IO | P0 |
| 2 | Add confidence estimator to GWT salience scoring | NT-CORE | P0 |
| 3 | Model domain coordination as cellular sheaf | NT-CORE/NT-MEMORY | P1 |
| 4 | Formalize ConsciousnessTree as NeuralFSM | NT-CORE | P1 |
| 5 | Add complexity estimator for E8 reasoning depth | NT-CORE | P2 |
| 6 | Train SEAL pipeline stage transitions from KB logs | NT-MIND | P2 |
