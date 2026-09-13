# NeoTrix Game Engine — Architecture Review & Roadmap

> **Date**: 2026-09-13  
> **Status**: Living document  
> **Scope**: `nt-world-sim` crate — full-stack game engine within NeoTrix

---

## 1. Current State Assessment

### What We Have (Accumulated)

#### Rust Engine (src/)
| Module | Files | Purpose |
|--------|-------|---------|
| `engine/` | 17 `.rs` | Core: ECS, renderer, camera, input, audio, event bus, dialogue, quest, NPC, resources, physics, particles, debug overlay, asset server, sprite batch, scene graph |
| `game/` | 27 `.rs` | Game systems: inventory, time, season, energy, farming, crafting, weather, quest, events, economy, shopping, artisan, recipes, skill, mining, combat, crops, perfection, relationship, cooking, fishing, foraging, NPC, dialogue, unified game world, game loop |
| `world/` | 5 `.rs` | World generation: tile, tile presets, generator, zone, pathfinding |
| `adapters/` | 4 `.rs` | Engine adapters: Bevy, Godot, Unity, Unreal |
| `codegen/` | 3 `.rs` | Code generation: parser, generator, game definition |
| `save/` | 1 `.rs` | Save system (stub) |
| `main.rs` | 1 | CLI/Tauri entry point |
| `tauri_bridge.rs` | 1 | Tauri ↔ engine bridge |
| `lib.rs` | 1 | Library root |

**Total**: ~60 source files across 7 directories.

#### Data Assets
| Category | Files | Content |
|----------|-------|---------|
| Tilesets | 3 JSON | `terrain.json`, `objects.json`, `characters.json` |
| Dialogues | 3 JSON | `elder.json`, `merchant.json`, `guard.json` |
| Quests | 2 JSON | `main_quest_01.json`, `side_quest_01.json` |
| Examples | 3 YAML | `stardew_valley.yaml`, `consciousness_valley.yaml`, `minimal_game.yaml` |

#### Documentation
23 docs in `docs/` covering: architecture research, RPG patterns, HTML5 game analysis, graphics/rendering, controls, systems, dialogue/quest design, tileset resources, map editor, MIR2 source analysis, Guigubahuang research, isometric rendering, and art style.

#### External References (docs/)
- `GAME_ENGINE_ARCHITECTURE.md` — engine blueprints
- `HTML5_RPG_PATTERNS.md` — HTML5 game patterns analyzed
- `MIR2_SOURCE_CODE.md` — MIR2 game source analysis
- `GUIGUBAHUANG_RESEARCH.md` — Chinese RPG research
- `RPG_ARCHITECTURE_RESEARCH.md` — RPG engine patterns
- `CROSS_ENGINE_FUSION.md` — cross-engine pattern synthesis

#### Build & Integration
- Cargo workspace member (`nt-world-sim`)
- Optional Tauri 2.0 integration (`tauri` feature)
- `neotrix-sim` dependency (simulation framework)
- Benchmarks in `benches/`

---

### Redundancy Analysis

#### Duplicate Systems Identified

| System | Locations | Issue |
|--------|-----------|-------|
| **Game Loop** | `engine::core::GameEngine`, `game::game_loop::GameLoop` | Two separate game loop implementations; `core` has fixed-timestep + state stack, `game_loop` has its own `GameState` enum |
| **GameState** | `engine::core::GameState`, `game::game_loop::GameState` | Two conflicting state enums — core has 8 states (Loading/Title/Creating/Playing/Paused/Dialogue/Combat/Cutscene), game_loop has its own |
| **Dialogue** | `engine::dialogue`, `game::dialogue` | Two dialogue systems; engine version is a tree runner, game version likely has game-specific logic |
| **NPC** | `engine::npc`, `game::npc` | Two NPC systems; engine has `NPCManager`, game has its own NPC logic |
| **Quest** | `engine::quest`, `game::quest` | Two quest systems; engine has `QuestManager`, game has its own quest logic |
| **Resources** | `engine::resources`, `engine::asset` | Two resource/asset systems with overlapping responsibilities |
| **Map** | `engine::map` (TileMap + AutoTile + FogOfWar + Minimap), `world::tile` (TileMap + Biome + WorldMap) | Two map data structures; engine version is renderer-ready, world version is for generation |
| **Tile** | `engine::map::Tile`, `world::tile::Tile` | Two Tile types — engine Tile has property/layer system, world Tile has biome/type system |
| **Event Bus** | `engine::event_bus`, `engine::core::CoreEventBus` | Two event bus implementations; core has ring-buffer, engine has separate event_bus module |
| **Input** | `engine::input`, `engine::input_map` | Two input modules — one for raw input, one for mapped bindings |
| **Physics** | `engine::physics` | Exists but may overlap with game-level collision in combat/farming |

#### Deduplication Priority

1. **Critical**: Merge `game_loop` into `engine::core` — single game loop
2. **Critical**: Merge dialogue systems — single dialogue runner
3. **Critical**: Merge NPC systems — single NPC manager
4. **Critical**: Merge quest systems — single quest manager
5. **High**: Unify map/tile types — one Tile type, one TileMap type
6. **High**: Consolidate resource/asset systems
7. **Medium**: Unify event bus implementations
8. **Low**: Input/input_map can coexist (layered design)

---

### Defect Analysis (Gaps)

| Category | Gap | Severity |
|----------|-----|----------|
| **Game Loop** | Two competing implementations; `game_loop.rs` duplicates `core.rs` | Critical |
| **Asset Pipeline** | No hot-reload, no format conversion, no CDN/local abstraction | High |
| **Save/Load** | `save/mod.rs` exists but is a stub; no serialization strategy defined | High |
| **UI Framework** | No UI system; debug overlay exists but no game UI (menus, HUD, inventory screens) | High |
| **Audio** | `engine::audio` module exists but no integration with game systems (no SFX triggers, no music zones) | Medium |
| **Rendering Backend** | `renderer.rs` has `CanvasRenderer` abstraction but no actual wgpu/SDL/WebGL backend | High |
| **Scene Graph** | `engine::scene` exists but unclear integration with ECS | Medium |
| **Testing** | Unit tests in ECS and core; no integration tests, no game system tests | Medium |
| **Error Handling** | `error.rs` exists but most modules use `unwrap()` | Medium |
| **Type Safety** | Dialogue/quest data loaded from JSON with manual parsing; no schema validation | Low |
| **Documentation** | 23 research docs but no API docs, no usage examples, no architecture decision records | Medium |

---

## 2. Target Architecture

### Core Engine Layers

```
┌─────────────────────────────────────────────────────┐
│  L4: Application Layer                              │
│  Game-specific logic, scenes, UI screens            │
│  src/game/ + src/game/game_loop.rs                  │
├─────────────────────────────────────────────────────┤
│  L3: Game Systems Layer                             │
│  Map, Dialogue, Quest, NPC, Inventory, Combat       │
│  src/game/{dialogue,quest,npc,inventory,combat}.rs  │
├─────────────────────────────────────────────────────┤
│  L2: ECS Layer                                      │
│  Entity-Component-System, SystemRunner, World       │
│  src/engine/ecs.rs + src/engine/core.rs             │
├─────────────────────────────────────────────────────┤
│  L1: Platform Layer                                 │
│  Rendering, Input, Audio, Window, File I/O          │
│  src/engine/{renderer,input,audio,asset}.rs         │
└─────────────────────────────────────────────────────┘
```

### Module Dependency Graph

```
                    ┌──────────────┐
                    │   main.rs    │
                    │ tauri_bridge │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │   game_loop  │  (Application Layer)
                    │   unified    │
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
   ┌──────▼──────┐  ┌─────▼──────┐  ┌──────▼──────┐
   │  dialogue   │  │   quest    │  │    npc      │  (Game Systems)
   │  farming    │  │  combat    │  │ inventory   │
   │  crafting   │  │  economy   │  │  skill      │
   └──────┬──────┘  └─────┬──────┘  └──────┬──────┘
          │                │                │
          └────────────────┼────────────────┘
                           │
                    ┌──────▼───────┐
                    │     ecs      │  (ECS Layer)
                    │     core     │
                    │  event_bus   │
                    │  resources   │
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
   ┌──────▼──────┐  ┌─────▼──────┐  ┌──────▼──────┐
   │  renderer   │  │   input    │  │   audio     │  (Platform)
   │  camera     │  │  input_map │  │   asset     │
   │ sprite_batch│  │            │  │             │
   └─────────────┘  └────────────┘  └─────────────┘
```

### Data Flow

```
Input Events ──→ InputState ──→ SystemRunner ──→ ECS World ──→ Renderer
                      │              │               │
                      │         ┌────▼────┐     ┌───▼───┐
                      │         │ Systems │     │Components│
                      │         │ (ordered)│    │ (typed) │
                      │         └────┬────┘     └───┬───┘
                      │              │               │
                      │         ┌────▼────────────┐  │
                      │         │  Game Systems   │  │
                      │         │  dialogue/quest  │  │
                      │         │  npc/inventory   │  │
                      │         └────┬────────────┘  │
                      │              │               │
                      └──────────────┼───────────────┘
                                     │
                              ┌──────▼──────┐
                              │  EventBus   │
                              │ (cross-sys) │
                              └─────────────┘
```

---

## 3. External Knowledge Fusion

### From Tiled Map Editor
- **JSON map format**: Layered tile data with per-tile properties — maps directly to `engine::map::MapLayer` + `TileProperty`
- **Auto-tile rules**: 47-bit bitmask auto-tiling — implemented in `engine::map::AutoTileSystem`
- **Property system**: Custom per-tile/per-layer/per-object properties — maps to `TileProperty`
- **Object layer**: Non-tile entities (NPCs, triggers, spawns) — maps to `world::zone::ZoneConnection`

### From RPG Maker
- **Event system**: Parallel/triggered/autorun events — maps to `game::events::GameEvent`
- **Common events**: Reusable event logic — pattern for `game::events` refactoring
- **Switches/variables**: Global boolean/integer state — maps to `engine::resources::ResourceManager`
- **Troop system**: Enemy group encounters — maps to `game::combat`

### From Godot/Redot
- **Scene tree**: Hierarchical node composition — maps to `engine::scene` (needs expansion)
- **Signal system**: Decoupled observer pattern — maps to `engine::event_bus` + `engine::core::CoreEventBus`
- **Resource system**: Reference-counted cached assets — maps to `engine::resources::TypedResourceManager`
- **AnimationPlayer**: Keyframe animation — maps to `engine::sprite_batch` (needs extension)

### From Bevy ECS
- **Archetype storage**: Component-dense storage for cache efficiency — current `HashMap<TypeId, ComponentVec>` can be upgraded
- **System ordering**: Explicit `.before()`/`.after()` constraints — `System::priority()` is a simplified version
- **Resource injection**: Global resources via `Res<T>`/`ResMut<T>` — maps to `engine::resources::TypedResourceManager`
- **Schedule**: Parallel system scheduling — `SystemRunner` currently runs sequentially

---

## 4. Cross-Domain Pattern Mapping

| Pattern | Source | NeoTrix Mapping | Status |
|---------|--------|-----------------|--------|
| **ECS** | Bevy/DOTS | `engine::ecs` — `World`, `Entity`, `Component`, `System` | Implemented (basic) |
| **Fixed Timestep** | Gaffer on Games | `engine::core::LoopTimer` + `GameEngine::run` | Implemented |
| **State Stack** | Custom | `engine::core::StateStack` — overlay push/pop | Implemented |
| **Tile Map** | Tiled | `engine::map::TileMap` + `AutoTileSystem` + `FogOfWar` | Implemented (engine layer) |
| **World Gen** | Perlin/Simplex | `world::generator::WorldGenerator` | Implemented |
| **Dialogue Tree** | ink/Yarn | `engine::dialogue::DialogueTree` + `DialogueRunner` | Implemented |
| **Quest State** | RPG Maker | `engine::quest::QuestManager` + `Objective` | Implemented |
| **NPC Behavior** | RPG Maker | `engine::npc::NPCManager` + `NPCSchedule` | Implemented |
| **Event Bus** | Godot signals | `engine::event_bus` + `engine::core::CoreEventBus` | Implemented (dual) |
| **Resource Cache** | Godot | `engine::resources::TypedResourceManager` | Implemented |
| **Sprite Batch** | Common | `engine::sprite_batch::SpriteBatch` + `DrawCallBatcher` | Implemented |
| **Pathfinding** | A* | `world::pathfinding::astar` | Implemented |
| **Scene Graph** | Godot | `engine::scene` | Stub only |
| **Asset Pipeline** | Bevy | `engine::asset::AssetServer` | Stub only |
| **Save/Load** | Custom | `save/mod.rs` | Stub only |
| **UI Framework** | — | — | Not started |
| **Audio Integration** | — | `engine::audio` | Stub only |
| **Physics** | — | `engine::physics` | Implemented (basic) |
| **Particles** | — | `engine::particle::ParticleSystem` | Implemented (basic) |

---

## 5. Redundancy Cleanup Plan

### Phase A: Critical Merges (Week 1)

| Action | Source | Target | Notes |
|--------|--------|--------|-------|
| Merge game loop | `game::game_loop` | `engine::core` | Keep `engine::core::GameEngine` as single loop. Delete `game::game_loop.rs` |
| Merge dialogue | `game::dialogue` | `engine::dialogue` | Keep engine version, add game-specific conditions as extension traits |
| Merge NPC | `game::npc` | `engine::npc` | Keep engine version, move game logic (relationships, schedules) into it |
| Merge quest | `game::quest` | `engine::quest` | Keep engine version, add game-specific objective types |

### Phase B: Type Unification (Week 2)

| Action | Source | Target | Notes |
|--------|--------|--------|-------|
| Unify Tile type | `world::tile::Tile` | `engine::map::Tile` | Merge biome data into engine Tile |
| Unify TileMap | `world::tile::WorldMap` | `engine::map::TileMap` | World generation produces engine TileMap |
| Consolidate events | `engine::event_bus` | `engine::core::CoreEventBus` | Keep CoreEventBus for core loop; engine event_bus for game events |
| Consolidate resources | `engine::resources` + `engine::asset` | `engine::asset::AssetServer` | Single asset/resource system |

### Phase C: File Cleanup (Week 2)

| Action | File | Reason |
|--------|------|--------|
| Delete | `game::game_loop.rs` | Merged into engine::core |
| Delete | `game::dialogue.rs` | Merged into engine::dialogue |
| Delete | `game::npc.rs` | Merged into engine::npc |
| Delete | `game::quest.rs` | Merged into engine::quest |
| Archive | `UNIVERSAL_GAME_ENGINE.md` | Superseded by this document |
| Archive | `UNIVERSAL_FUSION.md` | Content folded into ARCHITECTURE_REVIEW.md |
| Archive | `CROSS_ENGINE_FUSION.md` (root) | Content folded into Section 3 |

---

## 6. Core Roadmap

### Phase 1: Engine Core (Week 1–2)

- [ ] **P0**: Resolve dual game loop — single `GameEngine` entry point
- [ ] **P0**: Resolve dual GameState — single `GameState` enum
- [ ] **P0**: Merge dialogue systems — single `DialogueRunner`
- [ ] **P0**: Merge NPC systems — single `NPCManager`
- [ ] **P0**: Merge quest systems — single `QuestManager`
- [ ] **P1**: Unify Tile types — one `Tile` struct with all fields
- [ ] **P1**: Unify TileMap — world gen outputs engine TileMap
- [ ] **P1**: Consolidate event bus — one primary event channel
- [ ] **P2**: Add `System` dependency declarations (ordering constraints)
- [ ] **P2**: Add `World` resource storage (global singletons)

### Phase 2: Rendering (Week 3–4)

- [ ] **P0**: Define rendering backend trait (`RenderBackend`)
- [ ] **P1**: Implement wgpu backend (desktop) or soft-buffer (headless)
- [ ] **P1**: Sprite batch renderer — batch by texture, z-order
- [ ] **P1**: Tilemap renderer — chunk-based, frustum culling
- [ ] **P2**: Camera system — smooth follow, bounds clamping, zoom
- [ ] **P2**: Screen effects — fade, flash, shake
- [ ] **P3**: Particle system integration with game events

### Phase 3: Map System (Week 5–6)

- [ ] **P1**: Tile map data structure — chunk storage, lazy loading
- [ ] **P1**: Auto-tile system — 47-bit bitmask, terrain transitions
- [ ] **P2**: Fog of war — per-tile reveal state
- [ ] **P2**: Minimap — downscaled viewport overlay
- [ ] **P3**: Map editor integration — Tiled JSON import/export
- [ ] **P3**: Zone system — connections, transitions, encounters

### Phase 4: Game Systems (Week 7–8)

- [ ] **P1**: Dialogue system — conditions, branching, variables, callbacks
- [ ] **P1**: Quest system — objectives, tracking, rewards, journal
- [ ] **P1**: NPC system — schedules, dialogue triggers, relationships
- [ ] **P2**: Inventory system — slots, stacking, equipment
- [ ] **P2**: Combat system — turn-based, stats, skills
- [ ] **P2**: Economy — shops, shipping, currency
- [ ] **P3**: Farming — crop lifecycle, seasons, tools
- [ ] **P3**: Crafting — recipes, artisan machines

### Phase 5: Content (Week 9–10)

- [ ] Create tilesets — terrain, objects, characters (SVG → spritesheet)
- [ ] Write dialogue trees — elder, merchant, guard + new NPCs
- [ ] Design quests — main quest chain + side quests
- [ ] Build game world — zones, connections, encounters
- [ ] Define item database — weapons, tools, materials, crops

### Phase 6: Polish (Week 11–12)

- [ ] Audio integration — SFX triggers, music zones, ambient
- [ ] Save/load system — full world serialization, slot management
- [ ] UI framework — menus, HUD, dialogue boxes, inventory screens
- [ ] Performance profiling — ECS query benchmarks, render batch analysis
- [ ] Error handling — replace `unwrap()` with proper error types
- [ ] API documentation — rustdoc for all public types

---

## 7. Task Priority Matrix

| Priority | Task | Effort | Impact | Dependencies |
|----------|------|--------|--------|--------------|
| **P0** | Resolve dual game loop | Medium | Critical | None |
| **P0** | Merge dialogue systems | Medium | Critical | None |
| **P0** | Merge NPC systems | Medium | Critical | None |
| **P0** | Merge quest systems | Medium | Critical | None |
| **P0** | Unify Tile/TileMap types | Medium | High | None |
| **P1** | Rendering backend trait | High | High | P0 cleanup |
| **P1** | Sprite batch renderer | High | High | Rendering trait |
| **P1** | Tilemap renderer | High | High | Sprite batch |
| **P1** | Dialogue conditions/branching | Medium | High | P0 merge |
| **P1** | Quest objectives/rewards | Medium | High | P0 merge |
| **P1** | NPC schedules/relationships | Medium | High | P0 merge |
| **P2** | Camera system | Medium | Medium | Rendering |
| **P2** | Inventory system | Medium | Medium | ECS + UI |
| **P2** | Combat system | High | Medium | ECS + inventory |
| **P2** | Economy/shops | Medium | Medium | Inventory |
| **P2** | System ordering constraints | Low | Medium | ECS |
| **P3** | Map editor integration | High | Medium | Tilemap renderer |
| **P3** | Fog of war | Medium | Low | Tilemap |
| **P3** | Particle system | Low | Low | Renderer |
| **P3** | Audio integration | Medium | Low | Asset pipeline |
| **P3** | Save/load system | High | Medium | Serialization |
| **P3** | UI framework | High | High | Rendering |
| **P3** | Performance optimization | Medium | Medium | Profiling |

---

## 8. Multi-Agent Inspection Plan

### Automated Checks

| Check | Tool | Frequency | Agent |
|-------|------|-----------|-------|
| **Architecture compliance** | `cargo check --all-targets` | Every commit | CI |
| **Module boundary enforcement** | Custom linter (no cross-layer violations) | Every commit | NT-SHIELD |
| **Redundancy detection** | Grep for duplicate type definitions | Weekly | NT-META |
| **Test coverage** | `cargo tarpaulin` or `llvm-cov` | Weekly | NT-META |
| **Documentation coverage** | `cargo doc --no-deps` warnings | Weekly | NT-META |
| **Benchmark regression** | `cargo bench` + threshold alerts | Weekly | NT-MIND |
| **Dead code detection** | `cargo +nightly udeps` | Monthly | NT-REPAIR |
| **Dependency audit** | `cargo audit` | Monthly | NT-SHIELD |
| **API stability** | semver-checks on public API | Per release | NT-GOVERNANCE |

### Review Dimensions (mapped to NeoTrix D1-D50)

| Dimension | Scope | Action |
|-----------|-------|--------|
| D1 (Build) | All crates compile | `cargo check --all-targets` |
| D3 (Modules) | No orphan modules, no dead modules | Module dependency graph audit |
| D5 (Architecture) | Layer violations detected | Custom lint: game layer can't import platform directly |
| D8 (Tests) | Coverage ≥60% for engine, ≥40% for game | `cargo tarpaulin` |
| D17 (SelfTest) | Engine + game systems have SelfTest impls | SelfTest registry check |
| D46 (Review Discipline) | Every PR has architecture review | PR template enforcement |

### Quality Gates

| Gate | Criteria | Enforcement |
|------|----------|-------------|
| **Compile Gate** | `cargo check --all-targets` passes | Pre-commit hook |
| **Test Gate** | `cargo test --lib` passes | Pre-commit hook |
| **Redundancy Gate** | No duplicate type definitions across layers | CI lint |
| **Doc Gate** | All public types have rustdoc | CI warning check |
| **Benchmark Gate** | No >10% regression from baseline | Weekly CI |

---

## Appendix A: File Inventory

### Source Files (by directory)

```
src/
├── main.rs                    (1 file)
├── lib.rs                     (1 file)
├── tauri_bridge.rs            (1 file)
├── error.rs                   (1 file)
├── builder.rs                 (1 file)
├── engine/                    (17 files)
│   ├── mod.rs, core.rs, renderer.rs, ecs.rs
│   ├── input.rs, input_map.rs, audio.rs
│   ├── camera.rs, asset.rs, resources.rs
│   ├── event_bus.rs, scene.rs, map.rs
│   ├── dialogue.rs, quest.rs, npc.rs
│   ├── sprite_batch.rs, particle.rs, physics.rs
│   └── debug_overlay.rs
├── game/                      (27 files)
│   ├── mod.rs, game_loop.rs, unified.rs
│   ├── inventory.rs, time.rs, season.rs, energy.rs
│   ├── item.rs, dialogue.rs, npc.rs
│   ├── farming.rs, crafting.rs, weather.rs
│   ├── quest.rs, events.rs, economy.rs
│   ├── shopping.rs, artisan.rs, recipes.rs
│   ├── skill.rs, mining.rs, combat.rs
│   ├── crops.rs, perfection.rs, relationship.rs
│   ├── cooking.rs, fishing.rs, foraging.rs
│   └── ...
├── world/                     (5 files)
│   ├── mod.rs, tile.rs, tile_presets.rs
│   ├── generator.rs, zone.rs, pathfinding.rs
├── adapters/                  (5 files)
│   ├── mod.rs, bevy_adapter.rs, godot_adapter.rs
│   ├── unity_adapter.rs, unreal_adapter.rs
├── codegen/                   (3 files)
│   ├── mod.rs, parser.rs, generator.rs, game_def.rs
└── save/                      (1 file)
    └── mod.rs
```

### Data Files

```
data/
├── dialogues/
│   ├── elder.json
│   ├── merchant.json
│   └── guard.json
└── quests/
    ├── main_quest_01.json
    └── side_quest_01.json

assets/
└── tilesets/
    ├── terrain.json
    ├── objects.json
    └── characters.json
```

---

## Appendix B: Key Design Decisions

| Decision | Rationale | Alternatives Considered |
|----------|-----------|------------------------|
| **HashMap ECS over Archetype** | Simpler implementation; adequate for <10K entities | Bevy-style archetype (faster iteration, more complex) |
| **Fixed timestep at 60Hz** | Standard for 2D RPGs; physics stability | Variable timestep (simpler, less predictable) |
| **State stack over FSM** | Allows overlay stacking (pause menu over gameplay) | Flat FSM (simpler, no overlays) |
| **JSON data files** | Human-readable, Tiled-compatible | RON (Rust-native, less tooling), Binary (fast, not editable) |
| **Optional Tauri** | Desktop app is optional; engine can run headless | Required Tauri (tighter coupling, easier desktop) |
| **Separate engine/game layers** | Engine is reusable; game logic is specific | Monolithic (simpler, less flexible) |
