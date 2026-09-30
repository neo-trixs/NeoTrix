# handoff — 第二棵树 B 方案 第 2/8、3/8 模块回流（2026-09-30）

窗口任务：把 `nt_capability_bridge` → `l1_action/`、`ffi/` → `l0_substrate/`。
**未提交**（用户要求留在工作区，由用户提交）。

## ① 状态：三绿，工作区即为治愈态

| 检查 | 结果 |
|---|---|
| `cargo check -p neotrix --lib -j4` | rc=0 |
| `cargo check -p neotrix --lib --features ios-bridge -j4` | rc=0 |
| `cargo check -p neotrix --bin neotrix-experience -j4` | rc=0 |
| `check-layer-deps.sh --strict` | rc=0，`PASS: 0 new; 10 known` |
| `check-unwrap.sh --strict` | rc=1，**仅** 1 条既存 `apps/neobot-desktop/src/main.rs:51`（clean HEAD 亦红，非本次引入） |

## ② 🔴 最重要：共享分支 HEAD 一度是红的，且是别人提交造成的

`git mv` 会**自动 stage**。另一窗口（author `openhands`）用**不带 `--only`** 的
`git commit` 把整个 index 扫走 ⇒ **我的 14 个 rename 被卷进 `dcccecb6`
（`feat(neobot): 列表与成员改读真 store`）**，而我的内容改动当时还是 unstaged。

结果：`dcccecb6` 只搬了文件、**没有**搬 `mod.rs` 声明与路径，实测编译失败：

```
$ cargo check -p neotrix --lib            # dcccecb6
error[E0583]: file not found for module `nt_capability_bridge`
$ cargo check -p neotrix --lib --features ios-bridge   # dcccecb6
error[E0583]: file not found for module `nt_capability_bridge`
error[E0583]: file not found for module `ffi`
```

⇒ **默认 feature 也是红的**（`nt_capability_bridge` 的 `pub mod` 不受 cfg 门控）。
**本工作区的未提交改动就是修复它的东西**，请尽快提交，否则分支持续红。

这正是 AGENTS.md §1 记的「暂存区核对与提交不原子」事故，第二次实测复现。
⇒ **教训：`git mv` 在共享工作树里等于替别人准备了一份 commit 载荷。**
下轮改用「`git mv` 之后立刻 `git restore --staged <paths>`」，或直接
`cp`+`git rm`，把暂存决定权留给自己。

## ③ 提交时务必用 `--only`

我的 22 个文件（**不要** `git add -A`，共享树里有他窗 WIP）：
`.neotrix/layer-map.json`、`neotrix-core/build.rs`、`neotrix-core/src/lib.rs`、
`neotrix-core/src/neotrix/mod.rs`、`neotrix-core/src/l0_substrate/mod.rs`、
`neotrix-core/src/l0_substrate/ffi/*.rs`(11)、
`neotrix-core/src/l1_action/mod.rs`、
`neotrix-core/src/bin/experience/exp_distill.rs`、
`neotrix-core/src/l5_cognition/nt_core_cad_consciousness.rs`、
`neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs`、
`scripts/layer-deps-baseline.txt`、`scripts/unwrap-baseline.txt`

## ④ 本轮发现的三个隐性陷阱（都不在原任务清单里）

1. **`build.rs` 有 udl 路径**：`src/neotrix/ffi/neotrix.udl`。不改不会报错，
   只会让 `.udl` 改动**不再触发重编** ⇒ 已改。
2. **两个 ledger 按路径做 key**：`layer-deps-baseline.txt`(3 行)、
   `unwrap-baseline.txt`(58 行)。不迁就变 3+58 条「新违规」。已按路径前缀迁移。
3. 🔴 **`layer-deps-baseline.txt` 必须是排序过的**。门用
   `comm -23 "$CUR" "$BASELINE"`，而 `$CUR` 是 `sort -u` 过的、baseline 是原样读的
   ⇒ baseline 一旦乱序，`comm` 会**同时**报「resolved: 3」和「NEW: 3」自相矛盾。
   改路径后必须 `sort -u`。`unwrap-baseline.txt` 用 Python set 比较，**不**受此影响。

## ⑤ 未处理的既存问题（不在本任务范围，建议单开）

- `.neotrix/layer-map.json:92` 仍登记 `neotrix/proxy_daemon_wrapper`，
  但该文件（第 1/8 个模块）早已迁到 `l0_substrate/` ⇒ 门每次刷 6 条
  `WARN: declared tree missing`。本轮两个模块我都同步改了 layer-map key，
  **唯独这个漏的没动**（属别的模块的记录，不擅自改）。
- AGENTS.md §4.2 写 layer-deps「8 known」，clean HEAD 实测 **10 known**。门记录已过期。
- `check-unwrap.sh --strict` 在 clean HEAD 即红（`apps/.../main.rs:51`）。

## ⑥ 拓扑实测（干净检出测量台，`git worktree add --detach`）

| 指标 | 基线 8cfcc2b0 | 现在 | 差 |
|---|---|---|---|
| layered | 2363 files / 722,850 loc | 2376 / 726,379 | **+13 files**（+1 bridge、+12 ffi）|
| second-tree | 129 files / 44,885 loc | 116 / 41,356 | **−13 files** |

## ⑦ 收工自查

- worktree：本轮自建 1 个测量台
  `/var/folders/.../opencode/nt-base`（一次性，量完即用）⇒ **已 `worktree remove --force` 收掉**。
  他窗 3 个 worktree（`/private/tmp/nt-v9`、`.worktrees/merge-b`、`.worktrees/nt-stop`）**未碰**。
- 未提交改动去向：**明确留在主工作区**（用户要求不 commit），清单见 ③。
- 未跑 `--all-targets` / 全量 `cargo test`。唯一越界处：为了验证
  `exp_distill.rs`（它在 `src/bin/` 下，`--lib` 覆盖不到）单独跑了
  `--bin neotrix-experience`。

---

# 第 4/8 个模块：`nt_jev` → `l5_cognition/nt_jev`（同会话追加，未提交）

用户要求留在工作区（不 commit）。HEAD 起点 `18fe06ec`。

## ① 状态

| 检查 | 结果 |
|---|---|
| `cargo check -p neotrix --lib -j4` | **rc=0**，0 error 0 warning |
| `check-layer-deps.sh --strict` | **rc=0**，`PASS: 0 new; 12 known`（原 10） |
| `WARN: declared tree missing` | **0** |
| `check-unwrap.sh --strict` | rc=1，**1 条既存** `apps/neobot-desktop/src/main.rs:53`（该文件 clean-vs-HEAD，我零改动；上轮记为 `:51`，被行号漂移） |
| `nt_lock_audit.py neotrix-core/src` | **0 处**（2026-09-30 14:0x 改完 `.rs` 后重跑，非沿用旧值） |

## ② 引用面：30 处 / 14 文件（`rg --glob '*.rs'`，非 grep）

l6_meta 9（`evolving_evaluator.rs:13,14,198,208,209`；`evolution/absorber.rs:7,8,9`；
`evolution/loop_runner.rs:14`）· nt_crystal_core 13 · l5_cognition 4（`handlers_crystal.rs:51,52,54,55`）
· l3_embodiment 2（`guard.rs:16`、`safety_kernel.rs:12`）· 模块内自引 2（`audit.rs:87`、`conversion.rs:286` doc）。
未超 40；`nt_crystal_core` 之外只碰 3 个层目录（l3/l5/l6），未超阈值。

## ③ 搬运用 `mv` 不用 `git mv`（遵 §② 教训）

index **零变动**（`git status` 无任何 `A`/`R` staged）。14 文件中 **12 个逐字节等同 HEAD**，
仅 `audit.rs`/`conversion.rs` 各差 1 行（就是那 2 处路径改写）⇒ git 提交时会识别为 rename。

## ④ 三个隐性坑的实际命中情况

1. **`build.rs`**：只有 `ffi/neotrix.udl` 一条 `rerun-if-changed`，**与 nt_jev 无关**，
   `Cargo.toml` 亦零命中 ⇒ 本模块无此坑。
2. **`layer-deps-baseline.txt`**：⛔ **本轮新暴露 2 条**，不是「漏迁」而是**门第一次看见既存边**：
   `l3_embodiment` 的 `guard.rs`/`safety_kernel.rs` 早就在消费层归属为 l5 的 nt_jev，
   之前写 `crate::neotrix::nt_jev` 门匹配不到 `l5_cognition` 字面量；改路径后立刻被记成新违规。
   按「sanctioned channel」棘轮进 baseline（`--update-baseline` 生成，`git diff` 实测 **恰好 +2**，
   排序位置正确，`comm` 未报 resolved/NEW 自相矛盾）。
3. **`layer-map.json` key**：`neotrix/nt_jev` → `l5_cognition/nt_jev`，note 追加原路径与新路径。
   顺手把 `live_consumers` 由 5 条补到 **12 条**（`_rule` 要求消费点证据，旧列表漏掉 l3/l5/l6 全部）。
   `files/lines` 实测 14/4312 未变，未动。

## ⑤ 拓扑实测（脏树口径，PRE/POST 同台）

| 指标 | PRE | POST | 差 |
|---|---|---|---|
| layered | 2376 files / 726,391 loc | **2390 / 730,705** | **+14 files**，+4,314 loc |
| second-tree | 116 files / 41,356 loc | **102 / 37,045** | **−14 files**，−4,311 loc |

loc 差额可对账：+4,312（搬入 14 文件）+2（`l5_cognition/mod.rs` 的声明+注释）= +4,314 ✓；
−4,312 +1（`neotrix/mod.rs` +2 doc / −1 声明）= −4,311 ✓。

## ⑥ 提交清单（⛔ 必须 `--only`，共享树里另有他窗 WIP）

我的 **19 个路径**（其余 `git status` 里的 icons / `Cargo.lock` / `ffi/*.rs` / `nt_media/*` /
`capability_registry.json` / `results.tsv` 都不是我的）：

```
git commit --only \
  .neotrix/layer-map.json \
  scripts/layer-deps-baseline.txt scripts/unwrap-baseline.txt \
  neotrix-core/src/l5_cognition/mod.rs neotrix-core/src/neotrix/mod.rs \
  neotrix-core/src/l5_cognition/nt_jev \
  neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_crystal.rs \
  neotrix-core/src/l6_meta/evolving_evaluator.rs \
  neotrix-core/src/l6_meta/evolution/evolution_loop/absorber.rs \
  neotrix-core/src/l6_meta/evolution/evolution_loop/loop_runner.rs \
  neotrix-core/src/l3_embodiment/nt_shield/shield_core/guard.rs \
  neotrix-core/src/l3_embodiment/nt_shield/shield_core/safety_kernel.rs \
  neotrix-core/src/neotrix/nt_crystal_core/{nt_eval_loop,nt_jev_agentjev,nt_jev_calibration,nt_crystal_dialogue,nt_crystal_task_fusion,nt_self_iterate}.rs \
  neotrix-core/src/neotrix/nt_jev
```

⚠️ `docs/architecture/CODE-TOPOLOGY.md` **本轮之前就已被他窗改过**，`nt_topology.py` 又重写了它
⇒ 该文件是**共享脏文件**，提交前请人工裁决是否拆账。

## ⑦ 收工自查

### 8.1 worktree 去向

`nt_worktree_gate.sh check` → **3 个，全部是他窗的，本会话新建 0 个**：

```
/private/tmp/nt-v9                        a005db44  脏3  51M     近3h: no
.worktrees/merge-b                        1a48ecd3  脏3  4784M   近3h: YES   ⛔ 他窗在用
.worktrees/nt-stop (feat/im-stop)         b9be70d9  脏0  50M     近3h: no
合计 4885M（target 4718M）
```

⛔ **一个都没删**（2 个带未提交改动、1 个近 3h 有 `.rs` 活动 ⇒ R-DISK-5/DISK-1）。
`target` 4718M 可回收但属他窗，**留给他自己跑 `clean`**。

### 8.2 未提交改动去向

**明确留在主工作区，由用户提交**（用户指令：⛔ 不 commit）。清单见 ⑥，共 19 个路径。
本会话**未创建任何 patch/备份**——因为 14 个被删文件在工作区里是「旧路径 D + 新路径 untracked」，
内容全在新路径，无内容处于「只在删除侧」的状态，不存在丢失面。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**0**
- 跑过 `cargo check -p neotrix --lib`：**是**（rc=0）
- `check-unwrap.sh --strict` 红 1 条 = **既存/他窗**（`apps/neobot-desktop/src/main.rs:53`，clean-vs-HEAD）
- `check-layer-deps.sh --strict` **绿**；多出的 2 条 known = **本会话暴露的既存 L3→L5 边**，
  已棘轮并在 layer-map note 里写明理由
- 未跑 `--all-targets` / 全量 `cargo test`（遵硬约束）

## ⑧ 下一轮注意

- `nt_jev` 现在的 L5→L6 边为 **0**（`evolve.rs:9` 那句 `l6_meta` 是 `//!` doc 注释，被门
  `rg -v ':[0-9]+:\s*//'` 过滤），所以本模块**没有**引入新的 L5→L6 违规。
- 第二棵树还剩 4 个：`nt_file_ability`(44) / `nt_crystal_core`(52) / `nt_core_error`(1) /
  `nt_core_event_bus`(1)。`nt_crystal_core` 有 6 个 L1 消费者 ⇒ L1→L5 边会像本轮 L3 一样
  **一次性暴露多条**，建议提前预留 baseline 棘轮额度，别误判成「搬坏了」。
