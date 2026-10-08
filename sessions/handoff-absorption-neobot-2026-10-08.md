# Handoff 2026-10-08（absorption-batch-neobot 窗口）

## 1. 会话标识

- 窗口：批量吸收 + neobot 完善（space-bunny-free）
- 分支：`feat/multi-agent-absorb-2026-10-08`（**中途被其他窗口切过分支**：A4 提交 `9ebe90f7` 在 `feat/capability-absorb-20260828`，其内容已被本分支继承）
- 吸收对象：用户 34 条 URL + trendshift.io 榜单 + todos.dev + karotte 接线

## 2. 本会话做了什么

| 类别 | 产出 | 证据 |
|---|---|---|
| 吸收批次 | `ABSORPTION-BATCH-2026-10-08-NEOBOT-34.md`（34 源，25 插入/9 去重）、`ABSORPTION-BATCH-TRENDSHIFT-2026-10-08.md`、`ABSORPTION-TODOS-DEV-2026-10-08.md` | KB `~/.neotrix/knowledge.db`，FTS 同步 |
| 任务清单 | `ABSORPTION-TASKLIST-2026-10-08.md`（A1–A8/B1–B3/C1–C3） | TODO.md 顶部权威入口已指向它 |
| 接线代码 | A4 `nt_judge::FastPolicy` · B1 `nt_agent_eval` · A1 `nt_routing::RouteMode::Quota` · A2 `nt_channel_wecom` · B2 `tool_hook` + `checkpoint_tool_hook` · A3 `nt_llama::ssd_offload` · A8 `nt_agent_home` · A5 `html_to_markdown_via_mdream` · B3 `nt_reexplore` | 各带测试（详见下） |
| 机制加固 | `nt_topology.py` 手写段保留区（`<!-- MANUAL-BEGIN/END -->`） | `160d8c75` |
| 台账 | `capability_registry.json` +8 节点 +1 边（318→326 / 43→44） | `nt-registry-determinism.sh` PASS |

## 3. 测试证据（本窗实测）

| 命令 | 结果 |
|---|---|
| `cargo test -p neotrix --lib nt_judge` | 16 passed |
| `cargo test -p neotrix --lib nt_agent_eval` | 4 passed |
| `cargo test -p neotrix --lib nt_reexplore` | 3 passed |
| `cargo test -p neotrix --lib nt_nexus::checkpoint` | 12 passed |
| `cargo test -p neotrix --lib nt_io_agent_loop` | 22 passed |
| `cargo test -p neotrix-neobot --lib nt_routing` | 9 passed |
| `cargo test -p neotrix-neobot --lib nt_channel_wecom` | 3 passed |
| `cargo test -p neotrix-neobot --lib nt_llama` | 12 passed |
| `cargo test -p neotrix-neobot --lib nt_agent_home` | 2 passed |
| `cargo test -p neotrix --lib doc_parse` | 9 passed |
| `cargo check -p neotrix --bench agent_eval` | Finished（dev profile） |

## 4. 诚实未完成项（勿当已完成）

| # | 项 | 阻塞 |
|---|---|---|
| F4 | `cargo bench --bench agent_eval` 真实 Criterion 数 | 两次被 30min SIGTERM（release 双 crate-type + codegen-units=1 超预算） |
| F5 | mdream golden diff | 本机无 `MDREAM_BIN`（npm 有 `mdream@2.0.0`） |
| F6 | mubeng checker→rotator | 仓内 ProxyPool 无生产消费者 ⇒ intake |
| F7 | rea MCP spike | 需独立 worktree + `npx rea-agents` |
| F8 | `tool_hook` 生产组合点 | 仓内**零生产 AgentLoop 构造点**（仅测试）⇒ 机制闭环但零消费者 |
| F9 | C2 签名/公证 | 需 owner 凭证 |

## 5. 经验已吸收

cycle **2058**，7 条落 `kv_store(experience)/branch_2058*`（全部 NT-META）：
许可探针双件、排名只当信号、无消费者不接线、编辑后必复读、依赖倒置闭包、生成文档保留区、子代理 cancel 兜底。

## 6. 收工自查（AGENTS.md §8）

- **worktree 去向**：本窗**未创建任何 worktree**。`nt_worktree_gate.sh check` 报的
  `.worktrees/merge-b`（带未提交改动）**属他窗**，未碰；target 累计 5233M 可回收但**未删**（他窗可能仍在构建）。
- **未提交改动去向**：本窗所有改动均已 `git commit --only` 提交（HEAD `8199f15e`）。
  工作区仍显示的 M 文件（`.neotrix/capability_overrides.json`、`crates/nt-core-capability-tree/*`、
  `neotrix-core/src/l1_action/nt_act/*`、`sessions/*` 等）**全部属他窗在途 WIP**，未暂存、未提交、未还原。
- **知识库写入**：KB 写在 `~/.neotrix/knowledge.db`（仓外，git 保护不到）。

## 7. 下一个窗口的注意事项

1. 分支当前是 `feat/multi-agent-absorb-2026-10-08`，与 `feat/capability-absorb-20260828` 并行；
   A4 提交在后者、内容已在前者，**不要重复 apply**。
2. `CODE-TOPOLOGY.md` 维度 6 是手写段，已被保留区保护；重跑 `nt_topology.py` 不再丢失。
3. 十个 intake 裁决（A6/A7/B2 接线/B3 接线/F4/F5…）已逐条写进 `ABSORPTION-TASKLIST-2026-10-08.md`，
   续做前先读它的「接线裁决」行，避免把已裁决的 intake 重做成代码。