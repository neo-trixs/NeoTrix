# Targeted Research 514 — Internal Pain Points (WISER Loop)

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — production stubs, dead paths, and hardcoded return values
**Method:** Grep TODO/FIXME/HACK/unimplemented!, trace callers, verify no real impl exists

---

## Pain Point 1: McpRegistry is a Hollow Stub — PTC Pipeline Dead on Arrival

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:48-61`
**Severity:** P0

### What's Wrong

`McpRegistry` is defined as a zero-field unit struct with every method returning empty/zero/None:

```rust
pub struct McpRegistry;            // line 48 — no fields
impl McpRegistry {
    pub fn gateway(&self) -> Option<String> { None }          // line 51
    pub fn list_tools(&self) -> Vec<McpToolInfo> { Vec::new() } // line 52
    pub fn search(&self, _query: &str) -> Vec<McpToolInfo> { Vec::new() } // line 53
    pub fn publish(&mut self, ..) -> usize { 0 }             // line 54
    pub fn as_native_tools(&self) -> Vec<..> { Vec::new() }  // line 55
    pub fn tool_count(&self) -> usize { 0 }                  // line 56
    // ...
}
```

This is the **only** `McpRegistry` impl in the entire codebase (confirmed: grep finds 1 struct definition, 1 impl block). Two `FIXME` markers at lines 353 and 467 acknowledge this.

**Impact:** Three CLI commands silently produce no output:
- `/agent stubs` (line 350-359): PTC typed signatures → always "0 typed signatures"
- `/agent exec` (line 467-477): PTC execution → always "0 call(s)"  
- `/mcp discover` (line 362-374): MCP discovery → always "0 candidates"

The entire PTC (Programmatic Tool Calling) subsystem is wired but has no real MCP integration.

### Fix Sketch

```rust
// agent_cmds.rs:48-61 — replace hollow stub with real MCP bridge
use std::process::Command;

pub struct McpRegistry {
    servers: Vec<McpServerEntry>,
}

struct McpServerEntry {
    name: String,
    cmd: String,
    args: Vec<String>,
    tools: Vec<McpToolInfo>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self { servers: Self::load_from_config() }
    }

    fn load_from_config() -> Vec<McpServerEntry> {
        // Read ~/.neotrix/mcp.json or $NEOTRIX_MCP_CONFIG
        // Parse server definitions, spawn probe to list tools
        todo!("MCP config loading")
    }

    pub fn gateway(&self) -> Option<String> {
        self.servers.first().map(|s| s.name.clone())
    }

    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        self.servers.iter().flat_map(|s| s.tools.clone()).collect()
    }

    pub fn tool_count(&self) -> usize {
        self.servers.iter().map(|s| s.tools.len()).sum()
    }
}
```

**Prerequisite:** Define `mcp.json` schema (server name, command, args, env). Build config loader in `nt_io_mcp_config`.

---

## Pain Point 2: VerifierAgent Uses Keyword Heuristics Instead of VLM

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-282`
**Severity:** P0

### What's Wrong

`_verify_shot()` calls `simulate_verification()` which scores video shots by **counting keywords in the text description**:

```rust
fn simulate_verification(&self, description: &str, _ctx: Option<&str>) -> Vec<VerificationScore> {
    let entity_score = if desc_lower.contains("character") || desc_lower.contains("角色") {
        7   // ← keyword match, not image comparison
    } else {
        9
    };
    // ... same pattern for env_score, narrative_score, instruction_score
}
```

This means the entire quality-control verification pipeline (QualityControl → VerifierAgent → VerificationResult) is **cosmetic** — it always passes with 7-9 scores regardless of actual video content. A corrupted video with the word "character" in the description still gets entity_score=7.

The pipeline is called from `quality_control.rs:282` and feeds into regeneration decisions (line 203), but those decisions are based on fake data.

### Fix Sketch

```rust
// verifier_agent.rs:195 — replace simulate_verification with real VLM call
async fn verify_shot(
    &self,
    shot_id: &str,
    video_path: &str,
    spec_description: &str,
    memory_context: Option<&str>,
) -> Result<VerificationResult, VerificationError> {
    let start = Instant::now();

    // Extract keyframes from video
    let keyframes = extract_keyframes(video_path, 5)?;

    // Call VLM for each verification dimension
    let scores = self.verify_dimensions(&keyframes, spec_description, memory_context).await?;

    let total_score = self.calculate_total_score(&scores);
    let passed = total_score >= self.config.pass_threshold;

    Ok(VerificationResult {
        passed,
        total_score,
        scores,
        needs_regeneration: !passed && self.history.len() < self.config.max_regeneration_attempts as usize,
        verification_time_ms: start.elapsed().as_millis() as u64,
        ..Default::default()
    })
}

async fn verify_dimensions(&self, keyframes: &[Image], desc: &str, ctx: Option<&str>) -> Result<Vec<VerificationScore>, VerificationError> {
    // Use LLM provider from nt_io_llm to call vision model
    let provider = get_active_provider()?;
    let prompt = format!("Compare these video frames against the spec: {desc}");
    let response = provider.vision_call(keyframes, &prompt).await?;
    parse_dimension_scores(&response)
}
```

**Prerequisite:** Wire `VerifierAgent` to an LLM provider with vision capability (via `nt_io_llm`).

---

## Pain Point 3: KB Consistency Command Returns Empty String

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:150-160`
**Severity:** P1

### What's Wrong

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {  // opens DB successfully
        Some(c) => c,
        None => return CommandOutput::err("..."),
    };
    let out = String::new();  // ← empty output
    // TODO: setting_consistency module not found; stub
    // let _ = crate::...::setting_consistency::check_and_report_to_string(&conn, &mut out);
    CommandOutput::ok(&out)   // ← returns OK with empty string
}
```

The `setting_consistency` module is **commented out** in `nt_memory_kb/mod.rs:55`:

```rust
// pub mod nt_memory_setting_consistency;
```

The DB connection opens successfully, but the consistency check logic was never implemented. `/kb consistency` silently succeeds with zero output — a false positive.

### Fix Sketch

```rust
// nt_memory_kb/mod.rs:55 — uncomment the module
pub mod nt_memory_setting_consistency;

// kb_cmds.rs:150-160 — wire the real check
fn cmd_consistency(args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };

    let mut out = String::new();
    match setting_consistency::check_and_report_to_string(&conn, &mut out) {
        Ok(()) => {
            if out.is_empty() {
                CommandOutput::ok("设定一致性检查通过: 无冲突")
            } else {
                CommandOutput::ok(&out)
            }
        }
        Err(e) => CommandOutput::err(&format!("一致性检查失败: {e}")),
    }
}
```

**Prerequisite:** Verify `nt_memory_setting_consistency` module exists (or create it) with the `check_and_report_to_string` function that queries KB namespaces for setting conflicts.

---

## Summary

| # | Pain Point | File:Line | Severity | Root Cause |
|---|-----------|-----------|----------|------------|
| 1 | McpRegistry hollow stub | `agent_cmds.rs:48-61` | **P0** | Zero-field struct, all methods return empty/zero |
| 2 | VerifierAgent keyword heuristics | `verifier_agent.rs:195-282` | **P0** | `simulate_verification` counts words instead of calling VLM |
| 3 | KB consistency dead command | `kb_cmds.rs:150-160` | **P1** | Module commented out, returns empty string |

### Recommended Fix Order

1. **P1 (KB consistency)** — Lowest effort: uncomment module, verify it exists or create it, wire to existing KB query infrastructure
2. **P0 (McpRegistry)** — Medium effort: define MCP config schema, implement config loader, wire `gateway()` to real server list
3. **P0 (VerifierAgent)** — High effort: needs LLM vision provider integration, keyframe extraction pipeline, prompt engineering for dimension scoring
