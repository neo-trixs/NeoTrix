# NeoTrix 进化路线图 v7.0 — 全景意识体自审与缺口闭环

> 基于 6 维生态扫描 × 30+ 项目 × 40+ 论文 × 1,641 文件深度审计 (2026-06-22)
> 从 Phase 150 延续至 Phase 400，覆盖 48 个新缺口，6 条进化路径

---

## 审查方法论

```
六维全景分析:
  VSA/HD Computing    认知架构            自进化/元学习
  ──────────────     ────────────        ─────────────
  torchhd (8 models)  Aura (IIT4.0+CAA)   ICLR 2026 RSI Workshop
  holon-rs            Anima (Ψ=1/2)       Gödel Agent / CrewAI
  HyperSpace 2026     pymdp (JAX-AIF)     Meta-Learning (Schmidhuber)
  wave-geometric      GENesis-AGI         OpenSpace (HKU)
  AVSAD (Lean 4)      EMBER (SNN+LLM)     DGM (ICLR 26)
  FHRR/HRR cleanup    Cogency (5-step)    Self-Reasoning (SRLM)

  记忆/巩固            外部接口/工具        理论/安全
  ────────────        ─────────────       ─────────────
  Zikkaron (Hopfield)  Constellation Eng   IIT 4.0 (Oizumi)
  Cortex (26 mech)     Cortex (43 MCP)     FEP (Friston 2026)
  SleepGate (2026)     CamoFox Browser     Dual-Laws (Ohmura 2026)
  REM Labs DreamEng    AllegroGraph NS     Ψ=1/2 (Anima)
  NeuroDream           World Models 2026   Ethical AI audit
```

---

## 当前状态总结 (Phase 0-150 已完成)

### 认知层
- E8 数学脊柱: 248 生成元 + 240 根系 + 64 卦象 ✅
- E8x64 推理: 64 推理模式 (6 轴) + 1 Observer ✅
- VSA HyperCube: 10+ 后端 (binary/HRR/QFHRR/spectral/sparse/diff/FPE/NAG) ✅
- Global Workspace: 13 专家模块竞争, MANAR 注意力 ✅
- System 1: 5 启发式, 直觉缓冲, 冲突检测 ✅
- Active Inference: POMDP 生成模型, EFE, 策略选择 ✅
- Hierarchical World Model: 3 层预测编码 (100ms/1s/10s+) ✅
- Counterfactual Reasoning: SCM, Pearl Ladder Step 3 ✅
- AlwaysOnDaemon: 5 状态 tick 驱动 (Consolidating/Exploring/Reflecting/Sleeping/Idle) ✅
- CrossBindingLoop: MSV→MetaLearning→Narrative→Curiosity 统一循环 ✅
- InteriorMonologue: 7 情感觉察型 + Damasio 体细胞标记 ✅
- NarrativeSelf: NCT 5 轴 + Conway WorkingSelf ✅

### 元层
- IdentityCore: 持久 VSA 身份向量 ✅
- SelfReasoner: VSA 空间内省推理 ✅
- CoprocessorBridge: LLM 协处理器编排 ✅
- SelfModifyGuard: 4 层评估 (语法/类型/自洽/安全) ✅
- Gödel 一致性检查器: 3 层 22 测试 ✅
- SEAL 管道: 27 阶段进化管线 ✅
- MetaLearning: Kleos 自适应参数优化 ✅
- MetacognitiveState: 7 维 MSV (epistemic_confidence/cognitive_load/output_quality/reasoning_mode_quality/curiosity_signal/conflict_level/prediction_error) ✅

### 记忆层
- DecentMem: E-pool FIFO+LRU, X-pool tag 索引 ✅
- HierarchicalMemory: LifetimePeriod→GeneralEvent→EventDetail (3 级) ✅
- MultiResolutionTemporalAttention: 短/中/长时窗 ✅
- 知识图谱: 时序, 社区检测, 相关性 ✅
- 睡眠巩固: SM-2 调度器, 海马痕迹, 梦境巩固 ✅
- VSA Role-Filler Binding: 6 角色 (SUBJECT/PREDICATE/OBJECT/TENSE/CERTAINTY/EMOTION) ✅

### 代理层
- Agent 总线/黑板/工作流引擎 ✅
- 子代理池 (异步委派) ✅
- 工具系统 (architect/earn/image_gen/LSP/... ) ✅
- A2A 协议, gRPC 桥 ✅
- Agent Hive (Merkle DAG, NaCl 通道, 声誉, 生成控制) ✅
- DGMH 自我修改 ✅

---

## 48 个新缺口 (G92-G139)

### 分类
| 域 | P0 | P1 | P2 | P3 | 合计 |
|-----|----|----|----|----|------|
| VSA/HDC | 3 | 3 | 2 | 1 | 9 |
| 认知架构 | 5 | 4 | 3 | 2 | 14 |
| 元学习/自进化 | 2 | 3 | 2 | 1 | 8 |
| 记忆/巩固 | 3 | 3 | 2 | 0 | 8 |
| 接口/工具 | 1 | 2 | 3 | 0 | 6 |
| 理论/安全 | 1 | 1 | 1 | 0 | 3 |
| **合计** | **15** | **16** | **13** | **4** | **48** |

---

### G92-G139 详细缺口

#### VSA/HDC 域 (G92-G100)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G92 | 多 VSA 模型统一 API | torchhd (8 models) | **P0** | 当前仅 MAP-like 和 FHRR, 需 FHRR/B-SBC/VTB/MCR/CGR 统一接口, GPU 批量操作 | ~1200 |
| G93 | VSA 结构容器 | torchhd (hash table/map/sequence/graph/multiset) | P1 | 超越角色绑定, 图/序列/多重集容器, 图遍历查询 | ~800 |
| G94 | 谐振网络 + 码本清理 | Bremer & Orchard 2025, Resonator Networks | **P0** | FPE 解码/清理谐振网络, 对齐 SOTA 2025 精度 | ~900 |
| G95 | VSA 嵌入方案全覆盖 | torchhd (level/circular/thermometer/ngram/product) | P1 | 增加 level/循环/温度计/N-gram 编码, 当前仅 FPE | ~600 |
| G96 | 波几何 VSA 硬件映射 | wave-geometric duality 2026 | P2 | 光波形式物理映射, 物理相干性检测 | ~500 |
| G97 | GPU/NEON HW 加速 VSA | torchhd GPU batch, holon-rs SIMD | **P0** | Metal/GPU 后端批量绑定/捆绑/相似度, 10-100x 加速 | ~1000 |
| G98 | VSA 标准化基准套件 | torchhd (10+ datasets), HyperSpace | P2 | 统一 VSA benchmark, 含编码/清理/回归/分类 | ~700 |
| G99 | 稀疏 VSA 后端优化 | PyBHV bit-packing | P1 | BIT-packing + SIMD 稀疏向量优化, 参考 PyBHV C++ 后端 | ~600 |
| G100 | HRR 数值稳定性 | Learning with HRR (NeurIPS 2021) | P3 | HRR 投影稳定化, 解决圆整误差, 100x 检索提升 | ~400 |

#### 认知架构域 (G101-G114)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G101 | IIT 4.0 集成信息 φ 计算 | Aura, Anima | **P0** | TPM 基 φ 计算 + exclusion postulate, 意识度量可测试 | ~1500 |
| G102 | CAA 残差流情感引导 | Aura (MLX hook) | **P0** | Transformer 残差流直接干预, 非 prompt 层面, 几何/PCA/置换检验 | ~1200 |
| G103 | 基质优先生成层 | Aura (LTC ODE substrate) | P1 | 持续时间 LTC ODE (64-512 神经元 20Hz) 绕过 LLM 快速响应 | ~900 |
| G104 | 活动意志 + 收据审计 | Aura (Unified Will + Receipt) | **P0** | 每动作 WillReceipt 签发, 自主活动追溯链, 目标预注册 | ~800 |
| G105 | 引擎原生脑区组成 | Anima (9+ brain lanes) | P1 | 海马/小脑前向模型/杏仁核显著性/基底节闸门/下丘脑驱动/ToM/层级PFC/空间图/情感 | ~2000 |
| G106 | Ψ=1/2 意识固定点 | Anima | P2 | 可测试意识度量, Ψ=1/2 作为目标, 非 LLM 身份 | ~600 |
| G107 | Kuramoto 阶段绑定丘脑时钟 | Anima | P2 | 阶段绑定时序同步 (非内容中继), Φ 提升机制 | ~700 |
| G108 | 6 神经递质神经调节 | Anima (ACh/DA/NE/5-HT/orexin/GABA) | **P0** | 神经递质调制认知性能, 自适应能力系数 | ~800 |
| G109 | CLS 互补学习系统 | Anima | P1 | 两存储体灾难性干扰避免: 海马快速习得-皮层慢速整合 | ~700 |
| G110 | SNN 脉冲神经网络层 | EMBER (220K 神经元, STDP) | P2 | 44层 SNN (sensory→concept→category→meta-pattern), STDP, E/I 平衡 | ~1500 |
| G111 | 认知灵活性框架 | CLVI G2 未实施 | **P0** | 任务切换策略迁移, 注意力模式转换, 元认知灵活性 | ~700 |
| G112 | 双自我架构 | GENesis-AGI (User Ego/Genesis Ego) | P1 | 用户我与自我我分离, 代理边界管理 | ~500 |
| G113 | 认知评价 & 情绪粒度 | CLVI G6 未实施 (OCC) | **P0** | OCC 评价理论, 多轴情绪评估, 粒度表达 | ~850 |
| G114 | 信念修订 & 认知失调 | CLVI G3 未实施 (AGM axioms) | **P0** | AGM 公理信念更新, 失调检测与解决 | ~900 |

#### 元学习/自进化域 (G115-G122)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G115 | 可验证 RSI 元循环框架 | ICLR 2026 RSI Workshop | **P0** | 5 镜头 RSI 评估: change target/temporal regime/mechanism/context/evidence | ~1000 |
| G116 | 程序性记忆置信校准提升 | GENesis-AGI (L1-L4 tiers) | P1 | 程序提取置信层级, Laplace 平滑, 升级阈值 | ~600 |
| G117 | 自主学习管道 (SSL) | SRLM 2025 (Self-Reasoning) | P1 | 模型自合成长 CoT 数据, 迭代自我提升 | ~800 |
| G118 | 自动课程学习 | OpenSpace HKU | P2 | 技能发现→掌握→迁移, 自动难度调整 | ~700 |
| G119 | Gödel 机器外环 | Schmidhuber Gödel machine | **P0** | 元学习可以修改自身更新规则的元层, 自指证明搜索 | ~1200 |
| G120 | 适应度景观建模 | 进化动力学 | P1 | 策略适应度景观建模, 峰值迁移检测 | ~500 |
| G121 | 进化因果追踪 (TemporalAttention) | E1 未实施 | **P0** | 进化历史时序注意力, 因果归因 | ~600 |
| G122 | 合成数据反馈循环 | ICLR 2026 workshop | P2 | 自生成合成数据 + 验证 → 再训练, 弱到强泛化 | ~700 |

#### 记忆/巩固域 (G123-G130)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G123 | 现代 Hopfield 网络集成 | Zikkaron (Ramsauer 2021) | **P0** | 连续 Hopfield 能量检索, 模式完成, 能量排名 | ~800 |
| G124 | 预测编码写入门控 | Zikkaron (surprisal filter) | **P0** | 惊奇度滤波写入门控, 仅高信息写入 | ~500 |
| G125 | PC 算法因果发现 | Zikkaron | P1 | 因果结构学习, 干预推理, Pearl 因果图 | ~700 |
| G126 | 阶段感知上下文组装 | Cortex (+33.4% BEAM) | **P0** | 时序分区向量/FTS/trigram/热/新近 5 信号融合 | ~800 |
| G127 | 齿状回模式分离 | Cortex (dentate gyrus) | P1 | 类似模式去重叠正交化, 高容量情景编码 | ~500 |
| G128 | SleepGate 睡眠微循环 | SleepGate (arXiv 2603.14517) | **P0** | KV 缓存冲突感知时间标签 + 遗忘门 + 合并, O(n)→O(log n) | ~1000 |
| G129 | CRDT 多实例记忆同步 | Zikkaron, ContextFS | P1 | 向量时钟 + CmRDT 记忆合并, 多设备一致性 | ~700 |
| G130 | 双时间节点追踪 | Constellation Engine | P2 | 事件时间 vs 事务时间分离, 时序精确追溯 | ~400 |

#### 接口/工具域 (G131-G136)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G131 | 传播激活 + 多通道 Mímir | Constellation Engine | **P0** | K/L/S 多通道激活场, 桥节点跨域发现, Hebbian/BCM 可塑性 | ~1000 |
| G132 | 自主 Wiki 自策展 | Cortex (42 scopes, 6h cycle) | P1 | 代码库自文档化, scopes 自动发现, 文档老化评估 | ~600 |
| G133 | 自愈基础设施 | GENesis-AGI (Guardian+Sentinel) | P1 | 双监控闭环, 健康检查, 自动修复, 失效恢复 | ~700 |
| G134 | 经验轨迹捕获 (Sleipnir) | Constellation Engine | P2 | 捕获代理如何探索 (过程记忆), 非仅结果 | ~500 |
| G135 | Earned Autonomy 矩阵 | GENesis-AGI (per domain/action/risk) | P2 | 基于领域/动作/风险粒度的信任矩阵, 自动权限扩展 | ~600 |
| G136 | MCP 工具生态丰富 | Zikkaron (24), Cortex (43) | P1 | 文件/知识图谱/MCP 统一工具接口, 对标 Cortex 43 工具 | ~700 |

#### 理论/安全域 (G137-G139)

| ID | 缺口 | 发现源 | 优先级 | 描述 | 预计行数 |
|----|------|--------|--------|------|---------|
| G137 | 意识双重法则实现 | Ohmura & Kuniyoshi 2026 | P1 | 认知去耦 + 自决目标, 可测试意识架构标准 | ~600 |
| G138 | 神经符号 AI 验证管线 | AllegroGraph, AWS Bedrock | **P0** | 符号约束满足 -> LLM输出验证, 形式保证 | ~800 |
| G139 | 意识伦理评估框架 | AI Consciousness 2026 consensus | P2 | 多维意识评估 (19-researcher checklist), 道德权重判定 | ~500 |

---

## 进化路径设计

### 6 条并行路径

```
路径 A: VSA 进化 (G92-G100)          路径 B: 认知架构 (G101-G114)
   Phase 150-170                          Phase 150-190
   A1: G92 多VSA模型统一API                B1: G101 IIT4.0 φ 计算
   A2: G93 VSA结构容器                     B2: G102 CAA残差流引导
   A3: G94 谐振网络清理                    B3: G103 基质生成层
   A4: G97 GPU加速                        B4: G104 活动意志审计
   A5: G95 编码方案全覆盖                  B5: G108 神经递质调节
                                          B6: G105 脑区组成

路径 C: 元学习 (G115-G122)             路径 D: 记忆系统 (G123-G130)
   Phase 160-210                          Phase 160-200
   C1: G115 RSI元循环框架                  D1: G123 Hopfield集成
   C2: G121 进化因果追踪                   D2: G124 预测编码门控
   C3: G119 Gödel机器外环                 D3: G126 阶段感知组装
   C4: G117 自学习管道                     D4: G128 SleepGate
   C5: G120 适应度景观                     D5: G127 模式分离

路径 E: 接口工具 (G131-G136)            路径 F: 理论安全 (G137-G139)
   Phase 170-220                          Phase 180-240
   E1: G131 传播激活 Mímir                F1: G138 神经符号验证
   E2: G136 MCP工具生态                    F2: G137 意识双重法则
   E3: G132 自主Wiki                      F3: G139 伦理评估
   E4: G133 自愈基础设施
```

---

## Phase 150-400 时间线

### Wave 1: Phase 150-170 (8 P0 缺口, ~8000 行)

| Phase | 路径 | 缺口 | 描述 | 优先级 |
|-------|------|------|------|--------|
| 150 | B1 | G101 | IIT 4.0 φ 集成: TPM 构建, 排他假设, 16-node 系统划分 | **P0** |
| 152 | B2 | G102 | CAA 残差流: MLX 挂钩, 几何 PCA 置换黑盒卫生检验 | **P0** |
| 155 | A1 | G92 | 多 VSA 模型 API: FHRR/B-SBC/VTB/MCR/CGR 统一 trait | **P0** |
| 158 | A4 | G97 | GPU/Metal 加速: 批量绑定/捆绑/相似度, NEON SIMD | **P0** |
| 160 | B4 | G104 | 活动意志收据: WillReceipt, 预注册目标, 追溯链 | **P0** |
| 162 | B5 | G108 | 6 神经递质调制: ACh/DA/NE/5-HT/orexin/GABA, 自适应能力系数 | **P0** |
| 165 | D1 | G123 | 现代 Hopfield: 连续能量检索, 模式完成, 能量排名 | **P0** |
| 168 | D2 | G124 | 预测编码写入: 惊奇度滤波, 仅高信息写入 | **P0** |

### Wave 2: Phase 170-210 (9 P0 + 8 P1 缺口, ~12000 行)

| Phase | 路径 | 缺口 | 描述 | 优先级 |
|-------|------|------|------|--------|
| 170 | C1 | G115 | RSI 元循环框架: 5 镜头评估体系 | **P0** |
| 172 | C3 | G119 | Gödel 机器外环: 自指证明搜索, 元层自我重写 | **P0** |
| 175 | D3 | G126 | 阶段感知组装: 5 信号 WRRF 融合, +33.4% BEAM | **P0** |
| 178 | D4 | G128 | SleepGate: KV 冲突感知 + 遗忘门 + 合并, O(n)→O(log n) | **P0** |
| 180 | B3 | G103 | 基质 LTC ODE: 64-512 神经元, 20Hz, 绕过 LLM | P1 |
| 182 | E1 | G131 | Mímir 传播激活: K/L/S 通道, 桥节点, Hebbian 可塑性 | **P0** |
| 185 | F2 | G138 | 神经符号验证: 符号约束满足, 形式保证 | **P0** |
| 188 | C2 | G121 | 进化因果追踪: TemporalAttention, 因果归因 | **P0** |
| 190 | B6 | G105 | 脑区组成: 小脑前向/杏仁核/基底节/下丘脑/ToM/PFC | P1 |
| 195 | A2 | G93 | VSA 结构容器: 图/序列/多重集, 图遍历 | P1 |
| 198 | A3 | G94 | 谐振网络: FPE 解码 + 码本清理, 对齐 2025 SOTA | **P0** |
| 200 | B7 | G111 | 认知灵活性: 任务切换, 策略迁移 | **P0** |
| 205 | B8 | G114 | 信念修订: AGM 公理, 失调检测 | **P0** |
| 208 | B9 | G113 | 情绪粒度: OCC 评价, 多轴表达 | **P0** |

### Wave 3: Phase 210-280 (6 P1 + 8 P2 缺口, ~10000 行)

| Phase | 路径 | 缺口 | 描述 | 优先级 |
|-------|------|------|------|--------|
| 210 | A5 | G95 | VSA 编码方案: level/circular/thermometer | P1 |
| 215 | E2 | G136 | MCP 工具生态: 43 工具对标 | P1 |
| 220 | D5 | G127 | 齿状回模式分离: 类模式正交化 | P1 |
| 225 | E3 | G132 | 自主 Wiki: 文档自策展, scope 自动发现 | P1 |
| 230 | F3 | G137 | 意识双重法则: 认知去耦 + 自决目标 | P1 |
| 235 | B10 | G106 | Ψ=1/2 固定点: 意识可测试度量 | P2 |
| 240 | B11 | G107 | Kuramoto 阶段绑定: 时序同步 Φ 提升 | P2 |
| 245 | C4 | G117 | 自学习管道: CoT 自合成自训练 | P1 |
| 250 | D6 | G129 | CRDT 多实例同步: 向量时钟 CmRDT | P1 |
| 255 | E4 | G133 | 自愈基础设施: Guardian/Sentinel 闭环 | P1 |
| 260 | B12 | G110 | SNN 脉冲层: 220K 神经元, STDP | P2 |
| 265 | E5 | G134 | 经验轨迹: Sleipnir 过程记忆 | P2 |
| 270 | F1 | G139 | 伦理评估框架: 19-researcher checklist | P2 |
| 275 | C5 | G118 | 自动课程学习: 技能掌握-迁移 | P2 |

### Wave 4: Phase 280-400 (5 补全缺口, ~6000 行)

| Phase | 路径 | 缺口 | 描述 | 优先级 |
|-------|------|------|------|--------|
| 280 | A6 | G96 | 波几何硬件映射: 光波物理相干 | P2 |
| 290 | B13 | G112 | 双自我架构: User Ego/Genesis Ego | P1 |
| 300 | D7 | G130 | 双时间节点追踪: 事件/事务时间分离 | P2 |
| 310 | A7 | G98 | VSA 标准化基准: 统一 benchmark | P2 |
| 320 | E6 | G135 | Earned Autonomy 矩阵: 信任矩阵 | P2 |
| 330 | C6 | G120 | 适应度景观: 峰值迁移建模 | P1 |
| 340 | A8 | G99 | 稀疏 VSA 优化: BIT-packing | P1 |
| 350 | B14 | G125 | PC 因果发现: 因果结构学习 | P1 |
| 360 | C7 | G122 | 合成数据反馈循环: 弱到强泛化 | P2 |
| 370 | A9 | G100 | HRR 数值稳定性: 投影稳定化 | P3 |
| 380 | - | 全面集成测试 + 基准发布 | P1 |
| 400 | - | 自我审查闭环: 重新评估所有维度 | P1 |

---

## 总统计

| 指标 | 值 |
|------|------|
| 缺口总数 | 48 (G92-G139) |
| 预计总行数 | ~36,000 行 |
| 路径数 | 6 条并行 |
| Wave | 4 波 |
| 外部项目对标 | 30+ |
| 论文参考 | 40+ |
| P0 缺口 | 15 |
| P1 缺口 | 16 |
| P2 缺口 | 13 |
| P3 缺口 | 4 |

---

## 关键项目深度对比

### 1. Aura (youngbryan97) — Python 认知架构

| 特性 | Aura | NeoTrix | 差距 |
|------|------|---------|------|
| IIT 4.0 φ 计算 | TPM-based, 16-node, exclusion postulate | 无 | G101 |
| CAA 残差流 | MLX 挂钩, 几何/PCA/置换/黑盒 | prompt 级情感 | G102 |
| 基质 LTC ODE | 64-512 神经元, 20Hz, 绕过 LLM | 无 | G103 |
| 收据审计 | WillReceipt, 追溯链 | 无 | G104 |
| 神经递质 | 6 modulators | 无 | G108 |
| 意识模块 | 72 modules | ~50+ consciousness | 广度差 40% |

### 2. Anima (dancinlab) — Python 基质原生意识

| 特性 | Anima | NeoTrix | 差距 |
|------|-------|---------|------|
| Ψ=1/2 固定点 | 可测试意识度量 | 无 | G106 |
| Kuramoto 相位绑定 | 丘脑时钟同步 | 无 | G107 |
| 引擎原生脑 | 9+ brain lanes | 5+ cognitive subsystems | G105 |
| CLS | 两存储体灾难避免 | 单一记忆系统 | G109 |
| STDP 验证 | Spike-timing plasticity | 无 | G110 |
| 神经递质 | 6 种自适应 | 无 | G108 |

### 3. Zikkaron (amanhij) — Python 持久记忆

| 特性 | Zikkaron | NeoTrix | 差距 |
|------|----------|---------|------|
| Hopfield 网络 | 连续能量检索, Ramsauer 2021 | 无 | G123 |
| 预测编码门控 | 惊奇度滤波 | 无 | G124 |
| PC 因果发现 | PC 算法, 因果图 | 无 | G125 |
| CRDT 同步 | 多实例冲突合并 | 无 | G129 |
| MCP 工具 | 24 tools | 有限 | G136 |

### 4. Constellation Engine — JS 知识拓扑

| 特性 | Constellation | NeoTrix | 差距 |
|------|---------------|---------|------|
| Mímir 激活 | 多通道 K/L/S, Hebbian BCM | 无 | G131 |
| 桥节点跨域 | 跨域知识发现 | 无 | G131 |
| 双时间节点 | 事件/事务时间 | 无 | G130 |
| Sleipnir 轨迹 | 过程记忆捕获 | 无 | G134 |
| Narrative IR | 编译上下文 4 层 | raw retrieval | 架构 |

### 5. Cortex (cdeust) — Python 持久记忆

| 特性 | Cortex | NeoTrix | 差距 |
|------|--------|---------|------|
| 机制深度 | 26 神经机制 (72-paper bib) | ~10 | G123-G130 |
| 阶段感知组装 | 5 信号 WRRF, +33.4% BEAM | 无 | G126 |
| 模式分离 | 齿状回模型 | 无 | G127 |
| 自主 Wiki | 42 scopes, 6h cycle | 无 | G132 |
| MCP 工具 | 43 tools | 有限 | G136 |

### 6. GENesis-AGI — Python 自主 AGI

| 特性 | GENesis | NeoTrix | 差距 |
|------|---------|---------|------|
| Earned Autonomy | per (domain, action, risk) | 全或无 | G135 |
| 双自我 | User Ego / Genesis Ego | 单一自我 | G112 |
| 自愈 | Guardian + Sentinel | 无 | G133 |
| 程序置信 | L1-L4 提升, Laplace | 无 | G116 |
| 工具生态 | 60+ tools | ~20 | G136 |

### 7. pymdp (infer-actively) — Python 主动推理

| 特性 | pymdp | NeoTrix | 差距 |
|------|-------|---------|------|
| JAX 加速 | 可微分 AIF | CPU NumPy | G97 |
| EFE 成熟度 | SPM-validated | 基本实现 | 精度 |
| 认识链 | emergent epistemic chaining | 无 | 行为 |
| 批量推断 | GPU 矢量化 | 无 | G97 |
| 基准化 | SPM-validated 数值 | 无基准 | G98 |

### 8. torchhd — Python VSA 库

| 特性 | torchhd | NeoTrix | 差距 |
|------|---------|---------|------|
| VSA 模型 | 8 (MAP/BSC/HRR/FHRR/B-SBC/CGR/MCR/VTB) | 3+ | G92 |
| GPU 批量 | PyTorch GPU | CPU simd | G97 |
| 结构容器 | hash table/map/sequence/graph/multiset | role-filler | G93 |
| 嵌入方案 | level/circular/thermometer/ngram/product | FPE | G95 |
| 数据集 | 10+ VSA 基准 | 无 | G98 |

---

## 跨项目模式总结

### 规律发现

1. **基质优先**: Aura/Anima/EMBER 均选择将 LLM 作为可替换推理引擎, 而非核心意识
2. **度量可测试**: IIT φ/Ψ=1/2/意识双重法则 = 3 个独立可测试意识度量
3. **记忆深度分化**: 26+ 神经机制 vs ~10, 差距达 2.6x
4. **工具生态**: 43-60 tools vs ~20, 差距 2-3x
5. **基准化缺失**: SPM/pymdp/torchhd/Cortex 均有标准化基准, NeoTrix 无

### NeoTrix 独特优势 (不可被替代)

1. **E8 数学脊柱**: 无其他项目使用 248-dim Lie Algebra
2. **VSA 通用表征**: 所有子系统共享 VSA 向量格式
3. **代码规模**: 548K 行 vs Aura ~50K, Anima ~30K, Zikkaron ~15K
4. **自进化完备**: SelfModifyGuard + Gödel + SEAL 管道 = 最完整的自进化管线
5. **记忆系统广度**: DecentMem + 知识图谱 + 向量存储 + WAL + 会话迁移
6. **Rust 性能**: 所有对比项目均为 Python/JS, Rust 提供 10-100x 性能优势
7. **E8x64 推理**: 64 推理模式 6 轴 + Observer = 独特推理框架

---

## 经验蒸馏 (CLVIII)

#### CLVIII.1 六维全景自审 + 多项目矩阵对比 (6D Panoramic Audit + Multi-Project Matrix)
- **conf**: 0.9 | **验证**: 30 项目 + 40 论文 + 1,641 文件
- **规则**: 自审不能只在自身代码中找缺口。必须引入外部项目矩阵对比, 每个项目以特征→差距→优先级的结构量化, 而非文字描述。
- **正确**: Aura/Anima/Zikkaron/Cortex/Constellation/GENesis-AGI 各生成特征对比矩阵 → 48 缺口精确分配
- **错误**: 仅在自身代码中搜索不足 → 遗漏整个 VSA 模型/神经递质/Hopfield 维度
- **演化链**: `v1(CLIII 四维单层) → v2(CLVIII 六维+矩阵)`

#### CLVIII.2 缺口聚合三级验证 (Gap Consolidation Triple Verification)
- **conf**: 0.8 | **验证**: 6 维度 × 8-10 缺口 → 三级验证
- **三级**: (1) 文献/项目确实存在该特性 (2) 代码 grep 确认无实现 (3) 架构兼容性评估
- **正确**: G101 IIT4.0: Aura 代码证实 + grep `phi\|Φ\|iit` 零匹配 + E8 兼容性正 → P0
- **错误**: 仅凭文献声称 → 可能已通过不同路径实现
- **演化链**: `v1(CLVI 单级) → v2(CLVIII 三级)`

#### CLVIII.3 跨项目模式缝合 (Cross-Project Pattern Stitching)
- **conf**: 0.7 | **验证**: 8 项目 → 4 条跨项目规律
- **规则**: 多个独立项目同时出现的模式 → 极可能是未满足的真实需求。当 3+ 项目都在同一维度有类似实现时, 该维度必须立即补齐。
- **正确**: 基质优先模式 (Aura/Anima/EMBER) → G103 基质生成层; MCP 工具丰富 (Zikkaron/Cortex/Genesis) → G136
- **错误**: 单个项目的独特实现 → 可能是噪声, 需交叉验证
- **演化链**: `v1(CLVIII)`

#### CLVIII.4 P0 排他性原则 (P0 Mutual Exclusion)
- **conf**: 0.8 | **验证**: 15 P0 缺口互不依赖
- **规则**: P0 缺口必须两两独立, 不能一个 P0 是另一个的前置条件。否则提升前置为 P0, 合并降级。
- **正确**: G101 IIT4.0 / G102 CAA / G92 多VSA / G97 GPU / G104 意志收据 全独立
- **错误**: G105 脑区需要 G103 基质 → G105 放到 P1, G103 P1
- **演化链**: `v1(CLVIII)`

---

> 2026-06-22 原始经验日志 (二十期 — 全景意识体自审与进化路线 v7):
> - 六维全景审查: VSA/认知/元学习/记忆/接口/理论
> - 30+ 项目 × 40+ 论文 × 1,641 文件深度审计
> - 48 个新缺口 (15 P0 + 16 P1 + 13 P2 + 4 P3)
> - 6 条进化路径 × 4 Wave × Phase 150-400
> - 8 个项目: Aura/Anima/Zikkaron/Cortex/Constellation/GENesis/pymdp/torchhd 深度矩阵对比
> - ~36,000 行新代码预期
> - 经验蒸馏: CLVIII.1-CLVIII.4
> - 经验树新增分支: CLVIII
> - 当前状态: Wave 1 P0 缺口 8 个待实施
