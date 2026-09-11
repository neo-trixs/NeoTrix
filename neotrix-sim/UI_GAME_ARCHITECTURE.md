# NT-WORLD-SIM — Frontend Game Architecture

**Date**: 2026-09-11
**Status**: Design (ready for implementation)
**Depends on**: `ECS_PLAN.md` (Bevy ECS migration, Phase 1-3 minimum)

---

## 1. Tech Stack Recommendation

### Option Evaluation

| Option | Stack | Pros | Cons | Verdict |
|--------|-------|------|------|---------|
| **A** | Tauri + Bevy (Rust fullstack) | Unified Rust, shared types, zero serialization, ECS-native rendering | High complexity, two app loops to coordinate, Bevy rendering in Tauri webview requires workarounds | ⚠️ Overkill |
| **B** | Tauri + HTML5 Canvas/WebGL | Simple integration, hot-reloadable UI, existing ecosystem, Tauri IPC is mature | No ECS-native rendering, manual sprite management, 50+ agents need optimization | ✅ **Best** |
| **C** | Bevy standalone (pure game) | Full ECS rendering, zero bridge overhead | No desktop shell (menus, settings, file I/O), no Tauri IPC for consciousness core, harder to integrate with NeoTrix ecosystem | ❌ No desktop |
| **D** | Tauri + Leptos (Rust web) | Full Rust stack, SSR capable | Leptos still experimental, no game rendering primitives, overkill for visualization | ❌ Too early |

### Recommended: Option B — Tauri + HTML5 Canvas/WebGL

**Justification**:

1. **ECS plan already targets `bevy_ecs` standalone** (no rendering). The sim backend is pure computation. Rendering is a separate concern — perfect fit for a web frontend.
2. **Tauri IPC is mature** — `invoke()` / `emit()` maps directly to the existing `HostBridge` pattern (`SimReport` → frontend, `CoreDirective` → backend).
3. **Performance is sufficient** — HTML5 Canvas handles 100+ sprites at 60fps. For 50 agents + 200 resource nodes + terrain tiles, Canvas2D or a lightweight WebGL wrapper (e.g., PixiJS) is more than enough.
4. **Hot-reloadable UI** — Frontend changes don't require Rust recompilation. Iteration speed matters for game UI.
5. **Existing pattern** — The `bridge/host_bridge.rs` already defines `SimReport` (backend→frontend) and `CoreAdjustment` (frontend→backend). Tauri commands mirror this exactly.

### Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Backend | `neotrix-sim` (Rust) | Simulation engine, ECS systems, consciousness core |
| Desktop Shell | Tauri v2 | Window management, file I/O, system tray, menus |
| Frontend | TypeScript + Vite | UI framework, component rendering |
| Rendering | HTML5 Canvas 2D (Phase 1) → PixiJS/WebGL (Phase 2) | Game world visualization |
| State | Tauri events (`emit`/`listen`) | Real-time backend→frontend streaming |
| Commands | Tauri `invoke()` | Frontend→backend control (pause, step, inject) |

---

## 2. Game World Visualization

### 2.1 View Mode: Top-Down (Phase 1), Isometric (Phase 2)

Top-down is simpler to implement and matches the coordinate system (`position.x`, `position.y`). Isometric adds visual depth but requires coordinate transforms. Start top-down, add isometric as an enhancement.

### 2.2 Terrain Rendering

The backend already has `Heightmap` (128×128) and `BiomeMap`. The frontend renders this as a tile grid:

```
┌─────────────────────────────────────────────┐
│  Terrain Tile Rendering                      │
│                                              │
│  Heightmap → tile color (green→brown→white)  │
│  BiomeMap  → tile overlay (forest/sand/water) │
│  Combined  → final tile color                │
│                                              │
│  Tile size: 8px × 8px (128 tiles = 1024px)  │
│  Zoom levels: 0.5x, 1x, 2x, 4x             │
└─────────────────────────────────────────────┘
```

**Tile color mapping**:
- Water: `#3B82F6` (blue)
- Sand/Desert: `#F59E0B` (amber)
- Grass/Plain: `#22C55E` (green)
- Forest: `#166534` (dark green)
- Mountain: `#78716C` (stone)
- Snow: `#F5F5F4` (white)

### 2.3 Agent Sprites

Each agent is a colored circle with status bars:

```
  ┌─── Health bar (red→green)
  │ ┌─ Energy bar (yellow→blue)
  │ │
  ●──●  ← Agent circle (color = dominant trait)
  │
  └── ID label ("A-0")
```

**Color by dominant trait**:
- High sociability: `#8B5CF6` (violet)
- High curiosity: `#06B6D4` (cyan)
- High aggression: `#EF4444` (red)
- High cooperativeness: `#10B981` (emerald)
- Balanced: `#6B7280` (gray)

**Selection ring**: When agent is selected, draw a pulsing yellow ring around it.

### 2.4 Resource Nodes

Small diamond shapes with type-based coloring:

| Resource | Color | Icon |
|----------|-------|------|
| Food/Berries | `#F97316` (orange) | 🍊 |
| Wood | `#92400E` (brown) | 🪵 |
| Stone/Ore | `#6B7280` (gray) | ⛏️ |
| Water | `#3B82F6` (blue) | 💧 |
| Fish | `#06B6D4` (cyan) | 🐟 |

Depleted nodes rendered as faded/hollow diamonds.

### 2.5 Structures

Larger rectangular shapes with type labels:

| Structure | Shape | Color |
|-----------|-------|-------|
| Shelter | □ | `#A16207` (amber) |
| Farm | ▭ | `#65A30D` (lime) |
| Workshop | ▧ | `#7C3AED` (violet) |
| Watchtower | △ | `#DC2626` (red) |
| Market | ⬒ | `#2563EB` (blue) |
| Wall | █ | `#57534E` (stone) |
| Road | ═ | `#A8A29E` (sand) |

### 2.6 Pheromone Trails (Overlay)

Semi-transparent colored overlay on the terrain grid. Each pheromone type has a distinct hue:

| Pheromone | Color | Alpha |
|-----------|-------|-------|
| Food | Orange | 0.0–0.3 (strength-mapped) |
| Danger | Red | 0.0–0.4 |
| Social | Purple | 0.0–0.2 |
| Rest | Blue | 0.0–0.2 |
| Explore | Cyan | 0.0–0.15 |
| Territory | Green | 0.0–0.25 |

Render as additive-blended rectangles on top of terrain. Toggle visibility via UI checkbox.

### 2.7 Relationship Lines

When "Social Overlay" is enabled, draw lines between agents with relationship > 0:

- Line color: gradient from red (negative) → white (neutral) → green (positive)
- Line width: 1–3px based on relationship strength
- Only show for relationships above a threshold (e.g., > 0.2)

---

## 3. UI Component Layout

### 3.1 Overall Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  NT-WORLD-SIM                                    [⚙] [?] [─][□][×]│
├──────────────────────────────────────────────────────────────────┤
│  [▶ Play] [⏸ Pause] [⏭ Step] [🔄 Reset]  Speed: [1x ▾]  Tick: 1247  │
├──────────────────────────────────────┬───────────────────────────┤
│                                      │                           │
│                                      │   Agent Inspector         │
│          Game World                  │   ┌─────────────────┐    │
│          (Canvas)                    │   │ A-3              │    │
│                                      │   │ HP: ████████░░ 80│    │
│                                      │   │ EN: ██████░░░░ 60│    │
│                                      │   │ Position: (42,71)│    │
│                                      │   │ Trait: curious   │    │
│                                      │   │ Phi: 0.42        │    │
│                                      │   │ Action: Explore  │    │
│                                      │   └─────────────────┘    │
│                                      │                           │
│                                      │   Evolution Tracker       │
│                                      │   ┌─────────────────┐    │
│                                      │   │ Gen: 12          │    │
│                                      │   │ Pop: 23/50       │    │
│                                      │   │ Species: 4       │    │
│                                      │   │ Phi: ▁▃▅▇▆▅▃   │    │
│                                      │   └─────────────────┘    │
├──────────────────────────────────────┴───────────────────────────┤
│  [Minimap]  Events: Agent A-7 ate berries  │ Phi: 0.38  │ Coherence: 0.61 │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 Panel Descriptions

#### Control Bar (Top)

| Control | Type | Action |
|---------|------|--------|
| Play/Pause | Toggle button | Start/stop simulation loop |
| Step | Button | Advance one tick |
| Reset | Button | Reinitialize simulation |
| Speed selector | Dropdown | 0.5x, 1x, 2x, 5x, 10x, Max |
| Tick counter | Label | Current tick number |
| Time display | Label | In-sim time (day/night, season) |

#### Game World (Center)

- **Pan**: Click + drag (middle mouse or space+drag)
- **Zoom**: Mouse wheel (0.25x – 8x)
- **Select agent**: Left click on agent sprite
- **Select resource**: Left click on resource node
- **Context menu**: Right click for agent actions (when in "consciousness core" mode)

#### Agent Inspector (Right Panel)

Displays selected agent's full state:

```
Agent A-3
─────────
Core Stats:
  Health:  ████████░░ 80/100
  Energy:  ██████░░░░ 60/100
  Hunger:  ███░░░░░░░ 30/100
  Age:     247 ticks

Personality:
  Openness:       0.72 ████████░░
  Sociability:    0.45 █████░░░░░
  Curiosity:      0.81 █████████░
  Cooperativeness:0.63 ██████░░░░
  Aggression:     0.12 █░░░░░░░░░

Consciousness:
  Phi:            0.42
  Level:          Integration
  
Memory:
  Stream: 47 nodes (recent 20 shown)
  Graph:  156 nodes, 203 edges
  Spatial: 23 visited positions

Recent Actions:
  [Tick 1245] Explore (0.3, -0.7)
  [Tick 1240] Eat from R-12
  [Tick 1235] Talk to A-7

Theory of Mind:
  A-7: threat=0.1 coop=0.8
  A-2: threat=0.4 coop=0.3
```

#### Evolution Tracker (Right Panel, below Inspector)

```
Evolution History
─────────────────
Generation: 12
Population:  23 alive / 50 max
Species:     4 clusters

Fitness Trend (last 10 gens):
  Mean: ▁▃▅▇▆▅▃▁▂▃
  Max:  ▃▅▇▇▇▇▆▅▅▆

Phi Trend (last 100 ticks):
  ▁▂▃▄▅▆▇▆▅▄▃▂▃▄▅▆▇▇▆

Species Breakdown:
  ● Curious Explorers (8)
  ● Social Builders (6)
  ● Aggressive Hunters (5)
  ● Balanced Survivors (4)

Convergence State: Exploring
Selection Pressure: 0.45
Mutation Rate: 0.08
```

#### Memory Viewer (Right Panel, toggleable)

Graph visualization of selected agent's `GraphMemory`:

- Nodes: colored by `NodeKind` (Event=blue, Person=green, Place=orange)
- Edges: colored by `EdgeKind` (Temporal=gray, Social=purple, Spatial=cyan)
- Layout: force-directed (simple spring model, updated every 100ms)
- Interaction: click node to see details, scroll to zoom

#### Minimap (Bottom-left corner)

- 200×200px overlay showing entire world
- Agent positions as colored dots
- Resource nodes as tiny diamonds
- Camera viewport rectangle
- Click to jump to position

#### Event Log (Bottom bar)

Scrolling list of recent `SimEvent`s:

```
[1247] Agent A-7 ate from R-12
[1245] Agent A-3 explored north
[1243] Agent A-1 talked to A-5
[1240] Pheromone deposited: Food at (42, 71)
[1238] Economy: A-2 traded Food→Wood with A-9
```

---

## 4. Frontend-Backend Communication

### 4.1 Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Desktop App                      │
│                                                          │
│  ┌──────────────┐    Tauri IPC     ┌──────────────────┐ │
│  │   Frontend    │ ←──────────────→ │    Backend        │ │
│  │   (TypeScript)│   invoke()       │   (Rust)          │ │
│  │               │   emit()/listen() │                   │ │
│  │  ┌─────────┐ │                  │  ┌──────────────┐ │ │
│  │  │ Canvas  │ │   SimReport      │  │  WorldSim    │ │ │
│  │  │ Renderer│ │ ←─────────────── │  │  (ECS World) │ │ │
│  │  └─────────┘ │                  │  └──────────────┘ │ │
│  │  ┌─────────┐ │   CoreAdjustment │  ┌──────────────┐ │ │
│  │  │ UI      │ │ ───────────────→ │  │  HostBridge  │ │ │
│  │  │ Panels  │ │                  │  └──────────────┘ │ │
│  │  └─────────┘ │                  │  ┌──────────────┐ │ │
│  │               │                  │  │ Consciousness│ │ │
│  │               │                  │  │ Core Agent   │ │ │
│  └──────────────┘                  │  └──────────────┘ │ │
│                                     └──────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Data Flow

#### Backend → Frontend (Event Stream)

```rust
// In neotrix-sim backend, after each tick:
app.emit("sim-tick", SimReport { ... })?;

// Frontend listens:
listen("sim-tick", |event| {
    let report: SimReport = event.payload();
    update_renderer(report);
});
```

**Update frequency**: Every tick (fast tier = every 5 ticks for agent states). Full snapshot every tick, delta updates for agent positions between full snapshots.

#### Frontend → Backend (Tauri Commands)

```typescript
// Control commands
invoke("sim_play");
invoke("sim_pause");
invoke("sim_step");
invoke("sim_reset", { config: WorldSimConfig });

// Agent inspection
invoke("get_agent_state", { agentId: "A-3" }): AgentState
invoke("get_agent_memory", { agentId: "A-3" }): AgentMemory

// Consciousness core
invoke("select_core_agent", { agentId: "A-3" });
invoke("inject_action", { agentId: "A-3", action: AgentAction });
invoke("set_mode", { mode: "train" | "observe" | "control" });

// World queries
invoke("get_heightmap"): HeightmapData
invoke("get_biome_map"): BiomeMapData
invoke("get_resource_nodes"): ResourceNode[]
invoke("get_pheromone_field", { layer: "food" }): PheromoneData
```

### 4.3 State Synchronization Protocol

```rust
// Backend emits these events:
enum SimEvent {
    // Tick-level updates (every tick)
    TickUpdate {
        tick: u64,
        agents: Vec<AgentSnapshot>,      // positions, health, energy
        resources: Vec<ResourceSnapshot>, // amounts, depleted status
        pheromones: Option<PheromoneSnapshot>, // if overlay enabled
        events: Vec<SimEvent>,           // notable events this tick
    },

    // Full state (on demand or every N ticks)
    FullSnapshot {
        tick: u64,
        world: WorldSnapshot,
        agents: Vec<AgentFullState>,
        terrain: TerrainData,
    },

    // Agent-specific (when selected)
    AgentDetail {
        agent_id: String,
        state: AgentFullState,
        memory: AgentMemorySnapshot,
        relationships: Vec<Relationship>,
    },

    // Evolution (every evolution cycle)
    EvolutionUpdate {
        generation: u64,
        population: usize,
        species: Vec<SpeciesInfo>,
        fitness_trend: Vec<f64>,
        phi_trend: Vec<f64>,
    },

    // Consciousness core events
    CoreActionInjected {
        agent_id: String,
        action: AgentAction,
        tick: u64,
    },
}
```

### 4.4 Performance Considerations

| Strategy | Implementation |
|----------|---------------|
| **Delta compression** | Only send changed agent positions between full snapshots |
| **LOD (Level of Detail)** | Far agents: just position dot. Near agents: full sprite + bars |
| **Pheromone LOD** | Only compute pheromone overlay for visible tiles (viewport culling) |
| **Batch updates** | Aggregate all agent states into one event per tick (no per-agent events) |
| **Frontend throttle** | Render at 30fps, accept events at tick rate (may be higher) |

---

## 5. Consciousness Core Integration

### 5.1 Concept

The "consciousness core" is a single agent that the user (or the NeoTrix consciousness system) directly controls or observes. It explores the simulation world, makes decisions, and its behavior is studied/optimized.

### 5.2 Agent Selection

```typescript
// User clicks agent → selects as consciousness core
invoke("select_core_agent", { agentId: "A-3" });

// Backend marks agent:
// - Adds `ConsciousnessCore` marker component
// - Enables direct action injection from frontend
// - Logs all decisions for analysis
```

### 5.3 Camera Follow Mode

When a consciousness core agent is selected:

```typescript
// Frontend renderer
class Camera {
    followAgent(agentId: string) {
        this.followTarget = agentId;
        this.mode = "follow";
    }

    update(agentPositions: Map<string, Vec2>) {
        if (this.mode === "follow" && this.followTarget) {
            const pos = agentPositions.get(this.followTarget);
            if (pos) {
                this.centerOn(pos.x, pos.y);
            }
        }
    }
}
```

### 5.4 Three Operating Modes

| Mode | Control | Use Case |
|------|---------|----------|
| **Observe** | No user input, watch autonomous behavior | Study natural agent evolution |
| **Train** | User suggests actions, agent decides | Reinforcement learning, human-in-the-loop |
| **Control** | User directly executes actions | Debugging, exploration, consciousness experiments |

### 5.5 Mode: Observe (Default)

```
User watches agent A-3 explore the world.
- Camera follows A-3
- Inspector shows real-time stats
- Memory viewer shows graph growing
- No user input affects the agent
```

### 5.6 Mode: Train

```
User can suggest actions via UI buttons:
  [Eat] [Rest] [Explore] [Talk] [Trade] [Build] [Attack]

Clicking "Explore" → injects `CoreAdjustment::OverrideAction` with Explore.
Agent's own decision layer may override if survival needs dominate.
Backend logs: "User suggested Explore, agent chose Eat (hunger=85)"
```

### 5.7 Mode: Control

```
User has full control of the agent.
- All action buttons are enabled
- Action executes immediately (bypasses decision layers)
- Backend logs all user-injected actions
- Useful for: testing specific scenarios, consciousness exploration,
  "what if" experiments
```

### 5.8 Training Mode (Auto-Play)

A special mode where the consciousness core runs autonomously but with logging:

```rust
// Backend tracks:
struct TrainingSession {
    agent_id: String,
    start_tick: u64,
    actions: Vec<(u64, AgentAction, f64)>, // (tick, action, outcome_score)
    phi_history: Vec<f64>,
    reward_curve: Vec<f64>,
}

// Frontend shows:
// - Live action feed
// - Phi progression chart
// - Reward curve
// - "Export Session" button for analysis
```

### 5.9 Consciousness Core ↔ Host Bridge

The existing `HostBridge` in `bridge/host_bridge.rs` already supports this pattern:

```
User (Frontend) → CoreAdjustment → HostBridge → WorldSim.tick()
WorldSim.tick() → SimReport → HostBridge → User (Frontend)
```

The consciousness core agent gets special treatment:
- `CoreDirective` from user → agent's decision layer uses it as bonus weight
- `OverrideAction` from user → agent executes directly (bypasses layers)
- Agent's `SimReport` includes detailed memory/state for frontend visualization

---

## 6. File Structure

```
neotrix-sim/
├── Cargo.toml                    # Add tauri dependency
├── src-tauri/                    # NEW: Tauri backend
│   ├── Cargo.toml                # tauri, serde, neotrix-sim
│   ├── tauri.conf.json           # Tauri config
│   ├── src/
│   │   ├── main.rs               # Tauri app setup, commands
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── sim_runner.rs         # Simulation loop (async, emits events)
│   │   ├── state.rs              # Shared sim state (Arc<Mutex<WorldSim>>)
│   │   └── consciousness.rs      # Consciousness core agent management
│   └── icons/                    # App icons
│
├── src/                          # Existing sim backend (unchanged)
│   ├── world_sim.rs              # WorldSim (will be wrapped by sim_runner)
│   ├── bridge/
│   │   ├── host_bridge.rs        # SimReport, CoreDirective (existing)
│   │   └── trait_defs.rs         # Bridge traits (existing)
│   └── ...
│
└── ui/                           # NEW: Frontend
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    ├── src/
    │   ├── main.ts               # Entry point, Tauri event listeners
    │   ├── app.ts                # App state, mode management
    │   ├── renderer/
    │   │   ├── canvas.ts         # Canvas setup, pan/zoom
    │   │   ├── terrain.ts        # Heightmap/biome tile rendering
    │   │   ├── agents.ts         # Agent sprite rendering
    │   │   ├── resources.ts      # Resource node rendering
    │   │   ├── structures.ts     # Structure rendering
    │   │   ├── pheromones.ts     # Pheromone overlay rendering
    │   │   ├── relationships.ts  # Social connection lines
    │   │   ├── selection.ts      # Agent selection, highlight
    │   │   └── camera.ts         # Camera pan/zoom/follow
    │   ├── ui/
    │   │   ├── control-bar.ts    # Play/pause/step/speed controls
    │   │   ├── agent-inspector.ts # Agent detail panel
    │   │   ├── evolution-tracker.ts # Evolution stats panel
    │   │   ├── memory-viewer.ts  # Graph memory visualization
    │   │   ├── minimap.ts        # World minimap overlay
    │   │   ├── event-log.ts      # Event log panel
    │   │   └── consciousness.ts  # Core agent mode controls
    │   ├── types/
    │   │   ├── sim.ts            # TypeScript types (mirrors Rust structs)
    │   │   └── events.ts         # Event type definitions
    │   └── utils/
    │       ├── color.ts          # Color palettes, trait→color mapping
    │       └── math.ts           # Vec2, coordinate transforms
    └── public/
        └── fonts/
```

### 6.1 Key Files Explained

| File | Purpose |
|------|---------|
| `src-tauri/src/main.rs` | Tauri app entry, registers commands, sets up event emission |
| `src-tauri/src/commands.rs` | All `#[tauri::command]` functions (sim control, agent queries) |
| `src-tauri/src/sim_runner.rs` | Async loop that calls `WorldSim::tick()` and emits `SimReport` via Tauri events |
| `src-tauri/src/state.rs` | `Arc<Mutex<WorldSim>>` shared between command handlers and sim runner |
| `src-tauri/src/consciousness.rs` | Consciousness core agent selection, mode management, action injection |
| `ui/src/renderer/canvas.ts` | Main Canvas2D setup, render loop, input handling |
| `ui/src/renderer/terrain.ts` | Pre-renders heightmap/biome to an offscreen canvas (called once at init) |
| `ui/src/renderer/agents.ts` | Renders agent sprites with status bars, selection ring |
| `ui/src/ui/agent-inspector.ts` | Detailed agent state panel (personality, memory, relationships) |
| `ui/src/ui/memory-viewer.ts` | Force-directed graph layout of GraphMemory |
| `ui/src/types/sim.ts` | TypeScript interfaces matching Rust `Serialize` structs |

---

## 7. Implementation Phases

### Phase 1: Tauri Shell + Terrain Rendering (3–4 days)

**Goal**: Window opens, terrain renders, basic controls work.

**Tasks**:
1. Initialize Tauri v2 project in `src-tauri/`
2. Create `sim_runner.rs` — async loop calling `WorldSim::tick()`, emitting events
3. Create `commands.rs` — `sim_play`, `sim_pause`, `sim_step`, `sim_reset`
4. Create frontend Vite project in `ui/`
5. Implement Canvas setup with pan/zoom in `canvas.ts`
6. Implement terrain tile rendering from `Heightmap` + `BiomeMap` in `terrain.ts`
7. Wire up control bar (play/pause/step)

**Deliverable**: Desktop window showing the terrain, can play/pause simulation.

### Phase 2: Agent + Resource Rendering (2–3 days)

**Goal**: Agents and resources visible on the terrain.

**Tasks**:
1. Render agent sprites (colored circles + health/energy bars) in `agents.ts`
2. Render resource nodes (diamond shapes) in `resources.ts`
3. Implement agent selection (click to select) in `selection.ts`
4. Implement Agent Inspector panel in `agent-inspector.ts`
5. Wire up `TickUpdate` event to update positions every tick

**Deliverable**: Agents move on terrain, resources visible, can select and inspect agents.

### Phase 3: UI Panels + Pheromones (3–4 days)

**Goal**: Full UI with all panels and overlays.

**Tasks**1. Implement Evolution Tracker panel in `evolution-tracker.ts`
2. Implement Event Log in `event-log.ts`
3. Implement Minimap in `minimap.ts`
4. Implement pheromone overlay rendering in `pheromones.ts`
5. Implement relationship lines in `relationships.ts`
6. Add toggle controls for overlays (pheromones, relationships)

**Deliverable**: Complete UI with all panels, overlays toggleable.

### Phase 4: Consciousness Core (3–4 days)

**Goal**: User can select and control a consciousness core agent.

**Tasks**1. Implement `consciousness.rs` in Tauri backend (agent selection, mode management)
2. Implement consciousness mode controls in `ui/src/ui/consciousness.ts`
3. Implement camera follow mode in `camera.ts`
4. Implement action injection (user clicks action button → backend executes)
5. Implement training mode logging and visualization
6. Implement Memory Viewer (graph visualization) in `memory-viewer.ts`

**Deliverable**: User can select an agent as consciousness core, switch modes, inject actions, view memory graph.

### Phase 5: Polish + Performance (2–3 days)

**Goal**: Production-ready, performant, visually polished.

**Tasks**1. Implement delta compression (only send changed positions)
2. Implement LOD (far agents = dots, near = full sprites)
3. Add smooth interpolation for agent movement (tween between ticks)
4. Add isometric view mode (optional, coordinate transform)
5. Add tooltips for agents/resources
6. Add keyboard shortcuts (space=pause, arrow keys=step)
7. Performance profiling and optimization

**Deliverable**: Smooth 60fps rendering with 50+ agents, polished UI.

---

## 8. Estimated Effort

| Phase | Days | LOC (est.) | Dependencies |
|-------|------|-----------|-------------|
| Phase 1: Tauri Shell + Terrain | 3–4 | ~800 | ECS Plan Phase 1-2 (basic Bevy World) |
| Phase 2: Agents + Resources | 2–3 | ~600 | Phase 1 |
| Phase 3: UI Panels + Overlays | 3–4 | ~900 | Phase 2 |
| Phase 4: Consciousness Core | 3–4 | ~700 | Phase 3 |
| Phase 5: Polish + Performance | 2–3 | ~500 | Phase 4 |
| **Total** | **13–18** | **~3500** | ECS Plan Phase 1-3 |

### Prerequisites

The frontend can start development in parallel with ECS migration. Phase 1 of the UI (Tauri shell + terrain) only needs:
- `WorldSim::new()` working (already works)
- `WorldSnapshot` serialization (already works)
- `Heightmap` + `BiomeMap` accessible (already works)

Agent rendering (Phase 2) needs agent data accessible, which works with the current `Vec<SimAgent>`.

Full ECS integration (parallel agent processing) is only needed for Phase 5 performance optimization.

---

## 9. Dependency Changes

### Cargo.toml additions

```toml
[dependencies]
# Existing
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
rand = "0.8"

# NEW: Tauri
tauri = { version = "2", features = ["shell-open"] }
tauri-plugin-shell = "2"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

### Frontend package.json

```json
{
  "name": "neotrix-sim-ui",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vite": "^6.0.0"
  }
}
```

---

## 10. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Bevy ECS + Tauri event loop conflict** | Medium | Run sim in a separate thread, emit events via Tauri's async event system. Don't run Bevy App loop — use `bevy_ecs::World` directly. |
| **Canvas performance with 50+ agents** | Low | Canvas2D handles 200+ simple sprites at 60fps easily. Profile early. |
| **Type mismatch between Rust and TypeScript** | Medium | Derive `Serialize`/`Deserialize` on all shared types. Generate TypeScript types from Rust structs using `ts-rs` or `serde_ts`. |
| **Tauri v2 breaking changes** | Low | Pin to specific version. Tauri v2 is stable. |
| **Memory viewer complexity** | Medium | Force-directed layout can be expensive. Use a simple grid layout as fallback. Cap at 200 nodes visible. |
| **Pheromone overlay performance** | Medium | Only render visible tiles. Use offscreen canvas for pheromone layer, composite with `globalAlpha`. |

---

## Appendix A: Tauri Command Signatures

```rust
// src-tauri/src/commands.rs

#[tauri::command]
fn sim_play(state: State<'_, SimState>) -> Result<(), String>;

#[tauri::command]
fn sim_pause(state: State<'_, SimState>) -> Result<(), String>;

#[tauri::command]
fn sim_step(state: State<'_, SimState>) -> Result<WorldSnapshot, String>;

#[tauri::command]
fn sim_reset(state: State<'_, SimState>, config: WorldSimConfig) -> Result<(), String>;

#[tauri::command]
fn get_agent_state(state: State<'_, SimState>, agent_id: String) -> Result<AgentFullState, String>;

#[tauri::command]
fn get_agent_memory(state: State<'_, SimState>, agent_id: String) -> Result<AgentMemorySnapshot, String>;

#[tauri::command]
fn select_core_agent(state: State<'_, SimState>, agent_id: String) -> Result<(), String>;

#[tauri::command]
fn set_core_mode(state: State<'_, SimState>, mode: CoreMode) -> Result<(), String>;

#[tauri::command]
fn inject_action(state: State<'_, SimState>, agent_id: String, action: AgentAction) -> Result<(), String>;

#[tauri::command]
fn get_heightmap(state: State<'_, SimState>) -> Result<HeightmapData, String>;

#[tauri::command]
fn get_resource_nodes(state: State<'_, SimState>) -> Result<Vec<ResourceSnapshot>, String>;

#[tauri::command]
fn get_pheromone_field(state: State<'_, SimState>, layer: String) -> Result<PheromoneData, String>;
```

## Appendix B: TypeScript Type Definitions (Partial)

```typescript
// ui/src/types/sim.ts

interface WorldSnapshot {
    tick: number;
    time: string;
    population: number;
    mean_phi: number;
    mean_coherence: number;
    species_count: number;
    evolution_generations: number;
    resources_total: number;
    resources_depleted: number;
    total_relationships: number;
    total_trades: number;
    emotion_dominant: string;
    emotion_valence: number;
    emotion_arousal: number;
    safety_alerts: number;
}

interface AgentSnapshot {
    id: string;
    position: [number, number];
    health: number;
    energy: number;
    hunger: number;
    alive: boolean;
    dominant_trait: string;
    last_action: string;
}

interface AgentFullState {
    id: string;
    position: [number, number];
    health: number;
    energy: number;
    hunger: number;
    age: number;
    alive: boolean;
    personality: Personality;
    phi: number;
    recent_actions: string[];
}

interface Personality {
    openness: number;
    sociability: number;
    curiosity: number;
    cooperativeness: number;
    aggression: number;
}

type CoreMode = "observe" | "train" | "control";

type AgentAction =
    | { Move: { target: [number, number] } }
    | { Eat: { resource_id: string } }
    | { Rest: null }
    | { Talk: { target_id: string; message: string } }
    | { Explore: { direction: [number, number] } }
    | { Trade: { target_id: string; item: string; amount: number } }
    | { Build: { position: [number, number]; structure_type: string } }
    | { Attack: { target_id: string } }
    | { Harvest: { resource_id: string } }
    | { Think: null };
```
