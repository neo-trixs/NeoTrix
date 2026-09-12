# nt-world-sim Gap Analysis: Stardew Valley-Style Game

## Executive Summary

The engine has a solid foundation (ECS, scene graph, event bus, input, physics trait, audio trait, codegen) but is **~15% complete** for a Stardew Valley clone. The existing code is primarily a "Clawd on Desk" desktop pet engine repurposed as a game engine. Most game-critical systems are stubs or missing entirely.

**Verified file-by-file audit** (all 20+ source files read):
- `core/world.rs` (419 lines) — archetype-based ECS, functional
- `core/scheduler.rs` (174 lines) — wave scheduler, sequential within waves
- `core/entity.rs` (207 lines) — archetype + chunk storage
- `core/change_detection.rs` — `Changed<T>` wrapper
- `ecs/world.rs` (210 lines) — flat HashMap ECS (parallel to core/)
- `ecs/system.rs` (142 lines) — stub systems only
- `engine/renderer.rs` (367 lines) — trait defined, `CanvasRenderer` all `// TODO`
- `engine/physics.rs` (367 lines) — functional AABB + impulse resolution
- `engine/input.rs` (304 lines) — complete keyboard/mouse/gamepad
- `engine/scene.rs` (296 lines) — functional scene graph with hierarchy
- `engine/event_bus.rs` (212 lines) — typed event bus with priority
- `engine/events.rs` (132 lines) — event types (Move, Collision, Input, Game)
- `engine/audio.rs` (176 lines) — AudioManager + AudioBackend trait (no real backend)
- `engine/pet_state.rs` (447 lines) — Clawd desktop pet FSM (12 states)
- `engine/hook_system.rs` (338 lines) — Clawd hook/permission system
- `engine/theme_system.rs` (424 lines) — Clawd theme/animation system
- `mechanics/mod.rs` (293 lines) — ConsciousnessEntity, TransformComponent, RenderComponent, AiComponent, MaslowNeeds, SocialRelationship, EconomyComponent + 3 systems
- `mechanics/core_pet.rs` (230 lines) — Clawd pet state component
- `mechanics/core_hook.rs` (226 lines) — Clawd hook event processing
- `mechanics/core_theme.rs` (159 lines) — Clawd theme resource
- `builder.rs` (240 lines) — GameBuilder with StardewValley template (placeholder only)

---

## What Exists vs What's Needed

### ECS Layer — PARTIAL (two parallel systems, need consolidation)

| Feature | Status | Notes |
|---------|--------|-------|
| Entity/Component/System | `core/world.rs` — archetype-based `UniversalWorld` | Full CRUD, multi-component queries (2-tuple), archetype tracking with `HashSet<TypeId>` |
| Entity Model | `core/entity.rs` — `UniversalEntity { id, generation, archetype }` + `Archetype` + `Chunk` | Generational IDs, chunk-based storage defined but unused |
| Simple ECS | `ecs/world.rs` — flat `HashMap<TypeId, ComponentStorage>` | Simpler, blanket `impl<T: Any> Component for T`, used by `mechanics/` layer |
| Parallel Scheduler | `core/scheduler.rs` — wave-based topological sort via `SystemDependency` | Sequential within waves (rayon stub at line 125-127) |
| Simple Scheduler | `ecs/system.rs` — `SystemScheduler` | Priority-sorted sequential execution |
| Change Detection | `core/change_detection.rs` — `Changed<T>` wrapper | Tick-based dirty tracking, not wired into any system |
| Component Queries | `world.query::<(T1, T2)>()` — tuple-based | Only 2-component tuples; 3+ component queries missing |
| Resources | Both ECS have global resource storage (`HashMap<TypeId, Box>`) | `core/` has `insert_resource<T>`, `ecs/` identical API |

**Gap:** Two ECS implementations (`ecs/` and `core/`) that don't interop. `mechanics/` uses `ecs::World` while `builder.rs` and `main.rs` use `core::UniversalWorld`. Must consolidate to `core/` (archetype-based) before building game systems. The `ecs/` `System` trait signature (`fn update(&mut self, world: &mut World, dt: f32)`) differs from `core/`'s `UniversalSystem` trait (`fn update(&mut self, world: &mut UniversalWorld, dt: f32)`), making migration non-trivial.

### Engine Layer — STUBS

| Feature | Status | What's Needed |
|---------|--------|---------------|
| **Renderer** | `engine/renderer.rs:290-317` — `Renderer` trait defined (clear, draw_sprite, draw_tilemap, draw_text, draw_rect, draw_circle, draw_line, present). `CanvasRenderer` at line 320 — all 8 methods are `// TODO: 实现...` | Actual sprite batch rendering, texture atlas, tilemap renderer, animation player, lighting. **Zero rendering capability exists.** |
| **Physics** | `engine/physics.rs` — `SimplePhysicsWorld` with AABB collision detection (O(n²) brute force), impulse-based resolution, gravity, friction. Functional but basic. | Top-down collision (gravity not needed for Stardew), tile collision, interaction triggers, trigger zones, no spatial partitioning |
| **Input** | `engine/input.rs` — Complete `SimpleInputProvider` with keyboard (all keys), mouse (position/delta/scroll/buttons), gamepad (axes + buttons). `clear_frame()` for per-frame resets. | Complete. Only needs key rebinding system and input mapping. |
| **Scene Graph** | `engine/scene.rs` — `SceneGraph` with parent-child hierarchy, `update_transforms()` recursive world-transform propagation, `visible_nodes()` z-sorted. 296 lines, 12 tests passing. | Needs camera follow, parallax layers, render layers (ground/objects/effects/UI) |
| **Event Bus** | `engine/event_bus.rs` — `TypedEventBus` with priority handlers, queue-based processing. Plus `engine/events.rs` — `EventBus` (older, per-type queues). Two event systems coexist. | Consolidate to one. Needs event recording for replays, deferred events |
| **Audio** | `engine/audio.rs` — `AudioManager` with `AudioBackend` trait (load/play/stop/set_volume/set_position/is_playing). `StubAudioBackend` returns `Ok(())` for everything. No real audio. | Needs actual backend (`rodio`/`kira`), music crossfade, ambient system, spatial audio |
| **TileMap** | `engine/renderer.rs:218-253` — `TileMap { tiles: Vec<Vec<u32>>, tile_size, palette }` with `is_walkable()`. Single layer only. | Needs multi-layer (ground/decoration/object/water), chunks for large maps, save format, tile animations |
| **Camera** | `engine/renderer.rs:257-287` — `Camera { position, zoom, viewport }` with `world_to_screen()` / `screen_to_world()`. | Needs follow target with dead zone, screen shake, zoom regions, area transitions |
| **SpriteSheet** | `engine/theme_system.rs:280-318` — `SpriteSheet` with frame rect calculation. `AnimationPlayer` with play/stop/pause. | Designed for desktop pet, needs generalization for game sprites |

### Game Mechanics Layer — MINIMAL

| Feature | Status | What's Needed |
|---------|--------|---------------|
| **Player Movement** | `MovementSystem` at `ecs/system.rs:77-91` — stub (`// TODO: 实现移动逻辑`). `TransformComponent` at `mechanics/mod.rs:31-46` — has position, velocity, facing (u8) | WASD/arrow movement, 8-directional, tool use, animation state machine |
| **NPC AI** | `AiSystem` at `mechanics/mod.rs:257-293` — random state flip (`idle→exploring` at 1% chance/frame), simple move-to-target. No pathfinding, no schedules. | Daily schedules, A* pathfinding, social behaviors, dialogue triggers, schedule system |
| **Economy** | `EconomyComponent` at `mechanics/mod.rs:168-210` — `gold: i32` + `inventory: Vec<(String, i32)>`. Basic add/remove/count. No item database, no stack limits. | Full inventory system, item database, shops, shipping bin, stack limits, item categories |
| **Needs** | `MaslowNeeds` at `mechanics/mod.rs:84-139` — 5-tier hierarchy (physiological/safety/social/esteem/self_actualization). Increases over time, `satisfy()` decreases. | Player energy, hunger, social needs driving gameplay. Current impl is abstract, not game-ready |
| **Social** | `SocialRelationship` at `mechanics/mod.rs:142-165` — `trust`, `affinity`, `interactions`, `relationship_type` (string). Static, no system updates it. | Gift system, marriage, friendship levels, events, daily conversation limits |
| **Consciousness** | `ConsciousnessEntity` at `mechanics/mod.rs:10-28` — `phi`, `coherence`, `health`, `energy`. `ConsciousnessSystem` slowly increments phi/coherence. | **Not needed for Stardew Valley.** NeoTrix-specific. Should be behind feature flag or removed. |
| **Pet State** | `CorePetState` at `mechanics/core_pet.rs` — 12-state FSM for Clawd desktop pet. `PetStateSystem`, `EyeTrackingSystem`, `PermissionBubbleSystem`, `SessionSystem` — all Clawd-specific. | **Replace entirely.** These systems serve a desktop pet, not a game. Keep only reusable patterns (FSM, priority transitions). |

---

## Missing Systems (Critical for Stardew Valley)

### P0 — Core Gameplay Loop (blocking)

| System | Description | Components Needed | Systems Needed |
|--------|-------------|-------------------|----------------|
| **Inventory** | Grid-based inventory, item stacking, tool slots, hotbar | `Inventory { slots: Vec<Slot>, capacity, hotbar_index }`, `Item { id, name, stack_size, item_type }` | `InventorySystem`, `ItemUseSystem` |
| **Farming** | Till soil → plant seed → water → grow → harvest | `Farmland { tilled, watered, crop_id }`, `Crop { growth_stage, days_to_grow, season }` | `FarmingSystem`, `GrowthSystem`, `HarvestSystem` |
| **Time/Day Cycle** | Day phases (morning/afternoon/evening/night), seasons, year | `GameClock { day, season, year, time_of_day, time_speed }` | `TimeSystem`, `SeasonSystem` |
| **Tool System** | Hoe, watering can, pickaxe, axe, scythe, fishing rod | `Tool { tool_type, level, energy_cost, cooldown }`, `ToolAction { target_tile, action_type }` | `ToolSystem`, `ToolEffectSystem` |
| **Energy/Stamina** | Player energy depleted by actions, restored by food/sleep | `Stamina { current, max, regen_rate }` | `StaminaSystem` |

### P1 — World & NPCs

| System | Description | Components Needed | Systems Needed |
|--------|-------------|-------------------|----------------|
| **World Generation** | Farm layout, village, mines, forest, beach | Procedural/seed-based tilemap generation | `WorldGenSystem` |
| **NPC System** | Villagers with schedules, relationships, gift preferences | `Npc { name, schedule, location, dialogues }`, `NpcSchedule { time_blocks }` | `NpcScheduleSystem`, `NpcInteractionSystem` |
| **Dialogue System** | Branching dialogue trees, relationship-gated options | `DialogueTree { nodes, current_node }`, `DialogueChoice { text, condition }` | `DialogueSystem`, `DialogueUISystem` |
| **Pathfinding** | A* on tilemap, NPC daily routines | `PathfindingGrid`, `PathRequest` | `PathfindingSystem` |
| **Collision** | Tile-based collision for walls, water, fences | `TileCollider { collision_type }` | `TileCollisionSystem` |

### P2 — Economy & Progression

| System | Description | Components Needed | Systems Needed |
|--------|-------------|-------------------|----------------|
| **Crafting** | Recipe-based crafting, workbench categories | `Recipe { ingredients, result, workstation }`, `CraftingStation` | `CraftingSystem` |
| **Shop System** | Buy/sell, rotating stock, friendship discounts | `Shop { inventory, prices, stock_refresh }` | `ShopSystem`, `EconomySystem` |
| **Fishing** | Cast line, bite detection, minigame, fish database | `FishingSpot { fish_pool, difficulty }`, `Fish { id, difficulty, value }` | `FishingSystem`, `FishingMinigame` |
| **Mining** | Mine floors, ore nodes, monster encounters | `MineFloor { depth, tiles, enemies }`, `OreNode { ore_type, health }` | `MiningSystem` |
| **Combat** | Weapon swings, enemy AI, health/damage | `Health { current, max }`, `Damage { amount, type }`, `Enemy { ai_state }` | `CombatSystem`, `EnemyAISystem` |

### P3 — Seasons & Events

| System | Description | Components Needed | Systems Needed |
|--------|-------------|-------------------|----------------|
| **Seasons** | 4 seasons affecting crops, fish, forage, aesthetics | `Seasonal { season, growth_modifier }` | `SeasonalModifierSystem` |
| **Weather** | Sunny, rainy, stormy, snowy — affects gameplay | `Weather { current, forecast, duration }` | `WeatherSystem` |
| **Calendar/Events** | Festivals, birthdays, holidays | `Calendar { day, season, events }` | `CalendarSystem` |
| **Foraging** | Wild plants, trees, seasonal spawns | `ForageSpawn { item, season, spawn_rate }` | `ForageSystem` |

### P4 — UI Components (ALL MISSING)

| Component | Description |
|-----------|-------------|
| **HUD** | Health bar, energy bar, clock, gold display, current tool |
| **Inventory UI** | Grid display, item tooltips, drag-drop, equip |
| **Dialogue UI** | Text box, portrait, choice buttons, relationship indicator |
| **Crafting UI** | Recipe list, ingredient check, craft button |
| **Shop UI** | Buy/sell tabs, item list, price display, wallet |
| **Calendar UI** | Month view, event markers, day selector |
| **Map UI** | Minimap, location markers, fog of war |
| **Menu System** | Pause, settings, save/load, exit |
| **Toolbar/Hotbar** | Bottom bar with selected tool highlight |

### P5 — Rendering Pipeline (ALL STUBS)

| Feature | Description |
|---------|-------------|
| **Sprite Renderer** | Batch sprite rendering with texture atlases |
| **Animation System** | Sprite sheet playback, state machines, blending |
| **Tilemap Renderer** | Multi-layer tilemap with frustum culling |
| **Lighting** | Day/night cycle, torch/lamp light sources, ambient |
| **Particle System** | Crop growth, weather, tool effects, UI feedback |
| **Screen Transitions** | Fade, wipe, slide between areas |
| **Camera System** | Smooth follow, screen shake, zoom regions |

### P6 — Save/Load (MISSING)

| Feature | Description |
|---------|-------------|
| **Serialization** | ECS state → JSON/bincode for all entities + components |
| **World Persistence** | Tilemap, crops, buildings, NPC states |
| **Save Slots** | Multiple save files, autosave, version migration |
| **Import/Export** | Farm sharing |

---

## Architectural Issues

### 1. Dual ECS Problem
- `ecs/world.rs` — flat `HashMap<TypeId, ComponentStorage>` with blanket `impl<T: Any> Component for T`
- `core/world.rs` — archetype-based `UniversalWorld` with `Component: Clone + Send + Sync` bound
- `mechanics/` layer imports from `crate::ecs::{System, World}` (flat ECS)
- `builder.rs`, `lib.rs` `create_stardew_valley_game()` use `core::UniversalWorld`
- Entity IDs are incompatible: `ecs::Entity { id: u32, generation: u32 }` vs `core::UniversalEntity { id: EntityId(u64), generation: u32, archetype: ArchetypeId }`
- **Fix:** Delete `ecs/`, migrate all `mechanics/` code to `core/UniversalWorld`. Update `System` trait to accept `&mut UniversalWorld`.

### 2. CanvasRenderer is Empty
Every method in `CanvasRenderer` (lines 331-367) is `// TODO: 实现...`. The engine cannot render anything.
- **Fix:** Implement with `macroquad` (simpler) or `wgpu` (more control). Must support: sprite batching, texture atlases, tilemap rendering, text rendering, basic shapes.

### 3. No Asset Pipeline
No texture loading, no font rendering, no sprite sheet management. `SpriteSheet` struct exists in `theme_system.rs` but only calculates frame rects — no actual texture loading.
- **Fix:** Add `AssetManager` with async loading, handle caching, sprite atlas packing.

### 4. Mechanics Are Desktop-Pet-Specific
- `core_pet.rs` — 12-state FSM (Idle/Thinking/Typing/Building/Groove/Juggling/Error/Happy/Notification/Sweeping/Carrying/Sleeping)
- `core_hook.rs` — `CoreHookEvent` processing (SessionStart/End, ToolStart/End, PermissionRequest/Response)
- `core_theme.rs` — Theme resource with animation paths for pet states
- `pet_state.rs` — Full pet animation system (PetState, EyeTracking, PermissionBubble, SessionInfo, ZzzParticle)
- `hook_system.rs` — `HookManager`, `HookConfig`, `PermissionRequest`, `PermissionBubbleLayout`
- `theme_system.rs` — `ThemeConfig`, `ThemeManager`, `ThemeVariant` (Clawd/Calico/Cloudling)
- **Keep:** `EconomyComponent`, `MaslowNeeds`, `SocialRelationship`, FSM pattern, priority-based transitions
- **Replace:** All pet/hook/theme systems with Stardew-specific mechanics (farming, inventory, dialogue, etc.)

### 5. Codegen Targets Wrong Engines
`codegen/` generates Bevy/Unity/Godot stubs but the engine itself doesn't use any of them. The `GameDefinition` struct and `CodeGenerator` are disconnected from the actual engine.
- **Fix:** Either use Bevy as the actual backend (unify with `core/` ECS), or remove codegen and build native rendering with macroquad/wgpu.

### 6. Two Event Systems
- `engine/event_bus.rs` — `TypedEventBus` with `GameEvent` trait + `GameEventHandler` trait (priority-sorted)
- `engine/events.rs` — `EventBus` with per-type `VecDeque` queues + `EventHandler` trait (takes `&mut World`)
- Both are registered in `lib.rs` exports. `GameEngine` uses `EventBus` (old one).
- **Fix:** Consolidate to `TypedEventBus` (more mature, priority support).

### 7. Builder Template is Placeholder
`GameBuilder::stardew_valley()` at `builder.rs:148-154` only sets window size (1200x800), background color, and title. No systems, no components, no tilemap — just a shell.
- **Fix:** `stardew_valley()` should register all game systems, create player entity with components, load starter tilemap.

---

## Priority Implementation Order

### Phase 1: Foundation (Week 1-2)
1. **Consolidate ECS** — Delete `ecs/`, migrate to `core/UniversalWorld`
2. **Implement Renderer** — `macroquad` or `wgpu` backend with sprite + tilemap
3. **Asset Pipeline** — Texture/font loading, sprite atlas
4. **Player Movement** — WASD movement, tile collision, camera follow

### Phase 2: Core Loop (Week 3-4)
5. **Inventory System** — Grid inventory, hotbar, item definitions
6. **Tool System** — Hoe, watering can, axe, pickaxe, scythe
7. **Farming System** — Till, plant, water, grow, harvest
8. **Time/Day Cycle** — Clock, day phases, sleep mechanic
9. **Energy System** — Stamina, food consumption, sleep restore

### Phase 3: World (Week 5-6)
10. **World Generation** — Farm map, village, basic tilemap
11. **NPC System** — Villagers with positions, daily schedules
12. **Dialogue System** — Branching dialogue, gift giving
13. **Pathfinding** — A* for NPC movement

### Phase 4: Economy (Week 7-8)
14. **Shop System** — Buy/sell, Pierre's shop, traveling cart
15. **Crafting System** — Workbench, recipes
16. **Fishing System** — Cast, bite, minigame
17. **Save/Load** — Serialize game state

### Phase 5: Polish (Week 9-10)
18. **Seasons** — Visual changes, crop modifiers
19. **Weather** — Rain, snow, effects
20. **UI Overhaul** — All menus, HUD, dialogue boxes
21. **Audio** — Background music, SFX, ambient
22. **Lighting** — Day/night, indoor/outdoor

---

## Recommended Engine Stack

| Layer | Recommendation | Rationale |
|-------|---------------|-----------|
| **Rendering** | `macroquad` or `bevy` | Macroquad for simplicity, Bevy for ECS integration |
| **Audio** | `rodio` or `kira` | Cross-platform, spatial audio |
| **Serialization** | `serde` + `bincode` | Fast save/load |
| **UI** | `egui` (via macroquad/bevy) or custom | Immediate mode for dev, custom for production |
| **Tilemap** | Custom with `TileMap` struct | Stardew-specific multi-layer needs |
| **Physics** | Custom tile collision | 2D top-down doesn't need full physics engine |

---

## Estimated Effort

| Phase | Weeks | Scope |
|-------|-------|-------|
| Foundation | 2 | ECS consolidation, renderer, assets, movement |
| Core Loop | 2 | Inventory, tools, farming, time, energy |
| World | 2 | Map gen, NPCs, dialogue, pathfinding |
| Economy | 2 | Shops, crafting, fishing, save/load |
| Polish | 2 | Seasons, weather, UI, audio, lighting |
| **Total** | **10 weeks** | Playable Stardew Valley clone |

---

## Conclusion

The engine has the **right abstractions** (ECS, scene graph, event bus, physics trait, audio trait) but lacks **any concrete implementations** for game-specific systems. The existing mechanics are desktop-pet-specific and need replacement. The biggest blocker is the empty `CanvasRenderer` — nothing can be displayed.

**Immediate next steps:**
1. Delete `ecs/` module, consolidate to `core/`
2. Implement `CanvasRenderer` with a real graphics backend (macroquad recommended for speed)
3. Build inventory + farming as the MVP core loop

---

## Appendix: File-by-File Summary

| File | Lines | Purpose | Stardew-Ready? |
|------|-------|---------|----------------|
| `core/world.rs` | 419 | Archetype-based ECS (UniversalWorld) | Yes — needs 3+ component query support |
| `core/entity.rs` | 207 | Entity/Archetype/Chunk types | Yes — chunk storage unused |
| `core/scheduler.rs` | 174 | Wave-based parallel scheduler | Partial — parallelism is stub |
| `core/change_detection.rs` | — | Changed\<T\> wrapper | Unused, needs wiring |
| `ecs/world.rs` | 210 | Flat HashMap ECS | No — delete, migrate to core/ |
| `ecs/system.rs` | 142 | System trait + stub systems | No — delete, migrate to core/ |
| `engine/renderer.rs` | 367 | Renderer trait + empty CanvasRenderer | No — all TODO stubs |
| `engine/physics.rs` | 367 | AABB physics, impulse resolution | Partial — needs tile collision |
| `engine/input.rs` | 304 | Keyboard/mouse/gamepad input | Yes — complete |
| `engine/scene.rs` | 296 | Hierarchical scene graph | Yes — functional |
| `engine/event_bus.rs` | 212 | Typed event bus with priority | Yes — functional |
| `engine/events.rs` | 132 | Legacy event bus + event types | No — consolidate with event_bus.rs |
| `engine/audio.rs` | 176 | Audio manager + stub backend | No — needs real backend |
| `engine/pet_state.rs` | 447 | Clawd desktop pet FSM | No — game-specific replacement needed |
| `engine/hook_system.rs` | 338 | Clawd hook/permission system | No — game-specific replacement needed |
| `engine/theme_system.rs` | 424 | Clawd theme/animation system | No — generalize for game sprites |
| `mechanics/mod.rs` | 293 | Components + 3 systems | Partial — keep EconomyComponent/MaslowNeeds/SocialRelationship |
| `mechanics/core_pet.rs` | 230 | Clawd pet state | No — replace |
| `mechanics/core_hook.rs` | 226 | Clawd hook events | No — replace |
| `mechanics/core_theme.rs` | 159 | Clawd theme resource | No — replace |
| `builder.rs` | 240 | GameBuilder with templates | No — templates are placeholders |
| `lib.rs` | 290 | Module exports + GameEngine + create_stardew_valley_game() | No — game creation is stub |
| `codegen/` | — | Bevy/Unity/Godot code generators | No — disconnected from engine |

**Total source:** ~4,500 lines across 20+ files
**Game-functional code:** ~800 lines (ECS core, input, physics, scene graph, event bus)
**Desktop-pet code:** ~1,800 lines (pet_state, hook_system, theme_system, core_pet, core_hook, core_theme)
**Stubs/TODOs:** ~1,200 lines (renderer, systems, builder templates)
**Unused/dead code:** ~700 lines (codegen, ecs/, change_detection)
