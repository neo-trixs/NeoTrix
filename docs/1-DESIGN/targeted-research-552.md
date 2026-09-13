# Targeted Research 552 — NT-SHIELD Hardening

**Date**: 2026-09-12
**Sources**: computer-repair-skill (88lin), airgorah (martin-olivier)
**Domain**: NT-SHIELD (影卫)
**Layer**: L3 Embodiment

## Source Analysis

### computer-repair-skill (88lin)

**Core Axiom**: Evidence-first, read-only-first, confirm before state changes.

- 64 on-demand playbooks with routing — each playbook is self-contained with its own evidence collection
- Every repair step produces verifiable artifacts before any destructive operation
- Read-only diagnostics run first; state-changing operations require explicit confirmation
- Evidence packages carry file hashes, timestamps, and rule metadata for independent audit

### airgorah (martin-olivier)

**Core Pattern**: Privilege separation via polkit agent.

- GTK4 GUI runs as normal user; Rust backend performs privileged operations
- Privilege escalation goes through polkit D-Bus — the agent cannot self-escalate
- Each privilege request is an explicit `ContextRequest` with trust level
- GUI never holds elevated privileges beyond what polkit grants per-operation

## Absorbed Patterns into NT-SHIELD

### 1. Evidence-First Scan (`audit.rs`)

**New**: `EvidencePackage` struct + `evidence_first_scan()` method on `SecurityAudit`.

Every security finding now carries a verifiable evidence package:
- `file_hash`: SHA-256 of scanned file (integrity verification)
- `matched_line`: exact triggering line (not trimmed)
- `line_number`: precise 1-indexed location
- `rule_name`: which rule fired
- `scan_timestamp`: ISO-8601 timestamp
- `severity` + `owasp_tag`: calibrated classification

**Why**: Makes findings independently auditable outside the agent's trust boundary — same philosophy as SafetyKernel's signed evidence.

**File**: `neotrix-core/src/l3_embodiment/nt_shield/shield_core/audit.rs`

### 2. Read-Only-First Pentest Mode (`nt_shield_pentest_agent.rs`)

**New**: `PentestMode` enum (ReadOnlyFirst | ActiveExploitation) + `PentestAction` classification.

- Default mode is `ReadOnlyFirst` — recon, port scanning, header checks allowed
- Exploitation actions (SQLi, XSS, command injection) blocked unless explicitly confirmed
- `enable_active_exploitation(confirmed: bool)` requires SafetyKernel approval
- `validate_action()` returns error if action is mutating in read-only mode

**Why**: Prevents accidental state changes during vulnerability discovery. Follows computer-repair-skill pattern: diagnostics first, then controlled escalation.

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_pentest_agent.rs`

### 3. Privilege Separation Documentation (`context_boundary.rs`)

**New**: Module-level documentation mapping airgorah's polkit model to NeoTrix trust levels.

| airgorah Layer | NeoTrix TrustLevel | Allowed Operations |
|---|---|---|
| polkit agent (root) | `System` | Everything |
| GTK4 GUI (user) | `User` | read/write/execute/search |
| D-Bus interface | `Tool` | read/search only |
| External input | `External` | read only |
| Sandbox / untrusted | `Untrusted` | nothing |

**New**: `detect_privilege_escalation()` method — blocks Tool/External/Untrusted from mutating actions.

**Why**: The agent never escalates its own trust level. Privilege requires external `ContextRequest` with higher `TrustLevel`.

**File**: `neotrix-core/src/l3_embodiment/nt_shield/shield_core/context_boundary.rs`

### 4. Confirm-Before-Destructive Checks (`safety_kernel.rs`)

**New**: `DestructiveAction` enum + destructive detection in `SafetyKernel::check()`.

Destructive actions (file delete, directory delete, DB modify, system config change) always require explicit confirmation — regardless of risk score. The SafetyKernel refuses to auto-approve them.

- `DestructiveAction::from_request()` detects destructive patterns in action requests
- `SafetyKernel::check()` gates destructive actions before policy evaluation
- `requires_destructive_confirmation()` API for callers to check before proceeding
- Confirmation includes target, risk score, and explicit "CONFIRM/DENY" prompt

**Why**: computer-repair-skill axiom: "confirm before state changes." Even low-risk destructive actions get a human-in-the-loop gate.

**File**: `neotrix-core/src/l3_embodiment/nt_shield/shield_core/safety_kernel.rs`

## Integration Points

| New Component | Integrates With | Connection |
|---|---|---|
| `EvidencePackage` | `SafetyKernel._SignedEvidence` | Both produce verifiable artifacts |
| `PentestMode` | `SafetyKernel.check()` | Mode switches require kernel approval |
| `DestructiveAction` | `PermissionManager` | Destructive ops go through permission chain |
| `detect_privilege_escalation` | `ContextBoundary.validate()` | Catches escalation before trust check |

## Constellation Maturity

| Component | C0 | C1 | C2 | Notes |
|---|---|---|---|---|
| EvidencePackage | Y | Y | - | Unit tests pass, no integration tests yet |
| PentestMode | Y | Y | - | Unit tests pass, stub adapter |
| Privilege separation | Y | Y | - | Doc + unit tests, polkit not wired |
| DestructiveAction | Y | Y | - | Unit tests pass, SafetyKernel integration |

## Files Modified

1. `neotrix-core/src/l3_embodiment/nt_shield/shield_core/audit.rs` — EvidencePackage + evidence_first_scan
2. `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_pentest_agent.rs` — PentestMode + PentestAction
3. `neotrix-core/src/l3_embodiment/nt_shield/shield_core/context_boundary.rs` — Privilege separation docs + escalation detection
4. `neotrix-core/src/l3_embodiment/nt_shield/shield_core/safety_kernel.rs` — DestructiveAction + confirm-before-destructive
