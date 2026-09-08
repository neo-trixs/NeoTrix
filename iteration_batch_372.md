# Iteration Batch 372 — Sentiment / Intent / Text Classification Research Sweep

**Date**: 2026-09-06
**Research Scope**: Sentiment analysis, intent detection, text classification (2026 state-of-the-art)

---

## Sources Cited

| # | Paper / Resource | Year | Key Advance |
|---|-----------------|------|-------------|
| S1 | SemCap — Sentiment-aware semantic captioning for MABSA (Springer) | 2026 | Qwen3-VL caption as visual proxy; SPDR fusion for cross-modal sentiment |
| S2 | GloLoc — Contrastive Learning Enhanced Global-Local Fusion for MABSA | 2026 | BLIP/VILA global features + scene-graph local alignment; SentiAug |
| S3 | Explainable MABSA with Dependency-guided LLM (arXiv 2601.06848) | 2026 | MLLM generative framework; dependency-syntax pruning for aspect-aware reasoning |
| S4 | VADE — VAD-Enhanced MABSC (ACL Findings 2026) | 2026 | Continuous Valence-Arousal-Dominance replacing discrete polarity; Affect-Aware CLIP |
| S5 | BIPE — Image Perception Enhancement for MABSA | 2026 | Weakly-supervised segmentation; aspect-supplementary phrases; joint optimization |
| S6 | S3MIC — Slot-Aware Semantic Signaling for Multi-Intent Classification | 2026 | Slot logits concatenated with CLS for intent prediction; 52.5% SeFr Acc on MixATIS |
| S7 | Multi-turn Intent Classification with LLM-based Labeling (ACL 2026) | 2026 | Hybrid routing: fine-tuned RoBERTa (τ=0.7) → LLM escalation; simulated dialogue augmentation |
| S8 | LLMs Replace Fine-Tuned NLU? Decision Framework (arXiv 2608.20371) | 2026 | OOS recall 85.6 (LLM) vs 58.1 (RoBERTa); ASR noise robustness; schema-prompted LLM at ~94% |
| S9 | BTZSC — Zero-Shot Text Classification Benchmark (ICLR 2026) | 2026 | Rerankers dominate: Qwen3-Reranker-8B F1=0.72; embeddings best accuracy-latency trade-off |
| S10 | AMLDA — Prompt-Based Few-Shot Text Classification | 2026 | Bayesian Mutual Information verbalizer; multi-granularity label augmentation; difficulty-aware adversarial |
| S11 | LSR — Label Space Reduction for Transductive Zero-Shot Classification | 2026 | Iterative label space refinement via pseudo-labels; distillation for deployment |
| S12 | FRSR — Fast Retrieval and Slow Reasoning for MSA (ACL Findings 2026) | 2026 | Dual-process theory: Top-K cue retrieval → cross-modal reasoning; interpretable evidence selection |
| S13 | MGSI — Multi-Granularity Sentiment Integration for LLM-Based MSA | 2026 | Short/medium/long temporal encoding; polarity-aware enhancement; adaptive residual calibration |
| S14 | COTEA — Capsule-Optimized Tri-Modal Expert Alignment | 2026 | Variational capsule sparse expert for uncertainty modeling; Gromov-Wasserstein cross-modal alignment |
| S15 | Multimodal Sentiment Analysis: Emerging Innovations (Springer survey) | 2026 | PRISMA review of 58 studies; hybrid fusion dominates; cultural sensitivity gap |

---

## Defects Found (10 Total)

### D-372-1: EmotionLabel Enum Missing Continuous Affect Dimensions

**File**: `neotrix-core/src/unified/core/nt_core_self/emotion_state.rs:5-31`
**Research**: VADE (S4) demonstrates continuous VAD (Valence-Arousal-Dominance) modeling achieves SOTA over discrete polarity labels. MGSI (S13) and COTEA (S14) confirm that continuous affect captures near-neutral and ambiguous samples that discrete enums miss.
**Defect**: `EmotionLabel` is a discrete 11-variant enum (Neutral/Joy/Sadness/.../Thinking). The `EmotionState` computes valence/arousal/dominance as derived `f64` but these are never exposed as first-class continuous outputs — only the discrete `label()` is consumed downstream. The continuous VAD dimensions are lost at the `EmotionReport` boundary.
**Suggestion**: Add `vad: [f64; 3]` (Valence, Arousal, Dominance) as a continuous field on `EmotionReport`. Allow downstream consumers (attention routing, GWT broadcast) to consume continuous VAD directly instead of only the discrete label. This aligns with the 2026 finding that continuous affect outperforms discrete categories for MSA.

### D-372-2: No Aspect-Level Sentiment Granularity

**File**: `neotrix-core/src/unified/layers/emotion/nt_feel/nt_feel/emotion_engine.rs:180-209`
**Research**: MABSA (S1, S2, S3, S5) shows aspect-level sentiment — tracking sentiment toward specific aspects/targets within the same utterance — is critical for real-world social media and dialogue analysis.
**Defect**: `FeelEngine::detect_from_text()` produces a single global `EmotionLabel` per text. There is no mechanism to track sentiment toward multiple aspects or targets simultaneously (e.g., "the food was great but the service was terrible" → food=positive, service=negative).
**Suggestion**: Introduce an `AspectSentiment` struct: `{ aspect: String, sentiment: EmotionLabel, confidence: f64, vad: [f64; 3] }`. The `FeelEngine` should maintain a `Vec<AspectSentiment>` per utterance. This enables NeoTrix to handle real-world inputs where multiple conflicting sentiments coexist.

### D-372-3: Intent Engine Lacks Hybrid Confidence-Routed Architecture

**File**: `neotrix-core/src/unified/core/nt_core_aura/intent_engine.rs:29-60`
**Research**: The hybrid intent detection paradigm (S7, S8) shows fine-tuned models handle 80% of traffic at high confidence (τ=0.7), while ambiguous cases are escalated to LLMs for disambiguation. This outperforms either approach alone.
**Defect**: `IntentEngine::process_input()` always routes through the same VSA-based inference path regardless of confidence. When `gap_score > 0.5` (ambiguous), the engine stays in `Reasoning` phase but never escalates to an external disambiguation mechanism.
**Suggestion**: Add a `confidence_threshold: f64` config parameter (default 0.7). When `intent_confidence() < threshold`, trigger a secondary LLM-based disambiguation path using the top-3 candidate intents from the VSA engine. This matches the S7 finding that hybrid routing achieves better accuracy than pure fine-tuned or pure LLM approaches.

### D-372-4: No Out-of-Scope (OOS) Detection

**File**: `neotrix-core/src/unified/core/nt_core_aura/intent_engine.rs`
**Research**: LLM-based OOS detection achieves 85.6% recall vs 58.1% for fine-tuned RoBERTa (S8). OOS detection is critical for production conversational systems to gracefully handle inputs outside the trained intent space.
**Defect**: The `IntentEngine` has no mechanism to detect when an input falls outside the known intent space. All inputs are forced through intent resolution, even if they are gibberish or completely unrelated to any defined task.
**Suggestion**: Add an `Out-of-Scope Detector` that checks input similarity against the known intent embedding space. When the maximum cosine similarity to any known intent vector falls below a threshold, flag the input as OOS and route to a fallback/conversation mode instead of attempting forced resolution.

### D-372-5: No Dialogue Act Classification

**File**: `neotrix-core/src/unified/core/nt_core_aura/intent_engine.rs` (entire file)
**Research**: Multi-task learning across DAC + ID + SF (S6, S7) shows dialogue acts provide complementary information to intent detection. Request/assertion/question classification changes how the system should respond to the same intent.
**Defect**: `IntentEngine` conflates communicative intent (what the user wants done) with dialogue act (how they are communicating — request, assertion, question, clarification). These are orthogonal: "can you build a parser?" (intent=build, act=request) vs "I need a parser built" (intent=build, act=assertion) vs "should I build a parser?" (intent=question, act=clarification).
**Suggestion**: Add a `DialogueAct` enum (Request, Assertion, Question, Clarification, Greeting, Feedback) alongside intent resolution. Use the act to modulate response strategy (e.g., Question → provide options; Assertion → confirm and execute; Clarification → ask follow-up).

### D-372-6: No Slot Filling / Structured Extraction

**File**: `neotrix-core/src/unified/core/nt_core_aura/intent_frame.rs`
**Research**: Slot filling is tightly coupled with intent detection (S6, S7). S3MIC demonstrates slot-aware semantic signaling improves multi-intent classification by 1.2% on MixATIS.
**Defect**: `IntentFrame` captures intent as a label + confidence + VSA vector but does not extract structured entities (slots) from the input. For "build the parser in Rust by Friday", the system detects intent=build but does not extract {language: "Rust", deadline: "Friday"}.
**Suggestion**: Add a `slots: HashMap<String, String>` field to `IntentFrame`. During the scan/reasoning phase, extract entity slots using pattern matching or a lightweight NER step. Expose slots for downstream consumers (task planning, resource allocation).

### D-372-7: Text-Only Emotion Detection — No Multimodal Pathway

**File**: `neotrix-core/src/unified/layers/emotion/nt_feel/nt_feel/emotion_engine.rs:180-209`
**Research**: MSA reviews (S15) confirm that multimodal fusion (text + audio + vision) consistently outperforms text-only sentiment. FRSR (S12) and MGSI (S13) achieve SOTA through structured multimodal integration. SemCap (S1) and GloLoc (S2) show image+text fusion is essential for real-world social media.
**Defect**: `FeelEngine::detect_from_text()` only processes text strings. There is no pathway for audio features (tone, pitch, energy) or visual features (facial expression, scene sentiment) to influence emotion detection.
**Suggestion**: Define a `MultimodalFeelInput` struct with optional `text: Option<String>`, `audio_features: Option<AudioAffect>`, `visual_features: Option<VisualAffect>`. Add a `detect_multimodal()` method that fuses available modalities. Even if audio/visual features are initially stubbed, the interface should exist for future extension.

### D-372-8: No Zero-Shot / Few-Shot Text Classification Capability

**File**: No existing text classification module found in the codebase.
**Research**: BTZSC (S9) demonstrates that modern rerankers achieve F1=0.72 in zero-shot text classification. AMLDA (S10) shows few-shot classification with prompt-based approaches. LSR (S11) achieves 7% macro-F1 improvement through label space reduction.
**Defect**: NeoTrix has no text classification capability for categorizing incoming text (e.g., classifying user messages into topic categories, emotion categories, or task categories) without training examples. This limits the system's ability to route and prioritize incoming information.
**Suggestion**: Add a `TextClassifier` module in NT-WORLD or NT-IO with: (1) a zero-shot mode using reranker-based classification (Qwen3-Reranker pattern), (2) a few-shot mode using prompt-based verbalizer construction (AMLDA pattern), (3) an LSR-based label space reduction for large classification schemas. Expose as a capability in the capability registry.

### D-372-9: No Cross-Modal Alignment Mechanism for Sentiment

**File**: `neotrix-core/src/unified/layers/emotion/nt_feel/nt_feel/emotion_engine.rs`
**Research**: COTEA (S14) shows Gromov-Wasserstein distance + Fréchet regularization achieves multi-level cross-modal alignment. MGSI (S13) uses text-guided refinement to align non-text features with textual sentiment. SemCap (S1) uses Semantic Proxy Dynamic Routing for patch-level alignment.
**Defect**: If multimodal inputs are added (D-372-7), there is no alignment mechanism to ensure visual/audio features correspond to the correct text segments. The current design would naively concatenate features at the utterance level, losing fine-grained cross-modal correspondences.
**Suggestion**: Define a `CrossModalAligner` trait with an `align(text_features, audio_features, visual_features) -> AlignedMultimodalRepresentation` method. Implement with attention-based alignment (matching the SemCap SPDR pattern) where text tokens attend to relevant audio/visual patches.

### D-372-10: No Explainability / Rationale Generation for Sentiment Decisions

**File**: `neotrix-core/src/unified/layers/emotion/nt_feel/nt_feel/emotion_engine.rs:294-308`
**Research**: FRSR (S12) provides interpretable evidence through Top-K cue selection. Explainable MABSA (S3) generates natural language explanations for sentiment decisions. The PRISMA survey (S15) identifies interpretability as a critical gap.
**Defect**: `FeelEngine::attention_signal()` returns a salience score but provides no rationale for why that score was computed. When the system decides "this input requires urgent attention," there is no traceable evidence path from input features → sentiment detection → arousal computation → salience output.
**Suggestion**: Add an `Explanation` struct: `{ evidence_cues: Vec<String>, contributing_features: Vec<(String, f64)>, reasoning_path: String }`. The attention signal should include this explanation. For multimodal inputs, follow the FRSR pattern: identify the Top-K most salient cues (text phrases, audio segments, visual regions) that drove the sentiment decision.

---

## Summary

| Category | Defects | Severity |
|----------|---------|----------|
| Sentiment Architecture | D-372-1 (VAD continuity), D-372-2 (aspect-level), D-372-7 (multimodal), D-372-9 (alignment), D-372-10 (explainability) | High |
| Intent Architecture | D-372-3 (hybrid routing), D-372-4 (OOS detection), D-372-5 (dialogue acts), D-372-6 (slot filling) | High |
| Text Classification | D-372-8 (zero/few-shot capability) | Medium |

**Priority Order**: D-372-1 (VAD) → D-372-3 (hybrid intent) → D-372-5 (dialogue acts) → D-372-6 (slots) → D-372-2 (aspect-level) → D-372-4 (OOS) → D-372-8 (text classification) → D-372-7 (multimodal) → D-372-9 (alignment) → D-372-10 (explainability)

**Rationale**: VAD continuity (D-372-1) is foundational — all other sentiment advances build on continuous affect representation. Hybrid intent routing (D-372-3) directly impacts production reliability. Dialogue acts + slots (D-372-5/6) are low-effort structural additions. The remaining defects require more architectural work but are validated by 2026 SOTA findings.
