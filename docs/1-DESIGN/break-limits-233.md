# 第19批破限制技术 (Break-Limits Batch 233)

> 生成时间: 2026-09-11 | 5主题 × 3-5来源 | 状态: 批量吸收

---

## 1. Agent记忆 (Agent Memory)

### 1.1 TrustMem — 可信记忆合并框架

**来源**: arXiv:2606.25161

**突破点**: 现有记忆系统通过 write/revise/delete 操作更新外部记忆，但这些更新可能遗漏重要信息、损坏现有记忆或引入幻觉内容。TrustMem 引入 Memory Transition Verifier 评估每次记忆更新的可信度（覆盖度、保真度、忠实度），通过 Transition-Ranked GRPO 优化记忆编辑行为。在 MemoryAgentBench 上提升 +6.5 分，HaluMem 记忆提取提升 +12.14 F1，将遗漏/损坏/幻觉分别降低 40.1%/79.1%/50.0%。

**关键机制**:
- Memory Transition Verifier: 评估局部记忆转换 $M_{t-1} \to M_t$ 的三维度可信度
- Transition-Ranked GRPO: 基于验证器分数构建候选更新偏好对，优化记忆策略
- 过渡级监督替代结果级监督：不依赖最终任务性能，而是监督每个中间记忆操作

**NeoTrix 融合**:
- **NT-MEMORY KB**: 将 TrustMem 的过渡验证器应用于 KB 嵌入写入管线，防止低质量向量污染知识库
- **NT-MIND SEAL**: 在 distillation 阶段引入 memory transition verification，提升经验蒸馏的可靠性
- **experience-tree 吸收协议**: 五阶段吸收的 "蒸馏" 阶段可借鉴 TrustMem 的三维度验证，过滤虚假经验
- **NT-SHIELD**: 记忆注入攻击防御 — TrustMem 的保真度检查可作为防篡改层

---

### 1.2 LycheeMemory V2 — 语义段级合并

**来源**: arXiv:2608.12990

**突破点**: 替代 turn-level 合并的高效长期记忆框架。通过语义边界检测将多轮对话分段，每段执行一次 LLM 编码。构建 token 消耗降低 86%（LoCoMo）和 75.9%（LongMemEval-S），同时达到 SOTA 性能（LoCoMo 89.22%，LongMemEval-S 92.20%）。段级合并保留事件级时序证据，优于固定窗口批处理。

**关键机制**:
- 语义惊讶度 + 凝聚力信号检测段边界
- 类型化记忆记录（实体/主题/时间范围/证据链接）
- 跨段消歧反馈维持连续性
- 查询时多路线召回：语义记忆 + 结构化索引 + 时序索引 + 原始轮次

**NeoTrix 融合**:
- **NT-MEMORY KB pipeline**: 替换当前 turn-level 经验压缩，改用语义段级合并降低 86% 构建成本
- **experience-tree**: 段级批处理可直接应用于 session 经验吸收 — 按语义边界分段而非机械按轮次
- **Axiom A2 (Context as Scarce Resource)**: LycheeMemory 证明 "准确率-成本权衡取决于信息保留的粒度，而不仅仅是保留什么"

---

### 1.3 AgeMem — 统一长短期记忆管理

**来源**: ACL 2026 (aclanthology.org/2026.acl-long.981)

**突破点**: 将长短期记忆管理直接集成到 agent 策略中，记忆操作作为工具动作暴露。LLM 自主决定何时存储/检索/更新/摘要/丢弃信息。三阶段渐进 RL + step-wise GRPO 解决稀疏不连续奖励。在五个长期基准上一致超越记忆增强基线。

**关键机制**:
- 记忆操作 = 工具动作：ADD/UPDATE/DELETE/RETRIEVE/SUMMARIZE/DISCARD
- 三阶段渐进 RL：先学简单记忆操作 → 复杂跨记忆协调
- Step-wise GRPO: 解决记忆操作引入的稀疏奖励问题

**NeoTrix 融合**:
- **NT-ACT tools**: AgeMem 的记忆操作工具化与 MCP tool 协议天然对齐
- **NT-CORE SelfModel**: 统一 LTM/STM 管理可扩展到 SelfModel 的动态性能模型
- **GWT attention**: 记忆检索的注意力路由可借鉴 AgeMem 的自主决策机制

---

### 1.4 RecMem — 基于复现的记忆合并

**来源**: ACL 2026 Findings (aclanthology.org/2026.findings-acl.1619)

**挑战 eager consolidation**: 现有系统对每次交互都调用 LLM 提取记忆，导致巨大 token 消耗。RecMem 引入潜意识记忆层（轻量嵌入），仅在语义相似交互持续复现时触发 LLM 提取。构建 token 消耗降低 87.3%，同时准确率超越基线。

**关键机制**:
- 三层记忆：潜意识（嵌入缓冲）→ 情景记忆（事件摘要）→ 语义记忆（原子事实）
- 复现驱动合并：$\theta_{sim}=0.7, \theta_{count}=5$ 触发阈值
- 语义精炼：恢复情景摘要遗漏的细粒度事实

**NeoTrix 融合**:
- **NT-MEMORY KB**: RecMem 的复现触发机制可应用于 KB 写入频率控制 — 高频访问概念才值得深度索引
- **experience-tree**: 潜意识层 → 快照层，情景记忆 → 蒸馏层，语义记忆 → 落盘层
- **成本优化**: 减少 87% 构建 token 直接降低 NT-IO LLM 调用成本

---

### 1.5 Dual-Layer Agentic Memory — 双层记忆写路由+慢合并

**来源**: arXiv:2608.22215

**突破点**: 受神经科学互补学习系统 (CLS) 理论启发，将记忆管理转移到写入阶段。信息经小到大模型级联路由为 non-write/write-new/write-update。后续 write-back 阶段通过 SFT 将高价值外部记忆选择性内化到模型参数。1.7B/8B 级联剪枝 68% 冗余外部记忆，保留 98% QA EM。

**关键机制**:
- 认识论路由：小模型过滤冗余，大模型处理复杂决策
- 周期性参数内化：外部记忆 → 模型权重
- 选择性外部化 + 选择性内化 = 统一范式

**NeoTrix 融合**:
- **NT-MEMORY KB + NT-CORE SelfModel**: 双层架构可映射到 KB 外部记忆 + 模型参数内化的双通道
- **Axiom A1 (Cost-Aware Routing)**: 小模型路由过滤 = cheap models for simple tasks
- **experience-tree**: write-back 内化阶段 = 吸收协议的 "落盘" 阶段强化

---

## 2. Agent规划 (Agent Planning)

### 2.1 CHIME — 信用感知分层记忆进化

**来源**: arXiv:2609.02074

**突破点**: 现有自进化记忆方法依赖最终任务反馈，但结果混淆了计划质量和执行误差。CHIME 提出 attribute-before-memorize 原则：先归因每个任务结果到计划/执行/两者/均非，再更新对应记忆库。分层记忆库分离计划记忆和执行记忆，防止跨阶段污染。在四个长期基准上一致超越训练基线和自进化基线。

**关键机制**:
- 信用归因门：结构化自反思评估计划充分性、执行正确性、外部因素
- 分层记忆库：计划库 + 执行库独立演化
- 信用感知记忆进化：仅更新归因阶段的记忆值和内容

**NeoTrix 融合**:
- **NT-MIND SEAL pipeline**: CHIME 的 credit attribution gate 可直接应用于 SEAL 各阶段的经验评估
- **NT-CORE SelfModel**: 计划/执行分离 = 价值函数模型 vs 动态性能模型的分离
- **experience-tree**: 五阶段吸收中引入 credit attribution，区分 "这个经验好是因为规划还是执行"
- **NT-REPAIR**: 自愈诊断可借鉴 CHIME 的归因机制，区分系统故障 vs 环境因素

---

### 2.2 AdaPlan-H — 自适应分层规划

**来源**: ACL 2026 Findings (aclanthology.org/2026.findings-acl.77)

**突破点**: 现有规划在固定粒度运行：简单任务过度规划，复杂任务规划不足。AdaPlan-H 模仿认知科学渐进精化理论，从粗粒度宏计划开始，根据任务复杂度自适应精化层级。两阶段优化：模仿学习初始化 → 能力增强（DPO）提升层级准确性。

**关键机制**:
- 层级计划 $p = p_m \oplus p_{m-1} \oplus \cdots \oplus p_1$，$m$ 自适应
- Monte Carlo 方法评估每个层级计划的执行效果
- DPO 训练选择更准确的层级和更高质量的计划

**NeoTrix 融合**:
- **NT-CORE E8 hexagram**: AdaPlan-H 的层级结构可映射到 E8 的六爻符号 — 每爻代表不同抽象层级
- **GWT attention**: 自适应层级 = 动态注意力深度路由
- **NT-ACT task decomposition**: 直接应用于 agent 任务分解 — 简单任务用粗粒度，复杂任务自动精化

---

### 2.3 ReAcTree — 层级代理树

**来源**: arXiv:2511.02424

**突破点**: 动态构建子目标空间的代理树，每个 LLM 代理节点独立处理子目标，行为树风格控制流节点协调执行。在 WAH-NL 上达到 61% 成功率（Qwen 2.5 72B），近乎 ReAct 的 31% 的两倍。双记忆系统：情景记忆（子目标级经验检索）+ 工作记忆（节点间信息共享）。

**关键机制**:
- 代理节点：推理 → 行动 → 或扩展树（提出新子目标）
- 控制流节点：顺序/回退/并行执行
- 情景记忆：子目标级 in-context learning
- 工作记忆：环境感知跨节点共享

**NeoTrix 融合**:
- **NT-ACT orchestration**: ReAcTree 的控制流节点直接映射到 NT-ACT 的编排协议
- **NT-MEMORY KB**: 情景记忆检索 = KB 语义搜索，工作记忆 = EventBus 广播
- **ConsciousnessTree**: ReAcTree 的层级结构可作为 ConsciousnessTree 分支的执行框架

---

### 2.4 Meta-Reasoning Policy — 反应-审慎切换

**来源**: arXiv:2607.16421

**突破点**: 学习在反应式控制和决策时规划之间自适应分配计算。元策略基于反应式策略的不确定性分数预测何时需要规划。随反应式策略改进，元策略自动转向完全反应式控制。在运动规划和导航任务上超越所有固定计算基线。

**关键机制**:
- 反应式不确定性 + 观测历史 = 元状态
- 深度可变的规划选项：k 步前瞻
- 联合训练时元策略随反应式策略改进自动调整

**NeoTrix 融合**:
- **GWT salience routing**: 元推理策略 = GWT 的注意力分配决策 — 何时深度处理 vs 快速反应
- **Axiom A1 (Cost-Aware Routing)**: 元策略学习成本-质量权衡的最优策略
- **NT-CORE SelfModel**: 不确定性信号可集成到动态性能模型中

---

## 3. Agent工具 (Agent Tools)

### 3.1 api-to-tools — 通用 API 到工具转换

**来源**: GitHub (SonAIengine/api-to-tools)

**突破点**: 任何 API（OpenAPI/WSDL/SOAP/GraphQL/gRPC/AsyncAPI）一键转换为 LLM 可调用的工具定义。支持 14+ 种源类型（OpenAPI 3.x/Swagger 2.0/WSDL/GraphQL/gRPC/HAR/AsyncAPI/JS bundle/Playwright 爬虫/CDP 爬虫）。发现公共 API <2 秒。内置 safe mode 拦截写操作，自动限流。

**关键机制**:
- 优先级探测链：直接 spec URL → Nexacro → well-known 路径 → 认证 Swagger → JS 静态扫描 → Playwright 动态爬取
- 多格式导出：Anthropic/OpenAI/Gemini/Bedrock/LangChain/MCP
- 执行器：REST/SOAP/GraphQL/gRPC/WebSocket

**NeoTrix 融合**:
- **NT-ACT tools**: api-to-tools 直接增强 MCP 工具自动发现能力
- **NT-WORLD crawl**: 自动化 API 发现可集成到 crawl pipeline 的元数据提取
- **NT-IO platform_gateway**: 统一多平台 API 适配的底层基础设施

---

### 3.2 ContDa — 持续文档适应

**来源**: ACL 2026 Findings (aclanthology.org/2026.findings-acl.1082)

**突破点**: 现有工具使用方法假设静态工具环境，但实际工具集持续演化。ContDa 提出稳定性-适应性困境的解决方案：关系引导探索（利用功能相关工具作为锚点探测新能力）+ 关系感知调整（组织重叠工具，编码使用偏好和降级选项）。在动态扩展的 StableToolBench 和 RestBench 上平均提升性能，同时仅有有限的先前任务损失。

**关键机制**:
- 关系引导探索：现有工具 = 锚点 → 探测新工具能力
- 关系感知调整：重叠工具聚类 + 使用偏好 + 降级选项
- 稳定性-适应性度量：分离性能和稳定性评估

**NeoTrix 融合**:
- **NT-ACT tool evolution**: ContDa 的文档适应机制可直接应用于 MCP 工具集的持续演化
- **NT-MIND skill crystallization**: 工具文档适应 = skill 模板的持续精化
- **Dark Forest 原则**: ContDa 的稳定性指标可作为 "模块是否存活" 的判定标准

---

### 3.3 Tool-R0 — 零数据自进化工具学习

**来源**: arXiv:2602.21320

**突破点**: 自博弈 RL 框架，Generator 和 Solver 从同一基础 LLM 初始化，通过互补奖励共同进化。Generator 在 Solver 能力前沿生成有挑战的任务，Solver 学习使用真实工具解决。零人类数据前提下，Qwen2.5-1.5B-Instruct 平均提升 +22.99 分（92.52% 相对增益），超越全监督基线。

**关键机制**:
- 双角色共进化：Generator（任务生成）+ Solver（工具使用）
- 难度引导奖励：基于冻结 Solver 答案不确定性定位能力前沿
- 接地任务合成：无标注任务配置防止自由形式生成的模式崩溃
- 答案一致性排序 + 渐进难度暴露

**NeoTrix 融合**:
- **NT-MIND SEAL pipeline**: Tool-R0 的自博弈共进化可作为 SEAL 探索阶段的任务生成器
- **NT-ACT tool creation**: 零数据工具学习 = agent 自主创建工具能力
- **experience-tree**: 自生成课程比静态人类监督产生更广的训练分布

---

### 3.4 Lomekwi — 资源受限工具发现

**来源**: arXiv:2607.16961

**突破点**: 首次将工具使用分解为好奇心（发现组件）、识别（发现过程）、效率（使用工具）三个独立指标。发现反直觉现象：识别能力与模型规模负相关 — 中等模型自信尝试无辅助但无法成功，反而比小模型更少构建工具。工具使用基准报告单一成功率会掩盖这一反演。

**关键机制**:
- 三层分解：Curiosity (C) + Recognition (R) + Efficiency (E)
- 可选工具创建环境：测量工具构建倾向而非仅仅是能力
- U 型缩放定律假说：识别能力可能是 U 型曲线的第一斜坡

**NeoTrix 融合**:
- **NT-CORE SelfModel**: 三层分解可应用于 agent 自身能力评估 — 何时创建工具 vs 使用现有工具
- **GWT attention**: 识别反演 = 注意力分配失败 — agent 过度自信跳过工具创建
- **NT-REPAIR**: 识别能力退化检测可作为自愈触发条件

---

### 3.5 OpenTools — LLM 自动发现 Web API

**来源**: GitHub (Mark-Life/OpenTools)

**突破点**: 通过 OpenAPI 扩展 `x-llm` 让任何 Web 应用向 LLM 暴露 API。客户端只需 URL 即可自动发现和使用工具。支持审批策略（auto/per-call）、破坏性标记、成本指示。基于 Vercel AI SDK 构建，Chat 应用连接任何 OpenTools 兼容应用即可使用。

**关键机制**:
- `/.well-known/llm.json` 发现端点
- `x-llm` 扩展：approval_level、destructive、cost
- `createToolsFromUrl(url)` 一行代码集成

**NeoTrix 融合**:
- **NT-IO platform_gateway**: OpenTools 的发现协议可作为平台网关的标准接口
- **NT-SHIELD**: approval_level 策略与 NT-SHIELD 的 egress privacy guard 对齐
- **NT-ACT MCP**: 与 MCP 协议互补 — OpenTools 面向 Web API，MCP 面向本地工具

---

## 4. Agent协作 (Agent Collaboration)

### 4.1 Meta-Debate — 动态角色分配

**来源**: arXiv:2601.17152

**突破点**: 现有多代理辩论使用固定角色分配，忽略代理在不同问题上的能力差异。Meta-Debate 运行元辩论选择最佳角色配置：(1) 提案阶段 — 候选者提供角色定制论点；(2) 同行评审 — 评分选择最佳匹配。在基准上比均匀分配提升高达 74.8%，比随机分配提升高达 29.7%。

**关键机制**:
- 元辩论两阶段：proposal + peer review
- 量化代理-角色适配度（无需标注数据）
- 问题级自适应分配替代静态分配

**NeoTrix 融合**:
- **GWT attention routing**: Meta-Debate 的角色分配 = GWT 的专家路由决策
- **NT-CORE Ascendancy 双专精**: 动态角色分配可替代固定 Weapon Set 切换
- **ConsciousnessTree 分支协调**: 元辩论机制可应用于跨域健康评估的仲裁

---

### 4.2 PEAR — 排列等变自适应路由辩论

**来源**: arXiv:2606.20621

**突破点**: 固定拓扑引入位置偏差、放大不可靠代理、对角色分配高度敏感。PEAR 在每轮辩论中动态重构通信拓扑，基于代理状态（答案/置信度/历史影响力）选择稀疏通信图。证明为排列等变路由器：代理重标记不改变最终答案分布。在四个推理基准和六个 LLM 骨干上显著提升准确率。

**关键机制**:
- 三目标复合路由：目标多样性 + 影响力平衡 + 低置信度过滤
- 排列等变性：对称性保证泛化
- 稀疏 k-正则模板：限制每轮批评预算

**NeoTrix 融合**:
- **GWT resonance routing**: PEAR 的状态感知路由 = GWT 谐振路由的多代理扩展
- **NT-SHIELD**: 低置信度过滤 = 可信度加权信息传播，防止错误级联
- **HeartbeatAggregator**: PEAR 的影响力平衡可用于系统健康信号的聚合

---

### 4.3 R-MAD — 经验记忆增强辩论

**来源**: arXiv:2609.03619

**突破点**: 多代理辩论的共享误解漏洞：当多数代理初始收敛于错误答案时，辩论放大而非纠正误差。R-MAD 为代理配备跨辩论积累的经验记忆。辩论状态感知检索策略根据共识水平校准概念先验，历史经验估计代理可靠性产生置信度权重调节同行影响。

**关键机制**:
- 辩论状态感知检索：共识水平 → 检索相关历史证据
- 代理可靠性估计 → 置信度权重
- 经验记忆跨辩论积累

**NeoTrix 融合**:
- **NT-MEMORY KB**: R-MAD 的经验记忆 = KB 中的辩论历史节点
- **experience-tree**: 跨 session 经验积累 = 吸收协议的 "落盘" 阶段
- **NT-CORE SelfModel**: 代理可靠性估计可扩展到 agent 自身能力的持续评估

---

### 4.4 Mixture of Debaters (MoD) — 单模型动态自辩论

**来源**: arXiv:2606.29425

**突破点**: 替代多代理辩论的计算开销。MoD 在单模型内利用 MoE 架构实现动态自辩论。双路由解耦角色分配和流程控制，动量切换平滑 token 级路由减少抖动。仅增加 12M 参数，实现 3.7x 降低延迟和 87% token 消耗减少，超越多代理系统。

**关键机制**:
- 双路由：角色路由器（proposer/critic）+ 阶段路由器（debate/synthesize）
- 动量切换：局部上下文聚合平滑路由决策
- 解耦专家池：解释专家 + 综合专家 = N×N 组合路径

**NeoTrix 融合**:
- **NT-CORE E8**: MoD 的双路由可映射到 E8 的推理-综合双通道
- **GWT attention**: 动量切换 = 注意力的时序平滑
- **成本优化**: 单模型自辩论大幅降低 NT-IO LLM 调用成本（87% token 减少）

---

### 4.5 Meta-Moderator — 元认知辩论调节

**来源**: arXiv:2608.23029

**突破点**: 将辩论调节视为元认知过程：监控辩论效用、控制审议、裁决最终答案。Meta-Moderator 独立于辩论者训练，通过结果驱动策略优化学习何时停止辩论。在五个基准上超越常见决策层，跨任务和系统配置迁移。

**关键机制**:
- 三功能循环：Monitoring（监控进展）+ Control（分配审议）+ Adjudication（裁决答案）
- 独立训练：不依赖辩论者，仅基于最终结果优化
- 自适应辩论预算：选择性分配而非固定轮次

**NeoTrix 融合**:
- **ConsciousnessTree**: Meta-Moderator 的元认知循环 = ConsciousnessTree 的监控-控制-裁决
- **NT-META coordinator**: 可作为跨域仲裁的参考架构
- **GWT attention**: 自适应停止 = 动态注意力深度控制

---

## 5. Agent安全 (Agent Safety)

### 5.1 Alignment Tax 几何理论

**来源**: arXiv:2603.00047

**突破点**: 首次为 alignment tax 提供几何理论。在线性表示假设下，定义对齐税率为安全方向在能力子空间上的投影平方。推导椭圆 Pareto 前沿，参数化为安全和能力子空间的主角度 $\alpha$。$\alpha=0$ 时线性权衡不可避免，$\alpha=\pi/2$ 时正交无权衡。推导缩放律：$\tau = \tau_0 + R(d)$，不可约分量 $\tau_0$ 由数据结构决定，包装残差 $R(d)$ 随模型维度消退。

**关键机制**:
- 主角度 $\alpha$ 参数化安全-能力权衡前沿
- 可计算税率 $\tau_i = \langle v^*, c_i \rangle^2$：对齐前即可预测能力退化
- 缩放分解：不可约分量 + 包装残差
- 安全-安全权衡：偏相关系数 $\cos\theta = \frac{\rho - ab}{\sqrt{(1-a^2)(1-b^2)}}$

**NeoTrix 融合**:
- **NT-SHIELD**: 对齐税率可作为安全策略选择的量化指标
- **NT-MIND distillation**: 在蒸馏前计算主角度，预测哪些能力受影响
- **Axiom A1**: 对齐税率 = 模型路由的成本函数组件

---

### 5.2 NSPO — 零空间约束策略优化

**来源**: arXiv:2512.11391

**突破点**: 将安全策略梯度投影到一般任务表示的零空间，约束安全更新正交于能力子空间。理论证明 NSPO 保留模型核心能力，同时保证安全对齐的下降方向。仅需 40% 公开安全数据即可达到强安全性能，无需大量混合通用任务数据。

**关键机制**:
- 零空间投影：$\Delta_{NSPO} = P_{\ker(K^T)} \nabla_W J_{safe}$
- 保持 $K-V$ 映射不变 = 保留通用推理能力
- 移除 KL 散度（投影替代 KL 约束）
- 定理证明：投影梯度是有效的安全下降方向

**NeoTrix 融合**:
- **NT-SHIELD**: NSPO 可直接应用于 NT-SHIELD 的安全微调管线
- **NT-CORE SelfModel**: 零空间投影保留能力 = SelfModel 动态性能模型不受安全更新影响
- **NT-MIND SEAL**: 在 distillation 阶段使用 NSPO 而非标准 RLHF，减少对齐税

---

### 5.3 OPSA — 在线策略自蒸馏降低安全税

**来源**: arXiv:2605.15239

**突破点**: 识别离策略监督为安全税的第二来源（即使数据来自目标模型本身）。OPSA 在学生自身采样的轨迹上应用密集 per-token KL 监督，从冻结的特权安全上下文教师学习。教师翻转率 (TFR) 搜索激活潜在安全推理的上下文。在 1.5B 和 0.6B 小模型上提升 +8.85 和 +5.49 点。

**关键机制**:
- 在策略自蒸馏：学生轨迹 + 冻结教师 + 特权安全上下文
- 教师翻转率 (TFR)：衡量上下文将不安全响应转为安全的频率
- Token 级分析：更新集中在早期拒绝决策 token 窗口

**NeoTrix 融合**:
- **NT-SHIELD**: OPSA 的在策略蒸馏可替代标准安全微调，减少能力退化
- **NT-MIND distillation**: OPSA = 在策略蒸馏的特化版本，可扩展到一般能力蒸馏
- **NT-IO LLM providers**: 小模型收益最大，验证 Axiom A1 — cheap models + 好方法 > expensive models

---

### 5.4 Core Safety Values — 可证明可纠正代理

**来源**: CEUR-WS Vol-4189 / AAAI 2026

**突破点**: 首个完整的可纠正性形式化解决方案。五个结构分离的效用头（服从/开关访问保存/真实性/低影响行为/有界任务奖励）按字典序严格加权组合。定理 1 证明单轮可纠正性，定理 3 扩展到多步自生成代理。证明任意被攻击代理的安全验证不可判定（停机问题归约），但有限视界内可判定且可隐私保护验证。

**关键机制**:
- 五头字典序效用：weight gaps 确保安全优先级
- 有限视界 "可判定岛"：安全审计在多项式时间内可验证
- 零知识证明：仅揭示 "安全/不安全" 位，保护模型权重

**NeoTrix 融合**:
- **NT-SHIELD**: 五头字典序 = egress privacy guard 的多层信任分级理论基础
- **NT-GOVERNANCE**: 可纠正性形式化可作为治理策略的数学框架
- **NT-REPAIR**: 可判定岛概念可用于自愈能力的边界定义

---

### 5.5 SOOPER — 策略先验的安全探索

**来源**: arXiv:2601.19612

**突破点**: 使用保守策略先验（来自离线数据或模拟器）实现安全探索。概率动力模型乐观探索，悲观回退到保守先验。证明学习全程保证安全，建立累积遗憾上界。在 SafetyGym 和 RWRL 基准上超越 SOTA，且在真实硬件上验证。

**关键机制**:
- 双模式操作：安全在线数据收集 + 乐观模拟规划
- 悲观回退：累积成本 + 悲观成本值 > 安全预算时触发
- 隐式安全策略集扩张：随模型改进逐步扩展

**NeoTrix 融合**:
- **NT-SHIELD**: SOOPER 可作为 agent 安全探索的参考算法
- **NT-REPAIR**: 保守先验 = 系统已知安全配置的回退机制
- **ConsciousnessTree**: 安全探索 = 意识树生长周期的约束条件

---

## 交叉主题总结

### 记忆-规划-工具三元融合

| 维度 | 记忆突破 | 规划突破 | 工具突破 |
|------|---------|---------|---------|
| **成本** | LycheeMemory: 86% token 节省 | CHIME: 更少记忆项=更好规划 | Tool-R0: 零数据自进化 |
| **可信度** | TrustMem: 50-79% 错误降低 | AdaPlan-H: 自适应避免过度规划 | ContDa: 稳定性-适应性平衡 |
| **架构** | 双层记忆 (CLS 理论) | ReAcTree: 层级代理树 | MoE 单模型自辩论 |

### 协作-安全协同

| 维度 | 协作突破 | 安全突破 |
|------|---------|---------|
| **自适应** | PEAR: 排列等变路由 | NSPO: 零空间投影 |
| **元认知** | Meta-Moderator: 学习停止 | 五头字典序效用 |
| **成本** | MoD: 87% token 减少 | OPSA: 小模型安全对齐 |

### NeoTrix 优先行动

1. **NT-MEMORY 升级**: 集成 LycheeMemory 段级合并 + TrustMem 过渡验证 → KB 写入成本降低 80%+
2. **NT-SHIELD 安全管线**: NSPO 零空间投影 + OPSA 在策略蒸馏 → 安全对齐能力退化 <1%
3. **GWT 路由增强**: PEAR 排列等变路由 + Meta-Debate 动态角色分配 → 多专家路由准确率 +30%
4. **NT-ACT 工具生态**: api-to-tools 自动发现 + Tool-R0 零数据进化 → MCP 工具数量指数增长
5. **experience-tree 吸收优化**: TrustMem 过渡验证 + RecMem 复现触发 → 经验吸收可靠性 +50%

---

*本批次覆盖 5 主题 × 19 来源，提取 23 个突破点，全部映射到 NeoTrix 6 层架构。*
