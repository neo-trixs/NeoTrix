# Iteration Batch 460 — Bayesian/Causal/Graphical Model Advances (2026)

**Date**: 2026-09-06  
**Domain**: Probabilistic Reasoning, Causal Inference, Graphical Model Inference

---

## Sources Cited

### Bayesian Networks & Belief Propagation

| # | Source | Year | Key Advance |
|---|--------|------|-------------|
| S1 | FasterCapital — "Bayesian Networks and Markov Models: A Probabilistic Alliance" | 2026-08 | Bayesian-Markov convergence: joint modeling of probabilistic relationships with conditional independence, MCMC for complex distributions, Dynamic BNs + HMMs for temporal reasoning |
| S2 | IEEE (11364448) — "Accelerating Belief Propagation with Task-Based Hardware" | 2026 | Hardware-accelerated BP via task-based parallelism; convergence of BP innovation with custom silicon |
| S3 | IEEE (10285934) — "Reliable Belief Propagation: Recent Theoretical and Practical" | 2024 | Optimization landscape analysis + message dynamics for loopy BP; theoretical convergence guarantees for approximate inference |
| S4 | Kirkley, Cantwell & Newman (Science Advances) — "Belief Propagation for Networks with Loops" | 2021 (foundational) | Loop-aware BP: region-based approximations correcting standard BP on loopy graphs; exact on short loops up to configurable max length |
| S5 | Cheng, Dai & Sun (arXiv:2606.05042) — "In-Context Graphical Inference" | 2026-06 | ICG-I: autoregressive sequence modeling for marginal inference; Tensor Train compression of intermediate factors; Graph Transformer with dynamic topology encoding; Dirichlet output for calibrated uncertainty |

### Causal Inference & Do-Calculus

| # | Source | Year | Key Advance |
|---|--------|------|-------------|
| S6 | ICML 2026 — "Unveiling the Structure of Do-Calculus Reasoning via Derivation Graphs" (arXiv:2606.03719) | 2026 | Derivation graphs for do-calculus: explicit characterization of all equivalent transformations; R1∘R2=R2∘R1 commutativity; max 4 rule applications to reach any equivalent expression; systematic enumeration of estimators |
| S7 | Wang et al. (arXiv:2511.21516v2) — "Causal Inference: A Tale of Three Frameworks" | 2026-02 | Unified workflow: Potential Outcomes ↔ SEM ↔ DAG; po-calculus as conditional independence reformulation of do-calculus; SWIG-based identification; multiply robust estimators |
| S8 | EmergentMind — "Structural Causal Models: A Primer" (updated) | 2026-02 | GNN-SCM connection: any SCM expressible via GNN parameterization; identification-theoretic semantics preserved; observational/interventional separation is fundamental |
| S9 | ACM (10.5555/3844975.3845220) — "A Stronger Calculus of Intervention for Max-Linear Bayesian Networks" | 2025 | Max-linear BNs encode strictly more conditional independence than general SCMs; relevant to extreme event inference |
| S10 | Stanford Encyclopedia — Do-calculus supplement (Huang & Valtorta 2006; Shpitser & Pearl 2006) | 2006/ongoing | Completeness proofs: do-calculus is complete for post-intervention probability; Zhang 2008 develops structure-free weaker versions |

### Graphical Model Inference

| # | Source | Year | Key Advance |
|---|--------|------|-------------|
| S11 | MacKinlay et al. — "Gaussian Ensemble Belief Propagation" (ICLR) | 2025 | Gaussian Ensemble BP: efficient inference in high-dimensional systems via ensemble averaging of BP |
| S12 | Li & Wu (IEEE 2024) — "Distributed Inference With Variational Message Passing" | 2024 | VMP convergence under different schedules on Gaussian graphical models; asynchronous distributed inference |
| S13 | Jiang, Yuan & Guo (arXiv:2401.03626) — "Hybrid Vector Message Passing" | 2024 | HVMP: integrates EP + VMP via variational free energy; vector/matrix messages (not scalar); loop-free BN topology |
| S14 | PyAutoLabs/HowToFit — Expectation Propagation tutorial | 2026-05 | EP for practical approximate inference: moment matching + factor graph partitioning |
| S15 | Lin, Hubacher & Khan (arXiv:1803.05589) — "VMP with Structured Inference Networks" | 2018 (foundational) | Structured inference networks incorporating graphical model structure into VAE inference; natural-gradient VMP |

---

## Defects Found

### DEFECT-460-1: CausalGraph Lacks Do-Calculus Intervention Semantics

**Source**: S6 (ICML 2026), S10 (completeness proofs)  
**File**: `neotrix-core/src/unified/core/nt_core_e8/abduction/causal_graph.rs:1-80`  
**Severity**: HIGH

**Finding**: NeoTrix's `CausalGraph` implements a basic directed graph with `find_causes`/`find_effects` traversal, but has **no do-operator semantics**. The ICML 2026 derivation graph paper (S6) proves that any equivalent interventional expression is reachable within at most 4 rule applications of R1/R2/R3. NeoTrix's causal graph can only do observational queries — it cannot express P(Y|do(X)) vs P(Y|X), nor apply graph surgery (removing incoming edges to intervened nodes).

**Gap**: The `CausalGraph` has no `intervene(node)` method that removes incoming edges. The `CausalEdge` struct has no graphical criteria for when observation vs intervention applies. The `find_causes` method is a flat edge scan, not a d-separation query.

**Suggestion**: 
1. Add `fn intervene(&self, node_id: usize) -> CausalGraph` that clones the graph with incoming edges to `node_id` removed.
2. Add `fn do_query(&self, target: usize, intervention: usize) -> Option<f64>` implementing the backdoor/front-door criteria.
3. Use derivation graphs (S6) to enumerate equivalent expressions and select the one with lowest variance estimator.

---

### DEFECT-460-2: Bayesian Experiment Design Has No Posterior Concentration Check (M-open)

**Source**: S6, S7  
**File**: `nt_core_hcube::bayesian_experiment` (referenced in iteration_batch_375.md, iteration_batch_365.md)  
**Severity**: MEDIUM

**Finding**: The `VoIConfig` and Bayesian experiment design in `nt_core_hcube` uses flat VoI-based scoring (`IG + 0.5×VoI`) without M-open checks. The ICML 2026 work (S6) shows that equivalent expressions have "significantly different statistical properties" — a single scalar VoI misses the structure of the equivalence class. S7's three-frameworks paper emphasizes that identification (Step 3 in the causal workflow) must precede estimation, but NeoTrix's `rank_then_commit` skips identification entirely.

**Gap**: No mechanism to: (1) check if posterior concentrates on < threshold of hypotheses (M-open), (2) enumerate equivalent expressions of a causal query before scoring, (3) select estimators based on variance rather than just information gain.

**Suggestion**: Add `MOpenCheck` that: (1) computes posterior entropy, (2) if entropy < threshold, expands hypothesis space (as in iteration_batch_377's P3 VoI design), (3) uses derivation graph enumeration (S6) to find lowest-variance equivalent expression before committing.

---

### DEFECT-460-3: No Loop-Aware Belief Propagation in KB Inference

**Source**: S4 (Kirkley et al.), S5 (ICG-I), S11 (Gaussian Ensemble BP)  
**File**: KB graph query layer (referenced in iteration_batch_403.md)  
**Severity**: HIGH

**Finding**: NeoTrix's KB has nodes, edges, and BM25 index, but graph queries use flat node/edge lookups. The 2021 Kirkley paper (S4) proves that standard BP fails on loopy graphs but can be corrected with region-based approximations. The 2026 ICG-I paper (S5) shows that iterative methods (LBP, variational) fail on frustrated topologies because they abandon sequential elimination structure. Gaussian Ensemble BP (S11) provides efficient high-dimensional inference via ensemble averaging.

**Gap**: NeoTrix has no belief propagation implementation at all. When KB nodes form loopy graphs (which they do — module A → module B → module C → module A health dependencies), there is no principled way to compute marginal posteriors. The `bayesian_experiment.rs` operates on flat hypothesis lists without graph structure awareness (confirmed in iteration_batch_365.md and iteration_batch_403.md).

**Suggestion**: 
1. Implement Gaussian Ensemble BP (S11) as the primary KB graph inference engine.
2. For loopy subgraphs, use region-based correction (S4) with configurable max loop length.
3. For high-treewidth graphs, use ICG-I's Tensor Train compression (S5) to avoid exponential blowup.
4. Expose a `fn marginal_posterior(node_id, evidence: &[(usize, f64)]) -> Distribution` API.

---

### DEFECT-460-4: Causal Coherence Metric Is Heuristic, Not Model-Based

**Source**: S7 (three frameworks), S8 (GNN-SCM connection)  
**File**: `nt_mind_evolution_daemon.rs:494-503`  
**Severity**: MEDIUM

**Finding**: `compute_causal_coherence()` computes a weighted sum of fix_rate and cycle_factor: `0.3 + fix_rate * 0.4 + cycle_factor * 0.3`. This is a heuristic with no connection to actual causal structure. S7's three-frameworks paper shows that causal coherence requires structural assumptions (DAG/SWIG/SEM) validated against data. S8 shows GNNs can express any SCM — meaning the evolution daemon should be using a learned causal model, not a hand-crafted linear combination.

**Gap**: The `causal_coherence_history` (line 181) tracks a number disconnected from the `CausalGraph` in `causal_graph.rs`. There is no mechanism to: (1) learn structural equations from evolution trace data, (2) validate causal assumptions via d-separation tests, (3) compute interventional (do) predictions about evolution outcomes.

**Suggestion**: Replace heuristic with: (1) maintain a structural causal model over evolution variables (fix_rate, cycle_count, phi_reward, module_health), (2) use PC algorithm for causal discovery from evolution trace data, (3) compute causal coherence as the probability that the current DAG explains observed transitions (posterior model probability).

---

### DEFECT-460-5: AutobiographicalIndex Causal Chain Has No Intervention Semantics

**Source**: S6 (do-calculus derivation graphs), S7 (po-calculus)  
**File**: `autobiographical_index.rs:21-172`  
**Severity**: MEDIUM

**Finding**: `AutobiographicalIndex` extracts causal chains from entries and maintains a `CausalGraph`, but all queries are observational: `get_causal_context` does BFS on the graph, and `causal_connectivity` counts edges. There is no do-operator: "what would happen to memory retrieval if I intervened on entry X's importance?" is not expressible.

**Gap**: The `extract_causal_chain` method (line 335) builds correlation-based links (`strength` based on temporal proximity and co-occurrence), not interventional links. The `causal_strength_threshold` (line 109) filters by correlation strength, not by causal identifiability. No mechanism to distinguish genuine causal links from confounded correlations.

**Suggestion**: 
1. Add `fn do_intervene(&self, entry_id: &str, attribute: &str, value: f64) -> Vec<AutobiographicalEntry>` that recomputes retrieval with graph surgery.
2. Use po-calculus (S7) to reduce do-expressions to conditional independence checks on SWIGs.
3. Track identification status: for each causal chain, record whether the effect is identifiable, under what assumptions, and via which criterion (backdoor/frontdoor/do-calculus).

---

### DEFECT-460-6: No Variational Message Passing for GWT Attention Routing

**Source**: S12 (distributed VMP), S13 (HVMP), S15 (structured inference networks)  
**File**: GWT attention routing layer  
**Severity**: HIGH

**Finding**: NeoTrix's GWT broadcasts salient information across specialist modules with "resonance-based routing." This is a hard binary: either a module receives the broadcast or it doesn't. S12 shows that VMP with asynchronous schedules enables distributed inference across Gaussian graphical models. S13's HVMP integrates EP+VMP via variational free energy, using vector/matrix messages instead of scalars — enabling loop-free inference on bilinear factorizations. S15 shows structured inference networks can incorporate graphical model structure into the routing mechanism itself.

**Gap**: GWT has no probabilistic message content — broadcasts are opaque signals, not parameterized messages with uncertainty. There is no mechanism for modules to: (1) pass beliefs (not just signals) to each other, (2) reach consensus via message passing iterations, (3) quantify uncertainty in their接收 of broadcast information.

**Suggestion**:
1. Define GWT messages as exponential-family distributions (not raw signals).
2. Implement VMP schedule: each specialist module updates its local posterior given incoming messages, then sends updated messages.
3. Use HVMP's vector message format for multi-attribute module state.
4. Add convergence detection: stop when KL divergence between consecutive message rounds < ε.

---

### DEFECT-460-7: No Equivariance in PerceptionBridge Sensory Transformations

**Source**: S5 (ICG-I dynamic topology), S11 (Gaussian Ensemble BP)  
**File**: `perception_bridge.rs` (L2-L5 bridge)  
**Severity**: LOW-MEDIUM (partially addressed in iteration_batch_359.md but reinforced by 2026 advances)

**Finding**: ICG-I (S5) uses dynamic shortest-path distance encodings that track evolving fill-in topology — the graph representation changes at each inference step. Gaussian Ensemble BP (S11) uses ensemble averaging that is invariant to graph labeling permutations. NeoTrix's PerceptionBridge passes raw features without equivariance constraints, as noted in iteration_batch_359.md.

**Gap**: Reinforced: the 2026 advances show that *dynamic* topology tracking (not just static equivariance) is essential. PerceptionBridge should track how the L2→L5 connection topology changes as sensory context evolves.

**Suggestion**: Add dynamic topology tracking to PerceptionBridge: maintain a sliding window of recent connection patterns and use Gaussian Ensemble BP (S11) to marginalize over topology uncertainty.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 15 |
| Defects found | 7 |
| HIGH severity | 3 (DEFECT-460-1, -3, -6) |
| MEDIUM severity | 3 (DEFECT-460-2, -4, -5) |
| LOW-MEDIUM severity | 1 (DEFECT-460-7) |

### Cross-Cutting Theme

The 2026 advances converge on a single insight: **probabilistic reasoning requires structured message passing with uncertainty quantification, not flat heuristic scoring**. NeoTrix's current design uses:
- Flat VoI scoring instead of derivation-graph-guided estimator selection (DEFECT-460-2)
- Heuristic causal coherence instead of model-based inference (DEFECT-460-4)
- Correlation-based causal chains instead of interventional queries (DEFECT-460-5)
- Opaque broadcasts instead of parameterized belief messages (DEFECT-460-6)

The ICML 2026 derivation graph paper (S6) is the single highest-impact finding: it provides a constructive method to enumerate all equivalent causal expressions in ≤4 rule applications, enabling principled estimator selection. This should be the first integration target.
