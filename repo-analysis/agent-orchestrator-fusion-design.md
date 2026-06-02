# Agent Orchestrator (ComposioHQ) → NeoTrix 融合设计文档

> **目标**: 将 Agent Orchestrator 的并行 agent 编排 + git worktree 隔离模式融合到 NeoTrix background_loop
> **源项目**: https://github.com/ComposioHQ/agent-orchestrator — 7.3K⭐, MIT license, 3,288 tests

---

## 1. Project Overview

**Agent Orchestrator** 是一个 TypeScript monorepo（21 plugin packages），核心模式：

```
ao start → Dashboard(:3000) → Orchestrator Agent spawns Worker Agents
                                         ↓
                              Each in isolated git worktree
                                         ↓
                              Auto-handle CI/review feedback
                                         ↓
                              Auto-create PRs → merge queue
```

**8 Plugin Slots**: Runtime (tmux/process/docker) · Agent (claude-code/codex/aider) · Workspace (worktree/clone) · Tracker (github/linear) · SCM · Notifier (desktop/slack/discord) · Terminal (iterm2/web) · Lifecycle (non-pluggable)

**Session Lifecycle**: `spawning → working → pr_open → ci_failed → review_pending → changes_requested → approved → mergeable → merged → cleanup → done`

**Key Innovation**: Hash-based runtime data namespace (`sha256(configDir).slice(0,12)`) — multiple orchestrator checkouts never collide.

**NeoTrix BackgroundLoop** (Rust/tokio): Single-process ticker-based loop with 14 async tickers (save/consolidate/evolve/cleanup/mine/goal/metacog/thinking/crawler/awakening/exploration/telemetry/proxy_retry/agent_discovery). No parallel agent spawning, no worktree isolation, no PR lifecycle.

---

## 2. Core Abstraction Comparison

| 维度 | Agent Orchestrator | NeoTrix BackgroundLoop |
|------|-------------------|------------------------|
| **语言/运行时** | TypeScript/Node, pnpm monorepo | Rust/tokio async, cargo workspace |
| **核心抽象** | Plugin slots + Session lifecycle state machine | Async ticker loop + tokio::select! |
| **隔离机制** | `git worktree` per agent, hash-namespaced dirs | 无 (all in-process) |
| **Agent 类型** | Orchestrator Agent (coordinator) + Worker Agents (executors) | 无 (single SelfIteratingBrain) |
| **任务来源** | GitHub Issues / Linear tickets | GoalLoop auto-goal generation |
| **反馈回路** | CI failure → auto-fix, Review comment → auto-address | 内部 RL 奖励 + stagnation detection |
| **持久化** | Flat metadata files in `~/.agent-orchestrator/{hash}/` | `~/.neotrix/brain.json` + goals.json |
| **UI** | Next.js kanban dashboard (:3000) | None (headless REPL only) |
| **配置** | YAML (`agent-orchestrator.yaml`) | BackgroundConfig struct |
| **测试** | 3,288 tests (vitest) | ~50 tests (cargo test) |
| **扩展方式** | Plugin interface (TypeScript types) | Feature gates + hardcoded modules |
| **生命周期管理** | Formal state machine (11 states) | Tick interval + stagnation gates |
| **Prompt 组装** | 3-layer: base + config + project rules | ReasoningEngine (LLM provider) |
| **多项目** | Multi-repo from single config | Single project |
| **Dashboard** | Real-time kanban, agent status, CI status | Console logging only |
| **安全/访问控制** | Caffeinate sleep prevention, workspace sandboxing | Stealth-net proxy/Tor mode |
| **Runtime 选择** | tmux / ConPTY / Docker / k8s / SSH / e2b | None (in-process tokio tasks) |

---

## 3. Gap Matrix

### 3.1 What Agent Orchestrator has — NeoTrix doesn't (P1 candidates)

| # | GAP | 描述 | 等级 | Impact |
|---|-----|------|------|--------|
| G1 | **Git worktree isolation for parallel agents** | Each agent gets `git worktree add` — own branch, own directory, zero conflicts | **P0** | 核心编排能力 |
| G2 | **Session lifecycle state machine** | 11-state formal lifecycle: spawning→working→pr_open→ci_failed→review→changes→approved→merged→cleanup→done | **P0** | Agent 生命周期管理 |
| G3 | **Plugin architecture (8 slots)** | Runtime/Agent/Workspace/Tracker/SCM/Notifier/Terminal — 每个都是可插拔 TypeScript 接口 | **P1** | 架构可扩展性 |
| G4 | **Web dashboard (Next.js kanban)** | Real-time 6-column kanban with status dots, CI badges, PR tracking | **P1** | 可观测性 |
| G5 | **Feedback reaction system** | CI failure → agent auto-fix, reviewer comment → agent auto-address, `escalateAfter: 30m` | **P1** | 闭环自动化 |
| G6 | **Hash-based runtime isolation** | `sha256(configDir).slice(0,12)` 防碰撞命名空间 + `~/.agent-orchestrator/{hash}/` 目录结构 | **P2** | 多实例运行 |
| G7 | **Agent-agnostic runtime** | 支持 claude-code, codex, aider, cursor, opencode, kimicode 作为 worker | **P1** | 外部工具集成 |
| G8 | **Formal spawn flow** | Validate → reserve session → create workspace → build prompt → launch agent → persist metadata | **P0** | 可靠 agent 创建 |
| G9 | **Multi-project / multi-repo support** | 一个 config 文件管理多个 git repo | **P1** | 跨项目编排 |
| G10 | **Session metadata persistence** | Flat key=value metadata files per session (branch, issue, status, PR URL, worktree path) | **P2** | 可调试性 |
| G11 | **Reaction config schema** | YAML reactions: ci-failed, changes-requested, approved-and-green with auto/retries/escalateAfter | **P1** | 工作流自定义 |
| G12 | **Orchestrator agent pattern** | 一个 orchestrator agent 做协调 + 多个 worker agent 做执行 — 职责分离 | **P0** | 编排架构 |
| G13 | **CI merge queue** | 自动 rebase 后 merge，冲突处理 | **P2** | CI/CD 集成 |
| G14 | **Spawn flow 的 fail-fast** | Spawn 前先 validate issue，不分配资源 | **P1** | 资源保护 |

### 3.2 What NeoTrix has — Agent Orchestrator doesn't

| # | Feature | 描述 | 价值 |
|---|---------|------|------|
| N1 | **ReasoningBrain / CapabilityVector** | 22-dim + extension 自进化能力向量 | 核心 AI 能力 |
| N2 | **GoalLoop with Motivation** | 自动目标队列，动机驱动重平衡 | 自主目标生成 |
| N3 | **SiliconSelf / ThinkingBridge** | 10-domain attention, context window, self-repair | 认知自模型 |
| N4 | **MetaCognition cycle** | 弱项扫描 + 关键度检测 + 健康分 | 元认知 |
| N5 | **KnowledgeChain + UnifiedCrawler** | 自主知识挖掘 + web 爬取 | 持续学习 |
| N6 | **ExplorationPipeline (SEAL-Explore)** | 领域探索管道 | 未知领域探索 |
| N7 | **KnowledgePopulator / KnowledgeBridge** | 预注入知识源到 brain + ReasoningBank | 冷启动知识 |
| N8 | **B-Brain unified monitor** | 生物脑风格健康监控 + RED ALERT | 系统健康 |
| N9 | **Agent Protocol (UDP discovery)** | P2P agent 发现 :42069 | 多实例协调 |
| N10 | **Stealth-net** | proxy/Tor/geo-routing/中国绕过 | 网络隔离 |
| N11 | **Persona system** | AgentPersona with experience multiplier + bias | 角色个性化 |
| N12 | **Hypergraph memory** | 1000-capacity hypergraph in ReasoningBank | 记忆关联 |
| N13 | **Stagnation detection** | 每个子系统独立停滞检测 + 自动终止 | 系统鲁棒性 |
| N14 | **TelemetryCollector** | AtomicU64 counters + snapshot | 轻量遥测 |

### 3.3 Shared / Similar concepts

| Concept | Agent Orchestrator | NeoTrix |
|---------|-------------------|---------|
| Session persistence | Flat metadata files | goals.json + brain.json |
| Configuration | YAML file | BackgroundConfig struct |
| Lifecycle monitoring | Dashboard + polling | TelemetryCollector + println |
| Error handling | Retry with escalation | Stagnation counters + termination |
| Multi-instance | Hash-based namespace | Agent Protocol UDP discovery |
| Telemetry | Observability module | TelemetryCollector |

---

## 4. Priority Classification

### P0 — Immediate (核心编排能力缺失)

| Gap | 依赖 | 实现成本 | 融合策略 |
|-----|------|---------|----------|
| **G1: Git worktree isolation** | git CLI in path | 3-4 day | 新增 `worktree_manager.rs` 模块，包装 `git worktree add/remove/prune` |
| **G2: Session lifecycle state machine** | None | 2-3 day | 新增 `agent_session.rs`，枚举 11 状态 + 驱动函数 |
| **G8: Formal spawn flow** | G1, G2 | 2 day | 新增 `session_manager.rs`，validate→reserve→workspace→prompt→launch→persist |
| **G12: Orchestrator + Worker pattern** | G1, G2, G8 | 3 day | background_loop 新增 orchestrator_ticker + worker_spawner |

### P1 — High (编排功能完整)

| Gap | 依赖 | 实现成本 | 融合策略 |
|-----|------|---------|----------|
| **G3: Plugin architecture** | None (architectural) | 5-7 day | 定义 Rust trait 系统: `RuntimeProvider`, `AgentProvider`, `WorkspaceProvider` |
| **G4: Web dashboard** | None (frontend) | 5-7 day | 集成现有 mcp_tools.rs rmcp 服务 + 新增 HTTP 状态端点 |
| **G5: Feedback reaction** | G2, G8 | 2 day | 新增 `reaction_engine.rs` + YAML-style 反应配置 |
| **G7: Multi-agent runtime** | G12, G3 | 3 day | AgentProvider trait → 支持 subl/codex cli spawn |
| **G9: Multi-project** | G1 | 3 day | BackgroundConfig 扩展为项目列表 |
| **G11: Reaction config schema** | G5 | 1 day | 配置反序列化 + BackgroundConfig 扩展 |

### P2 — Nice to have

| Gap | 依赖 | 实现成本 | 融合策略 |
|-----|------|---------|----------|
| **G6: Hash-based isolation** | G1 | 1 day | `sha2::Sha256` 生成 runtime data 路径 |
| **G10: Session metadata** | G2 | 1 day | TOML/JSON flat 文件每 session |
| **G13: CI merge queue** | G2, G5 | 3 day | Git 操作包装器 + merge 策略 |
| **G14: Fail-fast spawn** | G8 | 0.5 day | validate_issue() 前置 |

---

## 5. Integration Points

### 5.1 新增文件

```
neotrix-core/src/neotrix/
├── agent_orchestrator/
│   ├── mod.rs                          # 模块导出
│   ├── session_lifecycle.rs            # G2: 11-state state machine
│   ├── session_manager.rs              # G8: Spawn flow
│   ├── worktree_manager.rs             # G1: Git worktree CRUD
│   ├── spawn_flow.rs                   # G8: validate → reserve → workspace → prompt → launch
│   ├── reaction_engine.rs              # G5: CI/review feedback → agent routing
│   ├── plugin_traits.rs                # G3: Runtime/Agent/Workspace/Tracker traits
│   ├── runtime_providers.rs            # G7: tmux/process/docker adapters
│   ├── agent_providers.rs              # G7: opencode/codex adapters
│   ├── orchestrator_agent.rs           # G12: coordination agent logic
│   ├── worker_agent.rs                 # G12: execution agent wrapper
│   ├── metadata_store.rs               # G10: flat key=value session persistence
│   └── config.rs                       # G11: YAML config schema + deserialization
```

### 5.2 现有文件修改

| 现有文件 | 修改点 | 关联 Gap |
|----------|--------|----------|
| `background_loop.rs` / `background_loop_impl/start.rs` | 新增 `orchestrator_ticker` (并行 agent 调度循环), 集成 `OrchestratorAgent` | G1, G2, G12 |
| `background_loop_impl/types.rs` | `BackgroundConfig` 扩展: `multi_project`, `agent_runtimes`, `reactions`, `max_parallel_agents` | G9, G11 |
| `reasoning_brain/goal_loop.rs` | GoalLoop 任务产出可路由到 Orchestrator 而非仅内部 pursue | G12 |
| `agent/team.rs` | AgentTeam 可调用 Agent Orchestrator 的 session lifecycle | G2 |
| `mcp_tools.rs` | 新增 `list_sessions`, `spawn_agent`, `attach_session` MCP 工具 | G4 |
| `entry/mod.rs` | 启动时可选起 Orchestrator dashboard HTTP 端点 | G4 |
| `agent_protocol/capabilities.rs` | CapabilityRouter 扩展为 support orchestrator worker route | G12 |

### 5.3 BackgroundLoop Ticker 集成

新增 **2 个 ticker** 到 BackgroundLoop:

```rust
// background_loop_impl/start.rs
let mut orchestrator_ticker = interval(Duration::from_secs(self.config.orchestrator_interval_secs));
// vs. Agent Orchestrator's polling loop approach

tokio::select! {
    // ... existing tickers ...
    _ = orchestrator_ticker.tick() => {
        // 1. Check GoalLoop for new tasks
        // 2. Decompose task into subtasks (LLM-based)
        // 3. Create worktrees for each subtask (worktree_manager)
        // 4. Spawn worker agents in each worktree
        // 5. Track session lifecycle
        // 6. Poll for completion/CI status
    }
}
```

### 5.4 Session Lifecycle State Machine

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Spawning,          // Creating worktree, setting up agent
    Working,           // Agent is actively coding
    PrOpen,            // PR created, waiting CI
    CiFailed,          // CI failed → auto-fix loop
    ReviewPending,     // Waiting human review
    ChangesRequested,  // Reviewer asked changes → auto-address
    Approved,          // PR approved, CI green
    Mergeable,         // Ready to merge
    Merged,            // Successfully merged
    Cleanup,           // Removing worktree
    Done,              // Terminal state
    Killed,            // Manual abort
    Failed,            // Unrecoverable error
}
```

### 5.5 Plugin Trait Design (Rust)

```rust
/// Runtime provider — manages agent execution sessions
#[async_trait]
pub trait RuntimeProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn create(&self, session: &SessionConfig) -> Result<RuntimeSession>;
    async fn destroy(&self, session_id: &str) -> Result<()>;
    async fn send(&self, session_id: &str, input: &str) -> Result<()>;
    async fn is_running(&self, session_id: &str) -> Result<bool>;
}

/// Agent provider — wraps CLI tool invocation
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_launch_command(&self, worktree_path: &Path, config: &AgentConfig) -> Vec<String>;
}

/// Workspace provider — git worktree or clone
#[async_trait]
pub trait WorkspaceProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn create(&self, branch: &str, base: &str) -> Result<Workspace>;
    async fn destroy(&self, path: &Path) -> Result<()>;
}
```

### 5.6 Git Worktree Manager

```rust
pub struct WorktreeManager {
    pub base_dir: PathBuf,      // e.g. ~/.neotrix/worktrees/
    pub git_dir: PathBuf,       // main repo's .git
}

impl WorktreeManager {
    pub fn create(&self, branch: &str) -> Result<Worktree> {
        // git branch <branch> origin/main
        // git worktree add ../neotrix-wt-{branch} {branch}
        // return Worktree { path, branch, created_at }
    }

    pub fn remove(&self, worktree: &Worktree) -> Result<()> {
        // git worktree remove {path}
        // git branch -D {branch}
    }

    pub fn list_active(&self) -> Result<Vec<Worktree>> {
        // git worktree list
    }

    pub fn prune_orphaned(&self) -> Result<u32> {
        // git worktree prune
    }
}
```

### 5.7 Spawn Flow 整合

整合到 GoalLoop 现有的 pursue 管道：

```
GoalLoop.dequeue_next()
    → 检测到外部任务类型 (agent_orchestrator_task)
    → 调用 OrchestratorAgent.decompose(task)
        → LLM split into independent subtasks
    → 每个子任务：
        1. SessionManager.validate(subtask)      # fail-fast
        2. WorktreeManager.create(branch)         # git worktree 隔离
        3. AgentProvider.build_cmd(worktree)      # opencode/claude-code 命令
        4. RuntimeProvider.create(session)        # tmux/process 会话
        5. MetadataStore.write(session_id, ...)   # 持久化
    → WorkerAgent.monitor(sessions)              # 跟踪生命周期
    → ReactionEngine.poll(sessions)              # CI/Review 反馈
```

---

## 6. KnowledgeSource Registration Scheme

### 6.1 新 KnowledgeSource 枚举变体

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnowledgeSource {
    // ... existing variants ...
    AgentOrchestrator,     // 并行编排 + worktree 隔离知识
    GitWorktree            // git worktree 操作模式
}
```

### 6.2 CapabilityVector 映射

| KnowledgeSource | 维度 | 注入值 | 说明 |
|----------------|------|--------|------|
| `AgentOrchestrator` | `parallelization` | 0.85 | 并行 agent 编排能力 |
| `AgentOrchestrator` | `orchestration` | 0.90 | 工作流编排能力 |
| `AgentOrchestrator` | `isolation` | 0.88 | 工作区隔离意识 |
| `AgentOrchestrator` | `lifecycle` | 0.82 | session 生命周期管理 |
| `GitWorktree` | `git_operations` | 0.95 | 高级 git 操作能力 |
| `GitWorktree` | `branch_management` | 0.90 | 分支管理 |

### 6.3 Seed Knowledge 注入

ReasoningBank 预注入 5 条种子记忆：

1. **Agent 编排模式**: "Orchestrator Agent 将大任务分解为独立子任务，每个子任务分配到隔离 worktree"
2. **Session 生命周期**: "session lifecycle = spawning → working → pr_open → ci_failed → review → changes → approved → mergeable → merged → cleanup → done"
3. **Worktree 隔离**: "每个 agent 有独立 git worktree，永不冲突；关键是 `--no-optional-locks` 和交错延迟"
4. **反馈反应**: "CI failure 自动路由→agent 修复；review comment 自动路由→agent 回应；escalateAfter 超时→人工通知"
5. **Plugin 架构**: "8 slot 插件系统: Runtime/Agent/Workspace/Tracker/SCM/Notifier/Terminal/Lifecycle，每个实现 TypeScript 接口"

---

## 7. Test Strategy

### 7.1 Unit Tests (新增 ~40 tests)

| 模块 | 测试内容 | 数量 | 策略 |
|------|---------|------|------|
| `session_lifecycle.rs` | 11-state 转换矩阵，非法转移 reject | 8 | 单元 (无外部依赖) |
| `session_manager.rs` | validate→spawn→abort，mock workspace | 6 | mock git/agent |
| `worktree_manager.rs` | create/remove/list/prune，冲突处理 | 6 | CI 环境 git 操作 |
| `reaction_engine.rs` | 3 种反馈的自动处理逻辑 | 5 | 单元 (mock agent) |
| `plugin_traits.rs` | 各 trait 的 mock 实现 + 组合测试 | 6 | 纯单元 |
| `spawn_flow.rs` | fail-fast 路径，完整 spawn 路径 | 4 | 集成 (mock) |
| `config.rs` | YAML 反序列化，非法配置 reject | 5 | 文件解析 |

### 7.2 Integration Tests (新增 ~8 tests)

| 测试 | 描述 | 验证 |
|------|------|------|
| `test_worktree_create_and_remove` | 创建/移除实际 git worktree | `git worktree list` |
| `test_spawn_full_flow` | 完整 spawn 流程 (mock agent) | session metadata 存在 |
| `test_parallel_agents` | 2 个 agent 在独立 worktree 同时工作 | 无冲突 |
| `test_lifecycle_full_cycle` | spawning→...→done 完整流转 | 状态序列正确 |
| `test_reaction_ci_failed_auto_fix` | CI 失败 → agent 自动修复 | 修复后状态 |
| `test_multi_project_config` | 2 个项目从同一 config | 隔离目录 |
| `test_concurrent_worktree_creation` | 5 个 worktree 同时创建 | SIGBUS 不触发 |
| `test_orchestrator_decompose_and_dispatch` | Orchestrator agent 分解 + 分发 | 正确任务分配 |

### 7.3 测试门控

```
cargo test -p neotrix --lib agent_orchestrator  # 全部通过
cargo test -p neotrix --lib worktree             # git 操作测试（CI 环境）
cargo check --lib                                # 0 new errors
cargo check --features full --lib                # 全 feature 编译
```

---

## 8. Implementation Roadmap

```
Phase 1 (P0 — 核心编排, ~7 d)
├── Day 1-2:  worktree_manager.rs + 基础 git worktree 操作
├── Day 3-4:  session_lifecycle.rs + 11-state state machine
├── Day 5-6:  session_manager.rs + spawn flow
└── Day 7:    orchestrator_agent.rs + background_loop ticker 集成

Phase 2 (P1 — 编排完整, ~7 d)
├── Day 1-2:  plugin_traits.rs + runtime_providers.rs (tmux/process)
├── Day 3-4:  agent_providers.rs (opencode/codex adapter)
├── Day 5:    reaction_engine.rs + config YAML schema
├── Day 6:    metadata_store.rs + session 持久化
└── Day 7:    Web dashboard (mcp_tools.rs HTTP 端点)

Phase 3 (P2 — 完善, ~3 d)
├── Day 1:    hash-based isolation + multi-project config
├── Day 2:    CI merge queue + conflict resolution
└── Day 3:    fail-fast spawn + test suite
```

---

## 9. 关键设计决策

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| Worktree 目录 | `../neotrix-wt-{branch}` vs `.neotrix/worktrees/{branch}` | `.neotrix/worktrees/{branch}` | NeoTrix 已有 `~/.neotrix/` 约定 |
| Session ID | `{project}-{num}` vs hash-prefixed | `{hash}-{project}-{num}` (全局) + `{project}-{num}` (用户可见) | 兼容多实例 + 易读 |
| Runtime 默认 | tmux vs process (ConPTY) | process (tokio::process::Command) | NeoTrix 是 Rust 项目，无需 tmux |
| Agent 默认 | opencode vs claude-code | opencode | NeoTrix 已在用 opencode CLI |
| Config 格式 | YAML vs TOML vs JSON | TOML | NeoTrix 已有 TOML 偏好 (Cargo.toml) |
| Plugin 加载 | 静态 trait vs 动态 dynlib | 静态 trait + feature gate | Rust 生态惯例，编译期安全 |
| Dashboard | Next.js 单独仓库 vs Rust HTTP 端点 | Rust axum + 现有 mcp_tools.rs | 减少语言跨栈，复用 tokio |

---

## 10. 风险 & 缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| macOS `SIGBUS` on concurrent worktree | 🔴 High | `--no-optional-locks` + 100ms 交错延迟 |
| Worktree 进程成为 zombie | 🟡 Medium | `BackgroundLoop.handles` + watchdog timeout |
| Git LFS 不兼容 worktree | 🟡 Medium | 检测 + 回退到 `clone` workspace mode |
| 多个 opencode 实例竞态 | 🟡 Medium | Agent Protocol 已有 UDP 发现，协同端口分配 |
| 文件系统 inotify 冲突 | 🟢 Low | 每个 worktree 独立，不走 shared watcher |
| LLM 分解任务不准确 | 🟡 Medium | 用户确认步骤 (同 Agent Orchestrator 的 plan approval) |

---

*生成日期: 2026-05-28 | 源: ComposioHQ/agent-orchestrator v0.9.2 (7.3K⭐, 3,288 tests)*
*集成点: background_loop.rs:346 (tokio::select! loop) + goal_loop.rs:goal_ticker → orchestrator_ticker*
