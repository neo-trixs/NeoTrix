# Targeted Research #459 — Internal Pain Points (WISER Iteration 4)

**Date:** 2026-09-12
**Method:** Grep-based scan of `neotrix-core/src/` for TODO/FIXME stubs, hardcoded returns, and dead code paths.

---

## Pain Point 1: `seo.rs` — Entire Module is a Stub

**File:** `neotrix-core/src/l1_action/nt_act/actions/seo.rs:29-31`
**Severity:** P1

**What's wrong:** `SeoAnalyzer::analyze()` returns `Err("not yet implemented")`. The entire module is a "fresh bud" (C0) with zero production logic — registered in the capability tree but completely non-functional. Any user invoking `/seo analyze` gets a runtime error with no useful output.

**Fix:**
```rust
pub fn analyze(&self, content: &str) -> Result<String, String> {
    let word_count = content.split_whitespace().count();
    let keyword_density = if word_count > 0 {
        content.split_whitespace()
            .filter(|w| w.len() > 6)
            .count() as f64 / word_count as f64
    } else {
        0.0
    };
    let readability = if word_count > 100 { "Good" } else { "Too short" };
    Ok(format!(
        "SEO Analysis: {} words | keyword_density: {:.1}% | readability: {} | title: {}",
        word_count, keyword_density * 100.0, readability,
        if content.contains("<h1>") { "Found" } else { "Missing" }
    ))
}
```

---

## Pain Point 2: `cmd_consistency` Returns Empty String

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:156-159`
**Severity:** P1

**What's wrong:** `/kb consistency` command opens the DB, then returns `CommandOutput::ok("")` with zero output. The `setting_consistency` module is commented out (`module not found`). Users get a success response with no information — a silent no-op that wastes a DB connection.

**Fix:**
```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let mut out = String::from("=== KB Consistency Report ===\n");
    // Inline stub: check for orphaned nodes (edges pointing to non-existent nodes)
    let orphaned: Vec<String> = conn.prepare(
        "SELECT id FROM nodes WHERE id NOT IN (SELECT source_id FROM edges) LIMIT 10"
    ).map(|mut stmt| {
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap_or_default()
            .filter_map(|r| r.ok())
            .collect()
    }).unwrap_or_default();
    out.push_str(&format!("Orphaned nodes (sample): {}\n", orphaned.len()));
    CommandOutput::ok(&out)
}
```

---

## Pain Point 3: `parallel_task.rs` — Retry Delay Ignored

**File:** `neotrix-core/src/l1_action/nt_act/parallel_task.rs:273-276`
**Severity:** P1

**What's wrong:** Exponential backoff delay is computed (`_delay`) but never used — the task is immediately re-pushed to the queue. This means failed GPU tasks retry instantly, creating a tight retry loop that wastes resources and can overwhelm the scheduler.

**Fix:**
```rust
task.status = TaskStatus::Pending;
let delay = self.config.retry_interval_base_secs
    * (2u32.pow(task.current_retries).min(self.config.retry_max_multiplier));
// Schedule delayed re-queue instead of immediate push
let scheduled_at = Instant::now() + Duration::from_secs(delay as u64);
task.scheduled_at = Some(scheduled_at);
self.delayed_queue.push(DelayedTask {
    task,
    available_at: scheduled_at,
});
// Check delayed queue on each tick
```
