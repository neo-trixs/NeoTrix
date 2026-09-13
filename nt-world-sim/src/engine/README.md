# NeoTrix Game Engine

A modular 2D game engine built in Rust with ECS, physics, rendering, audio, and full game systems.

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│  Game (top-level orchestrator)                               │
│  ├── GameEngine (state machine + fixed-timestep loop)        │
│  ├── World (ECS: entities + components + resources)          │
│  ├── SystemRunner (priority-ordered system execution)        │
│  ├── CanvasRenderer (draw command backend)                   │
│  ├── AudioManager (spatial audio + crossfade)                │
│  └── TypedEventBus (decoupled inter-system communication)    │
├──────────────────────────────────────────────────────────────┤
│  ECS Layer                                                   │
│  ├── Entity (generational arena IDs)                         │
│  ├── Components (Transform, Velocity, Health, Collider...)   │
│  ├── Systems (Movement, Collision, Camera, Render, AI)       │
│  └── Resources (GameCamera, TimeState, RenderCommandBuffer)  │
├──────────────────────────────────────────────────────────────┤
│  Subsystems                                                  │
│  ├── Physics (rigid bodies, colliders, AABB resolution)      │
│  ├── Map (multi-layer tile maps, auto-tiling, fog of war)    │
│  ├── Collision (tile-based collision, raycasting)            │
│  ├── Combat (damage calc, status effects, AI, loot)          │
│  ├── Dialogue (tree-based dialogue with conditions)          │
│  ├── Quest (objectives, rewards, prerequisites)              │
│  ├── NPC (AI behaviors, schedules, relationships)            │
│  ├── Inventory (items, equipment, rarity)                    │
│  ├── Skill (skills, affixes, cooldowns)                      │
│  ├── Effects (floating numbers, screen shake, particles)     │
│  ├── Save/Load (file/memory backends, auto-save)             │
│  └── Perf (spatial hashing, viewport culling, profiling)     │
├──────────────────────────────────────────────────────────────┤
│  Math Primitives (Vec2, Rect, Color, Transform)              │
└──────────────────────────────────────────────────────────────┘
```

## Module Descriptions

### Core Modules

| Module | Purpose |
|--------|---------|
| `core` | Math primitives: `Vec2`, `Rect`, `Color`, `Transform`. Single fact source for all math. |
| `ecs` | Entity-Component-System: `Entity`, `World`, `Component` trait, `System` trait, `SystemRunner`. |
| `renderer` | Rendering pipeline: `Sprite`, `TileMap`, `SpriteBatch`, `ParticleSystem`, `GameRenderer`. |
| `input` | Input handling: `KeyCode`, `MouseButton`, `GamepadAxis`, `InputState`, `InputProvider` trait. |
| `camera` | Camera system: `Camera2D` with follow, shake, bounds, fade. |
| `asset` | Asset management: `AssetServer` for textures, sounds, fonts with typed handles. |
| `event_bus` | Typed event bus: `TypedEventBus` with priority handlers for decoupled communication. |
| `audio` | Audio system: `AudioManager` with spatial audio, crossfader, sound pools, triggers. |
| `scene` | Scene graph: `SceneGraph` with hierarchical transforms, dirty flags, queries. |
| `physics` | Physics simulation: `RigidBody`, `Collider`, `SimplePhysicsWorld` with collision detection. |

### Game Systems

| Module | Purpose |
|--------|---------|
| `map` | Tile maps: multi-layer `TileMap`, `AutoTileSystem`, `FogOfWar`, `MinimapRenderer`. |
| `collision` | Tile collision: `TileCollisionSystem` with AABB overlap, raycast, movement resolution. |
| `mapgen` | Procedural generation: `ValueNoise`, biomes, region templates, `MapGenerator`. |
| `combat` | Combat system: `CombatManager`, damage calculation, status effects, loot tables. |
| `dialogue` | Dialogue system: `DialogueTree`, choices, conditions, effects, portraits. |
| `quest` | Quest system: `QuestManager`, objectives, rewards, prerequisites, journal. |
| `npc` | NPC system: `NPCManager`, AI behaviors, schedules, relationships, events. |
| `inventory` | Inventory: items, equipment slots, rarity, use effects. |
| `skill` | Skill system: skills, affixes, cooldowns, skill manager. |
| `components` | ECS components: `Transform`, `Velocity`, `Health`, `Collider`, marker components. |
| `systems` | ECS systems: Movement, Collision, Camera, Health, Render, AI systems. |

### Support Modules

| Module | Purpose |
|--------|---------|
| `ui` | UI widgets: panels, buttons, bars, inventory grid, minimap, dialogue box. |
| `effects` | Visual effects: floating numbers, skill effects, screen shake. |
| `save` | Save/load: file/memory backends, auto-save, game state serialization. |
| `perf` | Performance: spatial hashing, viewport culling, object pooling, profiling. |
| `resources` | Generic resource manager with reference counting and typed wrappers. |
| `game` | Top-level `Game` struct tying all subsystems together. |

## Naming Conventions

### Type Disambiguation

Several modules define similar type names. The engine uses renames in `mod.rs` to avoid conflicts:

| Original Type | Module | Re-exported As | Reason |
|---------------|--------|----------------|--------|
| `physics::Collider` | `physics` | `PhysicsCollider` | Avoids conflict with `components::Collider` |
| `map::TileMap` | `map` | `GameTileMap` | Avoids conflict with `renderer::TileMap` |
| `components::Transform` | `components` | `EcsTransform` | Avoids conflict with `core::Transform` |
| `effects::SkillEffect` | `effects` | `EffectSkillEffect` | Avoids conflict with `skill::SkillEffect` |

### ECS Conventions

- **Components**: Named after what they represent (`Health`, `Velocity`, `PlayerMarker`)
- **Systems**: Suffix with `System` (`MovementSystem`, `CameraSystem`)
- **Resources**: Named after their purpose (`GameCamera`, `TimeState`, `RenderCommandBuffer`)
- **Markers**: Zero-size components for archetype queries (`PlayerMarker`, `NpcMarker`)

### Priority Ordering

Systems run in priority order (lower = earlier):

```
PRIORITY_INPUT      = -200
PRIORITY_AI         = -100
PRIORITY_PHYSICS    = -50
PRIORITY_GAME_LOGIC = 0
PRIORITY_COLLISION  = 10
PRIORITY_CAMERA     = 20
PRIORITY_EFFECTS    = 50
PRIORITY_RENDER     = 100
PRIORITY_UI         = 200
```

## Usage Examples

### Basic Game Setup

```rust
use nt_world_sim::engine::{Game, GameConfig, GamePhase};

let config = GameConfig {
    title: "My Game".into(),
    width: 1280.0,
    height: 720.0,
    tick_rate: 60.0,
    ..Default::default()
};

let mut game = Game::new(config);
game.new_game(0, "Player1");

// Game loop
game.tick(1.0 / 60.0);
game.render(0.0);
```

### ECS Usage

```rust
use nt_world_sim::engine::{World, Entity, Component};

#[derive(Debug)]
struct Position { x: f32, y: f32 }
impl Component for Position {}

#[derive(Debug)]
struct Velocity { vx: f32, vy: f32 }
impl Component for Velocity {}

let mut world = World::new();
let entity = world.spawn();
world.insert(entity, Position { x: 0.0, y: 0.0 });
world.insert(entity, Velocity { vx: 1.0, vy: 0.0 });

// Query entities with both components
let moving = world.query2::<Position, Velocity>();
```

### Custom System

```rust
use nt_world_sim::engine::{System, World};

struct MySystem;

impl System for MySystem {
    fn name(&self) -> &str { "MySystem" }
    fn priority(&self) -> i32 { 0 }

    fn run(&mut self, world: &mut World, dt: f32) {
        let entities = world.query2::<Position, Velocity>();
        for e in entities {
            let vx = world.get::<Velocity>(e).unwrap().vx;
            let pos = world.get_mut::<Position>(e).unwrap();
            pos.x += vx * dt;
        }
    }
}
```

### Physics

```rust
use nt_world_sim::engine::{
    SimplePhysicsWorld, RigidBody, PhysicsCollider, PhysicsWorld,
    PhysicsEntity, Vec2,
};

let mut physics = SimplePhysicsWorld::new();
let entity = PhysicsEntity(0);
let body = RigidBody::dynamic();
let collider = PhysicsCollider::aabb(Vec2::new(16.0, 16.0));

physics.add_body(entity, body);
physics.add_collider(entity, collider);
physics.step(1.0 / 60.0);
```

### Audio

```rust
use nt_world_sim::engine::{AudioManager, SoundCategory, AudioTrigger};

let mut audio = AudioManager::new();
// Register backend, load sounds, play with categories
audio.set_master_volume(0.8);
audio.set_category_volume(SoundCategory::Music, 0.6);
audio.crossfade_music("battle_theme.ogg");
```

### Save/Load

```rust
use nt_world_sim::engine::{EngineSaveManager, FileSaveBackend, SaveGameState};

let backend = FileSaveBackend::new("saves/");
let mut mgr = EngineSaveManager::with_auto_save(Box::new(backend), 300.0);

// Create new game
let state = EngineSaveManager::new_game(0, "Hero");
mgr.save(0, &state)?;

// Load
let loaded = mgr.load(0)?;
```

## Integration Guide

### Platform Backends

The engine outputs `DrawCommand` enums. Platform backends consume these:

```rust
use nt_world_sim::engine::{DrawCommand, CanvasRenderer};

let mut renderer = CanvasRenderer::new(800.0, 600.0);
// ... game renders to canvas ...
let commands: Vec<DrawCommand> = renderer.drain_commands();

for cmd in commands {
    match cmd {
        DrawCommand::DrawSprite { texture, dest, .. } => { /* render sprite */ }
        DrawCommand::DrawRect { rect, color } => { /* render rect */ }
        DrawCommand::DrawText { text, position, .. } => { /* render text */ }
        DrawCommand::Present => { /* flip buffer */ }
        _ => {}
    }
}
```

### Tauri Integration

The engine supports Tauri desktop integration via the `tauri` feature flag:

```toml
[dependencies]
nt-world-sim = { path = ".", features = ["tauri"] }
```

### Event System

```rust
use nt_world_sim::engine::{TypedEventBus, GameEvent};
use std::any::Any;

#[derive(Debug)]
struct DamageEvent { amount: i32 }
impl GameEvent for DamageEvent {
    fn as_any(&self) -> &dyn Any { self }
}

let mut bus = TypedEventBus::new();
bus.send(DamageEvent { amount: 25 });
bus.process_all();
```

## File Structure

```
src/engine/
├── mod.rs              # Module declarations and re-exports
├── core.rs             # GameState, StateStack, GameEngine, LoopTimer
├── ecs.rs              # Entity, World, Component, System, SystemRunner
├── renderer.rs         # Sprite, TileMap, Camera, DrawCommand, GameRenderer
├── input.rs            # KeyCode, InputState, InputProvider
├── camera.rs           # Camera2D with follow/shake/bounds
├── asset.rs            # AssetServer, TextureData, SoundData
├── event_bus.rs        # TypedEventBus, GameEventHandler
├── audio.rs            # AudioManager, SpatialAudio, Crossfader
├── scene.rs            # SceneGraph, SceneNode
├── physics.rs          # RigidBody, Collider, PhysicsWorld
├── map.rs              # TileMap, MapLayer, AutoTile, FogOfWar
├── collision.rs        # AABB, TileCollisionSystem
├── mapgen.rs           # MapGenerator, ValueNoise, Biome
├── combat.rs           # CombatManager, StatusEffect, LootTable
├── dialogue.rs         # DialogueTree, DialogueRunner
├── quest.rs            # QuestManager, Quest, Objective
├── npc.rs              # NPCManager, NPC, NPCBehavior
├── inventory.rs        # Item, Inventory, EquipSlot
├── skill.rs            # Skill, SkillManager, SkillAffix
├── components.rs       # ECS components (Transform, Velocity, Health...)
├── systems.rs          # ECS systems (Movement, Collision, Render...)
├── ui.rs               # UI widgets (Panel, Button, Bar...)
├── effects.rs          # FloatingNumber, ScreenShake, EffectsRenderer
├── save.rs             # SaveBackend, EngineSaveManager
├── perf.rs             # SpatialHashGrid, PerfAggregator
├── resources.rs        # ResourceManager, ResourceHandle
├── game.rs             # Game struct (top-level orchestrator)
├── input_map.rs        # Input mapping utilities
├── sprite_batch.rs     # SpriteBatch for draw call optimization
├── particle.rs         # ParticlePool, Emitter
└── debug_overlay.rs    # DebugRenderer, FPS counter
```
