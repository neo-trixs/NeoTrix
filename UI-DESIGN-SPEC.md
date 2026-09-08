# NeoTrix Frontend Design Spec

Derived from analysis of Claude Desktop, OpenAI Codex CLI, Osaurus, Kun, Cursor, and Windsurf (Sep 2026).

---

## 1. Design Philosophy

**Warm minimalism + agent-native.** Not a cold dark-IDE clone. Not a toy chat bubble. A workspace where agents feel like co-workers, not plugins.

Core principles:
- **Warm neutrals over cold grays** — every app surveyed has shifted away from pure #000/#FFF toward warm-toned surfaces
- **Single accent voltage** — one brand color used sparingly, not rainbow UI
- **Glass/blur as depth, not decoration** — Osaurus pioneered per-window material glass; Cursor uses heavy diffuse shadows instead
- **Agent-first layout** — Claude Desktop: drag-and-drop panes. Kun: Code/Design/Write workspaces. Cursor: plan→accept/reject. Windsurf: Cascade front-and-center
- **Progressive disclosure** — thinking collapsed, tools compact, expand on demand

---

## 2. Color System

### 2.1 Primary Palette (Dark Theme — Default)

| Token | Hex | Role | Source Inspiration |
|-------|-----|------|-------------------|
| `canvas` | `#0E0E12` | App background, deepest layer | Codex pure black softened with blue |
| `surface-1` | `#16161D` | Sidebar, secondary panels | Osaurus `secondaryBackground` |
| `surface-2` | `#1E1E26` | Cards, elevated panels | Claude dark `bg-surface` |
| `surface-3` | `#26262F` | Hover states, input fill | Codex input box lighter |
| `border` | `#2A2A35` | Default borders, dividers | Windsurf subtle border |
| `border-strong` | `#3A3A48` | Focus rings, active borders | Claude dark `border-strong` |
| `text-primary` | `#E8E6F0` | Primary text | Claude dark `fg-base` warm |
| `text-secondary` | `#9896A3` | Secondary text, labels | Codex muted |
| `text-tertiary` | `#6B697A` | Disabled, timestamps | Claude dark `fg-subtle` |
| `accent` | `#C8783C` | Primary CTA, links | Claude coral/terra-cotta (dark variant) |
| `accent-hover` | `#D4894E` | Accent hover state | Cursor orange warm-shifted |
| `accent-subtle` | `rgba(200, 120, 60, 0.12)` | Accent tinted backgrounds | Claude `accent-light` |
| `success` | `#4ABA7A` | Success, green states | Claude dark semantic |
| `warning` | `#F5A623` | Warning states | Claude dark semantic |
| `error` | `#E05C4A` | Error, destructive actions | Claude dark semantic |
| `info` | `#5B9FE0` | Info, link highlights | Claude dark semantic |

### 2.2 Agent Timeline Pastels (from Cursor)

| Token | Hex | Agent Stage |
|-------|-----|-------------|
| `thinking` | `#DFA88F` | Thinking / reasoning |
| `searching` | `#9FC9A2` | Grep / search / explore |
| `reading` | `#9FBBE0` | File read / context loading |
| `editing` | `#C0A8DD` | Code edit / write |
| `complete` | `#C08532` | Task done |

### 2.3 Light Theme Overrides

| Token | Hex |
|-------|-----|
| `canvas` | `#F7F6F3` |
| `surface-1` | `#FFFFFF` |
| `surface-2` | `#F0EEEB` |
| `surface-3` | `#E8E6E2` |
| `border` | `#E2DFD8` |
| `text-primary` | `#1C1B19` |
| `text-secondary` | `#5A5750` |
| `accent` | `#D4763B` |
| `accent-hover` | `#C06228` |

---

## 3. Typography

### 3.1 Font Stack

```css
--font-sans: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
--font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
--font-display: 'Inter', 'SF Pro Display', system-ui, sans-serif;
```

- **Display**: Inter weight 400-500, letter-spacing -0.02em (inspired by Cursor's CursorGothic compressed feel, but using open-source Inter)
- **Body**: Inter weight 400, line-height 1.55-1.65
- **Mono**: JetBrains Mono on all code surfaces (shared by Cursor, Codex, Kun)

### 3.2 Type Scale

| Token | Size | Weight | Line Height | Letter Spacing | Use |
|-------|------|--------|-------------|----------------|-----|
| `display-xl` | 48px | 400 | 1.15 | -0.025em | Hero / splash |
| `display` | 36px | 400 | 1.2 | -0.02em | Page titles |
| `heading-1` | 28px | 500 | 1.25 | -0.015em | Section headers |
| `heading-2` | 22px | 500 | 1.3 | -0.01em | Card titles |
| `heading-3` | 18px | 500 | 1.35 | -0.005em | Subsection |
| `body-lg` | 16px | 400 | 1.6 | normal | Long-form text |
| `body` | 14px | 400 | 1.55 | normal | Default body |
| `body-sm` | 13px | 400 | 1.5 | normal | Chat messages |
| `caption` | 12px | 400 | 1.4 | normal | Labels, metadata |
| `code` | 13px | 400 | 1.5 | normal | Inline code |
| `mono` | 13px | 400 | 1.6 | normal | Code blocks |

---

## 4. Layout Architecture

### 4.1 App Shell (Desktop)

```
┌──────────────────────────────────────────────────────────────┐
│  Titlebar (52px) — traffic lights + window title + actions  │
├──────────┬───────────────────────────────────────────────────┤
│          │  Header Bar (48px) — breadcrumb + search + config│
│  Sidebar │───────────────────────────────────────────────────│
│  (260px) │                                                   │
│  collaps │  Main Content Area                               │
│  ible to │  (flex-1, scrollable)                            │
│  56px    │                                                   │
│          │  ┌─────────────────────────────────────────────┐ │
│          │  │  Chat / Editor / Canvas / Terminal          │ │
│          │  │  (tabbed or split-pane)                     │ │
│          │  │                                             │ │
│          │  │                                             │ │
│          │  └─────────────────────────────────────────────┘ │
│          │───────────────────────────────────────────────────│
│          │  Input Area (auto-height, max 200px)             │
├──────────┴───────────────────────────────────────────────────┤
│  Status Bar (28px) — context usage · model · status dot     │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 Sidebar (Osaurus/Kun Inspired)

| Element | Spec |
|---------|------|
| Width | 260px default, 56px collapsed (icon-only) |
| Background | `surface-1` with optional glass blur (macOS) |
| Sections | Navigation, Sessions, Agent status |
| Active indicator | 3px left accent bar OR subtle bg tint |
| Collapse animation | 200ms ease-out width transition |
| Search | Persistent search input at top |

### 4.3 Main Content Modes (Kun Workspaces)

| Mode | Purpose | Layout |
|------|---------|--------|
| **Chat** | Agent conversation | Message thread + input |
| **Code** | Code editor + terminal | Split: editor + terminal + preview |
| **Work** | Document editing | Split: editor + preview |
| **Design** | Visual prototype | Canvas + inspector + agent |
| **Explore** | Codebase search | Results tree + preview |

### 4.4 Status Bar (Bottom)

```
[🟢 Connected] · claude-4-opus · 42k/200k context (21%) · ↑12 ↓89 tok · ⏱ 2.3s
```

- Height: 28px
- Font: 11px caption
- Background: `surface-1`
- Status dot: animated pulse when active

---

## 5. Component Design

### 5.1 Chat Messages

| Property | Value |
|----------|-------|
| Max width | 768px (centered) |
| Bubble radius | 16px (Claude pattern) |
| User bubble | `surface-2` background |
| Assistant | No background, text on canvas |
| Avatar | 28px circle, initials or icon |
| Spacing between messages | 16px |
| Spacing between roles | 24px |
| Markdown rendering | Full support: code blocks, tables, lists, LaTeX |
| Code blocks | Monospace, `surface-3` background, 12px radius, copy button |
| Thinking | Collapsible, left-bordered with `thinking` pastel |
| Tool calls | Compact row: icon + name + duration + expand arrow |

### 4.3 Input Area (Claude/Osaurus Pattern)

```
┌─────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────┐   │
│  │  [📎] [🔧]  Message... (auto-expanding textarea)    │   │
│  │                                              [Send] │   │
│  └─────────────────────────────────────────────────────┘   │
│  Context: 45% used · Model: claude-4-opus · [↑ image]     │
└─────────────────────────────────────────────────────────────┘
```

| Element | Spec |
|---------|------|
| Container | `surface-2` background, 20px radius (pill-like), `border` 1px |
| Max height | 200px (then scroll) |
| Min height | 48px (single line) |
| Padding | 12px 16px |
| Shadow | `0 -2px 12px rgba(0,0,0,0.08)` subtle bottom glow |
| Action buttons | Left-aligned: attachment, tools, slash commands |
| Send button | Right-aligned, accent color, 32px circle, icon only |
| Status line | Below input: context %, model name, image attach button |
| Focus state | `border-strong` ring, 2px |
| Placeholder | `text-tertiary`, "Message NeoTrix..." |

### 4.4 Workspace Modes (Kun-Inspired)

NeoTrix should support three primary workspace modes, similar to Kun's Code/Design/Write:

| Mode | Purpose | Primary View |
|------|---------|-------------|
| **Terminal** | CLI, shell, commands | Terminal emulator + chat |
| **Editor** | Code editing, diffs | Split editor + agent panel |
| **Canvas** | Visual design, previews | Live preview + design chat |

Mode switcher: Tab bar below header, or keyboard shortcut (⌘+1/2/3).

---

## 5. Glass & Blur Effects (Osaurus Pattern)

### 5.1 Glass Materials

| Layer | Material | Blur | Opacity |
|-------|----------|------|---------|
| Window background | `hudWindow` | 30px | 0.55 backing |
| Sidebar | `sidebar` | 20px | 0.10 primary |
| Popovers / modals | `popover` | 25px | 0.12 primary |
| Input area | `hudWindow` | 15px | 0.08 |

### 5.2 Glass Token Spec

```json
{
  "glass": {
    "enabled": true,
    "material": "hudWindow",
    "blurRadius": 30,
    "opacityPrimary": 0.10,
    "opacitySecondary": 0.08,
    "opacityTertiary": 0.05,
    "edgeLight": "rgba(255,255,255,0.20)",
    "windowBackingOpacity": 0.55
  }
}
```

### 5.3 Elevation (Cursor Pattern — Shadow-Only, No Glass)

When glass is disabled (Linux/Windows fallback):

| Level | Shadow |
|-------|--------|
| Flat | none |
| Subtle | `0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.08)` |
| Card | `0 4px 12px rgba(0,0,0,0.15), 0 2px 4px rgba(0,0,0,0.10)` |
| Elevated | `0 8px 24px rgba(0,0,0,0.20), 0 4px 8px rgba(0,0,0,0.12)` |
| Modal | `0 16px 48px rgba(0,0,0,0.25), 0 8px 16px rgba(0,0,0,0.15)` |
| Focus ring | `0 0 0 3px rgba(200, 120, 60, 0.35)` |

---

## 6. Animation & Transitions

### 6.1 Timing Tokens (Osaurus Pattern)

| Token | Duration | Easing | Use |
|-------|----------|--------|-----|
| `quick` | 0.15s | ease-out | Button hover, focus |
| `medium` | 0.25s | ease-in-out | Panel open/close, sidebar collapse |
| `slow` | 0.35s | ease-in-out | Page transitions, mode switches |
| `spring` | 0.4s | spring(0.4, 0.8) | Bounce, emphasis |
| `streaming` | continuous | linear | Typing indicator, progress |

### 6.2 Key Animations

| Element | Animation |
|---------|-----------|
| Sidebar collapse | Width 260→56px, 0.25s ease-in-out, icon fade in |
| Message appear | Fade in + translateY(8px→0), 0.15s |
| Code block expand | Height animate, 0.2s ease-out |
| Thinking collapse | Height to 0 with opacity fade, 0.2s |
| Tool call shimmer | Left-to-right gradient sweep (Osaurus pattern), 1.5s loop |
| Status dot pulse | Scale 1→1.2→1, opacity 0.7→1→0.7, 2s infinite |
| Toast appear | Slide in from right + fade, 0.2s |
| Modal backdrop | Opacity 0→0.55, 0.2s |
| Send button | Scale 0.9→1 on hover, 0.1s |
| Context bar fill | Width transition, 0.3s ease-out |

### 6.3 Streaming UX (Kun Pattern)

```
Sending → Waiting → Thinking → Responding → Tool → Done
  ↓         ↓          ↓           ↓          ↓       ↓
icon     spinner    brain icon   print-head  wrench  check
         (pulse)    (muted)     (rhythm)    (shimmer)
```

Phase transitions use authoritative SSE events, not timers. Each phase has distinct motion:
- **Waiting**: single-cell dot pulse
- **Thinking**: brain icon with muted italic
- **Responding**: character-by-character print-head
- **Tool**: wrench icon with shimmer animation
- **Complete**: checkmark with scale bounce

---

## 7. Component Specs

### 7.1 Buttons

| Variant | Background | Text | Border | Radius | Height |
|---------|-----------|------|--------|--------|--------|
| Primary | `accent` | `#FFFFFF` | none | 8px | 36px |
| Secondary | `surface-2` | `text-primary` | `border` | 8px | 36px |
| Ghost | transparent | `text-secondary` | none | 8px | 36px |
| Icon | `surface-2` | `text-secondary` | none | 8px | 32px |
| Danger | `error` | `#FFFFFF` | none | 8px | 36px |

Padding: 8px 16px (horizontal), 0 12px (icon-only).
Font: 14px, weight 500.

### 7.2 Cards

```css
.card {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--shadow-subtle);
  transition: box-shadow 0.15s ease-out, border-color 0.15s ease-out;
}
.card:hover {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-card);
}
```

### 7.3 Input Fields

```css
.input {
  background: var(--surface-3);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 14px;
  color: var(--text-primary);
  transition: border-color 0.15s, box-shadow 0.15s;
}
.input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
  outline: none;
}
```

### 7.4 Status Indicators

| State | Color | Icon | Animation |
|-------|-------|------|-----------|
| Idle | `text-tertiary` | dot | none |
| Processing | `accent` | spinner | rotate 1s linear infinite |
| Thinking | `thinking` pastel | brain | pulse 2s |
| Error | `error` | ! circle | shake once |
| Success | `success` | check | scale bounce |
| Warning | `warning` | triangle | none |

### 7.5 Context Window Bar

```
┌──────────────────────────────────────────┐
│  ████████████░░░░░░░░  45% · 92K tokens  │
└──────────────────────────────────────────┘
```

- Height: 4px
- Fill: `accent` (under 70%), `warning` (70-90%), `error` (90%+)
- Track: `surface-3`
- Label: right-aligned, caption size

---

## 8. Navigation Patterns

### 8.1 Primary Navigation (Sidebar)

```
┌──────────────┐
│ 🔍 Search    │
│──────────────│
│ 🏠 Home      │
│ 💬 Chat      │  ← active: 3px left bar + accent bg tint
│ 📁 Projects  │
│ 🧩 Skills    │
│ ⚡ Agents    │
│──────────────│
│ 📋 Sessions  │  ← scrollable list
│   ├─ Session 1
│   ├─ Session 2
│   └─ Session 3
│──────────────│
│ ⚙️ Settings  │
│ 👤 Account   │
└──────────────┘
```

Active item: 3px left accent bar + `accent-subtle` background tint + `text-primary` text.
Inactive: `text-secondary` text, no background.

### 8.2 Breadcrumb (Header)

`NeoTrix > Chat > Session 123 > Branch A`

Each segment clickable. Separator: `/` or `›` with `text-tertiary`.

### 8.3 Keyboard Shortcuts (Claude Desktop Pattern)

| Shortcut | Action |
|----------|--------|
| `⌘+K` | Command palette |
| `⌘+/` | Keyboard shortcuts reference |
| `⌘+N` | New session |
| `⌘+;` | Side chat (branch) |
| `⌘+1/2/3` | Switch workspace mode |
| `⌘+B` | Toggle sidebar |
| `⌘+Shift+B` | Toggle browser panel |
| `Esc` | Close modal / dismiss |
| `Enter` | Send message |
| `Shift+Enter` | New line in input |

---

## 9. Responsive Breakpoints

| Breakpoint | Layout |
|------------|--------|
| `> 1200px` | Full: sidebar + main + optional right panel |
| `768-1200px` | Sidebar collapsed (icon-only), main fills |
| `< 768px` | No sidebar, bottom nav, mobile-optimized |

Sidebar collapse animation: `0.25s ease-in-out` width transition.

---

## 10. NeoTrix Brand Integration

### 10.1 Domain Color Mapping

Each NT-* domain gets a subtle tint for visual identity:

| Domain | Primary Color | Subtle Tint |
|--------|--------------|-------------|
| NT-CORE | `#E08A55` (coral) | `rgba(224, 138, 85, 0.10)` |
| NT-MIND | `#9FC9A2` (sage) | `rgba(159, 201, 162, 0.10)` |
| NT-MEMORY | `#9FBBE0` (blue) | `rgba(159, 187, 224, 0.10)` |
| NT-WORLD | `#C0A8DD` (lavender) | `rgba(192, 168, 221, 0.10)` |
| NT-ACT | `#C08532` (gold) | `rgba(192, 133, 50, 0.10)` |
| NT-IO | `#60A5FA` (sky) | `rgba(96, 165, 250, 0.10)` |
| NT-SHIELD | `#E05C4A` (red) | `rgba(224, 92, 74, 0.10)` |
| NT-FEEL | `#DFA88F` (peach) | `rgba(223, 168, 143, 0.10)` |

### 10.2 E8 Hexagram Visual Motif

- Decorative hexagonal grid as subtle background pattern (2% opacity)
- Loading states: hexagram rotation animation
- Status indicators: hexagram fill patterns for constellation maturity (C0-C6)

### 10.3 Constellation Maturity Badge

```
C0 ●  Compiles         — gray
C1 ●  Unit Tests       — blue
C2 ●  Integration      — green
C3 ●  Benchmarked      — gold
C4 ●  Pipeline         — coral
C5 ●  Self-Healing     — purple
```

---

## 11. Accessibility

- WCAG 2.2 AA minimum contrast ratios (4.5:1 text, 3:1 large text)
- All interactive elements keyboard-accessible
- Focus visible: 3px accent ring with 3px offset
- Screen reader labels on all icon-only buttons
- Reduced motion: disable spring animations, use instant transitions
- High contrast mode: increase border weight to 2px, text to 700 weight

---

## 12. Implementation Checklist

### Phase 1: Foundation
- [ ] CSS custom properties (color tokens, spacing, typography)
- [ ] Theme system (dark/light toggle, OS preference detection)
- [ ] Glass material system (macOS native, CSS fallback)
- [ ] Typography scale + font loading (Inter, JetBrains Mono)

### Phase 2: Core Components
- [ ] Button variants (primary, secondary, ghost, icon, danger)
- [ ] Card component with hover states
- [ ] Input fields (text, textarea, search)
- [ ] Status indicators + loading states
- [ ] Context window progress bar

### Phase 3: Layout
- [ ] App shell (titlebar, sidebar, main, statusbar)
- [ ] Sidebar (collapsible, sections, active states)
- [ ] Chat layout (message list, input area, centered 768px)
- [ ] Workspace mode switcher
- [ ] Responsive breakpoints

### Phase 4: Polish
- [ ] Agent timeline pastel system
- [ ] Streaming animations (thinking, tool call shimmer)
- [ ] Keyboard shortcuts
- [ ] Command palette (⌘+K)
- [ ] Toast notifications
- [ ] Domain color integration
- [ ] Constellation maturity badges

---

*Spec version: 1.0 · Generated Sep 5 2026*
*Sources: Claude Desktop (SaaSFrame DOM extraction), Codex CLI (GitHub issues/docs), Osaurus (theme docs + PRs), Kun (architecture docs), Cursor (DESIGN.md extraction), Windsurf (App Store analysis)*
