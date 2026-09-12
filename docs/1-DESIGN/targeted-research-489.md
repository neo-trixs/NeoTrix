# Targeted Research #489 — Internal Pain Points (WISER Iteration)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — production-critical stub functions, dead code, hardcoded values

---

## Pain Point 1: Consciousness Agent `take_snapshot` Uses Synthetic Data

**Location:** `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:271-276`

**Severity:** P0

**What's wrong:** The `take_snapshot()` method that feeds the entire consciousness self-audit loop computes phi and coherence from deterministic formulas (`0.5 + cycle * 0.001`), never reading actual system state. Every downstream probe (gap detection, patch generation, free energy) receives fabricated metrics — the self-healing loop is running blind.

```rust
// BEFORE (line 271-276)
fn take_snapshot(&mut self) {
    // TODO: 从实际系统获取状态
    self.state_snapshot = StateSnapshot::new(
        self.cycle,
        0.5 + (self.cycle as f64 * 0.001).min(0.5),
        0.5 + (self.cycle as f64 * 0.0005).min(0.5),
    );
}

// AFTER (fix sketch)
fn take_snapshot(&mut self) {
    let phi = crate::core::nt_core_self::metrics::compute_phi();       // real IIT integration
    let coherence = crate::core::gwt::compute_coherence();             // real GWT resonance
    self.state_snapshot = StateSnapshot::new(self.cycle, phi, coherence);
}
```

**Impact:** 100% of self-healing decisions are based on fabricated data. All probe→patch cycles are theater.

---

## Pain Point 2: `ProductionOrchestrator` Checkpoint Save/Restore Is No-Op

**Location:** `neotrix-core/src/l1_action/nt_act/actions/production_orchestrator.rs:224-242`

**Severity:** P1

**What's wrong:** `save_checkpoint()` accepts a workflow_id, gets the workflow reference, then does literally nothing — returns `Ok(())`. `restore_from_checkpoint()` does the inverse: sets status to Paused without restoring any data. A crash mid-workflow loses all progress permanently.

```rust
// BEFORE (line 224-232)
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    if let Some(workflow) = self.workflows.get(workflow_id) {
        // TODO: 实际保存检查点到持久化存储
        let _ = workflow;
        Ok(())
    } else { Err("工作流不存在".to_string()) }
}

// AFTER (fix sketch)
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let wf = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let json = serde_json::to_vec(wf)
        .map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(
        format!("{}/{}.ckpt", self.checkpoint_dir, workflow_id),
        &json,
    ).map_err(|e| format!("写入失败: {e}"))?;
    Ok(())
}
```

**Impact:** Long-running production workflows (batch video, trade orchestration) cannot survive restarts.

---

## Pain Point 3: `ResourceBudgetManager::estimate_cost` Hardcodes Stale Model Prices

**Location:** `neotrix-core/src/l1_action/nt_act/resource_budget.rs:288-296`

**Severity:** P2

**What's wrong:** Cost estimation uses a 4-model match table with prices frozen at an unknown point in time. Models like `gpt-4-turbo`, `claude-3.5-sonnet`, `gemini-1.5-pro`, `deepseek-r1` all fall to the `_ => 0.001` default — off by 10-100x. Budget enforcement becomes meaningless for any non-default model.

```rust
// BEFORE (line 288-296)
let cost_per_1k = match model {
    "gpt-4" => 0.03,
    "gpt-3.5-turbo" => 0.002,
    "claude-3" => 0.015,
    _ => 0.001,
};

// AFTER (fix sketch)
fn cost_per_1k_tokens(model: &str) -> f64 {
    match model {
        m if m.starts_with("gpt-4o")       => 0.0025,
        m if m.starts_with("gpt-4-turbo")  => 0.01,
        m if m.starts_with("gpt-4")        => 0.03,
        m if m.starts_with("claude-3.5")   => 0.003,
        m if m.starts_with("claude-3")     => 0.015,
        m if m.starts_with("gemini-1.5")   => 0.00125,
        m if m.contains("deepseek")        => 0.00014,
        _                                  => 0.01, // safe over-estimate
    }
}
```

**Impact:** Budget guards undercount costs 10-100x for most modern models, allowing runaway spend.

---

## Summary

| # | Pain Point | File:Line | Severity | Category |
|---|-----------|-----------|----------|----------|
| 1 | Consciousness snapshot uses synthetic phi/coherence | `agent.rs:271` | P0 | Hardcoded empty result |
| 2 | Production checkpoint save/restore is no-op | `production_orchestrator.rs:224` | P1 | Dead code path |
| 3 | Cost estimation hardcodes stale 4-model table | `resource_budget.rs:288` | P2 | Hardcoded values |
