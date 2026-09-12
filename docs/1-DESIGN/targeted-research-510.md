# Targeted Research #510 — Internal Pain Points

> WISER iteration loop: 3 pain points from `neotrix-core/src/`
> Date: 2026-09-13

---

## P1: `/kb consistency` is a Complete Stub (Empty Return)

**File:** `cli/commands/kb_cmds.rs:151-159`

**What's wrong:** `cmd_consistency()` opens the DB, creates an empty `String`, then returns `CommandOutput::ok("")`. The actual consistency check (`setting_consistency::check_and_report_to_string`) is commented out with a TODO saying the module was never found. Users running `/kb consistency` get a silent success with zero output — no error, no warning, no data.

**Severity:** P1 — CLI feature advertised in help text that silently does nothing.

**Fix sketch:**

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Query KB nodes for setting-type entries and check for contradictions
    let nodes: Vec<(String, String, String)> = conn
        .prepare("SELECT id, title, content FROM nodes WHERE type = 'setting'")
        .and_then(|mut stmt| {
            let rows = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
        })
        .unwrap_or_default();

    if nodes.is_empty() {
        return CommandOutput::ok("无 setting 类型节点, 无需一致性检查");
    }

    let mut contradictions = Vec::new();
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if text_jaccard(&nodes[i].2, &nodes[j].2) > 0.7
                && nodes[i].2 != nodes[j].2
            {
                contradictions.push(format!(
                    "'{}' 与 '{}' 内容高度相似但不同 (jaccard=低)",
                    nodes[i].1, nodes[j].1
                ));
            }
        }
    }
    if contradictions.is_empty() {
        CommandOutput::ok(&format!("检查完成: {} 个 setting 节点, 无矛盾", nodes.len()))
    } else {
        CommandOutput::ok(&format!(
            "发现 {} 处潜在矛盾:\n{}",
            contradictions.len(),
            contradictions.join("\n")
        ))
    }
}
```

---

## P1: ANE Direct Dispatch is a No-Op Passthrough

**File:** `core/l0_substrate/nt_core_deploy.rs:1200-1203`

**What's wrong:** `AneDirectProgram::dispatch()` returns `input.to_vec()` — a pure identity function. The `dispatch_time_us` field is hardcoded to `0.0` in `compile_program()`. Any code path calling `dispatch()` gets unmodified input with no latency measurement, making the entire ANE direct program struct a hollow shell.

**Severity:** P1 — Core inference path that silently skips all computation. If anything depends on ANE dispatch results, it gets wrong answers.

**Fix sketch:**

```rust
/// Dispatch input through the ANE program — at minimum, log that dispatch
/// was attempted and measure elapsed time. When real ANE backend is unavailable,
/// at least validate shapes and apply a passthrough with timing.
pub fn dispatch(&self, input: &[f32]) -> Vec<f32> {
    let start = std::time::Instant::now();
    // TODO(nt-physical): Wire real ANE dispatch when available.
    // For now, validate input is non-empty and measure passthrough time.
    assert!(!input.is_empty(), "ANE dispatch: empty input");
    let output = input.to_vec();
    let elapsed = start.elapsed().as_secs_f64() * 1e6; // microseconds
    tracing::debug!(
        program_id = %self.program_id,
        input_len = input.len(),
        elapsed_us = elapsed,
        "ANE direct dispatch (passthrough)"
    );
    output
}
```

---

## P2: Three Deprecated Registries Still Actively Used (fusion-plan-215 Never Executed)

**Files:**
- `core/nt_core_echo_terminal.rs:406-428` — `EchoPrmBridge` (deprecated, 748 lines)
- `core/nt_core_subagent.rs:278-300` — `SubAgentRegistry` (deprecated, 959 lines)
- `core/nt_core_reasoning.rs:70-110` — `MethodRegistry` (deprecated, 438 lines)

**What's wrong:** All three structs are marked deprecated via `Once::call_once` warnings pointing to `fusion-plan-215`. They're still actively instantiated in:
- `agent_cmds.rs:145` uses `SubAgentRegistry`
- `nt_core_orch_agent.rs:1089,1120,1159,1160` uses `SubAgentRegistry`
- `nt_core_reasoning.rs:393` exports `default_method_registry()`
- `EchoPrmBridge` is wired in `echo_terminal.rs` default impl

Each carries its own `Once` deprecation guard that fires on first construction. Total dead weight: ~2,146 lines across 3 files that log warnings but remain fully operational.

**Severity:** P2 — Not broken, but accumulated deprecation debt that inflates compile time and causes noisy logs on every cold start.

**Fix sketch — Step 1 (bridge, not removal):** Add delegation methods so callers can migrate incrementally:

```rust
// In nt_core_subagent.rs — add thin wrapper that delegates to CapabilityRegistry
pub fn migrate_to_capability_registry(registry: &mut CapabilityRegistry) {
    // SubAgentRegistry agents become Capability entries with kind = Agent
    let old = std::mem::take(&mut SubAgentRegistry::default().agents);
    for (id, def) in old {
        registry.register(Capability {
            id,
            kind: CapabilityKind::Agent,
            source: CapabilitySource::Local,
            constellation: Constellation::C0,
            ..Default::default()
        });
    }
}

// In nt_core_reasoning.rs — delegate to ReasoningStrategyRegistry
pub fn migrate_to_strategy_registry(registry: &mut ReasoningStrategyRegistry) {
    let old = MethodRegistry::new();
    for (method, spec) in old.method_map {
        registry.register(ReasoningStrategy {
            kind: method_to_strategy_kind(method),
            stage_range: spec.stage_range,
            complexity_ceiling: spec.complexity_ceiling,
            ..Default::default()
        });
    }
}
```

**Fix sketch — Step 2 (remove after migration):** `#[deprecated]` → `cfg(test)` gate → delete after callers migrated.

---

## Summary

| # | Pain Point | Severity | Location | Lines Affected |
|---|-----------|----------|----------|----------------|
| 1 | `/kb consistency` silent stub | P1 | `kb_cmds.rs:151` | ~10 |
| 2 | ANE dispatch no-op | P1 | `nt_core_deploy.rs:1200` | ~5 |
| 3 | fusion-plan-215 debt | P2 | 3 files, 2146 lines total | ~30 migration |

**Total fix effort:** ~45 lines of code + deprecation wiring.
