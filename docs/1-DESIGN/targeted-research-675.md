# Targeted Research 675 — Stub Fabricated Success Audit

## Scope

Searched 4 priority directories for functions returning fabricated success data:

| Directory | Files Scanned | Stubs Found |
|-----------|--------------|-------------|
| `l1_action/nt_io/nt_io_provider/` | 50+ | 0 (hardcoded test data in tests only) |
| `l5_cognition/nt_mind/mind_modules/` | 25 | 2 (bpco.rs C0 stub, wordpecker.rs basic stub) |
| `l6_meta/coordination/` | 23 | 3 (quality_control, quality_gate, verifier_agent) |
| `l3_embodiment/nt_shield/` | 80+ | 1 (nt_shield_internal_scan mock hosts) |

## Fixes Applied

### 1. `quality_control.rs` — `evaluate_check_item` (HIGH)

**Before**: Returned hardcoded scores per check type (Technical=0.85, Compliance=0.90, VisualConsistency=0.75, etc.), making every review appear to pass.

**After**: Returns `0.0` for all check types. No real AI analysis capability → score of 0.0 → always triggers Rejected status. Threshold logic preserved for when real VLM is wired.

**File**: `l6_meta/coordination/quality_control.rs:265-284`

### 2. `quality_control.rs` — `_review_by_ai` (HIGH)

**Before**: Returned `ReviewStatus::Approved` with fabricated high scores and empty issues list.

**After**: Returns `ReviewStatus::Rejected` (score=0.0 from `evaluate_check_item`), includes a Critical-severity issue explaining AI analysis is not wired, and comments explicitly state "no real AI analysis".

**File**: `l6_meta/coordination/quality_control.rs:202-267`

### 3. `quality_gate.rs` — `_ai_initial_review` (HIGH)

**Before**: Accepted externally-provided scores, labeled reviewer as `"AI (STUB)"` — callers could mistake externally-provided scores for AI analysis.

**After**: Changed reviewer label to `"External (not AI-analyzed)"`. Comments explicitly state scores are externally provided, not from real AI analysis. Aggregation logic preserved (still useful as a pipeline component, just honestly labeled).

**File**: `l6_meta/coordination/quality_gate.rs:145-184`

### 4. `verifier_agent.rs` — `auto_correct_prompt` (MEDIUM)

**Before**: Doc comment said "STUB: 仅追加 suggested_corrections, 无 LLM 重写" but the implementation was a trivial string append without clear warning.

**After**: Doc comment explicitly states this is a "placeholder implementation" and that simple concatenation "不会产生高质量修正 prompt". Clarified that real implementation needs LLM-based rewriting.

**File**: `l6_meta/coordination/verifier_agent.rs:254-258`

## Stubs Already Honest (No Fix Needed)

| File | Function | Status |
|------|----------|--------|
| `verifier_agent.rs:200` | `_verify_shot` | Uses `todo!()` — panics instead of fabricating data |
| `bpco.rs:42-50` | `critique` | Returns `score: 0.0` with explicit "not wired" rejection |
| `nt_shield_internal_scan.rs:182-207` | `discover_hosts`, `enumerate_services` | Return `Err("stub — requires integration")` |
| `nt_shield_pentest_agent.rs:161-177` | `detect_vulnerabilities` | Returns `Err("PentestGPT adapter not wired")` |
| `yoyobook.rs:62-63` | `extract_lessons` | C0 stub, returns all lessons (documented as C0) |
| `yoyo_gasp_site.rs` | `_GaspRuntimeMapper` | C1 reference node (documented as C1) |
| `wordpecker.rs:58-72` | `extract_entities` | Basic but functional (documented as C1) |

## Test Updates

`quality_control.rs` tests updated to match new behavior:
- `test_quality_pipeline`: Changed assertion from `approved == 1` to `rejected == 1` (AI review now Rejected)
- `test_ai_review_returns_rejected`: New test verifying AI review always Rejects when no real analysis capability is wired

`quality_gate.rs` tests unchanged — they test the aggregation logic which remains correct; only the `reviewer` label changed.

## Principle

**Rejected > Fabricated Approval**: When a capability is not wired, returning a clear rejection (score=0.0, status=Rejected, explicit issue) is always preferable to returning fabricated success data. This prevents silent pass-through of unreviewed content and makes the system's limitations visible to callers.
