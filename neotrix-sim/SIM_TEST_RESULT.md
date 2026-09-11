# NT-WORLD-SIM Simulation Test Result

**Date**: 2026-09-11
**Status**: PASSED

## Configuration
- **Initial agents**: 10
- **World size**: 1000x1000
- **Ticks**: 100

## Results

| Tick | Agents Alive | Global Phi |
|------|-------------|-----------|
| 0    | 10          | -         |
| 10   | 10          | 0.2762    |
| 20   | 10          | 0.2771    |
| 30   | 10          | 0.2777    |
| 40   | 10          | 0.2781    |
| 50   | 10          | 0.2784    |
| 100  | 10          | 0.2784    |

## Observations

- **100% survival rate**: All 10 agents survived 100 ticks (metabolism calibrated correctly)
- **Phi convergence**: Global coherence (phi) converged from 0.2762 to 0.2784, showing steady consciousness metric stability
- **No agent deaths**: Default config is balanced — agents can find food and rest without dying in a 100-tick run
- **Build time**: 2.84s (clean), test runtime: 0.10s

## Verdict

The simulation runs correctly with the default `WorldSimConfig`. The agent survival loop, consciousness metrics (phi/coherence), and multi-timescale tick scheduling all work as expected.
