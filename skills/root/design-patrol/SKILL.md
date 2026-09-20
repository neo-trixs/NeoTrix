---
name: design-patrol
description: Use when running automated design system health checks, detecting redundancy, flat defects, cross-domain misalignment, or dependency violations in the NeoTrix capability architecture. Auto-patrol for design capability quality.
---

# Design Patrol — 自动巡检修复

## Purpose

Automated health checks for the NeoTrix design capability architecture. Detects redundancy clusters, flat defects, cross-domain misalignment, and dependency violations. Part of the multi-agent self-inspection system.

## When to Use

- After any design skill or capability change
- During iteration cycles (weekly full scan)
- Before releases (mandatory full audit)
- When adding new capabilities (dependency check)

## 6 Patrol Rules

### PATROL-001: Dependency Graph Cycle Detection

**Detection**: Traverse the capability dependency graph (petgraph DAG). Any directed cycle = violation.

**Scan Command**:
```bash
# Read skills/index.json, extract dependencies, check for cycles
# Output: list of cycles with involved skills
```

**Fix Action**: Break cycle by removing the weakest dependency edge or introducing an intermediate layer.

### PATROL-002: Orphan Capability Detection

**Detection**: Capability nodes with zero in-degree AND zero out-degree in the capability graph.

**Scan**: Any skill in index.json with empty `dependencies: []` AND no other skill lists it as a dependency.

**Fix Action**: Either connect to relevant capability cluster or mark as deprecated.

### PATROL-003: Cross-Layer Violation Detection

**Detection**: L(n) module referencing L(n+k) where k > 0 (forward reference).

**Scan**: Check that nt_* modules only import from same layer or lower layers.

**Fix Action**: Introduce bridge module at the correct layer boundary.

### PATROL-004: Flat Defect Detection

**Detection**: Capability registered with constellation level < C3 and registered > 30 days ago.

**Scan**: Compare capability_registry.json timestamps against constellation levels.

**Fix Action**: Either elevate (implement Rust native) or mark as "LLM-only" explicitly.

### PATROL-005: Redundancy Cluster Detection

**Detection**: Same `provides` tag appearing in > 2 capability nodes.

**Scan**: Aggregate all `provides` arrays from capability_registry.json, find duplicates.

**Fix Action**: Merge overlapping capabilities into unified entry point.

### PATROL-006: Configuration Completeness

**Detection**: Skill missing `triggers` or `dependencies` fields in index.json.

**Scan**: Parse index.json, validate all skills have required fields.

**Fix Action**: Auto-supply empty `triggers: []` and `dependencies: []`.

## Execution Modes

### Incremental (per-commit)
- Run PATROL-001 (dependency cycles)
- Run PATROL-003 (cross-layer violations)
- Output: PASS/FAIL with specific violations

### Daily
- Run PATROL-002 (orphans)
- Run PATROL-005 (redundancy)
- Output: Warning list with recommendations

### Weekly (full scan)
- Run all 6 PATROL rules
- Generate comprehensive report
- Output: `patrol-report-YYYY-MM-DD.md`

### Per-iteration
- Full scan + auto-fix recommendations
- Integration with self-iteration-agent

## Output Contract

```markdown
## Design Patrol Report — YYYY-MM-DD

### Summary
- Rules executed: 6
- PASS: X | WARN: Y | FAIL: Z
- New issues since last scan: N

### Issues
1. [FAIL] PATROL-001: Cycle detected
   - Path: design-core → design-audit → design-core
   - Fix: Remove design-audit dependency on design-core

2. [WARN] PATROL-004: Flat defect
   - Capability: nt_design_icon (C2, registered 45 days ago)
   - Recommendation: Implement Rust native or mark LLM-only

...
```

## Integration

- Hooks into `self-iteration-agent` for automated scheduling
- Feeds into `repair-healer` for auto-fix execution
- Results written to `.neotrix/patrol/` directory
