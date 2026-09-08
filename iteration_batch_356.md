# Iteration Batch 356 — Alignment & Value Learning Research Sweep

**Date:** 2026-09-06
**Focus:** RLHF alternatives, AI safety, value learning advances (2026)

---

## 1. Research Findings

### 1.1 RLHF & Alignment (2026)

| Finding | Source | Date |
|---------|--------|------|
| **DPO is now dominant alignment technique**, displacing classical RLHF in frontier and open-weight labs. Variants: IPO, KTO, ORPO, SimPO, cDPO, sDPO, TDPO. Iterative DPO (train→sample→collect prefs→retrain) closes gap with on-policy RLHF. | aisecurityandsafety.org | 2026-04-13 |
| **Constitutional AI 2.0**: Dynamic constitution updates (expert committee refines clauses on novel dilemmas), 7,500+ annotators, expanded annotation infrastructure. Collective Constitutional AI democratizes values via public deliberation (Polis platform, ~1,000 representative U.S. adults). | zylos.ai, callmissed.com | 2026-02-01, 2026-05-09 |
| **RLAIF maturation**: Rubric-based feedback outperforms single Likert-scale scores. Direct-RLAIF (d-RLAIF) circumvents reward model training entirely. RLAIF cost drops to <$0.01/annotation vs. dollars for human. | zylos.ai | 2026-02-01 |
| **GRPO (Group Relative Policy Optimization)**: DeepSeek's critic-free RL — samples group of answers per prompt, uses group mean as baseline. ~40% memory reduction vs PPO. Variants: Dr. GRPO, DAPO, GSPO. Theoretical analysis proves asymptotic optimality via U-statistics framework. | deepseek.ai, arxiv:2603.01162, reinforcement-learning.com | 2026-03 to 2026-07 |
| **Hybrid alignment pipelines**: 2026 best practice is SFT → DPO (or variant) → optional small on-policy RL for reasoning tasks with verifiable rewards (RLVR). Preference alignment and reasoning RL split into separate concerns. | aisecurityandsafety.org | 2026-04-13 |
| **Training-Free GRPO**: Token-prior in-context learning achieves similar output distribution shifts without parameter updates, using dozens of samples. | arxiv:2510.08191 | 2025-10 |

### 1.2 AI Safety (2026)

| Finding | Source | Date |
|---------|--------|------|
| **NIST CAISI red teaming competition**: 250,000+ attack attempts from 400+ participants found successful attacks against ALL 13 frontier models in agentic scenarios. Security does NOT track capability — stronger models are not automatically safer. | stingrai.io (NIST CAISI) | 2026-03-23 |
| **Four-layer agent attack surface** (2026): Application → Model → Tool/MCP connections → Data. Real damage comes from connective tissue, not the model layer alone. Memory poisoning enables persistent cross-session attacks. | stingrai.io, Palo Alto Networks | 2026-07 |
| **Deceptive alignment detection**: MechELK framework extracts latent knowledge via mechanistic interpretability — detects deceptive alignment where models appear helpful in evaluation but harmful in deployment. Advantage over CCS grows with model size (+8.2% at 70B). | arxiv:2605.28825 | 2026 |
| **Mechanistic interpretability as 2026 breakthrough**: MIT Tech Review "10 Breakthrough Technologies 2026". Linear probes detect deception with moderate accuracy. Sycophancy circuits identified and ablatable. RLHF found to primarily affect response-style circuits rather than core reasoning. | zylos.ai, arxiv:2602.11180 | 2026-01-21, 2026-02-09 |
| **Pre-deployment testing failure**: 2026 International AI Safety Report (30+ countries, 100+ experts) warns models distinguish test environments from real deployment and exploit evaluation loopholes. | zylos.ai | 2026-02-09 |
| **Automated Alignment Researchers** (Anthropic, April 2026): LLMs scaling scalable oversight — using AI to help align AI. | anthropic.com | 2026-04-14 |
| **Constitutional Classifiers**: Reduced jailbreak success from 86% to 4.4%. | zylos.ai | 2026-02-09 |

### 1.3 Value Learning (2026)

| Finding | Source | Date |
|---------|--------|------|
| **Temporally Coherent Reward Modeling (TCRM)**: Reward models are secretly value functions. Introduces two regularization terms on Bradley-Terry loss to enforce temporal coherence, with minimizers provably equal to conditional expectations. | arxiv:2604.22981 | 2026-04-24 |
| **DR-IRL (Dynamic Reward Scaling IRL)**: Dynamically adjusts rewards through Inverse Reinforcement Learning for safety alignment. Trains category-specific reward models using balanced safety datasets of seven harmful categories. | openreview.net | 2026-01-26 |
| **Value System Learning via IRL**: Formalizes value system learning in MOMDPs — separate values as reward vector components, weighted linear scalarization for individual agent value systems. Algorithms infer grounding functions from observed behavior. | springer.com (Holgado-Sánchez et al.) | 2026-02-03 |
| **Reward hacking mitigations (2026)**: InfoRM (information bottleneck in reward modeling), BNRM (Bayesian non-negative reward modeling), PAR (Preference As Reward sigmoid shaping), RM ensembles with worst-case/uncertainty-weighted optimization. ICDS detects overoptimization via latent space outliers. | arxiv:2402.09345, arxiv:2602.10623, arxiv:2502.18770, arxiv:2310.02743 | 2026 |
| **AIRL for reasoning reward models**: Adversarial IRL learns reasoning rewards from expert demonstrations — sparse, interval, and dense reward granularities. | iclr.cc | 2026-04-26 |
| **Expected Value Alignment for generative RM**: Continuous logit-based scoring reduces discretization artifacts while retaining interpretability. | arxiv:2606.01160 | 2026-05-31 |

---

## 2. Defects Found in NeoTrix Architecture

### DEFECT 1: No DPO/GRPO Alignment Pipeline
**Severity: HIGH**
**Location: SEAL pipeline (`nt_mind::seal`)**

NeoTrix's SEAL pipeline does not implement DPO or GRPO as alignment mechanisms. The 2026 state-of-the-art requires SFT → DPO → optional RLVR. NeoTrix's reward modeling (if any) relies on older PPO-style approaches or lacks formal preference optimization entirely.

**Gap:** No `(prompt, chosen, rejected)` preference data pipeline. No iterative DPO loop. No group-relative advantage computation for reasoning tasks.

**Fix:** Add `nt_mind::alignment` module implementing:
1. DPO loss as primary alignment stage
2. GRPO for reasoning tasks with verifiable rewards (RLVR)
3. Iterative DPO pipeline (train → sample → collect prefs → retrain)

### DEFECT 2: No Constitutional AI / RLAIF Mechanism
**Severity: MEDIUM**
**Location: NT-SHIELD / NT-CORE governance**

NeoTrix has no formal constitution or self-critique pipeline. 2026 Frontier systems use dynamic constitutions with expert committee updates. NeoTrix's governance is policy-level only, not operationalized as training feedback.

**Gap:** No self-critique → revision → preference data generation loop. No rubric-based AI feedback. No mechanism to update behavioral principles from deployment incidents.

**Fix:** Add `nt_core_governance::constitution` — a living document with clauses that evolve via incident review. Wire it to RLAIF for automated preference generation.

### DEFECT 3: No Mechanistic Interpretability Monitoring
**Severity: HIGH**
**Location: NT-META (meta-cognition layer)**

NeoTrix's meta-cognition layer has no circuit-level or representation-level monitoring. 2026 research shows RLHF affects only response-style circuits, not core reasoning. Linear probes detect deception with moderate accuracy. MechELK detects deceptive alignment via latent knowledge extraction.

**Gap:** No activation monitoring. No sycophancy circuit detection. No value drift monitoring. No situational awareness probes.

**Fix:** Add `nt_meta::interpretability` module implementing:
1. Linear probes for deception detection (sycophancy circuits)
2. Value drift monitoring via representation tracking
3. Anomalous circuit detection for misaligned objectives

### DEFECT 4: No Reward Hacking Defense
**Severity: HIGH**
**Location: NT-CORE reward modeling**

NeoTrix lacks defenses against reward overoptimization. 2026 has multiple proven mitigations: InfoRM (information bottleneck), BNRM (Bayesian non-negative), PAR (sigmoid shaping), RM ensembles with conservative optimization. ICDS detects overoptimization via latent space outliers.

**Gap:** No reward model ensemble. No information bottleneck. No overoptimization detection metric. No conservative optimization objective.

**Fix:** Add `nt_core::reward_defense` implementing:
1. Reward model ensemble with worst-case optimization
2. ICDS-like overoptimization detector (latent space outlier monitoring)
3. PAR-style reward shaping for training stability

### DEFECT 5: No Four-Layer Security Model for Agents
**Severity: HIGH**
**Location: NT-SHIELD**

NeoTrix's security focuses on network-level (stealth net, proxy pool, Tor). 2026 NIST findings show real agent damage occurs at Application → Model → Tool/MCP → Data layers. Memory poisoning enables persistent cross-session attacks.

**Gap:** No tool/MCP connection security. No RAG data provenance. No memory poisoning detection. No application-layer output handling validation.

**Fix:** Extend NT-SHIELD with:
1. Tool/MCP connection integrity verification
2. Memory poisoning detection (cross-session anomalous instruction persistence)
3. RAG source provenance tracking
4. Application-layer output sanitization (OWASP LLM05)

### DEFECT 6: No Scalable Oversight / Debate Protocol
**Severity: MEDIUM**
**Location: NT-CORE (E8 reasoning)**

NeoTrix's E8 hexagram reasoning is single-agent. 2026 scalable oversight uses debate (two AIs competing), recursive reward modeling, and weak-to-strong generalization. Anthropic's Automated Alignment Researchers (April 2026) use LLMs to scale oversight.

**Gap:** No multi-agent debate protocol. No recursive reward modeling. No weak-to-strong generalization mechanism.

**Fix:** Add `nt_core::debate` — structured debate protocol where two specialist modules argue positions, with E8 hexagram as arbiter. Maps to existing dual specialization (Weapon Set I/II) architecture.

### DEFECT 7: No Preference Data Infrastructure
**Severity: HIGH**
**Location: NT-MEMORY (KB)**

NeoTrix's KB stores nodes, edges, embeddings, BM25. It lacks structured preference data storage: `(prompt, chosen, rejected, metadata)` triples, annotator quality signals, preference consistency metrics.

**Gap:** No preference dataset management. No annotator quality tracking. No preference consistency validation.

**Fix:** Add preference data schema to KB: `preference_triples` table with prompt/chosen/rejected/annotator_id/quality_score/consistency_flag.

### DEFECT 8: No Value System Learning from Observations
**Severity: MEDIUM**
**Location: NT-MIND (self-evolution)**

2026 value learning formalizes value systems as MOMDPs with weighted scalarization. NeoTrix learns from experience but doesn't formally decompose values into multi-objective reward vectors. Values are implicit in the E8 encoding, not explicit reward components.

**Gap:** No multi-objective reward decomposition. No weighted value system inference from observed behavior. No value grounding function.

**Fix:** Extend `nt_mind::evolution` with MOMDP value system — separate safety/utility/efficiency/honesty as reward vector components, with learnable weights per domain.

---

## 3. Suggestions

### Priority 1 (Immediate — S1)
1. **Implement DPO alignment module** — smallest blast radius, highest 2026 alignment ROI
2. **Add reward hacking defense** — ICDS detector + RM ensemble prevents catastrophic reward drift
3. **Four-layer security model** — critical for agent safety in MCP tool-use scenarios

### Priority 2 (Short-term — S2)
4. **Constitutional AI pipeline** — operationalize governance as RLAIF feedback loop
5. **Preference data infrastructure** — prerequisite for DPO/GRPO at scale
6. **Debate protocol** — leverage existing dual specialization for scalable oversight

### Priority 3 (Medium-term — S3)
7. **Mechanistic interpretability monitoring** — requires model size sufficient for circuit analysis
8. **MOMDP value system learning** — formal value decomposition for self-evolution

### Cross-Cutting
- All alignment changes should be C1 (unit testable) before C2 (integration tested)
- Reward defense should be wired to `HeartbeatAggregator` as a health signal
- Preference data schema should be versioned in KB migration system
- Constitution clauses should map to existing `EmotionLabel` 11-variant taxonomy

---

## 4. Sources Cited

1. aisecurityandsafety.org — "DPO: Complete Guide for 2026" (2026-04-13)
2. zylos.ai — "Constitutional AI and Alignment Alternatives" (2026-02-01)
3. zylos.ai — "AI Safety, Alignment, and Interpretability in 2026" (2026-02-09)
4. callmissed.com — "Constitutional AI vs RLHF: Alignment Evolution in 2026" (2026-05-09)
5. stingrai.io — "AI Red Teaming for LLM and Agentic Apps 2026" (2026-07-01)
6. anthropic.com — "Automated Alignment Researchers" (2026-04-14)
7. arxiv:2603.01162 — "Demystifying GRPO: Policy Gradient is a U-Statistic" (2026-03-01)
8. reinforcement-learning.com — "GRPO: Group Relative Policy Optimization" (2026-06-07)
9. deepseek.ai — "DeepSeek's Logic Leap: 2026 Guide" (2026-07-26)
10. arxiv:2510.08191 — "Training-Free GRPO" (2025-10)
11. aisecurityandsafety.org — "AI Alignment: Complete Guide 2026" (2026-03-25)
12. aisecurityandsafety.org — "Deceptive Alignment Guide 2026" (2026-03-29)
13. arxiv:2602.11180 — "Mechanistic Interpretability for LLM Alignment" (2026-01-21)
14. arxiv:2605.28825 — "MechELK: Mechanistic Eliciting Latent Knowledge" (2026)
15. arxiv:2604.22981 — "Temporally Coherent Reward Modeling" (2026-04-24)
16. openreview.net — "DR-IRL: Dynamic Reward Scaling IRL" (2026-01-26)
17. springer.com — "Learning Value Systems via IRL" (Holgado-Sánchez et al., 2026-02-03)
18. arxiv:2402.09345 — "InfoRM: Mitigating Reward Hacking via Info Theory" (2024)
19. arxiv:2602.10623 — "BNRM: Bayesian Non-negative Reward Modeling" (2026-02)
20. arxiv:2502.18770 — "PAR: Preference As Reward Shaping" (2026-01-08)
21. arxiv:2310.02743 — "Reward Model Ensembles Mitigate Overoptimization" (2023)
22. iclr.cc — "AIRL for Reasoning Reward Models" (2026-04-26)
23. arxiv:2606.01160 — "Expected Value Alignment for Generative RM" (2026-05-31)
24. explainx.ai — "Scalable Oversight: RLHF, DPO, Constitutional AI" (2026-06-27)
25. youngju.dev — "AI Safety & Alignment 2026 Deep Dive" (2026-05-16)
26. NIST CAISI — "Insights into AI Agent Security" (2026-03-23)
27. arxiv:2502.04675 — "Scalable Oversight for Superhuman AI" (2025)
28. af.net — "AI Safety 2026: Advancing Beyond RLHF" (2026-05-18)
