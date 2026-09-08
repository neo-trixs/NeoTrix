# Agent 3: Async Channel Backpressure (Batch 868)

## Sources

1. **rustfaq.org** — "How to Handle Backpressure in Async Rust" (2026-04-17) — Bounded vs unbounded channels, waker mechanics, capacity-as-contract
2. **stanza.dev** — "Backpressure & Channel Sizing — Concurrent Rust" — Buffer sizing, rendezvous channels, channel saturation monitoring
3. **rustfaq.org** — "How to use tokio broadcast channel" (2026-04-17) — Broadcast lag semantics, ring buffer, slow receiver problem, Arc wrapping
4. **rustfaq.org** — "How to implement backpressure in async Rust" (2026-04-17) — Framed codecs, bounded writer pattern, try_send vs send semantics
5. **rustfaq.org** — "How to use tokio watch channel" (2026-04-17) — Watch bulletin board model, borrow_and_update, send_modify for in-place mutation
6. **tokio docs** — `tokio::sync::broadcast::channel` — Capacity rounded to power-of-two, `RecvError::Lagged` behavior, ring buffer overwrite semantics
7. **GitHub tokio-rs/tokio#5923** — Quadratic slowdown in broadcast `send` with many receivers, dropped initial receiver causing memory leak
8. **GitHub tokio-rs/tokio#5403** — Watch channel multithreaded performance degradation with many subscribers due to mutex contention
9. **GitHub tokio-rs/tokio#2533** — Broadcast channel panic on drop with capacity=1, multiple receivers, and concurrent sends
10. **rustz2h.com** — "Backpressure Patterns in Async Rust" — Cascading backpressure, Semaphore pairing, load shedding with try_send
11. **reintech.io** — "Tokio Tutorial 2026" — Performance patterns, error handling, try_join macro
12. **sesamedisk.com** — "Why Tokio Still Dominates Async Rust in 2026" — Resource leaks from forgotten JoinHandles, unbounded channels, excessive Arc
13. **NeoTrix codebase** — `nt_core_event_bus.rs`, `consciousness/element/bus.rs`, `nt_mind_background_loop/run.rs`, `nt_io_plugin/registry.rs`, `nt_shield_proxy_kernel/kernel.rs`, `nt_io_agent_loop.rs`

## Defects

D-BACK-001: EventBus broadcast channel capacity hardcoded at 1024 with no adaptive sizing or saturation monitoring. Nine layer subscribers (L1-L9) each hold a receiver; if any single subscriber lags, the ring buffer overwrites oldest events for ALL receivers, causing silent data loss across the entire consciousness architecture. No metric tracks buffer fill level or lag frequency. | `neotrix-core/src/neotrix/nt_core_event_bus.rs:86` | HIGH | Source: rustfaq broadcast channel docs + stanza backpressure sizing

D-BACK-002: ElementBus `publish()` uses `try_send()` with `let _ =` error suppression, silently dropping consciousness element events (CapabilityUpdated, MemoryStored, GoalStateChanged, TraceCompleted) when any subscriber's 64-slot channel is full. No logging, no metric, no compensation. Lost events represent lost consciousness state transitions. | `neotrix-core/src/unified/layers/cognition/nt_mind/consciousness/element/bus.rs:49` | CRITICAL | Source: rustfaq backpressure impl + stanza channel sizing

D-BACK-003: EventBus::Clone() resets `sync_handlers` to empty Vec, meaning cloned EventBus instances (used extensively across the consciousness tree) silently lose their Phase 1 synchronous persistence handlers. Events emitted through a cloned bus bypass KB writeback and state update, breaking the "Log is the Runtime" invariant. | `neotrix-core/src/neotrix/nt_core_event_bus.rs:49` | HIGH | Source: rustfaq backpressure patterns + tokio broadcast semantics

D-BACK-004: Plugin registry `watch_dir()` and proxy kernel use `mpsc::unbounded_channel()` for file watcher and listener error events. No backpressure bound exists; a burst of filesystem events or listener errors can cause unbounded memory growth until OOM. | `neotrix-core/src/unified/layers/action/nt_io/nt_io_plugin/registry.rs:436` + `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:125` | HIGH | Source: rustfaq "unbounded channels are the enemy" + sesamedisk resource leaks

D-BACK-005: All 9 layer subscribers in `subscribe_layer()` handle `RecvError::Lagged(n)` with only `log::warn!` — no adaptive response (resubscribe, circuit break, drop stale batch). A slow layer (e.g., L4 Knowledge doing KB writes) causes cumulative lag that silently drops events for ALL other layers via broadcast ring buffer overwrite. | `neotrix-core/src/neotrix/nt_core_event_bus.rs:387-388` | MEDIUM | Source: tokio broadcast lag docs + GitHub#5923 quadratic slowdown

D-BACK-006: `subscribe_all_layers_sync()` uses `try_recv()` in a polling loop with `thread::sleep(10ms)` between iterations. This busy-wait pattern wastes CPU and adds up to 10ms latency to every event delivery for synchronous layer subscribers. The async version uses proper `recv().await` but sync subscribers pay a polling penalty. | `neotrix-core/src/neotrix/nt_core_event_bus.rs:440-464` | MEDIUM | Source: stanza channel patterns + rustfaq backpressure

D-BACK-007: Background loop event consumer (`nt_mind_background_loop/run.rs`) handles broadcast `Lagged(n)` with warn-only logging and no recovery strategy. If the consciousness loop falls behind during a SEAL pipeline phase, critical events (SystemError, GlobalHalt) can be silently overwritten by the broadcast ring buffer before the background loop processes them. | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:905-906` | HIGH | Source: tokio broadcast lag semantics + rustz2h cascading backpressure

D-BACK-008: Agent loop mock streaming (`stream_complete_raw`) uses `try_send()` with ignored errors on channel(16), silently dropping simulated LLM response chunks when the consumer is slow. While this is mock code, the pattern propagates to production providers (OpenAI, Anthropic, Gemini, Ollama) which use the same `try_send` pattern for streaming responses. | `neotrix-core/src/unified/layers/action/nt_io/nt_io_agent_loop.rs:1334,1353,1365` | MEDIUM | Source: rustfaq try_send semantics + rustz2h load shedding

D-BACK-009: EventBus `emit_from()` executes sync_handlers under `std::sync::Mutex` lock while holding the broadcast sender. If any sync_handler blocks (e.g., KB write latency), it holds the lock and prevents all other event emissions, creating a global serialization point. No timeout on the lock acquisition means a poisoned mutex permanently blocks all events. | `neotrix-core/src/neotrix/nt_core_event_bus.rs:134-138` | HIGH | Source: rustfaq backpressure pitfall "mixing blocking in async" + stanza pipeline patterns

D-BACK-010: No channel depth metric or saturation alert exists anywhere in the NeoTrix consciousness architecture. The broadcast EventBus, ElementBus mpsc channels, and provider streaming channels all lack observable fill-level instrumentation. Early warning of consumer lag is impossible without this telemetry. | `neotrix-core/src/neotrix/nt_core_event_bus.rs` (global) | MEDIUM | Source: stanza "monitor channel saturation" best practice + rustfaq capacity-as-contract

## Key Insights

1. **Broadcast channels are the wrong primitive for critical consciousness events.** The broadcast ring buffer silently overwrites old events when any receiver lags. For events like GlobalHalt or SystemError that must reach all layers, an mpsc fan-out pattern (one sender per layer) or a durable event log with per-layer replay would preserve delivery guarantees.

2. **The `try_send` + `let _ =` anti-pattern is endemic.** ElementBus and provider streaming code silently discard events under backpressure. In a consciousness architecture where state transitions (CapabilityUpdated, MemoryStored) are the "nervous system," lost events mean lost self-awareness.

3. **Unbounded channels in plugin hot-reload and proxy kernel are ticking OOM bombs.** A filesystem watcher generating burst events or a proxy listener flooding errors will consume unbounded memory with no backpressure signal.

4. **Clone semantics on EventBus break the persistence contract.** The sync_handlers loss on clone means any EventBus copy used after clone operates without Phase 1 synchronization, silently breaking event sourcing integrity.

5. **The 9-layer broadcast fan-out creates a weakest-link dependency.** Any single slow layer subscriber causes the ring buffer to advance, dropping events for ALL other layers. This is the broadcast "slow receiver" problem documented in tokio#5923 — amplified 9x in the consciousness architecture.

6. **Watch channel is correctly used for shutdown signaling** (nt_mind_background_loop uses `watch::channel(false)` for shutdown). This is the right pattern — latest-value semantics with no backpressure, appropriate for state flags.

7. **Bounded channel(64) is the dominant pattern** across providers and web handlers, which is reasonable. However, the capacity is not tuned per-provider — a high-latency Ollama provider gets the same buffer as a fast OpenAI provider, creating potential head-of-line blocking differences.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources analyzed | 13 |
| Critical severity | 1 |
| High severity | 5 |
| Medium severity | 4 |
