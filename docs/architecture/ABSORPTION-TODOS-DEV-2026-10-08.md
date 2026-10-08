# 吸收：todos.dev（任务驱动的人+agent 工作区）

> 源：`https://todos.dev/`（2026-10-08 抓取首页文案）。未取代码，仅取设计形状。
> 与 `ABSORPTION-BATCH-2026-10-08-NEOBOT-34.md` / `ABSORPTION-TASKLIST-2026-10-08.md` 同范式（NTS-B10）。

## 1. 它是什么

- todo 驱动的人机协作工作区：goals → todos → 并行 agents → 可视化进度。
- **一个 Chief agent 统一编排**：拆分任务、分配 agent、跟踪进度；Chief 不直接干活。
- 每个 todo 一个 agent、各自独立 worktree、互不干扰。
- 任意 coding agent 可经 MCP 接入（Claude Code/Codex/Cursor/OpenCode/dsh 等）。
- 过去 todos 可搜索（非 session-centric 的会话黑洞）。

## 2. 四字段映射

| Pattern | NeoTrix 落点 | 判定 | 接线裁决 |
|---|---|---|---|
| Chief 单点编排 | `nt-core-capability-tree` 的 dispatch / `l6_meta` agent loop | 强化（Chief≈我们没有的「编排面」） | 📋 路线图：给 neobot 加「唯一编排入口」语义 |
| per-todo 独立 worktree | `.worktrees/`（AGENTS.md §2 已有） | 强化（语义一致，零新增） | ✅ 已有，补文档对照即可 |
| past todos 可搜索 | KB `nodes`/FTS（854k 行已建） | 强化（我们更强） | ✅ 已有 |
| Usage 追踪 | `nt_store_quota`（QuotaWindow）+ A1 路由额度感知 | 强化 | ✅ 已落地（9ebe90f7/74961108 之后本会话 A1 接线） |
| Connections（Linear/Jira/Notion/Slack） | neobot channels（A2 dsh-im） | 强化 | 📋 同 A2 |
| Schedules | 未查到现成等价物；neobot routine | intake | 📋 评估 |
| PWA 手机端 | tauri/桌面发布（C2） | intake | 📋 同 C2 |

## 3. 对 neotrix code map 路线的直接增量

todos.dev 的「任务→agent→worktree→进度可视化」四元组，正是我们
`docs/architecture/CODE-TOPOLOGY.md` + `.project-map/` 网络要承载的运行时语义：

1. **把 `nt-registry-determinism.sh` 的「318 节点/43 边」从静态台账升级成
   「任务图」**：每个吸收批次、每个 ROADMAP 条目作为一个节点，
   `absorbed_capability` 四元组作为边属性（source→branch）。
2. **`nt_map_reconcile.py` 断言面等价于 todos.dev 的「进度看板」**：
   35 条谓词的 PASS/VIOLATED 就是全域 code map 的健康面板。
3. **chief 语义 → `nt_find.py`（84 条意图索引）**：意图→工具路由即「Chief 拆任务」
   的本地等价物；缺的是「单点入口」的文档化声明。

## 4. 裁决

- ✅ 不新增模块；todos.dev 所有机制在我方均有等价物或更深实现。
- 📋 唯一可执行增量：把 §3 三条写成 `CODE-TOPOLOGY.md` 的「运行时语义」小节
  （下一轮 code map 刷新时并入）。
