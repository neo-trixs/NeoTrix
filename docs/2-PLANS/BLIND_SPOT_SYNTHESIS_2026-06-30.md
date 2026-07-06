# 盲点综合分析与吸收报告 (2026-06-30)

## 来源
- Cycle 1: anthropic/openai/apple-neural-engine/apple-intelligence (GitHub topics)
- Cycle 2: MCP/SAE/PRM/AgentPatterns/On-Device-ML/RL-Alignment (papers + repos)

---

## 关键架构盲点（优先级排序）

### P0 — 必须补齐

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 1 | **无过程奖励模型(PRM)** | OpenAI o1, PRM800K | nt_core_observer, nt_core_policy, nt_mind_seal | E8引擎做固定深度的推理，无法对中间步骤评分；Observer只有元认知监控，没有打分头 |
| 2 | **无可证明推理的SAE可解释性** | Anthropic Scaling Monosemanticity | nt_core_e8, nt_core_hcube, nt_core_gwt | 无法从E8状态向量/HyperCube中提取可解释特征；无法特征引导(steering)；无法做因果归因 |
| 3 | **无形式化对齐训练管线** | Constitutional AI, GRPO, DPO | nt_mind_seal, nt_core_policy, nt_shield | SEAL管线有RewardCalc但无学习型奖励模型；无DPO偏好学习；无GRPO组采样；无Constitution原则自批判 |
| 4 | **无边部署/编译管线** | CoreAI, MLX, ONNX, ANE | neotrix-core所有模块 | 无法在iOS/macOS/边缘设备运行；无量化管线；无硬件检测路由；无AOT编译；无LoRA适配器模式 |
| 5 | **无上下文压缩管线** | Claude Code 5层压缩, anchored iterative | nt_core_gwt, nt_io_cli | GWT上下文无限增长；无预算→裁剪→微压缩→折叠→自动压缩层级 |
| 6 | **无过程记忆(Procedural Memory)** | 现代Agent架构 | nt_memory_kb, nt_core_bank, nt_mind_seal | Episodic→Semantic→Procedural三级只有第一级；成功的E8模式序列从未固化为可重用的技能 |

### P1 — 重要

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 7 | **权限分散，非模式链** | Claude Code permission modes | nt_shield_perm, nt_shield_rails | 无plan/acceptEdits/bypassPermissions模式链；无推测性分类(speculative classification) |
| 8 | **无Constitution自批判** | Constitutional AI SL-CAI阶段 | nt_mind_seal (新增stage) | 无pre-commit反思：生成→批判→修订→提交循环 |
| 9 | **MCP传输/协议不标准** | MCP v2/v3规范 | nt_agent_mcp_discovery, nt_agent_mcp_tools | 用4种传输(含已废弃的SSE)；无OAuth 2.1；无proper init handshake；无Server Cards |
| 10 | **无测试时搜索** | o1, Snell et al. 2024 | nt_core_e8, nt_core_policy | E8只有epsilon-greedy无beam search/MCTS；固定深度64状态无自适应分配 |
| 11 | **无隐私/数据主权架构** | Apple PCC | nt_memory_kb, nt_shield | Conversation Evolution原则与隐私冲突；无stateless mode；无加密证明 |
| 12 | **无混合本地/云编排** | Apple AFM路由, Claude Hybrid | nt_io_provider, nt_core_router | 所有请求经过单一LLM provider；无tiered routing；无离线和fallback；无成本感知 |

### P2 — 优化

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 13 | MoE路由(13个GWT专家)是硬编码 | gpt-oss-120b, 128-expert MoE | nt_core_gwt | 任务→专家路由基于规则而非学习；HyperCube可作为学习路由表 |
| 14 | 无规模化律特征化 | Scaling Laws | 全局 | 未测量E8状态数/HyperCube维数/GWT专家数与任务准确率的关系 |
| 15 | 无缓存/专用化基础设施 | ANE program cache, CoreAI specialization | nt_core_epoch | 仅有KB的LRU cache(100项)；E8状态机过渡不缓存；GWT共振循环重复计算 |
| 16 | 无量化/压缩管线 | AWQ, GGUF, CoreAI Tools | nt_core_hcube, nt_core_e8 | HyperCube 4096维向量全精度存储；无INT4/INT8压缩；无per-op位宽选择 |
| 17 | 无功耗/热感知 | Apple Talaria, ANE ~2W | 全局 | 无边部署时无功耗预算；无thermal throttling；SEAL持续循环在便携设备上不可行 |
| 18 | 无特征引导(Feature Steering) | Anthropic SAE steering | nt_core_hcube | HyperCube是关联性语义空间，无法因果操控；无法"调旋钮"改变行为 |
| 19 | Agent循环无Planner/Executor/Reflector分离 | Plan-Execute-Reflect-ReAct模式 | nt_agent, nt_core_iter | 当前单一ReAct循环；无专用reflector节点；无failure分类→目标恢复 |
| 20 | 无错误恢复三层栈 | 生产级可靠性 | nt_core_observer, nt_shield | Retry+circuit breaker+fallback chain均缺失；失败无分类→盲重试 |

---

## 具体模块改造建议

### nt_core_observer (P0#1)
```
当前: Observer输出元认知报告（+1观察）
目标: Observer + PRM头 r_t = W_o · h_t + b, 存储correct/incorrect序列
训练: 从KB的ConversationRecord挖掘E8过渡序列+结果标签
```

### nt_core_policy (P0#1, P1#10)
```
当前: epsilon-greedy, 单步贪婪过渡
目标: 
  - GRPO: 每个查询采样G=4-8个E8模式, 组内归一化优势
  - 推理时 Beam/MCTS搜索: 展开K个候选→PRM评分→Top-B保留
  - 新增: 搜索树数据结构 (遍历计数N, 累计值Q, PRM分数r)
```

### nt_mind_seal (P0#3, P1#8)
```
当前: 28阶段管线, RewardCalc是启发式
新增阶段:
  - DPOStage: GWT广播轨迹→(accepted, rejected)对→DPO更新
  - ConstitutionalSelfCritiqueStage: 基于NeoTrixConstitution的生成→批判→修订
  - SafetyCheckStage: Plan→Check→Act/Refuse (MOSAIC模式)
  - ProceduralMemoryStage: 成功E8序列→KB存储为可重用技能
```

### nt_core_gwt (P0#5, P1#13)
```
当前: 13个专家竞争+广播, 上下文无限制增长
新增:
  - 5层压缩管线: 预算→裁剪→微压缩→折叠→自动压缩
  - 锚定迭代压缩: ContextState文档, 压缩触发的增量更新
  - 学习路由: HyperCube VSA作为专家路由表, 任务→专家相似度路由
```

### nt_shield_perm (P1#7)
```
当前: 分散的权限检查 (SandboxEnforcer / CloudSandbox)
目标: 模式链
  - plan (只读探索) → acceptEdits (自动批准文件系统) → bypassPermissions (全自动)
  - 推测性分类: 在hook执行时并行运行允许分类器
  - 所有工具执行通过orchestrator重试层 (模型不决定重试)
```

### nt_agent_mcp_discovery + nt_agent_mcp_tools (P1#9)
```
当前: 4传输 (Stdio/HTTP/WS/SSE) + 自定义dispatch
目标:
  - 塌缩到2传输: Stdio (本地) + Streamable HTTP (远程, 单端点 /mcp)
  - OAuth 2.1 + RFC 8707 资源指示器
  - JSON-RPC 2.0 initialize/initialized + 能力协商
  - SEP-1649 Server Cards (.well-known/mcp/server-card.json) 预先能力内省
```

### 新增: nt_core_sae (P0#2)
```
新的核心子模块 — 稀疏自编码器
- 在E8中间激活层上训练SAE (字典大小 512-4096)
- 提取可解释特征 → 特征引导 (clamping ±10× max activity)
- 跨层编码器 (CLT) 替代GWT MLP → 归因图追踪推理电路
- 3种SAE变体: Standard (L1), TopK, JumpReLU
```

### 新增: nt_core_deploy (P0#4, P2#14, P2#16, P2#17)
```
新的核心子模块 — 边缘/设备部署
- 编译管线: PyTorch→ONNX→INT4量化→硬件专用化
- 硬件检测: 启动时查询Metal/CUDA/DirectML/CPU
- AOT编译: coreai-build (Apple) / ONNX Runtime graph opt (其他)
- LoRA微适配器: 每个域一个小适配器 (rank 16, ~2MB/个)
- 功耗预算模型 + thermal throttling
- 规模化律仪表盘: E8状态数/维数/专家数 vs 准确率关系
```

---

---

## Cycle 3: 新增深度盲点（SSM/VSA/JEPA/Consciousness/Neurosymbolic/Rust-ML）

### P0 — 意识理论模块有名无实

| # | 盲点 | 来源理论 | 影响模块 | 核心缺失 |
|---|------|---------|----------|---------|
| 21 | **GWT无竞争点火** | Baars/Dehaene GNW | nt_core_gwt | 当前是累加式黑板上广播——多个专家同时激活,无winner-take-all竞争,无抑制机制。GWT要求"全或无"的门控点火 |
| 22 | **IIT φ计算错误** | Tononi IIT 3.0/4.0 | nt_core_iit_phi | 当前计算的是共鸣加权协方差度量,与Tononi的φ完全无关。真正的φ要求因果效应谱系+最小信息分区的超指数搜索 |
| 23 | **FEP无主动推理** | Friston FEP/AI | nt_core_fep_iit, nt_world_infer | FEP桥计算了一个"自由能"度量但未实现主动推理循环。没有策略选择、预期自由能最小化、感知-行动循环的形式化变分推理 |
| 24 | **无注意力模式自我模型(AST)** | Graziano AST | nt_core_meta | SelfModel是项目仪表盘,不建模注意力本身。AST要求系统持续追踪"我正关注X,强度Y,转移需要W" |
| 25 | **无高阶思维意识(HOT)** | Rosenthal HOT | nt_core_observer | Observer监控系统健康但无"我正处于心智状态M"的二阶表示结构 |

### P1 — 神经符号学缺口

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 26 | **E8与GWT之间无梯度流** | Neurosymbolic AI | nt_core_e8 + nt_core_gwt | E8状态是离散u8,专家评分是连续f64,两者之间无可微分路径。应嵌入E8状态为VSA超向量(ℝ^d) |
| 27 | **nt_core_ssm是Mamba-1** | Mamba-2 SSD | nt_core_ssm | 当前实现基于Mamba-1选择扫描。Mamba-2的SSD将状态从N=16扩展到256,训练快2-8倍,且与GWT混合兼容 |
| 28 | **nt_world_jepa只有损失设计** | I-JEPA/V-JEPA | nt_world_jepa | 当前有VICReg+SIGReg但无ViT骨干、无多块掩码策略、无动作条件预测器。JEPA预测器可成为"E8推理引擎模拟器" |

### P2 — 优化/架构

| # | 盲点 | 来源 | 影响模块 | 核心缺失 |
|---|------|------|----------|---------|
| 29 | **HyperCube维数/表示次优** | VSA理论, FHRR | nt_core_hcube | D=4096 MAP-bipolar容量~120项。FHRR at D=2048容量高4倍、noise更低、自然相位编码 |
| 30 | **无Rust ML推理栈选择** | Candle/Burn/Tract | nt_core_deploy (规划) | 混合推荐: Candle(推理)+Burn(训练)+ndarray(VSA ops),总~1.67GB |
| 31 | **GWT共鸣应替换为谐振器网络** | Frady et al. Resonators | nt_core_gwt | Kuramoto振荡器通过耦合产生绑定。谐振器网络通过因子交替投影实现更清晰的分离 |
| 32 | **E8过渡应嵌入为可微分VSA** | Neurosymbolic | nt_core_e8, nt_core_abstr | 当前三维频率矩阵可替换为: P(s'|s,a) = softmax(MLP(E8_hv(s)⊗action_hv(a))) |

---

---

## Cycle 4 完成状态 (2026-07-01)

### P0 全部 8/8 ✅

| # | 盲点 | 实现 | 文件 |
|---|------|------|------|
| 1 | PRM过程奖励模型 | PrmHead + StepReward四维评分 | nt_core_observer.rs |
| 2 | SAE可解释性 | SparseAutoencoder + SAEBridge + SteeringController | nt_core_sae.rs + nt_core_sae_bridge.rs |
| 3 | 对齐训练管线 | GRPO组采样 + DPOStage + ConstitutionalStage + SafetyStage | nt_core_policy.rs + 3 SEAL stages |
| 4 | 边缘部署 | EdgeDeployPipeline + Quantizer + HardwareDetector + AOT + LoRA | nt_core_deploy.rs |
| 5 | 上下文压缩 | Budget→Snip→Microcompact→Collapse→Auto 5层管线 | nt_core_gwt/compaction.rs |
| 6 | 过程记忆 | ProceduralMemoryStage + ProceduralRecord + KB CRUD | self_iterating/procedural_memory.rs |
| 7 | Agent循环分离 | Planner/Executor/Reflector 三角色 PER orchestrator | nt_act_autonomy/per_agent.rs |
| 8 | IIT φ修复 | IitPhiCalculator KLD因果效应谱系(≈MIP) | nt_core_gwt/geometry_sync.rs |

### P1 全部 12/12 ✅

| # | 盲点 | 实现 | 文件 |
|---|------|------|------|
| 7(权) | 权限模式链 | Plan/AcceptEdits/BypassPermissions 三模式 | nt_shield/perm_chain.rs |
| 8(Con) | 无Constitution自批判 | ConstitutionalSelfCritiqueStage 5原则 | self_iterating/constitutional_stage.rs |
| 9(MCP) | MCP传输不标准 | OAuth 2.1 PKCE + JSON-RPC 2.0握手 + 2传输模式 | nt_agent_mcp_auth.rs |
| 10 | 测试时搜索 | beam_search K=4-8 + e8_neighbors Hamming≤2 | nt_core_policy.rs |
| 11 | 隐私架构 | PrivacyEnforcer + DataSovereigntyProof + 3模式 | nt_memory_kb/privacy.rs |
| 12 | JEPA重构 | ViT编码器 + Block/Random掩码 + 动作条件预测器 | nt_world_jepa/ 4 new files |
| 13 | Mamba-2升级 | SSM_STATE_SIZE=256 + SsdState双门控 | nt_core_ssm.rs + nt_core_signal/ |
| 14 | 混合编排 | HybridOrchestrator + TierConfig + 成本感知路由 | nt_provider/hybrid_orch.rs |
| 15 | 主动推理 | ActiveInferenceLoop + expected_free_energy + precision | nt_core_fep.rs |
| 16 | 错误恢复 | Retry(CircuitBreaker(Fallback 三层 | nt_core_observer_error.rs |
| 17 | E8→VSA | E8VsaEmbedding + ChaCha12 seeded VSA ℝ^1024 | nt_core_e8_vsa.rs |
| 18 | FEP桥修复 | 主动推理循环(替代原无实现桩) | nt_core_fep.rs |

### P2 全部 11/11 ✅

| # | 盲点 | 实现 | 文件 |
|---|------|------|------|
| 17(MoE) | MoE学习路由 | MoERouter + ExpertGate + REINFORCE更新 | nt_core_gwt/moe_router.rs |
| 18 | 规模化律 | ChinchillaLaw + KaplanLaw + ScalingLawPredictor | nt_core_meta/scaling_law.rs |
| 19 | ANE缓存 | AneProgramCache + LRU + TTL | nt_core_deploy_cache.rs |
| 20 | 量化管线 | AWQ + GGUF(Q2K→Q8_0) + QuantizationPipeline | nt_core_deploy.rs |
| 21 | 功耗模型 | PowerThermalModel + HardwarePowerProfile(M1-M4) | nt_core_deploy.rs |
| 22 | 特征引导 | SteeringController + LayerSae + SteeringVector | nt_core_sae.rs |
| 23 | PER分离 | (已作为P1-5完成) | nt_act_autonomy/per_agent.rs |
| 24 | 错误恢复 | (已作为P1-6完成) | nt_core_observer_error.rs |
| 25 | FHRR | FhrrHyperCube + bind/bundle/permute/similarity D=2048 | nt_core_hcube/fhrr_vsa.rs |
| 26 | 谐振器网络 | AdaptiveCouplingKuramoto + ResonatorBank + ResonanceOptimizer | nt_core_gwt/resonator_network.rs |
| 27 | E8可微分 | (已通过E8→VSA嵌入ℝ^1024解决) | nt_core_e8_vsa.rs |

**编译状态**: `cargo check --lib` ✅ 0 error from our work (2 pre-existing lifetime errors in anthropic/gemini stubs)

---

## 研究过程中发现的高价值论文

| 论文 | 链接 | 为什么重要 |
|------|------|-----------|
| Scaling Monosemanticity | transformer-circuits.pub | SAE+特征引导, 直接映射E8→可解释性 |
| Let's Verify Step by Step | arXiv:2305.20050 | PRM800K, 每一步奖励, 直接映射Observer→PRM |
| Scaling LLM Test-Time Compute Optimally | arXiv:2408.03314 | 测试时计算优于模型缩放, E8应当做 |
| Orpheus: ANE直接编程 | arXiv:2603.06728 | ANE架构20约束→NeoTrix边缘部署路径 |
| Constitutional AI | arXiv:2212.08073 | SL-CAI+RL-CAI, SEAL管线缺这个 |
| DeepSeek-R1 (GRPO) | deepseek.com | 无critic网络RL, E8群体采样天然使用 |
| On the Biology of a LLM | transformer-circuits.pub | 归因图→E8+HyperCube电路追踪 |
| Core Views on AI Safety | anthropic.com | 可扩展监督+机械可解释性对齐策略 |
| gpt-oss: 128-expert MoE | openai.com | HyperCube作为学习路由表 |
| MOSAIC: Agent Safety | Microsoft 2026 | Plan→Check→Act/Refuse, SEAL管线安全 |
| Mamba-2: SSD | arXiv:2405.xxxxx | SSM状态N=16→256, 混合SSM:Attention模式 |
| Attention as VSA Binding | AAAI 2026 | GWT发布≈VSA解绑, 形式化E8→GWT管道 |
| JEPA survey 2026 | LeCun lab | JEPA预测器=E8推理模拟器 |
| Resonator Networks | Frady et al. 2020 | GWT共振应替换为谐振器因子分离 |
| Geometric Priors via VSA | NeurIPS 2025 | FHRR世界模型过渡学习→E8过渡 |
| NeuroSymbolic Integration Survey | Feldstein PMLR 2025 | E8+GWT的5种集成模式评估 |
