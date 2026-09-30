# Handoff — 第二棵树 B 方案收官（2026-09-30）

## 1. 会话标识

- 窗口：s000
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-30
- 本会话末次提交：`6ad3a493`

## 2. 目标（一句话）

> 消除 `neotrix-core/src/neotrix/` 第二棵树：把 8 个模块物理回流到 L0–L6，并让门能看见此前逃逸的跨层违规。

## 3. 已完成

- [x] **A 方案**：8 模块全部登记 `.neototrix/layer-map.json`；`check-layer-deps.sh`
      支持目录 + 单文件树；修 `rg -n` 单文件输出导致 baseline 写成行号的 bug。
- [x] **B 方案 8/8 物理迁移**（本会话最后两笔）：
      - `nt_crystal_core`（52 文件）→ `l5_cognition/`（记账 7 条）
      - `nt_core_event_bus`（666 行）→ `l0_substrate/`（记账 4 条）
- [x] 目录清空：`neotrix-core/src/neotrix/` **130 文件 / 44,908 行 → 1 个 `mod.rs`（40 行纯 re-export 面）**。
- [x] 顺带清掉两批死代码（**用编译判据，不靠推理**）：
      - `error_conversions.rs`（32 行，4 个 `From` impl）：摘除声明后
        `--lib` 与 `--tests` **均 0 error** ⇒ 无任何 `?` 触发。
        ⚠️ 表面看极危险（`CapabilityError` 出现在 35 个文件）。
      - `nt_core_capability_tree` 的 10 个 re-export：`rg` 全仓零消费，
        且该 crate 不在 workspace members。
- [x] 修既存缺陷 `PlattParams::apply` 恒等参数 1 ULP 漂移 ⇒
      `cargo test -p neotrix --lib` **12,209 / 0 failed（首次全绿）**。
- [x] 3 个棘轮基线随迁改写（`unwrap` 46 / `truth-surface` 10 / `doc-drift` 1）。
- [x] `CODE-TOPOLOGY.md` 从干净检出重生成 + 生成器加「已清空」分支。

## 4. 正在改的文件（关键！逐个列）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| （无）本会话结束时**无**在改文件 | — | — |

工作区现存未提交改动**全部属他窗**，见 §8.2。

## 5. 下一步（按优先级排序）

1. **`.worktrees/merge-b` 的 4,673 MB target 需人工裁决**（见 §6 阻塞点）。
   我**故意没删**：该 worktree 脏且 1h45m 前有他窗 git 操作，
   删 target 会让他窗重编。决定权在你。
   `sh scripts/ops/nt_worktree_gate.sh clean` 可零风险回收。
2. **212 个真死文件 / 50,454 行**（`docs/architecture/ORPHAN-CODE-AUDIT-2026-09-30.md`）
   尚未处置。其中 81 个是伪「docs cleanup」提交 `477bf669` 脱离模块树的。
3. **重复类型**：352 组候选 / 394 可归并，但判据工具已证明不可靠
   （`scripts/dup-types-baseline.txt` 挂着否决横幅）。
   已完成 `GoalPriority` 3→1、`ConvergenceProof` 2→1。**不要批量盲合。**
4. **IM `/stop` 真实 bug**（`docs/plans/2026-09-29-im-stop-worker-pool-plan.md`）：
   outbox 记录无 `channel` 致队列堵塞；`deliver_result` 死代码；编辑目标 ID 错。
   分支 `feat/im-stop` 在 `.worktrees/nt-stop`（干净，未动）。

## 6. 阻塞点

- **`.worktrees/merge-b`：脏 + 疑似他窗在用，我不动。**
  - 脏文件：`Cargo.lock`(M)、`docs/architecture/CODE-TOPOLOGY.md`(M)、
    `neotrix-core/src/neotrix/proxy_daemon_wrapper.rs`(**已暂存删除**)
  - `neotrix-core/tests/*.rs` 5 个文件 mtime = 13:08（交接时 14:92 前 1h45m），
    与 `Cargo.lock`(13:09) 同批 ⇒ **他窗 1h45m 前在此做过 git 操作**。
  - 门已给出正确警告：「可能他窗在用，勿删」。
  - ⛔ 这 2 个 worktree 的未提交改动**不在任何提交里**
    （`/private/tmp/nt-v9` 也脏，3 个文件，非我创建）。
- `check-unwrap --strict` rc=1：唯一 new 违规 `apps/neobot-desktop/src/main.rs:53`（**他窗**）。
- `check-untracked-assets` rc=1：全部是 `apps/neobot-desktop/frontend/src/**`（**他窗**）。
  ⛔ 两处均按既定决策**不代改、不记账**。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再 `git status --short` 核对；预期看到大量 `apps/` 改动（他窗）。
- 禁止事项：
  - ⛔ 不要 `git add -A` / `git add .`（本会话曾误提交他窗删除的 5 个 `apps/` 图标）。
  - ⛔ 不要用 `git mv`（自动暂存，可能被他窗裸 `git commit` 卷走）。用 `mv` + 显式暂存删除侧。
  - ⛔ 不要动 `.worktrees/merge-b`、`/private/tmp/nt-v9`。
- 风险提示：
  - `neotrix-core/src/l1_action/nt_media/`、`crates/neotrix-neobot/`、`apps/` 均为他窗在途。
  - **`--lib` 通过 ≠ 全部通过**：本会话前 6 轮迁移都用 `rg … neotrix-core/src/ crates/` 定位引用，
    **漏掉 `examples/` 与 `benches/`**，是 `cargo check --all-targets` 抓出
    `examples/pdf_enhance_test.rs:6` 的。**搬迁后固定跑 `--all-targets`。**

## 8. 收工自查（2026-09-28 起必填）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
  路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
  /private/tmp/nt-v9            | a005db44 | HEAD       | 3 |    51M |    0M | no
  .worktrees/merge-b            | 1a48ecd3 | HEAD       | 3 |  4739M | 4673M | YES
  .worktrees/nt-stop            | b9be70d9 | feat/im-stop | 0 |  50M |    0M | no
  worktree=3 个 | 合计 4840M | target 占 4673M
  带未提交改动: 2 个 | 近3h有改动: 1 个
```

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | 本会话早期 B 方案合并台 | **仍活跃/疑似他窗 ⇒ 不动**，见 §6 |
| `.worktrees/nt-stop` | IM `/stop` 隔离开发 | **保留**（干净，分支 `feat/im-stop` 尚有待做） |
| `/private/tmp/nt-v9` | 非本会话创建 | **不动**（他窗） |
| `.worktrees/topo2`、`.worktrees/t3` | 本会话临时重生成拓扑 | ✅ 已 `git worktree remove --force` 收掉 |
| `/tmp/nt-pre` | 实证「既存缺陷」用 | ✅ 已 `git worktree remove --force` 收掉 |

⛔ 全部走 `git worktree remove`，**无一例手删目录**。

### 8.2 未提交改动的去向

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `sessions/handoff-secondtree-reflow-20260930.md` | 本会话早期交接 | ☑ `git add` 已提交（本笔） |
| `sessions/handoff-20260930-second-tree-final.md` | 本交接 | ☑ `git add` 已提交（本笔） |
| `apps/neobot-desktop/frontend/**` | 大量增删改 | ⛔ **他窗**，非我碰，不代管 |
| `.neotrix/capability_registry.json` | — | ⛔ 他窗 |
| `Cargo.lock` | — | ⛔ 他窗 |
| `neotrix-core/tests/*.disabled`、`*.rs` | — | ⛔ 他窗 |
| `.github/workflows/security-scan.yaml` | 未跟踪 | ⛔ 他窗 |
| `.neotrix/knowledge.db`、`capability_registry.json.bak-legacy` | 未跟踪 | ⛔ 他窗 |
| `.worktrees/merge-b` 的 3 个脏文件 | — | ⛔ 非本工作树，**原地保留未删** |

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：`0`（check 模式恒 0；警告见 §6）
- `cargo check -p neotrix --lib` / `--tests` / `--all-targets`：**均 0 error**
- `cargo test -p neotrix --lib`：**12,209 passed / 0 failed / 41 ignored**
- 门：`layer-deps --strict` rc=0（0 new / 19 known）· `layout --strict` rc=0 ·
  `truth-surface` rc=0 · `doc-drift` rc=0（0 死链）· `naming` rc=0
- 门红两处，**均为他窗 WIP，非本会话引入**：`check-unwrap --strict`（`apps/…/main.rs:53`）、
  `check-untracked-assets`（`apps/…/frontend/src/**`）。
- 提交前跑过 `cargo xl` 等价物（`cargo test -p neotrix --lib`）。

## 9. 本会话沉淀的方法论（3 条，都是我自己踩出来的）

1. **目录路径会决定门能不能看见违规。**
   搬迁前违规写作 `crate::neotrix::…`（**不含层名字面量**），层的正则匹配不到；
   搬成 `crate::l5_cognition::…` 后 7 条立刻显形。
   ⇒ 这不是「门坏了」，是**违规的表达方式恰好躲过了匹配式**。
   同机制第二面：**棘轮基线必须随搬迁同步改写**，否则搬迁等于给新违规发通行证
   （`unwrap-baseline` 的 46 条曾整片逃逸，且无任何门报告）。

2. **「PASS」必须写清是哪个 flag 下的。**
   我跑 `check-unwrap.sh`（**漏 `--strict`**）判「rc=0 PASS」，
   又用同一错误方式做证伪测试，得出「门对基线改动无反应 = 棘轮是假的」这个**错误结论**。
   重测后：故意改坏 1 条 → `--strict` FAIL，棘轮真在封。
   ⇒ 门记录纪律（R-SCAN-3）同样适用于**我临时跑的验证命令**。

3. **「既存缺陷」要实跑证明，不能靠单文件 diff。**
   缺陷只在别处被调用时，单文件逐字节等价**不足以**证。
   我先做内容等价 diff（不够），又在 `d7353404^` 的干净 worktree 实跑同一测试，
   数值完全相同 ⇒ 才敢下「既存、非我引入」的结论。
   副产品：**发现全套 12,249 个测试长期是红的，此前无人记录。**
