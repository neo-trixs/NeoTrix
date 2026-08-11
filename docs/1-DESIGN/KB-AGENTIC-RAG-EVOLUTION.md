# NT-KB Agentic RAG 进化设计 — 突破检索瓶颈

> 输入: LearnGraph.online Module-13 "RAG 演进历程" (Naive→Advanced→Modular→Graph→Agentic 五代)
> 结合 NeoTrix 自有资产 (E8/GWT/VSA HyperCube/ConsciousnessTree/SEAL) 设计
> 状态: **C1+C4 DONE (2026-08-11)** — 意图路由 + 反馈闭环 + E8 状态机 Agent 循环 + VSA 扩召已落地, 测试全绿
> C1 实现: `nt_memory_gwt_router.rs` (10 tests) + `nt_memory_feedback.rs` (7 tests) + KB 字段接线
> C4 实现: `nt_memory_e8_agent.rs` (12 tests) + `nt_memory_vsa_expand.rs` (7 tests) + T3 生产接线
> C4 T3 接线: `search()` 入口 GWT 路由决策 + VSA 扩召增强 (B1/B2 闭合);
>   `search_agentic()` 驱动 E8 状态机 + SEAL 反馈回流 (B8 闭合); `build_vsa_vocabulary()` 建词典

---

## 1. 现状盘点：已有能力矩阵

NeoTrix KB (`neotrix-core/src/neotrix/l3_memory_impl/nt_memory_kb/`) 已覆盖 RAG 前四代：

| RAG 代际 | 对应模块 | 文件:行号证据 |
|---------|---------|--------------|
| Naive (BM25/FTS) | `search_fts` + `Bm25Index` | `nt_memory_search.rs:9`, `bm25.rs` |
| Advanced (Dense) | `semantic_search` (cosine) + `pq_search` (ANN) | `mod.rs:1232,1282` |
| Modular (混合+重排) | `hybrid_search` RRF 融合 3 ranklist (FTS/BM25/Walsh) | `nt_memory_search.rs:221` |
| Graph (知识图谱) | `GraphRagStore` (实体/关系/社区/全局摘要) | `nt_memory_graphrag.rs` (3186 行) |
| Agentic (部分) | `AdaptiveRetrieval` (复杂度分类+CRAG 分级+迭代改写) | `nt_memory_adaptive_rag.rs:72,137,195` |

**额外独有资产**：SVAF 写入门禁、置信度存储、冲突 supersede、社区检测、隐私门禁、Walsh 正交通道、类型化块分块、tech-reserve。

## 2. 瓶颈诊断 (8 大瓶颈，证据优先)

### B1. 检索入口无意图路由 — search() 是"万能钥匙"
`mod.rs:1071` 的 `search()` 三级降级 (PQ→semantic→hybrid) 只看 **是否配置了 embedding**，
不区分查询意图。"比较 A 与 B" 和 "RAG 是什么" 走同一条路径 → 简单查询过重、复杂查询不足。

### B2. AdaptiveRAG 是"伪 Agentic"
`nt_memory_adaptive_rag.rs` 四大环节全部是启发式，非语义判断：
- `classify_query:72` — 大写单词计数 (count_entities)，中英文混合查询直接失效
- `grade_documents:137` — `term_match_ratio` 词法重叠率，非 LLM 相关性打分
- `rewrite_query:301` — 去停用词+去误导词，非语义改写
- 循环体 (`execute_pipeline:195`) 无规划/反思/工具层，只有 while 拼字符串

### B3. Graph 信号未融入主检索
`GraphRagStore` (3186 行) 与主检索解耦。`search()` 只通过 `entity_graph_scores`
(`nt_memory_search.rs:503`) 用 1-hop PPR 打分；**社区检索、全局摘要、子图扩展、图路径推理
均未接入统一入口**。GraphRAG 能力闲置率极高。

### B4. 无反馈闭环 — 检索不学习
`fuse_weights [0.25,0.15,0.40,0.20]` (`nt_memory_adaptive_rag.rs:46`) 与
`rerank_weight [0.3,0.7]` 是**硬编码常量，从不依据实际效果调整**。
检索结果是否被采纳、生成质量如何 → 无信号回写。违反 SEAL 自进化哲学。

### B5. 多跳检索是"伪多跳"
`iterative_retrieval:351` 的下一跳 = `format!("{} {}", current_query, query)` 拼接原查询，
不基于上一跳**实体/答案**定向扩展。无法做图路径推理式多跳。

### B6. 无查询分解
Hard 复杂度查询 (`entity_count>=4`) 只走拼接重试，不做 map-reduce 分解。
复杂分析题 (对比/综述/多条件) 命中率低。

### B7. 无真正的 Rerank 层
`hybrid_rerank_search` 的"rerank" = cosine 线性加权 (`combined = score*0.7 + emb*0.3`)，
无 cross-encoder、无 LLM 打分。Advanced RAG 的核心优势未兑现。

### B8. 检索↔生成断裂，signals 字段闲置
`SearchResult.signals` (`nt_memory_types`) 已设计但全链路为 `None`；
无"引用标注 + 可信度标注"输出；`AdaptiveRagResult` 不回流 KB。生成侧无法证明答案出处。

## 3. 目标架构：NT-KB Agentic RAG 闭环

核心思想：从"检索增强生成"进化到"**以 KB 为核心的 agentic 记忆检索循环**"，
把 NeoTrix 独有资产 (GWT 路由、E8 状态机、VSA 联想、SEAL 反馈) 接到五代 RAG 之上。

```
┌──────────────────────────────────────────────────────────────────┐
│                    GWT 注意力路由层 (替换 B1/B2 启发式)              │
│  查询 → 意图广播 → 各域 agent 共振响应 → 5 通道选择                    │
│  ├─ 事实性(Simple)    → 快速通道 FTS+BM25                         │
│  ├─ 语义性(Medium)    → 向量通道 embed+cosine+rerank              │
│  ├─ 关系性(多跳)      → 图通道 GraphRAG 子图扩展+社区               │
│  ├─ 分析性(Hard)      → Agent 循环 (E8 状态机: 规划→检索→反思→重试) │
│  └─ 对比/综述         → 分解器 map-reduce 并行子查询               │
└──────────────────────────┬───────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│                 检索编排器 Retrieval Orchestrator (新)             │
│  1. 多路召回: FTS5 │ BM25 │ Vector │ Walsh │ Graph(PPR) │ Community │
│  2. 统一融合: RRF + 可学习权重 (替代硬编码, B4)                     │
│  3. Rerank: cross-encoder 或 LLM 打分 (B7)                        │
│  4. 反馈评估: CRAG 分级 → 结果回流 KB (引用标注, B8)                │
└──────────────────────────┬───────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│    VSA HyperCube 联想层 (NeoTrix 独有)                             │
│    查询概念编码 → 激活关联概念 → 概念扩召回 → 注入 orchestrator       │
└──────────────────────────┬───────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│            生成上下文构建器 → LLM 生成 → 答案+引用                   │
└──────────────────────────┬───────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│           SEAL 反馈回流闭环 (B4): 采纳/弃用 → 权重自适应            │
│           + ConfidenceStore 更新 + access_count 提升               │
└──────────────────────────────────────────────────────────────────┘
```

### 3.1 GWT 路由层 (替换 `classify_query`)

```rust
// 现有: heuristic_classify (词法计数)
// 目标: GWT 广播 → 各域 agent 以 QueryIntent 共振响应 → 加权表决
pub struct QueryIntent {
    pub channel: RetrievalChannel,   // Fast | Vector | Graph | AgentLoop | Decompose
    pub complexity: QueryComplexity,
    pub confidence: f64,
    pub resonance: HashMap<DomainId, f64>, // 各域共振强度 (GWT 广播记录)
}
```

设计要点：
- 复用 `nt_core_self::AttentionManager` 的路由机制（Weapon Set 切换），
  CORE(推理) + WORLD(感知) + MEMORY(检索) 三域共振表决
- 不做 LLM 调用（保持廉价），用 **域信号 + 轻量语义特征** 替代纯词法
- 中英文查询处理：去停用词用 Unicode 感知 (`normalize_lang` 已有)

### 3.2 E8 Hexagram 查询状态机 (Agent 循环)

把 agent 检索循环映射到 E8 64 卦，每步 = 卦象状态转移：

| 阶段 | 卦象映射 | 行为 |
|------|---------|------|
| 初始理解 | 乾(☰) 纯阳 | 意图解析、信道选择 |
| 首轮检索 | 需(䷄) 待哺 | 多路召回 + 融合 |
| 相关判定 | 明夷(䷣) 察微 | 相关性分级 (替换 term 匹配) |
| 全部相关 | 泰(䷊) 通泰 | Generate — 直接生成 |
| 部分相关 | 革(䷰) 变革 | Rewrite + 重检索 (语义改写) |
| 全不相关 | 屯(䷂) 初创 | WebSearch 兜底 或 图通道补捞 |
| 图路径推进 | 渐(䷴) 渐进 | 实体提取 → 图扩展 → 下一跳 |
| 结果收敛 | 既济(䷾) 完成 | 反馈回流 + 引用标注 |

E8 是 NeoTrix 独有优势：状态转移即推理轨迹，可审计、可复现。

### 3.3 VSA HyperCube 联想扩召回

```rust
// 查询 → VSA 编码 → 关联概念召回 → 作为扩展查询词注入
pub fn vsa_associative_expansion(&self, query: &str) -> Vec<String> {
    let cube = self.hypercube.read();
    let q_vec = cube.encode(query);               // VSA embedding (符号)
    cube.associative_recall(&q_vec, 8)             // 近邻概念
        .into_iter().map(|c| c.term).collect()
}
```
注意: 区分 **KB embedding** (向量存储, 余弦) 与 **VSA embedding** (符号超向量, 关联)。
VSA 通道做**概念扩召**, KB 向量通道做**语义召回** — 两条互补, 不混淆 (CONTEXT.md 消歧)。

### 3.4 可学习融合权重 (SEAL 反馈闭环)

```rust
// 替换硬编码 fuse_weights:
// 反馈记录表 (SQLite): (query_family, strategy, adopted, rejected, latency_ms)
// 周期聚合 → 在线调整权重 (指数滑动平均)
pub struct FeedbackSignal {
    pub query_family: String,       // 意图家族 (e.g. "relation-multi-hop")
    pub strategy: RetrievalChannel,
    pub adopted_ids: Vec<String>,   // 生成采纳的节点
    pub rejected_ids: Vec<String>,  // 分级为 Irrelevant 的节点
    pub latency_ms: u64,
}
```
聚合规则：采纳率高 → 该通道权重上调；弃用率高 → 下调。上限/下限钳制防震荡。

### 3.5 真正的多跳 (图路径推理)

每跳提取**实体集** (复用 `graphrag_extract`)，下一跳查询 = 实体 + 关系约束，
而非拼接原查询。配合 `GraphCache` 邻居扩展做 2-3 跳受限遍历 (防指数爆炸, 带衰减系数)。

### 3.6 查询分解器 (map-reduce)

```rust
pub enum Decomposition {
    Flat(Vec<String>),          // 并行子查询 (无依赖)
    Sequential(Vec<String>),    // 依赖链 (多条件推理)
}
```
Hard 查询 → 规则+LLM 混合分解 → 子查询并行 `par_iter` → 结果 merge 去重 → 统一 Rerank。

## 4. 落地接线点 (现有代码最小侵入)

| 新组件 | 接线位置 | 说明 |
|--------|---------|------|
| `nt_memory_gwt_router.rs` (新) | `mod.rs:1071 search()` 入口之前 | 意图路由决策 |
| `nt_memory_orchestrator.rs` (新) | `mod.rs` 检索链 | 多路召回编排器 |
| `nt_memory_feedback.rs` (新) | `execute_pipeline` 尾部 + `search()` | 反馈回流闭环 |
| `nt_memory_decompose.rs` (新) | `AdaptiveRetrieval::execute_pipeline` | 查询分解器 |
| E8 状态机 | `nt_memory_adaptive_rag.rs:195` 重写循环 | 卦象状态转移 |
| Graph 信号接入 | `search()` 融合链 + `fuse_signals` | 社区/子图入 RRF |
| `signals` 字段激活 | `SearchResult.signals` | 引用标注+可信度 |

## 5. 实施路线图 (C1→C4)

- **C1 (本阶段)** — `nt_memory_gwt_router` + `nt_memory_feedback` 表结构:
  - TDD: 意图路由 5 通道单测 + 反馈回流 CRUD 单测
  - 验证: `cargo test -p neotrix --lib` 全绿
- **C2** — 查询分解器 + 并行子检索:
  - TDD: 分解器规则单测 + 并行合并正确性
  - 验证: Hard 查询召回率对比基准提升
- **C3** — Graph 信号接入主检索 + 可学习权重:
  - TDD: 融合权重调整收敛性单测 (模拟反馈序列)
  - 验证: 社区检索/子图扩展进入 unified entry
- **C4 (已完成)** — E8 状态机 Agent 循环 + VSA 扩召:
  - TDD: 卦象状态转移合法性 (E8 转移表单测) — `nt_memory_e8_agent.rs` 12 tests
  - TDD: VSA 联想相似度/扩召 — `nt_memory_vsa_expand.rs` 7 tests
  - T3: `search()` 入口 GWT 路由 + VSA 扩召; `search_agentic()` E8 驱动 + 反馈回流
  - 验证: 全量回归 6524 passed (2 flaky 时序测试单独运行全过)

## 6. 验收标准

1. `search()` 保留向后兼容 (旧调用不改签名)，新路由为增量增强
2. 每条检索结果 `signals` 携带来源通道 + 置信度 (B8 闭合)
3. 反馈闭环运行 N 次后融合权重收敛且钳制在 [0.05, 0.7]
4. 多跳查询 (2-hop) 较现 `iterative_retrieval` 命中率提升 (有基准)
5. 全链路 `cargo check --all-targets` + `cargo test` 双绿 (R-P 纪律)

## 7. 风险与未决

- **R1**: LLM 相关性打分 (B2/B7) 依赖 embedding 端点可用性 → 无端点时退化为启发式分级 (保留现有)
- **R2**: 并行分解检索的锁竞争 → 只读快照 (`clone_connection`) 按查询隔离
- **R3**: 图遍历爆炸 → 跳数钳制 3 + 衰减系数 0.5 + GraphCache 命中优先
- **R4**: E8 状态机复杂度 → 先做 8 态主链, 不追求 64 态全量
