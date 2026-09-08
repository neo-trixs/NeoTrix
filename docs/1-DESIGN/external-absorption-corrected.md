# 外部项目吸收 — 修正版 (Core Purpose + Problem Statement)

> **修正原因**: 前版吸收过度关注技术机制 (how)，忽略了核心目的 (why) 和要解决的具体问题 (what)。
> 本版按 "项目→核心目的→要解决的问题→具体任务→对 NeoTrix 的映射" 重新吸收。

---

## 1. Claude Code (Anthropic)

### 核心目的
让开发者在终端中用自然语言完成复杂开发任务 — 理解整个代码库、执行多文件编辑、管理 git 工作流。

### 要解决的问题
1. **上下文切换成本高**: 开发者在编辑器/终端/git/CI 之间频繁切换，打断心流
2. **大型代码库理解慢**: 不熟悉的项目需要数天才能上手
3. **重复性任务消耗精力**: 写测试、修 lint、解决合并冲突、更新依赖
4. **代码审查瓶颈**: 人工审查速度跟不上开发速度

### 具体解决的任务
- `/spec` → 需求规格化 (从模糊需求到结构化规格)
- `/plan` → 实现规划 (从规格到可执行计划)
- `/build` → 代码实现 (从计划到代码)
- `/test` → 测试编写 (从代码到测试)
- `/review` → 代码审查 (从 PR 到审查意见)
- `/ship` → 部署发布 (从审查到上线)

### 关键机制
- **Skills as structured workflows**: 每个 skill 是一个完整的工程流程 (spec→plan→build→test→review→ship)，不是 prompt 模板
- **Anti-rationalization tables**: 防止 agent 跳过质量步骤的硬约束
- **Hooks**: 确定性回调 (shell 命令)，在 LLM 外部执行，模型无法绕过
- **CLAUDE.md**: 项目级持久化记忆，跨会话保持

### NeoTrix 映射
| Claude Code 机制 | NeoTrix 对应 | 差距 |
|-----------------|-------------|------|
| Skills as workflows | SEAL pipeline stages | ✅ 已有，但缺少 anti-rationalization |
| Hooks (deterministic) | agent.rs hooks (QualityGate/TodoWarning) | ⚠️ 已有基础，需扩展为 27 events |
| CLAUDE.md memory | AGENTS.md + KB experience | ✅ 已有 |
| Parallel sub-agents | nt_act_orchestrator | ⚠️ 需要更强的并行调度 |
| Codebase comprehension | nt_world_code_search + nt_memory_kb | ✅ 已有 |

---

## 2. OpenAI Agents SDK

### 核心目的
用最少的原语 (Agent/Guardrail/Handoff) 构建多 agent 工作流，足够简单学得快，足够强大生产用。

### 要解决的问题
1. **框架过度抽象**: LangChain/AutoGen 抽象太多，学习曲线陡峭
2. **厂商锁定**: 绑定特定 LLM 提供商
3. **可观测性差**: 多 agent 系统调试困难
4. **持久化缺失**: agent 状态无法跨会话保持

### 具体解决的任务
- **Agent 定义**: instructions + tools + guardrails + handoffs
- **Handoff**: agent 委派给其他 agent (如: 编码 agent → 审查 agent)
- **Guardrail**: 输入/输出验证，与 agent 执行并行运行 (非串行)
- **Session**: 持久化记忆 (SQLite/Redis/MongoDB)
- **Tracing**: 内置可视化、调试、评估

### 关键机制
- **3 原语极简主义**: Agent + Guardrail + Handoff = 一切
- **Guardrails 并行**: 验证与执行同时进行，不阻塞
- **Provider-agnostic**: 通过适配器支持 100+ LLM
- **Tracing as first-class**: 每次执行自动追踪

### NeoTrix 映射
| OpenAI SDK 机制 | NeoTrix 对应 | 差距 |
|----------------|-------------|------|
| 3 原语极简 | Tool trait + ToolRegistry + Orchestrator | ⚠️ 原语更多，需简化 |
| Guardrails 并行 | nt_shield (串行) | ❌ 需改为并行 |
| Provider-agnostic | InferenceRouter (5 providers) | ⚠️ 需扩展到更多 |
| Session persistence | nt_memory_agent_session | ✅ 已有 |
| Tracing | opentelemetry (optional) | ⚠️ 需要默认启用 |

---

## 3. Easel (ZJU 419★)

### 核心目的
AI 社交媒体内容工作台 — 从热点发现到内容创作到多平台发布的端到端流程。

### 要解决的问题
1. **平台碎片化**: 6+ 平台 (小红书/抖音/快手/B站/知乎/微信视频) 格式、调性、要求各不同
2. **创作流程断裂**: 热点研究在一个工具，文案在另一个，视觉在第三个，发布手动跨平台
3. **无反馈闭环**: 发布后没有数据回流，不知道什么有效
4. **身份不连续**: 每次创作都从零开始，没有品牌调性积累

### 具体解决的任务
- **热点聚合**: 自动发现 6 平台热门话题
- **话题规划**: 从热点到内容日历
- **内容创作**: 112 个可执行技能 (不是 prompt，是真正运行的脚本)
- **多平台发布**: 浏览器自动化登录 + 发布到 6 个中国平台
- **数据归因**: 播放/互动数据回流，进化账号画像

### 关键机制
- **Skills with real execution**: 每个 skill 有 `scripts/` 目录，真正生成图片/音频/视频
- **6-dimension profile**: 定位/风格/受众/平台/偏好/记忆，跨会话进化
- **Project-based archiving**: 所有输出按项目保存在 `outputs/`
- **Attribution feedback loop**: 性能数据 → 画像进化

### NeoTrix 映射
| Easel 机制 | NeoTrix 对应 | 差距 |
|-----------|-------------|------|
| Skills with scripts | nt_act tools (HttpRequest/FileOp/Shell/Code/Knowledge) | ⚠️ 需要更多可执行技能 |
| 6-dimension profile | SelfModel + nt_memory_user | ⚠️ 需要更结构化的画像 |
| Project archiving | outputs/ 目录 | ✅ 已有 |
| Attribution loop | experience-tree absorption | ⚠️ 缺少性能数据回流 |
| Multi-platform publish | nt_io_web + nt_act_sandbox | ⚠️ 需要平台适配器 |

---

## 4. AutoGen (Microsoft)

### 核心目的
多 agent 对话框架 — 多个专业化 agent 通过消息传递协作完成任务。

### 要解决的问题
1. **单 agent 能力天花板**: 复杂任务需要多样专业知识
2. **动态任务分配难**: 哪个 agent 做哪件事需要实时决策
3. **调试多 agent 对话极难**: 消息流不可追踪
4. **缺乏可复用组件**: 每次都要从头构建 agent

### 具体解决的任务
- **Group Chat**: 多 agent 辩论/投票/协作
- **Asynchronous messaging**: 非阻塞 agent 通信
- **Layered API**: Core (消息传递) → AgentChat (快速原型) → Extensions (第三方集成)
- **Magentic-One**: SOTA multi-agent team (web browsing + code execution + file handling)
- **AutoGen Studio**: 无代码 GUI 构建多 agent 应用

### 关键机制
- **Conversation as orchestration**: agent 之间用对话协调，自然直觉
- **Distributed runtime**: agent 可在不同机器/进程/语言
- **Extensions ecosystem**: 社区构建的组件 (LLM 客户端/执行器/agent)

### NeoTrix 映射
| AutoGen 机制 | NeoTrix 对应 | 差距 |
|-------------|-------------|------|
| Conversation as orchestration | EventBus + agent message passing | ⚠️ 需要更自然的对话模式 |
| Layered API | L1-L6 层架构 | ✅ 已有 |
| Distributed runtime | nt_nexus consensus + CRDT | ⚠️ 需要更简单的分布式 |
| Extensions | nt_file_ability 适配器注册表 | ⚠️ 需要社区扩展机制 |

---

## 5. CrewAI

### 核心目的
角色扮演 AI agent 团队框架 — 平衡 agent 自主协作与确定性工作流控制。

### 要解决的问题
1. **自主性 vs 可控性矛盾**: 太自主不可预测，太僵硬无智能
2. **角色专业化不足**: 通用 agent 无法处理领域特定任务
3. **缺乏反馈学习**: agent 执行后不从结果中学习
4. **框架依赖**: 绑定 LangChain 或其他框架

### 具体解决的任务
- **Crews**: 角色化 agent 团队 (role/goal/backstory/tools)
- **Flows**: 事件驱动工作流 (@start/@listen/@router)
- **Crews + Flows**: 自主 agent 工作包裹在确定性编排中
- **Training**: agent 从反馈中学习
- **Checkpointing**: 保存/恢复 agent 执行状态

### 关键机制
- **Crews for autonomy, Flows for control**: 双架构解决自主 vs 可控矛盾
- **Role-based agents**: 每个 agent 有角色/目标/背景故事/工具集
- **Dynamic delegation**: agent 决定向谁求助
- **Standalone**: 零 LangChain 依赖

### NeoTrix 映射
| CrewAI 机制 | NeoTrix 对应 | 差距 |
|------------|-------------|------|
| Crews (autonomy) | GWT salience routing | ✅ 已有自主路由 |
| Flows (control) | SEAL pipeline stages | ✅ 已有确定性管线 |
| Crews + Flows | GWT + SEAL 组合 | ✅ 架构对齐 |
| Role-based agents | Domain modules (nt_core/nt_mind/etc) | ⚠️ 需要更显式的角色定义 |
| Training | experience-tree + KB absorption | ⚠️ 缺少在线学习 |
| Standalone | ✅ Rust 零依赖 | ✅ 已有 |

---

## 综合: NeoTrix 真实能力 vs 外部项目

### NeoTrix 实际做了什么 (基于代码)

| 能力 | 代码位置 | 成熟度 |
|------|---------|--------|
| **18-stage 推理内核** | ReasoningKernel (nt_io_standalone) | ✅ 工作中 |
| **统一工具网关** | HarnessTool (11 tools) | ✅ 工作中 |
| **知识库** | nt_memory_kb (79 模块) | ✅ 工作中 |
| **文件能力** | nt_file_ability (Office/PDF/图像) | ✅ 工作中 |
| **LLM 路由** | InferenceRouter (5 providers) | ✅ 工作中 |
| **Agent Hooks** | QualityGate/TodoWarning/Session | ✅ 基础 |
| **沙箱执行** | SandboxConfig/LocalSandbox | ✅ 工作中 |
| **反检测抓取** | BrowserScraper/RequestScraper | ✅ 工作中 |
| **安全审计** | SecurityAuditor (D1-D51) | ✅ 工作中 |
| **意识核心** | ConsciousnessTree + Phi | ⚠️ 原型 |
| **自我进化** | SEAL pipeline + experience-tree | ⚠️ 基础 |
| **多 agent 协同** | nt_act_orchestrator | ⚠️ 基础 |
| **视频生产管线** | nt_physical video modules | ⚠️ 新增 |
| **贸易编排** | TradeOrchestrator | ⚠️ 新增 |

### NeoTrix 缺什么 (基于外部项目对比)

| 缺失能力 | 来源项目 | 优先级 |
|---------|---------|--------|
| **Skills as executable scripts** | Easel (112 skills) | P0 |
| **Anti-rationalization gates** | Claude Code | P0 |
| **Guardrails 并行执行** | OpenAI SDK | P1 |
| **27 lifecycle hooks** | Claude Code | P1 |
| **Profile-driven adaptation** | Easel 6-dimension | P1 |
| **Online learning from feedback** | CrewAI Training | P2 |
| **Community extension mechanism** | AutoGen Extensions | P2 |
| **No-code agent builder** | AutoGen Studio | P3 |

---

## 吸收教训 (进化经验)

### 错误模式
1. **过度关注机制**: "KVMem uses paged KV virtualization" → 应该问 "KVMem 解决什么问题？"
2. **忽略核心目的**: 提取了 40 个设计模式，但没问 "这些项目为什么存在？"
3. **扁平化复杂系统**: 把 Easel 的 112 skills 简化为 "skill composition pattern"
4. **脱离实际代码**: 设计了 72 个决策，但没对照 NeoTrix 的真实代码

### 正确吸收流程
1. **先问 why**: 这个项目为什么存在？解决什么具体问题？
2. **再问 what**: 它具体做了哪些任务？
3. **然后 how**: 它用什么机制解决？
4. **最后 map**: NeoTrix 缺什么？差距在哪？

### 对架构设计的影响
- §0.15 程序族层级需要加入 **executable skills** 作为 L1 的核心能力
- 需要新增 **anti-rationalization gates** 作为 SEAL pipeline 的质量门
- **Guardrails 并行** 需要作为 L3 安全层的设计决策
- **Profile-driven adaptation** 需要扩展 SelfModel 为 6-dimension profile
