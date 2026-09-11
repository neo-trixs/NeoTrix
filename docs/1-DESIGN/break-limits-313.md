# 第99批破限制技术 — 5主题×5来源

> 搜索时间: 2026-09-11 | 批次: #99

---

## 主题1: 模型融合 (Model Soups & Weight Averaging)

| # | 来源 | 标题 | 核心洞见 |
|---|------|------|----------|
| 1 | [arXiv 2602.09689](https://arxiv.org/abs/2602.09689) (2026) | MonoSoup: Model Soups Need Only One Ingredient | 单 checkpoint + SVD 分解高/低能量方向，熵权自动重加权，无需多模型训练即达 ID-OOD 平衡。CLIP/Qwen 实验证明可替代多 checkpoint 方法 |
| 2 | [arXiv 2511.13254](https://arxiv.org/pdf/2511.13254) (2025) | Souper-Model: How Simple Arithmetic Unlocks SOTA LLM Performance | SoCE 方法 (模型选择 + 权重优化) 在 70B tool-use 模型上达 80.68% BFCL SOTA，超最佳单模型 2.7%。反相关准则选候选 + 优化权重 |
| 3 | [ICML 2022](https://proceedings.mlr.press/v162/wortsman22a/wortsman22a.pdf) | Model Soups (Foundational) | Greedy Soup + Uniform Soup 基础框架。CLIP/ALIGN 微调后平均权重，零推理开销提升 OOD 鲁棒性 |
| 4 | [ICCS 2025](https://dl.acm.org/doi/10.1007/978-3-031-97635-3_19) | Heterogeneous Model Soup for LLM Alignment | 异构模型融合：不同超参/数据/方法训练的模型也能 soup，多维评估 LLM 对齐效果 |
| 5 | [arXiv 2602.03702](http://arxiv.org/html/2602.03702v1) (2026) | Anytime Pretraining: Horizon-Free LR Schedules with Weight Averaging | 权重平均 + 简单无 horizon 学习率 = 任意时长预训练方案，可替代 cosine decay，150M/300M LM 实验验证 |

**关键模式**: 模型融合从"多 checkpoint 均匀平均"进化到"单 checkpoint SVD 分解"和"异构融合权重优化"，核心趋势是降低融合成本、扩展适用场景。

---

## 主题2: 知识编辑 (Knowledge Editing for LLMs)

| # | 来源 | 标题 | 核心洞见 |
|---|------|------|----------|
| 1 | [ACL 2026](https://aclanthology.org/2026.findings-acl.1892/) | Orthogonal Representation Editing (ORE) | 正交表示解耦语义纠缠，批量编辑时防止知识干扰。提出正交投影解决 batch editing 退化问题 |
| 2 | [ICLR 2026](https://arxiv.org/pdf/2505.18774) | DiKE: Disentangling Knowledge Representations for LLM Editing | KRD 模块将 subject 表示解耦为目标相关/无关两部分，DKE 模块注入新知识同时保留无关知识，优于 MEMIT/AlphaEdit |
| 3 | [ICLR 2026](https://proceedings.iclr.cc/paper_files/paper/2026/hash/73cadd87a4070ad4d836e9cacac22670-Abstract-Conference.html) | NeuralDB: Scaling Knowledge Editing to 100K Facts | 将 L&E 建模为 KV 数据库查询，非线性门控检索模块。扩展到 100K facts (50× prior work)，6 项 NLU 任务保持原性能 |
| 4 | [ACL 2025](https://aclanthology.org/2025.acl-long.665) | ChainEdit: Propagating Ripple Effects via Logical Rule-Guided Chains | 知识图谱逻辑规则 + LLM 推理联动，自动提取逻辑模式生成/编辑逻辑关联知识簇，逻辑泛化提升 30%+ |
| 5 | [OpenReview ICLR 2026](http://openreview.net/forum?id=WvRmaSD2QV) | Model Editing is Over: Revealing Its Illusory Success | LTE 方法基于捷径而非完整语义，最简否定查询即崩溃。呼吁重新审视 locate-then-edit 基础 |

**关键模式**: 知识编辑从单点修改 → 批量正交解耦 → 百万级 KV 数据库扩展，同时面临根本性质疑 (捷径问题)。趋势是混合方法 (编辑 + 上下文推理)。

---

## 主题3: 工具使用 (Tool Use & Function Calling)

| # | 来源 | 标题 | 核心洞见 |
|---|------|------|----------|
| 1 | [ICLR 2026](https://arxiv.org/abs/2604.06185) | WildToolBench: Benchmarking LLM Tool-Use in the Wild | 57 个 LLM 评估，无模型超 15% session 准确率。真实用户行为 (组合任务/隐式意图/指令转换) 远比人工复杂任务更具挑战 |
| 2 | [arXiv 2604.00835](https://arxiv.org/html/2604.00835v2) (2026) | Agentic Tool Use in LLMs: A Survey | 综述四分支演化：prompt plug-and-play → 监督学习 → 奖励驱动策略 → 评估。涵盖 ToolACE 自进化框架、ToolRL 结果奖励优化 |
| 3 | [ACL 2026](https://aclanthology.org/2026.acl-long.1573) | ToolScope: Tool Merging & Context-Aware Filtering | 工具合并去重 + 上下文感知筛选，工具选择准确率提升 8.38%-38.6%。解决冗余工具集和上下文长度限制 |
| 4 | [ACL 2026](https://aclanthology.org/2026.acl-long.855) | ToolPRM: Fine-Grained Inference Scaling for Function Calling | 过程奖励模型逐 intra-call 步打分 (函数名 + 参数填充)。原则："explore more but retain less"，结构化输出早期 JSON 错误不可恢复 |
| 5 | [ACM Survey 2026](https://dl.acm.org/doi/10.1145/3788284) | Function Calling in LLMs: Industrial Practices, Challenges | 三阶段流水线 (pre-call/on-call/post-call)。模型 scaling 显示 4B→7B 功能调用能力显著跃升，LoRA 微调后呈 scaling law |

**关键模式**: 工具使用从"能调用"→"能编排多工具"→"能在真实噪声环境中鲁棒调用"。关键瓶颈是用户行为的 wild nature 而非任务复杂度。

---

## 主题4: 代码生成 (Code Generation & Program Synthesis)

| # | 来源 | 标题 | 核心洞见 |
|---|------|------|----------|
| 1 | [arXiv 2503.01245](https://arxiv.org/abs/2503.01245) (2025) | LLMs for Code Generation: Comprehensive Survey | 全景综述：HumanEval 已饱和，o1/o3 突破性进展。ClarifyGPT 澄清歧义需求、AceCoder 双阶段提升需求理解 |
| 2 | [PLDI 2026](https://arxiv.org/abs/2604.13290) | Presynthesis: Scaling Program Synthesis with Finer-Grained Abstract Semantics | 离线预合成构建树自动机 + oracle，使搜索时剪枝从 O(n) 降至 O(1)。SQL/字符串/矩阵三领域大幅超越 prior work |
| 3 | [arXiv 2605.31058](https://arxiv.org/abs/2605.31058) (2026) | Combinatorial Synthesis: Scaling Code RLVR via ADR | 原子分解+重组生成可验证代码任务，解决 RLVR 数据稀缺问题。新颖性/难度/多样性均超基线 |
| 4 | [NeSy 2025](https://arxiv.org/abs/2504.17017) | Neural Theorem Proving: Generating Structured Proofs | 2 阶段微调 (SFT→RL) 生成 Isabelle 证明。三组件框架：NL 陈述 → LLM 证明 → 启发式构建，验证 AWS S3 策略 |
| 5 | [NeurIPS 2024](https://neurips.cc/virtual/2024/98527) | Reasoning in Reasoning (RiR): Hierarchical Framework | 规划者-执行者博弈统一分解与搜索。miniF2F 上 3× 加速，信息论解释有效性 |

**关键模式**: 代码生成从"通过测试"→"理解需求/澄清歧义"→"形式化验证"。RLVR + 原子分解成为扩展训练数据的新范式。

---

## 主题5: 数学推理 (Mathematical Reasoning & Theorem Proving)

| # | 来源 | 标题 | 核心洞见 |
|---|------|------|----------|
| 1 | [ACM Survey 2026](https://dl.acm.org/doi/10.1145/3786333) | A Survey on LLMs for Mathematical Reasoning | 两阶段认知：理解 + 答案生成。从直接预测到 CoT 推理，涵盖 test-time scaling、过程奖励模型、Forest-of-Thought |
| 2 | [ACL 2025](https://arxiv.org/abs/2501.11110) | Chain-of-Reasoning (CoR): Unified Multi-Paradigm | NLR+AR+SR 三范式协同，CoR-Math-7B 在定理证明上超 GPT-4o 41%，MATH 算术超 RL 方法 15% |
| 3 | [EACL 2026](https://aclanthology.org/2026.findings-eacl.76.pdf) | SymCode: Neurosymbolic Mathematical Reasoning | LLM 生成 SymPy 可验证 Python 代码，训练无关。MATH-500/OlympiadBench 准确率提升 13.6%，token 减少 60-77% |
| 4 | [EMNLP 2025](https://arxiv.org/abs/2506.17104) | DREAM: FOL Theorem Proving for LLMs | 公理驱动策略多样化 + 子命题错误反馈。DeepSeek-Prover-V2-7B 在 FOL 任务仅 4.2%，DREAM 提升至 6.4% |
| 5 | [ICML 2026](https://arxiv.org/abs/2504.21801) | DeepSeek-Prover-V2: RL for Subgoal Decomposition | 递归分解 + 冷启动 RL，MiniF2F-test 88.9%，PutnamBench 49/658。AIME 15 题解 6/15，接近 V3 的 8/15 (majority voting) |

**关键模式**: 数学推理从"单范式 CoT"→"多范式协同 (NLR+AR+SR)"→"形式化验证闭环 (Lean4/Isabelle)"。神经定理证明与形式化系统的差距正在快速缩小。

---

## 跨主题洞见

| 模式 | 主题映射 | NeoTrix 启示 |
|------|----------|-------------|
| **从均匀到自适应** | 融合→权重优化、编辑→正交解耦、推理→多范式 | 动态路由优于固定策略 |
| **从单点到规模化** | 融合→单checkpoint、编辑→100K facts、工具→wild环境 | KB 吸收需支持批量+规模化 |
| **形式化验证闭环** | 代码→Lean4/Isabelle、数学→FOL定理证明 | SEAL pipeline 可引入形式化验证阶段 |
| **过程奖励 > 结果奖励** | 工具→ToolPRM、代码→过程监督、数学→ORM→PRM | GWT attention 可融入过程级信号 |
