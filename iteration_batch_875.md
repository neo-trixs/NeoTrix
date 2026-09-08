# Iteration Batch 875 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Runtime Metrics (9)
- console-subscriber: Tokio task visualization
- tokio-metrics: runtime metrics collection
- RuntimeMetrics: worker utilization, poll durations
- TaskMonitor: per-task lifecycle tracking
- tokio_unstable: enable unstable features
- task naming: profiler-visible task names
- poll duration: how long tasks run
- scheduling delay: time waiting for worker
- budget_forced_yield_count: cooperative scheduling metric

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

## Defects Identified (42+)

### Async Runtime Metrics (14)
| ID | Defect | Severity |
|----|--------|----------|
| D-METRIC-001 | HeartbeatAggregator sync HashMap, no RuntimeMetrics | High |
| D-METRIC-002 | 40+ handlers spawned without TaskMonitor::instrument() | High |
| D-METRIC-003 | CognitiveLoadMonitor synthetic (no real RuntimeMetrics) | High |
| D-METRIC-004 | ParallelExecutor silently discards JoinHandle errors | Medium |
| D-METRIC-005 | Zero tokio-metrics/console-subscriber in Cargo.toml | High |
| D-METRIC-006 | EventBus broadcast lag logged but never metricated | Medium |
| D-METRIC-007 | No task names (tokio-console task list useless) | Medium |
| D-METRIC-008 | No poll_time_histogram or schedule_latency_histogram | High |
| D-METRIC-009 | No budget_forced_yield_count monitoring | High |
| D-METRIC-010 | Shutdown aborts tasks without knowing their state | Medium |
| D-METRIC-011 | No worker thread busy_ratio computation | High |
| D-METRIC-012 | Multiple throwaway runtimes fracture observability | Medium |
| D-METRIC-013 | No spawn_blocking pool monitoring | Medium |
| D-METRIC-014 | println! in async context (100+ instances) | Low |

### Async Task Scheduling (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-SCHED-001 | 30+ handlers with no cooperative budget accounting | High |
| D-SCHED-002 | ParallelExecutor serializes tasks (defeats work-stealing) | High |
| D-SCHED-003 | EventBus subscriber loops lack cooperative yields | Medium |
| D-SCHED-004 | HeartbeatAggregator synchronous, blocks Tokio workers | High |
| D-SCHED-005 | No task priority system | High |
| D-SCHED-006 | Mutex held across entire handler ticks (convoy effects) | High |
| D-SCHED-007 | std::sync::Mutex in parallel coordinator blocks workers | High |
| D-SCHED-008 | AsyncSafetyWrapper is static lint, not runtime enforcement | Medium |
| D-SCHED-009 | Ad-hoc tasks bypass coordinated shutdown | Medium |
| D-SCHED-010 | Timer synchronization can flood one worker thread | Low |

### Async Cancellation Safety (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-CSAFE-001 | tokio::io::copy in select! with shutdown (partial transfer loss) | High |
| D-CSAFE-002 | write_all/read_exact in fire-and-forget handlers | Medium |
| D-CSAFE-003 | Mutex::lock().await inside biased select! (starvation) | Medium |
| D-CSAFE-004 | copy_bidirectional without cancellation guard | Medium |
| D-CSAFE-005 | tokio::join! with io::copy — no shutdown | Medium |
| D-CSAFE-006 | watch::Receiver::borrow() after changed() race | Low |
| D-CSAFE-007 | No JoinSet for connection handlers | Medium |
| D-CSAFE-008 | biased; select! delays shutdown under lock contention | Low |

### Structured Concurrency (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-SCON-001 | BackgroundLoop Vec<JoinHandle> instead of JoinSet | High |
| D-SCON-002 | Shutdown deadline shared across 40+ handlers | High |
| D-SCON-003 | LLM providers fire-and-forget streaming tasks | High |
| D-SCON-004 | ParallelExecutor executes sequentially (false concurrency) | Medium |
| D-SCON-005 | EventBus 9-layer subscribers no lifecycle tracking | Medium |
| D-SCON-006 | ProxyKernel spawns 4+ tasks without tracking | Medium |
| D-SCON-007 | Batch processor silently swallows JoinError panics | Medium |
| D-SCON-008 | spawn_handler! macro no error propagation | Low |
| D-SCON-009 | ReasoningEngine fire-and-forget token streaming | Low |
| D-SCON-010 | Tor crawler tasks lack structured lifecycle | Medium |

## Key Insights (This Batch)

1. **spawn_handler! is single highest-leverage fix**: Adding TaskMonitor::instrument() + task naming inside this macro would instrument all 40+ handlers in one change.

2. **Zero tokio-metrics/console-subscriber**: Cannot see task poll duration, scheduling delay, or budget exhaustion. Entire async executor is opaque.

3. **ParallelExecutor serializes tasks**: Defeats work-stealing. Tasks that should run in parallel actually run sequentially.

4. **HeartbeatAggregator synchronous**: Blocks Tokio workers during health checks. Must be async.

5. **No task priority system**: Tokio has none, NeoTrix implements none. Consciousness-critical tasks cannot be prioritized.

6. **Mutex held across entire handler ticks**: Creates convoy effects. One slow handler blocks all others.

7. **tokio::io::copy in select! with shutdown**: Partial transfer loss when shutdown cancels mid-write. Must use cancel-safe alternative.

8. **40+ handlers with no cooperative budget accounting**: Tasks can run unconstrained and starve others.

9. **LLM providers fire-and-forget streaming**: Orphaned HTTP connections on shutdown.

10. **println! in async context (100+ instances)**: Blocks Tokio worker. Must use tracing.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 875 |
| New defects (this batch) | 42 |
| Cumulative defects | D01-D78289 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 99,283+ |
