# Targeted Research #531 — Internal Pain Points (Iteration 28)

**Date**: 2026-09-13
**Status**: DRAFT
**Scope**: neotrix-core/src — incomplete implementations, stubs, dead code

---

## Pain Point 1: PTC stubs & exec are no-ops

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:353-359` and `:467-468`
**Severity**: P1

**What's wrong**: The `agent stubs` and `agent exec` commands both return `Vec::new()` because `McpRegistry.gateway()` is not implemented. The commands exist, accept arguments, run validation, but produce zero results silently. A user calling `/agent stubs` or `/agent exec` gets a success message with "0 typed signatures" — no indication this is a known gap.

**Fix sketch**:
```rust
// agent_cmds.rs:353 — stubs command
"stubs" => {
    let stubs: Vec<serde_json::Value> = Vec::new();
    if stubs.is_empty() {
        return CommandOutput::ok(
            "⚠ PTC stubs: gateway() not yet wired — 0 signatures returned.\n\
             See agent_cmds.rs:353. Implement McpRegistry::gateway() to activate."
        );
    }
    // ... existing render path
}

// agent_cmds.rs:467 — exec command
"exec" => {
    if plan.stages() == 0 {
        return CommandOutput::err(
            "PTC exec: gateway() not implemented — no calls can be dispatched.\n\
             Wire McpRegistry::gateway() in nt_agent_mcp_gateway."
        );
    }
    // ... existing dispatch path
}
```

---

## Pain Point 2: `/kb setting_consistency` returns empty string

**File**: `neotrix-core/src/cli/commands/kb_cmds.rs:156-159`
**Severity**: P2

**What's wrong**: The `setting_consistency` subcommand creates a String buffer, comments out the only logic (`check_and_report_to_string`), and returns the empty buffer as `CommandOutput::ok(&out)`. Every call succeeds with an empty string. The `setting_consistency` module is referenced but missing.

**Fix sketch**:
```rust
// kb_cmds.rs:156-159 — either implement or gate the command
fn cmd_setting_consistency(args: &[String], conn: &Connection) -> CommandOutput {
    match crate::l1_action::nt_memory::nt_memory_kb::setting_consistency
        ::check_and_report_to_string(conn)
    {
        Ok(report) if report.is_empty() => {
            CommandOutput::ok("✅ All KB settings are consistent.")
        }
        Ok(report) => CommandOutput::ok(&report),
        Err(e) => CommandOutput::err(
            &format!("setting_consistency check failed: {e}\n\
                      Module may need wiring: nt_memory_kb::setting_consistency")
        ),
    }
}
```

---

## Pain Point 3: `ResourceBudgetManager::estimate_cost` uses hardcoded price list

**File**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs:288-296`
**Severity**: P1

**What's wrong**: Cost estimation is hardcoded to 3 model names (`gpt-4`, `gpt-3.5-turbo`, `claude-3`) with fixed per-1k-token rates. Any model not in this list falls back to `$0.001/1k`. This is completely stale — GPT-4o, Claude 3.5 Sonnet, Gemini, Llama, DeepSeek, etc. are all missing. The TODO comment says "实际调用成本计算" but the code never reads a config or calls a pricing API. Cost budgets based on this will be wildly inaccurate.

**Fix sketch**:
```rust
// resource_budget.rs:283-296 — load pricing from KB or config
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    let cost_per_1k = self.pricing_table.get(model).copied()
        .unwrap_or_else(|| {
            log::warn!(
                "No pricing for model '{}'; falling back to $0.001/1k. \
                 Add to pricing_table or KB 'model_pricing' namespace.",
                model
            );
            0.001
        });
    (token_count as f64 / 1000.0) * cost_per_1k
}

// Builder should populate from KB:
pub fn with_pricing(mut self, kb: &KnowledgeBase) -> Self {
    if let Ok(entries) = kb.kv_list("model_pricing") {
        for (model, rate_str) in entries {
            if let Ok(rate) = rate_str.parse::<f64>() {
                self.pricing_table.insert(model, rate);
            }
        }
    }
    self
}
```
