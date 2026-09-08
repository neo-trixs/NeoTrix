# Agent 4: Memory Safety Patterns (Batch 865)

## Sources
1. https://oneuptime.com/blog/post/2026-01-30-how-to-use-pin-and-unpin-in-async-rust/view — Pin/Unpin fundamentals, self-referential types, Unpin escape hatch
2. https://microsoft.github.io/RustTraining/async-book/ch04-pin-and-unpin.html — Pin guarantees, three pinning patterns (Box::pin, pin!, Pin::new), Unpin auto trait
3. https://www.application-architect.com/posts/rust-pin-and-unpin-memory-stability/ — PhantomPinned, unsafe pin projection pitfalls, pin-project usage
4. https://www.stanza.dev/courses/rust-unsafe/naked-functions/rust-unsafe-pin-self-referential — PhantomPinned, common pitfalls, manual vs crate pin projection
5. https://docs.rs/pin-project/latest/pin_project/ — pin-project macro: safe projection, #[pin] attribute, structurally pinned fields
6. https://google.github.io/zerocopy/zerocopy/index.html — zerocopy safe transmutation, FromBytes/IntoBytes traits, Project Safe Transmute RFC
7. https://docs.rs/zerocopy/latest/zerocopy — transmute!, transmute_ref!, try_transmute! macros, compile-time alignment/size checks
8. https://microsoft.github.io/RustTraining/rust-patterns-book/ch11-serialization-zero-copy-and-binary-data.html — zerocopy vs bytemuck vs unsafe transmute comparison, repr(C) packed struct alignment UB
9. https://medium.com/@Neha8661/the-phantom-type-pattern-how-rusts-zero-cost-abstractions-can-eliminate-entire-classes-of-runtime-1914558b555b — PhantomData zero-cost typestate, compile-time invariants
10. https://cyberguid.com/advanced-rust-security-patterns-2026/ — unsafe isolation patterns, Send+Sync, Arc/Mutex, security code review
11. https://softwarepatternslexicon.com/rust/idiomatic-rust-patterns/the-borrow-checker-and-lifetime-elision-patterns — lifetime elision, borrow checker patterns
12. https://lwn.net/Articles/994334/ — Project Safe Transmute, zerocopy 14K lines unsafe, bit validity + alignment invariants

## Defects

D-SAFE-001: Unchecked raw pointer cast from Vec<u8> to ProcExeTaskInfo without verifying buffer length >= size_of::<ProcExeTaskInfo>() — if sysctl returns fewer bytes than expected, the cast reads uninitialized/short memory | neotrix-sysctl/src/lib.rs:81 | high | Source: zerocopy/lwn.net (alignment + size invariants)

D-SAFE-002: PeekedStream implements AsyncRead/AsyncWrite with manual Pin projection via self.get_mut() instead of using pin-project crate — fragile pattern that silently breaks if PeekedStream becomes !Unpin (e.g., adding a non-Unpin field) | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:23-53 | medium | Source: pin-project docs, Stanza pin pitfalls

D-SAFE-003: XtlsStream implements AsyncRead/AsyncWrite with manual Pin::new(&mut self.inner) projection without pin-project — if XtlsStream ever gains a !Unpin field or self-referential pattern, the current projection becomes unsound | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/connector/vless/mod.rs:266-336 | medium | Source: pin-project docs, application-architect.com

D-SAFE-004: ObfuscatedStream relies on Unpin bound + manual Pin::new(&mut self.inner) projection instead of pin-project — all three async stream wrappers share the same fragile manual projection pattern, multiplying the risk surface | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/security.rs:352-458 | medium | Source: pin-project docs, Stanza pin pitfalls

D-SAFE-005: neotrix-sysctl crate allows unsafe code (#![allow(unsafe_code)]) but has no size_t validation before raw pointer cast — the ProcExeTaskInfo struct is #[repr(C)] but its layout may differ across macOS versions; no compile-time or runtime size assertion guards the cast | neotrix-sysctl/src/lib.rs:33-81 | high | Source: zerocopy (FromBytes/IntoBytes compile-time checks), lwn.net (alignment invariants)

D-SAFE-006: No zerocopy or bytemuck dependency in any Cargo.toml — the codebase has FFI struct casting (neotrix-sysctl) and binary parsing (mitm.rs HTTP parsing) but does not use safe transmutation crates for compile-time layout verification | (project-wide) | medium | Source: zerocopy docs, rust-patterns ch11

D-SAFE-007: RkyvStorage<K, V> uses PhantomData<(K, V)> for type markers but does not enforce Send + Sync bounds — when K or V are not Send/Sync, the storage becomes non-thread-safe without compile-time warning, and callers may assume thread-safety for KB operations | crates/neotrix-types/src/core/nt_core_rkyv.rs:20-23 | medium | Source: PhantomType pattern (medium.com), cyberguid.com Send+Sync patterns

D-SAFE-008: Backend::probe function returns Pin<Box<dyn Future>> — heap-allocates on every probe call during ordered backend routing; for high-frequency health checks this creates unnecessary allocation pressure where stack pinning or async_trait refinement would suffice | neotrix-core/src/unified/layers/perception/nt_world/nt_world_osint/backend_router.rs:16-19 | low | Source: pin!/Box::pin comparison (oneuptime, Microsoft training)

D-SAFE-009: Inconsistent #![forbid(unsafe_code)] enforcement — 77 modules declare the attribute individually rather than at crate root; neotrix-core/src/lib.rs has it but many submodules redundantly redeclare it, creating maintenance drift risk if a new module forgets | neotrix-core/src/lib.rs:14 + 77 submodules | low | Source: Rust security patterns (cyberguid.com), R-P1 axiom

D-SAFE-010: PeekedStream stores peeked: Vec<u8> and pos: usize as separate fields — if the struct is ever pinned and a future polls while pos is mid-way through peeked, moving the Vec's backing buffer (e.g., via realloc in a push) would invalidate the read cursor; currently safe only because PeekedStream is !Unpin-unaware | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:17-21 | low | Source: Pin self-referential problem (Microsoft training, oneuptime)

## Key Insights

1. **Manual pin projection is the dominant risk**: All three async stream wrappers (PeekedStream, XtlsStream, ObfuscatedStream) use manual `Pin::new(&mut self.inner)` or `self.get_mut().inner` instead of `pin-project`. The `Unpin` bound currently makes this sound, but it's a fragile contract — any field addition that breaks `Unpin` would silently introduce UB without compiler warning. The `pin-project` crate exists precisely for this: it generates projection code that is verified correct at compile time.

2. **neotrix-sysctl FFI is the highest-risk unsafe site**: The raw pointer cast at `lib.rs:81` reads `ProcExeTaskInfo` from a `Vec<u8>` without verifying the buffer is large enough. This is the exact pattern that `zerocopy::FromBytes` and `zerocopy::transmute!` are designed to guard against — they enforce size and alignment at compile time. The crate should either add `zerocopy` as a dependency or add an explicit `assert!(size >= size_of::<ProcExeTaskInfo>())` guard.

3. **PhantomData usage is minimal but under-leveraged**: Only one file (`nt_core_rkyv.rs`) uses `PhantomData`, and it's purely for type-level variance. The codebase does not use PhantomData for typestate patterns (e.g., marking connection states as sealed/unsealed, or marking security clearance levels), which would be a natural fit for NT-SHIELD's security domain.

4. **Zero-copy deserialization is partially adopted**: `RkyvStorage` uses rkyv for zero-copy serialization, but the project lacks `zerocopy` or `bytemuck` dependencies. Given the FFI struct casting in neotrix-sysctl and binary HTTP parsing in the MITM proxy, adopting `zerocopy` for compile-time layout verification would eliminate the most dangerous unsafe patterns.

5. **Pin allocation patterns are unnecessarily heavy**: The `Backend::probe` function uses `Pin<Box<dyn Future>>` which heap-allocates per call. For a health-check router that may be called frequently, stack pinning via `pin!` or restructuring to avoid boxing would reduce allocation pressure. The `ProxyStream` trait correctly bounds `Unpin`, but the Box-pinning pattern in the router is avoidable.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 12 |
| Files examined in codebase | 15+ |
| Unsafe sites found | 24 (across sysctl + test strings) |
| Pin/Unpin usage sites | 49 matches across 8 files |
| PhantomData usage | 1 file (nt_core_rkyv.rs) |
| zerocopy/bytemuck adoption | 0 (not in any Cargo.toml) |
