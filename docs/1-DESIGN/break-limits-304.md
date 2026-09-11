# 第90批破限制技术 — Break-Limits #304

> 采集日期: 2026-09-11 | 主题: 推荐系统 / 搜索引擎 / 知识蒸馏 / 模型合并 / 数据合成

---

## 主题1: 推荐系统 (Recommendation Systems)

### GenRec — Netflix LLM-backed推荐排序器
- **来源**: [arXiv:2608.10257](https://arxiv.org/abs/2608.10257) (2026-08-21)
- **摘要**: Netflix生产级LLM推荐排序器，基于内部基础LLM构建两阶段框架：Phase 1适配LLM理解目录与用户行为；Phase 2用推荐排序数据后训练，融合verbalized用户历史与上下文工程。A/B测试证实，用更少标注数据和输入信号即可在离线与在线指标上获得统计显著提升。核心范式转移：从特征工程到上下文工程，从定制架构到共享基础骨干。

### SEAR — 融合协同/语义/评分的LLM序列推荐
- **来源**: [WWW 2026](https://dl.acm.org/doi/10.1145/3774904.3792092) (2026-04-12)
- **摘要**: LLM驱动序列推荐框架，集成LLM嵌入提取模块融合协同信号、语义表征和评分信息三路特征。通过多粒度交互建模，在序列推荐任务上超越传统CF和基于内容的方法。

### RecBench+ — LLM个性化推荐助手基准
- **来源**: [WSDM 2026](https://arxiv.org/abs/2503.09382) (2026-02)
- **摘要**: 首个评估LLM作为个性化推荐助手能力的公开基准。涵盖硬条件+软偏好、多难度级别的查询。发现LLM具备初步推荐助手能力，擅长显式条件查询，但在需要推理或含误导信息的查询上仍有挑战。DeepSeek-R1在隐式条件查询上表现最佳。

### QueRec — 个性化查询驱动并行集成推荐
- **来源**: [EMNLP 2025](https://aclanthology.org/2025.findings-emnlp.446.pdf) (2025-11)
- **摘要**: 不依赖额外训练即可集成现有推荐系统。通过LLM生成用户偏好查询，交叉注意力机制用候选物品过滤生成的偏好，对比学习与偏好-物品匹配预训练目标对齐LLM与推荐器嵌入。性能提升达57%，同时改善推荐新颖性和多样性。

### LLM推荐系统综合综述
- **来源**: [IEEE Access 2025](https://ieeexplore.ieee.org/document/11129085) (2025-08-18)
- **摘要**: 全面分类LLM推荐系统为判别式、生成式、混合式、图增强和多模态五种范式。深入讨论幻觉、可扩展性、偏见和隐私等开放挑战，为LLM驱动推荐系统研究提供导航。

---

## 主题2: 搜索引擎 (Information Retrieval & Dense Retrieval)

### Scaling Laws for Dense Retrieval — 密集检索缩放定律
- **来源**: [SIGIR 2024](https://dl.acm.org/doi/10.1145/3626772.3657743)
- **摘要**: 首次系统研究密集检索模型的缩放定律。性能与模型大小、标注数量呈精确幂律关系，跨不同数据集和标注方法一致。缩放定律帮助优化训练过程，如解决预算约束下的资源分配问题。

### On the Scaling of Robustness and Effectiveness in Dense Retrieval
- **来源**: [SIGIR 2025](https://arxiv.org/abs/2505.24279) (2025-05-30)
- **摘要**: 研究密集检索鲁棒性与有效性的缩放定律。发现鲁棒性与有效性遵循不同缩放模式——要同时提升两者10%，需要GPT-4级别(175B)模型且10倍训练数据。为实际部署中平衡效率与鲁棒性提供指导。

### Scaling Laws for Embedding Dimension in IR
- **来源**: [arXiv:2602.05062](https://arxiv.org/abs/2602.05062) (2026-02-04)
- **摘要**: 系统分析嵌入维度与检索性能的关系。缩放行为符合幂律，可推导给定嵌入维度的性能缩放定律。对与训练任务对齐的评估任务，性能持续提升但收益递减；对未对齐任务，更大嵌入维度可能降低性能。

### Scaling Sparse and Dense Retrieval in Decoder-Only LLMs
- **来源**: [SIGIR 2025](https://arxiv.org/abs/2502.15526) (2025-02-21)
- **摘要**: 系统比较稀疏vs密集检索、CL vs KD vs组合在不同模型规模(1B/3B/8B)下的缩放行为。关键发现：缩放行为仅在CL训练下明显；稀疏检索在域内和域外均优于密集检索；CL+KD组合在8B规模达到SOTA。

### Negative Sampling Techniques in Information Retrieval — 负采样综述
- **来源**: [EACL 2026](https://arxiv.org/abs/2603.18005) (2026-01-09)
- **摘要**: 综合35篇核心论文，全面综述密集IR中的负采样技术。提出分类法涵盖随机、静态/动态挖掘和合成数据集三类方法。首次纳入LLM驱动的负采样方法，分析效果、计算成本和实现难度的权衡。

---

## 主题3: 知识蒸馏 (Knowledge Distillation)

### Scaling Laws for Task-Specific LLM Distillation
- **来源**: [arXiv:2606.24747](https://arxiv.org/abs/2606.24747) (2026-06-23, v2 2026-08-23)
- **摘要**: 推导领域特定LLM压缩的经验缩放定律，量化性能如何随数据集大小、压缩比、监督格式和迭代剪枝计划缩放。引入混合CoT监督损失稳定KL散度蒸馏。发现CoT监督能主动恢复剪枝擦除的通用知识。

### Distillation Scaling Laws — 蒸馏缩放定律
- **来源**: [arXiv:2502.08606](https://arxiv.org/abs/2502.08606) (2025-02-13, ICML 2025)
- **摘要**: 预测蒸馏学生模型性能的缩放定律，基于计算预算在教师与学生间的分配。关键发现：当教师已存在时蒸馏有优势；当需要训练教师时，监督学习更高效；大计算量下蒸馏与监督学习产生相同模型。

### SDPO — 自蒸馏策略优化 (Reinforcement Learning via Self-Distillation)
- **来源**: [arXiv:2601.20802](https://arxiv.org/abs/2601.20802) (2026-01)
- **摘要**: 单模型同时充当教师和学生的on-policy自蒸馏算法。当前策略在丰富环境反馈条件下生成self-teacher，通过logit级蒸馏损失匹配学生与self-teacher分布。超越GRPO达到48.8% vs 41.2%最终准确率，4倍更少生成即可达GRPO最终性能。

### OPSDL — On-Policy自蒸馏长上下文LLM
- **来源**: [arXiv:2604.17535](https://arxiv.org/abs/2604.17535) (2026-04-19)
- **摘要**: 利用模型自身强大的短上下文能力作为self-teacher监督长上下文生成。通过逐token反向KL散度提供密集监督信号，鼓励忠实使用相关证据并减少无关上下文引发的幻觉。在7B-32B模型上一致且大幅提升，不退化短上下文性能。

### LLM-Oriented Token-Adaptive Knowledge Distillation (AdaKD)
- **来源**: [AAAI 2026](https://ojs.aaai.org/index.php/AAAI/article/view/40701) (2026-03-14)
- **摘要**: 自适应蒸馏框架，根据每个token的实时学习状态调整蒸馏过程。Loss-driven Adaptive Token Focusing动态聚焦有价值token；Inverse Difficulty Temperature Scaling对困难token用低温精确纠错，对容易token用高温学习平滑分布。即插即用，跨方法和架构一致提升。

---

## 主题4: 模型合并 (Model Merging)

### Model Merging in Pre-training of LLMs — 预训练中的模型合并
- **来源**: [arXiv:2505.12082](https://arxiv.org/abs/2505.12082) (2025-05-17, v3 2025-05-22)
- **摘要**: 全面研究预训练过程中的模型合并技术。在从百万到1000亿+参数的Dense和MoE架构上，证明合并恒定学习率训练的checkpoint不仅显著提升性能，还能准确预测退火行为。为开源社区提供实用预训练合并指南。

### Model Merging in the Era of LLMs — FUSE分类法综述
- **来源**: [arXiv:2603.09938](https://arxiv.org/abs/2603.09938) (2026-03-10, v2 2026-03-30)
- **摘要**: 通过FUSE分类法(Foundation/Unified Strategies/Scenarios/Ecosystem)系统审视LLM时代模型合并。覆盖权重平均、任务向量算术、稀疏化增强、MoE架构和进化优化。提出跨架构合并理论基础方向：表征对齐或功能对应而非直接参数映射。

### In-the-Wild Model Merging for LLMs — 实际场景系统评估
- **来源**: [TMLR 2026](https://arxiv.org/abs/2511.21437) (2025-11-26, v2 2026-03-29)
- **摘要**: 大规模评估异构专家合并（可能训练于重叠或冲突目标）。评估6种SOTA方法、4个开源LLM、12个微调checkpoint、16个基准。关键发现：只有Task Arithmetic在"in-the-wild"设置下可靠提升性能；其他干扰感知和子空间方法通常无显著提升。

### ESM — Essential Subspace Merging (CVPR 2026)
- **来源**: [CVPR 2026](https://openaccess.thecvf.com/content/CVPR2026/html/Li_Model_Merging_in_the_Essential_Subspace_CVPR_2026_paper.html) (2026)
- **摘要**: 对参数更新诱导的特征偏移做PCA，主方向张成主导特征表征的本质子空间。每个任务的参数更新矩阵投影到各自本质子空间做低秩分解后合并。多级极化缩放策略放大关键知识参数、抑制冗余参数。在多任务模型合并上达到SOTA。

### TATR — Task Arithmetic in Trust Region (ICLR 2025)
- **来源**: [ICLR 2025](https://arxiv.org/abs/2501.15065) (2025-01-25)
- **摘要**: 形式化定义知识冲突——任务向量合并后某任务性能退化。冲突源于与任务特定损失梯度对齐的分量。TATR定义信任区域为参数空间中仅引起小变化的维度（梯度正交方向），限制合并在此区域内。作为即插即用模块兼容多种TA方法，8个数据集上可观提升。

---

## 主题5: 数据合成 (Synthetic Data Generation)

### Principled Synthetic Data — 首个推荐系统LLM缩放定律
- **来源**: [arXiv:2602.07298](https://arxiv.org/abs/2602.07298) (2026-02-07, v3 2026-06-01)
- **摘要**: 首次建立推荐系统LLM的缩放定律。两层合成数据：Layer 1通过item-text对齐和CF数据建立基础知识；Layer 2通过图随机游走生成位置去偏的用户交互历史。在0.6B-8B参数、163B token上建立幂律缩放。发现CF数据与UIH数据的不对称协同：CF数据将渐近UIH损失降低31%。

### Synthetic Data Generation Using LLMs — 文本与代码综述
- **来源**: [IEEE Access 2025](https://arxiv.org/abs/2503.14023) (2025-03-18, 2025-07-15发表)
- **摘要**: 全面综述LLM驱动合成数据生成。统一框架涵盖prompt-based、检索增强和迭代自精炼管线。覆盖低资源分类、QA、代码指令微调等任务。提出跨模态数据合成、自动prompt工程和鲁棒评估框架等开放方向。

### Scaling Low-Resource MT via Synthetic Data with LLMs
- **来源**: [EMNLP 2025](https://aclanthology.org/2025.emnlp-main.1408) (2025-11)
- **摘要**: 用LLM生成文档级合成语料，从英语Europarl扩展到147个低资源语言对。自动和人工评估证实高质量。引入SynOPUS公共合成并行数据集仓库。发现即使有噪声，LLM合成数据也能显著提升低资源MT性能。

### Active Synthetic Data Generation for Finetuning
- **来源**: [arXiv:2512.00884](https://arxiv.org/abs/2512.00884) (2025-11-30, v2 2026-02-09)
- **摘要**: 迭代闭环合成数据生成，由学生模型当前状态指导。对比静态生成，闭环策略在固定生成预算下提升学生性能。发现简单廉价的主动学习选择标准往往最有效。在4个数学/逻辑推理数据集和4个小语言模型上验证。

### Scaling Law-Guided Data Augmentation (SLGDA)
- **来源**: [ACM 2026](https://dl.acm.org/doi/10.1145/3787100) (2026-01-05)
- **摘要**: 首次提出评估LLM生成文本与人类自然语言一致性缩放定律的统一框架。SLGDA方法通过缩放定律对齐度排名和选择合成文本，比基线分类器提升约7%-10%准确率，超越近期方法1%-3%。链接缩放定律到语言冗余、经济性等更广泛视角。

---

## 跨主题洞察

| 维度 | 发现 |
|------|------|
| **缩放定律统一** | 推荐/检索/蒸馏/数据合成均出现幂律缩放定律，但各领域缩放指数差异显著（CF α≈0.35, UIH α≈0.59, dense retrieval鲁棒性需GPT-4级模型） |
| **合成数据闭环** | 推荐系统合成数据(CF+UIH)揭示不对称协同；检索用LLM标注替代点击信号；蒸馏的合成数据需CoT监督恢复通用知识 |
| **训练-free合并** | Task Arithmetic在in-the-wild设置下可靠，但异构/冲突专家合并仍是开放问题；本质子空间/信任区域方法缩小冲突 |
| **On-policy自蒸馏** | 单模型teacher-student双重角色成为趋势(SDPO/OPSD/OPSDL)，密集token级监督显著优于序列级奖励 |
| **范式融合** | 推荐系统从特征工程→上下文工程；检索从BM25→dense→稀疏+密集混合；模型合并从权重平均→子空间/激活感知 |
