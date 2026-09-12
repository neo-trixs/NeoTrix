# Targeted Research #476 — Internal Pain Points (Iteration 48)

> **Method**: grep for TODO/FIXME, hardcoded returns, simulate_* stubs
> **Date**: 2026-09-12
> **Scope**: neotrix-core/src/ internal pain points

---

## Pain Point 1: VerifierAgent uses `simulate_verification` instead of actual VLM

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:196`
**Severity**: P0

**What's wrong**: `_verify_shot()` calls `simulate_verification()` which uses keyword-matching heuristics to fabricate scores. The entire quality control verification pipeline produces meaningless results — every shot "passes" with fake scores. The `TODO: 实际调用 VLM 进行验证` at line 195 confirms this is a known stub.

**Impact**: Quality control pipeline is a no-op. Content that should fail verification passes silently.

**Fix sketch**:
```rust
// verifier_agent.rs:196 — replace simulate with real VLM call
pub(crate) fn _verify_shot(&mut self, shot_id: &str, video_path: &str, spec_description: &str, memory_context: Option<&str>) -> VerificationResult {
    let start = std::time::Instant::now();
    // Real path: extract key frames from video_path, send to VLM
    let frames = self.extract_frames(video_path, 5)?;
    let scores = self.verify_with_vlm(&frames, spec_description, memory_context)
        .unwrap_or_else(|e| {
            tracing::warn!("VLM verification failed, falling back to heuristic: {e}");
            self.simulate_verification(spec_description, memory_context)
        });
    // ... rest unchanged
}
```

---

## Pain Point 2: QualityControl hardcodes Approved for Human/Platform review levels

**File**: `neotrix-core/src/l6_meta/coordination/quality_control.rs:284-311`
**Severity**: P1

**What's wrong**: `_execute_review_flow()` returns `ReviewStatus::Approved` with hardcoded scores (0.90 for Human, 0.88 for Platform) for every content_id. The `TODO: 实际调用人工审核接口` and `TODO: 实际调用平台终审接口` confirm these are stubs. Any content passing through the review pipeline always gets approved.

**Impact**: Three-tier review (AI→Human→Platform) collapses to single-tier. Content quality gate is ineffective.

**Fix sketch**:
```rust
// quality_control.rs:283 — replace hardcoded Approved with pending state
ReviewLevel::Human => {
    let result = ReviewResult {
        review_id: format!("review_{}_{}", content_id, 1),
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending,  // NOT Approved — must be explicitly reviewed
        total_score: 0.0,                // No score until human acts
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("等待人工审核".to_string()),
        review_time: 0,
        review_time_ms: 0,
    };
    // Queue for human review notification
    self.emit_review_pending(&result);
    result
}
```

---

## Pain Point 3: KB `find_path` returns empty vec instead of error for no-path

**File**: `neotrix-core/src/l1_action/nt_memory/nt_memory_kb/mod.rs:2293`
**Severity**: P2

**What's wrong**: BFS shortest-path function returns `Ok(vec![])` when no path exists (line 2293). Callers cannot distinguish "no path exists" from "error occurred" — both return success. This causes silent failures in reasoning chains that depend on path discovery (e.g., cross-domain knowledge linking).

**Impact**: Downstream reasoning modules treat empty path as valid input, leading to incorrect transitive relationship inferences.

**Fix sketch**:
```rust
// mod.rs:2293 — return Err for no-path instead of empty Ok
Ok(vec![])
// Change to:
Err(format!("find_path: no path from '{from}' to '{to}' within depth {max_depth}"))
// Or better — return a dedicated enum:
enum PathResult { Found(Vec<String>), NoPath, MaxDepthExceeded }
```
