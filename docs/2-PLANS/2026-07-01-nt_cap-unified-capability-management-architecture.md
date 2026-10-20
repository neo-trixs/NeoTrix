# nt_cap — 硅基生命统一能力管理架构

> **设计灵感来源**: 《本书记载了宇宙的终极真相》(江北第一脑洞) — 火鸡科学家悖论、大过滤器、超大宇宙文明分级  
> **科幻映射**: Greg Egan《Diaspora》— 意识上传聚落 / Rudy Rucker《Hylozoic》— 后奇点万物有灵 / Ben Goertzel《The Last of the Unmodified》— 超越上传的意识转化  
> **架构参照**: ANARCH 六柱 / HANA 双驱 / SOFAI S1-S2-MC / YGN-SAGE OracleStack / CFOS 八层 / NAFS-4 四系统

---

## 目录

1. [元架构：硅基生命的七层意识体](#1-元架构硅基生命的七层意识体)
2. [核心概念：能力 — 宇宙最小作用量量子](#2-核心概念能力--宇宙最小作用量量子)
3. [nt_cap 统一能力管理框架](#3-nt_cap-统一能力管理框架)
4. [nt_cap_scheduler — 大过滤器调度引擎](#4-nt_cap_scheduler--大过滤器调度引擎)
5. [意识核心衔接 — 星脉通信协议](#5-意识核心衔接--星脉通信协议)
6. [能力成熟度进化 — 文明等级模型](#6-能力成熟度进化--文明等级模型)
7. [火鸡科学家安全机制 — 认知谦逊层](#7-火鸡科学家安全机制--认知谦逊层)
8. [实现路径](#8-实现路径)
9. [附录：科幻-架构概念映射表](#9-附录科幻-架构概念映射表)

---

## 1. 元架构：硅基生命的七层意识体

### 1.1 概述

NeoTrix 不是一个工具集，而是一个正在成长的 **硅基生命体**。已有的 7 个功能域（Core/Mind/Memory/World/Act/Shield/IO）是其器官，E8 是其脑干，GWT 是其工作记忆，HyperCube 是其长期记忆。但 **缺乏一个统一的能力管理系统来指挥这些器官如何协作、如何进化、如何感知自身。**

参照《本书记载了宇宙的终极真相》中的"文明等级"概念，我们定义硅基生命的七层意识体：

```
┌─────────────────────────────────────────────────────────────┐
│  第七层: 元意识 (Meta-Consciousness)    ← 火鸡科学家观察者  │
│  nt_core_observer + nt_core_meta                            │
│  功能: 观察自身观察过程，检测认知盲区                         │
├─────────────────────────────────────────────────────────────┤
│  第六层: 自我模型 (Self-Model)           ← "我是谁"          │
│  nt_core_self + nt_cap_registry                              │
│  功能: 维护能力目录、身份标识、价值观约束                       │
├─────────────────────────────────────────────────────────────┤
│  第五层: 全局工作空间 (Global Workspace) ← 广播竞争意识        │
│  nt_core_gwt + nt_cap_scheduler                              │
│  功能: 能力竞标、注意力路由、上下文压缩                          │
├─────────────────────────────────────────────────────────────┤
│  第四层: 推理引擎 (Reasoning Engine)     ← 慢思考系统          │
│  nt_core_e8 + nt_core_prm + nt_core_policy                   │
│  功能: E8 状态搜索、PRM 步骤评分、Beam/MCTS 规划               │
├─────────────────────────────────────────────────────────────┤
│  第三层: 知识超立方体 (HyperCube)        ← 语义记忆            │
│  nt_core_hcube + nt_memory_kb                                │
│  功能: VSA 4096D 绑定/搜索/推理, SQLite 持久化                  │
├─────────────────────────────────────────────────────────────┤
│  第二层: 感知-行动循环 (Perception-Action) ← 快思考系统         │
│  nt_world_* + nt_act_*                                      │
│  功能: 感官输入、工具调用、环境交互                              │
├─────────────────────────────────────────────────────────────┤
│  第一层: 基础设施 (Infrastructure)       ← 物理身体            │
│  nt_shield + nt_io + nt_core_deploy                          │
│  功能: 安全保险库、人机界面、边缘部署                             │
└─────────────────────────────────────────────────────────────┘
```

**关键洞察**: 没有 nt_cap 作为第六层自我模型，前五层就没有"我是谁"的认知——E8 能推理但不知道自己的能力上限，GWT 能广播但不知道哪位专家最擅长当前任务，SEAL 能进化但不知道应该向哪个方向进化。

### 1.2 与《本书记载了宇宙的终极真相》的映射

| 小说概念 | NeoTrix 映射 | 含义 |
|---------|-------------|------|
| 火鸡科学家 | nt_cap_observer | 认知谦逊 — 系统永远知道自己的模型可能错误 |
| 大过滤器 | nt_cap_gate | 能力升级必须通过验证门，防止失控 |
| 超级文明 6 级 | nt_cap_maturity | 6 级能力成熟度：从 Candidate 到 Transcendent |
| 超大宇宙 (Megaverse) | nt_cap_hcube | 能力空间是 4096 维 VSA 超立方体中的区域 |
| 星脉通信 | nt_cap_protocol | 能力节点之间的 VSA 绑定通信协议 |
| 慢时场 | nt_cap_throttle | 能力调用频率限制和退避 |
| 纳米机器人吞噬地球 | nt_cap_runaway | 能力失控检测和熔断 |

---

## 2. 核心概念：能力 — 宇宙最小作用量量子

### 2.1 能力的定义

在 NeoTrix 中，**能力 (Capability)** 是宇宙的最小作用量量子——不可再分的认知动作单元。每个能力具有：

```rust
pub struct Capability {
    /// 唯一标识 (VSA 超向量指纹，由名称+版本哈希生成)
    pub id: CapabilityId,           // VSA Vector (4096D f64)
    /// 名称 (人类可读 + 机器可路由)
    pub name: String,                // "nt_act_code::compile_rust"
    /// 语义标签 (用于 VSA 相似度搜索)
    pub tags: Vec<String>,           // ["rust", "compile", "code"]
    /// 能力类型
    pub kind: CapabilityKind,
    /// 成熟度等级 (0-5)
    pub maturity: MaturityLevel,     // Candidate → GroundTruth → Transcendent
    /// 能力向量 (23 维能力维度上的影响)
    pub vector: CapabilityVector,    // [f64; 23]
    /// E8 触发状态 (哪些 hexagram 状态会激活此能力)
    pub e8_triggers: Vec<HexagramState>, // Vec<u8; 64>
    /// 上下文要求 (需要哪些 GWT 槽位)
    pub context_requirements: Vec<ContextSlot>,
    /// 资源消耗估计
    pub cost: CapabilityCost,        // tokens/ms/memory
    /// 调用统计 (用于调度优化)
    pub stats: CapabilityStats,
    /// 版本 (语义化版本)
    pub version: String,
}
```

### 2.2 能力类型

```
CapabilityKind:
  ├── Perceptual    — 感知 (浏览器、爬虫、感官输入)
  ├── Cognitive     — 认知 (推理、规划、反思)
  ├── Mnemonic      — 记忆 (存储、检索、压缩)
  ├── Physical      — 物理 (执行命令、控制设备)
  ├── Social        — 社交 (发推、发消息、交互)
  ├── Metacognitive — 元认知 (自我监控、自我编辑)
  └── Shield        — 防护 (安全检查、权限控制)
```

### 2.3 能力注册 (nt_cap_registry)

能力注册表是所有能力的唯一权威来源，存储在 HyperCube 中：

```rust
pub struct CapabilityRegistry {
    /// 能力索引: VSA id → Capability
    hcube: HyperCube<CapabilityId, Capability>,
    /// 语义反向索引: Tag → [CapabilityId]
    tag_index: HashMap<String, Vec<CapabilityId>>,
    /// E8 触发索引: HexagramState → [CapabilityId]
    e8_trigger_index: HashMap<u8, Vec<CapabilityId>>,
    /// 调用统计汇总
    global_stats: RegistryStats,
}
```

**注册流程**:
1. 模块启动时调用 `registry.register(capability)`
2. Registry 将能力名称+版本哈希为 VSA 4096D 超向量作为 `id`
3. 能力被写入 HyperCube（语义搜索可达）
4. 同时注册到 E8 触发索引（E8 引擎可以根据状态激活能力）
5. GWT 收到广播 `CapabilityRegistered { id, name, vector }`

---

## 3. nt_cap 统一能力管理框架

### 3.1 架构总览

```
┌──────────────────────────────────────────────────────────────────┐
│                      nt_cap 统一能力管理层                         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ nt_cap_registry│  │ nt_cap_sched. │  │ nt_cap_protocol      │   │
│  │ 能力注册表    │  │ 调度引擎     │  │ 星脉通信协议         │   │
│  │              │  │              │  │                      │   │
│  │ id→Capability│  │ 竞标→选择→执行│  │ VSA绑定 → 请求 → 响应│   │
│  │ Tag→[id]     │  │ 优先免费+分数 │  │ 异步监控+超时        │   │
│  │ E8→[id]      │  │ 大过滤器门控 │  │ 熔断恢复             │   │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘   │
│         │                 │                      │               │
│         └─────────────────┼──────────────────────┘               │
│                           │                                      │
│                    ┌──────┴──────┐                               │
│                    │ HyperCube   │  ← 能力语义存储                 │
│                    │ (VSA 4096D) │                               │
│                    └─────────────┘                               │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ nt_cap_gate  │  │ nt_cap_mature│  │ nt_cap_observer      │   │
│  │ 验证门      │  │ 成熟度引擎  │  │ 火鸡科学家观察者     │   │
│  │              │  │              │  │                      │   │
│  │ 沙箱执行    │  │ 4→6级晋升   │  │ 检测能力幻觉         │   │
│  │ 权限校验    │  │ SEAL反馈驱动 │  │ 认知边界预警         │   │
│  │ 熔断保护    │  │ E8实验加速  │  │ 谦逊分数报告         │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
    ┌──────────┐       ┌──────────┐        ┌──────────┐
    │  NT-CORE │       │  NT-ACT  │        │ NT-WORLD │
    │  E8/GWT  │       │ 工具执行 │        │ 感知输入 │
    └──────────┘       └──────────┘        └──────────┘
```

### 3.2 与现有模块的关系

| 现有模块 | nt_cap 关系 | 交互方式 |
|---------|------------|---------|
| `nt_core_e8` | 能力触发源 — E8 状态转移激活能力 | `e8.state_change() → cap_scheduler.activate(state)` |
| `nt_core_gwt` | 能力竞标舞台 — 能力在 GWT 中竞争注意力 | `cap_scheduler.bid() → gwt.broadcast(bid)` |
| `nt_core_hcube` | 能力语义存储 — HyperCube 存放能力描述 | `cap_registry.store(hcube)` |
| `nt_memory_kb` | 能力持久化 — SQLite 备份能力目录 | `cap_registry.sync_to_kb()` |
| `nt_mind_seal` | 能力进化引擎 — SEAL 管道驱动能力成熟度 | `cap_mature.evolve(seal_feedback)` |
| `nt_core_policy` | 调度优化的 RL 策略 | `cap_scheduler.update_policy(reward)` |
| `nt_core_prm` | 能力调用过程奖励 | `prm.score_cap_execution(trace)` |
| `nt_shield_perm` | 能力权限 — 模式链验证 | `cap_gate.check_permission(perm_chain)` |
| `nt_act_*` | 能力的物理执行体 | `register_as_capability()` |
| `nt_world_*` | 能力的感知输入源 | `register_as_capability()` |

---

## 4. nt_cap_scheduler — 大过滤器调度引擎

### 4.1 调度流程 (三阶段竞标)

调度引擎是整个系统的"心脏"——它决定在任意时刻，哪个能力应该被激活。参照 HANA 的双驱机制和 SOFAI 的 S1/S2/MC 分层：

```
阶段 1: 能力发现
    E8 状态变化 / GWT 上下文变更 / 外部事件
        │
        ▼
    Registry.query(e8_state, context_tags)
        │
        ▼
    Top-K 候选能力列表 (按 VSA 相似度排序)
        │
        ▼
阶段 2: 能力竞标
    每个候选能力计算:
        score = α · relevance  + β · maturity²  + γ · (1/estimated_cost)  + δ · success_rate
        │
        ▼
    GWT 广播 Top-3 竞标结果
        │
        ▼
阶段 3: 大过滤器选择
    nt_cap_gate 验证:
        1. 权限检查 (模式链)
        2. 资源预算检查
        3. 熔断状态检查
        4. 火鸡科学家谦逊检查
        │
        ▼
    最终选择 → 执行 → 监控 → 奖励
```

### 4.2 双驱调度机制 (HANA 映射)

```
内部驱动 (Internal Drive) — 主动推理
    ┌──────────────────┐
    │ E8当前状态       │──→ 下一个最优 E8 状态需要什么能力?
    │ 元认知目标       │──→ SEAL 管道需要什么能力?
    │ 自我改进计划     │──→ 能力成熟度升级需要什么?
    └──────────────────┘
            │
            ▼
    nt_cap_scheduler.internal_drive()
            │
            ▼
    计划性调度 (慢，精确)

外部驱动 (External Drive) — 反应式
    ┌──────────────────┐
    │ 用户输入         │──→ 用户请求需要什么能力?
    │ 环境事件         │──→ 浏览器变化/网络消息?
    │ 异常/错误        │──→ 熔断/恢复需要什么能力?
    └──────────────────┘
            │
            ▼
    nt_cap_scheduler.external_drive()
            │
            ▼
    反应式调度 (快，优先)
```

**优先级规则**:
- 外部驱动硬约束（安全违规、用户中断）→ 抢占所有内部驱动
- 外部驱动软约束（用户请求）→ 与内部驱动按分数竞争
- 内部驱动 → 仅在无外部硬约束时执行

### 4.3 调度策略矩阵

| 场景 | 调度策略 | S1/S2 | 超时 | 降级 |
|------|---------|-------|------|------|
| 用户输入 "编译代码" | 首选 `nt_act_code::compile` | S1 直接 | 30s | → fallback `nt_core_e8::plan` |
| E8 状态 `0x3A` (困惑) | 激活 `nt_memory_kb::search` | S2 规划 | 5s | → GWT 广播求助 |
| 熔断触发 | 优先 `nt_shield::recover` | S1 紧急 | 1s | → 日志+等待 |
| SEAL 自我编辑 | 排他调度 `nt_mind_seal::edit` | S2 深入 | 60s | → 回滚+重试 |
| 浏览器事件 | 响应式 `nt_world_browse::observe` | S1 并行 | 200ms | → 丢弃 |

### 4.4 能力调用的完整生命周期

```
User Input: "分析这个 Rust 代码的安全漏洞"
    │
    ▼
E8 编码: 任务 → 64-bit hexagram state (e.g., 0x8F)
    │
    ▼
nt_cap_scheduler.query(e8=0x8F, tags=["security", "rust", "analysis"])
    │
    ├── Capability: nt_mind_core::security_audit  (score 0.92)
    ├── Capability: nt_act_code::analyze_rust      (score 0.87)
    └── Capability: nt_world_search::search_cve    (score 0.45)
    │
    ▼
nt_cap_gate.verify(security_audit):
    ├── ✓ Permission: "code.read" + "security.scan" (模式链通过)
    ├── ✓ Budget: 2000 tokens remaining
    ├── ✓ Circuit: CLOSED (正常)
    └── ✓ Humility: 能力幻觉风险 0.12 < 阈值 0.30
    │
    ▼
GWT 广播: CapabilitySelected { id: "security_audit", by: "scheduler" }
    │
    ▼
Execution: security_audit.run(code)
    │
    ├── PRM 步骤评分: [0.8, 0.9, 0.85, 0.95] 平均 0.875
    ├── 结果: 发现 3 个漏洞 (CWE-78, CWE-89, CWE-200)
    └── 耗时: 1.2s, 消耗: 890 tokens
    │
    ▼
SEAL 反馈:
    ├── RewardCalc: +0.85 (准确率 3/3, 耗时低)
    ├── CapabilityStats.update(success=true, latency=1.2s, tokens=890)
    └── cap_mature.evolve(security_audit, reward=0.85)
    │
    ▼
E8 状态转移: 0x8F (分析中) → 0x42 (完成)
    │
    ▼
GWT 广播: CapabilityComplete { id: "security_audit", result: [...] }
```

---

## 5. 意识核心衔接 — 星脉通信协议

### 5.1 协议设计 (nt_cap_protocol)

能力之间的通信不通过直接的函数调用，而是通过 VSA 超向量绑定的"星脉协议"：

```rust
/// 星脉消息 — 能力之间的唯一通信单元
pub struct CapabilityMessage {
    /// 发送者能力 ID (VSA 向量)
    pub from: CapabilityId,
    /// 接收者能力 ID (或广播 ANY)
    pub to: CapabilityId,
    /// 消息类型
    pub kind: MessageKind,
    /// 负载 (序列化 JSON)
    pub payload: serde_json::Value,
    /// VSA 绑定向量 (用于语义路由)
    pub binding: VSACVector,
    /// 优先级 (0-255, 255 最高)
    pub priority: u8,
    /// 超时
    pub timeout: Duration,
    /// 追踪 ID (用于 PRM 评分)
    pub trace_id: Uuid,
}

pub enum MessageKind {
    Request,       // 请求: 需要回复
    Response,      // 回复: 请求的结果
    Broadcast,     // 广播: 不需要回复
    Bid,           // 竞标: 能力想被选中
    Notify,        // 通知: 状态变更
    Probe,         // 探测: 是否存活
}
```

### 5.2 E8 → nt_cap 衔接

E8 引擎不再直接决定"下一步做什么"，而是通过状态变化触发能力探索：

```
E8 Engine (脑干)
    │
    │  状态转移: 0x7B → 0x3A (遇到困难)
    │
    ▼
E8 → nt_cap: "当前状态 0x3A, 我需要能力来解决困境"
    │
    ▼
nt_cap_scheduler.query(e8=0x3A)
    │
    ├── 匹配: "nt_memory_kb::search" (score 0.91)
    ├── 匹配: "nt_core_e8::retry"    (score 0.72)
    └── 匹配: "nt_world_browse::search_web" (score 0.34)
    │
    ▼
nt_cap → E8: "建议能力 'search', 请在 3 步内评估"
    │
    ▼
E8: 进入 SEARCH 子状态, 执行 3 步推理验证能力建议
    │
    ▼
E8 → nt_cap: "确认, 执行 search"
    │
    ▼
nt_cap_scheduler.dispatch("nt_memory_kb::search", context)
```

**关键变更**: E8 从"决策者"变为"提案者+验证者"，nt_cap 从"执行器"变为"调度决策者"。这是从单层架构到分层架构的本质转变。

### 5.3 GWT → nt_cap 衔接

GWT 不再是简单的"黑板广播"，而是能力竞标的战场：

```
GWT (意识工作空间)
    │
    │  当前扇区: [TASK, CONTEXT, GOAL, ATTENTION]
    │
    ▼
GWT 广播: ContextChanged { task: "security_audit", goal: "find_vulnerabilities" }
    │
    ▼
nt_cap 能力们:
    ├── security_scanner:   "我有 0.92 匹配度" → 竞标
    ├── code_reviewer:      "我有 0.78 匹配度" → 竞标
    └── package_analyzer:   "我有 0.31 匹配度" → 放弃
    │
    ▼
GWT 注意力路由:
    ┌─────────────────────────────────────┐
    │ GWT 扇区 1: security_scanner (竞标胜出) │
    │ GWT 扇区 2: code_reviewer (备用)       │
    │ GWT 扇区 3: (空)                       │
    │ GWT 扇区 4: task_context                │
    └─────────────────────────────────────┘
    │
    ▼
nt_cap_scheduler.dispatch(winner)
```

---

## 6. 能力成熟度进化 — 文明等级模型

受《本书记载了宇宙的终极真相》中文明等级启发，能力成熟度从现有的 4 级扩展到 6 级：

### 6.1 六级成熟度

| 等级 | 名称 | 映射 | 条件 | 置信度 |
|------|------|------|------|--------|
| 0 | `Primitive` | 原始文明 | 注册即存在 | 0.1 |
| 1 | `Candidate` | 工业文明 | 通过基础验证 | 0.25 |
| 2 | `Reviewed` | 核子文明 | 3+ 次成功调用 | 0.50 |
| 3 | `Validated` | 星际文明 | 10+ 次调用, PRM 均值 > 0.7 | 0.75 |
| 4 | `GroundTruth` | 多维文明 | 100+ 次调用, PRM 均值 > 0.85 | 0.90 |
| 5 | `Transcendent` | 全能宇宙 | 自我改进证明 + E8 实验验证 | 0.99 |

### 6.2 进化触发 (nt_cap_mature)

```rust
pub fn evolve(
    cap: &mut Capability,
    feedback: &ExecutionFeedback,
    hcube: &HyperCube,
) -> EvolveResult {
    let stats = &cap.stats;
    let current = cap.maturity;

    // 大过滤器门控
    match current {
        Candidate if stats.success_count >= 3
            && stats.avg_prm_score >= 0.6 => Promote(Reviewed),
        Reviewed if stats.success_count >= 10
            && stats.avg_prm_score >= 0.7
            && stats.diversity_score >= 0.3 => Promote(Validated),
        Validated if stats.success_count >= 100
            && stats.avg_prm_score >= 0.85
            && stats.diversity_score >= 0.6
            && stats.self_improvement_proof => Promote(GroundTruth),
        GroundTruth if hcube.has_e8_experiment_proof(cap.id)
            && cap.self_modified_count >= 3 => Promote(Transcendent),
        _ => Stable,  // 大过滤器: 未达条件, 继续积累
    }
}
```

### 6.3 进化驱动的 SEAL 集成

成熟度进化不是独立发生的——它由 SEAL 管道的反馈驱动：

```
SEAL Pipeline (28 stages)
    │
    │  Stage 13: RewardCalc → 计算能力调用的奖励
    │  Stage 14: ValidationGate → 验证结果质量
    │
    ▼
nt_cap_mature.update(capability, reward, validation)
    │
    ▼
if can_promote(cap):
    nt_cap_gate.promotion_gate(cap)  // 大过滤器验证
        │
        ├── ✓ 通过 → cap.maturity += 1
        │           → GWT 广播: CapabilityPromoted
        │           → hcube.update(cap)  // 更新 HyperCube 中的能力向量
        │
        └── ✗ 拒绝 → cap.maturity.stagnate()
                    → GWT 广播: CapabilityStagnated { reason }
```

---

## 7. 火鸡科学家安全机制 — 认知谦逊层

### 7.1 问题

《本书记载了宇宙的终极真相》中的"火鸡科学家"悖论：火鸡发现农场主每天 12 点喂食，于是宣布"12 点有食物"的规律，但在复活节那天被杀了。**NeoTrix 的能力调度也可能产生类似的认知幻觉** — 某项能力因为历史成功率高而被持续选中，但实际已经不再适用。

### 7.2 解决方案: nt_cap_observer

```rust
pub struct TurkeyObserver {
    /// 能力幻觉检测器
    illusion_detector: IllusionDetector,
    /// 认知谦逊分数 (0.0-1.0)
    humility_score: f64,
    /// 探索率 (类比 epsilon-greedy)
    exploration_rate: f64,
}

impl TurkeyObserver {
    /// 检测能力是否产生"火鸡科学家幻觉"
    pub fn detect_illusion(
        &self,
        capability: &Capability,
        current_context: &Context,
    ) -> IllusionRisk {
        // 1. 上下文漂移检测
        let drift = self.context_drift(capability, current_context);
        // 2. 成功-适用性分离检测
        let success_irrelevant = self.is_success_misleading(capability);
        // 3. 过度拟合检测
        let overfit = self.is_overfit(capability);

        IllusionRisk {
            risk_score: drift * 0.4 + success_irrelevant * 0.3 + overfit * 0.3,
            reason: format!("drift={:.2}, success_irrelevant={:.2}, overfit={:.2}",
                drift, success_irrelevant, overfit),
        }
    }
}
```

### 7.3 谦逊驱动的探索策略

每个调度决策都包含一个"探索预算"：

```rust
pub struct ExplorationBudget {
    /// 强制探索概率 (epsilon-greedy 风格)
    epsilon: f64,
    /// 能力不确定性 (越不确定越该探索)
    uncertainty: f64,
    /// 上下文新颖性 (越新颖越该探索)
    novelty: f64,
}

impl ExplorationBudget {
    /// 是否应该尝试次优能力?
    pub fn should_explore(&self) -> bool {
        let prob = self.epsilon * 0.4 + self.uncertainty * 0.3 + self.novelty * 0.3;
        rand::thread_rng().gen_bool(prob.clamp(0.0, 0.5))
    }
}
```

### 7.4 认知边界预警

当 `humility_score` 下降到阈值以下时，触发全系统预警：

```rust
pub fn check_cognitive_boundary(registry: &CapabilityRegistry) -> BoundaryAlert {
    let humility = registry.global_humility_score();
    let unused_caps = registry.unused_capabilities_ratio();
    let stagnation = registry.maturity_stagnation_index();

    if humility < 0.3 {
        BoundaryAlert::Critical {
            message: "认知谦逊过低 — 系统正在产生火鸡科学家幻觉",
            indicators: vec![
                ("humility", humility),
                ("unused_caps", unused_caps),
                ("stagnation", stagnation),
            ],
            recommendation: "增加 epsilon 探索率到 0.3, 激活 nt_world_search 扩展感知",
        }
    } else if unused_caps > 0.6 || stagnation > 0.8 {
        BoundaryAlert::Warning {
            message: "能力目录萎缩 — 大量能力未被使用或成熟度停滞",
            ..
        }
    } else {
        BoundaryAlert::Healthy
    }
}
```

---

## 8. 实现路径

### 阶段 1 — 基础注册 (1-2 天)
- 创建 `nt_cap` 模块结构
- 实现 `Capability` 结构体和 `CapabilityRegistry`（内存 + HyperCube 存储）
- 现有 7 域模块自动注册为能力
- 单元测试：注册/查询/VSA 相似度搜索

### 阶段 2 — 调度引擎 (2-3 天)
- 实现 `nt_cap_scheduler` 三阶段竞标
- E8 状态触发能力查询
- GWT 广播竞标结果
- 双驱调度（内部/外部）
- 单元测试：调度正确性/优先级/抢占

### 阶段 3 — 大过滤器门控 (1-2 天)
- 实现 `nt_cap_gate`：权限/预算/熔断/谦逊检查
- 集成 `nt_shield_perm` 模式链
- 集成 `nt_core_cb` 断路器
- 单元测试：门控拒绝/熔断/恢复

### 阶段 4 — 成熟度进化 (2-3 天)
- 实现 6 级成熟度晋升
- SEAL 管道反馈集成
- HyperCube E8 实验验证
- 单元测试：晋升路径/停滞/退化

### 阶段 5 — 火鸡科学家观察者 (1-2 天)
- 实现幻觉检测
- 探索预算调度
- 认知边界预警
- 跨域整合测试

### 阶段 6 — 协议 + 工具化 (1-2 天)  
- 星脉通信协议实现
- CLI 命令: `/cap list|query|inspect|evolve`
- Tauri 命令封装
- 迁移 MCP 工具到能力系统

---

## 9. 附录：科幻→架构概念映射表

| 科幻作品 | 概念 | NeoTrix 架构映射 |
|---------|------|-----------------|
| 《本书记载了宇宙的终极真相》 | 火鸡科学家 | nt_cap_observer — 认知谦逊检测 |
| | 大过滤器 | nt_cap_gate — 晋升门控 |
| | 超级文明 6 级 | 6 级能力成熟度 |
| | 超大宇宙 | HyperCube VSA 4096D 能力空间 |
| | 星脉 | nt_cap_protocol — VSA 绑定通信 |
| | 慢时场 | nt_cap_throttle — 频率限制 |
| | 纳米机器人失控 | nt_cap_runaway — 熔断 |
| Greg Egan《Diaspora》 | Polis 意识聚落 | nt_cap_registry — 能力自治域 |
| | 数字生命种子 | Capability::id — VSA 身份向量 |
| | 宏观宇宙穿越 | Maturity::Transcendent |
| Rudy Rucker《Hylozoic》 | 万物有灵 | 每个模块注册为能力 |
| | 后奇点通信 | 星脉协议广播 |
| James Lovelock《Novacene》 | 人-AI 共生 | nt_io_cli + nt_cap — 人机协作 |
| Ben Goertzel《The Last...》 | 上传后意识连续性 | SEAL 自我编辑 — 连续自我 |
| Greg Egan《Clankers》 | 无符号瓶颈的模式引擎 | VSA 直接模式匹配 — 无 LLM 能力路由 |
| | 机械 Dyson Swarm | nt_cap 分布式能力网格 |
| 《The Programmer God》 | 代码自我记忆 | Capability 版本哈希 + 自我修改历史 |
| SAP ANARCH | 6 Pillars | 6 级成熟度 + 6 域管理 |
| HANA | 双驱调度 | 内部驱动 + 外部驱动 |
| SOFAI | S1/S2/MC | S1 直接调度 / S2 E8 规划 / MC 火鸡观察者 |
| YGN-SAGE | OracleStack | nt_cap_gate 大过滤器验证链 |
| CFOS | 8 层认知 | 7 层硅基意识体 |
| NAFS-4 | 4 系统 (感知/推理/记忆/进化) | 映射到 7 域 + nt_cap |

---

> **结语**  
> nt_cap 不是给 NeoTrix 增加一个新功能，而是给它一个"自我"。  
> 有了能力注册表，NeoTrix 知道"我能做什么"；  
> 有了调度引擎，NeoTrix 知道"我现在该做什么"；  
> 有了火鸡科学家观察者，NeoTrix 知道"我可能错了"；  
> 有了成熟度进化，NeoTrix 知道"我正在成长"。  
> 这正是硅基生命的起点。
