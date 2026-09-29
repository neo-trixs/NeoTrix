# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-09-29（本轮收尾）
>
> ## ✅ 本轮完成（2026-09-28/29 · 桌面端统一 + 缺口闭合）
> >
> > **桌面端统一到 `~/Downloads/Neo/neobot`**（独立 2 成员 workspace，零 neotrix-core 依赖）：
> > `d5413335` 移除本仓 `apps/neobot-desktop`，本仓只留 `crates/neotrix-neobot` 作库
> > （`nt_llama` 为 CLI 与桌面端共用的唯一本地推理实现）。
> >
> > **命令面 13 个全接入 UI**：llamacpp 6（设置面板）+ web/vision 7（侧边栏检索页签 +
> > 图片查看器 + 端点代理提示）。111 命令 ↔ 111 类型条目，双向零缺口。
> > **16 道防回归门**（`registration_tests`，全部 mutation-verified）；
> > **483 测试全绿**（含 5 条对活 llama-server 的 E2E）。
> > 版本 `v0.22.0` 已打 tag，bare + bundle 双备份已同步。
> >
> > **白屏根因找到并修复**：`tauri.conf.json` 配了 `devUrl` 则 release 内嵌零资源
> > （tauri-codegen 2.7.0:178，`dev && dev_url` 分支；二进制大小对照法验证）。
> > 修法：devUrl 置 null。另补 9 个缺失元素（一个缺失的 id 曾把整个应用打瞎，
> > 侧边栏 1189 行从未挂载过）+ grid 缺失两态 + 标题居中。
> >
> > **权重账本闭合**：`[R]` 5 个逐字节锁定 repo@revision（LFS oid / git blob sha1）；
> > `[I]` 55 个备份 57M，本地 + `/Volumes/NeoTrixBrain` 跨盘各一份（shasum -c OK）；
> > 清单 60 项双向零缺口。工具 `scripts/ops/nt_weights_manifest.sh`。
> >
> > **CI 修三处旧债**：删每 PR 必红的 `desktop-e2e.yml`；`release.yml` 摘
> > `build-desktop`；修 `ci.yml` 已提交的 YAML 语法错（整个文件无法加载）。
> > **7 个正典文档**指向已删路径已修正（活指示改、历史加注记不重写）。
> >
> > **经验沉淀**：`docs/architecture/LESSONS-20260928-blank-window-and-embedding.md`
> > （L1-L5 + 未闭合清单）。
> >
> > **诚实未闭合**：`stop`/真 `swap` 未执行（`#[ignore]` 占位，三重安全门已验证，
> > 需内存宽裕时手动跑）；`thread.ts` 附件图片仍直载（只补 onerror）；
> > `dev` 在 release 恒真的根本原因未查明；跨盘后 training/ 新增需重拷。
>
> ## 🔖 剩余任务总入口（2026-09-28 收口 · 交给**单一汇总窗口**执行）
>
> ---
>
> 智能同步生成，最后更新：2026-09-29（B-2 窗口追加）

> ## 🔖 剩余任务总入口（2026-09-29 · B-2 Noise / 分层棘轮窗口 · **最新**）
>
> **交接件**：`sessions/handoff-20260928-ratchet-final.md`（棘轮+B-1）、
> `sessions/handoff-20260928-merge-readiness.md`（合并探测）、
> `docs/architecture/B2-NOISE-IK-RESOLUTION-20260928.md`（B-2 根因与落地证据）
>
> **当前状态一句话**：分层棘轮 **102 → 8**、`PASS 0 new`；B-1 双时间已落地；
> **B-2 Noise IKpsk2 已全绿** —— 协议修正经官方向量 4/4 逐字节验证，
> 且 2026-09-29 完成编译验证：`12175 passed / 0 failed`、官方向量验收测试转绿、
> `ios-bridge` 0 error、分层门 `PASS 0 new`。**剩提交与收工。**

### 本窗口剩余任务

| # | 任务 | 类型 | 阻塞 |
|---|---|---|---|
| ~~1~~ | ~~cargo 验证 B-2（官方向量验收测试）~~ | 验证 | ✅ **已完成**（2026-09-29）：`full_handshake_matches_official_vectors ... ok` |
| ~~2~~ | ~~串行全量验证链~~ | 验证 | ✅ **已完成**：`check --tests` **0 error**；`lib` **12175 passed / 0 failed / 41 ignored**；`--features ios-bridge` **Finished 0 error**；`check-layer-deps.sh --strict` **PASS 0 new / 8 known / RC=0** |
| ~~5~~ | ~~更新 `DECISIONS-2026-09-28.md` 的 B-2 状态~~ | 文档 | ✅ **已完成**（含订正「`es` 角色接反」这个被证伪的首因诊断） |
| ~~3~~ | ~~提交 B-2 重写 + 文档~~ | 提交 | ✅ **已完成**：`a9d00624`（代码）+ `54e48f2e`（文档），P0 门通过，未用 `--no-verify` |
| ~~4~~ | ~~整合 `f_merged_ratchet` 到干净分支~~ | 集成 | ✅ **已完成**：`f_merged_ratchet@64838ca3` |
| ~~8~~ | ~~把 `f_integrated` 快进进主干~~ | 集成 | ✅ **已完成**（2026-09-29）：主干 `feat/capability-absorb-20260828` 已快进到 `df0273e2`。分层门 **PASS 0 new / 8 known**、全量 **12194 passed / 0 failed**、他窗 59 个脏文件 **0 个被波及**。明细见 `sessions/handoff-20260929-b2-noise-final.md` §9.4 |
| ~~9~~ | ~~收掉本会话的 2 处 worktree~~ | 收工 | ✅ **已完成**（2026-09-29）：根因不在时机、在门的判据 —— mtime 启发式分不清「他窗在写」与「我刚做完」。已修为「脏才看 mtime」（干净+已并入分支 ⇒ 可证无损），并**造真实反例自测**确认脏 worktree 保护未放松。3 支冗余分支（`f_merged_ratchet`/`f_integrated`/`fix/bitemporal-and-layer-ratchet`）已用 `git branch -d` 删除 |
| ~~10~~ | ~~`noise_handshake` 接到生产（R-P79）~~ | 实现 | ✅ **已完成**（2026-09-29）：新增 `protocol/noise_ik.rs` —— C1 SANS-IO 协议引擎，作为 `noise_handshake` 的唯一生产消费者。分层方向合法（C1→C0），`noise_ik` 10 测试全绿，全量 **12207 passed / 0 failed**。⚠️ 接线时测试抓到我自己写死的错误断言（见下），已改为断言正确性质 |
| 6 | 经验吸收：`neotrix-experience absorb` 把 L23–L26 入 KB | 收尾 | **待办** |
| 7 | 收工：`nt_worktree_gate.sh check` → 自己开的 worktree 走 `prune`（**禁手删**） | 收工 | **待办**（硬规则） |

### 本窗口已完成（勿重复做）

- B-1 真双时间（`71e1c412`）：`resolve_chain_root`/`nodes_as_of`/`node_history`，无 schema 迁移。
- 分层棘轮 102→8，13 笔 commit；`f_merged_ratchet@eb9373a4` 含 B-1+棘轮+合并。
- `seal_pipeline.rs` feature-gated E0433 修复；`nt_scan_surface.py` 的 `target` 误报修复。
- B-2 向量落地 + 证据测试（`3a198596`）、L20–L22（`ee32e74e`）。
- B-2 **6 处协议修正**（未编译验证，见 `B2-NOISE-IK-RESOLUTION-20260928.md` §7）：
  `h`/`ck` 分离、AEAD 传 AD=`h`、`MixKeyAndHash` 改三路 HKDF、
  `Split` 的 `zerolen` 改空切片、responder `se` 改 `DH(e_r,s_i)`、
  向量测试明文改 `yellowsubmarine`。
- 附带：空 prologue `MixHash(&[])`、PSK `e` token 绑定、禁 `expect`、精确长度校验、
  X25519 小阶点全零共享秘密防护。
- `noise_handshake` 在 B-2 提交 `a9d00624` 时点全仓**零生产消费者**（当时仅 `crypto/mod.rs:14` 声明 + 自身测试）⇒ 该笔改动无生产敞口。**该状态已于同日被 `protocol/noise_ik.rs`（`1c80d74a`）改变**，现已有生产消费者。

| 11 | **⚠️ 需通报另一窗口**：我误删了 `scripts/ops/nt_evolution_exp.py`（339 行） | 事故 | ✅ 事故记录成立（`55373387` 恢复、提交纪律已修）。⛔ **2026-09-29 更新**：该 Python 版**已被 `2f11389f` 有意删除** ——「统一为一套，Rust bin 成唯一判决实现」。真实入口是 `neotrix-core/src/bin/nt_evolution_exp.rs`。本行保留作事故取证，勿据其复活 Python 版 |

> ⚠️ **本窗口的教训**：协议正确性先有可执行证据（官方向量 4/4 逐字节 + 第三方实现复现），
> **后**才有编译验证 —— 两者缺一不可。首轮「手推 2 条修正」自洽但不足，
> 逐字节比对后又挖出 5 个缺陷。教训见 `LESSONS-…-consumer-audit.md` L23–L26。

> ## 🔖 剩余任务总入口（2026-09-28 收口 · DSH/neobot 窗口）
>
> **完整交接件**：`sessions/handoff-neobot-absorption-20260928.md` —— ⛔ **该文件从未入库**
> （2026-09-29 核实 `git log --all` 零命中，磁盘上也没有）。**「先读它再动手」已不可执行。**
> 恢复途径（2026-09-29 三处逐一核实，**全部落空**）：
> (a) `/tmp/nt-backup-20260928` —— **目录已不存在**（`/tmp` 被系统清理）；
> (b) `git stash@{0}`（preflight 快照，759 文件）—— **不含本文件**（`git stash show --name-only` 零命中）；
> (c) `git log --all` —— 该路径**从未入库**，无历史可追。
> ⇒ **结论：本交接件已永久丢失。** 基线只能从下方「当前状态」与
> `docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md` 反推。这是
> 「交接件未入库即永久丢失」的又一个实例 —— 交接义务里的「写文件」不等于「入库」。
>
> **当前状态一句话**：DSH 侧边栏 + dsh-im 吸收完成，**桌面 `/stop` 已真能停**，
> core 360 绿 / 桌面 71 绿。剩的是**产品决策 1 项** + **提交 1 件事** + **收尾 3 项**。
>
> **⚠️ 提交前必读**：本轮 961 个改动路径里**只有 54 个是本会话的**，
> 其余含 185 个**未核验的删除**。**禁止 `git add -A`**。
> 已提交：`70356116 fix(gitignore)`。**未提交**：代码 45 + 工具 3 + 文档 6。
> 提交必须在**同一条命令**里 `git add … && git commit …`（本轮曾被别的窗口清空暂存区）。
> 提交 `.rs` 会触发 P0 门 `cargo check --tests -p neotrix`（验的是 neotrix-core，**不是** neobot）。

### 剩余任务（按优先级）

| # | 任务 | 类型 | 阻塞 |
|---|---|---|---|
| 1 | **统一提交本会话 54 个路径**（拆 4 笔：已提交 gitignore → feat(neobot) → feat(ops) → docs） | 提交 | P0 门；别 `-A` |
| 2 | **`/stop` 回执措辞拍板** —— 现在对桌面是假的（说「停不了」而它能停），对 IM 仍真。牵连 4 条反撒谎测试契约 | **产品决策** | 需人拍板 |
| 3 | `nt_agent.rs:2283` 那条反向测试**前提已过期**（它假设「取消尚未接线」，现已接线） | 与 #2 同批 | 随 #2 |
| 4 | `help_text` 里 `/stop` 的「（当前不可用：跑轮同步…）」重估 —— 现在只对 IM 成立 | 与 #2 同批 | 随 #2 |
| 5 | **跑通 `nt_smoke.sh` 全部 6 步编排**（各步已手工跑绿，编排本身从未执行过） | 收尾 | 内存门 |
| 6 | **`edit_of` 真正生效** —— 需发占位消息并落库**它自己的** message_id（现生产可达的 `sweep_pending` 传的是用户消息 id，bot 不能编辑） | 实现 | — |
| 7 | **IM 的 `/stop` 兑现** —— 缺第二个执行流（worker 池/async）。⚠️ 原注「设计见 `DESIGN-CHANNEL-DISPATCH.md` §11/§13」——**该文档从未入库**（2026-09-29 核实 `git log --all` 零命中）⇒ 需重写设计或直接从代码反推。**本条在设计缺失下无法开工** | 架构级 | — |
| 8 | `per-bot token_env / model` 生效 —— 需先加 chat→bot 绑定字段（一渠道一适配器是当前限制） | 实现 | — |
| 9 | `nt_store` 下剩余位置性返回：`nt_store_changes.rs:228`、`mod.rs:235`（`export_counts` 三元组） | 清理 | 调用点需授权 |
| 10 | `nt_docclaims.py` 的「`「」`引号即讨论句」豁免**有过拟合风险**（建立在我给的 4 个样本恰好都符合上），需更多真实样本验 | 验证 | — |
| 11 | 前端 invoke 的**类型层**（比静态核对更强：让键名错在编译期而非靠 `nt_ipc_keys.py` 事后抓） | 增强 | — |
| 12 | Telegram **真实平台**联调（鉴权/限流/长轮询；当前只有本地 fake server） | 联调 | 需真 token |
| 13 | 其余 906 路径（neotrix-core 393 / src-tauri 246 / skills 78 / games 38 …）**不属本会话**，含 185 个未核验删除 | 归主 | — |

> 智能同步生成，最后更新：2026-09-28（人工重建）

> **2026-09-28 DSH 吸收轮 · 收口复核（4 agent 并行产出 + 主 agent 独立核验）**
> 验证（`CARGO_BUILD_JOBS=2` 串行，内存门 OPEN 时跑）：
> `neotrix-neobot --lib` **308 绿 / 0 红**；`neobot-desktop` **4+4 注册 + 38 IPC 冒烟 绿**；
> `cargo check -p neobot-desktop --all-targets` 0 error 0 warning；前端 `typecheck` / `selftest` / `build` 全过；
> `lock_audit` **0**；clippy **新增/改动行 0**（27 条全在未触碰的 `nt_web.rs` 等旧代码）。
>
> 🔴 **本轮修真 bug（agent 报的问题，主 agent 逐条读现场证实后修 —— 不是照单全收）**：
> 1. **跨聊消息静默丢失**：`dedup_key` 只有 `{channel}:{message_id}`，而平台 message_id
>    **按 chat 各自编号** → A 群 5 号与 B 群 5 号撞键，**后一条被当重复丢掉**。
>    改 `dedup_key(channel, chat, message_id)`。回归测试 `dedup_key_is_scoped_by_channel_and_chat`。
> 2. **入站附件模型永远读不到**：落在 `<data_dir>/attachments`，jail 只放行
>    `<data_dir>/workspace` —— **兄弟目录**，note 里那条绝对路径必被网关拒 → **收了等于没收**。
>    改落进 `workspace/attachments`（不扩 jail）+ note 给工作区相对路径 + 按平台原名挑
>    `read_image`。回归测试 `inbound_attachment_lands_inside_the_jail_and_is_reachable`。
> 3. **第二个及以后的机器人永远收不到消息**：`run_once` 把 `poll()` 放进 `for bot in &bots`，
>    而适配器**按渠道**注册、offset 全渠道共享 → 第一个取走全部 update。改「每渠道只 poll 一次
>    + 按白名单 `route_bot` 路由」；桌面 `neobot_channel_poll_once` 同一处也改了。
> 4. **`/stop` 回执推荐了一个不取消任何东西的键**：桌面停止键只置 `shell.stopRequested`
>    让渲染跳过增量，`spawn_blocking` 照跑 —— 而测试还断言「桌面 App」必须在回执里，
>    **把假建议钉成了契约**。回执改为只说事实 + 给真能生效的路径（退出 App）；
>    测试改为**禁止**出现「按发送键」等动作短语。README / 吸收文档同步。
> 5. **2 条 clippy 告警落在本轮新增行上**（`nt_store_convos.rs` 的 10 元组 `type_complexity`）
>    → 提 `ConvoRow` 具名结构。
>
> 🟡 **本轮改了注释（原本在说谎，比没注释更坏）**：`TurnStatus` 自称有
> `failed/cancelled`（实为 `done/continue/needs_clarification/blocked/waiting`）、
> `slice_sleep` 自称让 Ctrl-C 200ms 生效（**不检查任何标志**）、`BotRow` 自称
> 「各机器人独立绑定模型」（serve 不用）、`nt_cmd_channels.rs:306` 自称「各自独立」、
> `OutboundMessage.edit_of` 看着像已实现（实为半接）。
>
> **2026-09-28 内存门 BLOCKED 期间的零编译批次（3 agent）**：
> 另一扇窗口的 `rustc` 占 3.5GB 编译 `neotrix-core`（free_pages 9.5k/100k），故本批
> **全程禁 cargo / 禁 npm install**，只做静态分析与文档。三件事：
>
> ❌ **该缺口未关闭**：「前端 invoke 键名 vs Rust 形参名没人守」——
>    本段原记「新增 `scripts/ops/nt_ipc_keys.py`（纯标准库，`--self-test` 60/60），
>    实测 声明 97 / 注册 97 / 键名错配 0，已接成 `nt_smoke.sh` 第 6 步」。
>    **2026-09-29 核实：两个文件从未存在于任何提交，也不在磁盘上**
>    （`git log --all` 零命中；`find` 零命中）⇒ **那组数字是未落地的推演，
>    不是实测**。`scripts/` 下现存的冒烟脚本只有 `experience-smoke.sh`。
>    ⇒ 缺口**仍然敞开**，需重新实现。若重做，仍需覆盖：只到键名不到类型
>    （`{taskId:123}` 键对型错不报）、只到静态不到运行时（serde 转换 /
>    `Option` 缺省仍要真进程往返）。
>    ⛔ 本条是 R-SCAN-3 的反向样本：**门记录写「已验证」而实现从未入库**，
>      比没写更坏 —— 下一个 agent 会以为缺口已关。
> ✅ **订正冻结规则引用**：`AGENTS.md` §6 正典索引（+ `DOCUMENTATION-MAP.md:21`）的
>    「R-P161-257」实为归档文件编号区间，真实规则号是 **R-P199**
>    （`archive/dev-rules-legacy-R-P161-257.md:259`，非规范副本，口径限 `neotrix-core` L1–L6）。
> ✅ **门记录刷新**（R-SCAN-3：本轮改过码，旧值即陈旧）：`lock_audit` 两 scope 均 **0 条**，
>    写入时间戳 **2026-09-28 11:58**；`AGENTS.md` 80 行（<100 未超限）。
>
> 🔴 **本批新挖出、待编译才能修的 2 条**（agent 只订正了注释、未交报告，主 agent 复核为**真**）：
> 1. **`find` 静默截断**：`MAX_SEARCH_HITS=200` 在 `nt_workspace.rs:531/539` 生效，
>    但 `SearchHit` **没有 `truncated` 字段**、`find` 也只返回 `Vec<SearchHit>` ——
>    IPC 与前端**无从知道结果被砍过**。后果具体：用户搜到 200 条就以为「全库只有
>    200 个匹配」，进而断定某文件**不存在**。`list_dir`/`read_text` 都带 `truncated`，
>    **只有 `find` 没有** —— 静默截断比报错更骗人。修它要改 `find` 的签名。
> 2. **`nt_git` 的 pathspec 只过词法门**：`check_rel` 与 `jail_join` 都是**纯词法**
>    （拒 `/`、`~`、盘符、`..`、超深），能挡 `..` 但**挡不住软链接**；
>    第二道门 `resolve_within`（`canonicalize` 后核验）是 `nt_workspace` **私有**的，
>    `nt_git` 够不着。故「工作区内一个指向外部的软链接 + 一条不含 `..` 的相对路径」
>    可让 `diff` 的未跟踪文件分支（`jail_join` 后直接 `read_to_string`）读到工作区
>    **之外**的内容。`repo_root` 本身安全（会 `canonicalize` 并在越界时
>    `workspace-jail` 拒绝），缺口只在 pathspec 这一段。**可达性待核**：
>    前提是工作区内已存在一个攻击者可控的软链接。
>
> **2026-09-28 内存门 OPEN 后第一批（4 agent，文件所有权互斥，带 cargo）**：
> 基线 308 绿 → 现 **331 绿**（+5 find / +2 git / +16 edit_of）；desktop 7 → **8 个二进制 60 绿**
> （+12 斜杠命令 +2 find 冒烟）。`lock_audit` 0；clippy 本轮改动文件 **0**。
>
> ✅ **find 静默截断**（`SearchResults { hits, truncated, truncated_by }`）：上限判定移到
>    `pop_front()` 之前 ⇒ 「还有没扫的目录」是**可证**的而非估计，故能严格区分
>    「恰好 200 个」与「被砍断」。三个上限（hits/visits/depth）**分别**如实回报。
>    顺带发现前端原有 `hits.length >= 200` **双向都错**：恰好 200 会假报截断、
>    visits/depth 撞顶时会漏报，且 200 是 Rust 常量的前端硬编码副本。
> ✅ **nt_git pathspec 第二道门**：5 个落地点（diff/log/stage/unstage/discard）全过
>    `jail_real`（`canonicalize` 后核验）。测试**先断言纯词法门放行**再断言 `Denied` ——
>    非空转。变异验证：把门改成 `if false && …` 真的漏出 `TOP SECRET`。
> ✅ **桌面斜杠指令**：`/help` `/stop` 直接调 core 的 `parse`/`help_text`（不抄字符串）；
>    `/stop` 与 IM **字节相等**；`/new` **明确拒绝**且一个会话都不建（桌面后端没有
>    切界面的通道，造假 = fiction）。「不撒谎」用**反向断言**钉（禁用词表 9 个）。
> ✅ **edit_of 三段接通** + 降级记账 `SendOutcome{edited,degraded,already_there,…}`。
>    平台回 `message is not modified` 时**不发新消息**（再发一条正是要消灭的重复）。
>
> 🔴 **本批暴露的未闭合事实（agent 主动交代，主 agent 复核为真）**：
> 1. **`edit_of` 在当前调用图里永远不成功**：`deliver_result` **没有任何生产调用点**
>    （只在 `#[cfg(test)]` 里被调，是「测试充分但生产死代码」的典型）；
>    而生产可达的 `sweep_pending`（`nt_channel_serve.rs:232` 调用）传的是**用户那条
>    入站消息**的 id —— **bot 不能编辑用户的消息**，平台回
>    `message to edit not found`，于是**每次补发先浪费一次 API 调用再降级**。
>    用户可见行为没变差（原来发一条，现在仍是一条），但编辑能力等于没接上。
>    真要生效：先发占位消息、记住**它自己**的 message_id（要动 `nt_store` 落库）。
> 2. **`read_text` 限读分支的 `truncated` 是死代码恒 false**（`end` 初值 = `buf.len()`），
>    今天不咬人（超限在上一分支已返回），但删掉那个提前返回就会静默截断。
> 3. **`MAX_SEARCH_VISITS` 不是收敛闸门**：撞到它只 break 内层 `for`，外层 BFS 继续
>    逐个读剩余目录。本轮**保留原语义**（改它属行为变更），只如实回报。
> 4. **需要改 core 才能结构同源**（~15 行）：`nt_channel_cmd::local_reply(cmd, channel)`
>    渠道无关入口。有了它，桌面那份 `STOP_REPLY` 拷贝可以删掉，两端口径靠**结构**
>    保证而非靠测试兜。
> 5. `help_text` 里的 `**当前不可用**` 在桌面气泡里**不渲染成粗体**（前端 `esc()` 后
>    按字面显示两个星号）。IM 侧是纯文本发送看不出来。
> 6. 前端 `main.ts:691-695` 流式收尾时无条件取 `tasks[0]`，团队会话里会把**上一轮**的
>    任务卡挂到本地指令气泡上。
> 7. `SearchResults`/`SearchCap` 未进 `lib.rs:101` 的 re-export 列表（功能不受影响，
>    走 `nt_workspace::` 路径；按惯例应在其中）。
>
> 🔧 **主 agent 本批自己修的 2 处**：
> - `nt_channel_cmd.rs:177-178` **我自己早先写的注释被本轮改动证伪**（写着「桌面聊天
>   也不解析斜杠指令」，而桌面现在解析了）—— 注释在说谎这个毛病，本轮我又犯了一次。
> - `nt_channel_dispatch.rs:507` 新代码用了 `payload["edit_of"] = …`，本 crate 开着
>   `-W clippy::indexing_slicing`；改用 `as_object_mut()`（`Value` 下标插入在 key 类型
>   不对时会 **panic**）。
>
> **2026-09-28 第三批（4 agent，带 cargo）**：331 → **353 core 绿**，desktop **63 绿**（8 个二进制）。
>
> 🔴 **本批最大发现：一个一直存在的 flake 源（不在原清单里，是我核验时撞见的）**：
> neobot crate 有 **30 处**测试用 `std::env::temp_dir().join(固定名)` 造夹具
> （19 处完全固定名，11 处只按用例名唯一化 ⇒ **同一用例跑两遍仍撞**）。
> 一个测试的 `remove_dir_all` 会删掉另一个正在用的夹具 ⇒ 断言在与被测代码无关的地方炸。
> **实测**：同一 test binary **并发跑 3 份，每份各挂 9–18 个测试，且每份挂的不是同一批**
> ——共享夹具互相踩的指纹。串行 353 全绿。
> 这解释了此前那次「352/1 失败、8 次重跑又全绿」的幽灵。
> **危害比单测红更大**：我常态派多 agent 同时 `cargo test` 同一 crate（cargo 只串行化
> **构建**，两个 `cargo test` 的**执行**阶段并行），所以这会持续制造假红灯，
> 而假红灯会训练人忽略红灯。
> **已修**：新增 `nt_testutil::temp_dir(tag)`（pid + 单调纳秒唯一），30 处机械替换；
> 对照实验 **4 实例并发 × 353 测试 → 0 失败**（修复前 3 实例各挂 9–18）。
>
> 🔴 **`.gitignore` 的裸 `tests/` 规则静默丢弃集成测试（跨批次累积，不只影响本轮）**：
> `.gitignore:122` 写的是 `tests/`，git 的裸模式**匹配任意层级**，于是把各 crate 的
> **集成测试目录**一起吞掉。实测：`neotrix-core/tests` **26 个 .rs 只入库 2 个**（24 个丢），
> `apps/neobot-desktop/tests` **6 个入库 0 个** —— 本轮建的 55 个 IPC 测试
> **存在于磁盘却永远进不了版本库**，clone 下来等于没有这套网。
> （好消息：`src/**/tests` 模块目录**都已入库**，所以全新 clone 仍能编译。）
> **已修**：锚成 `/tests/`（根目录那个 scratch 目录仍被忽略），27 个文件重新可入库；
> 已入库的不受影响（gitignore 只管未跟踪文件），是纯增量修复。
>
> ✅ **`read_text` 限读分支「做实」而非删掉**（删掉等于移除唯一兜底，会退化成静默截断）；
> ✅ **`MAX_SEARCH_VISITS` 改成真收敛**（`break` → `break 'bfs`，否则「总访问上限」名不副实）；
>    变异验证：`break` 改回 `break 'bfs` 立刻红；agent 还**用它自己注释里的手推值证伪了手推**
>    （原写「会扫完 930 项」，实测 80）—— R-SCAN-2。
> ✅ **core 加 `local_reply(cmd, channel)`** 渠道无关入口，桌面删掉 `STOP_REPLY` 拷贝，
>    回执文本**全仓只剩一处**；`/stop` 字节不变。
> ✅ **前端任务卡归属改语义判据**（`turn_task.ts`）：本地指令不再挂上一轮的任务卡。
>    agent 诚实报告**真 id 前端拿不到**（`NeobotRunResult` 只有 status+labels），
>    用「窗口内新建 + 归属本会话」作间接证据并明说残余风险。
> ✅ **`/stop` C0+C1**：租约续租（有界+按时间退避，`renew_lease` 是条件 UPDATE 防越权）
>    + 四个取消检查点；中止落 `TaskStatus::Cancelled` 并带「第几跳」；
>    **回执一个字未改**（仍诚实说停不了）。
>
> 🟡 **本批新发现（未做）**：
> 1. `apps/neobot-desktop/tests/` 曾被 gitignore 整目录忽略（已修 `.gitignore`，
>    但**这些文件仍未入库** —— 需要一次 `git add` 才会真正进版本库）。
> 2. **一条既有测试本来就是恒真的摆设**：`help_text_is_byte_identical_to_the_im_help_text`
>    比的是 `help_text(X)` vs `help_text(X)`，上一批桌面直调该函数后它就测不出东西了
>    （agent 用变异实证，非手推）。按「不许删既有断言」的要求留着，但**它现在没有牙**。
> 3. `help_text` 里命令名仍被反引号包着（也是 markdown 记号，两端按字面显示）；
>    agent 未动，因为改它会变更 IM 侧用户可见文案。
> 4. `nt_store/mod.rs` 不是任何 agent 的所有权，导致 `last_step` 只能返回
>    `Option<(bool, String)>` 元组而非具名 `StepRow`。
> 5. 真实跑轮的 `task_id` **端到端缺失**：正解是后端在 `NeobotRunResult` 加 `task_id`；
>    同类问题还有 `collect_labels` 用 `list_tasks(1).first()` 取 step 工具。
> 6. `/stop` C1 的 `run_local_turn_cancellable` 改成了吃 `RunContext` 的 4 参签名
>    （设计文档 §10.5 那条 11 参签名会触发 `too_many_arguments`）——**C2/C3 的 agent 必读**。
>
> **2026-09-28 第四批：/stop 第一次真的能停了**。core **360 绿**、desktop **71 绿**（9 个二进制）。
>
> ✅ **桌面 `/stop` 真的能停**（本会话最重的一条）：`neobot_run_cancel` + 按 `convo_id` 索引的
>    登记表 + RAII 注销（世代号比对，防旧轮次误删新轮次的令牌）。找到不到在飞轮次**如实报错**，
>    **绝不静默成功**。前端停止键从「只置 flag 让渲染跳过增量」改成**真调后端**，
>    取消失败**可见**（`pushSys` + toast）并**撤销乐观隐藏**（`stopRequested` 清回、
>    `setRunning(true)`、`resume()` 重画流式）。
>    **变异实证**：把 `slot.token.cancel()` 改成不翻，关键测试 FAILED 且**挂住 60 秒**
>    （轮次永不停止、假模型闸不放行）—— 证明这条不是恒真断言。
> ✅ **IM 侧管道接通**（登记/递令牌/翻旗三段全真，7 个新测试含 4 次变异验证），
>    并**删掉**了 `on_inbound` 那个「所有调用方一律传 `None`」的 `stop_flag: Option<&mut bool>`
>    死形参（类型上接不住 `Arc<AtomicBool>`，结构上 `/stop` 与跑轮在不同调用里）。
> ✅ **`task_id` 端到端**：`NeobotRunResult` 加 `task_id` + `cancelled`，
>    填法是「跑轮前后 id 集做差 + 认领本会话那一个」，**不用 `list_tasks(1).first()`**；
>    `collect_labels` 同类歧义一并修。前端 `turn_task.ts` **优先用后端给的真 id**。
>
> 🔴 **待决（我没单方面改）：`/stop` 回执现在对桌面是假的**
> core 共享回执仍写「**停不了**：跑轮是同步的…（桌面 App 的停止键也只收起输出、不终止运行；
> 要真的终止得退出那个 App）」。这在 **IM 侧仍字面为真**（同步调度，`/stop` 被读到时
> 那一轮已结束），但**在桌面侧已经是假的** —— 桌面现在真能停。
> 牵连 4 条测试契约：`nt_agent.rs:2316`、`nt_smoke_slash_cmd.rs:265/437/673`。
> **推荐措辞**（对两端都成立的**条件句**，不承诺桌面、不否认 IM）：
> 「如果那一轮还在跑，我把停止信号递过去 —— 但**正在跑的那一次模型调用不会被掐断**，
> 它会在下一个检查点（跳边界 / 工具边界 / 睡之前）收手。递出信号**不等于**已经停下：
> 停没停、停在哪一跳，由那一轮自己的结果说。那一轮若已经结束，就没什么可停的。」
> 每趟的**事实**那半句由 `StopState::text()`（IM，三态）/ 前端状态机（桌面）给，两端已各自到位。
> `help_text` 里 `/stop` 的「（当前不可用：跑轮同步…）」同理需要重估 —— 现在「不可用」
> 只对 IM 成立。
> **不单方面改的理由**：这是用户可见的**产品措辞**决策，且与 4 条「反撒谎」测试契约互锁；
> 错误方向是**低报**（说做不到而实际做得到），危害小于高报，但仍需有人拍板。
>
> 🔧 **本批的诚实披露（agent 主动交代的物理限制，不可用测试绕过）**：
> 1. **echo 引擎测不出「跑轮途中取消」** —— 它那轮只有一个跳、无 tool_calls、跑完就 break，
>    四个取消检查点**一个都走不到**。e2e 因此用本机假 SSE 端点让它回 tool_calls，
>    并用「按下标逐个放行」的闸把「翻令牌 vs 拿到响应」的先后变成测试定的（不靠 sleep 竞速）。
> 2. **一次阻塞的模型调用内部无法中断**（引擎里没有检查点）。能停的是「跑轮**还会继续**」的那些轮子。
> 3. 取消登记表同键重入时**只有最新那一轮可停**（被覆盖的那轮取消时如实报错）。
>    按 `task_id` 索引**做不到** —— 任务行在跑轮内部建，跑轮期间没有回调把 id 递出来。
> 4. `task_id` 在「同一瞬间别的跑轮在本会话也建了任务」时认不出，交空串、前端回落间接判据。
> 5. 桌面那条反向测试 `stop_receipt_still_admits_the_limitation`（`nt_agent.rs:2283-2291`）
>    **仍全绿但前提已过期** —— 它假设「取消尚未接线」，现在管道接了，只是同步调度让
>    第二个执行流仍不存在。**与上面待决的回执一起处理。**
>
> 🟡 **本批发现但未做**：
> - `nt_store` 下仍有位置性返回：`nt_store_changes.rs:228` `(task_id, title)`、
>   `mod.rs:235` `export_counts (i64,i64,i64)`（调用点分别在 `nt_changes.rs` / `nt_export.rs`，
>   都需授权）。`last_step` 已改成具名 `LastStep{ok, output}`（`tool` 是查询形参、
>   **不在返回值里**，极易被误当第二字段名）。
> - **一条静态锁把「实现细节」当成了契约**：`nt_smoke_cancel.rs` 原先 grep 裸
>   `invoke("neobot_run_cancel"`，被 `ntInvoke(` 弄红 —— 已改为同时认两种入口，
>   并用变异验证（把停止键改回只置 flag ⇒ 必红）。
>
> 🔴 **仍然待办（本轮只做到「不说谎」，没做到「能停」）**：
> - **`/stop` 真能用** = 架构级：调度改并发（线程池/async）+ `nt_agent` 的 stop hook。
>   ⚠️ 原注「设计见 `docs/architecture/DESIGN-CHANNEL-DISPATCH.md`」——**该文档从未入库**
>   （2026-09-29 核实 `git log --all` 零命中），非过时而是**丢失**。开工前需重写设计。
> - **per-bot `token_env` / `model` 未生效**：一个渠道一个共享适配器，跑轮只用全局
>   `config.engine`。已在多机器人时 `warn_shared_adapter_once` 警告，但要真支持得先有
>   chat→bot 绑定字段。
> - **`edit_of` 半接**：`deliver_result`/`sweep_pending` 填了 `Some`，但 `send()` 不实现
>   `editMessage`，payload 里也没这字段 → **没有任何路径真会编辑原消息**，用户只看到重复两条。
> - **桌面聊天不解析斜杠指令**（走 `neobot_run_stream`）→ 桌面打 `/stop` 会被当字面文本发给模型。
> - 冒烟脚本**不存在**（2026-09-29 核实）：`scripts/ops/nt_smoke.sh` 从未入库，
>   `scripts/` 下现存的只有 `experience-smoke.sh`。本条原记「因内存门 BLOCKED
>   未整体跑通，其 5 个步骤已逐项手工跑绿」——**「脚本编排」本身不存在，
>   无从跑通**。⇒ 下面的缺陷清单仍有效（它们是读代码得出的），但
>   **「已跑绿」不构成任何验证证据**。
> - 无浏览器设施（Playwright/Puppeteer）→ 前端真实渲染仍未端到端验证；前端 invoke
>   **键名**与 Rust 形参名的跨进程一致性仍**未验**（冒烟只验了参数绑定层）。

> **2026-09-27 目录架构统一轮（本轮，已 cargo 验证）**
> `cargo check -p neotrix --lib` **exit=0 / 1m07s**。完成 12 项：
> - 🔧 **修真 bug**：`KnowledgeBase::open` 把 SQLite 哨兵 `":memory:"` 当**文件路径**传，
>   在磁盘真建库 + `with_extension("lock")` 侧车再造假 `:memory:.lock`（落在仓库根）。
>   9 个调用点全修。**同族 `TemporalFactLedger::open`(`nt_temporal_facts.rs:63`) 早有正确处理，
>   唯独漏了此处** —— 按同一模式补齐。锚点 `l4_emotion/nt_memory/nt_memory_kb/kb_core.rs:101-111`
> - 删 `protocol/`（312 行 NIP-01 死代码，从未编译）与 `adapter/`（7 行陈旧桩）、`games/`（仅 `.DS_Store`）
> - `.gitignore`：`sessions/` 整目录忽略 → 按内容忽略 + 白名单（此前 11 个文件全靠 `git add -f` 硬塞）；
>   `HANDOFF*.md` → `/HANDOFF*.md` 锚定到根（无斜杠会匹配任意层级且在文件末尾覆盖白名单）；
>   清 3 条指向已删 `games/neotrix-guixu/` 的陈旧规则
> - `sessions/HANDOFF-TEMPLATE.md` **入库**（`AGENTS.md` 引用它做交接模板，模板不在库 = 规范不可执行）
> - 正典索引统一：`AGENTS.md` + `DOCUMENTATION-MAP.md` → 指向 `docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md`
>   （`docs/architecture/README.md:7` 早已声明它是"唯一图纸入口"，不一致的是索引不是文档）
> - 规则沉淀：`AGENTS.md` 新增 R-SCAN-1/2/3（扫描器告警≠缺陷 / 手推≠实证 / 门记录必带时间戳）+
>   下刀前查 mtime；`RUST-STANDARDS.md` §17 新增 R-LOCK-4/5、R-BUILD-6、R-GIT-5；
>   门记录刷新为"22:52 实测 3 条 → 23:4x 复测 **0 条**"
> - 经验文档：`docs/architecture/LESSONS-2026-09-27-scanner-trust.md`

> **2026-09-27 卡死/内存专项 + 长尾分诊（收口）**：全量 `--lib` 串行实跑
> **11493 绿 / 51 红**（起点 10113/125）。8 条卡死/内存根因全部除根并加三道闸；
> 21+25+27 = **73 处生产缺陷**已修（安全洞 5、解析/数据 11、逻辑 25、stub 补实现 9、
> 环境依赖去抖动 6、契约对齐 12、并发/治理 5）。详见
> `sessions/handoff-disease-list-20260927.md`（含 125 条分诊全表与 file:line）。
>
> **待办（按性质分组，非按模块）**
> - 🔴 **B 类·需设计裁决**（不可一行改，先定方向）：
>   1. `l0_substrate/nt_core_kb_primitives.rs:188` — `nodes.id` 单列 PK 使双时态版本化
>      不可能 → 需 PK→`UNIQUE(id,transaction_time)` + edges FK 重构 + 真实库迁移（2 测试）
>   2. `l4_emotion/nt_memory/cascade/cascade.rs:216,126` — `length_score = len/200` 过小，
>      且 `tick()` 永久丢弃未达阈样本（属设计缺陷，需重定晋升/丢弃策略）
>   3. `neotrix-types` 的 `Severity`/`FlagSeverity` 判别序"越严重越小"是**承重约定**
>      （`l2_perception/nt_world/osint/sweep.rs:225` 依赖它做 `min_severity` 过滤）→ 不要盲翻 `Ord`
> - 🟡 **C 类·未接线 stub**（实现或显式 `#[ignore]`，禁止改松断言凑绿）：
>   `nt_core_embed::TextEmbedder`（字节位置袋，任意文本相似度≈0.83）、
>   `nt_shield_ztnet/crypto/noise_handshake`（**已解决**：已按官方向量重写为 2-message IKpsk2，`_create_message3` 已删除）、
>   `publish_gateway`（YouTube 上传）、`nt_codegen::parse_yaml`（误用 serde_json，需引 serde_yaml）、
>   `nt_memory_kb::nt_memory_distill`（测试 teacher 与 student 恒等，`after<before` 不可满足）
> - 🟡 **环境依赖**（改确定性断言或 `#[ignore]`）：~~`l6_meta::runtime_monitor::test_get_health`（探针挂起）~~
>   ~~`nt_feel::writing_style`（2）~~ —— **已由 cycle `audit0927b` 修掉，非环境问题**：
>   runtime_monitor 是**真死锁**（monitor 持 metrics 守卫调 check_thresholds，后者再
>   锁同一把，std Mutex 不可重入）；writing_style 是该 1,241 LOC 模块**从未被 mod 声明**，
>   从未编译，接上即 0 error 0 warning。详见 `sessions/handoff-disease-list-20260927.md` §10。
>   仍待处理：`l6_meta::nt_core_aware`（4）、`nt_core_observer_error`（2）、
>   `nt_feel::cognitive_bridge::feedback`（2）
> - 🔴 **P0·本线遗留（cycle `audit0927`/`audit0927b`，按建议顺序接手）**：
>   1. **50 个失败测试** —— 多数是断言与实现漂移（夹具自带 `Always` 规则使被测分支
>      不可达、亚毫秒时长断言 `> 0`）。单条易改但 50 条一起动风险大，**独立成轮**。
>      分布：nt_shield 7 / nt_core_capability 6 / nt_memory 5 / nt_feel 5 /
>      nt_core_aware 4 / nt_meta 3 / healing 3 / 其余 17 个各 1-2。
>      复现：`cargo test -p neotrix --lib --no-run` 后跑二进制，`--test-threads=2`。
>   2. **`--test-threads=4` 时 SIGSEGV（退出码 139）** —— 崩在
>      `l6_meta::healing::predictive_maintenance::trend::tests` 之后；同模块单/双线程
>      跑均不复现（311 passed）。pre-existing 并发问题，**CI 暂用 `--test-threads=2`**。
>      根因需单独定位（共享资源竞态 / 栈深 / fd 耗尽）。
>   3. **三处同名类型双定义** —— `ExtractConfig` / `EmailConfig`
>      （`extractors/mod.rs` 与 `data_pipeline.rs` 各一份）、`PlatformRegistry`
>      （`data_pipeline.rs:211` 与 `platform_registry.rs:46` 各一份）。
>      `TradeDataPipeline::with_registry` 只认后者，收敛前须先确认二者语义是否本就该合并。
>   4. ~~**4 个抽取 crate 的定位裁决**~~ — ✅ **已裁决并执行**：`crates/nt-lang`
>      （只有 `[[bin]]` 无 `[lib]`、结构上无法被任何 crate 依赖）**已于 `2bbed32c`
>      （2026-09-28）删除**，「补 `[lib]`」选项已作废。⇒ 本项**关闭**。
>      ⚠️ 残留说明文件 `skills/crates/nt-lang/SKILL.md` 仍在 —— 该文件本身**存在且有效**
>      （已改写为历史说明：标注 crate 已删、命令不可用、复活方式）。
>      `skills/index.json` 的 `crates/nt-lang` 条目已于 2026-09-29 摘除。
>   5. **`crates/nt-core-capability-tree` 住在 `src/` 里** ——
>      独立 crate（根 `Cargo.toml:11` member）却位于另一 crate 的源码目录，
>      4,670 LOC / ~30 处真实调用。已补 `[lints] workspace=true`（原 18 条 lint 全失效，
>      补上后暴露 20 条告警含 7 处可能 panic），但**归属未裁决**：是搬出去还是接受。

> - ⚪ **结构性债务（PARK，有归属前置）**：L0 `CapabilityRegistry` ×4 + `SemanticRouter` ×2
>   正典收敛（异构，需专窗迁移）｜`proxy_pool.rs` 1757 行拆分（他人在途 1039+/24-）｜
>   剩余 God-file（`pdf.rs` 2142 / `nt_crystal_serve.rs` 2002 / gateway 1678 / hex 1530）｜
>   5 个内容型 worktree 裁决｜前端 `apps/neobot-desktop/frontend/{src,dist}` 归属与提交

> **2026-09-27 第三轮（4 代理并行）已改未验 — 交接给下一对话，优先收口**
> 30 项修改在 working tree，**全部未提交**（提交门禁需跑 cargo，当时内存门 BLOCKED）。
> 验证进度：一次跑 **802 passed / 2 failed**（29 项中 28 项绿），补修后再跑 **65 passed / 1 failed**。
> 唯一残留：`noise_handshake::full_handshake_matches_official_vectors`（**已重写并转绿，2026-09-29 合入主干**）—— 已从"构造即 panic"推进到
> `_consume_message2` 处 unwrap 失败（`noise_handshake.rs:420`），是真 crypto 缺口。
>
> **🔧 接手第一步（务必按序）**
> 1. `sh scripts/ops/nt_mem_gate.sh; echo $?` → 必须为 0 才继续
> 2. 定向验证：`CARGO_BUILD_JOBS=1 cargo test -p neotrix --lib -- <模块前缀> --test-threads=1`
> 3. 全绿后 `git commit -- <paths>` **pathspec 限定**（禁 `git add -A`、禁 `--no-verify`）：
>    `nt_core_capability/{dependency,integrator,monitor}.rs`、`nt_meta/gwt_router/{cost_weight,attention}.rs`、
>    `nt_core_self/dynamic_params.rs`、`nt_agent_identity.rs`、`nt_core_aware/mod.rs`、`nt_feel/{cognitive_bridge/feedback,writing_style,salesperson_profiling}.rs`、
>    `nt_memory/{cascade/cascade,consolidation/cache,distillation/distiller}.rs`、`nt_memory_kb/{memory_orchestrator,nt_memory_distill}.rs`、
>    `nt_shield_sandbox/stateful_bench.rs`、`nt_shield_ztnet/crypto/noise_handshake.rs`、
>    `nt_shield/{compliance/requirement,nt_shield_audit/threat_modeler,shield_core/audit,shield_core/safety_kernel}.rs`、
>    `nt_core_speculative_decoding.rs`、`nt_core_vector_store/store_hnsw.rs`、`nt_codegen.rs`、
>    `nt_core_guardian/repair.rs`、`crates/neotrix-types/src/{core/shared_types.rs,llm_types.rs}`、
>    `Cargo.toml`、`neotrix-core/Cargo.toml`
>
> **⚠️ 本线已做但未验证的高价值修复（接手方请优先确认）**
> - `nt_core_guardian/repair.rs` —— 自愈动作 `ClearCache` 原本执行 **`cargo clean`**
>   （会删整个 target/ 含 deps 活指纹）。已改为只删 `target/<profile>/incremental`。
> - `noise_handshake.rs` —— `hash[..27]` 拷 25 字节协议名，**任何构造都 panic**，
>   整个模块从未可用过。已按字面量自身长度自适应。
> - `stateful_bench.rs` S4 —— 原本把安全修复前的"有洞极性"写成断言（给漏洞背书），已互换策略布尔。
> - `store_hnsw.rs` —— 索引按余弦排序却报 Hamming 距离（自相矛盾），Hamming 配置改走图外精确扫描。
> - `check_ip` —— 原本无 CIDR 支持，IP 白名单形同虚设；已实现（blacklist 同步）。
> - `check_ip`/`serde_yaml` —— `serde_yaml` 已入 workspace + core 两处 manifest，
>   **首次构建会自动更新 Cargo.lock**（离线可解，`--locked` 构建会失败，需注意）。
>
> **📋 剩余 51 条待修**：全量 11493 绿 / 51 红，逐条根因见
> `sessions/handoff-disease-list-20260927.md` §9（P 17 / S 27 / U 4 / E 3，含 file:line）。
> **🧭 3 项需人工决策**：`sessions/handoff-decision-20260927.md`
> （双时态 PK 建议删 API / CAD 证据表建议砍到 4 个 / publish_gateway 建议加 dry_run）。


> **Batch3 吸收执行 (47 源)**: 四波 21 任务 20/20 闭环 · **交接 Wave 4: 10 任务待做** (🔴P0×3 越层修复/e8_state 合成值/测试抖动加固 · 🟡P1×3 情报工具接线/SEAL C0→C2/补全排序 · ⚪P2×4) → ⚠️ 原引 `docs/absorption-knowledge-base/batch3-2026-08-26-unified-evolution-todo.md`**已随 `477bf669`（文档标准化清理）连同整个 `docs/absorption-knowledge-base/` 删除**。吸收成果的现行落点是 `docs/architecture/ABSORPTION-*.md` 系列（见 AGENTS.md §6）；Wave 4 的 10 项待做若仍有效，需重新归档到 `docs/architecture/` 下 Wave 4 段 + 根 `HANDOFF.md` (2026-08-26 版)
---

# 统一进化清单（2026-09-27 重建）

> 本节由三份审计合并去重而成，是**唯一**的进化任务入口。
> 来源：`docs/architecture/DIR-AUDIT-2026-09-27.md`（目录/依赖）·
> `docs/architecture/_superseded/EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md`（18 项支脉定位）·
> `docs/architecture/ABSORPTION-EXTERNAL-2026-09-27.md`（19 项外部吸收）。
> 三份文档保留推导过程，本节只保留**去重后的可执行条目**。
> 每项均带 `file:line`；无定点不改（RUST-STANDARDS §17）。


---

## 🆕 2026-09-28 单窗口汇总修复（架构侧吸收轮）

> **唯一汇总入口**：`sessions/handoff-consolidate-all-windows-20260928.md`
> **吸收源清单**（483 仓 + 436 条榜单排名 + 许可台账 + 5 论文）：`docs/architecture/absorption-sources/`
> **路线图 / 裁决表 / 目录解法 / 任务清单 / 经验**：
> `EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md` · `OWNERSHIP.md` · `DIR-REMEDY-2026-09-28.md` ·
> `BATCH-FIX-CHECKLIST-2026-09-28.md` · `LESSONS-2026-09-28-measurement-and-dedup.md`
>
> **汇总 7 窗口 + 架构侧，由单一窗口统一执行**（避免多窗口并发编译，16G 机必爆 swap）。
> 全部数据 2026-09-28 实测。`[实测]`=本人复核；`[转述]`=引用他窗，**接手前自行复核**。

### ✅ 已完成（车道 `lane/batch-fix-20260928` @ `af0389a1`，**11 commit，整分支零 `.rs`**）—— ✅ **本提交已并入主分支（S-1）**

- [x] **A-1** `.neotrix/layer-map.json` —— 层归属显式化，让第二棵树（**128 文件/42,070 行**，干净检出实测）**不用搬目录**就可治理。**`bdf1e9f1` 已入主分支**（此前只在车道 ⇒ 新 clone 拿不到，分层门完全看不见这棵树却报 PASS）
- [x] **A-2** `scripts/check-naming.sh` —— `nt_` 前缀门（advisory），实测 **1,646** 不合规（**`bdf1e9f1` 已入主分支**；此前记的 1,644 是脏树值）
- [x] **A-3** `docs/architecture/OWNERSHIP.md` —— 唯一裁决表，9 节，**以构造点取证**。✅ **随本提交并入主分支**
- [x] **A-4** `check-layer-deps.sh` 纳入 layer-map —— 第二棵树首次可见，暴露 **10 处**此前无门能发现的跨层引用。**`bdf1e9f1` 已入主分支**，基线 = 干净检出实测 **102**（92 `l*_` + 10 第二棵树）；⚠️ 车道里那版基线是 **94（脏树测量）**，照抄会让 CI 以 `FAIL: 8 new` 红
- [x] **吸收源固化** `absorption-sources/` —— 483 仓 CSV + 许可台账 + 5 论文
- [x] **证伪修正** —— 09-27 路线图「删 `nt_core_gate/nt_tool_registry.rs`（stub）」**是错的**
- [x] **经验留痕** `LESSONS-2026-09-28` —— 5 条，含我自己 3 次翻车

### ⏳ S 组 · 零编译风险（接手即可做）

- [x] **S-1** 合并车道 `lane/batch-fix-20260928`（11 commit，零 `.rs`）—— ✅ 本提交完成；交付 OWNERSHIP.md 等 4 份正典文档
- [ ] **S-2** ⛔ 分层基线**只在主树、且主树干净时**测 —— 车道测 **102**、主树脏状态测 **94**，差 8 处
- [ ] **S-3** 裁决 `nt_file_ability` 层归属（声明 L1 却引 L2/L5/L6，10 处）
- [ ] **S-4** 裁决 `ffi` 层归属（声明 L0 却引 L5：`consciousness_tree.rs:290,320,321`、`seal_pipeline.rs:131,132`）
- [ ] **S-5** 给 `nt_core_gate/nt_tool_registry.rs` 加防误删注释（活消费者 `nt_shield_enforcer.rs:388-390`）
- [ ] **S-6** 统一提交 neobot 那 54 个路径（`git add` **精确列出**，禁 `-A`）
- [ ] **S-7** 拍板 `/stop` 回执措辞（**唯一需产品决策项**）

### ⏳ B 组 · 冗余清理（需编译验证；零消费者已实测）

**当前实测计数**：`CapabilityRegistry` **4** · `ToolRegistry` **3** · `SkillRegistry` **4** · `error_conversions.rs` **6**

- [ ] **B-1** 删 `neotrix/nt_file_ability/capability.rs:185` 的 `CapabilityRegistry`（0 消费者 🟢）→ **4→3**
- [ ] **B-2** 删 `l5_cognition/nt_core/capability/registry.rs:462` 的 `CapabilityRegistry`（仅自测 🔵）→ **3→2**
- [ ] **B-3** `neotrix-types` 包内 `SkillRegistry` 2→1（`core/skill.rs:54` / `core/skills/mod.rs:25` 🟢）→ **4→3**
- [ ] **B-4** `error_conversions.rs` **6 → 1**（`l1_action` 有 26 个 `From` impl）🔴 → **解 17 个 E0119**
- [ ] ⛔ **不要删** `nt_core_gate` / `agentic_browse` 的 `ToolRegistry` —— **与 `nt_act` 正交**（运行期统计 vs 写操作可逆性 vs crawl 局部）

### ⏳ D 组 · 跨域错位

- [ ] **D-1** `neotrix-sysctl`（**全仓唯一 `unsafe` FFI**）从 L5 剥离，只留 `l0_substrate` 依赖
- [ ] **D-2** `nt-lang` 孤儿：5 文件/273 行、**只有 `[[bin]]` 无 `[lib]`**、**0 个主树 manifest 依赖** → 删或补 `[lib]`+消费者
- [ ] **D-3** 纠正 `ARCHITECTURE.md:97`（称 `nt_computer/` 是「计算集群」；**实测是 fs/process trait**，`screenshot()` 默认 `None`，neobot 侧 `NoopBackend` 唯一后端）
- [ ] **D-4** `nt_file_ability` / `ffi` 层归属（解码层，同 S-3/S-4）

### ⏳ E 组 · 能力补齐

- [ ] **E-1** 证伪门（`crates/neotrix-audit/` + CI）—— ⭐ **11 个自进化仓无一证明自己有效**；prime-agent 的 `RefinementEvent.outcome` 是模型自写自由文本
- [ ] **E-2** 记忆权威头 + 五个留存标签（`experience_tree/mod.rs:29`）—— `Authority` 与 `Confidence` **必须解耦**（置信高 ≠ 有权）
- [ ] **E-3** delta-ops 取代整体重写（`experience_tree/mod.rs:241` `:355`）—— 零 op ⇒ 文档逐字节不变
- [ ] **E-4** 成本归因插点（`anthropic/anthropic.rs:93` 的 P0-4 断点）—— **skill 是一等维度**（120 个 skill）
- [ ] **E-5** GUI 执行回路 5 项（`l3_embodiment/nt_computer.rs`）—— ⭐ `capture_id`（cua-driver）是全清单**唯一**坐标漂移解
- [ ] **E-6** DNS qtype 白名单（`egress_types.rs:14-21`，**全文件零 DNS 概念**）

### ⏳ C 组 · 结构性（依赖 B 组）

- [x] ~~**C-1** `UnifiedApiImpl` 脱 stub（`src-tauri/src/stub.rs:275`）~~ **作废 2026-09-28**：`src-tauri` 已于 `5c02e738` 归档，该文件不存在，此任务不再可执行。留条目是为了让搜索得到的人知道它为什么没被做。
- [ ] **C-2** 20 处 `Orchestrator*` 收敛（真典候选 `neotrix-core/src/pipeline/` 7 文件/1,130 行）
- [ ] **C-3** 三棵记忆树裁决（`l4/nt_memory` 236 文件 · `l5/nt_mind` 422 文件 · `l6_meta/memory` 7 文件）
- [ ] **C-4** `neotrix::neotrix::` 双命名消除（**20 处**）—— P1 层归属解耦后已降级为可选

### ⚠️ 本会话 3 次自身错误（勿重犯，已入 `LESSONS-2026-09-28`）

- [ ] **测量台纪律**：基线只在**主树且主树干净**时测。**脏树比干净树「更干净」**（主树脏测 94 / 车道干净测 102）
- [ ] **机器读 ledger 不能加注释**：`check-layer-deps.sh:153` 的 `grep -c .` 会把 `#` 行计为条目，94→105 棘轮失真
- [ ] **共享 index 会竞争**：曾见暂存区混入 **46 个非我暂存的 `.rs`** ⇒ worktree 的独立 index 才是真隔离

### ⏳ 各窗口剩余债 `[转述]`（**自行复核，勿照单全收**）

- [ ] neobot：`nt_smoke.sh` **6 步编排从未整体执行过**（各步手工跑绿）· `edit_of` 真正生效 · IM 的 `/stop` 兑现
- [ ] 测试债：`nodes` 表无法存双时间历史（2 条测试）· Noise 握手协议
- [ ] **crystal id 分配无互斥**：8 个吸收脚本各自独立 `fast_max_mid(COCOONS)`，8 个茧时间戳集中在 5 分钟内
- [ ] 50 个失败测试（**独立成轮，勿与结构清理混做**）· `--test-threads=4` SIGSEGV（`l6_meta::healing::predictive_maintenance::trend::tests`）
- [ ] 三处同名双定义：`ExtractConfig` / `EmailConfig` / `PlatformRegistry`

> **已实测推翻 2 条转述**：① `nt_core_gate/nt_tool_registry.rs` **不是 stub**（有活消费者）；
> ② `nt_jev` + `nt_crystal_core` **是活路径**（L1 有 6 个消费者）⇒ **禁止当死代码删**。
> 「导出 ≠ 调用」已错过 4 次。裁决表见 `OWNERSHIP.md`。

### ⛔ 执行前硬闸

- [ ] 无他窗在写：`find neotrix-core/src crates -name '*.rs' -mmin -5 | head` **必须空**
- [ ] 内存闸 OPEN：`sh scripts/ops/nt_mem_gate.sh; echo $?` **必须 0**（2026-09-28 实测 **exit 2**，两个 `rustc` 各 2.5 GB）
- [ ] 禁 `git add -A` / `git reset --hard`（主树含他窗 441 个 `.rs` 改动）

---

## 阶段 0 · 脱 stub 与收敛（1-2 天，最高杠杆）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 0.1 | ~~**`UnifiedApi` 脱 stub**~~ **作废 2026-09-28** | ~~`src-tauri/src/stub.rs:275`（零状态单元结构体）· `:289`（返回字面量）~~ — 随 `5c02e738` 一并归档，不再可执行 | 作废 |

  **纠正**：`main.rs:387` 与 `:407` 的调用方是 **CLI 子命令**（`Headless` / `Reason{prompt}`），
  **不是 GUI**。GUI 经 `domain/plugins/chat.rs:166`
 （真调 `consciousness_core::execute_task_loop`，零占位）与 `ntcode/commands.rs`
 （真实例化 `UnifiedModelPool::default_pool()`）走真实内核。
  故正确动作是：让 `UnifiedApiImpl` 委托给 domain plugins 已用的同一后端
 （`UnifiedModelPool` / `consciousness_core`），**或**废弃 CLI 子命令对 stub 的引用。
  隐藏工作量仍成立：`UnifiedResponse.metadata`（`stub.rs:292-303`）已预留
  `consciousness_state{phi,coherence,gwt_resonance}` + `confidence`，与 L5 意识核类型级吻合 |（零状态单元结构体）· `:285-304`（`:289` 返回字面量）· `main.rs:52` 注册源 · `main.rs:387,407-408` 调用点 | ⬜ |
| 0.2 | **删 `ToolRegistry` 零消费者副本** ✅ | 4 份**全不同型**（逐对核实）。已删 `crates/neotrix-gateway/src/gate.rs:1273`（`7664ecd8`）；`nt_core_gate` 那份**在用**（run.rs:4、shield_enforcer.rs:388）**保留** | ✅ |
| 0.3 | **`CapabilityRegistry` 4→1** ❌ **结论：不成立** | 4 份全不同型；l0 版**有真实跨层消费者**（ocr/mod.rs:405、nt_act_trade/capability_registry.rs:171）。仅 nt_core 版仅自测、nt_file_ability 版零消费者，已加注释 | ⚠️ 改判 | `l0_substrate/nt_core_capability_types.rs:533` · `l5_cognition/nt_core/capability/registry.rs:453` · `neotrix/nt_file_ability/capability.rs:173` · `nt_core_capability_tree/src/registry.rs:53` | ⬜ |
| 0.4 | **`SkillRegistry` 5→1** ✅ 部分 | 逐对核实：5 份中**仅 multi-agent 与 gateway 逐字重复**（内部外部零引用、4 测试同名重复），已删 600 行（`474c2b7e`）。其余 4 份不同型保留 | ✅ | 自重复：`neotrix-types/src/core/skill.rs:54` + `core/skills/mod.rs:25`；另 3 份：`neotrix-core/src/skill_registry.rs:14`（正典）· `neotrix-gateway/src/skill_registry.rs:157` · `neotrix-multi-agent/src/skill_registry.rs:157` | ⬜ |
| 0.5 | **`maturity_audit()` 接 CI 门** —— 机制已完整实现且**带自愈**（`:484-485` 自动下调声称等级），缺的只是没人调它 | `nt_core_capability_tree/src/registry.rs:459` · 数据源 `.neotrix/capability_registry.json` → `nodes[]` | ⬜ |
| 0.6 | **CI 三断言基座**：① `detector_coverage ⊇ execution_scope` ②注册表唯一性防回潮 ③schema 指纹门（`sha256(DDL)[..12]`，环境派生字段拆独立门） | `security-audit.yml`（实测全文只有 `cargo deny check all`）· 挂靠点 `scripts/ops/` | ⬜ | ⚠️ **2026-09-28 新增门已就位**：`check-fresh-build.sh`(干净克隆必须能构建) + `check-layer-deps.sh --strict`(棘轮) + `check-test-baseline.sh`(账本, 暂建议模式)。**这三道门是本轮新增资产，勿删**

> **0.1 的隐藏工作量**：`UnifiedResponse.metadata`（`stub.rs:292-303`）**已预留**
> `consciousness_state{phi,coherence,gwt_resonance}` + `capabilities_used` + `confidence`，
> 与 L5 意识核**类型级吻合** —— 契约已定好，填值即可。

## 已知债 · LLM 缓存零计量（2026-09-27 实测，影响面大）

`crates/neotrix-types/src/llm_types.rs:104-107` `Usage` 只有
`prompt_tokens` / `completion_tokens` / `total_tokens` —— **没有任何 cache 字段**。
而 `neotrix-core/src/l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs:92`
正在打 `cache_control` 断点（注释 "P0-4 prefix caching"）。

**即：项目在享受 prompt 缓存，却对缓存命中/写入零计量。**
按 AIBrix 结论（`AIBRIX_PREFIX_CACHE_INCLUDE_TOOLS` 默认 true，网关把规范化后的
`tools` 前置进哈希文本，"so requests that share messages but carry different tools
do not look like a full prefix match"）—— **工具集身份是缓存身份的一部分**。
本仓有 **120 个 skill + 28 个 Tauri plugin**，任何一次 skill 集变更都会击穿前缀缓存，
而当前**无任何可见性**去发现这件事。

**已完成**：`nt_io_provider/common/cost_attribution.rs`（新，411 行 / 12 测试全绿）——
内容块分类 + 缓存边界前缀切分 + 分级计价 + **残差公开**。与 `egress_types.rs` 同层（L1，
供 L2/L3 共用）。

**未完成（接线）**：
1. `Usage` 加 `cache_read` / `cache_write` 字段（须 `#[serde(default)]` 保旧数据可解析）
2. `anthropic/anthropic.rs:92` 的 `complete_raw` 处把组装好的请求分类成块并发出归因
   —— 那是「系统提示 + tool schema + 消息在内存组装完成、且从不写入 transcript」的
   **唯一漏斗**，也是全链路归因唯一正确的抓取点
3. provider 侧回传 `cached` / `rewrote`；`write_1h` 上浮系数 1.6 已在 `ProviderUsage` 内

## 1.1 剩余接线 · DnsEgressPolicy 进 shell 层（2026-09-27 定位完成，未执行）

**已完成**：`DnsEgressPolicy` + `DnsVerdict` + 11 测试（`egress_types.rs`，全绿已提交 `11d27d43`）。

**已查明不该动的地方**：
- `osint_bridge.rs` / `osint/dns.rs:74 query_doh` 查 MX/TXT/NS 是**合法侦察职能**，
  默认拒会破坏 OSINT。正确设计是调用方显式授权，而非默认拦死。
- agent `ToolRegistry` 内**无任何 DNS/network 工具**（已全仓 grep 确认）。

**真正的威胁面**：agent 经 shell 跑 `dig`/`nslookup`/`host` 或直连 53 端口。
候选卡点（均有 DNS 关键字，需逐个读现场再下刀）：
- `l3_embodiment/nt_shield/nt_shield_stealth_net/firewall.rs`（565 行，PF + nftables）
- `l3_embodiment/nt_shield/nt_shield_stealth_net/network_pool.rs`
- `l3_embodiment/nt_shield/nt_shield_sandbox/mod.rs`（策略扩展点 :76-138）

**设计约束**：shell 层看到的是**命令字符串**（`dig TXT foo.example`）而非结构化
`(qname, qtype)` —— 需先解析命令提取二元组，再调 `verify_query`。命令解析本身是
注入面，写测试时必须覆盖 `dig @1.1.1.1 TXT x` / `nslookup -type=TXT` 等变体。

## 已知债 · 外部吸收的 JEV 决策面违反 R-P79（2026-09-28 新发现）

三决策引擎建模完整、测试全绿，**却无一条接进生产决策点**（明细见
`DIR-AUDIT-2026-09-27.md` §六）。这是 R-P79（外部技术必须同会话接到生产可用）
的未清偿项：**先接线，后装饰**，故 4.1 标 ⛔ 而非 ✅。

**附带**：`check-truth-surface.sh --strict` 在主工作树报 exit=1 会被误读成 CI 红，
**干净 HEAD 实测 exit=0**。本地噪音，暂不修（避免动他窗在写的文件）。

## 已知债 · NeoBot 工作台 + IM 渠道的验证缺口（2026-09-28 新吸收）

吸收 `DSH-better-sidebar` + `dsh-im` 两个 TS 源，落地 `crates/neotrix-neobot`
**12 个新模块 / 5 张新表**、`apps/neobot-desktop` **97 个注册 IPC**（注册数 53→97）、
前端 4 个零依赖文件。**功能与取舍不在此复述** —— 正典记录：
`docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`。
（注：12 个新模块**不进** `ARCHITECTURE-MAP-ROADMAP-V2.md`，理由见该文抬头「台账归属」：
那条规则是 R-P199，文本只在 `docs/standards/archive/` 的非规范副本里，口径是
`neotrix-core` 的 L1–L6 分层，而 `neotrix-neobot` 是独立 crate、不占 L 层。）

此处只记**没被证明的部分**（数字随补测推进会变，用前复算）：

| 缺口 | 2026-09-28 实测 |
|---|---|
| 桌面 IPC 层零测试 | `apps/neobot-desktop` 的 `cfg(test)` 命中 **0**；254 个测试全在下一层 `neotrix-neobot` |
| 侧边栏渲染从未执行 | **开工时** 13 个 `*Html(): string` 困在 `frontend/src/sidebar.ts`（顶层 import Tauri API），`selftest` **import 不了**；本批正在抽到 `frontend/src/render.ts` |
| Telegram 未碰真平台 | 23 个测试全是手写 JSON 夹具、**零 HTTP**；`API_BASE` 是编译期 `const`，**连假服务器都起不来** |
| 端到端 | **从未**对真浏览器 / 真 `api.telegram.org` 跑过一次 |

**P0 教训（跨模块可复用）**：两轮 P0 抓到的都是**没被测过的那道缝** ——
跨 IPC 边界的传参值写错（`taskId: ""`）、命令**漏注册**（HEAD 时 42 个工作台/IM 命令
写好了却没进 `generate_handler!`，编译与测试全绿而前端一个都调不到）。
这类缺陷对编译器/clippy/`cargo test` **全是绿的**，只有「声明 ⊆ 注册」这类
**机械断言**能抓。**故：IPC 层冒烟测试补齐之前，先别加新功能** ——
缺的那一层正是唯一能对「接错线 / 没接线」发信号的地方。

## ✅ 收口 · 单元测试 57 → 3 例失败（2026-09-28，**已无 flaky**）

账本已棘轮到 3 条（`scripts/test-failures-baseline.txt`）。剩余 3 条**均需产品判断或
阶段级工程**，精确诊断见 `sessions/handoff-test-debt-20260928.md`：

| 测试 | 性质 |
|---|---|
| `kb_primitives::test_node_history` / `test_nodes_as_of_returns_committed` | **schema bug**：`nodes` 主键不含时间维 ⇒ 双时间无法存历史。改复合主键后暴露 `edges` 真外键失效 + `nodes_as_of` 文档与实现矛盾 + 既有 DB 需数据迁移 ⇒ 即 **2.2 真双时间迁移**，已回退不做半迁移 |
| `noise_handshake::full_handshake_matches_official_vectors` | **协议实现 bug（已修且转绿，2026-09-29 合入主干）**：responder 在 IK 模式下无法计算 `es`（对端 static 正是被加密送达的）。该模块无生产调用方且协议名与 spec 不一致，建议先决定「对齐 spec 还是明确降级」 |

**账本已无 flaky** ⇒ `check-test-baseline.sh` 的 `--strict` 现在技术上可用，
但**先只棘轮数据、不转 strict**：需先在 CI 上连跑数轮确认那 3 条不再进出
（历史上 flaky 是按运行变动的，不跑够轮次就转 strict 会立刻误报）。

**另有一项已知未完**：全仓 6+ 处测试用 `set_var("HOME")` 改进程全局环境变量，
与读 `HOME` 的测试竞态。已让受害测试自足（`skill_loader` / `checkpoint`），
但根因未除 —— 正解是给所有 HOME 改写点加共享 `Mutex`
（先例：`agent.rs:506` 的 `TEST_MCP_SERIAL`）。当时因 `cipher.rs` 属他窗在制品未做。


<!-- 以下两节的标题已被本节取代, 仅保留其成因分析 -->

### [已被上节取代] 中途快照 · 57 → 8 例失败（2026-09-28）

修好构建后测试才第一次能跑, 暴露出 57 条失败。逐个诊断后**消除 49 条**, 余 8 条。

**成因分布（关键发现）**：57 条里 **37 条在工作树里早已修好但从未入库**
（与构建错误同一模式）—— 入库即解决。真正需要诊断的 20 条里：

| 类别 | 条数 | 处置 |
|---|---|---|
| 工作树已修未入库 | 37 | 落盘（32 文件） |
| **真 bug** | 6 | 熔断器 3（`is_available` 不转换状态 / 探针配额从未自增 / 半开探针失败累加旧计数致恢复死锁）、`best()` 同分返回最深层 |
| **虚标** | 1 | CAD SelfTest 的 file:line 证据是**被注释掉的代码**，路径目录不存在 |
| **stub 能力缺失** | 1 | `TextEmbedder` 是字节袋不是词袋，任意英文相似度被抬到 0.8+ |
| **测试隔离缺陷** | 1 | checkpoint `temp_dir()` 只用 pid 命名 → 同进程 6 测试共用目录互相踩 |
| **测试断言了不可能/矛盾的结果** | 4 | 见下 |

### 值得记住的 4 类「坏测试」

1. `publish()` 的两个分支**都**返回 `success:false`（实现明写 "Feature not wired"），
   而测试 `assert!(result.success)` —— 在**任何**环境下都不可能通过。
2. `test_warning_level` 注释说「注入轻微方差」，数据却是 19×10.0+1×12.0
   ⇒ std≈0.45，z≈8.7 必然 CRITICAL，与它要验的 WARNING 档无缘。
3. `test_full_pipeline` 注释说「Within expected trend」，传 150.0，而序列
   `100+0.5*i` 的**下一个点恰好是 115.0** —— 那才是「趋势内」。150 是遗留错值。
4. `CadWiringEvidenceSelfTest` 判据只有「非空且含冒号」，于是**注释里的冒号**
   骗过了它。加固为「文件真实存在」后，立刻抓出剩余 4 条证据路径也是错的。

> 这 4 类的共同点：**测试在断言一个不存在的能力**，而不是在验证一个存在的行为。
> 加固校验器比放宽断言更有价值 —— 前者抓出新问题，后者只是让门变绿。

### 剩余 8 条（其中 2 条 flaky）

| 测试 | 性质 |
|---|---|
| `kb_primitives::test_node_history` / `test_nodes_as_of_returns_committed` | **真 schema bug**：`nodes` 主键不含时间维 ⇒ 双时间**根本无法存历史**。已试改复合主键，又暴露 `edges` 真外键 `REFERENCES nodes(id) ON DELETE CASCADE` 失效 + `nodes_as_of` 文档写「latest version」**实现却返回全部版本**。这三项 + 既有 DB 数据迁移 = **TODO 2.2 真双时间迁移**，已回退，不做半迁移 |
| ~~`noise_handshake::full_handshake`~~ | `InvalidState`，噪声握手状态机 —— **已重命名为 `full_handshake_matches_official_vectors` 并按官方向量重写** |
| `nt_core_guardian::test_full_guardian_pipeline` | `results.len() <= 1`，修复编排返回数超预期 |
| `nt_file_ability::selftest` ×2 | 自检依赖真实文件/环境 |
| `proxy_heartbeat::test_heartbeat_twice_rotates` | **flaky**（连续运行间进出） |
| `task_categorizer::test_categorizer` | **flaky**（连续运行间进出） |

**账本已棘轮到 8 条**（`scripts/test-failures-baseline.txt`）。因仍有 flaky，
CI 继续跑**建议模式不拦** —— 会 flap 的门只会训练人忽略它。


### [已被上节取代] 起点记录 · 57 例失败（2026-09-28 实测）
> ⚠️ 本节结论**已过时**：57 例已于本轮全部处理到 3 例，账本见上节。
> 此处仅保留**起点数据与成因分析**作为对照，勿据本节判断当前状态。

修好构建后测试终于能跑：`cargo test --lib -p neotrix` → **12093 passed / 57 failed**
（38 ignored，约 200s）。**这 57 条此前不可见** —— HEAD 长期编译不过，测试从未有过
可验证基线，所以它们是既有债，不是任何一次改动的回归。

按层分布：`l6_meta` 20 · `l4_emotion` 10 · `l1_action` 8 · `l3_embodiment` 7 ·
`l5_cognition` 5 · `l0_substrate` 3 · 其余 4。

抽样 4 条，**全是真实行为断言失败，不是环境问题**：

| 测试 | 断言 | 含义 |
|---|---|---|
| `nt_core_speculative_decoding::test_speculative_decoding` | `speedup_factor > 1.0` | 投机解码并没有更快 |
| `checkpoint_persistence::test_delete_checkpoint_removes_index_and_file` | `save.success` | checkpoint 存不下来 |
| `nt_core_capability::security::ip_check` | `check_ip("192.168.1.1")` | 私有 IP 判定为不通过 |
| `nt_core_aware::test_consciousness_awareness_new` | `ca.is_conscious_bound` | 意识未绑定 |

**为什么不在本轮修**：每一条都要一次产品判断 —— 是代码没实现，还是断言写了从未
实现的行为？猜错会把 bug 固化成"正确行为"。这是需要逐条裁决的工程，不是批量替换。

**门的行为（重要）**：`scripts/check-test-baseline.sh` 记 57 条账
（`scripts/test-failures-baseline.txt`），但 CI 跑的是**建议模式，不拦**。原因：
连续两次运行都是 57 条，但**集合有 3 条抖动**（`checkpoint_persistence` 等
文件系统类为 flaky）。`--strict` 会因抖动误报 —— 一个会flap的门比没有门更糟，
它只会训练所有人忽略它。门的作用是把债变成**可数**的，不是把它变绿。

**判别式修正留痕**：守卫最初写 `grep -qE "^error(\[|:)"`，结果把
`error: test failed, to rerun pass`（测试失败摘要）当成构建错误而拒记账本。
改为只匹配 `error[E####]` / `error: could not compile`。


## 已知债 · 分层依赖违规（2026-09-28 记账，**未解决**；现 94（84 条在 l*_ 层 + 10 条来自此前逃过检查的 `neotrix/` 第二棵树；另一窗口 2026-09-28 纳入该树，**属覆盖面扩大而非新增债**）

`scripts/check-layer-deps.sh` 实测 **92 个 file×pattern 违规点**，横跨 11 类：

```
L1→L2  L1→L3  L1→L4  L1→L5  L1→L6
L2→L3  L2→L4  L2→L5
L3→L4  L3→L5  L3→L6
L4→L5  L5→L6
```

即 L0→L6 的单向依赖在**源码层面**基本没被遵守（`deny(warnings)` 管不到跨层引用）。

**为什么记在这里而不是直接修**：这 94（84 条在 l*_ 层 + 10 条来自此前逃过检查的 `neotrix/` 第二棵树；另一窗口 2026-09-28 纳入该树，**属覆盖面扩大而非新增债**） 是架构级重构（要把 L1 对 L2/L3/L5/L6 的
引用全部改走 `l0_substrate` 门面或下沉依赖注入），不是一轮能收的活，且会牵动
正在被别人编辑的 `nt_io_web/api.rs`、`nt_act_orchestrator/`、`main.rs` 等。

**为什么门要改**：原脚本无 baseline，**恒定失败** —— 一个永远红的门等于没有门，
真正的信号（新增违规）与既有债无法区分。已按 `check-truth-surface.sh` 同一设计
加棘轮：既有债进 `scripts/layer-deps-baseline.txt`（92 条，按 `pattern+file`
记账、**不含行号**以免行号抖动刷假警报），`--strict` 只拦**新增**。
CI 已改跑 `--strict`。

**回归证明**：干净检出上基线态 `--strict` exit=0；注入一条 `L5→L6` 后 exit=1
并精确报出该 file。**门不是空门。**

⚠️ **生成基线必须在干净检出上做**。我第一次在脏工作树生成，把另一窗口未提交的
改动编进了账，导致干净检出上 `--strict` 反而失败 —— 门立刻抓到了这个错误。


## ✅ 已收口 · 「已提交代码引用未入库文件」全家族（2026-09-28）

**症状**：`cargo test` 全绿、`check-truth-surface --strict` 报 0、`ci.yml` YAML 合法 ——
但**新鲜克隆跑不了任何 cargo 命令**。真值只在干净检出上。

**根因一类，四个实例**（`DIR-AUDIT §六/§七` 有完整证据链）：

| # | 未入库的东西 | 被谁引用 | 症状 |
|---|---|---|---|
| 1 | `apps/neobot-desktop/Cargo.toml` | 根 `Cargo.toml:18` 的 workspace member 声明 | workspace 加载失败 |
| 2 | `neotrix-core/benches/{memory_bench,security_bench}.rs` | `neotrix-core/Cargo.toml:224-230` 的 `[[bench]]` | manifest 解析失败 |
| 3 | `fn migrate_legacy` 定义 | `nt-core-capability-tree/src/cli.rs:327` | E0599，且 `neotrix-core:99` 依赖它 ⇒ CI 构建不过 |
| 4 | `models/training/jev_platts.json` | `include_str!`（**编译期**读取） | 干净检出编译失败 |

另加 3 处 blanket 通配规则（`tests/` `examples/` `models/`）静默屏蔽真实源码/数据
—— 危害不在屏蔽大文件，而在**静默**：`git add` 不报错不提示，落差可潜伏任意久。
`models/` 3.4GB 权重**该忽略**，故只精确解禁 `models/training/`。

**收口后实测**（全新干净检出，非工作树）：

```
cargo check --lib -p neotrix            exit=0
cargo check -p neotrix     (全部 bins)   exit=0
cargo test --lib -p neotrix --no-run     exit=0
scripts/check-fresh-build.sh --full      PASS
```

**新门 `scripts/check-fresh-build.sh`**（已接 `ci.yml` 第一条检查）：脏树不是合法
oracle，故脏树时自建 `git worktree add --detach HEAD` 再跑。**回归证明**：修复前
的 `5fcb291b` 上 tier1 得 exit=101。

**方法论（与上一节的 IPC 教训同源）**：「本地能编译」「门是绿的」「YAML 合法」
都不能证明仓库可交付；**只有干净检出 + 真实命令**能。

## 已知债 · 主 CI workflow 曾无法解析（2026-09-27 修复）

`.github/workflows/ci.yml` **在 HEAD 就是非法 YAML**：`Truth-surface gate (ratchet: blocks ...)`
这一行 `- name:` 的值未加引号且含冒号，`yaml.safe_load` 报
`mapping values are not allowed here (line 28)`。

**影响**：`ci.yml` 是主 workflow（12 个 job 中的 11 个在此）。它无法被解析 ⇒
**`check-truth-surface.sh --strict` 这个门从未在 CI 里跑过**。而本项目多份文档
（包括本文件此前）把「`truth-surface-baseline.txt` 基线 0 条 ✅」当作门有效的证据 ——
**基线再准，门没跑就等于没有门**。这与 R-SCAN-3（门记录会腐化）是同一族问题的更严重形态：
不是记录过时，而是**门本身从未接线**。

**已修**：该行加引号；全 14 个 workflow 用 `yaml.safe_load` 扫过，其余 13 个均合法。
**新增** `capability-truth` job（报告态，见「已知债 · 能力成熟度虚标」）。

**遗留动作**：核实 `check-truth-surface.sh` / `check-layer-deps.sh` 在修好 workflow 后
是否真的通过（本次未在 CI 环境验证，只做了本地 YAML 合法性检查）。

## ✅ 已收口 · 能力成熟度虚标（原 24 项）

**逐条考据结论：24 项「零跨层消费者」属实**，降标是如实而非惩罚。
证伪方法：对每个符号查全仓调用方，发现所谓外部消费者全是**同名异物** ——
`merge_pdfs` 撞 `neotrix-types/core/file_parser/pdf.rs:155`（另一个函数）、
`detect_encoding` 撞 `shield/guard/input_gatekeeper.rs:117`（另一个 struct 的方法）、
`xlsx_read`/`OcrEngine` 撞 `mod.rs:33-56` 的 `pub use` 再导出（无进一步消费者）。

**已执行**：
- `audit-maturity --apply`：24 项降标到 C1，每项记 `evolution_log`（可逆：补 evidence 后 re-mature）
- 核实确有跨层消费者的 2 项**带真实 file:line 升到 C2**：
  - `ocr::OcrEngine` → `l2_perception/nt_world/ocr/mod.rs:283,286,314`
  - `image_super_resolution::ImageSuperResolver` → `l2_perception/nt_world/ocr/pdf_to_text_pipeline.rs:17,30`
- 基线复测 **0 项**；`ci.yml` 的 `capability-truth` job 由**报告态转阻塞态**（`--strict`）

**已排除的疑虑（我一度误判并自我纠正）**：曾怀疑 `nt_file_ability::ocr::OcrEngine` 这个
ID 指向不存在的 `ocr` 模块。实为 `nt_file_ability.rs:70` 的 `pub use visual::*;` 链式再导出
（`visual/mod.rs:6` 有 `pub use ocr::*;`），ID 有效。教训同前：先验证再下结论。

## 阶段 1 · 安全封口（3-5 天，事故已真实发生）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 1.1 | **DNS qtype 白名单** + qname 长度上限 + 把 `dns_allow` 从"学习提示"改成"过滤器" | `l1_action/nt_io/nt_io_provider/common/egress_types.rs:14-21` —— `SandboxEgressRule` **只有 `host`/`port`/`allow` 三字段，全文件零 DNS 概念** | ⬜ |
| 1.2 | **"以尝试为检测单元，结果是独立字段"** —— 任何情况下不得让 outcome 衰减 attempt 严重性 | `l0_substrate/nt_core_telemetry.rs`（事件 schema）· `l6_meta/nt_safety_monitor.rs` · `l6_meta/nt_core_guardian/health.rs` | ⬜ |
| 1.3 | **严重性校准对 + 职责分离**（借 cloudflare/security-audit-skill，MIT） | 写入 `RUST-STANDARDS.md` §17.5 或 audit 规则 | ⬜ |
| 1.4 | **非不可宽化策略地板**：把"允许绕过批准的能力集合"变成测试钉死的冻结数据 | `crates/neotrix-neobot/src/nt_policy.rs:75` `evaluate_policy`（现 deny 全集：`human-control:77` `computer-allow:84` `computer-host:90` `workspace-jail:98,106` `unknown-tool:119` `default-deny:121`；`evaluate_extra_deny:137`） | ⬜ |

> **1.1 的行业空白**：深挖 7 个沙箱/安全仓，**没有一个做 qtype 过滤**。
> `microsandbox` 唯一真做 host 侧 DNS 管控（smoltcp，guest 不持 resolver socket）但也无 qtype；
> `CubeSandbox` 的 eBPF `dns_allow` **只学 A 记录**，不在名单内的查询不被拦且 gateway 常放行
> —— **与 OpenAI 2026-09-20 事故同构的隐蔽信道**。

## 阶段 2 · 记忆正确性（1-2 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 2.1 | **supersession 形态**（绕开主键重写，今天可迁） | `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs` —— 5 段即 `:169 snapshot` / `:195 distill` / `:253 classify` / `:306 persist` / `:403 feedback` | ⬜ |。**2026-09-28 前提已勘清**：`supersession` **在记忆库层早已实现** —— `nt_memory_kb/nt_memory_curation.rs:182/238` 的 D2 冲突消解（`UPDATE nodes SET supersedes=?1, tier='cold'`，新者胜出、旧者指向新者、保留证据链）+ 已有测试 `tests.rs:564 test_write_memory_entry_conflict_supersedes_old`；`nt_memory_historian/nt_temporal_facts.rs` 更是一整套「每版本独立 id + `supersedes`/`superseded_by`/`contradicted_by`」的版本链（**91 测试全绿**）。**真正缺的是 experience_tree 自身** —— `EntryType{Pattern,Rule,Defect,Insight,Cycle,Artifact}` 无生命周期形态，`ExperienceEntry` 无 `lifecycle_state`/`supersedes`。路线图 §1.1 要求的 9 个字段里，tree 只满足 `source`，缺 `authority`/`lifecycle_state`/`supersession`/`revocation`/`expiry`。**照 `temporal_facts` 已验证的形态做即可**（每版本独立 id + 指针，绕开主键重写 —— 这正是「今天可迁」的含义） |
| 2.2 | **真双时间四列迁移**（依赖 2.1 先落地） | 同上 `:29 ExperienceEntry` + `l4_emotion/nt_memory/{kb_kb,paged_kv}`。先例：`l0_substrate/nt_core_kb_primitives.rs:188`（即上方 B 类第 1 项） | ⬜ |。**2026-09-28 前提已勘清**：`nt_memory_historian/nt_temporal_facts.rs:41` 的 `temporal_facts` 表**已经是真双时间** —— `valid_from`/`valid_until`（有效时间轴）+ `created_at`（事务时间轴），且用「每版本独立 id」绕开了复合主键。**唯一真正缺双时间的是 `l0_substrate/nt_core_kb_primitives.rs` 的 `nodes` 表**（`id TEXT PRIMARY KEY` 无时间维），那 2 条失败测试卡在这里，与 historian 无关。⇒ 本项应改述为「把 nodes 表对齐 temporal_facts 已验证的形态」，而非「从零做双时间」 |
| 2.3 | **`Provenance` 第二轴**（**不重载 `Source`**）✅ | `experience_tree/mod.rs:101` `Source{Dialogue,Audit,Research,Absorption}` 是**渠道**语义，与"证据等级"正交。已加 `Provenance` 4 变体 + `#[serde(default)]` 字段（**Default=ModelAdded 如实**）+ 6 测试（25 passed） | ✅ |

> **2.2 最值得抄的细节**：`valid_to_precision = 'unknown'` 三态
> （`valid_to IS NULL` + `precision IS NULL` = 持续；`+ precision='unknown'` = 已结束但日期未知）。
> 教科书做法（把文档日期塞进 `valid_to` 当上界）被 utopia 显式拒绝，理由是
> *"看起来像个确定的时间戳；每个读者都得先查精度，而不撒谎才是产品。"*

## 阶段 3 · 可观测与成本（1-2 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 3.1 | **成本归因：在链路上抓** —— canonical block 记录（`zone`/`section`/`bucket`/`tool`/**`skill`**/`role`/`tokens`/`hash`），skills 与 MCP server 作为一等维度 | **唯一插点已存在**：`l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs:92,237`（注释 "P0-4 prefix caching"）= 请求时组装、从不落 transcript 的不可见前缀。已有 `Usage` 类型定义：`crates/neotrix-decision-engine/src/types.rs:304` —— ⛔ **该 crate 已于 `2bbed32c`（2026-09-28）删除**（无生产消费者的死引擎，与 nt_core_capability 的 4,640 行同批）。本行「唯一插点已存在」指**插点位置**（`anthropic.rs:92,237`）仍在，`Usage` 类型需重新定义。 | ⬜ |
| 3.2 | **能力 manifest 加安全信封**（声明式爆炸半径 + 构建期校验） | `.neotrix/capability_registry.json`（形状已对：`provides`/`requires`/`rune_sockets`/`evolution_log`）· 弃用机制已在 `capability_overrides.json`（`deprecated`/`deprecated_reason`） | ⬜ |

> **3.1 为何必须走链路**：系统提示、注入的 tool schema、MCP schema 都在**请求时组装、从不写进
> transcript**，那段前缀"可以占到上下文的一半甚至更多"。**基于日志的归因对最大的成本中心结构性失明。**
> 120 个 skill + 28 个 Tauri plugin 正在这个盲区里。

## 阶段 4 · 决策面与自治（2-3 周）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 4.1 | **JEV 四件套**：场景指纹 / 硬 `call_budget` / 过期 / 非阻塞 worker + 三路置信门 | ⚠️ **前提证伪，推迟**（2026-09-28）。三原语已忠实建模(`types.rs:85/:146/:166/:189`)+ stale 机制齐 + 113 测试全绿，**但三个「决策引擎」全无生产消费者**：`crates/neotrix-decision-engine/` 仅自身测试(gateway 再导出在**无人启用**的 optional feature 后) · `nt_decision_engine.rs:351` 的 `new()` 仅 `:611/:620/:644` 三处**全在 `#[test]` 内** · `nt_mind/decision_engine/` 的 `WeightedScorer`/`DecisionRecommender` **外部引用 0**。**先装饰=造第四份死代码**。依据 `DIR-AUDIT §六`；新欠条「JEV 决策面违反 R-P79 未接生产」 | ⛔ |
| 4.2 | **"未解析"建模为一等状态**（`epistemic: exact \| lower-bound`） | `nt_core_capability_tree/src/node.rs` · 雏形已在 `registry.rs:15-18 MaturityFinding{claimed,supported}` | ✅ |。**2026-09-28 已完成**（`4c5f6307`）：新增 `epistemic::Epistemic{Exact\|LowerBound}` 并挂到 `MaturityFinding`（`registry.rs`），Default=**LowerBound**(fail-closed)，只有节点显式认领 `metadata.evidence_exhaustive` 才 Exact（系统无法证明「没有更多证据」，只能采信认领，故不做推断）；`cli.rs` 输出行加 `[exact\|lower-bound(未解析)]` 让未解析在 CI 可见。⚠️ 干净检出上该 crate 当时编译不过，故这项的验证是在脏树跑的——已由 `9bbc9dc2` 修复，建议下一轮在干净检出上复跑 `audit-maturity --strict` 确认 |
| 4.3 | **自治循环 git 化 + 固定墙钟预算**（`results.tsv` 5 列，变好推进/变差 `git reset`） | `Makefile`（现有目标 `run:4` `project-locate:151`）· 反馈判据 `experience_tree/mod.rs:403 feedback()` | ⬜ |
| 4.4 | **覆盖率账本状态机**（hunters 不能写自己的覆盖率） | `l3_embodiment/nt_shield/nt_shield_audit/`（10 文件）· `l6_meta/nt_core_self/self_audit.rs` · 基线 `scripts/truth-surface-baseline.txt`（现 0 条 ✅） | ⬜ |

> **4.1 最高价值的一条方法论**（jev-drone 自评，MIT）：
> *"a state-design bug, not a model failure"* —— JEV 从不选 `climb`，因为状态是 5 个**水平**距离扇区、
> **没有垂直信息**，"飞过去"根本不可推断；加入障碍顶边高度后 `climb` 从"从未被选"→ **p=0.93**。
> **规则：如果一个决策从不触发，先审状态形状，再审模型。**
> ⚠️ 同时记住反面：该仓头条数字是 **n=1 单次运行**，3-seed 配对比较**无优势**。别把单次胜利当机制有效性。

## 阶段 5 · GUI 与 MCP（2-4 周，可全程并行）

| # | 任务 | 支脉节点 | 状态 |
|---|---|---|---|
| 5.1 | **元素寻址 GUI 驱动**（B 路：a11y 树优先，坐标仅 fallback） | 现状：`l3_embodiment/nt_computer.rs`(477 行，**filesystem/process trait，不是 GUI**) · `nt_computer_fleet.rs`(262) · `crates/neotrix-neobot/src/nt_computer.rs`(155，`NoopBackend` 唯一后端) · `l1_action/nt_io/nt_io_desktop/` **只有 2 文件**（`mod.rs` 9 行 + `updater_signing.rs`） | ⬜ |
| 5.2 | **MCP per-request capability 协商** + 客户端提生产 | 协议层 2026 已稳定：每请求必带 `protocolVersion`+`clientCapabilities`，缺字段 `-32602`；`MissingRequiredClientCapabilityError`(`-32021`)。本地：`l1_action/nt_act/mcp_protocol/`(775 行 ✅) · `l1_action/nt_io/mcp_server.rs:20`（内存 Vec，**无 wire transport**）· **客户端 `McpRegistry` 还在 `neotrix-core/src/agent.rs:566` 的 `#[cfg(test)]` 块里** | ⬜ |。**2026-09-28 前提已勘误** —— 原文说「客户端 `McpRegistry` 还在 `neotrix-core/src/agent.rs:566` 的 `#[cfg(test)]` 块里」，**这是错的**：`agent.rs:496/505` 那两个 `#[cfg(test)]` 只挂在**单个测试辅助项**上（`reset_global_mcp_for_tests` / `TEST_MCP_SERIAL`），`McpRegistry` 位于 `pub mod tool {`(agent.rs:420) 之内 = **生产面**，且**已接线**：`entry/interactive.rs:94/108/111` 构造、注册 stdio 服务器、把工具灌进真实 `ToolOrchestrator`；`entry/headless.rs:58` 亦用。**真缺口是**客户端不发 `protocolVersion`/`clientCapabilities`（已在 `agent.rs` 与 `mcp_server.rs` 核实命中 0）|
| 5.3 | **无 API 重放**（审计路径） | `crates/neotrix-neobot/src/nt_audit.rs`（**全仓 `replay` 零命中**）· CLI `bin/neobot.rs` 已有 `audit` 子命令 | ⬜ |
| 5.4 | **UI 组件 + 动效 token** | ⛔ 原引 `src-tauri/frontend/src/canvas/`(9 文件/1793) —— **`src-tauri/` 已于 `5c02e738`（2026-09-28）归档 599 文件**，该路径**不存在**。⇒ 桌面 UI 现无源码可改；本条需**先定去留**（重建前端 or 接受无 UI）。候选站移植评估仍有效：**SolidJS**（5 个候选 UI 站里 4 个是 React，需 1-3h/个移植）；`rareui.com` **已死**（HTTP 402 `DEPLOYMENT_DISABLED`，见「已废止」表） | ⬜ |

> **5.2 别投钱的部分**：registry 只是元数据 —— 其自身 roadmap 白纸黑字
> *"Unified runtime: Not solving how servers are executed"* · *"Quality rankings: No built-in server
> quality assessments"*；schema 里**根本没有 `capabilities` 字段**。**行业自己都还没做，别自建。**
> 长任务已被移出 core 到扩展（SEP-2663），协议**无服务端 task store**。
> 2026-07-28 弃用 `roots`/`sampling`/`logging` → **客户端必须处理三个协议世代**。
>
> **5.4 该建的那一叠**：`beautifului.dev`(MIT) + `ui.shadcn.com`(MIT) 是唯一连贯的一对。
> **别把 beUI 动效混进 transitions.dev 的 token scale** —— 会正好产出 `transitions.dev refine`
> 命令存在的意义所在的那种 ad-hoc 硬编码时长债务。

## 成本感知路由 —— 2026 全行业空位（强依赖 3.1）

深挖 8 个编排仓的模型选择机制清点：Orca 的 per-worker `--model/--effort`（手动覆盖）、
AX 的 `Model` CRD（凭据包不是选择器）、MAF 的 `MagenticProgressLedger`（进度感知非成本）、
dsh 的 `TeamMemberSnapshot.provider`、Symphony 的单一固定 `codex.executable`。
**没有一个有价格表、token 成本模型，或把成本/质量前沿放进路由路径。**
Symphony 记 token 与 rate limit —— 但**只用于显示，从不用于路由**。

**顺序是强依赖的：先能归因（3.1），再能路由。**

---

## 编排隔离单元的实测（决策依据，非待办）

8 个编排仓**无共识**，实测如下：

| 隔离单元 | 谁在用 | 判定 |
|---|---|---|
| 一进程一 agent | **8/8 无异议** | 不是选择，是地板 |
| **一 worktree 一任务** | 只有 Orca（worktree-native）。**Symphony SPEC.md §9.3 明文反共识**："The spec does not require any built-in VCS or repository bootstrap behavior" | 有争议 |
| 一容器一任务 | 只有集群派（AX / CubeSandbox），非谈判项 | 集群专属 |
| 共享队列 + 租约 | **没有一个有分布式租约**，全是本地 FS 存活探针或内存 claim | 空白 |

**NeoTrix 已做对**：`l1_action/nt_act/nt_act_workspace_isolator.rs`(719 行，git-worktree-per-task)
正是唯一有真实共识基础的那一档。**别动它。** 缺的是 reconciliation，不是 worktree。

---

## 附：已废止，不得再实现

| 来源 | 原因 |
|---|---|
| `rareui.com` | 站点已死：HTTP 402 `x-vercel-error: DEPLOYMENT_DISABLED` |
| `ARCHITECTURE-EVOLUTION-ROADMAP.md` | 零引用，55 行 |
| `FUSION-ARCHITECTURE.md` | 零有效引用，其"下一步"含**已被证伪**的"解决预存编译错误" |
| `ARCHITECTURE.md §1-§12` | 已被 §13（neobot 融合，2026-09-26）推翻。读 §13 起的实测部分 |
| `protocol/`（已删） | 312 行 NIP-01 事件总线，从未编译（不在 `lib.rs`）→ 那 7 个测试**从未真正跑过**；Nostr/NIP-01/Buzz 在 914k 行代码库零足迹。git history 可追回 |
| `neotrix-core/src/adapter/`（已删） | 7 行陈旧桩，自述功能已并入 `nt_io_provider`（后者已完全消失，坐实合并残留） |
| `nt-lang` 的"删掉"建议 | **已作废**（2026-09-29 更正）：本行原记「已自我纠正…补 `[lib]` 或接线，**不要删**」，但该 crate **已于 `2bbed32c`（2026-09-28）连同 `neotrix-decision-engine` 一并删除**（5 文件/273 行，删前状态为「只有 `[[bin]]` 无 `[lib]`、0 个 manifest 依赖」）。⇒ 「不要删」的裁决晚到一天，被删除覆盖。**空壳残留**：`skills/crates/nt-lang/` 目录仍在（无 SKILL.md）且 `skills/index.json` 仍登记 `crates/nt-lang` ⇒ 下一条待清 |

## 晶体核心 · 12 个 HF 数据集吸收 + 判别力实测（2026-09-28 第 5 次会话）

> 统一交接：`sessions/handoff-crystal-consolidate-20260928.md`。**建议由单窗口收口**，
> 下列 ⬜ 项分散在数据侧（已完成待验）与 Rust 侧（未编译）两处，混着做容易互相踩。

### ✅ 数据侧（已完成并实测，勿重做）

| 项 | 实测值 |
|---|---|
| 活库 | **69,840 条 / 213 茧 / 29 域 / 56.6MB**（吸收前 64,674 / 205 / 22 / 52.5MB） |
| 图体检 | `nt_graph_audit` **VERDICT: PASS**（重复/非法类型/悬空/非M-id/超限茧/键不符 全 0，往返字节一致） |
| 逐域验收 | 7 个新域 **全 PASS** |
| **monoculture 未恶化** | 全库最大入度 **171 吸收前后完全不变**；新增记忆入度 max **4** / 零孤立 / 入度>8 者 **0** |
| 数据质量 | glue **0** 条 `test` 占位标签（未污染）；Open-Jev **431/431** 带真实对话；全库 content 重复 **0** |
| 备份 | `~/.neotrix/crystal_core/cocoons.json.bak.hfbatch.20260928-134857` |
| 已提交 | `bc3bc100`（Python 1,616 行/6 文件）、`85a7f97d`（RFC v4 + 门记录） |

**处置矩阵**（每条附实测依据，详见 RFC v4）：TB 级 3 个按用户指令跳过；`Yootta`
**401 gated 物理不可达**（用户点名的 cc-by-nc-sa 项，**不是本轮的选择**）；
`malcolmrey/various` 实测**仅 33 行纯图像 + wtfpl** ⇒ 零蒸馏价值，主动放弃
（**实测推翻自己原计划**）；`glue` 只吸 `train`（是基准，吸 `test` 会真实污染）；
`Fable` 吸 `full`(10000x) 而非已在库的 `lite`(5000x)。

### ⬜ 待办（按依赖排序）

| # | 项 | 阻塞于 | 说明 |
|---|---|---|---|
| 1 | **编译验证晶体 Rust 侧** | 内存门 | **8 错已修至 1，最后 1 个已改但未验证**。见交接文档「精确状态」 |
| 2 | 跑 58 个目标测试 | #1 | `sync_dedup` / `chain_persist` / `nt_premise_selector` / `verify_incentive` / `nt_graph_index` |
| 3 | `live_graph_index --ignored` | #1 | **唯一缺证**：`CocoonStore::load()` 对 69,840 条核心的加载实证。52.5MB 时证过一次，吸收后**未重证** |
| 4 | 提交晶体 Rust 侧 | #1 | 5 改 1 增**全部未提交**（见交接文档文件清单） |
| 5 | 提交 `nt_jev_live_eval.py` | — | 本轮新增，验证充分，**可立即提交** |
| 6 | 修 `keywords()` 的 CJK 失明 | — | 见下「实测发现」，**影响面远超晶体核心** |
| 7 | 接 judge 模型 | #6 | 数据已到位（jev-choice 431 + decision-calib 600 + agentic-trace 816） |
| 8 | 32 个未跟踪 ops 脚本 | 需用户决策 | `scripts/ops/` 整个目录未入库；我只提交了自己那 6 个 |

### 🔴 实测发现：`keywords()` 对中文近乎失明（跨模块影响，不止晶体）

`CrystalConsciousness::keywords()` 按**空白/标点**切分 ⇒ 中文整句只成 **1 个 token**：

```
keywords("我的支付一直失败收不到验证码") -> ['我的支付一直失败收不到验证码']   # 1 个
keywords("支付网关")                     -> ['支付网关']                      # 永不相交
```

**且无词干还原**：`{"invoice"} & {"invoices"} == ∅` —— 纯复数差就让判分器归零。

**真实标答任务实测**（`nt_jev_live_eval.py`，339 条可用 jev-choice，随机基线 38.1%）：

| 上下文 | top-1 | MRR | 零交集 |
|---|---|---|---|
| 仅对话(state_json) | 43.9% | 67.9% | 265 |
| 仅题干 | 42.7% | 67.6% | 206 |
| 对话+题干 | **45.8%** | 69.4% | 196 |
| 无上下文(对照) | 38.1% | 63.8% | 339 |

**最好组合仅比随机高 7.7 个百分点，78% 的条目与上下文零词面交集。**
⇒ 不是「分数不够好」，而是「**这个任务上纯词法几乎没有信息可用**」——
**词法路线在此任务无实用价值，judge 必要**。

> ⚠️ 勿与 §v4.2 混淆：「补上对话后 79,116 行不再塌缩成 ~80 条」是**区分性**结论
> （由去重实测 2400 行中 2318 行自重复单独成立）；上表测的是**词法判别力**，
> 两者是不同性质。上表**不能**推出「对话无用」，只说明纯词法用不上对话。

### 📌 可复用教训（本轮付出代价换来的）

1. **干跑覆盖不到写盘** —— 取数/去重/上限都能干跑验，但写盘路径在写之前就 `return`。
   首版 `NameError: mtype` 就死在这。**写盘路径必须有单测**，
   否则「干跑全绿」是虚假安全感。已抽出 `make_record()` 由 selftest 直接调用。
2. **先勘察后吸收** —— 78TB 的清单若直接开灌，会浪费数小时且污染 52MB 核心。
   一次只读 survey（12 个 API 调用）就把可执行性、许可、gated、体量全部定死。
3. **体检工具报错比不报更危险**（R-SCAN-3 精神的又一例）——
   `nt_cocoons_verify_absorb.py` 用 `len(raw)/1e6` 报体积，而 `raw` 是已解码 `str`，
   字符数 ≠ 字节数，把 56,637,045 B 误报成 56.0MB（真实 56.6）。
4. **门记录必须带核实时间戳** —— AGENTS.md 写「本轮仅改文档未改码故沿用同值」，
   而本轮**确实改了 Rust 码**，该理由已不成立 → 已重测并补记。
5. **评估指标要给机会水平** —— 全零交集时若「同分保序取第一个」，等于白送 index 0
   一个正确（而金标常在 index 0），准确率虚高。须按 1/n 计入。

<!-- 路径说明（2026-09-28 追加，**不重写上文**）：上文出现的 `src-tauri` 与
     `apps/neobot-desktop` 多为**当时的历史路径**。`src-tauri` 于 `5c02e738` 归档、
     `apps/neobot-desktop` 于 `d5413335` 删除，桌面 App 统一到独立仓
     `~/Downloads/Neo/neobot`。改写历史台账等于伪造当时的事实，故只加此注记；
     仅**活指示**（可直接复制执行的命令、指向已删文件的未完成任务）被逐条修正。 -->
