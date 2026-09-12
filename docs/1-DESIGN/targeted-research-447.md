# Targeted Research 447 — Internal Pain Point Discovery

**Date:** 2026-09-12
**Method:** Grep for TODO/FIXME/HACK, hardcoded returns, dead branches in `neotrix-core/src/`
**Context:** Continuing WISER iteration loop. After fixing 8 critical issues (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB search, value_function, content_moderation, checkpoint_persistence, rate_limiter cleanup), we now hunt for the next batch.

---

## Pain Point 1 — TemporalContinuityChecker.calculate_frame_diff() returns hardcoded 0.05

**File:** `neotrix-core/src/l1_action/nt_act/temporal_continuity.rs:180`
**Severity:** P0

### What's wrong

The core frame comparison function returns a constant `0.05` regardless of input:

```rust
fn calculate_frame_diff(&self, _frame_a: &str, _frame_b: &str) -> f32 {
    // TODO: 实际调用帧差异计算
    0.05
}
```

`_frame_a` and `_frame_b` are ignored. Every caller of `check_first_last_frame()` gets fabricated diff values. The `check_scene_transition()` and `check_element_position()` methods always return `passed: true` with zero issues, making the entire 350-line module a non-functional shell.

**Impact:** Any video pipeline calling `TemporalContinuityChecker::check_all()` will never detect frame discontinuities, scene breaks, or element drift — the quality gate is blind.

### Fix (concrete sketch)

```rust
// Replace the stub with a real hash-based diff (fast, no deps):
fn calculate_frame_diff(&self, frame_a: &str, frame_b: &str) -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let hash = |s: &str| -> u64 {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    };
    let (ha, hb) = (hash(frame_a), hash(frame_b));
    let xor = ha ^ hb;
    let bits_set = xor.count_ones() as f64;
    bits_set / 64.0 // normalized Hamming distance
}
```

---

## Pain Point 2 — VerifierAgent._verify_shot() calls simulate_verification() with hardcoded scores

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-242`
**Severity:** P0

### What's wrong

The VLM verification entry point does not call any VLM. Instead it calls `simulate_verification()` which returns hardcoded scores (8, 7, 8, 9):

```rust
fn simulate_verification(&self, _description: &str, _context: Option<&str>) -> Vec<_VerificationScore> {
    vec![
        _VerificationScore { dimension: EntityConsistency, score: 8, ... },
        _VerificationScore { dimension: EnvironmentConsistency, score: 7, ... },
        _VerificationScore { dimension: NarrativeProgression, score: 8, ... },
        _VerificationScore { dimension: InstructionFollowing, score: 9, ... },
    ]
}
```

All 4 dimensions are always populated, all scores are always ≥7, so `passed` is always true (threshold is typically 0.7). The entire regeneration loop (`_generate_regeneration_request`) is dead code because verification never fails.

**Impact:** Video shots pass verification without any real VLM inspection. The regeneration pipeline is never triggered, so quality issues propagate unchecked.

### Fix (concrete sketch)

```rust
fn _verify_shot(&mut self, shot_id: &str, video_path: &str,
    spec_description: &str, memory_context: Option<&str>
) -> VerificationResult {
    let start = std::time::Instant::now();
    // Actual: send frames to VLM endpoint
    let scores = self.call_vlm_for_verification(video_path, spec_description, memory_context);
    let total_score = self.calculate_total_score(&scores);
    let passed = total_score >= self.config.pass_threshold;
    let needs_regeneration = !passed
        && self.history.len() < self.config.max_regeneration_attempts as usize;
    let result = VerificationResult {
        passed, total_score, scores,
        error_types: vec![],
        suggested_corrections: self.generate_corrections(&scores),
        needs_regeneration,
        verification_time_ms: start.elapsed().as_millis() as u64,
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 3 — QualityControlPipeline.evaluate_check_item() returns hardcoded scores

**File:** `neotrix-core/src/l6_meta/coordination/quality_control.rs:246-255`
**Severity:** P0

### What's wrong

Every quality check type returns a fixed fake score:

```rust
fn evaluate_check_item(&self, check_type: &_QualityCheckType) -> f32 {
    // TODO: 实际调用评估逻辑
    match check_type {
        Technical => 0.92, Compliance => 0.95,
        VisualConsistency => 0.88, NarrativeCoherence => 0.90,
        AudioVisualSync => 0.85, Performance => 0.87,
    }
}
```

The 3-level review flow (AI→Human→Platform) in `_execute_review_flow()` is therefore completely fabricated: AI always scores 0.85-0.95, Human always approves, Platform always approves. Tests pass because they assert against these fake constants.

**Impact:** Content moderation quality gates are no-ops. Low-quality video can pass through AI→Human→Platform without any real inspection.

### Fix (concrete sketch)

```rust
fn evaluate_check_item(&self, check_type: &_QualityCheckType) -> f32 {
    match check_type {
        Technical => self.evaluate_technical_quality(),
        Compliance => self.evaluate_compliance(),
        VisualConsistency => self.evaluate_visual_consistency(),
        NarrativeCoherence => self.evaluate_narrative_coherence(),
        AudioVisualSync => self.evaluate_av_sync(),
        Performance => self.evaluate_performance(),
    }
}

fn evaluate_visual_consistency(&self) -> f32 {
    // Use perceptual hash comparison across frames
    // or delegate to VLM for semantic consistency check
    self.vlm_client
        .call_consistency_check(&self.current_content_path)
        .unwrap_or(0.0)
}
```

---

## Summary

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| 1 | `temporal_continuity.rs:180` | `calculate_frame_diff()` returns hardcoded 0.05, all frame checks are no-ops | P0 |
| 2 | `verifier_agent.rs:195-242` | VLM verification calls `simulate_verification()` with constant scores, regeneration loop dead | P0 |
| 3 | `quality_control.rs:246-255` | `evaluate_check_item()` returns fixed scores, entire 3-level review flow is fake | P0 |

All three follow the same anti-pattern: **stub functions that return plausible-looking but fabricated results**, making downstream quality gates blind. They were likely scaffolded during initial development and never connected to real evaluation logic.
