# NeoTrix 数据源深度分析报告
## 基于2026年LLM架构趋势（Hybrid + MoE + Efficient Attention）

生成时间：2026-05-06
数据源：open开头 code agent + Hermes生态项目（共15个）

---

## 一、架构分布统计

| 架构类型 | 项目数量 | 占比 | 代表项目 |
|---------|---------|------|---------|
| Hybrid (Transformer+SSM) | 6 | 40% | openclaw/openclaw, open-webui/open-webui, NousResearch/hermes-agent |
| Pipeline (多阶段) | 4 | 26.7% | OpenGraph-AI/OpenAgent, NourResearch/hermes-agent-self-evolution, NourResearch/autonovel |
| LocalFirst (能效优化) | 2 | 13.3% | open-jarvis/OpenJarvis, mem0ai/mem0 |
| MoE (稀疏激活) | 1 | 6.7% | openai/gpt-oss |
| Transformer (优化版) | 1 | 6.7% | openai/openai-agents-python |
| Diffusion (非自回归) | 1 | 6.7% | ssgantayat/Open-dLLM |

**关键发现**：Hybrid架构主导（40%），符合2026年“混合化”趋势；Pipeline架构占26.7%，反映多阶段处理需求。

---

## 二、项目优先级排序（迭代顺序）

| 排名 | 项目 | 架构 | stars | 优先级 | 主要影响维度 |
|------|------|------|-------|--------|---------|
| 1 | openclaw/openclaw | Hybrid | 357,817 | 10 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 2 | openai/openai-agents-python | Transformer | 25,912 | 9 | inference_depth(0.15), analysis(0.12), synthesis(0.10) |
| 3 | openai/gpt-oss | MoE | 20,050 | 9 | synthesis(0.15), domain_specificity(0.12), inference_depth(0.10) |
| 4 | open-webui/open-webui | Hybrid | 135,000 | 8 | ui_native_states(0.15), semantic_layer(0.12), inference_depth(0.12) |
| 5 | Gen-Verse/OpenClaw-RL | Hybrid | 5,196 | 8 | inference_depth(0.15), analysis(0.12), synthesis(0.12) |
| 6 | mem0ai/mem0 | LocalFirst | 25,000 | 8 | inference_depth(0.15), domain_specificity(0.12), analysis(0.10) |
| 7 | NourResearch/hermes-agent | Hybrid | 15,000 | 9 | inference_depth(0.18), creativity(0.12), analysis(0.10) |
| 8 | NourResearch/hermes-agent-self-evolution | Pipeline | 2,500 | 8 | analysis(0.18), synthesis(0.15), inference_depth(0.12) |
| 9 | open-jarvis/OpenJarvis | LocalFirst | 2,019 | 7 | inference_depth(0.12), domain_specificity(0.12), analysis(0.08) |
| 10 | ssgantayat/Open-dLLM | Diffusion | 500 | 7 | creativity(0.18), experimental(0.15), analysis(0.08) |
| 11 | NourResearch/autonovel | Pipeline | 1,200 | 6 | creativity(0.20), synthesis(0.15), analysis(0.10) |
| 12 | 0xNyk/awesome-hermes-agent | Pipeline | 1,800 | 5 | analysis(0.12), synthesis(0.10), domain_specificity(0.10) |
| 13 | outsoure-e/hermes-workspace | Hybrid | 600 | 7 | inference_depth(0.15), analysis(0.12), synthesis(0.10) |
| 14 | nesquena/hermes-web-ui | Hybrid | 800 | 6 | ui_native_states(0.18), semantic_layer(0.12), inference_depth(0.10) |
| 15 | OpenGraph-AI/OpenAgent | Pipeline | 8 | 6 | analysis(0.15), synthesis(0.15), inference_depth(0.10) |

---

## 三、模块迭代路径映射

### 1. ReasoningBrain 模块
**触发条件**：Transformer、MoE、LocalFirst架构项目更新
**迭代动作**：
- Transformer：更新 `inference_depth`(+0.15), `analysis`(+0.12)
- MoE：更新 `synthesis`(+0.15), `domain_specificity`(+0.12)
- LocalFirst：更新 `inference_depth`(+0.12), `domain_specificity`(+0.12)

**涉及项目**：openai/openai-agents-python, openai/gpt-oss, open-jarvis/OpenJarvis, mem0ai/mem0

### 2. SelfIteratingBrain 模块
**触发条件**：Hybrid、Transformer、Pipeline架构项目更新
**迭代动作**：
- Hybrid：运行 `run_seal_loop()` 强化学习
- Transformer：应用 SEAL 循环更新权重
- Pipeline：执行多阶段迭代（Intent→Clarifier→Planner→Executor）

**涉及项目**：openclaw/openclaw, open-webui/open-webui, Gen-Verse/OpenClaw-RL, NourResearch/hermes-agent, NourResearch/hermes-agent-self-evolution

### 3. ReasoningBank 模块
**触发条件**：Hybrid、MoE、Pipeline架构项目更新
**迭代动作**：
- 存储新特性为 `ReasoningMemory`
- 提取洞察（`extract_insights()`）
- 多维度检索优化（task_type 过滤 + 相似度加权）

**涉及项目**：openclaw/openclaw, openai/gpt-oss, open-webui/open-webui, NourResearch/autonovel, 0xNyk/awesome-hermes-agent

### 4. SelfEvolver 模块
**触发条件**：Hybrid、Pipeline、LocalFirst架构项目更新
**迭代动作**：
- 定期GitHub项目检查（`start_periodic_check()`）
- 架构感知代码审查（`code_review_project()`）
- 自动增删数据源（`add_data_source()` / `remove_data_source()`）

**涉及项目**：openclaw/openclaw, NourResearch/hermes-agent, NourResearch/hermes-agent-self-evolution, NourResearch/autonovel, open-jarvis/OpenJarvis, mem0ai/mem0

### 5. PerformanceEvaluator 模块
**触发条件**：Transformer、Hybrid、Pipeline架构项目更新
**迭代动作**：
- 评估下游任务性能（`evaluate()`）
- 计算奖励分数（reward = capability_score_after - capability_score_before）
- 正则化防止能力向量偏离过远

**涉及项目**：openai/openai-agents-python, openclaw/openclaw, Gen-Verse/OpenClaw-RL, NourResearch/hermes-agent-self-evolution

---

## 四、完整迭代流程图

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
       ├─> Transformer: update inference_depth, analysis
       ├─> MoE: update synthesis, domain_specificity
       ├─> Hybrid: update inference_depth, creativity
       ├─> Diffusion: update experimental, creativity
       └─> Pipeline: update analysis, synthesis

2. 推理能力自我迭代 (Reasoning Capability)
   ├─> 生成self-edit (generate_self_edit)
   ├─> 应用架构特定更新 (apply architecture-specific updates)
   ├─> 评估性能 (PerformanceEvaluator)
   └─> 持久化 (brain.save())

3. 各模块迭代 (Module-Specific Iteration)
   ├─> ReasoningBrain: 能力向量更新
   ├─> SelfIteratingBrain: run_seal_loop()
   ├─> ReasoningBank: extract_insights()
   ├─> SelfEvolver: evolve_from_url()
   └─> PerformanceEvaluator: re-evaluate()

4. 架构特定优化 (2026趋势)
   ├─> Transformer (CS336模板): Pre-Norm+RMSNorm+RoPE+SwiGLU
   ├─> MoE (DeepSeek V4): 稀疏激活+MLA KV压缩
   ├─> Hybrid (Transformer+SSM): 局部精确+长程线性
   └─> Diffusion (Open-dLLM): 非自回归并行生成
```

---

## 五、数据源管理策略

### 新增数据源（Hermes生态）
已成功添加7个Hermes相关项目到 `code_agent_analysis.rs`：
1. NourResearch/hermes-agent (Hybrid, 优先级9)
2. NourResearch/hermes-agent-self-evolution (Pipeline, 优先级8)
3. NourResearch/autonovel (Pipeline, 优先级6)
4. mem0ai/mem0 (LocalFirst, 优先级8)
5. nesquena/hermes-web-ui (Hybrid, 优先级6)
6. outsoure-e/hermes-workspace (Hybrid, 优先级7)
7. 0xNyk/awesome-hermes-agent (Pipeline, 优先级5)

### 数据源增删规则
- **新增**：`code_review_project()` 评分 > 0.7 → `add_data_source()`
- **删除**：评分 < 0.3 或 `days_since_push > 90` → `remove_data_source()`
- **更新**：stars增长 >10% 或 `latest_release` 变化 → 触发 `trigger_self_iteration()`

### 自动迭代触发
- **定期检查**：每24小时运行 `start_periodic_check()`
- **架构感知**：根据项目架构类型自动路由到对应模块
- **奖励计算**：PerformanceEvaluator 计算 reward = capability_score_after - capability_score_before + 正则化项

---

## 六、2026架构趋势落地建议

1. **优先Hybrid架构**：openclaw、open-webui、hermes-agent等主导，应重点迭代 `SelfIteratingBrain` 和 `ReasoningBank`
2. **MoE稀疏激活**：gpt-oss项目，侧重 `synthesis` 和 `domain_specificity` 维度
3. **Pipeline多阶段**：hermes自我进化、autonovel等，强化 `analysis` 和 `synthesis` 流水线
4. **能效优化**：OpenJarvis、mem0，更新 `inference_depth` 和 `domain_specificity`
5. **扩散模型**：Open-dLLM，探索 `creativity` 和 `experimental` 维度

---

## 七、下一步行动

1. **编译修复**：修复 `mcp_gateway.rs` 未闭合分隔符错误（第447行）
2. **测试验证**：运行 `cargo test --lib -- code_agent_analysis` 验证新增项目
3. **数据注入**：调用 `ReasoningBank::seed_open_code_agents()` 注入初始数据
4. **启动迭代**：调用 `SelfEvolver::start_periodic_check()` 开始自动监测
5. **性能监控**：通过 `PerformanceEvaluator` 跟踪各模块迭代效果

---

*报告结束*
