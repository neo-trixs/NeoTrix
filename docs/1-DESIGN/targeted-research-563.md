# Targeted Research 563 — Internal Dispatch Route Wiring

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop, extending the `CAPABILITY_ROUTES` table and `dispatch_internal_capability` match block.

## Routes Added

| # | Keyword(s) | Capability Tag | NT Domain | Specialist | Purpose |
|---|-----------|---------------|-----------|------------|---------|
| 1 | `content_moderation`, `内容审核`, `内容过滤`, `内容安全`, `moderation` | `content_moderation` | NT-SHIELD | RiskAssessor | Content safety review via Egress Privacy Guard redaction |
| 2 | `check_registry`, `安全检查`, `检查注册`, `注册表检查`, `registry` | `check_registry` | NT-SHIELD | RiskAssessor | Security check registry scan (recon + signal count) |
| 3 | `layered_qa`, `分层质量`, `质量保证`, `多层检查`, `qa` | `layered_qa` | NT-META | MetaCognitionAnalyst | 3-layer QA: meta-observe + build health + code quality |
| 4 | `video_object_storage`, `视频存储`, `视频对象`, `视频资产`, `video_storage` | `video_object_storage` | NT-WORLD | CodeAnalyzer | Video asset metadata query via KB experience nodes |

## Implementation Details

### CAPABILITY_ROUTES entries (lines ~940-960)
Each route provides both Chinese and English keywords for bilingual matching. Routes follow existing convention: `(keyword, capability_tag, domain, specialist)`.

### dispatch_internal_capability match arms

1. **content_moderation** — Delegates to `redact_internals()` for content safety scanning. Reports input/output character count and whether issues were detected.

2. **check_registry** — Uses `AgenticScanner::recon_scan()` to enumerate entry points, frameworks, and complexity. Reports registered check count.

3. **layered_qa** — Combines three existing subsystems:
   - `MetaObserver::observe()` for meta-cognitive assessment (Φ, coherence)
   - `BuildWatchdog::check_health()` for build health percentage
   - `SentruxSensor::scan()` for code quality score

4. **video_object_storage** — Queries KB via `serve_core()` for video asset metadata. Reports matching asset count with top-3 entries.

## Domain Alignment

| Route | NT Domain | Rationale |
|-------|-----------|-----------|
| content_moderation | NT-SHIELD | Security/content safety = shield domain |
| check_registry | NT-SHIELD | Security scanning infrastructure |
| layered_qa | NT-META | Meta-cognitive quality oversight |
| video_object_storage | NT-WORLD | World perception / asset registry |

## Files Modified

- `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`
  - CAPABILITY_ROUTES: +20 entries (4 routes x 5 keywords each)
  - dispatch_internal_capability: +4 match arms (~80 lines)

## Verification

```sh
cargo check -p neotrix --lib 2>&1 | tail -5
```
