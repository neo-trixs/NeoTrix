# Targeted Research #513 — WISER Pain Point Scan

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — TODO/FIXME stubs, hardcoded returns, dead paths
**Method:** grep TODO/FIXME + unimplemented!/todo! + stub returns, then context-read top hits

---

## Pain Point 1 — PTC stubs: McpRegistry.gateway() returns empty

**Location:** `cli/commands/agent_cmds.rs:353-354` and `:467-468`

**What's wrong:** Both the `/mcp stubs` and `/mcp exec` commands build a plan then return `Vec::new()` because `McpRegistry.gateway()` is not implemented. The entire Programmatic Tool Calling (PTC) subsystem is dead — users get "0 typed signatures" and "0 call(s)" regardless of registered servers.

**Severity:** P1 — PTC is a core NT-ACT capability; the code compiles but produces no useful output.

**Fix sketch:**

```rust
// agent_cmds.rs:353-358 — replace stub with actual gateway call
let gateway = registry.gateway().await.map_err(|e| {
    CommandOutput::err(&format!("[ptc] gateway unavailable: {}", e))
})?;
let stubs: Vec<ProgrammaticStub> = gateway.list_stubs().await
    .unwrap_or_default();
```

---

## Pain Point 2 — KB consistency check is a no-op stub

**Location:** `cli/commands/kb_cmds.rs:150-159`

**What's wrong:** `cmd_consistency` opens the KB connection, allocates an empty string, prints a TODO comment about `setting_consistency` module, and returns `CommandOutput::ok("")`. The user runs `/kb consistency` and gets a blank success — no actual consistency checking happens.

**Severity:** P1 — Setting consistency is a core KB self-audit feature (对标网文每卷设定检查). Users trust the command returns real results.

**Fix sketch:**

```rust
// kb_cmds.rs:156-159 — replace empty stub with actual KB scan
let mut out = String::from("=== 设定一致性报告 ===\n");
let nodes = conn.prepare("SELECT id, namespace, key FROM kv_store")
    .and_then(|mut s| { /* iterate and check for duplicate keys */ });
// If setting_consistency module is missing, implement a basic
// duplicate-key and orphan-edge check inline, or gate the command
// behind a clear "not yet implemented" error instead of silent empty.
```

---

## Pain Point 3 — VerifierAgent uses simulate_verification instead of VLM

**Location:** `l6_meta/coordination/verifier_agent.rs:195-217` + `:222-257`

**What's wrong:** `_verify_shot` calls `simulate_verification` which is pure keyword-matching heuristic (checks for "character", "lighting", "action" in description text). It never calls an actual VLM model. The verification scores are deterministic based on string matching, so the quality gate is meaningless — it always passes for reasonable descriptions and fails only for short ones.

**Severity:** P1 — This is the NT-META quality control pipeline. Fake verification creates false confidence in generated content.

**Fix sketch:**

```rust
// verifier_agent.rs:195-196 — route through LLM capability
fn _verify_shot(&mut self, shot_id: &str, video_path: &str,
                spec: &str, ctx: Option<&str>) -> VerificationResult {
    let llm = self.llm_provider.as_ref()
        .expect("VerifierAgent requires an LLM provider");
    let prompt = format!("Verify video shot {} against spec: {}", shot_id, spec);
    let response = llm.complete(&prompt).await;  // actual VLM call
    // Parse response into VerificationScore[]
    // Fall back to simulate_verification only if LLM unavailable
}
```

---

## Summary

| # | File:Line | Issue | Severity |
|---|-----------|-------|----------|
| 1 | `agent_cmds.rs:353,467` | PTC stubs/exec return empty Vec — gateway() unimplemented | P1 |
| 2 | `kb_cmds.rs:150-159` | `/kb consistency` returns empty string — pure stub | P1 |
| 3 | `verifier_agent.rs:195-257` | VLM verification replaced by keyword heuristic | P1 |

All three are **P1** because they compile, appear functional to users, but silently produce no real results — the most dangerous category of defect.
