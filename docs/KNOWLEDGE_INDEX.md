# NeoTrix 项目知识综合索引

> 自动生成时间：2026-09-18 12:00:00
> 项目版本：v0.25.4 (Universal Model Gateway)

## 1. 架构文档索引

### 核心架构

| 文档 | 路径 | 描述 |
|------|------|------|
| CHANGELOG.md | `/Users/neo/Downloads/neotrix/CHANGELOG.md` | 项目变更日志 (v0.25.0-v0.25.4) |
| CONTEXT.md | `/Users/neo/Downloads/neotrix/CONTEXT.md` | 共享语言定义 (领域术语、架构模式、审计维度) |
| AGENTS.md | `/Users/neo/Downloads/neotrix/AGENTS.md` | AI Agent 指引文档 |

### 设计文档

| 文档 | 路径 | 主题 |
|------|------|------|
| 桌面AI聊天次世代设计 | `docs/2-PLANS/2026-07-01-desktop-ai-chat-nextgen-design.md` | CognitiveStream认知感知流式引擎 |
| nt-mind-evolve 设计 | `docs/2-PLANS/2026-07-01-nt-mind-evolve-design.md` | 演化循环系统 |
| 统一LLM代理网关 | `docs/2-PLANS/2026-06-30-unified-llm-proxy-gateway.md` | GatewayProvider架构 |

### 架构模式 (6层)

```
L6 Meta-Cognition (元认知层)  →  nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层)         →  nt_core + nt_mind
L4 Emotion (情感层)           →  nt_feel (核心情感引擎)
L3 Embodiment (具身层)        →  nt_physical + nt_shield + nt_feel
L2 Perception (感知层)        →  nt_world + nt_sense
L1 Action (行动层)            →  nt_act + nt_io + nt_memory
```

### 7大领域

| 领域 | 名称 | 代号 |
|------|------|------|
| NT-CORE | 基础领域 | E8引导者 |
| NT-MIND | 进化领域 | 进化工匠 |
| NT-MEMORY | 知识领域 | 知识守护者 |
| NT-WORLD | 感知领域 | 虚空探索者 |
| NT-ACT | 行动领域 | 行动执行者 |
| NT-IO | 界面领域 | 界面使徒 |
| NT-SHIELD | 安全领域 | 影卫 |
| NT-PHYSICAL | 具身领域 | 具身骨架 |
| NT-FEEL | 情感领域 | 情感中枢 |

## 2. KB 知识库统计

| 指标 | 数量 |
|------|------|
| **节点数 (nodes)** | 390,264 |
| **边数 (edges)** | 792,308 |
| **KV条目数 (kv_store)** | 9,216 |
| **会话日志数 (session_logs)** | 1,236 |

### 会话日志分类

| 类型 | 示例 |
|------|------|
| CLI会话 | `/help`, `/model list`, `/provider list`, `/e8 status` |
| 审计日志 | `audit_*_sess_*.jsonl` (1000+ 条) |
| 吸收日志 | `audit_1191_batch2_absorption.jsonl` |

### KB 数据库表 (50+)

核心表：`nodes`, `edges`, `kv_store`, `session_logs`, `experience`, `embeddings`, `reasoning_chains`, `procedural_memory`

## 3. 经验索引

### 经验分类统计

| 类别 | 数量 | 占比 |
|------|------|------|
| plan (计划) | 25 | 55.6% |
| architecture (架构) | 4 | 8.9% |
| frontend (前端) | 2 | 4.4% |
| strategy (策略) | 1 | 2.2% |
| efficiency (效率) | 1 | 2.2% |

### 架构经验

| 标题 | 来源 | 优先级 |
|------|------|--------|
| CognitiveStream 三阶段设计 | 桌面AI聊天设计文档 | high |
| 统一Provider评分公式 | 统一LLM网关设计 | high |
| GatewayProvider统一网关架构 | 统一LLM网关设计 | high |
| EGL演化泛化损失检测 | nt-mind-evolve设计 | high |

### 策略经验

| 标题 | 来源 | 优先级 |
|------|------|--------|
| 免费Provider三层体系 | 统一LLM网关设计 | high |
| 五种演化策略选择算法 | nt-mind-evolve设计 | high |
| 三级MutationScope自动选择 | nt-mind-evolve设计 | high |

### 前端经验

| 标题 | 来源 | 优先级 |
|------|------|--------|
| OfficeFloor 真实数据集成 | v0.25.1 | - |
| Tauri 后端 onboarding | v0.25.0 | - |

## 4. 关键洞察汇总 (Top 10)

### 1. 认知感知流式引擎 (CognitiveStream)
- **洞察**: 三阶段节奏控制 (init→streaming→complete) 可减少42%用户挫败感
- **证据**: CHI 2026研究
- **应用**: Desktop AI Chat 次世代设计

### 2. 统一LLM网关架构
- **洞察**: 单一GatewayProvider包装6层中间件，零改动现有10+调用点
- **问题**: 5个断开路由器、4个桩流式provider、零rate limiting
- **解决**: TokenBucket→CircuitBreaker→ProviderPool→FallbackChain

### 3. Provider评分公式
- **洞察**: Score = (success_rate²/p95_latency) × (1/cost_per_token)^β × health_penalty^γ
- **参数**: β=0.3(成本敏感度)、γ=2.0(健康惩罚)
- **选择算法**: Thompson Sampling

### 4. 免费Provider三层架构
- **Tier1**: 无密钥 (Pollinations/OVHcloud/Kilo)
- **Tier2**: 有密钥免费 (Groq/Gemini/OpenRouter)
- **Tier3**: 试用/社区 (DeepSeek/Mistral/Cohere)

### 5. 演化循环系统 (nt-mind-evolve)
- **洞察**: 6阶段15天实施: BenchmarkSuite→EvolutionLoop→EglTracker→TraitStore→PipelineAdapter→E2E
- **EGL检测**: 滚动窗口退化触发回滚 (regression_threshold=-0.05)
- **策略选择**: AdaptiveEvolve→GuidedSynthesis→SkillForge→Recombination→ParameterSearch

### 6. SelfModel 三重定义合并
- **洞察**: 静态身份 + 动态性能 + 价值函数三合一
- **位置**: `l6_meta/self_model_unified.rs`

### 7. L6 元认知归位
- **洞察**: 9个文件从L5移至L6，保持向后兼容
- **意义**: 架构层级正确性

### 8. L1→L5 反向依赖消除
- **洞察**: `ReasoningEngineProvider` trait 抽象L5具体类型
- **意义**: 依赖方向正确

### 9. Egress Privacy Guard 信任分层
- **Trusted**: 本地/Ollama — passthrough after secret scrub
- **Contracted**: 付费云 — 永远redact
- **Untrusted**: 免费/代理 — fail-closed block

### 10. Agent 一等公民
- **洞察**: AgentPersona 作为身份、AutonomyLevel 四级自治、SpendGate 成本阈值审批
- **应用**: Cumora + Munder Difflin 吸收

## 5. 项目演进里程碑

| 版本 | 日期 | 主题 |
|------|------|------|
| v0.25.4 | 2026-09-17 | Universal Model Gateway (5 Phase并行) |
| v0.25.3 | 2026-09-17 | 架构巡检 + 关键修复 (SelfModel/L6/反向依赖) |
| v0.25.2 | 2026-09-17 | 冗余清理 + 架构重构 (now_secs去重/Stub合并) |
| v0.25.1 | 2026-09-17 | Hive 真实数据集成 + BYOA 进程池 |
| v0.25.0 | 2026-09-17 | Cumora + Munder Difflin 吸收 (Agent一等公民) |

## 6. 核心设计原则

| 原则 | 代码 | 含义 |
|------|------|------|
| 零unsafe | R-P1 | `#![forbid(unsafe_code)]` |
| 构建缓存不可信 | R-P9 | 结构变更后强制 clean build |
| 清理前归档 | R-P81 | SafeDeleter 默认启用 archive_before_delete |
| 指针守恒 | HARD RULE | AGENTS.md 禁止内联经验表 |
| 外部文件惰性加载 | LAZY LOAD | 相关任务时按需读取 |

---

**索引统计**: 文档覆盖 1,222+ 页面, 390K+ 节点, 792K+ 边, 9K+ KV条目
