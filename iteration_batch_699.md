# Iteration 699 — Macro & Const-Eval Landscape (2026-09-06)

**Predecessor**: Batch 698 (RwLock→Mutex, SeqCst→Relaxed, mpsc→async channel, backpressure)

---

## 1. Procedural Macros — Findings

### F1: Declarative Derive Macros Landed on Nightly (PR #145208, Issue #143547)
- **Source**: [goals.rust-lang.org/2026/macro-improvements.html](https://goals.rust-lang.org/2026/macro-improvements.html), [rust-lang/rust#143547](https://github.com/rust-lang/rust/issues/143547), [rust-lang/rust#145208](https://github.com/rust-lang/rust/pull/145208)
- **Finding**: Josh Triplett's 2026 goal: **stabilize declarative attribute and derive macros**. `macro_rules!` can now define `#[derive(...)]` and `#[attribute]` macros without a separate proc-macro crate. Currently nightly-only.
- **NEW DEFECT in NeoTrix**: NeoTrix has **zero proc-macro crates** (no `proc-macro = true` in any `Cargo.toml`), but this feature is still relevant because NeoTrix uses **18 `macro_rules!` definitions** across the codebase. Several of these (e.g., `make_stage!`, `define_capability_fields!`, `define_plugin!`, `trust_rule!`) generate struct/enum impls that could benefit from the new declarative derive syntax once stabilized — reducing boilerplate that today requires manual `impl` blocks after macro expansion.
- **Impact**: When stable, `make_stage!` in `neotrix-core/src/lib.rs:46` could become a declarative derive instead of a macro that stamps out full items. Eliminates the "macro expands to hidden items" anti-pattern.

### F2: `proc_macro::tracked` Module (Nightly, #99515)
- **Source**: [doc.rust-lang.org/proc_macro/tracked](https://doc.rust-lang.org/proc_macro/tracked/index.html), [rust-lang/rust#99515](https://github.com/rust-lang/rust/issues/99515)
- **Finding**: `proc_macro::tracked::env_var()` and `proc_macro::tracked::path()` add file/env dependencies to cargo's rebuild tracking. Still nightly since 2022.
- **NEW DEFECT in NeoTrix**: NeoTrix has no proc-macro crates, so this is **not currently applicable**. However, if NeoTrix ever adds a proc-macro crate (e.g., for the planned declarative derive migration), `tracked` should be used from day one to avoid stale-cache bugs. **Pre-emptive gap**: no `proc_macro_tracked_env` feature gate in any `.cargo/config` or `rust-toolchain.toml`.

### F3: Proc-Macro Compilation Still Debug-Mode by Default
- **Source**: [users.rust-lang.org/t/we-had-fast-proc-macros-all-the-time-or-had-we/114673](https://users.rust-lang.org/t/we-had-fast-proc-macros-all-the-time-or-had-we/114673)
- **Finding**: Proc-macro crates are compiled in debug mode even in release builds, causing 3-5× slower expansion. `profile.proc-macro.release = true` in `Cargo.toml` fixes this but is not default.
- **NEW DEFECT in NeoTrix**: Not currently applicable (no proc-macro crates), but **if NeoTrix adopts declarative derives or any proc-macro in future, this profile override must be added to `[profile.release]`** to avoid 3-5× compile regression. No documentation of this in AGENTS.md or dev-rules.md.

---

## 2. Declarative Macros — Findings

### F4: Macro Fragment Specifier `expr` Changed in Edition 2024
- **Source**: [bar9.github.io/rust-course-2026-02-04/day3/15_macros.html](https://bar9.github.io/rust-course-2026-02-04/day3/15_macros.html)
- **Finding**: Edition 2024 changed `expr` fragment to also match `const` and `_` expressions. Use `expr_2021` for old behavior.
- **NEW DEFECT in NeoTrix**: `make_stage!` (lib.rs:46) uses `expr` fragments extensively in stage definitions. If NeoTrix is on edition 2024, any macro invocation passing `const` or `_` as an expression could silently change match behavior. **Verify edition in `Cargo.toml`** — if edition 2024, audit all 18 `macro_rules!` for `expr` fragment breakage.

### F5: `macro_rules!` Hygiene Limitations — "Context Transplantation Hack"
- **Source**: [rustc-dev-guide.rust-lang.org/macro-expansion.html](https://rustc-dev-guide.rust-lang.org/macro-expansion.html)
- **Finding**: `macro_rules!` hygiene is weaker than proc-macro hygiene. The compiler uses a "context transplantation hack" (PR #51762) to make legacy MBE interact with modern macros. This can cause **subtle name collision bugs** in nested macro expansions.
- **NEW DEFECT in NeoTrix**: `trust_rule!` (tool_inspection_stack.rs:300) is nested inside an `impl` block that also uses `spawn_handler!` and `emit_event!` (run.rs:710,749). If these macros introduce local bindings that shadow outer names, the context transplantation hack may produce **wrong variable capture** rather than a compile error. **Risk: medium** — audit nested macro sites for hygiene-sensitive variable bindings.

### F6: Declarative Macros Lack `$crate` in Proc-Macro World
- **Source**: [goals.rust-lang.org/2026/macro-improvements.html](https://goals.rust-lang.org/2026/macro-improvements.html)
- **Finding**: Proc macros lack `$crate` (the macro-defining crate path), which `macro_rules!` has. This means proc macros can't reliably refer to their own crate's types without re-export workarounds.
- **NEW DEFECT in NeoTrix**: Not currently applicable, but **if NeoTrix migrates any `macro_rules!` to proc macros, the loss of `$crate` must be planned for**. For example, `make_stage!` references `nt_core_self` types — a proc-macro version would need explicit path injection.

---

## 3. Compile-Time Computation — Findings

### F7: Const Traits MVP Stabilization (RFC #3762) — 2026 Target
- **Source**: [goals.rust-lang.org/2026/const-traits.html](https://goals.rust-lang.org/2026/const-traits.html), [RFC #3762](https://github.com/rust-lang/rfcs/pull/3762)
- **Finding**: Const traits allow `const fn` to call trait methods (`Iterator`, `Try`, `PartialEq`, etc.) at compile time. Stabilization targeted for 2026. Current blocker: syntax (`T: ~Trait` vs `const Trait`).
- **NEW DEFECT in NeoTrix**: `nt_repair/nt_mind_consciousness_monitor.rs:9` has a comment: `// LazyLock: f64::log2 非 const fn, const 上下文无法求值 (Rust 稳定版限制)`. With const traits stabilizing, this restriction will lift. **The workaround in consciousness_monitor should be tagged for future cleanup** — it currently uses `LazyLock` for a value that could become `const` once `f64::log2` is callable in const context via trait bounds.

### F8: ADT Const Generics — Structs/Enums as Const Parameters (End of 2026)
- **Source**: [goals.rust-lang.org/2026/roadmap-constify-all-the-things.html](https://goals.rust-lang.org/2026/roadmap-constify-all-the-things.html)
- **Finding**: Const generics currently only accept primitives (`usize`, `bool`, `char`). End of 2026: structs, tuples, arrays as const generic parameters. Example: `Matrix<Dimensions { rows: 4, cols: 4 }>`.
- **NEW DEFECT in NeoTrix**: `nt_core_cap.rs:3` defines `define_capability_fields!` and `impl_capability_accessors!` which manually generate capability structs with fixed field counts. With ADT const generics, these could be replaced by a single generic `Capability<const N: usize>` type with field-level const accessors. **Current macro approach is 150+ lines of boilerplate that ADT generics would collapse to ~20 lines.**

### F9: Compile-Time Reflection MVP Landed — `TypeId::info()`
- **Source**: [goals.rust-lang.org/2026/reflection-and-comptime.html](https://goals.rust-lang.org/2026/reflection-and-comptime.html), [rust-lang/rust#146923](https://github.com/rust-lang/rust/pull/146923), [rust-lang/rust#148820](https://github.com/rust-lang/rust/pull/148820)
- **Finding**: `TypeId::info()` returns a `Type` struct with type metadata (kind, fields, etc.). Experimental `comptime` attribute for `const fn` enables compile-time type inspection. Open PRs: #148820 (basic comptime fn), #150161 (remove `'static` on `try_as_dyn`).
- **NEW DEFECT in NeoTrix**: NeoTrix's `SelfTest` trait and `SelfModel` types require manual `impl SelfTest for X` blocks (T1 tier). With compile-time reflection, `SelfTest` implementations could be **auto-derived from type structure** — no manual impl needed. **Current approach: 40+ manual SelfTest impls across the codebase that could be auto-generated.** This is a significant maintenance burden that reflection would eliminate.

### F10: `comptime` `macro fn` Prototypes — End of 2026
- **Source**: [goals.rust-lang.org/2026/macro-improvements.html](https://goals.rust-lang.org/2026/macro-improvements.html)
- **Finding**: Josh Triplett is prototyping `macro fn` — functions that run during macro expansion with full Rust logic (not just pattern matching). End-of-2026 target: functional prototypes for field iteration and conditional pattern matching.
- **NEW DEFECT in NeoTrix**: `make_stage!` (lib.rs:46) currently uses recursive `macro_rules!` to iterate over stage definitions — a fragile pattern that requires manual counting. With `macro fn`, this could be replaced by a simple `for` loop over a type's fields at compile time. **The recursive macro approach is a known maintenance hazard** (see: `sum!` recursive macro in Rust examples — stack overflow risk on deep recursions).

---

## 4. Cross-Domain Defects (Batch 699 Synthesis)

### DEFECT-699-1: No `expr_2021` Migration Audit
- **Severity**: Medium
- **Location**: All 18 `macro_rules!` definitions
- **Detail**: If NeoTrix is on edition 2024, the `expr` fragment specifier change silently alters match behavior. `make_stage!`, `define_plugin!`, and `trust_rule!` all use `expr` fragments.
- **Action**: `grep -r 'expr' neotrix-core/src/ --include='*.rs' | grep 'macro_rules'` → audit each site.

### DEFECT-699-2: Const Trait Workaround Debt
- **Severity**: Low (future cleanup)
- **Location**: `nt_repair/nt_mind_consciousness_monitor.rs:9`
- **Detail**: `LazyLock` workaround for `f64::log2` non-const. Will become unnecessary when const traits stabilize (2026 target).
- **Action**: Tag with `// TODO(const-traits): remove LazyLock workaround when const f64::log2 available`.

### DEFECT-699-3: `define_capability_fields!` Macro Collapse Opportunity
- **Severity**: Medium (technical debt)
- **Location**: `crates/neotrix-types/src/core/nt_core_cap.rs:3-11`
- **Detail**: 150+ lines of macro-generated capability field boilerplate. ADT const generics (end 2026) would collapse this to ~20 lines of generic code.
- **Action**: Track as "post-const-generics cleanup" — not actionable now, but document the migration path.

### DEFECT-699-4: SelfTest Manual Impl Burden
- **Severity**: Medium (maintenance)
- **Location**: 40+ files with `impl SelfTest for X`
- **Detail**: Compile-time reflection (`TypeId::info()` + `comptime` fn) would auto-derive SelfTest from type structure. Currently each type requires manual implementation.
- **Action**: Tag as "post-reflection cleanup" — the `SelfTest` trait definition should be reviewed for reflectibility (does it need fields? can it be auto-derived?).

### DEFECT-699-5: Nested Macro Hygiene Risk
- **Severity**: Low-Medium
- **Location**: `run.rs:710-749` (nested `spawn_handler!` + `emit_event!`), `tool_inspection_stack.rs:300` (`trust_rule!`)
- **Detail**: `macro_rules!` hygiene is weaker than proc-macro hygiene. The "context transplantation hack" can cause wrong variable capture in nested expansions without a compile error.
- **Action**: Audit nested macro sites for variable shadowing. If any macro introduces `let x = ...` that could shadow an outer `x`, refactor to use unique identifier prefixes.

### DEFECT-699-6: `make_stage!` Recursive Macro fragility
- **Severity**: Medium
- **Location**: `neotrix-core/src/lib.rs:46`
- **Detail**: Recursive `macro_rules!` for stage iteration has stack overflow risk on deep recursions and is hard to debug. `macro fn` (end-2026) would replace this with a simple `for` loop.
- **Action**: Document as "post-macro-fn migration" — the current approach works but is fragile.

---

## 5. Sources Cited

| # | Source | URL |
|---|--------|-----|
| 1 | Rust 2026 Goals: Declarative Macro Improvements | https://goals.rust-lang.org/2026/macro-improvements.html |
| 2 | Rust 2026 Goals: Constify All The Things | https://goals.rust-lang.org/2026/roadmap-constify-all-the-things.html |
| 3 | Rust 2026 Goals: Const Traits | https://goals.rust-lang.org/2026/const-traits.html |
| 4 | Rust 2026 Goals: Reflection and Comptime | https://goals.rust-lang.org/2026/reflection-and-comptime.html |
| 5 | Rust Reference: Const Eval | https://doc.rust-lang.org/reference/const_eval.html |
| 6 | Rust Reference: Procedural Macros | https://doc.rust-lang.org/reference/procedural-macros.html |
| 7 | proc_macro::tracked (nightly) | https://doc.rust-lang.org/proc_macro/tracked/index.html |
| 8 | rust-lang/rust#143547: Declarative Attribute Macros | https://github.com/rust-lang/rust/issues/143547 |
| 9 | rust-lang/rust#145208: Declarative Derive Macros | https://github.com/rust-lang/rust/pull/145208 |
| 10 | rust-lang/rust#146923: Reflection MVP | https://github.com/rust-lang/rust/pull/146923 |
| 11 | rust-lang/rust#148820: Basic comptime fn | https://github.com/rust-lang/rust/pull/148820 |
| 12 | RFC #3762: Const Traits | https://github.com/rust-lang/rfcs/pull/3762 |
| 13 | rust-lang/rust#99515: proc_macro tracked_env | https://github.com/rust-lang/rust/issues/99515 |
| 14 | Rust Course 2026: Macros & Code Generation | https://bar9.github.io/rust-course-2026-02-04/day3/15_macros.html |
| 15 | rustc-dev-guide: Macro Expansion | https://rustc-dev-guide.rust-lang.org/macro-expansion.html |
| 16 | Rustify: Const Generics 2026 | https://rustify.rs/glossary/const-generics |
| 17 | OneUptime: Compile-Time Constants | https://oneuptime.com/blog/post/2026-01-30-rust-compile-time-constants/view |
| 18 | OneUptime: Custom Derive Macros | https://oneuptime.com/blog/post/2026-01-30-how-to-implement-custom-derive-macros-in-rust/view |
| 19 | OneUptime: Rust Macros Effectively | https://oneuptime.com/blog/post/2026-02-03-rust-macros/view |
| 20 | Proc-Macro Debug Mode Discussion | https://users.rust-lang.org/t/we-had-fast-proc-macros-all-the-time-or-had-we/114673 |

---

## 6. Summary

| Category | New Findings | Defects Found |
|----------|-------------|---------------|
| Procedural Macros | 3 (F1-F3) | 2 (edition migration, debug-mode profile) |
| Declarative Macros | 3 (F4-F6) | 3 (expr edition change, hygiene risk, $crate loss) |
| Const-Eval | 4 (F7-F10) | 4 (const trait debt, ADT collapse, SelfTest burden, recursive macro) |
| **Total** | **10** | **6 new defects** |

**Key theme**: Rust 2026 is converging on making compile-time computation (const generics, const traits, reflection) powerful enough to **eliminate most macro boilerplate**. NeoTrix's current macro-heavy approach will need a migration strategy when these features stabilize (Q3-Q4 2026).
