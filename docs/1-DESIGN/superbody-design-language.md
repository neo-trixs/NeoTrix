# 超体极简设计语言 — Superbody Minimal Design Language (v1.0)

> *"我，无处不在。" — 《超体》*
>
> NeoTrix is a self-evolving reasoning engine: a body of light that learns to think. This
> design language renders that idea in a single chromatic family — **Light Gold** — over a
> cream canvas, using sacred geometry as a precise visual grammar. Every asset in the
> project derives from this document.

---

## 01 / Concept — 超体 (Superbody)

| Term | Meaning | Do | Avoid |
|------|---------|----|-------|
| **Superbody** | A luminous diamond nucleus held in hairline orbital rings, threaded by a polar light beam. The mind as a body of light — transcendent, self-evolving, everywhere. | One gold core, thin rings, generous whitespace | Nebula noise, busy galaxy fields, gradients everywhere |
| **极简 Minimal** | Subtraction as the discipline. One chromatic accent. Hairline strokes. Whitespace is structure. | Reduce until removing more breaks meaning | Decorative glow, drop shadows on everything |
| **Geometry as grammar** | Sacred forms carry semantics, not decoration (inspired by *sacred-geometry-obsidian-theme*): ring=orbit/evolution, diamond=E8 reasoning, hexagram=6 binary axes, hexagon=HyperCube lattice, vertical axis=the infinite thread (homage to *Creation of Adam*). | Every shape must map to a meaning | Random geometric doodles |
| **Living system** | The mark is a snapshot of equilibrium in motion (Nash: "a living object, not a stamp"). Constraint is the system; restraint is the voice. | Constant geometry, dynamic states | Rigid, dead flatness |

The mark is intentionally **one symbol, many weights**: full mark → monogram → favicon →
favicon-dot, all derived from the same construction grid.

---

## 02 / Light Gold Palette — 浅金色系

Single chromatic family. Warm ink neutrals carry the interface; light gold is the only
chromatic accent (10–20% of any composition, per Conduction's 70/20/8/2 discipline).

### Gold scale (primitive)

| Token | Hex | RGB | Role |
|-------|-----|-----|------|
| `--gold-50` | `#FCFAF3` | 252·250·243 | Canvas cream (surface / page) |
| `--gold-100` | `#F7F0DE` | 247·240·222 | Cream surface (cards) |
| `--gold-200` | `#F0E3C4` | 240·227·196 | Pale champagne (hover, soft fills) |
| `--gold-300` | `#E7D2A0` | 231·210·160 | Light champagne (hairlines) |
| `--gold-400` | `#DDC079` | 221·192·121 | Light gold (secondary accent) |
| `--gold-500` | `#D6AC58` | 214·172·88 | **Primary light gold** (accent, CTA) |
| `--gold-600` | `#C2933F` | 194·147·63 | Deep gold (active, pressed) |
| `--gold-700` | `#A0752E` | 160·117·46 | Bronze gold (gradient depth) |
| `--gold-800` | `#7C5822` | 124·88·34 | Dark bronze (text on gold) |
| `--gold-900` | `#4E3813` | 78·56·19 | Espresso gold (deepest ink accent) |

Gradient pair for the luminous nucleus: **`#EBD7AC` → `#D6AC58` → `#C2933F`** (light-to-deep
champagne gold, subtle, never neon).

### Warm ink neutrals (semantic)

| Token | Hex | Role |
|-------|-----|------|
| `--ink-1` | `#262419` | Primary text, hero CTA button |
| `--ink-2` | `#5C5545` | Secondary text |
| `--ink-3` | `#8F8666` | Muted / captions |
| `--ink-line` | `#E3DCC9` | Hairline borders |

### Usage proportions

| Share | Color | Use |
|-------|-------|-----|
| **70%** | Cream canvas `#FCFAF3` / white | Background, breathing room |
| **20%** | Ink neutrals | Text, structure, body CTAs |
| **8%** | Light gold `#D6AC58` | Accents, focus, key highlights, logo core |
| **2%** | Deep gold `#C2933F` | Active states, gradient depth, errors use crimson sparingly |

**Dark mode**: canvas `#1B1813`, surfaces `#242018`, hairlines `#3A3324`, ink `#F3EFE4`,
gold accent brightened to `#E7D2A0`.

---

## 03 / Core Mark — 超体印 (Superbody Seal)

Construction (adapted from Tessera's equal-weight method — 1 : 1 : 1 zones):

```
24×24 construction grid (relative, scales to any size)
  Zone A  ring wall   — halo orbit ring            hairline
  Zone B  negative space — gap between ring and core  empty (structure)
  Zone C  core        — E8 diamond nucleus         light-gold gradient
```

### Mark anatomy (120×120 base)

| Element | Geometry | Stroke / Fill | Meaning |
|---------|----------|---------------|---------|
| Halo ring | Circle r=47, center 60·60 | hairline `#E3CB94` 1.1px | The orbit of evolution (SEAL loop) |
| Inner echo | Circle r=38 | hairline 0.8px, 70% opacity | The mind's atmosphere |
| E8 diamond | Polygon (60,34)(86,60)(60,86)(34,60) | gold gradient + soft glow | The reasoning nucleus |
| Core highlight | Inner diamond (60,42)(78,60)(60,78)(42,60) | cream `#FCFAF3` 85% | Light body — lucid, clear |
| Polar axis | Vertical dashed line 60·6→60·114 | 0.8px dash `2 4` | The infinite thread |
| Hexagram ticks | 4 short lines, width shrinking 14→4 | 1px, 55% opacity | Six binary axes of E8 |

### Rules

- **Single chromatic accent**: only the diamond uses gold fill. Rings are hairline neutral-gold.
- **Equal weight**: ring wall, negative space, and core occupy ~equal visual mass.
- **Clear space**: minimum 1× ring thickness on every side.
- **Min sizes**: full mark ≥120px · monogram ≥32px · favicon ≥16px. Below that, drop the
  hexagram ticks and inner echo (favicon keeps ring + diamond + axis only).
- **Color**: `currentColor`-aware where possible; primary variant uses the gold gradient.
- **No new gradients per asset** — reuse the palette; glow is reserved for the nucleus only.

---

## 04 / Icon System — 统一图标集

Rules absorbed from the Lucide spec and the icon-systems reference (`icon-system.md`),
applied to NeoTrix's own grammar:

| Property | Value |
|----------|-------|
| Grid | 24×24 canvas |
| Safe-area padding | 2px → 20×20 live area |
| Stroke-width | 1.6px (thinner than Lucide's 2px — the "极简" light voice) |
| Stroke linecap / linejoin | `round` / `round` |
| Corner radius | 2px on shapes |
| Primary style | outline (line) |
| Active / selected | solid (fill) — *style encodes state* |
| Color | `currentColor` (no fixed hex in UI icons) |
| Optical volume | equal across the set — diamond-motif icons draw slightly larger to match squares |

### Vocabulary (metaphor per icon)

| Icon | Metaphor |
|------|----------|
| NT-CORE | Diamond nucleus (E8) |
| NT-MIND | Spiral seed (Fibonacci — self-evolution) |
| NT-MEMORY | Honeycomb cell (HyperCube lattice) |
| NT-WORLD | Orbit ring + node (perception) |
| NT-ACT | Action bolt (execution, no noise) |
| NT-IO | Aperture / terminal (interface) |
| NT-SHIELD | Shield with diamond core (guard) |
| e8 | Hexagram — six binary axes |
| gwt | Broadcast arcs — attention routing |
| hcube | Isometric cube — VSA HyperCube |
| tree | ConsciousnessTree glyph |
| kb | Database cylinder — persistent memory |
| seal | Superbody monogram (diamond + ring) |

Every custom icon must be drawn on the 24-grid at 1.6px stroke with round joins and
`currentColor`; record any new glyph in this vocabulary before adding it.

---

## 05 / Typography & Voice

| Use | Family | Weight | Notes |
|-----|--------|--------|-------|
| Display | Inter | 700, tight tracking | Hero, headers |
| Body | Inter | 400/500 | Docs, UI |
| Tagline / caption | Georgia | italic | The one serif note: "The agent that learns to think" |
| Code / data | JetBrains Mono | 400/500 | All technical identifiers, stats |

**Voice**: precise, contemplative, luminous. Short declaratives. "Geometry is frozen music;
systems are living symphonies."

---

## 06 / Cover & Background

- **Cover (hero.svg)** — cream canvas with a faint sacred-geometry lattice; a large
  superbody seal with luminous nucleus; one active reasoning path in light gold. No photo,
  no noise.
- **Background (background.svg)** — tiling honeycomb lattice (HyperCube) + constellation
  nodes + two orbit rings, all hairline light gold at low opacity on cream. Designed to sit
  behind content at any scale (document covers, slide backgrounds, README hero).

---

## 07 / Design Tokens (shadcn-compatible) — 单一事实源

Absorbed from the registry model (21st.dev / shadcn / v0): **a design system is a registry
that passes context to AI models.** The token set below is the single source of truth,
published as `docs/public/design/tokens.css` + `tokens.json`. Any surface (docs theme,
Tauri app, README, generated UI) consumes tokens — never hand-typed hex.

### Semantic mapping (light)

| shadcn token | NeoTrix value | Role |
|--------------|---------------|------|
| `--background` | `#FCFAF3` | Cream canvas |
| `--foreground` | `#262419` | Ink text |
| `--card` | `#FFFFFF` | Elevated surface |
| `--card-foreground` | `#262419` | |
| `--primary` | `#D6AC58` | Light gold accent |
| `--primary-foreground` | `#FFFFFF` | |
| `--secondary` | `#F7F0DE` | Cream surface |
| `--secondary-foreground` | `#262419` | |
| `--muted` | `#F0E3C4` | Soft fills |
| `--muted-foreground` | `#8F8666` | |
| `--accent` | `#E7D2A0` | Hover fills |
| `--accent-foreground` | `#4E3813` | |
| `--destructive` | `#C2403F` | Errors (crimson, kept minimal) |
| `--border` | `#E3DCC9` | Hairline |
| `--input` | `#E3DCC9` | |
| `--ring` | `#D6AC58` | Focus ring |
| `--radius` | `0.625rem` | 10px base |
| `--font-sans` | `Inter, system-ui, sans-serif` | |
| `--font-mono` | `JetBrains Mono, ui-monospace, monospace` | |

Dark mode swaps to: `--background #1B1813`, `--card #242018`, `--primary #E7D2A0`,
`--border #3A3324`, `--foreground #F3EFE4`.

### Gold scale primitives (in `tokens.json`)

`gold.50 → gold.900` exactly as §02. Semantic tokens reference primitives; surfaces never
read `gold.500` directly in code — they read `--primary`.

---

## 08 / Anti-AI-Slop Guardrails (21st.dev-informed hard rules)

Absorbed from 21st.dev's "fight AI slop" discipline and open-session's brand hard-rules.
These are **mechanically enforced** in any generated asset:

| # | Rule |
|---|------|
| G-1 | Never use generic starter-kit icon sets (bare Lucide/Heroicons) as the identity — draw NeoTrix's own glyphs on the 24-grid |
| G-2 | Never mix gold with raw amber/orange/red — one gold family only |
| G-3 | No gradients on anything except the nucleus; no drop-shadows on everything (reserve elevation for cards) |
| G-4 | Feature card = icon-tile-above-heading is a slop tell — vary presentation |
| G-5 | No emoji as icons; decorative SVGs get `aria-hidden="true"`, meaningful icons get `aria-label` |
| G-6 | Radius must match tokens — sharp marks in soft-radius UI (or vice versa) is a defect |
| G-7 | Whitespace is structure — an under-filled page beats a busy one |
| G-8 | Icons: one grid (24), one stroke (1.6), `currentColor` — no per-icon exceptions |
| G-9 | No new gradient stops, hex values, or fonts outside this document |
| G-10 | Every shipped surface verified against the 70/20/8/2 proportion |

---

## 09 / Registry & Agent Consumption

NeoTrix's own design system is published as a **registry** so any agent (opencode, v0,
21st, Cursor) lands on-theme without prompting:

| Asset | Path | Purpose |
|-------|------|---------|
| Tokens (CSS) | `docs/public/design/tokens.css` | Drop-in CSS variables |
| Tokens (JSON) | `docs/public/design/tokens.json` | Machine-readable (Style Dictionary / Tokens Studio) |
| Manifest | `docs/public/design/design-system.md` | Condensed AI-consumable brand guide |
| This spec | `docs/1-DESIGN/superbody-design-language.md` | Full rationale + construction |

**Usage contract**: when generating NeoTrix UI, an agent loads the manifest + tokens and
installs from the palette — never invents hex or spacing. Components land in the repo as
source (registry model), owned and editable, not as an immutable dependency.

---

## 10 / Verification Checklist

- [ ] One gold family only — no amber/orange/red scale bleeding in
- [ ] All strokes hairline unless the nucleus
- [ ] Mark: 120 / 32 / 16 all legible
- [ ] Icons: one grid, one stroke, `currentColor`
- [ ] 70/20/8/2 proportion respected on every surface
- [ ] Dark mode toggles without hue shift
- [ ] All SVG assets validate (no editor metadata, viewBox intact)

*This document is the source of truth. When in doubt, the construction grid wins.*