# Targeted Research 464 — Internal Pain Points

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — hardcoded values, dead stubs, incomplete implementations
**Method:** grep TODO/FIXME + static analysis of return paths

---

## Pain Point 1: PTC Stubs/Exec Always Returns Empty

**File:** `cli/commands/agent_cmds.rs:353-354` and `agent_cmds.rs:467-468`
**Severity:** P1 — user-facing feature silently broken

**What's wrong:** Both `mcp stubs` and `mcp exec` commands allocate `Vec::new()` and never populate it. The `FIXME: McpRegistry.gateway() not yet implemented` comment explains why — the gateway dispatch is missing. Users see "PTC stubs: 0 typed signatures" and "PTC exec: 0 call(s)" with no error.

**Impact:** PTC (Programmatic Tool Calling) is a documented capability but is a dead shell. Agent sessions attempting to use typed-stub tool calling get zero results with a success status.

**Fix sketch:**

```rust
// agent_cmds.rs — "stubs" branch (line 353)
"stubs" => {
    let gateway = match registry.gateway() {
        Some(g) => g,
        None => return CommandOutput::err("McpRegistry.gateway() not configured"),
    };
    let stubs: Vec<serde_json::Value> = gateway.tool_schemas()
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s)
            .with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
// "exec" branch (line 467) — same pattern, call gateway.execute(plan)
```

---

## Pain Point 2: Cost Estimator Uses 3-Model Hardcoded Table

**File:** `l1_action/nt_act/resource_budget.rs:288-296`
**Severity:** P2 — silent cost miscalculation for any model not in the 3-entry match

**What's wrong:** `estimate_cost()` matches only `"gpt-4"`, `"gpt-3.5-turbo"`, `"claude-3"` — all other models fall through to `$0.001/1k` (the `_ =>` arm). NeoTrix routes through 20+ providers with dozens of model IDs (e.g. `anthropic/claude-3.5-sonnet`, `google/gemini-1.5-pro`, `meta-llama/llama-3-70b`). The cost estimate for any non-GPT4/3.5/Claude3 model is ~30-300x understated.

**Impact:** Budget enforcement is unreliable. A 1M-token run on `gpt-4o` would report ~$0.10 instead of the real ~$2.50.

**Fix sketch:**

```rust
// resource_budget.rs — replace hardcoded match
pub fn estimate_cost(&self, token_count: u64, model: &str, is_output: bool) -> f64 {
    let (input_per_m, output_per_m) = MODEL_PRICING
        .get(model)
        .copied()
        .unwrap_or((1.0, 1.0)); // fallback: $1/M tokens
    let per_m = if is_output { output_per_m } else { input_per_m };
    (token_count as f64 / 1_000_000.0) * per_m
}

// Static pricing table (extract from eval harness ModelSpec.pricing_per_1m_*)
lazy_static! {
    static ref MODEL_PRICING: HashMap<&'static str, (f64, f64)> = HashMap::from([
        ("gpt-4",                     (30.0,  60.0)),
        ("gpt-4o",                    (2.50,  10.0)),
        ("gpt-4o-mini",               (0.15,  0.60)),
        ("gpt-3.5-turbo",             (0.50,  1.50)),
        ("claude-3-opus",             (15.0,  75.0)),
        ("claude-3-sonnet",           (3.0,   15.0)),
        ("claude-3-haiku",            (0.25,  1.25)),
        ("claude-3-5-sonnet",         (3.0,   15.0)),
        ("gemini-1.5-pro",            (1.25,  5.0)),
        ("gemini-1.5-flash",          (0.075, 0.30)),
        ("llama-3-70b",               (0.88,  0.88)),
    ]);
}
```

---

## Pain Point 3: `/kb consistency` Command Is a Silent No-Op Stub

**File:** `cli/commands/kb_cmds.rs:157-159`
**Severity:** P1 — user calls command, gets empty success, nothing happens

**What's wrong:** `cmd_setting_consistency()` opens the DB connection, discards it, builds an empty `String::new()`, and returns `CommandOutput::ok(&out)` — i.e. success with no output. The commented-out line `// let _ = setting_consistency::check_and_report_to_string(...)` shows the real logic was removed when the module was deleted.

**Impact:** Users calling `/kb consistency` believe their KB is consistent when in reality no check ran. Silent failure with false-positive success is worse than an error.

**Fix sketch:**

```rust
// kb_cmds.rs — replace stub
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Run inline consistency checks
    let mut issues = Vec::new();
    // Check: orphan nodes (no edges)
    let orphan_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM nodes WHERE id NOT IN (SELECT source FROM edges UNION SELECT target FROM edges)", [], |r| r.get(0))
        .unwrap_or(0);
    if orphan_count > 0 {
        issues.push(format!("⚠ {} orphan nodes (no edges)", orphan_count));
    }
    // Check: orphan embeddings
    let orphan_emb: i64 = conn
        .query_row("SELECT COUNT(*) FROM embeddings WHERE node_id NOT IN (SELECT id FROM nodes)", [], |r| r.get(0))
        .unwrap_or(0);
    if orphan_emb > 0 {
        issues.push(format!("⚠ {} orphan embeddings", orphan_emb));
    }
    if issues.is_empty() {
        CommandOutput::ok("✅ KB consistency: no issues found")
    } else {
        CommandOutput::ok(&format!("KB consistency report:\n{}", issues.join("\n")))
    }
}
```

---

## Summary

| # | File | Severity | Type | Impact |
|---|------|----------|------|--------|
| 1 | `agent_cmds.rs:353,467` | P1 | Dead code stub | PTC feature returns empty with success status |
| 2 | `resource_budget.rs:288` | P2 | Hardcoded values | Cost estimation 30-300x off for 90%+ of models |
| 3 | `kb_cmds.rs:157` | P1 | Silent no-op | KB consistency check silently passes without running |
