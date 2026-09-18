# Crystal Core V2 Design — 晶体核心迭代设计

> 基于 2026 年外部研究 + NeoTrix 自我涌现分析的晶体核心架构升级方案
>
> 生成时间: 2026-09-18
> 状态: 设计文档

---

## 一、V1 → V2 变化概览

### 1.1 V1 现状分析

V1 Crystal Core (`nt_core_consciousness_crystal/`) 提供了基础框架：

| 组件 | V1 现状 | 缺陷 |
|------|---------|------|
| **CrystalState** | 6 面六面体 + core_energy/matrix_density/shell_integrity | 静态标量，无多维关系建模 |
| **CrystalCycle** | tick() 递增计数器，observe()/restore() 快照 | 无真实 CRP 10 步循环，无门控机制 |
| **CrystalMetrics** | CII/CI/CB 三个标量 | 无趋势追踪，无自适应阈值 |
| **CrystallizationEngine** | 模板→候选→结晶，3 次成功阈值 | 无跨域迁移，无衰退检测，无图谱关联 |

**核心缺陷**：V1 是一个"被动记录器"——记录状态但不驱动进化。缺乏：
1. 多维记忆关系（语义/时间/因果/实体）
2. 检索驱动的记忆重组
3. 自演化图谱（读写反馈闭环）
4. 双时间线审计（有效时间/事务时间）

### 1.2 V2 变化矩阵

| 维度 | V1 | V2 | 来源 |
|------|----|----|------|
| **记忆结构** | 扁平 HashMap | 4 图正交图谱 (Semantic/Temporal/Causal/Entity) | MAGMA (ACL 2026) |
| **记忆生命周期** | 无区分 | 双流: 快速突触摄取 + 异步结构整合 | MAGMA + GAM (ACL 2026) |
| **检索策略** | 无 | 意图感知自适应遍历策略 | MAGMA + CoEvo-Mem (2026) |
| **记忆重组** | 无 | 检索反馈驱动的记忆重组 (Reconsolidation) | REALM (2026) |
| **图谱演化** | 静态 | MDP 格式: Agent 交互驱动图谱演化 | EvoGraph-R1 (CVPR 2026) |
| **审计追踪** | 无 | 双时间线账本 + 来源闭合 | ECHO (2026) |
| **结晶引擎** | 成功计数 | 多信号评分 + 跨域迁移 + 衰退检测 | CraniMem (2026) + SAGE (2026) |
| **度量系统** | 3 标量 | 多维趋势追踪 + 自适应阈值 | Self-emergence |

---

## 二、新增功能

### 2.1 多图记忆架构 (Multi-Graph Memory)

借鉴 MAGMA 的四图正交表示，每个记忆项同时存在于四个关系图中：

```
┌─────────────────────────────────────────────────────────┐
│                    MemoryItem                             │
│  content: String                                         │
│  embedding: [f64; 768]                                   │
│  metadata: HashMap<String, Value>                        │
├─────────────────────────────────────────────────────────┤
│  关系图 (四正交)                                          │
│  ┌──────────────┐ ┌──────────────┐                      │
│  │ SemanticGraph │ │ TemporalGraph │                      │
│  │ 语义相似度边   │ │ 时间先后边     │                      │
│  └──────────────┘ └──────────────┘                      │
│  ┌──────────────┐ ┌──────────────┐                      │
│  │ CausalGraph   │ │ EntityGraph   │                      │
│  │ 因果关系边     │ │ 实体共现边     │                      │
│  └──────────────┘ └──────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

**实现**:

```rust
/// 多图记忆存储
pub struct MultiGraphMemory {
    /// 语义图: 节点=MemoryItem, 边=embedding cosine similarity
    pub semantic: Graph<MemoryNodeId, SemanticEdge>,
    /// 时间图: 节点=MemoryItem, 边=temporal ordering
    pub temporal: Graph<MemoryNodeId, TemporalEdge>,
    /// 因果图: 节点=MemoryItem, 边=causal relationship
    pub causal: Graph<MemoryNodeId, CausalEdge>,
    /// 实体图: 节点=MemoryItem, 边=entity co-occurrence
    pub entity: Graph<MemoryNodeId, EntityEdge>,

    /// 向量索引 (语义检索)
    pub vector_index: VectorIndex,

    /// BM25 索引 (关键词检索)
    pub bm25_index: BM25Index,

    /// 实体索引 (实体匹配)
    pub entity_index: EntityIndex,
}

pub struct SemanticEdge {
    pub similarity: f64,  // cosine similarity
    pub strength: f64,    // Hebbian 强度
}

pub struct TemporalEdge {
    pub order: TemporalOrder, // Before/After/Concurrent
    pub delta_ms: i64,        // 时间差
}

pub struct CausalEdge {
    pub confidence: f64,  // 因果置信度
    pub mechanism: String, // 因果机制描述
}

pub struct EntityEdge {
    pub co_occurrence: u32,  // 共现次数
    pub context_window: u32, // 共现上下文窗口
}
```

### 2.2 双流记忆生命周期 (Dual-Stream Evolution)

借鉴 MAGMA 的双流设计和 GAM 的语义事件触发整合：

```
                    外部输入
                        │
                        ▼
         ┌──────────────────────────────┐
         │    快速路径 (Fast Path)        │
         │    Synaptic Ingestion         │
         │                              │
         │  1. 实体提取 + 去重            │
         │  2. 向量嵌入                  │
         │  3. 写入 EpisodicBuffer       │
         │  4. 初始关系边创建             │
         │                              │
         │  延迟: < 50ms                 │
         └──────────────┬───────────────┘
                        │
                        ▼ (异步)
         ┌──────────────────────────────┐
         │    慢速路径 (Slow Path)        │
         │    Structural Consolidation   │
         │                              │
         │  1. 语义事件检测 (bt=1?)       │
         │  2. 图谱结构整合              │
         │  3. 因果关系发现              │
         │  4. 实体链接解析              │
         │  5. 衰退/去重/合并            │
         │                              │
         │  延迟: 1-10s (异步)           │
         └──────────────────────────────┘
```

```rust
/// 双流记忆引擎
pub struct DualStreamMemory {
    /// 快速路径: 突触摄取
    fast_path: SynapticIngestion,
    /// 慢速路径: 结构整合
    slow_path: StructuralConsolidation,
    /// 语义事件检测器
    event_detector: SemanticEventDetector,
    /// 整合队列
    consolidation_queue: VecDeque<MemoryItem>,
}

/// 语义事件检测器 — 判断何时触发整合
pub struct SemanticEventDetector {
    /// 语义漂移阈值
    drift_threshold: f64,
    /// 主题切换检测
    topic_shift_detector: TopicShiftDetector,
    /// 因果链断裂检测
    causal_break_detector: CausalBreakDetector,
}
```

### 2.3 检索驱动记忆重组 (Retrieval-Driven Reconsolidation)

借鉴 REALM 的核心洞察：检索不是终点，而是记忆进化的驱动力。

```
查询 → 检索候选 → 生成回答 → 反馈信号
  │                              │
  │                              ▼
  │                    ┌─────────────────────┐
  │                    │  Reconsolidation     │
  │                    │                     │
  │                    │  1. 检索质量评估      │
  │                    │  2. 证据链完整性检查   │
  │                    │  3. 冲突检测         │
  │                    │  4. 图谱结构重组      │
  │                    │  5. 边强度更新        │
  │                    └─────────────────────┘
  │                              │
  └──────────────────────────────┘
         闭环: 检索质量 → 图谱进化
```

```rust
/// 检索驱动记忆重组引擎
pub struct ReconsolidationEngine {
    /// 图谱引用
    memory: MultiGraphMemory,
    /// 重组策略
    strategy: ReconsolidationStrategy,
    /// 重组历史
    history: Vec<ReconsolidationEvent>,
}

pub enum ReconsolidationStrategy {
    /// 边强化: 成功检索的路径加强
    StrengthenPath { factor: f64 },
    /// 结构发现: 检索中发现的新关系
    DiscoverStructure { min_confidence: f64 },
    /// 冲突消解: 检索中发现的矛盾
    ResolveConflict { policy: ConflictPolicy },
    /// 节点分裂: 一个节点分化为多个
    SplitNode { threshold: f64 },
    /// 节点合并: 多个相似节点合并
    MergeNodes { similarity_threshold: f64 },
}

/// 重组事件
pub struct ReconsolidationEvent {
    pub query: String,
    pub retrieved_ids: Vec<MemoryNodeId>,
    pub feedback: RetrievalFeedback,
    pub changes: Vec<GraphMutation>,
    pub timestamp: u64,
}
```

### 2.4 自演化图谱 (Self-Evolving Graph)

借鉴 EvoGraph-R1 (CVPR 2026) 的 MDP 框架，将图谱视为可交互环境：

```
                    Agent 交互
                        │
                        ▼
         ┌──────────────────────────────┐
         │    图谱 = 可交互环境           │
         │    MDP: (s, a, r, s')        │
         │                              │
         │  状态: 图谱当前结构            │
         │  动作:                       │
         │    GRAPH_RETRIEVE — 查询      │
         │    GRAPH_EXPAND   — 扩展      │
         │    GRAPH_REFINE   — 修正      │
         │    GRAPH_PRUNE    — 剪枝      │
         │    ANSWER         — 终止      │
         │  奖励: 回答质量 + 结构一致性   │
         │                              │
         └──────────────────────────────┘
```

```rust
/// 自演化图谱引擎
pub struct SelfEvolvingGraph {
    /// 图谱状态
    graph: MultiGraphMemory,
    /// 策略网络
    policy: EvolutionPolicy,
    /// 经验回放
    replay_buffer: ReplayBuffer,
    /// 演化统计
    stats: EvolutionStats,
}

pub enum GraphAction {
    /// 检索相关子图
    Retrieve { query: String, top_k: usize },
    /// 扩展图谱 (添加新实体/关系)
    Expand { items: Vec<MemoryItem> },
    /// 修正图谱 (更新错误关系)
    Refine { node_id: MemoryNodeId, correction: Correction },
    /// 剪枝 (移除低质量/过期节点)
    Prune { criteria: PruneCriteria },
    /// 终止 (当前查询已满足)
    Answer { confidence: f64 },
}

pub struct EvolutionPolicy {
    /// 状态编码器
    state_encoder: Box<dyn Fn(&MultiGraphMemory) -> GraphState>,
    /// 动作选择器
    action_selector: Box<dyn Fn(&GraphState) -> GraphAction>,
    /// 奖励函数
    reward_fn: Box<dyn Fn(&GraphAction, &AnswerQuality) -> f64>,
}
```

### 2.5 双时间线审计账本 (Bitemporal Ledger)

借鉴 ECHO (2026) 的双时间线设计：

```
┌─────────────────────────────────────────────────────┐
│              Bitemporal Ledger Entry                  │
├─────────────────────────────────────────────────────┤
│  content: MemoryContent                              │
│  valid_from: DateTime     // 有效时间 (信息何时为真)  │
│  valid_to: Option<DateTime>  // 有效截止 (被替代时)   │
│  transaction_time: DateTime  // 事务时间 (何时写入)   │
│  superseded_by: Option<EntryId>  // 被哪条替代       │
│  source: ProvenanceChain     // 来源链               │
│  confidence: f64             // 置信度               │
└─────────────────────────────────────────────────────┘
```

```rust
/// 双时间线账本
pub struct BitemporalLedger {
    /// 条目存储
    entries: HashMap<EntryId, LedgerEntry>,
    /// 当前有效视图 (valid_to = None 的条目)
    current_view: HashMap<String, EntryId>,
    /// 来源链
    provenance: ProvenanceChain,
}

pub struct LedgerEntry {
    pub id: EntryId,
    pub content: MemoryContent,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub transaction_time: DateTime<Utc>,
    pub superseded_by: Option<EntryId>,
    pub source: ProvenanceChain,
    pub confidence: f64,
}

impl BitemporalLedger {
    /// 查询当前有效条目
    pub fn current(&self, key: &str) -> Option<&LedgerEntry> {
        self.current_view.get(key).and_then(|id| self.entries.get(id))
    }

    /// 查询某时间点的有效条目
    pub fn at_time(&self, key: &str, time: DateTime<Utc>) -> Option<&LedgerEntry> {
        self.entries.values()
            .filter(|e| e.key() == key)
            .filter(|e| e.valid_from <= time && e.valid_to.map_or(true, |t| t > time))
            .max_by_key(|e| e.transaction_time)
    }

    /// 创建新版本 (追加，不覆盖)
    pub fn append_version(&mut self, key: &str, content: MemoryContent, source: ProvenanceChain) -> EntryId {
        let new_id = EntryId::new();
        let now = Utc::now();

        // 标记旧条目为已替代
        if let Some(old_id) = self.current_view.remove(key) {
            if let Some(old) = self.entries.get_mut(&old_id) {
                old.valid_to = Some(now);
                old.superseded_by = Some(new_id.clone());
            }
        }

        let entry = LedgerEntry {
            id: new_id.clone(),
            content,
            valid_from: now,
            valid_to: None,
            transaction_time: now,
            superseded_by: None,
            source,
            confidence: 1.0,
        };

        self.entries.insert(new_id.clone(), entry);
        self.current_view.insert(key.to_string(), new_id.clone());
        new_id
    }
}
```

### 2.6 增强型结晶引擎

基于 CraniMem 的门控机制和 SAGE 的自演化读写反馈：

```rust
/// V2 结晶引擎 — 多信号 + 跨域 + 衰退检测
pub struct V2CrystallizationEngine {
    /// 基础结晶器 (保留 V1 兼容)
    base: CrystallizationEngine,
    /// 多信号评分器
    scorer: MultiSignalScorer,
    /// 跨域迁移器
    transfer: CrossDomainTransfer,
    /// 衰退检测器
    staleness: StalenessDetector,
    /// 图谱关联器
    graph_linker: GraphLinker,
}

pub struct MultiSignalScorer {
    /// 语义相似度信号
    pub semantic_score: f64,
    /// 实体匹配信号
    pub entity_score: f64,
    /// 时间一致性信号
    pub temporal_score: f64,
    /// 因果一致性信号
    pub causal_score: f64,
    /// 使用频率信号
    pub frequency_score: f64,
    /// 跨会话持久性信号
    pub persistence_score: f64,
}

impl MultiSignalScorer {
    /// 综合评分 (加权融合)
    pub fn score(&self) -> f64 {
        self.semantic_score * 0.25
            + self.entity_score * 0.20
            + self.temporal_score * 0.15
            + self.causal_score * 0.20
            + self.frequency_score * 0.10
            + self.persistence_score * 0.10
    }
}

pub struct StalenessDetector {
    /// 衰退窗口大小
    pub window_size: usize,
    /// 衰退阈值
    pub degradation_threshold: f64,
    /// 重新蒸馏阈值
    pub re_distill_threshold: f64,
}

impl StalenessDetector {
    /// 检测技能是否衰退
    pub fn detect(&self, trend: &SkillEvolutionTrend) -> StalenessStatus {
        if trend.success_rate_trend.len() < self.window_size {
            return StalenessStatus::Stable;
        }

        let recent: Vec<f64> = trend.success_rate_trend.iter()
            .rev().take(self.window_size).cloned().collect();

        let avg_recent: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let avg_historical: f64 = trend.success_rate_trend.iter()
            .take(trend.success_rate_trend.len() - self.window_size)
            .sum::<f64>()
            / (trend.success_rate_trend.len() - self.window_size).max(1) as f64;

        if avg_recent < avg_historical * self.degradation_threshold {
            if avg_recent < self.re_distill_threshold {
                StalenessStatus::ReDistill
            } else {
                StalenessStatus::Degrading
            }
        } else {
            StalenessStatus::Stable
        }
    }
}

pub enum StalenessStatus {
    Stable,
    Degrading,
    ReDistill,
}
```

---

## 三、性能优化

### 3.1 查询性能提升

| 优化项 | V1 | V2 | 预期提升 |
|--------|----|----|---------|
| **检索路径** | 遍历全量 | 自适应路由 (语义/关键词/实体三通道) | 3-5x |
| **图遍历** | BFS | 门控拓扑遍历 (策略引导) | 2-3x |
| **记忆索引** | 单向量 | BM25 + Vector + Entity 三索引融合 | 2x |
| **缓存策略** | 无 | 查询级缓存 + 图结构缓存 | 4x |

### 3.2 存储结构优化

```
V1 存储: HashMap<Face, FaceState> (扁平)
V2 存储:

┌──────────────────────────────────────────────────────┐
│  层级存储 (Hierarchical Storage)                       │
│                                                      │
│  L0: EpisodicBuffer (容量: 1000, 内存)               │
│    └─ 快速读写, 未整合记忆                            │
│                                                      │
│  L1: MultiGraphMemory (容量: 100K, 内存+SSD)         │
│    └─ 已整合图谱, 支持多路检索                         │
│                                                      │
│  L2: BitemporalLedger (容量: 无限, SQLite/KB)         │
│    └─ 持久化, 双时间线审计                            │
│                                                      │
│  L3: ColdStorage (容量: 无限, 远程/归档)              │
│    └─ 衰退记忆, 低优先级                              │
└──────────────────────────────────────────────────────┘
```

### 3.3 并发与异步优化

```rust
/// 异步双流引擎
pub struct AsyncDualStream {
    /// 快速路径 (tokio spawn, 非阻塞)
    fast_sender: mpsc::Sender<MemoryItem>,
    /// 慢速路径 (后台任务)
    slow_handle: JoinHandle<()>,
    /// 语义事件触发器
    event_trigger: broadcast::Sender<SemanticEvent>,
}

impl AsyncDualStream {
    pub fn new(memory: MultiGraphMemory) -> Self {
        let (fast_sender, fast_receiver) = mpsc::channel(256);
        let (event_trigger, _) = broadcast::channel(64);

        let slow_handle = tokio::spawn(async move {
            let mut receiver = fast_receiver;
            let mut consolidation = StructuralConsolidation::new(memory);

            while let Some(item) = receiver.recv().await {
                consolidation.buffer(item).await;
                if consolidation.should_consolidate().await {
                    consolidation.consolidate().await;
                }
            }
        });

        Self { fast_sender, slow_handle, event_trigger }
    }
}
```

---

## 四、知识图谱增强

### 4.1 四图正交关系模型

```
         Semantic Graph          Temporal Graph
    (概念相似度/关联)           (时间因果/序列)
         ┌───┐                   ┌───┐
         │ A │──── sim ────│ B │  │ A │── before ──│ B │
         └─┬─┘             └───┘  └─┬─┘            └───┘
           │                         │
     related_to                  happened_after
           │                         │
         ┌─▼─┐                   ┌───▼┐
         │ C │                   │ C  │
         └───┘                   └────┘

         Causal Graph             Entity Graph
    (因果关系/推导)              (实体共现/参与)
         ┌───┐                   ┌───┐
         │ A │── causes ──│ B │  │ A │── co_occurs ──│ B │
         └─┬─┘             └───┘  └─┬─┘              └───┘
           │                         │
     enables                    contains
           │                         │
         ┌─▼─┐                   ┌───▼┐
         │ C │                   │ C  │
         └───┘                   └────┘
```

### 4.2 图谱操作 API

```rust
impl MultiGraphMemory {
    /// 添加记忆项 (自动更新四图)
    pub fn add(&mut self, item: MemoryItem) -> MemoryNodeId {
        let node_id = MemoryNodeId::new();

        // 1. 添加到所有图
        self.semantic.add_node(node_id.clone(), item.clone());
        self.temporal.add_node(node_id.clone(), item.clone());
        self.causal.add_node(node_id.clone(), item.clone());
        self.entity.add_node(node_id.clone(), item.clone());

        // 2. 建立语义边 (与现有节点)
        for existing in self.semantic.nodes() {
            let sim = cosine_similarity(&item.embedding, &existing.embedding);
            if sim > 0.7 {
                self.semantic.add_edge(&node_id, &existing.id, SemanticEdge {
                    similarity: sim,
                    strength: sim * 0.5, // 初始 Hebbian 强度
                });
            }
        }

        // 3. 建立时间边 (按时间戳排序)
        if let Some(prev) = self.temporal.latest_before(item.timestamp) {
            self.temporal.add_edge(&prev, &node_id, TemporalEdge {
                order: TemporalOrder::Before,
                delta_ms: item.timestamp - prev.timestamp,
            });
        }

        // 4. 提取实体并建立实体边
        let entities = extract_entities(&item.content);
        for entity in &entities {
            for co_occurring in self.entity.find_by_entity(entity) {
                self.entity.add_edge(&node_id, &co_occurring, EntityEdge {
                    co_occurrence: 1,
                    context_window: 100,
                });
            }
        }

        // 5. 更新向量索引
        self.vector_index.insert(&node_id, &item.embedding);

        // 6. 更新 BM25 索引
        self.bm25_index.insert(&node_id, &item.content);

        node_id
    }

    /// 自适应检索 (意图感知路由)
    pub fn adaptive_retrieve(&self, query: &QueryIntent) -> Vec<RetrievalResult> {
        match query.intent_type {
            IntentType::Factual => {
                // 事实查询: 实体图 + 语义图
                let entity_results = self.entity.retrieve(&query.entities, 10);
                let semantic_results = self.vector_search(&query.embedding, 10);
                self.fuse_results(entity_results, semantic_results, FusionStrategy::RRF)
            }
            IntentType::Temporal => {
                // 时间查询: 时间图 + 因果图
                let temporal_results = self.temporal.query_range(query.time_range);
                let causal_results = self.causal.trace_forward(&query.start_node);
                self.fuse_results(temporal_results, causal_results, FusionStrategy::Weighted)
            }
            IntentType::Causal => {
                // 因果查询: 因果图 + 语义图
                let causal_chain = self.causal.trace_chain(&query.cause_node);
                let semantic_context = self.vector_search(&query.embedding, 5);
                self.fuse_results(causal_chain, semantic_context, FusionStrategy::Append)
            }
            IntentType::Associative => {
                // 关联查询: 语义图 + 实体图
                let semantic_neighbors = self.semantic.neighbors(&query.start_node, 5);
                let entity_bridge = self.entity.bridge(&query.entities);
                self.fuse_results(semantic_neighbors, entity_bridge, FusionStrategy::MMR)
            }
        }
    }
}
```

### 4.3 与现有知识系统的集成

```
                    现有系统
        ┌──────────────────────────────┐
        │  KB (SQLite)                  │
        │  ├── experience (379)         │
        │  ├── kv_store (9.2K)          │
        │  ├── nodes (390K)             │
        │  ├── edges (792K)             │
        │  └── embeddings (235K)        │
        └──────────────┬───────────────┘
                       │
                       │ 双向同步
                       ▼
        ┌──────────────────────────────┐
        │  Crystal Core V2              │
        │  ├── MultiGraphMemory         │
        │  ├── BitemporalLedger         │
        │  ├── ReconsolidationEngine    │
        │  └── SelfEvolvingGraph        │
        └──────────────────────────────┘
                       │
                       │ 提供
                       ▼
        ┌──────────────────────────────┐
        │  GWT 注意力路由               │
        │  SEAL 管线                    │
        │  ConsciousnessTree           │
        │  CTM 10 步循环                │
        └──────────────────────────────┘
```

---

## 五、自我演化机制

### 5.1 CTM 10 步循环 (CRP Cycle)

V2 的 CTM 循环实现 V2 设计文档中描述的完整 10 步：

```rust
impl CrystalConsciousnessV2 {
    /// 核心循环 — 完整 CRP 10 步
    pub async fn tick(&mut self, input: SensoryInput) -> CrystalOutput {
        // Phase 1: 感知 (Steps 1-4)
        let context = self.prepare_context(&input).await;

        // Phase 2: 竞争 (Steps 5-6)
        let chunk = self.compete_or_override(&context).await;

        // Phase 3: 扩散 (Steps 7-9)
        let responses = self.broadcast_to_arms(&chunk).await;
        let output = self.gate_and_select(responses).await;

        // Phase 4: 记忆 (Step 10)
        self.form_memory(&output).await;

        // V2 新增: 检索驱动重组
        self.reconsolidate(&output).await;

        output
    }

    /// Step 10 + 检索驱动重组
    async fn reconsolidate(&mut self, output: &CrystalOutput) {
        // 1. 评估检索质量
        let quality = self.assess_retrieval_quality(output);

        // 2. 如果质量低于阈值，触发重组
        if quality.score < self.reconsolidation_threshold {
            let event = self.reconsolidation_engine.reconsolidate(
                &output.query,
                &output.retrieved_ids,
                &quality,
            ).await;

            // 3. 记录重组事件到审计账本
            self.ledger.record_reconsolidation(event);
        }

        // 4. 更新图谱结构
        self.graph.evolve_from_feedback(&quality);
    }
}
```

### 5.2 自演化反馈闭环

```
              ┌─────────────────────────────────┐
              │         自演化闭环                │
              │                                 │
              │  写入 ←──── 新经验               │
              │    │                             │
              │    ▼                             │
              │  图谱更新 (MultiGraphMemory)     │
              │    │                             │
              │    ▼                             │
              │  检索 (AdaptiveRetrieve)         │
              │    │                             │
              │    ▼                             │
              │  回答生成                        │
              │    │                             │
              │    ▼                             │
              │  质量评估 (RewardSignal)          │
              │    │                             │
              │    ▼                             │
              │  ┌─────────────────────┐         │
              │  │ 写入反馈            │         │
              │  │ - 边强化/弱化       │         │
              │  │ - 节点分裂/合并     │         │
              │  │ - 新关系发现        │         │
              │  └─────────────────────┘         │
              │    │                             │
              │    └─────→ 回到图谱更新 ────────┘
              └─────────────────────────────────┘
```

### 5.3 进化操作类型

```rust
pub enum EvolutionOperation {
    /// Bud (萌芽): 新增节点
    Bud { item: MemoryItem },
    /// Graft (嫁接): 跨域关联
    Graft { source: MemoryNodeId, target_domain: Domain },
    /// Prune (修剪): 移除低质量
    Prune { criteria: PruneCriteria },
    /// CrossPollinate (异花授粉): 跨域知识迁移
    CrossPollinate { skill_id: String, target_domain: Domain },
    /// Strengthen (强化): 增强关联
    Strengthen { edge: EdgeId, factor: f64 },
    /// Weaken (弱化): 减弱关联
    Weaken { edge: EdgeId, factor: f64 },
    /// Consolidate (整合): 合并相似节点
    Consolidate { node_ids: Vec<MemoryNodeId> },
    /// Fragment (分裂): 分化节点
    Fragment { node_id: MemoryNodeId, into: Vec<MemoryItem> },
}
```

---

## 六、与 V1 的兼容性

### 6.1 渐进迁移路径

| Phase | 范围 | 风险 |
|-------|------|------|
| **Phase 1** | 扩展 CrystalState，添加 MultiGraphMemory 字段 | 低 — 向后兼容 |
| **Phase 2** | 实现 DualStreamMemory，替换 SimpleMemory | 中 — 接口变更 |
| **Phase 3** | 集成 BitemporalLedger，替换直接存储 | 中 — 持久化变更 |
| **Phase 4** | 启用 ReconsolidationEngine，接入检索反馈 | 高 — 闭环行为变更 |
| **Phase 5** | 启用 SelfEvolvingGraph，MDP 策略训练 | 高 — 新能力 |

### 6.2 V1 → V2 类型映射

| V1 类型 | V2 替代 | 兼容策略 |
|---------|---------|---------|
| `CrystalState` | `CrystalStateV2` | 新增 `multi_graph: MultiGraphMemory` 字段 |
| `FaceState` | `NodeState` | 扩展为含 graph_node_id 的结构 |
| `DefaultCrystalCycle` | `CrystalCycleV2` | 实现相同 trait，增加 async 方法 |
| `CrystalMetrics` | `CrystalMetricsV2` | 扩展为含趋势追踪的结构 |
| `CrystallizationEngine` | `V2CrystallizationEngine` | 包装 V1 引擎，添加新能力 |

---

## 七、验证标准

### 7.1 编译验证

```bash
cargo check --all-targets -p neotrix
cargo clippy -- -D unsafe_code
cargo deny ban  # 循环依赖检测
```

### 7.2 测试验证

```bash
# 多图记忆测试
cargo test -p neotrix --lib multi_graph_memory

# 双流生命周期测试
cargo test -p neotrix --lib dual_stream

# 检索驱动重组测试
cargo test -p neotrix --lib reconsolidation

# 自演化图谱测试
cargo test -p neotrix --lib self_evolving_graph

# 双时间线账本测试
cargo test -p neotrix --lib bitemporal_ledger

# V2 结晶引擎测试
cargo test -p neotrix --lib v2_crystallization

# 兼容性测试 (V1 行为保持)
cargo test -p neotrix --lib crystal_v1_compat
```

### 7.3 性能验证

| 指标 | V1 基线 | V2 目标 |
|------|---------|---------|
| 单次检索延迟 | ~10ms | < 5ms (索引优化) |
| 图谱插入延迟 | N/A | < 50ms (快速路径) |
| 整合延迟 | N/A | < 10s (异步) |
| 内存占用 | ~1MB | < 10MB (含图谱) |
| 检索准确率 | 基线 | +20% (多信号融合) |

---

## 八、实施计划

### Phase 1: 多图记忆 (2 周)

- [ ] 实现 `MultiGraphMemory` 核心结构
- [ ] 实现四图正交关系 (Semantic/Temporal/Causal/Entity)
- [ ] 实现向量索引 + BM25 索引 + 实体索引
- [ ] 单元测试覆盖

### Phase 2: 双流生命周期 (1 周)

- [ ] 实现 `DualStreamMemory`
- [ ] 实现 `SynapticIngestion` (快速路径)
- [ ] 实现 `StructuralConsolidation` (慢速路径)
- [ ] 实现语义事件检测器

### Phase 3: 检索驱动重组 (2 周)

- [ ] 实现 `ReconsolidationEngine`
- [ ] 实现 5 种重组策略
- [ ] 实现检索质量评估
- [ ] 集成到 CTM 循环

### Phase 4: 自演化图谱 (2 周)

- [ ] 实现 `SelfEvolvingGraph`
- [ ] 实现 MDP 状态/动作/奖励
- [ ] 实现进化策略
- [ ] 实现经验回放

### Phase 5: 审计与结晶 (1 周)

- [ ] 实现 `BitemporalLedger`
- [ ] 实现来源链追踪
- [ ] 升级结晶引擎
- [ ] 集成衰退检测

### Phase 6: 集成与验证 (1 周)

- [ ] 与 GWT 注意力路由集成
- [ ] 与 SEAL 管线集成
- [ ] 与 ConsciousnessTree 集成
- [ ] 全量测试 + 性能基准

---

## 九、参考文献

| 来源 | 核心贡献 | NeoTrix 映射 |
|------|---------|-------------|
| MAGMA (ACL 2026) | 四图正交记忆 + 自适应遍历 | MultiGraphMemory |
| DCPM (arXiv 2026) | 双过程认知记忆 + Schema 归纳 | DualStreamMemory |
| REALM (arXiv 2026) | 检索驱动记忆重组 | ReconsolidationEngine |
| ECHO (arXiv 2026) | 双时间线审计 + 来源闭合 | BitemporalLedger |
| CraniMem (arXiv 2026) | 门控多阶段记忆 | V2 Crystallization |
| GAM (ACL 2026) | 语义事件触发整合 | EventTriggeredConsolidation |
| SAGE (arXiv 2026) | 自演化图谱 + GFM 读取器 | SelfEvolvingGraph |
| CoEvo-Mem (arXiv 2026) | 检索-记忆共演化 | Reconsolidation Feedback |
| EvoGraph-R1 (CVPR 2026) | MDP 图谱演化 | GraphAction MDP |
| Mem0 (2026) | 生产级记忆基准 | 性能参考 |

---

> **版本**: v2.0-draft
> **状态**: 设计文档
> **下一步**: Phase 1 — 实现 MultiGraphMemory 核心结构
