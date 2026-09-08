# Agent 4: Async Timer Patterns (Batch 877)

## Sources
1. Tokio docs — `tokio::time` module (interval, sleep, timeout, MissedTickBehavior): https://docs.rs/tokio/latest/tokio/time/
2. Tokio docs — `Interval::tick()` cancel safety: https://docs.rs/tokio/latest/tokio/time/struct.Interval.html
3. Tokio docs — `sleep` function (millisecond granularity, max ~2.2 years): https://docs.rs/tokio/latest/tokio/time/fn.sleep.html
4. Tokio docs — `timeout` function (future polled before timeout check): https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
5. Tokio docs — `MissedTickBehavior` (Burst/Delay/Skip): https://docs.rs/tokio/latest/tokio/time/enum.MissedTickBehavior.html
6. Tokio GitHub — Interval burst behavior under high CPU load (#2301): https://github.com/tokio-rs/tokio/issues/2301
7. Tokio docs — `select!` macro (biased, cancel safety, timeout+shutdown): https://docs.rs/tokio/latest/tokio/macro.select.html
8. Tokio GitHub — Timeout on JoinHandle doesn't cancel task (#7213): https://github.com/tokio-rs/tokio/discussions/7213
9. Neurolaunch — `std::thread::sleep` blocks entire async executor: https://neurolaunch.com/rust-sleep
10. DeveloperLife — Cancellation safety in `select!`, interval vs sleep pattern: https://developerlife.com/2024/07/10/rust-async-cancellation-safety-tokio
11. Tokio source — `interval.rs` MissedTickBehavior implementation: https://github.com/tokio-rs/tokio/blob/master/tokio/src/time/interval.rs

## Defects

### D-TIMER-001: Background loop uses `MissedTickBehavior::Burst` (default) — under load, 30+ handlers fire back-to-back bursts, starving each other
- **File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:739`
- **Severity**: HIGH
- **Source**: Tokio docs (#2301), MissedTickBehavior docs
- **Detail**: `spawn_handler!` macro creates `tokio::time::interval()` which defaults to `Burst` behavior. When any handler takes longer than its interval (e.g. `consolidate` or `crystallization` taking >60s), the next tick fires immediately, causing a burst of rapid-fire calls. With 30+ concurrent handlers sharing the tokio executor, burst reclamation causes handler starvation cascades — a slow `cargo check` in convergence_pulse blocks all background handlers for 120s, then they all burst-fire simultaneously.

### D-TIMER-002: EventBus subscriber thread uses `std::thread::sleep(10ms)` in poll loop — blocks OS thread, freezes event delivery to all subscribers
- **File**: `neotrix-core/src/neotrix/nt_core_event_bus.rs:464`
- **Severity**: CRITICAL
- **Source**: Neurolaunch (thread::sleep blocks executor), Tokio docs
- **Detail**: `Err(TryRecvError::Empty)` branch calls `std::thread::sleep(Duration::from_millis(10))`. This blocks the entire OS thread for 10ms per empty poll. If a subscriber thread has no events, it wastes 100ms/s of OS thread time doing nothing. Under high event throughput, this creates a 10ms latency floor for event delivery. Should use `tokio::sync::broadcast::recv()` (async) or `tokio::time::sleep` if keeping the sync pattern.

### D-TIMER-003: Registry watcher uses `std::thread::sleep(self.poll_interval)` in `watch_loop` — blocks tokio runtime if called from async context
- **File**: `neotrix-core/src/neotrix/nt_core_capability_tree/src/registry_watcher.rs:252`
- **Severity**: HIGH
- **Source**: Neurolaunch, Tokio docs
- **Detail**: `watch_loop()` is documented as "同步轮询循环 (阻塞, 适合独立线程)" but `SharedWatcher::check_reload()` is used from async contexts. The exponential backoff at line 202 also uses `std::thread::sleep(backoff)` with up to `100 * 2^attempt` ms. If called from a tokio worker thread, this freezes the executor. The `RegistryWatcher` should either be fully async or the `watch_loop` must be guaranteed to only run on `spawn_blocking` threads.

### D-TIMER-004: `nt_core_forecast` LLM retry uses `std::thread::sleep` with up to 3600s duration — can block tokio worker for 1 hour
- **File**: `neotrix-core/src/unified/core/nt_core_forecast.rs:440,486`
- **Severity**: CRITICAL
- **Source**: Neurolaunch, Tokio docs
- **Detail**: Two `std::thread::sleep` calls: (1) line 440: `sleep(1200 + attempt * 800)` ms before each LLM attempt, and (2) line 486: `sleep(Duration::from_secs(retry_after))` where `retry_after` is parsed from server response and can be arbitrarily large (up to `2^attempt * 3` seconds, which at attempt=10 = 3072s ≈ 51min). If this runs inside a tokio task, the worker thread is frozen. Should use `tokio::time::sleep`.

### D-TIMER-005: Observer error recovery retry uses `std::thread::sleep` in circuit breaker path — blocks executor during error recovery
- **File**: `neotrix-core/src/unified/core/nt_core_observer_error.rs:59`
- **Severity**: MEDIUM
- **Source**: Neurolaunch, Tokio docs
- **Detail**: `ObserverErrorRecovery::execute_with_retry` calls `std::thread::sleep(Duration::from_millis(delay))` in the retry loop. The `delay` is computed by `RetryConfig::delay()` (exponential backoff). If this is called from an async context (e.g., during heartbeat aggregation or consciousness tick), the executor thread is blocked. The function signature is synchronous but callers may be in async contexts.

### D-TIMER-006: Background loop shutdown uses single 5-second deadline for ALL handlers — 30+ handlers share one abort deadline
- **File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/handlers.rs:48`
- **Severity**: MEDIUM
- **Source**: Tokio docs (cancellation safety), select! timeout patterns
- **Detail**: `let deadline = tokio::time::sleep(Duration::from_secs(5))` is shared across ALL handler handles via `tokio::select!`. The deadline is `tokio::pin!`'d and reused. But the loop iterates handles sequentially — the first handler might take 4.9s to observe the signal, leaving only 0.1s for the remaining 30+ handlers. Most handlers will be force-aborted rather than gracefully shutting down. Each handler should get its own independent deadline, or the deadline should scale with handler count.

### D-TIMER-007: `nt_world_crawl` fetcher uses `std::thread::sleep` for rate limiting between HTTP requests — blocks tokio executor
- **File**: `neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/fetcher.rs:265,278,291`
- **Severity**: HIGH
- **Source**: Neurolaunch, Tokio docs
- **Detail**: Three `std::thread::sleep` calls: (1) line 265: `sleep(delay_ms - duration_ms)` for inter-request rate limiting, (2) line 278: `sleep(backoff)` for retry backoff, (3) line 291: `sleep(5s)` for Tor safety delay. These are in synchronous `fetch()` / `fetch_with_retry()` / `fetch_tor_safe()` methods. If called from async crawl pipeline, they freeze the executor. The crawl pipeline uses `tokio::spawn` for concurrent fetches, so blocking sleep here freezes the worker thread serving that crawl task.

### D-TIMER-008: `dom_extractor` uses hardcoded 1000ms `tokio::time::sleep` in extraction loop — no adaptive DOM stability detection
- **File**: `neotrix-core/src/unified/layers/perception/nt_world/nt_world_crawl/dom_extractor/mod.rs:173`
- **Severity**: LOW
- **Source**: Tokio docs (sleep vs interval patterns)
- **Detail**: `tokio::time::sleep(Duration::from_millis(1000))` is used as a DOM stability wait. This is a fixed delay regardless of DOM change rate — if the DOM stabilizes in 100ms, 900ms is wasted; if it needs 2s, extraction starts too early. Should use either `interval`-based polling with early-exit or `tokio::time::timeout` around a DOM-changed check.

### D-TIMER-009: 30+ background handlers created via `tokio::spawn` with individual `interval` — no global concurrency limit, DoS-prone under memory pressure
- **File**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:738-887`
- **Severity**: MEDIUM
- **Source**: Tokio docs (bounded channels, backpressure), Hyaking (spawn unlimited tasks)
- **Detail**: Each `spawn_handler!` creates a new `tokio::spawn` with its own `tokio::time::interval`. There are 30+ such handlers (save, consolidate, goal, knowledge_chain, crystallization, plugin, exploration, curiosity, prediction, metacog, cleanup, backup, kb_guard, kb_backup, workspace_guard, agent_discovery, pending_absorption, daily_intel, always_on, scheduler, evolve, world_sense, proxy_heartbeat, skill_scan, nexus_weaver, healer_scan, system_health_heal, avatar_auto_distill, kb_absorb, seed_crawl_queue, session_recovery, crawl_queue, architecture_audit, novel_ingest, constitution_reload, consciousness_tick, second_brain, loop_readiness, market_re_eval, telemetry, wisdom, game_training). No semaphore or concurrency limiter gates how many can run simultaneously. Under memory pressure, all 30+ fire simultaneously, each potentially spawning sub-tasks.

### D-TIMER-010: `nt_io_provider` gateway semaphore spin-wait uses `tokio::time::sleep(20ms)` in loop — tight polling wastes executor cycles
- **File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/gateway/execution.rs:137`
- **Severity**: LOW
- **Source**: Tokio docs (best practices: yield_now in loops)
- **Detail**: The semaphore acquisition loop does `tokio::time::sleep(Duration::from_millis(20))` between `try_acquire` calls. This is a busy-wait with 20ms granularity. Under high concurrency, many tasks spin in this loop simultaneously. Should use the semaphore's built-in `acquire()` which suspends the task without polling, or at minimum use `tokio::task::yield_now()` between attempts.

### D-TIMER-011: `nt_shield_proxy_kernel` connection timeout uses `tokio::time::timeout` but retry sleep is also `tokio::time::sleep` — no circuit breaker for repeated timeout failures
- **File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:460,485`
- **Severity**: MEDIUM
- **Source**: Tokio docs (timeout + circuit breaker patterns)
- **Detail**: The retry loop at line 457 uses `tokio::time::sleep(delay)` for backoff (correct) and `tokio::time::timeout` for connection (correct). However, `self.security.should_retry()` is checked per-attempt but there's no global circuit breaker across connection attempts to the same target. If a target is consistently timing out, the proxy kernel retries `max_retries` times before giving up — each attempt consuming a timeout duration. A circuit breaker would fail-fast after the first timeout.

### D-TIMER-012: `network_monitor::tick()` chains multiple `tokio::time::sleep` calls sequentially — DNS flush + Shadowrocket suspend + DNS check adds 1s+ latency per tick
- **File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/network_monitor.rs:321-333`
- **Severity**: LOW
- **Source**: Tokio docs (sleep drift, interval patterns)
- **Detail**: `tick()` executes: `flush_dns()` → `suspend_shadowrocket()` → `sleep(500ms)` → `flush_dns()` → `check_dns_quality()` → if bad: `suspend_shadowrocket()` → `sleep(500ms)` → `flush_dns()`. This is a fixed sequential chain with 500ms+ sleep gaps. If called on a regular interval, the actual tick period is `interval + ~1000ms` due to the sleeps. Should use `interval` to maintain wall-clock cadence rather than adding sleep within the tick body.

### D-TIMER-013: `nt_core_forecast` retry parses `retry_after` from error message string — fragile parsing can yield 0 or unexpected values
- **File**: `neotrix-core/src/unified/core/nt_core_forecast.rs:475-482`
- **Severity**: LOW
- **Source**: Tokio docs (timeout error handling)
- **Detail**: `retry_after` is extracted by splitting the error message string: `.split("\"retry_after\":").nth(1).and_then(|s| s.trim_start().split(|c: char| !c.is_ascii_digit()).next())`. This string parsing is brittle — if the JSON format changes, `retry_after` becomes `None` and defaults to `2^attempt * 3` seconds. At high attempt counts, this exponential default (e.g. attempt=10 → 3072s) combined with `std::thread::sleep` creates extremely long blocking sleeps.

### D-TIMER-014: `nt_shield_stealth_net` proxy pool health check uses `tokio::time::timeout` with 5s deadline but no retry — single failure marks proxy unhealthy
- **File**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:619`
- **Severity**: MEDIUM
- **Source**: Tokio docs (timeout patterns)
- **Detail**: `tokio::time::timeout(Duration::from_secs(5), ...)` for proxy health check. A single 5s timeout (which can occur due to transient network jitter) marks the proxy as unhealthy. Should use `MissedTickBehavior::Skip`-style logic — require N consecutive timeouts before marking unhealthy, or use an `interval`-based health checker with `Delay` behavior to avoid burst health checks after transient failures.

### D-TIMER-015: `nt_io_mail` IMAP logout uses `tokio::time::timeout(5s)` but silently ignores `Elapsed` — IMAP session may hang on server
- **File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_mail/imap.rs:402`
- **Severity**: LOW
- **Source**: Tokio docs (timeout cancellation safety)
- **Detail**: `let _ = tokio::time::timeout(Duration::from_secs(5), self.session.logout()).await;` — the `Elapsed` error is silently discarded with `let _`. If the IMAP server doesn't respond to LOGOUT within 5s, the session is left in an undefined state. The timeout drops the future, but the underlying TCP connection may still be half-open. Should log the timeout and explicitly close the connection.

## Key Insights

1. **Systemic `std::thread::sleep` in async context**: 48 instances of `std::thread::sleep` across the codebase, with critical paths in `nt_core_forecast` (LLM retry), `nt_core_observer_error` (circuit breaker), `event_bus` (subscriber loop), and `registry_watcher`. Each blocks the tokio executor thread for the sleep duration.

2. **No `MissedTickBehavior` configuration anywhere**: Zero uses of `MissedTickBehavior` in the entire codebase. All 30+ background loop handlers and all interval-based patterns use the default `Burst` behavior. Under system load, this creates burst-fire cascades that starve other handlers.

3. **Background loop handler count explosion**: 30+ independent `tokio::spawn` tasks with individual `tokio::time::interval` timers, no global concurrency limiter. Each handler independently fires on its own interval. When multiple fire simultaneously (e.g. after a 120s `cargo check` blocks the executor), they create a thundering-herd effect.

4. **Shutdown deadline is shared, not per-handler**: The 5-second shutdown deadline is a single `tokio::time::sleep` shared across 30+ handler aborts. Handlers processed later in the iteration get almost no time to gracefully shut down.

5. **Rate limiting uses blocking sleep instead of async patterns**: The crawl pipeline (`fetcher.rs`) uses `std::thread::sleep` for inter-request delays. Since crawls are spawned as tokio tasks, this blocks the worker thread serving that task, reducing overall crawl parallelism.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 15 |
| Critical | 2 (D-TIMER-002, D-TIMER-004) |
| High | 3 (D-TIMER-001, D-TIMER-003, D-TIMER-007) |
| Medium | 5 (D-TIMER-005, D-TIMER-006, D-TIMER-009, D-TIMER-011, D-TIMER-014) |
| Low | 5 (D-TIMER-008, D-TIMER-010, D-TIMER-012, D-TIMER-013, D-TIMER-015) |
| Sources consulted | 11 |
