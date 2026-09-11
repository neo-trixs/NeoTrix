# 第6批破限制技术 (Batch 220)

> 研究日期: 2026-09-11
> 主题: 注意力机制 / 微调 / 检索增强 / 代码生成 / 多模态

---

## 1. 注意力机制限制

### 1.1 Attention-Free Transformer: Grassmann Flows

**来源**: "Causal Grassmann Attention-Free Sequence Model" (arXiv:2512.19428)

**突破点**: 完全移除注意力矩阵, 用 Grassmann 流形上的几何演化替代。Token 对被编码为 Gr(2,r) 上的子空间, 通过 Plücker 坐标投影后门控融合。在固定 rank 下复杂度 O(Ld²) 而非 O(L²d)。

**关键发现**:
- 在 Wikitext-2 上 13-18M 参数的 Grassmann 模型, 验证困惑度仅比同规模 Transformer 高 10-15%
- SNLI 上基于 Grassmann 的分类头 (85.50%) 略优于 Transformer 分类头 (85.45%)
- 理论复杂度线性, 但当前实现因 Plücker 坐标计算开销在短序列上实际更慢

**NeoTrix 融合**: GWT 注意力路由可引入 Grassmann 流形作为替代路由机制。对于长上下文场景 (>1M tokens), Grassmann mixing layer 可作为 GWT 的线性复杂度后备路径, 在 KV cache 压力过大时降级使用。

---

### 1.2 Condensate Theorem: 已训练模型的注意力是天然稀疏的

**来源**: "Condensate Theorem" (arXiv:2602.06317)

**突破点**: 证明训练好的 Transformer 中 >95% 的注意力质量集中在 <1% 的位置上, 且这个子集可以用模型自己的 Q·Kᵀ 分数动态识别。移除低贡献位置产生 bit-exact 相同输出 (IEEE 754 float32 ULP 以下)。

**关键数据**:
- Topological Attention kernel: 131K tokens 下 159x 加速 (3.94ms vs 628ms Flash Attention)
- 投影 1M tokens: ~1,275x 加速, ~32ms vs ~40.2s
- KV cache 压缩: 每查询仅存储 ~97 个位置 (Anchor + Window + Top-k), 7B 模型在 1M tokens 下实现 ~10,000x 压缩
- 跨 12 架构验证 (GPT-2, Pythia, Qwen2, TinyLlama, Mistral): 1,500+ 生成 token 零分歧

**NeoTrix 融合**: 这是 NT-MEMORY KV cache 优化的核心理论基础。Condensate Set 的 Anchor+Window+Dynamic Top-k 结构可直接用于 `kv_cache_optimizer.rs`, 将 KV cache 从 "保留所有位置" 降级为 "仅保留 Condensate Set", 实现真正 bit-exact 无损压缩。

---

### 1.3 Zero-Sum Linear Attention (ZeroS)

**来源**: "Zero-Sum Linear Attention" (arXiv:2602.05230)

**突破点**: 移除 softmax 的常数零阶项 1/t, 创建零和权重允许正负值, 使单层注意力能执行对比操作。径向-角解耦: 对一阶和高阶 softmax 残差施加学习门控, 再引入有符号 cosθ 项恢复方向效应。

**关键数据**:
- MAD 基准: ZeroS-SM (75.4) 超过 Transformer (74.5), 在 In-Context 和 Noisy Recall 任务上显著优于 LinAttn
- 复杂度 O(Nd²) 时间, O(d²) 内存, 理论上比 softmax 更具表达力
- 关键洞察: 凸组合 (softmax) 只能混合信息, 不能做减法/对比; 零和权重可以

**NeoTrix 融合**: GWT 的 salience 计算可用 ZeroS 替代标准 softmax, 使注意力层天然支持 "抑制无关信号" (负权重), 而非仅 "增强相关信号" (正权重)。这对 E8 hexagram 推理中的干扰过滤特别有价值。

---

### 1.4 Softmax Linear Attention (SLA)

**来源**: "Softmax Linear Attention" (arXiv:2602.01744)

**突破点**: 将 softmax 操作从 token 级提升到 head 级, 利用注意力头作为粗语义槽进行竞争性门控选择, 恢复 "赢家通吃" 动态。线性复杂度下恢复精确聚焦能力。

**关键数据**:
- Softmax-GLA: 平均检索准确率从 19.17% 提升到 26.88% (+7.71%)
- S-NIAH-3 (UUID 检索): 标准线性模型在此最难设置上挣扎, SLA 显著提升鲁棒性
- 参数效率: Softmax GLA 170M 达到的困惑度 ≈ 标准 GLA 340M 的水平
- Head 数量越多, SLA 收益越大: 困惑度改善从 H=4 的 2.37 增长到 H=16 的 6.71

**NeoTrix 融合**: SLA 可作为 NT-CORE GWT 路由的 head-level 选择器。每个注意力头作为 "语义专家", 通过 head-level softmax 竞争决定哪些专家获得信息, 实现比 token-level 注意力更粗粒度但更高效的路由。

---

### 1.5 Attention 机制架构趋势: Hybrid 取代纯 SSM

**来源**: "Challenging Quadratic Attention" (ACL 2026 BigPicture)

**突破点**: 截至 2026 年中, 所有 top-10 LLM (LLMSys 排行榜) 仍基于全注意力机制。但混合架构 (SSM+Attention) 在边缘和中等规模部署中获得显著牵引力: MiMo-v2.5-pro, Gemma-4-31b, GLM-5.1, DeepSeek-V4 均采用混合架构。

**关键发现**:
- 纯 SSM 在理论上不比 Transformer 更具表达力 (在 SETH 下)
- 关键优势在边缘/延迟敏感场景: RWKV7, Griffin, Samba
- 细分记忆模型 (Titans, B'MOJO): 短期/长期/永久存储分层
- 架构多样性时代: Transformer 仍是默认, 但替代架构在特定场景站稳脚跟

**NeoTrix 融合**: NT-CORE 的意识架构应采用 hybrid 策略: GWT 全注意力用于核心推理路径, 线性注意力 (DeltaNet/GLA) 用于 I/O 层的高吞吐低延迟场景, SSM 用于边缘部署。

---

## 2. 微调限制

### 2.1 qa-FLoRA: 无数据自适应 LoRA 融合

**来源**: "qa-FLoRA: Data-free query-adaptive Fusion of LoRAs for LLMs" (AAAI 2026)

**突破点**: 无需复合训练数据或领域样本, 通过测量基础模型与各 adapter 之间的分布散度动态计算 layer-level 融合权重。每个查询获得个性化的 adapter 组合。

**关键数据**:
- 跨 9 个多语言复合任务 (数学/代码/医学): 比静态融合提升 ~5-6%, 比无训练基线提升 ~7-10%
- 接近有监督融合基线的性能, 完全无需训练
- 层级融合权重具有可解释的融合模式

**NeoTrix 融合**: NT-ACT 的 skill 节点可为每个域训练独立 LoRA, 运行时通过 qa-FLoRA 根据输入查询动态组合。这解决了 "一个模型服务多域" 的知识干扰问题。

---

### 2.2 LiST: 测试时 LoRA 本地单纯形融合

**来源**: "LiST: Local-Simplex Test-Time LoRA Fusion" (arXiv:2608.22370)

**突破点**: 将 LoRA bank 转换为目标条件化的本地单纯形, 在推理时搜索样本特定融合权重。分支保持融合 (branch-preserving fusion) 避免直接参数平均导致的低秩结构破坏。

**关键数据**:
- 多模态和语言基准上超越静态融合和传统 TTA 基线
- 安全接受规则: 仅在改进 prompt-level 能量且满足可行性约束时部署搜索权重
- 回退到目标条件先验: 当搜索不可靠时安全回退

**NeoTrix 融合**: NT-ACT 的 Tool Router 可用 LiST 思想: 对每个工具调用, 从 LoRA bank 中检索相关 adapter, 在推理时动态融合, 实现 zero-shot 泛化到未见过的任务组合。

---

### 2.3 Adapter Fusion MLP: 增量语言扩展

**来源**: "Adapter Fusion for Multilingual Text2Cypher" (arXiv:2601.16097)

**突破点**: 融合 MLP + 动态门控: 恢复 ~75% 联合多语言微调的性能增益, 仅用 20% 训练数据。增量扩展: 新增语言只需一个 LoRA adapter + MLP 轻量重训练。

**关键数据**:
- 融合 MLP: 平均 ROUGE-L 0.79 vs 均匀线性合并 0.75
- 推理速度无下降
- 从 24K→36K→48K 样本: 联合微调需完整重训练; 融合 MLP 仅需 adapter + MLP 更新

**NeoTrix 融合**: NT-IO 的多语言 LLM provider 可用此模式: 每种语言一个 LoRA, 融合 MLP 根据输入语言动态门控, 增量支持新语言而无需重训练。

---

### 2.4 Prompt Tuning 缩放定律

**来源**: "LLM Finetuning Scaling Laws" (arXiv:2402.17193)

**突破点**: 发现乘法联合缩放定律: 微调数据量与其他缩放因子的乘法关系。PET 参数缩放基本无效 (|α| ≪ 1e-2)。LLM 模型缩放对微调的影响大于预训练数据缩放。

**关键发现**:
- PET 参数缩放: LoRA rank 和 prompt 长度增加带来的收益边际递减
- 任务依赖性: <数千样本用 Prompt/LoRA, 数百万样本用全量微调
- Prompt Tuning 在少样本时优于 LoRA, LoRA 在多数据时更稳定

**NeoTrix 融合**: NT-MIND 的 SEAL pipeline 微调策略应据此选择: 快速原型用 prompt tuning (少量参数), 稳定部署用 LoRA (中等数据), 深度定制用 FMT (大量数据)。不应盲目增加 LoRA rank。

---

### 2.5 ULPT: 超低维 Prompt Tuning

**来源**: "Ultra-Low-Dimensional Prompt Tuning via Random Projection" (EACL 2026)

**突破点**: 将 prompt embedding 投影到 2D 超低维空间, 用冻结随机矩阵上投影, 仅学习 shift/scale 向量。保存 98% 参数的同时匹配全维度 prompt tuning 性能。

**关键数据**:
- 跨 20+ NLP 任务: 匹配或超越全参数 prompt tuning
- 比 LoRA 参数更少, 适合大规模定制 (per-user tuning)
- 理论保证: Johnson-Lindenstrauss 保持关系结构, 低秩随机投影保留注意力机制的点积信息

**NeoTrix 融合**: NT-IO 的 per-user 个性化适配可用 ULPT: 每用户仅存储 ~2D embedding + 一个随机种子, 极大降低多用户部署成本。

---

## 3. 检索增强限制

### 3.1 RAG 失败模式系统分类 (33种)

**来源**: "A Systematic Taxonomy of Failure Modes in RAG Systems" (TrustNLP 2026)

**突破点**: 覆盖 7 个流水线阶段的 33 种失败模式, 含 8 种 agentic RAG 失败模式 (全部无同行评审实证)。关键洞察: 级联盲区 — 评估聚焦单阶段指标时系统性低估端到端失败率。

**关键发现**:
- 检索和生成失败最受研究, 表征、评估和 agentic 编排失败被严重低估
- 12 种失败模式 (36%) 缺乏同行评审实证, 全部 8 种 agentic 模式在其中
- Agentic RAG 是增长最快的部署模式, 但获得最少的科学审查

**NeoTrix 融合**: NT-MEMORY 的 KB 检索应实施此分类的监控: 对每个检索调用标注失败模式类别, 特别是 F5-F6 (表征失败) 和 F26-F33 (agentic 失败)。构建级联感知的评估框架。

---

### 3.2 RAG 失败 14 模式三轴分类

**来源**: "A Taxonomy of Failure Modes in RAG Systems" (arXiv:2604.02033)

**突破点**: 三轴分类 (检索/融合/生成), 14 模式互斥。关键发现: 随语料库质量上升, 瓶颈从检索转移到融合 — 高质量语料库中 47% 错误来自融合侧。纯幻觉稳定在 9-12% 不随语料库质量改善。

**关键数据**:
- 低语料库: 检索 52%, 融合 18%, 生成 30%
- 高语料库: 检索 32%, 融合 47%, 生成 21%
- G1 (纯幻觉): 在所有语料库质量下稳定在 9.1-11.2%, 是模型级而非系统级问题

**NeoTrix 融合**: NT-MEMORY 的检索策略应根据语料库质量自适应: 低质量时投资检索改进 (混合搜索+重排序), 高质量时投资融合改进 (上下文管理+多样性惩罚)。

---

### 3.3 Self-Correcting RAG (SCIM)

**来源**: "SCIM: Self-Correcting Iterative Mechanism for RAG" (Electronics 2026)

**突破点**: 多维质量评估 + 自适应检索 (augment/refine 模式)。仅用 Flan-T5-base (250M 参数) 无微调, 达到 17.2% 质量提升。35% 查询在 1-2 轮迭代内收敛。

**关键数据**:
- 质量 0.627 vs 标准 RAG 0.535 (+17.2%, p < 0.001, Cohen's d = 0.84)
- 高质量答案率: 22% → 38.5% (+16.5pp)
- 比 ITER-RETGEN 少 31% 文档检索, 快 22%
- Spearman ρ = 0.842 人类相关性

**NeoTrix 融合**: NT-MEMORY 的 KB 检索管线应集成 SCIM 的多维质量评估: 对每个检索结果评估 completeness/consistency/factualness, 根据评分触发 augment (补全缺失) 或 refine (验证矛盾) 模式。

---

### 3.4 Corrective RAG (CRAG)

**来源**: "Corrective Retrieval Augmented Generation" (arXiv:2401.15884)

**突破点**: 轻量检索评估器 → 三类置信度 (Correct/Incorrect/Ambiguous) → 差异化策略。错误时回退到大规模 web 搜索, 模糊时组合两者。即插即用, 可耦合任意 RAG 方法。

**关键数据**:
- 基于 SelfRAG-LLaMA2-7b: PopQA +20.0%, Biography +36.9%, Arc-Challenge +4.0%
- 自校正机制贡献 > 仅 web 搜索补充: 证明核心价值在校正而非额外信息

**NeoTrix 融合**: NT-MEMORY 的检索策略应实现 CRAG 模式: 检索评估器判断 KB 质量 → Correct 时精选知识条 → Incorrect 时回退到 NT-WORLD 的 web 搜索 → Ambiguous 时组合两者。

---

### 3.5 SC-RAG: 证据感知自纠正链式思维

**来源**: "Towards evidence-aware RAG via self-corrective chain-of-thought" (ScienceDirect 2025)

**突破点**: 语义检索器 + 无监督 aspect 检索器提取 token 级证据, 证据感知 CoT 解决 LLM 内部知识与外部检索知识的冲突。100% 推理效率: 减少 14.3% 推理时间。

**关键数据**:
- LaMP + HotpotQA: 超越 SOTA 1.0-30.3%
- 一步推理 vs 多步 CoT: 更高效且保持性能
- 内部-外部知识冲突解决: 显式验证或反驳初步结论

**NeoTrix 融合**: NT-MEMORY 的 KB embedding 检索应实现 aspect-based 细粒度证据提取, 用于 NT-CORE 推理时的知识冲突检测和解决。

---

## 4. 代码生成限制

### 4.1 Aria: 通用代码代理 + 验证 harness 实现全自动形式化验证

**来源**: "Harnessing Code Agents for Automatic Software Verification" (arXiv:2607.06341)

**突破点**: 将整个引理交给通用 LLM 代码代理 (Claude Code), 自由选择证明策略, 由验证 harness 保持正确性/完备性/终止性。零人工干预, 零失败。

**关键数据**:
- Iris 核心模块: 全部 4,257 引理证明, 零失败
- RustBelt: 全部 217 引理 (Arc/Mutex/RwLock/RefCell)
- reglang: 全部 318 引理 (prior LLM prover 仅证明 1/8)
- iris-lean: 全部 72 未移植引理, 跨证明器泛化
- 证明: 75.5% 首次尝试, 平均 0.38 次重试

**关键洞察**: 证明的正确性由内核保证 (LLM 错误不会通过), 将证明构造转化为对 ground truth 的引导搜索。

**NeoTrix 融合**: NT-ACT 的代码生成管线应集成验证 harness: 生成代码 → 形式化验证器检查 → 反馈修正循环。这将代码生成从 "测试通过" 提升到 "形式化证明正确"。

---

### 4.2 AutoRocq: LLM 代理自动验证

**来源**: "AutoRocq: Automatic Verification with LLM Agents" (arXiv:2511.17330)

**突破点**: LLM 代理 on-the-fly 学习, 通过与 Rocq (Coq) 交互的迭代改进循环构建证明。自主决定何时查询证明器数据库、何时生成策略。

**关键数据**:
- 数学引理: 48.0% (超越基线 15.2-172.8%)
- 验证引例: 30.9% (超越基线 10.6-204.6%)
- Linux 内核模块: 自动验证 12 个函数正确性引例 (基线仅 2-10)
- 168 个引例由 AutoRocq 独特证明

**NeoTrix 融合**: NT-ACT 的工具链可集成 AutoRocq 式的代理验证: 对生成的 Rust 代码, 代理自主与形式化验证器交互, 构建证明而非仅运行测试。

---

### 4.3 Gödel-Code-Prover: 分层证明搜索

**来源**: "Gödel-Code-Prover: Hierarchical Proof Search" (arXiv:2603.19329)

**突破点**: 将复杂验证目标递归分解为结构更简单的子目标, 再用策略级证明解决。单一统一策略同时处理分解和完成。分解评分 = 构造性证明 + 结构简单性。

**关键数据**:
- 427 个 Lean 4 任务: 62.0% 成功率 (2.6x 改进)
- 平均 8-17 个辅助引例, 130-167 行证明代码
- 推理时缩放: 成功率随搜索迭代单调递增
- 最复杂证明: 680 行证明代码

**NeoTrix 融合**: NT-ACT 的复杂任务分解可借鉴分层证明搜索: 将大型验证目标递归分解, 每层用评分函数选择最佳分解, 实现比 flat 生成更强的推理。

---

### 4.4 Neuro-Formal Verification (NFV)

**来源**: "Neuro-Formal Verification: Agentic Language-Agnostic Formal Program Reasoning" (arXiv:2608.21516)

**突破点**: AI 代码代理翻译 → 已建立验证器决定 → 主流语言问题获得机器检查证明。跨语言 (Python→Dafny/Lean), 92% 精度, 57% 覆盖率。

**关键数据**:
- 206 个 (程序, 规范) 对: 57% 获得证明 (Dafny 验证 70% 正确, CBMC 反例 63% 错误)
- LLM-as-judge 基线: 72% 精度 (58/206 错误) vs NFV 92% 精度
- 无法证明时安全中止 (不胡乱证明)

**NeoTrix 融合**: NT-ACT 的代码生成应集成 NFV 流水线: Python 代码 → 翻译到验证感知语言 → 形式化验证 → 返回证明或反例。比纯测试更可靠的正确性保证。

---

## 5. 多模态限制

### 5.1 VLM 100B 数据缩放

**来源**: "Scaling Pre-training to 100B Data for VLMs" (CVPR 2026)

**突破点**: 100B image-text 对预训练。传统西方中心基准 (ImageNet/COCO) 收益饱和, 但文化多样性任务和低资源语言持续提升。质量过滤 (如 CLIP) 可能损害数据多样性。

**关键发现**:
- SigLIP-H/14 在 10B→100B: ImageNet/COCO p=0.9 (不显著)
- Dollar Street 地理定位: 41.7% (100B 1 epoch) vs 35.9% (10B 10 epochs), 相同计算量
- 低资源语言: 收益随模型增大而扩大
- 质量过滤的悖论: 提升西方中心任务但损害文化多样性

**NeoTrix 融合**: NT-WORLD 的多模态预训练应保留长尾数据而非过度过滤。对于 NeoTrix 的 "AI-native" 定位, 文化多样性比西方基准性能更重要。

---

### 5.2 Kimi K3: 2.8T MoE 原生多模态

**来源**: "Kimi K3: Open Frontier Intelligence" (arXiv:2607.24653)

**突破点**: 2.8T 总参数, 104B 激活参数, 1M token 上下文。Kimi Delta Attention (KDA) + Attention Residuals + Stable LatentMoE (896 专家, 激活 16/每 token)。2.5x 缩放效率提升。

**关键架构创新**:
- Hybrid Attention: 3 KDA + 1 Gated MLA 每块
- Attention Residuals: 每层可选择性检索所有前序层表示
- MoonViT-V2: 27 层, ~0.4B 参数, 处理图像和视频
- Pixel-shuffle 2x2 下采样: 视觉 token 压缩 4x

**NeoTrix 融合**: NT-IO 的 LLM provider 应支持 Kimi K3 级别的 1M 上下文窗口。KDA 的 "3 linear + 1 global" 混合模式可作为 NT-CORE GWT 的参考架构。

---

### 5.3 InternVL-X: 视觉 Token 高效压缩

**来源**: "InternVL-X: Advancing with Efficient Visual Token Compression" (CVPR 2026)

**突破点**: 三种压缩方法组合 — PVTC (投影器级, 网格交叉注意力), LVTC (层级, 压缩→解压缩), RVTC (分辨率级, 像素匹配切片)。用 ≤18% 视觉 token 达到 SOTA。

**关键数据**:
- InternVL2-8B: 平均指标 +2.81%, 训练效率 +43%
- 仅 18% token 超越 LLaVA-NeXT 平均性能 +2.65%
- 前填充延迟 -11%, FLOPs -65.7%
- LVTC 关键洞察: 深层视觉 token 贡献最大, 压缩→解压缩优于仅压缩

**NeoTrix 融合**: NT-WORLD 的视觉处理应实现压缩→解压缩范式: 浅层压缩 token 减少计算, 深层解压缩恢复细节。这对视频处理尤其重要。

---

### 5.4 STAR-Pro: 渐进式视觉 Token 裁剪

**来源**: "STAR-Pro: Stage-Wise Token Adaptive Reduction" (arXiv:2609.05916)

**突破点**: 训练-free 两阶段框架: 自适应阶段 (pivoted QR 构建特征覆盖候选池) + 渐进阶段 (跨解码器层演化 text-to-visual attention 裁剪嵌套幸存集)。

**关键数据**:
- LLaVA-Video-7B: 减少 90.5% 视觉 token, 保留 92.7% 性能, 2.24x 推理加速
- 跨 7 个 LVLM 架构和 18 个基准验证
- 关键发现: 视觉 token 的重要性随深度显著变化, 一次性裁剪决策不可靠

**NeoTrix 融合**: NT-WORLD 的视觉 token 处理应采用渐进式裁剪: 初始保留宽覆盖, 随解码深度动态裁剪, 而非固定预算裁剪。

---

### 5.5 大型音频语言模型 (LALM)

**来源**: "A Survey of Large Audio Language Models" (arXiv:2605.20266)

**突破点**: 统一端到端框架处理语音/音乐/环境声。关键挑战: 离散 token 压缩可能丢弃声学安全线索, 连续流形保留但增加对抗攻击面。全双工交互 (非轮询式) 成为下一个前沿。

**架构趋势**:
- 推理 token + 重建 token 分离 (ReasoningCodec): 理解用紧凑特征, 生成用可重建特征
- 功能层专业化: 下层感知抽象, 中层跨模态对齐, 上层声学生成
- 100B text + 60B audio token 训练 (UniAudio 2.0)

**NeoTrix 融合**: NT-IO 的音频处理应实现 ReasoningCodec 模式: 理解用推理 token (语义紧凑), 生成用重建 token (声学细节)。全双工交互可作为 NT-ACT agent 的实时语音接口。

---

### 5.6 长视频理解: 代理式导航

**来源**: 多源综合 — Hour-LLaVA (NeurIPS 2025), LongVideo-R1 (arXiv:2602.20913), MemDreamer (arXiv:2606.07512), Symphony (CVPR 2026), VideoMind (EACL 2026)

**突破点**: 长视频理解的核心范式转移 — 从 "暴力全帧处理" 到 "代理式主动导航"。关键创新:

| 系统 | 核心方法 | 关键数据 |
|------|---------|---------|
| **Hour-LLaVA** | MemAug 记忆增强, 1-FPS 采样 + 6% token 输入 | 首个 hour-scale 训练+推理 |
| **LongVideo-R1** | 推理模块从顶层摘要导航到信息片段 | 平均 10.5 轮, 50% LVBench (3min/QA) |
| **MemDreamer** | 层级图记忆 + 代理检索, 感知-推理解耦 | 90.7 LVBench (SOTA), 仅 2% 上下文窗口 |
| **Symphony** | 多代理系统, 反思增强协作 | LVBench +5.0% over prior SOTA |
| **VideoMind** | 训练-free, MLLM 自我特化为多尺度搜索+细节模式 | 77.6% Video-MME (4.8% 提升) |

**关键发现**:
- MemDreamer: 感知-推理解耦 → 40x token 减少, 12.5 分绝对提升
- Agentic 能力与长视频理解性能强正相关 (r=0.897, p<0.01)
- 2 小时视频 @ 1FPS = 1.6M token, 超出现有上下文限制

**NeoTrix 融合**: NT-WORLD 的视频处理应实现 MemDreamer 式的感知-推理解耦: 流式感知构建层级图记忆, 推理模型在压缩拓扑上代理检索。这是处理 NT-ACT 视频内容创作管线中超长视频的唯一可行路径。

---

## NeoTrix 融合总结

### 跨域融合矩阵

| 技术 | NT-CORE | NT-MEMORY | NT-WORLD | NT-ACT | NT-IO |
|------|---------|-----------|----------|--------|-------|
| Condensate Theorem | | KV cache bit-exact 压缩 | | | |
| ZeroS / SLA | GWT 负权重路由 | | | | |
| Grassmann Flows | 长上下文 GWT 后备 | | | | |
| Hybrid 注意力 | 架构参考 | | | | |
| qa-FLoRA / LiST | | | | 动态 adapter 融合 | |
| SCIM / CRAG | | 多维检索质量评估 | | | |
| Aria / NFV | | | | 形式化验证 harness | |
| InternVL-X | | | 压缩→解压缩视觉处理 | | |
| MemDreamer | | | 层级图记忆长视频 | | |
| ReasoningCodec | | | | | 音频理解/生成分离 |

### 新增公理候选

| # | 公理 | 推论 |
|---|------|------|
| A4 | **Sparsity Is Learned, Not Imposed** — 训练好的模型天然稀疏, 暴力全注意力是浪费 | Condensate Set 可实现 bit-exact 无损压缩, KV cache 不需要保留所有位置 |
| A5 | **Perception-Reasoning Decoupling** — 感知和推理应解耦而非耦合 | 长视频/长上下文: 感知构建结构化记忆, 推理在压缩拓扑上代理检索 |
| A6 | **Negative Weights Enable Contrast** — 允许负权重的注意力层可执行减法操作 | ZeroS 比 softmax 更具表达力, GWT 应支持抑制机制而非仅增强 |
