# Targeted Research #488 — Internal Pain Points (WISER Loop Iteration 29)

**Date**: 2026-09-12
**Method**: Grep-driven sweep of `neotrix-core/src/` for TODO/FIXME stubs, hardcoded return values, and dead stubs-in-production-paths.
**Scope**: 3 high-severity pain points — deceptive stubs that silently produce fake success rather than failing honestly.

---

## Pain Point 1: `reference_generation.rs` — Fabricated Success Metrics

**Severity**: P0 (Data Integrity / Deceptive Output)
**File**: `neotrix-core/src/l1_action/nt_io/reference_generation.rs:224-272`
**What's wrong**: `video_to_video()`, `image_to_video()`, and `style_transfer()` return `success: true` with fabricated metrics (quality_score: 0.86–0.90, reference_similarity: 0.85–0.89, fake output paths like `{input}_generated.mp4`). Callers believe generation succeeded, write downstream workflows on phantom files. Unlike `image_to_image()` (which correctly returns `success: false`), these three methods lie.

**Why it's P0**: Any production pipeline using these methods will silently produce empty results while logging "success" — the worst failure mode (silent data loss).

### Fix Sketch

```rust
// neotrix-core/src/l1_action/nt_io/reference_generation.rs:224
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    let result = GenerationResult {
        success: false,
        output_paths: vec![],
        generation_time_ms: start.elapsed().as_millis() as u64,
        model_used: self.config.model_name.clone(),
        reference_similarity: 0.0,
        quality_score: 0.0,
        error: Some(format!(
            "video_to_video not yet implemented for model: {}. \
             Use nt_io_provider::gateway::unified_inference for actual inference.",
            self.config.model_name
        )),
    };
    self.history.push(result.clone());
    result
}
// Apply identical pattern to image_to_video() (line 241) and style_transfer() (line 258)
```

---

## Pain Point 2: `nt_feel_vtuber.rs` — Always-Neutral Emotion Detectors

**Severity**: P1 (Phantom Sensor / Silent Degradation)
**File**: `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-228`
**What's wrong**: `detect_from_voice()` and `detect_from_visual()` hardcode `_EmotionType::Neutral` at intensity 0.5 regardless of input. These are registered as real emotion sensors in the VTuber pipeline — downstream expression rendering, response generation, and persona adaptation all receive fabricated "neutral" readings, making the entire emotion system appear functional while being completely inert.

**Why it's P1**: The VTuber subsystem presents these as working sensors. Any integration test or demo will show "neutral" for all inputs, masking that the emotion pipeline is entirely disconnected from reality.

### Fix Sketch

```rust
// neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    // Return explicit not-implemented rather than fake neutral
    Err("Voice emotion detection requires ML model integration \
         (e.g. whisper- sentiment or wav2vec2-emotion). \
         Not wired — returning error to prevent silent neutral bias.".into())
}

pub fn detect_from_visual(&self, _image: &[u8]) -> Result<_EmotionReading, String> {
    Err("Visual emotion detection requires CV model integration \
         (e.g. FER or DeepFace). Not wired — returning error.".into())
}
```

---

## Pain Point 3: `provider_migration_router.rs` — No-Op Migration Execution

**Severity**: P1 (Silent Failure / Data Loss Risk)
**File**: `neotrix-core/src/l1_action/nt_act/actions/provider_migration_router.rs:273-280`
**What's wrong**: `_execute_migration()` sets status to `InProgress` and immediately returns `true` — no actual provider state is migrated. Callers believe migration completed successfully. The plan appears finished while the system remains on the old provider. Additionally, the method is prefixed with `_` (dead code convention), so it's never called — but the public `create_migration_plan()` populates plans that will never execute.

**Why it's P1**: If any future caller uses this (or the underscore prefix is removed), migrations will silently "succeed" while doing nothing — risking traffic continuing to a degraded provider.

### Fix Sketch

```rust
// neotrix-core/src/l1_action/nt_act/actions/provider_migration_router.rs:273
pub(crate) fn _execute_migration(&mut self, plan_id: usize) -> Result<bool, String> {
    let plan = self.migration_plans.get_mut(plan_id)
        .ok_or_else(|| format!("Migration plan {} not found", plan_id))?;

    plan.status = MigrationStatus::InProgress;

    // 1. Validate source provider is healthy before draining
    // 2. Update routing config to point to target provider
    // 3. Drain in-flight requests from source (graceful drain period)
    // 4. Mark plan Completed only after drain completes
    // 5. On failure: revert to source, mark Failed, log to EventBus

    Err("Migration execution not implemented — refusing to claim success \
         without actual provider state transition".into())
}
```

---

## Summary

| # | File:Line | Issue | Severity | Category |
|---|-----------|-------|----------|----------|
| 1 | `reference_generation.rs:224-272` | Fabricated `success: true` + fake metrics in 3 methods | **P0** | Deceptive output |
| 2 | `nt_feel_vtuber.rs:207-228` | Always-Neutral hardcode in emotion sensors | **P1** | Phantom sensor |
| 3 | `provider_migration_router.rs:273-280` | No-op migration marked as success | **P1** | Silent failure |

**Pattern observed**: All three are **stub implementations that return success instead of error**. The codebase convention (visible in `image_to_image` at line 205) is correct — return `success: false` + error message. These three are deviations from that convention and should be aligned.

**Total TODO/FIXME count in `neotrix-core/src/`**: 87 production-path TODOs (excluding test-only comments). The 3 above are highest severity because they actively deceive callers.
