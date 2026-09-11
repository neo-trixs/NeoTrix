# 第98批破限制技术

> 搜索时间: 2026-09-11 | 来源数: 50+ | 主题: 5

---

## 主题1: 持续预训练 (Continued Pretraining)

| 来源 | 标题 | 关键发现 |
|------|------|----------|
| arXiv:2402.17400 | Investigating Continual Pretraining in LLMs | 语义排序域间迁移效果最佳；持续预训练比FT在语义相似域间更优；遗忘是核心挑战 |
| arXiv:2603.27164 | daVinci-LLM: Towards the Science of Pretraining | Data Darwinism L0-L9框架；200+消融实验；处理深度是与规模并列的关键维度；自适应课程从基础→推理增强 |
| arXiv:2607.20548 | SOAP, Muon, and Beyond: Pushing LLM Pretraining Scales | Muon优化器在大batch下显著优于AdamW；二阶优化器可捕获损失地形几何；batch size可扩展至3072×8192 |
| Open Athena Blog | Improving LLM Pretraining Efficiency | Dense→MoE V1实现6.7×理论加速；256专家+partial key offset+MuonH组合达到7.8×；预注册损失预测精度0.8% |
| arXiv:2608.14071 | Scaling Domain Data Repetition in LLM Pretraining | 高质量域数据重复是有效策略但存在过拟合；重复次数与模型大小、TPP比协同；code/math/Wikipedia各有饱和动力学 |

---

## 主题2: 上下文蒸馏 (Context Distillation)

| 来源 | 标题 | 关键发现 |
|------|------|----------|
| arXiv:2602.12275 | On-Policy Context Distillation for Language Models (OPCD) | 在策略蒸馏：学生模型从自身生成轨迹学习；reverse KLD对齐context-conditioned教师；系统提示蒸馏节省推理开销 |
| arXiv:2605.28889 | Context Distillation as Latent Memory Management | 每个上下文蒸馏为独立LoRA适配器→模块化记忆库；Self-Gating决定是否激活；检索+路由+门控三阶段框架 |
| ICML 2026 | DiSC: Context Distillation Retains Post-Training Capabilities | Split Contexts方法：学生/教师条件化不同段→公共token最小化KL；新知识学习+旧能力遗忘的最佳权衡 |
| ACL 2026 | SADA: State-Aligned Distillation Adapters | 注意力块输出作为特征接口；状态对齐蒸馏强制适配器与全上下文oracle一致；内存占用显著低于ICL |
| arXiv:2606.24747 | Scaling Laws for Task-Specific LLM Distillation | 领域特异性压缩的缩放定律；CoT监督积极恢复剪枝擦除的通用知识；logit vs LoRA蒸馏对比 |

---

## 主题3: 稀疏微调 (Sparse Fine-Tuning)

| 来源 | 标题 | 关键发现 |
|------|------|----------|
| arXiv:2505.24037 | SEFT: Sparse Fine-Tuning via Sparsity Evolution | 动态演化稀疏拓扑：drop-and-grow周期更新非零索引；直接修改LLM参数而非LoRA；保留整体稀疏性 |
| arXiv:2607.09287 | Super-Tuning: From Activation-Aware Pruning to Sparse Fine-Tuning | Wanda风格激活-权重显著性分数选择固定稀疏支持；Supra混合适配器结合稀疏更新+LoRA；训练无需的支撑选择 |
| JMLR 328 | SPT: Sparsity-Aware Prompt Tuning | 为稀疏LLM定制的soft prompt补偿；渐进式剪枝+prompt长度与稀疏率成正比；80%稀疏LLaMA-13B提升7.88% |
| ICML 2025 | SparseLoRA: Accelerating LLM Fine-Tuning with Contextual Sparsity | SVD稀疏估计器动态选择权重子集；层/token/训练步三维度敏感性分析；计算成本降低2×，精度保持 |
| arXiv:2605.06402 | SparseForge: Semi-Structured LLM Sparsification via Annealing | Hessian感知重要性+渐进退火软掩码→硬件可执行结构稀疏；仅5B重训练token超越40B基线；2:4稀疏下57.27%零样本精度 |

---

## 主题4: 数据选择 (Data Selection)

| 来源 | 标题 | 关键发现 |
|------|------|----------|
| ACL 2026 | BLADE: Scalable Bi-level Adaptive Data Selection | Hessian-free框架；双层优化→惩罚单层目标；动态参考模型同步代理模型；随机块坐标Frank-Wolfe在线选择 |
| AAAI 2026 | Importance-Aware Data Selection for LLM Instruction Tuning | MIWV度量：有/无单样本示例的loss差；ICL发现高质量指令数据；1%数据即可显著提升 |
| arXiv:2605.30537 | Long-Term Effects of Data Selection in LLM Fine-Tuning | 短期最优选择可能导致长期rank reversal；myopic selection分析；LHAS目标增强覆盖+未来代理迁移+反集中 |
| ACL 2026 Findings | GALA: Geometric Data Selection with Strategic Prospecting | 潜在句嵌入聚类+几何距离锚点选择→去冗余；战略头脑风暴+反思生成高质量推理轨迹；动态验证模块 |
| arXiv:2601.23006 | InstructDiff: Domain-Adaptive Data Selection via Differential Entropy | 微分熵差异作为域自适应选择标准；推理任务偏好熵增(认知扩展)；通用任务偏好熵降(认知压缩)；10%数据超越全数据训练 |

---

## 主题5: 质量过滤 (Data Quality Filtering)

| 来源 | 标题 | 关键发现 |
|------|------|----------|
| arXiv:2510.00866 | The Data-Quality Illusion: Rethinking Classifier-based Quality Filtering | CQF隐式过滤HQ集本身；保留的是远离LQ集的数据而非类似HQ集；挑战CQF捕捉有意义质量概念的观点 |
| NAACL 2025 | FiNE: Filtering and Improving Noisy Data Elaborately | 三阶段：多因子过滤→参考质量增强→增广后过滤去冗余；解决复杂度偏向+知识不透明+多样性缺失 |
| NODALIDA 2025 | FinerWeb-10BT: LLM-Based Line-Level Filtering | GPT-4o mini行级标注→DeBERTa-v3分类器扩展；9类低质量标签；25%更少数据达到更优性能 |
| arXiv:2505.05427 | Ultra-FineWeb: Efficient Data Filtering and Verification | 快速验证策略评估数据对LLM训练的影响；优化正负样本选择；fastText轻量分类器；1T英文+120B中文tokens |
| NeurIPS 2023 | D4: Document De-Duplication and Diversification | 去重+语义多样化组合；预训练嵌入空间数据选择→20%效率提升+2%下游精度；智能重复优于随机重复 |

---

## 跨主题洞察

| 洞察 | 涉及主题 |
|------|----------|
| **域重要性随规模翻转** — 0.3B最优的域权重在1.2B可能失效 | 持续预训练 + 数据选择 |
| **稀疏化贯穿训练-推理** — 剪枝→稀疏微调→结构化推理全链路稀疏 | 稀疏微调 + 质量过滤 |
| **蒸馏可保留后训练能力** — DiSC证明split-context蒸馏兼顾新知识与旧能力 | 上下文蒸馏 + 持续预训练 |
| **质量 ≠ 相似度** — CQF实际过滤的是LQ数据而非选择HQ数据 | 质量过滤 + 数据选择 |
| **课程学习统一框架** — CGLS将渐进层扩展与数据课程统一为单策略 | 持续预训练 + 数据选择 |
