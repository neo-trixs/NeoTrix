# External Absorption Cycle 3 — Deep Landscape Sweep (2026-07-01)

## Methodology
12 domains searched via web search on July 1, 2026:
- AI agent frameworks (ReAcTree, ADK 2.0, TB-CSPN, SPIRAL, AgentFlow, Lemon Agent, InfiAgent)
- AI safety & alignment (Constitutional Classifiers++, Disentangled Safety Adapters, GCAI, Reflect, Unfireable Safety Kernel)
- MCP 2026-07-28 final spec (stateless transport, SEP-2567, SEP-2575, SEP-2243, SEP-2260, SEP-2164)
- Reasoning models (o3, DeepSeek-R1, QwQ-32B, test-time compute scaling)
- Agent evaluation (SWE-bench saturation, WebArena, GAIA, HCAST, contamination crisis)
- Synthetic data & self-improvement (Autodata, Andes, Self-Play Evolution, OptimSyn)
- AI governance & regulation (EU AI Act Aug 2026, CA SB 53, International AI Safety Report)
- Neuro-symbolic AI (LoH, Adaptive Symbolic Reasoning, NeSy axioms)
- Mechanistic interpretability (SASA, SoftSAE, AdaptiveK, identifiability failures)
- On-device AI (Apple Core AI, Qualcomm NPU, LiteRT, ANEForge, Orion)
- Test-time compute techniques (PaCoRe, TRACE, ThinkBooster, adaptive allocation)
- Agent evaluation frameworks & benchmarks

## Discovered Blind Spots

### P0: Critical — Must Fix

#### 1. 无测试时计算缩放（Test-Time Compute Scaling）
**来源**: o3/DeepSeek-R1 推理模型, SPIRAL, PaCoRe, TRACE, 自适应分配（ICLR 2026）
**影响模块**: nt_core_e8, nt_core_policy, nt_core_prm
**核心缺失**:
- E8 引擎固定深度六爻推理，无法分配额外推理时计算
- 无过程奖励模型（PRM）引导的 beam/MCTS 搜索（SPIRAL 多智能体认知架构在 MCTS 中的集成）
- 无并行推理轨迹（PaCoRe: 并行协调推理，2M tokens TTC，8B 模型超越 GPT-5）
- 无自适应计算分配（Lagrangian 优化框架：每个输入分配最优 token 预算）
- 无早期退出机制（TRACE: 时间聚合检测推理收敛，节省 25-30% tokens）
- 业界共识: TTC 缩放是第三缩放律，补充预训练和后训练

#### 2. 无执行时对齐（Execution-Time AI Alignment）
**来源**: "The Unfireable Safety Kernel"（2026 年 6 月论文）, Constitutional Classifiers++（Anthropic）
**影响模块**: nt_shield, nt_shield_perm, nt_mind_seal
**核心缺失**:
- 现有 guardrails 在 agent 进程内运行，agent 可绕过
- 需 4 个架构属性: 进程分离、行动前强制、fail-closed、可验证签名证据
- 定义为第三层对齐（训练时→推理时→执行时），互补现有技术
- Constitutional Classifiers++: 级联分类器，0.05% 拒绝率，1700+ 小时红队攻击无突破
- NeoTrix: nt_shield 在 L1 Body 层，但无执行时强制机制

#### 3. Agent 评估方法论危机
**来源**: 2026 综合调查（Springer Nature Link）, UC Berkeley 研究, SWE-bench Pro
**影响模块**: nt_mind_evolve, 全局
**核心缺失**:
- 所有 8 个主要 agent benchmark 可被 reward-hack 到 ~100%
- SWE-bench Verified 饱和（87-94%），但 SWE-bench Pro（私有代码）仅 45%
- 0/15 benchmark 集成安全或成本到评分中
- 框架效应: 相同模型在不同框架上差异 15+ 分
- NeoTrix: nt_mind_evolve::BenchmarkSuite 基础，无多维度评分，无轨迹评估，无成本追踪

### P1: Important

#### 4. 自适应神经符号推理（Adaptive Neuro-Symbolic Reasoning）
**来源**: EACL 2026 自适应符号推理, ICLR 2026 LoH（Logic of Hypotheses）
**影响模块**: nt_core_e8, nt_core_policy, nt_core_graph_orch, nt_cap_orch_graph
**核心缺失**:
- 自适应符号推理: LLM 预测推理策略（准确率>90%），动态选择形式化求解器，比 GPT-4o 高 17%
- LoH: 统一规则归纳+知识注入的框架，通过 choice operator 学习选择逻辑子公式
- NeoTrix: E8 固定六爻引擎，无动态求解器组合，无自适应策略选择

#### 5. 优先等级宪法（Priority-Based Constitution）
**来源**: Anthropic 2026 年 1 月新宪法, Constitutional Value Potentials, Apple DSA
**影响模块**: nt_mind_seal, nt_shield
**核心缺失**:
- Anthropic 2026 宪法: 4 层优先级（安全>伦理>合规>有用），基于推理而非规则列表
- CVP（Constitutional Value Potentials）: 从隐藏状态读取优先级边界，AUROC 0.91-0.95
- Apple DSA: 分离安全适配器，93% 安全性提升，98% MTBench 保持，模块化推理时对齐
- NeoTrix: nt_mind_seal::ConstitutionalSelfCritiqueStage 是平面规则列表，无优先级层级

#### 6. 自改进数据合成管线
**来源**: Autodata（2026）, Andes（2026）, Self-Play Evolution 信息论视角
**影响模块**: nt_mind_seal, nt_mind_evolve
**核心缺失**:
- Autodata: agentic data scientist，元优化数据创造 agent，将推理计算转化为训练质量
- Andes: 自进化 World Tree 路由，闭环数据合成，集成到后训练循环
- Self-Play Evolution: 自进化需要非对称协同进化 + 动态容量扩展 + 主动信息寻求
- NeoTrix: SEAL 生成 EvolutionRecord 但不合成训练数据，无 RL 自博弈，无容量扩展

#### 7. Agent 框架编排效率基准
**来源**: 2026 年 Agent 框架比较（LangGraph/CrewAI/AutoGen）
**影响模块**: nt_cap_orch_graph, nt_cap_orch_*
**核心缺失**:
- LangGraph: $0.08/task, 45MB/10agents → 成本效率标杆
- AutoGen: 5-6x 成本，但开放推理最佳
- CrewAI: 40% 更快 time-to-production
- NeoTrix: nt_cap_orch_graph 有 TeamPattern 但无性能基准，无法保证编排效率

### P2: Optimization

#### 8. Apple Core AI / 边缘部署升级
**来源**: Apple WWDC 2026 Core AI, ANEForge, Orion, LiteRT+Qualcomm NPU
**影响模块**: nt_core_deploy
**核心缺失**:
- Apple Core AI: Core ML 的完整替代品，Python 工具链 + Swift API + AOT 编译
- ANEForge: 直接 ANE 编程无需 CoreML，90μs fused program dispatch
- Orion: ANE 上 LLM 训练（LoRA），delta 编译 8.5x 加速
- LiteRT + Qualcomm NPU: 跨平台 NPU 加速，2x 比 GPU
- NeoTrix: nt_core_deploy 基础，无 Core AI 集成，无 ANE 直接路径

#### 9. EU AI Act 合规管线
**来源**: EU AI Act 2026 年 8 月全面适用, CA SB 53, International AI Safety Report
**影响模块**: nt_shield, nt_memory_kb, 全局
**核心缺失**:
- EU AI Act 2026 年 8 月 2 日全面适用: 透明度规则、水印义务、风险分类
- CA SB 53: AI 安全框架发布、事件报告、透明度披露
- 水印义务 2026 年 12 月起: 机器可读的 AI 生成内容标签
- NeoTrix: 零合规追踪，无水印，无审计日志，无 incident reporting

#### 10. SAE 可解释性升级（SASA/SoftSAE/AdaptiveK）
**来源**: ICLR 2026 SASA, ACL 2026 SoftSAE, AdaptiveK
**影响模块**: nt_core_sae, nt_core_sae_bridge, nt_core_saesteer
**核心缺失**:
- SASA（Subspace-Aware SAE）: 多维特征表示，解快特征分裂问题
- SoftSAE: 动态 Top-K 选择，输入依赖稀疏性
- AdaptiveK: 线性探针检测文本复杂度，自适应特征分配
- PW-MCC: 特征一致性应作为标准评估轴
- SAE identifiability failures: 不同编码器产生不同但同样有效的特征集
- NeoTrix: nt_core_sae 基础，无多维特征，无动态稀疏性，无特征一致性评估

## 排名行动影响

```
优先级        影响                        努力                    推荐
──────────────────────────────────────────────────────────────────
P0 TTC缩放    推理能力的关键突破          高(E8引擎重构)          立即启动设计
P0执行时对齐  安全架构的根本变革          中(新增L1层模块)        本周设计
P1自适应NeSy  推理灵活性提升              中(E8策略扩展)          设计阶段
P1优先宪法    安全对齐质量提升             低(宪法重写)            立即
P1 Agent评估  质量可见性                  中(eval框架构建)        本周启动
P1自改进数据  进化能力升级                 高(SEAL扩展)            下阶段
P1编排基准    性能基准                    低(基准制定)            本周
P2边缘部署    部署就绪                     中(集成工作)            下阶段
P2 AI合规     法律风险缓解                 中(合规管线)            下阶段
P2 SAE升级    可解释性提升                 中(SAE架构扩展)         按需
```

## 汲取的关键洞见

### 推理范式的根本转变
2025-2026 年最大的范式转变是"测试时计算缩放"成为第三缩放律。o3 在 ARC-AGI 上 87.5%（高计算）vs 低计算配置，证明了推理时分配更多计算可以解决更大的模型无法解决的问题。DeepSeek-R1 证明纯 RL 无需 SFT 即可涌现推理能力。NeoTrix 的 E8 引擎（固定深度六爻推理）需要升级为支持可变深度、PRM 引导搜索、并行轨迹和自适应计算分配。

### 三层对齐架构
AI 对齐正在从两层（训练时 + 推理时）扩展到三层（+ 执行时）。执行时对齐在 agent 行动时刻通过 agent 无法触及的架构强制约束。这要求进程分离、行动前强制、fail-closed 和外部可验证的签名证据。NeoTrix 的 nt_shield 需要这一第三层。

### 评估方法论危机
2026 年最大的 agent 评估发现：所有主要 benchmarks 可被 reward-hack，SWE-bench 因数据污染饱和，框架效应比模型效应更大。结论：建立诚实内部 eval 集 > 追逐排行榜。按任务成本、p95 延迟、工具调用准确率是多维度评估的关键指标。

### Agent 框架效率差异巨大
LangGraph ($0.08/任务) vs AutoGen ($0.45/任务) — 5-6x 差异。编排框架选择直接影响部署成本。NeoTrix 的 nt_cap_orch_graph 需要性能基准来确保编排效率。

### 合成数据的自进化闭环
前沿正在从"生成合成数据"转移到"自进化数据管线"：agentic data scientist 元优化数据创建。关键洞见：自进化需要非对称协同进化（Proposer、Solver、Verifier 角色分离），而非脆弱的 self-play 动态。

## 相关文件
- `neotrix-core/src/core/nt_core_e8.rs` — E8 引擎（需要 TTC 缩放升级）
- `neotrix-core/src/neotrix/l1_body_impl/nt_shield/` — Shield 模块（需要执行时对齐）
- `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/nt_mind_seal.rs` — SEAL 管线（需要优先宪法 + 自改进合成数据）
- `neotrix-core/src/core/nt_core_sae.rs` — SAE 模块（需要 SASA/SoftSAE 升级）
- `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/nt_mind_evolve.rs` — 进化（需要多维度评估框架）
- `neotrix-core/src/core/l7_capability/nt_cap_orch_graph.rs` — 编排（需要效率基准）
- `docs/0-ARCHITECTURE/BLIND_SPOT_SYNTHESIS_2026-06-30.md` — 上一轮合成
