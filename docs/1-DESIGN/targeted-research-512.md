# Targeted Research #512 — Internal Pain Points

## Pain Point 1: `/kb consistency` is a silent no-op stub

**Location:** `neotrix-core/src/cli/commands/kb_cmds.rs:150-159`

**What's wrong:** The `/kb consistency` command opens the KB connection, creates an empty string, then returns it. The actual consistency check (`setting_consistency::check_and_report_to_string`) is commented out with `TODO: setting_consistency module not found; stub`. Users invoke this command and receive an empty "OK" response, silently believing their KB is consistent when no check ran at all.

**Severity:** P0 — Silent data corruption risk. KB setting inconsistencies (orphan nodes, stale edges, version drift) go undetected.

**Fix sketch:**

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Inline basic consistency checks instead of relying on missing module
    let mut issues = Vec::new();
    // 1. Check for orphan nodes (edges referencing missing nodes)
    // 2. Check for duplicate namespace keys
    // 3. Check embedding dimension consistency
    let report = if issues.is_empty() {
        "✅ KB consistency: no issues found".to_string()
    } else {
        format!("⚠️ KB consistency: {} issue(s)\n{}", issues.len(),
            issues.iter().enumerate().map(|(i, s)| format!("  {}. {}", i+1, s)).collect::<Vec<_>>().join("\n"))
    };
    CommandOutput::ok(&report)
}
```

---

## Pain Point 2: PTC `stubs` and `exec` always return empty results

**Location:** `neotrix-core/src/cli/commands/agent_cmds.rs:350-377` (stubs) and `:460-477` (exec)

**What's wrong:** Both `mcp stubs` (render typed Python signatures) and `mcp exec` (execute governed tool calls) are wired to the CLI but `McpRegistry.gateway()` is unimplemented (`FIXME`). Both commands allocate empty `Vec::new()` and return zero-count results. The PTC (Programmatic Tool Calling) feature — a core differentiator advertised in CONTEXT.md — is completely non-functional. The validation gate runs (via `ProgrammaticPlanner::plan`) but the actual execution is dead.

**Severity:** P0 — Feature advertised but produces no output. Users who discover this via CLI will trust NeoTrix less.

**Fix sketch:**

```rust
// In "stubs" handler:
"stubs" => {
    let registry = match McpRegistry::global() {
        Some(r) => r,
        None => return CommandOutput::err("MCP registry not initialized"),
    };
    let stubs: Vec<serde_json::Value> = registry.tool_schemas()
        .iter()
        .map(|schema| schema.to_python_stub())
        .collect();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s)
            .with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
```

---

## Pain Point 3: `estimate_cost` uses stale hardcoded model prices

**Location:** `neotrix-core/src/l1_action/nt_act/resource_budget.rs:283-297`

**What's wrong:** The cost estimation function only knows 3 models (`gpt-4`, `gpt-3.5-turbo`, `claude-3`) with hardcoded per-1K-token rates. NeoTrix's IO layer supports dozens of providers (Ollama, Gemini, DeepSeek, local models, etc.). Any model not in the match falls through to a generic `$0.001/1K` rate, producing wildly inaccurate budgets. The TODO comment says "实际调用成本计算" — this was never implemented.

**Severity:** P1 — Cost tracking is a core axiom (A1: Cost-Aware Routing). Budget decisions based on this function will be systematically wrong for non-OpenAI models.

**Fix sketch:**

```rust
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // Query provider config for actual pricing, fall back to heuristics
    let cost_per_1k = Self::lookup_model_cost(model).unwrap_or_else(|| {
        // Heuristic: larger models cost more, Ollama/local = 0
        if model.contains("ollama") || model.contains("local") {
            return 0.0;
        }
        match model {
            m if m.contains("gpt-4o") => 0.005,
            m if m.contains("gpt-4") => 0.03,
            m if m.contains("gpt-3.5") || m.contains("gpt-4o-mini") => 0.0015,
            m if m.contains("claude-3-opus") => 0.075,
            m if m.contains("claude-3-sonnet") => 0.015,
            m if m.contains("claude") => 0.008,
            m if m.contains("deepseek") => 0.002,
            m if m.contains("gemini-flash") => 0.00075,
            m if m.contains("gemini") => 0.005,
            _ => 0.01,
        }
    });
    (token_count as f64 / 1000.0) * cost_per_1k
}
```
