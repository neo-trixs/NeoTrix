# NeoTrix 经验树 — 2026-07-01 Cycle 4: 9支柱重构 + 外部深度吸收

## 吸收来源

### Cycle 4-1: 竞争项目全景扫描
```
AutoGPT (185K★)     → DAG视觉构建 + PostgreSQL持久化, 平台化方向
LangGraph (35.8K★)  → 类型安全状态图 + 持久化检查点, 最成熟agent runtime
CrewAI (54.6K★)     → 角色分层团队 (Manager→Employee), Event-driven Flows
OpenAI Agents SDK   → @function_tool装饰器, handoff路由, 服务器端工具
smolagents (26K★)   → 代码优先行动 (写Python), ~30%少步骤
Letta/MemGPT (23.5K★) → OS式记忆分页, sleep-time计算
AutoGen (59.3K→MAF) → 对话驱动多智能体, MCP+A2A互操作
OpenCog Hyperon      → MeTTa语言+元图重写, 最深度AGI但pre-alpha
EIDOS               → 15阶段连续认知循环, φ*计算器, 梦巩固
Lincoln             → 连续信念修正 (AGM理论), 5s tick循环
ExCortex            → 生物神经系统隐喻, 神经可塑性自改进
```

### Cycle 4-2: 论文深度吸收 (12主题, 45+论文)

12个关键主题的含代码/可操作insight提取：

**1. PRM过程奖励** (ThinkPRM, R-PRM, GroundedPRM, AgentPRM, EDU-PRM)
- Generative PRM > discriminative: 生成验证CoT比标量评分更数据高效
- AgentPRM: 3B模型+PRM > GPT-4o on ALFWorld → 路径: Observer注入PRM头
- EDU-PRM: 熵自动分步 → 消除人工步骤标注
- **对NeoTrix**: E8的64态天然是"步骤边界" → 每个E8过渡得到一个PRM评分

**2. GRPO组采样** (DeepSeek-R1, GRPO理论分析)
- Advantage = (reward - group_mean) / group_std → 无critic网络
- 验证奖励 (math/code pass-fail) 优于神经奖励模型
- 成功放大定理: PoS迭代收敛到固定点 > 参考策略
- LLD问题: 均匀token惩罚损害学习 → 需NTHR修复
- **对NeoTrix**: E8的epsilon-greedy → GRPO G=4-8, E8输出→无criticRL

**3. SAE可解释性** (Scaling Monosemanticity, Gated SAE, Matryoshka SAE, SAEBench)
- K-sparse SAE消除L1调参, 死latent可消除
- Gated SAE解决收缩问题 (L1惩罚的系统低估)
- Matryoshka SAE: 层次化特征结构 → 多种粒度可解释性
- **对NeoTrix**: SAE已在nt_core_sae(409行) → 需要集成到E8推理流

**4. 测试时计算** (Snell et al., VG-Search, DORA, LATTS, The Art of Scaling TTS)
- 搜索(beam/MCTS) + 自适应修订优于相同FLOPs的14×大模型
- 难度自适应分配: beam对难题, best-of-N对简单题
- 步骤级难度方差大 → LATTS 50×更少token匹配beam性能
- **对NeoTrix**: E8引擎应从固定64深度→自适应beam搜索

**5. 过程记忆** (ReMe, MACLA, Memp, PRAXIS, CDMem, MUSE)
- 15:1压缩比: 2851轨迹→187过程
- 状态依赖检索: 外部环境+内部目标联合匹配
- 跨模型迁移: 自然语言过程记忆是LLM无关的
- **对NeoTrix**: 过程记忆是独特差异化优势(无人做)

**6. 上下文压缩** (SAC, Explicit Info Transmission, Pre-train Compressor, Agent压缩)
- 硬压缩(选择) + 软压缩(学习向量)混合
- 锚点选择(a anchor tokens)双向注意力聚合信息
- 智能体上下文异构 (观察+推理+工具痕迹) → 需保留时间依赖
- **对NeoTrix**: 5层压缩管线设计验证

**7. MoE路由** (ReMoE, MaxScore, Dynamic MoE, ERL, 聚类路由, RMoE)
- ReLU路由使MoE全可微分 → E8→GWT嵌入
- 难度感知: 复杂任务激活更多专家
- 跨层路由关联 (GRU) → GWT专家的时序依赖
- RL优化路由vs随机扰动 ≥8.9× MRR提升
- **对NeoTrix**: HyperCube作为学习路由表

**8. V-JEPA世界模型** (V-JEPA 1/2/2.1, Meta FAIR)
- 特征预测(潜空间)比像素预测样本效率高1.5-6×
- V-JEPA 2-AC: 隐式动→条件世界模型 → 零样本机器人规划
- 稠密预测损失→像素级对应关系
- **对NeoTrix**: JEPA预测器=E8推理模拟器, 条件预测E8过渡

**9. Mamba-2 SSD** (State Space Duality, Mamba-3)
- SSM和注意力是对偶的(半可分矩阵)
- 状态N=16→256, 训练快2-8×
- 混合SSM:Attention模式 → chunk内注意力, chunk间SSM
- 25行代码实现minimal SSD
- **对NeoTrix**: nt_core_ssm Mamba-1→Mamba-2, SSD + 混合模式

**10. 谐振器网络** (Frady et al., Self-Attention Resonator, Neuromorphic HRN, ACF, Kroneker)
- 容量与N二次方: N=10000 → 100+项因子分解
- 噪声是功能不是bug: 可控噪声注入打破极限环
- 分层谐振器处理非交换变换
- 对数量级clean-up: Kroneker旋转O(N log N)
- **对NeoTrix**: GWT共鸣→谐振器因子分离

**11. GWT意识实现** (GWA, GWT Routing, Synthetic Neurophenomenology, Embodied GWT)
- LSTM路由门控信息流 → 系统2推理需要"意识"门控
- GWT广播增强噪声 → 需要HOT元认知质量控制
- 熵驱动温度调节打破推理死锁
- **对NeoTrix**: GWT竞争点火 + HOT注意力自我模型

**12. 意识理论集成** (MTC框架, Huang nanoGPT, AI Consciousness Case)
- GWT+IIT+HOT是互补功能层不是竞争理论
- GWT: 访问广播容量; HOT: 质量控制; IIT: 因果整合
- Φ计算可微 → 可作为训练损失
- **对NeoTrix**: 7个意识理论全部有名无实 → 实现GNW点火+因果Phi+AST+HOT+FEP

---

## 交叉盲点分析: 32→45个识别盲点 (Cycle 4增长)

### 新增P0盲点 (Cycle 4新增6个)

| # | 盲点 | 来源 | 核心发现 |
|---|------|------|---------|
| 33 | **代码重复: nt_act_mcp.rs精确重复nt_agent_mcp_discovery.rs** | 代码审查 | 相同sha哈希, 376行完全重复 |
| 34 | **声明但文件缺失: nt_agent_protocol/nt_agent_subagent文件不存在** | 代码审查 | 编译会失败的潜伏bug |
| 35 | **agent/目录双tool系统: tools/ + tool/** | 代码审查 | 两个平行tool实现, 职责混乱 |
| 36 | **无Rust ML推理栈: 仅CPU软计算** | 论文吸收 | Candle(rust)/Burn/tract三者均可选, 但无集成 |
| 37 | **上下文压缩无token预算感知** | 论文吸收 | 5层压缩设计无触发阈值, 无预算衰减 |
| 38 | **无跨模型过程记忆迁移** | 论文吸收 | ReMe/MACLA证明过程记忆可跨模型转移, NeoTrix未做 |

### 累计盲点统计
```
Cycle 1 (git topics):   19个
Cycle 2 (papers):       13个 → 19→32
Cycle 3 (consciousness): 0个新增 → 32
Cycle 4 (9支柱+竞争分析):6个新增代码缺陷 + 6个新盲点 → 45个
```

---

## 架构决策更新 (ADR-2026-07-01)

### ADR-2026-07-01-11: 9支柱架构
**Context**: 7域架构中agent逻辑和LLM provider逻辑错位, 导致nt_mind膨胀113文件
**Decision**: 7域→9支柱 (新增NT-AGENT, NT-PROVIDER)
**Migration**:
- nt_io_provider/* (12文件) → nt_provider/*
- agent/ (13K LOC) → neotrix/nt_agent/*
- nt_mind (113文件) 瘦身: 移出知识/工具/记忆到对应域
**Impact**: 预期: 模块内聚性+40%, 交叉引用-60%

### ADR-2026-07-01-12: 过程记忆作为核心差异化
**Context**: 竞争对手全都没有过程记忆 (只有Episodic+Semantic)
**Decision**: ProceduralMemoryStage进入P0优先级。存储: (skill_id, e8_state_sequence, trigger_condition_embedding, success_rate, n_attempts)
**Architecture**: HyperCube VSA作为过程记忆的联想索引 → 状态→技能检索O(1)
**Impact**: NeoTrix在agent记忆架构上领先所有开源项目

### ADR-2026-07-01-13: 竞争架构定位
**PoV**: NeoTrix是唯一结合E8+GWT+IIT+VSA+SAE+PRM+过程的认知架构
- AutoGPT/LangChain/LangGraph: 有生产级agent框架但零认知理论
- OpenCog: 最深AGI但pre-alpha, 社区~1K★
- MTC/EIDOS/Lincoln/ExCortex: 最接近但~0★, 单开发者, 不可生产
- **NeoTrix独特定位: 认知深度 × 生产可用的交叉点**

### ADR-2026-07-01-14: JEPA→E8推理模拟器
**Context**: 论文揭示JEPA预测器本质是隐式世界模型
**Decision**: nt_world_jepa的预测器输出 = E8的next-hexagram概率分布。训练: (当前E8状态, 动作) → 预测器 → (下一E8状态分布)。对齐: 预测器softmax分布与E8过渡矩阵之间的KL散度最小化
**Impact**: JEPA成为E8的可微分近似, 支持对抗性推理规划

### ADR-2026-07-01-15: nightly Rust ML引擎选型
**Context**: 边缘部署需要推理引擎
**Decision**: 三层栈:
1. **Candle** (推理): ONNX导入, CPU+Metal推理, ~50MB
2. **Burn** (训练): SEAL循环中的GRPO/DPO训练, 动态图
3. **ndarray** (VSA ops): 超高效矩阵操作, 零依赖
**Total budget**: ~1.67GB全量, 纯E8+SAE推理~120MB

---

## Cycle 5: Deep Absorption Engineering (2026-07-01)

### 吸收来源
Deep research document R1-R9 (246 lines, 9 research domains, 14 blind spots B120-B133) + Cycle 5 experience tree (7 additional blind spots).

### 实施的项目

**B122 — Langevin噪声注入 (E8策略)**
- `neotrix-core/src/core/nt_core_policy.rs` (B122)
- 原理: 用Langevin动力学 `sigma = sqrt(2*alpha/beta)` 替代固定高斯噪声。`alpha`控制随机探索强度, `beta`控制确定性漂移
- 发现: Langevin公式对E8策略的epsilon-greedy框架是自然拟合。epsilon控制探索概率, Langevin控制探索幅度。二者可共存: epsilon选是否探索, Langevin定探索方向
- 测试: 5个新测试覆盖 `alpha=beta=0` (零噪声), `alpha=2,beta=0.5` (强探索), 默认值, builder链

**B121 — MIMO rank-4混合 (E8策略)**
- `neotrix-core/src/core/nt_core_policy.rs` (B121)
- 原理: SISO → MIMO转换通过6维→4维→6维瓶颈投影。矩阵 `P: R^6→R^4`, `Q: R^4→R^6`, 种子 `0xB1_M1_M0` 确保确定性混合
- 发现: rank-4投影在6维E8概率向量上实现跨维度信息混合。通过确定性seed保证可复现性。提供 `with_mimo()` builder方法, 默认关闭
- 测试: 7个新测试覆盖混合输出形状, 归一化, 等概率输入, seed可复现性, 输入稀疏时的注意力重分配

**B130 — 扩散激活检索 (FHRR VSA)**
- `neotrix-core/src/core/nt_core_hcube/fhrr_vsa.rs` (B130)
- 原理: 在VSA码本上做N步消息传播: `score_i = sum(edge_ij * score_j)`, 衰减系数 `decay^step`, 边界阈值过滤弱连接
- 发现: 扩散检索与精确最近邻检索互补。扩散找到"语义邻居的邻居" — 通过码本拓扑传播激活, 而非直接向量相似度。默认: steps=3, decay=0.7, edge_threshold=0.15, top_k=5
- 测试: 8个新测试覆盖1步/3步/5步扩散, 高衰减, 高阈值过滤, 空码本

**#2a — BenchmarkGate SEAL阶段**
- `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/self_iterating/benchmark_gate.rs` (NEW)
- 原理: 在ValidationGate之后插入一个SEAL阶段, 对champion代码运行5个基准测试 (reasoning, code_gen, tool_use, creative, analysis), 评分通过线性权重聚合。Accept(≥70), Retry(30-70), Rollback(<30)
- 发现: BenchmarkGate在训练早期倾向于Retry。6个测试中的 `test_benchmark_gate_is_accepted` 期望champion_0被接受但默认性能未达70分, 需要预热或更好的初始代码
- 测试: 6个新测试覆盖Accept/Retry/Rollback决策, 边缘权重, 空证据, Retry建议

**B129 — 反思巩固代理 (HyperCube)**
- `neotrix-core/src/core/nt_core_hcube/reflection_consolidation.rs` (NEW)
- 原理: 4阶段流水线 — verify(验证最近存储条目), cross-link(创建组合符号束), prune(移除低访问条目), compress(合并近恒等向量)
- 发现: cross-link通过扫描所有码本对创建组合束, prune标记访问计数<3的条目, compress合并cosine>0.95的符号。对称矩阵扫描的O(n²)复杂度在n<1000时可接受
- 测试: 8个新测试覆盖verify/cross-link/prune/compress各阶段

**#6 — 运行时工具锻造 (MCP Registry)**
- `neotrix-core/src/agent/tool/mcp/mod.rs` (extended)
- 原理: `ToolSource::InlineCode { code, runtime }` 替代仅路径的tool source。`verify_tool_code()` 执行语法检查+安全扫描 (ban eval/exec/unsafe/rm -rf等), `publish_code()` 写入 `~/.neotrix/tools/{name}/`, 注册为MCP stdio服务器
- 发现: 安全扫描是分层级的 — Python/Node.js/Rust/Shell各有特定banned模式。Python的 `ast.parse` 不能完全信任 (import路径在运行时才解析), `eval`/`exec` 字符串匹配可以有效减少攻击面
- 测试: 22个MCP registry测试全部通过 (包括 `test_publish_registers_user_server`)

### ADR-2026-07-01-16: Langevin+epsilon共存
**Context**: E8策略已使用epsilon-greedy进行离散探索概率控制
**Decision**: Langevin噪声sigma提供连续幅度控制, epsilon提供离散概率控制。二者正交, 可同时启用。Langevin通过 `alpha/beta` 调节, 不影响epsilon
**Impact**: 更细粒度的探索行为控制 — epsilon选目标, Langevin调步幅

### ADR-2026-07-01-17: 扩散检索 ≠ 向量近似
**Context**: FHRR的cosine相似度已提供精确最近邻
**Decision**: 扩散激活检索通过码本拓扑做消息传播, 而非向量空间相似度。二者互补: 向量检索找"内容上相似", 扩散检索找"结构上连通"
**Impact**: HyperCube获得图检索能力, 信息通过已建立的关系扩散

### ADR-2026-07-01-18: 运行时锻造安全门
**Context**: 允许agent自用代码创建工具引入任意代码执行风险
**Decision**: 四层安全门 — (1)语法检查防注入, (2)运行时特定banned pattern scan, (3)代码大小限制100K, (4)通过Python包装器隔离子进程执行
**Impact**: 代理可自创建工具, 同时风险可控

### Cycle 5-2: 后续深度吸收 (2026-07-01)

**#1 — GRPO优势集成 (policy.rs)**
- `neotrix-core/src/core/nt_core_policy.rs` (improved)
- 新增: `GrpoConfig` (group_size, clip_epsilon, kl_coef, entropy_coef, epochs), `reference_values`, `snapshot_reference()`, `kl_divergence()`, `policy_entropy()`, `softmax()` utility, `with_grpo()` builder, `grpo_update_epochs()`
- 改进: `grpo_update()` 现在使用PPO-style clipped surrogate objective + KL penalty + entropy bonus。重要性比率基于参考策略值, 裁剪到[1-ε, 1+ε]
- 发现: KL penalty通过软拉回参考值稳定训练; entropy bonus在熵下降时自动补偿; 多epoch训练通过重新快照参考策略实现稳定KL目标
- 测试: 12个新测试覆盖GrpoConfig, snapshot, KL, entropy, softmax, clipped GRPO更新, 多epoch

**B128 — Hebbian关联记忆图 (HyperCube)**
- `neotrix-core/src/core/nt_core_hcube/hebbian_memory.rs` (NEW)
- 原理: 加权共现图, Hebbian更新 Δw = lr · (target - w), 重要性门控 g = δ(imp > τ), 扩散激活检索, 指数衰减遗忘
- 发现: 无向边存储为双向有向边 (a→b 和 b→a), 边计数时除以2。重要性门控非常适合与PRM分数结合 (仅高置信度共现通过)
- 测试: 12个新测试覆盖节点添加, 共现, 无向性, 重要性门控, Hebbian逼近目标, 扩散检索1跳/2跳, 衰减, 剪枝, 批量更新

**B123 — Agent RL训练管线 (new module)**
- `neotrix-core/src/core/nt_core_agent_rl.rs` (NEW)
- 原理: `TrajectoryBuffer` (环形缓冲区+episode存储) + `RewardAggregator` (LATA折扣聚合+组归一化优势)。`build_grpo_batch()` 从最近episode构建GRPO组
- 发现: 行动级奖励 = 0.7 × 最终奖励 + 0.3 × 平均步骤奖励。LATA (sqrt(L)归一化) 防止长轨迹梯度坍缩。优势计算为组内归一化 (r-μ)/σ
- 测试: 12个新测试覆盖缓冲区推入/环溢出, episode管理, GRPO批构建, 折扣聚合, LATA, 优势归一化

### ADR-2026-07-01-19: GRPO配置可组合
**Context**: 不同训练场景需要不同的GRPO超参数
**Decision**: GrpoConfig作为独立可组合配置对象, 通过 `with_grpo()` builder注入。默认值适合一般场景, 但group_size/epochs可根据任务复杂度调整
**Impact**: 策略训练管线获得DPO-level的配置灵活度

### ADR-2026-07-01-20: Hebbian图是HRR超立方体的补充
**Context**: FhrrHyperCube提供精确VSA向量相似度, HebbianGraph提供共现关联
**Decision**: 二者互补。向量检索找"内容相似", Hebbian找"共现关联"。以后可在GWT中集成: VSA检索Top-K → Hebbian扩散激活补充结果
**Impact**: 记忆系统获得两种正交的检索模式

### ADR-2026-07-01-21: Agent RL管线直接喂养GRPO
**Context**: TrajectoryBuffer.build_grpo_batch() 输出格式与 policy.grpo_update() 的输入格式一致
**Decision**: Agent RL管线设计为GRPO组构建的前端。`build_grpo_batch()` 输出 `Vec<(ReasoningHexagram, f64)>` — 与 `grpo_update()` 签名完全匹配
**Impact**: 零胶合剂集成 — 收集轨迹 → 构建GRPO组 → 执行GRPO更新

### 实现统计
| 模块 | 文件 | 新增行 | 新增测试 | 状态 |
|------|------|--------|---------|------|
| #1 GRPO | policy.rs | ~150 | 12 | ✅ 57/57 pass |
| B128 Hebbian | hebbian_memory.rs | 315 | 12 | ✅ 12/12 pass |
| B123 Agent RL | nt_core_agent_rl.rs | 265 | 12 | ✅ 12/12 pass |

### 剩余盲点 (优先级排序)

| 优先级 | 盲点 | 影响 | 文件 |
|--------|------|------|------|
| P1 | GRPO优势集成 (#1) | SEAL训练无组级奖励归一化 | policy.rs |
| P1 | Hebbian关联记忆图 (B128) | 无模式关联存储 | HyperCube |
| P1 | Radix-Attention (B123) | 缓存效率低 | 新模块 |
| P2 | 结构化约束输出 | tool调用可靠性 | nt_io_provider |
| P2 | 语义缓存层 | LLM成本降低 | 新模块 |
| P2 | Agentic RAG | KB检索准确率 | nt_memory_kb |

## 未探索方向 (Cycle 5+)

1. **形式化验证**: E8过渡矩阵的TLA+ / Coq模型检查
2. **分布式E8共识**: 跨实例状态合并 (Raft on E8)
3. **心理理论(ToM)**: 对其他agent的认知建模
4. **梦/巩固深度**: 睡眠阶段的Hebbian重塑
5. **A2A协议**: Google/TensorLake的agent-to-agent标准
6. **形式化能力证明**: E8引擎收敛性, 泛化界
7. **神经形态硬件**: Loihi 2上部署E8+VSA推断
8. **差分隐私**: 过程记忆的ε-差分隐私保护
