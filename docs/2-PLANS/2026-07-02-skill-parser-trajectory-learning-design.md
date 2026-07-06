# SKILL.md Parser + Trajectory Learning Design

## Overview

Two P0 blind spots from External Absorption Cycle 3:

1. **SKILL.md parser**: NeoTrix has `skill_acquire.rs` (bank/validation/optimizer) but cannot parse the agentskills.io standard SKILL.md format used by 40+ platforms. Each skill document contains structured fields (`## Skill`, `## Purpose`, `## Steps`, `## Description`) that must be extracted for progressive disclosure.

2. **Trajectory-to-Learning**: E8/GWT produces state trajectories but never extracts reusable learnings from success/failure patterns. Inspired by IBM 2603.10600 (self-improving agents through trajectory analysis).

## Architecture

### Layer Placement

| Module | Location | Layer | Purpose |
|--------|----------|-------|---------|
| `SkillManifest` | `skill_acquire.rs` | L7 Capability | Parse SKILL.md structured fields |
| `SkillRegistry` | `skill_acquire.rs` | L7 Capability | Manage parsed skills by ID |
| `ProgressiveDisclosure` | `skill_acquire.rs` | L7 Capability | 3-level disclosure (trigger/summary/full) |
| `TrajectoryLearner` | `nt_core_trajectory_learn.rs` | L4 Cognition | Extract Strategy/Recovery/Optimization tips |
| `store_learning_report()` | `nt_memory_kb/mod.rs` | L3 Memory | Store learning reports as Insight nodes |
| KB absorption | `engine_core.rs` | L8 Autonomic | Wire trajectory analysis → KB |

### SKILL.md Format

```
## Skill
skill-name

## Purpose
What this skill does (1-2 lines)

## Steps
1. First step
2. Second step

## Description
Detailed description of the skill's behavior
```

### `SkillManifest` — Structured Parsing

```rust
pub struct SkillManifest {
    pub name: String,
    pub purpose: String,
    pub steps: Vec<String>,
    pub description: String,
    pub body: String,
}
```

Uses regex to extract each section: `## Skill\n(.+)` for name, `## Purpose\n(.+?)(?=\n##|\z)` for purpose, etc.

### `SkillRegistry` — Skill Management

```rust
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillManifest>,
}
```

- `new()` — scan `~/.claude/skills/` for `**/SKILL.md` files
- `get(name)` — lookup by name
- `list()` — all registered skills
- `search(query)` — case-insensitive substring match across name + purpose + description

### `ProgressiveDisclosure` — 3-Level API

```rust
pub struct ProgressiveDisclosure {
    pub manifest: Option<SkillManifest>,
}

impl ProgressiveDisclosure {
    pub fn trigger(&self) -> String;    // ~5 lines
    pub fn summary(&self) -> String;     // ~2 paragraphs
    pub fn full(&self) -> String;        // entire manifest
}
```

### `TrajectoryLearner` — Tip Extraction

Three tip types based on IBM 2603.10600 classification:

| Tip Type | Condition | Example |
|----------|-----------|---------|
| `Strategy` | consecutive success steps | "PatternMatcher→AnomalyDetector transition" |
| `Recovery` | failure followed by success | "When Planner fails, try CodeAnalyzer" |
| `Optimization` | duplicate action across steps | "Cache or deduplicate search calls" |

```rust
pub enum LearningTipType { Strategy, Recovery, Optimization }

pub struct LearningTip {
    pub tip_type: LearningTipType,
    pub pattern: String,
    pub recommendation: String,
    pub confidence: f64,
    pub provenance: String,
}

pub struct LearningReport {
    pub tips: Vec<LearningTip>,
    pub trajectory_id: String,
    pub total_steps: usize,
    pub success: bool,
}
```

### KB Absorption Pipeline (engine_core)

```
reason(task)
  → E8 state transitions
  → trajectory compression (TrajectoryCompressor)
  → analyze state patterns for repeated modes
  → build LearningReport as serde_json::Value
  → kb.store_learning_report(&report)
  → stored as Insight node, domain=trajectory_learning
```

## Layer Compliance

| Caller | Callee | Direction | Rule |
|--------|--------|-----------|------|
| `engine_core` (L8) | `TrajectoryLearner` (L4) | 8→4 ↓ | ✅ allowed |
| `engine_core` (L8) | `KnowledgeBase` (L3) | 8→3 ↓ | ✅ allowed |
| `KnowledgeBase` (L3) | `TrajectoryLearner` (L4) | — | ✅ no direct call; JSON barrier |

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `core/l7_capability/skill_acquire.rs` | MOD | +SkillManifest, SkillRegistry, ProgressiveDisclosure, extract_skill_from_text |
| `core/nt_core_trajectory_learn.rs` | NEW | TrajectoryLearner + 3 tip types + skill_doc generation |
| `core/mod.rs` | MOD | +pub mod nt_core_trajectory_learn + re-exports |
| `engine_core.rs` | MOD | +TrajectoryLearner import + KB absorption after compression |
| `nt_memory_kb/mod.rs` | MOD | +store_learning_report() |

## Test Results

- `trajectory_learn`: 6/6 ✅
- `skill_acquire`: 5/5 ✅ (0 regression)
- `cargo check --lib`: 0 errors ✅
- `cargo test --no-run`: 0 errors ✅

## Future Work

1. **Adaptive thresholds**: Instead of hardcoded `confidence: 0.7` for strategy tips, learn from KB feedback
2. **Multi-trajectory pattern mining**: Correlate patterns across multiple episodes
3. **Skill auto-generation**: From trajectory learning reports → auto-generate SKILL.md files
4. **Cross-session learning**: Persist `SkillRegistry` to disk and reload across sessions
