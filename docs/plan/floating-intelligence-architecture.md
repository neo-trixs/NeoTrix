# Floating Intelligence Bar — Architecture

> **Version**: 2026-09-16
> **Status**: Design
> **Convergence**: Tauri v2 desktop app (`ai.neotrix.desktop`)
> **Layer**: L2 Perception (nt_world) + L5 Cognition (nt_core/nt_mind) + L1 Action (nt_io)

---

## 1. Overview

### What Is It

The Floating Intelligence Bar (FIB) is a system-wide overlay that monitors what the user types in any application and provides AI-powered insights, knowledge retrieval, and smart suggestions — without stealing keyboard focus.

### What It Is NOT

- **Not an IME** — it never intercepts keystrokes or modifies input. Text flows normally to the target application.
- **Not a clipboard monitor** — it hooks at the accessibility layer, not the clipboard.
- **Not a browser extension** — it operates at the OS level, works in any application.
- **Not a chat window** — it surfaces inline ghost text and contextual actions, not conversational UI.

### Why This Exists

Developers constantly context-switch between their editor and knowledge sources (docs, KB, prior sessions). FIB collapses that loop: type a function name, see its docs; type a partial query, get KB matches; write a comment, see related code — all as dimmed suggestions at cursor, accepted with Tab.

```
┌──────────────────────────────────────────────────────────┐
│                    User's Application                     │
│                                                          │
│   fn calculate_total(items: &[Item]) -> f64 {           │
│       // cursor here → FIB shows:                        │
│       // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒   │
│       // ▒ sum of item.price * item.qty with tax ▒       │
│       // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒   │
│       items.iter()                                       │
│   }                                                     │
└──────────────────────────────────────────────────────────┘
```

---

## 2. System Architecture

### 2.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        NeoTrix Desktop (Tauri v2)                    │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  Main Window  │  │ Overlay Window│  │    Background Service    │  │
│  │  (chat/UI)    │  │ (FIB panel)  │  │  ┌────────────────────┐ │  │
│  │              │  │ transparent  │  │  │  Text Monitor Layer  │ │  │
│  │              │  │ alwaysOnTop  │  │  │  ┌────────────────┐  │ │  │
│  │              │  │ click-through│  │  │  │ AXObserver (mac)│  │ │  │
│  │              │  │              │  │  │  │ UIA (win)       │  │ │  │
│  └──────┬───────┘  └──────┬───────┘  │  │  │ AT-SPI (linux) │  │ │  │
│         │                  │          │  │  └───────┬────────┘  │ │  │
│         │                  │          │  │          │            │ │  │
│         │                  │          │  └──────────┼────────────┘ │  │
│         │                  │          │             │              │  │
│         │         ┌────────┴─────────┴─────────────┴──────┐       │  │
│         │         │         Intelligence Engine            │       │  │
│         │         │  ┌──────────┐ ┌─────┐ ┌────────────┐  │       │  │
│         │         │  │ KB Search│ │ VSA │ │ LLM Stream │  │       │  │
│         │         │  │ (FTS5+   │ │Hebb.│ │ (CoTGen)   │  │       │  │
│         │         │  │ embed)   │ │Graph│ │            │  │       │  │
│         │         │  └────┬─────┘ └──┬──┘ └─────┬──────┘  │       │  │
│         │         │       └──────────┼──────────┘          │       │  │
│         │         │                  │                     │       │  │
│         │         │          ┌───────┴───────┐             │       │  │
│         │         │          │ Context Engine │             │       │  │
│         │         │          │ (prefix+suffix│             │       │  │
│         │         │          │  assembly)    │             │       │  │
│         │         │          └───────┬───────┘             │       │  │
│         │         └──────────────────┼─────────────────────┘       │  │
│         │                            │                             │  │
│         │                   ┌────────┴────────┐                    │  │
│         │                   │ Overlay Renderer │                    │  │
│         │                   │ (ghost text,     │                    │  │
│         │                   │  inline actions) │                    │  │
│         │                   └─────────────────┘                    │  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Ownership

| Component | Crate | Module Path |
|-----------|-------|-------------|
| Text Monitor (macOS) | `neotrix-tauri` (Rust backend) | `src-tauri/src/fib/monitor_mac.rs` |
| Text Monitor (Windows) | `neotrix-tauri` | `src-tauri/src/fib/monitor_win.rs` |
| Text Monitor (Linux) | `neotrix-tauri` | `src-tauri/src/fib/monitor_linux.rs` |
| Overlay Window | `neotrix-tauri` | `src-tauri/src/fib/overlay.rs` |
| Intelligence Engine | `neotrix-core` | `neotrix-core/src/l5_cognition/nt_mind/fib/` |
| KB Search | `neotrix-core` | `neotrix-core/src/l1_action/nt_memory/nt_memory_kb/nt_memory_search.rs` |
| VSA Associative Recall | `neotrix-core` | `neotrix-core/src/core/nt_core_hcube/hebbian_memory.rs` |
| Context Assembly | `neotrix-core` | `neotrix-core/src/l2_perception/nt_world/fib_context.rs` |
| Overlay Frontend | `neocodex-frontend` | `src/views/fib-overlay/` |

---

## 3. Text Monitor Layer

### 3.1 macOS — AXUIElement + AXObserver

```rust
// src-tauri/src/fib/monitor_mac.rs
use axuielement::{AXUIElement, AXObserver};

pub struct MacTextMonitor {
    observer: AXObserver,
    focused_element: Option<AXUIElement>,
    last_text: String,
    debounce_tx: tokio::sync::mpsc::UnboundedSender<TextEvent>,
}

impl MacTextMonitor {
    pub fn new(debounce_tx: tokio::sync::mpsc::UnboundedSender<TextEvent>) -> Self {
        let (observer, _) = AXObserver::new(
            /* application pid */
        ).expect("failed to create AXObserver");
        Self { observer, focused_element: None, last_text: String::new(), debounce_tx }
    }

    /// Watch for focused UI element changes and text value mutations.
    pub fn start(&mut self) {
        // 1. Get system-wide element
        let system = AXUIElement::system_wide();

        // 2. Register for focused UI element notifications
        self.observer.add_notification_callback(
            system,
            "AXFocusedUIElementChanged",
            |element, notification| {
                // Re-attach value observer to new element
            },
        );

        // 3. Register for value-changed notifications on focused element
        // When text changes → emit TextEvent { text, cursor_position }
    }

    /// Extract text from AXUIElement value attribute.
    fn get_text(&self, element: &AXUIElement) -> Option<String> {
        element.attribute("AXValue").ok().and_then(|v| v.downcast().ok())
    }

    /// Get cursor position (AX insertion point).
    fn get_cursor_position(&self, element: &AXUIElement) -> Option<usize> {
        element.attribute("AXSelectedTextRange")
            .ok()
            .and_then(|v| v.downcast::<axuielement::AXValue>().ok())
            .and_then(|v| v.range_value().ok())
            .map(|r| r.location as usize)
    }
}
```

**Key behaviors:**
- Only monitors the focused application — no background scanning of all apps
- Re-registers observer when focus changes (AXFocusedUIElementChanged)
- Extracts `AXValue` (text) and `AXSelectedTextRange` (cursor) on each mutation
- Filters out redundant events when text hasn't actually changed

### 3.2 Windows — UI Automation API

```rust
// src-tauri/src/fib/monitor_win.rs
use windows::UI::UIAutomation::{IUIAutomation, IUIAutomationElement};

pub struct WinTextMonitor {
    automation: IUIAutomation,
    cached_pattern: Option<IUIAutomationValuePattern>,
}

impl WinTextMonitor {
    /// Get focused element and listen for property change events.
    pub fn start(&mut self) {
        let automation = windows::core::create_instance::<IUIAutomation>()?;
        let focused = automation.GetFocusedElement()?;

        // Register for AutomationPropertyChangedEventHandler
        // on ValuePattern.Value property
    }
}
```

### 3.3 Linux — AT-SPI2

```rust
// src-tauri/src/fib/monitor_linux.rs
// Uses atspi crate for AT-SPI2 accessibility interface
use atspi::accessibility::{Accessible, Role};

pub struct LinuxTextMonitor {
    // AT-SPI event listener
}
```

### 3.4 Event Normalization

All platform monitors emit the same normalized event:

```rust
#[derive(Debug, Clone)]
pub struct TextEvent {
    /// The application name (e.g., "Code", "Terminal")
    pub app_name: String,
    /// The full text content of the focused element
    pub text: String,
    /// Cursor offset within the text
    pub cursor: usize,
    /// Timestamp
    pub timestamp: std::time::Instant,
    /// Whether this is a selection change (not text change)
    pub is_selection: bool,
}
```

### 3.5 MutedAppList

```rust
/// Applications excluded from monitoring (security + noise).
const MUTED_APPS: &[&str] = &[
    "Keychain Access",
    "1Password",
    "Bitwarden",
    "KeePassXC",
    "NeoTrix",  // Don't monitor ourselves
];
```

---

## 4. Overlay Window Layer

### 4.1 Tauri Configuration

Add a second window in `tauri.conf.json`:

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "NeoTrix Desktop V2",
        "decorations": false,
        "transparent": true
      },
      {
        "label": "fib-overlay",
        "title": "",
        "width": 400,
        "height": 200,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "visible": false,
        "skipTaskbar": true,
        "resizable": false,
        "focus": false,
        "url": "fib-overlay/index.html"
      }
    ]
  }
}
```

### 4.2 Window Creation (Rust Backend)

```rust
// src-tauri/src/fib/overlay.rs
use tauri::{Manager, WebviewWindowBuilder, WebviewUrl};

pub fn create_overlay_window(app: &tauri::App) -> tauri::Result<tauri::WebviewWindow> {
    let overlay = WebviewWindowBuilder::new(
        app,
        "fib-overlay",
        WebviewUrl::App("fib-overlay/index.html".into()),
    )
    .title("")
    .inner_size(400.0, 200.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .build()?;

    // macOS: make click-through by default
    #[cfg(target_os = "macos")]
    overlay.with_webview(|webview| {
        let ns_window = webview.ns_window().unwrap();
        // Set ignoreMouseEvents = true initially
        objc::msg_send![ns_window, setIgnoresMouseEvents: true];
    })?;

    Ok(overlay)
}
```

### 4.3 Click-Through Toggle

The overlay is click-through by default. When the user hovers over it or presses a modifier key (Ctrl/Cmd), mouse events pass through to the application below. To interact with suggestions, the user moves the cursor into the overlay region.

```rust
// src-tauri/src/fib/click_through.rs
use rdev::{listen, Event, EventType, Key};

/// Global mouse hook to detect when cursor enters overlay bounds.
pub fn start_mouse_hook(overlay_bounds: Rect, tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>) {
    listen(move |event| {
        match event.event_type {
            EventType::MouseMove { x, y } => {
                let inside = overlay_bounds.contains(x as f64, y as f64);
                tx.send(MouseEvent::CursorMove { x, y, inside }).ok();
            }
            EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::MetaLeft) => {
                tx.send(MouseEvent::ModifierPressed).ok();
            }
            _ => {}
        }
    }).ok();
}
```

```rust
// In overlay.rs — toggle click-through
pub fn set_click_through(window: &WebviewWindow, ignore: bool) {
    #[cfg(target_os = "macos")]
    {
        let ns_window = window.ns_window().unwrap();
        objc::msg_send![ns_window, setIgnoresMouseEvents: ignore];
    }
    #[cfg(target_os = "windows")]
    {
        // WS_EX_TRANSPARENT flag toggle
    }
}
```

### 4.4 Positioning

The overlay positions itself relative to the cursor, clamped to monitor bounds.

```rust
/// Compute overlay position relative to cursor, clamped to monitor.
pub fn compute_position(
    cursor: (f64, f64),
    overlay_size: (f64, f64),
    monitor: &Monitor,
    offset: f64,  // distance below cursor
) -> (f64, f64) {
    let (cx, cy) = cursor;
    let (ow, oh) = overlay_size;
    let mon = monitor.size();

    let mut x = cx;
    let mut y = cy + offset;

    // Clamp horizontal
    if x + ow > mon.x + mon.width {
        x = mon.x + mon.width - ow;
    }
    if x < mon.x {
        x = mon.x;
    }

    // Clamp vertical — if overflows bottom, show above cursor
    if y + oh > mon.y + mon.height {
        y = cy - oh - offset;
    }
    if y < mon.y {
        y = mon.y;
    }

    (x, y)
}
```

### 4.5 Overlay Frontend (Web View)

The overlay web view runs minimal UI — no framework, pure vanilla JS + CSS for speed.

```html
<!-- fib-overlay/index.html -->
<div id="fib-container">
  <div id="fib-ghost" class="ghost-text" hidden></div>
  <div id="fib-actions" class="inline-actions" hidden></div>
</div>
```

```css
/* fib-overlay/styles.css */
#fib-container {
  position: fixed;
  pointer-events: none;  /* click-through by default */
  font-family: var(--font-mono);
  font-size: 13px;
}

#fib-container.interactive {
  pointer-events: auto;  /* enable when modifier held */
}

.ghost-text {
  color: rgba(180, 180, 200, 0.45);
  background: rgba(30, 30, 40, 0.85);
  border: 1px solid rgba(120, 120, 180, 0.2);
  border-radius: 6px;
  padding: 6px 10px;
  backdrop-filter: blur(8px);
  max-width: 380px;
  white-space: pre-wrap;
  pointer-events: none;
}

.inline-actions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
  opacity: 0;
  transition: opacity 150ms;
}

#fib-container.interactive .inline-actions {
  opacity: 1;
  pointer-events: auto;
}

.action-chip {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(80, 80, 120, 0.6);
  color: rgba(200, 200, 220, 0.8);
  cursor: pointer;
}
```

---

## 5. Intelligence Engine

### 5.1 Pipeline Overview

```
TextEvent
    │
    ▼
┌─────────────┐    300ms debounce
│  Debouncer   │──────────────────────┐
└─────────────┘                       │
                                      ▼
                              ┌───────────────┐
                              │Context Assembly│
                              │ prefix + suffix│
                              └───────┬───────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
            │  KB Search   │  │ VSA Recall   │  │ LLM Stream   │
            │  (FTS5+embed)│  │ (HebbianGraph│  │ (CoTGenerator│
            │              │  │  diffusion)   │  │  streaming)  │
            └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
                   │                 │                 │
                   └─────────────────┼─────────────────┘
                                     ▼
                             ┌───────────────┐
                             │ Result Merger  │
                             │ (dedup + rank) │
                             └───────┬───────┘
                                     │
                                     ▼
                             ┌───────────────┐
                             │ Overlay Render │
                             │ (ghost + chips)│
                             └───────────────┘
```

### 5.2 Context Assembly

```rust
/// Context window around cursor for intelligence queries.
pub struct FibContext {
    /// Lines before cursor (default 100)
    pub prefix: String,
    /// Lines after cursor (default 50)
    pub suffix: String,
    /// Application name
    pub app: String,
    /// Language hint (detected from syntax)
    pub language: Option<String>,
    /// File path if available (e.g., from editor)
    pub file_path: Option<String>,
}

impl FibContext {
    pub fn from_text_event(event: &TextEvent) -> Self {
        let lines: Vec<&str> = event.text.lines().collect();
        let cursor_line = event.text[..event.cursor]
            .lines()
            .count()
            .saturating_sub(1);

        let prefix_start = cursor_line.saturating_sub(100);
        let suffix_end = std::cmp::min(cursor_line + 50, lines.len());

        let prefix = lines[prefix_start..cursor_line].join("\n");
        let suffix = lines[cursor_line..suffix_end].join("\n");

        Self {
            prefix,
            suffix,
            app: event.app_name.clone(),
            language: detect_language(&event.app_name),
            file_path: None,
        }
    }
}

/// Detect likely programming language from application name.
fn detect_language(app: &str) -> Option<String> {
    match app {
        "Code" | "VSCodium" => Some("auto".into()), // editor can provide
        "Terminal" | "iTerm2" => Some("shell".into()),
        "RustRover" | "CLion" => Some("rust".into()),
        _ => None,
    }
}
```

### 5.3 KB Search (Hybrid Fusion)

Searches the local knowledge base using both FTS5 text matching and semantic embeddings, fusing results.

```rust
/// Hybrid KB search for FIB context.
pub fn search_kb(
    conn: &Connection,
    context: &FibContext,
    limit: usize,
) -> Vec<KbSearchResult> {
    // 1. Extract query terms from prefix
    let query = extract_query_terms(&context.prefix);

    // 2. FTS5 search (fast, BM25-ranked)
    let fts_results = search_fts(conn, &query, limit * 2)?;

    // 3. Semantic search (embedding similarity)
    let embed_results = search_embeddings(conn, &context.prefix, limit * 2)?;

    // 4. Reciprocal Rank Fusion
    let fused = rrf_fusion(&fts_results, &embed_results, k = 60);

    // 5. Return top-K
    fused.into_iter().take(limit).collect()
}

/// Extract meaningful query terms from context prefix.
/// Strategy: take last N tokens, filter stopwords, join.
fn extract_query_terms(prefix: &str) -> String {
    let words: Vec<&str> = prefix.split_whitespace().rev().take(20).collect();
    words.iter().rev().copied().collect::<Vec<_>>().join(" ")
}

/// Reciprocal Rank Fusion of two result lists.
fn rrf_fusion(
    a: &[SearchResult],
    b: &[SearchResult],
    k: u32,
) -> Vec<SearchResult> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut items: HashMap<String, SearchResult> = HashMap::new();

    for (rank, item) in a.iter().enumerate() {
        *scores.entry(item.id.clone()).or_default() += 1.0 / (k as f64 + rank as f64 + 1.0);
        items.insert(item.id.clone(), item.clone());
    }
    for (rank, item) in b.iter().enumerate() {
        *scores.entry(item.id.clone()).or_default() += 1.0 / (k as f64 + rank as f64 + 1.0);
        items.insert(item.id.clone(), item.clone());
    }

    let mut scored: Vec<_> = scores.into_iter().collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored.into_iter()
        .filter_map(|(id, _)| items.remove(&id))
        .collect()
}
```

### 5.4 VSA Associative Recall (HebbianGraph)

Uses the existing `HebbianGraph` from `nt_core_hcube::hebbian_memory` for associative recall — finding related concepts that don't match the literal query.

```rust
use crate::core::nt_core_hcube::hebbian_memory::{HebbianGraph, HebbianRetrievalResult};

/// Associative recall: given context keywords, spread activation through
/// the Hebbian graph to find related concepts.
pub fn vsa_recall(
    graph: &HebbianGraph,
    context_keywords: &[String],
    hops: usize,
    top_k: usize,
) -> Vec<VsaResult> {
    let mut all_results: Vec<(String, f64)> = Vec::new();

    for keyword in context_keywords {
        let result: HebbianRetrievalResult = graph.diffusion_retrieve(keyword, hops);
        all_results.extend(result.results);
    }

    // Merge by symbol, sum scores
    let mut merged: HashMap<String, f64> = HashMap::new();
    for (symbol, score) in all_results {
        *merged.entry(symbol).or_default() += score;
    }

    let mut ranked: Vec<_> = merged.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    ranked.into_iter()
        .take(top_k)
        .map(|(symbol, score)| VsaResult { symbol, score })
        .collect()
}
```

### 5.5 LLM Streaming Completion

When KB + VSA results are insufficient or the user is writing novel content, the LLM generates suggestions.

```rust
/// LLM suggestion request.
pub struct FibLlmRequest {
    pub prefix: String,
    pub suffix: String,
    pub kb_context: Vec<KbSearchResult>,  // top-3 KB matches for grounding
    pub vsa_context: Vec<VsaResult>,       // top-3 VSA associations
    pub task: FibTask,
}

pub enum FibTask {
    /// Complete the current line/code
    Autocomplete,
    /// Explain what the user is writing about
    Explain,
    /// Suggest the next logical step
    NextStep,
    /// Find errors in the current code
    Diagnose,
}

/// Stream LLM tokens to overlay via Tauri IPC.
pub async fn stream_llm_suggestion(
    request: FibLlmRequest,
    tx: &tokio::sync::mpsc::UnboundedSender<OverlayUpdate>,
) {
    // Build prompt with context
    let prompt = build_fib_prompt(&request);

    // Stream via existing CoTGenerator / provider
    let mut stream = crate::l5_cognition::nt_core_cot::CoTGenerator::stream_completion(
        &prompt,
        /* max_tokens: 200 */,
    ).await;

    let mut buffer = String::new();
    while let Some(token) = stream.next().await {
        buffer.push_str(&token);
        tx.send(OverlayUpdate::GhostTextUpdate {
            text: buffer.clone(),
            confidence: estimate_confidence(&request, &buffer),
        }).ok();
    }
}
```

### 5.6 Prompt Construction

```rust
fn build_fib_prompt(request: &FibLlmRequest) -> String {
    let mut prompt = String::new();

    // System instruction
    prompt.push_str("You are an inline coding assistant. ");
    prompt.push_str("Complete or explain the code at the cursor position. ");
    prompt.push_str("Output ONLY the suggested text, no explanation.\n\n");

    // KB context (grounding)
    if !request.kb_context.is_empty() {
        prompt.push_str("## Relevant Knowledge\n");
        for kb in &request.kb_context {
            prompt.push_str(&format!("- {}: {}\n", kb.title, kb.summary));
        }
        prompt.push('\n');
    }

    // VSA context (associations)
    if !request.vsa_context.is_empty() {
        prompt.push_str("## Related Concepts\n");
        for vsa in &request.vsa_context {
            prompt.push_str(&format!("- {}\n", vsa.symbol));
        }
        prompt.push('\n');
    }

    // Code context
    prompt.push_str(&format!("## Code Before Cursor\n```\n{}\n```\n", request.prefix));
    prompt.push_str(&format!("## Code After Cursor\n```\n{}\n```\n", request.suffix));

    // Task-specific instruction
    match request.task {
        FibTask::Autocomplete => prompt.push_str("Complete the next line: "),
        FibTask::Explain => prompt.push_str("Briefly explain what this code does: "),
        FibTask::NextStep => prompt.push_str("What should be implemented next? "),
        FibTask::Diagnose => prompt.push_str("Identify issues in this code: "),
    }

    prompt
}
```

---

## 6. UX Patterns

### 6.1 Ghost Text

The primary interaction pattern — a dimmed suggestion at the cursor position.

```
┌─────────────────────────────────────────────────────────┐
│ fn calculate_total(items: &[Item]) -> f64 {            │
│     let tax_rate = 0.08;                               │
│     // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│     // ▒ items.iter().map(|i| i.price * i.qty) ▒       │
│     // ▒      .sum::<f64>() * (1.0 + tax_rate) ▒       │
│     // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│     items.iter()  ← actual cursor                      │
│ }                                                      │
└─────────────────────────────────────────────────────────┘
```

**Interactions:**
| Input | Action |
|-------|--------|
| `Tab` | Accept suggestion, insert text |
| `Esc` | Dismiss suggestion |
| `↓` / `↑` | Cycle through alternative suggestions (if multiple) |
| Continue typing | Suggestion updates or fades out |
| Modifier (Ctrl/Cmd) held | Overlay becomes interactive, mouse events pass through |

### 6.2 Inline Actions

Small chips below the ghost text for secondary actions.

```
┌─────────────────────────────────────────────────────────┐
│ // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│ // ▒ items.iter().map(|i| i.price * i.qty) ▒           │
│ // ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│ [Docs] [Explain] [Tests]  ← chips (visible on hover)   │
└─────────────────────────────────────────────────────────┘
```

| Chip | Action |
|------|--------|
| `Docs` | Search KB for documentation related to the symbol under cursor |
| `Explain` | Generate natural language explanation of the code |
| `Tests` | Generate unit test suggestions |
| `Refactor` | Suggest refactoring improvements |

### 6.3 Selection-Anchored Suggestions

When the user selects text, FIB offers context-aware actions anchored to the selection.

```
┌─────────────────────────────────────────────────────────┐
│     let result = calculate_total(&items);  ◄── selected │
│ ┌──────────────────────────────────────┐                │
│ │ ▸ Find similar functions in KB       │                │
│ │ ▸ Generate docstring                 │                │
│ │ ▸ Check for edge cases               │                │
│ │ ▸ Suggest type annotations           │                │
│ └──────────────────────────────────────┘                │
└─────────────────────────────────────────────────────────┘
```

### 6.4 Dismissal Rules

The overlay auto-hides when:
1. User switches applications
2. User moves cursor away from the active region (>500ms)
3. 10 seconds pass with no interaction
4. User types more than 3 characters without accepting
5. The target application is in the `MUTED_APPS` list

---

## 7. Data Flow

### 7.1 Keystroke-to-Suggestion Pipeline

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Keystroke │────▶│ Monitor  │────▶│ Debounce │────▶│ Context  │
│ (AX/UIA)  │     │ (norm.)  │     │ (300ms)  │     │ Assembly │
└──────────┘     └──────────┘     └──────────┘     └─────┬────┘
                                                         │
                                                         ▼
                          ┌──────────────────────────────────┐
                          │        Parallel Fan-out           │
                          │                                  │
                          │  ┌────────┐ ┌──────┐ ┌────────┐ │
                          │  │KB Search│ │ VSA  │ │  LLM   │ │
                          │  │  <50ms │ │<30ms │ │<500ms  │ │
                          │  └───┬────┘ └──┬───┘ └───┬────┘ │
                          │      │         │         │      │
                          └──────┼─────────┼─────────┼──────┘
                                 │         │         │
                                 ▼         ▼         ▼
                          ┌──────────────────────────────────┐
                          │     Merge + Dedup + Rank         │
                          │     (KB+VSA fast, LLM streaming) │
                          └──────────────┬───────────────────┘
                                         │
                                         ▼
                          ┌──────────────────────────────────┐
                          │     Overlay Render (Tauri IPC)   │
                          │     Ghost text + action chips    │
                          └──────────────────────────────────┘
```

### 7.2 Timing Budget

```
Keystroke detected          0ms
Monitor normalizes         ~5ms
Debounce wait            +300ms
Context assembly          +10ms
KB search (FTS5)          +30ms  ← first result ready
VSA recall               +20ms  ← first result ready
Merge + render            +5ms
─────────────────────────────────
KB+VSA suggestion         ~350ms  (ghost text appears)

LLM first token         +200-500ms  (streaming, updates ghost text)
LLM complete            +500-2000ms (final suggestion)
```

### 7.3 IPC Messages (Rust ↔ Overlay)

```rust
/// Messages from backend to overlay window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OverlayUpdate {
    /// Ghost text content update
    GhostTextUpdate {
        text: String,
        confidence: f64,
    },
    /// Alternative suggestions available
    Alternatives {
        suggestions: Vec<Suggestion>,
    },
    /// Inline action chips
    Actions {
        actions: Vec<InlineAction>,
    },
    /// Hide overlay
    Hide,
    /// Show overlay at position
    Show { x: f64, y: f64 },
    /// Position update (cursor moved)
    Reposition { x: f64, y: f64 },
}

/// Messages from overlay to backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OverlayCommand {
    /// User accepted suggestion (Tab)
    Accept { suggestion_id: String },
    /// User dismissed (Esc)
    Dismiss,
    /// User selected alternative (↑↓)
    SelectAlternative { index: usize },
    /// User clicked action chip
    ActionClick { action: String },
    /// Request more alternatives
    LoadMore,
}
```

---

## 8. Privacy Architecture

### 8.1 Principles

1. **Local-only by default** — all processing happens on-device; no text leaves the machine unless the user explicitly enables cloud LLM
2. **Muted apps never monitored** — password managers, keychain, and NeoTrix itself are excluded
3. **No persistent text storage** — keystroke text is held in memory only, discarded after processing
4. **KB data is user-owned** — the knowledge base is a local SQLite database; no telemetry
5. **Transparent processing** — users can see exactly what data flows through FIB via a debug panel

### 8.2 MutedEventBus

```rust
/// Event bus that filters sensitive events before they reach the pipeline.
pub struct MutedEventBus {
    muted_apps: HashSet<String>,
    muted_patterns: Vec<Regex>,
    event_log: Vec<FilteredEvent>,  // audit trail (in-memory only)
}

impl MutedEventBus {
    pub fn process(&mut self, event: TextEvent) -> Option<TextEvent> {
        // 1. Check app name against mute list
        if self.muted_apps.contains(&event.app_name) {
            self.event_log.push(FilteredEvent {
                event: event.clone(),
                reason: FilterReason::MutedApp,
            });
            return None;
        }

        // 2. Check text content against sensitive patterns
        // (passwords, API keys, tokens)
        if self.matches_sensitive_pattern(&event.text) {
            self.event_log.push(FilteredEvent {
                event: event.clone(),
                reason: FilterReason::SensitiveContent,
            });
            return None;
        }

        // 3. Pass through
        Some(event)
    }

    fn matches_sensitive_pattern(&self, text: &str) -> bool {
        self.muted_patterns.iter().any(|p| p.is_match(text))
    }
}

/// Sensitive content patterns (never processed by FIB).
const SENSITIVE_PATTERNS: &[&str] = &[
    r"(?i)password\s*[:=]\s*\S+",
    r"(?i)api[_-]?key\s*[:=]\s*\S+",
    r"(?i)secret\s*[:=]\s*\S+",
    r"(?i)token\s*[:=]\s*\S+",
    r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
    r"sk-[a-zA-Z0-9]{20,}",      // OpenAI-style keys
    r"ghp_[a-zA-Z0-9]{36}",      // GitHub tokens
    r"xox[bpas]-[a-zA-Z0-9-]+",  // Slack tokens
];
```

### 8.3 Local-Only Mode

When the user disables cloud LLM (default):

```
┌─────────────────────────────────────────────────────┐
│                    FIB Pipeline                      │
│                                                     │
│  Text ──▶ KB Search (local SQLite) ──┐              │
│       ──▶ VSA Recall (in-memory)  ──┼──▶ Overlay   │
│       ──▶ Local LLM (optional)   ──┘              │
│                                                     │
│  ✗ No network requests                              │
│  ✗ No telemetry                                     │
│  ✗ No cloud API calls                               │
│  ✓ All processing on-device                         │
└─────────────────────────────────────────────────────┘
```

When cloud LLM is enabled (opt-in):

```
┌─────────────────────────────────────────────────────┐
│                    FIB Pipeline                      │
│                                                     │
│  Text ──▶ KB Search (local) ──┐                     │
│       ──▶ VSA Recall (local)──┼──▶ Merge ──┐        │
│       ──▶ Cloud LLM (opt-in)──┘            ├──▶ Overlay│
│                                            │        │
│  ⚠ Network request to configured provider          │
│  ⚠ Text sent (redacted: no passwords/keys)         │
│  ✓ User explicitly opted in                         │
└─────────────────────────────────────────────────────┘
```

### 8.4 Data Retention

| Data | Storage | Lifetime |
|------|---------|----------|
| Keystroke text | Memory (heap) | Until pipeline completes (~500ms) |
| KB search results | Memory | Until overlay dismissed |
| LLM suggestions | Memory | Until replaced or dismissed |
| Event filter log | Memory | Current session only |
| User KB entries | Local SQLite (`~/.neotrix/knowledge.db`) | Indefinite (user-managed) |
| HebbianGraph edges | Memory (serialized to KB) | Decays with `decay_factor` |

---

## 9. Performance Budget

### 9.1 Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Keystroke → first suggestion** | <350ms | End-to-end from AX notification to ghost text visible |
| **KB search latency** | <50ms | FTS5 query execution |
| **VSA recall latency** | <30ms | HebbianGraph diffusion (3 hops, top-10) |
| **LLM TTFT (time to first token)** | <500ms | From request to first streaming token |
| **Overlay render** | <16ms | One frame at 60fps |
| **Memory overhead** | <50MB | FIB process + overlay webview |
| **CPU (idle, no suggestions)** | <1% | Background monitoring only |
| **CPU (active, suggesting)** | <10% | During LLM streaming |

### 9.2 Optimization Strategies

**Debounce tuning:**
- 300ms default — balances responsiveness vs. noise
- 150ms for fast typers (detected via inter-keystroke interval)
- 500ms for slow typers (avoids firing during pauses)

**Speculative pre-fetch:**
```rust
/// When user pauses (no keystroke for 200ms), pre-fetch KB results
/// so they're ready when the 300ms debounce fires.
fn on_partial_keystroke(context: &FibContext) {
    // Start KB search in background
    // If debounce fires with same context → use cached results
    // If context changes → discard pre-fetch
}
```

**Result caching:**
```rust
/// Cache recent search results to avoid redundant queries.
struct FibCache {
    /// LRU cache: context hash → search results
    cache: lru::LruCache<u64, CachedResults>,
    /// Max entries
    capacity: usize,
}

impl FibCache {
    fn get_or_compute(&mut self, context: &FibContext, compute: impl FnOnce() -> Vec<SearchResult>) -> Vec<SearchResult> {
        let hash = xxhash_rust::xxh3::xxh3(&context.prefix.as_bytes());
        if let Some(cached) = self.cache.get(&hash) {
            return cached.results.clone();
        }
        let results = compute();
        self.cache.put(hash, CachedResults { results: results.clone() });
        results
    }
}
```

**Progressive rendering:**
1. KB results appear first (<50ms) — show immediately
2. VSA associations appear next (<80ms) — append
3. LLM streams in — update ghost text incrementally
4. No blocking: overlay renders at each stage

---

## 10. Implementation Phases

### Phase 1: Foundation (Weeks 1-3)

**Goal:** macOS text monitoring + basic overlay + KB search

- [ ] Create `src-tauri/src/fib/` module structure
- [ ] Implement `MacTextMonitor` with AXObserver
- [ ] Create overlay Tauri window (transparent, always-on-top)
- [ ] Basic ghost text rendering (no interactivity)
- [ ] Wire up debounce + context assembly
- [ ] Connect to existing KB search (`nt_memory_search::search_fts`)
- [ ] IPC bridge (backend ↔ overlay)
- [ ] MutedAppList + sensitive content filter

**Deliverable:** User types in VS Code, sees ghost text suggestions from local KB.

### Phase 2: Intelligence (Weeks 4-6)

**Goal:** VSA recall + LLM streaming + keyboard interaction

- [ ] Integrate `HebbianGraph::diffusion_retrieve` for associative recall
- [ ] LLM streaming via `CoTGenerator` with prompt construction
- [ ] Tab to accept / Esc to dismiss
- [ ] Multiple suggestion cycling (↑↓)
- [ ] Result fusion (KB + VSA + LLM)
- [ ] Confidence scoring and suggestion ranking
- [ ] Performance profiling and optimization

**Deliverable:** User gets multi-source suggestions with keyboard interaction.

### Phase 3: Rich UX (Weeks 7-9)

**Goal:** Inline actions, selection-anchored suggestions, click-through

- [ ] Inline action chips (Docs, Explain, Tests, Refactor)
- [ ] Selection-anchored context menu
- [ ] Click-through toggle (rdev mouse hook)
- [ ] Cursor-relative positioning with monitor clamping
- [ ] Reposition on cursor movement
- [ ] Auto-dismiss rules (app switch, timeout, typing)
- [ ] Visual polish: blur, fade-in/out, hover effects

**Deliverable:** Full interaction model with mouse and keyboard.

### Phase 4: Cross-Platform + Polish (Weeks 10-12)

**Goal:** Windows/Linux support, edge cases, performance hardening

- [ ] Windows `WinTextMonitor` via UI Automation
- [ ] Linux `LinuxTextMonitor` via AT-SPI2
- [ ] Multi-monitor support
- [ ] HiDPI/Retina rendering
- [ ] Dark mode / theme integration
- [ ] Debug panel (show FIB pipeline state)
- [ ] Performance benchmarks (<350ms target validation)
- [ ] Memory leak testing (24-hour soak test)
- [ ] Accessibility: voice-over compatible overlay

**Deliverable:** Production-ready FIB on all three platforms.

---

## Appendix A: Dependency Map

```
neotrix-core (existing)
├── nt_core_hcube::hebbian_memory    → VSA associative recall
├── nt_memory_kb::nt_memory_search   → KB FTS5 + embedding search
├── nt_core_gwt                      → Attention routing (salience)
├── nt_core_cot::CoTGenerator        → LLM streaming
└── nt_core_math::cosine_similarity  → Embedding similarity

neotrix-tauri (new: fib/ module)
├── fib::monitor_mac                 → AXUIElement + AXObserver
├── fib::monitor_win                 → UI Automation API
├── fib::monitor_linux               → AT-SPI2
├── fib::overlay                     → Tauri WebviewWindow
├── fib::click_through               → rdev global mouse hook
├── fib::positioning                 → Monitor-aware clamping
└── fib::ipc                         → OverlayUpdate / OverlayCommand

neocodex-frontend (new: fib-overlay/)
├── index.html                       → Minimal overlay shell
├── styles.css                       → Ghost text + action chips
└── overlay.js                       → IPC handler + DOM updates
```

## Appendix B: References

- `neotrix-core/src/core/nt_core_hcube/hebbian_memory.rs` — HebbianGraph implementation
- `neotrix-core/src/l1_action/nt_memory/nt_memory_kb/nt_memory_search.rs` — FTS5 + semantic search
- `neotrix-core/src/core/nt_core_gwt/` — Global Workspace Theory attention routing
- `src-tauri/tauri.conf.json` — Existing Tauri window configuration
- `docs/2-PLANS/2026-07-02-ui-architecture-specification.md` — UI architecture conventions
