# Game Engine Patterns 2024-2025 — Technical Reference

Cross-engine analysis of Unity, Unreal Engine, Godot, and Bevy.
Patterns mapped to universal abstractions for NeoTrix integration.

---

## Table of Contents

1. [Unity (2024-2025)](#unity)
2. [Unreal Engine (2024-2025)](#unreal-engine)
3. [Godot (2024-2025)](#godot)
4. [Bevy (2024-2025)](#bevy)
5. [Cross-Engine Commonalities](#cross-engine-commonalities)
6. [Universal Abstractions Map](#universal-abstractions-map)

---

## Unity

### 1. Unity 6 DOTS — ECS Architecture

**Entities 1.3 → 1.4 (March 2025 experimental)**
- API consolidation: deprecated Aspects in favor of SystemAPI.Query + Enableable Components
- Transform system unification: GameObjects and Entities converging toward shared Entity model
- Key milestone: "ECS for all" — future Unity major release will bring ECS into the engine core
- Havok or Unity Physics selectable for ECS; Built-In Physics for GameObject workflows

**Architectural decisions:**
- Entities 1.x package supports Unity 2022 LTS and Unity 6; deprecated APIs removed in future major release
- Hierarchical state machine for animation: thousands of states, blend graphs, transitions at scale
- Scene/build workflows streamlined for unified Entity/GameObject experience

**Performance patterns:**
- Burst compiler generates SIMD-optimized native code from C# HPC# subset
- C# Job System exposes internal C++ Job System for parallel execution
- Collections (NativeParallelHashMap, NativeList) provide thread-safe, cache-friendly data structures

**Ergonomics:**
- Entities Graphics package bridges ECS with rendering pipeline
- Netcode for Entities provides server-authoritative networking with client prediction
- Unity 6.1 (2025): higher frame rates, lower CPU/GPU load, improved debugging

### 2. Burst Compiler 2.0

**Current version: 1.8.30** (LLVM-based, IL→native CPU code)

**Key optimizations:**
- **Loop vectorization**: converts scalar loops to SIMD (4-wide/8-wide), processing multiple values simultaneously
- **NoAlias attribute**: tells compiler when pointers don't alias → avoids redundant reloads
- **AssumeRange**: constrains scalar-integer ranges for better codegen
- **Hint intrinsics**: provide compile-time data layout information
- **Unity.Mathematics**: SIMD-native math types (float4, int4) that Burst vectorizes aggressively
- **Memory aliasing hints**: `[NoAlias]` on method params, return values, structs, struct fields, jobs

**Developer ergonomics:**
- Burst compiles `[BurstCompile]`-attributed C# methods to native code
- Works with any Unity code, not just ECS/Jobs
- Free performance gains: recompilation with Burst often yields 2-10x speedups without code changes

### 3. Unity Collections

**Package versions: 1.2 → 2.3**

**Key data structures:**
| Type | Description | Thread Safety |
|------|-------------|---------------|
| `NativeList<T>` | Resizable unmanaged list | Thread/disposal-safe |
| `NativeParallelHashMap<TKey, TValue>` | Parallel-safe hash map | Multiple concurrent writers |
| `NativeHashMap<TKey, TValue>` | Single-thread optimized map | Single-writer, multiple readers |
| `NativeMultiHashMap<K, V>` | One-to-many mapping | Parallel-safe |
| `UnsafeList<T>` | Resizable list (no safety checks) | Manual |
| `FixedString4096Bytes` | Fixed-size string | Stack-allocated |

**Updates (2024-2025):**
- NativeHashMap added as optimized single-thread variant
- NativeParallelHashMap.ParallelWriter for scatter writes across jobs
- Collections 2.x: new API surface, deprecations for cleaner ergonomics
- All collections Burst-compatible, memory-managed via AllocatorHandle

### 4. Unity Inspector — Custom Editors

**Property Drawers (IMGUI-based, legacy):**
- `[CustomPropertyDrawer(typeof(MyClass))]` on a class inheriting `PropertyDrawer`
- Override `OnGUI(SerializedProperty, GUIContent)` for custom rendering
- Works with `[Serializable]` classes and `[SerializeField]` attributes

**Property Drawers (UI Toolkit, new):**
- Override `CreatePropertyGUI(SerializedProperty)` returning `VisualElement`
- Full CSS styling, flexbox layout, data binding
- Unity 6: extensible data binding system for Inspector↔Game synchronization

**Custom Inspectors:**
- `[CustomEditor(typeof(MyComponent))]` → override `OnInspectorGUI()`
- UI Toolkit-based inspectors use USS/UXML for layout
- `PropertyField` control auto-generates UI for any `SerializedProperty`

### 5. Addressables — Asset Management

**Package version: 3.1 → 4.0**

**Core concepts:**
- Assets organized into **Groups** with build/load/release lifecycle
- Automatic dependency resolution (replaces manual AssetBundle management)
- Remote content distribution: cloud-hosted bundles with versioning
- `AsyncOperationHandle<T>` for async load operations

**Key patterns:**
- Address = string key mapping to asset location
- Asset references via `AssetReference<T>` (deferred loading)
- Content update workflow: incremental builds for LiveOps
- `Addressables.LoadAssetAsync<T>()` → reference-counted loading
- `Addressables.Release()` → explicit memory management
- `Addressables.InstantiateAsync()` → scene/object instantiation

**Unity 6 updates:**
- Build profiles for per-platform configuration
- Cloud Diagnostics integration for performance monitoring
- Rollback capability for content updates

### 6. Unity UI Toolkit vs UGUI

| Aspect | UI Toolkit | UGUI |
|--------|-----------|------|
| **Architecture** | Retained-mode, web-inspired (CSS/UXML) | GameObject-based (Canvas) |
| **Rendering** | Single UXML → VisualElement tree | Multiple GameObjects per element |
| **Styling** | USS stylesheets (CSS-like) | Per-component `Graphic` settings |
| **Layout** | Flexbox model | RectTransform anchoring |
| **Data Binding** | Extensible binding system (Unity 6) | Manual serialization |
| **World Space** | Limited support | Full support |
| **Performance** | Better for static UI (fewer draw calls) | Better for dynamic/animated UI |
| **Editor Use** | Native (replacing IMGUI) | Not used in Editor |

**Unity 6.3 milestones:**
- Figma-to-Unity import
- Particles/3D objects in UI
- Soft masking
- Addressable Asset Bundles support
- Grid layout aligned with web standards

### 7. Netcode for GameObjects

**Package version: 2.13.0**

**Architecture:**
- High-level networking SDK for GameObject/MonoBehaviour workflows
- Client-hosted or server-authoritative models
- `NetworkBehaviour` extends `MonoBehaviour` with networking capabilities
- `NetworkObject` component for networked entities

**Key patterns:**
- **NetworkManager**: central hub for all netcode configuration
- **NetworkVariable<T>**: synchronized state with delta compression
- **RPCs (Remote Procedure Calls)**: `[ServerRpc]`, `[ClientRpc]` attributes
- **NetworkTransform**: automatic position/rotation synchronization
- **NetworkPrefab**: pre-registered prefabs for network spawning
- **Distributed Authority**: peer-to-peer alternative to client-host

**Complementary solutions:**
- **Netcode for Entities**: ECS-based, server-authoritative, for competitive action
- **Relay**: peer-to-peer NAT traversal (Unity Gaming Services)
- **Lobby**: session discovery and matchmaking

---

## Unreal Engine

### 1. UE 5.4/5.5 Features

**UE 5.4 (April 2024):**
- **Modular Control Rig**: build animation rigs from modular parts (new Modular Rig asset)
- **Automatic Retargeting**: one-click bipedal character retargeting
- **Motion Matching**: production-ready (battle-tested in Fortnite)
- **Choosers and Proxy Tables**: data-driven animation selection (Beta)
- **Nanite Tessellation**: render-time fine details (Experimental)
- **Substrate**: new material authoring framework (replaces legacy materials)
- **Chaos Destruction**: production-ready
- **Multi-Process Cook**: parallel build processing

**UE 5.5 (November 2024):**
- **MegaLights**: hundreds of dynamic shadow-casting lights (Experimental, "Nanite for lights")
- **Movie Render Graph**: graph-based rendering configuration (Beta)
- **MetaHuman Animator**: facial animation from audio (Experimental)
- **Animation Deformers**: contact deformation, squash-and-stretch (in Control Rig)
- **Skeletal Editor**: production-ready weight painting
- **Path Tracer**: production-ready, reference-quality rendering
- **Bindless Resources**: expanded platform support
- **Substrate**: Beta, all legacy material features supported
- **Mobile Renderer**: Dynamic Resolution in XR, new upscaling modes

### 2. Enhanced Input System

**Core concepts:**

| Asset | Purpose |
|-------|---------|
| **Input Action** | Logical representation of user intent ("Jump", "Fire") |
| **Input Mapping Context** | Collection of key→action mappings for a gameplay context |
| **Input Modifier** | Transforms input values (dead zone, negate, scalar, smooth) |
| **Input Trigger** | Defines when action fires (Down, Hold, Pressed, Chord) |

**Key patterns:**
- Mapping Contexts are stackable with priority (higher priority wins)
- Actions are instanced per player via `FInputActionInstance`
- `AccumulationBehavior` controls multi-key merging (override vs accumulate)
- `bConsumeInput` prevents lower-priority contexts from firing
- Runtime remapping: add/remove contexts dynamically
- Triggers: `Triggered`, `Started`, `Completed`, `Canceled` events
- Modifiers: `DeadZone`, `Negate`, `Scalar`, `Smooth`, `SwizzleAxis`, `ToWorldSpace`

**Developer ergonomics:**
- Data assets (not code-first) — designers can configure in Editor
- C++ binding via `BindAction()` on PlayerController or Character
- Blueprint-friendly: event delegates per action
- Replaces UE4 legacy Action/Axis mappings

### 3. Gameplay Ability System (GAS)

**Components:**

| Component | Role |
|-----------|------|
| **AbilitySystemComponent** | Central manager on Actor: abilities, tags, effects, attributes |
| **GameplayAbility** | Self-contained ability logic (blueprint or C++) |
| **AttributeSet** | Float values driving gameplay (health, mana, speed) |
| **GameplayEffect** | Modifies attributes over time (buffs/debuffs) |
| **GameplayTag** | Hierarchical tags for status/state representation |
| **GameplayCue** | Audio/visual feedback from effects |
| **AbilityTask** | Async building blocks within abilities |

**Key patterns:**
- Everything controlled via Gameplay Tags (hierarchical, queryable)
- Abilities handle: cooldowns, costs, activation conditions, input binding
- Effects: Instant, Duration, Infinite (with modifiers and aggregation)
- Network prediction: client-predicted abilities with server correction
- `GameplayAbilityTasks` for async flows (animation montages, waits, confirms)

**UE6 future:** GAS will be deprecated in favor of Verse Entity Framework-based replacement. Mover and Mass remain in UE6's future-facing stack.

### 4. Mass Entity (ECS Alternative)

**Overview:** Gameplay-focused framework for data-oriented calculations in UE.

**Key concepts:**
- **Fragments**: component-like data attached to entities
- **Tags**: zero-size markers for filtering
- **Archetypes**: entity configurations based on fragment combinations
- **Processors**: systems that operate on archetype queries
- **Subsystem**: manages processor execution order

**Patterns:**
- Processor/Query API simplified (experimental Query Executor)
- Mass Avoidance: force-based crowd avoidance integrated with MassEntity
- Works with World Partition for large-scale entity management
- AI integration: Mass Gameplay for pedestrian/crowd simulation

### 5. World Partition

**Core design:**
- Automatic data management and distance-based level streaming
- Single persistent level subdivided into streamable grid cells
- No manual sublevel division required (replaces Level Streaming)
- Streaming sources (player controllers, custom components) trigger load/unload

**Key features:**
- **One File Per Actor**: collaborative editing, reduced merge conflicts
- **Data Layers**: selective content loading (quest states, time-of-day)
- **HLOD (Hierarchical LOD)**: simplified distant representations
- **Runtime Streaming**: dynamic loading/unloading based on distance
- **Server Streaming**: `wp.Runtime.EnableServerStreaming` for dedicated servers

**Generation modes:**
- Non-partitioned (default)
- Partitioned (grid-based splitting)
- Hierarchical (multi-scale generation)
- Runtime (dynamic proximity-based generation)

### 6. PCG Framework

**Overview:** Visual scripting framework for procedurally populating worlds.

**Architecture:**
- **PCG Graph**: node-based graph editor (like Material Editor)
- **PCG Component**: holds graph instance, manages generation
- **PCG Volume**: spatial domain for generation
- **PCG World Actor**: singleton for world-level generation

**Node categories:**
- Blueprint: custom logic via `PCGBlueprintElement`
- Filter: data filtering by criteria
- Generic: non-spatial data operations
- Spatial: spatial relationships, queries, transforms
- Spawner: actor/instance placement at point locations

**Patterns:**
- Static Attributes (`$Position`) + Dynamic Attributes (runtime metadata)
- Graph Parameters for customizable, overridable values
- Graph Templates for reusable generation patterns
- Partitioned generation: grid cells streamable via World Partition
- Hierarchical generation: multi-scale detail (trees on large grid, grass on small)
- Runtime generation: dynamic in proximity to generation sources

**Integration:**
- PCG Biome Core/Sample plugins for ecosystem generation
- Shape Grammar for architectural generation
- GPU processing support for large-scale generation
- Quixel Megascans integration for asset population

### 7. Nanite/Lumen Rendering

**Nanite (Virtualized Micropolygon Geometry):**
- 5.4: Tessellation (Experimental) — render-time fine details
- 5.5: Continues improvements; still core to UE5 rendering pipeline
- Triangle-level LOD: automatic detail selection
- Virtual shadow maps: shadow quality scales with geometry density

**Lumen (Global Illumination):**
- Software/Hardware Ray Tracing modes
- 5.5: New reflection denoiser, translucency with refraction in reflections
- Real-time GI without baking
- Screen-space + ray-traced hybrid approach

**Substrate (Material Framework):**
- 5.4: Introduced as experimental
- 5.5: Beta — all legacy material features supported
- Physically-based material authoring with flexible layering
- Replaces fixed material models with composable parameters

---

## Godot

### 1. Godot 4.3/4.4 Features

**Godot 4.3:**
- Major optimization pass: Array.insert 10x faster, String operations 2x faster
- Rendering: 20% frame time improvement in geometry-heavy scenes
- Materials with `ambient_light_disabled` optimized
- Bicubic sampling for lightmaps
- Metal rendering backend work began

**Godot 4.4 (dev snapshot):**
- **Metal rendering backend**: native Metal API (replacing MoltenVK), faster on macOS
- **HDRI loading**: ~25x improvement, half-float conversion ~10x faster
- **Visual layers for DirectionalLight3D**
- **Fixed fog for Compatibility renderer**
- **GDScript syntax highlighter exposed to plugins**
- **Navigation triangulation partition option** for 2D
- **XR visibility mask support** (OpenXR)

### 2. GDExtension — Rust/C++ Bindings

**Official support:**
- C++ (godot-cpp) — officially supported
- Community bindings: Rust (godot-rust/gdext), D, Go, Nim, Swift, Odin

**godot-rust (gdext) patterns:**
```rust
#[derive(GodotClass)]
#[class(init, base=Sprite2D)]
struct Player {
    base: Base<Sprite2D>,
    #[init(val = 100)]
    hitpoints: i32,
    #[init(node = "Ui/HealthBar")]
    health_bar: OnReady<Gd<ProgressBar>>,
}

#[godot_api]
impl ISprite2D for Player {
    fn ready(&mut self) {
        self.health_bar.set_max(self.hitpoints as f64);
        self.health_bar.signals().value_changed().connect(|hp| {
            godot_print!("Health changed to: {hp}");
        });
    }
}

#[godot_api]
impl Player {
    #[func]
    fn take_damage(&mut self, damage: i32) {
        self.hitpoints -= damage;
        if self.hitpoints <= 0 {
            self.base_mut().queue_free();
        }
    }
}
```

**Key features:**
- `#[class(init, base=T)]`: automatic initialization + inheritance
- `OnReady<T>`: lazy node initialization on `_ready()`
- Type-safe signals: `signals().signal_name().connect(|args| {})`
- Three safeguard tiers: Strict (dev), Standard, Relaxed
- Version compatibility: GDExtension 4.2 works in 4.3, not reverse

### 3. RenderingDevice

**Architecture:** Abstraction layer over Vulkan, D3D12, Metal, WebGPU.

**Key concepts:**
- Low-level GPU API: buffers, textures, pipelines, draw/compute lists
- **Automatic render graph**: detects resource dependencies without programmer input
- **Resource tracking**: monitors buffer/texture usage to insert memory barriers
- **Local RenderingDevices**: secondary devices for multi-threaded GPU compute
- **SPIRV-reflect**: identifies read-only resources for dependency optimization

**GPU synchronization (4.3 upgrade):**
- Automatic adjacency list construction from resource usage
- Texture layout transitions handled by graph (read-to-write = write dependency)
- Shared texture slices: independent resource trackers per slice
- Significant performance improvement for particle systems, post-processing

**Developer ergonomics:**
- `RenderingServer.get_rendering_device()` for global device
- `RenderingServer.create_local_rendering_device()` for thread-local compute
- Compute shaders accessible directly from GDScript/C++
- Extension points for custom rendering features

### 4. Signal Improvements

**Typed signals (4.x):**
- GDScript: `signal health_changed(new_health: int)`
- Type-safe emission and connection
- Compile-time checking of signal arguments

**C++ (GDExtension):**
```cpp
// Type-safe signal connection in godot-rust
self.health_bar.signals().value_changed().connect(|hp: f64| {
    godot_print!("Health: {hp}");
});
```

**Patterns:**
- Signals are first-class (not stringly-typed in 4.x)
- `Signal` and `SignalName` types for type-safe references
- `connect()` with closures or method references
- Signal groups for broadcast patterns
- `SceneTreeTween` for animated signal-driven transitions

### 5. Resource System

**Core features:**
- **Resource caching**: shared resources loaded once, reference-counted
- **Local to Scene**: per-instance resource copies when needed
- **uid:// paths**: stable references across file moves
- **ResourceFormatLoader/Saver**: extensible serialization

**Patterns:**
- `@export var material: Material` — Inspector-editable resource references
- `preload()` / `load()` — synchronous resource loading
- `ResourceLoader.load()` — async with progress callbacks
- Resources are `RefCounted` — automatic memory management
- `PackedScene.instantiate()` — scene instantiation from resources

**GDExtension resource patterns:**
```rust
#[derive(GodotClass)]
#[class(base=Resource)]
struct MyResource {
    base: Base<Resource>,
    data: i32,
}
```

### 6. Input Improvements

**InputEvent rework (4.x):**
- `InputEvent` subclasses: `InputEventKey`, `InputEventMouseButton`, `InputEventJoypadMotion`
- `InputEventAction`: logical action mapping
- `InputMap`: project-wide action definitions
- `Input.is_action_pressed()`, `Input.get_axis()`, `Input.get_vector()`

**Enhanced patterns:**
- Device-aware input: per-device action mapping
- Echo events: repeated key presses
- `InputEventWithModifiers`: shift/ctrl/alt/meta state
- `InputEventScreenTouch`/`InputEventScreenDrag`: mobile input
- Parallax input: relative mouse movement for FPS cameras
- `Input.parse_input_event()` for custom event injection

---

## Bevy

### 1. Bevy 0.14/0.15 Features

**Bevy 0.14 (July 2024):**
- **Virtual Geometry (Meshlets)**: Nanite-like rendering for dense geometry
- **Sharp Screen-Space Reflections**: real-time ray-marched SSR
- **Depth of Field**: physical lens simulation
- **Per-Object Motion Blur**: relative-camera-speed blur
- **Volumetric Fog/Lighting**: god rays, atmospheric scattering
- **Filmic Color Grading**: complete tone mapping toolkit
- **PBR Anisotropy**: brushed metal, hair rendering
- **Auto Exposure**: dynamic camera exposure
- **Animation Graph**: low-level animation blending
- **ECS Observers and Hooks**: reactive component lifecycle
- **Computed States / Substates**: complex state machine modeling

**Bevy 0.15 (November 2024):**
- **Required Components**: rethink of entity spawning (deprecates all bundles)
- **Bubbling Observers**: event propagation up entity hierarchies
- **Entity Picking / Selection**: modular cross-context selection
- **Animation Masks**: per-bone animation blending
- **Visibility Bitmask Ambient Occlusion (VBAO)**: improved GTAO
- **Order Independent Transparency**: stable transparent rendering
- **Cosmic Text**: improved non-Latin text rendering
- **Gamepads as Entities**: device-as-entity pattern
- **Bevy Remote Protocol (BRP)**: external editor integration
- **Improved Text**: font smoothing control, cosmic text

### 2. Schedule v3 — New Scheduling Model

**Architecture:**
- Systems organized into **Schedules** (fixed-timestep, main, render)
- **SystemSets** for grouping and ordering
- **States** for app-level coordination
- **FixedMain** schedule runs at fixed timestep (decoupled from frame rate)

**Key patterns:**
```rust
app.add_systems(FixedUpdate, (
    physics_system,
    collision_system,
).chain())  // chaining ensures ordering
.in_set(PhysicsSet);
```

- `chain()`: explicit ordering between systems
- `in_set()`: system grouping
- `run_if()`: conditional system execution
- `ambiguous_with()`: declares systems that can safely run in parallel
- `.before()` / `.after()`: implicit ordering hints

**Schedule v3 improvements:**
- Deterministic execution order
- Parallel system scheduling with conflict detection
- FixedUpdate for physics/networking (frame-rate independent)
- Run conditions as system parameters
- Improved error handling and system failure recovery

### 3. Observers — Reactive Pattern

**Core concept:** Push-based event handling in ECS.

```rust
// Global observer: fires for ANY entity with Explode event
app.observe(|trigger: Trigger<Explode>, query: Query<&Position>| {
    let pos = query.get(trigger.target()).unwrap();
    // handle explosion at position
});

// Entity-specific observer
commands.spawn(Mine::random(&mut rng))
    .observe(|trigger: Trigger<Explode>| {
        // handle this specific mine
    });
```

**Key patterns:**
- `Trigger<T>`: typed event wrapper with target entity
- `Observer::new(fn)` → `.watch_entity(entity)` for reusable observers
- **Bubbling Observers**: events propagate up parent hierarchy
- Component lifecycle hooks: `on_add`, `on_remove`, `on_insert`
- Observers are entities with `Observer` component (can be managed, shared)

**Use cases:**
- Automatically respond to component addition/removal
- UI event handling (resize, click, hover)
- Cross-entity communication without polling
- Hierarchical event propagation (parent→child or child→parent)

### 4. Required Components

**Core innovation:** Component dependencies declared via `#[require()]`.

```rust
#[derive(Component)]
#[require(Node)]           // MyNode requires Node component
struct MyNode;

#[derive(Component)]
#[require(Transform)]      // Position requires Transform
struct Position;

#[derive(Component)]
#[require(
    Visibility,            // Visibility requires these
    InheritedVisibility,
    ViewVisibility
)]
struct VisibilityFlag;
```

**Impact:**
- Replaced ALL built-in bundles (SpriteBundle, PbrBundle, NodeBundle, etc.)
- Spawning simplified: `commands.spawn(Camera3d::default())` auto-adds required components
- Camera features declare dependencies: `MotionBlur` requires `DepthPrepass + MotionVectorPrepass`
- UI nodes: `Text::new("hello")` requires `Node`, `TextColor`, `TextFont`, `TextLayout`
- Scenes: `SceneRoot(scene)` requires `Transform + Visibility`

**Pattern:**
```rust
// Before (bundles):
commands.spawn(PbrBundle { mesh, material, transform, ..default() });

// After (required components):
commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
// Mesh3d auto-requires: Transform, Visibility, InheritedVisibility, ViewVisibility
```

### 5. Animation Graph

**Architecture:**
- Node-based animation blending graph
- Transition nodes with conditions
- Blend nodes (additive, override, mix)
- Parameter-driven transitions

**Features (0.14-0.15):**
- Animation masks: per-bone blending control
- Additive animation blending
- Animation events: callbacks at keyframes
- Generalized entity animation (not just SkinnedMesh)
- `AnimationPlayer` and `AnimationTransitions` components

**Example:**
```rust
commands.spawn((
    AnimationGraphPlayer::new(graph_handle),
    Transform::default(),
));
// Graph automatically handles blending between states
```

### 6. Scene Improvements

**SceneRoot (0.15):**
- `SceneRoot(Handle<Scene>)` replaces raw `Handle<Scene>`
- Requires `Transform + Visibility` automatically
- Dynamic scenes: `DynamicScene` for runtime scene creation

**Scene operations:**
- `Scene::write_to_world_with()` — deterministic scene instantiation
- `DynamicScene::write_to_world_with()` — runtime scene spawning
- `SceneInstanceReady` triggers observer on scene load completion
- Scene inspector: runtime scene visualization

**GPU-Driven Rendering (0.16):**
- CPU→GPU command buffer transfer for mesh rendering
- Indirect draw calls: single drawcall per object list
- 3x performance improvement on heavy scenes (Caldera Hotel test)
- Skinned mesh support added in 0.16

---

## Cross-Engine Commonalities

### 1. Entity Component System (ECS)

| Engine | ECS Implementation | Key Differences |
|--------|-------------------|-----------------|
| **Unity** | Entities package (DOTS) | Burst-compiled, Job System integration |
| **Unreal** | Mass Entity | Gameplay-focused, processor-based queries |
| **Godot** | Node tree (not true ECS) | Composition via scene tree |
| **Bevy** | Bevy ECS (Rust) | Pure ECS, Schedule v3, Observers |

**Universal pattern:** All engines moved toward data-oriented design with component-based composition. Unity and Bevy are pure ECS; Unreal uses Mass Entity as an ECS layer; Godot uses scene trees but is adding more ECS-like patterns.

### 2. Data-Oriented Design

| Engine | Approach |
|--------|----------|
| **Unity** | Burst + Jobs + Collections (native, SIMD) |
| **Unreal** | Mass Entity + Data Layers + World Partition |
| **Godot** | RenderingDevice (GPU data), GDExtension (native code) |
| **Bevy** | Rust ownership model, Schedule v3 parallelism |

### 3. Reactive/Observer Patterns

| Engine | Implementation |
|--------|---------------|
| **Unity** | C# events, UnityEvent, `ObservableProperty` |
| **Unreal** | Delegates, Gameplay Cues, Gameplay Tags |
| **Godot** | Signals (typed in 4.x), `emit_signal()` |
| **Bevy** | ECS Observers + Hooks + Bubbling |

**Common pattern:** All engines support typed event/observer systems. Bevy's Observers are the most ECS-native; Godot's signals are the most ergonomic; UE's delegates are the most flexible.

### 4. Procedural Generation

| Engine | System |
|--------|--------|
| **Unity** | Probuilder, custom scripts |
| **Unreal** | PCG Framework (graph-based, production-ready) |
| **Godot** | GDScript-based, no built-in framework |
| **Bevy** | Procedural mesh generation, noise libraries |

**UE PCG is the most mature:** Visual graph editor, partitioned generation, runtime generation, biome support. Unity and Godot lack dedicated frameworks; Bevy is community-driven.

### 5. Asset Management

| Engine | System |
|--------|--------|
| **Unity** | Addressables (async, reference-counted, cloud-capable) |
| **Unreal** | Asset Manager + Soft/Hard references + Chunk/Cluster |
| **Godot** | Resource system (cached, uid://, local-to-scene) |
| **Bevy** | Asset Server (async, Handle-based, AssetServer.load()) |

**Common pattern:** All engines use handle-based deferred loading with reference counting. Unity's Addressables is most feature-rich (cloud, versioning, incremental builds).

### 6. UI Systems

| Engine | Approach |
|--------|----------|
| **Unity** | UI Toolkit (web-inspired) + UGUI (legacy) |
| **Unreal** | UMG (Slate-based, visual designer) |
| **Godot** | Control nodes (scene-tree-based, CSS-like anchoring) |
| **Bevy** | bevy_ui (ECS-native, flexbox-inspired) |

**Trend:** All engines moving toward declarative, data-driven UI. Unity's UI Toolkit is most web-like; Godot's Control nodes are most intuitive; Bevy's UI is most ECS-integrated.

### 7. Networking

| Engine | Approach |
|--------|----------|
| **Unity** | Netcode for GameObjects/Entities + Relay/Lobby |
| **Unreal** | Built-in replication + GAS + Dedicated Server |
| **Godot** | MultiplayerAPI (high-level) + ENetMultiplayerPeer |
| **Bevy** | community crates (bevy_renet, etc.) |

### 8. Rendering Pipelines

| Engine | Pipeline |
|--------|----------|
| **Unity** | URP/HDRP + Render Graph + GPU Resident Drawer |
| **Unreal** | Deferred + Nanite + Lumen + Substrate |
| **Godot** | Forward+ / Mobile / Compatibility (Vulkan/D3D12/Metal) |
| **Bevy** | Custom PBR + Meshlets + GPU-driven rendering |

**Trend:** All engines supporting multiple rendering backends. Nanite (UE) and Meshlets (Bevy) represent the virtual geometry paradigm. Godot's RenderingDevice provides the lowest-level abstraction.

---

## Universal Abstractions Map

### Core Abstractions

| Abstraction | Unity | Unreal | Godot | Bevy |
|-------------|-------|--------|-------|------|
| **Entity** | `Entity` | `AActor` / `UMassEntity` | `Node` | `Entity` |
| **Component** | `IComponentData` | `UActorComponent` / `Fragment` | `Node property` | `Component` |
| **System** | `ISystem` / `SystemBase` | `UGameplaySystem` | `_process()` / `_physics_process()` | `System` |
| **Event** | `EventHandler` / `UnityEvent` | `FMulticastDelegate` | `Signal` | `Trigger<T>` |
| **Asset** | `Addressable` | `UAsset` | `Resource` | `Handle<T>` |
| **UI Element** | `VisualElement` (UIToolkit) | `UWidget` (UMG) | `Control` | `Node` (bevy_ui) |
| **Schedule** | `WorldSystemGroup` | `FTickFunction` | `SceneTree` | `Schedule` / `SystemSet` |
| **Query** | `EntityQuery` | `MassEntityQuery` | `get_children()` | `Query<T>` |
| **State** | ScriptableObject | `UObject` property | `Node` property | `States` trait |
| **Inspector** | PropertyDrawer | IDetailCustomization | EditorInspectorPlugin | Reflect |

### Performance Patterns

| Pattern | Unity | Unreal | Godot | Bevy |
|---------|-------|--------|-------|------|
| **SIMD** | Burst + Mathematics | SSE/NEON intrinsics | RenderingDevice | Rust auto-vectorization |
| **Threading** | Job System | FRunnable, AsyncTask | Thread class | Rayon + async tasks |
| **GPU Compute** | ComputeShader | ComputeShader | RenderingDevice compute | Compute Pipelines |
| **Memory** | NativeArray (unmanaged) | FMemory, TArray | RenderingDevice buffers | Rust ownership |
| **Batching** | BatchingRenderer | InstancedStaticMesh | MultiMeshInstance | GPU-driven rendering |
| **Culling** | FrustumCulling | Nanite culling | OcclusionCulling | VirtualGeometry culling |

### Developer Ergonomics

| Feature | Unity | Unreal | Godot | Bevy |
|---------|-------|--------|-------|------|
| **Hot Reload** | Domain Reload | Live Coding | Hot Reload (GDExtension) | Hot Reload (partial) |
| **Visual Editor** | Full (Inspector, UIToolkit) | Full (Blueprint, UMG) | Full (Scene, Inspector) | Minimal (editor WIP) |
| **Scripting** | C# | C++ / Blueprint / Verse | GDScript / C# / GDExtension | Rust |
| **Package Mgmt** | Package Manager | Plugin system | AssetLib | Cargo |
| **Version Control** | Unity Version Control | Perforce (built-in) | Git (manual) | Git (manual) |
| **Profiling** | Profiler, Frame Debugger | Unreal Insights, Stat | Visual Profiler, Monitors | Tracing, puffin |

### Architectural Decisions Summary

1. **ECS vs Object Tree**: Unity/Bevy → pure ECS; Unreal → Mass Entity layer; Godot → scene tree
2. **Rendering Abstraction**: Godot's RenderingDevice is most explicit; UE's pipeline most integrated; Unity's Render Graph most flexible; Bevy most GPU-driven
3. **Reactive Patterns**: All moving toward push-based (observers/signals/events); Bevy's Observers most ECS-native
4. **Asset Loading**: All use handle-based deferred loading; Unity's Addressables most feature-rich; Godot's Resource most lightweight
5. **Code Generation**: Unity (Burst) and Bevy (Rust) most aggressive about compile-time optimization; UE most runtime-flexible

---

## Appendix: Version Matrix

| Engine | Latest Stable | Key Version |
|--------|--------------|-------------|
| Unity | Unity 6 (6000.0) | Entities 1.3/1.4, Burst 1.8.30, Collections 2.3 |
| Unreal | UE 5.5 (Nov 2024) | Substrate Beta, MegaLights Experimental |
| Godot | Godot 4.3 stable, 4.4 dev | GDExtension 4.x, Metal backend |
| Bevy | Bevy 0.15 (Nov 2024) | 0.16 in development, GPU-driven rendering |

---

*Generated: 2026-09-12 | Sources: Official documentation, release notes, developer discussions*
