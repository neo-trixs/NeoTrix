# Desktop App

NeoTrix Desktop is a Tauri-based native application providing a graphical interface for NeoTrix. Available on macOS, Linux, and Windows.

## Installation

```bash
brew install neotrix-desktop
```

Or download the latest release from the [releases page](https://github.com/neotrix/neotrix/releases).

## Features

### Session Management

- **Persistent sessions** — sessions survive app restarts
- **Session history** — browse, search, and reopen previous sessions
- **Session export** — export as Markdown, JSON, or plain text

### File Access

- **Native file picker** — attach files, open directories
- **Drag & drop** — drag files into the conversation pane
- **Image preview** — inline rendering of supported image formats
- **Code viewer** — syntax-highlighted code blocks with copy support

### Settings Panel

- **Provider configuration** — add and switch between API providers
- **Model selection** — pick the active model per session
- **Theme** — light, dark, or system-auto
- **Font size** — adjustable editor and output font
- **Keyboard shortcuts** — fully customizable

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Enter` | Send message |
| `Cmd+K` | Clear conversation |
| `Cmd+N` | New session |
| `Cmd+S` | Save session |
| `Cmd+O` | Open file |
| `Cmd+,` | Open settings |
| `Cmd+Shift+P` | Command palette |
| `Escape` | Cancel current request |

## Building from Source

```bash
# Prerequisites: Rust, Node.js, pnpm
cd src-tauri/frontend
pnpm install
pnpm build
cd ../..
cargo build -p neotrix-tauri
./target/release/neotrix-desktop
```

## System Tray

The desktop app minimizes to the system tray. Right-click the tray icon to:
- Show/hide the window
- Start a new session
- Quit the application
