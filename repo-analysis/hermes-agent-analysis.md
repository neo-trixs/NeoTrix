# Hermes Agent (Nous Research) — 分析报告

> 分析日期: 2026-05-29
> 项目状态: v0.15.1 (2026.5.29), 172k stars, 28.8k forks, MIT

---

## 1. 正确 GitHub URL

**https://github.com/NousResearch/hermes-agent**

---

## 2. 核心架构（10 要点）

1. **AIAgent (`run_agent.py` ~4400 行)** — 同步编排引擎，单一类服务所有入口（CLI/Gateway/ACP/Batch/Cron），支持三种 API 模式（chat_completions / codex_responses / anthropic_messages）
2. **Prompt Builder (`agent/prompt_builder.py`)** — 从 SOUL.md + MEMORY.md + USER.md + skills + context files + tool-use guidance 组装 system prompt，session 内不可变以保持 prefix cache
3. **Tool Registry (`tools/registry.py`)** — 70+ tools, 28 toolsets，每个 tool 文件在 import 时自注册，支持顺序/并发执行 + 危险命令检测
4. **Session Storage (`hermes_state.py`)** — SQLite + FTS5 全文搜索，session lineage tracking（压缩产生父子 session），atomic writes
5. **Messaging Gateway (`gateway/`)** — 20 个平台适配器（Telegram/Discord/Slack/WhatsApp/Signal/Feishu/WeCom 等），统一路由 + 授权 + 钩子系统
6. **Skills System (`~/.hermes/skills/`)** — 渐进式加载（level 0-2），SKILL.md YAML frontmatter 格式，条件激活（fallback_for_toolsets），8+ hub 源集成
7. **Agent-Managed Skills (`skill_manage` tool)** — 复杂任务完成后自动创建/更新 skill，作为 procedural memory；支持 patch/edit/create/delete
8. **Persistent Memory** — MEMORY.md (2200 chars) + USER.md (1375 chars)，agent 通过 `memory` tool 自管理；双文件分隔 agent notes 和 user profile
9. **Context Compression (`agent/context_compressor.py`)** — 超 50% context 阈值时触发，压缩中间轮次为摘要，保护最近 20 条消息不压缩
10. **6 种 Terminal Backends** — local/Docker/SSH/Singularity/Modal/Daytona，一致的工具执行接口

---

## 3. 关键创新与差异化

### 核心差异化：Self-Improvement Loop

Hermes 的核心 claim：**"唯一具有内置学习循环的 agent"**。

其自改进机制是 **LLM-native** 而非数学建模：

| 特性 | Hermes Agent 实现 | 与 NeoTrix 对比 |
|------|-------------------|-----------------|
| **Skill 自动创建** | 5+ tool calls 的复杂任务完成后 → LLM 判断是否值得保存为 SKILL.md → 写入 `~/.hermes/skills/` | 类似 skill_crystal.rs 但更偏向 procedural (步骤列表) 而非 structural (向量) |
| **Skill 自改进** | 下次使用同一 skill 时，LLM 可在运行时优化步骤；`Curator` 后台静默维护 skill 库 | NeoTrix skill 是静态晶体（Vector→Code），Hermes 的 skill 是动态文档（Code→Refinement） |
| **Memory 自管理** | Agent 通过 `memory` tool 自主决定何时存储/合并/替换，LLM 判断重要性 | ReasoningBank 是结构化的向量记忆（Cosine similarity 检索），Hermes 是纯文本 prompt 注入 |
| **Honcho 用户建模** | 可选的 dialectic framework 构建用户行为模型 | 无对应功能 |
| **Session Search** | FTS5 检索 + LLM summarization 跨 session recall | ReasoningBank 的 recall_similar() 通过向量相似度检索 |
| **Curator 后台** | v2026.4.30 新增 — 后台自动维护 skill 库，检查过期/冲突/优化 | 类似 MetaCognitiveSelfCheck 但专注 skill 而非系统全局 |

### 其他差异化

- **Multi-platform Messaging** — 20 个平台适配器，Telegram/Discord/Slack/WhatsApp/Signal/Feishu/WeCom 等，NeoTrix 无此维度
- **300+ Model Support** — Nous Portal / OpenRouter / HuggingFace / OpenAI / Anthropic 等 18+ providers 统一 CLI 切换
- **Batch Trajectory Generation** — 生成 ShareGPT 格式轨迹用于训练下一代数模型，实验室级功能
- **Plugin System** — 3 种发现源（user/project/pip entry points），memory provider / context engine 两种单选插件类型
- **ACP Agent Protocol** — VS Code / Zed / JetBrains IDE 集成

---

## 4. 与 NeoTrix 的 Gap / Overlap 分析

### Overlap（功能重叠）

| 领域 | Hermes | NeoTrix | 分析 |
|------|--------|---------|------|
| 自迭代 | LLM 驱动的 skill 创建/改进 | SEAL loop + CapabilityVector 数学更新 | **基础思想相同但实现方向相反**：Hermes 是 LLM→文档（实践知识），NeoTrix 是数学→向量（能力编码） |
| 记忆系统 | MEMORY.md + USER.md + FTS5 session search | ReasoningBank + SHA256 缓存 + Cosine recall | Hermes 轻量实用，NeoTrix 结构化但更复杂 |
| Skills | `skill_manage` tool 自动创建 SKILL.md | `skill_crystal.rs` + 向量存储 | Hermes 的 skill 是文档，NeoTrix 的 skill**应该**是结构化的... 但 skill_crystal.rs 缺少清晰架构 |
| Multi-agent | `delegate_tool` 生成子 agent，独立会话 | `agent/team.rs` + Coordinator + 5 种 ProcessType | **NeoTrix 架构更完整**（RoutingTable/CriticNode），Hermes 更实用但简单 |
| MCP | `mcp_tool.py` 客户端 + `mcp_serve.py` 服务器模式 | `mcp_bridge.rs` + `mcp_tools.rs` | 功能对等，实现语言不同 |

### Gap（NeoTrix 缺、Hermes 有）

| 特性 | 优先级 | 建议 |
|------|--------|------|
| **Multi-platform Messaging Gateway** (Telegram/Discord/Slack/WhatsApp) | P1 | Knowledge only — NeoTrix 定位不是消费者消息 agent |
| **300+ Model Provider Resolution** | P1 | **Absorb** — NeoTrix 当前硬编码 LLM provider，需要统一 provider registry + fallback chain |
| **Batch Trajectory Generation** (ShareGPT 格式训练数据) | P2 | Knowledge only — 如果 NeoTrix 要训练模型，值得吸收 |
| **Skill Bundles** (多 skill 组合为单斜杠命令) | P2 | **Absorb** — 直接映射到 skill bundles 概念，与 skill_crystal.rs 互补 |
| **Plugin System** (user/project/pip entry points) | P2 | **Absorb** — 但需评估 Rust 生态的 plugin 实现成本 |
| **Honcho User Modeling** | P3 | Knowledge only — 通过 capabilitiy vector 的 user_affinity 已有类似功能 |
| **ACP IDE Integration** | P3 | Knowledge only — 非当前目标 |
| **20 Messaging Platform Adapters** | P4 | Skip — 不在 NeoTrix 路线图内 |

### Gap reverse（Hermes 缺、NeoTrix 有）

| 特性 | 重要性 | 说明 |
|------|--------|------|
| **CapabilityVector 数学建模** | 高 | Hermes 无等价物，纯 LLM 驱动 |
| **SEAL Algebra / Impact Matrix** | 高 | Hermes 无结构化分析工具 |
| **HyperCube VSA** | 中 | Hermes 无向量符号架构 |
| **GWT 注意力路由** | 中 | Hermes 无 consciousness 层 |
| **GoalLoop + CircuitBreaker** | 中 | Hermes 有 cron 但无 structured goal tracking |
| **AgentTeam Coordinator** | 中 | Hermes 的 delegate 是简单 spawn，无 routing table |
| **MetaCognitive Self-Check** | 中 | Hermes 的 Curator 仅 check skills，非系统全局 |
| **Self-Evolver (外部 GitHub URL 吸收)** | 中 | Hermes 通过 skills hub 安装外部 skill，非架构级吸收 |

---

## 5. 推荐：Absorb (Provider Registry) + Knowledge Only (Skills Hub) + Skip (Gateway)

### 优先级吸收建议

| 吸收项 | 等级 | 理由 | 估算工作量 |
|--------|------|------|-----------|
| **Provider Registry + Fallback Chain** | **T1** | NeoTrix 目前硬编码 provider，需要统一接口；这是架构级缺失 | 2-3 days |
| **Skill Bundles 概念** | **T2** | 直接对应 skill_crystal.rs 的"复合 skill"模式，低风险高收益 | 0.5 day |
| **技能自动创建流程** | **T2** | 现有 SEAL loop 的 `generate_self_edit()` 已接近，需对齐到 Hermes 的 `skill_manage` 模式 | 1 day |

### 知识注入建议

- **Provider Registry**: `KnowledgeSource` 枚举添加 `NousHermes` variant，`capability_vector()` 返回 `model_diversity: 0.9, provider_fallback: 0.9`
- **Skill Management**: 注入 3 条种子知识到 ReasoningBank（provider 切换模式、fallback 链配置、skill 自动创建触发条件）

### 跳过

- **Messaging Gateway** (20 平台适配器) — 非 NeoTrix 定位
- **ACP IDE Integration** — 非当前目标
- **Python 生态系统** (Hermes 是 Python，NeoTrix 是 Rust) — 架构参考不改语言

---

## 6. 总结

Hermes Agent 和 NeoTrix 有**共同的愿景但不同的实现哲学**：

| 维度 | Hermes (Nous Research) | NeoTrix |
|------|----------------------|---------|
| 语言 | Python | Rust |
| 自改进机制 | LLM prompt 驱动 → 文档化 skill | 数学向量 + SEAL loop → CapabilityVector |
| 记忆 | 纯文本 MEMORY.md + FTS5 | ReasoningBank + Cosine similarity |
| 知识表示 | 自然语言 SKILL.md | 22 维 CapabilityVector + KnowledgeSource |
| 多 agent | 简单的 delegate spawn | Coordinator + ProcessType + RoutingTable |
| 平台覆盖 | 20 个 messaging 平台 | 无 |
| 模型支持 | 300+ models, 18+ providers | 硬编码单 provider |
| 用户规模 | 172k stars, 大众市场 | 小规模，深度技术 |

**核心洞察**：Hermes 是**面向最终用户的 agent**（易用、多平台、实用），NeoTrix 是**面向 agent 架构研究**（结构化、可证明、严肃）。两者互补 > 竞争。
