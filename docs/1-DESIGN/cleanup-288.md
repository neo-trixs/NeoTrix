# Cleanup-288: Cross-Layer References

## Cross-Layer Dependencies

```
l5_cognition→l1_action:   4
l5_cognition→l2_perception: 1
l5_cognition→l6_meta:   2
```

## Analysis

Only `l5_cognition` (Cognition Layer) has cross-layer imports:

| Target Layer | Count | Direction |
|---|---|---|
| l1_action | 4 | Top-down (L5→L1) — cognition layer depends on action layer |
| l2_perception | 1 | Top-down (L5→L2) — cognition layer depends on perception layer |
| l6_meta | 2 | Bottom-up (L5→L6) — cognition layer depends on meta-cognition layer |

All other layers (`l1_action`, `l2_perception`, `l3_embodiment`, `l4_emotion`, `l6_meta`) have **zero** cross-layer imports to peers or upward layers. Clean layered architecture maintained.

## Notes

- `l5_cognition→l1_action` (4 refs): Cognitive reasoning may invoke action execution (e.g., tool calls, orchestration)
- `l5_cognition→l2_perception` (1 ref): Cognitive layer reads perception state (e.g., sensory integration hub)
- `l5_cognition→l6_meta` (2 refs): Cognitive layer accesses meta-cognition for self-awareness/coordination

Test and facade modules excluded from count.
