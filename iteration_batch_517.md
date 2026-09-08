# Iteration Batch 517 — External Research Loop

**Date**: 2026-09-06
**Domains**: Sentiment Analysis | Dialogue Systems | Text Classification
**Method**: arXiv live crawl → defect mapping → design gap extraction

---

## 1. Sentiment Analysis (2026 Advances)

### Sources
- [C³T: Counterfactual Causal Reasoning for Sentiment Shifts in Social-Media Conversation Trees](https://arxiv.org/abs/2609.02131) — EMNLP 2026
- [MAESTRO: Multimodal Adaptive Expert Selection with Text Routing and Ordinal Prototype Optimization](https://arxiv.org/abs/2608.30726) — Aug 2026
- [Cross-lingual ABSA with SeqLab and Aspect-Code Switching](https://arxiv.org/abs/2608.30425) — EMNLP 2026
- [CREMA-ASIS: Acoustic-Semantic Incongruity for LALMs](https://arxiv.org/abs/2608.28966) — EMNLP 2026 Findings
- [SpeechSense: Paralinguistic-Focused Fine-Grained Speech Sentiment](https://arxiv.org/abs/2608.17931) — ACM Multimedia 2026
- [Wazobia Eval: Nigerian Pidgin Emotion, Sarcasm, Cultural Reasoning](https://arxiv.org/abs/2608.21369) — Jun 2026
- [STAR-OPD: Structured Aspect-Cascade-Aware On-Policy Reward Distillation for ABSA](https://arxiv.org/abs/2608.20831) — Aug 2026
- [Human-Centered Perspective-Aware Framework for Reproducible ML Evaluation](https://arxiv.org/abs/2608.30842) — ACL SRW 2026
- [Data Science Pipeline for Person-Level Sentiment (MINOS)](https://arxiv.org/abs/2608.26135) — SGAI 2025/2026

### Key Findings

| Finding | Defect in NeoTrix | Severity |
|---------|-------------------|----------|
| **Causal sentiment shifts in conversation trees** (C³T): Sentiment in social threads shifts via discourse moves (denial, evidence, toxicity). Parent-child shift labels + counterfactual reasoning. | NeoTrix `sentiment_trend` is a single `f64` — no shift-type attribution, no parent-child differential, no causal intervention modeling. `emotion_state.rs:16-31` EmotionLabel has 11 flat variants with no causal provenance. | **High** |
| **Ordinal sentiment intensity** (MAESTRO): Sentiment has natural ordinal hierarchy (intensity grades). O-PCL enforces structured latent space preserving emotion order. | NeoTrix `EmotionLabel` is unweighted enum — no intensity/ordinal field. `EmotionDimension` (Frustration/Confidence/Joy/Urgency/Curiosity/Fatigue) is orthogonal but has no intensity ordering semantics. | **High** |
| **Acoustic-semantic incongruence** (CREMA-ASIS): LALMs fail on sarcasm/irony when acoustic and semantic cues diverge. Semantic dominance bias. | NeoTrix `detect_from_text` (`emotion_engine.rs:180`) is text-only. `nt_sense` `sentiment_trend` has no multimodal incongruence detection. No sarcasm/irony flag. | **Medium** |
| **Paralinguistic stance taxonomy** (SpeechSense): 8-class interpersonal stance (confident, impatient, etc.) detectable primarily through prosody. | NeoTrix has no paralinguistic stance detection. `EmotionLabel::Confused` and `::Thinking` are "expression-only" annotations, not grounded in acoustic analysis. | **Medium** |
| **Cultural emotion taxonomies** (Wazobia Eval): 16-category emotion taxonomy for Nigerian Pidgin, culturally specific registers not in conventional frameworks. | NeoTrix EmotionLabel is Western-centric (Plutchik's wheel). No culture/persona-adaptive emotion taxonomy. `nt_feel_vtuber.rs:71` has a separate `EmotionType` enum — duplicate, not unified. | **Medium** |
| **Disagreement-aware evaluation** (Pandita & Homan): Aggregating labels via plurality obscures minority perspectives. Human disagreement is signal, not noise. | NeoTrix `EmotionReport` (`emotion_state.rs:264`) uses single `emotion_label: EmotionLabel` — no distribution over labels, no disagreement/confidence field. | **High** |
| **Cross-lingual ABSA with sequence labeling**: Fine-grained aspect-target-sentiment extraction across 11 languages. | NeoTrix has no cross-lingual sentiment capability. `sentiment_trend` operates on single-language text. | **Low** |
| **Person-level sentiment from OSINT** (MINOS): Domain-informed algorithm for reputational risk detection beyond generic sentiment. | NeoTrix has no domain-specific sentiment adapters (e.g., risk/reputation). `nt_core_sense::sensory_processing` computes generic `sentiment_trend:0.0`. | **Low** |

---

## 2. Dialogue Systems (2026 Advances)

### Sources
- [Multi-turn Conversational AI Survey (Ahmed et al.)](https://arxiv.org/abs/2608.17605) — Aug 2026
- [Full-Duplex Dialogue with Proactive Turn-Taking (LPS-TC)](https://arxiv.org/abs/2608.28630) — ACM MM 2026
- [MemUse: Memory Evaluation for Long-Term Human-AI Conversation](https://arxiv.org/abs/2608.24189) — EMNLP 2026
- [Grounded Dialogue as Selective Belief Revision](https://arxiv.org/abs/2608.26035) — Aug 2026
- [DeepSAGE: Stage-Aware RL for CBT Counseling Dialogue](https://arxiv.org/abs/2608.22615) — Aug 2026
- [HealthCUES: Respiratory Signal Understanding for Conversational Healthcare](https://arxiv.org/abs/2608.26163) — SIGDIAL 2026
- [Agentic-DuplexGen: Decoupling Content, Timing, and Acoustics](https://arxiv.org/abs/2608.16053) — Aug 2026
- [CompanionSim: Anthropomorphism in Human-AI Relationships](https://arxiv.org/abs/2609.00250) — AIES 2026
- [TEIDAN: Multilingual Multiparty Dialogue Corpus](https://arxiv.org/abs/2609.00802) — ICMI 2026
- [Beyond Local Surprise: Preserve/Revise Decisions in Dialogue](https://arxiv.org/abs/2608.26035) — Aug 2026

### Key Findings

| Finding | Defect in NeoTrix | Severity |
|---------|-------------------|----------|
| **Natural Integration > Direct QA** (MemUse): 71-point gap between Direct QA accuracy and natural memory integration in conversation. Users satisfied when memory is woven in, not when recalled. | NeoTrix `nt_nexus` (cross-session memory) has no distinction between elicited retrieval vs. natural integration. `DialogueAbsorbBridge` absorbs experiences but no mechanism to weave prior context into responses naturally. | **High** |
| **Full-duplex proactive turn-taking** (LPS-TC): Real agents must interrupt, backchannel, and act proactively. Fine-grained action space covering reactive + proactive behaviors. | NeoTrix `DialogueAbsorbBridge` (in `agent_capability/mod.rs:1287`) routes dialogue as passive experience ingestion. No proactive turn-taking, no backchanneling, no interruption modeling. | **High** |
| **Stage-aware structured dialogue** (DeepSAGE): CBT sessions have 11 explicit stages with therapeutic objectives. DRL selects intentions that guide generation. | NeoTrix dialogue is flat — no stage/phase-aware routing. `dialogue_arc` in `sensory_types.rs:114` tracks phases but has no structured stage management or goal-directed progression. | **Medium** |
| **Preserve/Revise belief revision** (Liu et al.): Listeners preserve understanding despite local mismatch; revise only when uncertainty accumulates. | NeoTrix has no belief revision mechanism for dialogue grounding. `omniscient_view.rs` computes `dialogue_phases` but has no preserve/revise decision logic. | **Medium** |
| **Paralinguistic respiratory monitoring** (HealthCUES): Cough/throat-clearing detection during live conversation, subtype classification. | NeoTrix has zero paralinguistic signal processing. `nt_physical::audio_sync_library` handles sync patterns but no health/paralinguistic monitoring. | **Low** |
| **Companionship behaviors reduce trust** (CompanionSim): Validation/empathy in chatbots can reduce likability and trust, especially for women and older users. | NeoTrix `EmotionEngine` social state tracking (`emotion_engine.rs:69`) has conversational_turns counter but no companionship-behavior evaluation or trust-modulation framework. | **Medium** |
| **Content-Timing-Acoustics decoupling** (Agentic-DuplexGen): Dialogue synthesis should decouple content, timing, and acoustics rather than generating sequentially. | NeoTrix `storyboard_extractor.rs` and `narrative_structuring.rs` treat dialogue as text strings (`dialogue: Option<String>`) with no acoustic/timing decoupled pipeline. | **Medium** |

---

## 3. Text Classification (2026 Advances)

### Sources
- [SonicCaps: CLAP Models with Zero-Shot Classification from Audio Captioning](https://arxiv.org/abs/2609.02343) — Sep 2026
- [TabPFN as Calibration-Sensitive Head for Multimodal Embeddings](https://arxiv.org/abs/2607.11007) — Jul 2026
- [Activation Steering for Low-Resource Synthetic Data](https://arxiv.org/abs/2606.18389) — Jun 2026
- [Supervised Classification Heads as Semantic Prototypes (Weight Recycling)](https://arxiv.org/abs/2605.22484) — May 2026
- [GP-Adapter: Few-Shot OOD Detection with Gaussian Process](https://arxiv.org/abs/2606.07102) — IJCNN 2026
- [Head Ensemble Classifiers (HEC) for LVLM Few-Shot](https://arxiv.org/abs/2603.24181) — Mar 2026
- [Multi-Legal-Bench: Cross-Jurisdictional Few-Shot Evaluation](https://arxiv.org/abs/2605.29738) — Aug 2026
- [Gini Index for Prompt-Based Classification Debiasing](https://arxiv.org/abs/2603.15654) — Mar 2026
- [LabelFusion-TS: Multi-Modal Financial Text Classification](https://arxiv.org/abs/2608.11753) — Aug 2026
- [LLMs as Annotators: Beyond Aggregated Metrics](https://arxiv.org/abs/2605.13412) — ACL LAW XX 2026

### Key Findings

| Finding | Defect in NeoTrix | Severity |
|---------|-------------------|----------|
| **Calibration-aware classification heads** (TabPFN): Frozen encoder + lightweight head produces poorly calibrated confidence. TabPFN reduces NLL by 48-62% and ECE by 2-5x. | NeoTrix `EmotionEngine.detect_from_text` returns `EmotionLabel` with no confidence score, no calibration mechanism. No ECE/NLL tracking. | **High** |
| **OOD detection with uncertainty** (GP-Adapter): CLIP yields deterministic similarity — no uncertainty info critical under distribution shift. GP-based variance-aware confidence. | NeoTrix text classification has no OOD/out-of-distribution detection. No uncertainty quantification on `sentiment_trend` or emotion predictions. | **High** |
| **Activation steering for low-resource languages** (Cegin et al.): Steering early layers improves diversity of synthetic data. Language Steering + Quality Steering. | NeoTrix has no activation steering for multilingual synthetic data generation. `control_distillation.rs:286` has "few-shot LLM" template but no steering mechanism. | **Medium** |
| **Weight recycling for zero-shot alignment** (Méndez et al.): Repurposing classification head weights as semantic prototypes enables zero-shot without end-to-end training. | NeoTrix has no weight-recycling or prototype-based alignment mechanism. Emotion classification relies on rule-based `EmotionDimension` → `EmotionLabel` mapping. | **Medium** |
| **Class accuracy imbalance in prompt-based classification** (Lin): Gini index reveals long-tailed minority classes have worst accuracy. Post-hoc debiasing needed. | NeoTrix `EmotionLabel` has no accuracy tracking per label. No bias detection across emotion classes. `Confused`/`Thinking` are expression-only, not accuracy-audited. | **Medium** |
| **LLM annotation imperfection** (Humblot-Renaux et al.): LLM annotators are inconsistent — need multi-model ensemble, not single-model labels. | NeoTrix `detect_from_text` is single-path deterministic. No ensemble, no inter-model disagreement tracking, no annotation-quality gate. | **Medium** |
| **Few-shot gains track headroom, not language** (Multi-Legal-Bench): 8/28 model-jurisdiction pairs *lost* accuracy with few-shot. Label-set alignment predicts transfer quality better than language family. | NeoTrix has no few-shot learning pipeline for text classification. `control_distillation.rs` few-shot templates are LLM-only, no closed-set classifier adaptation. | **Low** |

---

## 4. Consolidated Defects and Suggestions

### Critical Defects (High Severity)

| # | Defect | Root Cause | Suggestion |
|---|--------|------------|------------|
| D1 | **Flat sentiment representation** — `sentiment_trend: f64` loses all causal, ordinal, and distributional information | `sensory_types.rs:115` | Replace with `SentimentReport { shift_type, ordinal_intensity, distribution: HashMap<EmotionLabel, f64>, causal_source, confidence }` |
| D2 | **No sentiment shift attribution** — no parent-child differential, no counterfactual reasoning for conversation threads | `emotion_state.rs:16-31` | Add `SentimentShift { from, to, trigger: DiscourseMove, weight }` to `EmotionReport`. Integrate C³T-style causal attribution. |
| D3 | **No disagreement/confidence on emotion predictions** — single label with no distribution | `emotion_state.rs:264` | Add `confidence: f64` and `label_distribution: Vec<(EmotionLabel, f64)>` to `EmotionReport`. |
| D4 | **No OOD detection on text classification** — no uncertainty quantification | `emotion_engine.rs:180` | Add GP-based or ensemble-based uncertainty estimate to `detect_from_text`. Return `Result<EmotionLabel, UncertaintyEstimate>`. |
| D5 | **Memory not naturally integrated** — Direct QA style, not conversational weaving | `agent_capability/mod.rs:1287` | Add `NaturalIntegrationScorer` that evaluates how smoothly prior context is woven into current response, not just retrieval recall. |
| D6 | **No full-duplex dialogue capability** — passive experience ingestion only | `DialogueAbsorbBridge` | Add proactive turn-taking, backchanneling, and interruption modeling to dialogue pipeline. |

### Medium Defects

| # | Defect | Suggestion |
|---|--------|------------|
| D7 | No acoustic-semantic incongruence detection (sarcasm/irony) | Add multimodal `IncongruenceDetector` to `nt_sense` with acoustic-semantic divergence scoring. |
| D8 | No paralinguistic stance detection (confident, impatient, etc.) | Extend `EmotionLabel` or add `StanceLabel` enum with 8+ interpersonal stances. |
| D9 | Western-centric emotion taxonomy (11 Plutchik variants) | Add culture/persona-adaptive taxonomy slots. Allow `EmotionLabel` to be extended via KB-stored persona profiles. |
| D10 | Duplicate EmotionType in `nt_feel_vtuber.rs` vs unified `EmotionLabel` | Merge `EmotionType` into `EmotionLabel` or add conversion bridge. |
| D11 | No stage-aware dialogue progression | Add structured stage management to `dialogue_arc` with goal-directed routing. |
| D12 | No preserve/revise belief revision in dialogue grounding | Add `BeliefRevisionEngine` that accumulates uncertainty before revising grounded understanding. |
| D13 | No companionship-behavior trust modulation | Add trust-modulation framework to `EmotionEngine` social state tracking. |
| D14 | No activation steering for multilingual synthetic data | Integrate steering mechanism into `control_distillation.rs` few-shot templates. |
| D15 | No accuracy tracking per emotion label class | Add per-class accuracy monitoring with Gini-index-style bias detection. |

### Low Defects

| # | Defect | Suggestion |
|---|--------|------------|
| D16 | No cross-lingual sentiment capability | Add language-aware sentiment adapter. |
| D17 | No domain-specific sentiment (risk/reputation) | Add domain-adaptive sentiment classifiers (MINOS-style). |
| D18 | No paralinguistic respiratory monitoring | Add `HealthCUES`-style paralinguistic monitoring to `nt_physical`. |
| D19 | No few-shot closed-set classifier adaptation | Add few-shot learning pipeline for emotion/sentiment classification. |

---

## 5. Priority Matrix

```
                    Impact
              High  │  Medium  │  Low
         ┌──────────┼──────────┼──────────┐
  Effort │          │          │          │
  Low    │  D3,D4   │  D8,D10  │  D19     │
         ├──────────┼──────────┼──────────┤
  Med    │  D1,D2   │  D7,D11  │  D16     │
         │  D5,D6   │  D12,D15 │  D17     │
         ├──────────┼──────────┼──────────┤
  High   │          │  D9,D13  │  D18     │
         │          │  D14     │          │
         └──────────┴──────────┴──────────┘
```

**Next iteration focus**: D1 (SentimentReport refactor) + D3 (confidence distributions) + D4 (OOD detection) — these are high-impact, medium-effort and unblock downstream improvements.

---

*Generated by iteration loop 517. Sources: 9 sentiment papers, 10 dialogue papers, 10 classification papers from arXiv Sep 2026 crawl.*
