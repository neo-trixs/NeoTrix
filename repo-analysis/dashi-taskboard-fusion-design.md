# dashi-taskboard 对标与融合设计 (Phase 2 — Kanban 正交化)

> 分析日期: 2026-08-07 | 域: NT-CORE (路由) + NT-ACT (看板) | 状态: 设计草案

## 1. 对标对象

**dashi-taskboard (Codex Taskboard)** — github.com/chuspeeism/dashi-taskboard (1.1k stars)
- 定位: local-first issue board, 浏览器运行, 可通过 CDP 注入嵌入 Codex
- 同一 HTTP API 驱动三端: React UI (`web/`) + `taskctl` CLI (`cli/`) + Codex Skill (`skills/manage-taskboard`)
- 存储: SQLite (`.data/taskboard.sqlite`); 云模式 Cloudflare D1 + R2
- 传输: SSE 实时广播变更, 重连客户端全量刷新

## 2. 能力矩阵 (dashi vs NeoTrix /board)

| 能力 | dashi-taskboard | NeoTrix `/board` (kanban_cmds.rs) | 差距 |
|------|----------------|-----------------------------------|------|
| 任务 CRUD | `taskctl issue create --status todo --priority high --labels` | `/board create <spec>` | ✅ 等价 |
| 状态机 | todo→in_progress→in_review→done (用户确认才 done) | pending→in_progress→blocked→done→cancelled→deferred | ✅ 等价 |
| 依赖/WIP 约束 | 无显式, 靠 skill 工作流 | dependency/block/wip limit | ✅ 更强 |
| issue↔git branch/worktree 绑定 | ✅ 每 issue 绑一个 branch 或 worktree, 从仓库扫描选项 | ❌ `WorkItem` 无 branch 字段 | **缺** |
| 会话关联 | ✅ `CODEX_THREAD_ID` 记录在 issue/comment mutation | ❌ 无 | **缺** |
| 实时同步 | ✅ SSE 广播 + 重连全量刷新 | ❌ 仅 JSON 文件手动 save/load | **缺** |
| CLI+UI 同 API | ✅ 同一 HTTP API | ✅ unified_cmd 桥已实现 | ✅ |
| agent 工作流 skill | ✅ Skill 教 agent: 检查→in_progress→乐观版本→验证→in_review | ⚠️ `/chain goal` 已做编排, 但无 board 状态机流转 | 部分 |
| 乐观版本/验证 | ✅ skill 显式要求 verify | ❌ 无 | 可选 |
| 多客户端/协作 | ✅ LAN 共享 | ❌ 单机 | 可选 |

## 3. 融合决策 (R-P42: 吸收强化现有节点, 禁止平行适配器)

**原则**: 不新建平行看板系统。`/board` 已 1644 行覆盖核心, 按 dashi 最佳实践**增强现有 WorkItem/命令**, 而非复制 dashi 架构。

### 3.1 增强 1: issue→branch 绑定 (高价值)
- `WorkItem` 增加字段: `git_branch: Option<String>` (serde default, 向后兼容旧 JSON)
- 新子命令: `/board branch <id> [branch-name]` — 无参时从当前仓库扫描 branch/worktree 选项 (dashi 同款)
- `/board create` 支持 `--branch <name>` 参数
- 验证: `/board list` 显示 branch 列

### 3.2 增强 2: 会话关联 (中价值)
- `WorkItem` 增加字段: `thread_id: Option<String>`
- `/board view <id>` 显示 thread_id (若有)
- `/board assign` 时自动记录当前 session thread (从 env 或 CLI 上下文)

### 3.3 增强 3: 状态机流转命令 (中价值)
- `/board move <id>` 已实现 phase 推进 (can_advance_to)
- 补充 dashi 的 **in_review 语义**: `/board move <id> --to in_review` 后不允许自动进 done, 需 `/board move <id> --to done --confirm` 显式确认
- 对齐 dashi 状态命名: `in_progress`/`in_review` 已在 phase_to_str 映射

### 3.4 增强 4: SSE 广播 (低价值, 架构改动大)
- 当前 board 是 CLI 内嵌 JSON 状态, 无服务端
- **决策**: 暂不做服务端广播。NoeCodex Tauri 已有 EventBus, 若未来需要多窗口同步可复用 EventBus 事件, 不引入 HTTP 服务
- 记录为 future work

## 4. 实现计划

| 步骤 | 改动 | 文件 | 验证 |
|------|------|------|------|
| 1 | WorkItem 加 git_branch + thread_id 字段 (serde default) | kanban_cmds.rs | cargo check + 旧 JSON load 兼容测试 |
| 2 | `/board branch <id> [name]` 子命令 | kanban_cmds.rs | `/board branch` 测试 |
| 3 | `/board create --branch <name>` | kanban_cmds.rs | 测试 |
| 4 | `/board move --to done --confirm` 确认语义 | kanban_cmds.rs | 测试 |
| 5 | `/board view` 显示 branch/thread | kanban_cmds.rs | 测试 |
| 6 | e2e: palette catalog 显示新子命令 | fixtures.ts | playwright |

## 5. 未决项 / 风险

- **branch 扫描**: 需要调用 git 命令, 依赖 cwd 是仓库 (与 `/chain review` 相同假设, 可复用)
- **thread_id 来源**: CLI 交互会话无 thread id, 需定义 fallback (空或 session hash)
- **旧 JSON 兼容**: 所有新字段必须 `#[serde(default)]`, 否则 `/board load` 旧文件反序列化失败
- **范围控制**: 本设计只做 board 域, 不触碰 workflow/routines 等平行系统 (Phase 2 后续章节)

## 6. 参考

- dashi-taskboard README: https://github.com/chuspeeism/dashi-taskboard
- 本地实现: `neotrix-core/src/cli/commands/kanban_cmds.rs` (1644 行)
- 链路命令: `neotrix-core/src/cli/commands/chain_cmds.rs` (`/chain status` 已汇总 board)
