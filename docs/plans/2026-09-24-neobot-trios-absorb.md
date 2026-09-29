# OpenMuse / openbot / cumora 三源吸收 → neobot 补完（2026-09-24）

> 范围：三库最新 main 全量精读（`neobot-absorb/` 9-24 快照），代码改动已落地
> `crates/neotrix-neobot` + `apps/neobot-desktop` + CLI。R-P79：本报告即接线索引。
> MIT 许可三库；吸收思想与机制，不搬代码；文件头均注来源。

## 1. 吸收映射表（来源文件 → neobot 落点）

| # | 来源 | 落点 | 内容 |
|---|---|---|---|
| 1 | OM `apps/server/src/engine/worker.ts` | `nt_store` tasks 列 + `recover_stale_running`/`cancel_task`/`retry_task` | lease 认领 + 失租回收（单机版：起点扫一次）+ attempts + error 列 |
| 2 | OM `packages/domain/src/agent.ts` | `nt_types::TaskStatus::Cancelled` | cancelled 终态；其余（Goal/Monitor/Idea）远期 |
| 3 | openbot `computer/gateway.ts` | `nt_agent` 审计先行 | 先写审计行（无论放行与否）再执行，崩溃不丢记录 |
| 4 | openbot `computer/policy.ts` | `nt_policy::evaluate_extra_deny` + `extra_deny` 配置 | deny 优先/缺省拒绝已有；补 operator deny-list + **坏规则照拒**（rule=`broken-rule`） |
| 5 | openbot `shared/routine-firing.ts` | `nt_routine::frame_firing/read_firing` | firing 帧单声明双读；整帧前缀匹配，首句引用不算 |
| 6 | openbot `routines/{schedule,runner,store}.ts` | `nt_store` routines 表 + `nt_routine::sweep/fire` | 15min 地板（`MIN_INTERVAL_SECS`）+ 10 连败自停（`FATIGUE_LIMIT`）+ 20 上限 + 2000 字 |
| 7 | openbot `server/src/audit.ts` | `nt_audit` 脱敏 key 扩展余量 + `prune_audit` | 留存清扫；key 表已对齐子集 |
| 8 | openbot take-the-wheel | `nt_store` control 表 + take/release/status + 交接审计 | 具名拒绝（报 holder）；CLI/桌面双入口 |
| 9 | cumora `agents/computer/engine.ts` | `nt_error::classify_engine_failure` + `CliEngine` 超时/上限 | 失败六分类 + kind 决策重试；CLI 300s 可调超时杀 + 64KiB 输出上限 |
| 10 | cumora `agents/tools-shared.ts` | `nt_http_engine` SYSTEM_PROMPT + bash/set_turn_status 描述 | CLI 即协议写进模型世界观；status 必填 + reason/next_step 可选 |
| 11 | cumora `agents/llm-ledger.ts` | ledger 列 + `nt_cost` | purpose + status + latency_ms + measured（**永不猜价**：未配置一律 0）；`NEOBOT_PRICE_*_PER_M` 显式计价 |
| 12 | cumora `realtime-outbox.ts` | outbox 列 + `drain/fail/prune` | attempts + available_at（认领时刻）；纯时间留存 |
| 13 | cumora `agents/cli-result.ts` | `nt_cli` 批量协议 + `NEOBOT_RESULT_PATH` | 坏行计数不炸；裸对象/数组/包络三形状；写失败 stderr 标记抢救 |
| 14 | cumora `agents/routing-claims.ts` | `claimed_at` + `sweep_stale_claims`（默认 300s） | 认领租约思想本地版（选举/sweep 多副本语义不取） |
| 15 | cumora personas 反独白/反重复 | 不取（多 agent 队聊语义，单机无载体） | 记录在案，远期多租户再议 |
| 16 | openbot CEL / MCP / SAML / Postgres / Redis | 不取（外部依赖/企业面，违本地独立保证） | 分歧见 §3 |

## 2. 本次修的旧 bug（实测抓获）

1. `ledger_sums` 把 `out_tokens` 读成了 `in_tokens` 列（索引 2 写两遍）——已修正。
2. `CliEngine` 无超时、无输出上限、stderr 丢失——重写为并发读 + 超时杀。
3. 审计后写（先执行后记录）——改为先写后执。
4. `neobot` CLI 缺 task 认领/取消/重跑、audit 留存、账本、例行、技能、成员、接管、策略演练——全补齐。

## 3. 单机分歧（相对三源，刻意）

- 无 Redis：seen/presence 纯内存；SKIP LOCKED 用 SQLite 原子 UPDATE 代。
- 无 cron：routines 用 interval_secs；无 CEL：extra_deny 小 matcher。
- 无多租户：members 仅首成员 owner；channel/DM/ visibility 私队位保留字段。
- 账本永不猜价；审计明细永不出前端（DTO 脱敏）。

## 4. 验证（本会话实测）

- `cargo check -p neotrix-neobot --all-targets`：0 error / 0 warning
- `cargo test -p neotrix-neobot --lib`：46 passed
- `cargo check -p neobot-desktop -p neotrix-tauri`：无新增 error（旧警告不动）
- 前端 `tsc --noEmit` + `vite build`：零错
- CLI parity（隔离 `NEOBOT_DATA_DIR`）：init/doctor/policy drill/run/task{list,claim,release,cancel,retry}/audit{list,prune}/ledger{,by-actor}/routine{add,list,fire,sweep,remove}/skill{list,install,show}/member{add,list,remove}/control{take,status,release}/models 全过
- `neobot policy drill`：8 项 PASS 常驻回归门

## 5. 并窗注意（2026-09-24 实测）

- `Cargo.toml` workspace members 里**没有** `games/neotrix-guixu`：
  其目录在工作区被别窗删除（未提交），加回去会炸所有 cargo 命令
  （manifest 缺失）。恢复目录的窗请顺手把 member 加回来。
- `cargo check -p neotrix-tauri` 当前红在 `neotrix-core/.../nt_mind_skill_engine`
 （`ResidencyAuditRow`/`INNER_ONLY` 不存在）——别窗重构中，与 neobot 无关
  （neotrix-core 不依赖 neotrix-neobot）。`neobot-desktop` 单独 check 是绿的。
