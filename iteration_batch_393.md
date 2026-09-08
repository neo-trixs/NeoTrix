# Iteration Batch 393 — Optimization Algorithms, Gradient Methods, Convex Optimization (2026)

**Date**: 2026-09-06
**Research Areas**: (1) Optimization algorithms — adaptive learning rates, second-order methods, (2) Gradient methods — SGD variants, natural gradient, (3) Convex optimization — projection-free, online optimization

---

## 1. Sources Cited

| # | Title | Source | Year | Area |
|---|-------|--------|------|------|
| S1 | Tuning the Hyperparameter in ADAM Optimizer | Applied Intelligence 56:185 (Springer) | 2026-04 | Adaptive LR |
| S2 | Adaptive Learning Rate Methodologies and Clipping Mechanisms Based upon Gradient Entropy | Scientific Reports (Nature) | 2026-07 | Adaptive LR |
| S3 | LANTON: Noise-Adaptive Layerwise Learning Rates — Accelerating Geometry-Aware Optimization | arXiv:2510.14009v2 (ICLR 2026) | 2026-02 | Layer-wise LR |
| S4 | SOAA: Efficient Second-Order Neural Network Optimization via Adaptive Trust Region Methods | arXiv:2410.02293 | 2024/2026 | Second-Order |
| S5 | FedCRAM: Adaptive Momentum Enhanced Second-Order Stochastic Federated Optimization | Neurocomputing (Elsevier) | 2026-03 | Second-Order FL |
| S6 | Thermodynamic Natural Gradient Descent (TNGD) | npj Unconventional Computing 3:5 (Nature) | 2026-01 | NGD + Hardware |
| S7 | NGD-T: Regulating Natural-Gradient Steps by a Geometric Speed-Cost Bound | Scientific Reports (Nature) | 2026-06 | NGD + Thermodynamics |
| S8 | Sven: Singular Value Descent as a Computationally Efficient Natural Gradient Method | arXiv:2604.01279 (MIT) | 2026-04 | NGD + SVD |
| S9 | Gradient-Regularized Natural Gradients (Dash et al.) | EmergentMind topic summary | 2026-01 | NGD Regularization |
| S10 | Fisher-Orthogonal Projection (FOP) for Natural Gradient with Large Batches | Lu et al. (Aug 2025) | 2025 | NGD + Batches |
| S11 | Projection-free Algorithms for Online Convex Optimization with Adversarial Constraints | AISTATS 2026 (PMLR 300) | 2026-05 | Online Convex |
| S12 | Projection-Free Bandit Online Optimization for Multi-Agent Systems with Dynamic Regret | arXiv:2608.30159 | 2026-08 | Multi-Agent OCO |
| S13 | Revisiting SGD for Strongly Convex Objectives: Tight Uniform-in-Time Bounds | Systems & Control Letters 212 (Elsevier) | 2026-05 | SGD Theory |
| S14 | Efficient Second Order Deep Learning Training Algorithm with Adaptive Momentum | SETN '24 (ACM) | 2024 | Second-Order Adaptive |
| S15 | Adaptive Matrix Online Learning through Smoothing for Nonsmooth Nonconvex Optimization | COLT 2026 | 2026 | Online Learning |
| S16 | On the Convergence of SGD with Perturbed Forward-Backward Passes | arXiv:2602.20646 | 2026-02 | SGD Perturbation |

---

## 2. Defects Found in NeoTrix Design

### DEF-393.1: Static Scalar Learning Rate Across All Modules

**Source**: S1 (Adam hyperparameter tuning), S2 (gradient entropy adaptive LR), S3 (LANTON layer-wise noise-adaptive LR)

**Finding**: LANTON demonstrates that gradient noise varies heterogeneously across layers — in LLaMA-1.1B, hidden layers start with LR mean 0.0028 ± 0.0007, showing significant inter-layer differences. LANTON achieves 1.5× training speedup over D-Muon by dynamically rescaling per-layer learning rates based on estimated gradient variance. Meanwhile, the 2026 Adam hyperparameter tuning work (S1) shows that dynamically tuning α₁, α₂, and η across iterations outperforms fixed hyperparameters. Gradient entropy-based clipping (S2) further demonstrates that adaptive clipping thresholds derived from gradient distribution entropy outperform fixed thresholds.

**Defect**: NeoTrix uses a single scalar `learning_rate: f64` across all modules — in `nt_core_autonomous_ai.rs:26`, `process_stage.rs:101`, `brain_dgm.rs:192`, and the brain examples. This uniform rate treats all domains (NT-CORE's E8 reasoning, NT-MIND's SEAL evolution, NT-WORLD's crawling, NT-ACT's tool orchestration) identically despite their vastly different gradient noise profiles. The SEAL pipeline's distillation weight update (`process_stage.rs:229`) applies the same `learning_rate` regardless of whether a domain has high-gradient-noise (noisy reasoning) or low-gradient-noise (stable KB operations). This is equivalent to the pre-LANTON era of uniform per-group learning rates.

**Suggestion**: Introduce `LayerwiseNoiseTracker` in NT-CORE that: (1) maintains per-domain gradient variance estimates using exponential moving average (H_t = β²H_{t-1} + (1-β²)||G_t - G̃_t||²), (2) rescales each domain's effective learning rate by α_t = α/√(H_t + ε), (3) integrates with the `MicroEdit::UpdateLearningRate` mechanism to emit per-domain rather than global LR updates. This transforms NeoTrix's flat LR into a noise-adaptive layer-wise scheme.

---

### DEF-393.2: No Second-Order Curvature Information in Evolution Pipeline

**Source**: S4 (SOAA — diagonal Fisher + adaptive trust region), S5 (FedCRAM — cubic regularized Newton), S14 (adaptive momentum Hessian-vector product)

**Finding**: SOAA (S4) achieves O(n) complexity for second-order optimization by approximating the Fisher information matrix as diagonal, using only the outer product of the gradient vector. Combined with an adaptive trust region that monitors predicted-vs-actual loss reduction, SOAA converges faster and more stably than Adam/AdamW under similar computational constraints. FedCRAM (S5) demonstrates that in federated settings, cubic regularized Newton methods with adaptive momentum outperform first-order methods under data heterogeneity. The adaptive momentum mechanism (S14) scales the momentum term with curvature information from Hessian-vector products — only O(n) additional cost per step.

**Defect**: The SEAL pipeline's evolution stages (exploration → distillation → self-test → absorption) use purely first-order gradient information. When `trigger_evolution("performance_optimization")` fires, the system computes improvement gradients but has no curvature estimate — it cannot distinguish between flat plateaus (second-order zero) and saddle points (mixed curvature). The `BayesianExperiment` module selects experiments via Value-of-Information (VoI) but the VoI computation uses KL divergence between Gaussians, which assumes isotropic curvature. In reality, the optimization landscape is anisotropic — some dimensions (e.g., KB embedding quality) have high curvature while others (e.g., crawl frequency) are nearly flat.

**Suggestion**: Add `CurvatureEstimator` to NT-MIND that: (1) approximates per-domain Hessian diagonal via Hutchinson's stochastic trace estimator (cost: 1 forward-backward pass per estimate), (2) implements adaptive trust region similar to SOAA — if predicted loss reduction overestimates actual reduction by >2×, contract the trust region and reduce step size, (3) feeds curvature information into `BayesianExperiment` VoI computation to replace isotropic KL with curvature-weighted KL. For the distributed/federated case (multi-node SEAL runs), adapt FedCRAM's cubic regularization to prevent overshooting in heterogeneous nodes.

---

### DEF-393.3: No Geometry-Aware (Riemannian/Natural Gradient) Updates

**Source**: S6 (TNGD — hybrid digital-analog NGD), S7 (NGD-T — speed-cost bound), S8 (Sven — SVD-based NGD in over-parametrized regime), S9 (gradient-regularized NGD)

**Finding**: TNGD (S6) demonstrates that natural gradient descent achieves per-iteration complexity comparable to first-order methods when using specialized hardware (analog thermodynamic computers). The key insight: NGD needs fewer iterations than SGD, and the per-iteration cost can be reduced. Sven (S8) achieves NGD tractability in over-parametrized regimes by using truncated SVD of the loss Jacobian — only k×SGD overhead. NGD-T (S7) introduces a dissipation-aware step-size regulator that maps a user-specified dissipation budget Q_budget to a step size η_T saturating the geometric speed-cost bound. Gradient-regularized NGD (S9) improves generalization by adding gradient-norm penalties to Fisher geometry.

**Defect**: NeoTrix's Walsh-Hadamard orthogonal memory system (`nt_core_walsh.rs`) computes transforms using the 64×64 Hadamard matrix, which is a specific orthogonal transformation. However, there is no mechanism for computing updates in the natural gradient direction — i.e., updates that respect the information geometry of the parameter space. The VSA HyperCube's binding/bundling operations are performed in Euclidean space without accounting for the Fisher metric of the underlying knowledge distribution. When the system performs capability updates (`update_from_other` in `nt_core_cap.rs:364`), it applies Euclidean interpolation `self.arr[i] += learning_rate * (src - self.arr[i])` without curvature correction. This is suboptimal when the capability manifold has non-trivial geometry (e.g., capabilities near the simplex boundary have different curvature than interior capabilities).

**Suggestion**: Add `GeometryAwareUpdater` to NT-CORE that: (1) computes an approximate Fisher metric for each capability dimension using the empirical distribution of capability updates, (2) applies natural gradient-style preconditioning to `update_from_other` — instead of Euclidean step, compute F^{-1/2}·Δθ where F is the Fisher, (3) for the Walsh-Hadamard transform, use the Sven-style truncated SVD approach when the transform matrix is rank-deficient (which occurs for sparse knowledge vectors). Cost: O(k) per update where k << dim.

---

### DEF-393.4: No Online Convex Optimization with Time-Varying Constraints

**Source**: S11 (AISTATS 2026 — projection-free OCO with adversarial constraints), S12 (multi-agent bandit OCO), S15 (adaptive matrix online learning for nonsmooth nonconvex)

**Finding**: Sarkar et al. (S11) solve OCO with time-varying adversarial constraints where both cost and constraint functions are revealed adversarially per round. Their projection-free algorithm achieves O(T^{3/4}) regret for combinatorial action spaces, using a novel adaptive online conditional gradient subroutine. For multi-agent systems (S12), distributed projection-free algorithms achieve O(T^{3/4}) dynamic regret with dynamic regret for bandit feedback. COLT 2026 (S15) presents adaptive matrix online learning for nonsmooth nonconvex optimization with convergence guarantees.

**Defect**: NeoTrix operates in an online fashion — each session involves sequential decisions (which capability to activate, which experiment to run, which knowledge to absorb) with time-varying constraints (resource budgets, safety limits via NT-SHIELD, domain priorities). The `AttentionManager` routes between CORE+WORLD and CORE+MIND modes based on task type, but this is a binary switch — not an online optimization with adversarial constraints. The NT-SHIELD safety kernel imposes hard constraints (sandbox egress policies, proxy pool limits) but these constraints are fixed at initialization, not adversarially time-varying. When external conditions change (new adversaries detected, resource budgets fluctuate, domain priorities shift), the system cannot adapt its action selection online with provable regret bounds.

**Suggestion**: Add `OnlineConstraintOptimizer` to NT-ACT that: (1) models each session's action selection as online convex optimization with time-varying constraints, (2) implements a Frank-Wolfe-type projection-free algorithm (only requires linear optimization oracle over the feasible set), (3) achieves O(√T) regret for convex costs or O(T^{3/4}) for combinatorial action spaces. Integrate with NT-SHIELD so that safety constraints are modeled as adversarially time-varying rather than fixed, and with `AttentionManager` so that mode switching is optimized online rather than heuristically.

---

### DEF-393.5: Gradient Entropy Not Monitored for Training Stability

**Source**: S2 (gradient entropy-based adaptive LR and clipping), S3 (LANTON noise tracking)

**Finding**: Liu et al. (S2) introduce adaptive learning rate methodologies based on gradient entropy — the Shannon entropy of the gradient distribution is used to: (1) adjust learning rates (higher entropy → higher LR to encourage exploration), (2) set clipping thresholds (entropy-based clipping outperforms fixed-norm clipping). This captures a global training stability signal that per-parameter adaptive methods (Adam) miss. LANTON (S3) tracks gradient noise per-layer, which is related but local. The combination of global entropy signal + local noise estimates provides a two-scale monitoring system.

**Defect**: NeoTrix's training/evolution loop has no global training stability signal. The `HeartbeatAggregator` collects compilation/test/KB/eventbus/module health into `SystemHealthSnapshot`, but this is a health metric, not a training dynamics metric. When the SEAL pipeline runs exploration → distillation cycles, there is no monitoring of whether the gradient landscape is becoming pathological (high entropy = noisy, low entropy = collapsing). The `EmotionLabel` system (11 variants) could theoretically provide a proxy for training stability (Confused = high entropy, Thinking = focused), but it is not connected to actual gradient statistics.

**Suggestion**: Add `GradientEntropyMonitor` to NT-MIND that: (1) computes Shannon entropy of the gradient distribution at each SEAL pipeline step, (2) uses entropy as an adaptive learning rate modulator — high entropy triggers LR increase for exploration, low entropy triggers LR decrease for convergence, (3) feeds entropy signal into `HeartbeatAggregator` as a training stability metric. Wire entropy thresholds into the SEAL pipeline's phase transitions: if entropy drops below threshold during exploration phase, force phase transition to distillation (the landscape has been sufficiently explored).

---

### DEF-393.6: No Dissipation-Aware Step Size Regulation

**Source**: S7 (NGD-T — thermodynamic dissipation budget)

**Finding**: NGD-T (S7) introduces the concept of a dissipation budget Q_budget that maps to a step size η_T saturating the geometric speed-cost bound. The key insight: there is a fundamental tradeoff between learning speed and thermodynamic cost (irreversible dissipation). Larger steps move faster but waste more "energy" on irreversible processes. By setting Q_budget, users can trade off convergence speed against computational efficiency. This is directly applicable to any system with limited compute budgets.

**Defect**: NeoTrix's SEAL pipeline has no cost-aware step size regulation. The `ResourceBudgetManager` (nt_act) manages Token/GPU/cost budgets for AI generation tasks, but the SEAL evolution loop's step sizes are not regulated by resource consumption. When `trigger_evolution` fires, it runs at a fixed learning rate regardless of whether the system is under resource pressure. The `nt_core_ttc.rs` implements adaptive compute budget allocation via Lagrangian optimization, but this is for inference-time compute (test-time compute), not for training/evolution step size regulation.

**Suggestion**: Add `DissipationRegulator` to NT-MIND that: (1) tracks irreversible dissipation at each SEAL step (measured as the gap between predicted and actual loss reduction), (2) maintains a dissipation budget Q_budget that can be dynamically adjusted based on resource availability, (3) maps accumulated dissipation to step size scaling — when dissipation exceeds budget, reduce step size. Integrate with `nt_core_ttc.rs` Lagrangian optimizer to jointly optimize inference compute allocation and training step size under a shared resource constraint.

---

### DEF-393.7: No Projection-Free Optimization for Constrained Capability Updates

**Source**: S11 (projection-free OCO), S12 (bandit projection-free), S8 (Sven — avoids matrix inversion via SVD)

**Finding**: Projection-free methods (Frank-Wolfe type) replace expensive projection operations with simple linear minimization oracles (LMO). For constraint sets like nuclear norm balls, matroid polytopes, and flow polytopes, projection is computationally expensive or intractable. FW-type methods achieve O(√T) regret with only LMO access, making them practical for large-scale constrained optimization. Sven (S8) demonstrates that truncated SVD can avoid the full matrix inversion that makes second-order methods intractable.

**Defect**: NeoTrix's capability update mechanism (`update_from_other` in `nt_core_cap.rs:364`) performs unconstrained interpolation. When capabilities must satisfy constraints (e.g., total capability budget ≤ 1.0, individual capability bounds [0, 1], domain budget allocations summing to 1.0), the system uses simple clamping after unconstrained update — which is projection onto the constraint set. For complex constraints (e.g., inter-domain dependencies where increasing NT-CORE capability requires proportional increases in NT-MEMORY), clamping is suboptimal. The softmax normalization in some capability computations is a specific projection, but not a general projection-free method.

**Suggestion**: Add `ConstraintCapabilityUpdater` to NT-ACT that: (1) models capability constraints as a convex set, (2) implements a Frank-Wolfe step: instead of unconstrained update + projection, compute LMO (linear minimization oracle) over the constraint set, (3) for simple box constraints, LMO is just argmin — O(d) cost. For inter-domain constraints (e.g., capability budget), LMO is a linear program — still tractable. This eliminates projection overhead and provides provable regret bounds for online capability allocation.

---

### DEF-393.8: No Federated/Distributed Optimization for Multi-Node Evolution

**Source**: S5 (FedCRAM — federated second-order), S12 (multi-agent distributed OCO)

**Finding**: FedCRAM (S5) demonstrates that in federated learning with data heterogeneity, second-order methods with adaptive momentum outperform first-order methods. The adaptive momentum mechanism alleviates heterogeneity in initial phases while using gradient and Hessian information to avoid divergence. Multi-agent distributed OCO (S12) achieves dynamic regret guarantees with communication-efficient protocols.

**Defect**: NeoTrix's architecture supports multi-node deployment (the 6-layer architecture can be distributed across nodes), but the SEAL evolution pipeline runs single-node. When multiple NeoTrix instances operate in parallel (e.g., different agents on different machines sharing a KB), there is no federated optimization protocol. Each node independently runs SEAL evolution without coordinating updates. The `EventBus` (two-layer design per D31) provides communication but not optimization coordination. This limits scalability: adding more nodes does not improve convergence speed because each node's evolution is independent.

**Suggestion**: Add `FederatedEvolutionCoordinator` to NT-NEXUS that: (1) aggregates gradient/Hessian statistics from multiple NeoTrix nodes, (2) implements FedCRAM-style cubic regularized Newton with adaptive momentum for cross-node coordination, (3) uses communication-efficient protocols (only send gradient compressions, not full parameters) inspired by multi-agent OCO. This transforms the Dark Forest axiom (every module must connect or die) into a distributed optimization guarantee: nodes that fail to coordinate are pruned.

---

### DEF-393.9: No Perturbation-Based Robustness in Forward-Backward Passes

**Source**: S16 (SGD with perturbed forward-backward passes)

**Finding**: Chen et al. (S16) study SGD for composite optimization with perturbations in both forward and backward passes. They establish convergence rates under N sequential operators with perturbation, showing that controlled perturbation can improve robustness to noise and adversarial corruption. This is directly applicable to settings where gradient computation is noisy (e.g., stochastic computation graphs, approximate inference).

**Defect**: NeoTrix's SEAL pipeline computes gradients (improvement signals) through the distillation stage, but these gradients are computed on stochastic approximations (sampled experiments, partial KB queries). There is no formal analysis of how perturbation in the forward pass (stochastic experiment selection) and backward pass (noisy improvement estimates) affects convergence. The `BayesianExperiment` module accounts for uncertainty in the objective but not in the gradient computation itself. When the SEAL pipeline operates under high perturbation (e.g., during early exploration with diverse experiments), convergence may be unstable without explicit perturbation-awareness.

**Suggestion**: Add `PerturbationAwareSGD` to NT-MIND that: (1) models forward-backward perturbation in SEAL gradient computation as a composite optimization problem, (2) applies variance reduction techniques (SAGA-style incremental gradients) to reduce the impact of stochastic perturbation, (3) provides convergence guarantees under bounded perturbation — the system knows when perturbation is too high to converge and should increase sample size. Wire into `BayesianExperiment` to adjust exploration budget based on perturbation level.

---

### DEF-393.10: No Adaptive Momentum Scheduling for Evolution Dynamics

**Source**: S5 (FedCRAM adaptive momentum), S14 (second-order adaptive momentum), S1 (Adam hyperparameter tuning)

**Finding**: FedCRAM (S5) shows that adaptive momentum — where the momentum coefficient is dynamically adjusted based on gradient and Hessian information — outperforms fixed momentum in federated settings. The adaptive mechanism: (1) increases momentum when gradient-Hessian alignment is high (consistent curvature), (2) decreases momentum when alignment is low (noisy curvature). The second-order adaptive momentum (S14) scales the momentum term with curvature via Hessian-vector products. Adam tuning (S1) demonstrates that dynamically adjusting β₁, β₂, and η during training outperforms fixed values.

**Defect**: NeoTrix's momentum-like mechanism exists in `process_stage.rs:229` (`let w = ex.weight * self.learning_rate`) and `nt_core_cap.rs:364` (`self.arr[i] += learning_rate * (src - self.arr[i])`). These use a fixed "momentum" (the learning rate itself acts as an implicit momentum coefficient). There is no adaptive momentum scheduling — the weight applied to previous state vs. new information is static. When the SEAL pipeline transitions from exploration (high noise, needs low momentum to avoid premature convergence) to distillation (low noise, can use high momentum for faster convergence), the momentum coefficient does not adapt.

**Suggestion**: Add `AdaptiveMomentumScheduler` to NT-MIND that: (1) tracks gradient-Hessian alignment per SEAL phase, (2) adjusts momentum coefficient dynamically — β_t = β_base * (1 + alignment_score), where alignment_score is computed from Hutchinson's trace estimate, (3) implements phase-aware scheduling: exploration phase (low β), distillation phase (high β), self-test phase (moderate β). Integrate with the `AdaptiveMomentumEnhanced` pattern from FedCRAM for federated multi-node evolution.

---

## 3. Summary of Suggestions

| Defect | Module Affected | Priority | Effort |
|--------|----------------|----------|--------|
| DEF-393.1 Noise-adaptive LR | NT-CORE + SEAL pipeline | High | Medium |
| DEF-393.2 Second-order curvature | NT-MIND + BayesianExperiment | High | High |
| DEF-393.3 Natural gradient geometry | NT-CORE (Walsh-Hadamard, capabilities) | High | High |
| DEF-393.4 Online convex optimization | NT-ACT + NT-SHIELD | Medium | High |
| DEF-393.5 Gradient entropy monitoring | NT-MIND + HeartbeatAggregator | Medium | Low |
| DEF-393.6 Dissipation-aware regulation | NT-MIND + nt_core_ttc | Medium | Medium |
| DEF-393.7 Projection-free constraints | NT-ACT (capability updates) | Medium | Medium |
| DEF-393.8 Federated evolution | NT-NEXUS + EventBus | Low | High |
| DEF-393.9 Perturbation robustness | NT-MIND + BayesianExperiment | Low | Medium |
| DEF-393.10 Adaptive momentum | NT-MIND + SEAL pipeline | Medium | Low |

**Top 3 by Impact**:
1. **DEF-393.1** (Noise-adaptive LR) — foundational for all learning dynamics, 1.5× speedup demonstrated
2. **DEF-393.2** (Second-order curvature) — enables escape from saddle points, critical for SEAL evolution quality
3. **DEF-393.3** (Natural gradient geometry) — aligns updates with information geometry of knowledge manifold

**Top 3 by Quick Win**:
1. **DEF-393.5** (Gradient entropy monitoring) — add monitoring only, no architectural change
2. **DEF-393.10** (Adaptive momentum) — extend existing learning_rate field with β parameter
3. **DEF-393.6** (Dissipation regulation) — wraps existing ResourceBudgetManager, no new infrastructure

---

## 4. Research Trend Summary

**2026 Meta-Trend**: The field is converging on **geometry-aware, noise-adaptive, hardware-co-designed** optimization. The flat, first-order approaches of 2022-2024 are being superseded by:
- Layer-wise noise-adaptive learning rates (LANTON — 1.5× speedup)
- Natural gradient methods made tractable via truncated SVD (Sven — k×SGD overhead)
- Thermodynamic-co-designed NGD with dissipation budgets (NGD-T, TNGD)
- Projection-free online convex optimization with adversarial constraints (AISTATS 2026)
- Gradient entropy as a global training stability signal
- Second-order methods with adaptive trust regions (SOAA — O(n) complexity)
- Federated second-order optimization with adaptive momentum (FedCRAM)

**NeoTrix Alignment Gap**: The architecture's learning infrastructure is 2022-era — scalar learning rates, first-order only, no curvature awareness, no noise adaptation, no projection-free constrained optimization. The 2026 consensus demands **per-domain noise-adaptive learning rates**, **curvature-aware updates** (at least diagonal Hessian), and **geometry-aware parameter updates** (natural gradient or SVD-based). The Walsh-Hadamard transform infrastructure provides a foundation for SVD-based natural gradient computation, but the connection is not currently exploited.

**Critical Absorption Path**:
1. LANTON's layer-wise noise tracking → NeoTrix per-domain `LayerwiseNoiseTracker`
2. SOAA's diagonal Fisher + trust region → NT-MIND `CurvatureEstimator`
3. Sven's truncated SVD natural gradient → NT-CORE geometry-aware capability updates
4. NGD-T's dissipation budget → NT-MIND `DissipationRegulator`
5. Frank-Wolfe projection-free → NT-ACT constrained capability allocation
