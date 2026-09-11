# Phase 3: Negotiation & Communication

## Files Created

| File | Status |
|------|--------|
| `src/society/negotiation.rs` | Created |
| `src/society/communication.rs` | Created |
| `src/society/mod.rs` | Updated (added `pub mod negotiation; pub mod communication;` + re-exports) |

## NegotiationEngine

- `NegotiationProposal` — offer/demand round tracking between two agents
- `start_negotiation()` — initiate a proposal
- `evaluate_proposal()` — check responder inventory + fairness threshold (0.3)
- `counter_proposal()` — swap offer/demand with concession decay (10% per round)
- `is_finished()` — accepted or max_rounds reached

## CommunicationChannel

- `Message` — id, sender, receiver (optional), channel, content, priority, timestamp
- `send()` — directed message with auto-ID and inbox registration
- `receive()` — agent inbox lookup
- `receive_channel()` — channel subscription lookup
- `broadcast()` — send to multiple receivers
- `tick()` — evict messages older than 100 ticks; cap at 5000 messages

## Verification

| Check | Result |
|-------|--------|
| `rustfmt` syntax check | Pass (files are valid Rust) |
| `cargo check -p neotrix-sim` | Blocked — pre-existing `ort = "^2.0"` dep error in `neotrix-core` (unrelated) |
| `cargo test -p neotrix-sim` | Blocked — same workspace-level dep resolution failure |

The `ort` crate has no stable `2.0.0` release on crates.io (only RC versions). This is a pre-existing issue in `neotrix-core/Cargo.toml` that blocks all workspace builds, not caused by this change. The new negotiation/communication modules depend only on `serde` (workspace dep) and `std::collections::HashMap`.
