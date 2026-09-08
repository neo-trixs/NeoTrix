# Iteration Batch 428 — Research Loop

**Date**: 2026-09-06
**Focus**: Desktop Frameworks, GUI Design Systems, UX/Accessibility

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | tauri v2.11.5 (crates.io) | 2026-07-01 | Tauri latest release, mobile support |
| S2 | Top 5 Electron alternatives (MōBrowser) | 2026-04-27 | Framework comparison: Electron/Tauri/MōBrowser/NW.js/Electrobun |
| S3 | Tynd (github/kvnpetit) | 2026 | TypeScript-first desktop, zero codegen typed RPC |
| S4 | Mirin (github/Netko-Labs) | 2026-06-10 | Bun-native CEF desktop framework |
| S5 | Volt (github/voltkithq) | 2026 | TypeScript + Rust, Boa engine, capability permissions |
| S6 | Vesper (github/DannelCu) | 2026-06-14 | Python-first desktop, typed IPC, sandboxing |
| S7 | 7onic Design System | 2026-03-11 | Figma-to-code token pipeline, llms.txt, dual Tailwind |
| S8 | Tokis (npm) | 2026-03-11 | Zero-runtime CSS, token-native, WCAG 2.2 AA |
| S9 | YunUI (github/YuhuanStudio) | 2026 | OKLCH tokens, React 19 + Tailwind v4, AI components |
| S10 | Nerio (github/vpavlov-me) | 2026 | Source-installed registry, MCP server, llms.txt |
| S11 | Dark Mode Accessibility Guide | 2026-01-06 | WCAG dark mode, elevation, contrast |
| S12 | AI Orchestration of Accessible Design (Timothy Graf) | 2026-08-07 | Proactive adaptive UX, WCAG 2.2, screen reader optimization |
| S13 | Beyond Predictability (Timothy Graf) | 2026-07-05 | Adaptive patterns, cognitive load, Gestalt, progressive disclosure |
| S14 | Effective UI Design Skill (sebastian-software) | 2026-02-05 | OKLCH, 8pt grid, fluid typography, dark mode, Core Web Vitals |
| S15 | Dark Mode Done Right (Art of Style Frame) | 2026-07-14 | Elevation-based dark mode, token architecture, WCAG 4.5:1 |

---

## Defects Found

### D1: Dark Mode Is Not Token-Defined — Only "自动适配" Claimed

**Current**: DESIGN-LANGUAGE.md:143 claims "暗色模式: 自动适配（非手动）" but provides zero dark-mode token definitions, no dual-palette architecture, and no elevation-based lighting rules.

**2026 Best Practice** (S11, S15): Dark mode must be a parallel palette with:
- Base surface #121212 (not #000000)
- Elevation via lighter surfaces (card #1e1e1e, modal #242424, menu #2c2c2c)
- Desaturated brand colors per theme
- WCAG 4.5:1 verified text-on-surface pairs

**Gap**: No `--color-surface-base`, `--color-surface-raised`, `--color-surface-overlay` tokens. No dark-mode brand color variants. No contrast validation pipeline.

### D2: Hex Color Values — No OKLCH Color Space

**Current**: DESIGN-LANGUAGE.md:18-20 uses raw hex (#4d6bfe, #7c3aed).

**2026 Best Practice** (S9, S14): OKLCH is the standard for perceptually uniform color systems. Enables:
- Relative color syntax (`oklch(from var(--brand) l c h / 0.5)`)
- Perceptually accurate contrast calculations
- Programmatic palette generation

**Gap**: Primitive tokens should use OKLCH. Hex is a lossy export format, not a source format.

### D3: No AI-Readable Component Metadata (llms.txt)

**Current**: DESIGN-LANGUAGE.md has no `llms.txt` or structured metadata for AI tools.

**2026 Best Practice** (S7, S10): Design systems ship `llms.txt` files so AI agents (Claude, Cursor, Copilot) automatically use tokens instead of hardcoded values. Nerio ships an MCP server for component discovery.

**Gap**: AI coding agents cannot discover NeoTrix design tokens or component contracts. They will default to hardcoded colors and arbitrary spacing.

### D4: No Figma-to-Code Token Pipeline

**Current**: Token sources are `tokens.json` → `tokens.css` → `design-system.md` (S7 shows this pattern).

**2026 Best Practice** (S7): Automated pipeline: `figma-tokens.json` → `npx sync-tokens` → CSS/Tailwind v3+v4/JS/TS/JSON outputs with breaking change detection.

**Gap**: Manual token authoring. No designer-developer sync. No automated diff detection when tokens change.

### D5: No WCAG 2.2 AA Compliance Specification

**Current**: DESIGN-LANGUAGE.md:148 mentions "对比度 AA（≥4.5:1）" but:
- No focus indicator specification
- No `prefers-reduced-motion` handling
- No `prefers-color-scheme` media query usage
- No target size minimums (44×44px WCAG 2.2)
- No drag-and-drop accessibility (WCAG 2.2 new criterion)

**2026 Best Practice** (S12, S13, S14): WCAG 2.2 adds: dragging alternatives, consistent help, focus appearance, target size. AI-orchestrated accessibility proactively adapts to user needs.

**Gap**: Missing 6+ WCAG 2.2 success criteria. No accessibility testing tooling integration.

### D6: Desktop Framework Stack Not Evaluated for 2026

**Current**: NeoTrix uses Tauri for desktop (src-tauri/). No evaluation of:

| Framework | Key Advantage | Relevance |
|-----------|---------------|-----------|
| Tynd (S3) | Zero-codegen typed RPC, ~6.5MB lite mode | Eliminates IPC boilerplate |
| Mirin (S4) | Bun-native, CEF consistent rendering | TypeScript-first, no Rust required |
| Volt (S5) | TypeScript orchestration + Rust-backed APIs, capability permissions | Capability-based security model |
| Vesper (S6) | Python-first, typed IPC, sandboxing | If Python tooling needed |
| Tauri v2.11 (S1) | Mobile support (iOS/Android), 24M+ downloads | Already using, but mobile gap unaddressed |

**Gap**: No architectural decision record (ADR) for framework choice. Tauri v2.11.5 has mobile support but DESIGN-LANGUAGE.md doesn't address mobile layout or capability-based permissions model.

### D7: No Component-Level Dark Mode Contract

**Current**: Buttons, Sidebar, Composer, Session defined only for light mode.

**2026 Best Practice** (S15): Every component needs dual-theme tokens. Charts specifically need a second palette (S15: "eight categorical series tuned for white background will have at least two colors that turn to mud on #121212").

**Gap**: No dark-mode component variants. No chart palette specification.

### E1: No Elevation System for Dark Mode

**Current**: `--shadow-md`, `--shadow-xs` defined but no elevation tokens.

**2026 Best Practice** (S11, S15): In dark mode, shadows vanish. Elevation = lighter surfaces. Need `--surface-level-0` through `--surface-level-4` tokens.

**Gap**: Shadow-based hierarchy will break in dark mode.

### D8: No `prefers-reduced-motion` Support

**Current**: Animation system (lines 84-109) defines keyframes and durations but no reduced-motion media query.

**2026 Best Practice** (S14): `@media (prefers-reduced-motion: reduce)` must disable or simplify animations. WCAG 2.2 requires this.

**Gap**: Users with vestibular disorders get forced animations.

### D9: Zero-Runtime CSS Not Adopted

**Current**: Uses CSS custom properties (good) but no mention of zero-runtime pattern.

**2026 Best Practice** (S8): Tokis achieves zero-runtime by precompiling all styles. 7onic uses Tailwind v3+v4 dual support with token-native approach.

**Gap**: If NeoTrix frontend uses CSS-in-JS, it adds runtime overhead. Should evaluate Tailwind v4 + design token integration.

### D10: No Accessibility Testing Toolchain

**Current**: L2 verification mentions contrast ratio but no tooling.

**2026 Best Practice** (S11): Combine automated contrast checking + keyboard-only navigation testing + screen reader testing + theme switching tests.

**Gap**: No axe-core, Lighthouse, or Playwright accessibility tests in CI.

---

## Defect Summary

| ID | Severity | Category | Description |
|----|----------|----------|-------------|
| D1 | **Critical** | Dark Mode | No dark-mode token architecture, only "自动适配" claim |
| D2 | **High** | Color System | Hex primitives instead of OKLCH |
| D3 | **High** | AI Integration | No llms.txt for AI tool discovery |
| D4 | **Medium** | Design Ops | No Figma-to-code token pipeline |
| D5 | **High** | Accessibility | Missing WCAG 2.2 AA specification (6+ criteria) |
| D6 | **Medium** | Architecture | No ADR for 2026 desktop framework landscape |
| D7 | **High** | Components | No dark-mode component variants |
| E1 | **High** | Dark Mode | No elevation token system for dark surfaces |
| D8 | **Medium** | Accessibility | No prefers-reduced-motion support |
| D9 | **Medium** | Performance | Zero-runtime CSS not evaluated |
| D10 | **Medium** | QA | No accessibility testing toolchain |

---

## Suggestions

### S1: Build Dual-Palette Token Architecture

```css
/* Light theme */
--surface-base: oklch(1.0 0 0);        /* #ffffff */
--surface-raised: oklch(0.97 0 0);     /* #f7f7f8 */
--surface-overlay: oklch(0.95 0 0);

/* Dark theme */
--surface-base: oklch(0.14 0 0);       /* #121212 */
--surface-raised: oklch(0.19 0 0);     /* #1e1e1e */
--surface-overlay: oklch(0.22 0 0);    /* #242424 */

/* Brand per theme */
--brand-light: oklch(0.55 0.25 265);   /* #4d6bfe */
--brand-dark: oklch(0.72 0.15 265);    /* desaturated for dark */
```

### S2: Ship llms.txt + MCP Server

```txt
# NeoTrix Design System

## Tokens
- Use --color-interactive-default for primary actions
- Use --spacing-* (2px grid) for layout
- Dark mode: prefer --brand-dark over --brand-light

## Components
- Button: .btn-primary, .btn-secondary, .btn-ghost
- Spacing: 8pt grid, never arbitrary
- Typography: fluid clamp(1rem, 2.5vw, 1.25rem)
```

### S3: Add WCAG 2.2 Compliance Checklist

```markdown
- [ ] Focus indicator: 2px solid, 3:1 contrast minimum
- [ ] Target size: 44×44px minimum
- [ ] prefers-reduced-motion: reduce keyframes
- [ ] prefers-color-scheme: dark token swap
- [ ] Drag-and-drop: keyboard alternative
- [ ] Contrast: 4.5:1 text, 3:1 UI components
```

### S4: Create ADR for Desktop Framework

Evaluate Tauri v2.11+ (current) vs Tynd (typed IPC) vs Volt (capability permissions) based on:
- Rust expertise availability
- Mobile support requirements
- IPC type safety needs
- Binary size constraints
- Capability-based security model alignment with NT-SHIELD

### S5: Implement Accessibility Testing in CI

```yaml
- name: A11y Tests
  run: |
    npx axe-core --rules color-contrast,focus-order
    npx lighthouse --only-categories=accessibility
    npx playwright test --grep @a11y
```

### S6: Elevation Token System

```css
/* Elevation levels (dark mode uses lighter = higher) */
--elevation-0: var(--surface-base);     /* background */
--elevation-1: var(--surface-raised);   /* cards */
--elevation-2: var(--surface-overlay);  /* modals */
--elevation-3: oklch(0.27 0 0);        /* menus, popovers */
--elevation-4: oklch(0.30 0 0);        /* tooltips, toasts */
```

---

## Iteration Metrics

- **Sources scanned**: 15
- **Defects identified**: 11 (3 Critical/High, 5 High, 3 Medium)
- **Suggestions generated**: 6 concrete implementation paths
- **Coverage**: Desktop framework (6 sources), GUI design (4 sources), UX/Accessibility (5 sources)
- **Next iteration focus**: Concrete implementation of D1 (dark mode tokens) + D3 (llms.txt)
