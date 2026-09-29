# rish-app 吸收 → neobot 核心建议与待优化点（2026-09-25）

> 来源：https://github.com/ZSeven-W/rish-app（MIT，pocket agent：手机本地执行 +
> 模型自由接入 + 工具执行 + 审批）。精读其 Rust core
> （`modules/rish/core/crates/rish-agent-core`：agent_policy / tool_registry /
> round_journal / ledger_batch / ledger_ops / transcript_store / WAL）与移动端
> harness/registry + grants 机制。思想吸收，不搬代码。

## 1. 已落地（本轮）

| # | rish 侧 | neobot 落点 |
|---|---|---|
| 1 | 嵌套写预算（single < batch < attempt，非法配置直接拒） | `WriteBudget` 进 `NeobotConfig` + validate 嵌套校验 + `execute_write/edit` 单次与单轮双门（超限分别 Invalid/Denied `write-budget`） |
| 2 | 转录上限（2MiB / 1024 条 / 128 轮） | `enforce_transcript_budget`（256KiB / 200 条本地值）：超限从旧往新丢 Tool 行，用户原文与回复保留，全量仍在 steps 表 |

## 2. 核心建议（按优先级）

### P0 —— 审批队列（当前最大缺口）
- rish：tool call 走 approval token（host 生成 UUID，按序消费）+ idempotency key（SHA256）+ 会话冻结 grant（conversation/workspace/binding-revision 绑定）。
- openbot 侧已有 `waiting_approval` 态，neobot 两边都没接：deny 即终局。
- 建议设计（下轮可做）：`approvals` 表（token PK，task/tool/args 脱敏快照，status pending，expires_at）→ 网关对 `approval:manual` 工具返回 `Waiting` + 落队 → `neobot approve <token> --allow/deny` + IPC + 前端审批卡（沿用 hero 卡槽三选一互斥）→ 批准后凭 token 执行一次（防重放：执行即销 token）。
- 工作量：约 1 轮（store 1 表 + gate 分支 + CLI 2 命令 + IPC 3 + 前端 1 卡）。

### P0 —— 纯 reducer 纪律（rish round_journal 律）
- rish：reducer 不读时钟/文件/注册表（Env/View 进，Effect 出），可测性来自输入全显式。
- neobot 现状：`evaluate_policy`/`evaluate_extra_deny` 已是纯函数 ✓；`run_loop` 混 IO 与决策。
- 建议：保持现状纪律（新逻辑先写纯函数 + 单测），`truncate/enforce_transcript_budget` 已是样子。

### P1 —— 安全投影（agent_policy 律）
- rish：policy describe 输出显式枚举 key，防路径/句柄/参数泄漏到 UI。
- neobot 现状：audit detail 脱敏 + DTO 脱敏 ✓；但 `ToolCall.args` 原样进 history 给模型（必要），以及 audit detail 含 intent（OK）。
- 待优化：`neobot task show` 若暴露 steps 输出，需默认截断 + 二次脱敏（目前无该命令，记账）。

### P1 —— 金色测试（golden.rs 律）
- rish：fixture 回放锁行为（journal 输入 → Effect 断言）。
- 建议：给 `evaluate_policy` 规则矩阵 + `run_loop` 三段式（deny/allow/dry-run）各加一个 golden JSON（输入固定，断言审计行序列）。现有单测是点断言， golden 补面。

### P1 —— 账本批插（ledger_batch 律）
- rish：hop 批量 buffered insert。
- neobot 现状：逐 hop `record_ledger`（SQLite 本地，量小无痛）。
- 待优化：hop 超 20/轮时批量提交（事务包起来，一次 commit）。触发阈值到再说，目前记账。

### P2 —— 引擎注册表校验（harness registry 律）
- rish：manifest id 正则 + 去重 + 模型目录校验，坏 manifest 启动即炸。
- neobot 现状：providers 表无 id 格式校验（`a/b` 也能存），模型名无校验。
- 待优化：`upsert_provider` 加 name 正则（`^[a-z0-9][a-z0-9._-]{0,63}$`）+ base_url 归一化（去尾斜杠， cardinal 已做一半）。

### P2 —— 移动端形态
- rish 是 pocket agent（手机本地执行）；neobot 是桌面 + CLI。
- 不建议跟进（形态分岔），但其“诚实边界”文档律值得抄：README 已有“本地独立保证”，建议每次加外部依赖时同步更新该段（R-P79 的文档侧）。

## 3. 本轮验证

- `cargo test -p neotrix-neobot --lib`：59 passed（含预算嵌套拒绝/单轮累计拒绝/转录裁剪 3 新单测）
- 生产代码 clippy 零新增警告（仅测试 expect 旧惯例）
