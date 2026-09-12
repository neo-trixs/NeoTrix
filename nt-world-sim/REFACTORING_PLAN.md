# nt-world-sim Refactoring Plan — Stardew Valley Game Engine

## Current State Analysis

### File Inventory (35 source files)

```
src/
├── lib.rs                    # GameEngine + re-exports
├── main.rs                   # CLI entry
├── builder.rs                # GameBuilder + BuiltGame (alt engine)
├── adapters/                 # Engine adapters (Unity/Unreal/Godot/Bevy)
├── codegen/                  # YAML-to-code generator
├── core/                     # Universal ECS (archetype-based, 5 files)
├── ecs/                      # Simple ECS (HashMap-based, 3 files)
├── engine/                   # Engine subsystems (10 files)
│   ├── renderer.rs, physics.rs, input.rs, audio.rs, scene.rs
│   ├── events.rs, event_bus.rs  ← DUAL event buses
│   ├── pet_state.rs, hook_system.rs, theme_system.rs  ← DESKTOP PET
└── mechanics/                # NeoTrix consciousness (4 files)
```

---

## 1. Redundancy Cleanup

### 1.1 Duplicate ECS Implementations

| Issue | Location | Action |
|-------|----------|--------|
| Two separate ECS worlds | `ecs::World` vs `core::UniversalWorld` | **Delete `ecs/` entirely** |
| Two Entity types | `ecs::Entity` (id: u32) vs `core::UniversalEntity` (EntityId + archetype) | **Keep `core::UniversalEntity`** |
| Two System traits | `ecs::System` vs `core::UniversalSystem` | **Keep `core::UniversalSystem`** |
| Two schedulers | `ecs::SystemScheduler` vs `core::ParallelScheduler` | **Keep `core::ParallelScheduler`** |

**Resolution**: Delete `src/ecs/` entirely. Update all imports to use `src/core/`. The simple ECS was a prototype; the archetype-based core ECS is the real implementation.

### 1.2 Duplicate Event Systems

| Issue | Location | Action |
|-------|----------|--------|
| Two event bus implementations | `engine::events::EventBus` + `engine::event_bus::TypedEventBus` | **Delete `events.rs`** |
| Different Event traits | `events::Event` vs `event_bus::GameEvent` | **Unify to `GameEvent`** |
| Different handler traits | `events::EventHandler` vs `event_bus::GameEventHandler` | **Keep `GameEventHandler`** |

**Resolution**: Delete `engine/events.rs`. Use `engine/event_bus.rs` everywhere.

### 1.3 Duplicate Transform Types

| Issue | Location | Action |
|-------|----------|--------|
| `engine::Transform` (position, rotation, scale) | `renderer.rs:146` | **Keep** as spatial transform |
| `mechanics::TransformComponent` (position, velocity, facing) | `mod.rs:31` | **Rename to `MovementComponent`** |

### 1.4 Desktop Pet Code (Not Game-Relevant)

| Module | Files | Action |
|--------|-------|--------|
| Pet FSM + components | `engine/pet_state.rs` | **Feature-gate** `#[cfg(feature = "pet")]` |
| Hook manager | `engine/hook_system.rs` | **Feature-gate** `#[cfg(feature = "pet")]` |
| Theme system (pet themes) | `engine/theme_system.rs` | **Feature-gate** `#[cfg(feature = "pet")]` |
| Pet mechanics | `mechanics/core_hook.rs`, `core_pet.rs`, `core_theme.rs` | **Delete** (stale duplicates) |

### 1.5 Stale Codegen/Adapters

| Module | Files | Action |
|--------|-------|--------|
| Code generator | `codegen/` (4 files) | **Feature-gate** `#[cfg(feature = "codegen")]` |
| Engine adapters | `adapters/` (5 files) | **Feature-gate** `#[cfg(feature = "adapters")]` |

### 1.6 Stale Mechanics Components

| Component | Action |
|-----------|--------|
| `ConsciousnessEntity` | **Delete** (NeoTrix-specific) |
| `AiComponent` | **Replace** with game NPC AI |
| `EconomyComponent` | **Replace** with inventory/gold system |
| `SocialRelationship` | **Replace** with NPC relationships |
| `MaslowNeeds` | **Delete** (not needed) |
| `ConsciousnessSystem` | **Delete** (NeoTrix-specific) |
| `MaslowSystem` | **Delete** (NeoTrix-specific) |
| `AiSystem` | **Replace** with game AI |

---

## 2. Architecture Restructuring

### 2.1 Proposed New Module Layout

```
src/
├── lib.rs                  # Public API + GameEngine
├── main.rs                 # CLI entry
├── builder.rs              # GameBuilder (updated)
│
├── core/                   # ECS + foundation (KEEP, minor cleanup)
│   ├── mod.rs
│   ├── entity.rs           # Entity, Archetype, Chunk
│   ├── world.rs            # World, Component, Resource, Event
│   ├── scheduler.rs        # ParallelScheduler
│   ├── change_detection.rs # Change tracking
│   └── event_bus.rs        # TypedEventBus (MOVED from engine/)
│
├── engine/                 # Rendering + physics + input
│   ├── mod.rs
│   ├── renderer.rs         # Color, Vec2, Rect, Transform, Sprite, Camera, Renderer
│   ├── physics.rs          # RigidBody, Collider, PhysicsWorld
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
│   ├── time.rs             # Game clock, seasons, day/night
│   ├── inventory.rs        # Inventory, items, stacking, equipment
│   ├── farming.rs          # Crop: planting, growth, harvesting
│   ├── crafting.rs         # Recipes, workstations
│   ├── dialogue.rs         # NPC dialogue trees, choices, gifts
│   ├── npc.rs              # NPC schedule, relationships
│   ├── weather.rs          # Rain, snow, sunny, storms
│   ├── energy.rs           # Player stamina
│   ├── combat.rs           # Mine combat (basic)
│   └── shop.rs             # Buy/sell, pricing
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
├── mechanics/              # MINIMAL: Keep only if needed
│   └── mod.rs              # (most content deleted)
│
├── adapters/               # FEATURE-GATED: #[cfg(feature = "adapters")]
├── codegen/                # FEATURE-GATED: #[cfg(feature = "codegen")]
└── pet/                    # FEATURE-GATED: #[cfg(feature = "pet")] (moved from engine/)
```

### 2.2 Dependency Flow

```
               game/ (Stardew mechanics)
              /       |        \
         world/     ui/      save/
          |          |          |
         engine/ (render, physics, input, audio)
              \       |       /
                core/ (ECS, scheduler, events)
```

### 2.3 Key Design Decisions

1. **Single ECS**: Use `core/` archetype-based ECS exclusively. Delete `ecs/`.
2. **Single EventBus**: Use `TypedEventBus` with priority handlers. Delete `events.rs`.
3. **TileMap in `world/`**: Move tile logic from `engine/renderer.rs` to `world/tile.rs` with rich properties.
4. **Game state as Resource**: All game systems store state via `World::insert_resource<T>()`.
5. **Components are plain structs**: No trait objects. Use blanket `impl<T: Any + Send + Sync> Component for T`.

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
```

**terrain.rs** — Procedural world generation
- Perlin noise for height map
- Biome selection (forest, plains, mountain, beach)
- River/stream carving
- Ore vein placement
- Farm plot layout

**zones.rs** — Zone system
- Farm (player home, crops, animals)
- Town (shops, NPCs, events)
- Mountain (mine entrance, quarry)
- Forest (foraging, secret woods)
- Beach (fishing, tide pools)
- Mine (120 floors, procedural)
- Desert (late game)

**pathfinding.rs** — A* for NPCs and player
- Grid-based A* on tile map
- Navmesh for complex areas
- Multi-tile entity support

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
    pub tool_slot: Option<Item>,    // hoe, pickaxe, axe, watering can
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
    pub workstation: Option<String>,  // workbench, furnace, etc.
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
    pub likes: Vec<u32>,   // item_ids
    pub dislikes: Vec<u32>,
}
pub struct ScheduleEntry {
    pub time: (u8, u8),   // hour, minute
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

1. **Delete `src/ecs/`** — Update all imports to `src/core/`
2. **Delete `src/engine/events.rs`** — Move `TypedEventBus` to `src/core/event_bus.rs`
3. **Delete `src/mechanics/`** — Remove NeoTrix consciousness components
4. **Create `src/world/`**:
   - `tile.rs` — TileType, TileProperties, extended TileMap
   - `terrain.rs` — Basic Perlin noise terrain generation
   - `zones.rs` — Zone enum with farm/town/forest
5. **Update `engine/renderer.rs`** — Remove TileMap (moved to world/), keep rendering types
6. **Create `src/game/mod.rs`** — Empty module shell
7. **Create `src/ui/mod.rs`** — Empty module shell
8. **Create `src/save/mod.rs`** — Empty module shell
9. **Feature-gate `adapters/`, `codegen/`, `pet/`**

**Deliverable**: Compiles, runs basic tile-based world rendering

### Phase 2: Game Mechanics (Week 3-5)

**Goal**: Playable farming loop

1. **`game/time.rs`** — Game clock with seasons, day/night
2. **`game/inventory.rs`** — Inventory system with items
3. **`game/farming.rs`** — Plant, water, grow, harvest cycle
4. **`game/energy.rs`** — Player stamina system
5. **`game/npc.rs`** — Basic NPC with schedule + movement
6. **`game/dialogue.rs`** — Simple dialogue tree
7. **`game/crafting.rs`** — Basic recipe system
8. **`world/pathfinding.rs`** — A* for NPC movement
9. **`world/chunk.rs`** — Chunk loading

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
8. **`builder.rs`** — Updated GameBuilder with template

**Deliverable**: Complete Stardew Valley-style game

---

## 5. File Operations

### Files to DELETE (16 files)

| File | Reason |
|------|--------|
| `src/ecs/mod.rs` | Replaced by `src/core/` |
| `src/ecs/world.rs` | Replaced by `src/core/world.rs` |
| `src/ecs/system.rs` | Replaced by `src/core/scheduler.rs` |
| `src/engine/events.rs` | Replaced by `src/core/event_bus.rs` |
| `src/mechanics/mod.rs` | NeoTrix-specific, not game-relevant |
| `src/mechanics/core_hook.rs` | Desktop pet, stale |
| `src/mechanics/core_pet.rs` | Desktop pet, stale |
| `src/mechanics/core_theme.rs` | Desktop pet, stale |

### Files to MOVE (3 files, behind feature gate)

| File | Destination |
|------|-------------|
| `src/engine/pet_state.rs` | `src/pet/state.rs` |
| `src/engine/hook_system.rs` | `src/pet/hook_system.rs` |
| `src/engine/theme_system.rs` | `src/pet/theme_system.rs` |

### Files to MOVE to core (1 file)

| File | Destination |
|------|-------------|
| `src/engine/event_bus.rs` | `src/core/event_bus.rs` |

### Files to MODIFY (8 files)

| File | Changes |
|------|---------|
| `src/lib.rs` | Remove ecs/ re-exports, remove mechanics/ re-exports, add game/world/ui/save modules |
| `src/main.rs` | Update imports |
| `src/builder.rs` | Update to use core/ ECS, add game system registration |
| `src/engine/mod.rs` | Remove event_bus (moved to core), remove pet_state/hook/theme |
| `src/engine/renderer.rs` | Remove TileMap/TileDef (moved to world/), keep rendering types |
| `src/core/mod.rs` | Add event_bus module |
| `src/core/entity.rs` | Minor cleanup |
| `Cargo.toml` | Add features: default = ["game"], optional = ["pet", "codegen", "adapters"] |

### Files to CREATE (22 files)

| File | Purpose |
|------|---------|
| `src/world/mod.rs` | World module declarations |
| `src/world/tile.rs` | TileType, TileProperties, TileMap |
| `src/world/terrain.rs` | TerrainGenerator |
| `src/world/zones.rs` | Zone system |
| `src/world/chunk.rs` | ChunkManager |
| `src/world/pathfinding.rs` | A* pathfinding |
| `src/game/mod.rs` | Game module declarations |
| `src/game/time.rs` | GameClock, Season |
| `src/game/inventory.rs` | Inventory, Item, Equipment |
| `src/game/farming.rs` | Crop, CropTile, FarmingSystem |
| `src/game/crafting.rs` | Recipe, CraftingSystem |
| `src/game/dialogue.rs` | DialogueTree, DialogueNode |
| `src/game/npc.rs` | Npc, ScheduleEntry |
| `src/game/weather.rs` | WeatherSystem |
| `src/game/energy.rs` | PlayerEnergy |
| `src/game/combat.rs` | MineCombat |
| `src/game/shop.rs` | ShopSystem |
| `src/ui/mod.rs` | UI module declarations |
| `src/ui/widgets.rs` | Widget trait + Button/Label/Panel/Slider |
| `src/ui/inventory_ui.rs` | InventoryUI |
| `src/ui/dialogue_ui.rs` | DialogueUI |
| `src/ui/hud.rs` | Hud |
| `src/ui/menu.rs` | MenuSystem |
| `src/save/mod.rs` | Save module declarations |
| `src/save/serializer.rs` | SaveSerializer |
| `src/save/slot.rs` | SaveSlot |
| `src/save/migration.rs` | SaveMigration |

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
noise = "0.9"        # Perlin noise for terrain
bincode = "1"        # binary serialization for saves
```

---

## 7. Migration Checklist

- [ ] Delete `src/ecs/` module
- [ ] Delete `src/engine/events.rs`
- [ ] Delete `src/mechanics/` module
- [ ] Move `src/engine/event_bus.rs` -> `src/core/event_bus.rs`
- [ ] Move pet files to `src/pet/` (feature-gated)
- [ ] Feature-gate `adapters/` and `codegen/`
- [ ] Update `src/lib.rs` exports
- [ ] Update `src/engine/mod.rs` exports
- [ ] Update `src/builder.rs` to use core/ ECS
- [ ] Remove TileMap/TileDef from `src/engine/renderer.rs`
- [ ] Update `Cargo.toml` features
- [ ] Create `src/world/` module (6 files)
- [ ] Create `src/game/` module (10 files)
- [ ] Create `src/ui/` module (5 files)
- [ ] Create `src/save/` module (4 files)
- [ ] Write unit tests for each new module
- [ ] Verify `cargo check --all-targets` passes
- [ ] Verify `cargo test -p nt-world-sim` passes
