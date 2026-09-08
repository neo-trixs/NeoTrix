# Iteration Batch 583 — Macro System & Code Generation Analysis

**Date**: 2026-09-06  
**Parent**: Batch 582 (property testing/tooling defects)  
**Focus**: Procedural macros, code generation, declarative macro patterns (2026 state-of-the-art)

---

## 1. Procedural Macros — 2026 Findings

### Source: Rust Reference (doc.rust-lang.org), Rust Project Goals 2026, RFC 3698

**New Defect D583-1: Derive Macro AST-Only Blindness (Confirmed + Extended)**
- Batch 582 noted `#[derive(Arbitrary)]` ignores cross-field invariants
- Root cause confirmed: derive macros receive **only token streams** (AST), never type-checked representations
- Derive macro runs **before** Rust performs type checking — it doesn't know if field types implement required traits
- Extension: This is a **fundamental architectural constraint**, not a solvable bug. Any derive macro that needs semantic relationships (field A implies field B) will fail silently
- Impact on NeoTrix: SelfModel derive macros for emotion states, capability trees are structurally blind to cross-field constraints

**New Defect D583-2: Proc Macro Crate Isolation Tax**
- Proc macros **must** be in a separate crate with `proc-macro = true` in Cargo.toml
- This forces every macro into its own crate, creating:
  - Separate version management
  - Dual compilation (proc macro crate + dependent crate)
  - `cargo check` slower because proc macros require full build of the macro crate + deps
  - `syn`/`quote`/`proc-macro2` trinity adds ~3 heavy dependencies to every macro-using project
- Batch 582 didn't quantify this: every custom derive = +1 crate, +3 deps, +dual compile

**New Finding D583-3: Declarative Derive Macros (RFC 3698) — Nightly Only**
- Rust 2026 goal: stabilize `macro_rules!`-based derive macros
- RFC 3698 defines `derive(Trait)` rules in declarative macros
- Would allow: `$crate` access (hygiene), no separate crate, faster compile
- **Not yet stable** — Josh Triplett (owner) proposes stabilization but no timeline
- NeoTrix implication: if stabilized, could eliminate many proc-macro crates, but currently unavailable

**New Finding D583-4: `macro fn` Mechanism — Design Phase**
- 2026 goal includes `macro fn` — a new construct combining `macro_rules!` pattern matching with Rust function syntax
- Would enable: iteration/matching inside macros without TT-munching gymnastics
- Prototype stage only — no RFC yet
- Relevance: could replace complex TT-munching parsers in NeoTrix DSL macros

---

## 2. Code Generation — 2026 Findings

### Source: Cargo Book, Knok (tensor compiler), build.rs patterns, ADHDecode

**New Defect D583-5: build.rs Host-vs-Target Platform Trap**
- `build.rs` runs on **host** machine, compiles for **target**
- When cross-compiling (e.g., x86 host → ARM target), build scripts must handle this manually
- `cc::Build` handles this via `TARGET` env var, but custom build scripts often don't
- Batch 582 didn't address: build scripts that work on native but fail on cross-compile

**New Defect D583-6: build.rs Caching Stale Generated Code**
- Cargo caches build script outputs aggressively via input hashing
- If `cargo::rerun-if-changed` is incomplete, Cargo assumes output never changes
- Generated code becomes **stale** until `cargo clean` — no compiler warning
- "Your binary will just lie" — documented trap in rustfaq.org
- Batch 582 didn't cover: build script caching producing silent incorrect binaries

**New Finding D583-7: Knok — build.rs as Tensor Compiler Frontend**
- Knok (2026-07-30): static-shape tensor graph compiler using `build.rs` as compilation frontend
- Graph defined in Rust → traced during build → lowered to MLIR → compiled by IREE → typed Rust wrapper generated
- Key insight: "data-dependent behavior must use graph operations such as `where` — that boundary is much clearer than trying to teach a macro an increasingly large subset of Rust"
- Shows build.rs can orchestrate complex external toolchains (IREE, MLIR, LLVM)
- NeoTrix: same pattern could work for VSA HyperCube compilation or SEAL pipeline codegen

**New Finding D583-8: capnpc_embedded — WASM-in-build.rs Pattern**
- Cap'n Proto schema compiler embedded as WASM module, executed in-process by pure-Rust WASM engine
- No system `capnp` binary, no C/C++ toolchain, no subprocess
- Compile schemas → `CodeGeneratorRequest` → feed to `capnpc` codegen
- Pattern: embed foreign compiler as WASM inside build.rs for zero-dependency builds

---

## 3. Declarative Macro Patterns — 2026 Findings

### Source: Rust Project Goals, rs4ts.dev, Rustaceans Medium, rustz2h.com

**New Defect D583-9: Fragment Specifier Follow Set Constraints**
- `:expr` can only be followed by `=>`, `,`, `;`, `]`, `)` — **not** by `:ident`
- Forces macros to use separators or `:tt` + manual parsing
- Example: `($e:expr $i:ident)` is **illegal** — must use `($e:expr, $i:ident)` or `:tt`
- Batch 582 didn't identify: follow set restrictions make certain DSL grammars impossible in `macro_rules!`

**New Defect D583-10: TT Munching Recursion Depth Limits**
- TT munching (recursive token consumption) has implicit depth limits
- Deeply nested token trees can hit macro expansion recursion limits
- No compile-time warning before hitting limit — just a hard error
- Batch 582 property testing can't shrink operations because TT munching is non-linear: removing one token tree from the middle breaks all subsequent recursive matches

**New Finding D583-11: Internal Rules (`@` prefix) as State Machines**
- Pattern: `@init`, `@body`, `@validate`, `@emit` prefixed rules create explicit state machines
- Different `@` prefixes = different parsing states
- "When you read `@parse`, `@validate`, and `@emit` rules, you understand the macro has three phases"
- NeoTrix: SEAL pipeline stages could be expressed as declarative macro state machines

**New Finding D583-12: Accumulator Pattern for Macro Results**
- Macros can't assign to variables — must pass "accumulator" through recursive calls
- Pattern: `$($acc:tt)*` collects processed data until base case extracts it
- Enables: building complex outputs from incremental parsing steps
- Directly relevant to macro-based DSLs in NeoTrix (SEAL stage definitions, capability tree declarations)

**New Finding D583-13: `macro_rules!` Stabilization Roadmap (2026)**
- Josh Triplett's plan: declarative attribute macros + declarative derive macros → stable
- Then: macro metavariable expressions, macro fragment fields
- Then: `macro fn` mechanism
- Goal: "vast majority of crates never need proc macros"
- Measurement: handling (1) iterating over struct fields, (2) conditionals based on pattern matching

---

## 4. NEW Defects vs Batch 582

| ID | Defect | Category | Severity |
|----|--------|----------|----------|
| D583-1 | Derive macro AST-only blindness (cross-field invariant blindness) | Architectural | **Critical** |
| D583-2 | Proc macro crate isolation tax (separate crate + dual compile) | Build | High |
| D583-5 | build.rs host-vs-target platform trap | Build | High |
| D583-6 | build.rs caching stale generated code silently | Build | **Critical** |
| D583-9 | Fragment specifier follow set constraints limit DSL grammar | Macro | Medium |
| D583-10 | TT munching recursion depth limits break shrinkability | Property Testing | High |

---

## 5. What's NEW vs Batch 582

| Dimension | Batch 582 | Batch 583 |
|-----------|-----------|-----------|
| `#[derive(Arbitrary)]` | Noted cross-field blindness | Confirmed AST-only root cause, quantified as fundamental architectural constraint |
| Proc macro overhead | Not measured | +1 crate, +3 deps, +dual compile per macro |
| Declarative derives | Not known | RFC 3698 exists, nightly only, stabilization proposed |
| `build.rs` traps | Not covered | Stale caching, host-vs-target, cross-compile failures |
| TT munching | Not analyzed | Recursive depth limits, non-linearity prevents shrinking |
| Macro state machines | Not known | `@`-prefix internal rules as explicit state machines |
| Accumulator patterns | Not covered | Token accumulator through recursive calls |
| WASM-in-build.rs | Not known | capnpc_embedded pattern for zero-dependency compilation |
| Tensor compilation via build.rs | Not known | Knok: Rust functions traced → MLIR → IREE |
| `macro fn` mechanism | Not known | Design phase, could replace TT-munching |

---

## 6. Sources

1. Rust Reference: Procedural Macros — https://doc.rust-lang.org/stable/reference/procedural-macros.html
2. OneUptime: Custom Derive Macros (2026-01-30) — https://oneuptime.com/blog/post/2026-01-30-how-to-implement-custom-derive-macros-in-rust/
3. Medium: ByteSerializable Derive (2026-08-06) — https://medium.com/@dipghoshraj/building-a-custom-derive-macro-in-rust
4. Rust RFC 3698: Declarative Derive Macros — https://rust-lang.github.io/rfcs/3698-declarative-derive-macros.html
5. Rust Project Goals 2026: Macro Improvements — https://rust-lang.github.io/rust-project-goals/2026/macro-improvements.html
6. ADHDecode: build.rs (2026-04-16) — https://adhdecode.com/articles/cargo/cargo-build-scripts-build-rs/
7. Knok: Tensor Graphs as Rust Build Artifacts (2026-07-30) — https://gmmyung.github.io/posts/2026-07-30-knok/
8. capnpc_embedded: WASM Schema Compiler — https://docs.rs/capnpc-embedded/latest/capnpc_embedded/
9. rs4ts.dev: Macro Patterns — https://rs4ts.dev/14-macros/02-macro-patterns/
10. rs4ts.dev: Declarative Macros — https://rs4ts.dev/14-macros/01-declarative-macros/
11. Medium/Rustaceans: DSL in Rust Part 2 (2026-01-10) — https://medium.com/rustaceans/dsl-in-rust-part-2
12. rustz2h.com: macro_rules! Patterns — https://rustz2h.com/chapter_16_advanced_architecture_and_idiomatic_rust/series_03_macros_and_metaprogramming/macro_rules_patterns
13. EliteDev: Rust Macros 8 Patterns (2026-05-17) — https://elitedev.in/rust/rust_macros_8_proven_patterns
14. OneUptime: Compile-Time Constants (2026-01-30) — https://oneuptime.com/blog/post/2026-01-30-rust-compile-time-constants/
15. rs4ts.dev: build.rs — https://rs4ts.dev/12-modules-packages/10-build-scripts/
16. Cargo Book: Build Script Examples — https://dirname.github.io/rust-std-doc/cargo/reference/build-script-examples.html

---

## 7. Recommendations for NeoTrix

1. **Derive Macro Strategy**: Accept AST-blindness as fundamental. For cross-field invariants, use **attribute macros** (which can modify the item) instead of derive macros, or add runtime validation functions post-derive.

2. **Macro Crate Consolidation**: Evaluate if any existing proc-macro crates can be merged. When declarative derives stabilize, migrate simple derives to eliminate proc-macro crate overhead.

3. **build.rs Hardening**: Audit all `cargo::rerun-if-changed` directives for completeness. Add missing file dependencies. Consider `cargo::rerun-if-env-changed` for environment-dependent builds.

4. **TT Munching Guards**: Add compile-time depth counters to recursive macros. Use `@`-prefix state machine pattern for complex parsing instead of deep TT recursion.

5. **Knok Pattern**: Consider `build.rs` as compilation frontend for VSA HyperCube or SEAL pipeline — define graphs in Rust, compile at build time, generate typed wrappers.

6. **Monitor RFC 3698**: Track declarative derive macro stabilization. When stable, evaluate migrating simple derives (Debug, Clone, Serialize-like) to declarative form.
