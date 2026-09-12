# Targeted Research #449 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-12
**Approach**: Internal-first pain point identification (WISER)
**Prior fixes (12)**: FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB search, value_function, content_moderation, checkpoint_persistence, rate_limiter cleanup, temporal_continuity, verifier_agent, quality_control, unified_api

---

## Pain Point 1: `checkpoint_persistence.rs:302` — `cleanup_expired()` Silently Leaks Files

**Severity**: P1

**Location**: `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs:298-305`

**What's wrong**: `cleanup_expired()` removes entries from the in-memory `index` HashMap but never deletes the corresponding checkpoint files from disk. Every expired checkpoint accumulates on disk forever. The `file_size` field at line 147 is also hardcoded to `0`, so `statistics().total_size` is always zero.

**Impact**: Disk usage grows unboundedly; `statistics()` reports 0 bytes even when gigabytes of checkpoint files exist; operators have no way to know storage is being consumed.

**Concrete fix (code sketch)**:

```rust
pub fn cleanup_expired(&mut self) -> usize {
    let now = current_timestamp();
    let expired: Vec<String> = self.index.iter()
        .filter(|(_, meta)| meta.expires_at.map_or(false, |exp| exp < now))
        .map(|(id, _)| id.clone())
        .collect();

    let count = expired.len();
    for id in &expired {
        if let Some(meta) = self.index.remove(id) {
            let file_path = std::path::Path::new(&self.config.storage_path)
                .join(format!("{}.json", id));
            let _ = std::fs::remove_file(&file_path);
        }
    }
    count
}
```

Also fix line 147:
```rust
// Replace:  file_size: 0, // TODO: 计算实际大小
file_size: serde_json::to_string(&data)
    .map(|s| s.len() as u64)
    .unwrap_or(0),
```

---

## Pain Point 2: `video_post_processor.rs:141-167` — Fabricated Quality Scores

**Severity**: P1

**Location**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:140-167`

**What's wrong**: `_align_colors()` and `stabilize()` return hardcoded `_PostProcessResult` structs with fabricated scores (0.92, 0.88, 0.95, etc.) and claimed 150 processed frames — without doing any actual processing. Any downstream system consuming these scores (quality gates, GWT attention routing, KB telemetry) operates on lies.

**Impact**: Quality metrics in KB are meaningless; self-healing loops can't detect real degradation; user sees "success" when nothing happened.

**Concrete fix (code sketch)**:

```rust
pub fn _align_colors(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    let output_path = format!("{}_color_aligned.mp4", video_path);
    match std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "colorbalance=rs=-0.05", "-y", &output_path])
        .output()
    {
        Ok(o) if o.status.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output_path),
            processed_frames: 0, // unknown without ffprobe
            color_consistency_score: 0.0,
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Ok(o) => _PostProcessResult { success: false, error: Some(String::from_utf8_lossy(&o.stderr).to_string()), ..Default::default() },
        Err(e) => _PostProcessResult { success: false, error: Some(e.to_string()), ..Default::default() },
    }
}
```

Same pattern for `stabilize()` with ffmpeg `vidstabdetect`/`vidstabtransform`.

---

## Pain Point 3: `kb_cmds.rs:360-362` — `/kb embed` Is a Silent No-Op Stub

**Severity**: P1

**Location**: `neotrix-core/src/cli/commands/kb_cmds.rs:360-362`

**What's wrong**: `cmd_embed()` returns `"Embed command stub — embedding module not yet wired"` as a success message. Users calling `/kb embed` think embeddings are being created. The KB's BM25+vector hybrid search depends on embeddings being populated. The `nt_memory_embed` module already exists (used by `cmd_distill` at line 369).

**Impact**: KB vector search silently returns empty results for any user who ran `/kb embed` thinking it worked; knowledge retrieval quality degrades without visible error.

**Concrete fix (code sketch)**:

```rust
fn cmd_embed(args: &[String]) -> CommandOutput {
    use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_embed::load_all_embeddings;
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Batch embed all nodes missing vectors
    let embedded = match crate::l1_action::nt_memory::nt_memory_kb::nt_memory_embed::embed_missing(&conn) {
        Ok(n) => n,
        Err(e) => return CommandOutput::err(&format!("Embed 失败: {}", e)),
    };
    let msg = format!("✅ Embedded {} nodes", embedded);
    CommandOutput::ok(&msg)
}
```

---

## Summary

| # | File:Line | Issue | Severity | Category |
|---|-----------|-------|----------|----------|
| 1 | `checkpoint_persistence.rs:302` | cleanup_expired leaks files; file_size hardcoded 0 | P1 | Silent data leak |
| 2 | `video_post_processor.rs:141-167` | Fabricated quality scores returned as real | P1 | False telemetry |
| 3 | `kb_cmds.rs:360-362` | /kb embed is no-op stub disguised as success | P1 | Silent no-op |

All three follow the same anti-pattern: **function appears to work (returns Ok/success) but does nothing** — the most dangerous class of bug because it evades error handling, logging, and user awareness.
