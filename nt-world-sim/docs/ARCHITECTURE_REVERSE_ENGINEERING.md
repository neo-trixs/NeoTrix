# Game Engine Architecture Reverse Engineering

> Universal patterns extracted from Unity, Unreal, Godot, and Bevy.
> Source: Multi-engine analysis for NeoTrix nt-world-sim architecture blueprint.

---

## Table of Contents

1. [Unity Analysis](#1-unity-analysis)
2. [Unreal Analysis](#2-unreal-analysis)
3. [Godot Analysis](#3-godot-analysis)
4. [Bevy Analysis](#4-bevy-analysis)
5. [Universal Patterns](#5-universal-patterns)
6. [Architecture Blueprint](#6-architecture-blueprint)
7. [Implementation Recommendations](#7-implementation-recommendations)

---

## 1. Unity Analysis

### 1.1 ECS (Entity Component System) — DOTS

Unity's Data-Oriented Tech Stack (DOTS) represents a fundamental shift from MonoBehaviour to a cache-friendly, data-oriented architecture.

**Core Architecture:**
```
World
├── EntityManager
│   ├── Archetype Table (unique component combinations)
│   │   ├── Archetype M: [Transform, Velocity, Renderer]
│   │   │   └── Chunk (16 KiB) → SoA arrays per component
│   │   └── Archetype N: [Transform, Health]
│   │       └── Chunk (16 KiB) → SoA arrays per component
│   └── Entity → Archetype mapping
├── SystemBase / ISystem
│   ├── OnUpdate()
│   ├── SystemGroup (ordering)
│   └── SystemState (dependency tracking)
└── EntityQuery
    ├── WithAll<T...>()
    ├── WithAny<T...>()
    └── Without<T...>()
```

**Key Design Decisions:**

| Decision | Implementation | Implication |
|----------|---------------|-------------|
| Archetype-based storage | Entities with identical component sets share a chunk | Iteration over homogeneous data is cache-optimal |
| 16 KiB chunk size | Fixed-size blocks, SoA layout per component | Predictable memory access, no per-entity allocation |
| Entity = lightweight ID | Integer index + version counter | Zero-cost entity references, no inheritance |
| Component = plain struct | `ISharedComponent`, `IComponentData`, `IBufferElementData` | Pure data, no behavior, no virtual dispatch |
| System = stateless function | `SystemBase.OnUpdate(ref SystemState)` | Automatic dependency tracking, parallel safety |
| Change detection | `EnabledRefRO<T>`, `ChangeVersion` per component | Only process modified data |

**System Ordering:**
```
SystemGroup (e.g., SimulationSystemGroup)
├── BeginSimulationEntityCommandBufferSystem
├── PhysicsSystemGroup
├── TransformSystemGroup
├── EndSimulationEntityCommandBufferSystem
└── PresentationSystemGroup
    └── RenderMeshes (hybrid, bridges to MonoBehaviour)
```

**Archetype Migration:** Adding/removing components moves entity to new archetype (chunk copy). Archetypes stabilize early in program lifetime → query caching is effective.

### 1.2 Tilemap System

Unity's Tilemap is a grid-based 2D level design system built on `TileBase` inheritance.

**Architecture:**
```
Grid (GameObject)
├── Tilemap (GameObject)
│   ├── TilemapCollider2D
│   ├── TilemapRenderer
│   └── TilemapData (sparse cell storage)
├── Tilemap (layer 2)
└── Tilemap (layer N)

TileSet (ScriptableObject)
├── TileBase subclass list
├── TileSprite
└── TileCollider

2D Extras (com.unity.2d.tilemap.extras):
├── RuleTile (auto-tiling via 3×3 neighbor rules)
├── RandomTile (Perlin noise placement)
├── AnimatedTile (sprite sequence playback)
├── SuperTile (multi-tile sprites)
├── OverrideTile (runtime sprite override)
└── TilemapPalette (asset organization)
```

**RuleTile Pattern:**
- 3×3 grid represents neighbor relationships
- Each neighbor cell has 3 states: `Don't Care`, `This` (match), `Not This` (exclude)
- Supports `Rotated`, `Mirror X/Y/XY` output modes
- Custom rules via `RuleTile<TNeighbor>` generic pattern
- Rule evaluation order matters (first match wins)

**Key Insight:** RuleTile implements a mini inference engine — pattern matching on local neighborhood. This is the same pattern used in procedural terrain generation (Wang tiles, marching squares).

### 1.3 Dialogue System (Yarn Spinner)

Yarn Spinner is Unity's dominant dialogue framework, used in Night in the Woods, A Short Hike, DREDGE.

**Architecture:**
```
YarnProject (ScriptableObject)
├── Program (compiled bytecode)
├── SharedVariables ($player_name, $quest_state)
└── Localization strings

DialogueRunner (MonoBehaviour)
├── DialogueHandler (compiled program)
├── DialogueViewBase[] (UI adapters)
└── VariableStorageBehaviour

Dialogue Flow:
  .yarn file → Compiler → Program → DialogueRunner → DialogueViewBase
```

**Node-Based Scripting:**
```yarn
title: Start
---
Narrator: Welcome, adventurer!
-> Go to forest <<jump Forest>>
-> Visit shop <<jump Shop>>

title: Forest
---
<<if $has_sword>>
Guard: You may pass!
<<else>>
Guard: You need a sword!
<<endif>>
```

**Yarn Spinner 3.0 Extensions:**
- **Storylets:** `when:` conditions with saliency scoring (most-specific wins)
- **Line Groups:** `=>` prefix for random variation selection
- **Smart Variables:** Computed variables with `= expression`
- **Shadow Lines:** Variable change listeners
- **Async Dialogue Views:** C# async/await integration

### 1.4 Key Unity Patterns

| Pattern | Unity Implementation | Generic Form |
|---------|---------------------|--------------|
| Data Container | ScriptableObject | Resource/asset as data blob |
| Event System | UnityEvent / C# events | Observer/delegate pattern |
| Async Loading | Addressable Assets | Handle-based async resource |
| Assembly Isolation | Assembly Definition (.asmdef) | Module boundaries |
| Hybrid ECS | MonoBehaviour + Entity | Bridge between paradigms |
| Visual Scripting | Visual Scripting (Bolt) | Node graph → bytecode |

---

## 2. Unreal Analysis

### 2.1 Component System — UObject

Unreal's architecture is built on UObject reflection, not classical ECS. It's a deep inheritance hierarchy with component composition.

**Object Hierarchy:**
```
UObjectBase
├── UObjectBaseUtility
│   └── UObject
│       ├── AActor (placed in world)
│       │   ├── ACharacter
│       │   ├── AController
│       │   └── APawn
│       ├── UActorComponent (no transform)
│       │   ├── USceneComponent (has transform)
│       │   │   └── UPrimitiveComponent (rendering + collision)
│       │   └── UNavigationInvokerComponent
│       └── USubsystem (engine services)
```

**Reflection System (UHT):**
```
UCLASS() → UClass (runtime type info)
UPROPERTY() → UProperty (serialized, GC-tracked, Blueprint-visible)
UFUNCTION() → UFunction (callable from Blueprint, replicated)
USTRUCT() → UStruct (value type, serialized)
UENUM() → UEnum (Blueprint-exposed enumeration)
```

**Key Insight:** UObject is not just a base class — it's a meta-object protocol. Every UCLASS has a UClass that describes its properties, functions, and inheritance chain. This enables:
- Automatic garbage collection (reference graph walking)
- Blueprint integration (property editor, event graph)
- Network replication (UPROPERTY replication)
- Serialization (binary + JSON)

### 2.2 Blueprint System

Blueprints are Unreal's visual scripting — compiled to bytecode executed on a stack-based VM.

**Compilation Pipeline:**
```
UBlueprint (asset)
├── UEdGraph (event graph, function graphs, macro graphs)
│   ├── UEdGraphNode (visual nodes)
│   └── UEdGraphPin (data connections)
├── FKismetCompilerContext
│   ├── Schema validation
│   ├── Type inference
│   ├── Latent node expansion (async tasks)
│   └── Bytecode generation
└── UBlueprintGeneratedClass (runtime UClass)
    ├── FKismetFunctionTable (function bytecode)
    └── FProperty tree (inherited + local properties)
```

**Blueprint Event Flow:**
```
Event Dispatcher (multicast delegate)
├── BindEvent (C++ or Blueprint)
├── CallEvent (from Blueprint graph)
└── UnbindEvent (cleanup)
```

**Key Patterns:**
- **Blueprint Interface:** Contract between unrelated Blueprints (duck typing via name matching)
- **Blueprint Function Library:** Static functions callable from any Blueprint
- **Event Dispatchers:** Multicast delegates for loose coupling
- **Construction Script:** Procedural setup at edit-time and spawn-time

### 2.3 Gameplay Ability System (GAS)

GAS is Unreal's production-grade framework for RPG/MOBA abilities, attributes, and effects.

**Architecture:**
```
UAbilitySystemComponent (per Actor)
├── Granted Abilities (UGameplayAbility[])
├── Active Effects (FActiveGameplayEffect[])
├── Gameplay Tags (FGameplayTagContainer)
├── Attribute Sets (UAttributeSet)
└── Gameplay Cues (audio/visual feedback)

UGameplayAbility
├── ActivateAbility() → async execution
├── EndAbility() → cleanup
├── CanActivateAbility() → tag + cost checks
├── CommitAbility() → resource cost + cooldown
└── Ability Tasks (async sub-operations)

UGameplayEffect
├── Duration Policy (Instant / Duration / Infinite)
├── Modifiers (add/multiply/override attribute)
├── Stacking (aggregate by source/target)
└── Gameplay Cues (on active, on execute, on remove)

FGameplayTag (hierarchical namespace)
├── Ability.Skill.Fireball
├── State.Dead
├── Damage.Physical
└── Tag matching: parent matches all children
```

**Tag-Based State Machine:**
```cpp
// Grant tag on ability activation
AbilitySystemComponent->AddLooseGameplayTag(FGameplayTag::RequestGameplayTag("State.Stunned"));

// Block abilities while stunned
if (AbilitySystemComponent->HasMatchingGameplayTag("State.Stunned")) {
    return false; // Cannot activate
}
```

**Key Insight:** GAS replaces traditional FSM/BT with a tag-driven architecture. State is represented as tags, transitions are tag additions/removals, and conditions are tag queries. This is more flexible than explicit state machines because states can compose (you can be Stunned AND Burning simultaneously).

### 2.4 Gameplay Tags

```
FGameplayTag (atomic tag)
FGameplayTagContainer (set of tags)
FGameplayTagQuery (boolean expression over tags)

Hierarchy:
  State
  ├── State.Alive
  │   ├── State.Alive.Healthy
  │   └── State.Alive.Injured
  └── State.Dead
  
  Ability
  ├── Ability.Skill
  │   ├── Ability.Skill.Fireball
  │   └── Ability.Skill.IceBlast
  └── Ability.Passive
```

**Usage Pattern:** Every gameplay state, ability, effect, and condition is expressed as tags. The system queries tag hierarchies, enabling pattern matching like "any ability under Ability.Skill."

### 2.5 Tilemap Tools (Paper2D)

Unreal's Paper2D provides sprite-based tilemap editing:
```
UPaperTileMap (Actor Component)
├── FTileMapRenderData
│   ├── Layers (rendering order)
│   └── Cells (tile data per layer)
├── UPaperTileSet
│   ├── Sprite sheets
│   ├── Tile collision data
│   └── Custom data per tile
└── FTileMapProxy (rendering optimization)
```

**Key Limitation:** Paper2D is less mature than Unity/Godot tilemap systems. Most Unreal 2D games use custom tilemap solutions.

### 2.6 Key Unreal Patterns

| Pattern | Unreal Implementation | Generic Form |
|---------|----------------------|--------------|
| Reflection | UHT macros (UCLASS, UPROPERTY) | Runtime type info + metadata |
| Component | ActorComponent / SceneComponent | Composition over inheritance |
| Tag System | Gameplay Tags (hierarchical) | Namespace-tagged state |
| Ability System | GAS (ability + effect + attribute) | Capability-based architecture |
| Visual Scripting | Blueprint (node graph → bytecode) | Visual programming |
| GC | UObject reference graph | Tracing garbage collector |
| Data-Driven | DataTable, DataAsset, CurveTable | Resource-based configuration |

---

## 3. Godot Analysis

### 3.1 Node System

Godot uses a scene tree of Nodes — the simplest and most composable architecture of all four engines.

**Architecture:**
```
SceneTree
├── Root (Window)
│   ├── GameScene (Node2D)
│   │   ├── Player (CharacterBody2D)
│   │   │   ├── Sprite2D
│   │   │   ├── CollisionShape2D
│   │   │   └── AnimationPlayer
│   │   ├── TileMapLayer (TileMapLayer)
│   │   └── UI (CanvasLayer)
│   │       ├── HealthBar (ProgressBar)
│   │       └── DialogueBox (Control)
│   └── Camera2D
```

**Node Lifecycle:**
```
_enter_tree()  → Node added to tree
_ready()       → All children ready (post-injection)
_process(delta) → Frame update (game logic)
_physics_process(delta) → Fixed timestep (physics)
_exit_tree()   → Node removed from tree
_free()        → Deallocated
```

**Node Communication:**
```
1. Signals (Observer pattern)
   signal health_changed(new_health: int)
   health_changed.connect(on_health_changed)

2. Groups (Tag/category)
   add_to_group("enemies")
   get_tree().get_nodes_in_group("enemies")

3. Direct reference
   get_node("../Enemy")

4. Autoload (Singleton)
   GlobalState.score += 10
```

**Key Design Principle:** "Composition over inheritance" — nodes are tiny, reusable units that attach to form complex behaviors. A CharacterBody2D might have 10+ child nodes each handling one concern.

### 3.2 TileMap System (Godot 4.x)

Godot's TileMap is the most mature tile system of all four engines.

**Architecture:**
```
TileSet (Resource)
├── TileSetSource[]
│   ├── TileSetAtlasSource (sprite sheet tiles)
│   │   ├── Tiles (indexed by atlas_coords + alternative_id)
│   │   ├── Physics layers (collision shapes)
│   │   ├── Navigation layers (pathfinding areas)
│   │   ├── Occlusion layers (light blocking)
│   │   └── Custom data layers (game properties)
│   └── TileSetScenesCollectionSource (scene-based tiles)
├── Terrain sets (auto-tiling rules)
│   ├── Terrain set (group of terrains)
│   │   ├── Terrain "Grass"
│   │   ├── Terrain "Dirt"
│   │   └── Terrain "Water"
│   └── Terrain mode (Match Corners / Match Sides / Match Corners and Sides)
├── Tile proxies (visual aliasing)
└── Patterns (TileMapPattern for procedural placement)

TileMapLayer (Node2D, Godot 4.3+)
├── Cell data (source_id, atlas_coords, alternative_id, transform)
├── Rendering layers
└── Physics layers

Tile identification (triple ID):
  source_id → atlas_source_id → alternative_tile_id
```

**Auto-Tiling (Terrain System):**
```
Terrain Set (mode: Match Corners and Sides)
├── Terrain 0: Grass (priority 0)
├── Terrain 1: Dirt (priority 1)
└── Terrain 2: Water (priority 2)

Rules define which terrain peering bits connect:
  Grass connects to: Grass, Dirt
  Dirt connects to: Grass, Dirt, Water
  Water connects to: Water only
```

**Key Insights:**
- **Property Layers:** Physics, navigation, occlusion, and custom data are separate layers on the TileSet, not mixed into tile data
- **Triple ID:** Tiles are identified by (source, atlas_coords, alternative) — supports multiple visual variants per logical tile
- **Scene Tiles:** A tile can be an entire instanced scene, enabling complex behavior per cell
- **Deferred Updates:** All tilemap changes batched at frame end for performance

### 3.3 Dialogue System (DialogueManager)

DialogueManager is the dominant Godot dialogue plugin (production-ready, MIT licensed).

**Architecture:**
```
DialogueManager (Autoload singleton)
├── DialogueHandler (runtime state)
├── DialogueResource (.dialogue files)
│   └── Nodes (title → dialogue lines + options + jumps)
└── DialogueLabel (UI node for timed text)

Dialogue Script Format:
  ~ start
  NPC: Hello there!
  => Option 1
      << set $likes_player = true >>
      NPC: Thanks!
  => Option 2
      NPC: Oh, okay...
  NPC: Goodbye!
  << jump next_conversation >>
```

**Stateless Design:**
- DialogueManager itself holds NO game state
- Game state (variables, flags) lives in your game's autoloads
- Mutations (`<< set $var = value >>`) call your game functions
- This prevents dialogue system from becoming a hidden state store

**Integration Pattern:**
```gdscript
# Start dialogue
DialogueManager.show_dialogue_balloon("res://dialogues/npc_intro.dialogue", "start")

# Handle responses
DialogueManager.dialogue_spoken.connect(on_dialogue_spoken)

func on_dialogue_spoken(line: DialogueLine):
    match line.character:
        "Guard":
            guard_sprite.play("talk")
```

### 3.4 Scene System

**PackedScene as Composable Unit:**
```
PackedScene (Resource)
├── Node tree (serialized)
├── Export variables (data)
└── Connections (signals)

Scene Instancing:
  var enemy = preload("res://enemies/goblin.tscn").instantiate()
  add_child(enemy)
```

**Key Patterns:**
- **Tool Scripts:** Run in editor, enable procedural level design
- **Export Variables:** Expose properties to editor, enable data-driven design
- **Custom Resources:** Define new data types with editor integration
- **Scene Inheritance:** Extend scenes without modifying base (override behavior)

### 3.5 Key Godot Patterns

| Pattern | Godot Implementation | Generic Form |
|---------|---------------------|--------------|
| Scene Tree | Node hierarchy | Recursive composition |
| Signals | Typed signal emission | Observer pattern |
| Resources | Custom Resource classes | Data-driven assets |
| Autoloads | Named singleton nodes | Service locator |
| Groups | String-based tagging | Dynamic categorization |
| Export Vars | `@export` annotations | Editor data binding |
| Scene Composition | PackedScene instantiation | Prefab pattern |

---

## 4. Bevy Analysis

### 4.1 ECS Implementation

Bevy's ECS is a pure-Rust implementation emphasizing ergonomics and parallelism.

**Architecture:**
```
World
├── Entities (u64 ID + generation counter)
├── Archetypes
│   ├── Archetype 0: [Position, Velocity]
│   │   └── Table (column-per-component, row-per-entity)
│   └── Archetype 1: [Position, Health]
│       └── Table
├── Resources (singletons: Time, Input, etc.)
└── Events (bounded queue: EventReader/EventWriter)

Schedule
├── SystemSet (grouped systems)
│   ├── First (startup-adjacent)
│   ├── PreUpdate
│   ├── Update
│   ├── PostUpdate
│   └── Last
├── System Ordering (explicit: .chain(), .before(), .after())
└── RunCondition (conditional execution)
```

**Query System:**
```rust
// Read-only query
fn read_positions(query: Query<(&Position, &Velocity)>) { ... }

// Mutable query
fn move_entities(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}

// Filtered query
fn enemies_near_player(
    query: Query<(&Position, &Health), (With<Enemy>, Without<Player>)>,
    player: Query<&Position, With<Player>>,
) { ... }

// Change detection
fn react_to_damage(query: Query<&Health, Changed<Health>>) { ... }

// Added detection
fn on_spawn(query: Query<Entity, Added<Player>>) { ... }
```

**Component Storage:**
- **Table Storage:** Default. Column-per-component, row-per-entity. Best for iteration.
- **Sparse Set:** O(1) insert/remove, worse iteration. Good for components that change frequently.

**Key Insight:** Bevy's query system is the most ergonomic of all four engines. Rust's type system enables compile-time borrow checking of ECS access — impossible to have data races in queries.

### 4.2 Plugin System

Bevy's plugin system is its core architectural pattern — everything is a plugin.

```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GameConfig::default())
            .add_systems(Startup, setup_system)
            .add_systems(Update, (movement_system, collision_system).chain())
            .add_systems(FixedUpdate, physics_system);
    }
}

// Registration
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)  // Window, Audio, Rendering, Input
        .add_plugins(MyPlugin)
        .run();
}
```

**DefaultPlugins includes:**
- WindowPlugin → WinitPlugin (OS window)
- RenderPlugin → WgpuPlugin (GPU rendering)
- AssetPlugin (async loading)
- AudioPlugin
- InputPlugin
- TransformPlugin

### 4.3 State Management

Bevy has a built-in state machine system.

```rust
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

// State-dependent systems
app.add_systems(OnEnter(GameState::Playing), setup_level);
app.add_systems(OnExit(GameState::Playing), cleanup_level);
app.add_systems(Update, gameplay_system.run_if(in_state(GameState::Playing)));
```

**SubStates:** Hierarchical states for complex UI flows:
```rust
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
struct MenuOpen {
    menu_type: MenuType,
}
```

### 4.4 Event System

```rust
#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: f32,
    source: Entity,
}

// Emit events
fn attack_system(mut events: EventWriter<DamageEvent>) {
    events.send(DamageEvent { target, amount: 10.0, source });
}

// Consume events (automatic cleanup after read)
fn damage_handler(mut events: EventReader<DamageEvent>, mut health: Query<&mut Health>) {
    for event in events.read() {
        if let Ok(mut hp) = health.get_mut(event.target) {
            hp.current -= event.amount;
        }
    }
}
```

**Key Pattern:** Events are buffered and auto-cleaned after all readers have consumed them. No manual cleanup needed.

### 4.4 Tilemap (bevy_ecs_tilemap)

```rust
// TilemapBundle configuration
commands.spawn(TilemapBundle {
    grid_size: TilemapGridSize::new(16.0, 16.0),
    map_type: TilemapType::Square,
    tile_size: TilemapTileSize::new(16.0, 16.0),
    storage: TilemapStorage::new(chunk_size, tilemap_entity),
    transform: get_tilemap_center_transform(&map_size, &grid_size, 0.0),
    render_settings: TilemapRenderSettings {
        render_chunk_size: UVec2::new(16, 16),
        ..default()
    },
    ..default()
});
```

**Key Features:**
- GPU instanced rendering (thousands of tiles in single draw call)
- Chunk-based loading (only render visible chunks)
- Custom shaders for per-tile effects
- Height-based sorting for isometric views

### 4.5 Key Bevy Patterns

| Pattern | Bevy Implementation | Generic Form |
|---------|--------------------|--------------|
| ECS | World + Query + System | Data-oriented architecture |
| Plugin | `impl Plugin for T` | Modular composition |
| Resource | `Res<T>` / `ResMut<T>` | Global singleton state |
| State | `States` trait + `OnEnter`/`OnExit` | Built-in FSM |
| Event | `EventReader<T>` / `EventWriter<T>` | Auto-cleanup pub/sub |
| Schedule | `Schedule` + `SystemSet` | Execution ordering |
| Bundle | `#[derive(Bundle)]` | Component grouping |
| Change Detection | `Changed<T>`, `Added<T>` | Reactive updates |

---

## 5. Universal Patterns

### 5.1 ECS Pattern (Entity-Component-System)

**Cross-Engine Comparison:**

| Aspect | Unity DOTS | Unreal | Godot | Bevy |
|--------|-----------|--------|-------|------|
| Entity | ID + version | UObject pointer | Node instance ID | u64 + generation |
| Component | `IComponentData` (struct) | UActorComponent (class) | Node children | `#[derive(Component)]` (struct) |
| System | `SystemBase.OnUpdate()` | `Tick()` methods | `_process()` / `_physics_process()` | `fn system()` |
| Storage | Archetype + Chunk (16 KiB) | UObject property tree | Scene tree hierarchy | Archetype + Table |
| Query | `EntityQuery` + filters | Component traversal | `get_nodes_in_group()` | `Query<T, F>` |
| Change Detection | Version numbers | Manual | Manual | `Changed<T>` / `Added<T>` |

**Universal ECS Pattern:**
```
Entity = unique identifier (lightweight, zero-cost)
Component = pure data (no behavior, no inheritance)
System = stateless function operating on component sets
Query = specification of which entities to process
World = container for all entities, components, and systems
```

**Academic Finding (Tasnim & Zhao, 2026):**
> "The archetype ECS achieves higher frame rate and better frame stability than alternative designs, due to improved cache efficiency and consistent entity access."
> - Formal model captures entity creation, component composition, system execution, and archetype migration as compositional state transitions.
> - Archetype-based layouts excel at large-scale iteration through improved cache efficiency.

### 5.2 Event-Driven Pattern (Pub/Sub)

| Engine | Implementation | Type Safety | Auto Cleanup |
|--------|---------------|-------------|--------------|
| Unity | C# events + UnityEvent | Strong | Manual |
| Unreal | Delegates (single/multi) + Event Dispatchers | Strong (template) | Manual |
| Godot | Signals (typed, first-class) | Strong | Automatic |
| Bevy | EventReader/EventWriter<T> | Strong (compile-time) | Automatic |

**Universal Pattern:**
```
Event = value object (timestamp, type, payload)
EventBus = central dispatcher (typed channels)
Producer = emits events to bus
Consumer = subscribes to bus, filters by type
Delivery = immediate (synchronous) or buffered (async)
```

### 5.3 State Machine Pattern (FSM)

| Engine | Implementation | Complexity |
|--------|---------------|------------|
| Unity | Animator Controller (visual FSM) | Complex (node graphs) |
| Unreal | Gameplay Tags (implicit FSM) | Flexible (composable) |
| Godot | Node-based FSM (per-entity) | Simple (child nodes) |
| Bevy | States trait (global FSM) | Simple (enum + systems) |

**Universal FSM Pattern:**
```
State = enum variant or named node
Transition = (from_state, to_state, condition)
Guard = boolean predicate on transition
Action = side effect on enter/exit
Hierarchical FSM = state contains sub-states
```

**Best Practice (from Godot community):**
```gdscript
# Node-based FSM (most composable)
class State:
    func enter(msg: Dictionary = {}): pass
    func exit(): pass
    func update(delta: float): pass
    func handle_input(event: InputEvent): pass

class StateMachine:
    var current: State
    func transition(to: State, msg: Dictionary = {}):
        current.exit()
        current = to
        current.enter(msg)
```

### 5.4 Behavior Tree Pattern (BT)

| Engine | Implementation | Notes |
|--------|---------------|-------|
| Unity | Behavior Designer, NodeCanvas | Third-party plugins |
| Unreal | BehaviorTree + Blackboard (built-in) | Production-grade |
| Godot | LimboAI, Blackboard | Plugin-based |
| Bevy | `bevy_behavior_tree` | Community crate |

**Universal BT Pattern:**
```
Composite:
  ├── Sequence (AND — all children must succeed)
  ├── Selector (OR — first child that succeeds)
  ├── Parallel (run N children simultaneously)
  └── RandomSelector (random order)

Decorator:
  ├── Inverter (negate result)
  ├── Repeater (run N times)
  ├── UntilFail (loop until failure)
  └── Cooldown (rate limit)

Leaf:
  ├── Action (do something, return Success/Running/Failure)
  ├── Condition (check something)
  └── Wait (pause)

Blackboard = shared key-value store for BT state
```

**Unreal's Blackboard:**
```
UBlackboardData
├── Keys (typed: Bool, Int, Float, String, Vector, Object, Enum)
├── Parent blackboard (inheritance)
└── Native values (C++ defined)

UBehaviorTreeComponent
├── Execute root node
├── Tick rate control
└── Abort types (self, lower priority, both)
```

### 5.5 Data-Driven Pattern (JSON/Config)

| Engine | Implementation | Format |
|--------|---------------|--------|
| Unity | ScriptableObject, Addressable | Binary + YAML |
| Unreal | DataTable, DataAsset, CurveTable | CSV, JSON, custom |
| Godot | Custom Resources | .tres (text), .res (binary) |
| Bevy | serde + any config format | JSON, TOML, RON, YAML |

**Universal Data-Driven Pattern:**
```
Schema = type definition (struct, class, resource)
Instance = concrete data conforming to schema
Loader = reads data from disk/network
Validator = ensures data conforms to schema
Hot Reload = re-load on file change (editor only)
Serialization = schema ↔ bytes (JSON, binary, custom)
```

### 5.6 Tilemap Pattern (Grid-Based Rendering)

**Universal Tilemap Architecture:**
```
TileSet/TilePalette
├── Tile Definition
│   ├── Visual (sprite, animation)
│   ├── Physics (collision shape)
│   ├── Navigation (walkable/not)
│   ├── Custom Data (properties)
│   └── Auto-tile Rules (neighbor matching)
├── Terrain System (auto-tiling)
│   ├── Terrain Sets (groups)
│   ├── Terrain Peering Bits (connection rules)
│   └── Priority (fallback resolution)
└── Tile Proxies (aliasing)

TileMap/TileMapLayer
├── Cell Storage (sparse or dense)
│   ├── source_id + atlas_coords + alternative_id
│   └── Transform (flip, rotate, offset)
├── Rendering
│   ├── Chunk-based batching
│   ├── GPU instancing
│   └── Sorting layers
├── Physics Integration
│   ├── Per-tile collider
│   └── Composite collider (optimization)
└── Layers (depth ordering)
```

**Auto-Tile Algorithm (universal across engines):**
```
1. For each cell, examine 4/8 neighbors
2. Generate bitmask from neighbor similarity
3. Look up tile variant from bitmask table
4. Apply visual (sprite, rotation, flip)
5. Handle edge cases (corners, transitions)
```

### 5.7 Dialogue Pattern (Node-Based Branching)

**Universal Dialogue Architecture:**
```
DialogueFile
├── Nodes (title → content)
│   ├── Lines (text + speaker + metadata)
│   ├── Options (player choices → jumps)
│   ├── Commands (<< imperative >>)
│   └── Conditions (<< if >> / << else >>)
├── Variables (persistent state)
│   ├── Game variables ($quest_complete)
│   ├── Temp variables (local scope)
│   └── Smart variables (computed)
└── Storylets (conditional availability)

Runtime:
  DialogueRunner
  ├── CurrentNode (execution pointer)
  ├── LineQueue (sequential playback)
  ├── OptionSelector (player input)
  ├── VariableStorage (state backend)
  └── DialogueView (UI adapter)
```

**Key Patterns:**
- **Node = scope:** Each node is an independent dialogue segment
- **Jump = goto:** `<< jump NodeTitle >>` for branching
- **Options = choice points:** Player selects from available options
- **Variables = state:** Persistent across dialogue sessions
- **Conditions = guards:** Show/hide lines based on state
- **Storylets = saliency:** Most-specific available node wins

### 5.8 Quest Pattern (State Machine)

**Universal Quest Architecture:**
```
Quest
├── ID (unique identifier)
├── State (enum: Available/Active/Completed/Failed)
├── Objectives (multiple per quest)
│   ├── Objective ID
│   ├── Type (kill, collect, talk, reach, custom)
│   ├── Target (entity/area/item reference)
│   ├── Count (progress tracking)
│   └── State (incomplete/complete)
├── Prerequisites (other quests that must be done)
├── Rewards (experience, items, unlocks)
└── Dialogue Hooks (on accept, on progress, on complete)

Quest State Machine:
  Unavailable → Available → Active → Completed
                                  → Failed → Available (retry)
```

**GAS Integration (Unreal):**
```
Quest State as Gameplay Tags:
  Quest.MainStory.FindSword     (state: Inactive/Active/Complete)
  Quest.Sidequest.GatherHerbs   (state: Inactive/Active/Complete)

Quest progression triggers Gameplay Effects:
  QuestComplete → grant XP → modify Attribute("Experience")
  QuestComplete → grant Item → modify Inventory component
```

---

## 6. Architecture Blueprint

### 6.1 Design Principles

From cross-engine analysis, the optimal game architecture follows these principles:

1. **Data-Oriented Core:** ECS for high-performance entity management
2. **Composition Over Inheritance:** Small, reusable components
3. **Event-Driven Communication:** Loose coupling between systems
4. **Data-Driven Configuration:** JSON/config for all game content
5. **Modular Plugin Architecture:** Each subsystem is independent
6. **Tag-Based State:** Composable state via hierarchical tags
7. **Resource-Based Assets:** Typed resources with hot reload

### 6.2 Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    L6: Tooling Layer                     │
│  Editor UI / Visual Scripting / Node Graph / Inspector   │
├─────────────────────────────────────────────────────────┤
│                    L5: Game Logic Layer                  │
│  Quest System / Dialogue System / Inventory / AI         │
├─────────────────────────────────────────────────────────┤
│                    L4: Framework Layer                   │
│  ECS Core / Event Bus / State Machine / Plugin System    │
├─────────────────────────────────────────────────────────┤
│                    L3: Subsystem Layer                   │
│  Tilemap / Rendering / Physics / Audio / Input / Network │
├─────────────────────────────────────────────────────────┤
│                    L2: Platform Layer                    │
│  Window / Filesystem / Threading / GPU Abstraction       │
├─────────────────────────────────────────────────────────┤
│                    L1: OS/Hardware Layer                 │
│  Memory Management / SIMD / Multithreading / GPU         │
└─────────────────────────────────────────────────────────┘
```

### 6.3 Core ECS Design

```rust
// Entity: lightweight identifier
struct Entity(u64, u32); // id + generation

// Component: pure data, no behavior
trait Component: 'static + Send + Sync {
    fn type_id() -> TypeId;
}

// System: stateless function
trait System: 'static + Send + Sync {
    fn run(&self, world: &mut World, resources: &Resources);
    fn reads(&self) -> Vec<TypeId>;  // component types read
    fn writes(&self) -> Vec<TypeId>; // component types written
}

// World: container for all state
struct World {
    archetypes: HashMap<ArchetypeId, Archetype>,
    entities: Vec<EntityMeta>,
    resources: Resources,
    events: EventRegistry,
}

// Archetype: groups entities with same components
struct Archetype {
    component_types: HashSet<TypeId>,
    tables: HashMap<TypeId, Box<dyn AnyTable>>, // column per component
    entities: Vec<Entity>,
}
```

### 6.4 Plugin Architecture

```rust
// Plugin trait: modular composition
trait Plugin: 'static {
    fn build(&self, app: &mut App);
    fn name(&self) -> &str { "unnamed" }
    fn dependencies(&self) -> Vec<&str> { vec![] }
}

// App builder: fluent configuration
struct App {
    world: World,
    schedule: Schedule,
    plugins: Vec<Box<dyn Plugin>>,
    startup_systems: Vec<Box<dyn System>>,
    update_systems: Vec<Box<dyn System>>,
}

impl App {
    fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }
    
    fn add_startup_system<S: System>(&mut self, system: S) -> &mut Self {
        self.startup_systems.push(Box::new(system));
        self
    }
    
    fn add_system<S: System>(&mut self, set: &str, system: S) -> &mut Self {
        self.schedule.add_system(set, Box::new(system));
        self
    }
}
```

### 6.5 Event Bus Design

```rust
// Typed event channel
struct EventChannel<E: Event> {
    buffer_a: Vec<E>,
    buffer_b: Vec<E>,
    read_buffer: usize,
    write_buffer: usize,
    readers: HashMap<TypeId, usize>, // reader → last read position
}

// Event bus with typed channels
struct EventBus {
    channels: HashMap<TypeId, Box<dyn AnyChannel>>,
}

impl EventBus {
    fn send<E: Event>(&mut self, event: E) {
        let channel = self.channels
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventChannel::<E>::new()));
        channel.downcast_mut::<EventChannel<E>>().unwrap().push(event);
    }
    
    fn read<E: Event>(&mut self) -> impl Iterator<Item = &E> {
        let channel = self.channels.get_mut(&TypeId::of::<E>()).unwrap();
        channel.downcast_mut::<EventChannel<E>>().unwrap().drain_read()
    }
}
```

### 6.6 Tag System Design

```rust
// Hierarchical tag (like Unreal Gameplay Tags)
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct TagPath(Vec<String>); // ["Ability", "Skill", "Fireball"]

// Tag query
enum TagQuery {
    Has(TagPath),
    HasAny(Vec<TagPath>),
    HasAll(Vec<TagPath>),
    Not(Box<TagQuery>),
    And(Box<TagQuery>, Box<TagQuery>),
    Or(Box<TagQuery>, Box<TagQuery>),
}

// Tag container (per entity)
struct TagContainer {
    tags: HashSet<TagPath>,
}

impl TagContainer {
    fn has(&self, tag: &TagPath) -> bool {
        // Check exact match OR any parent
        self.tags.contains(tag) || 
        self.tags.iter().any(|t| t.is_prefix_of(tag))
    }
    
    fn matches(&self, query: &TagQuery) -> bool {
        match query {
            TagQuery::Has(tag) => self.has(tag),
            TagQuery::Not(q) => !self.matches(q),
            TagQuery::And(a, b) => self.matches(a) && self.matches(b),
            TagQuery::Or(a, b) => self.matches(a) || self.matches(b),
            // ...
        }
    }
}
```

### 6.7 Dialogue System Design

```rust
// Dialogue node
struct DialogueNode {
    title: String,
    lines: Vec<DialogueLine>,
    options: Vec<DialogueOption>,
    conditions: Vec<Condition>,
}

struct DialogueLine {
    speaker: String,
    text: String,
    commands: Vec<Command>,
    metadata: HashMap<String, String>,
}

struct DialogueOption {
    text: String,
    condition: Option<Condition>,
    target_node: String,
    actions: Vec<Action>,
}

// Dialogue runner (stateless)
struct DialogueRunner {
    program: DialogueProgram,
    variable_store: Box<dyn VariableStore>,
}

impl DialogueRunner {
    fn start(&mut self, node_title: &str) -> DialogueEvent {
        let node = self.program.get_node(node_title);
        self.evaluate_node(node)
    }
    
    fn select_option(&mut self, option_index: usize) -> DialogueEvent {
        // Execute option actions, jump to target node
    }
}

// Storylet system (saliency-based selection)
struct StoryletSystem {
    storylets: Vec<Storylet>,
}

impl StoryletSystem {
    fn select(&self, context: &GameContext) -> Option<&Storylet> {
        let available: Vec<_> = self.storylets.iter()
            .filter(|s| s.condition.evaluate(context))
            .collect();
        
        // Select by specificity (most conditions = most specific)
        available.into_iter()
            .max_by_key(|s| s.condition.count())
    }
}
```

### 6.8 Tilemap System Design

```rust
// Tile definition
struct TileDefinition {
    visual: TileVisual,
    physics: Option<PhysicsShape>,
    navigation: bool,
    custom_data: HashMap<String, Value>,
    terrain_rules: Vec<TerrainRule>,
}

struct TileVisual {
    sprite: Handle<TextureAtlas>,
    region: Rect,
    animations: Vec<TileAnimation>,
    rotation: Rotation,
    flip_x: bool,
    flip_y: bool,
}

// TileSet: library of tile definitions
struct TileSet {
    tiles: HashMap<TileId, TileDefinition>,
    terrain_sets: Vec<TerrainSet>,
    proxies: HashMap<TileId, TileId>,
}

// Terrain auto-tiling
struct TerrainSet {
    mode: TerrainMode, // Corners, Sides, CornersAndSides
    terrains: Vec<Terrain>,
}

struct Terrain {
    name: String,
    color: Color,
    priority: u32,
    peering_bits: HashMap<BitMask, TileVariant>,
}

// TileMap: grid of cells
struct TileMap {
    grid_size: UVec2,
    cell_size: Vec2,
    layers: Vec<TileLayer>,
}

struct TileLayer {
    name: String,
    cells: HashMap<IVec2, CellData>,
    visible: bool,
    z_index: i32,
}

struct CellData {
    tile_id: TileId,
    terrain: Option<u32>,
    transform: TileTransform,
}

// Auto-tiling algorithm
fn compute_auto_tile(tilemap: &TileMap, pos: IVec2, terrain_set: &TerrainSet) -> TileVariant {
    let neighbors = get_8_neighbors(tilemap, pos);
    let bitmask = compute_peering_bits(neighbors, terrain_set.mode);
    terrain_set.get_tile_variant(bitmask)
}
```

### 6.9 Quest System Design

```rust
// Quest definition
struct Quest {
    id: QuestId,
    name: String,
    description: String,
    objectives: Vec<Objective>,
    prerequisites: Vec<QuestId>,
    rewards: Vec<Reward>,
    dialogue_hooks: HashMap<String, String>, // event → dialogue node
}

struct Objective {
    id: ObjectiveId,
    objective_type: ObjectiveType,
    target: Target,
    required_count: u32,
    current_count: u32,
    state: ObjectiveState,
}

enum ObjectiveType {
    Kill { entity_type: TagPath },
    Collect { item_id: ItemId },
    Talk { npc_id: EntityId },
    Reach { area: Area },
    Custom { handler: Box<dyn ObjectiveHandler> },
}

// Quest state machine
enum QuestState {
    Unavailable,
    Available,
    Active,
    Completed,
    Failed,
}

// Quest manager (ECS system)
fn quest_progression_system(
    mut quest_events: EventReader<QuestEvent>,
    mut quests: ResMut<QuestManager>,
    objectives: Query<&ObjectiveTracker>,
) {
    for event in quest_events.read() {
        match event {
            QuestEvent::ObjectiveProgress { quest_id, objective_id, count } => {
                quests.update_progress(*quest_id, *objective_id, *count);
            }
            QuestEvent::Complete { quest_id } => {
                quests.complete_quest(*quest_id);
            }
        }
    }
}
```

### 6.10 Complete System Wiring

```
App::new()
    // Foundation
    .add_plugin(EcsPlugin)
    .add_plugin(EventBusPlugin)
    .add_plugin(TagSystemPlugin)
    .add_plugin(PluginManagerPlugin)
    
    // Subsystems
    .add_plugin(RenderingPlugin)
    .add_plugin(PhysicsPlugin)
    .add_plugin(AudioPlugin)
    .add_plugin(InputPlugin)
    .add_plugin(TilemapPlugin)
    
    // Frameworks
    .add_plugin(StateMachinePlugin)
    .add_plugin(DialoguePlugin)
    .add_plugin(QuestPlugin)
    .add_plugin(InventoryPlugin)
    .add_plugin(AiPlugin)
    
    // Game Logic (project-specific)
    .add_plugin(GameplayPlugin)
    
    .run();
```

---

## 7. Implementation Recommendations

### 7.1 Priority Order

| Phase | Components | Rationale |
|-------|-----------|-----------|
| **P0: Foundation** | ECS core, Event bus, Plugin system | Everything depends on these |
| **P1: Core Systems** | Resource loading, Rendering, Input | Need to see something on screen |
| **P2: Game Systems** | Tilemap, State machine, Physics | Enable gameplay |
| **P3: Content Systems** | Dialogue, Quests, Inventory | Enable content |
| **P4: Advanced** | AI/BT, Networking, Modding | Polish and scale |

### 7.2 Key Architectural Decisions

| Decision | Recommendation | Source Engines |
|----------|---------------|----------------|
| Entity Storage | Archetype-based (SoA per chunk) | Unity DOTS, Bevy |
| Component Design | Plain data structs, no inheritance | All 4 engines |
| System Execution | Parallel scheduling with dependency tracking | Unity DOTS, Bevy |
| Event System | Typed channels with auto-cleanup | Godot signals, Bevy events |
| State Management | Global state enum + per-entity FSM | Bevy States, Godot nodes |
| Tag System | Hierarchical tags with prefix matching | Unreal Gameplay Tags |
| Tilemap | Triple-ID (source/atlas/alternative) + terrain auto-tiling | Godot TileSet |
| Dialogue | Node-based with storylets (saliency selection) | Yarn Spinner 3.0 |
| Quests | Tag-based state machine with GAS-style effects | Unreal GAS |
| Configuration | Serde-based, multiple formats (JSON/TOML/RON) | Bevy, Godot Resources |
| Modding | Plugin system + scripted behavior (WASM/Lua) | Godot GDExtension |

### 7.3 Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Entity count | 100K+ simultaneous | Unity DOTS Megacity demo |
| Archetype query | <1μs per 10K entities | Academic benchmarks (2026) |
| Event throughput | 1M events/sec | Bevy event system |
| Tilemap render | 60 FPS with 100K tiles | Godot TileMapLayer |
| Memory per entity | <32 bytes (ID + generation) | Bevy ECS |

### 7.4 Anti-Patterns to Avoid

| Anti-Pattern | Why | Better Approach |
|-------------|-----|-----------------|
| Deep inheritance trees | Fragile base class, tight coupling | ECS components |
| God objects (huge MonoBehaviours) | Hard to test, hard to parallelize | Small focused components |
| String-based keys for queries | Slow, error-prone | TypeId-based lookups |
| Manual event cleanup | Memory leaks, stale references | Auto-cleanup channels |
| Hidden state in singletons | Debugging nightmare | Explicit Resources |
| Per-entity FSM nodes | Memory overhead for large populations | Tag-based state |
| Hardcoded tile rules | Inflexible, hard to extend | Terrain system with priorities |
| Tightly coupled dialogue + quest | Can't reuse either | Event-driven interface |

---

## Appendix A: Source References

| Engine | Version | Key Source |
|--------|---------|-----------|
| Unity DOTS | Entities 1.3+ (2025) | docs.unity3d.com/Packages/com.unity.entities |
| Unity Tilemap | 2D Extras 8.88+ | docs.unity3d.com/Packages/com.unity.2d.tilemap.extras |
| Yarn Spinner | 3.0 | yarnspinner.dev/docs |
| Unreal | 5.8 (2026) | dev.epicgames.com/documentation/unreal-engine |
| Unreal GAS | 5.8 | GameplayAbilities plugin documentation |
| Godot | 4.4+ (2026) | docs.godotengine.org |
| Godot TileSet | 4.4+ | docs.godotengine.org/en/4.4/classes/class_tileset |
| DialogueManager | v4 (2026) | github.com/nathanhoad/godot_dialogue_manager |
| Bevy | 0.15+ (2026) | docs.rs/bevy |
| bevy_ecs_tilemap | 0.15+ | docs.rs/bevy_ecs_tilemap |
| Academic ECS | Tasnim & Zhao 2026 | arXiv:2606.14919 |

## Appendix B: Cross-Engine Pattern Matrix

| Pattern | Unity | Unreal | Godot | Bevy | Universal Name |
|---------|-------|--------|-------|------|---------------|
| ECS | DOTS Entities | (not native) | (not native) | bevy_ecs | **Entity-Component-System** |
| Event System | C# events | Delegates | Signals | EventReader/Writer | **Pub/Sub Events** |
| FSM | Animator | GAS Tags | Node-based | States trait | **State Machine** |
| BT | (plugin) | BehaviorTree | (plugin) | (crate) | **Behavior Tree** |
| Tilemap | Tilemap + RuleTile | Paper2D | TileMap + TileSet | bevy_ecs_tilemap | **Grid Tilemap** |
| Dialogue | Yarn Spinner | (custom) | DialogueManager | (custom) | **Node Dialogue** |
| Quest | (custom) | GAS Effects | (custom) | (custom) | **Tag Quest** |
| Data | ScriptableObject | DataTable | Resource | serde Config | **Data-Driven** |
| Plugin | Assembly Def | Module/Plugin | GDExtension | Plugin trait | **Modular Plugin** |
| GC | MonoBehaviour GC | UObject GC | RefCounted | Ownership | **Memory Management** |
| Reflection | System.Type | UHT/UClass | GDExtension | bevy_reflect | **Runtime Introspection** |
| Visual Scripting | Visual Scripting | Blueprint | (community) | (community) | **Node Programming** |
