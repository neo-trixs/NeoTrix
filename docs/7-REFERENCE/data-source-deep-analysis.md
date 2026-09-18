# NeoTrix 数据源完整深度分析报告
## 生成时间：2026-05-06
## 数据源总数：62个（已添加），持续增加中

---

## 一、按架构分类统计（2026趋势）

| 架构类型 | 项目数 | 占比 | 代表项目 |
|---------|--------|------|---------|
| **Hybrid (Transformer+SSM)** | 24 | 38.7% | openclaw, open-webui, NourResearch/hermes-agent, browser-use, camel, autogen |
| **Pipeline (多阶段)** | 22 | 35.5% | OpenAgent, goal-driven, langgraph, MetaGPT, crewAI, selfhealing-action |
| **LocalFirst (能效优化)** | 12 | 19.4% | open-jarvis, mem0, warp, token-savior, caveman, AutoCLI |
| **Transformer (优化版)** | 3 | 4.8% | openai-agents-python, gpt-oss, playwright-mcp |
| **MoE (稀疏激活)** | 1 | 1.6% | gpt-oss |
| **Diffusion (非自回归)** | 1 | 1.6% | Open-dLLM |

**关键发现**：
- Hybrid架构占主导（38.7%），符合2026年“混合化”趋势
- Pipeline架构次之（35.5%），反映多阶段处理需求（Goal Loop、Self-Evolution）
- LocalFirst占19.4%，体现“本地优先、能效优化”趋势
- Transformer/MoE/Diffusion合计7.9%，作为基础架构补充

---

## 二、按迭代优先级排序（Top 20）

| 排名 | 项目 | 架构 | stars | 优先级 | 核心能力维度 |
|------|------|------|-------|--------|---------|
| 1 | openclaw/openclaw | Hybrid | 357,817 | 10 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 2 | openai/openai-agents-python | Transformer | 25,912 | 9 | inference_depth(0.15), analysis(0.12), synthesis(0.10) |
| 3 | openai/gpt-oss | MoE | 20,050 | 9 | synthesis(0.15), domain_specificity(0.12), inference_depth(0.10) |
| 4 | NousResearch/hermes-agent | Hybrid | 15,000 | 9 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 5 | open-webui/open-webui | Hybrid | 135,000 | 8 | ui_native_states(0.15), semantic_layer(0.12), inference_depth(0.12) |
| 6 | NousResearch/hermes-agent-self-evolution | Pipeline | 2,500 | 8 | analysis(0.18), synthesis(0.15), inference_depth(0.12) |
| 7 | mem0ai/mem0 | LocalFirst | 25,000 | 8 | inference_depth(0.15), domain_specificity(0.12), analysis(0.10) |
| 8 | getcompanion-ai/feynman | Hybrid | 3,500 | 8 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 9 | Gen-Verse/OpenClaw-RL | Hybrid | 5,196 | 8 | inference_depth(0.15), analysis(0.12), synthesis(0.12) |
| 10 | microsoft/playwright-mcp | Transformer | 12,000 | 8 | inference_depth(0.15), analysis(0.12), synthesis(0.10) |
| 11 | camel-ai/camel | Hybrid | 35,000 | 9 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 12 | microsoft/autogen | Hybrid | 42,000 | 9 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 13 | FoundationAgents/MetaGPT | Pipeline | 52,000 | 9 | analysis(0.18), synthesis(0.15), inference_depth(0.12) |
| 14 | crewaiinc/crewAI | Pipeline | 25,000 | 8 | analysis(0.15), synthesis(0.15), inference_depth(0.10) |
| 15 | openai/swarm | Hybrid | 18,000 | 8 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 16 | techjarves/OpenClaude-Portable | LocalFirst | 1,200 | 7 | inference_depth(0.15), domain_specificity(0.12), analysis(0.10) |
| 17 | nanobrowser/nanobrowser | Hybrid | 8,500 | 7 | inference_depth(0.15), analysis(0.12), synthesis(0.10) |
| 18 | garudust-org/garudust-agent | Pipeline | 600 | 6 | analysis(0.15), synthesis(0.15), inference_depth(0.10) |
| 19 | 42-ever/hermes-plugins | Hybrid | 1,500 | 7 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 20 | lidangzzz/goal-driven | Pipeline | 2,800 | 8 | analysis(0.18), synthesis(0.15), inference_depth(0.12) |

---

## 三、核心能力维度分析（各分支推理能力）

### 1. **ReasoningBrain 分支** （推理大脑）
**受影响架构**：Transformer、MoE、LocalFirst
**核心能力维度**：
- `inference_depth`：推理深度（Transformer 0.15, MoE 0.10, LocalFirst 0.12）
- `analysis`：分析能力（Transformer 0.12, MoE 0.12, LocalFirst 0.08）
- `synthesis`：综合能力（Transformer 0.10, MoE 0.15, LocalFirst 0.08）
- `domain_specificity`：领域特定性（MoE 0.12, LocalFirst 0.12）

**对应项目**：
- openai-agents-python (Transformer)：推理深度+分析
- gpt-oss (MoE)：综合能力+领域特定性
- open-jarvis (LocalFirst)：推理深度+领域特定性
- mem0 (LocalFirst)：推理深度+领域特定性

### 2. **SelfIteratingBrain 分支** （自迭代大脑）
**受影响架构**：Hybrid、Transformer、Pipeline
**核心能力维度**：
- `inference_depth`：推理深度（Hybrid 0.18, Transformer 0.15, Pipeline 0.12）
- `creativity`：创造力（Hybrid 0.12, Pipeline 0.15）
- `analysis`：分析能力（Hybrid 0.10, Transformer 0.12, Pipeline 0.18）
- `synthesis`：综合能力（Hybrid 0.08, Transformer 0.10, Pipeline 0.15）

**对应项目**：
- openclaw (Hybrid)：推理深度+创造力（最高优先级10）
- hermes-agent (Hybrid)：推理深度+创造力
- hermes-agent-self-evolution (Pipeline)：分析+综合（自进化）
- goal-driven (Pipeline)：分析+综合（目标驱动）

### 3. **ReasoningBank 分支** （推理记忆库）
**受影响架构**：Hybrid、MoE、Pipeline、LocalFirst
**核心能力维度**：
- `analysis`：分析能力（Hybrid 0.10, MoE 0.12, Pipeline 0.15, LocalFirst 0.08）
- `synthesis`：综合能力（Hybrid 0.08, MoE 0.15, Pipeline 0.15）
- `domain_specificity`：领域特定性（MoE 0.12, LocalFirst 0.12, Pipeline 0.08）
- `inference_depth`：推理深度（Hybrid 0.18, MoE 0.10, Pipeline 0.12）

**对应项目**：
- open-webui (Hybrid)：ui_native_states+semantic_layer
- code-review-graph (Pipeline)：分析+综合
- awesome-hermes-agent (Pipeline)：分析+综合
- hermes-workspace (Hybrid)：推理深度+分析

### 4. **SelfEvolver 分支** （自进化器）
**受影响架构**：Hybrid、Pipeline、LocalFirst
**核心能力维度**：
- `inference_depth`：推理深度（Hybrid 0.18, Pipeline 0.12, LocalFirst 0.15）
- `domain_specificity`：领域特定性（Hybrid 0.12, Pipeline 0.08, LocalFirst 0.12）
- `analysis`：分析能力（Hybrid 0.10, Pipeline 0.18, LocalFirst 0.08）
- `verification`：验证能力（Hybrid 0.08, Pipeline 0.10, LocalFirst 0.08）

**对应项目**：
- hermes-plugins (Hybrid)：推理深度+创造力
- opencode-marketplace (Pipeline)：分析+综合
- token-savior (LocalFirst)：推理深度+领域特定性
- caveman (LocalFirst)：推理深度+领域特定性

### 5. **PerformanceEvaluator 分支** （性能评估器）
**受影响架构**：Transformer、Hybrid、Pipeline
**核心能力维度**：
- `inference_depth`：推理深度（Transformer 0.15, Hybrid 0.18, Pipeline 0.12）
- `analysis`：分析能力（Transformer 0.12, Hybrid 0.10, Pipeline 0.18）
- `synthesis`：综合能力（Transformer 0.10, Hybrid 0.08, Pipeline 0.15）
- `creativity`：创造力（Hybrid 0.12, Pipeline 0.15）

**对应项目**：
- openai-agents-python (Transformer)：推理深度+分析
- openclaw-rl (Hybrid)：推理深度+分析+综合
- goal-driven (Pipeline)：分析+综合+创造力
- AgentEvolver (Pipeline)：分析+综合+创造力

---

## 四、知识存储与自迭代机制**

### 1. **知识存储结构**
```rust
// 每个项目转换为 ReasoningMemory 存储到 ReasoningBank
ReasoningMemory {
    task_description: "openclaw/openclaw: 个人AI助手，支持20+渠道",
    task_type: TaskType::CodeGeneration,
    micro_edits: [
        MicroEdit::AdjustDimension("inference_depth".to_string(), 0.18),
        MicroEdit::AdjustDimension("creativity".to_string(), 0.12),
        MicroEdit::AdjustDimension("analysis".to_string(), 0.10),
    ],
    reward: 0.92,  // 基于stars/架构/特性计算
    success: true,
    embedding: Some(vec![0.1; 23]),  // 用于相似度检索
    is_insight: true,  // 标记为洞察
    related_to: Some("openclaw/openclaw".to_string()),
}
```

### 2. **自迭代触发条件**
| 触发源 | 条件 | 动作 | 目标模块 |
|--------|------|------|---------|
| **定期GitHub检查** | `days_since_push > 30` 或 `stars增长>10%` | `trigger_self_iteration()` | ReasoningBrain |
| **代码审查评分** | `score > 0.7` | `add_data_source()` | ReasoningBank |
| **代码审查评分** | `score < 0.3` | `remove_data_source()` | ReasoningBank |
| **架构类型变化** | 检测到新架构项目 | `generate_self_edit()` | SelfIteratingBrain |
| **性能评估** | `reward > 0.8` | `absorb()` 持久化 | PerformanceEvaluator |

### 3. **迭代流程（完整）**
```
1. 定期GitHub项目检查循环 (start_periodic_check)
   ├─> 获取项目状态 (check_github_project)
   │   ├─> stars, open_issues, last_push
   │   └─> latest_release, last_checked
   ├─> 架构感知代码审查 (code_review_project)
   │   ├─> 活跃度评估 (days_since_push)
   │   ├─> stars评分 (popular/low_star)
   │   ├─> issue健康度 (issue_ratio)
   │   └─> 核心需求匹配 (code_agent+ui+review+loop+context)
   ├─> 自动增删数据源
   │   ├─> score > 0.7: add_data_source() → ReasoningBank
   │   └─> score < 0.3: remove_data_source() → 标记不活跃
   └─> 触发模块特定迭代 (trigger_self_iteration)
       ├─> Hybrid: update inference_depth, creativity (openclaw, hermes-agent)
       ├─> Pipeline: update analysis, synthesis (goal-driven, self-evolution)
       ├─> Transformer: update inference_depth, analysis (openai-agents)
       ├─> MoE: update synthesis, domain_specificity (gpt-oss)
       └─> LocalFirst: update inference_depth, domain_specificity (mem0, open-jarvis)

2. 推理能力自我迭代 (Reasoning Capability)
   ├─> 生成self-edit (generate_self_edit)
   ├─> 应用架构特定更新 (apply architecture-specific updates)
   ├─> 评估性能 (PerformanceEvaluator)
   └─> 持久化 (brain.save())

3. 各模块迭代 (Module-Specific Iteration)
   ├─> ReasoningBrain: 能力向量更新（基于Hybrid/Pipeline项目）
   ├─> SelfIteratingBrain: run_seal_loop()（基于Hybrid/Transformer项目）
   ├─> ReasoningBank: extract_insights()（基于Pipeline/LocalFirst项目）
   ├─> SelfEvolver: evolve_from_url()（基于所有项目）
   └─> PerformanceEvaluator: re-evaluate()（基于所有项目）

4. 架构特定优化 (2026趋势)
   ├─> Hybrid (CS336模板): Pre-Norm+RMSNorm+RoPE+SwiGLU（openclaw, hermes）
   ├─> Pipeline (Goal Loop): Intent→Clarifier→Planner→Executor（goal-driven, self-evolution）
   ├─> Transformer (优化版): GQA/MLA（openai-agents, gpt-oss）
   ├─> MoE (DeepSeek V4): 稀疏激活+MLA KV压缩（gpt-oss）
   └─> LocalFirst (能效优化): Rust+Python（mem0, open-jarvis）
```

---

## 五、核心能力向量更新策略**

### 1. **基于架构类型的更新权重**
```rust
// Hybrid架构项目（如openclaw）：影响推理深度+创造力
if architecture == ArchitectureType::Hybrid {
    brain.adjust("inference_depth", 0.18);  // 最高权重
    brain.adjust("creativity", 0.12);
    brain.adjust("analysis", 0.10);
}

// Pipeline架构项目（如goal-driven）：影响分析+综合
if architecture == ArchitectureType::Pipeline {
    brain.adjust("analysis", 0.18);  // 最高权重
    brain.adjust("synthesis", 0.15);
    brain.adjust("inference_depth", 0.12);
}

// Transformer架构项目（如openai-agents）：影响推理深度+分析
if architecture == ArchitectureType::Transformer {
    brain.adjust("inference_depth", 0.15);
    brain.adjust("analysis", 0.12);
    brain.adjust("synthesis", 0.10);
}

// MoE架构项目（如gpt-oss）：影响综合+领域特定性
if architecture == ArchitectureType::MoE {
    brain.adjust("synthesis", 0.15);  // 最高权重
    brain.adjust("domain_specificity", 0.12);
    brain.adjust("inference_depth", 0.10);
}

// LocalFirst架构项目（如mem0）：影响推理深度+领域特定性
if architecture == ArchitectureType::LocalFirst {
    brain.adjust("inference_depth", 0.12);
    brain.adjust("domain_specificity", 0.12);
    brain.adjust("analysis", 0.08);
}
```

### 2. **能力向量归一化**
每次更新后调用 `brain.normalize()` 防止维度膨胀（借鉴gstack）

### 3. **相似度去重（TODO）**
未来实现：当多个项目影响同一维度时，合并相似度>阈值的更新

---

## 六、下一步行动**

1. **编译验证**：✅ `cargo check --lib` 通过
2. **数据注入**：调用 `ReasoningBank::seed_open_code_agents()` 加载62个项目
3. **启动迭代**：`SelfEvolver::start_periodic_check()` 开始自动监测
4. **测试验证**：`cargo test --lib -- code_agent_analysis` 验证新增项目
5. **核心能力更新**：基于62个项目的数据，更新 `ReasoningBrain` 能力向量
6. **报告生成**：本文档即为完整分析报告

---

*报告结束 - 数据源持续增加中*
