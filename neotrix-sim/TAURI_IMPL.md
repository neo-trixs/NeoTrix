# NT-WORLD-SIM Tauri Backend Implementation

## Status: ✅ Compiles Successfully

## Files Created

| File | Purpose |
|------|---------|
| `nt-world-sim/Cargo.toml` | Package manifest with tauri v2 + neotrix-sim deps |
| `nt-world-sim/build.rs` | Tauri build script |
| `nt-world-sim/src/main.rs` | Tauri commands wrapping WorldSim |
| `nt-world-sim/tauri.conf.json` | Tauri v2 config (window, security, frontend) |
| `nt-world-sim/dist/index.html` | Minimal HTML with canvas + sidebar |
| `nt-world-sim/dist/main.js` | Frontend JS with Tauri IPC calls |
| `nt-world-sim/icons/icon.png` | App icon (copied from src-tauri) |

## Tauri Commands

| Command | Signature | Description |
|---------|-----------|-------------|
| `sim_get_state` | `() -> SimStateDto` | Tick count, agent counts, mean stats |
| `sim_tick` | `(n: u32) -> SimStateDto` | Run N simulation ticks |
| `sim_get_agents` | `() -> Vec<AgentInfoDto>` | All agents with position/stats |
| `sim_get_world_map` | `() -> WorldMapData` | World dimensions + all agents |
| `sim_select_agent` | `(id: String) -> Option<AgentInfoDto>` | Single agent details |
| `sim_inject_action` | `(action: String) -> String` | Queue action string (TODO) |

## Architecture

```
Frontend (dist/main.js)
    ↕ Tauri IPC (invoke)
Backend (src/main.rs)
    ↕ Arc<Mutex<WorldSim>>
neotrix-sim (WorldSim)
```

- **State**: `Arc<SimState>` managed by Tauri, holds `tokio::sync::Mutex<WorldSim>`
- **DTOs**: `SimStateDto`, `AgentInfoDto`, `WorldMapData` — serializable snapshots
- **Tick**: async `WorldSim::tick()` called under lock, N ticks per command

## Build Fix Applied

Added `nt-world-sim` to workspace `members` in root `Cargo.toml` because `neotrix-sim` uses `workspace = true` for dependencies.

## TODO

- `sim_inject_action` currently stubs — needs string→AgentAction mapping
- Add icons for all platforms (currently only icon.png)
- Add `tauri-plugin-*` if IPC features needed
- Add build script for production bundling
