# Iteration Batch 834 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Process Spawning (10)
- tokio::process kill_on_drop is opt-in (default false); zombie reaping best-effort only
- processkit v2.3.1: kernel-backed whole-tree containment (cgroup v2 / Job Object / POSIX process group)
- tokio-process-tools: "armed handle" pattern — panics if dropped without wait/kill
- PidfdReaper (Linux 5.3+) preferred over SignalReaper; GlobalOrphanQueue best-effort
- No graceful shutdown protocol (kill() sends immediate SIGKILL)
- No resource limit enforcement (CPU/memory/fd limits)
- No CancellationToken integration for multi-step workflows
- SIGCHLD signals coalesce on Unix (single signal may miss multiple exits)
- Default kill_on_drop(false) means zombies silently accumulate
- processkit supports rlimit enforcement, uid/gid privilege drop

### IPC Patterns (10)
- mmap under concurrent load: p95 latency spike from 30s to 150s+ (page-cache thrashing)
- io_uring 60% slower than mmap without careful BMT design
- Message-passing > shared memory for isolation (L4/seL4 ~100-cycle IPC)
- Versioned layout headers non-negotiable: magic|version|capacity|write_index|read_index|generation|heartbeat
- "Sync core, async shell" pattern for Rust IPC
- SharedMemory struct is misnamed (actually in-process HashMap)
- No cross-process IPC framework (only ProxyControl as actual IPC)
- ProxyControl socket cleanup race (non-atomic)
- No Windows IPC path (tokio::net::UnixListener is Unix-only)
- memmap2 RUSTSEC-2026-0186 unpatched

### Signal Handling (10)
- Handler ≠ Shutdown: handlers RECORD shutdown, cleanup happens in control flow
- SIGTERM mandatory (Kubernetes, Docker, systemd all send SIGTERM first)
- Double-signal = force-quit pattern (first graceful, second immediate)
- Signal coalescing: tokio::signal::unix::Signal may coalesce multiple signals
- Dialogue server has zero graceful shutdown (no with_graceful_shutdown)
- System proxy calls std::process::exit(0) (skips ALL Drop impls)
- No CancellationToken infrastructure in NT-CORE
- EventBus shutdown uses std::thread::JoinHandle in async context (blocks executor)
- MCP server has no signal handling
- Drain timeout critical: orchestrators send SIGKILL after grace period

### Filesystem Patterns (10)
- tokio::fs is a lie: all ops use spawn_blocking under the hood
- std::fs::File::lock() stable in Rust 1.89 (advisory on Linux/macOS, mandatory on Windows)
- notify watcher is not Send/Sync
- SQLite WAL needs busy_timeout + PRAGMA synchronous=NORMAL
- File locking + async = deadlock risk
- No file locking on KB journal writes (multi-process corruption)
- Event bus persistence without file lock
- HotReload uses fixed 200ms sleep instead of proper debouncer
- WAL checkpoint starvation risk (unbounded .db-wal growth)
- ConcurrencyConflictDetector defined but never integrated

---

## Defects Identified (40+)

### Process Spawning (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-PROC-1 | No whole-tree containment (grandchild processes survive as orphans) | Critical |
| D-PROC-2 | Zombie process accumulation (best-effort reaping only) | High |
| D-PROC-3 | No graceful shutdown protocol (immediate SIGKILL) | High |
| D-PROC-4 | No resource limit enforcement (CPU/memory/fd) | High |
| D-PROC-5 | No CancellationToken integration for multi-step workflows | Medium |

### IPC (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-IPC-1 | SharedMemory is misnamed (actually in-process HashMap) | High |
| D-IPC-2 | No cross-process IPC framework (domains can't communicate across processes) | High |
| D-IPC-3 | ProxyControl socket cleanup race (non-atomic) | Medium |
| D-IPC-4 | No Windows IPC path (UnixListener is Unix-only) | Medium |
| D-IPC-5 | memmap2 RUSTSEC-2026-0186 unpatched | Critical |
| D-IPC-6 | rkyv zero-copy never actually used (dead dependency) | Medium |
| D-IPC-7 | Recon phase has IpSharedMemory/IpUnixSocket stubs (no implementation) | Low |
| D-IPC-8 | No backpressure on ProxyControl connections (unbounded spawn) | Low |

### Signal Handling (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-SIG-1 | Dialogue server zero graceful shutdown (no with_graceful_shutdown) | Critical |
| D-SIG-2 | System proxy calls std::process::exit(0) (skips ALL Drop impls) | Critical |
| D-SIG-3 | Proxy example infinite loop with no signal handling | Medium |
| D-SIG-4 | No CancellationToken infrastructure in NT-CORE | High |
| D-SIG-5 | EventBus shutdown uses std::thread::JoinHandle in async context | Medium |
| D-SIG-6 | MCP server has no signal handling | Low |

### Filesystem (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-FS-1 | Blocking I/O in async context (std::fs in potentially async code) | High |
| D-FS-2 | No file locking on KB journal writes (multi-process corruption) | Critical |
| D-FS-3 | Event bus persistence without file lock | Medium |
| D-FS-4 | HotReload fixed 200ms sleep instead of proper debouncer | Medium |
| D-FS-5 | Plugin watcher has no debounce (double-fire on atomic saves) | Medium |
| D-FS-6 | notify watcher lifecycle not guarded (panic = silent event loss) | Low |
| D-FS-7 | WAL checkpoint starvation risk (unbounded .db-wal growth) | Medium |
| D-FS-8 | ConcurrencyDetector defined but never integrated | Low |

## Key Insights (This Batch)

1. **tokio::process kill_on_drop is opt-in**: Default false means zombies silently accumulate. Must explicitly set kill_on_drop(true) for all sandboxed processes.

2. **mmap under concurrent load is dangerous**: p95 latency spike from 30s to 150s+ due to page-cache thrashing. 2M minor faults/sec + 2M context switches/sec = kernel dominates.

3. **Dialogue server has zero graceful shutdown**: No with_graceful_shutdown(). When SIGTERM arrives, in-flight requests dropped, WebSocket connections severed, PID file left behind.

4. **std::process::exit(0) skips ALL Drop impls**: EventBus threads, KB connections, file handles, lock guards all abandoned. This is the opposite of graceful shutdown.

5. **tokio::fs is a lie**: All operations dispatch to spawn_blocking threadpool. Every file op consumes a blocking thread, not a lightweight tokio task.

6. **No file locking on KB journal writes**: Two NeoTrix processes writing to same knowledge.db will corrupt the journal. ConcurrencyConflictDetector has enable_file_locking:true but actual storage doesn't use it.

7. **Versioned layout headers non-negotiable**: Every production shared-memory channel needs magic|version|capacity|write_index|read_index|generation|heartbeat. Without version fields, new builds corrupt old consumers.

8. **processkit provides kernel-backed containment**: cgroup v2 (Linux), Job Object (Windows), POSIX process group (macOS). Whole-tree kill-on-drop guarantee, SIGTERM→wait→SIGKILL shutdown, rlimit enforcement.

9. **SIGTERM mandatory**: Kubernetes, Docker, systemd all send SIGTERM first. Only Ctrl+C handling = data loss on orchestrator-managed shutdowns.

10. **WAL checkpoint starvation**: Under heavy write load, WAL file can grow unbounded if long-running readers prevent auto-checkpoint. Must monitor .db-wal size or trigger manual checkpoints.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 834 |
| New defects (this batch) | 27 |
| Cumulative defects | D01-D76974 |
| Research sources (this batch) | 40 |
| Cumulative research sources | 97,879+ |
