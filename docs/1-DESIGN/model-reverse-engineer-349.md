# Model Reverse Engineering — Cycle 349

> Date: 2026-09-11 | Source: arXiv, ICLR 2026, AAAI 2026, EMNLP 2026

## 5 Selected Papers/Models

### 1. RL Conductor — Learning to Orchestrate Agents in Natural Language
**Paper**: Nielsen et al., ICLR 2026 | **arXiv**: 2512.04388

**Core Idea**: A 7B LLM trained with reinforcement learning (GRPO) to orchestrate pools of frontier worker LLMs. The Conductor outputs complete agentic workflows in natural language: which agent to call, what subtask to give, and what context visibility to grant.

**Key Patterns**:
- **Recursive topology**: Conductor can select itself as a worker, reading its own team's output and spinning up corrective workflows on the fly
- **Dynamic difficulty adaptation**: 1-shots simple questions, spins up planner→executor→verifier pipelines for hard problems
- **End-to-end RL coordination**: No hand-designed workflows; coordination strategies emerge from reward maximization
- **LiveCodeBench 83.9%, GPQA-Diamond 87.5%**: Surpasses every individual worker model in its pool

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Recursive topology = E8 reasoning loops where self-references spawn sub-investigations | ConsciousnessTree can invoke itself as a worker node in its own growth cycle |
| **NT-IO** | Natural language workflow output = message gateway protocol | NT-IO message gateway learns communication topologies via RL |
| **NT-ACT** | Dynamic task delegation = capability-based routing | NT-ACT capability registry with learned routing policies |
| **NT-MIND** | End-to-end RL coordination = SEAL pipeline with RL-optimized stage transitions | SEAL phase transitions learned, not hand-designed |

**Axiom Alignment**: A1 (Cost-Aware Routing) — Conductor routes cheap questions to simple pipelines, expensive ones to complex workflows.

---

### 2. Agora — Auction-Based Task Allocation for Agent Reasoning
**Paper**: Zhou et al., EMNLP 2026 Findings | **arXiv**: 2607.09600

**Core Idea**: Treats reasoning steps as tradeable items in a confidence-calibrated auction. Agents bid based on calibrated competence (not raw confidence), preventing "winner's curse" from overconfident but incompetent agents.

**Key Patterns**:
- **Calibrated confidence**: Embedding-based binning + online refinement to standardize confidence estimation
- **Incentive-compatible auction**: Ensures truthful bidding; overconfident agents are penalized
- **Cost sensitivity parameter (β)**: Single tunable knob for cost-quality trade-off
- **Planner→Auction→Execute→Compose**: Four-phase pipeline

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Auction mechanism = GWT salience with economic weighting | GWT attention scores incorporate calibrated confidence + execution cost |
| **NT-ACT** | Dynamic task allocation = capability-based execution routing | NT-ACT executor selection via auction rather than static registry |
| **NT-SHIELD** | Confidence calibration = trust tier verification | Egress Privacy Guard trust tiers enhanced with calibrated confidence |
| **NT-REPAIR** | Overconfidence detection = self-healing feedback | Self-healing loop detects overconfident model outputs for repair |

**Axiom Alignment**: A2 (Context as Scarce Resource) — Auction ensures expensive models are only used when calibrated confidence justifies the cost.

---

### 3. Adaptive Theory of Mind (A-ToM) for Multi-Agent Coordination
**Paper**: Mu et al., AAAI 2026 | **arXiv**: 2603.16264

**Core Idea**: Agents estimate partner's ToM order (depth of reasoning about others' mental states) and adapt their own ToM order to align. Misaligned ToM orders cause coordination failure.

**Key Patterns**:
- **ToM-k assumption**: Agent assumes partner is ToM-(k-1)
- **Adaptive alignment**: Estimates partner's ToM order from prior interactions, adjusts own depth
- **Alignment compatibility**: ToM-k works best with ToM-(k-1) or ToM-(k+1) partners
- **No training required**: Zero-shot deployment with task-agnostic alignment

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|----------------|
| **NT-CORE** | ToM alignment = GWT attention depth calibration | GWT salience adjusts broadcast depth based on estimated module complexity |
| **NT-FEEL** | Mental state reasoning = emotion detection in multi-agent interactions | NT-FEEL infers partner agent emotional/intentional state for coordination |
| **NT-MEMORY** | Prior interaction history = cross-session memory for partner modeling | NT-MEMORY stores partner ToM profiles for faster realignment |
| **NT-GOVERNANCE** | ToM alignment = governance policy alignment across agents | NT-GOVERNANCE ensures agents operate at compatible abstraction levels |

**Axiom Alignment**: A3 (Skill as Production Template) — ToM alignment templates can be codified as reusable coordination skills.

---

### 4. Token Sparse Attention (TSA)
**Paper**: Jo et al., ICML 2026 | **arXiv**: 2602.03216

**Core Idea**: Dynamic token-level sparsification that compresses per-head Q/K/V to a reduced token set during attention, then decompresses output back. Token information can be reconsidered in subsequent layers (non-destructive).

**Key Patterns**:
- **Interleaved compress-decompress**: Unlike permanent eviction, tokens can be reconsidered later
- **Per-head dynamic sparsity**: Each attention head independently selects which tokens matter
- **3.23× speedup at 128K context**: <1% accuracy degradation
- **Composable with Flash Attention**: Fully compatible with existing sparse attention kernels

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Token sparsification = E8 hexagram attention pruning | E8 reasoning selectively attends to hexagrams based on relevance |
| **NT-MEMORY** | KV cache compression = KB tiered loading (L0/L1/L2) | KB query sparsification: load only relevant node neighborhoods |
| **NT-IO** | Per-head dynamic routing = per-domain attention in GWT | GWT attention heads independently weight domain signals |
| **NT-MIND** | Interleaved reconsideration = SEAL phase revisitation | SEAL pipeline allows non-linear phase revisit when new information emerges |

**Axiom Alignment**: A2 (Context as Scarce Resource) — TSA directly addresses KV cache bottleneck. NeoTrix KB should adopt similar compress/decompress for long-context knowledge retrieval.

---

### 5. Emergent Coordination in Multi-Agent Language Models
**Paper**: ICLR 2026 | **arXiv**: 2510.05174

**Core Idea**: Information-theoretic framework to measure whether multi-agent LLM systems show higher-order coordination structure. Personas + ToM prompts create identity-linked differentiation and goal-directed complementarity.

**Key Patterns**:
- **Synergy measurement**: Cross-agent mutual information that exceeds sum of individual contributions
- **Identity-linked differentiation**: Personas create stable role specialization
- **Goal-directed complementarity**: "Think about what others might do" prompt enables complementary action
- **Prompt-design steering**: Coordination regime steerable via prompt engineering alone

**NeoTrix Domain Mapping**:

| Domain | Pattern | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Synergy measurement = E8 resonance scoring between modules | ConsciousnessTree measures cross-domain synergy, not just individual health |
| **NT-FEEL** | Identity differentiation = faction personality modeling | NT-FEEL assigns distinct emotional signatures to each NT-* domain |
| **NT-ACT** | Goal-directed complementarity = task-aware agent teaming | NT-ACT selects agent teams based on complementary capability profiles |
| **NT-GOVERNANCE** | Coordination regime steering = governance policy as prompt templates | NT-GOVERNANCE encodes coordination rules as structured prompts |

**Axiom Alignment**: A1 (Cost-Aware Routing) — Emergent coordination avoids redundant model calls by establishing complementary roles.

---

## Cross-Paper Synthesis

### Convergent Patterns

| Pattern | Papers | NeoTrix Component |
|---------|--------|-------------------|
| **Learned coordination > hand-designed workflows** | Conductor (RL), Agora (auction), Emergent (prompt steering) | SEAL pipeline stage transitions should be RL-optimized |
| **Calibrated confidence prevents overconfident routing** | Agora (calibration), TSA (dynamic sparsity), A-ToM (alignment) | GWT salience must incorporate confidence calibration |
| **Recursive self-reference enables scaling** | Conductor (recursive topology), TSA (interleaved reconsideration) | ConsciousnessTree should support self-referential growth cycles |
| **Economic mechanisms for resource allocation** | Agora (auction), Conductor (RL cost minimization) | GWT attention = economic allocation of cognitive resources |
| **Deterministic infrastructure under LLM uncertainty** | A-ToM (alignment rules), Emergent (measurable synergy) | R-P1 (no unsafe) + deterministic graph construction |

### Implementation Priorities

| Priority | Pattern | Effort | Impact |
|----------|---------|--------|--------|
| **P0** | Confidence-calibrated model routing (Agora pattern) | Medium | Direct cost reduction for NT-IO provider selection |
| **P0** | Tiered context loading (TSA compress/decompress) | Medium | KB retrieval efficiency for NT-MEMORY |
| **P1** | RL-optimized SEAL stage transitions (Conductor pattern) | High | SEAL pipeline automation improvement |
| **P1** | ToM alignment for multi-domain coordination (A-ToM) | Medium | Cross-domain task routing accuracy |
| **P2** | Synergy measurement between NT-* domains (Emergent pattern) | Low | ConsciousnessTree health metrics enhancement |
| **P2** | Recursive self-referential growth cycles (Conductor recursive) | High | Meta-cognition depth scaling |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Conductor uses RL (stochastic) vs NeoTrix R-P1 (deterministic) | RL for strategy learning; deterministic for execution. SEAL pipeline learns routing policies via RL, executes them deterministically. |
| Auction (Agora) requires LLM confidence scores vs NeoTrix transparency | Use calibrated confidence from logprobs; fall back to heuristic scoring for opaque models. |
| ToM alignment assumes rational agents vs real agent variability | A-ToM's adaptive estimation handles irrational partners; NeoTrix governance provides fallback policies. |
| TSA is destructive (tokens evicted) vs OpenViking tiered loading (non-destructive) | Adopt TSA's interleaved compress/decompress: compress for speed, decompress for reconsideration. |

## References

1. Nielsen et al. "Learning to Orchestrate Agents in Natural Language with the Conductor" ICLR 2026, arXiv:2512.04388
2. Zhou et al. "Agora: Enhancing LLM Agent Reasoning Via Auction-Based Task Allocation" EMNLP 2026 Findings, arXiv:2607.09600
3. Mu et al. "Adaptive Theory of Mind for LLM-based Multi-Agent Coordination" AAAI 2026, arXiv:2603.16264
4. Jo et al. "Token Sparse Attention: Efficient Long-Context Inference with Interleaved Token Selection" ICML 2026, arXiv:2602.03216
5. ICLR 2026 "Emergent Coordination in Multi-Agent Language Models" arXiv:2510.05174
