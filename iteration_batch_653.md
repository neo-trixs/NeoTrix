# Iteration Batch 653 — Cybernetics × Control Theory × Systems Theory (2026)

**Date:** 2026-09-06
**Context:** Batch 652 proved (1) GWT broadcasts zero MI compression, (2) no channel capacity analysis, (3) no algorithm-hardware co-design pathway, (4) no MI-Entropy plane, (5) no rate-distortion for HyperCube.

---

## 1. Cybernetics Findings

### Finding 1.1: Homeostasis ≠ Setpoint — Dynamic Causal Control (Weinberger 2026)
**Source:** https://doi.org/10.1007/s10539-026-10018-8 (Biology & Philosophy, May 2026)

**Key insight:** Classic cyberneticists (Wiener) equated homeostasis with negative feedback to a setpoint. Weinberger (2026) corrects this: paradigm control feedback and homeostatic mechanisms **do not maintain variables at constant values** — intervening on them would destroy the mechanism's function. Instead, control systems **exploit lower-scale feedback loops to produce higher-scale causal relationships**. The Watt governor doesn't just maintain engine speed; it matches energy supply to demand.

**New defect in NeoTrix:**
- **DEFECT-C653-1: GWT resonance treats homeostasis as setpoint maintenance.** `GlobalWorkspace::resonant_broadcast()` in `nt_core_gwt/workspace.rs` operates as a winner-take-most competition with threshold activation (`activation >= self.threshold`). This is the Wiener-era "maintain variables constant" paradigm. It lacks the dynamic causal control layer where feedback loops are exploited to produce higher-scale relationships. The workspace doesn't track what the resonance **enables** (higher-scale causal relationships), only what it **maintains** (activation above threshold).
- **Impact:** GWT cannot explain how attention routing creates **functional** outcomes (e.g., "when NT-WORLD specializes in crawling, NT-MEMORY's retrieval pathway should restructure"). The system maintains activation levels but has no model of what those activations **enable**.

### Finding 1.2: Agent Cybernetics — Two-Level Homeostatic Architecture (arXiv:2605.10754, 2026)
**Source:** https://arxiv.org/html/2605.10754 (Agent Cybernetics paper, 2026)

**Key insight:** Foundation agents need **ultrastability** (Ashby): a fast inner loop preserving viability region Ω under routine perturbations, plus a slow outer loop that restructures Ω itself when sustained boundary violations occur. The fast/slow timescale separation is the resolution of the stability-adaptivity tradeoff.

**New defect in NeoTrix:**
- **DEFECT-C653-2: No ultrastable architecture in GWT.** `GlobalWorkspace` has only one feedback loop: `resonant_broadcast()` → update activations → next cycle. There is no slow outer loop that can restructure the viability region itself. When the system's 14 specialist modules become misaligned with the task landscape (e.g., too many security specialists, too few knowledge integration specialists), there is no mechanism to restructure the specialist set.
- **Missing:** `ViabilityRegion` struct that tracks which module configurations are "viable" and can itself be revised when sustained performance degradation is detected.
- **Impact:** The system cannot adapt its own cognitive architecture. It can route attention among existing specialists but cannot add/remove/reconfigure specialists in response to persistent failure.

### Finding 1.3: Allostatic Control — Goal Governance Under Changing Environments (arXiv:2607.21771, July 2026)
**Source:** https://arxiv.org/abs/2607.21771

**Key insight:** A preregistered experiment showed that evidence-gated reference movement (allostatic control) was **1.51% worse** than mechanically matched ungated adaptation. The failure was timing: the evidence-to-effect pathway often could not make correction effective before the environment changed again. The design principle: "an allostatic controller must be able to revise an inappropriate goal **faster than serviceability is lost** by continuing to defend it."

**New defect in NeoTrix:**
- **DEFECT-C653-3: No allostatic goal governance.** NeoTrix's SEAL pipeline has a fixed goal structure (distillation → crystallization → absorption). There is no mechanism to revise the goal itself when the environment changes. The ConsciousnessTree runs its 6-stage loop at a fixed cadence with fixed objectives.
- **Missing:** A fast/slow loop for goal revision where the slow loop monitors whether the current SEAL objectives remain appropriate, and can restructure them before serviceability is lost.
- **Impact:** When the task landscape shifts (e.g., from code generation to documentation), NeoTrix continues pursuing the same SEAL objectives rather than revising them.

### Finding 1.4: Feedback Loops as Cognitive Structure (Amresh 2026)
**Source:** https://doi.org/10.5281/zenodo.19351679 (Cognitive Cybernetics Technical Monograph Series 1-07, March 2026)

**Key insight:** Feedback is structural — it is the mechanism by which cognition becomes structure, not a corrective add-on. Four primary feedback channels operate in parallel: (1) outcome reinforcement, (2) termination reinforcement, (3) evaluation weighting, (4) navigation bias. Feedback effects are **cumulative and self-sealing**: reinforced paths require less control effort, deviation becomes increasingly expensive, and feedback can fix cognition rather than adjust it.

**New defect in NeoTrix:**
- **DEFECT-C653-4: No self-sealing detection in GWT.** When resonance clusters form (e.g., modules 9+10 always resonate), the resonance history accumulates and reinforces those paths. But there is no mechanism to detect when feedback has become self-sealing — when the system is functioning well while losing adaptability. The `resonance_history: Vec<ResonanceReport>` accumulates indefinitely without analysis for lock-in patterns.
- **Missing:** A `FeedbackLockInDetector` that monitors whether the same resonance clusters dominate across diverse task types, indicating self-sealing behavior.
- **Impact:** The system can become rigid without detection. A dominant cluster reinforces itself across all tasks, reducing diversity of attention routing.

### Finding 1.5: Post-Cybernetics — Critical Cybernetics (Zhou & Zhou, 2026)
**Source:** https://doi.org/10.5281/zenodo.20335909 (Zenodo, May 2026)

**Key insight:** Classical cybernetics drives systems to steady states. But in creative adaptation, steady-state locking suppresses sensitivity to weak signals. **Critical cybernetics** argues control should pin the system in the **near-critical region** near the percolation threshold — maintaining maximal response sensitivity without falling into chaos or freezing. Control is redefined from stabilization engineering to **sensitivity management engineering**.

**New defect in NeoTrix:**
- **DEFECT-C653-5: GWT entropy monitoring is binary (focused vs distributed) but not near-critical.** `ResonanceReport::is_focused()` checks `entropy < 1.0`, `is_distributed()` checks `entropy >= 2.0`. There is no concept of operating near a critical point where sensitivity is maximized. The system either locks attention (low entropy) or scatters it (high entropy), but never maintains the edge-of-chaos state where the system is maximally responsive to weak signals.
- **Missing:** A `CriticalityManager` that computes the system's distance from the percolation threshold and adjusts resonance parameters to maintain near-critical operation.
- **Impact:** The system cannot detect when it has drifted into either frozen (low sensitivity) or chaotic (noisy) regimes. It cannot actively maintain the optimal operating point between them.

### Finding 1.6: Interoceptive AI — Life-Inspired Internal State Regulation (Nature Machine Intelligence, August 2026)
**Source:** https://www.nature.com/articles/s42256-026-01296-8

**Key insight:** Interoception — monitoring and regulating internal bodily states to maintain homeostasis — underwrites organism survival. Developing interoceptive AI requires **explicit factorization of state variables** representing internal and external environments, plus mathematical formalization of life-inspired properties governing internal-state dynamics. Internal states function as **universally available and intrinsically valuable contexts** — stable reference signals that modulate learning and behavior.

**New defect in NeoTrix:**
- **DEFECT-C653-6: No explicit internal/external state factorization.** The GWT workspace mixes specialist activations (internal state) with task content (external state) in the same `broadcast_history: Vec<String>`. There is no factorized representation where internal state (system health, fatigue, coherence) is separated from external state (task requirements, environment).
- **Missing:** An `InternalStateVector` that represents the system's own health, coherence, uncertainty, and fatigue as a factorized internal state, separate from task content.
- **Impact:** The system cannot distinguish between "I am performing well on this task" (internal state) and "this task is difficult" (external state). This prevents interoceptive regulation — adjusting behavior based on internal state rather than just task demands.

---

## 2. Control Theory Findings

### Finding 2.1: MPC with Policy-Guided Terminal Ingredients (arXiv:2609.02628, September 2026)
**Source:** https://arxiv.org/abs/2609.02628

**Key insight:** Conventional MPC relies on terminal costs/constraints derived from a steady state. Policy-guided MPC constructs terminal costs from a known sub-optimal control policy, eliminating the need for a steady state or reference trajectory. The terminal region is defined around a center determined by a rollout of the policy.

**New defect in NeoTrix:**
- **DEFECT-C653-7: SEAL pipeline assumes steady-state optimization.** The SEAL pipeline (exploration → distillation → self-test → absorption) operates as a fixed sequence that returns to a "baseline" state after each cycle. There is no policy-guided adaptation when the system has no natural steady state (e.g., during rapid task switching or novel domain entry).
- **Missing:** A policy-guided SEAL variant that constructs terminal costs from a known sub-optimal policy (the current best-performing configuration) rather than assuming return to a fixed baseline.
- **Impact:** During rapid adaptation, SEAL wastes cycles trying to return to a baseline that is no longer appropriate.

### Finding 2.2: Jacobian-Free Nonlinear MPC (iSCD-MPC) (Kamaldar, August 2026)
**Source:** https://arxiv.org/html/2608.15322

**Key insight:** iSCD-MPC factors nonlinear dynamics into pseudo-linear form using state- and control-dependent coefficients (SCDCs), replacing nonconvex optimization with a sequence of constrained LQPs. Zero plant Jacobians needed. The algorithm contracts to a unique fixed point with rate proportional to the state norm, and the suboptimality gap vanishes quadratically at the origin. Complexity scales O(ℓ) — linearly with horizon — matching iLQR but undercutting SQP's O(ℓ³).

**New defect in NeoTrix:**
- **DEFECT-C653-8: No complexity-aware optimization in evolution pipeline.** The SEAL pipeline's distillation step uses arbitrary heuristics for combining capabilities. There is no framework that guarantees the optimization scales linearly with problem size, nor any guarantee that the pipeline converges to a fixed point.
- **Missing:** An SCDC-style factorization of NeoTrix's evolution dynamics that guarantees O(ℓ) convergence rather than unbounded exploration.
- **Impact:** As the capability space grows, the SEAL pipeline's distillation step may scale super-linearly, making evolution impractical for large capability sets.

### Finding 2.3: MPPI-PID — Information-Theoretic Control (arXiv:2603.29499, March 2026)
**Source:** https://arxiv.org/abs/2603.29499v1

**Key insight:** MPPI-PID optimizes PID gains (not raw control inputs) via sampling-based model predictive path integral control. An **information-theoretic interpretation** unifies conventional MPPI with MPPI-PID: the update rule minimizes KL divergence between the current policy and an improved policy, with the PID structure reducing the optimization dimensionality from N×horizon to just the number of gains. Effective sample size analysis shows dramatically improved sample efficiency.

**New defect in NeoTrix:**
- **DEFECT-C653-9: No information-theoretic unification of GWT and evolution.** The GWT resonance mechanism (based on hamming distance in E₈ state space) and the SEAL evolution pipeline (based on heuristics) are disconnected. There is no information-theoretic framework that unifies them — no KL divergence minimization between current attention policy and improved policy.
- **Missing:** An `InformationTheoreticController` that treats GWT attention routing as a policy to be optimized via KL divergence minimization, analogous to how MPPI-PID optimizes PID gains.
- **Impact:** GWT attention routing evolves by ad-hoc resonance boosting rather than by principled information-theoretic optimization. The system cannot guarantee that attention routing is moving toward maximum information gain.

### Finding 2.4: PID Setpoint Gap Action — Adaptive Aggressiveness (Control Engineering, September 2026)
**Source:** https://www.controleng.com/pid-spotlight-part-32-shaping-controller-response-using-setpoint-gap-action/

**Key insight:** Setpoint gap control shapes controller response to be aggressive for large disturbances and relaxed for small ones. The gap allows a normally unstable controller to stabilize after responding to a large disturbance. For constraint control, aggressive tuning applies only on one side of the setpoint.

**New defect in NeoTrix:**
- **DEFECT-C653-10: No adaptive aggressiveness in GWT attention.** The GWT `threshold` field is a fixed f64 value. There is no mechanism to make the system aggressive for large attention shifts (e.g., novel task type) while relaxed for small perturbations (e.g., routine task variation).
- **Missing:** An `AdaptiveThreshold` that widens/narrows the activation threshold based on the magnitude of the attention demand signal — aggressive for novel stimuli, relaxed for routine ones.
- **Impact:** The system treats all attention demands with the same aggressiveness, either over-reacting to noise or under-reacting to genuine novelty.

### Finding 2.5: Economic MPC — Policy Beyond Setpoint Tracking (arXiv:2609.02628, September 2026)
**Source:** Same paper as 2.1

**Key insight:** Economic MPC extends control objectives from setpoint stabilization to economic performance metrics (cost reduction, energy efficiency, profit maximization). The cost function replaces/augments the traditional quadratic tracking error.

**New defect in NeoTrix:**
- **DEFECT-C653-11: No economic objective in SEAL evolution.** The SEAL pipeline optimizes for "distillation quality" and "absorption success" — essentially tracking error minimization. There is no economic objective that weighs the **cost** of evolution (compute, time, complexity) against the **benefit** (capability gain, performance improvement).
- **Missing:** An `EconomicObjective` function that replaces tracking-error minimization with a net-benefit metric: `benefit(capability_gain) - cost(evolution_effort)`.
- **Impact:** The system may invest disproportionate evolution effort in capabilities with marginal benefit, or neglect high-benefit evolution because the tracking-error metric doesn't capture economic value.

---

## 3. Systems Theory Findings

### Finding 3.1: Structure-Dynamics Coevolution via Variational Principles (npj Complexity, June 2026)
**Source:** https://www.nature.com/articles/s44260-026-00087-x

**Key insight:** How organized structure arises from dynamics — and how it reshapes dynamics — remains unresolved. Variational approaches (action principles, stochastic path formulations, information-theoretic methods, thermodynamic extremal principles) offer a unifying perspective based on **selection under constraints**. But existing frameworks are fragmented. The key challenge: systems continuously reorganize the space in which their dynamics unfold. Structure is not just the result of dynamics — it actively modifies the rules of the game.

**New defect in NeoTrix:**
- **DEFECT-C653-12: No variational principle for GWT-coevolution.** The GWT resonance mechanism treats the specialist module states as fixed and only adjusts activations. There is no variational principle that captures how the structure (which specialists exist, their resonance connections) co-evolves with the dynamics (activation patterns, attention routing).
- **Missing:** A `VariationalCoEvolution` framework that jointly optimizes structure (specialist set, resonance matrix) and dynamics (activation patterns) under a unified objective.
- **Impact:** The system cannot explain or predict how structural changes (adding a specialist) cascade through dynamic changes (attention routing patterns), because the two are optimized separately.

### Finding 3.2: Engineering Emergent Features via Gradient Descent (arXiv:2603.15631)
**Source:** https://arxiv.org/pdf/2603.15631

**Key insight:** A general optimization-based pipeline automates engineering of complex systems with emergent properties by **repurposing descriptive statistics as loss functions** and using gradient descent. Applied to Kuramoto oscillator systems, this produces non-trivial global properties including higher-order synergistic information, multi-attractor metastability, and meso-scale structures. The pipeline creates a closed loop that unifies description and design.

**New defect in NeoTrix:**
- **DEFECT-C653-13: No gradient-based optimization of emergent properties.** NeoTrix's E₈ hexagram reasoning, GWT attention, and VSA embedding produce emergent properties (phi, coherence, resonance clusters), but these are not treated as differentiable loss functions. The system cannot back-propagate from desired emergent properties (e.g., "I want high phi + moderate coherence") to the micro-scale parameters that produce them.
- **Missing:** A differentiable pipeline from emergent metrics (phi, coherence, resonance entropy) back to the specialist module parameters (activation thresholds, resonance strengths, hexagram assignments).
- **Impact:** The system cannot be engineered to produce specific emergent properties. It can only observe emergent properties after the fact and adjust heuristically.

### Finding 3.3: Constraint-Closure Order Parameter Framework (Zenodo, March 2026)
**Source:** https://doi.org/10.5281/zenodo.19284704

**Key insight:** A minimal framework for emergence across complex systems defines order parameters: Closure Density (C), Constraint Dependence Depth (D), Resonance Coherence (R), Feedback Gain (G), Constraint Path Entropy (H), and Perturbation Tolerance (E). These define the conditions under which self-sustaining organization arises and persists. Emergence is formalized as a phase structure in multidimensional parameter space, distinguishing disorganized, proto-closure, strongly emergent, rigid, and adaptive regimes.

**New defect in NeoTrix:**
- **DEFECT-C653-14: No order parameters for consciousness emergence.** NeoTrix tracks phi (IIT integration), coherence, and resonance entropy as individual metrics. But there is no unified order-parameter framework that defines the phase structure of consciousness emergence. The system cannot answer: "Am I in a proto-emergent regime? Am I approaching rigidity? Am I at the edge of adaptive capacity?"
- **Missing:** A `ConsciousnessOrderParameters` struct that computes (C, D, R, G, H, E) from the existing metrics and maps the system's current position in the multidimensional parameter space.
- **Impact:** The system cannot diagnose its own regime (frozen, adaptive, rigid) or predict phase transitions.

### Finding 3.4: Complex Systems vs Complex Adaptive Systems — Distinction Matters (Frontiers, May 2026)
**Source:** https://www.frontiersin.org/journals/complex-systems/articles/10.3389/fcpxs.2026.1808634/full

**Key insight:** All CAS are complex systems, but not all complex systems are adaptive. Adaptivity — the ability of components to change in response to experience — has practical consequences for modeling, evidence, and intervention. Collapsing these under a single vague notion of "complexity" risks disappointment and misdirected effort.

**New defect in NeoTrix:**
- **DEFECT-C653-15: NeoTrix conflates complex and adaptive.** The architecture treats all 14 specialist modules as equally adaptive (each has an `activation` field that updates). But some modules are complex (nonlinear interactions, emergence) without being adaptive (they don't learn from experience). The `PatternMatcher` module is a complex signal processor but doesn't adapt its matching strategy based on past successes.
- **Missing:** An explicit distinction between `ComplexModule` (nonlinear, emergent behavior) and `AdaptiveModule` (learns from experience). The `SpecialistModule` struct should have a trait bound distinguishing adaptive from non-adaptive modules.
- **Impact:** The system assumes all specialists can learn, wasting computation on attempting to adapt non-adaptive components, or failing to adapt components that should be adaptive.

### Finding 3.5: CEDAR — LLM-Agent-Orchestrated Complex System Discovery (arXiv:2608.06871, August 2026)
**Source:** https://arxiv.org/abs/2608.06871

**Key insight:** CEDAR uses LLM agents with Monte Carlo Tree Search to discover complex systems satisfying behavioral goals. The LLM Editor acts as a variation operator and LLM Judge as a fitness function — casting system discovery as evolutionary search guided by MCTS. This enables goal-directed discovery of emergent behavior while preserving solution diversity.

**New defect in NeoTrix:**
- **DEFECT-C653-16: No LLM-guided architecture search.** NeoTrix's architecture (14 specialists, E₈ hexagram assignments, resonance matrix) is manually designed. There is no mechanism for an LLM agent to propose, evaluate, and refine architectural variants — no MCTS over architecture space.
- **Missing:** An `ArchitectureSearch` module that uses LLM-based variation and evaluation to explore the space of possible specialist configurations, resonance matrices, and hexagram assignments.
- **Impact:** The architecture remains fixed at design time. It cannot discover that a different specialist configuration would better serve the current task landscape.

---

## 4. Cross-Cutting Defects

### DEFECT-C653-17: No Two-Timescale Architecture
All 2026 cybernetics and control theory papers converge on the necessity of **two-timescale architectures**: fast inner loops for routine regulation, slow outer loops for structural revision. NeoTrix has only one timescale — the tick-level GWT broadcast cycle. There is no slow loop for:
- Restructuring the specialist module set
- Revising SEAL pipeline objectives
- Adjusting the viability region
- Re-anchoring goals when drift is detected

### DEFECT-C653-18: No Feedback Loop Taxonomy
The 2026 literature identifies multiple distinct feedback channels (outcome reinforcement, termination reinforcement, evaluation weighting, navigation bias). NeoTrix treats all feedback as a single `activation *= 1.0 - rate` decay. There is no taxonomy distinguishing:
- Reinforcement feedback (strengthening successful pathways)
- Termination feedback (stopping ineffective processes)
- Evaluation feedback (reweighting signal priorities)
- Navigation bias (preferring previously traversed paths)

### DEFECT-C653-19: No Information-Theoretic Control Unification
The MPPI-PID paper (2.3) provides an information-theoretic framework that unifies policy optimization via KL divergence minimization. NeoTrix's GWT (resonance-based attention) and SEAL (heuristic evolution) operate in separate theoretical frameworks. There is no information-theoretic bridge that would allow treating attention routing as a policy to be optimized.

### DEFECT-C653-20: No Near-Critical Operation Management
The critical cybernetics paper (1.5) defines control as sensitivity management engineering — pinning systems near the percolation threshold. NeoTrix's entropy monitoring (`is_focused`/`is_distributed`) is binary and reactive. There is no active management of the system's proximity to criticality, no measurement of distance from the percolation threshold, and no mechanism to maintain near-critical operation.

---

## 5. Summary: What's New vs. Batch 652

| Batch 652 Finding | Batch 653 Status | New Defects |
|---|---|---|
| (1) GWT zero MI compression | CONFIRMED + EXTENDED | C653-9: No info-theoretic unification; C653-13: No gradient on emergent properties |
| (2) No channel capacity analysis | CONFIRMED | C653-17: No two-timescale architecture; C653-18: No feedback taxonomy |
| (3) No algorithm-hardware co-design | PARTIALLY ADDRESSED | C653-8: No complexity-aware optimization |
| (4) No MI-Entropy plane | CONFIRMED + NEW ANGLE | C653-5: No near-critical management; C653-20: No criticality manager |
| (5) No rate-distortion for HyperCube | CONFIRMED | C653-12: No variational coevolution; C653-16: No LLM architecture search |

**New cross-cutting themes identified:**
1. **Two-timescale architecture** is mandatory for adaptive systems (convergent finding across cybernetics + control theory)
2. **Information-theoretic unification** of attention routing and evolution is feasible and necessary
3. **Near-critical operation** management is a new control paradigm that NeoTrix completely lacks
4. **Order parameter frameworks** for consciousness emergence are now available (C653-14)
5. **Interoceptive AI** requires explicit internal/external state factorization (C653-6)

---

## 6. Sources Cited

1. Weinberger, N. (2026). Homeostasis and causal control. *Biology & Philosophy*, 41(18). https://doi.org/10.1007/s10539-026-10018-8
2. Agent Cybernetics (2026). The Agent Use of Agent Beings: Agent Cybernetics Is the Missing Science. arXiv:2605.10754. https://arxiv.org/html/2605.10754
3. Allostatic Control Systems (2026). Goal Governance in Changing Environments. arXiv:2607.21771. https://arxiv.org/abs/2607.21771
4. Amresh, K. (2026). Cognitive Cybernetics Technical Monograph — Series 1-07: Feedback Loops as Cognitive Structure. Zenodo. https://doi.org/10.5281/zenodo.19351679
5. Zhou, C. & Zhou, Z. (2026). Post-Cybernetics C: Critical Cybernetics. Zenodo. https://doi.org/10.5281/zenodo.20335909
6. Life-inspired interoceptive AI (2026). *Nature Machine Intelligence*. https://www.nature.com/articles/s42256-026-01296-8
7. MPC in complex systems (2026). *Journal of Engineering and Technology*. https://link.springer.com/article/10.1186/s44147-026-00969-w
8. Kamaldar, M. (2026). Iterative State- and Control-Dependent MPC. arXiv:2608.15322. https://arxiv.org/html/2608.15322
9. MPPI-PID Control (2026). arXiv:2603.29499v1. https://arxiv.org/abs/2603.29499v1
10. PID Setpoint Gap Action (2026). *Control Engineering*. https://www.controleng.com/pid-spotlight-part-32-shaping-controller-response-using-setpoint-gap-action/
11. Policy-Guided MPC (2026). arXiv:2609.02628. https://arxiv.org/abs/2609.02628
12. Georgiev, G.Y. (2026). Toward variational principles for structure-dynamics coevolution. *npj Complexity*. https://www.nature.com/articles/s44260-026-00087-x
13. Engineering emergent features (2026). arXiv:2603.15631. https://arxiv.org/pdf/2603.15631
14. Cantellow, J. (2026). Constraint-Closure Order Parameter Framework. Zenodo. https://doi.org/10.5281/zenodo.19284704
15. Complex vs Complex Adaptive Systems (2026). *Frontiers in Complex Systems*. https://www.frontiersin.org/journals/complex-systems/articles/10.3389/fcpxs.2026.1808634/full
16. CEDAR (2026). Agent-Orchestrated Tree Search. arXiv:2608.06871. https://arxiv.org/abs/2608.06871
17. Hostiunin, S. (2026). Feedback as a Fundamental Principle of System Viability. Zenodo. https://doi.org/10.5281/zenodo.20834624
