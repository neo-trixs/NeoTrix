# Phase 4: Self-Play & Novelty Search

## Summary

Added self-play system and novelty-based behavior exploration to neotrix-sim evolution pipeline.

## New Modules

### `self_play.rs`
- **HistoricalAgent**: Wraps `SimAgent` with tick and win/loss stats
- **SelfPlaySystem**: Maintains agent history, selects opponents by fitness proximity, tracks matchup outcomes, identifies elite performers
- Key methods:
  - `record_agent()` - snapshot agent at current tick
  - `select_opponent()` - find closest-fitness opponent for balanced competition
  - `record_matchup()` - track head-to-head results
  - `get_elite()` - highest win-rate agent with at least one match

### `novelty.rs`
- **BehaviorDescriptor**: Agent trait vector with novelty score
- **NoveltySearch**: Archive-based novelty detection using k-nearest neighbor distance
- Key methods:
  - `novelty_score()` - average distance to k nearest archive entries
  - `add_to_archive()` - conditionally add if above novelty threshold
  - `compute_novelty()` - batch score all descriptors
  - `select_novel()` - pick top-n most novel candidates

## Files Changed

| File | Change |
|------|--------|
| `src/evolution/mod.rs` | Added `pub mod self_play; pub mod novelty;` |
| `src/evolution/self_play.rs` | New file |
| `src/evolution/novelty.rs` | New file |

## Test Results

```
self_play: 5 passed, 0 failed
novelty:   5 passed, 0 failed (includes world::tests novelty tests)
```

## Usage Pattern

```rust
use neotrix_sim::evolution::{SelfPlaySystem, NoveltySearch, BehaviorDescriptor};

// Self-play: record agents, pit them against each other
let mut sp = SelfPlaySystem::new();
sp.record_agent(&agent, tick);
if let Some(opponent) = sp.select_opponent(&current_agent) {
    sp.record_matchup(current_idx, opponent_idx, current_won);
}

// Novelty: discover diverse behaviors
let mut ns = NoveltySearch::new();
ns.add_to_archive(descriptor);
let score = ns.novelty_score(&candidate);
```
