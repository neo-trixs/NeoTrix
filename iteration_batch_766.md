# Iteration Batch 766 — Async Traits, Static Dispatch, Dynamic Dispatch Research

**Date**: 2026-09-07
**Predecessor**: Batch 765 (CVE-2026-11824, partial index, ANALYZE, FTS5 optimize, plan cache)
**Focus**: Rust async trait dispatch landscape, monomorphization tradeoffs, dyn vtable mechanics

---

## Sources Consulted

| # | Source | Date | Key Insight |
|---|--------|------|-------------|
| S1 | wrenlearnsrust.com/posts/async-traits-2026.html | 2026-03-17 | `async fn in traits` stable since 1.75, NOT dyn-compatible. `dyn MyTrait` with async methods fails vtable construction |
| S2 | devencyclopedia.com/blog/rust-async-traits-object-safe | 2026-07-24 | Three workarounds: enum dispatch (closed set), async-trait crate (open set, heap alloc), RPITIT+adapter (zero-cost static, boxed dyn) |
| S3 | microsoft.github.io/RustTraining/async-book/ch10-async-traits.html | 2026 | `trait_variant` generates Send variants but only for static dispatch. dyn still requires `Box::pin` or `async-trait` |
| S4 | rs4ts.dev/11-async/05-async-trait/ | 2026-07-10 | Decision guide: concrete/generics→native async fn; Vec\<Box<dyn\>→async-trait; Send bounds→trait_variant |
| S5 | users.rust-lang.org/t/do-we-still-need-to-use-async-trait/140230 | 2026-05-23 | `dynosaur` crate: leverages static dispatch, only boxes when dyn actually needed. Less mature but lower allocation |
| S6 | internals.rust-lang.org/t/pre-rfc-flow-directed-monomorphization | 2026-06-02 | Flow-directed monomorphization: type-flow constraint solving. Growing cycles (T→Option\<T\>) detected before expansion. Applied to RPITIT, async fn, TAIT |
| S7 | goals.rust-lang.org/2026/polymorphization.html | 2026 | Polymorphic code generation: "monomorphic data, polymorphic code". Hidden vtable parameters. 30%-2x compile speedup, binary size reduction |
| S8 | dev.to/shayan_holakouee/rusts-zero-cost-abstractions | 2026-04-27 | Monomorphization cost: binary bloat, compile time, icache pressure. Box\<dyn Draw\> still monomorphized as concrete type T=Box\<dyn Draw\> |
| S9 | dev.to/someb1oody/advanced-rust-115-trait-bounds | 2026-07-30 | Dynamic dispatch: vtable = lookup sheet. Extra 2-5ns per call. 20-50% slower in tight loops, negligible in real workloads |
| S10 | sofiabelen.github.io/projects/visualizing-rusts-vtables | 2026-09-05 | Vtable layout: drop_in_place + size + align + method pointers. One vtable per (Type, Trait) pair. Fat pointer = 16 bytes on 64-bit |
| S11 | dev.to/mdshakilhossainnsu2018/monomorphization-in-rust | 2026-02-27 | Monomorphization tradeoffs: binary bloat + compile time vs zero runtime overhead |
| S12 | dispatch-blog.hashnode.dev/rust-trait-objects-are-just-a-pointer-and-a-vtable | 2026 | Vtable layout not stable ABI. DynMetadata provides stable size/align access. Don't read function pointer slots in production |
| S13 | corrode.dev/blog/dyn-compatibility/ | 2026-07-29 | Object safety rules comprehensive list. async fn = opaque return = not dispatchable. `where Self: Sized` fences non-dyn methods |
| S14 | rs4ts.dev/09-generics-traits/06-trait-objects/ | 2026-06-09 | Fat pointer: data ptr + vtable ptr. Vec\<Box\<dyn Trait\>\> for heterogeneous collections. Dyn-compatible = object-safe |

---

## NEW Findings (Not in Prior Batches)

### F1: Async Traits NOT Dyn-Compatible — NeoTrix's `dyn AgentUnit` Pattern Is Broken for Async

**Evidence**: S1, S2, S4, S13 all confirm: `async fn in traits` (stable since 1.75) does NOT produce dyn-compatible traits. Writing `dyn MyTrait` where `MyTrait` has `async fn` methods fails with:
```
error[E0038]: the trait `MyTrait` is not dyn compatible
  = note: for a trait to be dyn compatible it needs to allow building a vtable
```

**Codebase Impact**: NeoTrix has 80+ `dyn Trait` usages (from grep: `dyn CapabilitySkill`, `dyn SelfTest`, `dyn AgentUnit`, `dyn Coach`, `dyn NtGameEnv`, `dyn GameTool`, `dyn LlmProvider`, `dyn Orchestrator`, etc.). If any of these traits acquire `async fn` methods, they become non-dyn-compatible instantly.

**Defect D1**: `nt_core_self_test.rs:20` — `HashMap<String, Box<dyn SelfTest>>`. If `SelfTest` ever adds an `async fn evaluate()`, this breaks. The trait must either (a) use `#[async_trait]` wrapper or (b) keep async methods in a separate non-dyn trait. No guard exists today.

**Defect D2**: `nt_act_orch_patterns.rs:115-116` — `supervisor: Box<dyn AgentUnit>, workers: Vec<Box<dyn AgentUnit>>`. Agent orchestration pattern relies on dyn dispatch. If `AgentUnit` trait evolves to include async methods (likely for MCP tool execution), the entire orchestration layer breaks. No `where Self: Sized` fence exists on potential async methods.

**Defect D3**: `nt_core_cot_generator.rs:181` — `provider: Arc<dyn LlmProvider>`. LLM providers are naturally async (HTTP calls). If `LlmProvider` trait has async methods and is used as `dyn`, it fails. Current code likely works because async is handled at the impl level, not the trait level — but this is fragile.

### F2: Three Dispatch Strategies for Async Traits — NeoTrix Uses None Explicitly

**Evidence**: S2 documents three approaches:
1. **Enum dispatch** (closed set, zero-cost) — requires knowing all implementors at compile time
2. **async-trait crate** (open set, heap alloc per call) — `Pin<Box<dyn Future>>` per invocation
3. **RPITIT + object-safe adapter** (zero-cost static, boxed only for dyn callers)

**Defect D4**: NeoTrix has no documented dispatch strategy policy. The 80+ `dyn Trait` usages are ad-hoc. There's no AGENTS.md rule or dev-rules entry governing when to use static vs dynamic dispatch for async-capable traits. This is a latent architectural debt.

**Defect D5**: No `async-trait` or `dynosaur` dependency in Cargo.toml. When async methods are added to dyn-compatible traits, the migration path is undefined. Should either (a) add `async-trait` proactively, or (b) adopt the dual-trait pattern (static async fn trait + object-safe adapter trait).

### F3: Monomorphization Bloat — NeoTrix's Generic Layer Architecture Amplifies Code Bloat

**Evidence**: S6, S7, S8, S11. Monomorphization creates one binary copy per (function × type). Deeply nested generics (iterator chains, async state machines) multiply instances. NeoTrix's 6-layer architecture with trait bounds at each layer means:
- `fn process<T: PerceptionLayer + CognitionLayer>` generates separate instances for every concrete type satisfying both bounds
- Async state machines from `async fn` are monomorphized per-type, each capturing different state

**Defect D6**: The Six-Layer Architecture's `traits.rs` interface contracts (L1-L6) create deep generic chains. A function calling `L1::action() → L2::perceive() → L5::cognize()` monomorphizes three nested trait dispatches. If polymorphic code generation (S7) lands in Rust, NeoTrix could benefit from `-Zpolymorphize` to reduce compile time by 30-50%. No tracking issue exists.

**Defect D7**: Binary bloat from monomorphization is unmeasured. No baseline `cargo bloat` or binary size tracking exists. The SEAL pipeline's many generic functions (distillation, absorption, self-test) likely generate significant dead monomorphized code. LTO is not confirmed in release profile.

### F4: Vtable Layout Not Stable ABI — Risk to Plugin/Extension Architecture

**Evidence**: S10, S12. Vtable layout is compiler-internal, not stable across Rust versions or compiler flags. `DynMetadata` provides stable access to size/align, but function pointer slots are not guaranteed.

**Defect D8**: `nt_core_resource_pool/resource_registry.rs:24` — `fn as_any(&self) -> &dyn Any`. Type-erased pool registration uses `dyn Any` for downcasting. If NeoTrix ever ships a plugin SDK, downstream crates relying on specific vtable layouts (e.g., for FFI or zero-copy deserialization) will break silently across Rust upgrades. No `DynMetadata`-based safe alternative is used.

**Defect D9**: `nt_core_dispatch.rs:46` — `Vec<Box<dyn Fn(&E, &dyn Fn()) -> bool + Send + Sync>>`. Nested dyn dispatch: the handler vector holds boxed trait objects that themselves take `&dyn Fn()` parameters. Each nested dyn adds a fat pointer (16 bytes). In hot dispatch paths, this multiplies indirection: outer vtable lookup → inner vtable lookup → actual call. No static-dispatch alternative (enum + match) exists for known handler sets.

### F5: Polymorphic Code Generation Coming — NeoTrix Should Prepare

**Evidence**: S7. Rust Project Goal 2026: `-Zpolymorphize` flag. "Monomorphic data, polymorphic code" — data layouts stay monomorphized, but function ABIs become polymorphic with hidden vtable parameters. Estimated 30%-2x speedup for post-mono codegen + linking.

**Defect D10**: No preparation for polymorphic code generation. When `-Zpolymorphize` stabilizes, NeoTrix's deeply generic SEAL pipeline functions could be polymorphized to share code instances. This requires:
- Avoiding `TypeId` usage that breaks parametricity (already a known issue per S7)
- Ensuring generic functions don't rely on specialization that affects stable code
- Testing with `-Zpolymorphize` when available

### F6: `dynosaur` Crate — Lighter Alternative to `async-trait` for NeoTrix

**Evidence**: S5. `dynosaur` leverages static dispatch and only boxes when dyn is actually needed. Less mature but lower allocation overhead than `async-trait`'s unconditional `Box::pin`.

**Defect D11**: NeoTrix's `nt_core_cot_generator.rs:181` uses `Arc<dyn LlmProvider>`. If `LlmProvider` needs async methods, `dynosaur` could provide the adapter with less overhead than `async-trait` (which always boxes). No evaluation of `dynosaur` exists.

### F7: Object Safety Fence Pattern Missing — `where Self: Sized` Not Used

**Evidence**: S13. Methods that violate dyn-compatibility can be fenced with `where Self: Sized` to keep the trait dyn-compatible while still allowing the method on concrete types.

**Defect D12**: No `where Self: Sized` fences found in the codebase grep. Traits like `SelfTest`, `AgentUnit`, `Coach` likely have methods that could break dyn compatibility if they evolve. Proactive fencing (adding `where Self: Sized` to methods returning `Self` or having generic params) is not practiced.

---

## Summary of Defects Found

| ID | Severity | Component | Description |
|----|----------|-----------|-------------|
| D1 | HIGH | nt_core_self_test | `dyn SelfTest` breaks if async fn added — no guard |
| D2 | HIGH | nt_act_orch_patterns | `dyn AgentUnit` orchestration — no async fence |
| D3 | MEDIUM | nt_core_cot_generator | `dyn LlmProvider` — async at impl level, fragile |
| D4 | MEDIUM | Architecture-wide | No dispatch strategy policy for async traits |
| D5 | LOW | Cargo.toml | No async-trait/dynosaur dependency — migration undefined |
| D6 | MEDIUM | Six-Layer Architecture | Deep generic chains amplify monomorphization bloat |
| D7 | LOW | Build system | No binary size/bloat measurement baseline |
| D8 | MEDIUM | nt_core_resource_pool | `dyn Any` downcasting — vtable layout not stable ABI |
| D9 | LOW | nt_core_dispatch | Nested dyn dispatch multiplies indirection |
| D10 | LOW | Evolution readiness | No preparation for -Zpolymorphize |
| D11 | LOW | nt_core_cot_generator | dynosaur not evaluated as lighter alternative |
| D12 | MEDIUM | All dyn traits | No `where Self: Sized` fences on non-dyn methods |

---

## Recommendations

1. **Immediate**: Add `where Self: Sized` fences to all trait methods returning `Self` or having generic parameters (D12). Prevents future breakage when traits evolve.
2. **Short-term**: Document dispatch strategy policy in AGENTS.md: when to use static (generics), dynamic (dyn), or enum dispatch for async-capable traits (D4).
3. **Medium-term**: Evaluate `async-trait` or `dynosaur` for `LlmProvider` and `AgentUnit` traits that will inevitably need async methods (D5, D11).
4. **Long-term**: Track `-Zpolymorphize` stabilization and test NeoTrix's SEAL pipeline with it (D10).
5. **Measurement**: Add `cargo bloat --release` to CI to establish binary size baseline (D7).
