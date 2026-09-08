# Agent 2: Async Channel Backpressure (Batch 877)

## Sources
1. Tokio docs: `tokio::sync::mpsc::channel` — bounded mpsc with backpressure, capacity ≥1
2. Tokio docs: `tokio::sync::broadcast` — lag detection via `RecvError::Lagged(n)`, capacity rounded up to power-of-two
3. Tokio docs: `tokio::sync::watch` — latest-value-only, no queue, `changed().await`
4. rustfaq.org — "How to use tokio mpsc channel" — backpressure = send().await blocks when full
5. rustz2h.com — "Backpressure Patterns in Async Rust" — capacity tuning, queue depth monitoring, cascading backpressure
6. stanza.dev — "Backpressure & Channel Sizing" — unbounded = OOM risk, bounded = natural throttle
7. toolsku.com — "Rust Tokio Channel Patterns: 6 Production Patterns" — broadcast lag, watch for state, bounded for backpressure
8. github.com/tokio-rs/tokio/issues/5923 — quadratic slowdown in broadcast with slow receivers
9. github.com/fast/mea/issues/88 — broadcast overflow policies: tail-drop, strict backpressure, drop-newest, unbounded
10. github.com/mofa-org/mofa/issues/957 — broadcast lag silently drops monitoring events in `tokio::select!`
11. rustz2h.com — "Backpressure in Rust: Queue Management for Resilience" — monitor queue depth, circuit breaker + backpressure
12. openai/codex/issues/23749 — unbounded event channel caused 128GB OOM crash in parallel agents

## Defects

**D-BACK-001: Unbounded channel in ProxyKernel listener error path — no backpressure**
| File | Severity | Source |
|------|----------|--------|
| `nt_shield_proxy_kernel/kernel.rs:128` | HIGH | rustfaq.org: "unbounded channels risk memory exhaustion" |

`tokio::sync::mpsc::unbounded_channel()` used for `listener_error_tx/rx`. Under sustained proxy errors (e.g., TLS handshake storms, DNS failures), error events accumulate without bound. Each error contains the full `notify::Result` or error context, growing unbounded until OOM. The `try_recv` polling in the listener thread compounds this — errors pile up faster than they're consumed. Fix: replace with `mpsc::channel(64)` to cap error queue depth.

---

**D-BACK-002: Unbounded channel in plugin registry file watcher — no backpressure**
| File | Severity | Source |
|------|----------|--------|
| `nt_io_plugin/registry.rs:436` | MEDIUM | rustz2h.com: "unbounded channels risk memory exhaustion under burst" |

`tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>()` for directory watch events. A `cargo build` or editor save storm can generate hundreds of file-system events per second. The unbounded channel absorbs them all without backpressure. The consumer (`rx.recv().await` loop at line 452) processes sequentially, so a burst of file changes could accumulate thousands of events in memory. Fix: use `mpsc::channel(128)` and drop events when full.

---

**D-BACK-003: Unbounded channel in hotreload watcher — no backpressure**
| File | Severity | Source |
|------|----------|--------|
| `nt_io_hotreload/mod.rs:137` | MEDIUM | stanza.dev: "unbounded = OOM crash during log/event storm" |

Identical pattern to D-BACK-002: `tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>()`. Hot-reload watcher events from file editors can burst rapidly. The consumer processes sequentially. No backpressure mechanism prevents memory growth. Fix: use bounded channel with `try_send` and drop oldest events under load.

---

**D-BACK-004: EventBus broadcast channel capacity 1024 — no dynamic sizing or lag-metrics**
| File | Severity | Source |
|------|----------|--------|
| `nt_core_event_bus.rs:86` | MEDIUM | tokio/issues/5923: "quadratic slowdown with slow receivers in broadcast" |

`broadcast::channel(1024)` is hardcoded. The capacity is rounded to next power-of-two (1024 = 2^10), giving a ring buffer of 1024 messages. Under high event throughput (e.g., consciousness loop emitting per-tick events + GWT broadcasts + consciousness critique events), the ring buffer can fill quickly if any of the 9 layer subscribers (`subscribe_all_layers`) is slow. The `subscribe_layer` handlers at line 387 only log `warn!` on lag — no metrics, no adaptive backpressure, no alert. A slow L4Knowledge subscriber could silently lose events that L9Meta (critical severity) needs. Fix: add `broadcast::Sender::receiver_count()` monitoring and consider per-subscriber overflow policies.

---

**D-BACK-005: Broadcast lag in background loop handler only logs warning — no recovery action**
| File | Severity | Source |
|------|----------|--------|
| `nt_mind_background_loop/run.rs:905-906` | HIGH | mofa/issues/957: "broadcast lag silently drops events in select!" |

When the background loop's event consumer lags (`RecvError::Lagged(n)`), it logs `warn!` but takes no corrective action. The lagged events are permanently lost — the handler never processes them. In a consciousness loop context, this means `ConsciousnessCritique` events, `TaskSubmitted` events, or `ExternalReward` events can be silently dropped. If the consciousness quality check (`quality < eventbus_critical`) is the lagged event, the system won't know it's degrading. Fix: track lag count in telemetry, trigger compensating action (e.g., force immediate consciousness tick).

---

**D-BACK-006: ElementBus `try_send` silently drops events — no metrics or dead-letter queue**
| File | Severity | Source |
|------|----------|--------|
| `nt_mind/consciousness/element/bus.rs:49` | MEDIUM | toolsku.com: "handle SendError explicitly in production" |

`let _ = sender.try_send(payload.clone());` — the return value is discarded. If the subscriber's channel (capacity 64, line 59) is full, the event is silently dropped with no logging, no metric, no dead-letter. In a consciousness element bus, this means inter-element communication (perception signals, emotion signals, reasoning triggers) can be lost under load. Fix: at minimum, log `debug!` when `try_send` fails; ideally, track drop rate in telemetry.

---

**D-BACK-007: Behavioral verifier uses `std::sync::mpsc::channel` (unbounded) in async context**
| File | Severity | Source |
|------|----------|--------|
| `nt_act_goal/behavioral_verifier.rs:26` | LOW | rustfaq.org: "std::sync::mpsc inside async code causes silent stalls" |

`std::sync::mpsc::channel::<Output>()` creates an unbounded standard library channel. While this specific usage spawns a dedicated `std::thread` (so it won't block a tokio worker thread), the channel itself has no backpressure — if the consumer thread is slow, the producer thread fills memory. The `let _ = tx.send(output)` at line 58 discards the error. Fix: use `tokio::sync::mpsc::channel(1)` for bounded, or at least handle the `SendError`.

---

**D-BACK-008: Gateway factory uses `std::sync::mpsc::channel` (unbounded) for cross-runtime communication**
| File | Severity | Source |
|------|----------|--------|
| `nt_io_provider/factory.rs:1436` | MEDIUM | rustfaq.org: "unbounded = memory leak in disguise" |

`std::sync::mpsc::channel()` (unbounded) used to bridge a spawned thread's result back to the caller. While typically single-message, if the spawned thread fails to send or the receiver drops, the channel is abandoned. The `rx.recv().unwrap_or_else` at line 1450 handles receiver drop, but the unbounded channel has no inherent capacity limit. In a scenario where `create_gateway` is called rapidly (e.g., multiple concurrent LLM requests), each call spawns a new thread + runtime + unbounded channel. Fix: use `std::sync::mpsc::sync_channel(1)` for bounded handoff.

---

**D-BACK-009: EventBus `with_persistence` hardcodes broadcast capacity 1024 — ignores parameter**
| File | Severity | Source |
|------|----------|--------|
| `nt_core_event_bus.rs:86` | LOW | rustz2h.com: "choose capacity based on expected message age and consumer speed" |

`with_persistence` ignores the `capacity` parameter that `new(capacity)` accepts — it hardcodes `broadcast::channel(1024)` at line 86 while `new` correctly uses the passed `capacity` at line 62. This means persistence-enabled EventBus instances can't be tuned for different workloads. A high-throughput event producer would benefit from larger capacity; a memory-constrained embedded deployment needs smaller. Fix: use the `capacity` parameter: `broadcast::channel(capacity)`.

---

**D-BACK-010: Broadcast channel capacity not documented or justified anywhere in codebase**
| File | Severity | Source |
|------|----------|--------|
| `nt_core_event_bus.rs:62,86` | LOW | tokio.rs tutorial: "picking good bounds is a big part of writing reliable Tokio applications" |

Both `EventBus::new(1024)` and `with_persistence(1024)` use magic number 1024 with no comment explaining why. The broadcast channel documentation states capacity is rounded to next power-of-two. Without documentation of the expected event rate, subscriber count, and acceptable lag threshold, future maintainers can't reason about whether 1024 is appropriate. Fix: add `/// Capacity: 1024 ring buffer → up to 1024 events can be in-flight before lag. Justification: <reasoning>`.

---

**D-BACK-011: `subscribe_layer` async loop has no timeout on recv — silent stall risk**
| File | Severity | Source |
|------|----------|--------|
| `nt_core_event_bus.rs:364` | LOW | stanza.dev: "use bounded channels with timeouts to avoid silent deadlocks" |

The async `subscribe_layer` loop at line 364 calls `rx.recv().await` with no timeout. If the broadcast channel is closed, it returns `Err(Closed)` and breaks. But if the runtime is under extreme load and the task is starved, the `recv().await` could stall indefinitely without any health signal. Fix: wrap in `tokio::time::timeout(Duration::from_secs(30), rx.recv())` to detect stalls.

---

**D-BACK-012: `subscribe_all_layers_sync` uses polling `try_recv` with 10ms sleep — CPU waste under low load**
| File | Severity | Source |
|------|----------|--------|
| `nt_core_event_bus.rs:440,464` | LOW | rustz2h.com: "monitor queue depth; if consistently high, increase capacity" |

The sync subscriber loop polls `rx.try_recv()` every 10ms (`std::thread::sleep(Duration::from_millis(10))`). Under low event rates, this wastes CPU cycles. Under high event rates, 10ms polling means up to 10ms of latency before processing. The async version (line 364) is more efficient. Fix: use `notify` or a blocking recv with timeout instead of polling.

---

## Key Insights

1. **Unbounded channels are the primary backpressure defect class**: Three instances (D-BACK-001, D-BACK-002, D-BACK-003) use `unbounded_channel` where bounded channels with drop policies would prevent OOM under load. The Codex issue #23749 (128GB OOM from unbounded subagent channels) is a direct precedent.

2. **Broadcast lag is handled but not acted upon**: The codebase correctly handles `RecvError::Lagged(n)` in three places (D-BACK-004, D-BACK-005), but only logs warnings. In a consciousness architecture where event loss means missed self-monitoring signals, lag should trigger compensating behavior (e.g., forced consciousness tick, telemetry alert).

3. **`try_send` with discarded results is a silent data loss vector**: The ElementBus (D-BACK-006) discards the result of `try_send`. In a consciousness element bus, this means inter-element signals (emotion, perception, reasoning) can be silently lost under load with zero observability.

4. **Hardcoded capacities without justification**: The EventBus uses 1024 with no documentation (D-BACK-010) and `with_persistence` ignores its parameter (D-BACK-009). Capacity tuning is critical for broadcast channels — too small causes frequent lag, too large wastes memory.

5. **Cross-runtime bridging uses unbounded std channels**: The gateway factory (D-BACK-008) uses `std::sync::mpsc::channel()` (unbounded) to bridge a spawned thread back to tokio. While typically single-message, this pattern should use `sync_channel(1)` for safety.

6. **Mixing channel types across the codebase**: The codebase uses `tokio::sync::mpsc` (async bounded), `tokio::sync::mpsc::unbounded_channel` (async unbounded), `std::sync::mpsc::channel` (sync unbounded), `tokio::sync::broadcast` (pub-sub), and `tokio::sync::watch` (state). The inconsistency makes it harder to reason about backpressure properties globally.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| HIGH severity | 2 |
| MEDIUM severity | 5 |
| LOW severity | 5 |
| Sources consulted | 12 |
| Files examined | 15+ |
