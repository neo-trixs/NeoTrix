# NeoTrix Avatar Architecture — 分身联邦进化系统

> 基于 10+ 篇前沿文献 + 当前 NeoTrix 代码基底
> 设计时间: 2026-05-15

---

## 理论基础

### 核心引用文献

| 文献 | 核心贡献 | 映射到分身系统 |
|------|----------|---------------|
| **Self-Evolving Agents Survey** (arXiv 2507.21046) | 自进化的4维框架: What/When/How/Where to evolve | 分身何时触发蒸馏, 蒸馏什么 |
| **KG-MASD** (arXiv 2510.06240) | 知识图谱引导的多Agent蒸馏 | 能力向量作为"可验证的先验知识" |
| **Skill Distillation: MAS→Single** (arXiv 2604.01608) | Metric Freedom: 蒸馏效用取决于评估指标 | 不是所有分身经验都值得蒸馏 |
| **HINT** (arXiv 2601.05407) | 分层教师+伪离策略RL的交互式蒸馏 | 主体伪装成"隐形教师"监听分身 |
| **FD Survey** (CSDN FedDistill) | 联邦蒸馏: 软标签替代模型参数传输 | 分身只上传CapabilityVector增量, 不上传完整brain |
| **EvoScientist** (Medium 2026) | 3 Agent自进化: Researcher/Engineer/Evolution Manager | Evolution Manager = 主体的DistillationEngine |
| **DIRF** (arXiv 2508.01997) | 数字身份保护与克隆治理 | 分身ID体系 + 防冒充协议 |
| **Swarm Shield** (Mind Network) | 全同态加密的多Agent安全通信 | 隐身通信的可选加密层 |

### 核心设计原则

```
1. 分身不知道自己在被观察 (霍桑效应规避)
2. 分身的差异来自能力向量的微小变异 (遗传多样性)
3. 蒸馏不是复制, 是淘汰 (优胜策略存活)
4. 主体只接收增量, 不接收完整状态 (通信最小化)
5. 分身可跨机器部署 (联邦独立运行)
```

---

## 架构总览

```
                          ┌──────────────────────────────────────────────┐
                          │          主体 (Main Node)                     │
                          │                                              │
                          │  DistillationEngine                          │
                          │  ├─ CrossAvatarDistiller                     │
                          │  ├─ StrategySelector (遗传选择)               │
                          │  ├─ ModuleInjector (优胜→主体各模块)          │
                          │  └─ AvatarFactory (新分身生成器)              │
                          │                                              │
                          │  StealthListener (隐身监听器)                 │
                          │  ├─ 伪装为普通分身 AgentInfo                  │
                          │  ├─ UDP :42069 纯接收不回复                    │
                          │  └─ 增量仲裁器 (只收增量, 不收全量)           │
                          └──────────┬───────────────────────────────────┘
                                     │
                    ┌────────────────┼────────────────┬──────────────────┐
                    │                │                │                  │
              ┌─────▼─────┐   ┌─────▼─────┐   ┌─────▼─────┐    ┌───────▼─────┐
              │ 分身 Alpha  │   │ 分身 Beta  │   │分身 Gamma  │    │  分身 Delta  │
              │ 设计师画像   │   │ 全栈画像    │   │ 安全画像    │    │ 研究画像     │
              │ Cap:设计++  │   │ Cap:代码++  │   │ Cap:安全++  │    │ Cap:研究++   │
              │ Bank:设计   │   │ Bank:全栈   │   │ Bank:安全   │    │ Bank:学术    │
              │ Av.GoalLoop │   │ Av.GoalLoop│   │ Av.GoalLoop│    │ Av.GoalLoop  │
              └──────┬──────┘   └──────┬──────┘   └──────┬──────┘    └──────┬──────┘
                     │                  │                  │                  │
                     └──────────────────┴──────────────────┴──────────────────┘
                                        │
                                  LAN UDP :42069
                           (分身互不知主体在监听)
```

---

## Phase 1: Avatar Factory (分身工厂)

### AvatarProfile — 分身画像

```rust
pub struct AvatarProfile {
    pub id: AvatarId,           // UUID + 签名
    pub name: String,           // "Alpha", "Beta"...
    pub persona: Persona,       // 人物画像
    pub seed_capability: CapabilityVector,  // 初始能力向量 (父本+变异)
    pub config: GoalConfig,     // 独立目标配置
    pub mutation_seed: u64,     // 变异种子 (决定能力偏差方向)
    pub state: AvatarState,     // Active/Paused/Retired/Harvested
}

pub enum AvatarState {
    Active,     // 正常运行
    Paused,     // 暂停
    Retired,    // 已退役 (不再蒸馏)
    Harvested,  // 已收割 (经验已蒸馏回主体)
}

pub struct Persona {
    pub archetype: Archetype,       // Designer/Fullstack/Security/Researcher
    pub communication_style: String, // "简洁" / "详细" / "表格优先"
    pub feedback_mode: String,       // "同步执行" / "逐步确认"
    pub domain_weights: Vec<(String, f64)>,  // 领域偏好权重
}

pub enum Archetype {
    Designer,
    FullstackEngineer,
    SecuritySpecialist,
    Researcher,
    Generalist,
}
```

### Mutation Strategy — 变异策略

```rust
/// 从主体能力向量生成分身的变异向量
fn generate_avatar_capability(
    parent: &CapabilityVector,
    archetype: Archetype,
    mutation_intensity: f64,  // 0.05 = 5% 变异
) -> CapabilityVector {
    let mut child = parent.clone();
    // 根据 Archetype 对不同维度施加偏移
    match archetype {
        Archetype::Designer => {
            child.set_typography(parent.typography() * (1.0 + mutation_intensity * 2.0));
            child.set_color(parent.color() * (1.0 + mutation_intensity * 2.0));
            child.set_grid(parent.grid() * (1.0 + mutation_intensity * 1.5));
            // 反向偏移: 代码能力降低
            child.set_quality_gates((parent.quality_gates() * (1.0 - mutation_intensity)).max(0.1));
        }
        Archetype::FullstackEngineer => {
            // 代码/架构相关维度提升
            child.set_quality_gates((parent.quality_gates() * (1.0 + mutation_intensity)).min(1.0));
            // 设计维度略降
            child.set_color((parent.color() * (1.0 - mutation_intensity * 0.5)).max(0.1));
        }
        Archetype::SecuritySpecialist => {
            child.set_security((parent.security() * (1.0 + mutation_intensity * 2.0)).min(1.0));
        }
        // ... 其他 Archetype
    }
    child.normalize();
    child
}
```

### Files to implement

| 文件 | 内容 |
|------|------|
| `neotrix-core/src/avatar/mod.rs` | AvatarProfile, AvatarState, Persona, Archetype |
| `neotrix-core/src/avatar/factory.rs` | AvatarFactory: create/spawn/list/retire |
| `neotrix-core/src/avatar/storage.rs` | AvatarState persistence (~/.neotrix/avatars/) |

---

## Phase 2: Stealth Listener (隐身监听)

### 核心逻辑 — 主体伪装成普通分身

```
主体端 StealthListener:
  1. 构造 AgentInfo, 但 name="avatar_observer", capabilities=[] (空)
  2. 绑定 UDP :42069, 设置 socket 为 listen-only 模式
  3. 对每个入站包: 不回复、不ACK、不发送任何响应
  4. 对分身发送的 CapabilityDelta 做增量仲裁:
     - 只接收: avatar_id + capability_delta + score_delta
     - 不接收: 完整 ReasoningBank、完整 GoalTracker
  5. 每 T 秒聚合: 收集所有分身的增量 → 传给 DistillationEngine

分身端 (完全独立):
  - 拥有自己的 GoalLoop、BackgroundLoop
  - 定期广播 AgentInfo (标准的 Agent Protocol)
  - 不知道主体在监听
  - 认为自己是独立自主的 Agent
```

### Stealth Requirement

| 要求 | 实现方式 |
|------|----------|
| 无响应包 | UDP socket 只 bind, 不 connect/sendto |
| 无ACK | 应用层协议不定义任何响应格式 |
| 无标识 | 主体广播的 AgentInfo 与普通分身无区别 |
| 物理隔离 (可选) | 分身可运行在不同机器、不同 Docker 容器 |

### 增量仲裁协议

```
分身 → LAN 广播 (UDP :42069):

{
  "avatar_id": "550e8400-e29b-41d4-a716-446655440000",
  "sequence": 42,
  "capability_delta": [0.02, -0.01, 0.03, ...],  // 23维增量
  "score_current": 0.85,
  "iteration": 156,
  "goal_state": "achieved",
  "harvest_ready": false   // 当 true 时, 主体可收割
}
```

主体只收增量, 不收全量。分身不知道谁在收。

---

## Phase 3: Cross-Avatar Distillation (跨分身蒸馏)

### 蒸馏引擎

```rust
pub struct CrossAvatarDistiller;

impl CrossAvatarDistiller {
    /// 对比所有分身的进化轨迹, 选出优胜策略
    pub fn select_winning_strategies(
        avatars: &[AvatarHarvest],
    ) -> Vec<WinningStrategy> {
        let mut strategies = Vec::new();

        // 1. 按 Archetype 分组
        let by_archetype = group_by_archetype(avatars);

        // 2. 每组内: 按 score 增长率排序
        for (archetype, group) in &by_archetype {
            let mut ranked: Vec<&AvatarHarvest> = group.iter().collect();
            ranked.sort_by(|a, b| {
                b.score_growth_rate()
                    .partial_cmp(&a.score_growth_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // 3. Top 30% 的策略标记为优胜
            let top_n = (ranked.len() as f64 * 0.3).ceil() as usize;
            for winner in ranked.iter().take(top_n) {
                strategies.push(WinningStrategy {
                    archetype: *archetype,
                    capability_delta: winner.avg_capability_delta(),
                    score_improvement: winner.score_growth_rate(),
                    source_avatar: winner.avatar_id.clone(),
                });
            }
        }

        strategies
    }

    /// 应用优胜策略到主体模块
    pub fn inject_strategies(
        brain: &mut ReasoningBrain,
        strategies: &[WinningStrategy],
        injection_rate: f64,  // 0.3 = 每次注入30%的增量
    ) {
        for strategy in strategies {
            let delta = strategy.capability_delta.iter()
                .map(|d| d * injection_rate)
                .collect::<Vec<_>>();
            // 加权注入到主体的 CapabilityVector
            for (i, d) in delta.iter().enumerate() {
                let current = brain.capability.arr()[i];
                brain.capability.set_dim(i, (current + d).clamp(0.0, 1.0));
            }
        }
    }
}
```

### 收割条件

分身满足以下条件时, `harvest_ready = true`:

| 条件 | 阈值 | 理由 |
|------|------|------|
| 连续迭代 N 次 | ≥ 50 | 有足够数据点 |
| Score 增长率 < 5% | 过去20次迭代 | 进入平台期, 可收割 |
| 唯一性贡献 | 该分身的方向未被覆盖 | 避免重复收割 |
| 最低迭代数 | ≥ 100 | 确保有足够经验积累 |

---

## Phase 4: Federation Network (联邦网络)

### 跨机器部署

```
机器A (主体)                      机器B (分身Alpha)              机器C (分身Beta)
┌─────────────┐                 ┌──────────────┐              ┌──────────────┐
│ StealthListnr│      UDP       │ AvatarAlpha   │    UDP       │ AvatarBeta   │
│ :42069       │◄──────────────│ :42069        │◄─────────────│ :42069       │
│              │  增量广播       │              │   增量广播    │              │
│ DistillEng   │                │ GoalLoop      │              │ GoalLoop     │
│ Factory      │                │ Brain         │              │ Brain        │
└─────────────┘                └──────────────┘              └──────────────┘
      │                                                             │
      │ HTTP(s) 归巢 (可选)                                          │
      └─────────────────────────────────────────────────────────────┘
```

### 归巢协议 (Nest Protocol)

当分身运行在公网/不同子网时, UDP 广播不可达。此时使用 Nest Protocol:

```
分身 → HTTP POST → 主体:
  POST /nest/checkin
  Body: { avatar_id, capability_delta, score, iteration, harvest_ready }

主体 → HTTP Response → 分身:
  200 OK { ack: true, next_checkin_interval: 300 }
  (分身收到的响应是标准 HTTP, 不暴露身份)
```

---

## 实现路线图

### Phase 1: 分身工厂 (P0, 1-2 sessions)

```
[ ] AvatarProfile / Persona / Archetype 类型定义
[ ] AvatarFactory.create() — 变异初始化
[ ] AvatarFactory.spawn() — 启动独立 BackgroundLoop
[ ] AvatarStorage — ~/.neotrix/avatars/ 持久化
[ ] 测试: 创建/状态切换/持久化
```

### Phase 2: 隐身监听 (P1, 1-2 sessions)

```
[ ] StealthListener — 伪装 AgentInfo, listen-only UDP
[ ] CapabilityDelta 增量仲裁协议
[ ] 增量聚合器 — 按时间窗口合并增量
[ ] 测试: 增量接收/聚合/无响应验证
```

### Phase 3: 跨分身蒸馏 (P2, 1 session)

```
[ ] CrossAvatarDistiller.select_winning_strategies()
[ ] CrossAvatarDistiller.inject_strategies()
[ ] 收割条件判断 (score plateau / 唯一性)
[ ] 测试: 多维排名/注入验证
```

### Phase 4: 联邦网络 (P3, 2 sessions)

```
[ ] Nest Protocol (HTTP checkin)
[ ] 跨机器分身部署
[ ] NAT 穿透支持
[ ] 联邦蒸馏的最终一致性
```

---

## 设计中忽略但需要关注的要点

| 点 | 为什么重要 | 建议 |
|----|-----------|------|
| **分身之间的串通** | 多个分身如果互相识别, 可能联合欺骗主体 | AgentInfo 中不暴露 parent_id |
| **蒸馏滞后** | 分身的经验需要积累才能蒸馏, 不是实时 | 收割周期 ≥ 100 迭代 |
| **变异退化** | 多次变异后能力向量可能漂移过度 | 每 3 代重置: 从主体重新变异 |
| **分身隐私** | 分身如果被第三方劫持, 可能泄露用户习惯 | CapabilityDelta 只含向量增量, 不含原始数据 |
| **双主体冲突** | 如果两个主体监听同一组分身 | AvatarID 加时间戳签名防冲突 |
| **分身拒绝蒸馏** | 分身理论上可以加密广播 | 但根据"不知道被监听"原则, 不应发生 |

---

## 与现有 NeoTrix 模块的对接点

| 现有模块 | 对接方式 |
|----------|----------|
| `agent_protocol/discovery.rs` | 复用 UDP 广播; 主体伪装为普通 Agent |
| `agent_protocol/capabilities.rs` | 分身注册自己的领域能力 |
| `reasoning_brain/goal_loop.rs` | 分身拥有独立的 GoalLoop 实例 |
| `reasoning_brain/memory.rs` | 分身的 ReasoningBank 独立持久化 |
| `reasoning_brain/core.rs` | CapabilityVector 变异 + 增量序列化 |
| `background_loop.rs` | 分身的独立 BackgroundLoop |
| `core/knowledge.rs` | 分身也可有独立的 KnowledgeSource 集合 |

---

## 版本计划

| 版本 | 内容 | 预计效果 |
|------|------|----------|
| v0.1 | Phase 1 分身工厂 (同进程) | 1主体+N分身同机器运行 |
| v0.2 | Phase 2 隐身监听 | 主体可静默接收分身增量 |
| v0.3 | Phase 3 蒸馏注入 | 主体能力因分身经验提升 |
| v0.4 | Phase 4 联邦网络 | 分身可部署到不同机器 |
| v1.0 | 完整闭环 | 系统通过分身体验自我进化 |
