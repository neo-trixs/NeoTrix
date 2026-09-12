# Targeted Research 509 — Pain Point Analysis

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ internal pain points (WISER iteration loop)

---

## Pain Point 1: `ProductionOrchestrator` Checkpoint is a No-Op

**File:** `l1_action/nt_act/actions/orchestration/production_orchestrator.rs:224-243`
**Severity:** P1

**What's wrong:** `save_checkpoint()` and `restore_from_checkpoint()` accept the workflow ID, do a lookup, then silently succeed without persisting anything. `save_checkpoint` ignores the workflow data (`let _ = workflow;`), and `restore` hardcodes `WorkflowStatus::Paused` regardless of actual state. If the process crashes mid-workflow, all progress is lost — the "production orchestrator" provides zero durability.

**Why it matters:** This is the backbone of `ProductionOrchestrator` (formerly `BatchProductionManager`). Users expect checkpoint/resume for long-running batch jobs. Without it, any failure forces full re-execution.

**Fix sketch:**

```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or_else(|| "工作流不存在".to_string())?;
    let checkpoint = serde_json::to_vec(workflow)
        .map_err(|e| format!("序列化失败: {e}"))?;
    let path = self.config.checkpoint_dir.join(format!("{workflow_id}.json"));
    std::fs::create_dir_all(&self.config.checkpoint_dir)
        .map_err(|e| format!("创建目录失败: {e}"))?;
    std::fs::write(&path, checkpoint)
        .map_err(|e| format!("写入检查点失败: {e}"))?;
    Ok(())
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = self.config.checkpoint_dir.join(format!("{workflow_id}.json"));
    let data = std::fs::read(&path)
        .map_err(|e| format!("读取检查点失败: {e}"))?;
    let workflow: Workflow = serde_json::from_slice(&data)
        .map_err(|e| format!("反序列化失败: {e}"))?;
    self.workflows.insert(workflow_id.to_string(), workflow);
    Ok(())
}
```

---

## Pain Point 2: `StyleHarmonizer` Returns Fabricated Metrics

**File:** `l5_cognition/nt_core/visual/style_harmonizer.rs:112-165`
**Severity:** P1

**What's wrong:** `_analyze_style()`, `_harmonize()`, and `_match_colors()` are the core API surface of `StyleHarmonizer`, yet all three return hardcoded fake data. `_analyze_style` always returns `quality_score: 0.85` and `style_similarity: 0.88`/`0.92`. The `success: true` in harmonize is always true. Downstream consumers (e.g., `VideoPostProcessor`, `VisualConsistencyManager`) cannot distinguish a real result from a stub — they silently propagate fabricated quality scores.

**Why it matters:** Any pipeline using `StyleHarmonizer` gets false confidence in output quality. Quality gates that depend on these scores become meaningless.

**Fix sketch:**

```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> Result<_StyleAnalysis, StyleError> {
    // Delegate to image analysis backend (e.g., color histogram, edge detection)
    let img = image::open(image_path).map_err(|e| StyleError::Io(e.to_string()))?;
    let features = extract_style_features(&img)?;
    Ok(_StyleAnalysis {
        features,
        dominant_colors: extract_dominant_colors(&img, 3)?,
        style_tags: classify_style(&features)?,
        quality_score: compute_quality_score(&img),
    })
}

pub(crate) fn _harmonize(&mut self, input_path: &str, reference_path: Option<&str>) -> Result<_StyleHarmonizationResult, StyleError> {
    let start = Instant::now();
    let reference = reference_path
        .map(|p| image::open(p))
        .transpose()
        .map_err(|e| StyleError::Io(e.to_string()))?;
    let input = image::open(input_path).map_err(|e| StyleError::Io(e.to_string()))?;
    let output = apply_color_transfer(&input, reference.as_ref())?;
    let out_path = format!("{}_harmonized.png", input_path);
    output.save(&out_path).map_err(|e| StyleError::Io(e.to_string()))?;
    Ok(_StyleHarmonizationResult {
        success: true,
        output_path: Some(out_path),
        style_similarity: compute_similarity(&input, &output),
        processing_time_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}
```

---

## Pain Point 3: `McpRegistry.gateway()` Never Implemented — PTC Stubs Always Empty

**File:** `cli/commands/agent_cmds.rs:353-359`
**Severity:** P2

**What's wrong:** The `mcp stubs` subcommand is meant to render Python typed-stub signatures for programmatic tool calling (PTC). However, `McpRegistry.gateway()` is not implemented, so `stubs` is hardcoded to `Vec::new()`. The command always reports `0 typed signatures`. This is a documented FIXME since the PTC feature was introduced.

**Why it matters:** PTC is a core feature for agent tool-calling efficiency (typed stubs enable parallel/chain calls in a single turn). The command exists but produces nothing — a dead entry point that misleads users into thinking no MCP servers are registered.

**Fix sketch:**

```rust
"stubs" => {
    let gateway = McpRegistry::global().gateway()
        .ok_or_else(|| CommandOutput::error("No MCP gateway registered. Run `mcp scan` first."))?;
    let stubs: Vec<serde_json::Value> = gateway.tools().iter().map(|tool| {
        serde_json::json!({
            "name": tool.name,
            "signature": tool.python_signature(),
            "description": tool.description,
        })
    }).collect();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s).with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
```

---

## Summary

| # | File:Line | Issue | Severity |
|---|-----------|-------|----------|
| 1 | `production_orchestrator.rs:224-243` | Checkpoint save/restore are no-ops | P1 |
| 2 | `style_harmonizer.rs:112-165` | All 3 core methods return hardcoded fake data | P1 |
| 3 | `agent_cmds.rs:353-359` | `mcp stubs` always returns empty — `gateway()` unimplemented | P2 |

All three are production-facing APIs with TODO/FIXME markers indicating incomplete implementation. Pain points 1 and 2 are P1 because they silently lie to consumers (false durability, false quality scores). Pain point 3 is P2 because it's a dead feature but not actively harmful.
