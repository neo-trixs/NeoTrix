# Iteration Batch 873 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Runtime Metrics (9)
- console-subscriber: Tokio task visualization
- tokio-metrics: runtime metrics collection
- RuntimeMetrics: worker utilization, poll durations
- instrument: tracing integration for async
- TaskMonitor: per-task lifecycle tracking
- tokio_unstable: enable unstable features
- task naming: profiler-visible task names
- poll duration: how long tasks run
- scheduling delay: time waiting for worker

### Async Task Scheduling (9)
- work-stealing: tasks migrate between workers
- cooperative scheduling: task budget per poll
- consume_budget: check if budget exhausted
- yield_now: cooperative yield
- block_in_place: move blocking to thread pool
- spawn_blocking: async-friendly blocking
- task priority: priority levels
- local queue: per-worker task queue
- global queue: shared task queue

### Async Cancellation Safety (9)
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

## Defects Identified (37+)

### Async Runtime Metrics (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-METRIC-001 | HeartbeatAggregator blind to Tokio runtime metrics | High |
| D-METRIC-002 | Leaked runtimes (mem::forget) bypass all monitoring | High |
| D-METRIC-003 | 5+ independent runtimes with no aggregate metrics | Medium |
| D-METRIC-004 | budget_forced_yield_count never tracked | High |
| D-METRIC-005 | Event bus lag detected but not diagnosed | Medium |
| D-METRIC-006 | AsyncSafetyWrapper uses string matching, not metrics | Medium |
| D-METRIC-007 | No poll_time_histogram or schedule_latency enabled | Medium |
| D-METRIC-008 | Scheduler heartbeat decoupled from executor health | Low |

### Async Task Scheduling (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-SCHED-001 | block_in_place monopolizing workers | Critical |
| D-SCHED-002 | Wrong yield type (std::thread vs Tokio) | High |
| D-SCHED-003 | EventBus std::sync::Mutex in async context | Critical |
| D-SCHED-004 | No task priority for consciousness | High |
| D-SCHED-005 | 48 std::thread::sleep calls blocking workers | Critical |
| D-SCHED-006 | Broadcast channel event loss | High |
| D-SCHED-007 | Single-lock serializing 30+ handlers | Critical |
| D-SCHED-008 | Unbounded absorption fan-out | High |
| D-SCHED-009 | Sequential await in parallel executor | Medium |
| D-SCHED-010 | Cooperative budget blindness to SQLite/std ops | Medium |

### Async Cancellation Safety (9)
| ID | Defect | Severity |
|----|--------|----------|
| D-CSAFE-012 | Single shared deadline consumed by first handler | High |
| D-CSAFE-013 | spawn_handler! acquires Mutex inside select | High |
| D-CSAFE-014 | EventBus consumer holds Mutex across await | High |
| D-CSAFE-015 | Bidirectional tunnel select lacks biased; | Medium |
| D-CSAFE-016 | Unpinned sleep recreated each GC loop iteration | Medium |
| D-CSAFE-017 | SOCKS5/HTTP listener selects lack biased; | Medium |
| D-CSAFE-018 | Sequential drain with shared 5s budget | Medium |
| D-CSAFE-019 | write_all in SOCKS5 error path cancel-unsafe | Low |
| D-CSAFE-020 | process::exit(0) bypasses all async cleanup | Low |

### Structured Concurrency (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-SCON-001 | Vec<JoinHandle> hand-rolled JoinSet lacking abort-on-drop | High |
| D-SCON-002 | Accept loops spawn unbounded fire-and-forget tasks | High |
| D-SCON-003 | Zero JoinSet usage | High |
| D-SCON-004 | Zero TaskTracker usage | High |
| D-SCON-005 | Zero CancellationToken usage | High |
| D-SCON-006 | Cooperative shutdown structurally impossible | High |
| D-SCON-007 | select! + RwLock leaves state inconsistent on cancel | Medium |
| D-SCON-008 | Shared 5s shutdown deadline starvation | Medium |
| D-SCON-009 | No task id tracking | Low |
| D-SCON-010 | No JoinError handling | Medium |

## Key Insights (This Batch)

1. **HeartbeatAggregator blind to Tokio metrics**: The "unified system health signal collector" tracks component health via string keys but has zero integration with Tokio's 30+ runtime metrics. Missing worker starvation, queue saturation, budget exhaustion.

2. **Leaked runtimes bypass monitoring**: factory.rs uses mem::forget to leak 2 Runtimes. These bypass all monitoring and cannot be cleaned up.

3. **48 std::thread::sleep calls**: Each blocks a Tokio worker thread. Creates "blocking minefield" where workers can be stalled for seconds.

4. **Single-lock serializing 30+ handlers**: BackgroundLoop shares one Arc<Mutex<BackgroundLoopHandle>> across 30+ independent handlers. Completely negates work-stealing.

5. **block_in_place monopolizing workers**: Long-running CPU work blocks Tokio worker. Must use spawn_blocking.

6. **EventBus consumer holds Mutex across await**: Creates priority inversion. Other handlers blocked.

7. **Accept loops spawn unbounded fire-and-forget**: SOCKS5/HTTP/MITM/DNS spawn unlimited tasks. OOM under load.

8. **select! + RwLock inconsistent state**: Documented bug class (Oxide RFD 400). Cancellation leaves shared state inconsistent.

9. **Shared 5s shutdown deadline**: First slow handler starves all others. Must use per-task timeouts.

10. **No poll_time_histogram or schedule_latency**: Cannot measure task performance or scheduling fairness.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 873 |
| New defects (this batch) | 37 |
| Cumulative defects | D01-D78209 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 99,211+ |
