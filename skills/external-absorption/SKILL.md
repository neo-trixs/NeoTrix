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

#### Sub-Agent Dispatch Contract (吸收研究子代理派发契约)

> **来源**: Cycle 159 三连失败教训 (子代理返回问题而非报告 ×3 → 手动 webfetch 兜底)。
> **根因**: 派发 prompt 未明确"自主执行契约"，子代理默认行为(澄清)压过了任务执行。

派发子代理时必须注入以下契约，缺一不可：

| # | 契约条款 | 强制内容 |
|---|----------|----------|
| **C1 自主执行** | `Do NOT ask clarifying questions. Execute autonomously. If a source is unreachable, mark it BLOCKED and continue to the next one. Zero questions, zero stall.` | 子代理不得返回任何问题/选项菜单，必须产出具名报告 |
| **C2 逐源输出** | `For EVERY source in the list, produce exactly one entry. No omissions.` | 防"部分报告"（仅覆盖可获取的来源） |
| **C3 磁盘持久化** | `Write your full findings to {path} BEFORE returning. Use write tool + verify wc -l.` | 防上下文压缩丢失；结果必须落在磁盘 artifact |
| **C4 固定模式** | 每个条目必须用四字段模式：`Source \| Pattern \| NeoTrix mapping \| Reinforce-or-New + consumer` | 输出必须结构化、可合并 |
| **C5 证据接地** | `Every claim must cite what you actually read (README/source path). No inference without evidence.` | 防幻觉 (R-P10/D16) |
| **C6 摘要返回** | `Final message = ONLY the compact table + pointer to the artifact file.` | 返回消息小、artifact 大 |

**派发前检查清单**：
- [ ] URL 清单已写入 `notes/absorption-{batch}.md` 清单头（不依赖对话记忆）
- [ ] prompt 包含 C1-C6 全部契约文本（逐字）
- [ ] 明确指定 artifact 落盘路径（`{PROJECT}/notes/absorption-{batch}.md`）
- [ ] 声明期望返回：表格 + 文件路径，两者缺一即视为失败

**子代理失败兜底协议**：
1. 若子代理返回空/问题/截断 → 判定失败（不重试超过 1 次）
2. 直接切换到主 agent 手动 `webfetch` 逐源抓取，用同一四字段模式产出
3. 每抓 3-5 个来源立即写入 artifact（防上下文压缩）

#### Research Output Schema (吸收研究输出模式)

```
| Source | Pattern (机制) | NeoTrix 映射节点 | 判定 (强化/新增) | 消费者 (R-P79) |
```

- **强化 (Reinforce)** = 注入已有节点（R-P42 正例）；须给出目标模块名
- **新增 (New)** = 能力树完全不存在；须给出同 session 消费者，否则 R-P79 拒绝
- **Blocked** = 来源不可获取（认证/404），注明原因

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

**子代理产出验证 (必做)**：子代理声称完成后，主 agent 必须
1. `wc -l` + `grep "Source|"` artifact 文件 → 确认条目数 = 来源数（防幻影报告，R-P49/R-P16）
2. 抽查 2-3 个条目引用是否与 README 原文一致（防幻觉，R-P10）
3. 只有验证通过才可将其合并入吸收矩阵；否则视为失败走兜底协议

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
