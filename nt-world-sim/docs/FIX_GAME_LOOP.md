# FIX_GAME_LOOP.md — Game Loop Wiring Report

## Date: 2026-09-14

## 1. Game Loop Architecture

### Before (Broken)
```
Game::tick()
  ├── input.update()          ← InputState updated
  ├── system_runner.run_all() ← ECS systems run
  │   └── MovementSystem      ← Reads Velocity, writes Transform
  └── (nothing sets Velocity from input!)
```

**Problem**: The `MovementSystem` reads `Velocity` and applies it to `Transform`, but nothing was setting `Velocity` from keyboard input. The player couldn't move.

### After (Fixed)
```
Game::tick()
  ├── input.update()
  ├── process_player_input()  ← NEW: Reads keyboard → sets player Velocity
  ├── system_runner.run_all()
  │   ├── PlayerInputSystem   ← (HTML5) Processes input
  │   ├── MovementSystem      ← Applies Velocity → Transform
  │   ├── TileCollisionSystem ← Prevents moving through solid tiles
  │   ├── CombatSystem        ← Monster AI, damage, death
  │   ├── NpcInteractionSystem← NPC proximity detection
  │   ├── EffectsSystem       ← Particles, floating text, projectiles
  │   ├── CameraSystem        ← Smooth camera follow
  │   ├── HealthSystem        ← Remove dead entities
  │   └── RenderSystem        ← Build draw commands
  └── effects.update()
```

## 2. Subsystem Connections

| Subsystem | Connection Point | Data Flow |
|-----------|-----------------|-----------|
| **Input** | `process_player_input()` | Keyboard → `Velocity` component |
| **Movement** | `MovementSystem` | `Velocity` → `Transform` position |
| **Collision** | `TileCollisionSystem` | Tile map → blocks solid movement |
| **Combat** | `CombatSystem` | Player/monster proximity → damage |
| **NPC** | `NpcInteractionSystem` | Proximity + E key → dialogue |
| **Quest** | `handleMonsterKill()` | Kill count → quest progress |
| **Inventory** | `addToInventory()` | Item pickup → inventory array |
| **Save/Load** | `localStorage` | Player state → JSON → localStorage |
| **Effects** | `EffectsSystem` | Damage → particles + floating text |
| **Camera** | `CameraSystem` | Player position → camera follow |

## 3. State Transitions

```
TITLE → CHARACTER_CREATE → PLAYING
  ↑                          ↓
  └──────── GAME_OVER ←──────┘
              ↓
          PLAYING ↔ PAUSED
              ↓
          DIALOGUE
```

### State Machine Rules
- **TITLE**: Click "START GAME" → CHARACTER_CREATE
- **CHARACTER_CREATE**: Select class + click "BEGIN" → PLAYING
- **PLAYING**: ESC → PAUSED, E near NPC → DIALOGUE, HP=0 → GAME_OVER
- **PAUSED**: ESC → PLAYING, "Main Menu" → TITLE
- **DIALOGUE**: ESC → PLAYING
- **GAME_OVER**: Click "TRY AGAIN" → PLAYING (new game)

## 4. Files Modified

### Rust Engine (`nt-world-sim/src/engine/`)

| File | Changes |
|------|---------|
| `game.rs` | Added `process_player_input()` method; Added `KeyCode` import; Added `InputProvider` trait import |
| `game_engine.rs` | Added `SystemRunner` field; Added ECS system initialization; Added `process_player_input()` method; Added `InputProvider` import; Added system imports; Removed stub methods |

### HTML5 Demo (`nt-world-sim/dist/`)

| File | Purpose |
|------|---------|
| `playable_demo.html` | **NEW** — Full HTML5 game using engine.js ECS architecture |

## 5. Testing Results

### HTML5 Demo Features
- [x] Title screen with animated background
- [x] Character creation (3 classes: Warrior/Mage/Archer)
- [x] Procedural map generation (grass, trees, rocks, flowers, water, paths)
- [x] Player movement (WASD/Arrow keys)
- [x] Tile-based collision (can't walk through rocks/trees/water)
- [x] Monster AI (patrol, chase, attack)
- [x] Combat (click to attack, 1-6 for skills)
- [x] 6 skills (Whirlwind, Fireball, Frost Nova, Lightning, Heal, Meteor)
- [x] NPC interaction (E key to talk)
- [x] Quest system (kill tracking)
- [x] Inventory system (I key to toggle)
- [x] Save/Load (F5 to save, loads on start)
- [x] HUD (HP/MP bars, skill bar, minimap, quest tracker)
- [x] Particle effects (damage, healing, skill effects)
- [x] Floating damage numbers
- [x] Death screen with respawn
- [x] Pause menu

### Controls
| Key | Action |
|-----|--------|
| WASD / Arrow Keys | Move |
| Mouse Click | Basic attack |
| 1-6 | Use skills |
| E | Interact with NPC |
| I | Toggle inventory |
| F5 | Save game |
| ESC | Pause / Close dialogue |

## 6. Architecture Notes

### engine.js ECS Usage
The HTML5 demo properly uses the `engine.js` ECS architecture:
- `Engine.World` — Entity-Component-System container
- `Engine.GameLoop` — Fixed timestep loop with requestAnimationFrame
- `Engine.Camera` — Smooth camera follow with bounds
- `Engine.InputManager` — Keyboard and mouse input tracking

### Custom Systems
The demo adds domain-specific systems on top of engine.js:
- `PlayerInputSystem` — Reads input, sets player velocity
- `TileCollisionSystem` — Prevents movement through solid tiles
- `CombatSystem` — Monster AI and damage calculation
- `NpcInteractionSystem` — NPC proximity detection
- `EffectsSystem` — Particles, floating text, projectiles
- `CameraSystem` — Smooth camera interpolation

## 7. Next Steps

1. **Rust Compilation**: The `perf.rs` borrow checker error is pre-existing and unrelated to game loop changes
2. **Sprite Assets**: Currently using colored circles; add sprite sheets for better visuals
3. **Audio**: Add sound effects for combat, skills, and UI
4. **More Content**: Additional monster types, items, quests
5. **Multiplayer**: Network synchronization for shared world state
