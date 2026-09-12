# Universal Game Engine Abstraction Layer

> **Design Document v1.0** — Engine-agnostic game framework for nt-world-sim.
> Maps to Bevy, Unity, Godot, Unreal, or any ECS-based engine via adapter pattern.

---

## 1. Core Abstractions (Engine-Agnostic)

All core types live in `src/core/`. They are engine-agnostic — no dependency on any specific engine.

### 1.1 Entity

**Existing**: `UniversalEntity` in `src/core/entity.rs:14`

```rust
pub struct UniversalEntity {
    pub id: EntityId,        // u64 — monotonic, never reused
    pub generation: u32,     // generational index for safety
    pub archetype: ArchetypeId,
}
```

**Design principles**:
- Entity IDs are **monotonic u64** — never reused, safe to cache externally
- Generation counter detects stale references (use-after-despawn)
- Archetype is metadata — engines that don't use archetypes ignore it
- Cross-engine mapping: `EntityId` → Unity `GameObject.GetInstanceID()`, Godot `ObjectID`, UE5 `FActorInstanceHandle`

### 1.2 Component

**Existing**: `Component` trait in `src/core/world.rs:6`

```rust
pub trait Component: Send + Sync + Clone + 'static {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
}
```

**Design principles**:
- Components are plain data — no methods, no logic
- `Clone + Send + Sync` — thread-safe, copyable
- Type-erased storage via `Box<dyn Any>` with `TypeId` key
- Three storage backends: Dense (Vec), Sparse (HashMap), SoA (Structure of Arrays)

**Standard component library** (to implement):

| Component | Fields | Source |
|-----------|--------|--------|
| `Transform` | position: Vec2, rotation: f32, scale: Vec2 | `renderer.rs:102` |
| `Sprite` | texture: Option<String>, rect: Rect, color: Color, z_index: i32 | `renderer.rs:120` |
| `RigidBody` | position, velocity, mass, body_type, gravity_scale, friction, restitution | `physics.rs:22` |
| `Collider` | AABB / Circle / Polygon | `physics.rs:62` |
| `Name` | name: String | — |
| `Parent` | parent: EntityId | scene.rs hierarchy |

### 1.3 System

**Existing**: `UniversalSystem` trait in `src/core/scheduler.rs:6`

```rust
pub trait UniversalSystem: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn read_components(&self) -> Vec<TypeId> { vec![] }
    fn write_components(&self) -> Vec<TypeId> { vec![] }
    fn enabled(&self) -> bool { true }
}
```

**Design principles**:
- Systems declare read/write sets → scheduler parallelizes non-conflicting waves
- Priority ordering for systems in the same wave
- `enabled()` allows runtime toggle without removal
- Existing `ParallelScheduler` already implements topological sort + wave scheduling

**System ordering conventions**:

| Phase | Systems | Priority |
|-------|---------|----------|
| 0 | Input collection | -100 |
| 1 | Physics step | 0 |
| 2 | Gameplay logic | 10 |
| 3 | UI update | 20 |
| 4 | Rendering | 100 |

### 1.4 Resource

**Existing**: `Resource` trait in `src/core/world.rs:12`

```rust
pub trait Resource: Send + Sync + 'static {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
}
```

**Standard resources**:

| Resource | Purpose | Fields |
|----------|---------|--------|
| `DeltaTime` | Frame time | delta: f32, elapsed: f64 |
| `InputState` | Current frame input | (see §3) |
| `CameraResource` | Active camera | camera: Camera |
| `AudioManager` | Sound playback | (see §4) |
| `AssetServer` | Asset loading | (see §8) |
| `SceneStack` | Scene management | (see §7) |

### 1.5 Event

**Existing**: `Event` trait in `src/core/world.rs:20` + `TypedEventBus` in `src/engine/event_bus.rs`

```rust
pub trait Event: Send + Sync + 'static { }
```

**Design principles**:
- Type-erased event bus with priority handlers
- Events are queued and processed at frame boundary
- Handler priority controls execution order
- All game events implement `GameEvent` → `Any + Send + Sync`

**Standard events**:

| Event | Payload | Purpose |
|-------|---------|---------|
| `CollisionEvent` | entity_a, entity_b, normal, penetration | Physics collision |
| `InputActionEvent` | action: String, state: InputState | Mapped input |
| `SceneTransitionEvent` | from, to, transition_type | Scene change |
| `AssetLoadedEvent` | handle, path, type | Asset ready |
| `DamageEvent` | target, amount, source | Combat |
| `UiClickEvent` | widget_id, position | UI interaction |

### 1.6 Scene

**Existing**: `SceneGraph` in `src/engine/scene.rs:47`

```rust
pub struct SceneGraph {
    nodes: HashMap<EntityId, SceneNode>,
    root: EntityId,
}
```

**SceneNode** contains:
- `entity: UniversalEntity` — linked to world
- `parent: Option<EntityId>` — hierarchy
- `children: Vec<EntityId>` — children
- `local_transform: Transform` — relative to parent
- `world_transform: Transform` — computed after `update_transforms()`
- `visible: bool` — render toggle
- `z_index: i32` — draw order

**Design principles**:
- Scene graph is a **view** over the world — entities live in `UniversalWorld`, scene graph manages hierarchy
- Transform propagation: parent → child recursive
- Z-index sorting for render order
- `visible_nodes()` returns sorted list for renderer

### 1.7 Asset

**Existing**: `AudioHandle(u32)` in `src/engine/audio.rs:5`

**To implement**:

```rust
pub struct AssetHandle<T> {
    pub id: u32,
    pub path: String,
    _marker: PhantomData<T>,
}

pub enum AssetState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
}

pub struct AssetEntry {
    pub state: AssetState,
    pub type_id: TypeId,
    pub data: Option<Box<dyn Any + Send + Sync>>,
}
```

---

## 2. Rendering Abstraction

**Existing**: `Renderer` trait + `CanvasRenderer` in `src/engine/renderer.rs:230`

### 2.1 Renderer Trait

```rust
pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform);
    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera);
    fn draw_text(&mut self, text: &str, position: Vec2, color: Color, size: f32);
    fn draw_rect(&mut self, rect: &Rect, color: Color);
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color);
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32);
    fn present(&mut self);
    fn viewport_size(&self) -> Vec2;
}
```

**Backend implementations**:

| Backend | Target | Notes |
|---------|--------|-------|
| `CanvasRenderer` | Web/browser | Records `DrawCommand` enums, consumed by JS/WASM |
| `WgpuRenderer` | Native desktop | Direct wgpu integration |
| `BevyRenderer` | Bevy engine | Delegates to Bevy's render pipeline |
| `UnityRenderer` | Unity | Generates Unity draw calls via FFI/interop |
| `GodotRenderer` | Godot | Uses Godot `CanvasItem` API |
| `UnrealRenderer` | Unreal | Uses UMG/Sprite rendering |

### 2.2 Camera

**Existing**: `Camera` in `src/engine/renderer.rs:186`

```rust
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}
```

**Extended API** (to add):

| Method | Description |
|--------|-------------|
| `follow(target: Vec2, lerp: f32)` | Smooth follow with lerp |
| `shake(intensity: f32, duration: f32)` | Screen shake |
| `world_to_screen(pos: Vec2) -> Vec2` | Coordinate conversion |
| `screen_to_world(pos: Vec2) -> Vec2` | Coordinate conversion |
| `visible_rect() -> Rect` | Visible world area |
| `zoom_to(target: f32, speed: f32)` | Smooth zoom |

### 2.3 Sprite

**Existing**: `Sprite` in `src/engine/renderer.rs:120`

```rust
pub struct Sprite {
    pub texture: Option<String>,
    pub rect: Rect,
    pub color: Color,
    pub z_index: i32,
}
```

**Extended fields** (to add):

| Field | Type | Purpose |
|-------|------|---------|
| `flip_x` | bool | Horizontal flip |
| `flip_y` | bool | Vertical flip |
| `rotation` | f32 | Sprite rotation (degrees) |
| `anchor` | Vec2 | Anchor point (0-1) |
| `region` | Option<Rect> | Source rect in atlas |

### 2.4 Tilemap

**Existing**: `TileMap` + `TileDef` in `src/engine/renderer.rs:139`

```rust
pub struct TileMap {
    pub tiles: Vec<Vec<u32>>,
    pub tile_size: Vec2,
    pub palette: HashMap<u32, TileDef>,
}
```

**Extended API** (to add):

| Method | Description |
|--------|-------------|
| `chunked_render(camera, chunk_size)` | Render only visible chunks |
| `auto_tile(x, y)` | Auto-tile neighbor detection |
| `get_neighbors(x, y) -> Vec<(i32, i32, u32)>` | Adjacent tile query |
| `is_walkable(x, y) -> bool` | Pathfinding check (existing) |

### 2.5 Particle System

**To implement**:

```rust
pub struct ParticleEmitter {
    pub position: Vec2,
    pub rate: f32,              // particles per second
    pub lifetime: (f32, f32),   // min/max lifetime
    pub speed: (f32, f32),      // min/max speed
    pub angle: (f32, f32),      // emission cone
    pub color: (Color, Color),  // start/end gradient
    pub size: (f32, f32),       // start/end size
    pub gravity: Vec2,
    pub max_particles: usize,
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub age: f32,
    pub color: Color,
    pub size: f32,
}

pub struct ParticleSystem {
    pub emitters: Vec<ParticleEmitter>,
    pub particles: Vec<Particle>,
}
```

---

## 3. Input Abstraction

**Existing**: `InputState`, `InputProvider`, `SimpleInputProvider` in `src/engine/input.rs`

### 3.1 InputMap (Action Mapping)

**To implement**:

```rust
pub struct InputMap {
    bindings: HashMap<String, Vec<InputBinding>>,
}

pub enum InputBinding {
    Key(KeyCode),
    KeyCombo(Vec<KeyCode>),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
    GamepadAxis { axis: GamepadAxis, threshold: f32, direction: AxisDirection },
    MouseAxis(MouseAxis),
}

pub enum AxisDirection { Positive, Negative }

impl InputMap {
    pub fn bind_action(action: &str, binding: InputBinding);
    pub fn unbind_action(action: &str, binding: InputBinding);
    pub fn load_from_file(path: &str) -> Result<Self, String>;
}
```

**Usage**:

```rust
let mut map = InputMap::new();
map.bind_action("move_up", InputBinding::Key(KeyCode::W));
map.bind_action("move_up", InputBinding::GamepadAxis {
    axis: GamepadAxis::LeftY, threshold: 0.5, direction: AxisDirection::Negative,
});
map.bind_action("jump", InputBinding::Key(KeyCode::Space));
map.bind_action("jump", InputBinding::GamepadButton(GamepadButton::A));
```

### 3.2 InputState (Query API)

**Existing**: `InputState` in `src/engine/input.rs:60`

Extended API:

| Method | Description |
|--------|-------------|
| `action_pressed(action: &str) -> bool` | Is action held |
| `action_just_pressed(action: &str) -> bool` | Was action just pressed |
| `action_just_released(action: &str) -> bool` | Was action just released |
| `action_axis(negative: &str, positive: &str) -> f32` | Axis value (-1..1) |
| `action_axis_2d(x: &str, y: &str) -> Vec2` | 2D axis vector |

### 3.3 Input Events

**Existing**: `KeyCode`, `MouseButton`, `GamepadAxis`, `GamepadButton` enums in `src/engine/input.rs`

Extended events:

```rust
pub enum InputEvent {
    KeyPressed { key: KeyCode },
    KeyReleased { key: KeyCode },
    MouseButtonPressed { button: MouseButton, position: Vec2 },
    MouseButtonReleased { button: MouseButton, position: Vec2 },
    MouseMoved { position: Vec2, delta: Vec2 },
    MouseScrolled { delta: f32 },
    GamepadConnected { id: usize },
    GamepadDisconnected { id: usize },
    GamepadButtonPressed { id: usize, button: GamepadButton },
    GamepadButtonReleased { id: usize, button: GamepadButton },
    GamepadAxisMoved { id: usize, axis: GamepadAxis, value: f32 },
}
```

---

## 4. Audio Abstraction

**Existing**: `AudioBackend` trait + `AudioManager` in `src/engine/audio.rs`

### 4.1 AudioSource

```rust
pub struct AudioSource {
    pub handle: AudioHandle,
    pub playing: bool,
    pub volume: f32,
    pub pan: f32,           // -1.0 left, 0.0 center, 1.0 right
    pub pitch: f32,
    pub looping: bool,
    pub position: Option<Vec2>,  // spatial audio
}
```

### 4.2 AudioBus

**Existing**: `master_volume`, `music_volume`, `sfx_volume` in `AudioManager`

Extended bus system:

```rust
pub struct AudioBus {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub children: Vec<AudioBus>,
}

impl AudioBus {
    pub fn master() -> Self;      // Bus("Master")
    pub fn music() -> Self;       // Bus("Master/Music")
    pub fn sfx() -> Self;         // Bus("Master/SFX")
    pub fn ui() -> Self;          // Bus("Master/UI")
    pub fn voice() -> Self;       // Bus("Master/Voice")
}
```

### 4.3 Audio Backend Implementations

| Backend | Target | Notes |
|---------|--------|-------|
| `StubAudioBackend` | Testing | No-op, existing |
| `RodioBackend` | Native desktop | Rust `rodio` crate |
| `WebAudioBackend` | Browser | Web Audio API via wasm-bindgen |
| `BevyAudio` | Bevy | Delegates to Bevy audio |
| `UnityAudio` | Unity | AudioSource/AudioListener |
| `GodotAudio` | Godot | AudioStreamPlayer |

---

## 5. UI Abstraction

**Existing**: `Widget`, `UiStyle`, `UiLayout` in `src/ui/widget.rs`

### 5.1 Widget Tree

```rust
pub struct WidgetNode {
    pub id: WidgetId,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub widget: Box<dyn WidgetTrait>,
    pub layout: UiLayout,
    pub style: UiStyle,
    pub visible: bool,
    pub interactive: bool,
}

pub trait WidgetTrait: Send + Sync {
    fn type_name(&self) -> &str;
    fn render(&self, renderer: &mut dyn Renderer, bounds: Rect);
    fn handle_event(&mut self, event: &UiEvent) -> bool;
}
```

### 5.2 Layout System

**Existing**: `UiLayout` enum in `src/ui/widget.rs:58`

Extended layouts:

```rust
pub enum UiLayout {
    Fixed { x: f32, y: f32, width: f32, height: f32 },
    Anchor { top: Option<f32>, bottom: Option<f32>, left: Option<f32>, right: Option<f32> },
    Center { width: f32, height: f32 },
    Flex { direction: FlexDirection, gap: f32, wrap: bool },
    Grid { columns: usize, rows: usize, gap: f32 },
}

pub enum FlexDirection {
    Row, Column, RowReverse, ColumnReverse,
}
```

### 5.3 Style Token System

**Existing**: `UiStyle` in `src/ui/widget.rs:8`

```rust
pub struct UiStyle {
    pub background: Color,
    pub border: Color,
    pub border_width: f32,
    pub text_color: Color,
    pub font_size: f32,
    pub padding: f32,
    pub corner_radius: f32,
}
```

**Extended tokens**:

| Token | Type | Purpose |
|-------|------|---------|
| `background` | Color | Fill color |
| `border` | Color | Border color |
| `border_width` | f32 | Border thickness |
| `text_color` | Color | Text color |
| `font_size` | f32 | Text size |
| `padding` | f32 | Inner spacing |
| `corner_radius` | f32 | Rounded corners |
| `margin` | f32 | Outer spacing |
| `opacity` | f32 | Transparency |
| `shadow_offset` | Vec2 | Drop shadow |
| `shadow_color` | Color | Shadow color |

**Theme presets** (existing + to add):

| Preset | Style |
|--------|-------|
| `default()` | Dark brown/gold (existing) |
| `stardew_wood()` | Wood panel style (existing) |
| `stardew_button()` | Wood button (existing) |
| `modern_dark()` | Dark gray/blue |
| `retro_green()` | CRT green-on-black |
| `minimal_white()` | Clean white |

### 5.4 UI Events

```rust
pub enum UiEvent {
    Click { widget_id: WidgetId, position: Vec2 },
    HoverEnter { widget_id: WidgetId },
    HoverExit { widget_id: WidgetId },
    Drag { widget_id: WidgetId, delta: Vec2 },
    DragStart { widget_id: WidgetId, position: Vec2 },
    DragEnd { widget_id: WidgetId, position: Vec2 },
    Scroll { widget_id: WidgetId, delta: f32 },
    Focus { widget_id: WidgetId },
    Blur { widget_id: WidgetId },
    TextInput { text: String },
    Resize { widget_id: WidgetId, size: Vec2 },
}
```

---

## 6. Physics Abstraction

**Existing**: `PhysicsWorld` trait + `SimplePhysicsWorld` in `src/engine/physics.rs`

### 6.1 Collider

**Existing**: `Collider` enum in `src/engine/physics.rs:62`

```rust
pub enum Collider {
    AABB { half_extents: Vec2 },
    Circle { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}
```

**Extended colliders**:

| Collider | Use case | Algorithm |
|----------|----------|-----------|
| `AABB` | Rectangles | `intersects()` existing |
| `Circle` | Spheres | Distance check |
| `Polygon` | Complex shapes | SAT (Separating Axis Theorem) |
| `Capsule` | Characters | AABB + Circle ends |
| `Raycast` | Projectiles | Ray-AABB/Circle/Polygon |
| `Sensor` | Triggers | No physics response |

### 6.2 RigidBody

**Existing**: `RigidBody` in `src/engine/physics.rs:22`

```rust
pub struct RigidBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub body_type: BodyType,
    pub gravity_scale: f32,
    pub friction: f32,
    pub restitution: f32,
}
```

**Extended fields**:

| Field | Type | Purpose |
|-------|------|---------|
| `linear_damping` | f32 | Velocity decay |
| `angular_velocity` | f32 | Rotation speed |
| `angular_damping` | f32 | Rotation decay |
| `max_velocity` | f32 | Speed cap |
| `is_sleeping` | bool | Optimization |
| `collision_layers` | u32 | Bitmask layers |

### 6.3 Collision Response

**Existing**: `CollisionInfo` in `src/engine/physics.rs:123`

```rust
pub struct CollisionInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub normal: Vec2,
    pub penetration: f32,
    pub contact_point: Vec2,
}
```

**Response modes**:

| Mode | Behavior | Implementation |
|------|----------|----------------|
| `Bounce` | Reflect velocity | restitution > 0 |
| `Slide` | Remove normal component | Platformer movement |
| `Trigger` | No physics, fire event | Sensor collider |
| `Fixed` | Resolve penetration only | Static objects |

### 6.4 Physics Backend Implementations

| Backend | Target | Notes |
|---------|--------|-------|
| `SimplePhysicsWorld` | Testing | Existing AABB-only |
| `RapierBackend` | Native | rapier2d crate |
| `BevyPhysics` | Bevy | Bevy Rapier plugin |
| `UnityPhysics` | Unity | Box2D / Unity Physics |
| `GodotPhysics` | Godot | GodotPhysics2D |
| `UnrealPhysics` | Unreal | Chaos Physics |

---

## 7. Scene Management

**Existing**: `SceneGraph` in `src/engine/scene.rs`

### 7.1 Scene Stack

```rust
pub struct SceneStack {
    scenes: Vec<Box<dyn Scene>>,
    transitions: Vec<SceneTransition>,
}

pub trait Scene: Send + Sync {
    fn name(&self) -> &str;
    fn on_enter(&mut self, world: &mut UniversalWorld);
    fn on_exit(&mut self, world: &mut UniversalWorld);
    fn on_pause(&mut self, world: &mut UniversalWorld);
    fn on_resume(&mut self, world: &mut UniversalWorld);
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn render(&self, renderer: &mut dyn Renderer, world: &UniversalWorld);
}
```

**Stack operations**:

| Method | Description |
|--------|-------------|
| `push(scene)` | Push new scene (pauses current) |
| `pop()` | Pop top scene (resumes previous) |
| `replace(scene)` | Pop + push in one step |
| `clear()` | Remove all scenes |
| `top()` | Get current active scene |

### 7.2 Scene Transitions

```rust
pub enum TransitionType {
    Instant,
    FadeToBlack { duration: f32 },
    FadeToColor { color: Color, duration: f32 },
    SlideLeft { duration: f32 },
    SlideRight { duration: f32 },
    SlideUp { duration: f32 },
    SlideDown { duration: f32 },
    CrossFade { duration: f32 },
    Iris { duration: f32 },
}

pub struct SceneTransition {
    pub from: String,
    pub to: String,
    pub transition: TransitionType,
    pub progress: f32,          // 0.0 → 1.0
    pub phase: TransitionPhase, // Out, Switch, In
}
```

### 7.3 Scene Lifecycle

```
push(scene)
  → current.on_pause()
  → scene.on_enter()
  → transition animates
  → scene becomes active

pop()
  → current.on_exit()
  → transition animates
  → previous.on_resume()
  → previous becomes active

replace(scene)
  → current.on_exit()
  → scene.on_enter()
  → scene becomes active (no pause/resume)
```

---

## 8. Asset Pipeline

### 8.1 AssetServer

```rust
pub struct AssetServer {
    loaded: HashMap<String, AssetEntry>,
    loading: Vec<(String, AssetType)>,
    hot_reload_enabled: bool,
    base_path: String,
}

pub enum AssetType {
    Texture,
    Sound,
    Font,
    Tilemap,
    Scene,
    Script,
    Shader,
}

impl AssetServer {
    pub fn load<T: Asset>(&mut self, path: &str) -> AssetHandle<T>;
    pub fn get<T: Asset>(&self, handle: &AssetHandle<T>) -> Option<&T>;
    pub fn get_mut<T: Asset>(&mut self, handle: &AssetHandle<T>) -> Option<&mut T>;
    pub fn watch(&mut self, path: &str);
    pub fn unwatch(&mut self, path: &str);
    pub fn tick(&mut self);  // process loading queue
}
```

### 8.2 AssetHandle

```rust
pub struct AssetHandle<T> {
    pub id: u32,
    pub path: String,
    _marker: PhantomData<T>,
}

impl<T> Clone for AssetHandle<T> { ... }
impl<T> Copy for AssetHandle<T> where T: 'static { ... }
impl<T> PartialEq for AssetHandle<T> { ... }
impl<T> Eq for AssetHandle<T> where T: 'static { ... }
```

### 8.3 AssetBundle

```rust
pub struct AssetBundle {
    pub name: String,
    pub assets: Vec<String>,
    pub priority: LoadPriority,
}

pub enum LoadPriority {
    Immediate,  // Block until loaded
    High,       // Load next frame
    Normal,     // Load when available
    Low,        // Load when idle
}
```

**Usage**:

```rust
let bundle = AssetBundle {
    name: "level_1".into(),
    assets: vec![
        "textures/tileset.png".into(),
        "sounds/bgm.ogg".into(),
        "scenes/level_1.tmx".into(),
    ],
    priority: LoadPriority::High,
};
asset_server.load_bundle(bundle);
```

---

## 9. Cross-Engine Mapping Table

### 9.1 Entity Mapping

| Abstraction | Our Engine (Rust) | Bevy (Rust) | Unity (C#) | Unreal (C++) | Godot (GDScript) |
|-------------|-------------------|-------------|------------|--------------|-------------------|
| Entity ID | `EntityId(u64)` | `Entity` | `int gameObjectId` | `AActor*` | `ObjectID` |
| Generation | `u32` | `u32` (entity.generation) | N/A (handle) | N/A | N/A |
| Archetype | `ArchetypeId(u64)` | N/A (table-based) | N/A | N/A | N/A |
| Component lookup | `TypeId` key | `ComponentId` | `GetComponent<T>()` | `FindComponentByClass()` | `get_node()` |
| Spawn | `world.spawn()` | `commands.spawn()` | `Instantiate()` | `GetWorld()->SpawnActor()` | `Node.new()` |
| Despawn | `world.despawn(e)` | `commands.entity(e).despawn()` | `Destroy(go)` | `Destroy()` | `queue_free()` |

### 9.2 Component Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| Transform | `Transform { position, rotation, scale }` | `Transform` | `Transform` | `USceneComponent` | `Node2D.position/rotation/scale` |
| Sprite | `Sprite { texture, rect, color }` | `SpriteBundle` | `SpriteRenderer` | `UPaperSpriteComponent` | `Sprite2D.texture` |
| RigidBody | `RigidBody { velocity, mass, body_type }` | `RigidBody` | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` |
| Collider | `Collider { AABB/Circle/Polygon }` | `Collider` | `Collider2D` | `UShapeComponent` | `CollisionShape2D` |
| Name | `Name(String)` | `Name` | `GameObject.name` | `GetActorName()` | `Node.name` |
| Camera | `Camera { position, zoom }` | `Camera2dBundle` | `Camera` | `UCameraComponent` | `Camera2D` |

### 9.3 System Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| System | `UniversalSystem::update()` | `fn system(world: ResMut<W>)` | `MonoBehaviour.Update()` | `Tick()` | `_process(delta)` |
| Scheduler | `ParallelScheduler` (wave) | `Schedule` (ECS) | `Update()` order | `FMath::Min` | `_process` order |
| Resource | `Resource` trait | `Resource` trait | `ScriptableObject` | `UGameInstanceSubsystem` | autoload singleton |
| Event | `TypedEventBus` | `EventWriter<T>/EventReader<T>` | `UnityEvent` | `FOnXxx delegate` | `signal` |
| Parallel | Wave scheduling | Automatic (ECS) | Manual (`[UpdateAfter]`) | TaskGraph | No native parallel |

### 9.4 Rendering Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| Renderer trait | `Renderer` trait | `RenderPlugin` | `SRP` / URP | `FSceneInterface` | `RenderingServer` |
| Camera | `Camera` | `Camera2dBundle` | `Camera` | `UCameraComponent` | `Camera2D` |
| Sprite | `Sprite` | `SpriteBundle` | `SpriteRenderer` | `UPaperSprite` | `Sprite2D` |
| Tilemap | `TileMap` | `TilemapBundle` | `Tilemap` | N/A | `TileMap` |
| Draw commands | `DrawCommand` enum | `RenderWorld` | `CommandBuffer` | `FRHICommandList` | `RenderingServer` |

### 9.5 Input Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| Key enum | `KeyCode` | `KeyCode` | `KeyCode` | `FKey` | `Key` |
| Mouse | `MouseButton` | `MouseButton` | `MouseButton` | `FKey` | `MouseButton` |
| Gamepad | `GamepadAxis/Button` | `GamepadAxis/Button` | `KeyCode.Joystick*` | `FInputKey` | `JoyAxis/JoyButton` |
| Input state | `InputState` | `Input<T>` | `Input.GetKey()` | `APlayerController` | `Input` singleton |
| Action map | `InputMap` | `InputMap` | `InputManager` | `EnhancedInput` | `InputMap` singleton |

### 9.6 Audio Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| Backend | `AudioBackend` trait | `AudioPlugin` | `AudioListener` | `FAudioDevice` | `AudioServer` |
| Source | `AudioHandle` | `AudioBundle` | `AudioSource` | `UAudioComponent` | `AudioStreamPlayer` |
| Volume | `AudioManager` | `AudioChannel` | `AudioMixer` | `USoundMix` | `AudioBus` |

### 9.7 Physics Mapping

| Abstraction | Our Engine | Bevy | Unity | Unreal | Godot |
|-------------|-----------|------|-------|--------|-------|
| Physics world | `PhysicsWorld` trait | `RapierPlugin` | `Physics2DSettings` | `FPhysScene` | `PhysicsServer2D` |
| Rigid body | `RigidBody` | `RigidBody` | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` |
| Collider | `Collider` | `Collider` | `Collider2D` | `UShapeComponent` | `CollisionShape2D` |
| Collision | `CollisionInfo` | `CollisionEvent` | `OnCollision2D` | `OnComponentHit` | `body_entered` signal |
| Gravity | `gravity: Vec2` | `Gravity` resource | `Physics2D.gravity` | `FPhysScene::SetGravity()` | `PhysicsServer2D.gravity` |

---

## 10. Implementation Priority

### Phase 1: Core ECS + Renderer + Input (Weeks 1-3)

**Goal**: Basic game loop with entities, rendering, and input.

| Task | File | Depends on | Status |
|------|------|------------|--------|
| Entity/Component/World | `core/entity.rs`, `core/world.rs` | — | ✅ Done |
| System scheduler | `core/scheduler.rs` | World | ✅ Done |
| Renderer trait + Canvas | `engine/renderer.rs` | — | ✅ Done |
| Input state + provider | `engine/input.rs` | — | ✅ Done |
| Input action mapping | `engine/input_map.rs` | InputState | 🔲 New |
| Standard components | `core/standard_components.rs` | World | 🔲 New |
| Game loop runner | `engine/runner.rs` | Scheduler, Renderer | 🔲 New |
| Camera follow/zoom | `engine/renderer.rs` (extend) | Camera | 🔲 New |

### Phase 2: Audio + UI + Physics (Weeks 4-6)

**Goal**: Sound, menus, and collision detection.

| Task | File | Depends on | Status |
|------|------|------------|--------|
| Audio backend trait | `engine/audio.rs` | — | ✅ Done |
| Audio bus system | `engine/audio_bus.rs` | AudioManager | 🔲 New |
| Rodio backend | `adapters/rodio_backend.rs` | AudioBackend | 🔲 New |
| Widget tree + layout | `ui/widget.rs` | — | ✅ Done |
| Flex/Grid layout | `ui/layout.rs` | Widget | 🔲 New |
| UI event handling | `ui/events.rs` | Widget | 🔲 New |
| Theme token system | `ui/theme.rs` | UiStyle | 🔲 New |
| Physics world trait | `engine/physics.rs` | — | ✅ Done |
| Rapier backend | `adapters/rapier_backend.rs` | PhysicsWorld | 🔲 New |
| Collision events | `engine/collision_events.rs` | PhysicsWorld | 🔲 New |

### Phase 3: Scene Management + Asset Pipeline (Weeks 7-9)

**Goal**: Multi-scene games with hot-reloadable assets.

| Task | File | Depends on | Status |
|------|------|------------|--------|
| Scene trait | `engine/scene.rs` | — | ✅ Done |
| Scene stack | `engine/scene_stack.rs` | Scene | 🔲 New |
| Scene transitions | `engine/transitions.rs` | SceneStack, Renderer | 🔲 New |
| Asset handle | `engine/asset_handle.rs` | — | 🔲 New |
| Asset server | `engine/asset_server.rs` | AssetHandle | 🔲 New |
| Hot-reload watcher | `engine/hot_reload.rs` | AssetServer | 🔲 New |
| Asset bundles | `engine/asset_bundle.rs` | AssetServer | 🔲 New |

### Phase 4: Cross-Engine Adapters (Weeks 10-12)

**Goal**: Ship to Bevy, Unity, Godot, Unreal via adapters.

| Task | File | Depends on | Status |
|------|------|------------|--------|
| Bevy adapter | `adapters/bevy_adapter.rs` | All core | ✅ Done (entity only) |
| Unity adapter | `adapters/unity_adapter.rs` | All core | ✅ Done (entity only) |
| Godot adapter | `adapters/godot_adapter.rs` | All core | ✅ Done (entity only) |
| Unreal adapter | `adapters/unreal_adapter.rs` | All core | ✅ Done (entity only) |
| Bevy full adapter | `adapters/bevy_full.rs` | Bevy adapter | 🔲 New |
| Unity full adapter | `adapters/unity_full.rs` | Unity adapter | 🔲 New |
| Godot full adapter | `adapters/godot_full.rs` | Godot adapter | 🔲 New |
| Unreal full adapter | `adapters/unreal_full.rs` | Unreal adapter | 🔲 New |
| Codegen per engine | `codegen/engine_gen.rs` | Adapters | 🔲 New |
| Game definition format | `codegen/game_def.rs` | — | ✅ Done |

---

## 11. Architecture Decisions

### AD-001: Trait-Based Backend Selection

**Decision**: All engine subsystems use trait objects for backend selection.

**Rationale**: Compile-time generics don't work for multi-engine targeting. Trait objects allow runtime backend selection.

**Pattern**:
```rust
pub struct Engine {
    renderer: Box<dyn Renderer>,
    audio: Box<dyn AudioBackend>,
    physics: Box<dyn PhysicsWorld>,
    input: Box<dyn InputProvider>,
}
```

### AD-002: Command Buffer Pattern

**Decision**: Renderer uses `DrawCommand` enum (already implemented in `renderer.rs:218`).

**Rationale**: Decouples game logic from rendering. Commands can be serialized, replayed, or dispatched to different backends.

### AD-003: Entity ID as u64

**Decision**: Entity IDs are `u64` not `u32`.

**Rationale**: 4 billion entities is future-proof. No wrap-around risk in long-running sessions. Cross-engine mapping: Unity uses `int` (32-bit) for instance IDs but can map to `u64`.

### AD-004: Wave-Based Parallel Scheduling

**Decision**: Systems run in waves based on dependency graph (already implemented in `scheduler.rs:57`).

**Rationale**: Automatic parallelism without manual annotation. Systems in the same wave have no conflicts and can run concurrently.

### AD-005: Type-Erased Component Storage

**Decision**: Components stored as `Box<dyn Any>` with `TypeId` keys.

**Rationale**: Simple, flexible, no proc macros required. Trade-off: runtime downcast overhead. Future optimization: SoA layout for hot components.

---

## 12. File Structure

```
nt-world-sim/src/
├── core/
│   ├── entity.rs              # EntityId, UniversalEntity, Archetype (✅ done)
│   ├── world.rs               # UniversalWorld, Component, Resource, Event (✅ done)
│   ├── scheduler.rs           # ParallelScheduler, UniversalSystem (✅ done)
│   └── standard_components.rs # Transform, Sprite, Name, Parent, etc. (🔲 new)
├── engine/
│   ├── renderer.rs            # Renderer trait, CanvasRenderer, Color, Vec2, etc. (✅ done)
│   ├── scene.rs               # SceneGraph, SceneNode (✅ done)
│   ├── physics.rs             # PhysicsWorld trait, SimplePhysicsWorld (✅ done)
│   ├── input.rs               # InputState, InputProvider, KeyCode, etc. (✅ done)
│   ├── audio.rs               # AudioBackend, AudioManager (✅ done)
│   ├── event_bus.rs           # TypedEventBus (✅ done)
│   ├── input_map.rs           # InputMap, action binding (🔲 new)
│   ├── audio_bus.rs           # AudioBus hierarchy (🔲 new)
│   ├── asset_handle.rs        # AssetHandle<T> (🔲 new)
│   ├── asset_server.rs        # AssetServer (🔲 new)
│   ├── hot_reload.rs          # File watcher (🔲 new)
│   ├── asset_bundle.rs        # AssetBundle (🔲 new)
│   ├── scene_stack.rs         # SceneStack, Scene trait (🔲 new)
│   ├── transitions.rs         # Scene transitions (🔲 new)
│   ├── runner.rs              # Game loop (🔲 new)
│   └── collision_events.rs    # CollisionEvent, CollisionResponse (🔲 new)
├── ui/
│   ├── widget.rs              # Widget, UiStyle, UiLayout (✅ done)
│   ├── layout.rs              # Flex, Grid layout (🔲 new)
│   ├── events.rs              # UiEvent enum (🔲 new)
│   ├── theme.rs               # Theme tokens, presets (🔲 new)
│   ├── hud.rs                 # HUD widget (✅ done)
│   ├── menu.rs                # Menu system (✅ done)
│   └── dialogue_ui.rs         # Dialogue system (✅ done)
├── adapters/
│   ├── mod.rs                 # Module declarations (✅ done)
│   ├── bevy_adapter.rs        # Bevy entity mapping (✅ done, extend)
│   ├── unity_adapter.rs       # Unity entity mapping (✅ done, extend)
│   ├── godot_adapter.rs       # Godot entity mapping (✅ done, extend)
│   ├── unreal_adapter.rs      # Unreal entity mapping (✅ done, extend)
│   ├── bevy_full.rs           # Bevy full integration (🔲 new)
│   ├── unity_full.rs          # Unity full integration (🔲 new)
│   ├── godot_full.rs          # Godot full integration (🔲 new)
│   ├── unreal_full.rs         # Unreal full integration (🔲 new)
│   ├── rodio_backend.rs       # Rodio audio backend (🔲 new)
│   └── rapier_backend.rs      # Rapier physics backend (🔲 new)
├── codegen/
│   ├── game_def.rs            # GameDefinition format (✅ done)
│   └── engine_gen.rs          # Per-engine code generation (🔲 new)
├── game/                      # Game-specific logic
├── world/                     # World simulation
├── save/                      # Save/load system
├── lib.rs
└── main.rs
```

---

## 13. Naming Conventions

| Pattern | Convention | Example |
|---------|-----------|---------|
| Engine types | `Universal*` | `UniversalEntity`, `UniversalWorld` |
| Traits | No prefix, descriptive | `Renderer`, `PhysicsWorld`, `AudioBackend` |
| Implementations | `*Backend` or `*Impl` | `CanvasRenderer`, `SimplePhysicsWorld` |
| Adapters | `*Adapter` | `BevyAdapter`, `UnityAdapter` |
| Components | PascalCase, noun | `Transform`, `Sprite`, `RigidBody` |
| Systems | Verb phrase | `PhysicsSystem`, `RenderSystem` |
| Resources | PascalCase, noun | `DeltaTime`, `InputState` |
| Events | `*Event` | `CollisionEvent`, `UiClickEvent` |
| Handles | `*Handle` | `AudioHandle`, `AssetHandle<T>` |

---

## 14. Testing Strategy

| Layer | Test type | Example |
|-------|----------|---------|
| Core | Unit test | `test_archetype_query_matching` (entity.rs) |
| Engine | Unit + integration | `test_audio_manager_load_play` (audio.rs) |
| UI | Unit test | `test_widget_fixed_bounds` (widget.rs) |
| Adapters | Unit test | `test_from_external_entity` (bevy_adapter.rs) |
| Cross-engine | Integration | Spawn entity → map to Bevy → verify |
| Performance | Benchmark | 10K entity spawn/despawn, 1K system tick |

**Existing test count**: 35+ unit tests across core/engine/adapters.
