# 经验沉淀 — 2026-09-28 架构侧吸收轮

> 承接 `LESSONS-2026-09-27-scanner-trust.md`（扫描器告警 ≠ 缺陷）。
> 本文记录本轮**我自己犯的三个错**与**一次判断纠偏**，供后续会话避免重犯。
> 全部数据 2026-09-28 实测。

---

## L1 ⭐ 脏树比干净树「更干净」—— 测量台纪律（R-SCAN-3 新变体）

**现象**：同一份代码，两个测量台差 8。

| 测量台 | 分层违规数 |
|---|---|
| 主树**脏**状态（含他窗 561 个未提交改动） | **94** |
| 隔离车道**干净 HEAD** | **102** |

**真因**：主树那批未提交改动恰好**修好了** 8 处 `l1_action → l2_perception` 违规。
⇒ 脏树因为「藏了修复」而显得更干净。

**为什么危险**：我把 94 写进了 `layer-deps-baseline.txt` 并当成真值。
若不是坚持在车道复测，这个错基线会**永久掩盖那 8 处修复** ——
棘轮会认为「这 8 处一直存在」，而实际主树已经修好。

**规约**：
1. 基线只在**主树**测，且测时主树相对其 HEAD **无未提交 `.rs` 改动**。
2. **隔离车道不得提交基线更新** —— 合并时先合代码，再在主树跑 `--update-baseline`。
3. 门记录要写**测量台**（主树/车道 × 干净/脏），不只是时间戳。

**与既有教训的关系**：`AGENTS.md` 记的「陈旧门记录会让下一个 agent 去修正确代码」是
**同一病的另一面** —— 那次是记录**过期**，这次是记录**测错地方**。两者都会让门失去意义。

---

## L2 机器读的 ledger 不能加注释

我给 `scripts/layer-deps-baseline.txt` 加了 `#` 注释头写警示，
结果 `check-layer-deps.sh:153` 的 `grep -c . "$BASELINE"` 把**注释行计为条目**，
94 立刻变 105，棘轮语义失真。已 revert。

**规约**：门/工具用 `grep -c .` / `wc -l` 消费的文件，**不能有任何非数据行**。
警示写文档，不要写进机器读的文件。

---

## L3 worktree 的独立 index 才是真隔离

**现象**：`git add` 后立刻 `git diff --cached`，暂存区出现 **46 个不是我暂存的 `.rs`**。
他窗在我 add 与 check 之间跑了 add。

**根因**：同仓库共享 `.git/index`。worktree 各有 `index` 文件
（`.git/worktrees/<name>/index`）⇒ **这才是隔离的边界**，不只是文件路径隔离。

**规约**：多窗口并发时，**任何暂存/提交操作必须在各自的 worktree 内做**。
主树的 `git add` 永远会与他窗竞争。

---

## L4 ⭐「同名 ≠ 重复」第 4 次实例 —— 且这次是**差点删掉活代码**

**旧结论**（2026-09-27 路线图 0.2）：
> 「删 `nt_core_gate/nt_tool_registry.rs`（45 行 stub）」

**实测证伪**：`l3_embodiment/nt_shield_enforcer.rs:388-390` 有活消费者：

```rust
ToolRegistry::new().register(ToolSpec::reversible("write_file", "undo_file"))
                   .register(ToolSpec::irreversible("git_force_push"))  // …
```

它承载**写操作可逆性**（`reversible` / `irreversible`），
与 `nt_act/tool_registry.rs` 的**运行期 `ToolStats` 正交** ——
字段集不同 ⇒ 不是重复。删它 = 打断 shield 的写操作单向事实源。

**同批证伪**：`agentic_browse::ToolRegistry`（`Vec<_ToolAction>`，crawl 域自包含）也是正交。

**历史**：「同名 ≠ 同类型」已错过 3 次（`CapabilityRegistry` 4 份 / `SearchResult` 9 份 /
三个决策引擎）。**第 4 次是 JEV —— 方向相反：它是活的**
（`nt_jev` + `nt_crystal_core`，L1 有 6 个消费者）。

**规约**：裁决**只认构造点**（`Type::new(` 的调用点）。
`pub mod` 声明、`pub use` 导出、名字相同 —— **都不算证据**。
完整裁决表见 `docs/architecture/OWNERSHIP.md`。

---

## L5 转述的结论必须复核（本轮两次不吻合）

`handoff-s-audit0927` 等交接里的「剩余项」，本会话实测发现 3 项已被覆盖或推翻
（`nt-lang` 实测 0 依赖确认孤儿 / `nt_core_capability_tree` **已移至 `crates/` 且是活路径** /
分层违规数已从 92 降到 84→94，**取决于测量台**）。

**规约**：交接文档里的结论标 `[转述]`，接手方**先复核再执行**。
本会话的做法是每条都重新 grep 一次 —— 这就是抓到 `nt_tool_registry` 假 stub 的原因。
