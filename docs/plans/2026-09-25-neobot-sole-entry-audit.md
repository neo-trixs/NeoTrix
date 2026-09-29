# 审计：neobot 作为唯一入口调用 neotrix 完整能力，还差哪些（2026-09-25）

目标：neobot App = 与 neotrix 晶体核心对话的**唯一入口**；neotrix 完整能力经此调用；可迭代升级。

## 1. 已通（现状基线）

| 方向 | 内容 | 证据 |
|---|---|---|
| 对话 | chat/completions（JSON + SSE）| nt_crystal_serve handle_chat；实测往返 |
| 模型 | 单 `neotrix-crystal`（池 27 自主选，熔断降级）| /v1/models；/healthz pool/degraded |
| 工具 | tools 进 / tool_calls 出 / 历史回环 | serve 透传；neobot 网关执行；实测写文件 |
| 执行 | neobot 侧 bash/读写/终态 + 审计先行 | audit 有 write_file allow 行 |
| 配对 | pair/unpair/status， desktop 状态行 | core_pair 表；neobot_core_status |
| 记忆 | neobot 侧 MEMORY.md 注入 | http_engine memory_context |

## 2. 未通：neotrix 能力缺口（按优先级）

**P0 — 服务端 agent 循环（真正的"完整能力"）**
- 现状：工具在 neobot 本地执行；neotrix 服务端的原生手（computer 操控、browse、它自己的 skills/memory）够不着。
- 抓手都现成：`crates/neotrix-gateway/skill_registry.rs`（scan/search/activate）、`hybrid_search.rs`（index/search/pack_context）、`neotrix-multi-agent/coordinator.rs`（create_plan/execute_plan）、`entry/browse.rs`（run_browse/run_search）。
- 建议：晶体加 `POST /v1/agents/run`（goal 进，多步执行，要么服务端直调 gateway 能力后回终态，要么把 neotrix 工具以 function schema 回给 neobot 执行——后者复用现有网关，更快）。

**P0 — 流式工具调用**
- 现状：SSE 是缓冲后一次性 flush；工具调用在流里不可见，neobot 要等整轮。
- 建议：SSE 按 delta 逐块 flush + `tool_calls` 以 `finish_reason=tool_calls` 独立事件发出。

**P1 — 能力广播（/capabilities）**
- 现状：neobot 靠写死的工具表和单模型假设；neotrix 上新能力前端无从得知。
- 建议：`GET /v1/capabilities`（模型 + 工具表 + 特性开关 + server 版本），neobot 启动拉一次，动态渲染模型面板/工具门。

**P1 — 记忆贯通**
- 现状：记忆是 neobot 单向注入；neotrix 侧记忆体（graph/spatial/meta）不连通，两边各记各的。
- ✅ 已落地（2026-09-25 夜）：晶体 `agents/run` 接经验 recall——`~/.neotrix` 审计 jsonl（491 个）+ KB 全文检索（ASCII 词 + CJK bigram 分词），命中拼入 notes 进上游 prompt，无命中回落客户端 memory；`features.memory_read=true` 实播。写回仍未做（下轮）。
- ✅ 写回已落地（同日深夜）：终态摘要追加 `audit_neobot_crystal.jsonl`（goal/output/model/steps/ts），recall glob 覆盖 → 下轮可读回。实测"抓取example页面标题"写回后被下轮 recall 命中。读贯通闭环完成。
- ✅ neotrix 原生手已落地（同日深夜）：`web_search`（UnifiedSearch，spawn_blocking）+ `web_fetch`（BrowserEngine Http 后端 Navigate+GetContent，4000 字截断，http/https fail-closed）进 agents/run ops 与工具表。实测 DDG 搜 Rust 版本、example.com 抓取全通。注意坑：BrowserEngine 含 Cell → 非 Sync，worker 内不可直接 .await，已用 blocking+独立 runtime 包住（仿 run_browse 模式）。
- 剩余原生手：computer 点击/输入（需 Chrome/CDP + 显示器，headless 不可用）、login（需人工）、graph/spatial 记忆体直连（当前经验 recall 已够用，列远期）。
- ✅ computer 真机操控已落地（同日深夜）：`web_act` op（navigate/click/type/text/shot，单会话单次语义，http/https fail-closed）经 headless Chrome CDP 直调；实测 example.com 点击跳转 iana.org 并读回。途中三坑：①BrowserEngine 含 Cell 非 Sync（blocking+独立 runtime 已解）；②僵尸 Chrome 占 profile 锁（失败重试前 pkill 专属 profile）；③neotrix Click 内置 post-click GetContent 与跳转竞态（转"已点、重读"部分成功）。login（需人工交互）与记忆体直连仍列远期，前者不可为、后者经验 recall 已够用。

**P1 — 池热刷新**
- 现状：加端点/换 key 必须重启服务（池启动时组装）。
- 建议：`POST /v1/admin/reload`（读配置重建池；本地回环 + 可选 token）。

**P2 — 入口唯一性 enforcement**
- 现状唯一性靠"自觉"：晶体只绑 127.0.0.1，无任何鉴权；neotrix 还有 TUI（entry/interactive）、headless、desktop、proxy-daemon 等直接入口；opencode CLI 直调 zen 完全绕过。
- 建议：晶体加本机 token（`CRYSTAL_TOKEN`，neobot 配对时写入，请求头校验）；opencode CLI 直调视为"维修通道"书面保留，不禁但审计声明。其余入口不动（那是 neotrix 自己的事，neobot 只保证"对话走晶体"）。

**P2 — 升级通道**
- 现状：server 无版本自述；neobot 不知道晶体新旧。
- 建议：`/v1/models` 回 `owned_by` 外加 `crystal_version`；neobot 设置页显示"晶体版本 x.y.z（池 N）"，版本落后 toast 提示重启升级。

**P3 — 意识/多智能体直调**
- `coordinator.create_plan/execute_plan`、`consciousness`、`experience_tree` 都是可调函数，但语义重（规划、长程状态），先经 P0 的 agent 端点统一收口，不单独开洞。

## 3. 迭代升级顺序（建议）

1. `/v1/agents/run`（neotrix 工具回给 neobot 执行版，2-3 天量，复用网关）
2. SSE 真流式 + tool_calls 事件（半天）
3. `/v1/capabilities` + `/v1/admin/reload` + 版本自述（半天）
4. 本机 token（1 小时）
5. 记忆读贯通（1 天）
6. 服务端直调 neotrix 原生手（按需，computer/browse 先行）

做完 1-4， "唯一入口 + 完整调用 + 可升级" 的架子就成立了。

## 4. 执行契约 v1（并行施工唯一依据，各方照此对接）

- 认证：`crystal.toml` 顶层 `token = "..."`（空 =  legacy 开放 + 启动 warn）；
  置了就强制校验所有 `/v1/*` + `/admin/*`（`Authorization: Bearer`），`/healthz` 开放。
  neobot 侧永不存 token 值：`core pair --token-env CRYSTAL_TOKEN`（缺省名），
  存名 + 每次请求现读环境（沿用 key_env 机制）。
- 版本：server 内 `CRYSTAL_VERSION = "0.2.0"`；`/healthz` + `/capabilities` 带 `crystal_version`。
- `GET /v1/capabilities` → `{model, crystal_version,
  tools: [{name, description}], features: {agent_run, memory_read,
  hot_reload, streaming_tool_calls}, pool: {count, degraded: []}}`。
- `POST /v1/admin/reload`（需认证）→ 重读配置重建池 → `{ok, pool, degraded}`。
- `POST /v1/agents/run`（需认证）
  `{goal, context?, memory?（客户端记忆文本）, max_steps?（缺省 8，上限 16）}`
  → `{status: done|error, output, trace: [{kind, detail}], model_used}`；
  服务端循环用 neotrix 原生手（skills/memory/coordinator 只读类 + workspace 内
  bash/读写，workspace 由请求 `workspace` 参数定、越狱拒绝uls；
  能力不齐就 `features.memory_read=false` 如实广播，不硬演）。
- SSE 真流式：`stream:true` 按块 flush；工具调用以独立事件
  `data: {"choices": [{"delta": {"tool_calls": [...]}, "index": 0}],
  "object": "chat.completion.chunk"}` 发出，最后 `data: [DONE]`。
- neobot 侧：`core_pair.token_env` 列；`core status` 显示版本/工具数；
  `core reload` CLI+IPC；`agent run` CLI（`neobot agent run "goal"`）+ IPC
  `neobot_agent_run`；SSE 解析补 tool_calls delta；frontend 设置页显示版本摘要。

## 5. 维修扫描记录（同日深夜，并行四线）

- 单测 76 全绿（晶体新增 4 个：snip/分词/ops 解析/workspace 越狱）。
- 修旧 flake（`tool_loop_runs_bash_then_done` 并行抖动）：根因是 fake server
  队列打空后吐 `{}`，重试烧队列永失败；改为打空循环重放。连续 3 轮全绿。
- clippy 双仓零新增 warn；晶体无 unwrap/expect/panic/todo。
- tools=9 在线；服务双活（crystal + desktop）。
- 外部死端点 9 家属上游 key/额度问题，非代码可修，配置跳过并日志留痕。
