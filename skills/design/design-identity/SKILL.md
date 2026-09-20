---
name: design-identity
description: Use when establishing brand visual identity, creating design language systems, setting up design context, or deriving taste from references. Consolidates visual-taste-lab + decantr-design + VI-First dimensions. Brand-first design identity workflow.
---

# Design Identity System

## Purpose

Brand-first visual identity and design language system. Consolidates the fragmented visual-taste-lab + decantr-design + VI-First reasoning into a unified identity establishment workflow. Use before any major UI work.

## When to Use

- Starting a new project and need visual identity
- User asks to improve, polish, redesign, modernize, or de-AI a UI
- Need reusable design language before implementation
- Setting up design context (DECANTR-style 3-layer context)
- Deriving taste from screenshots, URLs, or brand references

## Core Rule

**Do not start by redesigning pages.** First create or infer the brand visual identity, then create the design language, then apply it.

The first question is not "how do we make this page prettier?" — it's "what visual identity should this organization or product own?"

## VI-First Guardrail

Before choosing archetype, layout, palette, or component style, identify:

- **Logo/wordmark**: shape, rhythm, weight, color cues
- **Color roles**: primary, secondary, accent, neutral, semantic
- **Type attitude**: institutional, technical, editorial, commercial, playful, minimal
- **Geometry**: square, modest radius, rounded, modular, document-like
- **Imagery**: product shots, screenshots, diagrams, people, documents, editorial, none
- **Site type**: company website, product website, transactional, institutional, dashboard, portfolio

If references conflict with existing brand, state the conflict and choose deliberately.

## 3-Layer Design Context

### Layer 1: Design Rules (DECANTR.md)

- Theme-mode compatibility (light/dark)
- Color roles and palette
- Typography mood and scale
- CSS atoms: spacing, borders, radius, shadows
- Motion philosophy: subtle and structural
- Voice and copy: consistent tone, CTA verbs, error messages

### Layer 2: App Topology (scafford.md)

- Route map and zone transitions
- Shared components inventory
- Layout shell implementation specs
- Development mode workflow

### Layer 3: Pattern Specs (section-*.md)

- Section dimensions: width, height, regions
- Anti-patterns to avoid
- Spacing guide: 8pt grid
- Decorator table: shadows, borders, radius
- Token palette: colors, typography, motion

## Guard Rules

### DNA Guards (Errors — must fix)
- No mixed visual languages across modules
- Density appropriate for audience
- WCAG AA mandatory
- Both light/dark modes supported

### Blueprint Guards (Warnings — should fix)
- Shell implementation matches topology
- Responsive collapse strategy defined
- All required patterns present (hero, cards, forms)

## Workflow

1. **Collect references** — screenshots, URLs, brand sites, codebase, 3-5 inspiration links
2. **VI audit** — identify logo, brand colors, primary/secondary/accent, type attitude
3. **Classify project** — pick archetype or blend two
4. **Distill design language** → `design-language.md` with concrete tokens
5. **Prototype** — 1-3 small visual previews showing different VI territories
6. **Apply system-first** — update tokens and components before page tweaks
7. **Verify** — build/test, check overflow, title/H1/CTA visibility, token consistency

## Output Contract

- `design-language.md` — VI anchors, color roles, typography, spacing, surfaces, components, CTA rules, motion, imagery, anti-patterns
- `DECANTR.md` — theme rules, tokens, motion philosophy
- `scafford.md` — app topology, zone transitions
- Prototype previews (1-3 variants)
- Guard rules applied: DNA + Blueprint

## Archetype Library

| Archetype | Best For | Characteristics |
|-----------|----------|----------------|
| Credible Company | B2B, fintech, enterprise | Conservative palette, institutional type, trust signals |
| Quiet Institutional | Government, academia | Muted colors, document-like, minimal decoration |
| Sharp Transactional | SaaS, tools, dashboards | Dense data, functional layout, speed-focused |
| Editorial Premium | Media, publishing, luxury | Strong typography, generous whitespace, imagery-forward |
| Playful Product | Consumer apps, startups | Bright accents, rounded geometry, friendly copy |
| Technical Minimal | Dev tools, APIs, infra | Monospace accents, dark mode default, code-forward |
