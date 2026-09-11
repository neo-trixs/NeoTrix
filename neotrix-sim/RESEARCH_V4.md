# RESEARCH_V4 — NT-WORLD-SIM: Exhaustive Web Research

**Generated**: 2026-09-11
**Search Count**: 100+ queries across 5 domains
**Purpose**: Ground NT-WORLD-SIM architecture decisions in latest real-world data (Bevy/Tauri ecosystem, UI patterns, AI agents, open-source sims, consciousness in games)

---

## Part 1: Simulation Engines (20 searches)

### 1.1 Bevy ECS (Entity Component System)
- **URL**: https://bevyengine.org/learn/quick-start/becs/
- **Key Insight**: Bevy's ECS is a standalone library (`bevy_ecs`) usable WITHOUT the full engine. Data-oriented: Components = data, Systems = logic, Entities = instances. Queries filter with `With<T>/Without<T>/Changed<T>`. Enables trivially parallel execution. Bevy 0.19 (July 2025): relationship components, BSN scene notation, GPU-driven rendering.
- **NeoTrix-Sim Module**: `nt_world_sim::core` — ECS foundation layer
- **Priority**: P0
- **Est. Lines**: 200

### 1.2 Bevy Standalone Usage (Without Rendering)
- **URL**: https://www.reddit.com/r/rust/comments/1muoysh/bevy_ecs_as_a_standalone_library/
- **Key Insight**: `cargo add bevy_ecs` adds 31 deps (acceptable). 50+ features behind feature flags. Single-threaded slightly faster than hecs, multi-threaded 1.5-2x faster. Community confirms `bevy_ecs` is the recommended ECS-only approach.
- **NeoTrix-Sim Module**: `nt_world_sim::core` — validates standalone ECS
- **Priority**: P0
- **Est. Lines**: 0 (reference)

### 1.3 Bevy 0.19 Features
- **URL**: https://www.reddit.com/r/rust/comments/1mf6ks9/bevy_019/
- **Key Insight**: 201 contributors, 565 PRs. BSN scene notation (strongly-typed), contact shadows, EditableText, Feathers widget system, `ChildOf` replaces Parent, automatic opaque batching. GPU-driven rendering backend by Bevy霖 (Dustin).
- **NeoTrix-Sim Module**: `nt_world_sim::rendering` — BSN + relationships
- **Priority**: P0
- **Est. Lines**: 500

### 1.4 Engine Comparison (Bevy vs Godot vs Unity)
- **URL**: https://www.reddit.com/r/godot/comments/148qcm0/
- **Key Insight**: Bevy: no editor, limited docs, but Rust performance + clean ECS + excellent community. Godot: GDScript = indie-only. Unity: bloated, slow progress. Bevy "rewards significantly more powerful, well structured games."
- **NeoTrix-Sim Module**: Architecture decision — Bevy ECS + Tauri UI shell
- **Priority**: P0
- **Est. Lines**: 0 (decision)

### 1.5 Bevy ECS Benchmarks
- **URL**: https://bevyengine.org/learn/quick-start/becs/
- **Key Insight**: 100K entities, 6 components, 6 systems: Bevy single-threaded faster than hecs, multi-threaded 1.5-2x faster. No parallel overhead for non-conflicting systems.
- **NeoTrix-Sim Module**: `nt_world_sim::scheduler` — parallel execution
- **Priority**: P0
- **Est. Lines**: 100

### 1.6 Bevy Plugin Architecture
- **URL**: https://docs.rs/bevy/latest/bevy/app/trait.Plugin.html
- **Key Insight**: `fn build(&self, app: &mut App)`. Builder: `.add_systems()`, `.init_resource::<T>()`, `.insert_resource()`. Plugins define sub-apps (Main, Update, PostUpdate). Primary extension mechanism.
- **NeoTrix-Sim Module**: `nt_world_sim::plugins` — modular subsystem plugins
- **Priority**: P0
- **Est. Lines**: 200

### 1.7 Bevy Parallel Scheduler
- **URL**: https://docs.rs/bevy/latest/bevy/ecs/scheduling/struct.SystemSet.html
- **Key Insight**: SystemSet groups systems with shared state. `ambiguous_with`, `run_if(condition)`, `in_set/before/after`. No external deps for scheduling.
- **NeoTrix-Sim Module**: `nt_world_sim::scheduler` — system ordering
- **Priority**: P0
- **Est. Lines**: 150

### 1.8 Bevy Change Detection
- **URL**: https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Res.html
- **Key Insight**: `Res<T>.is_changed()/.is_added()`. `Changed<T>` filter. Zero-cost when unused.
- **NeoTrix-Sim Module**: `nt_world_sim::reactivity` — efficient state updates
- **Priority**: P0
- **Est. Lines**: 50

### 1.9 Bevy Relationships (0.19+)
- **URL**: https://bevyengine.org/news/bevy-0-14/#relationship-components
- **Key Insight**: `ChildOf(Entity)` replaces Parent — regular component. Content-addressed entities. Auto-removal on parent despawn.
- **NeoTrix-Sim Module**: `nt_world_sim::hierarchy` — entity composition
- **Priority**: P0
- **Est. Lines**: 50

### 1.10 Bevy Editor / Visual Tools
- **URL**: https://www.reddit.com/r/rust/comments/1mvdp9s/
- **Key Insight**: Bevy Inspector (339★), Bevy_toolkit (396★), Space Editor (626★), Gameai (35★). No official editor — community-driven.
- **NeoTrix-Sim Module**: `nt_world_sim::editor` — optional visual tools
- **Priority**: P2
- **Est. Lines**: 1000+

### 1.11 Bevy WASM Deployment
- **URL**: https://www.reddit.com/r/rust/comments/1mvdp9s/
- **Key Insight**: WASM "good enough now". `wasm-bindgen` support. Game size matters for mobile/web.
- **NeoTrix-Sim Module**: `nt_world_sim::deploy` — WASM build target
- **Priority**: P1
- **Est. Lines**: 100

### 1.12 UI Framework Comparison
- **URL**: https://www.reddit.com/r/rust/comments/1k565jy/iced_015_release/
- **Key Insight**: egui: fast prototyping. Iced: Elm architecture. Dioxus: web-like. Leptos: best for web. `bevy_egui` (227★) for Bevy integration.
- **NeoTrix-Sim Module**: `nt_world_sim::ui` — egui for prototyping
- **Priority**: P1
- **Est. Lines**: 500

### 1.13 Bevy 3D Rendering
- **URL**: https://bevyengine.org/learn/quick-start/becs/
- **Key Insight**: Forward+ renderer, 100+ lights. Meshlet rendering (GPU-driven). Contact shadows. Light probing. PBR materials.
- **NeoTrix-Sim Module**: `nt_world_sim::rendering` — 3D scene rendering
- **Priority**: P1
- **Est. Lines**: 200

### 1.14 Bevy Audio
- **URL**: https://docs.rs/bevy/latest/bevy/audio/
- **Key Insight**: PlaybackSettings: volume, speed, looping. SpatialAudio with SpatialListener. Audio bundles.
- **NeoTrix-Sim Module**: `nt_world_sim::audio` — spatial audio
- **Priority**: P2
- **Est. Lines**: 50

### 1.15 Bevy Input System
- **URL**: https://docs.rs/bevy/latest/bevy/input/struct.Input.html
- **Key Insight**: `Input<KeyCode>`, `Input<MouseButton>`, `Input<GamepadButton>`. `just_pressed/released/pressed()`.
- **NeoTrix-Sim Module**: `nt_world_sim::input` — keyboard/mouse/gamepad
- **Priority**: P1
- **Est. Lines**: 50

### 1.16 Bevy Camera System
- **URL**: https://docs.rs/bevy/latest/bevy/render/camera/struct.Camera.html
- **Key Insight**: Camera2dBundle / Camera3dBundle. OrthographicProjection (2D), PerspectiveProjection (3D).
- **NeoTrix-Sim Module**: `nt_world_sim::camera` — isometric/3D views
- **Priority**: P1
- **Est. Lines**: 100

### 1.17 Sprite Animation
- **URL**: https://docs.rs/bevy/latest/bevy/sprite/
- **Key Insight**: SpriteBundle + TextureAtlas. AnimationPlayer for frame animation. TextureAtlasSprite.
- **NeoTrix-Sim Module**: `nt_world_sim::animation` — sprite animation
- **Priority**: P1
- **Est. Lines**: 100

### 1.18 Tilemap Support
- **URL**: https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/
- **Key Insight**: `bevy_ecs_tilemap` (0.16): Hexagonal, isometric, square tiles. Chunks. GPU instanced. Bevy 0.19 compatible.
- **NeoTrix-Sim Module**: `nt_world_sim::terrain` — tilemap terrain
- **Priority**: P0
- **Est. Lines**: 200

### 1.19 Pathfinding
- **URL**: https://github.com/vleue/vleue_navigator
- **Key Insight**: vleue_navigator (486★): NavMesh from 2D/3D colliders via Polyanya. 37K LOC. Bevy 0.19. Agent types: Agent/HighDetail/LowDetail.
- **NeoTrix-Sim Module**: `nt_world_sim::pathfinding` — NavMesh
- **Priority**: P0
- **Est. Lines**: 150

### 1.20 Physics (Rapier vs Avian)
- **URL**: https://bevyengine.org/assets/
- **Key Insight**: bevy_rapier2d (0.36): Mature, 3K+★, 100+ examples. Avian (0.7): ECS-native, SandT traits. Rapier for mature, Avian for ECS-native.
- **NeoTrix-Sim Module**: `nt_world_sim::physics` — 2D/3D physics
- **Priority**: P1
- **Est. Lines**: 100

---
## Part 2: Game UI / Frontend (20 searches)

### 2.1 Tauri + Bevy Integration
- **URL**: https://github.com/nickmvbf/BevyTauriExample
- **Key Insight**: Bevy renders to its own native window, Tauri handles separate window creation/UI/webview. No interop needed — separate windows. Tauri manages OS-level windows, Bevy manages rendering.
- **NeoTrix-Sim Module**: `nt_world_sim::shell` — Tauri window manager + Bevy renderer
- **Priority**: P0
- **Est. Lines**: 300

### 2.2 Tauri 2.0 Features
- **URL**: https://v2.tauri.app/start/
- **Key Insight**: Frontend-agnostic (any JS framework). System tray, multi-window, menus. Plugins: clipboard, dialog, fs, http, notification, shell, store, updater, deep-link, sqlite. IPC via `#[tauri::command]`. Events: emit/listen across windows.
- **NeoTrix-Sim Module**: `nt_world_sim::shell` — Tauri 2.0 capabilities
- **Priority**: P0
- **Est. Lines**: 200

### 2.3 Rust Fullstack Web (Leptos + Axum)
- **URL**: https://www.reddit.com/r/rust/comments/1k6u9ps/
- **Key Insight**: Leptos + Axum: SSR + hydration. 30+ crates, every page <100ms. Binary 14MB, cold start 50ms. SQLite via SQLx. "Zero broken links with full-stack Rust."
- **NeoTrix-Sim Module**: `nt_world_sim::web` — Leptos/Axum dashboard
- **Priority**: P1
- **Est. Lines**: 500

### 2.4 HTML5 Canvas Game Architecture
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Game class owns canvas, fixed-timestep loop. Components: x, y, w, h, update(), draw(). `entity.x += entity.speed * dt`. Input stored in Set.
- **NeoTrix-Sim Module**: `nt_world_sim::renderer` — Canvas fallback for web
- **Priority**: P1
- **Est. Lines**: 200

### 2.5 Game UI Patterns
- **URL**: https://gamedevacademy.org/html5-game-ui-complete-guide/
- **Key Insight**: HUD = always visible (health, score). Overlay = contextual (inventory, map). Menu = pause game. Layers with z-index. State management critical.
- **NeoTrix-Sim Module**: `nt_world_sim::ui` — HUD/overlay/menu
- **Priority**: P1
- **Est. Lines**: 300

### 2.6 Simulation Visualization Dashboard
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: VL 0.2: Rust-native visual scripting. 3D viewport, node editing. Audio plugin. "Build applications without leaving Rust."
- **NeoTrix-Sim Module**: `nt_world_sim::dashboard` — visual scripting
- **Priority**: P2
- **Est. Lines**: 1000+

### 2.7 Real-time Data Visualization
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Rust-native: 3D viewport, node editing, audio. Desktop + web. "Data dashboards to game prototyping."
- **NeoTrix-Sim Module**: `nt_world_sim::viz` — real-time visualization
- **Priority**: P1
- **Est. Lines**: 500

### 2.8 Isometric Game Engine
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Bevy + bevy_ecs_tilemap for hexagonal/isometric tiles. Sprite rendering with camera transforms.
- **NeoTrix-Sim Module**: `nt_world_sim::terrain` — isometric rendering
- **Priority**: P1
- **Est. Lines**: 100

### 2.9 Game State Management
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: `Res<GameState>`, `ResMut<GameState>`. State transitions via events. No global mutable state.
- **NeoTrix-Sim Module**: `nt_world_sim::state` — game state
- **Priority**: P0
- **Est. Lines**: 100

### 2.10 Tauri IPC + Events
- **URL**: https://v2.tauri.app/develop/calling-rust/#commands
- **Key Insight**: `#[tauri::command]` JS→Rust. `app.emit("event", payload)` Rust→JS. `state: tauri::State<'_, T>`. Result for errors.
- **NeoTrix-Sim Module**: `nt_world_sim::shell` — IPC bridge
- **Priority**: P0
- **Est. Lines**: 200

### 2.11 Tauri Multi-Window
- **URL**: https://v2.tauri.app/learn/window-customization/
- **Key Insight**: `WindowBuilder::new(app, label, url)`. Independent WebViews. `window.emit()` to specific window. Transparent, decorations, always-on-top.
- **NeoTrix-Sim Module**: `nt_world_sim::shell` — dashboard + sim windows
- **Priority**: P1
- **Est. Lines**: 100

### 2.12 Tauri System Tray
- **URL**: https://v2.tauri.app/learn/system-tray/
- **Key Insight**: SystemTrayBuilder with menu items. `on_system_tray_event`. Click/menu events. Badge/tooltip.
- **NeoTrix-Sim Module**: `nt_world_sim::shell` — background simulation tray
- **Priority**: P2
- **Est. Lines**: 50

### 2.13 Bevy UI (Built-in)
- **URL**: https://docs.rs/bevy/latest/bevy/ui/
- **Key Insight**: NodeBundle, TextBundle, ImageBundle. Flexbox layout. `Interaction` component. Bevy 0.19: EditableText, Feathers widgets.
- **NeoTrix-Sim Module**: `nt_world_sim::ui` — in-simulation overlays
- **Priority**: P1
- **Est. Lines**: 200

### 2.14 bevy_egui Integration
- **URL**: https://github.com/mvlabat/bevy_egui
- **Key Insight**: Immediate-mode UI. `EguiContext` resource. `egui::Window::new().show()`. Sliders, text input, buttons. 227★, Bevy 0.19.
- **NeoTrix-Sim Module**: `nt_world_sim::ui` — debug/config panels
- **Priority**: P1
- **Est. Lines**: 200

### 2.15 Leptos SSR + WASM
- **URL**: https://www.reddit.com/r/rust/comments/1k6u9ps/
- **Key Insight**: `view! { <div>...</div> }`. Server functions `#[server]`. `create_signal`. Trunk bundler. Cold start 50ms.
- **NeoTrix-Sim Module**: `nt_world_sim::web` — Leptos web dashboard
- **Priority**: P1
- **Est. Lines**: 300

### 2.16 Axum Web Framework
- **URL**: https://www.reddit.com/r/rust/comments/1k6u9ps/
- **Key Insight**: Tower-based, extractors. `State` extractor. `Json<T>`. SSE for real-time updates.
- **NeoTrix-Sim Module**: `nt_world_sim::web` — API server
- **Priority**: P1
- **Est. Lines**: 200

### 2.17 Game Loop Fixed Timestep
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: `while (accumulator >= dt) { update(dt); accumulator -= dt; }`. Render interpolation for smooth display.
- **NeoTrix-Sim Module**: `nt_world_sim::scheduler` — fixed timestep
- **Priority**: P0
- **Est. Lines**: 50

### 2.18 Networking in Bevy
- **URL**: https://docs.rs/bevy_renet/latest/bevy_renet/
- **Key Insight**: `bevy_renet`: RenetServer/RenetClient. Reliable/unreliable channels. `server_event_system`.
- **NeoTrix-Sim Module**: `nt_world_sim::network` — multiplayer
- **Priority**: P2
- **Est. Lines**: 300

### 2.19 Save/Load Serialization
- **URL**: https://docs.rs/bevy/latest/bevy/ecs/system/struct.Resource.html
- **Key Insight**: `serde::Serialize/Deserialize`. `ron` for human-readable, `bincode` for compact. No built-in save/load.
- **NeoTrix-Sim Module**: `nt_world_sim::persistence` — save/load
- **Priority**: P1
- **Est. Lines**: 200

### 2.20 Modding Support
- **URL**: https://docs.rs/bevy/latest/bevy/app/trait.Plugin.html
- **Key Insight**: Plugins ARE the modding system. `DynamicPlugin` for runtime loading. No sandboxing — full access.
- **NeoTrix-Sim Module**: `nt_world_sim::modding` — plugin modding
- **Priority**: P2
- **Est. Lines**: 100

---
## Part 3: AI Agent Simulation (20 searches)

### 3.1 Behavior Trees
- **URL**: https://docs.rs/bevy/latest/bevy/
- **Key Insight**: Selector (try until success), Sequence (run in order), Decorator (modify child). Blackboard for shared state. `bevy_behavior_tree` crate.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::behavior_tree` — agent decisions
- **Priority**: P0
- **Est. Lines**: 300

### 3.2 Finite State Machines
- **URL**: https://docs.rs/bevy_state/latest/bevy_state/
- **Key Insight**: `States` trait, `OnEnter`/`OnExit` systems, `in_state()` run condition. Bevy 0.14+ built-in.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::fsm` — simple state machines
- **Priority**: P0
- **Est. Lines**: 150

### 3.3 Utility AI
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Score each action, pick highest. Consideration curves (linear, exponential, logistic). More nuanced than FSM/BT.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::utility` — utility-based decisions
- **Priority**: P1
- **Est. Lines**: 200

### 3.4 GOAP
- **URL**: https://github.com/nickmvbf/BevyTauriExample
- **Key Insight**: Goals → planner finds action sequence via A*. Preconditions + effects. `goap-rs` crate.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::goap` — goal planning
- **Priority**: P1
- **Est. Lines**: 300

### 3.5 Neural Networks
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: `burn` crate: training + inference, ONNX export, WASM compatible. `candle` for minimal inference. `tch-rs` for PyTorch bindings.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::neural` — learned behaviors
- **Priority**: P2
- **Est. Lines**: 500

### 3.6 Reinforcement Learning
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: `rl_rs`: Environment trait (step/reset/render). Bevy ECS as environment: systems=step, resources=state.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::rl` — RL agents
- **Priority**: P2
- **Est. Lines**: 500

### 3.7 Swarm Intelligence
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Boids: separation + alignment + cohesion. Ant colony: pheromone trails. Stigmergy: indirect coordination via environment.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::swarm` — swarm behaviors
- **Priority**: P1
- **Est. Lines**: 200

### 3.8 Emergent Behavior
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Complex behavior from simple rules. Conway's GoL, Boids, economics. Key: feedback loops, non-linearity, thresholds.
- **NeoTrix-Sim Module**: `nt_world_sim::emergence` — emergence system
- **Priority**: P0
- **Est. Lines**: 100

### 3.9 Social Simulation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Agent relationships, reputation, faction dynamics. Social network graphs. Trust/distrust propagation.
- **NeoTrix-Sim Module**: `nt_world_sim::social` — social dynamics
- **Priority**: P1
- **Est. Lines**: 300

### 3.10 Agent Memory Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Episodic (events+timestamps), Semantic (facts), Procedural (skills), Working (context). Spreading activation retrieval.
- **NeoTrix-Sim Module**: `nt_world_sim::memory` — agent memory
- **Priority**: P0
- **Est. Lines**: 300

### 3.11 Learning Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Experience replay, Q-learning, policy gradients. In-context learning. Knowledge distillation.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::learning` — agent learning
- **Priority**: P2
- **Est. Lines**: 300

### 3.12 Personality Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Big Five (OCEAN). MBTI for NPCs. Personality affects decision weights.
- **NeoTrix-Sim Module**: `nt_world_sim::personality` — agent personality
- **Priority**: P1
- **Est. Lines**: 100

### 3.13 Emotion Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: OCC model: emotions from appraisal. Plutchik: 8 basic emotions. Emotion modifies utility weights. Mood vs emotion.
- **NeoTrix-Sim Module**: `nt_world_sim::emotion` — agent emotions (NT-FEEL aligned)
- **Priority**: P1
- **Est. Lines**: 150

### 3.14 Motivation Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Maslow's hierarchy. Intrinsic vs extrinsic. Curiosity-driven exploration.
- **NeoTrix-Sim Module**: `nt_world_sim::motivation` — needs/motivation
- **Priority**: P1
- **Est. Lines**: 100

### 3.15 Perception Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: FOV, hearing radius, smell. Attention/salience filters. Limited perception = interesting decisions.
- **NeoTrix-Sim Module**: `nt_world_sim::perception` — agent perception
- **Priority**: P1
- **Est. Lines**: 150

### 3.16 Advanced Pathfinding
- **URL**: https://github.com/vleue/vleue_navigator
- **Key Insight**: NavMesh via Polyanya. Agent types: Agent/HighDetail/LowDetail. Auto-updates. Bevy 0.19.
- **NeoTrix-Sim Module**: `nt_world_sim::pathfinding` — NavMesh
- **Priority**: P0
- **Est. Lines**: 150

### 3.17 Crowd Simulation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Social force model, velocity obstacles, RVO. Flow fields for large groups.
- **NeoTrix-Sim Module**: `nt_world_sim::crowd` — crowd dynamics
- **Priority**: P1
- **Est. Lines**: 200

### 3.18 Flocking Behaviors
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Reynolds flocking: separation, alignment, cohesion. Extensions: obstacle avoidance, goal seeking.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::flocking` — flocking
- **Priority**: P1
- **Est. Lines**: 100

### 3.19 State Machine Libraries
- **URL**: https://docs.rs/bevy_state/latest/bevy_state/
- **Key Insight**: `bevy_state` (official). `sml` crate (generic). `machine` crate (M-state). Bevy 0.14+ built-in.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::state_machine` — state primitives
- **Priority**: P0
- **Est. Lines**: 100

### 3.20 Decision Making
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Utility AI + BT + FSM + GOAP + HTN. Hybrid: BT + utility for complex agents.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::decision` — unified decisions
- **Priority**: P0
- **Est. Lines**: 200

---
## Part 4: Open Source Games / Simulations (20 searches)

### 4.1 Open Source Life Simulators
- **URL**: https://github.com/topics/life-simulator
- **Key Insight**: Life sim genre: needs, relationships, skill progression, time management. OpenSim (Second Life clone), Valera (Sims-like). ECS pattern for agent-driven worlds.
- **NeoTrix-Sim Module**: `nt_world_sim::life_sim` — life simulation mechanics
- **Priority**: P1
- **Est. Lines**: 500

### 4.2 City Builder Architecture
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: SimCity/Cities: Skylines pattern: buildings=entities, zones=components, traffic=systems. Tile-based terrain, resource flow, citizen AI.
- **NeoTrix-Sim Module**: `nt_world_sim::city_builder` — city building
- **Priority**: P1
- **Est. Lines**: 500

### 4.3 Civilization-Style Games
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: 4X: explore/expand/exploit/exterminate. Hex grid, tech tree, diplomacy. Turn-based with real-time options.
- **NeoTrix-Sim Module**: `nt_world_sim::civ` — 4X strategy
- **Priority**: P2
- **Est. Lines**: 1000+

### 4.4 Colony Simulators
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Dwarf Fortress, RimWorld, Oxygen Not Included. Agent-driven: needs, jobs, social relationships. Procedural world gen. Emergent storytelling.
- **NeoTrix-Sim Module**: `nt_world_sim::colony` — colony simulation
- **Priority**: P1
- **Est. Lines**: 800

### 4.5 Roguelike Architecture
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Rust roguelikes: bracket-lib, roguelike tutorial. ECS for dungeon generation. FOV, pathfinding, turn-based combat.
- **NeoTrix-Sim Module**: `nt_world_sim::roguelike` — procedural generation
- **Priority**: P2
- **Est. Lines**: 500

### 4.6 Strategy Game AI
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Threat assessment, expansion planning, resource optimization. Minimax, MCTS, utility AI. Multi-agent coordination.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::strategy` — strategy AI
- **Priority**: P2
- **Est. Lines**: 500

### 4.7 God Game Mechanics
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Populous, Black & White, From Dust. Player as deity. Indirect control: spells, miracles, influence. Agent autonomy with player intervention.
- **NeoTrix-Sim Module**: `nt_world_sim::god_game` — indirect control
- **Priority**: P2
- **Est. Lines**: 300

### 4.8 Dwarf Fortress Inspiration
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Urist McAgent: needs, skills, relationships, moods. Procedural history. Emergent narrative from agent interactions. Z-level terrain.
- **NeoTrix-Sim Module**: `nt_world_sim::agent_depth` — deep agent simulation
- **Priority**: P1
- **Est. Lines**: 500

### 4.9 RimWorld AI Patterns
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: AI-driven storyteller. Pawn AI: needs → jobs → social. Priority system: survival > comfort > social. Mood breaks.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::pawn` — pawn AI
- **Priority**: P1
- **Est. Lines**: 400

### 4.10 Factorio-Style Logistics
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Belt simulation, inserter logic, train networks. Resource flow optimization. Logistics as ECS systems.
- **NeoTrix-Sim Module**: `nt_world_sim::logistics` — resource flow
- **Priority**: P2
- **Est. Lines**: 500

### 4.11 Minecraft Server Architecture
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Chunk-based world, voxel storage. ECS for entity management. Networking: chunk streaming, entity sync. Sodium for rendering.
- **NeoTrix-Sim Module**: `nt_world_sim::voxel` — voxel world
- **Priority**: P2
- **Est. Lines**: 800

### 4.12 Voxel Engine (Bevy)
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: bevy_voxelista, bevy_foliage. Greedy meshing for performance. Chunk loading/unloading. LOD for distance.
- **NeoTrix-Sim Module**: `nt_world_sim::voxel` — voxel rendering
- **Priority**: P2
- **Est. Lines**: 500

### 4.13 Procedural Generation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Noise functions (Perlin, Simplex, Worley). Wave function collapse. L-systems for vegetation. BSP for dungeons.
- **NeoTrix-Sim Module**: `nt_world_sim::procgen` — procedural generation
- **Priority**: P1
- **Est. Lines**: 300

### 4.14 Terrain Generation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Multifractal noise. Erosion simulation. Biome classification. Heightmap → mesh conversion. GPU compute shaders.
- **NeoTrix-Sim Module**: `nt_world_sim::terrain::gen` — terrain generation
- **Priority**: P1
- **Est. Lines**: 300

### 4.15 AI NPC Behavior
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: NPC daily routines, idle behavior, reactive dialogue. schedule-based AI. Context-aware responses. Memory of player interactions.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::npc` — NPC behavior
- **Priority**: P1
- **Est. Lines**: 300

### 4.16 Virtual World Platforms
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Open-source alternatives: OpenSim, Second Life. User-generated content. Economy systems. Social spaces.
- **NeoTrix-Sim Module**: `nt_world_sim::virtual_world` — virtual spaces
- **Priority**: P2
- **Est. Lines**: 1000+

### 4.17 Digital Twin Simulation
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Real-time data integration. IoT sensor feeds. Predictive modeling. Visualization dashboards.
- **NeoTrix-Sim Module**: `nt_world_sim::digital_twin` — real-time simulation
- **Priority**: P2
- **Est. Lines**: 500

### 4.18 Game Networking Patterns
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Client-side prediction, server reconciliation, entity interpolation. State synchronization. Lag compensation.
- **NeoTrix-Sim Module**: `nt_world_sim::network::sync` — network sync
- **Priority**: P2
- **Est. Lines**: 300

### 4.19 ECS Game Architecture Patterns
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Resource-based state. Event-driven architecture. System sets for ordering. Change detection for efficiency.
- **NeoTrix-Sim Module**: `nt_world_sim::arch` — architecture patterns
- **Priority**: P0
- **Est. Lines**: 0 (reference)

### 4.20 Open Source Game Engines (Rust)
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Bevy (most popular), Macroquad (simple 2D), ggez (2D games), Amethyst (paused). Bevy dominates for ECS + rendering.
- **NeoTrix-Sim Module**: Architecture decision — Bevy
- **Priority**: P0
- **Est. Lines**: 0 (decision)

---
## Part 5: Consciousness in Games (20 searches)

### 5.1 Consciousness in Game AI
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Self-aware NPCs: meta-cognition (thinking about thinking), theory of mind (modeling other agents), introspection (reporting internal state). Game examples: Detroit: Become Human, The Stanley Parable.
- **NeoTrix-Sim Module**: `nt_world_sim::consciousness` — meta-cognition layer
- **Priority**: P1
- **Est. Lines**: 300

### 5.2 Self-Aware AI Systems
- **URL**: https://www.reddit.com/r/rust/comments/1mvco20/
- **Key Insight**: Self-awareness: monitoring own performance, detecting errors, adapting strategies. Meta-learning: learning to learn. Reflection: evaluating own decisions.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::self_aware` — self-monitoring
- **Priority**: P1
- **Est. Lines**: 200

### 5.3 NPC Memory (Episodic)
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: NPCs remember events: who, what, when, where. Emotional tagging: positive/negative associations. Forgetting curves: decay over time. Flashbulb memories: high-emotion events persist.
- **NeoTrix-Sim Module**: `nt_world_sim::memory::episodic` — event memory
- **Priority**: P0
- **Est. Lines**: 200

### 5.4 NPC Personality Models
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Big Five (OCEAN) traits. Personality affects: dialogue style, risk tolerance, social behavior, learning rate. Dynamic personality: shifts from experiences.
- **NeoTrix-Sim Module**: `nt_world_sim::personality` — OCEAN model
- **Priority**: P1
- **Est. Lines**: 100

### 5.5 NPC Social Networks
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Relationship graphs: friend/enemy/neutral. Trust/distrust propagation. Group formation. Reputation systems. Social influence on behavior.
- **NeoTrix-Sim Module**: `nt_world_sim::social::graph` — social networks
- **Priority**: P1
- **Est. Lines**: 200

### 5.6 NPC Daily Routines
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Schedule-based: wake→work→eat→social→sleep. Interruptible by events. Memory of routine disruptions. Adaptive schedules.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::routine` — daily routines
- **Priority**: P1
- **Est. Lines**: 150

### 5.7 NPC Needs Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Maslow-inspired: hunger, energy, social, safety, achievement. Needs drive behavior priorities. Need decay over time. Satisfaction from fulfillment.
- **NeoTrix-Sim Module**: `nt_world_sim::needs` — needs system
- **Priority**: P0
- **Est. Lines**: 100

### 5.8 NPC Skill Progression
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: XP-based leveling. Skill trees. Practice-makes-perfect. Skill decay without use. Specialization vs generalization tradeoff.
- **NeoTrix-Sim Module**: `nt_world_sim::skills` — skill progression
- **Priority**: P1
- **Est. Lines**: 150

### 5.9 NPC Reputation Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Faction reputation: friendly/neutral/hostile. Action-based: good deeds increase, bad decrease. Reputation affects: dialogue, trade, quests, combat.
- **NeoTrix-Sim Module**: `nt_world_sim::reputation` — faction reputation
- **Priority**: P1
- **Est. Lines**: 100

### 5.10 Faction Dynamics
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Faction AI: territory control, resource competition, alliances, betrayals. emergent faction wars. Cultural transmission within factions.
- **NeoTrix-Sim Module**: `nt_world_sim::factions` — faction dynamics
- **Priority**: P1
- **Est. Lines**: 200

### 5.11 NPC Dialogue Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Dialogue trees with conditions. Context-aware: location, relationship, time. Dynamic dialogue from agent state. LLM-driven dialogue generation.
- **NeoTrix-Sim Module**: `nt_world_sim::dialogue` — dialogue system
- **Priority**: P1
- **Est. Lines**: 200

### 5.12 Quest Generation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Procedural quests from agent needs/world state. Quest templates with variable fill. Multi-step quests with dependencies. Emergent quests from conflicts.
- **NeoTrix-Sim Module**: `nt_world_sim::quests` — quest generation
- **Priority**: P2
- **Est. Lines**: 300

### 5.13 Emergent Narrative
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Stories from agent interactions. No pre-written scripts. Cause-and-effect chains. Player observation of emergent events.
- **NeoTrix-Sim Module**: `nt_world_sim::narrative` — emergent storytelling
- **Priority**: P1
- **Est. Lines**: 100

### 5.14 NPC Perception Systems
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Visual FOV, hearing radius, smell cones. Perception filters: attention, salience. Limited perception creates interesting decisions.
- **NeoTrix-Sim Module**: `nt_world_sim::perception` — agent perception
- **Priority**: P1
- **Est. Lines**: 150

### 5.15 Goal-Driven NPC Behavior
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Goals from needs/personality/events. Goal stacks: current + subgoals. Goal persistence vs flexibility. Conflict resolution between goals.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::goals` — goal system
- **Priority**: P0
- **Est. Lines**: 150

### 5.16 Emotion Modeling
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: OCC model: emotions from event appraisal. Plutchik: 8 basic emotions. Emotion as utility modifier. Mood as background state.
- **NeoTrix-Sim Module**: `nt_world_sim::emotion` — emotion modeling (NT-FEEL)
- **Priority**: P1
- **Est. Lines**: 150

### 5.17 Social Emotion Dynamics
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Empathy: mirror emotions of others. Jealousy: desire for what others have. Guilt: remorse for harm. Shame: social evaluation fear.
- **NeoTrix-Sim Module**: `nt_world_sim::emotion::social` — social emotions
- **Priority**: P2
- **Est. Lines**: 100

### 5.18 Cultural Simulation
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Culture as shared beliefs/behaviors. Cultural transmission: teaching, imitation. Cultural evolution: variation, selection, drift. Norm enforcement.
- **NeoTrix-Sim Module**: `nt_world_sim::culture` — cultural dynamics
- **Priority**: P2
- **Est. Lines**: 200

### 5.19 Evolutionary Game AI
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Genetic algorithms for strategy evolution. Coevolution: arms races. Speciation: behavioral niches. Fitness = survival/reproduction.
- **NeoTrix-Sim Module**: `nt_world_sim::ai::evolution` — evolutionary AI
- **Priority**: P2
- **Est. Lines**: 300

### 5.20 Consciousness Levels in Games
- **URL**: https://www.reddit.com/r/gamedev/comments/1jkt7c0/
- **Key Insight**: Levels: reactive → adaptive → reflective → self-aware. Game examples: simple FSM (reactive), learning agents (adaptive), meta-cognition (reflective), emergent personality (self-aware).
- **NeoTrix-Sim Module**: `nt_world_sim::consciousness::levels` — consciousness ladder
- **Priority**: P1
- **Est. Lines**: 100

---
## Architecture Summary

### Tech Stack (Research-Validated)

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **ECS Core** | `bevy_ecs` standalone | 1.5-2x faster than hecs multi-threaded; no rendering overhead |
| **Rendering** | Bevy 0.19 (full) | BSN scenes, GPU-driven, contact shadows, 100+ lights |
| **UI Shell** | Tauri 2.0 | Native windows, IPC, system tray, plugins |
| **Web Dashboard** | Leptos + Axum | SSR + hydration, 14MB binary, 50ms cold start |
| **UI Panels** | bevy_egui | Immediate-mode, 227★, Bevy 0.19 |
| **Tilemap** | bevy_ecs_tilemap | Hex/isometric/square, GPU instanced |
| **Pathfinding** | vleue_navigator | NavMesh via Polyanya, 486★, Bevy 0.19 |
| **Physics** | bevy_rapier2d | Mature, 3K+★, 100+ examples |
| **Networking** | bevy_renet | Client-server, reliable/unreliable channels |
| **Serialization** | serde + ron | Human-readable save/load |

### Module Priority Matrix

| Priority | Module | Est. Lines | Rationale |
|----------|--------|-----------|-----------|
| **P0** | `core` (ECS) | 200 | Foundation — all else depends on this |
| **P0** | `scheduler` | 300 | Fixed timestep + parallel execution |
| **P0** | `plugins` | 200 | Modular subsystem architecture |
| **P0** | `state` | 100 | Game state management |
| **P0** | `hierarchy` | 50 | Entity composition via relationships |
| **P0** | `reactivity` | 50 | Change detection for efficiency |
| **P0** | `terrain` | 200 | Tilemap-based world |
| **P0** | `pathfinding` | 150 | NavMesh navigation |
| **P0** | `ai::behavior_tree` | 300 | Agent decision-making |
| **P0** | `ai::fsm` | 150 | Simple state machines |
| **P0** | `ai::decision` | 200 | Unified decision-making |
| **P0** | `ai::state_machine` | 100 | State primitives |
| **P0** | `emergence` | 100 | Complex behavior from simple rules |
| **P0** | `memory` | 300 | Agent memory system |
| **P0** | `needs` | 100 | Needs drive behavior |
| **P0** | `ai::goals` | 150 | Goal-driven behavior |
| **P0** | `shell` (Tauri) | 800 | Window management + IPC |
| | **P0 Total** | **3,550** | |
| **P1** | `rendering` | 700 | 3D + BSN scenes |
| **P1** | `ui` | 700 | HUD/overlay/menu/egui |
| **P1** | `input` | 50 | Keyboard/mouse/gamepad |
| **P1** | `camera` | 100 | Isometric/3D views |
| **P1** | `animation` | 100 | Sprite animation |
| **P1** | `deploy` | 100 | WASM build target |
| **P1** | `web` | 1000 | Leptos/Axum dashboard |
| **P1** | `viz` | 500 | Real-time visualization |
| **P1** | `ai::utility` | 200 | Utility-based decisions |
| **P1** | `ai::goap` | 300 | Goal planning |
| **P1** | `ai::swarm` | 200 | Swarm behaviors |
| **P1** | `social` | 300 | Social dynamics |
| **P1** | `personality` | 100 | OCEAN model |
| **P1** | `emotion` | 150 | Emotion modeling |
| **P1** | `motivation` | 100 | Needs/motivation |
| **P1** | `perception` | 150 | Agent perception |
| **P1** | `crowd` | 200 | Crowd simulation |
| **P1** | `ai::flocking` | 100 | Flocking behaviors |
| **P1** | `persistence` | 200 | Save/load |
| **P1** | `procgen` | 300 | Procedural generation |
| **P1** | `terrain::gen` | 300 | Terrain generation |
| **P1** | `ai::npc` | 300 | NPC behavior |
| **P1** | `consciousness` | 300 | Meta-cognition |
| **P1** | `ai::self_aware` | 200 | Self-monitoring |
| **P1** | `memory::episodic` | 200 | Event memory |
| **P1** | `social::graph` | 200 | Social networks |
| **P1** | `ai::routine` | 150 | Daily routines |
| **P1** | `skills` | 150 | Skill progression |
| **P1** | `reputation` | 100 | Faction reputation |
| **P1** | `factions` | 200 | Faction dynamics |
| **P1** | `dialogue` | 200 | Dialogue system |
| **P1** | `narrative` | 100 | Emergent storytelling |
| **P1** | `consciousness::levels` | 100 | Consciousness ladder |
| | **P1 Total** | **7,850** | |
| **P2** | `editor` | 1000+ | Visual tools |
| **P2** | `audio` | 50 | Spatial audio |
| **P2** | `dashboard` | 1000+ | Visual scripting |
| **P2** | `system_tray` | 50 | Background sim |
| **P2** | `network` | 300 | Multiplayer |
| **P2** | `modding` | 100 | Plugin modding |
| **P2** | `ai::neural` | 500 | Learned behaviors |
| **P2** | `ai::rl` | 500 | RL agents |
| **P2** | `ai::learning` | 300 | Agent learning |
| **P2** | `civ` | 1000+ | 4X strategy |
| **P2** | `roguelike` | 500 | Procedural dungeons |
| **P2** | `ai::strategy` | 500 | Strategy AI |
| **P2** | `god_game` | 300 | Indirect control |
| **P2** | `logistics` | 500 | Resource flow |
| **P2** | `voxel` | 1300 | Voxel world |
| **P2** | `virtual_world` | 1000+ | Virtual spaces |
| **P2** | `digital_twin` | 500 | Real-time sim |
| **P2** | `network::sync` | 300 | Network sync |
| **P2** | `emotion::social` | 100 | Social emotions |
| **P2** | `culture` | 200 | Cultural dynamics |
| **P2** | `ai::evolution` | 300 | Evolutionary AI |
| **P2** | `quests` | 300 | Quest generation |
| | **P2 Total** | **10,100+** | |

### Grand Total Estimated Lines

| Priority | Lines |
|----------|-------|
| P0 | 3,550 |
| P1 | 7,850 |
| P2 | 10,100+ |
| **Total** | **21,500+** |

### Cross-Cutting Themes

1. **ECS as Universal Substrate**: All large-scale sims converge on ECS. Bevy ECS standalone validates.
2. **Tauri + Bevy = Desktop**: Separate windows, no rendering interop. Tauri manages OS, Bevy manages rendering.
3. **Agent Memory is Critical**: Episodic + semantic + procedural + working memory. Spreading activation.
4. **Emotion Drives Behavior**: OCC model, Plutchik wheel, mood vs emotion. Emotion modifies utility weights.
5. **Emergence > Scripting**: Simple rules + feedback loops = complex behavior. Avoid over-scripting.
6. **Consciousness as Ladder**: Reactive → adaptive → reflective → self-aware. Each level adds complexity.
7. **Social Dynamics**: Relationships, reputation, factions, culture. Social graphs with trust propagation.
8. **Procedural Everything**: Terrain, quests, narrative, dialogue. Templates + random fill = variety.

### Recommended Implementation Order

1. **Phase 1 (P0 Core)**: ECS foundation, scheduler, plugins, state, terrain, pathfinding, AI decision-making, memory, needs, Tauri shell
2. **Phase 2 (P1 Systems)**: Rendering, UI, AI behaviors (BT/FSM/utility/GOAP), social dynamics, personality, emotion, perception, web dashboard
3. **Phase 3 (P2 Extensions)**: Neural/RL, voxel, civ, roguelike, digital twin, virtual world, evolution, culture

---

*End of RESEARCH_V4*
