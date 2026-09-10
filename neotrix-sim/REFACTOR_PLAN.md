# NT-WORLD-SIM: 全量重构任务清单

> 基于 Generative Agents / Project Sid PIANO / CivSim / JaxLife / OpenLife 架构逆向
> + neotrix-sim 内部冗余/扁平/错位审计

## 一、冗余清理（R1-R3）

| ID | 问题 | 修复 | 优先级 |
|----|------|------|--------|
| R1 | 三重记忆：AgentMemory + MemoryStream + GraphMemory 重叠 | 删除 `SimAgent.memory`（AgentMemory），保留 MemoryStream（向量检索）+ GraphMemory（关系图谱），genome trait 改读 MemoryStream.len() | P0 |
| R2 | `cosine_sim` 重复实现（memory_stream + dual_representation） | 提取到 `foundation/math_bridge.rs` 共享 | P2 |
| R3 | SpatialMemory vs SpatialGrid 概念重叠 | 保留区分：SpatialGrid=世界真相（per-tick），SpatialMemory=agent 学习地图（per-agent），文档化差异 | P3 |

## 二、扁平模块激活（F1-F4）

| ID | 问题 | 修复 | 优先级 |
|----|------|------|--------|
| F1 | BehaviorVm 孤岛（Instruction 与 AgentAction 重复） | 转为 AgentAction 的编排层：`BehaviorVm.step()` 返回 `AgentAction`，删除重复的 Instruction enum，改为 `Vec<Vec<AgentAction>>` 序列 | P1 |
| F2 | LlmHooks 纯桩（无真实调用） | 保留为 trait 接口：`trait LlmProvider { fn invoke(&self, prompt: &str) -> String; }`，WorldSim 可选注入 | P2 |
| F3 | DualRepresentation 实例化但从未使用 | 接入 tick loop：每次 agent 行动时 `dual_repr.add(action_label, "action", attrs, tick)` | P2 |
| F4 | EmergenceDetector 655 行从未实例化 | 接入 `TickTier::Slow`：构造 summary 调用 `detector.analyze()` | P1 |

## 三、跨域错位修复（M1-M10）—— 决策环路补全

### M1: PlanningStack → decide_action（**最高优先级**）

**现状**：PlanningStack 有完整的目标生命周期（生成→排序→执行→推进→整合），但 `decide_action()` 是硬编码 if-else 链。

**Generative Agents 参考**：Planning 是核心——"translates conclusions and current environment into high-level action plans, then recursively into detailed behaviors"

**修复**：
```
decide_action() 新流程：
1. planning.generate_survival_goals(health, energy, hunger)
2. planning.generate_social_goals(nearby_agents)
3. planning.generate_exploration_goals(spatial_memory)
4. goal = planning.next_action()
5. if goal 有子步骤 → 取第一个子步骤作为 action
6. fallback → Explore
```

### M2: Personality → decide_action

**现状**：PersonalityDrift 漂移人格但人格不影响决策。agentic 高度 cooperative 与高度 aggressive 行为相同。

**OpenLife 参考**：individuation（个体化）——"agents differentiate over time through experience"

**修复**：`decide_action()` 中加入人格调制：
- `aggression > 0.7` → 有目标时倾向 Attack
- `cooperativeness > 0.7` → 有目标时倾向 Trade/Talk
- `curiosity > 0.7` → 无目标时倾向 Explore
- `sociability > 0.7` → 附近有 agent 时倾向 Talk

### M3: TheoryOfMind → decide_action

**Project Sid 参考**：Social Awareness 模块——"inferred others' sentiments through conversation and adjusted behavior accordingly"

**修复**：`decide_action()` 中遇到附近 agent 时：
- `tom.threat_of(target) > 0.6` → flee
- `tom.cooperativeness_of(target) > 0.6` → prefer Trade/Talk
- `tom.predict_action(target)` → 预判对方行为

### M4: ReflectionEngine → MemoryStream

**Generative Agents 参考**："Reflections are generated periodically when cumulative importance exceeds threshold"

**修复**：`TickTier::Slow` 中：
1. 对每个 agent，累加最近记忆 importance
2. 超过阈值时调 `reflect()` 生成 insight
3. 将 insight 作为 `MemoryKind::Reflection` 加入 MemoryStream

### M5: GraphMemory 写入

**OpenLife 参考**："semantically plastic memory graph"

**修复**：每次记录 MemoryStream 时同步创建 graph node + edge：
- 行动 → `NodeKind::Event`，与前一个行动用 `EdgeKind::Temporal` 连接
- 社交互动 → `NodeKind::Person` + `EdgeKind::Social`
- 决策使用 `spread_activation()` 获取相关记忆

### M6: SpatialMemory 写入

**修复**：agent 移动后调用 `spatial_memory.visit(pos, biome, resources, danger, tick)`
`decide_action()` 中使用 `spatial_memory.safe_locations()` 避开危险区域

### M7: Emotion 调制接入 WorldSim

**修复**：`emotion.process_events()` 后，读取 dominant emotion 的 `gwt_modulation()` 和 `memory_strategy()`：
- 焦虑 → 增加 rest 倾向
- 好奇 → 增加 explore 倾向
- 愤怒 → 增加攻击倾向

### M8: ActionAwareness.should_explore()

**修复**：`decide_action()` 检查 `action_awareness.should_explore()`，低置信度时倾向 Explore

### M9: ConvergenceDetector → 进化调制

**修复**：`evolution_cycle()` 中：
- `Converged` → 增加 mutation rate（打破局部最优）
- `Diverged` → 减少 mutation rate（稳定种群）
- `Exploring` → 保持默认
- `Exploiting` → 略增 mutation

### M10: ConstitutionalFeedback 类型安全

**修复**：新增 `evaluate_action(&self, action: &AgentAction, tick: u64)` 方法，直接 pattern match 枚举而非字符串匹配

## 四、死代码清理（D1-D15）

| ID | 问题 | 修复 |
|----|------|------|
| D1 | `AgentMemory.by_agent()` 无调用 | 删除（R1 一起清理） |
| D2 | `AgentMemory.recent()` 仅测试用 | 删除 |
| D3 | MemoryStream 检索从未从 WorldSim 调用 | 接入 M1 决策环路 |
| D4 | GraphMemory 6/8 方法从未生产调用 | 接入 M5 |
| D5 | SpatialMemory 8/12 方法从未生产调用 | 接入 M6 |
| D6 | PlanningStack 6/10 方法从未生产调用 | 接入 M1 |
| D7 | ReflectionEngine.reflect() 从未调用 | 接入 M4 |
| D8 | LlmHooks 整个模块死代码 | F2 处理 |
| D9 | EmergenceDetector 整个模块死代码 | F4 处理 |
| D10 | BehaviorVm 整个模块死代码 | F1 处理 |
| D11 | `ExperienceSignal.novelty_exposure` 未读 | 在 drift() 中用其增加 curiosity/openness |
| D12 | `ActionBudget.risk_per_action()` 未读 | 在 decide_action() 中低 health 时避免高 risk action |
| D13 | `ConvergenceDetector.state()` 未读 | M9 处理 |
| D14 | `GlobalCoherence.diversity_index` 未用 | 接入 emotion 或 emergence |
| D15 | `SimAgent.skills` 从未填充 | 在 evolution 中根据 action history 填充 |

## 五、架构重构：决策环路统一

### 5.1 统一 decide_action() 管线

当前 `decide_action()` 是 60 行 if-else 硬编码。重构为：

```rust
fn decide_action(&mut self, agent_id: &str, obs: &AgentObservation) -> AgentAction {
    let agent = self.get_agent(agent_id);
    
    // 1. 人格调制（M2）
    let personality_bias = agent.personality.action_bias();
    
    // 2. 情绪调制（M7）
    let emotion_bias = self.emotion.dominant_emotion().map(|e| e.action_bias());
    
    // 3. 心智理论（M3）
    let social_bias = self.theory_of_mind.action_bias(agent_id, &obs.nearby_agents);
    
    // 4. 目标规划（M1）
    let planned = self.planning.get(agent_id).next_action();
    
    // 5. 探索信号（M8）
    let explore_signal = self.action_awareness.get(agent_id).map(|a| a.should_explore());
    
    // 6. 成本/宪法约束
    let candidates = self.generate_candidates(obs, personality_bias, emotion_bias, social_bias, planned, explore_signal);
    candidates.into_iter()
        .filter(|a| self.action_costs.can_afford(a, agent.energy, agent.health))
        .filter(|a| self.constitutional.evaluate_action(a, self.tick).0 > 0.3)
        .max_by(|a| self.score_action(a, agent_id))
        .unwrap_or(AgentAction::Rest)
}
```

### 5.2 统一记忆管线

```
行动 → MemoryStream.add() + GraphMemory.add_node() + SpatialMemory.visit()
     → ReflectionEngine.on_new_memory() → 触发时 reflect() → MemoryStream.add(Reflection)
     → decide_action() 时 GraphMemory.spread_activation() + MemoryStream.retrieve()
```

## 六、执行顺序（推荐）

### Phase 1: 冗余清理 + 核心接线（3 天）
1. R1: 删除 AgentMemory，统一到 MemoryStream
2. M1: PlanningStack → decide_action
3. M2: Personality → decide_action
4. M3: TheoryOfMind → decide_action

### Phase 2: 记忆闭环（2 天）
5. M4: ReflectionEngine → MemoryStream
6. M5: GraphMemory 写入 + 查询
7. M6: SpatialMemory 写入 + 查询
8. D3/D4/D5/D6/D7: 死代码随接线自动激活

### Phase 3: 涌现 + 进化（2 天）
9. F4: EmergenceDetector 接入
10. M9: ConvergenceDetector → 进化调制
11. M8: ActionAwareness → 探索信号
12. M10: ConstitutionalFeedback 类型安全

### Phase 4: 架构统一（1 天）
13. 5.1: 统一 decide_action() 管线
14. 5.2: 统一记忆管线
15. F1: BehaviorVm 转为 AgentAction 编排层
16. F3: DualRepresentation 接入
17. R2: cosine_sim 提取共享

### Phase 5: 清理 + 验证（1 天）
18. D8/D9/D10: 移除或标注未激活模块
19. D11-D15: 小修复
20. 全量测试：cargo test + cargo clippy

## 七、外部模型架构对齐矩阵

| NeoTrix 模块 | Generative Agents | Project Sid PIANO | CivSim | JaxLife | OpenLife |
|--------------|-------------------|-------------------|--------|---------|----------|
| MemoryStream | ✅ Memory Stream | — | — | — | ✅ SDP |
| PlanningStack | ✅ Planning | — | — | — | — |
| ReflectionEngine | ✅ Reflection | — | — | — | — |
| GraphMemory | — | — | — | — | ✅ SDP |
| SpatialMemory | ✅ Environment Tree | — | — | — | — |
| PersonalityDrift | — | — | — | ✅ Evolution | ✅ Individuation |
| TheoryOfMind | — | ✅ Social Awareness | — | — | — |
| EmotionEngine | — | ✅ Emotion Module | — | — | — |
| ActionAwareness | — | ✅ Action Awareness | — | — | — |
| EmergenceDetector | — | — | ✅ Pattern Detector | — | — |
| ConvergenceDetector | — | — | — | ✅ Fitness | — |
| ConstitutionalFeedback | — | ✅ Constitutional | — | — | — |
| BehaviorVm | — | — | — | ✅ Programmable | — |
| ActionCostTable | — | — | ✅ Needs/Maslow | ✅ Energy | ✅ Metabolism |
| DualRepresentation | — | — | — | — | ✅ Distillate |
