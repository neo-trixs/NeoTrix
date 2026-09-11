# neotrix-sim Scan Report v2

**Date:** 2026-09-11
**Crate:** neotrix-sim

---

## 1. Compilation & Warnings

```
cargo check -p neotrix-sim 2>&1 | grep -E "error|warning" → (no output)
```

**Result:** CLEAN — zero errors, zero warnings.

---

## 2. Tests

```
cargo test -p neotrix-sim --lib 2>&1 | grep "test result"
```

**Result:** `test result: ok. 325 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

All 325 unit tests pass.

---

## 3. Dead Code / Unused Items

```
cargo check -p neotrix-sim 2>&1 | grep "dead_code|never used|never called" → (no output)
```

**Result:** CLEAN — no dead code, unused imports, or unused functions detected.

---

## 4. Agent Modules Audit

### Files on disk (`neotrix-sim/src/agents/`)

| # | File | Lines (est.) |
|---|------|-------------|
| 1 | `action_awareness.rs` | — |
| 2 | `action_costs.rs` | — |
| 3 | `event_reactive.rs` | — |
| 4 | `goal_outcome_feedback.rs` | — |
| 5 | `graph_memory.rs` | — |
| 6 | `intention_commitment.rs` | — |
| 7 | `memory_stream.rs` | — |
| 8 | `mod.rs` | 25 |
| 9 | `personality_drift.rs` | — |
| 10 | `pheromone.rs` | — |
| 11 | `planning.rs` | — |
| 12 | `reflection.rs` | — |
| 13 | `sim_agent.rs` | — |
| 14 | `social_learning.rs` | — |
| 15 | `spatial_memory.rs` | — |
| 16 | `thought_generation.rs` | — |

**Total:** 16 files (15 modules + mod.rs), 15 `.rs` module files.

---

### mod.rs Declarations vs Disk Files

| Module | mod.rs `pub mod` | mod.rs `pub use` | world_sim.rs import | Status |
|--------|:-:|:-:|:-:|--------|
| sim_agent | Yes | Yes (*) | Yes | Connected |
| action_awareness | Yes | Yes (*) | Yes | Connected |
| planning | Yes | Yes (*) | Yes | Connected |
| event_reactive | Yes | Yes (*) | Yes | Connected |
| personality_drift | Yes | Yes (*) | Yes | Connected |
| action_costs | Yes | Yes (*) | Yes | Connected |
| pheromone | Yes | Yes (*) | Yes | Connected |
| graph_memory | Yes | No | Yes (full path) | Connected |
| memory_stream | Yes | No | Yes (full path) | Connected |
| spatial_memory | Yes | No | Yes (full path) | Connected |
| reflection | Yes | No | Yes (full path) | Connected |
| goal_outcome_feedback | Yes | No | Yes (full path) | Connected |
| intention_commitment | Yes | No | Yes (full path) | Connected |
| thought_generation | Yes | No | Yes (full path) | Connected |
| social_learning | Yes | No | Yes (full path) | Connected |

(*) Re-exported via `pub use <module>::*`

**Result:** All 15 agent modules are declared in `mod.rs` and imported/used in `world_sim.rs`. No orphan modules.

---

## 5. world_sim.rs

- **Line count:** 1550
- **Structs defined:** `WorldSimConfig`, `EvolutionRecord`, `WorldSim`, `WorldSnapshot`
- **Key methods:** `new()`, `tick()`, `decide_action()` (5-layer pipeline), `execute_action()`, `evolution_cycle()`, `snapshot()`
- **Tick tiers used:** Reflex, Fast, Medium, Slow, Background (5-tier schedule)

---

## 6. Summary

| Metric | Value | Status |
|--------|-------|--------|
| Compile errors | 0 | PASS |
| Warnings | 0 | PASS |
| Dead code / unused | 0 | PASS |
| Tests | 325/325 pass | PASS |
| Agent modules (disk) | 15 + mod.rs | COMPLETE |
| Modules declared in mod.rs | 15 | 100% coverage |
| Modules imported in world_sim.rs | 15 | 100% coverage |
| Orphan modules (undeclared) | 0 | CLEAN |
| world_sim.rs size | 1550 lines | Moderate |

**Overall: neotrix-sim is clean — zero issues found.**
