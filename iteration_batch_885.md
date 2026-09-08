# Iteration Batch 885 — Sources 99635-99662

## Async Channel Backpressure (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99635 | Tokio MPSC Docs | docs.rs/tokio/mpsc | 2026 | `mpsc::channel(N)` bounded = backpressure; `unbounded_channel()` = OOM risk |
| 99636 | rustfaq.org | rustfaq.org/backpressure-async-rust | 2026-04-17 | 3 send primitives: `.await` (suspend), `try_send()` (shed), `send_timeout(dur)` (SLO) |
| 99637 | Meridian Space | rustycloud.org/data_pipelines/lesson-01 | 2025 | FlowPolicy enum: Backpressure/Shed/Timed per operator edge |
| 99638 | rustz2h.com | rustz2h.com/backpressure_async | 2025 | Cascading backpressure in multi-stage pipelines; semaphore + channel combined |
| 99639 | Tokio Broadcast Docs | docs.rs/tokio/broadcast | 2026 | Slow receiver: all retained until ALL get clone; `RecvError::Lagged(n)` |
| 99640 | Brandon Wie | brandonwie.dev/rust-async-channels | 2026-06-09 | Decision: one-to-one=oneshot, stream=mpsc(N), telemetry=unbounded |
| 99641 | mermaid ringbuffer | github.com/Numeration/gyre | 2025-10-30 | LMAX Disruptor async ring buffer for Tokio; publish auto-waits when full |
| 99642 | lockfree-queue | github.com/rtj1/lockfree-queue | 2026-01-31 | SPSC ~10-20ns/op; MPMC Vyukov ~30-50ns/op; cache-padded |

**Defect categories addressed**: D-BACK-001 to D-BACK-006, D-STREAM-010, D-SCHED-006

## Async Timer Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99643 | Tokio Interval Docs | docs.rs/tokio/Interval | 2026 | First tick immediate; MissedTickBehavior::Burst default causes tick storms |
| 99644 | Tokio Source | github.com/tokio-rs/tokio/interval.rs | 2026 | Burst→timeout+period; Delay→now+period; Skip→next multiple from start |
| 99645 | pgdog #1052 | github.com/pgdogdev/pgdog/issues/1052 | 2025 | SIGSEGV from intrusive linked list corruption in timer wheel under load |
| 99646 | Tokio #7384 | github.com/tokio-rs/tokio/issues/7384 | 2025 | RFC: per-worker local wheels (Nginx pattern); lazy registration on first poll |
| 99647 | Tokio #7883 | github.com/tokio-rs/tokio/issues/7883 | 2025 | Timer starvation: budget exhausted + None scheduler → infinite busy loop |
| 99648 | backon crate | docs.rs/backon | 2026 | ExponentialBuilder + sleep(tokio::time::sleep) + Retry-After header support |
| 99649 | governor crate | github.com/boinkor-net/governor | 2026 | GCRA: single AtomicU64 per limiter; 10x faster than Mutex; 584-year overflow |
| 99650 | tower-governor | github.com/benwis/tower-governor | 2026 | Tower integration: rate limit by IP/headers/custom key; burst_size config |

**Defect categories addressed**: D-TIMER-001 to D-TIMER-006, D-RUN-016, D-SCHED-001

## Async IO Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99651 | Oxide RFD 400 | oxide.computer/rfd/400 | 2025 | Authoritative cancel-safety: write_all NOT safe, write_all_buf IS safe |
| 99652 | Tokio BufWriter | docs.rs/tokio/BufWriter | 2026 | Silently discards buffer on drop; flush().await REQUIRED before drop |
| 99653 | sunshowers | sunshowers RustConf 2025 | 2025 | Cursor-based partial write tracking for cancel-safe write_all_buf |
| 99654 | tokio-rustls | docs.rs/tokio-rustls | 2026 | TlsStream behaves like BufWriter; flush() after write mandatory |
| 99655 | fast-socks5 | github.com/Numeration/fast-socks5 | 2026 | Typestate-based SOCKS5 API; no unsafe; bidirectional forwarding pattern |
| 99656 | hotpath-rs | github.com/hotpath/hotpath-rs | 2026 | `io!` macro wraps AsyncRead/Write; reports bytes, rate, P95 latency |
| 99657 | biriukov.dev | biriukov.dev/async-io | 2025 | Bidirectional copy prevents read starvation in proxy; loop-select creates implicit backpressure |
| 99658 | rust#136025 | github.com/rust-lang/rust/issues/136025 | 2025-01 | BufWriter flush is necessary but not sufficient; need sync_all for NFS |

**Defect categories addressed**: D-IO-001 to D-IO-010, D-CSAFE-003, D-CANCEL-003

## Runtime Configuration (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99659 | Tokio Builder | docs.rs/tokio/Builder | 2026 | worker_threads defaults to CPU count; event_interval default 61; LIFO slot optimization |
| 99660 | Tokio #7980 | github.com/tokio-rs/tokio/discussions/7980 | 2026-03-20 | Single runtime correct for vast majority; ONCE for app; libraries never create runtimes |
| 99661 | tokio-cpu-runtime | docs.rs/tokio-cpu-runtime | 2026 | Dedicated CPU runtime: cap workers at max(1, available_parallelism - reserve) |
| 99662 | tokio-metrics | github.com/tokio-rs/tokio-metrics | 2026 | RuntimeMonitor→intervals(); TaskMonitor per-task metrics; Prometheus export |
| 99663 | console-subscriber | github.com/tokio-rs/console | 2026 | Real-time task scheduling; requires tokio_unstable cfg + tracing feature |
| 99664 | Tokio coop blog | tokio.rs/blog/2020-04-preemption | 2020 | Budget=128 ops/poll; consume_budget() in compute loops; cooperative() wrapper |
| 99665 | tokio-graceful | github.com/plabayo/tokio-graceful | 2026 | Guard-based shutdown: strong/weak guards; shutdown_with_limit(timeout) |
| 99666 | Tokio shutdown guide | tokio.rs/tokio/topics/shutdown | 2026 | Signal→cancel→drain triad: CancellationToken + TaskTracker + timeout |

**Defect categories addressed**: D-RUN-001 to D-RUN-016, D-SCHED-001 to D-SCHED-007, D-CANCEL-001 to D-CANCEL-007

## New Defects (Batch 885)

### D-CHAN-001 HIGH: ElementBus try_send drops consciousness state transitions (D-BACK-002 reconfirmed)
- **Evidence**: Tokio docs confirm try_send returns Err(Full) — silent drop
- **Impact**: Consciousness state transitions lost permanently
- **Fix**: Replace ElementBus with bounded mpsc + FlowPolicy per operator edge

### D-CHAN-002 HIGH: Broadcast channel slow receiver loses events O(n²) waker (D-BACK-002 reconfirmed)
- **Evidence**: rustz2h.com confirms broadcast retains until ALL receivers get clone
- **Impact**: O(n²) wake overhead; slow consumer blocks all
- **Fix**: Use watch for latest-value; broadcast only for event systems

### D-CHAN-003 HIGH: 3 unbounded channels hotreload/proxy/plugin (D-BACK-001 reconfirmed)
- **Evidence**: Tokio docs: unbounded_channel = no backpressure = OOM risk
- **Impact**: Unbounded memory growth under load
- **Fix**: Replace with bounded mpsc + try_send for load-shedding

### D-TIMER-003 CRITICAL: nt_core_forecast sleep up to 3072s 51min (D-TIMER-004 reconfirmed)
- **Evidence**: Tokio timer wheel: L5 slot = 4096ms, total range ~28 hours; 3072s = 51min
- **Impact**: 51-minute sleep blocks worker thread; timers on that thread stop
- **Fix**: Replace with CancellationToken::cancelled() with periodic wake

### D-TIMER-005 HIGH: Default Burst behavior causes tick storms (D-TIMER-002 reconfirmed)
- **Evidence**: Tokio docs: Burst fires as fast as possible until caught up; many users hit this
- **Impact**: After worker starvation, burst catch-up floods CPU with timer events
- **Fix**: Set MissedTickBehavior::Skip for all interval handlers

### D-IO-011 HIGH: BufWriter drop silently discards unflushed data (D-IO-010 reconfirmed)
- **Evidence**: Tokio docs: "silently discards buffer on drop"; rust#136025 confirms
- **Impact**: Data loss on graceful shutdown without explicit flush
- **Fix**: Always flush().await before dropping BufWriter; use shutdown().await

### D-RUN-017 HIGH: timer wheel intrusive list corruption SIGSEGV under load
- **Evidence**: pgdog #1052: SIGSEGV from timer entry dropped while in wheel linked list
- **Impact**: Process crash under high timer creation rate
- **Fix**: Monitor timer creation rate; reduce per-query timeout creation; upgrade tokio

### D-RUN-018 HIGH: Budget exhaustion + None scheduler = infinite busy loop
- **Evidence**: Tokio #7883: defer() calls wake_by_ref() immediately when scheduler=None
- **Impact**: Worker thread blocked; all timers on that thread stop firing
- **Fix**: Ensure cooperative budget is not exhausted in long-running tasks

### D-IO-012 MED: TLS TlsStream requires explicit flush after every write
- **Evidence**: tokio-rustls docs: "TlsStream will behave like BufWriter"
- **Impact**: Data stays in rustls buffer until flushed to TcpStream
- **Fix**: Always flush() after write_all() to TlsStream

### D-IO-013 MED: Unbounded write_buf growth in BufWriter
- **Evidence**: No built-in limit on BufWriter buffer; consumer slowness → unbounded growth
- **Impact**: Memory exhaustion under slow consumer scenarios
- **Fix**: Use bounded mpsc channels before BufWriter; monitor buffer size
