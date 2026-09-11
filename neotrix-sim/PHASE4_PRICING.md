# Phase 4: Supply/Demand Pricing

## Overview

Supply/demand pricing system for neotrix-sim economic simulation.

## Changes

| File | Change |
|------|--------|
| `src/economy/mod.rs` | New module declaration |
| `src/economy/pricing.rs` | `ResourceMarket` + `PricingEngine` |
| `src/lib.rs` | Added `pub mod economy` |

## Key Types

| Type | Purpose |
|------|---------|
| `ResourceMarket` | Per-market supply/demand tracking with elasticity-based pricing |
| `PricingEngine` | Multi-market orchestrator with trade recording and arbitrage detection |

## Pricing Model

Price = base_price * (1 + (demand/supply - 1) * elasticity)

Default resources: food(10), wood(5), stone(8), metal(20), gold(50)

Decay per tick: supply *= 0.95, demand *= 0.90

## Arbitrage

`get_arbitrage_opportunities()` returns price differences > 1.0 across markets, sorted by spread descending.

## Verification

- `cargo check -p neotrix-sim`: 0 errors, 4 warnings (pre-existing dead_code)
- `cargo test -p neotrix-sim --lib -- pricing`: 4/4 passed
