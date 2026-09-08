# Agent 2: Async Task Monitoring (Batch 869)

## Sources
1. https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html — Tokio RuntimeMetrics API: num_alive_tasks, global_queue_depth, worker stats, poll_time_histogram, budget_forced_yield_count
2. https://docs.rs/tokio-metrics/latest/tokio_metrics/ — tokio-metrics crate: TaskMonitor, RuntimeMonitor, TaskMetrics (instrumented_count, mean_poll_duration, mean_scheduled_duration, slow_poll_ratio, long_delay_ratio)
3. https://github.com/tokio-rs/tokio-metrics — TaskMonitor::instrument pattern, RuntimeMonitor intervals, metrics-rs integration for Prometheus export
4. https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.TaskMonitor.html — Per-task instrumentation: first_poll_delay, scheduled_duration, idle_duration, slow_poll_threshold, long_delay_threshold
5. https://github.com/tokio-rs/console — tokio-console: task lifecycle visibility (RUNNING/IDLE/SCHED/DONE), resource contention, self-wakes/lost-waker/never-yielded warnings
6. https://docs.rs/tokio-console/latest/tokio_console/ — Console task details: poll histograms, scheduled time, busy/idle ratio, resource views
7. https://medium.com/rustaceans/a-deep-dive-into-tokio-console-6af9bfa870ee — Production debugging patterns: high busy time = CPU hog, high scheduled time = executor overload, poll count vs busy time ratio
8. https://medium.com/rustaceans/tokio-console-in-production-94a6ce4cd112 — Production observability: 5-15% overhead from console-subscriber, environment-based enable, Prometheus bridge via tracing Layer
9. https://hegdenu.net/posts/task-scheduled-time-in-console/ — Scheduled time diagnosis: tasks blocked by other tasks show high sched time, executor overload detection
10. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view — OpenTelemetry Tokio metrics: task_spawned counter, poll_duration histogram, active_tasks gauge, budget_forced_yields counter

## Defects

**D-TMON-001: HeartbeatAggregator is purely synchronous snapshot — no async runtime integration**
The `HeartbeatAggregator` at `neotrix-core/src/unified/core/nt_core_heartbeat.rs:32` is a plain `HashMap<String, ComponentHealth>` with no tokio integration. It provides a static `report()` method but never spawns a monitoring task, never polls `RuntimeMetrics`, and never exposes live system health to GWT attention routing. Per tokio-metrics best practice, runtime health should be continuously sampled via `RuntimeMonitor::intervals()` to detect task starvation, queue backlogs, and worker contention. The GWT AttentionManager cannot receive runtime-level health signals because HeartbeatAggregator has no async polling loop.
| File:line | Severity | Source |
|-----------|----------|--------|
| `nt_core_heartbeat.rs:32` | HIGH | tokio-metrics RuntimeMonitor pattern; tokio-console runtime introspection |

**D-TMON-002: Background loop spawns 30+ tasks with zero task lifecycle metrics**
The `spawn_handler!` macro at `run.rs:734-767` spawns 30+ independent tokio tasks (save, consolidate, goal, consciousness_tick, etc.) each with their own `tokio::time::interval`. None are wrapped with `TaskMonitor::instrument()`, so there is no visibility into per-handler poll duration, scheduling delay, or slow-poll ratio. If any handler becomes compute-bound (blocks executor), the only symptom would be other handlers' scheduling delays increasing — but nothing tracks `mean_scheduled_duration` or `long_delay_ratio`. The background loop is the nervous system of NeoTrix consciousness; blind spots here mask systemic degradation.
| File:line | Severity | Source |
|-----------|----------|--------|
| `run.rs:734-839` | HIGH | tokio-metrics TaskMonitor::instrument; tokio-console never-yielded warning |

**D-TMON-003: EventBus JoinHandles dropped on Clone — async subscriber tasks have no lifecycle tracking**
`EventBus::clone()` at `nt_core_event_bus.rs:40-52` creates a new empty `handles: Vec` for each clone, discarding the JoinHandles from the original. The `subscribe_all_layers()` function at line 401 returns handles but there is no mechanism to monitor whether subscriber tasks are alive, lagging, or stuck. The broadcast channel's `RecvError::Lagged(n)` at line 387 is logged but never exposed as a metric. Per tokio-console patterns, lost wakers and lagged events should trigger warnings and feed into runtime health dashboards.
| File:line | Severity | Source |
|-----------|----------|--------|
| `nt_core_event_bus.rs:40-52, 387` | MEDIUM | tokio-console lost-waker warning; broadcast lag as metric |

**D-TMON-004: Pervasive `Runtime::new().block_on()` anti-pattern prevents unified runtime monitoring**
Multiple code paths create throwaway tokio runtimes inside synchronous contexts: `nt_core_consciousness_core.rs:1868`, `nt_shield_sandbox/mod.rs:585`, `nt_shield_sandbox/judge.rs:570`, `nt_core_forecast.rs:373`, plus all test files. Each `Runtime::new()` creates an isolated executor invisible to any central metrics collection. This means: (a) no unified `live_tasks_count` across the process, (b) no ability to detect budget-forced-yields in these ephemeral runtimes, (c) resource waste from creating/destroying thread pools. The tokio-metrics `RuntimeMonitor` can only monitor a single runtime handle; scattered runtimes fracture observability.
| File:line | Severity | Source |
|-----------|----------|--------|
| `nt_core_consciousness_core.rs:1868`, `nt_shield_sandbox/mod.rs:585` | HIGH | tokio-metrics RuntimeMonitor single-handle constraint; OpenTelemetry active_tasks gauge |

**D-TMON-005: ParallelExecutor silently swallows JoinHandle errors — no task failure tracking**
`ParallelExecutor::execute()` at `nt_core_parallel/executor.rs:41` uses `if let Ok(res) = handle.await` which silently drops Err (cancelled/panicked tasks). There is no counter for task failures, no tracking of dropped_count, and no distinction between task cancellation and task panic. Per tokio-metrics `TaskMetrics`, `dropped_count` and `instrumented_count` should be tracked to detect task lifecycle anomalies. In a multi-agent coordinator, dropped tasks could silently lose work without any diagnostic signal.
| File:line | Severity | Source |
|-----------|----------|--------|
| `nt_core_parallel/executor.rs:41` | MEDIUM | tokio-metrics TaskMetrics dropped_count; tokio-console task DONE state |

**D-TMON-006: No `budget_forced_yield_count` monitoring — compute-heavy tasks undetectable**
Tokio's `budget_forced_yield_count` (available via `RuntimeMetrics` with `tokio_unstable`) tracks how often tasks exhaust their cooperative scheduling budget and are forced to yield. NeoTrix's background loop handlers and parallel executor tasks never check for this metric. A handler that does excessive synchronous computation (e.g., KB maintenance, knowledge_aging) could monopolize a worker thread without yielding, causing starvation of other handlers on the same thread. The `slow_poll_ratio` from tokio-metrics would also catch this, but neither metric is collected.
| File:line | Severity | Source |
|-----------|----------|--------|
| `run.rs:739-758` (all handlers) | HIGH | tokio RuntimeMetrics budget_forced_yield_count; tokio-metrics slow_poll_ratio |

**D-TMON-007: Scheduler heartbeat decoupled from tokio runtime task health**
The `nt_core_scheduler::engine` at `engine.rs:127-163` implements a logical heartbeat (`report_heartbeat` / `stale_jobs`) that tracks job-level liveness via wall-clock timestamps. However, this is entirely decoupled from tokio runtime task metrics. A scheduler job can have a fresh heartbeat timestamp while its backing tokio task is experiencing high `mean_scheduled_duration` (scheduling delay) or `long_delay_ratio` (executor overload). The scheduler's `heartbeat_stats()` reports healthy while the runtime is actually starving. These two monitoring layers must be fused for accurate health assessment.
| File:line | Severity | Source |
|-----------|----------|--------|
| `nt_core_scheduler/engine.rs:127-163` | MEDIUM | tokio-metrics mean_scheduled_duration; OpenTelemetry task scheduling latency histogram |

## Key Insights

1. **Observability gap is architectural, not incidental**: NeoTrix has zero `tokio-metrics` or `tokio-console` integration despite spawning 30+ long-lived background tasks. The `HeartbeatAggregator` provides component-level health but has no visibility into the async runtime substrate that drives all components. This is like having a health monitor that checks blood pressure but never looks at the heart rate.

2. **`Runtime::new()` proliferation fragments monitoring**: At least 5 production code paths and 15+ test paths create isolated tokio runtimes. Each is an observability blind spot. A single process-level runtime (or at minimum, a shared runtime handle) would enable unified metrics collection.

3. **Background loop is the highest-risk unmonitored surface**: The 30+ `spawn_handler!` tasks are the execution backbone of the entire consciousness system. A single compute-bound handler (e.g., `handle_knowledge_aging`, `handle_novel_ingest`) could starve all other handlers with no diagnostic signal. Adding `TaskMonitor::instrument()` to the `spawn_handler!` macro is the highest-leverage fix.

4. **Two separate heartbeat systems with no bridge**: The scheduler's logical heartbeat (`nt_core_scheduler`) and the physical runtime health (which should come from tokio-metrics) are independent. A stale scheduler heartbeat could be a false positive (network delay) or a true positive (task dead), but there's no runtime-level data to disambiguate.

5. **Production readiness requires tokio_unstable**: Many of the most valuable metrics (`budget_forced_yield_count`, `worker_steal_count`, `poll_time_histogram`) require `tokio_unstable`. NeoTrix should consider enabling this for production builds (it's already used for testing via `tokio_unstable` cfg), as the overhead of `Instant::now()` per poll is negligible compared to the diagnostic value.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources consulted | 10 |
