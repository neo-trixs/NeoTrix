---
name: performance
description: "Performance engineering — profiling, benchmarking, optimization, and bottleneck elimination"
version: "1.0.0"
author: "NeoTrix"
triggers: "performance|profil|benchmark|optimi|bottleneck|latency|throughput|memory|cpu|flamegraph|perf"
---

# Performance — Profiling, Benchmarking, Optimization

Systematic approach to measuring, analyzing, and improving performance. Measure first, optimize second.

## Golden Rule

> "Premature optimization is the root of all evil" — Knuth
> But: "We forget that there is a cost for not optimizing" — Hashimoto

**Profile → Measure → Identify bottleneck → Optimize → Verify**

## Profiling Tools (Rust)

| Tool | Type | Use Case |
|------|------|----------|
| `cargo flamegraph` | CPU sampling | Find hot paths |
| `cargo bench` | Micro-benchmark | Compare implementations |
| `criterion` | Statistical benchmark | Reliable benchmarks |
| `dhat` | Heap profiling | Memory allocation patterns |
| `memchr`/`bumpalo` | Custom allocators | Allocation-heavy code |
| `tracing` + `tokio-console` | Async runtime | Task scheduling, waits |
| `perf`/`samply` | OS-level | System call analysis |

## Benchmarking Framework

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse_large_input", |b| {
        b.iter(|| parse(&large_input()))
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

### Benchmark Best Practices
- Run in release mode (`--release`)
- Use ` criterion::black_box()` to prevent optimization
- Warm up before measuring
- Run multiple iterations for statistical significance
- Compare against baseline

## Optimization Playbook

### CPU Bound
1. Profile with flamegraph
2. Identify hot loops
3. Consider: algorithm change, SIMD, parallelism
4. Measure before/after

### Memory Bound
1. Profile allocations (dhat/valgrind)
2. Reduce allocations in hot paths
3. Consider: arena allocation, object pooling, zero-copy
4. Track memory over time

### I/O Bound
1. Trace async tasks (tokio-console)
2. Identify blocking in async contexts
3. Consider: batching, pipelining, connection pooling
4. Monitor queue depths

### Concurrency
1. Profile lock contention
2. Consider: lock-free structures, sharding, read-write locks
3. Avoid: unnecessary synchronization, over-parallelism

## Performance Budgets

```yaml
# Example budgets
api_p99_latency: 100ms
startup_time: 500ms
memory_baseline: 50MB
binary_size: 20MB
```

## Continuous Benchmarking

- Run benchmarks on every PR
- Detect regressions automatically
- Track trends over time
- Alert on threshold violations

## Integration Points

- **architecture-auditor**: Performance metrics in audit reports
- **Observability**: Latency/throughput metrics feed dashboards
- **CI/CD**: Benchmark gates in pipeline
- **Deployment**: Resource limits informed by profiling
