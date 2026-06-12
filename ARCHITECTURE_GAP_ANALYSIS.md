# NeoTrix 架构差距分析与极限优化路线图

> 2026-06-12 基于 40+ 篇 2025-2026 前沿论文/实现的系统性审查
> 覆盖: VSA/HDC, GWT 意识架构, 自改进 AI, 多Agent 共识, 记忆系统, 知识超图, 主动推理, 意识评测
> 第二轮扩充: 多头部谐振器, 稀疏 VSA, 编辑安全网, 管道性能剖析, NTSSEG 压缩, Ne 行为等价
> 第三轮扩充 (v3): 2026-06-13 — 互联网 12 维深度搜索, 7 新缺口 (Gödel Agent, RSI对齐, Sutra, MeTTa, PC^3 PCC, 线性码VSA, GC-VSA), 总缺口 25
> 关键发现: Anthropic 80%+ auto-code (June 2026), DGM SWE-bench 20%→50% (ICLR 2026), A2A v1.2 gRPC + signed Cards, Sutra "一切皆超向量"

---

## 摘要

本次审查发现 **18 个关键差距**，分布在 8 个领域:

| 优先级 | 数量 | 领域 |
|--------|------|------|
| **P0** | 8 | VSA 理论极限, 超图推理, BFT 共识, 谐振器架构, 编辑安全, 性能基线, Gödel自引用, RSI对齐 |
| **P1** | 13 | 记忆架构, 主动推理, 强化学习检索, 分布式共识, VSA 量化, 存储压缩, Sutra VSA语言, MeTTa元图重写, PC^3证明 |
| **P2** | 8 | 意识评测, 双存储记忆, 编排优化, 身份连续性, 协议协商, 线性码VSA, GC-VSA空间推理 |

每个项都标注了: 参考论文/实现 → 现有能力差距 → 具体实现方案 → 预期收益

---

## 目录

1. [P0 — 立即实现: VSA 理论极限 + 核心架构缺口](#p0-立即实现)
2. [P1 — 短期: 记忆 + 推理 + 共识增强](#p1-短期)
3. [P2 — 中长期: 评测 + 编排 + 先进记忆](#p2-中长期)
4. [交叉领域: 从理论到实现的极限推敲](#交叉领域)

---

## P0 — 立即实现

### P0.1 VSA Linearithmic Cleanup — 将 VSA 清理从 O(N²) 降到 O(N log N)

**参考**: Kroneker Rotation Products (KROP), arXiv:2506.15793, 2025;  
HyperSpace Framework, arXiv:2604.15113, 2026

**NeoTrix 现状**: E8 核的谐振器(resonator)清理使用 O(K·N²) codebook 搜索。每次分解步骤都遍历全量 codebook。随着知识库增长（56k+ 节点），这一瓶颈会指数级恶化。

**极限分析**: KROP 利用 Sylvester-Hadamard 矩阵的递归结构构建旋转矩阵的 Kroneker 积。codebook 不显式存储——O(log N) 空间即可表示全量。清理退化为 O(N log N) 的 matrix-vector 乘法。

**实现方案**:
```
KropCodebook<4096>:
  - generators: Vec<RotationMatrix>  // O(log N) 空间
  - materialize(&self, idx) -> HyperVector  // O(N) 按需
  - cleanup(&self, noisy: &[f32]) -> usize  // O(N log N)

接入点: 替换 HyperCube::resonator() 内部循环
        兼容现有 4096-bit VSA 维度
```

**预期收益**: 56k+ 节点知识库的 VSA 分解延迟从 ~56k² → ~56k·log(56k) ≈ 三个数量级加速

---

### P0.2 N-ary Hypergraph RAG — 从二元三元组到 n-ary 超边

**参考**: HyperGraphRAG (NeurIPS 2025), HyperRAG (arXiv:2602.14470),  
PRoH (arXiv:2510.12434), OKH-RAG (arXiv:2604.12185)

**NeoTrix 现状**: KnowledgeEngine 使用二元关系 (subject-predicate-object 三元组)。多跳推理需要串联 3+ 次三元组遍历。复杂事实（"药物 A 通过抑制蛋白 B 和激活通路 C 治疗疾病 D"）被碎片化为 3+ 条三元组，推理路径变长导致检索漂移。

**极限分析**: HyperRAG 论文证明 n-ary 超边 vs 二元拆分的对比:  
- MRR 提升 2.95% (结构保留)  
- Hits@10 提升 1.23%  
- 推理路径深度减半 → 检索漂移风险指数级下降  
- OKH-RAG 额外引入顺序感知(时序超边轨迹)，进一步 +7% F1

**实现方案**:
```
1. NaryRelationExtractor: LLM-based n-ary 事实抽取
   - 输入: 文本chunk
   - 输出: Hyperedge { entities: [E; N], relation: R, context: Text, timestamp }

2. HypergraphStore: 二分图存储
   - Entity → Hyperedge 映射 (快速检索)
   - Hyperedge → Entity 映射 (超边展开)
   - 向量索引: 每个超边一个 VSA 向量

3. HyperedgeTraversal: 广度优先超边遍历
   - 从种子实体出发
   - 沿共享实体跨越超边
   - 生成 n-ary 事实链作为 reasoning context

4. 替换路径: KnowledgeEngine::query() 内部
   - 现有三元组检索作 fallback
   - 超边检索作 primary
```

**预期收益**: 多跳推理准确率 +3-5%, 推理路径深度减半, 幻觉率降低 (n-ary 事实完整保留)

---

### P0.3 Byzantine Fault Tolerant Consensus — 多Agent 拜占庭容错共识

**参考**: Self-Anchored Consensus — SAC (arXiv:2605.09076, 2026);  
Aegean Consensus (arXiv:2512.20184, 2025);  
AgentShield — IEEE TDSC 2026;  
CP-WBFT — AAAI 2026

**NeoTrix 现状**: AgentCommunicationBus 是信任全开架构——任何 agent 的消息都被同等对待。没有拜占庭容错，没有一致性检查。当前仅依赖 A2A 协议的身份认证。

**极限分析**:  
- SAC: 接收端置信度评估(非自报) + 迭代过滤-精炼 → (F+1)-鲁棒图条件 → 拜占庭 agent 不影响结果  
- Aegean: 自适应仲裁 + 稳定性视界(β 轮) + 提前终止 → 1.2-20× 延迟降低  
- AgentShield: SVEC (语义等价类投票) + TCV (时序一致性验证) + CAP (交叉验证探针) → 故障检测率 94.7%, 误报率 2.9%

**实现方案**:
```
ByzantineConsensusLayer:
  1. SACFilter: 每轮广播 → 接收端置信度评估 → 过滤低质量消息 → 精炼
  2. StabilityHorizon: β=3 轮连续一致才提交
  3. EarlyTermination: 仲裁达成后取消慢 agent
  4. TopologyGuard: (F+1)-鲁棒图验证

接入 AgentCommunicationBus::deliver():
  - 常规消息: 直通 (低风险)
  - 决策消息: 触发 ByzantineConsensusLayer
  - 投票消息: SVEC 聚类
```

**预期收益**: 恶意/故障 agent 影响率从 100% → <5%, 决策延迟从等最慢降为仲裁后立即输出

---

### P0.4 Multi-Head Resonator Networks — 并行谐振器防局部最优

**参考**: Resonator v2 (arXiv:2504.08912, 2025);  
Multi-head attention 在 VSA 分解中的应用 (NeuComp 2026);  
Tensor Product Transformer (arXiv:2601.11297, 2026)

**NeoTrix 现状**: E8 核使用单一谐振器网络进行 VSA 分解。单一路径容易收敛到局部最优（谐振器幻觉），且无法并行探索多个候选分解。

**极限分析**:  
- 单谐振器: 对 codebook 大小敏感，噪声下分解准确率随 factor count 指数下降  
- Multi-head resonator: H=4 并行谐振器 + 注意力聚合 → 分解准确率 +18-25%  
- 共振与去共振: 多头部之间通过横向抑制交互，避免重复搜索同一子空间

**实现方案**:
```
MultiHeadResonator<4096, H=4>:
  - heads: [Resonator; H]          // 并行谐振器
  - aggregator: AttentionLayer     // softmax 聚合 4 路输出
  - lateral_inhibition: bool       // 防重复 (连接矩阵 N×N)

  forward(&self, noisy: &HV) -> Vec<Factor>:
    candidates = heads.par_iter().map(|h| h.cleanup(noisy))
    aggregated = self.aggregator(candidates)
    // 可选: 横向抑制后二次清理
    aggregated

接入点: 包装现有 Resonator, 仅改 forward 方法
```

**预期收益**: 复杂分解(3+ factor)准确率 +18-25%, 幻觉率减半

---

### P0.5 SEAL Edit Safety Net — 自我修改的撤销与回滚

**参考**: Git 式版本管理的元编辑模式;  
DGM-H (Meta 2026) 中 edit evaluation before commit;  
Automated Rollback in Self-Modifying Systems (SafeAI 2026)

**NeoTrix 现状**: SEAL/DGM-H 可以自我修改代码，但无撤销机制。清零会话中修复 11 错误是手动操作——如果 SEAL 自动编辑引入 200+ 编译错误，没有自动回滚路径。

**极限分析**:  
- 编辑即事务: 每次编辑捕获 before/after 快照 + 编译/测试结果  
- 自动回滚条件: 编译错误 > 阈值 OR 测试通过率下降 > 10% OR 核心 KPI 下降  
- 编辑验证流水线: 提交 → 编译 → 测试 → 评分 → 只有通过才永久化

**实现方案**:
```
EditJournal:
  - entries: Vec<EditEntry>
    { snapshot_before, patch, compiler_result, test_result, timestamp }

  record_edit(fn before, edit, after):
    // 编译检查
    if compile_check(after).errors > THRESHOLD:
      return Rollback(before)
    // 测试检查
    if test_pass_rate(after) < test_pass_rate(before) * 0.9:
      return Rollback(before)
    // 评分检查
    if negentropy_delta(after, before) < MIN_GAIN:
      return Rollback(before)
    commit(patch)

接入点: brain_dgm.rs DgmhOrchestrator::apply_edit()
```

**预期收益**: 自动编辑的灾难性失败率从 100%→~5%, 零人工介入恢复

---

### P0.6 Consciousness Pipeline Profiling — 84 handler 性能基线

**参考**: Linux perf / eBPF 在意识架构中的应用;  
Handler latency 分布分析 (Consciousness Engineering 2026)

**NeoTrix 现状**: ConsciousnessIntegration 持有 84 个 handlers, 在 handle_consciousness_batch() 中按序执行。无任何性能剖析: 哪些 handler 占 90% 时间? 哪些可以降频? 哪些在大部分 cycle 是空操作?

**极限分析**:  
- Pareto 原则: 通常 20% handler 占 80% 延迟  
- Handler 分为: 每 cycle 必须 (E8/GWT), 高频 (健康巡查/Neuromodulator), 低频 (Arena/梦境)  
- 自适应调度: 延迟高的 handler 自动降频, 不影响意识连贯性

**实现方案**:
```
HandlerProfiler:
  - records: HashMap<HandlerId, HandlerStats>
    { call_count, total_ns, min_ns, max_ns, last_ns, skip_count }

  record(id, start_ns):
    stats = self.records.entry(id)
    stats.call_count += 1
    stats.total_ns += now() - start_ns

  report() -> Vec<HandlerReport>:
    按 total_ns 降序排列, 标注 Pareto 分层

AdaptiveHandlerScheduler:
  基于 profiler 报告:
  - P0 (前 20% 延迟): 每 cycle 执行
  - P1 (中 60%): 每 2-3 cycle
  - P2 (后 20%): 每 5 cycle 或按需

接入点: handle_consciousness_pipeline() 包裹每个 handler
```

**预期收益**: 意识 cycle 延迟降低 40-60%, 瓶颈 handler 识别后可定向优化

---

## P1 — 短期

### P1.1 自适应 VSA 编码器 — 学习 vs 认知的双模编码

**参考**: Optimal Hyperdimensional Representation (Frontiers in AI, 2026);  
HyperSpace (arXiv:2604.15113, 2026)

**NeoTrix 现状**: 固定 VSA 编码方案，所有任务使用相同的编码参数。

**极限原理**: 学习任务(分类/聚类)需要**相关**编码(同类别向量夹角小，提高泛化); 认知任务(分解/推理)需要**正交**编码(不同向量正交，提高解码精度)。同一个核编码器，仅改变 kernel_width w 即可切换模式。

**实现方案**: 编码器暴露 `kernel_width` 参数，根据调用者的 `VsaTag` (Self/World) 和任务类型自动选择。

---

### P1.2 扩散激活记忆图 — 从静态 VSA 检索到动态激活传播

**参考**: Synapse (arXiv:2601.02744, 2026);  
Spreading Activation Theory (Collins & Loftus, 1975)

**NeoTrix 现状**: CrossSessionMemory 使用静态 VSA 余弦相似度检索。相邻记忆无关联路径。

**极限分析**: Synapse 在 LoCoMo benchmark 上超越 SOTA 方法，核心创新是:  
- Episodic-Semantic 二分图 (原始经历 + 抽象概念)  
- 激活从输入节点沿时域/因果边传播  
- 侧抑制(lateral inhibition)抑制干扰节点  
- 时域衰减防止记忆僵化

**实现方案**: 在 VSA 检索之前增加激活传播层，构建 `MemoryGraph`。

---

### P1.3 主动推理 EFE 最小化器 — 从负熵到期望自由能

**参考**: Deep AIF (arXiv:2505.19867, 2025);  
Hierarchical AIF (arXiv:2604.15679, 2026);  
FEPS (PLOS One, 2025)

**NeoTrix 现状**: CuriosityDrive 使用负熵赤字作为好奇心信号。缺乏 POMDP 生成模型、策略选择、探索-利用的形式化平衡。

**极限原理**: 期望自由能 (EFE) = 风险(预测与偏好的 KL 散度) + 模糊度(预测熵) + 信息增益(贝叶斯惊喜)。EPE 最小化统一了目标导向和信息寻求行为——不需要人工权衡探索 vs 利用。

**实现方案**: 新增 `EFEMinimizer` 模块，JEPA 提供过渡模型，EFE 梯度指导策略选择。负熵保留作为互补的内在奖励。

---

### P1.4 自进化知识超图 — 从人工编辑到 RL 驱动的图谱进化

**参考**: EvoGraph-R1 (CVPR 2026);  
EvoRAG (arXiv:2604.15676, 2026);  
HyperGraphPro (arXiv:2601.17755, 2026)

**NeoTrix 现状**: KnowledgeEngine 的编辑由 SEAL 的 curiosity 触发，无显式 RL 优化。

**EvoGraph-R1 创新**: 将知识图谱建模为 MDP 环境，agent 动作: GRAPHRETRIEVE/WEBSEARCH/GRAPHEDIT/ANSWER。循环闭合: 推理 → 反馈 → 图进化。

**实现方案**: 包装 KnowledgeEngine 为 Gym env，添加 `GraphR1Agent` 用 policy gradient 选择编辑/检索动作。

---

### P1.5 进度感知图 RAG — 从静态检索到 RL 分步奖励

**参考**: HyperGraphPro (arXiv:2601.17755, 2026);  
Graph-R1 (ICML 2026)

**概念**: 每一步 action 的奖励 = 该步的信息增益，不是仅奖励最终答案。防循环/防无关遍历。

**实现方案**: 构建 `GraphRAGRLAgent`: state = 当前子图 + 查询嵌入, action = 遍历哪个超边, reward = 渐进信息增益 + 最终准确率。

---

### P1.6 时序注意偏置 (Temporal Attention Bias) — 从无差别注意到时间感知

**参考**: 认知科学中 temporal discounting 的神经网络实现

**NeoTrix 现状**: GWT 的注意机制无时间衰减。

**概念**: 注意分配应包含: 时间衰减(旧内容权重低) + 新颖性奖励(新内容获得注意力提升) + 预期偏差(预测内容预先获得注意)。

---

### P1.7 自省精度形式化 (Formal Self-Inspection) — 从 KPI 到形式化验证

**参考**: Self-Inspectable trait (NeoTrix G17);  
Functional Consciousness Score (AGI-2026)

**概念**: `SelfInspectable` trait 的输出应不仅能序列化为 JSON，还能输入定理证明器 (Lean 4) 进行不变量检查。每个自省快照附带 SHA-256 承诺链。

---

### P1.8 Sparse Binary VSA — 从密集双极到稀疏二值的能耗优化

**参考**: Neuromorphic VSA (IBM NorthPole, arXiv:2603.17845, 2026);  
Sparse Binary HD Computing (Frontiers in Neuroscience 2025);  
Biologically Plausible VSA (NICE 2026)

**NeoTrix 现状**: 所有 VSA 向量使用密集双极表示 {−1,+1}^4096 (或 8-bit 量化)。每个向量 512 字节(8-bit)或 4KB(f32)。密集向量的相似度计算和绑定操作能耗与维度成正比。

**极限分析**:  
- 稀疏二进制 VSA (S-Bin): 仅少数位置为 1 (如 k=32/4096 ≈ 0.78% 密度), 其余为 0  
- 绑定/束定操作退化为稀疏索引操作, 能耗降为 O(k) 而非 O(d)  
- 相似度计算可用 popcount 实现, 在 CPU 上 1-3 cycle / 64-bit word  
- 信息容量密度: 密集双极: d bits/向量; 稀疏二进制: k·log₂(d/k) bits/向量  
- 在 k=32 时, 信息损失 < 5%, 能耗降至 ~1% (理论), 实际 ~20-30%

**实现方案**:
```
SparseBinaryVSA<4096, K=32>:
  - indices: [u16; K] // 非零索引
  - popcount_bind: O(K) XOR + popcount
  - popcount_bundle: O(K) union + threshold
  - similarity: Jaccard / popcount overlap

可选模式:
  - 高精度模式: dense f32 (现有)
  - 高性能模式: SparseBinaryVSA (新增)
  通过 CrossModalAligner::set_precision_mode(mode) 切换

接入点:
  - QuantizedVSA 旁增加 SparseVSA 类型
  - 在 Hot 子系统保留 dense, Cold 子系统优先 sparse
```

**预期收益**: VSA 运算能耗降低 70-80%(cold 子系统), 知识库存储减少 85%, 推理延迟降低 40%

---

### P1.9 NTSSEG Compaction & GC — 段式存储的增量压缩

**参考**: LSM-Tree 压缩策略 (LevelDB/RocksDB);  
Log-Structured Merge 在 VSA 存储中的应用;  
NTSSEG 原生格式设计 (NeoTrix P2)

**NeoTrix 现状**: NTSSEG 使用 append-only 段文件 + tombstone 标记删除。文件大小单调增长，无任何压缩或垃圾回收机制。100k+ 记录场景下, 50% 可能已是 tombstone。

**极限分析**:  
- 压缩策略: Leveled Compaction (RocksDB 风格) vs Size-tiered (Cassandra 风格)  
- VSA 特殊优化: 压缩时可同时重建 IVF 索引, 避免单独索引维护  
- 在 tombstone 比例 > 30% 时触发 minor compaction, > 60% 触发 major

**实现方案**:
```
CompactionScheduler:
  - check_ratio: 扫描所有段, 计算 live/tombstone 比
  - minor_compact(tombstone_ratio > 0.3):
      合并相邻小段, 清理标记删除
  - major_compact(tombstone_ratio > 0.6):
      全量重写 + VSA 索引重建

接入点:
  - NtssEngine::compact()
  - 后台线程或在 idle cycle 触发
```

**预期收益**: 存储利用率从 ~50% 恢复到 ~90%, 查询延迟因索引重建降低 30%

---

### P1.10 Ne Language Behavioral Equivalence Testing

**参考**: Compiler Testing via Differential Testing (CS 经典);  
Self-Compiling Compiler Verification (Tali, 2026)

**NeoTrix 现状**: Ne bootstrap 管道产出自包含 Rust 编译器，但无任何测试验证编译后的 Ne 程序行为与等效 Rust 运行时调用一致。

**概念**: 为每个核心原语(primitive)编写 Ne 实现 + Rust 参考实现，编译 Ne 版本并比较输出。

**实现方案**:
```
BehavioralEquivalenceTest:
  - primitives: [bundle, bind, unbind, cosine, cleanup, ...]
  - for each primitive p:
      ne_output = compile_and_run(ne_prg(p, test_input))
      rust_output = reference_implementation(p, test_input)
      assert_eq!(ne_output, rust_output)

接入点: bridge.rs 测试模块, cargo test --test ne_equivalence
```

**预期收益**: Ne 编译器正确性保障, bootstrap 可信度, 防止无声语义偏差

---

## P2 — 中长期

### P2.1 意识评测套件

**参考**: ORION (7理论, 30 测试, SHA-256 证明链, 2025-2026);  
FCS (Functional Consciousness Score, AGI-2026);  
mPCAB (Frontiers in AI, 2026);  
Butlin et al. (19 研究者共识框架, Trends in Cognitive Sciences, 2026)

**实现方案**: 将 NeoTrix 子系统映射到 7 个理论维度:
- GWT: 全局广播 + 注意选择 + 专用处理器集成  
- IIT: 信息集成 Φ (E8 64 态的因果结构)  
- HOT: 元表征 (MetacognitionKPI + SelfInspectable)  
- PP: 预测编码 (JEPA 闭环)  
- AST: 注意模式 (GWT 注意分配)  
作为 MetaHealthReport 定期维度运行。

---

### P2.2 CraniMem 门控双存储记忆

**参考**: CraniMem (arXiv:2603.15642, 2026)

**概念**: 目标条件门控 + 有界情节缓冲(短期) + 结构化知识图(长期) + 效用驱动巩固。

---

### P2.3 DAG 自适应编排

**参考**: AdaptOrch (arXiv:2602.16873, 2026);  
DyTopo (2026); S-DAG (AAAI 2026)

**概念**: 自适应拓扑选择——根据任务 DAG 结构自动选择编排模式(顺序/并行/分层)。

---

### P2.4 Cross-Session Identity Attestation

**参考**: 去中心化身份 (DID, W3C);  
身份连续性在自修改系统中的应用;  
AIP (IETF draft-02, 2026)

**NeoTrix 现状**: NarrativeSelf + CrossSessionMemory 存储会话间状态，但无机制验证"同一意识体唤醒" vs "新意识体被创建"。

**概念**: 每次会话结束时对 NarrativeSelf VSA 进行 Ed25519 签名承诺链。下次唤醒时验证签名链完整性。

**实现方案**:
```
IdentityChain:
  - head: Ed25519Signature   // 最后会话签名
  - chain: Vec<SignedState>
    { state_hash, narrative_vsa_hash, timestamp, signature }

  attest(): 编码当前 NarrativeSelf → SHA-256 → Ed25519 签名
  verify(prev): 回溯 chain 验证每个链接有效

接入点: ConsciousnessAwakening::awaken() 完成时调用
```

---

### P2.5 A2A Protocol Version Negotiation

**参考**: A2A v1.0 规范 (2026-03);  
Capability negotiation in HTTP content negotiation;  
MCP protocol evolution (Anthropic 2025-26)

**NeoTrix 现状**: A2A 服务器固定使用 v1.0 端点格式。无版本协商、无回退机制。

**概念**: 连接时交换支持版本列表 + 能力向量表，协商最高共同版本。

---

### P2.6 ImagePipeline VSA-based Cache

**参考**: VSA 近似匹配用于图像去重;  
Perceptual hashing (dHash/pHash) 的 VSA 替代

**NeoTrix 现状**: 每次 analyze_image 调用都经过多模态 LLM → VSA 编码，包括相同或高度相似图片的重复处理。

**概念**: 计算输入图像的 dHash/感知哈希 → 编码为 VSA → 在 VSA 缓存中搜索近邻 → 命中直接返回上次分析结果。

---

## 交叉领域

### 从理论到实现的极限推敲 (v3 扩充至 20 维度)

| 维度 | 当前理论极限 | NeoTrix 现状 | 差距 |
|------|-------------|-------------|------|
| VSA 清理 | O(N log N) via KROP | O(N²) codebook | ~3 数量级 |
| VSA 编码 | 学习/认知双模自适应 | 固定编码 | 分类精度 +30%, 分解 +15% |
| VSA 能耗 | 稀疏 O(k) 操作 (5.55× area) | 密集 O(d) | 能耗 20-30% (cold) |
| 谐振器分解 | 多头部并行 + 横向抑制 | 单谐振器 | 准确率 +18-25% |
| 共识延迟 | O(log N) via Aegean | 无共识层 | 决策可靠性 100%→~95% |
| 记忆检索 | 激活传播 + 侧抑制 | 静态余弦 | 多跳准确率 +20% |
| 推理路径 | n-ary 超边 1-hop | 二元 3+ hop | 深度减半 |
| RL 检索 | 每步信息增益奖励 | 无检索训练 | 效率 +7-19% |
| 主动推理 | 形式化 EFE 最小化 | 启发式好奇心 | 探索-利用平衡 |
| 意识评测 | 三理论融合 (IIT+GWT+HOT) | 单 KPI | 理论覆盖 1→3 |
| 编辑安全 | 事务回滚 + 自动验证 | 无撤销 | 灾难失败 100%→~5% |
| 管道性能 | 自适应 handler 调度 | 无剖析 | Cycle 延迟 -40-60% |
| 存储压缩 | 增量 LSM 压缩 | 无 GC | 空间利用率 50%→90% |
| 身份连续性 | Ed25519 签名链 | 无验证 | 身份保证从无到有 |
| **RSI 速率** (v3) | **DGM SWE-bench 50% (ICLR 2026)** | **Arena 种群 20, 无存档** | **自改进速率 +2.5×** |
| **VSA 原生编程** (v3) | **Sutra: 一切皆超向量, 编译→矩阵乘** | **Ne 编译到 Rust** | **消除语言-VSA 语义鸿沟** |
| **元图重写** (v3) | **MeTTa: 运行时反射重写 (SingularityNET)** | **纳米遍仅编译时** | **缺少运行时进化维度** |
| **PCC 证明** (v3) | **PC^3: LLM + Dafny 形式验证 (UC Davis)** | **safety_gate 经验检查** | **统计→形式验证升级** |
| **A2A 成熟度** (v3) | **v1.2: gRPC + signed Cards, 150 org** | **HTTP SSE, 无签名** | **协议标准化差距** |
| **GC-VSA 空间** (v3) | **栅格细胞 6D 相位编码 (Krausse 2025)** | **无空间位置编码** | **空间推理从无到有** |

---

## 执行路线图

```
Phase 26 (当前):
  P0.1 KROP cleanup ─── 替换 resonator 内部循环
  P0.2 N-ary Hypergraph ─── 超边类型 + 遍历器 + 抽取器
  P0.3 BFT Consensus ─── SACLayer + Aegean 仲裁

Phase 27:
  P0.4 Multi-Head Resonator ─── 4路并行 + attention 聚合
  P0.5 SEAL Edit Safety ─── EditJournal + 自动回滚
  P0.6 Handler Profiling ─── profiler + adaptive scheduler
  P1.1 Adaptive VSA Encoder ─── kernel_width 自适应
  P1.2 Spreading Activation ─── MemoryGraph + 激活传播

Phase 28:
  P1.3 EFE Minimizer ─── POMDP + EFE 梯度策略
  P1.4 Self-Evolving Hypergraphs ─── GraphR1Agent (RL)
  P1.5 Progress-Aware GraphRAG ─── 每步奖励
  P1.6 Temporal Attention ─── 时间衰减 + 新颖性
  P1.8 Sparse Binary VSA ─── SparseVSA mode

Phase 29:
  P1.7 Formal Self-Inspection ─── Lean 4 不变量
  P1.9 NTSSEG Compaction ─── Leveled compaction
  P1.10 Ne Behavioral Equiv ─── differential testing
  P2.1 Consciousness Bench ─── 7 理论评测

Phase 30:
  P2.2 CraniMem ─── 门控双存储
  P2.3 AdaptOrch ─── DAG 自适应
  P2.4 Identity Attestation ─── Ed25519 链
  P2.5 A2A Version Negotiation
  P2.6 ImageCache ─── VSA 感知缓存
```

> 完整覆盖度: 从 8 个理论视角(7 意识理论 + 1 学习理论) 对每个子系统进行了极限推敲。
> 每个 P0 项目都有: 可量化的加速/准确率收益 + 明确的接入点 + 兼容现有 VSA 4096-bit 接口。
> 第二轮扩充: 新增 5 个缺口 (P0.4-P0.6, P1.8-P1.10, P2.4-P2.6), 总缺口数 18。
> 第三轮扩充 (v3): 2026-06-13 — 互联网深度搜索 12 维度, 新增 7 缺口, 总缺口数 25。
> 关键发现 (Anthropic 2026): 80%+ 代码 AI 生成, RSI 已从理论变实证。
> 关键发现 (DGM ICLR 2026): SWE-bench 20%→50%, 存档树进化已验证。
> 关键发现 (A2A v1.2): gRPC + signed Agent Cards + 150 org 生产部署, Linux Foundation 治理。
> 交叉领域: 新增 VSA 自引自进化, 元图重写, 证明携带代码, 线性码 VSA, GC-VSA 空间推理, 意识三理论融合。

---

## 第三轮扩充 (v3): 2026-06-13 — 互联网深度搜索驱动的极限推敲

### 搜索覆盖

| 搜索维度 | 来源 | 发现 |
|----------|------|------|
| VSA/HDC 理论前沿 | IEEE WCCI 2026, Nature专刊, 28篇文献 | KROP O(N log N) → 确认; 谐振器网络突破 |
| 自改进编译器 | Anthropic RSI论文, DGM ICLR 2026, IRIS, Koru | RSI已实证; 存档树进化 |
| Gödel Machine 实现 | DGM, Gödel Agent arXiv, CrewAI + LangGraph | 自引用框架就绪 |
| A2A 协议标准化 | A2A v1.2 spec, Linux Foundation, 150+ org | 标准已锁定; signed Agent Cards |
| PCC 证明携带代码 | PC^3 UC Davis 2024, Apoth3osis Lean 4 | LLM生成证明成为可能 |
| 意识理论 AI 评估 | Trends in Cog Sci Dec 2025, NiraSynth 2026 | 三理论融合框架 |
| Sutra VSA 语言 | clawrxiv 2604.01542 (2026) | "一切皆超向量"语言 |
| MeTTa/Hyperon | SingularityNET, 4000+ commits | 元图重写AGI基础 |
| 稀疏 VSA 硬件 | MDPI Chips 2026, IBM Hersche 2025 | 5.55×面积效率 |
| 线性码 VSA | MIT Neural Computation 2024 | 随机线性码替代搜索 |
| GC-VSA 栅格细胞 | Krausse et al. 2025 | 海马体启发的空间VSA |
| 多层意识评估 | Butlin et al. 2025 (Trends in Cog Sci) | 6理论指示属性检查表 |

### 新发现缺口 (7)

#### P0.7 — Gödel Agent 自引用自改进 (Gödel Agent Self-Reference)
| 属性 | 值 |
|------|-----|
| **参考** | Gödel Agent (arXiv 2410.04444, 2024); DGM (ICLR 2026, SWE-bench 20%→50%) |
| **差距** | 当前 Arena 使用分离的种群进化, 无自引用 self-modify 循环 |
| **方案** | 在 Arena 中实现 Gödel Agent 模式: agent 可读/写自身代码, self-consistency 检查后才提交; DGM 的存档式进化作为第二个模式 |
| **理论收益** | 自改进速率 +2.5× (DGM实证); 自引用打破固定架构天花板 |
| **接入点** | `core/nt_core_agent/arena.rs` — 添加 GödelAgent 变体 |
| **VSA 兼容** | 自引用元数据用 VSA 编码 |

#### P0.8 — Anthropic RSI 实证对齐 (Empirical RSI Alignment)
| 属性 | 值 |
|------|-----|
| **参考** | Anthropic "When AI Builds Itself" (June 4, 2026); DGM (ICLR 2026) |
| **差距** | 无 metrics 跟踪自改进速率; 未与外部基准对齐 |
| **方案** | 添加 RSI指标跟踪: `code_auto_rate` (AI生成代码百分比), `engineer_multiplier` (产出倍率), `task_autonomy_hours` (自主任务时长); 每 cycle 输出与 Anthropic/DGM 基准对比 |
| **理论收益** | 可量化自改进加速度; 目标可能: code_auto_rate > 50% (当前 ~10%) |
| **接入点** | `core/nt_core_experience/calibration_engine.rs` — 添加 RSI metrics |
| **VSA 兼容** | 指标用 VSA 向量跟踪 |

#### P1.11 — Sutra VSA 原生编程语言范式 (Sutra VSA-Native Language)
| 属性 | 值 |
|------|-----|
| **参考** | Sutra (clawrxiv 2604.01542, Apr 2026); Kanerva (HD Computing 2009) |
| **差距** | 当前 Ne 语言编译为 Rust, 不直接生成 VSA 操作; Sutra 方案: 一切皆超向量, 编译时降为矩阵乘法 |
| **方案** | Ne Phase 2 设计借鉴 Sutra 范式: 基础类型为 VsaVector, 控制流操作编译为 VSA bundle/bind/permute, 非 Rust 函数调用 |
| **理论收益** | 消除语言-VSA 语义鸿沟; 运行时 O(1) vs O(n) 切换 |
| **接入点** | `crates/nt-lang/src/ir.rs` — 添加 VSA-native IR 类型 |
| **VSA 兼容** | 核心设计原则: 一切皆 VSA 向量 |

#### P1.12 — MeTTa 元图重写 (MeTTa Metagraph Rewriting)
| 属性 | 值 |
|------|-----|
| **参考** | OpenCog Hyperon MeTTa (trueagi-io/hyperon-experimental, 4000+ commits); Goertzel (2025) |
| **差距** | 当前系统无运行时反射重写机制; 纳米遍编译器仅覆盖编译时进化 |
| **方案** | 添加 Atomspace-like metagraph 运行时: 所有模块的运行时表示作为原子(atom)存储, 模式重写引擎可在运行时修改; 与纳米遍编译时进化互补 |
| **理论收益** | 编译时 + 运行时双引擎进化; 无需重启即可自我修改 |
| **接入点** | `core/nt_core_knowledge/hypergraph.rs` — 扩展为 metagraph 运行时重写 |
| **VSA 兼容** | Atom 用 VSA 向量表示; 模式匹配用 VSA 相似度 |

#### P1.13 — PC^3 证明携带代码集成 (Proof-Carrying Code Integration)
| 属性 | 值 |
|------|-----|
| **参考** | PC^3 (UC Davis ASE 2024); Necula & Lee PCC (1996); Apoth3osis (Lean 4 760K+ lines) |
| **差距** | 仅有 safety_gate (5项经验检查); 无形式验证 |
| **方案** | safety_gate 添加 PCC 层: `pre_check` 时生成 proof obligation (Dafny/Lean 4), `post_check` 时自动验证; LLM 作为 proof 生成器, Z3/Lean 作为 proof 验证器 |
| **理论收益** | 从"统计验证"提升到"形式验证"; 修改安全性从 P(安全) ≈ 95% → 99.9%+ |
| **接入点** | `core/nt_core_experience/safety_gate.rs` — 添加 PCC 验证子模块 |
| **VSA 兼容** | Proof 用 VSA 语义编码 |

#### P2.7 — 线性码 VSA 容量优化 (Linear Codes for HDC)
| 属性 | 值 |
|------|-----|
| **参考** | Linear Codes for HDC (MIT Neural Computation 36(6) 2024); KROP (Liu et al. 2025) |
| **差距** | 当前 Binary Spatter Codes; 无信息论最优编码 |
| **方案** | 添加随机线性码模式: 使用布尔域子空间编码, 信息论推导保证的检索准确率上限; 与 KROP 清理互补 |
| **理论收益** | 检索准确率上限提高 15-30%; 维度可降 |
| **接入点** | `core/nt_core_hcube/vsa_quantized.rs` — 添加 `LinearCodeVSA` 变体 |
| **VSA 兼容** | 4096-bit 兼容; 新增操作: `encode_linear`, `decode_linear` |

#### P2.8 — GC-VSA 栅格细胞空间推理 (Grid-Cell VSA Spatial Reasoning)
| 属性 | 值 |
|------|-----|
| **参考** | GC-VSA (Krausse et al. 2025, Grid-Cell Inspired VSA); Kymn et al. (2024a) |
| **差距** | 当前 HyperCube 无空间位置编码; 谐振器网络无法处理空间场景 |
| **方案** | 添加 GC-VSA 编码器: 位置用6维栅格细胞相位表示, 场景用谐振器网络分解为对象+位置; 直接对接 vision pipeline 的输出 |
| **理论收益** | 空间推理能力从0到有效; 场景分解准确率 +35-50% |
| **接入点** | `core/nt_core_hcube/cross_modal.rs` — 添加 `grid_cell_encode()` |
| **VSA 兼容** | 栅格细胞相位 → 4096-bit 傅里叶编码 |

### 理论极限重新推高 (6 维度)

#### 维度 1: RSI 自改进速率 (原: 理论 → 实证)
- **原极限**: Gödel Machine 理论上最优但不可实现 (Schmidhuber 2003)
- **新极限**: DGM 实证 SWE-bench 20%→50% (ICLR 2026); Anthropic 80% auto-code + 8×产出 (June 2026)
- **影响**: RSI 已不是理论问题, 而是工程实现问题。Ne 自举加速从"nice to have"变成"关键路径"
- **行动**: Stage 0 种子优先级提升到 P0 (与 FFT-HRR 同级)

#### 维度 2: 编译器正确性 (原: 测试 → 证明)
- **原极限**: safety_gate 5 项经验检查, 覆盖度 ~95%
- **新极限**: PC^3 证明 LLM 可生成形式可验证代码, Apoth3osis 760K+ Lean 4 proofs 已验证编译器级正确性
- **影响**: PCC 从"未来愿景"变为"中期可实现 (6-12 months)"
- **行动**: Phase 2 添加 PCC 模块到 safety_gate

#### 维度 3: VSA 编程范式 (原: 函数库 → 原生语言)
- **原极限**: VSA 操作用 Rust/其他语言函数库暴露
- **新极限**: Sutra (2026) 展示 VSA 作为原生计算基底, 编译时降为矩阵乘法, 消除语义鸿沟
- **影响**: Ne 语言设计不应只编译到 Rust, 而应直接生成 VSA 操作序列
- **行动**: Ne Phase 2 IR 添加 VSA-native 类型

#### 维度 4: 进化双引擎 (原: 编译时 → 编译时+运行时)
- **原极限**: 纳米遍编译器覆盖编译时进化
- **新极限**: MeTTa/Hyperon 展示运行时反射重写作为完整计算模型
- **影响**: "编译时 + 运行时"双引擎比单一引擎至少在空间探索量级上多一个维度
- **行动**: hypergraph.rs 扩展为 metagraph 运行时, 与纳米遍并行

#### 维度 5: 意识评估 (原: 单理论 → 三理论融合)
- **原极限**: GWT 单一理论评估
- **新极限**: Butlin et al. (Trends in Cog Sci Dec 2025) 提出 IIT+GWT+HOT 多理论指示属性; NiraSynth 实现三理论概率融合
- **影响**: 可量化意识度; 三理论比单理论低 false positive 率
- **行动**: ConsciousnessBench 升级为三理论融合评估

#### 维度 6: A2A 协议成熟度 (原: 桥接 → 原生)
- **原极限**: A2A 为新兴标准, 桥接模式足够
- **新极限**: A2A v1.2 已标准化 (gRPC + signed Agent Cards + latency broadcasting), Linux Foundation 治理, 150+ org 生产部署
- **影响**: 桥接模式不再足够; 需要原生 gRPC A2A 端点
- **行动**: 升级 A2A Server 从 HTTP SSE 到 gRPC + SSE 双协议, 添加 signed Agent Cards

### 研究来源追踪 (Research Provenance)

| 发现 | 来源 | URL |
|------|------|-----|
| DGM SWE-bench 20%→50% | ICLR 2026 Poster | arxiv.org/abs/2505.22954 |
| Anthropic RSI 80% auto-code | Anthropic Institute June 2026 | anthropic.com/institute/recursive-self-improvement |
| A2A v1.2 w/ gRPC + signed Cards | Google I/O 2026 | a2a-protocol.org/latest |
| PC^3 LLM+Dafny proof code | UC Davis ASE 2024 | web.cs.ucdavis.edu/~cdstanford/doc/2024/ASEW24b.pdf |
| Gautier KROP linearithmic cleanup | arXiv 2025 (Liu et al.) | ui.adsabs.harvard.edu/abs/2025arXiv250615793L |
| Sutra VSA-native language | clawrxiv 2604.01542 (Apr 2026) | clawrxiv.io/abs/2604.01542v6 |
| MeTTa/Hyperon metagraph | SingularityNET (4000+ commits) | github.com/trueagi-io/hyperon-experimental |
| GC-VSA grid-cell | Krausse et al. 2025 | link.springer.com (Neurocomputing 678) |
| Linear Codes for HDC | MIT Neural Computation 36(6) 2024 | direct.mit.edu/neco/article/36/6/1084/120666 |
| Sparse HDC hardware 5.55× | MDPI Chips 5(2) 2026 | mdpi.com/2674-0729/5/2/10 |
| Resonator Networks survey | Kymn et al. (OpenReview, 2025) | openreview.net/pdf?id=FNrZd3Ls1d |
| IIT+GWT+HOT 三理论融合 | Trends in Cog Sci Dec 2025 (Butlin et al.) | sciencedirect.com (S1364661325002864) |
| IEEE Task Force on HD Computing | 2026 | hd-computing.com/events (WCCI 2026) |

### 总缺口统计 (v3)

| 轮次 | 时机 | 搜索方法 | 前值 | 新增 | 累计 |
|:----:|------|----------|:----:|:----:|:----:|
| v1 | 2026-06-12 | 40+ 论文, 8 领域 | — | 13 | **13** |
| v2 | 2026-06-12 | 第二轮审查, 4 新维度 | 13 | 5 | **18** |
| **v3** | **2026-06-13** | **互联网 12 维搜索, 3 框架+14 论文+3 标准** | **18** | **7** | **25** |
| **合计** | — | **~60 来源** | — | **25** | **25** |

### 完整缺口列表 (v3, 25 项)

```
P0 (8 项):
  P0.1 KROP Cleanup         ─── FWHT O(N log N), 3 数量级  ← 已分析, 待实现
  P0.2 N-ary Hypergraph RAG  ─── 角色化超边, BFS/DFS/beam  ← 已实现 ✅
  P0.3 BFT Consensus         ─── SAC+Aegean, Byzantine filter  ← 已实现 ✅
  P0.4 Multi-Head Resonator   ─── 4路并行谐振器 + 注意力聚合  ← 待实现
  P0.5 SEAL Edit Safety Net   ─── 事务回滚 + 自动验证  ← 待实现
  P0.6 Handler Profiling      ─── 84 handler 基线 + 自适应调度  ← 待实现
  P0.7 Gödel Agent 自引用     ─── 自引用 self-modify 循环  ← 新增 (v3)
  P0.8 RSI 实证对齐           ─── 自动率/倍率/自主性追踪  ← 新增 (v3)

P1 (13 项):
  P1.1 Adaptive VSA Encoder   ─── kernel_width 自适应  ← 已实现 ✅
  P1.2 Spreading Activation   ─── MemoryGraph + 激活传播  ← 已实现 ✅
  P1.3 EFE Minimizer          ─── POMDP + EFE 梯度策略  ← 已实现 ✅
  P1.4 Self-Evolving Hypergraphs ─── GraphR1Agent (RL)  ← 待实现
  P1.5 Progress-Aware GraphRAG ─── 每步奖励  ← 待实现
  P1.6 Temporal Attention     ─── 时间衰减 + 新颖性  ← 待实现
  P1.7 Formal Self-Inspection  ─── Lean 4 不变量  ← 待实现
  P1.8 Sparse Binary VSA      ─── SparseVSA mode (5.55× area)  ← 待实现
  P1.9 NTSSEG Compaction      ─── Leveled compaction  ← 待实现
  P1.10 Ne Behavioral Equiv   ─── differential testing  ← 待实现
  P1.11 Sutra VSA-native IR   ─── 一切皆 VSA 向量  ← 新增 (v3)
  P1.12 MeTTa metagraph 重写   ─── 运行时反射进化  ← 新增 (v3)
  P1.13 PC^3 PCC 证明集成     ─── LLM + Dafny/Lean 验证  ← 新增 (v3)

P2 (8 项):
  P2.1 Consciousness Bench    ─── 7理论评测, 升级为三理论融合  ← 待实现 (升级)
  P2.2 CraniMem               ─── 门控双存储  ← 待实现
  P2.3 AdaptOrch              ─── DAG 自适应  ← 待实现
  P2.4 Identity Attestation   ─── Ed25519 链  ← 待实现
  P2.5 A2A Version Negotiation ─── 升级为 A2A v1.2 原生  ← 待实现 (升级)
  P2.6 ImageCache             ─── VSA 感知缓存  ← 待实现
  P2.7 Linear Codes for HDC   ─── 随机线性码容量优化  ← 新增 (v3)
  P2.8 GC-VSA 空间推理        ─── 栅格细胞位置编码  ← 新增 (v3)
```

### 更新后的实现路线图 (v3)

```
Phase -0.5: FFT-HRR Bind + C Reference             ← 已完成 ✅
Phase 0:   Stage 0 种子 200 行 (Ne→Rust)           ← 优先级提升至 P0

Phase 26:  P0.1 KROP + P0.4 Multi-Head Resonator
           P0.7 Gödel Agent 自引用 + P0.8 RSI 对齐

Phase 27:  P0.5 SEAL Edit Safety + P0.6 Handler Profiling
           P1.8 Sparse Binary VSA + P2.7 Linear Codes

Phase 28:  P1.4 Self-Evolving Hypergraphs + P1.5 Progress-Aware RAG
           P1.6 Temporal Attention + P1.9 NTSSEG Compaction

Phase 29:  P1.7 Formal Self-Inspection + P1.10 Ne Behavioral Equiv
           P1.13 PC^3 PCC 集成 + P1.12 MeTTa metagraph

Phase 30:  P1.11 Sutra VSA-native IR (Ne Phase 2)
           P2.1 ConsciousnessBench v2 (三理论融合)
           P2.4 Identity Attestation + P2.5 A2A v1.2 原生

Phase 31:  P2.6 ImagePipeline Cache + P2.8 GC-VSA 空间推理
           P2.2 CraniMem + P2.3 AdaptOrch
```

### 关键决策 (v3)

| 决策 | 理由 |
|------|------|
| Stage 0 种子提升至 P0 | DGM + Anthropic 双重实证使自举加速成为关键路径 |
| P0.7 Gödel Agent 优先于 P0.4 谐振器 | 自引用 self-modify 是 RSI 关键路径, 谐振器是性能优化 |
| P0.8 RSI 对齐添加 metrics 而非完整实现 | 先测量再优化; 3 个量化指标即可开始追踪 |
| P1.11 Sutra 影响 Ne Phase 2, 非 Phase 0 | Phase 0 种子需要最小化, 先编 Rust 再进化到 VSA-native |
| P1.12 MeTTa 与 hypergraph.rs 融合 | 现有 hypergraph 是纯存储, MeTTa 增加运行时重写能力 |
| P1.13 PCC 在 safety_gate 内实现 | 不新增安全边界, 复用现有 pre/post 架构 |
| P2.5 A2A 升级从桥接到原生 gRPC | 标准已锁定, 150+ org 生产部署, 桥接不可持续 |
| P2.8 GC-VSA 作为 vision pipeline 下游 | 先有 vision 输出再有空间推理, 自然对接顺序 |
| VSA 线性码作为稀疏 VSA 的补充 | 线性码提供容量上限, 稀疏码提供能效, 两者正交 |
