# 交接：孤儿治理收官 + 三道门修复（2026-10-05）

## 1. 本窗口做了什么（一句话）

把「mod-tree 孤儿」这一类**对所有门都隐形**的缺陷从「取证」推进到「清零」，
途中修掉 3 个**门/测试自身**的缺陷 —— 它们比孤儿本身更值钱，因为它们会主动误导下一个 agent。

## 2. 起点与终点

| 指标 | 起 | 终 |
|---|---|---|
| 保守口径孤儿（`mod_orphan::scan_tree`） | 35 | **2** |
| 宽松口径孤儿（`scan_orphan_files`） | 99 | 82 |
| 接入的从未编译代码 | — | **8,032 行** |
| 上线的从未运行过的测试 | — | **82 个** |
| 删除（全部有内容等价证明） | — | 9 文件 / 3,116 行 |

## 3. 我修的「门/测试自身」缺陷（本轮最值钱的产出）

### 3.1 `test_no_orphans_in_core` 从未执行过（`aeab45a5`）

查 `src/core` —— **该目录不存在** ⇒ `if src.exists()` 恒 false ⇒ 断言体空跑，
而「core 里没有孤儿」这个保证**从来没被测过**，长期显示为绿。
修法：指向真实源码根 + `src` 不存在时**硬失败**（含打印 cwd），
「测试没跑」从此不可能再伪装成「通过」。

### 3.2 删除声明门挂在 pre-commit 上从未生效（`ae0e8eae` / `c2563e65`）

实测 4 种提交方式：**pre-commit 时刻 `COMMIT_EDITMSG` 装的是上一次的消息**。

- 本次声明了也判不出 ⇒ **稳定误报红**（我自己被拒两次）
- 上次声明过、本次没声明 ⇒ **漏放行**
- 且 `--no-verify` 能整体跳过 pre-commit ⇒ **真事故反而过得去**（实测 rc=0 落账）

⇒ 搬到 `prepare-commit-msg`（`$1` 即本次消息；**实测不受 `--no-verify` 影响**）。

### 3.3 push 级删除声明门位于 `exit 0` 之后（`8415c2a2`）

死锁链：门不可达 ⇒ 期间他窗提交了 7 个含删除的 commit ⇒ 门一激活就报 7 个未声明
⇒ push 被封死 ⇒ 于是继续不激活。

⛔ 原设计两条出路都不可接受：`rebase` 补声明 = **重写他人共享历史**；继续不激活 = 门不存在。

✅ **破解：基线可前移**（`.neotrix/push_deletions_base`），已裁定历史不再追溯。
双向实测：无声明 `rc=1` / 有声明 `rc=0`。

### 3.4 `kb_search` 一个真 flaky（`171dfb04`）

三个 gate 共用一个计数器但**只有一个递增**，而 fail-open 测试断言该计数器增长
⇒ 能否通过取决于**另一个测试有没有先跑**（线程调度）。
实测全量 6 次红过 1 次，失败信息**谎称「接线已回退」**。
⇒ 拆成两个独立计数器。

### 3.5 检测器自身的假阳性（`1ea8a849`）—— 本轮最需要记住的一条

我按孤儿清单给 `mod.rs` 加 `pub mod handlers_game;` ⇒ 编译器报
`multiple handle_game_training found`，而**两个候选指向同一处**
⇒ 那文件本来就在编译树里，靠 `run.rs` 的 `#[path]` 引入，
而我的判据**只读当前目录 `mod.rs`**。

⇒ **30 个文件被误报为孤儿**，其中 7 个 `handlers_*` 全都在正常跑。

⚠️ 若我没做「实际接入」这一步，就会照着错误清单去「修」30 个正确代码。

## 4. ⭐ 本轮的方法论（三次实测推翻我自己的分类）

我给这些文件贴过「③独立能力」「④分叉有独有」「需重写接入」标签，
**三次都被实测推翻**：

| 我曾判为 | 实测真相 | 定案动作 |
|---|---|---|
| 5 个 `nt_mind/*.rs` 是「独立能力/分叉」 | `E0255` 重复定义 + `E0432` import 断裂 ⇒ **原位置无法编译** | 临时接入 |
| 2 个 `compat` 桥「方向存疑」 | **零消费者、纯 `pub use`** ⇒ 空壳 | grep 全仓 |
| `handlers_crystal` 「需重写接入」 | 依赖的字段**根本不存在** + 宏作用域受限 ⇒ 两处结构断裂 | 临时接入 |

⇒ **分类标签的价值在于可被推翻。定案动作永远是「临时接入，让编译器回答」** ——
成本一次 `cargo check`，而它推翻的判断已让我三次差点把 bug 修进正确代码。

## 5. ⭐ 检测器盲区（尚未修，接手者第一优先）

`mod_orphan::scan_tree` 只报告「某目录下未被声明的 `.rs` **文件**」，
而 `dual_track/mod.rs` 这类**孤儿目录模块**本身**就是**那个未声明的文件，
且其目录内没有其它 `.rs` ⇒ **扫描结果为空**。

⇒ **「孤儿目录模块」这一整类是盲区**，真实孤儿数高于我报的数。
已知同批（早先用未修好的 python 版扫到过）：
`crawl/{ordered_backend_router,dom_extractor}/mod.rs`、
`dream_replay/mod.rs`、`semantic_routing/mod.rs`。

修法方向：`scan_tree` 增加一条判据 —— 对每个含 `mod.rs` 的目录，
检查**该目录本身**是否被任何 `mod.rs` / `#[path]` / 文件模块声明引用。

## 6. 我犯的错（供接手者对照）

| # | 错误 | 代价 | 教训 |
|---|---|---|---|
| 1 | 用 `git log` 时间较新 ⇒ 判「新版」 | 差点删掉功能更少的分叉 | 时间戳不是内容 |
| 2 | 文件名相同 ⇒ 判「同一物」 | 差点删掉 36 个独有 fn | 同名 ≠ 同一符号（R-P16） |
| 3 | `fn` 名集合无独有 ⇒ 判「内容等价」 | 3 个文件各有 42/43/49 行独有代码 | 符号名 ≠ 实现 |
| 4 | 否定式断言用错路径 | `!contains()` **恒真 = 假绿** | 否定式断言必须配同向存在性断言 |
| 5 | 断言「某文件是孤儿」 | 我自己接入后测试立刻红 | 不断言具体文件名 |
| 6 | `git add && git commit` 忘了先 `rm` | 误读成「被并发冲掉」 | 报 no changes 先看 `git status` |
| 7 | 首次跑测试 OOM 却见 `mem_gate=OPEN` | 白跑一轮 | 门看瞬时值，看不到并行编译峰值 |

## 7. 剩余待办（**我未做，需决策**）

| 项 | 状态 | 阻塞原因 |
|---|---|---|
| `nt_memory_kb/context_budget.rs`（76 行） | **故意保留** | 能力**真实**（按模块动态 allocate/release/优先级），但**层位错误**（应在 `nt_core_context`）。并入 = 新造能力，非补缺陷 |
| 检测器「孤儿目录模块」盲区 | 未修 | 见 §5 |
| `spawn_handler!` 宏提到模块级 | 未做 | 要改 51 个调用点，且会与 TUI 那批改动撞车 |
| v1/v2 `orchestrator` 正典 | 未定 | v1 是活路径（`l1_facade.rs:27`），v2 仅 tests 在用 |

## 8. 收工自查

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
[worktree-gate] ⛔ 5 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /private/tmp/nt-probe
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/nt-v3
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/nt-v4
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/nt-verify
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
[worktree-gate] ♻️  target 累计 27577M ≥ 1024M ⇒ 零风险可回收：sh scripts/ops/nt_worktree_gate.sh clean
```

- 本会话**未新建任何 worktree**（探针全部用 `mktemp -d` 建在 `$TMPDIR` 一次性目录，
  探针自带 `trap cleanup EXIT`）。`/private/tmp/nt-probe` 不是我建的。
- 上述 4 个 `.worktrees/*` **都是他窗的**，`prune` 会先导出 patch，
  但**内容归属不是我的账**，我不动。

### 8.2 未提交改动的去向

**我的改动：零。** 本会话所有提交已落账，末笔 `aa0506bc`。

结束时工作树有 12 个已跟踪未提交文件，**逐个核实均非我所改**：

| 文件 | 归属 |
|---|---|
| `agent_guardrails/{policy_engine,input_validator,output_validator,mod}.rs`、`nt_shield/{mod,nt_shield_enforcer}.rs` | 他窗 WIP（`policy_engine.rs` mtime 距我查询 11 秒） |
| `goal_loop/loop_impl/core.rs` | **他窗新增的 emoji 簇测试**（已核实该 diff 不含我的 `truncate` 改动 ⇒ `3b742cfe` 已完整落账） |
| `.neotrix/capability_{overrides,registry}.json`、`neobot/src/nt_policy.rs`、`entry/headless.rs`、`act/actions/security/sandbox.rs` | 他窗 WIP |

⇒ 「留给下一个 agent」不算合法去向，但这些**本就不是我的**，
我不替它们声明去向 —— 各窗口在 §8.2 自己交代。

### 8.3 门状态

| 门 | 结果 | 时间 |
|---|---|---|
| `cargo test -p neotrix --lib -j2` | **13285 passed / 1 failed** | 21:2x |
| └ 唯一失败 | `policy_engine::test_false_positive_override_blocks_downgrade` | **他窗 WIP**（文件 `M`，mtime 11 秒前） |
| `cargo check -p neotrix --lib` | **0 error** | 21:2x |
| `nt_lock_audit` | **0 处** | 20:0x |
| `nt_mem_gate` | OPEN | 21:2x |
| `check-push-deletions.sh` | **PASS rc=0**（本轮新激活，双向实测过） | 20:5x |
| `check-feature-gates --quick` | ⚠️ **本轮未取得干净结果** —— 三次被他窗并发 WIP 打断（`node.rs` / `nt_provenance.rs` / `capability_registry.rs`），最后一次见其自行转绿 | — |

⚠️ `feature-gates` **未记 PASS** —— 门不可用时如实标注，不写虚假记录（R-SCAN-3）。

### 8.4 顺带提示接手者

- **测试编译须限并发**：`cargo test -p neotrix --lib -j2 -- <filter>`
  （`-j2` 在 `--` **之前**）。默认并行会把 rustc OOM-kill，而 `mem_gate` 仍报 OPEN。
- `push` 仍会被 `worktree_gate`（rc=4，他窗 4 个脏 worktree）拦住，
  但**删除声明门那一环已不再是死锁**。

## 9. 指针

- 判定方法论：`docs/architecture/` + 本文件 §4
- 检测器实现与口径：`neotrix-core/src/l0_substrate/nt_core_platform/mod_orphan.rs`
- 门实现：`scripts/check-commit-deletions.sh`、`scripts/check-push-deletions.sh`、`.githooks/prepare-commit-msg`
- 孤儿全量台账：`TODO.md` 待办 18（含补遗一/二/三）