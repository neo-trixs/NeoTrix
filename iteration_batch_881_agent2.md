# Iteration Batch 881 — Agent 2: Rust Async Channel Backpressure Research

## Research Scope

Deep research on Tokio `mpsc`, `broadcast`, `watch` channel backpressure semantics. 28 `mpsc::channel` sites, 2 `broadcast::channel` sites, 2 `watch::channel` sites found across NeoTrix. 5+ architectural defects identified.

---

## 1. Tokio Channel Backpressure Semantics (Reference)

| Channel | Backpressure Model | Failure Mode |
|---------|-------------------|--------------|
| `mpsc::channel(N)` | **Cooperative blocking**: `send().await` parks producer when buffer full. No messages lost. | Producer task suspended indefinitely if consumer is slow; potential deadlock if producer holds a lock. |
| `mpsc::unbounded_channel()` | **None**. `send()` always completes immediately (sync). | Unbounded memory growth if consumer falls behind. OOM-kill. |
| `broadcast::channel(N)` | **Drop-oldest**: ring buffer overwrites oldest message when full. Slow receivers get `Lagged(n)`. Capacity rounded up to next power-of-two. | Silent data loss for slow subscribers. No backpressure to sender — sender never blocks. |
| `watch::channel(init)` | **Overwrite-latest**: only last value retained. `changed()` awaits new unseen value. No queue depth concept. | No history. Rapid `send()` may cause receivers to miss intermediate values. |

### Key Gotcha: Capacity Rounding

`broadcast::channel(3)` → actual buffer = 4 (next power-of-two). The `capacity` argument is the *minimum*, not exact. Lag detection is based on the rounded size.

### Key Gotcha: `try_send` Silently Drops

`mpsc::Sender::try_send()` returns `Err(TrySendError::Full)` when buffer is full. If the caller does `let _ = tx.try_send(...)`, the message is silently dropped — no backpressure, no error visible.

---

## 2. NeoTrix Channel Inventory

### 2a. MPSC Bounded (capacity values)

| File | Line | Capacity | Context |
|------|------|----------|---------|
| `nt_io_provider/openai.rs` | 232 | 64 | LLM streaming response |
| `nt_io_provider/anthropic.rs` | 255 | 64 | LLM streaming response |
| `nt_io_provider/ollama.rs` | 133 | 64 | LLM streaming response |
| `nt_io_provider/gemini.rs` | 153 | 64 | LLM streaming response |
| `nt_io_provider/free_providers.rs` | 151,340,509,676 | 64 | LLM streaming (4 providers) |
| `nt_io_web/api.rs` | 1096 | 64 | Web API response |
| `nt_io_agent_loop.rs` | 1332 | 16 | Mock/test stream |
| `nt_core_llm.rs` | 59 | 64 | LLM stream_raw output |
| `nt_core_llm.rs` | 971,988,1009,1082 | 1 | Test assertions |
| `nt_mind/consciousness/element/bus.rs` | 59 | 64 | ElementBus per-subscriber |
| `nt_mind/reason/reasoning_engine/engine_core.rs` | 1907 | 64 | Test |
| `nt_core_gate/tests.rs` | 39,666 | 1 | Test |

### 2b. MPSC Unbounded

| File | Line | Context |
|------|------|---------|
| `nt_io_hotreload/mod.rs` | 137 | File watcher events |
| `nt_io_plugin/registry.rs` | 436 | Plugin directory watcher events |
| `nt_shield_proxy_kernel/kernel.rs` | 128 | Listener error channel |

### 2c. Broadcast

| File | Line | Capacity | Context |
|------|------|----------|---------|
| `nt_core_event_bus.rs` | 62 | Configurable | CoreEvent bus |
| `nt_core_event_bus.rs` | 86 | 1024 | EventBus with persistence |

### 2d. Watch

| File | Line | Context |
|------|------|---------|
| `nt_mind_background_loop/mod.rs` | 58 | Shutdown signal (`false`→`true`) |
| `nt_shield_proxy_kernel/kernel.rs` | 127 | Shutdown signal (`false`→`true`) |

### 2e. Std MPSC (sync)

| File | Line | Context |
|------|------|---------|
| `nt_act_goal/behavioral_verifier.rs` | 26 | Sync test channel |
| `nt_io_provider/factory.rs` | 1436 | Sync↔async bridge for gateway init |

---

## 3. Defects Identified

### Defect 1: EventBus Broadcast Capacity Hardcoded at 1024 — No Backpressure, Silent Drop-oldest

**Severity: HIGH** | **File: `nt_core_event_bus.rs:86`**

The `EventBus::with_persistence()` hardcodes `broadcast::channel(1024)`. This is a power-of-two, so actual buffer = 1024. But:

1. **No backpressure to producers.** `broadcast::Sender::send()` never blocks. If 9 layer subscribers (L1-L9 via `subscribe_all_layers`) are slow, the ring buffer fills and the *oldest event is silently overwritten*. The `Lagged(n)` error is only logged as a warning at line 388/471 — no structured metric, no recovery.

2. **Silent data loss for event sourcing.** The EventBus supports persistence via `EventEnvelope` (line 12-21), but the broadcast channel can lose events before slow subscribers read them. If any of the 9 layer subscribers lags, events are dropped from the ring buffer. The persistence log (line 163) writes *before* broadcast, so the log survives, but the broadcast receivers lose data silently.

3. **Clone loses sync_handlers and hooks.** `EventBus::clone()` (line 40-51) creates empty `sync_handlers` and empty `hooks` for the clone. If a cloned EventBus is used as a producer, its hooks and sync_handlers are not invoked — only the original's are.

**Impact:** Critical events (e.g., `GlobalHalt`, `ConsciousnessCritique` with low quality) can be lost if a single subscriber is slow. 1024 is small for a system with 9 subscribers + background loop handler.

**Fix:** Increase default capacity (e.g., 4096 or 8192). Add structured `broadcast_drops_total` counter (see kuksa-databroker #200 pattern). Consider `async-broadcast` with `overflow::Block` policy for critical events.

---

### Defect 2: `try_send` Silently Drops Messages in ElementBus

**Severity: MEDIUM** | **File: `nt_mind/consciousness/element/bus.rs:49`**

```rust
let _ = sender.try_send(payload.clone());
```

`ElementBus::publish()` uses `try_send()` on per-subscriber `mpsc::channel(64)`. If a subscriber is slow (buffer full), the message is **silently dropped** — no error, no metric, no retry. The `let _ =` discards the `TrySendError::Full` variant.

**Impact:** Consciousness element events (CapabilityUpdated, MemoryStored, GoalStateChanged, etc.) can be silently lost. A slow subscriber will miss state changes with no indication. The bus has 5 `EventKind` variants — if any subscriber for any kind falls behind, events vanish.

**Fix:** Either (a) use `send().await` for async backpressure (requires `publish` to be async), (b) log dropped messages as a metric, or (c) increase buffer size and add monitoring via `tx.capacity()` / `tx.available_capacity()`.

---

### Defect 3: Unbounded Channels for File Watchers — No Memory Bound

**Severity: MEDIUM** | **Files: `nt_io_hotreload/mod.rs:137`, `nt_io_plugin/registry.rs:436`**

Both file watcher systems use `mpsc::unbounded_channel()` for `notify::Event` results:

```rust
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();
```

With unbounded channels:
- `send()` is synchronous — always completes, never blocks.
- No backpressure — if the consumer task is slow (e.g., processing a WASM plugin load), file events queue unboundedly.
- A burst of file changes (e.g., `git checkout` on a directory with many plugins) can cause memory spike.

The `notify` crate itself sends events at OS filesystem notification rate, which can be very high on macOS FSEvents (coalesced batches).

**Impact:** During rapid file changes, unbounded event queue can grow without limit. In practice, plugin hot-reload is infrequent, so risk is low but the pattern is architecturally unsound per R-P1 (no implicit unbounded queuing).

**Fix:** Replace with `mpsc::channel(N)` where N is sized for expected burst (e.g., 256 or 512). The consumer loop already exists at `registry.rs:452` — just change the channel type.

---

### Defect 4: `broadcast::channel` Capacity Mismatch — `EventBus::new()` vs `with_persistence()`

**Severity: LOW** | **File: `nt_core_event_bus.rs:61-96`**

```rust
// EventBus::new()
let (sender, _) = broadcast::channel(capacity);  // line 62 — caller-provided

// EventBus::with_persistence()
let (sender, _) = broadcast::channel(1024);      // line 86 — hardcoded
```

`EventBus::new(capacity)` accepts a caller-provided capacity, but `with_persistence()` ignores the parameter and hardcodes 1024. The `Default` impl (line 54-57) calls `Self::new(1024)`. This means:

- If a user creates `EventBus::new(64)` for a low-latency test, but later switches to `with_persistence()`, they silently get 1024.
- Capacity is not preserved across the two constructors.

**Impact:** Inconsistent channel sizing. The `subscribe_all_layers` test (line 487) uses `EventBus::new(16)` — fine for tests but creates a 16-slot broadcast buffer for 9 subscribers, meaning just 2 events can lag a subscriber by 1.

**Fix:** Make `with_persistence` accept a capacity parameter, or extract a shared builder.

---

### Defect 5: `std::sync::mpsc` Used Inside Async Context (Potential Block)

**Severity: MEDIUM** | **File: `nt_io_provider/factory.rs:1436`**

```rust
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    let rt = tokio::runtime::Runtime::new()?;
    let gateway = rt.block_on(fut);
    std::mem::forget(rt);
    let _ = tx.send(gateway);
});
return rx.recv().unwrap_or_else(|_| GatewayV2::new());
```

This is an intentional design to avoid "Cannot start a runtime from within a runtime" panic. However:

1. `rx.recv()` **blocks the current thread**. If this code runs inside a tokio task (it does — via `init_reasoning_engine` → `create_gateway`), it blocks the tokio worker thread, potentially degrading runtime performance.

2. `std::mem::forget(rt)` leaks the Tokio runtime. The runtime's blocking pool threads are never joined. This is documented as intentional ("进程生命周期内保持存活") but means:
   - Blocking pool threads are never reclaimed.
   - If the spawned thread panics before `tx.send(gateway)`, `rx.recv()` returns `Err` and falls back to `GatewayV2::new()` — empty gateway with no providers.

3. No timeout on `rx.recv()`. If the gateway init hangs (e.g., network timeout not configured), the tokio worker thread is blocked indefinitely.

**Impact:** Blocking a tokio worker thread during gateway initialization can stall all other tasks on that worker. With work-stealing scheduler, the impact is mitigated but not eliminated.

**Fix:** Use `tokio::task::spawn_blocking` instead of `std::thread::spawn` + `rx.recv()`. Or use `tokio::sync::oneshot` + `spawn_blocking` to avoid blocking the async executor.

---

### Defect 6: No Graceful Shutdown for Broadcast Layer Subscribers

**Severity: LOW** | **File: `nt_core_event_bus.rs:401-412`**

```rust
pub fn subscribe_all_layers(bus: &EventBus) -> Vec<tokio::task::JoinHandle<()>> {
    vec![
        subscribe_layer(bus, LayerId::L1Body),
        // ... 8 more
    ]
}
```

Returns `JoinHandle<()>` but there is no shutdown signal wired to these handles. The `subscribe_layer` function (line 350-396) runs `loop { rx.recv().await }` and only breaks on `RecvError::Closed`. The channel closes when all `Sender` clones are dropped, but:

1. The `EventBus` holds the sender, and `EventBus` may live for the process lifetime.
2. There is no `CancellationToken` or `watch::Receiver<bool>` wired into the recv loop.
3. To stop these subscribers, you must drop the `EventBus` (and all clones), which may not be the desired shutdown behavior.

**Impact:** Layer subscribers run forever. No graceful drain on shutdown. If the system needs to shut down cleanly (e.g., persist in-flight events), these loops don't participate.

**Fix:** Add a `watch::Receiver<bool>` shutdown signal to `subscribe_layer` and break on it. This matches the pattern already used in `nt_mind_background_loop/run.rs:910` (`_ = rx.changed()`).

---

### Defect 7: Mock Stream Channel Capacity (16) Is Too Small for Production-like Testing

**Severity: LOW** | **File: `nt_io_agent_loop.rs:1332`**

```rust
let (tx, rx) = mpsc::channel(16);
```

The mock LLM stream uses capacity 16, while all real providers (OpenAI, Anthropic, Ollama, Gemini, free_providers) use capacity 64. This means:

- Tests that exercise backpressure behavior with the mock will trigger backpressure at 16 items, while production triggers at 64.
- `try_send` at line 1334 and 1365 silently drops on `Full` — in the mock, this happens at 16 chars, not 64.

**Impact:** Test fidelity gap. Backpressure-related bugs may not reproduce in tests.

**Fix:** Match mock capacity to production capacity (64) or parameterize via config.

---

## 4. Channel Selection Guide for NeoTrix

| Use Case | Current | Recommended | Reason |
|----------|---------|-------------|--------|
| EventBus (multi-subscriber broadcast) | `broadcast(1024)` | `broadcast(4096)` + metrics | 9 subscribers need more headroom |
| LLM streaming response | `mpsc(64)` | `mpsc(64)` ✓ | Appropriate for streaming chunks |
| ElementBus (per-subscriber) | `mpsc(64)` + `try_send` | `mpsc(256)` + `send().await` or metric on drop | Silent drops are unacceptable for consciousness events |
| File watcher events | `unbounded` | `mpsc(256)` | Bound the queue per R-P1 |
| Shutdown signal | `watch(false)` | `watch(false)` ✓ | Correct pattern |
| Gateway init sync bridge | `std::sync::mpsc` | `tokio::sync::oneshot` + `spawn_blocking` | Avoid blocking tokio worker |

---

## 5. Priority Ranking

| # | Defect | Severity | Effort | Impact |
|---|--------|----------|--------|--------|
| 1 | EventBus broadcast capacity too small, no drop metrics | HIGH | Low | Silent event loss across 9 layers |
| 2 | ElementBus `try_send` silently drops consciousness events | MEDIUM | Low | Missing state changes |
| 3 | Unbounded channels for file watchers | MEDIUM | Low | Violates bounded-queue principle |
| 5 | `std::sync::mpsc` blocks tokio worker thread | MEDIUM | Medium | Runtime degradation during init |
| 4 | Broadcast capacity mismatch between constructors | LOW | Trivial | Inconsistent sizing |
| 6 | No graceful shutdown for layer subscribers | LOW | Low | Clean shutdown gap |
| 7 | Mock channel capacity mismatch | LOW | Trivial | Test fidelity gap |

---

## 6. Sources

- Tokio docs: `tokio::sync::mpsc`, `tokio::sync::broadcast`, `tokio::sync::watch`
- Tokio tutorial: Backpressure and bounded channels
- kuksa-databroker #200: `broadcast_drops_total` OTel counter pattern
- mea #88: Broadcast overflow policy taxonomy (tail-drop / strict-backpressure / drop-newest)
- Rust forum: async broadcast with backpressure signaling patterns
- Tokio discussion #5948: watch vs broadcast — "a receiver may miss a value if it's too slow"
