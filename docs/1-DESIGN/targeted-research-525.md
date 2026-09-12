# Targeted Research #525 — Internal Pain Points (Iteration Loop)

**Date**: 2026-09-13
**Context**: Continuing iteration loop. 27 critical issues already fixed. These are 3 more internal pain points.

---

## Pain Point 1 — P0: Fake VLM Verification in VerifierAgent

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-282`
**Domain**: NT-L6 Meta-Cognition (quality gate for production pipeline)

### What's wrong

`verify_video_frame()` is the quality gate for the entire video generation pipeline. It is supposed to call a VLM (Vision-Language Model) to verify visual consistency, but instead it runs `simulate_verification()` — a **keyword-matching heuristic** that scores based on whether words like "character", "action", or "lighting" appear in the text description. This means:

- The quality gate **never actually sees the image/video**
- A description containing "character" gets entity_score=7; without it gets 9 — backwards logic
- `auto_correct_prompt()` (line 337) also has a TODO — it just appends suggested corrections, never calls an LLM

This silently passes bad content and rejects good content based on text heuristics.

### Severity: P0

This is the **only quality gate** for visual content production. Fake verification = no quality control.

### Fix sketch

```rust
// verifier_agent.rs:195
pub async fn verify_video_frame(
    &mut self, shot_id: &str, video_path: &str,
    spec_description: &str, memory_context: Option<&str>,
) -> VerificationResult {
    let start = std::time::Instant::now();
    // Call actual VLM provider (e.g. GPT-4V / Gemini Vision)
    let vlm_result = self.llm_provider
        .call_vision(vec![video_path], &self.build_vlm_prompt(spec_description))
        .await;
    let scores = match vlm_result {
        Ok(raw) => self.parse_vlm_scores(&raw),  // structured output parsing
        Err(e) => {
            tracing::warn!("VLM verification failed, falling back to heuristic: {e}");
            self.simulate_verification(spec_description, memory_context)
        }
    };
    // ... rest unchanged
}
```

---

## Pain Point 2 — P1: NT-SHIELD Internal Scan Returns Hardcoded Fake Hosts

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-207`
**Domain**: NT-SHIELD (security scanning)

### What's wrong

`discover_hosts()` (line 173) is supposed to perform real network host discovery (ARP ping / ICMP sweep) but instead returns **3 hardcoded fake hosts** with deterministic IPs, hostnames, MACs, and open ports:

```rust
let hosts = vec![
    _HostInfo { ip: "192.168.1.1".into(), hostname: Some("gateway".into()), ... },
    _HostInfo { ip: "192.168.1.10".into(), hostname: Some("webserver".into()), ... },
    _HostInfo { ip: "192.168.1.25".into(), hostname: Some("database".into()), ... },
];
```

Similarly `enumerate_services()` (line 210) returns hardcoded service data per IP.

This is a **security tool that lies about its results**. An agent relying on this for threat assessment will believe the network has exactly these 3 hosts, missing real threats.

### Severity: P1

Security tool returning fabricated data is worse than no tool — it creates false confidence.

### Fix sketch

```rust
// nt_shield_internal_scan.rs:173
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let subnet = self.config.target_subnet.as_deref()
        .unwrap_or("192.168.0.0/24");
    // Use `arp-scan` or `fscan` if available, else `ping` sweep
    let output = tokio::process::Command::new("arp-scan")
        .args(["--localnet", "--retry=2", "--timeout=1000"])
        .output().await
        .map_err(|e| format!("arp-scan not found: {e}"))?;
    let hosts = parse_arp_scan_output(&output.stdout);
    if hosts.is_empty() {
        // Fallback: ICMP ping sweep
        self.icmp_sweep(subnet).await
    } else {
        self.internal_hosts = hosts.clone();
        Ok(hosts)
    }
}
```

---

## Pain Point 3 — P1: PTC Stubs/Exec Always Returns Empty (Broken McpRegistry.gateway)

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:353-469`
**Domain**: NT-ACT (tool orchestration / programmatic tool calling)

### What's wrong

Two FIXME markers at lines 353 and 467 reference the same missing implementation:

- `/mcp stubs` (line 350-359): Creates `stubs: Vec::new()` — always returns **0 typed signatures**
- `/mcp exec` (line 461-469): Creates `results: Vec::new()` — always returns **0 execution results**

The `McpRegistry.gateway()` method that bridges planning to execution is not implemented. The `ProgrammaticPlanner::plan()` validates calls successfully, but the actual execution path returns empty. This means the entire PTC (Programmatic Tool Calling) subsystem — the core of agent multi-tool orchestration — is **dead on arrival**.

### Severity: P1

PTC is the mechanism agents use for chained/parallel tool calls in a single turn. Without it, agents must make serial single-tool calls, breaking multi-step workflows.

### Fix sketch

```rust
// agent_cmds.rs:461-469
// FIXME: McpRegistry.gateway() not yet implemented
let gateway = registry.gateway().ok_or("McpRegistry.gateway not initialized")?;
let mut results: Vec<serde_json::Value> = Vec::new();
for stage in plan.stages() {
    let stage_results = futures::future::join_all(
        stage.calls.iter().map(|call| {
            let gw = gateway.clone();
            async move { gw.execute(&call.tool, &call.args).await }
        })
    ).await;
    for r in stage_results {
        match r {
            Ok(v) => results.push(v),
            Err(e) => results.push(serde_json::json!({"error": e.to_string()})),
        }
    }
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | Fake VLM verification (keyword heuristic, never sees images) | `verifier_agent.rs:195` | **P0** | Quality gate blind — bad content passes, good content fails |
| 2 | Hardcoded fake network scan results | `nt_shield_internal_scan.rs:173` | **P1** | Security tool lies about network state |
| 3 | PTC stubs/exec always empty (gateway not implemented) | `agent_cmds.rs:353,467` | **P1** | Multi-tool orchestration completely broken |
