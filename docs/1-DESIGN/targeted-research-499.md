# Targeted Research #499 — Internal Pain Points (Iteration 49)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — Stub functions, hardcoded returns, dead code paths
**Method:** grep TODO/FIXME/HACK + hardcoded return patterns + dead_code annotations

---

## Pain Point 1: Speculative Decoding — Empty Token Generation

**File:** `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-185`
**Severity:** P1

**What's wrong:** `generate()` returns an empty `output_tokens: vec![]` with fabricated metrics. The function claims 2-4x throughput improvement and calculates fake acceptance/rejection counts, but produces zero actual tokens. Callers receive a `SpeculativeResult` that looks valid but contains no output. The entire speculative decoding module (EAGLE, Medusa, MTP, DFlash) is a data structure shell — no real draft model inference, no verification loop, no token acceptance logic.

**Impact:** Any code path calling `SpeculativeDecoder::generate()` silently gets empty output while metrics suggest success. This is a latent correctness bug: the system reports throughput multipliers for work it never did.

**Fix sketch:**
```rust
// Option A: Return Err for unimplemented path (fail-closed)
pub async fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<SpeculativeResult, String> {
    Err("speculative decoding not implemented — use direct inference".into())
}

// Option B: Delegate to direct inference as fallback
pub async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<SpeculativeResult, String> {
    let output = self.target_model.direct_generate(prompt, max_tokens).await?;
    Ok(SpeculativeResult {
        output_tokens: output,
        total_draft_tokens: 0,
        accepted_count: 0,
        rejection_count: 0,
        throughput_multiplier: 1.0,
        latency_savings_ms: 0,
    })
}
```

---

## Pain Point 2: Video Post-Processor — Hardcoded Fake Metrics

**File:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-368`
**Severity:** P1

**What's wrong:** `denoise()` and `sharpen()` return identical hardcoded structures: always 150 frames processed, deterministic scores (0.89/0.87/0.85 and 0.88/0.86/0.82), and fabricated output paths. No actual video processing occurs — no frame extraction, no filter application, no file I/O. The output path `"{video_path}_denoised.mp4"` is never created.

**Impact:** Downstream consumers (pipeline orchestrators, quality gates) trust these results and may report "denoising complete" to users while the original video is unchanged. This is a silent data integrity failure.

**Fix sketch:**
```rust
pub fn denoise(&self, video_path: &str) -> Result<_PostProcessResult, String> {
    let input = std::fs::metadata(video_path)
        .map_err(|e| format!("input not found: {e}"))?;
    if input.len() == 0 {
        return Err("empty video file".into());
    }
    // Actual: shell out to ffmpeg with nlmeans/temporal filters
    let status = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:p=7:t=3",
               "-y", &format!("{video_path}_denoised.mp4")])
        .status()
        .map_err(|e| format!("ffmpeg launch failed: {e}"))?;
    if !status.success() {
        return Err("ffmpeg denoise failed".into());
    }
    Ok(_PostProcessResult { success: true, /* ... */ })
}
```

---

## Pain Point 3: Production Orchestrator Checkpoint — No Persistence

**File:** `neotrix-core/src/l1_action/nt_act/actions/production_orchestrator.rs:224-242`
**Severity:** P1

**What's wrong:** `save_checkpoint()` discards the workflow reference (`let _ = workflow;`) and returns `Ok(())` without writing anything to disk. `restore_from_checkpoint()` sets status to `Paused` but never reads from or writes to any storage — it's a pure in-memory state mutation with no durability. Checkpoint IDs exist in the type system but have no backing store.

**Impact:** If the process crashes mid-workflow, all progress is lost. The `save_checkpoint`/`restore_from_checkpoint` API implies durability but provides none. Users calling these functions get false confidence that state is safe.

**Fix sketch:**
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("workflow not found")?;
    let path = std::path::Path::new(&self.config.storage_path)
        .join(format!("{workflow_id}.checkpoint.json"));
    let json = serde_json::to_string_pretty(workflow)
        .map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("write failed: {e}"))?;
    Ok(())
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = std::path::Path::new(&self.config.storage_path)
        .join(format!("{workflow_id}.checkpoint.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("read failed: {e}"))?;
    let restored: Workflow = serde_json::from_str(&json)
        .map_err(|e| format!("deserialize failed: {e}"))?;
    self.workflows.insert(workflow_id.to_string(), restored);
    Ok(())
}
```

---

## Summary

| # | File | Pain Point | Severity | Category |
|---|------|-----------|----------|----------|
| 1 | `speculative_decoding.rs:161` | `generate()` returns empty tokens + fake metrics | P1 | Stub returning fabricated data |
| 2 | `video_post_processor.rs:342,357` | `denoise()`/`sharpen()` return hardcoded fake scores | P1 | Stub returning fabricated data |
| 3 | `production_orchestrator.rs:224,235` | Checkpoint save/restore are no-ops | P1 | Silent durability loss |

**Pattern observed:** All 3 pain points share the same anti-pattern — functions with non-`Option`/non-`Result` return types that always succeed but produce fabricated results. The codebase would benefit from a lint rule: any function returning a result struct with `success: true` must either (a) write at least one byte to disk/network, or (b) be annotated `#[cfg(test)]` or `#[deprecated]`.
