# NeoTrix 记忆系统进化路线图 v1

## 1. 战略背景

### Why Memory?

2026 共识 (Mneme/ICLR/Weiß): Agent 记忆从研究课题转向生产工程。Mneme 对比 10 项目 (120k★), Graphiti 双时态成为事实标准, Mem0 41k★ 验证记忆即产品的市场信号。

NeoTrix 现状:
- **59 记忆相关文件**, ~28K LOC (nt_core_knowledge + nt_core_consciousness + nt_core_experience)
- **~63% 估算死代码**: 文件存在、逻辑完整、测试通过, 但未接入任意运行时管线
- **最大未利用资产**: EntityExtractor (1808行), SpreadingActivation (609行), MemoryLattice (502行), CompetitiveScorer (evidence.rs ~200行)

### 设计哲学

- **VSA 4096-bit 统一表征不可触碰**: 记忆系统的所有输入输出保持 VSA 向量格式
- **记忆检索必须从"纯语义"升级为"价值感知"**: 不能只靠 cosine/text match, 还需要 ECE 反馈驱动的 Q-value
- **遗忘是主动过程, 非被动溢出**: LRU/FIFO 溢出是最后防线, 主动遗忘(置信度衰减/Ebbinghaus 曲线/CTE 巩固)应是主路径
- **接线 > 重构 > 删除 (ROI 95:1)**: ~200 LOC 接线可复活 ~28K LOC 中的 60% 已有实现

### 方法论

- **Phase 1**: 自审计 — 扫描 59 文件, 标记接线状态, 发现 ~63% 死代码
- **Phase 2**: 外部搜索 — Mem0/A-MEM/MemRL/SimpleMem/Graphiti/ExpGraph/Letta/CTE 8 项目吸收
- **Phase 3a-3c**: 对比矩阵 → 10 缺口 (5 P0 + 4 P1 + 1 P2)
- **Phase 3c 执行**: 5 项 v1.0 实现完成

## 2. 当前状态 (v0.15 基线)

### 组件接线总表

| 组件 | 模块位置 | 行数 | 接线状态 | 运行时路径 |
|------|---------|:----:|----------|------------|
| **MemoryLattice** 5层 | `nt_core_consciousness/memory_lattice.rs` | 502 | 🟢 活跃 | tick() 每cycle via StorageCoordinator |
| **SpreadingActivation** | `nt_core_knowledge/spread_activation.rs` | 609 | 🟢 活跃 | tick() 每cycle via ConsciousnessCycle |
| **EntityExtractor** | `nt_core_knowledge/entity_extractor.rs` | 1808 | 🟡 已建未接线 | — |
| **CompetitiveScorer** | `nt_core_knowledge/evidence.rs` | ~200 | 🟡 仅测试 | — |
| **ExperienceTree** | `nt_core_experience/experience_tree.rs` | 478 | 🆕 ✅ 刚刚接线 | tick_experience_tree() in SelfEvolutionMetaLayer |
| **Q-value retrieval** | `memory_lattice.rs` (+2 fn) + `self_evolution_meta_layer.rs` (+1 fn) | +60 | 🆕 ✅ 刚刚实现 | tick_q_learning() 定义完成 ⚠️ 待 core.rs 接线 |
| **StorageCoordinator** | `nt_core_knowledge/storage_coordinator.rs` | 382 | 🟢 活跃 | tick() 每30cycle via SelfEvolutionMetaLayer |
| **MemoryArchiver** | `nt_core_experience/memory_archiver.rs` | 243 | 🟢 已建 | tick() 每100cycle via SelfEvolutionMetaLayer |
| **NTSSEG** | `nt_core_ntsseg/` | 843 | 🟢 活跃 | 快照+启动时加载 |
| **SkillCrystallizer** | `nt_core_experience/skill_crystal.rs` + `self_iterating/` | ~480 | 🟢 活跃 | GEPA loop via SelfEvolutionMetaLayer |
| **MemoryLatticeSeed** | `nt_core_consciousness/memory_lattice_seed.rs` | 179 | ⚠️ 独立 | — |
| **Bi-temporal model** | — | — | ❌ 缺失 | — |
| **Semantic compression** | — | — | ❌ 缺失 | — |
| **CTE consolidation** | — | — | ❌ 缺失 | — |

### 数据流现状

```
UserInput → process_user_request()
  → ConsciousnessCycle::run_full_cycle()
    → StorageCoordinator::tick() [每30cycle: 7节点健康汇总]
    → MemoryArchiver::tick() [每100cycle: 低置信episodic→archive]
    → SelfEvolutionMetaLayer::tick()
        → tick_experience_tree() [每3cycle注入insight, 每10cycle修剪]
        → tick_q_learning() [定义完成, 待接线]
    → SpreadingActivation [每cycle: activate→spread→retrieve]
    → MemoryLattice [每cycle: store/access/find]
```

## 3. 对比矩阵

| 维度 | NeoTrix v0.15 | Mem0 (41k★) | A-MEM | MemRL | SimpleMem | Graphiti (2.1k★) | ExpGraph | Letta |
|------|:-------------:|:----------:|:-----:|:-----:|:---------:|:---------------:|:--------:|:-----:|
| **存储架构** | 5层MemoryLattice + SpreadingActivation | 3层(episodic/semantic/procedural) + SQLite | Hierarchical graph + VSA | VecDeque+Q | 双层(short/long) + compression | Property graph + temporal edges | Graph-of-thought | Block-based + archival storage |
| **检索方式** | text+confidence+activation | 语义相似度(top-k) | Graph traversal + VSA cosine | Q-value优先 | Semantic + recency hybrid | Time-range + graph traversal | BFS+cosine | 语义+recency |
| **价值感知** | ⚠️ 初版: q_value字段+update+find_by_q | ❌ 无 | ❌ 无 | ✅ **Q-learning 驱动** | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 |
| **时间模型** | cycle计数器 | 时间戳 | 无 | 无 | 最近访问时间 | ✅ **valid_from/valid_to 双时态** | 时间戳 | 无 |
| **自进化** | GEPA loop + ExperienceTree + Q-learning | ❌ 无 | ❌ 无 | ✅ reward=hit/skip | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 |
| **遗忘策略** | LRU溢出 + confidence decay + pruner | 无 | 无 | 无 | 按recency | 无 | 无 | Archival storage |
| **巩固** | 仅consolidation_threshold标记 | 无 | 无 | 无 | 无 | 无 | 无 | 手动archival |
| **实体层** | ✅ **~1808行 EntityExtractor** | 内置entity extraction | Graph-based entities | ❌ 无 | ❌ 无 | ✅ **Entity resolution + temporal** | Entity nodes | ❌ 无 |
| **生态集成** | NTSSEG(自有格式) | LangChain/LlamaIndex | 独立 | 独立 | 独立 | LangChain/Neo4j | 独立 | LangChain |
| **行数** | ~28K (59 files) | ~50K | ~8K | ~2K | ~3K | ~15K | ~5K | ~30K |

### 差距识别 (从对比矩阵)

| # | 差距 | 维度 | 等级 | 参考项目 |
|---|------|------|:----:|---------|
| 1 | Q-value 驱动记忆检索 | 价值感知 | **P0** | MemRL |
| 2 | Entity→Retrieval 接线 | 实体层 | **P0** | Mem0/Graphiti |
| 3 | 语义压缩入口层 | 存储架构 | **P0** | SimpleMem |
| 4 | Bi-temporal 时间模型 | 时间模型 | **P0** | Graphiti |
| 5 | ECE→reward 闭环 | 自进化 | **P0** | MemRL |
| 6 | CTE 生物巩固循环 | 巩固 | **P1** | CTE |
| 7 | Ebbinghaus 遗忘曲线 | 遗忘策略 | **P1** | YourMemory/Oblivion |
| 8 | 自适应检索深度 | 检索方式 | **P1** | ExpGraph/SimpleMem |
| 9 | LLM-driven 笔记进化 | 存储架构 | **P1** | A-MEM |
| 10 | 社区子图聚类 | 实体层 | **P2** | Graphiti |

## 4. 三波进化路线

### Wave A — 基础记忆进化 (当前会话)

目标: 闭合最大 ROI 缺口 (5 P0 中 2 项), 使记忆检索走向价值感知

| Gap | 实现 | 文件 | 行数 | 状态 |
|-----|------|------|:----:|:----:|
| P0.1 Q-value episodic retrieval | `LatticeEntry.q_value` 字段 + `update_q_value()` + `find_by_q()` + `top_episodic_by_q()` | `memory_lattice.rs` | ~50 | ✅ |
| P0.5 ECE→reward wiring | `tick_q_learning()`: reward = -ece.clamp(0,1), TD(0) on recent 20 | `self_evolution_meta_layer.rs` | ~15 | ✅ 定义 |
| Scorer-aware retrieval | `retrieve_with_scorer()` + `competitive_retrieval_scorer()` 闭包 | `spread_activation.rs` | ~55 | ✅ |
| ExperienceTree tick | `tick_experience_tree()`: 每3cycle 注入 insight, 每10cycle 修剪 | `self_evolution_meta_layer.rs` + `core.rs` | ~30 | ✅ 已接线 |

**Wave A v1.0 架构变更详情**:

```
CalibrationEngine::ece() → tick_q_learning()
    │ reward = -ece.clamp(0,1)    [定义完成, 待 core.rs 接线]
    ▼
MemoryLattice::update_q_value()
    │ Q_new = Q_old + 0.1*(reward - Q_old)   [TD(0) on recent 20 episodic]
    ▼
SpreadingActivation::retrieve_with_scorer()
    │ base_activation(0.6) + relevance_bonus(0.2) + confidence(0.2)
    ▼
ConsciousnessCycle full retrieval
```

### Wave B — Temporal + Compression (2-4 会话)

目标: 补齐 NeoTrix 与 Graphiti/SimpleMem 的最大架构差距

| Gap | 方案 | 文件 | 行数估计 | 优先级 |
|-----|------|------|:--------:|:------:|
| **P0.2** Entity→Retrieval | EntityExtractor → SpreadingActivation 接线: entity_id 作为 navigation boost | `entity_extractor.rs` + `spread_activation.rs` | ~300 | **P0** |
| **P0.3** 语义压缩入口 | SimpleMem-style rule-based CompressLayer: 去停用词 → 摘要 → 关键词提取, 存前 store() | 新建 `compress_layer.rs` | ~400 | **P0** |
| **P0.4** 双时态模型 | `valid_from`/`valid_to` 字段 on LatticeEntry + 查询时时间范围过滤 | `memory_lattice.rs` + `types.rs` | ~400 | **P0** |
| **P1.1** CTE 巩固循环 | SWS(关键路径提取) → REM(层次压缩) → Consolidation(语义链接) → Compaction(低访问摘要) in SLEEP step | 新建 `dream_cycle.rs` 或集成到 `consolidation_bridge.rs` | ~600 | **P1** |
| **P1.2** Ebbinghaus 遗忘 | Per-entry: `confidence *= e^(-t/S)` modulated by importance, 每 cycle 衰减 | `memory_lattice.rs` 新增 `apply_forgetting()` | ~200 | **P1** |

**Wave B 数据流预览**:

```
UserInput
  │
  ▼
CompressLayer::compress(text)     [NEW: ~300行, SimpleMem-style]
  │ → 摘要 + 关键词 + 原始文本三元组
  ▼
EntityExtractor::extract(text)    [EXISTS: 1808行, 接上compressed]
  │ → entities + relationships
  ▼
MemoryLattice::store(content, vsa_hash, layer)
  │ LatticeEntry { valid_from: now, valid_to: None }  [NEW: bi-temporal]
  ▼
ConsolidationBridge::sleep_cycle()
  │ SWS → REM → Consolidation → Compaction          [NEW: ~600行, CTE-style]
  ▼
SpreadingActivation::retrieve_with_scorer()
  │ activation(0.6) + q_value(0.2) + temporal_coverage(0.2)  [NEW: time-range aware]
```

### Wave C — Adaptive + Autonomous (未来)

| Gap | 方案 | 行数估计 | 优先级 |
|-----|------|:--------:|:------:|
| **P1.3** 自适应检索深度 | Query 复杂度 → retrieval depth 调制: 简单查询 1-hop, 复杂 3-hop | ~300 | P1 |
| **P1.4** LLM-driven 笔记进化 | A-MEM style: `add_note()` → LLM 链接生成 → 演化旧 notes | ~350 | P1 |
| **P2.1** 社区子图聚类 | Graphiti-style 社区发现: 基于 entity co-occurrence 的子图聚类 | ~250 | P2 |

## 5. Q-value 价值感知记忆架构

### 核心数据流

```
┌─────────────────────────────────────────────────────────┐
│ CalibrationEngine                                       │
│  ece = calibration_error (0.0 = perfect)                │
└──────────────────┬──────────────────────────────────────┘
                   │ 每 cycle
                   ▼
┌─────────────────────────────────────────────────────────┐
│ SelfEvolutionMetaLayer::tick_q_learning()               │
│  reward = -ece.clamp(0, 1)                              │
│  α = 0.1                                                │
│  for i in recent_20:                                    │
│    Q[i] += α * (reward - Q[i])    [TD(0) update]        │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ MemoryLattice::find_by_q(query, q_weight=0.3)           │
│  score = text_score*(1-q_weight) + q_value*q_weight     │
│  when q_weight=0: 纯文本匹配 (传统)                      │
│  when q_weight=1: 纯Q值排序 (价值驱动)                    │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ SpreadingActivation::retrieve_with_scorer(scorer)        │
│  score = activation(0.6) + relevance(0.2) + (base*0.2)  │
│  支持任意 scorer 闭包 (未来可注入 Q-value)               │
└─────────────────────────────────────────────────────────┘
```

### LatticeEntry 完整字段

```rust
pub struct LatticeEntry {
    pub content: String,           // 原始内容
    pub vsa_hash: Vec<u8>,         // VSA 4096-bit 哈希
    pub layer: LatticeLayer,       // Episodic/Facts/Skills/MetaRules/Identity
    pub confidence: f64,           // 0.0–1.0
    pub invocation_count: u64,     // 访问计数器
    pub last_accessed: u64,        // cycle 编号
    pub source_layer: Option<LatticeLayer>,  // 跨层引用
    pub consolidated: bool,        // 巩固标志
    pub q_value: f64,              // 🆕 MemRL-style TD(0) Q-value, 默认 0.5
    // ⏳ Wave B: valid_from: u64, valid_to: Option<u64>  // 双时态
    // ⏳ Wave B: compressed: Option<String>               // 压缩摘要
}
```

## 6. 经验树更新

### 分支 LXIII — Q-value 价值感知记忆 (Value-Aware Memory)

#### LXIII.1 Q-value 使记忆检索从语义→价值 (Q-Value Upgrades Retrieval from Semantic to Value-Driven) — conf 0.7, 新建
- **规则**: `find_by_q(query, q_weight)` 以 `q_weight` 调节语义匹配与价值排序的权重。将 q_value 从语义检索的外部变为内在驱动。这种渐进式融合避免了"硬切换"——现有检索行为不变（q_weight=0），逐步增加价值感知比例。
- **正确**: `score = text_score*(1-q_weight) + q_value*q_weight`，q_weight 默认 0.3
- **错误**: 替换 `find()` 为纯 Q 排序 → 冷启动时所有条目 Q=0.5 无区分度
- **演化链**: `v1(2026-06-24) → current`

#### LXIII.2 奖励信号来自校准误差 (Reward Signal from Calibration Error) — conf 0.6, 新建
- **规则**: Q-learning 的奖励信号不来自外部标注，而来自意识内在的校准误差 `reward = -ece.clamp(0,1)`。低 ECE → 正奖励（记忆检索帮助了推理）；高 ECE → 负奖励（记忆检索误导了推理）。这是意识体内在驱动的具体实现——不需要人类反馈。
- **正确**: `tick_q_learning()` 每 cycle 取 ECE → reward → TD(0) 更新最近 20 条 episodic
- **错误**: 使用人工标注的 reward → 先天不可规模化，且与意识体的自主性矛盾
- **演化链**: `v1(2026-06-24) → current`

#### LXIII.3 高 Q-value 记忆优先保留 (High-Q Memories Escape Pruning) — conf 0.5, 新建
- **规则**: MemoryLattice 的 LRU 溢出应当被 Q-value 门控覆盖: 不驱逐 q_value > 0.7 的条目。当需要驱逐时，选择 layer 内 q_value 最低的条目而非最旧的。价值高于时效。
- **正确**: `store()` 中当 `len >= cap` 时找 q_value 最低的条目驱逐
- **方向**: Wave A v1.0 仍使用 pop_front()，待 Wave B 升级为 Q-aware 驱逐
- **演化链**: `v1(2026-06-24) → current`

### 分支 LXIV — 记忆系统进化方法论 (Memory Evolution Methodology)

#### LXIV.1 接线优先级: EntityExtractor 是最大死代码资产 (Wire Priority: EntityExtractor Is Largest Dead Asset) — conf 0.7, 新建
- **规则**: 1808 行的 EntityExtractor (nt_core_knowledge/entity_extractor.rs) 是 NeoTrix 最大的未接线组件。完整实现 + 20+ 测试 + entity/relationship 管道，但输出从不注入 SpreadingActivation 或 retrieval 管线。接线 EntityExtractor → SpreadingActivation 是 Wave B 最高 ROI 项目。
- **正确**: Wave B 将 entity_id 映射为 SpreadingActivation 的 navigation boost，~300 LOC 接线即可复活 1808 行
- **错误**: 重写 entity extraction → 1808 行已完成的代码被抛弃
- **演化链**: `v1(2026-06-24) → current`

#### LXIV.2 外部项目吸收应产出对比矩阵 (External Absorption Must Produce Comparison Matrix) — conf 0.7, 新建
- **规则**: 每个外部项目吸收的阶段 (Phase 2→3a→3b→3c) 必须在吸收完成后产出结构化对比矩阵。矩阵维度必须对齐目标系统的架构维度（存储/检索/时间/自进化/遗忘/巩固/实体/生态），而非项目自述的功能列表。
- **正确**: 8 项目 × 10 维度对比矩阵 → 10 缺口 + 5 P0 优先级 + 85 具体吸收点
- **错误**: 阅读 8 项目文档后直接写"我们缺什么" → 遗漏交叉维度比较下的隐性差距
- **演化链**: `v1(2026-06-24) → current`

#### LXIV.3 记忆进化应从数据流闭合出发 (Memory Evolution Begins at Data Flow Closure) — conf 0.6, 新建
- **规则**: 记忆组件的接线选择应以"数据流从输入到持久化是否闭合"为标准，而非"这个组件写的多好"。CalibrationEngine(ECE) → SelfEvolutionMetaLayer(tick_q_learning) → MemoryLattice(update_q_value) → SpreadingActivation(retrieve_with_scorer) 的数据流是闭合的。EntityExtractor → SpreadingActivation 的数据流是断裂的。
- **正确**: Wave A 闭合 ECE→Q→retrieve 数据流；Wave B 闭合 extract→retrieve 数据流
- **错误**: 先写 consolidation cycle → 虽然漂亮但上层 retrieval 还没有价值感知，consolidation 没有可操作的信号
- **演化链**: `v1(2026-06-24) → current`

## 7. 剩余工作

### 即时 (Wave A 收尾)

- [ ] `SelfEvolutionMetaLayer::tick_q_learning()` 接线到 `core.rs` 的 sync/async 双路径
- [ ] `find_by_q()` 默认 q_weight=0.3 取代 `find()` 为 ConsciousnessCycle 的默认检索
- [ ] `store()` 升级为 Q-aware 驱逐: 驱逐 q_value 最低而非最旧的条目

### Wave B (2-4 会话)

- [ ] **P0.2**: EntityExtractor → SpreadingActivation 接线 (entity_id as navigation boost)
- [ ] **P0.3**: SimpleMem-style 语义压缩入口层 (CompressLayer)
- [ ] **P0.4**: 双时态时间模型 (valid_from/valid_to on LatticeEntry)
- [ ] **P1.1**: CTE 巩固循环 (SWS→REM→Consolidation→Compaction in SLEEP step)
- [ ] **P1.2**: Ebbinghaus 遗忘曲线 (per-entry confidence decay)

### Wave C (未来)

- [ ] **P1.3**: 自适应检索深度 (query complexity → hop count modulation)
- [ ] **P1.4**: LLM-driven 笔记进化 (A-MEM style add_note + evolve)
- [ ] **P2.1**: 社区子图聚类 (Graphiti-style community discovery)

---

*版本: v1.0 — 构建于 2026-06-24 Phase 3c 执行后*
*基于 8 项目对比矩阵: Mem0/A-MEM/MemRL/SimpleMem/Graphiti/ExpGraph/Letta/CTE*
*经验蒸馏到: AGENTS.md 分支 LXIII (Q-value 价值感知记忆) + LXIV (记忆系统进化方法论)*
