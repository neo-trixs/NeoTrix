# 2026 UI/Frontend Ecosystem — Absorption Research

**Source**: Internet research (2026-07-22)
**Mapped to**: NT-IO (Interface), NT-WORLD (Rendering)

## Key Patterns

### P201: RICH Interaction Paradigm (Ant Design X)
4-stage AI interaction: Awaken → Express → Confirm → Feedback
- NT-IO mapping: NeoCodex chat interaction model
- Not UI components, but interaction phases as architectural slots

### P202: Component Manifest Standard (Lucent UI)
Machine-readable JSON component metadata: props, variants, guidelines, AI context
- NT-IO: every NeoCodex component needs `COMPONENT_MANIFEST`
- Exposed via MCP tools for zero-hallucination AI generation

### P203: A2UI Declarative JSON for Agent-Generated UI
Google A2UI v0.9 + Ant Group A2UI DynamicCard + Oracle Agent Spec
- **Adopt**: NeoCodex agent should emit A2UI JSON, not raw React
- Trusted catalog rendering = security boundary per R-P48

### P204: Unified Multi-Agent Desktop (AionUi)
30K★, Electron+React, unified agent management, MCP sync, 20+ model platforms, Chinese model support
- **Adopt**: NT-IO NeoCodex Desktop direction validated

### P205: Tailwind CSS Motion (Zero-Runtime)
Compile-time animation utilities, no JS runtime
- **Watch**: If standard, NT-IO emits `motion-*` classes directly

### P206: Chinese GenUI Trends
Liquid Glass aesthetics, Voice-First UI, Predictive UI driven by GWT attention routing
- NT-CORE GWT attention routing → directly drives UI layout priority
