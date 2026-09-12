# Targeted Research #450 — Internal Pain Point Identification

**Date:** 2026-09-12
**Method:** Codebase grep for TODO/FIXME/HACK, stub returns, dead code paths
**Scope:** `neotrix-core/src/` — production code (not tests)

## Summary

After 14 prior fixes (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB search, value_function, content_moderation, checkpoint_persistence, rate_limiter cleanup, temporal_continuity, verifier_agent, quality_control, unified_api, checkpoint cleanup, /kb embed), these 3 remaining pain points were identified:

| # | File | Line | Issue | Severity |
|---|------|------|-------|----------|
| P1 | `cli/commands/agent_cmds.rs` | 48-61 | **McpRegistry is a dead stub** — all methods return empty/None | **P0** |
| P2 | `l6_meta/coordination/layered_qa.rs` | 300-309 | **`execute_check` always returns `passed: true`** — QA layer is a no-op | **P1** |
| P3 | `cli/commands/kb_cmds.rs` | 151-159 | **`/kb consistency` returns empty string** — missing module integration | **P1** |

---

## Pain Point 1: McpRegistry Is a Dead Stub

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:48-61`
**Severity:** P0 (core CLI feature completely non-functional)

### What's Wrong

The `McpRegistry` struct at line 48 is a stub redefinition that shadows the real `McpRegistry` from `agent::tool::mcp`. All methods return empty results:

```rust
pub struct McpRegistry;  // unit struct — no data
impl McpRegistry {
    pub fn new() -> Self { Self }
    pub fn gateway(&self) -> Option<String> { None }  // always None
    pub fn list_tools(&self) -> Vec<McpToolInfo> { Vec::new() }  // always empty
    pub fn search(&self, _query: &str) -> Vec<McpToolInfo> { Vec::new() }
    pub fn tool_count(&self) -> usize { 0 }
    // ...
}
```

The real `McpRegistry` exists at `agent::tool::mcp` and is properly constructed in `entry/mod.rs:1437-1458` with `register_stdio()`, `as_native_tools()`, etc. But `get_mcp_registry()` at line 94 creates a **new stub** if the `OnceLock` isn't populated:

```rust
pub fn get_mcp_registry() -> Arc<RwLock<McpRegistry>> {
    MCP_REGISTRY.get().cloned()
        .unwrap_or_else(|| Arc::new(RwLock::new(McpRegistry::new())))
}
```

**Impact:** `/mcp stubs` always shows "0 typed signatures", `/mcp exec` always returns empty results, `/mcp search` finds nothing. The PTC (Programmatic Tool Calling) pipeline is dead on arrival.

### Fix

```rust
// agent_cmds.rs:48-61 — Replace stub McpRegistry with a thin wrapper
// that delegates to the real agent::tool::mcp::McpRegistry.
use crate::agent::tool::mcp::McpRegistry as RealMcpRegistry;

pub struct McpRegistry(Option<RealMcpRegistry>);
impl McpRegistry {
    pub fn new() -> Self { Self(Some(RealMcpRegistry::new())) }
    pub fn gateway(&self) -> Option<String> {
        self.0.as_ref().and_then(|r| r.gateway())
    }
    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        self.0.as_ref().map(|r| r.list_tools()).unwrap_or_default()
    }
    pub fn search(&self, query: &str) -> Vec<McpToolInfo> {
        self.0.as_ref().map(|r| r.search(query)).unwrap_or_default()
    }
    pub fn tool_count(&self) -> usize {
        self.0.as_ref().map(|r| r.tool_count()).unwrap_or(0)
    }
}
```

---

## Pain Point 2: `layered_qa::execute_check` Always Passes

**File:** `neotrix-core/src/l6_meta/coordination/layered_qa.rs:300-309`
**Severity:** P1 (QA layer is a no-op — every check passes unconditionally)

### What's Wrong

```rust
fn execute_check(&self, item: &_QACheckItem, _spec: &serde_json::Value, _output: &serde_json::Value) -> _QACheckResult {
    // TODO: 实际执行检查逻辑
    _QACheckResult {
        check_item_id: item.id.clone(),
        passed: true,  // ← ALWAYS TRUE regardless of input
        severity: IssueSeverity::Info,
        message: format!("检查 {} 通过", item.name),
        check_time_ms: 10,
    }
}
```

This means `_LayeredQAResult.passed` is always true, `_PublishDecision.publish` is always true. The entire 3-layer QA pipeline (AI → Human → Platform) silently approves everything.

### Fix

```rust
fn execute_check(&self, item: &_QACheckItem, spec: &serde_json::Value, output: &serde_json::Value) -> _QACheckResult {
    // Check required fields exist in output
    let passed = item.required_fields.iter().all(|f| {
        output.get(f).is_some() && !output.get(f).map_or(true, |v| v.is_null())
    });
    // Check content length minimums
    let content_ok = item.min_length.map_or(true, |min| {
        output.get("content").and_then(|c| c.as_str()).map_or(false, |s| s.len() >= min)
    }).unwrap_or(true);
    let overall = passed && content_ok;
    _QACheckResult {
        check_item_id: item.id.clone(),
        passed: overall,
        severity: if overall { IssueSeverity::Info } else { IssueSeverity::Warning },
        message: format!("检查 {}: {}", item.name, if overall { "通过" } else { "未通过 — 缺少必要字段" }),
        check_time_ms: 10,
    }
}
```

---

## Pain Point 3: `/kb consistency` Returns Empty String

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:151-159`
**Severity:** P1 (user-facing command silently succeeds with no output)

### What's Wrong

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let out = String::new();
    // TODO: setting_consistency module not found; stub
    // let _ = crate::l1_action::nt_memory::nt_memory_kb::setting_consistency::check_and_report_to_string(&conn, &mut out);
    CommandOutput::ok(&out)  // ← Returns empty string
}
```

The `setting_consistency` module doesn't exist in the codebase. Users get an empty success response.

### Fix

Replace with a basic consistency check using existing KB APIs — verify nodes have required fields, edges reference valid nodes, and embeddings match node count:

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let mut issues = Vec::new();
    // Check: orphan edges (edges referencing non-existent nodes)
    let orphan_count: u64 = conn.query_row(
        "SELECT COUNT(*) FROM edges e
         LEFT JOIN nodes n1 ON e.source = n1.id
         LEFT JOIN nodes n2 ON e.target = n2.id
         WHERE n1.id IS NULL OR n2.id IS NULL",
        [], |r| r.get(0),
    ).unwrap_or(0);
    if orphan_count > 0 {
        issues.push(format!("⚠ {} 条边引用了不存在的节点 (orphan edges)", orphan_count));
    }
    // Check: nodes without embeddings
    let no_embed: u64 = conn.query_row(
        "SELECT COUNT(*) FROM nodes n
         LEFT JOIN embeddings e ON n.id = e.node_id
         WHERE e.node_id IS NULL",
        [], |r| r.get(0),
    ).unwrap_or(0);
    if no_embed > 0 {
        issues.push(format!("⚠ {} 个节点缺少向量嵌入", no_embed));
    }
    if issues.is_empty() {
        CommandOutput::ok("✅ KB 一致性检查通过 — 无异常")
    } else {
        CommandOutput::ok(&format!("KB 一致性检查:\n{}", issues.join("\n")))
    }
}
```

---

## Priority Ranking

| Rank | Pain Point | Severity | Effort | Impact |
|------|-----------|----------|--------|--------|
| 1 | McpRegistry stub | P0 | Low | Unblocks PTC pipeline, /mcp exec, /mcp stubs |
| 2 | layered_qa always passes | P1 | Low | Makes QA layer functional |
| 3 | /kb consistency empty | P1 | Low | User-facing command works |

All three fixes are self-contained, low-risk, and can be verified independently.
