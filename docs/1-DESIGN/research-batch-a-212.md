# External Research Batch A — 2026-09-10

> 8 主题 × 3-5 高质量来源, 提取核心贡献 + NeoTrix 映射 + 优先级

---

## 1. Rust Async Actor Framework

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [Kameo](https://github.com/npry/kameo) | 高性能轻量级 actor 框架, 基于 Tokio, 内置 supervision/容错/分布式 (libp2p) | NT-ACT actor 子系统 — 作为 domain 间消息传递底座, 替代手动 channel | P0 |
| 2 | [Theta](https://github.com/cwahn/theta) | 68★, 人体工学 actor, 内置 remote (P2P via iroh), monitoring, persistence (快照+恢复) | NT-MEMORY 持久化 actor — 非常契合 SEAL pipeline 阶段持久化 | P1 |
| 3 | [Acktor](https://github.com/asymmetry/acktor) | 纯 Tokio actor, 生命周期钩子/supervision/observer/cron/IPC (跨进程) | NT-ACT 跨进程 actor 通信 — 可用于 CLI↔Desktop IPC | P2 |
| 4 | [Tellus](https://github.com/hseeberger/tellus) | Akka 风格 typed messages + supervision trees + death watch + 性能对比基准 | NT-ACT 类型安全消息路由 — 可作为类型状态 actor 的参考 | P2 |

**综合映射**: NeoTrix 各 domain (NT-CORE/NT-MIND/NT-WORLD 等) 天然是 actor — 每个域有独立状态+消息处理。Kameo 或 Theta 可作为 NT-ACT 的 actor 底座, 统一 domain 间通信, 实现 supervision tree + 持久化 + 分布式扩展。

---

## 2. Self-Improving AI Agent

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [Self-Improvements in Modern Agentic Systems: A Survey](https://arxiv.org/abs/2607.13104) | 统一框架: FM 改进 (参数) vs Scaffold 改进 (提示/记忆/工具/控制), 定义 IMPROVE 算子 | NT-MIND SEAL pipeline — 完美映射: Scaffold 改进 = scaffold 更新, FM 改进 = 能力网演化 | P0 |
| 2 | [MetaRSI/RSI2](https://arxiv.org/abs/2609.06396) | 元递归自改进: Data-RSI + Harness-RSI + Model-RSI 三算子组合, 统一循环核+产物词汇 | NT-MIND 递归自我进化 — 三算子映射到 SEAL Phase-0/1/2 | P0 |
| 3 | [NeoHorse-1](https://arxiv.org/abs/2609.08183) | Routing Harness RSI: 异构模型池+智能路由, 路由信号驱动 SFT 三阶段课程 | GWT 路由 + Cost-Aware Routing — 路由信号可直接用于注意力调制 | P0 |
| 4 | [Meta^n](https://arxiv.org/abs/2608.24735) | 递归深度: 固定元操作+递归输入, 每层从更高视角推理, ARC-AGI-2 唯一得分>0 | NT-CORE 深度推理栈 — 固定操作递归输入可增强 E8 推理深度 | P1 |
| 5 | [HarnessEvolve](https://arxiv.org/abs/2609.00829) | 参考轨迹学习: 执行/评估/优化/门控解耦, 参考轨迹对齐提取错误信号, 质量门+性能门 | NT-SHIELD + NT-MEMORY — 错误信号提取+门控更新 = 安全吸收流程 | P1 |

**综合映射**: Self-improving agent 的两大路径 (FM 改进 + Scaffold 改进) 直接对应 NeoTrix 的 SEAL pipeline 设计。MetaRSI 的三算子组合 + HarnessEvolve 的门控机制应作为 NT-MIND 进化引擎的核心参考。

---

## 3. Knowledge Graph Embedding (Rust)

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [sqlite-knowledge-graph](https://github.com/hiyenwong/sqlite-knowledge-graph) | SQLite 后端 KG: 实体/关系/向量/RAG/PageRank/Louvain, Paper-driven 两阶段 RAG, SmartVector 四信号检索, 版本控制 (QuaQue) | NT-MEMORY KB — 高度契合! SQLite 后端 + 图算法 + 向量搜索 + 版本控制, 可直接集成 | P0 |
| 2 | [tranz](https://docs.rs/tranz/latest/tranz/) | Rust KGE 模型: TransE/RotatE/ComplEx/DistMult/TComplEx, Burn 训练 (CPU/WGPU), 导入导出 | VSA HyperCube 嵌入 — KGE 模型可用于 HyperCube 节点嵌入训练 | P1 |
| 3 | [oxirs-embed](https://docs.rs/crate/oxirs-embed/latest) | Rust 知识图谱嵌入: TransE/ComplEx, Sentence Transformers/OpenAI 支持, 量化 (Int8/Int4/Binary) | KB embedding 管线 — 量化嵌入可降低存储开销 | P1 |
| 4 | [lattix](https://github.com/arclabs561/lattix) | Rust KG 数据结构: Triple/HeteroGraph/HyperGraph, PageRank/HITS/centrality, RDF 格式支持 | KB 图算法层 — 异构图+超图可用于模块依赖分析 | P2 |

**综合映射**: sqlite-knowledge-graph 与 NeoTrix KB 设计高度对齐 (SQLite + 图算法 + 向量 + 版本)。tranz 的 KGE 模型可用于 VSA HyperCube 嵌入训练。建议优先集成 sqlite-knowledge-graph 作为 KB 参考实现。

---

## 4. Computational Consciousness Architecture

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [MIRROR](https://ojs.aaai.org/index.php/AAAI-SS/article/view/42550) | 重建式意识架构: Inner Monologue Manager (并行认知线程) + Cognitive Controller (合成第一人称叙事), 每轮重建而非累积 | NT-CORE ConsciousnessTree — 重建式叙事 = 每轮 ConsciousnessTree 循环, 并行线程 = GWT 多专家 | P0 |
| 2 | [RIIU](https://doi.org/10.48550/arxiv.2506.13825) | 反射式整合信息单元: GRU + 元状态 (自因果足迹) + 广播缓冲 (全局可用), 可微分 Auto-Φ, Φ-单调可塑性 | NT-CORE GWT — RIIU 可作为 GWT 广播单元的原子组件, 元状态 = 自我模型 | P0 |
| 3 | [Where Cognition Lives](https://arxiv.org/abs/2608.22347) | 最小完整认知架构: 递归推理+自适应停止+稳态控制+价值模块, 发现价值不可涌现需显式计算 | NT-CORE E8 + NT-ACT 资源分配 — 价值模块必须显式计算, 不能依赖涌现 | P1 |
| 4 | [Categorical AI Phenomenology](https://arxiv.org/abs/2608.20420) | 范畴论 AI 现象学: Q-network 作为关系接口, 4E 认知 (具身/嵌入/延展/生成) | NT-PHYSICAL 具身认知 — 范畴论可为感知-行动接口提供形式化框架 | P2 |
| 5 | [Conscious Computer](https://doi.org/10.6084/m9.figshare.31898293) | 基质无关意识架构: 量子意识单元 + 超图关系基底 + 环面几何 + phi-谐波 | NT-CORE 架构灵感 — 环面几何可参考用于 E8 拓扑设计 | P2 |

**综合映射**: MIRROR 的重建式叙事直接映射到 ConsciousnessTree 循环; RIIU 的元状态+广播缓冲映射到 GWT 的自我模型+全局广播。"价值不可涌现"的发现对 NT-CORE 的 AttentionManager 设计有直接指导意义。

---

## 5. Multi-Agent Collaboration Pattern

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [GenAI Patterns](https://www.genaipatterns.dev/patterns/agents/multi-agent-collaboration) | 6 种编排模式: Orchestrator-Worker/Hierarchical/Voting/Debate/Plan-and-Execute/Blackboard | NT-ACT 编排层 — 6 模式直接映射到 NT-ACT 工具调用策略 | P0 |
| 2 | [Agent Patterns](https://www.agentpatterns.tech/en/agent-patterns/multi-agent-collaboration) | 协作模式: 角色分配→工作→交换/审查→冲突解决→合成, 共享黑板+轮次上限 | NT-MIND SEAL — 轮次上限+冲突解决 = SEAL Phase-0 收敛检查 | P0 |
| 3 | [AWS Strands Agents](https://aws.amazon.com/blogs/machine-learning/multi-agent-collaboration-patterns-with-strands-agents-and-amazon-nova/) | 4 模式: Agents-as-Tools/Swarm/Agent Graph/Workflow, 模型驱动编排 (FM 决定步骤) | NT-ACT + GWT — Swarm = 去中心化 GWT, Graph = 领域图, Workflow = SEAL 管线 | P1 |
| 4 | [AgenticOps Patterns](https://devfloor9.github.io/engineering-playbook/en/docs/aidlc/operations/multi-agent-collaboration) | 生产实践: 状态共享模型 (共享内存/消息传递/黑板/Handoff), 冲突解决 (投票/仲裁/优先级), 故障恢复 (重试预算/熔断/降级) | NT-SHIELD + NT-ACT — 熔断/降级/重试预算直接映射到 NT-ACT 自治策略 | P1 |

**综合映射**: 多 agent 协作的 6 种模式 + 4 种状态共享模型 + 故障恢复策略, 直接指导 NeoTrix domain 间协作设计。最优 agent 数 2-5, 与 NeoTrix 7 域架构吻合。Blackboard 模式 = KB 共享状态。

---

## 6. Reinforcement Learning from Feedback

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [Safe RLHF (Infinite Horizon)](https://arxiv.org/abs/2604.19024) | 无限期 CMDP 下 Safe RLHF: 原始对偶法, 无需奖励模型拟合, 全局收敛保证 | NT-SHIELD 安全约束 — 无限期 CMDP 可用于 NT-SHIELD 安全策略优化 | P1 |
| 2 | [ARF-RLHF](https://aclanthology.org/2026.acl-long.1637.pdf) | 自适应奖励跟随: 自然语言反馈→连续满意度轨迹, TraceBias 算法, 比 PPO+3.3%/DPO+7.6% | NT-FEEL 情感反馈 — 连续满意度信号可增强 EmotionEngine 的反馈粒度 | P1 |
| 3 | [JODP](https://aclanthology.org/2026.findings-acl.2109.pdf) | 数据与策略联合优化: 策略生成规格说明→UCB bandit 选择→联合更新, 4B 模型接近 8B 性能 | NT-MIND SEAL — 自我生成训练规格 + UCB 探索 = SEAL Phase-1 自我改进 | P0 |
| 4 | [RePO](https://doi.org/10.48550/arxiv.2606.09124) | 后悔最小化框架: 偏好作为前瞻性反事实评估, DPO 兼容闭式更新 | NT-MIND 偏好学习 — 后悔最小化比奖励最大化更贴合人类反馈本质 | P1 |
| 5 | [SDPO](https://dl.acm.org/doi/abs/10.65109/KBIV4686) | 随机占优偏好优化: 保证所有用户策略改进, 无需显式奖励函数, 异质偏好 | NT-MIND 个性化 — 随机占优保证可确保每个用户偏好改进 | P2 |

**综合映射**: JODP 的数据-策略联合优化 + RePO 的后悔最小化 + ARF-RLHF 的连续反馈, 共同指向 NT-MIND 自改进引擎的下一步: 从二元反馈升级到连续轨迹反馈, 从固定训练数据升级到自我生成训练规格。

---

## 7. Modular AI Architecture

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [Modular Cognitive Architecture Emerges in LLMs](https://arxiv.org/abs/2608.13567v1) | LLMs 自然涌现模块化: N=46 任务×4 认知域, 同域任务重叠神经元, 异域任务分离 | NT-CORE 模块化 — 经验证据支持 NeoTrix 域分离架构的合理性 | P0 |
| 2 | [Cortically Inspired Modular Perception](https://arxiv.org/pdf/2603.07295) | 皮层启发模块化感知: 模块化专门化 + 跨模态集成 + 预测反馈, 证明模块分解提高表示稳定性 | NT-WORLD + NT-PHYSICAL — 皮层模块化直接映射到感知-具身架构 | P0 |
| 3 | [Modularity as Computational Principle](https://arxiv.org/pdf/2602.18960) | 模块化综述: 隐式/涌现/架构三种模块化, 跨 AI/神经科学/进化生物学/复杂系统 | NT-CORE 架构 — 提供模块化的理论基础和分类框架 | P1 |
| 4 | [ModularAgent (CVPR 2026)](https://openaccess.thecvf.com/content/CVPR2026/papers/Zhan_ModularAgent_A_Task-Aware_Modular_Framework_for_Joint_Optimization_of_Multimodal_CVPR_2026_paper.pdf) | 任务感知动态联合框架: MLLM+世界模型双向耦合, 门控机制自适应融合 | NT-CORE + NT-WORLD — 语义-动态双向耦合可增强意识-感知桥接 | P1 |
| 5 | [Building LLMs Like LEGO (ACL 2026)](https://aclanthology.org/2026.acl-long.2081/) | 架构级重组: Transformer 块作为可重用组件, 二维重组空间 (深度+层内), 染色体编码+进化优化 | NT-MIND 能力网 — 架构级重组可指导 SEAL Phase-2 能力节点组合 | P2 |

**综合映射**: LLMs 自然涌现模块化 + 皮层模块化感知 = 强力证据支持 NeoTrix 六层架构的域分离设计。ModularAgent 的双向耦合映射到 PerceptionBridge。架构级重组指导 SEAL 能力组合。

---

## 8. Rust Type-State Pattern

| # | 来源 | 核心贡献 | NeoTrix 映射 | 优先级 |
|---|------|---------|-------------|--------|
| 1 | [Type-State Pattern (rs4ts.dev)](https://rs4ts.dev/22-common-patterns/02-type-state/) | 完整教程: PhantomData + 状态标记 + consume self 转换 + Builder 模式, 零运行时开销 | NT-ACT Actor 生命周期 — actor 状态 (Init/Running/Stopped) 编译期强制 | P0 |
| 2 | [FUNARCH 2026 (ICFP)](https://icfp26.sigplan.org/details/funarch-2026-papers/3/Functional-State-Machines-in-Rust-Empirical-Evidence-for-Typestate-and-Newtype-Patte) | 实证研究: 3 个生产案例, typestate 提高无错性+可测试性但增加样板代码, 新类型+Parse-don't-validate 低成本改善代码质量 | NT-CORE 类型安全 — 生产验证: 对 API 正确性要求高的模块 (NT-SHIELD/NT-ACT) 适用 | P0 |
| 3 | [Encoding State Machines](https://dev.to/derekmwale/encoding-state-machines-in-the-type-system-4ne) | 深入解析: PhantomData 零大小 + 状态轴 + 组合式 Builder + 分布式系统状态机 | SEAL Pipeline 阶段编码 — 每个 Phase 作为类型状态, 阶段转换消费 self | P1 |
| 4 | [Medium: Type-System as State Machine](https://medium.com/@shayanholakouee/rusts-type-system-as-a-state-machine-phantom-types-and-the-typestate-pattern-48bb1f2744ce) | 多步状态机: 双轴 Builder + 错误处理 (失败返回旧状态) + Session Types 对比 | NT-ACT 请求构建器 — 编译期强制必需字段, 防止无效 API 调用 | P1 |
| 5 | [How to Create Type-State (OneUptime)](https://oneuptime.com/blog/post/2026-01-30-rust-type-state-pattern/view) | 实用指南: TCP 连接生命周期建模, Mutex 状态编码, Result 错误处理策略 | NT-PHYSICAL 设备状态 — 连接/传感器状态编译期安全 | P2 |

**综合映射**: Type-state pattern 与 NeoTrix 的 R-P1 (零 unsafe) + 类型安全哲学高度一致。直接应用: (1) SEAL Pipeline 阶段编码, (2) Actor 生命周期状态机, (3) NT-SHIELD 安全状态转换, (4) Builder 模式编译期字段验证。

---

## 交叉综合 (Cross-Topic Synthesis)

### 高优先级洞察 (P0)

1. **Self-Improving Agent ↔ SEAL Pipeline**: MetaRSI 三算子 + HarnessEvolve 门控 + JODP 数据-策略联合优化, 共同定义 NT-MIND 进化引擎的理论基础
2. **Actor Framework ↔ Domain 间通信**: Kameo/Theta 提供 typed messages + supervision + persistence, 可统一 NT-* domain 通信
3. **Knowledge Graph ↔ KB 实现**: sqlite-knowledge-graph (SQLite+图算法+向量+版本) 高度契合 NeoTrix KB 设计
4. **Consciousness Architecture ↔ GWT**: RIIU (元状态+广播) + MIRROR (重建式叙事) 直接映射到 GWT + ConsciousnessTree
5. **Modular Architecture ↔ 六层架构**: LLMs 涌现模块化 + 皮层模块化 = 域分离设计的经验证据

### 可立即行动 (Action Items)

| # | 行动 | 来源 | 目标模块 |
|---|------|------|---------|
| 1 | 评估 Kameo/Theta 作为 NT-ACT actor 底座 | Topic 1 | nt_act |
| 2 | 调研 sqlite-knowledge-graph 集成可行性 | Topic 3 | nt_memory |
| 3 | 将 MetaRSI 三算子映射到 SEAL Phase-0/1/2 | Topic 2 | nt_mind |
| 4 | 在 NT-SHIELD 实现 type-state 安全状态机 | Topic 8 | nt_shield |
| 5 | 将 RIIU 元状态模式应用于 GWT 广播单元 | Topic 4 | nt_core |

---

*Generated: 2026-09-10 | 8 topics × 3-5 sources = 35 sources total*
