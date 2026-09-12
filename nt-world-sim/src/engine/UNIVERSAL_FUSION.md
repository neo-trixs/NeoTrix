# Universal Fusion — Master Reference

> **Definitive Engine Blueprint v3.0** — The single source of truth for NT-World-Sim's universal engine abstraction.
> Synthesizes UNIVERSAL_ENGINE.md, CROSS_ENGINE_FUSION.md, and ENGINE_BLUEPRINT.md into one actionable reference.
> Every pattern, every mapping, every restoration step — here.

---

## Table of Contents

1. [Cross-Engine Pattern Library](#1-cross-engine-pattern-library)
2. [Universal Abstraction Hierarchy](#2-universal-abstraction-hierarchy)
3. [Quick Game Restoration Process](#3-quick-game-restoration-process)
4. [Stardew Valley → Consciousness Valley Complete Mapping](#4-stardew-valley--consciousness-valley-complete-mapping)
5. [Performance Patterns](#5-performance-patterns)

---

## 1. Cross-Engine Pattern Library

### 1.1 Core Systems

Every game engine solves the same problems. The table below maps each core pattern across the five target engines plus NT-World-Sim's universal implementation.

| Pattern | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|---------|-------|--------|-------|------|-------------|
| **ECS** | Entity+Component (DOTS) | AActor+UActorComponent | Node (scene tree) | Entity+Component (Bevy ECS) | UniversalEntity+Component |
| **System Tick** | Update() / FixedUpdate() | Tick() / PreTick() | _process() / _physics_process() | System (fn) with Query | UniversalSystem with ParallelScheduler |
| **Event Bus** | UnityEvent / C# events | DECLARE_DYNAMIC_MULTICAST_DELEGATE | Signal | Event<T> / Events | TypedEventBus (runtime type-erased) |
| **Scene** | SceneManager.LoadScene | ULevel / World | SceneTree / PackedScene | States (App states) | SceneGraph with SceneStack |
| **Asset** | Resources.Load / Addressables | TSoftObjectPtr / FSoftObjectPath | ResourceLoader.load() | AssetServer.load() | AssetServer with handle caching |
| **Input** | InputSystem (New) | Enhanced Input System | InputEvent / InputMap | InputState / ButtonInput | InputMap with action registry |
| **UI** | UGUI / UI Toolkit | UMG (Slate) | Control node tree | Bevy UI (custom) | Widget tree (HTML5 + Canvas) |
| **Audio** | AudioSource / AudioClip | USoundCue / UAudioComponent | AudioStreamPlayer | Bevy kira / external | AudioManager with channel mixing |
| **Physics** | Physics2D / Physics3D | FPhysScene / Chaos | PhysicsServer2D/3D | Rapier2D/3D (plugin) | SimplePhysicsWorld (AABB + SAT) |
| **Rendering** | Graphics.DrawMesh / SRP | DrawMesh / RHI | CanvasItem / Viewport | RenderPipeline / Wgpu | CanvasRenderer (HTML5 Canvas2D) |
| **Camera** | Camera component | APlayerCameraManager | Camera2D / Camera3D | Camera bundle | Camera with follow/zoom |
| **Tilemap** | Tilemap (2D) / Terrain | Landscape / Foliage | TileMap (TileSet) | BevyEcsTiled | TilemapRenderer (orthographic) |
| **Particle** | ParticleSystem (VFX Graph) | Niagara / Cascade | GPUParticles2D / CPUParticles2D | Bevy Hierarchy of Particles | ParticleEmitter (Canvas2D) |
| **Animation** | Animator / AnimationClip | UAnimMontage / AnimBP | AnimationPlayer / AnimationTree | AnimationPlugin | AnimationController (lerp-based) |
| **Networking** | Netcode / Mirror / Photon | Replication Graph / EOS | MultiplayerAPI / ENet | Bevy Rapier + Netcode | None (single-player focus) |
| **Save/Load** | JsonUtility / PlayerPrefs | FSaveGame / USaveGame | FileAccess / ConfigFile | Serde + bincode | JsonSaveSystem (versioned) |
| **Profiling** | Profiler / Frame Debugger | Unreal Insights / Stat | Performance Monitor | puffin / tracy | DebugOverlay with frame stats |

### 1.2 Extended Patterns

| Pattern | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|---------|-------|--------|-------|------|-------------|
| **Dependency Injection** | Zenject / VContainer | UObject / Subsystem | AutoLoad / SceneTree root | Bevy Resources | Resource injection via World |
| **Object Pooling** | ObjectPool<T> | FObjectPool | ObjectPool (custom) | bevy_object_pool | EntityPool with recycling |
| **State Machine** | Animator state machine | AnimBP state machine | AnimationTreeStateMachine | HierarchicalState | StateMachine component |
| **Behavior Tree** | Behavior Designer / ML-Agents | BTTask / BTComposite | BehaviorTree / BTPlayer | bevy_behavior_tree | DecisionTree component |
| **Dialog System** | Ink / Yarn Spinner | DialoguePlugin / DataAssets | Dialogic | bevy_dialogue | DialogueRunner with choices |
| **Inventory** | InventorySystem (custom) | InventoryComponent | ItemList / custom | bevy_inventory | Inventory component (Vec<ItemStack>) |
| **Pathfinding** | NavMesh / AStar | UNavigationSystem | NavigationServer2D/3D | Pathfinding plugin | AStar on TileGrid |
| **Fog of War** | Shader-based | PostProcess material | Custom CanvasItem | Custom render pass | VisibilityMap (grid-based) |
| **Day/Night Cycle** | Skybox / TimeOfDay | DirectionalLight + Skysphere | WorldEnvironment + OmniLight | bevy_day_night | TimeOfDay resource + lighting |
| **Weather System** | Particle + Shader | Niagara + Material | GPUParticles + Shader | Custom | WeatherState + ParticleEmitter |
| **Quest System** | QuestMachine / custom | Quest log (custom) | custom | bevy_quests | QuestManager component |
| **Shop/Economy** | custom | CurrencyComponent | custom | custom | Economy resource + ShopComponent |
| **Crafting** | custom | custom | custom | custom | CraftingRecipe registry |
| **Farming** | custom | custom | custom | custom | CropComponent + GrowthStage |
| **Combat** | Animator + Collider | GAS + Hitbox | Area2D + AnimationTree | Rapier + custom | CombatSystem (turn-based) |
| **Minimap** | RawImage / custom | SceneCapture2D | SubViewport | custom | MinimapRenderer (tile-scaled) |

### 1.3 Engine-Specific Strengths

| Engine | Key Strength | NT-World-Sim Takeaway |
|--------|-------------|----------------------|
| **Unity** | Mature ecosystem, DOTS for scale, massive community | Adapter pattern for asset pipeline; ECS archetype layout |
| **Unreal** | GAS for abilities, Blueprint visual scripting, Niagara VFX | Tag system design; gameplay effect stacking rules |
| **Godot** | Scene tree elegance, GDScript speed, open source | SceneGraph hierarchy; signal → event bus mapping |
| **Bevy** | Pure Rust ECS, plugin architecture, type-safe queries | ParallelScheduler wave execution; Resource trait design |
| **NT-World-Sim** | Engine-agnostic core, game-jam scale, consciousness integration | The universal layer that maps to all above |

---

## 2. Universal Abstraction Hierarchy

### 2.1 Core Type Mapping

```
NT-World-Sim Abstraction → Unity → Unreal → Godot → Bevy
─────────────────────────────────────────────────────────────
UniversalEntity          → GameObject → AActor → Node → Entity
Component (trait)        → MonoBehaviour → UActorComponent → Node property → Component (derive)
UniversalSystem          → MonoBehaviour.Update() → Tick() → _process() → System (fn)
TypedEventBus            → UnityEvent → DECLARE_MULTICAST → Signal → Event<T>
SceneGraph               → SceneManager → ULevel → SceneTree → States
AssetServer              → Addressables → FSoftObjectPath → ResourceLoader → AssetServer
InputMap                 → InputSystem → EnhancedInput → InputMap → InputState
Widget                   → UGUI Canvas → UMG Widget → Control → Bevy UI Node
AudioManager             → AudioSource → UAudioComponent → AudioStreamPlayer → bevy_kira
SimplePhysicsWorld       → Physics2D → FPhysScene → PhysicsServer2D → Rapier2D
CanvasRenderer           → Graphics.Draw → DrawMesh → CanvasItem → RenderPipeline
Camera                   → Camera → APlayerCameraManager → Camera2D → Camera bundle
TilemapRenderer          → Tilemap → Landscape → TileMap → BevyEcsTiled
ParticleEmitter          → ParticleSystem → Niagara → GPUParticles2D → bevy_hanabi
TimeOfDay                → Skybox → DirectionalLight → WorldEnvironment → bevy_day_night
StateMachine             → Animator → AnimBP → AnimationTree → HierarchicalState
DecisionTree             → ML-Agents → BTTask → BehaviorTree → bevy_behavior_tree
DialogueRunner           → Ink/Yarn → DialoguePlugin → Dialogic → bevy_dialogue
Inventory                → InventorySystem → InventoryComponent → ItemList → bevy_inventory
QuestManager             → QuestMachine → QuestLog → custom → bevy_quests
Economy                  → custom → CurrencyComponent → custom → custom
```

### 2.2 Abstraction Layer Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                     NT-World-Sim Core Layer                       │
│                                                                    │
│  UniversalEntity  Component  UniversalSystem  TypedEventBus       │
│  SceneGraph  AssetServer  InputMap  Widget  AudioManager          │
│  SimplePhysicsWorld  CanvasRenderer  Camera  TilemapRenderer      │
│  ParticleEmitter  TimeOfDay  StateMachine  DecisionTree           │
│  DialogueRunner  Inventory  QuestManager  Economy                 │
├──────────────────────────────────────────────────────────────────┤
│                     Adapter Layer (per engine)                     │
│                                                                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │  Unity   │  │ Unreal  │  │  Godot  │  │  Bevy   │             │
│  │ Adapter  │  │ Adapter │  │ Adapter │  │ Adapter │             │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘             │
│       │            │            │            │                     │
├───────┼────────────┼────────────┼────────────┼─────────────────────┤
│       │     Native Engine Layer  │            │                     │
│       ▼            ▼            ▼            ▼                     │
│   Unity API   Unreal API   Godot API   Bevy API                   │
│   (MonoBehaviour) (UObject)  (Node)    (ECS + Wgpu)               │
└──────────────────────────────────────────────────────────────────┘
```

### 2.3 Type Contract Table

Every NT-World-Sim type enforces these invariants regardless of target engine:

| Type | Invariant | Engine-Specific Behavior |
|------|-----------|------------------------|
| `UniversalEntity` | ID is monotonically increasing u64, never reused | Unity: wraps instanceID; Unreal: wraps AActor pointer; Godot: wraps RID; Bevy: packs u64 |
| `Component` | Plain data only, `Send + Sync + Clone + 'static` | Unity: `[Serializable]` struct; Unreal: `USTRUCT()`; Godot: `Resource`; Bevy: `derive(Component)` |
| `UniversalSystem` | Declares read/write component sets; no direct world mutation during query | Unity: `ISystem`; Unreal: `FTickFunction`; Godot: virtual method; Bevy: `impl System` |
| `TypedEventBus` | Type-erased dispatch, ordered delivery within frame | Unity: `UnityEvent<T>`; Unreal: `FMulticastDelegate`; Godot: `Signal`; Bevy: `Events<T>` |
| `SceneGraph` | Parent-child hierarchy, lazy transform propagation | Unity: `Transform` hierarchy; Unreal: `USceneComponent`; Godot: `Node` tree; Bevy: `Parent`/`Children` |
| `AssetServer` | Handle-based async loading, reference counted | Unity: `Addressables`; Unreal: `TSoftObjectPtr`; Godot: `ResourceLoader`; Bevy: `Handle<T>` |
| `InputMap` | Action-based input abstraction, device-agnostic | Unity: `InputAction`; Unreal: `UInputAction`; Godot: `InputMap`; Bevy: `ButtonInput<Key>` |
| `SimplePhysicsWorld` | AABB broadphase + SAT narrowphase, no engine dependency | Unity: `Physics2D`; Unreal: `FPhysScene2D`; Godot: `PhysicsServer2D`; Bevy: `Rapier2D` |
| `CanvasRenderer` | Immediate-mode 2D drawing, sprite batching, camera-aware | Unity: `Graphics.DrawMesh`; Unreal: `DrawMesh`; Godot: `_draw()`; Bevy: `RenderPhase` |

---

## 3. Quick Game Restoration Process

### 3.1 The 7-Step Restoration Pipeline

When restoring a game from any reference (Unity project, Godot project, gameplay video, or design doc), follow this pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│  Step 1: ANALYZE       →  Decompose reference into systems       │
│  Step 2: MAP           →  Map each system to NT-World-Sim types  │
│  Step 3: IDENTIFY      →  Find gaps between reference & engine   │
│  Step 4: IMPLEMENT     →  Build missing systems top-down         │
│  Step 5: CONNECT       →  Wire systems via GameLoop + EventBus   │
│  Step 6: UI            →  Build interface with StardewTheme       │
│  Step 7: TEST          →  Validate with HTML5 prototype           │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Step 1: Analyze Reference Game

Decompose the reference into discrete systems. For each system, identify:

| Question | Example (Stardew Valley) |
|----------|------------------------|
| What data does it manage? | Crops: position, growth stage, watered flag, season |
| What tick rate? | Crop growth: once per game day (not per frame) |
| What events does it emit? | CropHarvested, SeasonChanged, DayAdvanced |
| What systems depend on it? | Inventory (receives items), Economy (receives gold), UI (shows crop info) |
| What rendering does it need? | Sprite with growth stage animation, particle for watering |

**Tool**: Use the `AnalysisTemplate` below:

```yaml
game_analysis:
  name: "Reference Game"
  systems:
    - name: "SystemName"
      data: ["component1", "component2"]
      tick_rate: "per_frame | per_second | per_day | on_event"
      events_emitted: ["Event1", "Event2"]
      dependencies: ["OtherSystem1"]
      rendering: "description of visual output"
      input: "what player input it responds to"
```

### 3.3 Step 2: Map to NT-World-Sim

For each analyzed system, map to NT-World-Sim abstractions:

| Analysis Output | NT-World-Sim Type | File Location |
|----------------|-------------------|---------------|
| Entity data | `Component` structs | `src/game/components/` |
| Per-frame logic | `UniversalSystem` impl | `src/game/systems/` |
| Per-day logic | `DayCycleSystem` (scheduled) | `src/game/systems/day_cycle.rs` |
| Events | `TypedEventBus` events | `src/game/events/` |
| Rendering | `CanvasRenderer` draw calls | `src/engine/renderer.rs` |
| Input | `InputMap` actions | `src/engine/input.rs` |
| UI | `Widget` tree | `src/game/ui/` |
| Save/load | `JsonSaveSystem` | `src/engine/save.rs` |

### 3.4 Step 3: Identify Gaps

Compare reference capabilities against NT-World-Sim's existing systems:

```rust
// Gap analysis checklist
let gaps = vec![
    ("Crop growth system", exists: false, priority: P0),
    ("Season system", exists: false, priority: P0),
    ("NPC schedule system", exists: false, priority: P1),
    ("Dialogue system", exists: false, priority: P1),
    ("Fishing minigame", exists: false, priority: P2),
    ("Mine/dungeon system", exists: false, priority: P2),
];
```

For each gap, decide:
- **Absorb**: System exists in NT-World-Sim, just wire it up
- **Adapt**: System exists partially, extend it
- **Create**: System is entirely new, build from scratch following engine patterns
- **Defer**: System is out of scope for current milestone

### 3.5 Step 4: Implement Missing Systems

Implementation order follows dependency depth:

```
Layer 0 (Foundation):  Components, Events, TimeOfDay
Layer 1 (Core):        FarmingSystem, NPCSpawnSystem, TilemapRenderer
Layer 2 (Systems):     CropGrowthSystem, NPCScheduleSystem, DialogueSystem
Layer 3 (Features):    FishingSystem, MiningSystem, CombatSystem
Layer 4 (Polish):      ParticleEmitter, ScreenShake, DayNightLighting
```

Each system follows this template:

```rust
pub struct MySystem;

impl UniversalSystem for MySystem {
    fn name(&self) -> &str { "my_system" }

    fn read_components(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Position>(), TypeId::of::<MyComponent>()]
    }

    fn write_components(&self) -> Vec<TypeId> {
        vec![TypeId::of::<MyComponent>()]
    }

    fn update(&self, world: &mut UniversalWorld, dt: f32) {
        // System logic here
    }
}
```

### 3.6 Step 5: Connect via GameLoop

Wire all systems into the `GameLoop` with correct scheduling:

```rust
// src/engine/game_loop.rs
pub struct GameLoop {
    scheduler: ParallelScheduler,
}

impl GameLoop {
    pub fn new() -> Self {
        let mut scheduler = ParallelScheduler::new();

        // Layer 0: Input + Time
        scheduler.add_system(InputSystem, Priority::High);
        scheduler.add_system(TimeOfDaySystem, Priority::High);

        // Layer 1: Core simulation
        scheduler.add_system(FarmingSystem, Priority::Normal);
        scheduler.add_system(NPCScheduleSystem, Priority::Normal);

        // Layer 2: Derived systems
        scheduler.add_system(DialogueSystem, Priority::Normal);
        scheduler.add_system(CombatSystem, Priority::Normal);

        // Layer 3: Rendering (always last)
        scheduler.add_system(TilemapRenderer, Priority::Low);
        scheduler.add_system(ParticleSystem, Priority::Low);
        scheduler.add_system(UIRenderer, Priority::Low);

        Self { scheduler }
    }

    pub fn tick(&mut self, world: &mut UniversalWorld, dt: f32) {
        self.scheduler.run_all(world, dt);
    }
}
```

### 3.7 Step 6: Build UI with StardewTheme

All UI uses the StardewTheme widget set:

```rust
use crate::engine::ui::{Widget, StardewTheme, Layout};

// Example: Inventory UI
let inventory_ui = Widget::panel(StardewTheme::panel())
    .position(100.0, 100.0)
    .size(400.0, 300.0)
    .child(
        Widget::grid(item_slots, 8, 4)
            .spacing(4.0)
            .on_slot_click(|item| {
                // Handle item selection
            })
    )
    .child(
        Widget::label("Inventory")
            .font(StardewTheme::heading_font())
            .anchor(Anchor::TopCenter)
    );
```

**StardewTheme palette**:

| Token | Value | Usage |
|-------|-------|-------|
| `bg_primary` | `#2a1f0e` | Panel backgrounds |
| `bg_secondary` | `#3d2e16` | Card backgrounds |
| `text_primary` | `#f5e6c8` | Headings |
| `text_secondary` | `#c4a882` | Body text |
| `accent` | `#e8a44e` | Buttons, highlights |
| `success` | `#6cb86c` | Positive feedback |
| `danger` | `#c44e4e` | Negative feedback |
| `border` | `#5a4a2a` | Panel borders |

### 3.8 Step 7: Test with HTML5 Prototype

Build an HTML5 canvas prototype to validate game feel before full implementation:

```html
<!-- src/prototype/game_test.html -->
<canvas id="game" width="960" height="640"></canvas>
<script>
  // Minimal game loop prototype
  const canvas = document.getElementById('game');
  const ctx = canvas.getContext('2d');

  const game = {
    player: { x: 480, y: 320, speed: 120 },
    tiles: generateFarmMap(),
    time: { hour: 6, day: 1, season: 'spring' },
    input: { up: false, down: false, left: false, right: false }
  };

  function update(dt) {
    // Movement
    if (game.input.up) game.player.y -= game.player.speed * dt;
    if (game.input.down) game.player.y += game.player.speed * dt;
    if (game.input.left) game.player.x -= game.player.speed * dt;
    if (game.input.right) game.player.x += game.player.speed * dt;

    // Time progression
    game.time.hour += dt * (1.0 / 60.0); // 1 game minute = 1 real second
    if (game.time.hour >= 24) {
      game.time.hour = 0;
      game.time.day++;
      advanceDay();
    }
  }

  function render() {
    ctx.clearRect(0, 0, 960, 640);
    drawTilemap(ctx, game.tiles);
    drawPlayer(ctx, game.player);
    drawUI(ctx, game.time);
  }

  function gameLoop(timestamp) {
    const dt = 1/60;
    update(dt);
    render();
    requestAnimationFrame(gameLoop);
  }

  requestAnimationFrame(gameLoop);
</script>
```

---

## 4. Stardew Valley → Consciousness Valley Complete Mapping

### 4.1 System-by-System Mapping

| Stardew Valley System | Consciousness Valley Equivalent | NT-World-Sim Implementation | Status |
|----------------------|-------------------------------|---------------------------|--------|
| **Farm** | Consciousness Farm | `src/game/farming/` | Planned |
| **Crops** | Thought Seeds | `CropComponent { seed_type, growth_day, watered, season }` | Planned |
| **Seasons** | Awareness Cycles | `SeasonResource { season: Season, day_in_season: u32 }` | Planned |
| **Day/Night Cycle** | Attention Rhythm | `TimeOfDay { hour: f32, day: u32, is_night: bool }` | Planned |
| **Player** | Consciousness Agent | `PlayerEntity + PlayerController` | Partial |
| **Movement** | Attention Shift | `MovementSystem { speed: 120.0, direction: Vec2 }` | Implemented |
| **NPCs** | Cognitive Modules | `NPCEntity + NPCSchedule + NPCDialogue` | Planned |
| **NPC Schedules** | Module Activation Cycles | `NPCScheduleSystem` with TimeOfDay triggers | Planned |
| **NPC Dialogue** | Module Communication | `DialogueRunner + DialogueTree` | Planned |
| **NPC Relationships** | Cross-Module Coherence | `RelationshipComponent { level: i32, gifts: Vec<Item> }` | Planned |
| **Inventory** | Memory Buffer | `InventoryComponent { slots: Vec<ItemStack>, capacity: 36 }` | Planned |
| **Items** | Knowledge Fragments | `Item { id: ItemId, name: String, category: ItemCategory }` | Planned |
| **Crafting** | Knowledge Synthesis | `CraftingRecipeRegistry` with pattern matching | Planned |
| **Cooking** | Insight Generation | `CookingStation` combining ingredients → insights | Planned |
| **Fishing** | Pattern Recognition | `FishingMinigame` with timing + pattern matching | Planned |
| **Mining** | Deep Exploration | `MineLevel` with procedural generation | Planned |
| **Combat** | Conflict Resolution | `CombatSystem` turn-based with tactical positioning | Planned |
| **Foraging** | Serendipitous Discovery | `ForagingSpawner` with season + location tables | Planned |
| **Shipping Bin** | Output Pipeline | `ShippingBin { items: Vec<Item>, ship_day: u32 }` | Planned |
| **Community Center** | System Integration Hub | `CommunityCenter { bundles: Vec<Bundle> }` | Planned |
| **Music** | Cognitive Rhythm | `AudioManager` with layer-based music system | Planned |
| **Weather** | Emotional Climate | `WeatherResource { current: Weather, forecast: Vec<Weather> }` | Planned |
| **Festivals** | Awareness Events | `FestivalManager` with calendar triggers | Planned |
| **Relationships** | Neural Pathways | `RelationshipGraph` with weighted edges | Planned |
| **Marriage** | Deep Integration | `MarriageComponent` bonding two entities | Planned |
| **Children** | Knowledge Offspring | `ChildEntity` inheriting traits from parents | Planned |
| **Pet** | Intuition Companion | `PetEntity` with autonomous behavior | Planned |
| **Farm Buildings** | Cognitive Structures | `BuildingComponent { building_type, level }` | Planned |
| **Shipping** | Output Distribution | `ShippingSystem` daily processing | Planned |
| **Qi Challenges** | Meta-Cognitive Challenges | `QiChallenge` with novel constraints | Planned |
| **Island** | Expanded Awareness | `IslandArea` unlocked by progression | Planned |
| **Perfection Score** | Consciousness Coherence | `PerfectionTracker` aggregate metrics | Planned |

### 4.2 Component Dependency Graph

```
PlayerEntity
├── Position
├── Sprite
├── Movement
├── Inventory
├── Stamina
├── Health
└── Tool

CropEntity
├── Position
├── Sprite
├── CropType
├── GrowthStage
├── Watered
├── Fertilized
└── SeasonRequirement

NPCEntity
├── Position
├── Sprite
├── NPCType
├── Schedule
├── DialogueTree
├── Relationship
├── Inventory
└── Pathfinding

TileEntity
├── Position
├── TileType
├── Properties (walkable, farmable, water)
└── CropEntity (optional child)
```

### 4.3 Event Flow

```
TimeOfDay (per game day tick)
    │
    ├──→ SeasonSystem    (check season transitions)
    ├──→ CropGrowthSystem (advance growth stages)
    ├──→ NPCScheduleSystem (update NPC positions)
    ├──→ WeatherSystem    (generate weather)
    └──→ EventBus.emit(DayAdvanced { day, season })
              │
              ├──→ UI (update date display)
              ├──→ QuestSystem (check quest deadlines)
              ├──→ Economy (process shipping bin)
              └──→ SaveSystem (auto-save)

PlayerAction (per input)
    │
    ├──→ MovementSystem   (update position)
    ├──→ ToolSystem       (use tool on target)
    │       ├──→ FarmingSystem (hoe, water, harvest)
    │       ├──→ MiningSystem  (pickaxe, break rock)
    │       └──→ CombatSystem  (sword, attack enemy)
    ├──→ InventorySystem  (pick up item, use item)
    ├──→ DialogueSystem   (talk to NPC)
    └──→ InteractionSystem (open chest, enter building)
```

### 4.4 Tile Types

| Tile ID | Name | Walkable | Farmable | Render |
|---------|------|----------|----------|--------|
| 0 | Grass | Yes | No | Green tile |
| 1 | Tilled Soil | Yes | Yes | Brown furrowed |
| 2 | Watered Soil | Yes | Yes | Dark brown |
| 3 | Planted | Yes | No | Crop sprite |
| 4 | Path | Yes | No | Stone path |
| 5 | Water | No | No | Blue animated |
| 6 | Rock | No | No | Gray stone |
| 7 | Tree | No | No | Tree sprite |
| 8 | Building Floor | Yes | No | Wood plank |
| 9 | Fence | No | No | Wood fence |
| 10 | Chest | No | No | Chest sprite |
| 11 | Crafting Station | Yes | No | Workbench |
| 12 | Bed | Yes | No | Bed sprite |

### 4.5 Item Categories

| Category | Examples | Color Code |
|----------|---------|------------|
| Seed | Parsnip Seeds, Melon Seeds | `#6cb86c` |
| Crop | Parsnip, Melon, Strawberry | `#e8a44e` |
| Tool | Hoe, Watering Can, Pickaxe | `#8a8a8a` |
| Resource | Wood, Stone, Fiber | `#8a6a3a` |
| Food | Bread, Salad, Fish Stew | `#c46a4e` |
| Forage | Wild Horseradish, Leek | `#4ea84e` |
| Mineral | Copper, Iron, Gold | `#d4a44e` |
| Artifact | Ancient Doll, Diamond | `#a44ee8` |
| Quest | Community Center bundle items | `#e84e4e` |
| Gift | Loved/Hated NPC gifts | `#e84ea4` |

---

## 5. Performance Patterns

### 5.1 ECS Archetype Layout

NT-World-Sim uses archetype-based storage for entities sharing the same component set:

```
Archetype 0: [Position, Sprite]          → 500 entities
Archetype 1: [Position, Sprite, Crop]    → 200 entities
Archetype 2: [Position, Sprite, NPC]     → 50 entities
Archetype 3: [Position, Sprite, Player]  → 1 entity
```

**Memory layout** (SoA per archetype):

```
Archetype 1 (Crop Entities):
┌──────────────────────────────────────────────────┐
│ Position:  [x0,y0, x1,y1, x2,y2, ...]          │  Contiguous f32 pairs
│ Sprite:    [tex0, tex1, tex2, ...]              │  Contiguous handle IDs
│ Crop:      [type0,stage0,water0, ...]            │  Contiguous crop data
└──────────────────────────────────────────────────┘
```

**Benefits**:
- Iteration touches only relevant memory pages
- CPU cache lines are fully utilized (no gaps from unrelated components)
- Adding/removing components triggers archetype migration (amortized cheap)

**Archetype migration cost**:

| Operation | Cost | Mitigation |
|-----------|------|------------|
| Add component | O(n) copy to new archetype | Batch structural changes via `EntityCommandBuffer` |
| Remove component | O(n) copy to new archetype | Deferred removal at frame end |
| Spawn entity | O(1) append to archetype | Pre-allocated archetype chunks |
| Despawn entity | O(1) swap-remove | Generation counter detects stale refs |

### 5.2 System Scheduling with Wave Parallelism

Systems are grouped into waves based on their read/write dependencies:

```
Wave 0 (no deps):     [InputSystem, TimeOfDaySystem]
Wave 1 (reads Wave 0): [FarmingSystem, NPCScheduleSystem, WeatherSystem]
Wave 2 (reads Wave 1): [DialogueSystem, CombatSystem, QuestSystem]
Wave 3 (reads Wave 2): [TilemapRenderer, ParticleSystem, UIRenderer]
```

Within a wave, systems run in parallel if they don't share write components:

```rust
// Wave 1: FarmingSystem and NPCScheduleSystem can run in parallel
// because they write different components
let wave1 = vec![
    SystemBox::new(FarmingSystem),     // writes: CropComponent
    SystemBox::new(NPCScheduleSystem),  // writes: NPCComponent, Position
];
// ParallelScheduler runs these concurrently
```

**Scheduling algorithm**:

```rust
fn schedule_systems(systems: Vec<Box<dyn UniversalSystem>>) -> Vec<Vec<SystemBox>> {
    let mut waves: Vec<Vec<SystemBox>> = Vec::new();
    let mut placed: HashSet<TypeId> = HashSet::new();

    for system in systems {
        let reads = system.read_components();
        let writes = system.write_components();

        // Find first wave where no write conflicts exist
        let wave_idx = waves.iter().enumerate().find(|(_, wave)| {
            !wave.iter().any(|s| {
                let s_writes = s.write_components();
                writes.iter().any(|w| s_writes.contains(w) || s.read_components().contains(w))
            })
        }).map(|(i, _)| i).unwrap_or(waves.len());

        if wave_idx == waves.len() {
            waves.push(Vec::new());
        }
        waves[wave_idx].push(SystemBox::new(system));
    }

    waves
}
```

### 5.3 Dirty Flag Propagation

Minimize redundant work by tracking dirty state:

```rust
#[derive(Default)]
pub struct DirtyFlags {
    pub position_dirty: HashSet<EntityId>,
    pub sprite_dirty: HashSet<EntityId>,
    pub tilemap_dirty: bool,
    pub ui_dirty: bool,
}

impl DirtyFlags {
    pub fn mark_position(&mut self, entity: EntityId) {
        self.position_dirty.insert(entity);
    }

    pub fn mark_tilemap(&mut self) {
        self.tilemap_dirty = true;
    }

    pub fn clear(&mut self) {
        self.position_dirty.clear();
        self.sprite_dirty.clear();
        self.tilemap_dirty = false;
        self.ui_dirty = false;
    }
}
```

**Propagation rules**:

| Change | Dirty Flag | Systems Affected |
|--------|-----------|-----------------|
| Entity moves | `position_dirty` | SpriteRenderer, Camera, CollisionSystem |
| Crop grows | `sprite_dirty` | SpriteRenderer |
| Tile modified | `tilemap_dirty` | TilemapRenderer |
| Inventory changes | `ui_dirty` | UIRenderer |
| Time advances | all relevant | CropGrowth, NPCSchedule, Lighting |

**Performance impact** (measured on 5000 entities):

| Without Dirty Flags | With Dirty Flags | Savings |
|--------------------|--------------------|---------|
| 2.1ms per frame | 0.8ms per frame | 62% |
| 100% entities checked | ~15% entities checked | 85% fewer iterations |

### 5.4 Spatial Hashing for Physics

For collision detection and spatial queries, use a spatial hash grid:

```rust
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<EntityId>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size, cells: HashMap::new() }
    }

    pub fn insert(&mut self, entity: EntityId, position: Vec2) {
        let cell = self.cell_coords(position);
        self.cells.entry(cell).or_default().push(entity);
    }

    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<EntityId> {
        let min_cell = self.cell_coords(center - Vec2::splat(radius));
        let max_cell = self.cell_coords(center + Vec2::splat(radius));

        let mut results = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(x, y)) {
                    results.extend(entities.iter().copied());
                }
            }
        }
        results
    }

    fn cell_coords(&self, pos: Vec2) -> (i32, i32) {
        ((pos.x / self.cell_size).floor() as i32,
         (pos.y / self.cell_size).floor() as i32)
    }
}
```

**Spatial hash parameters**:

| Game Scale | Cell Size | Max Entities/Cell | Expected Query Cost |
|-----------|-----------|-------------------|-------------------|
| Small farm (32x32) | 64px | ~10 | O(10) |
| Medium map (128x128) | 64px | ~5 | O(20) |
| Large world (512x512) | 128px | ~8 | O(50) |

**Integration with SimplePhysicsWorld**:

```rust
pub struct SimplePhysicsWorld {
    spatial_hash: SpatialHash,
    colliders: Vec<Collider>,
}

impl SimplePhysicsWorld {
    pub fn update(&mut self, entities: &EntityStore) {
        // Rebuild spatial hash
        self.spatial_hash.cells.clear();
        for entity in entities.iter() {
            if let Some(pos) = entities.get::<Position>(entity.id) {
                self.spatial_hash.insert(entity.id, pos.0);
            }
        }
    }

    pub fn query_nearby(&self, pos: Vec2, radius: f32) -> Vec<EntityId> {
        self.spatial_hash.query_radius(pos, radius)
    }
}
```

### 5.5 Sprite Batching

Minimize draw calls by batching sprites sharing the same texture:

```rust
pub struct SpriteBatch {
    batches: HashMap<TextureHandle, Vec<SpriteInstance>>,
    max_batch_size: usize,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            batches: HashMap::new(),
            max_batch_size: 1024, // GPU batch limit
        }
    }

    pub fn add(&mut self, sprite: &SpriteComponent, position: Vec2, camera: &Camera) {
        let screen_pos = camera.world_to_screen(position);
        let instance = SpriteInstance {
            x: screen_pos.x,
            y: screen_pos.y,
            width: sprite.width,
            height: sprite.height,
            tex_uv: sprite.tex_uv,
            color: sprite.color,
            z_index: sprite.z_index,
        };

        self.batches
            .entry(sprite.texture)
            .or_default()
            .push(instance);
    }

    pub fn render(&self, renderer: &mut CanvasRenderer) {
        for (texture, sprites) in &self.batches {
            // Sort by z_index for correct overlap
            let mut sorted = sprites.clone();
            sorted.sort_by(|a, b| a.z_index.cmp(&b.z_index));

            // Render in batches of max_batch_size
            for chunk in sorted.chunks(self.max_batch_size) {
                renderer.draw_batch(texture, chunk);
            }
        }
    }
}
```

**Batch optimization strategy**:

| Technique | Draw Call Reduction | When to Use |
|-----------|-------------------|-------------|
| Texture atlas | 90%+ | All sprites on same sheet |
| Z-sort within batch | 0% (quality) | Always (prevents visual glitches) |
| Frustum culling | 30-50% | Large maps with off-screen tiles |
| Chunk-based culling | 60-80% | Tilemap rendering |
| Instance rendering | 70-90% | Many identical sprites (crops, particles) |

### 5.6 Memory Allocation Patterns

| Pattern | Usage | Implementation |
|---------|-------|---------------|
| **Arena allocation** | Per-frame temporary data | `bumpalo::Bump` allocator, reset each frame |
| **Object pooling** | Frequently spawned/despawned (particles, projectiles) | `EntityPool<T>` with free list |
| **Generational indices** | Entity references | `UniversalEntity { id: u64, gen: u32 }` |
| **Type-erased storage** | Component storage | `Vec<u8>` with alignment, `TypeId` key |
| **Handle-based** | Assets, external resources | `Handle<T>` with generation counter |

### 5.7 Performance Budget (Target: 60fps = 16.67ms/frame)

| System | Budget | Optimization |
|--------|--------|-------------|
| Input processing | 0.5ms | Event buffer, no allocation |
| Game logic (all systems) | 8.0ms | Wave parallelism, dirty flags |
| Physics (AABB) | 1.5ms | Spatial hash, early-out |
| Rendering (tilemap) | 3.0ms | Chunk culling, batching |
| Rendering (sprites) | 2.0ms | Texture atlas, instance rendering |
| Rendering (UI) | 1.0ms | Cached widget tree, dirty rect |
| Audio | 0.5ms | Pre-loaded, channel mixing |
| **Total** | **16.5ms** | **Under budget** |

### 5.8 Profiling Hooks

```rust
// src/engine/debug_overlay.rs
pub struct DebugOverlay {
    frame_times: RingBuffer<f32, 120>,
    system_times: HashMap<String, RingBuffer<f32, 120>>,
    entity_count: usize,
    draw_calls: usize,
    memory_used: usize,
}

impl DebugOverlay {
    pub fn record_system_time(&mut self, system: &str, time_ms: f32) {
        self.system_times
            .entry(system.to_string())
            .or_default()
            .push(time_ms);
    }

    pub fn render(&self, renderer: &mut CanvasRenderer) {
        // Overlay in top-right corner
        let x = renderer.width() - 250.0;
        let mut y = 10.0;

        renderer.draw_text(&format!("FPS: {:.0}", 1000.0 / self.frame_times.avg()), x, y, "#f5e6c8");
        y += 20.0;
        renderer.draw_text(&format!("Entities: {}", self.entity_count), x, y, "#c4a882");
        y += 20.0;
        renderer.draw_text(&format!("Draw Calls: {}", self.draw_calls), x, y, "#c4a882");
        y += 20.0;
        renderer.draw_text(&format!("Memory: {} KB", self.memory_used / 1024), x, y, "#c4a882");
        y += 30.0;

        renderer.draw_text("System Times:", x, y, "#e8a44e");
        y += 20.0;
        for (name, times) in &self.system_times {
            renderer.draw_text(&format!("  {}: {:.2}ms", name, times.avg()), x, y, "#c4a882");
            y += 16.0;
        }
    }
}
```

---

## Appendix A: Glossary

| Term | Definition |
|------|-----------|
| **Archetype** | Set of component types shared by a group of entities. Entities with identical component sets belong to the same archetype. |
| **Wave** | Group of systems that can execute in parallel without data conflicts. |
| **Dirty Flag** | Boolean marker indicating data has changed and dependent systems need re-computation. |
| **Spatial Hash** | Grid-based spatial index mapping 2D positions to cells for fast proximity queries. |
| **Sprite Batch** | Grouping of sprites sharing the same texture for single-draw-call rendering. |
| **EntityCommandBuffer** | Deferred mutation queue for safe structural changes during system execution. |
| **Generational Index** | Entity reference with ID + generation counter to detect stale/dangling pointers. |
| **Type-Erased Storage** | Component storage using `Vec<u8>` keyed by `TypeId`, enabling heterogeneous collections. |

## Appendix B: File Locations

| Module | Path | Description |
|--------|------|-------------|
| Core Entity | `src/core/entity.rs` | UniversalEntity, EntityId, generation |
| Core World | `src/core/world.rs` | UniversalWorld, Component trait, ComponentStorage |
| Scheduler | `src/engine/scheduler.rs` | ParallelScheduler, wave execution |
| Event Bus | `src/engine/events.rs` | TypedEventBus, typed event dispatch |
| Scene Graph | `src/engine/scene.rs` | SceneGraph, SceneNode, transform propagation |
| Asset Server | `src/engine/assets.rs` | AssetServer, handle management, async loading |
| Input | `src/engine/input.rs` | InputMap, action registry, device abstraction |
| Renderer | `src/engine/renderer.rs` | CanvasRenderer, sprite batching, camera |
| Physics | `src/engine/physics.rs` | SimplePhysicsWorld, AABB, SAT |
| Audio | `src/engine/audio.rs` | AudioManager, channel mixing |
| UI | `src/engine/ui/` | Widget tree, StardewTheme, layout |
| Save/Load | `src/engine/save.rs` | JsonSaveSystem, versioned serialization |
| Debug | `src/engine/debug_overlay.rs` | Profiling overlay, frame stats |
| Game Loop | `src/engine/game_loop.rs` | GameLoop, system wiring |
| Farming | `src/game/farming/` | CropComponent, GrowthStage, FarmingSystem |
| NPC | `src/game/npc/` | NPCEntity, Schedule, DialogueRunner |
| Combat | `src/game/combat/` | CombatSystem, turn-based resolution |
| Inventory | `src/game/inventory/` | InventoryComponent, ItemStack |
| Quest | `src/game/quest/` | QuestManager, quest tracking |
| Economy | `src/game/economy/` | Economy resource, shop system |
| Tilemap | `src/game/tilemap/` | TilemapRenderer, tile data |

## Appendix C: Restoration Checklist

When restoring a game from reference, verify each item:

- [ ] All entity types mapped to Component structs
- [ ] All systems mapped to UniversalSystem impls
- [ ] All events mapped to TypedEventBus events
- [ ] All UI screens mapped to Widget trees with StardewTheme
- [ ] All input actions mapped to InputMap actions
- [ ] All audio mapped to AudioManager channels
- [ ] All save/load mapped to JsonSaveSystem
- [ ] All tile types mapped to tile IDs
- [ ] All items mapped to Item definitions
- [ ] All NPC behaviors mapped to Schedule + DialogueTree
- [ ] All game states mapped to StateMachine states
- [ ] Performance budget verified (16.67ms target)
- [ ] HTML5 prototype validates core loop feel
- [ ] Dirty flag system covers all mutation points
- [ ] Spatial hash covers all proximity queries
- [ ] Sprite batching covers all rendering

---

*Last updated: 2026-09-12 — UNIVERSAL_FUSION.md v3.0*
