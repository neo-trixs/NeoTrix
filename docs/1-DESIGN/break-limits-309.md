# 第95批 破限制技术 — 5大主题

> 采集时间: 2026-09-11 | 每主题 3-5 来源

---

## 主题1: 模型蒸馏 (Knowledge Distillation)

### 核心发现

- **Token-Adaptive KD (AdaKD)**: 根据学生模型实时学习状态自适应蒸馏过程，动态聚焦高价值token，引入逆难度温度缩放 (AAAI-26, Xie et al.)
- **数据高效蒸馏框架 (DED)**: 仅需 0.8k 精选样本即可突破推理蒸馏 scaling law，平衡域内/域外性能 (arXiv:2508.09883, Wu et al.)
- **蒸馏 Scaling Law**: Apple 提出蒸馏 scaling law，估算学生模型性能基于计算预算在教师/学生间的分配 (ICML 2025, Busbridge et al.)
- **Agent Distillation**: 将LLM Agent的完整任务解决行为蒸馏到小模型，包含推理能力和工具使用 (NeurIPS 2025)
- **序列截断蒸馏**: 仅训练前50% token即可保留~91%性能，训练时间/内存/FLOPs降低50% (ACL Findings 2026, Chen et al.)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | AAAI-26 | LLM-Oriented Token-Adaptive Knowledge Distillation (AdaKD) | 2026-03 |
| 2 | arXiv:2508.09883 | Beyond Scaling Law: A Data-Efficient Distillation Framework for Reasoning | 2025-08 |
| 3 | ICML 2025 | Distillation Scaling Laws | 2025 |
| 4 | NeurIPS 2025 | Distilling LLM Agent into Small Models with Retrieval and Code Tools | 2025 |
| 5 | ACL Findings 2026 | Distilling the Essence: Efficient Reasoning Distillation via Sequence Truncation | 2026-07 |

---

## 主题2: 提示工程 (Prompt Engineering)

### 核心发现

- **41+ 技术分类**: 系统综述将提示工程技术按应用域分为6大类，涵盖 BoT/CD-CoT/R-CoT/CoD 等新兴方法 (arXiv:2402.07927, Sahoo et al.)
- **Tree of Thoughts (ToT)**: 将LLM推理从线性链扩展为树状搜索，支持多路径探索和回溯 (Yao et al.)
- **Graph of Thoughts (GoT)**: 图结构提示框架，匹配人类非线性思维特征 (Yao et al. 2023)
- **ART (Automatic Multi-step Reasoning and Tool-use)**: 自动CoT + 外部工具集成，处理需要推理和工具交互的复杂任务
- **Meta Prompting**: 结构和语法优先于内容，生成二级更精确提示 (Cisco/Outshift)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2402.07927 | A Systematic Survey of Prompt Engineering in LLMs: Techniques and Applications | 2025-03 |
| 2 | IBM | The 2026 Guide to Prompt Engineering | 2026 |
| 3 | arXiv:2401.14423 | Prompt Design and Engineering: Introduction and Advanced Methods | 2024-05 |
| 4 | HuggingFace Blog | Advanced Prompt Engineering: Theory, Practice, and Implementation | 2025-11 |
| 5 | Cisco/Outshift | 6 Advanced AI Prompt Engineering Techniques for Better Outputs | 2025-12 |

---

## 主题3: 上下文学习 (In-Context Learning)

### 核心发现

- **ICL Scaling Law**: ICL性能遵循与模型深度L、宽度d、上下文长度k、训练数据D的幂律关系，存在尖锐相变 (arXiv:2511.06232, Mehta & Gupta)
- **Many-Shot ICL Scaling**: 1000+ demonstrations 时性能持续单调增长，自生成标注可扩展到many-shot场景 (ACL Findings 2026, Gu et al.)
- **ICL Continual Learning**: 通过任务调度和提示重排，ICL可实现类似人类的持续学习，线性注意力模型(MAMBA/RWKV)展现类人保留模式 (arXiv:2509.22764)
- **跨架构ICL分析**: Transformer/State-Space/Hybrid架构在ICL内部机制不同，Function Vectors主要位于self-attention和Mamba层 (ACL Findings 2026, Wang et al.)
- **MachineLearningLM**: 通过持续预训练将1024-shot ICL能力注入7B模型，OOD表格分类超越GPT-5-mini ~15% (arXiv:2509.06806, Dong et al.)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2511.06232 | Scaling Laws and In-Context Learning: A Unified Theoretical Framework | 2025-11 |
| 2 | ACL Findings 2026 | Many-Shot Scaling of In-Context Learning with Self-Generated Demonstrations | 2026-07 |
| 3 | arXiv:2509.22764 | In-Context Learning can Perform Continual Learning Like Humans | 2025-09 |
| 4 | ACL Findings 2026 | Understanding In-Context Learning Beyond Transformers | 2026-07 |
| 5 | arXiv:2509.06806 | MachineLearningLM: Scaling Many-shot ICL via Continued Pretraining | 2025-09 |

---

## 主题4: 指令调优 (Instruction Tuning)

### 核心发现

- **Dual-Track Scaling**: 模型规模和任务规模双重缩放是指令跟随成功的核心因素，任务缩放比实例缩放更关键 (Computational Linguistics, Lou et al.)
- **指令跟随 Pruning**: 动态结构化剪枝，剪枝掩码根据用户指令输入动态选择最相关参数，3B激活模型 rival 9B dense (ICML 2025, Hou et al.)
- **ROSE**: 奖励导向数据选择，仅选5%训练数据即可达到全量微调竞争力 (EMNLP Findings 2025, Wu et al.)
- **BIDS**: 影响力归一化+迭代选择平衡多样能力学习，解决数据选择偏向高影响力任务问题 (EMNLP Findings 2025, Dai et al.)
- **Alignment-Centric Survey**: 全面综述指令调优数据收集(专家标注/蒸馏/自改进)、微调策略(SFT/LoRA/Prefix)、评估协议 (arXiv:2508.17184, Han et al.)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | Computational Linguistics 50(3) | Large Language Model Instruction Following: A Survey of Progresses and Challenges | 2024 |
| 2 | ICML 2025 | Instruction-Following Pruning for Large Language Models | 2025-01 |
| 3 | EMNLP Findings 2025 | ROSE: A Reward-Oriented Data Selection Framework for Task-Specific IT | 2025-11 |
| 4 | EMNLP Findings 2025 | BIDS: Balanced and Influential Data Selection for Diverse Capabilities | 2025-11 |
| 5 | arXiv:2508.17184 | Towards Alignment-Centric Paradigm: A Survey of Instruction Tuning in LLMs | 2025-08 |

---

## 主题5: 对齐技术 (Alignment Techniques)

### 核心发现

- **Constitutional AI (CAI)**: 通过AI自我批判-修正+RLAIF，用原则替代人类反馈，Pareto改进helpfulness-harmlessness (Anthropic, 2022)
- **Defense-in-Depth 分析**: 7种对齐技术 × 7种失败模式的相关性分析，发现AI Debate + Red Teaming组合可防止几乎所有失败模式 (arXiv:2510.11235, Dung & Mai)
- **C3AI 框架**: 正面框架+行为导向原则比负面/特质导向原则更符合人类偏好；图选择方法改进安全对齐 (WWW 2025, Kyrychenko et al.)
- **小模型CAI崩溃**: Llama 3-8B上CAI改善安全性但降低helpfulness并导致模型崩溃，合成修订过滤可缓解 (Stanford CS224R, Zhang)
- **视觉语言模型对齐综述**: 85种对齐策略，分表征对齐(内部一致性)和行为对齐(输出安全性)两维度 (ScienceDirect 2026, Fang et al.)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | Anthropic/arXiv:2212.08073 | Constitutional AI: Harmlessness from AI Feedback | 2022-12 |
| 2 | arXiv:2510.11235 | AI Alignment Strategies from a Risk Perspective: Independent Safety Mechanisms or Shared Failures? | 2025-10 |
| 3 | WWW 2025/arXiv:2502.15861 | C3AI: Crafting and Evaluating Constitutions for Constitutional AI | 2025-02 |
| 4 | Stanford CS224R | Collapse-Resistant Constitutional AI for Small Language Models | 2025 |
| 5 | ScienceDirect | Alignment in Large Vision Language Models: A Survey | 2026 |

---

## 总结

| 主题 | 核心突破方向 | NeoTrix 映射 |
|------|------------|-------------|
| 模型蒸馏 | Token自适应蒸馏、数据高效蒸馏、序列截断、Agent行为蒸馏 | SEAL pipeline 知识压缩 |
| 提示工程 | ToT/GoT树图搜索、自动提示优化、元提示、工具集成 | GWT 注意力路由 |
| 上下文学习 | ICL scaling law、many-shot扩展、持续学习、跨架构ICL | SelfModel 上下文记忆 |
| 指令调优 | 双轨缩放、动态剪枝、数据选择、平衡能力学习 | NT-CORE 指令接口 |
| 对齐技术 | CAI原则化对齐、defense-in-depth、小模型鲁棒性、多模态对齐 | NT-SHIELD 安全治理 |
