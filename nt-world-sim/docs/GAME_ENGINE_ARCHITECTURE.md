# Game Engine Architecture Patterns — Comprehensive Report

> Research Date: 2026-09-13
> Sources: Godot Engine, Redot Engine, Bevy, Bullet3, ImGui, and game-engine/game-development GitHub topics
> Purpose: Extract architecture patterns applicable to NeoTrix NT-WORLD simulation capabilities

---

## Table of Contents

1. [Godot Engine Architecture](#1-godot-engine-architecture)
2. [Redot Engine](#2-redot-engine)
3. [Game Engine Patterns](#3-game-engine-patterns)
4. [Rendering Pipeline](#4-rendering-pipeline)
5. [Scene System](#5-scene-system)
6. [Physics System](#6-physics-system)
7. [Scripting System](#7-scripting-system)
8. [Resource Management](#8-resource-management)
9. [UI Framework](#9-ui-framework)
10. [Key Takeaways for NeoTrix](#10-key-takeaways-for-neotrix)

---

## 1. Godot Engine Architecture

**Source**: godotengine/godot (117k★), DeepWiki architecture docs

### 1.1 Repository Structure

```
godot/
├── core/          # Fundamental types, Object model, OS abstraction, I/O, config
├── scene/         # Scene graph, nodes, GUI, animation, resources
├── servers/       # Rendering, physics, audio, navigation, display servers
├── modules/       # Optional modules (GDScript, C#/Mono, etc.)
├── editor/        # Full editor UI and tools (TOOLS_ENABLED)
├── platform/      # Platform-specific OS and display implementations
├── drivers/       # Low-level hardware drivers (Vulkan, D3D12, GLES3)
├── main/          # Engine entry point and initialization
├── thirdparty/    # Bundled third-party libraries
├── doc/           # XML class reference documentation
└── tests/         # Unit and integration tests
```

### 1.2 Build Configurations

Three distinct binary types from the same source tree:

| Build Type | Flag | Contents |
|-----------|------|----------|
| Editor | `target=editor` | Runtime + full editor + import pipeline |
| Debug export | `target=template_debug` | Runtime + debug helpers, no editor |
| Release export | `target=template_release` | Minimal runtime only |

### 1.3 Engine Initialization Sequence

The `Main` class in `main/main.cpp` bootstraps the engine in phases:

1. **Core Setup** — Initializes `Engine`, `ProjectSettings`, `Input`, `TranslationServer`
2. **Platform Detection** — Configures display/rendering/audio drivers per platform
3. **Server Initialization** — Creates `DisplayServer`, `RenderingServer`, `PhysicsServer2D/3D`, `AudioServer`
4. **Scene System** — Registers node types, resources, theme system
5. **MainLoop Creation** — Either `SceneTree` (games) or `EditorNode` (editor)

### 1.4 Object and Type System

```
Object (base)
├── Node (scene objects)
│   ├── CanvasItem (2D)
│   │   ├── Node2D
│   │   └── Control (UI)
│   ├── Node3D (3D)
│   └── ...
├── Resource (loadable/saveable data)
└── ...
```

- All objects inherit from `Object`
- `ClassDB` singleton maintains class registry (methods, properties, signals)
- Enables GDScript introspection, editor Inspector, and GDExtension API

### 1.5 Layered Architecture

| Layer | Responsibility |
|-------|---------------|
| Platform | OS abstraction (OS_Windows, OS_Unix, OS_Web) |
| Core | Object model, math, I/O, config, resource management |
| Servers | Thread-safe APIs: RenderingServer, PhysicsServer, AudioServer |
| Scene | Node hierarchy, viewport, GUI controls, animation |
| Editor | Full IDE built on top of the engine's own UI system |

---

## 2. Redot Engine

**Source**: Redot-Engine/redot-engine (6k★), forked from Godot September 2024

### 2.1 What Redot Adds

Redot is a **community-driven fork** of Godot with the same architecture but:

- **MCP Integration** — Built-in Model Context Protocol support for AI integration (`redot-mcp.sh`)
- **Nix Build Support** — `nix run .` one-command build environment
- **AI Policy** — `AI_POLICY.md` defining AI contribution guidelines
- **Community Governance** — More open community-driven development model

### 2.2 Same Core Architecture

Redot preserves Godot's architecture exactly:

```
redot-engine/
├── core/          # Same as Godot
├── scene/         # Same as Godot
├── servers/       # Same as Godot
├── editor/        # Same as Godot
├── modules/       # Same as Godot
├── platform/      # Same as Godot
├── drivers/       # Same as Godot
└── ...
```

### 2.3 Redot-Specific Files

| File | Purpose |
|------|---------|
| `AI_POLICY.md` | Guidelines for AI-generated contributions |
| `REDOT_AUTHORS.md` | Redot-specific contributor credits |
| `redot-mcp.sh` | MCP server integration script |
| `flake.nix` | Nix build environment definition |

### 2.4 Key Insight

Redot demonstrates that well-architected engines can fork cleanly. The Server pattern (decoupled simulation from scene) makes it possible to swap entire subsystems without breaking the node API.

---

## 3. Game Engine Patterns

Extracted from 9,482+ game engine repos and 38,152+ game development repos.

### 3.1 Server Pattern (Godot/Redot)

**Most critical pattern**: Simulation logic runs in thread-safe "Server" singletons, decoupled from the scene tree.

```
Scene Layer          Server Layer
─────────────        ─────────────
Node3D ──RID──→ RenderingServer
RigidBody3D ──RID──→ PhysicsServer3D
AudioStreamPlayer ──RID──→ AudioServer
```

**Why it matters**:
- Physics can run on a separate thread
- Rendering can be double-buffered
- Scene tree stays responsive during heavy computation
- RIDs (Resource IDs) enable zero-copy references

### 3.2 ECS (Entity-Component-System)

**Source**: Bevy (48.2k★), EnTT (13.1k★)

```
Entity = unique ID
Component = data-only struct (Position { x, y })
System = function that iterates over components
```

**Godot's approach**: Hybrid — Node tree + component-like properties (not pure ECS)
**Bevy's approach**: Pure ECS with Rust's type system enforcing data-oriented design

### 3.3 Scene Tree Pattern

**Source**: Godot/Redot

```
SceneTree
└── Root (Window)
    ├── GameWorld
    │   ├── Player (CharacterBody3D)
    │   ├── Enemies (Node3D)
    │   │   ├── Enemy1 (RigidBody3D)
    │   │   └── Enemy2 (RigidBody3D)
    │   └── Environment
    └── UI (Control)
        ├── HUD
        └── PauseMenu
```

Key properties:
- **Hierarchical** — parent-child transforms cascade
- **Instanced** — scenes within scenes (composition)
- **Groups** — cross-tree tagging for queries
- **Owner** — tracks which scene "owns" a node

### 3.4 Signal/Observer Pattern

```cpp
// Godot signal system
ADD_SIGNAL(MethodInfo("health_changed", PropertyInfo(Variant::INT, "new_health")));
connect("health_changed", Callable(this, "_on_health_changed"));
emit_signal("health_changed", current_health);
```

### 3.5 Resource-Reference Pattern

- `Resource` — base class for reusable data
- `Ref<T>` — reference-counted smart pointer
- `ResourceLoader` — async/threaded loading pipeline
- `ResourceCache` — singleton cache prevents duplicate loads

### 3.6 Platform Abstraction Pattern

```
OS (abstract base)
├── OS_Windows
├── OS_Unix (Linux/macOS base)
├── OS_Web (Emscripten)
└── ...

DisplayServer (abstract base)
├── DisplayServerWindows
├── DisplayServerX11
├── DisplayServerWeb
└── ...
```

---

## 4. Rendering Pipeline

### 4.1 Godot's Rendering Architecture

```
Scene Nodes → RenderingServer → RenderingDevice → GPU Drivers
                                      │
                              ┌───────┴───────┐
                              │   Vulkan      │
                              │   D3D12       │
                              │   Metal       │
                              │   OpenGL 3.3  │
                              └───────────────┘
```

**Key design decisions**:
1. **Server Decoupling** — Scene nodes never touch GPU directly; they submit commands to `RenderingServer`
2. **Command Buffer** — RenderingServer queues commands, executes on render thread
3. **Multi-backend** — Vulkan (primary), D3D12, Metal, GLES3 (compatibility)
4. **RD Abstraction** — `RenderingDevice` provides unified API across backends

### 4.2 Rendering Pipeline Stages

| Stage | Function |
|-------|----------|
| **Culling** | Frustum/occlusion culling on scene tree |
| **Sorting** | Depth sorting for transparency, Z-prepass for opaque |
| **Shadow Maps** | Cascaded shadow maps (CSM) for directional lights |
| **Geometry** | Forward+/Forward rendering path |
| **Lighting** | PBR with clustered/forward lighting |
| **Post-Processing** | SSAO, SSR, DOF, bloom, tonemapping |
| **Output** | Swap chain presentation |

### 4.3 Two Rendering Paths

| Path | Target |
|------|--------|
| **Forward+** | Desktop, Vulkan/D3D12 (clustered lighting, SDFGI) |
| **Mobile** | Mobile GPUs, optimized bandwidth |
| **Compatibility** | OpenGL 3.3 / WebGL 2 (GLES3 backend) |

### 4.4 Shader System

GDScript shader language compiles to:
- GLSL (Vulkan/GLES3)
- HLSL (D3D12)
- MSL (Metal)

**Visual Shader** system provides node-graph shader editing that generates the same code.

---

## 5. Scene System

### 5.1 Node Lifecycle

```
Node lifecycle:
  _enter_tree()    → Node added to SceneTree
  _ready()         → All children ready (called bottom-up)
  _process(delta)  → Every frame (game logic)
  _physics_process(delta) → Fixed timestep (physics)
  _exit_tree()     → Node removed from SceneTree
```

### 5.2 Node Hierarchy

```
Node (base)
├── Node2D (2D transform)
│   ├── Sprite2D
│   ├── TileMapLayer
│   └── ...
├── Node3D (3D transform)
│   ├── Camera3D
│   ├── MeshInstance3D
│   ├── Light3D
│   └── ...
├── Control (UI base)
│   ├── Button
│   ├── Label
│   ├── VBoxContainer
│   └── ...
├── PhysicsBody2D/3D
│   ├── CharacterBody2D/3D
│   ├── RigidBody2D/3D
│   └── StaticBody2D/3D
├── AudioStreamPlayer
└── ...
```

### 5.3 Scene Instancing

Scenes are composable units:
```
Main Scene
├── World.tscn (instanced)
│   ├── Player.tscn (instanced)
│   └── Enemy.tscn (instanced × 10)
└── UI.tscn (instanced)
```

Format: `.tscn` (text), `.scn` (binary), `.tres`/`.res` (resources)

### 5.4 Viewport System

- `Viewport` — rendering context that displays scene content
- `SubViewport` — renders to texture (picture-in-picture, portals)
- `Window` — creates OS windows with Viewport

---

## 6. Physics System

### 6.1 Server-Client Architecture

```
Scene Nodes ←→ PhysicsServer ←→ Physics Engine Backend
                  │
         ┌────────┴────────┐
         │  Godot Physics  │  (built-in)
         │  Jolt Physics   │  (high-performance 3D)
         └─────────────────┘
```

### 6.2 Body Types

| Type | Physics | Control | Use Case |
|------|---------|---------|----------|
| **RigidBody** | Full Newtonian | Forces/implosions | Projectiles, debris, vehicles |
| **CharacterBody** | Kinematic | `move_and_slide()` | Player characters, NPCs |
| **StaticBody** | None | None | Walls, floors, environment |
| **AnimatableBody** | Sync | Scripted movement | Platforms, doors, elevators |

### 6.3 Physics Bodies Inheritance

```
CollisionObject2D/3D
├── PhysicsBody2D/3D
│   ├── RigidBody2D/3D (full simulation)
│   ├── CharacterBody2D/3D (kinematic)
│   ├── StaticBody2D/3D (immobile)
│   ├── AnimatableBody2D/3D (scripted)
│   └── VehicleBody3D (car physics)
└── Area2D/3D (detection zones)
```

### 6.4 Collision Shapes

- **Primitives**: Box, Sphere, Capsule, Cylinder
- **Complex**: Convex polygons, Concave (trimesh) meshes
- **Shape Owners**: Multiple shapes per body without extra nodes

### 6.5 Fixed Timestep

Physics runs at fixed interval (default 60Hz), decoupled from rendering:
```
process_delta = variable (vsync rate)
physics_delta = fixed (1/60s)
interpolation = blends physics states for smooth rendering
```

---

## 7. Scripting System

### 7.1 GDScript Pipeline

```
Source Code (.gd)
    ↓ Tokenizer (lexical analysis)
    ↓ Parser (AST construction)
    ↓ Analyzer (semantic analysis, type resolution)
    ↓ Compiler (bytecode generation)
    ↓ VM (bytecode execution)
```

### 7.2 Language Integration Architecture

```cpp
// ScriptLanguage abstraction
class ScriptLanguage {
    virtual Script *create_script() = 0;
    virtual void finish() = 0;
    virtual bool validate(const String &p_code) = 0;
    virtual void reload_all_scripts() = 0;
};

// Registration
ScriptServer::register_language(&gdscript_language);
ScriptServer::register_language(&csharp_language);
```

### 7.3 Supported Languages

| Language | Module | Backend |
|----------|--------|---------|
| GDScript | `modules/gdscript/` | Custom bytecode VM |
| C# | `modules/mono/` | .NET runtime (GodotSharp) |
| GDExtension | `modules/gdextension/` | C/C++/Rust native plugins |
| VisualScript | Deprecated | (removed in Godot 4) |

### 7.4 GDScript Language Server

LSP (Language Server Protocol) support for:
- Auto-completion
- Go-to-definition
- Hover documentation
- Refactoring

### 7.5 Key Pattern: Script Extends Node

```gdscript
extends CharacterBody3D  # Script inherits Node's class

@export var speed: float = 5.0

func _physics_process(delta):
    velocity = direction * speed
    move_and_slide()
```

---

## 8. Resource Management

### 8.1 Resource Loading Pipeline

```
User Code: load("res://textures/player.png")
    ↓
ResourceLoader::load()
    ↓
ResourceFormatLoader (per format)
    ├── ResourceFormatLoaderText (.tres, .tscn)
    ├── ResourceFormatLoaderBinary (.res, .scn)
    ├── ResourceFormatLoaderImage (textures)
    ├── ResourceFormatLoaderAudio (audio)
    └── ...
    ↓
ResourceCache::add() (singleton cache)
    ↓
Ref<Resource> returned
```

### 8.2 Resource Types

| Type | Extension | Description |
|------|-----------|-------------|
| Scene | `.tscn`/`.scn` | Node tree + properties |
| Texture | `.png`/`.jpg`/`.exr` | Image data |
| Mesh | `.obj`/`.glb`/`.gltf` | 3D geometry |
| Audio | `.wav`/`.ogg`/`.mp3` | Sound data |
| Script | `.gd`/`.cs` | Code |
| Material | `.tres` | Shader + parameters |
| Theme | `.tres` | UI styling |

### 8.3 Import Pipeline

Assets go through an import system on first load:
```
raw_asset.png → import("res://.import/player.png.import")
    ↓
    ├── Resize (if needed)
    ├── Compress (VRAM, lossless, etc.)
    ├── Generate mipmaps
    └── Save as .ctex (compressed texture)
```

### 8.4 Threaded Loading

```gdscript
ResourceLoader.load_threaded_request("res://large_scene.tscn")
# Later...
var scene = ResourceLoader.load_threaded_get("res://large_scene.tscn")
```

### 8.5 Resource Cache Behavior

- Resources with same path share the same instance
- `ResourceCache` is a global singleton
- Prevents duplicate loads of expensive assets
- `Resource.duplicate()` creates independent copies when needed

---

## 9. UI Framework

### 9.1 Control Hierarchy

```
Control (base)
├── Container
│   ├── VBoxContainer
│   ├── HBoxContainer
│   ├── GridContainer
│   ├── MarginContainer
│   └── ...
├── Button
│   ├── CheckBox
│   ├── CheckBox
│   └── OptionButton
├── Label
├── LineEdit
├── TextEdit
│   └── CodeEdit
├── RichTextLabel
├── Tree
├── ItemList
├── Panel
├── ProgressBar
├── ScrollContainer
└── ...
```

### 9.2 Layout System

**Anchors**: Relative positioning to parent edges
```
anchor_left = 0.0   (left edge)
anchor_right = 1.0  (right edge)
anchor_top = 0.0
anchor_bottom = 0.5 (half height)
```

**Containers**: Automatic child layout
- `VBoxContainer` — vertical stacking
- `HBoxContainer` — horizontal stacking
- `GridContainer` — grid with columns
- `MarginContainer` — adds padding

**Offsets**: Pixel-based fine-tuning within anchor constraints

### 9.3 Theme System

```gdscript
# Theme resource defines styling for all Control types
theme = Theme.new()
theme.set_stylebox("normal", "Button", stylebox)
theme.set_font("font", "Button", font)
theme.set_color("font_color", "Button", Color.WHITE)
```

- Themes cascade down the tree
- Per-control overrides
- Support for dark/light modes

### 9.4 Focus and Input

- Keyboard navigation via focus neighbors
- Mouse event routing through the tree
- `gui_input` signal for custom handling

---

## 10. Key Takeaways for NeoTrix

### 10.1 Patterns to Adopt

| Pattern | Godot Implementation | NeoTrix Application |
|---------|---------------------|---------------------|
| **Server Pattern** | RenderingServer, PhysicsServer, AudioServer decoupled from nodes | NT-ACT tools, NT-WORLD sensors as Server singletons; scene nodes send RIDs |
| **RID Decoupling** | Scene objects hold lightweight RIDs to Server objects | SensoryIntegrationHub could use RIDs for observation streams |
| **Resource Cache** | Global ResourceCache prevents duplicate loads | KB embedding cache with same-path deduplication |
| **Fixed Timestep** | Physics at fixed Hz, rendering interpolates | ConsciousnessTree tick could use fixed-rate reasoning with async observation |
| **Node Lifecycle** | _enter_tree → _ready → _process → _exit_tree | Module activation/deactivation hooks in NT-WORLD |
| **Signal/Observer** | Type-safe signals with ADD_SIGNAL macro | EventBus already in place; formalize signal type registry |
| **Scene Instancing** | Scenes compose via instancing | Skill trees compose via domain nodes |
| **Threading Model** | Main thread + render thread + worker pool | NT-MIND background loop + worker thread pool |
| **Module System** | Optional modules via SCons flags | Domain modules with compile-time feature flags |
| **Platform Abstraction** | OS subclass per platform | NT-PHYSICAL sensors/motors per platform |

### 10.2 Architecture Principles from Game Engines

1. **Server Decoupling** — Never let high-level code talk to hardware directly. Always go through a thread-safe server. This is the single most important pattern.

2. **RID + Server** — Scene objects are lightweight handles. All heavy state lives in servers. Enables:
   - Thread-safe simulation
   - Hot-swapping backends
   - Zero-copy references

3. **Resource Immutability by Default** — Resources are shared, not copied. Use `duplicate()` only when mutation is needed.

4. **Lifecycle Hooks** — Every object has well-defined creation/destruction points. No orphaned state.

5. **Fixed vs Variable Timestep** — Simulation (physics, AI reasoning) at fixed rate. Rendering/observation at variable rate. Interpolate between.

6. **Composition Over Inheritance** — Scenes compose via instancing. Nodes compose via the tree. Avoid deep inheritance chains.

7. **Signal-Based Communication** — Decouple modules via signals. Never direct function calls between distant modules.

8. **Lazy Loading** — Resources loaded on demand. Threaded loading for large assets. Progress tracking.

9. **Platform Abstraction Layer** — Never platform-specific code in core. Always through an abstract interface.

10. **Build System as Configuration** — Feature flags, optional modules, platform detection — all via build config, not runtime checks.

### 10.3 Code Examples

#### Server Pattern (Godot-style)

```cpp
// Scene side: lightweight
class RigidBody3D : public Node3D {
    RID body;  // Handle to server object
    
    void _ready() {
        body = PhysicsServer3D::get_singleton()->body_create();
        PhysicsServer3D::get_singleton()->body_set_space(body, get_world_3d()->get_space());
    }
    
    void _physics_process(float delta) {
        // Read state from server
        Transform3D transform = PhysicsServer3D::get_singleton()->body_get_state(
            body, PhysicsServer3D::BODY_STATE_TRANSFORM
        );
        set_global_transform(transform);
    }
};

// Server side: heavy, thread-safe
class PhysicsServer3D {
    virtual RID body_create() = 0;
    virtual void body_set_state(RID p_body, BodyState p_state, const Variant &p_variant) = 0;
    virtual Variant body_get_state(RID p_body, BodyState p_state) const = 0;
};
```

#### Resource Cache (Godot-style)

```cpp
// ResourceLoader with cache
Ref<Resource> ResourceLoader::load(const String &p_path) {
    // Check cache first
    if (ResourceCache::has(p_path)) {
        return ResourceCache::get(p_path);
    }
    
    // Determine format
    Ref<ResourceFormatLoader> loader = get_format_loader_for_path(p_path);
    Ref<Resource> resource = loader->load(p_path);
    
    // Cache for future loads
    ResourceCache::add(p_path, resource);
    return resource;
}
```

#### Signal System (Godot-style)

```cpp
// Define signals
ADD_SIGNAL(MethodInfo("damage_taken",
    PropertyInfo(Variant::FLOAT, "amount"),
    PropertyInfo(Variant::OBJECT, "source")
));

// Connect
player->connect("damage_taken", Callable(this, "_on_player_damaged"));

// Emit
emit_signal("damage_taken", 25.0, enemy);
```

#### Fixed Timestep Loop (Godot-style)

```cpp
// SceneTree main loop
void SceneTree::process(float p_delta) {
    // Fixed physics step
    physics_process_accumulator += p_delta;
    while (physics_process_accumulator >= physics_step) {
        _physics_process(physics_step);
        physics_process_accumulator -= physics_step;
    }
    
    // Variable render step
    _process(p_delta);
    
    // Render
    RenderingServer::get_singleton()->draw();
}
```

### 10.4 Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | Better Approach |
|-------------|-------------|-----------------|
| Direct GPU calls from game code | Not thread-safe, breaks rendering pipeline | Always go through RenderingServer |
| Deep inheritance chains | Fragile, hard to maintain | Composition via node tree |
| Mutable shared resources | Race conditions, data corruption | Immutable resources + duplicate() |
| No lifecycle management | Memory leaks, orphaned objects | Always implement _enter_tree/_exit_tree |
| Tight coupling between systems | Can't swap backends, hard to test | Server pattern + RID decoupling |
| Fixed timestep for rendering | Stuttering, wasted frames | Variable timestep with interpolation |

---

## Appendix A: Top Game Engine Repositories (2026)

| Repository | Stars | Language | Key Pattern |
|-----------|-------|----------|-------------|
| godotengine/godot | 117k | C++ | Scene tree + Server pattern |
| ocornut/imgui | 76.2k | C++ | Immediate mode GUI |
| bevyengine/bevy | 48.2k | Rust | Pure ECS + data-oriented |
| raysan5/raylib | 34.7k | C | Minimal C library |
| 4ian/GDevelop | 26.5k | JS | No-code event system |
| BabylonJS/Babylon.js | 26.1k | TS | WebGPU/WebGL engine |
| libgdx/libgdx | 25.4k | Java | Cross-platform framework |
| kitao/pyxel | 17.9k | Rust | Retro Python engine |
| aframevr/aframe | 17.6k | JS | WebXR + ECS |
| playcanvas/engine | 16.7k | JS | WebGL/WebGPU runtime |
| skypjack/entt | 13.1k | C++ | Header-only ECS |
| ebiten/ebiten | 13.5k | Go | Simple 2D engine |

## Appendix B: Key Libraries Used Across Engines

| Library | Purpose | Used By |
|---------|---------|---------|
| Bullet3 (14.7k★) | Physics simulation | Many engines |
| Dear ImGui (76.2k★) | Immediate mode GUI | Editors, tools |
| EnTT (13.1k★) | ECS framework | C++ engines |
| SDL2 | Window/input/audio | Many engines |
| Vulkan | GPU API | Godot, Bevy |
| Spine | 2D skeletal animation | Multiple engines |

---

*Report generated by NeoTrix NT-WORLD research agent. Last updated: 2026-09-13.*
