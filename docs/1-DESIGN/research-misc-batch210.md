# Research Misc Batch 210 — 2026-09-10

## 1. Perplexity Q2D Web
- **URL**: https://www.perplexity.ai/hub/blog/q2d-web
- **Status**: 403 Forbidden — access blocked by Perplexity
- **Core Contribution**: Unknown (inaccessible)
- **Absorbable Pattern**: N/A
- **NeoTrix Mapping**: N/A
- **Priority**: Skip

## 2. Trendshift.io
- **URL**: https://trendshift.io/
- **Core Contribution**: Alternative to GitHub Trending — catches repos as they rise, not after they peak. Live mentions + momentum ranking for daily/weekly/monthly/yearly trends. 150K+ monthly visitors.
- **Absorbable Pattern**: **Momentum-based discovery** — signal before mainstream adoption. Track velocity (mentions/bookmarks) not just stars. Live mentions as leading indicator of project trajectory.
- **NeoTrix Mapping**: NT-WORLD (perception) — potential signal source for emerging open-source patterns. NT-MIND (evolution) — observe what tools/patterns the ecosystem absorbs.
- **Priority**: P2 (signal source for market intelligence)

## 3. geojson-vt (Mapbox)
- **URL**: https://github.com/mapbox/geojson-vt/
- **Core Contribution**: High-performance JS library for slicing GeoJSON into vector tiles on-the-fly in the browser. No server needed. Handles 100MB+ datasets (5.4M points) with simplification per zoom level.
- **Absorbable Pattern**: **On-the-fly spatial indexing** — build index once, query lazily. Progressive simplification (detail at zoom, discard tiny features). Zero-copy raw tile access (`getTileRaw`).
- **NeoTrix Mapping**: NT-WORLD (perception) — spatial data slicing pattern applicable to KB chunking. NT-MEMORY — index-then-lazy-query for large knowledge bases.
- **Priority**: P2 (elegant spatial indexing pattern)

## 4. ripwire (Red Hat)
- **URL**: https://github.com/redhat-et/ripwire
- **Core Contribution**: "The ripgrep of AI context" — zero-dependency C++23 CLI + MCP server. Builds ranked deterministic call graph for coding agents. 58.3% top-10 retrieval (vs 40% best competitor). Indexes 0.25s/6.6MB vs 46.8s/391MB for graph-database alternative. 5% of naive grep-and-read token cost.
- **Absorbable Pattern**: **Terminality** — one question, one complete answer. Confidence-gated routing across 3 lanes (name-exact / subtoken+body / mention-anchor). Honest truncation disclosure. Token-budgeted output. 6 independent quality evidence families (McCabe, Halstead, Nagappan, etc.) with cross-correlation <0.168.
- **NeoTrix Mapping**: NT-CORE (GWT attention routing — confidence-gated lane selection). NT-MEMORY (graph-ranked retrieval, indexed call graph). NT-ACT (agent context compression — 5% token cost). NT-MIND (quality panel — multi-evidence review).
- **Priority**: P0 (directly applicable to NeoTrix's codebase exploration and agent orchestration)

## 5. lieflat-gongwen
- **URL**: https://github.com/larashero3-dotcom/lieflat-gongwen
- **Core Contribution**: 102万字 corpus distilled into quantified government document writing skill. 7 writing styles with measurable parameters (sentence length, punctuation density, heading structure). Counter-intuitive findings: 60-72% of good gov docs use zero percentages, 75-87% use natural endings (not formulaic closings).
- **Absorbable Pattern**: **Corpus-to-parameter distillation** — statistics from real expert work > assumed rules. "Bad articles are neat, good articles are uneven" — parameter ranges not mean-based合格线. Self-check script only flags hard conflicts (文种错位), not parameter deviation. Synthetic example audit loop (10 rewritten, 4 kept).
- **NeoTrix Mapping**: NT-MIND (distillation methodology — corpus→stats→skill→self-check). NT-IO (skill template pattern — SKILL.md + parameter cards + reference corpus). NT-MEMORY (experience as quantified parameters, not prose rules).
- **Priority**: P1 (exemplary skill distillation methodology — how to turn domain expertise into agent-usable quantified knowledge)

## 6. LinearAbilityCastingExtendedThreeJS
- **URL**: https://github.com/achrefelouafi/LinearAbiltyCastingExtendedThreeJS
- **Core Contribution**: Elemental VFX sandbox — 10 abilities with hand-written GLSL, 2,261 live sliders + 424 color pickers. All parameters editable while paused. Zero textures — procedural geometry, SDF shaders, raymarched volumes, GPU particles. Performance budget: max 4 concurrent casts.
- **Absorbable Pattern**: **Settings-as-API** — every tweakable value in one config object, shaders read it every frame, live editing without rebuild. Capture events not dimensions (timestamps, not positions). Hash-based geometry rebuild only when parameters change. Pooled abilities (type-level instance cap).
- **NeoTrix Mapping**: NT-PHYSICAL (procedural generation pattern — no disk textures, everything derived from parameters). NT-ACT (live parameter editing pattern for agent-tunable systems). NT-IO (editor pattern — lil-gui style real-time control panels).
- **Priority**: P2 (impressive procedural generation + live parameter architecture, but niche domain)

## 7. Kudu
- **URL**: https://github.com/adventdevinc/kudu
- **Core Contribution**: Free, open-source system cleaner & security scanner for Windows/Mac/Linux. 13+ cleaner categories (system/browser/app/gaming/registry), malware scanner with heuristic analysis, privacy shield (30+ Windows settings), CLI mode. 30 languages. JSON-based cleaning rules (no code needed).
- **Absorbable Pattern**: **Declarative cleanup rules** — JSON-defined cleaning rules separate from engine. CLI-first design. Local-only by default (opt-in cloud). Risk-graded deletion with restore points.
- **NeoTrix Mapping**: NT-SHIELD (cleanup orchestration pattern — scan→assess→delete with JSON rule engine). NT-WORLD (system scanning — multi-category cache detection).
- **Priority**: P2 (relevant to cleanup subsystem but NeoTrix already has own implementation)

## 8. i-have-adhd
- **URL**: https://github.com/ayghri/i-have-adhd
- **Core Contribution**: 36.3k-star skill that reformats agent output for ADHD-friendly consumption. 10 rules: lead with action, number steps, end with concrete next step, suppress tangents, restate state, specific time estimates, no preamble/recap. Works across Claude Code, Cursor, Codex, Windsurf, opencode.
- **Absorbable Pattern**: **Action-first output protocol** — state → next action → concrete step. Suppress preamble, tangents, closers. Number multi-step tasks. Rank long lists (max 5/group). Specific time estimates (minutes, not "a bit"). Every output ends with one concrete next step.
- **NeoTrix Mapping**: NT-IO (output formatting contract — agent response structure). NT-CORE (GWT attention — reduce noise in agent broadcasts).
- **Priority**: P1 (directly applicable to NeoTrix agent output quality — universal pattern for concise agent responses)

## 9. Mistral Legacy Code Modernization
- **URL**: https://mistral.ai/news/legacy-code-modernization/
- **Core Contribution**: European energy operator migrated 40K lines Fortran 77 → C++ (300K total codebase). Key lessons: (1) Build parity harness BEFORE migration (numerical agreement as proof), (2) Documentation-first — agents can't migrate unreadable code, (3) Structured workflows with human review gates beat full autonomy. Three agent modes tested: autonomous (poor quality), planner-coder-tester-reviewer (better but stalls), human-operated module-by-module (optimal).
- **Absorbable Pattern**: **Parity harness before migration** — dump state snapshots from legacy, verify C++ outputs match numerically. Caller-callee tree as module decomposition unit. Skill.md files to steer agents. Reviewer agent on cron for PR review loops. Middle-ground autonomy: structured agent workflow + human checkpoints.
- **NeoTrix Mapping**: NT-MIND (migration methodology — structured agent workflows). NT-ACT (parity harness pattern for code verification). NT-CORE (ConsciousnessTree — human-in-the-loop checkpoints).
- **Priority**: P0 (directly applicable to NeoTrix's own legacy integration and SEAL pipeline automation)

## 10. ScienceDirect S0092867426009426
- **URL**: https://www.sciencedirect.com/science/article/pii/S0092867426009426
- **Status**: 403 Forbidden — access blocked by ScienceDirect
- **Core Contribution**: Unknown (inaccessible)
- **Absorbable Pattern**: N/A
- **NeoTrix Mapping**: N/A
- **Priority**: Skip

## 11. Drosophila Connectome Sexual Dimorphism (PMC)
- **URL**: https://pmc.ncbi.nlm.nih.gov/articles/PMC12636603/
- **Core Contribution**: Complete connectome of Drosophila male central nervous system — sexual dimorphism mapped at synapse resolution. Janelia/Google Research collaboration (50+ authors). First whole-brain connectome showing sex-specific wiring differences. Preprint (bioRxiv 2025.10).
- **Absorbable Pattern**: **Complete graph mapping** — exhaustive node-by-node reconstruction of a biological neural network. Sexual dimorphism as structural modification of base connectome (not separate graphs). Community-scale collaboration with standardized annotation pipeline.
- **NeoTrix Mapping**: NT-CORE (E8 hexagram — biological neural network as analogy for consciousness architecture). NT-MEMORY (knowledge graph reconstruction — exhaustive entity-relationship mapping). NT-MIND (dimorphism as pattern: base structure + modifier, not separate systems).
- **Priority**: P2 (inspirational for NeoTrix's consciousness architecture — biological connectome as design reference)

---

## Priority Summary

| Priority | Source | Why |
|----------|--------|-----|
| **P0** | ripwire | Directly applicable: confidence-gated routing, token compression, quality panel |
| **P0** | Mistral Fortran→C++ | Migration methodology: parity harness, documentation-first, structured agent workflows |
| **P1** | i-have-adhd | Universal agent output protocol — action-first, numbered steps, concrete next |
| **P1** | lieflat-gongwen | Exemplary distillation: corpus→quantified parameters→skill→self-check |
| **P2** | Trendshift.io | Momentum-based discovery signal for market intelligence |
| **P2** | geojson-vt | On-the-fly spatial indexing pattern |
| **P2** | LinearAbilityCastingExtendedThreeJS | Settings-as-API + procedural generation architecture |
| **P2** | Kudu | Declarative cleanup rules (JSON-based) |
| **P2** | Drosophila connectome | Biological neural network as consciousness architecture reference |
| Skip | Perplexity Q2D, ScienceDirect | 403 access blocked |

## Cross-Source Patterns

| Pattern | Sources | NeoTrix Integration |
|---------|---------|---------------------|
| **Confidence-gated routing** | ripwire (3-lane router) | GWT attention routing with salience+cost weights |
| **Parity harness before migration** | Mistral Fortran→C++ | SEAL pipeline: verify before absorb |
| **Action-first output** | i-have-adhd | NT-IO agent response contract |
| **Corpus→quantified skill** | lieflat-gongwen | NT-MIND distillation methodology |
| **Declarative rules engine** | Kudu (JSON rules), ripwire (LINEAGE.md) | NT-SHIELD cleanup rules |
| **Honest truncation disclosure** | ripwire | Agent output honesty (label floor, not false zero) |
