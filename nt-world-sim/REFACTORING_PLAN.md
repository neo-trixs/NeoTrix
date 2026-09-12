# nt-world-sim Refactoring Plan — Stardew Valley Game Engine

## Current State Analysis

### File Inventory (35 source files, ~5,500 LOC)

```
src/
├── lib.rs                    (290 LOC) GameEngine + re-exports + create_stardew_valley_game()
├── main.rs                   (153 LOC) CLI demo entry — uses core/ ECS only
├── builder.rs                (240 LOC) GameBuilder + BuiltGame (alt engine, uses core/ ECS)
├── adapters/                 (4 files) Engine adapters — thin wrappers, no real logic
├── codegen/                  (3 files) YAML/JSON → Bevy/Unity/Godot code gen
├── core/                     (5 files) Universal ECS — archetype-based, full-featured
│   ├── entity.rs             (207 LOC) UniversalEntity, Archetype, Chunk, SoAStorage
│   ├── world.rs              (419 LOC) UniversalWorld, Component, Resource, Event, query
│   ├── scheduler.rs          (174 LOC) ParallelScheduler, wave-based execution
│   └── change_detection.rs   (413 LOC) Changed<T>, ChangeTick, ChangeDetector, ChangeTracker
├── ecs/                      (3 files) Simple ECS — HashMap-based, DUPLICATE of core/
│   ├── world.rs              (210 LOC) Entity, World, ComponentStorage
│   └── system.rs             (142 LOC) System, SystemScheduler, stub systems
├── engine/                   (10 files) Engine subsystems
│   ├── renderer.rs           (367 LOC) Color, Vec2, Rect, Transform, Sprite, TileMap, Camera, Renderer
│   ├── physics.rs            (367 LOC) RigidBody, Collider, SimplePhysicsWorld
│   ├── input.rs              (304 LOC) KeyCode, InputState, SimpleInputProvider
│   ├── audio.rs              (176 LOC) AudioManager, AudioBackend
│   ├── scene.rs              (296 LOC) SceneGraph, SceneNode
│   ├── events.rs             (132 LOC) EventBus, Event, EventHandler — DUPLICATE
│   ├── event_bus.rs          (212 LOC) TypedEventBus, GameEvent — BETTER version
│   ├── pet_state.rs          (447 LOC) 12-state Pet FSM, EyeTracking, PermissionBubble
│   ├── hook_system.rs        (338 LOC) HookManager, HookEvent, PermissionMode
│   └── theme_system.rs       (424 LOC) ThemeManager, SpriteSheet, AnimationPlayer
└── mechanics/                (4 files) NeoTrix consciousness — NOT game-relevant
    ├── mod.rs                (293 LOC) ConsciousnessEntity, TransformComponent, MaslowNeeds, etc.
    ├── core_pet.rs           (230 LOC) CorePetState — DUPLICATE of engine/pet_state.rs
    ├── core_hook.rs          (226 LOC) CoreHookManager — DUPLICATE of engine/hook_system.rs
    └── core_theme.rs         (159 LOC) CoreTheme — DUPLICATE of engine/theme_system.rs
```

---

## 1. Redundancy Cleanup

### 1.1 Duplicate ECS Implementations

| Issue | Location | Evidence | Action |
|-------|----------|----------|--------|
| Two ECS worlds | `ecs::World` (line 74, `ecs/world.rs`) vs `core::UniversalWorld` (line 27, `core/world.rs`) | Both have `spawn()`, `insert_component()`, `get_component()`, `query()` | **Delete `ecs/` entirely** |
| Two Entity types | `ecs::Entity { id: u32, generation: u32 }` vs `core::UniversalEntity { id: EntityId(u64), generation: u32, archetype: ArchetypeId }` | core version has archetype tracking | **Keep `core::UniversalEntity`** |
| Two System traits | `ecs::System` (line 4, `ecs/system.rs`) vs `core::UniversalSystem` (line 6, `core/scheduler.rs`) | core version has `read_components()`/`write_components()` for parallel scheduling | **Keep `core::UniversalSystem`** |
| Two schedulers | `ecs::SystemScheduler` (priority sort only) vs `core::ParallelScheduler` (wave-based topological sort) | core version supports dependency-based parallel execution | **Keep `core::ParallelScheduler`** |
| Blanket impl conflict | `ecs::world.rs:23` has `impl<T: Any + Send + Sync> Component for T` | This conflicts with `core::world.rs:6` requiring `Clone + 'static` | **Delete `ecs/`** |

**Resolution**: Delete `src/ecs/` entirely. The simple ECS was a prototype. Update all imports:
- `lib.rs:9-10` — change `pub use ecs::{Entity, World, System, SystemScheduler}` to `pub use core::{UniversalEntity as Entity, UniversalWorld as World, UniversalSystem as System, ParallelScheduler as SystemScheduler}`
- `mechanics/mod.rs:1` — change `use crate::ecs::{System, World}` to `use crate::core::{UniversalSystem as System, UniversalWorld as World}`
- `mechanics/*.rs` — update all `ecs` imports to `core`

### 1.2 Duplicate Event Systems

| Issue | Location | Evidence | Action |
|-------|----------|----------|--------|
| Two event buses | `engine::events::EventBus` (line 15) vs `engine::event_bus::TypedEventBus` (line 23) | Both use `HashMap<TypeId, VecDeque<...>>` pattern | **Delete `events.rs`** |
| Different Event traits | `events::Event: Any + Send + Sync` (line 5) vs `event_bus::GameEvent: Any + Send + Sync` (line 5) | Identical trait bounds | **Unify to `GameEvent`** |
| Different handler traits | `events::EventHandler` (line 10) vs `event_bus::GameEventHandler` (line 10) | GameEventHandler has `name()` + `priority()` | **Keep `GameEventHandler`** |
| events.rs EventBus uses World | `events.rs:11` `EventHandler::handle(&mut self, event: &dyn Event, world: &mut World)` | Tightly coupled to old ECS | **Delete** |

**Resolution**: Delete `engine/events.rs`. Move `engine/event_bus.rs` → `core/event_bus.rs`. Update imports:
- `lib.rs:25` — `pub use engine::event_bus::{...}` → `pub use core::event_bus::{...}`
- `engine/mod.rs:4` — remove `pub mod events`

### 1.3 Duplicate Transform Types

| Type | Location | Fields | Action |
|------|----------|--------|--------|
| `engine::Transform` | `renderer.rs:146` | position: Vec2, rotation: f32, scale: Vec2 | **Keep** as spatial transform |
| `mechanics::TransformComponent` | `mod.rs:31` | position: Vec2, velocity: Vec2, facing: u8 | **Rename to `MovementComponent`** |
| `engine::physics::RigidBody` | `physics.rs:17` | position: Vec2, velocity: Vec2, acceleration: Vec2 | Overlaps with TransformComponent |

**Resolution**: `MovementComponent` is a game-layer concept (movement + facing direction). `Transform` is engine-layer (spatial). `RigidBody` is physics-layer. No deletion needed, just rename.

### 1.4 Triple-Duplicate Pet/Hook/Theme Systems

The desktop pet code exists in THREE places:

| Concept | engine/ (detailed) | mechanics/core_*.rs (simplified) | Action |
|---------|-------------------|----------------------------------|--------|
| Pet FSM | `pet_state.rs` (447 LOC, 12 states, components, systems) | `core_pet.rs` (230 LOC, same states, simpler) | **Delete `core_pet.rs`** |
| Hook system | `hook_system.rs` (338 LOC, HookManager, PermissionMode) | `core_hook.rs` (226 LOC, simplified event processing) | **Delete `core_hook.rs`** |
| Theme system | `theme_system.rs` (424 LOC, ThemeManager, SpriteSheet) | `core_theme.rs` (159 LOC, simplified) | **Delete `core_theme.rs`** |

**Resolution**: Delete `src/mechanics/` entirely. The engine/ versions are more complete. Feature-gate `engine/pet_state.rs`, `engine/hook_system.rs`, `engine/theme_system.rs` behind `#[cfg(feature = "pet")]`.

### 1.5 Stale Mechanics Components

| Component | Location | Usage | Action |
|-----------|----------|-------|--------|
| `ConsciousnessEntity` | `mechanics/mod.rs:10` | Only in `lib.rs:125-128` (stardew demo) | **Delete** |
| `TransformComponent` | `mechanics/mod.rs:31` | Used in `lib.rs:130-132`, `mechanics/mod.rs:279-284` | **Move to `game/movement.rs`** as `MovementComponent` |
| `RenderComponent` | `mechanics/mod.rs:49` | Used in `lib.rs:134-136` | **Move to `engine/renderer.rs`** as `SpriteComponent` |
| `AiComponent` | `mechanics/mod.rs:67` | Used in `lib.rs:165` | **Replace** with game NPC AI |
| `MaslowNeeds` | `mechanics/mod.rs:86` | Used in `lib.rs:137` | **Delete** (not needed) |
| `SocialRelationship` | `mechanics/mod.rs:142` | Not used anywhere | **Delete** |
| `EconomyComponent` | `mechanics/mod.rs:168` | Used in `lib.rs:139-141` | **Replace** with `game/inventory.rs` |
| `ConsciousnessSystem` | `mechanics/mod.rs:213` | Used in `lib.rs:115` | **Delete** |
| `MaslowSystem` | `mechanics/mod.rs:234` | Used in `lib.rs:113` | **Delete** |
| `AiSystem` | `mechanics/mod.rs:257` | Used in `lib.rs:114` | **Replace** with game AI |

### 1.6 Stale Codegen/Adapters

| Module | Files | LOC | Usage | Action |
|--------|-------|-----|-------|--------|
| `codegen/` | 3 files | ~218 LOC | Only in `lib.rs:174` test + `main.rs:143` demo | **Feature-gate** `#[cfg(feature = "codegen")]` |
| `adapters/` | 4 files | ~360 LOC | Only in `lib.rs` (no re-exports) | **Feature-gate** `#[cfg(feature = "adapters")]` |

### 1.7 Dead Code in lib.rs

| Item | Location | Issue | Action |
|------|----------|-------|--------|
| `create_stardew_valley_game()` | `lib.rs:107-169` | Creates desktop pet entities, not game entities | **Delete** |
| `GameEngine` struct | `lib.rs:34-104` | Uses `ecs::World` (old ECS), not `core::UniversalWorld` | **Rewrite** to use `core/` |
| Test `test_ecs` | `lib.rs:177-182` | Uses `ecs::World` | **Update** to use `core/` |
| Test `test_physics` | `lib.rs:184-191` | Uses `ecs::Entity` | **Update** to use `core::EntityId` |

---

## 2. Architecture Restructuring

### 2.1 Proposed New Module Layout

```
src/
├── lib.rs                  # Public API + GameEngine (rewritten for core/)
├── main.rs                 # CLI entry (updated imports)
├── builder.rs              # GameBuilder (updated to use core/)
│
├── core/                   # ECS + foundation (KEEP, move event_bus here)
│   ├── mod.rs              # Add event_bus module
│   ├── entity.rs           # UniversalEntity, Archetype, Chunk (keep as-is)
│   ├── world.rs            # UniversalWorld, Component, Resource, Event (keep as-is)
│   ├── scheduler.rs        # ParallelScheduler (keep as-is)
│   ├── change_detection.rs # Changed<T>, ChangeTracker (keep as-is)
│   └── event_bus.rs        # TypedEventBus (MOVED from engine/)
│
├── engine/                 # Rendering + physics + input (CLEANED)
│   ├── mod.rs              # Remove events, event_bus, pet/hook/theme
│   ├── renderer.rs         # Color, Vec2, Rect, Transform, Sprite, Camera, Renderer
│   │                       # + SpriteComponent (moved from mechanics/)
│   ├── physics.rs          # RigidBody, Collider, SimplePhysicsWorld
│   ├── input.rs            # InputState, InputProvider, KeyCode
│   ├── audio.rs            # AudioManager
│   └── scene.rs            # SceneGraph
│
├── world/                  # NEW: World generation + tile system
│   ├── mod.rs
│   ├── tile.rs             # TileType, TileProperties, TileMap
│   ├── terrain.rs          # TerrainGenerator (noise, biomes)
│   ├── zones.rs            # Zone system (farm, town, mine, forest, beach)
│   ├── chunk.rs            # Chunk loading/unloading (16x16 tiles)
│   └── pathfinding.rs      # A* pathfinding
│
├── game/                   # NEW: Stardew Valley mechanics
│   ├── mod.rs
│   ├── time.rs             # GameClock, Season, day/night cycle
│   ├── inventory.rs        # Inventory, Item, Equipment, stacking
│   ├── farming.rs          # Crop, CropTile, planting/growth/harvest
│   ├── crafting.rs         # Recipes, workstations
│   ├── dialogue.rs         # DialogueTree, choices, gift effects
│   ├── npc.rs              # NPC schedule, relationships, AI
│   ├── weather.rs          # Rain, snow, sunny, storms
│   ├── energy.rs           # Player stamina/resonance
│   ├── combat.rs           # Mine combat (basic)
│   ├── shop.rs             # Buy/sell, pricing
│   └── movement.rs         # MovementComponent (moved from mechanics/)
│
├── ui/                     # NEW: UI framework
│   ├── mod.rs
│   ├── widgets.rs          # Button, Label, Panel, Slider
│   ├── inventory_ui.rs     # Inventory display
│   ├── dialogue_ui.rs      # Dialogue box
│   ├── hud.rs              # Health, energy, time, gold
│   ├── menu.rs             # Main menu, pause, settings
│   └── layout.rs           # UI layout engine
│
├── save/                   # NEW: Save/load
│   ├── mod.rs
│   ├── serializer.rs       # World state serialization
│   ├── slot.rs             # Save slots, auto-save
│   └── migration.rs        # Version migration
│
├── mechanics/              # DELETED (all content moved or removed)
│
├── adapters/               # FEATURE-GATED: #[cfg(feature = "adapters")]
├── codegen/                # FEATURE-GATED: #[cfg(feature = "codegen")]
└── pet/                    # FEATURE-GATED: #[cfg(feature = "pet")] (moved from engine/)
    ├── mod.rs
    ├── state.rs            # from engine/pet_state.rs
    ├── hook_system.rs      # from engine/hook_system.rs
    └── theme_system.rs     # from engine/theme_system.rs
```

### 2.2 Dependency Flow

```
                 game/ (Stardew mechanics)
                /       |        \
           world/     ui/      save/
            |          |          |
           engine/ (render, physics, input, audio)
                \       |       /
                  core/ (ECS, scheduler, events, event_bus)
```

### 2.3 Key Design Decisions

1. **Single ECS**: Use `core/` archetype-based ECS exclusively. Delete `ecs/`.
2. **Single EventBus**: Use `TypedEventBus` with priority handlers. Move to `core/`.
3. **TileMap ownership**: Move tile definitions from `engine/renderer.rs` to `world/tile.rs`. Keep `TileMap` in renderer as a renderable, but `TileType`/`TileProperties` live in `world/`.
4. **Game state as Resource**: All game systems store state via `World::insert_resource::<T>()`.
5. **Components are plain structs**: No trait objects. The `core::Component` trait requires `Send + Sync + Clone + 'static`.
6. **Systems implement `UniversalSystem`**: All systems use `core::UniversalSystem` trait with `read_components()`/`write_components()` for parallel scheduling.
7. **Builder uses core/**: `GameBuilder` creates `UniversalWorld` + `ParallelScheduler`, not the old `GameEngine`.

---

## 3. New Modules Needed for Stardew Valley

### 3.1 World Generation (`world/`)

**tile.rs** — Core tile definitions
```rust
pub enum TileType {
    Grass, TallGrass, Dirt, TillDirt, Water, DeepWater,
    Stone, Sand, Path, WoodFloor, StoneFloor,
    OreCopper, OreIron, OreGold,
    TreeOak, TreeMaple, TreePine,
    FarmCropEmpty, FarmCropStage1..4,
    House, Shop, Barn, Coop, Silo, Well,
    Fence, Gate, Door,
}
pub struct TileProperties {
    pub walkable: bool,
    pub farmable: bool,
    pub mineable: bool,
    pub fishable: bool,
    pub transit: Option<String>, // zone transition target
}
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub tiles: Vec<Vec<TileType>>,
    pub properties: HashMap<TileType, TileProperties>,
}
```

**terrain.rs** — Procedural world generation
- Perlin noise for height map (use `noise` crate)
- Biome selection (forest, plains, mountain, beach)
- River/stream carving
- Ore vein placement
- Farm plot layout

**zones.rs** — Zone system
```rust
pub enum Zone {
    Farm,        // Player home, crops, animals
    Town,        // Shops, NPCs, events
    Mountain,    // Mine entrance, quarry
    Forest,      // Foraging, secret woods
    Beach,       // Fishing, tide pools
    Mine,        // 120 floors, procedural
    Desert,      // Late game
}
pub struct ZoneTransition {
    pub from: Zone,
    pub to: Zone,
    pub tile_pos: (usize, usize),
    pub spawn_pos: Vec2,
}
```

**chunk.rs** — Chunk loading/unloading
- 16×16 tile chunks
- Load/unload based on player proximity
- Serialize loaded chunks for save/load

**pathfinding.rs** — A* for NPCs and player
- Grid-based A* on tile map
- Multi-tile entity support
- Walkability from `TileProperties`

### 3.2 Inventory System (`game/inventory.rs`)

```rust
pub struct Item {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub stack_size: u32,
    pub max_stack: u32,
    pub quality: Quality,
    pub icon: String,
    pub description: String,
    pub sell_price: u32,
}
pub enum ItemType {
    Seed, Crop, Tool, Weapon, Armor, Food, Material, Furniture, Artifact, Fish, Gem,
}
pub enum Quality { Normal, Silver, Gold, Iridium }
pub struct Inventory {
    pub slots: Vec<Option<InventorySlot>>,
    pub capacity: usize,
    pub hotbar: Vec<Option<InventorySlot>>,
    pub hotbar_selected: usize,
}
pub struct InventorySlot { pub item: Item, pub count: u32 }
pub struct Equipment {
    pub tool_slot: Option<Item>,
    pub weapon_slot: Option<Item>,
    pub boots_slot: Option<Item>,
    pub ring_slots: [Option<Item>; 2],
}
```

### 3.3 Farming System (`game/farming.rs`)

```rust
pub struct Crop {
    pub seed_id: u32,
    pub name: String,
    pub growth_stages: u32,
    pub total_days: u32,
    pub seasons: Vec<Season>,
    pub regrow_days: Option<u32>,
    pub sell_prices: [u32; 4], // normal, silver, gold, iridium
}
pub struct CropTile {
    pub state: CropState,
    pub day_planted: u32,
    pub watered: bool,
    pub fertilized: bool,
}
pub enum CropState { Empty, Planted, Growing(u32), Ready, Withered }
pub struct FarmingSystem {
    pub farm_plots: HashMap<(i32, i32), CropTile>,
    pub crop_db: HashMap<u32, Crop>,
}
```

### 3.4 Dialogue System (`game/dialogue.rs`)

```rust
pub struct DialogueTree {
    pub npc_id: u32,
    pub nodes: HashMap<u32, DialogueNode>,
    pub start_node: u32,
}
pub struct DialogueNode {
    pub text: String,
    pub responses: Vec<DialogueResponse>,
    pub conditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}
pub struct DialogueResponse {
    pub text: String,
    pub next_node: u32,
    pub conditions: Vec<Condition>,
    pub gift_item: Option<u32>,
}
pub enum Condition { HeartLevel(u32), HasItem(u32), QuestComplete(u32), Season(Season), TimeOfDay(u8) }
pub enum Effect { AddHeart(u32), RemoveHeart(u32), AddItem(u32), RemoveItem(u32), StartQuest(u32) }
```

### 3.5 Crafting System (`game/crafting.rs`)

```rust
pub struct Recipe {
    pub id: u32,
    pub name: String,
    pub ingredients: Vec<(u32, u32)>, // (item_id, count)
    pub result: (u32, u32),           // (item_id, count)
    pub workstation: Option<String>,
    pub unlocked: bool,
}
pub struct CraftingSystem {
    pub recipes: Vec<Recipe>,
    pub unlocked_recipes: HashSet<u32>,
}
```

### 3.6 NPC System (`game/npc.rs`)

```rust
pub struct Npc {
    pub id: u32,
    pub name: String,
    pub schedule: Vec<ScheduleEntry>,
    pub location: (i32, i32),
    pub hearts: u32,
    pub birthday: (Season, u32),
    pub likes: Vec<u32>,
    pub dislikes: Vec<u32>,
}
pub struct ScheduleEntry {
    pub time: (u8, u8),
    pub location: String,
    pub action: ScheduleAction,
}
pub enum ScheduleAction { Stand, WalkTo(String), Sit, Sleep, Work, Event(String) }
```

### 3.7 Season/Weather System (`game/weather.rs`)

```rust
pub enum Season { Spring, Summer, Fall, Winter }
pub enum Weather { Sunny, Rainy, Stormy, Snowy, Windy }
pub struct GameClock {
    pub season: Season,
    pub day_of_season: u32,
    pub hour: u8,
    pub minute: u8,
    pub total_days: u32,
    pub year: u32,
}
pub struct WeatherSystem {
    pub current: Weather,
    pub tomorrow: Weather,
    pub season_chances: HashMap<Season, Vec<(Weather, f32)>>,
}
```

### 3.8 Save/Load System (`save/`)

```rust
pub struct SaveData {
    pub version: u32,
    pub player: PlayerData,
    pub world: WorldData,
    pub npcs: Vec<NpcData>,
    pub farm: FarmData,
    pub inventory: InventoryData,
    pub time: GameClock,
    pub quests: Vec<QuestData>,
}
pub struct SaveManager {
    pub slots: Vec<SaveSlot>,
    pub auto_save_interval: f32,
}
```

### 3.9 UI Framework (`ui/`)

```rust
pub trait Widget {
    fn update(&mut self, dt: f32, input: &InputState);
    fn render(&self, renderer: &mut dyn Renderer);
    fn handle_click(&mut self, pos: Vec2) -> Option<Box<dyn WidgetAction>>;
}
pub struct Panel { pub rect: Rect, pub children: Vec<Box<dyn Widget>> }
pub struct Button { pub rect: Rect, pub label: String, pub on_click: Box<dyn Fn()> }
pub struct Label { pub text: String, pub position: Vec2, pub color: Color }
pub struct Slider { pub value: f32, pub min: f32, pub max: f32 }
pub struct InventoryUI { pub grid_size: (usize, usize), pub slot_size: f32 }
pub struct DialogueUI { pub tree: Option<DialogueTree>, pub current_node: u32 }
pub struct Hud { pub health: f32, pub energy: f32, pub gold: u32, pub time: GameClock }
```

---

## 4. Implementation Order

### Phase 1: Core Systems (Week 1-2)

**Goal**: Clean ECS foundation + basic world rendering

1. **Delete `src/ecs/`** (3 files) — Update all imports to `src/core/`
2. **Delete `src/engine/events.rs`** — Move `TypedEventBus` to `src/core/event_bus.rs`
3. **Delete `src/mechanics/`** (4 files) — Remove all NeoTrix consciousness components
4. **Create `src/world/`**:
   - `tile.rs` — TileType, TileProperties, extended TileMap
   - `terrain.rs` — Basic Perlin noise terrain generation
   - `zones.rs` — Zone enum with farm/town/forest
5. **Update `engine/renderer.rs`** — Remove `TileMap`/`TileDef` (moved to `world/`), keep rendering types
6. **Create `src/game/mod.rs`** — Empty module shell
7. **Create `src/ui/mod.rs`** — Empty module shell
8. **Create `src/save/mod.rs`** — Empty module shell
9. **Feature-gate `adapters/`, `codegen/`, pet code**
10. **Rewrite `lib.rs`** — Remove old `GameEngine`, remove `create_stardew_valley_game()`, use `core/` types
11. **Update `main.rs`** — Use `core::UniversalWorld` + `core::ParallelScheduler`

**Deliverable**: Compiles, runs basic tile-based world rendering

### Phase 2: Game Mechanics (Week 3-5)

**Goal**: Playable farming loop

1. **`game/time.rs`** — Game clock with seasons, day/night
2. **`game/inventory.rs`** — Inventory system with items
3. **`game/movement.rs`** — MovementComponent (from mechanics/)
4. **`game/farming.rs`** — Plant, water, grow, harvest cycle
5. **`game/energy.rs`** — Player stamina system
6. **`game/npc.rs`** — Basic NPC with schedule + movement
7. **`game/dialogue.rs`** — Simple dialogue tree
8. **`game/crafting.rs`** — Basic recipe system
9. **`world/pathfinding.rs`** — A* for NPC movement
10. **`world/chunk.rs`** — Chunk loading

**Deliverable**: Player can farm, talk to NPCs, craft items

### Phase 3: Content (Week 6-8)

**Goal**: Rich content and interactions

1. **`game/weather.rs`** — Weather system affecting crops
2. **`game/shop.rs`** — Buy/sell items
3. **`game/combat.rs`** — Basic mine combat
4. **`game/npc.rs`** — Relationships, gifts, events
5. **`ui/inventory_ui.rs`** — Inventory display
6. **`ui/dialogue_ui.rs`** — Dialogue box
7. **`ui/hud.rs`** — Status display
8. **`ui/widgets.rs`** — Reusable UI components
9. **`world/terrain.rs`** — Full biome generation

**Deliverable**: Full game loop with multiple areas

### Phase 4: Polish (Week 9-12)

**Goal**: Production-ready

1. **`save/serializer.rs`** — JSON/bin save format
2. **`save/slot.rs`** — Save slots + auto-save
3. **`ui/menu.rs`** — Main menu, pause, settings
4. **`engine/audio.rs`** — Sound effects + music
5. **`engine/scene.rs`** — Scene transitions
6. **Performance**: Chunk streaming, sprite batching
7. **Polish**: Animations, particles, screen shake
8. **`builder.rs`** — Updated GameBuilder with Stardew template

**Deliverable**: Complete Stardew Valley-style game

---

## 5. File Operations

### Files to DELETE (8 files)

| File | LOC | Reason |
|------|-----|--------|
| `src/ecs/mod.rs` | 5 | Replaced by `src/core/` |
| `src/ecs/world.rs` | 210 | Replaced by `src/core/world.rs` |
| `src/ecs/system.rs` | 142 | Replaced by `src/core/scheduler.rs` |
| `src/engine/events.rs` | 132 | Replaced by `src/core/event_bus.rs` |
| `src/mechanics/mod.rs` | 293 | NeoTrix-specific, not game-relevant |
| `src/mechanics/core_hook.rs` | 226 | Desktop pet duplicate |
| `src/mechanics/core_pet.rs` | 230 | Desktop pet duplicate |
| `src/mechanics/core_theme.rs` | 159 | Desktop pet duplicate |

**Total deleted**: ~1,397 LOC

### Files to MOVE (4 files)

| File | Destination | Notes |
|------|-------------|-------|
| `src/engine/event_bus.rs` | `src/core/event_bus.rs` | Update imports in engine/mod.rs → core/mod.rs |
| `src/engine/pet_state.rs` | `src/pet/state.rs` | Feature-gate behind `#[cfg(feature = "pet")]` |
| `src/engine/hook_system.rs` | `src/pet/hook_system.rs` | Feature-gate behind `#[cfg(feature = "pet")]` |
| `src/engine/theme_system.rs` | `src/pet/theme_system.rs` | Feature-gate behind `#[cfg(feature = "pet")]` |

### Files to MODIFY (10 files)

| File | Changes |
|------|---------|
| `src/lib.rs` | Remove `ecs/` re-exports (lines 9-10), remove `mechanics/` re-exports (lines 27-31), add `game/world/ui/save` modules, delete `GameEngine` struct (lines 34-104), delete `create_stardew_valley_game()` (lines 107-169), update tests |
| `src/main.rs` | Update imports from `core::` instead of mixing `core/` and `mechanics/` |
| `src/builder.rs` | Update to use `core::UniversalWorld` (already does), add game system registration |
| `src/engine/mod.rs` | Remove `pub mod events` (line 4), remove `pub mod event_bus` (line 5, moved to core), remove pet/hook/theme re-exports |
| `src/engine/renderer.rs` | Remove `TileMap`/`TileDef` (lines 197-253), move to `world/tile.rs` |
| `src/core/mod.rs` | Add `pub mod event_bus` |
| `src/core/entity.rs` | Minor: no changes needed |
| `src/core/world.rs` | Minor: no changes needed |
| `src/mechanics/mod.rs` | **DELETE entire file** |
| `Cargo.toml` | Add features: `default = ["game"]`, `optional = ["pet", "codegen", "adapters"]`, add `noise = "0.9"` |

### Files to CREATE (22 files)

| File | Purpose | Est. LOC |
|------|---------|----------|
| `src/world/mod.rs` | World module declarations | 20 |
| `src/world/tile.rs` | TileType, TileProperties, TileMap | 150 |
| `src/world/terrain.rs` | TerrainGenerator | 200 |
| `src/world/zones.rs` | Zone system | 100 |
| `src/world/chunk.rs` | ChunkManager | 120 |
| `src/world/pathfinding.rs` | A* pathfinding | 150 |
| `src/game/mod.rs` | Game module declarations | 30 |
| `src/game/time.rs` | GameClock, Season | 150 |
| `src/game/inventory.rs` | Inventory, Item, Equipment | 250 |
| `src/game/farming.rs` | Crop, CropTile, FarmingSystem | 200 |
| `src/game/crafting.rs` | Recipe, CraftingSystem | 120 |
| `src/game/dialogue.rs` | DialogueTree, DialogueNode | 150 |
| `src/game/npc.rs` | Npc, ScheduleEntry | 180 |
| `src/game/weather.rs` | WeatherSystem | 100 |
| `src/game/energy.rs` | PlayerEnergy | 80 |
| `src/game/combat.rs` | MineCombat | 150 |
| `src/game/shop.rs` | ShopSystem | 120 |
| `src/game/movement.rs` | MovementComponent | 40 |
| `src/ui/mod.rs` | UI module declarations | 20 |
| `src/ui/widgets.rs` | Widget trait + Button/Label/Panel/Slider | 200 |
| `src/ui/inventory_ui.rs` | InventoryUI | 150 |
| `src/ui/dialogue_ui.rs` | DialogueUI | 120 |
| `src/ui/hud.rs` | Hud | 100 |
| `src/ui/menu.rs` | MenuSystem | 120 |
| `src/ui/layout.rs` | UI layout engine | 100 |
| `src/save/mod.rs` | Save module declarations | 20 |
| `src/save/serializer.rs` | SaveSerializer | 150 |
| `src/save/slot.rs` | SaveSlot | 80 |
| `src/save/migration.rs` | SaveMigration | 60 |
| `src/pet/mod.rs` | Pet module (feature-gated) | 10 |

**Total created**: ~3,390 LOC (across 30 files)

---

## 6. Cargo.toml Updates

```toml
[features]
default = ["game"]
game = []
pet = []
codegen = []
adapters = []

[dependencies]
# keep existing deps, add:
noise = "0.9"        # Perlin noise for terrain generation
bincode = "1"        # binary serialization for saves
```

---

## 7. Migration Checklist

### Phase 1: Cleanup (Week 1)
- [ ] Delete `src/ecs/` module (3 files)
- [ ] Delete `src/engine/events.rs`
- [ ] Delete `src/mechanics/` module (4 files)
- [ ] Move `src/engine/event_bus.rs` → `src/core/event_bus.rs`
- [ ] Move pet files to `src/pet/` (feature-gated)
- [ ] Feature-gate `adapters/` and `codegen/`
- [ ] Update `src/lib.rs` exports (remove old ECS, add new modules)
- [ ] Update `src/engine/mod.rs` exports (remove moved/deleted modules)
- [ ] Update `src/builder.rs` to use `core/` ECS
- [ ] Remove `TileMap`/`TileDef` from `src/engine/renderer.rs`
- [ ] Update `Cargo.toml` features + add `noise` dependency
- [ ] Verify `cargo check --all-targets -p nt-world-sim` passes

### Phase 2: New Modules (Week 2-3)
- [ ] Create `src/world/` module (6 files)
- [ ] Create `src/game/` module (11 files)
- [ ] Create `src/ui/` module (6 files)
- [ ] Create `src/save/` module (4 files)
- [ ] Write unit tests for each new module
- [ ] Verify `cargo test -p nt-world-sim --lib` passes

### Phase 3: Integration (Week 4-5)
- [ ] Wire up game systems to `ParallelScheduler`
- [ ] Implement basic game loop (input → physics → game logic → render)
- [ ] Test Stardew Valley template builds and runs
- [ ] Verify `cargo check --all-targets` passes

---

## 8. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking `core/` ECS while cleaning `ecs/` | High | Run `cargo check` after each deletion |
| Feature-gate compilation issues | Medium | Test with `--features pet,codegen,adapters` separately |
| `noise` crate compatibility | Low | Pin to `noise = "0.9"` (stable API) |
| Save format instability | Medium | Implement version migration early |
| UI rendering without real GPU | Low | Use stub renderer (like current `CanvasRenderer`) |
