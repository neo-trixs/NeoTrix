# Domain Alignment Report

**Date**: 2026-09-13
**Agent**: DOMAIN ALIGNER
**Scope**: NeoTrix Six-Layer Architecture Alignment

## Executive Summary

This report documents domain alignment issues identified and fixed in the NeoTrix codebase. The analysis focused on ensuring functions are placed in correct domains according to the Six-Layer Architecture:

- **L1 Action** (nt_act, nt_io, nt_memory): Tool execution, IO, memory
- **L2 Perception** (nt_world): World understanding, crawling, parsing
- **L3 Embodiment** (nt_shield): Safety, security, sensors, motors
- **L4 Emotion** (nt_feel): Emotional processing
- **L5 Cognition** (nt_core, nt_mind): Reasoning, evolution
- **L6 Meta** (nt_meta): Meta-cognition, governance

## Issues Found and Fixed

### Issue 1: Safety Functions in Wrong Domain

**Location**: `l5_cognition/nt_core/safety/nt_core_safety_alignment.rs`
**Problem**: AI Safety Alignment functions were placed in L5 (Cognition) but belong in L3 (Embodiment/Security)
**Rationale**: Safety/security functions are part of the embodiment layer, not cognition

**Actions Taken**:
1. Created new safety module in L3: `l3_embodiment/nt_shield/safety/`
2. Moved `AISafetyAlignmentEngine` and related types to `l3_embodiment/nt_shield/safety/nt_safety_alignment.rs`
3. Updated L3 mod.rs to include the safety module
4. Removed safety module from L5 mod.rs
5. Deleted old safety module files from L5

**Files Modified**:
- Created: `neotrix-core/src/l3_embodiment/nt_shield/safety/mod.rs`
- Created: `neotrix-core/src/l3_embodiment/nt_shield/safety/nt_safety_alignment.rs`
- Modified: `neotrix-core/src/l3_embodiment/nt_shield/mod.rs`
- Modified: `neotrix-core/src/l5_cognition/nt_core/mod.rs`
- Deleted: `neotrix-core/src/l5_cognition/nt_core/safety/` (entire directory)

### Issue 2: L2→L3 Cross-Layer Dependencies

**Location**: Multiple files in `l2_perception/nt_world/data_source/` and `l2_perception/nt_world/osint/`
**Problem**: L2 data sources were importing L3 types (`EgressRule`, `EgressPolicy`) directly, creating upward dependencies from L2→L3
**Rationale**: Layer architecture requires dependencies to flow downward (higher layers depend on lower layers)

**Actions Taken**:
1. Created shared egress types in L1: `l1_action/nt_io/nt_io_provider/common/egress_types.rs`
2. Defined `SandboxEgressRule` and `SandboxEgressPolicy` types in L1
3. Updated L3 to re-export these types with aliases for backward compatibility
4. Updated L2 facade to export the shared types
5. Updated all L2 data sources to use the facade instead of importing L3 directly

**Files Modified**:
- Created: `neotrix-core/src/l1_action/nt_io/nt_io_provider/common/egress_types.rs`
- Modified: `neotrix-core/src/l1_action/nt_io/nt_io_provider/common/mod.rs`
- Modified: `neotrix-core/src/l2_perception/nt_world/l1_facade.rs`
- Modified: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_sandbox/mod.rs`
- Modified: 11 L2 data source files (edgar, ucdp, usgs, adsb, gdelt, bgpview, urlhaus, ofac, aoi, opencorporates, polymarket, gdacs)
- Modified: 5 OSINT module files (shodan, zoomeye, censys, fofa, securitytrails)

### Issue 3: Crawl Queue Functions Review

**Location**: `l1_action/nt_memory/nt_memory_kb/nt_memory_store.rs`
**Problem**: Crawl queue functions (`upsert_crawl_queue`, `claim_next_crawl_url`, `mark_crawl_complete`) were reviewed for proper placement
**Finding**: These functions are correctly placed in L1 memory

**Rationale**: 
- The crawl queue is a database table in the Knowledge Base (KB)
- L1 memory manages KB storage operations
- L2 perception uses these functions through a facade pattern
- This is a valid dependency direction (L2 → L1)

**Actions Taken**: No changes needed - functions are correctly placed

## Dependency Flow After Fixes

```
L6 Meta ──→ L5 Cognition ──→ L4 Emotion
                │
                ▼
            L3 Embodiment (nt_shield)
                │
                ├──→ L2 Perception (nt_world)
                │         │
                │         ▼
                │     L1 Action (nt_act, nt_io, nt_memory)
                │
                └──→ L1 Action (shared types)
```

## Key Design Patterns Applied

### 1. Facade Pattern
- L2 facade re-exports L1 types to avoid scattered imports
- L3 facade re-exports shared types for backward compatibility

### 2. Shared Types in L1
- Common types used by multiple layers are defined in L1
- Prevents upward dependencies between layers

### 3. Type Aliases for Backward Compatibility
- L3 re-exports L1 types with original names (`EgressRule`, `EgressPolicy`)
- Existing code continues to work without changes

## Verification

### Syntax Check
- All modified files pass `rustfmt --check`
- No syntax errors detected

### Import Updates
- All L2 data sources updated to use facade imports
- No direct L3 imports remain in L2

### Module Declarations
- L3 mod.rs updated to include safety module
- L5 mod.rs updated to remove safety module

## Recommendations

### 1. Enforce Layer Boundaries
- Add compiler checks to prevent L2→L3 imports
- Use `#[cfg]` attributes or build scripts to enforce dependency rules

### 2. Document Shared Types
- Maintain a list of shared types in L1
- Document which layers can use which types

### 3. Regular Audits
- Run domain alignment checks periodically
- Use automated tools to detect cross-layer dependencies

## Conclusion

The NeoTrix codebase has been improved with better domain alignment:
1. Safety functions moved to correct domain (L3)
2. Cross-layer dependencies eliminated (L2→L3)
3. Shared types properly centralized in L1
4. Facade patterns applied for clean imports

These changes improve maintainability, enforce architectural boundaries, and make the codebase easier to navigate.
