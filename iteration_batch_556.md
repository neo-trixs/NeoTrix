# Iteration Batch 556 — Research Loop

**Date**: 2026-09-06
**Previous**: Batch 555 (value allocation, self-model, B_max(t), SPD/sheaf, convex-shallow)
**Focus**: Cognitive Load Theory, Mental Models, Problem Solving & Subgoal Identification

---

## 1. Cognitive Load Theory Findings

### 1A. Germane Load Is Not Independent — It's an Allocated Residual
**Source**: Sweller, 2010; Kalyuga, 2011; Frontiers in Psychology 2026 (doi:10.3389/fpsyg.2026.1804728)

**Key Insight**: Germane load is NOT a third independent source alongside intrinsic and extraneous. It was reframed as the *share of working-memory resources allocated to schema construction* once extraneous load is minimized. Some researchers argue CLT needs only intrinsic and extraneous — germane is a derived quantity.

**Defect vs Batch 555**: Batch 555 modeled B_max(t) as a single bandwidth cap. But cognitive load theory reveals that bandwidth is *partitioned* into three competing streams (intrinsic + extraneous + germane). NeoTrix's `B_max(t)` must decompose into `B_intrinsic(t) + B_extraneous(t) + B_germane(t)` where each is dynamically allocated. The **value allocation module** from batch 555 is incomplete — it needs a *load partitioning sub-module* that tracks what fraction of attention budget goes to schema construction vs task execution vs noise suppression.

### 1B. Affective Arousal Selectively Predicts Germane Processing
**Source**: Frontiers in Psychology 2026 (doi:10.3389/fpsyg.2026.1804728)

**Key Insight**: Affective arousal (emotional state) is *selectively* and *positively* correlated with germane cognitive load (r=0.32***), while negatively correlated with extraneous load (r=-0.24**). Arousal does NOT predict intrinsic or extraneous load — it predicts schema-building effort specifically.

**Defect vs Batch 555**: Batch 555 did NOT connect the NT-FEEL emotion engine to cognitive load allocation. This is a critical wiring gap: **EmotionLabel must modulate B_germane(t) allocation**. When arousal is high, more bandwidth flows to schema construction; when arousal is low, bandwidth leaks to extraneous processing. The PerceptionBridge's `awareness_score()` should incorporate arousal-gated germane allocation.

### 1C. Expertise Reversal Effect — No Universal Good Design
**Source**: Yu-kai Chou 2026; Sweller foundational work

**Key Insight**: The worked example that helps a novice actively *burdens* an expert. There is no design-in-the-abstract — only design-for-one-learner's-current-schema. CLT's most durable finding undercuts its own universality.

**Defect vs Batch 555**: NeoTrix's AttentionManager routes based on task type, but does NOT account for the learner's current schema state. The Dual Specialization (Weapon Set I/II) must be *expertise-modulated*: the same module configuration that helps a novices-module-state actually hinders an expert's-module-state. Need a **schema-awareness signal** from NT-MEMORY's KB to gate attention routing.

---

## 2. Mental Models Findings

### 2A. Mental Models Are Iconic Representations, Not Logical Forms
**Source**: Johnson-Laird & Byrne 1991; Wikipedia 2026; Nature 2026 (CW-Net)

**Key Insight**: Mental models are *analogous* to the structure they represent — each part of a model corresponds to each part of reality (iconic), unlike logical forms which are symbolic. Reasoning depends on mental models, not logical form. People infer validity by checking if conclusion holds in all mental models of the premises.

**Defect vs Batch 555**: Batch 555 treated self-model as "programmed not emergent" but didn't specify the representation format. Mental model theory says the representation must be **iconic** (structure-preserving) — not symbolic. NeoTrix's VSA HyperCube is a symbolic vector representation. There is a **representation mismatch**: VSA encoding is compositional/symbolic, but effective reasoning requires iconic structure. Need a **HybridVSA module** that maintains both symbolic vectors AND iconic structural isomorphisms for reasoning tasks.

### 2B. Mental Simulations Are Resource-Bounded with a "Switch Point"
**Source**: Wang & Ullman 2025 (JEP:General, 154(8), 2105-2124), published 2026-06-09

**Key Insight**: People run approximate mental simulations that hit a **resource limit after a time threshold** ("switch point"). Short simulations (1-7s) are accurate; longer ones degrade. Individual differences in digit-span predict switch-point timing. The simulation is *partial* and degrades gracefully, not catastrophically.

**Defect vs Batch 555**: Batch 555's B_max(t) assumed a fixed bandwidth ceiling. Wang & Ullman show the simulation model has a **switch point** — a temporal threshold where behavior shifts from simulation-based to heuristic-based. NeoTrix needs a `simulation_switch_point(t)` mechanism: when reasoning time exceeds the switch point, the system should explicitly degrade from high-fidelity simulation to approximate heuristic, rather than continuing to burn tokens on an increasingly unreliable simulation. This is NOT just bandwidth limiting — it's a **mode transition**.

### 2C. CW-Net: Grounded Explanations Improve Mental Models
**Source**: Nature 2026 (doi:10.1038/s41586-026-10950-5)

**Key Insight**: Concept-Wrapper Network (CW-Net) grounds black-box ML planner reasoning in human-interpretable concepts, causally improving the human driver's mental model. Mental model improvement → better predictive ability + situational awareness. Explanations improve in surprising situations without degrading unsurprising ones.

**Defect vs Batch 555**: Batch 555 proposed Egress Privacy Guard for external LLM interactions. But CW-Net reveals an *internal* gap: NeoTrix's own decision-making (NT-CORE's E8 reasoning) produces no grounded concept-level explanations. When the system makes a decision, it should emit **concept-grounded explanations** using domain vocabulary (from CONTEXT.md shared language) — not opaque vector operations. The HeartbeatAggregator should track *explanation quality* as a health signal.

---

## 3. Problem Solving & Subgoal Identification Findings

### 3A. MiRA: Milestone-Based Dense Rewards for Long-Horizon Agents
**Source**: arXiv 2603.19685 (March 2026); Zhongzhu Zhou technical review 2026-03-23

**Key Insight**: Traditional RL for long tasks gives reward only at completion (sparse reward). MiRA generates subgoals from a teacher model, then uses **Potential-Based Reward Shaping (PBRS)** with a learned *potential critic* P_ψ to provide dense intermediate signals. The potential critic learns from successful trajectory traces — specifically, the longest-common-subsequence of actions that recur across successful runs.

**Defect vs Batch 555**: Batch 555's SEAL pipeline has 6 stages but no *dense milestone reward* between stages. MiRA shows that sparse stage-completion signals are insufficient — need **inter-stage potential functions**. Each SEAL stage transition should emit a potential score P_ψ(s_t) indicating progress along successful semantic paths. This connects to the value allocation module: P_ψ provides the *grounding signal* for value function updates.

### 3B. The "35-Minute Degradation Problem" Is Real and Universal
**Source**: zylos.ai 2026-01-16; multiple practitioner reports

**Key Insight**: Agents that perform reliably up to ~35 minutes degrade sharply beyond that. Causes compound: context window saturation, error compounding across many steps, absence of robust checkpointing. This is an *engineering* problem, not a theoretical one — context management + checkpointing + state isolation are the three pillars.

**Defect vs Batch 555**: Batch 555 identified bandwidth B_max(t) but didn't model **temporal degradation**. The 35-minute threshold suggests B_max(t) is not static but *decays over elapsed execution time*. Need a `B_effective(t) = B_max(t) * decay(t)` where `decay(t)` models accumulated context saturation. NT-NEXUS's cross-session memory must provide **checkpointing** to reset the decay curve — each checkpoint is a "fresh start" that restores B_effective.

### 3C. Subgoal Selection Lacks Automatic, Theoretically Justified Criteria
**Source**: emergentmind.com/goal-decomposition; ScienceDirect 2026 (doi:10.1016/j.knosys.2026.116178)

**Key Insight**: Current subgoal identification methods (LLM-based, heuristic-based, multiple-instance-learning-based) all lack *automatic, theoretically justified criteria* for how many subgoals to create and where to split. Thresholds are heuristic. The ScienceDirect paper uses multiple instance learning to improve subgoal identification accuracy but acknowledges the fundamental open problem.

**Defect vs Batch 555**: Batch 555's SEAL pipeline stages are architecturally fixed. But MiRA and the subgoal literature show that **stage boundaries should be dynamically determined** by problem structure, not hard-coded. NT-MIND's SEAL stages need a **dynamic stage-boundary mechanism** that uses VoI (Value-of-Information, already in `nt_core_hcube::bayesian_experiment`) to determine when to transition stages. VoI provides the theoretical justification: transition when the expected information gain from continuing the current stage drops below the expected gain from starting the next.

---

## 4. Summary: NEW Defects vs Batch 555

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| D1 | B_max(t) must decompose into intrinsic+extraneous+germane streams | NT-CORE + NT-FEEL | HIGH |
| D2 | EmotionLabel must modulate germane allocation (arousal→schema building) | NT-FEEL → NT-CORE | HIGH |
| D3 | Dual Specialization must be expertise-modulated (schema-awareness gating) | NT-CORE + NT-MEMORY | MEDIUM |
| D4 | VSA HyperCube is symbolic; reasoning needs iconic isomorphism — HybridVSA needed | NT-CORE + NT-MEMORY | HIGH |
| D5 | B_max(t) needs switch-point mode transition, not just bandwidth cap | NT-CORE | HIGH |
| D6 | E8 reasoning emits no concept-grounded explanations — explanation quality health signal | NT-CORE + NT-IO | MEDIUM |
| D7 | SEAL pipeline lacks dense milestone rewards — need inter-stage potential functions P_ψ | NT-MIND + NT-ACT | HIGH |
| D8 | B_effective(t) decays with elapsed execution — checkpointing resets decay curve | NT-NEXUS + NT-CORE | HIGH |
| D9 | SEAL stage boundaries should be dynamically determined via VoI, not hard-coded | NT-MIND + NT-CORE | MEDIUM |

---

## 5. Sources Cited

1. Frontiers in Psychology 2026 — "When arousal meets cognitive load: affective arousal and germane processing in a Stroop-like task" (doi:10.3389/fpsyg.2026.1804728)
2. Yu-kai Chou 2026 — "Cognitive Load Theory: S-Tier Behavioral Designer's Guide" (yukaichou.com)
3. Wang & Ullman 2025/2026 — "Resource bounds on mental simulations: Evidence from a liquid-reasoning task" (JEP:General 154(8), 2105-2124)
4. Nature 2026 — "Explainable deep learning improves human mental models of self-driving cars" (doi:10.1038/s41586-026-10950-5)
5. arXiv 2603.19685 — MiRA: "A Subgoal-driven Framework for Improving Long-Horizon LLM Agents" (March 2026)
6. Zhongzhu Zhou 2026 — MiRA Technical Review (zhongzhuzhou.org)
7. ScienceDirect 2026 — "Subgoal identification with multiple instance learning methods" (doi:10.1016/j.knosys.2026.116178)
8. zylos.ai 2026-01-16 — "Long-Running AI Agents and Task Decomposition 2026"
9. emergentmind.com — "Goal Decomposition: Principles & Applications"
10. Johnson-Laird & Byrne 1991 — Mental Model Theory of Reasoning (via Wikipedia 2026)
