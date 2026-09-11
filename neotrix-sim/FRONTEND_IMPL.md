# NT-WORLD-SIM Frontend Implementation

## Files Created

| File | Purpose |
|------|---------|
| `nt-world-sim/dist/index.html` | Game UI shell — canvas, sidebar panels, controls |
| `nt-world-sim/dist/main.js` | Game loop — rendering, camera, agent selection, Tauri IPC |

## Architecture

```
┌─────────────────────────────────────────────┐
│  index.html                                │
│  ┌──────────────┐ ┌──────────────────────┐  │
│  │  #canvas-wrap │ │  #sidebar            │  │
│  │  (world view) │ │  ├─ Controls         │  │
│  │  <canvas>     │ │  ├─ World State      │  │
│  └──────────────┘ │  ├─ Selected Agent    │  │
│                    │  ├─ Minimap           │  │
│                    │  └─ Event Log         │  │
│                    └──────────────────────┘  │
└─────────────────────────────────────────────┘
         │  Tauri IPC (invoke)
         ▼
┌─────────────────────────┐
│  Backend Commands       │
│  sim_get_state          │
│  sim_get_agents         │
│  sim_get_world_map      │
│  sim_select_agent       │
│  sim_tick               │
└─────────────────────────┘
```

## Features

- **Terrain rendering**: 128×128 tile grid with 13 biome colors based on heightmap
- **Agent rendering**: Circle markers with health bars; selected agent highlighted in red
- **Camera**: Pan (drag), zoom (scroll wheel), click-to-select
- **Minimap**: Overview panel showing all agent positions
- **Sidebar panels**: World state, selected agent stats (health/energy/hunger bars), event log
- **Controls**: Play (auto-tick loop), Pause, Step (single tick), Reset
- **Tauri IPC**: All data fetched via `invoke()` calls to Rust backend

## Required Backend Commands

| Command | Returns | Description |
|---------|---------|-------------|
| `sim_get_state` | `{ tick, agent_count, alive_count }` | World simulation state |
| `sim_get_agents` | `[{ id, x, y, health, energy, hunger, alive }]` | All agent data |
| `sim_get_world_map` | `{ heightmap: number[][], biomes: string[][] }` | Terrain data |
| `sim_select_agent` | — | Select agent by ID |
| `sim_tick` | — | Advance simulation by N ticks |

## Design Tokens

- Background: `#1a1a2e`
- Panel: `#16213e`
- Border: `#0f3460`
- Accent: `#e94560`
- Agent alive: `#4ecca3`
- Agent selected: `#e94560`
