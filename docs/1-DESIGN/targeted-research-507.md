# Targeted Research #507 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-13  
**Method**: Static analysis of `neotrix-core/src/` — TODO/FIXME/HACK comments, hardcoded returns, dead code paths  
**Scope**: 3 critical stub implementations that silently fail or return fabricated data

---

## Pain Point 1: Production Orchestrator Checkpoint is No-Op

**File**: `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_orchestrator.rs:224-243`  
**Severity**: **P0**

### What's Wrong

`save_checkpoint()` and `restore_from_checkpoint()` return `Ok(())` without persisting anything. Long-running production workflows lose all state on crash. The `_ = workflow;` line discards the workflow reference entirely.

### Evidence

```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    if let Some(workflow) = self.workflows.get(workflow_id) {
        // TODO: 实际保存检查点到持久化存储
        let _ = workflow;  // ← discards reference, no persistence
        Ok(())
    } else {
        Err("工作流不存在".to_string())
    }
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    // TODO: 实际从持久化存储恢复
    if let Some(workflow) = self.workflows.get_mut(workflow_id) {
        workflow.status = WorkflowStatus::Paused;  // ← only changes status, no state restored
        Ok(())
    } else {
        Err("工作流不存在".to_string())
    }
}
```

### Fix

```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let json = serde_json::to_string_pretty(workflow)
        .map_err(|e| format!("序列化失败: {e}"))?;
    let path = Path::new(&self.config.storage_path)
        .join(format!("{workflow_id}.checkpoint.json"));
    std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| std::fs::write(&path, json))
        .map_err(|e| format!("写入失败: {e}"))
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = Path::new(&self.config.storage_path)
        .join(format!("{workflow_id}.checkpoint.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取失败: {e}"))?;
    let workflow: Workflow = serde_json::from_str(&json)
        .map_err(|e| format!("反序列化失败: {e}"))?;
    self.workflows.insert(workflow_id.to_string(), workflow);
    Ok(())
}
```

---

## Pain Point 2: ResourceBudget Cost Estimation Uses Outdated Hardcoded Prices

**File**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs:283-297`  
**Severity**: **P1**

### What's Wrong

`estimate_cost()` only recognizes 3 model names (`gpt-4`, `gpt-3.5-turbo`, `claude-3`) with static prices. This breaks Cost-Aware Routing (Axiom A1) because:
1. Newer models (gpt-4o, claude-3.5-sonnet, gemini-flash) all fall to the default `$0.001/1k` bucket
2. No input/output token differentiation (output is 3-10x more expensive)
3. Prices are already stale — GPT-4 dropped to $0.03→$0.01 since launch

### Evidence

```rust
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // TODO: 实际调用成本计算
    let cost_per_1k = match model {
        "gpt-4" => 0.03,
        "gpt-3.5-turbo" => 0.002,
        "claude-3" => 0.015,
        _ => 0.001,  // ← all unknown models cost the same
    };
    (token_count as f64 / 1000.0) * cost_per_1k
}
```

### Fix

```rust
pub fn estimate_cost(&self, input_tokens: u64, output_tokens: u64, model: &str) -> f64 {
    let (input_price, output_price) = match model {
        "gpt-4o" | "gpt-4o-mini" => (0.0025 / 1000.0, 0.01 / 1000.0),
        "gpt-4-turbo" => (0.01 / 1000.0, 0.03 / 1000.0),
        "claude-3.5-sonnet" | "claude-3-sonnet" => (0.003 / 1000.0, 0.015 / 1000.0),
        "claude-3-haiku" => (0.00025 / 1000.0, 0.00125 / 1000.0),
        "gemini-flash" | "gemini-1.5-flash" => (0.000075 / 1000.0, 0.0003 / 1000.0),
        _ => (0.001 / 1000.0, 0.003 / 1000.0),  // conservative fallback
    };
    (input_tokens as f64 * input_price) + (output_tokens as f64 * output_price)
}
```

---

## Pain Point 3: MultiRegionScheduler Health Check Always Returns True

**File**: `neotrix-core/src/l1_action/nt_act/actions/infra/multi_region_scheduler.rs:191-199`  
**Severity**: **P1**

### What's Wrong

`_check_region_health()` unconditionally sets `RegionStatus::Available` and returns `true` without actually probing the region. A region that's down will keep receiving traffic. The `_` prefix on the method name also signals it's unused/dead code.

### Evidence

```rust
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        // TODO: 实际的健康检查逻辑
        region.status = RegionStatus::Available;  // ← always available
        true
    } else {
        false
    }
}
```

### Fix

```rust
pub(crate) async fn check_region_health(&mut self, region_id: &str) -> bool {
    let region = match self.regions.get(region_id) {
        Some(r) => r,
        None => return false,
    };
    let endpoint = format!("https://{}/health", region.endpoint);
    let healthy = match tokio::time::timeout(
        Duration::from_secs(5),
        reqwest::get(&endpoint),
    ).await {
        Ok(Ok(resp)) => resp.status().is_success(),
        _ => false,
    };
    if let Some(r) = self.regions.get_mut(region_id) {
        r.status = if healthy { RegionStatus::Available } else { RegionStatus::Degraded };
        r.last_health_check = Some(Instant::now());
    }
    healthy
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | Orchestrator checkpoint no-op | `production_orchestrator.rs:224` | **P0** | Production workflows lose state on crash |
| 2 | Cost estimation hardcoded 3 models | `resource_budget.rs:283` | **P1** | Cost-Aware Routing (A1) produces wrong prices |
| 3 | Health check always returns true | `multi_region_scheduler.rs:191` | **P1** | Unhealthy regions keep receiving traffic |

**Total fixes**: 3 pain points, 0 existing from this session (continuing from 27 prior fixes)
