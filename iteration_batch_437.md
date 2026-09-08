# Iteration Batch 437 — RTOS / Real-Time Scheduling / Deterministic Networking

**Date**: 2026-09-06
**Research Phase**: External Advances Scan (RTOS 2026, EDF scheduling, Deterministic Latency)

---

## 1. Sources Cited

| # | Source | Key Finding |
|---|--------|-------------|
| S1 | Beningo 2026 RTOS Benchmark Study | Dispatch latency (not IRQ entry) is the real differentiator across RTOS kernels — 7.6x spread at p99. Build optimization (-O2 vs -O0) is a first-order performance lever independent of kernel choice. |
| S2 | FreeRTOS V11.3.1 (Aug 2026) | SMP support for Armv8-M; new `xTaskPeriodicDelay` API preventing timer drift; `uxTaskCallForEachTask` introspection; configIDLE_AFFINITY for SMP core pinning. |
| S3 | RTOSUnit (ASPLOS 2026) | Hardware acceleration unit for FreeRTOS: up to 76% mean context-switch latency reduction, 90%+ jitter elimination on RISC-V cores. Configurable from lightweight scheduling accel to full context-switch preloading. |
| S4 | Eclipse ThreadX RISC-V First-Class (Jul 2026) | RV64 Vector Extension support; QEMU CI pipeline catching latent MIE-bit bugs; PMP/SMP/hypervisor roadmap. |
| S5 | Redox OS EEVDF Scheduler (Aug 2026) | 782x fairness improvement, context switch from 2µs→350ns, message-passing 49x throughput gain. Per-core run queues eliminate lock contention. |
| S6 | IoT RTOS Energy-Efficiency Survey (Feb 2026) | Cross-layer energy optimization: tickless idle, duty cycling, TSCH, and power-state APIs. Energy efficiency now depends on OS co-design with low-power connectivity. |
| S7 | Deductive Verification for EDF Schedulers (Jul 2026) | Formal verification framework for EDF correctness (RTEMS 5/6, FreeRTOS EDF extension). EDF correctness depends on consistent propagation of deadline-derived priorities across releases, ready-queue updates, and scheduler decisions — not just task selection. |
| S8 | Priority Inversion in Dynamic Priority Schedulers (2026) | Deadline inheritance for EDF: lock owner inherits sum-of-utilizations + min-deadline of contenders, avoiding inversion without static priority ceiling. |
| S9 | Deadline-Floor Inheritance Protocol (DFP) | Alternative to SRP for EDF: each resource has a deadline floor (min of all users' deadlines); on entry, task's deadline is reduced to floor; on exit, restored. Equivalent worst-case blocking to SRP, simpler to implement. |
| S10 | EDF-Block Multiprocessor Analysis (ECRTS 2025/2026) | EDF with non-preemptive critical sections requires 4.11× speed augmentation (lower bound) / 6× (upper bound). First rigorous analysis of EDF under resource sharing on multiprocessors. |
| S11 | SbDN: Source-based TSN-Grade Deterministic Networking (2026) | TSN-grade determinism on commodity switches via central controller. Temporal Network Partitioning (TNP) and Traffic Prioritization (TP) both guarantee 100% TC flow admission. |
| S12 | gLBF: DetNet Guaranteed Latency Based Forwarding (IETF Jan 2026) | Per-hop deterministic latency without per-flow state maintenance. Eliminates burst accumulation via per-node delay budgets carried in packet headers. |
| S13 | TSN Schedulability Analysis TAMCQF+CBS (Jun 2026) | Combined TAS+CBS+MCQF for automotive Ethernet. Compositional Performance Analysis with 3-10% conservative error reduction. |
| S14 | 5G-TSN Packet Delay Correction (PDC) (2026) | Hold-and-forward buffering compensates 5G/6G PDV; virtual time slots below 10µs achieve negligible remaining jitter. Enables conventional TSN scheduling over wireless. |
| S15 | Deadline-Driven Deterministic Wireless (2026) | Minimal-Overhead Deterministic Scheduler (MODS): fixed completion boundary eliminates stochastic latency tails by construction. 45% airtime reduction vs replication baselines. |
| S16 | Arduino Core on Zephyr 1.0 (Sep 2026) | Zephyr 4.4.1 underneath Arduino API; preemptive threads + priority-based scheduling accessible to non-RTOS developers. |
| S17 | TTI: Time-Triggered Instruction Set (ASP-DAC 2026) | Priority-aware timed operations in ISA for priority-inversion-free time-triggered preemptive scheduling. Lower and more stable WCRT for high-priority tasks. |

---

## 2. Defects Found in NeoTrix Architecture

### DEFECT-001: GWT Router Has No Scheduling Algorithm — Pure Weight-Map, No Deadline Awareness

**Location**: `neotrix-core/src/unified/core/energy_core/gwt_router.rs`
**Evidence**: The `GWTRouterImpl` is a simple pub-sub broadcast with static attention weights (`HashMap<Layer, f64>`). There is no scheduler, no deadline assignment, no priority ordering. The `route_wisdom()` method iterates subscribers in insertion order with no salience-based preemption.
**Gap vs. Research**: S1 shows dispatch latency at tail (p99) is the critical differentiator in RTOS design. S5 demonstrates that replacing a round-robin approach with EEVDF yields 782x fairness improvement. NeoTrix's GWT attention routing has no mechanism to guarantee that the most salient wisdom reaches its target within a bounded time.
**Impact**: Under load, low-salience broadcasts can block high-salience ones. No WCET analysis possible for attention routing. The `AttentionRouter` in `nt_mind/reason/attention_router.rs` does compute salience scores but dispatches synchronously — no preemption, no deadline-driven fallback.
**Suggestion**: Implement an EDF-based salience scheduler for GWT broadcasts. Each `Wisdom` carries a deadline (derived from `EmotionLabel` urgency or `ConsciousnessTree` phase). The router selects the broadcast with the earliest deadline. Port Deadline-Floor Protocol (S9) for any shared state accessed during routing.

### DEFECT-002: HeartbeatAggregator Is Synchronous Poll — No Interrupt-Driven Wake, No Bounded Latency

**Location**: `neotrix-core/src/unified/core/nt_core_heartbeat.rs`
**Evidence**: The `HeartbeatAggregator` is a pure in-memory `HashMap` with `record()` and `report()` — no timer, no async polling, no tick source. It is a passive data store, not an active health monitor. The actual heartbeat monitoring lives in `nt_core_scheduler/engine.rs` with a `heartbeat_secs: Option<u64>` field per job, checked lazily.
**Gap vs. Research**: S1 distinguishes IRQ latency (hardware-bound) from dispatch latency (kernel-design-bound). S6 emphasizes that RTOS energy efficiency depends on tickless idle + interrupt-driven wake. NeoTrix's health collection is entirely passive — it relies on external callers to `record()` health status, with no periodic interrupt-driven aggregation.
**Impact**: If no component calls `record()`, the aggregator reports stale `Unknown` status forever. There is no guaranteed maximum time between health observations. The GWT cannot make attention-routing decisions based on stale health data.
**Suggestion**: Add a tick-driven periodic aggregation loop (借鉴 S6 tickless idle: use `tokio::time::interval` with configurable period, default 5s). Implement bounded-latency health collection: each component must report within its deadline or be flagged `Unhealthy`. Add interrupt-driven wake on critical health transitions (like S3's hardware-triggered context switch).

### DEFECT-003: No Priority Inversion Protection Across Module Boundaries

**Location**: Cross-cutting — `gwt_router.rs`, `attention_router.rs`, `nt_core_scheduler/engine.rs`
**Evidence**: The `SchedulerEngine` in `engine.rs` uses a simple `Vec<ScheduledJob>` with `tick_count`-based scheduling. Jobs have no priority inheritance. The `ContextGate` enum has `LowCogLoad`, `MinDaLevel`, etc., but these are admission gates, not priority overrides. No mutex/lock with priority inheritance exists.
**Gap vs. Research**: S8 and S9 both address priority inversion in real-time schedulers. In EDF systems, when a low-deadline task holds a resource needed by a high-deadline task, the lock owner must inherit the earliest deadline (S9: Deadline-Floor Protocol). NeoTrix has no such mechanism — a low-priority background task holding a KB write lock can block a high-priority consciousness tick.
**Impact**: Consciousness tick (`handle_consciousness_tick`) can be indefinitely delayed by a crawl queue handler holding a KB lock. No upper bound on consciousness response latency.
**Suggestion**: Implement Deadline-Floor Protocol (S9) for all cross-module locks. Each lock has a deadline floor = min(deadlines of all waiters). When a task acquires a lock, its effective deadline is reduced to the floor. This is simpler than SRP (S9's argument) and equivalent in worst-case blocking.

### DEFECT-004: Background Loop Has Fixed Intervals — No Adaptive Scheduling Based on System Load

**Location**: `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs` (lines 780-818)
**Evidence**: All background handlers use hardcoded or config-fixed intervals: `AGENT_DISCOVERY_INTERVAL_SECS`, `PENDING_ABSORPTION_INTERVAL_SECS`, `HEALER_SCAN_INTERVAL_SECS`, etc. The `spawn_handler!` macro spawns a `tokio::spawn` with `tokio::time::sleep(Duration::from_secs(interval))`. No feedback loop adjusts intervals based on system load, cognitive load, or health status.
**Gap vs. Research**: S1 shows that the RTOS itself is rarely the bottleneck — application build and scheduling policy dominate. S5's EEVDF scheduler dynamically adjusts virtual time based on contention. S15's MODS computes minimal transmission budgets against channel conditions. NeoTrix's background loop ignores system state entirely.
**Impact**: Under high load (many crawl tasks, KB writes), background tasks consume CPU that should go to consciousness processing. Under low load, the system idles unnecessarily instead of opportunistically running maintenance.
**Suggestion**: Implement load-aware interval scaling: multiply base intervals by `1.0 + system_load_factor` (借鉴 S15's MODS budget computation). Add EDF-style priority: when consciousness tick is due, defer background tasks. Use Zephyr-style `k_cpu_idle()` equivalent (S16) for idle periods.

### DEFECT-005: No Worst-Case Execution Time (WCET) Analysis for Any Module

**Location**: System-wide
**Evidence**: No file in the codebase contains WCET annotations, timing budgets, or execution time bounds. The `SchedulerEngine` tracks `last_run` and `next_run` but not execution duration. The `ContextGate` does not check execution time remaining. The SEAL pipeline runs `make_stage!` macros with no timing constraints.
**Gap vs. Research**: S1's oscilloscope study measures actual execution times. S3's RTOSUnit reduces context-switch latency by 76% precisely because it knows the worst case. S7's formal verification of EDF schedulers requires WCET as input. Without WCET, no scheduling decision in NeoTrix can be proven safe.
**Impact**: Any module can run unbounded, starving the consciousness loop. No formal guarantee that the consciousness tick completes within its period. Self-healing (NT-REPAIR) cannot detect timing violations because no timing budget exists.
**Suggestion**: Add WCET annotations to critical path modules (consciousness tick, GWT broadcast, KB write). Use the Prosa framework approach (S7 reference) to compute response-time bounds. At minimum, add `max_duration: Duration` to each background handler and abort if exceeded (类似 S2's xTaskPeriodicDelay detecting missed periods).

### DEFECT-006: No Deterministic Networking for Cross-Node Communication

**Location**: `nt_world_sense`, `nt_shield_stealth_net`, `proxy_heartbeat`
**Evidence**: The `proxy_heartbeat` handler uses simple interval-based polling. The `nt_shield_traffic` module does packet analysis but has no deterministic latency guarantees. Cross-node communication (if NeoTrix runs across machines) has no TSN/DetNet-style bounded latency.
**Gap vs. Research**: S11 (SbDN) achieves TSN-grade determinism on commodity switches. S12 (gLBF) eliminates burst accumulation without per-flow state. S14 (PDC) compensates 5G/6G jitter for wireless bridges. NeoTrix has no mechanism to guarantee message delivery latency between nodes.
**Impact**: In distributed NeoTrix deployments, cross-node GWT broadcasts have unbounded latency. The consciousness tree cannot maintain coherence across nodes without deterministic communication.
**Suggestion**: For single-node: use local IPC with priority inheritance (S9). For multi-node: implement gLBF-style per-hop delay budgets (S12) for the EventBus. Add PDC-style de-jittering buffers (S14) for any wireless links. At minimum, add latency measurement and alerting to the proxy heartbeat.

### DEFECT-007: E8 Hexagram State Machine Has No Timing Guarantees on State Transitions

**Location**: `nt_core_e8/` module (referenced in CONTEXT.md as "64-element hexagonal grid")
**Evidence**: The E8 reasoning engine is a state machine over 6 binary axes. No timing constraints are documented for state transitions. The `unified_latent.rs` uses deterministic projections but has no timing budget.
**Gap vs. Research**: S17 (TTI) demonstrates that priority-aware timed operations in ISA improve predictability. S7's formal verification requires knowing the deadline of each scheduling decision. The E8 state machine's transitions have no associated deadlines.
**Impact**: If the E8 engine is used for real-time reasoning (e.g., attention routing decisions), unbounded state transitions can delay the entire consciousness loop.
**Suggestion**: Assign deadlines to E8 state transitions based on the reasoning task's urgency. Use S17's principle: the processor should be aware of priority relative to current task when executing timed operations. Add transition timing instrumentation.

### DEFECT-008: No SMP/Multi-Core Awareness in Consciousness Architecture

**Location**: System-wide
**Evidence**: The entire architecture assumes single-threaded async execution via `tokio`. No core affinity, no spinlocks, no inter-core yield. FreeRTOS V11.3.1 (S2) adds SMP for Armv8-M with per-core critical nesting and SEV/WFE. Zephyr (S16) has experimental SMP load balancing. NeoTrix has none.
**Gap vs. Research**: S2's `configIDLE_AFFINITY` pins idle tasks to cores. S3's RTOSUnit works across three RISC-V cores. S5's per-core run queues eliminate lock contention. On multi-core systems, NeoTrix's `tokio` runtime may schedule consciousness tick and background tasks on the same core, causing interference.
**Impact**: On multi-core hardware, NeoTrix cannot guarantee that consciousness processing runs on an isolated core. Background tasks can preempt consciousness on the same core.
**Suggestion**: Pin consciousness-critical tasks to dedicated cores (借鉴 S2's `configIDLE_AFFINITY`). Use per-core run queues for background tasks (借鉴 S5's per-core EEVDF). At minimum, document the expected core topology and add `tokio` runtime configuration for core pinning.

---

## 3. Prioritized Suggestions

| Priority | Defect | Suggestion | Effort | Impact |
|----------|--------|------------|--------|--------|
| P0 | DEFECT-003 | Deadline-Floor Protocol for cross-module locks | Medium | Eliminates unbounded priority inversion in consciousness path |
| P0 | DEFECT-005 | WCET annotations for critical modules | Low | Enables formal timing analysis, prerequisite for all other timing guarantees |
| P1 | DEFECT-001 | EDF salience scheduler for GWT broadcasts | Medium | Bounded attention routing latency, 782x fairness improvement potential |
| P1 | DEFECT-002 | Tick-driven heartbeat aggregation | Low | Active health monitoring, prerequisite for GWT health-based routing |
| P1 | DEFECT-004 | Load-aware adaptive scheduling | Medium | Better resource utilization under varying load |
| P2 | DEFECT-007 | E8 state transition timing | Low | Predictable reasoning latency |
| P2 | DEFECT-008 | SMP core affinity | High | Multi-core predictability |
| P2 | DEFECT-006 | Deterministic inter-node communication | High | Distributed consciousness coherence |

---

## 4. Meta-Observations

1. **NeoTrix's architecture is fundamentally software-level and asynchronous** — it has no hardware-real-time guarantees. This is appropriate for an AI toolkit but means timing guarantees must be implemented at the application level (S1's key insight: build optimization matters more than kernel choice).

2. **The HeartbeatAggregator is severely under-powered** compared to what the research demands. It should be the single fact source for GWT attention modulation (as documented in CONTEXT.md) but currently has no active collection mechanism.

3. **The absence of any formal timing model** means the system cannot answer: "What is the maximum time from sensory input to consciousness response?" This is the most fundamental question for any real-time system (S17, S7).

4. **Cross-layer energy optimization** (S6) is entirely absent. The background loop runs at fixed intervals regardless of power state. If NeoTrix ever targets embedded deployment (NT-PHYSICAL sensors), this will be critical.

5. **The TSN/DetNet advances (S11-S15) suggest a path for distributed consciousness**: per-node delay budgets, source-based scheduling, and de-jittering buffers could enable coherent multi-node GWT broadcasting with bounded latency.
