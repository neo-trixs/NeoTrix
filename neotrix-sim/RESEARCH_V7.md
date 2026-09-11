# RESEARCH_V7.md — NT-WORLD-SIM Technology Landscape

> Generated: 2026-09-11 | Sources: Web search + GitHub topics + docs.rs + direct fetches
> 30 topics covering: ECS engines, AI agent systems, simulation visualization, game infrastructure

---

## 1. Bevy ECS Game Tutorial 2025/2026

| Field | Value |
|-------|-------|
| **URL** | https://bevyengine.org/learn/ |
| **Key Insight** | Bevy 0.19.1 (Aug 2026) — data-driven ECS engine, 48.1k★. Quick Start Guide + official examples. Breaking API changes ~every 3 months. MSRV tracks latest stable Rust. Modular: use only what you need via cargo features (`2d`, `3d`, `ui`, `audio` profiles). |
| **NeoTrix Mapping** | NT-WORLD-SIM rendering backend. Bevy's ECS maps 1:1 to NeoTrix's entity-component architecture. The `App::new().add_plugins(DefaultPlugins)` pattern is the integration point. Use `bevy_state` for game state management matching NT-CORE's state machines. |

## 2. Bevy 2D Game Rendering

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy/latest/bevy/ |
| **Key Insight** | Bevy 2D pipeline: `bevy_sprite` + `bevy_sprite_render` + `bevy_core_pipeline`. Features: Sprite sheets, tilemap chunks, 9-patch slicing, texture atlases, sprite flipping, transparency, pixel grid snapping, CPU drawing. `Camera2d` with orthographic projection. Profile `2d` excludes all 3D code. |
| **NeoTrix Mapping** | Core rendering layer for NT-WORLD-SIM. Sprite-based agents + tilemap world. Use `bevy_sprite_render` for agent visualization. `Camera2d` orthographic projection for top-down/isometric view of agent world. `bevy_gizmos` for debug visualization of agent paths/relationships. |

## 3. Tauri 2.0 Game Development

| Field | Value |
|-------|-------|
| **URL** | https://github.com/tauri-apps/tauri |
| **Key Insight** | Tauri 2.0 provides desktop app shell with web frontend. Rust backend + webview frontend. IPC bridge for game state. Suitable for game editor/launcher but NOT for real-time rendering (webview compositing overhead). Best for: settings UI, save management, mod manager, analytics dashboard. |
| **NeoTrix Mapping** | NT-IO integration layer. Use Tauri for: game settings window, mod manager UI, analytics dashboard, save file browser. NOT for the game viewport itself (use Bevy). Tauri's `invoke()` bridge connects Bevy game state to web-based UI panels. |

## 4. Open Source Simulation Game Rust

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/game-engine?l=rust |
| **Key Insight** | Rust game engine ecosystem: Bevy (48.1k★), Fyrox (9.5k★), ggez (4.7k★), macroquad (4.6k★), Ambient (3.9k★), Ferrumc (2.4k★). Fyrox has built-in scene editor. Eldiron (813★) is a classic RPG creator with world editing tools. |
| **NeoTrix Mapping** | Bevy is the primary choice for NT-WORLD-SIM (largest ecosystem, ECS-native). Fyrox considered for editor tooling reference. Eldiron's world-editing patterns inform NT-WORLD-SIM's agent world editor design. |

## 5. Agent-Based Model Visualization

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/agent-based-model |
| **Key Insight** | 214 repos. Top: SWE-agent (20.3k★), Devon (3.5k★), MATSim (643★). Rust ABM: bourse (23★) — market simulation with Python API. Evoplex (146★) — cross-platform ABM on graphs. GAMA (114★) — agent-based modeling platform. Most ABMs use Python; Rust ABMs are rare but growing. |
| **NeoTrix Mapping** | MATSim's multi-agent transport simulation pattern informs NT-WORLD-SIM's agent mobility modeling. Evoplex's graph-based ABM maps to NeoTrix's KB graph structure. bourse's Rust+Python bridge pattern useful for NT-MIND's Python interop. |

## 6. Real-Time Simulation Visualization

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/rust-game-engine |
| **Key Insight** | frame-engine (1★) — "headless, deterministic, fixed-timestep core, paired with a companion 3D editor for inspecting and editing live world state." aeon-engine (12★) — "3D Safe Modular game engine" with ECS + scene graph. Key pattern: headless simulation core + separate visualization renderer. |
| **NeoTrix Mapping** | frame-engine's architecture (headless core + editor) matches NT-WORLD-SIM's planned split: `nt_world_sim` (pure ECS simulation) + `nt_world_sim_bevy` (Bevy rendering adapter). Fixed-timestep simulation with variable-rate rendering. |

## 7. Game Camera System 2D

| Field | Value |
|-------|-------|
| **URL** | https://github.com/bevyengine/bevy/tree/latest/examples/camera |
| **Key Insight** | Bevy camera examples: 2D top-down camera (smooth follow), Pan Camera (2D styled), Screen Shake, Camera Orbit, Projection Zoom. `bevy_camera_controller` provides FreeCamera, PanCamera. `Camera::viewport_to_world_2d` for coordinate conversion. |
| **NeoTrix Mapping** | 2D top-down camera with smooth follow = NT-WORLD-SIM agent view. Pan Camera for world exploration. Screen Shake for event feedback. `viewport_to_world_2d` critical for click-to-select agents. Implement camera states: follow agent, free roam, overview. |

## 8. Game Entity Component System

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_ecs/latest/bevy_ecs/ |
| **Key Insight** | Bevy ECS: Queries, Resources, Events, Observers, Relationships, Component Hooks, Change Detection, Fixed Timestep, Custom Schedules, System Piping. Key features: `Dynamic ECS` (runtime component creation), `Entity disabling` (hide without delete), `One Shot Systems` (flexible execution). |
| **NeoTrix Mapping** | Bevy ECS is NT-WORLD-SIM's runtime foundation. Dynamic ECS for runtime agent trait injection. Entity disabling for agent hibernation. Observers for agent event handling. Fixed timestep for deterministic simulation. System piping for agent behavior chains. |

## 9. Consciousness Simulation Game

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/agent-based-model (search: consciousness) |
| **Key Insight** | No direct "consciousness simulation game" found. Closest: SpikeAgent (62★) — LLM-based AI agent for neuroscience. Consciousness modeling in games is nascent. IIT (Integrated Information Theory) phi computation is computationally expensive. Most games use simplified models (attention, emotion, memory). |
| **NeoTrix Mapping** | NeoTrix IS the consciousness simulation. NT-WORLD-SIM visualizes ConsciousnessTree's internal state. Map: phi score → visual intensity, coherence → particle density, GWT attention → highlighted regions, emotion → color palette. The game IS the consciousness dashboard. |

## 10. AI Agent Behavior Tree

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/behavior-tree |
| **Key Insight** | 323 repos. Top: beehave (3.3k★, Godot), behaviac (3k★, Unity/FSM+HTN+BT), LimboAI (3k★, Godot 4). Rust: bonsai (1.1k★) — "Rust implementation of behavior trees for deterministic AI" with Bevy integration + Python bindings. Key: BT + FSM + HTN hybrid approaches dominate. |
| **NeoTrix Mapping** | bonsai (Rust, Bevy-native) is the primary candidate for NT-WORLD-SIM agent behavior. Its deterministic execution matches NT-CORE's E8 reasoning. `bonsai::bt::BehaviorTree` can wrap NT-MIND's skill nodes as BT actions. Python bindings enable NT-MIND's Python skill pipeline. |

## 11. AI Agent Goal Planning (GOAP)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/goap |
| **Key Insight** | 99 repos. Top: ReGoap (1.1k★, C#/Unity). Rust: dogoap (198★) — "GOAP with Bevy integration". rgoap (32★) — simple Rust GOAP. Key insight: GOAP = forward-chaining planner with preconditions/effects/costs. A* search through action space. |
| **NeoTrix Mapping** | dogoap's Bevy integration is directly usable. Map NT-MIND's skill preconditions → GOAP preconditions, skill effects → GOAP effects, skill cost → GOAP action cost. GOAP planner can select which NT-* skill to execute based on agent goals. Combine with bonsai BT for action execution. |

## 12. AI Agent Memory System

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/behavior-tree (search: memory) |
| **Key Insight** | Agent memory patterns: Working Memory (current state), Episodic Memory (past events), Semantic Memory (knowledge), Procedural Memory (skills). Most game AI uses simple blackboard pattern. Sophisticated: MATSim's agent history, GAMA's memory builtins. No dominant Rust library. |
| **NeoTrix Mapping** | NT-MEMORY IS the memory system. NT-WORLD-SIM agents query KB for: episodic (experience-tree), semantic (domain nodes), procedural (skill crystallization). Blackboard pattern for working memory (Bevy Resource). Map: agent memory = KB namespace per agent + local blackboard component. |

## 13. AI Agent Social Dynamics

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/agent-based-model (search: social) |
| **Key Insight** | Social simulation: MATSim (643★) models agent interactions in transport. COVID-ABS (93★) models social distancing. Acclimate (39★) models economic loss propagation through supply chains. Key pattern: agent relationships as graph edges with weight/type. |
| **NeoTrix Mapping** | Agent social dynamics = KB graph edges between agent entities. Edge types: trade, ally, rival, mentor, trade. Edge weights decay over time (memory forgetting). NT-WORLD-SIM visualizes social graph as minimap overlay. CharacterInteractionGraph from NT-CORE provides the data model. |

## 14. AI Agent Evolution (Genetic)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/evolutionary-algorithms (search: genetic) |
| **Key Insight** | Evolutionary algorithms for agents: genetic programming for strategy evolution, NEAT (neuroevolution), novelty search. Evoplex (146★) supports evolutionary game theory. snake_01 (17★, Rust) — "Snake AI, Genetic Algorithm". Key: fitness function defines survival, mutation introduces variation. |
| **NeoTrix Mapping** | NT-MIND's SEAL pipeline IS the evolution loop. NT-WORLD-SIM adds visual evolution: agents with higher fitness (phi score) reproduce, low-fitness agents degrade. Map: fitness = NT-CORE's phi × coherence. Mutation = NT-MIND's skill mutation. Selection = GWT attention routing (salient agents survive). |

## 15. Game HUD Design Patterns

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_ui/latest/bevy_ui/ |
| **Key Insight** | Bevy UI: ECS-driven, flexbox layout, text/sprite/UI nodes, picking system, focus management. Features: `bevy_ui_widgets` (buttons, checkboxes, sliders), `bevy_feathers` (styled widget collection). HUD pattern: camera-independent UI node tree with `TargetCamera` component. |
| **NeoTrix Mapping** | NT-WORLD-SIM HUD: agent status panel (top-left), mini world map (top-right), event log (bottom), emotion gauge (side). Use Bevy UI nodes with `TargetCamera` for screen-space overlay. `bevy_feathers` for styled panels. State-driven visibility (show/hide based on game state). |

## 16. Game Minimap Implementation

| Field | Value |
|-------|-------|
| **URL** | https://github.com/bevyengine/bevy/tree/latest/examples/2d (search: render-to-texture) |
| **Key Insight** | Bevy minimap pattern: secondary `Camera2d` with orthographic projection, rendering to texture (RTT), displayed as UI sprite. `RenderToTexture` component + separate camera hierarchy. Scale factor controls zoom level. Agent dots rendered as child entities of minimap camera. |
| **NeoTrix Mapping** | Minimap = second Camera2d with wide orthographic view, rendering agents as colored dots. Agent colors from EmotionLabel. Minimap frame rendered as UI element. Click minimap → teleport main camera. KB graph edges rendered as connection lines on minimap. |

## 17. Game Path Visualization

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/pathfinding?l=rust |
| **Key Insight** | Rust pathfinding: pathfinding crate (1.1k★) — A*, Dijkstra, Kuhn-Munkres. polyanya (514★) — navmesh pathfinding. vleue_navigator (489★) — "Pathfinding on NavMeshes for Bevy". seldom_map_nav (51★) — tilemap navmesh + pathfinding for Bevy. grid_pathfinding (28★) — jump point search. |
| **NeoTrix Mapping** | vleue_navigator is the Bevy-native pathfinding solution. Agent paths visualized as gizmo lines. pathfinding crate for algorithmic variety (A* for grid, Dijkstra for weighted graphs). KB graph paths visualized as animated edges. Path visualization = agent's planned action sequence rendered spatially. |

## 18. Game Graph Visualization

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/graph-database (search: visualization) |
| **Key Insight** | Graph visualization in games: force-directed layouts for social networks, hierarchical layouts for skill trees, spatial embedding for world maps. Bevy gizmos for line rendering. No dedicated Rust graph-viz game library. Most use egui debug overlays or custom shaders. |
| **NeoTrix Mapping** | NT-WORLD-SIM renders KB graph as force-directed layout overlay. Agent nodes = circles, edges = lines with thickness = relationship strength. Use Bevy gizmos for debug, custom mesh for production. ConsciousnessTree branches rendered as constellation pattern. Skill tree rendered as hierarchical graph. |

## 19. Game Event System

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy/latest/bevy/ecs/event/ |
| **Key Insight** | Bevy Events: type-erased, single-read consumption, buffered. Observers for reactive handling. `EventWriter<T>` / `EventReader<T>` for system access. `Trigger<E>` for observer propagation. Custom lifecycle events via Component Hooks. Event: automatic despawn after read. |
| **NeoTrix Mapping** | Bevy Events = NT-WORLD-SIM's agent communication channel. Agent actions emit events. Other agents observe/react via EventReader. Map: NT-CORE EventBus → Bevy Events. Observer pattern for GWT attention routing (only salient events propagate). Event history for episodic memory. |

## 20. Game State Management

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_state/latest/bevy_state/ |
| **Key Insight** | Bevy States: finite state machines as app-wide interdependent states. `States` trait + `OnEnter`/`OnExit` systems. `SubStates` for conditional states. `ComputedStates` for derived states. `StateScoped` entities auto-despawn on state exit. State transitions are deterministic. |
| **NeoTrix Mapping** | Map NT-CORE's state machines → Bevy States. Game states: Loading, MainMenu, Simulation, Paused, Debug. Sub-states: AgentFocus (Following, FreeRoaming, Inspecting). ComputedStates for derived data (e.g., "IsPaused" computed from Simulation state). StateScoped for UI panels. |

## 21. Game Save/Load Serialization

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/serde/latest/serde/ |
| **Key Insight** | Rust serde ecosystem: serde (10.8k★), serde_json (5.6k★), ron (4k★), bitcode (670★ binary), sonic-rs (919★ SIMD JSON). Bevy has `bevy_world_serialization` for ECS snapshot. RON (Rusty Object Notation) is Bevy's native config format. `serde_with` for custom de/serialization helpers. |
| **NeoTrix Mapping** | Save game = serialize Bevy World + KB state. Use RON for human-readable saves, bitcode for compact binary. Map: ECS world snapshot → serde Serialize, KB state → kv_store dump. `bevy_world_serialization` for ECS. Custom serializer for KB embeddings (skip large vectors, store references). |

## 22. Game Settings Menu

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_settings/latest/bevy_settings/ |
| **Key Insight** | Bevy Settings framework: `load_settings<T>()` / `save_settings<T>()`. Serde-based, file-backed. Supports nested structs, defaults, migration. `bevy_settings` crate provides `Settings<T>` resource. Combined with Bevy UI for runtime settings panels. |
| **NeoTrix Mapping** | GameSettings struct: resolution, fullscreen, audio volume, simulation speed, agent detail level, debug overlay toggle. Serde Serialize/Deserialize. File: `settings.ron`. UI: Bevy UI panel with sliders/checkboxes. Hot-reload via file watcher. Settings affect ECS systems via Run Conditions. |

## 23. Game Tutorial/Onboarding

| Field | Value |
|-------|-------|
| **URL** | https://github.com/bevyengine/bevy/tree/latest/examples/games (search: menu) |
| **Key Insight** | Bevy examples: `game_menu.rs` (simple game menu), `loading_screen.rs` (asset loading screen). Tutorial patterns: step-by-step overlay, highlight interactive elements, progress tracking. No built-in tutorial system in Bevy. Most games implement custom tutorial state machines. |
| **NeoTrix Mapping** | NT-WORLD-SIM onboarding: tutorial state with highlighted UI elements, tooltip popups, guided agent creation. Use Bevy State machine: `Tutorial` state with sub-states (Step1_CreateAgent, Step2_SetGoal, Step3_Observe). Tutorial completion stored in settings. Skip option for returning users. |

## 24. Game Physics 2D

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/physics-engine?l=rust |
| **Key Insight** | Bevy ecosystem: `bevy_rapier` (most popular), `avian` (successor to bevy_xpbd), `bevy_xpbd`. For NT-WORLD-SIM: lightweight physics sufficient (collision detection, basic rigid body). No need for full physics engine. Bevy's built-in spatial queries (raycast, overlap test) may suffice. |
| **NeoTrix Mapping** | Agent collision = spatial query (raycast or overlap). No rigid body physics needed (agents are point entities). Use Bevy's `MeshRayCast` for line-of-sight. Flow field pathfinding for crowd movement. Stigmergy visualization uses physics-like forces (attraction/repulsion between agents). |

## 25. Game Audio System

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_audio/latest/bevy_audio/ |
| **Key Insight** | Bevy Audio: `AudioPlayer<T>` / `AudioBundle`. Spatial audio 2D/3D. Decodable trait for custom sources. Soundtrack system (state-based music). Formats: OGG, WAV, MP3, FLAC, AAC, MP4. `bevy_audio` + `vorbis` minimum. `audio-all-formats` for full support. |
| **NeoTrix Mapping** | Audio layers: ambient (world background), agent (per-agent sounds), UI (click/hover), event (crisis/emergency). Spatial audio for agent proximity. State-based soundtrack (calm → tension → crisis). Audio volume settings per layer. Agent emotion affects audio pitch/speed. |

## 26. Game Networking/Multiplayer

| Field | Value |
|-------|-------|
| **URL** | https://github.com/AmbientRun/Ambient (search: multiplayer) |
| **Key Insight** | Ambient (3.9k★) — "The multiplayer game engine" in Rust with WASM. Ferrumc (2.4k★) — Minecraft server reimplementation with tokio async. Key pattern: ECS state sync via snapshot diffing. Deterministic lockstep for simulation. Entity interpolation for smooth rendering. |
| **NeoTrix Mapping** | NT-WORLD-SIM: single-player first, multiplayer later. Architecture: headless simulation server + Bevy clients. State sync via ECS snapshot diff (only changed components). Use tokio for async networking. Entity interpolation for remote agents. KB sync via delta updates. |

## 27. Game Asset Pipeline

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_asset/latest/bevy_asset/ |
| **Key Insight** | Bevy Assets: `AssetServer`, `Handle<T>`, asset processing pipeline. Features: hot-reloading (`file_watcher`), asset processor, custom asset loaders, compressed assets, embedded assets, web assets. `AssetSettings` for per-asset configuration. Multi-asset synchronization. |
| **NeoTrix Mapping** | Asset pipeline: sprites (agent avatars), tilesets (world), fonts (UI), audio (SFX/music), shaders (effects). Bevy's `AssetServer` loads from `assets/` directory. Hot-reloading for development. Asset processor for texture compression. Custom loader for KB-sourced assets (dynamic sprite generation from agent state). |

## 28. Game Modding Support

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/modding (search: rust) |
| **Key Insight** | Rust modding patterns: scripting (Lua via `mlua`/`rlua`), dynamic loading (`libloading`), DSL (custom languages). Bevy's plugin system as mod interface. `bevy_hotpatching` for runtime system replacement. No dominant Rust modding framework. Most use Lua or custom DSLs. |
| **NeoTrix Mapping** | Modding via: (1) Bevy plugins as mod units, (2) Lua scripting via `mlua` for behavior mods, (3) TOML/RON config files for parameter mods, (4) KB namespace extension for data mods. Plugin registry in KB. Hot-reload via `bevy_hotpatching`. Mod conflicts detected via KB consistency checks. |

## 29. Game Analytics

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/game-analytics (search: rust) |
| **Key Insight** | Game analytics patterns: event logging (actions, timestamps, outcomes), session recording, heatmap generation, behavioral clustering. No Rust-specific game analytics library. Most use custom event pipelines + database storage. Key: minimal overhead, async writes, batch processing. |
| **NeoTrix Mapping** | NT-WORLD-SIM analytics = NT-MEMORY event logging + NT-WORLD visualization. Agent behavior events → KB event log. Heatmap: aggregate agent positions over time. Behavioral clusters: group agents by action patterns. Analytics dashboard: Tauri webview with charts. Metrics: agent survival rate, goal completion, social graph density. |

## 30. Game Performance Optimization (Rust)

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy/latest/bevy/ (search: profiling) |
| **Key Insight** | Bevy performance: `trace_tracy` for profiling, `trace_chrome` for Chrome Tracing. `debug` feature for system timing. `multi_threaded` for parallelism. `dynamic_linking` for faster iteration. ECS archetypes for cache-friendly iteration. Bevy's schedule executor runs systems in parallel when no conflicts. |
| **NeoTrix Mapping** | Profile with Tracy. Target: 60fps with 1000+ agents. Optimization: (1) archetype-aware queries for cache locality, (2) change detection to skip unchanged agents, (3) fixed timestep for simulation (30Hz), variable for rendering (60Hz), (4) level-of-detail (simplify distant agents), (5) batch entity spawning. KB queries cached locally per agent. |

---

## Summary: Technology Stack for NT-WORLD-SIM

| Layer | Technology | Status |
|-------|-----------|--------|
| **Engine** | Bevy 0.19.1 | ✅ Primary choice |
| **ECS** | bevy_ecs | ✅ Built-in |
| **Rendering** | bevy_sprite + bevy_sprite_render | ✅ Built-in |
| **UI/HUD** | bevy_ui + bevy_feathers | ✅ Built-in |
| **State** | bevy_state | ✅ Built-in |
| **Audio** | bevy_audio | ✅ Built-in |
| **Settings** | bevy_settings | ✅ Built-in |
| **Serialization** | serde + ron | ✅ Ecosystem |
| **Pathfinding** | vleue_navigator | ✅ Bevy-native |
| **Behavior Trees** | bonsai | ✅ Rust + Bevy |
| **GOAP** | dogoap | ✅ Bevy-native |
| **Desktop Shell** | Tauri 2.0 | ✅ UI panels |
| **Networking** | tokio + custom | 🔧 Future |
| **Modding** | mlua + bevy plugins | 🔧 Future |
| **Analytics** | Custom + KB | 🔧 Build |

## Key Patterns Absorbed

| Pattern | Source | NeoTrix Mapping |
|---------|--------|-----------------|
| Headless core + visualization | frame-engine | `nt_world_sim` (logic) + `nt_world_sim_bevy` (render) |
| BT + GOAP hybrid AI | bonsai + dogoap | Behavior tree for execution, GOAP for planning |
| Agent memory = KB + blackboard | MATSim + general ABM | KB for long-term, Bevy Resource for working memory |
| Social dynamics = graph edges | MATSim + Acclimate | KB edges with weight/type/decay |
| Evolution = fitness + mutation | Evoplex + general EA | NT-MIND SEAL pipeline as evolution loop |
| Minimap via RTT camera | Bevy community | Second Camera2d + UI sprite |
| Save = ECS snapshot + KB | Bevy world_serialization + serde | Combined serialization |
| Modding via plugin + scripting | Bevy plugins + mlua | Plugin registry in KB |
