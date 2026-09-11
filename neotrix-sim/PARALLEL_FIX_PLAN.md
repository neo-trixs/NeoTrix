# NT-WORLD-SIM 并行修复计划

> Generated: 2026-09-11 | 基于 CAPABILITY_GAP_ANALYSIS.md
> 策略: 6 个并行 Agent，每 Agent 负责一个域，同时修复冗余+缺陷+错位

---

## Agent 分配

### Agent 1: NT-WORLD (世界感知域)
**目标**: 修复 R1 事件调度冗余 + D1/D3/D5 扁平缺陷 + M1/M4 跨域错位

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| EventMode 集成 | 冗余 | +60 | `event_reactive.rs` |
| Day/Night 循环 | 缺陷 | +200 | `environment/daynight.rs` (新建) |
| 天气系统 | 缺陷 | +200 | `environment/weather.rs` (新建) |
| Spawn 模板 | 缺陷 | +120 | `world_sim/spawn.rs` (新建) |
| 地图生成 (WFC) | 缺陷 | +400 | `environment/wfc.rs` (新建) |
| 信息素迁移 | 错位 | ±50 | `pheromone.rs` → NT-SHIELD |
| 派系领土迁移 | 错位 | ±80 | `faction.rs` → NT-WORLD territory |
| **小计** | | **~1,010** | |

### Agent 2: NT-ACT (行动执行域)
**目标**: 修复 R2 AI 决策冗余 + G03/G04/G05/G08 接通 + G24/G25 引擎统一

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| GOAP 接通 | 冗余激活 | +80 | `decision.rs:layer_goals()` |
| BT 接通 | 冗余激活 | +60 | `decision.rs:bt_root()` |
| Stimulus-Response | 缺陷 | +100 | `agents/stimulus.rs` (新建) |
| Attack/Gather | 冗余激活 | +80 | `actions.rs` match 补全 |
| DecisionEngine 统一 | 冗余融合 | +200 | `agents/decision_engine.rs` (新建) |
| 供需定价 | 缺陷 | +120 | `economy/pricing.rs` (新建) |
| 货币系统 | 缺陷 | +150 | `economy/currency.rs` (新建) |
| 合成/制作 | 缺陷 | +250 | `economy/crafting.rs` (新建) |
| 市场 | 缺陷 | +200 | `economy/marketplace.rs` (新建) |
| **小计** | | **~1,240** | |

### Agent 3: NT-MEMORY (知识守护域)
**目标**: 修复 R4 记忆操作冗余 + G09 持久化 + G15/G16/G17 记忆系统

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| Save/Load | 缺陷 | +300 | `world_sim/persistence.rs` (新建) |
| 语义记忆 | 缺陷 | +200 | `agents/semantic_memory.rs` (新建) |
| 记忆巩固 | 缺陷 | +150 | `agents/memory_consolidation.rs` (新建) |
| 记忆检索 | 缺陷 | +120 | `agents/memory_retrieval.rs` (新建) |
| 对抗性记忆 | 缺陷 | +150 | `agents/adversarial_memory.rs` (新建) |
| 因果记忆 | 缺陷 | +150 | `agents/causal_memory.rs` (新建) |
| 知识图谱 | 缺陷 | +200 | `agents/knowledge_graph.rs` (新建) |
| MemoryManager 统一 | 冗余融合 | +150 | `agents/memory_manager.rs` (新建) |
| **小计** | | **~1,420** | |

### Agent 4: NT-SOC (社会动力学域)
**目标**: 修复 R3 社交系统冗余 + G18/G19/G20 社交功能

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| 八卦协议接通 | 冗余激活 | +40 | `society/gossip.rs` → WorldSim |
| 派系系统接通 | 冗余激活 | +50 | `society/faction.rs` → WorldSim |
| 通信频道 | 缺陷 | +180 | `society/communication.rs` (新建) |
| SocialEngine 统一 | 冗余融合 | +150 | `society/social_engine.rs` (新建) |
| 谈判系统 | 缺陷 | +120 | `society/negotiation.rs` (新建) |
| 冲突解决 | 缺陷 | +200 | `society/conflict.rs` (新建) |
| 领导层级 | 缺陷 | +150 | `society/hierarchy.rs` (新建) |
| 规范演化 | 缺陷 | +100 | `society/norm_evolution.rs` (新建) |
| **小计** | | **~990** | |

### Agent 5: NT-MIND/SHIELD (进化+安全域)
**目标**: 修复 G21-G23 进化功能 + G42-G44 安全功能

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| 自我对弈 | 缺陷 | +200 | `evolution/self_play.rs` (新建) |
| 新颖性搜索 | 缺陷 | +180 | `evolution/novelty.rs` (新建) |
| 精英档案 | 缺陷 | +100 | `evolution/archive.rs` (新建) |
| 协进化 | 缺陷 | +200 | `evolution/coevolution.rs` (新建) |
| 课程学习 | 缺陷 | +150 | `evolution/curriculum.rs` (新建) |
| 迁移学习 | 缺陷 | +150 | `evolution/transfer.rs` (新建) |
| 多目标优化 | 缺陷 | +180 | `evolution/moo.rs` (新建) |
| 资源预算执行 | 缺陷 | +80 | `safety/budget_enforcement.rs` (新建) |
| 行为异常检测 | 缺陷 | +150 | `safety/anomaly.rs` (新建) |
| 进化沙箱 | 缺陷 | +150 | `safety/sandbox.rs` (新建) |
| **小计** | | **~1,540** | |

### Agent 6: NT-IO (界面使徒域)
**目标**: 修复 D1 Canvas 渲染 + D4 粒子/动画 + G30 UI + G13 时间控制

| 任务 | 类型 | LOC | 文件 |
|------|------|----:|------|
| Canvas 2D 渲染循环 | 缺陷 | +400 | `ui/renderer.rs` (新建) |
| Sprite 系统 | 缺陷 | +200 | `ui/sprite.rs` (新建) |
| Camera 控制 | 缺陷 | +100 | `ui/camera.rs` (新建) |
| Agent 选择/检查 | 缺陷 | +150 | `ui/agent_inspector.rs` (新建) |
| 时间控制 | 缺陷 | +80 | `ui/time_control.rs` (新建) |
| HUD 面板 | 缺陷 | +300 | `ui/panels/mod.rs` (新建) |
| 粒子系统 | 缺陷 | +300 | `ui/particle.rs` (新建) |
| 关键帧动画 | 缺陷 | +200 | `ui/animation.rs` (新建) |
| **小计** | | **~1,730** | |

---

## 执行顺序

```
Wave 1 (立即启动, Agent 1-6 并行):
  ├── Agent 1: NT-WORLD: DayNight + Weather + Spawn + WFC
  ├── Agent 2: NT-ACT: DecisionEngine + GOAP/BT 接通 + 经济
  ├── Agent 3: NT-MEMORY: Save/Load + 语义记忆 + 巩固 + 检索
  ├── Agent 4: NT-SOC: SocialEngine + 通信 + 八卦 + 派系
  ├── Agent 5: NT-MIND/SHIELD: 进化功能 + 安全功能
  └── Agent 6: NT-IO: Canvas 渲染 + UI + 粒子 + 动画

Wave 2 (Wave 1 完成后, Agent 7-9 清理):
  ├── Agent 7: 跨域错位迁移 (M1-M5)
  ├── Agent 8: 冗余清理 (R1-R5 去重)
  └── Agent 9: 集成测试 + 编译验证
```

---

## 编译验证

每个 Agent 完成后必须运行:
```bash
# 清理锁
pkill -9 -f cargo; rm -f target/debug/.cargo-lock

# 编译检查
cargo check -p neotrix-sim 2>&1 | grep -c "error\["

# 测试
cargo test -p neotrix-sim --lib 2>&1 | tail -5
```

**目标**: error count = 0, test count ≥ 395

---

## LOC 预算

| Agent | 域 | LOC |
|:-----:|:---|----:|
| 1 | NT-WORLD | ~1,010 |
| 2 | NT-ACT | ~1,240 |
| 3 | NT-MEMORY | ~1,420 |
| 4 | NT-SOC | ~990 |
| 5 | NT-MIND/SHIELD | ~1,540 |
| 6 | NT-IO | ~1,730 |
| **总计** | | **~7,930** |

> 比 CAPABILITY_CHECKLIST 的 10,250 LOC 少 22%，因为:
> 1. 聚焦冗余消减: 5 组冗余合并后减少 ~1,500 LOC
> 2. 跨域错位迁移: 5 处迁移是移动而非新建
> 3. 复用已有模块: GOAP/BT/Gossip/Faction 已有代码只需接通
