# 第97批 破限制技术 — 5大主题

> 采集时间: 2026-09-11 | 每主题 3-5 来源

---

## 主题1: 稀疏训练 (Sparse Training)

### 核心发现

- **SMET (Sparse Memory-Efficient Training)**: 解决 DST 在 LLM 训练中的 cold-start 不稳定性，优化器状态热启动 + 密度感知学习率缩放 + 仅活跃参数存储梯度/优化器状态，LLaMA 在 C4 上接近 dense 性能，显著降低内存开销 (arXiv:2606.00888, 2026-05)
- **Sparse DC Scaling**: 数据受限场景下 DST 缩放定律，引入稀疏感知缩放律 L(N,U,D,S)，稀疏训练延迟数据饱和使多 epoch 更有效，compute-optimal 稀疏下用 8-10× 更少 FLOPs 匹配 dense 精度 (arXiv:2606.01155, 2026)
- **LinBreg 多级框架**: 基于线性化 Bregman 迭代的稀疏训练，交替静态/动态稀疏模式 + 多级优化收敛保证，理论 FLOPs 从 38% 降至 6%，SparseProp 实现前向/反向各减 32%/58% (arXiv:2602.03535, 2026)
- **Chase (Channel-aware Dynamic Sparse)**: 首次将非结构化 DST 无缝转为通道级稀疏，UMM 指标渐进剪枝稀疏通道 + 全局参数重分布，ResNet-50 ImageNet 1.7× GPU 推理加速，无精度损失 (NeurIPS 2023)
- **AAAI 2026 Tutorial**: 十年稀疏训练综述，涵盖监督/无监督/持续学习/深度强化学习全场景，SET/RigL/DST 方法论系统化，提供实践指南 (boqian333/Truly-Sparse-Training, AAAI 2026)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2606.00888 | SMET: Memory-Efficient LLM Training with Dynamic Sparsity | 2026-05 |
| 2 | arXiv:2606.01155 | When Data Is Scarce: Scaling Sparse Language Models with Repeated Training | 2026-06 |
| 3 | arXiv:2602.03535 | Sparse Training via Multilevel Linearized Bregman Iterations | 2026-02 |
| 4 | NeurIPS 2023 | Chase: Dynamic Sparsity Is Channel-Level Sparsity Learner | 2023 |
| 5 | AAAI 2026 | TH01: A Decade of Sparse Training Tutorial | 2026 |

---

## 主题2: 混合精度 (Mixed Precision)

### 核心发现

- **FOG (Fast and Outlier-Guarded)**: 首次全 FP8 GEMM 训练（含注意力），架构设计抑制大激活离群值，8B 模型吞吐提升 43%，匹配 BF16 基线质量，kurtosis 监控预测长期不稳定性 (NeurIPS 2025)
- **Smooth-SwiGLU + FP8 2T tokens**: 扩展 FP8 训练至 2T tokens（此前限 100B），发现 SwiGLU 权重对齐过程导致离群值放大，Smooth-SwiGLU 修正确保稳定 + FP8 Adam 优化器状态，7B 模型 34% 吞吐提升 (arXiv:2409.12517)
- **μnit Scaling (μS)**: 首原则分析 Transformer 微缩放因子，无动态缩放/无特殊超参即可稳定 FP8 训练，1B-13B 模型全隐藏层 FP8 计算，质量等价 + 训练快 33% (ICML 2025)
- **Prompt20 完整指南**: 2026 混合精度标准 — BF16 安全默认，FP8 (e4m3 fwd/e5m2 bwd) 生产标准，FP4 (Blackwell) 前沿，Transformer Engine 必需，per-tensor 缩放 + 逐层排除 (2026-05)
- **MixZ++ (DeepSpeed)**: 混合精度 ZeRO++，量化冻结权重 (qwZ) + 分层分区 (hpZ)，LoRA 训练吞吐提升 3.3×，128 V100 上 Llama-2-70B (2026-03)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | NeurIPS 2025 | FOG: Fully FP8 GEMM LLM Training at Scale | 2025 |
| 2 | arXiv:2409.12517 | Scaling FP8 Training to Trillion-Token LLMs | 2024-09 |
| 3 | ICML 2025 (PMLR v267) | μnit Scaling: Simple and Scalable FP8 LLM Training | 2025-07 |
| 4 | blog.prompt20.com | Mixed Precision LLM Training: The Complete Guide | 2026-05 |
| 5 | DeepSpeed Tutorial | Mixed Precision ZeRO++ (MixZ++) | 2026-03 |

---

## 主题3: 数据并行 (Data Parallelism)

### 核心发现

- **veScale-FSDP**: RaggedShard 灵活分片格式 + 结构感知规划算法，支持块量化和非逐元素优化器 (Muon)，零拷贝通信，吞吐 5-66% 提升 + 内存降低 16-30%，万卡级扩展 (arXiv:2602.22437, 2026)
- **FCDP (Fully Cached Data Parallel)**: 主机内存作为快速缓存层而非溢出层，前向参数缓存后向复用消除 50% 跨节点通信，PEFT 场景冻结权重一次缓存，带宽受限集群吞吐提升 100× (arXiv:2602.06499, 2026-02)
- **CONA (Parallelism Strategy Chaining)**: 在线策略链式切换而非离线单策略，训练中动态切换最优并行配置，GPT-3 1.3B 到目标困惑度加速 1.4-9.6× (arXiv:2609.07236, EMNLP 2026)
- **DCP (Data-Centric Parallel)**: 数据驱动运行时，按批次序列长度动态调整并行度/梯度累积/重计算，变长长序列训练 2.88× 加速，仅需 10 行代码接入 (arXiv:2608.07524, 2026-07)
- **PyTorch FSDP2**: DTensor 原生分片 + 确定性低内存管理 + 隐式/显式预取，FULL_SHARD/SHARD_GRAD_OP/HYBRID_SHARD 策略，512 A100 上 175B 模型 55-60% GPU 利用率，近线性扩展 (VLDB 2023 + PyTorch docs)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2602.22437 | veScale-FSDP: Flexible and Scalable FSDP System | 2026 |
| 2 | arXiv:2602.06499 | FCDP: Fully Cached Data Parallel | 2026-02 |
| 3 | arXiv:2609.07236 | CONA: Parallelism Strategy Chaining for Fast Training Convergence | 2026-09 |
| 4 | arXiv:2608.07524 | DCP: Training Variable Long Sequences with Data-Centric Parallel | 2026-07 |
| 5 | VLDB 2023 + PyTorch docs | PyTorch FSDP: Experiences on Scaling Fully Sharded Data Parallel | 2023 |

---

## 主题4: 张量并行 (Tensor Parallelism)

### 核心发现

- **TSP (Tensor and Sequence Parallelism)**: 将 TP 和 SP 折叠到单一设备轴，每 rank 同时持有权重分片和序列分片，参数和激活内存各降 1/D，注意力广播权重 + MLP 环形权重流通，1024 MI300X 上 2.6× 吞吐提升 (arXiv:2604.26294, 2026)
- **LLEP (Least-Loaded Expert Parallelism)**: 动态路由过载设备溢出令牌+权重到欠载设备，考虑计算/内存/通信成本，5× 加速 + 4× 峰值内存降低，gpt-oss-120b 快 1.9× (arXiv:2601.17111, 2026-01)
- **C2R (Collaboration-Constrained Routing)**: 专家协作/特化视角，限制 top-K 路由组合到协作专家组 + 零冗余 all-to-all 单副本本地复制，LLaMA-MoE/Qwen-MoE 下游 0.51%/0.33% 提升 + 20-30% 运行时间节省 (NAACL 2025)
- **MoX (MoE Routing on Direct-Connect)**: 静态需求无关 MoE 路由，token 感知多播树 + 预计算负载均衡链路权重，1024 TPU Boardfly 上瓶颈链路负载降低 47%，接近理想交换机性能 (arXiv:2607.20220, 2026-07)
- **PyTorch TP Tutorial**: TP + FSDP 组合实战，TP 节点内 + FSDP 跨节点，Sequence Parallel 解决激活内存瓶颈，DTensor 自动插入 AllReduce，PCIe 环境下 TP+SP 比 FSDP 快 6× (2024)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2604.26294 | TSP: Folding Tensor and Sequence Parallelism for Memory-Efficient Training | 2026 |
| 2 | arXiv:2601.17111 | LLEP: Least-Loaded Expert Parallelism | 2026-01 |
| 3 | NAACL 2025 | C2R: Collaboration-Constrained Routing for Better Expert Parallelism | 2025 |
| 4 | arXiv:2607.20220 | MoX: Efficient MoE Routing on Direct-Connect Topologies | 2026-07 |
| 5 | PyTorch Tutorial | Large Scale Transformer Training with Tensor Parallel | 2024 |

---

## 主题5: 流水线并行 (Pipeline Parallelism)

### 核心发现

- **PIPEMORPH**: 滞后者弹性流水线系统，分析模型预测可容忍延迟阈值 + 自适应调度增加 slackness + CPU RDMA 解耦数据面消除头阻塞，128 H800 上 1.36× 超越 ZeroBubble，迭代时间降低 1.2-3.5× (USENIX NSDI 2026)
- **Zero Bubble PP**: 首个零气泡流水线，分解反向为 B(输入梯度)+W(参数梯度)，自动搜索最优调度 + 优化器后验证绕过同步，同内存下比 1F1B 快 23-31%，ZBV 同等内存零气泡 (arXiv:2401.10241, ICML 2024)
- **Merak 3D 并行框架**: 自动模型分区 + 移位关键路径调度 + 阶段感知重计算 + 子流水线 TP 通信重叠，64 GPU 上 20B 模型加速 1.61×，自动搜索最优 TP×PP×DP 分解 (arXiv:2206.04959)
- **3D 并行实战指南**: TP×PP×DP = world_size 硬约束，TP 在 NVLink/PP 在 IB/DP 在最慢织网，100B+ 模型主导策略，配置搜索已成独立子领域，NVL72 NVL8 级 TP (Scalable Book + Factryze, 2026)
- **Zero Bubble 开源实现**: 通用流水线运行时 + ZB/ZBV 重实现，V-Half (1/2 内存) / V-Min (1/3 内存) 变体，`--zero-bubble-v-schedule` 一行启用，支持 padding + 后验证 (GitHub sail-sg/zero-bubble-pipeline-parallelism)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | USENIX NSDI 2026 | PIPEMORPH: Straggler-Resilient Pipeline Parallelism | 2026 |
| 2 | arXiv:2401.10241 | Zero Bubble Pipeline Parallelism | 2024-01 |
| 3 | arXiv:2206.04959 | Merak: Automated 3D Parallelism Deep Learning Training | 2022 |
| 4 | Scalable Book + Factryze | 3D Parallelism: Scale Atlas | 2026 |
| 5 | GitHub sail-sg | Zero Bubble Pipeline Parallelism Implementation | 2024 |

---

## 跨主题洞察

| 洞察 | 涉及主题 |
|------|----------|
| **内存是终极瓶颈**: SMET 仅存活跃参数/veScale-RaggedShard/FCDP 缓存/TSP 折叠内存 → 所有方向都在做参数+激活+通信的三角博弈 | 全部 |
| **拓扑感知成为必选项**: TP 在 NVLink/PP 在 IB/DP 在慢织网/MoX 预计算链路权重 → 通信频率×带宽匹配是效率核心 | TP + PP + DP + EP |
| **调度从静态到动态**: CONA 在线策略链/PIPEMORPH 自适应调度/DCP 数据驱动 → 训练中运行时决策超越离线搜索 | DP + PP |
| **稀疏不只是压缩**: SMET 稀疏训练不是 efficient 的替代而是 scaling 的新维度，数据受限下稀疏改善 scaling trade-off | SP |
| **FP8 全面接管**: FOG 全 FP8 GEMM + μS 无缩放 FP8 + 2T token 扩展 → 2026 FP8 是训练生产默认 | MP |
