# 吸收：`multica-ai/multica` → neobot（2026-10-08）

> **入参**：裸 URL，配合用户说明「吸收完善 neobot 对应的能力」。零克隆； license= Apache-2.0（raw `LICENSE` 首行已核）。

## 0. 信号初筛
| 字段 | 值 |
|---|---|
| stars / forks / issues | 52,149 / 6,792 / 1,829 |
| archived / disabled | false / false |
| pushed_at / created | 2026-10-08 / 2026-01-13 |
| license (raw) | Apache License 2.0 |
| language / branch / size | Go / main / 155 MB |
| top dirs | apps/ packages/ server/ deploy/ scripts/ docs/ |

## 1. 定位定位
**Multica** is a source-available board workspace: assign issue to agent, agent picks up, reports progress/blockers, hands back for review; runtime daodaemon on your machine; runtimes Army = Claude Code/Codex/Cursor/...; squads, skills, skills-lock.json.

它是「agent 作为队友」的产品形状，**非代码生成库**；吸收进 neobot 应取的是设计形状，不是逐行 Go/TS 代码（许可明确是source-available, license 文本经 Apache-2.0）。

## 2. 四字段矩阵（人工 grounding）

| Source | Pattern（机制） | NeoTrix/neobot 映射节点 | 强化/新增 | 消费者 |
|---|---|---|---|---|
| README/VISION | **Issue→Assign→Progress→Blocker→Review →PR** 闭环事件流 | `nt_store_tasks`（AgentTask）+ ledger + `nt_channel_serve` | **强化候选**（任务聚合表与账本已有；缺少 Pull进程的「评论/review 回执」写入路径） | `nt_agent` 落 `steps/ledger`，扩展 `steps` 字段=事件流 |
| README | **Daemon runtime**： agent 在持久进程里执行，事件流留言 | `nt_channel_serve` + `refresh interval` worker pool（P1-1 有界池已落） | **强化候选**（已有 daemon 语义；缺中央「board」接口） | `nt_channel_serve.rs` worker pool |
| README | **Skills playbook**： solved problem 变成每个 agent 可读取的 playbook | neobot `nt_skills_catalog` / skill-market | **已覆盖，登记** | no action |
| README | **Squads**：一个 team 多 agent，一个 leader 分配 | `nt_routing::RouteGroup/RouteMode::Quota` + ledger key_env | **强化候选**（已有路由组与配额；缺 assignment 委派单行层） | `nt_agent`/`nt_routing` |
| CLI_AND_DAEMON.md | CLI⇄daemon 长连接+幂等 resume | `nt_daemon`、routines/lease | **已覆盖 conservative candidate** | review exact |
| skills-lock.json/AGENTS.md | **Skill索引+版本锁** 跟进 agent skills | `nt_skill_installer` / `skill_registry.rs` | **已覆盖** |登记 |
| selfhost compose stack | local-first daemon+db+web | neobot 单二进制+SQLite=更简栈 | **已覆盖/不新建** |记录 |

## 3. 能直接进 neobot 的三件事（设计，未改码）
1. **Task activity timeline**：每一项 AgentTask 在 ledger 空缺时，每跳追加一行「系统事件流」（我们已有 steps/ledger，缺 source 字段） ⇒ 给 `steps` 加 `event_kind` 列可行；**不引入新表**。
2. **Pause/resolve/blocker 投影**：当 agent 抛 blocker 时把需要人接盘的 Decision 落 `ledger status='blocker'`（重用 ledger 格式，仍以 `status` 表达语义） ⇒ 可复用。
3. **System-of-record 的 Issue 来源**：`tasks.source_url`/`origin`コーディネータ,若没有先使用已有 messages.origin（见 P1-1b evidence: convo.origin 已有 chat/sidebar/plugin 投影）。

## 4. 不做项（避免造第二张词汇表）
- 不做 board app 在 neobot 内部；板是产品层，neobot crate 是 agent 内核。
- 不 fork Go server/TS前端。
- 不建平行 agent registry（`nt_capability_registry` 已是单一集合）。

## 5. 下一步（需要 owner 指点）
- 若要把「blocker 可观测」提上日程：在 ledger 的 `status` 枚举里补明 `blocker` 并加烟测行，属小改。
- 若要「task eval atlas」入口：可先用 `steps` 表扩展 `event_kind`（lite 迁移，幂等 ALTER）。
 - 若要「task activity board」入口：先评估是否增 `steps.event_kind`（lite 迁移，幂等 ALTER），否则 `task list` stdout 已足够。

*版本：2026-10-08 初稿。判定依据：README.md/VISION.md 原始文本 + `docs/*` headings；latest pushed_at, 未 clone。*
