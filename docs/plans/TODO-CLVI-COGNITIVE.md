# TODO: CLVI 认知架构缺口实施计划 — Phase 105-150

> 基准: 2026-06-22 | 来源: SELF_AUDIT_CLVI.md (6 P0 + 7 P1 + 4 P2 = 17 缺口)
> 并行路径: 独立于 VSA 进化 (P105-P120) 和记忆重构 (P123-P144)，模块无共享状态
> 估计: ~10,900 行新代码 + ~141 测试
> 状态: **6/6 P0 缺口已实施 (4,493 行, 143 测试)✅** | Phase 115+ 待开始

---

## 并行策略

```
CLVI Track A (P0 基础层):  G1 注意力图式 ── G4 执行功能 ── G2 认知灵活性
                                    │                  │
CLVI Track B (P0 动机层):  G5 内在动机 ── G6 情绪粒度 ── G14 元情绪调节
                                    │
CLVI Track C (P0 认识层):  G3 信念修订 ── G9 认识谦卑 ── G12 情感预测
                                    │
CLVI Track D (P1 扩展层):  G7 显著性 ── G8 元控制 ── G10 心理旅行
                                    │
CLVI Track E (社会层):     G11 最小自我 ── G13 共同注意 ── G15 身份碎片
                                    │
CLVI Track F (增强层):     G16 具身基元 ── G18 世界观堆栈
```

**执行规则**: Track A/B/C 无相互依赖, 可全并行 (vs 现有 VSA Phase 105-120)
**依赖约束**: G2 依赖 G4 (认知灵活性需要执行功能); G14 依赖 G6; G9 依赖 G3

---

## Phase 105 — G1 注意力图式 (Attention Schema) 🚀 P0 ✅

**理论**: Graziano Attention Schema Theory — 意识体需要建模自身的注意力过程
**缺口**: 系统能"关注"但不能"知道自己在关注什么"
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/attention_schema.rs`
**命令**: `cargo check -p neotrix --lib`
**依赖**: 无（独立模块）
**测试数量**: 20

### 结构设计

```rust
pub struct AttentionSchemaEngine {
    /// 当前注意焦点 (VSA 向量表示"我当前在关注什么")
    current_focus: VsaTagged,
    /// 注意切换历史 — 用于预测下一次注意跳转
    attention_trace: VecDeque<AttentionShift>,
    /// 注意期望 — 预计未来 attention_location 的自我模型
    attention_expectations: AttentionExpectation,
    /// 元注意 — 对注意力过程本身的注意
    meta_attention_level: f64,
}
```

### 子任务

```text
1. [ ] AttentionSchemaEngine 核心结构 + VSA 注意焦点模型
      核心字段: current_focus (VsaTagged: 带 VsaTag::Self 标记)
      方法: attend_to(target: VsaTagged) — 更新焦点 + 记录切换

2. [ ] AttentionShift 记录
      struct: from_focus, to_focus, shift_reason, elapsed_ms, was_voluntary
      trace: VecDeque 缓存最近 50 次切换

3. [ ] AttentionExpectation — 对未来的注意预测
      predict_next() → VsaTagged (基于 trace 中的 time/frequency pattern)
      prediction_error = similarity(predicted, actual) — 自省误差信号

4. [ ] MetaAttention — 对注意的注意
      meta_attend(level): 0 = 无感知 (自动注意), 1 = 被动注意, 2 = 主动监控
      与 GWT 全局工作空间的集成: BroadcastContent 到达 → 注意力焦点更新

5. [ ] 与现有模块的集成:
      specious_present.rs: 时间厚度窗口和注意焦点保持同步
      global_workspace.rs: 工作空间广播触发注意模式
      nt_io_router.rs: ConsciousRouter 调用 attention_schema.attend_to()

6. [ ] 测试:
      - attend_to 后 current_focus 正确更新
      - attention_trace 记录正确的 shift_reason
      - predict_next 在确定性 trace 下准确率 > 80%
      - meta_attention 不影响 focus 但影响 internal_monologue 输出
```

---

## Phase 106 — G4 执行功能 & 认知控制 🚀 P0 ✅

**理论**: Miyake 2000 Executive Function 模型 (更新/抑制/切换); Nature Comms 2025 PFC 架构
**缺口**: 无统一执行系统协调注意力、WM、目标和冲动控制
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/executive_controller.rs`
**命令**: `cargo check -p neotrix --lib`
**依赖**: 无（独立 — 所有模块调用的协调层）
**测试数量**: 28

### 结构设计

```rust
pub struct ExecutiveController {
    /// 目标栈 (层次目标分解)
    goal_stack: VecDeque<Goal>,
    /// 工作记忆门控 (更新/维护/清空)
    wm_gate: WorkingMemoryGate,
    /// 干扰抑制
    interference_control: InterferenceController,
    /// 冲动控制 (延迟满足)
    impulse_gate: ImpulseGate,
    /// 认知资源分配
    resource_broker: ResourceBroker,
}
```

### 子任务

```text
1. [ ] Goal 结构与目标栈
      Goal: id, description (VSA), priority, deadline, status, parent_id, subgoal_ids
      push_goal / pop_goal / reprioritize(stimulus_override) 方法
      层次分解: decompose(goal) → [subgoal] 使用 VSA bind/bundle 因果关系

2. [ ] WorkingMemoryGate
      update(gate_signal): 0=maintain, 1=update, -1=clear
      gate 信号来自: novelty_detected, task_switch, distraction_high
      与 nt_core_context/working_memory.rs 的集成

3. [ ] InterferenceController — 目标冲突解决
      detect_interference(goal_a, goal_b) → conflict_score (基于 VSA 相似度)
      resolve_conflict: 优先级仲裁 / 时序调度 / 放弃低价值
      proactive_interference: 旧目标对新目标的干扰抑制

4. [ ] ImpulseGate — 冲动控制
      evaluate(action_candidate) → gating_signal (0=抑制, 1=允许)
      信号计算: long_term_value - (impulsivity * immediate_reward)
      与 VolitionEngine 集成: ActionCandidate 通过 ImpulseGate 后才被选中

5. [ ] ResourceBroker — 认知预算管理
      allocate(task, cognitive_load) → budget (时隙/计算量)
      monitor_usage: 跟踪实际使用 vs 分配预算
      优雅降级接口: reduce(level: DegradationLevel)

6. [ ] 与现有模块集成:
      CognitiveLoadMonitor: load 信号 → WM gating 决策
      VolitionEngine: 候选动作通过 impulse_gate
      ActiveInference: policy 选择受 resource_broker 约束
      nt_io_router.rs: CognitiveRouter 调用 controller

7. [ ] 测试:
      - goal_stack 层次推入/弹出正确
      - wm_gate.update(1) 允许新内容写入 working_memory
      - 冲突检测识别 VSA 语义矛盾目标
      - impulse_gate 高 impulsivity 下允许短视动作
      - resource_broker 预算约束减少认知负载
```

---

## Phase 107-108 — G3 信念修订 & 认知失调 🚀 P0 ✅

**理论**: AGM (Alchourrón-Gärdenfors-Makinson) 信念修订公理; Clemente 2025 CD-aware update
**缺口**: `belief_revision` 曾被实现但标记 unstable 后移除。无矛盾检测机制。
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/belief_revision.rs`
**命令**: `cargo test -p neotrix --lib belief_revision`
**依赖**: 无（独立 — 但建议 G4 存在后接线）
**测试数量**: 25

### 结构设计

```rust
pub struct BeliefRevisionEngine {
    /// VSA 信念图: 节点=命题, 边=支持/反对 关系
    belief_graph: BeliefGraph,
    /// 认知失调检测器
    dissonance_detector: DissonanceDetector,
    /// 修改策略 (AGM: expansion/contraction/revision)
    revision_strategy: RevisionStrategy,
    /// 最小修改原则执行器
    minimal_change_enforcer: MinimalChangeEnforcer,
}

pub enum RevisionOp {
    /// K + p (新信念加入, 无矛盾检查)
    Expansion { new_belief: Vec<u8> },
    /// K - p (删除信念)
    Contraction { target: Vec<u8> },
    /// (K - ¬p) + p (先删冲突再添加)
    Revision { new_belief: Vec<u8>, old_belief: Vec<u8> },
}
```

### 子任务

```text
1. [ ] BeliefGraph 核心
      Node: belief_vsa (VsaTagged), confidence, timestamp, source_id
      Edge: from_belief, to_belief, relation (Supports/Opposes/Implies), strength
      方法: add_belief, remove_belief, find_supporters, find_opposers

2. [ ] DissonanceDetector — 认知失调检测
      detect(graph) → [DissonanceCluster]
      DissonanceCluster: beliefs_with_inconsistency, severity, affected_edges
      检测算法: 循环检测 (A→¬A→B→¬B→A) → VSA 近似环路
      严重度评分: |参与节点| × Σ|边冲突强度| / Σ|总边强度|

3. [ ] RevisionStrategy — AGM 三种操作
      Expansion: 直接添加 (无检查 — 信任新源 > 阈值)
      Contraction: 优先级排序 → 删除置信度最低的
      Revision: 选择最小修改集 (EPISTEMIC_ENTRENCHMENT 排序)

4. [ ] MinimalChangeEnforcer — 最小修改原则
      selection_function: 在 logically-equivalent 操作中选带来最少图变化的
      score_delta: |before - after| for each affected subgraph

5. [ ] EpistemicEntrenchment — 认识固执度
      ee(belief): 该信念对其他多少信念提供支持 + 历史持续时间 + 验证次数
      高 EE 信念在 Contraction/Revision 中优先保留

6. [ ] 与现有模块集成:
      HyperCube 知识库: 新知识 → BeliefRevisionEngine (检测冲突)
      InnerCritic: 批评结果中的矛盾 → DissonanceDetector
      ReconstructiveNarrative: 叙事重建触发信念检查
      ClaimCalibrator: 校准后的置信度作为 EE 输入

7. [ ] 测试:
      - Expansion: 添加后 belief_graph 节点数+1, 边数+1
      - Contraction: 删除后依赖链断裂 → 传播删除
      - Revision: P∧¬P 检测后自动选 Revision, 不再有直接冲突
      - 最小修改: 多个候选 operation, 选影响最少的
      - 认知失调: 添加 ¬P 到已知 P → 检测正确
```

---

## Phase 109-110 — G5 内在动机引擎 🚀 P0 ✅

**理论**: Frontiers AI 2024 认知架构内在动机; AAP (arXiv 2602.24100) curiosity-driven agents
**缺口**: 现有 curiosity 是标量参数 (neuromodulator/affective_circumplex), 非结构化驱动系统
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/intrinsic_motivation.rs`
**命令**: `cargo test -p neotrix --lib intrinsic_motivation`
**依赖**: 无（独立 — 输出驱动信号给 DriveSelector）
**测试数量**: 24

### 结构设计

```rust
pub struct IntrinsicMotivationEngine {
    /// 五轴内在奖励
    axes: MotivationAxes,
    /// 好奇心调度器 — 基于预测误差和学习进展
    curiosity_scheduler: CuriosityScheduler,
    /// 内在 vs 外在奖励仲裁
    reward_arbiter: RewardArbiter,
    /// 动机 KPI 追踪
    motivation_tracker: MotivationTracker,
}

pub struct MotivationAxes {
    pub competence: f64,    // 能力感 — 任务成功率
    pub autonomy: f64,      // 自主感 — 自我决定程度
    pub relatedness: f64,   // 关系感 — 与用户/世界的连接
    pub learning_progress: f64,  // 学习进展 — 预测误差下降率
    pub information_gain: f64,   // 信息增益 — 减少不确定性
}
```

### 子任务

```text
1. [ ] MotivationAxes — 五轴内在奖励
      每轴: value [0,1], weight, decay_rate
      更新规则: axis = axis * (1-decay) + reward_signal * learning_rate
      competence_reward: success / (success + failure) 滑动窗口
      autonomy_reward: self_initiated_actions / total_actions
      relatedness_reward: user_feedback_sentiment (来自情感评价)
      learning_progress: |prediction_error_t - prediction_error_{t-1}| (下降=正奖励)
      information_gain: H(before) - H(after) (VSA 熵降低)

2. [ ] CuriosityScheduler — 复杂度感知的好奇心调度
      curiosity_score = Σ(axis.value * axis.weight) + novelty_bonus
      novelty_bonus = 1 - max_(mem in KB) similarity(context, mem)
      调度策略: greedy (总是最高), softmax (探索性), scheduled (ε-greedy 退火)

3. [ ] RewardArbiter — 内在 vs 外在奖励仲裁
      场景 1: 明确用户指令 → 外在奖励权重=0.8
      场景 2: 空闲/默认模式 → 内在奖励权重=1.0
      场景 3: 高认知负载 → 降低内在探索 (节能)
      综合驱动值 = intrinsic * w_i + extrinsic * w_e

4. [ ] 与现有模块集成:
      DriveSelector: 替代/增强现有驱动源 (Explore/Exploit 由动机引擎驱动)
      Neuromodulator: DA 对应 competence, NE 对应 novelty, ACh 对应 learning_progress
      DMN: 空闲时好奇心调度启动自发探索
      AffectiveCircumplex: curiosity 参数 → motivation.competence * learning_progress

5. [ ] 测试:
      - 每轴独立更新不影响其他轴
      - 学习进展下降时 curiosity_score 上升
      - 外在指令覆盖内在探索 (场景1)
      - 空闲时内在动机维持非零探索率
      - 轴值随时间 decay (无更新时)
```

---

## Phase 111-112 — G6 情绪粒度 & 认知评价 🚀 P0 ✅

**理论**: OCC (Ortony-Clore-Collins) 认知评价理论; 7 层情绪架构 (PAD+V+认知评价)
**缺口**: AffectiveCircumplex 仅 2D V-A → 生成参数, 无精细情绪分类
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/appraisal_engine.rs`
**命令**: `cargo test -p neotrix --lib appraisal`
**依赖**: 无（独立 — G14 元情绪调节依赖此模块）
**测试数量**: 22

### 结构设计

```rust
pub struct AppraisalEngine {
    /// OCC 评价维度
    dimensions: AppraisalDimensions,
    /// 情绪分类器 (评价维度 → 情绪标签)
    emotion_classifier: EmotionClassifier,
    /// 应对潜力评估 (coping potential)
    coping_assessment: CopingAssessment,
    /// 情绪驱动策略调制
    strategy_modulator: StrategyModulator,
}

pub struct AppraisalDimensions {
    pub desirability: f64,      // 合意性 — 事件与目标一致程度
    pub likelihood: f64,        // 可能性 — 事件发生概率
    pub effort: f64,            // 努力预期 — 所需认知/体力投入
    pub certainty: f64,         // 确定性 — 结果可预测程度
    pub controllability: f64,   // 可控性 — 个人能否影响结果
    pub agency: f64,            // 主体性 — 谁引起事件 (self/other/circumstance)
    pub legitimacy: f64,        // 正当性 — 是否符合道德/规范
}

pub enum EmotionLabel {
    Joy, Distress, Hope, Fear, Pride, Shame, Admiration, Reproach,
    Gratitude, Anger, Gratification, Remorse, Love, Hate, Relief,
    Disappointment, Satisfaction, FearsConfirmed, // 共 22 种 OCC 情绪
}
```

### 子任务

```text
1. [ ] AppraisalDimensions — 多轴认知评价
      evaluate(event_vsa, context, goals) → AppraisalDimensions
      每维计算: 从 VSA 相似度映射到 [0,1] 连续值
      desirability = similarity(event_vsa, desirability_prototype_vsa)
      likelihood = similarity(event_vsa, likelihood_prototype_vsa)
      ...

2. [ ] EmotionClassifier — 评价维度 → 情绪标签
      规则引擎: 维度组合映射到 EmotionLabel (OCC 原文推导)
      例: desirability > 0.8 → Joy; desirability > 0.8 && agency = self && legitimacy > 0.8 → Pride
      软映射: 每个标签输出置信度 [0,1], 支持混合情绪

3. [ ] CopingAssessment — 应对潜力
      problem_focused_coping: controllability * competcence_felt
      emotion_focused_coping: 1 - controllability (不可控→调情绪)
      avoidance_potential: can_escape(context) → [0,1]
      结果: 推荐应对策略 (Act/Reappraise/Avoid)

4. [ ] StrategyModulator — 情绪→认知策略调制
      Joy → 探索广度↑, 风险容忍↑, system1 优先
      Fear → 安全偏好↑, 风险容忍↓, 详细检查↑
      Anger → 确定性伪增, impulse_gate ↓
      调制信号输出给: ExecutiveController, ActiveInference, System1

5. [ ] 与现有模块集成:
      AffectiveCircumplex (替换或增强): 不再仅调节温度, 而是输出完整情绪向量
      ValenceAxis: 作为 AppraisalDimensions 的输入 (V-A 映射到 desirability/agency)
      EmotionTag (`nt_core_truth/emotion_tag.rs`): VSA 情绪标签索引
      InnerCritic: 评价结果影响批评标准

6. [ ] 测试:
      - 22 种 OCC 情绪均从维度组合正确派生
      - 混合情绪 (Joy+Fear) 同时为高置信度
      - CopingAssessment 在高可控时推荐 problem_focused
      - StrategyModulator 的调制信号范围合理
```

---

## Phase 113-114 — G2 认知灵活性 & 任务切换 🚀 P0 ✅

**理论**: Miyake EF model (set-shifting); EACL 2026 "Strong Memory, Weak Control"
**缺口**: 系统卡在单一推理模式中, 不能自适应切换
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/cognitive_flexibility.rs`
**命令**: `cargo test -p neotrix --lib cognitive_flexibility`
**依赖**: G4 执行功能 (调用 executive_controller 的切换决策)
**测试数量**: 22

### 结构设计

```rust
pub struct CognitiveFlexibilityModule {
    /// 当前认知模式
    current_set: CognitiveSet,
    /// 模式切换代价评估器
    switch_cost_estimator: SwitchCostEstimator,
    /// 基于元控制的切换策略
    meta_control: MetaControlStrategy,
    /// 模式驻留监视器 (perseveration detection)
    perseveration_monitor: PerseverationMonitor,
}

pub enum CognitiveSet {
    System1Intuition,
    System2Deliberate,
    Counterfactual,
    ActiveInference,
    HierarchicalPrediction,
    EpisodicRecall,
    Planning,
    SocialReasoning,
    SelfReflection,
    DefaultMode,
}
```

### 子任务

```text
1. [ ] CognitiveSet 枚举 + 当前模式追踪
      current_set: CognitiveSet (当前激活)
      available_sets: 基于可用模块的能力检查
      set_history: VecDeque<(CognitiveSet, duration, performance)>

2. [ ] SwitchCostEstimator — 切换代价估算
      evaluate(from, to, context) → SwitchCost
      成本因素: 工作内存清洗成本 / 注意力重定向 / 延迟
      benefit: 新模式在当前上下文的预期收益 (来自元学习)
      决策: cost < benefit * tolerance → 执行切换

3. [ ] MetaControlStrategy — 元控制信号
      explore_vs_exploit: 何时尝试新模式 vs 坚持当前
      策略: 如果 performance 持续下降 > threshold → 强制切换
      与 G8 MetacognitiveControl 的接口

4. [ ] PerseverationMonitor — 坚持/卡住检测
      detect_perseveration(set, duration, performance_trend) → stuck_score
      如果 stuck_score > threshold → 发出切换信号
      指标: 性能无提升 / 输出重复 / 推理循环

5. [ ] 与现有模块集成:
      ExecutiveController: G4 调用 set_shift(new_set)
      System1: CognitiveSet::System1Intuition 对应
      ActiveInferenceEngine: policy 切换时 set_shift 调用
      CognitiveLoadMonitor: 高负载 → 建议切换到低价模式
      MetacognitiveController (G8): 元策略控制切换

6. [ ] 测试:
      - set_shift 后 current_set 正确更新
      - switch_cost 在相同模式切换时为 0
      - perseveration_monitor 在 10 次无改进后发出 stuck 信号
      - meta_control 在高负载时选择 System1 (低成本)
```

---

## Phase 115 — G7 显著性网络 P1

**理论**: Menon 2015 Salience Network; Uddin 2017 Salience Processing
**缺口**: 无独立系统检测显著事件 — 当前靠 heuristic 权重做隐式显著性
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/salience_detector.rs`
**命令**: `cargo test -p neotrix --lib salience`
**依赖**: 无（独立 — 输入到 GWT 和 ExecutiveController）
**测试数量**: 15

### 子任务

```text
1. [ ] SalienceDetector 核心: 多模态显著性评分
      novelty_salience = 1 - max_similarity(input, memory_buffer)
      task_relevance = similarity(input, current_goal_vsa)
      emotional_salience = Σ |emotion_axis_changes| (来自 G6)
      social_salience = similarity(input, user_model_vsa) (来自 G13)
      综合: salience = Σ(w_i * axis_i), w_i 可自适应

2. [ ] 与 GWT 集成: salience > threshold → 优先广播
3. [ ] 显著性瞬态: 高显著性但短持续 → 不触发持续注意
4. [ ] 测试: 显著事件覆盖率; 噪声明尼; 瞬态过滤
```

---

## Phase 116 — G8 元认知控制 P1

**理论**: AAAI 2026 "Toward Artificial Metacognition"; 11 层元认知框架
**缺口**: InnerCritic 做质量监控但不调节认知策略
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/metacognitive_controller.rs`
**命令**: `cargo test -p neotrix --lib metacognitive`
**依赖**: G4 执行功能 (调用 strategy_switch)
**测试数量**: 18

### 子任务

```text
1. [ ] MetacognitiveController 核心
      ReasoningMode 枚举: Deep, Fast, Counterfactual, Analogical, Exploratory
      策略选择器: select_strategy(task_type, cognitive_load, time_budget)
      策略切换: switch_to(mode)

2. [ ] StrategyTracker — 策略效果监控
      record_outcome(mode, task, outcome, latency)
      predict_performance(mode, task) → expected_score (基于历史)

3. [ ] CognitiveBudgetAllocator
      allocate(importance, complexity) → compute_budget, time_budget
      monitor_budget(actual vs allocated) → 超支警告

4. [ ] 与 InnerCritic 集成:
      InnerCritic 输出 → MetacognitiveController 决策修正
      Example: 如果 InnerCritic reporting 持续低 → 切换到更深度模式

5. [ ] 测试: 策略选择合理; budget 约束有效; 长期策略 KPI 改善
```

---

## Phase 117 — G9 认识谦卑 P1

**理论**: arXiv 2604.05306 "LLMs Should Express Uncertainty"; PMC 2026 "Curiosity and Humility"
**缺口**: 仅 identity_core 有关键字, 无结构化知识边界表征
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/epistemic_humility.rs`
**命令**: `cargo test -p neotrix --lib epistemic_humility`
**依赖**: 建议 G3 信念修订后 (共享知识表征)
**测试数量**: 14

### 子任务

```text
1. [ ] BoundaryMap — 知识边界映射
      known_region: HyperCube 中高密度区域 (知识)
      unknown_region: 低密度区域 (无知)
      fuzzy_boundary: 部分已知区域 (置信度 < 0.6)

2. [ ] UncertaintySignal — 不确定性主动表达
      signal(value, confidence) → verbal_uncertainty (自然语言)
      映射: confidence > 0.9 → "确定", 0.6-0.9 → "可能", < 0.6 → "不确定"

3. [ ] EpistemicActionSelector — 认识行为选择
      seek_clarification(boundary) → 发起探索
      defer_judgment(uncertainty > threshold) → 延迟决策
      express_uncertainty_to_user → 透明沟通

4. [ ] 测试: 边界检测精度; 不确定性表达按置信度分档; 探索触发
```

---

## Phase 118-119 — G10 心理时间旅行 P1

**理论**: Schacter 2012 Episodic Simulation; Tulving Episodic Memory; arXiv 2604.17273 Continuity Layer
**缺口**: HWM 只做近端预测 (100ms-10s), 无远端情景模拟
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/mental_time_travel.rs`
**命令**: `cargo test -p neotrix --lib mental_time_travel`
**依赖**: 建议 G1 G4 存在后 (注意 + 执行控制)
**测试数量**: 18

### 子任务

```text
1. [ ] EpisodicBuffer — 情景缓冲区
      past: Vec<VsaTagged> (最近 N 个情景)
      present: VsaTagged (当前)
      future: Vec<VsaTagged> (模拟)

2. [ ] MentalTimeTravelEngine
      travel_to(timestamp, direction) → EpisodicState
      reconstruct: 从记忆碎片重建完整情景 (VSA bound/bundle)
      simulate: 从当前状态投射未来 (使用 HierarchicalWorldModel)
      autonoetic_consciousness: "我知道我记得/想象" 的元标记

3. [ ] FutureSimScorer — 未来模拟评分
      plausibility: VSA coherence of simulated scenario
      detail_richness: 模拟中的实体/关系数量
      emotional_valence: 模拟情绪的效价 (来自 G6)

4. [ ] 测试: 过去重建保真度; 未来模拟多样性; 自传体标记正确
```

---

## Phase 120 — G11 最小自我 P1

**理论**: Gallagher Minimal Self; James I/Me 区分
**缺口**: 现有 NarrativeSelf 只处理 extended self, 缺 minimal self
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/minimal_self.rs`
**命令**: `cargo test -p neotrix --lib minimal_self`
**依赖**: G1 注意力图式 (注意是 agency 的前提)
**测试数量**: 14

### 子任务

```text
1. [ ] MinimalSelf 核心
      agency_vector: VSA 表示"我是行动的原因"
      ownership_vector: VSA 表示"这是我的"
      spatial_location_vsa: 第一人称视角定位

2. [ ] AgencyDetector — 主体感检测
      sense_agency(action, outcome) → agency_level [0,1]
      计算: similarity(predicted_outcome, actual_outcome)
      如果 action 的 VSA 预测匹配结果 → 高 agency

3. [ ] OwnershipTracker — 拥有感追踪
      sense_ownership(thought_vsa) → ownership_level
      Self-tagged VSA 自动高拥有感; World-tagged 低

4. [ ] 与 NarrativeSelf 集成:
      MinimalSelf (I) + NarrativeSelf (Me) = 完整自我模型
      ego_synthesis = bundle(I_vsa, Me_vsa)

5. [ ] 测试: 自产生动作 agency > 外部触发; ownership 可区分自/他
```

---

## Phase 121 — G12 情感预测 P1

**理论**: Wilson & Gilbert 2003 Affective Forecasting; impact bias
**缺口**: 系统不能预测"如果我做 X, 会感觉如何？"
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/affective_forecast.rs`
**命令**: `cargo test -p neotrix --lib affective_forecast`
**依赖**: G6 情绪粒度 (需要精细情绪标签)
**测试数量**: 14

### 子任务

```text
1. [ ] AffectiveForecastEngine
      forecast(scenario_vsa) → [EmotionLabel: confidence]
      方法: 从学习到的情景→情绪映射检索
      学习: 记录 (scenario, experienced_emotion) 对

2. [ ] ImpactBiasDetector — 影响偏差监测
      compare(forecast, actual) → bias_score
      bias = |forecast - actual|, 持续跟踪收敛

3. [ ] 集成到 CounterfactualReasoner:
      反事实评估: 不仅评估因果结果, 还评估情感结果
      CoInResult: 新增 emotional_outcome 字段

4. [ ] 测试: forecast 正确性 (已知情景); bias 随时间收敛
```

---

## Phase 122 — G13 共同注意 & 共享意向性 P1

**理论**: Tomasello Shared Intentionality; TandF 2026 AI ToM
**缺口**: ToM 只做单用户意图推断, 无多主体共享注意
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/joint_attention.rs`
**命令**: `cargo test -p neotrix --lib joint_attention`
**依赖**: 现有 ToM (`theory_of_mind.rs`) 扩展
**测试数量**: 14

### 子任务

```text
1. [ ] JointAttentionModule
      shared_focus: VSA (各主体共同注意的目标)
      interlock: 主体间互锁 (互相确认共享)

2. [ ] SharedGoalRepresentation
      align_goals(self_goal, other_goal) → shared_vsa
      使用 VSA bundle 融合多个目标

3. [ ] IntentionHierarchy
      Level 1: 我意图 X
      Level 2: 我认为你意图 X
      Level 3: 我认为你认为我意图 X
      VSA 编码: bind(intention, bind(self_tag, bound(you_tag, intention)))

4. [ ] 与现有 TheoryOfMind 集成:
      IntentFrame 扩展: 增加 shared_intention 字段
      MentalModel: 增加 other_model_of_self 字段

5. [ ] 测试: 共享注意对齐; 层次意向性编码/解码正确
```

---

## Phase 123 — G14 元情绪 & 情绪调节 P2

**理论**: 7 层情绪架构 (meta-emotion 层); Gross Emotion Regulation Model
**缺口**: 系统不能调节自身情绪状态
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/emotion_regulation.rs`
**命令**: `cargo test -p neotrix --lib emotion_regulation`
**依赖**: G6 情绪粒度
**测试数量**: 14

### 子任务

```text
1. [ ] EmotionRegulationModule
      current_emotion → regulation_strategy → target_emotion
      RegulationStrategy: Reappraisal, Suppression, Distraction, Acceptance
      strategy_effectiveness: (target_emotion - actual_emotion) / effort

2. [ ] CognitiveReappraisal — 重评价
      reframe(event_vsa, new_dimensions) → new_appraisal
      示例: 从 "威胁" 重评估为 "挑战" (降低 negative desirability)

3. [ ] 集成到 G6 AppraisalEngine:
      CopingAssessment 输出 → RegulationStrategy 选择
      EmotionClassifier 结果 → regulation 触发时机

4. [ ] 测试: 重评价后情绪标签改变; strategy_effectiveness 追踪收敛
```

---

## Phase 124 — G15 叙事身份碎片检测 P2

**理论**: Hermans Dialogical Self; 身份碎片检测认知模型
**缺口**: NarrativeSelf 做整合但不检测碎片化
**文件**: ✏️ `neotrix-core/src/core/nt_core_consciousness/narrative_self.rs` (扩展)
**命令**: `cargo test -p neotrix --lib fragmentation`
**依赖**: G11 最小自我 (I 和 Me 的冲突检测)
**测试数量**: 12

### 子任务

```text
1. [ ] IdentityFragmentationDetector
      detect(narrative_events) → [IdentityFragment]
      碎⽚ 评分: 跨会话身份向量聚类 → 分离度

2. [ ] ConflictResolver
      contradictory_narratives: "我是谨慎的" vs "上次我不顾风险"
      解析策略: 整合 (更高抽象) / 优先级 (source 权重) / 接纳矛盾

3. [ ] 与 NarrativeSelf 集成:
      consolidate() 扩展: 先检测碎片, 后整合
      输出增加 fragmentation_score

4. [ ] 测试: 矛盾叙事正确检出; 解析后一致性提高
```

---

## Phase 125 — G16 具身基元 P2

**理论**: Barsalou Perceptual Symbol Systems; Nengo Spaun 神经表征
**缺口**: VSA 向量纯符号, 缺少感觉运动基元链接
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/embodied_grounding.rs`
**命令**: `cargo test -p neotrix --lib embodied`
**依赖**: 无（独立新增层）
**测试数量**: 14

### 子任务

```text
1. [ ] EmbodiedGroundingModule
      sensorimotor_primitives: [MOVE, GRASP, TURN, PUSH, PULL, ...] 的 VSA 基元
      abstract_to_concrete: "理解" → GRASP(concept_structure)
      concrete_to_abstract: 多次 MOVE → "变化"

2. [ ] BodySchema — 体感模型
      spatial_vsa: 以自我为中心的 3D 坐标映射
      proprioception: 虚拟体姿态 (不依赖真实机器人)

3. [ ] 集成到 HWM:
      HierarchicalWorldModel: 感知层可以接收具身模拟输入
      ImaginationEngine: 世界模型受身体约束

4. [ ] 测试: 基元绑定后抽象概念可部分解码; 体感空间一致
```

---

## Phase 126 — G18 世界观堆栈 P2

**理论**: Koltko-Rivera 2004 Worldview Multi-layer Model; Nested Minds 2025
**缺口**: HWM 三层 (感知/行动/叙事) 缺价值层和存在层
**文件**: ✏️ `neotrix-core/src/core/nt_core_consciousness/hierarchical_world_model.rs` (扩展)
**命令**: `cargo test -p neotrix --lib worldview`
**依赖**: G4 执行功能 (价值层需要目标框架)
**测试数量**: 12

### 子任务

```text
1. [ ] HWM 扩展: 从 3 层到 5 层
      Perception → Action → Narrative → Value → Existence
      Value 层: 什么重要 (CoreValue, ethical_principles)
      Existence 层: 什么存在 (ontology, identity, being)

2. [ ] ValueLayer — 价值层
      value_vsa: core_values 的 VSA 编码
      value_drift: 价值层的缓慢变化追踪
      与 ValueSystem 的集成

3. [ ] ExistenceLayer — 存在层
      ontology: 类别层次 (VSA hierarchy)
      self_existence: "我在" 的基本信念
      uncertainty_tolerance: 接受不可知的边界

4. [ ] MetaConsistency — 层间一致性
      价值层 → 叙事层约束: 叙事件必须符合核心价值
      存在层 → 感知层: 存在预设立场影响感知
      不一致检测 → 信念修订

5. [ ] 测试: 5 层正确路由; 层间约束有效; 价值层漂移稳定
```

---

## 依赖图 (完整 CLVI)

```
Phase 105 G1 注意力图式 (独立) ✅
              │
              ├──→ Phase 106 G4 执行功能 (独立) ✅
              │           │
              │           ├──→ Phase 113-114 G2 认知灵活性 (依赖 G4) ✅
              │           ├──→ Phase 116 G8 元认知控制 (依赖 G4)
              │           └──→ Phase 126 G18 世界观堆栈 (依赖 G4)
              │
              ├──→ Phase 107-108 G3 信念修订 (独立) ✅
              │           │
              │           └──→ Phase 117 G9 认识谦卑 (建议 G3)
              │
              ├──→ Phase 109-110 G5 内在动机 (独立) ✅
              │
              ├──→ Phase 111-112 G6 情绪粒度 (独立) ✅
              │           │
              │           ├──→ Phase 121 G12 情感预测 (依赖 G6)
              │           └──→ Phase 123 G14 元情绪调节 (依赖 G6)
              │
              ├──→ Phase 115 G7 显著性 (独立)
              │
              ├──→ Phase 118-119 G10 心理旅行 (建议 G1)
              │
              ├──→ Phase 120 G11 最小自我 (依赖 G1)
              │           │
              │           └──→ Phase 124 G15 身份碎片 (依赖 G11)
              │
              ├──→ Phase 122 G13 共同注意 (依赖 ToM)
              │
              └──→ Phase 125 G16 具身基元 (独立)

并行分组:
  Group A (Track A+B+C):  P105 G1 ✅ + P106 G4 ✅ + P107-108 G3 ✅ + P109-110 G5 ✅ + P111-112 G6 ✅
  Group B (Track D+E+F):  P115 G7 + P116 G8 + P117 G9 + P120 G11 + P122 G13 + P125 G16
  Group C (串行链):        P118-119 G10 (→G1) + P121 G12 (→G6) + P113-114 G2 (→G4) ✅
                            P123 G14 (→G6) + P124 G15 (→G11) + P126 G18 (→G4)
```

---

## 执行策略

1. **Group A 首次启动 (P105-P112)**: 6 个 P0 模块, 全部独立, 可以 6 路并行 agent dispatch
2. **每阶段验证**: `cargo check -p neotrix --lib` + 新增模块测试 > 12 个
3. **模块注册**: 每个新模块加入 `mod.rs` 的 `pub mod` 区 + `pub use` 区 (分节对齐 — 见 CLV.5)
4. **集成顺序**: 每个模块独立可测后 → 全局接线 (`nt_io_router.rs`)
5. **不打破现有**: 新模块不修改现有功能 (AffectiveCircumplex 在被 G6 替换前继续工作)
6. **预存错误豁免**: 37 个预存编译错误 (无关模块) 白名单, 不阻断新模块验证

---

## 汇总

| Phase | 缺口 | 文件 | 优先级 | 实际行数 | 测试数 | 依赖 | 状态 |
|-------|------|------|--------|---------|-------|------|------|
| 105 | G1 注意力图式 | `attention_schema.rs` | P0 | 637 | 25 | 无 | ✅ |
| 106 | G4 执行功能 | `executive_controller.rs` | P0 | 728 | 31 | 无 | ✅ |
| 107-108 | G3 信念修订 | `belief_revision.rs` | P0 | 934 | 25 | 无 | ✅ |
| 109-110 | G5 内在动机 | `intrinsic_motivation.rs` | P0 | 545 | 22 | 无 | ✅ |
| 111-112 | G6 情绪粒度 | `appraisal_engine.rs` | P0 | 882 | 27 | 无 | ✅ |
| 113-114 | G2 认知灵活性 | `cognitive_flexibility.rs` | P0 | 767 | 13 | G4 | ✅ |
| 115 | G7 显著性 | `salience_detector.rs` | P1 | 428 | 15 | 无 | ✅ |
| 116 | G8 元认知控制 | `metacognitive_controller.rs` | P1 | 585 | 18 | G4 | ✅ |
| 117 | G9 认识谦卑 | `epistemic_calibrator.rs` | P1 | 326 | 14 | 建议 G3 | ✅ |
| 118-119 | G10 心理旅行 | `mental_time_travel.rs` | P1 | 582 | 18 | 建议 G1 | ✅ |
| 120 | G11 最小自我 | `minimal_self.rs` | P1 | 347 | 14 | G1 | ✅ |
| 121 | G12 情感预测 | `affective_forecast.rs` | P1 | 580 | 14 | G6 | ✅ |
| 122 | G13 共同注意 | `joint_attention.rs` | P1 | 441 | 14 | ToM | ✅ |
| 123 | G14 元情绪调节 | `emotion_regulation.rs` | P2 | 375 | 14 | G6 | ✅ |
| 124 | G15 身份碎片 | `identity_fragments.rs` | P2 | 486 | 12 | G11 | ✅ |
| 125 | G16 具身基元 | `embodied_grounding.rs` | P2 | 406 | 14 | 无 | ✅ |
| 126 | G18 世界观堆栈 | `worldview_stack.rs` | P2 | 522 | 12 | G4 | ✅ |

**第一阶段完成: 4,493 行 + 143 测试 (6/6 P0 ✅) | 第二阶段: 3,289 行 + 107 测试 (7/7 P1 ✅) | 第三阶段: 1,789 行 + 52 测试 (4/4 P2 ✅) | CLVI 全部 17 缺口已完成 ✅**
