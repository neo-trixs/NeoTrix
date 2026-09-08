# Iteration Batch 614 — Game Engine / Physics / ECS Scan

**Date:** 2026-09-06
**Prior:** Batch 613 (zstd q1 compression, shared dictionaries 80-84%, tANS redundancy O(σ/n), XET privacy dedup, FAM-ANS sub-entropy text)

---

## 1. Game Engines (2026 State)

### 1.1 New: Box3D by Erin Catto (June 30, 2026)
- **What:** Open-source MIT-licensed 3D physics engine written in C17 (not C++) by the creator of Box2D.
- **Why it matters for NeoTrix:** Fills the WASM-feasible 3D physics gap. At ~916KB release binary, it is the smallest cross-platform deterministic 3D physics engine. Cross-platform determinism across thread counts and platforms — explicitly rejects `-ffast-math`. Already adopted by Facepunch Studios (s&box successor to Garry's Mod).
- **Defect vs batch 613:** Batch 613 had no physics engine landscape entry. Box3D creates a 3-way split in the open-source 3D physics market: Jolt (C++, mature, AAA), Rapier (Rust, WASM-first), Box3D (C17, smallest binary, determinism-first). NeoTrix's NT-PHYSICAL layer should document this triad.
- **Source:** [byteiota.com/box3d](https://byteiota.com/box3d-the-open-source-3d-physics-engine-built-for-games/), [HN thread](https://news.ycombinator.com/item?id=48745445)

### 1.2 Godot 4.6/4.7 — Jolt Physics as Default
- **What:** Godot 4.6 (Jan 2026) made Jolt physics the default, closing the 3D performance gap with Unity/Unreal. Godot 4.7 includes built-in Jolt integration.
- **Defect vs batch 613:** No prior batch tracked Godot's physics engine migration. This is architecturally significant — Jolt adoption signals the end of PhysX dominance in indie engines. NeoTrix should note: Jolt is now the de-facto open-source 3D physics standard.
- **Source:** [youngju.dev](https://www.youngju.dev/blog/culture/2026-05-15-game-engines-2026-godot-unity-bevy-unreal-defold-stride-comparison-deep-dive.en), [generalistprogrammer.com](https://generalistprogrammer.com/tutorials/game-development-engines-2025)

### 1.3 Bevy 0.18/0.19 — ECS Maturation
- **What:** Bevy 0.18 (Feb 2026) brings required components, improved scenes, better editor tooling. Bevy 0.19 (June 2026) is current. 18.5K GitHub stars. Bevy is the #1 most-searched Rust game engine (Google Trends peak 100 in Jan 2026).
- **Defect vs batch 613:** Bevy's archetype-based ECS now has formal `Access` bitset tracking for parallelism — the `component_read_and_writes`, `component_writes`, and `archetypal` fields enable fine-grained conflict detection that batch 613's ECS analysis lacked.
- **Source:** [bevy_ecs docs](https://docs.rs/bevy_ecs/latest/bevy_ecs/), [taintedcoders.com/bevy/ecs](https://taintedcoders.com/bevy/ecs)

### 1.4 Unity 6 — ECS 1.5 + GPU Resident Drawer
- **What:** Unity 6 adds GPU Resident Drawer, Render Graph, BIRP/URP/HDRP convergence work, and ECS 1.5. Steam market share stabilized at 35-38% (down from 45% peak in 2023 but not collapsing).
- **Defect vs batch 613:** No prior batch tracked Unity's ECS versioning. ECS 1.5 represents the DOTS maturation point — Unity is now the largest ECS deployment in production games.
- **Source:** [youngju.dev](https://www.youngju.dev/blog/culture/2026-05-16-open-source-game-engines-2026-godot-4-4-defold-bevy-gamemaker-phaser-construct-stride-pixijs-deep-dive.en), [tech-insider.org](https://tech-insider.org/best-game-engines-for-indie-devs-2026/)

---

## 2. Physics Engines

### 2.1 Rapier 0.32-0.35 — GPU Physics Roadmap
- **What:** Dimforge's 2025 review reveals:
  - **New BVH** with SIMD-accelerated tree traversal, replacing Hierarchical Sweep-and-Prune for both scene queries and broad-phase.
  - **Sparse voxel collider** — first general-purpose rigid-body engine to support voxels explicitly (lower memory, no ghost collisions).
  - **Persistent islands** — avoids re-extraction of collision graph connected components each frame.
  - **Simplified 3D friction model** — 25% speedup on contact-heavy scenes.
  - **SIMD-accelerated WASM packages** — 2-5x faster than 2024 versions on web.
  - **glam migration** — API now uses `glam` instead of `nalgebra` (enables rust-gpu backend).
  - **2026 roadmap:** robotics (Mujoco-inspired accuracy) + GPU physics via `rust-gpu`.
- **Defect vs batch 613:** Batch 613 had no GPU physics roadmap for Rapier. The WGSL→Slang→rust-gpu evolution is a critical signal: GPU-accelerated physics is becoming a Rust ecosystem reality. NeoTrix's NT-PHYSICAL should track this for potential integration.
- **Source:** [dimforge.com/blog/2026/01/09/the-year-2025-in-dimforge](https://dimforge.com/blog/2026/01/09/the-year-2025-in-dimforge)

### 2.2 Rapier 0.35 vs Box2D 3.1 vs Box3D — Godot Benchmarks
- **What:** Aug 2026 Godot benchmark comparing Rapier 0.35, Box2D 3.1, Box3D, and Jolt (Godot 4.7 built-in):
  - **2D:** Rapier is the only engine that keeps every scene stable. Box2D collapses pyramids and blows joints apart (marked "unstable"). Rapier uses 2.3x fewer cores.
  - **3D:** Jolt (Godot built-in) vs Rapier vs Box3D all compile. Box3D is alpha.
  - **New features:** Per-body solver iterations, per-joint stiffness/toggling for breakable constraints, rope and fixed joints in 2D/3D (Godot has no equivalent).
- **Defect vs batch 613:** No prior batch had cross-engine benchmark data. The instability of Box2D 3.1 on complex constraints is a critical finding — Rapier's constraint solver is significantly more robust.
- **Source:** [forum.godotengine.org](https://forum.godotengine.org/t/physics-engine-comparison-rapier-vs-godot-vs-box2d-3d-vs-jolt/142786)

### 2.3 Box3D — Technical Details
- **What:** C17, MIT license, cross-platform determinism, ~916KB binary, WASM-feasible. Features: continuous collision detection, convex hulls/capsules/spheres/triangle meshes/height fields, revolute/prismatic/distance/wheel/weld joints, double-precision large-world support, multithreaded contact solver with SIMD + graph coloring.
- **Defect vs batch 613:** Box3D explicitly rejects `-ffast-math` for determinism. This is a design philosophy that NeoTrix should adopt for any simulation-critical code paths.
- **Source:** [byteiota.com](https://byteiota.com/box3d-the-open-source-3d-physics-engine-built-for-games/)

---

## 3. ECS Architecture

### 3.1 Bevy ECS Access Tracking — Bitset Conflict Detection
- **What:** Bevy's `Access` struct tracks `component_read_and_writes`, `component_writes`, `component_read_and_writes_inverted`, `component_writes_inverted`, and `archetypal` bitsets. This enables:
  - Fast compatibility checks (`is_compatible`)
  - Conflict detection (`get_conflicts`)
  - Inverted access patterns (read-all-except / write-all-except)
  - Archetypal awareness (component presence affects query results even without direct access)
- **Defect vs batch 613:** Batch 613's ECS analysis was conceptual. This is concrete implementation detail: Bevy uses FixedBitSet for O(1) conflict detection. NeoTrix's NT-CORE consciousness tree could adopt similar bitset tracking for module dependency conflict detection.
- **Source:** [taintedcoders.com/bevy/ecs](https://taintedcoders.com/bevy/ecs), [docs.rs/bevy_ecs](https://docs.rs/bevy_ecs/latest/bevy_ecs/)

### 3.2 ECS Paradigm Shift — Inheritance Era to Data-Oriented Era
- **What:** The game industry transition: 1995-2015 was inheritance-based (MonoBehaviour, UObject, Node), 2015+ is data-oriented (ECS). Naughty Dog, Insomniac, Epic's Mass system in Unreal 5 all conceded "a cache miss is a frame drop."
- **Defect vs batch 613:** NeoTrix's own architecture uses a hybrid approach. The industry data confirms ECS is the performance-optimal pattern for entity-heavy systems. NeoTrix's `nt_core_self` module should evaluate whether its current OOP-style component model should migrate toward archetype-based storage.
- **Source:** [youngju.dev](https://www.youngju.dev/blog/culture/2026-05-14-bevy-game-engine-rust-hands-on-ecs-paradigm-modern-gamedev-deep-dive-2026.en)

### 3.3 Bevy Anti-Patterns (New Defect Class)
- **What:** Documented anti-patterns:
  1. Carrying `Vec<Entity>` through systems and doing lookups every frame
  2. Modeling everything as a component (state machines should be enums)
  3. Ignoring system ordering (parallel default = undefined order for same-component writes)
  4. Misunderstanding asset loading timing
- **Defect vs batch 613:** These anti-patterns have analogues in NeoTrix's own codebase. Anti-pattern #3 (system ordering) maps directly to NeoTrix's EventBus race condition potential — two systems modifying same EventBus topics without explicit ordering constraints.
- **Source:** [youngju.dev](https://www.youngju.dev/blog/culture/2026-05-14-bevy-game-engine-rust-hands-on-ecs-paradigm-modern-gamedev-deep-dive-2026.en)

---

## 4. Cross-Domain Defects Found

| ID | Defect | Domain | Severity |
|----|--------|--------|----------|
| D-614-1 | No GPU physics roadmap tracked for NeoTrix NT-PHYSICAL | Physics | Medium |
| D-614-2 | Box2D 3.1 constraint instability (pyramid collapse) not documented | Physics | Low |
| D-614-3 | Bevy Access bitset pattern applicable to NeoTrix module conflict detection | ECS | Medium |
| D-614-4 | EventBus race condition analogous to ECS system ordering anti-pattern | Architecture | High |
| D-614-5 | Box3D `-ffast-math` rejection philosophy not adopted in NeoTrix build rules | Build | Medium |
| D-614-6 | Unity ECS 1.5 deployment scale (35-38% Steam) not tracked | ECS | Low |
| D-614-7 | Rapier glam→rust-gpu path enables CPU/GPU code sharing — NeoTrix missing this pattern | Physics | Medium |

---

## 5. Sources Cited

1. [Game Engines 2026 Deep Comparison](https://www.youngju.dev/blog/culture/2026-05-15-game-engines-2026-godot-unity-bevy-unreal-defold-stride-comparison-deep-dive.en) — May 2026
2. [The 2026 State of Game Engines](https://www.gamineai.com/blog/the-2026-state-of-game-engines-unity-unreal-godot-and-the-rest) — Feb 2026
3. [Best Game Engines 2026 Compared](https://tech-insider.org/best-game-engines-for-indie-devs-2026/) — Aug 2026
4. [Box3D Announcement](https://byteiota.com/box3d-the-open-source-3d-physics-engine-built-for-games/) — Jul 2026
5. [Dimforge 2025 Year in Review](https://dimforge.com/blog/2026/01/09/the-year-2025-in-dimforge) — Jan 2026
6. [Rapier vs Box2D vs Box3D vs Jolt Godot Benchmark](https://forum.godotengine.org/t/physics-engine-comparison-rapier-vs-godot-vs-box2d-3d-vs-jolt/142786) — Aug 2026
7. [Bevy ECS Deep Dive](https://taintedcoders.com/bevy/ecs) — Jun 2026
8. [Bevy Game Engine Rust Hands-On](https://www.youngju.dev/blog/culture/2026-05-14-bevy-game-engine-rust-hands-on-ecs-paradigm-modern-gamedev-deep-dive-2026.en) — May 2026
9. [Bevy ECS Docs](https://docs.rs/bevy_ecs/latest/bevy_ecs/) — Current
10. [Open Source Game Engines 2026](https://www.youngju.dev/blog/culture/2026-05-16-open-source-game-engines-2026-godot-4-4-defold-bevy-gamemaker-phaser-construct-stride-pixijs-deep-dive.en) — May 2026
11. [Box2D vs Rapier vs Matter.js Browser Benchmarks](https://rozgames.com/article/physics-engines-browser-games-tested) — Dec 2025
12. [Bevy 0.18 Guide](https://rustify.rs/articles/rust-for-game-development-bevy-2026) — 2026
