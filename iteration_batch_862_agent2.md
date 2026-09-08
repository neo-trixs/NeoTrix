# Agent 2: io_uring Patterns (Batch 862)

## Sources
- https://dev.to/speed_engineer/iouring-adventures-rust-servers-that-love-syscalls-47nm (2026-04)
- https://www.devopsness.com/blog/linux-io-uring-async-io-patterns (2026-06)
- https://michaellaplante.com/blog/2026-08-14-demystifying-io-uring-building-high-performance-secure-network-services-in-rust/ (2026-08)
- https://michaellaplante.com/blog/2026-06-09-unlocking-peak-performance-with-rust-and-io-uring/ (2026-06)
- https://www.youtube.com/watch?v=KGUliLTT_YI (P99 CONF 2025 — Bridging epoll and io_uring)
- https://medium.com/rustaceans/building-a-web-server-on-linux-with-io-uring-and-tokio-10532373d933 (2026-03)
- https://johal.in/internals-rust-185-tokio-140-handle-async-io (2026-05)
- https://github.com/monoio-rs/monoio (ByteDance monoio runtime)
- https://github.com/vx416/play_iouring (2026-03 benchmarks)
- https://iggy.apache.org/blogs/2026/02/27/thread-per-core-io_uring/ (Apache Iggy migration)
- https://dev.to/sospeter/how-apache-iggy-ditched-tokio-for-thread-per-core-iouring-and-what-it-cost-them-3m75 (2026-07)
- https://tonbo.io/blog/async-rust-is-not-safe-with-io-uring (TCP leak via select+io_uring)
- https://github.com/tokio-rs/io-uring/issues/197 (SeqCst race condition)
- https://github.com/foyer-rs/foyer/issues/1286 (use-after-free on buffer drop)
- https://github.com/compio-rs/compio/issues/600 (u32 truncation for 4GB+ buffers)
- https://github.com/nubskr/walrus/issues/25 (panic on io_uring init failure)
- https://www.upwind.io/feed/io_uring-linux-performance-boost-or-security-headache (2025)
- https://without.boats/blog/io-uring/ (canonical cancellation-safety analysis)
- https://nordvarg.com/blog/kernel-bypassing-linux (kernel bypass patterns)
- https://github.com/SSL-ACTX/isla (custom TCP/IP stack in Rust)
- https://www.firezone.dev/blog/sans-io (sans-IO pattern)

## Defects

D-IO-001: **Epoll file I/O bottleneck via spawn_blocking** — `tokio::fs` operations in NeoTrix (sandbox docker.rs:286-296, kb API:369-403, nt_io web tiles:79, provider factory:1292-1295) route all file I/O through Tokio's blocking thread pool (512 threads max). Under KB-heavy workloads (crawl→parse→embed→store), the blocking pool saturates causing cascading latency. io_uring's completion-based model eliminates this entirely by making file reads first-class async. Apache Iggy reported exactly this issue as their "real dealbreaker" — Tokio's epoll treats files as always-ready, so the subsequent read blocks the thread on page-cache contention. | neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_api.rs:369 | high | Apache Iggy migration blog + dev.to uring adventures

D-IO-002: **tokio::net TcpStream mutex contention in NT-SHIELD proxy chain** — NT-SHIELD's network pool (`network_pool.rs:267-328`), proxy chain (`proxy_chain/chain.rs:336`), and Tor client (`tor_client.rs:148-150`) all create individual `tokio::net::TcpStream::connect` futures. Under high proxy churn (proxy pool refresh, health checks, Tor circuit rotation), these compete for the same epoll reactor and scheduler, creating lock contention on the reactor's interest set. With io_uring's multishot accept and registered FDs, each connection is zero-overhead after initial registration. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_stealth_net/network_pool.rs:267 | medium | devopsness uring patterns + iggy migration

D-IO-003: **Platform portability hard-coupled to epoll — no io_uring fallback strategy** — NeoTrix's Cargo.toml uses `tokio = { version = "1", features = ["full"] }` with no io_uring feature gates. The project targets macOS (kqueue) and Linux (epoll), but there's no compile-time runtime selection mechanism like Fusio's `MaybeOwned`/`MaybeSend` marker traits that flip buffer ownership rules per-platform. If io_uring is ever adopted, the poll-based buffer borrow model (`&mut [u8]` slices in async fn) will fail to compile under completion-based runtimes that require owned buffers or `'static` lifetime. | neotrix-core/Cargo.toml:33 | medium | P99 CONF fusio talk + medium tokio-uring article

D-IO-004: **Cancellation-safety hazard in select! with io_uring-bound futures** — NeoTrix uses `tokio::select!` throughout NT-SHIELD (proxy control, DNS intercept, network diagnostics) and NT-IO (web server, provider factory). If io_uring is adopted, the `select!` macro's implicit future-drop-on-branch-completion triggers a critical bug: the kernel has already committed to writing into the buffer via the SQE, but the future is dropped, causing the buffer to be freed while the kernel still holds a raw pointer. The `tonbo.io` analysis proves this causes TCP connection leaks across ALL io_uring runtimes (monoio, tokio-uring, compio). NeoTrix's pervasive `select!` usage makes this a high-probability defect surface. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/dns_intercept.rs:10 | critical | tonbo.io async-rust-not-safe-with-io-uring + without.boats/io-uring

D-IO-005: **No SQPOLL memory barrier awareness — potential SQ stall under high submission rate** — The `tokio-rs/io-uring` crate had a confirmed race condition (issue #197) where the SQPOLL wakeup flag check lacked a `SeqCst` fence, causing the kernel's polling thread to go to sleep while new SQEs were in flight. NeoTrix's KB ingestion pipeline (crawl→parse→embed→store) could hit this under high throughput: submissions arrive in bursts, the kernel thread sleeps, and the application never wakes it, causing permanent stalls until a timeout. The fix requires `SeqCst` fence before `sq_need_wakeup` — this is a kernel-level invariant that must be audited in any io_uring crate used. | neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_api.rs (future io_uring integration) | high | github.com/tokio-rs/io-uring/issues/197

D-IO-006: **Buffer lifetime violation risk in crawl pipeline under io_uring** — NT-WORLD's UnifiedCrawler performs sequential fetch→parse→classify→store operations where parsed content buffers are held across await points. Under io_uring, if a buffer is passed to the kernel for a read operation and then dropped/reallocated before the CQE arrives, the kernel writes into freed memory (use-after-free). The foyer-rs project (issue #1286) demonstrates this is flaky but reproducible: dropping an io_uring read handle immediately triggers malloc corruption. NeoTrix's crawl pipeline does exactly this pattern: `fetch URL → parse into buffer → store to KB`. | neotrix-core/src/unified/layers/perception/nt_world/ (crawl pipeline) | high | github.com/foyer-rs/foyer/issues/1286 + dev.to uring adventures

D-IO-007: **CQ overflow silent data loss without NODROP assertion** — io_uring's Completion Queue can overflow if in-flight operations exceed CQ capacity. The rustfs team (commit 52a99ed) found that without asserting `IORING_FEAT_NODROP` at ring creation, CQEs are silently dropped, causing pending entries to never be reclaimed and shutdown to hang. NeoTrix's crawl→embed pipeline could exceed CQ capacity during burst ingestion (multiple concurrent fetch+parse+store operations). Without NODROP + overflow monitoring, completions vanish silently, leaving futures permanently Pending. | neotrix-core/src/unified/layers/perception/nt_world/ (future io_uring integration) | high | github.com/rustfs/uring commit 52a99ed

D-IO-008: **u32 silent truncation for large buffer operations** — compio issue #600 revealed that io_uring SQE buffer length is cast `usize as u32` without bounds checking. A 4GB buffer becomes length 0, causing silent 0-byte I/O interpreted as EOF. NeoTrix's KB embedding pipeline processes large documents (crawl content, experience entries, media metadata) that could exceed 4GB when batch-processed. The truncation is silent — no error, just missing data. | neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/ (future io_uring integration) | medium | github.com/compio-rs/compio/issues/600

D-IO-009: **Kernel version hard-dependency blocks containerized deployment** — io_uring requires Linux 5.10+ for basic operations, 5.15+ for multishot accept, 6.0+ for ring messaging. NeoTrix's NT-SHIELD sandbox runs in Docker, where the default seccomp profile blocks `io_uring_setup` syscall (confirmed by walrus-rs issue #25 and Upwind security analysis). Google reported 60% of kernel exploits in 2022 targeted io_uring, leading them to disable it by default. NeoTrix cannot assume io_uring availability in container environments — any io_uring integration must have a graceful epoll fallback, not a panic. | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_sandbox/ | medium | github.com/nubskr/walrus/issues/25 + upwind.io security analysis

D-IO-010: **Compio boxes every I/O request — heap allocation per SQE** — Apache Iggy's migration analysis found that compio (the most actively maintained io_uring runtime) boxes every I/O request submitted to the submission queue, incurring a heap allocation per operation. For NeoTrix's high-frequency KB operations (thousands of embed+store cycles per session), this adds allocator pressure. monoio avoids this with a Slab allocator, but monoio's io_uring feature coverage lags behind compio's. The choice between runtimes involves a fundamental tradeoff: compio's feature completeness vs. monoio's allocation efficiency. | neotrix-core/src/unified/layers/action/nt_memory/ (future io_uring integration) | medium | iggy.apache.org thread-per-core blog + github compio

## Key Insights

1. **The poll→completion paradigm shift is non-trivial**: NeoTrix's entire async surface uses poll-based Tokio with `&mut [u8]` borrowed buffers. io_uring requires owned buffers (ownership transfer to kernel, returned on CQE). This is a compile-time breaking change, not a drop-in optimization. The Fusio project's `MaybeOwned`/`MaybeSend` marker traits provide the cleanest abstraction for runtime-agnostic I/O code.

2. **File I/O is the highest-value io_uring target**: NeoTrix's KB pipeline (crawl→parse→embed→store) is dominated by file reads/writes that currently go through Tokio's 512-thread blocking pool. io_uring makes file reads first-class async — the Iggy migration showed 18% throughput improvement and 46% P95 latency reduction just from eliminating the thread pool hop.

3. **NT-SHIELD's proxy network is already well-served by epoll**: For pure TCP socket operations, epoll is competitive with io_uring (benchmarks show roughly equal performance up to 1000 connections). The io_uring advantage only appears at very high connection counts (10K+) with multishot accept. NT-SHIELD should NOT be the first io_uring adoption target.

4. **The runtime landscape is fragmented and unstable**: monoio (ByteDance) is production-proven but feature-limited and maintenance is slowing. compio is most actively maintained but boxes every I/O. tokio-uring adds overhead from layering on Tokio. glommio (closest to Seastar) is effectively unmaintained. Any io_uring adoption must be runtime-abstracted to avoid lock-in.

5. **Cancellation safety is a systemic risk, not a per-bug issue**: io_uring's cancellation model is fundamentally incompatible with Rust's implicit future-drop-on-cancel semantics. This is not fixable at the application level — it requires either: (a) runtime-level cancellable I/O (monoio's approach), (b) compile-time owned buffers (Fusio's approach), or (c) halting the entire task until the kernel completes (unacceptable). NeoTrix's pervasive `select!` usage makes this the #1 adoption blocker.

6. **Security surface expansion**: io_uring bypasses traditional syscall monitoring (EDR/strace/seccomp). Google disabled it by default due to 60% of kernel exploits targeting it. NeoTrix's NT-SHIELD sandbox must account for this: container seccomp profiles block `io_uring_setup`, and security tooling won't see io_uring-based network activity.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources analyzed | 21 |
| Critical severity | 1 (D-IO-004) |
| High severity | 4 (D-IO-001, D-IO-005, D-IO-006, D-IO-007) |
| Medium severity | 5 (D-IO-002, D-IO-003, D-IO-008, D-IO-009, D-IO-010) |
