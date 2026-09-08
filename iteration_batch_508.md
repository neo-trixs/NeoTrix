# Iteration Batch 508 — NeoTrix Consciousness Architecture Research Loop

**Date**: 2026-09-06
**Focus**: Reinforcement Learning, Policy Optimization, Exploration — 2026 Advances

---

## 1. Sources Cited

### Reinforcement Learning (2026)
| # | Title | Source | Date |
|---|-------|--------|------|
| RL1 | Divergence Proximal Policy Optimization (DPPO) — replaces PPO ratio clipping with divergence-based mask for LLM vocabularies | arXiv 2602.04879 | 2026-02 |
| RL2 | Divergence Regularized Policy Optimization (DRPO) — smooth quadratic regularizer replaces DPPO's hard mask, bounded gradient weights | arXiv 2606.09821 | 2026-06 |
| RL3 | DPO Meets PPO: Reinforced Token Optimization (RTO) — token-wise reward from DPO fed into PPO stage | Microsoft Research, 2025 (extended 2026) | 2025-05 |
| RL4 | Sequence-Level PPO (SPPO) — reformulates reasoning as sequence-level contextual bandit, decoupled scalar value function | ACL 2026 | 2026 |
| RL5 | Flow-DPPO — divergence-based mask for flow matching models (image/video generation), asymmetric mask blocks only harmful updates | arXiv 2606.11025 | 2026-06 |
| RL6 | Cumulative Prefix-divergence PPO (CPPO) — position-weighted threshold + cumulative prefix budget for autoregressive trust regions | arXiv 2606.10968 | 2026-06 |
| RL7 | PPO Suffices for On-Policy RL — early stopping criterion decouples PPO hyperparameters, scaling law for threshold vs batch count | UAI 2026 (PMLR v337) | 2026-08 |
| RL8 | Prefix-Normalized Policy Optimization (PNPO) — geometric mean of prefix likelihood ratios for off-policy rollout reuse | arXiv 2608.01418 | 2026-08 |
| RL9 | DisPPO — quantile-based distributional RL for LLMs | ICML 2026 | 2026-07 |
| RL10 | Temporal Instance-Graph Policy Optimization (TIGPO) — persistent transition graphs across policy updates for long-horizon agents | arXiv 2609.03383 | 2026-09 |
| RL11 | PS-PPO — prefix-sampling for critic-free RLHF, truncated gradient estimator with unbiased correction | ICML 2026 | 2026-07 |
| RL12 | Environment-Regularized Policy Optimization (ERPO) — Query-KL regularizer on input side preserves exploration while bounding query drift | arXiv 2608.23311 | 2026-08 |

### Policy Optimization (2026)
| # | Title | Source | Date |
|---|-------|--------|------|
| PO1 | Generative Actor-Critic (GenAC) — generative critic with chain-of-thought reasoning + In-Context Conditioning for value estimation | arXiv 2604.10701 | 2026-04 |
| PO2 | CE-GPPO — Coordinating Entropy via Gradient-Preserving Clipping, reintroduces clipped token gradients in bounded manner | ACL 2026 | 2026 |
| PO3 | Decoupled Gradient Policy Optimization (DGPO) — probability gradient (not log-prob gradient) as primitive, polynomial + reciprocal radical decay | ACL 2026 | 2026 |
| PO4 | RLHF Deciphered — critical analysis of RLHF methodology, reward model limitations, SFT as initial policy | ACM Computing Surveys | 2026 |
| PO5 | Unified Post-Training Framework (UPT) — single policy gradient framework unifying pretraining/SFT/RLHF/RLVR | arXiv 2407.16216 (v2 2026) | 2026-05 |
| PO6 | Reinforcement Learning for LLM Post-Training: A Survey — decomposes all methods along prompt/response/gradient axes | alphaXiv | 2026-05 |

### Exploration (2026)
| # | Title | Source | Date |
|---|-------|--------|------|
| EX1 | Curiosity-Driven Exploration (CDE) — actor perplexity + multi-head critic variance as exploration bonus for RLVR | ICLR 2026 | 2026 |
| EX2 | Active Inference: Sufficient curiosity guarantees posterior consistency + no-regret optimization | arXiv 2602.06029 | 2026-02 |
| EX3 | In-Context Learning for Intrinsic Curiosity — ICL as update-free world model for exploration, negative result for general MDPs, positive for BED | arXiv 2606.19476 | 2026-06 |
| EX4 | Curiosity as Information Gain (CIG) — novelty sensitivity + learnability filtering + competence-weighted priority | arXiv 2603.00009 | 2026-03 |
| EX5 | From Curiosity to Competence — co-evolution of world models and exploration, hybrid curiosity+empowerment outperforms either alone | arXiv 2507.08210 | 2026 (extended) |
| EX6 | Strategy-aware Surprise (SuS) — strategy-level (not state-level) novelty signal, strategy stability + strategy surprise | arXiv 2601.10349 | 2026-01 |
| EX7 | Decoupled Exploration: Go-With-Uncertainty (GowU) — tree search decoupled from policy optimization, 10x more efficient exploration | arXiv 2603.22273 | 2026-03 |
| EX8 | Should You Use Your LLM to Explore or Exploit? — bandit framework for LLM explore/exploit decisions | UAI 2026 (PMLR v337) | 2026-08 |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-RL-001: λ-GRPO Uses Fixed PPO Ratio Clipping — Ignores Divergence-Based Trust Regions
**Priority**: HIGH
**Severity**: Algorithmic Obsolescence
**Location**: `neotrix-core/src/unified/core/nt_core_prm/learner.rs:543-664` (λ-GRPO loss function)

**Finding**: The λ-GRPO implementation uses standard PPO ratio clipping (`clip(r_t(θ), 1-ε, 1+ε)·A_t`) with a fixed ε=0.2. This is the exact mechanism that DPPO (RL1), DRPO (RL2), CPPO (RL6), and DGPO (PO3) have identified as structurally flawed for LLM vocabularies: ratio clipping over-penalizes low-probability tokens and under-constrains high-probability tokens, creating training instability.

**2026 Evidence**:
- DPPO (RL1) proves ratio clipping is a "noisy single-sample Monte Carlo estimate" of true policy divergence, leading to sub-optimal learning dynamics in long-tailed vocabularies
- DRPO (RL2) replaces DPPO's hard mask with smooth quadratic regularizer — achieves bounded gradient weights that attenuate diverging updates
- CPPO (RL6) adds position-weighted thresholds + cumulative prefix budget — autoregressive asymmetry means early tokens need tighter constraints
- DGPO (PO3) proves that log-probability gradients diverge as probability → 0; probability gradient (not log-prob) is the correct optimization primitive

**Impact**:
- λ-GRPO's trust region is misaligned with the actual token distribution geometry
- Training instability increases with vocabulary size (long-tailed distribution)
- No corrective signal once tokens cross the trust region boundary
- Entropy collapse risk from discarding low-probability token gradients

**Suggestion**: Replace PPO ratio clipping in λ-GRPO with one of:
- **Option A (recommended)**: DRPO-style smooth quadratic regularizer on Binary-TV divergence — same trust region geometry as DPPO but with bounded gradient weights at boundaries
- **Option B**: CPPO-style position-weighted threshold — add cumulative prefix budget since λ-GRPO operates on step-level advantages that compound across autoregressive positions
- **Option C**: DGPO-style probability gradient with decoupled decay — replace `∇θ log πθ` with `∇θ πθ` and apply polynomial decay (left) + reciprocal radical decay (right)

### DEFECT-RL-002: No Off-Policy Correction for Multi-Epoch Rollout Reuse
**Priority**: HIGH
**Severity**: Sample Efficiency Gap
**Location**: `neotrix-core/src/unified/core/nt_core_prm/learner.rs:699-714` (λ-GRPO learning step)

**Finding**: The λ-GRPO learner collects trajectories and applies multiple gradient steps per batch, but there is no importance-weighting correction for the growing learner-behavior mismatch. Each epoch of updates makes later minibatches increasingly off-policy.

**2026 Evidence**:
- PNPO (RL8) demonstrates that 4-epoch reuse with prefix-normalized importance weighting achieves +3.0pp over GSPO and uses 4x fewer rollouts for equivalent performance
- PPO Suffices (RL7) provides scaling law between early-stopping threshold and batch count — adaptive early stopping replaces brittle hyperparameter coupling
- PS-PPO (RL11) shows prefix-sampled truncated gradient estimator can be unbiased with importance correction

**Impact**:
- λ-GRPO wastes compute by discarding off-policy trajectories rather than correcting them
- No principled early-stopping criterion — relies on fixed hyperparameters
- Multi-epoch training quality degrades silently (no monitoring of learner-behavior drift)

**Suggestion**:
- Add cumulative importance ratio tracking per trajectory (PNPO's prefix-normalized variant to avoid product-form explosion)
- Implement RL7's early-stopping criterion as adaptive epoch count based on trajectory divergence
- Add a `PolicyLagMonitor` that tracks KL divergence between current and behavior policy per epoch, triggering early stop when threshold exceeded

### DEFECT-RL-003: No Generative Value Model for Credit Assignment
**Priority**: MEDIUM
**Severity**: Credit Assignment Gap
**Location**: Architecture-level — λ-GRPO is critic-free by design (uses group-relative advantages)

**Finding**: λ-GRPO deliberately avoids a learned value function, using GRPO's group-relative reward normalization instead. However, GenAC (PO1) demonstrates that discriminative critics have fundamental representational limitations — they cannot express the complexity of value functions under evolving policies. Generative critics with chain-of-thought reasoning and In-Context Conditioning achieve significantly better credit assignment.

**2026 Evidence**:
- GenAC (PO1) shows discriminative critics "do not improve reliably with scale" while generative critics with CoT reasoning + ICC sustain performance margin throughout training
- GenAC outperforms both value-free (GRPO/RLOO) and value-based (VC-PPO) baselines
- The gap widens as training progresses — value-free methods plateau earlier

**Impact**:
- For long-horizon reasoning tasks (SEAL pipeline's multi-step evolution cycles), group-relative advantages may be insufficient
- No mechanism to provide fine-grained per-step credit assignment within a SEAL cycle
- λ-GRPO's λ parameter is a fixed blend rather than a learned credit assignment

**Suggestion**: Add an optional `GenerativeCritic` module to the PRM system:
- LLM-based critic that performs CoT reasoning before value estimation
- In-Context Conditioning: inject current policy's capability profile (success rate, module maturity) into critic prompt
- Use for high-stakes SEAL decisions where group-relative advantages are too coarse (e.g., module promotion/demotion decisions)
- Keep λ-GRPO as default for cost-efficiency; GenAC as premium mode

### DEFECT-RL-004: GWT Attention Routing Has No Exploration-Exploitation Balance
**Priority**: HIGH
**Severity**: Architectural Gap
**Location**: `neotrix-core/src/unified/core/nt_core_gwt/workspace.rs` (GlobalWorkspace)

**Finding**: GWT attention routing uses resonance-based competition (WTA gate + oscillator network) for exploitation — selecting the most resonant specialist module. There is no systematic exploration mechanism to discover new specialist configurations or prevent premature convergence to familiar modules.

**2026 Evidence**:
- CDE (EX1) demonstrates actor perplexity + critic variance as exploration bonuses that prevent entropy collapse in RL
- CIG (EX4) decomposes curiosity into novelty sensitivity + learnability filtering + competence weighting — each component addresses a different failure mode
- SuS (EX6) shows strategy-level novelty (not state-level) is critical — exploring *how* to solve problems, not just *what* to solve
- GowU (EX7) achieves 10x more efficient exploration by decoupling it from policy optimization
- Active Inference (EX2) proves sufficient curiosity simultaneously ensures posterior consistency AND no-regret optimization

**Impact**:
- GWT may overfit to familiar specialist patterns, ignoring novel configurations
- No mechanism to discover that a new specialist combination could solve a problem better
- The oscillator network (Kuramoto) provides synchronization but not exploration pressure
- Resonance history tracks what was active, not what was *not* explored

**Suggestion**: Add exploration pressure to GWT:
- **Curiosity bonus**: When a specialist module's prediction confidence is low (high entropy over its outputs), boost its attention weight (CDE's actor perplexity approach)
- **Novelty detection**: Track specialist-module co-activation patterns in `resonance_history`; reward configurations that haven't been tried (count-based or RND-style)
- **Competence gating**: Use CIG's competence-weighted priority — only explore new configurations when current ones have high competence (prevents wasting compute on exploration when exploitation is needed)
- **Strategy surprise**: Compute SuS-style strategy embeddings from GWT broadcast history; reward transitions to novel strategy states

### DEFECT-RL-005: SEAL Pipeline Exploration Stage Uses No Intrinsic Motivation
**Priority**: MEDIUM
**Severity**: Exploration Efficiency Gap
**Location**: `neotrix-core/src/neotrix/ffi/seal_pipeline.rs:18-129` (SEAL exploration stage)

**Finding**: The SEAL pipeline's exploration stage collects `ExplorationResult` with discovered patterns, but there is no intrinsic motivation signal guiding *what* to explore. Exploration is driven by external task demands, not by curiosity or information gain.

**2026 Evidence**:
- CDE (EX1) achieves +3 point improvement over standard RLVR using curiosity-driven exploration bonuses
- CIG (EX4) discovers 34% more environment states than RND and 21% more than ICM with same compute budget
- Strategy-aware Surprise (EX6) achieves 17.4% Pass@1 improvement by exploring strategy space rather than state space

**Impact**:
- SEAL exploration may miss high-value patterns that aren't demanded by current tasks
- No mechanism to explore "adjacent possible" — areas just beyond current capability
- Exploration budget (`exploration_budget` in ConsciousnessTree) is modulated by forecast but not by information gain

**Suggestion**: Add intrinsic motivation to SEAL exploration:
- **Information gain bonus**: When exploring, weight patterns by expected reduction in knowledge gap (use `knowledge_gap_detector.rs`'s `exploration_priority` scores as a proxy for epistemic uncertainty)
- **Competence-weighted exploration**: Only explore domains where current competence is moderate (CIG's insight — too low = can't learn, too high = no novelty)
- **Strategy surprise**: Track SEAL exploration strategies over time; reward transitions to novel exploration strategies (SuS's approach)

### DEFECT-RL-006: No Query-Side Regularization for Distribution Drift
**Priority**: LOW
**Severity**: Distributional Stability Gap
**Location**: Architecture-level — no query distribution monitoring

**Finding**: NeoTrix's SEAL pipeline and GWT attention routing operate on whatever queries/tasks arrive, with no mechanism to detect or regularize drift in the distribution of queries being processed.

**2026 Evidence**:
- ERPO (RL12) demonstrates that policy drift in the *query distribution* (not just response distribution) is a critical failure mode — standard Policy-KL regularizer acts on responses but not inputs
- ERPO's Query-KL term bounds query distribution shift while preserving exploration (no gradient pressure on responses)

**Impact**:
- Over time, NeoTrix may specialize on a narrow query distribution, losing generalization
- GWT resonance patterns may overfit to recent query types
- No mechanism to detect when the system is drifting from its intended domain

**Suggestion**: Add a lightweight query distribution monitor:
- Track rolling distribution of query types (via E8 state mapping or task type classification)
- Compute KL divergence from reference distribution
- When drift exceeds threshold, increase exploration budget in SEAL pipeline (force exploration of underrepresented domains)

---

## 3. Summary of Suggestions (Priority-Ordered)

| # | Defect | Suggestion | Effort |
|---|--------|-----------|--------|
| 1 | RL-001: PPO ratio clipping in λ-GRPO | Replace with DRPO smooth quadratic regularizer or DGPO probability gradient | HIGH (core algorithm change) |
| 2 | RL-004: No GWT exploration pressure | Add curiosity bonus + novelty detection to GWT resonance | MEDIUM (new module) |
| 3 | RL-002: No off-policy correction | Add PNPO-style prefix importance weighting + adaptive early stopping | MEDIUM (augment existing) |
| 4 | RL-005: SEAL has no intrinsic motivation | Add information gain bonus to SEAL exploration stage | LOW (augment existing) |
| 5 | RL-003: No generative value model | Add optional GenerativeCritic to PRM system | MEDIUM (new module) |
| 6 | RL-006: No query distribution regularization | Add query KL monitoring + exploration budget coupling | LOW (new module) |

---

## 4. Cross-Iteration Patterns

This batch reveals a systemic gap: **NeoTrix has strong exploitation mechanisms (resonance, WTA, λ-GRPO) but weak exploration mechanisms**. The 2026 literature converges on a key insight: exploration and exploitation should be *architecturally decoupled* (GowU's paradigm) rather than blended into a single objective. NeoTrix's GWT does both via resonance competition, which biases toward exploitation. The SEAL pipeline does exploration but without intrinsic motivation signals. This asymmetry may cause the system to converge prematurely on familiar patterns while missing high-value novel configurations.

**Recommended meta-fix**: Create an `ExplorationArchitect` module (L6 meta-cognition layer) that:
1. Monitors GWT resonance entropy (low = over-exploitation, high = under-exploitation)
2. Adjusts SEAL exploration budget and intrinsic motivation signals
3. Tracks query distribution drift (ERPO's insight)
4. Reports to ConsciousnessTree for meta-level regulation

This would close the loop between exploration (SEAL) and exploitation (GWT) via a principled information-theoretic controller.
