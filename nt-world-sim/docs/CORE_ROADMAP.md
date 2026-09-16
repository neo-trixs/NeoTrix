# NT-WORLD-SIM Core Roadmap

> Generated: 2026-09-14
> Baseline: 91 Rust files, ~29.5K LOC Rust, ~57K LOC total (including HTML/JS)
> Target: Production-ready Xianxia cultivation RPG engine
> Horizon: 10 weeks

---

## Executive Summary

NT-WORLD-SIM is a consciousness-evolution simulation / cultivation RPG engine with a 6-layer architecture and dual-ECS foundation. The codebase compiles and has 532 passing tests, but suffers from **86K LOC of duplicated code** (30% of codebase), a **dual-ECS split** (core/ vs engine/), and **zero xianxia-specific systems** implemented (cultivation, sects, real-time combat). The 鬼谷八荒 reference game defines 10 cultivation realms, 12 weapon/element types, and deep NPC social systems that must be implemented.

**Current Maturity: C1** (Compiles + Basic Structure)

This roadmap consolidates findings from PROJECT_EVALUATION.md, ARCHITECTURE_REVERSE_ENGINEERING.md, GUIGUBAHUANG_RESEARCH.md, and ITERATION_REPORT.md into a 10-week execution plan across 5 phases.

---

## Current State Assessment

### Code Health

| Metric | Value | Target |
|--------|-------|--------|
| Rust files | 91 | <60 (after consolidation) |
| Rust LOC | ~29.5K | ~25K (after dedup) |
| Duplicated LOC | ~86K | 0 |
| Dead docs in src/ | ~180K | 0 |
| Test functions | 553 | 1000+ |
| Test coverage | ~15-20% | 60%+ |
| Compiler warnings | 18 | 0 |
| Passing tests | 532 | 532+ |
| Failing tests | 1 (race condition) | 0 |

### Architecture Gaps

| Gap | Severity | Status |
|-----|----------|--------|
| Dual ECS (core/ vs engine/) | Critical | Partially addressed in iteration |
| GameState duplication (3 enums) | High | Not started |
| 7 duplicate system pairs | High | Not started |
| Engine monolith (18K LOC) | Medium | Not started |
| No xianxia systems | High | Not started |
| No multiplayer | Medium | Not started |
| No modding support | Low | Not started |

### 鬼谷八荒 Feature Parity

| Feature | 鬼谷八荒 | NT-WORLD-SIM | Gap |
|---------|----------|--------------|-----|
| Cultivation (10 realms × 3 sub-stages) | ✅ | ❌ | Complete |
| Breakthrough (Human/Earth/Heaven paths) | ✅ | ❌ | Complete |
| Fate system (逆天改命) | ✅ | ❌ | Complete |
| 12 weapon/element types | ✅ | 🟡 5 skills | Partial |
| Sect system | ✅ | ❌ | Complete |
| NPC personality (internal + 2 external) | ✅ | ❌ | Complete |
| NPC relationships (7 tiers) | ✅ | ❌ | Complete |
| Real-time combat | ✅ | 🟡 Turn-based | Different |
| Artifact forging | ✅ | ❌ | Complete |
| Alchemy/talisman | ✅ | 🟡 Recipes only | Partial |
| Mod support | ✅ | ❌ | Complete |

---

## Architecture Goals

### Target Architecture (from ARCHITECTURE_REVERSE_ENGINEERING.md)

```
┌─────────────────────────────────────────────────────────┐
│                    L6: Tooling Layer                     │
│  Visual Editor / Mod API / Debug Console / Inspector    │
├─────────────────────────────────────────────────────────┤
│                    L5: Game Logic Layer                  │
│  Cultivation / Sect / Dialogue / Quest / NPC AI         │
├─────────────────────────────────────────────────────────┤
│                    L4: Framework Layer                   │
│  ECS Core / Event Bus / Tag System / Plugin System      │
├─────────────────────────────────────────────────────────┤
│                    L3: Subsystem Layer                   │
│  Tilemap / Rendering / Physics / Audio / Input / Net    │
├─────────────────────────────────────────────────────────┤
│                    L2: Platform Layer                    │
│  Window / Filesystem / Threading / GPU Abstraction      │
├─────────────────────────────────────────────────────────┤
│                    L1: OS/Hardware Layer                 │
│  Memory Management / SIMD / Multithreading / GPU        │
└─────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale | Source |
|----------|-----------|--------|
| Archetype ECS (SoA per chunk) | Cache-optimal iteration, proven at scale | Unity DOTS, Bevy, academic benchmarks |
| Auto-cleanup event channels | Zero manual cleanup, no stale references | Godot signals, Bevy events |
| Hierarchical tag system | Composable state, replaces FSM/BT | Unreal Gameplay Tags |
| Plugin architecture | Modular composition, independent systems | Bevy Plugin trait |
| Data-driven config (YAML/TOML) | Tune without recompilation | Bevy serde, Godot Resources |
| Node-based dialogue + storylets | Reusable dialogue segments, saliency-based | Yarn Spinner 3.0, DialogueManager |

---

## Feature Priorities

### P0: Must Ship (Weeks 1-4)
1. ECS consolidation (single canonical ECS)
2. Duplicate system elimination
3. Cultivation system (10 realms × 3 sub-stages)
4. Breakthrough mechanics (risk/reward, 3 paths)
5. Sect system (join, contribute, rank)
6. Playable demo (move → interact → cultivate → fight → save)

### P1: Should Ship (Weeks 5-8)
7. Real-time combat (WASD + skills)
8. NPC personality + relationship system
9. Artifact forging
10. 5 complete maps
11. 20+ NPCs with dialogue
12. 30+ quests
13. Ink-wash visual style
14. Multiplayer prototype

### P2: Nice to Have (Weeks 9-10)
15. Modding support (JSON config + event hooks)
16. Accessibility features
17. Performance optimization (60 FPS, 1000+ entities)
18. Localization framework
19. Mod marketplace

---

## Technical Debt

### Debt Inventory

| # | Debt Item | LOC Impact | Priority | Fix Effort |
|---|-----------|-----------|----------|------------|
| D1 | Dual ECS (`core/` vs `engine/`) | ~600 | P0 | 2 days |
| D2 | 7 duplicate system pairs | ~86K | P0 | 5 days |
| D3 | 180K dead docs in `src/engine/` | ~180K | P0 | 0.5 days |
| D4 | GameState triple duplication | 3 enums | P0 | 1 day |
| D5 | Engine monolith (18K LOC) | 18K | P1 | 3 days |
| D6 | No test coverage in game systems | 0% | P1 | 3 days |
| D7 | 18 compiler warnings | 18 | P1 | 0.5 days |
| D8 | Camera test race condition | 1 test | P1 | 1 day |
| D9 | Hardcoded game content | 30 recipes, 27 quests | P1 | 3 days |
| D10 | No save/load integration | 0% | P1 | 2 days |

### Deduplication Strategy (P0-2)

| System | Keep | Delete | Merge Into |
|--------|------|--------|------------|
| ECS | `core/` | `engine/ecs/` | `core/` |
| Combat | `engine/combat.rs` (30K) | `game/combat.rs` (7.8K) | `engine/combat.rs` |
| Inventory | `engine/inventory.rs` (18K) | `game/inventory.rs` (6.4K) | `engine/inventory.rs` |
| Dialogue | `engine/dialogue.rs` (29K) | `game/dialogue.rs` (2K) | `engine/dialogue.rs` |
| Quest | `game/quest.rs` (18K) + `engine/quest.rs` (24K) | — | Merge both |
| Event Bus | `engine/event_bus.rs` (6.3K) | `game/events.rs` (14.7K) | `engine/event_bus.rs` |
| Skill | `engine/skill.rs` (17K) | `game/skill.rs` (5.5K) | `engine/skill.rs` |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| ECS migration breaks tests | High | High | Run tests after each migration step; keep `core/` as canonical |
| Cultivation system too complex | Medium | High | Start with 3 realms, expand after core loop works |
| Real-time combat integration fails | Medium | High | Use `engine/combat.rs` (30K) as base, not `game/` |
| Save/load incompatible after refactor | Medium | Medium | Version save format; add migration path |
| Performance regression from consolidation | Low | Medium | Benchmark before/after; use `cargo bench` |
| Scope creep into Unity/Unreal features | High | Medium | Focus on 鬼谷八荒 parity, not engine parity |
| Dual-culture (Chinese/English) confusion | Low | Low | All comments/docs in English; game text in Chinese |

---

## Success Metrics

| Metric | Week 2 | Week 4 | Week 6 | Week 8 | Week 10 |
|--------|--------|--------|--------|--------|---------|
| Rust files | <70 | <60 | <60 | <60 | <60 |
| Duplicated LOC | 0 | 0 | 0 | 0 | 0 |
| Dead docs in src/ | 0 | 0 | 0 | 0 | 0 |
| Test coverage | 30% | 40% | 50% | 60% | 60%+ |
| Compiler warnings | 0 | 0 | 0 | 0 | 0 |
| Cultivation realms | 3 | 10 | 10 | 10 | 10 |
| NPC count | 0 | 0 | 10 | 20 | 20+ |
| Quest count | 0 | 5 | 15 | 30 | 30+ |
| Skill count | 5 | 10 | 30 | 50 | 50+ |
| Map count | 1 | 2 | 3 | 5 | 5 |
| Playable demo | ❌ | ✅ | ✅ | ✅ | ✅ |
| 60 FPS (1000+ entities) | ❌ | ❌ | ✅ | ✅ | ✅ |

---

## Timeline with Milestones

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Eliminate architectural debt, establish single source of truth, create playable demo.

#### Week 1: Architecture Consolidation

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 1.1 | Delete `engine/ecs/` module, migrate to `core/UniversalWorld` | P0 | 2 | None |
| 1.2 | Consolidate 7 duplicate system pairs (combat, inventory, dialogue, quest, event bus, skill, ECS) | P0 | 5 | 1.1 |
| 1.3 | Move 180K design docs out of `src/engine/` to `docs/` | P0 | 0.5 | None |
| 1.4 | Unify 3 GameState enums into single `GameState` | P0 | 1 | 1.2 |
| 1.5 | Fix 18 compiler warnings (unused imports, dead code) | P1 | 0.5 | 1.2 |
| 1.6 | Fix camera test race condition | P1 | 1 | None |
| 1.7 | Add `#[cfg(test)]` modules to all game systems | P1 | 2 | 1.2 |

**Milestone 1.1:** Single ECS, zero duplicate systems, zero warnings, all 532+ tests passing.

#### Week 2: Core Systems

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 2.1 | Implement auto-cleanup event channels (Bevy pattern) | P0 | 2 | 1.1 |
| 2.2 | Implement hierarchical tag system (Unreal Gameplay Tags pattern) | P0 | 2 | None |
| 2.3 | Implement auto-tiling system (Godot Terrain pattern) | P1 | 2 | None |
| 2.4 | Externalize game data to YAML/TOML (items, recipes, quests) | P1 | 3 | 1.2 |
| 2.5 | Wire game loop: move → interact → farm → mine → fish → dialogue → save | P0 | 3 | 1.2, 2.1 |
| 2.6 | Create playable demo (cultivation stub + basic world) | P0 | 2 | 2.5 |

**Milestone 1.2:** Playable demo with single ECS, event bus, tag system, and basic world loop.

**Exit Criteria Phase 1:**
- [ ] Single ECS (no dual-implementation)
- [ ] Zero duplicate systems
- [ ] Zero compiler warnings
- [ ] 532+ tests passing
- [ ] Playable demo (move, interact, basic cultivation)
- [ ] Design docs separated from source

---

### Phase 2: Feature Completion (Weeks 3-4)

**Goal:** Implement xianxia core systems and complete game content.

#### Week 3: Xianxia Systems

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 3.1 | Implement cultivation system (10 realms × 3 sub-stages) | P0 | 5 | Phase 1 |
| 3.2 | Implement breakthrough system (Human/Earth/Heaven paths, risk/reward) | P0 | 3 | 3.1 |
| 3.3 | Implement fate system (逆天改命, 4-6 choices per breakthrough) | P1 | 2 | 3.2 |
| 3.4 | Implement sect system (join, contribute, rank, treasure pavilion) | P0 | 4 | 3.1 |
| 3.5 | Implement artifact system (法宝, forging minigame, 5 slots) | P1 | 3 | 3.1 |
| 3.6 | Implement dual cultivation system (道侣, co-op cultivation) | P2 | 2 | 3.1 |

**Milestone 2.1:** Complete cultivation progression from 练气→登仙 with breakthrough mechanics.

#### Week 4: Game Content

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 4.1 | Create 5 complete maps (one per province: 白原, 永宁, 雷泽, 华封, 云陌) | P1 | 5 | Phase 1 |
| 4.2 | Add 20+ NPCs with personality traits (internal + 2 external) | P1 | 4 | 3.4 |
| 4.3 | Add 30+ quests (main + side, tag-based state machine) | P1 | 4 | 3.1 |
| 4.4 | Add 50+ skills across 6 elements + 6 physical types | P1 | 3 | 3.1 |
| 4.5 | Add 20+ enemy types with AI | P1 | 3 | 3.1 |
| 4.6 | Wire save/load to all game systems | P1 | 2 | 1.2 |

**Milestone 2.2:** Complete game content: 5 maps, 20 NPCs, 30 quests, 50 skills, 20 enemies.

**Exit Criteria Phase 2:**
- [ ] 10 cultivation realms with 3 sub-stages each
- [ ] Breakthrough system with 3 quality paths
- [ ] Sect system with ranks and contributions
- [ ] 5 maps with distinct themes
- [ ] 20+ NPCs with dialogue
- [ ] 30+ quests (main + side)
- [ ] 50+ skills (12 weapon/element types)
- [ ] Save/load working

---

### Phase 3: Polish (Weeks 5-6)

**Goal:** Production-quality performance, visuals, and UX.

#### Week 5: Visual Polish

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 5.1 | Implement ink-wash painting style renderer (水墨画 aesthetic) | P1 | 4 | Phase 2 |
| 5.2 | Add particle effects for all skills (12 types) | P1 | 3 | Phase 2 |
| 5.3 | Add weather system (rain, snow, fog, lightning) | P1 | 2 | Phase 2 |
| 5.4 | Add day/night cycle with lighting | P1 | 2 | Phase 2 |
| 5.5 | Add ambient sound effects (combat, nature, UI) | P1 | 2 | Phase 2 |
| 5.6 | Implement sprite batch instanced rendering | P1 | 2 | Phase 2 |

**Milestone 3.1:** Visually polished game with ink-wash style, particles, weather, and sound.

#### Week 6: UI/UX Polish

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 6.1 | Redesign all UI panels (inventory, dialogue, quest log, map) | P1 | 3 | Phase 2 |
| 6.2 | Add tooltips and help text for all interactions | P1 | 1 | 6.1 |
| 6.3 | Add keyboard shortcuts (configurable) | P1 | 1 | 6.1 |
| 6.4 | Add gamepad support (Xbox/PS controllers) | P2 | 2 | 6.1 |
| 6.5 | Add accessibility features (colorblind, text scaling) | P2 | 2 | 6.1 |
| 6.6 | Add spatial partitioning for collision (grid/quadtree) | P1 | 2 | Phase 2 |
| 6.7 | Add crash reporting | P1 | 1 | Phase 2 |

**Milestone 3.2:** Polished UI/UX with accessibility, gamepad support, and 60 FPS performance.

**Exit Criteria Phase 3:**
- [ ] Ink-wash painting visual style
- [ ] Particle effects for all 12 skill types
- [ ] Weather system (4 types)
- [ ] Day/night cycle
- [ ] Ambient sound effects
- [ ] 60 FPS with 1000+ entities
- [ ] <100ms save/load
- [ ] Accessible UI (colorblind, text scaling)

---

### Phase 4: Advanced Features (Weeks 7-8)

**Goal:** Multiplayer, modding, and cross-platform.

#### Week 7: Multiplayer

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 7.1 | Implement WebSocket server (lobby, rooms) | P1 | 3 | Phase 3 |
| 7.2 | Add player synchronization (position, state) | P1 | 3 | 7.1 |
| 7.3 | Add chat system (global, sect, whisper) | P1 | 2 | 7.1 |
| 7.4 | Add party system (2-4 players) | P2 | 2 | 7.2 |
| 7.5 | Add PvP system (duels, sect wars) | P2 | 3 | 7.2 |
| 7.6 | Add NPC personality + relationship system (7 tiers) | P1 | 4 | Phase 2 |

**Milestone 4.1:** Working multiplayer with sync, chat, and party system.

#### Week 8: Modding Support

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 8.1 | Implement mod loader (JSON config + event hooks) | P2 | 3 | Phase 3 |
| 8.2 | Add mod API documentation | P2 | 2 | 8.1 |
| 8.3 | Create mod examples (3 mods) | P2 | 2 | 8.1 |
| 8.4 | Add mod validation (schema check, conflict detection) | P2 | 2 | 8.1 |
| 8.5 | Implement real-time combat (WASD + dodge + combos) | P1 | 5 | Phase 3 |
| 8.6 | Implement alchemy/talisman crafting | P1 | 3 | Phase 3 |

**Milestone 4.2:** Moddable game with mod loader, API docs, and real-time combat.

**Exit Criteria Phase 4:**
- [ ] WebSocket multiplayer (2-4 players)
- [ ] Chat system (3 channels)
- [ ] Party system
- [ ] PvP system
- [ ] Mod loader with JSON config
- [ ] Mod API documentation
- [ ] Real-time combat (WASD + dodge)
- [ ] Alchemy/talisman crafting
- [ ] NPC relationship system (7 tiers)

---

### Phase 5: Release (Weeks 9-10)

**Goal:** Testing, documentation, and launch.

#### Week 9: Testing

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 9.1 | Add unit tests for all systems (target 60%+ coverage) | P0 | 5 | All phases |
| 9.2 | Add integration tests (ECS + systems + game loop) | P0 | 3 | 9.1 |
| 9.3 | Add performance tests (1000+ entities, 60 FPS target) | P1 | 2 | 9.1 |
| 9.4 | Add security tests (save file validation, mod validation) | P1 | 2 | 9.1 |
| 9.5 | Fix all remaining bugs | P0 | 3 | 9.1-9.4 |

**Milestone 5.1:** All tests passing, 60%+ coverage, zero bugs.

#### Week 10: Launch

| # | Task | Priority | Est. Days | Dependencies |
|---|------|----------|-----------|--------------|
| 10.1 | Create release build (optimized binary) | P0 | 1 | 9.5 |
| 10.2 | Write release notes (features, known issues) | P0 | 1 | 10.1 |
| 10.3 | Create user documentation (getting started, controls) | P1 | 2 | 10.1 |
| 10.4 | Set up CI/CD (GitHub Actions, automated testing) | P1 | 2 | 10.1 |
| 10.5 | Launch on Steam (store page, build upload) | P0 | 2 | 10.1-10.4 |

**Milestone 5.2:** Production release on Steam.

**Exit Criteria Phase 5:**
- [ ] 60%+ test coverage
- [ ] All tests passing (unit + integration + performance)
- [ ] Zero critical bugs
- [ ] Release build optimized
- [ ] Release notes published
- [ ] User documentation complete
- [ ] CI/CD pipeline running
- [ ] Steam launch

---

## Resource Requirements

### Development Effort

| Phase | Weeks | Estimated Hours | Focus |
|-------|-------|----------------|-------|
| Phase 1: Foundation | 1-2 | 80-100 | Architecture cleanup, ECS consolidation |
| Phase 2: Feature Completion | 3-4 | 100-120 | Xianxia systems, game content |
| Phase 3: Polish | 5-6 | 80-100 | Visual polish, UI/UX, performance |
| Phase 4: Advanced | 7-8 | 100-120 | Multiplayer, modding, combat |
| Phase 5: Release | 9-10 | 60-80 | Testing, documentation, launch |
| **Total** | **10** | **420-520** | — |

### Skill Requirements

| Skill | Phase | Hours |
|-------|-------|-------|
| Rust systems programming | 1-5 | 200+ |
| ECS architecture design | 1-2 | 40 |
| Game systems (cultivation, combat, NPC) | 2-4 | 120 |
| UI/UX design | 3, 6 | 60 |
| WebSocket/networking | 7 | 40 |
| Testing/QA | 9 | 40 |
| Documentation | 10 | 20 |

### Tool Requirements

| Tool | Purpose |
|------|---------|
| Rust 1.75+ | Core language |
| Cargo | Build system |
| serde + serde_yaml | Data serialization |
| rayon | Parallel scheduling |
| tokio | Async runtime (multiplayer) |
| web-sys | WASM target (optional) |
| GitHub Actions | CI/CD |
| Steam SDK | Distribution |

---

## Dependencies

### External Dependencies

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| Rust | 1.75+ | Core language | ✅ Available |
| serde | 1.0+ | Serialization | ✅ Available |
| serde_yaml | 0.9+ | YAML config | ✅ Available |
| rayon | 1.8+ | Parallel iteration | ✅ Available |
| tokio | 1.0+ | Async runtime | ✅ Available |
| web-sys | 0.2+ | WASM (optional) | ⚠️ Optional |
| Steam SDK | Latest | Distribution | ❌ Not integrated |

### Internal Dependencies

| Dependency | Blocks | Status |
|------------|--------|--------|
| ECS consolidation (Phase 1) | All subsequent phases | 🟡 Partially done |
| Event bus (Phase 1) | Game loop, multiplayer | ❌ Not started |
| Tag system (Phase 1) | Quests, dialogue, combat | ❌ Not started |
| Cultivation system (Phase 2) | Breakthrough, sect, artifact | ❌ Not started |
| Game loop (Phase 2) | All game content | ❌ Not started |
| Rendering (Phase 3) | Visual polish | 🟡 Partially done |
| Save/load (Phase 2) | Multiplayer, modding | 🟡 Partially done |

---

## Appendix: 鬼谷八荒 Implementation Reference

### Cultivation Realm Breakthrough Materials

| Transition | Materials Required |
|-----------|-------------------|
| 练气 → 筑基 | 6 Qi Pearls + Sandbury Bone + Duskflower Dew + Millennium Stalactite |
| 筑基 → 结晶 | Breakthrough Pill + Region boss drops |
| 结晶 → 金丹 | 6 Spirit Pearls + 9 Colored Orchid + Icy Begonia + Mindsnapper + Crimson Vine |
| 金丹 → 具灵 | Spirit Assembly Pill + Dragon Bone Gold + Blood Stone + Earth Fire Stone + 悟心珠 |
| 具灵 → 元婴 | Fusion Pill + Heavenly Qi + Red Coral + Weak Water + 5 more treasures |
| 元婴 → 化神 | 5 Divine Souls + Spirit Transformation Qi |
| 化神 → 悟道 | 3 Dao Souls + Dao Comprehension Pill |
| 悟道 → 羽化 | Heavenly Essence Pills + Primordial Air |
| 羽化 → 登仙 | Ascension quest completion + 3 Primordial Air |

### NPC Personality Matrix

| Internal Trait | External Trait 1 | External Trait 2 | Effect |
|---------------|-----------------|-----------------|--------|
| 仁善 (Benevolent) | 义气 (Loyalty) | 天伦 (Family) | Bonus with friends + family |
| 无私 (Selfless) | 忠贞 (Devotion) | 护短 (Protective) | Bonus with sect + spouse |
| 邪恶 (Evil) | 睚眦 (Vengeful) | 权力 (Power) | Many enemies, power-hungry |

### Equipment Slots

- 1 Ring
- 1 Mount
- 5 Battle items (pills/artifacts, max 5 stacks each)
- 5 Artifact slots (from Forge)
- 1 Heart Skill slot per type (8 total types)

---

## Appendix: Cross-Engine Pattern Reference

### ECS Comparison

| Aspect | Unity DOTS | Bevy | NT-WORLD-SIM Target |
|--------|-----------|------|---------------------|
| Entity | ID + version | u64 + generation | u64 + generation |
| Component | `IComponentData` | `#[derive(Component)]` | Plain struct, no inheritance |
| System | `SystemBase.OnUpdate()` | `fn system()` | Stateless function |
| Storage | Archetype + Chunk (16 KiB) | Archetype + Table | Archetype + Table |
| Query | `EntityQuery` + filters | `Query<T, F>` | `Query<T, F>` with 3+ component support |
| Change Detection | Version numbers | `Changed<T>` / `Added<T>` | `Changed<T>` (wire into systems) |
| Events | Manual cleanup | Auto-cleanup | Auto-cleanup channels |
| State | Animator Controller | `States` trait | Global enum + per-entity FSM |
| Tags | Gameplay Tags | (none native) | Hierarchical tag system |

### Auto-Tile Algorithm

```
1. For each cell, examine 4/8 neighbors
2. Generate bitmask from neighbor similarity
3. Look up tile variant from bitmask table
4. Apply visual (sprite, rotation, flip)
5. Handle edge cases (corners, transitions)
```

---

*"The engine is a cathedral with beautiful blueprints but missing foundation bolts. This roadmap tightens every bolt."*
