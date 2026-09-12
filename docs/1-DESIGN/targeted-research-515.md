# Targeted Research 515 — WISER Internal Pain Points (Iteration 6)

**Date**: 2026-09-13
**Method**: Codebase grep for TODO stubs, hardcoded returns, dead code
**Scope**: `neotrix-core/src/` (shield, CLI, core)

---

## Pain Point 1: NT-SHIELD Reverse Engineer — Fake Analysis Results

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_reverse_engineer.rs:71-77`

**What's wrong**: `_extract_cfg()` returns hardcoded `ControlFlowGraph { nodes: 12, edges: 18, loops: 3 }` instead of actually parsing Ghidra output. The `analyze_binary()` method (line 39) also returns hardcoded function signatures and vulnerability findings — never calls Ghidra headless. The entire `GhidraAnalyzer` is a facade over fake data.

**Severity**: P1 — Shield security analysis silently produces fabricated results. A user trusting these findings would have a false sense of security.

**Fix sketch**:
```rust
pub fn _extract_cfg(&self) -> _ControlFlowGraph {
    // Delegate to Ghidra headless Python API output
    let output = std::process::Command::new(&self.ghidra_path)
        .args(["analyzeHeadless", "/tmp", "proj",
               "-import", self.functions.first()
               .map(|f| &f.addr).unwrap_or(&String::new()).as_str(),
               "-postScript", "export_cfg.py"])
        .output()
        .expect("Ghidra headless failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    _parse_cfg_from_ghidra(&stdout)  // real parser
}
```

---

## Pain Point 2: NT-SHIELD Internal Scan — Hardcoded Network Topology

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-207`

**What's wrong**: `discover_hosts()` and `enumerate_services()` (line 210) both return hardcoded fake host/service data (192.168.1.1 gateway, 192.168.1.10 webserver, etc.) instead of actually calling `fscan` or system commands. The TODO at line 174 confirms: `// TODO: 实际调用 fscan 或系统命令`. The `ScanConfig` struct exists but is never used in the scan logic.

**Severity**: P1 — Internal network scanning produces fabricated inventory. Dangerous if used for security assessment.

**Fix sketch**:
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let output = std::process::Command::new("fscan")
        .args(["-t", &self.config.target_cidr, "-p", "all",
               "-o", "/tmp/fscan_hosts.json"])
        .output()
        .map_err(|e| format!("fscan not found: {}", e))?;
    let raw: Vec<_HostInfo> = serde_json::from_slice(&output.stdout)
        .unwrap_or_default();
    self.internal_hosts = raw.clone();
    Ok(raw)
}
```

---

## Pain Point 3: MCP Registry Facade — All Methods Return Empty

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:48-65`

**What's wrong**: `McpRegistry` is a stub struct (line 48) where every method returns `Vec::new()`, `0`, `None`, or no-ops. The real implementations exist in `nt_agent_mcp_gateway` and `nt_core_orch_agent` (both commented out at lines 7-8). This means CLI commands like `agent mcp list-tools`, `agent mcp search`, and `agent mcp publish` silently produce empty results.

**Severity**: P2 — MCP tool discovery is non-functional at CLI level. Users see no registered tools even when servers are running.

**Fix sketch**:
```rust
impl McpRegistry {
    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        // Delegate to real gateway
        crate::l1_action::nt_io::nt_agent_mcp_gateway::ProgrammaticPlanner::new(self)
            .list_available_tools()
            .into_iter()
            .map(|t| McpToolInfo {
                name: t.name,
                description: t.description,
                server_name: t.server_name,
            })
            .collect()
    }
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Pattern |
|---|-----------|-----------|----------|---------|
| 1 | GhidraAnalyzer returns fake CFG data | `nt_shield_reverse_engineer.rs:71` | P1 | Hardcoded return |
| 2 | FscanModule returns fake host inventory | `nt_shield_internal_scan.rs:173` | P1 | Hardcoded return |
| 3 | McpRegistry stub returns empty vectors | `agent_cmds.rs:48-65` | P2 | Dead code facade |

All three share the same root cause: **facade structs that were scaffolded for architecture documentation but never wired to real implementations**. The shield modules (P1) are highest priority because fabricated security data is actively dangerous.
