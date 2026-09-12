# NeoTrix Universal Game Engine Architecture

## 设计理念

基于外部游戏引擎架构模式（Unity ECS、Godot Node、Unreal Actor-Component），熔炼为通用抽象层，实现：

1. **聚焦冗余** — 识别并清理重复实现
2. **扁平缺陷** — 补齐缺失的通用能力
3. **跨域错位** — 统一不同领域的抽象层次

## 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    Game Layer (游戏层)                       │
│  Stardew Valley · Civilization · Tower Defense · RPG        │
├─────────────────────────────────────────────────────────────┤
│                  Mechanics Layer (机制层)                    │
│  Consciousness · Evolution · Knowledge · Social · Economy   │
├─────────────────────────────────────────────────────────────┤
│                  Engine Layer (引擎层)                       │
│  ECS · Rendering · Physics · Input · Audio · Scene Graph    │
├─────────────────────────────────────────────────────────────┤
│                  Platform Layer (平台层)                     │
│  Unity · Godot · Unreal · Web · Terminal                    │
└─────────────────────────────────────────────────────────────┘
```

## 核心抽象

### 1. Entity-Component-System (ECS)

```rust
// Entity: 身份标识 (generational arena)
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

// Component: 数据容器 (trait object)
pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// System: 逻辑处理器
pub trait System {
    fn update(&mut self, world: &mut World, dt: f32);
    fn priority(&self) -> i32 { 0 }
}

// World: 容器 + 调度器
pub struct World {
    entities: Vec<Option<Entity>>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    systems: Vec<Box<dyn System>>,
    resources: HashMap<TypeId, Box<dyn Resource>>,
}
```

### 2. Rendering Abstraction

```rust
pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform);
    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera);
    fn present(&mut self);
}

pub struct Sprite {
    pub texture: Handle<Texture>,
    pub rect: Rect,
    pub color: Color,
    pub z_index: i32,
}

pub struct TileMap {
    pub tiles: Vec<Vec<TileId>>,
    pub tile_size: Vec2,
    pub palette: Vec<TileDef>,
}
```

### 3. Physics Abstraction

```rust
pub trait PhysicsWorld {
    fn add_body(&mut self, entity: Entity, body: RigidBody);
    fn add_collider(&mut self, entity: Entity, collider: Collider);
    fn step(&mut self, dt: f32);
    fn query_point(&self, point: Vec2) -> Vec<Entity>;
    fn query_rect(&self, rect: Rect) -> Vec<Entity>;
}

pub struct RigidBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub body_type: BodyType, // Static, Dynamic, Kinematic
}

pub enum Collider {
    AABB { half_extents: Vec2 },
    Circle { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}
```

### 4. Input Abstraction

```rust
pub trait InputProvider {
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_key_just_pressed(&self, key: KeyCode) -> bool;
    fn get_mouse_position(&self) -> Vec2;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn get_gamepad_axis(&self, gamepad: usize, axis: GamepadAxis) -> f32;
}

pub enum KeyCode {
    W, A, S, D, Space, Enter, Escape,
    Num1, Num2, Num3, Num4, Num5,
    // ...
}
```

### 5. Scene Graph

```rust
pub struct SceneNode {
    pub entity: Entity,
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
    pub local_transform: Transform,
    pub world_transform: Transform,
}

pub struct SceneGraph {
    nodes: HashMap<Entity, SceneNode>,
    root: Entity,
}
```

### 6. Event System

```rust
pub trait Event: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

pub struct EventBus {
    queues: HashMap<TypeId, VecDeque<Box<dyn Event>>>,
    handlers: HashMap<TypeId, Vec<Box<dyn EventHandler>>>,
}

pub trait EventHandler {
    fn handle(&mut self, event: &dyn Event, world: &mut World);
}
```

## Engine Adapters

### Unity Adapter
```rust
pub struct UnityAdapter {
    // Maps NeoTrix ECS to Unity GameObjects
    // Maps NeoTrix Components to Unity MonoBehaviour
    // Maps NeoTrix Systems to Unity Update/LateUpdate
}
```

### Godot Adapter
```rust
pub struct GodotAdapter {
    // Maps NeoTrix ECS to Godot Node tree
    // Maps NeoTrix Components to Godot Node properties
    // Maps NeoTrix Systems to Godot _process/_physics_process
}
```

### Unreal Adapter
```rust
pub struct UnrealAdapter {
    // Maps NeoTrix ECS to Unreal Actors
    // Maps NeoTrix Components to Unreal ActorComponents
    // Maps NeoTrix Systems to Unreal Tick Functions
}
```

## NeoTrix Integration

### Existing Capabilities to Preserve
1. **Consciousness Metrics** — Phi, coherence, emergence
2. **Evolution Pipeline** — SEAL, fitness, mutation, speciation
3. **Knowledge System** — KB integration, graph memory
4. **Social Dynamics** — Relationships, economy, culture, governance
5. **AI Behaviors** — GOAP, behavior trees, emotion engine

### New Capabilities to Add
1. **Universal ECS** — Component composition, system scheduling
2. **Rendering Abstraction** — Sprite, tilemap, camera effects
3. **Physics Abstraction** — Rigid bodies, colliders, query
4. **Input Abstraction** — Keyboard, mouse, gamepad
5. **Scene Graph** — Hierarchical transforms, parent-child
6. **Event System** — Typed events, handlers, ordering

## Redundancy Cleanup

### Identified Redundancies
1. **Two Renderers** — `world_sim/renderer.rs` vs `ui/renderer.rs`
2. **Two Vec3** — `physics::Vec3` vs `foundation::math_bridge::Vec3`
3. **Two Bridges** — `bridge/` vs `core_bridge/`

### Cleanup Actions
1. Merge renderers into single `RendererTrait`
2. Unify math types into `foundation::math`
3. Consolidate bridge modules

## Cross-Domain Misalignment

### Issues
1. **Physics ≠ WorldSim** — Different physics models
2. **Agent owns everything** — No component separation
3. **Event bus unused by physics** — Disconnected systems

### Fixes
1. Create unified `PhysicsWorld` trait
2. Implement component-based `Agent` entity
3. Wire physics events to event bus

## Architecture Refactoring

### Before (Current)
```
neotrix-sim/
├── agents/          # Monolithic SimAgent
├── world_sim/       # WorldSim owns everything
├── physics/         # Robot-only physics
├── ui/              # Separate renderer
└── bridge/          # Sim↔Core bridge
```

### After (Refactored)
```
neotrix-sim/
├── ecs/             # Universal ECS
│   ├── entity.rs
│   ├── component.rs
│   ├── system.rs
│   └── world.rs
├── engine/          # Engine abstractions
│   ├── renderer.rs
│   ├── physics.rs
│   ├── input.rs
│   ├── audio.rs
│   └── scene.rs
├── mechanics/       # Game mechanics
│   ├── consciousness.rs
│   ├── evolution.rs
│   ├── knowledge.rs
│   ├── social.rs
│   └── economy.rs
├── adapters/        # Engine adapters
│   ├── unity.rs
│   ├── godot.rs
│   └── unreal.rs
└── platform/        # Platform layer
    ├── web.rs
    ├── desktop.rs
    └── terminal.rs
```

## Rapid Game Restoration

### Workflow
1. **Define Game** — Choose game type (Stardew, Civilization, etc.)
2. **Select Components** — Pick required ECS components
3. **Configure Systems** — Enable/disable engine systems
4. **Apply Adapter** — Map to target engine (Unity/Godot/Unreal)
5. **Generate Game** — Auto-generate game code

### Example: Stardew Valley Clone
```rust
// Components
let components = vec![
    Transform::default(),
    Sprite::new("player.png"),
    Farmer { farming_skill: 0 },
    Inventory::new(20),
    MaslowNeeds::default(),
];

// Systems
let systems = vec![
    MovementSystem,
    FarmingSystem,
    InventorySystem,
    MaslowSystem,
    RenderingSystem,
];

// Generate game
GameBuilder::new()
    .with_components(components)
    .with_systems(systems)
    .with_adapter(StardewAdapter)
    .build();
```

## Implementation Plan

### Phase 1: Core ECS (Week 1)
- [ ] Implement Entity, Component, System traits
- [ ] Create World container with component storage
- [ ] Add system scheduling and prioritization

### Phase 2: Engine Abstractions (Week 2)
- [ ] Implement Renderer trait with sprite/tilemap
- [ ] Implement PhysicsWorld trait with rigid bodies
- [ ] Implement InputProvider trait with keyboard/mouse

### Phase 3: Scene & Events (Week 3)
- [ ] Implement SceneGraph with hierarchical transforms
- [ ] Implement EventBus with typed events
- [ ] Wire existing systems to new architecture

### Phase 4: Engine Adapters (Week 4)
- [ ] Implement UnityAdapter for MonoBehaviour mapping
- [ ] Implement GodotAdapter for Node mapping
- [ ] Implement UnrealAdapter for Actor mapping

### Phase 5: Game Examples (Week 5)
- [ ] Stardew Valley clone using new architecture
- [ ] Civilization clone using new architecture
- [ ] Tower Defense clone using new architecture

## Success Metrics

1. **Code Reduction** — 30% less code through redundancy cleanup
2. **Engine Coverage** — Support Unity, Godot, Unreal, Web
3. **Game Types** — Support 5+ game genres
4. **Performance** — No regression from current benchmarks
5. **Maintainability** — Clear separation of concerns

## Conclusion

This architecture enables NeoTrix to:
1. **Rapidly prototype** games across multiple engines
2. **Reuse existing** consciousness, evolution, and social systems
3. **Clean up** redundant code and misaligned abstractions
4. **Scale** to support new game genres and engines

The key insight is that game engines share common patterns (ECS, rendering, physics, input) that can be abstracted, while NeoTrix's unique capabilities (consciousness, evolution, knowledge) can be layered on top as game mechanics.
