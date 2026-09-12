# Targeted Research 455 — Internal Pain Points (Post-Iteration 454)

## P1: `video_stitcher.rs` — `stitch()` Silently Succeeds Without Doing Anything

**File:** `neotrix-core/src/l1_action/nt_act/actions/video_stitcher.rs:186-199`

**What's wrong:** `stitch()` always returns `success: true` and claims to have produced output, but never calls FFmpeg or any real stitching logic. The caller receives a `StitchResult` indicating success and trusts it, so downstream pipeline stages proceed on unstitched video.

**Severity:** P1 — Silent data loss. Downstream consumers (video pipeline, batch orchestrator) proceed as if stitching completed.

**Fix sketch:**
```rust
pub fn stitch(&mut self, timeline: &Timeline, output_path: &str) -> StitchResult {
    let start = std::time::Instant::now();
    let cmd = self.generate_ffmpeg_command(timeline, output_path);
    let status = std::process::Command::new("sh")
        .args(["-c", &cmd])
        .status();
    let (success, error) = match status {
        Ok(s) => (s.success(), if s.success() { None } else { Some(format!("ffmpeg exited {s}")) }),
        Err(e) => (false, Some(e.to_string())),
    };
    let result = StitchResult {
        success,
        output_path: if success { Some(output_path.to_string()) } else { None },
        total_duration: timeline.total_duration,
        processing_time_ms: start.elapsed().as_millis() as u64,
        error,
    };
    self.history.push(result.clone());
    result
}
```

---

## P2: `parallel_task.rs` — Exponential Backoff Delay Computed but Never Applied

**File:** `neotrix-core/src/l1_action/nt_act/parallel_task.rs:273-276`

**What's wrong:** The retry logic computes `_delay` with exponential backoff but immediately pushes the task back to the queue without sleeping. Retried tasks fire instantly, defeating rate-limiting and causing burst-load on already-failing resources.

**Severity:** P2 — Retry thundering herd. Tasks that fail due to rate limits will immediately retry and fail again.

**Fix sketch:**
```rust
let delay = self.config.retry_interval_base_secs
    * (2u32.pow(task.current_retries).min(self.config.retry_max_multiplier));
task.status = TaskStatus::Pending;
task.scheduled_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(delay as u64));
self.task_queue.push(task);
```
And in the dispatch loop, skip tasks where `scheduled_at > now()`.

---

## P1: `kb_cmds.rs` — `/kb consistency` Returns Empty String

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:150-160`

**What's wrong:** The `/kb consistency` command opens the KB connection, then returns `CommandOutput::ok("")` — an empty string. The `setting_consistency` module was removed and the stub silently swallows the call. Users invoking `/kb consistency` get an empty success response with zero information.

**Severity:** P1 — User-facing feature is dead. Commands that appear to work but return nothing waste user time and erode trust.

**Fix sketch:**
```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Inline basic consistency checks until setting_consistency module is restored
    let mut issues = Vec::new();
    // Check for orphan nodes (edges pointing to missing nodes)
    if let Ok(mut stmt) = conn.prepare("SELECT COUNT(*) FROM edges WHERE target NOT IN (SELECT id FROM nodes)") {
        if let Ok(count) = stmt.query_row([], |r| r.get::<_, i64>(0)) {
            if count > 0 {
                issues.push(format!("{} orphan edges (target missing)", count));
            }
        }
    }
    if issues.is_empty() {
        CommandOutput::ok("一致性检查通过: 无异常")
    } else {
        CommandOutput::ok(&format!("发现 {} 个问题:\n{}", issues.len(), issues.join("\n")))
    }
}
```
