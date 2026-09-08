# Iteration Batch 470 — Text Classification, Sentiment Analysis, Opinion Mining

**Date:** 2026-09-06
**Focus:** NLU pipeline gaps — classification, sentiment, opinion/stance/argument

---

## Sources Cited

| # | Source | Year | Topic |
|---|--------|------|-------|
| S1 | Sharma & Sharma, "Zero-Shot and Few-Shot Scientific Text Classification Using Modern LLMs" (IJERT, May 2026) | 2026 | Zero-shot LLM classification benchmarks (LLaMA 3.1-8B achieves 99% on WoS-5736) |
| S2 | Aarab, "BTZSC: A Benchmark for Zero-Shot Text Classification Across Cross-Encoders, Embedding Models, Rerankers and LLMs" (arXiv 2603.11991, Mar 2026) | 2026 | Systematic ZSC benchmark comparing encoder vs decoder approaches |
| S3 | ScienceDirect, "Zero-Shot Text Classification: Harnessing Pretrained Language Models" (2026) | 2026 | NLI-based zero-shot pipeline with verbalizer templates + entailment scores |
| S4 | DimABSA / SemEval-2026 Task 3: Dimensional Aspect-Based Sentiment Analysis (ACL 2026) | 2026 | Valence-Arousal dimensional ABSA replacing categorical polarity labels; multilingual + multidomain |
| S5 | Springer, "Multimodal sentiment analysis: emerging innovations, core challenges, and future directions" (Discover AI, Mar 2026) | 2026 | Cross-modal fusion for text+image+audio+video sentiment |
| S6 | Emergent Mind topic tracker, "Multimodal ABSA (MABSA)" (updated Jan 2026) | 2026 | LLM-generated rationales, dependency-guided MABSA, prompt-controlled extraction |
| S7 | YouScan, "Aspect-Based Sentiment Analysis: The Complete Guide" (Mar 2026) | 2026 | Sarcasm detection, implicit aspect identification, domain dependency challenges |
| S8 | ACL 2026, "Modeling Human-Like Cognition for Stance Detection: CDSD" (2026) | 2026 | Dual-process (intuitive + analytical) stance detection; robustness to textual perturbations |
| S9 | ArgMining 2026 Workshop, ACL (Jul 2026) | 2026 | Argument quality assessment, stance classification, multimodal argument mining |
| S10 | Ameen et al., "TruthStance: Conversations on Truth Social" (arXiv 2602.14406, Feb 2026) | 2026 | Large-scale stance detection dataset (24K root posts, 523K comments, reply-tree structure) |

---

## Defects Identified

### DEFECT 1 — No Zero-Shot / Few-Shot Text Classification Pipeline

**Current state:** NeoTrix's `sensory_processing.rs:158-193` uses hardcoded keyword matching (`contains("what")`, `contains("fix")`, etc.) for dialogue phase classification. No NLI-based zero-shot classifier or few-shot example selection exists.

**Research gap:** S1 shows LLaMA 3.1-8B achieves 99% zero-shot accuracy on text classification. S2 benchmarks cross-encoders, embedding models, and LLMs for ZSC. S3 describes NLI-based zero-shot pipeline with verbalizer templates — a lightweight, no-fine-tuning approach.

**Concrete defect:** `GodViewReport` phase detection (`sensory_processing.rs:158-193`) is brittle keyword matching that fails on paraphrases, cross-lingual input, and domain-specific queries. No mechanism exists for:
- NLI-based zero-shot classification (entailment scoring against label descriptions)
- Few-shot example selection strategies (KNN, k-medoids, DPP from S2)
- Dynamic label set adaptation (new dialogue phases without code changes)

**Suggestion:** Add a `ZeroShotClassifier` module under `nt_world_sense` that:
1. Takes label descriptions (verbalizer templates) + input text
2. Computes entailment scores via NLI
3. Returns label + confidence
4. Integrates into `analyze_dialogue_arc()` replacing keyword matching

---

### DEFECT 2 — No Aspect-Based Sentiment Analysis (ABSA)

**Current state:** `EmotionLabel` has 11 discrete variants. `FeelEngine::detect_from_text()` (emotion_engine.rs:180-208) does keyword-matching for whole-text emotion. `sentiment_trend` in `DialogueArcAnalysis` is a linear regression slope over scalar satisfaction scores — it captures no per-aspect sentiment.

**Research gap:** SemEval-2026 Task 3 (S4) introduces dimensional ABSA with valence-arousal axes, replacing categorical positive/negative/neutral. MABSA (S6) adds cross-modal aspect extraction from text+image pairs.

**Concrete defect:** NeoTrix cannot answer: "What is the sentiment toward *aspect X* in this text?" For example, in "The API is fast but the documentation is terrible", the current system produces a single blended emotion/sentiment score. No aspect extraction, no per-aspect polarity, no valence-arousal dimensions.

**Suggestion:** Add an `AspectSentimentExtractor` that:
1. Extracts aspect terms from text (noun phrases / user-specified targets)
2. Assigns valence-arousal scores per aspect (not categorical labels)
3. Stores results in KB `aspect_sentiment` namespace for trend analysis
4. Integrates with `OmniscientView` to enrich `sentiment_trend` with per-aspect breakdown

---

### DEFECT 3 — No Multimodal Sentiment Fusion

**Current state:** `FeelEngine::detect_from_text()` is text-only. Cross-modal VSA exists (`CrossModalAligner` in `nt_core_hcube/cross_modal.rs`) but is for concept alignment, not sentiment fusion. `AudioSyncLibrary` and `VideoPostProcessor` exist but handle synchronization/quality, not emotional signal extraction.

**Research gap:** S5 (Springer 2026) identifies multimodal sentiment as the frontier — combining voice prosody, facial expression, body language, and text for unified emotion assessment. S6 shows MABSA models achieving strong gains on Twitter-15/17 benchmarks using context-aware attention fusion.

**Concrete defect:** For NT-PHYSICAL (embodiment) and NT-WORLD (perception), there is no pipeline to:
- Extract sentiment signal from audio (prosody, pitch, energy)
- Extract sentiment signal from visual (facial expression, gesture)
- Fuse text + audio + visual sentiment into unified assessment
- Feed fused result into `FeelEngine` or `EmotionEngine`

**Suggestion:** Add `MultimodalSentimentFusion` module under `nt_world_sense`:
1. Modality-specific sentiment extractors (text → existing FeelEngine, audio → prosody features, visual → expression classifier)
2. Cross-modal attention fusion (context-aware, as in S6)
3. Output: `FusedSentiment { text_va, audio_va, visual_va, fused_va, confidence }`
4. Route via PerceptionBridge to GWT for attention modulation

---

### DEFECT 4 — No Dimensional (Valence-Arousal) Emotion Representation

**Current state:** `EmotionLabel` is categorical (11 discrete labels). `FeelEngine` uses PAD (Pleasure-Arousal-Dominance) internally but exposes only categorical `current_label()`. No continuous valence-arousal mapping.

**Research gap:** SemEval-2026 DimABSA (S4) shows that valence-arousal dimensional modeling outperforms categorical sentiment for multilingual, multidomain scenarios. Russell's circumplex model (1980) is the theoretical foundation, now validated at scale in 2026 benchmarks.

**Concrete defect:** The internal PAD representation is computed but discarded when returning `EmotionLabel`. This means:
- GWT attention routing cannot use continuous arousal values for salience
- Cross-modal sentiment fusion has no continuous basis for alignment
- Temporal emotion trajectory analysis is limited to label transitions, not smooth curves
- Multilingual emotion detection fails because categorical labels are language-specific

**Suggestion:** Expose `EmotionReport { valence: f64, arousal: f64, dominance: f64 }` as first-class alongside `EmotionLabel`. Add `EmotionLabel::from_valence_arousal(v, a)` mapping function. Enable GWT to use continuous arousal for attention salience computation.

---

### DEFECT 5 — No Stance Detection Module

**Current state:** NeoTrix has no stance detection capability. `EmotionLabel` captures *how the system feels* but not *what position the system/user holds toward a target*. No target-aware sentiment exists.

**Research gap:** CDSD (S8, ACL 2026) demonstrates that dual-process (intuitive judgment + analytical reasoning) stance detection outperforms single-pass models and is robust to textual perturbations. TruthStance (S10) provides large-scale target-stance data for alt-platforms.

**Concrete defect:** NT-WORLD cannot answer: "What is the user's stance toward topic X?" For example:
- "Climate change requires immediate action" → stance: **support** (toward: climate_action)
- "AI regulation stifles innovation" → stance: **oppose** (toward: ai_regulation)
- These are NOT sentiment (emotion) — they are *target-directed attitudes*

Missing capabilities:
- Target-stance pair extraction from dialogue
- Stance classification with cognitive dual-process modeling
- Stance tracking across conversation turns (stance drift detection)
- Integration with ConsciousnessTree for cross-session stance memory

**Suggestion:** Add `StanceDetector` module under `nt_world_sense`:
1. Input: (text, target) → output: stance_label + confidence + evidence
2. Dual-process architecture: fast intuitive + slow analytical (per S8)
3. Store stance history in KB for cross-session stance memory (NT-NEXUS integration)
4. Feed into GWT as a new specialist module for "social cognition" attention routing

---

### DEFECT 6 — No Argument Mining Capability

**Current state:** No argument structure extraction, no premise-conclusion identification, no argument quality assessment, no persuasion analysis.

**Research gap:** ArgMining 2026 (S9) shows the field maturing with multimodal argument mining, explainability through reasoning, and LLM-based argument quality assessment. The UZH Shared Task on UN resolutions demonstrates structured argument mining at scale.

**Concrete defect:** NT-WORLD cannot parse argumentative structure from user input or external content. This limits:
- Understanding *why* a user holds a position (not just *that* they hold it)
- Evaluating the quality/logical soundness of arguments in crawled content
- Generating counter-arguments or strengthening user positions
- Misinformation detection (identifying weak/unsupported claims)

**Suggestion:** Add `ArgumentMiner` module under `nt_world_sense`:
1. Extract argument structure: claims, premises, inference schemes
2. Assess argument quality (strength, relevance, sufficiency)
3. Store argument graphs in KB for cross-session reasoning
4. Feed into E8 reasoning engine for dialectical argumentation support

---

### DEFECT 7 — Keyword-Based Emotion Detection Cannot Handle Sarcasm/Negation

**Current state:** `FeelEngine::detect_from_text()` (emotion_engine.rs:180-208) checks `contains("happy")`, `contains("angry")` etc. This fails on:
- Sarcasm: "Oh great, another bug" (positive words, negative sentiment)
- Negation: "I am not happy" (positive word, negative meaning)
- Implicit sentiment: "The battery lasts 5 minutes" (no sentiment words, negative meaning)

**Research gap:** S7 identifies sarcasm detection, implicit aspect identification, and negation handling as key unsolved challenges in ABSA. S8 shows dual-process models handle these via analytical reasoning on top of intuitive first-pass.

**Concrete defect:** The `detect_from_text` function produces incorrect EmotionLabel for sarcastic/negated/implicit text. This corrupts the entire downstream emotion pipeline (FeelEngine → EmbodiedEmotion → ConsciousnessTree → GWT).

**Suggestion:** Add a negation/sarcasm pre-processing layer:
1. Negation scope detection (input-level modifier: "not", "never", "no")
2. Sarcasm signal detection (incongruity between sentiment words and context)
3. Apply before keyword-based detection to flip/invert signals
4. Long-term: replace keyword detection with NLI-based zero-shot (per DEFECT 1)

---

## Summary

| # | Defect | Severity | Domain Impact |
|---|--------|----------|---------------|
| D1 | No zero-shot/few-shot text classifier | HIGH | NT-WORLD (perception), NT-CORE (GWT routing) |
| D2 | No aspect-based sentiment analysis | HIGH | NT-FEEL (emotion), NT-MEMORY (KB trends) |
| D3 | No multimodal sentiment fusion | MEDIUM | NT-PHYSICAL (embodiment), NT-WORLD (perception) |
| D4 | No dimensional VA emotion representation | HIGH | NT-CORE (GWT salience), NT-FEEL (emotion) |
| D5 | No stance detection | HIGH | NT-WORLD (perception), NT-MEMORY (cross-session) |
| D6 | No argument mining | MEDIUM | NT-CORE (E8 reasoning), NT-WORLD (perception) |
| D7 | Keyword emotion detection fails on sarcasm/negation | HIGH | NT-FEEL (emotion), all downstream consumers |

**Recommended priority order:** D7 → D1 → D4 → D2 → D5 → D3 → D6
(Rationale: D7 is foundational — all higher-level modules depend on correct emotion detection; D1/D4 enable the others)
