# Cross-Engine Fusion Architecture

> **Definitive Reference** — How nt-world-sim's universal engine restores games from any external engine model.
> Synthesizes research, audit findings, and architectural mapping into a single actionable document.

---

## 1. Engine Reverse-Engineering Results

### 1.1 Unity DOTS/ECS vs NT-World-Sim ECS

| Dimension | Unity DOTS | NT-World-Sim | Delta |
|-----------|-----------|-------------|-------|
| **Entity model** | `Entity` (u32 index + u32 version) in `Unity.Collections` | `UniversalEntity { id: EntityId(u64), generation: u32, archetype }` | NT uses u64 monotonic IDs — safer for cross-session persistence, no index recycling |
| **Component storage** | `Archetype`-based, chunked (16KB blocks), burst-compatible SoA | `ComponentStorage` enum: Dense(Vec), Sparse(HashMap), SoA | Unity chunks are memory-aligned for SIMD; NT SoA is type-erased `Vec<u8>` — functionally equivalent but no SIMD optimization |
| **System scheduling** | `SystemBase` with `OnUpdate`, `OnStart`, dependency graphs via `[UpdateBefore]`/`[UpdateAfter]` | `UniversalSystem` trait + `ParallelScheduler` with priority waves | Unity uses attribute-based dependency declaration; NT uses `read_components()`/`write_components()` for automatic wave scheduling |
| **Burst compilation** | `[BurstCompile]` → SIMD vectorized inner loops | None | **Gap**: NT lacks compile-time SIMD optimization. Mitigation: write hot loops in `unsafe`-free SIMD manually or use `std::simd` |
| **Jobs system** | `IJob`/`IJobParallelFor`/`IJobEntity` with `JobHandle` completion | `ParallelScheduler` wave execution | Unity Jobs are multi-threaded with work-stealing; NT runs waves single-threaded per system. **Gap**: no intra-system parallelism |
| **Aspects** | `IAspect` — query-side component grouping for faster iteration | No equivalent — systems query ad-hoc | **Gap**: NT lacks aspect objects. Add `Aspect` trait that bundles component sets into named query profiles |
| **Structural changes** | `EntityCommandBuffer` (deferred mutations) | Direct mutation in `update()` | **Gap**: NT has no deferred command buffer. Current approach is fine for <10K entities; add ECB if scaling past that |

**Key insight**: Unity DOTS optimizes for 100K+ entities with SIMD + Jobs. NT-World-Sim targets game-jam scale (<5K entities). The architecture is compatible — just missing performance layers.

### 1.2 Unreal GAS (Gameplay Ability System) vs NT-World-Sim

| Dimension | Unreal GAS | NT-World-Sim | Delta |
|-----------|-----------|-------------|-------|
| **Ability system** | `UGameplayAbility` + `UAbilitySystemComponent` (ASC) — tag-driven activation/cancellation | No equivalent — game logic lives in free functions | **Gap**: NT lacks tag-driven ability activation. Map to: `Ability` component + `AbilitySystem` resource |
| **Gameplay tags** | `FGameplayTag` — hierarchical string tags (`Ability.Skill.Fireball`) | No tag system | **Gap**: Add `GameplayTags` with hierarchical matching. Used for: ability activation, effect immunity, UI state |
| **Gameplay effects** | `UGameplayEffect` — duration-based stat modifiers (buffs/debuffs) with stacking rules | No stat modifier system | **Gap**: Add `StatModifier { stat, op, value, duration, stack_count }` component |
| **Attribute sets** | `UAttributeSet` — float attributes (Health, Mana, Stamina) with clamp/replication | No attribute system | Map to: `Attributes` component with `HashMap<String, f32>` |
| **Targeting** | `TargetActor` with radius/cone/line traces | None | Map to: spatial query on `UniversalWorld` |
| **Cooldowns** | Tag-based cooldown tracking per ability | None | Map to: `CooldownTracker` resource |

**Key insight**: GAS is a runtime data-driven ability framework. NT doesn't need the full GAS — just the tag + effect + attribute subset for NPC interactions and farming.

### 1.3 Godot Scene-Tree vs NT-World-Sim SceneGraph

| Dimension | Godot Scene Tree | NT-World-Sim SceneGraph | Delta |
|-----------|-----------------|------------------------|-------|
| **Hierarchy** | `Node` tree with `_process()` and `_physics_process()` callbacks | `SceneGraph { nodes: HashMap<EntityId, SceneNode>, root }` | Godot nodes are also processing units; NT nodes are pure data — processing lives in `UniversalSystem` |
| **Inheritance** | Scene inheritance (instantiate parent scene as child) | No scene inheritance | **Gap**: Add `SceneTemplate` with instantiation |
| **Groups** | `add_to_group("enemies")` — query all nodes in group | No group system | **Gap**: Add `EntityGroup` resource — `HashMap<String, HashSet<EntityId>>` |
| **Signals** | `signal` declarations + `connect()` — compile-time type-safe | `TypedEventBus` (runtime type-erased) | Godot signals are per-node; NT events are global. **Gap**: Add per-entity event channels |
| **Remote calls** | `rpc()` / `rpc_id()` for multiplayer | None | Out of scope for single-player sim |
| **Transform propagation** | Automatic parent→child in `_process()` | `update_transforms()` on SceneGraph | Equivalent — both recursive |

**Key insight**: Godot's scene tree is a combined data + processing hierarchy. NT separates these (SceneGraph for data, Systems for processing). The split is cleaner but loses Godot's convenience of per-node callbacks.

### 1.4 Bevy Schedule/States vs NT-World-Sim ParallelScheduler

| Dimension | Bevy | NT-World-Sim | Delta |
|-----------|------|-------------|-------|
| **Schedule** | `Schedule` with `SystemSet`s, `Configs`, automatic conflict resolution | `ParallelScheduler` with `priority: i32` waves | Bevy has finer granularity (sets within states); NT uses flat priority. **Gap**: Add `SystemSet` grouping |
| **States** | `States` trait — `OnEnter`/`OnExit` systems, state-conditional system loading | No state machine | **Gap**: Add `GameState` enum with `OnEnter`/`OnExit` hooks |
| **Resources** | `Res<T>` / `ResMut<T>` system parameters | `UniversalWorld.get_resource::<T>()` | Bevy uses Rust generics for type-safe access; NT uses type-erased `Box<dyn Any>`. Both work |
| **Events** | `EventWriter<T>` / `EventReader<T>` system params | `TypedEventBus` global | Bevy events are per-type with automatic accumulation; NT is manual. **Gap**: Add typed event readers |
| **Change detection** | `Added<T>`, `Changed<T>` filters | `change_detection.rs` (exists but minimal) | NT has foundation but not wired into queries. **Gap**: Add archetype-level change tracking |
| **App builder** | `App::new().add_systems(Startup, setup).add_systems(Update, (a, b, c))` | Manual system registration | **Gap**: Add builder pattern for system registration |

**Key insight**: Bevy's App builder + States + SystemSets give a declarative game structure. NT's ParallelScheduler is a lower-level primitive that can build the same patterns but requires manual wiring.

---

## 2. Universal Abstraction Mapping

Complete cross-engine concept mapping for game restoration:

| Concept | Unity | Unreal | Godot | Bevy | NT-World-Sim | Notes |
|---------|-------|--------|-------|------|-------------|-------|
| **Entity** | `GameObject` | `AActor` | `Node` | `Entity` | `UniversalEntity` | All are integer-indexed. NT uses u64 monotonic |
| **Component** | `MonoBehaviour` | `UActorComponent` | Node properties | `Component` | `Component` trait | NT: plain data, no methods |
| **System** | `MonoBehaviour.Update()` | `Tick()` | `_process()` | `System` | `UniversalSystem` | NT: explicit priority + read/write sets |
| **Resource** | Static class | `UGameInstance` | `@autoload` | `Resource` | `Resource` trait | NT: stored in `UniversalWorld.resources` |
| **Event** | `UnityEvent` | `DECLARE_DELEGATE` | `Signal` | `Event<T>` | `TypedEventBus` | NT: global bus, per-type handlers |
| **Scene** | `Scene` (`.unity`) | `Level` (`.umap`) | `Scene` (`.tscn`) | `State` | `SceneGraph` | NT: flat HashMap with parent pointers |
| **Asset** | `AssetDatabase` | `TSoftObjectPtr` | `Resource` | `AssetServer` | `AssetServer` (planned) | NT: planned with handle-based loading |
| **Input** | `InputAction` | `UInputAction` | `InputEvent` | `InputState` | `InputState` + `InputMap` | NT: action mapping via HashMap |
| **UI** | UGUI (`Canvas`) | UMG (`UWidget`) | `Control` node tree | egui/bevy_ui | `Widget tree` | NT: `WidgetTrait` + `UiLayout` |
| **Audio** | `AudioSource` | `USoundCue` | `AudioStreamPlayer` | BevyAudio | `AudioManager` + `AudioBackend` | NT: trait-based, stub/web/rodio |
| **Physics** | `Rigidbody2D` | `FPhysScene2D` | `PhysicsServer2D` | Rapier | `SimplePhysicsWorld` | NT: AABB collision + velocity integration |
| **Tilemap** | `Tilemap` (2D) | `TileBase` | `TileMap` | `Tilemap` | `TileMap` | NT: `Vec<Vec<u32>>` + palette |
| **Pathfinding** | NavMesh / A* | NavigationSystem | NavigationServer2D | `navmesh` | `AStar` in `world/pathfinding.rs` | NT: A* on tile grid |
| **Dialogue** | Yarn Spinner / Ink | DialogueTree asset | — | — | `DialogueTree` in `game/dialogue.rs` | NT: node-graph dialogue |
| **Time** | `Time.deltaTime` | `GetWorld()->GetDeltaSeconds()` | `get_process_delta_time()` | `Time` resource | `DeltaTime` resource + `GameTime` | NT: both frame time and game clock |
| **Camera** | `Camera` component | `UCameraComponent` | `Camera2D/3D` | `Camera` bundle | `Camera` in `engine/camera.rs` | NT: position + zoom + viewport |
| **State machine** | Animator Controller | State Tree | AnimationTree | `States` trait | (planned) `GameState` | NT: needs implementation |

---

## 3. Focused Redundancy Map

### 3.1 Module Redundancy Analysis

| Redundant Modules | Current State | Resolution | Action |
|-------------------|--------------|------------|--------|
| `core::EntityId` (entity.rs) + `physics::Entity` (physics.rs) | Two entity types — `EntityId(u64)` and physics `Entity { id, body_id }` | **UNIFIED** — physics should reference `EntityId`, not own entity type | Delete `physics::Entity`, use `EntityId` as physics body key |
| `renderer::TileMap` + `world::WorldMap` (generator.rs) | Two tile map representations | **UNIFIED** — `TileMap` in renderer is the visual representation; `world::generator.rs` produces data. Keep both, but remove duplicate field definitions | Merge tile data structures, keep render/data separation |
| `game::dialogue` + `ui::dialogue_ui` (if exists) | Dialogue logic vs dialogue rendering | **CONSOLIDATED** — `game/dialogue.rs` owns data, UI reads it | Move any UI-specific dialogue code to `ui/`, keep `game/dialogue.rs` as data-only |
| `game::npc` + `game::npcs` | `Npc` (runtime state) + `NpcDefinition` (static data) | **MERGED** — `npc.rs` has `Npc` + `NpcRole` + `Position`; `npcs.rs` has `NpcDefinition` + `NpcType` + schedules | Merge into single `npc.rs`: `Npc` (runtime) + `NpcTemplate` (static) + `NpcSchedule` (behavior) |
| `game::farming::CropDatabase` + `game::crops` | Crop definitions in two places | **FACTORY PATTERN** — `crops.rs` defines `Crop` type, `farming.rs` has `CropDatabase` | Keep `Crop` struct in `crops.rs`, move `CropDatabase` to be a `Resource` in `farming.rs` that loads from data |
| `game::crafting` + `game::recipes` | Crafting system + recipe data | **CONSOLIDATED** — `recipes.rs` has `Recipe`, `crafting.rs` has `CraftingSystem` | Keep both — data + system separation is correct. Ensure `recipes.rs` defines `CraftingRecipe` and `crafting.rs` defines the system |

### 3.2 Type Definition Placement Audit

| Type | Current Location | Correct Location | Action |
|------|-----------------|------------------|--------|
| `Component` impl for `Transform` | `engine/renderer.rs` | `core/entity.rs` or dedicated `core/components.rs` | Move to core — it's a fundamental component, not renderer-specific |
| `Component` impl for `Sprite` | `engine/renderer.rs` | `core/components.rs` | Move to core — `Sprite` is data, renderer consumes it |
| `Component` impl for `RigidBody` | `engine/physics.rs` | `core/components.rs` | Move to core — `RigidBody` is data, physics system consumes it |
| `Npc` struct | `game/npc.rs` | `game/npc.rs` | Keep — but remove duplicate `Position` type (use `core::Transform`) |
| `DialogueTree` | `game/dialogue.rs` | `game/dialogue.rs` | Keep — clean data definition |
| `GameTime` | `game/time.rs` | `game/time.rs` | Keep — but ensure it's a `Resource`, not a component |

### 3.3 Error Handling Unification

| Current Pattern | Target Pattern | Files Affected |
|-----------------|---------------|----------------|
| `Result<T, String>` in some places | `Result<T, String>` everywhere | All game modules |
| `unwrap()` / `expect()` in production paths | `map_err()` + `?` propagation | `game_loop.rs`, `npc.rs`, `farming.rs` |
| `panic!()` for unreachable states | `debug_assert!()` + graceful fallback | `physics.rs`, `scene.rs` |
| Mixed `Option<T>` + `Result<T, String>` | Standardize: `Option` for "may not exist", `Result` for "can fail" | All modules |

---

## 4. Flat Deficiency Patch List

Minimal implementation paths for missing features, ordered by dependency:

### 4.1 Asset Pipeline (Priority: P0 — blocks content loading)

**Current state**: `AudioHandle(u32)` exists. No generic asset system.

**Implementation**:
```rust
// src/engine/asset.rs — ADD to existing file
pub struct AssetServer {
    handles: HashMap<String, AssetEntry>,
    next_id: AtomicU32,
}

impl AssetServer {
    pub fn load<T: Asset>(&mut self, path: &str) -> AssetHandle<T> { ... }
    pub fn get<T: Asset>(&self, handle: &AssetHandle<T>) -> Option<&T> { ... }
    pub fn tick(&mut self) { /* process loaded assets */ }
}

// Placeholder textures for dev:
// "sprites/player.png" → 16x16 colored rectangle
// "sprites/tile_grass.png" → green square
```

**Estimated effort**: 4 hours

### 4.2 Input Mapping (Priority: P0 — blocks player control)

**Current state**: `InputState` tracks raw keys. No action mapping.

**Implementation**:
```rust
// src/engine/input.rs — ADD to existing file
pub struct InputMap {
    bindings: HashMap<String, Vec<InputBinding>>,
}

// Minimal startup config:
fn default_input_map() -> InputMap {
    let mut map = InputMap::new();
    map.bind_action("move_up", InputBinding::Key(KeyCode::W));
    map.bind_action("move_down", InputBinding::Key(KeyCode::S));
    map.bind_action("move_left", InputBinding::Key(KeyCode::A));
    map.bind_action("move_right", InputBinding::Key(KeyCode::D));
    map.bind_action("interact", InputBinding::Key(KeyCode::E));
    map.bind_action("inventory", InputBinding::Key(KeyCode::I));
    map.bind_action("pause", InputBinding::Key(KeyCode::Escape));
    map
}
```

**Estimated effort**: 2 hours

### 4.3 Physics→GameLoop Wiring (Priority: P1 — blocks physics simulation)

**Current state**: `SimplePhysicsWorld` exists with `step()`. Not called from game loop.

**Implementation**:
```rust
// src/game/game_loop.rs — ADD physics field + call
pub struct GameLoop {
    pub physics: SimplePhysicsWorld,  // ADD
    // ... existing fields
}

impl GameLoop {
    pub fn tick(&mut self, dt: f32) {
        self.physics.step(dt);  // ADD — must run before gameplay systems
        self.time.advance(dt);
        self.farming.update(&mut self.world, &self.time);
        // ... rest of systems
    }
}
```

**Estimated effort**: 1 hour

### 4.4 SceneGraph→GameLoop Wiring (Priority: P1 — blocks scene rendering)

**Current state**: `SceneGraph` exists. Not populated during game start.

**Implementation**:
```rust
// src/game/game_loop.rs — ADD scene field + populate in start_game()
pub struct GameLoop {
    pub scene: SceneGraph,  // ADD
    // ... existing fields
}

pub fn start_game(&mut self) {
    // Populate scene from tilemap + entities
    let root = self.scene.add_root("World");
    // Add tilemap nodes, NPC nodes, player node
    for (pos, tile) in self.world_map.tiles() {
        let entity = self.world.spawn();
        self.world.add_component(entity, Transform::from_tile(pos));
        self.world.add_component(entity, Sprite::from_tile(tile));
        self.scene.attach(root, entity);
    }
}
```

**Estimated effort**: 2 hours

### 4.5 Camera Integration (Priority: P1 — blocks viewport control)

**Current state**: `Camera` struct exists with `position` and `zoom`. No `follow()`.

**Implementation**:
```rust
// src/engine/camera.rs — ADD follow method
impl Camera {
    pub fn follow(&mut self, target: Vec2, lerp: f32, dt: f32) {
        self.position.x += (target.x - self.position.x) * lerp * dt;
        self.position.y += (target.y - self.position.y) * lerp * dt;
    }

    pub fn world_to_screen(&self, pos: Vec2) -> Vec2 {
        (pos - self.position) * self.zoom + Vec2::new(
            self.viewport_width / 2.0,
            self.viewport_height / 2.0,
        )
    }

    pub fn visible_rect(&self) -> Rect {
        let hw = self.viewport_width / (2.0 * self.zoom);
        let hh = self.viewport_height / (2.0 * self.zoom);
        Rect::new(
            self.position.x - hw,
            self.position.y - hh,
            self.viewport_width / self.zoom,
            self.viewport_height / self.zoom,
        )
    }
}
```

**Estimated effort**: 1 hour

### 4.6 GameState Machine (Priority: P2 — blocks scene transitions)

**Current state**: No state machine.

**Implementation**:
```rust
// src/engine/state.rs — NEW FILE
pub enum GameState {
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Inventory,
    Crafting,
}

pub struct StateMachine {
    current: GameState,
    on_enter: HashMap<TypeId, Box<dyn FnMut()>>,
    on_exit: HashMap<TypeId, Box<dyn FnMut()>>,
}

impl StateMachine {
    pub fn transition(&mut self, next: GameState) {
        // Call on_exit for current, on_enter for next
    }
}
```

**Estimated effort**: 3 hours

### 4.7 Entity Groups (Priority: P2 — blocks group queries)

**Current state**: No group system.

**Implementation**:
```rust
// src/core/world.rs — ADD to UniversalWorld
pub struct EntityGroups {
    groups: HashMap<String, HashSet<EntityId>>,
}

impl EntityGroups {
    pub fn add(&mut self, group: &str, entity: EntityId);
    pub fn remove(&mut self, group: &str, entity: EntityId);
    pub fn query(&self, group: &str) -> &HashSet<EntityId>;
}
```

**Estimated effort**: 1 hour

### 4.8 Stat Modifier System (Priority: P3 — blocks RPG mechanics)

**Current state**: No stat system.

**Implementation**:
```rust
// src/game/stats.rs — NEW FILE
pub enum StatOp { Add, Multiply, Set }

pub struct StatModifier {
    pub stat: String,
    pub op: StatOp,
    pub value: f32,
    pub source: String,
    pub duration: Option<f32>, // None = permanent
}

pub struct Attributes {
    pub base: HashMap<String, f32>,
    pub modifiers: Vec<StatModifier>,
}

impl Attributes {
    pub fn get(&self, stat: &str) -> f32 {
        let base = self.base.get(stat).copied().unwrap_or(0.0);
        self.modifiers.iter().fold(base, |acc, m| match m.op {
            StatOp::Add => acc + m.value,
            StatOp::Multiply => acc * m.value,
            StatOp::Set => m.value,
        })
    }
}
```

**Estimated effort**: 3 hours

---

## 5. Cross-Domain Misalignment Fixes

### 5.1 Orphaned Module Cleanup

| Module | Status | Action |
|--------|--------|--------|
| `game/menu.rs` | Orphaned — no imports, no consumers | **DELETE** — menu logic belongs in UI layer, not game logic |
| `ui/dialogue_ui.rs` | Orphaned — dialogue rendering duplicated | **DELETE** if `ui/dialogue.rs` exists; otherwise **CREATE** proper dialogue UI |
| `game/energy.rs` | Partially orphaned — `Energy` struct exists but unused in game loop | **INTEGRATE** — wire into `GameLoop.tick()` as resource |

### 5.2 Component Implant Migration

| Component | Current | Target | Migration Path |
|-----------|---------|--------|----------------|
| `Transform` | Defined in `engine/renderer.rs:102` | `core/components.rs` | Extract struct + `Component` impl to core |
| `Sprite` | Defined in `engine/renderer.rs:120` | `core/components.rs` | Extract struct + `Component` impl to core |
| `RigidBody` | Defined in `engine/physics.rs:22` | `core/components.rs` | Extract struct + `Component` impl to core |
| `Collider` | Defined in `engine/physics.rs:62` | `core/components.rs` | Extract struct + `Component` impl to core |
| `Name` | Not yet created | `core/components.rs` | Create `Name(String)` component |

### 5.3 Error Handling Unification

All game modules should use `Result<T, String>` consistently:
- `game/farming.rs`: Already uses `Result` — verify no `unwrap()` in production paths
- `game/npc.rs`: Uses direct struct access — add `Result` returns for fallible operations
- `game/dialogue.rs`: Uses `Option` for node lookup — keep `Option` (correct semantics)
- `game/game_loop.rs`: Uses `panic!()` for missing systems — convert to `Result` with fallback
- `engine/physics.rs`: Uses `debug_assert!()` — keep (correct for debug checks)

### 5.4 EventBus Game Loop Integration

```rust
// In GameLoop::tick():
pub fn tick(&mut self, dt: f32) {
    // 1. Process events from last frame
    self.event_bus.process_all(&mut self.world);

    // 2. Run systems
    self.physics.step(dt);
    self.time.advance(dt);
    self.farming.update(&mut self.world, &self.time);
    self.npc_system.update(&mut self.world, &self.time);
    self.dialogue_system.update(&mut self.world);

    // 3. Collect new events
    self.event_bus.flush();

    // 4. Scene graph update
    self.scene.update_transforms();
}
```

---

## 6. Stardew Valley → Consciousness Valley Mapping

How each core Stardew Valley system maps to NT-World-Sim implementation:

### 6.1 Implemented Systems

| Stardew System | NT-World-Sim Module | Status | Notes |
|----------------|-------------------|--------|-------|
| **Daily loop** (6AM→2AM) | `game/time.rs` — `GameTime` + `TimeSystem` | ✅ Implemented | `GameTime { hour, minute, day, season }` + `advance()` method |
| **Farming** (till→plant→water→harvest) | `game/farming.rs` + `game/crops.rs` | ✅ Implemented | `FarmPlot`, `Crop`, `CropStage`, `FarmingSystem` |
| **NPCs** (schedule, dialogue, gifts) | `game/npc.rs` + `game/npcs.rs` + `game/dialogue.rs` | ✅ Implemented | `Npc`, `NpcDefinition`, `DialogueTree` |
| **Crafting** (recipe + ingredients) | `game/crafting.rs` + `game/recipes.rs` | ✅ Implemented | `CraftingSystem`, `Recipe` |
| **Inventory** (items, stacking) | `game/inventory.rs` + `game/item.rs` | ✅ Implemented | `Inventory`, `Item`, `ItemStack` |
| **Seasons** (spring→summer→fall→winter) | `game/season.rs` | ✅ Implemented | `Season` enum + `SeasonEffects` |
| **Weather** (sunny, rainy, stormy) | `game/weather.rs` | ✅ Implemented | `Weather` enum + weather system |
| **Energy** (stamina, sleep recovery) | `game/energy.rs` | ✅ Implemented | `Energy` struct |

### 6.2 Partially Implemented Systems

| Stardew System | NT-World-Sim Module | Gap | Minimal Fix |
|----------------|-------------------|-----|-------------|
| **World generation** (farm, town, mine) | `world/generator.rs` | Generates tiles but no POI placement | Add `place_buildings()` after tile generation |
| **Pathfinding** (NPC movement) | `world/pathfinding.rs` | A* exists but not wired to NPC schedule | Add `NpcMovementSystem` that calls `find_path()` |
| **Tilemap rendering** | `engine/renderer.rs` — `TileMap` | Renders but no chunk-based culling | Add `chunked_render(camera, chunk_size)` |
| **Camera** | `engine/camera.rs` | Basic position/zoom, no follow | Add `Camera::follow(target, lerp, dt)` |

### 6.3 Planned Systems (Not Yet Implemented)

| Stardew System | Target Module | Implementation Path | Priority |
|----------------|--------------|---------------------|----------|
| **Skills** (farming, mining, fishing, foraging) | `game/skills.rs` — NEW | `SkillTree` component + `SkillSystem` resource. XP on action, level up, unlock recipes | P2 |
| **Combat** (melee, ranged, monsters) | `game/combat.rs` — NEW | `CombatSystem` + `Health` component + `DamageEvent`. Simple turn-based or real-time | P2 |
| **Shop/Economy** (buy, sell, prices) | `game/shop.rs` — NEW | `ShopInventory` + `Economy` resource + `ShopSystem`. Gold as currency | P2 |
| **Relationships** (friendship points, dating) | Extend `game/npc.rs` | Add `friendship: u32` to `Npc`, `GiftSystem`, `RelationshipEvent` | P2 |
| **Mine** (floors, rocks, ore, gems) | `game/mine.rs` — NEW | `MineFloor` + `MineSystem` + ore generation per floor | P3 |
| **Fishing** (timing, quality) | `game/fishing.rs` — NEW | `FishingRod` + `FishingSystem` + minigame state machine | P3 |
| **Building upgrades** (house, barn, coop) | `game/building.rs` — NEW | `Building` component + `UpgradeSystem` + resource costs | P3 |
| **Mail/Quests** | `game/quests.rs` — NEW | `Quest` + `QuestSystem` + mailbox UI | P3 |

### 6.4 Consciousness Valley Unique Systems

These are NT-World-Sim additions that go beyond Stardew Valley:

| System | Module | Purpose |
|--------|--------|---------|
| **Resonance** | `game/npc.rs` — `resonance` field | NPC relationship uses "resonance" instead of friendship points — thematic for consciousness simulation |
| **Awareness** | `game/npcs.rs` — `NpcType::Awareness` | NPCs represent cognitive faculties (Awareness, Focus, Creativity, Empathy) — not just villagers |
| **Meta-cognition events** | `game/dialogue.rs` | Dialogue trees can trigger self-reflection mechanics |
| **Universal engine adapters** | `adapters/` | Same game can run on Bevy, Godot, or web — cross-platform by design |

---

## 7. Quick Game Restoration Guide

Step-by-step process to restore any game from a reference (Stardew Valley, Harvest Moon, Rune Factory, etc.):

### Step 1: Identify Game Systems from Reference

Analyze the reference game and list all systems:

```
Example: Stardew Valley analysis
├── Time system (day/night cycle, seasons)
├── Farming (till, plant, water, harvest)
├── NPCs (schedule, dialogue, gifts, marriage)
├── Inventory (items, tools, seeds, crops)
├── Crafting (recipes, workbench)
├── Mining (floors, rocks, ore)
├── Combat (monsters, weapons, health)
├── Economy (shop, shipping bin, gold)
├── Skills (leveling, unlocks)
├── Weather (rain, storms, effects on crops)
└── Events (festivals, cutscenes)
```

### Step 2: Map to NT-World-Sim Abstractions

For each identified system, find the NT-World-Sim equivalent:

| Game System | NT-World-Sim Abstraction | Module |
|-------------|-------------------------|--------|
| Time | `GameTime` resource + `TimeSystem` | `game/time.rs` |
| Farming | `FarmPlot` + `Crop` + `FarmingSystem` | `game/farming.rs` |
| NPCs | `Npc` + `NpcSchedule` + `DialogueTree` | `game/npc.rs` |
| Inventory | `Inventory` + `Item` + `ItemStack` | `game/inventory.rs` |
| Crafting | `CraftingSystem` + `Recipe` | `game/crafting.rs` |
| Physics/Collision | `SimplePhysicsWorld` + `Collider` | `engine/physics.rs` |
| Rendering | `TileMap` + `Sprite` + `Camera` | `engine/renderer.rs` |
| Input | `InputState` + `InputMap` | `engine/input.rs` |
| Audio | `AudioManager` + `AudioBackend` | `engine/audio.rs` |
| UI | `Widget tree` + `UiLayout` | `ui/widget.rs` |

### Step 3: Implement Missing Systems

For systems without an NT-World-Sim equivalent, create minimal implementations:

```rust
// Example: Adding fishing to Consciousness Valley
// 1. Define components
#[derive(Component)]
pub struct FishingSpot { pub quality: u32 }

#[derive(Component)]
pub struct FishingState { pub casting: bool, pub timer: f32 }

// 2. Define system
pub struct FishingSystem;
impl UniversalSystem for FishingSystem {
    fn name(&self) -> &str { "FishingSystem" }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        // Query FishingSpot + Player nearby
        // Start minigame on interact
        // Timer-based catch resolution
    }
}

// 3. Register in game loop
game_loop.add_system(Box::new(FishingSystem), 15);
```

### Step 4: Connect via GameLoop

Wire all systems into the game loop with correct ordering:

```rust
pub fn build_game_loop() -> GameLoop {
    let mut loop = GameLoop::new();

    // Phase 0: Input (priority -100)
    loop.add_system(Box::new(InputSystem), -100);

    // Phase 1: Physics (priority 0)
    loop.add_system(Box::new(PhysicsSystem), 0);

    // Phase 2: Gameplay (priority 10-19)
    loop.add_system(Box::new(TimeSystem), 10);
    loop.add_system(Box::new(FarmingSystem), 11);
    loop.add_system(Box::new(NpcScheduleSystem), 12);
    loop.add_system(Box::new(NpcMovementSystem), 13);
    loop.add_system(Box::new(DialogueSystem), 14);
    loop.add_system(Box::new(FishingSystem), 15);  // NEW
    loop.add_system(Box::new(MiningSystem), 16);   // NEW
    loop.add_system(Box::new(CombatSystem), 17);   // NEW

    // Phase 3: UI (priority 20)
    loop.add_system(Box::new(UiSystem), 20);

    // Phase 4: Rendering (priority 100)
    loop.add_system(Box::new(RenderSystem), 100);

    loop
}
```

### Step 5: Build UI with StardewTheme

Use the existing UI framework for all game screens:

```rust
// ui/stardew_theme.rs provides:
// - Pixel-art style fonts
// - Brown/earth-tone color palette
// - Standard layouts (dialogue box, inventory grid, shop menu)

fn build_dialogue_ui(npc: &Npc, tree: &DialogueTree) -> WidgetNode {
    WidgetNode::panel(StardewTheme::dialogue_box())
        .child(WidgetNode::portrait(&npc.name))
        .child(WidgetNode::text(&tree.current_text()))
        .child(WidgetNode::choices(&tree.choices()))
}
```

### Step 6: Test with HTML5 Prototype

Use the CanvasRenderer backend for browser testing:

```bash
# Build for web
wasm-pack build --target web --out-dir www/pkg

# Open www/index.html in browser
# All systems run via WASM — no native dependencies
```

### Restoration Checklist

- [ ] All game systems identified from reference
- [ ] Each system mapped to NT-World-Sim abstraction
- [ ] Missing systems implemented (minimal viable)
- [ ] All systems registered in GameLoop with correct priority
- [ ] EventBus wired for inter-system communication
- [ ] UI screens built with StardewTheme
- [ ] Input mapped via InputMap
- [ ] Camera following player
- [ ] SceneGraph populated at game start
- [ ] Save/load working via SaveSystem
- [ ] HTML5 build compiles and runs
- [ ] Core gameplay loop functional (15-min session)

---

## Appendix A: Module Dependency Graph

```
nt-world-sim/
├── src/core/           ← Foundation layer (no dependencies)
│   ├── entity.rs       ← EntityId, UniversalEntity, Archetype
│   ├── world.rs        ← UniversalWorld, Component, Resource, Event traits
│   ├── scheduler.rs    ← UniversalSystem, ParallelScheduler
│   └── change_detection.rs
│
├── src/engine/         ← Engine layer (depends on core)
│   ├── renderer.rs     ← Renderer trait, CanvasRenderer, Camera, Sprite, TileMap
│   ├── physics.rs      ← SimplePhysicsWorld, Collider, RigidBody
│   ├── input.rs        ← InputState, KeyCode, InputProvider
│   ├── audio.rs        ← AudioManager, AudioBackend
│   ├── scene.rs        ← SceneGraph, SceneNode
│   ├── event_bus.rs    ← TypedEventBus
│   ├── asset.rs        ← AssetServer (planned)
│   └── camera.rs       ← Camera (extends renderer.rs Camera)
│
├── src/game/           ← Game layer (depends on core + engine)
│   ├── game_loop.rs    ← GameLoop (orchestrates everything)
│   ├── time.rs         ← GameTime, TimeSystem
│   ├── farming.rs      ← FarmPlot, FarmingSystem
│   ├── crops.rs        ← Crop, CropStage
│   ├── npc.rs          ← Npc, NpcRole, Dialogue integration
│   ├── npcs.rs         ← NpcDefinition, NpcSchedule (to merge with npc.rs)
│   ├── dialogue.rs     ← DialogueTree, DialogueNode
│   ├── inventory.rs    ← Inventory, ItemStack
│   ├── item.rs         ← Item, ItemType
│   ├── crafting.rs     ← CraftingSystem
│   ├── recipes.rs      ← Recipe
│   ├── season.rs       ← Season, SeasonEffects
│   ├── weather.rs      ← Weather
│   └── energy.rs       ← Energy
│
├── src/world/          ← World generation (depends on core)
│   ├── generator.rs    ← WorldGenerator, TileType
│   ├── tile.rs         ← Tile
│   ├── pathfinding.rs  ← AStar
│   └── zone.rs         ← Zone
│
├── src/ui/             ← UI layer (depends on engine)
│   └── widget.rs       ← Widget, UiLayout, UiStyle
│
├── src/adapters/       ← Engine adapters (optional, depends on engine)
│   └── (bevy, godot, unity adapters)
│
└── src/save/           ← Persistence (depends on core)
    └── (save/load system)
```

## Appendix B: Build & Test Commands

```bash
# Build (all targets)
cargo build -p nt-world-sim

# Check (fast)
cargo check -p nt-world-sim

# Test
cargo test -p nt-world-sim

# Build for web
wasm-pack build --target web --out-dir www/pkg

# Run locally
cargo run -p nt-world-sim

# Run tests specifically for nt-world-sim
cargo test -p nt-world-sim --lib
```

## Appendix C: Key File Locations

| File | Lines | Purpose |
|------|-------|---------|
| `src/core/entity.rs` | 207 | EntityId, UniversalEntity, Archetype, Chunk |
| `src/core/world.rs` | — | UniversalWorld, Component/Resource/Event traits |
| `src/core/scheduler.rs` | — | UniversalSystem trait, ParallelScheduler |
| `src/engine/renderer.rs` | — | Renderer trait, Camera, Sprite, TileMap, DrawCommand |
| `src/engine/physics.rs` | — | SimplePhysicsWorld, RigidBody, Collider |
| `src/engine/input.rs` | — | InputState, KeyCode, InputProvider |
| `src/engine/scene.rs` | — | SceneGraph, SceneNode |
| `src/engine/event_bus.rs` | — | TypedEventBus |
| `src/engine/audio.rs` | — | AudioManager, AudioBackend |
| `src/engine/camera.rs` | — | Camera (extended) |
| `src/game/game_loop.rs` | — | GameLoop orchestrator |
| `src/game/farming.rs` | — | FarmPlot, FarmingSystem |
| `src/game/npc.rs` | 125 | Npc, NpcRole, Position |
| `src/game/npcs.rs` | 103 | NpcDefinition, NpcType, schedules |
| `src/game/dialogue.rs` | — | DialogueTree, DialogueNode |
| `src/game/time.rs` | — | GameTime, TimeSystem |
| `src/world/generator.rs` | — | WorldGenerator, TileType |
| `src/world/pathfinding.rs` | — | AStar pathfinding |
| `src/ui/widget.rs` | — | Widget, UiLayout, UiStyle |
