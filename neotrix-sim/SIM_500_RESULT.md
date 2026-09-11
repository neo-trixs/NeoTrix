# SIM_500 Test Result

**Date**: 2026-09-11
**Test**: `sim_500_ticks` — 500-tick simulation stress test
**Status**: PASSED

## Configuration

- Default `WorldSimConfig`
- 10 initial agents
- 500 ticks at full system throughput

## Results

| Metric | Value |
|--------|-------|
| Final alive | 50 |
| Total deaths | 0 |
| Total ticks | 500 |
| Evolution events | 1 (Gen 1 at tick 500) |

## Tick-by-Tick Summary

| Tick | Alive | Deaths | Notes |
|------|-------|--------|-------|
| 0 | 10 | 0 | Initial population |
| 100 | 10 | 0 | Stable, no losses |
| 200 | 10 | 0 | Stable, no losses |
| 300 | 10 | 0 | Stable, no losses |
| 400 | 10 | 0 | Stable, no losses |
| 500 | 50 | 0 | Evolution cycle spawned 40 new agents |

## Consciousness Metrics (at tick 500)

| Metric | Value |
|--------|-------|
| Mean Phi | 0.0400 |
| Best Phi | 0.1850 (agent #7) |
| Population | 10 → 50 (evolution) |
| Species | 0 |

## Evolution Cycle

At tick 500, the background evolution cycle triggered:
- Fitness: 0.3797 (mean) / 0.3860 (max)
- Species count: 4
- Pressure: 0.42
- Spawned 40 new agents via self-play

## Observations

1. **Zero mortality** — All 10 initial agents survived the full 500 ticks. The simulation is stable under default conditions.
2. **Evolution triggered at tick 500** — The `EvolutionGen` background tier fires at the end, producing 40 offspring from 10 parents (5x multiplier).
3. **Phi decay** — Mean phi decreased from ~0.08 (tick 120) to 0.04 (tick 500), suggesting coherence stabilizes at a lower equilibrium over time.
4. **Best phi shift** — The best-performing agent shifted from agent #9 to agent #7 around tick 480, indicating selection pressure is active.
5. **Population explosion** — Post-evolution population jumped to 50, demonstrating the evolution system can grow the population without crashes.

## Files

- Test: `neotrix-sim/tests/sim_500_test.rs`
- Result: `neotrix-sim/SIM_500_RESULT.md`
