# Iteration Batch 427 — Research Loop: Agent-Based / System Dynamics / Monte Carlo

**Date**: 2026-09-06
**Focus**: ABM frameworks, SD modeling, MC variance reduction
**Sources**: 8 web sources + 3 grep scans of neotrix-core

---

## Sources Cited

1. **Mesa 4.0a0** (2026-03-14, Zenodo) — Agent-based modeling in Python. Introduces timed agent actions (Action system with interrupt/resume/progress), DiscreteEventSimulator for next-event time progression, EventGenerator pause/resume, exception hierarchy, abstract DiscreteSpace. [mesa.readthedocs.io](https://mesa.readthedocs.io/)
2. **Vandin et al.** (2026-07-20, alphaXiv 2607.17948) — "Towards Agentic ABMs": LLM-driven agents inside Mesa ABMs via tool-calling. Smaller LLMs (<2B) fail semantic classification; 4B+ pass. Computational overhead: ~7.3s per LLM decision vs ~0.1s for symbolic. Statistical model checking confirms behavioral preservation but massive cost.
3. **D2D — Diagrams-to-Dynamics** (2026-06-18, BMC Medicine) — Converts causal loop diagrams to exploratory SD models under uncertainty with minimal user input. Python package + web app. Shows leverage points distinguishable without full calibration.
4. **CLD+ notation** (2026-01-20, Int'l J. Social Research Methodology) — Enhanced causal loop diagram that labels variable types (stock/rate/auxiliary), link types (material/information), delays as first-class nodes. Structurally equivalent to SFD — enables automatic CLD→SFD conversion.
5. **Code-First SD Methodology** (2026 ISDC) — LLM-generated Python code as primary SD representation. Validatable Model Standard (VMS) ensures machine-parsable, auditable, structurally recoverable models. Diagrams derived from code, not constructed manually.
6. **EM2C — Entropic Mirror Monte Carlo** (2026-02, arXiv 2602.03165) — Adaptive importance sampling combining entropic mirror descent with Markovian dynamics. Geometric convergence to target distribution. Handles multimodal high-dimensional targets. Delayed weighting enables rapid resampling.
7. **ML-Enhanced IS for Distortion Risk Measures** (2026-07-08, European Actuarial Journal) — Combines importance sampling with ML approximations (Gaussian SVM, k-NN, linear SVM) for black-box models. Iterative refinement for extreme tails. RMSE reduction up to 8.8x vs crude MC.
8. **Active Learning IS for Variance Reduction** (2026-04-30, IOPScience) — Gaussian Process Sampler proposes optimal sampling distributions based on loss heuristics. Data-driven IS that infers physically meaningful distributions automatically. TCP interface for compiled simulation code.
9. **VMC Gradient IS** (2026-02-10, IOPScience) — Adaptive overdispersed importance sampling for variational Monte Carlo. One-parameter family of distributions with gradient-based tuning. Up to 100x cost reduction for peaked quantum chemistry wavefunctions.
10. **Multi-Index IS for MV-SDEs** (2026-08-29, J. Computational & Applied Math) — Multi-index Monte Carlo + IS for McKean-Vlasov equations. Reduces complexity from O(TOL⁻⁴) to O(TOL⁻²(log TOL⁻¹)²). Applicable to opinion dynamics, collective behavior, pedestrian dynamics.

---

## Defects Identified in NeoTrix

### DEFECT-427-ABM-1: Zero Agent-Based Modeling Infrastructure
- **Evidence**: Grep for `agent.based|ABM|AgentSet|agent.scheduler|AgentActivation` returns 0 results in neotrix-core. Only "Mesa" references are GPU renderer strings in stealth fingerprinting (`nt_shield_stealth_net/system_fingerprint.rs:315`, `nt_world_crawl/stealth.rs:455`).
- **Gap**: NT-CORE's E8 reasoning + GWT attention routing model individual cognitive agents but has no ABM framework to simulate emergent multi-agent behaviors. The SEAL pipeline (NT-MIND) models agent evolution but cannot run population-level simulations.
- **Impact**: Cannot simulate agent swarms, model cognitive architecture populations, or test self-evolution dynamics at scale. The "Dark Forest" axiom (modules must connect) is violated — SEAL evolution logic is untestable at population level.

### DEFECT-427-ABM-2: No Timed/Event-Driven Agent Actions
- **Evidence**: Mesa 4.0a0 introduces `Action` system with interrupt/resume/progress tracking, `DiscreteEventSimulator` for next-event time progression, and `EventGenerator` pause/resume. NeoTrix has no equivalent.
- **Gap**: NT-ACT's tool orchestration is purely tick-based. No concept of long-running actions with progress, interruption, or resumption. This is critical for NT-PHYSICAL (sensors/motors operate on real-time event schedules, not fixed ticks).
- **Impact**: NT-PHYSICAL motor commands, NT-SHIELD stealth operations, and NT-ACT MCP tool calls cannot express duration, interruption, or partial completion.

### DEFECT-427-ABM-3: No LLM-as-Agent Performance Tax Modeling
- **Evidence**: Vandin et al. show LLM-agent decisions cost ~7.3s each (vs ~0.1s symbolic). Even one LLM agent in a model adds ~2.5 hours per 20 simulations.
- **Gap**: NeoTrix uses LLMs (NT-IO) throughout but has no model of the computational cost of LLM-agent decisions within simulation loops. The `BayesianExperimentDesign` (nt_core_hcube) assumes cheap sampling.
- **Impact**: SEAL pipeline experiments involving LLM-in-the-loop agents will have unpredictable wall-clock time. No budget-aware scheduling for LLM agent decisions.

### DEFECT-427-SD-1: No System Dynamics / Stock-Flow Modeling
- **Evidence**: Grep for `stock.and.flow|system.dynamics|causal.loop` returns 0 relevant results. Only one tangential mention in seeds.rs about "root system dynamics."
- **Gap**: NT-MIND models self-evolution (SEAL pipeline) as a directed graph, but has no stock-and-flow representation of capability accumulation, fatigue buildup, or knowledge degradation. These are natural stocks/flows.
- **Impact**: Cannot model feedback loops in self-evolution (e.g., more learning → faster evolution → more fatigue → slower learning). ConsciousnessTree health signals cannot be modeled as feedback systems.

### DEFECT-427-SD-2: No Causal Loop Diagram Representation
- **Evidence**: D2D (2026) shows CLDs can be converted to SD models under uncertainty. CLD+ (2026) makes this conversion automatic.
- **Gap**: NeoTrix's E8 hexagram reasoning encodes causal relationships implicitly but has no explicit CLD/CLD+ representation. No way to visualize or validate feedback loop structure.
- **Impact**: Architecture decisions (e.g., "adding NT-FEEL improves NT-CORE reasoning") cannot be validated as feedback loops. The ConsciousnessTree's 11 branches have implicit interactions but no explicit causal model.

### DEFECT-427-SD-3: No LLM-to-Simulation Pipeline
- **Evidence**: Code-First SD (2026 ISDC) shows LLMs can generate validatable Python SD models. VMS ensures structural recoverability.
- **Gap**: NT-MIND's SEAL pipeline uses LLMs for distillation but cannot generate executable SD models from natural language problem descriptions. No VMS-like validation of generated models.
- **Impact**: Cannot auto-generate system dynamics models from domain knowledge. Manual model construction remains a bottleneck.

### DEFECT-427-MC-1: No Importance Sampling / Variance Reduction Framework
- **Evidence**: Grep finds only 2 MC-related references: MCTS in TTC (`nt_core_ttc.rs:409`) and GRPO importance sampling ratio (`nt_memory_resource_ingest.rs:996`). Neither implements proper IS variance reduction.
- **Gap**: `BayesianExperimentDesign` uses brute-force MC sampling (64 samples per experiment by default). No importance sampling, stratified sampling, or control variates. The VoI computation `≈ E[prior→posterior KL]` is variance-heavy for rare-event hypotheses.
- **Impact**: Bayesian experiment design is inefficient. For rare hypotheses (low prior), the posterior estimation is noisy. The M-open check's adequacy metric is unreliable with high-variance estimates.

### DEFECT-427-MC-2: No Adaptive/Machine-Learning-Enhanced Sampling
- **Evidence**: EM2C (2026) shows geometric convergence via entropic mirror descent + Markovian dynamics. ML-enhanced IS achieves up to 8.8x RMSE reduction. Active-learning IS uses Gaussian Processes.
- **Gap**: NeoTrix has no adaptive proposal distribution, no ML-guided sampling, and no variance reduction optimization loop. The BayesianExperimentDesign's hypothesis pool is static.
- **Impact**: For complex posterior landscapes (multi-modal hypothesis spaces), the current brute-force approach will be exponentially inefficient.

### DEFECT-427-MC-3: No Rare Event Simulation for System Health
- **Evidence**: Multi-Index IS for MV-SDEs (2026) achieves O(TOL⁻²(log TOL⁻¹)²) complexity for rare events. Critical for safety-relevant rare failures.
- **Gap**: NT-SHIELD threat detection and NT-REPAIR self-healing must detect rare failure modes. No rare-event simulation framework exists.
- **Impact**: Self-healing capabilities (NT-REPAIR) cannot be validated against rare but catastrophic failure scenarios. The "Dark Forest" axiom requires modules to connect — self-healing logic is untestable for edge cases.

---

## Suggestions

| ID | Category | Suggestion | Priority |
|----|----------|------------|----------|
| S-427-1 | ABM | Implement `nt_core_abm` module with AgentSet, timed Action system, and DiscreteEventSimulator inspired by Mesa 4.0a0 | P2 |
| S-427-2 | ABM | Add `Action` trait with `on_start/on_resume/on_complete/on_interrupt` hooks for NT-PHYSICAL motor control | P3 |
| S-427-3 | ABM | Create LLM-agent cost model: track `llm_decision_cost_ms` in SEAL pipeline budgets | P2 |
| S-427-4 | SD | Implement `nt_core_sd` module: Stock, Flow, Auxiliary, MaterialLink, InformationLink types | P2 |
| S-427-5 | SD | Add CLD/CLD+ parser: parse causal loop diagrams → stock-flow diagrams → executable model | P3 |
| S-427-6 | SD | Integrate D2D-style leverage point analysis for ConsciousnessTree branch optimization | P3 |
| S-427-7 | SD | Add LLM-to-SD model generation pipeline with VMS validation | P3 |
| S-427-8 | MC | Implement `nt_core_mc_is` module: importance sampling with proposal distributions | P2 |
| S-427-9 | MC | Add EM2C-style adaptive IS with entropic mirror descent for BayesianExperimentDesign | P3 |
| S-427-10 | MC | Create rare-event simulation framework for NT-SHIELD/NT-REPAIR validation | P2 |
| S-427-11 | MC | Implement ML-guided proposal distribution optimization (Gaussian Process / active learning) | P3 |
| S-427-12 | Integration | Wire SD stock-flow models into ConsciousnessTree health signal aggregation | P2 |
| S-427-13 | Integration | Add agent-based population simulation for SEAL pipeline evolution testing | P3 |

---

## Summary

**Total defects found**: 9 (3 ABM, 3 SD, 3 MC)
**Sources consulted**: 10 (8 web, 2 grep scans)
**Critical gaps**: NeoTrix has zero ABM infrastructure, zero SD modeling, and only rudimentary MC (MCTS + GRPO ratio). The 2026 research landscape shows mature frameworks (Mesa 4, EM2C, D2D, CLD+) that could be absorbed. The biggest architectural risk is DEFECT-427-MC-1: the BayesianExperimentDesign's efficiency degrades exponentially for rare hypotheses without variance reduction.
