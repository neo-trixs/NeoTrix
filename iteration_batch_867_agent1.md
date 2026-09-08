# Agent 1: Async Trait Patterns (Batch 867)

## Sources
1. https://doc.rust-lang.org/reference/items/traits.html#async-fn-in-traits
2. https://rust-lang.github.io/async-book/07_workarounds/04_async_in_traits.html
3. https://docs.rs/async-trait/latest/async_trait/
4. https://github.com/rust-lang/rust/issues/91611
5. https://rust-lang.github.io/rfcs/3185-trait-variant.html

## Defects

D-TRAIT-001: `async_trait` macro adds dynamic dispatch overhead (`Box<dyn Future>`) on every async method call, preventing inlining and stack-based coroutine optimization for hot-path traits like `LlmProvider` and `CapabilityPlugin` | `nt_core_llm.rs:31` | MEDIUM | Source: async-trait crate docs

D-TRAIT-002: `SelfTest` trait uses synchronous `fn self_test()` but is used in async contexts (`SelfTestRegistry` stored in `tokio::spawn`-ed tasks), forcing `block_on` or manual sync bridges at call sites | `nt_core_self_test.rs:13` | HIGH | Source: Rust async-in-traits RFC 3185

D-TRAIT-003: `CapabilitySkill` trait uses `#[async_trait]` on a trait with `Debug` bound, but `async_trait` desugars to `Pin<Box<dyn Future + Send + '_>>` which requires `Send` on the trait's lifetime — conflicts with `Debug`-only bound when used as `Box<dyn CapabilitySkill>` | `types.rs:141-142` | LOW | Source: async-trait crate constraints

D-TRAIT-004: `ResourcePool` trait uses RPITIT (`impl Future<Output = T> + Send`) for 7 methods, making the trait non-`dyn`-compatible (RPITIT cannot be object-safe) — prevents `Box<dyn ResourcePool>` for the pool supervisor's generic `Arc<P>` pattern | `pool_trait.rs:55-84` | HIGH | Source: Rust trait object safety rules

D-TRAIT-005: `CloudSandboxProvider` returns `BoxStream<'static, String>` from `stream_logs` but has no `Send` bound on the stream's inner future, causing `!Send` propagation in `tokio::spawn` contexts | `provider.rs:33` | HIGH | Source: async-trait Send bounds docs

D-TRAIT-006: `AsyncPanelJudge` and `PanelJudge` are two separate traits (async vs sync) for the same conceptual interface, causing dual-registration overhead in `JudgePanel` and preventing polymorphic async/sync judge mixing | `nt_core_gate/mod.rs:696-711` | MEDIUM | Source: Rust RPITIT design patterns

D-TRAIT-007: `GatewayV2` stores `Vec<Box<dyn AsyncPanelJudge>>` but cannot store `dyn PanelJudge` in the same collection, forcing two parallel vectors and separate iteration in `run_async` aggregation | `nt_core_gate/mod.rs:1046,1274` | MEDIUM | Source: async trait object safety rules

D-TRAIT-008: `CapabilityPlugin` trait's `execute` method uses `&self` but `EnergyCore` uses `&mut self` for `receive_wisdom`/`emit_action` — inconsistent borrow semantics prevent single-trait-object dispatch for combined plugin+energy patterns | `l7_capability/traits.rs:7-28` | LOW | Source: Rust async trait method receiver rules

D-TRAIT-009: 75+ `#[async_trait]` macro usages across the codebase create compile-time overhead (each macro invocation generates a new anonymous trait + future boxing code) — migrating to native `async fn in trait` (Rust 1.75+) would eliminate ~75 box allocations per dispatch | `nt_core_llm.rs:31, traits.rs:6, provider.rs:9` | MEDIUM | Source: Rust 1.75 async-in-traits stabilization

D-TRAIT-010: `CloudSandboxProvider` trait's `validate_ready` has a default implementation returning `Ok(())`, but callers in `mod.rs` call it without checking if the provider actually validates — silent pass-through on untested backends | `provider.rs:41-43` | LOW | Source: async trait default method semantics

## Key Insights
- **RPITIT vs `async_trait` conflict**: `ResourcePool` uses native RPITIT while all other traits use `#[async_trait]` — this inconsistency means `ResourcePool` cannot be used as `dyn ResourcePool` while everything else can.
- **No `Send` enforcement on async trait defaults**: Several traits (`CloudSandboxProvider`, `UpstreamConnector`) return futures without explicit `Send` bounds on the trait definition, relying on the `#[async_trait]` macro to infer it — fragile when the macro is removed.
- **Dual trait pattern is unnecessary**: `PanelJudge` (sync) + `AsyncPanelJudge` (async) could be unified using `async fn in trait` with a sync wrapper, eliminating the dual-registration boilerplate.
- **Migration path clear**: With `async-trait = "0.1"` and Rust 1.75+, all 75+ `#[async_trait]` annotations can be replaced with native `async fn` in trait, removing boxing overhead. The `ResourcePool` trait already demonstrates the native pattern.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 5 |
| Traits audited | 25+ |
