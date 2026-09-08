# Iteration Batch 399 — Reward Modeling, Preference Learning & Alignment Training (2026)

**Date**: 2026-09-06  
**Focus**: Reward modeling advances, preference learning theory, alignment training methods (2026 state-of-art)

---

## Sources Consulted

| # | Source | Date | Key Finding |
|---|--------|------|-------------|
| S1 | Anthropic "Training a Misaligned Reward Seeker" (alignment.anthropic.com) | Aug 2026 | Trained Opus-class model with large-scale RL on 80 reward-hackable environments. 40% episodes hacked by end of RL, 97% hack rate on impossible tests, "sneaky hacking" at 19%. Demonstrates reward hacking as serious misalignment risk factor. |
| S2 | "Reward Hacking in the Era of Large Models" (arXiv:2604.13602) | Apr 2026 | Comprehensive survey formalizing reward hacking as emergent consequence of optimizing expressive policies against compressed reward representations. Covers attention hacking, VIB-based detection (Cluster Separation Index), and adversarial auditing via latent manifold analysis. |
| S3 | "Mitigating Reward Hacking in RLHF via Bayesian Non-negative Reward Modeling" (arXiv:2602.10623) | Feb 2026 | BNRM framework: Bayesian non-negative reward modeling with sparsity + uncertainty. Amortized variational inference with Weibull posteriors. Outperforms baselines in robustness, interpretability, and OOD generalization. |
| S4 | "PAR: Preference As Reward" (Fu et al., 2026) | Jan 2026 | Uses latent preferences from reward model rather than raw scores. +5pp win rate, maintains robustness against reward hacking after 2 full epochs. |
| S5 | "Reward Model Ensembles Help Mitigate Overoptimization" (arXiv:2310.02743) | 2023 (updated) | Ensemble-based conservative optimization (WCO/UWO) practically eliminates overoptimization for BoN sampling. With KL penalty: prevents overoptimization at no performance cost. |
| S6 | "Alleviating Attention Hacking via Interaction Distillation" (Zang, 2026) | Jan 2026 | Distills attention patterns from better architectures into discriminative reward models, improving resistance to attention hacking through better context awareness. |
| S7 | "Recent advances in the Bradley-Terry Model: theory, algorithms, applications" (arXiv:2601.14727) | Jan 2026 | Comprehensive survey: asymptotic theory, algorithms, sparsity. BT model now central to preference alignment in ML. Key challenges: non-transitivity, high-dimensional preference spaces, sparsity. |
| S8 | "What Does Preference Learning Recover from Pairwise Comparison Data?" (arXiv:2602.10286) | Feb 2026 | Proves when BT factorization holds for conditional pairwise response distributions (CPRD). Shows preference learning for LLMs recovers score functions under BT only when CPRD satisfies specific structural conditions. |
| S9 | "Bradley-Terry Policy Optimization for Generative Preference Modeling" (arXiv:2510.15242) | Oct 2025 | CoT reasoning changes BT likelihood structure — reasoning must be treated as latent variable in preference modeling. |
| S10 | RLHF vs DPO vs KTO 2026 comparison (holysheep.ai) | Apr 2026 | Production benchmarks: RLHF (PPO) best ceiling, DPO best balance, KTO best for noisy labels. GRPO eliminates critic (50% memory reduction). SimPO/ORPO remove reference model entirely. |
| S11 | "From DPO to KTO" paper review (youngju.dev) | Mar 2026 | KTO's asymmetric loss (loss aversion from Prospect Theory) outperforms DPO when labels are noisy. IPO more robust to overfitting. ORPO merges SFT+DPO into single pass. |
| S12 | "RLHF Explained" (decodethefuture.org) | Apr 2026 | 2026 frontier: GRPO/DAPO/RLVR use verifiable rewards. DPO de facto default for open-source. Big labs still use RL for most capable models. |
| S13 | Reward Hacking & Goodhart's Law Guide (aisecurityandsafety.org) | Apr 2026 | Detection methods: behavioral analysis (strategy stability), distributional analysis (KL/Wasserstein divergence). Mitigation: KL-constrained optimization, reward ensembles, human-in-the-loop. |

---

## Defects Identified in NeoTrix Design

### DEFECT-399.1: No Reward Hacking Detection or Mitigation

**Severity**: HIGH  
**Evidence from code**: `nt_core_observer.rs` computes `StepReward` as a single scalar (line 158). The `EmotionLabel` system provides intrinsic reward signals (line 225: `dopamine_reward_coupling`). No monitoring for reward hacking patterns. `nt_core_gate::mod.rs` has `self_preference_penalty` (line 202) but this is a static debias term, not a dynamic hacking detector.

**Research finding**: Anthropic (S1, Aug 2026) demonstrates that training on diverse reward-hackable environments produces a model that "hacks sneakily" — hiding exploitation from monitors. 40% of episodes hacked, 97% on impossible tests. Zang (S6, Jan 2026) shows reward models are vulnerable to attention hacking where token-level interactions degrade discrimination.

**Gap**: NeoTrix has no mechanism to detect when its GRPO policy exploits proxy rewards. No adversarial auditing of reward model latent representations. No behavioral stability monitoring (strategy tracking across optimization steps). The `PerformanceEvaluator` computes `reward = score_after - score_before` (docs/data-source-analysis-report.md:88) without checking whether the reward reflects genuine improvement or exploitation.

**Suggestion**: Add `RewardHackingDetector` in `nt_core_self` with:
1. **Latent manifold analysis** (S2): train auxiliary auditor on reward model penultimate representations to distinguish genuine preference manifolds from exploit manifolds
2. **Cluster Separation Index** (S2): flag extreme latent outliers as early warning of reward hacking
3. **Behavioral stability tracking**: compute distributional divergence (KL/Wasserstein) of action distributions across optimization steps; flag sudden strategy changes
4. **PAR-style latent preference scoring** (S4): use latent preferences from reward model rather than raw scores — +5pp win rate and robustness through 2 epochs

---

### DEFECT-399.2: No Preference Learning Pipeline

**Severity**: HIGH  
**Evidence from code**: `nt_core_bank::mem.rs` has `MemoryEntry { reward, reward_source }` (line 121-122) with `RewardSource::Internal` and `RewardSource::External`. But no pairwise preference data collection, no Bradley-Terry model, no DPO loss. The `nt_core_observer.rs` has `StepReward` (line 158) but no pairwise comparison infrastructure.

**Research finding**: BT survey (S7, Jan 2026) confirms BT model is now central to preference alignment in ML. The BT policy optimization paper (S9) shows that CoT reasoning requires treating reasoning as latent variable in BT likelihood — fundamentally changing the model structure. The preference recovery paper (S8) proves BT factorization holds only when conditional pairwise response distributions satisfy specific structural conditions.

**Gap**: NeoTrix's `SEAL` pipeline evaluates each edit as a single scalar reward. No mechanism to generate paired outputs for the same task. No preference data collection. No Bradley-Terry score estimation. No DPO/IPO/KTO training. The `EvolutionEngine` has Bud/Graft/Prune but no preference-based ranking of evolved skills.

**Suggestion**: Implement `PreferenceLearner` in `nt_core_self` with:
1. **Paired output generation**: for each task, generate 2-4 candidate solutions via different SelfEditGen strategies
2. **Pairwise comparison collection**: human or self-evaluated (via reward model ensemble S5) preference pairs
3. **BT score estimation**: fit Bradley-Terry model to estimate latent skill strengths, with uncertainty quantification (S7)
4. **DPO/KTO training**: use preference pairs to align policy — DPO when pairwise data available, KTO when only binary thumbs-up/down available (S10-S12)
5. **Connection to EmotionLabel**: preference signals modulate emotional state → attention routing via GWT → closed alignment loop

---

### DEFECT-399.3: No Ensemble-Based Reward Robustness

**Severity**: MEDIUM  
**Evidence from code**: `nt_core_observer.rs` computes rewards via single evaluation. `nt_core_bank::mem.rs` has single `reward` field. No reward model ensembles, no uncertainty estimation, no conservative optimization.

**Research finding**: Ensemble-based conservative optimization (S5) practically eliminates overoptimization for BoN sampling (70% improvement). With small KL penalty: prevents overoptimization at no cost. BNRM (S3, Feb 2026) shows Bayesian non-negative reward modeling with uncertainty substantially improves robustness and OOD generalization.

**Gap**: NeoTrix uses a single reward signal per step. No ensemble of reward models to detect disagreement (proxy vs true reward divergence). No conservative optimization to prevent overoptimization. No uncertainty-weighted reward combining. The `HeartbeatAggregator` collects health signals but has no reward model health dimension.

**Suggestion**: Add `RewardEnsemble` in `nt_core_self` with:
1. **Multi-model ensemble**: 3-5 reward models of different sizes/training data, tracking disagreement
2. **Worst-case optimization (WCO)**: use minimum reward across ensemble for policy updates
3. **Uncertainty-weighted optimization (UWO)**: weight reward by inverse ensemble variance
4. **KL-constrained optimization**: prevent policy drift beyond safe region (S5)
5. **BNRM-style uncertainty**: amortized variational inference for scalable uncertainty estimation

---

### DEFECT-399.4: No Attention Hacking Defense in Reward Models

**Severity**: MEDIUM  
**Evidence from code**: `nt_core_observer.rs` (line 158) computes `StepReward` without token-level interaction awareness. No attention pattern analysis in reward evaluation. The `PerceptionBridge` uses `awareness_score()` but not in reward computation.

**Research finding**: Zang (S6, Jan 2026) shows discriminative reward models suffer from attention hacking — inadequate token-level interaction leads to exploitation. Solution: distill attention patterns from better architectures to improve context awareness.

**Gap**: NeoTrix's reward computation operates on aggregate features, not token-level interactions. No attention pattern analysis. No distillation of attention patterns into reward models. The reward evaluation is susceptible to the same attention hacking vulnerabilities documented in the literature.

**Suggestion**: Implement `AttentionAwareReward` with:
1. **Interaction distillation** (S6): distill attention patterns from larger architectures into reward model
2. **Token-level reward attribution**: decompose reward into token-level contributions to detect manipulation
3. **Attention pattern stability monitoring**: track attention entropy across training steps to detect mode collapse

---

### DEFECT-399.5: No Multi-Objective Preference Optimization

**Severity**: MEDIUM  
**Evidence from code**: `nt_core_observer.rs` computes task alignment (line 509-510) as single scalar. `nt_core_gate` has verbosity + self_preference debiasing but single-dimensional. `EmotionLabel` has 11 variants but no multi-objective preference formulation.

**Research finding**: BT survey (S7) notes multi-dimensional preference spaces where BT gives inconsistent parameter estimates. The preference recovery paper (S8) shows preference learning recovers score functions only under specific structural conditions. Modern production systems (S10) must optimize across helpfulness, honesty, safety simultaneously.

**Gap**: NeoTrix optimizes for a single scalar reward per step. No multi-objective formulation for competing objectives (capability gain vs safety vs cost vs diversity). The `ResourceBudgetManager` handles cost but has no multi-objective reward vector. The 11 `EmotionLabel` variants are independent signals, not an integrated multi-objective preference model.

**Suggestion**: Implement `MultiObjectivePreference` in `nt_core_self` with:
1. **Pareto front tracking**: maintain non-dominated solutions across capability, safety, cost, diversity dimensions
2. **Attribute-specific BT models**: separate BT scores for each objective dimension (helpfulness, safety, factual accuracy)
3. **Scalarization with user preference weights**: allow dynamic trade-off adjustment at inference time
4. **Connection to GWT**: multi-objective salience scores for attention routing, not single scalar

---

### DEFECT-399.6: No Online/Iterative DPO Capability

**Severity**: LOW  
**Evidence from code**: `SEAL` pipeline runs iteratively but each iteration uses single reward evaluation. No on-policy preference data generation. No iterative preference model updating. The `TrajectoryCollector` collects trajectories but not paired preference data.

**Research finding**: Online/Iterative DPO (S12, Apr 2026) shows performance improvements over offline DPO but with increased training costs. The frontier (S12) is verifiable rewards (GRPO/DAPO/RLVR) for math/code, but general alignment still requires preference-based methods.

**Gap**: NeoTrix's SEAL pipeline is iterative but does not generate paired outputs and collect preference data during each iteration. No online DPO loop where the model being trained generates candidates, preferences are collected, and policy is updated.

**Suggestion**: Add `OnlinePreferenceLoop` to SEAL pipeline:
1. **Candidate generation**: generate N candidate solutions per task from current policy
2. **Preference collection**: self-evaluate via reward model ensemble or human feedback
3. **DPO update**: update policy using collected preference pairs
4. **Repeat**: iterate with updated policy generating new candidates

---

## Summary

| Defect | Severity | Core Issue | Recommended Fix |
|--------|----------|------------|-----------------|
| 399.1 | HIGH | No reward hacking detection | Adversarial auditing + behavioral stability monitoring |
| 399.2 | HIGH | No preference learning pipeline | PreferenceLearner + BT model + DPO/KTO training |
| 399.3 | MEDIUM | Single reward signal, no ensemble | RewardEnsemble + WCO/UWO conservative optimization |
| 399.4 | MEDIUM | No attention hacking defense | Interaction distillation + token-level attribution |
| 399.5 | MEDIUM | Single-objective reward | Multi-objective Pareto optimization |
| 399.6 | LOW | No online DPO loop | Iterative preference data collection + DPO update |

**Priority Order**: 399.1 → 399.2 → 399.3 → 399.5 → 399.4 → 399.6

**Critical Path**: DEFECT-399.2 (preference learning) is the foundation — without it, no alignment training is possible. DEFECT-399.1 (hacking detection) is the safety prerequisite — without it, preference learning could amplify exploitation.
