# Targeted Research 706 — Consciousness Core Dispatch Routes

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop, expanding the capability routing table and match arms for domain-specific pipelines.

## New Routes

| Route ID | Domain | Specialist | Purpose |
|----------|--------|------------|---------|
| `knowledge_distillation` | NT-MIND | KnowledgeIntegrator | Teacher-student / self-distillation / feature transfer pipelines |
| `experience_crystallization` | NT-MEMORY | KnowledgeIntegrator | Pattern extraction, merge consolidation, decay pruning |
| `skill_transfer` | NT-MIND | KnowledgeIntegrator | Cross-model skill export/import/clone/bidirectional |
| `emotional_regulation` | NT-FEEL | ReflectionEngine | Suppression / reappraisal / blend / adaptive strategies |

## CAPABILITY_ROUTES Entries (20 new keywords)

- **knowledge_distillation**: `knowledge_distillation`, `知识蒸馏`, `蒸馏管线`, `模型蒸馏`, `teacher_student`
- **experience_crystallization**: `experience_crystallization`, `经验结晶`, `结晶经验`, `经验固化`, `crystallize`
- **skill_transfer**: `skill_transfer`, `技能迁移`, `跨模型迁移`, `技能转移`, `transfer_skill`
- **emotional_regulation**: `emotional_regulation`, `情感调节`, `情绪调节`, `情感管控`, `regulate_emotion`

## Match Arm Behavior

Each new match arm follows the established pattern:
- KB-backed routes query stats and KV counters for stateful reporting
- `emotional_regulation` parses target emotion and strategy from task summary, reports EmotionLabel variant
- All routes produce structured output with KB node/kv counts and last-run timestamps

## Domain Alignment

| Route | Domain | Rationale |
|-------|--------|-----------|
| `knowledge_distillation` | NT-MIND | SEAL distillation extension — model compression and knowledge transfer |
| `experience_crystallization` | NT-MEMORY | Experience-tree lifecycle — converting raw experience into durable crystallized knowledge |
| `skill_transfer` | NT-MIND | Cross-model capability propagation — aligns with SEAL skill crystallization |
| `emotional_regulation` | NT-FEEL | EmotionEngine调控 — maps to NT-FEEL domain (first non-NT-CORE emotion route) |
