# NeoTrix UI Absorption & Development Plan

**Date**: 2026-07-03
**Sources**: NovaChat Design Spec + Osaurus GitHub README

## 1. NovaChat Function Absorption

| # | Feature | Status | Priority | Files to Change |
|---|---------|--------|----------|-----------------|
| 1 | Session time grouping (Today/Yesterday/7d/Earlier) | ❌ | P0 | `sessionSlice.ts` + `SessionList.tsx` |
| 2 | Session search (type filter) | ⚠️ | P0 | `SearchOverlay.tsx` + `SessionList.tsx` |
| 3 | Pinned sessions | ❌ | P0 | `sessionSlice.ts` add `pinned` field |
| 4 | Model selector in TopBar | ❌ | P0 | `App.tsx` (new `ModelSelector.tsx`) |
| 5 | Stop generation button (→ ■) | ❌ | P0 | `InputPanel.tsx` |
| 6 | Streaming loading dots (pre-first-token) | ❌ | P1 | `ChatPanel.tsx` |
| 7 | Scroll-to-bottom button | ❌ | P1 | `ChatPanel.tsx` |
| 8 | Message edit + resend | ❌ | P1 | `ChatPanel.tsx` + store |
| 9 | Auto-resizing textarea (1-10 rows) | ❌ | P1 | `InputPanel.tsx` |
| 10 | Token count in input | ❌ | P1 | `InputPanel.tsx` |
| 11 | Sidebar icon mode (64px collapsed) | ❌ | P1 | `SessionList.tsx` + CSS |
| 12 | Attachment hover preview | ❌ | P2 | `InputPanel.tsx` |
| 13 | Export/import from TopBar menu | ❌ | P1 | `App.tsx` (new `TopBar.tsx`) |
| 14 | API key mask + keychain | ⚠️ | P1 | `Settings.tsx` + Rust `keyring` |
| 15 | Streaming throttle (16ms frame) | ❌ | P1 | `ChatPanel.tsx` |
| 16 | Artifact panel (code/doc preview) | ❌ | P2 | new `ArtifactPanel.tsx` |

## 2. Osaurus Feature Absorption

| # | Feature | Status | Priority | Files to Change |
|---|---------|--------|----------|-----------------|
| 17 | MCP tool registry UI | ⚠️ | P2 | new `ToolPanel.tsx` |
| 18 | Privacy filter UI (PII mask) | ⚠️ | P2 | `PrivacyFilterPage.tsx` |
| 19 | Sandbox manager UI | ✅ | - | `SandboxManagerPage.tsx` |
| 20 | Voice input | ⚠️ | P2 | `InputPanel.tsx` + Rust |
| 21 | Skills management UI | ⚠️ | P2 | new `SkillsPanel.tsx` |
| 22 | Identity (crypto) UI | ✅ | - | `IdentityManagerPage.tsx` |
| 23 | Right Panel: Files / Agent / Artifact | ❌ | P2 | `RightPanel.tsx` |
| 24 | Customizable keyboard shortcuts | ❌ | P2 | `ShortcutsPanel.tsx` |
| 25 | Automation (schedules/watchers) | ❌ | P3 | new `AutomationPage.tsx` |
| 26 | DevTools panel (MCP inspect / E8 trace) | ❌ | P3 | new `DevToolsPage.tsx` |

## 3. Phased Implementation

### Phase 1 — Chat UX Upgrade (P0)
**Goal**: Production-grade chat experience

| Task | Est. | Detail |
|------|------|--------|
| 1.1 Session time grouping | 2h | `sessionSlice.ts`: add `lastActive` field, `groupedSessions()` getter. `SessionList.tsx`: render groups with labels |
| 1.2 Session search | 1h | Wire `SearchOverlay` to filter `sessions` array, debounced input |
| 1.3 Pinned sessions | 1h | Add `pinned: boolean` to `Session` type; pin/unpin via context menu |
| 1.4 Model selector | 2h | New `ModelSelector.tsx` dropdown in TopBar; read `providerSlice` |
| 1.5 Stop generation | 1h | `InputPanel.tsx`: show ■ when `agentBusy`, emit cancel signal |
| 1.6 Streaming init dots | 1h | `ChatPanel.tsx`: show animated dots before first token |

### Phase 2 — Session Management (P1)
**Goal**: Multi-session at production level

| Task | Est. | Detail |
|------|------|--------|
| 2.1 Sidebar collapsed icon mode | 2h | 64px mode with icon-only sessions, tooltip on hover |
| 2.2 Message edit + resend | 2h | New `editMessage(index)` store method; inline textarea replacement |
| 2.3 Scroll-to-bottom button | 1h | IntersectionObserver on bottomRef; show floating button when hidden |
| 2.4 Auto-resize textarea | 1h | `InputPanel.tsx`: watch `scrollHeight`, clamp 1-10 rows |
| 2.5 TopBar export menu | 1h | Kebab ▾ menu with Export / Import / Clear / Delete |
| 2.6 API key mask + keychain | 3h | Rust Tauri command `store_api_key` / `get_api_key` using `keyring` crate |

### Phase 3 — Right Panel + Tools (P2)
**Goal**: Extend right panel with knowledge/tool/artifact views

| Task | Est. | Detail |
|------|------|--------|
| 3.1 Artifact panel | 4h | New tab: rendered markdown, code with syntax highlight, file preview |
| 3.2 MCP tool panel | 3h | List tools, search, invoke; real-time status per tool |
| 3.3 Privacy filter UI | 2h | PII pattern config, test panel, masking preview |
| 3.4 Skills panel | 2h | List installed skills, search, import from URL/local |
| 3.5 Streaming throttle | 2h | Buffer tokens into 16ms requestAnimationFrame batches |

### Phase 4 — Advanced (P3)
**Goal**: Automation + voice + customization

| Task | Est. | Detail |
|------|------|--------|
| 4.1 Automation panel | 6h | Cron schedules, file watchers, webhook triggers |
| 4.2 Voice input | 4h | ANE offline STT via Tauri plugin |
| 4.3 Keyboard shortcut editor | 2h | `ShortcutsPanel` editable key bindings |
| 4.4 DevTools panel | 3h | E8 state viewer, MCP call inspector, GWT resonance graph |

## 4. UI Label Inventory (Current vs Target)

```
Sidebar:
├── Header: "NeoTrix" + [+][↓][←→]                                  ✅
├── Search bar (input filters session list)                          ❌ P0
├── Time groups: Today / Yesterday / 7 days / Earlier                 ❌ P0
│   ├── Session item: [icon] name  [pinned★] [fork] [export]          ⚠️ P0
│   └── Hover: [Rename] [Pin/Unpin] [Delete] ⋮                       ⚠️ P1
└── User bar: [avatar] name ▾                                        ✅

TopBar (new):
├── Session name (editable inline)                                    ❌ P1
├── Model: [Claude/GPT/Gemini...] ▾                                   ❌ P0
├── ▾ More: Export / Import / Clear / Delete                          ❌ P1
└── ☀/☾ theme toggle / ⌘K command palette                            ✅

Chat Area:
├── WelcomeState: greeting + 4 suggestion cards                      ✅
├── Messages:
│   ├── User: right-aligned bubble, hover:[Copy][Edit↩][Delete]      ⚠️ P1
│   ├── Assistant: left-aligned, hover:[Copy][Regen][👍][👎]        ✅
│   ├── Loading dots animation (pre-first token)                      ❌ P1
│   └── Streaming: cursor ▊                                          ✅
├── [↓ Scroll to bottom] button                          ❌ P1
└── Auto-scroll on new content                                         ✅

Input Area:
├── [Attach 📎] button                                              ✅
├── Textarea: auto-resize 1-10 rows                                  ❌ P1
├── [Voice 🎤] button                                                ⚠️ P2
├── [■ Stop | ↗ Send] toggle                                         ❌ P0
├── 123/4096 tokens indicator                                        ❌ P1
└── Drag-and-drop overlay                                            ✅

Right Panel (3 tabs):
├── Evolution: SEAL pipeline, maturity badges                        ✅
├── Files: project tree                                               ✅
└── Meta: self model state                                           ✅
    NEW: [Artifact] tab (code preview, rendered doc)                  ❌ P2
    NEW: [Tools/MCP] tab (tool list, invoke)                         ❌ P2

Settings (7 tabs):
├── Provider: type/model/api key/base url/temperature                ✅
├── General: theme/font size/language                                ✅
├── API: key mask + keychain storage                                 ⚠️ P1
├── Knowledge: embedding search                                      ⚠️
├── Privacy: storage/telemetry/local-first/censor + PII               ⚠️ P2
├── Shortcuts: display → editable                                   ❌ P2
└── About                                                             ✅

Status Bar:
├── Ready | Thinking [dot]   │ ䷀ Grounding  │ GWT 3/6  │ SEAL L3  │ 1/5   ✅
```

## 5. Architecture Notes

- **Model selector**: Reuse `providerSlice.providerConfig`; list models from `ProviderConfig.model` field. Group by provider type. Add "Open Settings" link at bottom.
- **Pinned sessions**: Add `pinned: boolean` to `Session` type. Sort pinned first in each time group. Star icon in session item.
- **Stop generation**: Use `streamingSlice.cancel()` signal. The InputPanel's send button morphs to a stop button (■) when `agentBusy === true`.
- **Scroll-to-bottom**: IntersectionObserver on scroll container's `bottomRef`. Button appears when user scrolls up >200px from bottom.
- **Token counting**: Simple `value.split(/\s+/).filter(Boolean).length` for word count; optional integration with `tiktoken-rs` via Tauri command.
- **Keychain**: Rust `keyring` crate via Tauri command `store_api_key(key)`, `get_api_key()` — replaces `localStorage` storage.
- **Voice input**: ANE offline STT via `coreml-rs` or Tauri plugin; microphone button → recording indicator → transcribed text.
- **Streaming throttling**: Buffer streaming tokens into 32ms chunks rendered via `requestAnimationFrame` to avoid layout thrashing.

## 6. Existing Components (Already Done)

| Component | File | Status |
|-----------|------|--------|
| Command Palette | `CommandPalette.tsx` | ✅ |
| Search Overlay | `SearchOverlay.tsx` | ✅ |
| Right Panel | `RightPanel.tsx` | ✅ |
| Chat Panel | `ChatPanel.tsx` | ✅ |
| Input Panel | `InputPanel.tsx` | ✅ |
| Session List | `SessionList.tsx` | ✅ |
| Settings Modal | `Settings.tsx` | ✅ (7 tabs) |
| Sandbox Manager | `SandboxManagerPage.tsx` | ✅ |
| Identity Manager | `IdentityManagerPage.tsx` | ✅ |
| Agent Manager | `AgentManager.tsx` | ✅ |
| Privacy Filter | `PrivacyFilterPage.tsx` | ✅ |
| User Popover | `UserPopover.tsx` | ✅ |
| Status Bar | `StatusBar.tsx` | ✅ |
| Command Palette | `CommandPalette.tsx` | ✅ |
| Consciousness Pet | `ConsciousPet.tsx` | ✅ |
| Provider Config | `ProviderConfig.tsx` | ✅ |
| Session List CSS | `SessionList.module.css` | ✅ |
| Chat Panel CSS | `ChatPanel.module.css` | ✅ |
| Input Panel CSS | `InputPanel.module.css` | ✅ |
