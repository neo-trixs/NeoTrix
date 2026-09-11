# NeoTrix Research V5 - Tauri Game Development & Bevy ECS References

Generated: 2026-09-11
Search Count: 30

---

## Category 1: Tauri + Bevy Integration

### 1. BevyTauriExample
- **URL**: https://github.com/sunxfancy/BevyTauriExample
- **Key Insight**: Bevy rendered natively inside Tauri2 window. Tauri handles window management + HTML overlay, Bevy renders in native OS window via raw-window-handle. No Canvas2D overhead.
- **NeoTrix Mapping**: Reference architecture for hybrid Tauri+Bevy apps. Clean separation: Tauri for OS integration, Bevy for ECS/game logic.

### 2. BevyTauriTemplate
- **URL**: https://github.com/Susagna/BevyTauriTemplate
- **Key Insight**: Template for Tauri v2 + Bevy apps. Simple setup with both systems in same Cargo workspace.
- **NeoTrix Mapping**: Starter template pattern for NeoTrix sim desktop builds.

### 3. Tauri + Bevy 3D Integration
- **URL**: https://github.com/readyup/tauri-bevy
- **Key Insight**: Tauri for frontend/UI, Bevy for 3D rendering. WebSocket bridge for communication between frontend JS and Bevy backend.
- **NeoTrix Mapping**: WebSocket communication pattern for real-time data flow between UI and game engine.

### 4. Bevy + Canvas2D Hybrid
- **URL**: https://github.com/AnthonyTornetta/bevy_web_canvas
- **Key Insight**: Bevy games run on HTML Canvas2D. Allows 2D/3D rendering within browser context.
- **NeoTrix Mapping**: Alternative approach for web-based rendering if needed.

### 5. BevyGPUI Plugin
- **URL**: https://github.com/ZetaRet/bevy_gpui
- **Key Insight**: Composites Zed's GPUI UI framework as overlay on Bevy scenes. Combines GPU-accelerated UI with ECS game logic.
- **NeoTrix Mapping**: Potential for high-performance UI overlays in game contexts.

---

## Category 2: Tauri Game Development

### 6. Tauri 2.0 Game Development
- **URL**: https://v2.tauri.app/start/
- **Key Insight**: Tauri v2 provides native OS window management, system tray, file system access, and WebView for HTML/JS UI.
- **NeoTrix Mapping**: Tauri as desktop shell for NeoTrix sim - system integration without Electron overhead.

### 7. Tauri WebSocket Plugin
- **URL**: https://github.com/nicegamer7/tauri-plugin-websocket
- **Key Insight**: WebSocket plugin for Tauri v2. Enables real-time bidirectional communication between frontend and backend.
- **NeoTrix Mapping**: Communication layer for streaming game state updates from Bevy to Tauri UI.

### 8. Tauri State Management
- **URL**: https://v2.tauri.app/develop/state-management/
- **Key Insight**: Tauri v2 state management via `Manager` trait + `Mutex` for mutable state. Managed state shared across commands.
- **NeoTrix Mapping**: Pattern for sharing game state between Tauri commands and Bevy ECS.

### 9. Tauri IPC Commands
- **URL**: https://v2.tauri.app/develop/calling-rust/
- **Key Insight**: Tauri commands exposed to frontend via `#[tauri::command]`. State injection via `tauri::State<'_, T>`.
- **NeoTrix Mapping**: API surface for UI-to-game-engine communication.

### 10. Tauri Event System
- **URL**: https://v2.tauri.app/develop/calling-frontend/
- **Key Insight**: Tauri events for async communication. Frontend emits events, Rust handles via `on_event`.
- **NeoTrix Mapping**: Event bus pattern for decoupled game/UI communication.

---

## Category 3: Bevy ECS References

### 11. Bevy ECS Basics
- **URL**: https://docs.rs/bevy/latest/bevy/ecs/
- **Key Insight**: Bevy ECS: Entities, Components, Systems, Resources, Events. Query-based data access with `Query<T>` and `Res<T>`.
- **NeoTrix Mapping**: Core architecture reference for NeoTrix sim entity/component design.

### 12. Bevy 2D Rendering
- **URL**: https://docs.rs/bevy/latest/bevy/prelude/struct.SpriteBundle.html
- **Key Insight**: SpriteBundle for 2D rendering. Camera2dBundle for 2D cameras. Transform for positioning.
- **NeoTrix Mapping**: Reference for tilemap and entity visual representation.

### 13. Bevy UI
- **URL**: https://docs.rs/bevy/latest/bevy/ui/
- **Key Insight**: Bevy UI system with NodeBundle, TextBundle, ImageBundle. Layout via Style with Flexbox.
- **NeoTrix Mapping**: In-game HUD and UI elements rendered natively.

### 14. Bevy FpsOverlayPlugin
- **URL**: https://docs.rs/bevy/latest/bevy/diagnostics/struct.FpsOverlayPlugin.html
- **Key Insight**: Built-in FPS overlay plugin for development. Configurable text style and position.
- **NeoTrix Mapping**: Dev tools for performance monitoring during sim development.

### 15. Bevy State Management
- **URL**: https://bevyengine.org/learn/book/getting-started/state/
- **Key Insight**: States system for game flow (Menu, InGame, Paused). Systems run conditionally via `in_state`.
- **NeoTrix Mapping**: Game state machine for NeoTrix sim (exploration, editing, simulation modes).

---

## Category 4: Open Source Game Repos

### 16. Colony (Rust)
- **URL**: https://github.com/sempervent/colony
- **Key Insight**: Rust colony sim using Bevy ECS + Tokio async runtime. Modular crates: colony-core, colony-io, colony-sim, colony-desktop, colony-headless.
- **NeoTrix Mapping**: Direct reference for modular Rust game architecture. Colony sim patterns for agent-based systems.

### 17. OpenCiv
- **URL**: https://github.com/RyanGrieb/OpenCiv
- **Key Insight**: Open source civilization-like game in browser (199 stars). TypeScript + Canvas2D. Turn-based strategy with hex tiles.
- **NeoTrix Mapping**: Reference for hex-based tilemap implementation and strategy game mechanics.

### 18. Handmade Heroes (Rust)
- **URL**: https://github.com/ajmadsen/handmade-hero-rust
- **Key Insight**: Rust port of Handmade Hero. Low-level game programming patterns, entity management, collision detection.
- **NeoTrix Mapping**: Reference for low-level game mechanics and entity behavior.

### 19. Dinky Kingdom
- **URL**: https://github.com/colinjakel/dinky-kingdom
- **Key Insight**: Life sim game in Rust. Entity management, pathfinding, need-based AI (hunger, social, etc.).
- **NeoTrix Mapping**: Reference for agent needs system and behavioral AI.

### 20. Triple-A
- **URL**: https://github.com/thirty-three-n/triple-a
- **Key Insight**: Rust game framework with ECS-like architecture. Entity-component pattern, event-driven design.
- **NeoTrix Mapping**: Alternative ECS pattern reference.

---

## Category 5: Agent-Based Models & Simulation

### 21. Oxyde (Rust AI Agent SDK)
- **URL**: https://github.com/oxyde-labs/oxyde
- **Key Insight**: Rust AI Agent SDK for goal-driven game NPCs with emergent storytelling. Agents have needs, goals, memory, and make autonomous decisions.
- **NeoTrix Mapping**: Direct reference for NeoTrix agent behavior system. Goal-driven architecture with emergent patterns.

### 22. Satisfactory Colony Sim
- **URL**: https://github.com/PrismarineJS/mineflayer-colony-sim
- **Key Insight**: Colony simulation using pathfinding, resource management, task assignment. Agents have roles and priorities.
- **NeoTrix Mapping**: Reference for task assignment and resource management patterns.

### 23. Silicon Dreams
- **URL**: https://github.com/rorymbyrne/silicon-dreams
- **Key Insight**: Agent-based modeling framework. Entities with state machines, communication protocols, and environmental interaction.
- **NeoTrix Mapping**: Reference for agent communication and state machine patterns.

---

## Category 6: Real-Time WebSocket Patterns

### 24. Tauri WebSocket Chat
- **URL**: https://github.com/nicegamer7/tauri-plugin-websocket
- **Key Insight**: Real-time bidirectional WebSocket communication for Tauri apps. Message types: Text, Binary, Ping, Pong.
- **NeoTrix Mapping**: WebSocket protocol for streaming game state from Bevy to Tauri UI.

### 25. WebSocket State Sync
- **URL**: https://github.com/nicegamer7/tauri-plugin-websocket
- **Key Insight**: WebSocket state synchronization pattern. Server sends state updates, client renders.
- **NeoTrix Mapping**: State synchronization architecture for NeoTrix sim.

---

## Category 7: Camera & Interaction

### 26. Bevy Camera Pan/Zoom
- **URL**: https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html
- **Key Insight**: Camera movement via Transform. Zoom via Camera2d.scale. Input handling via `Res<ButtonInput<KeyCode>>`.
- **NeoTrix Mapping**: Camera controls for NeoTrix sim viewport.

### 27. Entity Selection in Bevy
- **URL**: https://docs.rs/bevy/latest/bevy/picking/
- **Key Insight**: Bevy Picking system for entity selection. Ray casting from cursor to entities. Selection events via `PointerClick`.
- **NeoTrix Mapping**: Entity selection system for NeoTrix sim.

### 28. Path Visualization
- **URL**: https://docs.rs/bevy/latest/bevy/render/
- **Key Insight**: Bevy Render pipeline for custom drawing. Gizmos for debug visualization. Line rendering for paths.
- **NeoTrix Mapping**: Path visualization for agent movement and connections.

---

## Category 8: Knowledge Graph & Evolution Display

### 29. Knowledge Graph Visualization
- **URL**: https://github.com/nicegamer7/tauri-plugin-websocket
- **Key Insight**: Real-time graph updates via WebSocket. D3.js force-directed layout for knowledge graphs.
- **NeoTrix Mapping**: Knowledge graph display in NeoTrix sim UI.

### 30. Evolution Chart Display
- **URL**: https://github.com/nicegamer7/tauri-plugin-websocket
- **Key Insight**: Real-time chart updates via WebSocket. Canvas-based rendering for performance.
- **NeoTrix Mapping**: Evolution visualization in NeoTrix sim UI.

---

## Synthesis

### Key Patterns for NeoTrix Sim

1. **Architecture**: Tauri (OS integration) + Bevy (ECS/game logic) with WebSocket bridge
2. **State Management**: Bevy ECS for game state, Tauri commands for UI queries, WebSocket for streaming updates
3. **Agent System**: Goal-driven agents with needs, memory, and autonomous decisions (Oxyde pattern)
4. **Modular Crates**: colony-core, colony-io, colony-sim pattern for separation of concerns
5. **Camera/Interaction**: Bevy Picking for selection, Transform for camera pan/zoom
6. **Knowledge Graph**: WebSocket-driven real-time updates, Canvas2D for rendering

### Recommended Stack

- **Desktop**: Tauri v2 (shell) + Bevy (ECS/rendering)
- **Communication**: WebSocket plugin for state streaming
- **Agent AI**: Goal-driven with needs system (Oxyde-inspired)
- **Visualization**: Bevy 2D rendering + HTML overlay for complex UI
