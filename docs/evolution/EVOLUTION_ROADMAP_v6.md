# NeoTrix 全景深度审查与进化路线图 v6.0

> 基准日期: 2026-06-22
> 方法: 60+ 篇论文 + 30+ GitHub 项目 + 4 维并行深度分析 (VSA/自进化/记忆/认知架构)
> 对标: holon-rs, torchhd, vsa-optim-rs, Isildur, Evolver, OpenSpace, Space-Agent, EvoScientist, DGM, Ouroboros, MUE-X, Mem0, TiMem, DecentMem, OpenCog, Nengo/Spaun, pymdp, PyPhi, Cortex-rs, Engram-rs

---

## 零、审查方法

```
四维并行深度分析:
  ┌─────────────────────────────────────────────────────────────┐
  │  VSA/HD Computing    │  自进化/自修改 AI    │  记忆系统      │  认知架构       │
  │  (8 项目+5 论文)     │  (8 项目+10 论文)    │  (8 项目+6 论文) │  (10 项目+8 论文)│
  ├─────────────────────────────────────────────────────────────┤
  │  holon-rs            │  Evolver / GEP       │  DecentMem       │  OpenCog/Hyperon │
  │  torchhd             │  OpenSpace (HKU)     │  Mem0            │  Nengo/Spaun     │
  │  vsa-optim-rs        │  Space-Agent         │  MemGPT/Letta    │  pymdp (JAX)     │
  │  Isildur             │  EvoScientist        │  TiMem (ACL 26)  │  PyPhi 4.0       │
  │  hyperspace          │  DGM (ICLR 26)       │  Omi             │  PyHGF           │
  │  vsa-gym-wrapper     │  Ouroboros           │  Chroma/Qdrant   │  Cortex-rs       │
  │  PlausiDen           │  MUE-X               │  MEM1 (ICLR 26)  │  Engram-rs       │
  │  AVSAD (ICLR 26 WS)  │  MARIA OS SMAS       │  AgentRR         │  xagent (GPU)    │
  └─────────────────────────────────────────────────────────────┘
  产出: 20 新缺口 + 6 进化路径 + 更新经验树
```

---

## 一、深度对比查漏 — 20 个新缺口

### VSA/Representation 层 (V1-V6)

| ID | 缺口 | NeoTrix 现状 | 对标项目/论文 | 优先级 | 理论收益 |
|----|------|-------------|-------------|--------|---------|
| **V1** | **Fractional Power Encoding (FPE)** | 无连续标量编码。仅离散符号 VSA | holon-rs `encode_scalar()`, torchhd FHRR, SSP (arXiv 2604.22863 Wave-Geometric Duality) | **P0** | 连续值平滑编码→世界模型的关键前提。温度、位置、幅度等物理量直接嵌入 VSA 空间 |
| **V2** | **多 VSA 模型支持** (FHRR, B-SBC, VTB) | 仅 MAP (f64 bipolar) + HRR (FFT) | torchhd 8 模型, holon 3 模型, AVSAD (ICLR 26 WS 自动发现) | P1 | 不同 VSA 模型有不同代数性质；FHRR 使 FPE 可行，B-SBC 使稀疏高效，VTB 使学生成绑定 |
| **V3** | **在线子空间学习 (CCIPCA)** | 无流形学习。仅 bundle-average 原型 | holon-rs `OnlineSubspace`, holon Python CCIPCA | **P0** | 零硬编码异常检测 + 逐字段归因。`residual(vec)` → anomaly score |
| **V4** | **VSA Cleanup & Resonator Network** | 无噪声清理。复合 VSA 向量噪声累积无衰减 | Bremer & Orchard ACL 2024 MLE+CLE, Frady resonator network | P1 | 无 cleanup 时 VSA 操作约 3-5 次组合后保真度不可逆下降 |
| **V5** | **高级组合原语** | 仅 bind/bundle/permute/similarity | holon-rs `difference/negate/amplify/blend/attend/analogy/project/conditional_bind/invert/complexity/segment/resonance` | P1 | VSA 从"存储机制"升级为"推理代数"。pattern transfer (A:B::C:?) |
| **V6** | **VSA 梯度压缩** | 无训练集成 | vsa-optim-rs (2.9x 加速, 90% 梯度压缩) | P2 | 在本地硬件上高效微调大模型的 VSA 原生方法 |

### 记忆系统 (M1-M7)

| ID | 缺口 | NeoTrix 现状 | 对标项目/论文 | 优先级 | 理论收益 |
|----|------|-------------|-------------|--------|---------|
| **M1** | **RL 驱动的记忆巩固** | 确定性逐出规则 (FIFO+LRU) | MEM1 ICLR 2026 (RL 巩固, 3.5x 更长视界), 加权惊讶巩固 (12x 压缩 91.4% 保留) | **P0** | 替代硬编码 LRU，通过下游任务性能训练逐出策略 |
| **M2** | **时间分层记忆组织** | DecentMem 扁平双池 (E-pool + X-pool) | TiMem ACL 2026 (5 层时间树: 片段→会话→每日→每周→画像, 52% 更少 token, SoTA) | P1 | 相同召回预算下因果层级远优于扁平向量存储 |
| **M3** | **记忆 CRUD 协议** | 仅追加/读取，无更新/删除 | Mem0 (LLM 判断 ADD/UPDATE/DELETE/NOOP) | **P0** | 矛盾积累 (用户偏好变化时旧事实与新事实共存)。违反意识体十条#5 (版本化事实) |
| **M4** | **向量时钟 / CRDT 一致性** | DecentMem 单体，多 agent 无收敛保证 | ContextFS, 多代理记忆论文 arXiv 2603.10062 | P1 | 分布式记忆的收敛性保证是"去中心化记忆"的前提条件 |
| **M5** | **离线 ("睡眠") 处理周期** | MemoryConsolidationPipeline 同步/在线 | Active Dreaming Memory, Letta 睡眠时间代理, MEM1 ICLR 2026 | P2 | 异步巩固 10-20x 压缩，回放惊讶子集进行反事实模拟 |
| **M6** | **元认知置信度校准** | MetaAccuracy KPI 与记忆检索质量无交叉 | Mem0 每检索置信度, TiMem LoCoMo 基准 | P1 | 知道"这个检索的可信度是多少"，而不仅仅是检索到 |
| **M7** | **过程记忆层** | SkillMemory 存储技能，但与情景经验无联系 | AgentRR (2505.17716) 经验重放 2-3x 重用提升 | P1 | 将推理轨迹编码为可重用的过程模式 (VSA 绑定) |

### 自进化系统 (E1-E6)

| ID | 缺口 | NeoTrix 现状 | 对标项目/论文 | 优先级 | 理论收益 |
|----|------|-------------|-------------|--------|---------|
| **E1** | **进化因果追踪** | SEAL 记录日志但无因果链分析 | OpenSpace 级联进化, DGM 存档树, Ouroboros Git 谱系 | **P0** | TemporalAttentionStack 将自我修改历史作为时间序列 → 预测哪些修改将成功 |
| **E2** | **内在好奇心驱动的探索** | 系统反应式等待外部触发 | Evolver 知识差距检测, DGM 开放式存档, Ouroboros 后台意识 | P1 | 主动探索: 预测误差 → 好奇心信号 → 探索行动 |
| **E3** | **叠加验证管线** | Gödel 检查器 + SelfModifyGuard 独立运行 | MARIA OS δM + Lyapunov, DGM 基准验证, OpenSpace 级联降级 | **P0** | 语法→类型→安全→自洽→回归→基准 六层一体化验证 |
| **E4** | **优雅降级 (自我修改版)** | 原则 8 声明但未与 SEAL 集成 | DGM 崩溃 → 整个运行失败。Ouroboros 僵尸进程 | P1 | 子系统失效时不崩溃，缩小能力范围保持连贯 |
| **E5** | **DGM-H 形式化元层** | DGM-H 模式存在但未形式化 | MARIA OS 形式化 δM 算子 + 修改边界, MUE-X AST 变异策略 | P2 | "元重写元"的形式化安全保障，而非 LLM prompt 级别的约束 |
| **E6** | **主动推理控制回路** | FEP-IIT 桥有自由能计算但无策略选择 | pymdp 1.0 JAX (预期自由能 G, 策略推断, 行动采样) | P2 | 不仅"理解世界"还要"主动选择行动最小化预期自由能" |

### 认知架构 (C1-C5)

| ID | 缺口 | NeoTrix 现状 | 对标项目/论文 | 优先级 | 理论收益 |
|----|------|-------------|-------------|--------|---------|
| **C1** | **预测编码作为一等公民** | 预测模块存在但未与 GWT 注意力循环集成 | PyHGF 层级高斯过滤, xagent 7 阶段预测处理, 预测编码 RTL (arXiv 2603.18066) | P1 | 预测误差 → GWT 广播事件 (高 PE → 注意力捕获, 类似生物新奇检测) |
| **C2** | **层级高斯过滤** | 感知是平坦的，无递归精度加权信念更新 | PyHGF gHGF 可微树, Nengo 层级皮层区域 | P2 | 生物感知的层级信念传播 (Friston 2019), VSA 超立方体自然支持层级 |
| **C3** | **脉冲神经元具身接口** | 纯符号/VSA, 无生物现实主义 | xagent WGSL GPU 脉冲内核, Nengo 2.5M 脉冲神经元 | P3 | VSA 打包 → 发放率编码 → 马达指令。脉冲时间依赖性可塑性 (STDP) |
| **C4** | **可微元学习** | Meta-cognition KPI 追踪但参数调整基于规则 | pymdp 1.0 JAX 完全可微, 元梯度可计算核心超参数 | P1 | 衰减率/阈值/态转变概率可通过梯度优化, 与 SEAL 互补 (梯度+突变) |
| **C5** | **9D 编辑表面分类** | 无编辑操作分类体系 | HarnessX 9 维编辑分类, MARIA OS 修改边界 | P2 | 编辑的分类法: 局部/全局、安全/危险、可逆/不可逆、影响范围 |

### 工程/基础设施 (I1-I3)

| ID | 缺口 | NeoTrix 现状 | 对标项目/论文 | 优先级 | 理论收益 |
|----|------|-------------|-------------|--------|---------|
| **I1** | **轨迹压缩** | 所有轨迹以原始 JSON 保留 | HarnessX trajectory compression | P1 | 存储 -80%, 检索加速 5x |
| **I2** | **标准化进化基准** | `tests/` 缺少 seql_bench | DGM SWE-bench + Polyglot, OpenSpace GDPVal | P1 | 可量化的演化增益跟踪 |
| **I3** | **CRDT 驱动的完整 DecentMem 部署** | DecentMem 单体实现, 未与分布式系统集成 | DecentMem 论文 + 多代理记忆论文 | P2 | 多 agent 共享 + 去中心化收敛保证 |

---

## 二、已有多轮进化路径 (回顾)

> 已有缺口 A1-A5 (死亡代码) + B1-B10 (架构缺失) + C1-C10 (工程缺陷) + G46-G55 + G56-G75 — 详见 `EVOLUTION_ROADMAP.md` 和 `ARCHITECTURE_GAP_ANALYSIS.md`
> Phase 0-15 (A1-A5 + Gödel) ✅ 完成 / Phase 30-60 (B1-B10) ✅ 完成

---

## 三、全新六路进化路径 (Phase 105-240)

```
Phase 105-120: VSA 深度进化   [V1-V6 优先级排序]
Phase 120-150: 记忆系统重构    [M1-M7]
Phase 150-180: 自进化管道升级  [E1-E6]
Phase 180-210: 认知架构融合    [C1-C5]
Phase 210-240: 工程基础设施    [I1-I3]
```

### Phase 105-120: VSA 深度进化

| 阶段 | 任务 | 关键文件 | 依赖 | 预期周期 |
|------|------|---------|------|---------|
| **P105** | FPE 编码器: `encode_scalar()`, `encode_scalar_log()`, `encode_scalar_circular()` + SSP 空间语义指针 | `core/nt_core_hcube/fpe.rs` | V1 | 2-3 天 |
| **P108** | 多模型抽象层: VsaModel trait + FHRR/B-SBC/VTB 后端注册 | `core/nt_core_hcube/models.rs` | P105 | 2 天 |
| **P111** | CCIPCA 在线子空间学习: `OnlineSubspace` + `anomalous_component()` | `core/nt_core_hcube/subspace.rs` | V3 | 2 天 |
| **P114** | Cleanup & Resonator: `cleanup()` codebook search + resonator factorization | `core/nt_core_hcube/cleanup.rs` | P105+P108 | 3 天 |
| **P117** | 高级 VSA 原语: `attend()`, `analogy()`, `resonance()`, `project()`, `conditional_bind()` | `core/nt_core_hcube/primitives.rs` | P105 | 2 天 |
| **P120** | VSA 梯度压缩桥: 与 vsa-optim-rs 类似的 `VSAGradientCompressor` | `core/nt_core_hcube/gradient.rs` | V6 | 2 天 |

### Phase 120-150: 记忆系统重构

| 阶段 | 任务 | 关键文件 | 依赖 | 预期周期 |
|------|------|---------|------|---------|
| **P123** | RL 巩固: `SurpriseScorer` + RL policy 替代 FIFO/LRU 逐出 | `core/nt_core_experience/decent_mem.rs` | M1, Phase 30-60 DecentMem | 3 天 |
| **P126** | 时间分层记忆 (TiMem): 5 层 TMT + 跨层 VSA 摘要巩固 | `core/nt_core_experience/timem.rs` | M2 | 4 天 |
| **P129** | CRUD 协议: `MemoryOperation {Add,Update,Delete,Noop}` + VSA 冲突检测 | `core/nt_core_experience/memory_ops.rs` | M3 | 2 天 |
| **P132** | CRDT 一致性: `VectorClock` + VSA CmRDT 合并 | `core/nt_core_experience/crdt.rs` | M4 | 3 天 |
| **P135** | 睡眠周期: `SleepCycle` + 后台 `ConsolidationWorker` (GWT 空闲时隙) | `core/nt_core_consciousness/sleep.rs` | M5 | 3 天 |
| **P138** | 置信度校准: VSA 空间线性探测器 → `MetaAccuracy` KPI 反馈 | `core/nt_core_experience/calibration.rs` | M6 | 2 天 |
| **P141** | 过程记忆: `ProceduralMemory` (轨迹 → VSA 可重用过程模式) | `core/nt_core_experience/procedural.rs` | M7, CoEvolutionBridge | 3 天 |
| **P144** | 统一记忆总线: 所有记忆子系统统一 VSA 协议 | 跨文件重构 | P123-P141 | 3 天 |

### Phase 150-180: 自进化管道升级

| 阶段 | 任务 | 关键文件 | 依赖 | 预期周期 |
|------|------|---------|------|---------|
| **P150** | 进化因果追踪: TemporalAttentionStack on evolution history | `core/nt_core_self/evolution_trace.rs` | E1, Phase 0-15 Gödel | 3 天 |
| **P153** | 好奇心驱动探索: 知识缺口检测 → 探索触发器 → 主动探索行动 | `core/nt_core_experience/curiosity.rs` | E2 | 3 天 |
| **P156** | 叠加验证管线: 统一 SafetyGate + Gödel + 回归 + Benchmark | `core/nt_core_experience/stacked_validation.rs` | E3, Gödel, SelfModifyGuard | 4 天 |
| **P159** | 优雅降级集成: VsaTag 隔离 → SEAL 降级表 | `core/nt_core_experience/graceful.rs` | E4 | 2 天 |
| **P162** | DGM-H 形式化: δM 算子 + 修改边界 + 收敛保证 | `core/nt_core_experience/dgmh_formal.rs` | E5, MARIA OS 参考 | 4 天 |
| **P165** | 主动推理回路: `ActiveInferencePolicy` + `VSAGenerativeModel` + 预期自由能 G | `core/nt_core_fep_iit/active_inference.rs` | E6, pymdp 参考 | 4 天 |
| **P168** | 进化基准套件: seql_bench + GDPVal 移植 | `tests/evolution_bench.rs` | I2 | 2 天 |
| **P171** | 轨迹压缩: 进化日志 JSON → 压缩格式 (-80% 存储) | `core/nt_core_experience/trace_compress.rs` | I1 | 1 天 |

### Phase 180-210: 认知架构融合

| 阶段 | 任务 | 关键文件 | 依赖 | 预期周期 |
|------|------|---------|------|---------|
| **P180** | 预测编码集成: `VsaPredictionError → AttentionBoost`, 精度加权 PE | `core/nt_core_prediction/predictive_coding.rs` | C1, E8 态 + GWT | 3 天 |
| **P183** | 层级高斯过滤: `VSAHierarchicalFilter` (6 层抽象对应 E8 态频率带) | `core/nt_core_prediction/hierarchical.rs` | C2, HyperCube | 4 天 |
| **P186** | 脉冲神经元桥: `SNNBridge` trait + wgpu 脉冲内核 + STDP | `neotrix/nt_world_sense/snn.rs` | C3 | 5 天 |
| **P189** | 可微元学习: `MetaLearningOptimizer` (JAX/Candle 元图, 暴露核心超参数) | `core/nt_core_meta/meta_learning.rs` | C4 | 3 天 |
| **P192** | 9D 编辑分类: `EditSurface` enum (HarnessX 对齐) | `core/nt_core_self_modify/edit_surface.rs` | C5 | 1 天 |
| **P195** | 集成: 主动推理回路 ↔ 预测编码 ↔ 好奇心驱动探索 | 跨模块集成 | P165+P180+P183+P189 | 3 天 |

### Phase 210-240: 工程基础设施

| 阶段 | 任务 | 关键文件 | 依赖 | 预期周期 |
|------|------|---------|------|---------|
| **P210** | DecentMem CRDT 完整部署 + 多 agent 共享 | `agent/cognitive_memory/` | I3 | 3 天 |
| **P213** | `unwrap()` 清理: 从 1200 → 200 (80% 覆盖率) | 全代码库 | C4 (已规划) | 5 天 |
| **P216** | reqwest/rustls 版本冲突解决 | `Cargo.toml` | 已有 | 1 天 |
| **P219** | GitHub CI + Release 管道 | `.github/` | 1.1 | 2 天 |
| **P222** | OS 沙箱 (wasmtime 集成) | `nt_shield_sandbox/` | 2.1 | 3 天 |
| **P225** | 代码覆盖率 + clippy 零警告 | `scripts/`, Cargo.toml | 5.1+8.1 | 1 天 |
| **P228** | 插件系统实现 | `agent/plugins/` | 6.1 | 3 天 |
| **P231** | 第一版公开文档 + README 更新 | `docs/`, `README.md` | all | 2 天 |

---

## 四、对标项目深度分析摘要

### VSA 领域

| 项目 | 关键特性 | NeoTrix 可借鉴 |
|------|---------|---------------|
| **holon-rs** (Rust, MIT) | FPE/CCIPCA/Engram/高级原语/12x 快于 Python | V1, V3, V5 — 直接移植 CCIPCA 和 FPE 到 NeoTrix VSA 层 |
| **torchhd** (Python, MIT) | 8 VSA 模型, 序列编码, 记忆模块, cleanup | V2, V4 — 多模型抽象层设计模式 |
| **vsa-optim-rs** (Rust, MIT) | 90% 梯度压缩, 2.9x 加速, 确定性预测 | V6 — 梯度压缩桥, 对在本地微调大模型至关重要 |
| **Isildur** (Python, Apache-2.0) | NN→HV 转换, Hamming 距离推理, FPGA 目标 | V6 延伸 — 无 BP 推理, 但短期非核心 |
| **AVSAD** (ICLR 26 WS) | LLM + Lean 4 自动发现新 VSA 绑定操作 | 远期: 自动 VSA 设计 |

### 自进化领域

| 项目 | 关键特性 | NeoTrix 可借鉴 |
|------|---------|---------------|
| **Evolver** (GPL-3.0, 8.7k ★) | GEP 基因编码, 命令白名单, 4590 对照试验 | E3 叠加验证的安全模型 |
| **OpenSpace** (MIT, 19.5k ★) | 3 种进化模式 (FIX/DERIVED/CAPTURED), 级联进化, GDPVal | E1 因果追踪 + E3 验证管线 |
| **DGM** (MIT, ICLR 26) | 开放式存档, SWE-bench 20%→50%, 迭代代码重写 | E5 形式化元层的基础 |
| **Ouroboros** (MIT, 3.4k ★) | 自我重写 + 多模型审查 + 宪法, Git 原生回滚 | E4 优雅降级 + E3 安全 |
| **MARIA OS** (闭源) | 形式化 δM + Lyapunov + 修改边界 + 责任门 | E5 DGM-H 形式化的蓝本 |

### 记忆领域

| 项目 | 关键特性 | NeoTrix 可借鉴 |
|------|---------|---------------|
| **TiMem** (ACL 26) | 5 层时间树, 52% 更少 token, 时间连续性作为组织原则 | M2 分层记忆的直接可移植设计 |
| **Mem0** (Y C S24) | LLM 驱动 CRUD, 混合检索, 50k+ 开发者 | M3 CRUD 协议的工业级验证 |
| **MEM1** (ICLR 26) | RL 巩固, 3.5x 视界, 恒定上下文 | M1 逐出策略的 RL 方法 |

### 认知架构领域

| 项目 | 关键特性 | NeoTrix 可借鉴 |
|------|---------|---------------|
| **pymdp 1.0** (JAX, 674 ★) | 完全可微主动推理, 预期自由能 G, 批量策略推断 | E6 Active Inference + C4 可微元学习 |
| **Cortex-rs** (Rust, 401 ★) | GWT 运行时, 工作记忆/CLS/元认知, 插件通道 | C1 预测编码与 GWT 集成的参考实现 |
| **xagent** (Rust GPU) | WGSL 脉冲内核, 7 阶段预测处理 | C3 SNN 桥的 GPU 内核模式 |

---

## 五、优先级决策矩阵

```
                   高收益          中收益          低收益
  低工作量    V1 FPE, V3 CCIPCA  V5 高级原语      V6 梯度压缩
              M3 CRUD, M7 过程    I2 基准套件
              M1 RL 巩固         I1 轨迹压缩
              E1 因果追踪
              E3 叠加验证

  中工作量    M2 分层记忆         V2 多模型        C5 9D 分类
              C4 可微元学习       E2 好奇心探索    I3 CRDT
              M6 置信度校准       E4 优雅降级
                                 C1 预测编码

  高工作量    E6 主动推理          C2 层级过滤       C3 脉冲接口
              E5 DGM-H 形式化     M5 睡眠周期
                                 P213 unwrap 清理
```

**立即启动 (Phase 105-108)**: V1 (FPE), V3 (CCIPCA), M3 (CRUD), E1 (因果追踪), E3 (叠加验证)

---

## 六、时间线与里程碑

```
Phase 105-120 (VSA深度进化): 2026-07-01 → 2026-07-14   🎯 FPE + CCIPCA + Cleanup
Phase 120-150 (记忆重构):     2026-07-15 → 2026-08-05   🎯 RL巩固 + 分层记忆 + CRDT
Phase 150-180 (进化升级):     2026-08-06 → 2026-08-25   🎯 因果追踪 + 叠加验证 + 主动推理
Phase 180-210 (认知架构):     2026-08-26 → 2026-09-15   🎯 预测编码 + 可微元学习
Phase 210-240 (工程基建):     2026-09-16 → 2026-09-30   🎯 CRDT部署 + unwrap清理 + CI
```

---

## 七、缺失能力全景 (附 AGENTS.md 断言源)

| 断言 (AGENTS.md) | 当前实现 | 本轮缺口覆盖 |
|-----------------|---------|-------------|
| 原则#2 (统一 VSA) | HyperCube VSA ✅ | V2 多模型统一抽象层使"所有子系统共享 VSA"更真实 |
| 原则#5 (自身-世界边界) | VsaTag ✅ | M3 CRUD + M4 CRDT 版本化事实, 强化边界 |
| 原则#7 (内在驱动) | 好奇心信号 exists | E2 系统化好奇心驱动探索, 填补知识缺口 → 行动回路 |
| 原则#8 (优雅降级) | 声明 ✅ | E4 降级集成到 SEAL |
| 原则#9 (自省精度) | MetaAccuracy KPI ✅ | M6 置信度校准交叉关联 |
| 原则#10 (连续性) | SpeciousPresent ✅ | M2 分层时间记忆强化跨会话连续性 |
