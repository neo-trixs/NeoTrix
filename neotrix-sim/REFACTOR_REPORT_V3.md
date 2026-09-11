# REFACTOR_REPORT_V3 — neotrix-sim Deep Refactoring

**Date**: 2026-09-11  
**Baseline**: RESEARCH_V3.md cross-cutting themes  
**Files Changed**: 2 (`world_sim.rs`, `social_learning.rs`)  
**Net Change**: -31 lines (67 removed, 36 added)  
**Verification**: 325/325 tests pass, 0 warnings

---

## Issues Found & Fixed

### 1. Unused Import (HIGH)

**File**: `world_sim.rs:39`  
**Issue**: `PheromoneSignal` imported but never used in scope  
**Fix**: Removed from import list  
**Impact**: Eliminates compiler warning, cleaner dependency graph

### 2. Misleading Unused Variable Prefix (HIGH)

**File**: `world_sim.rs:1012`  
**Issue**: `message: _message` — underscore prefix implies unused, but `_message` was dereferenced later on line 1024 as `&_message`  
**Fix**: Renamed to `message` (no underscore prefix)  
**Impact**: Eliminates confusion; variable is genuinely used in culture meme creation

### 3. Duplicate Inventory Building Code (HIGH)

**File**: `world_sim.rs:1039-1107`  
**Issue**: The `Trade` action handler contained two near-identical 30-line blocks building inventories for the trading agent and target agent. The only difference was the agent position used for radius filtering.  
**Fix**: Extracted `build_inventory_near(x, y, radius)` helper method that encapsulates the resource→inventory mapping logic. Both blocks now call this helper with their respective positions.  
**Impact**: 
- 67 → 36 lines (31 lines removed)
- Single source of truth for resource→inventory mapping
- Future changes to inventory logic only need one edit
- Aligned with research finding P5 (Skill as Reusable Template)

### 4. Redundant Clone in Evolution Cycle (MEDIUM)

**File**: `world_sim.rs:1355`  
**Issue**: `valid_offspring.clone()` was passed to `extend()` while `valid_offspring` was later consumed by a `for` loop. The clone was necessary for the `extend` call but the `.clone()` on the entire Vec followed by `extend` is semantically equivalent to `extend_from_slice` or `iter().cloned()`.  
**Fix**: Changed to `extend(valid_offspring.iter().cloned())` — marginally clearer intent.  
**Note**: Full elimination of the clone would require restructuring the speciation/spawn order, which is a larger refactor deferred to a future pass.

### 5. Duplicate Step Numbering (LOW)

**File**: `world_sim.rs:529,600`  
**Issue**: Two comments both labeled `// 9.` — one for planning, one for evolution cycle  
**Fix**: Renumbered evolution cycle to `// 15.` to maintain sequential order in the tick() function  
**Impact**: Correct documentation; no runtime effect

### 6. Unused Struct Field (MEDIUM)

**File**: `agents/social_learning.rs:32`  
**Issue**: `_max_pattern_history: usize` field was never read or written outside the struct  
**Fix**: Removed the field and its initialization  
**Impact**: Cleaner struct definition, no wasted memory per SocialLearning instance

---

## Cross-Domain Alignment Analysis

### EventBus Connections
All modules properly connected via `SimulationBus`. Events flow:
- Clock → Bus (time events)
- PhiBridge → Bus (consciousness metrics)
- PheromoneField → Bus (pheromone deposits)
- WorldSim → Bus (agent actions, economy transactions, resource depletion)

No missing connections found.

### Data Flow Directionality
One-way flows identified as intentional:
- `PhiBridge` → `CoherenceTracker` (phi feeds coherence, not reverse)
- `SafetyMonitor` → `AuditTrail` (alerts flow to audit, not reverse)

These are correct by design (monitoring/auditing are sink nodes).

### Naming Conventions
All public APIs use `snake_case` consistently. No violations found.

---

## Pipeline Unification

### Input → Transform → Output Pattern
Each subsystem follows the pattern:
| Module | Input | Transform | Output |
|--------|-------|-----------|--------|
| ActionAwareness | AgentAction + SimAgent | Rule-based prediction | ExpectedOutcome |
| ThoughtGeneration | Memory + Planning + Awareness | Summary composition | MemoryNode |
| IntentionCommitment | PlanningStack + tick | BDI commitment logic | CommittedIntention |
| SocialLearning | ObservedAction | Pattern tracking | mimicry_bias |
| GoalOutcomeFeedback | Goal + Action + Awareness | Success evaluation | GoalOutcomeRecord |

All pipelines are clean with no missing transforms.

---

## Performance Notes

### No Blocking in Async Context
The `tick()` method is `async` and all `.await` points are on `SimulationBus::emit()` which is non-blocking.

### Allocation Hotspots
- `build_observation()` creates `Vec<NearbyAgent>` and `Vec<NearbyResource>` per agent per tick — acceptable for <50 agents
- `agent_ids: Vec<String>` collected per tick for iteration — necessary due to borrow checker constraints

---

## Remaining Opportunities (Deferred)

| Item | Priority | Rationale |
|------|----------|-----------|
| Agent lookup pattern (`agents.iter().find(\|a\| a.core.id == id)`) repeated 15+ times | P1 | Could use index map, but requires careful borrow management |
| `println!` in `compute_consciousness_metrics` and `evolution_cycle` | P2 | Should use structured logging/tracing |
| `EvolutionRecord` cloned into `evolution_history` Vec | P2 | Could use `Rc` or index-based storage |
| `action_awareness` HashMap lookup on every tick | P2 | Hot path; consider direct field on SimAgent |

---

## Verification

```
cargo check -p neotrix-sim     → 0 warnings, 0 errors
cargo test -p neotrix-sim --lib → 325/325 passed
```
