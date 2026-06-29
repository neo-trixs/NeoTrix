# neotrix-core Binaries

All functionality consolidated into the single `neotrix` binary.

| Legacy Binary | Replaced By | Source |
|---------------|-------------|--------|
| `neotrix` | `neotrix` | `src/main.rs` |
| `ne-dialog` | `neotrix --tui` | `src/tui.rs` |
| `neotrix-web` | `neotrix --web` | `src/web.rs` |
| `nt_design_token` | `neotrix token` | inline in `main.rs` |
| `neotrix-transit` | `neotrix transit` | `src/transit.rs` (requires `stealth-net`) |

## Legacy source files (kept for reference, not compiled as binaries)

| File | Notes |
|------|-------|
| `ne_dialog.rs` | Superseded by `src/tui.rs` |
| `neotrix_web.rs` | Superseded by `src/web.rs` |
| `nt_design_token.rs` | Inlined in `main.rs` `Token` handler |
| `neotrix_transit.rs` | Superseded by `src/transit.rs` |

## Other source files (still active via main binary)

| File | Description |
|------|-------------|
| `consciousness_seed.rs` | Consciousness seed — keep |
| `cycle.rs` | Cycle simulation — keep |
| `e2e_consciousness.rs` | End-to-end consciousness test — keep |

## Archived (one-off experiments, not actively maintained)

Files marked `// @archive` are one-off experiments from earlier sessions. They compile as part of the workspace but are not declared as named binaries in Cargo.toml.
