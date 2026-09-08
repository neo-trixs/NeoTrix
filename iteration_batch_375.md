# Iteration Batch 375 — NeoTrix Consciousness Architecture

**Date**: 2026-09-06
**Research Domains**: Autonomous Navigation | Decision Making Under Uncertainty | Planning Algorithms

---

## 1. Sources Cited

### 1.1 Autonomous Navigation

| # | Source | Key Finding |
|---|--------|-------------|
| S1 | RealSense + LimX Dynamics, GTC 2026 ([therobotreport.com](https://www.therobotreport.com/realsense-unveils-autonomous-humanoid-navigation-gtc-2026/)) | Dense 3D depth perception fused with NVIDIA CuVSLAM + vSLAM enables humanoid robots to localize, map, and navigate autonomously in 3D space. Simulation-first (Isaac Lab) bridges sim-to-real gap. |
| S2 | SoftServe, "AI-Powered Navigation for AMRs" (2025) | Multi-sensor fusion (LiDAR+camera+IMU+GPS-RTK), RL-based path planning, edge AI, and decentralized fleet coordination are the four pillars of next-gen navigation. |
| S3 | Smashing Robotics, "Sim-to-Real Transfer Guide" (Sep 2026) | Domain randomization, system identification, and progressive neural networks are production-standard for sim-to-real. First deployment is data-gathering, not final test. |
| S4 | Springer, "Sim2Real Transfer of DRL for Robotic Pick-and-Place" (Jul 2026) | PPO-trained policies in Isaac Lab/Isaac Sim with domain randomization achieve reliable transfer. Actuator-level modeling is critical for torque-controlled robots. |
| S5 | Nature Scientific Reports, "End-to-end Sim-to-Real RL via Neural Stylisation" (Mar 2026) | Neural style transfer reinterprets unpaired real-world data to synthesize training trajectories, bridging domain gap without paired data. |
| S6 | arXiv:2608.22629, "Real2Sim2Real Pipeline" (Aug 2026) | Real2Sim calibration (friction/inertia/gravity via genetic algorithms) + TQC-based RL → significant improvement in tracking accuracy and policy robustness for torque-controlled arms. |

### 1.2 Decision Making Under Uncertainty

| # | Source | Key Finding |
|---|--------|-------------|
| S7 | Harikumar et al., UAI 2026 ([arXiv:2607.08590](https://arxiv.org/abs/2607.08590)) | AR-DEIG: Adversarially Robust Decision Expected Information Gain. Shifts focus from parameter inference to adversarially robust decision utility. Conventional design converges to high-confidence yet fragile decisions. |
| S8 | Dixon, "Adaptive AI Delegation under Uncertainty" (Jun 2026, [arXiv:2606.29406](https://arxiv.org/abs/2606.29406)) | Bayesian governance policy for dynamic allocation of decision authority to AI. Sequential Bayesian governance outperforms specialized heuristics across heterogeneous AI-quality regimes. |
| S9 | Kleisarchaki, "HMM-POMDP Framework for F1 Energy Strategy" (Mar 2026, [arXiv:2603.01290](https://arxiv.org/abs/2603.01290)) | 40-state HMM achieving 96.8% ERS-level accuracy. Decomposition of ERS=Low into harvest vs. derate states — latent state decomposition as key for strategic inference. |
| S10 | Arcieri et al., "Bridging POMDPs and Bayesian Decision Making" (ETH Zürich, 2023, Reliability Eng.) | MCMC sampling of HMM parameters → POMDP solutions merged with Bayesian decision theory → robust to epistemic uncertainty. First real-world application with continuous data. |
| S11 | Arce et al., "A Unifying Bayesian Framework for Adversarial Robustness" (Jun 2026) | Bayesian framework models adversarial uncertainty through a stochastic channel, articulating all probabilistic assumptions. |
| S12 | ICLR 2026, "Provable Adversarial Robustness in In-Context Learning" (Feb 2026, [arXiv:2602.17743](https://arxiv.org/abs/2602.17743)) | Robustness-capacity trade-off: maximum adversarial shift scales with √(attention head dimension). Sample complexity grows as ρ². |

### 1.3 Planning Algorithms

| # | Source | Key Finding |
|---|--------|-------------|
| S13 | arXiv:2604.03208, "Hierarchical Planning with Latent World Models (HWM)" (Apr 2026) | Multi-temporal-scale world models in shared latent space. Long-horizon predictions serve as subgoals for short-horizon via latent matching. 70% success vs 0% for single-level planning. 3x less compute. |
| S14 | Zylos Research, "AI Agent Goal Decomposition and Hierarchical Planning" (Mar 2026) | HTN+LLM integration: compound tasks (LLM-interpreted) + primitive tasks (operators). Plan-then-Execute vs Interleaved/ReAct tradeoff. Capability bundles as pre-authorized plans. |
| S15 | Hu et al., "LLM-Grounded Dynamic Task Planning with Hierarchical Temporal Logic" (Feb 2026, [arXiv:2602.09472](https://arxiv.org/abs/2602.09472)) | Neuro-symbolic: LLM → hierarchical LTLf specifications → STAP solver. Receding horizon planning handles stochastic environmental changes. 93% success in complex scenarios vs 19% for monolithic LLM approaches. |
| S16 | Kawabe et al., "Hierarchical LLM-Based Multi-Agent Framework" (Feb 2026, [arXiv:2602.21670](https://arxiv.org/abs/2602.21670)) | Multi-layer LLM agents with prompt optimization. Forward step (decompose) + Feedback step (verify and optimize prompts). |
| S17 | Choi et al., "ReAcTree: Hierarchical LLM Agent Trees with Control Flow" (AAMAS 2026) | Dynamic tree of agent nodes + control flow nodes (behavior-tree-inspired). Isolates subgoals to reduce hallucination. Targeted in-context example selection per node. |
| S18 | arXiv:2604.23194, "From Coarse to Fine: Self-Adaptive Hierarchical Planning" (Apr 2026) | Self-adaptive plans tailored to task difficulty. Optimized via imitation learning and capability enhancement. |
| S19 | Temporal Replay 2026 (May 2026) | Durable execution for AI agents: Serverless Workers, Workflow Streams (durable streaming), External Payload Storage, Rust SDK (Public Preview). |
| S20 | arXiv:2602.14344, "Zero-Shot Instruction Following via Structured LTL" (2026) | LTL conditions policy on sequences of Boolean formulae from Büchi automaton. Hierarchical neural architecture encodes logical structure with temporal attention. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-1: No Formal Temporal Constraint Language (Critical)

**Source**: S13, S15, S16, S17, S20
**Severity**: HIGH — architectural gap

The 2026 research consensus is that LLM-only planning fails at scale (19% success for monolithic LLM in complex scenarios per S15). The field has converged on **hierarchical LTL/LTLf** (Linear Temporal Logic on finite traces) as the formal backbone, with LLMs as the natural-language-to-specification translator. NeoTrix has no formal temporal constraint language.

**NeoTrix gap**: The SEAL pipeline, ConsciousnessTree growth cycles, and E8 hexagram reasoning all operate without formally specified temporal invariants. A growth cycle that violates a temporal property (e.g., "absorption must complete before next cycle starts") is only caught at runtime if at all.

**Suggestion**: Introduce a lightweight LTLf substrate in `nt_core` for expressing temporal invariants on pipeline stages. LLM generates H-LTLf specs from natural language intent; formal solver verifies before execution. This bridges the neuro-symbolic gap identified by S15.

---

### DEFECT-2: No Adversarial Robustness in Decision Selection (Critical)

**Source**: S7, S8, S11, S12
**Severity**: HIGH — security + reliability

Harikumar et al. (S7, UAI 2026) demonstrate that conventional decision-aware design converges to **high-confidence yet fragile decisions** under adversarial perturbation. NeoTrix's E8 hexagram reasoning and Bayesian experiment design (`VoI` in `nt_core_hcube::bayesian_experiment`) optimize for nominal utility but have no adversarial robustness mechanism.

**NeoTrix gap**: The VoI (Value-of-Information) metric selects experiments based on expected information gain under the learned model. If the model is perturbed (adversarial or distributional shift), the selected experiment may be informatively optimal but decision-fragile. The `M-open check` only detects hypothesis concentration, not adversarial robustness.

**Suggestion**: Implement AR-DEIG-style decision selection: when selecting the next SEAL experiment or evolution action, evaluate not just `VoI(θ)` but also `min_ξ V(θ, d*(ξ))` over an adversarial budget ε. Add an `adversarial_robustness` dimension to the SelfTest registry (T3 production wiring) that checks whether evolution decisions remain stable under perturbed model parameters.

---

### DEFECT-3: No Sim-to-Real Calibration Loop for Knowledge Transfer (Moderate)

**Source**: S3, S4, S5, S6
**Severity**: MEDIUM — fidelity gap

The sim-to-real research (S3-S6) has converged on **Real2Sim2Real** as the production pipeline: calibrate simulator parameters from real-world observations, train in calibrated sim, deploy. NeoTrix absorbs external knowledge (codebases, papers, APIs) but has no calibration step between "learned in simulation (KB)" and "deployed in production (real code)".

**NeoTrix gap**: R-P79 mandates external absorption must connect to production in the same session, but there is no systematic way to verify that knowledge learned in the KB "simulation" transfers accurately to the production codebase. The `converge_check()` audits ghost modules but not knowledge fidelity.

**Suggestion**: Add a `knowledge_fidelity_check()` phase to the SEAL pipeline between Distillation and Absorption. For each absorbed pattern: (1) generate a test case from the pattern, (2) verify it against the actual production code, (3) measure the fidelity gap. Low-fidelity patterns get flagged for manual review. This is the knowledge-transfer analog of domain randomization.

---

### DEFECT-4: No Hierarchical Latent World Model (Moderate)

**Source**: S13, S14, S18
**Severity**: MEDIUM — planning efficiency

HWM (S13) demonstrates that multi-temporal-scale world models in a shared latent space achieve 70% task success vs 0% for single-level planning, with 3x less compute. NeoTrix's SEAL pipeline operates at a single temporal granularity — each cycle has the same structure regardless of whether it's a micro-adjustment or a macro-evolution.

**NeoTrix gap**: The `RhythmRecalculator` adjusts segment durations via power law, but the SEAL pipeline itself has no hierarchical planning abstraction. Long-horizon evolution goals (e.g., "absorb 10 external repos this quarter") and short-horizon cycle goals (e.g., "distill this one paper") share the same flat pipeline.

**Suggestion**: Introduce hierarchical planning in NT-MIND: a long-horizon world model that predicts multi-cycle outcomes, with short-horizon models executing within each cycle. The long-horizon model generates subgoals via latent matching (as HWM does), and the short-horizon model executes. This reduces planning compute and improves success rate on long-horizon evolution tasks.

---

### DEFECT-5: No Bayesian Governance for AI Decision Authority (Moderate)

**Source**: S8, S10
**Severity**: MEDIUM — governance

Dixon (S8) shows that sequential Bayesian governance outperforms specialized heuristics for dynamically allocating decision authority to AI. NeoTrix's NT-GOVERNANCE domain (`Gov-衡`) enforces constitution compliance but has no quantitative framework for deciding *when* to trust AI-generated decisions vs requiring human review.

**NeoTrix gap**: The `Disclosure Ladder` (AnchorPromote) controls tool budget, not decision authority. There is no mechanism to dynamically shift between AI-autonomous and human-in-the-loop based on evidence quality, uncertainty level, or organizational risk tolerance.

**Suggestion**: Implement a Bayesian governance policy in NT-GOVERNANCE that maintains a posterior over AI decision quality, updated after each SEAL cycle. When posterior concentration on "AI-competent" is above threshold → full autonomy. Below threshold → escalate. This is the decision-authority analog of the Disclosure Ladder.

---

### DEFECT-6: No Latent State Decomposition for Strategic Inference (Minor)

**Source**: S9, S14
**Severity**: LOW — reasoning depth

Kleisarchaki (S9) demonstrates that decomposing a single latent state into strategically distinct sub-states (ERS=Low → harvest vs. derate) achieves 96.3% detection accuracy. NeoTrix's `SelfModel` types (static/dynamic/value) are coarse — they don't decompose ambiguous states into strategically distinct sub-states.

**NeoTrix gap**: When the HeartbeatAggregator reports "module health = degraded", there's no decomposition into "degraded due to genuine failure" vs "degraded due to intentional exploration (SEAL phase)". This mirrors the harvest/derate ambiguity.

**Suggestion**: Extend `SystemHealthSnapshot` with state decomposition labels: for each health signal, attach a latent state that distinguishes genuine degradation from intentional variance. This improves both GWT attention routing and governance decisions.

---

### DEFECT-7: No Plan Verification Before Execution (Minor)

**Source**: S15, S17
**Severity**: LOW — safety

ReAcTree (S17) and H-LTLf (S15) both verify plans before execution using formal methods. NeoTrix executes SEAL pipeline stages without pre-execution verification of the plan.

**NeoTrix gap**: The pipeline proceeds stage-to-stage without checking whether the next stage's preconditions are satisfiable given current state. Failures are caught at the SelfTest stage (post-execution), not before.

**Suggestion**: Add a lightweight `plan_verify()` step before each SEAL stage transition that checks precondition satisfiability against the current KB state. This catches impossible plans early, reducing wasted compute.

---

## 3. Summary of Suggestions

| # | Defect | Severity | Suggested Fix | Target Module |
|---|--------|----------|---------------|---------------|
| 1 | No formal temporal constraint language | HIGH | LTLf substrate + H-LTLf for pipeline invariants | `nt_core` |
| 2 | No adversarial robustness in decisions | HIGH | AR-DEIG-style min-max decision selection | `nt_core_hcube`, `nt_meta` |
| 3 | No sim-to-real calibration for knowledge | MEDIUM | `knowledge_fidelity_check()` in SEAL | `nt_mind` (SEAL) |
| 4 | No hierarchical latent world model | MEDIUM | Multi-temporal-scale planning | `nt_mind` |
| 5 | No Bayesian governance for decision authority | MEDIUM | Posterior-over-quality governance policy | `nt_governance` |
| 6 | No latent state decomposition | LOW | Health signal decomposition labels | `nt_core_heartbeat` |
| 7 | No plan verification before execution | LOW | Pre-execution precondition check | `nt_mind` (SEAL) |

---

## 4. Cross-Cutting Themes

1. **Neuro-symbolic convergence**: The 2026 research consensus is LLM + formal methods (not LLM alone). NeoTrix should adopt a neuro-symbolic planning substrate.
2. **Adversarial-aware Bayesian decision-making**: Nominal optimality is insufficient. AR-DEIG (S7) and Bayesian governance (S8) both shift from "optimal under model" to "robust under perturbation."
3. **Hierarchical temporal abstraction**: Single-granularity planning fails at scale. HWM (S13), HTN+LLM (S14), and hierarchical LTL (S15) all use multi-scale planning.
4. **Real2Sim2Real for knowledge**: Knowledge fidelity requires calibration loops, not one-shot absorption.

---

*Generated by iteration 375 of the NeoTrix research loop. Sources accessed 2026-09-06.*
