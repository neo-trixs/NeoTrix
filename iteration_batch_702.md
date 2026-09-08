# Iteration Batch 702 — GUI Framework Landscape: Tauri/egui/Iced (2026-09-06)

## Context
Batch 701 confirmed async-std dead (RUSTSEC-2025-0052), async trait not dyn-safe, Send bounds don't propagate, cancellation data loss at .await, async drop not stabilized. This batch audits 2026 state of desktop GUI frameworks for NeoTrix NT-IO layer.

---

## 1. TAURI 2.x (v2.11.5, Jul 1 2026)

### NEW Defects Found

| ID | Severity | Defect | Source |
|----|----------|--------|--------|
| T-D1 | **CRITICAL** | **CVE-2026-42184 — Origin Confusion**: `is_local_url()` on Windows/Android uses `split_once('.')` which only checks first subdomain. Attacker hosting `http://app.evil.com/` bypasses local origin check, invokes IPC commands restricted to local frontend. Fixed in 2.11.1 but affects 2.0–2.11.0. | [GHSA-7gmj-67g7-phm9](https://github.com/tauri-apps/tauri/security/advisories/GHSA-7gmj-67g7-phm9), [CVE-2026-42184](https://nvd.nist.gov/vuln/detail/CVE-2026-42184) |
| T-D2 | **HIGH** | **RUSTSEC-2026-0098 — rustls-webpki URI name constraints ignored**: Transitive dependency `rustls-webpki` 0.102.8 incorrectly accepts URI name constraints. Tauri's TLS stack affected. Patched in `>=0.103.12`. | [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098.html), [tauri#15244](https://github.com/tauri-apps/tauri/issues/15244) |
| T-D3 | **MEDIUM** | **Cold start latency uncontrolled**: OS webview (WKWebView/WebView2) initialization cost not under Tauri's control. Windows WebView2 first launch noticeably slower due to runtime caching. Not instant despite ~0.3s average. | [rustify.rs tutorial](https://rustify.rs/articles/rust-tauri-v2-desktop-app-tutorial-2026) |
| T-D4 | **MEDIUM** | **IPC boundary design fragility**: Deciding Rust↔JS logic split is the hardest architectural decision. Over-chatty `invoke()` calls or bloated Rust layer both cause problems. No guidance or constraints enforced. | [rustify.rs tutorial](https://rustify.rs/articles/rust-tauri-v2-desktop-app-tutorial-2026) |
| T-D5 | **LOW** | **CSS inconsistency across platforms**: WKWebView (macOS) vs WebView2 (Windows) vs WebKitGTK (Linux) produce minor CSS differences. Advanced animations, grid/container queries problematic. No pixel-perfect guarantee. | [rustify.rs tutorial](https://rustify.rs/articles/rust-tauri-v2-desktop-app-tutorial-2026) |
| T-D6 | **LOW** | **Mobile support immature**: iOS/Android support functional but less mature than desktop. Production mobile apps recommended to use `flutter_rust_bridge` or UniFFI instead. | [Tauri docs](https://v2.tauri.app/) |

### Key Numbers
- Bundle: ~5 MB vs Electron 120 MB+
- RAM: ~30 MB vs Electron 150 MB+
- Current version: 2.11.5 (rapid release cadence: 2.11.0→2.11.5 in 2 months)

---

## 2. EGUI (v0.36.1, Aug 7 2026)

### NEW Defects Found

| ID | Severity | Defect | Source |
|----|----------|--------|--------|
| E-D1 | **HIGH** | **Breaking API churn every release**: 0.34.0 deprecated `App::update` → `App::ui`, removed `Context` as main entrypoint. 0.35.0 removed `Modifiers` from `RawInput` (now an `Event`). 0.36.0 removed `clip_rect_margin`. MSRV jumped 1.88→1.92→1.95 in 3 releases. | [egui CHANGELOG](https://github.com/emilk/egui/blob/main/CHANGELOG.md) |
| E-D2 | **MEDIUM** | **O(n²) word boundary scan regression**: Fixed in 0.34.2 with regression test. Previously unbounded quadratic text layout scan. | [egui#8077](https://github.com/emilk/egui/pull/8077) |
| E-D3 | **MEDIUM** | **DragValue crash through small floats**: 0.34.0 fix for crash when dragging a DragValue through small float values. | [egui#7939](https://github.com/emilk/egui/pull/7939) |
| E-D4 | **MEDIUM** | **Grid window resize asymmetry**: Window with Grid was widenable but not shrinkable again. Fixed in 0.36.0. | [egui#8386](https://github.com/emilk/egui/pull/8386) |
| E-D5 | **LOW** | **TexturesDelta accidental drop**: Prevented accidental dropping of TexturesDelta in 0.36.0. Silent resource leak if dropped. | [egui#8356](https://github.com/emilk/egui/pull/8356) |
| E-D6 | **LOW** | **Instable IDs with animated panels**: Widget IDs unstable when panels animate open/close. Fixed in 0.34.0. | [egui#7994](https://github.com/emilk/egui/pull/7994) |

### NEW Features (Relevant to NeoTrix)

| Feature | Version | Significance |
|---------|---------|--------------|
| **egui_mcp** — MCP server for AI agents | 0.35.0 | AI can see/use/control egui apps via inspection protocol. Directly relevant to NT-IO agent interface. |
| **Atoms** layout primitives | 0.32.0 | New composable layout units for text/images. Replaces ~130 lines of Button layout math. |
| **CSS-like classes** on Ui | 0.35.0 | Context-dependent widget behavior modification. Step toward CSS-like styling. |
| **skrifa + vello_cpu** font rendering | 0.34.0 | Replaced ab_glyph. Enables font hinting, variations, color emoji support. |
| **Mobile keyboard** (autocomplete/IME) | 0.36.0 | iOS/Android keyboard now functional for eframe web. |
| **BoxedWidget** (dyn dispatch) | 0.36.0 | Dynamically dispatched widgets. First dyn-safe widget type. |
| **Drag-to-open panels** | 0.36.0 | UX improvement for collapsible panels. |

### Key Numbers
- Current version: 0.36.1 (Aug 7 2026)
- Downloads: 20M+ total, 4.5M recent
- MSRV: 1.95 (as of 0.36.0)
- Rendering backend: egui-wgpu (default since 0.34, was glow)

---

## 3. ICED (v0.14.0, Dec 7 2025)

### NEW Defects Found

| ID | Severity | Defect | Source |
|----|----------|--------|--------|
| I-D1 | **HIGH** | **Image flicker/disappear regression**: `Handle::from_bytes` images flicker/disappear when subscriptions active. New in 0.14 (not in 0.13.1). Internal texture cache drops/recreates GPU texture on each `view()` call. Must store Handle in state. | [iced#3160](https://github.com/iced-rs/iced/issues/3160), [iced#3190](https://github.com/iced-rs/iced/issues/3190) |
| I-D2 | **HIGH** | **tiny-skia pixel ghosting**: Rapid scrolling in pick_list overlay breaks clipping bounds. Dirty region (Scissor Rect) calculation fails to catch up with high-frequency scrolling. Leaked pixels pollute underlying widgets permanently. wgpu backend unaffected. | [iced#3351](https://github.com/iced-rs/iced/issues/3351) |
| I-D3 | **HIGH** | **Scrollable state leakage**: Scroll state preservation uses widget type + position in tree, NOT widget ID. Two different scrollables at same tree position share state. Tab switching causes scroll position cross-contamination. | [iced#3345](https://github.com/iced-rs/iced/issues/3345) |
| I-D4 | **MEDIUM** | **pane_grid event capture asymmetry**: `shell.capture_event()` called on ButtonPressed but NOT ButtonReleased. Underlying widgets can't receive mouse-down events but can receive mouse-up. | [iced#3175](https://github.com/iced-rs/iced/issues/3175) |
| I-D5 | **MEDIUM** | **Text overflows bounds with Wrapping::None**: Text draws past its bounds, overlapping sibling widgets. No truncation or ellipsis. Regression from 0.13. | [iced#3150](https://github.com/iced-rs/iced/issues/3150) |
| I-D6 | **LOW** | **No Android support**: Still on roadmap, not available. | [iced.rs](https://iced.rs) |

### NEW Features (Relevant to NeoTrix)

| Feature | Version | Significance |
|---------|---------|--------------|
| **Input method support** | 0.14 | Non-Latin language text input. Global app requirement. |
| **20% WebGPU rendering improvement** | 0.14 | Smarter caching, optimized draw calls. |
| **Time-travel debugging** | 0.14 | `iced::application::timed` — state at time T is function of initial state + all Messages. Deterministic replay. |
| **Daemon mode** | 0.14 | Background apps without windows. Useful for NT-IO headless services. |
| **Sipper** (async streams) | 0.14 | `iced_runtime::sipper` for streaming async actions. |

### Key Numbers
- Current version: 0.14.0 (Dec 7 2025 — stale, 9 months without release)
- Downloads: 2.4M total, 609K recent
- Renderers: wgpu (Vulkan/Metal/DX12) + tiny-skia (software fallback)

---

## Comparative Matrix (Batch 702)

| Dimension | Tauri 2.x | egui 0.36 | Iced 0.14 |
|-----------|-----------|-----------|-----------|
| **Last release** | Jul 1 2026 | Aug 7 2026 | Dec 7 2025 |
| **Critical CVE** | CVE-2026-42184 (origin confusion) | None found | None found |
| **API stability** | Moderate (plugin system) | Poor (breaking every release) | Good (fewer breaking changes) |
| **Mobile support** | iOS+Android (immature) | eframe web (keyboard works) | Not available |
| **AI/Agent integration** | None native | egui_mcp (MCP server) | None native |
| **Async model** | JS↔Rust IPC | Immediate mode (no async) | Elm Architecture + Task/Subscription |
| **Dyn safety** | N/A | BoxedWidget (0.36.0) | Not addressed |
| **Font rendering** | OS webview dependent | skrifa+vello_cpu (hinting) | System fonts |
| **Render backend** | OS webview (WKWebView/WebView2) | egui-wgpu (default) | wgpu + tiny-skia |

---

## Defects Summary by NeoTrix Relevance

### Must-Fix Before NT-IO Integration
1. **T-D1**: Tauri origin confusion — affects any app using custom protocols on Windows/Android
2. **I-D1**: Iced image flicker — breaks visual asset display in any subscription-active app
3. **I-D3**: Iced scroll state leakage — breaks tabbed interfaces with scrollable content

### Architecture Implications
1. **egui MCP inspection** (E-feature): Directly enables NT-IO agent-to-GUI interaction. Strong candidate for NT-IO desktop interface.
2. **egui API churn** (E-D1): Every major version breaks `App::update`/`App::ui`. NeoTrix integration would need version pinning strategy.
3. **Tauri IPC boundary** (T-D4): The Rust↔JS split decision is underspecified. NeoTrix consciousness architecture needs clear boundary contracts.
4. **Iced time-travel debugging** (I-feature): Deterministic state replay aligns with NeoTrix's SEAL pipeline observability needs.

### Source URLs
- https://github.com/tauri-apps/tauri/security/advisories/GHSA-7gmj-67g7-phm9
- https://nvd.nist.gov/vuln/detail/CVE-2026-42184
- https://rustsec.org/advisories/RUSTSEC-2026-0098.html
- https://github.com/emilk/egui/blob/main/CHANGELOG.md
- https://github.com/emilk/egui/releases/tag/0.36.0
- https://github.com/emilk/egui/releases/tag/0.35.0
- https://github.com/emilk/egui/releases/tag/0.34.0
- https://github.com/iced-rs/iced/issues/3351
- https://github.com/iced-rs/iced/issues/3345
- https://github.com/iced-rs/iced/issues/3160
- https://github.com/iced-rs/iced/issues/3175
- https://github.com/iced-rs/iced/issues/3150
- https://blog.wybxc.cc/blog/rust-gui-survey-2026/
- https://rustify.rs/articles/rust-tauri-v2-desktop-app-tutorial-2026
