# 专项排行榜 (2026-09-10)

## Rust 生态

| # | 来源 | Stars/指标 | 核心功能 | 架构模式 | NeoTrix 映射 | 优先级 |
|---|------|-----------|----------|----------|-------------|--------|
| 1 | [Awesome Rust](https://github.com/rust-unofficial/awesome-rust) | 59.3k ⭐ | Rust 生态资源精选列表，覆盖库/框架/工具分类 | 策展型索引 (curated index) | NT-CORE 生态扫描 | P1 |
| 2 | [crates.io](https://crates.io/crates) | 332K+ crates | Rust 官方包注册中心，依赖解析/版本管理 | Registry + 依赖图 | NT-MEMORY 知识索引 | P0 |
| 3 | [lib.rs](https://lib.rs/) | 332,278 crates | crates.io 轻量替代索引，分类浏览/排行/筛选 | 分类策展索引 | NT-MEMORY 知识检索 | P1 |
| 4 | [Are We Web Yet?](https://www.arewewebyet.org/) | — | Rust Web 开发生态状态：框架(DB/HTTP/ORM)成熟度评级 | 状态仪表盘 (status dashboard) | NT-IO Web 能力评估 | P2 |
| 5 | [Are We Async Yet?](https://areweasyncyet.rs/) | — | async/await 稳定化追踪：tokio/futures/smol 生态 | 特性追踪 (feature tracker) | NT-CORE 异步能力验证 | P2 |

### Rust 生态关键发现

- **核心 crate**: tokio (异步运行时), axum/actix-web (HTTP), serde (序列化), sqlx/diesel (DB)
- **ML 生态**: burn, candle, tract-onnx, ort (ONNX Runtime) — Rust ML 推理趋于成熟
- **WebAssembly**: wasm-bindgen, yew, leptos — 浏览器端 Rust 已可用
- **嵌入式**: embedded-hal, probe-rs, smoltcp — IoT/嵌入式 Rust 生态完善

---

## AI Agent 专项

| # | 项目 | Stars | 核心功能 | 架构模式 | NeoTrix 映射 | 优先级 |
|---|------|-------|----------|----------|-------------|--------|
| 6 | [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT) | 187.2k | 自主 AI Agent 平台：可视化构建器 + 45+ 集成 + Agent 运行时 | 平台化 Agent 编排 (Platform + Marketplace) | NT-ACT 自主行动 + NT-IO 集成网关 | P1 |
| 7 | [LangChain](https://github.com/langchain-ai/langchain) | 146.1k | Agent 工程平台：LLM 链/Agent/工具链/RAG | 链式抽象 + 生态集成 (Chain + LangGraph + LangSmith) | NT-MIND 抽象层 + NT-MEMORY RAG | P0 |
| 8 | [AutoGen](https://github.com/microsoft/autogen) | 60.9k | 多 Agent AI 框架 (⚠️ 维护模式)：消息传递/事件驱动/分布式运行时 | 分层 Agent 架构 (Core→AgentChat→Extensions) | NT-CORE 多 Agent 协作 | P2 (已迁移至 MAF) |
| 9 | [CrewAI](https://github.com/crewAIInc/crewAI) | 58.3k | 角色扮演多 Agent 编排：Crews(自主协作) + Flows(事件驱动) | 角色基 Agent + 事件驱动工作流 | NT-ACT Agent 编排 + NT-FEEL 角色模型 | P1 |
| 10 | [MetaGPT](https://github.com/FoundationAgents/MetaGPT) | 70.3k | 多 Agent 软件公司模拟：PM/架构师/工程师角色 + SOP 驱动 | SOP 即代码 (Code = SOP(Team)) | NT-MIND SOP 驱动进化 + NT-ACT 自动化 | P1 |

### AI Agent 关键洞察

| 维度 | 趋势 |
|------|------|
| **架构** | 从单体 Agent → 多 Agent 协作 → 平台化 Agent Marketplace |
| **控制** | Crews(自主) vs Flows(精确控制) 融合为最佳实践 |
| **Rust 空白** | Python 主导，Rust Agent 框架空白 — NeoTrix 机会 |
| **安全** | Safe RLHF 思路可迁移至 Agent 行为约束 |

---

## 知识图谱专项

| # | 项目 | Stars | 核心功能 | 架构模式 | NeoTrix 映射 | 优先级 |
|---|------|-------|----------|----------|-------------|--------|
| 11 | [Apache TinkerPop](https://github.com/apache/tinkerpop) | 2.1k | 图计算框架：Gremlin 遍历语言 + OLTP/OLAP + 多语言驱动 | 图遍历抽象层 (Gremlin + TinkerGraph) | NT-MEMORY 知识图谱查询 | P2 |
| 12 | [Neo4j](https://github.com/neo4j/neo4j) | 17.2k | 原生图数据库：Cypher 查询 + ACID + 高性能图遍历 | 原生图存储 + 声明式查询 | NT-MEMORY KB 图查询后端 | P0 |
| 13 | [ArangoDB](https://github.com/arangodb/arangodb) | 14.3k | 多模型数据库：文档 + 图 + KV + AQL 查询 + ArangoSearch | 多模型统一 (Document + Graph + KV) | NT-MEMORY 多模型存储 | P1 |
| 14 | [Weaviate](https://github.com/weaviate/weaviate) | 16.8k | 向量数据库：对象+向量存储 + 语义搜索 + RAG + 混合搜索 | 向量+标量混合 + 模块化向量化 | NT-MEMORY 向量检索 | P0 |
| 15 | [Qdrant](https://github.com/qdrant/qdrant) | 34.5k | 高性能向量搜索引擎：Rust 实现 + HNSW + 量化压缩 + 分布式 | Rust 向量引擎 + SIMD 加速 | NT-MEMORY 向量检索 (Rust 原生) | P0 |

### 知识图谱架构对比

```
图数据库谱系:
├── 原生图: Neo4j (Cypher, ACID)
├── 多模型: ArangoDB (AQL, 文档+图+KV)
├── 图计算: TinkerPop (Gremlin, OLTP+OLAP)
└── 向量搜索: Weaviate (GraphQL, 混合) / Qdrant (REST/gRPC, Rust)
```

### NeoTrix KB 选型建议

| 需求 | 推荐 | 理由 |
|------|------|------|
| 本地知识图谱 | SQLite KB (现有) + Neo4j (可选) | NeoTrix 已有 SQLite，轻量场景够用 |
| 向量语义检索 | Qdrant (Rust 原生) | 与 NeoTrix 技术栈一致，性能最优 |
| 混合搜索 (BM25+向量) | Weaviate | 内置 hybrid search，RAG 友好 |
| 图遍历分析 | Neo4j + Gremlin | 复杂关系推理 |

---

## 强化学习专项

| # | 项目 | Stars | 核心功能 | 架构模式 | NeoTrix 映射 | 优先级 |
|---|------|-------|----------|----------|-------------|--------|
| 16 | [Gymnasium](https://github.com/Farama-Foundation/Gymnasium) | 12.5k | RL 标准 API：环境接口 + 参考环境集 + 多环境族 (Classic/MuJoCo/Atari) | 标准接口 + 可插拔环境 | NT-MIND SEAL 环境抽象 | P0 |
| 17 | [Ray RLlib](https://github.com/ray-project/ray) | 43.8k | 分布式 AI 计算引擎：RL + 训练 + 调参 + 推理 + 服务 | 分布式 Actor + 任务调度 | NT-ACT 分布式执行 + NT-PHYSICAL 计算调度 | P1 |
| 18 | [Stable Baselines3](https://github.com/DLR-RM/stable-baselines3) | 13.8k | 可靠 RL 算法实现：PPO/SAC/DQN/TD3/A2C + PyTorch | sklearn 风格 API + 可组合策略 | NT-MIND 算法参考实现 | P1 |
| 19 | [Safe RLHF](https://github.com/PKU-Alignment/safe-rlhf) | 1.6k | 安全 RLHF：约束价值对齐 + 惩罚成本模型 + 安全偏好数据集 | 受限优化 (reward max + cost constraint) | NT-SHIELD 安全约束 + NT-FEEL 价值对齐 | P2 |

### RL 关键洞察

| 维度 | 洞察 |
|------|------|
| **Gymnasium** | RL 领域的 "HTTP" — 标准接口定义，所有算法/环境必须兼容 |
| **Ray** | 分布式 RL 计算的事实标准，RLlib 可扩展至数千环境并行 |
| **SB3** | 研究级 RL 的 "pandas" — 简单 API + 可靠实现 + 丰富文档 |
| **Safe RLHF** | LLM 安全对齐前沿 — 约束优化思路可迁移至 NeoTrix Agent 安全 |

### NeoTrix SEAL × RL 映射

```
SEAL Pipeline (NeoTrix)          RL Concepts (外部)
─────────────────────          ─────────────────────
探索阶段 (Explore)         ←→  环境交互 (Env.step)
蒸馏阶段 (Distill)         ←→  策略评估 (Policy.eval)
自测阶段 (SelfTest)        ←→  奖励信号 (Reward)
吸收阶段 (Absorb)          ←→  策略更新 (Policy.update)
失败重试 (Retry)           ←→  探索-利用 (Explore-Exploit)
```

---

## 跨专项优先级总结

| 优先级 | 项目 | 领域 | 理由 |
|--------|------|------|------|
| **P0** | Qdrant | 向量数据库 | Rust 原生，与 NeoTrix 技术栈一致，KB 向量检索核心 |
| **P0** | Neo4j | 图数据库 | 行业标准图数据库，复杂关系推理 |
| **P0** | Weaviate | 向量+混合搜索 | RAG 场景首选，hybrid search 内置 |
| **P0** | LangChain | AI Agent | Agent 工程事实标准，生态最广 |
| **P0** | Gymnasium | RL API | SEAL 环境抽象参考 |
| **P1** | CrewAI | AI Agent | Crews+Flows 双模式，角色化 Agent |
| **P1** | MetaGPT | AI Agent | SOP 驱动，软件公司模拟 |
| **P1** | Ray | 分布式计算 | RLlib + 分布式执行，可扩展 |
| **P1** | SB3 | RL 算法 | 可靠实现，研究基线 |
| **P1** | ArangoDB | 多模型 DB | 文档+图+KV 统一，灵活 |
| **P2** | Safe RLHF | 安全对齐 | Agent 安全约束参考 |
| **P2** | TinkerPop | 图计算 | Gremlin 标准，OLTP+OLAP |
| **P2** | AutoGen | 多 Agent | 已迁移 MAF，仅参考架构 |
| **P2** | AutoGPT | Agent 平台 | Marketplace 模式参考 |
