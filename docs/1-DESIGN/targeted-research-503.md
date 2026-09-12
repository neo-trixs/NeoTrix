# Targeted Research 503 — Internal Pain Points (Cycle 503)

**Date**: 2026-09-12
**Method**: Grep for TODO/FIXME/HACK, empty return values, hardcoded stubs
**Scope**: `neotrix-core/src/` (100+ TODO matches, 22 hardcoded returns, 100+ stubs)

---

## Pain Point 1: OSINT Fallback Backends Silently Return Zero Findings

**File**: `neotrix-core/src/l2_perception/nt_world/osint/backend_router.rs:224-244`
**Severity**: P1

**What's wrong**: The fallback backend structs (`SimpleDnsResult`, `SimpleHttpResult`, `SimpleFofaResult`) always return `findings_count() -> 0` and no data via `as_dns()`/`as_http()`. When the native backend fails (requires API keys for Shodan/Fofa, or DNS resolution fails), the router selects the fallback — but the fallback produces an empty result. The caller sees "0 findings" with no indication the fallback was a no-op placeholder.

**Impact**: OSINT reconnaissance silently produces empty results when the primary backend is unavailable. Users get a false sense of "nothing found" instead of "backend unavailable, please configure API key."

```rust
// FIX: Make fallback backends error instead of silently returning empty results
// In BackendRouter::probe_and_select, replace Ok(SimpleXxxResult) with Err:

#[derive(Debug)]
struct SimpleDnsResult;
impl _BackendResult for SimpleDnsResult {
    fn findings_count(&self) -> usize { 0 }
    // REMOVE this impl entirely — the fallback should never succeed
}

// In default_dns_backends(), change the fallback probe:
Backend {
    name: "dns-fallback".into(),
    probe: |_target, _client| Box::pin(async move {
        // Return Err so probe_and_select skips this and reports "all failed"
        Err(_BackendError::Unavailable(
            "dns-fallback: no-op stub; configure native backend".into()
        ))
    }),
},
// Apply same pattern to http-fallback and fofa-fallback
```

---

## Pain Point 2: `/kb consistency` Command Returns Empty String

**File**: `neotrix-core/src/cli/commands/kb_cmds.rs:150-160`
**Severity**: P1

**What's wrong**: `cmd_consistency` opens the KB, creates an empty string `out`, then returns `CommandOutput::ok(&out)` — always an empty success message. The actual consistency check is commented out with `// TODO: setting_consistency module not found; stub`. Users calling `/kb consistency` think the check passed, when it was never run.

**Impact**: KB consistency violations go undetected. This is the equivalent of a doctor saying "you're healthy" without examining you.

```rust
// FIX: Either implement the check or return an explicit "not implemented" error
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Option A: Return explicit "not yet implemented" (preferred)
    CommandOutput::ok("⚠️ consistency check not yet implemented (setting_consistency module missing). Run /kb doctor instead.")
    // Option B: Wire up real check once module exists:
    // let mut out = String::new();
    // setting_consistency::check_and_report_to_string(&conn, &mut out);
    // CommandOutput::ok(&out)
}
```

---

## Pain Point 3: `/mcp stubs` Always Shows "0 typed signatures"

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:350-359`
**Severity**: P1

**What's wrong**: The `/mcp stubs` subcommand (PTC — Programmatic Tool Calling) creates an empty `Vec::new()` for stubs and displays "0 typed signatures". The `FIXME: McpRegistry.gateway() not yet implemented` comment confirms this is dead code. Any agent or user calling `/mcp stubs` gets a misleading success response showing zero tools.

**Impact**: PTC (Programmatic Tool Calling) — one of the absorbed terminologies from the 22-source batch — is documented as a capability but returns nothing. Agent tool-routing cannot discover available tool signatures.

```rust
// FIX: Wire stubs to the actual McpRegistry tool list
"stubs" => {
    let registry = get_mcp_registry();
    let registry = registry.blocking_read();
    let stubs: Vec<serde_json::Value> = registry
        .list_all_tools()
        .iter()
        .map(|t| serde_json::json!({
            "name": t.name,
            "server": t.server_name,
            "signature": format!("{}({})", t.name, t.input_schema.get("properties")
                .map(|p| p.as_object()
                    .map(|m| m.keys().cloned().collect::<Vec<_>>().join(", "))
                    .unwrap_or_default())
                .unwrap_or_default()),
        }))
        .collect();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s).with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
```

---

## Summary

| # | File:Line | Issue | Severity | Fix Complexity |
|---|-----------|-------|----------|----------------|
| 1 | `osint/backend_router.rs:224-244` | Fallback backends silently return 0 findings | P1 | Low — change probe to Err |
| 2 | `cli/commands/kb_cmds.rs:150-160` | `/kb consistency` always returns empty | P1 | Low — return "not implemented" or wire up |
| 3 | `cli/commands/agent_cmds.rs:350-359` | `/mcp stubs` always shows 0 tools | P1 | Medium — wire to McpRegistry |

All three are **P1** — they produce misleading success responses when the underlying feature is non-functional. Users cannot distinguish "feature works, nothing found" from "feature is a dead stub."
