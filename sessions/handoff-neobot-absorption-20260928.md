# Handoff · NeoBot 侧边栏/IM 吸收轮（单窗口汇总收口）

> 用途：让**一个**窗口汇总本轮所有窗口的产物并统一修复/提交。本文件是自足的 —— 读完不需要回看对话。
> 写完已重读验证落盘（项目规则 R-P16）。

## 1. 会话标识

- 窗口：`s000` 系列（DSH 侧边栏 + dsh-im 吸收轮的主窗口，连续多轮多 agent 派发）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-28 16:0x
- 内存门纪律：全程 `sh scripts/ops/nt_mem_gate.sh && CARGO_BUILD_JOBS=1 cargo …`；
  门 BLOCKED 就不起 cargo。**注意：这台 16G 机上多个窗口同时 cargo 会爆 swap。**

## 2. 目标（一句话）

把 `DSH-better-sidebar`（侧边栏文件工作台）与 `dsh-im`（IM 多渠道接入）的**架构**吸收进
`crates/neotrix-neobot` + `apps/neobot-desktop`，并让 `/stop` 从「只能说停不了」变成「桌面真能停」。

## 3. 已完成（全部经中央核验，非 agent 自述）

### 3.1 吸收的模块（新增 13 个，`crates/neotrix-neobot/src/`）

`nt_workspace`（两道 jail + 有界列举/搜索/限读）、`nt_changes`（before/after 账）、
`nt_git`（porcelain CLI + 软链接第二道门）、`nt_sidebar`（页签/查看器注册表）、
`nt_side_chat`（parent_id/origin）、`nt_channel` / `nt_channel_cmd` / `nt_channel_dispatch` /
`nt_channel_telegram` / `nt_channel_serve`、`nt_vision`（图片 side-channel）、
`nt_cancel`（StopToken）、`nt_testutil`（测试临时目录唯一化）。

桌面侧新增 44 个 IPC 命令（53 → 97），全部「有声明有注册、0 orphan」。

### 3.2 修真 bug（每条都读现场证实过，不是照单全收）

| # | 缺陷 | 危害 |
|---|---|---|
| 1 | `dedup_key` 缺 `chat` | 平台 message_id **按 chat 各自编号** ⇒ 跨聊撞键，**后一条消息被当重复静默丢弃** |
| 2 | 入站附件落 `<data_dir>/attachments`，jail 只放行 `.../workspace` | 兄弟目录 ⇒ 给模型的绝对路径必被网关拒 ⇒ **附件收了但永远读不到** |
| 3 | `poll()` 放进 `for bot in &bots`，而适配器按渠道注册、offset 全渠道共享 | **第二个及以后的机器人永远收不到消息且一声不响**（serve + 桌面两处） |
| 4 | `nt_git` pathspec 只过词法门 | 工作区内一个指向外部的软链接可让 `diff` 读到工作区之外 |
| 5 | `find` 静默截断 200 命中且无 `truncated` 位 | 用户会断定某文件**不存在** |
| 6 | `read_text` 限读分支 `truncated` 恒 false；`MAX_SEARCH_VISITS` 只 break 内层 for | 「总访问上限」名不副实 |
| 7 | `edit_of` 三段半接 | payload 不透传 + `send()` 不实现 `editMessage` |
| 8 | `LastStep` 返回 `Option<(bool,String)>` | 位置性返回；`tool` 是查询形参**不在返回值里**，极易误当第二字段名 |

### 3.3 `/stop`：桌面现在真能停

- `neobot_run_cancel` + 按 `convo_id` 索引的登记表 + RAII 注销（世代号比对，防旧轮次误删新轮次令牌）。
  **找不到在飞轮次如实报错，绝不静默成功。**
- 前端停止键从「只置 `shell.stopRequested` 让渲染跳过增量」改成**真调后端**；取消失败**可见**
  （`pushSys` + toast）并**撤销乐观隐藏**（`stopRequested` 清回 / `setRunning(true)` / `resume()`）。
- 四个取消检查点（跳边界 / 工具边界 / sleep 前 / 入轮清旗）；中止落 `TaskStatus::Cancelled`
  并带「第几跳」；公开状态返 `Waiting`（人可接手），**不新增 `TurnStatus` 变体**。
- IM 侧管道接通（登记/递令牌/翻旗），并删掉「所有调用方一律传 `None`」的 `stop_flag` 死形参。
- **变异实证**：把 `slot.token.cancel()` 改成不翻 → 关键测试 FAILED 且**挂住 60 秒**（轮次永不停止、
  假模型闸不放行）。证明该断言非恒真。

### 3.4 两个跨批次的隐形缺陷

1. **测试 flake 源（一直存在）**：30 处测试用 `std::env::temp_dir().join(固定名)` 造夹具
   （19 处完全固定名，11 处只按用例名唯一化 ⇒ 同一用例跑两遍仍撞），并发跑互相 `remove_dir_all`
   擦除对方夹具。**实测 3 实例并发各挂 9–18 个、且每份挂的不是同一批**；串行 353 全绿。
   已加 `nt_testutil::temp_dir(tag)`（pid + 单调纳秒）并机械替换 29 处；
   **对照实验 4 实例 × 353 → 0 失败**。
   ⚠️ 这条对我方影响很大：常态多 agent 同时 `cargo test` 同一 crate
   （**cargo 只串行化构建，两个 `cargo test` 的执行阶段是并行的**）。

2. **`.gitignore` 裸 `tests/` 静默丢弃集成测试**：git 裸模式**匹配任意层级**，
   `neotrix-core/tests` 26 个 .rs **只入库 2 个**，`apps/neobot-desktop/tests` 6 个 **入库 0 个**
   —— 新 clone 等于没有这套网。已锚成 `/tests/`（**这一笔已提交**：`70356116`）。

### 3.5 验证基线（交接时的权威数字）

```
cargo test -p neotrix-neobot --lib     → 360 passed / 0 failed
cargo test -p neobot-desktop            → 9 个测试二进制 / 71 passed / 0 failed
npm run typecheck / selftest            → 干净 / 自测全过
npm run build                          → 成功
python3 scripts/ops/nt_lock_audit.py    → neobot 0 / neotrix-core 0 / desktop 0
python3 scripts/ops/nt_ipc_keys.py      → 声明 98 / 注册 98 / 键名错配 0
python3 scripts/ops/nt_docclaims.py     → 3 error（其中 1 真 2 误报，已修真的那条）
```

## 4. 正在改的文件（关键 —— 逐个列）

### 4.1 本会话**尚未入库**的 54 个路径

| 区域 | 路径数 | 是否可独立提交 |
|---|---|---|
| `crates/neotrix-neobot/**` | 31 | **是**，但会触发 P0 门（见 §6） |
| `apps/neobot-desktop/**` | 14 | **是**，同上 |
| `scripts/ops/nt_ipc_keys.py`、`nt_docclaims.py`、`nt_smoke.sh` | 3 | 是（纯 python，不触发 P0） |
| `AGENTS.md`/`DOCUMENTATION-MAP.md`/`RUST-STANDARDS.md`/`TODO.md` | 4 | 是 |
| `docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`、`DESIGN-CHANNEL-DISPATCH.md` | 2 | 是 |

### 4.2 **不属于本会话**的 906 个路径（绝对不要顺手 commit 进去）

`neotrix-core`（393）· `src-tauri`（246）· `skills`（78，**含 75 个删除**）· `games`（38，**全删**）·
`docs` 另 87 · `scripts/ops` 另 31 · `sessions`（14）· `thirdparty`/`evals`/`deny.toml`。

⚠️ 那 **185 个删除**（skills 75 / src-tauri 67 / games 38）来自更早的清理轮，**本窗口未核验**。
提交前逐条确认，别把别人的删除卷进来。

### 4.3 我改过一行但**故意没提交**的

`neotrix-core/src/neotrix/nt_crystal_core/nt_premise_selector.rs:192`
（`默认 cap = 5` → `= 3`，该常量今日被降到 3 而这处漏改）。它属于 §4.2 那棵未跟踪树，
会随那棵树将来被提交时一起走。

## 5. 下一步（按优先级）

1. **统一提交本会话那 54 个路径**（`git add` 精确列出，**不要 `git add -A`**）。
   建议拆 4 笔：`fix(gitignore)` 已提交 → `feat(neobot)`（代码+测试）→
   `feat(ops)`（3 个工具）→ `docs`。
2. **拍板 `/stop` 回执措辞**（见 §6 阻塞 2）—— 唯一需要产品决策的项。
3. 跑通 `scripts/ops/nt_smoke.sh` 的**全部 6 步编排**（各步已手工跑绿，**编排本身从未执行过**）。
4. `edit_of` 真正生效（需发占位消息并落库**它自己的** message_id）。
5. IM 的 `/stop` 兑现（并发化调度，第二个执行流 —— 设计见 `DESIGN-CHANNEL-DISPATCH.md` §11/§13）。
6. 其余零星债见 `TODO.md`（位置性返回、help 反引号、`nt_docclaims` 过拟合验证等）。

## 6. 阻塞点

**阻塞 1（提交失败，我撞到了，下一窗口会再撞）**
`git add <80 个文件>` 后 `git diff --cached` 确实显示 80 files，但**紧接着的 `git commit`
报「no changes added to commit」** —— 暂存区在这两步之间被**清空**。仓库有多个活跃窗口，
`git log` 可见别的窗口在跑 `git worktree add` 等 git 操作。
→ **对策**：`git add` 与 `git commit` 必须写在**同一条命令**里（`git add … && git commit …`），
不要分两次调用；提交后立刻 `git log --oneline -1` 确认。

**阻塞 2（P0 门会拦 Rust 提交）**
`.githooks/pre-commit` 对**任何** `.rs` 变更跑 `cargo check --tests -p neotrix -j4`
（`core.hooksPath=.githooks`）。而 `neotrix-core` **正被另一扇窗口大改**（393 个路径），
所以：① 现在很可能是**红的** ⇒ 我的提交会被门拦下；② 即使绿了，**这个门验的是 neotrix-core，
不是 neobot**，用它给 neobot 的提交背书是不成立的。
→ **对策**：先实测 `cargo check --tests -p neotrix -j4` 再决定。
**不要用 `--no-verify`**（hook 自己写着「会破坏 P0」）；要么等 neotrix-core 绿，要么先只提交
纯 python/文档那几笔（不触发门），把 Rust 提交单独处理并在 commit message 里**如实写明
本提交未经 P0 门验证**。

**阻塞 3（`/stop` 回执现在是假的，待产品拍板）**
core 共享回执仍写「**停不了**：跑轮是同步的…（桌面 App 的停止键也只收起输出、不终止运行；
要真的终止得退出那个 App）」。这在 **IM 侧字面为真**，但**桌面侧已假**（桌面现在能停）。
牵连 4 条测试契约：`nt_agent.rs:2316`、`nt_smoke_slash_cmd.rs:265/437/673`；
另有 `nt_agent.rs:2283-2291` 那条反向测试**前提已过期**（它假设「取消尚未接线」）。
推荐措辞（对两端都成立的**条件句**）已写在 `TODO.md` 顶部批次块里。
**为什么不代劳**：这是用户可见的产品措辞，且错误方向是**低报**（说做不到而实际做得到），
危害小于高报，但该有人拍板；且它与 4 条「反撒谎」契约互锁。

## 7. 给接手会话的话

**恢复命令**
```sh
cd /Users/neo/Downloads/neotrix && git branch --show-current   # feat/capability-absorb-20260828
git log --oneline -3            # 顶部应是 70356116 fix(gitignore)
git status --porcelain | wc -l  # 约 960 —— 大部分**不是**本会话的
sh scripts/ops/nt_mem_gate.sh    # 重型 cargo 前必查，非 0 不要起
```

**禁止事项**
- 禁止 `git add -A` / `git add .`（会卷进 906 个他人路径 + 185 个未核验的删除）。
- 禁止在内存门 BLOCKED 时起 cargo；禁止 `--all-targets`；禁止设 `CARGO_TARGET_DIR`
  （会造成全量重复编译，内存磁盘双爆）。
- 禁止 `--no-verify` 绕过 P0 门。
- 派多 agent 时**文件所有权必须互斥**：`nt_channel_serve.rs` / `nt_channel_dispatch.rs` /
  `nt_agent.rs` / `nt_cmd_run.rs` 是高频冲突点（历史上出过三次覆盖事故）。
- **下刀前查并发**：`git status --porcelain <file>` + `stat -f "%Sm"`，
  mtime 数秒内变过 = 别人在写，换文件或先通报。

**风险提示**
- 与其他窗口同改的文件：`neotrix-core/**`（另一窗口大改中）、`src-tauri/**`、
  `scripts/ops/nt_locate.py`、`skills/**`。
- 我方独占（本会话内）：`crates/neotrix-neobot/**`、`apps/neobot-desktop/**`。
- **读退出码别经管道**：`cmd | head -1; echo $?` 拿到的是 `head` 的退出码。
  本会话因此**两次**误判内存门（一次以为门开了导致 build 没跑、一次以为门关了）。
  正确写法：`sh nt_mem_gate.sh >/tmp/g; rc=$?`。

## 8. 经验吸收落盘记录（experience-tree 协议）

- **已落盘 16 条**到 `kv_store(experience, …)`：`defect 4 / insight 6 / pattern 3 / rule 3`，
  主题集中在「测试网隐形失效」「反撒谎注释与回执」「agent 文件所有权」「shell/git 纪律」
  「取消三态」「私有安全原语」。
- ⚠️ **协议偏差（须知）**：`neotrix-experience absorb` **忽略了我传入的 `session_id` 与 `cycle`**，
  自行生成 `sess_<ts>_<hex>` / `cycle=unknown`，所以我的分支 key 是 `branch_unknown_*`，
  `list --cycle 200` 查不到本轮。下次若要按 cycle 核对落盘，**不能信 `--cycle` 参数**，
  改用特征短语 `query` 核对。
- ⚠️ **route_table 有 2 条不精确路由**：`route` 是**追加**语义且 CLI 无撤销命令，
  我为「测试临时目录并发 flake」「gitignore 集成测试丢失」注册的关键词，
  首条被模糊匹配到了一个**旧的、内容相近但不同**的分支（`branch_1144_3_15eaf9` /
  `branch_audit0927_0_bfa3a1`），我的正确分支排在第二位。
  `route-verify` 报 0 ghost（两者都真实存在），`--clean` 删不掉。
  协议禁止手改 `kv_store`，故**原样保留**；影响仅是搜这两个词会多返回一条相邻主题的旧分支。
  正确分支：`branch_unknown_0_31a0c9`（临时目录 flake）/ `branch_unknown_1_401fa2`（gitignore）。
- 我**直接执行了 absorb**（未走 `~/.neotrix/pending-absorb.json`），并已删除该 pending 文件 ——
  否则后台循环会再吸一遍，而幂等门禁按 `session_id` 判重，**拦不住**（CLI 已换过 id）。
