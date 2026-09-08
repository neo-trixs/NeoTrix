# Iteration Batch 589 — Personalization / Recommendation / User Profiling

## NEW vs Batch 588

Batch 588 covered secrets lifecycle, deadlock cascades, feature flag state explosion. Batch 589 pivots to **personalization infrastructure** — the missing layer between NeoTrix's agent awareness and actual user-facing intelligence.

---

## Source 1: Hierarchical User Modeling (PHF Framework)

**Source**: Wang et al., "Beyond Isolated Behaviors: Hierarchical User Modeling for LLM Personalization" (arXiv:2606.02300, 2026-06)

**Key Finding**: Flat behavioral paradigms (aggregating user behaviors without hierarchical structure) are insufficient for LLM personalization. PHF proposes three levels: **Practice** (individual behaviors) → **Habitus** (temporally accumulated stable dispositions) → **Field** (shared regularities across similar users). K-Means clustering over habitus representations induces latent fields.

**NEW Defect vs Batch 588**: NeoTrix's SelfModel types (`nt_core_meta::SelfModel`, `nt_core_self::SelfModel`, `nt_core_self_model::SelfModel`) treat user/system identity as flat three-way split (structural identity, performance model, value model). There is **no hierarchical behavioral abstraction** — no Practice→Habitus→Field decomposition. The ConsciousnessTree tracks module health but not behavioral disposition formation over time. **Defect: behavioral personalization data has no structural home in the 6-layer architecture.**

---

## Source 2: IRIS — Dynamic User Personas from Implicit Streams

**Source**: "Learning Dynamic User Personas from Implicit Interaction Streams via Iterative Refinement" (arXiv:2607.26473, 2026-07)

**Key Finding**: IRIS learns user personas entirely from implicit interaction logs — no labels, demographics, or explicit ratings. Uses a closed-loop: memory extraction → persona inference → behavior prediction → prediction error → persona refinement. A **stability gate** prevents oscillation from noisy individual events. Achieves 61.0% decision prediction accuracy on real Reddit data.

**NEW Defect vs Batch 588**: NeoTrix has no prediction-error-driven persona refinement loop. The SEAL pipeline evolves architecture, not user understanding. The experience-tree absorption writes to KB but has no feedback mechanism where **user behavior prediction failure triggers persona model updates**. **Defect: no closed-loop persona refinement; user models are static snapshots, not continuously adapted.**

---

## Source 3: HypReflect — Hypotheses-Guided Self Distillation

**Source**: "Hypotheses-Guided Self Distillation for Continual Personalization" (arXiv:2609.00251, 2026-08)

**Key Finding**: Infers explicit, **uncertainty-aware preference hypotheses** from diverse user signals. Reflectively refines them as evidence accumulates. Incorporates user model through hypotheses-guided self-distillation. Generalizes to unseen users and cross-domain settings.

**NEW Defect vs Batch 588**: NeoTrix's EmotionLabel (11 variants) captures system emotion but not **user preference uncertainty quantification**. There is no mechanism to represent confidence intervals on inferred user preferences. When the system is uncertain about a user, it has no way to express that uncertainty or route to information-gathering actions. **Defect: no uncertainty quantification on user preference models; system acts on point estimates without confidence.**

---

## Source 4: HORIZON — Cross-Domain Sequential Recommendation Benchmark

**Source**: ACL 2026 Findings, "HORIZON: A Benchmark for in-the-wild User Behavior Modelling" (aclanthology.org/2026.findings-acl.1503)

**Key Finding**: 54M users, 35M items across domains. Key insight: **strong in-distribution results do not reliably translate to real-world robustness**. BERT4Rec degrades significantly for out-of-distribution users. LLMs do not consistently outperform specialized architectures. Models need to handle unseen users, temporally distant scenarios, and item turnover (only 60% catalog overlap).

**NEW Defect vs Batch 588**: NeoTrix has no cross-domain user behavior transfer mechanism. The KB stores knowledge but not **cross-domain preference transfer models**. When a user's behavior in one domain (e.g., code tasks) should inform recommendations in another (e.g., content creation), there's no bridging mechanism. **Defect: no cross-domain preference transfer; each domain's user model is siloed.**

---

## Source 5: KGERA — Knowledge Graph Enhanced Reasoning for Recommendations

**Source**: Nature Scientific Reports, "KGERA: knowledge graph enhanced reasoning architecture for recommendation systems" (2026-03)

**Key Finding**: Test-time reasoning over structured knowledge graph achieves 50-55% improvement over baselines. Critically, **negative coefficients on some ensemble components** (PopRec, KGE) indicate they function as debiasers, not relevance amplifiers — easy to misinterpret if you read coefficients as "good" vs "bad." 10ms per-user inference. Combines 9 heterogeneous recommenders through interpretable stacking.

**NEW Defect vs Batch 588**: NeoTrix's GWT attention routing broadcasts salient information but has **no negative-signal debiasing mechanism**. When multiple specialist modules contribute attention signals, there's no interpretable fusion that identifies which signals are **counterbalancing systematic biases** vs amplifying relevance. The HeartbeatAggregator collects health signals but doesn't model them as an interpretable ensemble with coefficient-level transparency. **Defect: no interpretable negative-signal debiasing in cross-module attention fusion.**

---

## Source 6: PersonaMem-v3 — Omni-Platform Personal Intelligence

**Source**: "PersonaMem-v3: Toward Omni-Platform Personal Intelligence" (arXiv:2608.21381, 2026-07)

**Key Finding**: Seeds from 1M+ anonymized engagement histories across social media, chatbot, calendar, AI-companion. Evaluates whether agents can: (1) infer holistic user understanding from cross-platform evidence, (2) personalize responses, (3) hold back when personalization would be inappropriate/repetitive/outdated/unnecessary. **Over-personalization** is explicitly modeled as a failure mode.

**NEW Defect vs Batch 588**: NeoTrix has no **personalization restraint mechanism**. The system has no concept of "hold back when personalization is inappropriate." There's no deduplication of personalized responses, no staleness detection on user models, no privacy-budget enforcement on cross-platform inference. The Egress Privacy Guard prevents NeoTrix's source code from leaking outward but has **no inward-facing restraint on how aggressively the system infers and uses user data**. **Defect: no over-personalization guard; system has no mechanism to deliberately NOT personalize.**

---

## Source 7: Collaborative Filtering vs Content-Based (NVECTA Guide)

**Source**: NVECTA, "Collaborative Filtering vs Content-Based vs Hybrid" (2026-05)

**Key Finding**: The failure modes of collaborative and content-based filtering are **almost perfectly complementary**. CF struggles when data is sparse; CB doesn't. CB struggles to push users beyond existing tastes; CF doesn't. Hybrid systems route each user/item through whichever approach has sufficient data. Key: **switching strategy** (CF for users with history, CB for new users) is the most practical hybrid for production.

**NEW Defect vs Batch 588**: NeoTrix has no **cold-start routing strategy** for user interactions. When a new user engages with the system, there's no mechanism to fall back from behavioral-model-based personalization to content/similarity-based personalization. The system either has behavioral data or doesn't — no graceful degradation. **Defect: no cold-start routing; binary personalization mode with no fallback.**

---

## Source 8: Persona Planning — Living Infrastructure (CleverX/Clayton Johnson)

**Sources**: CleverX "How to Create User Personas (2026)" (2026-01), Clayton Johnson "Target Audience Persona" (2026-07), Koji "User Personas in 2026" (2026-05)

**Key Finding**: Personas must be **living infrastructure**, not deliverables. Review every 90 days. "Persona drift" is a real phenomenon — personas from 2024 may be misleading by 2026. Best practice: 3-5 primary personas max; more signals over-segmentation. Anti-personas (who you are NOT building for) are as valuable as primary personas. Demographics alone are weak predictors; goals and jobs-to-be-done are strong predictors.

**NEW Defect vs Batch 588**: NeoTrix's domain terms (CONTEXT.md) are versioned but user models have **no drift detection or staleness mechanism**. The experience-tree writes to KB with timestamps but has no automatic re-evaluation trigger when user behavior diverges from stored models. There's no concept of "anti-user" or "adversarial user" modeling — the system assumes all users are cooperative. **Defect: no persona drift detection, no staleness re-evaluation, no anti-user modeling.**

---

## Summary: 8 NEW Defects (vs Batch 588's 5 findings)

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| D1 | No hierarchical behavioral abstraction (Practice→Habitus→Field) | NT-CORE / SelfModel | High |
| D2 | No closed-loop persona refinement (prediction error → model update) | NT-MIND / SEAL | High |
| D3 | No uncertainty quantification on user preference models | NT-CORE / EmotionLabel | Medium |
| D4 | No cross-domain preference transfer | NT-MEMORY / KB | Medium |
| D5 | No interpretable negative-signal debiasing in attention fusion | NT-CORE / GWT | High |
| D6 | No over-personalization guard / personalization restraint | NT-SHIELD / Egress | High |
| D7 | No cold-start routing strategy (CF↔CB fallback) | NT-ACT / Orchestration | Medium |
| D8 | No persona drift detection or anti-user modeling | NT-MEMORY / Experience | Medium |

## Sources Cited

1. Wang et al. (2026-06). "Beyond Isolated Behaviors: Hierarchical User Modeling for LLM Personalization." arXiv:2606.02300
2. IRIS (2026-07). "Learning Dynamic User Personas from Implicit Interaction Streams via Iterative Refinement." arXiv:2607.26473
3. HypReflect (2026-08). "Hypotheses-Guided Self Distillation for Continual Personalization." arXiv:2609.00251
4. HORIZON (2026). ACL 2026 Findings. aclanthology.org/2026.findings-acl.1503
5. KGERA (2026-03). Nature Scientific Reports. doi:10.1038/s41598-026-42865-6
6. PersonaMem-v3 (2026-07). arXiv:2608.21381
7. NVECTA (2026-05). "Collaborative Filtering vs Content-Based vs Hybrid." nvecta.com
8. Koji (2026-05). "User Personas in 2026." koji.so
9. CleverX (2026-01). "How to Create User Personas (2026)." cleverx.com
10. Clayton Johnson (2026-07). "Target Audience Persona." claytonjohnson.com
11. Siift (2026-06). "What Is a User Persona? A 2026 Guide." siift.ai
12. YouGov (2026-03). "AI Personas." yougov.com
