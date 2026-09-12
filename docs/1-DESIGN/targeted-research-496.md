# Targeted Research #496 — Internal Pain Points (Iteration 50)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — hollow facades, synthetic state, redundant registries

---

## Pain Point 1: PTC (Programmatic Tool Calling) is a Dead Facade

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:48-76`
**Severity**: P1

**What's wrong**: The entire PTC subsystem — `McpRegistry`, `McpDiscovery`, `ProgrammaticPlanner`, and `Plan` — are all stub structs with hardcoded empty returns. Every method returns `Vec::new()`, `None`, `0`, or an error string. The `/mcp stubs` and `/mcp exec` commands silently return "0 stubs" and "0 results" with success status. Users see a functional CLI that does nothing.

```rust
// agent_cmds.rs:48-76 — all methods are stubs
pub struct McpRegistry;
impl McpRegistry {
    pub fn new() -> Self { Self }
    pub fn gateway(&self) -> Option<String> { None }
    pub fn list_tools(&self) -> Vec<McpToolInfo> { Vec::new() }
    pub fn search(&self, _query: &str) -> Vec<McpToolInfo> { Vec::new() }
    pub fn publish(&mut self, ...) -> usize { 0 }
    pub fn as_native_tools(&self) -> Vec<Box<dyn NativeTool>> { Vec::new() }
}
pub struct ProgrammaticPlanner;
impl ProgrammaticPlanner {
    pub fn plan(&self, _calls: Vec<ProgrammaticCall>) -> Result<Plan, String> {
        Err("ProgrammaticPlanner::plan not yet implemented".to_string())
    }
}
pub struct Plan;
impl Plan { pub fn stages(&self) -> usize { 0 } }
```

**Fix sketch**:
```rust
pub struct McpRegistry {
    servers: HashMap<String, McpServer>,
    tool_index: HashMap<String, Vec<McpToolInfo>>,
}
impl McpRegistry {
    pub fn new() -> Self {
        Self { servers: HashMap::new(), tool_index: HashMap::new() }
    }
    pub fn gateway(&self) -> Option<String> {
        self.servers.values().next().map(|s| s.endpoint.clone())
    }
    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        self.tool_index.values().flatten().cloned().collect()
    }
    pub fn register_stdio(&mut self, server: &str, cmd: &str, args: &[&str],
                          tools: Vec<McpToolDef>) {
        let endpoint = format!("stdio:{}:{}", cmd, args.join(" "));
        self.servers.insert(server.to_string(), McpServer { endpoint });
        self.tool_index.insert(server.to_string(),
            tools.into_iter().map(|t| McpToolInfo {
                name: t.name, description: t.description,
                server_name: server.to_string(),
            }).collect());
    }
}
```

---

## Pain Point 2: Consciousness Core StateSnapshot is Entirely Synthetic

**File**: `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:271-276`
**Severity**: P0

**What's wrong**: `take_snapshot()` generates all 18+ system health metrics from the cycle counter using deterministic formulae (`todo_count: 100 - (cycle * 10)`, `memory_usage_mb: 2000 - (cycle * 100)`). The system reports itself as "improving" every cycle regardless of actual state. The consciousness core — the system's self-monitoring loop — is blind. It cannot detect real regressions, resource exhaustion, or code quality degradation.

```rust
// agent.rs:270-276 — synthetic state, no real metrics
fn take_snapshot(&mut self) {
    // TODO: 从实际系统获取状态
    self.state_snapshot = StateSnapshot::new(
        self.cycle,
        0.5 + (self.cycle as f64 * 0.001).min(0.5),
        0.5 + (self.cycle as f64 * 0.0005).min(0.5),
    );
}

// state.rs:94-118 — all values derived from cycle counter
pub fn new(cycle: u32, phi: f64, coherence: f64) -> Self {
    Self {
        cycle,
        phi, coherence,
        todo_count: 100 - (cycle * 10).min(95),
        memory_usage_mb: 2000 - (cycle as u64 * 100).min(1500),
        injection_risks: 8 - (cycle as i32).min(8) as u32,
        // ... all synthetic
    }
}
```

**Fix sketch**:
```rust
fn take_snapshot(&mut self) {
    let todo_count = self.scan_todo_count(); // grep src/ for TODO markers
    let mem = std::fs::read_dir("src/")
        .map(|d| d.count() as u32)
        .unwrap_or(0);
    self.state_snapshot = StateSnapshot {
        cycle: self.cycle,
        todo_count,
        interfaces_total: mem,
        // Real phi/coherence from GWT engine, not formulae
        phi: self.gwt_engine.current_phi(),
        coherence: self.gwt_engine.current_coherence(),
        memory_usage_mb: sys_info::mem_info()
            .map(|m| m.used as u64)
            .unwrap_or(0),
        // ... fill from actual subsystem queries
    };
}
```

---

## Pain Point 3: SubAgentRegistry Redundant with CapabilityRegistry

**File**: `neotrix-core/src/core/nt_core_subagent.rs:278-303`
**Severity**: P2

**What's wrong**: The file itself documents the redundancy: `TODO(fusion-plan-215): Merge into CapabilityRegistry — SubAgentRegistry is a redundant registry`. It maintains a parallel `HashMap<String, SubAgentDef>` with its own scan/load/refresh logic, duplicating what `CapabilityRegistry` already provides. Every `SubAgentRegistry::new()` emits a deprecation warning. Dead code risk: two registries diverge over time.

```rust
// nt_core_subagent.rs:278-303 — redundant, self-deprecated
/// TODO(fusion-plan-215): Merge into `CapabilityRegistry` — SubAgentRegistry is a redundant
/// registry that overlaps with capability-based agent management.
pub struct SubAgentRegistry {
    agents: HashMap<String, SubAgentDef>,
    source_dirs: Vec<PathBuf>,
    scan_count: u64,
}
impl SubAgentRegistry {
    pub fn new() -> Self {
        SUBAGENT_REGISTRY_DEPRECATED.call_once(|| {
            tracing::warn!(
                "SubAgentRegistry is deprecated — merge into CapabilityRegistry (fusion-plan-215)"
            );
        });
        // ... full implementation still exists
    }
}
```

**Fix sketch**:
```rust
// nt_core_subagent.rs — delete SubAgentRegistry entirely
// Move scan logic to CapabilityRegistry::scan_agent_dirs()
// Replace all call sites with CapabilityRegistry methods

// In capability_registry.rs:
impl CapabilityRegistry {
    pub fn scan_agent_dirs(&mut self, dirs: &[PathBuf]) {
        for dir in dirs {
            if !dir.exists() { continue; }
            for entry in std::fs::read_dir(dir).into_iter().flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "md") { continue; }
                if let Ok(def) = SubAgentDef::parse(&path) {
                    self.register_agent(def);
                }
            }
        }
    }
}
```

---

## Summary

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| 1 | `agent_cmds.rs:48-76` | PTC/McpRegistry/Planner all empty stubs — dead facade | P1 |
| 2 | `agent.rs:271-276`, `state.rs:94-118` | Consciousness core generates synthetic metrics — self-monitoring is blind | P0 |
| 3 | `nt_core_subagent.rs:278-303` | SubAgentRegistry redundant with CapabilityRegistry (self-documented) | P2 |

**Total TODO/FIXME stubs in neotrix-core/src/**: ~85 found, 3 critical reported above.
**Cumulative iterations**: 50 (27 + 3 from #495 + 3 from #496 = 33 fixed, 52 remaining in backlog).
