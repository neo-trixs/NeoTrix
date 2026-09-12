# NT-WORLD-SIM

Consciousness evolution simulation engine inspired by Stardew Valley.

## Features

- 6-layer consciousness architecture (Meta-Cognition → Action)
- 8 NPCs with dialogue trees, gift preferences, and schedules
- 8 crop types with full growth lifecycle
- 30 crafting recipes with skill requirements
- 30 cooking recipes with energy restoration
- 10 fish types with season/time/depth spawning
- 6 forage items with zone restrictions
- 5 skills × 10 levels with profession specializations
- 27 quests (Main/Side/Daily/Special/Achievement)
- 8 seasonal events (Flower Festival, Harvest Festival, etc.)
- Marriage and children system with skill inheritance
- Perfection tracker across 8 endgame categories
- Turn-based combat with 4 enemy types
- Procedural mine floors with mineral extraction
- Multi-engine code generation (Bevy/Unity/Godot/Unreal)

## Quick Start

```bash
cargo build -p nt-world-sim
cargo test -p nt-world-sim
```

### With Tauri Desktop Support

```bash
cargo build -p nt-world-sim --features tauri
```

## HTML5 Games

- `dist/consciousness_valley.html` — Main game
- `dist/launcher.html` — Game launcher
- `dist/evolution.html` — Evolution visualization

## Architecture

See `docs/ARCHITECTURE.md` for the full architecture overview.
See `docs/SYSTEMS.md` for detailed system reference.

## Module Structure

```
src/
├── core/          # ECS: Entity, World, Scheduler, Math
├── engine/        # Renderer, Physics, Input, Audio, Scene, Camera, Assets
├── game/          # All game mechanics (26 modules)
├── world/         # Tiles, Generator, Zones, Pathfinding
├── ui/            # Theme, Widgets, HUD, Inventory UI, Dialogue, Minimap
├── codegen/       # Game definition parser + code generator
├── adapters/      # Bevy/Unity/Godot/Unreal engine adapters
├── save/          # Slot-based save/load system
├── builder.rs     # Fluent game builder API
├── error.rs       # GameError enum
└── tauri_bridge.rs # Tauri desktop integration (behind "tauri" feature)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `serde` / `serde_json` | Serialization |
| `serde_yaml` | YAML game definition parsing |
| `once_cell` | Lazy statics |
| `tokio` | Async runtime |
| `rand` | Random number generation |
| `uuid` | Unique ID generation |
| `neotrix-sim` | Simulation primitives |
| `tauri` (optional) | Desktop framework |

## License

Part of the NeoTrix project.
