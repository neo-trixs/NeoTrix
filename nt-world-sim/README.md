# nt-world-sim

Universal Game Engine Abstraction Layer for NeoTrix.

## Overview

nt-world-sim provides a universal Entity-Component-System (ECS) abstraction with built-in physics, input, rendering, and event systems. It supports game definition via YAML/JSON with code generation for **Bevy**, **Unity DOTS**, **Godot 4**, and **Unreal Engine 5**.

## Architecture

```
src/
├── lib.rs                   # Crate root — re-exports + GameEngine facade
├── ecs/                     # Universal ECS core
│   ├── world.rs             # World, Entity, Component
│   └── system.rs            # System, SystemScheduler
├── core/                    # Extended abstractions
│   ├── entity.rs            # UniversalEntity, Archetype, Chunk, SoAStorage
│   ├── world.rs             # UniversalWorld, Resource, Event, ComponentTuple
│   ├── change_detection.rs  # Changed<T>, ChangeTick, ChangeDetector
│   └── scheduler.rs         # ParallelScheduler, UniversalSystem, SystemDependency
├── engine/                  # Game engine subsystems
│   ├── renderer.rs          # Renderer, CanvasRenderer, Color, Vec2, Rect, Transform, Sprite
│   ├── physics.rs           # RigidBody, Collider, PhysicsWorld, SimplePhysicsWorld
│   ├── input.rs             # InputState, KeyCode, MouseButton, GamepadAxis, InputProvider
│   ├── events.rs            # EventBus, EventHandler, MoveEvent, CollisionEvent, GameEvent
│   ├── pet_state.rs         # PetState (12-state FSM), PetStateSystem, EyeTracking
│   ├── hook_system.rs       # HookEvent, HookManager, HookConfig
│   └── theme_system.rs      # ThemeConfig, ThemeManager, ThemeVariant, AnimationDef
├── mechanics/               # High-level game mechanics
│   ├── core_pet.rs          # PetStateComponent, PermissionBubble, ZzzParticle
│   ├── core_hook.rs         # PermissionMode, PermissionRequest, PermissionHotkeys
│   └── core_theme.rs        # SpriteSheet, AnimationPlayer, ShadowDef
├── adapters/                # Engine-specific adapters
│   ├── bevy_adapter.rs      # Bevy ECS adapter
│   ├── unity_adapter.rs     # Unity DOTS adapter
│   ├── godot_adapter.rs     # Godot 4 adapter
│   └── unreal_adapter.rs    # Unreal Engine 5 adapter
└── codegen/                 # Code generation
    ├── game_def.rs          # GameDefinition, EntityDef, EngineConfig
    ├── parser.rs            # GameDefParser (YAML/JSON)
    └── generator.rs         # CodeGenerator (Bevy/Unity/Godot output)
```

## Quick Start

```rust
use nt_world_sim::{
    World, Entity, System, SystemScheduler,
    engine::{TransformComponent, RenderComponent, Vec2, GameEngine, CanvasRenderer},
    mechanics::{ConsciousnessEntity, MaslowNeeds, MaslowSystem, AiSystem},
};

fn main() {
    let renderer = Box::new(CanvasRenderer::new(800.0, 600.0));
    let mut engine = GameEngine::new(renderer);

    // Add systems
    engine.add_system(Box::new(MaslowSystem));
    engine.add_system(Box::new(AiSystem));

    // Spawn an entity
    let player = engine.spawn();
    engine.add_component(player, ConsciousnessEntity::new("core"));
    engine.add_component(player, TransformComponent::new(Vec2::new(300.0, 300.0)));
    engine.add_component(player, RenderComponent::new("🧠", "#1565c0"));
    engine.add_component(player, MaslowNeeds::new());

    // Game loop
    engine.update(0.016);
    engine.render();
}
```

## Stardew Valley Demo

```rust
use nt_world_sim::create_stardew_valley_game;

let mut engine = create_stardew_valley_game();
// Pre-configured with 7 consciousness entities, pet/hook/theme systems
loop {
    engine.update(0.016);
    engine.render();
}
```

## Game Definition Format

Create a `game.yaml` to define your game:

```yaml
name: MyGame
version: "0.1.0"
engine:
  target: bevy
  features: [2d, physics, audio]

entities:
  Player:
    components: [Transform, Sprite, PlayerController]
    systems: [PlayerInputSystem, MovementSystem]

systems:
  MovementSystem:
    priority: 50
    read: [Transform, Velocity]
    write: [Transform]
```

Generate code from the definition:

```rust
use nt_world_sim::codegen::{GameDefParser, CodeGenerator};

let game_def = GameDefParser::parse("game.yaml".as_ref()).unwrap();
let generator = CodeGenerator::new(game_def);

// Generate engine-specific code
let bevy_code = generator.generate_bevy();
let unity_code = generator.generate_unity();
let godot_code = generator.generate_godot();
```

## Core Modules

### ECS (`ecs/`)

| Type | Purpose |
|------|---------|
| `World` | Entity storage and component queries |
| `Entity` | Lightweight entity handle (generation + index) |
| `Component` | Trait for ECS components |
| `System` | Trait for game systems with `update()` |
| `SystemScheduler` | Sequential system execution |

### Core (`core/`)

| Type | Purpose |
|------|---------|
| `UniversalWorld` | Extended world with archetype-based storage |
| `UniversalEntity` | Entity with archetype + chunk position |
| `ParallelScheduler` | Dependency-aware parallel system execution |
| `Changed<T>` | Change detection wrapper |

### Engine (`engine/`)

| Type | Purpose |
|------|---------|
| `GameEngine` | Top-level facade: world + scheduler + renderer + physics + input + events |
| `Renderer` / `CanvasRenderer` | Rendering abstraction |
| `SimplePhysicsWorld` | 2D physics with rigid bodies and colliders |
| `EventBus` | Event dispatch (move, collision, input, game) |
| `PetStateSystem` | 12-state pet FSM (Idle, Happy, Sad, Angry, Sleepy, etc.) |
| `HookManager` | Event hook system with permission modes |
| `ThemeManager` | Visual theme configuration and management |

### Mechanics (`mechanics/`)

| Type | Purpose |
|------|---------|
| `ConsciousnessEntity` | Domain entity (core, mind, memory, world, act, io, feel) |
| `MaslowNeeds` | Maslow hierarchy needs simulation |
| `AiComponent` | AI behavior component |
| `EconomyComponent` | In-game economy |

## Engine Adapters

| Engine | Adapter | Mapping |
|--------|---------|---------|
| Bevy | `BevyAdapter` | ECS component/system mapping |
| Unity DOTS | `UnityAdapter` | `IComponentData` / `ISystem` mapping |
| Godot 4 | `GodotAdapter` | Node/signal mapping |
| Unreal Engine 5 | `UnrealAdapter` | GAS attribute mapping |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop framework |
| `serde` / `serde_json` | Serialization |
| `serde_yaml` | YAML parsing |
| `neotrix-sim` | Simulation primitives |
| `tokio` | Async runtime |
| `rand` | Random number generation |
| `uuid` | Unique ID generation |

## License

Part of the NeoTrix project.
