# NeoTrix Desktop V2 — Changelog

## v0.25.4 (2026-09-17)

### Universal Model Gateway — 通用模型网关 + 全流程实现

> 5个 Phase 并行完成: Model Gateway → Provider Registry → Semantic Router → Agent Handoff → MCP Bridge

#### Phase 1: Model Gateway 核心 (nt_core_model_gateway.rs)
- `ModelGateway` — 统一模型网关入口
- `CostGate` — 成本门控 (月预算 + 模型预算)
- `FallbackChain` — 降级链 (coding/reasoning/simple 三条链)
- `ProviderPool` — 连接池管理
- `GatewayRequest` / `GatewayResponse` — 统一请求/响应
- 7 单元测试

#### Phase 2: Provider Registry (nt_core_byoa.rs)
- `ProviderInfo` — Provider 元数据结构
- `ProviderType` — Api / Cli / Local 枚举
- 5 预设 Provider: Claude Code / Codex / Gemini CLI / Ollama / GPT-4o
- `register_provider()` / `list_providers()` / `check_providerAvailability()`
- 4 单元测试

#### Phase 3: Semantic Router (nt_core_semantic_router.rs)
- `SemanticRouter` — 语义路由器 (confidence-based dispatch)
- `SemanticRouteDecision` — 路由决策结构
- 置信度阈值: 高→首选模型, 低→fallback 强模型
- 预设: coding→claude-code, reasoning→gpt-4o, simple→gemini-flash
- 4 单元测试

#### Phase 4: Agent Handoff (nt_core_hive.rs)
- `delegate_task()` — 委派任务给指定 agent
- `delegate_to_best()` — 基于能力匹配委派
- `collect_result()` — 收集任务结果
- `agent_status_summary()` — Agent 状态摘要
- `AgentStatus` 结构体
- 3 单元测试

#### Phase 5: MCP Bridge (nt_io_mcp_bridge.rs)
- `McpBridge` — MCP 协议桥接
- `McpTool` / `McpToolCall` / `McpToolResult` — MCP 工具类型
- `McpServer` / `McpTransport` — 外部 MCP 服务器
- 预注册: kb_query / kb_search / experience_query / agent_list
- 4 单元测试

---

## v0.25.3 (2026-09-17)

### 架构巡检 + 关键修复

> 多 Agent 巡检发现26个问题，修复3个关键架构缺陷

#### 巡检结果
- **10 冗余**: 重复类型、facade 过度 re-export
- **6 扁平架构**: mega files、L5 有40+平铺模块
- **10 跨域错位**: L6 元认知在 L5、L1→L5 反向依赖

#### 关键修复

##### Fix 1: SelfModel 三重定义合并
- `l6_meta/self_model_unified.rs` — 统一 SelfModel facade
- 合并: 静态身份 + 动态性能 + 价值函数
- 旧路径 re-export 保持向后兼容

##### Fix 2: L6 元认知模块归位
- 9个文件从 `l5_cognition/nt_core/nt_meta/` → `l6_meta/nt_meta/`
- 旧路径 re-export 保持向后兼容

##### Fix 3: L1→L5 反向依赖消除
- `nt_core_task_dispatcher.rs` 定义 `ReasoningEngineProvider` trait
- 消除 L1 对 L5 具体类型的直接 import

---

## v0.25.2 (2026-09-17)

### 冗余清理 + 架构重构

> 消除13处 now_secs() 重复、合并5个 stub facade、修正 agent_circuit_breaker 跨域错位

#### 冗余清理

##### now_secs() 去重 (13→1)
- `l0_substrate/nt_core_time.rs` — 共享时间工具模块
- `now_secs()` / `now_secs_i64()` / `now_millis()`
- 替换13个模块中的重复实现，改为 `use crate::l0_substrate::nt_core_time::now_secs`

##### Stub Facade 合并 (6→1)
- `l5_cognition/layer_aliases.rs` — 统一层别名模块
- `act_facade`, `io_facade`, `kb_facade`, `l2_facade`, `l3_facade`, `io_skills_facade` → 全部指向 `layer_aliases`
- 更新39处 import 为统一路径

#### 跨域修正

##### Agent Circuit Breaker 归位 (L3→L5)
- `nt_shield_agent_circuit_breaker.rs` → `nt_core_agent_circuit_breaker.rs`
- Agent 行为控制 (steer→constrain→stop) 是认知层职责，非安全基础设施
- 从 L3 Shield 移至 L5 Cognition

---

## v0.25.1 (2026-09-17)

### Hive 真实数据集成 + BYOA 进程池

> Phase 3.5: HiveRouter → OfficeFloor 真实数据、BYOA 长连接进程池

#### 后端 (Rust)

##### HiveAgentLoop (nt_io_hive_agent_loop.rs)
- `HiveAgentLoop` — AgentLoop wrapper with inbox/outbox/blackboard coordination
- `coordinated_turn()` — drain inbox → inject context → run turn → post to blackboard
- `send_to()` / `broadcast()` — send messages to other agents
- `bb_read()` / `bb_write()` — blackboard access

##### Office Floor Tauri Commands (commands/hive.rs)
- `hive_get_floor_state` — returns agents + messages + stats
- `hive_send_message` — send test messages between agents

##### BYOA Process Pool (nt_core_byoa.rs)
- `PooledSession` — long-lived process with open stdin/stdout
- `acquire_pooled()` / `send_pooled()` / `release_pooled()`
- `evict_idle_pooled()` — cleanup idle sessions (configurable timeout)
- 4 new pool tests

#### 前端 (TypeScript)

##### OfficeFloor 真实数据
- `api/hive.ts` — TypeScript API bindings
- `OfficeFloor` now supports `useRealData` prop

---

## v0.25.0 (2026-09-17)

### Cumora + Munder Difflin 吸收 — Agent 一等公民 + 安全控制 + 记忆增强

> 吸收 [cumora](https://github.com/yetone/cumora) agent-as-teammate + [munder-difflin](https://github.com/chaitanyagiri/munder-difflin) GOD agent/circuit breaker/gallery 模式。

#### 后端 (Rust)

##### P0: Agent Identity System (nt_agent_identity.rs)
- `AgentPersona` — 一等公民身份: id/name/avatar/specialty/personality/autonomy/cost_budget/provider/model
- `AutonomyLevel` enum — ReadOnly / Constrained / Supervised / Autonomous
- `AgentStatus` enum — Available / Busy / Unavailable / Disabled
- `AgentIdentityRegistry` — 注册/查询/选择/序列化，KB `agent_identity` namespace 持久化
- `AgentPreset` enum — 6 预设角色模板 (编码员/审查员/研究员/调试员/架构师/写作者)
- `SpendGate` + `SpendVerdict` — per-session/daily/lifetime 成本阈值 → 人类审批
- `select_best()` — 专长匹配 + 可用性 + 成本排序选择最佳 agent
- 10 单元测试

##### P0: Agent Circuit Breaker (nt_shield_agent_circuit_breaker.rs)
- `AgentCircuitBreaker` — steer→constrain→stop 三级行为控制 (区别于 service circuit breaker)
- `BreakerLevel` — Normal / Steer / Constrain / Stop
- `BreakerConfig` — 可配阈值: 连续失败/成本/工具轮次/冷却时间
- `BreakerVerdict` — Proceed / Steer{suggestion} / Constrain{allowed_tools} / Stop{reason, cooldown}
- `human_override()` — 人类一键重置 agent 到 Normal
- 9 单元测试

##### P0: GOD Agent Router (nt_core_god_agent.rs)
- `GodAgent` — 中央路由编排器: 分类任务→选择 persona→调度模型→熔断管理→人类升级
- `TaskClassifier` — 关键词规则分类 (10 种任务类型)
- `AgentSelectionConfig` — 专长偏好/成本上限/自治级别
- `EscalationRules` — 停止/成本/模糊 → 人类升级
- `RoutingDecision` — 完整路由记录 (分类/选中 agent/模型/是否升级)
- 7 单元测试

##### P1: Agent Gallery (nt_agent_gallery.rs)
- `AgentGallery` — 浏览/安装/卸载预设 agent 角色
- `GalleryPreset` — 扩展预设 (builtin + community)
- `register_community()` — 社区贡献预设
- `install()` → `AgentPersona` — 从预设安装到 registry
- `stats()` — 预设/安装/社区统计
- 6 单元测试

##### P2: Memory Palace 语义索引 (memory_palace.rs 增强)
- 关键词倒排索引 — item 入库时自动提取关键词
- `recall_semantic(query, limit)` — 关键词匹配 + strength 加权排序
- `semantic_search(query, limit)` — Jaccard 相似度 × strength 综合评分
- `keyword_stats()` — 索引统计
- 向后兼容: 原有 `recall/strengthen/weakest_items` API 不变
- 4 新增单元测试

##### P2: Hive Coordination Protocol (nt_core_hive.rs)
- `HiveRouter` — 中央路由: 消息投递 + 黑板 + 事件日志
- `AgentMailbox` — per-agent inbox/outbox (FIFO, 溢出自动丢弃旧消息)
- `Blackboard` — 共享黑板 (key-value + TTL + LRU 淘汰 + 前缀查询)
- `EventLog` — append-only 事件日志 (按类型/agent 过滤)
- `HiveMessage` — agent 间消息 (Task/Result/Query/Response/Status/Escalation/Broadcast)
- 8 单元测试

#### Tauri 后端 (src-tauri)

##### P1: Onboarding Wizard (commands/onboarding.rs)
- `onboarding_check_prereqs()` — 检查 git/node/npm/cargo/rustc/python3/ffmpeg/ollama
- `onboarding_get_tips()` — 快速入门提示
- `onboarding_complete()` — 标记首次运行完成
- `onboarding_is_completed()` — 检查是否已完成引导

#### 前端 (SolidJS)

##### P1: Onboarding Wizard (OnboardingWizard.tsx + OnboardingWizard.css)
- 首次运行覆盖层: 依赖检查 → 系统信息 → 快速入门 → 完成
- 每个工具显示: 安装状态 + 版本 + 安装提示 + 是否必需
- CSS 动画: spinner + fadeIn

##### P1: KanbanAgentBoard → RightBar 集成
- 替换原有静态占位符为 `KanbanAgentBoard` 组件
- 支持拖拽、暂停、停止、选择 agent
- 5 列看板: 等待/运行/审查/完成/出错

#### 模块注册
- `l6_meta/mod.rs`: + `nt_agent_identity`, `nt_agent_gallery`
- `l5_cognition/mod.rs`: + `nt_core_god_agent`, `nt_core_hive`
- `l3_embodiment/nt_shield/mod.rs`: + `nt_shield_agent_circuit_breaker`
- `src-tauri/commands/mod.rs`: + `onboarding`
- `src-tauri/main.rs`: + 4 onboarding commands

##### P2: Office Floor 可视化 (OfficeFloor.tsx)
- 2D top-down agent 工位地图 (SVG)
- 每个 agent = desk circle + avatar + status ring + activity animation
- 通信粒子动画 (task/result/query/escalation)
- Hover 显示 specialty 标签
- 图例: 状态颜色 + 消息类型

##### P2: RightBar Kanban ↔ Floor 视图切换
- agents tab 增加 "看板/地图" 切换按钮
- KanbanAgentBoard: 5列看板 + 拖拽
- OfficeFloor: 2D 工位地图 + 动画

##### P2: BYOA — Bring Your Own Agent (nt_core_byoa.rs)
- `ExternalAgentConfig` — CLI 命令/参数/环境变量/通信模式
- `ExternalAgentManager` — 注册/会话管理/成本追踪
- 3 预设: Claude Code / Codex / Gemini CLI
- `send_message()` — stdio 通信
- `check_availability()` — 检查 CLI 是否可用
- `availability_status()` — 全量可用性报告
- 6 单元测试

##### Hive → AgentLoop 集成 (nt_io_hive_agent_loop.rs)
- `HiveAgentLoop` — AgentLoop wrapper with inbox/outbox/blackboard coordination
- `coordinated_turn()` — drain inbox → inject context → run turn → post to blackboard
- `send_to()` / `broadcast()` — send messages to other agents
- `bb_read()` / `bb_write()` — blackboard access
- Non-invasive decorator pattern: core AgentLoop unchanged

##### Office Floor 真实数据 (Tauri commands)
- `hive_get_floor_state` — returns agents + messages + stats from HiveRouter
- `hive_send_message` — send test messages between agents
- `api/hive.ts` — TypeScript API bindings
- `OfficeFloor` now supports `useRealData` prop — fetches real data on mount

##### BYOA Process Pool (nt_core_byoa.rs)
- `PooledSession` — long-lived process with open stdin/stdout
- `acquire_pooled()` — get or create persistent session
- `send_pooled()` — send message via pooled session (no respawn)
- `release_pooled()` — kill pooled session
- `evict_idle_pooled()` — cleanup idle sessions (configurable timeout)
- `pool_stats()` — per-agent pool usage
- 4 new pool tests (timeout, stats, acquire, evict)

---

## v0.24.0 (2026-09-17)

### Cumora 吸收 — P0/P1/P2/P3 后端增强 + 桌面 App 改进

> 吸收 [cumora](https://github.com/yetone/cumora) 协调/写安全/计算设备抽象模式到 Rust 后端 + 前端。

#### 后端 (Rust)

##### P0: Hold-Token 门控覆盖 (write_guard_types.rs)
- `HoldToken` + `HoldTokenStore` (TTL + GC + atomic consume)
- `WriteGuardVerdict::Hold` variant — hold 期间阻止覆盖
- `compute_scope(action, payload)` 确定性 scope 判定
- `should_allow_override(force, has_hold_token)` 覆盖门控
- 8 单元测试

#### P1: Deterministic Fallback + Verbatim-Dup (nt_memory_write_guard_fallback.rs)
- `deterministic_fallback()` — classifier 不可用时最窄确定性放行
- `verbatim_content_fingerprint()` — 标准化 + hash 用于去重
- `check_verbatim_dup()` — KB 写前原子 verbatim 去重检测
- `has_hold_conflict()` — hold 冲突检测
- 12 单元测试

#### P2: Shape-Level 协调原则 (nt_core_coordination_principles.rs)
- 5 条 shape-level 原则 (HumanAddresses / ReplyFromPosted / PostOptimistic / NoRepeatStopDone / NeverClaimSlot)
- `ShapeLevelCoordinator::decide()` — 纯函数无状态协调引擎
- 7 单元测试

#### P3: Computer First-Class Abstraction (nt_computer.rs)
- `NtComputer` trait — 文件系统 + 进程 + 系统信息 + 屏幕抽象
- `LocalComputer` — 本地实现
- `MockComputer` — 测试 mock (预编程文件/命令)
- 6 单元测试

#### 依赖变更
- `neotrix-types/Cargo.toml`: + `once_cell = "1"`

#### 前端 (SolidJS)

##### P0: Hold-Token + Verbatim-Dup 防重复发送 (chat.ts + Chat.tsx)
- `acquireHold/releaseHold/isHeld/gcHolds` — 会话级 hold token (30s TTL)
- `contentFingerprint()` — SHA-256 内容指纹用于 verbatim-dup 检测
- `sendMessage` 发送前检查 hold 状态 + 重复消息 warning
- 所有流式路径 (onDone/onError/handleStop/watchdog) 释放 hold token

##### P1: Deterministic Fallback + 自动重试 (errorRootCause.ts + Chat.tsx)
- `RootCause` 增加 `retryAfterMs/suggestFallback/suggestCompact` 字段
- 新增 503 overloaded → 自动切换备用模型规则
- 429 rate limit → 5s 自动重试
- timeout/network → 3s 自动重试
- `scheduleRetry/cancelRetry` — 最多 2 次自动重试 + 倒计时 UI
- 错误 toast 显示重试倒计时 + 取消按钮

##### P2: RightBar Agents Tab (RightBar.tsx)
- 新增 "协调" tab — 多 agent 协调状态可视化
- 显示 agent 状态、Shape-Level 原则、Hold Token 状态

#### 桌面 App UX 增强 (Cumora 吸收)

##### P0: Hold-Token 可视化 (Chat.tsx + Sidebar.tsx)
- 发送按钮 pulsing amber 边框 + 倒计时 tooltip
- 输入框上方 hold 倒计时横条 (30s 进度条)
- 输入框 placeholder 变化 "消息处理中…"
- Sidebar 会话行 amber 圆点指示器
- 状态栏 "Hold · 12s" 实时倒计时

##### P0: Verbatim-Dup 用户可见提示 (Chat.tsx)
- 输入框上方黄色提示条 "与上条消息内容相同"
- 5s 自动消失 + "忽略" 按钮
- sendMessage 逻辑: 指纹比对 → setDupWarning

##### P1: 统一 OS 状态 Badge (Chat.tsx)
- 状态栏左侧统一 badge: 🟢 空闲 / 🧠 思考中 · domain / ⚡ 生成中 / 🔧 工具调用
- 替换原有碎片化显示 (权限模式 + 上下文 + 模型)
- Hold 状态优先显示

##### P1: 上下文预算可视化条 (Chat.tsx)
- 状态栏上下文百分比增加细进度条
- ≥80% 红色 + 可点击 compact
- 替换原有纯数字显示

##### P2: 消息阶段分隔线 (Chat.tsx)
- 工具调用完成后插入 "── 工具调用完成 ──" 分隔线
- 减少视觉噪音，保持对话结构清晰

---

## v0.23.1 (2026-09-09)

### UI 交互修复

#### ModelSwitcher — 点击空白处关闭弹窗
- **问题**: 模型选择弹窗点击空白处无法关闭
- **修复**: 添加 `document.addEventListener('mousedown', handleClickOutside)` 监听，点击弹窗外区域自动关闭
- **涉及文件**: `src/components/ModelSwitcher.tsx`

#### SettingsModal — 交通灯点击无反应
- **问题**: 设置界面左上角三色交通灯点击无交互反应
- **根因**: `data-tauri-drag-region` 属性导致 Tauri 拦截所有鼠标事件，按钮无法接收 click
- **修复**: 
  - 移除 TrafficLights 组件上的 `data-tauri-drag-region` 属性
  - 提升 `.traffic` z-index 到 9999 确保在 modal 之上
- **涉及文件**: `src/components/TrafficLights.tsx`, `src/styles/index.css`

### Free LLM 池子 + 代理池修复

#### Free LLM 池子 — 本地 OpenAI-compatible 模型未被识别为免费
- **问题**: `provider_pool.toml` 中 `provider = "openai"` 的本地模型被 `is_free_provider()` 过滤掉
- **根因**: `is_free_provider()` 硬编码只认 `llamacpp | groq | openrouter | siliconflow`
- **修复**: 
  - 新增 `is_free_provider_with_url()` 函数：如果 `base_url` 指向 `127.0.0.1`/`localhost`/`0.0.0.0`，自动标记为免费
  - 所有 `is_free_provider` 调用点统一迁移到 `is_free_provider_with_url()`
- **涉及文件**: `src-tauri/src/domain/plugins/stubs.rs`

#### 模型池 — 只显示主 provider，忽略 pool entries
- **问题**: `read_provider_config()` 只读 `config.toml`，不读 `provider_pool.toml`
- **修复**: `read_provider_config()` 合并 pool entries 到 `providers[]` 数组
- **效果**: 模型池列表从 1 个 provider 增加到 2 个（llamacpp + qwen3.5-9b-fable）
- **涉及文件**: `src-tauri/src/domain/plugins/stubs.rs`

#### 网络代理订阅 IP 池子 — cache 不存在导致返回空
- **问题**: `proxy_pool_status` 从 `proxy_pool_cache.json` 读取，但该文件不存在 → 返回 0 节点
- **修复**: 
  - Cache 不存在时，从 `subscriptions.json` 中提取 `http://ip:port` 格式的直连代理节点
  - 新增 `extract_direct_proxies()` 和 `infer_geo_from_ip()` 辅助函数
  - 首次提取后写入 cache，后续读 cache
- **涉及文件**: `src-tauri/src/commands/proxy_pool.rs`

## v0.19.0-rc2 (2026-09-08)

### Chat Plugin: 对话打通 + Tauri 事件发射
- **问题**: 前端调用 `neocodex_send_message_stream` 报 "Command not found"；流式事件未发射导致前端卡在生成中
- **根因**: ChatPlugin 只有 `send` action，没有 `send_message_stream`/`stop_stream`/`get_session_messages`；无 Tauri 事件发射
- **修复**: 
  - ChatPlugin 新增 3 个 action，`send_message_stream` 调用本地 llama.cpp (`http://127.0.0.1:8080/v1/chat/completions`)
  - 使用 `OnceLock<AppHandle>` 存储 app handle，通过 `setup` hook 注入
  - `call_llm` 发射 `neocodex_stream_start`/`neocodex_stream_token`/`neocodex_stream_end`/`neocodex_stream_done` 事件
- **涉及文件**: `src-tauri/src/domain/plugins/chat.rs`, `src-tauri/src/domain/plugins/mod.rs`, `src-tauri/src/main.rs`

### 意识核心集成完成
- **目标**: 将 chat plugin 连接到 neotrix-core 的意识核心 (ConsciousnessCore)，实现任务分解 → 模型路由 → 自动执行
- **修复**: 
  - 修复 neotrix-core 编译错误 (kanban_cmds.rs: `let mut mgr` + 重复 Mutex import)
  - 添加 neotrix crate 作为 Tauri app 依赖
  - 实现 `LlmPoolExecutor` (SolutionExecutor trait)，从 config.toml 读取模型，调用本地/远程 LLM
  - `call_llm` 使用 `CORE.write().execute_task_loop()` 执行完整任务闭环
- **当前状态**: 意识核心已集成，chat plugin 可通过 consciousness core 执行任务分解 → 模型路由 → 执行

### ModelSwitcher: 动态模型池加载
- **问题**: 模型名硬编码为 `neotrix-core` 或 GGUF 文件名
- **修复**: ModelSwitcher 并行加载 `providerConfig()` (config.toml) + `getModelPoolStatus()` (provider_pool.toml)，合并去重后动态展示所有可用模型
- **涉及文件**: `src/components/ModelSwitcher.tsx`

### Theme Consistency Fix
- **问题**: 对话界面 (main) 变黑，左侧栏 (sidebar) 保持白色，主题不一致
- **根因**: `body` 背景用硬编码渐变，`glass-side` 用硬编码 `#fbfbfa`，`--color-canvas`/`--color-panel` 变量存在但未被任何组件引用
- **修复**:
  - `body` 背景改为 `var(--color-canvas, #ffffff)` — 主题变量控制
  - `.glass-side` 背景改为 `var(--color-panel, #f9fafb)` — 主题变量控制
  - `:root` 定义完整 surface tokens（canvas/panel/line/ink 等），所有主题继承
  - 删除死文件 `src/style/main.css`（v4 Tailwind 暗色主题，从未 import）
  - 删除死文件 `src/styles/design-tokens.css`（从未 import）
- **涉及文件**: `src/styles/index.css`

### Model Name Display Fix
- **问题**: 模型切换器显示原始 GGUF 文件名 `Agents-A1-4B-kimi-Preview-heretic-IQ4_NL`，截断后为 `Agents-A1-4B-k...`
- **修复**: `pillModel()` 去掉 GGUF 后缀 (`heretic`/`IQ4_NL`/`gguf`)，`-`/`_` 转空格，显示 `Agents A1 4B kimi Preview`
- **涉及文件**: `src/components/ModelSwitcher.tsx`

### Tauri Window Rounded Corners Fix
- **问题**: 窗口失去圆角
- **根因**: `html`/`body` 缺少 `overflow: hidden` + `border-radius: 12px`，与 Tauri `windowEffects.radius: 12` 不匹配
- **修复**: `html` 和 `body` 都加了 `overflow: hidden; border-radius: 12px`
- **涉及文件**: `src/styles/index.css`

### Agent Stub Plugin (Real Data)
- **问题**: Agent 插件 `provider_config`/`provider_status`/`pool_sufficiency`/`discover_models` 返回空 stub
- **修复**: 读取 `~/.config/neotrix/config.toml` 返回真实数据
- **涉及文件**: `src-tauri/src/domain/plugins/stubs.rs`

---

## v0.19.0-rc1 (2026-09-08)

### Architecture
- 12 Domain Plugins registered via `DomainRegistry`
- All frontend API routed through `adapter.ts` → `enhancedInvoke` → `domain_call`
- Tauri 2 with `decorations: false`, `transparent: true`, `windowEffects.radius: 12`

### API Routing Migration
- 9 API files changed from `call()` to `enhancedInvoke()`
- Adapter `DOMAIN_MAP` expanded with `memory_*`, `kb_doc_*`, `save_api_key`/`has_api_key`/`delete_api_key`
- `mapAction()` strips prefixes: `neocodex_`, `kb_`, `canvas_`, `provider_`, `pool_`, `discover_`, `probe_`, `memory_`

### Config
- `~/.config/neotrix/config.toml`: `provider = "llamacpp"`, `default_model = "Agents-A1-4B-kimi-Preview-heretic-IQ4_NL"`

---

## Regression Rules

1. **CSS 变量单一事实源**: 所有背景/边框/文字色必须用 CSS 变量，禁止硬编码 hex
2. **主题继承**: `:root` 定义默认值，`[data-theme]` 仅覆盖 accent 色
3. **死文件清理**: 未 import 的 CSS/JS 文件必须删除，防止混淆
4. **模型名显示**: ModelSwitcher 从 config.toml + provider_pool.toml 动态加载，禁止硬编码模型名
5. **窗口圆角**: `html`/`body` 必须有 `overflow: hidden; border-radius: 12px`
6. **构建验证**: 每次修改 CSS/组件后，必须 `npx vite build` + `cargo build --release`，检查 dist 产物
7. **Chat action 命名**: 前端 `neocodex_send_message_stream` → adapter 剥离前缀 → `send_message_stream`，chat plugin 必须有此 action
8. **动态模型列表**: ModelSwitcher 必须并行加载 `providerConfig()` + `getModelPoolStatus()`，合并去重

<!-- 路径说明（2026-09-28 追加，**不重写上文**）：上文出现的 `src-tauri` 与
     `apps/neobot-desktop` 均为**当时的历史路径**。`src-tauri` 于 `5c02e738` 归档、
     `apps/neobot-desktop` 于 `d5413335` 删除，桌面 App 统一到独立仓
     `~/Downloads/Neo/neobot`。改写历史记录等于伪造当时的事实，故只加此注记。 -->
