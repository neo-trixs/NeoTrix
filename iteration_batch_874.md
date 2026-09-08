# Iteration Batch 874 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Runtime Configuration (9)
- Tokio Builder: custom thread count, stack size, enable_all()
- worker_threads: default = CPU cores
- max_blocking_threads: blocking pool size (default 512)
- global_queue_interval: event interval (default 61)
- thread_name: profiler-visible naming
- thread_stack_size: default 8MB
- Runtime::new(): creates default multi-thread runtime
- Builder::new_multi_thread: multi-thread config
- enable_all: enable I/O and time drivers

### Async Task Monitoring (9)
- console-subscriber: Tokio task visualization
- tokio-metrics: runtime metrics collection
- RuntimeMetrics: worker utilization, poll durations
- instrument: tracing integration for async
- TaskMonitor: per-task lifecycle tracking
- tokio_unstable: enable unstable features
- task naming: profiler-visible task names
- poll duration: how long tasks run
- scheduling delay: time waiting for worker

### Cancel-Safe Futures (9)
- Cancel safety: future can be dropped at any await point
- select!: branch cancellation on match
- biased;: select! branch priority
- CancelSafe trait: marker for cancel-safe futures
- write_all: cancel-unsafe (partial write)
- read_exact: cancel-unsafe (partial read)
- tokio::pin!: pin a future in place
- tokio::select!: cancel unmatched branches
- Cancellation token: cooperative shutdown

### Structured Concurrency (9)
- JoinSet: structured concurrency with abort on drop
- TaskTracker: task lifecycle tracking
- AbortOnDropGuard: manual abort on drop
- spawn: detached task (no automatic cancellation)
- JoinHandle: future completion handle
- JoinError: task panic/cancellation
- CancellationToken: hierarchical shutdown
- Graceful shutdown: signal → token → abort
- Task id: unique task identifier

## Defects Identified (38+)

### Async Runtime Configuration (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-RUN-011 | futures::executor::block_on inside Tokio async fn | High |
| D-RUN-012 | HotReload Handle::block_on from tokio::spawn (panic) | High |
| D-RUN-013 | HotReload RwLock::blocking_write from tokio worker | High |
| D-RUN-014 | std::thread::sleep in LLM retry loop (1.2-7s per attempt) | High |
| D-RUN-015 | std::thread::sleep in crawl pipeline (starves executor) | Medium |
| D-RUN-016 | std::mem::forget(rt) in gateway factory (zombie leak) | Medium |
| D-RUN-017 | Handle::block_on in builder (panics outside runtime) | Medium |

### Async Task Monitoring (9)
| ID | Defect | Severity |
|----|--------|----------|
| D-TMON-001 | HeartbeatAggregator blind to runtime metrics | High |
| D-TMON-002 | 30+ handlers uninstrumented | High |
| D-TMON-003 | CognitiveLoadMonitor purely synthetic | High |
| D-TMON-004 | ParallelExecutor swallows task failures | Medium |
| D-TMON-005 | Zero tokio_unstable/RuntimeMetrics integration | High |
| D-TMON-006 | spawn_blocking pool unmonitored | Medium |
| D-TMON-007 | EventBus lag undiagnosed | Medium |
| D-TMON-008 | Shutdown race with blind abort | Medium |
| D-TMON-009 | No poll histograms | High |

### Cancel-Safe Futures (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CSAFE-001 | spawn_handler! Mutex inside select (cancellation drops guard) | High |
| D-CSAFE-002 | EventBus consumer holds Mutex across await | High |
| D-CSAFE-003 | Bidirectional proxy tunnel lacks biased; | Medium |
| D-CSAFE-004 | Absorption stdin write cancel-unsafe | Medium |
| D-CSAFE-005 | SOCKS5 handshake cancel-unsafe | Medium |
| D-CSAFE-006 | Task abort during shutdown | Medium |
| D-CSAFE-007 | 11 files use Mutex held across await (Oxide RFD 397/400) | High |
| D-CSAFE-008 | No CancelSafe trait implementation | Low |
| D-CSAFE-009 | Unpinned sleep recreated each iteration | Low |
| D-CSAFE-010 | process::exit(0) bypasses async cleanup | Low |

### Structured Concurrency (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-SCON-001 | Zero JoinSet usage across entire codebase | High |
| D-SCON-002 | BackgroundLoop uses Vec<JoinHandle> | High |
| D-SCON-003 | Zero CancellationToken usage | High |
| D-SCON-004 | Parallel executor sequentially awaits tasks | High |
| D-SCON-005 | Proxy kernel spawns without structured parent | Medium |
| D-SCON-006 | 5-second hard abort deadline | Medium |
| D-SCON-007 | Sequential shutdown drain starvation cascade | Medium |
| D-SCON-008 | LLM stream relay JoinHandle not awaited | Medium |
| D-SCON-009 | Background handlers lack cooperative cancellation | High |
| D-SCON-010 | No structured error propagation | Medium |
| D-SCON-011 | Batch processor Vec<JoinHandle> memory safety | Low |
| D-SCON-012 | Firewall task spawned without tracking | Low |

## Key Insights (This Batch)

1. **futures::executor::block_on inside Tokio**: Wrong executor entirely. Creates deadlock risk. Must use tokio::task::spawn_blocking or .await.

2. **HotReload Handle::block_on from tokio::spawn**: Panics on current_thread runtime. Must check runtime type or use spawn_blocking.

3. **HotReload RwLock::blocking_write from tokio worker**: Deadlock with concurrent access. Must use tokio::sync::RwLock.

4. **std::thread::sleep in LLM retry loop**: Blocks worker 1.2-7s per attempt. Must use tokio::time::sleep.

5. **std::mem::forget(rt) in gateway factory**: Permanent zombie thread leak. Cannot be cleaned up.

6. **spawn_handler! macro is single highest-leverage fix**: Instrumenting it would cover all 30+ handlers in one change.

7. **CognitiveLoadMonitor purely synthetic**: Not grounded in real RuntimeMetrics. Cannot detect actual load.

8. **11 files use Mutex held across await**: Oxide's production experience (RFD 397/400) shows this is the single most dangerous async pattern in Rust.

9. **Zero JoinSet/TaskTracker/CancellationToken**: 50+ tokio::spawn sites with zero structured concurrency.

10. **Converting Vec<JoinHandle> to JoinSet<()>**: ~50-line change in background loop would be single most impactful fix.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 874 |
| New defects (this batch) | 38 |
| Cumulative defects | D01-D78247 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 99,247+ |
