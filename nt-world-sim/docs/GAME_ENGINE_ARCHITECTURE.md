# Game Engine Architecture Patterns — Comprehensive Report

> Research Date: 2026-09-13
> Sources: Game Programming Patterns (Nystrom), Godot Engine, Bevy Engine, Unity DOTS, academic papers (ACM/SAC 2026, arXiv), MDN, LearnOpenGL, MonoGame, Vulkan Documentation, multiple GDC-style architecture analyses
> Purpose: Extract architecture patterns applicable to NeoTrix NT-WORLD simulation capabilities

---

## Table of Contents

1. [Game Loop](#1-game-loop)
2. [Entity Component System (ECS)](#2-entity-component-system-ecs)
3. [State Machine Pattern](#3-state-machine-pattern)
4. [Rendering Architecture](#4-rendering-architecture)
5. [Input System](#5-input-system)
6. [Audio System](#6-audio-system)
7. [Physics & Collision](#7-physics--collision)
8. [Resource Management](#8-resource-management)
9. [Scene Graph](#9-scene-graph)
10. [Cross-Cutting Patterns](#10-cross-cutting-patterns)

---

## 1. Game Loop

The game loop is the heart of every engine. It continuously runs while the game is active: process input → update state → render output.

### 1.1 Fixed Timestep vs Variable Timestep

**Source**: gameprogrammingpatterns.com (Nystrom), Gaffer on Games, MonoGame docs, Unity docs, NovaECS

#### Variable Timestep (Simple)

Each update receives actual elapsed time. Logic scales by delta time.

```rust
// Simple variable timestep
while running {
    let dt = clock.elapsed_since_last_frame();
    process_input();
    update(dt);
    render();
}
```

**Pros**: Simple, smooth on all frame rates.
**Cons**: Non-deterministic physics. Floating-point accumulation errors cause divergence. High-speed objects tunnel through walls on slow frames.

#### Fixed Timestep with Accumulator (Recommended)

The gold standard. Decouples logic rate from display rate.

```rust
const FIXED_DT: f64 = 1.0 / 60.0; // 60 Hz logic
const MAX_FRAME_TIME: f64 = 0.25;  // Spiral of death guard

let mut previous_time = instant::now();
let mut accumulator = 0.0;

while running {
    let current_time = instant::now();
    let mut frame_time = current_time - previous_time;
    previous_time = current_time;

    // Clamp to prevent spiral of death
    if frame_time > MAX_FRAME_TIME {
        frame_time = MAX_FRAME_TIME;
    }

    accumulator += frame_time;

    // Fixed-rate logic updates
    while accumulator >= FIXED_DT {
        process_input();
        update(FIXED_DT);
        accumulator -= FIXED_DT;
    }

    // Interpolation factor for smooth rendering
    let alpha = accumulator / FIXED_DT;
    render(alpha);
}
```

**Key insight from Gaffer on Games**: The accumulator pattern separates update from render. The game simulates at constant rate using fixed steps; the visible frame rate can vary independently.

#### Display Rate vs Logic Rate Table

| Display Rate | Logic Rate | Logic Ticks Per Frame | Visual FPS |
|-------------|-----------|----------------------|------------|
| 60 Hz | 60 Hz | 1 | 60 |
| 144 Hz | 60 Hz | 0 or 1 | 144 |
| 30 Hz | 60 Hz | 2 | 30 |

### 1.2 Spiral of Death Prevention

When a frame takes longer than `FIXED_DT`, multiple update iterations run. If rendering also slows, you get exponential slowdown. Solution: cap `frame_time` to `MAX_FRAME_TIME` (typically 0.25s = 4 Hz minimum logic).

### 1.3 Interpolation for Smooth Rendering

```rust
// Store previous and current state
struct Interpolatable {
    prev_position: Vec2,
    curr_position: Vec2,
}

// Render with interpolation
fn render(entity: &Interpolatable, alpha: f32) {
    let render_pos = entity.prev_position.lerp(entity.curr_position, alpha);
    draw_sprite(render_pos);
}
```

Many 2D games skip interpolation and accept minor visual quantization — stutter is less noticeable with pixel art (GameCodex docs).

---

## 2. Entity Component System (ECS)

**Sources**: ACM SAC 2026 "The Essence of ECS", arXiv 2508.15264 "Concurrency in ECS", Blubber Engine paper (SciTePress 2026), Bevy ECS docs, Unity DOTS, NovaECS, Stranne.EcsArchitecture

### 2.1 Core ECS Concepts

| Concept | Definition | Implementation |
|---------|-----------|----------------|
| **Entity** | Lightweight unique identifier (integer/UUID) | Index into component storage |
| **Component** | Plain data record, no behavior | Struct/POD attached to entities |
| **System** | Stateless function that iterates over entities with required components | Function operating on queries |
| **World** | Container storing all entities, components, and resources | Central data store |
| **Archetype** | Entities with identical component composition grouped together | Cache-friendly columnar storage |
| **Query** | Mechanism to find entities matching component requirements | Filter specification |

### 2.2 ECS in Rust — Bevy Pattern

```rust
use bevy::prelude::*;

// Components are normal Rust structs
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32 }

#[derive(Component)]
struct Sprite {
    texture: Handle<Image>,
    size: Vec2,
}

// Systems are normal Rust functions
fn movement_system(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x * time.delta_seconds();
        pos.y += vel.y * time.delta_seconds();
    }
}

// Resources are global singletons
#[derive(Resource)]
struct GameClock {
    elapsed: f32,
}

// App assembly
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameClock { elapsed: 0.0 })
        .add_systems(Update, movement_system)
        .run();
}
```

### 2.3 Archetype-Based ECS (Academic Foundation)

From "The Essence of Entity Component System" (ACM SAC 2026):

- Entities with identical component sets are grouped into **archetypes**
- Archetypes are dense columnar tables (Structure of Arrays)
- This maximizes spatial locality → cache-friendly iteration
- Archetype ECS achieves higher frame rate and better frame stability than alternatives

**Two ECS data layout approaches**:

| Layout | Modification Cost | Iteration Cost | Best For |
|--------|------------------|----------------|----------|
| **Sparse Set** | Cheaper entity modifications | Slower iteration | Dynamic entities |
| **Archetype** | Expensive (archetype change) | Fast iteration (cache-friendly) | Large-scale simulation |

### 2.4 ECS Concurrency Model

From arXiv 2508.15264:

```
ECS programs are inherently concurrent.
Systems that read different components can run in parallel.
Mutations are synchronized through archetype tables.
```

**Deterministic-by-construction**: A class of Core ECS programs behave deterministically regardless of scheduling. Systems are stateless; mutations are queued and flushed at synchronization points.

### 2.5 ECS Layered Architecture (Production Pattern)

From Stranne.EcsArchitecture:

```
{ProjectName}.sln
├── {ProjectName}.Contracts/    # Cross-layer DTOs and interfaces
├── {ProjectName}.Core/         # Pure ECS game logic (no engine dependency)
├── {ProjectName}.Adapter/      # Core-to-Engine bridge
├── {ProjectName}.Engine/       # Engine-specific implementation
└── {ProjectName}.Core.Tests/   # Unit tests
```

**Design principles**:
- Separation of Concerns: Game logic separated from rendering engine
- Deterministic: Pure ECS logic with seed-based reproducibility
- Engine Agnostic: Core logic works with any rendering engine

### 2.6 Command Buffer Pattern

For deferred mutations (can't modify world during iteration):

```rust
// NovaECS pattern
const SpawnSystem = system('Spawn', (ctx) => {
    const cmd = ctx.commandBuffer;
    const e = cmd.create(true);           // Deferred entity creation
    cmd.add(e, Position, { x: 10, y: 10 });
    cmd.add(e, Velocity, { x: 1, y: 0 });
}).stage('preUpdate')
  .before('set:Gameplay')
  .flushPolicy('afterStage')
  .build();
```

### 2.7 System Scheduling

```
System execution stages:
├── startup    → Run once on first tick
├── preUpdate  → Pre-processing (spawning, input collection)
├── update     → Main game logic (movement, combat)
└── postUpdate → Cleanup, death removal, event processing
```

Systems declare dependencies:
- `.before('set:Gameplay')` — run before this system set
- `.after('Move')` — run after a specific system
- `.inSet('Gameplay')` — belong to a named set
- `.flushPolicy('afterStage')` — when to apply command buffer mutations

---

## 3. State Machine Pattern

**Sources**: gameprogrammingpatterns.com, GDevelop docs, Unity Learn, GameDev.net forums, WebGameDev.com

### 3.1 Finite State Machine (FSM)

The fundamental pattern for game states, character states, and AI behavior.

```rust
// Enum-based FSM (simple approach)
enum GameState {
    Loading,
    Menu,
    Playing,
    Paused,
    GameOver,
}

struct StateMachine {
    current: GameState,
}

impl StateMachine {
    fn transition(&mut self, new_state: GameState) {
        self.exit_current();
        self.current = new_state;
        self.enter_current();
    }

    fn enter_current(&self) {
        match &self.current {
            GameState::Menu => { /* show menu UI */ }
            GameState::Playing => { /* start game loop */ }
            GameState::Paused => { /* freeze updates, show pause overlay */ }
            _ => {}
        }
    }

    fn exit_current(&self) {
        match &self.current {
            GameState::Playing => { /* save state, cleanup */ }
            _ => {}
        }
    }
}
```

### 3.2 State Pattern (OOP Alternative)

```rust
trait State {
    fn enter(&mut self);
    fn update(&mut self, dt: f32);
    fn render(&self);
    fn exit(&mut self);
}

struct PlayingState { /* ... */ }
impl State for PlayingState {
    fn enter(&mut self) { /* initialize level */ }
    fn update(&mut self, dt: f32) { /* game logic */ }
    fn render(&self) { /* draw world */ }
    fn exit(&mut self) { /* cleanup */ }
}

struct StateManager {
    states: Vec<Box<dyn State>>,
}

impl StateManager {
    fn push(&mut self, state: Box<dyn State>) {
        if let Some(current) = self.states.last_mut() {
            current.exit();
        }
        state.enter();
        self.states.push(state);
    }

    fn pop(&mut self) {
        if let Some(mut state) = self.states.pop() {
            state.exit();
        }
        if let Some(current) = self.states.last_mut() {
            current.enter();
        }
    }
}
```

### 3.3 Hierarchical State Machine (HSM)

As states grow, flat FSMs become unmanageable. HSMs group related states:

```
Grounded (parent)
├── Idle
├── Running
├── Crouching
└── Walking

Airborne (parent)
├── Jumping
├── Falling
└── DoubleJumping
```

**Shared transitions**: "fall off ledge" works from any grounded sub-state.

### 3.4 Game Flow FSM

Typical transitions:

```
         load
Initial ──────→ Menu
                  │
              play│abandon
                  ↓
                Playing ←──→ Paused
                  │            │
              lose│quit    quit│
                  ↓            │
                GameOver ──────┘
                   │
                restart
                   ↓
                 Menu
```

### 3.5 State Management with ECS

From GameDev.net discussions: In ECS, game states are typically managed as **Resources** rather than entities:

```rust
#[derive(Resource)]
struct AppState {
    current: GameState,
    transition_queue: Vec<GameState>,
}

// Systems check state before running
fn gameplay_system(
    state: Res<AppState>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    if state.current != GameState::Playing { return; }
    // ... normal ECS logic
}
```

---

## 4. Rendering Architecture

**Sources**: Godot docs, Unity Tilemap docs, MonoGame, LearnOpenGL, Stride docs, MDN, K-State CIS 580 textbook

### 4.1 Sprite Batching

The core optimization for 2D rendering: group sprites sharing the same texture into a single draw call.

```rust
struct SpriteBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    current_texture: Option<TextureId>,
    sort_mode: SpriteSortMode,
}

enum SpriteSortMode {
    Deferred,       // Batch all, draw at once (default)
    Immediate,      // Draw each sprite immediately
    Texture,        // Sort by texture to minimize state changes
    BackToFront,    // Sort by z-order (for transparency)
    FrontToBack,    // Sort by z-depth (for occlusion culling)
}

impl SpriteBatch {
    fn begin(&mut self, sort_mode: SpriteSortMode) {
        self.vertices.clear();
        self.indices.clear();
        self.current_texture = None;
        self.sort_mode = sort_mode;
    }

    fn draw(&mut self, texture: TextureId, pos: Vec2, src_rect: Rect, color: Color) {
        // If sort by texture and texture changed, flush current batch
        if self.sort_mode == SpriteSortMode::Texture
            && self.current_texture != Some(texture)
        {
            self.flush();
        }

        // Add quad vertices and indices
        let base_vertex = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            Vertex { position: pos, uv: src_rect.top_left(), color },
            Vertex { position: pos + Vec2::new(src_rect.w, 0.0), uv: src_rect.top_right(), color },
            Vertex { position: pos + src_rect.size, uv: src_rect.bottom_right(), color },
            Vertex { position: pos + Vec2::new(0.0, src_rect.h), uv: src_rect.bottom_left(), color },
        ]);
        self.indices.extend_from_slice(&[
            base_vertex, base_vertex+1, base_vertex+2,
            base_vertex, base_vertex+2, base_vertex+3,
        ]);
        self.current_texture = Some(texture);
    }

    fn flush(&mut self) {
        if self.vertices.is_empty() { return; }
        // Submit batch to GPU
        gpu_submit(self.current_texture, &self.vertices, &self.indices);
        self.vertices.clear();
        self.indices.clear();
    }

    fn end(&mut self) {
        self.flush();
    }
}
```

**Optimization rules**:
- Sort opaque sprites front-to-back (early z-rejection)
- Sort transparent sprites back-to-front (correct alpha blending)
- Trim transparent sprite boundaries tightly
- Each full-screen post-processing effect adds 100% overdraw

### 4.2 Tile Map Rendering

**Sources**: Godot, Unity, MDN, MonoGame docs

#### Tile Data Structure

```rust
struct TileMap {
    tiles: Vec<Vec<TileId>>,  // 2D grid of tile IDs
    tileset: Tileset,
    quadrant_size: usize,      // Batch chunk size (default 16)
}

struct Tileset {
    atlas: TextureAtlas,       // Single texture containing all tiles
    tile_size: Vec2,           // Size of each tile in pixels
    tiles: Vec<TileData>,      // Per-tile metadata (collision, animation, etc.)
}

struct TileData {
    source_rect: Rect,         // Region in atlas
    collision_shape: Option<CollisionShape>,
    animation_frames: Option<Vec<Rect>>,
}
```

#### Efficient Rendering Algorithm

```rust
fn render_tilemap(
    tilemap: &TileMap,
    camera: &Camera,
    spriteBatch: &mut SpriteBatch,
) {
    // Calculate visible tile range (frustum culling)
    let start_col = (camera.left() / tilemap.tileset.tile_size.x).floor() as usize;
    let end_col = (camera.right() / tilemap.tileset.tile_size.x).ceil() as usize;
    let start_row = (camera.top() / tilemap.tileset.tile_size.y).floor() as usize;
    let end_row = (camera.bottom() / tilemap.tileset.tile_size.y).ceil() as usize;

    spriteBatch.begin(SpriteSortMode::Texture);

    // Render back-to-front for correct overlap
    for row in start_row..=end_row.min(tilemap.tiles.len() - 1) {
        for col in start_col..=end_col.min(tilemap.tiles[0].len() - 1) {
            let tile_id = tilemap.tiles[row][col];
            if tile_id == EMPTY_TILE { continue; }

            let tile_data = &tilemap.tileset.tiles[tile_id as usize];
            let screen_pos = Vec2::new(
                col as f32 * tilemap.tileset.tile_size.x,
                row as f32 * tilemap.tileset.tile_size.y,
            );

            spriteBatch.draw(
                tilemap.tileset.atlas.texture,
                screen_pos,
                tile_data.source_rect,
                Color::WHITE,
            );
        }
    }

    spriteBatch.end();
}
```

#### Tile Map Optimization Techniques

From Unity docs:
- **Single Renderer**: Tilemap uses one Renderer for the whole map (vs. one per sprite). Less overhead.
- **Quadrant Batching**: Groups tiles into chunks (default 16×16) for batched drawing.
- **Reduce Serialization**: Tilemap data is compact (ID + position per tile) vs. full GameObjects.
- **Y-Sort**: Draw tiles in Y-order for correct depth in top-down games.

From MDN:
- **Pre-render chunks**: Split map into sections, pre-render to textures, blit once per frame.
- **Only render visible tiles**: Cull off-screen tiles before issuing draw calls.
- **Static vs Scrolling**: Static maps (Pac-Man, Arkanoid) need no scrolling; scrolling maps need camera-relative rendering.

### 4.3 Camera System

```rust
struct Camera {
    position: Vec2,
    viewport_size: Vec2,
    zoom: f32,
    bounds: Option<Rect>,      // World bounds to clamp within
    target: Option<Entity>,    // Entity to follow
    follow_speed: f32,
    look_ahead: Vec2,          // Offset in movement direction
}

impl Camera {
    fn update(&mut self, dt: f32) {
        if let Some(target_pos) = self.get_target_position() {
            // Smooth follow with lerp
            let desired = target_pos + self.look_ahead;
            let current = self.position;
            self.position = current.lerp(desired, self.follow_speed * dt);

            // Clamp to world bounds
            if let Some(bounds) = self.bounds {
                self.position = self.position.clamp(
                    bounds.top_left() + self.viewport_size * 0.5,
                    bounds.bottom_right() - self.viewport_size * 0.5,
                );
            }
        }
    }

    fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos - self.position) * self.zoom + self.viewport_size * 0.5
    }

    fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.viewport_size * 0.5) / self.zoom + self.position
    }

    fn visible_bounds(&self) -> Rect {
        let half = self.viewport_size * 0.5 / self.zoom;
        Rect::from_center(self.position, half * 2.0)
    }
}
```

### 4.4 Parallax Scrolling

**Source**: Godot Parallax2D, K-State CIS 580, HaxeFlixel

Create illusion of depth by scrolling layers at different speeds.

```rust
struct ParallaxLayer {
    texture: TextureId,
    scroll_factor: f32,    // 1.0 = same as camera, 0.5 = half speed (far), 0.0 = static
    repeat: bool,          // Infinite scrolling
    auto_scroll: Vec2,     // Constant movement (clouds, water)
    offset: Vec2,
}

struct ParallaxBackground {
    layers: Vec<ParallaxLayer>,
}

impl ParallaxBackground {
    fn render(&self, camera: &Camera, spriteBatch: &mut SpriteBatch) {
        for layer in &self.layers {
            let parallax_offset = camera.position * layer.scroll_factor;

            let transform = Mat4::from_translation(Vec3::new(
                -parallax_offset.x + layer.offset.x,
                -parallax_offset.y + layer.offset.y,
                0.0,
            ));

            spriteBatch.begin_with_transform(transform);

            if layer.repeat {
                // Tile the texture to fill viewport
                self.render_tiled(layer, camera, spriteBatch);
            } else {
                spriteBatch.draw(layer.texture, Vec2::ZERO, layer.full_rect(), Color::WHITE);
            }

            spriteBatch.end();
        }
    }
}
```

**Godot Parallax2D properties**:
- `scroll_scale`: Vector2 multiplier (1,1 = normal, 0.5,0.5 = half speed = appears farther)
- `repeat_size`: For infinite scrolling backgrounds
- `autoscroll`: Constant movement independent of camera
- `limit_begin`/`limit_end`: Clamp scrolling range

### 4.5 Particle Systems

**Source**: LearnOpenGL, Vooga engine

```rust
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Color,
    life: f32,          // Remaining lifetime
    max_life: f32,       // Initial lifetime
    size: f32,
}

struct ParticleEmitter {
    particles: Vec<Particle>,
    max_particles: usize,
    spawn_rate: f32,          // Particles per second
    spawn_accumulator: f32,
    texture: TextureId,
    // Emitter properties
    position: Vec2,
    direction: Vec2,
    spread: f32,              // Angle spread in radians
    min_speed: f32,
    max_speed: f32,
    min_life: f32,
    max_life: f32,
    min_size: f32,
    max_size: f32,
    start_color: Color,
    end_color: Color,
    gravity: Vec2,
}

impl ParticleEmitter {
    fn update(&mut self, dt: f32) {
        // Spawn new particles
        self.spawn_accumulator += self.spawn_rate * dt;
        while self.spawn_accumulator >= 1.0 && self.particles.len() < self.max_particles {
            self.spawn_particle();
            self.spawn_accumulator -= 1.0;
        }

        // Update existing particles
        for particle in &mut self.particles {
            particle.life -= dt;
            particle.velocity += self.gravity * dt;
            particle.position += particle.velocity * dt;

            // Interpolate color over lifetime
            let t = 1.0 - (particle.life / particle.max_life);
            particle.color = self.start_color.lerp(self.end_color, t);
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);
    }

    fn spawn_particle(&mut self) {
        let angle = self.direction.angle() + random_range(-self.spread, self.spread);
        let speed = random_range(self.min_speed, self.max_speed);
        let life = random_range(self.min_life, self.max_life);

        self.particles.push(Particle {
            position: self.position,
            velocity: Vec2::from_angle(angle) * speed,
            color: self.start_color,
            life,
            max_life: life,
            size: random_range(self.min_size, self.max_size),
        });
    }

    fn render(&self, spriteBatch: &mut SpriteBatch) {
        for particle in &self.particles {
            let alpha = (particle.life / particle.max_life) * particle.color.a;
            spriteBatch.draw(
                self.texture,
                particle.position - Vec2::splat(particle.size * 0.5),
                self.source_rect,
                Color { a: alpha, ..particle.color },
            );
        }
    }
}
```

---

## 5. Input System

**Sources**: Bevy input module, Godot input, various game dev articles

### 5.1 Input Mapping (Keys to Actions)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    MoveLeft,
    MoveRight,
    Jump,
    Attack,
    Interact,
    Pause,
    MenuConfirm,
    MenuCancel,
}

struct InputMapping {
    key_bindings: HashMap<KeyCode, Action>,
    gamepad_bindings: HashMap<GamepadButton, Action>,
    mouse_bindings: HashMap<MouseButton, Action>,
}

impl InputMapping {
    fn default_rpg() -> Self {
        let mut key_bindings = HashMap::new();
        key_bindings.insert(KeyCode::A, Action::MoveLeft);
        key_bindings.insert(KeyCode::Left, Action::MoveLeft);
        key_bindings.insert(KeyCode::D, Action::MoveRight);
        key_bindings.insert(KeyCode::Right, Action::MoveRight);
        key_bindings.insert(KeyCode::Space, Action::Jump);
        key_bindings.insert(KeyCode::Z, Action::Attack);
        key_bindings.insert(KeyCode::E, Action::Interact);
        key_bindings.insert(KeyCode::Escape, Action::Pause);
        key_bindings.insert(KeyCode::Return, Action::MenuConfirm);
        key_bindings.insert(KeyCode::Back, Action::MenuCancel);
        Self { key_bindings, gamepad_bindings: HashMap::new(), mouse_bindings: HashMap::new() }
    }
}
```

### 5.2 Input State Buffering

```rust
struct InputState {
    current: HashSet<Action>,
    previous: HashSet<Action>,
    just_pressed: HashSet<Action>,
    just_released: HashSet<Action>,
    buffered_actions: VecDeque<(Action, f32)>, // (action, timestamp)
    buffer_duration: f32,                      // e.g., 0.1s
}

impl InputState {
    fn update(&mut self, mapping: &InputMapping, raw_keys: &HashSet<KeyCode>, time: f32) {
        // Save previous frame
        self.previous = self.current.clone();

        // Map raw keys to actions
        self.current.clear();
        for key in raw_keys {
            if let Some(action) = mapping.key_bindings.get(key) {
                self.current.insert(*action);
            }
        }

        // Compute edge detection
        self.just_pressed = self.current.difference(&self.previous).cloned().collect();
        self.just_released = self.previous.difference(&self.current).cloned().collect();

        // Add pressed actions to buffer
        for action in &self.just_pressed {
            self.buffered_actions.push_back((*action, time));
        }

        // Expire old buffered actions
        while let Some((_, timestamp)) = self.buffered_actions.front() {
            if time - timestamp > self.buffer_duration {
                self.buffered_actions.pop_front();
            } else {
                break;
            }
        }
    }

    fn is_down(&self, action: Action) -> bool { self.current.contains(&action) }
    fn just_pressed(&self, action: Action) -> bool { self.just_pressed.contains(&action) }
    fn just_released(&self, action: Action) -> bool { self.just_released.contains(&action) }

    fn consume_buffered(&mut self, action: Action) -> bool {
        if let Some(pos) = self.buffered_actions.iter().position(|(a, _)| *a == action) {
            self.buffered_actions.remove(pos);
            true
        } else {
            false
        }
    }
}
```

**Input buffering** is critical for responsive controls: if the player presses jump slightly before landing, the jump should still execute on landing.

### 5.3 Touch/Mouse Handling

```rust
struct TouchState {
    touches: Vec<Touch>,
    mouse_position: Vec2,
    mouse_buttons: HashSet<MouseButton>,
}

struct Touch {
    id: usize,
    position: Vec2,
    start_position: Vec2,
    phase: TouchPhase, // Began, Moved, Ended, Cancelled
}

enum TouchPhase {
    Began,
    Moved,
    Ended,
    Cancelled,
}

impl TouchState {
    fn swipe_direction(&self) -> Option<Vec2> {
        self.touches.first().map(|t| {
            let delta = t.position - t.start_position;
            if delta.length() > SWIPE_THRESHOLD {
                delta.normalize()
            } else {
                Vec2::ZERO
            }
        })
    }
}
```

---

## 6. Audio System

**Sources**: Godot audio system docs, Unreal Engine audio docs, Godot Core System audio_system.md

### 6.1 Sound Manager Architecture

```rust
enum AudioCategory {
    Music,
    SFX,
    Voice,
    Ambient,
}

struct AudioManager {
    music_player: AudioStreamPlayer,
    sfx_pool: SoundPool,
    category_volumes: HashMap<AudioCategory, f32>,
    master_volume: f32,
    current_music: Option<AudioSource>,
}

impl AudioManager {
    fn play_music(&mut self, source: AudioSource, fade_duration: f32) {
        if let Some(current) = &self.current_music {
            // Crossfade: fade out current, fade in new
            self.music_player.fade_out(current, fade_duration);
        }
        self.music_player.fade_in(source.clone(), fade_duration);
        self.current_music = Some(source);
    }

    fn play_sfx(&mut self, source: AudioSource) {
        self.sfx_pool.play(source);
    }

    fn set_volume(&mut self, category: AudioCategory, volume: f32) {
        self.category_volumes.insert(category, volume);
        self.apply_volumes();
    }

    fn apply_volumes(&self) {
        for (category, vol) in &self.category_volumes {
            let effective = vol * self.master_volume;
            // Apply to relevant audio streams
        }
    }
}
```

### 6.2 Audio Pooling

Pre-allocate audio instances to avoid runtime allocation.

```rust
struct SoundPool {
    available: Vec<AudioStreamPlayer>,
    active: Vec<AudioStreamPlayer>,
    max_instances: usize,
}

impl SoundPool {
    fn new(max_instances: usize) -> Self {
        let available = (0..max_instances)
            .map(|_| AudioStreamPlayer::new())
            .collect();
        Self { available, active: Vec::new(), max_instances }
    }

    fn play(&mut self, source: AudioSource) {
        let player = if let Some(p) = self.available.pop() {
            p
        } else if self.active.len() < self.max_instances {
            AudioStreamPlayer::new()
        } else {
            // Steal oldest active player
            self.active.remove(0)
        };

        player.set_source(source);
        player.play();
        self.active.push(player);
    }

    fn update(&mut self) {
        // Move finished players back to available pool
        self.active.retain(|p| {
            if p.is_playing() { true }
            else { self.available.push(p.clone()); false }
        });
    }
}
```

### 6.3 Spatial Audio

For 2D games, spatial audio adjusts volume based on distance from listener:

```rust
fn spatial_audio_system(
    listener_pos: Vec2,
    query: Query<(&Position, &AudioSource)>,
    audio: &mut AudioManager,
) {
    for (pos, source) in &query {
        let distance = pos.distance_to(listener_pos);
        let volume = (1.0 - (distance / MAX_AUDIO_DISTANCE).clamp(0.0, 1.0))
            .powi(2); // Quadratic falloff

        // Pan based on relative X position
        let pan = ((pos.x - listener_pos.x) / MAX_AUDIO_DISTANCE).clamp(-1.0, 1.0);

        audio.set_spatial_volume(source, volume, pan);
    }
}
```

---

## 7. Physics & Collision

**Sources**: LearnOpenGL collision detection, MDN 3D collision, gameprogrammingpatterns.com

### 7.1 AABB Collision Detection

The fastest 2D collision test — axis-aligned bounding boxes.

```rust
struct AABB {
    min: Vec2,  // Top-left
    max: Vec2,  // Bottom-right
}

impl AABB {
    fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self { min: pos, max: pos + size }
    }

    fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
    }

    fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
    }
}
```

### 7.2 Circle Collision

```rust
fn circles_collide(pos_a: Vec2, radius_a: f32, pos_b: Vec2, radius_b: f32) -> bool {
    let distance_sq = pos_a.distance_squared_to(pos_b);
    let radii_sum = radius_a + radius_b;
    distance_sq <= radii_sum * radii_sum
}
```

### 7.3 Tile-Based Collision

```rust
fn check_tile_collision(
    entity: &AABB,
    tilemap: &TileMap,
    velocity: Vec2,
) -> CollisionResult {
    let mut result = CollisionResult::default();

    // Calculate tile range to check
    let expanded = AABB::from_pos_size(
        entity.min + velocity.min(Vec2::ZERO),
        entity.max - entity.min + velocity.abs(),
    );

    let start_col = (expanded.min.x / tilemap.tile_size.x).floor() as i32;
    let end_col = (expanded.max.x / tilemap.tile_size.x).ceil() as i32;
    let start_row = (expanded.min.y / tilemap.tile_size.y).floor() as i32;
    let end_row = (expanded.max.y / tilemap.tile_size.y).ceil() as i32;

    for row in start_row..=end_row {
        for col in start_col..=end_col {
            let tile = tilemap.get_tile(col, row);
            if tile.is_none() || !tile.unwrap().has_collision { continue; }

            let tile_aabb = AABB::from_pos_size(
                Vec2::new(col as f32 * tilemap.tile_size.x, row as f32 * tilemap.tile_size.y),
                tilemap.tile_size,
            );

            if entity.intersects(&tile_aabb) {
                result.collisions.push(TileCollision {
                    tile_pos: IVec2::new(col, row),
                    tile_aabb,
                    // Calculate overlap for resolution
                    overlap: calculate_overlap(entity, &tile_aabb),
                });
            }
        }
    }

    result
}

fn calculate_overlap(a: &AABB, b: &AABB) -> Vec2 {
    let overlap_x = (a.max.x - b.min.x).min(b.max.x - a.min.x);
    let overlap_y = (a.max.y - b.min.y).min(b.max.y - a.min.y);
    Vec2::new(overlap_x, overlap_y)
}
```

### 7.4 Simple Physics (Gravity, Velocity)

```rust
const GRAVITY: f32 = 980.0;  // pixels/s²
const TERMINAL_VELOCITY: f32 = 600.0;

#[derive(Component)]
struct PhysicsBody {
    velocity: Vec2,
    on_ground: bool,
    mass: f32,
}

fn physics_system(
    dt: f32,
    mut bodies: Query<(&mut Position, &mut PhysicsBody)>,
) {
    for (mut pos, mut body) in &mut bodies {
        // Apply gravity
        if !body.on_ground {
            body.velocity.y = (body.velocity.y + GRAVITY * dt).min(TERMINAL_VELOCITY);
        }

        // Apply velocity
        pos.x += body.velocity.x * dt;
        pos.y += body.velocity.y * dt;
    }
}
```

### 7.5 Collision Response

```rust
fn resolve_collision(entity: &mut PhysicsBody, collision: &TileCollision, dt: f32) {
    let overlap = collision.overlap;

    // Determine minimum penetration axis
    if overlap.x < overlap.y {
        // Horizontal resolution
        if entity.velocity.x > 0.0 {
            // Moving right, push left
        } else {
            // Moving left, push right
        }
        entity.velocity.x = 0.0;
    } else {
        // Vertical resolution
        if entity.velocity.y > 0.0 {
            // Moving down (landing)
            entity.on_ground = true;
        } else {
            // Moving up (hitting ceiling)
        }
        entity.velocity.y = 0.0;
    }
}
```

---

## 8. Resource Management

**Sources**: Vulkan docs, Unity docs, Cocos Asset Manager, GameDev.net

### 8.1 Resource Manager Pattern

```rust
struct ResourceManager {
    cache: HashMap<String, Arc<dyn Resource>>,
    ref_counts: HashMap<String, usize>,
}

trait Resource: Send + Sync + 'static {
    fn load(path: &str) -> Result<Self> where Self: Sized;
    fn as_any(&self) -> &dyn Any;
}

impl ResourceManager {
    fn load<T: Resource>(&mut self, path: &str) -> Arc<T> {
        // Check cache
        if let Some(cached) = self.cache.get(path) {
            *self.ref_counts.get_mut(path).unwrap() += 1;
            return Arc::clone(cached).downcast::<T>().unwrap();
        }

        // Load new resource
        let resource = Arc::new(T::load(path).expect("Failed to load resource"));
        self.cache.insert(path.to_string(), resource.clone() as Arc<dyn Resource>);
        self.ref_counts.insert(path.to_string(), 1);
        resource.downcast::<T>().unwrap()
    }

    fn release(&mut self, path: &str) {
        if let Some(count) = self.ref_counts.get_mut(path) {
            *count -= 1;
            if *count == 0 {
                self.cache.remove(path);
                self.ref_counts.remove(path);
            }
        }
    }
}
```

### 8.2 Object Pooling

Pre-allocate and reuse objects to avoid allocation spikes.

```rust
struct ObjectPool<T> {
    available: Vec<T>,
    active_count: usize,
    factory: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    fn new(initial_size: usize, factory: impl Fn() -> T + 'static) -> Self {
        let available = (0..initial_size).map(|_| factory()).collect();
        Self { available, active_count: 0, factory: Box::new(factory) }
    }

    fn acquire(&mut self) -> T {
        self.active_count += 1;
        if let Some(obj) = self.available.pop() {
            obj
        } else {
            (self.factory)()  // Pool exhausted, create new
        }
    }

    fn release(&mut self, obj: T) {
        self.active_count -= 1;
        self.available.push(obj);
    }

    fn prewarm(&mut self, count: usize) {
        for _ in 0..count {
            self.available.push((self.factory)());
        }
    }
}

// Usage for projectiles
let bullet_pool = ObjectPool::new(100, || Bullet {
    position: Vec2::ZERO,
    velocity: Vec2::ZERO,
    active: false,
});
```

### 8.3 Asset Bundle Pattern

Partition resources for modular loading:

```rust
struct AssetBundle {
    name: String,
    assets: HashMap<String, Box<dyn Any>>,
    loaded: bool,
}

impl AssetBundle {
    async fn load(name: &str) -> Result<Self> {
        // Load bundle manifest
        let manifest = load_manifest(name).await?;
        let mut assets = HashMap::new();

        // Preload critical assets synchronously
        for asset_path in &manifest.critical_assets {
            let asset = load_asset(asset_path).await?;
            assets.insert(asset_path.clone(), asset);
        }

        Ok(Self { name: name.to_string(), assets, loaded: true })
    }

    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.assets.get(key)?.downcast_ref::<T>()
    }
}
```

---

## 9. Scene Graph

**Sources**: Godot scene system, academic papers on engine architecture

### 9.1 Scene Tree Structure

```
SceneTree
├── Root (Node2D)
│   ├── TileMap (background)
│   ├── Entities (Node2D)
│   │   ├── Player
│   │   │   ├── Sprite
│   │   │   ├── Collider
│   │   │   └── AnimationPlayer
│   │   ├── Enemy_01
│   │   └── Enemy_02
│   ├── Foreground (Node2D)
│   └── UI (CanvasLayer)
│       ├── HealthBar
│       └── ScoreDisplay
```

### 9.2 Node Hierarchy

```rust
struct Node2D {
    name: String,
    position: Vec2,
    rotation: f32,
    scale: Vec2,
    visible: bool,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl Node2D {
    fn global_position(&self, tree: &SceneTree) -> Vec2 {
        let local = self.position;
        match self.parent {
            Some(parent_id) => {
                let parent = tree.get(parent_id);
                parent.global_position(tree) + local.rotated(parent.global_rotation(tree))
            }
            None => local,
        }
    }
}
```

### 9.3 Scene Composition

Godot-style: Scenes are composable. A "Player" scene contains its own node tree, can be instanced into any other scene.

```rust
struct Scene {
    root: NodeId,
    nodes: SlotMap<NodeId, Node2D>,
}

impl Scene {
    fn instantiate(&self, parent: &mut Scene) -> NodeId {
        // Deep copy node tree, remap IDs
        let new_root = self.clone_into(parent);
        // Connect signals, set up groups
        new_root
    }
}
```

---

## 10. Cross-Cutting Patterns

### 10.1 Event System (Publish/Subscribe)

Decouple modules through events:

```rust
enum GameEvent {
    EntityDamaged { entity: Entity, damage: f32 },
    PlayerDied { entity: Entity },
    ItemCollected { entity: Entity, item: ItemId },
    LevelCompleted { level: usize },
}

struct EventBus {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&GameEvent)>>>,
}

impl EventBus {
    fn subscribe<T: Fn(&GameEvent) + 'static>(&mut self, handler: T) {
        // Register handler
    }

    fn publish(&self, event: &GameEvent) {
        // Notify all relevant listeners
    }
}
```

### 10.2 Component Categories (ECS Taxonomy)

From Stranne.EcsArchitecture:

| Category | Purpose | Examples |
|----------|---------|----------|
| **Singleton** | One per game instance | `GameState`, `AudioManager`, `Camera` |
| **Value** | Data attached to entities | `Position`, `Velocity`, `Health`, `Sprite` |
| **Tag** | Marker components (no data) | `Player`, `Enemy`, `Static`, `Dirty` |
| **Buffer** | Frame-lagged data | `PreviousPosition`, `InputSnapshot` |

### 10.3 Time Management

```rust
struct Time {
    delta: f32,           // Frame time
    elapsed: f64,         // Total time
    time_scale: f32,      // 1.0 = normal, 0.0 = paused, 0.5 = slow-mo
    fixed_dt: f32,        // Physics timestep
}

impl Time {
    fn scaled_delta(&self) -> f32 {
        self.delta * self.time_scale
    }
}
```

### 10.4 Debug Rendering

```rust
#[cfg(debug_assertions)]
fn debug_render(world: &World, camera: &Camera, spriteBatch: &mut SpriteBatch) {
    // Draw AABBs
    for (pos, aabb) in world.query::<(&Position, &AABB)>() {
        spriteBatch.draw_rect_debug(
            camera.world_to_screen(aabb.min),
            aabb.max - aabb.min,
            Color::GREEN.alpha(0.3),
        );
    }

    // Draw velocity vectors
    for (pos, vel) in world.query::<(&Position, &Velocity)>() {
        let start = camera.world_to_screen(*pos);
        let end = start + *vel * 0.05;
        spriteBatch.draw_line_debug(start, end, Color::RED);
    }

    // Draw tile grid
    for row in 0..tilemap.rows {
        for col in 0..tilemap.cols {
            let screen_pos = camera.world_to_screen(Vec2::new(
                col as f32 * TILE_SIZE, row as f32 * TILE_SIZE
            ));
            spriteBatch.draw_rect_debug(screen_pos, Vec2::splat(TILE_SIZE), Color::WHITE.alpha(0.1));
        }
    }
}
```

---

## Summary: Architecture Decision Matrix

| Pattern | Recommended Approach | Key Trade-off |
|---------|---------------------|---------------|
| **Game Loop** | Fixed timestep + accumulator + interpolation | Determinism vs visual smoothness |
| **ECS** | Archetype-based (Bevy/DOTS style) | Iteration speed vs modification cost |
| **State** | Hierarchical FSM or enum-based | Complexity vs flexibility |
| **Rendering** | Sprite batch + texture sorting | Draw call count vs sort overhead |
| **Tile Map** | Quadrant batching + frustum culling | Memory vs draw calls |
| **Camera** | Smooth follow + world clamping | Latency vs responsiveness |
| **Input** | Action mapping + buffering | Responsiveness vs complexity |
| **Audio** | Pool-based + category mixing | Memory vs allocation spikes |
| **Physics** | AABB + tile-based resolution | Accuracy vs performance |
| **Resources** | Reference-counted cache + pooling | Memory vs load time |

---

## References

1. Nystrom, R. "Game Programming Patterns" — gameprogrammingpatterns.com
2. Tasnim et al. "The Essence of Entity Component System" — ACM SAC 2026
3. "Exploring the Theory and Practice of Concurrency in ECS" — arXiv 2508.15264
4. Atwi & Sharafeddin "A Data-Oriented ECS Architecture for Real-Time Game Engines" — SciTePress 2026
5. "Visualising Game Engine Subsystem Coupling Patterns" — arXiv 2309.06329
6. Godot Engine docs — godotengine.org
7. Bevy Engine docs — bevy.org / docs.rs/bevy
8. Unity Manual — docs.unity3d.com
9. MonoGame docs — docs.monogame.net
10. MDN "Tiles and tilemaps overview" — developer.mozilla.org
11. LearnOpenGL collision detection — learnopengl.com
12. Vulkan Tutorial: Resource Management — docs.vulkan.org
13. Stranne.EcsArchitecture — github.com/stranne
14. NovaECS — github.com/esengine/NovaECS
