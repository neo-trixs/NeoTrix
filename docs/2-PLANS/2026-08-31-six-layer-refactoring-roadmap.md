# 六层架构重构路线图

## 概览

| 阶段 | 目标 | 预估时间 | 风险等级 |
|------|------|----------|----------|
| Phase 0: 准备 | 创建接口契约、目录结构 | 1天 | 低 |
| Phase 1: L1 Action | 迁移工具/行动层 | 2-3天 | 低 |
| Phase 2: L2 Perception | 迁移感知层 | 2-3天 | 低 |
| Phase 3: L3 Embodiment | 迁移具身层 | 3-4天 | 中 |
| Phase 4: L4 Emotion | 迁移情感层 | 2-3天 | 低 |
| Phase 5: L5 Cognition | 迁移认知层 | 3-4天 | 中 |
| Phase 6: L6 Meta-Cognition | 迁移元认知层 | 2-3天 | 高 |
| Phase 7: 清理验证 | 删除旧结构、全测试 | 1-2天 | 低 |

**总计**: 16-23 天 (可并行压缩至 10-14 天)

---

## Phase 0: 准备工作 (Day 1)

### 0.1 创建六层目录结构
```
neotrix-core/src/
├── l1_action/           # L1 行动层
│   ├── nt_act/
│   ├── nt_io/
│   └── nt_memory/
├── l2_perception/       # L2 感知层
│   ├── nt_world/
│   └── nt_sense/
├── l3_embodiment/       # L3 具身层
│   ├── nt_physical/
│   ├── nt_shield/
│   └── nt_feel/         # 情感也在具身层
├── l4_emotion/          # L4 情感层 (未来独立)
│   └── nt_feel/
├── l5_cognition/        # L5 认知层
│   ├── nt_core/
│   └── nt_mind/
└── l6_meta/             # L6 元认知层 (未来)
    ├── nt_meta/
    ├── nt_repair/
    └── nt_nexus/
```

### 0.2 定义层间接口 (trait)
每层暴露 `LayerInterface` trait，下层实现，上层依赖。

### 0.3 建立绞杀器
旧代码通过 `LegacyAdapter` 适配到新层。

---

## Phase 1: L1 Action 层 (Day 2-4)

### 1.1 目标结构
```
l1_action/
├── nt_act/
│   ├── mod.rs
│   ├── action_cache.rs
│   ├── autonomy/
│   ├── code/
│   ├── crypto.rs
│   ├── disk_guard.rs
│   ├── goal/
│   ├── media.rs
│   ├── orchestrator/
│   ├── sandbox.rs
│   └── seo.rs
├── nt_io/
│   ├── mod.rs
│   ├── ai_image_prompts.rs
│   ├── cozyclay.rs
│   ├── eli5.rs
│   ├── excalidraw.rs
│   ├── generative_media_skills.rs
│   ├── hermes_community.rs
│   ├── hermes_quota.rs
│   ├── pi_agent_desktop.rs
│   ├── promo_bgm.rs
│   └── web/
└── nt_memory/
    ├── mod.rs
    ├── babeldoc.rs
    ├── graphify.rs
    ├── historian/
    ├── kb/
    ├── knowledge_graph/
    ├── leann_store.rs
    ├── pdf_math_translate.rs
    ├── spatial/
    └── yopedia.rs
```

### 1.2 迁移步骤
1. 创建目录和 `mod.rs`
2. 移动文件 (保持内容不变)
3. 更新 `use` 路径
4. 验证编译通过

### 1.3 接口定义
```rust
// l1_action/trait.rs
pub trait ActionLayer {
    fn execute(&self, action: Action) -> Result<ActionResult>;
    fn get_tools(&self) -> Vec<Tool>;
}
```

---

## Phase 2: L2 Perception 层 (Day 5-7)

### 2.1 目标结构
```
l2_perception/
├── nt_world/
│   ├── mod.rs
│   ├── absorber/
│   ├── adsb.rs
│   ├── agent_reach.rs
│   ├── aoi.rs
│   ├── bgpview.rs
│   ├── browse/
│   ├── browse_auto/
│   ├── code_search.rs
│   ├── crawl/
│   └── sense/           # 感知桥接
└── nt_sense/
    ├── mod.rs
    ├── visual_cortex.rs
    ├── auditory_cortex.rs
    ├── sensory_hub.rs
    └── perception_bridge.rs
```

---

## Phase 3: L3 Embodiment 层 (Day 8-11)

### 3.1 目标结构
```
l3_embodiment/
├── nt_physical/
│   ├── mod.rs
│   ├── sense.rs
│   ├── motor.rs
│   ├── safety.rs
│   ├── power.rs
│   ├── body_schema.rs
│   ├── kinematics.rs
│   ├── calibration.rs
│   └── odometry.rs
├── nt_shield/
│   ├── mod.rs
│   ├── sandbox/
│   └── stealthnet/
└── nt_feel/             # 情感在具身层 (阶段4再分离)
    ├── mod.rs
    ├── emotion_state.rs
    ├── affective_interface.rs
    └── intrinsic_motivation.rs
```

---

## Phase 4: L4 Emotion 层 (Day 12-14)

### 4.1 从 L3 分离
```
l4_emotion/
├── nt_feel/
    ├── mod.rs
    ├── core.rs              # EmotionEngine
    ├── regulation.rs        # 情绪调节
    ├── expression.rs        # 情绪表达
    ├── social.rs            # 社交情感
    └── triggers.rs          # 触发规则
```

L3 的 `nt_feel` 变为薄适配层，委托给 L4。

---

## Phase 5: L5 Cognition 层 (Day 15-18)

### 5.1 目标结构
```
l5_cognition/
├── nt_core/
│   ├── mod.rs
│   ├── e8/
│   ├── gwt/
│   ├── hcube/
│   ├── self/
│   ├── signal/
│   ├── blueprint.rs
│   ├── design_extract.rs
│   ├── doop.rs
│   ├── gencad.rs
│   ├── kernel.rs
│   ├── parallel/
│   ├── simplify.rs
│   ├── three_scope_map.rs
│   ├── fep_iit/
│   ├── iit_phi.rs
│   └── heartbeat.rs
└── nt_mind/
    ├── mod.rs
    ├── seal/
    ├── absorption_registry.rs
    ├── autofixer.rs
    ├── background_config.rs
    ├── background_loop/
    ├── benchmark.rs
    ├── build_runner.rs
    ├── cleanup.rs
    ├── bpco.rs
    └── gasp.rs
```

---

## Phase 6: L6 Meta-Cognition 层 (Day 19-21)

### 6.1 目标结构
```
l6_meta/
├── nt_meta/
│   ├── mod.rs
│   ├── coordinator/
│   ├── metacognition_loop.rs
│   └── diagnostician/
├── nt_repair/
│   ├── mod.rs
│   ├── consciousness_monitor.rs
│   └── eval_harness.rs
└── nt_nexus/
    ├── mod.rs
    ├── consonance.rs
    ├── evolution.rs
    ├── meta_observer.rs
    └── transcendent_loop.rs
```

---

## Phase 7: 清理验证 (Day 22-23)

### 7.1 删除旧结构
- 删除 `neotrix/l1_body_impl` 等旧目录
- 删除 `core/l0_substrate` 等旧层级
- 更新根 `mod.rs`

### 7.2 全量验证
```bash
cargo check --all-targets -p neotrix
cargo test -p neotrix --all-targets
cargo test -p neotrix-sim
```

---

## 接口契约设计

### 层间依赖规则
```
L6 Meta 依赖 L5 接口
L5 Cognition 依赖 L4 接口
L4 Emotion 依赖 L3 接口
L3 Embodiment 依赖 L2 接口
L2 Perception 依赖 L1 接口
L1 Action 无下层依赖
```

### 关键 Trait 定义

```rust
// L1 → L2: Action 请求感知
pub trait PerceptionProvider {
    fn get_current_state(&self) -> PerceptionState;
    fn request_attention(&self, focus: Focus);
}

// L2 → L3: 感知请求具身动作
pub trait EmbodimentProvider {
    fn execute_movement(&self, cmd: MovementCmd) -> Result<()>;
    fn get_body_state(&self) -> BodyState;
}

// L3 → L4: 具身请求情感调节
pub trait EmotionProvider {
    fn regulate(&self, emotion: Emotion, intensity: f32) -> RegulatedEmotion;
    fn get_emotional_state(&self) -> EmotionalState;
}

// L4 → L5: 情感请求认知决策
pub trait CognitionProvider {
    fn decide(&self, context: DecisionContext) -> Decision;
    fn reason(&self, query: ReasoningQuery) -> ReasoningResult;
}

// L5 → L6: 认知请求元认知监控
pub trait MetaProvider {
    fn reflect(&self, cognition_state: CognitionState) -> Reflection;
    fn evolve(&self, plan: EvolutionPlan) -> EvolutionResult;
}
```

---

## 迁移检查清单 (每阶段)

- [ ] 创建目标目录结构
- [ ] 移动文件内容
- [ ] 更新所有 `use` 路径
- [ ] 实现层接口 Trait
- [ ] 编写适配器 (旧→新)
- [ ] `cargo check` 通过
- [ ] 相关测试通过
- [ ] 更新文档

---

## 风险缓解

| 风险 | 缓解措施 |
|------|----------|
| 编译错误积累 | 每移动一个文件立即 `cargo check` |
| 测试失败 | 优先修复测试，再移动下一个 |
| 循环依赖 | 严格遵循层级依赖规则，用 trait 隔离 |
| 接口变更 | 先定稿接口，再实现 |

---

## 并行化策略

可并行执行的阶段：
- Phase 1 (L1) 和 Phase 2 (L2) 可并行 (无依赖)
- Phase 3 (L3) 依赖 Phase 2 完成
- Phase 4 (L4) 可与 Phase 5 (L5) 并行 (通过接口隔离)
- Phase 6 (L6) 必须最后

---

## 回滚策略

每阶段完成后打 Tag：
```bash
git tag refactor-l1-action-done
git tag refactor-l2-perception-done
...
```

失败时可快速回滚到上一个稳定 Tag。

---

## 里程碑

| 里程碑 | 标志 |
|--------|------|
| M1 | L1 编译通过，测试通过 |
| M2 | L1+L2 编译通过，集成测试通过 |
| M3 | L1-L3 完整链路通 |
| M4 | L4 情感独立，L3 适配 |
| M5 | L5 认知层通 |
| M6 | L6 元认知接入 |
| M7 | 旧结构删除，全量测试通过 |