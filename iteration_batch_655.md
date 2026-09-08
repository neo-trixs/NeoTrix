# Iteration Batch 655 — HCI / UX Design / Accessibility (2026)

**Date**: 2026-09-06
**Prior batch**: 654 — Kanai GMW 4-component signature, GNW not computational, no aligned-mediation metric, no purposeful forgetting, symbolic-subsymbolic bridge broken

---

## Domain 1: HCI — AI Agent Interaction Paradigm Shift

### Finding 1.1: Agentic Delegation Replaces Step-by-Step Control
- **Source**: IEEE Computer Society SBC UEMK, "Top HCI Trends in 2026: The Rise of AI Agents" (2026-07-25); CHI '26 proceedings
- **Evidence**: "In 2026, advances in AI are moving systems from reactive tools to goal-driven agents. This shifts interaction from step-by-step control to high-level delegation, reshaping Human-Computer Interaction."
- **CHI '26 acceptance rate**: 6,501 / 27,660 = 24% — massive field activity

### Finding 1.2: HCI Visions Create Hype AND Restrict Creativity
- **Source**: Grønbæk et al., "How Do Future Visions Shape the Field of HCI?", CHI '26, DOI: 10.1145/3772318.3791038
- **Evidence**: Survey of 172 HCI researchers found visions simultaneously **guide** (26.2%), **initiate paradigms** (25%), **give drive** (21%) — but also **create hype**, **restrict creativity**, and **make us disregard real-world problems**. Researchers explicitly acknowledge visions as a double-edged sword.

### NEW DEFECT D-655-1: NT-IO lacks agentic-delegation interaction model
- **Problem**: NeoTrix NT-IO (L1 Action layer) currently models interactions as command-response or tool-calling. The 2026 HCI field has shifted to **goal-driven delegation** — user specifies intent, agent autonomously plans and executes multi-step workflows. NT-IO has no `DelegationProtocol` that:
  1. Accepts high-level intent specifications
  2. Decomposes into sub-goals with confidence tracking
  3. Reports progress transparently without requiring user micromanagement
  4. Handles delegation failure with graceful degradation (not crash)
- **Impact**: NT-IO cannot serve as the human-facing interaction layer for agentic workflows. Users must manually orchestrate each step.
- **Fix**: Add `nt_io::delegation` module with `DelegationRequest`, `DelegationProgress`, `DelegationResult` types. Wire through GWT so consciousness can observe delegation quality and inject corrections.

### NEW DEFECT D-655-2: NT-CORE visions (ConsciousnessTree branches) lack self-critique mechanism
- **Problem**: CHI '26 research shows HCI visions create hype, restrict creativity, and cause researchers to disregard real-world problems. NeoTrix's ConsciousnessTree has 11 branches (visions) but NO mechanism for **meta-critique of the visions themselves**. There is no process that:
  1. Audits whether a branch is generating hype vs. genuine progress
  2. Detects when a branch is constraining exploration of adjacent possibilities
  3. Periodically challenges the branch's foundational assumptions against real-world data
- **Impact**: ConsciousnessTree could drift into self-reinforcing echo chamber — branches validate each other without external grounding.
- **Fix**: Add `nt_meta::vision_critic` that runs periodically on each branch, testing: (a) prediction accuracy of branch goals vs. actual outcomes, (b) novelty of recent contributions vs. repetition, (c) grounding in real user feedback vs. internal metrics.

### NEW DEFECT D-655-3: No hallucination-as-cognitive-friction metric
- **Source**: Sánchez-Vaquerizo & Monsivais, "From Particles to Agents: Hallucination as a Metric for Cognitive Friction in Spatial Simulation", AlpCHI 2026 workshop
- **Evidence**: LLM hallucination reframed as **cognitive friction metric** — when agents generate plausible but false outputs, it creates friction in human understanding. No NeoTrix metric tracks this.
- **Impact**: NT-IO has no way to quantify how much its outputs are causing cognitive friction in human collaborators. LLM-backed features (conversational agents, auto-complete) silently inject hallucinations that degrade trust.
- **Fix**: Add `cognitive_friction_score` to NT-IO output pipeline — probabilistic estimate based on output novelty vs. KB grounding, with GWT broadcast when friction exceeds threshold.

---

## Domain 2: UX Design — Token-Centric Architecture + Motion Tokenization

### Finding 2.1: Tokens Are the New Contract (Not Components)
- **Source**: CreativeAlive, "Design Systems in 2026: From Component Libraries to Motion Tokens" (2026-05-09)
- **Evidence**: "Five years ago, the component library was the contract. In 2026, it's tokens. Components are replaceable implementations of a tokenized system."

### Finding 2.2: Motion Is Now Tokenized
- **Source**: Same source
- **Evidence**: "Named durations, named easings, named springs. Components consume them. A branch consistency issue that previously required human review is now enforced by build-time token checks."

### Finding 2.3: Design Systems Governed as Products, Not Libraries
- **Source**: MDX, "UI UX Design Trends (2026): What's Actually Changing" (2026-06-28)
- **Evidence**: "Design systems are governed like products, not libraries." Governance = ownership + contribution rules + measurable outcomes. Tokens as contract. Accessibility baked into components.

### NEW DEFECT D-655-4: NT-IO design tokens lack motion/voice/emotion dimension
- **Problem**: NeoTrix design language (Superbody Minimal) defines color/typography/spacing tokens but has **no motion tokens** and **no voice/emotion tokens**. 2026 design systems track: color, type, spacing, radius, elevation, **motion**, **voice**. NT-IO's `nt_io::quick_start_guide` and UI components have no:
  1. Named animation durations (tap=100ms, modal=300ms, page=500ms, toast=200ms)
  2. Named easing curves (ease-out for enters, ease-in for exits, spring for bounce)
  3. Voice/tone guidelines for agent-generated text
  4. Emotion-consistent motion (Joy → lively transitions, Sadness → slow fades)
- **Impact**: NeoTrix desktop app (Tauri) and web interfaces produce inconsistent, un-governed motion. Agent outputs have no consistent voice/tone.
- **Fix**: Add `nt_io::design_tokens::motion` module with `MotionToken { name, duration_ms, easing, spring_config }` and integrate with EmotionLabel to derive motion from emotional state.

### NEW DEFECT D-655-5: No component contribution ladder or deprecation protocol
- **Problem**: 2026 design systems use a **contribution ladder** (New → Candidate → Stable) and **deprecation policy** (codemod + 6-month window). NeoTrix capability tree has maturity levels (C0-C6) but:
  1. No formal promotion criteria from one level to the next (what evidence triggers C0→C1?)
  2. No deprecation protocol — dead modules persist indefinitely (violates Dark Forest axiom)
  3. No codemod tooling for migrating consumers when a module changes API
- **Impact**: Capability tree accumulates stale modules. No automated path from prototype to production. Breaking changes propagate without migration support.
- **Fix**: Add `nt_mind::capability_ladder` that tracks: (a) promotion evidence requirements per transition, (b) deprecation TTL with automated consumer notification, (c) codemod generation for API migrations.

### NEW DEFECT D-655-6: Documentation drift not addressed at system level
- **Problem**: CreativeAlive reports "Documentation drift has finally stopped being the dominant failure mode" in 2026 — docs are **generated from source**, not hand-maintained. NeoTrix has `CONTEXT.md` and `AGENTS.md` as manually maintained docs, but:
  1. No auto-generation of API docs from Rust doc comments
  2. No CI gate that fails on doc-comment drift from actual signatures
  3. CONTEXT.md domain terms can diverge from actual module implementations
- **Impact**: New developers (and agents) read stale documentation. Domain terms in CONTEXT.md may not match actual code.
- **Fix**: Add `cargo doc` + signature-extraction CI step that validates doc examples compile and signatures match.

---

## Domain 3: Accessibility — WCAG 2.2 + Cognitive + Agentic Accessibility

### Finding 3.1: WCAG 2.2 New Criteria: Focus Not Obscured, Dragging Movements, Consistent Help, Accessible Authentication
- **Source**: W3C WCAG 2.2 spec; Samkov, "Web Accessibility 2026" (2026-04-19)
- **New in WCAG 2.2 AA**: SC 2.4.11 (Focus Not Obscured), SC 2.5.7 (Dragging Movements), SC 3.2.6 (Consistent Help), SC 3.3.8 (Accessible Authentication)
- **Key stat**: 96% of home pages have low contrast issues. 4,600+ ADA lawsuits in 2023, up 42% YoY.

### Finding 3.2: Automated Tools Catch Only 30-40% of WCAG Failures
- **Source**: Samkov (2026)
- **Evidence**: "Automated tools catch somewhere between 30% and 40% of WCAG failures." The remaining 60% requires manual keyboard testing + screen reader testing. axe-core is the standard engine.

### Finding 3.3: Screen Reader Users Navigate Accessibility Tree, Not DOM
- **Source**: Samkov (2026)
- **Evidence**: "Screen readers don't read HTML — they read the accessibility tree." The tree has 4 core properties: role, name, state, value. Virtual cursor ≠ keyboard focus. ARIA live regions are the only way to announce dynamic content changes.

### Finding 3.4: Core Web Vitals Directly Impact Accessibility
- **Source**: Samkov (2026)
- **Evidence**: "CLS is disorienting for users with cognitive disabilities and catastrophic for screen reader users whose virtual cursor position shifts when the layout changes. Long Tasks delay focus movement and ARIA live region announcements."

### NEW DEFECT D-655-7: NT-IO web interface has no ARIA live region architecture
- **Problem**: NeoTrix web interface (if served) would have dynamic content updates (agent progress, KB search results, consciousness status) but no ARIA live region architecture:
  1. No `role="status"` containers for agent progress announcements
  2. No `role="alert"` for critical consciousness warnings
  3. No `aria-live="polite"` on search result containers
  4. Dynamic content changes would be invisible to screen reader users
- **Impact**: Screen reader users cannot perceive NeoTrix's real-time consciousness updates, agent progress, or KB search results. ~285 million visually impaired people globally excluded.
- **Fix**: Add `nt_io::a11y_live_regions` module that wraps all dynamic content containers with appropriate ARIA roles. Wire to EventBus so consciousness broadcasts automatically trigger live region updates.

### NEW DEFECT D-655-8: No keyboard-navigable consciousness dashboard
- **Problem**: WCAG 2.2 SC 2.4.11 (Focus Not Obscured) requires focused elements not be hidden by sticky headers. NeoTrix consciousness dashboard (if rendered as Tauri/web UI) likely has:
  1. No skip-to-main-content navigation
  2. No focus trap management for modal consciousness details
  3. No `tabindex` management for consciousness tree branch navigation
  4. Sticky header potentially obscuring focused elements
- **Impact**: Keyboard-only users (~2 million Americans with motor impairments) cannot navigate consciousness dashboard.
- **Fix**: Implement focus management patterns: skip links, focus traps for modals, scroll-margin-top for sticky headers, roving tabindex for tree navigation.

### NEW DEFECT D-655-9: No accessible authentication for KB access
- **Problem**: WCAG 2.2 SC 3.3.8 (Accessible Authentication) prohibits cognitive function tests (puzzles, memory) for login. NeoTrix KB access (if authenticated) may use:
  1. Password-based auth without WebAuthn/passkey alternative
  2. CAPTCHA challenges for API rate limiting
  3. Session tokens that require memory of complex strings
- **Impact**: Users with cognitive disabilities (dyslexia 15-20% of population, ADHD 8-10% of adults) cannot authenticate.
- **Fix**: Support WebAuthn/passkey authentication as primary method. Never use CAPTCHA. Provide biometric or device-based auth alternatives.

### NEW DEFECT D-655-10: No accessibility regression testing in CI
- **Problem**: 2026 standard practice is axe-core in CI pipeline catching critical/serious violations. NeoTrix CI runs `cargo check`, `cargo test`, `npm test` but:
  1. No automated accessibility scanning of web output
  2. No keyboard navigation testing
  3. No screen reader testing workflow
  4. No WCAG conformance level tracking
- **Impact**: Accessibility regressions ship silently. Legal exposure increases with each release.
- **Fix**: Add `axe-core` integration to web build pipeline. Add keyboard-only test suite. Track WCAG 2.2 AA conformance as a build artifact.

---

## Cross-Domain Synthesis: Meta-Defects

### NEW DEFECT D-655-11: Symbolic-subsymbolic bridge still broken (confirmed from Batch 654)
- **Reinforcement**: CHI '26 shows HCI field shifting to agentic delegation (subsymbolic/LLM-based) while design systems formalize token contracts (symbolic). NeoTrix has no bridge between these. The VSA HyperCube (symbolic) cannot ingest or represent the continuous, probabilistic nature of LLM agent outputs. GWT cannot route between symbolic knowledge and subsymbolic generation.

### NEW DEFECT D-655-12: No purposeful forgetting mechanism (confirmed from Batch 654)
- **Reinforcement**: Design systems 2026 have **deprecation policies** (codemod + 6-month window). NeoTrix has no analogous mechanism for knowledge deprecation. KB entries accumulate without expiry. ConsciousnessTree branches never retire. This is the architectural correlate of "no purposeful forgetting."

### NEW DEFECT D-655-13: No aligned-mediation metric (confirmed from Batch 654)
- **Reinforcement**: Accessibility testing reveals that automated tools catch only 30-40% of issues — the rest requires human judgment (screen reader testing, cognitive accessibility assessment). NeoTrix has no metric for how well its outputs are **mediated** for human understanding. The GWT salience score measures internal attention but not external communication quality.

---

## Summary Table

| ID | Domain | Defect | Severity | Source |
|----|--------|--------|----------|--------|
| D-655-1 | HCI | No agentic-delegation interaction model in NT-IO | HIGH | IEEE CS 2026, CHI '26 |
| D-655-2 | HCI | ConsciousnessTree branches lack self-critique | MEDIUM | Grønbæk et al. CHI '26 |
| D-655-3 | HCI | No hallucination-as-cognitive-friction metric | MEDIUM | AlpCHI 2026 workshop |
| D-655-4 | UX | Design tokens lack motion/voice/emotion dimension | HIGH | CreativeAlive 2026, MDX 2026 |
| D-655-5 | UX | No component contribution ladder or deprecation protocol | MEDIUM | CreativeAlive 2026 |
| D-655-6 | UX | Documentation drift not addressed at system level | LOW | CreativeAlive 2026 |
| D-655-7 | A11y | No ARIA live region architecture | HIGH | Samkov 2026, WCAG 2.2 |
| D-655-8 | A11y | No keyboard-navigable consciousness dashboard | HIGH | WCAG 2.2 SC 2.4.11 |
| D-655-9 | A11y | No accessible authentication for KB access | MEDIUM | WCAG 2.2 SC 3.3.8 |
| D-655-10 | A11y | No accessibility regression testing in CI | HIGH | axe-core standard practice |
| D-655-11 | Cross | Symbolic-subsymbolic bridge broken | CRITICAL | Batch 654 + CHI '26 |
| D-655-12 | Cross | No purposeful forgetting mechanism | CRITICAL | Batch 654 + design deprecation |
| D-655-13 | Cross | No aligned-mediation metric | CRITICAL | Batch 654 + a11y testing gap |

---

## Sources Cited

1. IEEE Computer Society SBC UEMK, "Top HCI Trends in 2026: The Rise of AI Agents" (2026-07-25) — https://edu.ieee.org/in-uemk-cs/blog/2026/07/25/top-hci-trends-in-2026-the-rise-of-ai-agents/
2. Grønbæk et al., "How Do Future Visions Shape the Field of HCI?", CHI '26 (2026-04-13) — https://doi.org/10.1145/3772318.3791038
3. Sánchez-Vaquerizo & Monsivais, "From Particles to Agents: Hallucination as a Metric for Cognitive Friction", AlpCHI 2026 — arXiv:2601.21977
4. CreativeAlive, "Design Systems in 2026: From Component Libraries to Motion Tokens" (2026-05-09) — https://creativealive.com/design-systems-2026-component-libraries-motion-tokens/
5. MDX, "UI UX Design Trends (2026): What's Actually Changing" (2026-06-28) — https://mdx.so/blog/ui-ux-design-trends-2026-whats-actually-changing
6. Samkov, "Web Accessibility 2026: WCAG 2.2 & Screen Readers" (2026-04-19) — https://samcheek.com/blog/web-accessibility-a11y-guide-2026
7. W3C, "Web Content Accessibility Guidelines (WCAG) 2.2" — https://www.w3.org/TR/WCAG22/
8. W3C WCAG 2 Changelog (2026-05-05) — https://www.w3.org/WAI/standards-guidelines/wcag/changelog/
9. AccessiTool, "WCAG Screen Reader Requirements — Complete Guide 2026" (2026-06-25) — https://www.accessitool.com/blog/wcag-screen-reader-requirements-complete-guide-2026
10. TheWCAG, "Accessibility Testing Guide 2026" — https://www.thewcag.com/testing-guide
