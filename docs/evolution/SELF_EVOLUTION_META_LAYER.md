# NeoTrix 意识自进化元层架构 v1

> 闭环 5 条断裂反馈回路，让 NeoTrix 从自身经验中学习并改进自己

## 核心发现：5 条断裂回路

```
CalibrationEngine ──ECE/meta-d──✗──→ MetaCognitiveLoop    [回路1]
LossFunction ──────composite───✗──→ SelfModifyAgent       [回路2]
MetaCognitiveLoop ──plans───────✗──→ SelfEvolutionLoop    [回路3]
SelfModifyGuard ────4层门控──────✗──→ 全部nop              [回路4]
ConsciousnessCycle ─12步────────✗──→ 全部stub              [回路5]
```

## 架构设计

### SelfEvolutionMetaLayer

统一所有子循环的协调器。位于 `CoreNtCoreSelf` 顶层。

```
┌──────────────────────────────────────────────────────────────┐
│                    SelfEvolutionMetaLayer                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐     │
│  │ FeedbackBridge│ │ LoopCoordinator│ │ GuardActivator   │     │
│  │ 回路1-3闭合 │  │ 决定运行哪个 │  │ 填充门控层      │     │
│  └──────┬──────┘  └──────┬───────┘  └────────┬─────────┘     │
│         │                │                    │               │
│         ▼                ▼                    ▼               │
│  ┌─────────────────────────────────────────────────────┐     │
│  │               Metadata / Event Bus                    │     │
│  └─────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────┘
         │                 │                   │
         ▼                 ▼                   ▼
  Consciousness     MetaCognitive       SelfModifyAgent
  Pipeline           Loop               + SelfEvolutionLoop
```

### 回路1: Calibration → MetaCognition

```rust
// FeedbackBridge 每 cycle 调用:
fn bridge_calibration_to_meta(
    calibration: &CalibrationEngine,
    meta: &mut MetaCognitiveLoop,
) {
    let stats = calibration.stats();
    // 将真实 ECE 注入 meta_accuracy（替代原来的合成值）
    meta.record_meta_accuracy(ECE_target, 1.0 - stats.ece);
    // 将 meta-d' 注入 meta_knowledge
    meta.meta_knowledge.record_calibration_meta_d(stats.meta_d);
}
```

### 回路2: Loss → SelfModify

```rust
fn bridge_loss_to_self_modify(
    loss: &LossFunction,
    agent: &mut SelfModifyAgent,
    pipeline: &ConsciousnessPipeline,
) {
    let composite = loss.compute();
    // 如果总损失超过阈值，生成自修改提案
    if composite.total > LOSS_TRIGGER_THRESHOLD { // 0.35
        let proposal = SelfModifyProposal {
            target: ModifyTarget::PipelineStage { phase: "all" },
            source_code: generate_pipeline_reconfig(pipeline, &composite),
            rationale: format!("Composite loss {:.3} exceeds threshold", composite.total),
            expected_impact: -composite.total * 0.5,
        };
        agent.enqueue(proposal);
    }
    // 各维度损失影响不同目标
    if composite.calibration_loss > 0.4 {
        // 校准损失高 → 修改校准器参数
    }
    if composite.prediction_error > 0.5 {
        // 预测误差高 → 修改世界模型
    }
}
```

### 回路3: MetaCognition → SelfEvolution

```rust
fn bridge_meta_to_evolution(
    meta_result: &MetaCycleResult,
    evolution: &mut SelfEvolutionLoop,
) {
    for plan in &meta_result.plans {
        let mutation = evolution.planner.create_mutation(
            MutationTarget::Architecture,
            plan.description.clone(),
            plan.expected_impact,
        );
        evolution.mutate(mutation);
    }
}
```

### 回路4: GuardActivator

```rust
fn activate_guards() -> SelfModifyGuard {
    SelfModifyGuard {
        shield: Some(Box::new(|target: &ModifyTarget| {
            // 禁止修改核心身份字段
            !matches!(target, ModifyTarget::Parameter { path } if path.starts_with("identity."))
        })),
        swords: Some(Box::new(|source: &str| {
            // 扫描危险代码模式
            let dangerous = ["std::process::Command", "std::os::raw", "ptr::null"];
            !dangerous.iter().any(|d| source.contains(d))
        })),
        llm_validator: None, // 可选：LLM 验证打分
        ball_verifier: Some(Box::new(|_proposal: &SelfModifyProposal| {
            // 约束满足检查
            true // 简化版
        })),
    }
}
```

### 回路5: ConsciousnessCycle 实现

将 12 步存根转化为真实处理，连接 ConsciousnessIntegration 的 batch processing:

```rust
impl ConsciousnessCycle {
    pub fn run_cycle(&mut self, input: Option<VsaTagged>) -> CycleResult {
        let start = Instant::now();
        let mut steps = Vec::new();
        let mut state = input;

        // 1. Gather — 收集感知输入
        if self.config.enable_gather {
            // 从 sensory buffer 收集数据
            steps.push(self.step_gather(&mut state));
        }
        // 2. Gate — 认知负荷门控
        if self.config.enable_gate {
            steps.push(self.step_gate(&state));
        }
        // 3. Propose — 生成假设
        // 4. Compete — 注意力竞争
        // ... 等 12 步
        // 12. Sleep — 巩固

        CycleResult {
            cycle_num: self.cycle_num,
            steps_completed: steps.iter().map(|s| s.step).collect(),
            overall_success: steps.iter().all(|s| s.success),
            total_duration_ms: start.elapsed().as_millis() as u64,
            output_state: state,
            c_score: 0.5, // 实际计算
            steps_executed: steps.iter().map(|s| format!("{:?}", s.step)).collect(),
        }
    }
}
```

## 数据结构

```rust
pub struct SelfEvolutionMetaLayer {
    // 拥有的子组件
    pub feedback_bridge: FeedbackBridge,
    pub loop_coordinator: LoopCoordinator,
    pub guard_activator: GuardActivator,

    // 引用到外部组件的桥
    pub calibration: Option<*mut CalibrationEngine>,
    pub loss: Option<*mut LossFunction>,
    pub meta_loop: Option<*mut MetaCognitiveLoop>,
    pub self_modify: Option<*mut SelfModifyAgent>,
    pub evolution: Option<*mut SelfEvolutionLoop>,
    pub neuromodulator: Option<*mut NeuromodulatorEngine>,
    pub pipeline: Option<*mut ConsciousnessPipeline>,

    // 运行时状态
    pub last_trigger_cycle: u64,
    pub last_evolution_time: Instant,
    pub evolution_count: u64,
    pub meta_accuracy_history: VecDeque<f64>,
    pub loss_history: VecDeque<f64>,
    pub intervention_log: VecDeque<InterventionRecord>,
}

pub struct InterventionRecord {
    pub cycle: u64,
    pub trigger: InterventionTrigger,  // MetaPlan | LossSpike | Manual
    pub target: String,
    pub success: bool,
    pub impact: f64,
    pub duration_ms: u64,
}
```

## 集成到 ConsciousnessIntegration

```rust
impl ConsciousnessIntegration {
    pub fn tick_meta_layer(&mut self) {
        let Some(ref mut meta_layer) = self.meta_layer else { return };

        // 步骤1: 闭合回路1 — 校准 → 元认知
        if let (Some(cal), Some(meta)) = (self.calibration.as_ref(), self.meta_cognition_loop.as_mut()) {
            meta_layer.feedback_bridge.bridge_calibration_to_meta(cal, meta);
        }

        // 步骤2: 闭合回路2 — 损失 → 自修改
        if let (Some(loss), Some(agent)) = (self.composite_loss.as_ref(), self.self_modify_agent.as_mut()) {
            if let Some(pipeline) = self.pipeline.as_ref() {
                meta_layer.feedback_bridge.bridge_loss_to_self_modify(loss, agent, pipeline);
            }
        }

        // 步骤3: 闭合回路3 — 元认知计划 → SEAL
        if let (Some(meta), Some(evolution)) = (self.meta_cognition_loop.as_mut(), self.evolution_loop.as_mut()) {
            let meta_result = meta.run_cycle();
            meta_layer.feedback_bridge.bridge_meta_to_evolution(&meta_result, evolution);
        }

        // 步骤4: 协调子循环
        meta_layer.loop_coordinator.tick(
            self.neuromodulator.as_ref(),
            self.composite_loss.as_ref(),
        );

        // 记录干预历史
        meta_layer.record_cycle();
    }
}
```

## 实施路线

| 步骤 | 描述 | 估计行数 | 测试 |
|------|------|----------|------|
| 1 | `SelfEvolutionMetaLayer` 结构体 + 构造 | 80 | 3 |
| 2 | `FeedbackBridge` 回路1-3 | 150 | 6 |
| 3 | `LoopCoordinator` 子循环调度 | 120 | 5 |
| 4 | `GuardActivator` 门控4层填充 | 100 | 4 |
| 5 | ConsciousnessCycle 12步实现 | 400 | 12 |
| 6 | 集成到 ConsciousnessIntegration.tick_meta_layer() | 80 | 3 |
| 7 | 测试闭环验证 | 200 | 8 |
| **总计** | | **~1130** | **41** |
