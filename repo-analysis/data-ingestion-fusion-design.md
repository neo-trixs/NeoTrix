# 数据摄取层融合设计 v1

> 2026-05-28 | 基于架构扫描: 19 文件 ~6000 行 → 整合为 12 文件 ~4000 行

---

## 1. 全景差距图 (双向对比)

### 现有 6 个摄取入口

| 入口 | 位置 | 行数 | 获取 | 处理 | 存储 | 调用者 |
|------|------|------|------|------|------|--------|
| WebKnowledgeMiner | `reasoning_brain/web_miner.rs` | 639 | reqwest(自建) + GitHub API | 5 源类型检测 + keyword 分析 | CV + RBK | ExplorationPipeline |
| KnowledgeMiner | `reasoning_brain/knowledge_miner.rs` | 526 | git clone --depth=1 | 仓库分析(tech stack) | CV + RBK | ExplorationPipeline / KnowledgeChain |
| ExplorationPipeline | `reasoning_brain/exploration_pipeline.rs` | 679 | 委托 Web+Know Miner | 3 阶段+auto-goal | CV + RBK + KE | BackgroundLoop(mine_ticker) |
| UnifiedCrawler | `crawler/unified.rs` | 585 | FetcherPool(reqwest/自建) | Classifier+Mapper | CV + RBK + HC | BackgroundLoop(crawler_ticker) |
| SelfEvolver | `reasoning_brain/self_evolver.rs` | 541 | reqwest(自建) + git clone | tri-stream 分析 | CV + RBK + brain.json | 用户触发 |
| KnowledgeChain | `reasoning_brain/knowledge_chain.rs` | 349 | 委托 KnowledgeMiner | 6 阶段 | RBK | BackgroundLoop(mine_ticker) |

**CV = CapabilityVector, RBK = ReasoningBank, HC = HyperCube, KE = KnowledgeEngine**

### 重叠矩阵

```
                    HTTP  Git    API    URL分类  CV映射   Gap种子  编排器
WebKnowledgeMiner   🔴    🔴    🟢    🟢     🔴     —       —
KnowledgeMiner      —     🔴    —     —      —      —       —
ExplorationPipeline  —    —     —     🟢     —     🔴     🟢(编排)
UnifiedCrawler      🔴    —     —     🟢     🔴     —      🟢(编排)
SelfEvolver         🔴    🔴    —     —      —      —       —
KnowledgeChain       —    —     —     —      —      —      🟢(编排)

🔴 = 独立实现导致重复  🟢 = 概念重叠但可统一
```

---

## 2. 优先级矩阵

| ID | 整合项 | 影响(代码消除) | 风险 | 顺序依赖 |
|----|--------|---------------|------|----------|
| **F-01** | 合并 scraper.rs → fetcher.rs | -275 行, 消除 1 HTTP 实现 | 低: 文件级合并 | 无 |
| **F-02** | 提取共享 KnowledgeAbsorber | 消除 2 套 CV 映射表 | 中: 需改调用方 | F-01 后 |
| **F-03** | 合并 KnowledgeChain → ExplorationPipeline | -349 行, 消除 1 编排器 | 低: 委托模式 | F-02 后 |
| **F-04** | 统一 Git 克隆: SelfEvolver → KnowledgeMiner | 消除 1 套 git clone 实现 | 中: 需保留 tri-stream | F-02 后 |
| **F-05** | 统一缺口分析: GapAnalyzer | 消除 2 套 gap→种子逻辑 | 低: 新模块 | F-03 后 |
| **F-06** | 统一 URL 分类: 3套→1套 SourceClassifier | 消除 3 套分类枚举 | 中: API 变更 | F-01 后 |
| **F-07** | 默认激活 BackgroundLoop | — | 低: 配置项 | F-03 后 |
| **F-08** | 循环节点测试 | — | 低: 纯新增 | F-07 后 |

---

## 3. 详细设计

### F-01: 合并 scraper.rs → crawler/fetcher.rs

**策略**: 将 `AntiDetect` + `referer spoofing` 移植到 `FetcherPool`，`scraper.rs` 保留为薄兼容层(re-export)

**变更文件**:
- `crawler/fetcher.rs`: 添加 `anti_detect` 模块(UA 轮转 + 指纹), `google_get()` 方法
- `scraper.rs`: 保留 `pub use` 重导出，内部逻辑迁移
- 引用 `scraper.rs` 的文件(mcp_tools.rs, background_loop.rs)保持无感

**Key design**:
```rust
// FetcherPool 新增
pub fn google_get(&self, url: &str) -> FetchResult { ... }  // 添加 Google Referer
pub fn with_anti_detect(mut self) -> Self { ... }

// 内部 UA 池从 5 个扩展到 12 个
```

---

### F-02: 提取共享 KnowledgeAbsorber

**策略**: 从 `KnowledgeMapper::apply_to_brain()` 和 `WebKnowledgeMiner::mine_all()` 提取共性 → `knowledge_absorber.rs`

**变更文件**:
- 新增: `reasoning_brain/knowledge_absorber.rs` (单职责: 注册 KnowledgeSource + 应用 MicroEdit + 存入 ReasoningBank)
- 修改: `web_miner.rs` → 委托 `KnowledgeAbsorber`
- 修改: `crawler/mapper.rs` → `KnowledgeMapper` 使用 `KnowledgeAbsorber`

```rust
// knowledge_absorber.rs
pub struct KnowledgeAbsorber;

impl KnowledgeAbsorber {
    pub fn absorb(
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        source: KnowledgeSource,
        vector_deltas: Vec<(String, f64)>,  // dimension_name → delta
        memories: Vec<ReasoningMemory>,
    ) -> AbsorbResult;
    
    pub fn register_custom_source(
        brain: &mut ReasoningBrain,
        name: &str,
        vector: CapabilityVector,
    );
}
```

**集成点**:
- `WebKnowledgeMiner::mine_all()` → 收集 `vector_deltas` → 调用 `KnowledgeAbsorber::absorb()`
- `KnowledgeMapper::apply_to_brain()` → 收集 `vector_deltas` → 调用 `KnowledgeAbsorber::absorb()`

---

### F-03: 合并 KnowledgeChain → ExplorationPipeline

**策略**: `KnowledgeChain` 成为 `ExplorationPipeline` 的过期委托(shim)，逻辑移动后标记 `#[deprecated]`

**变更**:
- `ExplorationPipeline` 新增 `mine_round()` (现有 `KnowledgeChain::run_chain()` 逻辑)
- `KnowledgeChain::run_chain()` → 委托 `ExplorationPipeline::mine_round()`
- `background_loop.rs`: 仅保留 `exploration_pipeline`, 移除 `knowledge_chain` 字段

---

### F-04: 统一 Git 克隆

**策略**: 在 `KnowledgeMiner` 中暴露共享 `clone_repo()` 方法，`SelfEvolver` 委托之

**变更**:
- `knowledge_miner.rs`: `pub fn clone_repo(url, work_dir) -> Result<PathBuf>` (可复用)
- `self_evolver.rs`: `fetch_information()` 中 GitHub URL 走 `KnowledgeMiner::clone_repo()`
- 安全检查(anti-injection)仅保留一份

---

### F-05: 统一缺口分析: GapAnalyzer

**策略**: 新增 `reasoning_brain/gap_analyzer.rs`

```rust
pub struct GapAnalyzer;

impl GapAnalyzer {
    /// 检测 CapabilityVector 薄弱维度 → 生成种子 URL
    pub fn analyze_gaps(cap: &CapabilityVector) -> Vec<GapSeed>;
    
    /// 给 ExplorationPipeline 和 UnifiedCrawler 同时注入种子
    pub fn seed_all(
        exploration: &mut ExplorationPipeline,
        crawler: &Option<UnifiedCrawler>,
        brain: &ReasoningBrain,
    );
}

pub struct GapSeed {
    pub url: String,
    pub target_dimension: String,
    pub priority: f64,
    pub target_pipeline: PipelineType,  // Exploration | Crawler | Both
}
```

**集成点**:
- `ExplorationPipeline::auto_generate_goals()` → 委托 `GapAnalyzer::analyze_gaps()`
- `AttentionRouter::seed_crawler_from_gaps()` → 委托 `GapAnalyzer::seed_all()`
- `BackgroundLoop` 每 600s 调用 `GapAnalyzer::seed_all()`

---

### F-06: 统一 URL 分类

**策略**: 合并 3 套分类体系为 `SourceClassifier` (见可用枚举定义)

```rust
// core/knowledge/source_classifier.rs
pub enum SourceCategory {
    Wikipedia,       // 含子域: en/zh/fr/...
    ArXiv,
    GitHub,
    GenericWeb,
    PDF,
    News,
    Academic,
    Documentation,
    TorOnion,
    SocialMedia,
}
pub enum FetchProtocol { Http, Tor, Browser }

pub struct SourceClassifier;
impl SourceClassifier {
    pub fn classify(url: &str) -> (SourceCategory, FetchProtocol);
    pub fn to_crawl_topic(cat: &SourceCategory) -> CrawlTopic;
    pub fn to_task_type(cat: &SourceCategory) -> TaskType;
}
```

**集成点**: `WebSourceType`/`UnifiedSourceType`/部分 `CrawlTopic` 全部委托给 `SourceClassifier`

---

### F-07: 默认激活

**变更**:
- `BackgroundLoop::new()` 默认包含 crawler + exploration_pipeline
- `BackgroundConfig` 默认值: `enable_crawler = true`, `exploration_interval_secs = 600`
- 移除 `"[bg] crawler not configured"` 警告

---

### F-08: 循环节点测试

| 测试 | 验证内容 |
|------|----------|
| `test_fetcher_anti_detect` | UA 轮转 + referer spoofing |
| `test_knowledge_absorber_absorb` | 注册 source + apply edits + store memory |
| `test_gap_analyzer_seed_all` | gap 检测 + 双管道种子注入 |
| `test_exploration_round` | 完整 explore→mine→absorb 循环 |
| `test_crawler_cycle` | fetch→classify→map→absorb 闭环 |
| `test_background_loop_default` | 默认激活时 crawler + exploration 自动初始化 |
| `test_source_classifier_all` | 3 套旧分类一致性验证 |

---

## 4. 9层数据流 (整合后)

```
                                     ┌──────────────────────────────────┐
                                     │        KnowledgeAbsorber          │
                                     │  (注册知识源 + 维度delta + 存储)   │
                                     └────────┬─────────┬───────────────┘
                                              │         │
                    ┌─────────────────────────┘         └──────────────┐
                    ▼                                                   ▼
  ┌───────────────────────────────┐          ┌────────────────────────────┐
  │     ExplorationPipeline       │          │       UnifiedCrawler       │
  │  (唯一编排器: mine/explore)    │          │  (种子爬取管道)             │
  │  委托: WebMiner + KnowMiner   │          │  FetcherPool(AntiDetect)   │
  └───────────┬───────────────────┘          │  → SourceClassifier        │
              │                              │  → KnowledgeAbsorber       │
              ▼                              └───────────┬────────────────┘
  ┌──────────────────────┐                               │
  │     GapAnalyzer       │◄──────────────────────────────┘
  │  (统一缺口→种子生成)   │◄── AttentionRouter
  └──────────┬───────────┘
             │ seed_all(exploration, crawler)
             ▼
  ┌──────────────────────┐
  │    BackgroundLoop     │
  │  600s → GapAnalyzer  │
  │  600s → Exploration  │
  │  12h  → Crawler      │
  │  30s  → brain.json   │
  └──────────────────────┘
```

---

## 5. 文件变更清单

| 操作 | 文件 | 说明 |
|------|------|------|
| 新增 | `reasoning_brain/knowledge_absorber.rs` | ~60 行: 共享吸收逻辑 |
| 新增 | `reasoning_brain/gap_analyzer.rs` | ~80 行: 缺口分析 + 种子分发 |
| 新增 | `core/knowledge/source_classifier.rs` | ~50 行: URL 分类合并 |
| 大幅修改 | `crawler/fetcher.rs` | +60 行: AntiDetect + google_get |
| 大幅修改 | `background_loop.rs` | ~50 行: 默认激活 + GapAnalyzer 集成 |
| 修改 | `web_miner.rs` | -200 行: 委托 KnowledgeAbsorber |
| 修改 | `crawler/mapper.rs` | -100 行: 委托 KnowledgeAbsorber |
| 修改 | `exploration_pipeline.rs` | -100 行: 委托 GapAnalyzer + 吸收 KnowledgeChain |
| 修改 | `knowledge_miner.rs` | +20 行: 公开 clone_repo() |
| 修改 | `self_evolver.rs` | -50 行: 委托 clone_repo() |
| 修改 | `mod.rs`(reasoning_brain) | +2 行: 注册新模块 |
| 修改 | `mod.rs`(core/knowledge) | +1 行: 注册 source_classifier |
| 降级 | `scraper.rs` | 保留 re-export, 逻辑迁移 |
| 降级 | `knowledge_chain.rs` | 添加 `#[deprecated]` shim |
| 降级 | `attention_router.rs` | seed_crawler_from_gaps() 委托 GapAnalyzer |
| 新增 | `reasoning_brain/integration_tests.rs` | ~120 行: 循环节点测试 |

**总计**: ~5 新增 + ~10 修改 + ~2 降级 = ~17 文件, 净行数消减约 50%

---

## 6. 执行顺序

```
Phase 0: Fusion Design ✅ (本文档)
Phase 1: F-01 合并 scraper → fetcher (安全, 零级联)
Phase 2: F-02 提取 KnowledgeAbsorber (关键依赖)
Phase 3: F-03 合并 KnowledgeChain → ExplorationPipeline
Phase 4: F-05 GapAnalyzer (F-03 后, 编排器稳定)
Phase 5: F-04 统一 Git clone (可并行)
Phase 6: F-06 统一 URL 分类 (F-01 后)
Phase 7: F-07 默认激活
Phase 8: F-08 循环测试
```

每阶段 cargo check --lib 门控。
