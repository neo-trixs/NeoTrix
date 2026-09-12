# Targeted Internal Pain Points — Round 446

## Methodology

WISER approach: internal-first pain point identification. Scanned `neotrix-core/src/` for TODO stubs, hardcoded returns, and dead code paths that silently break production behavior.

---

## Pain Point #1 — Content Moderation Output Risk Is Always 0.1 (Silent Security Bypass)

**File:** `neotrix-core/src/l3_embodiment/nt_shield/content_moderation.rs:243-246`

**What's wrong:** `evaluate_output_risk()` unconditionally returns `0.1` regardless of content. This means the entire output moderation pipeline is bypassed — generated images/videos/text that violate NSFW, violence, or brand safety policies will always score "low risk" and pass through. The input prompt moderation works (uses keyword matching), but the output moderation is a no-op.

**Severity:** **P0** — Security bypass. Any generated content that should be blocked will pass through unchecked.

**Fix sketch:**

```rust
fn evaluate_output_risk(&self, content_type: ContentType, metadata: &HashMap<String, String>, category: &RiskCategory) -> f64 {
    let content_ref = metadata.get("content_ref").map(|s| s.as_str()).unwrap_or("");
    if content_ref.is_empty() { return 0.1; }

    let mut score = 0.1_f64;
    match content_type {
        ContentType::GeneratedImage | ContentType::GeneratedVideo => {
            if let Some(aesthetic) = metadata.get("nsfw_score") {
                if let Ok(v) = aesthetic.parse::<f64>() { score = score.max(v); }
            }
            if let Some(labels) = metadata.get("safety_labels") {
                if labels.contains("violence") || labels.contains("explicit") { score = score.max(0.85); }
            }
        }
        ContentType::GeneratedText => {
            let text_lower = content_ref.to_lowercase();
            if category == &RiskCategory::NSFW && (text_lower.contains("nude") || text_lower.contains("porn")) {
                score = 0.9;
            }
        }
        _ => {}
    }
    score
}
```

---

## Pain Point #2 — Checkpoint Persistence Save/Load/Delete Are All No-Ops (Lost State on Restart)

**File:** `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs:147-244`

**What's wrong:** `save_checkpoint()` only inserts into an in-memory HashMap (line 159), `load_checkpoint()` constructs empty `CheckpointData` (line 176-181), and `delete_checkpoint()` only removes from the HashMap (line 212). On process restart, all checkpoint state is lost. The `file_size` field is hardcoded to `0` (line 147). This makes the `ProductionOrchestrator`'s fault tolerance promise entirely fake.

**Severity:** **P0** — Data loss. Long-running workflows lose all progress on crash/restart.

**Fix sketch:**

```rust
// In save_checkpoint — persist to disk:
let checkpoint_dir = self.config.base_dir.join(&checkpoint_id);
std::fs::create_dir_all(&checkpoint_dir)?;
let data_path = checkpoint_dir.join("checkpoint.json");
let json = serde_json::to_vec_pretty(&data)?;
std::fs::write(&data_path, &json)?;
// Compute file_size from serialized bytes
let mut meta = meta;
meta.file_size = json.len() as u64;

// In load_checkpoint — read from disk:
let data_path = self.config.base_dir.join(checkpoint_id).join("checkpoint.json");
if data_path.exists() {
    let bytes = std::fs::read(&data_path)?;
    let data: CheckpointData = serde_json::from_slice(&bytes)?;
    self.current = Some(data);
}

// In delete_checkpoint — remove files:
let checkpoint_dir = self.config.base_dir.join(checkpoint_id);
if checkpoint_dir.exists() { std::fs::remove_dir_all(&checkpoint_dir)?; }
```

---

## Pain Point #3 — Rate Limiter Cleanup Is Empty (Unbounded Memory Growth)

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_rate_limiter.rs:232-234`

**What's wrong:** `cleanup()` accepts a `max_age: Duration` parameter but does nothing. The rate limiter accumulates entries per API key/IP indefinitely. In long-running sessions or production daemons, this is an unbounded memory leak — each API call creates an entry that is never evicted.

**Severity:** **P1** — Memory leak in long-running processes. Slow degradation over hours/days.

**Fix sketch:**

```rust
pub fn cleanup(&mut self, max_age: Duration) {
    let cutoff = Instant::now() - max_age;
    let stale_keys: Vec<String> = self.limiters.iter()
        .filter(|(_, entry)| entry.last_access < cutoff && entry.count == 0)
        .map(|(k, _)| k.clone())
        .collect();
    for key in &stale_keys {
        self.limiters.remove(key);
    }
    self.stats.evictions += stale_keys.len() as u64;
}
```

---

## Summary

| # | File:Line | Severity | Issue |
|---|-----------|----------|-------|
| 1 | `content_moderation.rs:243` | P0 | Output risk always 0.1 — security bypass |
| 2 | `checkpoint_persistence.rs:147-244` | P0 | Save/load/delete are in-memory only — data loss |
| 3 | `nt_act_rate_limiter.rs:232` | P1 | Cleanup is empty — unbounded memory growth |

## Search Statistics

- TODO/FIXME/HACK count in `neotrix-core/src/`: **~75** hits
- Hardcoded stub functions found: **12+** (content moderation, checkpoint, rate limiter, cache, cost tracker, scheduler)
- Previous rounds fixed: 5 issues (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB cosine sim, SelfModel value_function)
