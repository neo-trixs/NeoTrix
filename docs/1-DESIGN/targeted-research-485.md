# Targeted Research #485 — Internal Pain Points (WISER Loop)

**Date:** 2026-09-12
**Method:** Grep scan for TODO/FIXME, hardcoded returns, dead code in `neotrix-core/src/`
**Scope:** 3 critical internal pain points

---

## Pain Point 1 — VideoPostProcessor `denoise()`/`sharpen()` Return Fabricated Results

**File:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-368`
**Severity:** P0 (Critical — callers trust fake scores)

### What's Wrong

`denoise()` and `sharpen()` claim `success: true` and return fabricated quality scores (0.89, 0.87, etc.) without performing any actual video processing. Any downstream component that checks `success` or reads the scores will operate on fabricated data. The `processed_video_path` points to a file that doesn't exist.

```rust
// Lines 342-353 — current broken code
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    // TODO: 实际调用降噪逻辑
    _PostProcessResult {
        success: true,  // LIE: no processing happened
        processed_video_path: Some(format!("{}_denoised.mp4", video_path)),  // file doesn't exist
        processed_frames: 150,  // fabricated
        color_consistency_score: 0.89,  // fabricated
        temporal_stability_score: 0.87,  // fabricated
        quality_improvement_score: 0.85,  // fabricated
        processing_time_ms: 6000,  // fabricated
        error: None,
    }
}
```

### Fix (5-10 lines)

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    // Delegate to actual ffmpeg-based temporal denoise or return unsupported
    match self.ffmpeg_denoise(video_path) {
        Ok(out) => _PostProcessResult {
            success: true,
            processed_video_path: Some(out),
            processed_frames: 0, // filled by pipeline
            color_consistency_score: 0.0,
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Err(e) => _PostProcessResult {
            success: false,
            error: Some(format!("denoise unsupported: {e}")),
            ..Default::default()
        },
    }
}
```

---

## Pain Point 2 — QualityControl Auto-Approves Human/Platform Review With Fabricated Scores

**File:** `neotrix-core/src/l6_meta/coordination/quality_control.rs:283-312`
**Severity:** P0 (Critical — bypasses the entire review pipeline)

### What's Wrong

`_execute_review_flow()` iterates over review levels (`AI` → `Human` → `Platform`). The `Human` and `Platform` arms immediately return `ReviewStatus::Approved` with hardcoded scores (0.90, 0.88) and fabricated comments ("人工审核通过"). This means the 3-tier quality gate is effectively a no-op — every content passes regardless of quality.

```rust
// Lines 283-312 — current broken code
ReviewLevel::Human => {
    // TODO: 实际调用人工审核接口
    ReviewResult {
        status: ReviewStatus::Approved,  // ALWAYS passes
        total_score: 0.90,  // fabricated
        check_scores: HashMap::new(),  // empty
        comments: Some("人工审核通过".to_string()),  // fabricated
        review_time_ms: 0,  // no actual review
        .. // more fabricated fields
    }
}
```

### Fix (5-10 lines)

```rust
ReviewLevel::Human => {
    // Return Pending instead of Approved — human must actually review
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, 1),
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending,  // NOT Approved
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec!["requires_human_review".into()],
        comments: Some("awaiting human review queue".into()),
        review_time: 0,
        review_time_ms: 0,
    }
}
```

Same pattern for `ReviewLevel::Platform` — return `Pending` with a clear "awaiting platform" marker instead of fabricated approval.

---

## Pain Point 3 — Redundant Registries Marked for Merge (fusion-plan-215) Never Cleaned

**Files:**
- `neotrix-core/src/core/nt_core_subagent.rs:278-284` — `SubAgentRegistry`
- `neotrix-core/src/core/nt_core_reasoning.rs:70-71` — `MethodRegistry`
- `neotrix-core/src/core/nt_core_echo_terminal.rs:406-414` — `EchoPrmBridge`
**Severity:** P1 (Architectural debt — 3 redundant registries, each logging deprecation warnings on construction)

### What's Wrong

Three structs are marked `TODO(fusion-plan-215): Merge into X` but remain fully present and wired into the codebase. Each emits a `tracing::warn!` deprecation notice on every construction, polluting logs. They duplicate functionality already handled by `CapabilityRegistry`, `ReasoningStrategyRegistry`, and `MetaGoalBridge` respectively. This is dead weight that adds cognitive load and compile time without value.

```rust
// nt_core_subagent.rs:287-291
pub fn new() -> Self {
    SUBAGENT_REGISTRY_DEPRECATED.call_once(|| {
        tracing::warn!(
            "SubAgentRegistry is deprecated — merge into CapabilityRegistry (fusion-plan-215)"
        );
    });
    // ... full implementation still runs
}
```

### Fix (5-10 lines)

Either complete the merge or gate behind a feature flag to stop the warnings:

```rust
#[deprecated(note = "Use CapabilityRegistry instead (fusion-plan-215)")]
pub struct SubAgentRegistry { /* ... */ }

impl SubAgentRegistry {
    pub fn new() -> Self {
        // Remove the tracing::warn! — #[deprecated] handles notification.
        // Or, if merge is ready, replace body with:
        // panic!("SubAgentRegistry removed — use CapabilityRegistry")
        let user_dir = dirs::home_dir().unwrap_or_default().join(".neotrix/agents");
        Self { agents: HashMap::new(), source_dirs: vec![user_dir], scan_count: 0 }
    }
}
```

Apply the same `#[deprecated]` pattern to `MethodRegistry` and `EchoPrmBridge`, removing their `call_once` + `warn!` boilerplate.

---

## Summary

| # | Pain Point | Location | Severity | Impact |
|---|-----------|----------|----------|--------|
| 1 | VideoPostProcessor returns fabricated scores | `video_post_processor.rs:342-368` | P0 | Downstream trusts fake quality data |
| 2 | QualityControl auto-approves Human/Platform review | `quality_control.rs:283-312` | P0 | 3-tier review pipeline is a no-op |
| 3 | 3 redundant registries never merged (fusion-plan-215) | `nt_core_subagent.rs`, `nt_core_reasoning.rs`, `nt_core_echo_terminal.rs` | P1 | Deprecation warnings pollute logs, dead weight |
