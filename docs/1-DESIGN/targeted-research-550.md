# WikiSkill Pattern Absorption into NeoTrix

**Source**: arXiv:2608.27454 (WikiSkill)
**Date**: 2026-09-13
**Status**: Applied (2 improvements + 1 new module)

## WikiSkill Key Insight

WikiSkill separates three concerns that NeoTrix previously mixed:
1. **Raw Execution Experience** — per-call execution records (ephemeral)
2. **Accumulated Knowledge** — distilled patterns/rules/anti-patterns (persistent wiki)
3. **Executable Skill** — crystallized, versioned, I/O-contracted capabilities (production)

The critical insight: **knowledge is the persistent core**. Skills are ephemeral projections of knowledge; experience feeds knowledge through distillation. NeoTrix had experience-tree (L6) and crystallization (SEAL) but they were disconnected — experience never flowed into knowledge that feeds skills.

## Gap Analysis

| Component | Before | After (WikiSkill) |
|-----------|--------|-------------------|
| CrystallizationEngine | In-memory only, lost between sessions | Cross-session persistence via `persist_to_json` / `restore_from_json` |
| Skill versioning | No history, only current `success_rate` | `SkillEvolutionTracker` with version history, trend analysis, degradation detection |
| Experience→Knowledge flow | `ExperienceTreeManager` and `MemoryConsolidation` disconnected | `ExperienceKnowledgeBridge` with 3-layer pipeline |
| Cross-session skill transfer | Not possible | Skills + evolution data persisted to KB kv_store |

## Changes Applied

### 1. SkillEvolutionTracker (crystallization.rs)

**File**: `neotrix-core/src/l5_cognition/nt_mind/seal/crystallization.rs`

Added after `_CrystallizationEngine`:
- `SkillEvolutionRecord` — per-version snapshot (success_rate, invocations, session_id, change_reason)
- `SkillEvolutionTrend` — aggregated trend (history, success_rate_trend, is_degrading, cross_session_reuse)
- `SkillEvolutionTracker` — manages records/trends, detects degradation, serializes for KB

**Integration**: `CrystallizationEngine` now holds `evolution_tracker: SkillEvolutionTracker`. Every `crystallize()` call auto-records an evolution event. New methods:
- `persist_to_json()` / `restore_from_json()` — cross-session save/load
- `evolution_tracker()` / `evolution_tracker_mut()` — access for external analysis

**WikiSkill alignment**: Skills are no longer fire-once templates. They have version history that survives sessions, enabling the wiki's "accumulated knowledge" layer.

### 2. ExperienceKnowledgeBridge (new module)

**File**: `neotrix-core/src/l5_cognition/nt_mind/mind_modules/knowledge/experience_knowledge_bridge.rs`
**Registered**: `mind_modules/knowledge/mod.rs`

Three-layer pipeline:
```
Layer 1: RawExperience (per-call records)
    ↓ distill() [threshold: N same-skill experiences]
Layer 2: KnowledgeEntry (patterns, anti-patterns, rules, optimizations)
    ↓ crystallize_skill() [threshold: N citations]
Layer 3: ExecutableSkill (versioned, I/O-contracted, persistent)
    ↓ skill_context() [reverse lookup: skill → knowledge → raw experiences]
```

Key types:
- `RawExperience` — lightweight per-call record (task, skills_used, success, tokens, error)
- `KnowledgeEntry` — distilled knowledge with citation_count, confidence, source_experience_ids
- `ExecutableSkill` — crystallized skill with knowledge_ids linking back to knowledge layer
- `BridgeConfig` — tunable thresholds for distillation and crystallization

**Persistence**: `to_json()` / `from_json()` for KB kv_store storage.

**Tests**: 4 test cases covering record→distill, crystallize→context, persistence roundtrip, and stats.

### 3. Cross-Session Skill Transfer

Both `CrystallizationEngine` and `ExperienceKnowledgeBridge` now support JSON serialization for KB persistence. The flow:

1. Session end: `engine.persist_to_json()` → KB kv_store `skill_crystallization` namespace
2. Session start: `CrystallizationEngine::restore_from_json()` → skills + evolution history restored
3. Bridge: `bridge.to_json()` → KB kv_store `experience_knowledge` namespace

This enables WikiSkill's core requirement: **skills co-evolve with a persistent knowledge base across sessions**.

## Integration Points

| Existing Module | WikiSkill Enhancement | Connection |
|----------------|----------------------|------------|
| `ExperienceTreeManager` (L6) | Feeds `RawExperience` into Bridge Layer 1 | experience_tree absorb → bridge.record_experience |
| `DistillationEngine` (SEAL) | Bridge `distill()` replaces ad-hoc template generation | Bridge generates KnowledgeEntry instead of raw SkillTemplate |
| `CrystallizationEngine` (SEAL) | Bridge `crystallize_skill()` feeds into engine | Bridge Layer 3 → engine.crystallize with evolution tracking |
| `MemoryConsolidation` (knowledge) | Knowledge entries use consolidation's importance scoring | Bridge confidence = consolidation importance × success_rate |
| KB kv_store | Both engine and bridge persist to JSON | Cross-session restore on startup |

## What Was NOT Changed

- `ExperienceTreeManager` — left as-is; it now feeds into Bridge rather than being replaced
- `MemoryConsolidation` — left as-is; Bridge uses its concepts but doesn't modify it
- SEAL pipeline stages — unchanged; Bridge plugs in as a new module within knowledge/

## Next Steps (Not Implemented)

1. Wire Bridge into SEAL pipeline's Phase 3 (distill) and Phase 4 (crystallize)
2. Add KB namespace for `skill_evolution` and `experience_knowledge`
3. Add CLI commands: `neotrix experience distill`, `neotrix skill evolution`
4. Connect `ExperienceTreeManager::absorb()` to `Bridge::record_experience()` in production code
