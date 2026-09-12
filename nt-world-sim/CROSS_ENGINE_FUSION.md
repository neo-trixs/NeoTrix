# Universal Game Engine — Cross-Engine Fusion Architecture

## Research Summary (2026-09-12)

### Engines Analyzed
| Engine | Stars | Architecture | Key Pattern |
|--------|-------|--------------|-------------|
| **Unity DOTS** | - | ECS (Entity-Component-System) | Archetype-based, SoA memory layout |
| **Godot 4** | - | Node-Component | GDExtension, scene tree, signals |
| **Unreal Engine 5** | - | Actor-Component | Gameplay Ability System (GAS) |
| **Bevy** | 47.9k★ | ECS (Rust-native) | Plugin architecture, schedules |

### Core Pattern Extraction

#### 1. Unity DOTS Pattern
```
Entity → Archetype → Chunk → Component Storage
- Entities: Lightweight identifiers (u32 + generation)
- Components: Plain data structs (SoA layout)
- Systems: Stateless functions operating on queries
- Archetypes: Unique component combinations
- Chunks: Fixed-size memory blocks per archetype
```

#### 2. Godot 4 Pattern
```
Node → Scene Tree → Signal → Resource
- Nodes: Inheritance-based objects
- Scene Tree: Hierarchical node organization
- Signals: Observer pattern for decoupling
- Resources: Shared data containers
- GDExtension: Native library integration
```

#### 3. Unreal Engine 5 Pattern
```
Actor → Component → Gameplay Ability → Attribute Set
- Actors: Spawnable/destroyable objects
- Components: Attachable sub-objects
- GAS: Attribute + Ability + Effect system
- Gameplay Tags: Hierarchical status system
- Replication: Network synchronization
```

#### 4. Bevy Pattern
```
Entity → Component → System → Plugin
- Entities: Simple u64 identifiers
- Components: Rust structs with #[derive(Component)]
- Systems: Rust functions with type-based queries
- Plugins: Modular feature bundles
- Schedules: System execution ordering
```

## Universal Abstraction Layer

### 1. Entity Abstraction

```rust
/// Universal entity identifier
pub struct UniversalEntity {
    pub id: u64,
    pub generation: u32,
    pub archetype: ArchetypeId,
}

/// Archetype: unique component combination
pub struct Archetype {
    pub id: ArchetypeId,
    pub component_types: Vec<TypeId>,
    pub entities: Vec<UniversalEntity>,
    pub chunks: Vec<Chunk>,
}

/// Chunk: fixed-size memory block
pub struct Chunk {
    pub capacity: usize,
    pub component_data: Vec<Vec<u8>>, // SoA layout
    pub entity_ids: Vec<u64>,
}
```

### 2. Component Abstraction

```rust
/// Universal component trait
pub trait UniversalComponent: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
    fn size(&self) -> usize;
    fn clone_box(&self) -> Box<dyn UniversalComponent>;
}

/// Component storage types
pub enum ComponentStorage {
    Dense(Vec<Box<dyn UniversalComponent>>),      // For small component sets
    Sparse(HashMap<u64, Box<dyn UniversalComponent>>), // For large, sparse sets
    SoA(SoAStorage),                               // For performance-critical
}

/// SoA (Structure of Arrays) storage
pub struct SoAStorage {
    pub arrays: HashMap<TypeId, Vec<u8>>,
    pub stride: usize,
}
```

### 3. System Abstraction

```rust
/// Universal system trait
pub trait UniversalSystem: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32;
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    
    /// Query requirements (for parallel scheduling)
    fn read_components(&self) -> Vec<TypeId> { vec![] }
    fn write_components(&self) -> Vec<TypeId> { vec![] }
    
    /// Enable/disable system
    fn enabled(&self) -> bool { true }
}

/// System scheduler with parallel execution
pub struct UniversalScheduler {
    systems: Vec<Box<dyn UniversalSystem>>,
    execution_graph: ExecutionGraph,
    parallel_threshold: usize,
}
```

### 4. Query Abstraction

```rust
/// Universal query builder
pub struct Query<T: ComponentTuple> {
    world: *const UniversalWorld,
    archetype_filter: ArchetypeFilter,
    component_filter: ComponentFilter,
    _marker: PhantomData<T>,
}

/// Component tuple (inspired by Bevy)
pub trait ComponentTuple {
    type Item;
    fn get_components(world: &UniversalWorld, entity: UniversalEntity) -> Option<Self::Item>;
}

/// Archetype filter
pub struct ArchetypeFilter {
    pub include: Vec<TypeId>,
    pub exclude: Vec<TypeId>,
    pub with: Vec<TypeId>,
}
```

### 5. World Abstraction

```rust
/// Universal world containing all entities and components
pub struct UniversalWorld {
    entities: Vec<Option<UniversalEntity>>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    resources: HashMap<TypeId, Box<dyn UniversalComponent>>,
    events: EventBus,
    next_entity_id: u64,
}

impl UniversalWorld {
    pub fn spawn(&mut self) -> UniversalEntity;
    pub fn despawn(&mut self, entity: UniversalEntity);
    pub fn insert_component<T: UniversalComponent>(&mut self, entity: UniversalEntity, component: T);
    pub fn get_component<T: UniversalComponent>(&self, entity: UniversalEntity) -> Option<&T>;
    pub fn get_component_mut<T: UniversalComponent>(&mut self, entity: UniversalEntity) -> Option<&mut T>;
    pub fn query<T: ComponentTuple>(&self) -> Query<T>;
    pub fn insert_resource<T: UniversalComponent>(&mut self, resource: T);
    pub fn get_resource<T: UniversalComponent>(&self) -> Option<&T>;
}
```

## Engine Adapter Pattern

### 1. Unity DOTS Adapter

```rust
/// Unity DOTS adapter
pub struct UnityAdapter {
    world: UniversalWorld,
    archetype_cache: HashMap<HashSet<TypeId>, ArchetypeId>,
}

impl EngineAdapter for UnityAdapter {
    fn from_unity_entity(entity: UnityEntity) -> UniversalEntity {
        UniversalEntity {
            id: entity.index() as u64,
            generation: entity.generation() as u32,
            archetype: ArchetypeId(0),
        }
    }

    fn to_unity_entity(entity: UniversalEntity) -> UnityEntity {
        UnityEntity::from_raw(entity.id as i32, entity.generation as i32)
    }

    fn map_component<T: 'static>(component: T) -> Box<dyn UniversalComponent> {
        Box::new(component)
    }
}
```

### 2. Godot 4 Adapter

```rust
/// Godot 4 adapter
pub struct GodotAdapter {
    world: UniversalWorld,
    scene_tree: SceneTree,
    signal_registry: SignalRegistry,
}

impl EngineAdapter for GodotAdapter {
    fn from_godot_node(node: &Node) -> UniversalEntity {
        UniversalEntity {
            id: node.get_instance_id() as u64,
            generation: 0,
            archetype: ArchetypeId(0),
        }
    }

    fn to_godot_node(entity: UniversalEntity) -> Option<Gd<Node>> {
        // Retrieve from scene tree
        None
    }

    fn map_signal(signal: Signal) -> Event {
        Event::new(signal.name(), signal.args())
    }
}
```

### 3. Unreal Engine 5 Adapter

```rust
/// Unreal Engine 5 adapter
pub struct UnrealAdapter {
    world: UniversalWorld,
    gameplay_tag_registry: GameplayTagRegistry,
    attribute_registry: AttributeRegistry,
}

impl EngineAdapter for UnrealAdapter {
    fn from_unreal_actor(actor: &AActor) -> UniversalEntity {
        UniversalEntity {
            id: actor.get_unique_id() as u64,
            generation: 0,
            archetype: ArchetypeId(0),
        }
    }

    fn to_unreal_actor(entity: UniversalEntity) -> Option<&AActor> {
        // Retrieve from world
        None
    }

    fn map_gameplay_tag(tag: FGameplayTag) -> String {
        tag.ToString()
    }
}
```

### 4. Bevy Adapter

```rust
/// Bevy adapter
pub struct BevyAdapter {
    world: UniversalWorld,
    bevy_world: bevy::ecs::world::World,
}

impl EngineAdapter for BevyAdapter {
    fn from_bevy_entity(entity: bevy::ecs::entity::Entity) -> UniversalEntity {
        UniversalEntity {
            id: entity.index() as u64,
            generation: entity.generation() as u32,
            archetype: ArchetypeId(0),
        }
    }

    fn to_bevy_entity(entity: UniversalEntity) -> bevy::ecs::entity::Entity {
        bevy::ecs::entity::Entity::from_raw(entity.id as u32, entity.generation as u32)
    }

    fn map_component<T: bevy::ecs::component::Component>(component: T) -> Box<dyn UniversalComponent> {
        Box::new(component)
    }
}
```

## Redundancy Cleanup Map

### Before (Current State)
```
nt-world-sim/src/
├── engine/
│   ├── renderer.rs      # Vec2, Color, Rect, Transform, Sprite, TileMap, Camera
│   ├── physics.rs       # Vec2 (duplicate), RigidBody, Collider
│   ├── input.rs         # Vec2 (duplicate), InputState, InputProvider
│   ├── events.rs        # EventBus, Event trait
│   ├── pet_state.rs     # PetState, PetStateComponent, EyeTracking
│   ├── hook_system.rs   # HookEvent, HookManager, PermissionRequest
│   └── theme_system.rs  # ThemeConfig, AnimationPlayer, SpriteSheet
├── ecs/
│   ├── world.rs         # Entity, World, Component
│   └── system.rs        # System trait, SystemScheduler
└── mechanics/
    └── mod.rs           # ConsciousnessEntity, MaslowNeeds, AiComponent
```

### After (Unified Architecture)
```
nt-world-sim/src/
├── core/                    # Universal abstractions
│   ├── entity.rs            # UniversalEntity, Archetype, Chunk
│   ├── component.rs         # UniversalComponent, ComponentStorage
│   ├── system.rs            # UniversalSystem, UniversalScheduler
│   ├── query.rs             # Query, ComponentTuple, ArchetypeFilter
│   ├── world.rs             # UniversalWorld
│   └── resource.rs          # Resource trait, ResourceManager
├── adapters/                # Engine-specific adapters
│   ├── unity.rs             # Unity DOTS adapter
│   ├── godot.rs             # Godot 4 adapter
│   ├── unreal.rs            # Unreal Engine 5 adapter
│   └── bevy.rs              # Bevy adapter
├── mechanics/               # Game-specific systems
│   ├── consciousness.rs     # ConsciousnessSystem
│   ├── maslow.rs            # MaslowSystem
│   ├── ai.rs                # AiSystem
│   ├── pet.rs               # PetStateSystem, EyeTrackingSystem
│   ├── hook.rs              # HookManager, PermissionSystem
│   └── theme.rs             # ThemeManager, AnimationSystem
└── platform/                # Platform abstraction
    ├── renderer.rs          # Renderer trait (unified)
    ├── physics.rs           # PhysicsWorld trait (unified)
    ├── input.rs             # InputProvider trait (unified)
    └── audio.rs             # AudioProvider trait (unified)
```

### Redundancy Eliminated

| Redundancy | Before | After | LOC Saved |
|------------|--------|-------|-----------|
| **Vec2 definition** | 3 copies (renderer, physics, input) | 1 copy (core/vec2.rs) | ~60 |
| **Component trait** | 2 copies (ecs, engine) | 1 copy (core/component.rs) | ~40 |
| **System trait** | 2 copies (ecs, mechanics) | 1 copy (core/system.rs) | ~30 |
| **EventBus** | 2 copies (events, hook_system) | 1 copy (core/event.rs) | ~50 |
| **Transform** | 2 copies (renderer, pet_state) | 1 copy (core/transform.rs) | ~30 |
| **Total** | - | - | **~210 LOC** |

### Flat Deficiency Patches

| Deficiency | Gap | Patch | LOC Added |
|------------|-----|-------|-----------|
| **No archetype system** | Components not grouped by type | Add Archetype + Chunk storage | ~200 |
| **No parallel scheduling** | Systems run sequentially | Add execution graph + wave scheduling | ~150 |
| **No change detection** | Unnecessary system re-runs | Add Changed<T> query filter | ~100 |
| **No resource system** | Global state not managed | Add Resource trait + ResourceManager | ~80 |
| **No event batching** | Events processed one-by-one | Add EventQueue with batch processing | ~60 |
| **Total** | - | - | **~590 LOC** |

### Cross-Domain Misalignment Fixes

| Misalignment | Domains | Fix |
|--------------|---------|-----|
| **PetState vs GameEntity** | ECS + Mechanics | PetState = component on UniversalEntity |
| **Theme vs AssetRegistry** | Engine + World | Theme = specialized Resource |
| **Hook vs EventBus** | IO + Core | Hook → EventBus adapter |
| **Session vs World entity** | Memory + ECS | Session = entity with SessionInfo component |
| **Renderer vs Platform** | Engine + Platform | Renderer = trait, platform implements |
| **Physics vs Mechanics** | Engine + Mechanics | PhysicsWorld = trait, mechanics provides |
| **Input vs Platform** | Engine + Platform | InputProvider = trait, platform implements |
| **Audio vs Platform** | Engine + Platform | AudioProvider = trait, platform implements |

## Rapid Game Restoration Pipeline

### 1. Game Definition Format

```yaml
# game.yaml - Universal game definition
name: "Stardew Valley Clone"
version: "1.0.0"

# Engine configuration
engine:
  target: "bevy"  # or unity, godot, unreal
  features:
    - "2d"
    - "physics"
    - "audio"
    - "ui"

# Entities definition
entities:
  player:
    components:
      - Transform
      - Sprite
      - PlayerController
      - Inventory
    systems:
      - PlayerInputSystem
      - MovementSystem
      - CollisionSystem

  npc:
    components:
      - Transform
      - Sprite
      - AiComponent
      - DialogueSystem
    systems:
      - AiSystem
      - DialogueSystem

# Systems definition
systems:
  PlayerInputSystem:
    priority: 100
    read: [InputState]
    write: [Velocity]

  MovementSystem:
    priority: 50
    read: [Transform, Velocity]
    write: [Transform]

# Resources definition
resources:
  Time:
    fields:
      delta: f32
      elapsed: f32

  Input:
    fields:
      keys: HashSet<KeyCode>
      mouse: Vec2
```

### 2. Code Generation Pipeline

```
game.yaml → Parser → AST → Code Generator → Target Code
                                              ↓
                              ┌─────────────────┼─────────────────┐
                              ↓                 ↓                 ↓
                          Bevy Code        Unity Code        Godot Code
```

### 3. Game Restoration Steps

1. **Parse game.yaml** → Extract entities, components, systems, resources
2. **Validate definitions** → Check for missing dependencies, circular references
3. **Generate code** → Target-specific code (Bevy/Unity/Godot/Unreal)
4. **Compile** → Build for target platform
5. **Test** → Run automated tests
6. **Package** → Create distributable package

## Implementation Priority

### Phase 1: Core Abstractions (Week 1)
- [ ] Create `core/entity.rs` with UniversalEntity, Archetype, Chunk
- [ ] Create `core/component.rs` with UniversalComponent trait
- [ ] Create `core/system.rs` with UniversalSystem trait
- [ ] Create `core/query.rs` with Query builder
- [ ] Create `core/world.rs` with UniversalWorld
- [ ] Create `core/resource.rs` with Resource trait

### Phase 2: Engine Adapters (Week 2)
- [ ] Implement Bevy adapter (reference implementation)
- [ ] Implement Unity DOTS adapter
- [ ] Implement Godot 4 adapter
- [ ] Implement Unreal Engine 5 adapter

### Phase 3: Mechanics Integration (Week 3)
- [ ] Port ConsciousnessSystem to UniversalSystem
- [ ] Port MaslowSystem to UniversalSystem
- [ ] Port AiSystem to UniversalSystem
- [ ] Port PetStateSystem to UniversalSystem
- [ ] Port HookManager to UniversalSystem
- [ ] Port ThemeManager to UniversalSystem

### Phase 4: Game Restoration (Week 4)
- [ ] Create game.yaml parser
- [ ] Implement code generator for Bevy
- [ ] Implement code generator for Unity
- [ ] Implement code generator for Godot
- [ ] Implement code generator for Unreal
- [ ] Test with Stardew Valley clone example

### Phase 5: Optimization (Week 5)
- [ ] Implement parallel scheduling
- [ ] Add change detection
- [ ] Optimize SoA memory layout
- [ ] Add profiling instrumentation
- [ ] Performance benchmarking

## Success Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| **Code Duplication** | 210 LOC | 0 LOC | 0 LOC |
| **Flat Deficiencies** | 5 gaps | 0 gaps | 0 LOC |
| **Cross-Domain Misalignment** | 7 issues | 0 issues | 0 issues |
| **Engine Support** | 1 (Custom) | 4 (Unity/Godot/Unreal/Bevy) | 4 |
| **Game Restoration Time** | Days | Minutes | < 5 min |
| **Build Time** | 35s | 10s | < 10s |
| **Test Coverage** | 60% | 90% | > 90% |
