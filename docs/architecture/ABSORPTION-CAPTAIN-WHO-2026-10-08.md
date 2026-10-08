# 吸收：`Tiga001/Captain_Who` → neobot（2026-10-08）

> **入参**：裸 URL `https://github.com/Tiga001/Captain_Who`，无其他说明文字。
> **依据**：`NEOTRIX-STD-1.0.md` **NTS-B10**（B10.1–B10.4）· 操作面 `skills/external-absorption/SKILL.md`「URL-only 熔炼模式」。
> **取法**：零克隆取源（GitHub API + `raw.githubusercontent.com` + webfetch）；本批**未取逐字代码**，只取设计形状。

---

## 0. 信号初筛

【实测】`api.github.com/repos/Tiga001/Captain_Who`（2026-10-08 13:5x CST）：

| 字段 | 值 |
|---|---|
| stars / forks / open_issues | 20 / 1 / 0 |
| archived / disabled | false / false |
| pushed_at / created_at | 2026-10-07 / 2026-07-07 |
| license | **Apache-2.0**（spdx）⇒ 可取设计，注意 attribution |
| language / topics | Rust / agent, agentic-ai, agentic-workflow, agents |
| default_branch / size | main / 44 MB（Electron 桌面：React + Rust core-server + protocol fixture）|
| 根目录文件形状 | `Cargo.toml` + `crates/{core,core-server,mcp-client,protocol-rs}` + `src/` Electron + `docs/` 架构文档全套 |

⇒ **放行**：可以设计层吸收。`pushed_at` 近 24h，仍在主动演化。

---

## 1. 熔炼成束（上下文工件）

| 束 | 来源（main 分支） | 取舍 |
|---|---|---|
| README | `README.md`（12 KB 中文） | 事实：本地优先桌面 AI 助手：对话+文件+终端+内置浏览器+子智能体并行；Skill 安装（GitHub/本地）、MCP、子智能体模板、权限分级、`Thinking…` 流式。 |
| 系统边界 | `docs/architecture/overview.md`（~7 KB） | Renderer/Main/Core Server 三进程分工：**Renderer 只见类型化 Host API**、Main 持有 OS/Electron 权限、Core Server 持有 Agent 业务+SQLite+多数安全状态机；stdin/stdout 行分隔 JSON-RPC 2.0。 |
| Agent runtime | `docs/architecture/agent-runtime-and-providers.md`（~头部段） | Provider-neutral Tool 循环 / 上下文容量判断 / **Checkpoint** / Tool 投影 / 取消与 Steering；Provider 只管 wire payload。 |
| 上下文管理 | `docs/architecture/context-management.md` | 逻辑日志 = `messages + Trace/model-context journal + active compaction head`，所有消费者经 `ContextAssembler` 派生；不原地改、只追加终态事实。 |
| Multi-Agent | `docs/architecture/multi-agent.md` | **父子树 + Mailbox + Wake + receipt + 事件**；每次真正执行仍交给唯一 Agent Loop；Agent/Turn/Runtime 三概念分离；子会话 observer-only；**先写 SQLite 再发通知**；未知外部副作用绝不自动重放 ⇒ `outcome_unknown`；Turn 不删 Agent，可 `followup_task` 再叫醒。 |

仓内其余子系统（browser-automation / command-sessions / approval / output archive / storage lifecycle / threat model）本批**登记不读**，待清单触发时取。

---

## 2. 化为已有（四字段矩阵 · 人工 grounding，不凭关键词）

| Source | Pattern（机制） | NeoTrix 映射节点 | 强化/新增 | 消费者（R-P79） |
|---|---|---|---|---|
| multi-agent.md | Agent 父子树 + Mailbox/Wake/receipt/事件；子会话 observer-only | neobot `nt_channel_serve` worker 并行 + outbox 投递 | **强化候选**（我们已有 worker 池雏形，但无 registry 记录子会话身份） | `nt_channel_serve.rs` + `nt_store_convos.rs`；**接线裁决 = 路线图（P1）**：并发上限 + 子会话 observer 只读投影 |
| multi-agent.md | **结果/状态/cursor 先落 SQLite，再发通知**；未知外部副作用 `outcome_unknown` 不自动重放 | neobot `nt_channel_dispatch` outbox + `TurnStatus` | **强化**（outbox 已存在，见 N1/N2 收口）⇒ 缺的是 `outcome_unknown` 语义位 | `nt_store/` 与 `nt_channel_dispatch.rs`；可加一列 `outcome_unknown` |
| multi-agent.md | 同一 Agent 同时最多一个活跃 Turn；根 Human Turn 与子 Wake Turn 共用进程级并发上限 | `nt_channel_serve.rs:283` 每消息一线程（N6.6 债） | **新增候选**（我们目前无上限）⇒ 这是**本轮最该抄的形状** | `nt_channel_serve` 有界池 + 令牌租约 |
| agent-runtime + context-management | 逻辑日志 = messages + journal + **active compaction head**；不原地改 | `nt_loop_core` compact_context / `maybe_compact_context`（✅ 已接生产，OPEN-DEFECTS #6 封口） | **强化候选**（我们已有 budget 门+降级记账，但缺持久 compaction head） | `nt_loop_step.rs` 持久化 head 列 |
| agent-runtime | Checkpoint / Tool 投影 / 取消与 Steering | `nt_loop_checkpoint` / `nt_turn_ledger` | 已覆盖 ⇒ **登记，不新增** | — |
| overview | Renderer/Main/Core Server 信任边界 + JSON-RPC 2.0 行分隔 | neobot `nt_channel_serve` + `nt_cli` envelope | 已覆盖（`nt_cli` 已成束）⇒ **登记，不新增** | — |
| docs/subsystems/* | Skills/MCP/子智能体模板 | `nt_skills_catalog`/`nt_mcp_client`（neobot 侧已有对应能力） | 已覆盖 ⇒ 登记 | — |

> ⚠️ 判定纪律（本仓教训）：以上「强化候选 / 新增候选」**未经代码证据即下锤** ⇒ 本轮全部降级为**路线图**，禁止直接改代码。真正开工的每一项必须先 `grep` 现场确认调用点，再立 TODO。

---

## 3. R-P79 接线裁决（本批）

| 裁决 | 机制 | 落点 |
|---|---|---|
| 📋 路线图 P1 | `nt_channel_serve` 有界 worker 池 + observer-only 子会话 + `outcome_unknown` 语义 | `crates/neotrix-neobot/src/nt_channel_serve.rs` + `nt_store_messages` |
| 📋 路线图 P1 | 持久 **active compaction head**（紧凑化头位落库，ContextAssembler 只读既存前缀） | `nt_loop_step.rs` + `nt_store_messages`（新列，幂等 ALTER） |
| ✅ 已覆盖不做 | JSON-RPC 行分隔、outbox 先写后发、MCP catalog、Skills catalog、Turn 并发上限语义 | — |
| ❌ 拒绝 | 子 Agent 可被用户直接对话、Usage 在树层重聚合（违反不变量 #6）、Provider wire 污染 application 层 | —— |

---

## 4. 下一窗口开工清单（按优先级）

1. **`nt_channel_serve` 有界池**：每渠道一个 `tokio`/线程信号量（或 `sync_channel(N)`）+ 超时兜底；验收：100 条连灌不溢线程，outbox 出站仍按序。
2. **子会话 observer-only 投影**：父会话页只读 `origin=agent` 的 `messages` 行；验收：父页不泄露子会话正文。
3. **`outcome_unknown` 列**：ledger 加 `outcome_unknown INTEGER DEFAULT 0`；恢复路径凡不能证明安全即置 1。
4. **持久 compaction head**：`messages` 加 `compaction_head_seq` 列，`maybe_compact_context` 写 1 行摘要后同步更新；ContextAssembler 下次只读 head 之后。

> 本批只落**文档 + KB 节点**，未改 `.rs` ⇒ `cargo test` 无需重跑。代码项进 TODO/交接后按「先测后接线」循环。

---

*落账：KB 节点（见会话补充）· `docs/architecture/ABSORPTION-CAPTAIN-WHO-2026-10-08.md` · TODO N7「可执行」指针已加。*
