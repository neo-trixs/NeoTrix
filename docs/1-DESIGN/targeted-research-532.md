# Targeted Research #532 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-13
**Source**: grep scan of `neotrix-core/src/` for TODO/FIXME/HACK, hardcoded returns, dead code
**Severity legend**: P0 = silent user-facing failure, P1 = broken self-healing/evolution, P2 = dead code / cosmetic

---

## Pain Point 1 — PTC `stubs` and `exec` return empty results (P0)

**Location**: `neotrix-core/src/cli/commands/agent_cmds.rs:353-358` and `:467-468`

**What's wrong**: Both `/mcp stubs` and `/mcp exec` command handlers contain:
```rust
// FIXME: McpRegistry.gateway() not yet implemented
let stubs: Vec<serde_json::Value> = Vec::new();  // stubs
let results: Vec<serde_json::Value> = Vec::new(); // exec
```
The PTC (Programmatic Tool Calling) pipeline plans the call graph successfully via `ProgrammaticPlanner`, but the actual execution gate — `McpRegistry.gateway()` — is unimplemented. The stubs command also returns zero signatures. Both commands silently report success with empty data. Users see `"⚡ PTC exec: 2 stage(s), 0 call(s)"` with no indication that the gateway is the bottleneck.

**Fix sketch**:
```rust
// agent_cmds.rs — "stubs" arm, replace Vec::new() with gateway stub generation
let gateway = registry.gateway().await.map_err(|e| {
    CommandOutput::err(&format!("[stubs] gateway unavailable: {e}"))
})?;
let stubs: Vec<serde_json::Value> = gateway.generate_stubs().await;
let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());

// agent_cmds.rs — "exec" arm, replace Vec::new() with gateway dispatch
let gateway = registry.gateway().await.map_err(|e| {
    CommandOutput::err(&format!("[exec] gateway unavailable: {e}"))
})?;
let results: Vec<serde_json::Value> = gateway.execute_plan(&plan).await;
let s = format!("⚡ PTC exec: {} stage(s), {} call(s)\n", plan.stages(), results.len());
```
If `McpRegistry::gateway()` genuinely cannot be implemented yet, the command should return an explicit error instead of empty success:
```rust
return CommandOutput::err("PTC gateway not yet wired — see fusion-plan-215");
```

---

## Pain Point 2 — NT-GOVERNANCE human oversight registration is a no-op (P1)

**Location**: `neotrix-core/src/l6_meta/coordination/nt_governance/mod.rs:10-13`

**What's wrong**: The entire human oversight self-test registration function is empty:
```rust
pub fn register_human_oversight_self_tests(
    _registry: &mut crate::core::nt_core_self_test::SelfTestRegistry,
) {
    // TODO: 实现人类监督治理自测注册
}
```
This is the NT-GOVERNANCE domain's only entry point for human oversight self-tests. The function signature accepts a `SelfTestRegistry` but registers zero tests. The `skill_validator` module is imported but `human_oversight` is never declared. The Dark Forest axiom requires every module to compile + test + connect — this module compiles but has zero consumers and zero tests, making it dead weight.

**Fix sketch**:
```rust
// nt_governance/mod.rs
pub mod skill_validator;
pub mod human_oversight;  // declare the module

pub fn register_human_oversight_self_tests(
    registry: &mut crate::core::nt_core_self_test::SelfTestRegistry,
) {
    use crate::core::nt_core_self_test::SelfTest;
    struct HumanOversightTest;
    impl SelfTest for HumanOversightTest {
        fn name(&self) -> &str { "nt_governance_human_oversight" }
        fn self_test(&self) -> Result<(), Vec<String>> {
            // Verify governance constraints are enforced
            // e.g. verify skill_validator rejects invalid skills
            Ok(())
        }
    }
    registry.register(Box::new(HumanOversightTest));
}
```

---

## Pain Point 3 — VerifierAgent uses simulate_verification instead of VLM (P1)

**Location**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-196`

**What's wrong**: The `_verify_shot` method (video quality gate) bypasses actual visual verification:
```rust
// TODO: 实际调用 VLM 进行验证
let scores = self.simulate_verification(spec_description, memory_context);
```
The `simulate_verification` method uses keyword matching on the spec description to assign scores — no actual video content is analyzed. This means the entire quality control pipeline (`/kb quality-control`) approves content based on text heuristics, not visual ground truth. A video with completely wrong frames but matching keywords in the spec would pass.

**Fix sketch**:
```rust
// verifier_agent.rs — _verify_shot, replace simulate call with VLM dispatch
pub(crate) fn _verify_shot(
    &mut self,
    shot_id: &str,
    video_path: &str,
    spec_description: &str,
    memory_context: Option<&str>,
) -> VerificationResult {
    let start = std::time::Instant::now();

    // Extract keyframes from video_path
    let keyframes = self.extract_keyframes(video_path).unwrap_or_default();

    // Dispatch to VLM provider for visual verification
    let scores = if !keyframes.is_empty() {
        self.verify_with_vlm(&keyframes, spec_description, memory_context)
    } else {
        // Fallback to heuristic only when keyframe extraction fails
        tracing::warn!("no keyframes extracted for {shot_id}, falling back to heuristic");
        self.simulate_verification(spec_description, memory_context)
    };

    let total_score = self.calculate_total_score(&scores);
    let passed = total_score >= self.config.pass_threshold;
    // ... rest unchanged
}
```

---

## Summary

| # | Pain Point | Severity | File | Line |
|---|-----------|----------|------|------|
| 1 | PTC stubs/exec return empty Vec | P0 | agent_cmds.rs | :354, :468 |
| 2 | Governance human oversight no-op | P1 | nt_governance/mod.rs | :10-13 |
| 3 | VerifierAgent skips VLM verification | P1 | verifier_agent.rs | :195 |

**Recommended fix order**: 1 → 3 → 2 (user-facing first, then quality gate, then self-test wiring).
