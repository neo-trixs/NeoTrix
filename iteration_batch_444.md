# Iteration Batch 444 — Audio/Music/Sound Design External Research

**Date**: 2026-09-06
**Research Focus**: AI Music Generation, Audio Synthesis/TTS, Sound Design/Spatial Audio (2026 advances)

---

## Sources Cited

| # | Source | Date | Focus |
|---|--------|------|-------|
| 1 | indiependr.ai — AI Music Composition in 2026 | 2026-06-09 | Suno/Udio/ElevenLabs composition workflow, style control limits |
| 2 | bytewaves.news — AI Music Generation 2026 | 2026-06-11 | Suno/Udio state of AI audio |
| 3 | digitalapplied.com — 7-axis matrix comparison | 2026-04-28 | License clarity, client workflows, platform comparison |
| 4 | youngju.dev — Deep Dive (Suno v4/Udio/MusicGen/AIVA/Mubert/Soundraw) | 2026-05-16 | Motif generation, guide tracks, model architectures |
| 5 | toolchase.com — Best AI Music Generators 2026 | 2026-08-28 | Use-case routing, Udio export restriction post-UMG settlement |
| 6 | artefactfx.com — AI Music Generators 2026 | 2026-06-01 | AI fingerprints, spectral artifacts, detection/removal |
| 7 | murmur — TTS Trends 2026 | 2026-06-09 | Realtime TTS, on-device models, agent stack integration |
| 8 | marktechpost — TTS Benchmarks 2026 | 2026-05-30 | IndexTTS-2, CosyVoice 2, VibeVoice, Fish Audio, decision matrix |
| 9 | zylos.ai — Voice AI State of the Art 2026 | 2026-01-25 | Open-source TTS models, latency benchmarks, voice-to-voice |
| 10 | musitechnic — Audio Industry Trends 2026 | 2026-05-07 | Spatial audio, Dolby Atmos mainstream, adaptive soundscapes |
| 11 | soundverse.ai — Spatial Audio Music Production 2026 | 2026-02-03 | AI-driven 3D audio, object-based mixing, HRTF |
| 12 | soundverse.ai — AI in Sound Design 2026 | 2026-01-31 | ML audio processing, timbre modeling, stem separation |
| 13 | adc conference — Procedural Audio Pipelines 2026 | 2026-08-21 | Klang/K++, Rhizome, procedural audio in games |
| 14 | trocglobal — CES 2026 Trends | 2026-01-16 | Synthetic voice, open-ear, spatial audio chips |
| 15 | generalistprogrammer — Game Audio Dev 2026 | 2026-06-17 | AI-assisted sound generation pipeline, middleware integration |

---

## Defects Found in NeoTrix Design

### DEFECT-1: No Music Generation Pipeline Exists (Critical Gap)
**Severity**: HIGH
**Evidence**: `nt_core_agent_patterns.rs:159-165` defines `ai-music-generator` as a pattern with `requires_voice: true`, but no actual module implements music generation. The `AudioSyncPattern` library (`audio_sync_library.rs`) only handles sync timing for pre-existing audio — it cannot generate music.
**2026 Reality**: Suno v5.5, Udio, ElevenLabs Music, AIVA, Mubert all provide full composition pipelines. The critical workflow is: text prompt → structured composition (verse/chorus/bridge) → stems → DAW integration.
**Suggestion**: Create `nt_act::music_generation` module with provider abstraction (Suno API, Udio API, ElevenLabs Music, local MusicGen 3.3B via audiocraft). Must support: (a) structured prompt → multi-section output, (b) stem separation (Demucs/Spleeter), (c) style reference upload, (d) MIDI export from AIVA-style orchestral generation.

### DEFECT-2: VoiceConfig Lacks 2026 TTS Provider Routing
**Severity**: HIGH
**Evidence**: `emotion/traits.rs:67-74` — `VoiceConfig` has `tts_provider: String` and `stt_provider: String` as raw strings with no provider registry or capability negotiation. `nt_feel_vtuber.rs:269` has `voice: None // TODO: 集成 TTS`.
**2026 Reality**: TTS is no longer one market — it's a stack of workflows (realtime agents, long-form narration, dubbing, on-device). Key models: ElevenLabs v3 (long-form), CosyVoice 2 (ultra-low-latency streaming, 0.5B params), VibeVoice (90-min long-form), Kokoro 82M (CPU-only), Fish Audio S2 Pro (multilingual). Each has different latency/quality/cost tradeoffs.
**Suggestion**: Replace `tts_provider: String` with a `TtsProviderRegistry` that exposes: capability negotiation (latency tier, streaming support, max duration, languages), provider health check, cost estimation, and automatic routing based on task requirements (realtime agent → CosyVoice 2, long-form narration → VibeVoice/ElevenLabs, budget → Kokoro).

### DEFECT-3: No Spatial Audio Pipeline
**Severity**: HIGH
**Evidence**: `audio_sync_library.rs` is stereo-only (no 3D positioning, no HRTF, no object-based audio). The `SoundSource` in `embodied_senses.rs:280+` uses `azimuth` and `distance` for a simple attenuation model, but no Dolby Atmos / Sony 360RA / binaural rendering.
**2026 Reality**: Dolby Atmos is now the industry standard for music on Apple Music, Tidal, Amazon Music HD. Spatial audio is mainstream across music, podcast, and film. AI tools analyze frequency/tone/rhythmic patterns to suggest optimal spatial positions (soundverse.ai). HRTF-based head-tracked rendering is expected on current-gen consoles and VR.
**Suggestion**: Create `nt_physical::spatial_audio` module with: (a) object-based audio model (position, velocity, size per sound source), (b) HRTF convolution for binaural rendering, (c) Dolby Atmos ADM-BWF export, (d) AI-assisted spatial positioning (frequency → height mapping, rhythmic → width mapping), (e) integration with `AudioSyncPattern` for 3D sync checking.

### DEFECT-4: No AI Fingerprint/Detection Awareness
**Severity**: MEDIUM
**Evidence**: No module tracks or detects AI-generated audio artifacts. `AudioSyncPattern` treats all audio identically.
**2026 Reality**: AI-generated music carries detectable spectral signatures — Suno produces 32 kHz sampling artifacts and high-frequency haze above 14 kHz; Mureka has harmonic predictability; Soundraw has compression artifacts. Detection models can identify platform-specific output. Google DeepMind × ElevenLabs launched SynthID audio watermarking.
**Suggestion**: Add `nt_world::audio_fingerprint` module that: (a) detects AI-generated audio via spectral analysis, (b) tracks SynthID watermarks for provenance, (c) provides quality scoring (human-likeness metrics), (d) integrates with Egress Privacy Guard to prevent AI-audio detection leaking NeoTrix internal audio processing capabilities.

### DEFECT-5: No Realtime Voice Agent Support
**Severity**: MEDIUM
**Evidence**: `VoiceInput` in `nt_act_voice` (referenced in `nt_world_sense_hub.rs:7`) is a simple capture mechanism. No support for turn-taking, interruption handling, emotional continuity, or sub-second response latency.
**2026 Reality**: Realtime voice agents require: <800ms total latency (voice→LLM→voice), turn-taking detection, interruption handling, emotional continuity across turns, streaming synthesis. Inworld TTS-2 and PilotTTS demonstrate natural language steering with bracketed directions, cross-lingual voice synthesis.
**Suggestion**: Create `nt_act::voice_agent` module with: (a) STT→LLM→TTS pipeline with streaming, (b) turn-taking detection (VAD + semantic boundary), (c) interruption/overlap handling, (d) emotional state carry-over between turns, (e) configurable latency/quality tradeoff (realtime vs quality mode).

### DEFECT-6: No Stem Separation / Audio Decomposition
**Severity**: MEDIUM
**Evidence**: `AssetRegistry` stores audio as opaque blobs (`VoiceAsset`). No capability to decompose mixed audio into stems.
**2026 Reality**: Stem separation (Demucs, Spleeter, Stable Audio Tools) is a standard prerequisite for remix, sample replacement, and AI-generated audio cleaning. Soundverse explicitly markets stem separation as a core AI tool.
**Suggestion**: Add `nt_act::stem_separator` that wraps Demucs (or equivalent) with: (a) vocal/drum/bass/other separation, (b) quality scoring per stem, (c) integration with `AssetRegistry` to store separated stems as individual assets, (d) batch processing for podcast/video audio tracks.

### DEFECT-7: No Procedural Audio Generation
**Severity**: MEDIUM
**Evidence**: `AudioSyncPattern` only syncs pre-existing audio. No runtime synthesis of sound effects or adaptive audio.
**2026 Reality**: Procedural audio pipelines (Klang/K++, Rhizome, MetaSounds) enable real-time synthesis of interactive sounds for games and VR. AI-generated ambience beds and foley variations are drafted in seconds, then layered in DAW. Adaptive soundscapes respond dynamically to context/user behavior.
**Suggestion**: Create `nt_physical::procedural_audio` module with: (a) FMOD/Wwise middleware integration, (b) parameter-driven sound generation (weather → ambient, tension → heartbeat), (c) AI-assisted foley generation from text descriptions, (d) runtime variation (no exact repeat), (e) memory-efficient streaming for long-form procedural textures.

### DEFECT-8: No Cross-Lingual Voice Consistency
**Severity**: LOW
**Evidence**: `VoiceConfig` has a single `language: String` field. No mechanism for maintaining voice identity across languages.
**2026 Reality**: Cross-lingual voice synthesis (maintaining voice characteristics across languages) is a key 2026 capability. Inworld TTS-2, Google Gemini native translation, and Fish Audio S2 Pro all support this. Critical for dubbing workflows.
**Suggestion**: Extend `VoiceConfig` with `language_map: HashMap<String, VoiceProfile>` where each `VoiceProfile` stores language-specific parameters (phoneme mapping, prosody adjustments) while preserving the base voice identity hash for cross-lingual consistency verification.

---

## Summary

| Category | Defects | Severity |
|----------|---------|----------|
| Music Generation | 1 (pipeline missing) | HIGH |
| TTS/Voice Synthesis | 3 (provider routing, realtime agent, cross-lingual) | HIGH/MED/LOW |
| Spatial Audio | 1 (3D pipeline missing) | HIGH |
| Audio Intelligence | 2 (fingerprint detection, stem separation) | MED/MED |
| Procedural Audio | 1 (runtime synthesis missing) | MED |

**Total**: 8 defects identified. 3 HIGH, 4 MEDIUM, 1 LOW.
**Priority Fix Order**: DEFECT-1 (music gen) → DEFECT-2 (TTS registry) → DEFECT-3 (spatial audio) → DEFECT-5 (voice agent) → DEFECT-6 (stem separation) → DEFECT-4 (fingerprint) → DEFECT-7 (procedural) → DEFECT-8 (cross-lingual).
