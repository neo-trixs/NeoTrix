# NeoTrix Design System — AI-Consumable Manifest

**Superbody Minimal · Light Gold** — condensed brand guide for agents. Full spec:
`docs/1-DESIGN/superbody-design-language.md`. Tokens: `tokens.css` + `tokens.json`.

## The idea in one line
NeoTrix is a body of light that learns to think. A diamond nucleus (E8) in hairline orbital
rings, threaded by a polar beam — rendered in one chromatic family: **Light Gold** on a
cream canvas. Minimal means one accent, hairline strokes, whitespace as structure.

## Palette (never invent hex — use these)

| Role | Token | Hex |
|------|-------|-----|
| Canvas | `--background` | `#FCFAF3` |
| Ink | `--foreground` | `#262419` |
| Primary accent | `--primary` | `#D6AC58` |
| Deep gold | `--gold-600` | `#C2933F` |
| Champagne | `--gold-200` | `#F0E3C4` |
| Hairline | `--border` | `#E3DCC9` |
| Focus ring | `--ring` | `#D6AC58` |
| Radius | `--radius` | `0.625rem` (10px) |

Dark mode: canvas `#1B1813`, primary `#E7D2A0`, border `#3A3324`, ink `#F3EFE4`.

## Geometry grammar (sacred forms carry meaning)
- **Ring** = orbit / evolution (SEAL loop)
- **Diamond** = E8 reasoning nucleus
- **Hexagram** = six binary axes
- **Hexagon** = HyperCube lattice
- **Vertical axis** = the infinite thread
- Glow + gradient reserved for the nucleus ONLY. Everything else hairline.

## Icon rules
24×24 grid · 2px padding · 1.6px stroke · round caps/joins · `currentColor` · one family
(NeoTrix's own glyphs in `docs/public/icons/`). Outline is default; solid encodes
active/selected. No Lucide/Heroicons defaults as identity.

## Typography
Sans: **Inter** (700 display / 400-500 body). Mono: **JetBrains Mono**. Serif: **Georgia
italic** — only for the tagline "The agent that learns to think".

## Hard rules (from the anti-slop gate)
1. One gold family — no amber/orange/red bleed.
2. No gradients except the nucleus; reserve drop-shadows for cards.
3. 70/20/8/2 proportion: canvas / ink / gold accent / deep-gold-or-error.
4. Feature card = icon-tile-over-heading is a slop tell — vary it.
5. No emoji icons; `aria-hidden` on decorative SVG, `aria-label` on meaningful.
6. Whitespace beats busy. Under-filled wins.

## Usage contract
Load `tokens.css` or `tokens.json`, read this manifest, then generate. Components land in
the repo as editable source (registry model). Verify every surface against the checklist in
§10 of the full spec before shipping.