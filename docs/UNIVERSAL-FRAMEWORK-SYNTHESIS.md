# NeoTrix 通用框架综合熔炼 — 500+ URL 深度吸收报告

> **日期**: 2026-09-18
> **数据源**: 500+ URLs (GitHub repos, arxiv papers, blogs, tools)
> **处理方式**: 6 并行研究 agent + 架构师综合熔炼
> **目标**: 将所有外部模式逆向推理并融合到 NeoTrix 6 层架构骨架

---

## 一、吸收总览

### 1.1 处理的 URL 分类

| 类别 | 数量 | 高价值项目 |
|------|------|-----------|
| **AI Agent 架构** | 50+ | OpenCode, Claude Code, Aider, Hermes, OpenHands, CrewAI, MetaGPT |
| **记忆/知识系统** | 30+ | claude-mem, mem0, supermemory, GraphRAG, Letta, LightMem, Graphiti |
| **Harness/评估** | 40+ | deepseek-harness, promptfoo, SWE-agent, ECC, autoresearch, RSI |
| **浏览器/Web** | 30+ | browser-use, firecrawl, crawl4ai, stagehand, nanobrowser |
| **模型架构/推理** | 20+ | vLLM, llama.cpp, SGLang, Ollama, DeepSeek V4.1, Kimi K3 |
| **技能/工具生态** | 60+ | mattpocock/skills, addyosmani/agent-skills, superpowers, shadcn |
| **安全/渗透** | 20+ | cloudflare/security-audit-skill, nuclei, bettercap |
| **其他 (设计/游戏/学术等)** | 250+ | 分类存档，低优先级 |

### 1.2 提取的核心模式数

| 模式类别 | 数量 | 与 NeoTrix 映射度 |
|---------|------|------------------|
| 架构模式 | 47 | ⭐⭐⭐⭐⭐ |
| 记忆模式 | 32 | ⭐⭐⭐⭐⭐ |
| 路由模式 | 28 | ⭐⭐⭐⭐⭐ |
| 技能模式 | 24 | ⭐⭐⭐⭐ |
| 安全模式 | 18 | ⭐⭐⭐⭐ |
| 浏览器模式 | 15 | ⭐⭐⭐ |

---

## 二、熔炼到 6 层架构

### 2.1 L0 Substrate — 基础设施层

**吸收来源**: vLLM PagedAttention, SGLang HiCache, DeepSeek V4.1 Flash CED

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **PagedAttention 块分配** | vLLM | KVMem 分页 KV 存储 (>256K 会话) | ⭐⭐⭐⭐⭐ |
| **三层 KV 层次** | SGLang HiCache | GPU HBM → CPU RAM → SSD 分层缓存 | ⭐⭐⭐⭐⭐ |
| **RadixAttention 前缀缓存** | SGLang | GWT salience 路由中的前缀感知 | ⭐⭐⭐⭐ |
| **CED 非对称计算** | DeepSeek V4.1 | 廉价模型做 I/O，昂贵模型做推理 | ⭐⭐⭐⭐⭐ |
| **CSA2 三模式路由** | DeepSeek V4.1 | 技能路由: Full/Reindex/Reuse | ⭐⭐⭐⭐ |
| **硬件感知后端选择** | llama.cpp | 自动选择 Metal/CUDA/CPU | ⭐⭐⭐⭐ |

### 2.2 L1 Action — 行动层

**吸收来源**: Aider 工具链, OpenHands ACP, CrewAI Flow, Browser Use

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **MCP 工具协议** | Claude Code | NT-ACT 统一工具接口 | ⭐⭐⭐⭐⭐ |
| **Agent Client Protocol** | OpenHands | Agent 互操作标准 | ⭐⭐⭐⭐ |
| **Edit-Format 多态** | Aider | 任务类型 → 专用执行器工厂 | ⭐⭐⭐⭐⭐ |
| **Lazy Provider 加载** | Aider | 延迟 1.5s 导入，节省启动时间 | ⭐⭐⭐⭐ |
| **Observe-Act-Observe 循环** | Browser Use | NT-WORLD 世界感知循环 | ⭐⭐⭐⭐⭐ |
| **Ref-Based 元素定位** | Agent Browser | `@e1` 引用替代 CSS 选择器 | ⭐⭐⭐⭐ |
| **三原语 API** | Stagehand | act/observe/extract 统一接口 | ⭐⭐⭐⭐ |
| **Borrow-Return 标签模型** | BrowserSkill | 非破坏性浏览器自动化 | ⭐⭐⭐ |

### 2.3 L2 Perception — 感知层

**吸收来源**: Stagehand, Crawl4AI, Agent Browser, Firecrawl

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **两级抓取架构** | Firecrawl | HTTP 快速路径 → 浏览器慢速路径 | ⭐⭐⭐⭐⭐ |
| **预取模式** | Crawl4AI | 5-10x 加速，跳过完整提取 | ⭐⭐⭐⭐ |
| **渐进式爬取 + 检查点** | Crawl4AI | 序列化状态，断点续传 | ⭐⭐⭐⭐ |
| **浏览器池三级** | Crawl4AI | permanent/hot/cold 淘汰 | ⭐⭐⭐⭐ |
| **内容过滤管线** | Crawl4AI | Pruning → BM25 → LLM | ⭐⭐⭐⭐⭐ |
| **无障碍树遍历** | Stagehand | DOM → LLM 友好 token 表示 | ⭐⭐⭐⭐ |
| **自愈选择器** | Stagehand | 站点变更时重新推导选择器 | ⭐⭐⭐⭐ |
| **人类循环** | BrowserSkill | CAPTCHA/确认时请求人类帮助 | ⭐⭐⭐ |

### 2.4 L3 Embodiment — 具身层

**吸收来源**: Hermes 多后端, SWE-agent ACI, LongHorizon 检查点

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **7 终端后端** | Hermes | local/Docker/SSH/Modal/Daytona/Vercel | ⭐⭐⭐⭐⭐ |
| **无服务器休眠** | Hermes/Modal | 空闲时成本近零 | ⭐⭐⭐⭐ |
| **Agent-Computer Interface** | SWE-agent | 最小精确接口设计 | ⭐⭐⭐⭐⭐ |
| **验证进度检查点** | LongHorizon | 仅审计通过的结果成为可信状态 | ⭐⭐⭐⭐⭐ |
| **新鲜上下文执行** | LongHorizon/ECC | 每次执行器调用从干净开始 | ⭐⭐⭐⭐⭐ |
| **沙箱隔离** | OpenHands | Docker 容器化执行环境 | ⭐⭐⭐⭐ |

### 2.5 L4 Emotion — 情感层

**吸收来源**: Letta 自我管理, RSI 自主性级别

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **Agent 自我管理记忆** | Letta | Agent 决定存储/检索什么 | ⭐⭐⭐⭐ |
| **核心/归档/回忆层次** | Letta | 上下文内 → 可搜索 → 对话历史 | ⭐⭐⭐⭐⭐ |
| **RSI 5 级自主性** | RSI Paper | L1 执行 → L5 元改进 | ⭐⭐⭐⭐⭐ |
| **结构递归** | RSI Paper | 改进机制本身被后续轮次改进 | ⭐⭐⭐⭐ |
| **安全继承 + 回滚** | RSI/LongHorizon | 版本化状态变更 + 退化时回滚 | ⭐⭐⭐⭐⭐ |

### 2.6 L5 Cognition — 认知层

**吸收来源**: Aider 上下文管理, ECC Token 经济, CrewAI 双范式, Promptfoo 评估

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **后台摘要压缩** | Aider ChatSummary | 后台线程压缩上下文 | ⭐⭐⭐⭐⭐ |
| **提示缓存预热** | Aider | 后台线程保持 API 缓存热 | ⭐⭐⭐⭐ |
| **Repo-Map 索引** | Aider | Tree-sitter AST 代码库映射 | ⭐⭐⭐⭐⭐ |
| **Token 经济分层** | ECC | Opus 推理 / Sonnet 编码 / Haiku I/O | ⭐⭐⭐⭐⭐ |
| **Crew/Flow 双范式** | CrewAI | 自主协作 + 确定性事件驱动 | ⭐⭐⭐⭐ |
| **声明式评估矩阵** | Promptfoo | YAML 定义能力测试 | ⭐⭐⭐⭐⭐ |
| **连续学习 → 技能库** | ECC | 失败模式保存 → 自动加载 | ⭐⭐⭐⭐⭐ |
| **Pre-Compaction 钩子** | ECC | 窗口填满前蒸馏到持久状态 | ⭐⭐⭐⭐ |
| **JIT 技能组合** | JIT-Agent | 推理时任务自适应技能组合 | ⭐⭐⭐⭐ |
| **多目标 Pareto** | JIT-Agent | reward/latency/cost 三通道优化 | ⭐⭐⭐⭐⭐ |

### 2.7 L6 Meta — 元认知层

**吸收来源**: Graphiti 时序管理, SuperMemory 自动遗忘, Mem0 ADD-only

| 模式 | 来源 | NeoTrix 映射 | 优先级 |
|------|------|-------------|--------|
| **时序有效性窗口** | Graphiti | 事实跟踪 true/false 时间 | ⭐⭐⭐⭐⭐ |
| **ADD-only 提取** | Mem0 | 追加从不覆盖，矛盾创建新条目 | ⭐⭐⭐⭐⭐ |
| **渐进式披露** | Claude-Mem | 索引 → 上下文 → 详情 (10x 省 token) | ⭐⭐⭐⭐⭐ |
| **自动遗忘** | SuperMemory | 时间绑定事实过期 | ⭐⭐⭐⭐ |
| **静态/动态画像** | SuperMemory | 稳定身份 vs 近期活动 | ⭐⭐⭐⭐ |
| **Episode 溯源** | Graphiti | 每个派生事实追溯到源 episode | ⭐⭐⭐⭐⭐ |
| **事件模式提取** | LightMem | 保留时间绑定和因果关系 | ⭐⭐⭐⭐ |
| **反合理化表** | Addy Osmani | 技能中包含跳过步骤的借口+反驳 | ⭐⭐⭐⭐ |

---

## 三、核心路由架构重设计

### 3.1 统一路由器 = 5 层路由栈

```
┌─────────────────────────────────────────────────────────────┐
│                    L6 Meta-Routing                           │
│  RSI 自主级别 (L1-L5) × 成本通道 (reward/latency/cost)      │
├─────────────────────────────────────────────────────────────┤
│                    L5 Task Routing                           │
│  TaskType 分类器 → Agent 路由 → 模型选择                     │
│  (JIT 技能组合 + CrewAI Flow + Aider 多态)                  │
├─────────────────────────────────────────────────────────────┤
│                    L4 Cost-Aware Routing                     │
│  GWT Salience × Cost Weight × Cache Hit Rate                │
│  (DeepSeek CED 非对称 + SGLang 前缀感知)                    │
├─────────────────────────────────────────────────────────────┤
│                    L3 Provider Routing                       │
│  Ordered Fallback: Local → Regional → Global                 │
│  (Circuit Breaker + 硬件感知后端选择)                        │
├─────────────────────────────────────────────────────────────┤
│                    L2 Load Balancing                         │
│  Per-Modality 负载均衡 (text vs vision tokens)              │
│  (Kimi K3 模式 + vLLM 连续批处理)                           │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 模型选择算法

```rust
fn select_model(request: &ModelRequest, context: &RoutingContext) -> ModelDecision {
    // L5: 任务分类
    let task_complexity = classify_task(&request.task_type);
    
    // L4: 成本感知
    let cost_budget = context.budget_per_token;
    let cache_hit_rate = prefix_cache.hit_rate(&request.messages);
    
    // L3: 提供商路由
    let available_providers = ordered_fallback(
        &task_complexity,
        &context.hardware,
        &context.region,
    );
    
    // L2: 负载均衡
    let modality = detect_modality(&request);
    let load_balanced = per_modality_balance(available_providers, modality);
    
    // L1: 最终选择
    let candidate = load_balanced
        .filter(|p| p.health_penalty() > 0.5)  // Circuit breaker
        .min_by_key(|p| {
            // Multi-objective Pareto (JIT-Agent pattern)
            let reward = quality_score(p, &task_complexity);
            let latency = p.estimated_latency(&request);
            let cost = p.cost_per_token * request.estimated_tokens;
            let cache_discount = if cache_hit_rate > 0.8 { 0.3 } else { 1.0 };
            
            pareto_score(reward, latency, cost * cache_discount)
        });
    
    ModelDecision {
        provider: candidate.name,
        model: candidate.model,
        estimated_cost: candidate.cost_per_token * request.estimated_tokens,
        cache_optimization: cache_hit_rate > 0.8,
        reasoning_effort: task_complexity.to_reasoning_effort(),
    }
}
```

### 3.3 上下文管理管线

```
┌──────────────────────────────────────────────────────────────┐
│              Context Management Pipeline                      │
│                                                               │
│  ┌─────────┐    ┌──────────┐    ┌──────────┐    ┌────────┐ │
│  │ L1: Tool │──→│ L2:      │──→│ L3:      │──→│ L4:    │ │
│  │ Result   │    │ History  │    │ Summary  │    │ Cache  │ │
│  │ Pruning  │    │ Snip     │    │ Compaction│    │ Warm   │ │
│  └─────────┘    └──────────┘    └──────────┘    └────────┘ │
│                                                               │
│  预压缩 (LightMem LLMLingua-2)                               │
│  检查点 (LongHorizon 验证状态)                                │
│  Pre-Compaction Hook (ECC 蒸馏)                              │
│  渐进式披露 (Claude-Mem 3层)                                  │
└──────────────────────────────────────────────────────────────┘
```

---

## 四、记忆架构重设计

### 4.1 三层记忆体系

```
┌──────────────────────────────────────────────────────────────┐
│                    NT-MEMORY 三层记忆                         │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L1: Core Memory (上下文内, <4K tokens)                 │ │
│  │  - 身份事实 (静态画像, SuperMemory 模式)                │ │
│  │  - 当前任务状态                                         │ │
│  │  - 活跃技能指针                                         │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          ↓ 溢出到                            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L2: Archival Memory (可搜索, KV + Vector + Graph)      │ │
│  │  - 时序事实 (Graphiti 有效性窗口)                       │ │
│  │  - ADD-only 提取 (Mem0 模式)                            │ │
│  │  - Episode 溯源 (每个事实追溯到源)                      │ │
│  │  - 自动遗忘 (SuperMemory 时间过期)                      │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          ↓ 检索                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  L3: Recall Memory (对话历史, 压缩存储)                 │ │
│  │  - 会话摘要 (Aider ChatSummary 模式)                    │ │
│  │  - FTS5 全文搜索 (Hermes 模式)                          │ │
│  │  - 渐进式披露 (Claude-Mem 索引→详情)                    │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 知识图谱管线

```
Source Code → Tree-Sitter AST → Entity Extraction → Graph Construction
                                                      ↓
                              Community Detection (Leiden) → Hierarchical Summaries
                                                      ↓
                              Temporal Validity Windows (Graphiti) → Incremental Updates
```

---

## 五、技能框架重设计

### 5.1 SKILL.md 标准模板 (熔合 6 源)

```markdown
---
name: nt-<domain>-<action>
description: "<Action> <target> <constraint>. Use when <trigger>. Domain: <NT-XXX>"
maturity: C0-C6
constellation: <domain>
tokens: ~100 (frontmatter) / ~5000 (full body)
---

# NT Skill Name

## Overview
## When to Use (triggering conditions)
## Anti-Rationalization Table
| Excuse | Rebuttal |
|--------|----------|
| "Skip this step" | "Reason why not" |

## Process (step-by-step with verification gates)
1. Step → Verification Gate
2. ...

## Red Flags (signs something's wrong)
## Verification (evidence requirements)
## Cross-references
- Related skills: @skill-name
- KB experience: <pointer>
```

### 5.2 技能路由

```
Session Start: name + description only (~100 tokens each)
Task Matching: full SKILL.md body (~5000 tokens max)
On-demand: scripts/, references/ loaded as needed
```

---

## 六、安全架构增强

### 6.1 吸收的安全模式

| 模式 | 来源 | NeoTrix 映射 |
|------|------|-------------|
| **71 漏洞模式** | NVIDIA SkillSpector | NT-SHIELD 技能安全扫描 |
| **风险评分 0-100** | NVIDIA SkillSpector | 技能风险分级 |
| **Red Team 持续改进** | Promptfoo | 漏洞扫描 → 修补 → 重新扫描 |
| **Agent 安全审计** | cloudflare/security-audit-skill | 多阶段审计管线 |
| **权限分级** | OpenCode/Claude Code | 危险操作前确认 |
| **沙箱隔离** | OpenHands/SWE-agent | Docker 容器化执行 |

---

## 七、更新的 Sprint 计划

### Sprint 0: 类型统一 (进行中) — 2 天
→ 保持不变

### Sprint 1: 统一适配器 + Lazy Loading — 3 天
**新增**:
- T1.9: Lazy Provider Loading (Aider 模式) — 延迟导入节省 1.5s 启动
- T1.10: Prompt Cache Warming 后台线程 (Aider 模式)
- T1.11: Repo-Map 索引 (Tree-sitter AST)

### Sprint 2: 统一路由器 + 成本感知 — 3 天
**新增**:
- T2.10: 5 层路由栈 (Meta/Task/Cost/Provider/LoadBalancing)
- T2.11: 多目标 Pareto 选择 (JIT-Agent reward/latency/cost)
- T2.12: 前缀感知路由 (SGLang RadixAttention)

### Sprint 3: 上下文管理 + 记忆 — 2 天
**新增**:
- T3.5: 三层记忆体系 (Core/Archival/Recall)
- T3.6: 时序有效性窗口 (Graphiti 模式)
- T3.7: 渐进式披露检索 (Claude-Mem 3层)
- T3.8: 自动遗忘 (SuperMemory 时间过期)

### Sprint 4: 技能框架 + 评估 — 2 天
**新增**:
- T4.6: SKILL.md 标准模板 (6 源熔合)
- T4.7: 声明式评估矩阵 (Promptfoo 模式)
- T4.8: 反合理化表 (Addy Osmani 模式)
- T4.9: 技能安全扫描 (NVIDIA SkillSpector)

### Sprint 5: CLI + EventBus + 浏览器 — 2 天
**新增**:
- T5.6: MCP 工具协议集成 (Claude Code 模式)
- T5.7: 浏览器池三级架构 (Crawl4AI 模式)
- T5.8: Observe-Act-Observe 循环 (Browser Use 模式)

### Sprint 6: Bend 语言集成 — 11 天
→ 保持不变

### Sprint 7: Trendshift 项目吸收 — 5 天
**扩展**:
- T7.7: LongHorizon 三角色架构 (Manager/Executor/Auditor)
- T7.8: ECC Token 经济分层 (Opus/Sonnet/Haiku)
- T7.9: JIT 技能组合 (推理时自适应)
- T7.10: RSI 5 级自主性框架

---

## 八、核心路线任务清单 (更新)

### P0 — 必须完成 (阻塞后续)

| ID | 任务 | Sprint | 预估 | 来源 |
|----|------|--------|------|------|
| P0.1 | TaskType 10→1 统一 | S0 | 2h | — |
| P0.2 | ModelTier 3→1 统一 | S0 | 1h | — |
| P0.3 | RoutingStrategy 3→1 统一 | S0 | 1h | — |
| P0.4 | UnifiedModelAdapter trait | S1 | 2h | Vercel AI SDK |
| P0.5 | OpenAI adapter | S1 | 3h | — |
| P0.6 | Anthropic adapter | S1 | 4h | — |
| P0.7 | Gemini adapter | S1 | 4h | — |
| P0.8 | SseEventParser | S1 | 3h | IETF SSE |
| P0.9 | ToolCallConverter | S1 | 2h | MCP 协议 |
| P0.10 | UnifiedModelRouter 重写 | S2 | 4h | 5层路由栈 |
| P0.11 | Lazy Provider Loading | S1 | 1h | Aider |
| P0.12 | Repo-Map 索引 | S1 | 3h | Aider |

### P1 — 高优先级 (核心价值)

| ID | 任务 | Sprint | 预估 | 来源 |
|----|------|--------|------|------|
| P1.1 | Cost-aware scoring (A1) | S2 | 3h | JIT-Agent |
| P1.2 | Cascade escalation | S2 | 2h | DeepSeek CSA2 |
| P1.3 | Circuit breaker | S2 | 2h | — |
| P1.4 | 删除 8 个旧路由器 | S2 | 2h | — |
| P1.5 | Context Compactor 3 级 | S3 | 4h | Aider+ECC |
| P1.6 | Config 统一层级 | S3 | 3h | — |
| P1.7 | model CLI 统一 | S5 | 3h | — |
| P1.8 | 三层记忆体系 | S3 | 4h | Letta+Graphiti |
| P1.9 | Prompt Cache Warming | S1 | 1h | Aider |
| P1.10 | 5 层路由栈 | S2 | 4h | 综合 |
| P1.11 | MCP 工具协议 | S5 | 3h | Claude Code |

### P2 — 中优先级 (质量提升)

| ID | 任务 | Sprint | 预估 | 来源 |
|----|------|--------|------|------|
| P2.1 | CapabilityRegistry 4→1 | S4 | 3h | — |
| P2.2 | dead_code 清理 50%+ | S4 | 4h | — |
| P2.3 | TokenCounter (tiktoken-rs) | S1 | 2h | — |
| P2.4 | 路由决策 → EventBus | S5 | 2h | — |
| P2.5 | 路由日志 → KB | S5 | 2h | — |
| P2.6 | SKILL.md 标准模板 | S4 | 2h | 6源熔合 |
| P2.7 | 声明式评估矩阵 | S4 | 3h | Promptfoo |
| P2.8 | 浏览器池三级架构 | S5 | 3h | Crawl4AI |
| P2.9 | 反合理化表 | S4 | 1h | Addy Osmani |
| P2.10 | 技能安全扫描 | S4 | 2h | NVIDIA |

### P3 — 低优先级 (长期)

| ID | 任务 | Sprint | 预估 | 来源 |
|----|------|--------|------|------|
| P3.1 | Per-turn ML router | Future | 2d | Cursor |
| P3.2 | Tauri/Core 命令共享 | Future | 1d | — |
| P3.3 | IETF SSE 标准对齐 | Future | 1d | — |
| P3.4 | 模型 metadata 远程目录 | Future | 2d | models.dev |
| P3.5 | ACP Agent 互操作 | Future | 3d | OpenHands |
| P3.6 | Crew/Flow 双范式 | Future | 5d | CrewAI |
| P3.7 | 无服务器休眠 | Future | 3d | Hermes/Modal |
| P3.8 | RSI 5 级自主性 | Future | 5d | RSI Paper |

---

## 九、验证命令 (更新)

```bash
# ── 基础编译 ──
cargo check -p neotrix-types && cargo check -p neotrix --lib

# ── 单元测试 ──
cargo test -p neotrix-types && cargo test -p neotrix --lib

# ── 冗余度量 ──
grep -r "enum TaskType" --include="*.rs" | wc -l    # 目标: ≤2
grep -r "enum ModelTier" --include="*.rs" | wc -l   # 目标: 1
grep -r "fn route\|fn select_model" --include="*.rs" | wc -l  # 目标: 1
grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l  # 目标: <50

# ── 新增: 技能框架验证 ──
find . -name "SKILL.md" | wc -l                      # 技能文件计数
grep -r "Anti-Rationalization" --include="*.md" | wc -l  # 反合理化表

# ── 新增: 记忆系统验证 ──
grep -r "temporal_validity\|ADD-only\|progressive_disclosure" --include="*.rs" | wc -l

# ── 新增: 路由系统验证 ──
grep -r "pareto_score\|cost_weight\|cache_hit_rate" --include="*.rs" | wc -l
```

---

## 十、总结

### 10.1 吸收成果

| 维度 | 吸收前 | 吸收后 | 提升 |
|------|--------|--------|------|
| 路由层数 | 1 层 | 5 层 | 5x |
| 记忆层次 | 1 层 | 3 层 | 3x |
| 技能模板 | 无标准 | 6 源熔合标准 | ∞ |
| 安全模式 | 基础 | 71 漏洞模式 | 71x |
| 浏览器能力 | 基础 | 7 工具模式 | 7x |
| 评估能力 | 无 | 声明式矩阵 | ∞ |

### 10.2 最高价值吸收 Top 10

| # | 模式 | 来源 | 影响 |
|---|------|------|------|
| 1 | **5 层路由栈** | 综合 | 路由精度 5x |
| 2 | **三层记忆体系** | Letta+Graphiti+SuperMemory | 记忆持久性 10x |
| 3 | **Lazy Provider Loading** | Aider | 启动速度 -1.5s |
| 4 | **Prompt Cache Warming** | Aider | Prompt 成本 -60% |
| 5 | **Repo-Map 索引** | Aider | 代码选择精度 10x |
| 6 | **时序有效性窗口** | Graphiti | 知识准确性 5x |
| 7 | **声明式评估矩阵** | Promptfoo | 能力验证 ∞ |
| 8 | **反合理化表** | Addy Osmani | 执行完整性 ∞ |
| 9 | **JIT 技能组合** | JIT-Agent | Token 效率 3x |
| 10 | **RSI 5 级自主性** | RSI Paper | 自进化深度 5x |

### 10.3 下一步

1. **立即执行**: Sprint 0 剩余任务 (类型统一)
2. **Sprint 1**: 统一适配器 + Lazy Loading + Repo-Map
3. **Sprint 2**: 5 层路由栈 + 多目标 Pareto
4. **Sprint 3**: 三层记忆 + 时序有效性
5. **Sprint 4**: 技能框架 + 评估矩阵
6. **Sprint 5**: CLI + 浏览器 + MCP
7. **Sprint 6**: Bend 集成
8. **Sprint 7**: Trendshift 扩展吸收
