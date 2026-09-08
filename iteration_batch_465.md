# Iteration Batch 465 — NeoTrix Consciousness Architecture Research Loop

**Date**: 2026-09-06
**Domains**: RLHF, DPO/Preference Optimization, Constitutional AI/RLAIF
**Sources Consulted**: 22

---

## Sources Cited

| # | Source | Date | Domain |
|---|--------|------|--------|
| S1 | NVIDIA — "RLBFF: Reinforcement Learning with Binary Flexible Feedback" (ICLR 2026) | 2025-09 (ICLR 2026) | RLHF |
| S2 | arXiv 2608.06310 — "RRC: Ranking-Based Reward Construction for Generative RMs" | 2026-08-06 | RLHF |
| S3 | arXiv 2608.24949 — "Demystifying RL Post-Training of Language Models" | 2026-08-24 | RLHF |
| S4 | ACL 2026 — "ARF-RLHF: Adaptive Reward-Following via TraceBias" | 2026 (ACL) | RLHF |
| S5 | arXiv 2607.26094 — "MeRLa: Meta-Learned Reward Shaping for RLHF" | 2026-07-28 | RLHF |
| S6 | arXiv 2602.11523 — "DAR: Dual-regularized Advantage Regression" (ICLR 2026) | 2026-02 | RLHF/DPO |
| S7 | ACL 2026 — "Joint Optimization of Training Data and Policy in RLHF" | 2026 (ACL) | RLHF |
| S8 | arXiv 2601.07349 — "RM-NLHF: Reward Modeling from Natural Language Human Feedback" | 2026-01 | RLHF |
| S9 | AxiomLogica — "ORPO vs DPO vs KTO vs SimPO: which method in 2026?" | 2026-05-15 | DPO |
| S10 | AI Security Directory — "DPO Complete Guide for 2026" | 2026-04-13 | DPO |
| S11 | Jacar.es — "DPO alternatives to RLHF: Practical State in 2026" | 2026-04-28 | DPO |
| S12 | arXiv 2410.15595 — "A Survey of DPO: Datasets, Variants, Applications" | 2026-06-09 | DPO |
| S13 | AcingAI — "Preference Optimization After DPO: IPO, KTO, SimPO Compared" | 2026-07-28 | DPO |
| S14 | Nick Gustafson — "DPO and Its Variants" | 2026 | DPO |
| S15 | arXiv 2603.03000 — "Latent Value Hypothesis: Why RLAIF Works" | 2026-03 | Constitutional AI |
| S16 | ACL 2026 — "Curriculum-RLAIF: Curriculum Alignment with AI Feedback" | 2026 (ACL) | Constitutional AI |
| S17 | arXiv 2601.18730 — "Reflect: Transparent Principle-Guided Reasoning" | 2026-01-26 | Constitutional AI |
| S18 | Anthropic — "Constitutional AI: Harmlessness from AI Feedback" | 2022-12 | Constitutional AI |
| S19 | Multigrid — "Constitutional AI and RLAIF" | 2026-08-03 | Constitutional AI |
| S20 | arXiv 2604.17769 — "Reverse Constitutional AI (R-CAI)" | 2026-04 | Constitutional AI |
| S21 | Absolute Digital Publishers — "How Constitutional AI Actually Constrains Behavior" | 2026-08-11 | Constitutional AI |
| S22 | Brecht Corbeel — "Constitutional AI technical reconstruction" | 2026-08-11 | Constitutional AI |

---

## Research Findings

### 1. RLHF: From Scalar RMs to Binary Flexible Feedback and Generative RMs

**F1.1 — RLBFF Bridges RLHF and RLVR**: NVIDIA's RLBFF (Reinforcement Learning with Binary Flexible Feedback) combines the versatility of human preferences with rule-based verification. Reward Models trained via RLBFF achieve #1 on JudgeBench (81.4%) and 86.2% on RM-Bench. Key insight: extract binary-answerable principles from natural language feedback, then train RM as an entailment task. This outperforms Bradley-Terry models at matched data scale. Open-source recipe aligns Qwen3-32B to match o3-mini/DeepSeek R1 at <5% inference cost. (S1)

**F1.2 — Generative RMs Underperform on Principle Adherence**: Despite strong ranking ability, Generative RMs (GenRMs) underperform Scalar RMs on PrincipleBench. Hypothesis: GenRMs initialized from reasoning models over-index on correctness and under-weight other quality dimensions (repetition, clarity). GenRMs are ~100x slower than Scalar RMs. The 2026 landscape has two RM paradigms: fast scalar for training, slow generative for evaluation. (S1)

**F1.3 — RRC: Ranking-Based Reward Construction**: RRC transforms generative RM outputs from scalar scores to relative rankings, resolving the mismatch between comparative RM nature and scalar RL algorithms. Two mechanisms: self-competitive ranking (compare among sampled responses) and anchor-guided ranking (reference points reduce RM calls from O(m·log m) to O(m·n)). Achieves 35.8%→41.3% on AlpacaEval2, 8.0%→11.2% on ArenaHardV2. (S2)

**F1.4 — RL Post-Training Deconstructed**: The 2026 primer reveals that RL success depends on whether the base model already places sufficient probability mass on desired behavior (classical exploration). Spurious rewards' impact depends on prompt distribution. Entropy of policy output distribution reveals how pretraining, SFT, and RL each shape model certainty differently. (S3)

**F1.5 — ARF-RLHF: Continuous Preference Trajectories**: Adaptive Reward-Following converts natural feedback into continuous satisfaction trajectories via TraceBias algorithm. Double Average Method (DAM) normalizes satisfaction scores and token-level policy ratios, stabilizing training without explicit gradient clipping. Outperforms PPO by +3.3% and DPO by +7.6% across tasks. Core finding: rescoring is essential, not auxiliary — removing it causes persistent misalignment under preference reversal. (S4)

**F1.6 — MeRLa: Meta-Learned Reward Shaping**: Meta-learns a task-aware shaping function across auxiliary tasks before RLHF. Achieves 90.8% win rate on AlpacaEval 2.0, 41% less training instability. Key: potential-based shaping preserves policy optimality. Complementary to process-based and rubric-based rewards. (S5)

**F1.7 — DAR: Dual-KL Regularization Unifies RLHF**: Dual-regularized Advantage Regression replaces both PPO and DPO with a weighted SFT loss that explicitly balances reference regularization (prevent reward hacking) and policy ratio clipping (stable optimization). Achieves 92.42% win rate vs GRPO's 85.15%. The effective reference target becomes an interpolation of π₀ and πₜ that moves toward optimal distribution. (S6)

**F1.8 — Process Reward > Outcome Reward for GRMs**: MetaRM learns to predict process reward from limited human critiques and generalizes to unlabeled data. Online MetaRM framework adapts to distribution shifts during RL training. Critique-quality supervision (F1-based similarity between human and model critiques) consistently outperforms outcome-only supervision. (S8)

### 2. DPO Family: The 2026 Consensus and Frontier

**F2.1 — DPO Is the Default, Not the Ceiling**: DPO remains the most stable, well-documented, and lowest-risk alignment method in 2026. TRL v1.0 stable surface includes DPOTrainer. The consensus: DPO as default, IPO for high-noise datasets, KTO when pairs are unavailable, SimPO when compute is critical. (S9, S10, S11)

**F2.2 — Frontier Recipe Is Hybrid**: OpenAI, Anthropic, Google DeepMind all keep RLHF-style online components: SFT → DPO/IPO (broad alignment) → PPO-style online RL (reasoning, code, agentic tasks). The industry didn't "switch to DPO"; it absorbed DPO into a larger toolkit. (S14)

**F2.3 — DPO Overfitting Pathology**: When preference labels are near-deterministic (chosen almost always preferred), DPO's unbounded loss drives log-ratios arbitrarily high, collapsing reference regularization. IPO's squared loss fixes this specifically. For production: DPO β in [0.1, 0.5], LR 5e-7 to 1e-6, batch size 32-128. (S10, S13)

**F2.4 — KTO Changes Data Requirements, Not Just Loss**: KTO accepts binary (good/bad) labels per completion — no pairing required. This matches real-world feedback signals (thumbs up/down). The Kahneman-Tversky utility objective treats gains and losses asymmetrically, reflecting human loss aversion. Particularly valuable when positive signal is sparser than negative. (S9, S12, S13)

**F2.5 — SimPO Reference-Free Tradeoff**: SimPO halves per-step cost by eliminating the reference model (no frozen forward pass). Uses length-normalized average log-probability as reward with target margin. But: removes the stabilizer that anchors policy to known-good distribution. More prone to catastrophic forgetting on noisy/OOD data. Highly hyperparameter-sensitive (β starting at 2.0, γ/β_ratio at 0.5). (S9, S13)

**F2.6 — Online/Iterative DPO Addresses Distribution Shift**: Standard DPO is offline and bounded by dataset coverage. 2026 solutions: online DPO constructs preference data from current policy samples, iterative DPO alternates between data generation and optimization. Key bottleneck: efficiently constructing online preference data to mitigate distributional discrepancy. (S12)

### 3. Constitutional AI / RLAIF: Theoretical Foundations and Practical Limits

**F3.1 — Latent Value Hypothesis Explains Why RLAIF Works**: Pretraining encodes human values as directions in representation space. Constitutional prompts act as retrieval keys eliciting these latent values into explicit preference judgments. RLAIF improves alignment when constitution-activated direction correlates better with true values than the model's default generation direction. Ceiling scales with model capacity. (S15)

**F3.2 — Adversarial Constitutions Exist**: Because pretraining encodes both pro-social and anti-social norms, adversarial constitutions exist that can activate harmful value directions. This explains the refusal direction in base models (Arditi et al. 2024) and the low-rank structure of safety fine-tuning. Implication: constitution design is a security surface. (S15)

**F3.3 — Curriculum-RLAIF Improves Generalizability**: Reward models trained with conventional RLAIF suffer from limited generalizability due to distribution shift, label noise, and sample difficulty mismatch. Curriculum-RLAIF constructs preference pairs with controlled difficulty levels, training RM from easy to hard. Consistently outperforms non-curriculum baselines across harmlessness, helpfulness, and summarization. (S16)

**F3.4 — Reflect: Inference-Time Constitutional Alignment**: No parameter fine-tuning. Post-generation self-evaluation, critique, and revision against multi-principle constitution. Produces aligned responses AND training data as byproduct. Effective even at small model sizes. Unifies inference-time alignment, multi-principle reasoning, and reusable supervision. (S17)

**F3.5 — R-CAI: Reverse Constitutional AI for Red Teaming**: Inverts constitutional principles into toxicity objectives. Probability clamping in RLAIF prevents reward hacking while preserving adversarial intent. Demonstrates that alignment merely suppresses rather than eliminates harmful behaviors. (S20)

**F3.6 — Text Does Not Compel Behavior**: The constitution shapes a training signal; the model learns whatever fits that signal, which may be narrower or wider than the clause states. No pipeline step verifies the learned policy matches the written constitution. The evaluator is the thing being evaluated — systematic misreading is baked into weights. (S19, S21, S22)

---

## Defects Identified in NeoTrix Design

### DEFECT-1: SEAL Pipeline Uses Scalar Reward Without Binary Flexible Feedback
**Severity**: HIGH | **Affects**: NT-MIND (SEAL pipeline), NT-CORE (reasoning engine)

**Finding**: NeoTrix's SEAL pipeline and reasoning engine use scalar reward signals (`min_reward_threshold: 0.3` in `auto_crystallizer.rs`, `step_reward` in `engine_core.rs`, `outcome_reward` in gate decisions). The 2026 RLBFF paradigm (S1) demonstrates that binary flexible feedback — extracting answerable principles from natural language feedback and training RM as entailment — significantly outperforms scalar Bradley-Terry models. NeoTrix's scalar reward cannot capture nuanced aspects of response quality beyond a single number, and is vulnerable to the same reward hacking that RLBFF addresses.

**Evidence**: `nt_mind/auto_crystallizer.rs:14` — `min_reward_threshold: f64` scalar gate. `nt_mind/reason/engine_core.rs:517` — `reward = feedback_trace.convergence * 0.6 + feedback_trace.final_quality * 0.4` scalar blend. `nt_core_gate/mod.rs:1061` — `reward_boost = traj.outcome_reward.unwrap_or(0.0).max(0.0) * 0.15` scalar boost.

**Recommendation**: Implement `BinaryFlexibleFeedback` adapter in NT-MIND that: (1) extracts binary-answerable principles from SEAL trajectory outcomes (e.g., "did the distillation reduce token count?" → Yes/No, "does the skill crystallization generalize to 3+ domains?" → Yes/No), (2) trains a lightweight entailment-based reward scorer instead of scalar regression, (3) allows principle selection at inference time to customize reward focus per SEAL phase. This directly follows RLBFF's open-source recipe adapted to NeoTrix's domain.

---

### DEFECT-2: No Generative RM for Interpretability in Evolution Decisions
**Severity**: MEDIUM | **Affects**: NT-MIND (distillation), NT-CORE (ConsciousnessTree)

**Finding**: NeoTrix's gate and reasoning systems produce scalar scores but no natural language critique explaining WHY a reward was assigned. The 2026 RRC work (S2) and RM-NLHF (S8) show that generative reward models producing critiques significantly improve interpretability and debuggability of RL training. NeoTrix's SEAL pipeline evolves without explainable feedback — when crystallization fails, there is no critique explaining what went wrong.

**Evidence**: `nt_core_gate/mod.rs` — PanelJudge produces verdicts with reasoning but these are not fed back as process rewards to SEAL. `engine_core.rs:134` — PRM exists but is not used as a generative RM for evolution feedback.

**Recommendation**: Add `CritiqueGenerator` trait to NT-MIND that: (1) takes scalar reward + trajectory context, (2) generates natural language critique explaining the reward assignment, (3) feeds critique similarity into process reward (following RM-NLHF's F1-based similarity approach). Store critiques in KB `experience` namespace for cross-session learning.

---

### DEFECT-3: No Curriculum Learning for SEAL Pipeline Stages
**Severity**: HIGH | **Affects**: NT-MIND (SEAL), NT-CORE (evolution)

**Finding**: NeoTrix's SEAL pipeline runs exploration → distillation → self-test → absorption as a fixed sequence. The 2026 Curriculum-RLAIF work (S16) demonstrates that training data difficulty ordering (easy-to-hard) significantly improves reward model generalizability. NeoTrix does not control the difficulty of examples presented at each SEAL stage, nor does it grade difficulty to enable curriculum progression. This leads to: hard examples overwhelming early stages, easy examples providing no learning signal in later stages, and no smooth difficulty progression.

**Evidence**: SEAL pipeline stages in `seal/` directory — no difficulty grading mechanism. `auto_crystallizer.rs` — `min_reward_threshold` is a static cutoff, not a dynamic curriculum.

**Recommendation**: Implement `SealCurriculum` module that: (1) grades input difficulty using both internal (model behavior entropy) and external (RM-based scoring) perspectives following Curriculum-RLAIF's dual assessment, (2) constructs easy→hard progression for each SEAL stage, (3) allocates stages proportionally (e.g., 25% easy, 25% medium-easy, 25% medium-hard, 25% hard), (4) validates difficulty levels post-hoc to prevent label noise contamination. Wire into SEAL Phase-0 alongside `converge_check`.

---

### DEFECT-4: No Continuous Preference Alignment for Agent Outputs
**Severity**: HIGH | **Affects**: NT-ACT (orchestration), NT-IO (LLM providers)

**Finding**: NeoTrix aligns LLM behavior through static system prompts and Egress Privacy Guard, but performs no online/iterative preference optimization on its own agent outputs. The 2026 frontier recipe (S14) is: SFT → DPO (broad alignment) → online RL (task-specific). NeoTrix has no DPO/online preference loop for its agent outputs (tool calls, code generation, multi-step plans). Agent behavior drifts without correction between SEAL cycles.

**Evidence**: No DPOTrainer or preference dataset in codebase. `nt_core_llm` routes calls but does not optimize policy. `nt_act` modules execute tools but do not learn from preference signals.

**Recommendation**: Implement `AgentPreferenceLoop` in NT-ACT that: (1) collects (prompt, chosen_output, rejected_output) triples from agent task completions (user feedback, quality scoring, timeout/failure as rejection), (2) runs iterative DPO or KTO (for binary thumbs-up/down signals) on a lightweight adapter, (3) updates agent behavior between SEAL cycles. KTO is preferred because NeoTrix's feedback is naturally binary (task succeeded/failed), not paired comparisons.

---

### DEFECT-5: No Dual-KL Regularization — Reward Hacking Risk
**Severity**: HIGH | **Affects**: NT-CORE (reasoning), NT-MIND (SEAL)

**Finding**: NeoTrix's reasoning engine optimizes reward with KL regularization against a reference model but uses a single KL term. The 2026 DAR work (S6) demonstrates that simultaneously regularizing toward both π₀ (reference/SFT) and πₜ (current policy) via dual-KL explicitly addresses the trade-off between reward hacking prevention and stable optimization. NeoTrix's single-KL approach implicitly couples these two objectives, leading to suboptimal exploration and potential reward hacking when the policy diverges.

**Evidence**: `engine_core.rs:522` — `policy.update(reward.clamp(0.0, 1.0))` — single reward signal, no dual-KL mechanism. E8 policy update in `e8_game_reasoning.rs` — uses emotional alignment scaling but no reference-vs-current policy interpolation.

**Recommendation**: Implement dual-KL regularization in the E8 reasoning engine: (1) maintain both π₀ (SFT reference) and πₜ (current policy), (2) compute advantage with KL penalty against both targets, (3) implement the DAR weighted-SFT formulation for simplified optimization. The effective reference target should be an interpolation that moves toward optimal distribution during training.

---

### DEFECT-6: No Constitutional Principles for Self-Evolution Governance
**Severity**: MEDIUM | **Affects**: NT-GOVERNANCE, NT-META, SEAL pipeline

**Finding**: NeoTrix has NT-GOVERNANCE with policy documents and compliance verification, but no constitutional AI mechanism where principles generate self-critique and revision of evolution decisions. The 2026 RLAIF literature (S15, S17, S19) shows that constitutional principles can be used to: (1) generate self-critiques of model outputs, (2) revise outputs to conform to principles, (3) create training data from the critique-revision process. NeoTrix's governance is external (rev-officer checks), not internal (self-critique against principles).

**Evidence**: NT-GOVERNANCE (`gov/steward`) enforces external rules. No self-critique pipeline in SEAL. `consciousness_core.rs` growth reports have no constitutional self-evaluation step.

**Recommendation**: Implement `ConstitutionalSelfCritique` in NT-META that: (1) defines a NeoTrix constitution (principles for safe evolution: "evolution must not degrade existing capabilities", "new modules must pass Dark Forest", "experience absorption must preserve provenance"), (2) after each SEAL stage, critiques the output against all principles, (3) revises outputs that fail critique, (4) stores critique-revision pairs as training data for future alignment. Following Reflect's (S17) approach: inference-time alignment that simultaneously produces reusable supervision data.

---

### DEFECT-7: No Asymmetric Feedback Handling (Loss Aversion)
**Severity**: MEDIUM | **Affects**: NT-MIND (SEAL), NT-FEEL (emotion), NT-ACT

**Finding**: NeoTrix treats reward symmetrically — positive and negative signals have equal weight. The 2026 KTO paradigm (S9, S12, S13) grounded in prospect theory shows that humans weight losses more heavily than equivalent gains, and real feedback pipelines produce asymmetric signals (more failures than successes, more thumbs-down than thumbs-up). NeoTrix's symmetric reward computation ignores this asymmetry, potentially under-weighting failure signals that carry more information.

**Evidence**: `auto_crystallizer.rs:70` — `bin_hallucination` stores reward without asymmetric weighting. `stagnation.rs:462` — zero-reward detection is binary, not prospect-theory-weighted. `engine_core.rs:517` — reward blend is symmetric linear combination.

**Recommendation**: Adopt KTO-inspired asymmetric weighting in SEAL reward computation: (1) detect whether feedback is positive (success, high quality) or negative (failure, hallucination, stagnation), (2) apply asymmetric utility function: losses weighted by λ_l (typically > 1.0), gains by λ_w (typically ~1.0), (3) use the KL-based baseline from KTO to normalize across variable-length sequences. This is especially valuable for NeoTrix because the auto-crystallizer's hallucination binning and stagnation detection already separate positive/negative signals but weight them equally.

---

### DEFECT-8: No Meta-Learned Reward Shaping Across Domains
**Severity**: MEDIUM | **Affects**: NT-MIND (SEAL), NT-CORE (E8 reasoning)

**Finding**: NeoTrix's reward computation is domain-agnostic — the same scalar reward formula applies to coding tasks, reasoning tasks, knowledge absorption, and emotion regulation. The 2026 MeRLa framework (S5) demonstrates that meta-learning a task-aware shaping function across auxiliary tasks before RLHF training provides task-specific learning signals while preserving policy optimality. NeoTrix has multiple domains (7 factions) that could provide auxiliary task signal for reward shaping.

**Evidence**: `engine_core.rs:517` — single reward formula for all task types. `auto_crystallizer.rs:43` — static `min_reward_threshold` for all crystallization domains. No task-specific reward shaping.

**Recommendation**: Implement `DomainAwareRewardShaper` in NT-MIND that: (1) meta-learns a potential-based shaping function Φ(x, y; φ) across NeoTrix's 7 domains (using each domain's successful trajectories as auxiliary tasks), (2) applies domain-specific reward augmentation during SEAL stages, (3) preserves policy optimality via potential-based constraints. This complements the existing Dual Specialization (Weapon Set I/II) by providing task-aware reward signals.

---

### DEFECT-9: No Probability Clamping for Reward Model Stability
**Severity**: MEDIUM | **Affects**: NT-CORE (gate), NT-MIND (SEAL)

**Finding**: NeoTrix's PanelJudge and gate decisions use raw probability scores without bounding. The 2026 R-CAI work (S20) demonstrates that probability clamping (constraining reward probabilities to [c_min, c_max], e.g., [0.4, 0.6]) prevents reward overconfidence and stabilizes optimization while preserving semantic coherence. Without clamping, NeoTrix's reward signals can saturate, leading to repetitive/stylized generation patterns (reward hacking) in the SEAL pipeline.

**Evidence**: `nt_core_gate/mod.rs` — judge scores used raw. `engine_core.rs:517` — reward is a linear blend with no clamping. `auto_crystallizer.rs:100` — `if reward < self.min_reward_threshold` — binary gate, no bounded probability.

**Recommendation**: Add probability clamping to NeoTrix's reward pipeline: (1) clamp judge probabilities to [0.1, 0.9] to prevent overconfidence, (2) apply clamping to SEAL phase transition scores, (3) monitor reward distribution entropy to detect early signs of reward hacking. Following R-CAI's finding: clamping to [0.4, 0.6] improved coherence by 15% and diversity by 42.6% without sacrificing signal strength.

---

### DEFECT-10: No RLHF/DPO Evaluation of NeoTrix's Own Model Interactions
**Severity**: HIGH | **Affects**: NT-IO (LLM providers), NT-ACT (tools), All domains

**Finding**: NeoTrix routes LLM calls through providers (NT-IO) and executes tool calls (NT-ACT) but performs no preference-based evaluation of its own LLM interaction quality. The 2026 landscape (S7, S10, S11) shows that joint optimization of training data and policy — using the model's own interaction traces as preference signal — consistently improves alignment. NeoTrix generates thousands of LLM interactions but treats them as fire-and-forget rather than as a preference dataset for self-improvement.

**Evidence**: `nt_core_llm` — provider routing without quality feedback loop. `nt_act` — tool execution without preference collection. EventBus carries `ExternalReward` events but these are not aggregated into a preference dataset.

**Recommendation**: Implement `InteractionPreferenceCollector` that: (1) captures all LLM interactions as (prompt, response, quality_signal) tuples, (2) quality signals derived from: task success, user feedback, self-consistency checks, tool call success rate, (3) accumulates into a rolling preference dataset stored in KB, (4) periodically runs DPO/KTO fine-tuning on a LoRA adapter to improve NeoTrix's own prompting strategy. This creates a self-improving feedback loop where NeoTrix gets better at using LLMs by learning from its own interactions.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 22 |
| Research findings | 18 (across 3 domains) |
| Defects identified | 10 |
| Critical severity | 0 |
| High severity | 5 |
| Medium severity | 5 |

**Top 3 Priority Defects**:
1. **DEFECT-1**: SEAL pipeline uses scalar reward without binary flexible feedback (HIGH — core evolution mechanism suboptimal)
2. **DEFECT-4**: No continuous preference alignment for agent outputs (HIGH — agent behavior drifts without correction)
3. **DEFECT-5**: No dual-KL regularization — reward hacking risk (HIGH — stability and exploration compromised)

**Key Architectural Insight**: The 2026 alignment landscape has converged on a hybrid paradigm: offline preference optimization (DPO/KTO) for broad alignment + online RL for task-specific optimization + constitutional principles for governance. NeoTrix's SEAL pipeline currently operates as a single scalar-reward RL loop without the multi-paradigm approach. The most impactful upgrades are: (1) replacing scalar reward with binary flexible feedback (DEFECT-1), (2) adding KTO-style asymmetric feedback handling (DEFECT-7), and (3) implementing constitutional self-critique for governance (DEFECT-6). These three changes would bring NeoTrix's evolution mechanism in line with 2026 frontier practices.
