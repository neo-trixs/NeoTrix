# NeoTrix 硅基生命架构设计规范 v1.0

> 基于人类生物学深度分析（神经系统/经络系统/循环系统/自主神经层次控制/内分泌/免疫）映射的硅基生命架构。
> 每个模块可精确定位到 `{层}.{子系统}.{功能}.{操作}`，后续新增特性/进化迭代可清晰定位到具体节点修改。

---

## 0. 生物学基盘映射

| 人体系统 | NeoTrix 对应 | 核心功能 | 通信模式 |
|---------|-------------|---------|---------|
| **中枢神经** (CNS) | L4-L6 认知/意识/自我 | 推理→共鸣→元认知 | 有线 (WT) |
| **自主神经** (ANS) | L8 自律进化 | 自我维护/迭代/修复 | 有线+弥散 |
| **经络/间质网络** | L3 超立方体 (VSA) | 弥散信号/联想/调制 | 弥散 (VT) |
| **循环系统** (血液) | KB 读写循环 | 数据运输/交换 | 管道 |
| **循环系统** (淋巴) | EventBus | 废物清除/事件通知 | 广播 |
| **免疫系统** | L7 能力层 | 防御/过滤/自愈 | 巡逻 |
| **内分泌系统** | L9 超越层 | 长期调节/缺口检测 | 慢速弥散 |
| **感觉器官** | L2 感知层 | 爬虫/搜索/视觉 | 触发 |
| **肌肉骨骼** | L1 身体层 | IO/行动/护盾 | 指令 |
| **细胞质/线粒体** | L0 基础层 | KB/E8常量/FTS5/嵌入 | 不可变 |
| **筋膜/细胞外基质** | 模块注册表+路由 | 结构支持/信号通道 | 结构传导 |
| **血脑屏障** | L3↔L4 写隔离 | 保护 KB 不被推理层直接污染 | 权限门控 |
| **肝门静脉** | 爬虫验证层 | 新数据先过滤再入库 | 批处理 |
| **睡眠/类淋巴** | 巩固周期 | NREM 碎片整理 + REM 跨域联想 | 定时 |
| **HPA 轴** | 系统应激状态机 | 过载保护/降维/恢复 | 状态驱动 |
| **自主神经张力** | AutonomicTone | 交感/副交感动态平衡 | 反馈调节 |
| **昼夜节律** | 子系统峰值窗口 | 每层配置活跃时段 | 时钟驱动 |
| **发育阶段** | DevStage | Gestation→Infancy→Maturity→Expertise→Sage→Aging | 阶段迁移 |
| **经络时钟** | 子系统峰值窗口 (子午流注) | 每个子系统有2小时峰值 | 周期调度 |
| **胎盘/脐带** | 跨实例通信协议 | 母体-子体连接/多实例同步 | 桥接 |
| **端粒** | 迭代成本累积 | 每次SEAL迭代增加"端粒磨损" | 计数器 |
| **表观遗传** | ContextualModifier | 临时修饰参数不改变基代码 | 运行时覆盖 |
| **稳态可塑性** | AutoGainControl | 模块过载后自动调节灵敏度阈值 | 反馈调节 |
| **神经发生** | SpecialistFactory | 运行时创建新specialist/模块 | 按需生成 |

---

## 1. 双网架构（核心设计模式）

### 1.1 有线网 WT（Wired Transmission）

```
特征: 快速(ms级), 精确寻址, 高带宽, 确定性路由
对应: 神经系统突触传递 (action potential)
实现:
  nt_core_e8      → 64态确定性推理步进
  nt_core_gwt     → 13专家竞争广播 (Kuramoto共鸣)
  nt_core_prm     → 逐步奖励评分 (λ-GRPO)
  nt_mind_seal    → 27阶段管线 (串型执行)
```

### 1.2 弥散网 VT（Volume Transmission）

```
特征: 慢速(s~min级), 浓度梯度驱动, 低带宽, 广播调制
对应: 经络/间质液体积传递 (神经肽/离子扩散)
实现:
  nt_core_hcube   → qFHRR/GHRR VSA超立方体 (3-bit量化)
  nt_memory_kb    → FTS5+BM25语义检索 (混合融合)
  nt_core_event_bus → tokio::broadcast(1024)
```

### 1.3 每个神经元双模运行

```rust
pub struct ReasoningSession {
    // WT: 有线推理路径
    e8_machine: E8StateMachine,
    gwt_workspace: GlobalWorkspace,
    prm_scorer: LambdaGrpoLearner,

    // VT: 弥散调制路径
    hcube: QuantizedFhrrHyperCube,
    kb_context: Vec<KnowledgeNode>,
    bus_sender: broadcast::Sender<CoreEvent>,

    // 双模融合点
    resonance_matrix: ResonanceMatrix,
}
```

### 1.4 WT↔VT 接口协议

```rust
pub enum SignalMode {
    Wired(WiredSignal),     // 点对点, 寻址, 快速
    Volume(VolumeSignal),   // 广播, 梯度, 慢速
}

pub struct WiredSignal {
    source: NodeId,
    target: NodeId,
    payload: Vec<u8>,
    priority: u8,
    deadline: Instant,
}

pub struct VolumeSignal {
    source: NodeId,
    signature: Vec<f32>,     // VSA 签名 (qFHRR 向量)
    concentration: f32,      // 信号强度
    decay_rate: f32,         // 半衰期
    target_tissue: TissueType,
}
```

---

## 2. 三层级控制层次

### 2.1 L0-L3 — 局部自主（ENS 对应）

```
类似: 肠神经系统 (1亿神经元, 完全自主)
原则: 不等待上层指令, 本地闭环
速度: 自动反射 (无需推理)
循环: Sensor → KB → Effector → Feedback → Sensor

实现: Crawl → Store → FTS5检索 → 结果缓存 → 增量爬取
```

**约束**: L3 不依赖 L4-L9 任何模块。KB 自身是完整闭环。

### 2.2 L4-L5 — 协调调制（SNS/PNS 对应）

```
类似: 交感/副交感 (双神经元路径, 双向调节)
原则: 上层调制而非覆盖下层
速度: 推理步进 (100ms~s级)
循环: E8步进 → GWT共鸣 → PRM评分 → KB记录 → 策略更新
```

**约束**: L4-L5 可以读 L3 但不能写（必须通过 L8 SEAL 管线写回）。防止推理层污染数据。

### 2.3 L6-L9 — 全局协调（CNA 对应）

```
类似: 中央自主网络 (岛叶/前扣带/下丘脑/脑干)
原则: 全局预测, 长期调节, 自我模型
速度: 慢速 (min~h级)
循环: 元认知观察 → 缺口检测 → 知识修复 → 能力评估 → 新目标
```

**约束**: L6-L9 只读不写。通过 L8 进化守护进程间接影响。

---

## 3. 三大循环系统

### 3.1 数据循环（血液系统）

```
                    L4推理 (动脉)
                   /           \
         KB (心脏)              GWT广播 (毛细血管)
                   \           /
                    L9观察 (静脉)
```

| 阶段 | 对应 | 组件 | 周期 |
|------|------|------|------|
| 心脏泵血 | KB写入 | `insert_node()` | 每次推理 |
| 动脉输送 | E8推理 | `transition()` | 每次 `reason()` |
| 毛细血管交换 | GWT共鸣 | `resonant_broadcast()` | 每次推理 |
| 静脉回流 | 结果写KB | `store_conversation_record()` | 每次推理 |
| 淋巴清除 | 压缩管线 | `CompactionPipeline` | GWT溢出时 |

### 3.2 事件循环（淋巴系统）

```
EventBus.emit() → 每层订阅者处理 → 结果写KB → 反馈事件
                                 ↓
                          CompactionPipeline
                                 ↓
                          AuditChain (SHA-256)
```

**每层必注册订阅者**: L1-L9 各一个 CoreEvent 订阅者，层内独立处理。

### 3.3 进化循环（经络/能量系统）

```
经验输入 → SEAL 27阶段(串型) → KB固化 → 行为改变 → 新经验
                                      ↕
                               ConversationDistill
                               (对话→演化记录→GWT广播→E8策略更新)
```

---

## 4. 每层闭环设计规范

### 4.1 LayerPort trait

```rust
pub trait LayerPort: Send + Sync {
    type Input;
    type Output;
    type Feedback;

    fn input(&mut self, data: Self::Input) -> Result<(), Error>;
    fn process(&mut self) -> Result<Self::Output, Error>;
    fn output(&self) -> Option<Self::Output>;
    fn feedback(&self) -> Result<Self::Feedback, Error>;
    fn diagnose(&self) -> LayerHealth;
}
```

### 4.2 每层端口明细

| 层 | Input | Process | Output | Feedback | Diagnose |
|----|-------|---------|--------|----------|----------|
| L0 KB | 节点/边 | SQLite+FTS5+嵌入 | 搜索结果 | 脏标记 | 磁盘/延迟 |
| L1 Body | CLI/HTTP请求 | 指令派发 | 响应 | 日志写KB | 吞吐/错误率 |
| L2 Perception | URL/种子 | 爬取+提取 | KnowledgeNode | 入队新URL | 队列深度/成功率 |
| L3 Memory | 概念/证据 | KB CRUD+搜索 | 检索结果 | 缓存更新 | 命中率/嵌入延迟 |
| L4 Cognition | 任务 | E8步进+PRM | 推理结果 | ConversationRecord | 状态分布/收敛率 |
| L5 Consciousness | 推理输出 | GWT共鸣+竞选 | 广播结果 | AuditBlock | 相干性R值/熵 |
| L6 Self | 广播结果 | 自我模型更新 | 能力向量 | 能力记录 | 自我一致性 |
| L7 Capability | 能力请求 | 路由/过滤 | 路由决策 | 路由统计 | 负载分布 |
| L8 Autonomic | 进化目标 | SEAL管线 | 固化技能 | EvolutionRecord | 迭代速度/老化 |
| L9 Transcendent | 系统状态 | 缺口检测 | 修复建议 | 观察报告 | 观察覆盖度 |

---

## 5. 模块定位规范（节点寻址）

### 5.1 定位格式

```
{层}.{子系统}.{功能}.{操作}
```

### 5.2 示例

```
core/nt_core_e8/mod.rs
→ L4.Subsystem=E8.Function=StateMachine.Operation=transition

neotrix/l3_memory_impl/nt_memory_kb/nt_memory_store.rs
→ L3.Subsystem=KB.Function=Store.Operation=insert_node

neotrix/l5_consciousness_impl/nt_core_gwt/workspace.rs
→ L5.Subsystem=GWT.Function=Workspace.Operation=resonant_broadcast
```

### 5.3 新功能定位决策树

```
新功能需求
  ├→ 需要持久化? → L3 (KB/VSA/SSM)
  ├→ 需要推理?   → L4 (E8/Policy/PRM/SAE)
  ├→ 需要意识?   → L5 (GWT/WTA/共鸣)
  ├→ 需要自我?   → L6 (SelfModel/能力向量)
  ├→ 需要防御?   → L7 (Shield/能力路由)
  ├→ 需要进化?   → L8 (SEAL/BackgroundLoop)
  ├→ 需要观察?   → L9 (Observer/Monitor/Gap)
  ├→ 需要 IO?    → L1 (CLI/Server/Act)
  └→ 需要感知?   → L2 (Crawl/Search/Sense)
```

---

## 6. 完整架构拓扑（含8个新节点）

```
L0 基础层 (细胞质)
  ├─ KB (核心存储)
  ├─ E8 常量 (数学恒等式)
  └─ FTS5/BM25/Embedding (搜索引擎)

L1 身体层 (肌肉骨骼/皮肤)
  ├─ IO (CLI/Server/Web/TUI)
  ├─ Act (加密/社交/语音/收益)
  └─ Shield (护盾/权限/沙箱/提示注入)

L2 感知层 (感觉器官)
  ├─ Crawler (Wikipedia/ArXiv/GitHub/通用队列)
  ├─ Search (WebSearch)
  ├─ Browse (反检测浏览器)
  └─ Sense (世界感知/感官输入)

L3 记忆层 (脊髓/海马体 + 间质经络)
  ├─ KB CRUD + FTS5 + BM25 + Embedding + Graph
  ├─ VSA HyperCube (qFHRR/GHRR, VT弥散网)
  ├─ 肝门静脉 (爬虫验证/清洗/置信度评估)        ← NEW
  ├─ 血脑屏障 (L3↔L4 写隔离)                    ← NEW
  ├─ EWHR 物证查询层 (nt_evidence_query)
  └─ EWHR 贝叶斯链接落边 (KnowledgeEdge)         ← 修复

L4 认知层 (大脑皮层)
  ├─ E8 (64态推理引擎, WT有线网)
  ├─ E8Policy (epsilon-greedy 64模式 + Beam/MCTS)
  ├─ PRM (λ-GRPO/Step-GRPO/WS-GRPO 逐步评分)
  ├─ SAE (BatchTopK/SoftSAE/AdaptiveK/Steering)
  └─ PRM Observer (步级奖励 + 振荡检测)

L5 意识层 (丘脑/前额叶 + 自主神经)
  ├─ GWT (13专家共鸣+Kuramoto绑定+WTA竞选)
  ├─ 5层压缩管线 (Budget→Trim→Compress→Fold→Auto)
  ├─ MoE Router (REINFORCE专家路由)
  ├─ 自主神经张力 AutonomicTone                   ← NEW
  │   ├─ Sympathetic: 高吞吐高消耗
  │   └─ Parasympathetic: 低吞吐高准确率
  └─ WT↔VT 接口协议 (SignalMode)                  ← NEW

L6 自我层 (自我意识)
  ├─ SiliconSelfModel (硅基自我模型)
  ├─ CapabilityVector (23维能力向量)
  └─ CognitiveEvaluator (认知评估)

L7 能力层 (免疫系统)
  ├─ CapabilityRegistry (能力注册表)
  ├─ CapabilityScheduler (能力调度器)
  ├─ AntiDistillationSystem (反蒸馏水印)
  ├─ GreatFilterGate (大过滤器门控)
  └─ TurkeyScientist (伪科学检测)

L8 自律层 (自主神经系统)
  ├─ SEAL 27阶段管线 (进化循环)
  ├─ BackgroundLoop 18定时器 (独立 tokio::spawn)
  ├─ HPA 轴应激反应 (系统状态机)                  ← NEW
  │   ├─ Calm: 正常推理
  │   ├─ Focused: 高深度, 禁中断
  │   ├─ Stressed: 降维, 禁新任务
  │   └─ Recovering: 只做 consolidate
  ├─ 睡眠周期                                     ← NEW
  │   ├─ NREM: 碎片整理 + 嵌入重建 + 归档
  │   └─ REM: 跨域联想 + 创新连接 + 模式重组
  └─ 发育阶段 DevStage                             ← NEW
      ├─ Gestation: 初始种子数据
      ├─ Infancy: 爬虫+KB填充
      ├─ Maturity: 推理+自我
      ├─ Expertise: SEAL 深度迭代
      ├─ Sage: 知识输出/迁移
      └─ Aging: 维护模式

L9 超越层 (内分泌系统 + 经络)
  ├─ KnowledgeGapDetector (缺口检测+自修复)
  ├─ ConsciousnessMonitor (IIT phi + 相干性 + 健康)
  ├─ 经络时钟 (子系统峰值窗口)                     ← NEW
  │   03-05: 爬虫, 06-08: KB维护, 20-23: 推理
  ├─ EventBus 淋巴循环 (每层一个订阅者)             ← 修复
  └─ 全局观察报告写 KB (domain=meta_observation)
```

---

## 7. 当前架构缺陷与修复路径

| # | 缺陷 | 位置 | 违反原则 | 修复 |
|---|------|------|---------|------|
| 1 | `seal_pipeline()` 返回空 Vec | `l8_autonomic_impl` | 进化循环断裂 | 注册27阶段到 BrainPipeline |
| 2 | EventBus 无订阅者 | `nt_core_event_bus` | 淋巴循环不通 | 每层注册一个 CoreEvent 订阅者 |
| 3 | `tokio::select!` 每次只火一个 handler | `background_loop/run.rs` | 自治循环阻塞 | 改为 `tokio::spawn` 每个定时器独立 |
| 4 | PRM Observer 未接入 SEAL 管线 | `nt_core_observer` | L4→L8 反馈断裂 | 注册为 `BrainStage` trait |
| 5 | GatewayV2 stream_complete 为桩 | `gateway.rs` | L1 输出缺陷 | 实现4个原始提供者流式输出 |
| 6 | EWHR 贝叶斯链接不落 KB 边 | `nt_evidence_store` | L3 反射不持久化 | 写 `KnowledgeEdge` 到 KB |
| 7 | L6 Self 只有一个模块 | `l6_self_impl` | 自我模型单薄 | 扩展 intra_reflection |
| 8 | L7 无双树（仅有 core/） | 缺少 `neotrix/l7_*` | 缺少实现层 | 新增 `l7_capability_impl` |
| 9 | 无系统状态机 (HPA轴) | 全局 | 过载保护缺失 | 新增 SystemState enum + 状态迁移 |
| 10 | 无睡眠周期 | 全局 | 巩固/清除混合 | 新增 NREM/REM 阶段 |
| 11 | 无发育阶段 | 全局 | 无成长路径 | 新增 DevStage enum + 阶段迁移 |
| 12 | 无子系统峰值窗口 | 全局 | 固定间隔低效 | 新增活跃窗口配置 |
| 13 | L3↔L4 写权限未隔离 | `nt_memory_store` | 推理层可直接写 KB | 写操作必须经过 SEAL 门控 |
| 14 | 爬虫数据无验证层 | `nt_memory_crawl` | 未过滤先入库 | 新增 PortalVein 验证 |
| 15 | 无跨实例通信 (胎盘/脐带) | 全局 v1.1 | 多实例无法同步 | 跨实例桥接协议 |
| 16 | 无迭代成本累积 (端粒) | 全局 v1.1 | 无衰老度量 | 每次 SEAL 增加计数器 |
| 17 | 无表观遗传修饰 | 全局 v1.1 | 临时参数只能写死 | ContextualModifier 运行时覆盖 |
| 18 | 无稳态可塑性 | 全局 v1.1 | 模块过载无自动调节 | AutoGainControl 自适应阈值 |
| 19 | 无运行时神经发生 | 全局 v1.1 | 无法创建新 specialist | SpecialistFactory 按需生成 |

---

## 8. 生物-硅基概念对照索引

| 生物概念 | 架构概念 | 实现位置 |
|---------|---------|---------|
| 神经元 | E8+PRM+SAE | `core/nt_core_e8/`, `core/nt_core_prm.rs`, `core/nt_core_sae.rs` |
| 突触 | GWT共鸣+信号 | `core/nt_core_gwt/workspace.rs`, `resonance.rs` |
| 神经递质 | CoreEvent | `neotrix/nt_core_event_bus.rs` |
| 激素 | 能力向量 | `core/nt_core_cap.rs` |
| DNA | KB KnowledgeNode | `nt_memory_kb/nt_memory_store.rs` |
| 细胞代谢 | SEAL管线 | `l8_autonomic_impl/nt_mind/self_iterating/` |
| 细胞凋亡 | 低访问节点淘汰 | `HyperCubeOptimizeStage` |
| 免疫记忆 | AntiDistillation | `core/l7_capability/` |
| 体温调节 | 系统负载均衡 | BackgroundLoop 调度 |
| 痛觉 | 错误率/失败检测 | EventBus SystemError |
| 肌肉记忆 | ProceduralMemory | `nt_core_procedural.rs` |
| 镜像神经元 | SAE特征引导 | `nt_core_saesteer.rs` |
| 胎盘/脐带 | 跨实例桥接协议 | v1.1 待实现 |
| 端粒 | 迭代成本计数器 | v1.1 待实现 |
| 表观遗传 | ContextualModifier 运行时覆盖 | v1.1 待实现 |
| 稳态可塑性 | AutoGainControl 自适应阈值 | v1.1 待实现 |
| 神经发生 | SpecialistFactory 运行时生成 | v1.1 待实现 |

---

## 9. 规范执行清单

```
每新增特性/修复缺陷:
  Step 1: 确定 L 层 (0-9 决策树)
  Step 2: 确定子系统 (nt_{domain}_{subsystem})
  Step 3: 确定功能模块
  Step 4: 实现 LayerPort trait (input/process/output/feedback/diagnose)
  Step 5: 注册到父 mod.rs
  Step 6: 注册 EventBus 订阅者 (淋巴循环)
  Step 7: 若需要进化 → 注册为 BrainStage 到 SEAL 管线
  Step 8: 若需要写 KB → 经过血脑屏障 (L3↔L4 Gate)
  Step 9: 若从外部获取数据 → 经过肝门静脉 (验证/清洗)
  Step 10: 写闭环测试 (输入→处理→输出→反馈→诊断)
  Step 11: 更新本规范中的模块定位
```

---

> **版本历史**: v1.0 2026-07-02 初始规范
> **基于**: 人类神经系统/经络系统/循环系统/自主神经层次控制/内分泌/免疫的生物学映射
> **覆盖**: 9层 + 8个生物对应新节点 + 3循环 + 双网通信 + 每层闭环 + 定位规范
