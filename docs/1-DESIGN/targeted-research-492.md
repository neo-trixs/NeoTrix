# Targeted Research #492 — Internal Pain Points (Iteration Loop)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/
**Approach:** WISER — Winnow pain points from code search, Improve with concrete fixes

---

## Pain Point 1: `video_post_processor` — Fake Processing Results (P0)

**File:** `l3_embodiment/nt_physical/video_post_processor.rs:342-368`

**What's wrong:**
`denoise()` and `sharpen()` return hardcoded `_PostProcessResult` structs with fake scores (0.89, 0.87, etc.) and fabricated frame counts. Callers (like `VideoPostProcessor`) trust these results for quality decisions. The functions don't actually process any video — they just append a suffix to the filename and lie about success.

**Severity:** P0 — Users receive "processed" files that are identical to input. Quality scores are fabricated, breaking any downstream quality gate.

**Concrete fix:**
```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    // Delegate to ffmpeg denoise or return explicit Unsupported
    let output_path = format!("{}_denoised.mp4", video_path);
    match std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:p=3:r=9", &output_path])
        .output()
    {
        Ok(o) if o.status.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output_path),
            processed_frames: 0, // filled by ffprobe post-hoc
            color_consistency_score: 0.0,
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: 0,
            error: None,
        },
        Ok(o) => _PostProcessResult {
            success: false,
            processed_video_path: None,
            error: Some(String::from_utf8_lossy(&o.stderr).into()),
            ..Default::default()
        },
        Err(e) => _PostProcessResult {
            success: false,
            error: Some(format!("ffmpeg not available: {e}")),
            ..Default::default()
        },
    }
}
```

---

## Pain Point 2: `production_orchestrator` — Checkpoint Save/Restore Is No-Op (P0)

**File:** `l1_action/nt_act/actions/production_orchestrator.rs:224-242`

**What's wrong:**
`save_checkpoint()` accepts a workflow ID but does zero persistence — the `Ok(())` is returned unconditionally. `restore_from_checkpoint()` just flips status to `Paused` without restoring any state. This means any long-running batch production loses all progress on crash or restart. The checkpoint system is the safety net for `ProductionOrchestrator`; without it, there is no crash recovery.

**Severity:** P0 — Production workflows lose all progress on failure. Checkpoint feature is advertised but non-functional.

**Concrete fix:**
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let path = std::path::Path::new(&self.config.checkpoint_dir)
        .join(format!("{workflow_id}.json"));
    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("创建目录失败: {e}"))?;
    let json = serde_json::to_string_pretty(workflow)
        .map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("写入检查点失败: {e}"))
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = std::path::Path::new(&self.config.checkpoint_dir)
        .join(format!("{workflow_id}.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取检查点失败: {e}"))?;
    let restored: Workflow = serde_json::from_str(&json)
        .map_err(|e| format!("反序列化失败: {e}"))?;
    self.workflows.insert(workflow_id.to_string(), restored);
    Ok(())
}
```

---

## Pain Point 3: `quality_control` — Human & Platform Review Auto-Approve (P1)

**File:** `l6_meta/coordination/quality_control.rs:283-311`

**What's wrong:**
`_execute_review_flow()` has a 3-stage review pipeline (AI → Human → Platform). The Human and Platform stages return hardcoded `ReviewStatus::Approved` with fabricated scores (0.90, 0.88) without calling any external interface. This defeats the entire purpose of the quality gate — content passes all three stages automatically. The `TODO` comments acknowledge this but the hardcoded approval is silently dangerous.

**Severity:** P1 — Quality control pipeline is theater. Content auto-approves through human and platform stages, defeating the 3-tier safety model documented in CONTEXT.md under `QualityControlPipeline`.

**Concrete fix:**
```rust
ReviewLevel::Human => {
    let result = self.human_review_client
        .submit(content_id)
        .await
        .map(|r| ReviewResult {
            review_id: format!("review_{}_{}", content_id, r.reviewer_id),
            content_id: content_id.to_string(),
            level: ReviewLevel::Human,
            status: if r.passed { ReviewStatus::Approved } else { ReviewStatus::Rejected },
            total_score: r.score,
            check_scores: r.check_scores,
            issues: r.issues,
            comments: r.comments,
            review_time: r.review_time,
            review_time_ms: r.review_time_ms,
        })
        .unwrap_or_else(|e| ReviewResult {
            review_id: format!("review_{}_err", content_id),
            content_id: content_id.to_string(),
            level: ReviewLevel::Human,
            status: ReviewStatus::Pending, // NOT auto-approved on error
            total_score: 0.0,
            check_scores: HashMap::new(),
            issues: vec![format!("人工审核接口调用失败: {e}")],
            comments: None,
            review_time: 0,
            review_time_ms: 0,
        });
    results.push(result);
}
```

---

## Summary

| # | File | Issue | Severity | Impact |
|---|------|-------|----------|--------|
| 1 | `video_post_processor.rs:342` | Fake denoise/sharpen results | **P0** | Users get unprocessed files with fake quality scores |
| 2 | `production_orchestrator.rs:224` | Checkpoint save/restore is no-op | **P0** | Batch production crashes lose all progress |
| 3 | `quality_control.rs:283` | Human/Platform review auto-approve | **P1** | Quality gate is theater — all content passes |

All three follow the same anti-pattern: public API surface returns fabricated success while the real work is stubbed with `TODO`. P0s should be fixed before any release; P1 should be fixed before beta.
