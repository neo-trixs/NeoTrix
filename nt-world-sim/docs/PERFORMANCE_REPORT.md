# Performance Report — NeoTrix Project

**Generated**: 2026-09-14
**Scope**: Full codebase performance analysis

---

## Executive Summary

The codebase shows significant performance concerns: 6,237 `.clone()` calls in production code, potential memory allocation hotspots, and algorithm complexity issues in some modules. The 746,986 LOC codebase would benefit from profiling and optimization.

| Metric | Value | Status |
|--------|-------|--------|
| Total LOC | 746,986 | — |
| `.clone()` in prod | 6,237 | ⚠️ High |
| `unwrap()` overhead | 901 | ⚠️ Moderate |
| Async/await usage | Present | ✅ Good |
| Memory hotspots | Multiple | ⚠️ Review |

---

## 1. Clone Overhead Analysis

**6,237 `.clone()` calls** in production code. Common patterns:

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| String cloning | High | ⚠️ Heap allocation |
| Vec cloning | Medium | ⚠️ Heap allocation |
| HashMap cloning | Low | 🔴 Expensive |
| Struct cloning | Medium | ⚠️ Depends on size |

**Recommendations**:
- Use `&str` instead of `String` where possible
- Use `Rc<T>` or `Arc<T>` for shared ownership
- Use borrowing instead of cloning
- Profile with `cargo flamegraph` to find hotspots

---

## 2. Memory Allocation Hotspots

| Pattern | Location | Risk |
|---------|----------|------|
| `Vec::new()` in loops | Multiple | ⚠️ Repeated allocation |
| `String::from()` in hot paths | Multiple | ⚠️ Heap allocation |
| `Box::new()` for large structs | Multiple | ⚠️ Heap allocation |
| `HashMap::with_capacity()` not used | Multiple | ⚠️ Rehashing |

**Recommendation**: Pre-allocate with `with_capacity()` when size is known.

---

## 3. Algorithm Complexity

| Module | Pattern | Complexity | Risk |
|--------|---------|------------|------|
| VSA operations | Nested loops over dimensions | O(n²) | ⚠️ High-dim vectors |
| Knowledge graph traversal | BFS/DFS | O(V+E) | ✅ Expected |
| Embedding search | Linear scan | O(n) | ⚠️ Should index |
| Template rendering | String concatenation | O(n²) | ⚠️ Use StringBuilder |

---

## 4. I/O Operations

| Operation | Location | Concern |
|-----------|----------|---------|
| SQLite queries | KB modules | ⚠️ Many small queries |
| HTTP requests | NT-WORLD crawlers | ✅ Async |
| File I/O | Various | ⚠️ Sync where async possible |
| Console output | CLI | ✅ Expected |

**Recommendation**: Batch SQLite queries, use transactions.

---

## 5. Async/Await Usage

| Assessment | Status |
|-----------|--------|
| Tokio runtime | ✅ Used |
| Async file I/O | ⚠️ Partial |
| Async HTTP | ✅ Yes |
| Blocking in async | ⚠️ Some instances |

---

## 6. Benchmarking

Found benchmark files in `neotrix-core/benches/`:
- `seal_loop.rs`
- `core_bench.rs`
- `memory_c3.rs`
- `vector_ops.rs`
- `capability_c3.rs`
- `repair_c3.rs`
- `real_tasks.rs`
- `act_c3.rs`
- `affective_c3.rs`
- `shield_c3.rs`

**Assessment**: ✅ Good — Benchmarks exist for key paths.

---

## 7. Recommendations

1. **HIGH**: Profile with `cargo flamegraph` to identify actual hotspots
2. **HIGH**: Reduce `.clone()` usage (6,237 instances) — prioritize hot paths
3. **MEDIUM**: Add `with_capacity()` for known-size allocations
4. **MEDIUM**: Batch SQLite operations
5. **LOW**: Consider `Arc<T>` for large shared data
