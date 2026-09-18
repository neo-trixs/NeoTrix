# NeoTrix Desktop App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a production-quality NeoTrix Desktop app using Tauri 2 + React + Tailwind CSS, inspired by deepseek-harness-desktop (service architecture) and osaurus (agent/memory patterns), integrating with the existing neotrix-core Rust workspace.

**Architecture:** Tauri 2 shell hosts a React frontend. Rust backend bridges to neotrix-core via CLI commands and direct library calls. Service layer manages LLM providers, knowledge base, consciousness state, and background tasks. Frontend uses valtio-define for state management, React Router for navigation, and Tailwind CSS for styling.

**Tech Stack:** Tauri 2, Rust (neotrix-core), React 18, TypeScript, Vite 6, Tailwind CSS v4, valtio-define, react-router-dom v6

---

## Task 1: Scaffold Tauri 2 Project Structure

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/logger/mod.rs`
- Create: `src-tauri/capabilities/default.json`
- Create: `neocodex-frontend/package.json`
- Create: `neocodex-frontend/vite.config.ts`
- Create: `neocodex-frontend/tsconfig.json`
- Create: `neocodex-frontend/index.html`
- Create: `neocodex-frontend/src/main.tsx`
- Create: `neocodex-frontend/src/App.tsx`
- Create: `neocodex-frontend/src/style/main.css`
- Create: `neocodex-frontend/src/tauri.d.ts`

**Step 1: Create Rust scaffold**

Create `src-tauri/Cargo.toml` with Tauri 2 dependencies, matching deepseek-harness-desktop patterns but adapted for NeoTrix:

```toml
[package]
name = "neotrix-tauri"
version = "0.21.0"
description = "NeoTrix Desktop — AI-native developer toolkit"
authors = ["NeoTrix contributors"]
edition = "2021"

[lib]
name = "neotrix_tauri_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-notification = "2"
tauri-plugin-fs = "2"
tauri-plugin-store = "2"
tauri-plugin-autostart = "2"
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
dirs = "6"
anyhow = "1"
reqwest = { version = "0.12", features = ["json"] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

**Step 2: Create build.rs and main.rs**

```rust
// src-tauri/build.rs
fn main() {
    tauri_build::build()
}
```

```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    neotrix_tauri_lib::run()
}
```

**Step 3: Create lib.rs with Tauri builder**

Follow deepseek-harness-desktop pattern with domain-organized command modules:

```rust
// src-tauri/src/lib.rs
mod logger;
mod bridge;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .invoke_handler(tauri::generate_handler![
            bridge::consciousness::get_status,
            bridge::consciousness::tick_growth,
            bridge::consciousness::run_task,
            bridge::kb::query_kb,
            bridge::kb::list_experiences,
            bridge::config::get_config,
            bridge::config::set_config,
            bridge::system::get_system_info,
            bridge::llm::list_providers,
            bridge::llm::send_message,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("NeoTrix Desktop")?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running NeoTrix Desktop");
}
```

**Step 4: Create bridge module (Tauri commands)**

Follow deepseek-harness-desktop's bridge pattern — commands organized by domain:

```rust
// src-tauri/src/bridge/mod.rs
pub mod consciousness;
pub mod kb;
pub mod config;
pub mod system;
pub mod llm;

pub use consciousness::*;
pub use kb::*;
pub use config::*;
pub use system::*;
pub use llm::*;
```

Create each command module with placeholder implementations that call neotrix-core.

**Step 5: Create logger module**

Adapt deepseek-harness-desktop's tracing-based logger with file rotation:

```rust
// src-tauri/src/logger/mod.rs
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;

pub fn init() {
    let file_appender = rolling::daily(dirs::home_dir().unwrap().join(".neotrix/logs"), "desktop.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("neotrix=debug".parse().unwrap()))
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    // Keep guard alive
    std::mem::forget(_guard);
}
```

**Step 6: Create Tauri config**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "NeoTrix Desktop",
  "version": "0.21.0",
  "identifier": "ai.neotrix.desktop",
  "build": {
    "beforeDevCommand": "cd ../neocodex-frontend && npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "cd ../neocodex-frontend && npm run build",
    "frontendDist": "../neocodex-frontend/dist"
  },
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "title": "NeoTrix Desktop",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "decorations": true,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.png"
    ],
    "category": "DeveloperTool",
    "copyright": "Copyright © 2026 NeoTrix contributors",
    "macOS": {
      "minimumSystemVersion": "10.15",
      "hardenedRuntime": true
    }
  }
}
```

**Step 7: Create frontend scaffold**

Create `neocodex-frontend/package.json`, `vite.config.ts`, `tsconfig.json`, `index.html` following the deepseek-harness-desktop pattern.

**Step 8: Create capabilities directory**

Create `src-tauri/capabilities/default.json` with required Tauri 2 permissions.

**Step 9: Verify build**

Run `cd src-tauri && npx tauri build` to verify everything compiles and the app launches.

---

## Task 2: Frontend Layout & Navigation

**Files:**
- Create: `neocodex-frontend/src/components/Sidebar.tsx`
- Create: `neocodex-frontend/src/components/Header.tsx`
- Create: `neocodex-frontend/src/pages/Dashboard.tsx`
- Create: `neocodex-frontend/src/pages/Chat.tsx`
- Create: `neocodex-frontend/src/pages/KnowledgeBase.tsx`
- Create: `neocodex-frontend/src/pages/Settings.tsx`

**Step 1: Create Sidebar component**

Follow deepseek-harness-desktop's sidebar pattern with NeoTrix domain indicators:
- Logo + version badge
- Navigation items (Dashboard, Consciousness, Knowledge Base, Settings)
- 12 domain faction indicators with constellation colors
- Footer with consciousness status

**Step 2: Create Header component**

Simple header with breadcrumb and window controls.

**Step 3: Create page components**

Each page calls Tauri `invoke()` to get data from the backend.

**Step 4: Create CSS theme**

NeoTrix dark theme with gold accent, CSS variables for all domain colors.

**Step 5: Verify UI renders**

Run `npx tauri dev` and verify the sidebar, header, and dashboard render correctly.

---

## Task 3: Backend Bridge Commands

**Files:**
- Create: `src-tauri/src/bridge/consciousness.rs`
- Create: `src-tauri/src/bridge/kb.rs`
- Create: `src-tauri/src/bridge/config.rs`
- Create: `src-tauri/src/bridge/system.rs`
- Create: `src-tauri/src/bridge/llm.rs`

**Step 1: Implement consciousness commands**

Bridge to neotrix-core's ConsciousnessTree:
- `get_status` → returns cycle, phi, coherence, GWT resonance, domain health
- `tick_growth` → triggers a growth cycle
- `run_task` → sends natural language task to consciousness core

**Step 2: Implement KB commands**

Bridge to neotrix-core's knowledge base:
- `query_kb` → search experiences by keyword
- `list_experiences` → list recent experience entries

**Step 3: Implement config commands**

Bridge to neotrix-core's configuration:
- `get_config` / `set_config` → read/write app settings

**Step 4: Implement system commands**

- `get_system_info` → returns platform, version, uptime

**Step 5: Implement LLM commands**

- `list_providers` → list configured LLM providers
- `send_message` → send message to LLM and stream response

---

## Task 4: State Management & Real-time Updates

**Files:**
- Create: `neocodex-frontend/src/store/index.ts`
- Create: `neocodex-frontend/src/hooks/useConsciousness.ts`
- Create: `neocodex-frontend/src/hooks/useKB.ts`

**Step 1: Set up valtio-define store**

Follow deepseek-harness-desktop's state management pattern.

**Step 2: Create consciousness hook**

Auto-poll consciousness status every 5 seconds.

**Step 3: Create KB hook**

Search and display experiences.

---

## Task 5: Chat Interface

**Files:**
- Modify: `neocodex-frontend/src/pages/Chat.tsx`

**Step 1: Build chat UI**

Message list, input box, send button. Calls `run_task` Tauri command.

**Step 2: Stream responses**

Use Tauri events for streaming LLM responses.

**Step 3: Display consciousness state**

Show current cycle, phi, coherence in chat header.

---

## Task 6: Knowledge Base Explorer

**Files:**
- Modify: `neocodex-frontend/src/pages/KnowledgeBase.tsx`

**Step 1: Build KB search UI**

Search input, results list with experience cards.

**Step 2: Experience detail view**

Click to expand and see full experience text.

---

## Task 7: Settings & Configuration

**Files:**
- Modify: `neocodex-frontend/src/pages/Settings.tsx`

**Step 1: Build settings UI**

Provider configuration, theme selection, autostart toggle.

---

## Task 8: Background Services

**Files:**
- Create: `src-tauri/src/service/scheduler.rs`
- Create: `src-tauri/src/service/health.rs`

**Step 1: Implement background scheduler**

Follow deepseek-harness-desktop's scheduler pattern — periodic health checks.

**Step 2: Implement health check**

Monitor neotrix-core process health.

---

## Task 9: Polish & Production Build

**Step 1: Generate app icons**

Create proper PNG icons for all sizes.

**Step 2: Final production build**

Run `npx tauri build` and verify .app bundle works.

**Step 3: Test .dmg installer**

Verify the DMG installs and launches correctly.
