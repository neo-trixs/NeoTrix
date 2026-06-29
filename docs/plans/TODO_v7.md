# NeoTrix 进化补缺 — Wave 1 实施任务

> 基于 EVOLUTION_ROADMAP_v7.md, Phase 150-170 (8 P0 并行)
> 预计: ~8000 行, 6 路并行 agent, 无相互依赖

---

## Phase 150 — G101 IIT 4.0 φ 集成

**文件**: `neotrix-core/src/core/nt_core_consciousness/iit_phi.rs`
**参考**: Aura IIT implementation, Oizumi 2014 IIT 4.0
**预计**: ~1500 行, 15-20 测试

### 实现内容
- [x] StateGrid: 16-node 二元状态向量
- [x] TPM: 条件状态转移矩阵
- [x] φ 计算: 系统划分 + 有效信息 EI
- [x] MIP: 最小信息划分搜索 (排他假设)
- [x] Φ 3.0: 完整集成信息度量
- [x] 意识检测: Φ > 0 阈值判定
- [x] ConsciousnessLoop 集成: 每 tick IIT 快照

### 测试
- [x] 全连接 vs 断连系统 φ 差
- [x] 排他假设验证
- [x] Φ 单调性 (加连接增 Φ)
- [x] 已知意识系统验证

---

## Phase 152 — G102 CAA 残差流情感引导

**文件**: `neotrix-core/src/core/nt_core_consciousness/caa_steering.rs` (增强)
**参考**: Aura MLX CAA, Arditi 2024 representation engineering
**预计**: ~1200 行, 12-15 测试

### 实现内容
- [x] GeometryValidator: 几何属性检验 (PCA/线性/正交)
- [x] PermutationTest: 置换显著检验
- [x] BlackBoxHygiene: 黑盒卫生检验 (独立分布)

### 测试
- [x] 几何属性验证
- [x] 置换检验 p-value
- [x] 黑盒卫生检验

---

## Phase 155 — G92 多 VSA 模型统一 API

**文件**: `neotrix-core/src/core/nt_core_hcube/vsa_multi_model.rs`
**参考**: torchhd 8 VSA models, Holon-rs
**预计**: ~1200 行, 18-22 测试

### 实现内容
- [x] VsaModel trait: bind/bundle/similarity/invert/cleanup
- [x] MAP: Multiply-Add-Permute (已有)
- [x] BSC: Binary Spatter Code (已有)
- [x] HRR: Holographic Reduced Representation
- [x] FHRR: Fourier HRR (已有)
- [ ] B-SBC: Binary Sparse Block Code
- [ ] VTB: Vector-derived Transformation Binding
- [ ] CGR: Circular Generalize Regression
- [ ] MCR: Matrix Coding Representation
- [x] ModelFactory: 运行时模型选择
- [ ] GPU batch 准备: PyTorch 绑定计划

### 测试
- [x] 所有模型 bind/unbind 正确性 (16 tests)
- [x] 跨模型相似度一致性
- [x] 运行时模型切换 (ModelFactory)
- [ ] 模型间向量转换
- [ ] 性能基准

---

## Phase 158 — G97 GPU/Metal 加速 VSA

**文件**: `neotrix-core/src/core/nt_core_hcube/vsa_gpu.rs`
**参考**: torchhd GPU batch, Metal Performance Shaders
**预计**: ~1000 行, 8-10 测试

### 实现内容
- [x] MetalAccel: Apple Silicon GPU 加速层 (auto-detect via cfg!/process)
- [x] GpuVsaBackend trait: Metal/CPU fallback
- [x] BatchBind: 批量绑定
- [x] BatchSimilarity: 批量余弦相似度
- [x] AutoDetect: 运行时检测 Metal 可用性
- [x] Fallback: GPU 不可用 -> CPU

### 测试
- [x] GPU vs CPU 结果一致性
- [x] Fallback 正确性
- [ ] 加速比测量 (最小 5x)
- [ ] 大批量稳定性

---

## Phase 160 — G104 活动意志收据

**文件**: `neotrix-core/src/core/nt_core_consciousness/unified_will.rs`
**参考**: Aura Unified Will + Receipt, 自主活动追溯
**预计**: ~800 行, 10-12 测试

### 实现内容
- [x] WillReceipt: 空结构体 (预存 stub)
- [x] ReceiptChain: 空结构体 (预存 stub)
- [x] WillState: 空结构体 (预存 stub)
- [ ] WillRegister: 目标 - 活动绑定
- [ ] WillAuditor: 收据完整性审计
- [ ] FirstPersonRef 集成
- [ ] ConsciousnessLoop 挂载

### 测试
- [ ] 收据链完整性
- [ ] 活动-目标绑定
- [ ] 状态转换正确性
- [ ] 审计检测篡改
- [ ] 竞态条件

---

## Phase 162 — G108 6 神经递质调制

**文件**: `neotrix-core/src/core/nt_core_consciousness/neuromodulation.rs`
**参考**: Anima 6-transmitter system
**预计**: ~800 行, 10-12 测试

### 实现内容
- [x] Neurotransmitter: DA/NE/5-HT/ACh/Orexin/GABA (6-channel ODE)
- [x] ModulatorState: 每个递质浓度 [0,1]
- [x] SynthesisRate: 合成速率 (tonic)
- [x] CognitiveEffect: 认知效果映射 (学习率/唤醒/情绪/警觉/抑制/睡眠压力)
- [x] ModulatorInteraction: alertness + net_arousal + sleep_pressure
- [ ] CircadianRhythm: 昼夜节律调制
- [ ] GWT 集成: 调制注意力和学习参数

### 测试
- [x] 递质浓度动力学 (tonic/phasic)
- [x] 认知效果映射正确性
- [x] 6-channel stats 报告
- [ ] 昼夜节律周期
- [ ] GWT 参数调制

---

## Phase 165 — G123 现代 Hopfield 网络

**文件**: `neotrix-core/src/core/nt_core_hcube/hopfield_network.rs`
**参考**: Zikkaron, Ramsauer 2021, Modern Hopfield Networks
**预计**: ~800 行, 12-15 测试

### 实现内容
- [x] HopfieldEnergy: 连续能量函数 (soft exponential)
- [x] MemoryPattern: 存储模式 Vec<Vec<f64>>
- [x] RetrieveByEnergy: 能量排名检索
- [x] PatternComplete: 噪声模式完成
- [x] UpdateRule: 迭代状态更新
- [x] StorageCapacity: 动态容量估计

### 测试
- [x] 能量单调递减
- [x] 模式检索正确性
- [x] 噪声容限
- [x] 容量估计

---

## Phase 168 — G124 预测编码写入门控

**文件**: `neotrix-core/src/core/nt_core_consciousness/predictive_gate.rs`
**参考**: Zikkaron surprisal filter, predictive coding theory
**预计**: ~500 行, 8-10 测试

### 实现内容
- [x] SurprisalScore: 惊奇度计算 (prediction_error / expected_uncertainty)
- [x] GateDecision: 惊奇度阈值 + 动态自适应
- [x] WriteGate: 高惊奇通过 / 低惊奇过滤 (should_write)
- [x] ContextAwareThreshold: 基于滑动窗口的自适应阈值
- [x] GateProbability: 门控概率逻辑函数
- [ ] DecentMem 集成: 写前惊奇评估
- [ ] MetaLearning 集成: 阈值自调优

### 测试
- [x] 惊奇度计算正确性
- [x] 门控决策精确率/召回率
- [x] 自适应阈值行为
- [x] 阈值钳位
- [x] 重置

---

## Wave 1 集成测试

**文件**: `tests/wave1_integration_tests.rs`
**预计**: ~800 行, 20-25 测试

### 测试范围
- [ ] IIT φ + ConsciousnessLoop 集成
- [ ] CAA + 认知评价集成
- [ ] 多 VSA + Hopfield 协同
- [ ] GPU + CPU 一致性大规模
- [ ] 意志收据 + 自我模型
- [ ] 神经递质 + GWT 注意力
- [ ] 预测门控 + DecentMem 写
- [ ] 全系统压力测试

---

## 合计

| 模块 | 行数 | 测试 |
|------|------|------|
| G101 IIT 4.0 | 987 | 15 ✅ |
| G102 CAA validation | 423 | 8 ✅ |
| G92 多VSA | 392 | 16 ✅ |
| G97 GPU | 318 | 8 ✅ |
| G104 意志(stub) | 7 | 0 (pre-existing stub) |
| G108 6-神经递质 | ~300(ext) | 10 (新增) ✅ |
| G123 Hopfield | 175 | 10 ✅ |
| G124 预测门控 | 164 | 9 ✅ |
| **合计** | **~2766** | **~76** ✅ |
