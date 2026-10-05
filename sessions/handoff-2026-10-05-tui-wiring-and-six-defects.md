# handoff — TUI 接线 + 六个真缺陷（2026-10-05）

> 窗口任务：让 NeoTrix 自主补齐缺陷 / 把已吸收能力接进 CLI 架构。
> 本文档只记**实测证据**与**未解决项**，不记快照正文。

---

## 1. 两个 commit（已入库）

| commit | 内容 |
|---|---|
| `e6386797` | 12 files — 六个真缺陷 + TUI 配色集中 |
| `a0289cd3` | 1 file — 补提交 `nt_tui_theme.rs` |

⚠️ **`a0289cd3` 是必需的补救，不是冗余**：`git commit --only` **不含未 tracked 路径**，
即使该路径已 `git add`。所以 `e6386797` 漏了 `nt_tui_theme.rs`，而同 commit 的
`mod.rs` 已含 `pub mod nt_tui_theme;` ⇒ 若无 `a0289cd3`，主干引用不存在的模块、编译不过。

⇒ **给下一个 agent 的操作性教训**：新建文件 + `mod.rs` 改动**不能在同一个
`--only` commit 里**，必须拆两个 commit，或先 `git add` 后用不带 `--only` 的
`git commit <path>` 补。

### 修掉的六个真缺陷（全附证据，详见 commit message）

1. **`nt_memory_pack` 头部解析无边界检查** ⇒ 畸形 `.ntpack` 令 release
   `panic = "abort"` **整进程中止**。生产路径 `nt_geo_ntpack_cold.rs:78`。
   新增 `Cursor`；`dict_len` 原为**分配炸弹**（16 字节文件可申请 4 GiB）。
2. **`noise_handshake` 两条 `#[ignore]` 前提已失效** ⇒ crypto 门被挂起。
   ⭐ 其中「nonce 必须自增」此前**无任何活动测试覆盖**（活动测试用 `seal()`
   绕过 `encrypt_and_hash`）⇒ nonce 复用回归会静默通过。
3. **`goal_loop::truncate` 三种单位混用** ⇒ 中文超预算 25 列（旧测试全 ASCII）。
4. **`estimate_tokens` 两处违背 P0-7 单一事实源** ⇒ CJK **高估 3×**。
5. **两处重言式断言** `assert!(X.is_empty() || !X.is_empty())` = 永不失败的测试。
6. **`tun_device` doc 谎称已实现**（29 行只有零引用枚举 + `#[ignore]` 空体测试）。

---

## 2. ⛔ 我自己引入并被下个 commit 修掉的 bug（重要方法论）

`truncate` 的补白我放在 `marker` **之后** ⇒ CJK + `max=28` 时结果
`"让系统自主进化并持续涌现... "`（**末字符是空格**），列宽 28 达标但
`ends_with("...")` 契约破坏。

- 他窗在 `3b742cfe` 修掉，诊断正确（补白须在 marker 之前）。
- ⛔ **我的测试 `:649` 明明有 `assert!(long.ends_with("..."))` 却没抓到** ——
  因为当时 `nt-core-capability-tree/node.rs` 处于编译失败的中间态，
  **测试二进制根本没跑起来**，我的"全量 passed"是另一个中间态下取的。
- ⇒ **教训**：`test result: ok` 只在**该次运行的编译确实包含我的代码**时才算数。
  他窗 WIP 频繁使编译失败 ⇒ 验证结论必须标注**时间戳与当时 HEAD**，
  否则会像我这次一样，把"未验证"误报成"已通过"。

---

## 3. ⛔ 交接时点的未验证项（接手者须知）

`3b742cfe` 修完 `truncate` 后，我**未能**再跑一次验证 —— `neotrix-neobot`
（他窗 WIP）出现 `E0252`/`E0599` 编译失败，测试二进制无法构建。

**接手者第一件事**（他窗收敛后）：

```sh
sh scripts/ops/nt_mem_gate.sh; echo "rc=$?"      # 非 0 禁起构建
cargo test -p neotrix --lib truncate            # 应含 test_truncate_cjk_respects_column_width
cargo test -p neotrix --lib nt_memory_pack      # 17
cargo test -p neotrix --lib noise_handshake     # 3（含 2 条解除挂起的 crypto 门）
cargo test -p neotrix --lib                     # 全量
```

**最后一次可信的全量结果**：`13233 passed; 0 failed; 38 ignored`
—— 时间点在他窗 `3b742cfe` 之前，**不含**该 commit 对 `truncate` 的修正。

---

## 4. 门状态（实测，非声称）

| 门 | 值 | 时间 |
|---|---|---|
| `nt_mem_gate.sh` | rc=0（avail 176G） | 本窗口开始 |
| `nt_lock_audit.py neotrix-core/src` | 可疑 **0** 处，rc=0 | 收尾复核 |
| `nt_worktree_gate.sh check` | rc=0 | 收尾 |
| `cargo check -p neotrix --lib` | ✅ 0 error | 阻塞解除后 |
| `cargo test -p neotrix-types --lib` | ✅ **440 passed** | 独立 crate，不受阻塞 |
| `check-layer-deps.sh --strict` | ⚠️ `FAIL: 1 new` | **非本会话引入** |

### 层门那 1 条的归属（已用干净检出定性）

`l2_perception → nt_model_cli.rs:338`（引用 `nt_world::social_access::probe::run_with_timeout`）。
`nt_model_cli.rs` 的 `git status` **为空**（本会话从未改它）。
按 AGENTS.md「测量台必须是 `git worktree add --detach HEAD` 的干净检出」实测：
**干净 HEAD（不含本会话任何改动）同样 `FAIL: 1 new`** ⇒ 既有问题，来自 `2bbed32c`。

⚠️ **该门现在是红的，CI 会红**。修法二选一：
① 把 `nt_model_cli.rs:338` 的 L2 引用改走 facade；
② 确认 `probe.rs` 是否 sanctioned 通道后扩baseline。
**本会话未做**（非本会话引入，且改层引用须走消费方自己那层的 facade）。

---

## 5. 收工自查（模板 §8）

### 8.1 worktree 去向

- `nt_worktree_gate.sh check` 输出见上节，rc=0。
- **本会话新建的 worktree：1 个** ——
  `/var/folders/…/opencode/nt-clean-gate`（用途：用**干净检出**定性层门那 1 条
  是否本会话引入）。去向：✅ **已 `git worktree remove --force` 移除**，
  `git worktree list` 已复核无残留。
- **其余 4 个 worktree 非本会话所开**，且门报「2 个近 3h 仍有 .rs 改动 ⇒可能他窗在用」：
  `.worktrees/merge-b`、`.worktrees/nt-stop`、`.worktrees/nt-v2`、`.worktrees/nt-verify`
  ⇒ ⛔ **不动**（手删即永久丢失，必须走 `prune`）。

### 8.2 未提交改动的去向

**本会话的 13 个改动文件全部已入库**（`e6386797` + `a0289cd3`），
`git status --porcelain --<我的13个路径>` 已复核为空。

| 文件 | 改动 | 去向 |
|---|---|---|
| `nt_tui_theme.rs`（新建） | TUI 配色集中点 + 2 不变量测试 | ☑ 已提交 `a0289cd3` |
| `l1_action/mod.rs` | 注册 `pub mod nt_tui_theme;` | ☑ 已提交 `e6386797` |
| `l1_action/nt_tui_app.rs` | bar_row 绝对耗时 / picker 视口 / 11 处样式 / `/exit` | ☑ 已提交 `e6386797` |
| `l1_action/nt_dialogue_tui.rs` | 4 处样式 + `demand_style` 改纯转出 | ☑ 已提交 `e6386797` |
| `l1_action/nt_io/nt_io_mention.rs` | 竞态测试唯一名 | ☑ 已提交 `e6386797` |
| `l1_action/nt_file_ability/chunk_planner.rs` | `estimate_tokens` 转出事实源 | ☑ 已提交 `e6386797` |
| `nt_shield_ztnet/crypto/noise_handshake.rs` | 解除 2 条失效 `#[ignore]` | ☑ 已提交 `e6386797` |
| `nt_shield_ztnet/packet/tun_device.rs` | doc 降级 + 空体测试换实质断言 | ☑ 已提交 `e6386797` |
| `nt_shield_audit/cloudflare_patterns.rs` | 重言式断言 → 钉 severity→status | ☑ 已提交 `e6386797` |
| `l4_emotion/nt_memory/addressable_store.rs` | `estimate_tokens` + `make_summary` 单位 | ☑ 已提交 `e6386797` |
| `nt_memory_kb/nt_memory_pack.rs` | `Cursor` 边界检查 + 3 回归测试 | ☑ 已提交 `e6386797` |
| `goal_loop/loop_impl/core.rs` | `truncate` 列宽（**已被 `3b742cfe` 补正**） | ☑ 已提交 `e6386797` |
| `neotrix-types/.../bank/mod.rs` | 重言式断言 → 钉遍历含起点 | ☑ 已提交 `e6386797` |

**其余 40 处未提交改动全部是他窗的**（`llm_judge` / `nt_capability_registry` /
`node.rs` / `gateway/*` / `nt_determinism` / `nt_pet` / `5a3dce85` 那批吸收文档…）
⇒ ⛔ **本会话一律未碰、未提交、未 `stash`**。

### 8.3 门状态补充

- `cargo check -p neotrix --lib`：☑ 是（阻塞解除后 0 error）
- `cargo test -p neotrix --lib`：☑ 是（`13233 passed`，**但见 §3 时间戳告警**）
- 门红归属：层门 `FAIL: 1 new` = **他窗/既有**，非本会话引入（已用干净检出定性）

---

## 6. 下一轮的真实缺口（已用证据排除伪候选）

**被证据否决，不该做**：

|候选 | 否决依据 |
|---|---|
| `astar`/`build_flow` 调度层 | 承载 crate `neotrix-game` 已在 `2bbed32c` 被删，且**删前即零引用**（在 `2bbed32c^` 上 `git grep` 过）⇒ 强接就是造第二个 `astar` |
| `nt_fov`（`compute_fov`/`los_clear`） | 同形态零消费，且 `nt_game` 生态已拆 ⇒ **ROUND25 把它漏记了**，但同样无消费者 |
| `panel.rs` 接 TUI | TUI 各区几何已由 ratatui `Block::borders` 负责（6 处）⇒ 会**双层边框** |
| `/lsp` | `LspManager` 消费者 0、无 status API ⇒ 前置门第2 问「路径可达」答不出 |
| `nt_token_guard.rs` 的 `chars/4` | 独立 crate，引用 `neotrix-core` 会**成环**；且语义不同（向上取整、空串=0，用于 warn/block 门） |
| `crystal_integration.rs` | **实测编译通过**，文件第 3 行注释明写「Dead test，门控在不存在的 feature 后」⇒ 有意为之。⚠️ 但这本身是「永远绿的门」模式 |

**真实待办（按杠杆）**：

1. **`ext.rs` 18 个被注释测试**：14 个的禁用理由「cosine_similarity not migrated」
   对它们**不成立**（它们用的 API 全部存在且 `pub`）⇒ 12 个 `pub` API 零覆盖。
   解除需实跑 `cargo test -p neotrix --lib nt_core_bank` 逐个确认。
2. **`/mcp` slash 命令**：命令骨架 15 分钟，但**数据源跨 crate**——
   `GLOBAL_MCP` 是私有 static 无公开读口，`bin/ntcode.rs` 完全无 `McpRegistry`。
   需先裁决：给 `run_tui_session` 传参，还是 `agent.rs` 加 `global_mcp_snapshot()`。
3. **层门那 1 条**（见 §4）—— 现在 CI 会红。
4. **`nt_absorption_live.py` 判据收紧**：ROUND25 §6.2指出
   `find_file` 认「任意 basename 存在即存活」⇒「`死亡=0` 永远是假的」。
   实测仍如此（`导出未接线` 从 5→6，**六天没改善**）。

---

*End of handoff — 2026-10-05*

---

## 7. 追加（第二轮，已实测完成）

### commit `69d97c4d` — 恢复 14 个被注释测试（**已实测通过**）

`neotrix-core/src/l1_action/nt_core_bank/bank/ext.rs`：18 个被注释 `#[test]`
中只有 5 个真需要 `cosine_similarity`（实测该函数全仓零命中）。
其余 14 个是**连坐禁用** —— 其依赖 API 全部存在且 `pub`（逐个核实 13 个）。

- 恢复 14 个（`#[test]` 27 → 41）
- 删 2 个 `{ ... }` 占位桩（不是被禁测试，是空壳）
- 保留 5 个真禁用项及其理由行
- 补 `MemoryTier` import（`test_promote_tiers` 需要）
- 修一处**孤立 `}`**（删桩后遗留，会打乱花括号配平）
- 顺带修该文件里 `test_enable_hypergraph` 的**重言式断言**（与
  `neotrix-types` 那份是同一 bug 的两份镜像）

**实测结果**（阻塞解除后）：
```
cargo test -p neotrix --lib nt_core_bank   → 106 passed; 0 failed
--list 确认 ext::tests:: 共 41 个全部注册
cargo test -p neotrix --lib                → 13263 passed; 0 failed
nt_lock_audit.py                            → 可疑 0 处
```

⚠️ 期间一次全量出现 `nt_shield_agentic_scan` 失败 —— `git status` 显示该文件
**非本会话改动**，且第二次跑即绿 ⇒ 并发竞态，非回归。

### P2（`nt_absorption_live.py` 判据）—— **实测后决定不改**

花时间量化后得出与原先不同的结论，故记录**否掉的理由**：

- 量化 1：`basename` 唯一命中（`:94`）在真实文档上 **8/8 全是真命中**，无假阳性
- 量化 2：「只出现在 tests/ 目录的标识符」= **0/67** ⇒ 该判据无收获，
  因为标识符都在**混合文件**里（`astar` 定义在生产区、调用在测试区，同一文件）
- 真病根：`:140-145` 纯标识符只验「出现次数 > 0」，而 `rg_count` 搜的范围
  **含测试目录** ⇒ 「测试里出现」即算存活 ⇒ 这才是 `死亡=0` 的机制
- ⛔ 但要修它必须能枚举 `#[cfg(test)]` 边界（判定粒度到文件内行区间），
  属独立工作。**且工具注释 §3.1 已诚实标注「偏宽」**，而 `--graph`
  已能抓到真缺陷（`astar`/`build_flow`）。⇒ 不在本轮改。

### 我自己踩了 AGENTS.md 明令禁止的 `rg -E`

写验证脚本时用了 `rg -E`，本机报 `unknown encoding`（AGENTS.md 已记此坑）。
⇒ 一律用 `rg -n`，零命中先确认退出码 1 而非 2。

---

## 8. 追加（第三轮）—— commit `cf9e03c4`：层门 L1→L2 收敛

### 做了什么

`check-layer-deps.sh` 的规则（`scripts/check-layer-deps.sh:85`）是
`check_layer "l1_action" … "l2_perception" …` ⇒ **层不得引用更高层**，
故 `nt_model_cli.rs` 直引 `l2_perception/…/probe::run_with_timeout` 记违规。

- `nt_action_facade.rs` 加两条 `pub use`（转出 `run_with_timeout` + `RunOutcome`）
- `nt_model_cli.rs:338` 改走 `crate::l1_action::nt_action_facade::run_with_timeout`

⭐ 该 facade `:438-449` 的注释本就写着「本段按目标层**逐批追加**
（先 L3/L6，**再 L2/L4/L5**）」⇒ L2 是仓库自己排好的下一批，本次只是补做。

⛔ **未选替代方案「把 `capture_model_command` 搬进 L2」**：那是语义变更而非接线。
   该函数承载「模型询问」三职责（去 ANSI / 空 stdout 判失败 / 超时与失败可区分），
   搬层牵动 `run_once` + `run_capture`；而文件头正记着 2026-10-03
   「删转发层前没查全部调用方 ⇒ 主路径被打断」的教训。⇒ 只收敛路径，不动语义。

### 验证

- ✅ `cargo check -p neotrix --lib` 0 error
- ✅ **干净检出**（`git worktree add --detach HEAD`）层门实测：
  `nt_model_cli.rs` 与 `nt_action_facade.rs` **均已从 NEW 清单消失**
- ✅ 语义未变：独立 crate 复刻完整判定链并实跑 5 条断言全通过 ——
  ⭐ `timed_out` **先判**（超时被 kill 的子进程必然 `success == false`）
  / 空 stdout / 非零退出 / spawn 失败
- ⛔ 未在 `neotrix-core` 内实跑 `--lib`（他窗 `consciousness_runtime.rs` 持续 WIP 阻塞）

### ⛔ 遗留：层门**仍然红**，但已不是本会话引入

干净检出实测 `FAIL: 2 new`：

```
l5_cognition	neotrix-core/src/l1_action/nt_act/nt_act_trade/capability_registry.rs
l5_cognition	neotrix-core/src/l1_action/nt_capability_bridge.rs
```

⇒ 这两条来自**已入库的他人 commit**（`5d00db76`「5 个贸易能力真正进入市场」、
`8bfff437`「能力插件市场」），**不是工作树 WIP** ⇒ **主干 CI 现在会红**。

**接手者第一件事**：修这2 条或扩 baseline。修法与本次同型 ——
经 `nt_action_facade` 转出（`l5_cognition` 那侧对应 `l5_cognition/l1_facade.rs`）。

## 9. 我这轮踩到的协作现实（值得记）

他窗 WIP 使`neotrix-core` 的 `--lib` 测试二进制**反复构建失败**（本轮观察到
`E0599`/`E0308`/`E0433`/`E0593`/`E0382`/`E0252`，轮转于
`node.rs` / `capability_registry.rs` / `consciousness_runtime.rs` /
`nt_determinism.rs` / `nt_pet.rs`）。

⇒ **可复用的应对**（本轮验证有效）：
1. `git commit --only` 先入库，commit message 里**显式标注哪些已实测、哪些未实测**
2. 用**独立 crate** 复刻关键判定链并实跑（本次 5 条语义断言）
3. **绝不在测试二进制没构建成功时报「测试通过」**——
   我上一轮就犯过这个错（报`13233 passed` 而实际二进制根本没起来）

⚠️ `/private/tmp/nt-probe` worktree 有 **5 处未提交改动不在任何提交里**
（含 `consciousness_runtime.rs`）⇒ ⛔ **禁止手删**，必须走
`sh scripts/ops/nt_worktree_gate.sh prune`。

## 10. 他窗审计查实、本会话未碰的高价值缺口

另一个窗口在对比 `codewhale-hq/Codewhale` 时查实（其 commit message 原文，
**本会话未独立复核**，仅转述以便接手者跟进）：

- ⛔ **`guard/agent_guardrails/`（4 文件 460 行）整目录未被 `mod` 声明 ⇒ 不编译**，
  而 `input_validator.rs` 自述 `R-P132: Guardrails non-optional in production.`
  ⇒ **文档断言与事实相反**。它检测的正是 `rm -rf`/`sudo`/注入/凭据泄漏。
- ⛔ `nt_policy` 的 `WORD_NEEDLES` **无破坏性命令**（`rm`/`dd`/`mkfs`/
  `git push --force` 全放行）
- ⛔ `nt_permission_profiles` 继承合并**无单调**；`action_type_to_key` 只产 4 键
  ⇒ 内置 11 条规则里 **7 条永不可达**，含 `read_secrets` 与 `git_force_push`
  两条 `Deny`；且 `require_approval` 把 `Deny` **降级为「需审批」**
- ⛔ `ActionSandbox` tie-break 让**后注册的 allow 压过先注册的 deny**，
  内置硬拒前缀 `rm:`/`drop:`/`wipe:` 可被一次 `add_rule` 覆盖
- ⭐ `from_client_spelling` 确认「导出 ≠ 接入」（6 个调用点全在 `mod tests` 内）

⇒ 按AGENTS.md R-P79，**「整目录不编译但自称生产强制」是本仓当前最严重的
未修缺口之一**（它不是「导出≠接入」，而是「导出且声明强制，但根本不编译」）。

---

## 11. 追加（第四轮）—— commit `93405549`：agent_guardrails 的 `E0255`

### ⛔ 先纠正我上一轮的错误结论（重要）

上一轮 §10 写「`R-P132` 是**虚构**规则」——**该结论错误**。查实：

```
docs/standards/archive/R-P111-160-RECOVERY.md
:24| R-P129 | HITL gate for high-stakes | agent_guardrails/mod.rs:5
:26| R-P132 | Guardrails non-optional in production | agent_guardrails/mod.rs:3
```

⇒ **R-P132 / R-P129 真实存在**（2026-09-21 打捞记录，正文首源就是该目录）。
我当时**只查了 `AGENTS.md` 与 `RUST-STANDARDS.md` 就下结论**，
漏了 `docs/standards/archive/` ⇒ 正是 **R-SCAN-1b**（裸grep/窄检索的
命中不构成证据）的形态。**已修正。**

⚠️ 附带：**`R-SEC07`（Guardrail decouple-or-judge）在 `docs/standards/` 全无出处**
⇒ 只在该目录内 3 处自引用，属**无源规则引用**。

### 实测结论：「加一行 pub mod」不是接线

临时加 `pub mod agent_guardrails;` 试编译，暴露的是**一串**错误：

| 位置 | 错误 |
|---|---|
| `policy_engine.rs:26` | `E0255` ViolationSeverity 重定义（**已修，本 commit**） |
| `input_validator.rs:16` | 同型 `E0255` |
| `policy_engine.rs:197/199` | `expected &[InputViolation], found &Vec<OutputViolation>` |
| `mod.rs:35` | derive 重复（`Copy` 等） |
| `policy_engine.rs:40` | import 与本地定义冲突 |

⇒ 该目录约 1600 行（15940+3947+14551+16342 B）**自 2026-09-21 从未编译**。
**接线是「移植项目」，不是「加一行声明」。**

⚠️ 其中 `compute_verdict` 那个类型不匹配**不是笔误就是设计缺口**：
`compute_verdict` 只吃 `&[InputViolation]`，而 output 侧的
`Vec<OutputViolation>` 被直接传入 ⇒ **output 违规无法进入 verdict 计算**。
接线前必须先裁决这个语义问题。

### 本 commit 严格限定为一处确定的 `E0255`

- `policy_engine.rs:26` 删掉 import 里的 `ViolationSeverity`
  （同文件 `:34` 已有 `pub enum` 定义）
- ✅ `guard/mod.rs` **已还原**，`git status` 干净
  ⇒ **本 commit 不改变任何编译可达性**，零风险
- ⛔ 未实跑 `--lib`：他窗 `nt_approval.rs` 处于 `E0753` 状态阻塞测试二进制

### 接手者的接线专项（若要做）

1. 先裁决 `compute_verdict` 的 input/output violation 类型不匹配
2. 修余下 4 处编译错误（`input_validator.rs:16` 同型 E0255 等）
3. 加 `pub mod agent_guardrails;` 后跑 `cargo test -p neotrix --lib agent_guardrails`
   —— 该目录**自带测试**（`input_validator.rs` 有 `test_injection_detected` /
   `test_credential_detected` / `test_injection_safe_input`），是验证其正确性的天然抓手
4. 接线后检查层门（本次核实：无 `crate::l[0-6]_` 跨层引用 ⇒ 不触发层门）
5. ⭐ 接线价值：已编译的 3 个兄弟文件里 `prompt_guardian.rs`
   **危险模式命中 0**，而 `input_validator.rs` 有 prompt injection +
   credential leak + `rm -rf`/`sudo` 检测 ⇒ 更完整的那套正是被遗漏的

---

## 12. 追加（第五轮）—— commit `3476e3ec`：⭐ 层门**首次全绿**

```
bash scripts/check-layer-deps.sh --strict   # 干净检出 HEAD=3476e3ec
violation sites: 13   baseline entries: 13
PASS: 0 new violation(s); 13 known/recorded.
```

本会话开始时是 `FAIL: 1 new`；他窗两个 commit（`5d00db76`「5 个贸易能力真正
进入市场」、`8bfff437`「能力插件市场」）把它推到 `2 new` ⇒ **主干 CI 本来会红**。
现全部修复。

### 两条违规完全同类，且**都在测试区**

| 文件 | 行 | 形态 |
|---|---|---|
| `nt_capability_bridge.rs` | 640 | `let _rt = ConsciousnessRuntime::new()` |
| `nt_act/nt_act_trade/capability_registry.rs` | 1130 | 同上 |

`awk` 扫 `#[cfg(test)]` 之前的生产区 ⇒ **两文件生产区零跨层引用**。

### ⛔ 为什么不改门去排除 `#[cfg(test)]`

`scripts/check-layer-deps.sh:70` 自身注释明写
「string literals still count (conservative: **may over-report, never under-**)」
⇒ **过报是该门有意的保守设计**。放宽会削弱捕获面，而本次两条违规是真的
（只是恰好在测试区）⇒ 改引用路径，不改门。

### ⚠️ 我踩的坑（值得记）：层的注释过滤是**逐行前缀匹配**，不是语义判断

第一版我在注释里写了完整路径 `// \`crate::l5_cognition::…\``，
结果**层门仍报违规** —— `:70` 只排除**行首 `//`** 的整行注释，
而我那是**行尾续行**（上一行有 `let _rt =`）⇒ 被判为依赖。

⇒ 写注释时**不要在行尾续行里留层名**，否则会被门当真依赖。
⇒ 这也解释了门为何「may over-report」：它宁可误报也不漏。

### ✅ 验证

- `cargo check -p neotrix --lib` ⇒ 0 error
- 层门干净检出 ⇒ `PASS: 0 new`
- `rg 'crate::l5_cognition|crate::l6_meta'` 两文件 ⇒ 零残留
- **单独跑**：`nt_capability_bridge` ⇒ 9 passed；
  `nt_act_trade` ⇒ **622 passed**（我改的那个测试确实执行了，非被过滤）
- ⛔ **全量 `--lib` 有 3 个失败，但判定为并发竞态、非本会话引入**：
  - 两次全量跑的失败集合**不同**（第一次含 `nt_permission_profiles::test_action_type_to_key`，
    第二次换成 `nt_approval::tests::force_push_is_hard_denied_even_in_full_auto`）
  - `nt_permission_profiles.rs` mtime 19:41（他窗活跃修改中）
  - 我的三个文件在失败清单里 **0 命中**
  - ⭐ 仓库已有同类前科：`nt_io_mention` 竞态（我上上轮修的那个）

### 🩸 我这轮第二次踩 `rg -E`（AGENTS.md 已明令禁止）

写轮询脚本时用了 `rg -oE 'E[0-9]{4}'` ⇒ 本机报
`unknown encoding`（AGENTS.md 记录的坑）。**我上一轮已犯过一次**，
两次都写进交接了 ⇒ 后续一律 `rg -n`，零命中先确认退出码 1 而非 2。

---

## 13. 追加（第六~八轮）：收尾状态快照

> 快照时间：**2026-10-05 21:59**。以下每个数字都是该时刻实测，非引用。

### 13.1 本会话入库的 8 个 commit（全部已核实在 HEAD 祖先链）

| commit | 文件数 | 内容 |
|---|---|---|
| `e6386797` | 12 | 六个真缺陷 + TUI 配色集中（+596/-121） |
| `a0289cd3` | 1 | **补提交** `nt_tui_theme.rs`（`--only` 漏了新文件） |
| `69d97c4d` | 1 | 恢复 14 个被注释测试（+156/-180） |
| `cf9e03c4` | 2 | 收敛 L1→L2 |
| `93405549` | 1 | 修 `agent_guardrails` 的 `E0255` |
| `3476e3ec` | 3 | 收敛 L1→L5 ⇒ **层门首次全绿** |
| `d71ba80e` | 1 | clippy 硬红告警 + NaN 静默退空条 |
| `6a59833a` | 3 | 吸收 break-ui：grapheme 截断 |

### 13.2 门状态（21:59 实测）

| 门 | 值 |
|---|---|
| `nt_mem_gate.sh` | rc=0 |
| `nt_lock_audit.py neotrix-core/src` | 可疑 **0** 处 |
| `bash scripts/check-layer-deps.sh --strict` | ✅ **PASS: 0 new**（13 known） |
| `cargo clippy -p nt-term-viz --all-targets` | ✅ 零 warning |
| `cargo fmt --check` | 🔴 exit=1（**主干长期红**，见§13.4） |
| `cargo clippy -p neotrix --lib --no-deps` | 🔴 **624 error**（见 §13.3） |

### 13.3 🔴 clippy 实测：624 error，且门跑不完

**关键口径**：`--no-deps` 才只算 `neotrix` 本体。**不加 `--no-deps` 时
依赖 crate 自己的 `#![deny(warnings)]` 会把 warning 升级成 error 并灌满输出**
⇒ 我第一版命令因此误判过一次。

| 类型 | 数量 | 能否机器修 |
|---|---|---|
| `unreadable_literal` | **282** | ✅ `--fix` 全自动 |
| `allow_attributes_without_reason` | 98 | ⚠️ 需人写 reason |
| `allow_attributes`（该用 `#[expect]`） | 75 | ⚠️ 语义决策 |
| `similar_names` | 65 | ❌ 人判断 |
| `empty_line_after_doc_comment` | 34 | ✅ 自动 |
| `needless_raw_string_hashes` | 30 | ✅ 自动 |
| `unnested_or_patterns` | 14 | ✅ 自动 |
| `many_single_char_names` | 13 | ❌ 人判断 |
| 🔴 **`E0282` 编译错误** | 1 | **阻塞 clippy 跑完** |

**245 个文件**受影响。热点：`nt_shield_stealth_net/geo_proxy.rs`(82)、
`l3_vendor_skills/mod.rs`(22)、`proxy_detection/ip_fingerprint.rs`(19)。

⛔ **`E0282`（`nt_permission_profiles.rs:124` `HashMap` 类型推断失败）是编译错误
不是 lint** ⇒ `-D warnings` 下 clippy **中途失败** ⇒
**624 只是「已扫到的部分」，真实总量可能更多。**

### 13.4 `cargo fmt` 实测：主干长期红

- 工作树：14818 diff 块 / 1885 文件；**干净 HEAD**：14798 块 / 1884 文件
  ⇒ **与未提交改动无关，是主干长期不合规**
- 工作树只新增 **1 个**不合规文件（`nt_io_agent_loop/mod.rs`，他窗的）
- 性质分类：**43.9% 是「单行展开成多行」**（rustfmt 默认 `max_width=100`），
  7.0% 是反向合并，9.7% 等量替换
- ⚠️ 我第一版统计错在 ANSI 转义 `\x1b(B` 使 `^\+` 匹配不到
  ⇒ 曾误判成「93% 只删不增」。清洗需 `re.sub(r'\x1b\([AB0-9;]*','',txt)`

### 13.5 门配置的真实覆盖（实测，非推测）

| 门 | 命令 | 实际覆盖 |
|---|---|---|
| `ci.yml:390` | `cargo clippy --lib -p neotrix -- -D warnings` | **仅 `neotrix` lib** ⇒ 624 error，硬红 |
| `lint.yml:40` | `cargo clippy --all-targets --all-features -- -D warnings` | ⚠️ **无 `-p`** ⇒ 在仓库根跑，只查根包 ⇒ **几乎等于没跑** |

⇒ `Cargo.toml:50-69` 配的 **18 条 restriction lint**
（`unwrap_used`/`indexing_slicing`/`panic`/`await_holding_lock`…，注释写
「bug prevention R-P2xx」）**除 `neotrix` lib 外全部未被门覆盖**。
`neotrix-types` 的 **508 个 `indexing_slicing` error** 就是这个盲区的产物
（`self_model.rs` 一家占大半，是数值线性代数，全是 `a[i][j]`）。

### 13.6 ⛔ 剩余任务（6项）

**clippy 线（4项，互相依赖，当前全被阻塞）**

| # | 项 | 阻塞原因 |
|---|---|---|
| **P0′** | 修 `nt_permission_profiles.rs:124` 的 `E0282` | 🔴 **他窗活跃 WIP**（mtime 21:59:32，就在快照前几分钟） |
| **P1** | 对非 WIP 文件跑 `clippy --fix`（~350 条机械） | 被 P0′ 阻塞 |
| **P2** | 173 条 `#[allow]` 语义决策 | 被 P1 阻塞 |
| **C2/C1** | 先给超量 crate 加带 reason 的 crate 级 `allow`，再 `lint.yml`加 `--workspace` | 需裁决 |

⚠️ **`clippy --fix` 会重写文件** ⇒ 若清单混入他窗 WIP 就等于覆盖他的工作，
而这个风险**无法用 `git status` 完全排除**（他可能只改了一半）。
⇒ 建议等他窗收敛后一次性做 P0′+P1。

**功能线（2项，需裁决方向）**

| # | 项 | 阻塞点 |
|---|---|---|
| `/mcp` | 数据源跨 crate：`GLOBAL_MCP` 私有 static 无读口，`bin/ntcode.rs` 完全无 `McpRegistry` | 选：改 `run_tui_session` 签名传参 vs `agent.rs` 加 `global_mcp_snapshot()` |
| `agent_guardrails` 接线 | 实测 5 处编译错误（`E0255`×2、`compute_verdict` 类型不匹配、derive 重复）⇒ 是**移植项目**非接线 | 先裁决 `compute_verdict(&[InputViolation])` 被传 `Vec<OutputViolation>` 是笔误还是设计缺口 |

### 13.7 ⚠️ 两条不是任务但更紧急

1. **`/private/tmp/nt-probe` 有 8 处未提交改动不在任何提交里**
   （早先是 5 处，在增长；含 `consciousness_runtime.rs`）
   ⇒ ⛔ **禁止手删**，必须 `sh scripts/ops/nt_worktree_gate.sh prune`。
   这正是 AGENTS.md 记录的 850 处丢失事故场景。
2. **他窗 WIP 频繁使 `neotrix-core` 测试二进制构建失败**
   ⇒ 记住：**测试二进制没构建成功时，`test result: ok` 不算证据。**
   本会话曾因此误报过一次「13233 passed」，已在 §2 记录。

### 13.8 我在本会话犯的错（留作反面教材）

| # | 错 | 代价 |
|---|---|---|
| 1 | `--only` 漏提交新文件（2 次） | 一次让主干编译不过，一次让 break-ui 改动滞留未入库 |
| 2 | 把「未验证」报成「13233 passed」 | 违反「证据先于断言」 |
| 3 | 只查 `AGENTS.md`/`RUST-STANDARDS.md` 就判「R-P132 是虚构的」 | 漏了 `docs/standards/archive/` ⇒ **结论完全错** |
| 4 | 用 `rg -E`（本机 `unknown encoding`） | 连续 3 次 |
| 5 | `cargo clippy` 不加 `--no-deps` | 依赖的 error 灌满输出 ⇒ 误判归属 |
| 6 | diff 统计未清 ANSI `\x1b(B` | 把 4.1% 误判成 93% |
| 7 | 注释里留完整层路径且写成**行尾续行** | 层门只排除行首 `//` ⇒ 自己触发自己修的违规 |

⇒ 共性：**每一条都是「窄检索当下结论」，而仓库反复教我「grep 只找候选行，不下结论」**。

---

## 14. 收工自查（模板 §8 必填项，覆写 §8.1/8.2 的早期快照）

> 快照 **2026-10-05 21:59**。§8.1/8.2 记录的是会话早期状态，**以本节为准**。

### 14.1 worktree 去向

`nt_worktree_gate.sh check` rc=0。本会话**新建 3 个临时 worktree，全部已移除**：

| worktree | 用途 | 去向 |
|---|---|---|
| `…/opencode/nt-clean-gate` | 干净检出定性层门违规归属 | ✅ 已 remove |
| `…/opencode/nt-final-gate` | 干净检出终验层门 PASS | ✅ 已 remove |
| `…/opencode/nt-gate-ok2` | 干净检出终验层门 PASS | ✅ 已 remove |
| `…/opencode/nt-fmt` | 干净检出测 `cargo fmt` 基线 | ✅ 已 remove |

`git worktree list` 已复核无残留。

⛔ **其余 worktree 非本会话所开，一律未动**（门报「近 3h 仍有 .rs 改动 ⇒ 可能他窗在用」）：
`.worktrees/merge-b`、`.worktrees/nt-stop`、`.worktrees/nt-v2`、`.worktrees/nt-verify`、
`.worktrees/nt-v3`、`/private/tmp/nt-probe`
⇒ ⛔ **禁止手删**（AGENTS.md 记录过 850 处未提交改动被手删丢失的事故），
必须 `sh scripts/ops/nt_worktree_gate.sh prune`（内建双闸 + patch 兜底）。

### 14.2 未提交改动的去向

**本会话的改动 100% 已入库**，共 8 个 commit（清单见 §13.1），全部核实在 HEAD 祖先链。

⚠️ **他窗的未提交改动本会话一律未碰、未提交、未 `stash`、未 `git add`**。
收工时 `git status --porcelain` 仍有约 40 处改动，属他窗 WIP。

### 14.3 门状态归属（哪些红是本会话引入）

| 门 | 状态 | 归属 |
|---|---|---|
| `nt_mem_gate.sh` | ✅ rc=0 | — |
| `nt_lock_audit.py` | ✅ 可疑 0处 | — |
| `check-layer-deps.sh --strict` | ✅ **PASS: 0 new** | 本会话**修复**（原 1→2 new） |
| `cargo clippy -p nt-term-viz` | ✅ 零 warning | 本会话修复 |
| `cargo clippy -p neotrix --lib --no-deps` | 🔴 624 error | **主干既有**，非本会话引入 |
| `cargo fmt --check` | 🔴 exit=1 | **主干既有**（干净 HEAD 同值，14798 块/1884 文件） |

### 14.4 收工义务三项

- ☑ `nt_worktree_gate.sh check` 已跑（rc=0）
- ☑ 自己开的 worktree 全部 `remove`（4 个，见 §14.1）
- ☑ 本文档已写（§8 必填项 + §13 状态快照 + §14 收工自查）

---

## 15. 追加（第九轮）：clippy 线· C 路线 —— 三族 2,391 条的定性

### 15.1 ⚠️ 先纠正两条我自己报错的数字

| # | 我报过 | 真相 | 教训 |
|---|---|---|---|
| 1 | 「`neotrix` 有 **624** clippy error」 | 那是 `E0282` **编译错误**导致 clippy **中途失败**时的部分 | 编译失败时的扫描结果不是数据 |
| 2 | 「JSON 只 **626** 条，人类输出的 42,694 是重复渲染」 | ⛔ **错**。cargo 自己的汇总行明写 `due to 42693 previous error` ⇒ **42,693 是权威值**，**JSON 被 rustc 诊断上限截断** | **被截断的结构化输出同样不是数据** |

⇒ 两条同型：**任何"扫描结果"在跑完之前都不可信。**

### 15.2 P0′ 已解除

`nt_permission_profiles.rs:124` 的 `E0282`（`HashMap` 类型推断失败）已被
他窗修好（22:00:59 加了显式类型标注）⇒ `rg -c 'error\[E'` = **0**
⇒ clippy 能跑完了。

### 15.3 42,693 条的真实归因（我上轮归因错了）

| | 数量 | 占比 |
|---|---|---|
| **未配**（clippy 默认组：`style`/`pedantic`/`complexity`） | **36,701** | **86%** |
| 现配 18 条 restriction lint（`Cargo.toml:50-69`） | 5,992 | 14% |

⚠️ **我上轮说「39% 是文档/风格」严重低估**，实际 86%。
且**那 36,701 条根本没配在 `[workspace.lints.clippy]` 里**，
是 clippy 默认开启的 `style` 组被 `-D warnings` 升级 ⇒
**改 restriction lint 配置解决不了主体**。

Top 类别：`must_use_candidate` 8,916 · `doc_markdown` 5,883 ·
`uninlined_format_args` 4,511 · `indexing_slicing` 4,188 ·
`cast_precision_loss` 2,533 · `missing_errors_doc` 2,024 ·
`used_underscore_items` 1,357。

### 15.4 C 路线：三族 2,391 条全部定性完毕

| lint | 数量 | 定性 |
|---|---|---|
| `used_underscore_items` | 1,357 | `_`前缀声明「死代码」却**被使用** ⇒ 声明与现实矛盾 |
| `used_underscore_binding` | 495 | 参数 `_`前缀错误（但**混着该留前缀的**） |
| `let_underscore_must_use` | 539 | 丢弃有意义的返回值，**多数是误报**（显式 `let _ =` 本就是正确写法） |

**分布**：2,119 个文件（很分散，单文件 1–2 条）。
热点：`seal_core/self_iterating/loop_impl/core.rs`(463)、
`nt_crawl_geo.rs`(185)、`seal_loop.rs`(158)、`nt_memory_pack_chunked.rs`(150)。

### 15.5 ⭐ 最有价值的产出：证明「不能机器批量改」

`agent.rs` 同函数族**混着两种相反情况**（commit `a083834a`）：

| 参数 | 是否被用 | `_` 前缀 | 裁决 |
|---|---|---|---|
| `_role`(:232) | ✅ 用了 `_role.name` | ❌ 错 | 去掉前缀（**已修**） |
| `_task`(:239) | ❌ 没用 | ✅ 对 | **保留** |

⇒ 机器批量改必然改错一半。这条经验适用于全部 2,391 条。

### 15.6 三条真样本（供接手者直接用）

**① `used_underscore_items`** —— `l1_action/nt_act/video_quality_scorer.rs`
```rust
pub struct _FrameQuality { ... }      // :30  _ 前缀 = 声明「死代码」
let quality = _FrameQuality { ... }    // :173 却被实例化使用
```

**② `used_underscore_binding`** —— `agent.rs:232`（已修，见§15.5）

**③ `let_underscore_must_use`** —— `experience_tree/self_reflection.rs:159`
```rust
let _ = self.buffer.lock()...push(...);   // push() 返回 usize（新长度）
```

### 15.7⚠️ 另一处**作者自认**的死路径（非 clippy 可发现）

`agent.rs:237-244` `AgentTeam::execute`：
```
/// Returns an error result per agent — no real execution is wired yet.
/// Callers should not treat `success: true` as evidence of task completion.
```
⇒ 恒返回 `success: false` + `"execution not wired"`。R-P79「导出≠接入」家族。

### 15.8 剩余 clippy 线（需裁决，非执行细节）

42,693 条里 36,701 来自默认 lint 组 ⇒ 三条路：

| 方案 | 内容 | 代价 |
|---|---|---|
| **A** | 只降**已配**的 `indexing_slicing`(4,188)/`string_slice`(487)/`let_underscore_must_use`(539) 并写 reason | 治标（14%），门仍红 |
| **B** | 加 ~12 条 `allow` 把36,701 也降下来⇒ **门能真跑绿** | 等于**关掉 clippy 绝大部分默认检查** |
| **C** | 只做真信号（已执行完，见§15.4） | 已完成 |

⚠️ **B 是明确的「降标准」决策，需授权**。我倾向先 A 后 B。
⚠️ **`clippy --fix` 现在不能跑**：会重写 2,126 个文件，
其中必然混入他窗 WIP（`neotrix-neobot/*` 当前就编译不过）。

---

## 16. 收尾快照（2026-10-05 23:5x）+ 两条重要的状态变化

### 16.1 ⭐ 他窗已完成 `agent_guardrails` 接线 —— 我上轮判「需专项裁决」的那项

我在 §11 判定：接线是「1600 行移植项目」而非「加一行 `pub mod`」，
实测暴露 5 处编译错误（`E0255`×2、`compute_verdict` 类型不匹配、derive 重复），
并**还原了 `guard/mod.rs`**。他窗随后自行完成了这项工作：

| 证据 | 实测 |
|---|---|
| `guard/mod.rs:16` | 已含 `pub mod agent_guardrails;` ✅ |
| 同型 `E0255`（`input_validator.rs` 的 `use super::{…ViolationSeverity}`） | 命中 **0** ⇒ 已修 ✅ |
| `cargo check -p neotrix --lib` | ✅ **通过（21 秒）** |
| `input_validator.rs` | **+304 行**（他窗补的实现） |

⚠️ **但 `--lib` 测试目标仍编译不过**，3 个错误全在**他窗 WIP**：
- `E0583 file not found` — `nt_io_agent_loop/mod.rs:29` 声明了不存在的 `nt_loop_canary_tests`
- `unknown start of token: \` — `input_validator.rs:291` 有个转义错误 `(\&token)`
- `invalid format string: field access isn't supported` — `input_validator.rs:721` 的
  断言里 `{benign}` 后紧跟 `{r.violations:?}`，格式串不支持这种字段访问

⇒ 接手者若跑 `--lib` 测试，会看到这 3 个错误。**不是本会话引入**（本会话只改过
`policy_engine.rs`，已入库 `93405549`）。

### 16.2 ⭐ 归因再纠正：那 86% 不是「clippy 默认组」

我 §15.3 说 36,701 条（86%）来自「clippy 默认开启的 `style`/`pedantic` 组」——
**这个说法不准确**。实测：

```sh
# 最小复现（无任何 lint 配置）
$ cargo clippy -- -W clippy::uninlined_format_args
warning: variables can be used directly in the `format!` string
```
⇒ **`uninlined_format_args` 默认不开**（`pedantic` 是 opt-in 组）。

**真根因**：`neotrix-core/src/lib.rs:21`
```rust
#![warn(clippy::all, clippy::pedantic)]   // ← 仓库主动开的
```

⇒ 那 86% 来自 **`neotrix-core` 主动开启 `pedantic`**，不是 clippy 默认行为。

### 16.3 ⭐ 由此得出「不降标准」的真正含义（用户明确决策：不降标准）

仓库的标准其实**定义得很清楚且很高**：

| 行 | 配置 | 含义 |
|---|---|---|
| `lib.rs:21` | `#![warn(clippy::all, clippy::pedantic)]` | pedantic **全开** |
| `lib.rs:22` | `#![cfg_attr(not(test), deny(warnings))` | **生产构建零容忍** |
| `lib.rs:34` | `reason = "legacy allowances — tracked for removal"` | 豁免**必须带 reason 且标记待清** |
| `lib.rs:24-33` | 已有 9 条 `allow` | 有既存豁免清单 |

⚠️ **其他 4 个 crate（`nt-term-viz` / `neotrix-types` / `neotrix-neobot` /
`nt-core-capability-tree`）都没有 `clippy::pedantic`**
⇒ 它们现在实质**无 pedantic 门**。

⇒ **「降标准」=给那 36,701 条加 `allow`，直接违反 `:22` 的 `deny(warnings)`
与 `:34` 的「tracked for removal」承诺⇒ 不做。**

**正解 = 棘轮（ratchet），且仓库已有验证过的范式**：
`scripts/check-layer-deps.sh` + `scripts/layer-deps-baseline.txt`
（从 101 → 8，`PASS 0 new`）。

**照抄做 clippy**：
1. 建 `nt_clippy_ratchet.sh` + baseline（**不动代码**）——
   基线 = 当前违规的 `(lint, file)` 指纹集合，门规则 **`PASS 0 new`**：
   **新增违规即红，已有积压只减不增** ⇒ 这**不是降标准**
2. 按 lint 清账（每笔独立 commit，棘轮递减）：
   `unreadable_literal`(282, ✅`--fix`) →
   `allow_attributes_without_reason`(98, ⚠️补 reason，对齐 `:34` 格式) →
   `allow_attributes`→`#[expect]`(75, ⚠️语义) →
   `empty_line_after_doc_comment`(35) + `needless_raw_string_hashes`(30)（✅ 自动）→
   `similar_names`/`many_single_char_names`(79, ❌人判断)
3. **独立决策**：`lint.yml:40`（无 `-p`，在仓库根跑）本意应是「全 workspace lint」
   但实际几乎没跑 ⇒ 把那 4 个 crate 纳入门范围 = **提升标准**（与用户决策一致），
   代价是一次性暴露 `neotrix-types` 的 508 条 `indexing_slicing`。

### 16.4 ⛔ 建棘轮基线的前提（现在还不能建）

`neotrix-neobot/*` 当前编译不过（他窗 WIP：`nt_pet.rs`/`nt_determinism.rs`/
`nt_channel_dispatch.rs`）⇒ **现在建基线会把不完整状态冻进去**，之后还得重基线。

⇒ **等他窗编译通过再一次性建基线**（这也是本会话第4 次栽在
「未跑完的扫描不能当基线」）。

### 16.5 本会话 9 个 commit（全部已核实在 HEAD 祖先链）

| commit | 内容 |
|---|---|
| `e6386797` | 六个真缺陷 + TUI 配色集中（12 files, +596/-121） |
| `a0289cd3` | **补提交** `nt_tui_theme.rs` |
| `69d97c4d` | 恢复 14 个被注释测试 |
| `cf9e03c4` | 收敛 L1→L2 |
| `93405549` | 修 `agent_guardrails` 的 `E0255` |
| `3476e3ec` | 收敛 L1→L5 ⇒ **层门首次全绿** |
| `d71ba80e` | clippy 硬红告警 + NaN 静默退空条 |
| `6a59833a` | 吸收 break-ui：grapheme 截断 |
| `a083834a` | 修 `agent.rs` 的 `_role` 错误前缀 |

### 16.6 收尾状态

| 项 | 值 |
|---|---|
| `nt_mem_gate.sh` | rc=0 |
| `check-layer-deps.sh --strict` | ✅ **PASS: 0 new**（13 known） |
| `nt_lock_audit.py` | ✅ 可疑 0 处 |
| `cargo check -p neotrix --lib` | ✅ 通过 |
| 临时 worktree | ✅ 4 个全部已 remove，`git worktree list` 无残留 |
| 他窗未提交改动 | ⛔ 一律未碰、未提交、未 stash |
| `/private/tmp/nt-probe` | ⚠️ **8 处未提交改动不在任何提交里** ⇒ ⛔ 禁手删，必须 `prune` |
