# Iteration Batch 385 — Speech AI 2026 Research + Design Defect Analysis

**Date**: 2026-09-06
**Domains**: Speech Recognition, Speaker Verification, Speech Enhancement

---

## Part 1: Research Sources

### 1A. Speech Recognition (ASR) — 2026 Advances

| # | Source | Key Finding |
|---|--------|-------------|
| S1 | ForaSoft (May 2026) | Streaming ASR in 2026 is dominated by 3 options: Whisper (open-source, needs streaming wrapper), Deepgram Nova-3 (~6.8% streaming WER, <300ms latency, Flux end-of-turn detection), AssemblyAI Universal-3 Pro (~6.3% mean WER, immutable transcripts, ~300ms). Voice agents live or die on **end-of-turn detection** — built into hosted APIs, missing from open-source Whisper loops. |
| S2 | CodeSOTA (Mar 2026) | Lowest WER: IBM Granite Speech 4.1 2B at 5.33% mean WER across 8 datasets. Whisper large-v3 still competitive at 2.x% on clean English. **Uni-ASR** (arXiv:2603.11123) proposes unified LLM-based architecture for both streaming and non-streaming ASR, eliminating the batch/streaming bifurcation. |
| S3 | HuggingFace ASR Leaderboard (2026) | New entrants: Qwen3-ASR-1.7B (4.45M downloads), NVIDIA Parakeet-TDT 0.6B (streaming-native), Cohere Transcribe-03-2026 (489K downloads). Mistral Voxtral-Mini-4B-Realtime: 4B-param real-time ASR model. |
| S4 | Whisper-Streaming Topic (EmergentMind, May 2026) | WhisperPipe (2026), CarelessWhisper (2025), Simul-Whisper: multiple approaches to convert Whisper's offline encoder-decoder into streaming. Key techniques: block-diagonal causal attention, finite look-ahead, rolling buffers. LocalAgreement-2 policy achieves ~3.3s average latency on English ESIC. |
| S5 | Skrew Voice AI Stack (Apr 2026) | Modern ASR incorporates **speaker diarization, emotion detection, and paralinguistic analysis** (tone, pacing, emphasis) — not just transcription. These feed richer context into downstream processing. |
| S6 | Uni-ASR (arXiv:2603.11123, Mar 2026) | Unified LLM-based framework integrating streaming and non-streaming ASR. Addresses the fundamental architectural split between real-time and batch recognition that plagues 2026 deployments. |

### 1B. Speaker Verification & Anti-Spoofing — 2026 Advances

| # | Source | Key Finding |
|---|--------|-------------|
| S7 | Orbit/Devotel (Jul 2026) | Voice biometrics (ECAPA-TDNN) and anti-spoofing (AASIST) are **separate systems solving different questions**. Speaker verification alone is "demonstrably vulnerable" to AI voice cloning. Both must run together. Two integration strategies: early/late integration in a unified model. |
| S8 | Fora Soft (Jul 2026) | Anti-spoofing check sits between match and decision in the verification pipeline. Voice is a "username, not a password." ECAPA-TDNN is the dominant speaker-embedding architecture. Self-hosted voice biometrics costs mostly fixed (compute), vs SMS OTP scaling linearly. |
| S9 | WildSpoof 2026 Challenge (Jan 2026, arXiv:2601.17557) | **Spoofing-Aware Speaker Verification (SASV)**: cascaded framework integrating Wavelet Prompt-Tuned XLSR-AASIST countermeasure with multi-model ensemble. SASV-EER drops below 1% in top systems. WPT-XLSR-AASIST: 0.16% EER on in-domain spoof detection, but cross-domain generalization gap remains (a-DCF degrades to 0.32–0.41 on out-of-domain). |
| S10 | VoxENES 2026 (arXiv:2607.11706) | Benchmarking spoofing detectors against **LLM-era TTS and voice conversion**. Legacy spoofing benchmarks overestimate detector robustness. LLM-driven TTS creates artifacts distinct from earlier TTS systems. |
| S11 | SASV Topic (EmergentMind, Feb 2026) | SASV unifies speaker verification + anti-spoofing. Key integration: score-level fusion, DNN embedding fusion, probabilistic product fusion. Cross-domain generalization is the primary unsolved challenge. |
| S12 | Surfing AI (Jun 2026) | Voice liveness detection must cover: replay attacks, TTS spoofing, voice conversion deepfakes, synthetic impersonation, call injection fraud. Datasets must include multiple spoof types, real device variability, environmental noise. |

### 1C. Speech Enhancement & Separation — 2026 Advances

| # | Source | Key Finding |
|---|--------|-------------|
| S13 | ICASSP 2026 URGENT Challenge (arXiv:2601.13531) | Track 1: universal speech enhancement. Track 2: speech quality assessment for enhanced speech. 80+ team registrations, 29 valid entries. SE is moving toward **universal enhancement** — single model handling noise, reverb, bandwidth limitation, and codec distortion simultaneously. |
| S14 | Geneses (arXiv:2601.18456, Jan 2026) | Unified generative speech enhancement and separation. End-to-end approaches concatenating SE→SS suffer from **complex degradations beyond additive noise** (reverb, codec, bandwidth). Generative models address this holistically. |
| S15 | ScienceDirect (Jul 2026) | Comprehensive review of noise reduction: transition from traditional statistical signal processing to **data-driven deep learning**. Domain-robust frameworks, efficient architectures, multimodal integration, self-supervised paradigms. |
| S16 | Audio Source Separation Market (Feb 2026) | Market: $1.8B (2025) → $2.34B (2026), CAGR 30%. Speech separation market growing rapidly. Key players: Deezer, AudioShake, Moises, LALAL.AI. |
| S17 | Zylos AI (Jan 2026) | Voice AI in 2026: sub-800ms end-to-end voice-to-voice latency. Krisp (real-time noise cancellation), ElevenLabs (multimodal conversational AI with text+voice concurrent processing). Speechmatics TTS: ~150ms first audio bytes. |

---

## Part 2: Defects Identified in NeoTrix Design

### DEFECT 385-1: No Streaming ASR Pipeline — Batch/Stream Bifurcation Gap

**Research basis**: S1, S2, S4, S6
**2026 reality**: Uni-ASR (arXiv:2603.11123) demonstrates that streaming and non-streaming ASR can be unified in a single LLM-based architecture. Deepgram Flux embeds end-of-turn detection directly in the speech model (<400ms). AssemblyAI emits immutable transcripts in ~300ms. The ASR market in 2026 has bifurcated into low-latency streaming (Deepgram, AssemblyAI) and high-accuracy batch (Whisper, NVIDIA Canary) — but the trend is toward unification.
**Existing gap**: `nt_feel_vtuber.rs:192` has `detect_from_voice(&self, _audio: &[u8])` with `_audio` prefixed — indicating the voice processing stub is not implemented. The `SubtitleEngine::generate()` at `nt_world_video_pipeline.rs:1385` accepts raw text, not streaming ASR output. No LocalAgreement-2 or similar streaming policy exists. No end-of-turn detection.
**Impact**: NT-PHYSICAL and NT-FEEL cannot process live spoken input in real-time. Voice-controlled smart home patterns (`nt_core_agent_patterns.rs:186`) are declared but not backed by streaming infrastructure. NeoTrix voice agent use cases (meeting notes, live captions, voice commands) are impossible without streaming ASR.
**Fix**:
1. Add `nt_io::speech::StreamingASR` module implementing LocalAgreement-2 streaming policy with Whisper-compatible backend (or expose Deepgram/AssemblyAI adapters via trait).
2. Implement `EndOfTurnDetector` — integrate Flux-style semantic+prosodic turn detection (not silence-based).
3. Define `StreamingTranscript` type with partial/final distinction, word-level timestamps, and speaker diarization fields.
4. Wire to EventBus as `SpeechTranscriptEvent` with partial/final flags.
**Effort**: High — 3-4 weeks for core streaming pipeline + adapter integration.

### DEFECT 385-2: No Spoofing-Aware Speaker Verification (SASV)

**Research basis**: S7, S8, S9, S10, S11, S12
**2026 reality**: Speaker verification alone is "demonstrably vulnerable" to AI voice cloning (S7). SASV (S9) integrates anti-spoofing countermeasure (AASIST/WPT-XLSR-AASIST) with speaker verification (ECAPA-TDNN). Top systems achieve SASV-EER <1%. But cross-domain generalization remains the primary unsolved challenge (a-DCF degrades 0.32–0.41 on out-of-domain). VoxENES 2026 (S10) shows legacy spoofing benchmarks overestimate detector robustness against LLM-era TTS.
**Existing gap**: No speaker verification module exists in NeoTrix. The `VoiceCloneGuard` was proposed in iteration_batch_343.md:84 but never implemented. NT-SHIELD handles network stealth and proxy pools but has no audio-level authentication. `detect_from_voice` in `nt_feel_vtuber.rs` is a stub. No ECAPA-TDNN or AASIST integration.
**Impact**: Any voice-authenticated interaction (call center, smart home, voice biometric login) is vulnerable to AI voice cloning attacks. NeoTrix cannot verify speaker identity or detect synthetic speech. Voice-initiated high-risk actions (account changes, data access) have no biometric gate.
**Fix**:
1. Add `nt_shield::voice_auth` module with:
   - `SpeakerVerifier` — ECAPA-TDNN embedding + cosine similarity scoring.
   - `AntiSpoofCountermeasure` — AASIST or WPT-XLSR-AASIST (wavelet prompt-tuned) for live/spoof detection.
   - `SASVEngine` — cascaded pipeline: CM → ASV → score-level fusion → decision.
2. Define `Voiceprint` type (enrolled embedding + metadata) and `VerificationResult` (score, decision, spoof_probability).
3. Add cross-domain adaptation: fine-tuning hooks for domain-specific voice data (e.g., noisy phone channels vs clean microphone).
4. Wire to EventBus as `VoiceAuthEvent` for audit trail.
**Effort**: High — 4-5 weeks (ECAPA-TDNN + AASIST integration + enrollment pipeline).

### DEFECT 385-3: No Universal Speech Enhancement Pipeline

**Research basis**: S13, S14, S15, S17
**2026 reality**: ICASSP 2026 URGENT Challenge (S13) demonstrates the shift toward **universal speech enhancement** — single models handling noise + reverb + bandwidth + codec distortions simultaneously. Geneses (S14) shows end-to-end SE→SS concatenation fails on complex degradations; generative models address this holistically. ScienceDirect review (S15) documents the full transition from statistical to deep-learning NR. Krisp (S17) achieves real-time noise cancellation for live calls.
**Existing gap**: NeoTrix has no speech enhancement module. The `AudioSyncPattern` (CONTEXT.md) for dynamic manga has no audio preprocessing. `nt_feel_vtuber.rs` stubs accept raw audio with no noise reduction or enhancement. The video pipeline (`nt_world_video_pipeline.rs`) processes audio for subtitles but doesn't enhance it.
**Impact**: Voice input in noisy environments (smart home, call center, outdoor) degrades ASR accuracy. Multi-speaker separation (cocktail party problem) is unsolved. NeoTrix cannot clean incoming audio before processing, causing error propagation through the entire speech stack.
**Fix**:
1. Add `nt_physical::audio_processing` module with:
   - `SpeechEnhancer` — real-time noise reduction (Krisp-style neural NR or DeepFilterNet).
   - `UniversalSE` — unified enhancement model (noise + reverb + bandwidth, per ICASSP 2026 URGENT).
   - `SpeechSeparator` — multi-speaker separation (DPRNN or TF-GridNet based).
2. Define `AudioStream` trait with `enhance()`, `separate()`, `denoise()` operations.
3. Pipeline order: `raw_audio → SpeechEnhancer → SpeakerSeparator → StreamingASR`.
4. Wire enhancement quality metrics to EventBus for monitoring.
**Effort**: Medium-High — 3-4 weeks (neural NR integration + separation model).

### DEFECT 385-4: No Paralinguistic/Emotion-from-Voice Extraction

**Research basis**: S5, S17
**2026 reality**: Modern ASR systems (S5) incorporate emotion detection, speaker diarization, and paralinguistic analysis (tone, pacing, emphasis) — feeding richer context into downstream processing. Voice AI in 2026 (S17) emphasizes emotional intelligence and multimodal integration. ElevenLabs (S17) processes both spoken language and typed text concurrently with emotional awareness.
**Existing gap**: `nt_feel::EmotionEngine` has 11 EmotionLabel variants but `detect_from_voice` is a stub (prefixed `_audio`). The `AffectBehaviorDecoupler` (proposed in iteration_batch_343.md:54) routes facial expression/voice tone but "voice tone" is not implemented — only a placeholder. No pitch/speech-rate/spectral analysis. The `DynamicParams` (speed/amplitude/frequency) exist but aren't connected to audio input.
**Impact**: NeoTrix cannot perceive emotion from voice tone. Voice interactions are emotionally blind — a user speaking with anger/fear/excitement is treated identically. This breaks the NT-FEEL emotion regulation loop for voice channels and undermines the E8 consciousness architecture's emotional grounding.
**Fix**:
1. Add `nt_feel::audio_affect::AudioAffectExtractor` that analyzes: pitch variation, speech rate, spectral energy distribution, voice intensity, and speaking style.
2. Map extracted features to `EmotionLabel` (11 variants) using trained classifier or heuristic rules.
3. Connect to `DynamicParams` — derive speed/amplitude/frequency from audio stream directly.
4. Wire as `VoiceEmotionEvent` to EventBus for NT-FEEL integration.
5. Feed into `AffectBehaviorDecoupler` as a first-class emotion channel.
**Effort**: Medium — 2-3 weeks (feature extraction + mapping + EventBus wiring).

### DEFECT 385-5: No Speech Separation / Cocktail Party Solution

**Research basis**: S14, S16
**2026 reality**: Speech separation (cocktail party problem) has seen "revolutionary advances" with DNNs (arXiv:2508.10830). Geneses (S14) demonstrates unified generative SE+SS models that handle complex degradations holistically. Market: $1.8B→$2.34B (2026), 30% CAGR (S16). Key players (AudioShake, Moises, LALAL.AI) provide production-grade source separation.
**Existing gap**: NeoTrix has no multi-speaker separation capability. The `SensoryIntegrationHub` and `PerceptionBridge` process audio as a single stream. No diarization → no per-speaker routing. The `SpeakerDiarization` capability is referenced in HuggingFace models (pyannote) but not integrated.
**Impact**: In multi-speaker environments (meetings, smart home with multiple users, call centers), NeoTrix cannot distinguish speakers or route audio to correct handlers. This makes voice-authenticated interactions unreliable in real-world (noisy, multi-speaker) conditions.
**Fix**:
1. Integrate `pyannote/speaker-diarization-3.1` or NVIDIA `diar_streaming_sortformer` for real-time speaker diarization.
2. Add `SpeechSeparator` module (TF-GridNet or DPRNN) for waveform-level source separation.
3. Define `SpeakerSegment` type with speaker_id, timestamp range, and separated audio.
4. Wire diarized segments to `StreamingASR` for per-speaker transcription.
**Effort**: Medium — 2-3 weeks (pyannote integration + separation model).

### DEFECT 385-6: No Cross-Domain Voice Robustness

**Research basis**: S9, S10, S11
**2026 reality**: SASV systems achieve SASV-EER <1% in-domain but degrade to a-DCF 0.32–0.41 on out-of-domain (S9). VoxENES 2026 (S10) shows legacy benchmarks overestimate robustness against LLM-era TTS. Cross-domain generalization is the primary unsolved challenge in voice biometrics (S11).
**Existing gap**: NeoTrix's voice modules (all stubs) have no domain adaptation strategy. No fine-tuning hooks for channel-specific data (phone vs microphone vs meeting room). No benchmark suite for cross-domain evaluation. The `Rune Socketing` system (5 rune colors) could theoretically configure domain-specific voice processing but no voice runes exist.
**Impact**: Voice authentication and ASR accuracy degrade unpredictably when deployed across different audio environments (phone calls, meeting rooms, outdoor, noisy cafes). A system trained on clean studio audio fails on phone channel audio. No systematic way to measure or improve cross-domain robustness.
**Fix**:
1. Define `VoiceDomain` enum: `CleanStudio`, `PhoneChannel`, `MeetingRoom`, `Outdoor`, `NoisyCafe`.
2. Add domain-specific fine-tuning pipeline: pre-trained base model + small adapter per domain (LoRA-style).
3. Create cross-domain evaluation suite using VoxENES 2026 methodology.
4. Wire domain classification to automatic adapter selection.
5. Add `VoiceRobustnessMetrics` to EventBus for monitoring domain performance.
**Effort**: High — 4-5 weeks (domain classification + adapter training + evaluation suite).

---

## Part 3: Summary

| ID | Defect | Severity | Effort |
|----|--------|----------|--------|
| 385-1 | No streaming ASR pipeline | Critical | High |
| 385-2 | No spoofing-aware speaker verification (SASV) | Critical | High |
| 385-3 | No universal speech enhancement | High | Medium-High |
| 385-4 | No paralinguistic/emotion-from-voice | High | Medium |
| 385-5 | No speech separation (cocktail party) | Medium | Medium |
| 385-6 | No cross-domain voice robustness | Medium | High |

**Total estimated effort**: 17-24 weeks for full speech stack implementation.

**Critical path**: 385-1 (streaming ASR) → 385-3 (enhancement) → 385-5 (separation) → 385-2 (SASV) → 385-4 (emotion) → 385-6 (robustness).

**Key 2026 insight**: The speech AI stack has moved from isolated components to **unified, multimodal pipelines**. NeoTrix's modular architecture (NT-IO, NT-FEEL, NT-PHYSICAL, NT-SHIELD) is well-positioned but currently lacks the connecting infrastructure. The biggest gap is the absence of any real-time audio processing pipeline — voice is the most underdeveloped modality in the consciousness architecture.
