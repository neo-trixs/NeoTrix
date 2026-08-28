# NeoTrix Rust 并行 Session 启动指引

## 概览

需要并行启动 4 个独立 Session，每个处理一个核心 Rust 接线任务。所有任务共享同一个 KB (`~/.neotrix/knowledge.db`)，但修改不同的代码模块。

## Session 分配

| Session | 任务 | 核心文件 | 预估工时 |
|---------|------|----------|----------|
| **A** | task-cog-meta | `nt_core_meta/` (10文件, 3.5K行) | 2-3 Session |
| **B** | task-cog-consciousness | `consciousness/` (~12K行) | 3-4 Session |
| **C** | task-cog-reasoning | `nt_core_reasoning.rs` + `reasoning_engine/` | 2 Session |
| **D** | task-awakening-p2b | `nt_memory_embed.rs` + MiniLM 服务 | 1 Session + infra |

---

## 共同前置条件

```bash
# 1. 克隆仓库 (每个 Session 独立工作树)
cd /Users/neo/Downloads
git worktree add ../neotrix-session-A main
git worktree add ../neotrix-session-B main
git worktree add ../neotrix-session-C main
git worktree add ../neotrix-session-D main

# 2. 进入各自目录
cd /Users/neo/Downloads/neotrix-session-A  # Session A
# cd /Users/neo/Downloads/neotrix-session-B  # Session B
# cd /Users/neo/Downloads/neotrix-session-C  # Session C
# cd /Users/neo/Downloads/neotrix-session-D  # Session D

# 3. 确认 KB 可访问
sqlite3 ~/.neotrix/knowledge.db "SELECT COUNT(*) FROM nodes;"
# 应返回 389740+
```

---

## Session A: task-cog-meta (元认知循环激活)

### 目标
连接 `knowledge_gap_reports` (68条) + `node_dimensions` 抽象层级到 `knowledge_gap_detector` 和 `planner`

### 核心文件
```
neotrix-core/src/core/nt_core_meta/
├── mod.rs                      # 入口
├── knowledge_gap_detector.rs   # 核心：需读取 KB
├── planner.rs                  # 核心：需读取 KB  
├── meta_cognition_loop.rs      # 主循环
└── ...
```

### 关键接口设计

```rust
// nt_core_meta/src/knowledge_gap_detector.rs
pub struct KnowledgeGapDetector {
    kb: Arc<KnowledgeBase>,
}

impl KnowledgeGapDetector {
    pub async fn detect_gaps(&self) -> Result<Vec<GapReport>> {
        // 1. 读取 knowledge_gap_reports 表 (status='pending')
        // 2. 基于 node_dimensions 抽象层级分析缺失层级
        // 3. 基于 embeddings 计算连通度 < 阈值的节点
        // 4. 生成新的 gap 报告写回 KB
    }
    
    pub async fn analyze_abstraction_coverage(&self, domain: &str) -> AbstractionCoverage {
        // 查询各 abstraction 层级的节点数量
        // 识别缺失层级 (如 case_study 有但 architecture 少)
    }
}
```

### 验收标准
- [ ] `cargo test -p neotrix --lib nt_core_meta` 通过
- [ ] 能从 KB 读取 `knowledge_gap_reports` (68条)
- [ ] 能分析 `node_dimensions` 抽象层级分布
- [ ] 能写入新的 gap 报告到 KB

---

## Session B: task-cog-consciousness (ConsciousnessTree 激活)

### 目标
激活 ConsciousnessTree 11 分支，注入健康指标、KB embeddings 数据流

### 核心文件
```
neotrix-core/src/core/
├── nt_core_consciousness_core.rs    # 核心 (12K行)
├── nt_core_consciousness_review.rs  # 审查模块
├── consciousness/
│   ├── awakening.rs                 # 觉醒流程
│   ├── stream_buffer.rs             # 流缓冲
│   ├── curiosity_drive.rs           # 好奇心驱动
│   └── branches/                    # 11 分支实现
```

### 关键接口设计

```rust
// consciousness 分支节点已在 KB 中 (consciousness://branch/NT-CORE 等)
pub struct ConsciousnessTree {
    kb: Arc<KnowledgeBase>,
    branches: HashMap<BranchId, BranchState>,
}

impl ConsciousnessTree {
    pub async fn awaken(&mut self) -> Result<()> {
        // 1. 从 KB 读取 11 个分支节点
        // 2. 注入 embeddings 作为初始状态
        // 3. 启动 6 阶段循环
    }
    
    pub async fn inject_kb_health(&mut self) -> Result<()> {
        // 从 KB 计算: 节点数/边数/嵌入覆盖率/连通度
        // 作为 health 指标注入各分支
    }
    
    pub async fn run_cycle(&mut self) -> Result<CycleReport> {
        // Soil -> Roots -> Trunk -> Branches -> Fruits -> Core
    }
}
```

### 验收标准
- [ ] `cargo test -p neotrix --lib consciousness` 通过
- [ ] 11 分支节点从 KB 正确加载
- [ ] 6 阶段循环至少跑通 1 个完整周期
- [ ] 健康指标正确计算并注入

---

## Session C: task-cog-reasoning (推理引擎连接)

### 目标
连接 `causal_rules` (170条) + `reasoning_chains` (30条) 到推理引擎

### 核心文件
```
neotrix-core/src/core/
├── nt_core_reasoning.rs          # 427行入口
├── reasoning_engine/
│   ├── forward.rs                # 前向推理
│   ├── backward.rs               # 反向推理
│   ├── abductive.rs              # 归因推理
│   └── chain_executor.rs         # 链执行器
```

### 关键接口设计

```rust
// reasoning_engine/chain_executor.rs
pub struct ChainExecutor {
    kb: Arc<KnowledgeBase>,
}

impl ChainExecutor {
    pub async fn execute_forward(&self, premise: &str, max_depth: u8) -> Result<ReasoningResult> {
        // 1. FTS 搜索 premise 相关节点
        // 2. 读取 causal_rules (condition -> outcome)
        // 3. 沿 leads_to / synthesizes / analogizes 边遍历
        // 4. 返回推理链 + 置信度
    }
    
    pub async fn execute_backward(&self, goal: &str) -> Result<ReasoningResult> {
        // 反向搜索: 从 goal 反推 premise
    }
    
    pub async fn execute_abductive(&self, observation: &str) -> Result<ReasoningResult> {
        // 归因推理: 找出最佳解释
    }
}

// 关键表映射
// causal_rules: condition, action, outcome, confidence
// reasoning_chains: chain_type, premise_ids, conclusion_id, rule_ids
```

### 验收标准
- [ ] `cargo test -p neotrix --lib reasoning` 通过
- [ ] 能执行前向/反向/归因三种推理模式
- [ ] 能读取 `causal_rules` (170条) 和 `reasoning_chains` (30条)
- [ ] 能沿 KB 边遍历 (`leads_to`, `synthesizes`, `analogizes`, `counterfactual_of`)

---

## Session D: task-awakening-p2b (Embedding 质量升级)

### 目标
部署 MiniLM 服务，重新生成全部 389,740 个节点的 embeddings

### 核心文件
```
neotrix-core/src/neotrix/l3_memory_impl/nt_memory_kb/
├── kb_vector_index.rs    # HNSW 索引集成点
├── nt_memory_embed.rs    # EmbeddingConfig::from_env()
└── kb_embedding.rs       # 批量嵌入生成
```

### 步骤

```bash
# 1. 启动 MiniLM 服务
cd /Users/neo/Downloads/neotrix/deploy/minilm
docker-compose up -d

# 2. 验证服务
curl http://localhost:8237/health
curl -X POST http://localhost:8237/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs": ["test"]}'

# 3. 设置环境变量
export NEOTRIX_EMBEDDING_ENDPOINT=http://localhost:8237
export NEOTRIX_EMBEDDING_API_KEY=

# 4. 运行重新嵌入脚本
python3 scripts/regenerate_all_embeddings.py
```

### 重新嵌入脚本需实现

```python
# scripts/regenerate_all_embeddings.py
# 1. 读取所有 nodes (分批)
# 2. 调用 MiniLM 服务批量生成 embeddings
# 3. 更新 embeddings 表
# 4. 重建 HNSW 索引 (kb_vector_index.rs)
# 5. 验证质量提升 (语义相似度测试)
```

### 验收标准
- [ ] MiniLM 服务正常运行 (port 8237)
- [ ] 389,740 个节点 embeddings 全部重新生成
- [ ] HNSW 索引重建成功 (`kb_vector_index.rs`)
- [ ] 语义相似度测试通过 (相似概念 cosine > 0.7)
- [ ] 向量搜索 p95 < 5ms (HNSW 生效)

---

## 协调机制

### 每日同步
- 时间: 每天 10:00
- 方式: 共享文档更新进度
- 内容: 完成项 / 阻塞项 / 下一步

### 冲突预防
- 每个 Session 修改不同模块目录
- KB 只读/追加，不删除现有数据
- 共享表: `knowledge_gap_reports`, `reasoning_chains`, `node_dimensions`

### 里程碑

| 周 | 里程碑 |
|----|--------|
| Week 1 | 4 个 Session 全部启动，核心接口定义完成 |
| Week 2 | Session A/D 完成，Session B/C 核心逻辑跑通 |
| Week 3 | 4 个 Session 全部通过测试，集成验证 |
| Week 4 | 性能调优 + 文档完善 + 合并主分支 |

---

## 快速启动命令

```bash
# 一键创建 4 个 worktree
cd /Users/neo/Downloads/neotrix
for s in A B C D; do
  git worktree add ../neotrix-session-$s main
done

# 并行启动 4 个终端
# Terminal 1: cd ../neotrix-session-A && code .
# Terminal 2: cd ../neotrix-session-B && code .
# Terminal 3: cd ../neotrix-session-C && code .
# Terminal 4: cd ../neotrix-session-D && code .
```
