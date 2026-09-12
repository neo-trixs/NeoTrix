# nt-world-sim Gap Analysis: Stardew Valley-Style Game

## Executive Summary

The engine has a solid foundation (ECS, scene graph, event bus, input, physics trait, audio trait, codegen) but is **~15% complete** for a Stardew Valley clone. The existing code is primarily a "Clawd on Desk" desktop pet engine repurposed as a game engine. Most game-critical systems are stubs or missing entirely.

---

## What Exists vs What's Needed

### ECS Layer — PARTIAL (two parallel systems, need consolidation)

| Feature | Status | Notes |
|---------|--------|-------|
| Entity/Component/System | `core/` has archetype-based ECS with `UniversalWorld` | Full CRUD, multi-component queries, archetype tracking |
| Simple ECS | `ecs/` has flat `HashMap<TypeId, HashMap<u32, Box>>` | Simpler, used by mechanics/ layer |
| Parallel Scheduler | `core/scheduler.rs` — wave-based topological sort | Sequential within waves (parallelism stub) |
| Change Detection | `core/change_detection.rs` — `Changed<T>` wrapper | Tick-based dirty tracking |
| Component Queries | `query::<(T1, T2)>()` — tuple-based | Only 2-component tuples implemented |

**Gap:** Two ECS implementations (`ecs/` and `core/`) that don't interop. Must consolidate to `core/` (archetype-based) before building game systems.

### Engine Layer — STUBS

| Feature | Status | What's Needed |
|---------|--------|---------------|
| **Renderer** | `Renderer` trait defined, `CanvasRenderer` all TODO stubs | Actual sprite batch rendering, texture atlas, tilemap renderer, animation player, lighting |
| **Physics** | `SimplePhysicsWorld` — functional AABB collision, impulse resolution | Top-down collision (no gravity needed for Stardew), tile collision, interaction triggers |
| **Input** | `SimpleInputProvider` — full keyboard/mouse/gamepad | Complete. Needs key rebinding system |
| **Scene Graph** | `SceneGraph` — hierarchical transforms, z-sorting | Complete for 2D. Needs camera follow, parallax layers |
| **Event Bus** | `TypedEventBus` — typed, priority handlers | Complete. Needs event recording for replays |
| **Audio** | `AudioManager` + `AudioBackend` trait — structure only | Needs actual backend (rodio/kira), music crossfade, ambient system |
| **TileMap** | `TileMap` struct with palette, walkability | Needs multi-layer (ground/decoration/object), chunks, save format |
| **Camera** | `Camera` with zoom, world↔screen | Needs follow target, dead zone, screen shake, transitions |

### Game Mechanics Layer — MINIMAL

| Feature | Status | What's Needed |
|---------|--------|---------------|
| **Player Movement** | `MovementSystem` (stub) + `TransformComponent` | WASD/arrow movement, 8-directional, tool use, stamina |
| **NPC AI** | `AiSystem` (random walk stub) | Daily schedules, pathfinding (A*), social behaviors, dialogue triggers |
| **Economy** | `EconomyComponent` — gold + `Vec<(String, i32)>` inventory | Full inventory system, item database, shops, shipping bin |
| **Needs** | `MaslowNeeds` — 5-tier hierarchy | Player energy, hunger, social needs driving gameplay |
| **Social** | `SocialRelationship` — trust/affinity | Gift system, marriage, friendship levels, events |
| **Pet State** | `CorePetState` — 12-state FSM (Clawd desktop pet) | This is for a desktop pet, NOT a game. Needs replacement |

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
- `ecs/` (simple flat storage) used by `mechanics/` layer
- `core/` (archetype-based) used by `main.rs` and `builder.rs`
- These don't share entity IDs or component storage
- **Fix:** Delete `ecs/`, migrate all code to `core/UniversalWorld`

### 2. CanvasRenderer is Empty
Every method in `CanvasRenderer` is `// TODO`. The engine cannot render anything.
- **Fix:** Implement with `wgpu` or `macroquad` backend

### 3. No Asset Pipeline
No texture loading, no font rendering, no sprite sheet management.
- **Fix:** Add `AssetManager` with async loading, handle caching

### 4. Mechanics Are Desktop-Pet-Specific
`core_pet.rs`, `core_hook.rs`, `core_theme.rs`, `pet_state.rs`, `hook_system.rs` — all built for a "Clawd on Desk" desktop pet, not a game.
- **Keep:** `EconomyComponent`, `MaslowNeeds`, `SocialRelationship` (reusable)
- **Replace:** All pet/hook/theme systems with Stardew-specific mechanics

### 5. Codegen Targets Wrong Engines
Generates Bevy/Unity/Godot stubs but the engine itself doesn't use any of them.
- **Fix:** Either use Bevy as the actual backend, or remove codegen and build native rendering

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
2. Implement `CanvasRenderer` with a real graphics backend
3. Build inventory + farming as the MVP core loop
