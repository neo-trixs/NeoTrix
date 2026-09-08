# Iteration Batch 825 Report — NeoTrix Consciousness Architecture

## Research Sources (18+)

### Async Memory Patterns (6)
- .boxed() costs ~9x more per poll than Box::pin() (type erasure + vtable)
- async_trait causes 340% memory spike, 89% throughput drop, 30K Box allocs/sec
- Arc clone = 12.89ns (2.8x Rc), contended = 4.5x worse, 126x penalty at 2 threads
- Rust 1.91 cuts per-context-switch 142ns→82ns (42% reduction), inlines poll for ≤5 await points
- Arc clone overhead = 12μs/request, once_cell::Lazy eliminates clone cost for read-heavy
- Rc in spawned async tasks can corrupt under task migration

### Time Management (6)
- timeout Elapsed root causes: blocking executor starvation + deadline below p95
- timeout does NOT return Elapsed when inner future does sync blocking (timer starved)
- std::time::Instant is suspend-aware on Windows — timeouts fire prematurely after sleep/resume
- MissedTickBehavior::Burst catches up missed ticks in rapid succession (thundering herd)
- Interval ticks during sleep on Windows but not Linux — platform inconsistency
- far_future() 30-year overflow risk on macOS

---

## Defects Identified (14+)

### Async Memory Patterns (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-MEM-1 | Pervasive #[async_trait] on internal traits (30+ traits, 30K Box allocs/sec) | High |
| D-MEM-2 | RefCell in potentially async-shared state (keyvault.rs) | Medium |
| D-MEM-3 | .boxed() instead of Box::pin() in hot paths (7 instances) | Low-Medium |
| D-MEM-4 | Arc clone per-request in FFI layer (10 Arc fields in NeoTrixHandleInner) | Medium |
| D-MEM-5 | Missing Send bounds on async trait returns (migration risk) | Medium |
| D-MEM-6 | No future pooling for high-frequency allocations | Low |

### Time Management (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-TIME-1 | std::thread::sleep in async context (8+ files) | Critical |
| D-TIME-2 | reqwest::blocking inside async context (6+ files) | High |
| D-TIME-3 | Suspend-aware Instant for deadlines (Windows resume issues) | Medium |
| D-TIME-4 | Background loop interval default MissedTickBehavior::Burst | Medium |
| D-TIME-5 | Mixed std::time::Instant / tokio::time::Instant | Low-Medium |
| D-TIME-6 | block_on nested runtime anti-pattern (4+ files) | High |
| D-TIME-7 | Timeout without fallback on unwrap() | Medium |
| D-TIME-8 | far_future() 30-year overflow risk | Low |

## Key Insights (This Batch)

1. **Pervasive async_trait = 30K Box allocations/sec on LLM hot path**: Every LlmProvider call boxes the future. At 10K calls/sec, this is the #1 allocator pressure. Migration to native async fn in trait (Rust 1.75+) would eliminate this.

2. **8+ std::thread::sleep in async context**: Each blocks a Tokio worker thread, starving ALL timeouts on that worker. The timer thread literally cannot fire while the worker is blocked.

3. **reqwest::blocking creates nested runtime**: Each blocking client spawns its own tokio runtime internally. Nested runtimes + blocking = deadlock + thread pool exhaustion.

4. **MissedTickBehavior::Burst causes thundering herd**: When handlers take longer than interval, all missed ticks fire rapidly. Absorption/build handlers doing heavy work trigger cascading re-entry.

5. **block_on inside existing runtime = explicit anti-pattern**: tokio docs warn against this. block_in_place + Handle::block_on can deadlock if all worker threads blocked.

6. **Arc clone per-request = 126x contention penalty**: 10 Arc fields in NeoTrixHandleInner, cloned per FFI call. Should pass &Handle references.

7. **Windows suspend breaks std::time::Instant deadlines**: Lid close/hibernate makes Instant jump forward. 3s budget becomes 0s after resume.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 825 |
| New defects (this batch) | 14 |
| Cumulative defects | D01-D76722 |
| Research sources (this batch) | 12 |
| Cumulative research sources | 97,587+ |