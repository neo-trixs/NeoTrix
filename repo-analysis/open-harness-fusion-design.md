# OpenHarness → NeoTrix 架构融合设计

> **分析日期**: 2026-05-28
> **OpenHarness**: v0.1.9, 13.2K⭐, Python, ~11,733 LOC, 163 文件
> **NeoTrix**: v2, Rust, ~72,000 LOC, 286 Rust 文件, 1,248+ 测试

---

## 1. 项目概览

### OpenHarness

OpenHarness 是 Claude Code 的极轻量级 Python 复刻——以 3% 的代码量实现 ~80% 的核心 Agent 功能。本质是一个 **Agent Harness**（围绕 LLM 的完整基础设施：手、眼、记忆、安全边界）。提供 43 个工具、54 个命令、技能系统（SKILL.md）、插件生态、权限控制、多 Agent 协调，以及一个内置的个人助手 ohmo（支持飞书/Slack/Telegram/Discord）。

**定位**: 研究级轻量 harness，Python 生态，快速实验和扩展
**哲学**: "模型即 Agent，代码即 Harness"——模型提供智能，Harness 提供能力

### NeoTrix

NeoTrix 是一个 **Self-Improving Meta-Cognitive Agent System**——具备自我意识、能力向量进化、超立方体知识表示、全局工作空间注意力的下一代 AI Agent 框架。以 Rust 构建，强调类型安全、零成本抽象、编译期特性门控。

**定位**: 生产级自治 Agent 系统，自进化认知架构
**哲学**: 能力向量驱动的自我迭代，代数框架下的认知进化

---

## 2. 架构逐层对比

### 2.1 整体架构风格

| 维度 | OpenHarness | NeoTrix |
|------|-------------|---------|
| **语言** | Python 3.10+ | Rust (edition 2021) |
| **代码量** | ~11,733 LOC | ~72,000 LOC |
| **文件数** | 163 | ~296 (.rs) |
| **测试** | 114 单元/集成 + 6 E2E | 1,248+ |
| **架构层数** | 扁平 10 子系统 | 4 层严格分层 |
| **配置** | JSON + 环境变量 + TOML | 特性门控 + env + JSON |
| **打包** | pip / curl 一键安装 | cargo build / nix |
| **UI** | React/Ink TUI | CLI TUI (termion) |
| **CLI** | `oh` 命令 (54 子命令) | `neotrix` CLI (子命令) |
| **编译时间** | 零（解释型） | 增量秒级，全量分钟级 |
| **依赖管理** | uv/pip | cargo + feature gates |
| **运行时** | Python asyncio | tokio async runtime |
| **许可证** | MIT | MIT |

### 2.2 Agent 循环 (Engine / Agent Loop)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **流式 Tool-Call** | ✅ query→stream→tool_call→loop | ✅ ReasoningEngine·reason() | 等效 |
| **并行工具执行** | ✅ asyncio.gather | ✅ tokio::spawn | 等效 |
| **指数退避重试** | ✅ API Retry with Backoff | ✅ provider/ 内置重试 | 等效 |
| **Token 计数与成本追踪** | ✅ TokenCounting + CostTracking | ⚠️ event_bus 可追踪，无内置 | NeoTrix 缺失开箱计费 |
| **Dry-run 安全预览** | ✅ `oh --dry-run` | ❌ 无等效 | NeoTrix 缺失 |
| **最大轮次控制** | ✅ --max-turns | ✅ SEAL loop 内置 | 等效 |
| **Session 恢复** | ✅ /resume → history | ✅ background_loop | 等效 |

### 2.3 工具系统 (Toolkit / Tools)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **工具数量** | 43+ | 5 (内置) + MCP 可扩展 | NeoTrix 内置工具极少 |
| **文件操作** | Read/Write/Edit/Glob/Grep/Bash | patch.rs（手术式编辑） | OpenHarness 文件工具更丰富 |
| **Web 搜索** | WebSearch + WebFetch | web_miner.rs + ScraperEngine | 等效 |
| **MCP 协议** | ✅ MCP 客户端 (stdio+HTTP) | ✅ MCP Registry (rmcp 0.5) | 等效 |
| **Notebook 工具** | ✅ NotebookEdit | ❌ | NeoTrix 缺失 |
| **Cron 调度** | ✅ CronCreate/List/Delete | ❌ | NeoTrix 缺失 |
| **LSP 集成** | ✅ ToolSearch / LSP | ❌ | NeoTrix 缺失 |
| **Meta 工具** | ✅ Skill/Config/Brief/Sleep | ❌ | NeoTrix 缺失 |
| **Worktree 支持** | ✅ Worktree | ✅ worktree.rs | 等效 |
| **Pydantic 输入验证** | ✅ 全工具 Pydantic | ✅ Rust 类型系统 | 等效（Rust 更安全） |
| **工具生命周期钩子** | ✅ PreToolUse/PostToolUse | ⚠️ event_bus 可模拟 | NeoTrix 无原生钩子系统 |

### 2.4 技能系统 (Skills)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **技能定义格式** | SKILL.md Markdown | ❌ 无标准格式 | NeoTrix 缺失 |
| **按需加载** | ✅ on-demand skill loading | ✅ KnowledgeSource 枚举 | 概念不同，非直接映射 |
| **来源级联** | 内置→用户→项目→ohmo→插件 | 34 个 KnowledgeSource 枚举变体 | NeoTrix 更结构化但更静态 |
| **兼容 anthropic/skills** | ✅ 完全兼容 | ❌ | NeoTrix 缺失生态兼容 |
| **运行时注册** | ✅ 文件系统扫描 | ⚠️ custom_sources HashMap | NeoTrix 有但未完善 |
| **技能即命令** | ✅ `/skill_name` 直接调用 | ❌ | NeoTrix 缺失 |

### 2.5 命令系统 (Commands)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **命令数量** | 54 | ~10 | OpenHarness 命令更丰富 |
| **/plan** | ✅ | ✅ orchestrator/PlannerNode | 等效 |
| **/commit** | ✅ | ❌ 依赖外部 git | NeoTrix 缺失 |
| **/resume** | ✅ | ✅ background_loop | 等效 |
| **/skills** | ✅ list + invoke | ❌ | NeoTrix 缺失 |
| **/plugin** | ✅ install/enable/list | ❌ | NeoTrix 缺失 |
| **/provider** | ✅ list/use/add | ❌ provider/ 无 CLI | NeoTrix 缺失 |
| **/permissions** | ✅ 模式切换 | ❌ 无交互 CLI | NeoTrix 缺失 |
| **命令可扩展** | ✅ 插件注册 | ❌ | NeoTrix 缺失 |

### 2.6 内存系统 (Memory)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **跨会话持久化** | ✅ MEMORY.md | ✅ ReasoningBank (1805 LOC) | NeoTrix 远超 |
| **上下文压缩** | ✅ Auto-Compact | ⚠️ 无自动压缩 | OpenHarness 胜出 |
| **CLAUDE.md 注入** | ✅ 发现+注入 | ✅ AGENTS.md/CLAUD.md 注入 | 等效 |
| **向量检索** | ❌ | ✅ CapabilityVector cosine | NeoTrix 独有 |
| **衰退/遗忘** | ❌ | ✅ cortex_memory decay | NeoTrix 独有 |
| **超立方体知识** | ❌ | ✅ KnowledgeHyperCube 4096-dim VSA | NeoTrix 独有 |
| **记忆分层** | ❌ | ✅ Tier1/L1/offload/pipeline | NeoTrix 独有 |

### 2.7 安全与权限 (Governance / Security)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **权限模式** | Default/Auto/Plan | ✅ policy.rs + permission.rs | 等效 |
| **路径级规则** | ✅ path_rules JSON | ✅ Guard path rules | 等效 |
| **命令黑名单** | ✅ denied_commands | ✅ policy.rs | 等效 |
| **交互式审批** | ✅ y/n dialog | ✅ eventsystem 可对接 | 等效 |
| **Pre/Post 钩子** | ✅ PreToolUse/PostToolUse | ❌ 无标准钩子API | NeoTrix 缺失 |
| **密钥管理** | ✅ auth CLI + keyring | ✅ Vault + KeyVault | NeoTrix 更完整 |
| **审计日志** | ✅ 隐式 | ✅ Audit::operation_log | NeoTrix 更正式 |
| **沙箱执行** | ❌ | ✅ Wasm sandbox (feature) | NeoTrix 独有 |
| **代理网络** | ❌ | ✅ stealth_net 10K+ LOC | NeoTrix 独有 |

### 2.8 多 Agent 协调 (Swarm / Multi-Agent)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **子 Agent 生成** | ✅ Subagent Spawning | ✅ sub_agent.rs | 等效 |
| **团队注册与任务** | ✅ Team Registry | ✅ team.rs (Boss/AllVote/Chain/Devil) | NeoTrix 模式更多 |
| **后台任务生命周期** | ✅ Background Task Lifecycle | ✅ orchestrator DAG | NeoTrix 更复杂 |
| **协调器路由** | ✅ coordinator | ✅ Coordinator::route_task() | 等效 |
| **Agent 协议** | ❌ | ✅ agent_protocol UDP 发现 | NeoTrix 独有 |
| **辩论模式** | ❌ | ✅ debate ProcessType | NeoTrix 独有 |
| **Agent 模板** | ❌ | ✅ 9 agent presets | NeoTrix 独有 |
| **ClawTeam 集成** | 🔜 Roadmap | ❌ | 双方均未完成 |

### 2.9 提供者系统 (Providers / LLM)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **Anthropic** | ✅ | ✅ provider/anthropic | 等效 |
| **OpenAI** | ✅ | ✅ provider/openai | 等效 |
| **Copilot** | ✅ OAuth device flow | ❌ | NeoTrix 缺失 |
| **Codex** | ✅ subscription bridge | ❌ | NeoTrix 缺失 |
| **Ollama** | ✅ | ✅ | 等效 |
| **Gemini** | ✅ | ✅ | 等效 |
| **Moonshot/Kimi** | ✅ | ❌ | NeoTrix 缺失 |
| **GLM/Zhipu** | ✅ | ❌ | NeoTrix 缺失 |
| **MiniMax** | ✅ | ❌ | NeoTrix 缺失 |
| **NVIDIA NIM** | ✅ | ❌ | NeoTrix 缺失 |
| **Groq** | ✅ | ❌ | NeoTrix 缺失 |
| **Provider 即工作流** | ✅ profile-scoped | ❌ 单层配置 | OpenHarness 更灵活 |
| **兼容端点注册** | ✅ oh provider add | ❌ | NeoTrix 缺失 |

### 2.10 插件系统 (Plugins)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **插件定义** | JSON + .md | ❌ | NeoTrix 缺失 |
| **命令扩展** | ✅ commands/*.md | ❌ | NeoTrix 缺失 |
| **钩子扩展** | ✅ hooks/hooks.json | ❌ | NeoTrix 缺失 |
| **Agent 扩展** | ✅ agents/*.md | ❌ | NeoTrix 缺失 |
| **兼容 claude-code 插件** | ✅ 12 官方插件验证 | ❌ | NeoTrix 缺失 |
| **插件管理 CLI** | ✅ oh plugin list/install/enable | ❌ | NeoTrix 缺失 |

### 2.11 元认知与自我进化

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **自我模型** | ❌ | ✅ SelfModel + MetaCognitiveLoop | NeoTrix 独有 |
| **弱点分析器** | ❌ | ✅ WeaknessAnalyzer | NeoTrix 独有 |
| **进化规划** | ❌ | ✅ EvolutionPlanner | NeoTrix 独有 |
| **能力向量** | ❌ | ✅ CapabilityVector 22-dim | NeoTrix 独有 |
| **SEL 循环** | ❌ | ✅ SEAL loop (S-02~S-05) | NeoTrix 独有 |
| **超立方体 VSA** | ❌ | ✅ KnowledgeHyperCube 4096-dim | NeoTrix 独有 |
| **全局工作空间 GWT** | ❌ | ✅ GlobalWorkspace + competition | NeoTrix 独有 |
| **SiliconSelf 思维模型** | ❌ | ✅ 10 推理策略 + 10 注意力域 | NeoTrix 独有 |
| **RL 奖励学习** | ❌ | ✅ world_model.rl_iterate() | NeoTrix 独有 |

### 2.12 开发体验 (DX)

| 特性 | OpenHarness | NeoTrix | 差距分析 |
|------|-------------|---------|----------|
| **安装复杂度** | 一键 curl/bash | cargo build | OpenHarness 更易 |
| **设置向导** | ✅ `oh setup` 交互式 | ⚠️ env 变量 | OpenHarness 更友好 |
| **Dry-run 预览** | ✅ `oh --dry-run` | ❌ | NeoTrix 缺失 |
| **TUI** | ✅ React/Ink 全功能 | ⚠️ 基础 CLI TUI | OpenHarness 更精美 |
| **调试模式** | ✅ `--debug` | ⚠️ RUST_LOG=enabled | 等效 |
| **文档** | README + SHOWCASE + CONTRIBUTING | ARCHITECTURE_V2 + AGENTS.md | 等效 |
| **E2E 测试** | ✅ 22 E2E 场景 | ⚠️ 测试分散 | OpenHarness E2E 更系统 |
| **多语言支持** | ✅ 中英文 README | ❌ 仅中文 | 可忽略 |

---

## 3. 差距矩阵 (Gap Matrix)

### 3.1 OpenHarness 有 → NeoTrix 无 (NeoTrix 应吸收)

| 编号 | 特性 | 类别 | 影响力 | 紧急性 | 优先级 |
|------|------|------|--------|--------|--------|
| G-01 | **Plugin 插件系统**（claude-code 兼容） | 生态 | 高 | 中 | **P0** |
| G-02 | **54 个 CLI 命令**（/commit、/plan、/skills 等） | DX | 高 | 中 | **P0** |
| G-03 | **Hooks 生命周期系统**（Pre/PostToolUse） | 架构 | 高 | 高 | **P0** |
| G-04 | **Dry-run 安全预览**（`--dry-run`） | DX | 中 | 低 | P1 |
| G-05 | **Notebook 工具**（Jupyter 编辑） | 工具 | 中 | 低 | P1 |
| G-06 | **Cron 调度工具**（定时任务） | 工具 | 低 | 低 | P2 |
| G-07 | **LSP 集成**（语言服务器搜索） | 工具 | 中 | 低 | P1 |
| G-08 | **React TUI 全功能终端** | UX | 高 | 中 | **P0** |
| G-09 | **交互式设置向导**（oh setup） | DX | 中 | 中 | P1 |
| G-10 | **Cost Tracking 成本追踪** | 监控 | 中 | 低 | P2 |
| G-11 | **CLAUDE.md 自动发现与注入** | DX | 中 | 中 | P1 |
| G-12 | **Copilot/Codex 认证流** | Provider | 中 | 中 | P1 |
| G-13 | **上下文自动压缩 (Auto-Compact)** | Memory | 高 | 中 | **P0** |
| G-14 | **更多 LLM Provider**（Moonshot/GLM/MiniMax/NVIDIA/Groq） | Provider | 中 | 低 | P2 |
| G-15 | **技能即命令**（`/skill_name` 直接调用） | DX | 中 | 中 | P1 |

### 3.2 NeoTrix 有 → OpenHarness 无 (NeoTrix 差异化优势)

| 编号 | 特性 | 类别 | 说明 |
|------|------|------|------|
| N-01 | **CapabilityVector 22 维能力向量** | 核心 | 无等效 |
| N-02 | **KnowledgeHyperCube 4096-dim VSA** | 核心 | 无等效 |
| N-03 | **GlobalWorkspace 注意力路由** | 核心 | 无等效 |
| N-04 | **SEAL 自迭代循环** | 进化 | 无等效 |
| N-05 | **MetaCognition 元认知层** | 元 | 无等效 |
| N-06 | **SelfModel + WeaknessAnalyzer** | 自省 | 无等效 |
| N-07 | **Wasm Sandbox 沙箱执行** | 安全 | 无等效 |
| N-08 | **stealth_net 代理网络 (10K+ LOC)** | 网络 | 无等效 |
| N-09 | **SiliconSelf 思维模型** | 认知 | 无等效 |
| N-10 | **ReasoningBank 1805 行内存** | 记忆 | 无等效（更浅） |
| N-11 | **orchestrator DAG 编排** | 编排 | 无等效 |
| N-12 | **event_bus JSONL 事件系统** | 可观测 | 无等效 |
| N-13 | **agent_protocol UDP 发现** | 网络 | 无等效 |
| N-14 | **42 个 reasoning_brain 子模块** | 推理 | 无等效 |
| N-15 | **10 个编译期 feature gate** | 构建 | 无等效 |

### 3.3 双方都缺失

| 编号 | 特性 | 说明 |
|------|------|------|
| B-01 | **持久化 Agent 记忆共享**（多 Agent 共享记忆） | 两方均为单 Agent 中心 |
| B-02 | **性能基准套件** | 两者均无 benchmark CI |
| B-03 | **完整 API 服务模式** | OpenHarness 有 React TUI，NeoTrix 有 server/ 但均不完整 |
| B-04 | **OAuth 远程 Agent 认证** | 均缺失 |
| B-05 | **多租户支持** | 均面向单用户 |

---

## 4. 优先级分类 (Priority Classification)

### P0 — 必须吸收（高影响 + 中高紧急）

| 特性 | 吸收策略 | 预估工作量 |
|------|----------|-----------|
| **Plugin 系统** | 新建 `plugin/` 模块，定义 `.neotrix-plugin/plugin.json` 格式，实现扫描/注册/加载 | ~400 LOC |
| **Hooks 系统** | 在 `agent/hooks.rs` 实现 PreToolUse/PostToolUse 特征 + 注册表 | ~300 LOC |
| **React TUI** | 复用 server/ 已有 HTTP 接口，新建前端 `frontend/` 目录 | ~2000 LOC (前端) |
| **Auto-Compact 上下文压缩** | 在 `core/memory/pipeline.rs` 增加压缩策略 | ~250 LOC |
| **54 CLI 命令子集** | 在 `cli/commands/` 扩展 `/plan`, `/commit`, `/skills`, `/resume` 等 | ~800 LOC |

### P1 — 应吸收（中影响 + 低中紧急）

| 特性 | 吸收策略 | 预估工作量 |
|------|----------|-----------|
| **Dry-run 预览** | 在 `ReasoningEngine` 增加 `dry_run()` 模式，跳过工具执行 | ~150 LOC |
| **Notebook 工具** | 新建 `agent/tools/notebook.rs` | ~200 LOC |
| **LSP 集成** | 复用 MCP → MCP LSP server 连接 | ~150 LOC |
| **交互设置向导** | 扩展 `cli/` 增加 `neotrix setup` 交互式流程 | ~500 LOC |
| **CLAUDE.md 发现** | 复用 AGENTS.md 发现逻辑，扩展文件匹配 | ~100 LOC |
| **Copilot/Codex 认证** | 在 `provider/` 增加 OAuth device flow 实现 | ~400 LOC |
| **技能即命令** | 在 `cli/commands/` 增加 skills 注册和 dispatch | ~200 LOC |

### P2 — 可吸收（低影响 + 低紧急）

| 特性 | 吸收策略 | 预估工作量 |
|------|----------|-----------|
| **Cron 调度** | 在 `background_loop.rs` 增加 cron 触发器 | ~200 LOC |
| **Cost Tracking** | 在 `event_bus.rs` 增加 token 计数事件 | ~100 LOC |
| **更多 Provider** | 按 Moonshot/GLM/MiniMax/NVIDIA/Groq 顺序添加 | ~150 LOC each |

---

## 5. 集成点 (Integration Points with Existing Modules)

### 5.1 Plugin 系统 → 集成点

```
┌─ 新模块: agent/plugin/ ─────────────────────┐
│  mod.rs       → 插件路由 + 注册表              │
│  types.rs     → PluginManifest JSON 结构       │
│  loader.rs    → 文件系统扫描 + 加载             │
│  commands.rs  → 命令扩展注册                   │
│  hooks.rs     → 钩子扩展注册                   │
└──────────────────────────────────────────────┘
      │
      ├─→ agent/hooks.rs     (新增钩子系统)
      ├─→ agent/commands/    (命令注册)
      ├─→ agent/skills/      (技能扩展)
      └─→ core/knowledge.rs  (KnowledgeSource::Plugin 变体)
```

### 5.2 Hooks 系统 → 集成点

```
┌─ 新模块: agent/hooks.rs ─────────────────────┐
│  HookEvent 枚举: PreToolUse / PostToolUse     │
│  HookRegistry: 注册 + 触发                    │
│  HookHandler trait                            │
└──────────────────────────────────────────────┘
      │
      ├─→ agent/tool/mod.rs   (工具执行前后调用)
      ├─→ agent/tool/lifecycle.rs (集成生命周期)
      └─→ event_bus.rs        (钩子事件记录)
```

### 5.3 CLI 命令扩展 → 集成点

```
┌─ 扩展: cli/commands/ ────────────────────┐
│  mod.rs   → 路由                          │
│  plan.rs  → orchestrator + SEAL          │
│  commit.rs→ git MCP + patch              │
│  skills.rs→ KnowledgeSource 列表+加载     │
│  resume.rs→ background_loop 恢复          │
│  provider.rs→ provider 管理 CLI          │
└──────────────────────────────────────────┘
      │
      ├─→ reasoning_brain/goal_loop.rs
      ├─→ agent/tools/patch.rs
      ├─→ core/knowledge.rs
      ├─→ background_loop.rs
      └─→ provider/ mod
```

### 5.4 Auto-Compact → 集成点

```
┌─ 扩展: core/memory/pipeline.rs ─────────┐
│  CompactStrategy 枚举:                    │
│    TurnBased / TokenUsage / TimeBased    │
│  compact() → 摘要 + 压缩                 │
└──────────────────────────────────────────┘
      │
      ├─→ core/memory/l1.rs    (L1 触发)
      ├─→ core/memory/tier.rs  (层级联动)
      └─→ reasoning_brain/reasoning_engine.rs (压缩请求)
```

### 5.5 React TUI → 集成点

```
┌─ 新目录: frontend/ ─────────────────────┐
│  package.json                           │
│  src/App.tsx    (React TUI)              │
│  src/Stream.tsx (流式输出)               │
│  src/CmdPicker.tsx (/ 命令选择器)        │
│  src/PermDialog.tsx (权限对话框)          │
└──────────────────────────────────────────┘
      │
      └─→ server/ws.rs    (WebSocket 流)
      └─→ server/http.rs  (REST API)
```

---

## 6. KnowledgeSource 注册方案

以下新特性引入需要注册新的 KnowledgeSource 变体：

### 6.1 新增枚举变体

```rust
// core/knowledge.rs
pub enum KnowledgeSource {
    // ... 现有 34+ 变体
    
    /// Plugin ecosystem — 兼容 claude-code 插件格式
    PluginSystem,
    
    /// CLI commands system — 54 命令操作模式
    CommandPalette,
    
    /// Hook lifecycle — PreToolUse / PostToolUse 事件
    HookLifecycle,
    
    /// Notion/Jupyter notebook editing
    NotebookTooling,
    
    /// LSP integration — language server protocol
    LspIntegration,
    
    /// Cron/scheduled tasks
    CronScheduler,
}
```

### 6.2 CapabilityVector 映射

```rust
impl KnowledgeSource {
    pub fn capability_vector(&self) -> CapabilityVector {
        match self {
            KnowledgeSource::PluginSystem => CapabilityVector::from_values([
                ("extensibility", 0.92),
                ("ecosystem_compat", 0.90),
                ("hook_customization", 0.85),
                // ... 默认 0.0 for 其余
            ]),
            KnowledgeSource::CommandPalette => CapabilityVector::from_values([
                ("cli_richness", 0.95),
                ("user_workflow", 0.90),
                ("tool_discovery", 0.88),
            ]),
            KnowledgeSource::HookLifecycle => CapabilityVector::from_values([
                ("lifecycle_governance", 0.93),
                ("observability", 0.88),
                ("security_enforcement", 0.85),
            ]),
            KnowledgeSource::NotebookTooling => CapabilityVector::from_values([
                ("data_science", 0.90),
                ("interactive_compute", 0.88),
            ]),
            KnowledgeSource::LspIntegration => CapabilityVector::from_values([
                ("code_intelligence", 0.92),
                ("language_awareness", 0.90),
            ]),
            KnowledgeSource::CronScheduler => CapabilityVector::from_values([
                ("automation", 0.88),
                ("time_awareness", 0.85),
            ]),
            _ => self.capability_vector(), // 委托现有
        }
    }
}
```

### 6.3 source_weight 优先级

```rust
impl KnowledgeSource {
    pub fn source_weight(&self) -> f64 {
        match self {
            KnowledgeSource::PluginSystem => 0.90,    // P0 — 高优先级吸收
            KnowledgeSource::HookLifecycle => 0.88,   // P0
            KnowledgeSource::CommandPalette => 0.85,  // P0
            KnowledgeSource::NotebookTooling => 0.60, // P1
            KnowledgeSource::LspIntegration => 0.55,  // P1
            KnowledgeSource::CronScheduler => 0.30,   // P2
            _ => self.source_weight(),
        }
    }
}
```

---

## 7. 测试策略

### 7.1 Plugin 系统测试

| 测试 | 类型 | 描述 |
|------|------|------|
| `test_plugin_discovery` | 单元 | 扫描 `.neotrix-plugin/` 目录发现 3 个插件 |
| `test_plugin_manifest_parse` | 单元 | 解析 `plugin.json` 含/不含 commands/hooks/agents |
| `test_plugin_command_registration` | 集成 | 插件注册命令，CLI 可路由 |
| `test_plugin_hook_execution` | 集成 | PreToolUse 钩子修改工具参数 |
| `test_plugin_isolation` | 单元 | 错误插件不会崩溃宿主 |

### 7.2 Hooks 系统测试

| 测试 | 类型 | 描述 |
|------|------|------|
| `test_pre_tool_hook` | 单元 | PreToolUse 钩子拦截并修改参数 |
| `test_post_tool_hook` | 单元 | PostToolUse 钩子记录结果 |
| `test_hook_chain_order` | 单元 | 多钩子按注册顺序执行 |
| `test_hook_error_handling` | 单元 | 钩子失败不阻塞工具执行 |

### 7.3 CLI 命令测试

| 测试 | 类型 | 描述 |
|------|------|------|
| `test_plan_command` | 集成 | `/plan` 触发 orchestrator 规划 |
| `test_commit_command` | 集成 | `/commit` 调用 git 工具 |
| `test_skills_command` | 集成 | `/skills` 列出 KnowledgeSource |
| `test_resume_command` | 集成 | `/resume` 从 checkpoint 恢复 |

### 7.4 Auto-Compact 测试

| 测试 | 类型 | 描述 |
|------|------|------|
| `test_turn_based_compact` | 单元 | 每 N 轮触发压缩 |
| `test_token_based_compact` | 单元 | Token 超限触发压缩 |
| `test_compact_lossless` | 集成 | 压缩后关键信息不丢失 |
| `test_compact_recovery` | 集成 | 压缩后恢复原始上下文 |

### 7.5 E2E 场景

| 场景 | 描述 |
|------|------|
| `test_plugin_install_and_use` | 安装插件 → 使用插件命令 → 验证输出 |
| `test_hook_security_audit` | 注册安全审计钩子 → 触发文件编辑 → 验证拦截 |
| `test_auto_compact_long_session` | 长时间会话 → 自动压缩 → 继续对话 |
| `test_cli_plan_to_commit` | `/plan` → 执行 → `/commit` 完整工作流 |

---

## 8. 实施路线图

### Phase 1: P0 核心 (2-3 sessions)

```
Session 1: Hooks 系统 + Plugin 系统基础
  → agent/hooks.rs           (HookEvent, HookRegistry)
  → agent/plugin/mod.rs      (PluginManifest, 扫描)
  → KnowledgeSource 注册     (PluginSystem, HookLifecycle)
  → cargo check --lib        (0 errors)
  
Session 2: CLI 命令扩展 (Top 10)
  → cli/commands/{plan,commit,skills,resume,provider}.rs
  → cli/commands/mod.rs 路由
  → cargo check --lib        (0 errors)

Session 3: Auto-Compact + 测试
  → core/memory/pipeline.rs  压缩策略
  → 全部 P0 测试
  → cargo test --lib         全部 passing
```

### Phase 2: P1 增强 (2 sessions)

```
Session 4: Dry-run + Notebook + LSP
  → ReasoningEngine.dry_run()
  → agent/tools/notebook.rs
  → MCP LSP 桥接

Session 5: 设置向导 + CLAUDE.md
  → cli/setup.rs
  → core/file_parser.md 扩展
  → Copilot/Codex auth flow
```

### Phase 3: P2 完善 (1 session)

```
Session 6: Cron + Cost + 更多 Provider
  → background_loop cron
  → event_bus token counting
  → provider/moonshot, provider/glm, provider/minimax
```

---

## 9. 总结

| 维度 | 评估 |
|------|------|
| **总体匹配度** | OpenHarness 是 NeoTrix 最接近的参考架构，覆盖了 NeoTrix 缺失的 DX 层 |
| **最大差距** | Plugin 系统、Hooks 生命周期、CLI 命令丰富度 |
| **NeoTrix 核心优势** | 元认知、自迭代、超立方体、能力向量——OpenHarness 完全没有 |
| **吸收策略** | 直接吸收 OpenHarness 的 Plugin/Hooks/Commands 模式，用 Rust 重写 |
| **生态兼容** | Plugin 系统应兼容 claude-code 插件格式以获取即用生态 |
| **建议** | 优先吸收 Plugin + Hooks + Command，这三者形成完整的外部扩展契约 |

---

*生成工具: opencode + webfetch + ARCHITECTURE_V2.md 分析*
*源码: https://github.com/HKUDS/OpenHarness*
