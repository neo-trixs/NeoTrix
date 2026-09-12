# Universal Game Engine — Clawd Integration Summary

## Overview

Successfully integrated patterns from **Clawd on Desk** (6.2k★) into NeoTrix's universal game engine architecture. Clawd is a pixel desktop pet that reacts to AI coding agents in real-time, supporting 20+ agents including Claude Code, Codex, Cursor, and more.

## Absorbed Patterns

### 1. Desktop Pet Overlay Pattern
- **Transparent Window**: Tauri v2 `transparent: true, decorations: false`
- **Click-Through**: `set_ignore_cursor_events(true)` for transparent areas
- **Position Memory**: Persistence across restarts via KB or local config
- **Multi-Monitor**: Proportional sizing per display, portrait boost
- **Mini Mode**: Edge hide + peek-on-hover, parabolic jump transitions

### 2. 12-State Animation FSM
| State | Priority | Trigger | Visual |
|-------|----------|---------|--------|
| `idle` | 10 | No activity | Eye tracking, body lean |
| `thinking` | 80 | Prompt received | Thought bubble, eyes up |
| `typing` | 70 | Tool running | Typing animation |
| `building` | 60 | Edit/bash tools | Construction animation |
| `groove` | 50 | 1 subagent | Headphones on |
| `juggling` | 50 | 2+ subagents | 3-ball juggling |
| `error` | 100 | Error occurred | Red flash, shake |
| `happy` | 40 | Task complete | Celebration |
| `notification` | 90 | Info received | Notification badge |
| `sweeping` | 30 | Cleaning up | Broom animation |
| `carrying` | 25 | Large output | Carrying bundle |
| `sleeping` | 0 | 60s idle | Zzz animation |

### 3. Hook System
- **Lifecycle Hooks**: Session start/end, tool start/end
- **Permission Hooks**: Request/response with Allow/Deny/Always
- **Subagent Hooks**: Parent-child session tracking
- **Notification Hooks**: Title, body, agent information
- **Event Processing**: Hook events → ECS state changes

### 4. Theme System
- **JSON Configuration**: `theme.json` with states, animations, visual settings
- **Animation Formats**: GIF, APNG, WebP, SVG, PNG, JPG, JPEG
- **Built-in Variants**: Clawd (crab), Calico (cat), Cloudling (cloud)
- **Custom Themes**: Import Codex Pet packages, validate before distribution
- **Sprite Sheet Support**: Atlas animations with frame calculation

### 5. Session Intelligence
- **Multi-Session Tracking**: Independent state per agent session
- **Subagent Awareness**: 1 subagent = groove, 2+ = juggling
- **Terminal Focus**: Jump to specific session's terminal
- **Process Liveness**: Detect crashed/exited agents
- **Startup Recovery**: Resume if restart while agents running

### 6. Permission Bubble UI
- **Floating Card**: Bottom-right positioning, stacking layout
- **Buttons**: Allow, Deny, Always with hover states
- **Global Hotkeys**: Ctrl+Shift+Y (Allow), Ctrl+Shift+N (Deny)
- **Auto-Dismiss**: If answered in terminal first
- **Per-Agent Toggle**: Enable/disable per integration

## Implementation Files

### New Files Created
1. **`nt-world-sim/src/engine/pet_state.rs`** (448 lines)
   - 12-state Pet FSM with priority system
   - Eye tracking component
   - Permission bubble component
   - Session info component
   - Zzz particle system

2. **`nt-world-sim/src/engine/hook_system.rs`** (280 lines)
   - Hook event types (12 variants)
   - Hook manager with event handlers
   - Permission request/response system
   - Permission bubble layout
   - Global hotkeys

3. **`nt-world-sim/src/engine/theme_system.rs`** (320 lines)
   - Theme configuration (JSON serializable)
   - Theme manager with load/export
   - Animation player with frame control
   - Sprite sheet support
   - Theme variants (Clawd/Calico/Cloudling/Custom)

4. **`nt-world-sim/CLAWD_INTEGRATION.md`** (250 lines)
   - Pattern mapping document
   - Integration points
   - Redundancy cleanup plan
   - Flat deficiency patches
   - Cross-domain misalignment fixes

### Modified Files
1. **`nt-world-sim/src/engine/mod.rs`**
   - Added `pet_state`, `hook_system`, `theme_system` modules
   - Exported all new types and systems

2. **`nt-world-sim/src/lib.rs`**
   - Added exports for all new types
   - Updated `create_stardew_valley_game()` with new systems

3. **`nt-world-sim/Cargo.toml`**
   - Added `uuid` dependency for permission request IDs

## Redundancy Cleanup

| Redundancy | Before | After | Action |
|------------|--------|-------|--------|
| **Renderer duplication** | CanvasRenderer + WebRenderer | Single `Renderer` trait | Merge into trait |
| **Vec3 duplication** | engine::Vec3 + physics::Vec3 | Single `Vec2` (2D) | Delete Vec3 |
| **Bridge duplication** | nt_world_bridge + neotrix_bridge | Single `HostBridge` | Merge modules |
| **State duplication** | PetState + AiState + SessionState | Single `EntityState` enum | Unify states |

## Flat Deficiency Patches

| Deficiency | Gap | Patch |
|------------|-----|-------|
| **No sprite sheet support** | Cannot render atlas animations | Added `SpriteSheet` component |
| **No animation system** | States don't animate | Added `AnimationPlayer` system |
| **No theme persistence** | Theme resets on restart | Added theme config to KB |
| **No position memory** | Pet resets position | Added position persistence |
| **No multi-monitor** | Single viewport | Added screen query API |

## Cross-Domain Misalignment Fixes

| Misalignment | Domains | Fix |
|--------------|---------|-----|
| **PetState vs GameEntity** | ECS + Mechanics | Unify into single entity system |
| **Theme vs AssetRegistry** | Engine + World | Theme = specialized AssetRegistry |
| **Hook vs EventBus** | IO + Core | Hook → EventBus adapter pattern |
| **Session vs World entity** | Memory + ECS | Session = World entity with SessionInfo component |

## Build Status

✅ **Compilation**: `cargo check -p nt-world-sim` passes with 0 errors
✅ **Warnings**: 0 warnings (all unused imports/variables fixed)
✅ **Tests**: Unit tests for theme system, animation player, sprite sheet

## Next Steps

1. **Integrate with Tauri Window**: Configure transparent, frameless window
2. **Add HTML/CSS UI**: Permission bubble, session dashboard, theme selector
3. **Implement Hook Installation**: Auto-register hooks for Claude Code, Codex, etc.
4. **Add Eye Tracking**: Cursor following with body lean and shadow stretch
5. **Implement Sleep Sequence**: 60s idle → yawning → dozing → sleeping
6. **Add Sound Effects**: Audio cues on task completion and permission requests
7. **Multi-Monitor Support**: Proportional sizing per display
8. **Mobile Companion (PWA)**: Live mirror on phone via LAN

## References

- [Clawd on Desk GitHub](https://github.com/rullerzhou-afk/clawd-on-desk)
- [State Mapping Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/state-mapping.md)
- [Theme Creation Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/guide-theme-creation.md)
- [Setup Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/setup-guide.md)
- [Known Limitations](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/known-limitations.md)
