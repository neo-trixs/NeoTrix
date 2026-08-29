# Conversation-as-OS Agent Bot (对话即操作系统)

## Description
Fused evolution skill: crystallizes the 2026 grok-bot / Agent-Harness ecosystem into NeoTrix's
"对话即操作系统" desktop App Bot pattern. Absorbs `deepseek-ai/deepseek-harness`, `b-nnett/grok-bot-0.18-reconstructed`,
`openclaw/openclaw`, `GetBusbar/busbar`, `anthropics/skills` (Agent Skills standard),
`TencentCloud/TencentDB-Agent-Memory` (four-state memory), `diegosouzapw/OmniRoute` (gateway routing),
and maps each to NeoTrix's 7 domains. Use when evolving the App Bot, the skill engine, or the KB asset model.

## Skill Type
architecture

## Tags
- app-bot
- grok-bot
- agent-harness
- conversation-as-os
- permission-gating
- agent-skills-standard
- four-state-memory
- capability-evolution

## Trigger Phrases
- "对话即操作系统" / "conversation as OS" / "App Bot"
- "熔炼 grok-bot / Harness" / "forge agent bot skill"
- "进化全域能力" / "evolve agent capabilities"
- "Agent Skills 标准接入" / "permission gating 对齐 Busbar"

## Fused Pattern (对话即操作系统)

The winning 2026 form factor is a **local-first desktop agent that treats the chat as the OS shell**:
the model plans, delegates to sub-agents, calls tools/skills, and the human gates risky actions
("control what AI can do before it acts"). NeoTrix already owns the front-end half
(`neocodex-frontend` Chat.tsx: AgentActivityBar, AutonomyMeter, permission-mode persist, reasoning layer).
This skill closes the loop by aligning that front-end with the absorbed back-end patterns.

### Source → Mechanism → NeoTrix Mapping (C1-C6 四字段)

| Source | Pattern (机制) | NeoTrix 映射节点 | 判定 | 消费者 (R-P79) |
|--------|----------------|------------------|------|----------------|
| `deepseek-ai/deepseek-harness` (月度#1) | 开源 agent harness，规划+工具+子代理 | NT-CORE 推理编排 / NT-ACT 执行 | 强化 | Chat.tsx 工具进度条+审批 |
| `b-nnett/grok-bot-0.18-reconstructed` | macOS 桌面 grok-bot 重建，BYO key | NT-IO 桌面 App Bot 形态 | 新增→对齐 dsh-desktop | neocodex-frontend |
| `openclaw/openclaw` (年度#1) | Any OS 个人 AI 助手 | NT-IO 跨平台外壳 | 强化 | Tauri 桌面壳 |
| `GetBusbar/busbar` (Featured) | "Control what AI can do before it acts" 动作门控 | NT-SHIELD/CORE 权限门控 | 强化 | AutonomyMeter + 权限持久化 |
| `anthropics/skills` (Agent Skills 标准) | 标准化 SKILL.md 格式 | NT-MIND 技能结晶格式 | 强化 | skill engine 加载契约 |
| `TencentCloud/TencentDB-Agent-Memory` | 团队级记忆中枢：ChatMemory/Skill/LLM-Wiki/Code-Graph 四态 | NT-MEMORY KB 资产模型 | 新增 | KB schema + 跨会话持久 |
| `diegosouzapw/OmniRoute` | 350 providers, quota-aware 自动 fallback, token 压缩 | NT-ACT 网关选择 `total_calls ascending` | 强化 | gateway provider 路由 |
| `oomol-lab/open-connector` (Featured) | AI agents 开源 connector 网关 | NT-ACT MCP 网关 | 强化 | mcp-gateway skill |
| `NousResearch/hermes-agent` | "与你共同成长" 自进化 | NT-MIND SEAL 循环 | 强化 | SEAL pipeline |
| `affaan-m/ECC` | harness 性能优化：instincts/memory/security | NT-MIND 强化维度 | 新增 | SEAL 维度扩展 |

## Capability Evolution Map (全域)

- **NT-CORE**: E8/Seed Graph 输出 self-contained 可验证架构图（吸收 `tt-a1i/archify` 可验证图）；
  AutonomyMeter 对齐 Busbar "action-before-it-acts" 门控语义。
- **NT-MIND**: 采纳 Agent Skills 标准作为技能结晶格式；SEAL 强化维度补 `instincts`/`security`（ECC）。
- **NT-MEMORY**: KB 升级为 ChatMemory / Skill / LLM-Wiki / Code-Graph 四态资产 + 跨会话持久（TencentDB）。
- **NT-WORLD**: `nt_file_ability` 强化多格式解析（firecrawl `anydoc`/`pdf-inspector` + `Scrapling`）。
- **NT-ACT**: MCP 网关（`open-connector`）+ 多智能体并行编排（`stablyai/orca`）+ gateway quota-aware 选择（OmniRoute）。
- **NT-IO**: 直接对齐「对话即操作系统」桌面 App Bot（`dsh-desktop`/`grok-bot` 重建/`openhuman`/`cc-switch`）。
- **NT-SHIELD**: 权限门控（Busbar）+ 安全技能路由（`zhaoxuya520/reverse-skill` 自进化经验库）+ Egress Privacy Guard。

## Build Guidance (本 session 可落地)

1. 桌面 App Bot 形态已具（`neocodex-frontend`）：确认 AutonomyMeter + 权限模式持久化 = Busbar 门控等价物。
2. 技能结晶格式对齐 `anthropics/skills`：新建/现有 `skills/**/SKILL.md` 统一 frontmatter（name/description/triggers/self_evolution_maturity）。
3. KB 四态资产：在 `nt_memory_kb` 增加 `kind ∈ {chat_memory, skill, llm_wiki, code_graph}` 列，复用现有 nodes/edges 表。
4. 网关选择：`nt_core_llm` gateway 已实现 `total_calls ascending` 轮换；OmniRoute 启示补 quota-aware 自动 fallback。

## Verification
- 技能作为能力树节点 bud 后须有消费者（R-P79），否则降级路线图。
- 接线落地后 `cargo check -p neotrix` 0 error + 对应单测通过。
