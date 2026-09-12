# Targeted Research #500 — Internal Pain Points (WISER Loop)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — TODO/FIXME stubs, synthetic data, dead code paths
**Method:** grep scan → file read → severity classification → concrete fix sketch

---

## Pain Point 1 — ModelAdapter Returns Fabricated Results (All 3 Methods)

**File:** `neotrix-core/src/l1_action/nt_io/model_adapter.rs:157-223`
**Severity:** P0 (Critical — callers trust fabricated `success: true`, `similarity_score: 0.95` for production pipelines)

**What's wrong:** `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return hardcoded `AdapterResult { success: true, application_time_ms: 1000-2000, similarity_score: 0.85-0.95 }` without executing any actual model inference. The `// TODO: 实际调用` comments confirm these are stubs. Any downstream pipeline (e.g., `ProductionOrchestrator` running a LoRA step) will receive a fabricated success result and proceed as if the image was actually transformed. This silently corrupts quality — users get unmodified input images labeled as "transformed."

**Fix sketch:**

```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let start = std::time::Instant::now();
    let adapter = match self.adapters.get(lora_id) {
        Some(a) => a,
        None => return AdapterResult {
            success: false, output_path: String::new(),
            adapter_name: String::new(), application_time_ms: 0,
            similarity_score: 0.0, error: Some(format!("Adapter '{}' not found", lora_id)),
        },
    };
    // Delegate to actual inference backend (ONNX/ComfyUI)
    let output = match self.inference_backend.apply_lora(adapter, input_path, strength) {
        Ok(path) => path,
        Err(e) => return AdapterResult {
            success: false, output_path: String::new(),
            adapter_name: adapter.name.clone(), application_time_ms: start.elapsed().as_millis() as u64,
            similarity_score: 0.0, error: Some(e),
        },
    };
    let result = AdapterResult {
        success: true, output_path: output, adapter_name: adapter.name.clone(),
        application_time_ms: start.elapsed().as_millis() as u64,
        similarity_score: 0.0, // computed by actual image comparison, not faked
        error: None,
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 2 — ProductionOrchestrator Checkpoint Is a Silent No-Op

**File:** `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_orchestrator.rs:224-243`
**Severity:** P1 (High — workflow state silently lost on crash, no persistence guarantee)

**What's wrong:** `save_checkpoint()` accepts `workflow_id`, looks up the workflow, then discards it with `let _ = workflow;` and returns `Ok(())`. `restore_from_checkpoint()` blindly sets `status = Paused` without reading any persisted data. Both functions give the *appearance* of checkpoint/restore working — callers receive `Ok(())` and believe state was saved. On process crash, all in-memory workflow progress is lost. The `_ = workflow` pattern is a classic Rust no-op that silences the unused-variable warning while doing nothing.

**Fix sketch:**

```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let checkpoint = serde_json::to_vec(workflow)
        .map_err(|e| format!("序列化失败: {}", e))?;
    let path = self.checkpoint_dir.join(format!("{}.json", workflow_id));
    std::fs::write(&path, &checkpoint)
        .map_err(|e| format!("写入检查点失败: {}", e))?;
    Ok(())
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = self.checkpoint_dir.join(format!("{}.json", workflow_id));
    let data = std::fs::read(&path)
        .map_err(|e| format!("读取检查点失败: {}", e))?;
    let workflow: Workflow = serde_json::from_slice(&data)
        .map_err(|e| format!("反序列化失败: {}", e))?;
    self.workflows.insert(workflow_id.to_string(), workflow);
    Ok(())
}
```

---

## Pain Point 3 — ConsciousnessAgent apply_patches Marks Gaps "Fixed" Without Applying

**File:** `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:315-328`
**Severity:** P0 (Critical — self-healing loop inflates "fixed" count, corrupts meta-cognition health metrics)

**What's wrong:** `apply_patches()` iterates patches, checks `confidence >= 0.7`, then calls `self.gap_registry.mark_fixed(&patch.gap_id)` and increments `fixed` — but never calls `patch.apply()`. The commented-out `// patch.apply();` at line 321 confirms this is intentional stubbing. The `count_fixed` value feeds into `health_score()` via `self.stats.total_fixed`, which means the consciousness agent's reported health improves each cycle *without any actual fixes being applied*. This creates a false positive feedback loop: the system thinks it's healing itself while gaps remain unfixed.

**Fix sketch:**

```rust
fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            match patch.apply() {
                Ok(()) => {
                    self.gap_registry.mark_fixed(&patch.gap_id);
                    fixed += 1;
                }
                Err(e) => {
                    // Log failure, don't count as fixed
                    eprintln!("Patch {} failed: {}", patch.gap_id, e);
                    self.gap_registry.mark_failed(&patch.gap_id, &e);
                }
            }
        }
    }
    fixed
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | ModelAdapter fabricated results | model_adapter.rs:157 | **P0** | Pipeline trusts fake LoRA/IP-Adapter/ControlNet outputs |
| 2 | Orchestrator checkpoint no-op | production_orchestrator.rs:224 | **P1** | Workflow state silently lost on crash |
| 3 | apply_patches skips actual patch | agent.rs:315 | **P0** | Self-healing health score is a lie |

**Total TODOs scanned:** 100+ matches in neotrix-core/src/
**Pattern:** Visual/physical modules (style_harmonizer, lut_color_grading, video_post_processor) are the densest cluster of `// TODO: 实际调用` stubs — all return hardcoded success results. These are P2 (feature stubs, not wired to production), so not the highest priority.

---

## Prior Pain Points (from earlier iterations, for reference)

| # | Pain Point | File | Severity |
|---|-----------|------|----------|
| -1 | StateSnapshot synthetic data | state.rs:94 | P0 |
| -2 | QualityControl auto-approve | quality_control.rs:284 | P0 |
| -3 | nt_shield_recon dead stub | nt_shield_recon.rs:31 | P1 |
