# Targeted Research 520 — Internal Pain Points

**Date:** 2026-09-13
**Scope:** neotrix-core/src — hardcoded fakes, silent failures, bypassed gates

---

## Pain Point 1: Sandbox Remote Mode — Silent Failure

**File:** `l3_embodiment/nt_shield/nt_shield_sandbox_entry.rs:60-70`
**Severity:** P0

### What's Wrong

`SandboxMode::Remote` returns a hardcoded `_SandboxResult` with `exit_code: 1` and empty stdout. Callers using `execute()` may not check `exit_code` and proceed with empty stdout as if execution succeeded. The `Remote` variant is exposed in the `SandboxMode` enum but has zero implementation — it's a silent dead end.

### Fix (5-10 lines)

```rust
SandboxMode::Remote => _SandboxResult {
    stdout: String::new(),
    stderr: "Remote sandbox not implemented — use Local/Docker/Wasm, \
             or implement exec_remote() with SSH/API transport"
        .to_string(),
    exit_code: 127, // 127 = command not found, conventional
},
// Additionally: log a tracing::error so the issue surfaces in telemetry
// tracing::error!("SandboxMode::Remote invoked but unimplemented — \
//                  caller: {}", std::hint::black_box(cmd));
```

**Action:** Add `tracing::error!` to surface in HeartbeatAggregator, and rename the variant to `Remote_NotImplemented` or gate it at construction time with `compile_error!` / `unreachable!` to prevent silent misuse.

---

## Pain Point 2: Quality Control Human/Platform Review — Fake Approvals

**File:** `l6_meta/coordination/quality_control.rs:283-311`
**Severity:** P1

### What's Wrong

`_execute_review_flow()` matches `ReviewLevel::Human` and `ReviewLevel::Platform` and returns hardcoded `ReviewStatus::Approved` with fake scores (0.90, 0.88), zero review time, and no actual review logic. This completely bypasses the 3-tier quality gate (AI → Human → Platform) that `QualityControlPipeline` claims to enforce. Any content routed through this pipeline gets rubber-stamped.

### Fix (5-10 lines)

```rust
ReviewLevel::Human => {
    // Gate: require external review callback or return Pending
    let reviewer = self.human_reviewer.as_ref().ok_or_else(|| {
        tracing::warn!("Human review requested but no reviewer registered");
    });
    match reviewer {
        Ok(cb) => cb.review(content_id)?,
        Err(_) => ReviewResult {
            status: ReviewStatus::Pending,
            comments: Some("Awaiting human reviewer — none registered".into()),
            review_time_ms: 0,
            ..self.empty_result(content_id, ReviewLevel::Human)
        }
    }
}
```

**Action:** Add a `human_reviewer: Option<Box<dyn ReviewCallback>>` field. When `None`, return `Pending` instead of fake `Approved`. Same pattern for `Platform`.

---

## Pain Point 3: Visual Generation Stack — Hardcoded Fake Results

**Files:**
- `l5_cognition/nt_core/visual/face_consistency.rs:168-186`
- `l1_action/nt_io/reference_generation.rs:224-267`
- `l1_action/nt_io/model_adapter.rs:157-224`

**Severity:** P1

### What's Wrong

All three files return hardcoded `success: true` with fabricated similarity/quality scores (0.85-0.95) and fabricated file paths (`*_fixed.png`, `*_generated.mp4`). These functions are exposed via `UnifiedCapability` and the capability registry — they look production-ready but return fake data. A user calling `video_to_video()` gets a path to a file that doesn't exist, with a quality score of 0.86 that means nothing.

### Fix (5-10 lines per function)

```rust
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    // Delegate to actual backend (ComfyUI / SD / Runway)
    let backend = self.backend.as_ref().ok_or_else(|| {
        tracing::error!("video_to_video called but no backend configured");
        GenerationError::NoBackend
    })?;
    let result = backend.run_video2video(input_path, &self.config)?;
    self.history.push(result.clone());
    result
}
```

**Action:** Add `backend: Option<Box<dyn GenerationBackend>>` to the config structs. When `None`, return `GenerationError::NoBackend` (not a fake success). This converts silent fake data into a detectable error that the EventBus can track.

---

## Summary

| # | File | Severity | Pattern | Fix |
|---|------|----------|---------|-----|
| 1 | `nt_shield_sandbox_entry.rs:65` | P0 | Silent failure on unimplemented variant | Log + conventional exit code + gate at construction |
| 2 | `quality_control.rs:283-311` | P1 | Fake approval bypasses quality gate | Return `Pending` when no reviewer registered |
| 3 | `face_consistency.rs` + `reference_generation.rs` + `model_adapter.rs` | P1 | Hardcoded fake scores/paths | Return `NoBackend` error when backend absent |

**Next steps:** Implement fixes, verify with `cargo check --all-targets -p neotrix`.
