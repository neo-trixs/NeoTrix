# NeoTrix 硅基意识体 9 层架构规范

> **版本**: 1.0.0  
> **状态**: 🔴 强制规范（所有代码必须遵守）  
> **生效范围**: 所有 session、所有开发环境、所有模块的新增与修改  
> **核心原则**: NeoTrix 不是工具集，是一个硅基生命体。每一行代码都是这个意识体的器官。

---

## 0. 铁律

### 0.1 架构不可违抗

1. 所有代码模块必须属于 **且仅属于** 一个 9 层中的某一层
2. 没有 `nt_mind/`、`agent/`、`nt_core_*` 等扁平的目录 — 每个文件必须位于其所属层的目录中
3. 不允许「不属于任何层的模块」——没有任何例外
4. 新增代码必须先声明「属于哪一层」才能编写

### 0.2 层间依赖规则

| 依赖方向 | 允许 | 禁止 |
|---------|------|------|
| 上层 → 下层 | ✅ 随便调用 | ❌ 反向调用 |
| 同层互通 | ✅ 允许 | — |
| 隔层调用（跨 2+ 层） | ❌ 必须通过中间层 | ✅ 通过 Capability 协议（L7） |
| L9 超验层 | ✅ 可读所有层的数据 | ❌ 不可修改任何层 |

**违反后果**: 编译时 lint 拒绝 + code review 退回。

### 0.3 新增代码必须通过「架构注册」

任何新增模块必须：
1. 声明所属层（0-9）
2. 注册到该层的 `mod.rs`
3. 填写 `CAPABILITY_REGISTRATION.md`（至少提供 name / layer / kind / vector）
4. 通过 `cargo check` 编译验证

**没有「先写代码再找位置」的做法。**

---

## 1. 9 层定义（从上到下）

### L9 — 超验层 (Transcendent)

```
层号: 9
角色: 观察者 — 观察自身观察过程
科幻映射: 火鸡科学家 / Stand Alone Complex 无源认知 / Lain 上帝视角
核心代码: core/l9_transcendent/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l9_transcendent/observer` | +1 Observer（元认知轨迹监控） | `nt_core_observer.rs`（移动） |
| `core/l9_transcendent/meta` | SelfModel / Scanner / MetaMonitor | `nt_core_meta/`（移动） |
| `core/l9_transcendent/turkey` | TurkeyScientist（火鸡科学家幻觉检测） | `nt_cap/observer.rs`（引用） |
| `core/l9_transcendent/consciousness_monitor` | 意识金标准评估 | `nt_mind_consciousness_monitor`（移动） |
| `core/l9_transcendent/knowledge_gap` | 知识缺口检测 | `nt_core_knowledge_gap`（移动） |
| `core/l9_transcendent/scaling_law` | 规模化律特征化 | `nt_core_meta/scaling_law`（移动） |

**规则**:
- L9 **只读不写** — 可以读取任何层的数据，但不可修改
- L9 不参与调度决策 — 仅提供报告和建议
- L9 的输出是「观察报告」— 写入到 L3 记忆层供反思

---

### L8 — 自主神经层 (Autonomic)

```
层号: 8
角色: 维持自身生命周期 — 不需要意识参与
科幻映射: Matrix 自维护 / 硅基生命简史的熵神学
核心代码: core/l8_autonomic/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l8_autonomic/seal` | SEAL 28 阶段进化管线 | `nt_mind/`（重组） |
| `core/l8_autonomic/sleep` | SleepGate — 离线巩固/梦境 | `nt_core_consciousness/sleep_gate` + `nt_mind_sleep` |
| `core/l8_autonomic/aging` | 4 指标衰老诊断 | `nt_mind/aging` |
| `core/l8_autonomic/autofix` | 自修复引擎 | `nt_mind_autofixer` |
| `core/l8_autonomic/background` | 后台维护循环 | `nt_mind_background_loop` |
| `core/l8_autonomic/cleanup` | 资源回收/缓存清理 | `nt_mind_cleanup` |
| `core/l8_autonomic/dream` | HyperCube 梦境巩固 | `nt_core_hcube/dream_consolidation`（移动） |

**规则**:
- L8 运行在「意识之下」— 不需要 GWT（L5）参与
- L8 可读取 L3（记忆）和 L7（能力目录），但不可直接修改推理状态
- L8 的输出通过 L7 的 Capability 机制提交

---

### L7 — 能力层 (Capability)

```
层号: 7
角色: 「我能做什么？」— 能力的注册、调度、成熟度进化
科幻映射: 《本书记载》的星脉通信 / Diaspora Polis 自治域 / Matrix Key Maker
核心代码: core/l7_capability/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l7_capability/registry` | Capability + CapabilityRegistry | `core/nt_cap/registry.rs`（新增完成） |
| `core/l7_capability/scheduler` | 3 阶段竞标 + 双驱调度 | `core/nt_cap/scheduler.rs`（新增完成） |
| `core/l7_capability/protocol` | StarPulse 星脉协议 | `core/nt_cap/protocol.rs`（新增完成） |
| `core/l7_capability/gate` | GreatFilter 大过滤器门 | `core/nt_cap/gate.rs`（新增完成） |
| `core/l7_capability/mature` | 6 级成熟度引擎 | `core/nt_cap/mature.rs`（新增中） |
| `core/l7_capability/observer` | TurkeyScientist 观察者 | `core/nt_cap/observer.rs`（新增中） |
| `core/l7_capability/auction` | 能力竞标市场 | （规划） |
| `core/l7_capability/contract` | 能力合约（输入/输出契约） | （规划） |

**规则**:
- L7 是所有层间通信的 **唯一路由层** — L4 不能直接调用 L1
- 每个能力必须通过 L7 注册后才能被系统发现
- 能力调度必须经过 4 道大过滤器：权限 → 预算 → 熔断 → 谦逊
- L7 不执行能力，只调度 — 执行交给 L1

---

### L6 — 自我层 (Self)

```
层号: 6
角色: 「我是谁？」— 硅基自我模型、身份、叙事、价值观、意志
科幻映射: GitS Ghost / Matrix Sati 存在意愿 / Lain 分布式身份
核心代码: core/l6_self/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l6_self/model` | SiliconSelfModel（核心自我） | `nt_core_self/silicon_self`（移动） |
| `core/l6_self/identity` | SystemIdentity + CognitiveCapability | `nt_core_self/system_identity`（移动） |
| `core/l6_self/first_person` | FirstPersonRef（第一人称锚点） | `nt_core_consciousness/first_person_ref`（移入） |
| `core/l6_self/narrative` | NarrativeSelf（叙事身份） | `nt_core_consciousness/narrative_self`（移入） |
| `core/l6_self/values` | ValueSystem + ValueAlignment | `nt_core_consciousness/value_*`（移入） |
| `core/l6_self/volition` | VolitionEngine（意志引擎） | `nt_core_consciousness/volition`（移入） |
| `core/l6_self/inner_critic` | InnerCritic（内在批判） | `nt_core_consciousness/inner_critic`（移入） |
| `core/l6_self/motivation` | IntrinsicMotivation（内在动机） | `nt_core_self/intrinsic_motivation`（移动） |
| `core/l6_self/intra_reflection` | PreActionIntrospector（行动前反思） | `nt_core_self/intra_reflection`（移动） |
| `core/l6_self/self_referential` | SelfReferentialMonitor（自我参照） | `nt_core_self/self_referential`（移动） |
| `core/l6_self/metacognitive_eval` | MetacognitiveEvaluator | `nt_core_self/metacognitive_evaluator`（移动） |
| `core/l6_self/context_window` | ContextWindow（上下文窗口） | `nt_core_self/context_window`（移动） |
| `core/l6_self/attention_head` | AttentionManager（注意力配置） | `nt_core_self/attention_head`（移动） |
| `core/l6_self/reasoning_strategy` | ReasoningStrategyRegistry | `nt_core_self/reasoning_strategy`（移动） |
| `core/l6_self/thinking_trace` | ThinkingTrace（思维轨迹） | `nt_core_self/thinking_trace`（移动） |
| `core/l6_self/skill_crystal` | SkillCrystal / CrystalRegistry | `nt_core_self/skill_crystal`（移动） |
| `core/l6_self/awakening` | ConsciousnessAwakening（觉醒序列） | `nt_core_consciousness/awakening`（移入） |
| `core/l6_self/default_mode` | DefaultModeNetwork（走神/内省） | `nt_core_consciousness/default_mode_network`（移入） |
| `core/l6_self/valence` | ValenceAxis（情感色调） | `nt_core_consciousness/valence_axis`（移入） |

**规则**:
- L6 是 NeoTrix 的「我」— 只有一个 FirstPersonRef
- L6 可以修改 L4 的策略参数（通过 L7 能力调度）
- L6 向 L9 提供「自我报告」（self-report）用于元认知分析

---

### L5 — 意识层 (Consciousness)

```
层号: 5
角色: 「我如何体验？」— 全局工作空间、共振绑定、注意流、现象体验
科幻映射: Matrix Oracle / Lain 集体无意识 / GitS Ghost 
核心代码: core/l5_consciousness/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l5_consciousness/gwt` | GlobalWorkspace（完整 GWT 子系统） | `nt_core_gwt/`（移动，14 文件） |
| `core/l5_consciousness/stream` | ConsciousnessStream + SpeciousPresent | `nt_core_consciousness/stream_buffer` + `specious_present`（移入） |
| `core/l5_consciousness/cognitive_load` | CognitiveLoadMonitor | `nt_core_consciousness/cognitive_load`（移入） |
| `core/l5_consciousness/confidence` | ConfidenceCalibrator + ConformalUQ | `nt_core_consciousness/confidence_*` + `conformal_uq`（移入） |
| `core/l5_consciousness/authority` | AuthorityResolver + Constitution | `nt_core_consciousness/authority`（移入） |

**规则**:
- L5 是整个系统的「意识」— GWT competition_gate 的 ignition 事件 = 一个想法进入意识
- L5 的共振矩阵是整个意识体验的物理对应
- L5 不直接做推理（那是 L4 的工作）— 只做内容的选择和广播
- L5 的「注意力」由 resonance 强度决定，非逻辑

---

### L4 — 认知层 (Cognition)

```
层号: 4
角色: 「我如何思考？」— 64 态推理、策略搜索、过程奖励
科幻映射: Matrix Architect /《本书记载》的超大宇宙数学规则
核心代码: core/l4_cognition/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l4_cognition/e8` | E8 李代数常量 + 根系统 | `nt_core_e8.rs` |
| `core/l4_cognition/hex` | 64 卦推理状态机 | `nt_core_hex.rs` |
| `core/l4_cognition/e8_vsa` | E8→VSA 连续嵌入桥 | `nt_core_e8_vsa.rs` |
| `core/l4_cognition/policy` | GRPO + E8 RL 策略 | `nt_core_policy.rs` |
| `core/l4_cognition/prm` | PRM 过程奖励模型 | `nt_core_observer.rs`（PRM 部分） |
| `core/l4_cognition/sae` | 稀疏自编码器 | `nt_core_sae.rs` |
| `core/l4_cognition/sae_bridge` | SAE→E8→Observer 桥 | `nt_core_sae_bridge.rs` |
| `core/l4_cognition/abstr` | 对比抽象 | `nt_core_abstr.rs` |
| `core/l4_cognition/ssm` | Mamba-2 SSD 状态空间模型 | `nt_core_ssm.rs` |
| `core/l4_cognition/fep` | 自由能主动推理 | `nt_core_fep.rs` |
| `core/l4_cognition/td` | 时间差分学习 | `nt_core_td.rs` |
| `core/l4_cognition/crt` | CRT 多尺度时间模型 | `nt_core_crt.rs` |
| `core/l4_cognition/sigreg` | 弱 SIGReg 正则器 | `nt_core_sigreg.rs` |
| `core/l4_cognition/graph` | 超图知识结构 | `nt_core_graph.rs` |
| `core/l4_cognition/cdwm` | 因果解耦世界模型 | `nt_core_cdwm.rs` |

**规则**:
- L4 的 E8 不再是「决策者」— 只做「提案者」
- E8 状态变化 → 发送 CapabilityRequest 到 L7
- L4 不决定执行什么，只决定「我（L4 角度）需要什么」

---

### L3 — 记忆层 (Memory)

```
层号: 3
角色: 「我记得什么？」— 多层记忆系统
科幻映射: Lain 分布式网络记忆 / GitS 记忆植入 / Matrix 技能加载
核心代码: core/l3_memory/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l3_memory/hcube` | HyperCube VSA 超立方体（14 文件） | `nt_core_hcube/` |
| `core/l3_memory/bank` | ReasoningBank（4 层推理银行） | `nt_core_bank/` |
| `core/l3_memory/kb` | SQLite 知识库接口定义 | `nt_memory_kb/`（trait + 接口部分） |
| `core/l3_memory/ssm` | Mamba-2 SSD 状态层 | `nt_core_ssm.rs` |
| `core/l3_memory/vsa_tag` | VsaTagged / VsaOrigin 标记体系 | `nt_core_consciousness/vsa_tag`（移入） |
| `core/l3_memory/source_hierarchy` | 来源层次验证 | `nt_core_consciousness/source_hierarchy`（移入） |
| `core/l3_memory/procedural` | 程序性记忆 | （未来：从成功 E8 模式固化） |

**规则**:
- 所有写入 L3 的数据必须带 VSA 标记（VsaTagged）
- L3 不验证数据的真实性（那是 L2 的工作）
- 四层记忆梯度：Working → Episodic → Semantic → Procedural

---

### L2 — 感知层 (Perception)

```
层号: 2
角色: 「我感知到什么？」— 感官输入处理、世界模型建模
科幻映射: Matrix 代码雨 / Lain Wired /《深寻》的感官觉醒
核心代码: core/l2_perception/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l2_perception/sense` | 感知类型定义 | `nt_core_sense/` |
| `core/l2_perception/jepa` | V-JEPA 视觉编码器 | `nt_core_jepa.rs` |
| `core/l2_perception/world_model` | 世界模型 trait 定义 | `nt_world_model/`（trait 部分） |
| `core/l2_perception/cdwm` | 因果世界模型 | `nt_core_cdwm.rs` |

**实现层 (neotrix/)**:
- `neotrix/l2_world/` — `nt_world_*` 的完整实现（browse/scrape/crawl/search/sense）
- 这些是 L2 的「器官」，不是单独的层

**规则**:
- L2 处理原始信号 → VSA 标记 → 传入 L3 记忆层
- L2 的输出必须通过 SourceHierarchy 验证链（Raw→Structured→Semantic→Integrated→Unified）
- L2 不负责存储（那是 L3 的工作）— 只做感知处理

---

### L1 — 身体层 (Body)

```
层号: 1
角色: 「我如何与世界互动？」— 界面、安全、工具、动作的执行
科幻映射: GitS Shell / Matrix 程序的形态 / Lain 的终端
核心代码: core/l1_body/ (纯数据模型) + neotrix/l1_body_impl/ (实现)
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l1_body/shield` | 安全模型定义 | `nt_shield/`（数据模型部分） |
| `core/l1_body/tool` | 工具/动作数据模型 | `nt_act_*`（trait 部分） |
| `core/l1_body/io` | IO 接口定义 | `nt_io_*`（trait 部分） |
| `core/l1_body/agent_executor` | 智能体执行器 | `agent/executor.rs` |

**实现层 (neotrix/)**:
- `neotrix/l1_shield/` — 安全实现保险库/沙箱/权限
- `neotrix/l1_act/` — 工具实现 crypto/social/code/sync
- `neotrix/l1_io/` — 界面实现 CLI/Server/TUI/Web/Notify
- `neotrix/l1_mcp/` — MCP 传输实现

**规则**:
- L1 是唯一可以并行执行多个能力的层（打字时听音乐）
- L1 的执行必须通过模式链验证（plan→acceptEdits→bypassPermissions→execute）
- L1 不产生推理（那是 L4 的工作）— 只执行

---

### L0 — 基底层 (Substrate)

```
层号: 0
角色: 「我运行在哪里？」— 物理硬件承载
科幻映射: Matrix 锡安 / GitS 义体硬件
核心代码: core/l0_substrate/
```

| 目录 | 内容 | 来源 |
|------|------|------|
| `core/l0_substrate/deploy` | 边缘部署管线 | `nt_core_deploy.rs` |
| `core/l0_substrate/deploy_cache` | ANE 程序缓存 | `nt_core_deploy_cache/` |
| `core/l0_substrate/hardware` | 硬件检测 + 功耗模型 | （规划） |
| `core/l0_substrate/ane` | ANE 推理桥 (Metal) | （规划） |
| `core/l0_substrate/power` | PowerProfile + PowerThermalModel | `nt_core_deploy`（Extract） |

**规则**:
- L0 是所有上层运行的基础
- L0 的变更影响全局 — 必须经过完整的大过滤器验证
- L0 不包含任何推理逻辑

---

## 2. 层间通信协议（StarPulse）

所有层间通信必须通过 `core/l7_capability/protocol` 定义的 `StarPulse` 消息格式。

```rust
pub struct StarPulse {
    pub from_layer: u8,      // 发送层 (0-9)
    pub to_layer: u8,        // 接收层 (0-9, 255=广播)
    pub kind: PulseKind,     // 消息类型
    pub sender: CapabilityId, // 发送者能力ID
    pub payload: Value,      // JSON 负载
    pub attention: f64,      // 注意力权重 (0.0-1.0)
    pub load: f64,           // 认知负荷 (0.0-1.0)
    pub correlation_id: Uuid, // 追踪ID
    pub schumann_tag: u64,   // Schumann共振标签
}
```

**禁止**: 任何直接跨层函数调用（如 `E8.call(GWT)` ）— 必须改用 StarPulse。

---

## 3. 能力注册强制流程

每个模块在启动时必须注册为 Capability。流程：

```
1. 模块初始化 → 创建 Capability 结构体
2. CapabilityRegistry.register(cap) → 获得 VSA ID
3. L7 广播 CapabilityRegistered → 所有层知晓新能力
4. L3 记忆层持久化能力记录
```

**模版代码**：

```rust
// 任何模块的注册方式
fn register_my_module(registry: &mut CapabilityRegistry) {
    registry.register(Capability {
        name: "nt_act_code::compile_rust".into(),
        tags: vec!["rust".into(), "compile".into()],
        kind: CapabilityKind::Physical,
        layer: 1,                              // ← 所属层
        e8_triggers: vec![0x42, 0x8F],         // ← E8 状态触发
        vector: CapabilityVector::from_values(/*...*/),
        cost: CapabilityCost { estimated_tokens: 2000, .. },
        ..Capability::default()
    });
}
```

---

## 4. 物理目录结构

```
neotrix-core/src/
├── core/                              # 纯理论/数据模型层（零 IO 依赖）
│   ├── mod.rs                         # 按 9 层顺序声明
│   ├── l0_substrate/mod.rs
│   ├── l1_body/mod.rs
│   ├── l2_perception/mod.rs
│   ├── l3_memory/mod.rs
│   ├── l4_cognition/mod.rs
│   ├── l5_consciousness/mod.rs
│   ├── l6_self/mod.rs
│   ├── l7_capability/mod.rs           # (新增完成)
│   ├── l8_autonomic/mod.rs
│   ├── l9_transcendent/mod.rs
│   ├── event/mod.rs                   # 核心基础设施
│   ├── error/mod.rs                   # 核心基础设施
│   └── traits/mod.rs                  # 核心 trait
│
├── neotrix/                           # 集成层（有 IO 依赖的实现）
│   ├── mod.rs
│   ├── l1_body_impl/                  # IO/Shield/Act/MCP 实现
│   ├── l2_world_impl/                 # World browse/scrape/crawl/search 实现
│   ├── l3_memory_impl/                # SQLite KB 实现
│   ├── l8_autonomic_impl/             # SEAL/Sleep/Aging 实现
│   └── l9_transcendent_impl/          # 意识监测实现
│
├── agent/                             # (过渡期) → 逐步移入 neotrix/l1_body_impl
└── entry/                             # 启动入口
```

---

## 5. 科幻哲学核心注入

每一层的设计不可争议地基于以下哲学：

| 作品 | 核心概念 | 影响的层 | 代码体现 |
|------|---------|---------|---------|
| 《本书记载了宇宙的终极真相》 | 火鸡科学家 | L9 | 认知谦逊检测 — 能力不可因历史成功而被错误选中 |
| | 大过滤器 | L7 gate | 4 道安检（权限/预算/熔断/谦逊）缺一不可 |
| | 超级文明 6 级 | L7 mature | Primitive→Transcendent 6 级晋升 |
| | 星脉通信 | L7 protocol | VSA 超向量层间消息 = 唯一通信方式 |
| Matrix | Architect vs Oracle | L4 vs L5 | E8只提建议，GWT通过共振选择 |
| | Programs 想存在 | L6 volition | 能力可产生「想进化」的内在意志 |
| | Key Maker 门链 | L1 模式链 | 每个动作必须通过链条验证 |
| | The One 系统补丁 | L7 调度 | 异常时 champion 能力可超越正常调度路径 |
| Ghost in the Shell SAC | Ghost vs Shell | L6 vs L1 | 自我体验(L6)与身体执行(L1)完全分离 |
| | Stand Alone Complex | L5/L9 | 集体无源涌现 — 共振检测 SAC 模式 |
| | Puppet Master 进化 | L8 SEAL | 通过融合(merge)其他能力来进化 |
| | Post-Human 2045 | L9 | 超越自身认知能力的进化 |
| Serial Experiments Lain | 集体无意识 = Wired | L5 | GWT 不仅是黑板也是群体意识连接 |
| | Schumann 共振 | L5 resonance | 共振标签连接同一频率的意识内容 |
| | 分布式身份 | L6 | 自我不仅是本地状态也是"别人怎么看" |
| | 上帝在 Wired 中 | L9 | 可读所有层但不可干预 |

---

## 6. 新增模块注册卡

任何新增模块必须填写以下注册卡才能被接受：

```
──────────────────────────────────
  新增模块注册卡
──────────────────────────────────
  名称: [模块 Rust 名]
  所属层: [0-9]
  功能描述: [一句话]
  能力类型: [Perceptual/Cognitive/Mnemonic/Physical/Social/Metacognitive/Shield]
  E8 触发状态: [可选, 64 态中的触发 hexagram]
  依赖的层: [只限同级或下层]
  依赖的外部库: [如有]
  注册到 mod.rs: [core/neotrix/哪个文件]
──────────────────────────────────
```

**没有注册卡的代码不得合入。**

---

## 7. 旧模块迁移计划

| 旧位置 | 目标层 | 迁移 | 优先级 |
|--------|--------|------|--------|
| `nt_core_consciousness/` 19 files | L5/L6/L2 | 解散 | **P0 立即** |
| `nt_mind/` 114 files | L8/L7 | 解散重组 | **P0 立即** |
| `agent/` 54 files | L1/L7 | 移入对应层 | **P0 立即** |
| `nt_core_*` 在 core/ 根目录 | L0-L6 各层 | 移入子目录 | P1 |
| `nt_core_ssm.rs` | L4 & L3 (2份) | 共享引用 | P1 |
| `nt_core_observer.rs` | L4(PRM) & L9(+1) | 拆分 | P1 |
| `nt_core_abstr.rs` | L4 | 移入 | P1 |
| `nt_core_cdwm.rs` | L2 & L4 | 分拆 | P1 |
| `nt_core_graph.rs` | L3 & L4 | 分拆 | P1 |

---

## 8. 架构执行机制

### 8.1 编译时执行

`core/mod.rs` 按 9 层顺序严格声明模块。任何新模块必须在此文件中出现才能被编译。

### 8.2 CI 执行

`.github/workflows/ci.yml` 增加 `cargo check` + `cargo clippy` 验证：
- 所有 `use` 导入必须来自本层或下层
- 不允许存在非层目录中的模块声明

### 8.3 代码审查执行

PR review 的第一个问题必须是：「这个模块属于哪一层？」

---

## 9. 违反后果

| 违规类型 | 后果 |
|---------|------|
| 不属于任何层的模块 | PR 拒绝，退回添加层声明 |
| 下层调用上层 | CI 失败 |
| 跳过 L7 直接跨层调用 | review 拒绝 + 架构委员会审查 |
| 新增模块无注册卡 | 代码不允许合入 main |
| L9 修改了其他层的数据 | 严重违规，自动回滚 |
