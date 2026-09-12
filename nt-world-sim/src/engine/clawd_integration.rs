# Universal Game Engine — Clawd Integration

## Pattern Absorption (2026-09-12, Clawd on Desk)

### Desktop Pet Pattern

| Feature | Clawd Implementation | NeoTrix Universal Engine |
|---------|---------------------|--------------------------|
| **Transparent Window** | Electron `BrowserWindow({ transparent: true, frame: false })` | Tauri `transparent: true, decorations: false` |
| **Click-Through** | `setIgnoreMouseEvents(true, { forward: true })` | `set_ignore_cursor_events(true)` in Tauri v2 |
| **Position Memory** | localStorage persistence across restarts | KB persistence or local config |
| **Multi-Monitor** | Proportional sizing per display, portrait boost | Screen info query + adaptive viewport |
| **Mini Mode** | Edge hide + peek-on-hover, parabolic jump | State machine with screen-edge physics |
| **Drag from Any State** | Pointer Capture API prevents fast-flick drops | `on_mouse_drag` event + velocity tracking |
| **Sleep Sequence** | 60s idle → yawning → dozing → sleeping | Timer system + animation state machine |
| **Click Reactions** | Double-click poke, 4-click flail | Input counter + animation trigger |

### 12-State Animation FSM

```
idle → thinking → typing → building → subagent (groove/juggling)
  ↓                                         ↓
sleeping ← error ← notification ← sweeping ← happy
```

| State | Trigger | Visual | NeoTrix ECS Component |
|-------|---------|--------|----------------------|
| `idle` | No activity | Eye tracking, body lean | `PetState::Idle { eye_target: Vec2 }` |
| `thinking` | Prompt received | Thought bubble, eyes up | `PetState::Thinking { duration: f32 }` |
| `typing` | Tool running | Typing animation | `PetState::Typing { progress: f32 }` |
| `building` | Edit/bash tools | Construction animation | `PetState::Building { tools: Vec<String> }` |
| `groove` | 1 subagent | Headphones on | `PetState::Groove { agent_id: u64 }` |
| `juggling` | 2+ subagents | 3-ball juggling | `PetState::Juggling { count: usize }` |
| `error` | Error occurred | Red flash, shake | `PetState::Error { message: String }` |
| `happy` | Task complete | Celebration | `PetState::Happy { duration: f32 }` |
| `notification` | Info received | Notification badge | `PetState::Notification { text: String }` |
| `sweeping` | Cleaning up | Broom animation | `PetState::Sweeping { items: usize }` |
| `carrying` | Large output | Carrying bundle | `PetState::Carrying { size: f32 }` |
| `sleeping` | 60s idle | Zzz animation | `PetState::Sleeping { zzz: Vec<ZzzParticle> }` |

### Hook System Pattern

```rust
/// Hook event types (from Clawd)
pub enum HookEvent {
    // Lifecycle
    SessionStart { agent: String, session_id: String },
    SessionEnd { agent: String, session_id: String },
    
    // Tool events
    ToolStart { tool: String, args: serde_json::Value },
    ToolEnd { tool: String, result: serde_json::Value },
    
    // Permission
    PermissionRequest { tool: String, args: serde_json::Value },
    PermissionResponse { approved: bool, always: bool },
    
    // Subagent
    SubagentStart { parent: String, child: String },
    SubagentEnd { child: String },
    
    // Notification
    Notification { title: String, body: String },
}

/// Hook registration (per agent)
pub struct HookConfig {
    pub agent: String,
    pub hooks_dir: PathBuf,      // e.g., ~/.claude/hooks/
    pub http_port: Option<u16>,  // For permission hooks
    pub enabled: bool,
}
```

### Theme System Pattern

```rust
/// Theme configuration (from Clawd theme.json)
pub struct ThemeConfig {
    pub name: String,
    pub author: String,
    pub version: String,
    
    // States → animation files
    pub states: HashMap<PetState, AnimationDef>,
    
    // Visual settings
    pub scale: f32,
    pub anchor: Vec2,        // Attachment point
    pub shadow: ShadowDef,
    pub eye_tracking: bool,
}

/// Animation definition
pub struct AnimationDef {
    pub format: String,      // "gif", "apng", "webp", "svg", "png"
    pub path: String,        // Relative to theme dir
    pub frames: Option<u32>, // For sprite sheets
    pub fps: Option<f32>,    // Override default FPS
}

/// Theme variants (Clawd has 3 built-in)
pub enum ThemeVariant {
    Clawd,      // Pixel crab
    Calico,     // 三花猫
    Cloudling,  // 云宝
    Custom(PathBuf),
}
```

### Session Intelligence Pattern

```rust
/// Session tracking (from Clawd Dashboard)
pub struct SessionInfo {
    pub id: String,
    pub agent: String,
    pub alias: String,        // e.g., "Claude Code — ~/project"
    pub state: PetState,
    pub started_at: Instant,
    pub last_event: Instant,
    pub folder: PathBuf,
    pub terminal_pid: Option<u32>,
    pub subagents: Vec<SubagentInfo>,
    pub permission_pending: Option<PermissionRequest>,
}

/// Subagent awareness
pub struct SubagentInfo {
    pub id: String,
    pub parent_session: String,
    pub state: PetState,
    pub started_at: Instant,
}

/// Session priority (highest wins for display)
pub fn session_priority(state: &PetState) -> i32 {
    match state {
        PetState::Error { .. } => 100,
        PetState::PermissionRequest { .. } => 90,
        PetState::Thinking { .. } => 80,
        PetState::Typing { .. } => 70,
        PetState::Building { .. } => 60,
        PetState::Groove { .. } => 50,
        PetState::Juggling { .. } => 50,
        PetState::Happy { .. } => 40,
        PetState::Notification { .. } => 35,
        PetState::Sweeping { .. } => 30,
        PetState::Carrying { .. } => 25,
        PetState::Idle { .. } => 10,
        PetState::Sleeping { .. } => 0,
    }
}
```

### Permission Bubble Pattern

```rust
/// Permission bubble layout (from Clawd)
pub struct PermissionBubble {
    pub request: PermissionRequest,
    pub position: Vec2,        // Bottom-right stacking
    pub z_order: u32,
    pub opacity: f32,          // Fade in/out
    pub buttons: Vec<Button>,  // Allow, Deny, Always
}

/// Permission request
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub agent: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub timestamp: Instant,
}

/// Permission handling modes
pub enum PermissionMode {
    AskEveryTime,
    QuestionPromptsOnly,
    AutoApprove,  // Degrades after restart
}

/// Global hotkeys
pub struct PermissionHotkeys {
    pub allow: String,  // Ctrl+Shift+Y
    pub deny: String,   // Ctrl+Shift+N
}
```

## Integration Points

### 1. Desktop Pet → Tauri Window

```rust
// Tauri v2 configuration for desktop pet
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
        let window = app.get_webview_window("main").unwrap();
        
        // Transparent, frameless, click-through
        window.set_transparent(true)?;
        window.set_decorations(false)?;
        window.set_ignore_cursor_events(true)?;
        
        // Always on top
        window.set_always_on_top(true)?;
        
        // Position memory (load from config)
        if let Some(pos) = load_position() {
            window.set_position(pos)?;
        }
        
        Ok(())
    })
```

### 2. Hook System → EventBus

```rust
// Hook events flow into ECS via EventBus
pub fn process_hook_event(event: HookEvent, world: &mut World) {
    match event {
        HookEvent::SessionStart { agent, session_id } => {
            let entity = world.spawn();
            world.insert_component(entity, SessionInfo::new(&agent, &session_id));
            world.insert_component(entity, PetState::Thinking { duration: 0.0 });
        }
        HookEvent::ToolStart { tool, .. } => {
            // Update all active sessions' state
            let sessions = world.query::<SessionInfo>();
            for session_entity in sessions {
                if let Some(state) = world.get_component_mut::<PetState>(session_entity) {
                    *state = PetState::Typing { progress: 0.0 };
                }
            }
        }
        // ... more handlers
    }
}
```

### 3. Theme System → Asset Pipeline

```rust
// Theme loading from JSON
pub fn load_theme(path: &Path) -> Result<ThemeConfig> {
    let json = std::fs::read_to_string(path.join("theme.json"))?;
    let config: ThemeConfig = serde_json::from_str(&json)?;
    
    // Validate all animation files exist
    for (state, anim) in &config.states {
        let anim_path = path.join(&anim.path);
        if !anim_path.exists() {
            return Err(anyhow!("Missing animation for {:?}: {}", state, anim.path));
        }
    }
    
    Ok(config)
}
```

### 4. Session Intelligence → World Query

```rust
// Get highest priority session for display
pub fn get_active_session(world: &World) -> Option<(Entity, &SessionInfo)> {
    let sessions = world.query::<SessionInfo>();
    sessions
        .filter_map(|entity| {
            let info = world.get_component::<SessionInfo>(entity)?;
            Some((entity, info))
        })
        .max_by_key(|(_, info)| session_priority(&info.state))
}
```

## Redundancy Cleanup (Post-Absorption)

| Redundancy | Before | After | Action |
|------------|--------|-------|--------|
| **Renderer duplication** | CanvasRenderer + WebRenderer | Single `Renderer` trait | Merge into trait |
| **Vec3 duplication** | engine::Vec3 + physics::Vec3 | Single `Vec2` (2D) | Delete Vec3 |
| **Bridge duplication** | nt_world_bridge + neotrix_bridge | Single `HostBridge` | Merge modules |
| **State duplication** | PetState + AiState + SessionState | Single `EntityState` enum | Unify states |

## Flat Deficiency Patches

| Deficiency | Gap | Patch |
|------------|-----|-------|
| **No sprite sheet support** | Cannot render atlas animations | Add `SpriteSheet` component to Renderer |
| **No animation system** | States don't animate | Add `AnimationPlayer` system |
| **No theme persistence** | Theme resets on restart | Add theme config to KB |
| **No position memory** | Pet resets position | Add position persistence |
| **No multi-monitor** | Single viewport | Add screen query API |

## Cross-Domain Misalignment Fixes

| Misalignment | Domains | Fix |
|--------------|---------|-----|
| **PetState vs GameEntity** | ECS + Mechanics | Unify into single entity system |
| **Theme vs AssetRegistry** | Engine + World | Theme = specialized AssetRegistry |
| **Hook vs EventBus** | IO + Core | Hook → EventBus adapter pattern |
| **Session vs World entity** | Memory + ECS | Session = World entity with SessionInfo component |
