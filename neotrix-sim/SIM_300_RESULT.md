# Sim 300-Tick Test Results

**Date**: 2026-09-11  
**Config**: `WorldSimConfig::default()` (10 agents, seed=42, 1000x1000 world)

## Summary

| Metric | Value |
|--------|-------|
| Initial agents | 10 |
| Final alive | 10 |
| Total deaths | 0 |
| Tick 0 alive | 10 |
| Tick 50 alive | 10 |
| Tick 100 alive | 10 |
| Tick 150 alive | 10 |
| Tick 200 alive | 10 |
| Tick 250 alive | 10 |
| Tick 300 alive | 10 |

## Phi / Coherence Trend

| Tick | mean_phi | best_phi | pop |
|------|----------|----------|-----|
| 20 | 0.1000 | 0.2762 | 10 |
| 100 | 0.1000 | 0.2784 | 10 |
| 200 | 0.0501 | 0.2804 | 10 |
| 300 | 0.0417 | 0.2450 | 10 |

## Observations

- **100% survival rate** — zero deaths across 300 ticks.
- `mean_phi` declined from 0.10 → 0.042 over 300 ticks (phi regularization settling).
- `best_phi` peaked at tick 200 (0.2804) then dipped to 0.2450 at tick 300.
- Species count stayed at 0 (no speciation within 300 ticks — evolution interval is 500).
- No safety alerts or audit entries triggered.
