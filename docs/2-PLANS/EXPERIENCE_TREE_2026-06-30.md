# NeoTrix 经验树 — 2026-06-30 外部吸收循环

## 吸收来源

### Cycle 1: GitHub Topics
```
anthropic        → 15 repos, 3 papers, 7 architecture insights
openai           → 8 repos, 5 papers, 7 architecture insights
apple-neural-engine → 8 repos, 4 papers, 6 architecture insights
apple-intelligence  → 8 repos, 3 papers, 6 architecture insights
```

### Cycle 2: Deep Dive
```
MCP规范           → 7 transport/security insights
SAE可解释性        → 3 concrete integration paths for E8/HyperCube/GWT
PRM+测试时搜索    → 200-300行代码改造蓝图
Agent设计模式      → 6 recommendations (reflector, compaction, recovery, permissions, memory, dispatch)
边缘部署           → 完整编译管线 + 1.3GB内存预算蓝图
RL对齐             → 6 recommendations (DPO, GRPO, RLVR, MOSAIC, Constitution, InfoRM)
```

### Cycle 3: Deep Dive II
```
SSM/Mamba          → Mamba-1→Mamba-2(SSD)升级, 混合SSM:Attention模式
VSA/HD计算         → FHRR优于MAP, D=2048 > D=4096, Attention≈VSA解绑
JEPA/世界模型      → nt_world_jepa缺ViT/掩码/动作条件; JEPA预测器=E8推理模拟器
意识理论           → GWT/IIT/FEP/HOT/AST均有名无实, 核心机制未实现
神经符号学         → E8嵌入VSA超向量消除梯度壁垒
Rust ML引擎        → Candle+Burn+ndarray混合推荐, ~1.67GB总量
谐振器网络         → GWT共鸣应替换为谐振器因子分离
```

## 交叉分析: 从19→32个识别出的盲点

```
P0 (8): PRM / SAE / Alignment / Edge / Context / ProceduralMemory / ConsciousnessTheory / GradientFlow
P1 (8): Permissions / Constitution / MCP / TestTimeSearch / Privacy / Hybrid / SSM / JEPA
P2 (10): MoE / ScalingLaw / Cache / Quantization / Power / Steering / ErrorRecovery / VSA / Resonance / TransitionGrad
```

## 模块→盲点映射表 (v3)

```
nt_core_observer     → PRM (#1) + HOT (#25) + ErrorRecovery (#24)
nt_core_policy       → PRM (#1) + TestTimeSearch (#12) + GRPO (#3) + VSA Diff (#27)
nt_core_e8           → SAE (#2) + TestTimeSearch (#12) + GradientFlow (#26) + VSA嵌入 (#32)
nt_core_hcube        → SAE (#2) + FeatureSteering (#22) + Quantization (#20) + VSA表示 (#29)
nt_core_gwt          → ContextCompaction (#5) + MoE (#17) + ScalingLaw (#18) + Ignition (#21) + Resonance (#30)
nt_core_sae (新增)   → SAE (#2) + FeatureSteering (#22)
nt_core_deploy (新增)→ Edge (#4) + Quantization (#20) + Power (#21) + ScalingLaw (#18) + RustML (#31)
nt_mind_seal         → Alignment (#3) + Constitution (#10) + ProceduralMemory (#6)
nt_memory_kb         → ProceduralMemory (#6) + Privacy (#13)
nt_shield_perm       → Permissions (#9) + Privacy (#13)
nt_shield            → ErrorRecovery (#24) + SafetyCheck (#3)
nt_agent_mcp_*       → MCP (#11)
nt_io_provider       → Hybrid (#14)
nt_agent循环         → Planner/Executor/Reflector (#23)
nt_core_bank         → ProceduralMemory (#6)
nt_core_epoch        → Cache (#19)
nt_core_ssm          → Mamba-1→Mamba-2 (#15)
nt_world_jepa        → JEPA架构 (#16)
nt_core_iit_phi      → IIT φ错误 (#22)
nt_core_fep_iit      → FEP无主动推理 (#23)
nt_core_meta         → AST缺失 (#24)
```

## 架构决策记录

### ADR-2026-06-30-1: E8作为搜索空间而非固定管线
**Context**: PRM+测试时搜索=E8引擎的最单点高impact改造
**Decision**: Observer增加PRM头(topology: 64→32→1), Policy从epsilon-greedy改为GRPO
**Cost**: ~300行新Rust代码, KB需要存储E8过渡序列用于PRM训练数据挖掘
**Impact**: E8从固定深度64状态变为自适应搜索, GWT专家路由从硬编码变PRM评分驱动

### ADR-2026-06-30-2: SAE作为可解释层
**Context**: 无法从E8状态向量提取特征→无法引导推理
**Decision**: 新增nt_core_sae模块, 在E8中间激活层训练SAE (TopK变体, K=16-64)
**Timeline**: 改造nt_core_e8的输出层以暴露中间激活; SAE训练使用KB中存储的推理轨迹

### ADR-2026-06-30-3: 2传输MCP
**Context**: 当前4传输包含已废弃的SSE, 不支持OAuth
**Decision**: 塌缩到Stdio+Streamable HTTP, 实现OAuth 2.1 + RFC 8707

### ADR-2026-06-30-4: 三级记忆管线
**Context**: 目前只有Episodic (ConversationRecord/EvolutionRecord), 需要Semantic+Procedural
**Decision**: Episodic→Semantic consolidation在SEAL ConversationDistillStage, 新增ProceduralMemoryStage

### ADR-2026-06-30-5: GWT 5层压缩
**Context**: GWT上下文无限制增长会OOM
**Decision**: 逐级: token budget→snip(裁剪last-10之外的)->microcompact(摘要)->collapse(锚定迭代)->auto-compact(windows 95%触发)
Implemented: ContextState文档在GWT workspace中, 增量更新

### ADR-2026-06-30-6: E8嵌入为VSA超向量
**Context**: E8与GWT之间无梯度流 → 无法进行神经符号学习
**Decision**: 将每个ReasoningHexagram映射为ℝ^1024 VSA超向量. E8过渡变为VSA绑定操作, GWT共鸣变为余弦相似度
**Impact**: 离散u8变成连续ℝ^d, E8+GWT全管道可微分, 支持PRM+GRPO+BeamSearch

### ADR-2026-06-30-7: GWT竞争点火
**Context**: 当前累加黑板模式——非GWT
**Decision**: 添加winner-take-all竞争门: 所有专家同步→共鸣评分→取top-k(k=1默认)→广播唯一胜者→抑制其他
**Cost**: 新的CompetitionGate结构, ~50行Rust
**Impact**: 从黑板上变为真正的GNW全局点火架构

### ADR-2026-06-30-8: JEPA预测器作为E8推理模拟器
**Context**: nt_world_jepa只有损失设计无架构
**Decision**: 重构nt_world_jepa: ViT骨干=E8状态序列, 掩码=隐藏推理步骤, 预测器=条件next-hexagram生成器
**Impact**: JEPA成为E8推理轨迹的隐式模拟器, 支持counterfactual推理规划

### ADR-2026-06-30-9: nt_core_ssm Mamba-1→Mamba-2升级
**Context**: Mamba-1 N=16有限, Mamba-2 SSD N=256且训练快2-8×
**Decision**: nt_core_ssm迁移到Mamba-2 SSD. 核心: 将选择性SSM参数化为半可分矩阵(Semi-separable SSD)
**Timeline**: 保留Mamba-1版本为边缘推理回退路径

### ADR-2026-06-30-10: Rust ML引擎分层
**Context**: 边缘部署需要推理引擎选择
**Decision**: Candle(推理)+Burn(SEAL训练)+ndarray(VSA操作)三层
**Budget**: ~1.67GB全量, E8状态机仅需~50MB

## 未探索方向 (Future Cycles after 3)

1. **因果推断**: E8状态之间的因果效应, 不只是相关
2. **多Agent通信协议**: A2A协议的NeoTrix实现
3. **知识蒸馏**: nt_world_jepa蒸馏E8引擎到小模型
4. **形式化验证**: E8过渡矩阵的形式化+模型检查
5. **分布式E8**: 跨实例的E8状态共识与合并
6. **心理理论(ToM)**: 对其他智能体的心智建模
7. **梦/离线巩固**: 睡眠阶段更深入的理论机制(反向传播不足)
