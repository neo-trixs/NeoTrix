# Legacy Archive

This directory contains archived code from the old architecture that has been superseded by the new 6-layer architecture.

## Archived Directories

| Directory | Original Location | Reason | Date |
|-----------|-------------------|--------|------|
| l7_capability_impl | neotrix/l7_capability_impl | Capability system - superseded by l5_cognition | 2026-09-01 |
| l9_transcendent_impl | neotrix/l9_transcendent_impl | Transcendent layer - merged into l6_meta | 2026-09-01 |
| l10_transcendent_impl | neotrix/l10_transcendent_impl | Transcendent layer - merged into l6_meta | 2026-09-01 |

## New Architecture

The new 6-layer architecture is located in `neotrix-core/src/`:

```
l1_action/          # L1 Action Layer
l2_perception/      # L2 Perception Layer
l3_embodiment/      # L3 Embodiment Layer
l4_emotion/         # L4 Emotion Layer
l5_cognition/       # L5 Cognition Layer
l6_meta/            # L6 Meta-Cognition Layer
```

## Migration Status

- [x] Phase 1: Archive unused directories (l7, l9, l10)
- [ ] Phase 2: Migrate core modules (nt_feel, nt_core, nt_mind)
- [ ] Phase 3: Migrate implementation layers (nt_act, nt_io, nt_memory, nt_world, nt_shield)
- [ ] Phase 4: Migrate meta-cognition layer (nt_meta, nt_repair, nt_nexus)
- [ ] Phase 5: Clean up and remove old directories

## Notes

- These files are kept for reference only
- Do not modify archived code
- All new development should use the new architecture
