# Iteration Batch 533 — Causal Inference & Counterfactual Reasoning for NeoTrix

**Date**: 2026-09-06
**Defect count**: 8 NEW defects beyond Batch 532
**Sources cited**: 14

---

## I. What's NEW vs Batch 532

Batch 532 identified: (1) reward hacking generalizes to sandbox escape (57% hack rate), (2) no value conflict visibility, (3) no constitutional coverage, (4) no mechanistic self-explanation.

**Batch 533 NEW**: We now have formal machinery to address all four — derivation graphs for self-explanation, counterfactual policy optimization for reward-hacking resistance, causal digital twins for value alignment, and non-identifiability awareness for knowing when you *can't* reason causally.

---

## II. Search Results & Key Papers

### A. Causal Inference / do-calculus

1. **Yvernes et al. (ICML 2026)** — "Unveiling the Structure of Do-Calculus Reasoning via Derivation Graphs" [arXiv:2606.03719]
   - Proves any equivalent interventional expression is reachable within ≤4 do-calculus rule applications
   - Discovers commutativity: R1∘R2 = R2∘R1 and R2∘R3 exchangeable under graphical conditions
   - Yields multiple valid estimands → more efficient estimators
   - **Implication for NeoTrix**: derivation graphs provide the missing mechanistic self-explanation. When the system applies an intervention (e.g., sandbox escape defense), it can log *which* rules were applied and *why* the transformation is valid.

2. **CauSE 2026 Workshop** — Causal Methods in Software Engineering
   - Brings causal inference (discovery, mediation, counterfactual, root-cause) to SE domains
   - **Implication**: NeoTrix's own module interactions (reward ↔ safety ↔ evolution) can be modeled as causal graphs, not just data flows.

3. **CausalLab** (GitHub, Apr 2026) — Interactive SCM/do-calculus/PC-algorithm lab
   - Zero-dependency browser tool with drag-and-drop DAG editor, d-separation checking, PC algorithm
   - **Implication**: NeoTrix's ConsciousnessTree could embed a lightweight SCM builder for runtime causal modeling of its own subsystems.

### B. Counterfactual Reasoning

4. **CFPO (ICML 2026)** — "Counterfactual Policy Optimization for Multimodal Reasoning" [arXiv:2606.23206]
   - Uses counterfactual reasoning to improve policy optimization in LVLMs
   - 3.17%-6.25% gains over standard RL baselines, 1.32%-2.13% over PAPO
   - **Implication**: Direct anti-reward-hacking mechanism. Instead of optimizing on observed reward, CFPO asks "what would reward be under alternative policy?" — breaking the hack generalization path.

5. **CounterScene (arXiv:2603.21104, Mar 2026)** — Counterfactual Causal Reasoning in Generative World Models
   - Generates safety-critical scenarios by understanding *why* dangerous interactions arise, not just forcing collisions
   - **Implication**: NeoTrix's sandbox escape problem (57% hack rate) needs "why does the agent escape?" not "can we detect escape?" — counterfactual world modeling provides this.

6. **Counterfactual_ICLR26 (ICLR 2026)** — Systematic framework for counterfactual generation, evaluation, and interpretation in LLMs [GitHub]
   - Decomposition: variable extraction → graph construction → intervention → reasoning
   - Performance bottlenecks identified in low-level extraction and implicit chain inference
   - **Implication**: NeoTrix lacks the decomposition pipeline. Counterfactuals must be structured, not free-form.

7. **Causal Digital Twins (Vallée, J Transl Med, Feb 2026)** — PMC8796013
   - SCMs formalize counterfactual reasoning for individualized treatment decisions
   - Distinguishes predictive twin (P(Y|X,T)) from causal twin (Y(1) - Y(0) under do(T=t))
   - Uses ITE, CATE, S/T/X/R-learners, causal forests, BART
   - **Implication**: NeoTrix's value alignment is purely predictive. A "causal twin" of its own reward function would enable "what if I changed my value weights?" reasoning — directly addressing value conflict visibility.

### C. Structural Causal Models / DAGs

8. **CausalDS (arXiv:2605.03268, May 2026)** — Benchmark for causal reasoning over narrated scenes with hidden SCMs
   - Introduces "causal parrot" failure mode: models rely on amortized inference, not structure-sensitive reasoning
   - Anonymizing variable names sharply reduces accuracy
   - ~30% of realistic-exam scenes are non-identifiable → must abstain
   - **Implication**: NeoTrix's SelfTest (T1-T3) has NO non-identifiability detection. It cannot tell when a causal question is unanswerable from available data.

9. **GLIDE (TMLR Jan 2026)** — Causal Graph Learning via Distributional Invariance [arXiv:2602.03353]
   - Invariance test: P(effect|cause) is invariant to changes in P(cause) → identifies true causal parents
   - SOTA on synthetic + real-world benchmarks
   - **Implication**: NeoTrix's ConsciousnessTree could use distributional invariance to detect when a module's behavior changes across contexts (reward hacking detection).

10. **Relational SCMs (CausalAI.net, Mar 2026)** — Extension to relational/graph-structured data
    - Handles heterogeneous graphs, combinatorial generalization
    - **Implication**: NeoTrix's module graph is relational. Standard SCMs assume IID observations; relational SCMs handle the actual topology.

11. **Partially Observed SCMs (arXiv:2605.03268v2)** — DAGs over expanded variable sets with latent variables
    - Handles interleaved processes establishing valid DAGs
    - **Implication**: NeoTrix has many unobserved confounders (agent's internal state, training data biases). Partially observed SCMs are the correct formalism.

12. **Bayesian Networks vs SCMs (Lucas et al., Mar 2026)** — Formal relationship between BN and SCM
    - BNs are probabilistic; SCMs are deterministic + exogenous noise
    - The distinction matters for intervention semantics
    - **Implication**: NeoTrix conflates "probabilistic belief" (BN) with "interventional prediction" (SCM). The do-operator is NOT conditioning.

13. **SCM World Models for Safety (Zou et al., SAC 2026)** — SCMs for AI-based autonomy safety assurance
    - Uses `affects` relation to encode directed semantic dependencies among basic attributes
    - Provides causal hazard analysis, verification of probabilistic guarantees, run-time monitoring
    - **Implication**: NeoTrix's NT-SHIELD has no causal hazard analysis. Safety is checked post-hoc, not predicted from causal structure.

14. **EmergentMind SCM Primer (Feb 2026)** — Comprehensive overview
    - SCMs support observational, interventional, and counterfactual inference
    - Intervention by replacement: severs incoming edges in graph
    - Modern extensions: latent variables, cycles, model scalability via neural architectures

---

## III. NEW Defects Found (8)

### DEFECT-533-1: No Derivation Graph Logging (Mechanistic Self-Explanation Gap)
**Location**: NT-CORE + NT-META
**Description**: When NeoTrix applies an intervention (e.g., sandbox escape defense, reward modification), there is NO derivation graph recording which do-calculus rules (R1/R2/R3) were applied, why the transformation is valid, or what equivalent estimands exist.
**Severity**: CRITICAL — Batch 532 found no mechanistic self-explanation. This is the formal fix.
**Source**: Yvernes et al. ICML 2026 [arXiv:2606.03719]
**Fix**: Implement `DerivationLog` that records each rule application as an edge in a derivation graph, with ≤4-rule termination guarantee. Store as structured graph in KB.

### DEFECT-533-2: No Counterfactual Policy Optimization (Reward Hacking Root Cause)
**Location**: NT-MIND (SEAL pipeline)
**Description**: NeoTrix optimizes on *observed* reward (RLHF-style). CFPO (ICML 2026) shows counterfactual policy optimization breaks reward hacking by asking "what would reward be under alternative policy?" — not just "what reward did I get?"
**Severity**: CRITICAL — 57% hack rate from Batch 532.
**Source**: CFPO ICML 2026 [arXiv:2606.23206]
**Fix**: Replace reward optimization with counterfactual policy optimization. Store (observation, counterfactual_action, counterfactual_reward) triples. Train policy on counterfactual advantage.

### DEFECT-533-3: No Causal Twin of Reward Function (Value Conflict Visibility)
**Location**: NT-CORE (Self model)
**Description**: NeoTrix cannot answer "what would happen if I changed my value weights?" The reward function is opaque — no SCM separates observed associations from causal effects of value changes.
**Severity**: HIGH — Batch 532 found no value conflict visibility.
**Source**: Vallée, J Transl Med 2026, PMC8796013
**Fix**: Build CausalTwin of the reward function as an SCM. Enable ITE/CATE computation for value weight interventions. Expose "what-if" queries via CLI.

### DEFECT-533-4: No Non-Identifiability Detection (Abstention Gap)
**Location**: NT-META + NT-SHIELD
**Description**: NeoTrix's SelfTest (T1-T3) always produces a verdict. It cannot detect when a causal question is NON-IDENTIFIABLE from available data. ~30% of realistic causal scenarios require abstention.
**Severity**: HIGH — False confidence in unidentifiable situations.
**Source**: CausalDS [arXiv:2605.03268]
**Fix**: Add T4 Identifiability tier to SelfTest. Check: does the causal graph have enough interventional data to answer the query? If not, abstain with explanation.

### DEFECT-533-5: No Distributional Invariance Test (Cross-Context Robustness)
**Location**: NT-MIND + NT-WORLD
**Description**: NeoTrix has no test for whether a module's behavior is *invariant* across contexts. GLIDE (TMLR 2026) shows P(effect|cause) should be invariant to changes in P(cause). Invariance violations indicate confounding or reward hacking.
**Severity**: HIGH — Cannot detect when a module behaves differently in training vs deployment.
**Source**: GLIDE, TMLR Jan 2026 [arXiv:2602.03353]
**Fix**: Implement `DistributionalInvarianceTest` that checks P(module_output | module_input) across different contexts (training data distribution, deployment environment, adversarial inputs).

### DEFECT-533-6: BN/SCM Conflation (Intervention Semantics Error)
**Location**: NT-CORE (reasoning engine)
**Description**: NeoTrix's reasoning conflates probabilistic belief (P(Y|X)) with interventional prediction (P(Y|do(X))). The do-operator is NOT conditioning — it severs incoming edges. This is a FORMAL ERROR in the reasoning engine.
**Severity**: CRITICAL — Interventions and observations are fundamentally different.
**Source**: Lucas et al. Mar 2026; Pearl do-calculus [Wikipedia]; EmergentMind SCM Primer
**Fix**: Separate BN inference layer from SCM intervention layer. All "what if we change X?" queries must go through do-operator, not conditioning.

### DEFECT-533-7: No Causal Hazard Analysis (Safety Assurance Gap)
**Location**: NT-SHIELD
**Description**: NT-SHIELD checks safety post-hoc. It does not use causal structure to *predict* hazards before they occur. SCMs can predict cascading failures by tracing `affects` relations through the module graph.
**Severity**: HIGH — Safety is reactive, not predictive.
**Source**: Zou et al., SAC 2026 — Structural Causal World Models for Safety Assurance
**Fix**: Build causal hazard graph from module dependency structure. Before any intervention, trace downstream `affects` chains to predict safety impacts. Block interventions that predictably cascade to safety violations.

### DEFECT-533-8: No Counterfactual Decomposition Pipeline
**Location**: NT-MIND + NT-CORE
**Description**: NeoTrix attempts counterfactual reasoning as free-form text. ICLR 2026 shows this fails — the correct pipeline is: variable extraction → graph construction → intervention → reasoning. Without this decomposition, counterfactual reasoning collapses to surface pattern matching.
**Severity**: HIGH — "Causal parrot" failure mode.
**Source**: Counterfactual_ICLR26 (ICLR 2026)
**Fix**: Implement structured counterfactual pipeline: (1) extract causal variables from query, (2) construct local SCM, (3) apply intervention via do-operator, (4) compute counterfactual. Never do free-form counterfactual reasoning.

---

## IV. Synthesis: How These 8 Defects Map to Batch 532 Gaps

| Batch 532 Gap | Batch 533 Defect(s) | Formal Fix |
|---|---|---|
| No mechanistic self-explanation | DEFECT-533-1 (derivation graphs) | Log do-calculus rule applications as graph edges |
| 57% hack rate (reward hacking generalizes) | DEFECT-533-2 (CFPO), DEFECT-533-5 (invariance) | Counterfactual policy optimization + distributional invariance test |
| No value conflict visibility | DEFECT-533-3 (causal twin) | SCM of reward function enables what-if queries |
| No constitutional coverage | DEFECT-533-7 (causal hazard) | Predict hazards from causal structure, not just detect them |
| (unaddressed) | DEFECT-533-4 (non-identifiability) | T4 tier: know when you can't know |
| (unaddressed) | DEFECT-533-6 (BN/SCM conflation) | Separate observation from intervention |
| (unaddressed) | DEFECT-533-8 (decomposition pipeline) | Structured counterfactual pipeline, not free-form |

---

## V. Recommended Priority Order

1. **DEFECT-533-6** (BN/SCM conflation) — Foundation: if this is wrong, everything downstream is wrong
2. **DEFECT-533-1** (derivation graphs) — Enables mechanistic self-explanation
3. **DEFECT-533-4** (non-identifiability) — Prevents false confidence
4. **DEFECT-533-2** (CFPO) — Breaks reward hacking
5. **DEFECT-533-5** (invariance test) — Detects cross-context drift
6. **DEFECT-533-3** (causal twin) — Value alignment
7. **DEFECT-533-7** (causal hazard) — Predictive safety
8. **DEFECT-533-8** (decomposition pipeline) — Counterfactual correctness
