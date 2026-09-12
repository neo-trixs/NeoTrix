# Game Engine Architecture Technical Reference

Comprehensive mapping of Unity DOTS/ECS, Unreal Engine, Godot, and Bevy patterns
to universal abstractions. Generated 2026-09-12.

---

## 1. Unity DOTS/ECS

### 1.1 Entity-Component-System Architecture

Unity DOTS implements a strict ECS separation:

- **Entity**: A lightweight integer handle (32-bit) representing identity. No data, no behavior.
- **Component**: A plain data struct (blittable types only). No methods, no inheritance.
- **System**: Stateless logic that queries entities by component composition.

```
Entity = { Index: u32, Version: u32 }
Component = struct { field: T }  // no behavior
System = fn(Query<(&Pos, &mut Vel)>)  // stateless function
```

**Key distinction**: Systems never store per-frame state. All state lives in components or managed resources.

### 1.2 Chunk-Based Memory Layout

Unity ECS uses **archetype-based storage**:

- Entities with identical component types share an **archetype**.
- Each archetype stores entities in **chunks** (16 KiB blocks).
- Each chunk contains a dense array per component type.
- Changing an entity's components (add/remove) moves it to a different archetype.

```
Archetype M: [EntityA, EntityB] → chunks: [Pos[], Vel[], Health[]]
Archetype N: [EntityC]          → chunks: [Pos[], Vel[]]

Chunk layout (16 KiB):
┌─────────────┬─────────────┬─────────────┐
│ Entity[]    │ Pos[]       │ Vel[]       │
│ (ids)       │ (SoA dense) │ (SoA dense) │
└─────────────┴─────────────┴─────────────┘
```

**Performance implication**: Iterating over a chunk gives perfect cache locality for each component array. Systems processing many entities with the same archetype get L1/L2 cache hits.

### 1.3 Job System and Burst Compiler

- **C# Job System**: Safe multithreaded execution. Jobs declare read/write dependencies; the scheduler runs non-conflicting jobs in parallel.
- **Burst Compiler**: LLVM-based compiler that translates IL/C# to optimized native SIMD code. Requires unmanaged (blittable) types.

```
[BurstCompile]
struct MoveJob : IJobChunk
{
    public ComponentTypeHandle<Position> PositionType;
    public ComponentTypeHandle<Velocity> VelocityType;
    public float DeltaTime;

    public void Execute(ArchetypeChunk chunk, int chunkIndex, ...) { ... }
}
```

**Relationship**: Jobs operate on chunks; Burst compiles job code to native. Together they give near-C performance from C#.

### 1.4 Unity Collections (NativeArray, NativeHashMap)

All DOTS data structures live in **unmanaged memory** outside the GC heap:

| Type | Purpose | GC Pressure |
|------|---------|-------------|
| `NativeArray<T>` | Fixed-size contiguous buffer | Zero |
| `NativeList<T>` | Resizable unmanaged list | Zero |
| `NativeHashMap<K,V>` | Hash map in unmanaged memory | Zero |
| `NativeQueue<T>` | FIFO queue | Zero |

- Must be explicitly `Dispose()`d (or chained via `Dispose(JobHandle)`).
- Allocator determines lifetime: `Temp` (frame), `TempJob` (job chain), `Persistent`.
- Safe to pass to Burst-compiled jobs.

### 1.5 SystemBase vs ISystem

| Feature | SystemBase | ISystem |
|---------|-----------|---------|
| Type | Class (reference) | Struct (value) |
| Managed state | Yes (fields allowed) | No (unmanaged only) |
| Burst compatible | No | Yes |
| Performance | Slower (virtual dispatch) | Faster (static dispatch) |
| Query syntax | `GetEntityQuery()` | `SystemAPI.Query()` |
| Lifecycle | `OnCreate/OnUpdate/OnDestroy` | Same, via `ref` self |

**Recommendation**: Use `ISystem` by default; use `SystemBase` only when managed state is unavoidable.

### 1.6 Component Lookup Patterns

```csharp
// ISystem (preferred)
ref var pos = ref SystemAPI.GetRW<Position>(entity);
ref readonly var vel = ref SystemAPI.GetRO<Velocity>(entity);

// SystemBase
var lookup = GetComponentDataFromEntity<Position>(isReadOnly: false);
lookup[entity] = newPos;

// EntityQuery (batch)
var query = new EntityQueryDesc { All = new[] { typeof(Position), typeof(Velocity) } };
```

**Key pattern**: `ComponentDataFromEntity<T>` (now `ComponentLookup<T>`) provides random-access O(1) lookup by entity, usable from jobs.

### 1.7 Scene Management

- **SubScenes**: ECS worlds loaded as nested scenes. Only entities inside SubScenes exist in ECS.
- **Convert-to-ECS**: GameObjects converted to entities at bake time or runtime.
- **ScenePartitioning**: Streaming via `SceneStreamingSystem` for open worlds.

### 1.8 Asset Loading

- **Entity Objects**: Assets referenced via `EntityObject` (managed wrapper) or `BlobAssetReference<T>` (unmanaged, serialized inline).
- **Blob Assets**: Immutable, burst-compatible data blobs stored inline in component memory.
- **SubScene Streaming**: Assets loaded/unloaded per scene partition.

### 1.9 Input

- Uses Unity's **Input System** package (Action-based).
- In ECS, input is typically read in a dedicated `InputSystem` that writes to singleton components.
- Input actions map to `InputAction` assets, polled per frame.

### 1.10 UI

- **UI Toolkit**: XML/CSS-based, integrates with ECS via `PanelEventHandler`.
- **uGUI**: Legacy, compatible via `InputSystemUIInputModule`.
- DOTS UI sample demonstrates hybrid GameObject/ECS UI patterns.

---

## 2. Unreal Engine

### 2.1 Actor-Component Model

Unreal uses an **inheritance-based** actor-component model:

- **AActor**: Base entity class with transform, replication, lifecycle.
- **UActorComponent**: Non-attachment component (no transform).
- **USceneComponent**: Transform-holding component (attachment hierarchy).
- **UPrimitiveComponent**: Renderable + collidable scene component.

```
AActor
  └─ USceneComponent (RootComponent)
       ├─ USkeletalMeshComponent
       ├─ UCameraComponent
       └─ USphereComponent (collision)
```

**Key distinction from ECS**: Components can have behavior (virtual methods), state (UPROPERTY), and inheritance. Not pure data.

### 2.2 Gameplay Ability System (GAS)

A production-grade framework for abilities, attributes, and effects:

| Component | Role |
|-----------|------|
| `UAbilitySystemComponent` | Bridge between Actor and GAS. Manages abilities, attributes, effects, tags. |
| `UGameplayAbility` | An action (active or passive). Contains execution logic, costs, cooldowns. |
| `FGameplayAttribute` | Float stat (health, mana, damage). Stored in `UAttributeSet`. |
| `UGameplayEffect` | Modifies attributes (instant, duration, infinite). Stacking rules. |
| `UAbilityTask` | Async sub-operation within an ability (wait for event, animate, etc.). |
| `FGameplayTag` | Hierarchical label for categorization (see below). |

**GAS Pipeline**:
```
Input → AbilitySystemComponent → ActivateAbility()
  → CommitAbility() (cost/cooldown)
  → ApplyGameplayEffect() → Modifies AttributeSet
  → GameplayCue (VFX/SFX feedback)
```

### 2.3 Gameplay Tags

Hierarchical string labels with dot-separated levels:

```
Damage.Type.Fire
Damage.Type.Ice
Character.Enemy.Zombie
Movement.Mode.Swimming
Event.RequestReset
```

**Operations**: `HasTag`, `HasAny`, `HasAll`, `MatchesTag` (parent-aware), `MatchesTagExact`.

**Storage**: `FGameplayTagContainer` — a set of tags with efficient matching. Defined via `.ini` files, DataTables, or C++ macros (`UE_DEFINE_GAMEPLAY_TAG`).

**Design pattern**: Tags replace enums for categorization, enabling hierarchical matching without inheritance.

### 2.4 Data-Driven Design

| Mechanism | Purpose |
|-----------|---------|
| `UDataAsset` | Single asset with custom fields. Blueprint-editable. |
| `UDataTable` | Rows mapped to `FTableRowBase` structs. Importable from CSV/Excel. |
| `UCurveTable` | Interpolation curves for float values. |
| `UDataRegistry` | Global registry for runtime data lookup. |

**Data flow**: Designer edits Excel → export CSV → import DataTable → gameplay reads `FindRow<T>()`.

### 2.5 Modular Gameplay Features

- **Game Feature Plugins**: Standalone feature modules that can be activated/deactivated at runtime.
- **Actions**: `AddComponents` (inject components onto actors), `AddDataRegistry`, `AddWorldPartitionContent`.
- Enables live-service modular content without code changes.

```
Plugins/GameFeatures/
  └─ MyFeature/
       ├─ MyFeature.uplugin
       └─ Content/
            └─ MyFeatureData (GameFeatureData asset)
```

### 2.6 Input: Enhanced Input System

- **UInputAction**: Logical action ("Jump", "Fire").
- **UInputMappingContext**: Maps physical inputs to actions, stackable for context switching.
- **UEnhancedInputComponent**: Actor component that binds actions to delegates.
- Supports modifier stacks, dead zones, player-mappable keys.

### 2.7 UI: UMG (Unreal Motion Graphics)

- **UUserWidget**: Base class for all UI.
- **Widget Blueprint**: Visual editor for layout.
- **Widget Component**: Renders UI in 3D world space.
- **Common UI Plugin**: Cross-platform input-aware UI framework.

### 2.8 Key Design Patterns

- **Delegate/Event**: Multi-cast delegates for decoupled communication (`DECLARE_DYNAMIC_MULTICAST_DELEGATE`).
- **Subsystem**: UEngineSubsystem, UWorldSubsystem, ULocalPlayerSubsystem — lifecycle-managed singletons.
- **Tick Group**: Deterministic update ordering via `PrePhysics`, `DuringPhysics`, `PostPhysics`.
- **Replication**: Built-in actor/property replication for multiplayer.

---

## 3. Godot

### 3.1 Node-Component Architecture

Godot uses a **scene-tree** of nodes, where scenes are composable sub-trees:

- **Node**: Base unit. Has a name, callbacks (`_process`, `_physics_process`), and can be extended via script.
- **Scene**: A tree of nodes saved as a `.tscn` file. Instanced as a unit.
- **Composition**: Scenes are instanced as child nodes, creating a scene-in-scene hierarchy.

```
MainScene (Node2D)
  ├─ Player (CharacterBody2D)
  │    ├─ Sprite2D
  │    ├─ CollisionShape2D
  │    └─ Camera2D
  └─ HUD (CanvasLayer)
       ├─ Label
       └─ Button
```

**Key distinction**: Nodes combine entity + component + system behavior in one object. Not a pure ECS.

### 3.2 Scene System

- Scenes are **blueprints** — templates that can be instanced repeatedly.
- `PackedScene` is a `Resource` that stores the node tree.
- Instancing: `load("res://enemy.tscn").instantiate()` → returns a node tree.
- **Inheritance**: Scenes can inherit from other scenes (scene inheritance, not just instancing).

### 3.3 Resource System

`Resource` is the base class for all serializable data:

- **Reference-counted** (via `RefCounted`).
- **Cached globally** by path — second `load()` returns the same instance.
- **Local to scene**: `set_local_to_scene(true)` duplicates the resource per scene instance.
- **PackedScene** is itself a Resource that can instantiate nodes.

```
Resource (RefCounted)
  ├─ Texture2D
  ├─ AudioStream
  ├─ Script
  ├─ PackedScene
  └─ CustomResource (user-defined)
```

**Loading patterns**:
- `load("res://path")` — load at runtime (async-capable).
- `preload("res://path")` — load at compile time (GDScript only).
- `ResourceLoader.load()` — async with progress callback.

### 3.4 Signal-Based Communication (Observer Pattern)

Signals are first-class objects (since Godot 4.0):

```gdscript
# Declaration
signal health_changed(new_health: int)

# Emission
health_changed.emit(current_health)

# Connection (in code)
enemy.health_changed.connect(_on_enemy_health_changed)

# Connection (in editor): Node dock → Signals tab → connect to method
```

**Properties**:
- Signals are typed (parameter lists).
- Can be connected to any `Callable` (method, lambda, other signal).
- Multiple receivers per signal.
- One-shot connections: `connect(callable, CONNECT_ONE_SHOT)`.
- Signal forwarding: `signal_a.connect(signal_b.emit)`.

**Design pattern**: Godot's primary decoupling mechanism. Replaces direct method calls between nodes.

### 3.5 GDExtension for Rust

**GDExtension** allows native code (C/C++/Rust) to extend Godot without recompiling the engine:

- **godot-rust (gdext)**: Rust bindings for Godot 4 GDExtension.
- Rust structs derive `#[derive(GodotClass)]` to become Godot classes.
- Supports `Node`, `Resource`, `RefCounted` base classes.
- `#[func]` exposes methods to GDScript/Editor.
- `#[export]` exposes properties to the Inspector.

```rust
#[derive(GodotClass)]
#[class(base=Node2D)]
struct Player {
    speed: f32,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Player {
    fn init(base: Base<Node2D>) -> Self {
        Self { speed: 200.0, base }
    }

    fn process(&mut self, delta: f64) {
        // game logic
    }
}
```

### 3.6 Input Handling

Godot uses **Input Actions** (project-wide named actions):

- **Polling**: `Input.is_action_pressed("jump")` in `_physics_process()`.
- **Event-based**: `_unhandled_input(event)` for gameplay input (after UI consumes).
- **`_input(event)`**: Raw input, before UI processing.
- Actions map multiple physical inputs (keyboard + gamepad + mouse).
- `Input.get_vector()` and `Input.get_axis()` for normalized movement.

### 3.7 UI

- **Control nodes**: Button, Label, Panel, etc. — inherit from `Control`.
- **Layout**: Anchors + containers (HBox, VBox, Grid, Margin).
- **Theme system**: StyleBox, Font, Color overrides. Theme resources shared across UI.
- **No separate UI language**: UI is just nodes in the scene tree.

### 3.8 Scene Management

- **SceneTree**: The root data structure. All nodes live in this tree.
- **`get_tree().change_scene_to_file("res://level2.tscn")`**: Replace current scene.
- **Scene switching**: The old scene is freed; the new scene becomes the tree root.
- **No native scene streaming**: Must implement manually or use addons.

---

## 4. Bevy (Rust-Native)

### 4.1 ECS with Schedule, SystemSet, States

Bevy's ECS is Rust-native with zero-cost abstractions:

```rust
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32 }

fn movement_system(
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}
```

**Schedule**: A dependency graph of systems that determines execution order.

```rust
app.add_systems(Update, (
    movement_system.after(physics_system),
    render_system.after(movement_system),
));
```

**SystemSet**: Named groups of systems for bulk configuration.

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet { Physics, Gameplay, Rendering }

app.configure_sets(Update, (
    GameSet::Physics,
    GameSet::Gameplay.after(GameSet::Physics),
    GameSet::Rendering.after(GameSet::Gameplay),
));
```

**States**: Finite state machines for app-wide mode management.

```rust
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    #[default] Loading,
    Playing,
    Paused,
}

app.add_systems(OnEnter(GameState::Playing), setup_game);
app.add_systems(OnExit(GameState::Playing), cleanup_game);
```

### 4.2 Resources vs Components

| Concept | Scope | Access | Use Case |
|---------|-------|--------|----------|
| `Component` | Per-entity | `Query` | Position, Health, Velocity |
| `Resource` | Global singleton | `Res<T>` / `ResMut<T>` | Time, AssetServer, Config |

```rust
#[derive(Resource)]
struct GameConfig { difficulty: f32 }

// In system:
fn my_system(config: Res<GameConfig>) {
    println!("Difficulty: {}", config.difficulty);
}
```

**Key distinction**: Resources are one-per-type, globally unique. Components are per-entity, many instances.

### 4.3 Events System

Events are typed messages that exist for one or more frames:

```rust
#[derive(Event)]
struct DamageEvent { target: Entity, amount: f32 }

// Sending
fn attack_system(
    mut events: EventWriter<DamageEvent>,
) {
    events.send(DamageEvent { target, amount: 25.0 });
}

// Receiving
fn damage_system(
    mut events: EventReader<DamageEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        if let Ok(mut h) = health.get_mut(event.target) {
            h.0 -= event.amount;
        }
    }
}
```

**Lifetime**: Events survive 2 update ticks (readable by systems that run after the sender).

### 4.4 App Builder Pattern

Bevy apps are built via a fluent builder:

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(GameConfig { difficulty: 1.0 })
    .init_state::<GameState>()
    .add_systems(Startup, setup)
    .add_systems(Update, (
        movement_system,
        collision_system.after(movement_system),
    ))
    .run();
```

**Plugin trait**: Modular extension point.

```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MyResource::default())
           .add_systems(Update, my_system);
    }
}

// Usage
app.add_plugins(MyPlugin);
```

### 4.5 Plugin System

Every subsystem in Bevy is a plugin:

```
DefaultPlugins = [
    CorePlugin,         // Time, TaskPool
    TransformPlugin,    // GlobalTransform
    DiagnosticsPlugin,  // FPS, frame time
    InputPlugin,        // Keyboard/mouse/gamepad
    AssetPlugin,        // Asset loading
    ScenePlugin,        // Scene management
    AudioPlugin,        // Sound
    WinitPlugin,        // Window creation
    RenderPlugin,       // GPU rendering
    ...
]
```

Plugins can depend on other plugins, add systems, resources, events, and configure schedules.

### 4.6 Asset Loading

- **AssetServer**: Async loading via `asset_server.load("path/to/asset")`.
- **Handle<T>**: Typed reference to an asset. Asset may not be loaded yet.
- **Assets<T>**: Storage for loaded assets. `Res<Assets<Mesh>>` in systems.
- **Hot reloading**: Assets update on disk → automatically reloaded.
- **bevy_asset_loader**: Community plugin for declarative asset collection loading with state-based transitions.

```rust
#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "player.png")]
    player_sprite: Handle<Image>,
    #[asset(path = "bgm.ogg")]
    background_music: Handle<AudioSource>,
}
```

### 4.7 Input

Bevy provides built-in input resources:

```rust
fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity>,
) {
    for mut vel in &mut query {
        if keys.pressed(KeyCode::KeyW) {
            vel.y = 1.0;
        }
    }
}
```

Supports: Keyboard, Mouse (button + motion), Gamepad (axes + buttons), Touch.

### 4.8 UI

Bevy UI uses Flexbox/CSS Grid layout:

```rust
commands.spawn(Node {
    width: percent(100.0),
    height: percent(100.0),
    justify_content: JustifyContent::Center,
    ..default()
}).with_children(|parent| {
    parent.spawn(Text::new("Hello"));
});
```

- **Node**: Layout container (flexbox).
- **Text**: Text rendering.
- **ImageNode**: Image display.
- **Interaction**: Hover/Press detection via components.
- **Observers**: Reactive pattern for widget events.

### 4.9 Observers (Reactive Pattern)

Bevy's newer observer pattern for entity-level events:

```rust
#[derive(Event)]
struct ClickEvent;

commands.spawn(Button).observe(|trigger: On<ClickEvent>, mut query: Query<&mut BackgroundColor>| {
    info!("Button clicked!");
});
```

**Propagation**: Events can propagate up/down the entity hierarchy via `EntityTraversal`.

---

## 5. Universal Abstraction Mapping

### 5.1 Core Abstraction Patterns

| Abstraction | Unity DOTS | Unreal | Godot | Bevy |
|-------------|-----------|--------|-------|------|
| **Entity** | Entity (int handle) | AActor | Node | Entity (int) |
| **Component** | IComponentData | UActorComponent | Node (self) | Component (struct) |
| **System** | ISystem / SystemBase | Tick functions | _process/_physics_process | fn System |
| **World/Scene** | World + SubScene | UWorld + ULevel | SceneTree | World |
| **Singleton** | Entity (singleton) | UGameInstance / Subsystem | AutoLoad node | Resource |
| **Event** | EventSystem (custom) | Delegate / Event Dispatcher | Signal | Event<T> |
| **State Machine** | Custom (or custom) | StateTree / AnimBP | Custom | States (built-in) |
| **Plugin/Module** | Package | UGameInstancePlugin / Module | GDExtension | Plugin trait |

### 5.2 Scene Management

| Pattern | Unity DOTS | Unreal | Godot | Bevy |
|---------|-----------|--------|-------|------|
| **Scene model** | SubScene (ECS world) | ULevel (actor collection) | SceneTree (node tree) | Scene (entity collection) |
| **Instancing** | Entity Instantiate | SpawnActor | PackedScene.instantiate() | Commands.spawn() |
| **Streaming** | SceneStreamingSystem | World Partition | Manual / addon | Manual / addon |
| **Switching** | LoadSceneAsync | OpenLevel | change_scene_to_file() | State transitions |
| **Composition** | Prefab → Entity | Blueprint | Scene instancing | Bundle + Plugin |

### 5.3 Asset Loading

| Pattern | Unity DOTS | Unreal | Godot | Bevy |
|---------|-----------|--------|-------|------|
| **Loading** | Addressables / Resources | TSoftObjectPtr / AssetManager | load() / preload() | AssetServer::load() |
| **Handle type** | AssetReference / BlobAsset | TSoftObjectPtr<T> | Resource (RefCounted) | Handle<T> |
| **Caching** | Automatic | Automatic | Automatic (by path) | Automatic (by handle) |
| **Async** | UnityWebRequest / Addressables | StreamableManager | ResourceLoader (threaded) | Async TaskPool |
| **Hot reload** | No (editor only) | Yes (editor) | Yes (running game) | Yes (running game) |
| **Format** | .asset, .prefab | .uasset | .tres, .tscn | Any (configurable) |

### 5.4 Input

| Pattern | Unity DOTS | Unreal | Godot | Bevy |
|---------|-----------|--------|-------|------|
| **Abstraction** | InputAction asset | InputAction | InputMap action | ButtonInput<T> |
| **Polling** | `InputSystem` singleton | Enhanced Input poll | `Input.is_action_pressed()` | `Res<ButtonInput<K>>` |
| **Events** | InputAction callbacks | Enhanced Input delegates | `_unhandled_input(event)` | EventReader |
| **Context** | Action map layers | MappingContext stack | Input actions | No built-in context |
| **Remapping** | Runtime rebinding | PlayerMappableKeys | Project Settings | Manual |

### 5.5 UI

| Pattern | Unity DOTS | Unreal | Godot | Bevy |
|---------|-----------|--------|-------|------|
| **Framework** | UI Toolkit / uGUI | UMG (Widget Blueprint) | Control nodes | Node (flexbox) |
| **Layout** | USS (CSS-like) | Canvas + Anchors | Anchors + Containers | Flexbox / CSS Grid |
| **Styling** | USS stylesheets | Widget Style / Material | Theme resources | Style components |
| **Data binding** | UXML binding | Property binding | Export vars + signals | Components + observers |
| **World UI** | World Space Canvas | Widget Component | SubViewport + Control | SubWorldBundle (manual) |

### 5.6 Key Design Patterns

| Pattern | Unity DOTS | Unreal | Godot | Bevy |
|---------|-----------|--------|-------|------|
| **Observer** | Custom event system | Delegate/Multicast | Signal (built-in) | Event<T> + Observer |
| **Command** | EntityCommand buffer | Console command | SceneTree.call_deferred | Commands (deferred) |
| **State Machine** | Custom | AnimBP / StateTree | Custom | States (built-in) |
| **Object Pool** | NativeArray reuse | Object pool pattern | Object pool pattern | Custom |
| **Service Locator** | World.GetExistingSystem | Subsystem registry | AutoLoad singleton | Resource |
| **Strategy** | System variants | Behavior tree tasks | Polymorphic nodes | SystemSet variants |
| **Flyweight** | Shared components | DataAsset instances | Resource sharing | Resource |
| **Data-Driven** | BlobAsset / DataTable | DataTable / DataAsset | Resource files | Asset<T> |

---

## 6. Cross-Engine Pattern Comparison

### 6.1 Composition vs Inheritance

| Engine | Primary Approach | Flexibility |
|--------|-----------------|-------------|
| Unity DOTS | **Strict composition** (ECS) | Highest — zero-cost, parallel |
| Unreal | **Inheritance + composition** (Actor-Component) | High — but OOP overhead |
| Godot | **Scene composition** (node trees) | High — intuitive, visual |
| Bevy | **Strict composition** (ECS) | Highest — zero-cost, Rust |

### 6.2 Parallelism Model

| Engine | Approach | Safety |
|--------|----------|--------|
| Unity DOTS | Job System + Burst (explicit parallel) | Job safety checks |
| Unreal | Task Graph (implicit parallelism) | Thread safety via design |
| Godot | Main thread + server threads | Locked servers |
| Bevy | Automatic system parallelism (dependency graph) | Rust ownership + Send/Sync |

### 6.3 Memory Model

| Engine | Allocation | Cache Strategy |
|--------|-----------|----------------|
| Unity DOTS | NativeArray (unmanaged) | Chunk-based (SoA) |
| Unreal | UObject (GC-managed) | Component arrays (AoS-ish) |
| Godot | RefCounted + Godot allocator | Node tree traversal |
| Bevy | Rust heap + arena | Archetype-based (SoA) |

### 6.4 Reflection / Introspection

| Engine | Mechanism |
|--------|-----------|
| Unity DOTS | `ISharedComponentData`, `TypeManager` |
| Unreal | **UObject reflection** (UPROPERTY, UFUNCTION, UClass) — deepest |
| Godot | `ClassDB`, `@export`, Variant system |
| Bevy | `Reflect` trait + `TypePath` |

### 6.5 Networking / Replication

| Engine | Built-in? | Approach |
|--------|----------|----------|
| Unity DOTS | Netcode for Entities | ECS state sync, snapshots |
| Unreal | Replication system | Actor replication, RPCs |
| Godot | ENetMultiplayerPeer | High-level multiplayer API |
| Bevy | No built-in (leafwing/bevy_renet) | Plugin ecosystem |

---

## 7. Abstraction Opportunities for NeoTrix

Based on cross-engine analysis, these are universal abstractions that map across all four engines:

| Abstraction | Description | Engines Using It |
|-------------|-------------|-----------------|
| **Entity** | Lightweight identity handle | All 4 |
| **Component** | Pure data attached to entities | DOTS, Bevy (strict); UE, Godot (loose) |
| **System/Logic** | Stateless processing of component queries | DOTS, Bevy (strict); UE (tick), Godot (_process) |
| **Resource/Singleton** | Globally unique data | Bevy (Resource), UE (Subsystem), Godot (AutoLoad) |
| **Event/Signal** | Decoupled communication | Godot (Signal), UE (Delegate), Bevy (Event), DOTS (custom) |
| **Schedule** | Deterministic execution ordering | Bevy (Schedule), UE (Tick Groups), DOTS (SystemGroup) |
| **State Machine** | App-level mode transitions | Bevy (States), UE (StateTree), DOTS/Godot (custom) |
| **Asset Handle** | Async reference to loaded data | All 4 (Handle/SoftObjectPtr/Resource) |
| **Plugin** | Modular extension point | All 4 (Plugin/Module/GDExtension/Package) |
| **Query** | Filtered access to entity data | DOTS (EntityQuery), Bevy (Query), UE (ActorIterator) |

---

*End of reference. 350+ lines.*
