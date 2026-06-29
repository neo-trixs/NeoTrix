// NeoTrix Design Absorption Framework (DAF) v5
//
// 深度分析: Codex (OpenAI) + Claude Code (Anthropic) 架构对比与吸收
// 第二轮深度吸收 (2026-06-17): 5层定制栈 / Agent Teams / 沙箱规则引擎 / ToolDispatch管道 / 验证循环
// 分析基础: Codex CLI/App/Plugins GA (March 2026), Claude Code v2.1.88 源码泄露 (512K lines TypeScript)
//           Claude Code Dynamic Workflows + Harness (June 2026, Opus 4.8)
// 核心发现: 98.4% 基础设施 / 1.6% AI 决策 — 真正的工程复杂度在 harness
//
// ┌──────────────────────────────────────────────────────────────────┐
// │ 0. 核心哲学: 两个系统各自发现了什么                                 │
// ├──────────────────────────────────────────────────────────────────┤
// │ Codex 范式:  Manager-Worker + Cloud Sandbox + Plugin             │
// │   - Manager 做规划和判断, Worker 在隔离沙箱中执行                 │
// │   - 每个 Worker 有独立文件系统/依赖/工具                          │
// │   - TOML agent 定义文件 + AGENTS.md 分层指令链                    │
// │   - Plugin = skill + MCP 配置捆绑, 可分发/安装/卸载               │
// │   - 路径寻址 (agents/frontend), CSV 批量 spawn                   │
// │                                                                  │
// │ Claude Code 范式: Dynamic Harness + SubAgent Teams + Memory       │
// │   - 5层定制栈: AGENTS.md → Skills → MCP → Subagents → Plugins     │
// │     (每个层解决不同问题, 依次构建)                                  │
// │   - Agent Teams: 队友间 peer-to-peer 邮箱 + 共享任务列表拉取       │
// │   - 代码验证环: handler 执行后自动验证输出质量, 门控内部 flag      │
// │   - 工具分发管道: approval → sandbox → execution → retry-on-deny  │
// │   - OS 级沙箱: Bubblewrap/Mac Seatbelt/Windows Restricted Token    │
// │   - 44 个未发布特性标记: KAIROS(守护进程)/ULTRAPLAN(远程规划)      │
// │     COORDINATOR_MODE(集群)/DREAM(内存自我整理)                     │
// │   - Harness 是"模型周围的一切", 动态生成每任务的编排程序          │
// │   - 6 种编排模式: classify-act / fan-out / pipeline /            │
// │     adversarial / research-write / iterative-refinement           │
// │   - 4 层记忆: CLAUDE.md → MEMORY.md → MemoryTool → Transcript    │
// │   - 5 层上下文压缩: snip → microcompact → collapse →             │
// │     auto-compact → blocking (层级渐进, 各牺牲不同维度)            │
// │   - 3 种固定 SubAgent 类型: explorer / plan / worker             │
// │   - Agent Teams: peer-to-peer 邮箱通信, 非父子                      │
// │   - 7 个权限模式 + ML 自动分类器                                  │
// └──────────────────────────────────────────────────────────────────┘
//
// ┌──────────────────────────────────────────────────────────────────┐
// │ 1. 本质分析: 共 17 个可吸收特性按六个维度评分                     │
// ├──────────────────────────────────────────────────────────────────┤
// │ 评分: E=表征效率, R=推理深度, S=自我认知, W=世界模型,           │
// │        M=记忆组织, P=感知宽度, A=自主性, G=优雅降级               │
// ├──────────────────────────────────────────────────────────────────┤
// │ 特性                                    │ 来源      │ 评分        │
// ├──────────────────────────────────────────────────────────────────┤
// │ ① Dynamic Workflow Harness               │ Claude    │ R++++ S+++  │
// │    - 按任务动态生成编排脚本 (JS)           │           │ A+++ P++   │
// │    - 修复 agentic laziness/drift/bias     │           │            │
// │    → NeoTrix: E8 态推理→动态调度指令       │           │            │
// │                                                                  │
// │ ② 6 种编排模式                            │ Claude    │ R++++      │
// │    - classify-act / fan-out / pipeline     │           │ S+++       │
// │    - adversarial / research→write / iter   │           │            │
// │    → NeoTrix: TaskDecomposition 扩展         │           │            │
// │                                                                  │
// │ ③ 4 层记忆架构                             │ Claude    │ S++++ M+++ │
// │    - CLAUDE.md (显式, 每次加载)              │           │ W+++       │
// │    - MEMORY.md (隐式, 自动发现/写入)         │           │            │
// │    - MemoryTool (API 层, 长期 agent)        │           │            │
// │    - 会话完整转录 (append-only JSONL)       │           │            │
// │    → 对比 NeoTrix: 有 VSA 语义记忆但无层级  │           │            │
// │                                                                  │
// │ ④ 5 层上下文压缩 (Compaction Pipeline)      │ Claude    │ R+++ E+++ │
// │    - snip: 丢弃旧结果 (免费, 高损失)         │           │ G+++      │
// │    - microcompact: 清除单结果 (保留缓存域)   │           │            │
// │    - collapse: 压缩上下文 (保留原始)         │           │            │
// │    - auto-compact: AI 摘要 (高成本, 低损失)  │           │            │
// │    - blocking: 只接受人工 /compact           │           │            │
// │    → 对比 NeoTrix: 单层 ContextCompressor    │           │            │
// │                                                                  │
// │ ⑤ Permission/Approval 系统                  │ Claude    │ S+++ G++  │
// │    - 7 种权限模式 (allow/deny/ask/auto/...)  │           │            │
// │    - ML 自动分类器判断需确认的操作           │           │            │
// │    - deny-first 评估, 拒绝→反馈到 loop       │           │            │
// │    - 子 agent 继承父 session 的运行时覆写    │           │            │
// │    → NeoTrix: 无权限系统                      │           │            │
// │                                                                  │
// │ ⑥ TOML Agent 定义                           │ Codex     │ P+++ A++  │
// │    - standalone TOML: name/description/      │           │            │
// │      model/sandbox_mode/instructions/MCP     │           │            │
// │    - 全局 ~/.codex/agents/ + 项目级          │           │            │
// │    - 3 内建: default / worker / explorer     │           │            │
// │    → NeoTrix: SubAgentConfig 类似但非 TOML   │           │            │
// │                                                                  │
// │ ⑦ AGENTS.md 分层指令链                       │ Codex     │ M+++ S++  │
// │    - 全局(~/.codex/) → 项目根 → 子目录        │           │            │
// │    - AGENTS.override.md 临时覆盖             │           │            │
// │    - 32KB 上限, 从根向下拼接                   │           │            │
// │    → NeoTrix: AGENTS.md 文件已存在但无层级    │           │            │
// │                                                                  │
// │ ⑧ Skill (SKILL.md) 按需加载                  │ Codex+    │ E+++ P++  │
// │    - 渐进式加载: metadata→选择性完整加载      │ Claude    │ G++       │
// │    - 技能目录: SKILL.md + scripts/refs/assets │           │            │
// │    - 显式(/skill) + 隐式(description 匹配)    │           │            │
// │    - 跨平台标准 (Codex/Claude/Cursor/35+)     │           │            │
// │    → NeoTrix: 已有类似但无 metadata 预算      │           │            │
// │                                                                  │
// │ ⑨ Sandbox Manifest (便携式沙箱定义)           │ Codex     │ W+++ G++  │
// │    - 文件系统/Shell/编辑器标准化规约          │           │            │
// │    - 跨 7 个沙箱提供商可移植                   │           │            │
// │    → NeoTrix: SandboxExecutor 存在但无 manifest│           │            │
// │                                                                  │
// │ ⑩ Peer-to-Peer Agent 通信                    │ Claude    │ R+++ A++  │
// │    - Agent Teams: 队友间 mailbox 通信         │           │            │
// │    - 非父子, 共享任务列表                      │           │            │
// │    → NeoTrix: 父子 SubAgent 模式, 无 P2P      │           │            │
// │                                                                  │
// │ ⑪ Adversarial Verification                   │ Claude    │ S++++ R++ │
// │    - 对抗性 agent 试图打破对方结果            │           │            │
// │    - 多 agent 独立验证后收敛                   │           │            │
// │    → NeoTrix: UltraReview 单 pass             │           │            │
// │                                                                  │
// │ ⑫ 工具按需加载 (Lean Default Toolset)         │ Claude    │ E+++      │
// │    - 默认 20 工具, 40+ 可用                    │           │ G++       │
// │    - 只在需要时注册, 保持上下文精简             │           │            │
// │    → NeoTrix: 115+ handlers 全部 Hot 加载      │           │            │
// │                                                                  │
// │ ⑬ Prompt Cache 优化                          │ Claude    │ E++++     │
// │    - 固定前缀 (system+tools+stable) 缓存       │           │            │
// │    - 90%+ 缓存命中率, 大幅降成本               │           │            │
// │    → NeoTrix: 无缓存策略                        │           │            │
// │                                                                  │
// │ ⑭ Harness 错误恢复 (Resumability)             │ Claude    │ G++++     │
// │    - 中断后恢复, fork, rewind                 │           │            │
// │    - Failure Mode 检测: lazy/drift/bias       │           │            │
// │    → NeoTrix: 无恢复机制                        │           │            │
// │                                                                  │
// │ ⑮ 批量并行 (CSV Batch Spawn)                 │ Codex     │ A++ P++   │
// │    - CSV 定义批量子任务批量 spawn             │           │            │
// │    → NeoTrix: 无批量接口                       │           │            │
// │                                                                  │
// │ ⑯ SubAgent 类型分化                           │ Claude    │ R+++ P++  │
// │    - explorer (只读/轻量) / plan / worker     │           │ E++       │
// │    - 不同模型/工具/上下文预算                   │           │            │
// │    → NeoTrix: 单一 SubAgentRuntime              │           │            │
// │                                                                  │
// │ ⑰ 会话持久化为完整转录                        │ Claude    │ M++++ S++ │
// │    - append-only JSONL, 支持回溯/重现           │           │            │
// │    - Session 存储层, 非仅日志                   │           │            │
// │    → NeoTrix: 文件日志轮转但无结构化转录        │           │            │
// │                                                                  │
// │ ⑱ 共享任务列表 (Pull-based)                     │ Claude    │ A+++ P++  │
// │    - Agent Teams 共享任务列表, 代理主动拉取     │           │            │
// │    - 非 Manager 推送, 减少编排开销              │           │            │
// │    → NeoTrix: SharedTaskList ✅                  │           │            │
// │                                                                  │
// │ ⑲ ToolDispatchPipeline (4-phase)                │ Codex     │ G++++ S++ │
// │    - approval → sandbox → execution → retry      │           │            │
// │    - 每阶段可独立失败/降级                       │           │            │
// │    → NeoTrix: ToolDispatchPipeline + ApprovalStage ✅            │           │
// │                                                                  │
// │ ⑳ VerifyLoop (Two-tier quality)                 │ Claude    │ S++++ R++ │
// │    - handler 执行后自动验证输出                  │           │            │
// │    - Basic: 空/错误标记/未知handler              │           │            │
// │    - Deep: pattern匹配/能力对应验证              │           │            │
// │    → NeoTrix: VerifyLoop + BasicVerifier ✅      │           │            │
// │                                                                  │
// │ ㉑ Sandbox Rule Engine (execpolicy DSL)          │ Codex     │ W+++ G++  │
// │    - ~/.codex/rules/*.rules DSL 执行策略          │           │            │
// │    - 禁止命令: shell/python/git 硬编码           │           │            │
// │    → NeoTrix: SandboxExecutor 存在但无规则引擎   │           │            │
// │                                                                  │
// │ ㉒ Plugin 分发系统                               │ Codex     │ P+++ A++  │
// │    - 打包 skills + MCP + app 配置为可分发单元    │           │            │
// │    - 版本化, 可发现, 企业治理                    │           │            │
// │    → NeoTrix: 无任何分发机制                    │           │            │
// └──────────────────────────────────────────────────────────────────┘
//
// ┌──────────────────────────────────────────────────────────────────┐
// │ 2. 缺口分析 & 实施路线图 (优先级排序)                              │
// ├──────────────────────────────────────────────────────────────────┤
// │ Phase A: 基础设施 (已实现 Phase A 四大核心)                       │
// │ ✅ SubAgent (spawn/step/step_to_result)                           │
// │ ✅ LeadAgent (plan/execute/summary)                               │
// │ ✅ Preview (多方案对比推荐)                                        │
// │ ✅ UltraReview + AutoFix (8 维度 + 修复任务生成)                   │
// │ ✅ PersistentGoal (持久化目标管理)                                 │
// │ ✅ DesignFramework (吸收框架)                                      │
// │                                                                  │
// │ Phase B: 编排 & 通信 (已完成)                                      │
// │ ✅ DynamicWorkflow Harness                                        │
// │    - trait Harness + 4 impl: FanOut/Pipeline/Adversarial/Clsfy    │
// │    - HarnessRegistry 自动关键词选择                                │
// │    - LeadAgent::plan() 委托 Harness                                │
// │                                                                  │
// │ ✅ PermissionSystem                                               │
// │    - PermissionGate 4 模式 + AutoClassify 规则表                   │
// │    - 集成到 modules.rs dispatch (handler 执行前检查)               │
// │    - PermissionOverrides (agent 级 + 全局)                         │
// │                                                                  │
// │ ✅ PeerToPeerMailbox                                              │
// │    - PeerMailbox in AgentCommunicationBus                         │
// │    - send/read_all/unread_count                                   │
// │    - register/unregister 自动同步                                  │
// │                                                                  │
// │ ✅ SharedTaskList (Pull-based)                                     │
// │    - agent 主动 claim_next() 拉取任务                              │
// │    - 依赖追踪 + 自动过滤已完成依赖                                │
// │                                                                  │
// │ ✅ VerifyLoop (Two-tier quality)                                  │
// │    - handler 执行后自动验证输出                                   │
// │    - BasicVerifier: 空/Error标记/未知handler                      │
// │    - 集成到 modules.rs dispatch (match→验证→profiler)             │
// │    - pass_rate() 元监控                                           │
// │                                                                  │
// │ ✅ ToolDispatchPipeline (4-phase)                                 │
// │    - ApprovalStage: 权限检查                                       │
// │    - 可扩展后续阶段: Sandbox/Execution/Retry                       │
// │                                                                  │
// │ Phase C: 记忆 & 上下文 (已完成)                                    │
// │ ✅ CompactionPipeline (5 层骨架)                                   │
// │    - Tier 1-5 enum + CompactionReport                              │
// │    - Compactable trait + compress() 自动升级                       │
// │    - CompactionHistory 追踪                                       │
// │    - ✅ Compactable for AgentCommunicationBus                       │
// │                                                                  │
// │ ✅ SessionTranscript (JSONL append-only)                          │
// │    - 7 event types: session_start/end, handler_dispatch,          │
// │      user_input, agent_communication, permission_check,           │
// │      verify_check                                                 │
// │    - flush() → disk, replay() ← disk                              │
// │    - event_counts() summary                                       │
// │    - enabled/disabled toggle                                      │
// │    - Wired into modules.rs: permission_gate + verify_loop +       │
// │      handler dispatch all record to transcript                    │
// │                                                                  │
// │ ✅ AgentMemory (VSA-based, 3-layer equivalent)                     │
// │    - MemoryEntry (explicit + auto_discovered)                     │
// │    - MemoryPattern (distilled from entries)                       │
// │    - MemoryLesson (highest maturity)                              │
// │    - query_by_tag/search/high_confidence/to_markdown               │
// │    - auto-prune at max_entries                                    │
// │    - JSON persistence to disk (with_path)                          │
// │                                                                  │
// │ ✅ DaemonMode (背景守护进程)                                        │
// │    - background tick in handle_consciousness_batch                 │
// │    - file-based inbox for inter-session messaging                  │
// │    - periodic snapshot persistence                                │
// │    - 14 dispatch handlers wired                                   │
// │                                                                  │
// │ ✅ AdversarialVerifier (Verifier trait deep tier)                  │
// │    - 3-pass verification: basic + capability-pattern +            │
// │      inconsistency detection                                      │
// │                                                                  │
// │                                                                  │
// │ Phase D: 进阶能力 (后续)                                            │
// │ 🔲 SubAgent 类型分化                                                │
// │    - ExplorerRuntime (read-only, 轻量上下文)                        │
// │    - PlanRuntime (重推理, 多工具)                                   │
// │    - WorkerRuntime (执行导向, 默认工具)                              │
// │    → priority: medium, 2 session                                  │
// │                                                                  │
// │ 🔲 Harness 恢复机制 (Resumability)                                 │
// │    - Session snapshot: lead_agent + sub_agent 状态定期快照          │
// │    - Resume: 从快照恢复而非从头开始                                  │
// │    - FailureDetector: 检测 lazy/drift/bias 模式                     │
// │    → priority: low, 2-3 session                                   │
// │                                                                  │
// │ 🔲 CSV Batch Spawn                                                │
// │    - CSV 定义: task_id, capability, description, deps, config       │
// │    - LeadAgent::spawn_batch_from_csv(path) -> Vec<AgentId>          │
// │    → priority: low, 1 session                                     │
// └──────────────────────────────────────────────────────────────────┘
//
// ┌────────────────────────────────────────────────────────────────────┐
// │ 3. 关键设计决策 (来自两个架构的教训)                                │
// ├────────────────────────────────────────────────────────────────────┤
// │ 3.1 动态 Harness vs 固定 Harness                                    │
// │     Codex 固定 Manager-Worker, Claude 动态生成编排.                │
// │     NeoTrix 选择: 混合 — Default = FanOut, 但 GWT 注意力可          │
// │     择触发 Adversarial/Pipeline Harness. 通过在 E8 64 态推理核      │
// │     中编码 Harness 类型, 让 SEAL 自进化选择.                        │
// │                                                                      │
// │ 3.2 权限系统的位置                                                   │
// │     Claude Code 在 tool dispatch 前加 permission gate.               │
// │     NeoTrix: 在 modules.rs dispatch rail 之前加 PermissionGate.     │
// │     拒绝→不执行 handler, 消息返回给 E8 推理核.                      │
// │     不引入独立权限子系统—在 HandlerRegistry 中加 permission 元数据.  │
// │                                                                      │
// │ 3.3 上下文压缩的 VSA 纳入                                            │
// │     Claude Code 用 LLM 做 auto-compact (昂贵).                       │
// │     NeoTrix: Tier 4 auto-compact = VSA 向量摘要而非 LLM 摘要.       │
// │     将 AgentBus 消息/工具输出编码为 VSA 向量, 用超立方体距离保持     │
// │     语义. 这比 LLM 便宜 100x.                                       │
// │                                                                      │
// │ 3.4 记忆层与 VSA 统一                                                 │
// │     所有 4 层共享 VSA 向量表征. Layer 1 = 显式注入的 VSA tag,        │
// │     Layer 2 = 自动 VSA 编码, Layer 3 = HyperCube 语义检索,          │
// │     Layer 4 = NTSSEG 段式存储. 无需格式转换.                        │
// │                                                                      │
// │ 3.5 工具按需加载 = HandlerRegistry tier 热迁移                       │
// │     不用在启动时决定 Hot/Warm. HandlerRegistry 有 access_count,      │
// │     10 周期无访问 → Warm, 50 周期无访问 → Cold (懒加载).             │
// │     需要时 promote 回 Hot. 这是现有 tier 系统的超集.                 │
// │                                                                      │
// │ 3.6 验证环的位置                                                     │
// │     Claude Code 内部 gated behind employee-only flag.                │
// │     NeoTrix: VerifyLoop 在 handler dispatch 之后, profiler 之前.     │
// │     默认 enabled=true, 可通过 dispatch 切换. 未来可接 Verifier trait │
// │     的 AdversarialVerifier(对抗验证) 替代 BasicVerifier(规则).       │
// │                                                                      │
// │ 3.7 Pull-based 任务分配 vs Push-based                                │
// │     Codex Push (Manager-Spawn), Claude Pull (Agent-Claim).          │
// │     NeoTrix 两者皆有: LeadAgent::execute_goal() = Push (规划时决定), │
// │     SharedTaskList::claim_next() = Pull (代理就绪时自行领取).        │
// │     选择由 Harness 决定: PipelineHarness=Push, FanOut=可选 Pull.     │
// │                                                                      │
// │ 3.8 工具分发管道 = PermissionGate 的超集                             │
// │     Codex 的 ToolOrchestrator 实现 4-phase pipeline.                 │
// │     NeoTrix 当前 ApprovalStage ≈ PermissionGate.check().             │
// │     后续 SandboxStage/ExecutionStage/RetryStage 可插入而不破坏现有.  │
// └────────────────────────────────────────────────────────────────────┘
//
// ┌──────────────────────────────────────────────────────────────────┐
// │ 4. 验证清单 (Phase B + C 混合)                                     │
// ├──────────────────────────────────────────────────────────────────┤
// │ [x] DynamicWorkflowHarness trait + 4 实现                         │
// │ [x] Harness 自动选择 (在 LeadAgent::plan 中)                      │
// │ [x] PermissionGate 集成到 modules.rs dispatch                    │
// │ [x] Permission 继承 (子 agent 继承父运行时覆写, PermissionOverrides)│
// │ [x] PeerToPeerMailbox in AgentCommunicationBus                   │
// │ [x] SharedTaskList (pull-based, 依赖追踪)                         │
// │ [x] VerifyLoop (handler 执行后验证, BasicVerifier)                │
// │ [x] ToolDispatchPipeline + ApprovalStage                          │
// │ [x] CompactionPipeline 5 层骨架 + Compactable trait              │
// │ [x] 编译: 0 errors in all new files (196 pre-existing, unrelated) │
// │ [x] 降级: 无 Harness→FanOut, 无 Permission→AllowAll               │
// │ [x] Compactable impl for AgentCommunicationBus                    │
// │ [x] SessionTranscript (JSONL, 7 event types, flush/replay)         │
// │ [x] AgentMemory (3-layer: Entry/Pattern/Lesson, VSA-based)        │
// │ [x] DaemonMode (background tick, file inbox, snapshot)            │
// │ [x] AdversarialVerifier (3-pass deep tier verification)           │
// │ [x] Transcript wired into modules.rs: permission/verify/dispatch  │
// │ [x] 14 new dispatch handlers (transcript/memory/daemon)           │
// │ [ ] Sandbox Rule Engine (execpolicy DSL)                          │
// │ [ ] Plugin 分发系统 (skill+MCP 打包为可分发单元)                    │
// └──────────────────────────────────────────────────────────────────┘

// 本文件不实现任何运行时逻辑——它是设计文档 + 思维 check list。
// 每次实现新特性前，对照上面 6 个问题自检。
