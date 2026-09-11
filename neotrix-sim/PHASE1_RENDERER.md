# Phase 1: World Renderer — Terminal Rendering

## Summary

Implemented a terminal-based world renderer for `neotrix-sim` that visualizes the simulation state using ANSI escape codes and ASCII characters.

## Files

| File | Action |
|------|--------|
| `neotrix-sim/src/world_sim/renderer.rs` | Created |
| `neotrix-sim/src/world_sim/mod.rs` | Updated (`pub mod renderer;` added) |

## Implementation Details

### Renderer Struct

- `width` / `height` — terminal viewport dimensions
- `viewport_x` / `viewport_y` — world-space camera offset
- `zoom` — scale factor (0.2x – 5.0x)

### Rendering Pipeline

1. Clear screen (`\x1b[2J\x1b[H`)
2. Print header: tick number, alive agent count, average energy
3. For each viewport cell, map screen coords → world coords
4. Query agents at world position; render character based on agent personality:
   - `P` — high aggression (>0.7)
   - `p` — moderate aggression (>0.3)
   - `N` — low aggression (<=0.3)
   - `.` — empty terrain
5. Print controls footer

### Controls

| Method | Action |
|--------|--------|
| `pan(dx, dy)` | Move viewport by delta (zoom-adjusted) |
| `zoom_in()` | Scale ×1.2, capped at 5.0 |
| `zoom_out()` | Scale ÷1.2, floored at 0.2 |
| `center_on_agent(agent)` | Snap viewport center to agent position |

## Tests

```
test world_sim::renderer::tests::test_renderer_creation ... ok
test world_sim::renderer::tests::test_renderer_pan ... ok
test world_sim::renderer::tests::test_renderer_zoom ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

## Notes

- Adapted from original spec: `AgentCore` has no `species` field, so agent rendering uses `personality.aggression` thresholds instead.
- Zoom test fixed: double `zoom_out()` needed because `1.2 / 1.2 = 1.0` (not `< 1.0`).
