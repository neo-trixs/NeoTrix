# Iteration Batch 481 — Audio Subsystem Gap Analysis

**Date**: 2026-09-06
**Scope**: Speech Synthesis, Audio Analysis, Music Information Retrieval
**Method**: External research scan → codebase gap identification → defect/fix proposal

---

## 1. Sources Cited

### Speech Synthesis (2026)
| # | Source | Key Finding |
|---|--------|-------------|
| S1 | DualSpeechLM (AAAI-26, Wang et al.) | Unified speech understanding + generation via dual speech token modeling; single USTokenizer for both modalities |
| S2 | UniSonate (ACL 2026, Qiang et al.) | Unified flow-matching framework for speech, music, and sound effects via MM-DiT with dynamic token injection |
| S3 | UniSRM (ACL 2026, Wang et al.) | Unified speech reward model for reasoning-based fine-grained assessment across quality, naturalness, coherence |
| S4 | VoXtream (ICASSP 2026) | Fully autoregressive zero-shot streaming TTS, 102ms initial delay |
| S5 | FlashTTS (Interspeech 2026) | Multi-token prediction + mean-flow distillation, ~325ms first-packet latency |
| S6 | IEAT (arXiv 2601.04960) | Unified spoken language model with injected emotional-attribution thinking for human-like interaction |
| S7 | awesome-text-to-speech (GitHub, 2026) | Curated catalog: SAE emotion control, LLaDA-TTS masked diffusion, Maya1 3B expressive TTS |

### Audio Analysis (2026)
| # | Source | Key Finding |
|---|--------|-------------|
| A1 | DCASE 2026 Challenge | 7 tasks: Heterogeneous Audio Classification (BST taxonomy), Noise-aware ASD, Spatial Semantic Segmentation, Audio Moment Retrieval, Domain-Agnostic Incremental Learning |
| A2 | WOOT (Signal Processing 2026, Pham et al.) | Open-World SED with deformable attention, feature disentanglement, one-to-many matching for known+unknown events |
| A3 | FlexSED (2025) | Open-vocabulary sound event detection |
| A4 | MOSS-Audio (OpenMOSS, 2026) | 4B/8B open-source audio foundation model: unified speech+sound+music+captioning+QA+reasoning |
| A5 | Audio-Visual Intelligence Survey (2026) | Unified taxonomy: Perception, Generation, Interaction; foundation model paradigm shift |
| A6 | DCASE Task 4 (arXiv 2604.00776) | Spatial Semantic Segmentation — joint detection + separation in complex spatial audio mixtures |

### Music Information Retrieval (2026)
| # | Source | Key Finding |
|---|--------|-------------|
| M1 | MuScriptor (Kyutai, July 2026) | Open multi-instrument transcription: Onset F1 60.4 (vs YourMT3+ 32.5), 170k real recordings, RL post-training |
| M2 | MuFun (arXiv 2508.01178) | Unified music foundation model: 390s long-context, multi-layer feature fusion, MuCUE benchmark |
| M3 | AMT Challenge 2025 (arXiv 2603.27528) | MusicFM self-supervised foundation model for music, multi-decoder T5 architecture |
| M4 | ISMIR 2026 (Abu Dhabi) | Theme "Crossroads" — MIR intersecting humanities, health, synesthesia |
| M5 | MIREX 2026 Beat Tracking | Beat tracking evaluation with listener-annotated beat locations |
| M6 | Multimodal AMT (ScienceDirect 2026) | Cross-modal Transformer integrating audio spectrograms + MusicXML for polyphonic transcription |

---

## 2. Codebase State (Current Audio Implementation)

| Module | File | Status |
|--------|------|--------|
| TTS/STT traits | `emotion/traits.rs:97-133` | Trait defined; implementations are placeholders |
| VTuber TTS | `nt_feel_vtuber.rs:274-293` | `synthesize_speech()` returns `vec![]`, `transcribe_speech()` returns `""` |
| Audio Orchestrator | `nt_act/audio_orchestrator.rs:151-153` | `generate_tts()` is a stub (`TODO: 实际调用 TTS API`) |
| Audio Mixing | `nt_act/audio_orchestrator.rs:209-212` | Ducking logic is `TODO: 实现闪避逻辑` |
| Audio Analysis | `nt_act/audio_orchestrator.rs:222-232` | `analyze_audio()` returns hardcoded values |
| Emotion from Voice | `nt_feel_vtuber.rs:192-201` | Always returns Neutral/0.5 (`TODO: 集成语音情绪识别模型`) |
| Audio Sync Library | `nt_physical/audio_sync_library.rs` | Static preset patterns only; no runtime beat detection |
| Voice Asset Registry | `nt_world/asset_registry.rs:130-147` | Storage only; no TTS pipeline, no voice cloning integration |

---

## 3. Defects Found (12 Total)

### D1: TTS/STT Entirely Stub — No Speech Synthesis Capability
**Severity**: Critical
**File**: `nt_act/audio_orchestrator.rs:151-153`, `nt_feel_vtuber.rs:274-293`
**Gap**: Both TTS and STT are placeholder stubs returning empty data. 2026 research shows speech synthesis has crossed into production-ready territory with streaming models (VoXtream 102ms latency, FlashTTS 325ms) and unified speech-language models (DualSpeechLM, UniSonate). NeoTrix has zero actual speech synthesis capability.
**Fix**: Integrate a streaming TTS provider (CosyVoice 2 for self-hosted, or ElevenLabs API) behind the existing `VoiceConfig` trait. Implement `generate_tts()` to call the provider. Add a streaming audio output buffer for real-time playback.

### D2: No Unified Speech Understanding Model
**Severity**: High
**File**: `emotion/traits.rs:121-129`
**Gap**: NeoTrix models TTS and STT as separate, disconnected operations. The 2026 paradigm (DualSpeechLM [S1], UniSonate [S2]) unifies speech understanding and generation in a single model with dual speech token modeling (semantic tokens for understanding, acoustic tokens for generation). This architecture enables zero-shot voice cloning, emotion-aware synthesis, and speech-to-speech translation.
**Fix**: Design a `SpeechFoundationModel` trait that wraps a unified speech-language model, providing both `understand()` and `generate()` from a single model instance. This aligns with the USTokenizer pattern from DualSpeechLM.

### D3: Emotion-from-Voice Always Returns Neutral
**Severity**: High
**File**: `nt_feel_vtuber.rs:192-201`
**Gap**: `detect_from_voice()` returns hardcoded Neutral/0.5. 2026 research shows SAE-based interpretable emotion control in TTS [S7], and IEAT [S6] enables emotion-aware reasoning internalized in the model. NeoTrix's emotion layer cannot actually perceive vocal emotion, making the entire multi-modal emotion detection pipeline (text+voice+visual) effectively unimodal.
**Fix**: Integrate a pre-trained audio emotion model (e.g., based on HuBERT/WavLM fine-tuned on emotion datasets). Map its output to `EmotionLabel` (11 variants). Connect to `EmotionLayer::detect_from_voice()` contract.

### D4: No Audio Event Detection / Sound Scene Analysis
**Severity**: High
**File**: No dedicated SED module exists
**Gap**: NeoTrix has no sound event detection capability. DCASE 2026 [A1] defines the state of the art with 7 tasks including Heterogeneous Audio Classification (BST taxonomy), Open-World SED [A2], Spatial Semantic Segmentation [A6], and Audio Moment Retrieval. The `nt_sense::embodied_senses.rs` module does simple volume/frequency perception but cannot classify or detect events.
**Fix**: Add `nt_sense::audio_event_detector` implementing DCASE-style SED. Use a foundation model backbone (MOSS-Audio [A4] or AudioSet-pretrained) for closed-world classification, with WOOT-style [A2] open-world extension for unknown event detection.

### D5: No Beat Tracking / Rhythm Analysis
**Severity**: High
**File**: `audio_sync_library.rs` (static patterns only)
**Gap**: `AudioSyncPattern` has a `music_beat` preset for "配乐节拍同步" but there is no runtime beat detection or tempo analysis. MIREX 2026 beat tracking [M5] and the 2025 AMT Challenge [M3] show beat tracking is foundational for MIR. Without runtime beat detection, the `music_beat` sync pattern cannot adapt to actual audio.
**Fix**: Add a `BeatTracker` module in `nt_physical` or `nt_sense` that wraps librosa-style beat tracking or a transformer-based model (e.g., Beat Transformer [ISMIR 2022]). Output beat onset times, tempo, and phase to feed `AudioSyncPattern::check_sync()`.

### D6: No Multi-Instrument Music Transcription
**Severity**: Medium-High
**File**: No AMT module exists
**Gap**: MuScriptor [M1] achieves Onset F1 of 60.4 on multi-instrument real recordings (vs 32.5 for prior SOTA). NeoTrix has no music transcription capability despite having `AudioType::Music` and beat sync patterns. This means the audio subsystem cannot analyze music content, only treat it as an opaque waveform.
**Fix**: Add `nt_sense::music_transcriber` wrapping MuScriptor or MT3 for audio→MIDI transcription. Enable beat extraction, chord recognition, and instrument identification from real recordings.

### D7: Ducking Logic Not Implemented
**Severity**: Medium
**File**: `audio_orchestrator.rs:209-212`
**Gap**: The `enable_ducking` flag exists in `AudioMixConfig` but the actual ducking implementation is `TODO`. Audio ducking (sidechain compression: lowering BGM when voiceover is active) is essential for multi-track mixing in dynamic comic production.
**Fix**: Implement the FFmpeg `sidechaincompress` filter chain in `generate_mix_command()`. The config already has `ducking_strength`, `ducking_attack_secs`, and `ducking_release_secs` — wire them to the filter graph.

### D8: No Audio Quality Assessment / Reward Model
**Severity**: Medium
**File**: `audio_orchestrator.rs:98-111`
**Gap**: `AudioAnalysis` only captures basic metrics (duration, sample rate, loudness). UniSRM [S3] shows that reasoning-based fine-grained assessment across quality, naturalness, and context-level coherence is now a solved problem. NeoTrix cannot evaluate whether its own audio output is high quality.
**Fix**: Add an `AudioQualityAssessor` trait with methods for MOS estimation, naturalness scoring, and emotion fidelity. Use UniSRM-style reasoning-based assessment to gate audio output quality in the production pipeline.

### D9: No Open-World Audio Classification
**Severity**: Medium
**File**: No module addresses unknown audio events
**Gap**: NeoTrix's audio processing is entirely closed-world (known types only). WOOT [A2] and DCASE 2026 Task 7 (Domain-Agnostic Incremental Learning) show that real-world audio systems must handle novel, unseen sound events. The `AudioType` enum is fixed (Ambient/Action/Music/Dialogue/Special).
**Fix**: Add an `OpenWorldAudioClassifier` that can detect and flag unknown audio events. Use feature disentanglement (class-specific vs class-agnostic) from WOOT. Store unknown events in KB for human review.

### D10: No Spatial Audio Processing
**Severity**: Medium
**File**: `nt_sense::embodied_senses.rs:280-420`
**Gap**: `embodied_senses.rs` handles audio perception with azimuth and volume attenuation but operates on mono sources. DCASE 2026 Task 3 (Semantic Acoustic Imaging) and Task 4 (Spatial Semantic Segmentation) [A6] require multi-channel spatial audio processing, source separation, and localization from spatial audio mixtures.
**Fix**: Extend `SoundSource` in `embodied_senses.rs` to support multi-channel (stereo/spatial) audio. Add source separation for overlapping events.

### D11: No Audio-to-Audio Generation (Voice Conversion / Style Transfer)
**Severity**: Medium
**File**: `emotion/traits.rs` has no voice conversion trait
**Gap**: 2026 voice AI includes voice cloning in 30 seconds, professional cloning from 30 minutes, and voice design from text prompts [S7 sources]. NeoTrix's `VoiceAsset` has `tts_model_path` and `sample_paths` but no pipeline for voice conversion or zero-shot cloning.
**Fix**: Add a `VoiceConversion` trait with `clone_voice(reference_audio, text)`, `convert_style(source_audio, target_style)`, and `design_voice(prompt)`. Integrate with a voice conversion model or API.

### D12: Audio Subsystem Not Connected to GWT Attention Routing
**Severity**: Low-Medium
**File**: No audio events flow to `nt_core_gwt`
**Gap**: GWT broadcasts salient information across specialist modules, but audio events (sound detection, beat detection, emotion-from-voice) never reach the GWT attention router. This means audio insights cannot modulate consciousness-level attention. The `PerceptionBridge` connects visual perception but has no audio equivalent.
**Fix**: Add an `AudioPerceptionBridge` that feeds audio event detections, beat tracking results, and voice emotion signals into GWT's salience computation. Audio salience should modulate attention alongside visual salience.

---

## 4. Suggestions (Ranked by Impact)

1. **Priority 1 — Speech Foundation Integration**: Implement D1+D2 together by integrating a unified speech-language model (CosyVoice 2 or SpeechLLM architecture) behind the existing trait contracts. This single change resolves TTS, STT, voice cloning, and emotion-from-voice simultaneously.

2. **Priority 2 — Audio Event Detection**: Implement D4 using MOSS-Audio [A4] or AudioSet-pretrained models. This is the foundation for D9 (open-world) and D10 (spatial).

3. **Priority 3 — Beat Tracking**: Implement D5 by wrapping a transformer-based beat tracker. This makes the existing `AudioSyncPattern::music_beat` functional rather than decorative.

4. **Priority 4 — Audio Quality Gate**: Implement D8 to create a self-assessment loop. Without quality feedback, the audio subsystem cannot self-improve (violates SEAL pipeline principles).

5. **Priority 5 — Complete Ducking**: Implement D7 — this is purely mechanical (FFmpeg filter graph) and unblocks multi-track mixing for dynamic comic production.

---

## 5. Cross-Dimension Synthesis

The overarching defect is that **NeoTrix's audio subsystem is a structural skeleton without nervous tissue**. The trait contracts and data structures exist (`VoiceConfig`, `AudioSyncPattern`, `AudioType`, `EmotionLayer::synthesize_speech`), but every implementation is a stub. Meanwhile, 2026 has moved to unified speech-language models that understand and generate in one pass, foundation audio models that handle speech+music+sound in a single architecture, and open-world classifiers that detect novel events.

The fix strategy should follow the **UniSonate principle** [S2]: unify fragmented audio capabilities (TTS, TTS, music generation, sound effects) into a single model architecture rather than building separate adapters for each.

---

*Batch 481 complete. 12 defects identified across 3 research domains. Next iteration: visual generation and video synthesis gap analysis.*
