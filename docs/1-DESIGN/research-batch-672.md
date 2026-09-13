# Research Batch 672 — Game Engine Resources Analysis

**Date**: 2026-09-13
**Sources**: 5 repositories analyzed for game engine patterns and NeoTrix fusion opportunities

---

## 1. audinowho/RogueElements

**URL**: https://github.com/audinowho/RogueElements
**Stars**: 63 | **Forks**: 9 | **License**: MIT

### Summary
C# roguelike map generation library built to be game-agnostic. Core architecture revolves around three classes: `MapGen`, `GenStep`, and `IGenContext`. Generation is implemented as interchangeable shader-pass-like steps that share a base class. Each step operates on an `IGenContext` interface representing the map.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Shader-Pass Pipeline** | GenSteps chained sequentially, each transforms map state | SEAL pipeline phases (Soil→Roots→Trunk→Branches→Fruits→Core) |
| **Interface Segregation** | `IGenContext` → `ITiledGenContext` progressive capability interfaces | Six-Layer Architecture traits (L1-L6 `traits.rs` contracts) |
| **GenStep Base Class** | Single `Apply` function on `IGenContext`; steps are composable units | UnifiedCapability trait / capability nodes |
| **Context Mutation Tracking** | Each GenStep reads/writes map state; diagnostic methods snapshot full state | ConsciousnessTree growth cycle snapshots |
| **Game-Agnostic Abstraction** | No gameplay logic in generation; only produces structural output | NeoTrix separation: SEAL produces artifacts, NT-ACT consumes them |
| **FloorPlan → Tile Conversion** | Abstract room layout separate from tile-level rendering | Two-view architecture: CapabilityTree (evolution) ↔ CapabilityRegistry (runtime) |

### Fusion Opportunities

1. **SEAL Phase Pipeline = GenStep Pipeline**: RogueElements' shader-pass model maps directly to NeoTrix's SEAL pipeline. Each SEAL phase (Soil, Roots, Trunk, Branches, Fruits, Core) should be a GenStep-like unit with typed `Apply` on a generation context. This enables: (a) hot-swapping phases during development, (b) diagnostic snapshotting at any phase boundary, (c) parallel phase execution where dependencies allow.

2. **Interface Segregation for Capability Layers**: RogueElements' `IGenContext` → `ITiledGenContext` progression mirrors NeoTrix's L1-L6 trait segregation. Each layer should declare minimal required capabilities via trait bounds, not concrete types. Implement `TiledGenContext`-style progressive enrichment: `ActionLayer` requires `PerceptionLayer` which requires `EmbodimentLayer`.

3. **GenStep Composability → Skill Node Composition**: Each GenStep is an atomic unit with constraints on context type. This maps to NeoTrix's skill nodes (Small Passive / Notable Passive / Keystone). A Keystone skill node is equivalent to a complex GenStep that requires multiple interface capabilities.

### Implementation Changes

- `nt_core_self`: Adopt GenStep pattern for SEAL phases — each phase as a struct implementing `Apply(ctx: &mut SealContext)`
- `nt_core_capability_tree`: Implement progressive capability interfaces (RogueElements' `ITiledGenContext` pattern) for cross-layer trait bounds
- `seal`: Add diagnostic snapshotting at phase boundaries (RogueElements' map-state-at-breakpoint pattern)

---

## 2. edenb-dev/Game-Map-Maker

**URL**: https://github.com/edenb-dev/Game-Map-Maker
**Stars**: 2 | **Forks**: 0 | **License**: MIT

### Summary
Java GUI program for creating 2D game maps. Built with JavaFX, features sprite sheet picking, grid-based drawing, import/export, zoom, block picker mode, and hand mode for panning. Simple but demonstrates core map editing UX patterns.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Sprite Sheet Grid System** | 256px-wide sprite sheets, 64×64 tiles with 1px separators | MediaAssetRegistry tile-based asset organization |
| **Mode State Machine** | Draw/Pan/Pick modes toggled via keyboard shortcuts | GWT attention modes (acquisition/evolution switching) |
| **Serialization Format** | Simple text-based map export (`Map.txt`) | KB persistence format / ExperienceIndex pointers |
| **Canvas Zoom/Pan** | Scroll-wheel zoom + drag-pan with hand mode | NT-IO viewport management for Tauri/desktop UI |
| **Right-Click Delete** | Destructive action with visual feedback | SafeDeleter with archive-before-delete (R-P81) |

### Fusion Opportunities

1. **Map Editor for NeoTrix Visualization**: The sprite-picking + grid-drawing + mode-state-machine pattern could power a visual skill tree editor in NT-IO's Tauri desktop app. Users could visually compose SEAL pipelines by dragging GenStep nodes onto a canvas — analogous to how Game-Map-Maker lets users paint tile maps.

2. **Text-Based Serialization → KB Export**: Game-Map-Maker's simple text export format is instructive for NT-MEMORY export/import. The key insight: serialization should be human-readable (not binary) for debugging and version control. NeoTrix's KB export should use a similar flat-text format.

3. **Mode State Machine for Multi-Agent Workflows**: The Draw/Pan/Pick mode toggling pattern maps to NT-ACT's agent-mode switching. Each mode constrains available actions and input interpretation — directly analogous to how NeoTrix's Dual Specialization routes between CORE+WORLD and CORE+MIND modes.

### Implementation Changes

- `nt_io`: Adopt mode-state-machine pattern for Tauri desktop UI (visual pipeline editor)
- `nt_memory`: Use human-readable flat-text format for KB export alongside SQLite
- `nt_act`: Implement explicit mode-switching for agent workflows (draw/inspect/config modes)

---

## 3. LyraVoid/NTE-Compass

**URL**: https://github.com/LyraVoid/NTE-Compass
**Stars**: 11 | **Forks**: 1 | **License**: AGPL-3.0

### Summary
Interactive web map for the game "Neverness to Everness" (异环). Vue 3 + Leaflet-based application with collectible tracking, progress persistence (localStorage), community-contributed markers, category filtering, fuzzy search, and a design system with theme switching. Production-quality UX for interactive spatial data.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Leaflet + Marker Clustering** | Spatial data overlay with clustered rendering for density | KB embedding visualization / hypercube spatial index |
| **Progress Tracking (localStorage)** | Client-side state persistence for collectible completion | ExperienceIndex tracking / SelfTest tier progression |
| **Category Filtering + Fuzzy Search** | Multi-dimensional data filtering with search | GWT salience filtering + BM25 search (nt_memory) |
| **Theme System (Light/Dark/Auto)** | CSS variable-based theme with `.dark` class toggle | NT-IO design system (Superbody Minimal light gold) |
| **Community Contribution Model** | Editor mode for adding/editing markers with PR workflow | NT-MEMORY experience absorption (community knowledge) |
| **Pinia State Management** | Centralized reactive state with localStorage persistence | EventBus + KB state layer |
| **Component Design System** | `ui/Btn`, `ui/Panel`, `ui/Dialog`, `ui/Toggle` etc. | NT-IO component library |

### Fusion Opportunities

1. **Spatial Knowledge Index**: NTE-Compass's Leaflet + MarkerCluster pattern maps directly to NT-MEMORY's knowledge visualization. The VSA HyperCube's high-dimensional embeddings need a spatial projection for human browsing — Leaflet's tile-based rendering with clustering is the right pattern for this.

2. **Collectible Progress = Skill Node Progression**: NTE-Compass tracks collectible completion (found/not-found) with localStorage persistence. This is isomorphic to NeoTrix's Constellation maturity tracking (C0→C5). Port the progress-tracking pattern: each skill node has a completion state, auto-saved, filterable by status.

3. **Community Contribution Workflow**: NTE-Compass's editor-mode + PR-based contribution model maps to NT-MEMORY's experience absorption. External knowledge contributors submit via structured format (markers-data.json equivalent), reviewed before KB merge.

4. **Design System Reuse**: NTE-Compass's `ui/` component library (Btn, Panel, Dialog, Toggle, TextInput, EmptyState, TypePill) with CSS variable theming is a clean reference for NT-IO's Tauri desktop UI components.

### Implementation Changes

- `nt_memory`: Implement spatial knowledge index using Leaflet-like tile projection for VSA HyperCube embeddings
- `nt_core_self`: Adopt collectible-tracking pattern for Constellation maturity persistence
- `nt_io`: Build component library following NTE-Compass's `ui/` pattern (Btn, Panel, Dialog, Toggle)
- `nt_memory`: Implement community contribution workflow with structured JSON + review-before-merge

---

## 4. mapeditor/tiled-resources

**URL**: https://github.com/mapeditor/tiled-resources
**Stars**: 23 | **Forks**: 8 | **License**: Public Domain

### Summary
Free tileset resources for the Tiled map editor. Contains RPG and RTS tileset categories with terrain transitions compatible with Tiled's Terrain Brush. Points to OpenGameArt.org and itch.io as additional resource sources.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Terrain Transition Tiles** | Automatic tile-edge blending via Terrain Brush | EmotionLabel smooth transitions between states |
| **Genre-Based Asset Organization** | RPG/RTS top-level categorization | NT-* domain-based asset organization |
| **Tileset Metadata Standards** | Terrain types, tile properties, auto-tile rules | KB node edge metadata / trait annotations |
| **Open Asset Ecosystem** | Curated list of external asset sources (OpenGameArt, itch.io) | NT-WORLD crawler sources / Ordered Backend Router |

### Fusion Opportunities

1. **Terrain Transition → Emotion Transitions**: Tiled's terrain-transition tiles (automatic edge blending between terrain types) map to NeoTrix's emotion state transitions. EmotionLabel variants (Neutral→Joy→Sadness→Anger) should use terrain-transition-like blending — smooth gradients at boundaries rather than hard switches. This aligns with NT-FEEL's emotion regulation engine.

2. **Genre-Based Skill Organization**: Tiled-resources' RPG/RTS top-level organization maps to NT-* domain organization. But NeoTrix could adopt a finer-grained taxonomy: within each domain, skills are organized by "terrain type" (acquisition, transformation, consumption, generation) — each terrain type has its own transition rules.

3. **Open Asset Sources → NT-WORLD Crawler Sources**: The curated list of external asset sources (OpenGameArt, itch.io) informs NT-WORLD's Ordered Backend Router for knowledge sources. Each source should have: name, URL pattern, content type, reliability tier, and fallback order.

### Implementation Changes

- `nt_feel`: Implement terrain-transition-style emotion blending (smooth gradient between EmotionLabel states)
- `nt_world`: Add curated knowledge source registry following tiled-resources' external-source pattern
- `nt_memory`: Organize skill nodes by "terrain type" (acquisition/transformation/consumption/generation) within each domain

---

## 5. darkrishabh/agent-skills-eval

**URL**: https://github.com/darkrishabh/agent-skills-eval
**Stars**: 733 | **Forks**: 35 | **License**: MIT

### Summary
Test runner for [agentskills.io](https://agentskills.io)-style AI agent skills. Runs each eval twice — once `with_skill` loaded into context, once `without_skill` (baseline) — has a judge model grade both outputs, produces side-by-side comparison report. Implements full agentskills.io specification: SKILL.md validation, evals/evals.json, artifact layout.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **With/Without Baseline Comparison** | Same prompt → skill vs no-skill → judge-graded output | SelfTest T1-T3 tiers (existence → registration → production wiring) |
| **Judge-Graded Outputs** | Independent model grades both outputs against assertions | ConsciousnessTree health scoring |
| **Artifact Layout (iteration-N)** | Structured workspace: meta.json, benchmark.json, eval outputs | ExperienceIndex cycle artifacts |
| **OpenAI-Compatible Provider** | Single `Provider` interface (5 fields, 1 method) for any backend | NT-IO LLM provider abstraction |
| **SKILL.md Validation** | Strict schema: name, description, license, compatibility | SKILL-SPEC.md contract (<200 lines) |
| **Static HTML Reports** | Self-contained reports from disk artifacts | NT-IO report generation |
| **Custom Provider Interface** | Implement `Provider` for local/custom backends | NT-IO ordered backend router |
| **Tool-Call Assertions** | Deterministic checks for agent tool usage | NT-ACT tool-call validation |
| **JSONL Event Streaming** | Structured events for downstream analysis | EventBus event format |

### Fusion Opportunities

1. **Skill Evaluation → SelfTest Enhancement**: agent-skills-eval's `with_skill`/`without_skill` baseline comparison is exactly what NeoTrix's SelfTest T3 tier needs. Currently T3 requires "production wiring" (the detection function is called by non-test code). agent-skills-eval shows how to make this measurable: run the same task with and without a skill, grade both outputs, produce a lift report. This turns SelfTest from a binary pass/fail into a quantitative skill-effectiveness metric.

2. **Provider Interface → NT-IO LLM Router**: agent-skills-eval's `Provider` interface (5 fields: name, model, baseUrl, apiKey, complete()) is cleaner than most LLM client abstractions. NT-IO should adopt this minimal interface for its ordered backend router, where each provider is a `Provider` implementation and the router chains them with fallback.

3. **Artifact Layout → Experience Index**: The `iteration-N/<eval>/<mode>/outputs` artifact structure maps to NeoTrix's experience-tree branch structure. Each SEAL cycle should produce `iteration-N/<phase>/<domain>/outputs` artifacts with the same metadata pattern (meta.json, benchmark.json, timing.json).

4. **Judge-Graded Self-Reflection**: The judge-model pattern (independent model grades outputs against assertions) maps to NT-META's meta-cognition. Each SEAL phase should have an independent "judge" that evaluates phase outputs against expected assertions — this is the meta-audit dimension (D37-D40).

5. **JSONL Events → EventBus Enhancement**: agent-skills-eval's JSONL event streaming (structured events with timing, tokens, cost) should inform NT-IO's EventBus format. Events should be structured JSONL with: event_type, timestamp, source_module, payload, timing_ms, tokens_used.

### Implementation Changes

- `nt_core_self`: Implement with/without baseline comparison for SelfTest T3 — run task with skill loaded vs without, grade both
- `nt_io`: Adopt `Provider` interface (5-field minimal) for LLM router
- `nt_memory`: Use `iteration-N/<phase>/<domain>/` artifact layout for experience-tree branches
- `nt_meta`: Add judge-graded phase evaluation (independent model grades SEAL phase outputs)
- `nt_io`: Enhance EventBus to JSONL-structured format with timing/token metadata

---

## Cross-Cutting Fusion Matrix

| Source | RogueElements | Game-Map-Maker | NTE-Compass | tiled-resources | agent-skills-eval |
|--------|--------------|----------------|-------------|-----------------|-------------------|
| **NT-CORE** | GenStep → SEAL phases | — | — | — | — |
| **NT-MIND** | GenStep composability | — | — | — | Skill evaluation baseline |
| **NT-MEMORY** | — | Text serialization | Spatial index + contribution | — | Artifact layout |
| **NT-WORLD** | — | — | Leaflet clustering | Crawler sources | — |
| **NT-ACT** | — | Mode state machine | — | — | Tool-call assertions |
| **NT-IO** | — | Canvas zoom/pan | Design system + themes | — | Provider interface + JSONL |
| **NT-FEEL** | — | — | — | Terrain transitions | — |
| **NT-SHIELD** | — | — | — | — | — |
| **NT-META** | — | — | — | — | Judge-graded evaluation |

## Priority Recommendations

| Priority | Change | Source | Impact |
|----------|--------|--------|--------|
| **P0** | SEAL phases as GenStep pipeline with diagnostic snapshots | RogueElements | Architecture: phases become composable, debuggable |
| **P0** | SelfTest T3 with/without baseline comparison | agent-skills-eval | Quality: skills become measurably effective |
| **P1** | Provider interface (5-field minimal) for LLM router | agent-skills-eval | NT-IO: cleaner provider abstraction |
| **P1** | Spatial knowledge index (Leaflet-like tile projection) | NTE-Compass | NT-MEMORY: human-browsable VSA HyperCube |
| **P1** | Component library (Btn/Panel/Dialog/Toggle) | NTE-Compass | NT-IO: design system foundation |
| **P2** | Emotion terrain-transition blending | tiled-resources | NT-FEEL: smooth state transitions |
| **P2** | Text-based KB export format | Game-Map-Maker | NT-MEMORY: human-readable persistence |
| **P2** | Mode state machine for agent workflows | Game-Map-Maker + NTE-Compass | NT-ACT: explicit mode switching |
| **P3** | Community contribution workflow (JSON + review) | NTE-Compass | NT-MEMORY: external knowledge ingestion |
| **P3** | Crawler source registry (name/url/type/tier/fallback) | tiled-resources | NT-WORLD: structured source management |
