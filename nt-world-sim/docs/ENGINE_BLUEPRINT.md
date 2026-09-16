# NT-World-Sim Engine Blueprint

> **Master Reference Document** — The definitive architecture for NT-World-Sim's universal game engine.
> Synthesizes UNIVERSAL_ENGINE.md, CROSS_ENGINE_FUSION.md, and all research into a single authoritative source.

---

## 1. Engine Architecture Overview

### 1.1 Six-Layer Architecture

NT-World-Sim follows the NeoTrix six-layer consciousness-embodiment architecture, adapted for game simulation:

```
┌─────────────────────────────────────────────────────┐
│  L6 Meta-Cognition  │  ConsciousnessTree, SEAL loop │
│  L5 Cognition        │  Decision trees, NPC AI       │
│  L4 Emotion          │  EmotionEngine, mood system   │
│  L3 Embodiment       │  Body schema, sensors, safety │
│  L2 Perception       │  World sense, input, camera   │
│  L1 Action           │  Physics, rendering, audio    │
└─────────────────────────────────────────────────────┘
```

For game engine purposes, the practical layer mapping is:

| Layer | Engine Responsibility | Key Modules |
|-------|----------------------|-------------|
| **L1 Action** | Rendering, physics, audio output, tilemap drawing | `renderer.rs`, `physics.rs`, `audio.rs` |
| **L2 Perception** | Input collection, camera transforms, spatial queries | `input.rs`, `camera.rs`, `scene.rs` |
| **L3 Embodiment** | Entity lifecycle, component storage, world state | `entity.rs`, `world.rs`, `scheduler.rs` |
| **L4 Emotion** | Game feel: screen shake, particles, juice effects | `particle.rs`, `debug_overlay.rs`, `sprite_batch.rs` |
| **L5 Cognition** | Game logic: farming, NPCs, dialogue, crafting | `game/` directory |
| **L6 Meta** | Save/load, profiling, debug overlay, scene stack | `save/`, `debug_overlay.rs` |

### 1.2 Core Abstractions

All core types live in `src/core/`. They are engine-agnostic — no dependency on any specific rendering or physics engine.

#### Entity

```rust
// src/core/entity.rs
pub struct UniversalEntity {
    pub id: EntityId,        // u64 — monotonic, never reused
    pub generation: u32,     // generational index for safety
    pub archetype: ArchetypeId,
}
```

- Entity IDs are **monotonic u64** — never reused, safe to cache externally
- Generation counter detects stale references (use-after-despawn)
- Archetype is metadata — engines that don't use archetypes ignore it

#### Component

```rust
// src/core/world.rs
pub trait Component: Send + Sync + Clone + 'static {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
}
```

- Components are plain data — no methods, no logic
- `Clone + Send + Sync` — thread-safe, copyable
- Type-erased storage via `Box<dyn Any>` with `TypeId` key
- Three storage backends: Dense (Vec), Sparse (HashMap), SoA (Structure of Arrays)

Standard component library:

| Component | Fields | Purpose |
|-----------|--------|---------|
| `Transform` | position: Vec2, rotation: f32, scale: Vec2 | Spatial transform |
| `Sprite` | texture, rect, color, z_index | Visual representation |
| `RigidBody` | position, velocity, mass, body_type | Physics body |
| `Collider` | AABB / Circle / Polygon | Collision shape |
| `Name` | name: String | Debug identification |
| `Parent` | parent: EntityId | Hierarchy link |
| `Velocity` | x: f32, y: f32 | Movement vector |
| `Health` | current: f32, max: f32 | Entity health |
| `Timer` | remaining: f32, repeat: bool | Time-based triggers |

#### System

```rust
// src/core/scheduler.rs
pub trait UniversalSystem: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn read_components(&self) -> Vec<TypeId> { vec![] }
    fn write_components(&self) -> Vec<TypeId> { vec![] }
    fn enabled(&self) -> bool { true }
}
```

System ordering conventions:

| Phase | Systems | Priority |
|-------|---------|----------|
| 0 | Input collection | -100 |
| 1 | Physics step | 0 |
| 2 | Gameplay logic | 10-19 |
| 3 | UI update | 20 |
| 4 | Rendering | 100 |

#### Resource

```rust
// src/core/world.rs
pub trait Resource: Send + Sync + 'static {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
}
```

Standard resources:

| Resource | Purpose |
|----------|---------|
| `DeltaTime` | Frame time (delta + elapsed) |
| `InputState` | Current frame input |
| `CameraResource` | Active camera |
| `AudioManager` | Sound playback |
| `AssetServer` | Asset loading |
| `SceneStack` | Scene management |
| `PhysicsConfig` | Gravity, iterations |

#### Event

```rust
// src/core/world.rs + src/engine/event_bus.rs
pub trait Event: Send + Sync + 'static { }
```

Standard events: `CollisionEvent`, `InputActionEvent`, `SceneTransitionEvent`, `AssetLoadedEvent`, `DamageEvent`, `UiClickEvent`, `EntitySpawnedEvent`, `EntityDespawnedEvent`.

### 1.3 Module Dependency Graph

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
│   ├── asset.rs        ← AssetServer
│   ├── camera.rs       ← Camera2D (extended)
│   ├── sprite_batch.rs ← SpriteBatchExt (texture-atlas batching)
│   ├── particle.rs     ← ParticleSystem, ParticleEmitter
│   └── debug_overlay.rs← DebugRenderer, ProfilerBars
│
├── src/game/           ← Game layer (depends on core + engine)
│   ├── game_loop.rs    ← GameLoop (orchestrates everything)
│   ├── time.rs         ← GameTime, TimeSystem
│   ├── farming.rs      ← FarmPlot, FarmingSystem
│   ├── crops.rs        ← Crop, CropStage
│   ├── npc.rs          ← Npc, NpcRole, Position
│   ├── npcs.rs         ← NpcDefinition, NpcSchedule
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
│   ├── widget.rs       ← Widget, UiLayout, UiStyle, WidgetTrait
│   └── stardew_theme.rs← StardewTheme (pixel-art palette)
│
├── src/adapters/       ← Engine adapters (optional, depends on engine)
│   ├── bevy_adapter.rs
│   ├── godot_adapter.rs
│   └── unity_adapter.rs
│
└── src/save/           ← Persistence (depends on core)
    └── save_system.rs
```

---

## 2. Universal Engine Pattern Library

This section shows how every core game engine pattern maps across five engines. NT-World-Sim's goal is to be the **restoration target** — given a game built in any engine, we can re-implement it using our abstractions.

### 2.1 Entity-Component Pattern

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Entity** | `GameObject` | `AActor` | `Node` | `Entity` (u64 packed) | `UniversalEntity` (u64 monotonic + u32 gen) |
| **Component** | `MonoBehaviour` | `UActorComponent` | Node properties | `Component` trait | `Component` trait (plain data) |
| **Storage** | Inline on GO | Component array | Node properties | Archetype chunks (16KB) | Dense/Sparse/SoA enum |
| **Add** | `AddComponent<T>()` | `CreateDefaultSubobject<T>()` | `Node.property = val` | `commands.spawn().insert(T)` | `world.add_component(e, T)` |
| **Get** | `GetComponent<T>()` | `FindComponentByClass<T>()` | `Node.T` | `query.get(e)` | `world.get_component::<T>(e)` |
| **Remove** | `Destroy(component)` | `RemoveSubObject()` | `Node.property = null` | `commands.entity(e).remove::<T>()` | `world.remove_component::<T>(e)` |
| **Query** | `FindObjectsOfType<T>()` | `TActorIterator<T>` | `get_tree().get_nodes_in_group()` | `Query<&T>` | `world.query::<T>()` |
| **Generational?** | No (handle recycled) | No (pointer) | No (RID) | Yes (packed u64) | Yes (u64 id + u32 gen) |

### 2.2 System Tick / Game Loop

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Per-frame** | `MonoBehaviour.Update()` | `AActor::Tick()` | `Node._process(dt)` | `fn system(Res<W>)` | `UniversalSystem::update(world, dt)` |
| **Fixed step** | `FixedUpdate()` | `Tick()` (fixed) | `Node._physics_process(dt)` | `FixedUpdate` set | `SimplePhysicsWorld::step(dt)` |
| **Scheduler** | Execution order | `FTaskGraph` | Call order | `Schedule` (ECS) | `ParallelScheduler` (priority waves) |
| **Dependency** | `[UpdateBefore/After]` | `TaskGraph` edges | Manual ordering | `before()/in_set()` | `read_components()` / `write_components()` |
| **Parallelism** | Manual jobs | Task graph | None native | Automatic (ECS) | Wave-based (no intra-system parallel) |
| **Startup** | `Awake()` + `Start()` | `BeginPlay()` | `_ready()` | `Startup` systems | `GameLoop::start_game()` |

### 2.3 Event System

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Declaration** | `UnityEvent<T>` field | `DECLARE_DELEGATE()` macro | `signal name(args)` | `Event<T>` struct | `Event` trait |
| **Dispatch** | `Invoke(args)` | `Broadcast(args)` | `emit("name", args)` | `EventWriter<T>::send(e)` | `world.send_event(e)` |
| **Handler** | `AddListener(method)` | `BindDynamic()` | `connect("name", obj, "method")` | `EventReader<T>` | `world.receive_events::<T>()` |
| **Scope** | Per-component | Per-delegate | Per-node (signal) | Per-type (global) | Global `TypedEventBus` |
| **Type safety** | Yes (generic) | Partial (dynamic) | Runtime (string) | Yes (generic) | Type-erased (TypeId) |
| **Queued?** | Frame-same | Frame-same | Frame-same | Accumulated | Accumulated + flush |

### 2.4 Scene Management

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Scene** | `.unity` file | `.umap` level | `.tscn` file | `State` | `SceneGraph` + `SceneStack` |
| **Hierarchy** | Transform parent/child | `USceneComponent` tree | `Node` tree | `Parent`/`Children` | `SceneNode` with parent/children |
| **Transition** | `SceneManager.LoadScene()` | `UGameplayStatics::OpenLevel()` | `SceneTree.change_scene()` | State change | `SceneStack::push/pop/replace` |
| **Effects** | Transition asset | `UMG` loading screen | `CanvasLayer` | Custom | `TransitionType` enum (Fade/Slide/Iris) |
| **Data passing** | Static `DontDestroyOnLoad` | `UGameInstance` | `Autoload` | `States` + resources | `SceneData::Map(HashMap)` |
| **Instantiation** | `Instantiate(prefab)` | `SpawnActor()` | `instantiate(scene)` | `commands.spawn()` | `SceneGraph::add_root()` |

### 2.5 Asset Loading

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **System** | `AssetDatabase` | `FStreamableManager` | `ResourceLoader` | `AssetServer` | `AssetServer` (planned) |
| **Handle** | `T` (reference) | `TSoftObjectPtr<T>` | `Resource` | `Handle<T>` | `AssetHandle<T>` |
| **Async** | `Addressables.LoadAsync()` | `AsyncLoad()` | `load_threaded_request()` | `server.load()` (async) | `server.load()` (callback) |
| **Placeholder** | Sprite atlas fallback | `TSoftObjectPtr` null | Missing texture | Fallback assets | Colored rectangles |
| **Caching** | Automatic | Automatic | Automatic | Manual | `HashMap<String, AssetEntry>` |
| **Atlas** | `SpriteAtlas` | `UPaperSprite` atlas | `TileSet` atlas | `TextureAtlas` | `SpriteBatchExt` |

### 2.6 Input

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Raw** | `Input.GetKey()` | `IsInputKeyDown()` | `Input.is_action_pressed()` | `Input<KeyCode>` | `InputState::is_key_down()` |
| **Action map** | `InputAction` asset | `UInputAction` | `InputMap` singleton | `InputMap` | `InputMap` with `InputBinding` |
| **Events** | `InputSystem.onPress` | `EnhancedInput` events | `InputEvent` subclass | `InputEvent<T>` | `InputEvent` enum |
| **Gamepad** | `Gamepad` class | `FInputGamepad` | `Input.get_joy_axis()` | `GamepadInput` | `GamepadAxis` / `GamepadButton` |
| **Touch** | `Input.touchCount` | — | `InputEventScreenTouch` | — | `TouchPressed/Moved/Released` |
| **Mouse lock** | `Cursor.lockState` | `SetShowMouseCursor()` | `Input.set_mouse_mode()` | `CursorGrabMode` | (planned) |

### 2.7 UI

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **System** | UGUI / UI Toolkit | UMG | `Control` nodes | egui / `bevy_ui` | `WidgetTrait` + `WidgetNode` |
| **Layout** | RectTransform + anchors | UMG canvas | Anchors + containers | Flexbox-like | `UiLayout` enum (Fixed/Anchor/Flex/Grid/Stack) |
| **Styling** | UI Toolkit USS | UMG style set | Theme resource | (manual) | `UiStyle` tokens |
| **Events** | `Button.onClick` | `OnClicked` delegate | `pressed` signal | Interaction system | `UiEvent` enum |
| **Text** | `TextMeshProUGUI` | `UTextBlock` | `Label` | `TextBundle` | `Label` widget |
| **Scrolling** | `ScrollRect` | `SScrollBox` | `ScrollContainer` | `ScrollArea` | `ScrollArea` widget |
| **Input** | `InputField` | `SEditableText` | `LineEdit` / `TextEdit` | `TextInteraction` | `TextInput` widget |

### 2.8 Audio

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Source** | `AudioSource` | `UAudioComponent` | `AudioStreamPlayer` | `AudioBundle` | `AudioSource` + `AudioBackend` |
| **Manager** | `AudioManager` | `FAudioDevice` | `AudioServer` | `AudioPlugin` | `AudioManager` |
| **Bus/Mixer** | `AudioMixer` | `USoundMix` | `AudioBus` | `AudioChannel` | `AudioBus` (hierarchy) |
| **Spatial** | `spatialBlend` | `USpatializationVolume` | `AudioStreamPlayer2D` | `SpatialScale` | `SpatialAudioSource` |
| **SFX** | `PlayOneShot()` | `PlaySoundAtLocation()` | `AudioStreamPlayer.play()` | `audio.play()` | `AudioManager::play_sfx()` |
| **Music** | Loop + crossfade | Crossfade via blend | `AudioStreamPlayer` loop | Channel-based | `AudioBus` BGM channel |

### 2.9 Physics

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **Engine** | Box2D / PhysX | ChaosPhysics | GodotPhysics2D | Rapier | `SimplePhysicsWorld` |
| **Body** | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` | `RigidBody` | `RigidBody` component |
| **Collider** | `Collider2D` | `UShapeComponent` | `CollisionShape2D` | `Collider` | `Collider` enum (AABB/Circle/Polygon) |
| **Raycast** | `Physics2D.Raycast()` | `LineTraceSingle()` | `Physics2D.intersect_ray()` | `RayCaster` | `PhysicsWorld::raycast()` |
| **Joint** | `DistanceJoint2D` | `UPhysicsConstraint` | `Joint2D` | `RevoluteJoint` | `Joint` enum |
| **Layers** | Layer + mask | Object channel | Layer + mask | Collision layers | `collision_layer` / `collision_mask` u32 |
| **Sleep** | Auto | Auto | Auto | Auto | `is_sleeping` flag |

### 2.10 Rendering

| Dimension | Unity | Unreal | Godot | Bevy | NT-World-Sim |
|-----------|-------|--------|-------|------|-------------|
| **2D** | `SpriteRenderer` | `UPaperSprite` | `Sprite2D` | `SpriteBundle` | `Renderer::draw_sprite()` |
| **Tilemap** | `Tilemap` | `TileBase` | `TileMap` | `Tilemap` | `TileMap` + chunked render |
| **Text** | `TextMeshPro` | `UTextRender` | `Label` | `TextBundle` | `Renderer::draw_text()` |
| **Shapes** | `LineRenderer` | `DrawDebug` | `draw_*()` | `DebugLines` | `Renderer::draw_rect/circle/line()` |
| **Camera** | `Camera` | `UCameraComponent` | `Camera2D` | `Camera2D` | `Camera2D` with follow/shake |
| **Particles** | `ParticleSystem` | `UNiagaraSystem` | `GPUParticles2D` | `ParticleSystem` | `ParticleSystem` + `ParticleEmitter` |
| **Batching** | SRP batcher | Batched draw calls | Automatic | Automatic | `SpriteBatchExt` (texture-keyed) |
| **Backend** | Graphics API | RHI / Vulkan | Vulkan / OpenGL | `RenderPipeline` | `CanvasRenderer` (WASM) / `WgpuRenderer` |

---

## 3. Quick Game Restoration Guide

Step-by-step process to restore any game from a reference using NT-World-Sim.

### Step 1: Analyze Reference Game Systems

Decompose the reference game into discrete systems. Example for Stardew Valley:

```
Stardew Valley Systems:
├── Time system (day/night cycle, seasons, clock)
├── Farming (till, plant, water, harvest, scarecrows)
├── NPCs (schedule, dialogue, gifts, marriage, events)
├── Inventory (items, tools, seeds, crops, stacking)
├── Crafting (recipes, workbench, materials)
├── Mining (floors, rocks, ore, gems, ladder)
├── Combat (monsters, weapons, health, dodge)
├── Economy (shop, shipping bin, gold, prices)
├── Skills (farming/mining/foraging/fishing levels)
├── Weather (rain, storms, effects on crops)
├── Buildings (upgrades, barn, coop, silo)
├── Fishing (cast, timing, minigame, quality)
└── Events (festivals, cutscenes, mail)
```

### Step 2: Map to NT-World-Sim Abstractions

For each system, find the NT-World-Sim equivalent or determine if a new module is needed:

| Game System | NT-World-Sim Abstraction | Module | Status |
|-------------|-------------------------|--------|--------|
| Time | `GameTime` resource + `TimeSystem` | `game/time.rs` | Implemented |
| Farming | `FarmPlot` + `Crop` + `FarmingSystem` | `game/farming.rs` | Implemented |
| NPCs | `Npc` + `NpcSchedule` + `DialogueTree` | `game/npc.rs` + `dialogue.rs` | Implemented |
| Inventory | `Inventory` + `Item` + `ItemStack` | `game/inventory.rs` | Implemented |
| Crafting | `CraftingSystem` + `Recipe` | `game/crafting.rs` | Implemented |
| Seasons | `Season` enum + `SeasonEffects` | `game/season.rs` | Implemented |
| Weather | `Weather` enum + weather system | `game/weather.rs` | Implemented |
| Physics | `SimplePhysicsWorld` + `Collider` | `engine/physics.rs` | Implemented |
| Rendering | `TileMap` + `Sprite` + `Camera` | `engine/renderer.rs` | Implemented |
| Input | `InputState` + `InputMap` | `engine/input.rs` | Implemented |
| Audio | `AudioManager` + `AudioBackend` | `engine/audio.rs` | Implemented |
| UI | `Widget tree` + `UiLayout` | `ui/widget.rs` | Implemented |
| Pathfinding | A* on tile grid | `world/pathfinding.rs` | Implemented |
| Skills | `SkillTree` + `SkillSystem` | `game/skills.rs` | **Gap** |
| Combat | `CombatSystem` + `Health` | `game/combat.rs` | **Gap** |
| Economy | `ShopInventory` + `Economy` | `game/shop.rs` | **Gap** |
| Mine | `MineFloor` + `MineSystem` | `game/mine.rs` | **Gap** |
| Fishing | `FishingRod` + `FishingSystem` | `game/fishing.rs` | **Gap** |

### Step 3: Implement Missing Systems

For each gap, create minimal implementations following the NT-World-Sim pattern:

```rust
// Example: Adding fishing
// 1. Define components
#[derive(Component, Clone)]
pub struct FishingSpot { pub quality: u32 }

#[derive(Component, Clone)]
pub struct FishingState { pub casting: bool, pub timer: f32, pub catch: Option<ItemId> }

// 2. Define system
pub struct FishingSystem;
impl UniversalSystem for FishingSystem {
    fn name(&self) -> &str { "FishingSystem" }
    fn priority(&self) -> i32 { 15 }

    fn read_components(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Transform>(), TypeId::of::<FishingSpot>()]
    }

    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        // Query FishingSpot + Player nearby
        // Start minigame on interact
        // Timer-based catch resolution
        // Emit ItemAcquiredEvent on success
    }
}

// 3. Register in game loop
game_loop.add_system(Box::new(FishingSystem), 15);
```

### Step 4: Connect via GameLoop

Wire all systems into the game loop with correct priority ordering:

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
    loop.add_system(Box::new(FishingSystem), 15);
    loop.add_system(Box::new(MiningSystem), 16);
    loop.add_system(Box::new(CombatSystem), 17);

    // Phase 3: UI (priority 20)
    loop.add_system(Box::new(UiSystem), 20);

    // Phase 4: Rendering (priority 100)
    loop.add_system(Box::new(RenderSystem), 100);

    loop
}
```

### Step 5: Build UI with StardewTheme

Use the UI framework for all game screens:

```rust
fn build_dialogue_ui(npc: &Npc, tree: &DialogueTree) -> WidgetNode {
    WidgetNode::panel(StardewTheme::dialogue_box())
        .child(WidgetNode::portrait(&npc.name))
        .child(WidgetNode::text(&tree.current_text()))
        .child(WidgetNode::choices(&tree.choices()))
}

fn build_inventory_ui(inventory: &Inventory) -> WidgetNode {
    WidgetNode::panel(StardewTheme::inventory_panel())
        .child(WidgetNode::grid(8, 4, |slot| {
            WidgetNode::item_slot(slot.item.as_ref(), slot.count)
        }))
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

## 4. Stardew Valley → Consciousness Valley Complete Mapping

Every system from Stardew Valley mapped to NT-World-Sim with implementation details.

### 4.1 Implemented Systems

| Stardew System | NT-World-Sim Module | Key Types | Status |
|----------------|-------------------|-----------|--------|
| **Daily loop** (6AM→2AM) | `game/time.rs` | `GameTime { hour, minute, day, season }`, `TimeSystem` | Implemented |
| **Farming** (till→plant→water→harvest) | `game/farming.rs` + `crops.rs` | `FarmPlot`, `Crop`, `CropStage`, `FarmingSystem` | Implemented |
| **NPCs** (schedule, dialogue, gifts) | `game/npc.rs` + `npcs.rs` + `dialogue.rs` | `Npc`, `NpcDefinition`, `DialogueTree` | Implemented |
| **Crafting** (recipe + ingredients) | `game/crafting.rs` + `recipes.rs` | `CraftingSystem`, `Recipe` | Implemented |
| **Inventory** (items, stacking) | `game/inventory.rs` + `item.rs` | `Inventory`, `Item`, `ItemStack` | Implemented |
| **Seasons** (spring→summer→fall→winter) | `game/season.rs` | `Season` enum + `SeasonEffects` | Implemented |
| **Weather** (sunny, rainy, stormy) | `game/weather.rs` | `Weather` enum + weather system | Implemented |
| **Energy** (stamina, sleep recovery) | `game/energy.rs` | `Energy` struct | Implemented |

### 4.2 Partially Implemented Systems

| Stardew System | NT-World-Sim Module | Gap | Fix |
|----------------|-------------------|-----|-----|
| **World generation** (farm, town, mine) | `world/generator.rs` | Generates tiles but no POI placement | Add `place_buildings()` after tile generation |
| **Pathfinding** (NPC movement) | `world/pathfinding.rs` | A* exists but not wired to NPC schedule | Add `NpcMovementSystem` that calls `find_path()` |
| **Tilemap rendering** | `engine/renderer.rs` | Renders but no chunk-based culling | Add `chunked_render(camera, chunk_size)` |
| **Camera** | `engine/camera.rs` | Basic position/zoom, no follow | Add `Camera::follow(target, lerp, dt)` |

### 4.3 Planned Systems (Not Yet Implemented)

| Stardew System | Target Module | Implementation Path | Priority |
|----------------|--------------|---------------------|----------|
| **Skills** (farming, mining, fishing, foraging) | `game/skills.rs` | `SkillTree` component + `SkillSystem` resource. XP on action, level up, unlock recipes | P2 |
| **Combat** (melee, ranged, monsters) | `game/combat.rs` | `CombatSystem` + `Health` component + `DamageEvent`. Simple turn-based or real-time | P2 |
| **Shop/Economy** (buy, sell, prices) | `game/shop.rs` | `ShopInventory` + `Economy` resource + `ShopSystem`. Gold as currency | P2 |
| **Relationships** (friendship points, dating) | Extend `game/npc.rs` | Add `friendship: u32` to `Npc`, `GiftSystem`, `RelationshipEvent` | P2 |
| **Mine** (floors, rocks, ore, gems) | `game/mine.rs` | `MineFloor` + `MineSystem` + ore generation per floor | P3 |
| **Fishing** (timing, quality) | `game/fishing.rs` | `FishingRod` + `FishingSystem` + minigame state machine | P3 |
| **Building upgrades** (house, barn, coop) | `game/building.rs` | `Building` component + `UpgradeSystem` + resource costs | P3 |
| **Mail/Quests** | `game/quests.rs` | `Quest` + `QuestSystem` + mailbox UI | P3 |

### 4.4 Consciousness Valley Unique Systems

These are NT-World-Sim additions that go beyond Stardew Valley:

| System | Module | Purpose |
|--------|--------|---------|
| **Resonance** | `game/npc.rs` — `resonance` field | NPC relationship uses "resonance" instead of friendship points — thematic for consciousness simulation |
| **Awareness** | `game/npcs.rs` — `NpcType::Awareness` | NPCs represent cognitive faculties (Awareness, Focus, Creativity, Empathy) — not just villagers |
| **Meta-cognition events** | `game/dialogue.rs` | Dialogue trees can trigger self-reflection mechanics |
| **Universal engine adapters** | `adapters/` | Same game can run on Bevy, Godot, or web — cross-platform by design |

---

## 5. Cross-Engine Adapter Pattern

How to adapt any external engine's output to NT-World-Sim's abstractions.

### 5.1 Adapter Architecture

```
External Engine (Unity/Unreal/Godot/Bevy)
    │
    ▼
┌─────────────────────┐
│   Export Layer       │  Extract game state to JSON/binary
│   (Data Dumper)      │  Entities, components, scenes, assets
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   Import Adapter     │  Parse external format
│   (Format Reader)    │  Map to NT-World-Sim types
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   NT-World-Sim ECS   │  UniversalEntity + Components
│   (Runtime)          │  UniversalSystem::update()
└─────────────────────┘
```

### 5.2 Entity Mapping

Each engine's entity type maps to `UniversalEntity`:

```rust
// Unity adapter
fn import_unity_entity(go: &GameObject) -> UniversalEntity {
    UniversalEntity {
        id: EntityId::new(go.instance_id as u64),
        generation: 0, // Unity doesn't use generations
        archetype: ArchetypeId::default(),
    }
}

// Bevy adapter
fn import_bevy_entity(e: bevy::ecs::entity::Entity) -> UniversalEntity {
    UniversalEntity {
        id: EntityId::new(e.to_bits()),
        generation: (e.to_bits() >> 32) as u32,
        archetype: ArchetypeId::default(),
    }
}
```

### 5.3 Component Mapping

External engine components are converted to NT-World-Sim components:

```rust
// Unity Transform → NT Transform
fn import_unity_transform(t: &UnityTransform) -> Transform {
    Transform {
        position: Vec2::new(t.position.x, t.position.y),
        rotation: t.rotation.eulerAngles.z,
        scale: Vec2::new(t.localScale.x, t.localScale.y),
    }
}

// Bevy Sprite → NT Sprite
fn import_bevy_sprite(s: &SpriteBundle) -> Sprite {
    Sprite {
        texture: Some(s.texture.path().to_string()),
        rect: Rect::new(0.0, 0.0, s.custom_size.unwrap_or(Vec2::ONE)),
        color: s.color,
        z_index: 0,
    }
}
```

### 5.4 Scene Graph Import

```rust
// Import Unity scene hierarchy
fn import_unity_scene(objects: &[GameObject]) -> SceneGraph {
    let mut graph = SceneGraph::new();
    let root = graph.add_root("ImportedScene");

    for go in objects {
        let entity = import_unity_entity(go);
        let node = SceneNode {
            entity,
            parent: go.parent.map(|p| EntityId::new(p.instance_id as u64)),
            children: vec![],
            local_transform: import_unity_transform(&go.transform),
            world_transform: Transform::identity(),
            visible: go.active_self,
            z_index: 0,
            tag: go.tag.clone(),
        };
        graph.add_node(node);
    }

    graph.update_transforms();
    graph
}
```

### 5.5 System Scheduling Import

External engine execution order is mapped to NT-World-Sim priority:

```rust
// Map Unity MonoBehaviour execution order
fn import_unity_system_order(mono_behaviours: &[String]) -> Vec<(Box<dyn UniversalSystem>, i32)> {
    let mut systems = Vec::new();
    for (i, name) in mono_behaviours.iter().enumerate() {
        let priority = -100 + (i as i32 * 10); // Spread across -100..100
        systems.push((create_system(name), priority));
    }
    systems
}
```

### 5.6 Supported Export Formats

| Engine | Export Method | Format | Data |
|--------|--------------|--------|------|
| **Unity** | Custom editor script | JSON | GameObjects + Components + Scenes |
| **Unreal** | Blueprint export | JSON | Actors + Components + Levels |
| **Godot** | Scene serialization | `.tscn` / JSON | Nodes + Resources |
| **Bevy** | ECS snapshot | JSON / RON | Entities + Components + Resources |
| **Generic** | Tiled / JSON | TMX / JSON | Tilemaps + objects |

---

## 6. Performance Patterns

### 6.1 ECS Archetype Layout

NT-World-Sim uses archetype-based storage for entities sharing the same component set:

```
Archetype 0: [Transform, Sprite]           → 1,200 entities (tilemap tiles)
Archetype 1: [Transform, Sprite, RigidBody]→ 45 entities (physics objects)
Archetype 2: [Transform, Sprite, Npc]      → 12 entities (NPCs)
Archetype 3: [Transform, RigidBody]        → 3 entities (projectiles)
```

**Benefits**:
- Components of same type are contiguous in memory → cache-friendly iteration
- Querying `[Transform, Sprite]` skips unrelated archetypes
- Adding/removing components moves entity between archetypes (structural change)

**Implementation** (`src/core/entity.rs`):

```rust
pub struct Archetype {
    pub id: ArchetypeId,
    pub entities: Vec<EntityId>,
    pub component_types: Vec<TypeId>,
    pub storage: HashMap<TypeId, ComponentStorage>,
}

pub enum ComponentStorage {
    Dense(Vec<Box<dyn Any>>),      // Fast iteration, slow insert
    Sparse(HashMap<EntityId, Box<dyn Any>>),  // Fast insert, slower iteration
    SoA(Vec<Vec<u8>>),            // Structure of Arrays, best for SIMD
}
```

**When to use which**:

| Storage | Best For | Tradeoff |
|---------|----------|----------|
| Dense | Hot components (Transform, Sprite) — iterated every frame | Insert/remove O(n) |
| Sparse | Cold components (Name, QuestState) — rarely iterated | Iteration slower |
| SoA | Performance-critical hot loops — SIMD-friendly | Memory overhead |

### 6.2 System Scheduling with Wave Parallelism

Systems are grouped into waves based on their read/write dependencies:

```
Wave 0 (priority -100):  [InputSystem]
Wave 1 (priority 0):     [PhysicsSystem]
Wave 2 (priority 10):    [TimeSystem, FarmingSystem, NpcScheduleSystem]
Wave 3 (priority 11):    [NpcMovementSystem, DialogueSystem]
Wave 4 (priority 20):    [UiSystem]
Wave 5 (priority 100):   [RenderSystem]
```

**Dependency resolution** via `read_components()` / `write_components()`:

```rust
impl ParallelScheduler {
    fn build_waves(&self, systems: &[Box<dyn UniversalSystem>]) -> Vec<Vec<usize>> {
        let mut waves: Vec<Vec<usize>> = Vec::new();
        let mut written: HashSet<TypeId> = HashSet::new();

        // Sort by priority
        let mut sorted: Vec<(usize, i32)> = systems.iter()
            .enumerate()
            .map(|(i, s)| (i, s.priority()))
            .collect();
        sorted.sort_by_key(|(_, p)| *p);

        let mut current_wave: Vec<usize> = vec![];
        for (idx, _) in sorted {
            let sys = &systems[idx];
            let reads = sys.read_components();
            let writes = sys.write_components();

            // Check if this system conflicts with current wave
            let conflicts = reads.iter().any(|r| written.contains(r))
                || writes.iter().any(|w| written.contains(w));

            if conflicts {
                waves.push(std::mem::take(&mut current_wave));
                written.clear();
            }

            current_wave.push(idx);
            writes.iter().for_each(|w| { written.insert(*w); });
        }

        if !current_wave.is_empty() {
            waves.push(current_wave);
        }

        waves
    }
}
```

### 6.3 Dirty Flag Propagation

Avoid redundant computation by tracking which components changed:

```rust
pub struct ChangeTracker {
    dirty: HashSet<(EntityId, TypeId)>,
    frame_number: u64,
}

impl ChangeTracker {
    pub fn mark_dirty(&mut self, entity: EntityId, component: TypeId) {
        self.dirty.insert((entity, component));
    }

    pub fn is_dirty(&self, entity: EntityId, component: TypeId) -> bool {
        self.dirty.contains(&(entity, component))
    }

    pub fn clear(&mut self) {
        self.dirty.clear();
        self.frame_number += 1;
    }
}
```

**Propagation rules**:

| Trigger | Dirty Flags Set |
|---------|----------------|
| `world.add_component(e, T)` | `(e, T)` |
| `world.remove_component(e, T)` | `(e, T)` |
| `Transform` changed | `(e, Transform)`, `(e, Sprite)` — sprite needs re-render |
| `Sprite.texture` changed | `(e, Sprite)` |
| `RigidBody.position` changed | `(e, RigidBody)`, `(e, Transform)` — sync position |
| Parent changed | `(e, Transform)` + all children `(child, Transform)` |

### 6.4 Spatial Hashing for Physics

Broad-phase collision detection using spatial hashing:

```rust
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<PhysicsEntity>>,
}

impl SpatialHash {
    pub fn insert(&mut self, entity: PhysicsEntity, aabb: &Rect) {
        let min_cell = self.cell_coords(aabb.min);
        let max_cell = self.cell_coords(aabb.max);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                self.cells.entry((x, y)).or_default().push(entity);
            }
        }
    }

    pub fn query(&self, aabb: &Rect) -> Vec<PhysicsEntity> {
        let min_cell = self.cell_coords(aabb.min);
        let max_cell = self.cell_coords(aabb.max);
        let mut results = Vec::new();

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(x, y)) {
                    results.extend(entities);
                }
            }
        }

        results.sort_unstable();
        results.dedup();
        results
    }

    fn cell_coords(&self, pos: Vec2) -> (i32, i32) {
        ((pos.x / self.cell_size).floor() as i32,
         (pos.y / self.cell_size).floor() as i32)
    }
}
```

**Performance characteristics**:

| Scenario | Brute Force O(n²) | Spatial Hash O(n·k) | Improvement |
|----------|-------------------|---------------------|-------------|
| 100 entities, uniform | 10,000 checks | ~400 checks (k≈4) | 25× |
| 1,000 entities, uniform | 1,000,000 checks | ~4,000 checks | 250× |
| 10,000 entities, clustered | 100,000,000 checks | ~40,000 checks | 2,500× |

### 6.5 Sprite Batching

Batch sprites by texture to minimize draw calls:

```rust
// src/engine/sprite_batch.rs
pub struct SpriteBatchExt {
    batches: HashMap<String, Vec<BatchEntry>>,
    max_batch_size: usize,  // Default: 1024
}

impl SpriteBatchExt {
    pub fn add(&mut self, texture: &str, entry: BatchEntry) {
        self.batches
            .entry(texture.to_string())
            .or_default()
            .push(entry);
    }

    pub fn to_draw_commands(&self) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        for (texture, entries) in &self.batches {
            let mut sorted = entries.clone();
            sorted.sort_by_key(|e| e.z_order);
            for entry in sorted {
                cmds.push(DrawCommand::DrawSprite {
                    texture: texture.clone(),
                    dest: Rect::new(entry.x, entry.y, entry.width, entry.height),
                    color: entry.color,
                    z_index: entry.z_order,
                });
            }
        }
        cmds
    }
}
```

**Batching strategy**:

1. Sort sprites by texture key (group same texture together)
2. Within same texture, sort by z_index (painter's algorithm)
3. Emit one `DrawCommand` per batch (minimizes state changes)
4. Split batches exceeding `max_batch_size` (GPU limit)

**Expected draw call reduction**:

| Scenario | Without Batch | With Batch | Reduction |
|----------|--------------|------------|-----------|
| 1,000 sprites, 20 textures | 1,000 draw calls | 20 draw calls | 50× |
| 5,000 sprites, 50 textures | 5,000 draw calls | 50 draw calls | 100× |
| Tilemap (400 tiles, 4 textures) | 400 draw calls | 4 draw calls | 100× |

### 6.6 Memory Layout Guidelines

| Component | Storage | Rationale |
|-----------|---------|-----------|
| `Transform` | Dense (SoA preferred) | Iterated every frame by rendering, physics, and gameplay systems |
| `Sprite` | Dense | Iterated every frame by rendering system |
| `RigidBody` | Dense | Iterated every frame by physics system |
| `Collider` | Dense | Iterated by physics broad-phase |
| `Velocity` | Dense | Updated every frame by physics |
| `Name` | Sparse | Rarely iterated, used for debug only |
| `Health` | Sparse | Only iterated by combat/damage systems |
| `Timer` | Sparse | Only iterated by time-dependent systems |
| `Npc` | Sparse | Only iterated by NPC AI systems |
| `Inventory` | Sparse | Only iterated by UI and crafting |

---

## Appendix A: Build & Test Commands

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

## Appendix B: Key File Locations

| File | Purpose |
|------|---------|
| `src/core/entity.rs` | EntityId, UniversalEntity, Archetype, Chunk |
| `src/core/world.rs` | UniversalWorld, Component/Resource/Event traits |
| `src/core/scheduler.rs` | UniversalSystem trait, ParallelScheduler |
| `src/engine/renderer.rs` | Renderer trait, Camera, Sprite, TileMap, DrawCommand |
| `src/engine/physics.rs` | SimplePhysicsWorld, RigidBody, Collider |
| `src/engine/input.rs` | InputState, KeyCode, InputProvider |
| `src/engine/scene.rs` | SceneGraph, SceneNode |
| `src/engine/event_bus.rs` | TypedEventBus |
| `src/engine/audio.rs` | AudioManager, AudioBackend |
| `src/engine/camera.rs` | Camera (extended) |
| `src/engine/sprite_batch.rs` | SpriteBatchExt (texture-keyed batching) |
| `src/engine/particle.rs` | ParticleSystem, ParticleEmitter |
| `src/engine/debug_overlay.rs` | DebugRenderer, ProfilerBars |
| `src/game/game_loop.rs` | GameLoop orchestrator |
| `src/game/farming.rs` | FarmPlot, FarmingSystem |
| `src/game/npc.rs` | Npc, NpcRole, Position |
| `src/game/npcs.rs` | NpcDefinition, NpcType, schedules |
| `src/game/dialogue.rs` | DialogueTree, DialogueNode |
| `src/game/time.rs` | GameTime, TimeSystem |
| `src/world/generator.rs` | WorldGenerator, TileType |
| `src/world/pathfinding.rs` | AStar pathfinding |
| `src/ui/widget.rs` | Widget, UiLayout, UiStyle |
