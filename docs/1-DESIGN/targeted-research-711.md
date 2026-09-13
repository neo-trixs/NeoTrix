# Targeted Research 711 — Internal Dispatch Routes Expansion

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new capability routes to the consciousness core dispatch function, expanding the internal routing table and match arms for task decomposition and execution.

## New Routes

| Route ID | Keywords (CN/EN) | Domain | Specialist | Purpose |
|----------|-------------------|--------|------------|---------|
| `memory_pruning` | 记忆剪枝, 记忆衰减, 衰减清理, prune_memory | NT-MEMORY | KnowledgeRetriever | Memory pruning and decay — LRU/threshold/decay strategies for KB entries |
| `skill_versioning` | 技能版本, 技能控制, 版本控制, version_skill | NT-MIND | KnowledgeIntegrator | Skill version control — diff/rollback/tag/status operations on skill registry |
| `emotional_memory` | 情感记忆, 情绪记忆, 记忆标签, emotion_memory | NT-FEEL | ReflectionEngine | Emotional memory tagging — tag/query/decay memories with EmotionLabel 11-variant |
| `confidence_calibration` | 置信度校准, 置信度, 校准置信, calibrate_confidence | NT-META | MetaCognitionAnalyst | Confidence calibration — ECE/Brier score measurement and recalibration |

## Domain Mapping

- **NT-MEMORY** (memory_pruning): Extends existing memory subsystem (adjacent to `memory_consolidation`, `kb_governance`)
- **NT-MIND** (skill_versioning): Extends SEAL/skill subsystem (adjacent to `skill_evolution`, `skill_crystallize`)
- **NT-FEEL** (emotional_memory): Extends emotion subsystem (adjacent to `emotional_regulation`, `emotion_blending`)
- **NT-META** (confidence_calibration): Extends meta-cognition subsystem (adjacent to `meta_cognition`, `self_test_t3`)

## Implementation Pattern

Each match arm follows the established convention:
1. Open KB via `KnowledgeBase::open(None)`
2. Read stats and query KV counters for persistence
3. Parse `task.summary` for mode/strategy selection
4. Return `(executed: bool, output: String)` tuple

## CAPABILITY_ROUTES Added

20 new keyword entries (4 routes × 5 keywords each including the canonical route ID).

## Match Arms Added

4 new `match` branches in `dispatch_internal_capability()`, each with KB-backed state tracking and strategy parsing.
