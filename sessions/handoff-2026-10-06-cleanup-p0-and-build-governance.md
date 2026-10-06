# handoff — 2026-10-06 收尾：磁盘治理 + P0 注册表数据丢失 + 交接索引

> 分支 `feat/capability-absorb-20260828` · 本窗口提交 4 个 · 磁盘 48Gi → 248Gi
> 上游交接：`handoff-2026-10-06-capability-invoke.md`（同一日的接线轮）

## 0. 四件事与净结果

| # | 事项 | 结果 |
|---|---|---|
| 1 | **P0：一次只读写命令销毁能力注册表 318 节点** | 已修 + 加跨进程门 |
| 2 | 70 份 handoff 无索引、缺陷散落沉底 | 已建索引（现 72 份）+ 缺陷单一入口 |
| 3 | 磁盘 300G 被 `cargo clean` 挥霍 | **可用 48Gi → 248Gi** |
| 4 | 编译耗时/target 治理 | `target` 9.7G → 6.8G（**−30%**）+ lld 接上 |

## 1. P0：写命令把注册表覆盖成 41 节点，退出码 0

拿已提交注册表喂真二进制跑 `get`（**只读**子命令）：318 节点/43 边 → **41/0**，
3/3 复现，**退出码 0**。三处缺陷叠加：

1. `CapabilityNode::kind` 是**唯一没 `#[serde(default)]`** 的语义字段，
   而已提交文件里 **318 个节点全无此键** ⇒ `from_str` 整体失败；
2. `load_registry` 用 `Err(_)` 吞错误 ⇒ 误走「老 schema」迁移（0 节点）；
3. `save_registry` 在 `run()` 末尾**无条件**执行 ⇒ 写回 41 个 roadmap 节点。

**修法**（`83bc568b`）：`kind` 加 `serde(default)`（默认 `Skill`，依据是实测
该文件里 `consciousness::gap::*` 一个都没有，缺口由 runtime 另行注册）；
迁移判别改为**结构**判别并抽成 `looks_like_legacy_schema()`，新 schema 损坏时
**响亮报错拒绝继续**。

⚠️ 附带修：快照**跨进程不确定**（直接序列化照抄 `HashMap` 迭代序，
同一输入 5 次写盘产出 **5 个不同 md5**）⇒ 改走 `serde_json::Value` 中转。
这也是工作树里那两个 `.neotrix/*.json` 出现 706 增/706 删**假 diff** 的成因。

**门**：`scripts/ops/nt-registry-determinism.sh`（起 5 次独立进程比字节 +
断言节点/边零丢失），已接 CI 与 `task-index.json`。**双向验证过**：有 bug 时
`FAIL … RC=1`，修好后 `PASS … RC=0`。

## 2. 磁盘：300G 是怎么浪费的，又该怎么正确清

上一步我用了 `cargo clean` —— **错的工具**。本轮量清了正确做法：

```
rm -rf target/debug/incremental   →  回收 14.1G（22G → 7.9G）
cargo build -p neotrix-types      →  10.17s，只重编 1 个 crate，依赖不重编
```

⇒ `target/debug/incremental` 曾占 **14G / 22G（67%）**，是本仓最大的**无界增长**
文件（随编辑次数累积）。**这是 `cargo clean` 的正确替代品。**

⚠️ `cargo-sweep` 按天龄**恰好治不了**这类堆积：实测超 1 天的只有 9 个文件，
而改 profile/rustflags 造成的陈旧代是**当天**产生的（`rand` 有 17 个哈希版本、
**500 个** crate 重名并存）。

删掉的垃圾：`crates/neotrix-neobot/src/nt_provider.rs.bak`、
`.neotrix/capability_registry.json.bak-legacy`（我的修复让生成它的那条 legacy
路径不再可达 ⇒ 死文件）。

⛔ **未动**：`.neotrix/knowledge.db`（查实为 KB 真实数据，`nt_core_state`/
`kb_primitives`/`exp_store` 写它）、`.neotrix/patches/`（9 个，他窗 R-DISK-5
兜底）、`.disabled` 测试、`models/`（gitignored 且 git 保护不到）。

## 3. 编译治理（`df490a75`）

`[profile.dev] debug = "line-tables-only"` + `[profile.dev.package."*"] debug = 0`
⇒ **9.7G → 6.8G（−30%）**。选它不选更小的扁平 `debug = 0`（6.0G），因为实测
`RUST_BACKTRACE=1`：扁平 0 的栈帧只有函数名，本配置下我们自己的 crate 仍有
`probe_frames at ./tests/…:4:14`。

⛔ **已撤回一条不实的结论**：曾在配置与 commit 里写「−45% 编译耗时」。
四次 clean 构建实测 2m55s/3m17s/4m31s/5m14s，**不随 debug 递减** ⇒ 本机耗时
是噪声（共享树多 agent 并发），只有**体积**是确定性的。撤回痕迹保留在
`Cargo.toml` 注释里，避免下个窗口从 git log 再发现它并上当。

**lld 已接**（`brew upgrade lld` 22.1.7 → 23.1.2 才可用；22.1.7 直接报
`libSystem.tbd: unknown architecture: arm64e.x1-macos`）。

## 4. 工具评估：装与不装都基于实测

| 工具 | 实测结论 | 决定 |
|---|---|---|
| `cargo-sweep` | 无 release 资产（只能源码编译）；当时超 7 天仅 4 文件 ⇒ 回收≈0 | **不装** |
| `cargo-nextest` | `group_contracts::*` **13/13 在 nextest 下失败、`cargo test` 下全绿**。定位：`test_sync_group` 单独跑 PASS、与 12 兄弟同跑 FAIL ⇒ **共用固定临时目录**，nextest 多进程并行互相踩 | 装了，**不采用** |
| `lld` | 升级后可用 | **接上** |
| `mold` | 未装；走 LLD 同源 TAPI 解析器，很可能同样失败 | 不装 |
| `kache` / `cargo-overstay` / `Cargo-Rail` | 本机不存在，我对其行为**无可靠认知** | **不写进配置**（不做没验证的承诺） |

## 5. 交接与缺陷清单

- `sessions/README.md` —— **72 份逐份入表，脚本对账零遗漏**（此前 70 份无索引）。
- `sessions/OPEN-DEFECTS.md` —— 缺陷单一入口，P0/P1/P2 + 「编译与磁盘」专项 +
  「本轮新发现」。逐条标 `已核实`/`未核实`/`已修`；⚠️ `未核实` 项是转述，
  **别当既成事实动手**。

**⚠️ 当前 HEAD 是红的，不是本轮引入**：`neotrix` crate 编译不过
（`unused variable: interp`，`input_validator.rs:589`，提交 `0c5b1bc9`）。
证据：`cargo check`（**不调链接器**）同样失败 ⇒ 与构建配置改动无关。
该文件仍被持续编辑 ⇒ 未代改。

## 8. 收工自查（§8 三项必填）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 的输出（EXIT=**4**）：

```
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 66M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2   | 6b57fe08 | HEAD | 0 | 81M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/toctou  | 2cc4badf | HEAD | 3 | 69M | 0M | YES
[worktree-gate] ℹ️ **主树**：21 处未提交 | target 12930M（只报告，不影响退出码）
[worktree-gate] ⛔ 2 个 worktree 的未提交改动不在任何提交里：merge-b / toctou
```

**本会话新建的 worktree：无。** 本轮所有临时目录都用 `mktemp -d` 且脚本内
`trap` 清理（`nt-registry-determinism.sh` 已审过：唯一写入目标是临时目录）。

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | 非本会话 | ⛔ **不动**（他窗 WIP，3 处脏） |
| `.worktrees/nt-v2` | 非本会话 | 不动（干净） |
| `.worktrees/toctou` | 非本会话 | ⛔ **不动**（近 3h 有 `.rs` 活动 ⇒ 明确在用） |

⛔ 未执行 `prune`：三个都不是本会话创建的，且 `merge-b`/`toctou` 带未提交改动
——按 R-DISK-5 必须先 patch 兜底，而它们**不是我窗的** WIP。

### 8.2 未提交改动的去向

主树 21 处未提交，**属于本会话的只有 2 个**（其余全为他窗 WIP，不动）：

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `docs/architecture/LESSONS-2026-09-28-measurement-and-dedup.md` | 追加 L6–L8（对比须同命令同产物 / 耗时是噪声 / 装工具先量收益） | ☐ `git add` 已提交 |
| `docs/architecture/LESSONS-20260929-checked-is-not-verified.md` | 追加 §9–§13（测试区分力靠变异验证 / 跨进程缺陷 / 门会返回 0 / 配置静默不生效=假成功 / author 不能区分窗口） | ☐ `git add` 已提交 |
| 本会话的 4 个提交 | `82824585` `83bc568b` `0adfa4ce` `bc458f57` `df490a75` `2cc4badf` | ☑ 全部已入库 |

⛔ **明确不动**（他窗 WIP，交接者自行处理）：`.neotrix/capability_*.json`、
`crates/neotrix-types/src/core/nt_core_approval.rs`、
`neotrix-core/src/l3_embodiment/**`、`neotrix-core/src/l5_cognition/**`、
`neotrix-core/src/l6_meta/nt_approval.rs`、`scripts/ops/nt_sdlc_metrics.py`、
`.neotrix/patches/*`、`.neotrix/knowledge.db`。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**4**（他窗 worktree 带脏 + 主树有他窗 WIP）
- 提交前是否跑过测试：☑ 是 — `nt_core_capability_tree` **54 绿**、
  `neotrix-neobot --lib` **561 绿**、`nt-registry-determinism.sh` **PASS**
- 门红归属：
  - `cargo build -p neotrix` 红 ⇒ **他窗引入**（提交 `0c5b1bc9`）
  - `scan_tree_sorted_by_lines_desc` 红 ⇒ **他窗引入**（`64913227`）
  - `nt_lock_audit`（capability-tree + core）⇒ **0 处**
  - `check-layer-deps --strict` ⇒ **0 new**（13 known）
  - `neobot-check-market.sh` ⇒ **PASS**

## 9. 下一步（按优先级）

1. **修 P0-0**：`input_validator.rs:589` 的 `unused variable: interp`（当前 HEAD 编译不过）
2. **执行通路**：`capability_invoke` 仍不执行能力本体 ⇒ 裁决 A/B/C
   （见 `handoff-2026-10-06-capability-invoke.md` §3）。⛔ 别让 neobot 直接
   `use neotrix_core`（循环依赖）
3. **计数语义**：`dispatch_by_capability` 只解析就计数 ⇒ 污染 `never_invoked`
4. **U+FFFD**：core 2 处（OPEN-DEFECTS N2），建议加门防复发
5. `.project-map/`（354M）是否清理 —— 不在授权内，需确认

## 10. 纪律记录（本窗口实际执行）

- 提交撞上 `.git/index.lock` ⇒ 查明是**他窗活跃的 `.githooks/pre-commit`**
  （PID 21213）⇒ **等待 45s** 而非删锁（删锁会破坏他的提交）
- `registry.rs` 的 mtime 比当前时间**还晚 54 分钟** ⇒ 判定疑似他窗带偏时钟在写
  ⇒ **放弃**该文件的星号清理（纯装饰不值得撞车）
- 每条新写测试都做了**变异验证**；三个变异中有一个抓不到，据此删掉了
  `sort_json_keys`（证实是死代码）而非留着