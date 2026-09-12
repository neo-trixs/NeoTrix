# Targeted Internal Pain Point Research #451

**Date:** 2026-09-12
**Method:** WISER (internal-first pain point scan)
**Scope:** neotrix-core/src/ — hardcoded returns, stubs, dead code

---

## Pain Point 1 — P0: Embedding Search Always Returns Empty

**File:** `neotrix-core/src/core/nt_core_bank/bank/search.rs:270`

**What's wrong:** `retrieve_relevant_by_embedding` hardcodes `cosine_similarity` to `0.0_f64`, then immediately filters `score > 0.0` — meaning embedding-based retrieval **always returns empty Vec**. The BM25 fallback path works, but any code path calling the embedding variant silently produces no results. This is the core retrieval function for the reasoning bank's vector search.

**Severity:** P0 — silent data loss; downstream RAG pipeline gets zero matches.

**Fix sketch:**
```rust
// search.rs:264-272 — replace hardcoded 0.0 with actual cosine similarity
.filter_map(|m| {
    m.embedding.as_ref().map(|emb| {
        let sim = cosine_similarity(task_embedding, emb);
        (sim, m)
    })
})
.filter(|(score, _)| *score > 0.0)
```
Add helper:
```rust
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}
```

---

## Pain Point 2 — P1: `/kb consistency` Is a Dead Stub (Module Exists But Commented Out)

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:157` + `neotrix-core/src/l1_action/nt_memory/nt_memory_kb/mod.rs:55`

**What's wrong:** The `setting_consistency` module (305 lines, full implementation of KB drift/conflict/duplicate detection) exists at `nt_memory_setting_consistency.rs` but is **commented out in mod.rs** (`// pub mod nt_memory_setting_consistency;`). The CLI command `cmd_consistency` opens the DB connection then returns an empty string. Root cause: `Severity` enum definition is missing (line 17 has `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` but no `enum Severity { ... }` block before `impl Severity`).

**Severity:** P1 — user runs `/kb consistency` and gets empty output, no feedback.

**Fix sketch:**
```rust
// nt_memory_setting_consistency.rs:17 — add missing enum definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str { ... }
}
```
Then uncomment in mod.rs:
```rust
pub mod nt_memory_setting_consistency;
```

---

## Pain Point 3 — P1: McpRegistry + ProgrammaticPlanner Are Entirely Stubbed

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:48-72`

**What's wrong:** `McpRegistry` is a struct with **every method returning empty/0/None/`Vec::new()`**. `McpDiscovery::scan_path()` returns empty. `ProgrammaticPlanner::plan()` returns `Err("not yet implemented")`. The real implementations live in `crate::l1_action::nt_io::nt_agent_mcp_gateway` (line 7 shows a commented-out import). This stub shadows the real types, so `/mcp list`, `/mcp search`, `/mcp stubs`, and PTC planning all silently do nothing.

**Severity:** P1 — MCP tooling and programmatic tool calling are dead.

**Fix sketch:**
```rust
// agent_cmds.rs:48 — delete the stub struct, use real import
// Remove lines 48-74 entirely. Uncomment line 7:
use crate::l1_action::nt_io::nt_agent_mcp_gateway::{ProgrammaticCall, ProgrammaticPlanner};
// And re-export the real McpRegistry from the gateway module.
```
If the gateway module doesn't export McpRegistry yet, add:
```rust
// In nt_agent_mcp_gateway mod.rs:
pub use McpRegistry;
```

---

## Summary

| # | Location | Severity | Issue | Files |
|---|----------|----------|-------|-------|
| 1 | `search.rs:270` | P0 | Embedding search hardcoded to 0.0, always returns empty | `bank/search.rs` |
| 2 | `kb_cmds.rs:157` + `mod.rs:55` | P1 | Setting consistency module commented out, missing Severity enum | `kb_cmds.rs`, `nt_memory_setting_consistency.rs`, `mod.rs` |
| 3 | `agent_cmds.rs:48-72` | P1 | McpRegistry/ProgrammaticPlanner stubs shadow real implementations | `agent_cmds.rs` |
