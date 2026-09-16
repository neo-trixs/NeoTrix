# Universal Game Engine Abstraction Layer

> **Design Document v2.0** — Engine-agnostic game framework for nt-world-sim.
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

**Cross-engine mapping**:

| Engine | Entity Type | ID Type | Generational? | Notes |
|--------|------------|---------|---------------|-------|
| **Our Engine** | `UniversalEntity` | `EntityId(u64)` | Yes (u32 gen) | Archetype-based |
| **Bevy** | `Entity` | `u64` (index + generation packed) | Yes | `Entity::from_raw()` |
| **Unity** | `GameObject` | `int instanceID` | No | Handle-based, recycled |
| **Unreal** | `AActor*` | Pointer | No | `FActorInstanceHandle` for weak refs |
| **Godot** | `Node` | `ObjectID` (RID) | No | `NodePath` for string-based |

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

**Standard component library**:

| Component | Fields | Purpose | Source |
|-----------|--------|---------|--------|
| `Transform` | position: Vec2, rotation: f32, scale: Vec2 | Spatial transform | `renderer.rs:96` |
| `Sprite` | texture, rect, color, z_index | Visual representation | `renderer.rs:109` |
| `RigidBody` | position, velocity, mass, body_type | Physics body | `physics.rs:18` |
| `Collider` | AABB / Circle / Polygon | Collision shape | `physics.rs:58` |
| `Name` | name: String | Debug identification | — |
| `Parent` | parent: EntityId | Hierarchy link | `scene.rs` |
| `Velocity` | x: f32, y: f32 | Movement vector | — |
| `Health` | current: f32, max: f32 | Entity health | — |
| `Timer` | remaining: f32, repeat: bool | Time-based triggers | — |

**Cross-engine mapping**:

| Component | Our Engine | Bevy | Unity | Unreal | Godot |
|-----------|-----------|------|-------|--------|-------|
| Transform | `Transform { position, rotation, scale }` | `Transform` | `Transform` | `USceneComponent` | `Node2D.position/rotation/scale` |
| Sprite | `Sprite { texture, rect, color }` | `SpriteBundle` | `SpriteRenderer` | `UPaperSpriteComponent` | `Sprite2D.texture` |
| RigidBody | `RigidBody { velocity, mass, body_type }` | `RigidBody` | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` |
| Collider | `Collider { AABB/Circle/Polygon }` | `Collider` | `Collider2D` | `UShapeComponent` | `CollisionShape2D` |
| Name | `Name(String)` | `Name` | `GameObject.name` | `GetActorName()` | `Node.name` |

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

**System ordering conventions**:

| Phase | Systems | Priority |
|-------|---------|----------|
| 0 | Input collection | -100 |
| 1 | Physics step | 0 |
| 2 | Gameplay logic | 10 |
| 3 | UI update | 20 |
| 4 | Rendering | 100 |

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| System | `UniversalSystem::update()` | `fn system(world: ResMut<W>)` | `MonoBehaviour.Update()` | `Tick()` | `_process(delta)` |
| Scheduler | `ParallelScheduler` (wave) | `Schedule` (ECS) | `Update()` order | TaskGraph | `_process` order |
| Dependency | `SystemDependency` | `in_set()` / `before()` | `[UpdateAfter]` | TaskGraph edges | Manual ordering |
| Parallelism | Wave scheduling | Automatic (ECS) | Manual | TaskGraph | No native parallel |

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
| `PhysicsConfig` | Gravity, iterations | gravity: Vec2, iterations: u32 |

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Resource | `Resource` trait | `Resource` trait | `ScriptableObject` | `UGameInstanceSubsystem` | autoload singleton |
| Access | `world.get_resource::<T>()` | `Res<T>` / `ResMut<T>` | ` FindObjectOfType<T>()` | `GetGameInstance()->GetSubsystem<T>()` | `get_node("/root/Singleton")` |

### 1.5 Event

**Existing**: `Event` trait in `src/core/world.rs:20` + `TypedEventBus` in `src/engine/event_bus.rs`

```rust
pub trait Event: Send + Sync + 'static { }
```

**Standard events**:

| Event | Payload | Purpose |
|-------|---------|---------|
| `CollisionEvent` | entity_a, entity_b, normal, penetration | Physics collision |
| `InputActionEvent` | action: String, state: InputState | Mapped input |
| `SceneTransitionEvent` | from, to, transition_type | Scene change |
| `AssetLoadedEvent` | handle, path, type | Asset ready |
| `DamageEvent` | target, amount, source | Combat |
| `UiClickEvent` | widget_id, position | UI interaction |
| `EntitySpawnedEvent` | entity: EntityId | Entity created |
| `EntityDespawnedEvent` | entity: EntityId | Entity destroyed |

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Event | `TypedEventBus` | `EventWriter<T>/EventReader<T>` | `UnityEvent` | `FOnXxx delegate` | `signal` |
| Dispatch | `world.send_event(e)` | `events.send(e)` | `Invoke()` | `Broadcast()` | `emit()` |
| Handler | `world.receive_events::<T>()` | `EventReader<T>` | listener method | delegate binding | `connect()` |

### 1.6 Scene / Node / Transform

**Existing**: `SceneGraph` in `src/engine/scene.rs:47`

```rust
pub struct SceneGraph {
    nodes: HashMap<EntityId, SceneNode>,
    root: EntityId,
    tag_index: HashMap<String, EntityId>,
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
- `tag: Option<String>` — named lookup

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Scene graph | `SceneGraph` | Hierarchy plugin | `Transform` hierarchy | `USceneComponent` tree | `Node` tree |
| Node | `SceneNode` | `Entity` + `Parent`/`Children` | `GameObject` | `AActor` | `Node` |
| Transform | `Transform` (local + world) | `Transform` | `Transform` | `USceneComponent` | `Node2D` properties |
| Reparenting | `graph.reparent(id, new_parent)` | `HierarchyQuery::push()` | `SetParent()` | `AttachToActor()` | `add_child()` |
| Dirty flag | `node.dirty` | Changed detection | `hasChanged` | `SetRelativeRotation()` | N/A (auto) |

---

## 2. Rendering Abstraction

**Existing**: `Renderer` trait + `CanvasRenderer` in `src/engine/renderer.rs:233`

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

**Extended draw methods** (to add):

| Method | Signature | Purpose |
|--------|-----------|---------|
| `draw_polygon` | `fn draw_polygon(&mut self, vertices: &[Vec2], color: Color, fill: bool)` | Filled/outlined polygon |
| `draw_arc` | `fn draw_arc(&mut self, center: Vec2, radius: f32, start: f32, end: f32, color: Color)` | Arc segment |
| `draw_bezier` | `fn draw_bezier(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, color: Color)` | Bezier curve |
| `draw_string` | `fn draw_string(&mut self, text: &str, pos: Vec2, font: &Font, size: f32, color: Color)` | Font-based text |
| `set_clip` | `fn set_clip(&mut self, rect: Option<Rect>)` | Scissor clipping |
| `push_transform` | `fn push_transform(&mut self, transform: &Transform)` | Transform stack |
| `pop_transform` | `fn pop_transform(&mut self)` | Restore transform |

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

**Existing**: `Camera` in `src/engine/renderer.rs:174` + `Camera2D` in `src/engine/camera.rs:22`

```rust
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub follow_smoothing: f32,
    pub bounds: Option<CameraBounds>,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset: Vec2,
    pub screen_fade: f32,
    pub fade_speed: f32,
}
```

**Camera API**:

| Method | Description |
|--------|-------------|
| `follow(target_pos, lerp)` | Smooth follow with lerp factor |
| `snap_to(target_pos)` | Instant teleport |
| `shake(intensity)` | Trigger screen shake |
| `update_shake(dt)` | Update shake offset, returns current offset |
| `world_to_screen(pos)` | World → screen coordinates |
| `screen_to_world(pos)` | Screen → world coordinates |
| `visible_world_rect()` | AABB of visible world area |
| `is_visible(rect)` | Frustum check for AABB |
| `fade_to_black(dt)` | Fade out, returns true when complete |
| `fade_from_black(dt)` | Fade in, returns true when complete |
| `clamp_to_bounds()` | Clamp to CameraBounds |
| `effective_position()` | Position + shake offset |

**Extended camera features** (to add):

| Feature | Type | Purpose |
|---------|------|---------|
| `look_ahead` | Vec2 | Predict target position for platformers |
| `dead_zone` | Rect | Inner area where camera doesn't follow |
| `parallax_layers` | Vec<(f32, f32)> | Per-layer scroll factors |
| `zoom_range` | (f32, f32) | Min/max zoom limits |
| `rotation_smoothing` | f32 | Smooth rotation follow |

### 2.3 Sprite

**Existing**: `Sprite` in `src/engine/renderer.rs:109`

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
| `anchor` | Vec2 | Anchor point (0-1), default (0.5, 0.5) |
| `region` | Option<Rect> | Source rect in atlas |
| `opacity` | f32 | Transparency (0-1) |
| `blend_mode` | BlendMode | Additive, Multiply, Alpha |

### 2.4 Tilemap

**Existing**: `TileMap` + `TileDef` in `src/engine/renderer.rs:142`

```rust
pub struct TileMap {
    pub tiles: Vec<Vec<u32>>,
    pub tile_size: Vec2,
    pub palette: HashMap<u32, TileDef>,
}
```

**Extended tilemap features**:

| Feature | Type | Purpose |
|---------|------|---------|
| `layers` | Vec<TileMap> | Multi-layer rendering |
| `chunk_size` | usize | Chunk size for culling (default: 16) |
| `auto_tile` | bool | Automatic neighbor-based tiling |
| `isometric` | bool | Isometric vs orthogonal |
| `wrap_x` / `wrap_y` | bool | World wrapping |

**Extended API**:

| Method | Description |
|--------|-------------|
| `chunked_render(camera, chunk_size)` | Render only visible chunks |
| `auto_tile(x, y)` | Auto-tile neighbor detection |
| `get_neighbors(x, y) -> Vec<(i32, i32, u32)>` | Adjacent tile query |
| `set_tile(x, y, tile_id)` | Modify tile (existing) |
| `get_tile(x, y) -> Option<u32>` | Read tile (existing) |
| `is_walkable(x, y) -> bool` | Pathfinding check (existing) |

### 2.5 Particle System

**Existing**: `ParticleSystem` in `src/engine/renderer.rs:425`

```rust
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub gravity: Vec2,
    pub max_particles: usize,
}
```

**Extended particle system**:

```rust
pub struct ParticleEmitter {
    pub position: Vec2,
    pub rate: f32,              // particles per second
    pub lifetime: (f32, f32),   // min/max lifetime
    pub speed: (f32, f32),      // min/max speed
    pub angle: (f32, f32),      // emission cone (radians)
    pub color: (Color, Color),  // start/end gradient
    pub size: (f32, f32),       // start/end size
    pub gravity: Vec2,
    pub max_particles: usize,
    pub shape: EmitterShape,    // Point, Circle, Rectangle, Line
    pub blend_mode: BlendMode,
}

pub enum EmitterShape {
    Point,
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Line { start: Vec2, end: Vec2 },
    Cone { angle: f32, distance: f32 },
}
```

### 2.6 Debug Renderer

**Existing**: `DebugRenderer` in `src/engine/renderer.rs:486`

```rust
pub struct DebugRenderer {
    pub show_grid: bool,
    pub grid_color: Color,
    pub collision_color: Color,
    pub text_overlays: Vec<(String, Vec2, Color, f32)>,
    frame_times: Vec<f32>,
    frame_idx: usize,
}
```

**Extended debug features**:

| Feature | Method | Purpose |
|---------|--------|---------|
| Grid | `render_grid(camera, cell_size)` | World-space grid overlay |
| Collision boxes | `render_collision_box(camera, rect)` | Collider visualization |
| Text overlays | `add_text_overlay(text, pos, color, size)` | On-screen text |
| FPS counter | `avg_fps()` | Frame rate display |
| Profiler bars | `render_profiler(labels, times)` | CPU time visualization |
| Path visualization | `render_path(points, color)` | AI pathfinding debug |
| Entity labels | `render_entity_names(world, camera)` | Show entity names |

### 2.7 Screen Effects

**Existing**: `ScreenEffects` in `src/engine/renderer.rs:575`

```rust
pub struct ScreenEffects {
    pub fade_color: Color,
    pub fade_alpha: f32,
    pub fade_speed: f32,
    pub flash_color: Color,
    pub flash_alpha: f32,
    pub flash_speed: f32,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset: Vec2,
}
```

**Extended screen effects**:

| Effect | Method | Purpose |
|--------|--------|---------|
| Fade in/out | `start_fade_in()` / `start_fade_out()` | Transition overlay |
| Flash | `start_flash(color)` | Hit/damage flash |
| Shake | `start_shake(intensity)` | Impact feedback |
| Tint | `set_tint(color, duration)` | Screen color wash |
| Vignette | `set_vignette(intensity, color)` | Edge darkening |
| Chromatic aberration | `set_chromatic_aberration(strength)` | Distortion effect |
| Scanlines | `set_scanlines(intensity)` | CRT effect |

---

## 3. Input Abstraction

**Existing**: `InputState`, `InputProvider`, `SimpleInputProvider` in `src/engine/input.rs`

### 3.1 InputMap (Action Mapping)

**To implement**: `engine/input_map.rs`

```rust
pub struct InputMap {
    bindings: HashMap<String, Vec<InputBinding>>,
    axes: HashMap<String, AxisBinding>,
}

pub enum InputBinding {
    Key(KeyCode),
    KeyCombo(Vec<KeyCode>),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
    GamepadAxis { axis: GamepadAxis, threshold: f32, direction: AxisDirection },
    MouseAxis(MouseAxis),
}

pub struct AxisBinding {
    pub negative: InputBinding,
    pub positive: InputBinding,
    pub dead_zone: f32,
    pub sensitivity: f32,
    pub gravity: f32,
}

pub enum AxisDirection { Positive, Negative }
pub enum MouseAxis { X, Y, ScrollX, ScrollY }
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
map.bind_axis("move_x", AxisBinding {
    negative: InputBinding::Key(KeyCode::A),
    positive: InputBinding::Key(KeyCode::D),
    dead_zone: 0.1,
    sensitivity: 1.0,
    gravity: 3.0,
});
```

### 3.2 InputState (Query API)

**Existing**: `InputState` in `src/engine/input.rs:60`

Extended API:

| Method | Description |
|--------|-------------|
| `action_pressed(action) -> bool` | Is action held |
| `action_just_pressed(action) -> bool` | Was action just pressed this frame |
| `action_just_released(action) -> bool` | Was action just released this frame |
| `action_axis(negative, positive) -> f32` | Axis value (-1..1) |
| `action_axis_2d(x, y) -> Vec2` | 2D axis vector |
| `action_axis_2d_deadzone(x, y, dz) -> Vec2` | 2D with deadzone |
| `any_action_pressed(actions: &[&str]) -> bool` | Any of the actions held |

### 3.3 Input Events

**Existing**: `KeyCode`, `MouseButton`, `GamepadAxis`, `GamepadButton` enums in `src/engine/input.rs`

```rust
pub enum InputEvent {
    KeyPressed { key: KeyCode },
    KeyReleased { key: KeyCode },
    KeyHeld { key: KeyCode, duration: f32 },
    MouseButtonPressed { button: MouseButton, position: Vec2 },
    MouseButtonReleased { button: MouseButton, position: Vec2 },
    MouseMoved { position: Vec2, delta: Vec2 },
    MouseScrolled { delta: f32 },
    GamepadConnected { id: usize },
    GamepadDisconnected { id: usize },
    GamepadButtonPressed { id: usize, button: GamepadButton },
    GamepadButtonReleased { id: usize, button: GamepadButton },
    GamepadAxisMoved { id: usize, axis: GamepadAxis, value: f32 },
    TouchPressed { id: usize, position: Vec2 },
    TouchMoved { id: usize, position: Vec2, delta: Vec2 },
    TouchReleased { id: usize, position: Vec2 },
}
```

### 3.4 Gamepad Support

**Existing**: `GamepadAxis`, `GamepadButton` in `src/engine/input.rs:39-57`

**Extended gamepad features**:

| Feature | API | Purpose |
|---------|-----|---------|
| Rumble | `set_rumble(gamepad_id, low_freq, high_freq, duration)` | Haptic feedback |
| Vibration | `vibrate(gamepad_id, intensity, duration)` | Simple vibration |
| Player mapping | `set_player_mapping(gamepad_id, player_slot)` | Assign gamepad to player |
| Deadzone | `set_deadzone(axis, threshold)` | Axis deadzone |
| Invert | `set_axis_inverted(axis, inverted)` | Invert axis direction |

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Key enum | `KeyCode` | `KeyCode` | `KeyCode` | `FKey` | `Key` |
| Mouse | `MouseButton` | `MouseButton` | `MouseButton` | `FKey` | `MouseButton` |
| Gamepad | `GamepadAxis/Button` | `GamepadAxis/Button` | `KeyCode.Joystick*` | `FInputKey` | `JoyAxis/JoyButton` |
| Action map | `InputMap` | `InputMap` | `InputManager` | `EnhancedInput` | `InputMap` singleton |
| Input state | `InputState` | `Input<T>` | `Input.GetKey()` | `APlayerController` | `Input` singleton |

---

## 4. Audio Abstraction

**Existing**: `AudioBackend` trait + `AudioManager` in `src/engine/audio.rs`

### 4.1 AudioSource

```rust
pub struct AudioSource {
    pub handle: AudioHandle,
    pub playing: bool,
    pub volume: f32,        // 0.0 - 1.0
    pub pan: f32,           // -1.0 left, 0.0 center, 1.0 right
    pub pitch: f32,         // 1.0 = normal speed
    pub looping: bool,
    pub position: Option<Vec2>,  // spatial audio position
    pub max_distance: f32,       // spatial falloff max
    pub rolloff: f32,            // distance attenuation factor
    pub bus: AudioBusId,         // which bus this plays on
}
```

**Extended AudioSource API**:

| Method | Description |
|--------|-------------|
| `play()` | Start playback |
| `pause()` | Pause playback |
| `stop()` | Stop and reset |
| `seek(position_secs)` | Jump to position |
| `fade_to(target_volume, duration)` | Smooth volume change |
| `fade_out_and_stop(duration)` | Fade then stop |
| `set_3d_attributes(pos, velocity)` | Update spatial position |

### 4.2 AudioBus

**Existing**: `master_volume`, `music_volume`, `sfx_volume` in `AudioManager`

Extended bus system:

```rust
pub struct AudioBus {
    pub id: AudioBusId,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub solo: bool,
    pub parent: Option<AudioBusId>,
    pub children: Vec<AudioBusId>,
    pub effects: Vec<AudioEffect>,
}

pub enum AudioEffect {
    LowPass { cutoff: f32 },
    HighPass { cutoff: f32 },
    Reverb { room_size: f32, damping: f32 },
    Delay { time: f32, feedback: f32 },
    Compressor { threshold: f32, ratio: f32 },
}
```

**Default bus hierarchy**:

```
Master (1.0)
├── Music (0.7)
│   ├── BGM
│   └── Ambient
├── SFX (1.0)
│   ├── Combat
│   ├── UI
│   └── Environment
└── Voice (1.0)
    ├── Dialogue
    └── Narration
```

### 4.3 Spatial Audio (2D Positional)

```rust
pub struct SpatialAudioListener {
    pub position: Vec2,
    pub velocity: Vec2,
    pub orientation: f32,       // rotation in radians
    pub max_distance: f32,      // listener range
}

pub struct SpatialAudioSource {
    pub position: Vec2,
    pub velocity: Vec2,
    pub min_distance: f32,      // full volume within this radius
    pub max_distance: f32,      // silent beyond this radius
    pub rolloff_model: RolloffModel,
    pub pan_mode: SpatialPanMode,
}

pub enum RolloffModel {
    Linear,      // volume = 1 - (dist - min) / (max - min)
    Logarithmic, // volume = min / (min + rolloff * (dist - min))
    Custom,      // user-defined curve
}

pub enum SpatialPanMode {
    SimulatedStereo,  // pan based on angle to listener
    HRTF,             // head-related transfer function (advanced)
}
```

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Backend | `AudioBackend` trait | `AudioPlugin` | `AudioListener` | `FAudioDevice` | `AudioServer` |
| Source | `AudioHandle` | `AudioBundle` | `AudioSource` | `UAudioComponent` | `AudioStreamPlayer` |
| Bus/Volume | `AudioBus` | `AudioChannel` | `AudioMixer` | `USoundMix` | `AudioBus` |
| Spatial | `SpatialAudioSource` | `SpatialScale` | `AudioSource.spatialBlend` | `USpatializationVolume` | `AudioStreamPlayer2D` |

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
    pub focusable: bool,
}

pub trait WidgetTrait: Send + Sync {
    fn type_name(&self) -> &str;
    fn render(&self, renderer: &mut dyn Renderer, bounds: Rect);
    fn handle_event(&mut self, event: &UiEvent) -> bool;
    fn measure(&self, available: Vec2) -> Vec2;
    fn layout(&mut self, bounds: Rect);
}
```

**Widget operations**:

| Operation | Method | Description |
|-----------|--------|-------------|
| Add child | `add_child(widget)` | Append to children list |
| Remove child | `remove_child(id)` | Remove by WidgetId |
| Find child | `find_child(predicate)` | Recursive search |
| Traversal | `for_each(callback)` | Depth-first traversal |
| Hit test | `hit_test(position) -> Option<WidgetId>` | Find widget at point |
| Focus | `set_focus(id)` | Move keyboard focus |

### 5.2 Layout System

**Existing**: `UiLayout` enum in `src/ui/widget.rs`

```rust
pub enum UiLayout {
    Fixed { x: f32, y: f32, width: f32, height: f32 },
    Anchor { top: Option<f32>, bottom: Option<f32>, left: Option<f32>, right: Option<f32> },
    Center { width: f32, height: f32 },
    Flex { direction: FlexDirection, gap: f32, wrap: bool, align: FlexAlign },
    Grid { columns: usize, rows: usize, gap: f32, column_ratios: Option<Vec<f32>> },
    Stack { direction: StackDirection, gap: f32 },
    Relative { width_percent: f32, height_percent: f32 },
}

pub enum FlexDirection { Row, Column, RowReverse, ColumnReverse }
pub enum FlexAlign { Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly }
pub enum StackDirection { Horizontal, Vertical }

pub struct FlexItem {
    pub grow: f32,       // flex-grow
    pub shrink: f32,     // flex-shrink
    pub basis: Option<f32>, // flex-basis
    pub align_self: Option<FlexAlign>,
}
```

**Layout algorithm**:
1. Measure phase: each widget reports its intrinsic size
2. Distribute phase: flex items distribute available space
3. Position phase: place items according to alignment
4. Layout phase: set final bounds on each widget

### 5.3 Style (Theme Tokens)

**Existing**: `UiStyle` in `src/ui/widget.rs`

```rust
pub struct UiStyle {
    pub background: Color,
    pub border: Color,
    pub border_width: f32,
    pub text_color: Color,
    pub font_size: f32,
    pub padding: f32,
    pub corner_radius: f32,
    pub margin: f32,
    pub opacity: f32,
    pub shadow_offset: Vec2,
    pub shadow_color: Color,
    pub shadow_blur: f32,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}
```

**Theme token system**:

| Category | Tokens | Purpose |
|----------|--------|---------|
| Color | `primary`, `secondary`, `surface`, `error`, `on_primary`, `on_surface` | Semantic colors |
| Typography | `heading_font`, `body_font`, `caption_font`, `mono_font` | Font families |
| Spacing | `space_xs`, `space_sm`, `space_md`, `space_lg`, `space_xl` | Consistent spacing |
| Radius | `radius_sm`, `radius_md`, `radius_lg`, `radius_full` | Corner rounding |
| Shadow | `shadow_sm`, `shadow_md`, `shadow_lg` | Elevation levels |

**Theme presets**:

| Preset | Style |
|--------|-------|
| `default()` | Dark brown/gold |
| `stardew_wood()` | Wood panel style |
| `stardew_button()` | Wood button |
| `modern_dark()` | Dark gray/blue |
| `retro_green()` | CRT green-on-black |
| `minimal_white()` | Clean white |
| `cyberpunk()` | Neon pink/cyan on dark |
| `nature()` | Earth tones, green accents |

### 5.4 UI Events

```rust
pub enum UiEvent {
    Click { widget_id: WidgetId, position: Vec2 },
    DoubleClick { widget_id: WidgetId, position: Vec2 },
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
    ValueChanged { widget_id: WidgetId, value: f32 },
    SelectionChanged { widget_id: WidgetId, index: usize },
}
```

### 5.5 Widget Types

| Widget | Purpose | Key Properties |
|--------|---------|----------------|
| `Button` | Clickable action | label, on_click, disabled |
| `Label` | Text display | text, font_size, color, alignment |
| `Image` | Texture display | texture, tint, fit |
| `Panel` | Container | background, border, padding |
| `Slider` | Value input | min, max, value, step, on_change |
| `TextInput` | Text input | placeholder, max_length, on_submit |
| `CheckBox` | Toggle | checked, on_toggle |
| `ProgressBar` | Progress display | value, max, bar_color |
| `ScrollArea` | Scrollable container | content_size, scrollbar |
| `Dropdown` | Selection | options, selected, on_select |
| `ListView` | Item list | items, selected_index, on_item_click |
| `TabBar` | Tab switching | tabs, selected, on_tab_change |

---

## 6. Physics Abstraction

**Existing**: `PhysicsWorld` trait + `SimplePhysicsWorld` in `src/engine/physics.rs`

### 6.1 Collider

**Existing**: `Collider` enum in `src/engine/physics.rs:58`

```rust
pub enum Collider {
    AABB { half_extents: Vec2 },
    Circle { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}
```

**Extended colliders**:

| Collider | Shape | Use Case | Algorithm |
|----------|-------|----------|-----------|
| `AABB` | Rectangle | General purpose | `intersects()` O(1) |
| `Circle` | Circle | Round objects | Distance check O(1) |
| `Polygon` | Convex polygon | Complex shapes | SAT O(n) |
| `Compound` | Union of shapes | Multi-part objects | Composite test |
| `Raycast` | Ray | Projectiles, line of sight | Ray-AABB/Circle/Polygon |
| `Sensor` | Any shape | Triggers, zones | Same as above, no response |

**Extended collider properties**:

| Property | Type | Purpose |
|----------|------|---------|
| `offset` | Vec2 | Position offset from entity |
| `rotation` | f32 | Collider rotation (for polygons) |
| `sensor` | bool | Trigger-only (no physics response) |
| `collision_layer` | u32 | Bitmask: which layer this belongs to |
| `collision_mask` | u32 | Bitmask: which layers to collide with |
| `friction` | f32 | Surface friction |
| `restitution` | f32 | Bounciness |

### 6.2 RigidBody

**Existing**: `RigidBody` in `src/engine/physics.rs:18`

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

**Extended RigidBody fields**:

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `linear_damping` | f32 | 0.0 | Velocity decay per second |
| `angular_velocity` | f32 | 0.0 | Rotation speed (rad/s) |
| `angular_damping` | f32 | 0.0 | Rotation decay |
| `max_velocity` | f32 | inf | Speed cap |
| `is_sleeping` | bool | false | Optimization |
| `sleep_threshold` | f32 | 0.5 | Velocity threshold for sleep |
| `fixed_rotation` | bool | false | Lock rotation |
| `bullet` | bool | false | Continuous collision for fast objects |
| `collision_layers` | u32 | 0xFFFFFFFF | Layer bitmask |
| `collision_events` | bool | true | Emit collision events |

**Body types**:

| Type | Behavior | Typical Use |
|------|----------|-------------|
| `Static` | Immovable, infinite mass | Walls, floors |
| `Dynamic` | Full physics simulation | Players, enemies, projectiles |
| `Kinematic` | Script-controlled, affects others | Moving platforms, doors |

### 6.3 Collision Response

**Existing**: `CollisionInfo` in `src/engine/physics.rs:123`

```rust
pub struct CollisionInfo {
    pub entity_a: PhysicsEntity,
    pub entity_b: PhysicsEntity,
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
| `Callback` | Custom user response | `on_collision` handler |

**Collision event types**:

```rust
pub enum CollisionEventType {
    Started,    // First frame of contact
    Persisting, // Ongoing contact
    Ended,      // Separation
}
```

### 6.4 Raycast

```rust
pub struct RaycastQuery {
    pub origin: Vec2,
    pub direction: Vec2,
    pub max_distance: f32,
    pub collision_mask: u32,
    pub query_type: RaycastType,
}

pub enum RaycastType {
    Ray,        // Single ray
    CircleCast, // Swept circle
    BoxCast,    // Swept AABB
}

pub struct RaycastHit {
    pub entity: PhysicsEntity,
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
    pub fraction: f32,
}

impl PhysicsWorld {
    fn raycast(&self, query: &RaycastQuery) -> Vec<RaycastHit>;
    fn raycast_first(&self, query: &RaycastQuery) -> Option<RaycastHit>;
    fn raycast_all(&self, query: &RaycastQuery) -> Vec<RaycastHit>;
    fn overlap_circle(&self, center: Vec2, radius: f32, mask: u32) -> Vec<PhysicsEntity>;
    fn overlap_box(&self, center: Vec2, half_extents: Vec2, mask: u32) -> Vec<PhysicsEntity>;
}
```

### 6.5 Joints

```rust
pub enum Joint {
    Distance {
        anchor_a: Vec2,
        anchor_b: Vec2,
        min_distance: f32,
        max_distance: f32,
    },
    Revolute {
        anchor_a: Vec2,
        anchor_b: Vec2,
        motor_speed: f32,
        max_torque: f32,
        angle_limits: Option<(f32, f32)>,
    },
    Prismatic {
        anchor_a: Vec2,
        anchor_b: Vec2,
        axis: Vec2,
        motor_speed: f32,
        max_force: f32,
        limits: Option<(f32, f32)>,
    },
    Weld {
        anchor: Vec2,
        frequency: f32,
        damping: f32,
    },
    Rope {
        anchor_a: Vec2,
        anchor_b: Vec2,
        max_distance: f32,
    },
    Friction {
        anchor: Vec2,
        max_force: f32,
        max_torque: f32,
    },
}

pub struct JointHandle(pub u32);

impl PhysicsWorld {
    fn create_joint(&mut self, entity_a: PhysicsEntity, entity_b: PhysicsEntity, joint: Joint) -> JointHandle;
    fn destroy_joint(&mut self, handle: JointHandle);
    fn get_joint_anchor(&self, handle: JointHandle) -> (Vec2, Vec2);
    fn set_joint_motor(&mut self, handle: JointHandle, speed: f32, max_force: f32);
}
```

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Physics world | `PhysicsWorld` trait | `RapierPlugin` | `Physics2DSettings` | `FPhysScene` | `PhysicsServer2D` |
| Rigid body | `RigidBody` | `RigidBody` | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` |
| Collider | `Collider` | `Collider` | `Collider2D` | `UShapeComponent` | `CollisionShape2D` |
| Joint | `Joint` | `RevoluteJoint` | `DistanceJoint2D` | `UPhysicsConstraintComponent` | `Joint2D` |
| Raycast | `raycast()` | `RayCaster::new()` | `Physics2D.Raycast()` | `LineTraceSingle()`` | `Physics2D.intersect_ray()` |
| Gravity | `gravity: Vec2` | `Gravity` resource | `Physics2D.gravity` | `FPhysScene::SetGravity()` | `PhysicsServer2D.gravity` |

---

## 7. Scene Management

**Existing**: `SceneGraph` in `src/engine/scene.rs`

### 7.1 Scene Stack

```rust
pub struct SceneStack {
    scenes: Vec<Box<dyn Scene>>,
    transitions: Vec<SceneTransition>,
    pending_operations: Vec<SceneOperation>,
}

pub enum SceneOperation {
    Push(Box<dyn Scene>),
    Pop,
    Replace(Box<dyn Scene>),
    Clear,
}

pub trait Scene: Send + Sync {
    fn name(&self) -> &str;
    fn on_enter(&mut self, world: &mut UniversalWorld, data: Option<Box<dyn Any>>);
    fn on_exit(&mut self, world: &mut UniversalWorld);
    fn on_pause(&mut self, world: &mut UniversalWorld);
    fn on_resume(&mut self, world: &mut UniversalWorld);
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn render(&self, renderer: &mut dyn Renderer, world: &UniversalWorld);
    fn handle_event(&mut self, event: &InputEvent) -> bool;
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
| `depth()` | Number of scenes in stack |
| `find_by_name(name)` | Find scene by name |

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
    Wipe { direction: Vec2, duration: f32 },
    Zoom { center: Vec2, duration: f32 },
}

pub struct SceneTransition {
    pub from: String,
    pub to: String,
    pub transition: TransitionType,
    pub progress: f32,          // 0.0 → 1.0
    pub phase: TransitionPhase,
    pub callback: Option<Box<dyn FnOnce()>>,
}

pub enum TransitionPhase {
    Out,    // Fading/sliding out current scene
    Switch, // Swap scenes
    In,     // Fading/sliding in new scene
}
```

### 7.3 Scene Lifecycle

```
push(scene)
  → current.on_pause()
  → scene.on_enter(data)
  → transition animates (Out → Switch → In)
  → scene becomes active

pop()
  → current.on_exit()
  → transition animates
  → previous.on_resume()
  → previous becomes active

replace(scene)
  → current.on_exit()
  → scene.on_enter(data)
  → scene becomes active (no pause/resume)
```

### 7.4 Scene Data Passing

```rust
pub enum SceneData {
    None,
    Value(Box<dyn Any>),
    Map(HashMap<String, Box<dyn Any>>),
}

// Usage:
scene_stack.push_with_data(
    Box::new(BattleScene::new(enemy_id)),
    SceneData::Map(HashMap::from([
        ("player_hp".into(), Box::new(100u32)),
        ("difficulty".into(), Box::new(2.5f32)),
    ])),
);

// In on_enter:
fn on_enter(&mut self, world: &mut UniversalWorld, data: Option<Box<dyn Any>>) {
    if let Some(data) = data {
        if let Some(map) = data.downcast_ref::<HashMap<String, Box<dyn Any>>>() {
            if let Some(hp) = map.get("player_hp") {
                self.player_hp = *hp.downcast_ref::<u32>().unwrap();
            }
        }
    }
}
```

---

## 8. Asset Pipeline

### 8.1 AssetServer

**Existing**: `AssetServer` in `src/engine/asset.rs:55`

Extended asset server:

```rust
pub struct AssetServer {
    textures: HashMap<String, TextureData>,
    sounds: HashMap<String, SoundData>,
    fonts: HashMap<String, FontData>,
    loaded: HashMap<String, AssetEntry>,
    loading_queue: VecDeque<LoadRequest>,
    watchers: HashMap<String, WatcherEntry>,
    base_path: String,
    hot_reload_enabled: bool,
    next_id: u32,
}

pub struct AssetEntry {
    pub path: String,
    pub state: AssetState,
    pub type_id: TypeId,
    pub last_modified: SystemTime,
    pub ref_count: u32,
}

pub enum AssetState {
    NotLoaded,
    Loading,
    Loaded,
    Failed(String),
    Unloaded,
}

pub enum AssetType {
    Texture,
    Sound,
    Font,
    Tilemap,
    Scene,
    Script,
    Shader,
    Config,
}
```

**AssetServer API**:

| Method | Description |
|--------|-------------|
| `load::<T>(path)` | Load asset, returns handle |
| `load_async::<T>(path)` | Async load, returns future |
| `get::<T>(handle)` | Get loaded asset by handle |
| `get_mut::<T>(handle)` | Get mutable reference |
| `try_get::<T>(handle)` | Non-panicking get |
| `unload(path)` | Unload asset, free memory |
| `reload(path)` | Force reload from disk |
| `watch(path)` | Enable hot-reload for path |
| `unwatch(path)` | Disable hot-reload |
| `tick(dt)` | Process loading queue, check file changes |
| `load_bundle(bundle)` | Load grouped assets |
| `preload(paths)` | Start loading in background |
| `is_loaded(path) -> bool` | Check if asset is ready |
| `memory_usage() -> usize` | Total memory used by assets |

### 8.2 AssetHandle

**Existing**: `AssetHandle<T>` in `src/engine/asset.rs:4`

```rust
pub struct AssetHandle<T> {
    pub id: u32,
    pub _marker: std::marker::PhantomData<T>,
}
```

**Extended handle features**:

| Feature | Implementation | Purpose |
|---------|---------------|---------|
| Type safety | `PhantomData<T>` | Compile-time type checking |
| Weak references | `WeakAssetHandle<T>` | Don't prevent unloading |
| Validation | `is_valid(server) -> bool` | Check handle is still valid |
| Generational | Include generation in ID | Detect stale handles |

### 8.3 AssetBundle

```rust
pub struct AssetBundle {
    pub name: String,
    pub assets: Vec<BundleAsset>,
    pub priority: LoadPriority,
    pub on_complete: Option<Box<dyn FnOnce(&AssetServer) + Send>>,
}

pub struct BundleAsset {
    pub path: String,
    pub asset_type: AssetType,
    pub required: bool,      // fail bundle if this fails
}

pub enum LoadPriority {
    Immediate,  // Block until loaded
    High,       // Load next frame
    Normal,     // Load when available
    Low,        // Load when idle
}

// Usage:
let bundle = AssetBundle {
    name: "level_1".into(),
    assets: vec![
        BundleAsset { path: "textures/tileset.png".into(), asset_type: AssetType::Texture, required: true },
        BundleAsset { path: "sounds/bgm.ogg".into(), asset_type: AssetType::Sound, required: false },
        BundleAsset { path: "scenes/level_1.tmx".into(), asset_type: AssetType::Tilemap, required: true },
    ],
    priority: LoadPriority::High,
    on_complete: Some(Box::new(|server| {
        println!("Level 1 assets loaded!");
    })),
};
asset_server.load_bundle(bundle);
```

### 8.4 Asset Manifest

```json
{
  "name": "my_game",
  "version": "1.0.0",
  "bundles": {
    "core": {
      "assets": [
        { "path": "fonts/main.ttf", "type": "font", "required": true },
        { "path": "sounds/click.wav", "type": "sound", "required": true }
      ],
      "priority": "immediate"
    },
    "level_1": {
      "assets": [
        { "path": "textures/level_1.png", "type": "texture", "required": true },
        { "path": "tilemaps/level_1.json", "type": "tilemap", "required": true },
        { "path": "sounds/level_1_bgm.ogg", "type": "sound", "required": false }
      ],
      "priority": "high",
      "depends_on": ["core"]
    },
    "level_2": {
      "assets": [
        { "path": "textures/level_2.png", "type": "texture", "required": true },
        { "path": "tilemaps/level_2.json", "type": "tilemap", "required": true }
      ],
      "priority": "normal",
      "depends_on": ["core"]
    }
  },
  "hot_reload": {
    "enabled": true,
    "paths": ["scripts/", "shaders/"],
    "debounce_ms": 500
  }
}
```

**Cross-engine mapping**:

| Concept | Our Engine | Bevy | Unity | Unreal | Godot |
|---------|-----------|------|-------|--------|-------|
| Asset server | `AssetServer` | `AssetServer` | `AssetDatabase` | `FAssetManager` | `ResourceLoader` |
| Handle | `AssetHandle<T>` | `Handle<T>` | `Asset<T>` | `TObjectPtr<T>` | `Resource` |
| Bundle | `AssetBundle` | `AssetBundle` | `AssetBundle` | `FPrimaryAssetId` | — |
| Loading | `load(path)` | `load()` | `Resources.Load()` | `LoadObject()` | `load()` |
| Hot reload | File watcher | `AssetServer::watch()` | Domain reload | `FAssetData::GetTagValue()` | Editor import |

---

## 9. Cross-Engine Mapping Table

### 9.1 Complete Mapping

| Abstraction | Our Engine (Rust) | Bevy (Rust) | Unity (C#) | Unreal (C++) | Godot (GDScript) |
|-------------|-------------------|-------------|------------|--------------|-------------------|
| **Entity** | `UniversalEntity` | `Entity` | `GameObject` | `AActor` | `Node` |
| **Entity ID** | `EntityId(u64)` | `Entity` (u64 packed) | `int instanceID` | `FActorInstanceHandle` | `ObjectID` / `NodePath` |
| **Component** | `Component` trait | `Component` derive | `MonoBehaviour` | `UActorComponent` | N/A (node properties) |
| **System** | `UniversalSystem` | `fn system(ResMut<W>)` | `MonoBehaviour.Update()` | `Tick()` | `_process(delta)` |
| **Resource** | `Resource` trait | `Resource` trait | `ScriptableObject` | `UGameInstanceSubsystem` | autoload singleton |
| **Event** | `TypedEventBus` | `EventWriter/Reader<T>` | `UnityEvent` | `FOnXxx delegate` | `signal` |
| **Transform** | `Transform { pos, rot, scale }` | `Transform` | `Transform` | `USceneComponent` | `Node2D` properties |
| **Sprite** | `Sprite` | `SpriteBundle` | `SpriteRenderer` | `UPaperSprite` | `Sprite2D` |
| **Camera** | `Camera2D` | `Camera2dBundle` | `Camera` | `UCameraComponent` | `Camera2D` |
| **Tilemap** | `TileMap` | `TilemapBundle` | `Tilemap` | N/A | `TileMap` |
| **Renderer** | `Renderer` trait | `RenderPlugin` | `SRP/URP` | `FSceneInterface` | `RenderingServer` |
| **Draw commands** | `DrawCommand` enum | `RenderWorld` | `CommandBuffer` | `FRHICommandList` | `RenderingServer` |
| **Physics** | `PhysicsWorld` trait | `RapierPlugin` | `Physics2DSettings` | `FPhysScene` | `PhysicsServer2D` |
| **Rigid body** | `RigidBody` | `RigidBody` | `Rigidbody2D` | `UPrimitiveComponent` | `RigidBody2D` |
| **Collider** | `Collider` | `Collider` | `Collider2D` | `UShapeComponent` | `CollisionShape2D` |
| **Joint** | `Joint` | `RevoluteJoint` | `DistanceJoint2D` | `UPhysicsConstraintComponent` | `Joint2D` |
| **Raycast** | `raycast()` | `RayCaster` | `Physics2D.Raycast()` | `LineTraceSingle()` | `Physics2D.intersect_ray()` |
| **Input state** | `InputState` | `Input<T>` | `Input.GetKey()` | `APlayerController` | `Input` singleton |
| **Action map** | `InputMap` | `InputMap` | `InputManager` | `EnhancedInput` | `InputMap` singleton |
| **Audio backend** | `AudioBackend` trait | `AudioPlugin` | `AudioListener` | `FAudioDevice` | `AudioServer` |
| **Audio source** | `AudioHandle` | `AudioBundle` | `AudioSource` | `UAudioComponent` | `AudioStreamPlayer` |
| **Audio bus** | `AudioBus` | `AudioChannel` | `AudioMixer` | `USoundMix` | `AudioBus` |
| **Scene graph** | `SceneGraph` | Hierarchy plugin | `Transform` hierarchy | `USceneComponent` tree | `Node` tree |
| **Scene stack** | `SceneStack` | State plugin | `SceneManager` | `UGameInstance` | `SceneTree` |
| **Scene transition** | `SceneTransition` | `States` | `SceneManager.LoadSceneAsync()` | `UGameplayStatics::OpenLevel()` | `SceneTree.change_scene()` |
| **Widget/UI** | `WidgetTrait` | `UiRect` / `Style` | `UI Toolkit` | `UWidget` / UMG | `Control` nodes |
| **Layout** | `UiLayout` (Flex/Grid) | `FlexboxLayout` | `LayoutGroup` | `UVerticalBox` / `UHorizontalBox` | `Container` nodes |
| **Asset handle** | `AssetHandle<T>` | `Handle<T>` | `Asset<T>` | `TObjectPtr<T>` | `Resource` |
| **Asset server** | `AssetServer` | `AssetServer` | `AssetDatabase` | `FAssetManager` | `ResourceLoader` |
| **Game loop** | `ParallelScheduler` | `App::run()` | `MonoBehaviour` lifecycle | `UGameInstance` | `SceneTree` process loop |

### 9.2 Mapping Patterns

| Pattern | Our Engine | Adapts To |
|---------|-----------|-----------|
| **ECS** | `UniversalWorld` + `Component` trait | Bevy ECS, Unity DOTS, Flecs |
| **Scene graph** | `SceneGraph` + `SceneNode` | Unity Hierarchy, Godot Node tree, UE Actor tree |
| **Trait-based backend** | `Box<dyn Renderer>`, `Box<dyn AudioBackend>` | Compile-time generics in target engine |
| **Command buffer** | `DrawCommand` enum | Unity `CommandBuffer`, UE render graph |
| **Event bus** | `TypedEventBus` | Bevy `EventWriter`, Unity `UnityEvent`, Godot `signal` |

---

## 10. Implementation Priority

### Phase 1: Core ECS + Renderer + Input (Weeks 1-3)

**Goal**: Basic game loop with entities, rendering, and input.

| Task | File | Effort | Depends on | Status |
|------|------|--------|------------|--------|
| Entity/Component/World | `core/entity.rs`, `core/world.rs` | — | — | ✅ Done |
| System scheduler | `core/scheduler.rs` | — | World | ✅ Done |
| Renderer trait + Canvas | `engine/renderer.rs` | — | — | ✅ Done |
| Input state + provider | `engine/input.rs` | — | — | ✅ Done |
| Standard components | `core/standard_components.rs` | 2 days | World | 🔲 New |
| Input action mapping | `engine/input_map.rs` | 3 days | InputState | 🔲 New |
| Game loop runner | `engine/runner.rs` | 2 days | Scheduler, Renderer | 🔲 New |
| Camera follow/zoom/shake | `engine/camera.rs` (extend) | 1 day | Camera | 🔲 New |
| Draw command pipeline | `engine/renderer.rs` (extend) | 2 days | Renderer | 🔲 New |
| Sprite batch optimization | `engine/renderer.rs` (extend) | 2 days | Renderer | 🔲 New |

**Estimated effort**: 12 days (2.4 weeks)

### Phase 2: Audio + UI + Physics (Weeks 4-6)

**Goal**: Sound, menus, and collision detection.

| Task | File | Effort | Depends on | Status |
|------|------|--------|------------|--------|
| Audio backend trait | `engine/audio.rs` | — | — | ✅ Done |
| Audio bus system | `engine/audio_bus.rs` | 2 days | AudioManager | 🔲 New |
| Spatial audio | `engine/audio.rs` (extend) | 2 days | AudioBackend | 🔲 New |
| Rodio backend | `adapters/rodio_backend.rs` | 3 days | AudioBackend | 🔲 New |
| Widget tree + layout | `ui/widget.rs` | — | — | ✅ Done |
| Flex/Grid layout | `ui/layout.rs` | 4 days | Widget | 🔲 New |
| UI event handling | `ui/events.rs` | 2 days | Widget | 🔲 New |
| Theme token system | `ui/theme.rs` | 2 days | UiStyle | 🔲 New |
| Standard widgets | `ui/widgets/*.rs` | 5 days | Widget, Layout | 🔲 New |
| Physics world trait | `engine/physics.rs` | — | — | ✅ Done |
| Raycast/overlap queries | `engine/physics.rs` (extend) | 2 days | PhysicsWorld | 🔲 New |
| Joint system | `engine/joints.rs` | 3 days | PhysicsWorld | 🔲 New |
| Rapier backend | `adapters/rapier_backend.rs` | 4 days | PhysicsWorld | 🔲 New |
| Collision events | `engine/collision_events.rs` | 1 day | PhysicsWorld | 🔲 New |

**Estimated effort**: 32 days (6.4 weeks)

### Phase 3: Scene Management + Asset Pipeline (Weeks 7-9)

**Goal**: Multi-scene games with hot-reloadable assets.

| Task | File | Effort | Depends on | Status |
|------|------|--------|------------|--------|
| Scene trait | `engine/scene.rs` | — | — | ✅ Done |
| Scene stack | `engine/scene_stack.rs` | 2 days | Scene | 🔲 New |
| Scene transitions | `engine/transitions.rs` | 4 days | SceneStack, Renderer | 🔲 New |
| Scene data passing | `engine/scene_stack.rs` (extend) | 1 day | SceneStack | 🔲 New |
| Asset handle | `engine/asset.rs` (extend) | 1 day | — | 🔲 New |
| Asset server | `engine/asset.rs` (extend) | 3 days | AssetHandle | 🔲 New |
| Hot-reload watcher | `engine/hot_reload.rs` | 3 days | AssetServer | 🔲 New |
| Asset bundles | `engine/asset.rs` (extend) | 2 days | AssetServer | 🔲 New |
| Asset manifest | `engine/manifest.rs` | 2 days | AssetServer | 🔲 New |
| Async loading | `engine/asset.rs` (extend) | 2 days | AssetServer | 🔲 New |

**Estimated effort**: 20 days (4 weeks)

### Phase 4: Cross-Engine Adapters (Weeks 10-12)

**Goal**: Ship to Bevy, Unity, Godot, Unreal via adapters.

| Task | File | Effort | Depends on | Status |
|------|------|--------|------------|--------|
| Bevy adapter | `adapters/bevy_adapter.rs` | — | All core | ✅ Done (entity only) |
| Unity adapter | `adapters/unity_adapter.rs` | — | All core | ✅ Done (entity only) |
| Godot adapter | `adapters/godot_adapter.rs` | — | All core | ✅ Done (entity only) |
| Unreal adapter | `adapters/unreal_adapter.rs` | — | All core | ✅ Done (entity only) |
| Bevy full adapter | `adapters/bevy_full.rs` | 10 days | All core | 🔲 New |
| Unity full adapter | `adapters/unity_full.rs` | 10 days | All core | 🔲 New |
| Godot full adapter | `adapters/godot_full.rs` | 8 days | All core | 🔲 New |
| Unreal full adapter | `adapters/unreal_full.rs` | 12 days | All core | 🔲 New |
| Codegen per engine | `codegen/engine_gen.rs` | 5 days | Adapters | 🔲 New |
| Game definition format | `codegen/game_def.rs` | — | — | ✅ Done |

**Estimated effort**: 45 days (9 weeks)

### Total Effort Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 2.4 weeks | Core ECS + Rendering + Input |
| Phase 2 | 6.4 weeks | Audio + UI + Physics |
| Phase 3 | 4 weeks | Scene Management + Assets |
| Phase 4 | 9 weeks | Cross-Engine Adapters |
| **Total** | **21.8 weeks** | Complete Universal Engine |

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

### AD-006: Scene Graph as World View

**Decision**: Scene graph is a view over the world, not a separate container.

**Rationale**: Entities live in `UniversalWorld`. Scene graph manages hierarchy and transform propagation. Avoids data duplication and synchronization issues.

### AD-007: Event-Driven Decoupling

**Decision**: Subsystems communicate via events, not direct function calls.

**Rationale**: Reduces coupling between systems. Events can be logged, replayed, and filtered. Supports cross-engine event mapping.

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
│   ├── renderer.rs            # Renderer trait, CanvasRenderer, Color, Vec2 (✅ done)
│   ├── scene.rs               # SceneGraph, SceneNode (✅ done)
│   ├── physics.rs             # PhysicsWorld trait, SimplePhysicsWorld (✅ done)
│   ├── input.rs               # InputState, InputProvider, KeyCode (✅ done)
│   ├── audio.rs               # AudioBackend, AudioManager (✅ done)
│   ├── camera.rs              # Camera2D, CameraBounds (✅ done)
│   ├── asset.rs               # AssetHandle, AssetServer (✅ done)
│   ├── event_bus.rs           # TypedEventBus (✅ done)
│   ├── input_map.rs           # InputMap, action binding (🔲 new)
│   ├── audio_bus.rs           # AudioBus hierarchy (🔲 new)
│   ├── hot_reload.rs          # File watcher (🔲 new)
│   ├── scene_stack.rs         # SceneStack, Scene trait (🔲 new)
│   ├── transitions.rs         # Scene transitions (🔲 new)
│   ├── joints.rs              # Joint system (🔲 new)
│   ├── runner.rs              # Game loop (🔲 new)
│   ├── collision_events.rs    # CollisionEvent (🔲 new)
│   ├── manifest.rs            # Asset manifest (🔲 new)
│   └── debug_renderer.rs      # Debug rendering (🔲 new)
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
| Enums | PascalCase | `BodyType`, `TransitionType` |
| Module files | snake_case | `input_map.rs`, `scene_stack.rs` |

---

## 14. Testing Strategy

| Layer | Test Type | Example | Coverage |
|-------|----------|---------|----------|
| Core | Unit test | `test_archetype_query_matching` (entity.rs) | 100% core |
| Engine | Unit + integration | `test_audio_manager_load_play` (audio.rs) | 90% engine |
| UI | Unit test | `test_widget_fixed_bounds` (widget.rs) | 80% UI |
| Adapters | Unit test | `test_from_external_entity` (bevy_adapter.rs) | 70% adapters |
| Cross-engine | Integration | Spawn entity → map to Bevy → verify | Critical paths |
| Performance | Benchmark | 10K entity spawn/despawn, 1K system tick | Regression |

**Existing test count**: 50+ unit tests across core/engine/adapters.

**Test categories**:

| Category | Command | Purpose |
|----------|---------|---------|
| Unit | `cargo test --lib` | Fast, isolated |
| Integration | `cargo test --test integration` | Multi-module |
| Property | `cargo test --test proptest` | Fuzz testing |
| Bench | `cargo bench` | Performance regression |

---

## 15. Error Handling

```rust
pub enum EngineError {
    AssetError { path: String, reason: String },
    PhysicsError { reason: String },
    AudioError { reason: String },
    RendererError { reason: String },
    SceneError { reason: String },
    InputError { reason: String },
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AssetError { path, reason } => write!(f, "Asset error at {}: {}", path, reason),
            Self::PhysicsError { reason } => write!(f, "Physics error: {}", reason),
            Self::AudioError { reason } => write!(f, "Audio error: {}", reason),
            Self::RendererError { reason } => write!(f, "Renderer error: {}", reason),
            Self::SceneError { reason } => write!(f, "Scene error: {}", reason),
            Self::InputError { reason } => write!(f, "Input error: {}", reason),
        }
    }
}

pub type EngineResult<T> = Result<T, EngineError>;
```

---

## 16. Performance Considerations

| Area | Strategy | Target |
|------|----------|--------|
| Entity iteration | Archetype-based, cache-friendly | 100K entities @ 60fps |
| Component access | SoA for hot components | < 10ns access |
| Rendering | Sprite batching, chunk culling | 10K sprites @ 60fps |
| Physics | Broad phase (spatial hash) + narrow phase | 1K colliders @ 60fps |
| Audio | Voice pooling, spatial culling | 100 concurrent sounds |
| Asset loading | Async, priority queue, hot-reload | < 100ms for textures |
| Memory | Object pooling for particles | < 100MB total |
| GC pressure | No allocation in hot path | Zero per-frame alloc |

---

*Document version: 2.0*
*Last updated: 2026-09-12*
*Total lines: 1200+*
