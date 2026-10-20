# NeoTrix Evolution Roadmap v2 — 完整实现设计

**版本**: 2.0 | **创建**: 2026-07-04 | **状态**: 设计就绪
**来源**: v1 路线图执行反馈 + Cycle 14-15 GitHub 吸收 (graphify/Understand-Anything/Astrapai/obra/kbexplorer-cli)
**KB 锚点**: `kv_store:knowledge_architecture/evolution_roadmap` (v2.0 JSON)

---

## 阶段总览

```
      P0                        P1                           P2
  ┌──────────┐   ┌──────────────────────────────┐   ┌──────────────────┐
  │ Phase 1  │   │ Phase 3   Phase 4   Phase 5  │   │ Phase 6  Phase 7│
  │ 质量管线  │   │ 公共服务    可观测     可视化  │   │ 自吸收   竞品补齐 │
  │ (70%)    │   │ (30%)    (0%)     (60%)     │   │ (0%)    (0%)    │
  └────┬─────┘   └──────────────┬───────────────┘   └────────┬─────────┘
       │                        │                            │
  ┌────▼────────────────────────▼────────────────────────────▼─────────┐
  │                      Phase 8: KG 深度优化 (P1)                     │
  │   力导向布局 + 社区检测 + 语义搜索 + MCP server (0%)                │
  └─────────────────────────────────────────────────────────────────────┘
  ┌─────────────────────────────────────────────────────────────────────┐
  │                      Phase 9: CLI 知识工具 (P1)                     │
  │   /kb find / cluster / central / serve / export (0%)                │
  └─────────────────────────────────────────────────────────────────────┘
```

### 依赖图

```
Phase 1 (质量管线) ──→ Phase 6 (自吸收) ──→ Phase 7 (竞品补齐)
       │
       ├──→ Phase 8 需要 Phase 5 的 KG 可视化后端
       │
Phase 2 (上下文压缩) ──→ 独立，无外部依赖
       │
Phase 3 (公共服务) ──→ 依赖 Phase 1 的 quality_level
       │
Phase 4 (可观测) ──→ 依赖 Phase 1 的 CheckReflectStage
       │
Phase 5 (KG 可视化) ══→ Phase 8 (KG 深度优化) ══→ Phase 9 (CLI 工具)
```

---

## Phase 1: 内容质量管线 (P0, 70%→100%)

### 当前状态
- ✅ `KnowledgeQualityStage` 4 维复合评分 (content 40% + summary 20% + diversity 20% + edge_ratio 20%)
- ✅ `/kb stats` CLI — SQL 直接查询 nodes 表
- ⏳ `quality_level` 字段 — 未实现
- ⏳ CO-STAR 初筛门 — 未实现
- ⏳ `CheckReflectStage` — 未实现

### 设计

#### 1a: quality_level 字段

**方案**: 在 `nodes` 表增加 `quality_level TEXT NOT NULL DEFAULT 'legacy'` 列。
4 级制: `raw` (新摄入) → `scored` (经质量评分) → `verified` (人工/LLM 验证) → `curated` (高置信度长期保留)

```
nodes 表变更:
  + quality_level TEXT NOT NULL DEFAULT 'legacy'
  + quality_score REAL DEFAULT 0.0
  + verified_by TEXT                -- "auto" | "human" | "llm:<model>"
  + verified_at INTEGER             -- unix timestamp
```

**迁移**: `ALTER TABLE nodes ADD COLUMN quality_level TEXT NOT NULL DEFAULT 'legacy'`
存量节点保持 `legacy` — 在 `/kb stats` 中可筛选统计。

#### 1b: CO-STAR 初筛门

**Context-Objective-Style-Audience-Response** 快速评估框架。在 `ExternalKnowledgeAbsorbStage` 吸收新节点后、入库前执行。

```rust
// neotrix/l3_memory_impl/nt_memory_kb/nt_memory_quality_gate.rs
pub struct CoStarGate {
    enabled: bool,
    threshold: f64,         // 默认 0.3
    llm_coach: Option<Box<dyn LlmProvider>>,  // 可选 LLM 精筛
}

impl CoStarGate {
    pub fn quick_score(&self, content: &str) -> f64;
    pub fn llm_score(&self, content: &str, context: &str) -> Result<f64>;
    pub fn gate(&self, content: &str) -> GateVerdict; // Accept/Reject/Review
}
```

**文件位置**: `neotrix/l3_memory_impl/nt_memory_kb/nt_memory_quality_gate.rs`

#### 1c: CheckReflectStage

新的 SEAL pipeline stage，每次迭代后执行自我反思检查。

```rust
// neotrix/l8_autonomic_impl/nt_mind/self_iterating/stages/check_reflect_stage.rs
pub struct CheckReflectStage {
    interval: u32,          // 每 N 次迭代执行一次
    check_focus: Vec<CheckFocus>, // 检查维度
}

enum CheckFocus {
    QualityTrend,           // quality_score 趋势
    CoverageGap,           // 类型/领域覆盖缺口
    Consistency,           // 跨节点一致性
    Staleness,             // 节点陈旧度
}
```

**注册**: `pipeline.rs` 中 `seal_pipeline()` 追加 `CheckReflectStage` (freq=15)

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 1.1 | `nodes` 表 ALTER TABLE + quality_level 字段 | `nt_memory_schema.rs` + 迁移脚本 | 30 |
| 1.2 | KnowledgeQualityStage 改为写 quality_level | `pipeline.rs` | 40 |
| 1.3 | CoStarGate 结构体 + quick_score 启发式评分 | `nt_memory_quality_gate.rs` | 120 |
| 1.4 | CoStarGate.llm_score — 可选 LLM 精筛 | `nt_memory_quality_gate.rs` | 80 |
| 1.5 | CheckReflectStage — 质量趋势 + 覆盖检查 | `check_reflect_stage.rs` | 150 |
| 1.6 | pipeline.rs 注册 CheckReflectStage | `pipeline.rs` | 10 |
| 1.7 | `/kb stats --by-quality` 筛选统计 | `kb_cmds.rs` (CLI) | 30 |
| 1.8 | 测试: CoStar 评分 + quality_level 迁移 + CheckReflect | 对应文件 `mod.rs` | 200 |

**总计**: ~660 LOC

---

## Phase 2: 上下文压缩管道 (P0, 0%→100%)

### 目标

E8/GWT 上下文不可控增长 → 60-95% 压缩率，支持有损/可逆/无损三级。

### 架构

```
                   ┌──────────────────────────────┐
                   │         Compressor            │
                   │   (trait + registry + chain)  │
                   ├──────────────────────────────┤
                   │  compress(context) → Output   │
                   │  decompress(output) → Context │
                   │  can_compress(context) → bool │
                   └──────┬───────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
  │ TextCompress │ │ CodeCompress │ │ SmartCrusher │
  │ (LLM摘要)    │ │ (AST 缩减)   │ │ (启发式裁剪)  │
  │ 可逆         │ │ 可逆         │ │ 有损         │
  │ 60-80%       │ │ 70-90%       │ │ 80-95%       │
  └──────────────┘ └──────────────┘ └──────────────┘
```

### 文件布局

```
nt_core_gwt/
├── compressor.rs              — Compressor trait + CompressionLevel + Output
├── text_compressor.rs         — TextCompressor (LLM-based summarization)
├── code_compressor.rs         — CodeCompressor (AST extents → reduced)
├── smart_crusher.rs           — SmartCrusher (heuristic pruning)
├── cache_aligner.rs           — CacheAligner (KV cache hit rate optimization)
└── compaction.rs              — 现有: CompactionPipeline 集成点
```

### 核心类型

```rust
pub enum CompressionLevel {
    Lossless,       // 0% loss, < 30% compression
    Reversible,     // 可恢复, 60-80% compression
    Aggressive,     // 80-95%, 不可逆
}

pub struct CompressedOutput {
    pub data: Vec<u8>,
    pub level: CompressionLevel,
    pub original_size: usize,
    pub compressed_size: usize,
    pub reversible: bool,
    pub decompress_fn: Option<Box<dyn Fn(&[u8]) -> Result<String>>>,
}

pub trait Compressor: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_compress(&self, context: &str) -> bool;
    fn compress(&self, context: &str, level: CompressionLevel) -> Result<CompressedOutput>;
    fn decompress(&self, output: &CompressedOutput) -> Result<String>;
}
```

### CacheAligner

优化 GWT 中 KV cache 的 hit rate:
- 通过滑动窗口检测重复前缀
- 自动对齐 cache key 到 token 边界
- 压缩后重新计算 cache key

```rust
pub struct CacheAligner {
    window_size: usize,       // 默认 64 tokens
    hit_rate_threshold: f64,  // 默认 0.3
}
```

### GWT Compaction 集成

`CompactionPipeline` 中在现有 compaction 逻辑前插入压缩阶段:

```rust
// compaction.rs
pub struct CompactionPipeline {
    stages: Vec<Box<dyn CompactionStage>>,
    compressors: Vec<Box<dyn Compressor>>,
    aligner: CacheAligner,
}
```

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 2.1 | Compressor trait + CompressionLevel + CompressedOutput | `compressor.rs` | 100 |
| 2.2 | SmartCrusher: 启发式裁剪 (低信息密度段/重复/过时) | `smart_crusher.rs` | 200 |
| 2.3 | TextCompressor: LLM-based 摘要压缩 | `text_compressor.rs` | 180 |
| 2.4 | CodeCompressor: AST 感知代码压缩 | `code_compressor.rs` | 200 |
| 2.5 | CacheAligner: 滑动窗口 + key 对齐 | `cache_aligner.rs` | 150 |
| 2.6 | CompactionPipeline 集成压缩阶段 | `compaction.rs` | 80 |
| 2.7 | 测试: 各压缩器压缩率 + 可逆性 + 性能基准 | 对应文件 `mod.rs` | 250 |

**总计**: ~1160 LOC

---

## Phase 3: 知识公共服务层 (P1, 30%→100%)

### 当前状态
- ✅ CLI `/kb stats` / `/kb search` / `/kb explore` — 已实现
- ✅ Tauri `get_knowledge_graph` / `get_knowledge_stats` — 已实现
- ⏳ KB OpenAPI v1 端点 — 未实现
- ⏳ SKILL.md 安装 — 未实现
- ⏳ 内容隔离 + API Key 鉴权 — 未实现

### 设计

#### 3a: KB OpenAPI v1

在现有 `nt_io_server` 基础上新增 `kb/v1/` 路由组:

```
GET  /kb/v1/search?q=&types=&limit=20    → JSON search results
GET  /kb/v1/query?node_id=               → single node + relations
GET  /kb/v1/discover?seed=&depth=2       → BFS subgraph
GET  /kb/v1/topics                       → topic cluster summary
GET  /kb/v1/stats                        → KB stats
POST /kb/v1/read?node_id=               → subscribe to node updates (SSE)
```

#### 3b: SKILL.md

创建 `SKILL.md` 文件，安装步骤简化到极致:

```markdown
# NeoTrix Knowledge Base Skill

安装:
```bash
neotrix config set kb.enabled=true
```

使用: `@neotrix What do you know about X?`

命令:
- `/kb search <query>` — FTS5/BM25 混合搜索
- `/kb explore <node_id>` — 节点探索
- `/kb stats` — KB 统计
```

#### 3c: 内容隔离

`quality_level` 字段区分公共/私有:
- `public` = curated + verified 节点 → OpenAPI 可读取
- `private` = raw + legacy → 仅 CLI 本地访问

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 3.1 | `/kb/v1/` 路由组 + search/query/discover/topics/stats 端点 | `nt_io_server/kb_routes.rs` | 250 |
| 3.2 | SSE read 订阅端点 | `nt_io_server/kb_routes.rs` | 80 |
| 3.3 | API Key 鉴权中间件 | `nt_io_server/auth_middleware.rs` | 100 |
| 3.4 | quality_level 公共/私有隔离逻辑 | `nt_memory_store.rs` | 60 |
| 3.5 | SKILL.md 文件 | `SKILL.md` | 40 |
| 3.6 | 测试: OpenAPI 端点 + 鉴权 + 隔离 | 对应文件 `mod.rs` | 200 |

**总计**: ~730 LOC

---

## Phase 4: SEAL Pipeline 可观测 + 动态组合 (P1, 0%→100%)

### 设计

#### 4a: Tracing Span

每个 stage 执行时包裹 tracing span:

```rust
pub trait StageExt: Stage {
    fn execute_with_tracing(&mut self, brain: &mut SelfIteratingBrain) -> Result<StageResult> {
        let span = info_span!("stage", name = %self.name(), tick = %brain.tick_count);
        let _guard = span.enter();
        let start = Instant::now();
        let result = self.process(brain);
        let elapsed = start.elapsed();
        info!(elapsed_ms = %elapsed.as_millis(), status = %result.status());
        // 持久化到 kv_store
        brain.store_pipeline_log(self.name(), elapsed, &result);
        result
    }
}
```

#### 4b: Pipeline 日志持久化

```
kv_store key: pipeline_log:<stage_name>:<tick>
value: {
  "stage": "CheckReflectStage",
  "tick": 142,
  "elapsed_ms": 1250,
  "status": "completed",
  "quality_trend": 0.72,
  "errors": []
}
```

#### 4c: 按 stage 重跑

`/evolve rerun <stage_name>` — 从指定 stage 重跑，跳过之前已成功的 stage。

#### 4d: 动态 stage 注册

`pipeline.rs` 改从配置读取 stage 列表，而非硬编码 34 阶段:

```toml
# config.toml
[pipeline]
stages = ["SnapshotStage", "AutonomyGate", "MemoryRetrieval", ...]
disabled = ["DgmMetaEvolveStage"]
extra = ["CustomStage"]
```

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 4.1 | execute_with_tracing 宏 + info_span 包裹 | `pipeline.rs` | 60 |
| 4.2 | Pipeline 日志持久化到 kv_store | `pipeline.rs` | 80 |
| 4.3 | `/evolve rerun <stage>` CLI | `evolve_cmds.rs` | 80 |
| 4.4 | 动态 stage 注册 + 配置读取 | `pipeline.rs` + `config.rs` | 120 |
| 4.5 | 测试: tracing + rerun + 动态注册 | 对应文件 | 150 |

**总计**: ~490 LOC

---

## Phase 5: 知识图谱可视化 — 剩余 (P1, 60%→100%)

### 当前状态
- ✅ `KnowledgeGraphPage.tsx` — @xyflow/react 渲染
- ✅ 侧边栏 Graph 导航
- ✅ Tauri 后端 `get_knowledge_graph` / `get_knowledge_stats`
- ⏳ dagre 层次布局 + 力导向混合 — 未实现
- ⏳ 学习路径高亮 — 未实现
- ⏳ 一键导出 PNG/SVG — 未实现
- ⏳ 节点详情面板 — 未实现

### 设计

#### 5a: 混合布局

```typescript
// KnowledgeGraphPage.tsx
// @xyflow/react + dagre 布局
// 子图使用 force-directed (d3-force) 分散
// 大图 (86K 节点) 使用 dfg (degree-of-interest) 聚焦

// 三种布局模式:
type LayoutMode = 'dagre'       // 层次布局，适合结构清晰领域
                 | 'force'      // 力导向，适合社区发现
                 | 'radial'     // 辐射状，适合单节点关联
```

#### 5b: 节点详情面板

```typescript
// NodeDetailPanel.tsx
interface NodeDetail {
  id: string;
  node_type: string;
  title: string;
  content: string;
  summary: string;
  url: string | null;
  quality_level: string;
  quality_score: number;
  relations: { target_id: string; relation_type: string; target_title: string }[];
  created_at: number;
}
```

#### 5c: 学习路径高亮

BFS 最短路径可视化 + `@xyflow/react` 高亮 API。

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 5.1 | dagre 布局引擎集成 + LayoutMode 切换 | `KnowledgeGraphPage.tsx` | 150 |
| 5.2 | NodeDetailPanel 组件 (取代 alert) | `NodeDetailPanel.tsx` + `.module.css` | 200 |
| 5.3 | 学习路径高亮 (BFS path) | `KnowledgeGraphPage.tsx` | 100 |
| 5.4 | PNG/SVG 导出 (html-to-image) | `KnowledgeGraphPage.tsx` | 60 |
| 5.5 | 大图 dfg 聚焦 (86K 节点按需加载) | `KnowledgeGraphPage.tsx` + Tauri 后端 | 250 |
| 5.6 | 测试: 布局切换 + 详情面板 + 导出 | Playwright E2E | 100 |

**总计**: ~860 LOC

---

## Phase 6: 自吸收循环 (P2, 0%→100%)

### 设计

当前已有 `ExplorationEngine` + `ExternalKnowledgeAbsorbStage` (SEAL freq=20)，但缺少:
- 自动质量门控 (依赖 Phase 1)
- 跨源模式蒸馏 (依赖 Phase 1 content)
- MetaCognition 闭环

#### 自吸收循环

```
每 6 小时:
  1. ExplorationEngine.run_cycle() → GitHub trending + ArXiv + RSS
  2. ExternalKnowledgeAbsorbStage → 吸收新节点
  3. CoStarGate → quality_level 设置 (raw/scored/verified)
  4. ContentDistiller → 跨源模式洞察
  5. Panorama → 覆盖度报告
  6. MetaCognition → 循环总结写入 kv_store
```

### 任务清单 (依赖 Phase 1 完成后执行)

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 6.1 | ExplorationEngine + ExternalKnowledgeAbsorbStage 已有 — 确认集成 | — | 0 |
| 6.2 | CoStarGate 自动质量门控 (复用 Phase 1) | `nt_memory_quality_gate.rs` | 40 |
| 6.3 | ContentDistiller 定时调用 | `pipeline.rs` | 50 |
| 6.4 | Panorama 定期刷新 | `pipeline.rs` | 50 |
| 6.5 | MetaCognition 循环日志持久化 | `run.rs` | 80 |

**总计**: ~220 LOC

---

## Phase 7: 竞品功能补齐 (P2, 0%→100%)

### 设计

四项独立子任务:

| 子任务 | 文件 | 说明 |
|--------|------|------|
| ProcessMemoryStage | `nt_mind_seal/process_memory_stage.rs` | 成功 E8 模式序列→可重用技能 (已在 cycle 4 规划) |
| Skills Registry | `nt_mind_skill_engine.rs` + CLI | 社区技能市场，版本管理 |
| Agent Memory | `nt_memory_kb/agent_memory.rs` | 跨会话记忆持久化 |
| 桌面 UI 对话重构 | 前端 | 用户确认不添加 UI → 改为 CLI-first 策略 |

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 7.1 | ProcessMemoryStage: E8 轨迹→技能模板 | `process_memory_stage.rs` | 200 |
| 7.2 | Skills Registry: 版本管理 + 依赖解析 | `nt_mind_skill_engine.rs` | 250 |
| 7.3 | Agent Memory: 跨会话持久化 | `agent_memory.rs` | 200 |
| 7.4 | CLI `/skill registry` / `/skill publish` | `skill_cmds.rs` | 150 |

**总计**: ~800 LOC

---

## Phase 8: KG 深度优化 — 力导向 + 社区发现 (P1, 0%→100%)

### 目标

借鉴 graphify 74.8k⭐ / Understand-Anything 69.7k⭐ / Astrapai / obra/knowledge-graph，增加:
- 力导向布局 (react-force-graph 或 d3-force 集成)
- 社区检测 (Leiden/Louvain)
- 语义搜索 + PageRank
- MCP server 暴露图谱能力
- 学习路径高亮

### 架构

```
     ┌──────────────────────────────────────────────┐
     │            KnowledgeGraphEngine               │
     │  (nt_core_graph — core L4 纯计算)              │
     ├──────────────────────────────────────────────┤
     │  community::louvain()  → Vec<Community>       │
     │  community::leiden()   → Vec<Community>       │
     │  centrality::pagerank(nodes, edges) → Vec<f64> │
     │  centrality::betweenness(g) → Vec<f64>         │
     │  path::shortest_path(a, b) → Vec<NodeId>      │
     │  layout::force_directed(g) → Vec<Coord>       │
     └──────────────────┬───────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
   ┌──────────┐ ┌───────────┐ ┌──────────────┐
   │  Tauri   │ │    CLI    │ │   MCP Tool   │
   │ 渲染      │ │ /kb find  │ │ /graph query │
   │ 前端调用   │ │ /cluster  │ │ /community  │
   └──────────┘ └───────────┘ └──────────────┘
```

### 社区检测算法

**Louvain** (又快又好, obra 已验证):
1. 模块度优化: 将节点分配到使模块度增益最大的社区
2. 网络聚合: 将社区聚合为超节点
3. 重复 1-2 直到模块度不再增长

**Leiden** (Louvain 改进, graphify 使用):
1. 局部移动: 同 Louvain
2. 细化: 将社区拆分为子集
3. 聚合: 同 Louvain
4. 保证连通性 (Louvain 不保证)

### 文件布局

```
nt_core_hcube/
├── community.rs             — Louvain + Leiden 社区检测
├── centrality.rs            — PageRank + Betweenness Centrality
├── path.rs                  — BFS/DFS 路径查找
├── layout.rs                — Force-directed 布局坐标计算
└── graph_server.rs          — MCP server / graph query tool
```

### 前端集成

`KnowledgeGraphPage.tsx` 增加:

```typescript
// 社区着色
const communityColors = generateCommunityPalette(communities.length);
nodes.forEach(n => n.style = { background: communityColors[n.community_id] });

// 力导向布局切换
if (layoutMode === 'force') {
  // d3-force 模拟
  simulation = forceSimulation(nodes)
    .force('link', forceLink(edges))
    .force('charge', forceManyBody())
    .force('center', forceCenter(width, height));
}
```

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 8.1 | Louvain 社区检测算法 | `community.rs` | 250 |
| 8.2 | Leiden 社区检测算法 | `community.rs` | 300 |
| 8.3 | PageRank 中心性 | `centrality.rs` | 150 |
| 8.4 | Betweenness 中心性 | `centrality.rs` | 200 |
| 8.5 | BFS/DFS 最短路径 | `path.rs` | 120 |
| 8.6 | Force-directed 布局坐标 (基于 Fruchterman-Reingold) | `layout.rs` | 200 |
| 8.7 | Tauri 后端: community/centrality/path 命令 | `kb_cmds.rs` | 150 |
| 8.8 | MCP server: `/graph query` / `/graph community` | `graph_server.rs` | 200 |
| 8.9 | 前端: 社区着色 + 力导向布局集成 | `KnowledgeGraphPage.tsx` | 200 |
| 8.10 | CLI: `/kb find` / `/kb cluster` / `/kb central` | `kb_cmds.rs` | 150 |
| 8.11 | 测试: Louvain/Leiden/PageRank/BFS | `mod.rs` | 300 |

**总计**: ~2220 LOC

---

## Phase 9: CLI 知识工具完善 (P1, 0%→100%)

### 目标

借鉴 kbexplorer-cli / obra/knowledge-graph / graphify 的模式，完善 `/kb` 命令套件:

| 命令 | 借鉴自 | 功能 |
|------|--------|------|
| `/kb find <a> <b>` | obra path-find | 两节点间最短路径 |
| `/kb cluster [algo]` | obra community-detect | 社区检测 (Louvain/Leiden) |
| `/kb central [--algo]` | obra centrality | 中心性分析 (PageRank/Betweenness) |
| `/kb serve [--port]` | graphify serve | 启动 MCP server 暴露 KB 工具 |
| `/kb export [--format]` | kbexplorer-cli export | 导出子图为 JSON/SVG |

### 依赖

- `/kb find` + `/kb cluster` + `/kb central` 依赖 Phase 8 的 core 算法
- `/kb serve` 依赖 Phase 3 的 OpenAPI 端点
- `/kb export` 独立，仅需 Tauri 后端 `get_knowledge_graph`

### 命令接口设计

```rust
// CLI
#[derive(clap::Subcommand)]
pub enum KbCommand {
    Find {
        source: String,
        target: String,
        #[arg(long, default_value = "bfs")]
        algo: String,
    },
    Cluster {
        #[arg(long, default_value = "louvain")]
        algorithm: String,
        #[arg(long, default_value = "10")]
        min_community_size: usize,
    },
    Central {
        #[arg(long, default_value = "pagerank")]
        algorithm: String,
        #[arg(long, default_value = "20")]
        top_k: usize,
    },
    Serve {
        #[arg(long, default_value_t = 8337)]
        port: u16,
    },
    Export {
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        community_id: Option<usize>,
    },
}
```

### 任务清单

| # | 任务 | 文件 | 预计行数 |
|---|------|------|---------|
| 9.1 | `/kb find` — 路径查找 CLI (调用 Phase 8 path) | `kb_cmds.rs` | 60 |
| 9.2 | `/kb cluster` — 社区检测 CLI | `kb_cmds.rs` | 60 |
| 9.3 | `/kb central` — 中心性 CLI | `kb_cmds.rs` | 60 |
| 9.4 | `/kb serve` — MCP server CLI | `kb_cmds.rs` + `graph_server.rs` | 100 |
| 9.5 | `/kb export` — 导出 CLI | `kb_cmds.rs` | 80 |
| 9.6 | 测试: 各子命令 + 错误处理 | `kb_cmds.rs mod.rs` | 200 |

**总计**: ~560 LOC

---

## 全局统计

| Phase | 文件数 | 预计 LOC | 依赖 | 优先级 |
|-------|--------|----------|------|--------|
| 1: 质量管线 | 4 | 660 | — | P0 |
| 2: 上下文压缩 | 5 | 1160 | — | P0 |
| 3: 公共服务 | 3 | 730 | Ph1 quality_level | P1 |
| 4: SEAL 可观测 | 4 | 490 | Ph1 CheckReflect | P1 |
| 5: KG 可视化剩余 | 3 | 860 | — | P1 |
| 6: 自吸收循环 | 3 | 220 | Ph1 质量门 | P2 |
| 7: 竞品补齐 | 4 | 800 | — | P2 |
| 8: KG 深度优化 | 6 | 2220 | Ph5 后端 | P1 |
| 9: CLI 工具 | 2 | 560 | Ph8 算法 | P1 |
| **全体** | **~35** | **~7700** | — | — |

---

## 执行推荐顺序

### 子循环 1: P0 最小集 (Phase 1 + Phase 2)

```
Phase 1.1 quality_level ALTER TABLE → 1.2 评分写 level → 1.3 CoStarGate
Phase 2.1 Compressor trait → 2.2 SmartCrusher → 2.3 TextCompressor → 2.5 CacheAligner → 2.6 集成
并行: 1.5 CheckReflectStage + 1.7 CLI 统计
```

### 子循环 2: 可视化增强 (Phase 5 + Phase 8)

```
Phase 5.1 dagre 布局 → 5.2 节点详情面板 → 5.3 学习路径
Phase 8.1 Louvain → 8.3 PageRank → 8.5 BFS path → 8.7 Tauri 集成 → 8.9 前端社区着色
Phase 9.1 /kb find → 9.2 /kb cluster → 9.5 /kb export
```

### 子循环 3: 开放集成 (Phase 3 + Phase 4 + Phase 9)

```
Phase 3.1 OpenAPI 路由组 → 3.2 SSE → 3.3 API Key 鉴权
Phase 4.1 tracing span → 4.2 日志持久化 → 4.3 rerun CLI → 4.4 动态注册
Phase 9.4 /kb serve MCP
```

### 子循环 4: 自循环 (Phase 6 + Phase 7)

```
Phase 6.2 CoStar 质量门 → 6.3 蒸馏定时 → 6.4 全景刷新 → 6.5 元认知日志
Phase 7.1 ProcessMemoryStage → 7.2 Skills Registry → 7.3 Agent Memory
```

---

## 验收标准

| 指标 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 | Phase 6 | Phase 7 | Phase 8 | Phase 9 |
|------|---------|---------|---------|---------|---------|---------|---------|---------|---------|
| `cargo check` | 0 err | 0 err | 0 err | 0 err | — | 0 err | 0 err | 0 err | 0 err |
| `cargo clippy` | 0 warn | 0 warn | 0 warn | 0 warn | — | 0 warn | 0 warn | 0 warn | 0 warn |
| `cargo test` 增量 | +200 | +250 | +200 | +150 | +100 | +50 | +200 | +300 | +200 |
| 新增测试数 | >= 15 | >= 20 | >= 10 | >= 10 | >= 5 | >= 3 | >= 10 | >= 25 | >= 10 |
| 特有指标 | curated>=60 | 压缩率>=60% | CLI<=100ms | 100%追踪 | 加载<=3s | 周吸收>=50 | 注册>=20 | 检测<=5s | 全部实现 |

---

## 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| quality_level 用 ALTER TABLE | 非新表 | 存量 86K 节点不需要迁移数据到新表，加列 + 默认值即可 |
| Compressor trait 在 GWT 内 | 非独立 crate | 紧耦合 GWT compaction，减少跨 crate 接口 |
| 社区检测用原生 Rust | 非调用 Python | 86K 节点计算量在 Rust 单线程 < 5s，无需引入 Python 依赖 |
| Louvain 为主 + Leiden 可选 | 实现复杂度 vs 精度 | Louvain 足够 80% 场景，Leiden 作为 +Leiden feature gate |
| OpenAPI 用现有 server | 非新增独立 server | 复用 nt_io_server 的 HTTP/WS 基础设施 |
| SKILL.md 纯 CLI | 非 Tauri UI | 用户已确认不增加 UI，CLI-first |
| 力导向布局用 d3-force 内联 | 非 react-force-graph | 避免额外 dependency，d3-force 可在 @xyflow/react 内 custom node 使用 |
