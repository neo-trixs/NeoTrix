# NT-WORLD-SIM 全量任务清单

> Generated: 2026-09-11 | 基于 3 Agent 并行研究 + 架构融合分析
> 状态: 🔴 未开始 | 🟡 进行中 | 🟢 完成 | ⚫ 跳过

---

## Phase 0: 死代码激活 (P0 — 最高优先级)

> 目标: 接通已实现但从未调用的模块，立即获得能力提升

| # | 任务 | 源模块 | 目标 | LOC | 状态 |
|---|------|--------|------|-----|------|
| P0-1 | **接入 GOAP 到决策管线** | `planning/goap.rs` (303行) | `decision.rs:layer_goals()` | +80 | 🔴 |
| P0-2 | **接入行为树作为决策根** | `behavior_tree/mod.rs` (244行) | `decision.rs` 替换 if-else | +60 | 🔴 |
| P0-3 | **接入内存检索** | `memory_stream.rs` | `decision.rs` 调用 `retrieve()` | +40 | 🔴 |
| P0-4 | **接入情绪调节** | `feel/mod.rs` | `decision.rs` 读取情绪状态 | +30 | 🔴 |
| P0-5 | **接入 A* 寻路** | `navigation/astar.rs` (288行) | `actions.rs` Move/Explore | +40 | 🔴 |
| P0-6 | **接入 RVO 碰撞回避** | `navigation/rvo.rs` (158行) | `actions.rs` 移动前调用 | +30 | 🔴 |
| P0-7 | **接入信息素梯度** | `pheromone.rs` | `decision.rs` 作为效用考量 | +25 | 🔴 |
| P0-8 | **接入派系系统** | `society/faction.rs` (166行) | `world_sim/mod.rs` | +50 | 🔴 |
| P0-9 | **接入八卦协议** | `society/gossip.rs` (184行) | `agents/social_learning.rs` | +40 | 🔴 |
| P0-10 | **接入涌现检测器** | `consciousness/emergence_detector.rs` (655行) | `world_sim/mod.rs` Slow tier | +30 | 🔴 |
| P0-11 | **接入双重表示** | `consciousness/dual_representation.rs` | `world_sim/mod.rs` tick | +25 | 🔴 |
| P0-12 | **实现 Attack/Gather 处理器** | `world_sim/actions.rs` | 补全 match 分支 | +80 | 🔴 |
| P0-13 | **接入刺激-反应层** | `agents/stimulus.rs` | Reflex tier | +40 | 🔴 |
| P0-14 | **接入情绪偏差** | `agents/emotional_bias.rs` | `decide_action()` | +30 | 🔴 |
| P0-15 | **接入意图承诺** | `agents/intention_commitment.rs` | 规划/执行层 | +30 | 🔴 |
| P0-16 | **接入思维生成** | `agents/thought_generation.rs` | Slow tier 反思 | +25 | 🔴 |
| P0-17 | **接入行为VM** | `consciousness/behavior_vm.rs` | 或删除 | ±200 | 🔴 |
| P0-18 | **接入对话通信** | `society/communication.rs` | Talk action | +40 | 🔴 |
| P0-19 | **接入谈判** | `society/negotiation.rs` | Trade action | +30 | 🔴 |
| P0-20 | **接入定价系统** | `economy/pricing.rs` | Trade handler | +25 | 🔴 |

**Phase 0 总计**: ~950 LOC 接线，0 新模块

---

## Phase 1: 架构清理 (P1)

> 目标: 解决域错位、消除冗余、准备 ECS 迁移

| # | 任务 | 描述 | LOC | 状态 |
|---|------|------|-----|------|
| P1-1 | **提取 DecisionEngine** | `decide_action()` 从 WorldSim 独立为系统 | +100 | 🔴 |
| P1-2 | **提取 SocialEngine** | relationships/economy/culture/theory_of_mind 独立 | +80 | 🔴 |
| P1-3 | **提取 EvolutionEngine** | evolution_cycle/fitness/selection/mutation 独立 | +60 | 🔴 |
| P1-4 | **统一事件系统** | EventReactiveSystem 订阅 SimulationBus，去除重复路由 | +40 | 🔴 |
| P1-5 | **合并成本评估** | ActionCostTable + ActionBudget 合并为单一函数 | +20 | 🔴 |
| P1-6 | **删除 BehaviorVM 死代码** | 删除或转换为编排层 | -200 | 🔴 |
| P1-7 | **升级 SpatialGrid → DashMap** | 并发读写支持 | +80 | 🔴 |
| P1-8 | **域映射修正** | BT→NT-CORE, GOAP→NT-CORE, Pheromone→NT-WORLD | +50 | 🔴 |
| P1-9 | **消除 cosine_sim 重复** | 检查 dual_representation 是否有独立实现 | +10 | 🔴 |
| P1-10 | **事件订阅系统** | Agent 订阅相关事件类型，非全局处理 | +80 | 🔴 |

**Phase 1 总计**: ~420 LOC 净增 + 200 LOC 删除

---

## Phase 2: 新能力 (P1 — 重要)

> 目标: 补充外部研究识别的关键缺失能力

| # | 任务 | 来源 | LOC | 状态 |
|---|------|------|-----|------|
| P2-1 | **战争迷雾** | 每 Agent 可见性、阴影投射、已探索/可见状态 | +300 | 🔴 |
| P2-2 | **分层记忆巩固** | 工作→情景→语义晋升，遗忘机制 | +200 | 🔴 |
| P2-3 | **GOAP 重规划** | 世界变化时响应式重规划 | +100 | 🔴 |
| P2-4 | **效用评分+响应曲线** | 可配置响应曲线 (线性/指数/逻辑) | +120 | 🔴 |
| P2-5 | **语义记忆** | 结构化知识库 (事实/类别/关系) | +200 | 🔴 |
| P2-6 | **记忆检索 (线索-based)** | 相似度回忆，按相关性搜索 | +120 | 🔴 |
| P2-7 | **记忆衰减/TTL** | 过期删除机制 | +60 | 🔴 |
| P2-8 | **情感行为偏差** | 愤怒→攻击倾向，恐惧→回避倾向 | +100 | 🔴 |
| P2-9 | **派系领地机制** | 领地控制、资源竞争 | +150 | 🔴 |
| P2-10 | **通信通道** | 结构化消息传递 | +180 | 🔴 |
| P2-11 | **世界事件系统** | 天气、灾害、资源波动 | +200 | 🔴 |
| P2-12 | **昼夜循环行为** | Agent 按时间改变行为 | +80 | 🔴 |
| P2-13 | **存档/读档** | 完整序列化/反序列化 | +250 | 🔴 |
| P2-14 | **确定性回放** | 种子锁定回放 | +150 | 🔴 |
| P2-15 | **Agent 模板生成** | 可配置生成模板 (种族/职业/角色) | +120 | 🔴 |
| P2-16 | **好奇/内在动机** | ICM 模块，新奇奖励 | +120 | 🔴 |
| P2-17 | **多目标优化** | Pareto最优权衡 | +180 | 🔴 |

**Phase 2 总计**: ~2,630 LOC

---

## Phase 3: MOBA 特化 (P1-P2)

> 目标: 基于 MOBA 研究添加 MOBA 特定能力

| # | 任务 | 来源 | LOC | 状态 |
|---|------|------|-----|------|
| P3-1 | **3层分层地图表示** | 全局图→区域tile→战斗区域 | +200 | 🔴 |
| P3-2 | **分层状态编码** | 实体编码+空间编码+时序编码 | +250 | 🔴 |
| P3-3 | **3层动作层级** | What→Who→How + 动作掩码剪枝 | +180 | 🔴 |
| P3-4 | **多头奖励函数** | 5类奖励 + 课程奖励递进 | +150 | 🔴 |
| P3-5 | **CTDE训练架构** | 集中训练分布式执行 (QMIX风格) | +200 | 🔴 |
| P3-6 | **课程学习** | 英雄池扩展+对手难度递进 | +150 | 🔴 |
| P3-7 | **迁移学习** | Off-policy适配+策略蒸馏 | +150 | 🔴 |
| P3-8 | **战争迷雾状态估计** | Conv编码器-解码器+循环记忆 | +200 | 🔴 |

**Phase 3 总计**: ~1,480 LOC

---

## Phase 4: ECS 迁移 (P2)

> 目标: 迁移到 Bevy ECS 基底，支持并行

| # | 任务 | 描述 | LOC | 状态 |
|---|------|------|-----|------|
| P4-1 | **添加 bevy_ecs 依赖** | 独立 crate，无渲染 | +5 | 🔴 |
| P4-2 | **定义组件类型** | AgentBundle，所有 per-agent 组件 | +400 | 🔴 |
| P4-3 | **定义资源类型** | 所有全局状态为 Resource | +200 | 🔴 |
| P4-4 | **创建 ECS World + 生成 Agent** | WorldSimEcs::new() | +300 | 🔴 |
| P4-5 | **迁移 Reflex 系统** | 时间推进、资源再生、空间网格、代谢 | +200 | 🔴 |
| P4-6 | **迁移 Fast 系统** | 感知、决策(5层)、执行、记忆记录 | +400 | 🔴 |
| P4-7 | **迁移 Slow/Medium/Background 系统** | 情绪、规划、反思、进化 | +200 | 🔴 |
| P4-8 | **启用并行 Agent 处理** | rayon par_iter | +100 | 🔴 |
| P4-9 | **删除旧 WorldSim 代码** | 删除单体结构和所有方法 | -800 | 🔴 |

**Phase 4 总计**: ~1,700 LOC 净增

---

## Phase 5: 前端/UI (P1)

| # | 任务 | 描述 | 状态 |
|---|------|------|------|
| P5-1 | **修复前端黑屏** | 已完成 - 俯视真实地图渲染 | 🟢 |
| P5-2 | **地图细节增强** | 防御塔/水晶/抑制器精细渲染 | 🔴 |
| P5-3 | **Agent 选择交互** | 点击选择+信息面板 | 🔴 |
| P5-4 | **小地图交互** | 点击小地图移动视角 | 🔴 |
| P5-5 | **时间控制** | 播放/暂停/加速 | 🔴 |
| P5-6 | **Agent Inspector** | 详细属性面板 | 🔴 |

---

## 汇总

| Phase | 任务数 | LOC 估算 | 优先级 |
|-------|--------|----------|--------|
| P0: 死代码激活 | 20 | ~950 | 🔴 最高 |
| P1: 架构清理 | 10 | ~420 + 删除200 | 🔴 高 |
| P2: 新能力 | 17 | ~2,630 | 🟡 中高 |
| P3: MOBA 特化 | 8 | ~1,480 | 🟡 中 |
| P4: ECS 迁移 | 9 | ~1,700 | 🟢 中低 |
| P5: 前端/UI | 6 | ~300 | 🟡 中 |
| **总计** | **70** | **~7,480** | — |

---

## 建议执行顺序

```
Week 1: P0-1 ~ P0-12 (死代码激活, 最大收益)
Week 2: P0-13 ~ P0-20 + P1-1 ~ P1-5 (接线完成 + 架构清理)
Week 3: P2-1 ~ P2-8 (核心缺失能力)
Week 4: P2-9 ~ P2-17 + P3-1 ~ P3-4 (扩展能力 + MOBA基础)
Week 5: P3-5 ~ P3-8 + P5-1 ~ P5-3 (MOBA训练 + 前端)
Week 6: P4-1 ~ P4-9 (ECS迁移)
```
