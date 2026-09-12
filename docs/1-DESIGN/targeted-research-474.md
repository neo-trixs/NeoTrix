# Targeted Research 474 — Internal Pain Points (WISER Loop #4)

Date: 2026-09-12
Scope: `neotrix-core/src/` — TODOs, hardcoded values, dead code paths

---

## Pain Point 1: Speculative Decoding Fabricates Results

**File:** `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-184`
**Severity:** P0 — returns fabricated metrics to callers, lies about token generation

**What's wrong:** `SpeculativeDecoder::generate()` never actually runs draft verification. It returns `output_tokens: vec![]` (empty) while reporting fabricated `accepted_count`, `throughput_multiplier: 2.0-4.0`, and `latency_savings_ms`. Any downstream consumer trusting these metrics gets phantom speedup data. The function computes `expected_accepted = draft_tokens * acceptance_rate` but never verifies any tokens — it's pure arithmetic on stale stats.

**Fix sketch:**
```rust
pub async fn generate(&self, prompt: &str, max_tokens: usize) -> SpeculativeResult {
    // 1. Acquire draft model lock
    let draft = self.draft_model.lock().await;
    let draft_tokens = draft.generate(prompt, self.draft_model.num_draft_tokens).await;

    // 2. Batch-verify with target model (single forward pass)
    let target = self.target_model.lock().await;
    let verification = target.verify(prompt, &draft_tokens).await;

    // 3. Accept prefix until first rejection; re-verify from rejection point
    let accepted = verification.prefix_length();
    let remaining = if accepted < draft_tokens.len() {
        target.generate(prompt, max_tokens - accepted).await
    } else {
        vec![]
    };

    SpeculativeResult {
        output_tokens: [verification.accepted().to_vec(), remaining].concat(),
        total_draft_tokens: draft_tokens.len(),
        accepted_count: accepted,
        rejection_count: draft_tokens.len() - accepted,
        throughput_multiplier: 1.0 + (accepted as f64 / max_tokens as f64),
        latency_savings_ms: (accepted as u64 * self.avg_token_ms),
    }
}
```

---

## Pain Point 2: Internal Network Scanner Returns Hardcoded Host Data

**File:** `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-206`
**Severity:** P1 — security scanner returns fake network topology; no actual scanning occurs

**What's wrong:** `discover_hosts()` ignores the `// TODO: 实际调用 fscan 或系统命令` and returns 3 hardcoded `_HostInfo` entries (`192.168.1.1`, `192.168.1.10`, `192.168.1.25`) with fabricated IPs, hostnames, MACs, and open ports. `enumerate_services()` (line 210) does the same — a `match host` returning canned service banners. Any security audit or penetration test workflow built on this module gets entirely fabricated intelligence.

**Fix sketch:**
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let subnet = self.config.subnet.as_deref().unwrap_or("192.168.0.0/24");
    let output = tokio::process::Command::new("fscan")
        .args(["-t", subnet, "-p", "all", "-o", "json"])
        .output().await.map_err(|e| format!("fscan not found: {e}"))?;

    let hosts: Vec<_HostInfo> = serde_json::from_slice(&output.stdout)
        .unwrap_or_default();

    if hosts.is_empty() {
        return Err("No hosts discovered — verify subnet and fscan availability".into());
    }

    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## Pain Point 3: PTC Stubs & Exec Always Return Empty Results

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:353-354` and `:467-468`
**Severity:** P1 — programmatic tool calling feature is completely non-functional

**What's wrong:** The `stubs` subcommand (line 353) always returns `Vec::new()` with 0 typed signatures. The `exec` subcommand (line 467) always returns `Vec::new()` with 0 results. Both are gated on `// FIXME: McpRegistry.gateway() not yet implemented`. The `McpRegistry` struct exists but has no `gateway()` method bridging to the agent runtime. Users calling `/mcp stubs` or `/mcp exec` get a success message with zero results — silent failure.

**Fix sketch:**
```rust
// In McpRegistry, implement gateway():
impl McpRegistry {
    pub async fn gateway(&self) -> Result<Vec<McpToolEntry>, String> {
        let mut tools = Vec::new();
        for server in self.servers.iter() {
            if let Some(handle) = server.handle.as_ref() {
                let discovered = handle.list_tools().await?;
                tools.extend(discovered);
            }
        }
        Ok(tools)
    }
}

// In agent_cmds.rs, wire it:
let registry = get_mcp_registry();
let tools = registry.read().await.gateway().await.unwrap_or_default();
let stubs: Vec<serde_json::Value> = tools.iter().map(|t| render_stub(t)).collect();
```

---

## Summary

| # | File | Issue | Severity |
|---|------|-------|----------|
| 1 | `speculative_decoding.rs:161` | `generate()` returns empty tokens + fabricated throughput metrics | P0 |
| 2 | `nt_shield_internal_scan.rs:173` | `discover_hosts()` returns 3 hardcoded fake hosts | P1 |
| 3 | `agent_cmds.rs:353,467` | PTC stubs/exec always return `Vec::new()` (gateway unimplemented) | P1 |

All three are silent failures — the code compiles and "succeeds" but produces fabricated/empty data.
