# SIM_200_RESULT

## Test: 200-Tick Simulation

**Date:** 2026-09-11
**Status:** PASS

### Configuration

| Parameter | Value |
|-----------|-------|
| World Size | 1000×1000 |
| Initial Agents | 10 |
| Seed | 42 |
| Ticks per Hour | 100 |
| Evolution Interval | 500 |

### Results

| Metric | Initial | After 200 Ticks |
|--------|---------|-----------------|
| Population | 10 | 10 (100% survival) |
| Mean Phi | 0.1000 | 0.0501 |
| Best Phi | 0.2762 | 0.2804 |
| Species Count | 0 | 0 |

### Coherence Timeline

| Tick | Mean Phi | Best Phi | Pop |
|------|----------|----------|-----|
| 20 | 0.1000 | 0.2762 | 10 |
| 40 | 0.1000 | 0.2771 | 10 |
| 60 | 0.1000 | 0.2777 | 10 |
| 80 | 0.1000 | 0.2781 | 10 |
| 100 | 0.1000 | 0.2784 | 10 |
| 120 | 0.0820 | 0.2786 | 10 |
| 140 | 0.0694 | 0.2788 | 10 |
| 160 | 0.0606 | 0.2789 | 10 |
| 180 | 0.0544 | 0.2764 | 10 |
| 200 | 0.0501 | 0.2804 | 10 |

### Observations

- **Full survival**: All 10 agents survived 200 ticks — no deaths
- **Phi convergence**: Mean phi decreased from 0.10 → 0.05 over 200 ticks (coherence stabilized)
- **Best phi stable**: Best individual phi hovered ~0.28, slight uptick to 0.2804 at tick 200
- **No speciation**: Species count remained at 0 — no evolutionary divergence in 200 ticks (expected, evolution interval is 500)
- **No safety alerts**: No violations or audit entries during simulation
