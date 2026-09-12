# Clawd on Desk — Integration Guide

## Overview

Clawd on Desk (6.2k★) is a pixel desktop pet that reacts to AI coding agents in real-time. This document maps Clawd's patterns to NeoTrix's universal game engine architecture.

## Key Patterns Absorbed

### 1. Desktop Pet Overlay

**Clawd Implementation:**
- Electron `BrowserWindow({ transparent: true, frame: false })`
- `setIgnoreMouseEvents(true, { forward: true })` for click-through
- localStorage for position memory
- Multi-monitor proportional sizing

**NeoTrix Mapping:**
```rust
// Tauri v2 configuration
tauri::Builder::default()
    .setup(|app| {
        let window = app.get_webview_window("main").unwrap();
        window.set_transparent(true)?;
        window.set_decorations(false)?;
        window.set_ignore_cursor_events(true)?;
        window.set_always_on_top(true)?;
        Ok(())
    })
```

### 2. 12-State Animation FSM

**State Transitions:**
```
idle -> thinking -> typing -> building -> groove/juggling
  |                                        |
  v                                        v
sleeping <- error <- notification <- sweeping <- happy
```

**Priority System:**
- Error: 100 (highest)
- Notification: 90
- Thinking: 80
- Typing: 70
- Building: 60
- Groove/Juggling: 50
- Happy: 40
- Sweeping: 30
- Carrying: 25
- Idle: 10
- Sleeping: 0 (lowest)

### 3. Hook System

**Hook Types:**
- Session lifecycle (start/end)
- Tool events (start/end)
- Permission requests/responses
- Subagent events (start/end)
- Notifications

**Registration Pattern:**
```rust
pub struct HookConfig {
    pub agent: String,
    pub hooks_dir: PathBuf,
    pub http_port: Option<u16>,
    pub enabled: bool,
}
```

### 4. Theme System

**Theme Configuration:**
```json
{
  "name": "Clawd",
  "author": "rullerzhou-afk",
  "states": {
    "idle": { "format": "gif", "path": "idle.gif" },
    "thinking": { "format": "gif", "path": "thinking.gif" }
  },
  "scale": 1.0,
  "eye_tracking": true
}
```

**Supported Formats:**
- GIF, APNG, WebP, SVG, PNG, JPG, JPEG

### 5. Session Intelligence

**Session Tracking:**
- Multi-session support across agents
- Subagent awareness (1 = groove, 2+ = juggling)
- Terminal focus jumping
- Process liveness detection

**Priority Resolution:**
```rust
pub fn session_priority(state: &PetState) -> i32 {
    match state {
        PetState::Error { .. } => 100,
        PetState::Thinking { .. } => 80,
        // ...
    }
}
```

### 6. Permission Bubble

**UI Pattern:**
- Floating card from bottom-right
- Stacking layout for multiple requests
- Allow/Deny/Always buttons
- Global hotkeys (Ctrl+Shift+Y/N)

**Handling Modes:**
- Ask every time
- Question prompts only
- Auto-approve (degrades after restart)

## Integration Points

### 1. Tauri Window Configuration

```rust
// src-tauri/tauri.conf.json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true
      }
    ]
  }
}
```

### 2. ECS Components

```rust
// Pet state component
pub struct PetStateComponent {
    pub state: PetState,
    pub state_timer: f32,
    pub idle_timer: f32,
}

// Eye tracking component
pub struct EyeTracking {
    pub cursor_position: Vec2,
    pub eye_position: Vec2,
    pub max_offset: f32,
}

// Permission bubble component
pub struct PermissionBubble {
    pub active: bool,
    pub request_id: Option<String>,
    pub position: Vec2,
    pub opacity: f32,
}

// Session info component
pub struct SessionInfo {
    pub id: String,
    pub agent: String,
    pub alias: String,
    pub subagents: Vec<SubagentInfo>,
}
```

### 3. Systems

```rust
// Pet state system
pub struct PetStateSystem;
impl System for PetStateSystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // Update state timers
        // Handle transitions
        // Update Zzz particles
    }
}

// Eye tracking system
pub struct EyeTrackingSystem;
impl System for EyeTrackingSystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // Only active in idle state
        // Smooth eye movement
    }
}

// Permission bubble system
pub struct PermissionBubbleSystem;
impl System for PermissionBubbleSystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // Fade in/out
        // Button hover states
    }
}

// Session system
pub struct SessionSystem;
impl System for SessionSystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // Clean expired sessions
        // Update subagent counts
    }
}
```

## Redundancy Cleanup

| Redundancy | Before | After | Action |
|------------|--------|-------|--------|
| Renderer duplication | CanvasRenderer + WebRenderer | Single `Renderer` trait | Merge |
| Vec3 duplication | engine::Vec3 + physics::Vec3 | Single `Vec2` | Delete Vec3 |
| Bridge duplication | nt_world_bridge + neotrix_bridge | Single `HostBridge` | Merge |
| State duplication | PetState + AiState + SessionState | Single `EntityState` | Unify |

## Flat Deficiency Patches

| Deficiency | Gap | Patch |
|------------|-----|-------|
| No sprite sheet support | Cannot render atlas animations | Add `SpriteSheet` component |
| No animation system | States don't animate | Add `AnimationPlayer` system |
| No theme persistence | Theme resets on restart | Add theme config to KB |
| No position memory | Pet resets position | Add position persistence |
| No multi-monitor | Single viewport | Add screen query API |

## Cross-Domain Misalignment Fixes

| Misalignment | Domains | Fix |
|--------------|---------|-----|
| PetState vs GameEntity | ECS + Mechanics | Unify into single entity system |
| Theme vs AssetRegistry | Engine + World | Theme = specialized AssetRegistry |
| Hook vs EventBus | IO + Core | Hook -> EventBus adapter pattern |
| Session vs World entity | Memory + ECS | Session = World entity with SessionInfo component |

## Testing Strategy

1. **Unit Tests:** PetState transitions, priority resolution
2. **Integration Tests:** Hook event -> ECS state change
3. **Visual Tests:** Animation rendering, permission bubble layout
4. **Performance Tests:** Multi-session tracking, subagent awareness

## References

- [Clawd on Desk GitHub](https://github.com/rullerzhou-afk/clawd-on-desk)
- [State Mapping Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/state-mapping.md)
- [Theme Creation Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/guide-theme-creation.md)
- [Setup Guide](https://github.com/rullerzhou-afk/clawd-on-desk/blob/main/docs/guides/setup-guide.md)
