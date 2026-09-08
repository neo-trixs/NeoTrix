# Agent 3: Proc-Macro Patterns (Batch 861)

## Sources
- https://doc.rust-lang.org/reference/procedural-macros.html — Rust proc-macro reference (derive, attribute, function-like)
- https://rust-lang.github.io/rust-crate-guide/ proc-macros.html — Guide to writing proc-macros with syn/quote
- https://docs.rs/syn/latest/syn/ — syn crate docs (AST parsing, Visit trait, parse_file)
- https://docs.rs/quote/latest/quote/ — quote crate docs (code generation via quote! macro)
- https://doc.rust-lang.org/reference/attributes/codegen.html — Derive macro attribute reference
- https://github.com/dtolnay/syn — syn source (proc-macro AST parsing patterns)
- NeoTrix codebase: `neotrix-core/src/unified/layers/cognition/nt_mind/graph_build.rs`, `neotrix-core/src/unified/core/nt_core_self_review/scanners.rs`, `crates/neotrix-types/src/core/nt_core_cap.rs`, `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs`, `neotrix-core/src/unified/layers/action/nt_infra_tracing.rs`, `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield/tool_inspection_stack.rs`

## Defects

**D-MACRO-001: Swapped struct_name/trait_name in impl block AST parsing** | `neotrix-core/src/unified/layers/cognition/nt_mind/graph_build.rs:60-65` | High | NeoTrix graph_build.rs

When parsing `syn::Item::Impl`, the code assigns `struct_name` from the trait path and `trait_name` from `self_ty` — the exact inverse of their intended meaning. For `impl Trait for Struct`, `struct_name` receives the trait path and `trait_name` receives the struct type. All downstream consumers of `ParsedItem::ImplBlock` receive inverted semantics.

**D-MACRO-002: count_lines_in_last_commit returns deletion count when no insertions** | `neotrix-core/src/unified/core/nt_core_self_review/scanners.rs:714` | Medium | NeoTrix scanners.rs

`s.split(',').nth(1)` grabs the second comma-delimited field. When `git diff --shortstat` reports only deletions (e.g., `"1 file changed, 3 deletions(-)"`), `nth(1)` returns the deletion count, which is parsed and returned as "lines added" — a semantic inversion.

**D-MACRO-003: estimate_brace_depth counts braces in string literals and comments** | `neotrix-core/src/unified/core/nt_core_self_review/scanners.rs:724-736` | Medium | NeoTrix scanners.rs

The fallback `estimate_brace_depth` iterates raw `char`s, counting `{` and `}` without excluding string literals, comments, or macro content. Used as fallback when `syn::parse_file` fails (line 290), it over-reports depth for files containing JSON snippets, format strings, or doc comments with braces.

**D-MACRO-004: NUM_FIELDS hardcoded to 23, not derived from field list** | `crates/neotrix-types/src/core/nt_core_cap.rs:6` | High | NeoTrix nt_core_cap.rs

`define_capability_fields!` hardcodes `pub const NUM_FIELDS: usize = 23` inside the macro body rather than computing `usize::count` from the input tokens. Adding or removing a field from the invocation without manually updating the constant causes silent array index out-of-bounds at runtime (`self.arr[IDX_*]`).

**D-MACRO-005: Duplicate field list in define_capability_fields and impl_capability_accessors** | `crates/neotrix-types/src/core/nt_core_cap.rs:20-44 vs 282-306` | High | NeoTrix nt_core_cap.rs

The 23-field list must be kept in sync across two separate macro invocations (`define_capability_fields!` at line 20 for constants/names, `impl_capability_accessors!` at line 282 for getter/setter methods). A mismatch between them — adding a field to one but not the other — causes silent index corruption or missing accessors with no compile error.

**D-MACRO-006: spawn_handler! third arm is dead code** | `neotrix-core/src/unified/layers/cognition/nt_mind/nt_mind_background_loop/run.rs:764-766` | Low | NeoTrix run.rs

The third macro arm `($interval:expr, $lock:ident, $body:expr)` is never invoked — all 40+ call sites use either the 2-arg closure form or the 3-arg literal+closure form. Dead macro arms increase cognitive load and may mislead future maintainers into thinking this syntax is supported.

**D-MACRO-007: parse_rust_file silently returns empty vec on syn parse failure** | `neotrix-core/src/unified/layers/cognition/nt_mind/graph_build.rs:16-18` | Medium | NeoTrix graph_build.rs

When `syn::parse_file` returns `Err`, the function silently returns an empty `Vec<ParsedItem>`. This hides parse failures that could indicate code corruption, syntax errors in generated code, or macro expansion failures. Downstream graph construction proceeds with an incomplete AST, producing a silently incomplete dependency graph.

**D-MACRO-008: nt_infra_tracing uses lazy_static! while rest of codebase uses LazyLock** | `neotrix-core/src/unified/layers/action/nt_infra_tracing.rs:127-129` | Low | NeoTrix nt_infra_tracing.rs

`GLOBAL_COLLECTOR` uses `lazy_static::lazy_static!` while the rest of NeoTrix (60+ statics across 30+ files) has migrated to `std::sync::LazyLock`. This creates an unnecessary external crate dependency inconsistency and prevents potential future `const` initialization optimizations.

**D-MACRO-009: trust_rule! macro panics at runtime on invalid regex** | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield/tool_inspection_stack.rs:300-308` | Medium | NeoTrix tool_inspection_stack.rs

The `trust_rule!` macro uses `regex::Regex::new($pattern).expect("static trust regex")`, which panics at runtime if a regex pattern is invalid. Since these are security-critical trust rules (T01-T12), a typo in a pattern string causes a production panic rather than a compile-time error. The `regex!` crate or a `const fn` wrapper would catch invalid patterns at build time.

**D-MACRO-010: #[allow(dead_code)] on ImplBlock fields masks D-MACRO-001 bug** | `neotrix-core/src/unified/layers/cognition/nt_mind/graph_build.rs:11` | Medium | NeoTrix graph_build.rs

All three fields of `ParsedItem::ImplBlock` are annotated `#[allow(dead_code)]`, which suppresses the compiler warning that `struct_name` and `trait_name` are never read. This annotation directly masks the symptom of D-MACRO-001 — if the fields were read, the swapped values would cause visible incorrect behavior.

## Key Insights

- **No proc-macro crates exist in NeoTrix.** The project uses `macro_rules!` exclusively for code generation (13 definitions) and `syn` for runtime AST analysis (60+ call sites). This is a deliberate architectural choice — proc-macro crates add compile-time complexity and require separate crate packaging.
- **`macro_rules!` is used for 3 distinct patterns:** declarative code generation (`make_stage!`, `impl_evolution_capable!`, `define_capability_fields!`), boilerplate reduction (`spawn_handler!`, `register_if!`, `trust_rule!`), and logging wrappers (`log_error!`, `log_warn!`, `log_info!`, `log_debug!`).
- **The `CapabilityVector` macro system (D-MACRO-004/005) is the highest-risk pattern** — it has two independent macro invocations that must stay synchronized, with a hardcoded constant that must match the field count. A proc-macro derive or a single unified macro would eliminate this synchronization requirement.
- **`include_str!` for compile-time embedding** is used for 3 static assets (frontend.html, openapi.yaml, pgl_domains.txt), which is idiomatic Rust — no issues found with these.
- **The `syn` integration for AST analysis is extensive** (graph_build.rs + scanners.rs = 260+ lines), performing runtime code analysis for self-review and dependency graph construction. This is unusual — most Rust projects use proc-macros for compile-time AST transformation. NeoTrix uses syn at runtime for meta-cognitive self-analysis.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources reviewed | 6 web + 6 codebase files |
| Proc-macro crates in NeoTrix | 0 |
| macro_rules! definitions | 13 |
| syn call sites | 60+ |
| include_str! usages | 3 |
| LazyLock statics | 60+ |
| lazy_static! statics | 1 (inconsistent) |
