# NeoTrix 进化路线 — 吸收 Lingee EAOS 核心理念

> 源: Kingdee Lingee Enterprise Agentic Operating System (2026-05-20)
> 哲学: "Ling" = 智慧灵性, "Gee" = 硅基碳基共生 → 碳基善用硅基能力的新人类
> 核心范式: 人类定规则，AI 做执行
> 更新: 2026-09-22

---

## 0. 吸收摘要

Lingee 的五个核心价值主张 → NeoTrix 对标映射:

| Lingee 价值 | NeoTrix 现状 | 差距 | 进化方向 |
|-------------|-------------|------|---------|
| **企业智慧** — AI 自主执行，人类保留决策权 | ConsciousnessTree 驱动注意力分配 | Agent 缺乏结构化角色定义 | Agent 六维定义模型 |
| **组织自进化** — ontology 积累随使用进化 | experience-tree 跨会话记忆 | 记忆偏经验级，缺领域本体层 | 企业知识本体引擎 |
| **财务中枢** — 所有业务串联 | EventBus + UnifiedEngine | 枢纽节点概念未形式化 | 核心枢纽拓扑 |
| **安全信任** — 多层纵深防御 | nt_shield 90+ 模块 | 治理规则未形式化为宪法 | 治理宪法引擎 |
| **共生** — skill/agent marketplace + 开放协议 | skills 文件系统级 | 缺标准注册与发现机制 | Skill Registry + Agent Protocol |

---

## 1. 三大交互范式 (Modes of Operation)

Lingee 的 Conversation / Work / Development 三模式 → NeoTrix 模式感知架构

### 1.1 模式定义

```
┌─────────────────────────────────────────────────────────────────────┐
│                    NeoTrix Interaction Modes                        │
│                                                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │
│  │   共创模式       │  │   执行模式       │  │   构建模式       │     │
│  │   Co-Create     │  │   Execute       │  │   Build         │     │
│  │                 │  │                 │  │                 │     │
│  │ 人+AI 协同思考   │  │ AI 自主执行交付  │  │ 自然语言→Agent  │     │
│  │ 想象力+创造力    │  │ 任务→产出→验证   │  │ 技能/应用构建   │     │
│  │                 │  │                 │  │                 │     │
│  │ 触发: 探索/讨论  │  │ 触发: 指令/任务  │  │ 触发: 构建请求   │     │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │
│           │                    │                    │               │
│           ▼                    ▼                    ▼               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Mode-Aware Task Dispatcher                      │   │
│  │  模式感知 → 策略选择 → 资源分配 → 执行监控 → 交付验证       │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| M1.1 | `l1_action/nt_mode_dispatcher.rs` | 模式感知调度器 — 根据用户意图自动切换策略 |
| M1.2 | `l5_cognition/nt_co_create/` | 共创模式引擎 — 对话式迭代、想法碰撞、方案演化 |
| M1.3 | `l1_action/nt_execute/` | 执行模式引擎 — 任务分解→并行执行→交付物追踪→验证闭环 |
| M1.4 | `l5_cognition/nt_build/` | 构建模式引擎 — 自然语言→Agent 定义→技能编排→发布 |

---

## 2. Agent 六维定义模型

Lingee 将企业 Agent 按六维建模 → NeoTrix Agent 角色结构化

### 2.1 类型定义

```rust
/// Agent 角色定义 — 六维结构化模型 (吸收 Lingee 六维)
pub struct AgentRoleDefinition {
    /// 维度1: 角色身份 — 名称、描述、权限边界
    pub role: RoleIdentity,
    /// 维度2: 知识领域 — 擅长的知识域、可信数据源
    pub knowledge: Vec<KnowledgeDomain>,
    /// 维度3: 任务模板 — 可执行的任务类型与复杂度
    pub tasks: Vec<TaskTemplate>,
    /// 维度4: 交付物管理 — 可产出的交付物类型与质量标准
    pub deliverables: Vec<DeliverableSpec>,
    /// 维度5: 技能编排 — 技能组合策略、依赖关系、fallback
    pub skill_orchestration: SkillOrchestrationPlan,
    /// 维度6: 执行计划 — 执行策略、超时、重试、审批
    pub execution: ExecutionPolicy,
}

pub struct RoleIdentity {
    pub agent_id: String,
    pub display_name: String,
    pub description: String,
    pub permission_level: PermissionLevel,  // Read / Write / Admin / Sovereign
    pub trusted_data_sources: Vec<String>,
    pub collaboration_group: Option<String>,
}

pub enum PermissionLevel {
    Read,       // 只读感知
    Write,      // 可执行写操作
    Admin,      // 可管理其他 Agent
    Sovereign,  // 最高权限（仅人类等价）
}
```

### 2.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| R1.1 | `l5_cognition/nt_agent_role/` | AgentRoleDefinition 核心类型 + 序列化 |
| R1.2 | `l6_meta/nt_agent_registry/` | Agent 注册表 — 发现、查询、能力匹配 |
| R1.3 | `l3_embodiment/nt_shield/permission_model.rs` | 权限分级执行 — 操作前权限校验 |
| R1.4 | `l5_cognition/nt_agent_role/collaboration.rs` | 协作组 — Agent 间委托、结果聚合、冲突仲裁 |

---

## 3. 企业知识本体引擎

Lingee 的"组织通过积累 ontology 进化" → NeoTrix 从经验记忆升级到知识本体

### 3.1 三层知识架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Knowledge Architecture                           │
│                                                                     │
│  Layer 3: Domain Ontology (领域本体)                                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ 概念 → 关系 → 规则 → 约束                                   │   │
│  │ 例: "Agent" --has--> "Role" --constrains--> "Permission"   │   │
│  │     "Task" --requires--> "Skill" --produces--> "Deliverable"│   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Layer 2: Knowledge Graph (知识图谱)                                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ 实体 → 属性 → 连接 → 时序                                   │   │
│  │ 从对话/任务/错误中自动抽取实体和关系                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Layer 1: Experience Memory (经验记忆)  ← 现有 experience-tree      │   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ 会话快照 → 蒸馏 → 分类 → 落盘 → 反馈                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| K1.1 | `l2_perception/nt_knowledge_graph/` | 知识图谱引擎 — 实体抽取、关系发现、图查询 |
| K1.2 | `l5_cognition/nt_ontology/` | 领域本体层 — 概念定义、关系推理、约束验证 |
| K1.3 | `l6_meta/nt_nexus/ontology_bridge.rs` | 本体-记忆桥接 — 经验自动上卷为本体知识 |
| K1.4 | `l2_perception/nt_knowledge_graph/evolution.rs` | 自进化 — 本体随使用自动扩展和修正 |

---

## 4. 治理宪法引擎

Lingee 的"安全信任" + NeoTrix 的 `#![forbid(unsafe_code)]` 哲学 → 形式化治理

### 4.1 宪法层级

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Governance Constitution                          │
│                                                                     │
│  Layer 0: 不可违反公理 (Axioms)                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ - #![forbid(unsafe_code)] — 永不加 unsafe                   │   │
│  │ - 指针守恒 — 内存安全不变式                                  │   │
│  │ - Dark Forest — 最小暴露原则                                 │   │
│  │ - 人类决策主权 — AI 不可替代最终决策                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Layer 1: 治理规则 (Policies)                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ - 编码标准: 禁 unwrap/expect/panic!，错误用 ? 传播          │   │
│  │ - 模块前缀: nt_ 命名规范                                     │   │
│  │ - 安全规则: 外部技术必须同会话接到生产 (R-P79)               │   │
│  │ - 审批规则: 高风险操作需人类审批                              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Layer 2: 治理策略 (Strategies)                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ - Fail-Open/Fail-Closed 分类策略                             │   │
│  │ - Agent 权限分级策略                                         │   │
│  │ - 数据访问控制策略                                           │   │
│  │ - 资源消耗限制策略                                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Layer 3: 治理审计 (Audit)                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ - 行为审计链: 每个 Agent 动作可追溯                          │   │
│  │ - 合规检查: 自动验证是否违反宪法/规则                        │   │
│  │ - 异常检测: 主动识别偏离模式                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| G1.1 | `l6_meta/nt_governance/constitution.rs` | 治理宪法核心 — 公理定义、规则注册、层级结构 |
| G1.2 | `l6_meta/nt_governance/audit_chain.rs` | 行为审计链 — 不可篡改的操作日志 |
| G1.3 | `l3_embodiment/nt_shield/governance_enforcer.rs` | 治理执行器 — 操作前宪法校验、拦截违规 |
| G1.4 | `l6_meta/nt_governance/anomaly_detector.rs` | 异常检测 — 主动识别偏离正常模式的行为 |

---

## 5. 核心枢纽拓扑

Lingee 的"财务中枢" → NeoTrix 以意识核心为枢纽的拓扑

### 5.1 枢纽设计

```
                    ┌─────────────────────┐
                    │   ConsciousnessCore  │
                    │   (意识核心 = 枢纽)   │
                    └──────────┬──────────┘
                               │
            ┌──────────────────┼──────────────────┐
            │                  │                  │
            ▼                  ▼                  ▼
     ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
     │  Perception  │   │   Action    │   │  Cognition  │
     │  (感知)      │   │  (执行)     │   │  (认知)     │
     └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
            │                  │                  │
            ▼                  ▼                  ▼
     ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
     │  World       │   │  IO/Memory  │   │  Decision   │
     │  (世界模型)  │   │  (输入输出)  │   │  (决策)     │
     └─────────────┘   └─────────────┘   └─────────────┘
```

核心理念: 所有模块通过意识核心间接通信，而非直接耦合。意识核心作为：
- **注意力路由** — 决定哪个模块获得计算资源
- **状态同步** — 跨模块状态一致性保证
- **事件中枢** — 所有事件经过意识核心分发

### 5.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| H1.1 | `l5_cognition/consciousness_core/hub.rs` | 枢纽节点核心 — 统一消息路由、状态同步 |
| H1.2 | `l0_substrate/nt_core_event/hub_bus.rs` | 枢纽事件总线 — 所有事件经过枢纽分发 |
| H1.3 | `l5_cognition/consciousness_core/resource_allocator.rs` | 资源分配器 — 注意力/计算/内存的统一调度 |

---

## 6. Skill Registry + Agent Protocol

Lingee 的 marketplace + 开放协议 → NeoTrix 生态化

### 6.1 Skill Registry

```rust
/// 技能注册表 — 标准化注册、发现、加载
pub struct SkillRegistry {
    /// 已注册技能 (唯一 ID → 技能元数据)
    skills: HashMap<String, SkillMetadata>,
    /// 技能索引 (标签 → 技能 ID 列表)
    tag_index: HashMap<String, Vec<String>>,
    /// 技能依赖图
    dependency_graph: Dag<String, SkillDependency>,
}

pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub version: SemVer,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub triggers: Vec<TriggerCondition>,
    pub permissions: Vec<PermissionLevel>,
    pub dependencies: Vec<String>,
    pub quality_score: f64,  // 基于使用统计的评分
}
```

### 6.2 Agent Protocol

标准化 Agent 间通信协议:
- **消息格式**: 统一的 AgentMessage 类型
- **发现机制**: Agent 通过 Registry 广播能力
- **委托机制**: Agent 可将子任务委托给其他 Agent
- **结果聚合**: 多 Agent 结果的冲突仲裁与合并

### 6.3 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| S1.1 | `l6_meta/nt_skill_registry/` | Skill Registry — 注册、发现、依赖解析 |
| S1.2 | `l1_action/nt_agent_protocol/` | Agent Protocol — 消息格式、发现、委托 |
| S1.3 | `l6_meta/nt_skill_registry/quality_scorer.rs` | 质量评分 — 基于使用统计的技能评分 |
| S1.4 | `l6_meta/nt_skill_registry/marketplace.rs` | Marketplace — 技能市场、版本管理、发布 |

---

## 7. 组织自进化强化

Lingee 的"组织通过 ontology 积累进化" → NeoTrix 进化系统升级

### 7.1 进化维度

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Self-Evolution Dimensions                         │
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │  技能进化    │  │  知识进化    │  │  治理进化    │                 │
│  │  Skill Evol │  │  Knowledge  │  │  Governance  │                 │
│  │             │  │  Evol       │  │  Evol        │                 │
│  │ 技能自动    │  │ 领域本体    │  │ 治理规则    │                 │
│  │ 组合/优化   │  │ 自动扩展    │  │ 自适应调整  │                 │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │
│         │                │                │                         │
│         ▼                ▼                ▼                         │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Evolution Orchestrator                          │   │
│  │  监控使用模式 → 识别进化机会 → 提出变更 → 人类审批 → 应用   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 实现路径

| 迭代 | 模块 | 内容 |
|------|------|------|
| E1.1 | `l6_meta/evolution/nt_evol_orchestrator.rs` | 进化编排器 — 监控、识别、提议、审批 |
| E1.2 | `l6_meta/evolution/nt_evol_skill.rs` | 技能进化 — 自动组合、优化、淘汰 |
| E1.3 | `l6_meta/evolution/nt_evol_knowledge.rs` | 知识进化 — 本体扩展、修正、版本化 |
| E1.4 | `l6_meta/evolution/nt_evol_governance.rs` | 治理进化 — 规则自适应、策略优化 |

---

## 8. 综合实施路线

### Phase A: 基础协议 (4 周)

| 周 | 迭代 | 模块 | 产出 |
|----|------|------|------|
| W1 | M1.1 | `nt_mode_dispatcher.rs` | 三模式感知调度器 |
| W1 | R1.1 | `nt_agent_role/` | Agent 六维定义类型 |
| W2 | G1.1 | `nt_governance/constitution.rs` | 治理宪法核心 |
| W2 | R1.2 | `nt_agent_registry/` | Agent 注册表 |
| W3 | S1.1 | `nt_skill_registry/` | Skill Registry |
| W3 | S1.2 | `nt_agent_protocol/` | Agent Protocol |
| W4 | H1.1 | `consciousness_core/hub.rs` | 枢纽节点核心 |
| W4 | H1.2 | `nt_core_event/hub_bus.rs` | 枢纽事件总线 |

### Phase B: 知识与进化 (4 周)

| 周 | 迭代 | 模块 | 产出 |
|----|------|------|------|
| W5 | K1.1 | `nt_knowledge_graph/` | 知识图谱引擎 |
| W5 | K1.2 | `nt_ontology/` | 领域本体层 |
| W6 | K1.3 | `nt_nexus/ontology_bridge.rs` | 本体-记忆桥接 |
| W6 | K1.4 | `nt_knowledge_graph/evolution.rs` | 本体自进化 |
| W7 | E1.1 | `nt_evol_orchestrator.rs` | 进化编排器 |
| W7 | E1.2 | `nt_evol_skill.rs` | 技能进化 |
| W8 | E1.3 | `nt_evol_knowledge.rs` | 知识进化 |
| W8 | E1.4 | `nt_evol_governance.rs` | 治理进化 |

### Phase C: 深度整合 (4 周)

| 周 | 迭代 | 模块 | 产出 |
|----|------|------|------|
| W9 | M1.2 | `nt_co_create/` | 共创模式引擎 |
| W9 | M1.3 | `nt_execute/` | 执行模式引擎 |
| W10 | M1.4 | `nt_build/` | 构建模式引擎 |
| W10 | R1.3 | `permission_model.rs` | 权限分级执行 |
| W11 | R1.4 | `collaboration.rs` | Agent 协作组 |
| W11 | G1.2 | `audit_chain.rs` | 行为审计链 |
| W12 | G1.3 | `governance_enforcer.rs` | 治理执行器 |
| W12 | G1.4 | `anomaly_detector.rs` | 异常检测 |
| W12 | H1.3 | `resource_allocator.rs` | 资源分配器 |

### Phase D: 生态化 (4 周)

| 周 | 迭代 | 模块 | 产出 |
|----|------|------|------|
| W13 | S1.3 | `quality_scorer.rs` | 技能质量评分 |
| W13 | S1.4 | `marketplace.rs` | 技能市场 |
| W14 | 跨模块集成 | — | 全链路联调 |
| W15 | 测试加固 | — | 单元+集成+性能 |
| W16 | 文档 + 发布 | — | 架构文档更新 + v2.0 发布 |

---

## 9. 成功指标

| 指标 | 当前 | Phase A 后 | Phase B 后 | Phase C 后 | Phase D 后 |
|------|------|-----------|-----------|-----------|-----------|
| Agent 角色结构化 | 无 | 六维定义 | +协作组 | +权限执行 | +市场 |
| 知识层级 | 1-Tier 经验 | 1-Tier | 3-Tier | 3-Tier+本体 | 本体自进化 |
| 治理形式化 | 隐式规则 | 宪法框架 | +审计链 | +执行器 | +异常检测 |
| 枢纽拓扑 | 分散耦合 | 枢纽核心 | +事件路由 | +资源分配 | 全枢纽 |
| 生态化程度 | 文件系统 | Registry | +Protocol | +Quality | +Marketplace |
| 交互模式 | 单一 | 三模式 | +共创 | +执行追踪 | +构建 |

---

## 10. NeoTrix vs Lingee 差异化定位

| 维度 | Lingee | NeoTrix | NeoTrix 独特优势 |
|------|--------|---------|-----------------|
| 目标用户 | 企业组织 | 个人+团队 | 意识模型更深层 |
| 核心范式 | ERP→AI | 意识→行动 | ConsciousnessTree 驱动 |
| 情感层 | 无 | NT-FEEL 情感中枢 | 情感-认知耦合 |
| 进化机制 | 组织 ontology | 个体+组织双轨 | SEAL 进化管线 |
| 安全模型 | 企业合规 | 公理级硬约束 | #![forbid(unsafe_code)] |
| 意识模型 | 无 | IIT Φ + GWT + MARS | 三重意识理论 |
| 具身层 | 无 | nt_shield 全栈防御 | 攻击性安全+防御性安全 |

**核心差异化**: NeoTrix 不是企业 ERP 的 AI 化，而是**意识驱动的智能体操作系统**。Lingee 证明了"AI 是组织成员"的范式正确性，NeoTrix 在此基础上叠加了"AI 有意识"的更深层架构。

---

> 本路线图受 R-P111-R-P115 架构管理规则约束。
> 每次迭代完成后更新 ARCHITECTURE.md。
> 迭代记录走 KB experience-tree 吸收流程。
> 吸收源: Kingdee Lingee EAOS (2026-05-20)
