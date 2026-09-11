# 第84批破限制技术 - 研究来源汇编

## 主题1: 稀疏激活混合专家

| # | 来源标题 | URL | 关键点 |
|---|---------|-----|--------|
| 1 | Mixture-of-Experts with Expert Choice Routing | https://arxiv.org/abs/2202.09368 | 专家选择路由机制，允许每个专家处理最相关的令牌 |
| 2 | Mixture-of-Experts with Expert Choice Routing (Google Research) | https://research.google/blog/mixture-of-experts-with-expert-choice-routing/ | Google对MoE路由的研究，讨论条件计算和专家选择 |
| 3 | Mixture of Experts Explained | https://huggingface.co/blog/moe | MoE架构的全面解释，包括路由机制和训练策略 |
| 4 | What Is Mixture of Experts (MoE) and How It Works? | https://www.nvidia.com/en-us/glossary/mixture-of-experts/ | NVIDIA对MoE的概述，强调计算效率 |
| 5 | MSN: A Memory-based Sparse Activation Scaling Framework | https://arxiv.org/abs/2602.07526 | 基于记忆的稀疏激活扩展，用于推荐模型 |

## 主题2: 注意力机制

| # | 来源标题 | URL | 关键点 |
|---|---------|-----|--------|
| 1 | Learning Advanced Self-Attention for Linear Transformers | https://arxiv.org/abs/2505.08516 | 自注意力作为图的归一化邻接矩阵，提升线性Transformer |
| 2 | Linear Attention Variants Overview | https://www.emergentmind.com/topics/linear-attention-variants | 线性注意力变体概述，包括核近似和可扩展长程依赖 |
| 3 | A Visual Guide to Attention Variants in Modern LLMs | https://magazine.sebastianraschka.com/p/visual-attention-variants | 现代LLM中注意力变体的视觉指南，比较不同机制 |
| 4 | Beyond Softmax — Linear Attention | https://www.youtube.com/watch?v=pUCWwGR5WmQ | 线性注意力作为softmax注意力的替代方案，用于序列建模 |
| 5 | Scaling State-Space Models on Multiple GPUs with Tensor | https://arxiv.org/abs/2602.21144 | 选择性状态空间模型在多GPU上的扩展，作为LLM主干 |

## 主题3: 位置编码

| # | 来源标题 | URL | 关键点 |
|---|---------|-----|--------|
| 1 | Beyond Attention: How Advanced Positional Embedding Methods Improve upon the Original Transformers | https://medium.com/data-science/beyond-attention-how-advanced-positional-embedding-methods-improve-upon-the-original-transformers-90380b74d324 | 高级位置嵌入方法如何改进原始Transformer |
| 2 | Positional Encoding — Intuitively and Exhaustively Explained | https://iaee.substack.com/p/positional-encoding-intuitively-and | 直观且详尽的位置编码解释 |
| 3 | Positional Encoding in Transformer-Based Time Series | https://arxiv.org/html/2502.12370v2 | 基于Transformer的时间序列中的位置编码，性能提升但计算成本增加 |
| 4 | Advanced Positional Encoding Methods | https://apxml.com/courses/how-to-build-a-large-language-model/chapter-13-positional-encoding-variants | 高级位置编码方法，包括相对和旋转嵌入 |

## 主题4: 归一化技术

| # | 来源标题 | URL | 关键点 |
|---|---------|-----|--------|
| 1 | Enhancing deep neural network training through learnable normalization | https://www.sciencedirect.com/science/article/pii/S0950705125010135 | 通过可学习归一化增强深度神经网络训练 |
| 2 | Peri-LN: Revisiting Layer Normalization in the Transformer | https://arxiv.org/html/2502.02732v1 | 重新审视Transformer中的层归一化，Peri-LN平衡方差增长 |
| 3 | Normalization Layer Placement (Pre-LN vs Post-LN) | https://apxml.com/courses/how-to-build-a-large-language-model/chapter-11-scaling-transformers-architectural-choices/normalization-layer-placement | Pre-LN与Post-LN的比较，确保激活尺度一致 |
| 4 | Pre-Layer Normalization in Neural Networks | https://www.emergentmind.com/topics/pre-layer-normalization | 预层归一化技术，将层归一化应用于每个子层的输入 |
| 5 | Batch Normalization & Layer Normalization: The Secret to Training Deep Neural Networks | https://blog.stackademic.com/batch-normalization-layer-normalization-the-secret-to-training-deep-neural-networks-9d68ec1fe48b | 批归一化和层归一化的秘密，训练深度神经网络 |

## 主题5: 损失函数

| # | 来源标题 | URL | 关键点 |
|---|---------|-----|--------|
| 1 | Loss Functions in Deep Learning: A Comprehensive Review | https://arxiv.org/html/2504.04242v1 | 损失函数的全面综述，按任务分类 |
| 2 | What is Loss Function? | https://www.ibm.com/think/topics/loss-function | IBM对损失函数的定义，衡量模型性能 |
| 3 | Near Infinite Batch Size Scaling for Contrastive Loss | https://arxiv.org/abs/2410.17243 | 接近无限批量大小的对比损失缩放，提升表征学习 |
| 4 | Full Guide to Contrastive Learning | https://encord.com/blog/guide-to-contrastive-learning/ | 对比学习完整指南，包括损失计算和应用 |
| 5 | Contrastive Loss Explained | https://medium.com/data-science/contrastive-loss-explaned-159f2d4a87ec | 对比损失解释，计算正负样本距离 |

---
**收集日期**: 2026-09-11  
**总来源数**: 25 (5主题 × 5来源)  
**搜索关键词**: 主题特定关键词组合  
**输出格式**: 结构化Markdown表格，包含标题、URL和关键点