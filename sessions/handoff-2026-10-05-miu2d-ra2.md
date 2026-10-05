# handoff — 2026-10-05 — miu2d / rust-alert 吸收轮（确定性纪律 + 涌现指纹）

## 1. 会话标识

- 窗口：opencode 会话（无 tty 标识）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-10-05 10:1x
- 吸收轮文档：`docs/architecture/ABSORPTION-MIU2D-RA2-2026-10-05.md`

## 2. 目标（一句话）

> 深度吸收 `luckyyyyy/miu2d`（MIT，2D ARPG 引擎）与 `rust-alert` 组织下全部非 fork Rust 仓
> （主源 `ra2.exe`，Apache-2.0）的可迁移能力，补齐 neotrix 的对应能力；用户中途追加
> 「意识涌现，全域产品目标」⇒ 落点选在**涌现可判定性**上。

## 3. 已完成

- [x] **前提修正**：`rust-alert` 是组织主页（15 仓），无同名仓库 ⇒ 问用户确认范围，用户选「全部非 fork Rust 仓」。
- [x] **miu2d 全文精读** 3 个 Rust 核（`pathfinder.rs` 1097 / `ai_search.rs` 663 / `collision.rs` 379 行）+
      7 子系统代理逐文件调研（带 `file:line`）。
- [x] **ra2.exe 逐文件调研** `projects/engine/` 10 crate / 82,766 行：确定性纪律、状态绑定、
      `ra-net` lockstep、命令校验、空间/寻路/游戏性。
- [x] **⭐ 落地第一批：3 处 LIVE 非确定性修复**（LLM 网关 / 缓存热路径，全部有仓内已修兄弟作范本）：
      - `neotrix-core/src/l0_substrate/nt_core_cache.rs` — `set_exact` 淘汰受害者由**任意**改为
        按 `inserted_at` 最旧（FIFO）+ key 兜底；`get_semantic` 同分加 key 升序兜底 +
        `best_sim >= 1.0` 提前退出（终止证书抄 miu2d `ai_search.rs`）。
      - `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/selection.rs` —
        `build_candidate_chain` 补 `.then_with(名字升序)`（同文件 `select_best` 已有该范式）。
        危害到顶：`resolve_default_model_sync` 取 `chain.first()` ⇒ **默认模型**曾随 HashMap 序漂移。
- [x] **⭐ 落地第二批：新建** `crates/neotrix-neobot/src/nt_determinism.rs`
      （无状态可寻址采样 / 无别名摘要 `Digest` / `sorted_keys`；零新依赖）。
- [x] **⭐ 落地第三批：涌现指纹** — `nt_capability_registry::capability_digest() -> Result<u64,String>`
      （生产函数），被 `consciousness_runtime.rs` 运行期探针消费并输出 `digest`。
- [x] **验证**：`nt_core_cache` 22/22 · `build_candidate_chain_tie` 1/1 · `nt_determinism` 11/11 ·
      涌现指纹 2/2 · 运行期探针 1/1，0 编译错误。
- [x] **来源登记**：`absorption-sources/repos.csv` +2 行；`LICENSES.md` 新增「第五批」含 8 仓只读设计裁决。
- [x] **门**：死锁审计 0 处（rc=0，2026-10-05 10:00 核实）。

## 4. 正在改的文件（逐个列）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| `neotrix-core/src/l0_substrate/nt_core_cache.rs` | 2 处缺陷修复 + 2 项回归测试，**已编译已测（22/22）** | 是 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/selection.rs` | 1 处缺陷修复（1 行 tie-break + 注释），**已测（随 [2] 覆盖）** | 是 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/mod.rs` | 新增 1 项测试（`test_build_candidate_chain_tie_is_name_ascending`），**已测（1/1）** | 是 |
| `crates/neotrix-neobot/src/nt_determinism.rs` | **新建**，11 项测试全绿 | 是（新文件，注意 `git add` 需显式路径） |
| `crates/neotrix-neobot/src/nt_capability_registry.rs` | 新增 `capability_digest()` 生产函数 + `use` + 2 项测试 | 是 |
| `neotrix-core/src/l5_cognition/nt_core_consciousness/consciousness_runtime.rs` | 探针输出新增 `digest` 字段（**测试内**，非生产路径） | 是 |
| `docs/architecture/ABSORPTION-MIU2D-RA2-2026-10-05.md` | 新建吸收文档 | 是 |
| `docs/architecture/absorption-sources/{repos.csv,LICENSES.md}` | 新增来源与许可裁决 | 是 |
| `sessions/handoff-2026-10-05-miu2d-ra2.md` | 本文件 | 是 |

⛔ **与本会话无关的他人改动（同树共存，勿混入提交）**：
`neotrix-core/src/l1_action/nt_io_mention.rs`、`l6_meta/nt_meta/eval_engine/llm_judge.rs`、
`l5_cognition/nt_mind/nt_mind/seal_core/core/mod.rs`、`l1_action/nt_tui_app.rs`、`entry/headless.rs`、
`scripts/ops/neobot-check-emergence.mjs`、`.neotrix/capability_{registry,overrides}.json`、`Cargo.lock`。

## 5. 下一步（按优先级）

1. ⛔ **最高优先 —— 治理漏洞，非优化**：
   `crates/neotrix-multi-agent/src/god_agent.rs` 的 `AgentSelectionConfig` **4 个字段里 3 个从未被读**
   （`prefer_specialists` / `max_cost_per_decision` / `fallback_agent_id`；只有 `min_autonomy` 被消费）。
   ⇒ **`max_cost_per_decision` 是空闸**：一个 `cost_budget` 超限的 agent 仍会被选中。
   同文件的 `GodAgent::route` 目前 0 生产调用者，所以**先定「接线还是判死」，再修**。
2. **接线或判死** `l3_embodiment/nt_flow::{astar,build_flow}` 与 `nt_astar::AStar`（ROUND25 遗留），
   并统一两套启发度量（同层一个曼哈顿、一个 octile，且都 0 消费者 ⇒ 改动面小）。
3. **接线前先修** `…/gateway/routing/subgrid.rs` 的 `select_best_for_profile`
   —— 它目前只因 `CapabilityCoordinator::coordinate` 无生产调用者而未爆，
   但**缺**同文件 `select_best` 已有的 name tie-break。
4. **合并性能项（本轮刻意未动）**：`build_candidate_chain` 仍是「全量 `sort_by` + O(n·m) `chain.contains`
   去重环 + 只取前 `limit`」⇒ 单次 `min_by`/堆可降复杂度。
5. **摘要覆盖面的机械保障**：给关键子系统上指纹时，需解决「加字段漏摘不被发现」
   （`ra2.exe` 缺的正是这个，且它自己记录了）。

## 6. 阻塞点

- **无硬阻塞。**
- ⚠️ 软阻塞：`check-layer-deps.sh --strict` 在当前 HEAD **rc=1**（14 sites / 13 baseline → 1 new），
  违规文件 `neotrix-core/src/l1_action/nt_model_cli.rs`（l1→l2），最后提交 `ed5992c1`（2026-10-03）。
  **非本会话引入**（该文件 `git diff HEAD` 为空，且不在本会话改动清单内）。
  ⚠️ 脏树下计数不可直接与基线比对（AGENTS.md §4.2），要判定须用
  `git worktree add --detach HEAD` 的干净检出。
- ⚠️ **构建锁排队严重**：实测同时有 3–4 个他窗 `cargo test -p neotrix --lib` 跑在同一 target 目录
  （其中只有一扇窗正确用了 `scripts/ops/nt_build_lock.sh`）。曾因此把我的一次验证超时。

## 7. 给接手会话的话

- 恢复命令：先读本文件 + `docs/architecture/ABSORPTION-MIU2D-RA2-2026-10-05.md`，
  再跑 `git status --short` / `git diff --stat` 核对；patch 兜底在
  `.neotrix/patches/2026-10-05-miu2d-ra2-determinism.patch`（769 行 / 6 文件，
  `git apply --check --reverse` 已验证可回放）。
- ⛔ **提交必须 `git commit --only <显式路径>`**（共享 index；§4 表里列了本会话该提交的文件）。
- ⛔ **禁止多窗并发全量构建**。一律 `bash scripts/ops/nt_build_lock.sh --strict -- <cmd>`
  （该脚本只动 `$TMPDIR` 锁目录，无仓库写操作，已审）。
- ⚠️ **不要在共享树上做「回退-verify」表演**（本会话已犯，会污染他窗正在编译的代码）。
  要证明判别力，用 `/tmp` 里 `rustc` 单独编译的旁路程序。
- ⚠️ **两条被实测推翻的手推，别再照抄**：
  (a) 语义层 tie-break 测试的误通过率**不是** `(1/n)^rounds`（实测 0.033，非 1e-27）——
      同键集下 SipHash 相对序**强相关**，非均匀随机置换；
  (b) `CapabilityRegistry::nodes` 是 **`IndexMap`**（插入序），遍历本就确定 ——
      「必须排序否则漂移」在该处**不成立**。
- 风险提示：本会话与至少 3 个他窗共用主树，**无文件重叠**，但门的结果受他窗 WIP 影响。

## 8. 收工自查（必填）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出（rc=0）：

```
[worktree-gate] ℹ️  **主树**：21 处未提交 | target 167755M（**只报告，不影响退出码**）
[worktree-gate] worktree=2 个 | 合计 116M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
```

- **本会话未新建任何 worktree。**
- `.worktrees/merge-b`（3 处脏改动）与 `.worktrees/nt-stop`（干净）**均为他窗所建，本会话未触碰**。
  ⛔ 不代为 prune（那是他人未提交工作，AGENTS.md §1 明令「带脏文件的 worktree 只删生成物」）。

### 8.2 未提交改动的去向

⛔ 本会话**未执行任何 `git commit`**（用户未要求）。按模板要求给出**合法去向**：**patch 兜底**。

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `neotrix-core/src/l0_substrate/nt_core_cache.rs` | 2 处确定性修复 + 2 项回归测试 | ☑ patch 兜底 |
| `…/gateway/routing/selection.rs` | `build_candidate_chain` tie-break | ☑ patch 兜底 |
| `…/gateway/mod.rs` | tie-break 回归测试 | ☑ patch 兜底 |
| `crates/neotrix-neobot/src/nt_determinism.rs` | 新建：确定性三件套 + 11 测试 | ☑ patch 兜底 |
| `crates/neotrix-neobot/src/nt_capability_registry.rs` | `capability_digest()` + 2 测试 | ☑ patch 兜底 |
| `…/nt_core_consciousness/consciousness_runtime.rs` | 探针输出 `digest` | ☑ patch 兜底 |
| `docs/architecture/ABSORPTION-MIU2D-RA2-2026-10-05.md` | 吸收文档 | ☑ 本文件树内新增（git 已可见） |
| `docs/architecture/absorption-sources/{repos.csv,LICENSES.md}` | 来源 + 许可裁决 | ☑ 同上 |
| `sessions/handoff-2026-10-05-miu2d-ra2.md` | 本交接 | ☑ 同上 |

**patch 单一入口**：`.neotrix/patches/2026-10-05-miu2d-ra2-determinism.patch`
（769 行，覆盖上述前 6 个代码文件；已 `git apply --check --reverse` 验证与工作树一致）。
⚠️ 三份 `docs/` 文件是**新增未跟踪**文件，**不在**该 patch 内（patch 只覆盖 tracked diff + 1 个新代码文件）
⇒ 提交时需 `git add docs/architecture/ABSORPTION-MIU2D-RA2-2026-10-05.md docs/architecture/absorption-sources/ sessions/handoff-2026-10-05-miu2d-ra2.md`。
⚠️ `.neotrix/patches/` 是否 gitignored 需接手者确认；若被忽略则另寻兜底。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**0**
- 提交前是否跑过验证：**☐ 是** —— 跑了，但**未**跑全量 `cargo xl`。已跑（经构建锁串行）：
  `cargo test -p neotrix --lib nt_core_cache`（22/22）、
  `cargo test -p neotrix --lib build_candidate_chain_tie`（1/1）、
  `cargo test -p neotrix-neobot --lib nt_determinism`（11/11）、
  `cargo test -p neotrix-neobot --lib 涌现`（2/2）、
  `cargo test -p neotrix --lib 运行期探针`（1/1）。
  ⛔ **全量 `cargo test -p neotrix --lib`（13,000+ 测试）本会话未跑** —— 因 3–4 个他窗并发占满 target 目录。
    **接手者请在独占窗口补跑全量**，本会话只保证上述 5 个定向集绿。
- 门红归属：
  - `check-layer-deps.sh --strict` rc=1（1 new，`l1_action/nt_model_cli.rs`）⇒ **他窗/先前提交**（`ed5992c1`，2026-10-03），非本会话。
  - `check-truth-surface.sh` rc=0（仅 `UNCOMMITTED_DEP nt_determinism.rs`，本会话新文件未提交所致）。
  - `nt_lock_audit.py neotrix-core/src` **0 处**（rc=0，2026-10-05 10:00）—— 改 `.rs` 后已重跑。
