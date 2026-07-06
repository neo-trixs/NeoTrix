# External Repository Feature Absorption Workflow

## Description
Analyze external repositories, compare against NeoTrix, identify feature gaps, assess architecture fit, and iteratively absorb features.

## Skill Type
workflow

## Tags
- absorption
- architecture
- gap-analysis
- repo-analysis
- iterative-feature

## Trigger Phrases
- "分析这个仓库" / "analyze this repo"
- "对比当前项目" / "compare with current project"
- "吸收XX的功能" / "absorb features from"
- "哪些有而本项目没有" / "what features do they have that we don't"
- "融合到当前架构" / "integrate into current architecture"
- "继续后续迭代" / "continue with remaining phases" / "继续后续迭代对应的模块任务"
  → 自动加载 TODO.md，进入下一个未完成的 Phase

## Workflow

### Phase 0: Project Baseline
1. Read `AGENTS.md`, `TODO.md`, `Cargo.toml` for current state
2. Run `cargo check --lib 2>&1 | grep "^error" -A 3` to establish compilation baseline
3. Identify: what is NeoTrix? core abstractions? current completion %?

### Phase 1: External Repository Analysis
For each external repo:
1. `webfetch` README + docs for high-level understanding
2. Clone with `git clone --depth=1`
3. Use subagent (explore) to analyze: module structure, core abstractions, code size
4. Extract: what problem does it solve? key mechanisms? design philosophy?

### Phase 2: Gap Identification
1. For each feature found in external repos, search NeoTrix source to confirm presence/absence:
   ```bash
   grep -r "feature_name" src/ --include="*.rs" | head -5
   ```
2. Classify each gap:
   - ✅ Present: has equivalent
   - ❌ Absent: no equivalent
3. Build a feature matrix: Source | Feature | Present? | Location

### Phase 3: Architecture Fit Assessment
Evaluate each ❌ feature for integration feasibility:

| Tier | Criteria | Action |
|------|----------|--------|
| 🟢 Tier 1 | Existing type/struct — just add enum variant or field | +10-30 lines |
| 🔵 Tier 2 | Existing function/impl block — add new method | +30-100 lines |
| 🟡 Tier 3 | Replace monolith with pipeline pattern | 1-2 day refactor |
| 🟠 Tier 4 | New module, but architecture compatible | New file |
| 🔴 Tier 5 | Requires architecture adjustment | Assess ROI first |
| ⛔ Blocked | Architecture conflict | Document for later |

### Phase 4: Iterative Absorption (the Loop)

```
Each iteration:
  ┌──────────────────────────────────────────┐
  │ 1. Write code                          │
  │ 2. cargo check --lib (0 errors gate)   │
  │ 3. cargo test --lib <module>           │
  │ 4. If fail → fix, goto 2               │
  │ 5. Update TODO.md                      │
  │ 6. User presents next task             │
  └──────────────────────────────────────────┘
```

Per-phase discipline:
- **Phase N**: 1 feature per phase (or 1 cohesive group)
- **Pre-existing errors** not caused by changes: note but proceed
- **Borrow checker issues**: use `take/put_back` for self-borrows, `Send + Sync` bounds for trait objects
- **Test gap**: always write tests for new modules (min 3-5 tests per new file)

### Phase 5: Review & Document
1. `cargo check --lib` — must be 0 errors
2. Run all affected module tests
3. Summarize: which features absorbed, from which source, file locations
4. Update AGENTS.md lookup chains if adding new core abstractions
5. Update TODO.md: mark absorbed features as [[completed]]

## Success Criteria
- `cargo check --lib`: 0 errors (ignoring pre-existing errors in unrelated modules)
- All new module tests pass
- Feature gap count reduced (track in TODO.md)
- No regression in existing test suites

## Common Patterns

### Self-borrow resolution
When pipeline stages need `&mut self` and pipeline is a field of self:
```rust
let mut pipeline = std::mem::take(&mut self.pipeline);
let result = pipeline.execute(self);
self.pipeline = pipeline;
```

### Send + Sync for trait objects
When adding traits to structs used in async contexts:
```rust
pub trait MyTrait: Send + Sync { }
```

### Module registration
New file → declare in parent's `mod.rs` → add `pub use` re-exports → `cargo check`
